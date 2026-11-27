class_name AvatarRig
extends Node3D
## Shared Blender 5.2 DTI Wardrobe + Companion Rig (`AvatarRig.gd`)
## Loads `res://assets/models/ella_dti_wardrobe.glb` and `res://assets/models/ella_world_environment.glb`
## so Godot 4 Android players and Three.js Web players share identical 1.55x world scale, modular slots,
## colors, fabric finishes (`matte`, `satin`, `glitter`), skeletal animations, 3D pets, and hoverboards.

static var _wardrobe_packed: PackedScene = null
static var _env_templates: Dictionary = {}
static var _assets_probed: bool = false

var peer_id: String = ""
var username: String = ""
var display_name: String = ""
var role: String = "friend"
var current_avatar: Dictionary = {}
var current_anim_key: String = "idle"
var current_emote_key: String = ""
var equipped_pet_id: String = ""
var equipped_mount_id: String = ""

var visual_root: Node3D = null
var glb_instance: Node3D = null
var anim_player: AnimationPlayer = null
var fallback_root: Node3D = null
var pet_pivot: Node3D = null
var pet_wing_l: Node3D = null
var pet_wing_r: Node3D = null
var mount_pivot: Node3D = null
var nametag_label: Label3D = null
var voice_ring: MeshInstance3D = null

var _slot_nodes: Dictionary = {} # canonical slot name -> Node3D
var _slot_meshes_map: Dictionary = {} # canonical slot name -> Array[MeshInstance3D]
var _pet_bob_time: float = 0.0

const PET_NODE_MAP := {
	"cloud_puppy": "Env_Pet_CloudPuppy",
	"starlight_kitsune": "Env_Pet_StarlightKitsune",
	"shadow_kitty": "Env_Pet_ShadowKitty"
}

const MOUNT_NODE_MAP := {
	"starlight_board": "Env_Mount_Hoverboard",
	"hoverboard": "Env_Mount_Hoverboard",
	"sunset_skates": "Env_Mount_StarlightSkates",
	"starlight_skates": "Env_Mount_StarlightSkates",
	"comet_surfer": "Env_Mount_CometSurfer"
}

static func _ensure_shared_packs() -> void:
	if _assets_probed:
		return
	_assets_probed = true
	if ResourceLoader.exists("res://assets/models/ella_dti_wardrobe.glb"):
		_wardrobe_packed = load("res://assets/models/ella_dti_wardrobe.glb") as PackedScene
	if ResourceLoader.exists("res://assets/models/ella_world_environment.glb"):
		var env_packed := load("res://assets/models/ella_world_environment.glb") as PackedScene
		if env_packed != null:
			var env_root := env_packed.instantiate() as Node3D
			if env_root != null:
				_index_env_templates(env_root)
				env_root.free()

static func _index_env_templates(node: Node) -> void:
	if node is Node3D and (node.name.begins_with("Env_Pet_") or node.name.begins_with("Env_Mount_")):
		var dup := (node as Node3D).duplicate() as Node3D
		dup.position = Vector3.ZERO
		_env_templates[str(node.name)] = dup
		return
	for c in node.get_children():
		_index_env_templates(c)

static func clone_companion_template(tpl_name: String) -> Node3D:
	_ensure_shared_packs()
	if tpl_name.is_empty() or not _env_templates.has(tpl_name):
		return null
	var src: Node3D = _env_templates[tpl_name]
	var dup := src.duplicate() as Node3D
	dup.position = Vector3.ZERO
	return dup

static func release_shared_caches() -> void:
	for k in _env_templates.keys():
		var n: Variant = _env_templates[k]
		if is_instance_valid(n):
			(n as Node).free()
	_env_templates.clear()
	_wardrobe_packed = null
	_assets_probed = false

func _ready() -> void:
	_ensure_shared_packs()
	visual_root = Node3D.new()
	visual_root.name = "VisualRoot"
	add_child(visual_root)

	mount_pivot = Node3D.new()
	mount_pivot.name = "MountPivot"
	add_child(mount_pivot)

	pet_pivot = Node3D.new()
	pet_pivot.name = "PetPivot"
	pet_pivot.position = Vector3(0.85, 0.95, -0.45)
	add_child(pet_pivot)

	_build_voice_ring_and_nametag()
	_instantiate_wardrobe_or_fallback()
	if not current_avatar.is_empty():
		apply_avatar_appearance(current_avatar)
	else:
		apply_avatar_appearance({})
	set_companions(equipped_pet_id, equipped_mount_id)

func _build_voice_ring_and_nametag() -> void:
	nametag_label = Label3D.new()
	nametag_label.position = Vector3(0.0, 2.38, 0.0)
	nametag_label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	nametag_label.font_size = 34
	nametag_label.outline_size = 10
	nametag_label.modulate = Color("#f8fafc")
	nametag_label.outline_modulate = Color("#0f172a")
	add_child(nametag_label)
	_refresh_nametag()

	voice_ring = MeshInstance3D.new()
	var torus := TorusMesh.new()
	torus.inner_radius = 0.52
	torus.outer_radius = 0.64
	voice_ring.mesh = torus
	var ring_mat := StandardMaterial3D.new()
	ring_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	ring_mat.albedo_color = Color("#34d399")
	voice_ring.material_override = ring_mat
	voice_ring.position = Vector3(0.0, 0.08, 0.0)
	voice_ring.visible = false
	add_child(voice_ring)

func set_player_identity(p_id: String, uname: String, dname: String, u_role: String) -> void:
	peer_id = p_id
	username = uname
	display_name = dname if not dname.is_empty() else uname
	role = u_role
	_refresh_nametag()

func _refresh_nametag() -> void:
	if nametag_label == null:
		return
	var prefix := "👑 " if role == "host" else ("⭐ " if role == "subadmin" else "")
	nametag_label.text = prefix + (display_name if not display_name.is_empty() else username)
	nametag_label.modulate = Color("#fde047") if role == "host" else Color("#f8fafc")

func set_speaking(speaking: bool) -> void:
	if voice_ring != null:
		voice_ring.visible = speaking

static func _is_canonical_slot_name(n: String) -> bool:
	return (
		n == "Body_Stylized"
		or n == "Body_Male"
		or n == "Face_Brows"
		or n == "Face_Brows_Male"
		or n == "Face_SmileTeeth"
		or n == "Eye_L"
		or n == "Eye_R"
		or n.begins_with("Hair_")
		or n.begins_with("Top_")
		or n.begins_with("Bottom_")
		or n.begins_with("Shoe_")
		or n.begins_with("Hat_")
		or n.begins_with("Acc_")
	)

func _instantiate_wardrobe_or_fallback() -> void:
	if _wardrobe_packed != null:
		glb_instance = _wardrobe_packed.instantiate() as Node3D
		if glb_instance != null:
			# Match exact 1.55x world scale used by StylizedCharacterRig in `public/js/world3d.js`
			glb_instance.scale = Vector3(1.55, 1.55, 1.55)
			visual_root.add_child(glb_instance)
			_clone_all_mesh_materials(glb_instance)
			_index_canonical_slots(glb_instance, "")

	fallback_root = Node3D.new()
	fallback_root.name = "BlockyFallback"
	var torso := _make_box(Vector3(0.72, 0.86, 0.38), Vector3(0, 1.12, 0), "#f472b6")
	torso.name = "BlockyTorso"
	var head := _make_box(Vector3(0.56, 0.56, 0.56), Vector3(0, 1.86, 0), "#f8d4b4")
	head.name = "BlockyHead"
	var legs := _make_box(Vector3(0.68, 0.68, 0.36), Vector3(0, 0.35, 0), "#60a5fa")
	legs.name = "BlockyLegs"
	fallback_root.add_child(torso)
	fallback_root.add_child(head)
	fallback_root.add_child(legs)
	visual_root.add_child(fallback_root)
	fallback_root.visible = (glb_instance == null)

func _make_box(sz: Vector3, pos: Vector3, hex_col: String) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = sz
	mi.mesh = bm
	mi.position = pos
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color.from_string(hex_col, Color.WHITE)
	mat.roughness = 0.55
	mi.material_override = mat
	return mi

func _clone_all_mesh_materials(node: Node) -> void:
	if node is AnimationPlayer and anim_player == null:
		anim_player = node as AnimationPlayer
	if node is MeshInstance3D:
		var mi := node as MeshInstance3D
		if mi.mesh != null:
			for s_idx in range(mi.mesh.get_surface_count()):
				var base_mat := mi.get_active_material(s_idx)
				if base_mat != null:
					var dup_mat := base_mat.duplicate() as Material
					dup_mat.resource_name = base_mat.resource_name
					mi.set_surface_override_material(s_idx, dup_mat)
	for child in node.get_children():
		_clone_all_mesh_materials(child)

func _index_canonical_slots(node: Node, active_slot: String) -> void:
	var slot_key := active_slot
	var nname := str(node.name)
	if slot_key.is_empty() and node is Node3D and _is_canonical_slot_name(nname):
		slot_key = nname
		_slot_nodes[slot_key] = node as Node3D
		_slot_meshes_map[slot_key] = []
	if not slot_key.is_empty() and node is MeshInstance3D:
		(_slot_meshes_map[slot_key] as Array).append(node as MeshInstance3D)
	for child in node.get_children():
		_index_canonical_slots(child, slot_key)

func apply_avatar_appearance(av: Dictionary) -> void:
	current_avatar = av.duplicate(true)
	if visual_root == null:
		return

	var avatar_mode := str(av.get("avatarMode", "stylized")).to_lower()
	var preset := str(av.get("characterPreset", "ella_idol")).to_lower()
	var body_type := str(av.get("bodyType", "male" if (preset == "jack_tech49" or preset == "maverick_cabin") else "female")).to_lower()
	var is_male := (body_type == "male")
	var skin_col := Color.from_string(str(av.get("skinColor", "#ffdfc4")), Color("#ffdfc4"))
	var eye_col := Color.from_string(str(av.get("eyeColor", "#60a5fa")), Color("#60a5fa"))
	var hair_style := str(av.get("hairStyle", "tech_crop" if is_male else "twin_pigtails"))
	var hair_col := Color.from_string(str(av.get("hairColor", "#fde047")), Color("#fde047"))
	var top_style := str(av.get("topStyle", "tech_jacket" if is_male else "glam_gown"))
	var top_col := Color.from_string(str(av.get("topColor", av.get("torsoColor", "#f472b6"))), Color("#f472b6"))
	var bot_style := str(av.get("bottomStyle", "tech_pants" if is_male else "pleated_skirt"))
	var bot_col := Color.from_string(str(av.get("bottomColor", av.get("legsColor", "#6366f1"))), Color("#6366f1"))
	var shoe_style := str(av.get("shoeStyle", "flight_boots" if is_male else "runway_heels"))
	var shoe_col := Color.from_string(str(av.get("shoeColor", "#fbbf24")), Color("#fbbf24"))
	var hat_style := str(av.get("hat", "none" if is_male else "crown"))
	var hat_col := Color.from_string(str(av.get("hatColor", "#fbbf24")), Color("#fbbf24"))
	var acc_style := str(av.get("accessory", "aviators" if is_male else "star_purse"))
	var acc_col := Color.from_string(str(av.get("accessoryColor", "#ec4899")), Color("#ec4899"))
	var finish := str(av.get("fabricFinish", "glitter")).to_lower()
	var is_full_gown := (top_style == "glam_gown")

	var rough := 0.58
	var metal := 0.05
	if finish == "satin":
		rough = 0.26
		metal = 0.22
	elif finish == "glitter":
		rough = 0.16
		metal = 0.52

	var use_glb := (glb_instance != null and avatar_mode != "blocky")
	if glb_instance != null:
		glb_instance.visible = use_glb
	if fallback_root != null:
		fallback_root.visible = not use_glb
		if not use_glb:
			_tint_node_material(fallback_root.get_node_or_null("BlockyHead") as MeshInstance3D, skin_col, 0.55, 0.0)
			_tint_node_material(fallback_root.get_node_or_null("BlockyTorso") as MeshInstance3D, top_col, rough, metal)
			_tint_node_material(fallback_root.get_node_or_null("BlockyLegs") as MeshInstance3D, bot_col, rough, metal)
			return

	for slot_name: String in _slot_nodes.keys():
		var snode: Node3D = _slot_nodes[slot_name]
		var vis := false
		if slot_name == "Body_Stylized" or slot_name == "Face_Brows":
			vis = not is_male
		elif slot_name == "Body_Male" or slot_name == "Face_Brows_Male":
			vis = is_male
		elif slot_name == "Face_SmileTeeth" or slot_name == "Eye_L" or slot_name == "Eye_R":
			vis = true
		elif slot_name.begins_with("Hair_"):
			vis = (slot_name == "Hair_" + hair_style)
		elif slot_name.begins_with("Top_"):
			vis = (slot_name == "Top_" + top_style)
		elif slot_name.begins_with("Bottom_"):
			vis = (not is_full_gown and bot_style != "none" and slot_name == "Bottom_" + bot_style)
		elif slot_name.begins_with("Shoe_"):
			vis = (slot_name == "Shoe_" + shoe_style)
		elif slot_name.begins_with("Hat_"):
			vis = (hat_style != "none" and hat_style != "wings" and slot_name == "Hat_" + hat_style)
		elif slot_name == "Acc_wings":
			vis = (acc_style == "wings" or hat_style == "wings")
		elif slot_name.begins_with("Acc_"):
			vis = (acc_style != "none" and slot_name == "Acc_" + acc_style)

		snode.visible = vis
		for mi_var in (_slot_meshes_map.get(slot_name, []) as Array):
			var mi := mi_var as MeshInstance3D
			mi.visible = vis
			_apply_wardrobe_surface_tints(slot_name, mi, is_male, skin_col, eye_col, hair_col, top_col, bot_col, shoe_col, hat_col, acc_col, acc_style != "none", rough, metal)

	_ensure_category_fallback("Hair_", "Hair_tech_crop" if is_male else "Hair_twin_pigtails")
	_ensure_category_fallback("Top_", "Top_tech_jacket" if is_male else "Top_glam_gown")
	if not is_full_gown:
		_ensure_category_fallback("Bottom_", "Bottom_tech_pants" if is_male else "Bottom_pleated_skirt")
	_ensure_category_fallback("Shoe_", "Shoe_flight_boots" if is_male else "Shoe_runway_heels")

func _ensure_category_fallback(prefix: String, default_slot: String) -> void:
	var any_vis := false
	for k: String in _slot_nodes.keys():
		if k.begins_with(prefix) and (_slot_nodes[k] as Node3D).visible:
			any_vis = true
			break
	if not any_vis and _slot_nodes.has(default_slot):
		(_slot_nodes[default_slot] as Node3D).visible = true
		for mi_var in (_slot_meshes_map.get(default_slot, []) as Array):
			(mi_var as MeshInstance3D).visible = true

func is_slot_visible(slot_name: String) -> bool:
	return _slot_nodes.has(slot_name) and (_slot_nodes[slot_name] as Node3D).visible

func _tint_node_material(mi: MeshInstance3D, col: Color, rough: float, metal: float) -> void:
	if mi == null or not (mi.material_override is StandardMaterial3D):
		return
	var sm := mi.material_override as StandardMaterial3D
	sm.albedo_color = col
	sm.roughness = rough
	sm.metallic = metal

func _apply_wardrobe_surface_tints(
	slot_name: String,
	mi: MeshInstance3D,
	is_male: bool,
	skin_col: Color,
	eye_col: Color,
	hair_col: Color,
	top_col: Color,
	bot_col: Color,
	shoe_col: Color,
	hat_col: Color,
	acc_col: Color,
	has_acc: bool,
	rough: float,
	metal: float
) -> void:
	if mi.mesh == null:
		return
	for s_idx in range(mi.mesh.get_surface_count()):
		var mat := mi.get_surface_override_material(s_idx)
		if not (mat is StandardMaterial3D):
			continue
		var sm := mat as StandardMaterial3D
		var mname := sm.resource_name
		if mname.contains("Skin"):
			sm.albedo_color = skin_col
			sm.roughness = 0.46
			sm.metallic = 0.02
		elif mname.contains("Lips"):
			sm.albedo_color = (skin_col * 0.76) if is_male else Color("#d85c7a")
			sm.albedo_color.a = 1.0
		elif mname.contains("EyeIris"):
			sm.albedo_color = eye_col
			sm.roughness = 0.08
		elif mname.contains("Brow"):
			sm.albedo_color = hair_col * 0.58
			sm.albedo_color.a = 1.0
		elif mname.contains("Hair"):
			sm.albedo_color = hair_col
			sm.roughness = 0.34
		elif mname.contains("Top"):
			sm.albedo_color = top_col
			sm.roughness = rough
			sm.metallic = metal
		elif mname.contains("Bottom"):
			sm.albedo_color = bot_col
			sm.roughness = rough
			sm.metallic = metal
		elif mname.contains("Shoe") and not mname.contains("ShoeSole"):
			sm.albedo_color = shoe_col
			sm.roughness = 0.22 if metal > 0.4 else 0.34
		elif mname.contains("Hat"):
			sm.albedo_color = hat_col
			sm.roughness = 0.28
			sm.metallic = 0.32
		elif mname.contains("Accent"):
			if slot_name.begins_with("Hair_"):
				sm.albedo_color = acc_col if has_acc else top_col
			elif slot_name.begins_with("Top_") or slot_name.begins_with("Bottom_"):
				sm.albedo_color = acc_col if has_acc else hat_col
				sm.roughness = rough
				sm.metallic = metal
			else:
				sm.albedo_color = acc_col
		elif mname.contains("Wings"):
			sm.albedo_color = acc_col

func set_companions(pet_id: String, mount_id: String) -> void:
	equipped_pet_id = pet_id
	equipped_mount_id = mount_id
	pet_wing_l = null
	pet_wing_r = null
	if pet_pivot == null or mount_pivot == null:
		return
	for c in pet_pivot.get_children():
		c.queue_free()
	for c in mount_pivot.get_children():
		c.queue_free()

	if not pet_id.is_empty():
		var pet_node: Node3D = null
		if pet_id == "baby_crystal_dragon" or pet_id == "baby_dragon":
			var body := clone_companion_template("Env_Pet_BabyDragon_Body")
			if body != null:
				var dragon_grp := Node3D.new()
				dragon_grp.name = "ArticulatedBabyDragon"
				dragon_grp.add_child(body)
				pet_wing_l = clone_companion_template("Env_Pet_BabyDragon_WingL")
				pet_wing_r = clone_companion_template("Env_Pet_BabyDragon_WingR")
				if pet_wing_l != null:
					dragon_grp.add_child(pet_wing_l)
				if pet_wing_r != null:
					dragon_grp.add_child(pet_wing_r)
				pet_node = dragon_grp
		else:
			pet_node = clone_companion_template(str(PET_NODE_MAP.get(pet_id, "")))
		if pet_node == null:
			pet_node = _make_box(Vector3(0.38, 0.34, 0.46), Vector3.ZERO, "#f9a8d4")
		pet_pivot.add_child(pet_node)

	if not mount_id.is_empty():
		var m_node := clone_companion_template(str(MOUNT_NODE_MAP.get(mount_id, "")))
		if m_node == null:
			m_node = _make_box(Vector3(0.85, 0.10, 1.65), Vector3(0, 0.08, 0), "#38bdf8")
		mount_pivot.add_child(m_node)
	_apply_sleep_or_mount_pose()

func _apply_sleep_or_mount_pose() -> void:
	if visual_root == null:
		return
	if current_emote_key == "sleep":
		visual_root.rotation.x = -PI * 0.47
		visual_root.position = Vector3(0.0, 0.56, 0.36)
	else:
		visual_root.rotation.x = 0.0
		visual_root.position = Vector3(0.0, 0.18 if not equipped_mount_id.is_empty() else 0.0, 0.0)

func set_motion_and_emote(anim_key: String, emote_key: String) -> void:
	if current_anim_key == anim_key and current_emote_key == emote_key:
		return
	current_anim_key = anim_key
	current_emote_key = emote_key
	_apply_sleep_or_mount_pose()

	if anim_player == null:
		return
	var desired := "Idle_Glam"
	if not emote_key.is_empty():
		if emote_key == "sleep":
			desired = "Idle_Glam"
		else:
			var cap := emote_key.substr(0, 1).to_upper() + emote_key.substr(1)
			if anim_player.has_animation("Emote_" + cap):
				desired = "Emote_" + cap
	else:
		match anim_key:
			"walk":
				desired = "Walk_Runway"
			"run":
				desired = "Run_Obby"
			"hover":
				desired = "Emote_Pose" if anim_player.has_animation("Emote_Pose") else "Idle_Glam"
			"jump":
				desired = "Jump_Air"
			_:
				desired = "Idle_Glam"
	if anim_player.has_animation(desired) and anim_player.current_animation != desired:
		anim_player.play(desired, 0.18)

func _process(delta: float) -> void:
	_pet_bob_time += delta
	if pet_pivot != null and pet_pivot.get_child_count() > 0:
		pet_pivot.position.y = 0.95 + sin(_pet_bob_time * 3.1) * 0.12
		pet_pivot.rotation.y = sin(_pet_bob_time * 1.4) * 0.25
		if pet_wing_l != null:
			pet_wing_l.rotation.z = sin(_pet_bob_time * 9.5) * 0.38
		if pet_wing_r != null:
			pet_wing_r.rotation.z = -sin(_pet_bob_time * 9.5) * 0.38
	if mount_pivot != null and mount_pivot.get_child_count() > 0:
		mount_pivot.position.y = 0.06 + sin(_pet_bob_time * 4.2) * 0.04
