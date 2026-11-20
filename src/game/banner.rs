//! Full-screen messages between moments of play: whose turn it is, what just happened,
//! and what comes next. This is also where domain outcomes become screen changes.

use bevy::prelude::*;

use crate::domain::rules::{AfterDeath, AfterGate};
use crate::i18n::{Strings, fill};

use super::assets::GameAssets;
use super::level::PendingBuild;
use super::overlay::no_overlay;
use super::theme::{AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Language, Screen, Worlds};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    Play,
    Results,
}

#[derive(Resource, Debug, Clone)]
pub struct Banner {
    pub title: String,
    pub lines: Vec<String>,
    pub then: Then,
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Banner), show)
        .add_systems(Update, proceed.run_if(in_state(Screen::Banner).and_then(no_overlay)));
}

/// Who is up, in words: "Player 2 · Bolt".
pub fn player_tag(game: &Game, index: usize) -> String {
    let p = &game.0.players[index];
    format!("{} · {}", p.config.label, p.config.character.name())
}

/// The banner shown before someone (re)starts playing.
pub fn turn_banner(t: &Strings, game: &Game, worlds: &Worlds, title: String, extra: Option<String>) -> Banner {
    let p = game.0.player();
    let world = p.world.min(worlds.0.len() - 1);
    let mut lines = Vec::new();
    if let Some(extra) = extra {
        lines.push(extra);
    }
    lines.push(fill(
        t.world_line,
        &[&player_tag(game, game.0.current), &(world + 1), &game.0.total_worlds, &t.world(world)],
    ));
    lines.push(format!("{} · {}", t.lives(p.lives), t.age(p.config.age)));
    Banner {
        title,
        lines,
        then: Then::Play,
    }
}

/// Applies a death to the match and moves to the right banner.
pub fn after_death(commands: &mut Commands, t: &Strings, game: &mut Game, worlds: &Worlds, reason: &str) {
    let who = player_tag(game, game.0.current);
    let lives = game.0.player().lives;
    let banner = match game.0.died() {
        AfterDeath::Continue => turn_banner(t, game, worlds, reason.to_string(), Some(fill(t.lost_life, &[&who]))),
        AfterDeath::SwitchTo(next) => {
            let extra = fill(
                t.over_to,
                &[&who, &t.lives(lives.saturating_sub(1)), &game.0.players[next].config.label],
            );
            turn_banner(t, game, worlds, reason.to_string(), Some(extra))
        }
        AfterDeath::GameOver => Banner {
            title: t.game_over.into(),
            lines: vec![reason.to_string(), t.see_scores.into()],
            then: Then::Results,
        },
    };
    if banner.then == Then::Play {
        commands.insert_resource(PendingBuild);
    }
    commands.insert_resource(banner);
    commands.set_state(Screen::Banner);
}

/// Applies an opened gate to the match and moves to the right banner.
pub fn after_gate(commands: &mut Commands, t: &Strings, game: &mut Game, worlds: &Worlds) {
    let who = player_tag(game, game.0.current);
    let banner = match game.0.gate_opened() {
        AfterGate::NextWorld => turn_banner(t, game, worlds, t.world_cleared.into(), None),
        AfterGate::Finished(AfterDeath::SwitchTo(next)) => {
            let extra = fill(t.cleared_all_over_to, &[&who, &game.0.players[next].config.label]);
            turn_banner(t, game, worlds, t.champion.into(), Some(extra))
        }
        AfterGate::Finished(_) => Banner {
            title: t.champion.into(),
            lines: vec![fill(t.cleared_all, &[&who]), t.see_scores.into()],
            then: Then::Results,
        },
    };
    if banner.then == Then::Play {
        commands.insert_resource(PendingBuild);
    }
    commands.insert_resource(banner);
    commands.set_state(Screen::Banner);
}

fn show(
    mut commands: Commands,
    banner: Res<Banner>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
    language: Res<Language>,
) {
    let a = &*assets;
    let t = language.t();
    let card = commands
        .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL), BorderColor::all(ui::TEXT), AccentBorder))
        .with_children(|c| {
            c.spawn((label(a, banner.title.clone(), 46.0, ui::TEXT), AccentText));
            for line in &banner.lines {
                c.spawn(label(a, line.clone(), 22.0, ui::TEXT));
            }
            let hint = match banner.then {
                Then::Play => t.press_space_play,
                Then::Results => t.press_space_results,
            };
            c.spawn(label(a, hint, 18.0, ui::MUTED));
        })
        .id();
    commands
        .spawn((DespawnOnExit(Screen::Banner), ui::fullscreen(Color::srgba(0.0, 0.0, 0.0, 0.35), 50)))
        .add_child(card);
}

fn proceed(keys: Res<ButtonInput<KeyCode>>, banner: Res<Banner>, mut next: ResMut<NextState<Screen>>) {
    if keys.any_just_pressed([KeyCode::Space, KeyCode::Enter]) {
        next.set(match banner.then {
            Then::Play => Screen::Running,
            Then::Results => Screen::Results,
        });
    }
}
