# numhop — fourth hero: Pebble the penguin

## Objective
Add a fourth playable hero, Pebble the penguin, modelled, rigged and animated in
Blender from `assets-src/build_models.py`, so each player can pick Pip, Bolt, Fern
or Pebble.

## Why
The user asked for a fourth character to choose from.

## Scope
- assets-src/build_models.py: `build_pebble`, `RIG_PEBBLE`, a `roll` option on
  `biped_clips` for a waddling walk (default keeps the other heroes unchanged),
  Pebble in `rig_character`, `RIGGED` and `BUILDERS`.
- assets/models/hero_pebble.glb exported with `idle`, `walk`, `jump`, `death`.
- src/domain/rules.rs: `Character::Pebble`, name, four-way cycle.
- src/game: load the model and clips, `Rig::Pebble`, `hero_model`.
- README: mention Pebble.

## Constraints
- Same coordinate contract (Z-up in Blender, +X facing, origin at feet).
- Blender MCP safe mode: no exec/open/os/classes and no state between calls.
- Only export hero_pebble.glb; other .glb files stay byte-identical.
- TDD off; runner bin/test. Never run cargo fmt.

## Tasks
- [x] T1 Pebble model + rig + clips in build_models.py; build in Blender; check look and clips
- [x] T2 Export hero_pebble.glb; verify clips, skin and death landing
- [x] T3 Rust: Character::Pebble in the cycle, assets, rig, clips; tests
- [x] T4 Verify: tests, clippy, run the game; README

## Acceptance criteria
- The menu Hero row cycles Pip → Bolt → Fern → Pebble (right) and backwards (left).
- Pebble idles on the menu, waddles, jumps and plays the death clip in game.
- Loading still waits for every model and clip. Tests and clippy pass.

## Progress
- Started 2026-11-21 on branch feat/fourth-hero-pebble.
- T1: build_pebble (egg body, white front and face, beak, flippers, red scarf,
  flat feet, tail), RIG_PEBBLE, biped clips with roll=0.14 for a waddle; the
  roll option defaults to 0.04 so Pip, Bolt and Fern are unchanged.
- T2: hero_pebble.glb: walk/idle/jump/death, one 7-joint skin, 0 unweighted
  verts. Death first sank 3.3 cm (the tail reaches x=-0.243), so back is 0.245;
  the death clip now ends at z=0.002. Other .glb files unchanged.
- T3: Character::Pebble in ALL, name, next and previous; Rig, clips, assets
  (16 models), hero_model. Cycle and menu tests updated for four heroes.
- T4: bin/test pass, clippy clean. README paragraph mentions Pebble and is
  reflowed to 80 columns. Scripted tour with seat 1 temporarily on Pebble:
  menu idle, HUD name and walking all show.

## Next step
User play-tests Pebble.
