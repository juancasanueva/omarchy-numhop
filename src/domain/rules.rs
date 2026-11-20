//! Match rules: players, lives, score, turns, and difficulty progression.
//!
//! One or two players. Each starts with three lives. A death (enemy, pit, wrong answer,
//! or running out of time) costs a life and, with two players, passes the turn. A right
//! answer keeps the controller. Difficulty grows with how far the player has come.

use std::collections::HashSet;

use super::math::{AgeLevel, MAX_TIER, MIN_TIER};

pub const START_LIVES: u8 = 3;
pub const QUESTION_SECONDS: f32 = 30.0;

pub const SCORE_CRYSTAL: u32 = 250;
pub const SCORE_GATE: u32 = 500;
pub const SCORE_STOMP: u32 = 100;
pub const SCORE_GEM: u32 = 20;
pub const GEMS_PER_CRYSTAL: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Character {
    Pip,
    Bolt,
}

impl Character {
    pub const ALL: [Character; 2] = [Character::Pip, Character::Bolt];

    pub fn name(self) -> &'static str {
        match self {
            Character::Pip => "Pip",
            Character::Bolt => "Bolt",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Character::Pip => Character::Bolt,
            Character::Bolt => Character::Pip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerConfig {
    pub label: String,
    pub age: AgeLevel,
    pub character: Character,
}

/// What a player has done inside the current world. It survives deaths so a player
/// does not have to answer the same crystal twice, and it resets on the next world.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldProgress {
    pub used_crystals: HashSet<usize>,
    pub collected_gems: HashSet<usize>,
    pub checkpoint: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub config: PlayerConfig,
    pub lives: u8,
    pub score: u32,
    pub gems: u32,
    pub world: usize,
    pub finished: bool,
    pub progress: WorldProgress,
}

impl Player {
    fn new(config: PlayerConfig) -> Self {
        Self {
            config,
            lives: START_LIVES,
            score: 0,
            gems: 0,
            world: 0,
            finished: false,
            progress: WorldProgress::default(),
        }
    }

    pub fn active(&self) -> bool {
        self.lives > 0 && !self.finished
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterDeath {
    /// The same player keeps going, from their checkpoint.
    Continue,
    /// The controller goes to another player.
    SwitchTo(usize),
    /// Nobody can play any more.
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterGate {
    /// The player moves on to the next world and keeps playing.
    NextWorld,
    /// The player cleared every world; the turn passes or the game ends.
    Finished(AfterDeath),
}

#[derive(Debug, Clone)]
pub struct Match {
    pub players: Vec<Player>,
    pub current: usize,
    pub total_worlds: usize,
}

impl Match {
    pub fn new(configs: Vec<PlayerConfig>, total_worlds: usize) -> Self {
        assert!(!configs.is_empty(), "a match needs at least one player");
        assert!(total_worlds > 0);
        Self {
            players: configs.into_iter().map(Player::new).collect(),
            current: 0,
            total_worlds,
        }
    }

    pub fn solo(&self) -> bool {
        self.players.len() == 1
    }

    pub fn player(&self) -> &Player {
        &self.players[self.current]
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current]
    }

    /// The difficulty tier for the current player at `ratio` (0..=1) through the
    /// current world. Progress counts across all worlds, so the last stretch of the
    /// last world is always the hardest.
    pub fn tier(&self, ratio: f32) -> u8 {
        let p = self.player();
        let overall = (p.world as f32 + ratio.clamp(0.0, 1.0)) / self.total_worlds as f32;
        let tier = MIN_TIER + (overall * MAX_TIER as f32).floor() as u8;
        tier.clamp(MIN_TIER, MAX_TIER)
    }

    pub fn gem_collected(&mut self, gem: usize) -> bool {
        let p = self.player_mut();
        if p.progress.collected_gems.insert(gem) {
            p.gems += 1;
            p.score += SCORE_GEM;
            true
        } else {
            false
        }
    }

    pub fn stomped(&mut self) {
        self.player_mut().score += SCORE_STOMP;
    }

    pub fn checkpoint_reached(&mut self, checkpoint: usize) -> bool {
        let p = self.player_mut();
        if p.progress.checkpoint.is_none_or(|c| checkpoint > c) {
            p.progress.checkpoint = Some(checkpoint);
            true
        } else {
            false
        }
    }

    pub fn crystal_solved(&mut self, crystal: usize) {
        let p = self.player_mut();
        if p.progress.used_crystals.insert(crystal) {
            p.score += SCORE_CRYSTAL;
            p.gems += GEMS_PER_CRYSTAL;
        }
    }

    /// The gate question was answered correctly.
    pub fn gate_opened(&mut self) -> AfterGate {
        let solo = self.solo();
        let total = self.total_worlds;
        let p = self.player_mut();
        p.score += SCORE_GATE;
        p.world += 1;
        p.progress = WorldProgress::default();
        // A solo run gets a spare life for every world cleared, as a reward.
        if solo {
            p.lives = p.lives.saturating_add(1);
        }
        if p.world >= total {
            p.finished = true;
            AfterGate::Finished(self.pass_turn())
        } else {
            AfterGate::NextWorld
        }
    }

    /// The current player died: an enemy, a pit, a wrong answer, or the clock.
    pub fn died(&mut self) -> AfterDeath {
        let p = self.player_mut();
        p.lives = p.lives.saturating_sub(1);
        self.pass_turn()
    }

    /// Hands the controller to the next player who can still play, preferring someone
    /// other than the current player.
    fn pass_turn(&mut self) -> AfterDeath {
        let n = self.players.len();
        for step in 1..=n {
            let idx = (self.current + step) % n;
            if self.players[idx].active() {
                if idx == self.current {
                    return AfterDeath::Continue;
                }
                self.current = idx;
                return AfterDeath::SwitchTo(idx);
            }
        }
        AfterDeath::GameOver
    }

    pub fn over(&self) -> bool {
        self.players.iter().all(|p| !p.active())
    }

    /// Players ordered by worlds cleared, then score. Ties keep player order.
    pub fn ranking(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.players.len()).collect();
        idx.sort_by(|&a, &b| {
            let (pa, pb) = (&self.players[a], &self.players[b]);
            pb.world.cmp(&pa.world).then(pb.score.cmp(&pa.score))
        });
        idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(label: &str, age: AgeLevel) -> PlayerConfig {
        PlayerConfig {
            label: label.into(),
            age,
            character: Character::Pip,
        }
    }

    fn duo() -> Match {
        Match::new(vec![cfg("P1", AgeLevel::Twelve), cfg("P2", AgeLevel::Six)], 2)
    }

    #[test]
    fn death_passes_turn_in_two_player_mode() {
        let mut m = duo();
        assert_eq!(m.died(), AfterDeath::SwitchTo(1));
        assert_eq!(m.players[0].lives, START_LIVES - 1);
        assert_eq!(m.current, 1);
        assert_eq!(m.died(), AfterDeath::SwitchTo(0));
    }

    #[test]
    fn solo_death_continues_until_out_of_lives() {
        let mut m = Match::new(vec![cfg("P1", AgeLevel::Six)], 2);
        assert_eq!(m.died(), AfterDeath::Continue);
        assert_eq!(m.died(), AfterDeath::Continue);
        assert_eq!(m.died(), AfterDeath::GameOver);
        assert!(m.over());
    }

    #[test]
    fn out_player_is_skipped() {
        let mut m = duo();
        m.players[1].lives = 0;
        assert_eq!(m.died(), AfterDeath::Continue);
        assert_eq!(m.current, 0);
    }

    #[test]
    fn tier_grows_with_progress() {
        let mut m = duo();
        assert_eq!(m.tier(0.0), 1);
        assert_eq!(m.tier(0.6), 2);
        m.players[0].world = 1;
        assert_eq!(m.tier(0.1), 3);
        assert_eq!(m.tier(1.0), 4);
    }

    #[test]
    fn crystal_scores_once() {
        let mut m = duo();
        m.crystal_solved(3);
        m.crystal_solved(3);
        assert_eq!(m.player().score, SCORE_CRYSTAL);
        assert_eq!(m.player().gems, GEMS_PER_CRYSTAL);
    }

    #[test]
    fn progress_survives_death_but_resets_per_world() {
        let mut m = Match::new(vec![cfg("P1", AgeLevel::Six)], 2);
        m.crystal_solved(1);
        assert!(m.checkpoint_reached(0));
        m.died();
        assert!(m.player().progress.used_crystals.contains(&1));
        assert_eq!(m.player().progress.checkpoint, Some(0));
        assert_eq!(m.gate_opened(), AfterGate::NextWorld);
        assert!(m.player().progress.used_crystals.is_empty());
        assert_eq!(m.player().progress.checkpoint, None);
    }

    #[test]
    fn solo_gets_extra_life_per_world() {
        let mut m = Match::new(vec![cfg("P1", AgeLevel::Six)], 2);
        m.gate_opened();
        assert_eq!(m.player().lives, START_LIVES + 1);
    }

    #[test]
    fn finishing_all_worlds_hands_over_and_ends() {
        let mut m = duo();
        m.gate_opened();
        assert_eq!(m.gate_opened(), AfterGate::Finished(AfterDeath::SwitchTo(1)));
        assert!(m.players[0].finished);
        m.players[1].lives = 1;
        assert_eq!(m.died(), AfterDeath::GameOver);
        assert!(m.over());
        assert_eq!(m.ranking()[0], 0);
    }

    #[test]
    fn gems_count_once() {
        let mut m = duo();
        assert!(m.gem_collected(4));
        assert!(!m.gem_collected(4));
        assert_eq!(m.player().gems, 1);
    }
}
