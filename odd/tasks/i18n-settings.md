# numhop — Spanish translation and language setting

## Objective
Translate every player-visible string to Spanish and add a Settings page in the menu
where the language (English / Español) is selected and remembered.

## Why
The players are Spanish-speaking kids; the original mariomath was Spanish.

## Scope
- Domain carries no English prose: question explanations become structured data,
  labels move to the adapter.
- `src/i18n`: typed catalogs (one struct, EN and ES instances) so a missing string is a
  compile error; formatting helper with numbered placeholders; tests for placeholder
  parity between languages.
- Settings persisted in `~/.config/numhop/settings.toml` (atomic save). First-run
  default: `~/.config/omarchy/language`, then `$LANG`, then English.
- Menu gets a Settings page (Language, Back); language switches live.
- Spanish math notation: MCD / mcm, D/U columns (decenas/unidades).

## Constraints
- Technical artifacts (code, comments) in English; Spanish only in the ES catalog,
  neutral register, kid-friendly.
- TDD: off (no project config). Runner: `bin/test` (`cargo test`).

## Tasks
- [x] T1 Domain: structured explanations, no English labels in domain
- [x] T2 i18n catalogs EN/ES + formatter + tests
- [x] T3 Settings persistence + default language detection + tests
- [x] T4 Wire language through every screen; Settings page in menu
- [x] T5 Verify: tests, clippy, Spanish screenshots, README, commit

## Acceptance criteria
- Every visible string comes from the catalog; Spanish run shows no English (except
  brand names Pip, Bolt, Grumble, numhop and key names).
- Language choice survives restart; switching in Settings updates the menu at once.

## Progress
- Started 2026-11-20.
- T1: Question.explanation removed; i18n derives the worked solution from the prompt
  (domain exposes factor_string and common_denominator). Labels moved to catalogs.
- T2: src/i18n/{mod,en,es}.rs; `catalog!` macro lists templates; tests check
  placeholder parity, no leftover placeholders in explanations, MCD/mcm.
- T3: src/settings.rs, atomic save, default Omarchy language → $LC_ALL/$LANG → en.
  NUMHOP_LANG overrides for one run; capture mode never saves.
- T4: every screen reads `Language`; Settings page in menu (Language, Back); quit
  confirm accepts S (sí) and Y.
- T5: 35 tests pass, clippy clean. Spanish capture checked all 14 screens; live
  switch Español→English redraws the menu at once; no settings file written in
  capture mode.
- Not verified by a human: real save on switch outside capture mode (unit-tested only).

## Next step
User play-tests in Spanish; merge branch when happy.
