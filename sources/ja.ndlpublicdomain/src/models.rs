use aidoku::alloc::{string::String, vec::Vec};
use serde::Deserialize;

/// One curated work of `works.json`.
#[derive(Deserialize)]
pub struct Work {
	pub key: String,
	pub title: String,
	pub authors: Vec<String>,
	pub year: String,
	pub description: String,
	pub tags: Vec<String>,
	pub cover: Cover,
	/// The part of each frame to keep, leaving out the ruler and colour chart
	/// NDL photographs beside a book. The whole frame when absent.
	pub crop: Option<Crop>,
	/// Serve each frame as two pages, right half first. For books whose every
	/// page stands alone, so they read at full size on a phone held upright.
	#[serde(default)]
	pub split: bool,
	/// Frames dropped from each end of a volume: the covers, shot with the chart.
	#[serde(default)]
	pub skip_start: usize,
	#[serde(default)]
	pub skip_end: usize,
	pub volumes: Vec<Volume>,
}

#[derive(Deserialize)]
pub struct Cover {
	pub pid: String,
	/// 1-based frame number, as in the viewer on dl.ndl.go.jp.
	pub frame: u32,
	/// IIIF region, `x,y,w,h` in pixels of the full frame.
	pub region: String,
}

/// Edges of the kept area as fractions of the frame's width and height.
#[derive(Deserialize, Clone, Copy)]
pub struct Crop {
	pub left: f32,
	pub top: f32,
	pub right: f32,
	pub bottom: f32,
}

#[derive(Deserialize)]
pub struct Volume {
	pub pid: String,
	pub title: String,
}

/// IIIF Presentation 2.1 manifest (only the fields we use).
#[derive(Deserialize)]
pub struct Manifest {
	#[serde(default)]
	pub metadata: Vec<Metadata>,
	#[serde(default)]
	pub sequences: Vec<Sequence>,
}

#[derive(Deserialize)]
pub struct Metadata {
	pub label: String,
	pub value: serde_json::Value,
}

#[derive(Deserialize)]
pub struct Sequence {
	pub canvases: Vec<Canvas>,
}

#[derive(Deserialize)]
pub struct Canvas {
	pub width: u32,
	pub height: u32,
	pub images: Vec<Annotation>,
}

#[derive(Deserialize)]
pub struct Annotation {
	pub resource: Resource,
}

#[derive(Deserialize)]
pub struct Resource {
	pub service: Service,
}

#[derive(Deserialize)]
pub struct Service {
	#[serde(rename = "@id")]
	pub id: String,
}
