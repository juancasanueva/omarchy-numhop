"""Procedural model builder for numhop.

Run inside Blender (Z-up). Every model is built as a single mesh object whose
geometry is already in final position, so object transforms stay identity at
export time. Characters face +X. Export uses export_yup=True, so Blender +X is
game +X, Blender +Z is game +Y (up) and Blender +Y is game -Z.

Usage from Blender's Python console or MCP:
    (paste this whole file; MCP safe mode forbids exec, open and os)
    build_all(export_dir="/path/to/assets/models")
"""

import math

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

PI = math.pi


# ---------------------------------------------------------------- materials

def srgb(hex_color):
    """Convert an sRGB hex string to a linear RGBA tuple for shader inputs."""
    hex_color = hex_color.lstrip("#")
    out = []
    for i in (0, 2, 4):
        c = int(hex_color[i:i + 2], 16) / 255.0
        out.append(c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4)
    return (out[0], out[1], out[2], 1.0)


def _socket(node, identifier):
    return next((s for s in node.inputs if s.identifier == identifier), None)


_MATERIALS = {}


def mat(name, color, roughness=0.6, metallic=0.0, emission=None, strength=0.0):
    """Create or reuse a Principled BSDF material with flat colors."""
    key = "nh_" + name
    if key in _MATERIALS:
        return _MATERIALS[key]
    m = bpy.data.materials.get(key) or bpy.data.materials.new(key)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    _socket(bsdf, "Base Color").default_value = srgb(color)
    _socket(bsdf, "Roughness").default_value = roughness
    _socket(bsdf, "Metallic").default_value = metallic
    if emission is not None:
        _socket(bsdf, "Emission Color").default_value = srgb(emission)
        _socket(bsdf, "Emission Strength").default_value = strength
    m.diffuse_color = srgb(color)
    _MATERIALS[key] = m
    return m


# ---------------------------------------------------------------- geometry

def g_cube(bm, bevel=0.0, segments=2):
    bmesh.ops.create_cube(bm, size=1.0)
    if bevel > 0:
        bmesh.ops.bevel(
            bm,
            geom=list(bm.verts) + list(bm.edges),
            offset=bevel,
            offset_type="OFFSET",
            segments=segments,
            profile=0.5,
            affect="EDGES",
        )


def g_sphere(bm, u=16, v=10):
    bmesh.ops.create_uvsphere(bm, u_segments=u, v_segments=v, radius=1.0)


def g_ico(bm, subdiv=2):
    bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=1.0)


def g_cyl(bm, segments=12, r1=1.0, r2=1.0, depth=1.0):
    bmesh.ops.create_cone(
        bm, cap_ends=True, cap_tris=False, segments=segments,
        radius1=r1, radius2=r2, depth=depth,
    )


def g_crystal(bm, segments=6, r=1.0, up=1.0, down=1.0):
    """Faceted double pyramid along +Z: crown height `up`, pavilion `down`."""
    ring = [bm.verts.new((r * math.cos(2 * PI * i / segments),
                          r * math.sin(2 * PI * i / segments), 0.0))
            for i in range(segments)]
    top = bm.verts.new((0, 0, up))
    bot = bm.verts.new((0, 0, -down))
    for i in range(segments):
        a, b = ring[i], ring[(i + 1) % segments]
        bm.faces.new([a, b, top])
        bm.faces.new([b, a, bot])


def g_torus(bm, major=1.0, minor=0.25, seg=18, rseg=8):
    rings = []
    for i in range(seg):
        a = 2 * PI * i / seg
        ring = []
        for j in range(rseg):
            b = 2 * PI * j / rseg
            ring.append(bm.verts.new((
                (major + minor * math.cos(b)) * math.cos(a),
                (major + minor * math.cos(b)) * math.sin(a),
                minor * math.sin(b),
            )))
        rings.append(ring)
    for i in range(seg):
        for j in range(rseg):
            bm.faces.new([rings[i][j], rings[(i + 1) % seg][j],
                          rings[(i + 1) % seg][(j + 1) % rseg],
                          rings[i][(j + 1) % rseg]])


def g_prism(bm, points_xz, depth):
    """Extrude a convex XZ polygon along Y, centered on y=0."""
    front = [bm.verts.new((x, -depth / 2, z)) for x, z in points_xz]
    back = [bm.verts.new((x, depth / 2, z)) for x, z in points_xz]
    bm.faces.new(front)
    bm.faces.new(list(reversed(back)))
    n = len(points_xz)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new([front[i], back[i], back[j], front[j]])


def build_geom(bm, spec):
    """Dispatch a geometry spec tuple. MCP safe mode forbids calling function
    values, so geometry is described as data and built here."""
    kind = spec[0]
    if kind == "cube":
        g_cube(bm, spec[1], spec[2])
    elif kind == "sphere":
        g_sphere(bm, spec[1], spec[2])
    elif kind == "ico":
        g_ico(bm, spec[1])
    elif kind == "cyl":
        g_cyl(bm, spec[1], spec[2], spec[3], spec[4])
    elif kind == "crystal":
        g_crystal(bm, spec[1], spec[2], spec[3], spec[4])
    elif kind == "torus":
        g_torus(bm, spec[1], spec[2], spec[3], spec[4])
    elif kind == "prism":
        g_prism(bm, spec[1], spec[2])


def trs(loc=(0, 0, 0), rot=(0, 0, 0), scale=(1, 1, 1)):
    return Matrix.LocRotScale(Vector(loc), Euler(rot, "XYZ"), Vector(scale))


# MCP safe mode forbids class definitions, so a model is a plain dict.
def new_model(name):
    """Start a model that accumulates primitive parts into one bmesh."""
    return {"name": name, "bm": bmesh.new(), "materials": []}


def add(m, geom, material, matrix=None, smooth=False):
    """Append one primitive part, transformed by `matrix`, to model `m`."""
    if material not in m["materials"]:
        m["materials"].append(material)
    idx = m["materials"].index(material)
    tmp = bmesh.new()
    build_geom(tmp, geom)
    if matrix is not None:
        bmesh.ops.transform(tmp, matrix=matrix, verts=tmp.verts)
    vmap = {v: m["bm"].verts.new(v.co) for v in tmp.verts}
    for f in tmp.faces:
        nf = m["bm"].faces.new([vmap[v] for v in f.verts])
        nf.material_index = idx
        nf.smooth = smooth
    tmp.free()


def finish(m):
    """Turn the accumulated bmesh into a mesh object in its own collection."""
    bm = m["bm"]
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(m["name"])
    bm.to_mesh(mesh)
    bm.free()
    for material in m["materials"]:
        mesh.materials.append(material)
    coll = bpy.data.collections.get(m["name"])
    if coll is None:
        coll = bpy.data.collections.new(m["name"])
        bpy.context.scene.collection.children.link(coll)
    obj = bpy.data.objects.new(m["name"], mesh)
    coll.objects.link(obj)
    return obj


# Shorthand geometry constructors with fixed parameters.
def cube(bevel=0.0, segments=2):
    return ("cube", bevel, segments)


def sphere(u=16, v=10):
    return ("sphere", u, v)


def ico(subdiv=2):
    return ("ico", subdiv)


def cyl(segments=12, r1=1.0, r2=1.0, depth=1.0):
    return ("cyl", segments, r1, r2, depth)


def crystal(segments=6, r=1.0, up=1.0, down=1.0):
    return ("crystal", segments, r, up, down)


def torus(major=1.0, minor=0.25, seg=18, rseg=8):
    return ("torus", major, minor, seg, rseg)


def prism(points_xz, depth):
    return ("prism", points_xz, depth)


# ---------------------------------------------------------------- models

def build_pip():
    m = new_model("hero_pip")
    orange = mat("pip_orange", "#F28A2E", 0.55)
    white = mat("pip_white", "#FFF4E6", 0.6)
    dark = mat("pip_dark", "#3A2A24", 0.5)
    black = mat("eye_black", "#141016", 0.25)
    shine = mat("eye_shine", "#FFFFFF", 0.2)
    pink = mat("pip_pink", "#E8838F", 0.6)

    # Body and belly.
    add(m, sphere(18, 10), orange, trs((0, 0, 0.25), scale=(0.21, 0.19, 0.22)), True)
    add(m, sphere(14, 8), white, trs((0.1, 0, 0.24), scale=(0.12, 0.14, 0.15)), True)
    # Head, snout, nose.
    add(m, sphere(20, 12), orange, trs((0.04, 0, 0.53), scale=(0.25, 0.24, 0.23)), True)
    add(m, sphere(14, 8), white, trs((0.22, 0, 0.47), scale=(0.13, 0.12, 0.08)), True)
    add(m, sphere(10, 6), dark, trs((0.345, 0, 0.495), scale=(0.035, 0.035, 0.03)), True)
    # Cheek tufts.
    for s in (-1, 1):
        add(m, sphere(12, 8), white, trs((0.12, s * 0.17, 0.47), scale=(0.08, 0.06, 0.07)), True)
    # Eyes with highlights.
    for s in (-1, 1):
        add(m, sphere(12, 8), black, trs((0.235, s * 0.1, 0.585), scale=(0.035, 0.04, 0.05)), True)
        add(m, sphere(8, 6), shine, trs((0.262, s * 0.09, 0.605), scale=(0.012, 0.012, 0.014)), True)
    # Ears with pink insides.
    for s in (-1, 1):
        add(m, cyl(4, 0.1, 0.0, 0.22), orange,
              trs((-0.01, s * 0.13, 0.77), rot=(s * -0.35, -0.1, 0), scale=(1, 0.8, 1)))
        add(m, cyl(4, 0.06, 0.0, 0.15), pink,
              trs((0.02, s * 0.13, 0.755), rot=(s * -0.35, -0.1, 0), scale=(0.5, 0.7, 1)))
    # Arms and feet.
    for s in (-1, 1):
        add(m, sphere(10, 6), orange, trs((0.11, s * 0.17, 0.27), scale=(0.055, 0.05, 0.06)), True)
        add(m, sphere(10, 6), dark, trs((0.06, s * 0.1, 0.045), scale=(0.08, 0.06, 0.045)), True)
    # Fluffy tail with dark tip.
    add(m, sphere(14, 8), orange, trs((-0.24, 0, 0.31), rot=(0, 0.75, 0), scale=(0.13, 0.08, 0.08)), True)
    add(m, sphere(12, 8), dark, trs((-0.33, 0, 0.42), rot=(0, 0.75, 0), scale=(0.07, 0.06, 0.06)), True)
    return finish(m)


def build_bolt():
    m = new_model("hero_bolt")
    teal = mat("bolt_teal", "#22B8B0", 0.45, 0.2)
    white = mat("bolt_white", "#F2F5F7", 0.4)
    grey = mat("bolt_grey", "#3B4048", 0.5, 0.4)
    screen = mat("bolt_screen", "#121820", 0.3)
    glow = mat("bolt_eye", "#7CF7FF", 0.3, 0, "#7CF7FF", 4.0)
    orange = mat("bolt_orange", "#FF8A3D", 0.5)
    yellow = mat("bolt_yellow", "#FFD23F", 0.4, 0, "#FFD23F", 2.0)

    # Legs and feet.
    for s in (-1, 1):
        add(m, cyl(10, 0.05, 0.05, 0.12), grey, trs((0, s * 0.11, 0.12)))
        add(m, cube(0.02), grey, trs((0.03, s * 0.11, 0.035), scale=(0.18, 0.11, 0.07)))
    # Body with chest light.
    add(m, cube(0.05), teal, trs((0, 0, 0.33), scale=(0.44, 0.4, 0.34)))
    add(m, cube(0.015), orange, trs((0.225, 0, 0.31), scale=(0.02, 0.16, 0.12)))
    add(m, cyl(10, 0.03, 0.03, 0.02), yellow, trs((0.24, 0, 0.31), rot=(0, PI / 2, 0)))
    # Arms.
    for s in (-1, 1):
        add(m, sphere(12, 8), white, trs((0.02, s * 0.235, 0.36), scale=(0.07, 0.06, 0.07)), True)
        add(m, sphere(10, 6), grey, trs((0.05, s * 0.245, 0.27), scale=(0.05, 0.045, 0.05)), True)
    # Neck and head.
    add(m, cyl(10, 0.06, 0.06, 0.05), grey, trs((0, 0, 0.52)))
    add(m, cube(0.05), white, trs((0, 0, 0.67), scale=(0.44, 0.42, 0.27)))
    # Face screen with a round glowing eye.
    add(m, cube(0.02), screen, trs((0.215, 0, 0.67), scale=(0.03, 0.34, 0.19)))
    add(m, cyl(16, 0.065, 0.065, 0.02), glow, trs((0.235, 0, 0.67), rot=(0, PI / 2, 0)))
    add(m, cyl(12, 0.025, 0.025, 0.022), screen, trs((0.24, 0, 0.675), rot=(0, PI / 2, 0)))
    # Ear bolts.
    for s in (-1, 1):
        add(m, cyl(10, 0.045, 0.045, 0.04), teal, trs((0, s * 0.225, 0.67), rot=(PI / 2, 0, 0)))
    # Antenna.
    add(m, cyl(8, 0.015, 0.015, 0.1), grey, trs((-0.04, 0, 0.84)))
    add(m, sphere(10, 6), yellow, trs((-0.04, 0, 0.9 - 0.035), scale=(0.035, 0.035, 0.035)), True)
    return finish(m)


def build_grumble():
    m = new_model("enemy_grumble")
    purple = mat("grumble_purple", "#8E44C9", 0.55)
    deep = mat("grumble_deep", "#4E1F78", 0.5)
    white = mat("grumble_white", "#FFFFFF", 0.3)
    black = mat("eye_black", "#141016", 0.25)
    add(m, sphere(20, 12), purple, trs((0, 0, 0.3), scale=(0.4, 0.36, 0.31)), True)
    # Spikes along the top ridge, tilted along the body curve, kept off the face.
    for th in (-1.25, -0.75, -0.25, 0.25):
        x = 0.4 * math.sin(th) * 0.93
        z = 0.3 + 0.31 * math.cos(th) * 0.93
        add(m, cyl(6, 0.085, 0.0, 0.2), deep, trs((x, 0, z), rot=(0, th, 0)))
    for s in (-1, 1):
        for th in (-0.7, 0.0):
            add(m, cyl(6, 0.07, 0.0, 0.16), deep,
                trs((0.4 * math.sin(th) * 0.8, s * 0.17, 0.3 + 0.31 * 0.85), rot=(s * -0.55, th, 0)))
    # Angry eyes and brows.
    for s in (-1, 1):
        add(m, sphere(12, 8), white, trs((0.3, s * 0.12, 0.36), scale=(0.07, 0.075, 0.085)), True)
        add(m, sphere(10, 6), black, trs((0.36, s * 0.11, 0.35), scale=(0.035, 0.04, 0.045)), True)
        add(m, cube(0.01, 1), deep, trs((0.345, s * 0.12, 0.455), rot=(s * 0.5, 0, 0), scale=(0.06, 0.15, 0.045)))
    # Frown with two little fangs.
    add(m, sphere(12, 6), deep, trs((0.375, 0, 0.215), scale=(0.035, 0.12, 0.03)), True)
    for s in (-1, 1):
        add(m, cyl(6, 0.025, 0.0, 0.055), white, trs((0.395, s * 0.06, 0.225)))
    # Feet.
    for s in (-1, 1):
        add(m, sphere(10, 6), deep, trs((0.08, s * 0.17, 0.035), scale=(0.1, 0.07, 0.04)), True)
    return finish(m)


def _block_frame(m, frame_mat, socket_mat):
    add(m, cube(0.06, 2), frame_mat, trs(scale=(0.92, 0.92, 0.92)))
    # Corner studs make the frame read as a crafted box.
    for sx in (-1, 1):
        for sz in (-1, 1):
            for sy in (-1, 1):
                add(m, sphere(8, 6), frame_mat,
                      trs((sx * 0.44, sy * 0.44, sz * 0.44), scale=(0.06, 0.06, 0.06)), True)
    # A socket ring on each side face.
    for rot, loc in _side_faces(0.455):
        add(m, cyl(12, 0.3, 0.3, 0.03), socket_mat, trs(loc, rot=rot))


def _side_faces(dist):
    """Rotations that point local +Z out of each side face (±X, ±Y)."""
    return [
        ((PI / 2, 0, 0), (0, -dist, 0)),    # -Y, faces the camera
        ((-PI / 2, 0, 0), (0, dist, 0)),    # +Y
        ((0, PI / 2, 0), (dist, 0, 0)),     # +X
        ((0, -PI / 2, 0), (-dist, 0, 0)),   # -X
    ]


def build_block_crystal():
    m = new_model("block_crystal")
    gold = mat("block_gold", "#F0A830", 0.35, 0.6)
    socket = mat("block_socket", "#8A5A1C", 0.5, 0.3)
    gem = mat("crystal_cyan", "#38E8FF", 0.15, 0, "#38E8FF", 3.0)
    _block_frame(m, gold, socket)
    for rot, loc in _side_faces(0.47):
        add(m, crystal(6, 0.25, 0.06, 0.02), gem, trs(loc, rot=rot))
    return finish(m)


def build_block_used():
    m = new_model("block_used")
    dull = mat("block_dull", "#6B5D52", 0.8, 0.1)
    socket = mat("block_hole", "#2F2722", 0.9)
    _block_frame(m, dull, socket)
    return finish(m)


def build_block_brick():
    m = new_model("block_brick")
    stone = mat("brick_stone", "#A79A8C", 0.85)
    light = mat("brick_light", "#BFB3A5", 0.85)
    dark = mat("brick_dark", "#8C8073", 0.85)
    add(m, cube(0.04), dark, trs(scale=(0.94, 0.94, 0.94)))
    # Four raised stones per side, staggered colors, read as carved masonry.
    tiles = [(-0.235, 0.235, light), (0.235, 0.235, stone),
             (-0.235, -0.235, stone), (0.235, -0.235, light)]
    for rot, loc in _side_faces(0.485):
        for u, v, col in tiles:
            local = trs((u, v, 0), scale=(0.44, 0.44, 0.03))
            add(m, cube(0.03, 1), col, trs(loc, rot=rot) @ local)
    # Top and bottom caps.
    for z in (0.485, -0.485):
        add(m, cube(0.03, 1), stone, trs((0, 0, z), scale=(0.9, 0.9, 0.03)))
    return finish(m)


def build_ground_tile():
    m = new_model("ground_tile")
    soil = mat("ground_soil", "#8B5A34", 0.9)
    soil_dark = mat("ground_soil_dark", "#6E4426", 0.9)
    grass = mat("ground_grass", "#5BC24A", 0.7)
    grass_dark = mat("ground_grass_dark", "#45A53A", 0.7)
    pebble = mat("ground_pebble", "#B9A48E", 0.8)

    add(m, cube(), soil, trs((0, 0, -0.575), scale=(1.0, 3.0, 0.85)))
    add(m, cube(), soil_dark, trs((0, 0, -0.95), scale=(1.0, 2.9, 0.1)))
    add(m, cube(), grass, trs((0, 0, -0.075), scale=(1.0, 3.0, 0.15)))
    # Grass lip hanging over the front edge, spaced to tile along X.
    for x in (-0.333, 0.0, 0.333):
        add(m, sphere(10, 6), grass_dark, trs((x, -1.5, -0.15), scale=(0.17, 0.08, 0.09)), True)
    # Pebbles set into the front face.
    for x, z, s in ((-0.28, -0.45, 0.07), (0.22, -0.7, 0.09), (0.05, -0.32, 0.05)):
        add(m, ico(1), pebble, trs((x, -1.5, z), scale=(s, 0.03, s * 0.8)))
    return finish(m)


def build_gem():
    m = new_model("gem")
    pink = mat("gem_pink", "#FF3FA4", 0.15, 0, "#FF3FA4", 1.5)
    add(m, crystal(8, 0.15, 0.12, 0.25), pink, trs((0, 0, 0.065)))
    return finish(m)


def build_key():
    m = new_model("key")
    gold = mat("key_gold", "#FFC233", 0.3, 0.85)
    gem = mat("key_gem", "#38E8FF", 0.15, 0, "#38E8FF", 2.0)
    # Bow (ring) facing the camera, in the XZ plane.
    add(m, torus(0.11, 0.035, 18, 8), gold, trs((-0.21, 0, 0), rot=(PI / 2, 0, 0)))
    add(m, crystal(6, 0.05, 0.03, 0.03), gem, trs((-0.21, 0, 0), rot=(PI / 2, 0, 0)))
    # Collar and shaft.
    add(m, cyl(12, 0.05, 0.05, 0.05), gold, trs((-0.08, 0, 0), rot=(0, PI / 2, 0)))
    add(m, cyl(12, 0.028, 0.028, 0.36), gold, trs((0.12, 0, 0), rot=(0, PI / 2, 0)))
    # Bit with two teeth.
    add(m, cube(0.008), gold, trs((0.22, 0, -0.065), scale=(0.05, 0.05, 0.1)))
    add(m, cube(0.008), gold, trs((0.29, 0, -0.05), scale=(0.04, 0.05, 0.07)))
    return finish(m)


def build_gate():
    m = new_model("gate")
    stone = mat("gate_stone", "#8A93A6", 0.85)
    stone_dark = mat("gate_stone_dark", "#6B7387", 0.85)
    gem = mat("gate_gem", "#38E8FF", 0.15, 0, "#38E8FF", 3.0)
    portal = mat("gate_portal", "#7B3FF2", 0.3, 0, "#7B3FF2", 2.5)
    swirl = mat("gate_swirl", "#D9B8FF", 0.3, 0, "#D9B8FF", 3.0)

    # Base step and pillars.
    add(m, cube(0.03), stone_dark, trs((0, 0, 0.06), scale=(2.0, 0.6, 0.12)))
    for s in (-1, 1):
        add(m, cube(0.03), stone, trs((s * 0.8, 0, 1.06), scale=(0.4, 0.55, 1.88)))
        add(m, cube(0.03), stone_dark, trs((s * 0.8, 0, 0.25), scale=(0.44, 0.6, 0.2)))
    # Arch made of voussoirs on a 0.8 radius centerline.
    n = 7
    for i in range(n):
        a = PI * (i + 0.5) / n
        col = stone if i % 2 == 0 else stone_dark
        size = (0.4, 0.55, 2 * PI * 0.8 / (2 * n) * 0.94)
        if i == n // 2:
            size = (0.46, 0.6, size[2] * 1.1)
        add(m, cube(0.03), col,
              trs((0.8 * math.cos(a), 0, 2.0 + 0.8 * math.sin(a)), rot=(0, -a, 0), scale=size))
    add(m, crystal(6, 0.09, 0.04, 0.02), gem, trs((0, -0.31, 2.8), rot=(PI / 2, 0, 0)))
    # Portal: a rectangle plus a half disc filling the arch opening.
    pts = [(-0.6, 0.12), (0.6, 0.12), (0.6, 2.0)]
    for i in range(1, 12):
        a = PI * i / 12
        pts.append((0.6 * math.cos(a), 2.0 + 0.6 * math.sin(a)))
    pts.append((-0.6, 2.0))
    add(m, prism(pts, 0.04), portal, None)
    # Swirl rings floating on the portal surface.
    for r, z in ((0.42, 1.45), (0.25, 1.45), (0.1, 1.45)):
        add(m, torus(r, 0.022, 20, 6), swirl, trs((0, -0.04, z), rot=(PI / 2, 0, 0)))
    return finish(m)


def build_flag():
    m = new_model("flag_checkpoint")
    wood = mat("flag_wood", "#8B5A34", 0.8)
    stone = mat("flag_stone", "#8A93A6", 0.85)
    teal = mat("flag_teal", "#22C7B8", 0.6)
    gold = mat("flag_gold", "#FFC233", 0.3, 0.8)
    add(m, cyl(12, 0.16, 0.13, 0.12), stone, trs((0, 0, 0.06)))
    add(m, cyl(10, 0.04, 0.035, 1.82), wood, trs((0, 0, 1.03)))
    add(m, sphere(10, 8), gold, trs((0, 0, 1.94), scale=(0.06, 0.06, 0.06)), True)
    add(m, prism([(0.04, 1.35), (0.72, 1.58), (0.04, 1.85)], 0.04), teal, None)
    return finish(m)


def build_tree():
    m = new_model("tree")
    bark = mat("tree_bark", "#7A4B2A", 0.85)
    leaf = mat("tree_leaf", "#3FAF4E", 0.7)
    leaf_light = mat("tree_leaf_light", "#62C95A", 0.7)
    add(m, cyl(8, 0.16, 0.1, 1.4), bark, trs((0, 0, 0.7)))
    add(m, ico(2), leaf, trs((0, 0, 2.2), scale=(0.8, 0.75, 0.8)))
    add(m, ico(1), leaf_light, trs((0.5, -0.15, 1.75), scale=(0.5, 0.5, 0.45)))
    add(m, ico(1), leaf_light, trs((-0.5, 0.1, 1.85), scale=(0.45, 0.45, 0.42)))
    return finish(m)


def build_bush():
    m = new_model("bush")
    leaf = mat("bush_leaf", "#4DBA4A", 0.7)
    leaf_light = mat("bush_leaf_light", "#6FD65E", 0.7)
    add(m, ico(2), leaf, trs((0, 0, 0.4), scale=(0.42, 0.38, 0.4)))
    add(m, ico(1), leaf_light, trs((0.33, -0.05, 0.27), scale=(0.28, 0.26, 0.27)))
    add(m, ico(1), leaf_light, trs((-0.33, 0.05, 0.25), scale=(0.26, 0.25, 0.25)))
    return finish(m)


def build_cloud():
    m = new_model("cloud")
    white = mat("cloud_white", "#FFFFFF", 0.9)
    puffs = [(-1.05, 0, -0.05, 0.42), (-0.5, 0, 0.15, 0.6), (0.2, 0, 0.25, 0.7),
             (0.85, 0, 0.0, 0.5), (0.0, 0, -0.15, 0.5)]
    for x, y, z, r in puffs:
        add(m, sphere(16, 10), white, trs((x, y, z), scale=(r, r * 0.75, r * 0.85)), True)
    return finish(m)


BUILDERS = [
    build_pip, build_bolt, build_grumble, build_block_crystal, build_block_used,
    build_block_brick, build_ground_tile, build_gem, build_key, build_gate,
    build_flag, build_tree, build_bush, build_cloud,
]


# ---------------------------------------------------------------- pipeline

def clear_scene():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    for coll in list(bpy.data.collections):
        bpy.data.collections.remove(coll)
    for mesh in list(bpy.data.meshes):
        bpy.data.meshes.remove(mesh)
    for m in list(bpy.data.materials):
        bpy.data.materials.remove(m)
    _MATERIALS.clear()


def recenter(obj):
    """Shift centered models so their bounding box center sits on the origin."""
    xs = [v.co.x for v in obj.data.vertices]
    ys = [v.co.y for v in obj.data.vertices]
    zs = [v.co.z for v in obj.data.vertices]
    c = Vector(((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2, (min(zs) + max(zs)) / 2))
    for v in obj.data.vertices:
        v.co -= c


CENTERED = {"gem", "key", "cloud"}


def export_glb(obj, path):
    for o in bpy.context.view_layer.objects:
        o.select_set(False)
    saved = obj.location.copy()
    obj.location = (0, 0, 0)
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.export_scene.gltf(
        filepath=path,
        use_selection=True,
        export_format="GLB",
        export_yup=True,
        export_apply=True,
    )
    obj.location = saved
    obj.select_set(False)


def bounds(obj):
    xs = [v.co.x for v in obj.data.vertices]
    ys = [v.co.y for v in obj.data.vertices]
    zs = [v.co.z for v in obj.data.vertices]
    return (min(xs), min(ys), min(zs)), (max(xs), max(ys), max(zs))


def build_all(export_dir=None, layout_spacing=3.5):
    clear_scene()
    objects = []
    for i, build in enumerate(BUILDERS):
        obj = build()
        if obj.name in CENTERED:
            recenter(obj)
        objects.append(obj)
    report = []
    for i, obj in enumerate(objects):
        if export_dir:
            path = export_dir.rstrip("/") + "/" + obj.name + ".glb"
            export_glb(obj, path)
        lo, hi = bounds(obj)
        tris = sum(len(p.vertices) - 2 for p in obj.data.polygons)
        report.append((obj.name, lo, hi, tris))
        # Spread the models out in the .blend for browsing; exports use origin.
        obj.location = ((i % 7) * layout_spacing, (i // 7) * layout_spacing, 0)
    return report
