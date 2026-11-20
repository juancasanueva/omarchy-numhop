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
    /// An improvement is not on disk yet; saving is retried until it is.
    unsaved: bool,
}

impl SavedRecords {
    /// Records that are used but never saved.
    pub fn in_memory(records: Records) -> Self {
        Self { records, path: None, unsaved: false }
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
    let Some(path) = settings::records_path() else { return SavedRecords::in_memory(Records::default()) };
    match settings::load_records(&path) {
        Ok(records) => SavedRecords { records, path: Some(path), unsaved: false },
        Err(err) => {
            // Saving now would replace records we could not read, so this run keeps
            // its progress in memory only.
            warn!("could not read records from {}: {err}; progress will not be saved this run", path.display());
            SavedRecords::in_memory(Records::default())
        }
    }
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
    for (i, p) in players.iter().enumerate() {
        // Being in world N means N worlds were cleared: a chosen start is limited to
        // worlds already reached, and clearing the last world leaves `world` at the total.
        let worlds_cleared = p.world;
        let gain = saved.records.merge(&SavedRecords::key(&names, i), p.score, worlds_cleared);
        saved.unsaved |= gain.best_score || gain.worlds_cleared;
    }
    if !saved.unsaved {
        return;
    }
    let Some(path) = saved.path.clone() else { return };
    match settings::save_records(&path, &saved.records) {
        Ok(()) => saved.unsaved = false,
        Err(err) => warn!("could not save records to {}: {err}; will try again", path.display()),
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

    #[test]
    fn a_failed_save_is_tried_again_at_the_next_banner() {
        let dir = std::env::temp_dir().join(format!("numhop-test-retry-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // A plain file where the folder should be makes the first save fail.
        let blocker = dir.join("numhop");
        std::fs::write(&blocker, "").unwrap();
        let path = blocker.join("records.toml");

        let mut app = app_with(Records::default());
        app.insert_resource(SavedRecords { records: Records::default(), path: Some(path.clone()), unsaved: false });
        app.world_mut().resource_mut::<Game>().0.player_mut().world = 1;
        app.update();
        assert!(app.world().resource::<SavedRecords>().unsaved, "the failed save is remembered");

        std::fs::remove_file(&blocker).unwrap();
        app.update();
        assert!(!app.world().resource::<SavedRecords>().unsaved);
        assert_eq!(settings::load_records(&path).unwrap().get("lucía").worlds_cleared, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
