class_name WorldController
extends Node3D
## Cross-Play 3D World Engine (`WorldController.gd`)
## Implements exact coordinate, 1-indexed voxel, physics, GLB asset, and gameplay parity with
## `public/js/world3d.js` and `src/world.rs`:
## - Zone 0: 210x210 Starlight Plaza, Sky Obby (`easy`) + Pro Obby Gauntlet (`hard`) with PBR textures,
##   moving/blinking/bounce/sweeper platforms, Speedrun Start/Finish gates & 3D Top-5 Leaderboards,
##   DTI Fashion Runway, Starlight Crafting Bench, 6 Claimable Home Plots + Mailboxes + Doorbells +
##   1-indexed Voxel/Prop/Interactive Furniture (`sit`, `sleep`, door/lamp `toggle`) & Build Mode,
##   synced Day/Sunset/Aurora Night sky + Falling Star Shards.
## - Zone 1: Crystal Cavern (stepped amethyst/cyan terraces, crystal veins, chests, sculpted slimes & bats, drops).
## - Zone 2: Debtor's Yard (community-service chores & release gate).
## - Zone 3: Shadow Rift Mortal Arena (molten lava river, bridges, Sanctuary, sculpted spiders/wraiths/golems,
##   100/150 HP combat & respawn, Co-Op Obsidian Rift Dragon Boss + multi-ring shockwaves).

signal context_hint_changed(hint_text: String)
signal toast_requested(message: String)
signal stats_updated(coins: int, hp: int, max_hp: int, zone: int, speedrun_label: String)

const ZONE_PLAZA: int = 0
const ZONE_DUNGEON: int = 1
const ZONE_PRISON: int = 2
const ZONE_ARENA: int = 3

const PLAZA_PORTAL_POS := Vector3(18.0, 0.0, 16.0)
const DUNGEON_RETURN_POS := Vector3(0.0, 0.0, 28.0)
const PLAZA_ARENA_PORTAL_POS := Vector3(-22.0, 0.0, 28.0)
const ARENA_RETURN_POS := Vector3(0.0, 0.0, 30.0)
const PRISON_RELEASE_POS := Vector3(0.0, 0.0, 11.5)
const BOSS_ALTAR_POS := Vector3(0.0, 0.0, -24.5)
const CRAFTING_BENCH_POS := Vector3(12.5, 0.0, 6.5)

const HOME_PLOTS_STATIC := [
	{"id": "plot_1", "name": "🏡 Sunset Plot #1", "cx": 52, "cz": -26, "half": 9, "mbox": Vector3(52.0, 0.0, -16.2)},
	{"id": "plot_2", "name": "🌸 Rosewater Villa", "cx": 82, "cz": -26, "half": 9, "mbox": Vector3(79.0, 0.0, -16.2)},
	{"id": "plot_3", "name": "🌊 Starlight Cottage", "cx": 52, "cz": 20, "half": 9, "mbox": Vector3(49.5, 0.0, 10.2)},
	{"id": "plot_4", "name": "🏡 Starlight Plot #4", "cx": 82, "cz": 20, "half": 9, "mbox": Vector3(82.0, 0.0, 10.2)},
	{"id": "plot_5", "name": "🌲 Woodland Chalet", "cx": -64, "cz": -16, "half": 10, "mbox": Vector3(-61.5, 0.0, -5.2)},
	{"id": "plot_6", "name": "🏡 West Villa Plot #6", "cx": -64, "cz": 32, "half": 10, "mbox": Vector3(-64.0, 0.0, 21.2)}
]

const PROP_NODE_MAP := {
	"stairs": "Prop_Stairs",
	"kitchen_counter": "Prop_Kitchen_Counter",
	"kitchen_sink": "Prop_Kitchen_Sink",
	"kitchen_stove": "Prop_Kitchen_Stove",
	"kitchen_fridge": "Prop_Kitchen_Fridge",
	"dining_table": "Prop_Dining_Table",
	"chair": "Prop_Chair",
	"bath_tub": "Prop_Bath_Tub",
	"bath_vanity": "Prop_Bath_Vanity",
	"bath_toilet": "Prop_Bath_Toilet",
	"bed": "Prop_Bed",
	"sofa": "Prop_Sofa",
	"nightstand_lamp": "Prop_Nightstand_Lamp",
	"crystal_lamp": "Prop_Crystal_Lamp",
	"wardrobe": "Prop_Wardrobe",
	"rug": "Prop_Rug"
}

const LOOT_NODE_MAP := {
	"legendary_ray_gun": "Loot_Ray_Gun",
	"crystal_blade": "Loot_Crystal_Blade",
	"starlight_wand": "Loot_Starlight_Wand",
	"aegis_heart_charm": "Loot_Aegis_Charm",
	"rift_beacon": "Loot_Rift_Beacon",
	"pet_egg": "Loot_Pet_Egg",
	"aurora_crystal_lamp": "Loot_Aurora_Lamp",
	"geode_lamp": "Loot_Geode_Lamp",
	"slime_lava_lamp": "Loot_Slime_Lamp",
	"bat_plushie": "Loot_Bat_Plush",
	"royal_chalice": "Loot_Royal_Chalice",
	"dragon_egg_relic": "Loot_Dragon_Egg"
}

var net: NetworkClient = null
var touch: MultiTouchController = null

var world_env: WorldEnvironment = null
var sun_light: DirectionalLight3D = null
var main_camera: Camera3D = null

var plaza_root: Node3D = null
var dungeon_root: Node3D = null
var prison_root: Node3D = null
var arena_root: Node3D = null
var voxels_root: Node3D = null
var dynamic_entities_root: Node3D = null
var sky_shards_root: Node3D = null
var boss_shockwaves_root: Node3D = null

var _env_templates: Dictionary = {}
var _prop_templates: Dictionary = {}
var wormhole_shader: Shader = null
var obby_textures: Dictionary = {}

var colliders_by_zone: Dictionary = {0: [], 1: [], 2: [], 3: []}
var moving_platforms: Array = []
var blinking_platforms: Array = []
var jump_pads: Array = []
var sweeper_hazards: Array = []
var leaderboard_labels: Dictionary = {}
var mailbox_labels: Dictionary = {}

var self_rig: AvatarRig = null
var remote_rigs: Dictionary = {} # peer_id -> {rig, target_pos, target_yaw, zone}
var voxel_nodes: Dictionary = {} # "x,y,z" -> Node3D
var voxel_sigs: Dictionary = {} # "x,y,z" -> String
var saved_blocks: Dictionary = {} # "x,y,z" -> Dictionary

var active_zone: int = ZONE_PLAZA
var player_pos: Vector3 = Vector3(0.0, 0.2, 4.0)
var player_vel: Vector3 = Vector3.ZERO
var player_yaw: float = 0.0
var is_grounded: bool = true
var active_emote: String = ""
var emote_timer: float = 0.0
var jump_queued: bool = false
var state_send_accum: float = 0.0
var last_portal_ms: int = 0
var checkpoint_pos: Vector3 = Vector3(0.0, 0.5, 4.0)

var coins_balance: int = 120
var arena_hp: int = 100
var arena_max_hp: int = 100
var arena_dead: bool = false
var prison_tasks_remaining: int = 0
var in_debtors_prison: bool = false
var inventory_items: Dictionary = {}
var owned_pets: Array = []
var owned_mounts: Array = []
var equipped_pet: String = ""
var equipped_mount: String = ""
var home_plots_state: Array = []
var runway_state: Dictionary = {}
var obby_leaderboards: Dictionary = {"easy": [], "hard": []}
var active_speedrun_course: String = ""
var active_speedrun_start_ms: int = 0

var build_mode: bool = false
var build_tool: String = "place" # "place" | "remove"
var selected_block_kind: String = "block"
var selected_block_color: String = "#f472b6"
var selected_block_rot: int = 0

var dungeon_nodes: Dictionary = {}
var dungeon_mobs: Dictionary = {}
var dungeon_drops: Dictionary = {}
var arena_mobs: Dictionary = {}
var arena_boss: Dictionary = {}
var prison_nodes: Dictionary = {}
var sky_falling_stars: Array = []

var entity_meshes: Dictionary = {} # key -> Node3D
var boss_shockwave_pool: Array[MeshInstance3D] = []

func setup(network_client: NetworkClient, touch_ctrl: MultiTouchController) -> void:
	net = network_client
	touch = touch_ctrl
	net.welcome_received.connect(_on_welcome_snapshot)
	net.world_event_received.connect(_on_world_event)
	net.remote_states_received.connect(_on_remote_states)
	touch.world_tap_requested.connect(_on_world_tap)

func _ready() -> void:
	_load_shared_resources()
	_setup_lighting_and_camera()
	_build_zone_roots()
	_build_plaza_zone()
	_build_dungeon_zone()
	_build_prison_zone()
	_build_arena_zone()
	_spawn_local_avatar()
	_set_zone_visibility(ZONE_PLAZA)

func _exit_tree() -> void:
	for k in _env_templates.keys():
		var n: Node = _env_templates[k]
		if is_instance_valid(n):
			n.free()
	_env_templates.clear()
	for k in _prop_templates.keys():
		var n: Node = _prop_templates[k]
		if is_instance_valid(n):
			n.free()
	_prop_templates.clear()

func _load_shared_resources() -> void:
	if ResourceLoader.exists("res://assets/models/ella_world_environment.glb"):
		var env_packed := load("res://assets/models/ella_world_environment.glb") as PackedScene
		if env_packed != null:
			var env_inst := env_packed.instantiate() as Node3D
			if env_inst != null:
				_index_named_templates(env_inst, "Env_", _env_templates)
				env_inst.free()
	if ResourceLoader.exists("res://assets/models/ella_world_props_loot.glb"):
		var props_packed := load("res://assets/models/ella_world_props_loot.glb") as PackedScene
		if props_packed != null:
			var props_inst := props_packed.instantiate() as Node3D
			if props_inst != null:
				_index_named_templates(props_inst, "Prop_", _prop_templates)
				_index_named_templates(props_inst, "Loot_", _prop_templates)
				props_inst.free()
	if ResourceLoader.exists("res://shaders/StargateWormhole.gdshader"):
		wormhole_shader = load("res://shaders/StargateWormhole.gdshader") as Shader
	for style in ["hex_scifi", "marble_tiles", "metal_plate", "red_brick"]:
		var diff_path := "res://assets/textures/obby/%s_diff.jpg" % style
		if ResourceLoader.exists(diff_path):
			obby_textures[style] = load(diff_path) as Texture2D

func _index_named_templates(node: Node, prefix: String, target_dict: Dictionary) -> void:
	var nname := str(node.name)
	if node is Node3D and nname.begins_with(prefix):
		var dup := (node as Node3D).duplicate() as Node3D
		dup.position = Vector3.ZERO
		target_dict[nname] = dup
		return
	for c in node.get_children():
		_index_named_templates(c, prefix, target_dict)

func clone_env_template(tpl_name: String) -> Node3D:
	if tpl_name.is_empty() or not _env_templates.has(tpl_name):
		return null
	var src: Node3D = _env_templates[tpl_name]
	var dup := src.duplicate() as Node3D
	dup.position = Vector3.ZERO
	return dup

func clone_prop_template(tpl_name: String) -> Node3D:
	if tpl_name.is_empty() or not _prop_templates.has(tpl_name):
		return null
	var src: Node3D = _prop_templates[tpl_name]
	var dup := src.duplicate() as Node3D
	dup.position = Vector3.ZERO
	return dup

func _setup_lighting_and_camera() -> void:
	world_env = WorldEnvironment.new()
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color("#7dd3fc")
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color("#e2e8f0")
	env.ambient_light_energy = 0.72
	env.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	world_env.environment = env
	add_child(world_env)

	sun_light = DirectionalLight3D.new()
	sun_light.light_color = Color("#fffbeb")
	sun_light.light_energy = 1.25
	sun_light.rotation_degrees = Vector3(-48.0, 36.0, 0.0)
	sun_light.shadow_enabled = true
	add_child(sun_light)

	main_camera = Camera3D.new()
	main_camera.fov = 64.0
	main_camera.near = 0.15
	main_camera.far = 260.0
	main_camera.current = true
	add_child(main_camera)

func _build_zone_roots() -> void:
	plaza_root = Node3D.new()
	plaza_root.name = "PlazaZone"
	dungeon_root = Node3D.new()
	dungeon_root.name = "DungeonZone"
	prison_root = Node3D.new()
	prison_root.name = "PrisonZone"
	arena_root = Node3D.new()
	arena_root.name = "ArenaZone"
	voxels_root = Node3D.new()
	voxels_root.name = "SavedVoxels"
	dynamic_entities_root = Node3D.new()
	dynamic_entities_root.name = "DynamicEntities"
	sky_shards_root = Node3D.new()
	sky_shards_root.name = "SkyShards"
	boss_shockwaves_root = Node3D.new()
	boss_shockwaves_root.name = "BossShockwaves"

	plaza_root.add_child(voxels_root)
	plaza_root.add_child(sky_shards_root)
	arena_root.add_child(boss_shockwaves_root)
	add_child(plaza_root)
	add_child(dungeon_root)
	add_child(prison_root)
	add_child(arena_root)
	add_child(dynamic_entities_root)

func _spawn_local_avatar() -> void:
	self_rig = AvatarRig.new()
	self_rig.name = "SelfAvatar"
	add_child(self_rig)

func _add_platform(zone: int, parent: Node3D, spec: Dictionary) -> Dictionary:
	var pos := Vector3(float(spec.get("x", 0.0)), float(spec.get("y", 0.0)), float(spec.get("z", 0.0)))
	var sz := Vector3(float(spec.get("w", 2.0)), float(spec.get("h", 0.5)), float(spec.get("d", 2.0)))
	var mi := MeshInstance3D.new()
	var bm := BoxMesh.new()
	bm.size = sz
	mi.mesh = bm
	mi.position = pos

	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color.from_string(str(spec.get("color", "#e2e8f0")), Color.WHITE)
	mat.roughness = float(spec.get("roughness", 0.55))
	mat.metallic = float(spec.get("metallic", 0.1))
	var tex_style := str(spec.get("texture_style", ""))
	if not tex_style.is_empty() and obby_textures.has(tex_style):
		mat.albedo_texture = obby_textures[tex_style]
		mat.uv1_triplanar = true
		mat.uv1_scale = Vector3(0.35, 0.35, 0.35)
	if spec.has("emissive"):
		mat.emission_enabled = true
		mat.emission = Color.from_string(str(spec["emissive"]), mat.albedo_color)
		mat.emission_energy_multiplier = float(spec.get("emissive_energy", 0.45))
	mi.material_override = mat
	parent.add_child(mi)

	var rec := {
		"mesh": mi,
		"mat": mat,
		"pos": pos,
		"size": sz,
		"min_x": pos.x - sz.x * 0.5,
		"max_x": pos.x + sz.x * 0.5,
		"min_z": pos.z - sz.z * 0.5,
		"max_z": pos.z + sz.z * 0.5,
		"top_y": pos.y + sz.y * 0.5,
		"bottom_y": pos.y - sz.y * 0.5,
		"type": str(spec.get("type", "solid")),
		"solid": true,
		"vel_x": 0.0,
		"vel_z": 0.0
	}
	for k in spec.keys():
		if not rec.has(k):
			rec[k] = spec[k]
	(colliders_by_zone[zone] as Array).append(rec)
	return rec

func _create_stargate(parent: Node3D, pos: Vector3, title: String, subtitle: String, primary_hex: String, secondary_hex: String) -> void:
	var grp := Node3D.new()
	grp.position = pos
	parent.add_child(grp)

	var frame_node := clone_env_template("Env_Stargate_Frame")
	if frame_node != null:
		grp.add_child(frame_node)
		var ring_node := clone_env_template("Env_Stargate_InnerRing")
		if ring_node != null:
			grp.add_child(ring_node)
	else:
		for side in [-1.0, 1.0]:
			var pillar := MeshInstance3D.new()
			var pbox := BoxMesh.new()
			pbox.size = Vector3(0.68, 4.1, 0.68)
			pillar.mesh = pbox
			pillar.position = Vector3(side * 1.85, 2.15, 0.0)
			var pmat := StandardMaterial3D.new()
			pmat.albedo_color = Color("#1e1b4b")
			pillar.material_override = pmat
			grp.add_child(pillar)
		var lintel := MeshInstance3D.new()
		var lbox := BoxMesh.new()
		lbox.size = Vector3(4.65, 0.72, 0.88)
		lintel.mesh = lbox
		lintel.position = Vector3(0.0, 4.35, 0.0)
		grp.add_child(lintel)

	var disc := MeshInstance3D.new()
	var quad := QuadMesh.new()
	quad.size = Vector2(3.55, 3.55)
	disc.mesh = quad
	disc.position = Vector3(0.0, 2.28, 0.0)
	if wormhole_shader != null:
		var sm := ShaderMaterial.new()
		sm.shader = wormhole_shader
		sm.set_shader_parameter("primary_color", Color.from_string(primary_hex, Color("#38bdf8")))
		sm.set_shader_parameter("secondary_color", Color.from_string(secondary_hex, Color("#6366f1")))
		disc.material_override = sm
	grp.add_child(disc)

	var lbl := Label3D.new()
	lbl.text = title + "\n" + subtitle
	lbl.position = Vector3(0.0, 5.75, 0.0)
	lbl.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	lbl.font_size = 28
	lbl.outline_size = 8
	lbl.modulate = Color.from_string(primary_hex, Color.WHITE)
	grp.add_child(lbl)

func _build_plaza_zone() -> void:
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 0, "y": -1.0, "z": 0, "w": 210, "h": 2.0, "d": 210, "color": "#4ade80", "roughness": 0.82})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 0, "y": 0.04, "z": 0, "w": 18, "h": 0.08, "d": 18, "color": "#e2e8f0"})

	# Disco Dance Floor & Cozy Campfire
	var disco_cols := ["#f472b6", "#38bdf8", "#fbbf24", "#a78bfa", "#34d399"]
	for gx in range(4):
		for gz in range(4):
			var c: String = disco_cols[(gx + gz) % disco_cols.size()]
			_add_platform(ZONE_PLAZA, plaza_root, {"x": -13 + gx * 2, "y": 0.08, "z": -2 + gz * 2, "w": 1.9, "h": 0.14, "d": 1.9, "color": c, "emissive": c, "emissive_energy": 0.35})

	var pit_glb := clone_env_template("Env_Campfire_Pit")
	if pit_glb != null:
		pit_glb.position = Vector3(-6.0, 0.0, 11.0)
		plaza_root.add_child(pit_glb)
	var tent_glb := clone_env_template("Env_Camp_Tent")
	if tent_glb != null:
		tent_glb.position = Vector3(-11.5, 0.0, 15.5)
		plaza_root.add_child(tent_glb)

	_add_platform(ZONE_PLAZA, plaza_root, {"x": -6.0, "y": 0.35, "z": 14.2, "w": 4.2, "h": 0.7, "d": 1.1, "color": "#92400e"})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": -9.2, "y": 0.35, "z": 11.0, "w": 1.1, "h": 0.7, "d": 4.2, "color": "#92400e"})
	var fire_light := OmniLight3D.new()
	fire_light.position = Vector3(-6.0, 1.85, 11.0)
	fire_light.light_color = Color("#fb923c")
	fire_light.light_energy = 2.0
	fire_light.omni_range = 15.0
	plaza_root.add_child(fire_light)

	# DTI Fashion Runway (`x = -24`) + Shared Couture Stage GLBs
	_add_platform(ZONE_PLAZA, plaza_root, {"x": -18.2, "y": 0.14, "z": 1.0, "w": 2.0, "h": 0.28, "d": 6.0, "color": "#fbcfe8"})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": -24.0, "y": 0.25, "z": -9.5, "w": 10.5, "h": 0.5, "d": 6.0, "color": "#fdf2f8"})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": -24.0, "y": 0.25, "z": 1.5, "w": 5.4, "h": 0.5, "d": 16.0, "color": "#fff1f2", "emissive": "#f472b6", "emissive_energy": 0.18})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": -24.0, "y": 0.28, "z": 11.2, "w": 7.4, "h": 0.56, "d": 5.6, "color": "#fce7f3", "emissive": "#ec4899", "emissive_energy": 0.28})
	var dti_arch := clone_env_template("Env_DTI_Arch")
	if dti_arch != null:
		dti_arch.position = Vector3(-24.0, 0.5, -6.5)
		plaza_root.add_child(dti_arch)

	# Starlight Crafting Bench (`(12.5, 0, 6.5)`)
	var bench_glb := clone_env_template("Env_Crafting_Bench")
	if bench_glb != null:
		bench_glb.position = CRAFTING_BENCH_POS
		plaza_root.add_child(bench_glb)
	else:
		_add_platform(ZONE_PLAZA, plaza_root, {"x": CRAFTING_BENCH_POS.x, "y": 0.55, "z": CRAFTING_BENCH_POS.z, "w": 2.2, "h": 1.1, "d": 1.4, "color": "#7c3aed", "emissive": "#38bdf8", "emissive_energy": 0.35})
	var craft_lbl := Label3D.new()
	craft_lbl.text = "🔨 Starlight Crafting Workshop\nOpen Homes & Craft Menu!"
	craft_lbl.position = CRAFTING_BENCH_POS + Vector3(0, 2.35, 0)
	craft_lbl.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	craft_lbl.font_size = 24
	craft_lbl.outline_size = 8
	plaza_root.add_child(craft_lbl)

	# Stargate Portals (Crystal Cavern & Shadow Rift Arena)
	_create_stargate(plaza_root, PLAZA_PORTAL_POS, "🌀 STARGATE — CRYSTAL CAVERN", "Step In or Tap ⚡ • Mine Gems & Coins!", "#38bdf8", "#6366f1")
	_create_stargate(plaza_root, PLAZA_ARENA_PORTAL_POS, "⚔️ SHADOW RIFT — MORTAL ARENA", "Step In or Tap ⚡ • Battle Monsters & Dragon Boss!", "#ef4444", "#f97316")

	# 6 Claimable Home Plots & 3D Mailboxes
	for pdef in HOME_PLOTS_STATIC:
		var cx := float(pdef["cx"])
		var cz := float(pdef["cz"])
		var span := float(int(pdef["half"]) * 2)
		_add_platform(ZONE_PLAZA, plaza_root, {"x": cx, "y": 0.03, "z": cz, "w": span, "h": 0.06, "d": span, "color": "#bbf7d0"})
		var mpos: Vector3 = pdef["mbox"]
		var mb_glb := clone_env_template("Env_Plot_Mailbox")
		if mb_glb != null:
			mb_glb.position = mpos
			plaza_root.add_child(mb_glb)
		else:
			_add_platform(ZONE_PLAZA, plaza_root, {"x": mpos.x, "y": 0.6, "z": mpos.z, "w": 0.65, "h": 1.2, "d": 0.65, "color": "#f472b6"})
		var mlbl := Label3D.new()
		mlbl.position = mpos + Vector3(0, 1.85, 0)
		mlbl.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		mlbl.font_size = 24
		mlbl.outline_size = 8
		mlbl.text = str(pdef["name"]) + "\nAvailable — Tap Homes!"
		plaza_root.add_child(mlbl)
		mailbox_labels[str(pdef["id"])] = mlbl

	# Jump Pads
	for jp in [
		{"x": 7.5, "y": 0.2, "z": -7.5, "bounce": 20.0},
		{"x": -7.5, "y": 0.2, "z": -7.5, "bounce": 20.0},
		{"x": 0.0, "y": 8.6, "z": -34.0, "bounce": 23.0}
	]:
		var pad := _add_platform(ZONE_PLAZA, plaza_root, {"x": jp["x"], "y": jp["y"], "z": jp["z"], "w": 2.6, "h": 0.4, "d": 2.6, "color": "#ec4899", "emissive": "#db2777", "type": "jumppad", "bounce_vel": jp["bounce"]})
		jump_pads.append(pad)

	# Course 1: Sky Obby (`easy`)
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 0.0, "y": 0.4, "z": -12.0, "w": 4.5, "h": 0.8, "d": 3.5, "color": "#fbbf24", "type": "checkpoint"})
	var easy_gate := clone_env_template("Env_Obby_Gate")
	if easy_gate != null:
		easy_gate.position = Vector3(0.0, 0.8, -12.0)
		plaza_root.add_child(easy_gate)

	for s in [
		{"x": 0.0, "y": 1.4, "z": -16.5, "c": "#ef4444", "tex": "red_brick"},
		{"x": -3.2, "y": 2.5, "z": -20.0, "c": "#38bdf8", "tex": "hex_scifi"},
		{"x": 0.0, "y": 3.6, "z": -23.5, "c": "#34d399", "tex": "marble_tiles"},
		{"x": 3.4, "y": 4.8, "z": -27.0, "c": "#a78bfa", "tex": "hex_scifi"},
		{"x": 0.0, "y": 6.0, "z": -30.5, "c": "#ef4444", "tex": "red_brick"}
	]:
		_add_platform(ZONE_PLAZA, plaza_root, {"x": s["x"], "y": s["y"], "z": s["z"], "w": 3.1, "h": 0.6, "d": 3.1, "color": s["c"], "texture_style": s["tex"]})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 0.0, "y": 8.2, "z": -34.0, "w": 5.5, "h": 0.6, "d": 4.2, "color": "#fbbf24", "type": "checkpoint"})

	var m1 := _add_platform(ZONE_PLAZA, plaza_root, {"x": -6.5, "y": 11.5, "z": -34.0, "w": 3.6, "h": 0.6, "d": 3.6, "color": "#38bdf8", "type": "moving", "base_x": -6.5, "base_y": 11.5, "base_z": -34.0, "amp_x": 3.5, "amp_z": 0.0, "speed": 1.6})
	var m2 := _add_platform(ZONE_PLAZA, plaza_root, {"x": -11.0, "y": 13.5, "z": -26.0, "w": 3.6, "h": 0.6, "d": 3.6, "color": "#c084fc", "type": "moving", "base_x": -11.0, "base_y": 13.5, "base_z": -26.0, "amp_x": 0.0, "amp_z": 4.0, "speed": 1.8})
	moving_platforms.append(m1)
	moving_platforms.append(m2)

	var rcols := ["#ef4444", "#f97316", "#eab308", "#22c55e", "#38bdf8", "#8b5cf6"]
	for i in range(rcols.size()):
		_add_platform(ZONE_PLAZA, plaza_root, {"x": -11.0 + float(i + 1) * 2.2, "y": 14.8 + float(i) * 0.25, "z": -20.0, "w": 2.1, "h": 0.45, "d": 3.0, "color": rcols[i]})
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 6.5, "y": 16.5, "z": -20.0, "w": 8.0, "h": 1.0, "d": 8.0, "color": "#fef08a", "emissive": "#fbbf24", "type": "checkpoint"})

	var easy_board_frame := clone_env_template("Env_Leaderboard_Board")
	if easy_board_frame != null:
		easy_board_frame.position = Vector3(4.5, 0.0, -11.5)
		plaza_root.add_child(easy_board_frame)
	var easy_board := Label3D.new()
	easy_board.position = Vector3(4.5, 2.5, -11.5)
	easy_board.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	easy_board.font_size = 24
	easy_board.outline_size = 8
	easy_board.text = "🌈 SKY OBBY TOP 5\nStep on Gold Pad to Start!"
	plaza_root.add_child(easy_board)
	leaderboard_labels["easy"] = easy_board

	# Course 2: Pro Obby Gauntlet (`hard`)
	_build_pro_obby_gauntlet()

func _build_pro_obby_gauntlet() -> void:
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 52.0, "y": 0.4, "z": 52.0, "w": 5.0, "h": 0.8, "d": 5.0, "color": "#f59e0b", "emissive": "#d97706", "type": "checkpoint"})
	var hard_gate := clone_env_template("Env_Obby_Gate")
	if hard_gate != null:
		hard_gate.position = Vector3(52.0, 0.8, 52.0)
		plaza_root.add_child(hard_gate)

	for s in [
		{"x": 56.2, "y": 2.0, "z": 52.0, "w": 2.0, "d": 2.0, "c": "#ef4444", "tex": "red_brick"},
		{"x": 59.8, "y": 3.5, "z": 49.2, "w": 1.85, "d": 1.85, "c": "#a855f7", "tex": "hex_scifi"},
		{"x": 63.6, "y": 5.0, "z": 52.4, "w": 1.85, "d": 1.85, "c": "#38bdf8", "tex": "metal_plate"}
	]:
		_add_platform(ZONE_PLAZA, plaza_root, {"x": s["x"], "y": s["y"], "z": s["z"], "w": s["w"], "h": 0.5, "d": s["d"], "color": s["c"], "texture_style": s["tex"]})

	moving_platforms.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": 67.8, "y": 6.6, "z": 52.4, "w": 2.4, "h": 0.5, "d": 2.4, "color": "#22d3ee", "type": "moving", "base_x": 67.8, "base_y": 6.6, "base_z": 52.4, "amp_x": 0.0, "amp_z": 3.4, "speed": 2.2}))
	moving_platforms.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": 71.2, "y": 8.2, "z": 52.4, "w": 2.4, "h": 0.5, "d": 2.4, "color": "#c084fc", "type": "moving", "base_x": 71.2, "base_y": 8.2, "base_z": 52.4, "amp_x": 2.2, "amp_z": 0.0, "speed": 2.4}))
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 76.0, "y": 9.6, "z": 52.4, "w": 4.8, "h": 0.6, "d": 4.8, "color": "#f59e0b", "type": "checkpoint"})

	for bd in [
		{"x": 79.0, "y": 11.1, "z": 56.2, "phase": 0, "c": "#f472b6"},
		{"x": 82.0, "y": 12.6, "z": 60.0, "phase": 1200, "c": "#fb7185"},
		{"x": 85.0, "y": 14.1, "z": 63.8, "phase": 2400, "c": "#f472b6"},
		{"x": 85.0, "y": 15.6, "z": 68.4, "phase": 600, "c": "#fb7185"},
		{"x": 84.0, "y": 17.0, "z": 72.8, "phase": 1800, "c": "#f472b6"}
	]:
		var bp := _add_platform(ZONE_PLAZA, plaza_root, {"x": bd["x"], "y": bd["y"], "z": bd["z"], "w": 2.2, "h": 0.5, "d": 2.2, "color": bd["c"], "emissive": bd["c"], "type": "blinking", "cycle_ms": 5100, "solid_ms": 2600, "warn_ms": 900, "phase_ms": bd["phase"]})
		blinking_platforms.append(bp)

	_add_platform(ZONE_PLAZA, plaza_root, {"x": 83.0, "y": 18.0, "z": 77.6, "w": 4.8, "h": 0.6, "d": 4.8, "color": "#f59e0b", "type": "checkpoint"})
	jump_pads.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": 80.6, "y": 18.5, "z": 78.6, "w": 2.2, "h": 0.4, "d": 2.2, "color": "#ec4899", "emissive": "#db2777", "type": "jumppad", "bounce_vel": 21.5}))
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 74.8, "y": 22.0, "z": 81.2, "w": 2.8, "h": 0.5, "d": 2.8, "color": "#38bdf8"})
	jump_pads.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": 74.8, "y": 22.45, "z": 81.2, "w": 2.2, "h": 0.4, "d": 2.2, "color": "#ec4899", "emissive": "#db2777", "type": "jumppad", "bounce_vel": 22.0}))
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 68.6, "y": 25.8, "z": 84.4, "w": 2.6, "h": 0.5, "d": 2.6, "color": "#a78bfa"})
	jump_pads.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": 68.6, "y": 26.25, "z": 84.4, "w": 2.2, "h": 0.4, "d": 2.2, "color": "#ec4899", "emissive": "#db2777", "type": "jumppad", "bounce_vel": 22.0}))
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 61.5, "y": 29.0, "z": 85.5, "w": 5.0, "h": 0.6, "d": 5.0, "color": "#f59e0b", "type": "checkpoint"})

	for sd in [
		{"x": 59.2, "y": 30.2, "z": 79.4, "sz": 5.4, "r": 2.75, "spd": 1.75},
		{"x": 56.6, "y": 31.6, "z": 73.0, "sz": 5.2, "r": 2.65, "spd": -2.0},
		{"x": 54.6, "y": 33.0, "z": 66.8, "sz": 5.0, "r": 2.55, "spd": 2.2}
	]:
		_add_platform(ZONE_PLAZA, plaza_root, {"x": sd["x"], "y": sd["y"], "z": sd["z"], "w": sd["sz"], "h": 0.6, "d": sd["sz"], "color": "#1e1b4b", "emissive": "#4f46e5"})
		var sw := Node3D.new()
		sw.position = Vector3(float(sd["x"]), float(sd["y"]) + 0.82, float(sd["z"]))
		var bar := MeshInstance3D.new()
		var bar_mesh := BoxMesh.new()
		bar_mesh.size = Vector3(float(sd["r"]) * 2.0, 0.26, 0.26)
		bar.mesh = bar_mesh
		var bmat := StandardMaterial3D.new()
		bmat.albedo_color = Color("#fb7185")
		bmat.emission_enabled = true
		bmat.emission = Color("#f43f5e")
		bmat.emission_energy_multiplier = 0.85
		bar.material_override = bmat
		sw.add_child(bar)
		plaza_root.add_child(sw)
		sweeper_hazards.append({"node": sw, "pos": sw.position, "radius": float(sd["r"]), "speed": float(sd["spd"]), "angle": 0.0})

	_add_platform(ZONE_PLAZA, plaza_root, {"x": 55.5, "y": 34.4, "z": 60.4, "w": 4.8, "h": 0.6, "d": 4.8, "color": "#f59e0b", "type": "checkpoint"})
	for bd2 in [
		{"x": 59.5, "y": 35.8, "z": 62.6, "phase": 0, "c": "#38bdf8"},
		{"x": 63.6, "y": 37.3, "z": 64.6, "phase": 1700, "c": "#a855f7"}
	]:
		blinking_platforms.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": bd2["x"], "y": bd2["y"], "z": bd2["z"], "w": 2.0, "h": 0.5, "d": 2.0, "color": bd2["c"], "emissive": bd2["c"], "type": "blinking", "cycle_ms": 4800, "solid_ms": 2400, "warn_ms": 850, "phase_ms": bd2["phase"]}))

	moving_platforms.append(_add_platform(ZONE_PLAZA, plaza_root, {"x": 67.5, "y": 38.8, "z": 66.6, "w": 2.5, "h": 0.5, "d": 2.5, "color": "#f472b6", "type": "moving", "base_x": 67.5, "base_y": 38.8, "base_z": 66.6, "amp_x": 1.8, "amp_z": 1.8, "speed": 2.2}))
	_add_platform(ZONE_PLAZA, plaza_root, {"x": 72.6, "y": 40.2, "z": 69.2, "w": 7.6, "h": 1.0, "d": 7.6, "color": "#e0f2fe", "emissive": "#38bdf8", "type": "checkpoint"})

	var hard_board_frame := clone_env_template("Env_Leaderboard_Board")
	if hard_board_frame != null:
		hard_board_frame.position = Vector3(48.0, 0.0, 52.0)
		plaza_root.add_child(hard_board_frame)
	var hard_board := Label3D.new()
	hard_board.position = Vector3(48.0, 2.6, 52.0)
	hard_board.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	hard_board.font_size = 24
	hard_board.outline_size = 8
	hard_board.text = "🔥 PRO GAUNTLET TOP 5\nStep on Gold Pad to Start!"
	plaza_root.add_child(hard_board)
	leaderboard_labels["hard"] = hard_board

func _build_dungeon_zone() -> void:
	_add_platform(ZONE_DUNGEON, dungeon_root, {"x": 0, "y": -1.0, "z": 0, "w": 84, "h": 2.0, "d": 84, "color": "#14102b"})
	for ledge in [
		[-16.0, 0.9, -9.5, 5.4, 4.4, "#312e81"],
		[-16.0, 1.8, -16.0, 7.2, 7.2, "#3b0764"],
		[16.0, 0.9, -9.5, 5.4, 4.4, "#312e81"],
		[16.0, 1.8, -16.0, 7.2, 7.2, "#3b0764"],
		[-17.0, 2.5, -21.2, 4.8, 4.0, "#4c1d95"],
		[-18.0, 3.2, -26.0, 7.6, 7.2, "#581c87"],
		[0.0, 1.1, -12.5, 5.8, 4.2, "#312e81"],
		[0.0, 2.2, -18.0, 7.4, 6.4, "#3b0764"],
		[0.0, 3.4, -23.5, 5.6, 4.4, "#4c1d95"],
		[0.0, 4.5, -30.0, 9.2, 8.2, "#581c87"]
	]:
		var top_y := float(ledge[1])
		_add_platform(ZONE_DUNGEON, dungeon_root, {"x": ledge[0], "y": top_y * 0.5, "z": ledge[2], "w": ledge[3], "h": top_y, "d": ledge[4], "color": ledge[5], "emissive": "#7e22ce", "emissive_energy": 0.22})
	_create_stargate(dungeon_root, DUNGEON_RETURN_POS, "🌟 STARLIGHT PLAZA", "Step In or Tap ⚡ • Return to Plaza", "#38bdf8", "#f472b6")

func _build_prison_zone() -> void:
	_add_platform(ZONE_PRISON, prison_root, {"x": 0, "y": -1.0, "z": 0, "w": 34, "h": 2.0, "d": 34, "color": "#334155"})
	_create_stargate(prison_root, PRISON_RELEASE_POS, "⚖️ RELEASE GATE", "Complete Chores & Step Through!", "#fbbf24", "#38bdf8")

func _build_arena_zone() -> void:
	_add_platform(ZONE_ARENA, arena_root, {"x": 0, "y": -1.65, "z": 0, "w": 120, "h": 2.0, "d": 120, "color": "#450a0a"})
	var lava := MeshInstance3D.new()
	var lplane := BoxMesh.new()
	lplane.size = Vector3(116.0, 0.08, 116.0)
	lava.mesh = lplane
	lava.position = Vector3(0.0, -0.45, 0.0)
	var lmat := StandardMaterial3D.new()
	lmat.albedo_color = Color("#ff4500")
	lmat.emission_enabled = true
	lmat.emission = Color("#ea580c")
	lmat.emission_energy_multiplier = 1.3
	lava.material_override = lmat
	arena_root.add_child(lava)

	_add_platform(ZONE_ARENA, arena_root, {"x": 0, "y": -0.5, "z": 15.0, "w": 60, "h": 1.0, "d": 36, "color": "#1c1917"})
	_add_platform(ZONE_ARENA, arena_root, {"x": 0, "y": -0.5, "z": -20.0, "w": 60, "h": 1.0, "d": 26, "color": "#18181b"})
	for bx in [-16.0, 0.0, 16.0]:
		_add_platform(ZONE_ARENA, arena_root, {"x": bx, "y": 0.08, "z": -5.0, "w": 5.2, "h": 0.36, "d": 5.2, "color": "#292524", "emissive": "#9a3412"})
	_create_stargate(arena_root, ARENA_RETURN_POS, "🌟 STARLIGHT PLAZA", "Safe Sanctuary Return Gate", "#34d399", "#38bdf8")

	# North Island Obsidian Rift Dragon Summoning Altar
	var altar_glb := clone_env_template("Env_Boss_Altar")
	if altar_glb != null:
		altar_glb.position = BOSS_ALTAR_POS
		arena_root.add_child(altar_glb)
	else:
		_add_platform(ZONE_ARENA, arena_root, {"x": BOSS_ALTAR_POS.x, "y": 0.35, "z": BOSS_ALTAR_POS.z, "w": 3.6, "h": 0.7, "d": 3.6, "color": "#0f172a", "emissive": "#9333ea"})
	var altar_lbl := Label3D.new()
	altar_lbl.position = BOSS_ALTAR_POS + Vector3(0, 2.8, 0)
	altar_lbl.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	altar_lbl.font_size = 26
	altar_lbl.outline_size = 8
	altar_lbl.modulate = Color("#c084fc")
	altar_lbl.text = "🐉 OBSIDIAN DRAGON ALTAR\nTap ⚡ Action to Summon World Boss!"
	arena_root.add_child(altar_lbl)

func queue_jump() -> void:
	jump_queued = true

func trigger_emote(emote_name: String, duration_sec: float = 3.2) -> void:
	active_emote = emote_name
	emote_timer = duration_sec
	if self_rig != null:
		self_rig.set_motion_and_emote(_compute_current_anim(), active_emote)

func toggle_equipped_hoverboard() -> void:
	if net == null:
		return
	net.send_game_command({"type": "companions", "action": "equip_mount", "mountId": "toggle"})

func find_nearest_interactive_plaza_target() -> Dictionary:
	# 1. Nearby neighborhood Mailbox (`ring_doorbell`)
	for p in home_plots_state:
		if not (p is Dictionary):
			continue
		var mbox: Array = p.get("mailboxPos", []) if p.get("mailboxPos") is Array else []
		if mbox.size() >= 3:
			var mpos := Vector3(float(mbox[0]), float(mbox[1]), float(mbox[2]))
			if player_pos.distance_to(mpos) < 2.8:
				var owner_u := str(p.get("ownerUsername", "")) if p.get("ownerUsername") != null else ""
				if not owner_u.is_empty() and (net == null or owner_u.to_lower() != net.username.to_lower()):
					return {
						"action": "doorbell",
						"plotId": str(p.get("plotId", "")),
						"label": "🔔 Tap ⚡ Action to Ring %s's Doorbell" % str(p.get("ownerDisplayName", owner_u))
					}

	# 2. Nearby interactive Voxel Prop (doors, windows, lamps, chairs, sofas, beds) within 2.8m
	var best_d := 2.85
	var best_hit: Dictionary = {}
	var px := int(round(player_pos.x))
	var py := int(floor(player_pos.y + 0.15)) + 1
	var pz := int(round(player_pos.z))
	for dx in range(-3, 4):
		for dy in range(-1, 3):
			for dz in range(-3, 4):
				var key := "%d,%d,%d" % [px + dx, py + dy, pz + dz]
				if not saved_blocks.has(key):
					continue
				var b: Dictionary = saved_blocks[key]
				var kind := str(b.get("kind", "block"))
				var bx := int(b.get("x", 0))
				var by := int(b.get("y", 1))
				var bz := int(b.get("z", 0))
				var bpos := Vector3(float(bx), float(by - 1) + 0.5, float(bz))
				var d := player_pos.distance_to(bpos)
				if d >= best_d:
					continue
				if kind == "door" or kind == "double_door" or kind == "window" or kind == "crystal_lamp" or kind == "nightstand_lamp":
					best_d = d
					best_hit = {
						"action": "toggle_block",
						"x": bx,
						"y": by,
						"z": bz,
						"kind": kind,
						"label": "🚪 Tap ⚡ Action to Toggle %s" % kind.replace("_", " ").capitalize()
					}
				elif kind == "chair" or kind == "sofa":
					best_d = d
					best_hit = {
						"action": "sit_furniture",
						"x": bx,
						"y": by,
						"z": bz,
						"rot": int(b.get("rot", 0)),
						"label": "🪑 Tap ⚡ Action to Sit on %s" % kind.capitalize()
					}
				elif kind == "bed":
					best_d = d
					best_hit = {
						"action": "sleep_bed",
						"x": bx,
						"y": by,
						"z": bz,
						"rot": int(b.get("rot", 0)),
						"label": "🛏️ Tap ⚡ Action to Rest on Bed"
					}
	return best_hit

func trigger_context_action() -> void:
	if active_zone == ZONE_ARENA and arena_dead:
		if net != null:
			net.send_game_command({"type": "arena_respawn"})
		return

	if active_zone == ZONE_PLAZA:
		if net != null and player_pos.distance_to(PLAZA_PORTAL_POS) < 3.6:
			net.send_game_command({"type": "zone_portal", "zone": ZONE_DUNGEON})
			return
		if net != null and player_pos.distance_to(PLAZA_ARENA_PORTAL_POS) < 3.6:
			net.send_game_command({"type": "zone_portal", "zone": ZONE_ARENA})
			return
		if build_mode:
			_place_or_remove_block_in_front()
			return
		for star in sky_falling_stars:
			if star is Dictionary and net != null:
				var sp := Vector3(float(star.get("x", 0.0)), float(star.get("y", 0.0)), float(star.get("z", 0.0)))
				if player_pos.distance_to(sp) < 5.5:
					net.send_game_command({"type": "sky", "action": "catch_star", "starId": str(star.get("id", ""))})
					return

		var interact := find_nearest_interactive_plaza_target()
		if not interact.is_empty():
			var act := str(interact.get("action", ""))
			if act == "doorbell" and net != null:
				net.send_game_command({"type": "homes", "action": "ring_doorbell", "plotId": str(interact["plotId"])})
				return
			elif act == "toggle_block" and net != null:
				net.send_game_command({
					"type": "block",
					"action": "toggle",
					"x": int(interact["x"]),
					"y": int(interact["y"]),
					"z": int(interact["z"])
				})
				return
			elif act == "sit_furniture":
				player_pos = Vector3(float(interact["x"]), float(int(interact["y"]) - 1) + 0.45, float(interact["z"]))
				player_vel = Vector3.ZERO
				player_yaw = float(int(interact.get("rot", 0))) * (PI * 0.5)
				trigger_emote("sit", 30.0)
				return
			elif act == "sleep_bed":
				player_pos = Vector3(float(interact["x"]), float(int(interact["y"]) - 1) + 0.55, float(interact["z"]))
				player_vel = Vector3.ZERO
				player_yaw = float(int(interact.get("rot", 0))) * (PI * 0.5)
				trigger_emote("sleep", 30.0)
				return

		if net != null:
			net.send_game_command({
				"type": "raygun_zap",
				"x": player_pos.x - sin(player_yaw) * 8.0,
				"y": player_pos.y + 1.1,
				"z": player_pos.z - cos(player_yaw) * 8.0
			})
	elif active_zone == ZONE_DUNGEON and net != null:
		if player_pos.distance_to(DUNGEON_RETURN_POS) < 3.8:
			net.send_game_command({"type": "zone_portal", "zone": ZONE_PLAZA})
			return
		var best_mob := _nearest_dict_target(dungeon_mobs, 6.5)
		if not best_mob.is_empty():
			net.send_game_command({"type": "dungeon_attack", "mobId": best_mob, "mode": "blast"})
			return
		var best_node := _nearest_dict_target(dungeon_nodes, 5.2)
		if not best_node.is_empty():
			net.send_game_command({"type": "dungeon_interact", "nodeId": best_node})
			return
	elif active_zone == ZONE_ARENA and net != null:
		if player_pos.distance_to(ARENA_RETURN_POS) < 3.8:
			net.send_game_command({"type": "zone_portal", "zone": ZONE_PLAZA})
			return
		if bool(arena_boss.get("active", false)):
			var bp := Vector3(float(arena_boss.get("x", 0.0)), float(arena_boss.get("y", 0.0)), float(arena_boss.get("z", -24.5)))
			if player_pos.distance_to(bp) < 18.0:
				# Authoritative boss ID expected by `WorldEngine::dungeon_attack_mob` (`src/world.rs:4667`)
				net.send_game_command({"type": "dungeon_attack", "mobId": "obsidian_dragon", "mode": "blast"})
				return
		elif player_pos.distance_to(BOSS_ALTAR_POS) < 5.2:
			var has_beacon := int(inventory_items.get("rift_beacon", 0)) > 0
			net.request_json("/api/ella-world/arena/boss", HTTPClient.METHOD_POST, {"token": net.auth_token, "useBeacon": has_beacon}, net.auth_token, func(_s: int, d: Dictionary) -> void:
				if d.has("error"):
					toast_requested.emit(str(d["error"]))
				else:
					if d.get("boss") is Dictionary:
						_sync_boss_state(d["boss"])
					toast_requested.emit("🐉 Obsidian Rift Dragon summoned!")
			)
			return
		var arena_mob := _nearest_dict_target(arena_mobs, 8.0)
		if not arena_mob.is_empty():
			net.send_game_command({"type": "dungeon_attack", "mobId": arena_mob, "mode": "blast"})
	elif active_zone == ZONE_PRISON and net != null:
		if prison_tasks_remaining <= 0 and player_pos.distance_to(PRISON_RELEASE_POS) < 4.0:
			net.send_game_command({"type": "zone_portal", "zone": ZONE_PLAZA})
			return
		var pnode := _nearest_dict_target(prison_nodes, 5.0)
		if not pnode.is_empty():
			net.send_game_command({"type": "prison_work", "nodeId": pnode})

func _nearest_dict_target(dict_map: Dictionary, max_dist: float) -> String:
	var best_id := ""
	var best_d := max_dist
	for k in dict_map.keys():
		var item: Dictionary = dict_map[k]
		if item.has("alive") and not bool(item["alive"]):
			continue
		var avail := bool(item.get("available", item.get("ready", true)))
		if not avail:
			continue
		var p := Vector3(float(item.get("x", 0.0)), float(item.get("y", 0.0)), float(item.get("z", 0.0)))
		var d := player_pos.distance_to(p)
		if d < best_d:
			best_d = d
			best_id = str(k)
	return best_id

func compute_front_build_cell() -> Vector3i:
	var fwd := Vector3(-sin(player_yaw), 0.0, -cos(player_yaw)).normalized()
	var target := player_pos + fwd * 2.2
	var gx := clampi(int(round(target.x)), -100, 100)
	var gy := clampi(int(floor(player_pos.y + 0.15)) + 1, 1, 80)
	var gz := clampi(int(round(target.z)), -100, 100)
	return Vector3i(gx, gy, gz)

func _place_or_remove_block_in_front() -> void:
	if net == null:
		return
	var cell := compute_front_build_cell()
	net.send_game_command({
		"type": "block",
		"action": build_tool,
		"x": cell.x,
		"y": cell.y,
		"z": cell.z,
		"color": selected_block_color,
		"kind": selected_block_kind,
		"rot": selected_block_rot
	})

func _on_world_tap(_screen_pos: Vector2) -> void:
	if build_mode and active_zone == ZONE_PLAZA:
		_place_or_remove_block_in_front()

func _process(delta: float) -> void:
	if net == null or net.auth_token.is_empty():
		return
	var now_sec := Time.get_ticks_msec() * 0.001
	var now_ms := Time.get_ticks_msec()

	_step_dynamic_platforms(delta, now_sec, now_ms)
	_step_local_player_physics(delta, now_ms)
	_step_camera()
	_step_remote_avatars(delta)
	_step_speedrun_gates_and_portals(now_ms)
	_update_context_hint()

	if Input.is_action_just_pressed("ui_accept") and not touch.modal_blocking:
		jump_queued = true

	state_send_accum += delta
	if state_send_accum >= 0.05:
		state_send_accum = 0.0
		var anim := _compute_current_anim()
		net.send_player_state(player_pos, player_yaw, anim, active_emote, active_zone)

func _step_dynamic_platforms(delta: float, now_sec: float, now_ms: int) -> void:
	for mp in moving_platforms:
		var old_x := float(mp["pos"].x)
		var old_z := float(mp["pos"].z)
		var spd := float(mp.get("speed", 1.6))
		var nx := float(mp["base_x"]) + sin(now_sec * spd) * float(mp.get("amp_x", 0.0))
		var nz := float(mp["base_z"]) + cos(now_sec * spd) * float(mp.get("amp_z", 0.0))
		var sz: Vector3 = mp["size"]
		mp["pos"] = Vector3(nx, float(mp["base_y"]), nz)
		(mp["mesh"] as Node3D).position = mp["pos"]
		mp["min_x"] = nx - sz.x * 0.5
		mp["max_x"] = nx + sz.x * 0.5
		mp["min_z"] = nz - sz.z * 0.5
		mp["max_z"] = nz + sz.z * 0.5
		mp["vel_x"] = (nx - old_x) / maxf(delta, 0.001)
		mp["vel_z"] = (nz - old_z) / maxf(delta, 0.001)

	for bp in blinking_platforms:
		var cycle := int(bp.get("cycle_ms", 5100))
		var solid_ms := int(bp.get("solid_ms", 2600))
		var phase := (now_ms + int(bp.get("phase_ms", 0))) % cycle
		var is_solid := phase < solid_ms
		bp["solid"] = is_solid
		var mi := bp["mesh"] as MeshInstance3D
		mi.visible = is_solid or (phase < solid_ms + 300 and (phase / 90) % 2 == 0)

	for sw in sweeper_hazards:
		sw["angle"] = float(sw["angle"]) + float(sw["speed"]) * delta
		var sw_node := sw["node"] as Node3D
		sw_node.rotation.y = float(sw["angle"])
		if active_zone == ZONE_PLAZA:
			var cpos: Vector3 = sw["pos"]
			if absf(player_pos.y - (cpos.y - 0.5)) < 0.9 and Vector2(player_pos.x - cpos.x, player_pos.z - cpos.z).length() < float(sw["radius"]):
				var bar_dir := Vector2(cos(float(sw["angle"])), -sin(float(sw["angle"])))
				var rel := Vector2(player_pos.x - cpos.x, player_pos.z - cpos.z)
				var perp := absf(rel.x * bar_dir.y - rel.y * bar_dir.x)
				if perp < 0.45:
					player_vel.x += bar_dir.y * 12.0
					player_vel.z -= bar_dir.x * 12.0
					player_vel.y = maxf(player_vel.y, 6.5)

static func _is_voxel_passable(b: Dictionary) -> bool:
	var kind := str(b.get("kind", "block"))
	var is_open := bool(b.get("open", false))
	if kind == "rug" or kind == "crystal_lamp" or kind == "nightstand_lamp":
		return true
	if (kind == "door" or kind == "double_door") and is_open:
		return true
	return false

static func _voxel_support_top_y(b: Dictionary) -> float:
	var by := float(int(b.get("y", 1)))
	var kind := str(b.get("kind", "block"))
	if kind == "slab":
		return by - 0.5
	if kind == "chair" or kind == "sofa":
		return by - 0.45
	if kind == "bed":
		return by - 0.4
	return by

func _step_local_player_physics(delta: float, _now_ms: int) -> void:
	if emote_timer > 0.0:
		emote_timer -= delta
		if emote_timer <= 0.0:
			active_emote = ""

	var move_in := touch.get_combined_move_vector() if touch != null else Vector2.ZERO
	if move_in.length_squared() > 0.01 or jump_queued:
		active_emote = ""

	var has_mount := not equipped_mount.is_empty()
	var top_speed := 11.4 if has_mount else 7.6
	var cam_yaw := touch.camera_yaw if touch != null else 0.0
	var fwd := Vector3(-sin(cam_yaw), 0.0, -cos(cam_yaw))
	var right := Vector3(cos(cam_yaw), 0.0, -sin(cam_yaw))
	var wish := (right * move_in.x + fwd * (-move_in.y))
	if wish.length() > 1.0:
		wish = wish.normalized()

	player_vel.x = lerpf(player_vel.x, wish.x * top_speed, minf(1.0, delta * 14.0))
	player_vel.z = lerpf(player_vel.z, wish.z * top_speed, minf(1.0, delta * 14.0))
	if wish.length_squared() > 0.01:
		player_yaw = atan2(-wish.x, -wish.z)

	if jump_queued:
		jump_queued = false
		if is_grounded:
			player_vel.y = 12.6 if has_mount else 11.4
			is_grounded = false

	player_vel.y = maxf(player_vel.y - 26.0 * delta, -42.0)

	var prev_y := player_pos.y
	var next_x := player_pos.x + player_vel.x * delta
	var next_z := player_pos.z + player_vel.z * delta

	# Horizontal XZ wall & closed-door pushout in Starlight Plaza (`y = 1..80`, bottom = `y - 1`)
	if active_zone == ZONE_PLAZA and active_emote != "sit" and active_emote != "sleep":
		var foot_cell_y := int(floor(player_pos.y + 0.25)) + 1
		for check_y in [foot_cell_y, foot_cell_y + 1]:
			var cx := int(round(next_x))
			var cz := int(round(next_z))
			for ox in range(-1, 2):
				for oz in range(-1, 2):
					var wkey := "%d,%d,%d" % [cx + ox, check_y, cz + oz]
					if not saved_blocks.has(wkey):
						continue
					var wb: Dictionary = saved_blocks[wkey]
					if _is_voxel_passable(wb):
						continue
					var wkind := str(wb.get("kind", "block"))
					if wkind == "slab" or wkind == "stairs" or wkind == "chair" or wkind == "sofa" or wkind == "bed":
						continue
					var wtop := _voxel_support_top_y(wb)
					var wbot := float(int(wb.get("y", 1)) - 1)
					if player_pos.y + 0.35 < wtop and player_pos.y + 1.55 > wbot:
						var bx := float(int(wb.get("x", 0)))
						var bz := float(int(wb.get("z", 0)))
						var min_x := bx - 0.82
						var max_x := bx + 0.82
						var min_z := bz - 0.82
						var max_z := bz + 0.82
						if next_x > min_x and next_x < max_x and next_z > min_z and next_z < max_z:
							var pen_l := absf(next_x - min_x)
							var pen_r := absf(max_x - next_x)
							var pen_f := absf(next_z - min_z)
							var pen_b := absf(max_z - next_z)
							var mpen := minf(minf(pen_l, pen_r), minf(pen_f, pen_b))
							if mpen == pen_l:
								next_x = min_x
							elif mpen == pen_r:
								next_x = max_x
							elif mpen == pen_f:
								next_z = min_z
							else:
								next_z = max_z

	player_pos.x = next_x
	player_pos.z = next_z
	player_pos.y += player_vel.y * delta

	var bound := 103.0 if active_zone == ZONE_PLAZA else (40.0 if active_zone == ZONE_DUNGEON else (15.5 if active_zone == ZONE_PRISON else 56.0))
	player_pos.x = clampf(player_pos.x, -bound, bound)
	player_pos.z = clampf(player_pos.z, -bound, bound)

	# Resolve support surface across active zone platforms + 1-indexed saved blocks in Plaza
	is_grounded = false
	var best_top := -999.0
	var support_rec: Dictionary = {}
	for c in colliders_by_zone.get(active_zone, []):
		if not bool(c.get("solid", true)):
			continue
		if player_pos.x + 0.32 >= float(c["min_x"]) and player_pos.x - 0.32 <= float(c["max_x"]) and player_pos.z + 0.32 >= float(c["min_z"]) and player_pos.z - 0.32 <= float(c["max_z"]):
			var ty := float(c["top_y"])
			if prev_y >= ty - 0.55 and player_pos.y <= ty + 0.08 and player_vel.y <= 0.0:
				if ty > best_top:
					best_top = ty
					support_rec = c

	if active_zone == ZONE_PLAZA:
		var gx := int(round(player_pos.x))
		var gz := int(round(player_pos.z))
		var base_cy := int(floor(player_pos.y))
		for dy in range(0, 4):
			var key := "%d,%d,%d" % [gx, base_cy + dy, gz]
			if saved_blocks.has(key):
				var b: Dictionary = saved_blocks[key]
				if not _is_voxel_passable(b):
					var bkind := str(b.get("kind", "block"))
					if bkind == "door" or bkind == "double_door":
						continue
					var btop := _voxel_support_top_y(b)
					if prev_y >= btop - 0.65 and player_pos.y <= btop + 0.10 and player_vel.y <= 0.0 and btop > best_top:
						best_top = btop
						support_rec = {"type": "voxel"}

	if best_top > -900.0:
		player_pos.y = best_top
		player_vel.y = 0.0
		is_grounded = true
		var stype := str(support_rec.get("type", ""))
		if stype == "moving":
			player_pos.x += float(support_rec.get("vel_x", 0.0)) * delta
			player_pos.z += float(support_rec.get("vel_z", 0.0)) * delta
		elif stype == "jumppad":
			player_vel.y = float(support_rec.get("bounce_vel", 20.0))
			is_grounded = false
		elif stype == "checkpoint":
			checkpoint_pos = Vector3(float(support_rec["pos"].x), best_top + 0.25, float(support_rec["pos"].z))

	if player_pos.y < -12.0:
		player_pos = checkpoint_pos
		player_vel = Vector3.ZERO

	if self_rig != null:
		self_rig.position = player_pos
		self_rig.rotation.y = player_yaw + PI
		self_rig.set_speaking(net != null and net.ptt_active)
		self_rig.set_motion_and_emote(_compute_current_anim(), active_emote)

func _compute_current_anim() -> String:
	if not is_grounded:
		return "jump"
	var hspd := Vector2(player_vel.x, player_vel.z).length()
	if not equipped_mount.is_empty() and hspd > 0.4:
		return "hover"
	if hspd > 5.2:
		return "run"
	if hspd > 0.35:
		return "walk"
	return "idle"

func _step_camera() -> void:
	if main_camera == null or touch == null:
		return
	var target := player_pos + Vector3(0.0, 1.55, 0.0)
	var dist := touch.camera_distance
	var yaw := touch.camera_yaw
	var pitch := touch.camera_pitch
	var offset := Vector3(
		sin(yaw) * cos(pitch) * dist,
		-sin(pitch) * dist,
		cos(yaw) * cos(pitch) * dist
	)
	main_camera.position = main_camera.position.lerp(target + offset, 0.22)
	main_camera.look_at(target, Vector3.UP)

func _step_remote_avatars(delta: float) -> void:
	for pid in remote_rigs.keys():
		var rec: Dictionary = remote_rigs[pid]
		var rig: AvatarRig = rec["rig"]
		var rzone := int(rec.get("zone", ZONE_PLAZA))
		rig.visible = (rzone == active_zone)
		rig.position = rig.position.lerp(rec["target_pos"] as Vector3, minf(1.0, delta * 14.0))
		rig.rotation.y = lerp_angle(rig.rotation.y, float(rec["target_yaw"]) + PI, minf(1.0, delta * 14.0))

	if net != null:
		for did in dungeon_drops.keys():
			var d: Dictionary = dungeon_drops[did]
			var dp := Vector3(float(d.get("x", 0.0)), float(d.get("y", 0.0)), float(d.get("z", 0.0)))
			if player_pos.distance_to(dp) < 2.2:
				net.send_game_command({"type": "dungeon_pickup", "dropId": str(did)})

func _step_speedrun_gates_and_portals(now_ms: int) -> void:
	if active_zone == ZONE_PLAZA and net != null:
		if player_pos.distance_to(Vector3(0.0, 0.4, -12.0)) < 3.6 and active_speedrun_course != "easy":
			active_speedrun_course = "easy"
			active_speedrun_start_ms = now_ms
			net.send_game_command({"type": "obby", "action": "start", "course": "easy"})
		elif active_speedrun_course == "easy" and player_pos.distance_to(Vector3(6.5, 16.5, -20.0)) < 4.5:
			active_speedrun_course = ""
			net.send_game_command({"type": "obby", "action": "finish", "course": "easy"})

		if player_pos.distance_to(Vector3(52.0, 0.4, 52.0)) < 3.8 and active_speedrun_course != "hard":
			active_speedrun_course = "hard"
			active_speedrun_start_ms = now_ms
			net.send_game_command({"type": "obby", "action": "start", "course": "hard"})
		elif active_speedrun_course == "hard" and player_pos.distance_to(Vector3(72.6, 40.2, 69.2)) < 4.6:
			active_speedrun_course = ""
			net.send_game_command({"type": "obby", "action": "finish", "course": "hard"})

		if now_ms - last_portal_ms > 2800:
			if Vector2(player_pos.x - PLAZA_PORTAL_POS.x, player_pos.z - PLAZA_PORTAL_POS.z).length() < 2.4 and absf(player_pos.y) < 3.2:
				last_portal_ms = now_ms
				net.send_game_command({"type": "zone_portal", "zone": ZONE_DUNGEON})
			elif Vector2(player_pos.x - PLAZA_ARENA_PORTAL_POS.x, player_pos.z - PLAZA_ARENA_PORTAL_POS.z).length() < 2.4 and absf(player_pos.y) < 3.2:
				last_portal_ms = now_ms
				net.send_game_command({"type": "zone_portal", "zone": ZONE_ARENA})

	var sr_label := ""
	if not active_speedrun_course.is_empty():
		var elapsed := float(now_ms - active_speedrun_start_ms) * 0.001
		sr_label = "%s %.1fs" % ["🌈 Obby" if active_speedrun_course == "easy" else "🔥 Pro", elapsed]
	stats_updated.emit(coins_balance, arena_hp, arena_max_hp, active_zone, sr_label)

func _update_context_hint() -> void:
	var hint := ""
	if active_zone == ZONE_PLAZA:
		if player_pos.distance_to(PLAZA_PORTAL_POS) < 4.2:
			hint = "🌀 Tap ⚡ Action to Enter Crystal Cavern"
		elif player_pos.distance_to(PLAZA_ARENA_PORTAL_POS) < 4.2:
			hint = "⚔️ Tap ⚡ Action to Enter Shadow Rift Arena"
		elif player_pos.distance_to(CRAFTING_BENCH_POS) < 4.2:
			hint = "🔨 Starlight Crafting Bench — Open Homes & Craft Menu!"
		elif build_mode:
			hint = "🧱 Build Mode ON (%s, rot %d°) — Tap ⚡ or Screen to %s" % [selected_block_kind, selected_block_rot * 90, build_tool]
		else:
			var interact := find_nearest_interactive_plaza_target()
			if not interact.is_empty():
				hint = str(interact.get("label", ""))
	elif active_zone == ZONE_DUNGEON:
		hint = "💎 Crystal Cavern — Tap ⚡ Near Veins, Chests, or Mobs!"
	elif active_zone == ZONE_ARENA:
		if arena_dead:
			hint = "💀 Defeated! Tap ⚡ Action to Respawn at Safe Sanctuary"
		elif player_pos.distance_to(BOSS_ALTAR_POS) < 5.2 and not bool(arena_boss.get("active", false)):
			hint = "🐉 Tap ⚡ Action to Summon Obsidian Rift Dragon!"
		else:
			hint = "⚔️ Shadow Rift Arena — Tap ⚡ to Blast Mobs & Jump Over Boss Waves!"
	elif active_zone == ZONE_PRISON:
		hint = "🧹 Debtor's Yard (%d chores left) — Tap ⚡ Near Chore Stations!" % prison_tasks_remaining
	context_hint_changed.emit(hint)

# ============================================================================
# Server Snapshot & Live Event Reconciliation
# ============================================================================

func _on_welcome_snapshot(snap: Dictionary) -> void:
	var self_obj: Dictionary = snap.get("self", {}) if snap.get("self") is Dictionary else {}
	var user_obj: Dictionary = self_obj.get("user", snap.get("user", {})) if self_obj.get("user", snap.get("user", {})) is Dictionary else {}
	var fallback_sid := net.self_id if net != null else ""
	var fallback_uname := net.username if net != null else ""
	var fallback_dname := net.display_name if net != null else ""
	var fallback_role := net.user_role if net != null else "friend"

	if self_rig != null:
		self_rig.set_player_identity(
			str(self_obj.get("id", fallback_sid)),
			str(user_obj.get("username", fallback_uname)),
			str(user_obj.get("displayName", fallback_dname)),
			str(user_obj.get("role", fallback_role))
		)
		if user_obj.get("avatar") is Dictionary:
			self_rig.apply_avatar_appearance(user_obj["avatar"])
		equipped_pet = str(user_obj.get("equippedPet", "")) if user_obj.get("equippedPet") != null else ""
		equipped_mount = str(user_obj.get("equippedMount", "")) if user_obj.get("equippedMount") != null else ""
		self_rig.set_companions(equipped_pet, equipped_mount)

	if user_obj.get("moderation") is Dictionary:
		_apply_moderation_state(user_obj["moderation"])
	if snap.get("moderation") is Dictionary:
		_apply_moderation_state(snap["moderation"])

	if self_obj.has("x"):
		player_pos = Vector3(float(self_obj.get("x", 0.0)), maxf(0.2, float(self_obj.get("y", 0.2))), float(self_obj.get("z", 4.0)))
		checkpoint_pos = player_pos
	if self_obj.has("zone"):
		_set_zone_visibility(int(self_obj["zone"]))

	if snap.get("economy") is Dictionary:
		_apply_economy_state(snap["economy"])
	if snap.get("companions") is Dictionary:
		_apply_companions_state(snap["companions"])
	if snap.get("blocks") is Array:
		_rebuild_all_voxels(snap["blocks"])
	if snap.get("homes") is Array:
		_apply_home_plots(snap["homes"])
	elif snap.get("homePlots") is Array:
		_apply_home_plots(snap["homePlots"])
	if snap.get("obbyLeaderboards") is Dictionary:
		_apply_obby_leaderboards(snap["obbyLeaderboards"])
	if snap.get("sky") is Dictionary:
		_apply_sky_state(snap["sky"])
	if snap.get("runway") is Dictionary:
		runway_state = snap["runway"]
	if snap.get("dungeon") is Dictionary:
		_apply_dungeon_sync(snap["dungeon"])
	if snap.get("arena") is Dictionary:
		_apply_arena_sync(snap["arena"])
	if snap.get("arenaStatus") is Dictionary:
		_apply_arena_player_status(snap["arenaStatus"])
	if snap.get("prison") is Dictionary:
		_apply_prison_sync(snap["prison"])

	var seen := {}
	if snap.get("players") is Array:
		for p in snap["players"]:
			if p is Dictionary:
				var pid := str(p.get("id", ""))
				if not pid.is_empty() and pid != fallback_sid:
					seen[pid] = true
					_upsert_remote_player_full(p)
	for existing_id in remote_rigs.keys():
		if not seen.has(existing_id):
			(remote_rigs[existing_id]["rig"] as Node3D).queue_free()
			remote_rigs.erase(existing_id)

func _on_world_event(ev: Dictionary) -> void:
	var t := str(ev.get("type", ""))
	var cur_sid := net.self_id if net != null else ""
	var cur_uname := net.username if net != null else ""
	match t:
		"world_sync_digest":
			if ev.get("players") is Array:
				for p in ev["players"]:
					if p is Dictionary and str(p.get("id", "")) != cur_sid:
						_upsert_remote_player_full(p)
			if ev.get("homes") is Array:
				_apply_home_plots(ev["homes"])
			if ev.get("sky") is Dictionary:
				_apply_sky_state(ev["sky"])
			if ev.get("runway") is Dictionary:
				runway_state = ev["runway"]
		"player_join":
			if ev.get("player") is Dictionary and str(ev["player"].get("id", "")) != cur_sid:
				_upsert_remote_player_full(ev["player"])
		"player_leave":
			var pid := str(ev.get("id", ""))
			if remote_rigs.has(pid):
				(remote_rigs[pid]["rig"] as Node3D).queue_free()
				remote_rigs.erase(pid)
		"avatar_updated":
			var u_dict: Dictionary = ev.get("user", {}) if ev.get("user") is Dictionary else {}
			var uname := str(u_dict.get("username", ev.get("username", "")))
			var av: Dictionary = u_dict.get("avatar", ev.get("avatar", {})) if u_dict.get("avatar", ev.get("avatar", {})) is Dictionary else {}
			var pet := str(u_dict.get("equippedPet", "")) if u_dict.get("equippedPet") != null else ""
			var mnt := str(u_dict.get("equippedMount", "")) if u_dict.get("equippedMount") != null else ""
			if uname.to_lower() == cur_uname.to_lower() and self_rig != null:
				if not av.is_empty():
					self_rig.apply_avatar_appearance(av)
				equipped_pet = pet
				equipped_mount = mnt
				self_rig.set_companions(pet, mnt)
			for rpid in remote_rigs.keys():
				var rig: AvatarRig = remote_rigs[rpid]["rig"]
				if rig.username.to_lower() == uname.to_lower():
					if not av.is_empty():
						rig.apply_avatar_appearance(av)
					rig.set_companions(pet, mnt)
		"companions_update", "companions_sync":
			if ev.get("companions") is Dictionary:
				_apply_companions_state(ev["companions"])
			if ev.get("economy") is Dictionary:
				_apply_economy_state(ev["economy"])
		"block":
			if ev.get("blocks") is Array:
				_sync_voxels_diff(ev["blocks"])
			else:
				_apply_single_block_event(ev)
		"player_zone", "zone_transition":
			var zid := str(ev.get("id", ""))
			var nz := int(ev.get("zone", ZONE_PLAZA))
			if zid == cur_sid:
				_set_zone_visibility(nz)
				player_pos = Vector3(float(ev.get("x", 0.0)), maxf(0.2, float(ev.get("y", 0.2))), float(ev.get("z", 4.0)))
				player_vel = Vector3.ZERO
				checkpoint_pos = player_pos
			elif remote_rigs.has(zid):
				remote_rigs[zid]["zone"] = nz
				remote_rigs[zid]["target_pos"] = Vector3(float(ev.get("x", 0.0)), float(ev.get("y", 0.0)), float(ev.get("z", 0.0)))
		"economy_update":
			if ev.get("economy") is Dictionary:
				_apply_economy_state(ev["economy"])
			if ev.get("moderation") is Dictionary:
				_apply_moderation_state(ev["moderation"])
		"moderation_update":
			if ev.get("moderation") is Dictionary:
				_apply_moderation_state(ev["moderation"])
		"craft_ack":
			if ev.get("economy") is Dictionary:
				_apply_economy_state(ev["economy"])
			if ev.get("arenaStatus") is Dictionary:
				_apply_arena_player_status(ev["arenaStatus"])
			toast_requested.emit("🔨 Crafted %s %s!" % [str(ev.get("itemIcon", "✨")), str(ev.get("itemName", "Item"))])
		"obby_speedrun_ack":
			var oact := str(ev.get("action", ""))
			if oact == "finish" and ev.has("timeFormatted"):
				toast_requested.emit("🏁 Obby Finished in %s! (+%d 🪙)" % [str(ev.get("timeFormatted", "")), int(ev.get("coinsAwarded", 0))])
		"homes_sync", "homes_ack":
			if ev.get("plots") is Array:
				_apply_home_plots(ev["plots"])
			elif ev.get("homes") is Array:
				_apply_home_plots(ev["homes"])
			elif ev.get("homePlots") is Array:
				_apply_home_plots(ev["homePlots"])
		"home_doorbell":
			toast_requested.emit("🔔 %s rang the doorbell at %s!" % [str(ev.get("ringerName", "Friend")), str(ev.get("plotName", "Home"))])
		"obby_leaderboard", "obby_sync":
			if ev.get("leaderboards") is Dictionary:
				_apply_obby_leaderboards(ev["leaderboards"])
			elif ev.get("obbyLeaderboards") is Dictionary:
				_apply_obby_leaderboards(ev["obbyLeaderboards"])
		"sky_sync":
			if ev.get("sky") is Dictionary:
				_apply_sky_state(ev["sky"])
		"runway":
			if ev.get("runway") is Dictionary:
				runway_state = ev["runway"]
		"dungeon_sync":
			var dsub: Dictionary = ev["dungeon"] if ev.get("dungeon") is Dictionary else ev
			_apply_dungeon_sync(dsub)
		"dungeon_mobs":
			if ev.get("mobs") is Array:
				_sync_mob_list(ZONE_DUNGEON, dungeon_mobs, ev["mobs"])
		"arena_sync":
			var asub: Dictionary = ev["arena"] if ev.get("arena") is Dictionary else ev
			_apply_arena_sync(asub)
		"arena_mobs":
			if ev.get("mobs") is Array:
				_sync_mob_list(ZONE_ARENA, arena_mobs, ev["mobs"])
			if ev.get("boss") is Dictionary:
				_sync_boss_state(ev["boss"])
		"arena_player_status", "arena_health":
			var st_obj: Dictionary = ev["arenaStatus"] if ev.get("arenaStatus") is Dictionary else ev
			_apply_arena_player_status(st_obj)
			if str(ev.get("event", "")) == "died":
				toast_requested.emit("💀 Defeated by %s! Tap ⚡ Action to Respawn!" % str(ev.get("source", "Shadow Rift")))
		"prison_sync":
			var psub: Dictionary = ev["prison"] if ev.get("prison") is Dictionary else ev
			_apply_prison_sync(psub)
		"action_error":
			toast_requested.emit(str(ev.get("message", ev.get("error", "Action blocked."))))

func _apply_arena_player_status(st: Dictionary) -> void:
	arena_hp = int(st.get("hp", arena_hp))
	arena_max_hp = int(st.get("maxHp", arena_max_hp))
	if st.has("alive"):
		arena_dead = not bool(st["alive"])
	elif st.has("dead"):
		arena_dead = bool(st["dead"])
	else:
		arena_dead = (arena_hp <= 0)

func _apply_moderation_state(mod: Dictionary) -> void:
	prison_tasks_remaining = int(mod.get("prisonTasksRemaining", prison_tasks_remaining))
	in_debtors_prison = bool(mod.get("inDebtorsPrison", in_debtors_prison))

func _on_remote_states(players: Array, _rtt_ms: float) -> void:
	var cur_sid := net.self_id if net != null else ""
	for p in players:
		if not (p is Dictionary):
			continue
		var pid := str(p.get("id", ""))
		if pid.is_empty() or pid == cur_sid:
			continue
		if not remote_rigs.has(pid):
			continue
		var rec: Dictionary = remote_rigs[pid]
		rec["target_pos"] = Vector3(float(p.get("x", 0.0)), float(p.get("y", 0.0)), float(p.get("z", 0.0)))
		rec["target_yaw"] = float(p.get("ry", 0.0))
		rec["zone"] = int(p.get("zone", ZONE_PLAZA))
		var rig: AvatarRig = rec["rig"]
		rig.set_speaking(bool(p.get("speaking", false)) and not bool(p.get("muted", true)))
		rig.set_motion_and_emote(str(p.get("anim", "idle")), str(p.get("emote", "")))

func _upsert_remote_player_full(p: Dictionary) -> void:
	var pid := str(p.get("id", ""))
	var u: Dictionary = p.get("user", {}) if p.get("user") is Dictionary else {}
	var rec: Dictionary
	if remote_rigs.has(pid):
		rec = remote_rigs[pid]
	else:
		var rig := AvatarRig.new()
		add_child(rig)
		rec = {
			"rig": rig,
			"target_pos": Vector3(float(p.get("x", 0.0)), float(p.get("y", 0.0)), float(p.get("z", 0.0))),
			"target_yaw": float(p.get("ry", 0.0)),
			"zone": int(p.get("zone", ZONE_PLAZA))
		}
		rig.position = rec["target_pos"]
		remote_rigs[pid] = rec

	var r: AvatarRig = rec["rig"]
	r.set_player_identity(pid, str(u.get("username", "friend")), str(u.get("displayName", "Friend")), str(u.get("role", "friend")))
	if u.get("avatar") is Dictionary:
		r.apply_avatar_appearance(u["avatar"])
	var pet := str(u.get("equippedPet", "")) if u.get("equippedPet") != null else ""
	var mnt := str(u.get("equippedMount", "")) if u.get("equippedMount") != null else ""
	r.set_companions(pet, mnt)
	rec["zone"] = int(p.get("zone", rec["zone"]))

func _set_zone_visibility(zone: int) -> void:
	active_zone = zone
	plaza_root.visible = (zone == ZONE_PLAZA)
	dungeon_root.visible = (zone == ZONE_DUNGEON)
	prison_root.visible = (zone == ZONE_PRISON)
	arena_root.visible = (zone == ZONE_ARENA)
	if world_env != null and world_env.environment != null:
		match zone:
			ZONE_DUNGEON:
				world_env.environment.background_color = Color("#0f0a24")
			ZONE_ARENA:
				world_env.environment.background_color = Color("#2a0808")
			ZONE_PRISON:
				world_env.environment.background_color = Color("#1e293b")
			_:
				world_env.environment.background_color = Color("#7dd3fc")

func _apply_economy_state(econ: Dictionary) -> void:
	coins_balance = int(econ.get("coins", coins_balance))
	if econ.has("prisonTasksRemaining"):
		prison_tasks_remaining = int(econ["prisonTasksRemaining"])
	if econ.get("inventory") is Dictionary:
		inventory_items = econ["inventory"]

func _apply_companions_state(comp: Dictionary) -> void:
	if comp.get("ownedPets") is Array:
		owned_pets = comp["ownedPets"]
	elif comp.get("unlockedPets") is Array:
		owned_pets = comp["unlockedPets"]
	if comp.get("ownedMounts") is Array:
		owned_mounts = comp["ownedMounts"]
	elif comp.get("unlockedMounts") is Array:
		owned_mounts = comp["unlockedMounts"]
	equipped_pet = str(comp.get("equippedPet", "")) if comp.get("equippedPet") != null else ""
	equipped_mount = str(comp.get("equippedMount", "")) if comp.get("equippedMount") != null else ""
	if self_rig != null:
		self_rig.set_companions(equipped_pet, equipped_mount)

static func _voxel_sig(b: Dictionary) -> String:
	return "%s|%s|%d|%s|%s" % [
		str(b.get("kind", "block")),
		str(b.get("color", "#f472b6")),
		int(b.get("rot", 0)),
		"1" if bool(b.get("open", false)) else "0",
		str(b.get("lootItemId", ""))
	]

func _rebuild_all_voxels(blocks: Array) -> void:
	for k in voxel_nodes.keys():
		(voxel_nodes[k] as Node3D).queue_free()
	voxel_nodes.clear()
	voxel_sigs.clear()
	saved_blocks.clear()
	for b in blocks:
		if b is Dictionary:
			_place_voxel_visual(b)

func _sync_voxels_diff(blocks: Array) -> void:
	var seen := {}
	for b in blocks:
		if not (b is Dictionary):
			continue
		var key := "%d,%d,%d" % [int(b.get("x", 0)), int(b.get("y", 1)), int(b.get("z", 0))]
		seen[key] = true
		var sig := _voxel_sig(b)
		if voxel_sigs.get(key, "") != sig:
			_place_voxel_visual(b)
	for existing_key in voxel_nodes.keys():
		if not seen.has(existing_key):
			(voxel_nodes[existing_key] as Node3D).queue_free()
			voxel_nodes.erase(existing_key)
			voxel_sigs.erase(existing_key)
			saved_blocks.erase(existing_key)

func _apply_single_block_event(ev: Dictionary) -> void:
	var act := str(ev.get("action", "place"))
	var x := int(ev.get("x", 0))
	var y := int(ev.get("y", 1))
	var z := int(ev.get("z", 0))
	var key := "%d,%d,%d" % [x, y, z]
	if act == "remove" or act == "clear":
		if act == "clear":
			_rebuild_all_voxels([])
		elif voxel_nodes.has(key):
			(voxel_nodes[key] as Node3D).queue_free()
			voxel_nodes.erase(key)
			voxel_sigs.erase(key)
			saved_blocks.erase(key)
	else:
		_place_voxel_visual(ev)

func _place_voxel_visual(b: Dictionary) -> void:
	var x := int(b.get("x", 0))
	var y := maxi(1, int(b.get("y", 1)))
	var z := int(b.get("z", 0))
	var key := "%d,%d,%d" % [x, y, z]
	if voxel_nodes.has(key):
		(voxel_nodes[key] as Node3D).queue_free()
	saved_blocks[key] = b
	voxel_sigs[key] = _voxel_sig(b)

	var kind := str(b.get("kind", "block"))
	var col_hex := str(b.get("color", "#f472b6"))
	var rot := int(b.get("rot", 0))
	var is_open := bool(b.get("open", false))
	var loot_id := str(b.get("lootItemId", "")) if b.get("lootItemId") != null else ""
	var base_bottom_y := float(y - 1)
	var node: Node3D = null

	# 1. Placed Dungeon/Crafted Loot Trophy or Decor Item
	if not loot_id.is_empty() and LOOT_NODE_MAP.has(loot_id):
		var loot_tpl := clone_prop_template(str(LOOT_NODE_MAP[loot_id]))
		if loot_tpl != null:
			loot_tpl.position = Vector3(float(x), base_bottom_y, float(z))
			loot_tpl.rotation.y = float(rot) * (PI * 0.5)
			node = loot_tpl

	# 2. Hinged Single & Double Doors
	if node == null and (kind == "door" or kind == "double_door"):
		var door_grp := Node3D.new()
		door_grp.position = Vector3(float(x), base_bottom_y, float(z))
		door_grp.rotation.y = float(rot) * (PI * 0.5)
		var frame := clone_prop_template("Prop_Door_Frame")
		var leaf := clone_prop_template("Prop_Door_Leaf")
		var pivot := Node3D.new()
		pivot.position = Vector3(-0.41, 0.0, 0.0)
		if frame != null and leaf != null:
			door_grp.add_child(frame)
			pivot.add_child(leaf)
		else:
			var dm := MeshInstance3D.new()
			var db := BoxMesh.new()
			db.size = Vector3(1.85 if kind == "double_door" else 0.92, 1.95, 0.12)
			dm.mesh = db
			dm.position = Vector3(0.41, 0.98, 0.0)
			var dmat := StandardMaterial3D.new()
			dmat.albedo_color = Color.from_string(col_hex, Color("#f472b6"))
			dm.material_override = dmat
			pivot.add_child(dm)
		pivot.rotation.y = PI * 0.48 if is_open else 0.0
		door_grp.add_child(pivot)
		node = door_grp

	# 3. Shared Blender 5.2 Interior Furniture & Stairs Props (`base_bottom_y = y - 1`)
	if node == null and PROP_NODE_MAP.has(kind):
		var prop_tpl := clone_prop_template(str(PROP_NODE_MAP[kind]))
		if prop_tpl != null:
			prop_tpl.position = Vector3(float(x), base_bottom_y, float(z))
			prop_tpl.rotation.y = float(rot) * (PI * 0.5)
			node = prop_tpl

	# 4. Standard Voxel Primitives (`block`, `glass`, `slab`, `window`, fallback)
	if node == null:
		var mi := MeshInstance3D.new()
		var bm := BoxMesh.new()
		var y_center := base_bottom_y + 0.5
		if kind == "slab":
			bm.size = Vector3(1.0, 0.5, 1.0)
			y_center = base_bottom_y + 0.25
		elif kind == "rug":
			bm.size = Vector3(1.95, 0.06, 1.95)
			y_center = base_bottom_y + 0.03
		elif kind == "window":
			bm.size = Vector3(0.96, 0.96, 0.16)
		else:
			bm.size = Vector3(1.0, 1.0, 1.0)
		mi.mesh = bm
		var mat := StandardMaterial3D.new()
		mat.albedo_color = Color.from_string(col_hex, Color("#f472b6"))
		if kind == "glass" or kind == "window":
			mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			mat.albedo_color.a = 0.44
		mi.material_override = mat
		mi.position = Vector3(float(x), y_center, float(z))
		mi.rotation.y = float(rot) * (PI * 0.5)
		node = mi

	# Attach toggleable warm OmniLight3D on interactive lamps (`open == false` => ON)
	if (kind == "crystal_lamp" or kind == "nightstand_lamp" or loot_id.ends_with("_lamp")) and not is_open:
		var lamp_light := OmniLight3D.new()
		lamp_light.position = Vector3(0.0, 0.85, 0.0)
		lamp_light.light_color = Color("#fde68a")
		lamp_light.light_energy = 1.25
		lamp_light.omni_range = 7.5
		node.add_child(lamp_light)

	voxels_root.add_child(node)
	voxel_nodes[key] = node

func _apply_home_plots(plots: Array) -> void:
	home_plots_state = plots
	for p in plots:
		if not (p is Dictionary):
			continue
		var pid := str(p.get("plotId", ""))
		if mailbox_labels.has(pid):
			var lbl: Label3D = mailbox_labels[pid]
			var owner_dname := str(p.get("ownerDisplayName", "")) if p.get("ownerDisplayName") != null else ""
			var title := str(p.get("customTitle", "")) if p.get("customTitle") != null else str(p.get("defaultName", pid))
			if owner_dname.is_empty():
				lbl.text = "%s\n🟢 Unclaimed — Tap Homes to Claim!" % title
			else:
				lbl.text = "%s\n👑 Owner: %s%s" % [title, owner_dname, " 🔒" if bool(p.get("doorLocked", false)) else ""]

func _apply_obby_leaderboards(lb: Dictionary) -> void:
	obby_leaderboards = lb
	for course_id in ["easy", "hard"]:
		if not leaderboard_labels.has(course_id):
			continue
		var lbl: Label3D = leaderboard_labels[course_id]
		var rows: Array = lb.get(course_id, []) if lb.get(course_id) is Array else []
		var header := "🌈 SKY OBBY TOP 5" if course_id == "easy" else "🔥 PRO GAUNTLET TOP 5"
		var lines := PackedStringArray([header])
		if rows.is_empty():
			lines.append("No records yet — Set #1!")
		else:
			for i in range(mini(rows.size(), 5)):
				var r: Dictionary = rows[i]
				lines.append("#%d %s — %s" % [i + 1, str(r.get("displayName", "Player")), str(r.get("timeFormatted", ""))])
		lbl.text = "\n".join(lines)

func _apply_sky_state(sky: Dictionary) -> void:
	var phase := str(sky.get("phase", "day"))
	if active_zone == ZONE_PLAZA and world_env != null and world_env.environment != null:
		match phase:
			"sunset":
				world_env.environment.background_color = Color("#fb7185")
				sun_light.light_color = Color("#fdba74")
				sun_light.light_energy = 0.95
			"night", "aurora_night":
				world_env.environment.background_color = Color("#0f172a")
				sun_light.light_color = Color("#818cf8")
				sun_light.light_energy = 0.45
			_:
				world_env.environment.background_color = Color("#7dd3fc")
				sun_light.light_color = Color("#fffbeb")
				sun_light.light_energy = 1.25

	sky_falling_stars = sky.get("fallingStars", []) if sky.get("fallingStars") is Array else []
	for c in sky_shards_root.get_children():
		c.queue_free()
	for s in sky_falling_stars:
		if s is Dictionary:
			var mi := MeshInstance3D.new()
			var sm := SphereMesh.new()
			sm.radius = 0.48
			mi.mesh = sm
			mi.position = Vector3(float(s.get("x", 0.0)), float(s.get("y", 0.6)) + 0.5, float(s.get("z", 0.0)))
			var mat := StandardMaterial3D.new()
			mat.albedo_color = Color("#fde047")
			mat.emission_enabled = true
			mat.emission = Color("#fbbf24")
			mat.emission_energy_multiplier = 1.2
			mi.material_override = mat
			sky_shards_root.add_child(mi)

func _apply_dungeon_sync(d: Dictionary) -> void:
	if d.get("nodes") is Array:
		dungeon_nodes.clear()
		for n in d["nodes"]:
			if n is Dictionary:
				dungeon_nodes[str(n.get("id", ""))] = n
		_sync_resource_markers(ZONE_DUNGEON, "dnode_", dungeon_nodes, "#38bdf8")
	if d.get("mobs") is Array:
		_sync_mob_list(ZONE_DUNGEON, dungeon_mobs, d["mobs"])
	if d.get("drops") is Array:
		dungeon_drops.clear()
		for dr in d["drops"]:
			if dr is Dictionary:
				dungeon_drops[str(dr.get("id", ""))] = dr
		_sync_resource_markers(active_zone, "drop_", dungeon_drops, "#fbbf24")

func _apply_arena_sync(a: Dictionary) -> void:
	if a.get("mobs") is Array:
		_sync_mob_list(ZONE_ARENA, arena_mobs, a["mobs"])
	if a.get("boss") is Dictionary:
		_sync_boss_state(a["boss"])
	if a.get("drops") is Array:
		for dr in a["drops"]:
			if dr is Dictionary:
				dungeon_drops[str(dr.get("id", ""))] = dr
		_sync_resource_markers(ZONE_ARENA, "adrop_", dungeon_drops, "#fbbf24")

func _apply_prison_sync(p: Dictionary) -> void:
	if p.get("nodes") is Array:
		prison_nodes.clear()
		for n in p["nodes"]:
			if n is Dictionary:
				prison_nodes[str(n.get("id", ""))] = n
		_sync_resource_markers(ZONE_PRISON, "pnode_", prison_nodes, "#fbbf24")

func _sync_resource_markers(zone: int, prefix: String, map_data: Dictionary, hex_col: String) -> void:
	var parent := dungeon_root if zone == ZONE_DUNGEON else (prison_root if zone == ZONE_PRISON else arena_root)
	var valid := {}
	for k in map_data.keys():
		var mkey := prefix + str(k)
		valid[mkey] = true
		var item: Dictionary = map_data[k]
		var node: Node3D
		if entity_meshes.has(mkey):
			node = entity_meshes[mkey]
		else:
			var mi := MeshInstance3D.new()
			var pm := PrismMesh.new()
			pm.size = Vector3(0.9, 1.4, 0.9)
			mi.mesh = pm
			var mat := StandardMaterial3D.new()
			var nkind := str(item.get("kind", ""))
			var c_hex := "#fbbf24" if nkind.contains("chest") else hex_col
			mat.albedo_color = Color.from_string(c_hex, Color("#38bdf8"))
			mat.emission_enabled = true
			mat.emission = mat.albedo_color
			mat.emission_energy_multiplier = 0.65
			mi.material_override = mat
			parent.add_child(mi)
			entity_meshes[mkey] = mi
			node = mi
		node.position = Vector3(float(item.get("x", 0.0)), float(item.get("y", 0.0)) + 0.75, float(item.get("z", 0.0)))
		node.visible = bool(item.get("available", item.get("ready", true)))

	for ek in entity_meshes.keys():
		if str(ek).begins_with(prefix) and not valid.has(ek):
			(entity_meshes[ek] as Node3D).queue_free()
			entity_meshes.erase(ek)

func _instantiate_mob_visual(mkind: String, zone: int) -> Node3D:
	var lower := mkind.to_lower()
	if lower.contains("bat"):
		var body := clone_env_template("Env_Mob_CrystalBat_Body")
		if body != null:
			var grp := Node3D.new()
			grp.add_child(body)
			var wl := clone_env_template("Env_Mob_CrystalBat_WingL")
			var wr := clone_env_template("Env_Mob_CrystalBat_WingR")
			if wl != null:
				grp.add_child(wl)
			if wr != null:
				grp.add_child(wr)
			return grp
	elif lower.contains("golem"):
		var gbody := clone_env_template("Env_Mob_LavaGolem_Body")
		if gbody != null:
			var grp := Node3D.new()
			grp.add_child(gbody)
			var al := clone_env_template("Env_Mob_LavaGolem_ArmL")
			var ar := clone_env_template("Env_Mob_LavaGolem_ArmR")
			if al != null:
				grp.add_child(al)
			if ar != null:
				grp.add_child(ar)
			return grp
	elif lower.contains("wraith"):
		var w := clone_env_template("Env_Mob_VoidWraith")
		if w != null:
			return w
	elif lower.contains("spider"):
		var sp := clone_env_template("Env_Mob_RiftSpider")
		if sp != null:
			return sp
	else:
		var sl := clone_env_template("Env_Mob_ShadowSlime")
		if sl != null:
			return sl

	var mi := MeshInstance3D.new()
	var sm := SphereMesh.new()
	sm.radius = 0.65
	sm.height = 1.3
	mi.mesh = sm
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color("#ef4444" if zone == ZONE_ARENA else "#a855f7")
	mat.emission_enabled = true
	mat.emission = mat.albedo_color
	mat.emission_energy_multiplier = 0.45
	mi.material_override = mat
	mi.position.y = 0.65
	return mi

func _sync_mob_list(zone: int, store: Dictionary, arr: Array) -> void:
	var prefix := "dmob_" if zone == ZONE_DUNGEON else "amob_"
	var parent := dungeon_root if zone == ZONE_DUNGEON else arena_root
	store.clear()
	var valid := {}
	for m in arr:
		if not (m is Dictionary):
			continue
		var mid := str(m.get("id", ""))
		store[mid] = m
		var mkey := prefix + mid
		valid[mkey] = true
		var node: Node3D
		if entity_meshes.has(mkey):
			node = entity_meshes[mkey]
		else:
			node = _instantiate_mob_visual(str(m.get("kind", "shadow_slime")), zone)
			parent.add_child(node)
			entity_meshes[mkey] = node
		node.position = Vector3(float(m.get("x", 0.0)), float(m.get("y", 0.0)), float(m.get("z", 0.0)))
		node.rotation.y = float(m.get("ry", 0.0))
		node.visible = bool(m.get("alive", true))

	for ek in entity_meshes.keys():
		if str(ek).begins_with(prefix) and not valid.has(ek):
			(entity_meshes[ek] as Node3D).queue_free()
			entity_meshes.erase(ek)

func _ensure_shockwave_ring(idx: int) -> MeshInstance3D:
	while boss_shockwave_pool.size() <= idx:
		var ring := MeshInstance3D.new()
		var torus := TorusMesh.new()
		torus.inner_radius = 0.90
		torus.outer_radius = 1.0
		ring.mesh = torus
		var sw_mat := StandardMaterial3D.new()
		sw_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		sw_mat.albedo_color = Color(0.75, 0.52, 0.99, 0.88)
		ring.material_override = sw_mat
		ring.visible = false
		boss_shockwaves_root.add_child(ring)
		boss_shockwave_pool.append(ring)
	return boss_shockwave_pool[idx]

func _sync_boss_state(b: Dictionary) -> void:
	arena_boss = b
	var mkey := "arena_dragon_boss"
	var active := bool(b.get("active", false)) and int(b.get("hp", 0)) > 0
	if not active:
		if entity_meshes.has(mkey):
			(entity_meshes[mkey] as Node3D).visible = false
		for ring in boss_shockwave_pool:
			ring.visible = false
		return

	var node: Node3D
	if entity_meshes.has(mkey):
		node = entity_meshes[mkey]
	else:
		var grp := Node3D.new()
		var dbody := clone_env_template("Env_Boss_ObsidianDragon_Body")
		if dbody != null:
			grp.add_child(dbody)
			var dwl := clone_env_template("Env_Boss_ObsidianDragon_WingL")
			var dwr := clone_env_template("Env_Boss_ObsidianDragon_WingR")
			if dwl != null:
				grp.add_child(dwl)
			if dwr != null:
				grp.add_child(dwr)
		else:
			var body := MeshInstance3D.new()
			var cm := PrismMesh.new()
			cm.size = Vector3(2.8, 3.8, 3.4)
			body.mesh = cm
			body.position.y = 1.9
			var bmat := StandardMaterial3D.new()
			bmat.albedo_color = Color("#0f172a")
			bmat.emission_enabled = true
			bmat.emission = Color("#9333ea")
			bmat.emission_energy_multiplier = 0.75
			body.material_override = bmat
			grp.add_child(body)
		var lbl := Label3D.new()
		lbl.name = "BossLabel"
		lbl.position = Vector3(0, 4.6, 0)
		lbl.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		lbl.font_size = 28
		lbl.outline_size = 8
		lbl.modulate = Color("#f472b6")
		grp.add_child(lbl)
		arena_root.add_child(grp)
		entity_meshes[mkey] = grp
		node = grp

	node.visible = true
	node.position = Vector3(float(b.get("x", 0.0)), float(b.get("y", 0.0)), float(b.get("z", -24.5)))
	node.rotation.y = float(b.get("ry", 0.0))
	var blbl := node.get_node_or_null("BossLabel") as Label3D
	if blbl != null:
		blbl.text = "🐉 %s (%d / %d HP)" % [str(b.get("name", "Obsidian Rift Dragon")), int(b.get("hp", 0)), int(b.get("maxHp", 18))]

	var waves: Array = b.get("shockwaves", []) if b.get("shockwaves") is Array else []
	var used_rings := 0
	for w in waves:
		if not (w is Dictionary):
			continue
		var rad := float(w.get("radius", 0.0))
		if rad <= 0.2:
			continue
		var ring := _ensure_shockwave_ring(used_rings)
		ring.visible = true
		ring.position = Vector3(float(w.get("x", node.position.x)), 0.25, float(w.get("z", node.position.z)))
		ring.scale = Vector3(rad, 1.0, rad)
		used_rings += 1
	for i in range(used_rings, boss_shockwave_pool.size()):
		boss_shockwave_pool[i].visible = false

func _notification(what: int) -> void:
	if what == NOTIFICATION_PREDELETE:
		for k in _env_templates.keys():
			var n: Variant = _env_templates[k]
			if is_instance_valid(n):
				(n as Node).free()
		_env_templates.clear()
		for k in _prop_templates.keys():
			var n: Variant = _prop_templates[k]
			if is_instance_valid(n):
				(n as Node).free()
		_prop_templates.clear()
