//! Plays the synthesized sound effects from [`crate::domain::sound`].
//!
//! Any system asks for an effect with `commands.write_message(PlaySfx(Sfx::Jump))`;
//! one system turns each request into a short-lived audio entity, unless sound is off.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

pub use crate::domain::sound::Sfx;

/// A request to play one sound effect.
#[derive(Message, Debug, Clone, Copy)]
pub struct PlaySfx(pub Sfx);

/// Sound effects are off. Changed from the Settings page and saved.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Muted(pub bool);

/// One ready-to-play audio asset per effect, rendered once at startup.
#[derive(Resource)]
struct SoundBank(HashMap<Sfx, Handle<AudioSource>>);

pub fn plugin(app: &mut App) {
    app.add_message::<PlaySfx>()
        .add_systems(Startup, build_bank)
        .add_systems(PostUpdate, play);
}

fn build_bank(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>) {
    let bank = Sfx::ALL
        .into_iter()
        .map(|sfx| (sfx, sources.add(AudioSource { bytes: sfx.wav().into() })))
        .collect();
    commands.insert_resource(SoundBank(bank));
}

fn play(mut requests: MessageReader<PlaySfx>, bank: Option<Res<SoundBank>>, muted: Res<Muted>, mut commands: Commands) {
    // Each effect plays at most once a frame, so two gems taken together do not double up.
    let mut played = HashSet::new();
    for PlaySfx(sfx) in requests.read() {
        let Some(bank) = bank.as_ref() else { continue };
        if muted.0 || !played.insert(*sfx) {
            continue;
        }
        commands.spawn((AudioPlayer::new(bank.0[sfx].clone()), PlaybackSettings::DESPAWN));
    }
}
