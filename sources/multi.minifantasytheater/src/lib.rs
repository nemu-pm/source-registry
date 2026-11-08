#![no_std]

use aidoku::{
	AidokuError, Chapter, ContentRating, FilterValue, Manga, MangaPageResult, MangaStatus, Page,
	PageContent, Result, Source, UpdateStrategy, Viewer,
	alloc::{String, Vec, format, string::ToString, vec},
	imports::{net::Request, std::parse_date},
	prelude::*,
};

mod models;
mod settings;

use models::{Credits, CreditsFile};

const SITE: &str = "https://www.peppercarrot.com";
const STRIPS: &str = "https://www.peppercarrot.com/0_sources/miniFantasyTheater";
const MANGA_KEY: &str = "mini-fantasy-theater";
const LICENCE_NOTICE: &str = "CC BY-SA 4.0, AI derivation prohibited";

/// The site's language codes with the ISO code the app uses: every language the
/// site has rendered pages for.
const LANGUAGES: [(&str, &str); 9] = [
	("en", "en"),
	("ja", "ja"),
	("fr", "fr"),
	("es", "es"),
	("de", "de"),
	("ca", "ca"),
	("nn", "nn"),
	("sl", "sl"),
	("tp", "tok"),
];

/// Credit roles in the order the site lists them.
const ROLES: [(&str, &str); 5] = [
	("art", "Art"),
	("scenario", "Scenario"),
	("translation", "Translation"),
	("proofreading", "Proofreading"),
	("contribution", "Contribution"),
];

const DESCRIPTION: &str = "Mini Fantasy Theater is a weekly gag strip of four-panel episodes: fantasy tropes, \
geek jokes and an Avian Intelligence parrot. It is drawn by David Revoy, the author of Pepper&Carrot.\n\n\
Licence: Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0), https://creativecommons.org/licenses/by-sa/4.0/\n\
AI derivation prohibited: https://www.davidrevoy.com/article1178/license-update-ai-derivation-prohibited-on-all-my-art-lore-stories-and-comics\n\
Author and artist: David Revoy (https://www.davidrevoy.com). Translations are made by the community; \
each episode's contributors are listed on its last page.\n\
Read and support the comic at https://www.peppercarrot.com/en/webcomics/miniFantasyTheater.html";

struct MiniFantasyTheater;

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

fn episode_url(code: &str, episode: &str) -> String {
	format!("{SITE}/{code}/miniFantasyTheater/{episode}.html")
}

/// `.../ja/miniFantasyTheater/035.html` -> "035".
fn episode_from_href(href: &str) -> Option<&str> {
	let episode = href.rsplit('/').next()?.strip_suffix(".html")?;
	(episode.len() == 3 && episode.bytes().all(|byte| byte.is_ascii_digit())).then_some(episode)
}

/// "35. あなたのすべての言葉を送信します" -> the title after the number.
fn title_from_caption(caption: &str) -> Option<String> {
	let (_, title) = caption.split_once(". ")?;
	Some(title.trim().to_string())
}

/// The ISO date in a localised "Published on 2026-11-04." caption.
fn date_from_caption(caption: &str) -> Option<i64> {
	let bytes = caption.as_bytes();
	(0..bytes.len().checked_sub(9)?).find_map(|start| {
		let window = bytes.get(start..start + 10)?;
		let is_date = window.iter().enumerate().all(|(index, byte)| match index {
			4 | 7 => *byte == b'-',
			_ => byte.is_ascii_digit(),
		});
		if !is_date {
			return None;
		}
		parse_date(caption.get(start..start + 10)?, "yyyy-MM-dd")
	})
}

/// The episodes published in one language. The site's series page is the only
/// official list; it marks each episode `translated` when that language has
/// rendered pages, and otherwise shows it in English, which we leave out.
fn fetch_chapters(code: &str, iso: &str) -> Result<Vec<(u32, Chapter)>> {
	let page = Request::get(format!("{SITE}/{code}/webcomics/miniFantasyTheater.html"))?.html()?;
	let figures = page
		.select("figure.thumbnail.translated")
		.ok_or_else(|| AidokuError::message("Could not read the episode list."))?;

	let mut chapters = Vec::new();
	for figure in figures {
		let Some(link) = figure.select_first("figcaption a") else {
			continue;
		};
		let Some(href) = link.attr("href") else {
			continue;
		};
		let Some(episode) = episode_from_href(&href) else {
			continue;
		};
		let Ok(number) = episode.parse::<u32>() else {
			continue;
		};
		let date_uploaded = figure
			.select_first("figcaption span")
			.and_then(|span| span.text())
			.and_then(|caption| date_from_caption(&caption));
		chapters.push((
			number,
			Chapter {
				key: format!("{episode}/{code}"),
				title: link.text().and_then(|text| title_from_caption(&text)),
				chapter_number: Some(number as f32),
				date_uploaded,
				url: Some(episode_url(code, episode)),
				language: Some(iso.to_string()),
				thumbnail: figure
					.select_first("img")
					.and_then(|image| image.attr("src")),
				..Default::default()
			},
		));
	}
	Ok(chapters)
}

/// Credits are nice to have on top of the notice, so a failure here must not
/// hide the strip.
fn fetch_credits(url: String) -> Credits {
	Request::get(url)
		.ok()
		.and_then(|request| request.send().ok())
		.and_then(|mut response| response.get_json::<CreditsFile>().ok())
		.map(|file| file.credits)
		.unwrap_or_default()
}

/// "David Revoy <https://framagit.org/Deevad>" -> "David Revoy".
fn display_name(credit: &str) -> &str {
	credit
		.split_once(" <")
		.map_or(credit, |(name, _)| name)
		.trim()
}

/// The chapter's last page: the attribution CC BY-SA asks for, translators
/// included. The strip itself only names the artist.
fn credits_page(chapter: &Chapter, episode: &str, code: &str) -> String {
	let mut credits = fetch_credits(format!("{STRIPS}/.{episode}.json"));
	credits.extend(fetch_credits(format!(
		"{STRIPS}/lang/{code}/{episode}_info.json"
	)));

	let mut lines = vec![format!(
		"**{}**",
		chapter.title.as_deref().unwrap_or("Mini Fantasy Theater")
	)];
	for (key, label) in ROLES {
		let names: Vec<&str> = credits
			.get(key)
			.into_iter()
			.flatten()
			.map(|credit| display_name(credit))
			.filter(|name| !name.is_empty() && *name != "original version")
			.collect();
		if !names.is_empty() {
			lines.push(format!("{label}: {}", names.join(", ")));
		}
	}
	if !credits.contains_key("art") {
		lines.push("Art: David Revoy".to_string());
	}
	lines.push(format!("Licence: {LICENCE_NOTICE}"));
	lines.push(episode_url(code, episode));
	lines.join("\n\n")
}

fn series() -> Manga {
	Manga {
		key: MANGA_KEY.to_string(),
		title: "Mini Fantasy Theater".to_string(),
		cover: Some(format!(
			"{STRIPS}/low-res/thumbnails/028-thumb_miniFantasyTheater.jpg"
		)),
		authors: Some(vec!["David Revoy".to_string()]),
		artists: Some(vec!["David Revoy".to_string()]),
		description: Some(DESCRIPTION.to_string()),
		url: Some(format!("{SITE}/en/webcomics/miniFantasyTheater.html")),
		tags: Some(vec![
			"Fantasy".to_string(),
			"Comedy".to_string(),
			"Full color".to_string(),
			"CC BY-SA 4.0".to_string(),
		]),
		status: MangaStatus::Ongoing,
		content_rating: ContentRating::Safe,
		viewer: Viewer::LeftToRight,
		update_strategy: UpdateStrategy::Always,
		..Default::default()
	}
}

impl Source for MiniFantasyTheater {
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
				["mini", "fantasy", "theater", "theatre", "revoy", "parrot"]
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
			// Newest episode first; within an episode, in the order languages
			// were picked in settings. The sort is stable and languages are
			// appended in that order.
			let mut chapters: Vec<(u32, Chapter)> = Vec::new();
			for iso in settings::selected_languages() {
				let Some(code) = code_for_iso(&iso) else {
					continue;
				};
				chapters.extend(fetch_chapters(code, &iso)?);
			}
			chapters.sort_by(|(left, _), (right, _)| right.cmp(left));
			manga.chapters = Some(chapters.into_iter().map(|(_, chapter)| chapter).collect());
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let (episode, code) = chapter
			.key
			.split_once('/')
			.ok_or_else(|| AidokuError::message("Unknown chapter."))?;
		if iso_for_code(code).is_none() {
			bail!("Unsupported language.");
		}

		let quality = settings::quality();
		Ok(vec![
			Page {
				content: PageContent::url(format!(
					"{STRIPS}/{quality}/{code}_{episode}_miniFantasyTheater.jpg"
				)),
				..Default::default()
			},
			Page {
				content: PageContent::text(credits_page(&chapter, episode, code)),
				..Default::default()
			},
		])
	}
}

register_source!(MiniFantasyTheater);
