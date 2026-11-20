//! Level layouts, written as ASCII maps so they can be read and tested without the engine.
//!
//! Each character is one tile. The last string is row 0 (the bottom). Legend:
//! `=` ground, `B` stone block, `C` crystal (math) block, `*` gem, `e` enemy,
//! `P` player start, `F` checkpoint, `K` key (gate question), `G` gate,
//! `t` tree and `b` bush (decoration, not solid).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Ground,
    Block,
    Crystal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decor {
    Tree,
    Bush,
}

/// A grid cell: `col` grows to the right, `row` grows upward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub col: i32,
    pub row: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub sky: [f32; 3],
    pub fog: [f32; 3],
    pub sun: [f32; 3],
    pub sun_lux: f32,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub name: &'static str,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<(Cell, Tile)>,
    pub gems: Vec<Cell>,
    pub enemies: Vec<Cell>,
    pub checkpoints: Vec<Cell>,
    pub decor: Vec<(Cell, Decor)>,
    pub start: Cell,
    pub key: Cell,
    pub gate: Cell,
    pub palette: Palette,
}

impl Level {
    pub fn parse(name: &'static str, rows: &[&str], palette: Palette) -> Result<Self, String> {
        let height = rows.len() as i32;
        let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
        let mut level = Level {
            name,
            width,
            height,
            tiles: Vec::new(),
            gems: Vec::new(),
            enemies: Vec::new(),
            checkpoints: Vec::new(),
            decor: Vec::new(),
            start: Cell { col: -1, row: -1 },
            key: Cell { col: -1, row: -1 },
            gate: Cell { col: -1, row: -1 },
            palette,
        };
        let (mut starts, mut keys, mut gates) = (0, 0, 0);
        for (i, line) in rows.iter().enumerate() {
            let row = height - 1 - i as i32;
            for (col, ch) in line.chars().enumerate() {
                let cell = Cell { col: col as i32, row };
                match ch {
                    ' ' => {}
                    '=' => level.tiles.push((cell, Tile::Ground)),
                    'B' => level.tiles.push((cell, Tile::Block)),
                    'C' => level.tiles.push((cell, Tile::Crystal)),
                    '*' => level.gems.push(cell),
                    'e' => level.enemies.push(cell),
                    'F' => level.checkpoints.push(cell),
                    't' => level.decor.push((cell, Decor::Tree)),
                    'b' => level.decor.push((cell, Decor::Bush)),
                    'P' => {
                        level.start = cell;
                        starts += 1;
                    }
                    'K' => {
                        level.key = cell;
                        keys += 1;
                    }
                    'G' => {
                        level.gate = cell;
                        gates += 1;
                    }
                    other => return Err(format!("{name}: unknown tile {other:?} at {col},{row}")),
                }
            }
        }
        if starts != 1 || keys != 1 || gates != 1 {
            return Err(format!("{name}: needs exactly one P, K and G"));
        }
        level.checkpoints.sort_by_key(|c| c.col);
        Ok(level)
    }

    pub fn solid_at(&self, cell: Cell) -> Option<Tile> {
        self.tiles.iter().find(|(c, _)| *c == cell).map(|(_, t)| *t)
    }

    /// Crystal blocks in a stable order; their index is their identity in the rules.
    pub fn crystals(&self) -> Vec<Cell> {
        self.tiles
            .iter()
            .filter(|(_, t)| *t == Tile::Crystal)
            .map(|(c, _)| *c)
            .collect()
    }

    /// The column a player respawns at: the latest checkpoint reached, or the start.
    pub fn respawn(&self, checkpoint: Option<usize>) -> Cell {
        checkpoint
            .and_then(|i| self.checkpoints.get(i).copied())
            .unwrap_or(self.start)
    }
}

pub fn worlds() -> Vec<Level> {
    vec![
        Level::parse(
            "Sunny Meadow",
            MEADOW,
            Palette {
                sky: [0.53, 0.78, 0.96],
                fog: [0.70, 0.86, 0.98],
                sun: [1.0, 0.96, 0.88],
                sun_lux: 12_000.0,
            },
        )
        .expect("meadow map is valid"),
        Level::parse(
            "Dusk Hills",
            DUSK,
            Palette {
                sky: [0.36, 0.25, 0.48],
                fog: [0.80, 0.52, 0.50],
                sun: [1.0, 0.70, 0.50],
                sun_lux: 7_000.0,
            },
        )
        .expect("dusk map is valid"),
        Level::parse(
            "Frost Peaks",
            FROST,
            Palette {
                sky: [0.62, 0.80, 0.93],
                fog: [0.88, 0.93, 0.98],
                sun: [0.90, 0.95, 1.0],
                sun_lux: 10_000.0,
            },
        )
        .expect("frost map is valid"),
    ]
}

const MEADOW: &[&str] = &[
    "",
    "            *                               C",
    "                                           * *                                   *",
    "                                                                                                  B",
    "        C  BCB                 C           BCB   B  B    ***   CBBC      B     B   B   C         BB",
    "                   *** B  B         B                                   BB                      BBB   K",
    "  P t    b      e            t     BB   e      Fb       e   e        t BBB  e             e b  BBBB      G  t",
    "========================  ========================  ============================   ===========================",
    "========================  ========================  ============================   ===========================",
];
const DUSK: &[&str] = &[
    "",
    "                                     *        BCB",
    "                                                         **                *",
    "                                     B                          C                                *",
    "         C              CBC                 CBB         B  B          B  B   B   C   BCB                 C              B",
    "            **   B  B              B   B                             BB                        B   B          BB  BB   BB  K",
    "  P   b       e             e  e         t        e   F       e  e  BBB            e     e  t         e    e          BBB     G",
    "==================  ================   ==================  ===============   ===================   =============  ================",
    "==================  ================   ==================  ===============   ===================   =============  ================",
];
const FROST: &[&str] = &[
    "",
    "                                                           *",
    "                       *                                  ***                                *                               **",
    "                **                 *                 *                                   *               *",
    "         C            BCB                 C              CBC             C                  BCB              C              CBBC    BB",
    "                           B   B              BB                  BB                                        B  B                   BBB  K",
    "  P   t     e       b   e    e          t   e   F           e   e          b   e    e          F  e  t          e   e        b    BBBB    G",
    "================  ================   ===============   ===============  ================   =============   =============  ==================",
    "================  ================   ===============   ===============  ================   =============   =============  ==================",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worlds_parse() {
        let worlds = worlds();
        assert_eq!(worlds.len(), 3);
        for w in &worlds {
            assert!(w.crystals().len() >= 5, "{} has few crystals", w.name);
            assert!(!w.checkpoints.is_empty());
        }
    }

    #[test]
    fn spawn_points_stand_on_ground() {
        for w in worlds() {
            let mut spawns = vec![w.start, w.gate];
            spawns.extend(w.checkpoints.iter().copied());
            spawns.extend(w.enemies.iter().copied());
            for s in spawns {
                let below = Cell { col: s.col, row: s.row - 1 };
                assert!(w.solid_at(below).is_some(), "{}: nothing under {:?}", w.name, s);
                assert!(w.solid_at(s).is_none(), "{}: {:?} is inside a block", w.name, s);
            }
        }
    }

    #[test]
    fn key_comes_before_gate() {
        for w in worlds() {
            assert!(w.key.col < w.gate.col);
        }
    }

    #[test]
    fn respawn_falls_back_to_start() {
        let w = &worlds()[0];
        assert_eq!(w.respawn(None), w.start);
        assert_eq!(w.respawn(Some(0)), w.checkpoints[0]);
        assert_eq!(w.respawn(Some(99)), w.start);
    }

    #[test]
    fn unknown_tile_is_rejected() {
        let palette = worlds()[0].palette;
        assert!(Level::parse("bad", &["P K G x", "======="], palette).is_err());
    }
}
