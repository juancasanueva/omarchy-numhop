# numhop — skeletal walk, idle and jump animations

## Objective
Rig Pip, Bolt and Grumble in Blender with a small armature, bind each body part
rigidly to its bone, author `walk`, `idle` and `jump` clips, export them in the same
.glb files, and play them in Bevy driven by movement state.

## Why
Characters currently slide with a procedural bob; real walk cycles read much better
for kids.

## Scope
- assets-src/build_models.py: armature + rigid vertex groups + actions (reproducible).
- assets/models/{hero_pip,hero_bolt,enemy_grumble}.glb re-exported with animations.
- src/game: animation graph per model, state machine idle/walk/jump with blending,
  walk speed scaled to run speed; menu heroes idle; enemies walk.
- Keep landing squash; drop the procedural run bob.

## Constraints
- Same coordinate contract (Z-up in Blender, +X facing, origin at feet).
- Blender MCP safe mode: no exec/open/os/classes.
- TDD off; runner bin/test.

## Tasks
- [x] T1 Rig + clips for Pip, Bolt, Grumble in Blender; export; verify in Blender
- [ ] T2 Bevy: load clips, build graphs, attach to spawned scenes
- [ ] T3 Bevy: animation state machine (idle/walk/jump), speed scaling, menu idle
- [ ] T4 Verify: tests, clippy, capture screenshots, commit

## Acceptance criteria
- Walking hero visibly swings legs/arms in game; idle when still; jump pose in air.
- Grumbles animate while patrolling. No regressions in tests or capture run.

## Progress
- Started 2026-11-20.
- T1: rigs + clips built via build_models.py (bone= tags, rigid deform layer,
  NLA tracks). GLBs carry 1 skin each; Pip/Bolt walk/idle/jump, Grumble walk/idle;
  0 unweighted verts; walk cycle checked in Blender (opposite arm/leg, bob, tail).
