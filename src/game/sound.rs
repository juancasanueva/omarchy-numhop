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

#[cfg(test)]
mod tests {
    use super::*;

    fn sound_app(muted: bool) -> App {
        let mut app = App::new();
        // Stand-in handles: the test counts the audio entities, nothing is decoded.
        let bank = Sfx::ALL.into_iter().map(|sfx| (sfx, Handle::default())).collect();
        app.add_message::<PlaySfx>()
            .insert_resource(SoundBank(bank))
            .insert_resource(Muted(muted))
            .add_systems(Update, play);
        app
    }

    fn players(app: &mut App) -> usize {
        app.world_mut().query::<&AudioPlayer>().iter(app.world()).count()
    }

    #[test]
    fn a_request_plays_its_sound() {
        let mut app = sound_app(false);
        app.world_mut().write_message(PlaySfx(Sfx::Jump));
        app.update();
        assert_eq!(players(&mut app), 1);
    }

    #[test]
    fn nothing_plays_while_muted() {
        let mut app = sound_app(true);
        for sfx in Sfx::ALL {
            app.world_mut().write_message(PlaySfx(sfx));
        }
        app.update();
        assert_eq!(players(&mut app), 0);
    }

    #[test]
    fn requests_made_while_muted_are_dropped() {
        let mut app = sound_app(true);
        app.world_mut().write_message(PlaySfx(Sfx::Jump));
        // Run only `Update`: a full `app.update()` clears old messages in `First`, which
        // would hide a `play` that skipped reading them while muted.
        app.world_mut().run_schedule(Update);
        app.world_mut().resource_mut::<Muted>().0 = false;
        app.world_mut().run_schedule(Update);
        assert_eq!(players(&mut app), 0, "the muted jump must not play once sound is back on");
    }

    #[test]
    fn the_same_sound_plays_once_a_frame() {
        let mut app = sound_app(false);
        app.world_mut().write_message(PlaySfx(Sfx::Gem));
        app.world_mut().write_message(PlaySfx(Sfx::Gem));
        app.world_mut().write_message(PlaySfx(Sfx::Stomp));
        app.update();
        assert_eq!(players(&mut app), 2);
    }
}
