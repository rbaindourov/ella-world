"""
ELLA World — Headless Blender 5.2 Male Character & Oblivion "Tech-49" Wardrobe Builder
======================================================================================
Extends `assets/blender/ella_dti_wardrobe.blend` and exports `public/models/ella_dti_wardrobe.glb`
with an authentic male character rig (`Body_Male`, `Face_Brows_Male`) inspired by Jack Harper
(Tom Cruise, Tech-49 in Oblivion) plus modular male & unisex sci-fi/casual wardrobe slots:
  - Body_Male (athletic V-taper torso, flat pectoral chest, smooth masculine jaw, thicker neck, neutral lips)
  - Face_Brows_Male (straighter, thicker, lower-set masculine hero eyebrows)
  - Hair_tech_crop (Tom Cruise Oblivion short textured side-part crew haircut, smooth SUBSURF dome + locks)
  - Hair_maverick_quiff (wind-swept tousled hero quiff, smooth SUBSURF dome + swept locks)
  - Top_tech_jacket (Oblivion white/slate Tech-49 tactical flight jacket + Mandarin collar + 3D shoulder yokes,
    chest harness, wrist gauntlets, Tech-49 badge & utility belt)
  - Top_hero_henley (masculine flat-chested athletic short-sleeve crew tee with bisect-trimmed cuffs & collar)
  - Bottom_tech_pants (Oblivion tactical flight trousers + articulated charcoal knee armor pads)
  - Shoe_flight_boots (armored tactical flight boots + silver shin buckles)
  - Acc_aviators (brushed-silver double-bridge teardrop Aviator sunglasses with dark mirror lenses)
  - Hat_recon_cap (Jack Harper curved-brim Recon baseball cap)

Usage:
  blender -b /home/ella/projects/ella-world/assets/blender/ella_dti_wardrobe.blend \
          -P /home/ella/projects/ella-world/scripts/build_male_oblivion_wardrobe.py
"""

import math
import os
import bpy
import bmesh
from mathutils import Euler, Matrix, Vector
from mathutils.bvhtree import BVHTree

REPO_ROOT = "/home/ella/projects/ella-world"
BLEND_PATH = os.path.join(REPO_ROOT, "assets", "blender", "ella_dti_wardrobe.blend")
GLB_PATH = os.path.join(REPO_ROOT, "public", "models", "ella_dti_wardrobe.glb")
PREVIEW_DIR = os.environ.get(
    "ELLA_PREVIEW_DIR",
    "/home/ella/.gemini/antigravity/brain/345cfb38-5b9c-47a3-bf92-6f8a080de950/scratch/renders",
)

GENERATED_OBJECTS = [
    "Body_Male",
    "Face_Brows_Male",
    "Hair_tech_crop",
    "Hair_maverick_quiff",
    "Top_tech_jacket",
    "Top_hero_henley",
    "Bottom_tech_pants",
    "Shoe_flight_boots",
    "Acc_aviators",
    "Hat_recon_cap",
]


def smoothstep(edge0: float, edge1: float, x: float) -> float:
    if edge1 == edge0:
        return 0.0
    t = max(0.0, min(1.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3.0 - 2.0 * t)


def band_weight(val: float, low0: float, low1: float, high1: float, high0: float) -> float:
    return smoothstep(low0, low1, val) * (1.0 - smoothstep(high1, high0, val))


def ensure_material(name: str, base_rgb, roughness: float = 0.42, metallic: float = 0.25):
    mat = bpy.data.materials.get(name)
    if mat is None:
        mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    out_node = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = (base_rgb[0], base_rgb[1], base_rgb[2], 1.0)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    nt.links.new(bsdf.outputs["BSDF"], out_node.inputs["Surface"])
    return mat


def cleanup_previous_generated():
    for name in GENERATED_OBJECTS:
        existing = bpy.data.objects.get(name)
        if existing is not None:
            mesh_data = existing.data if existing.type == "MESH" else None
            bpy.data.objects.remove(existing, do_unlink=True)
            if mesh_data is not None and mesh_data.users == 0:
                bpy.data.meshes.remove(mesh_data)


def duplicate_skinned_object(src_name: str, new_name: str):
    src = bpy.data.objects[src_name]
    new_mesh = src.data.copy()
    new_mesh.name = f"{new_name}_Mesh"
    new_obj = src.copy()
    new_obj.name = new_name
    new_obj.data = new_mesh
    bpy.context.scene.collection.objects.link(new_obj)
    return new_obj


def parent_to_head_bone_world(obj, arm_obj):
    ref = bpy.data.objects["Acc_sunglasses"]
    obj.parent = arm_obj
    obj.parent_type = "BONE"
    obj.parent_bone = "head"
    obj.location = (0.0, 0.0, 0.0)
    obj.rotation_euler = (0.0, 0.0, 0.0)
    obj.scale = (1.0, 1.0, 1.0)
    obj.matrix_parent_inverse = ref.matrix_parent_inverse.copy()


def transfer_weights_from(target_obj, source_obj, arm_obj):
    # Remove existing Armature modifiers temporarily so DataTransfer sits at index 0 cleanly
    for m in list(target_obj.modifiers):
        if m.type == "ARMATURE":
            target_obj.modifiers.remove(m)

    target_obj.parent = arm_obj
    target_obj.parent_type = "OBJECT"
    target_obj.matrix_world = Matrix.Identity(4)

    bpy.ops.object.select_all(action="DESELECT")
    target_obj.select_set(True)
    bpy.context.view_layer.objects.active = target_obj

    dt = target_obj.modifiers.new(name="TempWeightTransfer", type="DATA_TRANSFER")
    dt.object = source_obj
    dt.use_vert_data = True
    dt.data_types_verts = {"VGROUP_WEIGHTS"}
    dt.vert_mapping = "POLYINTERP_NEAREST"

    bpy.ops.object.datalayout_transfer(modifier=dt.name)
    bpy.ops.object.modifier_apply(modifier=dt.name)

    arm_mod = target_obj.modifiers.new(name="Armature", type="ARMATURE")
    arm_mod.object = arm_obj


def set_bmesh_smooth(bm):
    for f in bm.faces:
        f.smooth = True


def append_bmesh_part(bm_dest: bmesh.types.BMesh, bm_part: bmesh.types.BMesh, mat_idx: int):
    for f in bm_part.faces:
        f.material_index = mat_idx
        f.smooth = True
    tmp = bpy.data.meshes.new("_tmp_part")
    bm_part.to_mesh(tmp)
    bm_dest.from_mesh(tmp)
    bpy.data.meshes.remove(tmp)


def smooth_chest_region(mesh, z_min=1.13, z_max=1.37, max_ax=0.165, iterations=6, factor=0.55):
    """Apply localized Laplacian smoothing to front chest/waist so flattened bust folds vanish 100%."""
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bm.verts.ensure_lookup_table()
    chest_verts = [
        v for v in bm.verts
        if z_min <= v.co.z <= z_max and abs(v.co.x) <= max_ax and v.co.y < -0.045
    ]
    for _ in range(iterations):
        bmesh.ops.smooth_vert(bm, verts=chest_verts, factor=factor, use_axis_x=True, use_axis_y=True, use_axis_z=True)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    set_bmesh_smooth(bm)
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=False)
    mesh.update()


def build_tube_along_curve(points, radii_xy, mat_slot: int, segs: int = 8, cap_ends: bool = True) -> bmesh.types.BMesh:
    """Create a smooth beveled 3D quad tube/strip along 3D control points."""
    bm = bmesh.new()
    rings = []
    n = len(points)
    for i, pt in enumerate(points):
        p = Vector(pt)
        if i == 0:
            tang = (Vector(points[1]) - p).normalized()
        elif i == n - 1:
            tang = (p - Vector(points[-2])).normalized()
        else:
            tang = (Vector(points[i + 1]) - Vector(points[i - 1])).normalized()

        ref_up = Vector((0.0, 1.0, 0.0)) if abs(tang.y) < 0.85 else Vector((1.0, 0.0, 0.0))
        u_axis = tang.cross(ref_up).normalized()
        v_axis = u_axis.cross(tang).normalized()
        rx, ry = radii_xy[i]
        ring = []
        for k in range(segs):
            ang = (k / segs) * math.tau
            off = u_axis * (math.cos(ang) * rx) + v_axis * (math.sin(ang) * ry)
            ring.append(bm.verts.new(p + off))
        rings.append(ring)

    for i in range(n - 1):
        r0, r1 = rings[i], rings[i + 1]
        for k in range(segs):
            kn = (k + 1) % segs
            f = bm.faces.new([r0[k], r0[kn], r1[kn], r1[k]])
            f.material_index = mat_slot
            f.smooth = True

    if cap_ends and len(rings) >= 2:
        f0 = bm.faces.new(list(reversed(rings[0])))
        f0.material_index = mat_slot
        f0.smooth = True
        f1 = bm.faces.new(rings[-1])
        f1.material_index = mat_slot
        f1.smooth = True

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return bm


def build_body_male():
    """Sculpt Body_Male from Body_Stylized while preserving 25-bone weights and socket alignment."""
    body_male = duplicate_skinned_object("Body_Stylized", "Body_Male")
    mesh = body_male.data

    for v in mesh.vertices:
        x, y, z = v.co.x, v.co.y, v.co.z
        ax = abs(x)

        # 1. Flatten female bust mounds AND fill under-bust crease into smooth athletic male chest
        w_chest = band_weight(z, 1.15, 1.22, 1.315, 1.355) * (1.0 - smoothstep(0.115, 0.155, ax))
        if w_chest > 0.0 and y < -0.075:
            target_pec_y = -0.123 + 0.014 * ((ax / 0.13) ** 2)
            y = y + (target_pec_y - y) * (0.90 * w_chest)

        # 2. Fill out pinched female hourglass waist into straight athletic male waist
        if ax < 0.175:
            w_waist = band_weight(z, 0.99, 1.055, 1.15, 1.215)
            x *= 1.0 + 0.30 * w_waist
            if y < 0.0:
                y *= 1.0 + 0.04 * w_waist

        # 3. Slightly narrow feminine outer hip flare
        if ax < 0.19:
            w_hip = band_weight(z, 0.82, 0.88, 0.96, 1.01)
            x *= 1.0 - 0.055 * w_hip

        # 4. Broaden male lats, shoulders, deltoids & arms subtly
        w_upper = band_weight(z, 1.19, 1.26, 1.37, 1.425)
        if w_upper > 0.0:
            x *= 1.0 + 0.07 * w_upper
        if ax > 0.185 and 0.96 <= z <= 1.37:
            w_arm = smoothstep(0.185, 0.24, ax) * (1.0 - smoothstep(0.33, 0.365, ax))
            y *= 1.0 + 0.08 * w_arm

        # 5. Stronger athletic male neck
        if ax < 0.095:
            w_neck = band_weight(z, 1.395, 1.415, 1.462, 1.488)
            x *= 1.0 + 0.14 * w_neck
            y *= 1.0 + 0.07 * w_neck

        # 6. Smooth, natural masculine jaw & chin (gentle broad curve across z=1.475..1.585 — zero jowl pinch!)
        w_jaw = band_weight(z, 1.472, 1.492, 1.545, 1.595)
        if w_jaw > 0.0 and ax < 0.105:
            x *= 1.0 + 0.055 * w_jaw
            if ax < 0.042 and y < -0.125 and z < 1.520:
                y -= 0.0038 * w_jaw * (1.0 - smoothstep(0.018, 0.042, ax))

        # Crisp nasal bridge definition
        if ax < 0.022 and y < -0.140 and 1.568 <= z <= 1.625:
            w_nose = band_weight(z, 1.568, 1.582, 1.612, 1.625) * (1.0 - smoothstep(0.008, 0.022, ax))
            y -= 0.0035 * w_nose

        # Confident masculine mouth contour (less pouty)
        if ax < 0.045 and y < -0.148 and 1.512 <= z <= 1.548:
            w_lip = band_weight(z, 1.512, 1.522, 1.538, 1.548) * (1.0 - smoothstep(0.024, 0.045, ax))
            y += 0.0032 * w_lip
            x *= 1.0 + 0.06 * w_lip

        v.co.x, v.co.y, v.co.z = x, y, z

    # Neutralize rosy-pink lip/blush vertex colors on Body_Male head
    ca = mesh.color_attributes.get("Color")
    if ca and ca.domain == "CORNER":
        for poly in mesh.polygons:
            for li in poly.loop_indices:
                vi = mesh.loops[li].vertex_index
                if mesh.vertices[vi].co.z > 1.48:
                    c = ca.data[li].color
                    if (c[0] - c[1]) > 0.08:
                        avg = min(1.0, (c[0] + c[1] + c[2]) / 3.0 + 0.12)
                        ca.data[li].color = (avg, avg, avg, c[3])

    mesh.update()
    smooth_chest_region(mesh, z_min=1.13, z_max=1.35, max_ax=0.145, iterations=5, factor=0.5)
    return body_male


def build_face_brows_male(arm_obj):
    """Build thicker, straighter, lower-set masculine hero eyebrows parented to head bone."""
    bm = bmesh.new()
    for side in (-1.0, 1.0):
        stations = [
            (0.017, -0.1665, 1.6485, 0.0068, 0.0055),
            (0.036, -0.1680, 1.6505, 0.0074, 0.0058),
            (0.055, -0.1638, 1.6530, 0.0072, 0.0056),
            (0.071, -0.1545, 1.6542, 0.0060, 0.0050),
            (0.084, -0.1415, 1.6495, 0.0038, 0.0040),
        ]
        rings = []
        for x_abs, cy, cz, half_h, half_d in stations:
            cx = side * x_abs
            pts = [
                Vector((cx, cy - half_d, cz)),
                Vector((cx, cy, cz + half_h)),
                Vector((cx, cy + half_d, cz)),
                Vector((cx, cy, cz - half_h)),
            ]
            if side < 0:
                pts.reverse()
            rings.append([bm.verts.new(p) for p in pts])

        for i in range(len(rings) - 1):
            r0, r1 = rings[i], rings[i + 1]
            for j in range(4):
                jn = (j + 1) % 4
                bm.faces.new([r0[j], r0[jn], r1[jn], r1[j]])
        bm.faces.new(list(reversed(rings[0])))
        bm.faces.new(rings[-1])

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    set_bmesh_smooth(bm)

    mesh = bpy.data.meshes.new("Face_Brows_Male_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=True)
    mesh.materials.append(bpy.data.materials["Mat_Avatar_Brow"])

    obj = bpy.data.objects.new("Face_Brows_Male", mesh)
    bpy.context.scene.collection.objects.link(obj)
    parent_to_head_bone_world(obj, arm_obj)
    return obj


def _build_clean_sculpted_male_hair(slot_name: str, style: str, arm_obj, body_male_obj):
    """
    Build 100% manifold, full-coverage quad-based Pixar/DTI male hairstyles:
      - Raycasts against Body_Male's exact cranium BVHTree from head center so every single temple,
        parietal ridge, forehead, sideburn, and nape vertex sits cleanly OUTSIDE the scalp (zero bare
        temple skin poke-through!)
      - Regular watertight quad grid + feathered flat-ribbon hero hair locks + Catmull-Clark SUBSURF!
    """
    scalp_bm = bmesh.new()
    scalp_bm.from_mesh(body_male_obj.data)
    bvh = BVHTree.FromBMesh(scalp_bm)
    scalp_bm.free()

    bm = bmesh.new()

    u_segs = 36   # azimuth around head (ang = 3*pi/2 is front -Y)
    v_rings = 12  # elevation from lower hairline up to crown apex

    head_center = Vector((0.0, -0.026, 1.635))
    crown_z = 1.805 if style == "crop" else 1.818

    # Step 1: Compute unit direction vectors & exact BVH cranium hit distances across the (v_rings x u_segs) grid
    dirs_grid = []
    raw_r_grid = []
    for vi in range(v_rings):
        vf = vi / (v_rings - 1)
        row_dirs = []
        row_r = []
        for ui in range(u_segs):
            ang = (ui / u_segs) * math.tau
            ca, sa = math.cos(ang), math.sin(ang)

            # Sculpted masculine lower hairline contour:
            # Front (-Y, sa ≈ -1): natural forehead hairline around Z = 1.682
            # Temple corner (|ca| ≈ 0.72, sa ≈ -0.68): classic subtle masculine temple contour (+0.008)
            # Sideburn (|ca| > 0.75, sa ≈ -0.44): neat tapered sideburns down to Z = 1.578 in front of ear
            # Ear arch (|ca| > 0.80, sa ≈ -0.02): clean arch above ear helix at Z = 1.656
            # Back nape (sa > 0.35): tapers down occiput to Z = 1.515
            front_w = max(0.0, -sa) ** 1.5
            back_w = max(0.0, sa) ** 1.2
            temple_w = math.exp(-(((sa - (-0.72)) / 0.18) ** 2)) * (abs(ca) ** 1.6)
            sideburn_w = math.exp(-(((sa - (-0.43)) / 0.18) ** 2)) * (abs(ca) ** 2.0)
            ear_arch_w = math.exp(-(((sa - (-0.02)) / 0.22) ** 2)) * (abs(ca) ** 2.0)

            base_z = (
                1.640
                + 0.044 * front_w
                + 0.010 * temple_w
                - 0.125 * back_w
                - 0.064 * sideburn_w
                + 0.022 * ear_arch_w
            )
            if sa < -0.5:
                base_z += 0.004 * math.sin(ca * 2.4 - 0.4)

            phi = vf * (math.pi * 0.465)
            z_target = base_z + math.sin(phi) * (crown_z - base_z)
            horiz_r = max(0.018, math.cos(phi) * 0.135)
            target_pt = Vector((ca * horiz_r, head_center.y + sa * horiz_r * 1.12, z_target))
            ray_dir = (target_pt - head_center).normalized()

            hit_loc, _, _, hit_dist = bvh.ray_cast(head_center, ray_dir, 0.35)
            r_scalp = hit_dist if (hit_loc is not None and hit_dist > 0.05) else 0.132
            row_dirs.append(ray_dir)
            row_r.append(r_scalp)

        dirs_grid.append(row_dirs)
        raw_r_grid.append(row_r)

    # Smooth scalp radii gently across neighbors while clamping >= raw_r + 0.002 so SUBSURF shrinkage never dips inside temples!
    smooth_r = [list(row) for row in raw_r_grid]
    for _ in range(2):
        nxt = [list(row) for row in smooth_r]
        for vi in range(v_rings):
            v_prev = max(0, vi - 1)
            v_next = min(v_rings - 1, vi + 1)
            for ui in range(u_segs):
                u_prev = (ui - 1) % u_segs
                u_next = (ui + 1) % u_segs
                avg = (
                    0.44 * smooth_r[vi][ui]
                    + 0.14 * (smooth_r[vi][u_prev] + smooth_r[vi][u_next] + smooth_r[v_prev][ui] + smooth_r[v_next][ui])
                )
                nxt[vi][ui] = max(avg, raw_r_grid[vi][ui] + 0.003)
        smooth_r = nxt

    outer_grid = []
    inner_grid = []
    for vi in range(v_rings):
        vf = vi / (v_rings - 1)
        row_out = []
        row_in = []
        for ui in range(u_segs):
            ang = (ui / u_segs) * math.tau
            sa = math.sin(ang)
            ray_dir = dirs_grid[vi][ui]
            r_base = smooth_r[vi][ui]

            vol = 0.0145 if style == "crop" else 0.0215
            # Keep crisp fullness on sides/temples and taper gently right at hairline edge (vf=0)
            rim_taper = 0.42 + 0.58 * smoothstep(0.0, 0.28, vf)
            tex = (0.0018 if style == "crop" else 0.0038) * math.sin(ang * 10.0 + vf * 3.5) * smoothstep(0.15, 0.8, vf)

            r_out = r_base + vol * rim_taper + tex
            r_in = r_base - 0.0035

            p_out = head_center + ray_dir * r_out
            p_in = head_center + ray_dir * r_in

            if style == "quiff" and sa < -0.25 and vf > 0.20:
                qw = ((-sa - 0.25) / 0.75) * smoothstep(0.20, 0.85, vf)
                p_out.z += 0.015 * qw
                p_out.y -= 0.012 * qw

            row_out.append(bm.verts.new(p_out))
            row_in.append(bm.verts.new(p_in))

        outer_grid.append(row_out)
        inner_grid.append(row_in)

    # Crown apex hit check
    up_dir = Vector((0.0, -0.04 if style == "quiff" else 0.0, 1.0)).normalized()
    hit_top, _, _, top_dist = bvh.ray_cast(head_center, up_dir, 0.35)
    top_r = (top_dist if hit_top is not None else 0.150) + (0.016 if style == "crop" else 0.024)
    apex_out = bm.verts.new(head_center + up_dir * top_r)
    apex_in = bm.verts.new(head_center + up_dir * (top_r - 0.016))

    for vi in range(v_rings - 1):
        for ui in range(u_segs):
            un = (ui + 1) % u_segs
            bm.faces.new([outer_grid[vi][ui], outer_grid[vi][un], outer_grid[vi + 1][un], outer_grid[vi + 1][ui]])
            bm.faces.new([inner_grid[vi][un], inner_grid[vi][ui], inner_grid[vi + 1][ui], inner_grid[vi + 1][un]])

    for ui in range(u_segs):
        un = (ui + 1) % u_segs
        bm.faces.new([outer_grid[-1][ui], outer_grid[-1][un], apex_out])
        bm.faces.new([inner_grid[-1][un], inner_grid[-1][ui], apex_in])
        bm.faces.new([inner_grid[0][ui], inner_grid[0][un], outer_grid[0][un], outer_grid[0][ui]])

    # 2. Broad, flat-ribbon overlapping sculpted hair locks hugging the outer hair dome across the full crown & temples!
    def snap_lock_pt_to_dome(pt_xyz, lift_mm: float):
        v = Vector(pt_xyz)
        d = (v - head_center)
        dist = d.length
        dn = d.normalized()
        hl, _, _, hd = bvh.ray_cast(head_center, dn, 0.35)
        min_r = (hd if hl is not None else dist) + lift_mm
        return head_center + dn * max(dist, min_r)

    if style == "crop":
        # Full-width Tom Cruise Oblivion short textured side-part locks (sweeping from left part X=-0.075 across to +0.098)
        lock_specs = [
            ([(-0.070, -0.075, 1.790), (-0.046, -0.138, 1.760), (-0.018, -0.165, 1.706)], 0.013),
            ([(-0.044, -0.072, 1.796), (-0.012, -0.144, 1.764), (0.020, -0.167, 1.708)], 0.014),
            ([(-0.012, -0.072, 1.798), (0.024, -0.144, 1.764), (0.056, -0.162, 1.708)], 0.014),
            ([(0.022, -0.070, 1.794), (0.058, -0.138, 1.758), (0.088, -0.150, 1.704)], 0.013),
            ([(0.055, -0.062, 1.782), (0.088, -0.120, 1.744), (0.110, -0.130, 1.692)], 0.012),
            ([(-0.078, -0.065, 1.782), (-0.098, -0.122, 1.742), (-0.112, -0.132, 1.690)], 0.012),
        ]
        rad_root, rad_tip = (0.026, 0.0065), (0.010, 0.0035)
    else:
        # Tousled Maverick hero quiff with fuller swept waves (cleanly tucked inside Recon Cap crown!)
        lock_specs = [
            ([(-0.068, -0.070, 1.798), (-0.038, -0.144, 1.780), (-0.006, -0.168, 1.718)], 0.015),
            ([(-0.034, -0.068, 1.804), (0.002, -0.150, 1.784), (0.036, -0.168, 1.720)], 0.016),
            ([(0.002, -0.068, 1.804), (0.038, -0.146, 1.780), (0.070, -0.160, 1.716)], 0.015),
            ([(0.036, -0.064, 1.792), (0.070, -0.132, 1.764), (0.096, -0.142, 1.704)], 0.013),
            ([(-0.074, -0.062, 1.784), (-0.096, -0.122, 1.748), (-0.104, -0.132, 1.692)], 0.012),
        ]
        rad_root, rad_tip = (0.026, 0.0075), (0.010, 0.0038)

    for pts_3, lift_base in lock_specs:
        v0 = snap_lock_pt_to_dome(pts_3[0], lift_base)
        v1 = snap_lock_pt_to_dome(pts_3[1], lift_base + 0.002)
        v2 = snap_lock_pt_to_dome(pts_3[2], lift_base * 0.55)
        sampled_pts = []
        sampled_rad = []
        for si in range(7):
            t = si / 6.0
            p = ((1 - t) ** 2) * v0 + 2 * (1 - t) * t * v1 + (t ** 2) * v2
            p = snap_lock_pt_to_dome(p, lift_base * (1.0 - 0.35 * t))
            rx = rad_root[0] * (1 - t) + rad_tip[0] * t
            ry = rad_root[1] * (1 - t) + rad_tip[1] * t
            sampled_pts.append(p)
            sampled_rad.append((rx, ry))
        lock_bm = build_tube_along_curve(sampled_pts, sampled_rad, mat_slot=0, segs=8, cap_ends=True)
        append_bmesh_part(bm, lock_bm, 0)
        lock_bm.free()

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    set_bmesh_smooth(bm)

    mesh = bpy.data.meshes.new(f"{slot_name}_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(bpy.data.materials["Mat_Avatar_HairSolid"])

    obj = bpy.data.objects.new(slot_name, mesh)
    bpy.context.scene.collection.objects.link(obj)

    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    sub = obj.modifiers.new(name="SmoothHair", type="SUBSURF")
    sub.levels = 1
    sub.render_levels = 1
    bpy.ops.object.modifier_apply(modifier=sub.name)

    obj.data.validate(clean_customdata=True)
    obj.data.update()
    parent_to_head_bone_world(obj, arm_obj)
    return obj


def build_male_hairs(arm_obj, body_male_obj):
    return [
        _build_clean_sculpted_male_hair("Hair_tech_crop", "crop", arm_obj, body_male_obj),
        _build_clean_sculpted_male_hair("Hair_maverick_quiff", "quiff", arm_obj, body_male_obj),
    ]


def _sculpt_masculine_sweater_shell(mesh, lower_hem_z: float = 0.955):
    """Transform Top_sweater's torso from feminine hourglass/bust into a smooth, crease-free athletic male cut."""
    for v in mesh.vertices:
        x, y, z = v.co.x, v.co.y, v.co.z
        ax = abs(x)
        is_torso = ax < 0.195 or z >= 1.25

        if is_torso:
            # 1. Flatten bust peaks AND pull out under-bust crease into straight masculine chest
            w_chest = band_weight(z, 1.14, 1.21, 1.32, 1.37) * (1.0 - smoothstep(0.125, 0.175, ax))
            if w_chest > 0.0 and y < -0.075:
                pec_y = -0.138 + 0.015 * ((ax / 0.145) ** 2)
                y = y + (pec_y - y) * (0.94 * w_chest)

            # 2. Widen pinched waist AND tuck lower hip flare so torso forms a clean masculine V-taper
            if ax < 0.19:
                w_waist = band_weight(z, 0.99, 1.055, 1.16, 1.225)
                x *= 1.0 + 0.32 * w_waist
                if y < 0.0:
                    y *= 1.0 + 0.06 * w_waist
                else:
                    y *= 1.0 + 0.12 * w_waist

                w_hip_trim = 1.0 - smoothstep(0.97, 1.04, z)
                if z < 1.04:
                    x *= 1.0 - 0.08 * w_hip_trim

            # 3. Broaden male shoulders & upper lats
            w_sh = band_weight(z, 1.22, 1.28, 1.38, 1.43)
            x *= 1.0 + 0.08 * w_sh

            # 4. Close front V-neck dip completely up to z = 1.395
            if ax < 0.075 and y < -0.04 and z > 1.31:
                z = max(z, 1.395 - 0.12 * ax)

        if ax >= 0.195 and 0.94 <= z <= 1.37:
            y *= 1.08

        v.co.x, v.co.y, v.co.z = x, y, z

    # Level the arched side-hip waist boundary ring so zero bare skin triangles appear at the hips!
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bm.verts.ensure_lookup_table()
    waist_boundary = set()
    for e in bm.edges:
        if e.is_boundary:
            for v in e.verts:
                if v.co.z < 1.12 and abs(v.co.x) < 0.195:
                    waist_boundary.add(v)
    for v in waist_boundary:
        v.co.z = lower_hem_z
        # Give slight extra hip clearance so body skin never pokes through
        v.co.x *= 1.04
        v.co.y *= 1.05

    # Relax the ring right above the waist boundary so spacing is even
    ring1 = set()
    for v in waist_boundary:
        for e in v.link_edges:
            ov = e.other_vert(v)
            if ov not in waist_boundary and ov.co.z < 1.12:
                ring1.add(ov)
    for v in ring1:
        v.co.z = min(v.co.z, lower_hem_z + 0.045)
        v.co.x *= 1.02
        v.co.y *= 1.03

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    set_bmesh_smooth(bm)
    bm.to_mesh(mesh)
    bm.free()

    mesh.update()
    smooth_chest_region(mesh, z_min=1.06, z_max=1.37, max_ax=0.165, iterations=8, factor=0.58)


def build_top_tech_jacket(arm_obj, mat_tech_dark):
    """
    Build the iconic Oblivion (2013) Jack Harper 'Tech-49' Tactical Flight Jacket using 100% clean,
    razor-smooth 3D quad paneling:
      - Smooth light-grey/white athletic jacket shell (Mat_Avatar_Top)
      - High stand-up Mandarin collar (Mat_Avatar_Top outer + Mat_Avatar_TechDark inner trim)
      - 3D Charcoal Shoulder Yoke Caps, Center Zipper Placket, Diagonal Oblivion Chest Seams & Wrist Gauntlets
      - 3D Glowing 'TECH-49' Chest Telemetry Badge & Left-Sleeve Recon Patch
      - 3D Tactical Utility Belt, Brushed-Silver Buckle & Twin Hip Recon Pouches
    """
    jacket_obj = duplicate_skinned_object("Top_sweater", "Top_tech_jacket")
    mesh = jacket_obj.data
    _sculpt_masculine_sweater_shell(mesh, lower_hem_z=0.968)

    # Slightly taper outer wrist sleeve vertices (Z=1.00..1.10, |X|>0.285) so 3D charcoal wrist gauntlets wrap cleanly!
    for v in mesh.vertices:
        ax = abs(v.co.x)
        if ax > 0.285 and 0.995 <= v.co.z <= 1.105:
            t_f = (1.105 - v.co.z) / 0.110
            cx = math.copysign(0.308 + 0.040 * t_f, v.co.x)
            cy = 0.002 - 0.014 * t_f
            v.co.x = cx + (v.co.x - cx) * 0.84
            v.co.y = cy + (v.co.y - cy) * 0.84
    mesh.update()

    mat_top = bpy.data.materials["Mat_Avatar_Top"]
    mat_accent = bpy.data.materials["Mat_Avatar_Accent"]
    mat_silver = bpy.data.materials["Mat_Avatar_Silver"]

    mesh.materials.clear()
    mesh.materials.append(mat_top)        # slot 0: Crisp white/light-grey jacket shell
    mesh.materials.append(mat_tech_dark)  # slot 1: Smooth 3D charcoal tactical armor & seams
    mesh.materials.append(mat_accent)     # slot 2: Tech-49 cyan/amber telemetry badge & sleeve patch
    mesh.materials.append(mat_silver)     # slot 3: Brushed silver hardware & buckle

    bm = bmesh.new()
    bm.from_mesh(mesh)
    for f in bm.faces:
        f.material_index = 0
        f.smooth = True

    # 1. Smooth 3D High Stand-Up Oblivion Mandarin Collar (z = 1.352..1.470)
    collar_bm = bmesh.new()
    segs = 24
    r_inner_x, r_inner_y = 0.069, 0.063
    thick = 0.015
    z_bot, z_top = 1.352, 1.470
    neck_cy = -0.005

    outer_bot, outer_top, inner_bot, inner_top = [], [], [], []
    for i in range(segs + 1):
        a0 = -math.pi / 2.0 + 0.22
        a1 = 1.5 * math.pi - 0.22
        ang = a0 + (a1 - a0) * (i / segs)
        ca, sa = math.cos(ang), math.sin(ang)
        notch_taper = smoothstep(0.0, 0.14, min(i / segs, 1.0 - i / segs))
        zt = z_top - 0.012 * (1.0 - notch_taper)

        ix, iy = ca * r_inner_x, neck_cy + sa * r_inner_y
        ox, oy = ca * (r_inner_x + thick * 1.15), neck_cy + sa * (r_inner_y + thick)
        inner_bot.append(collar_bm.verts.new((ix * 1.12, iy * 1.18, z_bot)))
        inner_top.append(collar_bm.verts.new((ix * 0.96, iy, zt)))
        outer_bot.append(collar_bm.verts.new((ox * 1.22, oy * 1.25, z_bot)))
        outer_top.append(collar_bm.verts.new((ox, oy, zt)))

    for i in range(segs):
        f_out = collar_bm.faces.new([outer_bot[i], outer_bot[i + 1], outer_top[i + 1], outer_top[i]])
        f_out.material_index = 0
        f_in = collar_bm.faces.new([inner_bot[i + 1], inner_bot[i], inner_top[i], inner_top[i + 1]])
        f_in.material_index = 1
        f_top = collar_bm.faces.new([outer_top[i], outer_top[i + 1], inner_top[i + 1], inner_top[i]])
        f_top.material_index = 1
        f_bot = collar_bm.faces.new([inner_bot[i], inner_bot[i + 1], outer_bot[i + 1], outer_bot[i]])
        f_bot.material_index = 0

    for cap_quad in [
        [inner_bot[0], outer_bot[0], outer_top[0], inner_top[0]],
        [outer_bot[-1], inner_bot[-1], inner_top[-1], outer_top[-1]],
    ]:
        fc = collar_bm.faces.new(cap_quad)
        fc.material_index = 1
    bmesh.ops.recalc_face_normals(collar_bm, faces=collar_bm.faces)
    for f in collar_bm.faces:
        f.smooth = True
    tmp_c = bpy.data.meshes.new("_tmp_collar")
    collar_bm.to_mesh(tmp_c)
    collar_bm.free()
    bm.from_mesh(tmp_c)
    bpy.data.meshes.remove(tmp_c)

    # Tucked inner-throat dark undershirt behind Mandarin V-notch (never protrudes past white chest!)
    bib_bm = bmesh.new()
    bmesh.ops.create_cube(bib_bm, size=1.0)
    bmesh.ops.scale(bib_bm, vec=(0.036, 0.016, 0.044), verts=bib_bm.verts)
    bmesh.ops.translate(bib_bm, vec=(0.0, -0.076, 1.396), verts=bib_bm.verts)
    append_bmesh_part(bm, bib_bm, 1)
    bib_bm.free()

    # Smooth center-front charcoal zipper seam strip (Z = 1.015..1.365)
    zip_pts = [
        (0.0, -0.143, 1.015),
        (0.0, -0.144, 1.120),
        (0.0, -0.143, 1.225),
        (0.0, -0.138, 1.305),
        (0.0, -0.106, 1.368),
    ]
    zip_bm = build_tube_along_curve(zip_pts, [(0.0085, 0.005)] * len(zip_pts), mat_slot=1, segs=8)
    append_bmesh_part(bm, zip_bm, 1)
    zip_bm.free()

    # Smooth diagonal Oblivion chest harness seams + 3D Charcoal Shoulder Epaulet Yokes + Forearm Gauntlets
    for side in (-1.0, 1.0):
        diag_pts = [
            (side * 0.008, -0.142, 1.215),
            (side * 0.055, -0.141, 1.255),
            (side * 0.105, -0.132, 1.305),
            (side * 0.150, -0.102, 1.355),
        ]
        diag_bm = build_tube_along_curve(diag_pts, [(0.0065, 0.005)] * len(diag_pts), mat_slot=1, segs=8)
        append_bmesh_part(bm, diag_bm, 1)
        diag_bm.free()

        yoke_pts = [
            (side * 0.092, -0.012, 1.398),
            (side * 0.142, -0.008, 1.382),
            (side * 0.192, 0.002, 1.345),
            (side * 0.228, 0.010, 1.292),
        ]
        yoke_rad = [(0.052, 0.014), (0.058, 0.016), (0.054, 0.015), (0.042, 0.010)]
        yoke_bm = build_tube_along_curve(yoke_pts, yoke_rad, mat_slot=1, segs=10)
        append_bmesh_part(bm, yoke_bm, 1)
        yoke_bm.free()

        # Full-wrap smooth 3D Charcoal Forearm/Wrist Tactical Gauntlet Cuff
        cuff_pts = [
            (side * 0.308, 0.002, 1.095),
            (side * 0.330, -0.006, 1.050),
            (side * 0.348, -0.012, 1.010),
        ]
        cuff_rad = [(0.048, 0.048), (0.045, 0.045), (0.041, 0.041)]
        cuff_bm = build_tube_along_curve(cuff_pts, cuff_rad, mat_slot=1, segs=14)
        append_bmesh_part(bm, cuff_bm, 1)
        cuff_bm.free()

    # 3. Smooth 3D Tactical Utility Belt + Brushed-Silver Buckle + Twin Hip Recon Pouches + Tech-49 Badges
    belt_bm = bmesh.new()
    b_segs = 32
    belt_rx, belt_ry = 0.165, 0.130
    belt_cy = -0.025
    bz0, bz1 = 0.962, 1.022
    b_in0, b_in1, b_out0, b_out1 = [], [], [], []
    for i in range(b_segs):
        ang = (i / b_segs) * math.tau
        ca, sa = math.cos(ang), math.sin(ang)
        shape = 1.0 - 0.05 * (sa ** 2)
        ix, iy = ca * (belt_rx - 0.016) * shape, belt_cy + sa * (belt_ry - 0.016) * shape
        ox, oy = ca * (belt_rx + 0.010) * shape, belt_cy + sa * (belt_ry + 0.010) * shape
        b_in0.append(belt_bm.verts.new((ix, iy, bz0)))
        b_in1.append(belt_bm.verts.new((ix, iy, bz1)))
        b_out0.append(belt_bm.verts.new((ox, oy, bz0)))
        b_out1.append(belt_bm.verts.new((ox, oy, bz1)))

    for i in range(b_segs):
        ni = (i + 1) % b_segs
        for quad in [
            [b_out0[i], b_out0[ni], b_out1[ni], b_out1[i]],
            [b_in0[ni], b_in0[i], b_in1[i], b_in1[ni]],
            [b_out1[i], b_out1[ni], b_in1[ni], b_in1[i]],
            [b_in0[i], b_in0[ni], b_out0[ni], b_out0[i]],
        ]:
            f = belt_bm.faces.new(quad)
            f.material_index = 1
            f.smooth = True
    bmesh.ops.recalc_face_normals(belt_bm, faces=belt_bm.faces)
    append_bmesh_part(bm, belt_bm, 1)
    belt_bm.free()

    def add_beveled_box(mat_slot: int, center, size_xyz, rot_euler=(0.0, 0.0, 0.0)):
        box_bm = bmesh.new()
        bmesh.ops.create_cube(box_bm, size=1.0)
        bmesh.ops.scale(box_bm, vec=size_xyz, verts=box_bm.verts)
        bmesh.ops.bevel(
            box_bm,
            geom=list(box_bm.edges),
            offset=min(size_xyz) * 0.18,
            segments=2,
            profile=0.5,
            affect="EDGES",
        )
        rot_m = Euler(rot_euler, "XYZ").to_matrix()
        bmesh.ops.rotate(box_bm, cent=(0, 0, 0), matrix=rot_m, verts=box_bm.verts)
        bmesh.ops.translate(box_bm, vec=center, verts=box_bm.verts)
        bmesh.ops.recalc_face_normals(box_bm, faces=box_bm.faces)
        append_bmesh_part(bm, box_bm, mat_slot)
        box_bm.free()

    add_beveled_box(3, (0.0, -0.153, 0.992), (0.058, 0.018, 0.062))
    add_beveled_box(2, (0.0, -0.160, 0.992), (0.030, 0.010, 0.028))
    add_beveled_box(1, (-0.158, -0.052, 0.990), (0.032, 0.064, 0.074), (0.0, 0.0, -0.28))
    add_beveled_box(1, (0.158, -0.052, 0.990), (0.032, 0.064, 0.074), (0.0, 0.0, 0.28))
    add_beveled_box(3, (0.064, -0.142, 1.292), (0.052, 0.014, 0.028), (-0.06, 0.0, -0.12))
    add_beveled_box(2, (0.064, -0.147, 1.292), (0.040, 0.010, 0.016), (-0.06, 0.0, -0.12))
    add_beveled_box(2, (0.246, -0.028, 1.238), (0.016, 0.038, 0.044), (0.0, 0.52, 0.0))

    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=False)
    mesh.update()

    transfer_weights_from(jacket_obj, bpy.data.objects["Top_sweater"], arm_obj)
    return jacket_obj


def build_top_hero_henley(arm_obj, mat_tech_dark):
    """
    Build a razor-clean casual masculine athletic short-sleeve crew T-shirt ('Top_hero_henley'):
      - Straight full-coverage waist hem down to Z=0.922 + 3D ribbed T-shirt waist hem band (zero side-hip gaps!)
      - Exact planar `bmesh.ops.bisect_plane` perpendicular to each upper arm + 3D bicep cuff & crew collar bands!
    """
    henley_obj = duplicate_skinned_object("Top_sweater", "Top_hero_henley")
    mesh = henley_obj.data
    _sculpt_masculine_sweater_shell(mesh, lower_hem_z=0.932)

    mat_top = bpy.data.materials["Mat_Avatar_Top"]
    mesh.materials.clear()
    mesh.materials.append(mat_top)        # slot 0: Uniform crisp shirt body & waist hem
    mesh.materials.append(mat_tech_dark)  # slot 1: Smooth 3D crew collar, placket & sleeve cuff bands

    bm = bmesh.new()
    bm.from_mesh(mesh)

    for side in (-1.0, 1.0):
        geom_all = list(bm.verts) + list(bm.edges) + list(bm.faces)
        bmesh.ops.bisect_plane(
            bm,
            geom=geom_all,
            plane_co=(side * 0.242, 0.012, 1.205),
            plane_no=(side * 0.72, 0.0, -0.69),
            clear_outer=True,
            clear_inner=False,
        )

    for f in bm.faces:
        f.material_index = 0
        f.smooth = True

    # Smooth 3D ribbed crew-neck collar ring + subtle Oblivion 3-button Henley chest placket!
    collar_pts = []
    segs = 26
    for i in range(segs + 1):
        ang = (i / segs) * math.tau
        collar_pts.append((math.cos(ang) * 0.074, -0.005 + math.sin(ang) * 0.066, 1.398 - 0.010 * max(0.0, -math.sin(ang))))
    collar_bm = build_tube_along_curve(collar_pts, [(0.015, 0.016)] * len(collar_pts), mat_slot=1, segs=8, cap_ends=False)
    append_bmesh_part(bm, collar_bm, 1)
    collar_bm.free()

    placket_pts = [(0.0, -0.141, 1.280), (0.0, -0.136, 1.335), (0.0, -0.114, 1.384)]
    placket_bm = build_tube_along_curve(placket_pts, [(0.010, 0.005)] * len(placket_pts), mat_slot=1, segs=8)
    append_bmesh_part(bm, placket_bm, 1)
    placket_bm.free()

    # Smooth 3D full-coverage T-shirt lower torso hem cylinder (Z = 0.968..1.125, generously clears hips!)
    hem_bm = bmesh.new()
    h_segs = 32
    hem_cy = -0.022
    h_rings_spec = [
        (0.968, 0.172, 0.132),
        (1.018, 0.168, 0.128),
        (1.072, 0.162, 0.125),
        (1.125, 0.156, 0.124),
    ]
    h_out_rows, h_in_rows = [], []
    for hz, hrx, hry in h_rings_spec:
        ro, ri = [], []
        for i in range(h_segs):
            ang = (i / h_segs) * math.tau
            ca, sa = math.cos(ang), math.sin(ang)
            shape = 1.0 - 0.04 * (sa ** 2)
            ro.append(hem_bm.verts.new((ca * hrx * shape, hem_cy + sa * hry * shape, hz)))
            ri.append(hem_bm.verts.new((ca * (hrx - 0.022) * shape, hem_cy + sa * (hry - 0.022) * shape, hz)))
        h_out_rows.append(ro)
        h_in_rows.append(ri)

    for r_idx in range(len(h_rings_spec) - 1):
        for i in range(h_segs):
            ni = (i + 1) % h_segs
            fo = hem_bm.faces.new([h_out_rows[r_idx][i], h_out_rows[r_idx][ni], h_out_rows[r_idx + 1][ni], h_out_rows[r_idx + 1][i]])
            fo.material_index = 0
            fo.smooth = True
            fi = hem_bm.faces.new([h_in_rows[r_idx][ni], h_in_rows[r_idx][i], h_in_rows[r_idx + 1][i], h_in_rows[r_idx + 1][ni]])
            fi.material_index = 0
            fi.smooth = True
    for i in range(h_segs):
        ni = (i + 1) % h_segs
        for cap_q in [
            [h_in_rows[0][i], h_in_rows[0][ni], h_out_rows[0][ni], h_out_rows[0][i]],
            [h_out_rows[-1][i], h_out_rows[-1][ni], h_in_rows[-1][ni], h_in_rows[-1][i]],
        ]:
            fc = hem_bm.faces.new(cap_q)
            fc.material_index = 0
            fc.smooth = True
    bmesh.ops.recalc_face_normals(hem_bm, faces=hem_bm.faces)
    append_bmesh_part(bm, hem_bm, 0)
    hem_bm.free()

    # Casual slim dark waist belt at shirt hem (Z = 0.962..1.006) for crisp tailored silhouette over any pants!
    cbelt_bm = bmesh.new()
    cb_in0, cb_in1, cb_out0, cb_out1 = [], [], [], []
    for i in range(h_segs):
        ang = (i / h_segs) * math.tau
        ca, sa = math.cos(ang), math.sin(ang)
        shape = 1.0 - 0.04 * (sa ** 2)
        cb_in0.append(cbelt_bm.verts.new((ca * 0.156 * shape, hem_cy + sa * 0.120 * shape, 0.962)))
        cb_in1.append(cbelt_bm.verts.new((ca * 0.156 * shape, hem_cy + sa * 0.120 * shape, 1.006)))
        cb_out0.append(cbelt_bm.verts.new((ca * 0.175 * shape, hem_cy + sa * 0.136 * shape, 0.962)))
        cb_out1.append(cbelt_bm.verts.new((ca * 0.175 * shape, hem_cy + sa * 0.136 * shape, 1.006)))
    for i in range(h_segs):
        ni = (i + 1) % h_segs
        for q in [
            [cb_out0[i], cb_out0[ni], cb_out1[ni], cb_out1[i]],
            [cb_in0[ni], cb_in0[i], cb_in1[i], cb_in1[ni]],
            [cb_out1[i], cb_out1[ni], cb_in1[ni], cb_in1[i]],
            [cb_in0[i], cb_in0[ni], cb_out0[ni], cb_out0[i]],
        ]:
            fq = cbelt_bm.faces.new(q)
            fq.material_index = 1
            fq.smooth = True
    bmesh.ops.recalc_face_normals(cbelt_bm, faces=cbelt_bm.faces)
    append_bmesh_part(bm, cbelt_bm, 1)
    cbelt_bm.free()

    for side in (-1.0, 1.0):
        cuff_pts = [
            (side * 0.232, 0.012, 1.216),
            (side * 0.246, 0.014, 1.198),
        ]
        cuff_bm = build_tube_along_curve(cuff_pts, [(0.050, 0.050), (0.048, 0.048)], mat_slot=1, segs=14, cap_ends=False)
        append_bmesh_part(bm, cuff_bm, 1)
        cuff_bm.free()

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=False)
    mesh.update()
    transfer_weights_from(henley_obj, bpy.data.objects["Top_sweater"], arm_obj)
    return henley_obj


def build_bottom_tech_pants(arm_obj, mat_tech_dark):
    """
    Build Oblivion 'Bottom_tech_pants' from Bottom_cargo_pants with full proud 3D articulated
    charcoal knee armor pads, cargo pocket clasps, and tapered boot-tuck ankle cuffs.
    """
    pants_obj = duplicate_skinned_object("Bottom_cargo_pants", "Bottom_tech_pants")
    mesh = pants_obj.data

    for v in mesh.vertices:
        x, y, z = v.co.x, v.co.y, v.co.z
        w_waist = band_weight(z, 0.94, 1.00, 1.06, 1.09)
        x *= 1.0 + 0.09 * w_waist
        w_hip = band_weight(z, 0.82, 0.88, 0.95, 1.00)
        x *= 1.0 - 0.045 * w_hip

        # Taper lower pant cuff cleanly into the top of Shoe_flight_boots (Z = 0.22..0.28) so boot buckles shine!
        if z < 0.28:
            t_tuck = 1.0 - smoothstep(0.14, 0.28, z)
            ankle_cx = math.copysign(0.088, x)
            ankle_cy = -0.010
            x = ankle_cx + (x - ankle_cx) * (1.0 - 0.30 * t_tuck)
            y = ankle_cy + (y - ankle_cy) * (1.0 - 0.30 * t_tuck)
            if z < 0.215:
                z = 0.215 + (z - 0.215) * 0.25

        v.co.x, v.co.y, v.co.z = x, y, z

    while len(mesh.color_attributes) > 0:
        mesh.color_attributes.remove(mesh.color_attributes[0])

    mat_bot = bpy.data.materials["Mat_Avatar_Bottom"]
    mat_silver = bpy.data.materials["Mat_Avatar_Silver"]
    mesh.materials.clear()
    mesh.materials.append(mat_bot)        # slot 0: Primary flight trouser shell
    mesh.materials.append(mat_tech_dark)  # slot 1: 3D articulated knee armor & pocket flaps
    mesh.materials.append(mat_silver)     # slot 2: Silver tactical clasps

    bm = bmesh.new()
    bm.from_mesh(mesh)
    for f in bm.faces:
        f.material_index = 0
        f.smooth = True

    for side in (-1.0, 1.0):
        pad_bm = bmesh.new()
        bmesh.ops.create_cube(pad_bm, size=1.0)
        bmesh.ops.scale(pad_bm, vec=(0.086, 0.034, 0.112), verts=pad_bm.verts)
        bmesh.ops.bevel(pad_bm, geom=list(pad_bm.edges), offset=0.009, segments=2, profile=0.5, affect="EDGES")
        bmesh.ops.translate(pad_bm, vec=(side * 0.090, -0.116, 0.515), verts=pad_bm.verts)
        bmesh.ops.recalc_face_normals(pad_bm, faces=pad_bm.faces)
        append_bmesh_part(bm, pad_bm, 1)
        pad_bm.free()

        flap_bm = bmesh.new()
        bmesh.ops.create_cube(flap_bm, size=1.0)
        bmesh.ops.scale(flap_bm, vec=(0.028, 0.082, 0.032), verts=flap_bm.verts)
        bmesh.ops.bevel(flap_bm, geom=list(flap_bm.edges), offset=0.005, segments=1, profile=0.5, affect="EDGES")
        bmesh.ops.translate(flap_bm, vec=(side * 0.192, -0.020, 0.652), verts=flap_bm.verts)
        bmesh.ops.recalc_face_normals(flap_bm, faces=flap_bm.faces)
        append_bmesh_part(bm, flap_bm, 1)
        flap_bm.free()

        clasp_bm = bmesh.new()
        bmesh.ops.create_cube(clasp_bm, size=1.0)
        bmesh.ops.scale(clasp_bm, vec=(0.012, 0.024, 0.024), verts=clasp_bm.verts)
        bmesh.ops.translate(clasp_bm, vec=(side * 0.205, -0.020, 0.642), verts=clasp_bm.verts)
        bmesh.ops.recalc_face_normals(clasp_bm, faces=clasp_bm.faces)
        append_bmesh_part(bm, clasp_bm, 2)
        clasp_bm.free()

    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=False)
    mesh.update()
    transfer_weights_from(pants_obj, bpy.data.objects["Bottom_cargo_pants"], arm_obj)
    return pants_obj


def build_shoe_flight_boots(arm_obj, body_male_obj, mat_tech_dark):
    """
    Build 100% closed-toe, smooth rounded 3D armored tactical 'Shoe_flight_boots' enclosing Body_Male's
    feet completely (zero bare toe bumps!) with chunky charcoal lug soles, armored shin plates & dual silver buckles.
    """
    # Measure exact right-foot bounds from Body_Male (x > 0.01, z < 0.26) so clearance is 100% guaranteed!
    foot_verts = [v.co for v in body_male_obj.data.vertices if v.co.x > 0.01 and v.co.z < 0.25]
    foot_cx = sum(v.x for v in foot_verts) / len(foot_verts) if foot_verts else 0.090
    min_fy = min((v.y for v in foot_verts), default=-0.195)  # front toe tip (-Y)
    max_fy = max((v.y for v in foot_verts), default=0.075)   # heel back (+Y)
    half_fx = max((abs(v.x - foot_cx) for v in foot_verts), default=0.052)

    bm = bmesh.new()
    segs = 24

    for side in (-1.0, 1.0):
        cx = side * foot_cx
        # Build smooth multi-ring boot silhouette from sole bottom (Z=-0.004) up to armored calf shaft (Z=0.255)
        # Each station: (z, y_front, y_back, rx, mat_band)
        zf0 = min_fy - 0.014  # 14mm generous clearance ahead of bare toes!
        zb0 = max_fy + 0.010  # 10mm clearance behind heel
        rx_foot = half_fx + 0.012

        stations = [
            (-0.004, zf0 + 0.006, zb0 - 0.004, rx_foot + 0.002),  # 0: sole bottom bevel
            (0.004,  zf0,         zb0,         rx_foot + 0.006),  # 1: chunky lug sole outer flare
            (0.032,  zf0 + 0.002, zb0 - 0.002, rx_foot + 0.004),  # 2: sole top welt
            (0.056,  zf0 + 0.010, zb0 - 0.006, rx_foot),          # 3: rounded combat-boot toe box crown
            (0.088,  zf0 + 0.068, zb0 - 0.012, rx_foot - 0.005),  # 4: instep slope
            (0.128,  -0.068,      zb0 - 0.016, 0.056),            # 5: ankle collar
            (0.190,  -0.064,      zb0 - 0.018, 0.058),            # 6: mid shin shaft
            (0.252,  -0.062,      zb0 - 0.016, 0.062),            # 7: top boot cuff (overlaps pant hem!)
        ]

        rings = []
        for z_s, yf, yb, rx_s in stations:
            cy_s = 0.5 * (yf + yb)
            ry_s = 0.5 * (yb - yf)
            ring = []
            for k in range(segs):
                ang = (k / segs) * math.tau
                ca, sa = math.cos(ang), math.sin(ang)
                # Slightly square-round tactical toe box (superellipse exponent)
                sq = 1.0 + 0.07 * (ca ** 2) * (sa ** 2)
                px = cx + ca * rx_s * sq
                py = cy_s + sa * ry_s * sq
                ring.append(bm.verts.new((px, py, z_s)))
            rings.append(ring)

        for ri in range(len(rings) - 1):
            r0, r1 = rings[ri], rings[ri + 1]
            # Rings 0->2 are chunky charcoal lug sole (slot 1); rings 2->7 are boot upper (slot 0)
            m_idx = 1 if ri < 2 else 0
            for k in range(segs):
                kn = (k + 1) % segs
                f = bm.faces.new([r0[k], r0[kn], r1[kn], r1[k]])
                f.material_index = m_idx
                f.smooth = True

        # Cap sole bottom & shaft top cleanly
        f_bot = bm.faces.new(list(reversed(rings[0])))
        f_bot.material_index = 1
        f_bot.smooth = True
        f_top = bm.faces.new(rings[-1])
        f_top.material_index = 1
        f_top.smooth = True

        # Front armored charcoal shin guard tongue + dual brushed-silver tactical straps/buckles
        guard_bm = bmesh.new()
        bmesh.ops.create_cube(guard_bm, size=1.0)
        bmesh.ops.scale(guard_bm, vec=(0.062, 0.024, 0.115), verts=guard_bm.verts)
        bmesh.ops.bevel(guard_bm, geom=list(guard_bm.edges), offset=0.006, segments=2, profile=0.5, affect="EDGES")
        bmesh.ops.translate(guard_bm, vec=(cx, -0.068, 0.178), verts=guard_bm.verts)
        bmesh.ops.recalc_face_normals(guard_bm, faces=guard_bm.faces)
        append_bmesh_part(bm, guard_bm, 1)
        guard_bm.free()

        for bz in (0.142, 0.198):
            buckle_bm = bmesh.new()
            bmesh.ops.create_cube(buckle_bm, size=1.0)
            bmesh.ops.scale(buckle_bm, vec=(0.078, 0.028, 0.022), verts=buckle_bm.verts)
            bmesh.ops.bevel(buckle_bm, geom=list(buckle_bm.edges), offset=0.004, segments=1, profile=0.5, affect="EDGES")
            bmesh.ops.translate(buckle_bm, vec=(cx, -0.072, bz), verts=buckle_bm.verts)
            bmesh.ops.recalc_face_normals(buckle_bm, faces=buckle_bm.faces)
            append_bmesh_part(bm, buckle_bm, 2)
            buckle_bm.free()

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    set_bmesh_smooth(bm)

    mesh = bpy.data.meshes.new("Shoe_flight_boots_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=True)

    mat_shoe = bpy.data.materials["Mat_Avatar_Shoe"]
    mat_silver = bpy.data.materials["Mat_Avatar_Silver"]
    mesh.materials.append(mat_shoe)       # slot 0: Boot upper
    mesh.materials.append(mat_tech_dark)  # slot 1: Charcoal lug sole & shin armor plate
    mesh.materials.append(mat_silver)     # slot 2: Dual silver tactical shin buckles

    boots_obj = bpy.data.objects.new("Shoe_flight_boots", mesh)
    bpy.context.scene.collection.objects.link(boots_obj)
    transfer_weights_from(boots_obj, body_male_obj, arm_obj)
    return boots_obj


def build_acc_aviators(arm_obj, mat_aviator_lens):
    """
    Build Tom Cruise Oblivion / Maverick classic teardrop Aviator Sunglasses ('Acc_aviators')
    with brushed-silver double-bridge wireframe and dark smoky mirror lenses.
    """
    bm = bmesh.new()
    n_pts = 24
    for side in (-1.0, 1.0):
        lens_verts_f = []
        lens_verts_b = []

        cx = side * 0.0545
        cz = 1.621

        for i in range(n_pts):
            ang = (i / n_pts) * math.tau
            ca, sa = math.cos(ang), math.sin(ang)
            rx = 0.0445
            rz = 0.028 if sa > 0 else 0.038
            # Classic aviator teardrop dip toward outer-bottom cheek + straight flat brow top
            droop = -0.0075 * max(0.0, -sa) * (0.35 + 0.65 * max(0.0, ca))
            top_flat = -0.0055 * (max(0.0, sa) ** 2)

            lx_local = ca * rx
            lz = cz + sa * rz + droop + top_flat
            lx = cx + side * lx_local
            ly = -0.181 + 0.015 * ((abs(lx) / 0.098) ** 2)

            lens_verts_f.append(bm.verts.new((lx, ly - 0.0025, lz)))
            lens_verts_b.append(bm.verts.new((lx, ly + 0.0020, lz)))

        f_front = bm.faces.new(lens_verts_f if side > 0 else list(reversed(lens_verts_f)))
        f_front.material_index = 1
        f_front.smooth = True
        f_back = bm.faces.new(list(reversed(lens_verts_b)) if side > 0 else lens_verts_b)
        f_back.material_index = 1
        f_back.smooth = True

        rim_verts_out_f = []
        rim_verts_out_b = []
        for i in range(n_pts):
            vf = lens_verts_f[i].co
            vb = lens_verts_b[i].co
            rad_dir = Vector((vf.x - cx, 0.0, vf.z - cz)).normalized()
            rim_verts_out_f.append(bm.verts.new(vf + rad_dir * 0.0034 + Vector((0.0, -0.0010, 0.0))))
            rim_verts_out_b.append(bm.verts.new(vb + rad_dir * 0.0034 + Vector((0.0, 0.0010, 0.0))))

        for i in range(n_pts):
            ni = (i + 1) % n_pts
            for quad in [
                [lens_verts_f[i], lens_verts_f[ni], rim_verts_out_f[ni], rim_verts_out_f[i]],
                [rim_verts_out_f[i], rim_verts_out_f[ni], rim_verts_out_b[ni], rim_verts_out_b[i]],
                [rim_verts_out_b[i], rim_verts_out_b[ni], lens_verts_b[ni], lens_verts_b[i]],
            ]:
                fq = bm.faces.new(quad)
                fq.material_index = 0
                fq.smooth = True

        arm_bm = bmesh.new()
        bmesh.ops.create_cube(arm_bm, size=1.0)
        bmesh.ops.scale(arm_bm, vec=(0.0040, 0.145, 0.0050), verts=arm_bm.verts)
        bmesh.ops.translate(arm_bm, vec=(side * 0.102, -0.096, 1.628), verts=arm_bm.verts)
        bmesh.ops.recalc_face_normals(arm_bm, faces=arm_bm.faces)
        append_bmesh_part(bm, arm_bm, 0)
        arm_bm.free()

    for bar_z, bar_y, bar_w, bar_h in [
        (1.644, -0.181, 0.046, 0.0034),
        (1.629, -0.182, 0.028, 0.0038),
    ]:
        br_bm = bmesh.new()
        bmesh.ops.create_cube(br_bm, size=1.0)
        bmesh.ops.scale(br_bm, vec=(bar_w, 0.005, bar_h), verts=br_bm.verts)
        bmesh.ops.translate(br_bm, vec=(0.0, bar_y, bar_z), verts=br_bm.verts)
        bmesh.ops.recalc_face_normals(br_bm, faces=br_bm.faces)
        append_bmesh_part(bm, br_bm, 0)
        br_bm.free()

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Acc_aviators_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=True)
    mesh.materials.append(bpy.data.materials["Mat_Avatar_Silver"])
    mesh.materials.append(mat_aviator_lens)

    obj = bpy.data.objects.new("Acc_aviators", mesh)
    bpy.context.scene.collection.objects.link(obj)
    parent_to_head_bone_world(obj, arm_obj)
    return obj


def build_hat_recon_cap(arm_obj, mat_tech_dark):
    """Build Jack Harper's Oblivion curved-brim Recon Baseball Cap ('Hat_recon_cap') with generous hair clearance."""
    bm = bmesh.new()

    segs = 28
    rings = 8
    cy = -0.026
    base_z = 1.686
    top_z = 1.858
    rx0, ry0 = 0.138, 0.166

    crown_out = []
    crown_in = []
    for ri in range(rings):
        v_frac = ri / (rings - 1)
        phi = v_frac * (math.pi * 0.47)
        r_scale = math.cos(phi)
        z = base_z + math.sin(phi) * (top_z - base_z)
        r_o, r_i = [], []
        for si in range(segs):
            ang = (si / segs) * math.tau
            tilt_z = -0.012 * math.sin(ang) * (1.0 - v_frac)
            ca, sa = math.cos(ang), math.sin(ang)
            r_o.append(bm.verts.new((ca * rx0 * r_scale, cy + sa * ry0 * r_scale, z + tilt_z)))
            r_i.append(bm.verts.new((ca * (rx0 - 0.006) * r_scale, cy + sa * (ry0 - 0.006) * r_scale, z + tilt_z - 0.004 * v_frac)))
        crown_out.append(r_o)
        crown_in.append(r_i)

    apex_o = bm.verts.new((0.0, cy, top_z + 0.004))
    apex_i = bm.verts.new((0.0, cy, top_z - 0.003))
    for ri in range(rings - 1):
        for si in range(segs):
            ni = (si + 1) % segs
            fo = bm.faces.new([crown_out[ri][si], crown_out[ri][ni], crown_out[ri + 1][ni], crown_out[ri + 1][si]])
            fo.material_index = 0
            fo.smooth = True
            fi = bm.faces.new([crown_in[ri][ni], crown_in[ri][si], crown_in[ri + 1][si], crown_in[ri + 1][ni]])
            fi.material_index = 1
            fi.smooth = True
    for si in range(segs):
        ni = (si + 1) % segs
        f_ao = bm.faces.new([crown_out[-1][si], crown_out[-1][ni], apex_o])
        f_ao.material_index = 0
        f_ao.smooth = True
        f_ai = bm.faces.new([crown_in[-1][ni], crown_in[-1][si], apex_i])
        f_ai.material_index = 1
        f_ai.smooth = True
        f_rim = bm.faces.new([crown_in[0][si], crown_in[0][ni], crown_out[0][ni], crown_out[0][si]])
        f_rim.material_index = 1
        f_rim.smooth = True

    # Curved Forward Baseball Visor Brim
    brim_bm = bmesh.new()
    u_steps = 14
    v_steps = 6
    grid_top, grid_bot = [], []
    for vi in range(v_steps + 1):
        vf = vi / v_steps
        row_t, row_b = [], []
        half_w = 0.114 * (1.0 - 0.22 * (vf ** 1.6))
        y_pos = -0.168 - 0.115 * vf
        z_center = 1.696 - 0.016 * vf
        for ui in range(u_steps + 1):
            un = (ui / u_steps) * 2.0 - 1.0
            x = un * half_w
            arch = -0.022 * (un ** 2)
            tip_y_curve = 0.024 * (un ** 2) * vf
            row_t.append(brim_bm.verts.new((x, y_pos + tip_y_curve, z_center + arch + 0.0045)))
            row_b.append(brim_bm.verts.new((x, y_pos + tip_y_curve, z_center + arch - 0.0035)))
        grid_top.append(row_t)
        grid_bot.append(row_b)

    for vi in range(v_steps):
        for ui in range(u_steps):
            ft = brim_bm.faces.new([grid_top[vi][ui + 1], grid_top[vi][ui], grid_top[vi + 1][ui], grid_top[vi + 1][ui + 1]])
            ft.material_index = 0
            ft.smooth = True
            fb = brim_bm.faces.new([grid_bot[vi][ui], grid_bot[vi][ui + 1], grid_bot[vi + 1][ui + 1], grid_bot[vi + 1][ui]])
            fb.material_index = 1
            fb.smooth = True

    for ui in range(u_steps):
        fr = brim_bm.faces.new([grid_top[-1][ui], grid_bot[-1][ui], grid_bot[-1][ui + 1], grid_top[-1][ui + 1]])
        fr.material_index = 0
    for vi in range(v_steps):
        fl = brim_bm.faces.new([grid_top[vi + 1][0], grid_bot[vi + 1][0], grid_bot[vi][0], grid_top[vi][0]])
        fl.material_index = 0
        fr2 = brim_bm.faces.new([grid_top[vi][-1], grid_bot[vi][-1], grid_bot[vi + 1][-1], grid_top[vi + 1][-1]])
        fr2.material_index = 0

    bmesh.ops.recalc_face_normals(brim_bm, faces=brim_bm.faces)
    tmp_b = bpy.data.meshes.new("_tmp_brim")
    brim_bm.to_mesh(tmp_b)
    brim_bm.free()
    bm.from_mesh(tmp_b)
    bpy.data.meshes.remove(tmp_b)

    # Top button + Front Recon mission patch
    btn_bm = bmesh.new()
    bmesh.ops.create_icosphere(btn_bm, subdivisions=2, radius=0.010)
    bmesh.ops.scale(btn_bm, vec=(1.0, 1.0, 0.55), verts=btn_bm.verts)
    bmesh.ops.translate(btn_bm, vec=(0.0, cy, top_z + 0.005), verts=btn_bm.verts)
    append_bmesh_part(bm, btn_bm, 1)
    btn_bm.free()

    patch_bm = bmesh.new()
    bmesh.ops.create_cube(patch_bm, size=1.0)
    bmesh.ops.scale(patch_bm, vec=(0.056, 0.008, 0.032), verts=patch_bm.verts)
    bmesh.ops.bevel(patch_bm, geom=list(patch_bm.edges), offset=0.003, segments=1, profile=0.5, affect="EDGES")
    bmesh.ops.rotate(patch_bm, cent=(0, 0, 0), matrix=Euler((-0.34, 0.0, 0.0), "XYZ").to_matrix(), verts=patch_bm.verts)
    bmesh.ops.translate(patch_bm, vec=(0.0, -0.181, 1.752), verts=patch_bm.verts)
    bmesh.ops.recalc_face_normals(patch_bm, faces=patch_bm.faces)
    append_bmesh_part(bm, patch_bm, 2)
    patch_bm.free()

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Hat_recon_cap_Mesh")
    bm.to_mesh(mesh)
    bm.free()
    mesh.validate(clean_customdata=True)
    mesh.materials.append(bpy.data.materials["Mat_Avatar_Hat"])
    mesh.materials.append(mat_tech_dark)
    mesh.materials.append(bpy.data.materials["Mat_Avatar_Accent"])

    obj = bpy.data.objects.new("Hat_recon_cap", mesh)
    bpy.context.scene.collection.objects.link(obj)
    parent_to_head_bone_world(obj, arm_obj)
    return obj


def set_bsdf_color(mat_name: str, rgb):
    mat = bpy.data.materials.get(mat_name)
    if not mat or not mat.node_tree:
        return
    for n in mat.node_tree.nodes:
        if n.type == "BSDF_PRINCIPLED":
            n.inputs["Base Color"].default_value = (rgb[0], rgb[1], rgb[2], 1.0)
        elif n.type == "MIX" and len(n.inputs) > 7:
            n.inputs[7].default_value = (rgb[0], rgb[1], rgb[2], 1.0)


def render_verification_previews():
    """Render studio verification cards AFTER saving/exporting so temporary lights/cameras never pollute .blend."""
    os.makedirs(PREVIEW_DIR, exist_ok=True)
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.film_transparent = False

    if not scene.world:
        scene.world = bpy.data.worlds.new("StudioWorld")
    scene.world.use_nodes = True
    bg = scene.world.node_tree.nodes.get("Background")
    if bg:
        bg.inputs[0].default_value = (0.09, 0.12, 0.18, 1.0)
        bg.inputs[1].default_value = 1.0

    cam_data = bpy.data.cameras.new("VerifyCam")
    cam_data.lens = 52
    cam_obj = bpy.data.objects.new("VerifyCam", cam_data)
    scene.collection.objects.link(cam_obj)
    scene.camera = cam_obj

    for lname, loc, energy, col, rot in [
        ("VerifyKey", (1.65, -2.35, 2.35), 215.0, (1.0, 0.98, 0.94), (0.92, 0.0, 0.52)),
        ("VerifyFill", (-1.85, -1.95, 1.65), 115.0, (0.80, 0.90, 1.0), (1.0, 0.0, -0.55)),
        ("VerifyRim", (0.0, 1.85, 2.35), 155.0, (0.88, 0.95, 1.0), (-0.9, 0.0, 0.0)),
    ]:
        ldata = bpy.data.lights.new(lname, type="AREA")
        ldata.energy = energy
        ldata.size = 1.8
        ldata.color = col
        lobj = bpy.data.objects.new(lname, ldata)
        scene.collection.objects.link(lobj)
        lobj.location = loc
        lobj.rotation_euler = rot

    def show_only(slot_set):
        for o in bpy.data.objects:
            if o.type == "MESH":
                vis = o.name in slot_set
                o.hide_render = not vis
                o.hide_viewport = not vis

    set_bsdf_color("Mat_Avatar_Skin", (0.78, 0.56, 0.41))
    set_bsdf_color("Mat_Avatar_HairSolid", (0.12, 0.07, 0.04))
    set_bsdf_color("Mat_Avatar_Brow", (0.09, 0.05, 0.03))
    set_bsdf_color("Mat_Avatar_Lips", (0.64, 0.43, 0.36))
    set_bsdf_color("Mat_Avatar_EyeIris", (0.14, 0.52, 0.78))
    set_bsdf_color("Mat_Avatar_Top", (0.86, 0.89, 0.93))
    set_bsdf_color("Mat_Avatar_Bottom", (0.35, 0.41, 0.48))
    set_bsdf_color("Mat_Avatar_Shoe", (0.18, 0.22, 0.28))
    set_bsdf_color("Mat_Avatar_Accent", (0.10, 0.72, 0.96))
    set_bsdf_color("Mat_Avatar_Hat", (0.18, 0.25, 0.34))

    jack_suit = {
        "Body_Male",
        "Eye_L",
        "Eye_R",
        "Face_Brows_Male",
        "Face_SmileTeeth",
        "Hair_tech_crop",
        "Top_tech_jacket",
        "Bottom_tech_pants",
        "Shoe_flight_boots",
        "Acc_aviators",
    }

    # 1. Full Head-to-Toe Front — Jack Tech-49
    show_only(jack_suit)
    scene.render.resolution_x = 800
    scene.render.resolution_y = 1040
    cam_data.lens = 52
    cam_obj.location = (0.0, -3.10, 0.94)
    cam_obj.rotation_euler = (1.56, 0.0, 0.0)
    scene.render.filepath = os.path.join(PREVIEW_DIR, "jack_tech49_front.png")
    bpy.ops.render.render(write_still=True)

    # 2. Full Head-to-Toe 3/4 Hero Angle — Jack Tech-49
    cam_obj.location = (1.25, -2.85, 0.96)
    cam_obj.rotation_euler = (1.55, 0.0, 0.41)
    scene.render.filepath = os.path.join(PREVIEW_DIR, "jack_tech49_three_quarter.png")
    bpy.ops.render.render(write_still=True)

    # 3. Head & Collar Close-up WITHOUT sunglasses
    jack_no_shades = set(jack_suit) - {"Acc_aviators"}
    show_only(jack_no_shades)
    scene.render.resolution_x = 800
    scene.render.resolution_y = 850
    cam_data.lens = 68
    cam_obj.location = (0.16, -0.98, 1.60)
    cam_obj.rotation_euler = (1.56, 0.0, 0.16)
    scene.render.filepath = os.path.join(PREVIEW_DIR, "jack_face_closeup.png")
    bpy.ops.render.render(write_still=True)

    # 4. Head & Collar Close-up WITH Aviator Shades
    show_only(jack_suit)
    scene.render.filepath = os.path.join(PREVIEW_DIR, "jack_aviators_closeup.png")
    bpy.ops.render.render(write_still=True)

    # 5. Lakeside Cabin / Maverick Casual Boy Look (Full Head-to-Toe with Recon Cap, Maverick Quiff, Henley Tee, Dark Trousers & Flight Boots)
    set_bsdf_color("Mat_Avatar_Top", (0.85, 0.88, 0.91))
    set_bsdf_color("Mat_Avatar_Bottom", (0.14, 0.22, 0.34))
    set_bsdf_color("Mat_Avatar_Shoe", (0.18, 0.14, 0.11))
    maverick_look = {
        "Body_Male",
        "Eye_L",
        "Eye_R",
        "Face_Brows_Male",
        "Face_SmileTeeth",
        "Hair_maverick_quiff",
        "Hat_recon_cap",
        "Acc_aviators",
        "Top_hero_henley",
        "Bottom_tech_pants",
        "Shoe_flight_boots",
    }
    show_only(maverick_look)
    scene.render.resolution_x = 800
    scene.render.resolution_y = 1040
    cam_data.lens = 52
    cam_obj.location = (-0.95, -2.95, 0.95)
    cam_obj.rotation_euler = (1.56, 0.0, -0.31)
    scene.render.filepath = os.path.join(PREVIEW_DIR, "maverick_cabin_look.png")
    bpy.ops.render.render(write_still=True)


def main():
    print("[MaleWardrobe] Starting v3 full-coverage smooth Blender 5.2 male character & Oblivion Tech-49 build...")
    cleanup_previous_generated()

    arm_obj = bpy.data.objects["Ella_DTI_Armature"]
    prev_pose_pos = arm_obj.data.pose_position
    arm_obj.data.pose_position = "REST"
    bpy.context.view_layer.update()

    mat_tech_dark = ensure_material("Mat_Avatar_TechDark", (0.055, 0.075, 0.11), roughness=0.42, metallic=0.28)
    mat_aviator_lens = ensure_material("Mat_Avatar_AviatorLens", (0.02, 0.03, 0.05), roughness=0.06, metallic=0.65)

    body_male = build_body_male()
    build_face_brows_male(arm_obj)
    build_male_hairs(arm_obj, body_male)
    build_top_tech_jacket(arm_obj, mat_tech_dark)
    build_top_hero_henley(arm_obj, mat_tech_dark)
    build_bottom_tech_pants(arm_obj, mat_tech_dark)
    build_shoe_flight_boots(arm_obj, body_male, mat_tech_dark)
    build_acc_aviators(arm_obj, mat_aviator_lens)
    build_hat_recon_cap(arm_obj, mat_tech_dark)

    arm_obj.data.pose_position = prev_pose_pos
    bpy.context.view_layer.update()

    for o in bpy.data.objects:
        if o.type in ("MESH", "ARMATURE"):
            o.hide_viewport = False
            o.hide_render = False
            o.hide_set(False)

    print(f"[MaleWardrobe] Saving updated master Blend -> {BLEND_PATH}")
    bpy.ops.wm.save_as_mainfile(filepath=BLEND_PATH)

    print(f"[MaleWardrobe] Exporting WebGL runtime GLB -> {GLB_PATH}")
    bpy.ops.export_scene.gltf(
        filepath=GLB_PATH,
        export_format="GLB",
        export_yup=True,
        export_apply=False,
        export_skins=True,
        export_animations=True,
        export_animation_mode="NLA_TRACKS",
        export_texcoords=True,
        export_normals=True,
        export_attributes=True,
    )

    print("[MaleWardrobe] Rendering 5 EEVEE studio verification previews...")
    render_verification_previews()
    print("[MaleWardrobe] v2 Build & studio verification renders complete!")


if __name__ == "__main__":
    main()
