#![no_std]

use aidoku::{
	Chapter, ContentRating, FilterValue, Manga, MangaPageResult, MangaStatus, Page, PageContent,
	Result, Source, UpdateStrategy, Viewer,
	alloc::{String, Vec, format, string::ToString, vec},
	imports::net::Request,
	prelude::*,
};

mod models;
mod settings;

use models::{Episode, Languages};

const SITE: &str = "https://www.peppercarrot.com";
const SOURCES: &str = "https://www.peppercarrot.com/0_sources";
const MANGA_KEY: &str = "pepper-and-carrot";

/// Pepper&Carrot's own language codes ("pseudo-ISO") with the ISO code the app
/// uses. Only languages listed here are offered; the site has about seventy.
const LANGUAGES: [(&str, &str); 10] = [
	("en", "en"),
	("ja", "ja"),
	("cn", "zh"),
	("fr", "fr"),
	("de", "de"),
	("es", "es"),
	("it", "it"),
	("pt", "pt"),
	("ru", "ru"),
	("kr", "ko"),
];

const DESCRIPTION: &str = "Pepper&Carrot is a free and open webcomic about Pepper, a young witch, and her cat Carrot. \
It is written and drawn by David Revoy.\n\n\
Licence: Creative Commons Attribution 4.0 (CC BY 4.0), https://creativecommons.org/licenses/by/4.0/\n\
Author and artist: David Revoy (https://www.davidrevoy.com). Translations are made by the Pepper&Carrot community; \
the translators of each language are listed on the chapter and on every episode's credits page.\n\
Read and support the comic at https://www.peppercarrot.com";

struct PepperCarrot;

fn code_for_iso(iso: &str) -> Option<&'static str> {
	LANGUAGES
		.iter()
		.find(|(_, mapped)| *mapped == iso)
		.map(|(code, _)| *code)
}

fn iso_for_code(code: &str) -> Option<&'static str> {
	LANGUAGES
		.iter()
		.find(|(known, _)| *known == code)
		.map(|(_, iso)| *iso)
}

/// `ep24_The-Unity-Tree` -> (24, "The Unity Tree").
fn parse_episode_name(name: &str) -> Option<(u32, String)> {
	let (prefix, title) = name.split_once('_')?;
	let number = prefix.strip_prefix("ep")?.parse::<u32>().ok()?;
	Some((number, title.replace('-', " ")))
}

fn cover_url(episode_name: &str, number: u32, code: &str, quality: &str) -> String {
	format!(
		"{SOURCES}/{episode_name}/{quality}/{code}_Pepper-and-Carrot_by-David-Revoy_E{number:02}.jpg"
	)
}

fn fetch_episodes() -> Result<Vec<Episode>> {
	Request::get(format!("{SOURCES}/episodes.json"))?
		.send()?
		.get_json::<Vec<Episode>>()
}

/// Translator names per language. Attribution is nice to have, so a failure
/// here must not hide the chapters.
fn fetch_languages() -> Option<Languages> {
	Request::get(format!("{SOURCES}/langs.json"))
		.ok()?
		.send()
		.ok()?
		.get_json::<Languages>()
		.ok()
}

fn series() -> Manga {
	Manga {
		key: MANGA_KEY.to_string(),
		title: "Pepper&Carrot".to_string(),
		cover: Some(cover_url("ep01_Potion-of-Flight", 1, "en", "low-res")),
		authors: Some(vec!["David Revoy".to_string()]),
		artists: Some(vec!["David Revoy".to_string()]),
		description: Some(DESCRIPTION.to_string()),
		url: Some(SITE.to_string()),
		tags: Some(vec![
			"Fantasy".to_string(),
			"Comedy".to_string(),
			"Witches".to_string(),
			"Open source".to_string(),
			"CC BY 4.0".to_string(),
		]),
		status: MangaStatus::Ongoing,
		content_rating: ContentRating::Safe,
		viewer: Viewer::LeftToRight,
		update_strategy: UpdateStrategy::Always,
		..Default::default()
	}
}

impl Source for PepperCarrot {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// The site is one series, so there is a single entry and no paging.
		let matches = match query.as_deref().map(str::trim) {
			None | Some("") => true,
			Some(text) => {
				let text = text.to_lowercase();
				["pepper", "carrot", "revoy", "witch", "komona"]
					.iter()
					.any(|word| word.contains(text.as_str()) || text.contains(word))
			}
		};
		let entries = if matches && page <= 1 {
			vec![series()]
		} else {
			Vec::new()
		};
		Ok(MangaPageResult {
			entries,
			has_next_page: false,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if needs_details {
			let mut details = series();
			details.chapters = manga.chapters.take();
			manga = details;
		}

		if needs_chapters {
			let episodes = fetch_episodes()?;
			let translators = fetch_languages();
			let quality = settings::quality();
			let mut chapters: Vec<Chapter> = Vec::new();

			// Newest episode first; within an episode, in the order languages
			// were picked in settings.
			for episode in episodes.iter().rev() {
				let Some((number, title)) = parse_episode_name(&episode.name) else {
					continue;
				};
				for iso in settings::selected_languages() {
					let Some(code) = code_for_iso(&iso) else {
						continue;
					};
					if !episode
						.translated_languages
						.iter()
						.any(|known| known == code)
					{
						continue;
					}
					let scanlators = translators
						.as_ref()
						.and_then(|languages| languages.get(code))
						.map(|entry| entry.translators.clone())
						.filter(|names| !names.is_empty());
					chapters.push(Chapter {
						key: format!("{}/{code}", episode.name),
						title: Some(title.clone()),
						chapter_number: Some(number as f32),
						scanlators,
						url: Some(format!("{SITE}/{code}/webcomic/{}.html", episode.name)),
						language: Some(iso.clone()),
						thumbnail: Some(cover_url(&episode.name, number, code, quality)),
						..Default::default()
					});
				}
			}
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let (episode_name, code) = chapter
			.key
			.split_once('/')
			.ok_or_else(|| aidoku::AidokuError::message("Unknown chapter."))?;
		if iso_for_code(code).is_none() {
			bail!("Unsupported language.");
		}
		let (number, _) = parse_episode_name(episode_name)
			.ok_or_else(|| aidoku::AidokuError::message("Unknown episode."))?;

		// Pages P00..=P<total_pages> exist for every listed episode; the last
		// one is the credits page.
		let total_pages = fetch_episodes()?
			.into_iter()
			.find(|episode| episode.name == episode_name)
			.map(|episode| episode.total_pages)
			.ok_or_else(|| aidoku::AidokuError::message("Episode not found."))?;

		let quality = settings::quality();
		Ok((0..=total_pages)
			.map(|index| Page {
				content: PageContent::url(format!(
					"{SOURCES}/{episode_name}/{quality}/{code}_Pepper-and-Carrot_by-David-Revoy_E{number:02}P{index:02}.jpg"
				)),
				..Default::default()
			})
			.collect())
	}
}

register_source!(PepperCarrot);
