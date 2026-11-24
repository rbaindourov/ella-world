#!/usr/bin/env python3
"""
ELLA World — Headless Blender 5.2 Props, Home Interiors & Legendary Loot Pack Builder
(`scripts/build_props_and_loot_pack.py`)

Follows `blender-3d-asset-pipeline` rules:
  1. Reuses & normalizes CC0 PolyHaven 3D base meshes (`dining_table`, `painted_wooden_chair_01`,
     `painted_wooden_cabinet_02`, `painted_wooden_nightstand`, `vintage_day_bed`, `brass_goblets`)
     with mandatory parent-Empty `matrix_world` flattening (Rule A) and multi-slot PBR material
     assignment (Rule C).
  2. Crafts voxel-grid-aligned (`1x1x1` / `1x2x1`) interactive hinged Doors & Windows, Stairs,
     Kitchen & Bathroom appliances, Monster/Chest 3D collectibles, and the first Legendary item:
     the **Starlight Plasma Ray Gun (`Loot_Ray_Gun`)**.
  3. Saves native `.blend` (`assets/blender/ella_world_props_loot.blend`), exports WebGL `.glb`
     (`public/models/ella_world_props_loot.glb` with `export_yup=True`), and renders two EEVEE
     studio verification sheets (`scratch/props_furniture_preview.png` &
     `scratch/loot_raygun_preview.png`).

Run via:
  /home/ella/.local/bin/blender -b --factory-startup -P /home/ella/projects/ella-world/scripts/build_props_and_loot_pack.py
"""

import math
import os
import bpy
import bmesh
from mathutils import Matrix, Vector, Euler

REPO_ROOT = "/home/ella/projects/ella-world"
CC0_DIR = os.path.join(REPO_ROOT, "scratch", "cc0_sources")
OUT_BLEND = os.path.join(REPO_ROOT, "assets", "blender", "ella_world_props_loot.blend")
OUT_GLB = os.path.join(REPO_ROOT, "public", "models", "ella_world_props_loot.glb")
PREVIEW_FURNITURE = os.path.join(REPO_ROOT, "scratch", "props_furniture_preview.png")
PREVIEW_LOOT = os.path.join(REPO_ROOT, "scratch", "loot_raygun_preview.png")

os.makedirs(os.path.dirname(OUT_BLEND), exist_ok=True)
os.makedirs(os.path.dirname(OUT_GLB), exist_ok=True)
os.makedirs(os.path.join(REPO_ROOT, "scratch"), exist_ok=True)


# ============================================================================
# 1. Scene Reset & Shared Stylized PBR Material Palette
# ============================================================================

def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if "World" not in bpy.data.worlds:
        w = bpy.data.worlds.new("World")
    else:
        w = bpy.data.worlds["World"]
    bpy.context.scene.world = w
    w.use_nodes = True
    bg = w.node_tree.nodes.get("Background")
    if bg:
        bg.inputs[0].default_value = (0.14, 0.16, 0.24, 1.0)
        bg.inputs[1].default_value = 0.85


def make_pbr_mat(
    name: str,
    color=(0.8, 0.8, 0.8, 1.0),
    metallic=0.0,
    roughness=0.45,
    emission=None,
    emission_strength=0.0,
    alpha=1.0,
):
    mat = bpy.data.materials.get(name) or bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    out = nodes.new("ShaderNodeOutputMaterial")
    out.location = (320, 0)
    bsdf = nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.location = (0, 0)

    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    if "Alpha" in bsdf.inputs:
        bsdf.inputs["Alpha"].default_value = alpha
    if alpha < 0.999:
        mat.blend_method = "BLEND" if hasattr(mat, "blend_method") else "HASHED"
        if hasattr(mat, "surface_render_method"):
            mat.surface_render_method = "BLENDED"
    if emission is not None and "Emission Color" in bsdf.inputs:
        bsdf.inputs["Emission Color"].default_value = emission
        bsdf.inputs["Emission Strength"].default_value = emission_strength

    links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def build_palette():
    return {
        # Dynamic player-swatch tint slot (replaced per-instance in Three.js if block.color is set)
        "tint": make_pbr_mat("Mat_Prop_Tint", (0.95, 0.45, 0.72, 1.0), metallic=0.06, roughness=0.38),
        "wood_warm": make_pbr_mat("Mat_Prop_WoodWarm", (0.58, 0.36, 0.20, 1.0), metallic=0.02, roughness=0.52),
        "wood_dark": make_pbr_mat("Mat_Prop_WoodDark", (0.30, 0.18, 0.11, 1.0), metallic=0.04, roughness=0.48),
        "wood_cream": make_pbr_mat("Mat_Prop_WoodCream", (0.94, 0.91, 0.85, 1.0), metallic=0.02, roughness=0.42),
        "porcelain": make_pbr_mat("Mat_Prop_Porcelain", (0.96, 0.97, 0.99, 1.0), metallic=0.05, roughness=0.16),
        "marble": make_pbr_mat("Mat_Prop_Marble", (0.91, 0.92, 0.95, 1.0), metallic=0.04, roughness=0.22),
        "chrome": make_pbr_mat("Mat_Prop_Chrome", (0.84, 0.88, 0.93, 1.0), metallic=0.92, roughness=0.14),
        "gold": make_pbr_mat(
            "Mat_Prop_Gold",
            (0.98, 0.78, 0.22, 1.0),
            metallic=0.88,
            roughness=0.18,
            emission=(0.98, 0.72, 0.15, 1.0),
            emission_strength=0.12,
        ),
        "obsidian": make_pbr_mat("Mat_Prop_Obsidian", (0.11, 0.13, 0.20, 1.0), metallic=0.35, roughness=0.28),
        "glass": make_pbr_mat(
            "Mat_Prop_Glass",
            (0.62, 0.90, 0.99, 1.0),
            metallic=0.10,
            roughness=0.08,
            emission=(0.35, 0.78, 0.98, 1.0),
            emission_strength=0.18,
            alpha=0.36,
        ),
        "water": make_pbr_mat(
            "Mat_Prop_Water",
            (0.22, 0.82, 0.96, 1.0),
            metallic=0.08,
            roughness=0.10,
            emission=(0.15, 0.68, 0.92, 1.0),
            emission_strength=0.28,
            alpha=0.78,
        ),
        "mirror": make_pbr_mat(
            "Mat_Prop_Mirror",
            (0.82, 0.93, 1.0, 1.0),
            metallic=0.96,
            roughness=0.05,
            emission=(0.65, 0.88, 1.0, 1.0),
            emission_strength=0.22,
        ),
        "burner": make_pbr_mat(
            "Mat_Prop_Burner",
            (1.0, 0.36, 0.12, 1.0),
            metallic=0.15,
            roughness=0.25,
            emission=(1.0, 0.38, 0.08, 1.0),
            emission_strength=2.2,
        ),
        "lamp_glow": make_pbr_mat(
            "Mat_Prop_LampGlow",
            (1.0, 0.93, 0.68, 1.0),
            metallic=0.0,
            roughness=0.28,
            emission=(1.0, 0.88, 0.52, 1.0),
            emission_strength=2.4,
        ),
        "fabric_white": make_pbr_mat("Mat_Prop_FabricWhite", (0.97, 0.96, 0.98, 1.0), metallic=0.0, roughness=0.72),
        "velvet_crimson": make_pbr_mat("Mat_Prop_Velvet", (0.62, 0.09, 0.24, 1.0), metallic=0.08, roughness=0.58),
        "basalt": make_pbr_mat("Mat_Loot_Basalt", (0.18, 0.17, 0.23, 1.0), metallic=0.12, roughness=0.72),
        "amethyst": make_pbr_mat(
            "Mat_Loot_Amethyst",
            (0.76, 0.34, 0.98, 1.0),
            metallic=0.18,
            roughness=0.12,
            emission=(0.72, 0.28, 0.98, 1.0),
            emission_strength=2.1,
        ),
        "slime_glow": make_pbr_mat(
            "Mat_Loot_Slime",
            (0.20, 0.95, 0.62, 1.0),
            metallic=0.05,
            roughness=0.16,
            emission=(0.16, 0.92, 0.55, 1.0),
            emission_strength=1.9,
        ),
        "plush_lavender": make_pbr_mat("Mat_Loot_PlushBody", (0.72, 0.56, 0.96, 1.0), metallic=0.0, roughness=0.82),
        "plush_pink": make_pbr_mat("Mat_Loot_PlushPink", (0.98, 0.55, 0.78, 1.0), metallic=0.0, roughness=0.76),
        "ruby_gem": make_pbr_mat(
            "Mat_Loot_Ruby",
            (0.98, 0.18, 0.36, 1.0),
            metallic=0.15,
            roughness=0.10,
            emission=(0.98, 0.16, 0.34, 1.0),
            emission_strength=1.6,
        ),
        "dragon_scale": make_pbr_mat(
            "Mat_Loot_DragonEgg",
            (0.54, 0.24, 0.92, 1.0),
            metallic=0.68,
            roughness=0.18,
            emission=(0.32, 0.75, 0.98, 1.0),
            emission_strength=0.65,
        ),
        "raygun_body": make_pbr_mat(
            "Mat_RayGun_Body",
            (0.88, 0.18, 0.58, 1.0),
            metallic=0.74,
            roughness=0.20,
            emission=(0.45, 0.08, 0.32, 1.0),
            emission_strength=0.25,
        ),
        "raygun_plasma": make_pbr_mat(
            "Mat_RayGun_Plasma",
            (0.22, 0.95, 1.0, 1.0),
            metallic=0.10,
            roughness=0.08,
            emission=(0.18, 0.92, 1.0, 1.0),
            emission_strength=3.2,
        ),
    }


# ============================================================================
# 2. Geometry & CC0 Import / Matrix-Flattening Utilities
# ============================================================================

def add_part_box(bm, mat_idx, size=(1.0, 1.0, 1.0), center=(0.0, 0.0, 0.0), rot_euler=(0.0, 0.0, 0.0), bevel=0.0):
    res = bmesh.ops.create_cube(bm, size=1.0)
    verts = res["verts"]
    sx, sy, sz = size
    for v in verts:
        v.co.x *= sx
        v.co.y *= sy
        v.co.z *= sz
    if bevel > 0.0:
        edges = list({e for v in verts for e in v.link_edges})
        bmesh.ops.bevel(bm, geom=edges, offset=bevel, segments=2, profile=0.5, affect="EDGES")
        verts = list(bm.verts)
    rot_m = Euler(rot_euler, "XYZ").to_matrix().to_4x4()
    xform = Matrix.Translation(Vector(center)) @ rot_m
    if bevel == 0.0:
        bmesh.ops.transform(bm, matrix=xform, verts=verts)
        for f in {f for v in verts for f in v.link_faces}:
            f.material_index = mat_idx


def create_mesh_object(name: str, materials: list):
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    for m in materials:
        mesh.materials.append(m)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def append_prim_box(bm, mat_idx, size, center, rot_euler=(0.0, 0.0, 0.0)):
    sub = bmesh.new()
    bmesh.ops.create_cube(sub, size=1.0)
    for v in sub.verts:
        v.co.x *= size[0]
        v.co.y *= size[1]
        v.co.z *= size[2]
    edges = list(sub.edges)
    bevel_amt = min(size) * 0.08
    if bevel_amt > 0.002:
        bmesh.ops.bevel(sub, geom=edges, offset=bevel_amt, segments=2, profile=0.5, affect="EDGES")
    xform = Matrix.Translation(Vector(center)) @ Euler(rot_euler, "XYZ").to_matrix().to_4x4()
    bmesh.ops.transform(sub, matrix=xform, verts=sub.verts)
    for f in sub.faces:
        f.material_index = mat_idx
        f.smooth = True
    _merge_bmesh(bm, sub)
    sub.free()


def append_prim_cyl(bm, mat_idx, r1, r2, depth, center, segments=16, rot_euler=(0.0, 0.0, 0.0), smooth=True):
    sub = bmesh.new()
    bmesh.ops.create_cone(
        sub,
        cap_ends=True,
        cap_tris=False,
        segments=segments,
        radius1=r1,
        radius2=r2,
        depth=depth,
    )
    xform = Matrix.Translation(Vector(center)) @ Euler(rot_euler, "XYZ").to_matrix().to_4x4()
    bmesh.ops.transform(sub, matrix=xform, verts=sub.verts)
    for f in sub.faces:
        f.material_index = mat_idx
        f.smooth = smooth
    _merge_bmesh(bm, sub)
    sub.free()


def append_prim_sphere(bm, mat_idx, radius, center, u_seg=16, v_seg=12, scale=(1.0, 1.0, 1.0), smooth=True):
    sub = bmesh.new()
    bmesh.ops.create_uvsphere(sub, u_segments=u_seg, v_segments=v_seg, radius=radius)
    for v in sub.verts:
        v.co.x *= scale[0]
        v.co.y *= scale[1]
        v.co.z *= scale[2]
        v.co += Vector(center)
    for f in sub.faces:
        f.material_index = mat_idx
        f.smooth = smooth
    _merge_bmesh(bm, sub)
    sub.free()


def append_prim_torus(bm, mat_idx, major_r, minor_r, center, maj_seg=20, min_seg=10, rot_euler=(0.0, 0.0, 0.0)):
    sub = bmesh.new()
    for i in range(maj_seg):
        a = (2.0 * math.pi * i) / maj_seg
        ca, sa = math.cos(a), math.sin(a)
        for j in range(min_seg):
            b = (2.0 * math.pi * j) / min_seg
            cb, sb = math.cos(b), math.sin(b)
            r = major_r + minor_r * cb
            sub.verts.new((r * ca, r * sa, minor_r * sb))
    sub.verts.ensure_lookup_table()
    for i in range(maj_seg):
        ni = (i + 1) % maj_seg
        for j in range(min_seg):
            nj = (j + 1) % min_seg
            v00 = sub.verts[i * min_seg + j]
            v10 = sub.verts[ni * min_seg + j]
            v11 = sub.verts[ni * min_seg + nj]
            v01 = sub.verts[i * min_seg + nj]
            f = sub.faces.new((v00, v10, v11, v01))
            f.material_index = mat_idx
            f.smooth = True
    xform = Matrix.Translation(Vector(center)) @ Euler(rot_euler, "XYZ").to_matrix().to_4x4()
    bmesh.ops.transform(sub, matrix=xform, verts=sub.verts)
    _merge_bmesh(bm, sub)
    sub.free()


def append_prim_star(bm, mat_idx, outer_r, inner_r, thickness, center, rot_euler=(0.0, 0.0, 0.0)):
    sub = bmesh.new()
    pts = 5
    ring_top = []
    ring_bot = []
    for i in range(pts * 2):
        ang = (math.pi * i) / pts - math.pi / 2.0
        r = outer_r if i % 2 == 0 else inner_r
        x, y = math.cos(ang) * r, math.sin(ang) * r
        ring_top.append(sub.verts.new((x, y, thickness * 0.5)))
        ring_bot.append(sub.verts.new((x, y, -thickness * 0.5)))
    c_top = sub.verts.new((0.0, 0.0, thickness * 0.65))
    c_bot = sub.verts.new((0.0, 0.0, -thickness * 0.65))
    n = len(ring_top)
    for i in range(n):
        ni = (i + 1) % n
        f1 = sub.faces.new((c_top, ring_top[i], ring_top[ni]))
        f2 = sub.faces.new((c_bot, ring_bot[ni], ring_bot[i]))
        f3 = sub.faces.new((ring_bot[i], ring_bot[ni], ring_top[ni], ring_top[i]))
        for f in (f1, f2, f3):
            f.material_index = mat_idx
            f.smooth = False
    xform = Matrix.Translation(Vector(center)) @ Euler(rot_euler, "XYZ").to_matrix().to_4x4()
    bmesh.ops.transform(sub, matrix=xform, verts=sub.verts)
    _merge_bmesh(bm, sub)
    sub.free()


def _merge_bmesh(dst_bm, src_bm):
    tmp = bpy.data.meshes.new("_tmp_merge")
    src_bm.to_mesh(tmp)
    dst_bm.from_mesh(tmp)
    bpy.data.meshes.remove(tmp)


def load_cc0_bmesh(blend_filename: str, mesh_names=None, target_bounds=(0.88, 0.88, 0.85), mat_map=None):
    """
    Appends mesh objects from a downloaded CC0 PolyHaven `.blend`, applies Rule A parent-Empty
    `matrix_world` flattening, normalizes into `target_bounds` `(sx, sy, sz)` centered on `(0, 0)`
    with base at `Z = 0.0`, and returns a combined `bmesh`.
    """
    blend_path = os.path.join(CC0_DIR, blend_filename)
    if not os.path.exists(blend_path):
        return None

    with bpy.data.libraries.load(blend_path, link=False) as (data_from, data_to):
        if mesh_names:
            data_to.objects = [n for n in data_from.objects if n in mesh_names]
        else:
            data_to.objects = list(data_from.objects)

    imported_meshes = []
    imported_others = []
    for o in data_to.objects:
        if o is None:
            continue
        bpy.context.scene.collection.objects.link(o)
        if o.type == "MESH":
            imported_meshes.append(o)
        else:
            imported_others.append(o)

    bpy.context.view_layer.update()

    # Rule A: Bake matrix_world BEFORE deleting any parent Empty nodes
    for o in imported_meshes:
        if o.data.users > 1:
            o.data = o.data.copy()
        mw = o.matrix_world.copy()
        o.parent = None
        o.data.transform(mw)
        o.matrix_world = Matrix.Identity(4)
        o.data.update()

    for o in imported_others:
        bpy.data.objects.remove(o, do_unlink=True)

    if not imported_meshes:
        return None

    combined = bmesh.new()
    for o in imported_meshes:
        m_idx = 0
        if mat_map and o.name in mat_map:
            m_idx = mat_map[o.name]
        sub = bmesh.new()
        sub.from_mesh(o.data)
        for f in sub.faces:
            f.material_index = m_idx
            f.smooth = True
        _merge_bmesh(combined, sub)
        sub.free()
        mesh_data = o.data
        bpy.data.objects.remove(o, do_unlink=True)
        bpy.data.meshes.remove(mesh_data)

    # Compute bounding box & normalize cleanly to target_bounds with bottom at Z = 0
    xs = [v.co.x for v in combined.verts]
    ys = [v.co.y for v in combined.verts]
    zs = [v.co.z for v in combined.verts]
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    min_z, max_z = min(zs), max(zs)
    cx = 0.5 * (min_x + max_x)
    cy = 0.5 * (min_y + max_y)
    dx = max(1e-4, max_x - min_x)
    dy = max(1e-4, max_y - min_y)
    dz = max(1e-4, max_z - min_z)

    sx = target_bounds[0] / dx
    sy = target_bounds[1] / dy
    sz = target_bounds[2] / dz

    for v in combined.verts:
        v.co.x = (v.co.x - cx) * sx
        v.co.y = (v.co.y - cy) * sy
        v.co.z = (v.co.z - min_z) * sz

    bmesh.ops.recalc_face_normals(combined, faces=combined.faces)
    return combined


def finalize_obj(name: str, bm: bmesh.types.BMesh, materials: list, location=(0.0, 0.0, 0.0)):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    obj = create_mesh_object(name, materials)
    bm.to_mesh(obj.data)
    bm.free()
    obj.data.update()
    obj.location = Vector(location)
    return obj


# ============================================================================
# 3. Build Architectural & Home Interior Props
# ============================================================================

def build_all_assets(pal):
    built = {}

    # 1) Prop_Stairs (1x1x1: rises from front -Y -> back +Y in Blender, i.e., +Z -> -Z in Three.js)
    bm = bmesh.new()
    # Lower step (front half: y in [-0.5, 0.0], z in [0.0, 0.5])
    append_prim_box(bm, 0, (0.98, 0.50, 0.48), (0.0, -0.25, 0.24))
    # Upper step (back half: y in [0.0, 0.5], z in [0.0, 1.0])
    append_prim_box(bm, 0, (0.98, 0.50, 0.98), (0.0, 0.25, 0.49))
    # Plush stair runner carpet strip (tintable!)
    append_prim_box(bm, 1, (0.56, 0.52, 0.04), (0.0, -0.24, 0.49))
    append_prim_box(bm, 1, (0.56, 0.04, 0.52), (0.0, 0.0, 0.74))
    append_prim_box(bm, 1, (0.56, 0.52, 0.04), (0.0, 0.25, 0.99))
    # Gold stair nosing bars
    append_prim_cyl(bm, 2, 0.022, 0.022, 0.62, (0.0, -0.48, 0.50), rot_euler=(0.0, math.pi / 2, 0.0))
    append_prim_cyl(bm, 2, 0.022, 0.022, 0.62, (0.0, 0.01, 1.00), rot_euler=(0.0, math.pi / 2, 0.0))
    built["Prop_Stairs"] = finalize_obj("Prop_Stairs", bm, [pal["wood_warm"], pal["tint"], pal["gold"]])

    # 2) Prop_Door_Frame + Prop_Door_Leaf (1x2x1 hinged door)
    bm_f = bmesh.new()
    # Left & right jambs + header + threshold
    append_prim_box(bm_f, 0, (0.09, 0.18, 2.0), (-0.455, 0.0, 1.0))
    append_prim_box(bm_f, 0, (0.09, 0.18, 2.0), (0.455, 0.0, 1.0))
    append_prim_box(bm_f, 0, (1.0, 0.20, 0.12), (0.0, 0.0, 1.94))
    append_prim_box(bm_f, 1, (0.98, 0.22, 0.03), (0.0, 0.0, 0.015))
    built["Prop_Door_Frame"] = finalize_obj("Prop_Door_Frame", bm_f, [pal["wood_cream"], pal["gold"]])

    # Door Leaf: local origin at hinge X = 0 (placed at x = -0.41 in Three.js) so width spans x in [0.0, 0.82]
    bm_l = bmesh.new()
    # Main painted door slab (tintable!) with upper window cutout built from stiles & rails
    append_prim_box(bm_l, 0, (0.82, 0.08, 1.12), (0.41, 0.0, 0.58))
    append_prim_box(bm_l, 0, (0.14, 0.08, 0.74), (0.07, 0.0, 1.50))
    append_prim_box(bm_l, 0, (0.14, 0.08, 0.74), (0.75, 0.0, 1.50))
    append_prim_box(bm_l, 0, (0.82, 0.08, 0.14), (0.41, 0.0, 1.81))
    # Decorative lower raised panel
    append_prim_box(bm_l, 1, (0.54, 0.105, 0.68), (0.41, 0.0, 0.56))
    # Upper glass lite + mullions
    append_prim_box(bm_l, 2, (0.54, 0.025, 0.60), (0.41, 0.0, 1.46))
    append_prim_box(bm_l, 1, (0.03, 0.09, 0.60), (0.41, 0.0, 1.46))
    append_prim_box(bm_l, 1, (0.54, 0.09, 0.03), (0.41, 0.0, 1.46))
    # Brass doorknobs on both sides + star rosette
    append_prim_cyl(bm_l, 3, 0.035, 0.035, 0.14, (0.70, 0.0, 0.96), rot_euler=(math.pi / 2, 0.0, 0.0))
    append_prim_sphere(bm_l, 3, 0.048, (0.70, -0.085, 0.96))
    append_prim_sphere(bm_l, 3, 0.048, (0.70, 0.085, 0.96))
    built["Prop_Door_Leaf"] = finalize_obj(
        "Prop_Door_Leaf",
        bm_l,
        [pal["tint"], pal["wood_cream"], pal["glass"], pal["gold"]],
    )

    # 3) Prop_Window_Frame + Prop_Window_Sash (1x1x1 casement window)
    bm_wf = bmesh.new()
    append_prim_box(bm_wf, 0, (1.0, 0.22, 0.09), (0.0, 0.0, 0.045))  # windowsill
    append_prim_box(bm_wf, 0, (1.0, 0.18, 0.08), (0.0, 0.0, 0.96))   # header
    append_prim_box(bm_wf, 0, (0.08, 0.18, 0.84), (-0.46, 0.0, 0.50))
    append_prim_box(bm_wf, 0, (0.08, 0.18, 0.84), (0.46, 0.0, 0.50))
    built["Prop_Window_Frame"] = finalize_obj("Prop_Window_Frame", bm_wf, [pal["wood_cream"]])

    # Window Sash: local origin at hinge X = 0 (placed at x = -0.42 in Three.js), width [0.0, 0.84], z in [0.09, 0.92]
    bm_ws = bmesh.new()
    append_prim_box(bm_ws, 0, (0.84, 0.06, 0.06), (0.42, 0.0, 0.12))
    append_prim_box(bm_ws, 0, (0.84, 0.06, 0.06), (0.42, 0.0, 0.89))
    append_prim_box(bm_ws, 0, (0.06, 0.06, 0.72), (0.03, 0.0, 0.505))
    append_prim_box(bm_ws, 0, (0.06, 0.06, 0.72), (0.81, 0.0, 0.505))
    # Cross mullions + latch
    append_prim_box(bm_ws, 1, (0.035, 0.065, 0.72), (0.42, 0.0, 0.505))
    append_prim_box(bm_ws, 1, (0.72, 0.065, 0.035), (0.42, 0.0, 0.505))
    append_prim_box(bm_ws, 2, (0.74, 0.02, 0.73), (0.42, 0.0, 0.505))
    append_prim_sphere(bm_ws, 3, 0.028, (0.79, -0.045, 0.505))
    built["Prop_Window_Sash"] = finalize_obj(
        "Prop_Window_Sash",
        bm_ws,
        [pal["tint"], pal["wood_cream"], pal["glass"], pal["gold"]],
    )

    # 4) Prop_Kitchen_Counter (1x1x1)
    bm = bmesh.new()
    append_prim_box(bm, 3, (0.94, 0.86, 0.10), (0.0, 0.02, 0.05))      # kickplate
    append_prim_box(bm, 0, (0.96, 0.90, 0.78), (0.0, 0.0, 0.49))       # cabinet body (tintable)
    append_prim_box(bm, 1, (1.00, 0.96, 0.09), (0.0, -0.01, 0.925))    # marble/quartz top
    append_prim_box(bm, 1, (1.00, 0.08, 0.08), (0.0, 0.44, 0.99))      # backsplash
    # Drawer & cabinet door fronts (front is -Y in Blender -> +Z in Three.js)
    append_prim_box(bm, 2, (0.84, 0.03, 0.18), (0.0, -0.455, 0.74))
    append_prim_box(bm, 2, (0.40, 0.03, 0.44), (-0.22, -0.455, 0.39))
    append_prim_box(bm, 2, (0.40, 0.03, 0.44), (0.22, -0.455, 0.39))
    # Brass handles
    append_prim_cyl(bm, 4, 0.016, 0.016, 0.22, (0.0, -0.48, 0.74), rot_euler=(0.0, math.pi / 2, 0.0))
    append_prim_cyl(bm, 4, 0.014, 0.014, 0.14, (-0.06, -0.48, 0.44))
    append_prim_cyl(bm, 4, 0.014, 0.014, 0.14, (0.06, -0.48, 0.44))
    built["Prop_Kitchen_Counter"] = finalize_obj(
        "Prop_Kitchen_Counter",
        bm,
        [pal["tint"], pal["marble"], pal["wood_cream"], pal["obsidian"], pal["gold"]],
    )

    # 5) Prop_Kitchen_Sink (1x1x1)
    bm = bmesh.new()
    append_prim_box(bm, 3, (0.94, 0.86, 0.10), (0.0, 0.02, 0.05))
    append_prim_box(bm, 0, (0.96, 0.90, 0.76), (0.0, 0.0, 0.48))
    # Counter rim around basin
    append_prim_box(bm, 1, (1.00, 0.18, 0.10), (0.0, -0.39, 0.91))
    append_prim_box(bm, 1, (1.00, 0.22, 0.10), (0.0, 0.37, 0.91))
    append_prim_box(bm, 1, (0.16, 0.56, 0.10), (-0.42, -0.01, 0.91))
    append_prim_box(bm, 1, (0.16, 0.56, 0.10), (0.42, -0.01, 0.91))
    # Stainless basin floor & shimmering water
    append_prim_box(bm, 2, (0.68, 0.56, 0.04), (0.0, -0.01, 0.76))
    append_prim_box(bm, 4, (0.66, 0.54, 0.05), (0.0, -0.01, 0.83))
    # Gooseneck chrome faucet + handles
    append_prim_cyl(bm, 2, 0.024, 0.024, 0.24, (0.0, 0.33, 1.06))
    append_prim_torus(bm, 2, 0.09, 0.022, (0.0, 0.24, 1.16), rot_euler=(0.0, math.pi / 2, 0.0))
    append_prim_cyl(bm, 2, 0.022, 0.022, 0.08, (-0.16, 0.33, 0.99))
    append_prim_cyl(bm, 2, 0.022, 0.022, 0.08, (0.16, 0.33, 0.99))
    # Front cabinet doors
    append_prim_box(bm, 5, (0.40, 0.03, 0.52), (-0.22, -0.455, 0.44))
    append_prim_box(bm, 5, (0.40, 0.03, 0.52), (0.22, -0.455, 0.44))
    built["Prop_Kitchen_Sink"] = finalize_obj(
        "Prop_Kitchen_Sink",
        bm,
        [pal["tint"], pal["marble"], pal["chrome"], pal["obsidian"], pal["water"], pal["wood_cream"]],
    )

    # 6) Prop_Kitchen_Stove (1x1x1)
    bm = bmesh.new()
    append_prim_box(bm, 0, (0.96, 0.90, 0.88), (0.0, 0.0, 0.44))
    append_prim_box(bm, 1, (0.98, 0.92, 0.06), (0.0, 0.0, 0.91))       # dark cooktop
    append_prim_box(bm, 0, (0.98, 0.10, 0.22), (0.0, 0.41, 1.03))      # back guard
    # 4 Glowing burner coils
    for bx in (-0.24, 0.24):
        for by in (-0.20, 0.16):
            append_prim_cyl(bm, 1, 0.14, 0.14, 0.02, (bx, by, 0.95))
            append_prim_torus(bm, 2, 0.095, 0.016, (bx, by, 0.965))
            append_prim_torus(bm, 2, 0.048, 0.013, (bx, by, 0.968))
    # Front oven door + glowing window + handle + 4 knobs
    append_prim_box(bm, 3, (0.82, 0.04, 0.56), (0.0, -0.46, 0.42))
    append_prim_box(bm, 4, (0.58, 0.05, 0.30), (0.0, -0.465, 0.40))
    append_prim_cyl(bm, 3, 0.018, 0.018, 0.72, (0.0, -0.50, 0.65), rot_euler=(0.0, math.pi / 2, 0.0))
    for kx in (-0.30, -0.10, 0.10, 0.30):
        append_prim_cyl(bm, 3, 0.032, 0.032, 0.04, (kx, -0.47, 0.81), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Kitchen_Stove"] = finalize_obj(
        "Prop_Kitchen_Stove",
        bm,
        [pal["tint"], pal["obsidian"], pal["burner"], pal["chrome"], pal["lamp_glow"]],
    )

    # 7) Prop_Kitchen_Fridge (1x2x1 Retro Smeg-style 2-door fridge)
    bm = bmesh.new()
    append_prim_box(bm, 0, (0.92, 0.88, 1.92), (0.0, 0.02, 0.98))
    # Rounded upper freezer door & lower fridge door
    append_prim_box(bm, 1, (0.88, 0.07, 0.54), (0.0, -0.44, 1.63))
    append_prim_box(bm, 1, (0.88, 0.07, 1.24), (0.0, -0.44, 0.68))
    # Chrome retro handles & star emblem
    append_prim_cyl(bm, 2, 0.020, 0.020, 0.28, (-0.32, -0.50, 1.60))
    append_prim_cyl(bm, 2, 0.020, 0.020, 0.46, (-0.32, -0.50, 0.96))
    append_prim_star(bm, 3, 0.075, 0.035, 0.025, (0.0, -0.485, 1.72), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Kitchen_Fridge"] = finalize_obj(
        "Prop_Kitchen_Fridge",
        bm,
        [pal["tint"], pal["porcelain"], pal["chrome"], pal["gold"]],
    )

    # 8) Prop_Dining_Table (1x1x1 from CC0 `dining_table.blend`: wood base + tintable tablecloth)
    bm_tbl = load_cc0_bmesh(
        "dining_table.blend",
        target_bounds=(0.96, 0.96, 0.85),
        mat_map={"dining_table": 0, "dining_table_cloth": 1},
    )
    if bm_tbl is None:
        bm_tbl = bmesh.new()
        append_prim_box(bm_tbl, 1, (0.96, 0.96, 0.08), (0.0, 0.0, 0.81))
        for lx in (-0.38, 0.38):
            for ly in (-0.38, 0.38):
                append_prim_cyl(bm_tbl, 0, 0.04, 0.03, 0.78, (lx, ly, 0.39))
    built["Prop_Dining_Table"] = finalize_obj(
        "Prop_Dining_Table",
        bm_tbl,
        [pal["wood_warm"], pal["tint"]],
    )

    # 9) Prop_Chair (1x1x1 from CC0 `painted_wooden_chair_01.blend` + plush tintable seat pad)
    bm_ch = load_cc0_bmesh(
        "painted_wooden_chair_01.blend",
        target_bounds=(0.62, 0.64, 0.96),
        mat_map={"painted_wooden_chair_01": 0},
    )
    if bm_ch is None:
        bm_ch = bmesh.new()
        append_prim_box(bm_ch, 0, (0.56, 0.56, 0.06), (0.0, 0.0, 0.46))
    append_prim_box(bm_ch, 1, (0.52, 0.50, 0.07), (0.0, -0.02, 0.50))
    built["Prop_Chair"] = finalize_obj("Prop_Chair", bm_ch, [pal["wood_cream"], pal["tint"]])

    # 10) Prop_Bath_Tub (1x1x1 Clawfoot bubble-bath tub)
    bm = bmesh.new()
    # Outer tub shell + inner water + rim
    append_prim_box(bm, 0, (0.98, 0.82, 0.54), (0.0, 0.0, 0.36))
    append_prim_torus(bm, 1, 0.42, 0.055, (0.0, 0.0, 0.63), maj_seg=24, min_seg=10)
    append_prim_box(bm, 2, (0.84, 0.68, 0.06), (0.0, 0.0, 0.56))
    # Fluffy bath bubbles!
    for bx, by, br in ((-0.24, -0.14, 0.09), (-0.12, 0.16, 0.11), (0.18, -0.10, 0.10), (0.25, 0.15, 0.085), (0.0, 0.0, 0.11)):
        append_prim_sphere(bm, 3, br, (bx, by, 0.60))
    # 4 Gold clawfeet + chrome mixer tap
    for fx in (-0.38, 0.38):
        for fy in (-0.30, 0.30):
            append_prim_sphere(bm, 4, 0.065, (fx, fy, 0.065))
    append_prim_cyl(bm, 5, 0.025, 0.025, 0.24, (0.0, 0.36, 0.74))
    append_prim_cyl(bm, 5, 0.022, 0.022, 0.14, (0.0, 0.28, 0.84), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Bath_Tub"] = finalize_obj(
        "Prop_Bath_Tub",
        bm,
        [pal["porcelain"], pal["tint"], pal["water"], pal["fabric_white"], pal["gold"], pal["chrome"]],
    )

    # 11) Prop_Bath_Vanity (1x1.8x1 Sink cabinet + backlit glowing vanity mirror)
    bm = bmesh.new()
    append_prim_box(bm, 0, (0.90, 0.72, 0.78), (0.0, 0.06, 0.39))
    append_prim_box(bm, 1, (0.94, 0.78, 0.08), (0.0, 0.05, 0.82))
    append_prim_cyl(bm, 1, 0.28, 0.22, 0.12, (0.0, 0.02, 0.90))
    append_prim_cyl(bm, 2, 0.24, 0.24, 0.03, (0.0, 0.02, 0.95))
    append_prim_cyl(bm, 3, 0.022, 0.022, 0.18, (0.0, 0.28, 1.02))
    # Framed arched mirror with LED halo
    append_prim_box(bm, 4, (0.68, 0.05, 0.76), (0.0, 0.42, 1.38))
    append_prim_box(bm, 5, (0.58, 0.06, 0.66), (0.0, 0.405, 1.38))
    built["Prop_Bath_Vanity"] = finalize_obj(
        "Prop_Bath_Vanity",
        bm,
        [pal["tint"], pal["porcelain"], pal["water"], pal["chrome"], pal["gold"], pal["mirror"]],
    )

    # 12) Prop_Bath_Toilet (1x1x1 Porcelain toilet)
    bm = bmesh.new()
    append_prim_cyl(bm, 0, 0.22, 0.18, 0.38, (0.0, -0.06, 0.19))
    append_prim_sphere(bm, 0, 0.28, (0.0, -0.10, 0.40), scale=(0.95, 1.18, 0.62))
    append_prim_torus(bm, 1, 0.20, 0.045, (0.0, -0.10, 0.48), maj_seg=20, min_seg=8)
    append_prim_box(bm, 0, (0.48, 0.22, 0.46), (0.0, 0.24, 0.68))
    append_prim_box(bm, 0, (0.52, 0.25, 0.06), (0.0, 0.24, 0.93))
    append_prim_cyl(bm, 2, 0.018, 0.018, 0.10, (-0.22, 0.12, 0.84), rot_euler=(0.0, math.pi / 2, 0.0))
    built["Prop_Bath_Toilet"] = finalize_obj(
        "Prop_Bath_Toilet",
        bm,
        [pal["porcelain"], pal["tint"], pal["chrome"]],
    )

    # 13) Prop_Bed (1x1x1 Cozy Starlight Bed from CC0 `vintage_day_bed.blend` + mattress, pillow, blanket)
    bm_bed = load_cc0_bmesh(
        "vintage_day_bed.blend",
        target_bounds=(0.96, 0.98, 0.88),
        mat_map={"vintage_day_bed": 0},
    )
    if bm_bed is None:
        bm_bed = bmesh.new()
        append_prim_box(bm_bed, 0, (0.96, 0.10, 0.88), (0.0, 0.44, 0.44))
    # Plush mattress, tintable quilted blanket, and fluffy pillow
    append_prim_box(bm_bed, 1, (0.86, 0.86, 0.20), (0.0, -0.02, 0.36))
    append_prim_box(bm_bed, 2, (0.89, 0.60, 0.22), (0.0, -0.16, 0.38))
    append_prim_box(bm_bed, 1, (0.64, 0.24, 0.12), (0.0, 0.28, 0.51), rot_euler=(0.18, 0.0, 0.0))
    append_prim_star(bm_bed, 3, 0.10, 0.045, 0.025, (0.0, 0.42, 0.74), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Bed"] = finalize_obj(
        "Prop_Bed",
        bm_bed,
        [pal["wood_warm"], pal["fabric_white"], pal["tint"], pal["gold"]],
    )

    # 14) Prop_Nightstand_Lamp (1x1x1 CC0 `painted_wooden_nightstand.blend` + warm starlight lamp)
    bm_ns = load_cc0_bmesh(
        "painted_wooden_nightstand.blend",
        target_bounds=(0.68, 0.68, 0.60),
        mat_map={"painted_wooden_nightstand": 0, "painted_wooden_nightstand_drawer": 1},
    )
    if bm_ns is None:
        bm_ns = bmesh.new()
        append_prim_box(bm_ns, 0, (0.66, 0.66, 0.60), (0.0, 0.0, 0.30))
    # Lamp base, stem, and warm emissive lampshade
    append_prim_cyl(bm_ns, 2, 0.11, 0.09, 0.05, (0.0, 0.0, 0.625))
    append_prim_cyl(bm_ns, 2, 0.024, 0.024, 0.22, (0.0, 0.0, 0.74))
    append_prim_cyl(bm_ns, 3, 0.22, 0.13, 0.24, (0.0, 0.0, 0.92))
    append_prim_star(bm_ns, 2, 0.055, 0.025, 0.02, (0.0, 0.0, 1.07), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Nightstand_Lamp"] = finalize_obj(
        "Prop_Nightstand_Lamp",
        bm_ns,
        [pal["wood_cream"], pal["tint"], pal["gold"], pal["lamp_glow"]],
    )

    # 15) Prop_Wardrobe (1x2x1 CC0 `painted_wooden_cabinet_02.blend` normalized to 1.96m)
    bm_wd = load_cc0_bmesh(
        "painted_wooden_cabinet_02.blend",
        target_bounds=(0.94, 0.76, 1.96),
        mat_map={
            "painted_wooden_cabinet_02": 0,
            "painted_wooden_cabinet_02_door": 1,
            "painted_wooden_cabinet_02_drawer": 1,
        },
    )
    if bm_wd is None:
        bm_wd = bmesh.new()
        append_prim_box(bm_wd, 0, (0.94, 0.76, 1.96), (0.0, 0.0, 0.98))
    append_prim_star(bm_wd, 2, 0.09, 0.04, 0.03, (0.0, -0.39, 1.55), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Wardrobe"] = finalize_obj(
        "Prop_Wardrobe",
        bm_wd,
        [pal["wood_warm"], pal["tint"], pal["gold"]],
    )

    # 16) Prop_Rug (1x0.06x1 Plush woven floor rug)
    bm = bmesh.new()
    append_prim_box(bm, 1, (0.98, 0.98, 0.03), (0.0, 0.0, 0.015))
    append_prim_box(bm, 0, (0.84, 0.84, 0.045), (0.0, 0.0, 0.024))
    append_prim_star(bm, 2, 0.24, 0.11, 0.02, (0.0, 0.0, 0.048))
    built["Prop_Rug"] = finalize_obj("Prop_Rug", bm, [pal["tint"], pal["wood_cream"], pal["gold"]])

    # ============================================================================
    # 4. Build Legendary Starlight Ray Gun & New Monster / Chest 3D Loot
    # ============================================================================

    # 17) Loot_Ray_Gun — FIRST LEGENDARY ITEM!
    # Grip centered near (0, 0, 0.12), barrel extending forward along -Y in Blender (+Z in Three.js)
    bm = bmesh.new()
    # Ergonomic angled grip + pommel cap
    append_prim_box(bm, 1, (0.085, 0.11, 0.26), (0.0, 0.06, 0.13), rot_euler=(-0.28, 0.0, 0.0))
    append_prim_cyl(bm, 2, 0.055, 0.045, 0.04, (0.0, 0.095, 0.01), rot_euler=(-0.28, 0.0, 0.0))
    # Trigger guard
    append_prim_torus(bm, 3, 0.055, 0.012, (0.0, -0.02, 0.18), rot_euler=(0.0, math.pi / 2, 0.0))
    # Sleek retro-futuristic anodized receiver body
    append_prim_cyl(bm, 0, 0.102, 0.074, 0.38, (0.0, -0.04, 0.29), rot_euler=(math.pi / 2, 0.0, 0.0))
    # Glowing cyan plasma core chamber band visibly wrapping middle of receiver
    append_prim_cyl(bm, 4, 0.112, 0.106, 0.11, (0.0, -0.03, 0.29), rot_euler=(math.pi / 2, 0.0, 0.0))
    # Swept golden dorsal fin & rear power dial
    append_prim_box(bm, 2, (0.026, 0.26, 0.11), (0.0, 0.01, 0.40), rot_euler=(0.22, 0.0, 0.0))
    append_prim_sphere(bm, 4, 0.058, (0.0, 0.16, 0.29))
    # Tapered emitter barrel + 3 concentric chrome/gold Tesla rings + glowing star muzzle
    append_prim_cyl(bm, 3, 0.044, 0.026, 0.32, (0.0, -0.36, 0.29), rot_euler=(math.pi / 2, 0.0, 0.0))
    for ry, rr in ((-0.26, 0.098), (-0.36, 0.080), (-0.45, 0.064)):
        append_prim_torus(bm, 2, rr, 0.014, (0.0, ry, 0.29), rot_euler=(math.pi / 2, 0.0, 0.0))
    append_prim_sphere(bm, 4, 0.050, (0.0, -0.53, 0.29))
    append_prim_star(bm, 2, 0.068, 0.030, 0.020, (0.0, -0.18, 0.42))
    built["Loot_Ray_Gun"] = finalize_obj(
        "Loot_Ray_Gun",
        bm,
        [pal["raygun_body"], pal["obsidian"], pal["gold"], pal["chrome"], pal["raygun_plasma"]],
    )

    # 18) Loot_Geode_Lamp (Amethyst Geode Lamp — split volcanic rock shell + glowing quartz cluster)
    bm = bmesh.new()
    append_prim_cyl(bm, 0, 0.28, 0.24, 0.07, (0.0, 0.0, 0.035))  # wood base
    append_prim_sphere(bm, 1, 0.29, (0.0, 0.11, 0.36), u_seg=12, v_seg=10, scale=(1.0, 0.58, 1.18), smooth=False)
    # Cluster of glowing amethyst crystal spires bursting out the hollow front cavity
    spires = [
        ((0.0, -0.10, 0.34), 0.105, 0.38, (0.42, 0.0, 0.0)),
        ((-0.11, -0.07, 0.30), 0.085, 0.32, (0.36, -0.30, 0.1)),
        ((0.11, -0.07, 0.31), 0.085, 0.33, (0.36, 0.30, -0.1)),
        ((-0.06, -0.08, 0.45), 0.075, 0.28, (0.24, -0.18, 0.0)),
        ((0.07, -0.08, 0.44), 0.075, 0.28, (0.24, 0.18, 0.0)),
        ((0.0, -0.13, 0.22), 0.070, 0.24, (0.58, 0.0, 0.0)),
    ]
    for pos, rad, h, rot in spires:
        append_prim_cyl(bm, 2, rad, 0.008, h, pos, segments=6, rot_euler=rot, smooth=False)
    built["Loot_Geode_Lamp"] = finalize_obj(
        "Loot_Geode_Lamp",
        bm,
        [pal["wood_dark"], pal["basalt"], pal["amethyst"]],
    )

    # 19) Loot_Slime_Lamp (Glow-Slime Terrarium Jar)
    bm = bmesh.new()
    append_prim_cyl(bm, 0, 0.26, 0.24, 0.06, (0.0, 0.0, 0.03))             # brass base
    append_prim_cyl(bm, 1, 0.23, 0.23, 0.50, (0.0, 0.0, 0.31), segments=20) # glass jar
    append_prim_cyl(bm, 2, 0.17, 0.19, 0.10, (0.0, 0.0, 0.59))             # cork stopper
    # Cute bouncy glowing slime inside + two button eyes!
    append_prim_sphere(bm, 3, 0.17, (0.0, 0.0, 0.22), scale=(1.08, 1.08, 0.88))
    append_prim_sphere(bm, 4, 0.026, (-0.055, -0.145, 0.25))
    append_prim_sphere(bm, 4, 0.026, (0.055, -0.145, 0.25))
    built["Loot_Slime_Lamp"] = finalize_obj(
        "Loot_Slime_Lamp",
        bm,
        [pal["gold"], pal["glass"], pal["wood_warm"], pal["slime_glow"], pal["obsidian"]],
    )

    # 20) Loot_Bat_Plush (Chibi Crystal Bat Plushie)
    bm = bmesh.new()
    # Chubby sitting plush body + head
    append_prim_sphere(bm, 0, 0.20, (0.0, 0.0, 0.20), scale=(1.0, 0.92, 1.05))
    append_prim_sphere(bm, 0, 0.17, (0.0, -0.02, 0.47))
    # Pink belly patch + oversized bat ears + cute feet
    append_prim_sphere(bm, 1, 0.12, (0.0, -0.11, 0.20), scale=(0.95, 0.45, 1.0))
    append_prim_cyl(bm, 0, 0.085, 0.015, 0.22, (-0.12, -0.01, 0.66), segments=10, rot_euler=(0.0, -0.28, 0.0))
    append_prim_cyl(bm, 0, 0.085, 0.015, 0.22, (0.12, -0.01, 0.66), segments=10, rot_euler=(0.0, 0.28, 0.0))
    append_prim_cyl(bm, 1, 0.05, 0.01, 0.16, (-0.115, -0.03, 0.65), segments=8, rot_euler=(0.0, -0.28, 0.0))
    append_prim_cyl(bm, 1, 0.05, 0.01, 0.16, (0.115, -0.03, 0.65), segments=8, rot_euler=(0.0, 0.28, 0.0))
    # Little crystal wings + button eyes + star nose
    append_prim_box(bm, 2, (0.24, 0.04, 0.18), (-0.26, 0.04, 0.28), rot_euler=(0.0, 0.0, 0.25))
    append_prim_box(bm, 2, (0.24, 0.04, 0.18), (0.26, 0.04, 0.28), rot_euler=(0.0, 0.0, -0.25))
    append_prim_sphere(bm, 3, 0.032, (-0.065, -0.165, 0.49))
    append_prim_sphere(bm, 3, 0.032, (0.065, -0.165, 0.49))
    append_prim_sphere(bm, 0, 0.065, (-0.10, -0.14, 0.06))
    append_prim_sphere(bm, 0, 0.065, (0.10, -0.14, 0.06))
    built["Loot_Bat_Plush"] = finalize_obj(
        "Loot_Bat_Plush",
        bm,
        [pal["plush_lavender"], pal["plush_pink"], pal["amethyst"], pal["obsidian"]],
    )

    # 21) Loot_Royal_Chalice (CC0 `brass_goblet_01` normalized + ruby & cyan star gems!)
    bm_ch = load_cc0_bmesh(
        "brass_goblets.blend",
        mesh_names=["brass_goblet_01"],
        target_bounds=(0.38, 0.38, 0.66),
        mat_map={"brass_goblet_01": 0},
    )
    if bm_ch is None:
        bm_ch = bmesh.new()
        append_prim_cyl(bm_ch, 0, 0.16, 0.05, 0.25, (0.0, 0.0, 0.12))
        append_prim_cyl(bm_ch, 0, 0.06, 0.19, 0.34, (0.0, 0.0, 0.48))
    # Ring of glowing rubies & starlight gems around chalice cup
    for i in range(6):
        ang = (2.0 * math.pi * i) / 6.0
        gx, gy = math.cos(ang) * 0.185, math.sin(ang) * 0.185
        gem_mat = 1 if i % 2 == 0 else 2
        append_prim_sphere(bm_ch, gem_mat, 0.028, (gx, gy, 0.50))
    built["Loot_Royal_Chalice"] = finalize_obj(
        "Loot_Royal_Chalice",
        bm_ch,
        [pal["gold"], pal["ruby_gem"], pal["raygun_plasma"]],
    )

    # 22) Loot_Dragon_Egg (Prismatic Dragon Egg on Royal Velvet Cushion)
    bm = bmesh.new()
    append_prim_box(bm, 0, (0.62, 0.62, 0.14), (0.0, 0.0, 0.07))
    for cx in (-0.30, 0.30):
        for cy in (-0.30, 0.30):
            append_prim_sphere(bm, 1, 0.055, (cx, cy, 0.08))
    append_prim_sphere(bm, 2, 0.24, (0.0, 0.0, 0.40), u_seg=20, v_seg=16, scale=(1.0, 1.0, 1.32))
    append_prim_torus(bm, 1, 0.21, 0.022, (0.0, 0.0, 0.34))
    built["Loot_Dragon_Egg"] = finalize_obj(
        "Loot_Dragon_Egg",
        bm,
        [pal["velvet_crimson"], pal["gold"], pal["dragon_scale"]],
    )

    # 23) Loot_Display_Pedestal (Marble & Gold Holo-Stand for displaying any Backpack relic!)
    bm = bmesh.new()
    append_prim_cyl(bm, 0, 0.38, 0.34, 0.08, (0.0, 0.0, 0.04), segments=8)
    append_prim_cyl(bm, 0, 0.24, 0.28, 0.20, (0.0, 0.0, 0.18), segments=8)
    append_prim_cyl(bm, 1, 0.32, 0.30, 0.05, (0.0, 0.0, 0.305), segments=8)
    append_prim_cyl(bm, 2, 0.24, 0.24, 0.02, (0.0, 0.0, 0.34), segments=16)
    built["Loot_Display_Pedestal"] = finalize_obj(
        "Loot_Display_Pedestal",
        bm,
        [pal["marble"], pal["gold"], pal["raygun_plasma"]],
    )

    # 24) Prop_Sofa (Cozy plush velvet loveseat sofa facing -Y -> +Z in Three.js)
    bm_sofa = load_cc0_bmesh(
        "GreenChair_01.blend",
        target_bounds=(0.94, 0.78, 0.84),
        mat_map={"GreenChair_01": 0},
    )
    if bm_sofa is None:
        bm_sofa = bmesh.new()
    else:
        bm_sofa.free()
        bm_sofa = bmesh.new()
    # Sculpted 2-cushion loveseat within 1x1 footprint (or wide plush profile) with wooden feet & gold stars
    for lx in (-0.40, 0.40):
        for ly in (-0.30, 0.30):
            append_prim_cyl(bm_sofa, 1, 0.038, 0.026, 0.14, (lx, ly, 0.07), segments=10)
    # Base frame, plush backrest (+Y), rolled armrests (±X), and 2 plump seat cushions
    append_prim_box(bm_sofa, 0, (0.94, 0.76, 0.22), (0.0, 0.0, 0.23))
    append_prim_box(bm_sofa, 0, (0.94, 0.22, 0.54), (0.0, 0.28, 0.56), rot_euler=(-0.08, 0.0, 0.0))
    for sx in (-0.41, 0.41):
        append_prim_box(bm_sofa, 0, (0.16, 0.76, 0.34), (sx, 0.0, 0.44))
        append_prim_cyl(bm_sofa, 0, 0.085, 0.085, 0.76, (sx, 0.0, 0.60), rot_euler=(math.pi / 2, 0.0, 0.0))
    for cx in (-0.19, 0.19):
        append_prim_box(bm_sofa, 2, (0.36, 0.52, 0.13), (cx, -0.06, 0.38))
    # Accent throw pillow + gold star button tufts
    append_prim_box(bm_sofa, 2, (0.24, 0.12, 0.24), (-0.22, 0.14, 0.54), rot_euler=(0.22, 0.0, 0.18))
    append_prim_star(bm_sofa, 3, 0.055, 0.025, 0.02, (0.18, 0.16, 0.62), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Sofa"] = finalize_obj(
        "Prop_Sofa",
        bm_sofa,
        [pal["tint"], pal["wood_warm"], pal["fabric_white"], pal["gold"]],
    )

    # 25) Prop_Crystal_Lamp (Tall Starlight Floor Lamp with toggleable warm glow shade)
    bm_fl = bmesh.new()
    append_prim_cyl(bm_fl, 1, 0.24, 0.20, 0.06, (0.0, 0.0, 0.03), segments=18)
    append_prim_cyl(bm_fl, 0, 0.07, 0.05, 0.18, (0.0, 0.0, 0.14), segments=12)
    append_prim_cyl(bm_fl, 1, 0.028, 0.024, 1.08, (0.0, 0.0, 0.74), segments=12)
    append_prim_sphere(bm_fl, 3, 0.065, (0.0, 0.0, 0.88))
    # Flared pleated starlight shade + glowing bulb core + star finial
    append_prim_cyl(bm_fl, 2, 0.28, 0.16, 0.34, (0.0, 0.0, 1.36), segments=18)
    append_prim_torus(bm_fl, 1, 0.28, 0.014, (0.0, 0.0, 1.19))
    append_prim_torus(bm_fl, 1, 0.16, 0.014, (0.0, 0.0, 1.53))
    append_prim_star(bm_fl, 1, 0.075, 0.032, 0.022, (0.0, 0.0, 1.62), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Prop_Crystal_Lamp"] = finalize_obj(
        "Prop_Crystal_Lamp",
        bm_fl,
        [pal["tint"], pal["gold"], pal["lamp_glow"], pal["amethyst"]],
    )

    # 26) Loot_Crystal_Blade — Crafted Starlight Crystal Blade (Right-hand weapon & pedestal relic)
    # Hilt grip near (0, 0.02, 0.14), faceted crystal blade extending forward along -Y & upward (+Z)
    bm_sw = bmesh.new()
    append_prim_cyl(bm_sw, 1, 0.028, 0.026, 0.24, (0.0, 0.06, 0.12), segments=12, rot_euler=(0.65, 0.0, 0.0))
    append_prim_sphere(bm_sw, 0, 0.052, (0.0, 0.14, 0.03))
    # Winged golden crossguard + central starlight gem
    append_prim_box(bm_sw, 0, (0.34, 0.055, 0.055), (0.0, -0.02, 0.22), rot_euler=(0.65, 0.0, 0.0))
    append_prim_sphere(bm_sw, 3, 0.055, (0.0, -0.02, 0.22))
    # Faceted dual-tone crystal blade extending forward-up along (-Y, +Z)
    append_prim_box(bm_sw, 2, (0.095, 0.032, 0.68), (0.0, -0.24, 0.48), rot_euler=(0.65, 0.0, 0.0))
    append_prim_cyl(bm_sw, 3, 0.058, 0.004, 0.26, (0.0, -0.49, 0.79), segments=4, rot_euler=(-0.92, 0.0, 0.0), smooth=False)
    built["Loot_Crystal_Blade"] = finalize_obj(
        "Loot_Crystal_Blade",
        bm_sw,
        [pal["gold"], pal["obsidian"], pal["raygun_plasma"], pal["amethyst"]],
    )

    # 27) Loot_Starlight_Wand — Crafted Astral Starlight Wand (Right-hand weapon & pedestal relic)
    bm_wd2 = bmesh.new()
    append_prim_cyl(bm_wd2, 0, 0.024, 0.018, 0.56, (0.0, -0.10, 0.26), segments=14, rot_euler=(0.58, 0.0, 0.0))
    append_prim_sphere(bm_wd2, 2, 0.042, (0.0, 0.06, 0.04))
    append_prim_torus(bm_wd2, 0, 0.11, 0.018, (0.0, -0.28, 0.52), rot_euler=(0.58, math.pi / 2, 0.0))
    # Wings flanking wand head + glowing 5-point star core
    for sx in (-1.0, 1.0):
        append_prim_box(bm_wd2, 3, (0.18, 0.025, 0.08), (sx * 0.16, -0.28, 0.53), rot_euler=(0.4, 0.0, -sx * 0.38))
    append_prim_star(bm_wd2, 1, 0.12, 0.052, 0.038, (0.0, -0.30, 0.55), rot_euler=(0.58 + math.pi / 2, 0.0, 0.0))
    built["Loot_Starlight_Wand"] = finalize_obj(
        "Loot_Starlight_Wand",
        bm_wd2,
        [pal["gold"], pal["lamp_glow"], pal["ruby_gem"], pal["raygun_plasma"]],
    )

    # 28) Loot_Aegis_Charm — Crafted Aegis Heart Charm (100 -> 150 Max HP in Shadow Rift!)
    bm_ac = bmesh.new()
    append_prim_cyl(bm_ac, 0, 0.16, 0.14, 0.05, (0.0, 0.0, 0.025), segments=12)
    # Sculpted golden shield-ring + twin ruby heart lobes + bottom point
    append_prim_torus(bm_ac, 0, 0.22, 0.026, (0.0, 0.0, 0.34), rot_euler=(math.pi / 2, 0.0, 0.0))
    append_prim_sphere(bm_ac, 1, 0.115, (-0.085, 0.0, 0.40), scale=(1.0, 0.58, 1.0))
    append_prim_sphere(bm_ac, 1, 0.115, (0.085, 0.0, 0.40), scale=(1.0, 0.58, 1.0))
    append_prim_cyl(bm_ac, 1, 0.145, 0.008, 0.24, (0.0, 0.0, 0.26), segments=12, rot_euler=(math.pi, 0.0, 0.0))
    append_prim_star(bm_ac, 2, 0.065, 0.028, 0.03, (0.0, -0.055, 0.36), rot_euler=(math.pi / 2, 0.0, 0.0))
    built["Loot_Aegis_Charm"] = finalize_obj(
        "Loot_Aegis_Charm",
        bm_ac,
        [pal["gold"], pal["ruby_gem"], pal["raygun_plasma"]],
    )

    # 29) Loot_Rift_Beacon — Crafted Obsidian Rift Beacon (Summons Co-Op Obsidian Dragon Boss!)
    bm_rb = bmesh.new()
    append_prim_cyl(bm_rb, 0, 0.22, 0.26, 0.08, (0.0, 0.0, 0.04), segments=6, smooth=False)
    append_prim_torus(bm_rb, 1, 0.22, 0.022, (0.0, 0.0, 0.36), rot_euler=(0.65, 0.35, 0.0))
    append_prim_torus(bm_rb, 3, 0.19, 0.018, (0.0, 0.0, 0.36), rot_euler=(-0.55, 0.55, 0.0))
    append_prim_cyl(bm_rb, 2, 0.14, 0.01, 0.24, (0.0, 0.0, 0.48), segments=5, smooth=False)
    append_prim_cyl(bm_rb, 2, 0.14, 0.01, 0.24, (0.0, 0.0, 0.24), segments=5, rot_euler=(math.pi, 0.0, 0.0), smooth=False)
    built["Loot_Rift_Beacon"] = finalize_obj(
        "Loot_Rift_Beacon",
        bm_rb,
        [pal["obsidian"], pal["gold"], pal["burner"], pal["amethyst"]],
    )

    # 30) Loot_Pet_Egg — Hatchable Starlight Companion Egg
    bm_pe = bmesh.new()
    append_prim_torus(bm_pe, 0, 0.20, 0.055, (0.0, 0.0, 0.055), maj_seg=18, min_seg=10)
    append_prim_sphere(bm_pe, 1, 0.21, (0.0, 0.0, 0.30), u_seg=20, v_seg=16, scale=(1.0, 1.0, 1.30))
    for ang_i in range(5):
        ang = ang_i * (math.tau / 5.0)
        append_prim_star(
            bm_pe,
            2,
            0.055,
            0.025,
            0.018,
            (math.cos(ang) * 0.195, math.sin(ang) * 0.195, 0.31),
            rot_euler=(math.pi / 2, 0.0, ang + math.pi / 2),
        )
    append_prim_torus(bm_pe, 2, 0.14, 0.016, (0.0, 0.0, 0.52), rot_euler=(0.15, 0.0, 0.0))
    built["Loot_Pet_Egg"] = finalize_obj(
        "Loot_Pet_Egg",
        bm_pe,
        [pal["plush_pink"], pal["raygun_plasma"], pal["gold"]],
    )

    # 31) Loot_Aurora_Lamp — Rare Crafted Glowing Aurora Crystal Table/Pedestal Lamp
    bm_al = bmesh.new()
    append_prim_cyl(bm_al, 0, 0.22, 0.18, 0.07, (0.0, 0.0, 0.035), segments=8)
    append_prim_cyl(bm_al, 0, 0.04, 0.03, 0.38, (0.0, 0.0, 0.24), segments=10)
    append_prim_torus(bm_al, 0, 0.22, 0.020, (0.0, 0.0, 0.46))
    for i in range(6):
        ang = i * (math.tau / 6.0)
        cx, cy = math.cos(ang) * 0.21, math.sin(ang) * 0.21
        mat_slot = 1 if i % 2 == 0 else 2
        append_prim_cyl(bm_al, mat_slot, 0.045, 0.008, 0.22, (cx, cy, 0.35), segments=6, rot_euler=(math.pi, 0.0, 0.0), smooth=False)
    append_prim_sphere(bm_al, 3, 0.12, (0.0, 0.0, 0.48))
    built["Loot_Aurora_Lamp"] = finalize_obj(
        "Loot_Aurora_Lamp",
        bm_al,
        [pal["gold"], pal["raygun_plasma"], pal["amethyst"], pal["lamp_glow"]],
    )

    return built


# ============================================================================
# 5. Export Master `.blend`, Export `.glb`, and Render EEVEE Studio Previews
# ============================================================================

def setup_studio_render(cam_loc, look_at, out_path):
    scene = bpy.context.scene
    avail_engines = {e.identifier for e in scene.render.bl_rna.properties["engine"].enum_items}
    scene.render.engine = "BLENDER_EEVEE_NEXT" if "BLENDER_EEVEE_NEXT" in avail_engines else "BLENDER_EEVEE"
    scene.render.resolution_x = 1280
    scene.render.resolution_y = 720
    scene.render.filepath = out_path

    # Key, Fill, Rim lights
    lights = []
    for lname, lpos, energy, col in [
        ("KeyLight", (4.5, -6.5, 6.8), 650.0, (1.0, 0.96, 0.90)),
        ("FillLight", (-5.5, -4.5, 4.2), 320.0, (0.78, 0.88, 1.0)),
        ("RimLight", (0.0, 5.5, 5.0), 420.0, (1.0, 0.72, 0.92)),
    ]:
        ld = bpy.data.lights.new(name=lname, type="AREA")
        ld.energy = energy
        ld.color = col
        ld.size = 4.5
        lo = bpy.data.objects.new(lname, ld)
        scene.collection.objects.link(lo)
        lo.location = Vector(lpos)
        direction = Vector(look_at) - Vector(lpos)
        lo.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
        lights.append(lo)

    cam_d = bpy.data.cameras.new("PreviewCam")
    cam_d.lens = 28.0
    cam_o = bpy.data.objects.new("PreviewCam", cam_d)
    scene.collection.objects.link(cam_o)
    cam_o.location = Vector(cam_loc)
    cam_o.rotation_euler = (Vector(look_at) - Vector(cam_loc)).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam_o

    bpy.ops.render.render(write_still=True)

    for lo in lights:
        d = lo.data
        bpy.data.objects.remove(lo, do_unlink=True)
        bpy.data.lights.remove(d)
    bpy.data.objects.remove(cam_o, do_unlink=True)
    bpy.data.cameras.remove(cam_d)


def main():
    reset_scene()
    pal = build_palette()
    built = build_all_assets(pal)

    # Ensure every object sits at (0, 0, 0) for clean Three.js `.clone()` instancing before .glb export
    for obj in built.values():
        obj.location = Vector((0.0, 0.0, 0.0))

    total_verts = sum(len(o.data.vertices) for o in built.values())
    total_polys = sum(len(o.data.polygons) for o in built.values())
    print(f"Built {len(built)} prop/loot meshes: {total_verts} verts, {total_polys} polys")

    # 1. Save master `.blend` & export `.glb`
    bpy.ops.wm.save_as_mainfile(filepath=OUT_BLEND)
    bpy.ops.export_scene.gltf(
        filepath=OUT_GLB,
        export_format="GLB",
        export_yup=True,
        export_apply=True,
        export_materials="EXPORT",
    )
    print(f"Exported GLB: {OUT_GLB} ({os.path.getsize(OUT_GLB)} bytes)")

    # 2. Arrange Furniture Showcase Grid & Render EEVEE Studio Sheet 1
    for obj in built.values():
        obj.hide_render = True

    furniture_order = [
        "Prop_Stairs",
        "Prop_Door_Frame",
        "Prop_Window_Frame",
        "Prop_Kitchen_Counter",
        "Prop_Kitchen_Sink",
        "Prop_Kitchen_Stove",
        "Prop_Kitchen_Fridge",
        "Prop_Dining_Table",
        "Prop_Chair",
        "Prop_Sofa",
        "Prop_Crystal_Lamp",
        "Prop_Bath_Tub",
        "Prop_Bath_Vanity",
        "Prop_Bath_Toilet",
        "Prop_Bed",
        "Prop_Nightstand_Lamp",
        "Prop_Wardrobe",
        "Prop_Rug",
    ]
    for idx, k in enumerate(furniture_order):
        col = idx % 6
        row = idx // 6
        gx = (col - 2.5) * 1.55
        gy = (row - 1.0) * 1.85
        built[k].location = Vector((gx, gy, 0.0))
        built[k].hide_render = False
        if k == "Prop_Door_Frame":
            built["Prop_Door_Leaf"].location = Vector((gx - 0.41, gy, 0.0))
            built["Prop_Door_Leaf"].rotation_euler = Euler((0.0, 0.0, 0.65), "XYZ")
            built["Prop_Door_Leaf"].hide_render = False
        elif k == "Prop_Window_Frame":
            built["Prop_Window_Sash"].location = Vector((gx - 0.42, gy, 0.0))
            built["Prop_Window_Sash"].rotation_euler = Euler((0.0, 0.0, 0.55), "XYZ")
            built["Prop_Window_Sash"].hide_render = False

    setup_studio_render((0.0, -9.2, 5.8), (0.0, 0.2, 0.75), PREVIEW_FURNITURE)

    # 3. Arrange Crafted Weapons, Charms, Beacons, Pet Egg & Relics Showcase & Render Sheet 2
    for obj in built.values():
        obj.hide_render = True
        obj.rotation_euler = Euler((0.0, 0.0, 0.0), "XYZ")
        obj.scale = Vector((1.0, 1.0, 1.0))

    loot_order = [
        "Loot_Ray_Gun",
        "Loot_Crystal_Blade",
        "Loot_Starlight_Wand",
        "Loot_Aegis_Charm",
        "Loot_Rift_Beacon",
        "Loot_Pet_Egg",
        "Loot_Aurora_Lamp",
        "Loot_Geode_Lamp",
        "Loot_Slime_Lamp",
        "Loot_Bat_Plush",
        "Loot_Royal_Chalice",
        "Loot_Dragon_Egg",
        "Loot_Display_Pedestal",
    ]
    for idx, k in enumerate(loot_order):
        col = idx % 7
        row = idx // 7
        gx = (col - 3.0) * 0.92
        gy = (row - 0.5) * 1.15
        built[k].location = Vector((gx, gy, 0.0))
        if k == "Loot_Ray_Gun":
            built[k].rotation_euler = Euler((0.0, 0.0, 0.78), "XYZ")
            built[k].scale = Vector((1.28, 1.28, 1.28))
        built[k].hide_render = False

    setup_studio_render((0.0, -5.8, 2.35), (0.0, 0.0, 0.35), PREVIEW_LOOT)
    print("Studio verification renders saved cleanly!")


if __name__ == "__main__":
    main()
