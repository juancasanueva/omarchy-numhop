//! The end of a match: who cleared the most worlds, then who scored the most.

use bevy::prelude::*;

use crate::domain::rules::Match;

use super::assets::GameAssets;
use super::banner::turn_banner;
use super::level::PendingBuild;
use super::overlay::no_overlay;
use super::theme::{AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Screen, Worlds};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Results), show)
        .add_systems(Update, choose.run_if(in_state(Screen::Results).and_then(no_overlay)));
}

fn ordinal(n: usize) -> &'static str {
    match n {
        0 => "1st",
        1 => "2nd",
        _ => "3rd",
    }
}

fn show(mut commands: Commands, game: Res<Game>, assets: Res<GameAssets>, rounding: Res<Rounding>) {
    let a = &*assets;
    let m = &game.0;
    let ranking = m.ranking();
    let headline = if m.solo() {
        let p = &m.players[0];
        if p.finished { "You cleared every world!".to_string() } else { "Great run!".to_string() }
    } else {
        let (first, second) = (&m.players[ranking[0]], &m.players[ranking[1]]);
        if first.world == second.world && first.score == second.score {
            "It's a tie!".to_string()
        } else {
            format!("{} wins!", first.config.label)
        }
    };
    let card = commands
        .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL_SOLID), BorderColor::all(ui::TEXT), AccentBorder))
        .with_children(|c| {
            c.spawn(label(a, "Results", 24.0, ui::MUTED));
            c.spawn((label(a, headline, 46.0, ui::TEXT), AccentText));
            for (place, &i) in ranking.iter().enumerate() {
                let p = &m.players[i];
                c.spawn(label(
                    a,
                    format!(
                        "{}  {} · {}  —  {} of {} worlds · {} pts · {} gems",
                        ordinal(place),
                        p.config.label,
                        p.config.character.name(),
                        p.world.min(m.total_worlds),
                        m.total_worlds,
                        p.score,
                        p.gems
                    ),
                    22.0,
                    ui::TEXT,
                ));
            }
            c.spawn(label(a, "Space play again · Enter menu · Q quit", 18.0, ui::MUTED));
        })
        .id();
    commands
        .spawn((DespawnOnExit(Screen::Results), ui::fullscreen(ui::DIM, 50)))
        .add_child(card);
}

fn choose(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    worlds: Res<Worlds>,
    mut commands: Commands,
    mut next: ResMut<NextState<Screen>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        // Same players, same picks, fresh match.
        let configs = game.0.players.iter().map(|p| p.config.clone()).collect();
        let fresh = Game(Match::new(configs, worlds.0.len()));
        let banner = turn_banner(&fresh, &worlds, "Ready?".into(), None);
        commands.insert_resource(fresh);
        commands.insert_resource(banner);
        commands.insert_resource(PendingBuild);
        next.set(Screen::Banner);
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Escape]) {
        next.set(Screen::Menu);
    }
}
