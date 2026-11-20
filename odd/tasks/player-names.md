# numhop — players type their own names

## Objective
Let each kid type a name in the menu, shown everywhere instead of "Player 1/2", and
remember the names between runs.

## Why
Kids want to see their own name on the banner, HUD and results.

## Scope
- Menu: a name row per player (Enter to type, Backspace, Enter saves, Esc cancels);
  age and hero rows read "Lucía's level" once a name is set.
- While typing, menu keys (WASD, Space, Q, ?) type letters instead of acting.
- Empty name falls back to "Player N" / "Jugador N".
- Names saved in `~/.config/numhop/settings.toml` (`player1`, `player2`), not when
  `NUMHOP_CAPTURE` or `NUMHOP_LANG` is set.

## Constraints
- Names: letters (accents, ñ), digits, space, hyphen, apostrophe; at most 12 characters.
  No quotes or backslashes, so the settings file stays valid.
- Do not run `cargo fmt` (the repo is not rustfmt-formatted).
- TDD off; runner `bin/test`.

## Tasks
- [x] T1 Domain + settings: name cleaning rules; Settings keeps names; parse/render/save
      round-trip; tests
- [x] T2 Game: PlayerNames resource loaded at start and saved on change; menu name rows,
      typing mode, overlay ignores keys while typing; labels use names
- [x] T3 i18n, docs and verify: strings in both languages, README, tests, clippy,
      screenshot of the menu with a name being typed

## Acceptance criteria
- Typing "Lucía" for Player 1 shows "Lucía" in the menu rows, banner, HUD and results.
- Q, Space, W/A/S/D and ? type letters while editing, and work as before otherwise.
- Names survive a restart; an empty name shows "Player 1".

## Progress
- 2026-11-20: started on branch feat/player-names.

- T1: rules::clean_name / name_char_allowed / MAX_NAME_CHARS; Settings.names saved as
  player1/player2, cleaned on read and write.
- T2: PlayerNames resource; save_settings runs when the language or names change (not
  under NUMHOP_CAPTURE or NUMHOP_LANG); menu Name rows with typing mode (type_name);
  overlay ignores Q and ? while typing; age and hero rows and match labels use names.
- T3: name_placeholder and name_hint strings; English "{0}'s level" and "{0}'s hero";
  README; the tour gained a Type step and types "Lucía" (shot 02a-typing-name).

## Verification
- bin/test: 58 passed (7 new: name cleaning, names cleaned in settings, typing keys,
  backspace and odd characters, length limit, Enter and Esc, fallback name; the
  settings round-trip test now carries names).
- cargo clippy --all-targets: clean.
- Screenshots: the menu while typing "Lucía_" with the Spanish hint; the banner and HUD
  show "Lucía · Pip". The real ~/.config/numhop/settings.toml was not touched by the
  tour.

- Native review (4 lenses, lineage review-8be9edd78fd6ffae): approved and acknowledged.
  Advisories: the Esc test starts from an empty name, so it cannot catch a wiped name;
  the "Íñigo O'Neil-2" case mixes the character rules with truncation; names are
  dropped when the language line is invalid; the test count above is 7 new, not 6;
  overlay/menu system order (checked: harmless, the overlay ignores Esc and Enter
  when closed).

- Follow-up (user asked): the Esc test starts from a saved name ("Mateo") and checks
  it survives; the name test shows hyphens, digits and apostrophes kept on a short
  name and truncation on its own.

- Re-review (lineage review-233789fffc1cd1b5) found a real CRITICAL bug: passing
  `&mut ResMut<T>` as `&mut T` marks the resource changed every frame, so while typing
  the menu redrew and settings.toml was rewritten every frame. Fixed with
  bypass_change_detection() and set_changed() only on real changes; new in-engine test
  idle_typing_frames_change_nothing fails without the fix. bin/test 59 passed.
- The review's targeted validation of the fix was killed twice before returning a
  result; the user chose to ship with that validation unfinished.

## Next step
Done; merged to main.
