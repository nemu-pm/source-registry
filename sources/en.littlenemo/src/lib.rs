#![no_std]

use aidoku::{
	Chapter, ContentRating, FilterValue, ImageRequestProvider, Manga, MangaPageResult, MangaStatus,
	Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
	alloc::{String, Vec, format, string::ToString, vec},
	helpers::uri::encode_uri_component,
	imports::net::Request,
	prelude::*,
};

mod models;

use models::ApiResponse;

const API: &str = "https://commons.wikimedia.org/w/api.php";
const CATEGORY: &str = "https://commons.wikimedia.org/wiki/Category:Little_Nemo_in_Slumberland";
const MANGA_KEY: &str = "little-nemo-in-slumberland";
/// Wikimedia asks clients to identify themselves and to give a way to reach them.
/// https://foundation.wikimedia.org/wiki/Policy:Wikimedia_Foundation_User-Agent_Policy
const USER_AGENT: &str = "nemu-source-registry/1 (https://github.com/nemu-pm/source-registry)";
/// Wikimedia only serves standard thumbnail widths; narrower originals come back as they are.
const PAGE_WIDTH: u32 = 1920;
const COVER_WIDTH: u32 = 960;
const COVER_FILE: &str = "Little Nemo 1908-07-26.jpg";
/// The API takes at most this many titles per request.
const BATCH: usize = 50;

/// `date<TAB>file name` per line, oldest first. Generated and checked by
/// `scripts/check-commons-licences.py`, which is also where a file is added or removed.
const STRIPS: &str = include_str!("strips.tsv");

const DESCRIPTION: &str = "Little Nemo in Slumberland is Winsor McCay's Sunday newspaper comic, \
first published in the New York Herald and the New York American between 1905 and 1911. Every \
night Nemo sets out for Slumberland, and every strip ends with him waking up.\n\n\
Licence: Public domain. McCay died in 1934 and these strips were published before 1931. Each \
file's licence is on its Wikimedia Commons page, and files that stop being marked public domain \
are not shown: https://commons.wikimedia.org/wiki/Category:Little_Nemo_in_Slumberland\n\
Scans: Wikimedia Commons contributors.\n\n\
Content note: these are strips from 1905-1911 and some contain racist caricatures of the period, \
including the character Imp.";

struct LittleNemo;

fn strips() -> impl Iterator<Item = (&'static str, &'static str)> {
	STRIPS.lines().filter_map(|line| line.split_once('\t'))
}

/// A year's chapter, from `1905` to `1911`.
fn years() -> Vec<&'static str> {
	let mut years: Vec<&str> = strips().filter_map(|(date, _)| date.get(..4)).collect();
	years.dedup();
	years
}

fn cover_url() -> String {
	format!(
		"https://commons.wikimedia.org/wiki/Special:FilePath/{}?width={COVER_WIDTH}",
		encode_uri_component(COVER_FILE)
	)
}

fn series() -> Manga {
	Manga {
		key: MANGA_KEY.to_string(),
		title: "Little Nemo in Slumberland".to_string(),
		cover: Some(cover_url()),
		authors: Some(vec!["Winsor McCay".to_string()]),
		artists: Some(vec!["Winsor McCay".to_string()]),
		description: Some(DESCRIPTION.to_string()),
		url: Some(CATEGORY.to_string()),
		tags: Some(vec![
			"Fantasy".to_string(),
			"Newspaper strip".to_string(),
			"Dreams".to_string(),
			"Public domain".to_string(),
		]),
		status: MangaStatus::Completed,
		content_rating: ContentRating::Safe,
		viewer: Viewer::LeftToRight,
		update_strategy: UpdateStrategy::Never,
		..Default::default()
	}
}

fn with_user_agent(request: Request) -> Request {
	request
		.header("User-Agent", USER_AGENT)
		.header("Api-User-Agent", USER_AGENT)
}

/// The scaled image for each of up to `BATCH` files, keeping only files Commons
/// still marks as public domain.
fn fetch_batch(files: &[&str]) -> Result<Vec<(String, String)>> {
	let titles = files
		.iter()
		.map(|file| format!("File:{file}"))
		.collect::<Vec<_>>()
		.join("|");
	let url = format!(
		"{API}?action=query&format=json&formatversion=2&origin=*&prop=imageinfo&iiprop=url%7Cextmetadata\
		&iiextmetadatafilter=License&iiurlwidth={PAGE_WIDTH}&titles={}",
		encode_uri_component(titles)
	);
	let response = with_user_agent(Request::get(url)?)
		.send()?
		.get_json::<ApiResponse>()?;
	Ok(response
		.query
		.map(|query| query.pages)
		.unwrap_or_default()
		.into_iter()
		.filter_map(|page| {
			let info = page.imageinfo.into_iter().next()?;
			let license = info.extmetadata?.license?.value;
			if license != "pd" {
				return None;
			}
			Some((page.title, info.thumburl?))
		})
		.collect())
}

impl Source for LittleNemo {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// One series, so a single entry and no paging.
		let matches = match query.as_deref().map(str::trim) {
			None | Some("") => true,
			Some(text) => {
				let text = text.to_lowercase();
				["little nemo", "nemo", "slumberland", "winsor", "mccay"]
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
			// Newest year first, like the other sources in this directory.
			manga.chapters = Some(
				years()
					.into_iter()
					.enumerate()
					.rev()
					.map(|(index, year)| Chapter {
						key: year.to_string(),
						title: Some(year.to_string()),
						chapter_number: Some((index + 1) as f32),
						scanlators: Some(vec!["Wikimedia Commons".to_string()]),
						url: Some(format!("{CATEGORY},_{year}")),
						language: Some("en".to_string()),
						..Default::default()
					})
					.collect(),
			);
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let files: Vec<&str> = strips()
			.filter(|(date, _)| date.starts_with(chapter.key.as_str()))
			.map(|(_, file)| file)
			.collect();
		if files.is_empty() {
			bail!("Unknown chapter.");
		}

		let mut pages = Vec::with_capacity(files.len());
		for batch in files.chunks(BATCH) {
			let images = fetch_batch(batch)?;
			// The API returns pages in its own order; the strips read by date.
			for file in batch {
				let title = format!("File:{file}");
				if let Some((_, url)) = images.iter().find(|(name, _)| *name == title) {
					pages.push(Page {
						content: PageContent::url(url.as_str()),
						..Default::default()
					});
				}
			}
		}
		Ok(pages)
	}
}

impl ImageRequestProvider for LittleNemo {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(with_user_agent(Request::get(url)?))
	}
}

register_source!(LittleNemo, ImageRequestProvider);
