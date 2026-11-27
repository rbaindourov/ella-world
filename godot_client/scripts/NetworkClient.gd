class_name NetworkClient
extends Node
## Cross-Play Network Client (`NetworkClient.gd`)
## Speaks:
##   1. HTTPS REST API (`/api/ella-world/*` including Invite-Only & Multi-Playground endpoints)
##   2. Persistent WSS Spine (`/api/ella-world/ws?token=...`)
##   3. Native `EWLD` v1 Binary UDP (`:44445/udp`) via `EllaNativeNet` GDExtension (`rust_opus`)
##      with automatic pure-GDScript `PacketPeerUDP` fallback + 48 kHz RFC 6716 Opus Voice bridge.

signal auth_succeeded(session_data: Dictionary)
signal auth_failed(error_message: String)
signal welcome_received(snapshot: Dictionary)
signal world_event_received(event_data: Dictionary)
signal remote_states_received(players: Array, rtt_ms: float)
signal playgrounds_updated(playgrounds_data: Dictionary)
signal connection_status_changed(transport_label: String, rtt_ms: float)
signal session_revoked(reason: String)

const PROTOCOL_VERSION: int = 1

const OP_HELLO: int = 0x01
const OP_WELCOME: int = 0x02
const OP_C2S_STATE: int = 0x10
const OP_S2C_WORLD_STATE: int = 0x11
const OP_C2S_PING: int = 0x12
const OP_S2C_PONG: int = 0x13
const OP_VOICE_OPUS: int = 0x20
const OP_EVENT_JSON: int = 0x30
const OP_EVENT_ACK: int = 0x31

const SETTINGS_PATH := "user://ella_world_client.cfg"

var server_base_url: String = "https://robertscreations.com:4444"
var native_udp_port: int = 44445
var auth_token: String = ""
var admin_token: String = ""
var machine_seal: String = ""
var current_world_slug: String = "ella"
var current_world_name: String = "Ella's World"
var self_id: String = ""
var username: String = ""
var display_name: String = ""
var user_role: String = "friend"
var client_id: String = ""

var ws_peer: WebSocketPeer = null
var ws_connected: bool = false
var ws_reconnect_cooldown: float = 0.0

var native_ext: RefCounted = null
var fallback_udp: PacketPeerUDP = null
var udp_welcomed: bool = false
var udp_hello_retry_timer: float = 0.0
var ping_timer: float = 0.0
var c2s_seq: int = 0
var last_rtt_ms: float = 0.0

var _seen_state_seqs: Dictionary = {}
var _seen_state_seq_order: Array[int] = []
var _seen_chat_ids: Dictionary = {}

var ptt_active: bool = false
var voice_muted: bool = true
var mic_player: AudioStreamPlayer = null
var mic_capture_effect: AudioEffectCapture = null
var voice_out_player: AudioStreamPlayer = null
var voice_out_playback: AudioStreamGeneratorPlayback = null

func _ready() -> void:
	_load_saved_settings()
	if client_id.is_empty():
		client_id = "godot_%08x%08x" % [randi(), Time.get_ticks_usec() & 0xFFFFFFFF]
		_save_settings()
	if ClassDB.class_exists(&"EllaNativeNet"):
		native_ext = ClassDB.instantiate(&"EllaNativeNet")
	_setup_voice_audio_nodes()

func _setup_voice_audio_nodes() -> void:
	voice_out_player = AudioStreamPlayer.new()
	var gen := AudioStreamGenerator.new()
	gen.mix_rate = 48000.0
	gen.buffer_length = 0.35
	voice_out_player.stream = gen
	voice_out_player.bus = &"Master"
	add_child(voice_out_player)
	voice_out_player.play()
	voice_out_playback = voice_out_player.get_stream_playback() as AudioStreamGeneratorPlayback

	var mic_bus_idx := AudioServer.get_bus_index(&"MicCapture")
	if mic_bus_idx >= 0 and AudioServer.get_bus_effect_count(mic_bus_idx) > 0:
		mic_capture_effect = AudioServer.get_bus_effect(mic_bus_idx, 0) as AudioEffectCapture

func set_ptt_speaking(active: bool) -> void:
	if ptt_active == active:
		return
	ptt_active = active
	voice_muted = not active
	if active:
		if OS.get_name() == "Android":
			OS.request_permission("android.permission.RECORD_AUDIO")
		if mic_player == null and AudioServer.get_bus_index(&"MicCapture") >= 0:
			mic_player = AudioStreamPlayer.new()
			mic_player.stream = AudioStreamMicrophone.new()
			mic_player.bus = &"MicCapture"
			add_child(mic_player)
		if mic_player != null and not mic_player.playing:
			if mic_capture_effect != null:
				mic_capture_effect.clear_buffer()
			mic_player.play()
	else:
		if mic_player != null and mic_player.playing:
			mic_player.stop()

func _load_saved_settings() -> void:
	var cfg := ConfigFile.new()
	if cfg.load(SETTINGS_PATH) == OK:
		server_base_url = str(cfg.get_value("net", "server_base_url", server_base_url))
		auth_token = str(cfg.get_value("auth", "token", ""))
		machine_seal = str(cfg.get_value("auth", "machine_seal", ""))
		current_world_slug = str(cfg.get_value("auth", "world_slug", "ella"))
		username = str(cfg.get_value("auth", "username", ""))
		client_id = str(cfg.get_value("auth", "client_id", ""))

func _save_settings() -> void:
	var cfg := ConfigFile.new()
	cfg.set_value("net", "server_base_url", server_base_url)
	cfg.set_value("auth", "token", auth_token)
	cfg.set_value("auth", "machine_seal", machine_seal)
	cfg.set_value("auth", "world_slug", current_world_slug)
	cfg.set_value("auth", "username", username)
	cfg.set_value("auth", "client_id", client_id)
	cfg.save(SETTINGS_PATH)

func set_server_url(raw_url: String) -> void:
	var trimmed := raw_url.strip_edges().trim_suffix("/")
	if not trimmed.is_empty():
		server_base_url = trimmed
		_save_settings()

func _parse_host_and_tls() -> Dictionary:
	var clean := server_base_url.strip_edges().trim_suffix("/")
	var is_tls := clean.begins_with("https://")
	var rest := clean.trim_prefix("https://").trim_prefix("http://")
	var slash_idx := rest.find("/")
	if slash_idx >= 0:
		rest = rest.substr(0, slash_idx)
	var host := rest
	var port := 443 if is_tls else 80
	var colon_idx := rest.rfind(":")
	if colon_idx > 0:
		host = rest.substr(0, colon_idx)
		port = int(rest.substr(colon_idx + 1))
	return {"host": host, "port": port, "tls": is_tls}

func _tls_options_for_host(host: String) -> TLSOptions:
	var lower := host.to_lower()
	if lower == "127.0.0.1" or lower == "localhost" or lower.begins_with("192.168.") or lower.begins_with("10."):
		return TLSOptions.client_unsafe()
	return TLSOptions.client()

func request_json(path: String, method: int = HTTPClient.METHOD_GET, body_dict: Variant = null, bearer_token: String = "", callback: Callable = Callable()) -> void:
	var req := HTTPRequest.new()
	req.timeout = 12.0
	var info := _parse_host_and_tls()
	if bool(info["tls"]):
		req.set_tls_options(_tls_options_for_host(str(info["host"])))
	add_child(req)

	var headers := PackedStringArray(["Accept: application/json"])
	var payload := ""
	if body_dict != null:
		headers.append("Content-Type: application/json")
		payload = JSON.stringify(body_dict)
	if not bearer_token.is_empty():
		headers.append("Authorization: Bearer " + bearer_token)

	var full_url := server_base_url.trim_suffix("/") + path
	req.request_completed.connect(func(_result: int, response_code: int, _resp_headers: PackedStringArray, body: PackedByteArray) -> void:
		req.queue_free()
		var text := body.get_string_from_utf8()
		var parsed: Variant = JSON.parse_string(text) if not text.is_empty() else {}
		var data: Dictionary = parsed if parsed is Dictionary else {"raw": text}
		if data.has("machineSeal") and not str(data["machineSeal"]).is_empty():
			machine_seal = str(data["machineSeal"])
			_save_settings()
		if callback.is_valid():
			callback.call(response_code, data)
	)
	var err := req.request(full_url, headers, method, payload)
	if err != OK:
		req.queue_free()
		if callback.is_valid():
			callback.call(0, {"ok": false, "error": "Network request failed to start (%d)." % err})

func build_machine_auth_fields() -> Dictionary:
	var fields := {
		"clientId": client_id,
		"deviceTelemetry": _build_device_telemetry()
	}
	if not machine_seal.is_empty():
		fields["machineSeal"] = machine_seal
	return fields

func _with_machine_fields(base: Dictionary) -> Dictionary:
	var merged := base.duplicate(true)
	var mf := build_machine_auth_fields()
	for k in mf.keys():
		merged[k] = mf[k]
	return merged

func login_with_password(uname: String, pw: String, world_slug: String = "") -> void:
	var body := _with_machine_fields({
		"username": uname.strip_edges(),
		"password": pw
	})
	if not world_slug.strip_edges().is_empty():
		body["worldSlug"] = world_slug.strip_edges().to_lower()
	request_json("/api/ella-world/login", HTTPClient.METHOD_POST, body, "", _on_auth_http_response)

func resume_saved_session() -> void:
	if auth_token.is_empty():
		auth_failed.emit("No saved session token.")
		return
	var body := _with_machine_fields({
		"token": auth_token
	})
	request_json("/api/ella-world/resume", HTTPClient.METHOD_POST, body, "", _on_auth_http_response)

func redeem_friend_invite(code: String, uname: String, dname: String, pw: String) -> void:
	var body := _with_machine_fields({
		"code": code.strip_edges().to_upper(),
		"username": uname.strip_edges(),
		"displayName": dname.strip_edges(),
		"password": pw
	})
	request_json("/api/ella-world/invites/redeem", HTTPClient.METHOD_POST, body, "", _on_auth_http_response)

func create_family_playground(code: String, world_name: String, world_slug: String, admin_pw: String, host_uname: String, host_dname: String, host_pw: String) -> void:
	var body := _with_machine_fields({
		"code": code.strip_edges().to_upper(),
		"worldName": world_name.strip_edges(),
		"worldSlug": world_slug.strip_edges().to_lower(),
		"adminPassword": admin_pw,
		"hostUsername": host_uname.strip_edges(),
		"hostDisplayName": host_dname.strip_edges(),
		"hostPassword": host_pw
	})
	request_json("/api/ella-world/invites/redeem-family", HTTPClient.METHOD_POST, body, "", _on_auth_http_response)

func switch_playground(target_slug: String) -> void:
	if auth_token.is_empty():
		return
	var body := _with_machine_fields({
		"token": auth_token,
		"worldSlug": target_slug.strip_edges().to_lower()
	})
	request_json("/api/ella-world/playgrounds/switch", HTTPClient.METHOD_POST, body, auth_token, _on_auth_http_response)

func redeem_additional_invite(code: String, cb: Callable) -> void:
	var body := _with_machine_fields({
		"token": auth_token,
		"code": code.strip_edges().to_upper()
	})
	request_json("/api/ella-world/invites/join-world", HTTPClient.METHOD_POST, body, auth_token, func(status: int, data: Dictionary) -> void:
		if status >= 200 and status < 300 and data.has("token"):
			_apply_authenticated_session(data)
			auth_succeeded.emit(data)
		if cb.is_valid():
			cb.call(status, data)
	)

func refresh_my_playgrounds() -> void:
	if auth_token.is_empty():
		return
	request_json("/api/ella-world/playgrounds/mine", HTTPClient.METHOD_GET, null, auth_token, func(status: int, data: Dictionary) -> void:
		if status >= 200 and status < 300:
			playgrounds_updated.emit(data)
	)

func logout_cleanly() -> void:
	var old_tok := auth_token
	disconnect_realtime()
	auth_token = ""
	admin_token = ""
	_save_settings()
	if not old_tok.is_empty():
		request_json("/api/ella-world/logout", HTTPClient.METHOD_POST, {"token": old_tok})

func _on_auth_http_response(status: int, data: Dictionary) -> void:
	if status >= 200 and status < 300 and data.has("token"):
		_apply_authenticated_session(data)
		auth_succeeded.emit(data)
	else:
		var msg := str(data.get("error", "Sign-in failed (HTTP %d)." % status))
		auth_failed.emit(msg)

func _apply_authenticated_session(data: Dictionary) -> void:
	disconnect_realtime()
	auth_token = str(data.get("token", ""))
	if data.has("machineSeal") and not str(data["machineSeal"]).is_empty():
		machine_seal = str(data["machineSeal"])
	# Reset per-playground admin token unless newly minted by /invites/redeem-family
	admin_token = str(data.get("adminToken", "")) if data.has("adminToken") else ""

	var wm: Dictionary = data.get("worldMeta", {}) if data.get("worldMeta") is Dictionary else {}
	current_world_slug = str(data.get("worldSlug", wm.get("worldSlug", current_world_slug)))
	current_world_name = str(data.get("worldName", wm.get("worldName", current_world_name)))
	if data.has("nativeUdpPort"):
		native_udp_port = int(data["nativeUdpPort"])
	elif wm.has("nativeUdpPort"):
		native_udp_port = int(wm["nativeUdpPort"])

	var self_obj: Dictionary = data.get("self", {}) if data.get("self") is Dictionary else {}
	self_id = str(data.get("selfId", self_obj.get("id", "")))
	var user_obj: Dictionary = self_obj.get("user", data.get("user", {})) if self_obj.get("user", data.get("user", {})) is Dictionary else {}
	username = str(user_obj.get("username", username))
	display_name = str(user_obj.get("displayName", username))
	user_role = str(user_obj.get("role", "friend"))
	_save_settings()

	connect_realtime_transports()
	if data.has("players") or data.has("blocks"):
		welcome_received.emit(data)
	if data.has("myPlaygrounds"):
		playgrounds_updated.emit(data)
	else:
		refresh_my_playgrounds()

func connect_realtime_transports() -> void:
	if auth_token.is_empty():
		return
	_connect_ws_spine()
	_connect_native_udp()

func disconnect_realtime() -> void:
	set_ptt_speaking(false)
	_seen_state_seqs.clear()
	_seen_state_seq_order.clear()
	_seen_chat_ids.clear()
	if ws_peer != null:
		ws_peer.close()
		ws_peer = null
	ws_connected = false
	udp_welcomed = false
	if native_ext != null:
		native_ext.call("disconnect_udp")
	if fallback_udp != null:
		fallback_udp.close()
		fallback_udp = null

func _connect_ws_spine() -> void:
	if ws_peer != null:
		ws_peer.close()
	ws_peer = WebSocketPeer.new()
	ws_peer.inbound_buffer_size = 262144
	ws_peer.outbound_buffer_size = 131072
	var info := _parse_host_and_tls()
	var scheme := "wss" if bool(info["tls"]) else "ws"
	var ws_url := "%s://%s:%d/api/ella-world/ws?token=%s" % [
		scheme,
		str(info["host"]),
		int(info["port"]),
		auth_token.uri_encode()
	]
	var tls_opts: TLSOptions = _tls_options_for_host(str(info["host"])) if bool(info["tls"]) else null
	ws_peer.connect_to_url(ws_url, tls_opts)
	ws_connected = false

func _connect_native_udp() -> void:
	var info := _parse_host_and_tls()
	var host := str(info["host"])
	udp_welcomed = false
	udp_hello_retry_timer = 0.6

	if native_ext != null:
		var ok := bool(native_ext.call("connect_udp", host, native_udp_port, auth_token))
		if ok:
			return

	fallback_udp = PacketPeerUDP.new()
	if fallback_udp.connect_to_host(host, native_udp_port) == OK:
		fallback_udp.put_packet(encode_ewld_hello(auth_token))

func _process(delta: float) -> void:
	if auth_token.is_empty():
		return
	var now_ms := Time.get_ticks_msec() & 0x7FFFFFFF
	_poll_ws_spine()
	_poll_udp_fastlane(delta, now_ms)
	_pump_ptt_microphone()

func _pump_ptt_microphone() -> void:
	if not ptt_active or mic_capture_effect == null or native_ext == null or not udp_welcomed:
		return
	var avail := mic_capture_effect.get_frames_available()
	if avail >= 480:
		var frames := mic_capture_effect.get_buffer(min(avail, 3840))
		native_ext.call("push_mic_stereo_frames", frames)

func _poll_ws_spine() -> void:
	if ws_peer == null:
		ws_reconnect_cooldown -= get_process_delta_time()
		if ws_reconnect_cooldown <= 0.0:
			ws_reconnect_cooldown = 3.0
			_connect_ws_spine()
		return

	ws_peer.poll()
	var st := ws_peer.get_ready_state()
	if st == WebSocketPeer.STATE_OPEN:
		if not ws_connected:
			ws_connected = true
			if udp_welcomed:
				_send_ws_json({"type": "udp_fastlane", "active": true})
			_emit_transport_status()
		while ws_peer.get_available_packet_count() > 0:
			var pkt := ws_peer.get_packet()
			if ws_peer.was_string_packet():
				var txt := pkt.get_string_from_utf8()
				var parsed: Variant = JSON.parse_string(txt)
				if parsed is Dictionary:
					_dispatch_incoming_json(parsed, false)
			else:
				var decoded := decode_s2c_world_state_bytes(pkt, Time.get_ticks_msec() & 0x7FFFFFFF)
				if not decoded.is_empty():
					var rtt := float(decoded.get("rttMs", last_rtt_ms))
					if rtt > 0.0:
						last_rtt_ms = rtt
					remote_states_received.emit(decoded.get("players", []), last_rtt_ms)
	elif st == WebSocketPeer.STATE_CLOSED:
		ws_connected = false
		ws_peer = null
		ws_reconnect_cooldown = 2.0
		_emit_transport_status()

func _poll_udp_fastlane(delta: float, now_ms: int) -> void:
	if native_ext != null:
		var packets: Array = native_ext.call("poll_packets", now_ms)
		for item in packets:
			if not (item is Dictionary):
				continue
			var kind := str(item.get("kind", ""))
			match kind:
				"welcome":
					if not udp_welcomed:
						udp_welcomed = true
						if ws_connected:
							_send_ws_json({"type": "udp_fastlane", "active": true})
						_emit_transport_status()
				"world_state":
					var rtt := float(item.get("rttMs", last_rtt_ms))
					if rtt > 0.0:
						last_rtt_ms = rtt
					remote_states_received.emit(item.get("players", []), last_rtt_ms)
				"pong":
					last_rtt_ms = float(item.get("rttMs", last_rtt_ms))
					_emit_transport_status()
				"event_json":
					if not ws_connected:
						var parsed: Variant = JSON.parse_string(str(item.get("json", "")))
						if parsed is Dictionary:
							_dispatch_incoming_json(parsed, true)
				"voice_pcm":
					var frames: PackedVector2Array = item.get("frames", PackedVector2Array())
					if voice_out_playback != null and not frames.is_empty():
						if voice_out_playback.get_frames_available() >= frames.size():
							voice_out_playback.push_buffer(frames)
		if not udp_welcomed:
			udp_hello_retry_timer -= delta
			if udp_hello_retry_timer <= 0.0:
				udp_hello_retry_timer = 1.5
				native_ext.call("resend_hello")
	elif fallback_udp != null:
		while fallback_udp.get_available_packet_count() > 0:
			var raw := fallback_udp.get_packet()
			var res := decode_ewld_packet(raw, now_ms)
			var kind := str(res.get("kind", ""))
			if kind == "welcome":
				if not udp_welcomed:
					udp_welcomed = true
					if ws_connected:
						_send_ws_json({"type": "udp_fastlane", "active": true})
					_emit_transport_status()
			elif kind == "world_state":
				var rtt := float(res.get("rttMs", last_rtt_ms))
				if rtt > 0.0:
					last_rtt_ms = rtt
				remote_states_received.emit(res.get("players", []), last_rtt_ms)
			elif kind == "event_json" and not ws_connected:
				var parsed: Variant = JSON.parse_string(str(res.get("json", "")))
				if parsed is Dictionary:
					_dispatch_incoming_json(parsed, true)
		if not udp_welcomed:
			udp_hello_retry_timer -= delta
			if udp_hello_retry_timer <= 0.0:
				udp_hello_retry_timer = 1.5
				fallback_udp.put_packet(encode_ewld_hello(auth_token))

	ping_timer += delta
	if ping_timer >= 3.0:
		ping_timer = 0.0
		if udp_welcomed and native_ext != null:
			native_ext.call("send_ping", now_ms)
		_emit_transport_status()

func send_player_state(pos: Vector3, yaw: float, anim: String, emote: String, zone: int) -> void:
	if auth_token.is_empty():
		return
	c2s_seq += 1
	var now_ms := Time.get_ticks_msec() & 0x7FFFFFFF
	if udp_welcomed:
		if native_ext != null:
			native_ext.call("send_c2s_state", c2s_seq, now_ms, pos.x, pos.y, pos.z, yaw, anim, emote, voice_muted, ptt_active, zone)
			return
		elif fallback_udp != null:
			fallback_udp.put_packet(encode_ewld_c2s_state(c2s_seq, now_ms, pos, yaw, anim, emote, voice_muted, ptt_active, zone))
			return
	if ws_connected and ws_peer != null:
		ws_peer.put_packet(encode_c2s_state_raw_28(c2s_seq, now_ms, pos, yaw, anim, emote, voice_muted, ptt_active, zone))

func send_game_command(cmd: Dictionary) -> void:
	if auth_token.is_empty():
		return
	var json_str := JSON.stringify(cmd)
	if ws_connected and ws_peer != null:
		ws_peer.send_text(json_str)
	elif udp_welcomed and native_ext != null:
		native_ext.call("send_event_json", json_str)

func _send_ws_json(payload: Dictionary) -> void:
	if ws_connected and ws_peer != null:
		ws_peer.send_text(JSON.stringify(payload))

func _record_seen_state_seq(sseq: int) -> void:
	if sseq <= 0 or _seen_state_seqs.has(sseq):
		return
	_seen_state_seqs[sseq] = true
	_seen_state_seq_order.append(sseq)
	while _seen_state_seq_order.size() > 256:
		var oldest: int = _seen_state_seq_order.pop_front()
		_seen_state_seqs.erase(oldest)

func should_accept_world_event(msg: Dictionary) -> bool:
	var msg_type := str(msg.get("type", ""))
	var sseq := int(msg.get("stateSeq", 0)) if msg.has("stateSeq") else 0
	if msg_type == "chat":
		var chat_obj: Variant = msg.get("chat", msg.get("entry", null))
		if chat_obj is Dictionary:
			var cid := str(chat_obj.get("id", ""))
			if not cid.is_empty():
				if _seen_chat_ids.has(cid):
					return false
				_seen_chat_ids[cid] = true
				if _seen_chat_ids.size() > 256:
					_seen_chat_ids.clear()
					_seen_chat_ids[cid] = true
				_record_seen_state_seq(sseq)
				return true
	if sseq > 0 and msg_type != "world_sync_digest" and msg_type != "welcome_snapshot" and msg_type != "pong" and msg_type != "cmd_ack":
		if _seen_state_seqs.has(sseq):
			return false
		_record_seen_state_seq(sseq)
	return true

func _dispatch_incoming_json(msg: Dictionary, _from_udp: bool = false) -> void:
	var msg_type := str(msg.get("type", ""))
	if msg_type == "welcome_snapshot":
		if msg.has("stateSeq"):
			_record_seen_state_seq(int(msg["stateSeq"]))
		if msg.has("self") and msg["self"] is Dictionary:
			self_id = str(msg["self"].get("id", self_id))
		if msg.get("chatHistory") is Array:
			for c in msg["chatHistory"]:
				if c is Dictionary and c.has("id"):
					_seen_chat_ids[str(c["id"])] = true
		welcome_received.emit(msg)
		return
	if msg_type == "session_revoked":
		var reason := str(msg.get("reason", "Your session was signed out by a parent."))
		disconnect_realtime()
		session_revoked.emit(reason)
		return
	if not should_accept_world_event(msg):
		return
	world_event_received.emit(msg)

func _emit_transport_status() -> void:
	var mode := "Offline"
	if udp_welcomed and native_ext != null:
		mode = "EWLD UDP+Opus"
	elif udp_welcomed:
		mode = "EWLD UDP"
	elif ws_connected:
		mode = "WSS Spine"
	connection_status_changed.emit(mode, last_rtt_ms)

func _build_device_telemetry() -> Dictionary:
	var scr := DisplayServer.screen_get_size()
	var scale_x100 := int(round(DisplayServer.screen_get_scale() * 100.0))
	return {
		"gpuVendor": RenderingServer.get_video_adapter_vendor(),
		"gpuRenderer": RenderingServer.get_video_adapter_name(),
		"glCaps": OS.get_model_name(),
		"screenW": maxi(int(scr.x), 1280),
		"screenH": maxi(int(scr.y), 720),
		"colorDepth": 24,
		"pixelRatioX100": maxi(scale_x100, 100),
		"cpuCores": maxi(OS.get_processor_count(), 1),
		"maxTouchPoints": 10 if DisplayServer.is_touchscreen_available() else 0,
		"osFamily": OS.get_name(),
		"timezone": Time.get_time_zone_from_system().get("name", "UTC"),
		"language": OS.get_locale()
	}

# ============================================================================
# Static Binary `EWLD` v1 Codec Helpers (Pure-GDScript Fallback + Unit Tests)
# ============================================================================

static func anim_to_code(anim: String) -> int:
	match anim:
		"walk": return 1
		"run": return 2
		"jump": return 3
		"hover": return 4
		_: return 0

static func code_to_anim(code: int) -> String:
	match code:
		1: return "walk"
		2: return "run"
		3: return "jump"
		4: return "hover"
		_: return "idle"

static func emote_to_code(emote: String) -> int:
	match emote:
		"pose": return 1
		"peace": return 2
		"dance": return 3
		"spin": return 4
		"wave": return 5
		"cheer": return 6
		"flip": return 7
		"sit": return 8
		"sleep": return 9
		_: return 0

static func code_to_emote(code: int) -> String:
	match code:
		1: return "pose"
		2: return "peace"
		3: return "dance"
		4: return "spin"
		5: return "wave"
		6: return "cheer"
		7: return "flip"
		8: return "sit"
		9: return "sleep"
		_: return ""

static func wrap_ewld(pkt_type: int, body: PackedByteArray) -> PackedByteArray:
	var out := PackedByteArray()
	out.append(0x45)
	out.append(0x57)
	out.append(0x4C)
	out.append(0x44)
	out.append(PROTOCOL_VERSION)
	out.append(pkt_type & 0xFF)
	out.append_array(body)
	return out

static func encode_ewld_hello(token: String) -> PackedByteArray:
	var tb := token.to_utf8_buffer()
	var sp := StreamPeerBuffer.new()
	sp.big_endian = false
	sp.put_u16(tb.size())
	sp.put_data(tb)
	return wrap_ewld(OP_HELLO, sp.data_array)

static func encode_c2s_state_raw_28(seq: int, client_ms: int, pos: Vector3, yaw: float, anim: String, emote: String, muted: bool, speaking: bool, zone: int) -> PackedByteArray:
	var sp := StreamPeerBuffer.new()
	sp.big_endian = false
	sp.put_u8(OP_C2S_STATE)
	sp.put_u32(seq & 0xFFFFFFFF)
	sp.put_u32(client_ms & 0xFFFFFFFF)
	sp.put_float(pos.x)
	sp.put_float(pos.y)
	sp.put_float(pos.z)
	sp.put_float(yaw)
	sp.put_u8(anim_to_code(anim))
	sp.put_u8(emote_to_code(emote))
	var flags := ((zone & 0x3F) << 2)
	if muted:
		flags |= 1
	if speaking:
		flags |= 2
	sp.put_u8(flags)
	return sp.data_array

static func encode_ewld_c2s_state(seq: int, client_ms: int, pos: Vector3, yaw: float, anim: String, emote: String, muted: bool, speaking: bool, zone: int) -> PackedByteArray:
	return wrap_ewld(OP_C2S_STATE, encode_c2s_state_raw_28(seq, client_ms, pos, yaw, anim, emote, muted, speaking, zone))

static func decode_ewld_packet(raw: PackedByteArray, now_ms: int) -> Dictionary:
	if raw.size() < 6:
		return {}
	if raw[0] != 0x45 or raw[1] != 0x57 or raw[2] != 0x4C or raw[3] != 0x44 or raw[4] != PROTOCOL_VERSION:
		return {}
	var pkt_type := int(raw[5])
	var body := raw.slice(6)
	match pkt_type:
		OP_WELCOME:
			if body.size() < 9:
				return {}
			var sp := StreamPeerBuffer.new()
			sp.big_endian = false
			sp.data_array = body
			var blocks_ver := sp.get_u64()
			var id_len := sp.get_u8()
			if body.size() < 9 + id_len:
				return {}
			var peer_id := body.slice(9, 9 + id_len).get_string_from_utf8()
			return {"kind": "welcome", "selfId": peer_id, "blocksVersion": blocks_ver}
		OP_S2C_WORLD_STATE:
			return decode_s2c_world_state_bytes(body, now_ms)
		OP_EVENT_JSON:
			if body.size() <= 4:
				return {}
			return {"kind": "event_json", "json": body.slice(4).get_string_from_utf8()}
		_:
			return {}

static func decode_s2c_world_state_bytes(body: PackedByteArray, now_ms: int) -> Dictionary:
	if body.size() < 11 or int(body[0]) != OP_S2C_WORLD_STATE:
		return {}
	var sp := StreamPeerBuffer.new()
	sp.big_endian = false
	sp.data_array = body
	sp.seek(1)
	var server_tick := sp.get_u32()
	var echo_ms := sp.get_u32()
	var count := sp.get_u16()
	var players: Array = []
	for _i in range(count):
		if sp.get_position() >= body.size():
			break
		var id_len := sp.get_u8()
		if sp.get_position() + id_len + 19 > body.size():
			break
		var pid := body.slice(sp.get_position(), sp.get_position() + id_len).get_string_from_utf8()
		sp.seek(sp.get_position() + id_len)
		var px := sp.get_float()
		var py := sp.get_float()
		var pz := sp.get_float()
		var pry := sp.get_float()
		var anim_c := sp.get_u8()
		var emote_c := sp.get_u8()
		var flags := sp.get_u8()
		players.append({
			"id": pid,
			"x": px,
			"y": py,
			"z": pz,
			"ry": pry,
			"anim": code_to_anim(anim_c),
			"emote": code_to_emote(emote_c),
			"muted": (flags & 1) != 0,
			"speaking": (flags & 2) != 0,
			"zone": (flags >> 2) & 0x3F
		})
	var rtt := 0.0
	if echo_ms > 0:
		var diff := int((now_ms - echo_ms) & 0x7FFFFFFF)
		if diff >= 0 and diff < 5000:
			rtt = float(diff)
	return {
		"kind": "world_state",
		"serverTick": server_tick,
		"rttMs": rtt,
		"players": players
	}
