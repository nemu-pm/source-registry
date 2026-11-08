#![no_std]

use aidoku::{
	AidokuError, Chapter, ContentRating, FilterValue, ImageRequestProvider, Manga, MangaPageResult,
	MangaStatus, Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
	alloc::{String, Vec, format, string::ToString, vec},
	imports::net::Request,
	prelude::*,
};

mod models;

use models::{Canvas, Crop, Manifest, Volume, Work};

const IIIF: &str = "https://dl.ndl.go.jp/api/iiif";
const CREDIT: &str = "国立国会図書館 National Diet Library, Japan";
const USER_AGENT: &str = "nemu-source-registry/1 (https://github.com/nemu-pm/source-registry)";
/// Widest image asked for; NDL's IIIF server tops out at 5000 px.
const PAGE_WIDTH: u32 = 1800;
const COVER_WIDTH: u32 = 600;
/// The value of a manifest's "Access Restrictions" for インターネット公開（保護期間満了）.
const PUBLIC_DOMAIN: &str = "PDM";

const WORKS: &str = include_str!("works.json");

struct NdlPublicDomain;

fn works() -> Result<Vec<Work>> {
	serde_json::from_str(WORKS).map_err(|_| AidokuError::message("Could not read the work list."))
}

fn cover_url(work: &Work) -> String {
	let cover = &work.cover;
	format!(
		"{IIIF}/{}/R{:07}/{}/{COVER_WIDTH},/0/default.jpg",
		cover.pid, cover.frame, cover.region
	)
}

fn manga_for(work: &Work) -> Manga {
	let mut tags = work.tags.clone();
	tags.push("Public domain".to_string());
	Manga {
		key: work.key.clone(),
		title: work.title.clone(),
		cover: Some(cover_url(work)),
		authors: Some(work.authors.clone()),
		artists: Some(work.authors.clone()),
		description: Some(format!(
			"{}\n\nLicence: Public domain. 公開範囲 インターネット公開（保護期間満了）, \
			reused under https://www.ndl.go.jp/en/use/reproduction/index.html\nPublished: {}\nSource: {CREDIT}",
			work.description, work.year
		)),
		url: work
			.volumes
			.first()
			.map(|volume| format!("https://dl.ndl.go.jp/pid/{}", volume.pid)),
		tags: Some(tags),
		status: MangaStatus::Completed,
		content_rating: ContentRating::Safe,
		viewer: Viewer::RightToLeft,
		update_strategy: UpdateStrategy::Never,
		..Default::default()
	}
}

fn chapter_for(volume: &Volume, number: usize) -> Chapter {
	Chapter {
		key: volume.pid.clone(),
		title: Some(volume.title.clone()),
		chapter_number: Some(number as f32),
		scanlators: Some(vec![CREDIT.to_string()]),
		url: Some(format!("https://dl.ndl.go.jp/pid/{}", volume.pid)),
		language: Some("ja".to_string()),
		..Default::default()
	}
}

fn fetch_manifest(pid: &str) -> Result<Manifest> {
	Request::get(format!("{IIIF}/{pid}/manifest.json"))?
		.header("User-Agent", USER_AGENT)
		.send()?
		.get_json::<Manifest>()
}

fn is_public_domain(manifest: &Manifest) -> bool {
	manifest
		.metadata
		.iter()
		.any(|row| row.label == "Access Restrictions" && row.value.as_str() == Some(PUBLIC_DOMAIN))
}

/// The IIIF image URL of one rectangle of a frame, in pixels.
fn region_url(service: &str, x: u32, y: u32, width: u32, height: u32) -> String {
	let size = width.min(PAGE_WIDTH);
	format!("{service}/{x},{y},{width},{height}/{size},/0/default.jpg")
}

/// A frame's image URLs in reading order.
fn frame_urls(canvas: &Canvas, crop: Option<Crop>, split: bool) -> Vec<String> {
	let Some(image) = canvas.images.first() else {
		return Vec::new();
	};
	let service = image.resource.service.id.as_str();
	let (frame_w, frame_h) = (canvas.width as f32, canvas.height as f32);
	let crop = crop.unwrap_or(Crop {
		left: 0.0,
		top: 0.0,
		right: 1.0,
		bottom: 1.0,
	});
	let x = (crop.left * frame_w) as u32;
	let y = (crop.top * frame_h) as u32;
	let width = ((crop.right - crop.left) * frame_w) as u32;
	let height = ((crop.bottom - crop.top) * frame_h) as u32;
	if split {
		let half = width / 2;
		vec![
			region_url(service, x + half, y, width - half, height),
			region_url(service, x, y, half, height),
		]
	} else {
		vec![region_url(service, x, y, width, height)]
	}
}

impl Source for NdlPublicDomain {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// A short fixed list, so no paging and no request to NDL's search API.
		if page > 1 {
			return Ok(MangaPageResult {
				entries: Vec::new(),
				has_next_page: false,
			});
		}
		let needle = query
			.as_deref()
			.map(str::trim)
			.filter(|text| !text.is_empty())
			.map(str::to_lowercase);
		let entries = works()?
			.iter()
			.filter(|work| match &needle {
				None => true,
				Some(text) => {
					work.title.to_lowercase().contains(text.as_str())
						|| work
							.authors
							.iter()
							.any(|name| name.to_lowercase().contains(text.as_str()))
						|| work.description.to_lowercase().contains(text.as_str())
				}
			})
			.map(manga_for)
			.collect();
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
		let works = works()?;
		let work = works
			.iter()
			.find(|work| work.key == manga.key)
			.ok_or_else(|| AidokuError::message("Unknown work."))?;

		if needs_details {
			let mut details = manga_for(work);
			details.chapters = manga.chapters.take();
			manga = details;
		}

		if needs_chapters {
			// Last volume first, like the other sources in this directory.
			manga.chapters = Some(
				work.volumes
					.iter()
					.enumerate()
					.rev()
					.map(|(index, volume)| chapter_for(volume, index + 1))
					.collect(),
			);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let works = works()?;
		let work = works
			.iter()
			.find(|work| {
				work.key == manga.key && work.volumes.iter().any(|volume| volume.pid == chapter.key)
			})
			.ok_or_else(|| AidokuError::message("Unknown volume."))?;

		let manifest = fetch_manifest(&chapter.key)?;
		// NDL marks its status per item and can change it, so ask each time
		// rather than trust the list this source shipped with.
		if !is_public_domain(&manifest) {
			bail!("This volume is no longer marked public domain by NDL.");
		}
		let canvases = manifest
			.sequences
			.first()
			.map(|sequence| sequence.canvases.as_slice())
			.unwrap_or_default();
		let end = canvases.len().saturating_sub(work.skip_end);
		let start = work.skip_start.min(end);

		Ok(canvases[start..end]
			.iter()
			.flat_map(|canvas| frame_urls(canvas, work.crop, work.split))
			.map(|url| Page {
				content: PageContent::url(url),
				..Default::default()
			})
			.collect())
	}
}

impl ImageRequestProvider for NdlPublicDomain {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(Request::get(url)?.header("User-Agent", USER_AGENT))
	}
}

register_source!(NdlPublicDomain, ImageRequestProvider);
