# numhop — sound effects

## Objective
Give the game sound: short effects for jumping, gems, stomps, deaths, questions,
answers, checkpoints, cleared worlds, game over and menu moves, with an on/off switch
in Settings that is saved.

## Why
The game is silent. For young players, audio feedback on a right or wrong answer and
on every jump makes the game feel alive and makes outcomes clearer.

## Scope
- `src/domain/sound.rs`: a pure, tested synthesizer. Each effect is a list of tones
  (wave, pitch slide, start, length, volume) rendered to 16-bit mono PCM and wrapped
  as WAV bytes. No sound files, no third-party audio, nothing to license.
- `src/game/sound.rs`: builds one `AudioSource` per effect at startup; systems send an
  `Sfx` message and one system plays it, unless sound is off.
- Hooks: jump, gem, stomp, caught by a Grumble, fall, question opens, right, wrong,
  time up, checkpoint, world cleared or champion, game over, menu move and confirm.
- Settings page: a "Sound On/Off" row, saved as `sound = "on"|"off"` in settings.toml.
- `Cargo.toml`: enable Bevy's `wav` feature (only Ogg is on by default).

## Constraints
- Do not run `cargo fmt`. TDD off; runner `bin/test`; `cargo clippy --all-targets`.
- A missing `sound` line means on, so existing settings files keep working.
- Scripted screenshots (`NUMHOP_CAPTURE`) stay silent and still show "On".
- UI copy in English and Spanish.

## Tasks
- [x] T1 Domain synth: `Sfx` recipes, rendering, WAV encoding, tests
- [x] T2 Settings: `muted` field, parse/render, tests
- [x] T3 Game: sound plugin, `wav` feature, hooks across play/question/banner/menu
- [x] T4 Menu: Sound row on Settings page, en/es strings, capture silence
- [x] T5 Verify: tests, clippy, scripted tour, README note (listening by ear is pending: the user)

## Acceptance criteria
- Each listed event plays its own short sound; none clicks or clips.
- Turning sound off in Settings silences everything and survives a restart.
- An old settings file without a `sound` line loads with sound on.

## Progress
- 2026-11-20: started on branch feat/sound-effects.
- 2026-11-20: T1–T5 done. Synth in src/domain/sound.rs (14 effects, square/triangle/
  noise, 22.05 kHz mono WAV in memory); game/sound.rs plays `PlaySfx` messages in
  PostUpdate, once per effect per frame, skipped when `Muted`. Settings row Sound
  (en On/Off, es Sí/No), saved as `sound = "on"|"off"`. Capture mode keeps sound
  shown on but sets GlobalVolume to silent.
- Evidence: bin/test 68 passed; cargo clippy --all-targets clean; bin/screenshots
  tour exit 0 with all 20 shots and no audio errors (Bevy decodes each source before
  applying volume, so a bad WAV would have panicked); 02c-settings-switched.png shows
  the Sound row.
- Merged and pushed to main (a3a5f8f). Review approved and acknowledged; advisory
  follow-ups only. The reliability lens flagged (WARNING) that no test proved the mute
  gate, and that the Sound row toggle was untested.
- [x] T6 Tests for the review follow-ups: `play` skips everything while muted and drops
  those requests, plays one of each effect per frame; the Sound row switches with
  Left/Right/Enter and the Language row leaves it alone. bin/test 73 passed; clippy
  clean; removing the mute check makes `nothing_plays_while_muted` fail.
- T6 review approved and acknowledged, with two advisories, both fixed: the "dropped while
  muted" check could not fail (a full app.update clears old messages first), so it is
  now its own test that runs only Update; the menu test is renamed to
  `the_language_row_leaves_the_sound_alone`. bin/test 74 passed; clippy clean; making
  `play` return early while muted without reading makes the new test fail.
- Pending: the user listens in a real game and tunes any sound that feels off.
