# numhop — third hero: Fern the frog

## Objective
Add a third playable hero, Fern the frog, modelled, rigged and animated in Blender
from `assets-src/build_models.py`, so each player can pick Pip, Bolt or Fern.

## Why
The user asked for a third character to choose from.

## Scope
- assets-src/build_models.py: `build_fern`, `RIG_FERN`, Fern in `rig_character`,
  `RIGGED` and `BUILDERS` (reproducible, same pipeline as Pip and Bolt).
- assets/models/hero_fern.glb exported with `idle`, `walk`, `jump`, `death`.
- src/domain/rules.rs: `Character::Fern`, name, three-way cycle with `previous`.
- src/game: load the model and clips, `Rig::Fern`, `hero_model`, menu hero row
  steps back with left.
- README: mention Fern.

## Constraints
- Same coordinate contract (Z-up in Blender, +X facing, origin at feet).
- Blender MCP safe mode: no exec/open/os/classes; paste the whole script.
- Only re-export hero_fern.glb; other .glb files stay byte-identical.
- TDD off; runner bin/test. Never run cargo fmt.

## Tasks
- [x] T1 Fern model + rig + clips in build_models.py; build in Blender; check look and clips
- [x] T2 Export hero_fern.glb; verify clips and skin
- [x] T3 Rust: Character::Fern, cycle both ways, assets, rig, clips, menu
- [x] T4 Verify: tests, clippy, run the game; README

## Acceptance criteria
- The menu Hero row cycles Pip → Bolt → Fern (right) and backwards (left).
- Fern idles on the menu, walks, jumps and plays the death clip in game.
- Loading still waits for every model and clip. Tests and clippy pass.

## Progress
- Started 2026-11-21 on branch feat/third-hero-fern.
- T1: build_fern (round body, wide head with a smile, bulging eyes, sprout on top,
  webbed three-toed feet), RIG_FERN, biped clips with a bouncier walk
  (legs 0.5, arms 0.5, bob 0.045), back 0.21. Checked in Blender next to Pip.
- T2: hero_fern.glb exported alone (MCP safe mode: helpers pasted, no exec);
  tracks walk/idle/jump/death, 0 unweighted verts, death ends lying at z=0.
  Other .glb files unchanged. assets-src/numhop.blend not re-saved (it would
  only hold Fern and Pip from this partial build); the script stays the source.
- T3: Character::Fern with next/previous; menu Hero row: left steps back. Rig,
  clips, assets (15 models), hero_model wired. Tests for both cycle directions.
- T4: bin/test all pass (139 lib tests), clippy clean. Scripted tour run with
  seat 1 temporarily on Fern: menu idle, HUD name, jump pose in game all show.

## Next step
User play-tests Fern; tune walk bounce or colours if wanted.
