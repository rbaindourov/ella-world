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
