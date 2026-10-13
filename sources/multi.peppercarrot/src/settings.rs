use aidoku::{
	alloc::{String, Vec, vec},
	imports::defaults::defaults_get,
};

const LANGUAGES_KEY: &str = "languages";
const QUALITY_KEY: &str = "quality";

/// Languages the user picked in settings, as ISO codes. English when unset.
pub fn selected_languages() -> Vec<String> {
	match defaults_get::<Vec<String>>(LANGUAGES_KEY) {
		Some(languages) if !languages.is_empty() => languages,
		_ => vec![String::from("en")],
	}
}

/// `low-res` (default) or `hi-res`: the directory pages are served from.
pub fn quality() -> &'static str {
	match defaults_get::<String>(QUALITY_KEY).as_deref() {
		Some("hi-res") => "hi-res",
		_ => "low-res",
	}
}
