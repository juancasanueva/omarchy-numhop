//! Player settings, kept in `~/.config/numhop/settings.toml`, and the players' records,
//! kept in `~/.local/share/numhop/records.toml`.
//!
//! The file is tiny and flat (`language = "es"`, `player1 = "Lucía"`, `sound = "off"`), so it is read
//! line by line rather than pulling in a TOML parser. Names are cleaned on the way in and
//! out, so they never hold a quote or a backslash. It is saved atomically: written beside the target, then
//! renamed over it, so a crash never leaves a half-written file.

use std::io;
use std::path::{Path, PathBuf};

use crate::domain::records::Records;
use crate::domain::rules::clean_name;
use crate::i18n::Lang;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    pub lang: Lang,
    /// The names the players typed; empty means "Player 1" or "Player 2".
    pub names: [String; 2],
    /// Sound effects are off. A file without a `sound` line means sound is on.
    pub muted: bool,
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
    let muted = value(text, "sound") == Some("off");
    Some(Settings { lang, names: [name("player1"), name("player2")], muted })
}

pub fn render(settings: &Settings) -> String {
    let [one, two] = &settings.names;
    format!(
        "# numhop settings\nlanguage = \"{}\"\nplayer1 = \"{}\"\nplayer2 = \"{}\"\nsound = \"{}\"\n",
        settings.lang.code(),
        clean_name(one),
        clean_name(two),
        if settings.muted { "off" } else { "on" }
    )
}

pub fn load(path: &Path) -> Option<Settings> {
    parse(&std::fs::read_to_string(path).ok()?)
}

pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    write_atomically(path, &render(settings))
}

/// Writes beside the target, then renames over it, so a crash never leaves half a file.
/// The temporary name carries the process id, so two running copies of the game never
/// write into the same temporary file.
fn write_atomically(path: &Path, text: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!("toml.{}.tmp", std::process::id()));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

/// `$XDG_DATA_HOME/numhop/records.toml`, or `~/.local/share/numhop/records.toml`.
pub fn records_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("numhop").join("records.toml"))
}

/// The saved records. A missing file means none yet. Bytes that are not valid text
/// are replaced, so one damaged line loses only that line. A file that exists but
/// cannot be read is an error, so the caller can avoid saving over it.
pub fn load_records(path: &Path) -> io::Result<Records> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Records::parse(&String::from_utf8_lossy(&bytes))),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Records::default()),
        Err(err) => Err(err),
    }
}

pub fn save_records(path: &Path, records: &Records) -> io::Result<()> {
    write_atomically(path, &records.render())
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
        muted: false,
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
        let settings = Settings { lang: Lang::Es, names: ["Lucía".into(), "Íñigo".into()], muted: true };
        save(&path, &settings).unwrap();
        assert_eq!(load(&path), Some(settings));
        assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1, "temp file left behind");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn records_round_trip_through_a_file_and_start_empty() {
        let dir = temp_dir("records");
        let path = dir.join("numhop/records.toml");
        assert_eq!(load_records(&path).unwrap(), Records::default(), "no file yet means no records");
        let mut records = Records::default();
        records.merge("lucía", 1200, 2);
        save_records(&path, &records).unwrap();
        assert_eq!(load_records(&path).unwrap(), records);
        assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1, "temp file left behind");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_bad_byte_in_the_records_loses_only_its_line() {
        let dir = temp_dir("records-bytes");
        let path = dir.join("records.toml");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, b"[\"ana\"]\nbest_score = 70\n\xff\xfe junk\nworlds_cleared = 2\n").unwrap();
        let records = load_records(&path).unwrap();
        assert_eq!(records.get("ana"), crate::domain::records::Record { best_score: 70, worlds_cleared: 2 });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn records_that_cannot_be_read_are_an_error_not_empty() {
        // A folder where the file should be cannot be read as one.
        let dir = temp_dir("records-unreadable");
        std::fs::create_dir_all(dir.join("records.toml")).unwrap();
        assert!(load_records(&dir.join("records.toml")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_or_missing_language_is_ignored() {
        assert_eq!(parse("language = \"klingon\""), None);
        assert_eq!(parse("# empty"), None);
        assert_eq!(parse("language = \"es\""), Some(Settings { lang: Lang::Es, names: Default::default(), muted: false }));
    }

    #[test]
    fn names_are_cleaned_when_read_and_written() {
        let settings = parse("language = \"en\"\nplayer1 = \"  Ana   María \"\nplayer2 = \"Bob\\\\\"").unwrap();
        assert_eq!(settings.names, ["Ana María".to_string(), "Bob".to_string()]);
        let messy = Settings { lang: Lang::En, names: ["Zoe\"".into(), String::new()], muted: false };
        assert_eq!(render(&messy), "# numhop settings\nlanguage = \"en\"\nplayer1 = \"Zoe\"\nplayer2 = \"\"\nsound = \"on\"\n");
    }

    #[test]
    fn sound_is_on_unless_the_file_says_off() {
        assert!(!parse("language = \"en\"").unwrap().muted, "older files have no sound line");
        assert!(!parse("language = \"en\"\nsound = \"on\"").unwrap().muted);
        assert!(!parse("language = \"en\"\nsound = \"loud\"").unwrap().muted);
        assert!(parse("language = \"en\"\nsound = \"off\"").unwrap().muted);
    }

    #[test]
    fn defaults_prefer_omarchy_then_locale() {
        assert_eq!(default_lang(Some("es\n"), Some("en_US.UTF-8")), Lang::Es);
        assert_eq!(default_lang(None, Some("es_AR.UTF-8")), Lang::Es);
        assert_eq!(default_lang(Some("fr"), Some("de_DE")), Lang::En);
        assert_eq!(default_lang(None, None), Lang::En);
    }
}
