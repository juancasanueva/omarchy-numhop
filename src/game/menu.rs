//! The start menu: one or two players, each with an age level and a hero.
//! Choices are remembered between matches.

use bevy::prelude::*;

use crate::domain::math::AgeLevel;
use crate::domain::rules::{Character, Match, PlayerConfig};

use super::assets::GameAssets;
use super::banner::turn_banner;
use super::level::{LevelEntity, MainCamera, PendingBuild, facing_rotation, hero_model};
use super::overlay::no_overlay;
use super::theme::{Accent, AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Screen, Worlds};

#[derive(Resource, Debug, Clone)]
pub struct MenuChoices {
    pub players: usize,
    pub configs: [PlayerConfig; 2],
    pub row: usize,
}

impl Default for MenuChoices {
    fn default() -> Self {
        Self {
            players: 1,
            configs: [
                PlayerConfig {
                    label: "Player 1".into(),
                    age: AgeLevel::Six,
                    character: Character::Pip,
                },
                PlayerConfig {
                    label: "Player 2".into(),
                    age: AgeLevel::Twelve,
                    character: Character::Bolt,
                },
            ],
            row: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Players,
    Age(usize),
    Hero(usize),
    Start,
}

impl MenuChoices {
    fn rows(&self) -> Vec<Row> {
        let mut rows = vec![Row::Players, Row::Age(0), Row::Hero(0)];
        if self.players == 2 {
            rows.extend([Row::Age(1), Row::Hero(1)]);
        }
        rows.push(Row::Start);
        rows
    }
}

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct MenuHero(usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<MenuChoices>()
        .add_systems(OnEnter(Screen::Menu), (enter, draw).chain())
        .add_systems(
            Update,
            (navigate, draw.run_if(resource_changed::<MenuChoices>), spin_heroes)
                .chain()
                .run_if(in_state(Screen::Menu).and_then(no_overlay)),
        );
}

fn enter(
    mut commands: Commands,
    old: Query<Entity, With<LevelEntity>>,
    assets: Res<GameAssets>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut clear: ResMut<ClearColor>,
) {
    commands.remove_resource::<Game>();
    commands.remove_resource::<PendingBuild>();
    for e in &old {
        commands.entity(e).despawn();
    }
    clear.0 = Color::srgb(0.53, 0.78, 0.96);
    if let Ok(mut cam) = camera.single_mut() {
        *cam = Transform::from_xyz(0.0, 2.2, 7.5).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y);
    }
    // A little stage of ground for the heroes to stand on, with some scenery.
    for x in -8..=8 {
        commands.spawn((
            DespawnOnExit(Screen::Menu),
            WorldAssetRoot(assets.ground.clone()),
            Transform::from_xyz(x as f32, 0.0, 0.0),
        ));
    }
    for (x, z, model) in [(-4.5, -2.0, &assets.tree), (4.8, -2.5, &assets.tree), (-2.6, -1.0, &assets.bush), (2.8, -1.2, &assets.bush)] {
        commands.spawn((
            DespawnOnExit(Screen::Menu),
            WorldAssetRoot(model.clone()),
            Transform::from_xyz(x, 0.0, z),
        ));
    }
    commands.spawn((
        DespawnOnExit(Screen::Menu),
        WorldAssetRoot(assets.cloud.clone()),
        Transform::from_xyz(-3.0, 5.0, -6.0),
    ));
    commands.spawn((
        DespawnOnExit(Screen::Menu),
        WorldAssetRoot(assets.crystal.clone()),
        Transform::from_xyz(0.0, 3.6, -1.0).with_scale(Vec3::splat(0.8)),
    ));
}

fn draw(
    mut commands: Commands,
    choices: Res<MenuChoices>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
    accent: Res<Accent>,
    old_ui: Query<Entity, With<MenuUi>>,
    old_heroes: Query<Entity, With<MenuHero>>,
) {
    for e in old_ui.iter().chain(old_heroes.iter()) {
        commands.entity(e).despawn();
    }
    let a = &*assets;

    // Heroes on the stage reflect the current picks.
    let positions: &[f32] = if choices.players == 1 { &[-1.6] } else { &[-2.2, -0.9] };
    for (i, x) in positions.iter().enumerate() {
        commands.spawn((
            MenuHero(i),
            DespawnOnExit(Screen::Menu),
            WorldAssetRoot(hero_model(a, choices.configs[i].character)),
            Transform::from_xyz(*x, 0.0, 1.0).with_scale(Vec3::splat(1.6)).with_rotation(facing_rotation(1.0)),
        ));
    }

    let rows = choices.rows();
    let card = commands
        .spawn((
            Node {
                min_width: px(470),
                ..ui::card(rounding.0)
            },
            BackgroundColor(ui::PANEL),
            BorderColor::all(ui::TEXT),
            AccentBorder,
        ))
        .with_children(|c| {
            c.spawn((label(a, "numhop", 64.0, ui::TEXT), AccentText));
            c.spawn(label(a, "Hop, jump, and solve!", 20.0, ui::MUTED));
        })
        .id();

    for (i, row) in rows.iter().enumerate() {
        let selected = i == choices.row;
        let (name, value) = match row {
            Row::Players => ("Players".to_string(), format!("{}", choices.players)),
            Row::Age(p) => (format!("{} level", choices.configs[*p].label), choices.configs[*p].age.label().to_string()),
            Row::Hero(p) => (format!("{} hero", choices.configs[*p].label), choices.configs[*p].character.name().to_string()),
            Row::Start => ("Start".to_string(), String::new()),
        };
        let border = if selected { accent.0 } else { Color::NONE };
        let row_entity = commands
            .spawn((
                Node {
                    width: percent(100),
                    justify_content: if *row == Row::Start { JustifyContent::Center } else { JustifyContent::SpaceBetween },
                    column_gap: px(30),
                    padding: UiRect::axes(px(16), px(10)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(rounding.0.min(8.0))),
                    ..default()
                },
                BorderColor::all(border),
                BackgroundColor(if selected { Color::srgba(1.0, 1.0, 1.0, 0.08) } else { Color::NONE }),
            ))
            .with_children(|r| {
                let size = if *row == Row::Start { 28.0 } else { 20.0 };
                r.spawn(label(a, name, size, if *row == Row::Start { ui::TEXT } else { ui::MUTED }));
                if !value.is_empty() {
                    let shown = if selected { format!("‹  {value}  ›") } else { value };
                    r.spawn(label(a, shown, 20.0, ui::TEXT));
                }
            })
            .id();
        commands.entity(card).add_child(row_entity);
    }
    commands.entity(card).with_child(label(a, "Up/Down choose · Left/Right change · Enter start · ? help", 15.0, ui::MUTED));

    commands
        .spawn((
            MenuUi,
            DespawnOnExit(Screen::Menu),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                padding: UiRect::right(px(70)),
                ..default()
            },
        ))
        .add_child(card);
}

fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    mut choices: ResMut<MenuChoices>,
    worlds: Res<Worlds>,
    mut commands: Commands,
) {
    let rows = choices.rows();
    let row = rows[choices.row.min(rows.len() - 1)];
    if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Tab]) {
        choices.row = (choices.row + 1) % rows.len();
    }
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        choices.row = (choices.row + rows.len() - 1) % rows.len();
    }
    let change = keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::KeyA, KeyCode::KeyD]);
    let confirm = keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    let start = keys.just_pressed(KeyCode::Space) || (confirm && row == Row::Start);

    if start {
        let configs = choices.configs[..choices.players].to_vec();
        let game = Game(Match::new(configs, worlds.0.len()));
        let banner = turn_banner(&game, &worlds, "Ready?".into(), None);
        commands.insert_resource(game);
        commands.insert_resource(banner);
        commands.insert_resource(PendingBuild);
        commands.set_state(Screen::Banner);
        return;
    }
    if change || confirm {
        match row {
            Row::Players => {
                choices.players = if choices.players == 1 { 2 } else { 1 };
                choices.row = 0;
            }
            Row::Age(p) => choices.configs[p].age = choices.configs[p].age.toggle(),
            Row::Hero(p) => choices.configs[p].character = choices.configs[p].character.next(),
            Row::Start => {}
        }
    }
}

fn spin_heroes(time: Res<Time>, mut heroes: Query<(&mut Transform, &MenuHero)>) {
    let t = time.elapsed_secs();
    for (mut tf, hero) in &mut heroes {
        let wobble = (t * 1.3 + hero.0 as f32 * 1.7).sin() * 0.35;
        tf.rotation = facing_rotation(1.0) * Quat::from_rotation_y(wobble);
        tf.translation.y = ((t * 3.0 + hero.0 as f32).sin().abs()) * 0.12;
    }
}
