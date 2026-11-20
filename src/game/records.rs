//! Keeps the players' records between runs: loaded at startup, merged with the match
//! whenever a banner or the results appear (after every gate, death and match end),
//! and saved when something improved.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::domain::records::{Records, record_key};
use crate::settings;

use super::{Game, PlayerNames, Screen, capturing};

/// The records, and where to save them; `None` when nothing may be written.
#[derive(Resource, Default)]
pub struct SavedRecords {
    pub records: Records,
    path: Option<PathBuf>,
}

impl SavedRecords {
    /// Records that are used but never saved.
    pub fn in_memory(records: Records) -> Self {
        Self { records, path: None }
    }

    /// The record key for seat `seat`, from the names typed in the menu.
    pub fn key(names: &PlayerNames, seat: usize) -> String {
        record_key(&names.0[seat], seat)
    }
}

/// Each player's best score from before this match, to tell a new best on the results.
#[derive(Resource, Debug)]
pub struct BestBefore(pub Vec<u32>);

pub fn plugin(app: &mut App) {
    app.insert_resource(load())
        .add_systems(OnEnter(Screen::Banner), keep_records)
        .add_systems(OnEnter(Screen::Results), keep_records)
        .add_systems(OnExit(Screen::Results), forget_best_before)
        .add_systems(OnEnter(Screen::Menu), forget_best_before);
}

/// Scripted screenshots start with no records and never save, so they neither depend on
/// nor change the player's real progress.
fn load() -> SavedRecords {
    if capturing() {
        return SavedRecords::in_memory(Records::default());
    }
    let path = settings::records_path();
    let records = path.as_deref().map(settings::load_records).unwrap_or_default();
    SavedRecords { records, path }
}

fn keep_records(
    game: Res<Game>,
    names: Res<PlayerNames>,
    mut saved: ResMut<SavedRecords>,
    best_before: Option<Res<BestBefore>>,
    mut commands: Commands,
) {
    let players = &game.0.players;
    if best_before.is_none() {
        let bests = (0..players.len()).map(|i| saved.records.get(&SavedRecords::key(&names, i)).best_score).collect();
        commands.insert_resource(BestBefore(bests));
    }
    let mut improved = false;
    for (i, p) in players.iter().enumerate() {
        let gain = saved.records.merge(&SavedRecords::key(&names, i), p.score, p.world);
        improved |= gain.best_score || gain.worlds_cleared;
    }
    if !improved {
        return;
    }
    if let Some(path) = &saved.path
        && let Err(err) = settings::save_records(path, &saved.records)
    {
        warn!("could not save records to {}: {err}", path.display());
    }
}

/// A finished or abandoned match leaves no snapshot behind for the next one.
fn forget_best_before(mut commands: Commands) {
    commands.remove_resource::<BestBefore>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::math::AgeLevel;
    use crate::domain::records::Record;
    use crate::domain::rules::{Character, Match, PlayerConfig};

    fn app_with(records: Records) -> App {
        let config = PlayerConfig {
            label: "Lucía".into(),
            age: AgeLevel::Eight,
            character: Character::Pip,
            start_world: 0,
        };
        let mut app = App::new();
        app.insert_resource(Game(Match::new(vec![config], 5)))
            .insert_resource(PlayerNames(["Lucía".into(), String::new()]))
            .insert_resource(SavedRecords::in_memory(records))
            .add_systems(Update, keep_records);
        app
    }

    #[test]
    fn the_match_merges_progress_but_judges_new_bests_against_the_start() {
        let mut old = Records::default();
        old.merge("lucía", 800, 1);
        let mut app = app_with(old);
        app.update();
        assert_eq!(app.world().resource::<BestBefore>().0, vec![800]);

        // Two gates later Lucía has 1200 points: kept at once, judged against 800.
        {
            let mut game = app.world_mut().resource_mut::<Game>();
            let p = game.0.player_mut();
            p.score = 1200;
            p.world = 2;
        }
        app.update();
        let saved = &app.world().resource::<SavedRecords>().records;
        assert_eq!(saved.get("lucía"), Record { best_score: 1200, worlds_cleared: 2 });
        assert_eq!(app.world().resource::<BestBefore>().0, vec![800], "the snapshot stays for the whole match");
    }
}
