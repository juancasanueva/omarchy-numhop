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
    /// The valley floor behind the play strip.
    pub floor: [f32; 3],
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
                floor: [0.38, 0.62, 0.30],
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
                floor: [0.42, 0.34, 0.42],
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
                floor: [0.42, 0.34, 0.42],
            },
        )
        .expect("frost map is valid"),
        Level::parse(
            "Golden Dunes",
            DUNES,
            Palette {
                sky: [0.95, 0.78, 0.52],
                fog: [0.98, 0.88, 0.70],
                sun: [1.0, 0.92, 0.75],
                sun_lux: 14_000.0,
                floor: [0.42, 0.34, 0.42],
            },
        )
        .expect("dunes map is valid"),
        Level::parse(
            "Starry Night",
            NIGHT,
            Palette {
                sky: [0.08, 0.10, 0.25],
                fog: [0.18, 0.20, 0.38],
                sun: [0.70, 0.78, 1.0],
                sun_lux: 5_000.0,
                floor: [0.42, 0.34, 0.42],
            },
        )
        .expect("night map is valid"),
        Level::parse(
            "Coral Coast",
            CORAL,
            Palette {
                sky: [0.45, 0.82, 0.88],
                fog: [0.70, 0.92, 0.92],
                sun: [1.00, 0.97, 0.90],
                sun_lux: 13_000.0,
                floor: [0.86, 0.78, 0.58],
            },
        )
        .expect("coral map is valid"),
        Level::parse(
            "Autumn Woods",
            AUTUMN,
            Palette {
                sky: [0.93, 0.66, 0.42],
                fog: [0.96, 0.80, 0.62],
                sun: [1.00, 0.85, 0.65],
                sun_lux: 9_000.0,
                floor: [0.62, 0.40, 0.22],
            },
        )
        .expect("autumn map is valid"),
        Level::parse(
            "Misty Marsh",
            MARSH,
            Palette {
                sky: [0.55, 0.64, 0.58],
                fog: [0.72, 0.78, 0.72],
                sun: [0.92, 0.95, 0.88],
                sun_lux: 6_500.0,
                floor: [0.32, 0.42, 0.30],
            },
        )
        .expect("marsh map is valid"),
        Level::parse(
            "Volcano Valley",
            VOLCANO,
            Palette {
                sky: [0.35, 0.12, 0.10],
                fog: [0.55, 0.22, 0.15],
                sun: [1.00, 0.60, 0.40],
                sun_lux: 8_000.0,
                floor: [0.25, 0.18, 0.18],
            },
        )
        .expect("volcano map is valid"),
        Level::parse(
            "Rainbow Heights",
            RAINBOW,
            Palette {
                sky: [0.80, 0.70, 0.95],
                fog: [0.95, 0.85, 0.95],
                sun: [1.00, 0.95, 1.00],
                sun_lux: 12_000.0,
                floor: [0.70, 0.62, 0.85],
            },
        )
        .expect("rainbow map is valid"),
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
const DUNES: &[&str] = &[
    "",
    "",
    "                     *                                     *                               ***                           **",
    "               **               *              **              *               *               **              *               *",
    "        C           BCB                C                  CBC            C                BCBCB           C             CBBC             BB",
    "                         B   B                       BB                               B  B                           BB                 BBB  K",
    "  P  t    e        b  e    e        e    t  F      e    e          e   e   b        e   e           F   e          e   e   b       e   BBBB      G  t",
    "===============  ==============   =============  =============   =============   ==============  =============   =============   =====================",
    "===============  ==============   =============  =============   =============   ==============  =============   =============   =====================",
];
const NIGHT: &[&str] = &[
    "",
    "",
    "                   *                    *                                       *               *                          **",
    "               *              *             **             *              *             *            **             *             *           **",
    "       C          BCB                  CBC          C             C            BCB             CBC          C             CBBC                      BB",
    "                      B             BB            B            BBB                         B  B                        BB            B             BBB  K",
    "  P   b   e         e   e         t   e  e      F   e  e         e   e        b  e  e        e   e        F   e          e   e         e  e       BBBB      G  t",
    "==============   ============   ============  ============   ============   ===========   ===========  ============   ===========   ==========  ================",
    "==============   ============   ============  ============   ============   ===========   ===========  ============   ===========   ==========  ================",
];

const CORAL: &[&str] = &[
    "",
    "",
    "                       ***                        *                     **                     *         ***                            **",
    "                                *      **                *                                *",
    "             C         BCB                        C                    CBBC                    C         BCB                   C       CBBC             C        C         BB",
    "                                      B  B                                         BB                                                                                     BBB  K",
    "  P t     e  e     e     e  e       e  e          e  e         F  e   e  e  e     BBB          e      e                 F   e       e  e      t        e    t    e   b   BBBB      G  t",
    "===============================  =======================   ==============================  =====================   ==============================   ===================================",
    "===============================  =======================   ==============================  =====================   ==============================   ===================================",
];
const AUTUMN: &[&str] = &[
    "",
    "",
    "              *       *                       **                                                  *                                    *                          **         *",
    "                            *                                                    **                                 *",
    "             BCB      C                      CBBC                       C                        BCB       C                         BCBCB         C             CBBC        C           BB",
    "                                  BB                                            B  B                                                                                                    BBB  K",
    "  P t  b  e     e     e          BBB   t   e   e  e   t       F   t   e     t  e  e            e   e   t    e  e        e  F  e  e      e    t   e     b         e      e   e  e  t    BBBB      G  t",
    "===========================   ==========================  ================================   ======================   ===================================   =========================================",
    "===========================   ==========================  ================================   ======================   ===================================   =========================================",
];
const MARSH: &[&str] = &[
    "",
    "",
    "             *       **                        ***                                              *                                    **                      *       ***",
    "                                      **                                                 *                          *                                                          *",
    "             C      CBBC                       BCB                              C             BCBCB        C                        CBBC                     C       BCB             C        BB",
    "                                     B  B                               BB                                                                     BB                                            BBB  K",
    "  P t   t     e  e      e          e   e  e   e  e   e        e  F     BBB      e    e      e  e      b   e    b          F   e     e    e    BBB  e       e  e  b      e          e    e   BBBB      G  t",
    "=============================   =========================   ============================  =========================   ================================   =====================   =========================",
    "=============================   =========================   ============================  =========================   ================================   =====================   =========================",
];
const VOLCANO: &[&str] = &[
    "",
    "",
    "            ***                               *                                              **                               *        *             *          *            **",
    "                                                       *             **                                         *",
    "            BCB                      C      BCBCB                             C             CBBC                             BCB       C           BCBCB        C           CBBC          BB",
    "                       BB                                           B  B                                BB                                                                               BBB  K",
    "  P t   b    e     t  BBB          e  e  e  e    e           F    e   e  e   e  e  t       e    e   t  BBB  e      e  F  e  e  e   b  e  e        e  e  e  t   e           e    e  b    BBBB      G  t",
    "==============================   =====================   ============================   =======================  ===========================   ======================   ==============================",
    "==============================   =====================   ============================   =======================  ===========================   ======================   ==============================",
];
const RAINBOW: &[&str] = &[
    "",
    "",
    "            *          *                         **                      *                       *                                         *                **       *            *",
    "                                *                                                 **                                                                                                             *",
    "            C        BCBCB                      CBBC                     C                      BCB                      C               BCBCB             CBBC      C           BCB      C              BB",
    "                                       BB                                        B  B                      BB                                                                                           BBB  K",
    "  P t       e    b   e  e  e          BBB   t  e    e         e  F   b  e   e  e  e  e        e   e   t   BBB         e   e  e  F    t  e   e    e         e  e   e    e       e    e   e    e         BBBB      G  t",
    "===============================   =======================   =============================   =====================   ================================   ===================   ===================   ==================",
    "===============================   =======================   =============================   =====================   ================================   ===================   ===================   ==================",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worlds_parse() {
        let worlds = worlds();
        assert_eq!(worlds.len(), 10);
        for w in &worlds {
            assert!(w.crystals().len() >= 5, "{} has few crystals", w.name);
            assert!(!w.checkpoints.is_empty());
        }
    }

    #[test]
    fn each_world_has_more_grumbles_and_no_fewer_crystals() {
        let worlds = worlds();
        for pair in worlds.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            assert!(b.enemies.len() > a.enemies.len(), "{} has no more Grumbles than {}", b.name, a.name);
            assert!(b.crystals().len() >= a.crystals().len(), "{} has fewer crystals than {}", b.name, a.name);
        }
    }

    #[test]
    fn grumbles_are_spaced_out_and_clear_of_gap_edges() {
        // A Grumble needs room to patrol, and one right at a gap would leave no safe
        // landing. assets-src/gen_worlds.py follows the same rules.
        for w in worlds() {
            let mut cols: Vec<i32> = w.enemies.iter().map(|e| e.col).collect();
            cols.sort_unstable();
            for pair in cols.windows(2) {
                assert!(pair[1] - pair[0] >= 3, "{}: Grumbles at {} and {} are too close", w.name, pair[0], pair[1]);
            }
            for e in &w.enemies {
                for col in e.col - 2..=e.col + 2 {
                    let ground = Cell { col, row: e.row - 1 };
                    assert!(w.solid_at(ground).is_some(), "{}: the Grumble at {} is next to a gap", w.name, e.col);
                }
            }
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
    fn gaps_are_jumpable() {
        // A jump carries about 5 tiles across; 3-tile gaps leave room for a late takeoff.
        for w in worlds() {
            let mut gap = 0;
            for col in 0..w.width {
                if w.solid_at(Cell { col, row: 0 }).is_some() {
                    assert!(gap <= 3, "{}: a {gap}-tile gap ends at column {col}", w.name);
                    gap = 0;
                } else {
                    gap += 1;
                }
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
