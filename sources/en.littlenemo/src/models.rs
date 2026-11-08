use aidoku::alloc::{string::String, vec::Vec};
use serde::Deserialize;

/// Response of the Commons `action=query&prop=imageinfo` API (`formatversion=2`).
#[derive(Deserialize)]
pub struct ApiResponse {
	pub query: Option<Query>,
}

#[derive(Deserialize)]
pub struct Query {
	#[serde(default)]
	pub pages: Vec<ApiPage>,
}

#[derive(Deserialize)]
pub struct ApiPage {
	pub title: String,
	#[serde(default)]
	pub imageinfo: Vec<ImageInfo>,
}

#[derive(Deserialize)]
pub struct ImageInfo {
	/// The scaled copy, or the original if it is narrower than the requested width.
	pub thumburl: Option<String>,
	pub extmetadata: Option<ExtMetadata>,
}

#[derive(Deserialize)]
pub struct ExtMetadata {
	#[serde(rename = "License")]
	pub license: Option<MetadataValue>,
}

#[derive(Deserialize)]
pub struct MetadataValue {
	pub value: String,
}
