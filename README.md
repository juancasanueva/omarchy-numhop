# numhop

![numhop: Pip runs through Sunny Meadow under a row of crystal blocks](docs/screenshot.png)

A 3D math platformer for [Omarchy](https://omarchy.org), written in Rust with
[Bevy](https://bevyengine.org). Two siblings take turns running, jumping, and
solving math at their own level. A right answer keeps the controller; a wrong one
costs a life and passes the turn.

The game design (turn-based sibling play, age-tiered topics, a 30-second clock)
is inspired by [pgarevalo/mariomath](https://github.com/pgarevalo/mariomath).
Everything here is original: the code, the characters (Pip the fox, Bolt the
robot, and the Grumbles), and the 3D models, which are built procedurally in
Blender from `assets-src/build_models.py`.

## How it plays

- **One or two players.** Each starts with three lives. Solo players earn an extra
  life for every world cleared.
- **Crystal blocks**: jump into one from below for a question. Right: +250 points
  and 3 gems.
- **The key** before each gate asks the final question of the world. Right:
  +500 and on to the next world.
- **Lose a life** to a Grumble, a gap, a wrong answer, or the clock. In a
  two-player game the turn passes. Stomp Grumbles for +100.
- **Difficulty** grows from ★ to ★★★★ as the player progresses through the worlds.

| Level  | Topics |
|--------|--------|
| Age 6  | Addition with carrying, subtraction without borrowing, in school column layout (tens in red, units in blue) |
| Age 12 | Greatest common divisor, least common multiple, fractions (+ − × ÷) simplified to lowest terms |

After every answer, the game shows the worked solution.

## Shortcuts

| Key | Action |
|-----|--------|
| Left / Right, A / D | Move |
| Space, Up, W | Jump (hold for higher) |
| 1 2 3 4 | Pick an answer |
| Arrow keys + Enter | Choose an answer |
| Enter / Space | Continue |
| ? or F1 | Show all shortcuts |
| Q | Quit (asks first during a match) |

The game follows the Omarchy theme accent live, and its panels use your
Hyprland window rounding.

## Build and install

```bash
bin/test         # unit tests: math generators, match rules, level maps, theme parsing
bin/build        # optimized binary in build/numhop
bin/install      # builds the Arch package and installs it into the app launcher
bin/screenshots  # plays a scripted session and saves a screenshot of each screen
```

It needs Rust (`cargo`) and a Vulkan driver, both of which Omarchy ships with.

## Layout

```text
src/domain/   pure rules, no engine: math questions, match rules, level maps, RNG
src/game/     Bevy adapter: level building, physics, questions, HUD, menus, theme
assets/       glTF models (from Blender) and the Noto fonts
assets-src/   the Blender file and the script that builds every model
pkgbuild/     PKGBUILD, desktop entry, and icon
```

Levels are ASCII maps in `src/domain/levels.rs`, so you can design a new world in
a text editor.

## License

MIT. The bundled Noto fonts are under their own license, in
`assets/fonts/LICENSE-noto.txt`.
