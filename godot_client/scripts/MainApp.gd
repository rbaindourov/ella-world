extends Node
## Root Application Controller (`MainApp.gd`)
## Wires `NetworkClient`, `WorldController`, `MultiTouchController`, Invite-Only Onboarding,
## Clean Fullscreen Landscape HUD, Chat Drawer, Glam Wardrobe, Homes/Workshop, My Playgrounds Switcher,
## Kid Host Invite Drafter, and Per-Playground Parent Admin Panel.

var net: NetworkClient = null
var world: WorldController = null
var touch_layer: CanvasLayer = null
var touch_ctrl: MultiTouchController = null
var ui_layer: CanvasLayer = null

# Auth UI refs
var auth_overlay: Control = null
var server_url_edit: LineEdit = null
var auth_tabs: TabContainer = null
var login_user_edit: LineEdit = null
var login_pass_edit: LineEdit = null
var login_world_edit: LineEdit = null
var star_code_edit: LineEdit = null
var star_user_edit: LineEdit = null
var star_dname_edit: LineEdit = null
var star_pass_edit: LineEdit = null
var fam_code_edit: LineEdit = null
var fam_wname_edit: LineEdit = null
var fam_wslug_edit: LineEdit = null
var fam_admin_pw_edit: LineEdit = null
var fam_host_user_edit: LineEdit = null
var fam_host_pass_edit: LineEdit = null
var auth_status_label: Label = null

# Gameplay HUD refs
var hud_root: Control = null
var playground_btn: Button = null
var stats_label: Label = null
var net_badge_label: Label = null
var hint_label: Label = null
var toast_label: Label = null
var toast_timer: float = 0.0

# Collapsible Chat & World Menu Drawer refs
var chat_panel: PanelContainer = null
var chat_log_rich: RichTextLabel = null
var chat_input_edit: LineEdit = null

var menu_backdrop: ColorRect = null
var menu_panel: PanelContainer = null
var menu_tabs: TabContainer = null

var playgrounds_list_vbox: VBoxContainer = null
var extra_invite_edit: LineEdit = null
var homes_list_vbox: VBoxContainer = null
var admin_status_label: Label = null
var admin_invites_vbox: VBoxContainer = null
var admin_pw_edit: LineEdit = null

var emote_cycle := ["wave", "dance", "spin", "cheer", "pose", "peace", "flip"]
var emote_idx: int = 0

func _ready() -> void:
	net = NetworkClient.new()
	net.name = "NetworkClient"
	add_child(net)

	touch_layer = CanvasLayer.new()
	touch_layer.layer = 2
	add_child(touch_layer)
	touch_ctrl = MultiTouchController.new()
	touch_ctrl.name = "MultiTouchController"
	touch_layer.add_child(touch_ctrl)

	world = WorldController.new()
	world.name = "WorldController"
	add_child(world)
	world.setup(net, touch_ctrl)

	ui_layer = CanvasLayer.new()
	ui_layer.layer = 5
	add_child(ui_layer)

	_build_auth_overlay()
	_build_hud_and_drawers()
	_wire_signals()
	_check_cmdline_invite_or_saved_session()

func _wire_signals() -> void:
	net.auth_succeeded.connect(_on_auth_succeeded)
	net.auth_failed.connect(_on_auth_failed)
	net.welcome_received.connect(_on_welcome_snapshot)
	net.world_event_received.connect(_on_world_event)
	net.playgrounds_updated.connect(_on_playgrounds_updated)
	net.connection_status_changed.connect(_on_net_status)
	net.session_revoked.connect(_on_session_revoked)
	world.context_hint_changed.connect(func(txt: String) -> void:
		if hint_label != null:
			hint_label.text = txt
			hint_label.visible = not txt.is_empty()
	)
	world.toast_requested.connect(show_toast)
	world.stats_updated.connect(_on_stats_updated)

func _check_cmdline_invite_or_saved_session() -> void:
	server_url_edit.text = net.server_base_url
	login_user_edit.text = net.username
	login_world_edit.text = net.current_world_slug

	for arg in OS.get_cmdline_args():
		if arg.contains("invite="):
			var idx := arg.find("invite=")
			var code := arg.substr(idx + 7).split("&")[0].strip_edges().to_upper()
			if code.begins_with("FAMILY-"):
				fam_code_edit.text = code
				auth_tabs.current_tab = 2
			elif not code.is_empty():
				star_code_edit.text = code
				auth_tabs.current_tab = 1
			return

	if not net.auth_token.is_empty() and not DisplayServer.get_name().contains("headless"):
		auth_status_label.text = "Reconnecting saved session..."
		net.resume_saved_session()

func _make_card_style(bg_hex: String = "#0f172a", alpha: float = 0.92, border_hex: String = "#38bdf8") -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	var c := Color.from_string(bg_hex, Color(0.06, 0.09, 0.16))
	c.a = alpha
	sb.bg_color = c
	sb.border_color = Color.from_string(border_hex, Color(0.22, 0.74, 0.97, 0.45))
	sb.border_color.a = 0.42
	sb.set_border_width_all(2)
	sb.set_corner_radius_all(16)
	sb.set_content_margin_all(14)
	return sb

func _make_btn(label_text: String, fill_hex: String = "#0284c7", text_hex: String = "#ffffff", min_sz: Vector2 = Vector2(110, 44)) -> Button:
	var b := Button.new()
	b.text = label_text
	b.custom_minimum_size = min_sz
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color.from_string(fill_hex, Color("#0284c7"))
	sb.set_corner_radius_all(12)
	sb.set_content_margin_all(8)
	b.add_theme_stylebox_override("normal", sb)
	var hover_sb := sb.duplicate() as StyleBoxFlat
	hover_sb.bg_color = sb.bg_color.lightened(0.12)
	b.add_theme_stylebox_override("hover", hover_sb)
	b.add_theme_stylebox_override("pressed", hover_sb)
	b.add_theme_color_override("font_color", Color.from_string(text_hex, Color.WHITE))
	b.add_theme_font_size_override("font_size", 16)
	return b

func _build_auth_overlay() -> void:
	auth_overlay = Control.new()
	auth_overlay.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	ui_layer.add_child(auth_overlay)

	var dimmer := ColorRect.new()
	dimmer.color = Color(0.035, 0.051, 0.086, 0.86)
	dimmer.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	auth_overlay.add_child(dimmer)

	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	auth_overlay.add_child(center)

	var card := PanelContainer.new()
	card.custom_minimum_size = Vector2(640, 480)
	card.add_theme_stylebox_override("panel", _make_card_style("#0f172a", 0.95, "#38bdf8"))
	center.add_child(card)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 10)
	card.add_child(vbox)

	var title := Label.new()
	title.text = "👑 Ella's World — Invite-Only Family 3D Playgrounds"
	title.add_theme_font_size_override("font_size", 22)
	title.add_theme_color_override("font_color", Color("#ffffff"))
	vbox.add_child(title)

	var srv_row := HBoxContainer.new()
	var srv_lbl := Label.new()
	srv_lbl.text = "Server:"
	srv_lbl.add_theme_color_override("font_color", Color("#cbd5e1"))
	server_url_edit = LineEdit.new()
	server_url_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	server_url_edit.placeholder_text = "https://robertscreations.com:4444"
	srv_row.add_child(srv_lbl)
	srv_row.add_child(server_url_edit)
	vbox.add_child(srv_row)

	auth_tabs = TabContainer.new()
	auth_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vbox.add_child(auth_tabs)

	# Tab 0: Sign In
	var tab_login := VBoxContainer.new()
	tab_login.name = "✨ Sign In"
	tab_login.add_theme_constant_override("separation", 8)
	login_user_edit = _add_labeled_input(tab_login, "Username", "e.g. ella")
	login_pass_edit = _add_labeled_input(tab_login, "Password", "Your secret password", true)
	login_world_edit = _add_labeled_input(tab_login, "Playground (optional)", "ella")
	var btn_signin := _make_btn("🚀 Enter Playground", "#38bdf8", "#0f172a", Vector2(220, 46))
	btn_signin.pressed.connect(func() -> void:
		net.set_server_url(server_url_edit.text)
		auth_status_label.text = "Signing in..."
		net.login_with_password(login_user_edit.text, login_pass_edit.text, login_world_edit.text)
	)
	tab_login.add_child(btn_signin)
	auth_tabs.add_child(tab_login)

	# Tab 1: Redeem STAR-... Friend Invite
	var tab_star := VBoxContainer.new()
	tab_star.name = "🎟️ Friend Invite"
	tab_star.add_theme_constant_override("separation", 6)
	star_code_edit = _add_labeled_input(tab_star, "Friend Invite Code", "STAR-XXXX-XXXX")
	star_user_edit = _add_labeled_input(tab_star, "Choose Username", "e.g. leo")
	star_dname_edit = _add_labeled_input(tab_star, "Display Name", "e.g. Leo")
	star_pass_edit = _add_labeled_input(tab_star, "Create Password", "4+ characters", true)
	var btn_redeem := _make_btn("🎟️ Redeem Friend Invite & Join!", "#34d399", "#0f172a", Vector2(260, 44))
	btn_redeem.pressed.connect(func() -> void:
		net.set_server_url(server_url_edit.text)
		auth_status_label.text = "Redeeming STAR invite..."
		net.redeem_friend_invite(star_code_edit.text, star_user_edit.text, star_dname_edit.text, star_pass_edit.text)
	)
	tab_star.add_child(btn_redeem)
	auth_tabs.add_child(tab_star)

	# Tab 2: Create Family Playground (`FAMILY-...`)
	var tab_fam := ScrollContainer.new()
	tab_fam.name = "🏡 Family Founder"
	var fam_vb := VBoxContainer.new()
	fam_vb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	fam_vb.add_theme_constant_override("separation", 6)
	fam_code_edit = _add_labeled_input(fam_vb, "Parent Founder Code", "FAMILY-XXXX-XXXX")
	fam_wname_edit = _add_labeled_input(fam_vb, "Playground Name", "e.g. Maya's World")
	fam_wslug_edit = _add_labeled_input(fam_vb, "Playground Slug", "e.g. maya-world")
	fam_admin_pw_edit = _add_labeled_input(fam_vb, "Parent Admin Password", "Parent master password", true)
	fam_host_user_edit = _add_labeled_input(fam_vb, "Kid Host Username", "e.g. maya")
	fam_host_pass_edit = _add_labeled_input(fam_vb, "Kid Host Password", "Kid login password", true)
	var btn_fam := _make_btn("🏡 Create Family Playground!", "#fbbf24", "#0f172a", Vector2(260, 44))
	btn_fam.pressed.connect(func() -> void:
		net.set_server_url(server_url_edit.text)
		auth_status_label.text = "Creating Playground..."
		net.create_family_playground(
			fam_code_edit.text,
			fam_wname_edit.text,
			fam_wslug_edit.text,
			fam_admin_pw_edit.text,
			fam_host_user_edit.text,
			fam_host_user_edit.text.capitalize(),
			fam_host_pass_edit.text
		)
	)
	fam_vb.add_child(btn_fam)
	tab_fam.add_child(fam_vb)
	auth_tabs.add_child(tab_fam)

	auth_status_label = Label.new()
	auth_status_label.text = "Invite-only family platform • Sign in or enter a STAR / FAMILY invite code."
	auth_status_label.add_theme_color_override("font_color", Color("#fbbf24"))
	auth_status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	vbox.add_child(auth_status_label)

func _add_labeled_input(parent: Container, label_txt: String, placeholder: String, secret: bool = false) -> LineEdit:
	var row := HBoxContainer.new()
	var lbl := Label.new()
	lbl.text = label_txt
	lbl.custom_minimum_size = Vector2(185, 0)
	lbl.add_theme_color_override("font_color", Color("#f1f5f9"))
	var ed := LineEdit.new()
	ed.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	ed.placeholder_text = placeholder
	ed.secret = secret
	row.add_child(lbl)
	row.add_child(ed)
	parent.add_child(row)
	return ed

func _build_hud_and_drawers() -> void:
	hud_root = Control.new()
	hud_root.visible = false
	hud_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hud_root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	ui_layer.add_child(hud_root)

	# Top Bar (Left: Playground Switcher + Stats | Right: Emote, Build, Chat, World Menu)
	var top_bar := HBoxContainer.new()
	top_bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	top_bar.offset_left = 14
	top_bar.offset_top = 12
	top_bar.offset_right = -14
	top_bar.offset_bottom = 60
	top_bar.add_theme_constant_override("separation", 8)
	hud_root.add_child(top_bar)

	playground_btn = _make_btn("🌟 Ella's World ▾", "#1e293b", "#38bdf8", Vector2(175, 42))
	playground_btn.pressed.connect(func() -> void: _open_menu_tab(0))
	top_bar.add_child(playground_btn)

	var stats_pill := PanelContainer.new()
	stats_pill.add_theme_stylebox_override("panel", _make_card_style("#0f172a", 0.86, "#38bdf8"))
	stats_label = Label.new()
	stats_label.text = "🪙 120 • 🌟 Plaza"
	stats_label.add_theme_color_override("font_color", Color("#f8fafc"))
	stats_pill.add_child(stats_label)
	top_bar.add_child(stats_pill)

	net_badge_label = Label.new()
	net_badge_label.text = "Connecting..."
	net_badge_label.add_theme_color_override("font_color", Color("#34d399"))
	top_bar.add_child(net_badge_label)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spacer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top_bar.add_child(spacer)

	var btn_emote := _make_btn("💃 Emote", "#4f46e5", "#ffffff", Vector2(96, 42))
	btn_emote.pressed.connect(func() -> void:
		var pick: String = emote_cycle[emote_idx % emote_cycle.size()]
		emote_idx += 1
		world.trigger_emote(pick)
	)
	top_bar.add_child(btn_emote)

	var btn_build := _make_btn("🧱 Build", "#059669", "#ffffff", Vector2(96, 42))
	btn_build.pressed.connect(func() -> void:
		world.build_mode = not world.build_mode
		show_toast("🧱 Build Mode " + ("ON" if world.build_mode else "OFF"))
	)
	top_bar.add_child(btn_build)

	var btn_chat := _make_btn("💬 Chat", "#0284c7", "#ffffff", Vector2(92, 42))
	btn_chat.pressed.connect(func() -> void:
		chat_panel.visible = not chat_panel.visible
	)
	top_bar.add_child(btn_chat)

	var btn_menu := _make_btn("☰ Menu", "#db2777", "#ffffff", Vector2(98, 42))
	btn_menu.pressed.connect(func() -> void: _open_menu_tab(1))
	top_bar.add_child(btn_menu)

	# Context Hint & Toast Banners
	hint_label = Label.new()
	hint_label.set_anchors_preset(Control.PRESET_CENTER_TOP)
	hint_label.offset_left = -260
	hint_label.offset_right = 260
	hint_label.offset_top = 68
	hint_label.offset_bottom = 102
	hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hint_label.add_theme_color_override("font_color", Color("#fef08a"))
	hint_label.add_theme_font_size_override("font_size", 17)
	hud_root.add_child(hint_label)

	toast_label = Label.new()
	toast_label.set_anchors_preset(Control.PRESET_CENTER_TOP)
	toast_label.offset_left = -280
	toast_label.offset_right = 280
	toast_label.offset_top = 104
	toast_label.offset_bottom = 138
	toast_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	toast_label.add_theme_color_override("font_color", Color("#38bdf8"))
	toast_label.visible = false
	hud_root.add_child(toast_label)

	# Bottom-Right Ergonomic Thumb-Arc Action Cluster (Jump, Action/Attack, Hoverboard, Hold PTT Mic)
	var arc_root := Control.new()
	arc_root.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	arc_root.offset_left = -310
	arc_root.offset_top = -250
	arc_root.offset_right = -18
	arc_root.offset_bottom = -18
	arc_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hud_root.add_child(arc_root)

	var jump_btn := _make_btn("⬆️\nJUMP", "#38bdf8", "#0f172a", Vector2(96, 96))
	jump_btn.position = Vector2(186, 126)
	jump_btn.button_down.connect(func() -> void: world.queue_jump())
	arc_root.add_child(jump_btn)

	var act_btn := _make_btn("⚡\nACTION", "#fbbf24", "#0f172a", Vector2(86, 86))
	act_btn.position = Vector2(84, 138)
	act_btn.pressed.connect(func() -> void: world.trigger_context_action())
	arc_root.add_child(act_btn)

	var board_btn := _make_btn("🛹 Board", "#4f46e5", "#ffffff", Vector2(98, 52))
	board_btn.position = Vector2(82, 72)
	board_btn.pressed.connect(func() -> void: world.toggle_equipped_hoverboard())
	arc_root.add_child(board_btn)

	var ptt_btn := _make_btn("🎙️ Hold Mic", "#059669", "#ffffff", Vector2(106, 52))
	ptt_btn.position = Vector2(186, 62)
	ptt_btn.button_down.connect(func() -> void:
		net.set_ptt_speaking(true)
		ptt_btn.text = "🎙️ LIVE!"
	)
	ptt_btn.button_up.connect(func() -> void:
		net.set_ptt_speaking(false)
		ptt_btn.text = "🎙️ Hold Mic"
	)
	arc_root.add_child(ptt_btn)

	# Collapsible Chat Overlay (Top-Left below status bar so left joystick stays unobstructed!)
	chat_panel = PanelContainer.new()
	chat_panel.visible = false
	chat_panel.position = Vector2(16, 68)
	chat_panel.size = Vector2(390, 230)
	chat_panel.add_theme_stylebox_override("panel", _make_card_style("#0f172a", 0.88, "#38bdf8"))
	hud_root.add_child(chat_panel)

	var chat_vb := VBoxContainer.new()
	chat_panel.add_child(chat_vb)
	chat_log_rich = RichTextLabel.new()
	chat_log_rich.size_flags_vertical = Control.SIZE_EXPAND_FILL
	chat_log_rich.bbcode_enabled = false
	chat_log_rich.scroll_following = true
	chat_vb.add_child(chat_log_rich)
	var chat_hb := HBoxContainer.new()
	chat_input_edit = LineEdit.new()
	chat_input_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	chat_input_edit.placeholder_text = "Say hi to friends..."
	var chat_send := _make_btn("Send", "#38bdf8", "#0f172a", Vector2(72, 38))
	var send_fn := func() -> void:
		var msg := chat_input_edit.text.strip_edges()
		if not msg.is_empty():
			net.send_game_command({"type": "chat", "text": msg})
			chat_input_edit.text = ""
	chat_send.pressed.connect(send_fn)
	chat_input_edit.text_submitted.connect(func(_t: String) -> void: send_fn.call())
	chat_hb.add_child(chat_input_edit)
	chat_hb.add_child(chat_send)
	chat_vb.add_child(chat_hb)

	_build_world_menu_modal()

func _build_world_menu_modal() -> void:
	menu_backdrop = ColorRect.new()
	menu_backdrop.visible = false
	menu_backdrop.color = Color(0.03, 0.05, 0.09, 0.72)
	menu_backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	hud_root.add_child(menu_backdrop)

	menu_panel = PanelContainer.new()
	menu_panel.visible = false
	menu_panel.set_anchors_preset(Control.PRESET_CENTER)
	menu_panel.offset_left = -430
	menu_panel.offset_right = 430
	menu_panel.offset_top = -255
	menu_panel.offset_bottom = 255
	menu_panel.add_theme_stylebox_override("panel", _make_card_style("#0f172a", 0.96, "#38bdf8"))
	hud_root.add_child(menu_panel)

	var root_vb := VBoxContainer.new()
	menu_panel.add_child(root_vb)

	var hdr := HBoxContainer.new()
	var htitle := Label.new()
	htitle.text = "✨ Ella's World Studio, Playgrounds & Safety Hub"
	htitle.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	htitle.add_theme_font_size_override("font_size", 20)
	var btn_close := _make_btn("✕ Close", "#334155", "#ffffff", Vector2(92, 38))
	btn_close.pressed.connect(_close_menu)
	hdr.add_child(htitle)
	hdr.add_child(btn_close)
	root_vb.add_child(hdr)

	menu_tabs = TabContainer.new()
	menu_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root_vb.add_child(menu_tabs)

	# Tab 0: 🌍 My Playgrounds Switcher
	var pg_tab := VBoxContainer.new()
	pg_tab.name = "🌍 Playgrounds"
	var pg_info := Label.new()
	pg_info.text = "Your 3D Avatar travels with you! Coins, Homes & Chat stay isolated per Playground."
	pg_info.add_theme_color_override("font_color", Color("#cbd5e1"))
	pg_tab.add_child(pg_info)
	var pg_scroll := ScrollContainer.new()
	pg_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	playgrounds_list_vbox = VBoxContainer.new()
	playgrounds_list_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pg_scroll.add_child(playgrounds_list_vbox)
	pg_tab.add_child(pg_scroll)
	var join_hb := HBoxContainer.new()
	extra_invite_edit = LineEdit.new()
	extra_invite_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	extra_invite_edit.placeholder_text = "Have another STAR-XXXX-XXXX code? Redeem here!"
	var btn_join_more := _make_btn("🎟️ Join Playground", "#34d399", "#0f172a", Vector2(170, 40))
	btn_join_more.pressed.connect(func() -> void:
		net.redeem_additional_invite(extra_invite_edit.text, func(st: int, d: Dictionary) -> void:
			if st >= 200 and st < 300:
				extra_invite_edit.text = ""
				show_toast("🎉 Joined Playground!")
				_close_menu()
			else:
				show_toast(str(d.get("error", "Invalid invite code.")))
		)
	)
	var btn_logout := _make_btn("🚪 Sign Out", "#be123c", "#ffffff", Vector2(120, 40))
	btn_logout.pressed.connect(func() -> void:
		_close_menu()
		net.logout_cleanly()
		_show_auth_screen("Signed out cleanly.")
	)
	join_hb.add_child(extra_invite_edit)
	join_hb.add_child(btn_join_more)
	join_hb.add_child(btn_logout)
	pg_tab.add_child(join_hb)
	menu_tabs.add_child(pg_tab)

	# Tab 1: 👗 Glam Closet & Runway
	var glam_tab := VBoxContainer.new()
	glam_tab.name = "👗 Glam & Runway"
	_populate_glam_tab(glam_tab)
	menu_tabs.add_child(glam_tab)

	# Tab 2: 🐾 Pets, Hoverboards & Sky
	var comp_tab := VBoxContainer.new()
	comp_tab.name = "🐾 Pets & Sky"
	_populate_companions_and_sky_tab(comp_tab)
	menu_tabs.add_child(comp_tab)

	# Tab 3: 🏡 Homes, Build & Workshop
	var home_tab := VBoxContainer.new()
	home_tab.name = "🏡 Homes & Craft"
	_populate_homes_and_craft_tab(home_tab)
	menu_tabs.add_child(home_tab)

	# Tab 4: 🎟️ Invites & Parent Admin
	var admin_tab := VBoxContainer.new()
	admin_tab.name = "🛡️ Invites & Parent"
	_populate_invites_and_admin_tab(admin_tab)
	menu_tabs.add_child(admin_tab)

func _populate_glam_tab(tab: VBoxContainer) -> void:
	var row1 := HBoxContainer.new()
	for preset in [
		["👑 Glam Starlight", {"avatarMode": "stylized", "bodyType": "female", "hairStyle": "long_glam", "topStyle": "cropped_puff", "topColor": "#f472b6", "bottomStyle": "pleated_skirt", "bottomColor": "#60a5fa", "shoeStyle": "platform_boots", "fabricFinish": "glitter"}],
		["🩰 Velvet Gown", {"avatarMode": "stylized", "bodyType": "female", "hairStyle": "twin_buns", "topStyle": "velvet_corset", "topColor": "#a855f7", "bottomStyle": "mermaid_gown", "bottomColor": "#ec4899", "fabricFinish": "satin"}],
		["✈️ Tech-49 Hero", {"avatarMode": "stylized", "bodyType": "male", "hairStyle": "tech_crop", "topStyle": "tech_jacket", "topColor": "#e2e8f0", "bottomStyle": "tech_pants", "bottomColor": "#475569", "shoeStyle": "flight_boots", "accessory": "aviators", "fabricFinish": "matte"}],
		["🧱 Classic Blocky", {"avatarMode": "blocky", "topColor": "#38bdf8", "bottomColor": "#fbbf24"}]
	]:
		var b := _make_btn(str(preset[0]), "#4f46e5", "#ffffff", Vector2(180, 42))
		var patch: Dictionary = preset[1]
		b.pressed.connect(func() -> void:
			net.send_game_command({"type": "avatar", "avatar": patch})
			show_toast("✨ Outfit updated! Synced across all your Playgrounds.")
		)
		row1.add_child(b)
	tab.add_child(row1)

	var color_row := HBoxContainer.new()
	for col_pair in [["Pink", "#f472b6"], ["Sky", "#38bdf8"], ["Emerald", "#34d399"], ["Gold", "#fbbf24"], ["Violet", "#a855f7"], ["Snow", "#f8fafc"]]:
		var cb := _make_btn("🎨 " + str(col_pair[0]), str(col_pair[1]), "#0f172a", Vector2(118, 38))
		var hex := str(col_pair[1])
		cb.pressed.connect(func() -> void:
			net.send_game_command({"type": "avatar", "avatar": {"topColor": hex, "torsoColor": hex}})
		)
		color_row.add_child(cb)
	tab.add_child(color_row)

	var finish_row := HBoxContainer.new()
	for fin in ["matte", "satin", "glitter"]:
		var fb := _make_btn("✨ Finish: " + fin.capitalize(), "#0284c7", "#ffffff", Vector2(160, 38))
		fb.pressed.connect(func() -> void:
			net.send_game_command({"type": "avatar", "avatar": {"fabricFinish": fin}})
		)
		finish_row.add_child(fb)
	tab.add_child(finish_row)

	var rw_hdr := Label.new()
	rw_hdr.text = "👠 DTI Timed Fashion Runway Show:"
	rw_hdr.add_theme_color_override("font_color", Color("#fbcfe8"))
	tab.add_child(rw_hdr)

	var rw_row := HBoxContainer.new()
	var btn_start_rw := _make_btn("🎬 Start / Join Show", "#db2777", "#ffffff", Vector2(185, 40))
	btn_start_rw.pressed.connect(func() -> void: net.send_game_command({"type": "runway", "action": "start_show"}))
	var btn_ready_rw := _make_btn("✅ Outfit Ready", "#38bdf8", "#0f172a", Vector2(145, 40))
	btn_ready_rw.pressed.connect(func() -> void: net.send_game_command({"type": "runway", "action": "ready_show"}))
	rw_row.add_child(btn_start_rw)
	rw_row.add_child(btn_ready_rw)
	for stars in [3, 4, 5]:
		var vb := _make_btn("⭐ %d★" % stars, "#fbbf24", "#0f172a", Vector2(84, 40))
		vb.pressed.connect(func() -> void:
			var target_u := str(world.runway_state.get("show", {}).get("spotlightUsername", net.username))
			net.send_game_command({"type": "runway", "action": "vote", "targetUsername": target_u, "stars": stars})
		)
		rw_row.add_child(vb)
	tab.add_child(rw_row)

func _populate_companions_and_sky_tab(tab: VBoxContainer) -> void:
	var pet_row := HBoxContainer.new()
	for p in [["🐶 Cloud Puppy", "cloud_puppy"], ["🦊 Kitsune", "starlight_kitsune"], ["🐱 Shadow Kitty", "shadow_kitty"], ["🐲 Crystal Dragon", "baby_crystal_dragon"]]:
		var pb := _make_btn(str(p[0]), "#0284c7", "#ffffff", Vector2(180, 42))
		var pid := str(p[1])
		pb.pressed.connect(func() -> void:
			net.send_game_command({"type": "companions", "action": "adopt_pet", "petId": pid})
		)
		pet_row.add_child(pb)
	tab.add_child(pet_row)

	var mnt_row := HBoxContainer.new()
	for m in [["🛹 Hoverboard", "starlight_board"], ["🛼 Sunset Skates", "sunset_skates"], ["☄️ Comet Surfer", "comet_surfer"]]:
		var mb := _make_btn(str(m[0]), "#4f46e5", "#ffffff", Vector2(190, 42))
		var mid := str(m[1])
		mb.pressed.connect(func() -> void:
			net.send_game_command({"type": "companions", "action": "buy_mount", "mountId": mid})
		)
		mnt_row.add_child(mb)
	tab.add_child(mnt_row)

	var sky_row := HBoxContainer.new()
	for mode in [["Auto", "auto"], ["Sunny Day", "day"], ["Sunset", "sunset"], ["Aurora Night", "aurora_night"]]:
		var sb := _make_btn("🌤️ " + str(mode[0]), "#334155", "#f8fafc", Vector2(160, 40))
		var mval := str(mode[1])
		sb.pressed.connect(func() -> void:
			net.send_game_command({"type": "sky", "action": "set_sky", "mode": mval})
		)
		sky_row.add_child(sb)
	tab.add_child(sky_row)

func _populate_homes_and_craft_tab(tab: VBoxContainer) -> void:
	var build_row := HBoxContainer.new()
	for k in ["block", "glass", "stairs", "bed", "sofa", "dining_table", "chair", "crystal_lamp", "Erase"]:
		var kb := _make_btn(k.capitalize(), "#059669" if k != "Erase" else "#be123c", "#ffffff", Vector2(86, 36))
		kb.pressed.connect(func() -> void:
			world.build_mode = true
			if k == "Erase":
				world.build_tool = "remove"
			else:
				world.build_tool = "place"
				world.selected_block_kind = k
			show_toast("🧱 Selected: " + k)
		)
		build_row.add_child(kb)
	tab.add_child(build_row)

	var hscroll := ScrollContainer.new()
	hscroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	homes_list_vbox = VBoxContainer.new()
	homes_list_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hscroll.add_child(homes_list_vbox)
	tab.add_child(hscroll)

	var craft_row := HBoxContainer.new()
	for rec in [["⚔️ Crystal Blade", "crystal_blade"], ["🪄 Starlight Wand", "starlight_wand"], ["✨ Starlight Elixir", "starlight_potion"], ["🔮 Rift Beacon", "rift_beacon"]]:
		var rb := _make_btn("🔨 " + str(rec[0]), "#fbbf24", "#0f172a", Vector2(185, 38))
		var rid := str(rec[1])
		rb.pressed.connect(func() -> void:
			net.send_game_command({"type": "craft", "recipeId": rid})
		)
		craft_row.add_child(rb)
	tab.add_child(craft_row)

func _populate_invites_and_admin_tab(tab: VBoxContainer) -> void:
	var host_hb := HBoxContainer.new()
	var host_note := LineEdit.new()
	host_note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	host_note.placeholder_text = "Kid Host: Draft Friend Invite (e.g. For Bestie Sophia — awaits 1-click Parent Approval)"
	var btn_draft := _make_btn("🎟️ Draft Host Invite", "#f472b6", "#0f172a", Vector2(185, 40))
	btn_draft.pressed.connect(func() -> void:
		net.request_json("/api/ella-world/invites/host-draft", HTTPClient.METHOD_POST, {
			"token": net.auth_token,
			"labelNote": host_note.text.strip_edges()
		}, net.auth_token, func(st: int, d: Dictionary) -> void:
			if st >= 200 and st < 300 and d.get("invite") is Dictionary:
				var code := str(d["invite"].get("code", ""))
				admin_status_label.text = "⏳ Drafted %s — Waiting for 1-click Parent Approval!" % code
			else:
				admin_status_label.text = str(d.get("error", "Only this Playground's Kid Host can draft invites."))
		)
	)
	host_hb.add_child(host_note)
	host_hb.add_child(btn_draft)
	tab.add_child(host_hb)

	var adm_hb := HBoxContainer.new()
	admin_pw_edit = LineEdit.new()
	admin_pw_edit.secret = true
	admin_pw_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	admin_pw_edit.placeholder_text = "Parent Master Password for this Playground"
	var btn_adm_login := _make_btn("🔐 Parent Unlock", "#38bdf8", "#0f172a", Vector2(155, 40))
	btn_adm_login.pressed.connect(_admin_unlock_and_load)
	var btn_adm_new_inv := _make_btn("➕ Active STAR Invite", "#34d399", "#0f172a", Vector2(180, 40))
	btn_adm_new_inv.pressed.connect(func() -> void:
		if net.admin_token.is_empty():
			admin_status_label.text = "Unlock Parent Admin first."
			return
		net.request_json("/api/ella-world/admin/invites", HTTPClient.METHOD_POST, {
			"action": "create",
			"labelNote": "Android Parent Invite",
			"maxUses": 1,
			"expiresInHours": 168,
			"starterCoins": 120
		}, net.admin_token, func(_st: int, d: Dictionary) -> void:
			if d.get("invite") is Dictionary:
				var c := str(d["invite"].get("code", ""))
				DisplayServer.clipboard_set(c)
				admin_status_label.text = "✅ Active Invite Created & Copied: " + c
				_refresh_admin_invites()
		)
	)
	adm_hb.add_child(admin_pw_edit)
	adm_hb.add_child(btn_adm_login)
	adm_hb.add_child(btn_adm_new_inv)
	tab.add_child(adm_hb)

	admin_status_label = Label.new()
	admin_status_label.text = "Parents manage each Playground with a single Master Password."
	admin_status_label.add_theme_color_override("font_color", Color("#fbbf24"))
	tab.add_child(admin_status_label)

	var inv_scroll := ScrollContainer.new()
	inv_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	admin_invites_vbox = VBoxContainer.new()
	admin_invites_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inv_scroll.add_child(admin_invites_vbox)
	tab.add_child(inv_scroll)

func _admin_unlock_and_load() -> void:
	net.request_json("/api/ella-world/admin/login", HTTPClient.METHOD_POST, {
		"password": admin_pw_edit.text,
		"worldSlug": net.current_world_slug
	}, "", func(st: int, d: Dictionary) -> void:
		if st >= 200 and st < 300 and d.has("adminToken"):
			net.admin_token = str(d["adminToken"])
			admin_status_label.text = "✅ Parent Admin unlocked for %s!" % net.current_world_name
			_refresh_admin_invites()
		else:
			admin_status_label.text = str(d.get("error", "Incorrect Parent Master Password."))
	)

func _refresh_admin_invites() -> void:
	if net.admin_token.is_empty():
		return
	net.request_json("/api/ella-world/admin/invites", HTTPClient.METHOD_GET, null, net.admin_token, func(st: int, d: Dictionary) -> void:
		if st < 200 or st >= 300:
			return
		for c in admin_invites_vbox.get_children():
			c.queue_free()
		var arr: Array = d.get("invites", []) if d.get("invites") is Array else []
		for inv in arr:
			if not (inv is Dictionary):
				continue
			var row := HBoxContainer.new()
			var code := str(inv.get("code", ""))
			var status := str(inv.get("status", ""))
			var lbl := Label.new()
			lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			lbl.text = "%s [%s] — %s (%d/%d uses)" % [code, status, str(inv.get("labelNote", "")), int(inv.get("uses", 0)), int(inv.get("maxUses", 1))]
			row.add_child(lbl)
			var copy_b := _make_btn("Copy", "#334155", "#ffffff", Vector2(72, 34))
			copy_b.pressed.connect(func() -> void:
				DisplayServer.clipboard_set(code)
				show_toast("📋 Copied " + code)
			)
			row.add_child(copy_b)
			if status == "pending_parent_approval":
				var app_b := _make_btn("✅ Approve", "#34d399", "#0f172a", Vector2(106, 34))
				app_b.pressed.connect(func() -> void:
					net.request_json("/api/ella-world/admin/invites", HTTPClient.METHOD_POST, {"action": "approve", "code": code}, net.admin_token, func(_s: int, _r: Dictionary) -> void:
						_refresh_admin_invites()
					)
				)
				row.add_child(app_b)
			elif status == "active":
				var rev_b := _make_btn("Revoke", "#be123c", "#ffffff", Vector2(84, 34))
				rev_b.pressed.connect(func() -> void:
					net.request_json("/api/ella-world/admin/invites", HTTPClient.METHOD_POST, {"action": "revoke", "code": code}, net.admin_token, func(_s: int, _r: Dictionary) -> void:
						_refresh_admin_invites()
					)
				)
				row.add_child(rev_b)
			admin_invites_vbox.add_child(row)
	)

func _open_menu_tab(tab_idx: int) -> void:
	menu_backdrop.visible = true
	menu_panel.visible = true
	touch_ctrl.modal_blocking = true
	menu_tabs.current_tab = clampi(tab_idx, 0, menu_tabs.get_tab_count() - 1)
	net.refresh_my_playgrounds()
	_rebuild_homes_list()

func _close_menu() -> void:
	menu_backdrop.visible = false
	menu_panel.visible = false
	touch_ctrl.modal_blocking = false

func _rebuild_homes_list() -> void:
	for c in homes_list_vbox.get_children():
		c.queue_free()
	for p in world.home_plots_state:
		if not (p is Dictionary):
			continue
		var row := HBoxContainer.new()
		var pid := str(p.get("plotId", ""))
		var owner_u := str(p.get("ownerUsername", "")) if p.get("ownerUsername") != null else ""
		var title := str(p.get("customTitle", "")) if p.get("customTitle") != null else str(p.get("defaultName", pid))
		var lbl := Label.new()
		lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		lbl.text = "%s — %s" % [title, "Unclaimed" if owner_u.is_empty() else ("Owned by " + str(p.get("ownerDisplayName", owner_u)))]
		row.add_child(lbl)
		var tp_b := _make_btn("🚶 Visit", "#0284c7", "#ffffff", Vector2(88, 34))
		var mbox: Array = p.get("mailboxPos", [0.0, 0.0, 0.0]) if p.get("mailboxPos") is Array else [0.0, 0.0, 0.0]
		tp_b.pressed.connect(func() -> void:
			if mbox.size() >= 3:
				world.player_pos = Vector3(float(mbox[0]), 0.4, float(mbox[2]))
				_close_menu()
		)
		row.add_child(tp_b)
		if owner_u.is_empty():
			var cl_b := _make_btn("🏡 Claim", "#34d399", "#0f172a", Vector2(96, 34))
			cl_b.pressed.connect(func() -> void:
				net.send_game_command({"type": "homes", "action": "claim", "plotId": pid})
			)
			row.add_child(cl_b)
		elif owner_u.to_lower() == net.username.to_lower():
			var un_b := _make_btn("Release", "#be123c", "#ffffff", Vector2(88, 34))
			un_b.pressed.connect(func() -> void:
				net.send_game_command({"type": "homes", "action": "unclaim", "plotId": pid})
			)
			row.add_child(un_b)
		homes_list_vbox.add_child(row)

func _on_auth_succeeded(data: Dictionary) -> void:
	auth_overlay.visible = false
	hud_root.visible = true
	touch_ctrl.enabled_gameplay = true
	touch_ctrl.modal_blocking = false
	touch_ctrl.reset_default_joy_origin()
	playground_btn.text = "🌟 %s ▾" % str(data.get("worldName", net.current_world_name))
	show_toast("✨ Welcome to %s!" % str(data.get("worldName", net.current_world_name)))

func _on_auth_failed(err_msg: String) -> void:
	_show_auth_screen(err_msg)

func _show_auth_screen(msg: String) -> void:
	auth_overlay.visible = true
	hud_root.visible = false
	touch_ctrl.enabled_gameplay = false
	auth_status_label.text = msg

func _on_session_revoked(reason: String) -> void:
	_show_auth_screen(reason)

func _on_welcome_snapshot(snap: Dictionary) -> void:
	if snap.has("worldName"):
		playground_btn.text = "🌟 %s ▾" % str(snap["worldName"])
	if snap.get("chat") is Array and chat_log_rich != null:
		chat_log_rich.clear()
		for c in snap["chat"]:
			if c is Dictionary:
				_append_chat_entry(c)

func _on_world_event(ev: Dictionary) -> void:
	var t := str(ev.get("type", ""))
	if t == "chat":
		if ev.get("chat") is Dictionary:
			_append_chat_entry(ev["chat"])
		elif ev.get("entry") is Dictionary:
			_append_chat_entry(ev["entry"])
	elif t == "homes_sync":
		_rebuild_homes_list()

func _append_chat_entry(entry: Dictionary) -> void:
	if chat_log_rich == null:
		return
	var author := str(entry.get("displayName", entry.get("username", "System")))
	var text := str(entry.get("text", ""))
	chat_log_rich.append_text("[%s]: %s\n" % [author, text])

func _on_playgrounds_updated(data: Dictionary) -> void:
	for c in playgrounds_list_vbox.get_children():
		c.queue_free()
	var raw_arr: Variant = data.get("myPlaygrounds", data.get("playgrounds", []))
	var arr: Array = raw_arr if raw_arr is Array else []
	for pg in arr:
		if not (pg is Dictionary):
			continue
		var slug := str(pg.get("slug", "ella"))
		var wname := str(pg.get("worldName", slug))
		var is_cur := (slug == net.current_world_slug)
		var row := HBoxContainer.new()
		var lbl := Label.new()
		lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		lbl.text = "%s (%s) — %d online" % [wname, slug, int(pg.get("onlineCount", 0))]
		row.add_child(lbl)
		var sw_btn := _make_btn("✅ Active" if is_cur else "🚀 Hop Here", "#334155" if is_cur else "#38bdf8", "#f8fafc" if is_cur else "#0f172a", Vector2(130, 38))
		sw_btn.disabled = is_cur
		sw_btn.pressed.connect(func() -> void:
			_close_menu()
			net.switch_playground(slug)
		)
		row.add_child(sw_btn)
		playgrounds_list_vbox.add_child(row)

func _on_net_status(mode: String, rtt_ms: float) -> void:
	if net_badge_label != null:
		net_badge_label.text = "%s%s" % [mode, (" • %dms" % int(round(rtt_ms))) if rtt_ms > 0.5 else ""]

func _on_stats_updated(coins: int, hp: int, max_hp: int, zone: int, sr_label: String) -> void:
	if stats_label == null:
		return
	var zname := "🌟 Plaza"
	match zone:
		1: zname = "💎 Cavern"
		2: zname = "⚖️ Yard"
		3: zname = "⚔️ Arena ❤️ %d/%d" % [hp, max_hp]
	stats_label.text = "🪙 %d • %s%s" % [coins, zname, (" • " + sr_label) if not sr_label.is_empty() else ""]

func show_toast(msg: String) -> void:
	if toast_label == null:
		return
	toast_label.text = msg
	toast_label.visible = true
	toast_timer = 3.6

func _process(delta: float) -> void:
	if toast_timer > 0.0:
		toast_timer -= delta
		if toast_timer <= 0.0 and toast_label != null:
			toast_label.visible = false
