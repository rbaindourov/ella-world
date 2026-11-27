class_name AvatarRig
extends Node3D
## Shared Blender 5.2 DTI Wardrobe + Companion Rig (`AvatarRig.gd`)
## Loads `res://assets/models/ella_dti_wardrobe.glb` and `res://assets/models/ella_world_environment.glb`
## so Godot 4 Android players and Three.js Web players share identical modular slots, colors,
## fabric finishes (`matte`, `satin`, `glitter`), skeletal animations, 3D pets, and hoverboards.

static var _wardrobe_packed: PackedScene = null
static var _env_packed: PackedScene = null
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
var mount_pivot: Node3D = null
var nametag_label: Label3D = null
var voice_ring: MeshInstance3D = null

var _slot_meshes: Dictionary = {}
var _pet_bob_time: float = 0.0

const PET_NODE_MAP := {
	"cloud_puppy": "Env_Pet_CloudPuppy",
	"starlight_kitsune": "Env_Pet_StarlightKitsune",
	"shadow_kitty": "Env_Pet_ShadowKitty",
	"baby_crystal_dragon": "Env_Pet_BabyDragon",
	"baby_dragon": "Env_Pet_BabyDragon"
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
		_env_packed = load("res://assets/models/ella_world_environment.glb") as PackedScene

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

func _instantiate_wardrobe_or_fallback() -> void:
	if _wardrobe_packed != null:
		glb_instance = _wardrobe_packed.instantiate() as Node3D
		if glb_instance != null:
			visual_root.add_child(glb_instance)
			_index_glb_hierarchy(glb_instance)

	# Build lightweight blocky/procedural fallback too so `"blocky"` avatarMode works!
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

func _index_glb_hierarchy(node: Node) -> void:
	if node is AnimationPlayer and anim_player == null:
		anim_player = node as AnimationPlayer
	if node is MeshInstance3D:
		var mi := node as MeshInstance3D
		_slot_meshes[mi.name] = mi
		# Duplicate surface materials per avatar instance so recoloring one player never tints others
		if mi.mesh != null:
			for s_idx in range(mi.mesh.get_surface_count()):
				var base_mat := mi.get_active_material(s_idx)
				if base_mat != null:
					mi.set_surface_override_material(s_idx, base_mat.duplicate() as Material)
	for child in node.get_children():
		_index_glb_hierarchy(child)

func apply_avatar_appearance(av: Dictionary) -> void:
	current_avatar = av.duplicate(true)
	if visual_root == null:
		return

	var avatar_mode := str(av.get("avatarMode", "stylized")).to_lower()
	var body_type := str(av.get("bodyType", "female")).to_lower()
	var is_male := (body_type == "male")
	var skin_col := Color.from_string(str(av.get("skinColor", "#f6c8b6")), Color("#f6c8b6"))
	var eye_col := Color.from_string(str(av.get("eyeColor", "#38bdf8")), Color("#38bdf8"))
	var hair_style := str(av.get("hairStyle", "long_glam"))
	var hair_col := Color.from_string(str(av.get("hairColor", "#4a2511")), Color("#4a2511"))
	var top_style := str(av.get("topStyle", "cropped_puff"))
	var top_col := Color.from_string(str(av.get("topColor", av.get("torsoColor", "#f472b6"))), Color("#f472b6"))
	var bot_style := str(av.get("bottomStyle", "pleated_skirt"))
	var bot_col := Color.from_string(str(av.get("bottomColor", av.get("legsColor", "#60a5fa"))), Color("#60a5fa"))
	var shoe_style := str(av.get("shoeStyle", "platform_boots"))
	var shoe_col := Color.from_string(str(av.get("shoeColor", "#f8fafc")), Color("#f8fafc"))
	var hat_style := str(av.get("hat", "none"))
	var hat_col := Color.from_string(str(av.get("hatColor", "#fbbf24")), Color("#fbbf24"))
	var acc_style := str(av.get("accessory", "none"))
	var acc_col := Color.from_string(str(av.get("accessoryColor", "#38bdf8")), Color("#38bdf8"))
	var finish := str(av.get("fabricFinish", "matte")).to_lower()

	var rough := 0.58
	var metal := 0.05
	if finish == "satin":
		rough = 0.26
		metal = 0.22
	elif finish == "glitter":
		rough = 0.16
		metal = 0.58

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

	for mesh_name: String in _slot_meshes.keys():
		var mi: MeshInstance3D = _slot_meshes[mesh_name]
		if mesh_name == "Body_Stylized":
			mi.visible = not is_male
		elif mesh_name == "Body_Male":
			mi.visible = is_male
		elif mesh_name == "Face_Brows":
			mi.visible = not is_male
		elif mesh_name == "Face_Brows_Male":
			mi.visible = is_male
		elif mesh_name.begins_with("Hair_"):
			mi.visible = (mesh_name == "Hair_" + hair_style)
		elif mesh_name.begins_with("Top_"):
			mi.visible = (mesh_name == "Top_" + top_style)
		elif mesh_name.begins_with("Bottom_"):
			mi.visible = (mesh_name == "Bottom_" + bot_style)
		elif mesh_name.begins_with("Shoe_"):
			mi.visible = (mesh_name == "Shoe_" + shoe_style)
		elif mesh_name.begins_with("Hat_"):
			mi.visible = (hat_style != "none" and mesh_name == "Hat_" + hat_style)
		elif mesh_name.begins_with("Acc_"):
			mi.visible = (acc_style != "none" and mesh_name == "Acc_" + acc_style)

		_apply_wardrobe_surface_tints(mi, skin_col, eye_col, hair_col, top_col, bot_col, shoe_col, hat_col, acc_col, rough, metal)

	# Ensure fallback slot visibility if an unknown custom style key was passed
	_ensure_category_fallback("Hair_", "Hair_short_crop" if is_male else "Hair_long_glam")
	_ensure_category_fallback("Top_", "Top_tech_jacket" if is_male else "Top_cropped_puff")
	_ensure_category_fallback("Bottom_", "Bottom_tech_pants" if is_male else "Bottom_pleated_skirt")
	_ensure_category_fallback("Shoe_", "Shoe_flight_boots" if is_male else "Shoe_platform_boots")

func _ensure_category_fallback(prefix: String, default_slot: String) -> void:
	var any_vis := false
	for k: String in _slot_meshes.keys():
		if k.begins_with(prefix) and (_slot_meshes[k] as MeshInstance3D).visible:
			any_vis = true
			break
	if not any_vis and _slot_meshes.has(default_slot):
		(_slot_meshes[default_slot] as MeshInstance3D).visible = true

func _tint_node_material(mi: MeshInstance3D, col: Color, rough: float, metal: float) -> void:
	if mi == null or not (mi.material_override is StandardMaterial3D):
		return
	var sm := mi.material_override as StandardMaterial3D
	sm.albedo_color = col
	sm.roughness = rough
	sm.metallic = metal

func _apply_wardrobe_surface_tints(
	mi: MeshInstance3D,
	skin_col: Color,
	eye_col: Color,
	hair_col: Color,
	top_col: Color,
	bot_col: Color,
	shoe_col: Color,
	hat_col: Color,
	acc_col: Color,
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
		elif mname.contains("EyeIris"):
			sm.albedo_color = eye_col
		elif mname.contains("Hair") or mname.contains("Brow"):
			sm.albedo_color = hair_col
		elif mname.contains("Top"):
			sm.albedo_color = top_col
			sm.roughness = rough
			sm.metallic = metal
		elif mname.contains("Bottom"):
			sm.albedo_color = bot_col
			sm.roughness = rough
			sm.metallic = metal
		elif mname.contains("Shoe"):
			sm.albedo_color = shoe_col
		elif mname.contains("Hat"):
			sm.albedo_color = hat_col
		elif mname.contains("Accent") or mname.contains("Wings"):
			sm.albedo_color = acc_col

func set_companions(pet_id: String, mount_id: String) -> void:
	equipped_pet_id = pet_id
	equipped_mount_id = mount_id
	if pet_pivot == null or mount_pivot == null:
		return
	for c in pet_pivot.get_children():
		c.queue_free()
	for c in mount_pivot.get_children():
		c.queue_free()

	if not pet_id.is_empty():
		var pet_node := _clone_env_node(str(PET_NODE_MAP.get(pet_id, "")))
		if pet_node == null:
			pet_node = _make_box(Vector3(0.38, 0.34, 0.46), Vector3.ZERO, "#f9a8d4")
		pet_pivot.add_child(pet_node)

	if not mount_id.is_empty():
		var m_node := _clone_env_node(str(MOUNT_NODE_MAP.get(mount_id, "")))
		if m_node == null:
			m_node = _make_box(Vector3(0.85, 0.10, 1.65), Vector3(0, 0.08, 0), "#38bdf8")
		mount_pivot.add_child(m_node)
		visual_root.position.y = 0.18
	else:
		visual_root.position.y = 0.0

func _clone_env_node(target_name: String) -> Node3D:
	if target_name.is_empty() or _env_packed == null:
		return null
	var root := _env_packed.instantiate() as Node3D
	if root == null:
		return null
	var found := root.find_child(target_name, true, false) as Node3D
	var dup: Node3D = null
	if found != null:
		dup = found.duplicate() as Node3D
		dup.position = Vector3.ZERO
	root.free()
	return dup

func set_motion_and_emote(anim_key: String, emote_key: String) -> void:
	if current_anim_key == anim_key and current_emote_key == emote_key:
		return
	current_anim_key = anim_key
	current_emote_key = emote_key
	if anim_player == null:
		return
	var desired := "Idle_Glam"
	if not emote_key.is_empty():
		var cap := emote_key.substr(0, 1).to_upper() + emote_key.substr(1)
		if anim_player.has_animation("Emote_" + cap):
			desired = "Emote_" + cap
	else:
		match anim_key:
			"walk":
				desired = "Walk_Runway"
			"run", "hover":
				desired = "Run_Obby"
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
	if mount_pivot != null and mount_pivot.get_child_count() > 0:
		mount_pivot.position.y = 0.06 + sin(_pet_bob_time * 4.2) * 0.04
