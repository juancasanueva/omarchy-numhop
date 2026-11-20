# numhop — 3D math platformer for Omarchy

## Objective
An original 2.5D/3D side-scrolling math platformer in Rust (Bevy 0.20), inspired by the
game design of pgarevalo/mariomath (turn-based sibling competition, age-tiered math),
with original characters and 3D models built procedurally in Blender via MCP, packaged
as an Omarchy app (PKGBUILD, .desktop, icon, theme accent).

## Problem / Why
The reference is a 2D browser game built on Nintendo IP and without a license. We keep
only the game design (mechanics and math curriculum) and rebuild everything original.

## Scope
- Pure domain: math question generators (age 6: addition with carrying, subtraction
  without borrowing; age 12: GCD, LCM, fractions +-×÷ simplified), 4 tiers, 4 options.
- Pure rules: players, lives, score, turns, tier from progress, checkpoints.
- Bevy adapter: menu (1P/2P, age, character), side-scrolling 3D levels (2 worlds),
  AABB physics, enemies (stomp), pits, crystal blocks -> question, key -> gate question,
  30 s timer, HUD, turn overlay, results screen, Omarchy accent followed live.
- Blender models exported as .glb into assets/models (source .blend in assets-src).
- Omarchy packaging: bin/build, bin/test, bin/install, pkgbuild/{PKGBUILD,.desktop,.svg}.

## Constraints
- No Nintendo names, characters, art, or look. Original characters: Pip (fox), Bolt (robot),
  Grumble (enemy).
- Technical artifacts in English.
- TDD mode: off (source: no project/session TDD config). Runner: `cargo test`.

## Tasks
- [ ] T1 Scaffold Cargo project, bin scripts, Bevy 0.20 window opens
- [ ] T2 Domain: math generators + tests
- [ ] T3 Domain: game rules (turns, lives, tiers, scoring) + tests
- [ ] T4 Blender models: heroes, enemy, blocks, ground, gem, key, gate, flag, decor -> .glb
- [ ] T5 Bevy: level building, player movement/physics, camera, enemies, pits
- [ ] T6 Bevy: question overlay (timer, options, keyboard), HUD, menus, turns, results
- [ ] T7 Omarchy integration: theme accent live, PKGBUILD, .desktop, icon, README
- [ ] T8 Verify: cargo test, build, run screenshot, makepkg build

## Acceptance criteria
- `bin/test` passes; `bin/build` produces build/numhop; game playable end to end
  with keyboard for 1 and 2 players; models load; package builds with makepkg.

## Progress
- Started 2026-11-20.

## Next step
T1 + T2 inline; T4 in a parallel fork (disjoint surface: assets/, assets-src/).
