//! The Bevy adapter: everything the player sees and touches. The rules it applies live
//! in `crate::domain`, so this layer only translates input, physics, and time into
//! domain calls and draws the result.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};

use crate::domain::levels::{self, Level};
use crate::domain::rng::Rng;
use crate::domain::rules::Match;
use crate::i18n::{Lang, Strings};
use crate::settings::{self, Settings};

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

/// `NUMHOP_LANG` picks the language for one run without touching saved settings, which
/// keeps scripted screenshots and tests from rewriting the player's choice.
fn initial_language() -> (Lang, bool) {
    if let Some(lang) = std::env::var("NUMHOP_LANG").ok().and_then(|c| Lang::from_code(&c)) {
        return (lang, false);
    }
    let persist = std::env::var_os("NUMHOP_CAPTURE").is_none();
    (settings::load_or_default().lang, persist)
}

#[derive(Resource)]
struct PersistLanguage(bool);

fn save_language(language: Res<Language>, persist: Res<PersistLanguage>) {
    if !persist.0 || language.is_added() {
        return;
    }
    if let Some(path) = settings::settings_path()
        && let Err(err) = settings::save(&path, &Settings { lang: language.0 })
    {
        warn!("could not save settings to {}: {err}", path.display());
    }
}

/// The match in progress. Present from the moment the menu starts a game.
#[derive(Resource)]
pub struct Game(pub Match);

pub fn run() -> AppExit {
    let (lang, persist) = initial_language();
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
        .insert_resource(Language(lang))
        .insert_resource(PersistLanguage(persist))
        .add_systems(Update, save_language.run_if(resource_changed::<Language>))
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
