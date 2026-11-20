# numhop — Age 8 and Age 10 levels (Spanish Primaria 2º–5º)

## Objective
Add two age levels between Age 6 and Age 12, with topics from the Spanish primary
curriculum (LOMLOE), so the menu cycles 6 → 8 → 10 → 12.

## Why
The jump from Age 6 (carrying, no-borrow subtraction) to Age 12 (MCD, mcm,
fraction operations) skips most of Primaria.

## Scope
- Age 8 (2º–3º): subtraction with borrowing (2–3 digits, column layout),
  times tables (tiers unlock 2·5·10 → 2–10 → 6–9), multiplying by one digit
  (column layout with carries).
- Age 10 (4º–5º): exact division by one digit, adding and subtracting decimals
  (tenths, then hundredths), a fraction of a number ("3/4 of 20").
- Spanish school notation: C/D/U and UM column headers, `:` for division,
  decimal comma, "me llevo una" by adding the carried 1 to the next subtrahend.
- Worked explanations for every new topic in English and Spanish.

## Constraints
- Four distinct options per question, `validate` must hold for every topic and tier.
- Age 6 and Age 12 behavior is unchanged.
- TDD off (same as earlier features); runner `bin/test`.

## Tasks
- [x] T1 Domain: AgeLevel Eight/Ten with next/previous, Topic::ALL, ColumnOp in
      Vertical prompts, Decimal answers, six generators, validate, tests
- [x] T2 i18n: age and topic names, notation strings, localized prompts and
      answers (decimal comma, `:`), explanations for each new topic, tests
- [x] T3 Game: menu cycles four ages both ways, column renderer for any number of
      columns and ×, one-line and fraction-of prompts, localized option text
- [x] T4 Docs and verify: README level table, math module doc, clippy, tests,
      screenshots

## Acceptance criteria
- Menu offers Age 6, 8, 10, 12 for each player; questions match the chosen age.
- Every new topic × tier generates valid questions with four distinct options.
- Explanations contain the answer and no leftover placeholders in both languages.
- Spanish shows decimal commas, `:` for division and C/D/U headers.

## Progress
- 2026-11-20: user chose two levels (Age 8 and Age 10) over one per school year.
  Branch feat/primaria-levels.

- T1: AgeLevel Eight/Ten with ALL/next/previous; Topic::ALL (11 topics); ColumnOp in
  Vertical; Decimal (exact hundredths) answers; subtraction_steps/multiplication_steps;
  six generators with real-mistake distractors (no borrow, no carry, misaligned points).
- T2: 29 new strings per language; prompt_line/value/decimal localize `:` and the decimal
  comma; column-by-column explanations. Spanish menu labels shortened to fit the card
  ("8 años: restas y tablas", "10 años: divisiones y decimales").
- T3: menu Left = previous age, Right/Enter = next; vertical() renders 1–4 columns with
  U/T/H/Th (U/D/C/UM) headers and place colors (hundreds green, thousands purple).
- T4: README level table and Spanish notation note; module doc.

## Verification
- bin/test: 50 passed (14 new). cargo clippy --all-targets: clean.
- Sample run printed questions and explanations for every new topic, tiers 1 and 4, in
  both languages; wording and distractors checked by eye.
- Screenshots (scripted tour, Spanish): menu with Age 8/Age 10 fits; 2- and 3-column
  multiplication with C/D/U headers; fraction-of with stacked fraction (forced via a
  temporary hack, removed).
- Not seen on screen: decimals, division, times tables (plain one-line prompts, same
  path as GCD/LCM) and a 4-column (UM) layout.
- Pre-existing, unrelated: the scripted tour's 05-question shot shows the level
  instead of the question on main too; the tour falls out of step after it.
- rustfmt is not used in this repo (no config reproduces main); do not run cargo fmt.

- Native review (4 lenses, lineage review-6ae28d97045e57b6): approved and acknowledged.
  Advisory, non-blocking: hundredths operands can print trailing zeros (2.00); the
  borrow explanation is picked by topic rather than by whether a column borrows;
  decimals duplicates the option picker; Decimal prompt keeps `plus: bool`; carry
  rule recomputed in the explanation; menu direction not covered by a test.

- Follow-up (user asked): operands never print trailing zeros (no 3.0 or 2.50; the
  zero-answer nudge skips whole numbers), pinned by the decimals test; the subtraction
  explanation now walks the columns whenever any column borrows, whatever the topic
  (new test). bin/test 51 passed, clippy clean.

- Re-review of the full candidate (lineage review-bf1a71084e326449): approved and
  acknowledged. Remaining advisories for later: decimals option picker, Decimal
  `plus: bool`, carry rule in the explanation, subtraction_steps computed twice,
  Vertical doc says hundreds (4 columns possible), menu direction untested,
  exact-string tests for decimal and fraction-of explanations.

## Next step
Done; merged to main.
