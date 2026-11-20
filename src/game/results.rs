//! The end of a match: who cleared the most worlds, then who scored the most.

use bevy::prelude::*;

use crate::domain::records::is_new_best;
use crate::domain::rules::Match;
use crate::i18n::{Strings, fill};

use super::assets::GameAssets;
use super::banner::turn_banner;
use super::level::PendingBuild;
use super::overlay::no_overlay;
use super::records::BestBefore;
use super::theme::{AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Language, Screen, Worlds};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Results), show)
        .add_systems(Update, choose.run_if(in_state(Screen::Results).and_then(no_overlay)));
}

fn show(
    mut commands: Commands,
    game: Res<Game>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
    language: Res<Language>,
    best_before: Option<Res<BestBefore>>,
) {
    let a = &*assets;
    let t = language.t();
    let m = &game.0;
    let ranking = m.ranking();
    let headline = if m.solo() {
        let p = &m.players[0];
        if p.finished { t.solo_cleared.to_string() } else { t.great_run.to_string() }
    } else {
        let (first, second) = (&m.players[ranking[0]], &m.players[ranking[1]]);
        if first.world == second.world && first.score == second.score {
            t.tie.to_string()
        } else {
            fill(t.wins, &[&first.config.label])
        }
    };
    let card = commands
        .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL_SOLID), BorderColor::all(ui::TEXT), AccentBorder))
        .with_children(|c| {
            c.spawn(label(a, t.results, 24.0, ui::MUTED));
            c.spawn((label(a, headline, 46.0, ui::TEXT), AccentText));
            for line in result_lines(t, m, best_before.as_deref()) {
                c.spawn(label(a, line, 22.0, ui::TEXT));
            }
            c.spawn(label(a, t.results_hint, 18.0, ui::MUTED));
        })
        .id();
    commands
        .spawn((DespawnOnExit(Screen::Results), ui::fullscreen(ui::DIM, 50)))
        .add_child(card);
}

/// One line per player, best first, marking scores that beat that player's best from
/// before the match.
fn result_lines(t: &Strings, m: &Match, best_before: Option<&BestBefore>) -> Vec<String> {
    m.ranking()
        .into_iter()
        .enumerate()
        .map(|(place, seat)| {
            let p = &m.players[seat];
            let line = fill(
                t.result_line,
                &[
                    &t.ordinal(place),
                    &p.config.label,
                    &p.config.character.name(),
                    &p.world.min(m.total_worlds),
                    &m.total_worlds,
                    &p.score,
                    &p.gems,
                ],
            );
            // The snapshot is indexed by seat, not by place in the ranking.
            if is_new_best(p.score, best_before.and_then(|b| b.0.get(seat).copied())) {
                format!("{line}  ·  {}", t.new_best)
            } else {
                line
            }
        })
        .collect()
}

fn choose(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    worlds: Res<Worlds>,
    language: Res<Language>,
    mut commands: Commands,
    mut next: ResMut<NextState<Screen>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        // Same players, same picks, fresh match.
        let configs = game.0.players.iter().map(|p| p.config.clone()).collect();
        let fresh = Game(Match::new(configs, worlds.0.len()));
        let t = language.t();
        let banner = turn_banner(t, &fresh, &worlds, t.ready.into(), None);
        commands.insert_resource(fresh);
        commands.insert_resource(banner);
        commands.insert_resource(PendingBuild);
        next.set(Screen::Banner);
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Escape]) {
        next.set(Screen::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::math::AgeLevel;
    use crate::domain::rules::{Character, PlayerConfig};
    use crate::i18n::Lang;

    fn duo(scores: [u32; 2]) -> Match {
        let config = |label: &str| PlayerConfig {
            label: label.into(),
            age: AgeLevel::Eight,
            character: Character::Pip,
            start_world: 0,
        };
        let mut m = Match::new(vec![config("Lucía"), config("Mateo")], 5);
        m.players[0].score = scores[0];
        m.players[1].score = scores[1];
        m
    }

    #[test]
    fn new_best_marks_follow_each_seat_not_the_ranking() {
        let t = Lang::En.strings();
        // Mateo (seat 2) wins and is listed first; only Lucía beat her old best.
        let m = duo([300, 900]);
        let lines = result_lines(t, &m, Some(&BestBefore(vec![200, 1000])));
        assert!(lines[0].contains("Mateo") && !lines[0].contains(t.new_best), "{lines:?}");
        assert!(lines[1].contains("Lucía") && lines[1].contains(t.new_best), "{lines:?}");
    }

    #[test]
    fn no_snapshot_means_no_new_best_claims() {
        let t = Lang::En.strings();
        let lines = result_lines(t, &duo([300, 900]), None);
        assert!(lines.iter().all(|l| !l.contains(t.new_best)), "{lines:?}");
    }
}
