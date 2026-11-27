class_name MultiTouchController
extends Control
## Landscape-First Split-Screen Multi-Touch & Desktop Input (`MultiTouchController.gd`)
## - Left 44% of viewport: 360° floating analog virtual joystick (`move_vector: Vector2`)
## - Right 56% of viewport: 1-finger swipe-to-look (`camera_yaw`, `camera_pitch`) + 2-finger pinch-zoom (`camera_distance`)
## - Strictly isolated by `InputEventScreenTouch.index` so left-thumb movement never steals right-thumb Jump/Action/Camera!

signal world_tap_requested(screen_pos: Vector2)

var enabled_gameplay: bool = false
var modal_blocking: bool = false

var move_vector: Vector2 = Vector2.ZERO
var camera_yaw: float = 0.0
var camera_pitch: float = -0.34
var camera_distance: float = 7.4

var _joy_touch_idx: int = -1
var _joy_origin: Vector2 = Vector2(146.0, 560.0)
var _joy_knob: Vector2 = Vector2(146.0, 560.0)
const JOY_MAX_RADIUS: float = 68.0

var _right_touches: Dictionary = {} # touch_idx -> Vector2
var _right_tap_start_pos: Dictionary = {}
var _right_tap_start_ms: Dictionary = {}
var _last_pinch_dist: float = -1.0
var _rmb_dragging: bool = false

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_PASS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

func reset_default_joy_origin() -> void:
	var vp := get_viewport_rect().size
	_joy_origin = Vector2(148.0, maxf(vp.y - 156.0, 260.0))
	_joy_knob = _joy_origin

func _gui_input(event: InputEvent) -> void:
	if not enabled_gameplay or modal_blocking:
		_release_all_touches()
		return

	var vp := get_viewport_rect().size
	var left_split_x := vp.x * 0.44

	if event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		if st.pressed:
			if st.position.x <= left_split_x and _joy_touch_idx == -1 and st.position.y > 96.0:
				_joy_touch_idx = st.index
				_joy_origin = st.position
				_joy_knob = st.position
				move_vector = Vector2.ZERO
				queue_redraw()
			elif st.position.x > left_split_x:
				_right_touches[st.index] = st.position
				_right_tap_start_pos[st.index] = st.position
				_right_tap_start_ms[st.index] = Time.get_ticks_msec()
				_update_pinch_seed()
		else:
			if st.index == _joy_touch_idx:
				_joy_touch_idx = -1
				reset_default_joy_origin()
				move_vector = Vector2.ZERO
				queue_redraw()
			elif _right_touches.has(st.index):
				var start_pos: Vector2 = _right_tap_start_pos.get(st.index, st.position)
				var start_ms: int = int(_right_tap_start_ms.get(st.index, 0))
				_right_touches.erase(st.index)
				_right_tap_start_pos.erase(st.index)
				_right_tap_start_ms.erase(st.index)
				_update_pinch_seed()
				if st.position.distance_to(start_pos) < 14.0 and (Time.get_ticks_msec() - start_ms) < 280:
					world_tap_requested.emit(st.position)

	elif event is InputEventScreenDrag:
		var sd := event as InputEventScreenDrag
		if sd.index == _joy_touch_idx:
			var delta := sd.position - _joy_origin
			if delta.length() > JOY_MAX_RADIUS:
				delta = delta.normalized() * JOY_MAX_RADIUS
			_joy_knob = _joy_origin + delta
			var raw_vec := delta / JOY_MAX_RADIUS
			move_vector = raw_vec if raw_vec.length() > 0.10 else Vector2.ZERO
			queue_redraw()
		elif _right_touches.has(sd.index):
			_right_touches[sd.index] = sd.position
			if _right_touches.size() >= 2:
				var keys := _right_touches.keys()
				var p0: Vector2 = _right_touches[keys[0]]
				var p1: Vector2 = _right_touches[keys[1]]
				var cur_dist := p0.distance_to(p1)
				if _last_pinch_dist > 8.0:
					var pinch_delta := cur_dist - _last_pinch_dist
					camera_distance = clampf(camera_distance - pinch_delta * 0.025, 3.0, 16.5)
				_last_pinch_dist = cur_dist
			else:
				camera_yaw -= sd.relative.x * 0.0065
				camera_pitch = clampf(camera_pitch - sd.relative.y * 0.0055, -1.22, 0.32)

	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_RIGHT:
			_rmb_dragging = mb.pressed
		elif mb.pressed and mb.button_index == MOUSE_BUTTON_WHEEL_UP:
			camera_distance = clampf(camera_distance - 0.65, 3.0, 16.5)
		elif mb.pressed and mb.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			camera_distance = clampf(camera_distance + 0.65, 3.0, 16.5)

	elif event is InputEventMouseMotion and _rmb_dragging:
		var mm := event as InputEventMouseMotion
		camera_yaw -= mm.relative.x * 0.0065
		camera_pitch = clampf(camera_pitch - mm.relative.y * 0.0055, -1.22, 0.32)

func _update_pinch_seed() -> void:
	if _right_touches.size() >= 2:
		var keys := _right_touches.keys()
		var p0: Vector2 = _right_touches[keys[0]]
		var p1: Vector2 = _right_touches[keys[1]]
		_last_pinch_dist = p0.distance_to(p1)
	else:
		_last_pinch_dist = -1.0

func _release_all_touches() -> void:
	if _joy_touch_idx != -1 or not _right_touches.is_empty():
		_joy_touch_idx = -1
		_right_touches.clear()
		_right_tap_start_pos.clear()
		_right_tap_start_ms.clear()
		_last_pinch_dist = -1.0
		move_vector = Vector2.ZERO
		reset_default_joy_origin()
		queue_redraw()

func get_combined_move_vector() -> Vector2:
	if not enabled_gameplay or modal_blocking:
		return Vector2.ZERO
	if _joy_touch_idx != -1 and move_vector.length_squared() > 0.001:
		return move_vector
	var kb := Vector2.ZERO
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT):
		kb.x -= 1.0
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT):
		kb.x += 1.0
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP):
		kb.y -= 1.0
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN):
		kb.y += 1.0
	return kb.normalized() if kb.length() > 1.0 else kb

func _notification(what: int) -> void:
	if what == NOTIFICATION_RESIZED and _joy_touch_idx == -1:
		reset_default_joy_origin()
		queue_redraw()

func _draw() -> void:
	if not enabled_gameplay or modal_blocking:
		return
	var base_pos := _joy_origin
	var knob_pos := _joy_knob if _joy_touch_idx != -1 else _joy_origin
	var active_alpha := 0.85 if _joy_touch_idx != -1 else 0.46
	draw_circle(base_pos, JOY_MAX_RADIUS + 6.0, Color(0.059, 0.090, 0.165, active_alpha * 0.72))
	draw_arc(base_pos, JOY_MAX_RADIUS, 0.0, TAU, 48, Color(0.22, 0.74, 0.97, active_alpha), 3.0, true)
	draw_circle(knob_pos, 28.0, Color(0.957, 0.447, 0.714, active_alpha))
	draw_arc(knob_pos, 28.0, 0.0, TAU, 32, Color(1.0, 1.0, 1.0, active_alpha), 2.5, true)
