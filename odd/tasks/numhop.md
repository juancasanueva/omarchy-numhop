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
- [x] T1 Scaffold Cargo project, bin scripts, Bevy 0.20 window opens
- [x] T2 Domain: math generators + tests
- [x] T3 Domain: game rules (turns, lives, tiers, scoring) + tests
- [x] T4 Blender models: heroes, enemy, blocks, ground, gem, key, gate, flag, decor -> .glb
- [x] T5 Bevy: level building, player movement/physics, camera, enemies, pits
- [x] T6 Bevy: question overlay (timer, options, keyboard), HUD, menus, turns, results
- [x] T7 Omarchy integration: theme accent live, PKGBUILD, .desktop, icon, README
- [x] T8 Verify: cargo test, build, run screenshot, makepkg build

## Acceptance criteria
- `bin/test` passes; `bin/build` produces build/numhop; game playable end to end
  with keyboard for 1 and 2 players; models load; package builds with makepkg.

## Progress
- Started 2026-11-20.
- T1–T6 done. 27 unit tests pass. Scripted capture (NUMHOP_CAPTURE) verified menu,
  banner, running, question, feedback, turn handover, and help screens visually.
- Bevy 0.20 API notes: WorldAssetRoot (not SceneRoot), conditions use `.and_then`,
  buttons are `ui_widgets::Button` + `Activate` observer + `Hovered`, FontSize::Px,
  Node.border_radius, MessageWriter<AppExit>.
- Default Bevy font lacks − × ÷ ★ ♥: bundled Noto Sans Bold + Noto Sans Symbols2.

- T7/T8 done: PKGBUILD (`makepkg -f --nodeps`) built numhop-0.1.0-1 with check()
  passing; package contents verified (binary, 14 models, fonts, desktop, icon,
  licenses). desktop-file-validate clean. Extended capture verified gate question,
  world 2 (Dusk Hills), game over, and results. Clippy clean, 27 tests pass.
- Not done: `makepkg -i` install (needs the user's sudo); not played by a human yet.

## Next step
User runs `bin/install` and play-tests; tune jump/difficulty from feedback.
