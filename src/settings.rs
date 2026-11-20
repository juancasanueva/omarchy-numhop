//! Player settings, kept in `~/.config/numhop/settings.toml`.
//!
//! The file is tiny and flat (`language = "es"`), so it is read line by line rather than
//! pulling in a TOML parser. It is saved atomically: written beside the target, then
//! renamed over it, so a crash never leaves a half-written file.

use std::io;
use std::path::{Path, PathBuf};

use crate::i18n::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    pub lang: Lang,
}

/// `$XDG_CONFIG_HOME/numhop/settings.toml`, or `~/.config/numhop/settings.toml`.
pub fn settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("numhop").join("settings.toml"))
}

pub fn parse(text: &str) -> Option<Settings> {
    let lang = text.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if key.trim() != "language" {
            return None;
        }
        Lang::from_code(value.trim().trim_matches('"'))
    })?;
    Some(Settings { lang })
}

pub fn render(settings: &Settings) -> String {
    format!("# numhop settings\nlanguage = \"{}\"\n", settings.lang.code())
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
        save(&path, &Settings { lang: Lang::Es }).unwrap();
        assert_eq!(load(&path), Some(Settings { lang: Lang::Es }));
        assert!(!path.with_extension("toml.tmp").exists(), "temp file left behind");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_or_missing_language_is_ignored() {
        assert_eq!(parse("language = \"klingon\""), None);
        assert_eq!(parse("# empty"), None);
        assert_eq!(parse("language = \"es\""), Some(Settings { lang: Lang::Es }));
    }

    #[test]
    fn defaults_prefer_omarchy_then_locale() {
        assert_eq!(default_lang(Some("es\n"), Some("en_US.UTF-8")), Lang::Es);
        assert_eq!(default_lang(None, Some("es_AR.UTF-8")), Lang::Es);
        assert_eq!(default_lang(Some("fr"), Some("de_DE")), Lang::En);
        assert_eq!(default_lang(None, None), Lang::En);
    }
}
