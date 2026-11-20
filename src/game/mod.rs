//! The Bevy adapter: everything the player sees and touches. The rules it applies live
//! in `crate::domain`, so this layer only translates input, physics, and time into
//! domain calls and draws the result.

use std::path::PathBuf;

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};

use crate::domain::levels::{self, Level};
use crate::domain::rng::Rng;
use crate::domain::rules::Match;
use crate::i18n::{Lang, Strings};
use crate::settings::{self, Settings};

pub use sound::Muted;

mod animation;
mod assets;
mod banner;
mod capture;
mod hud;
mod level;
mod menu;
mod overlay;
mod play;
mod question;
mod results;
mod sound;
mod theme;
mod ui;

pub const APP_ID: &str = "numhop";

/// The screens of the game. Level entities live across `Banner`, `Running`, and
/// `Asking`, and are rebuilt explicitly when the player or the world changes.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Loading,
    Menu,
    Banner,
    Running,
    Asking,
    Results,
}

#[derive(Resource)]
pub struct Worlds(pub Vec<Level>);

#[derive(Resource)]
pub struct GameRng(pub Rng);

/// The language everything is shown in. Changed from the Settings page and saved.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language(pub Lang);

impl Language {
    pub fn t(&self) -> &'static Strings {
        self.0.strings()
    }
}

/// The names the players typed in the menu; empty means "Player 1" or "Player 2".
/// Saved with the language.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerNames(pub [String; 2]);

/// The settings to start with, and whether changes are saved. `NUMHOP_LANG` picks the
/// language for one run without touching saved settings, and scripted screenshots
/// (`NUMHOP_CAPTURE`) start with blank names and sound shown on, so neither rewrites
/// the player's choices.
fn initial_settings() -> (Settings, bool) {
    let mut settings = settings::load_or_default();
    let mut persist = true;
    if capturing() {
        settings.names = Default::default();
        settings.muted = false;
        persist = false;
    }
    if let Some(lang) = std::env::var("NUMHOP_LANG").ok().and_then(|c| Lang::from_code(&c)) {
        settings.lang = lang;
        persist = false;
    }
    (settings, persist)
}

/// Scripted screenshots are running (`bin/screenshots`).
fn capturing() -> bool {
    std::env::var_os("NUMHOP_CAPTURE").is_some()
}

#[derive(Resource)]
struct PersistSettings(bool);

fn save_settings(language: Res<Language>, names: Res<PlayerNames>, muted: Res<Muted>, persist: Res<PersistSettings>) {
    if !persist.0 || language.is_added() || names.is_added() || muted.is_added() {
        return;
    }
    let settings = Settings {
        lang: language.0,
        names: names.0.clone(),
        muted: muted.0,
    };
    if let Some(path) = settings::settings_path()
        && let Err(err) = settings::save(&path, &settings)
    {
        warn!("could not save settings to {}: {err}", path.display());
    }
}

/// The match in progress. Present from the moment the menu starts a game.
#[derive(Resource)]
pub struct Game(pub Match);

pub fn run() -> AppExit {
    let (settings, persist) = initial_settings();
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "numhop".into(),
                        name: Some(APP_ID.into()),
                        resolution: (1280, 800).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_dir().to_string_lossy().into_owned(),
                    ..default()
                }),
        )
        .init_state::<Screen>()
        .insert_resource(Worlds(levels::worlds()))
        .insert_resource(GameRng(Rng::from_time()))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.78, 0.96)))
        .insert_resource(Language(settings.lang))
        .insert_resource(PlayerNames(settings.names))
        .insert_resource(Muted(settings.muted))
        // Scripted screenshots play silently, without changing what the menu shows.
        .insert_resource(GlobalVolume::new(if capturing() { Volume::SILENT } else { Volume::Linear(1.0) }))
        .insert_resource(PersistSettings(persist))
        .add_systems(
            Update,
            save_settings.run_if(
                resource_changed::<Language>
                    .or_else(resource_changed::<PlayerNames>)
                    .or_else(resource_changed::<Muted>),
            ),
        )
        .add_plugins((
            assets::plugin,
            animation::plugin,
            theme::plugin,
            overlay::plugin,
            menu::plugin,
            level::plugin,
            play::plugin,
            question::plugin,
            banner::plugin,
            hud::plugin,
            results::plugin,
            sound::plugin,
            capture::plugin,
        ))
        .run()
}

/// Where the models and fonts live. A development checkout uses its own `assets`
/// folder; an installed package uses `/usr/share/numhop/assets`. `NUMHOP_ASSETS`
/// overrides both.
fn asset_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("NUMHOP_ASSETS") {
        return PathBuf::from(dir);
    }
    let system = PathBuf::from("/usr/share/numhop/assets");
    // An installed binary uses the installed assets, even if the checkout it was
    // built from still exists on this machine.
    let installed = std::env::current_exe().is_ok_and(|exe| exe.starts_with("/usr"));
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !installed && dev.join("models").is_dir() {
        return dev;
    }
    system
}
