# numhop — adaptive practice (option A)

## Objective
Make every match practise what each player gets wrong. Weak topics come up more
often, missed questions return a little later, and each topic's difficulty follows
the player's answers. No new screens.

## Why
Today `math::generate` picks a topic from the player's age group completely at
random, and the difficulty tier depends only on how far into the match the player
is. A child who keeps failing borrowing sees it no more often than anything else.

## Scope
- `src/domain/practice.rs` (pure, tested):
  - `Skill` per topic: `tier` (1–4), `misses` (recent weight), `streak` (right in a row).
    A right answer lowers `misses` by 1, and every 3 in a row raise the tier. A wrong
    answer or time-up adds 2 to `misses` (capped at 6), resets the streak, and lowers
    the tier.
  - `Practice` per player: skills by topic. A topic's weight is `1 + misses`, so every
    topic keeps a chance. A topic seen for the first time starts at the match's
    progress tier.
  - `RetryQueue`: a missed question comes back after 2 other questions, with its
    options reshuffled, once.
- `Topic::id`/`from_id`: stable names for the file.
- `Records` keeps each player's `Practice` beside the scores in `records.toml`,
  as `practice.<topic> = "tier misses streak"` lines; damaged lines are skipped.
  `absorb` keeps this run's skill for a topic and takes the other copy's otherwise.
- Game: the question `open` takes a due retry first, otherwise picks a weighted topic
  at that topic's tier. `finish` records the outcome once, queues misses, and marks
  the records unsaved (saved at the next banner). Retries live for the session and
  are cleared on the menu.

## Constraints
- Do not run `cargo fmt`. TDD off; runner `bin/test`; `cargo clippy --all-targets`.
- Topics stay within the player's age group.
- Old records files without practice lines load unchanged.

## Tasks
- [x] T1 Domain: Topic ids, Skill, Practice, weighted pick, RetryQueue, tests
- [x] T2 Records: practice per player, parse/render/absorb, tests
- [x] T3 Game: open uses retries, weights and skill tiers; finish records outcomes and queues misses; clear on menu
- [x] T4 Verify: tests, clippy, scripted tour, README note

## Acceptance criteria
- A topic missed twice is picked clearly more often than an untouched one, but the
  others still come up.
- Three right answers in a row on a topic raise its tier; a miss lowers it (1–4).
- A missed question is asked again two questions later, with options in a new order.
- Practice survives a restart and an old records file still loads.

## Progress
- 2026-11-20: started on branch feat/adaptive-practice. User chose option A.
- 2026-11-20: T1–T4 done. domain/practice.rs (Skill, Practice, pick_topic weighted,
  RetryQueue, next_question, record_answer) with Topic ids; Records holds practice per
  key (practice.<topic> lines, absorb keeps this run's topics); question.rs open uses
  next_question, finish records once for the answering seat and marks records unsaved
  (saved at the next banner); Retries per seat, cleared on the menu. README note.
- Evidence: bin/test 118 passed (13 new: tiers, caps, weighted pick ~5:1:1, retry
  timing and reshuffle, retry missed only once, skill tiers, file round trip, damaged
  lines, absorb); clippy clean; both screenshot tours reach the results, and the real
  records file was not touched.
- The full tour's fixed number of falls broke when `1` happened to be the right answer
  (a different turn order), so it now uses PlayToResults like the README scenario.
- Known limit: practice changed after the last banner is lost if the game is closed
  mid-world (saved at the next banner, like the scores).
