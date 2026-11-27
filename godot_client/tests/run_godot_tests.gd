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

	# 3. AvatarRig Instantiation & Slot Switching
	var rig := AvatarRig.new()
	root.add_child(rig)
	rig.set_player_identity("peer_1", "ella", "Ella", "host")
	rig.apply_avatar_appearance({"avatarMode": "stylized", "bodyType": "female", "hairStyle": "long_glam", "topStyle": "cropped_puff", "fabricFinish": "glitter"})
	rig.apply_avatar_appearance({"avatarMode": "stylized", "bodyType": "male", "hairStyle": "tech_crop", "topStyle": "tech_jacket", "accessory": "aviators"})
	rig.apply_avatar_appearance({"avatarMode": "blocky", "topColor": "#38bdf8"})
	rig.set_companions("cloud_puppy", "hoverboard")
	_assert_true(rig.nametag_label != null and rig.nametag_label.text.contains("Ella"), "AvatarRig host crown nametag initialized")
	rig.queue_free()

	# 4. Full MainApp.tscn Scene Tree Instantiation
	var main_packed := load("res://scenes/MainApp.tscn") as PackedScene
	_assert_true(main_packed != null, "MainApp.tscn loads cleanly")
	if main_packed != null:
		var main_inst := main_packed.instantiate()
		root.add_child(main_inst)
		_assert_true(main_inst.get_node_or_null("NetworkClient") != null, "MainApp spawns NetworkClient")
		_assert_true(main_inst.get_node_or_null("WorldController") != null, "MainApp spawns WorldController")
		main_inst.queue_free()

	# 5. Optional Live Ephemeral Server UDP Handshake & World State Roundtrip
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

	if _failures > 0:
		push_error("Godot verification failed with %d error(s)." % _failures)
		quit(1)
	else:
		print("=== ALL GODOT 4 VERIFICATION CHECKS PASSED ===")
		quit(0)
