use aidoku::alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::Deserialize;

/// One entry of `0_sources/episodes.json`.
#[derive(Deserialize)]
pub struct Episode {
	/// Directory name, e.g. `ep24_The-Unity-Tree`.
	pub name: String,
	/// Pages excluding the credits page; pages `P00..=P<total_pages>` exist.
	pub total_pages: u32,
	/// Pepper&Carrot language codes the episode is published in.
	pub translated_languages: Vec<String>,
}

/// One entry of `0_sources/langs.json` (only the field we use).
#[derive(Deserialize)]
pub struct LanguageEntry {
	#[serde(default)]
	pub translators: Vec<String>,
}

pub type Languages = BTreeMap<String, LanguageEntry>;
