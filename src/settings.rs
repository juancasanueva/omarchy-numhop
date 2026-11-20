//! Player settings, kept in `~/.config/numhop/settings.toml`.
//!
//! The file is tiny and flat (`language = "es"`, `player1 = "Lucía"`), so it is read
//! line by line rather than pulling in a TOML parser. Names are cleaned on the way in and
//! out, so they never hold a quote or a backslash. It is saved atomically: written beside the target, then
//! renamed over it, so a crash never leaves a half-written file.

use std::io;
use std::path::{Path, PathBuf};

use crate::domain::rules::clean_name;
use crate::i18n::Lang;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    pub lang: Lang,
    /// The names the players typed; empty means "Player 1" or "Player 2".
    pub names: [String; 2],
}

/// `$XDG_CONFIG_HOME/numhop/settings.toml`, or `~/.config/numhop/settings.toml`.
pub fn settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("numhop").join("settings.toml"))
}

/// The value of `key = "value"` lines, without the quotes.
fn value<'a>(text: &'a str, wanted: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == wanted).then(|| value.trim().trim_matches('"'))
    })
}

pub fn parse(text: &str) -> Option<Settings> {
    let lang = value(text, "language").and_then(Lang::from_code)?;
    let name = |key| value(text, key).map(clean_name).unwrap_or_default();
    Some(Settings { lang, names: [name("player1"), name("player2")] })
}

pub fn render(settings: &Settings) -> String {
    let [one, two] = &settings.names;
    format!(
        "# numhop settings\nlanguage = \"{}\"\nplayer1 = \"{}\"\nplayer2 = \"{}\"\n",
        settings.lang.code(),
        clean_name(one),
        clean_name(two)
    )
}

pub fn load(path: &Path) -> Option<Settings> {
    parse(&std::fs::read_to_string(path).ok()?)
}

pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, render(settings))?;
    std::fs::rename(&tmp, path)
}

/// The language to start with when nothing is saved: Omarchy's chosen language, then
/// the system locale, then English.
pub fn default_lang(omarchy_language: Option<&str>, locale: Option<&str>) -> Lang {
    omarchy_language
        .and_then(Lang::from_code)
        .or_else(|| locale.and_then(Lang::from_code))
        .unwrap_or_default()
}

/// Saved settings, or first-run defaults read from the system.
pub fn load_or_default() -> Settings {
    if let Some(saved) = settings_path().and_then(|p| load(&p)) {
        return saved;
    }
    let omarchy = std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".config/omarchy/language"))
        .and_then(|p| std::fs::read_to_string(p).ok());
    let locale = std::env::var("LC_ALL")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("LANG").ok());
    Settings {
        lang: default_lang(omarchy.as_deref(), locale.as_deref()),
        names: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("numhop-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn round_trips_through_a_file() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("numhop/settings.toml");
        let settings = Settings { lang: Lang::Es, names: ["Lucía".into(), "Íñigo".into()] };
        save(&path, &settings).unwrap();
        assert_eq!(load(&path), Some(settings));
        assert!(!path.with_extension("toml.tmp").exists(), "temp file left behind");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_or_missing_language_is_ignored() {
        assert_eq!(parse("language = \"klingon\""), None);
        assert_eq!(parse("# empty"), None);
        assert_eq!(parse("language = \"es\""), Some(Settings { lang: Lang::Es, names: Default::default() }));
    }

    #[test]
    fn names_are_cleaned_when_read_and_written() {
        let settings = parse("language = \"en\"\nplayer1 = \"  Ana   María \"\nplayer2 = \"Bob\\\\\"").unwrap();
        assert_eq!(settings.names, ["Ana María".to_string(), "Bob".to_string()]);
        let messy = Settings { lang: Lang::En, names: ["Zoe\"".into(), String::new()] };
        assert_eq!(render(&messy), "# numhop settings\nlanguage = \"en\"\nplayer1 = \"Zoe\"\nplayer2 = \"\"\n");
    }

    #[test]
    fn defaults_prefer_omarchy_then_locale() {
        assert_eq!(default_lang(Some("es\n"), Some("en_US.UTF-8")), Lang::Es);
        assert_eq!(default_lang(None, Some("es_AR.UTF-8")), Lang::Es);
        assert_eq!(default_lang(Some("fr"), Some("de_DE")), Lang::En);
        assert_eq!(default_lang(None, None), Lang::En);
    }
}
