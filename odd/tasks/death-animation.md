# numhop — hero death animation

## Objective
When a Grumble catches a hero, play a death clip made in Blender, and only after it
ends show the death banner and respawn.

## Why
The hero currently respawns the instant a Grumble touches them, which feels abrupt
and hides what happened.

## Scope
- assets-src/build_models.py: a `death` clip for Pip and Bolt (startled hop, tip over
  backwards, bounce, lie still), stored as its own NLA track.
- assets/models/hero_pip.glb and hero_bolt.glb re-exported with the clip; other
  models untouched.
- Game: on a Grumble hit the level freezes (no input, Grumbles stop), the clip plays
  once and holds its last pose, then the usual death banner and respawn follow.
- Falls into gaps, wrong answers and the clock keep their current flow.

## Constraints
- Same coordinate contract as the other clips (Z-up in Blender, +X facing, feet at
  origin); export with NLA_TRACKS like walk/idle/jump.
- Blender MCP safe mode: paste the whole script; no exec/open/os.
- Do not run `cargo fmt`. TDD off; runner `bin/test`.

## Tasks
- [x] T1 Blender: author the `death` clip in build_models.py, rebuild in Blender,
      export the two hero .glb files, check the pose by rendering a few frames
- [x] T2 Game: Motion::Death plays once and holds; loading requires the clip for
      heroes; a dying phase freezes play and hands over to the death banner afterwards
- [x] T3 Verify: tests, clippy, screenshot mid-death, README note

## Acceptance criteria
- A Grumble hit shows the hero's death clip for about a second before the banner.
- During the clip the hero, the Grumbles and the camera stay put; no input acts.
- The respawn happens only after the banner, as before.

## Progress
- 2026-11-20: started on branch feat/death-animation. User asked for the animation to
  be made in Blender.

- T1: build_models.py biped_clips gains `death` (startle f3, hop and tip f8, flat on
  the back f14, bounce f18, still f22-30; `back` lift 0.2 Pip / 0.22 Bolt). Rebuilt
  in Blender 5.2 through MCP (only the two heroes; safe mode forbids calling builder
  functions from a list, so they were called directly) and exported both heroes with
  tracks walk, idle, jump, death. Frames 0-30 rendered and checked; Pip's tail tip
  dips ~9 cm below the floor while lying, hidden by the grass.
- T2: Motion::Death plays once (no repeat) and holds; heroes require jump and death
  clips. play.rs: a Grumble hit sets Player.knocked_out and inserts Dying; play
  systems pause while Dying exists; `dying` freezes Grumbles and calls after_death
  after 1.6 s; Dying is cleared on leaving Running.
- T3: tour step TouchGrumble and shot 12a-knocked-out; README note.

## Verification
- bin/test: 62 passed (new: death clip node and idle fallback); clippy clean.
- Screenshot 12a-knocked-out shows Bolt tipping over beside a frozen Grumble; the
  tour still reaches the results screen.

- Native review (4 lenses): approved and acknowledged. Advisories: no App-level test of
  the freeze-then-banner timing; DEATH_SECONDS is synced to the clip by hand; heroes
  now require the death clip to load. Checked: every respawn rebuilds a fresh Player,
  so knocked_out never carries over; there is no level clock to race the death.

## Next step
Done; merged to main.
