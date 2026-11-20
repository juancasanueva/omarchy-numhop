//! Full-screen messages between moments of play: whose turn it is, what just happened,
//! and what comes next. This is also where domain outcomes become screen changes.

use bevy::prelude::*;

use crate::domain::rules::{AfterDeath, AfterGate};

use super::assets::GameAssets;
use super::level::PendingBuild;
use super::overlay::no_overlay;
use super::theme::{AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Screen, Worlds};

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

fn lives_line(lives: u8) -> String {
    match lives {
        0 => "No lives left".into(),
        1 => "1 life left".into(),
        n => format!("{n} lives left"),
    }
}

/// The banner shown before someone (re)starts playing.
pub fn turn_banner(game: &Game, worlds: &Worlds, title: String, extra: Option<String>) -> Banner {
    let p = game.0.player();
    let world = p.world.min(worlds.0.len() - 1);
    let mut lines = Vec::new();
    if let Some(extra) = extra {
        lines.push(extra);
    }
    lines.push(format!(
        "{} — World {}/{}: {}",
        player_tag(game, game.0.current),
        world + 1,
        game.0.total_worlds,
        worlds.0[world].name
    ));
    lines.push(format!("{} · {}", lives_line(p.lives), p.config.age.label()));
    Banner {
        title,
        lines,
        then: Then::Play,
    }
}

/// Applies a death to the match and moves to the right banner.
pub fn after_death(commands: &mut Commands, game: &mut Game, worlds: &Worlds, reason: &str) {
    let who = player_tag(game, game.0.current);
    let lives = game.0.player().lives;
    let banner = match game.0.died() {
        AfterDeath::Continue => turn_banner(game, worlds, reason.to_string(), Some(format!("{who} lost a life"))),
        AfterDeath::SwitchTo(next) => {
            let extra = format!("{who}: {}. Over to {}!", lives_line(lives.saturating_sub(1)), game.0.players[next].config.label);
            turn_banner(game, worlds, reason.to_string(), Some(extra))
        }
        AfterDeath::GameOver => Banner {
            title: "Game over".into(),
            lines: vec![reason.to_string(), "Let's see the scores.".into()],
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
pub fn after_gate(commands: &mut Commands, game: &mut Game, worlds: &Worlds) {
    let who = player_tag(game, game.0.current);
    let banner = match game.0.gate_opened() {
        AfterGate::NextWorld => turn_banner(game, worlds, "World cleared! +500".into(), None),
        AfterGate::Finished(AfterDeath::SwitchTo(next)) => {
            let extra = format!("{who} cleared every world! Over to {}.", game.0.players[next].config.label);
            turn_banner(game, worlds, "Champion!".into(), Some(extra))
        }
        AfterGate::Finished(_) => Banner {
            title: "Champion!".into(),
            lines: vec![format!("{who} cleared every world!"), "Let's see the scores.".into()],
            then: Then::Results,
        },
    };
    if banner.then == Then::Play {
        commands.insert_resource(PendingBuild);
    }
    commands.insert_resource(banner);
    commands.set_state(Screen::Banner);
}

fn show(mut commands: Commands, banner: Res<Banner>, assets: Res<GameAssets>, rounding: Res<Rounding>) {
    let a = &*assets;
    let card = commands
        .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL), BorderColor::all(ui::TEXT), AccentBorder))
        .with_children(|c| {
            c.spawn((label(a, banner.title.clone(), 46.0, ui::TEXT), AccentText));
            for line in &banner.lines {
                c.spawn(label(a, line.clone(), 22.0, ui::TEXT));
            }
            let hint = match banner.then {
                Then::Play => "Press Space to play",
                Then::Results => "Press Space for the results",
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
