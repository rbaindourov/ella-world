#!/usr/bin/env python3
"""
Headless Blender 5.2 LTS Environment Asset Builder for ELLA World.
Sources real PolyHaven CC0 3D models (jacaranda_tree, tree_small_02, flood_lights,
Chandelier_01, painted_wooden_bench, street_lamp_01, stone_fire_pit), normalizes and
decimates them for crisp 60fps Three.js WebGL rendering, adds couture DTI Fashion
Runway architectural assemblies (fluted arch, pleated velvet backdrop, faceted star
crest, draped velvet stanchion), packs 512x512 textures, exports:
  - assets/blender/ella_world_environment.blend
  - public/models/ella_world_environment.glb
  - scratch/env_trees_preview.png
  - scratch/env_dti_stage_preview.png
  - scratch/env_pack_telemetry.json
"""
import json
import math
import random
from pathlib import Path

import bmesh
import bpy
import numpy as np
from mathutils import Euler, Matrix, Vector

REPO_ROOT = Path(__file__).resolve().parent.parent
SRC_DIR = REPO_ROOT / "scratch/cc0_env_sources"
OUT_BLEND = REPO_ROOT / "assets/blender/ella_world_environment.blend"
OUT_GLB = REPO_ROOT / "public/models/ella_world_environment.glb"
OUT_TELEMETRY = REPO_ROOT / "scratch/env_pack_telemetry.json"
PREVIEW_TREES = REPO_ROOT / "scratch/env_trees_preview.png"
PREVIEW_DTI = REPO_ROOT / "scratch/env_dti_stage_preview.png"
PREVIEW_STARGATE_CAMP = REPO_ROOT / "scratch/env_stargate_camp_preview.png"
PREVIEW_ENEMIES = REPO_ROOT / "scratch/env_enemies_preview.png"
PREVIEW_EXPANSION = REPO_ROOT / "scratch/env_expansion_preview.png"


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def load_resized_image(path: Path, max_dim: int = 512, colorspace: str = "sRGB", tint_rgb=None, tag: str = ""):
    key = f"IMG_{path.stem}_{max_dim}_{tag}"
    if key in bpy.data.images:
        return bpy.data.images[key]
    img = bpy.data.images.load(str(path), check_existing=False)
    img.name = key
    if img.size[0] > max_dim or img.size[1] > max_dim:
        scale = max_dim / max(img.size[0], img.size[1])
        img.scale(max(16, int(img.size[0] * scale)), max(16, int(img.size[1] * scale)))
    if tint_rgb is not None:
        # Bake tint directly into pixels so glTF 2.0 exports exact cherry-blossom / lilac colors
        px = np.empty(len(img.pixels), dtype=np.float32)
        img.pixels.foreach_get(px)
        arr = px.reshape((-1, 4))
        fac = float(tint_rgb[3])
        target = np.array([tint_rgb[0], tint_rgb[1], tint_rgb[2]], dtype=np.float32)
        arr[:, :3] = arr[:, :3] * (1.0 - fac) + target * fac
        img.pixels.foreach_set(arr.ravel())
    try:
        img.colorspace_settings.name = colorspace
    except Exception:
        pass
    img.pack()
    return img


def make_pbr_mat(
    name: str,
    base_color=(0.8, 0.8, 0.8, 1.0),
    roughness=0.5,
    metallic=0.0,
    emissive=None,
    emissive_strength=0.0,
    diff_path: Path = None,
    alpha_path: Path = None,
    tint_rgb=None,
    double_sided=False,
    max_tex=512,
):
    if name in bpy.data.materials:
        return bpy.data.materials[name]
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    mat.use_backface_culling = not double_sided
    nt = mat.node_tree
    nt.nodes.clear()

    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = base_color
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    if emissive is not None and emissive_strength > 0.0:
        if "Emission Color" in bsdf.inputs:
            bsdf.inputs["Emission Color"].default_value = (*emissive[:3], 1.0)
            bsdf.inputs["Emission Strength"].default_value = emissive_strength
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])

    if diff_path and diff_path.exists():
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = load_resized_image(
            diff_path,
            max_dim=max_tex,
            colorspace="sRGB",
            tint_rgb=tint_rgb,
            tag=name if tint_rgb is not None else "",
        )
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])

    if alpha_path and alpha_path.exists():
        atex = nt.nodes.new("ShaderNodeTexImage")
        atex.image = load_resized_image(alpha_path, max_dim=max_tex, colorspace="Non-Color")
        nt.links.new(atex.outputs["Color"], bsdf.inputs["Alpha"])
        mat.blend_method = "CLIP"
        mat.alpha_threshold = 0.42
        mat.use_backface_culling = False

    return mat


def append_objects_from_blend(blend_path: Path, name_predicate):
    """Appends matching mesh objects from a CC0 .blend and bakes their world transforms."""
    with bpy.data.libraries.load(str(blend_path), link=False) as (data_from, data_to):
        wanted = [n for n in data_from.objects if name_predicate(n)]
        data_to.objects = wanted

    appended = []
    for o in data_to.objects:
        if o is None:
            continue
        bpy.context.collection.objects.link(o)
        appended.append(o)

    # Rule A from blender-3d-asset-pipeline: clear parent keeping transform & bake matrix_world
    bpy.context.view_layer.update()
    mesh_objs = [o for o in appended if o.type == "MESH"]
    for o in mesh_objs:
        if o.data.users > 1:
            o.data = o.data.copy()
        mw = o.matrix_world.copy()
        o.parent = None
        o.data.transform(mw)
        o.matrix_world = Matrix.Identity(4)
        o.data.update()
        # Clear any heavy modifiers (geometry nodes, weighted normal) cleanly
        for mod in list(o.modifiers):
            o.modifiers.remove(mod)
    for o in appended:
        if o.type != "MESH":
            bpy.data.objects.remove(o, do_unlink=True)
    return mesh_objs


def join_and_decimate(objs, name: str, target_polys: int = None):
    if not objs:
        raise RuntimeError(f"No objects to join for {name}")
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    if len(objs) > 1:
        bpy.ops.object.join()
    obj = bpy.context.active_object
    obj.name = name
    obj.data.name = f"{name}_Mesh"

    if target_polys and len(obj.data.polygons) > target_polys:
        ratio = max(0.03, min(0.95, target_polys / len(obj.data.polygons)))
        mod = obj.modifiers.new(name="Decimate", type="DECIMATE")
        mod.ratio = ratio
        mod.use_collapse_triangulate = True
        bpy.ops.object.modifier_apply(modifier=mod.name)

    return obj


def normalize_origin_and_scale(obj, target_height: float = None, target_width: float = None, anchor_z: str = "bottom"):
    bpy.context.view_layer.update()
    verts = [v.co for v in obj.data.vertices]
    if not verts:
        return obj
    min_x = min(v.x for v in verts)
    max_x = max(v.x for v in verts)
    min_y = min(v.y for v in verts)
    max_y = max(v.y for v in verts)
    min_z = min(v.z for v in verts)
    max_z = max(v.z for v in verts)

    cx = (min_x + max_x) * 0.5
    cy = (min_y + max_y) * 0.5
    if anchor_z == "top":
        cz = max_z
    elif anchor_z == "center":
        cz = (min_z + max_z) * 0.5
    else:
        cz = min_z

    h = max(1e-4, max_z - min_z)
    w = max(1e-4, max_x - min_x)
    scale = 1.0
    if target_height is not None:
        scale = target_height / h
    elif target_width is not None:
        scale = target_width / w

    mat = Matrix.Scale(scale, 4) @ Matrix.Translation(Vector((-cx, -cy, -cz)))
    obj.data.transform(mat)
    obj.matrix_world = Matrix.Identity(4)
    obj.data.update()
    return obj


def build_botanical_tree(
    name: str,
    src_subdir: str,
    trunk_pred,
    leaf_pred,
    bark_tex: Path,
    leaf_diff: Path,
    leaf_alpha: Path,
    target_height: float,
    trunk_target_polys: int,
    canopy_clusters: int,
    cluster_scale_range: tuple,
    leaf_tint_rgba=None,
    seed: int = 42,
):
    """
    Combines a real PolyHaven CC0 photogrammetry sculpted trunk with instanced,
    UV-mapped real CC0 leaf clusters placed organically on the upper branch crown.
    """
    bp = SRC_DIR / src_subdir / f"{src_subdir}.blend"
    trunk_objs = append_objects_from_blend(bp, trunk_pred)
    trunk = join_and_decimate(trunk_objs, f"{name}_Trunk", target_polys=trunk_target_polys)
    normalize_origin_and_scale(trunk, target_height=target_height * 0.88, anchor_z="bottom")

    bark_mat = make_pbr_mat(
        f"Mat_{name}_Bark",
        base_color=(0.35, 0.24, 0.16, 1.0),
        roughness=0.86,
        diff_path=bark_tex,
        max_tex=512,
    )
    trunk.data.materials.clear()
    trunk.data.materials.append(bark_mat)

    leaf_objs = append_objects_from_blend(bp, leaf_pred)
    if not leaf_objs:
        raise RuntimeError(f"No leaf objects found for {name}")
    leaf_tpl = join_and_decimate(leaf_objs, f"{name}_LeafTpl", target_polys=28)
    normalize_origin_and_scale(leaf_tpl, target_width=1.0, anchor_z="center")

    leaf_mat = make_pbr_mat(
        f"Mat_{name}_Leaves",
        base_color=(0.35, 0.72, 0.32, 1.0),
        roughness=0.62,
        diff_path=leaf_diff,
        alpha_path=leaf_alpha,
        tint_rgb=leaf_tint_rgba,
        double_sided=True,
        max_tex=512,
    )

    # Sample upper crown vertices from the real sculpted trunk branches
    verts = [v.co.copy() for v in trunk.data.vertices]
    max_z = max(v.z for v in verts)
    crown_pts = [v for v in verts if v.z >= max_z * 0.44]
    if not crown_pts:
        crown_pts = verts

    rng = random.Random(seed)
    # Build canopy mesh by copying leaf_tpl geometry onto distributed crown anchor points
    canopy_bm = bmesh.new()
    src_bm = bmesh.new()
    src_bm.from_mesh(leaf_tpl.data)
    src_bm.verts.ensure_lookup_table()
    src_uv = src_bm.loops.layers.uv.active
    dst_uv = canopy_bm.loops.layers.uv.new("UVMap")

    # Select spatially diverse anchor points around the crown
    rng.shuffle(crown_pts)
    chosen = []
    min_spacing = target_height * 0.085
    for pt in crown_pts:
        if len(chosen) >= canopy_clusters:
            break
        if all((pt - c).length >= min_spacing for c in chosen):
            chosen.append(pt)
    while len(chosen) < canopy_clusters:
        base = rng.choice(crown_pts)
        jitter = Vector((
            rng.uniform(-0.65, 0.65),
            rng.uniform(-0.65, 0.65),
            rng.uniform(-0.25, 0.55),
        ))
        chosen.append(base + jitter)

    for anchor in chosen:
        sc = rng.uniform(cluster_scale_range[0], cluster_scale_range[1])
        rot = Euler((
            rng.uniform(-0.55, 0.55),
            rng.uniform(-0.55, 0.55),
            rng.uniform(0.0, math.tau),
        ), "XYZ").to_matrix().to_4x4()
        outward = Vector((anchor.x, anchor.y, 0.0))
        if outward.length > 0.05:
            outward.normalize()
        offset = anchor + outward * rng.uniform(0.12, 0.52) + Vector((0, 0, rng.uniform(0.05, 0.45)))
        xf = Matrix.Translation(offset) @ rot @ Matrix.Scale(sc, 4)

        v_map = {}
        for v in src_bm.verts:
            v_map[v.index] = canopy_bm.verts.new(xf @ v.co)
        for f in src_bm.faces:
            try:
                nf = canopy_bm.faces.new([v_map[v.index] for v in f.verts])
                nf.smooth = True
                if src_uv:
                    for sl, dl in zip(f.loops, nf.loops):
                        dl[dst_uv].uv = sl[src_uv].uv.copy()
            except ValueError:
                pass

    src_bm.free()
    bpy.data.objects.remove(leaf_tpl, do_unlink=True)

    canopy_mesh = bpy.data.meshes.new(f"{name}_CanopyMesh")
    canopy_bm.to_mesh(canopy_mesh)
    canopy_bm.free()
    canopy_obj = bpy.data.objects.new(f"{name}_Canopy", canopy_mesh)
    bpy.context.collection.objects.link(canopy_obj)
    canopy_obj.data.materials.append(leaf_mat)

    # Join trunk + canopy into one clean multi-material tree asset
    tree = join_and_decimate([trunk, canopy_obj], name, target_polys=None)
    normalize_origin_and_scale(tree, target_height=target_height, anchor_z="bottom")
    for p in tree.data.polygons:
        p.use_smooth = True
    return tree


def append_box(bm, mat_idx, size, loc, rot=(0, 0, 0), bevel_width=0.0):
    xf = Matrix.Translation(Vector(loc)) @ Euler(rot, "XYZ").to_matrix().to_4x4() @ Matrix.Diagonal((*size, 1.0))
    ret = bmesh.ops.create_cube(bm, size=1.0, matrix=xf)
    verts = ret["verts"]
    faces = list({f for v in verts for f in v.link_faces})
    for f in faces:
        f.material_index = mat_idx
    if bevel_width > 0.0:
        edges = list({e for f in faces for e in f.edges})
        bmesh.ops.bevel(bm, geom=edges, offset=bevel_width, segments=2, affect="EDGES")


def append_cyl(bm, mat_idx, r1, r2, depth, segs, loc, rot=(0, 0, 0)):
    xf = Matrix.Translation(Vector(loc)) @ Euler(rot, "XYZ").to_matrix().to_4x4()
    ret = bmesh.ops.create_cone(
        bm,
        cap_ends=True,
        cap_tris=False,
        segments=segs,
        radius1=r1,
        radius2=r2,
        depth=depth,
        matrix=xf,
    )
    verts = ret["verts"]
    for v in verts:
        for f in v.link_faces:
            f.material_index = mat_idx
            f.smooth = True


def build_dti_couture_arch():
    """High-craft DTI Runway Couture Archway (fluted rose-gold columns, marble plinths, molded arch rib)."""
    bm = bmesh.new()
    m_marble = make_pbr_mat("Mat_DTI_Marble", base_color=(0.97, 0.94, 0.96, 1.0), roughness=0.26, metallic=0.04)
    m_rosegold = make_pbr_mat("Mat_DTI_RoseGold", base_color=(0.94, 0.64, 0.72, 1.0), roughness=0.22, metallic=0.72)
    m_gold = make_pbr_mat("Mat_DTI_Gold", base_color=(0.98, 0.78, 0.28, 1.0), roughness=0.2, metallic=0.82)
    m_glow = make_pbr_mat(
        "Mat_DTI_PinkNeon",
        base_color=(1.0, 0.72, 0.88, 1.0),
        roughness=0.18,
        emissive=(0.95, 0.35, 0.68),
        emissive_strength=2.2,
    )

    span_half = 3.35
    col_h = 2.65
    for side in (-1.0, 1.0):
        px = side * span_half
        # Stepped marble plinth
        append_box(bm, 0, (0.95, 0.95, 0.24), (px, 0, 0.12), bevel_width=0.03)
        append_box(bm, 2, (0.82, 0.82, 0.08), (px, 0, 0.28), bevel_width=0.015)
        append_box(bm, 0, (0.72, 0.72, 0.22), (px, 0, 0.43), bevel_width=0.02)
        # Fluted column shaft
        append_cyl(bm, 0, 0.24, 0.21, col_h, 16, (px, 0, 0.54 + col_h * 0.5))
        for k in range(8):
            ang = k * (math.tau / 8.0)
            append_cyl(
                bm,
                1,
                0.035,
                0.032,
                col_h - 0.16,
                6,
                (px + math.cos(ang) * 0.22, math.sin(ang) * 0.22, 0.54 + col_h * 0.5),
            )
        # Capital
        append_cyl(bm, 2, 0.24, 0.36, 0.22, 16, (px, 0, 0.54 + col_h + 0.11))
        append_box(bm, 0, (0.78, 0.78, 0.14), (px, 0, 0.54 + col_h + 0.29), bevel_width=0.02)

    # Molded semicircular couture arch rib with inner pink neon piping & gold keystone
    arch_center_z = 0.54 + col_h + 0.32
    arch_r = span_half
    segs = 26
    for i in range(segs):
        a0 = math.pi * (i / segs)
        a1 = math.pi * ((i + 1) / segs)
        am = 0.5 * (a0 + a1)
        seg_len = arch_r * (a1 - a0) * 1.06
        cx = math.cos(am) * arch_r
        cz = arch_center_z + math.sin(am) * arch_r
        rot_y = am - math.pi * 0.5
        # Main rose-gold rib
        append_box(bm, 1, (seg_len, 0.42, 0.34), (cx, 0, cz), rot=(0, -rot_y, 0))
        # Outer gold cornice trim
        ox = math.cos(am) * (arch_r + 0.2)
        oz = arch_center_z + math.sin(am) * (arch_r + 0.2)
        append_box(bm, 2, (seg_len * 1.04, 0.48, 0.09), (ox, 0, oz), rot=(0, -rot_y, 0))
        # Inner neon light strip
        ix = math.cos(am) * (arch_r - 0.19)
        iz = arch_center_z + math.sin(am) * (arch_r - 0.19)
        append_box(bm, 3, (seg_len * 1.02, 0.14, 0.08), (ix, 0, iz), rot=(0, -rot_y, 0))

    # Crown keystone medallion at top center
    append_box(bm, 2, (0.62, 0.56, 0.62), (0, 0, arch_center_z + arch_r + 0.05), bevel_width=0.04)
    append_cyl(bm, 2, 0.06, 0.06, 0.42, 10, (0, 0, arch_center_z + arch_r - 0.32))

    mesh = bpy.data.meshes.new("Env_DTI_Arch_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_DTI_Arch", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_marble, m_rosegold, m_gold, m_glow):
        obj.data.materials.append(m)
    return obj


def build_dti_backdrop():
    """High-craft VIP Fashion Runway Backstage Photo Wall with pleated velvet drapes & gold cornice."""
    bm = bmesh.new()
    m_velvet = make_pbr_mat("Mat_DTI_Velvet", base_color=(0.24, 0.09, 0.46, 1.0), roughness=0.78, metallic=0.04)
    m_velvet_hi = make_pbr_mat("Mat_DTI_VelvetLight", base_color=(0.35, 0.13, 0.62, 1.0), roughness=0.72, metallic=0.06)
    m_gold = make_pbr_mat("Mat_DTI_Gold")
    m_marble = make_pbr_mat("Mat_DTI_Marble")
    m_glow = make_pbr_mat("Mat_DTI_PinkNeon")

    w = 10.4
    h = 6.2
    # Main velvet wall core
    append_box(bm, 0, (w, 0.42, h), (0, 0, h * 0.5))
    # Sculpted vertical velvet curtain pleats facing the runway (-Y in Blender / +Z in Three.js)
    num_pleats = 22
    for i in range(num_pleats):
        px = -w * 0.46 + (w * 0.92) * (i / (num_pleats - 1))
        append_cyl(bm, 1 if i % 2 == 0 else 0, 0.24, 0.22, h - 0.55, 8, (px, -0.20, h * 0.5))

    # Gilded top cornice & bottom dado plinth
    append_box(bm, 3, (w + 0.65, 0.72, 0.38), (0, 0, 0.19), bevel_width=0.03)
    append_box(bm, 2, (w + 0.85, 0.82, 0.36), (0, 0, h + 0.12), bevel_width=0.04)
    append_box(bm, 3, (w + 0.55, 0.68, 0.26), (0, 0, h - 0.16), bevel_width=0.02)
    # Recessed pink neon cove strip beneath top cornice (front face)
    append_box(bm, 4, (w - 0.6, 0.12, 0.10), (0, -0.34, h - 0.34))

    # Ornate fluted side pilasters
    for side in (-1.0, 1.0):
        sx = side * (w * 0.5 + 0.18)
        append_box(bm, 3, (0.76, 0.78, 0.65), (sx, -0.06, 0.325), bevel_width=0.03)
        append_cyl(bm, 3, 0.28, 0.26, h - 0.9, 14, (sx, -0.1, h * 0.5))
        append_box(bm, 2, (0.86, 0.86, 0.42), (sx, -0.06, h - 0.2), bevel_width=0.04)

    # Central stepped gold pediment arch above the wall
    append_box(bm, 2, (4.2, 0.62, 0.42), (0, 0.04, h + 0.46), bevel_width=0.04)
    append_box(bm, 2, (2.6, 0.54, 0.36), (0, 0.04, h + 0.82), bevel_width=0.04)

    mesh = bpy.data.meshes.new("Env_DTI_Backdrop_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_DTI_Backdrop", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_velvet, m_velvet_hi, m_gold, m_marble, m_glow):
        obj.data.materials.append(m)
    return obj


def build_dti_star_crest():
    """Sculpted 8-point faceted Starlight Crest + jeweled halo ring (centered at origin)."""
    bm = bmesh.new()
    m_gold = make_pbr_mat("Mat_DTI_Gold")
    m_glow = make_pbr_mat("Mat_DTI_PinkNeon")
    m_diamond = make_pbr_mat(
        "Mat_DTI_Diamond",
        base_color=(0.75, 0.95, 1.0, 1.0),
        roughness=0.08,
        metallic=0.2,
        emissive=(0.45, 0.82, 1.0),
        emissive_strength=1.1,
    )

    # 8-point faceted star built from radiating tapered pyramids
    for i in range(8):
        ang = i * (math.tau / 8.0)
        is_major = i % 2 == 0
        length = 1.25 if is_major else 0.88
        rad = 0.28 if is_major else 0.2
        xf = (
            Matrix.Rotation(ang, 4, "Y")
            @ Matrix.Translation(Vector((0, 0, length * 0.5)))
        )
        ret = bmesh.ops.create_cone(
            bm,
            cap_ends=True,
            cap_tris=True,
            segments=4,
            radius1=rad,
            radius2=0.015,
            depth=length,
            matrix=xf,
        )
        for v in ret["verts"]:
            for f in v.link_faces:
                f.material_index = 0 if is_major else 1

    # Central faceted diamond gem
    ret = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=0.42)
    for v in ret["verts"]:
        for f in v.link_faces:
            f.material_index = 2

    # Outer jeweled halo ring (continuous tangential segments in X-Z plane)
    ring_r = 1.48
    ring_segs = 32
    for i in range(ring_segs):
        a0 = i * (math.tau / ring_segs)
        a1 = (i + 1) * (math.tau / ring_segs)
        p0 = Vector((math.cos(a0) * ring_r, 0, math.sin(a0) * ring_r))
        p1 = Vector((math.cos(a1) * ring_r, 0, math.sin(a1) * ring_r))
        mid = (p0 + p1) * 0.5
        dv = p1 - p0
        rot = dv.to_track_quat("Z", "Y").to_euler("XYZ")
        append_cyl(bm, 1 if i % 2 == 0 else 0, 0.085, 0.085, dv.length * 1.15, 8, tuple(mid), rot=tuple(rot))

    mesh = bpy.data.meshes.new("Env_DTI_StarCrest_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_DTI_StarCrest", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_gold, m_glow, m_diamond):
        obj.data.materials.append(m)
    return obj


def build_dti_stanchion():
    """Brass/gold stanchion post + draped catenary velvet rope segment (2.0m span along +Y)."""
    bm = bmesh.new()
    m_gold = make_pbr_mat("Mat_DTI_Gold")
    m_rope = make_pbr_mat(
        "Mat_DTI_VelvetRope",
        base_color=(0.86, 0.14, 0.45, 1.0),
        roughness=0.68,
        emissive=(0.45, 0.05, 0.22),
        emissive_strength=0.25,
    )

    # Weighted brass base & fluted post
    append_cyl(bm, 0, 0.22, 0.18, 0.07, 16, (0, 0, 0.035))
    append_cyl(bm, 0, 0.045, 0.042, 0.92, 12, (0, 0, 0.52))
    append_cyl(bm, 0, 0.085, 0.085, 0.05, 12, (0, 0, 0.88))
    ret = bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.075, matrix=Matrix.Translation((0, 0, 1.02)))
    for v in ret["verts"]:
        for f in v.link_faces:
            f.material_index = 0
            f.smooth = True

    # Draped continuous catenary velvet rope from Y=0.04 to Y=1.96
    rope_segs = 14
    span = 2.0
    for i in range(rope_segs):
        t0 = i / rope_segs
        t1 = (i + 1) / rope_segs
        y0 = 0.04 + t0 * (span - 0.08)
        y1 = 0.04 + t1 * (span - 0.08)
        z0 = 0.86 - math.sin(t0 * math.pi) * 0.28
        z1 = 0.86 - math.sin(t1 * math.pi) * 0.28
        p0 = Vector((0, y0, z0))
        p1 = Vector((0, y1, z1))
        mid = (p0 + p1) * 0.5
        dv = p1 - p0
        rot = dv.to_track_quat("Z", "X").to_euler("XYZ")
        append_cyl(bm, 1, 0.038, 0.038, dv.length * 1.18, 8, tuple(mid), rot=tuple(rot))

    mesh = bpy.data.meshes.new("Env_DTI_Stanchion_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_DTI_Stanchion", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_gold, m_rope):
        obj.data.materials.append(m)
    return obj


def build_stargate_frame():
    """
    High-craft Blender 5.2 Stargate Outer Ring, 9 Sculpted Glowing Chevron Housings,
    and Beveled Boarding Ramp (origin at ground center (0,0,0), aperture center at z=2.95m).
    """
    bm = bmesh.new()
    m_naquadah = make_pbr_mat(
        "Mat_Stargate_Naquadah",
        base_color=(0.18, 0.21, 0.28, 1.0),
        roughness=0.34,
        metallic=0.76,
    )
    m_trim = make_pbr_mat(
        "Mat_Stargate_Trim",
        base_color=(0.42, 0.48, 0.60, 1.0),
        roughness=0.25,
        metallic=0.86,
    )
    m_chevron = make_pbr_mat(
        "Mat_Stargate_ChevronGlow",
        base_color=(1.0, 0.62, 0.16, 1.0),
        roughness=0.14,
        emissive=(1.0, 0.45, 0.08),
        emissive_strength=3.2,
    )
    m_ramp = make_pbr_mat(
        "Mat_Stargate_Ramp",
        base_color=(0.14, 0.15, 0.22, 1.0),
        roughness=0.48,
        metallic=0.42,
    )
    m_cyan_strip = make_pbr_mat(
        "Mat_Stargate_CyanGlow",
        base_color=(0.45, 0.92, 1.0, 1.0),
        roughness=0.14,
        emissive=(0.18, 0.78, 1.0),
        emissive_strength=2.6,
    )

    center_z = 2.95
    outer_r = 2.86
    segs = 36
    # Outer stepped naquadah ring housing (36 beveled radial segments)
    for i in range(segs):
        a0 = i * (math.tau / segs)
        a1 = (i + 1) * (math.tau / segs)
        am = 0.5 * (a0 + a1)
        seg_len = outer_r * (a1 - a0) * 1.06
        cx = math.cos(am) * outer_r
        cz = center_z + math.sin(am) * outer_r
        rot_y = am - math.pi * 0.5
        # Main heavy ring block
        append_box(bm, 0, (seg_len, 0.56, 0.52), (cx, 0, cz), rot=(0, -rot_y, 0))
        # Outer rim stepped bevel flange
        ox = math.cos(am) * (outer_r + 0.28)
        oz = center_z + math.sin(am) * (outer_r + 0.28)
        append_box(bm, 1, (seg_len * 1.04, 0.34, 0.14), (ox, 0, oz), rot=(0, -rot_y, 0))
        # Inner aperture lip ring
        ix = math.cos(am) * (outer_r - 0.54)
        iz = center_z + math.sin(am) * (outer_r - 0.54)
        append_box(bm, 1, (seg_len * 0.86, 0.44, 0.12), (ix, 0, iz), rot=(0, -rot_y, 0))

    # 9 Iconic Sculpted Stargate Chevrons spaced every 40° (top chevron at 90°)
    for c in range(9):
        ang = math.pi * 0.5 + c * (math.tau / 9.0)
        cx = math.cos(ang) * (outer_r + 0.06)
        cz = center_z + math.sin(ang) * (outer_r + 0.06)
        rot_y = ang - math.pi * 0.5
        # Outer armored chevron clamp
        append_box(bm, 1, (0.58, 0.72, 0.68), (cx, 0, cz), rot=(0, -rot_y, 0), bevel_width=0.03)
        # Inner pointing V-tip housing
        tx = math.cos(ang) * (outer_r - 0.24)
        tz = center_z + math.sin(ang) * (outer_r - 0.24)
        append_box(bm, 0, (0.34, 0.66, 0.32), (tx, 0, tz), rot=(0, -rot_y, 0))
        # Glowing crystal chevron lens (front & back faces visible!)
        gx = math.cos(ang) * (outer_r + 0.04)
        gz = center_z + math.sin(ang) * (outer_r + 0.04)
        append_box(bm, 2, (0.28, 0.78, 0.36), (gx, 0, gz), rot=(0, -rot_y, 0))

    # Stepped Boarding Plinth & Sci-Fi Ramp at base (z = 0..0.42)
    append_box(bm, 3, (5.2, 2.6, 0.22), (0, 0, 0.11), bevel_width=0.03)
    append_box(bm, 0, (4.2, 1.8, 0.16), (0, 0, 0.28), bevel_width=0.02)
    # Glowing cyan guide rails along the boarding ramp
    for side in (-1.0, 1.0):
        append_box(bm, 3, (0.52, 2.4, 0.68), (side * 2.38, 0, 0.34), bevel_width=0.03)
        append_box(bm, 4, (0.12, 2.2, 0.08), (side * 2.02, 0, 0.36))

    mesh = bpy.data.meshes.new("Env_Stargate_Frame_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Stargate_Frame", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_naquadah, m_trim, m_chevron, m_ramp, m_cyan_strip):
        obj.data.materials.append(m)
    return obj


def build_stargate_inner_ring():
    """
    Separate spinning Stargate Inner Glyph Track Ring (centered at (0,0,0) in X-Z plane)
    so Three.js can rotate it smoothly inside Env_Stargate_Frame.
    """
    bm = bmesh.new()
    m_track = make_pbr_mat(
        "Mat_Stargate_GlyphTrack",
        base_color=(0.24, 0.28, 0.38, 1.0),
        roughness=0.28,
        metallic=0.82,
    )
    m_glyph = make_pbr_mat(
        "Mat_Stargate_GlyphGlow",
        base_color=(0.62, 0.90, 1.0, 1.0),
        roughness=0.16,
        emissive=(0.35, 0.75, 1.0),
        emissive_strength=1.55,
    )

    ring_r = 2.48
    segs = 36
    for i in range(segs):
        a0 = i * (math.tau / segs)
        a1 = (i + 1) * (math.tau / segs)
        am = 0.5 * (a0 + a1)
        seg_len = ring_r * (a1 - a0) * 1.04
        cx = math.cos(am) * ring_r
        cz = math.sin(am) * ring_r
        rot_y = am - math.pi * 0.5
        # Track ring segment
        append_box(bm, 0, (seg_len, 0.36, 0.28), (cx, 0, cz), rot=(0, -rot_y, 0))
        # Raised radial glyph divider rib
        dx = math.cos(a0) * ring_r
        dz = math.sin(a0) * ring_r
        append_box(bm, 0, (0.045, 0.44, 0.30), (dx, 0, dz), rot=(0, -(a0 - math.pi * 0.5), 0))
        # Illuminated constellation glyph dot/bar on each bay
        append_box(
            bm,
            1,
            (seg_len * 0.42, 0.42, 0.09 if i % 2 == 0 else 0.14),
            (cx, 0, cz),
            rot=(0, -rot_y, 0),
        )

    mesh = bpy.data.meshes.new("Env_Stargate_InnerRing_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Stargate_InnerRing", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_track, m_glyph):
        obj.data.materials.append(m)
    return obj


def build_camp_tent():
    """
    Cozy Starlight A-Frame Glamping Tent (3.6m wide x 4.0m long x 2.55m tall)
    with striped canvas walls, cedar ridge poles, rolled open front flaps,
    sleeping mat, cozy pillow, warm interior hanging lantern, and guy-lines.
    """
    bm = bmesh.new()
    m_canvas = make_pbr_mat("Mat_Camp_Canvas", base_color=(0.97, 0.94, 0.88, 1.0), roughness=0.82)
    m_stripe = make_pbr_mat("Mat_Camp_Stripe", base_color=(0.94, 0.42, 0.65, 1.0), roughness=0.78)
    m_wood = make_pbr_mat("Mat_Camp_Wood", base_color=(0.44, 0.26, 0.13, 1.0), roughness=0.76)
    m_blanket = make_pbr_mat("Mat_Camp_Blanket", base_color=(0.32, 0.62, 0.88, 1.0), roughness=0.85)
    m_glow = make_pbr_mat(
        "Mat_Camp_LanternGlow",
        base_color=(1.0, 0.92, 0.62, 1.0),
        roughness=0.15,
        emissive=(1.0, 0.78, 0.32),
        emissive_strength=3.0,
    )

    # Ground tarp / floor mat
    append_box(bm, 3, (3.2, 3.6, 0.06), (0, 0, 0.03), bevel_width=0.015)

    # True pitched A-frame roof slopes meeting at apex (0, y, 2.32) down to eaves (±1.62, y, 0.06)
    apex_z = 2.34
    half_w = 1.64
    slope_len = math.hypot(half_w, apex_z)
    tilt = math.atan2(half_w, apex_z)  # ~0.61 rad (~35°)
    for side in (-1.0, 1.0):
        # Negative sign on side * tilt leans the top INWARD toward X=0 to form a crisp /\ peak!
        rot_y = -side * tilt
        # Main cream canvas roof/wall panel
        append_box(
            bm,
            0,
            (0.08, 3.72, slope_len),
            (side * half_w * 0.5, 0, apex_z * 0.5),
            rot=(0, rot_y, 0),
        )
        # Decorative pastel-pink glamping accent band along lower eaves
        append_box(
            bm,
            1,
            (0.10, 3.76, 0.42),
            (side * half_w * 0.86, 0, apex_z * 0.16),
            rot=(0, rot_y, 0),
        )
        # Front rolled entrance canvas flaps tucked cleanly along the /\ door edges
        append_cyl(
            bm,
            1,
            0.13,
            0.12,
            slope_len * 0.86,
            10,
            (side * half_w * 0.52, -1.86, apex_z * 0.48),
            rot=(0, rot_y, 0),
        )
        # Guy-line stakes
        append_cyl(bm, 2, 0.04, 0.025, 0.32, 6, (side * 2.15, -1.5, 0.14), rot=(0, -side * 0.25, 0))
        append_cyl(bm, 2, 0.04, 0.025, 0.32, 6, (side * 2.15, 1.5, 0.14), rot=(0, -side * 0.25, 0))

    # Crisp solid triangular back gable wall (true 3-vertex extruded wedge, zero stair-stepping!)
    v_bl = bm.verts.new((-half_w + 0.04, 1.80, 0.04))
    v_br = bm.verts.new((half_w - 0.04, 1.80, 0.04))
    v_top = bm.verts.new((0.0, 1.80, apex_z - 0.04))
    v_bl2 = bm.verts.new((-half_w + 0.04, 1.88, 0.04))
    v_br2 = bm.verts.new((half_w - 0.04, 1.88, 0.04))
    v_top2 = bm.verts.new((0.0, 1.88, apex_z - 0.04))
    for tri in (
        (v_bl, v_br, v_top),
        (v_br2, v_bl2, v_top2),
        (v_bl, v_bl2, v_top2, v_top),
        (v_br, v_top, v_top2, v_br2),
    ):
        try:
            f = bm.faces.new(tri)
            f.material_index = 0
        except ValueError:
            pass

    # Crossed Cedar Ridge & A-Frame Entrance Poles
    append_cyl(bm, 2, 0.06, 0.06, 4.18, 10, (0, 0, apex_z), rot=(math.pi * 0.5, 0, 0))
    for y_end in (-1.92, 1.92):
        for side in (-1.0, 1.0):
            append_cyl(
                bm,
                2,
                0.055,
                0.05,
                slope_len + 0.22,
                8,
                (side * half_w * 0.5, y_end, apex_z * 0.52),
                rot=(0, -side * tilt, 0),
            )

    # Cozy interior sleeping pad, fluffy pillow & warm hanging lantern
    append_box(bm, 1, (1.35, 2.15, 0.14), (0, 0.35, 0.13), bevel_width=0.04)
    append_box(bm, 0, (0.78, 0.42, 0.18), (0, 1.15, 0.24), bevel_width=0.06)
    append_cyl(bm, 2, 0.02, 0.02, 0.36, 6, (0, -0.55, apex_z - 0.18))
    append_cyl(bm, 4, 0.13, 0.11, 0.24, 10, (0, -0.55, apex_z - 0.44))

    mesh = bpy.data.meshes.new("Env_Camp_Tent_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Camp_Tent", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_canvas, m_stripe, m_wood, m_blanket, m_glow):
        obj.data.materials.append(m)
    return obj


def build_camp_log_bench():
    """Sculpted split-cedar natural log bench with rich cedar bark & sawn wood grain ends (4.1m wide)."""
    bm = bmesh.new()
    m_bark = make_pbr_mat("Mat_Camp_Bark", base_color=(0.34, 0.19, 0.09, 1.0), roughness=0.88)
    m_cutwood = make_pbr_mat("Mat_Camp_CutWood", base_color=(0.76, 0.57, 0.36, 1.0), roughness=0.66)

    # Main horizontal split log seat with warm cedar bark
    append_cyl(bm, 0, 0.32, 0.30, 4.05, 14, (0, 0, 0.46), rot=(0, math.pi * 0.5, 0))
    # Flattened smooth top seat plank inlay
    append_box(bm, 1, (4.02, 0.54, 0.12), (0, 0, 0.68), bevel_width=0.025)
    # Pale sawn wood end caps
    for side in (-1.0, 1.0):
        append_cyl(bm, 1, 0.27, 0.27, 0.04, 14, (side * 2.025, 0, 0.46), rot=(0, math.pi * 0.5, 0))
        # Notched timber support feet
        append_cyl(bm, 0, 0.22, 0.26, 0.42, 10, (side * 1.38, 0, 0.21))

    mesh = bpy.data.meshes.new("Env_Camp_LogBench_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Camp_LogBench", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_bark, m_cutwood):
        obj.data.materials.append(m)
    return obj


def append_sphere(bm, mat_idx, radius, loc, scale=(1.0, 1.0, 1.0), u_segs=14, v_segs=10):
    xf = Matrix.Translation(Vector(loc)) @ Matrix.Diagonal((scale[0], scale[1], scale[2], 1.0))
    ret = bmesh.ops.create_uvsphere(bm, u_segments=u_segs, v_segments=v_segs, radius=radius, matrix=xf)
    for v in ret["verts"]:
        for f in v.link_faces:
            f.material_index = mat_idx
            f.smooth = True


def build_mob_shadow_slime():
    """Sculpted Crown Shadow Slime with organic base tendrils, crystal core, glossy eyes & dark crown."""
    bm = bmesh.new()
    m_gel = make_pbr_mat(
        "Mat_Mob_SlimeGel",
        base_color=(0.46, 0.16, 0.88, 1.0),
        roughness=0.14,
        metallic=0.05,
        emissive=(0.35, 0.08, 0.72),
        emissive_strength=0.45,
    )
    m_core = make_pbr_mat(
        "Mat_Mob_SlimeCore",
        base_color=(1.0, 0.36, 0.78, 1.0),
        roughness=0.1,
        emissive=(1.0, 0.22, 0.65),
        emissive_strength=2.4,
    )
    m_white = make_pbr_mat("Mat_Mob_EyeWhite", base_color=(0.98, 0.99, 1.0, 1.0), roughness=0.12)
    m_pupil = make_pbr_mat("Mat_Mob_EyePupil", base_color=(0.06, 0.08, 0.16, 1.0), roughness=0.08)
    m_crown = make_pbr_mat(
        "Mat_Mob_DarkGold",
        base_color=(0.96, 0.74, 0.18, 1.0),
        roughness=0.22,
        metallic=0.82,
    )

    # Squishy dome body + puddles/lobes around base (-Y is forward in Blender -> +Z in Three.js)
    append_sphere(bm, 0, 0.62, (0, 0, 0.48), scale=(1.06, 1.02, 0.84), u_segs=18, v_segs=12)
    for i in range(7):
        ang = i * (math.tau / 7.0)
        append_sphere(
            bm,
            0,
            0.22,
            (math.cos(ang) * 0.52, math.sin(ang) * 0.50, 0.14),
            scale=(1.15, 1.15, 0.62),
            u_segs=10,
            v_segs=8,
        )

    # Glowing inner crystal heart
    append_cyl(bm, 1, 0.22, 0.02, 0.32, 6, (0, 0.05, 0.56))
    append_cyl(bm, 1, 0.02, 0.22, 0.24, 6, (0, 0.05, 0.34))

    # Expressive big eyes + fierce brow ridges on front (-Y)
    for sx in (-0.22, 0.22):
        append_sphere(bm, 2, 0.13, (sx, -0.54, 0.56), scale=(1.0, 0.55, 1.05), u_segs=12, v_segs=8)
        append_sphere(bm, 3, 0.075, (sx, -0.60, 0.56), scale=(1.0, 0.5, 1.0), u_segs=10, v_segs=8)
        append_sphere(bm, 2, 0.028, (sx - 0.03, -0.635, 0.60), u_segs=6, v_segs=6)
        append_box(bm, 3, (0.20, 0.06, 0.045), (sx, -0.57, 0.70), rot=(0, 0.28 * (-1 if sx < 0 else 1), 0))

    # Mini crooked royal slime crown on top
    append_cyl(bm, 4, 0.24, 0.28, 0.14, 8, (0.04, 0, 0.98), rot=(0, 0.12, 0))
    for k in range(5):
        ca = k * (math.tau / 5.0)
        append_cyl(
            bm,
            4,
            0.06,
            0.008,
            0.16,
            4,
            (0.04 + math.cos(ca) * 0.23, math.sin(ca) * 0.23, 1.11),
        )

    mesh = bpy.data.meshes.new("Env_Mob_ShadowSlime_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Mob_ShadowSlime", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_gel, m_core, m_white, m_pupil, m_crown):
        obj.data.materials.append(m)
    return obj


def build_mob_crystal_bat_parts():
    """Sculpted Obsidian Crystal Bat Body + Articulated Scalloped Left/Right Wings."""
    m_fur = make_pbr_mat("Mat_Mob_BatFur", base_color=(0.14, 0.11, 0.28, 1.0), roughness=0.65)
    m_wing = make_pbr_mat(
        "Mat_Mob_BatMembrane",
        base_color=(0.22, 0.68, 0.96, 1.0),
        roughness=0.28,
        emissive=(0.48, 0.22, 0.92),
        emissive_strength=0.85,
        double_sided=True,
    )
    m_eye = make_pbr_mat(
        "Mat_Mob_BatEye",
        base_color=(1.0, 0.88, 0.18, 1.0),
        roughness=0.1,
        emissive=(1.0, 0.78, 0.12),
        emissive_strength=2.6,
    )
    m_fang = make_pbr_mat("Mat_Mob_Fang", base_color=(0.98, 0.98, 1.0, 1.0), roughness=0.18)

    bm = bmesh.new()
    # Plump torso + head facing -Y
    append_sphere(bm, 0, 0.36, (0, 0.04, 0.56), scale=(0.95, 1.15, 1.05), u_segs=14, v_segs=10)
    append_sphere(bm, 0, 0.28, (0, -0.26, 0.72), scale=(1.02, 0.96, 0.95), u_segs=14, v_segs=10)
    # Oversized tufted bat ears + glowing crystal inner ear
    for sx in (-1.0, 1.0):
        append_cyl(bm, 0, 0.13, 0.015, 0.42, 6, (sx * 0.19, -0.22, 1.05), rot=(0.12, -sx * 0.22, 0))
        append_cyl(bm, 1, 0.075, 0.01, 0.30, 5, (sx * 0.19, -0.25, 1.03), rot=(0.12, -sx * 0.22, 0))
        # Glowing golden eyes & tiny white fangs
        append_sphere(bm, 2, 0.07, (sx * 0.12, -0.49, 0.76), u_segs=8, v_segs=8)
        append_cyl(bm, 3, 0.025, 0.004, 0.11, 5, (sx * 0.07, -0.48, 0.58), rot=(math.pi, 0, 0))
    # Dorsal crystal spines along back
    for i in range(3):
        append_cyl(bm, 1, 0.065, 0.008, 0.22, 5, (0, 0.24 + i * 0.11, 0.76 - i * 0.12), rot=(-0.65, 0, 0))

    body_mesh = bpy.data.meshes.new("Env_Mob_CrystalBat_Body_Mesh")
    bm.to_mesh(body_mesh)
    bm.free()
    body_obj = bpy.data.objects.new("Env_Mob_CrystalBat_Body", body_mesh)
    bpy.context.collection.objects.link(body_obj)
    for m in (m_fur, m_wing, m_eye, m_fang):
        body_obj.data.materials.append(m)

    def make_wing(name, side_sign):
        wbm = bmesh.new()
        # Wing hinge at local (0,0,0), extending outward along side_sign * X
        # Upper bone spar
        append_cyl(
            wbm,
            0,
            0.04,
            0.025,
            0.95,
            6,
            (side_sign * 0.46, -0.06, 0.08),
            rot=(0, side_sign * (math.pi * 0.46), 0),
        )
        # Wing claw thumb
        append_cyl(wbm, 1, 0.022, 0.005, 0.16, 5, (side_sign * 0.42, -0.15, 0.16), rot=(0.6, 0, 0))
        # 3 finger bone ribs + glowing membrane panels
        for k, (fx, fy, fz) in enumerate([
            (0.92, 0.04, 0.06),
            (0.72, 0.22, -0.12),
            (0.42, 0.26, -0.18),
        ]):
            p_end = Vector((side_sign * fx, fy, fz))
            p_start = Vector((side_sign * 0.36, -0.04, 0.06))
            mid = (p_start + p_end) * 0.5
            dv = p_end - p_start
            rot = dv.to_track_quat("Z", "Y").to_euler("XYZ")
            append_cyl(wbm, 0, 0.022, 0.012, dv.length, 5, tuple(mid), rot=tuple(rot))

        # Scalloped membrane fan
        v0 = wbm.verts.new((0.0, -0.04, 0.04))
        v1 = wbm.verts.new((side_sign * 0.42, -0.10, 0.13))
        v2 = wbm.verts.new((side_sign * 0.95, 0.02, 0.06))
        v3 = wbm.verts.new((side_sign * 0.72, 0.22, -0.12))
        v4 = wbm.verts.new((side_sign * 0.42, 0.26, -0.18))
        v5 = wbm.verts.new((side_sign * 0.08, 0.20, -0.12))
        for tri in ((v0, v1, v4), (v1, v2, v3), (v1, v3, v4), (v0, v4, v5)):
            try:
                f = wbm.faces.new(tri)
                f.material_index = 1
            except ValueError:
                pass

        wm = bpy.data.meshes.new(f"{name}_Mesh")
        wbm.to_mesh(wm)
        wbm.free()
        wo = bpy.data.objects.new(name, wm)
        bpy.context.collection.objects.link(wo)
        wo.data.materials.append(m_fur)
        wo.data.materials.append(m_wing)
        return wo

    wing_l = make_wing("Env_Mob_CrystalBat_WingL", -1.0)
    wing_r = make_wing("Env_Mob_CrystalBat_WingR", 1.0)
    return body_obj, wing_l, wing_r


def build_mob_lava_golem_parts():
    """Hulking Obsidian Magma Golem Body + Articulated Stone-Fist Left/Right Arms."""
    m_obsidian = make_pbr_mat(
        "Mat_Mob_Obsidian",
        base_color=(0.13, 0.11, 0.16, 1.0),
        roughness=0.58,
        metallic=0.28,
    )
    m_magma = make_pbr_mat(
        "Mat_Mob_MagmaCore",
        base_color=(1.0, 0.38, 0.06, 1.0),
        roughness=0.15,
        emissive=(1.0, 0.28, 0.02),
        emissive_strength=3.4,
    )
    m_gold_horn = make_pbr_mat(
        "Mat_Mob_BronzeHorn",
        base_color=(0.78, 0.48, 0.18, 1.0),
        roughness=0.32,
        metallic=0.68,
    )

    bm = bmesh.new()
    # Chunky stone legs & boots
    for sx in (-0.36, 0.36):
        append_box(bm, 0, (0.44, 0.52, 0.62), (sx, 0, 0.31), bevel_width=0.04)
        append_box(bm, 1, (0.46, 0.54, 0.08), (sx, 0, 0.52))
    # Armored pelvis + glowing magma waist core
    append_box(bm, 0, (1.02, 0.62, 0.34), (0, 0, 0.76), bevel_width=0.04)
    append_cyl(bm, 1, 0.42, 0.48, 0.28, 8, (0, 0, 1.04))
    # Broad volcanic boulder chest + magma fissure cross + shoulder pauldrons
    append_box(bm, 0, (1.42, 0.84, 0.82), (0, -0.04, 1.52), bevel_width=0.07)
    append_box(bm, 1, (0.34, 0.88, 0.48), (0, -0.05, 1.52), bevel_width=0.02)
    append_box(bm, 1, (0.86, 0.88, 0.16), (0, -0.05, 1.56), bevel_width=0.02)
    for sx in (-1.0, 1.0):
        append_box(
            bm,
            0,
            (0.56, 0.72, 0.48),
            (sx * 0.84, -0.02, 1.82),
            rot=(0, sx * 0.22, 0),
            bevel_width=0.05,
        )
        append_cyl(bm, 1, 0.12, 0.02, 0.34, 5, (sx * 0.92, 0.02, 2.15), rot=(0, sx * 0.28, 0))

    # Horned obsidian helm head + blazing visor slit facing -Y
    append_box(bm, 0, (0.64, 0.60, 0.56), (0, -0.10, 2.18), bevel_width=0.05)
    append_box(bm, 1, (0.46, 0.12, 0.13), (0, -0.38, 2.20))
    for sx in (-1.0, 1.0):
        append_cyl(
            bm,
            2,
            0.11,
            0.015,
            0.52,
            6,
            (sx * 0.42, -0.08, 2.48),
            rot=(-0.18, sx * 0.48, 0),
        )

    body_mesh = bpy.data.meshes.new("Env_Mob_LavaGolem_Body_Mesh")
    bm.to_mesh(body_mesh)
    bm.free()
    body_obj = bpy.data.objects.new("Env_Mob_LavaGolem_Body", body_mesh)
    bpy.context.collection.objects.link(body_obj)
    for m in (m_obsidian, m_magma, m_gold_horn):
        body_obj.data.materials.append(m)

    def make_arm(name, side_sign):
        abm = bmesh.new()
        # Shoulder pivot at (0,0,0), hanging downward to z = -1.25
        append_box(abm, 0, (0.38, 0.38, 0.56), (side_sign * 0.12, 0, -0.28), bevel_width=0.04)
        append_cyl(abm, 1, 0.16, 0.16, 0.14, 8, (side_sign * 0.14, 0, -0.60))
        # Massive stone gauntlet forearm & fist
        append_box(abm, 0, (0.52, 0.54, 0.64), (side_sign * 0.16, -0.06, -0.96), bevel_width=0.05)
        append_box(abm, 1, (0.56, 0.58, 0.12), (side_sign * 0.16, -0.06, -1.12))
        am = bpy.data.meshes.new(f"{name}_Mesh")
        abm.to_mesh(am)
        abm.free()
        ao = bpy.data.objects.new(name, am)
        bpy.context.collection.objects.link(ao)
        ao.data.materials.append(m_obsidian)
        ao.data.materials.append(m_magma)
        return ao

    arm_l = make_arm("Env_Mob_LavaGolem_ArmL", -1.0)
    arm_r = make_arm("Env_Mob_LavaGolem_ArmR", 1.0)
    return body_obj, arm_l, arm_r


def build_mob_void_wraith():
    """Hovering Phantom Void Wraith in tattered cowl with glowing cyan eyes, rune halo & twin plasma scythes."""
    bm = bmesh.new()
    m_cloak = make_pbr_mat("Mat_Mob_WraithCloak", base_color=(0.09, 0.07, 0.22, 1.0), roughness=0.72)
    m_void = make_pbr_mat("Mat_Mob_WraithVoid", base_color=(0.02, 0.02, 0.05, 1.0), roughness=0.9)
    m_cyan = make_pbr_mat(
        "Mat_Mob_WraithCyan",
        base_color=(0.25, 0.92, 1.0, 1.0),
        roughness=0.12,
        emissive=(0.12, 0.82, 1.0),
        emissive_strength=3.1,
    )

    # Billowing tapered phantom cloak + tattered bottom tendrils
    append_cyl(bm, 0, 0.58, 0.26, 1.38, 12, (0, 0, 0.92))
    for i in range(8):
        ang = i * (math.tau / 8.0)
        append_cyl(
            bm,
            0,
            0.16,
            0.015,
            0.52,
            5,
            (math.cos(ang) * 0.44, math.sin(ang) * 0.44, 0.24),
            rot=(math.sin(ang) * 0.22, -math.cos(ang) * 0.22, 0),
        )

    # Deep pointed hood + dark void face + piercing cyan eyes facing -Y
    append_sphere(bm, 0, 0.36, (0, -0.02, 1.72), scale=(1.0, 1.08, 1.12), u_segs=12, v_segs=10)
    append_cyl(bm, 0, 0.22, 0.02, 0.36, 6, (0, 0.16, 2.08), rot=(-0.45, 0, 0))
    append_sphere(bm, 1, 0.25, (0, -0.18, 1.70), u_segs=10, v_segs=8)
    for sx in (-0.10, 0.10):
        append_box(bm, 2, (0.11, 0.06, 0.05), (sx, -0.40, 1.73), rot=(0, sx * 2.2, 0))

    # Floating spectral rune halo behind shoulders + twin curved energy scythe arms
    for i in range(12):
        a0 = i * (math.tau / 12.0)
        append_box(
            bm,
            2,
            (0.18, 0.05, 0.05),
            (math.cos(a0) * 0.64, 0.22, 1.62 + math.sin(a0) * 0.64),
            rot=(0, -a0, 0),
        )

    for sx in (-1.0, 1.0):
        append_cyl(bm, 0, 0.09, 0.05, 0.68, 6, (sx * 0.56, -0.18, 1.28), rot=(0.55, sx * 0.55, 0))
        # Glowing plasma scythe blade sweeping forward
        append_box(
            bm,
            2,
            (0.06, 0.72, 0.16),
            (sx * 0.76, -0.54, 1.12),
            rot=(0.25, 0, -sx * 0.32),
            bevel_width=0.015,
        )

    mesh = bpy.data.meshes.new("Env_Mob_VoidWraith_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Mob_VoidWraith", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_cloak, m_void, m_cyan):
        obj.data.materials.append(m)
    return obj


def build_mob_rift_spider():
    """Neon Mech-Crystal Arachnid with faceted obsidian carapace, venom abdomen & 6 crimson legs."""
    bm = bmesh.new()
    m_shell = make_pbr_mat(
        "Mat_Mob_SpiderShell",
        base_color=(0.12, 0.10, 0.18, 1.0),
        roughness=0.28,
        metallic=0.62,
    )
    m_crimson = make_pbr_mat(
        "Mat_Mob_SpiderCrimson",
        base_color=(1.0, 0.16, 0.34, 1.0),
        roughness=0.14,
        emissive=(1.0, 0.08, 0.26),
        emissive_strength=2.8,
    )

    # Cephalothorax + glowing multi-eye cluster + pincers facing -Y
    append_sphere(bm, 0, 0.34, (0, -0.18, 0.42), scale=(1.05, 1.15, 0.72), u_segs=12, v_segs=8)
    # Raised faceted crystal abdomen at rear (+Y)
    append_sphere(bm, 0, 0.48, (0, 0.44, 0.56), scale=(1.0, 1.22, 0.86), u_segs=12, v_segs=10)
    append_cyl(bm, 1, 0.24, 0.04, 0.38, 6, (0, 0.48, 0.88), rot=(-0.3, 0, 0))
    for sx in (-1.0, 1.0):
        append_sphere(bm, 1, 0.06, (sx * 0.11, -0.52, 0.46), u_segs=8, v_segs=6)
        append_sphere(bm, 1, 0.04, (sx * 0.20, -0.48, 0.50), u_segs=6, v_segs=6)
        append_cyl(bm, 1, 0.045, 0.008, 0.26, 5, (sx * 0.14, -0.58, 0.34), rot=(1.2, 0, sx * 0.35))

    # 6 Articulated Spider Legs (3 per side) arching up from body and planting cleanly at z=0
    for sx in (-1.0, 1.0):
        for li, y_off in enumerate((-0.26, 0.04, 0.34)):
            yaw = (li - 1) * 0.38
            knee = Vector((sx * 0.68, y_off - sx * yaw * 0.15, 0.68))
            hip = Vector((sx * 0.28, y_off, 0.42))
            foot = Vector((sx * 0.96, y_off + (li - 1) * 0.26, 0.02))
            for p0, p1, r0, r1, m_idx in (
                (hip, knee, 0.055, 0.045, 0),
                (knee, foot, 0.045, 0.018, 1),
            ):
                mid = (p0 + p1) * 0.5
                dv = p1 - p0
                rot = dv.to_track_quat("Z", "Y").to_euler("XYZ")
                append_cyl(bm, m_idx, r0, r1, dv.length * 1.05, 6, tuple(mid), rot=tuple(rot))

    mesh = bpy.data.meshes.new("Env_Mob_RiftSpider_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Mob_RiftSpider", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_shell, m_crimson):
        obj.data.materials.append(m)
    return obj


# ============================================================================
# Expansion Pack 3D Assets: Pets, Hoverboards/Mounts, Workshop, Mailbox & Boss
# ============================================================================


def build_pet_cloud_puppy():
    """Fluffy white Cloud Puppy facing -Y (+Z in Three.js) with sky-blue ears, wagging puff tail & gold halo."""
    bm = bmesh.new()
    m_cloud = make_pbr_mat("Mat_Pet_CloudWhite", base_color=(0.98, 0.99, 1.0, 1.0), roughness=0.58)
    m_sky = make_pbr_mat(
        "Mat_Pet_SkyBlue",
        base_color=(0.45, 0.82, 0.99, 1.0),
        roughness=0.36,
        emissive=(0.25, 0.68, 0.98),
        emissive_strength=0.35,
    )
    m_dark = make_pbr_mat("Mat_Pet_DarkEye", base_color=(0.06, 0.08, 0.16, 1.0), roughness=0.10)
    m_pink = make_pbr_mat("Mat_Pet_RosyPink", base_color=(0.98, 0.48, 0.72, 1.0), roughness=0.42)
    m_gold = make_pbr_mat(
        "Mat_Pet_HaloGold",
        base_color=(1.0, 0.84, 0.24, 1.0),
        roughness=0.18,
        metallic=0.82,
        emissive=(1.0, 0.78, 0.18),
        emissive_strength=1.2,
    )

    # Plump cloud-puff torso + chest puff + head
    append_sphere(bm, 0, 0.23, (0, 0.06, 0.28), scale=(1.0, 1.22, 0.92), u_segs=14, v_segs=10)
    append_sphere(bm, 0, 0.16, (0, -0.14, 0.30), u_segs=12, v_segs=8)
    append_sphere(bm, 0, 0.21, (0, -0.22, 0.50), scale=(1.04, 0.98, 0.96), u_segs=14, v_segs=10)
    append_sphere(bm, 0, 0.10, (0, -0.39, 0.45), scale=(1.05, 0.85, 0.78), u_segs=10, v_segs=8)
    append_sphere(bm, 3, 0.038, (0, -0.47, 0.47), u_segs=8, v_segs=6)  # cute pink nose

    # Floppy sky-blue ears, glossy eyes & rosy cheeks
    for sx in (-1.0, 1.0):
        append_sphere(bm, 1, 0.095, (sx * 0.21, -0.18, 0.48), scale=(0.48, 0.78, 1.35), u_segs=10, v_segs=8)
        append_sphere(bm, 2, 0.036, (sx * 0.085, -0.405, 0.53), u_segs=8, v_segs=6)
        append_sphere(bm, 3, 0.028, (sx * 0.135, -0.385, 0.47), scale=(1.1, 0.45, 0.75), u_segs=8, v_segs=6)
        for fy in (-0.11, 0.18):
            append_sphere(bm, 0, 0.075, (sx * 0.12, fy, 0.08), scale=(0.95, 1.05, 1.0), u_segs=8, v_segs=6)

    # Fluffy cloud-puff tail + floating golden starlight halo ring above head
    append_sphere(bm, 1, 0.11, (0, 0.34, 0.38), u_segs=10, v_segs=8)
    for i in range(12):
        a0 = i * (math.tau / 12.0)
        a1 = (i + 1) * (math.tau / 12.0)
        p0 = Vector((math.cos(a0) * 0.13, -0.22 + math.sin(a0) * 0.13, 0.76))
        p1 = Vector((math.cos(a1) * 0.13, -0.22 + math.sin(a1) * 0.13, 0.76))
        mid = (p0 + p1) * 0.5
        dv = p1 - p0
        rot = dv.to_track_quat("Z", "Y").to_euler("XYZ")
        append_cyl(bm, 4, 0.018, 0.018, dv.length * 1.18, 6, tuple(mid), rot=tuple(rot))

    mesh = bpy.data.meshes.new("Env_Pet_CloudPuppy_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Pet_CloudPuppy", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_cloud, m_sky, m_dark, m_pink, m_gold):
        obj.data.materials.append(m)
    return obj


def build_pet_starlight_kitsune():
    """Sculpted 3-tailed Rose-Gold & Cream Starlight Kitsune spirit fox facing -Y."""
    bm = bmesh.new()
    m_coat = make_pbr_mat("Mat_Pet_KitsuneCoat", base_color=(0.99, 0.68, 0.34, 1.0), roughness=0.46)
    m_cream = make_pbr_mat("Mat_Pet_KitsuneCream", base_color=(1.0, 0.96, 0.90, 1.0), roughness=0.50)
    m_star = make_pbr_mat(
        "Mat_Pet_KitsuneStar",
        base_color=(0.98, 0.38, 0.78, 1.0),
        roughness=0.18,
        emissive=(0.98, 0.28, 0.72),
        emissive_strength=1.9,
    )
    m_eye = make_pbr_mat("Mat_Pet_DarkEye")

    # Slender fox body + cream chest ruff + pointed snout
    append_sphere(bm, 0, 0.21, (0, 0.04, 0.29), scale=(0.90, 1.32, 0.92), u_segs=14, v_segs=10)
    append_sphere(bm, 1, 0.15, (0, -0.16, 0.31), scale=(0.95, 0.95, 1.08), u_segs=10, v_segs=8)
    append_sphere(bm, 0, 0.19, (0, -0.24, 0.52), scale=(1.02, 0.98, 0.94), u_segs=14, v_segs=10)
    append_cyl(bm, 1, 0.095, 0.025, 0.20, 8, (0, -0.43, 0.47), rot=(1.35, 0, 0))
    append_sphere(bm, 2, 0.040, (0, -0.41, 0.60), u_segs=8, v_segs=6)  # glowing forehead gem

    # Tall pointed fox ears + eyes + 4 nimble paws
    for sx in (-1.0, 1.0):
        append_cyl(bm, 0, 0.075, 0.010, 0.24, 6, (sx * 0.13, -0.22, 0.74), rot=(0.10, -sx * 0.22, 0))
        append_cyl(bm, 2, 0.042, 0.006, 0.16, 5, (sx * 0.13, -0.24, 0.73), rot=(0.10, -sx * 0.22, 0))
        append_sphere(bm, 3, 0.034, (sx * 0.085, -0.40, 0.54), u_segs=8, v_segs=6)
        for fy in (-0.12, 0.18):
            append_cyl(bm, 0, 0.042, 0.032, 0.22, 8, (sx * 0.10, fy, 0.11))

    # 3 Fanning spirit tails with glowing starlight tips!
    for ti, spread in enumerate((-0.42, 0.0, 0.42)):
        tx = math.sin(spread) * 0.26
        tz = 0.44 + (0.06 if ti == 1 else 0.0)
        append_sphere(bm, 0, 0.13, (tx * 0.7, 0.38, tz), scale=(0.78, 1.55, 0.85), u_segs=10, v_segs=8)
        append_sphere(bm, 2, 0.075, (tx * 1.15, 0.56, tz + 0.09), u_segs=8, v_segs=6)

    mesh = bpy.data.meshes.new("Env_Pet_StarlightKitsune_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Pet_StarlightKitsune", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_coat, m_cream, m_star, m_eye):
        obj.data.materials.append(m)
    return obj


def build_pet_shadow_kitty():
    """Obsidian-velvet mystic Shadow Kitty with glowing neon-violet eyes & golden crescent collar."""
    bm = bmesh.new()
    m_fur = make_pbr_mat("Mat_Pet_ShadowFur", base_color=(0.08, 0.07, 0.15, 1.0), roughness=0.45, metallic=0.12)
    m_violet = make_pbr_mat(
        "Mat_Pet_KittyViolet",
        base_color=(0.76, 0.32, 1.0, 1.0),
        roughness=0.12,
        emissive=(0.68, 0.22, 1.0),
        emissive_strength=2.6,
    )
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")

    append_sphere(bm, 0, 0.19, (0, 0.05, 0.27), scale=(0.88, 1.24, 0.94), u_segs=14, v_segs=10)
    append_sphere(bm, 0, 0.19, (0, -0.20, 0.50), scale=(1.06, 0.95, 0.96), u_segs=14, v_segs=10)
    # Collar + golden moon charm
    append_cyl(bm, 1, 0.135, 0.135, 0.045, 14, (0, -0.17, 0.37), rot=(0.25, 0, 0))
    append_sphere(bm, 2, 0.042, (0, -0.30, 0.34), u_segs=8, v_segs=6)

    for sx in (-1.0, 1.0):
        append_cyl(bm, 0, 0.072, 0.008, 0.19, 5, (sx * 0.12, -0.19, 0.70), rot=(0.06, -sx * 0.18, 0))
        append_cyl(bm, 1, 0.038, 0.005, 0.12, 4, (sx * 0.12, -0.21, 0.69), rot=(0.06, -sx * 0.18, 0))
        append_sphere(bm, 1, 0.042, (sx * 0.082, -0.375, 0.52), scale=(1.15, 0.55, 0.85), u_segs=8, v_segs=6)
        for fy in (-0.10, 0.17):
            append_cyl(bm, 0, 0.038, 0.030, 0.20, 8, (sx * 0.09, fy, 0.10))

    # Graceful S-curved mystic cat tail with glowing violet tip
    for i, (ty, tz) in enumerate([(0.26, 0.34), (0.36, 0.46), (0.38, 0.60)]):
        append_sphere(bm, 1 if i == 2 else 0, 0.048, (0.04 * i, ty, tz), u_segs=8, v_segs=6)

    mesh = bpy.data.meshes.new("Env_Pet_ShadowKitty_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Pet_ShadowKitty", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_fur, m_violet, m_gold):
        obj.data.materials.append(m)
    return obj


def build_pet_baby_dragon_parts():
    """Articulated Prismatic Baby Crystal Dragon (Body + WingL + WingR)."""
    m_scale = make_pbr_mat(
        "Mat_Pet_DragonScale",
        base_color=(0.24, 0.78, 0.98, 1.0),
        roughness=0.22,
        metallic=0.38,
        emissive=(0.14, 0.45, 0.85),
        emissive_strength=0.45,
    )
    m_crystal = make_pbr_mat(
        "Mat_Pet_DragonCrystal",
        base_color=(0.98, 0.36, 0.82, 1.0),
        roughness=0.14,
        emissive=(0.95, 0.25, 0.78),
        emissive_strength=1.8,
        double_sided=True,
    )
    m_belly = make_pbr_mat("Mat_Pet_DragonBelly", base_color=(0.98, 0.95, 0.82, 1.0), roughness=0.35)
    m_eye = make_pbr_mat("Mat_Pet_HaloGold")

    bm = bmesh.new()
    append_sphere(bm, 0, 0.20, (0, 0.04, 0.38), scale=(0.95, 1.18, 1.05), u_segs=14, v_segs=10)
    append_sphere(bm, 2, 0.14, (0, -0.09, 0.36), scale=(0.88, 0.65, 0.98), u_segs=10, v_segs=8)
    append_sphere(bm, 0, 0.18, (0, -0.18, 0.62), scale=(1.0, 1.08, 0.95), u_segs=14, v_segs=10)
    append_sphere(bm, 0, 0.10, (0, -0.35, 0.58), scale=(0.95, 1.05, 0.75), u_segs=10, v_segs=8)

    for sx in (-1.0, 1.0):
        append_cyl(bm, 1, 0.042, 0.006, 0.20, 5, (sx * 0.10, -0.10, 0.80), rot=(-0.38, -sx * 0.24, 0))
        append_sphere(bm, 3, 0.038, (sx * 0.095, -0.31, 0.65), u_segs=8, v_segs=6)
        append_sphere(bm, 0, 0.065, (sx * 0.11, 0.02, 0.18), u_segs=8, v_segs=6)

    # Spiked crystal tail
    append_cyl(bm, 0, 0.09, 0.02, 0.36, 8, (0, 0.34, 0.30), rot=(-1.15, 0, 0))
    append_cyl(bm, 1, 0.065, 0.008, 0.14, 4, (0, 0.52, 0.24), rot=(-1.35, 0, 0))
    for i in range(3):
        append_cyl(bm, 1, 0.038, 0.006, 0.12, 4, (0, 0.02 + i * 0.11, 0.58 - i * 0.08), rot=(-0.45, 0, 0))

    b_mesh = bpy.data.meshes.new("Env_Pet_BabyDragon_Body_Mesh")
    bm.to_mesh(b_mesh)
    bm.free()
    b_obj = bpy.data.objects.new("Env_Pet_BabyDragon_Body", b_mesh)
    bpy.context.collection.objects.link(b_obj)
    for m in (m_scale, m_crystal, m_belly, m_eye):
        b_obj.data.materials.append(m)

    def make_wing(name, side_sign):
        wbm = bmesh.new()
        append_box(wbm, 0, (0.38, 0.04, 0.04), (side_sign * 0.19, 0.0, 0.06), rot=(0, -side_sign * 0.28, 0))
        append_box(
            wbm,
            1,
            (0.38, 0.24, 0.02),
            (side_sign * 0.22, 0.08, 0.02),
            rot=(0.18, -side_sign * 0.22, 0),
        )
        wm = bpy.data.meshes.new(f"{name}_Mesh")
        wbm.to_mesh(wm)
        wbm.free()
        wo = bpy.data.objects.new(name, wm)
        bpy.context.collection.objects.link(wo)
        wo.data.materials.append(m_scale)
        wo.data.materials.append(m_crystal)
        return wo

    return b_obj, make_wing("Env_Pet_BabyDragon_WingL", -1.0), make_wing("Env_Pet_BabyDragon_WingR", 1.0)


def build_mount_hoverboard():
    """Sleek Anti-Grav Starlight Hoverboard (1.28m long along Y, 0.48m wide along X)."""
    bm = bmesh.new()
    m_deck = make_pbr_mat("Mat_Mount_Deck", base_color=(0.10, 0.14, 0.28, 1.0), roughness=0.24, metallic=0.68)
    m_neon = make_pbr_mat(
        "Mat_Mount_CyanGlow",
        base_color=(0.18, 0.92, 1.0, 1.0),
        roughness=0.12,
        emissive=(0.12, 0.88, 1.0),
        emissive_strength=2.6,
    )
    m_pink = make_pbr_mat(
        "Mat_Mount_PinkGlow",
        base_color=(0.98, 0.34, 0.76, 1.0),
        roughness=0.18,
        emissive=(0.96, 0.22, 0.68),
        emissive_strength=1.8,
    )

    append_box(bm, 0, (0.46, 1.24, 0.07), (0, 0, 0.10), bevel_width=0.028)
    append_box(bm, 2, (0.34, 0.86, 0.02), (0, 0, 0.14), bevel_width=0.01)
    for sy in (-0.32, 0.32):
        append_box(bm, 0, (0.30, 0.22, 0.018), (0, sy, 0.15))
        append_cyl(bm, 1, 0.15, 0.12, 0.05, 16, (0, sy, 0.055))
    for sx in (-0.235, 0.235):
        append_box(bm, 1, (0.025, 1.12, 0.03), (sx, 0, 0.10))

    mesh = bpy.data.meshes.new("Env_Mount_Hoverboard_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Mount_Hoverboard", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_deck, m_neon, m_pink):
        obj.data.materials.append(m)
    return obj


def build_mount_starlight_skates():
    """Twin Winged Roller-Thruster Starlight Skates spaced for avatar feet (±0.22 X)."""
    bm = bmesh.new()
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")
    m_neon = make_pbr_mat("Mat_Mount_CyanGlow")
    m_pink = make_pbr_mat("Mat_Mount_PinkGlow")

    for sx in (-0.22, 0.22):
        append_box(bm, 0, (0.18, 0.42, 0.06), (sx, -0.02, 0.07), bevel_width=0.015)
        for wy in (-0.13, 0.11):
            append_cyl(bm, 1, 0.055, 0.055, 0.20, 12, (sx, wy, 0.055), rot=(0, math.pi * 0.5, 0))
        append_box(
            bm,
            2,
            (0.03, 0.22, 0.14),
            (sx + (0.11 if sx > 0 else -0.11), 0.08, 0.16),
            rot=(0.35, 0, 0),
        )

    mesh = bpy.data.meshes.new("Env_Mount_StarlightSkates_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Mount_StarlightSkates", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_gold, m_neon, m_pink):
        obj.data.materials.append(m)
    return obj


def build_mount_comet_surfer():
    """Royal-Gold & Cosmic-Violet Comet Surfer Glider Board with swept wings (1.46m long)."""
    bm = bmesh.new()
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")
    m_violet = make_pbr_mat("Mat_Pet_KittyViolet")
    m_cyan = make_pbr_mat("Mat_Mount_CyanGlow")

    append_sphere(bm, 0, 0.28, (0, 0, 0.11), scale=(0.92, 2.55, 0.22), u_segs=16, v_segs=10)
    for sx in (-1.0, 1.0):
        append_box(
            bm,
            1,
            (0.32, 0.48, 0.035),
            (sx * 0.34, 0.18, 0.12),
            rot=(0.08, -sx * 0.15, -sx * 0.32),
            bevel_width=0.01,
        )
    append_sphere(bm, 2, 0.09, (0, -0.42, 0.15), scale=(1.0, 1.3, 0.55), u_segs=10, v_segs=8)
    append_cyl(bm, 2, 0.11, 0.06, 0.24, 12, (0, 0.68, 0.11), rot=(math.pi * 0.5, 0, 0))

    mesh = bpy.data.meshes.new("Env_Mount_CometSurfer_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Mount_CometSurfer", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_gold, m_violet, m_cyan):
        obj.data.materials.append(m)
    return obj


def build_crafting_bench():
    """Starlight Workshop Crafting Bench + Crystal Anvil + Glowing Arcane Forge Spire."""
    bm = bmesh.new()
    m_wood = make_pbr_mat("Mat_Bench_Wood", base_color=(0.38, 0.22, 0.12, 1.0), roughness=0.62)
    m_stone = make_pbr_mat("Mat_Bench_Obsidian", base_color=(0.14, 0.16, 0.26, 1.0), roughness=0.35, metallic=0.58)
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")
    m_cyan = make_pbr_mat("Mat_Mount_CyanGlow")
    m_pink = make_pbr_mat("Mat_Mount_PinkGlow")

    # Heavy timber workbench table (2.1m x 1.0m x 0.92m)
    append_box(bm, 0, (2.10, 1.00, 0.12), (0, 0, 0.88), bevel_width=0.025)
    append_box(bm, 0, (1.88, 0.82, 0.06), (0, 0, 0.26))
    for lx in (-0.88, 0.88):
        for ly in (-0.38, 0.38):
            append_box(bm, 0, (0.14, 0.14, 0.84), (lx, ly, 0.42))

    # Left side: Starlight Crystal Anvil + Golden Smithing Hammer
    append_box(bm, 1, (0.48, 0.32, 0.16), (-0.56, -0.04, 1.02), bevel_width=0.02)
    append_box(bm, 1, (0.64, 0.26, 0.14), (-0.56, -0.04, 1.18), bevel_width=0.02)
    append_cyl(bm, 3, 0.11, 0.02, 0.24, 6, (-0.88, -0.04, 1.18), rot=(0, -math.pi * 0.5, 0))
    append_cyl(bm, 0, 0.024, 0.024, 0.34, 8, (-0.42, -0.22, 1.28), rot=(0, 0.95, 0.35))
    append_box(bm, 2, (0.12, 0.08, 0.08), (-0.29, -0.16, 1.35), rot=(0, 0.2, 0.35))

    # Center: Floating Arcane Rune Ring + Prism Crystal Core
    append_cyl(bm, 2, 0.28, 0.32, 0.05, 16, (0.08, 0.02, 0.96))
    append_cyl(bm, 3, 0.15, 0.01, 0.36, 6, (0.08, 0.02, 1.38))
    append_cyl(bm, 3, 0.15, 0.01, 0.22, 6, (0.08, 0.02, 1.10), rot=(math.pi, 0, 0))

    # Right side: Glowing Alchemist Potion Flasks
    append_sphere(bm, 4, 0.11, (0.64, -0.10, 1.05), u_segs=12, v_segs=10)
    append_cyl(bm, 2, 0.04, 0.05, 0.09, 10, (0.64, -0.10, 1.18))
    append_sphere(bm, 3, 0.085, (0.82, 0.12, 1.02), u_segs=10, v_segs=8)

    mesh = bpy.data.meshes.new("Env_Crafting_Bench_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Crafting_Bench", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_wood, m_stone, m_gold, m_cyan, m_pink):
        obj.data.materials.append(m)
    return obj


def build_plot_mailbox():
    """Neighborhood Home Mailbox with Doorbell Button (🔔) facing -Y (+Z in Three.js)."""
    bm = bmesh.new()
    m_post = make_pbr_mat("Mat_Mailbox_Post", base_color=(0.95, 0.96, 0.98, 1.0), roughness=0.36)
    m_box = make_pbr_mat("Mat_Mailbox_Body", base_color=(0.24, 0.68, 0.96, 1.0), roughness=0.30, metallic=0.25)
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")
    m_bell = make_pbr_mat(
        "Mat_Mailbox_Doorbell",
        base_color=(1.0, 0.32, 0.58, 1.0),
        roughness=0.16,
        emissive=(1.0, 0.24, 0.52),
        emissive_strength=2.0,
    )

    append_box(bm, 0, (0.28, 0.28, 0.08), (0, 0, 0.04), bevel_width=0.015)
    append_box(bm, 0, (0.12, 0.12, 1.06), (0, 0, 0.56), bevel_width=0.01)
    # Arched mailbox chest
    append_box(bm, 1, (0.36, 0.52, 0.24), (0, -0.02, 1.18), bevel_width=0.025)
    append_cyl(bm, 1, 0.18, 0.18, 0.52, 14, (0, -0.02, 1.30), rot=(math.pi * 0.5, 0, 0))
    # Golden mail slot trim + Glowing Ringable Doorbell Button on front (-Y)!
    append_box(bm, 2, (0.24, 0.03, 0.05), (0, -0.285, 1.30))
    append_cyl(bm, 2, 0.065, 0.065, 0.03, 14, (0, -0.285, 1.15), rot=(math.pi * 0.5, 0, 0))
    append_sphere(bm, 3, 0.045, (0, -0.30, 1.15), u_segs=10, v_segs=8)
    # Raised parcel flag on right side
    append_box(bm, 3, (0.025, 0.05, 0.24), (0.20, -0.08, 1.38))
    append_box(bm, 3, (0.025, 0.12, 0.08), (0.20, -0.12, 1.48))

    mesh = bpy.data.meshes.new("Env_Plot_Mailbox_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Plot_Mailbox", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_post, m_box, m_gold, m_bell):
        obj.data.materials.append(m)
    return obj


def build_obby_gate():
    """Sleek Starlight Speedrun Laser Gate Arch (3.8m span x 3.5m tall) for Sky & Pro Obby."""
    bm = bmesh.new()
    m_pearl = make_pbr_mat("Mat_Gate_Pearl", base_color=(0.95, 0.96, 1.0, 1.0), roughness=0.26, metallic=0.35)
    m_slate = make_pbr_mat("Mat_Gate_Slate", base_color=(0.11, 0.14, 0.24, 1.0), roughness=0.32, metallic=0.68)
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")
    m_cyan = make_pbr_mat(
        "Mat_Gate_Emitter",
        base_color=(0.22, 0.88, 1.0, 1.0),
        roughness=0.12,
        emissive=(0.18, 0.78, 1.0),
        emissive_strength=2.6,
    )

    for sx in (-1.0, 1.0):
        px = sx * 1.78
        append_cyl(bm, 1, 0.30, 0.36, 0.22, 12, (px, 0, 0.11))
        append_cyl(bm, 0, 0.16, 0.20, 3.22, 12, (px, 0, 1.80))
        append_cyl(bm, 2, 0.22, 0.22, 0.10, 12, (px, 0, 1.10))
        append_cyl(bm, 2, 0.22, 0.22, 0.10, 12, (px, 0, 2.55))
        append_sphere(bm, 3, 0.16, (px, 0, 3.52), u_segs=10, v_segs=8)

    # Overhead arch header beam + gold crest
    append_box(bm, 1, (3.92, 0.34, 0.28), (0, 0, 3.36), bevel_width=0.03)
    append_box(bm, 2, (3.98, 0.38, 0.07), (0, 0, 3.53), bevel_width=0.015)
    append_sphere(bm, 3, 0.20, (0, 0, 3.72), scale=(1.0, 0.55, 1.15), u_segs=10, v_segs=8)

    mesh = bpy.data.meshes.new("Env_Obby_Gate_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Obby_Gate", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_pearl, m_slate, m_gold, m_cyan):
        obj.data.materials.append(m)
    return obj


def build_leaderboard_board():
    """Twin-pedestal Hall of Fame Top-5 Leaderboard Frame facing -Y (+Z in Three.js Y-up)."""
    bm = bmesh.new()
    m_slate = make_pbr_mat("Mat_Gate_Slate")
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")
    m_glow = make_pbr_mat("Mat_Gate_Emitter")

    for sx in (-1.0, 1.0):
        px = sx * 1.56
        append_box(bm, 0, (0.42, 0.48, 0.18), (px, 0, 0.09), bevel_width=0.02)
        append_cyl(bm, 0, 0.11, 0.13, 3.25, 10, (px, 0, 1.72))
        append_sphere(bm, 1, 0.15, (px, 0, 3.42), u_segs=10, v_segs=8)

    # Beveled scoreboard housing centered at Z=2.15 so screenMesh (3.08 x 2.12 at y=2.15, z=+0.14) sits flush
    append_box(bm, 0, (3.44, 0.20, 2.38), (0, 0.02, 2.15), bevel_width=0.03)
    # Gold outer bezel frame bars
    append_box(bm, 1, (3.50, 0.24, 0.10), (0, 0.0, 3.34), bevel_width=0.015)
    append_box(bm, 1, (3.50, 0.24, 0.10), (0, 0.0, 0.96), bevel_width=0.015)
    for sx in (-1.0, 1.0):
        append_box(bm, 1, (0.10, 0.24, 2.46), (sx * 1.70, 0.0, 2.15), bevel_width=0.015)
    append_sphere(bm, 2, 0.16, (0, -0.06, 3.52), u_segs=10, v_segs=8)

    mesh = bpy.data.meshes.new("Env_Leaderboard_Board_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Leaderboard_Board", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_slate, m_gold, m_glow):
        obj.data.materials.append(m)
    return obj


def build_boss_altar():
    """Obsidian Dragon Summoning Altar for the North Island of Shadow Rift."""
    bm = bmesh.new()
    m_obs = make_pbr_mat("Mat_Boss_Obsidian", base_color=(0.09, 0.08, 0.14, 1.0), roughness=0.28, metallic=0.62)
    m_magma = make_pbr_mat(
        "Mat_Boss_Magma",
        base_color=(1.0, 0.26, 0.08, 1.0),
        roughness=0.14,
        emissive=(1.0, 0.18, 0.05),
        emissive_strength=3.0,
    )
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")

    append_cyl(bm, 0, 1.65, 1.85, 0.24, 8, (0, 0, 0.12))
    append_cyl(bm, 0, 1.15, 1.35, 0.28, 8, (0, 0, 0.38))
    append_cyl(bm, 0, 0.48, 0.62, 0.92, 6, (0, 0, 0.96))
    append_cyl(bm, 2, 0.62, 0.48, 0.12, 8, (0, 0, 1.46))
    # 4 Curved obsidian dragon-horn pillars surrounding floating magma summoning orb
    for i in range(4):
        ang = (i + 0.5) * (math.tau / 4.0)
        px, py = math.cos(ang) * 1.18, math.sin(ang) * 1.18
        append_cyl(bm, 0, 0.18, 0.03, 1.55, 6, (px, py, 1.12), rot=(py * 0.22, -px * 0.22, 0))
    append_sphere(bm, 1, 0.30, (0, 0, 1.88), u_segs=14, v_segs=12)

    mesh = bpy.data.meshes.new("Env_Boss_Altar_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new("Env_Boss_Altar", mesh)
    bpy.context.collection.objects.link(obj)
    for m in (m_obs, m_magma, m_gold):
        obj.data.materials.append(m)
    return obj


def build_boss_obsidian_dragon_parts():
    """Massive Co-Op Obsidian Dragon Boss (Body + Articulated WingL + WingR) facing -Y."""
    m_obs = make_pbr_mat("Mat_Boss_Obsidian")
    m_magma = make_pbr_mat("Mat_Boss_Magma")
    m_violet = make_pbr_mat(
        "Mat_Boss_VoidMembrane",
        base_color=(0.58, 0.12, 0.88, 1.0),
        roughness=0.22,
        emissive=(0.82, 0.14, 0.42),
        emissive_strength=1.5,
        double_sided=True,
    )
    m_gold = make_pbr_mat("Mat_Pet_HaloGold")

    bm = bmesh.new()
    # Armored draconic torso + glowing molten magma chest core + muscular legs
    append_sphere(bm, 0, 0.98, (0, 0.18, 1.52), scale=(0.96, 1.32, 1.15), u_segs=16, v_segs=12)
    append_sphere(bm, 1, 0.56, (0, -0.58, 1.56), scale=(0.86, 0.52, 0.96), u_segs=12, v_segs=10)
    # Sinuous armored neck + fierce horned dragon skull & lower jaw facing -Y
    append_cyl(bm, 0, 0.48, 0.36, 1.12, 10, (0, -0.52, 2.38), rot=(0.52, 0, 0))
    append_box(bm, 0, (0.74, 0.98, 0.52), (0, -1.02, 2.86), rot=(0.14, 0, 0), bevel_width=0.05)
    append_box(bm, 0, (0.56, 0.78, 0.24), (0, -1.12, 2.52), rot=(-0.22, 0, 0), bevel_width=0.03)
    append_sphere(bm, 1, 0.22, (0, -1.02, 2.66), u_segs=10, v_segs=8)  # fiery maw glow

    # Swept crown horns, glowing magma eyes, stout legs & claws
    for sx in (-1.0, 1.0):
        append_cyl(bm, 3, 0.14, 0.015, 1.05, 6, (sx * 0.34, -0.52, 3.32), rot=(-0.68, -sx * 0.32, 0))
        append_sphere(bm, 1, 0.095, (sx * 0.28, -1.38, 2.94), u_segs=8, v_segs=8)
        append_cyl(bm, 0, 0.36, 0.26, 1.32, 10, (sx * 0.68, -0.05, 0.66))
        append_box(bm, 0, (0.52, 0.68, 0.22), (sx * 0.68, -0.18, 0.11), bevel_width=0.03)

    # Heavy spiked tail sweeping +Y + dorsal magma crystal ridges
    append_cyl(bm, 0, 0.52, 0.12, 2.15, 10, (0, 1.65, 0.92), rot=(-1.18, 0, 0))
    append_cyl(bm, 1, 0.26, 0.02, 0.58, 4, (0, 2.68, 0.52), rot=(-1.32, 0, 0))
    for i in range(5):
        append_cyl(bm, 1, 0.14, 0.015, 0.44, 5, (0, -0.25 + i * 0.42, 2.55 - i * 0.28), rot=(-0.48, 0, 0))

    b_mesh = bpy.data.meshes.new("Env_Boss_ObsidianDragon_Body_Mesh")
    bm.to_mesh(b_mesh)
    bm.free()
    b_obj = bpy.data.objects.new("Env_Boss_ObsidianDragon_Body", b_mesh)
    bpy.context.collection.objects.link(b_obj)
    for m in (m_obs, m_magma, m_violet, m_gold):
        b_obj.data.materials.append(m)

    def make_boss_wing(name, side_sign):
        wbm = bmesh.new()
        append_cyl(
            wbm,
            0,
            0.12,
            0.06,
            2.25,
            8,
            (side_sign * 1.08, -0.08, 0.32),
            rot=(0, side_sign * (math.pi * 0.42), 0),
        )
        append_cyl(wbm, 3, 0.07, 0.01, 0.38, 5, (side_sign * 1.05, -0.28, 0.56), rot=(0.65, 0, 0))
        for fx, fy, fz in [(2.15, 0.12, 0.22), (1.72, 0.58, -0.22), (1.05, 0.72, -0.42)]:
            p0 = Vector((side_sign * 0.18, 0.0, 0.12))
            p1 = Vector((side_sign * fx, fy, fz))
            mid = (p0 + p1) * 0.5
            dv = p1 - p0
            rot = dv.to_track_quat("Z", "Y").to_euler("XYZ")
            append_cyl(wbm, 0, 0.055, 0.018, dv.length, 6, tuple(mid), rot=tuple(rot))
        append_box(
            wbm,
            2,
            (1.95, 0.92, 0.04),
            (side_sign * 1.15, 0.34, 0.02),
            rot=(0.24, -side_sign * 0.22, 0),
        )
        wm = bpy.data.meshes.new(f"{name}_Mesh")
        wbm.to_mesh(wm)
        wbm.free()
        wo = bpy.data.objects.new(name, wm)
        bpy.context.collection.objects.link(wo)
        for m in (m_obs, m_magma, m_violet, m_gold):
            wo.data.materials.append(m)
        return wo

    return (
        b_obj,
        make_boss_wing("Env_Boss_ObsidianDragon_WingL", -1.0),
        make_boss_wing("Env_Boss_ObsidianDragon_WingR", 1.0),
    )


def build_cc0_prop(name, src_subdir, pred, target_polys, target_height=None, target_width=None, anchor_z="bottom", diff_tex=None, mat_override=None):
    bp = SRC_DIR / src_subdir / f"{src_subdir}.blend"
    objs = append_objects_from_blend(bp, pred)
    obj = join_and_decimate(objs, name, target_polys=target_polys)
    normalize_origin_and_scale(obj, target_height=target_height, target_width=target_width, anchor_z=anchor_z)
    if mat_override is not None:
        obj.data.materials.clear()
        obj.data.materials.append(mat_override)
    elif diff_tex and diff_tex.exists():
        mat = make_pbr_mat(f"Mat_{name}", roughness=0.52, diff_path=diff_tex, max_tex=512)
        obj.data.materials.clear()
        obj.data.materials.append(mat)
    for p in obj.data.polygons:
        p.use_smooth = True
    return obj


def render_studio_preview(objects_to_show, out_path: Path, cam_loc, cam_target, spacing=0.0):
    """Renders an 960x560 EEVEE Next studio preview of the selected assets."""
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 960
    scene.render.resolution_y = 560
    scene.render.film_transparent = False
    scene.render.image_settings.file_format = "PNG"
    scene.render.filepath = str(out_path)

    # World sky lighting
    world = bpy.data.worlds.new("PreviewWorld")
    scene.world = world
    world.use_nodes = True
    bg = world.node_tree.nodes.get("Background")
    if bg:
        bg.inputs[0].default_value = (0.58, 0.78, 0.96, 1.0)
        bg.inputs[1].default_value = 0.95

    # Hide all non-target meshes
    saved_pos = {}
    for o in bpy.data.objects:
        if o.type == "MESH":
            o.hide_render = o not in objects_to_show
            saved_pos[o.name] = o.location.copy()

    if spacing > 0:
        start_x = -0.5 * spacing * (len(objects_to_show) - 1)
        for idx, o in enumerate(objects_to_show):
            o.location = Vector((start_x + idx * spacing, 0, 0))

    # Studio floor + 3-point lights + camera
    bpy.ops.mesh.primitive_plane_add(size=80, location=(0, 0, -0.02))
    floor = bpy.context.active_object
    floor.data.materials.append(make_pbr_mat("Mat_PreviewFloor", base_color=(0.35, 0.72, 0.42, 1.0), roughness=0.85))

    sun_data = bpy.data.lights.new("SunKey", type="SUN")
    sun_data.energy = 3.2
    sun = bpy.data.objects.new("SunKey", sun_data)
    sun.rotation_euler = Euler((0.75, 0.2, 0.65), "XYZ")
    bpy.context.collection.objects.link(sun)

    fill_data = bpy.data.lights.new("FillRim", type="SUN")
    fill_data.energy = 1.1
    fill_data.color = (1.0, 0.9, 0.96)
    fill = bpy.data.objects.new("FillRim", fill_data)
    fill.rotation_euler = Euler((0.9, -0.4, -2.1), "XYZ")
    bpy.context.collection.objects.link(fill)

    cam_data = bpy.data.cameras.new("PreviewCam")
    cam_data.lens = 38
    cam = bpy.data.objects.new("PreviewCam", cam_data)
    cam.location = Vector(cam_loc)
    direction = Vector(cam_target) - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    bpy.context.collection.objects.link(cam)
    scene.camera = cam

    bpy.ops.render.render(write_still=True)

    # Restore positions & remove preview rig objects
    for o in (floor, sun, fill, cam):
        bpy.data.objects.remove(o, do_unlink=True)
    for o in bpy.data.objects:
        if o.type == "MESH" and o.name in saved_pos:
            o.hide_render = False
            o.location = saved_pos[o.name]


def main():
    clear_scene()
    built = {}

    # 1. Realistic CC0 Botanical Trees (3 distinct varieties)
    built["Env_Tree_Blossom"] = build_botanical_tree(
        name="Env_Tree_Blossom",
        src_subdir="jacaranda_tree",
        trunk_pred=lambda n: n == "jacaranda_tree_trunk_LOD1",
        leaf_pred=lambda n: n in ("jacaranda_tree_leaves_c_LOD1", "jacaranda_tree_leaves_d_LOD1"),
        bark_tex=SRC_DIR / "jacaranda_tree/textures/jacaranda_tree_trunk_diff_1k.png",
        leaf_diff=SRC_DIR / "jacaranda_tree/textures/jacaranda_tree_leaves_diff_1k.png",
        leaf_alpha=SRC_DIR / "jacaranda_tree/textures/jacaranda_tree_leaves_alpha_1k.png",
        target_height=6.8,
        trunk_target_polys=1250,
        canopy_clusters=82,
        cluster_scale_range=(1.1, 1.72),
        leaf_tint_rgba=(1.0, 0.48, 0.78, 0.72),  # Vibrant Cherry Blossom Pink!
        seed=101,
    )

    built["Env_Tree_Lilac"] = build_botanical_tree(
        name="Env_Tree_Lilac",
        src_subdir="jacaranda_tree",
        trunk_pred=lambda n: n == "jacaranda_tree_trunk_LOD1",
        leaf_pred=lambda n: n in ("jacaranda_tree_leaves_c_LOD1", "jacaranda_tree_leaves_e_LOD1"),
        bark_tex=SRC_DIR / "jacaranda_tree/textures/jacaranda_tree_trunk_diff_1k.png",
        leaf_diff=SRC_DIR / "jacaranda_tree/textures/jacaranda_tree_leaves_diff_1k.png",
        leaf_alpha=SRC_DIR / "jacaranda_tree/textures/jacaranda_tree_leaves_alpha_1k.png",
        target_height=6.5,
        trunk_target_polys=1200,
        canopy_clusters=80,
        cluster_scale_range=(1.05, 1.65),
        leaf_tint_rgba=(0.66, 0.44, 1.0, 0.58),  # Rich Starlight Jacaranda Lavender!
        seed=202,
    )

    built["Env_Tree_Emerald"] = build_botanical_tree(
        name="Env_Tree_Emerald",
        src_subdir="tree_small_02",
        trunk_pred=lambda n: n == "tree_small_02_trunk",
        leaf_pred=lambda n: n == "tree_small_02_leaves_d_LOD1",
        bark_tex=SRC_DIR / "tree_small_02/textures/tree_small_02_diff_1k.jpg",
        leaf_diff=SRC_DIR / "tree_small_02/textures/tree_small_02_leaves_diff_1k.png",
        leaf_alpha=SRC_DIR / "tree_small_02/textures/tree_small_02_leaves_alpha_1k.png",
        target_height=6.4,
        trunk_target_polys=1250,
        canopy_clusters=72,
        cluster_scale_range=(1.05, 1.6),
        leaf_tint_rgba=(0.16, 0.72, 0.28, 0.3),  # Vibrant Lush Emerald Park Tree!
        seed=303,
    )

    # 2. Realistic DTI Fashion Runway Stage Props (PolyHaven CC0 + Couture Assemblies)
    built["Env_DTI_Spotlight"] = build_cc0_prop(
        name="Env_DTI_Spotlight",
        src_subdir="flood_lights",
        pred=lambda n: n.startswith("flood_lights_standing_single"),
        target_polys=1250,
        target_height=2.35,
        anchor_z="bottom",
        mat_override=make_pbr_mat(
            "Mat_DTI_Spotlight",
            base_color=(0.85, 0.75, 0.35, 1.0),
            roughness=0.35,
            metallic=0.45,
            emissive=(1.0, 0.94, 0.75),
            emissive_strength=0.18,
            diff_path=SRC_DIR / "flood_lights/textures/flood_lights_diff_1k.jpg",
            max_tex=512,
        ),
    )

    built["Env_DTI_Chandelier"] = build_cc0_prop(
        name="Env_DTI_Chandelier",
        src_subdir="Chandelier_01",
        pred=lambda n: n == "Chandelier_01",
        target_polys=1950,
        target_height=1.55,
        anchor_z="top",
        mat_override=make_pbr_mat(
            "Mat_DTI_Chandelier",
            base_color=(0.98, 0.84, 0.42, 1.0),
            roughness=0.22,
            metallic=0.78,
            emissive=(1.0, 0.92, 0.72),
            emissive_strength=0.45,
            diff_path=SRC_DIR / "Chandelier_01/textures/Chandelier_01_diff_1k.jpg",
            max_tex=512,
        ),
    )

    built["Env_DTI_Bench"] = build_cc0_prop(
        name="Env_DTI_Bench",
        src_subdir="painted_wooden_bench",
        pred=lambda n: n == "painted_wooden_bench",
        target_polys=374,
        target_width=2.25,
        anchor_z="bottom",
        diff_tex=SRC_DIR / "painted_wooden_bench/textures/painted_wooden_bench_diff_1k.jpg",
    )

    built["Env_DTI_Lamp"] = build_cc0_prop(
        name="Env_DTI_Lamp",
        src_subdir="street_lamp_01",
        pred=lambda n: n == "street_lamp_01",
        target_polys=1350,
        target_height=4.2,
        anchor_z="bottom",
        mat_override=make_pbr_mat(
            "Mat_DTI_StreetLamp",
            base_color=(0.22, 0.24, 0.30, 1.0),
            roughness=0.36,
            metallic=0.62,
            diff_path=SRC_DIR / "street_lamp_01/textures/street_lamp_01_diff_1k.jpg",
            max_tex=512,
        ),
    )

    built["Env_Campfire_Pit"] = build_cc0_prop(
        name="Env_Campfire_Pit",
        src_subdir="stone_fire_pit",
        pred=lambda n: n == "stone_fire_pit",
        target_polys=850,
        target_width=2.3,
        anchor_z="bottom",
        diff_tex=SRC_DIR / "stone_fire_pit/textures/stone_fire_pit_diff_1k.jpg",
    )

    built["Env_DTI_Arch"] = build_dti_couture_arch()
    built["Env_DTI_Backdrop"] = build_dti_backdrop()
    built["Env_DTI_StarCrest"] = build_dti_star_crest()
    built["Env_DTI_Stanchion"] = build_dti_stanchion()

    # 3. Stargate Portal Assembly (Outer Ring + Spinning Inner Glyph Track)
    built["Env_Stargate_Frame"] = build_stargate_frame()
    built["Env_Stargate_InnerRing"] = build_stargate_inner_ring()

    # 4. Cozy Glamping & Campfire Clearing Pack (CC0 Props + Custom Tent & Log Bench)
    built["Env_Camp_Tent"] = build_camp_tent()
    built["Env_Camp_LogBench"] = build_camp_log_bench()

    built["Env_Camp_Lantern"] = build_cc0_prop(
        name="Env_Camp_Lantern",
        src_subdir="Lantern_01",
        pred=lambda n: not n.endswith("_LOD1"),
        target_polys=950,
        target_height=0.52,
        anchor_z="bottom",
        mat_override=make_pbr_mat(
            "Mat_Camp_CC0Lantern",
            base_color=(0.92, 0.72, 0.28, 1.0),
            roughness=0.28,
            metallic=0.68,
            emissive=(1.0, 0.78, 0.28),
            emissive_strength=0.65,
            diff_path=SRC_DIR / "Lantern_01/textures/Lantern_01_brass_diff_1k.png",
            max_tex=512,
        ),
    )

    built["Env_Camp_Stool"] = build_cc0_prop(
        name="Env_Camp_Stool",
        src_subdir="folding_wooden_stool",
        pred=lambda n: not n.endswith("_LOD1"),
        target_polys=700,
        target_height=0.58,
        anchor_z="bottom",
        diff_tex=SRC_DIR / "folding_wooden_stool/textures/folding_wooden_stool_diff_1k.jpg",
    )

    built["Env_Camp_Stump"] = build_cc0_prop(
        name="Env_Camp_Stump",
        src_subdir="tree_stump_01",
        pred=lambda n: not n.endswith("_LOD1"),
        target_polys=850,
        target_height=0.72,
        anchor_z="bottom",
        diff_tex=SRC_DIR / "tree_stump_01/textures/tree_stump_01_diff_1k.jpg",
    )

    built["Env_Camp_Pot"] = build_cc0_prop(
        name="Env_Camp_Pot",
        src_subdir="pot_enamel_01",
        pred=lambda n: not n.endswith("_LOD1"),
        target_polys=650,
        target_width=0.42,
        anchor_z="bottom",
        diff_tex=SRC_DIR / "pot_enamel_01/textures/pot_enamel_01_diff_1k.jpg",
    )

    built["Env_Camp_Boulder"] = build_cc0_prop(
        name="Env_Camp_Boulder",
        src_subdir="rock_07",
        pred=lambda n: not n.endswith("_LOD1"),
        target_polys=600,
        target_width=1.65,
        anchor_z="bottom",
        diff_tex=SRC_DIR / "rock_07/textures/rock_07_diff_1k.jpg",
    )

    # 5. Sculpted & Articulated 3D Enemy Pack (Cavern + Mortal Shadow Rift Arena)
    built["Env_Mob_ShadowSlime"] = build_mob_shadow_slime()
    bat_body, bat_wl, bat_wr = build_mob_crystal_bat_parts()
    built["Env_Mob_CrystalBat_Body"] = bat_body
    built["Env_Mob_CrystalBat_WingL"] = bat_wl
    built["Env_Mob_CrystalBat_WingR"] = bat_wr

    golem_body, golem_al, golem_ar = build_mob_lava_golem_parts()
    built["Env_Mob_LavaGolem_Body"] = golem_body
    built["Env_Mob_LavaGolem_ArmL"] = golem_al
    built["Env_Mob_LavaGolem_ArmR"] = golem_ar

    built["Env_Mob_VoidWraith"] = build_mob_void_wraith()
    built["Env_Mob_RiftSpider"] = build_mob_rift_spider()

    # 6. Expansion Pack: Adoptable 3D Pets, Hoverboards/Mounts, Workshop, Mailbox & Obsidian Dragon Boss
    built["Env_Pet_CloudPuppy"] = build_pet_cloud_puppy()
    built["Env_Pet_StarlightKitsune"] = build_pet_starlight_kitsune()
    built["Env_Pet_ShadowKitty"] = build_pet_shadow_kitty()
    pdrag_body, pdrag_wl, pdrag_wr = build_pet_baby_dragon_parts()
    built["Env_Pet_BabyDragon_Body"] = pdrag_body
    built["Env_Pet_BabyDragon_WingL"] = pdrag_wl
    built["Env_Pet_BabyDragon_WingR"] = pdrag_wr

    built["Env_Mount_Hoverboard"] = build_mount_hoverboard()
    built["Env_Mount_StarlightSkates"] = build_mount_starlight_skates()
    built["Env_Mount_CometSurfer"] = build_mount_comet_surfer()

    built["Env_Crafting_Bench"] = build_crafting_bench()
    built["Env_Plot_Mailbox"] = build_plot_mailbox()
    built["Env_Obby_Gate"] = build_obby_gate()
    built["Env_Leaderboard_Board"] = build_leaderboard_board()
    built["Env_Boss_Altar"] = build_boss_altar()
    boss_body, boss_wl, boss_wr = build_boss_obsidian_dragon_parts()
    built["Env_Boss_ObsidianDragon_Body"] = boss_body
    built["Env_Boss_ObsidianDragon_WingL"] = boss_wl
    built["Env_Boss_ObsidianDragon_WingR"] = boss_wr

    # Reset all asset transforms to origin before saving .blend and exporting .glb
    for o in built.values():
        o.location = Vector((0, 0, 0))
        o.rotation_euler = Euler((0, 0, 0), "XYZ")
        o.scale = Vector((1, 1, 1))

    # Render EEVEE visual verification previews BEFORE exporting
    render_studio_preview(
        [built["Env_Tree_Blossom"], built["Env_Tree_Emerald"], built["Env_Tree_Lilac"]],
        PREVIEW_TREES,
        cam_loc=(0, -17.5, 5.2),
        cam_target=(0, 0, 3.2),
        spacing=6.5,
    )

    # Compose DTI Stage miniature diorama preview
    built["Env_DTI_Backdrop"].location = Vector((0, 5.5, 0))
    built["Env_DTI_StarCrest"].location = Vector((0, 4.9, 4.1))
    built["Env_DTI_Arch"].location = Vector((0, 0.5, 0))
    built["Env_DTI_Chandelier"].location = Vector((0, 0.5, 6.45))
    built["Env_DTI_Spotlight"].location = Vector((-4.2, -1.8, 0))
    built["Env_DTI_Lamp"].location = Vector((4.2, -1.8, 0))
    built["Env_DTI_Bench"].location = Vector((-4.4, 2.2, 0))
    built["Env_DTI_Stanchion"].location = Vector((2.6, -2.5, 0))
    render_studio_preview(
        [
            built["Env_DTI_Backdrop"],
            built["Env_DTI_StarCrest"],
            built["Env_DTI_Arch"],
            built["Env_DTI_Chandelier"],
            built["Env_DTI_Spotlight"],
            built["Env_DTI_Lamp"],
            built["Env_DTI_Bench"],
            built["Env_DTI_Stanchion"],
        ],
        PREVIEW_DTI,
        cam_loc=(6.8, -11.8, 4.8),
        cam_target=(0, 2.2, 2.8),
        spacing=0.0,
    )

    # Compose Stargate + Camping Clearing preview
    built["Env_Stargate_Frame"].location = Vector((-4.2, 0.5, 0))
    built["Env_Stargate_InnerRing"].location = Vector((-4.2, 0.5, 2.95))
    built["Env_Camp_Tent"].location = Vector((4.2, 1.5, 0))
    built["Env_Campfire_Pit"].location = Vector((2.6, -2.5, 0))
    built["Env_Camp_LogBench"].location = Vector((2.6, -5.0, 0))
    built["Env_Camp_Stump"].location = Vector((5.2, -2.5, 0))
    built["Env_Camp_Lantern"].location = Vector((5.2, -2.5, 0.72))
    built["Env_Camp_Stool"].location = Vector((0.6, -2.6, 0))
    built["Env_Camp_Pot"].location = Vector((3.8, -2.2, 0))
    built["Env_Camp_Boulder"].location = Vector((6.6, -1.2, 0))
    render_studio_preview(
        [
            built["Env_Stargate_Frame"],
            built["Env_Stargate_InnerRing"],
            built["Env_Camp_Tent"],
            built["Env_Campfire_Pit"],
            built["Env_Camp_LogBench"],
            built["Env_Camp_Stump"],
            built["Env_Camp_Lantern"],
            built["Env_Camp_Stool"],
            built["Env_Camp_Pot"],
            built["Env_Camp_Boulder"],
        ],
        PREVIEW_STARGATE_CAMP,
        cam_loc=(0.5, -13.5, 4.8),
        cam_target=(0.5, -0.5, 2.0),
        spacing=0.0,
    )

    # Compose 3D Enemy Roster Lineup Preview
    built["Env_Mob_ShadowSlime"].location = Vector((-4.8, 0, 0))
    built["Env_Mob_CrystalBat_Body"].location = Vector((-2.5, 0, 0.5))
    built["Env_Mob_CrystalBat_WingL"].location = Vector((-2.78, 0, 1.12))
    built["Env_Mob_CrystalBat_WingR"].location = Vector((-2.22, 0, 1.12))
    built["Env_Mob_RiftSpider"].location = Vector((-0.2, 0, 0))
    built["Env_Mob_VoidWraith"].location = Vector((2.2, 0, 0.15))
    built["Env_Mob_LavaGolem_Body"].location = Vector((4.8, 0, 0))
    built["Env_Mob_LavaGolem_ArmL"].location = Vector((3.95, 0, 1.75))
    built["Env_Mob_LavaGolem_ArmR"].location = Vector((5.65, 0, 1.75))
    render_studio_preview(
        [
            built["Env_Mob_ShadowSlime"],
            built["Env_Mob_CrystalBat_Body"],
            built["Env_Mob_CrystalBat_WingL"],
            built["Env_Mob_CrystalBat_WingR"],
            built["Env_Mob_RiftSpider"],
            built["Env_Mob_VoidWraith"],
            built["Env_Mob_LavaGolem_Body"],
            built["Env_Mob_LavaGolem_ArmL"],
            built["Env_Mob_LavaGolem_ArmR"],
        ],
        PREVIEW_ENEMIES,
        cam_loc=(0.0, -9.8, 2.8),
        cam_target=(0.0, 0.0, 1.1),
        spacing=0.0,
    )

    # Compose Expansion Pack Showcase Preview (4 Pets, 3 Mounts, Workshop, Mailbox, Obby Gate, Leaderboard, Altar & Obsidian Dragon Boss)
    for o in built.values():
        o.location = Vector((0, 0, 0))
    built["Env_Pet_CloudPuppy"].location = Vector((-5.2, -3.2, 0))
    built["Env_Pet_StarlightKitsune"].location = Vector((-3.6, -3.2, 0))
    built["Env_Pet_ShadowKitty"].location = Vector((-2.0, -3.2, 0))
    built["Env_Pet_BabyDragon_Body"].location = Vector((-0.4, -3.2, 0))
    built["Env_Pet_BabyDragon_WingL"].location = Vector((-0.55, -3.2, 0.48))
    built["Env_Pet_BabyDragon_WingR"].location = Vector((-0.25, -3.2, 0.48))
    built["Env_Mount_Hoverboard"].location = Vector((1.4, -3.2, 0))
    built["Env_Mount_StarlightSkates"].location = Vector((2.8, -3.2, 0))
    built["Env_Mount_CometSurfer"].location = Vector((4.4, -3.2, 0))
    built["Env_Plot_Mailbox"].location = Vector((-5.6, 0.2, 0))
    built["Env_Crafting_Bench"].location = Vector((-3.1, 0.2, 0))
    built["Env_Boss_Altar"].location = Vector((0.0, 0.2, 0))
    built["Env_Boss_ObsidianDragon_Body"].location = Vector((3.8, 0.6, 0))
    built["Env_Boss_ObsidianDragon_WingL"].location = Vector((3.0, 0.6, 1.9))
    built["Env_Boss_ObsidianDragon_WingR"].location = Vector((4.6, 0.6, 1.9))
    built["Env_Obby_Gate"].location = Vector((-3.2, 4.2, 0))
    built["Env_Leaderboard_Board"].location = Vector((2.8, 4.2, 0))
    render_studio_preview(
        [
            built["Env_Pet_CloudPuppy"],
            built["Env_Pet_StarlightKitsune"],
            built["Env_Pet_ShadowKitty"],
            built["Env_Pet_BabyDragon_Body"],
            built["Env_Pet_BabyDragon_WingL"],
            built["Env_Pet_BabyDragon_WingR"],
            built["Env_Mount_Hoverboard"],
            built["Env_Mount_StarlightSkates"],
            built["Env_Mount_CometSurfer"],
            built["Env_Plot_Mailbox"],
            built["Env_Crafting_Bench"],
            built["Env_Obby_Gate"],
            built["Env_Leaderboard_Board"],
            built["Env_Boss_Altar"],
            built["Env_Boss_ObsidianDragon_Body"],
            built["Env_Boss_ObsidianDragon_WingL"],
            built["Env_Boss_ObsidianDragon_WingR"],
        ],
        PREVIEW_EXPANSION,
        cam_loc=(0.0, -13.0, 4.9),
        cam_target=(0.0, 0.5, 1.4),
        spacing=0.0,
    )

    # Restore all origins to (0,0,0) for clean GLB template export
    for o in built.values():
        o.data.validate(verbose=False)
        o.location = Vector((0, 0, 0))
        o.hide_render = False

    # Purge unused CC0 orphan materials & unreferenced .exr images before packing
    for _ in range(3):
        bpy.ops.outliner.orphans_purge(do_local_ids=True, do_linked_ids=True, do_recursive=True)
    for img in list(bpy.data.images):
        if img.users == 0 or (not img.packed_file and img.filepath and not Path(bpy.path.abspath(img.filepath)).exists()):
            bpy.data.images.remove(img, do_unlink=True)

    OUT_BLEND.parent.mkdir(parents=True, exist_ok=True)
    OUT_GLB.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.file.pack_all()
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT_BLEND))

    bpy.ops.object.select_all(action="DESELECT")
    for o in built.values():
        o.select_set(True)

    bpy.ops.export_scene.gltf(
        filepath=str(OUT_GLB),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_materials="EXPORT",
        export_texcoords=True,
        export_normals=True,
        export_image_format="WEBP",
        export_image_quality=78,
        export_yup=True,
    )

    telemetry = {
        "blend_path": str(OUT_BLEND.relative_to(REPO_ROOT)),
        "blend_kb": round(OUT_BLEND.stat().st_size / 1024, 1),
        "glb_path": str(OUT_GLB.relative_to(REPO_ROOT)),
        "glb_kb": round(OUT_GLB.stat().st_size / 1024, 1),
        "assets": {},
    }
    total_polys = 0
    for k, o in built.items():
        pc = len(o.data.polygons)
        vc = len(o.data.vertices)
        total_polys += pc
        telemetry["assets"][k] = {
            "polys": pc,
            "verts": vc,
            "dims": [round(float(d), 3) for d in o.dimensions],
            "materials": [m.name for m in o.data.materials if m],
        }
        print(f"Exported {k}: {pc} polys, {vc} verts, dims={telemetry['assets'][k]['dims']}")
    telemetry["total_polys"] = total_polys
    OUT_TELEMETRY.write_text(json.dumps(telemetry, indent=2))
    print(f"\nSaved {OUT_BLEND} ({telemetry['blend_kb']} KB) & {OUT_GLB} ({telemetry['glb_kb']} KB), total={total_polys} polys")


if __name__ == "__main__":
    main()
