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

    #[cfg(test)]
    pub fn is_unsaved(&self) -> bool {
        self.unsaved
    }

    /// Something changed that the next banner should save.
    pub fn mark_unsaved(&mut self) {
        self.unsaved = true;
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
    app.insert_resource(load());
    lifecycle(app);
}

/// When records are merged, and when the before-the-match snapshot is taken and dropped.
fn lifecycle(app: &mut App) {
    app.add_systems(OnEnter(Screen::Banner), keep_records)
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
    load_from(settings::records_path())
}

fn load_from(path: Option<PathBuf>) -> SavedRecords {
    let Some(path) = path else { return SavedRecords::in_memory(Records::default()) };
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
        // Being in world N means N worlds were cleared, and clearing the last world leaves
        // `world` at the total. That holds for a chosen start only because the menu limits
        // it to worlds already reached; `Match::new` itself only keeps it inside the game.
        let worlds_cleared = p.world;
        let gain = saved.records.merge(&SavedRecords::key(&names, i), p.score, worlds_cleared);
        saved.unsaved |= gain.best_score || gain.worlds_cleared;
    }
    if !saved.unsaved {
        return;
    }
    let Some(path) = saved.path.clone() else { return };
    // Another copy of the game may have saved since this one started; fold that in
    // first, so this save does not erase it. If the file cannot be read now, do not
    // write over it either: try again at the next banner.
    match settings::load_records(&path) {
        Ok(on_disk) => saved.records.absorb(&on_disk),
        Err(err) => {
            warn!("could not read records from {}: {err}; will try saving again", path.display());
            return;
        }
    }
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

    /// The match and its records, with `keep_records` running every frame.
    fn app_with(records: Records) -> App {
        let mut app = match_app(records);
        app.add_systems(Update, keep_records);
        app
    }

    /// The match and its records, with no records systems yet.
    fn match_app(records: Records) -> App {
        let config = PlayerConfig {
            label: "Lucía".into(),
            age: AgeLevel::Eight,
            character: Character::Pip,
            start_world: 0,
        };
        let mut app = App::new();
        app.insert_resource(Game(Match::new(vec![config], 5)))
            .insert_resource(PlayerNames(["Lucía".into(), String::new()]))
            .insert_resource(SavedRecords::in_memory(records));
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

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("numhop-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn saving_keeps_what_another_copy_of_the_game_saved_meanwhile() {
        let dir = temp_dir("two-copies");
        let path = dir.join("records.toml");
        let mut app = app_with(Records::default());
        app.insert_resource(SavedRecords { records: Records::default(), path: Some(path.clone()), unsaved: false });
        app.update();

        // The other window saves Mateo's progress after this one loaded its records.
        let mut theirs = Records::default();
        theirs.merge("mateo", 640, 2);
        settings::save_records(&path, &theirs).unwrap();

        app.world_mut().resource_mut::<Game>().0.player_mut().world = 1;
        app.update();
        let on_disk = settings::load_records(&path).unwrap();
        assert_eq!(on_disk.get("mateo"), Record { best_score: 640, worlds_cleared: 2 });
        assert_eq!(on_disk.get("lucía").worlds_cleared, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn practice_alone_is_saved_at_the_next_banner() {
        use crate::domain::math::{Topic, generate_topic};
        use crate::domain::practice::RetryQueue;
        use crate::domain::rng::Rng;
        let dir = temp_dir("practice-save");
        let path = dir.join("records.toml");
        let mut app = app_with(Records::default());
        app.insert_resource(SavedRecords { records: Records::default(), path: Some(path.clone()), unsaved: false });
        app.update();
        // No score changes; only an answer recorded during the question.
        {
            let mut saved = app.world_mut().resource_mut::<SavedRecords>();
            let question = generate_topic(Topic::Division, 2, &mut Rng::new(2));
            saved.records.record_answer("lucía", &mut RetryQueue::default(), &question, false, false);
            saved.mark_unsaved();
        }
        app.update();
        let on_disk = settings::load_records(&path).unwrap();
        assert_eq!(on_disk.practice("lucía").0[&Topic::Division].misses, 2);
        assert!(!app.world().resource::<SavedRecords>().unsaved);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unreadable_records_file_is_never_saved_over() {
        // A folder where the file should be cannot be read as one.
        let dir = temp_dir("unreadable");
        let path = dir.join("records.toml");
        std::fs::create_dir_all(&path).unwrap();
        let saved = load_from(Some(path));
        assert!(saved.path.is_none(), "this run must keep its progress in memory only");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_snapshot_lives_for_one_match_and_play_again_takes_a_new_one() {
        use bevy::state::app::StatesPlugin;
        let mut old = Records::default();
        old.merge("lucía", 800, 0);
        let mut app = match_app(old);
        app.add_plugins(StatesPlugin).init_state::<Screen>();
        lifecycle(&mut app);
        let go = |app: &mut App, screen: Screen| {
            app.world_mut().resource_mut::<NextState<Screen>>().set(screen);
            app.update();
        };
        let best_before = |app: &App| app.world().get_resource::<BestBefore>().map(|b| b.0.clone());

        go(&mut app, Screen::Banner);
        assert_eq!(best_before(&app), Some(vec![800]));
        app.world_mut().resource_mut::<Game>().0.player_mut().score = 1200;
        go(&mut app, Screen::Running);
        go(&mut app, Screen::Results);
        assert_eq!(best_before(&app), Some(vec![800]), "the results judge against the start of the match");

        // Play again: the new match is judged against the 1200 just saved.
        go(&mut app, Screen::Banner);
        assert_eq!(best_before(&app), Some(vec![1200]));
        go(&mut app, Screen::Menu);
        assert_eq!(best_before(&app), None, "leaving for the menu drops the snapshot");
    }

    #[test]
    fn a_failed_save_is_tried_again_at_the_next_banner() {
        let dir = temp_dir("retry");
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
