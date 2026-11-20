# numhop — saved progress and high scores

## Objective
Remember each player's best score and the furthest world they reached, across runs,
and let them start a match from any world they have already reached.

## Why
Every match starts at world 1 and nothing survives closing the game. With five
worlds, kids replay the first ones over and over, and a great run leaves no trace.

## Scope
- `src/domain/records.rs`: pure records per player. Key: the cleaned name, lowercased;
  a blank name uses `#1`/`#2`, which no typed name can produce. Each record holds
  `best_score` and `worlds_cleared`. Merging a match result reports what improved.
- Saved in `$XDG_DATA_HOME/numhop/records.toml` (default `~/.local/share`), one TOML
  table per player, written atomically like settings.
- Records merge every time a banner shows (after each gate, death and match start)
  and when the results show, so quitting mid-match keeps the worlds already cleared.
- `PlayerConfig.start_world`: a match starts each player at their chosen world.
- Menu: a "Start world" row per player, cycling through the worlds they have reached;
  it re-clamps when the name changes.
- Results: a "New best!" mark on lines that beat the saved best score.
- Scripted screenshots (`NUMHOP_CAPTURE`) neither read nor write records.

## Constraints
- Do not run `cargo fmt`. TDD off; runner `bin/test`; `cargo clippy --all-targets`.
- A missing or unreadable records file means no records; never crash on it.
- UI copy in English and Spanish.

## Tasks
- [x] T1 Domain records: key, merge, unlocked world, parse/render, tests
- [x] T2 Rules: `start_world` in `PlayerConfig`, `Match::new` honours it, tests
- [x] T3 Storage: records path, atomic save shared with settings, load
- [x] T4 Game: records resource, update on gate and results, "New best!" on results
- [x] T5 Menu: Start world row per player, clamped to unlocked, en/es strings, tests
- [x] T6 Verify: tests, clippy, scripted tour, README note

## Acceptance criteria
- Clearing world 2 then quitting lets that player start at world 3 next time.
- A player who has cleared nothing sees only world 1.
- Beating a saved best score shows "New best!" and the new score is kept.
- Names differing only in case share one record.
- A missing, empty or corrupted records file loads as empty and the game runs.

## Progress
- 2026-11-20: started on branch feat/saved-progress. User chose records plus a world
  picker over records only.
- 2026-11-20: T1–T6 done. domain/records.rs (key, merge, startable_worlds, parse/render);
  PlayerConfig.start_world (Match clamps past the end); settings.rs records_path,
  load/save_records sharing write_atomically; game/records.rs plugin (SavedRecords,
  BestBefore snapshot on the first banner, merge on every Banner/Results, save only on
  improvement, forget on Menu or leaving Results); menu Start world row (clamped to
  the name now in the seat); results "New best!"; README section.
- Evidence: bin/test 88 passed; clippy clean; scripted tour exit 0, 20 shots, no
  records file written; the two-player menu with both Start world rows fits.
- Known, pre-existing: the scripted tour no longer reaches the crystal question
  (shot 06 shows no question, results 0/0). Plain main at 6204002 does the same twice
  in a row, so this change did not cause it. "New best!" was not seen on screen for
  that reason; keep_records is covered by an app test.
- Review review-05f3da51c2184841 approved and acknowledged, with warnings that broke
  this feature's own constraints, so they were fixed as task T7:
- [x] T7 Review fixes: load_records reads bytes and decodes lossily (one bad byte loses
  one line) and returns an error for a file that exists but cannot be read, and that
  run keeps progress in memory only instead of saving over it; a failed save sets
  `unsaved` and is retried at the next banner; startable_worlds uses saturating_add
  (a corrupted huge count no longer panics); "New best!" is the tested pure
  `is_new_best`, with no claim when the snapshot is missing; world-index comment, one
  menu helper returns (world, startable), results block reindented.
  bin/test 93 passed; clippy clean; mutating the retry or the overflow makes their
  tests fail.
- Review review-d2d1370bfd0fb870 approved and acknowledged, with two warnings. The
  user chose to fix them before merging (T8).
- [x] T8 Two windows and the results wiring: before each save, the file on disk is
  re-read and absorbed (better value per player wins), and a file that has become
  unreadable is not written over (retried at the next banner); the temporary file name
  carries the process id. "New best!" lines come from the pure `result_lines`, tested
  for per-seat lookup (not ranking place) and no claim without a snapshot; the
  BestBefore lifecycle (snapshot, kept through Results, renewed on play again, dropped
  on Menu) is tested with real state transitions; load_from(unreadable) keeps no path.
  Also renamed MenuChoices::effective_start_world, noted the start-world invariant,
  and fixed the scope line. bin/test 99 passed; clippy clean; dropping the merge
  before saving, or looking bests up by ranking place, makes their tests fail.
