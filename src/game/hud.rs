//! The heads-up display: one panel per player, the current world and difficulty, a
//! controls hint, and short toasts like "+100".

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

use crate::domain::math::tier_stars;

use super::assets::GameAssets;
use super::level::{Grid, Player};
use super::theme::{Accent, AccentText, Rounding};
use super::ui::{self, label, symbol_font};
use super::{Game, Language, Screen, Worlds};
use crate::i18n::fill;

#[derive(Resource, Default)]
pub struct Toast {
    text: String,
    remaining: f32,
}

impl Toast {
    pub fn show(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.remaining = 1.6;
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct PlayerPanel(usize);

#[derive(Component)]
struct PanelLives(usize);

#[derive(Component)]
struct PanelStats(usize);

#[derive(Component)]
struct WorldText;

#[derive(Component)]
struct TierText;

#[derive(Component)]
struct ToastText;

pub fn plugin(app: &mut App) {
    app.init_resource::<Toast>().add_systems(
        Update,
        (spawn_or_despawn, update.run_if(resource_exists::<Game>)).chain().run_if(not(in_state(Screen::Loading))),
    );
}

fn spawn_or_despawn(
    mut commands: Commands,
    game: Option<Res<Game>>,
    screen: Res<State<Screen>>,
    roots: Query<Entity, With<HudRoot>>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
    language: Res<Language>,
) {
    let t = language.t();
    let wanted = game.is_some() && matches!(screen.get(), Screen::Banner | Screen::Running | Screen::Asking);
    let exists = !roots.is_empty();
    if wanted == exists {
        return;
    }
    if !wanted {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    }
    let game = game.unwrap();
    let a = &*assets;
    let radius = rounding.0;

    let root = commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(px(16)),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .id();

    let top = commands
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            ..default()
        })
        .id();

    let panels = commands
        .spawn(Node {
            column_gap: px(12),
            ..default()
        })
        .id();
    for (i, p) in game.0.players.iter().enumerate() {
        let panel = commands
            .spawn((
                PlayerPanel(i),
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(px(14), px(10)),
                    row_gap: px(2),
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(radius)),
                    min_width: px(220),
                    ..default()
                },
                BackgroundColor(ui::PANEL),
                BorderColor::all(Color::NONE),
            ))
            .with_children(|c| {
                c.spawn(label(a, format!("{} · {}", p.config.label, p.config.character.name()), 20.0, ui::TEXT));
                c.spawn((PanelLives(i), Text::new(""), symbol_font(a, 22.0), TextColor(ui::BAD)));
                c.spawn((PanelStats(i), label(a, "", 16.0, ui::MUTED)));
            })
            .id();
        commands.entity(panels).add_child(panel);
    }

    let world_box = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                padding: UiRect::axes(px(14), px(10)),
                border_radius: BorderRadius::all(px(radius)),
                ..default()
            },
            BackgroundColor(ui::PANEL),
        ))
        .with_children(|c| {
            c.spawn((WorldText, label(a, "", 20.0, ui::TEXT)));
            c.spawn(Node { column_gap: px(8), align_items: AlignItems::Center, ..default() })
                .with_children(|r| {
                    r.spawn(label(a, t.difficulty, 16.0, ui::MUTED));
                    r.spawn((TierText, Text::new(""), symbol_font(a, 20.0), TextColor(ui::TEXT), AccentText));
                });
        })
        .id();
    commands.entity(top).add_children(&[panels, world_box]);

    let toast = commands
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_child((ToastText, label(a, "", 34.0, ui::TEXT)))
        .id();

    let hint = commands
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_child(label(
            a,
            t.hud_hint,
            16.0,
            Color::srgba(1.0, 1.0, 1.0, 0.8),
        ))
        .id();

    commands.entity(root).add_children(&[top, toast, hint]);
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update(
    time: Res<Time<Real>>,
    game: Res<Game>,
    worlds: Res<Worlds>,
    grid: Res<Grid>,
    accent: Res<Accent>,
    language: Res<Language>,
    mut toast: ResMut<Toast>,
    player: Query<&Transform, With<Player>>,
    mut panels: Query<(&PlayerPanel, &mut BorderColor)>,
    mut texts: ParamSet<(
        Query<(&PanelLives, &mut Text)>,
        Query<(&PanelStats, &mut Text)>,
        Query<&mut Text, With<WorldText>>,
        Query<&mut Text, With<TierText>>,
        Query<(&mut Text, &mut TextColor), With<ToastText>>,
    )>,
) {
    let m = &game.0;
    let t = language.t();
    for (panel, mut border) in &mut panels {
        let color = if panel.0 == m.current && !m.solo() { accent.0 } else { Color::NONE };
        *border = BorderColor::all(color);
    }
    for (l, mut text) in &mut texts.p0() {
        let p = &m.players[l.0];
        // The symbols font has no Latin letters, so status words go in the stats line.
        text.0 = if p.finished { "★★★".into() } else { "♥".repeat(p.lives as usize) };
    }
    for (s, mut text) in &mut texts.p1() {
        let p = &m.players[s.0];
        let status = if p.finished {
            t.status_cleared.to_string()
        } else if p.lives == 0 {
            t.status_out.to_string()
        } else {
            String::new()
        };
        text.0 = fill(
            t.hud_stats,
            &[&status, &p.score, &p.gems, &(p.world + 1).min(m.total_worlds), &m.total_worlds],
        );
    }
    let world = grid.world.min(worlds.0.len() - 1);
    for mut text in &mut texts.p2() {
        text.0 = fill(t.hud_world, &[&(world + 1), &t.world(world)]);
    }
    let ratio = player.single().map(|t| t.translation.x / grid.width.max(1) as f32).unwrap_or(0.0);
    let tier = m.tier(ratio);
    for mut text in &mut texts.p3() {
        text.0 = tier_stars(tier);
    }
    toast.remaining = (toast.remaining - time.delta_secs()).max(0.0);
    let alpha = (toast.remaining / 0.4).min(1.0);
    for (mut text, mut color) in &mut texts.p4() {
        if text.0 != toast.text {
            text.0 = toast.text.clone();
        }
        color.0 = accent.0.with_alpha(alpha);
    }
}
