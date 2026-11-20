//! Follows the Omarchy theme accent live.
//!
//! A theme switch replaces files and symlinks rather than editing them, so instead of an
//! inotify watch that would need re-arming, the file is polled by modification time.

use std::path::PathBuf;
use std::time::SystemTime;

use bevy::prelude::*;

pub const FALLBACK_ACCENT: &str = "#FFD60A";

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Accent(pub Color);

/// Marks UI text drawn in the accent color.
#[derive(Component)]
pub struct AccentText;

/// Marks UI borders drawn in the accent color.
#[derive(Component)]
pub struct AccentBorder;

/// Marks UI backgrounds drawn in the accent color.
#[derive(Component)]
pub struct AccentBackground;

/// Hyprland's window rounding, so panels match the windows around the game.
/// Omarchy's default is 0, which means sharp corners.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Rounding(pub f32);

fn hypr_rounding() -> f32 {
    let output = std::process::Command::new("hyprctl")
        .args(["getoption", "decoration:rounding", "-j"])
        .output();
    let Ok(output) = output else { return 0.0 };
    let text = String::from_utf8_lossy(&output.stdout);
    // The JSON is tiny and flat: {"option": "...", "int": 8, ...}.
    text.split("\"int\":")
        .nth(1)
        .and_then(|rest| rest.split([',', '}']).next())
        .and_then(|n| n.trim().parse::<f32>().ok())
        .unwrap_or(0.0)
        .clamp(0.0, 24.0)
}

#[derive(Resource)]
struct ThemeWatch {
    path: Option<PathBuf>,
    modified: Option<SystemTime>,
    timer: Timer,
}

pub fn plugin(app: &mut App) {
    let path = theme_colors_path();
    let accent = path.as_ref().and_then(|p| read_accent(p)).unwrap_or_else(fallback);
    app.insert_resource(Accent(accent))
        .insert_resource(Rounding(hypr_rounding()))
        .insert_resource(ThemeWatch {
            modified: path.as_ref().and_then(|p| modified(p)),
            path,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        })
        .add_systems(Update, (poll_theme, apply_accent).chain());
}

fn fallback() -> Color {
    parse_hex(FALLBACK_ACCENT).unwrap()
}

fn theme_colors_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".local/state/omarchy/current/theme/colors.toml"))
}

fn modified(path: &PathBuf) -> Option<SystemTime> {
    // `metadata` follows symlinks, so a re-pointed theme link changes the time we see.
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

fn read_accent(path: &PathBuf) -> Option<Color> {
    let text = std::fs::read_to_string(path).ok()?;
    parse_accent(&text)
}

/// Finds `accent = "#rrggbb"` in an Omarchy `colors.toml`.
pub fn parse_accent(toml: &str) -> Option<Color> {
    toml.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        if key.trim() != "accent" {
            return None;
        }
        parse_hex(value.trim().trim_matches('"').trim_matches('\''))
    })
}

pub fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(hex, 16).ok()?;
    Some(Color::srgb_u8((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

fn poll_theme(time: Res<Time<Real>>, mut watch: ResMut<ThemeWatch>, mut accent: ResMut<Accent>) {
    if !watch.timer.tick(time.delta()).just_finished() {
        return;
    }
    let Some(path) = watch.path.clone() else { return };
    let now = modified(&path);
    if now == watch.modified {
        return;
    }
    watch.modified = now;
    let next = read_accent(&path).unwrap_or_else(fallback);
    if accent.0 != next {
        accent.0 = next;
    }
}

fn apply_accent(
    accent: Res<Accent>,
    mut texts: Query<&mut TextColor, With<AccentText>>,
    mut borders: Query<&mut BorderColor, With<AccentBorder>>,
    mut backgrounds: Query<&mut BackgroundColor, With<AccentBackground>>,
    added_text: Query<(), Added<AccentText>>,
    added_border: Query<(), Added<AccentBorder>>,
    added_bg: Query<(), Added<AccentBackground>>,
) {
    let fresh = !added_text.is_empty() || !added_border.is_empty() || !added_bg.is_empty();
    if !accent.is_changed() && !fresh {
        return;
    }
    for mut c in &mut texts {
        c.0 = accent.0;
    }
    for mut b in &mut borders {
        *b = BorderColor::all(accent.0);
    }
    for mut bg in &mut backgrounds {
        bg.0 = accent.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_accent_line() {
        let toml = "foreground = \"#ffffff\"\naccent = \"#b59790\"\n";
        assert_eq!(parse_accent(toml), Some(Color::srgb_u8(0xb5, 0x97, 0x90)));
    }

    #[test]
    fn ignores_similar_keys_and_bad_values() {
        assert_eq!(parse_accent("accent_dark = \"#000000\""), None);
        assert_eq!(parse_accent("accent = \"blue\""), None);
    }
}
