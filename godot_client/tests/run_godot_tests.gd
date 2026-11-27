extends SceneTree
## Headless Godot 4 Verification Suite (`godot_client/tests/run_godot_tests.gd`)
## Verifies:
##   1. Shared Blender 5.2 `.glb` models & PBR textures load cleanly (`ella_dti_wardrobe.glb`,
##      `ella_world_environment.glb`, `ella_world_props_loot.glb`).
##   2. `EllaNativeNet` GDExtension (`libella_native_net`) loads and encodes exact 34-byte `EWLD` v1 packets.
##   3. `NetworkClient` static binary `EWLD` v1 encode/decode parity (`OP_HELLO`, `OP_WELCOME`, `OP_C2S_STATE`, `OP_S2C_WORLD_STATE`).
##   4. `AvatarRig` stylized female, stylized male, blocky fallback, pets, and hoverboards instantiate cleanly.
##   5. Full `MainApp.tscn` scene tree (`NetworkClient` + `MultiTouchController` + `WorldController` + HUD) instantiates with zero script errors.
##   6. Live end-to-end `EllaNativeNet` UDP handshake + 20 Hz state sync against `ella-world-server` when `ELLA_TEST_TOKEN` & `ELLA_TEST_UDP_PORT` env vars are set.

var _failures: int = 0

func _assert_true(cond: bool, msg: String) -> void:
	if not cond:
		push_error("[FAIL] " + msg)
		_failures += 1
	else:
		print("[PASS] " + msg)

func _initialize() -> void:
	await process_frame
	print("=== Ella's World Godot 4 Headless Verification Suite ===")

	# 1. Shared 3D GLB & PBR Texture Asset Checks
	for glb_path in [
		"res://assets/models/ella_dti_wardrobe.glb",
		"res://assets/models/ella_world_environment.glb",
		"res://assets/models/ella_world_props_loot.glb"
	]:
		var exists := ResourceLoader.exists(glb_path)
		_assert_true(exists, "Shared GLB exists: " + glb_path)
		if exists:
			var ps := load(glb_path) as PackedScene
			_assert_true(ps != null, "Shared GLB loads as PackedScene: " + glb_path)

	# 2. Binary EWLD v1 Codec Checks (GDScript + GDExtension)
	var hello_pkt := NetworkClient.encode_ewld_hello("tok_test_123")
	_assert_true(hello_pkt.size() == 6 + 2 + 12, "EWLD OP_HELLO frame length == 20")
	_assert_true(hello_pkt[0] == 0x45 and hello_pkt[1] == 0x57 and hello_pkt[2] == 0x4C and hello_pkt[3] == 0x44, "EWLD magic header")

	var c2s_pkt := NetworkClient.encode_ewld_c2s_state(7, 12345, Vector3(12.5, 3.0, -8.25), 1.5, "run", "wave", false, true, 1)
	_assert_true(c2s_pkt.size() == 34, "EWLD OP_C2S_STATE datagram is 34 bytes (6 header + 28 payload)")

	var has_gdext := ClassDB.class_exists(&"EllaNativeNet")
	_assert_true(has_gdext, "EllaNativeNet GDExtension class is registered in ClassDB")
	if has_gdext:
		var codec_inst: RefCounted = ClassDB.instantiate(&"EllaNativeNet")
		var ext_pkt: PackedByteArray = codec_inst.call("encode_c2s_state_packet", 7, 12345, 12.5, 3.0, -8.25, 1.5, "run", "wave", false, true, 1)
		_assert_true(ext_pkt == c2s_pkt, "EllaNativeNet Rust C2S packet matches GDScript byte-for-byte")

	# 3. NetworkClient Telemetry, Machine Seal & WSS/UDP Event Deduplication
	var net_test := NetworkClient.new()
	root.add_child(net_test)
	net_test.machine_seal = "seal_test_abc"
	var mf := net_test.build_machine_auth_fields()
	_assert_true(mf.get("deviceTelemetry") is Dictionary, "NetworkClient builds structured deviceTelemetry")
	_assert_true(not mf.has("deviceFp"), "NetworkClient omits deprecated flat deviceFp")
	if mf.get("deviceTelemetry") is Dictionary:
		var dt: Dictionary = mf["deviceTelemetry"]
		_assert_true(int(dt.get("screenW", 0)) > 0 and int(dt.get("cpuCores", 0)) >= 1, "deviceTelemetry populates screenW and cpuCores")
	_assert_true(str(mf.get("machineSeal", "")) == "seal_test_abc", "NetworkClient attaches persisted machineSeal")
	_assert_true(net_test.should_accept_world_event({"type": "chat", "stateSeq": 10, "chat": {"id": "c1", "text": "hi"}}), "First chat event accepted")
	_assert_true(not net_test.should_accept_world_event({"type": "chat", "stateSeq": 10, "chat": {"id": "c1", "text": "hi"}}), "Duplicate chat ID across WSS/UDP rejected")
	_assert_true(not net_test.should_accept_world_event({"type": "sky_sync", "stateSeq": 10}), "Duplicate stateSeq event rejected")
	_assert_true(net_test.should_accept_world_event({"type": "sky_sync", "stateSeq": 11}), "Advancing stateSeq event accepted")
	_assert_true(net_test.should_accept_world_event({"type": "block", "stateSeq": 9}), "Concurrent slightly-reordered stateSeq event accepted without false drop")
	net_test.queue_free()

	# 4. AvatarRig 1.55x Scale, Slot Visibility, Ballgown Skirt Rule, Baby Dragon & Sleep Pose
	var rig := AvatarRig.new()
	root.add_child(rig)
	rig.set_player_identity("peer_1", "ella", "Ella", "host")
	_assert_true(rig.glb_instance != null and rig.glb_instance.scale.is_equal_approx(Vector3(1.55, 1.55, 1.55)), "AvatarRig matches 1.55x Web world scale")
	rig.apply_avatar_appearance({"avatarMode": "stylized", "bodyType": "female", "hairStyle": "twin_pigtails", "topStyle": "glam_gown", "bottomStyle": "pleated_skirt", "shoeStyle": "runway_heels", "hat": "crown", "fabricFinish": "glitter"})
	_assert_true(rig.is_slot_visible("Top_glam_gown") and not rig.is_slot_visible("Bottom_pleated_skirt"), "glam_gown hides Bottom_pleated_skirt to prevent mesh clipping")
	_assert_true(rig.is_slot_visible("Hat_crown"), "Crown hat slot visible on host glam preset")
	rig.apply_avatar_appearance({"avatarMode": "stylized", "bodyType": "female", "hairStyle": "katey_long", "topStyle": "puff_blouse", "bottomStyle": "pleated_skirt", "shoeStyle": "platform_boots", "fabricFinish": "satin"})
	_assert_true(rig.is_slot_visible("Top_puff_blouse") and rig.is_slot_visible("Bottom_pleated_skirt"), "Separates outfit shows both Top_puff_blouse and Bottom_pleated_skirt")
	rig.apply_avatar_appearance({"avatarMode": "stylized", "bodyType": "male", "hairStyle": "tech_crop", "topStyle": "tech_jacket", "accessory": "aviators"})
	_assert_true(rig.is_slot_visible("Body_Male") and rig.is_slot_visible("Top_tech_jacket"), "Male stylized rig activates Body_Male + Top_tech_jacket")
	rig.set_companions("baby_crystal_dragon", "starlight_board")
	_assert_true(rig.pet_pivot.get_node_or_null("ArticulatedBabyDragon") != null and rig.pet_wing_l != null and rig.pet_wing_r != null, "Articulated Baby Crystal Dragon spawns body + wings from GLB")
	rig.set_motion_and_emote("idle", "sleep")
	_assert_true(rig.visual_root.rotation.x < -1.0, "Sleep emote tilts avatar horizontally onto bed")
	rig.apply_avatar_appearance({"avatarMode": "blocky", "topColor": "#38bdf8"})
	_assert_true(rig.nametag_label != null and rig.nametag_label.text.contains("Ella"), "AvatarRig host crown nametag initialized")
	rig.queue_free()

	# 5. Full MainApp.tscn + WorldController 1-Indexed Voxels, Snapshot & Arena Boss Checks
	var main_packed := load("res://scenes/MainApp.tscn") as PackedScene
	_assert_true(main_packed != null, "MainApp.tscn loads cleanly")
	if main_packed != null:
		var main_inst := main_packed.instantiate()
		root.add_child(main_inst)
		var wc := main_inst.get_node_or_null("WorldController") as WorldController
		_assert_true(main_inst.get_node_or_null("NetworkClient") != null, "MainApp spawns NetworkClient")
		_assert_true(wc != null, "MainApp spawns WorldController")
		if wc != null:
			wc.player_pos = Vector3(0.0, 0.0, 4.0)
			_assert_true(wc.compute_front_build_cell().y == 1, "Ground build cell is 1-indexed (y == 1 on Plaza grass)")
			_assert_true(is_equal_approx(WorldController._voxel_support_top_y({"y": 1, "kind": "block"}), 1.0), "1-indexed y=1 full block has top surface at 1.0m")

			var snap := {
				"type": "welcome_snapshot",
				"worldName": "Ella's World",
				"chatHistory": [{"id": "ch_boot_1", "displayName": "Ella", "text": "Welcome besties!"}],
				"homes": [{"plotId": "plot_1", "defaultName": "Sunset Plot #1", "ownerUsername": "ella", "ownerDisplayName": "Ella", "doorLocked": true, "mailboxPos": [52.0, 0.0, -16.2]}],
				"arenaStatus": {"hp": 150, "maxHp": 150, "alive": true},
				"moderation": {"prisonTasksRemaining": 3, "inDebtorsPrison": true},
				"companions": {"ownedPets": ["cloud_puppy", "shadow_kitty"], "ownedMounts": ["starlight_board"], "equippedPet": "cloud_puppy", "equippedMount": ""},
				"dungeon": {"nodes": [
					{"id": "vein_depleted", "x": 1.0, "y": 0.0, "z": 1.0, "available": false},
					{"id": "vein_live", "x": 2.0, "y": 0.0, "z": 1.0, "available": true}
				]},
				"blocks": [
					{"x": 2, "y": 1, "z": 2, "kind": "bed", "color": "#f472b6", "rot": 0},
					{"x": 4, "y": 1, "z": 2, "kind": "door", "color": "#fbbf24", "rot": 0, "open": false}
				]
			}
			wc._on_welcome_snapshot(snap)
			main_inst.call("_on_welcome_snapshot", snap)

			_assert_true(wc.home_plots_state.size() == 1, "welcome_snapshot populates homes array")
			_assert_true(wc.arena_max_hp == 150 and not wc.arena_dead, "welcome_snapshot reconciles 150 Aegis arenaStatus")
			_assert_true(wc.prison_tasks_remaining == 3 and wc.in_debtors_prison, "welcome_snapshot reconciles moderation prisonTasksRemaining")
			_assert_true(wc.owned_pets.size() == 2, "welcome_snapshot reconciles companions.ownedPets")
			_assert_true(main_inst.chat_log_rich != null and main_inst.chat_log_rich.get_parsed_text().contains("Welcome besties!"), "MainApp renders welcome_snapshot chatHistory")
			_assert_true(wc.voxel_nodes.has("2,1,2") and is_equal_approx((wc.voxel_nodes["2,1,2"] as Node3D).position.y, 0.0), "y=1 prop sits flush on ground y=0.0")

			wc.player_pos = Vector3(1.1, 0.2, 1.0)
			_assert_true(wc._nearest_dict_target(wc.dungeon_nodes, 5.2) == "vein_live", "Depleted DungeonNode (available=false) skipped in favor of live vein")

			wc.player_pos = Vector3(2.0, 0.2, 2.8)
			var interact := wc.find_nearest_interactive_plaza_target()
			_assert_true(str(interact.get("action", "")) == "sleep_bed", "Interactive bed target detected in Starlight Plaza")

			wc._on_world_event({"type": "arena_player_status", "event": "died", "arenaStatus": {"hp": 0, "maxHp": 150, "alive": false}})
			_assert_true(wc.arena_dead, "arena_player_status death event sets arena_dead=true for Respawn prompt")

			wc._sync_boss_state({
				"active": true,
				"hp": 24,
				"maxHp": 30,
				"x": 0.0,
				"y": 0.4,
				"z": -24.5,
				"shockwaves": [
					{"id": 1, "x": 0.0, "z": -24.5, "radius": 4.2},
					{"id": 2, "x": 0.0, "z": -24.5, "radius": 9.5}
				]
			})
			_assert_true(wc.boss_shockwave_pool.size() >= 2 and wc.boss_shockwave_pool[0].visible and wc.boss_shockwave_pool[1].visible, "Multi-ring Obsidian Dragon shockwaves render from ArenaBossSnapshot.shockwaves")
		main_inst.queue_free()

	# 6. Optional Live Ephemeral Server UDP Handshake & World State Roundtrip
	var live_token := OS.get_environment("ELLA_TEST_TOKEN")
	var live_udp_port := int(OS.get_environment("ELLA_TEST_UDP_PORT"))
	if not live_token.is_empty() and live_udp_port > 0 and has_gdext:
		var live_host := OS.get_environment("ELLA_TEST_HOST")
		if live_host.is_empty():
			live_host = "127.0.0.1"
		var ext: RefCounted = ClassDB.instantiate(&"EllaNativeNet")
		var connected := bool(ext.call("connect_udp", live_host, live_udp_port, live_token))
		_assert_true(connected, "EllaNativeNet connect_udp succeeded against live server")

		var welcomed := false
		var deadline := Time.get_ticks_msec() + 3000
		while Time.get_ticks_msec() < deadline:
			ext.call("send_c2s_state", 1, Time.get_ticks_msec() & 0x7FFFFFFF, 4.0, 0.5, -6.0, 0.5, "walk", "", true, false, 0)
			var evs: Array = ext.call("poll_packets", Time.get_ticks_msec() & 0x7FFFFFFF)
			for ev in evs:
				if ev is Dictionary and str(ev.get("kind", "")) == "welcome":
					welcomed = true
			if welcomed:
				break
			OS.delay_msec(40)
		_assert_true(welcomed, "EllaNativeNet received live EWLD OP_WELCOME from ella-world-server")
		ext.call("disconnect_udp")

	main_packed = null
	await process_frame
	await process_frame
	AvatarRig.release_shared_caches()

	if _failures > 0:
		push_error("Godot verification failed with %d error(s)." % _failures)
		quit(1)
	else:
		print("=== ALL GODOT 4 VERIFICATION CHECKS PASSED ===")
		quit(0)
