"""Generates numhop world maps 6-10 from pieces the first five worlds already use.

Usage: python3 assets-src/gen_worlds.py src/domain/levels.rs
The maps are seeded, so the output is the same on every run. Paste the printed
`const` blocks over CORAL..RAINBOW in src/domain/levels.rs; stats go to stderr.

Rows are indexed by height: 0 and 1 are ground, 2 is where heroes and Grumbles
stand, 4 is the crystal / BCB platform height, 5-7 hold gems and low extras.
Prints Rust `const` blocks; checks the same rules as the level tests.
"""
import random
import re
import sys

HEIGHT = 9  # rows 0..8; row 8 stays empty like the existing maps


def blank(w):
    return {r: [" "] * w for r in range(2, HEIGHT)}


def chunk(w, ground=True, items=()):
    rows = blank(w)
    for col, row, ch in items:
        rows[row][col] = ch
    return {"w": w, "ground": ground, "rows": rows}


def run(rng, w, enemies, decor=True):
    items = []
    slots = list(range(2, w - 2))
    rng.shuffle(slots)
    used = set()
    for c in slots:
        if enemies == 0:
            break
        if any(abs(c - u) < 2 for u in used):
            continue
        items.append((c, 2, "e"))
        used.add(c)
        enemies -= 1
    if decor and rng.random() < 0.5:
        free = [c for c in range(1, w - 1) if all(abs(c - u) > 0 for u in used)]
        if free:
            items.append((rng.choice(free), 2, rng.choice("tb")))
    return chunk(w, items=items)


def crystal(rng):
    items = [(2, 4, "C")]
    if rng.random() < 0.5:
        items.append((2, 6, "*"))
    return chunk(5, items=items)


def bcb(rng):
    items = [(2, 4, "B"), (3, 4, "C"), (4, 4, "B")]
    if rng.random() < 0.5:
        items += [(2, 6, "*"), (3, 6, "*"), (4, 6, "*")]
    else:
        items += [(3, 6, "*")]
    return chunk(7, items=items)


def cbbc(rng):
    items = [(2, 4, "C"), (3, 4, "B"), (4, 4, "B"), (5, 4, "C"), (3, 6, "*"), (4, 6, "*")]
    return chunk(8, items=items)


def bcbcb(rng):
    items = [(2, 4, "B"), (3, 4, "C"), (4, 4, "B"), (5, 4, "C"), (6, 4, "B"), (4, 6, "*")]
    return chunk(9, items=items)


def steps(rng):
    # Two low blocks with gems over the gap between them, as in Sunny Meadow.
    items = [(2, 3, "B"), (5, 3, "B"), (3, 5, "*"), (4, 5, "*")]
    return chunk(8, items=items)


def mound(rng):
    # A small heap to hop over, as in Dusk Hills.
    items = [(1, 2, "B"), (2, 2, "B"), (3, 2, "B"), (2, 3, "B"), (3, 3, "B")]
    return chunk(6, items=items)


def gap(rng, w):
    items = [(w // 2, 5, "*")] if rng.random() < 0.5 else []
    return chunk(w, ground=False, items=items)


def checkpoint(rng):
    return chunk(5, items=[(2, 2, "F")])


def start():
    return chunk(6, items=[(2, 2, "P"), (4, 2, "t")])


def night_end():
    """The final staircase, key and gate, cut verbatim from Starry Night."""
    src = open(sys.argv[1]).read()
    body = re.search(r"const NIGHT: &\[&str\] = &\[(.*?)\];", src, re.S).group(1)
    lines = re.findall(r'"(.*?)"', body)
    assert len(lines) == HEIGHT
    rows = {HEIGHT - 1 - i: l for i, l in enumerate(lines)}
    width = max(len(l) for l in lines)
    # Start where the last stretch of ground starts, keeping its Grumbles.
    cut = rows[1].rindex(" ") + 1
    assert cut <= rows[2].index("BBBB")
    out = {r: list(rows[r][cut:].ljust(width - cut)) for r in range(2, HEIGHT)}
    ground = rows[1][cut:].ljust(width - cut)
    assert set(ground) == {"="}, ground
    return {"w": width - cut, "ground": True, "rows": out}


def place_grumbles(rng, lines, count):
    """Put `count` Grumbles where a hero could stand, at least 3 tiles apart, clear
    of the start, gap edges, checkpoints, heaps and the final staircase."""
    rows = {HEIGHT - 1 - i: list(l) for i, l in enumerate(lines)}
    width = max(len(l) for l in lines)
    for r in rows:
        rows[r] += [" "] * (width - len(rows[r]))
    stairs = "".join(rows[2]).index("BBBB")
    ok = []
    for c in range(10, stairs - 2):
        near = range(c - 2, c + 3)
        if any(rows[1][x] != "=" for x in near):
            continue  # a gap edge
        if any(rows[2][x] != " " for x in near) or rows[3][c] != " ":
            continue  # a checkpoint, a heap, a tree or a bush
        ok.append(c)
    rng.shuffle(ok)
    placed = []
    for c in ok:
        if len(placed) == count:
            break
        if all(abs(c - p) >= 3 for p in placed):
            placed.append(c)
    assert len(placed) == count, (count, len(placed))
    for c in placed:
        rows[2][c] = "e"
    return ["".join(rows[r]).rstrip() for r in range(HEIGHT - 1, -1, -1)]


def assemble(chunks):
    w = sum(c["w"] for c in chunks)
    rows = {r: [] for r in range(HEIGHT)}
    for c in chunks:
        for r in range(2, HEIGHT):
            rows[r] += c["rows"][r]
        g = ["="] * c["w"] if c["ground"] else [" "] * c["w"]
        rows[0] += g
        rows[1] += g
    lines = ["".join(rows[r]).rstrip() for r in range(HEIGHT - 1, -1, -1)]
    return lines


def plan(rng, features, grumbles, run_w, gaps):
    """Interleave features with runs; drop a checkpoint every third of the way."""
    chunks = [start(), run(rng, run_w[1], 0)]
    n = len(features)
    marks = {n // 3, (2 * n) // 3} if n >= 9 else {n // 2}
    gap_left = list(gaps)
    for i, f in enumerate(features):
        if i in marks:
            chunks += [checkpoint(rng), run(rng, rng.randint(*run_w), 0)]
        chunks.append(f(rng))
        chunks.append(run(rng, rng.randint(*run_w), 0))
        if gap_left and i % 2 == 1:
            # A gap, then a short landing strip with no Grumble on it.
            chunks.append(gap(rng, gap_left.pop(0)))
            chunks.append(run(rng, rng.randint(2, 3), 0, decor=False))
    chunks.append(night_end())
    lines = assemble(chunks)
    # The staircase stretch keeps the Grumbles it came with from Starry Night.
    return place_grumbles(rng, lines, grumbles - sum(c["rows"][2].count("e") for c in chunks[-1:]))


def check(name, lines):
    rows = {HEIGHT - 1 - i: l for i, l in enumerate(lines)}
    width = max(len(l) for l in lines)
    at = lambda c, r: rows[r][c] if c < len(rows[r]) else " "
    solid = lambda c, r: at(c, r) in "=BC"
    counts = {ch: sum(l.count(ch) for l in lines) for ch in "CPKGFe"}
    assert counts["P"] == counts["K"] == counts["G"] == 1, (name, counts)
    assert counts["C"] >= 5 and counts["F"] >= 1, (name, counts)
    for r in rows:
        for c, ch in enumerate(rows[r]):
            if ch in "PGFe":
                assert solid(c, r - 1) and not solid(c, r), (name, ch, c, r)
            if ch == "C":
                # Bumped from below: ground under it and room to stand.
                assert solid(c, 1) and not solid(c, 2) and not solid(c, 3), (name, "C", c, r)
    g = 0
    for c in range(width):
        if solid(c, 0):
            assert g <= 3, (name, "gap", c)
            g = 0
        else:
            g += 1
    assert rows[3].index("K") < rows[2].index("G")
    return counts, width


WORLDS = [
    # name, const, seed, features, Grumbles, stretch widths, gap widths
    ("Coral Coast", "CORAL", 6,
     [crystal, bcb, steps, crystal, cbbc, mound, crystal, bcb, crystal, cbbc, crystal, crystal],
     20, (2, 5), [2, 3, 2, 3, 3]),
    ("Autumn Woods", "AUTUMN", 7,
     [bcb, crystal, mound, cbbc, crystal, steps, bcb, crystal, bcbcb, crystal, cbbc, crystal],
     22, (2, 5), [3, 2, 3, 3, 3]),
    ("Misty Marsh", "MARSH", 8,
     [crystal, cbbc, steps, bcb, mound, crystal, bcbcb, crystal, cbbc, mound, crystal, bcb, crystal],
     24, (2, 5), [3, 3, 2, 3, 3, 3]),
    ("Volcano Valley", "VOLCANO", 9,
     [bcb, mound, crystal, bcbcb, steps, crystal, cbbc, mound, bcb, crystal, bcbcb, crystal, cbbc],
     26, (2, 4), [3, 3, 3, 2, 3, 3]),
    ("Rainbow Heights", "RAINBOW", 10,
     [crystal, bcbcb, mound, cbbc, crystal, steps, bcb, mound, crystal, bcbcb, cbbc, crystal, bcb, crystal],
     28, (2, 4), [3, 3, 3, 3, 3, 3, 3]),
]

for name, const, seed, features, grumbles, run_w, gaps in WORLDS:
    rng = random.Random(seed)
    lines = plan(rng, features, grumbles, run_w, gaps)
    counts, width = check(name, lines)
    print(f"// {name}: width {width}, crystals {counts['C']}, grumbles {counts['e']}, checkpoints {counts['F']}",
          file=sys.stderr)
    print(f"const {const}: &[&str] = &[")
    for l in lines:
        print(f'    "{l}",')
    print("];")
