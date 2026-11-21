//! Loads every model and font up front, and reports a missing file in the window
//! instead of starting a game with invisible pieces.

#![allow(clippy::too_many_arguments)]

use bevy::asset::LoadState;
use bevy::prelude::*;

use super::animation::{CharacterClips, ClipsMissing};
use super::{Language, Screen};
use crate::i18n::fill;

#[derive(Resource, Clone)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub symbols: Handle<Font>,
    pub hero_pip: Handle<WorldAsset>,
    pub hero_bolt: Handle<WorldAsset>,
    pub hero_fern: Handle<WorldAsset>,
    pub hero_pebble: Handle<WorldAsset>,
    pub enemy: Handle<WorldAsset>,
    pub crystal: Handle<WorldAsset>,
    pub used: Handle<WorldAsset>,
    pub brick: Handle<WorldAsset>,
    pub ground: Handle<WorldAsset>,
    pub gem: Handle<WorldAsset>,
    pub key: Handle<WorldAsset>,
    pub gate: Handle<WorldAsset>,
    pub flag: Handle<WorldAsset>,
    pub tree: Handle<WorldAsset>,
    pub bush: Handle<WorldAsset>,
    pub cloud: Handle<WorldAsset>,
    /// The animated characters' whole files, for their named clips.
    pub gltf_pip: Handle<Gltf>,
    pub gltf_bolt: Handle<Gltf>,
    pub gltf_fern: Handle<Gltf>,
    pub gltf_pebble: Handle<Gltf>,
    pub gltf_grumble: Handle<Gltf>,
}

impl GameAssets {
    fn all_models(&self) -> [(&'static str, &Handle<WorldAsset>); 16] {
        [
            ("hero_pip", &self.hero_pip),
            ("hero_bolt", &self.hero_bolt),
            ("hero_fern", &self.hero_fern),
            ("hero_pebble", &self.hero_pebble),
            ("enemy_grumble", &self.enemy),
            ("block_crystal", &self.crystal),
            ("block_used", &self.used),
            ("block_brick", &self.brick),
            ("ground_tile", &self.ground),
            ("gem", &self.gem),
            ("key", &self.key),
            ("gate", &self.gate),
            ("flag_checkpoint", &self.flag),
            ("tree", &self.tree),
            ("bush", &self.bush),
            ("cloud", &self.cloud),
        ]
    }
}

#[derive(Component)]
struct LoadingText;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load)
        .add_systems(Update, wait_for_assets.run_if(in_state(Screen::Loading)));
}

fn model(server: &AssetServer, name: &str) -> Handle<WorldAsset> {
    server.load(GltfAssetLabel::Scene(0).from_asset(format!("models/{name}.glb")))
}

fn load(mut commands: Commands, server: Res<AssetServer>, language: Res<Language>) {
    let assets = GameAssets {
        font: server.load("fonts/NotoSans-Bold.ttf"),
        symbols: server.load("fonts/NotoSansSymbols2-Regular.ttf"),
        hero_pip: model(&server, "hero_pip"),
        hero_bolt: model(&server, "hero_bolt"),
        hero_fern: model(&server, "hero_fern"),
        hero_pebble: model(&server, "hero_pebble"),
        enemy: model(&server, "enemy_grumble"),
        crystal: model(&server, "block_crystal"),
        used: model(&server, "block_used"),
        brick: model(&server, "block_brick"),
        ground: model(&server, "ground_tile"),
        gem: model(&server, "gem"),
        key: model(&server, "key"),
        gate: model(&server, "gate"),
        flag: model(&server, "flag_checkpoint"),
        tree: model(&server, "tree"),
        bush: model(&server, "bush"),
        cloud: model(&server, "cloud"),
        gltf_pip: server.load("models/hero_pip.glb"),
        gltf_bolt: server.load("models/hero_bolt.glb"),
        gltf_fern: server.load("models/hero_fern.glb"),
        gltf_pebble: server.load("models/hero_pebble.glb"),
        gltf_grumble: server.load("models/enemy_grumble.glb"),
    };
    commands.insert_resource(assets);
    commands.spawn((
        LoadingText,
        DespawnOnExit(Screen::Loading),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(Text::new(language.t().loading), TextColor(Color::WHITE))],
    ));
}

fn wait_for_assets(
    assets: Res<GameAssets>,
    server: Res<AssetServer>,
    mut next: ResMut<NextState<Screen>>,
    texts: Query<&Children, With<LoadingText>>,
    mut writer: Query<&mut Text>,
    language: Res<Language>,
    clips: Option<Res<CharacterClips>>,
    clips_missing: Option<Res<ClipsMissing>>,
) {
    let mut failed = Vec::new();
    if clips_missing.is_some() {
        failed.push("character animations");
    }
    let mut pending = 0;
    let fonts = [("NotoSans-Bold.ttf", assets.font.id().untyped()), ("NotoSansSymbols2-Regular.ttf", assets.symbols.id().untyped())];
    let models = assets.all_models().map(|(n, h)| (n, h.id().untyped()));
    for (name, id) in fonts.into_iter().chain(models) {
        match server.get_load_state(id) {
            Some(LoadState::Loaded) => {}
            Some(LoadState::Failed(_)) => failed.push(name),
            _ => pending += 1,
        }
    }
    if !failed.is_empty() {
        let message = fill(language.t().load_failed, &[&failed.join(", ")]);
        for children in &texts {
            for child in children.iter() {
                if let Ok(mut text) = writer.get_mut(child) {
                    text.0 = message.clone();
                }
            }
        }
        return;
    }
    // The menu shows animated heroes, so it waits for their clips too.
    if pending == 0 && clips.is_some() {
        next.set(Screen::Menu);
    }
}
