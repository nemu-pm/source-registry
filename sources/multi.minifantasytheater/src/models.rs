use aidoku::alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::Deserialize;

/// Credits keyed by role (`art`, `scenario`, `translation`, `proofreading`, ...).
pub type Credits = BTreeMap<String, Vec<String>>;

/// `0_sources/miniFantasyTheater/.<NNN>.json` (art and scenario) and
/// `lang/<code>/<NNN>_info.json` (translation and proofreading); only the field
/// we use.
#[derive(Deserialize)]
pub struct CreditsFile {
	#[serde(default)]
	pub credits: Credits,
}
