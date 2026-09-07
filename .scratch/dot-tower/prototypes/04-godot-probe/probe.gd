extends Node2D
## dot-tower — Godot bringup probe harness.
##
## This is not the game. It is the Godot counterpart of the Bevy harness in
## `src/lib.rs`, carrying the same probes A–F so the two can be read side by
## side. It runs on desktop and on Android from the same source, because
## ticket 02 settled that layout branches on aspect ratio, never on platform.
##
## Probes:
##   A. Touch transport   — does input arrive, per-finger, at the coordinates we expect?
##   B. UI multi-touch    — do four buttons register simultaneous presses? (#11553 analogue)
##   C. Safe area         — does `get_display_safe_area()` account for cutouts? (#23003 analogue)
##   D. Text crispness    — what content scale yields crisp pixel text at ~2.625× density?
##   E. Save durability   — does a write inside `APPLICATION_PAUSED` complete?
##   F. UI rendering      — does the UI layer render at all, and without flicker? (#14710 analogue)
##
## WHAT DESKTOP CAN AND CANNOT SETTLE. Probe B is an engine-logic question — it
## asks whether the widget layer aggregates pointers the transport kept apart —
## so synthetic `InputEventScreenTouch`es answer it on desktop, through the same
## viewport GUI path a real touchscreen feeds. `--selftest` does exactly that.
## Probes C, E and F are hardware and driver questions and desktop cannot speak
## to them; probe F in particular rendered perfectly on macOS/Metal for Bevy too,
## which is the whole reason ticket 04 needed a device. See README.

const SYNTH_BASE: int = 101  ## Synthetic touch indices start here, so probe A can tell them from fingers.
const HUD_FONT_SIZE: int = 13
const ABILITIES: int = 4

# --- A: touch transport ----------------------------------------------------
var touches: Dictionary = {}        # touch index -> Vector2
var max_fingers: int = 0            # hardware only
var max_synth: int = 0
var seen_min := Vector2.INF
var seen_max := -Vector2.INF

# --- B: UI multi-touch -----------------------------------------------------
# Three independent counts of the same gesture.
#
# `gui_held`    — what the raw events say, read in the Button's own _gui_input.
# `signal_held` — what Button's pressed/released signals say.
# `ts_held`     — what TouchScreenButton says. It is a Node2D rather than a
#                 Control, and it is Godot's documented answer for on-screen
#                 action buttons precisely because Controls reach the widget
#                 layer through mouse emulation, which has one pointer.
#
# The counts disagreeing IS the finding: that is the exact shape of Bevy's
# #11553, where two fingers down and one released releases everything.
var gui_held: Dictionary = {}
var signal_held: Dictionary = {}
var ts_held: Dictionary = {}
var max_gui_held: int = 0
var max_signal_held: int = 0
var max_ts_held: int = 0
var last_press: String = "none yet"

var ability_buttons: Array[Button] = []
var touch_buttons: Array[TouchScreenButton] = []

# --- C: safe area ----------------------------------------------------------
var safe_area_line: String = "not read yet"
var cutout_line: String = "not read yet"

# --- D: text crispness -----------------------------------------------------
var content_scale: float = 1.0

# --- E: save durability ----------------------------------------------------
var save_report: String = "no pause yet"
var notifications: Array[String] = []

# --- F / chrome ------------------------------------------------------------
var hud: Label
var frame: Panel
var sprite: Sprite2D
var pixel_font: Font = null
var font_note: String = ""
var env_lines := PackedStringArray()


func _ready() -> void:
	_make_font()
	_make_sprite()
	_make_ui()
	_make_touch_buttons()
	_read_environment()
	_read_safe_area()
	get_viewport().size_changed.connect(_on_viewport_resized)
	for line in env_lines:
		print("[env] ", line)

	var args := OS.get_cmdline_user_args()
	if args.has("--selftest"):
		_run_selftest()
	else:
		var at := args.find("--shot")
		if at != -1 and at + 1 < args.size():
			_capture_to(args[at + 1])


func _on_viewport_resized() -> void:
	_read_safe_area()


# ---------------------------------------------------------------------------
# Construction
# ---------------------------------------------------------------------------

## Probe D's other half — `FontSmoothing::None` in the Bevy harness, three
## properties on a duplicated fallback font here. If the fallback is not a
## FontFile we say so rather than silently rendering smoothed text, because a
## smoothed HUD makes probe D unreadable without anyone noticing.
func _make_font() -> void:
	var base: Font = ThemeDB.fallback_font
	if base is FontFile:
		var ff: FontFile = (base as FontFile).duplicate()
		ff.antialiasing = TextServer.FONT_ANTIALIASING_NONE
		ff.subpixel_positioning = TextServer.SUBPIXEL_POSITIONING_DISABLED
		ff.hinting = TextServer.HINTING_NONE
		pixel_font = ff
		font_note = "antialiasing off"
	else:
		pixel_font = base
		font_note = "FALLBACK NOT A FontFile — text is smoothed, probe D unreliable"


## The sprite layer, which in the Bevy run rendered correctly throughout while
## the UI layer did not. A real Sprite2D — a canvas item, not a Control — behind
## the HUD is what makes that distinction visible here too.
func _make_sprite() -> void:
	var img := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	for y in 8:
		for x in 8:
			var lit := (x + y) % 2 == 0
			img.set_pixel(x, y, Color(0.95, 0.45, 0.15) if lit else Color(0.20, 0.10, 0.04))
	sprite = Sprite2D.new()
	sprite.texture = ImageTexture.create_from_image(img)
	sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	sprite.scale = Vector2(10, 10)
	sprite.position = get_viewport_rect().size * Vector2(0.5, 0.55)
	add_child(sprite)


func _make_ui() -> void:
	var layer := CanvasLayer.new()
	add_child(layer)

	# Probe C. Drawn from the safe area, so a wrong inset shows as a frame in
	# the wrong place rather than as a number nobody checks.
	frame = Panel.new()
	var box := StyleBoxFlat.new()
	box.bg_color = Color(0, 0, 0, 0)
	box.border_color = Color(0.25, 0.85, 0.45, 0.9)
	box.set_border_width_all(2)
	frame.add_theme_stylebox_override("panel", box)
	frame.set_anchors_preset(Control.PRESET_FULL_RECT)
	frame.mouse_filter = Control.MOUSE_FILTER_IGNORE
	layer.add_child(frame)

	# Probe F's stress. A translucent panel carrying the HUD, deliberately
	# overlapping the sprite — #14710-class corruption shows at exactly this
	# UI-over-canvas boundary.
	var hud_bg := PanelContainer.new()
	var hud_box := StyleBoxFlat.new()
	hud_box.bg_color = Color(0.02, 0.02, 0.05, 0.72)
	hud_box.border_color = Color(0.35, 0.35, 0.5, 0.8)
	hud_box.set_border_width_all(1)
	hud_box.content_margin_left = 6
	hud_box.content_margin_top = 6
	hud_box.content_margin_right = 6
	hud_box.content_margin_bottom = 6
	hud_bg.add_theme_stylebox_override("panel", hud_box)
	hud_bg.set_anchors_preset(Control.PRESET_TOP_WIDE)
	hud_bg.offset_left = 8
	hud_bg.offset_top = 8
	hud_bg.offset_right = -8
	hud_bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	frame.add_child(hud_bg)

	hud = Label.new()
	hud.add_theme_font_override("font", pixel_font)
	hud.add_theme_font_size_override("font_size", HUD_FONT_SIZE)
	hud.add_theme_color_override("font_color", Color(0.86, 0.90, 0.96))
	hud.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hud.mouse_filter = Control.MOUSE_FILTER_IGNORE
	hud_bg.add_child(hud)

	# Probe B, arm one: four Control Buttons. This is the direct analogue of the
	# bevy_ui path ticket 02 banned, and the thing to be suspicious of.
	var bar := HBoxContainer.new()
	bar.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	bar.offset_left = 8
	bar.offset_right = -8
	bar.offset_top = -88
	bar.offset_bottom = -8
	bar.add_theme_constant_override("separation", 6)
	frame.add_child(bar)

	for i in ABILITIES:
		var b := Button.new()
		b.text = "%d" % (i + 1)
		b.add_theme_font_override("font", pixel_font)
		b.add_theme_font_size_override("font_size", HUD_FONT_SIZE)
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.size_flags_vertical = Control.SIZE_EXPAND_FILL
		# Buttons take focus by default; four of them fighting over it during a
		# multi-touch press is noise we do not want inside probe B.
		b.focus_mode = Control.FOCUS_NONE
		b.gui_input.connect(_on_button_gui_input.bind(i))
		b.button_down.connect(_on_button_down.bind(i))
		b.button_up.connect(_on_button_up.bind(i))
		bar.add_child(b)
		ability_buttons.append(b)

	# Probe D's live control. The Bevy harness could only report the arithmetic;
	# here the value can be walked until the text looks right, which is the
	# question ticket 02 asked and could not answer from source.
	var scale_bar := HBoxContainer.new()
	scale_bar.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	scale_bar.offset_left = 8
	scale_bar.offset_right = -8
	scale_bar.offset_top = -212
	scale_bar.offset_bottom = -176
	scale_bar.add_theme_constant_override("separation", 6)
	frame.add_child(scale_bar)

	for spec in [["scale -", -0.125], ["scale +", 0.125]]:
		var b := Button.new()
		b.text = str(spec[0])
		b.add_theme_font_override("font", pixel_font)
		b.add_theme_font_size_override("font_size", HUD_FONT_SIZE)
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.focus_mode = Control.FOCUS_NONE
		b.pressed.connect(_nudge_content_scale.bind(float(spec[1])))
		scale_bar.add_child(b)


## Probe B, arm two: four TouchScreenButtons. Node2D rather than Control, and
## the node Godot documents for on-screen action buttons. If arm one caps at one
## simultaneous press and this does not, that is the whole answer for the four
## hero ability buttons — and it is a node choice, not a fork in the engine.
func _make_touch_buttons() -> void:
	for i in ABILITIES:
		var tb := TouchScreenButton.new()
		tb.texture_normal = _slab(Color(0.16, 0.18, 0.26, 1.0))
		tb.texture_pressed = _slab(Color(0.30, 0.55, 0.35, 1.0))
		# An explicit shape, rather than relying on the texture bounds, so the
		# hit area is unambiguous when the two ever disagree.
		var shape := RectangleShape2D.new()
		shape.size = Vector2(96, 56)
		tb.shape = shape
		tb.shape_centered = false
		tb.pressed.connect(_on_ts_pressed.bind(i))
		tb.released.connect(_on_ts_released.bind(i))
		# Parented to the safe-area frame, not to the root. A Node2D under a
		# Control inherits its transform, so the row lands inside the inset for
		# free — and stays there under probe D's content scale, which would
		# otherwise pull the UI and the 2D layer apart. Ticket 10: nothing may
		# occlude the ability bar, and sitting under the system bars counts.
		frame.add_child(tb)
		touch_buttons.append(tb)
	frame.resized.connect(_layout_touch_buttons)
	_layout_touch_buttons.call_deferred()


func _slab(c: Color) -> ImageTexture:
	var img := Image.create(96, 56, false, Image.FORMAT_RGBA8)
	img.fill(c)
	return ImageTexture.create_from_image(img)


## TouchScreenButtons are Node2Ds, so they carry no anchors and have to be
## placed against the viewport by hand — and replaced whenever it resizes, which
## on Android happens on rotation and on the first real surface.
func _layout_touch_buttons() -> void:
	var size := frame.size
	var pad := 8.0
	var gap := 6.0
	var w := (size.x - pad * 2.0 - gap * (ABILITIES - 1)) / float(ABILITIES)
	var y := size.y - 168.0
	for i in touch_buttons.size():
		var tb := touch_buttons[i]
		tb.position = Vector2(pad + float(i) * (w + gap), y)
		tb.scale = Vector2(w / 96.0, 56.0 / 56.0)
		var shape := tb.shape as RectangleShape2D
		if shape:
			shape.size = Vector2(96, 56)


# ---------------------------------------------------------------------------
# Probe A — touch transport
# ---------------------------------------------------------------------------

## Runs before the GUI sees the event and never marks it handled, so a touch
## landing on an ability button is counted by both probe A and probe B.
func _input(event: InputEvent) -> void:
	var pos := Vector2.ZERO
	var idx := -1
	if event is InputEventScreenTouch:
		var t := event as InputEventScreenTouch
		idx = t.index
		pos = t.position
		if t.pressed:
			touches[idx] = pos
		else:
			touches.erase(idx)
	elif event is InputEventScreenDrag:
		var d := event as InputEventScreenDrag
		idx = d.index
		pos = d.position
		touches[idx] = pos
	else:
		return

	# Synthetic presses sit at fixed button centres. Letting them into the
	# high-water mark or the reach extents would quietly corrupt both readings.
	if idx < SYNTH_BASE:
		_note_extent(pos)
		if touches.size() > 0:
			sprite.position = pos

	var hw := 0
	var sy := 0
	for k in touches:
		if k >= SYNTH_BASE:
			sy += 1
		else:
			hw += 1
	max_fingers = maxi(max_fingers, hw)
	max_synth = maxi(max_synth, sy)


## Probe A's edge-coordinate half (#7528 analogue): walking a finger into each
## bezel should drive these to 0 and to the window bounds. Anything short of
## that is the edge bug, and it is why ticket 02 wanted targets inset.
func _note_extent(p: Vector2) -> void:
	seen_min = Vector2(minf(seen_min.x, p.x), minf(seen_min.y, p.y))
	seen_max = Vector2(maxf(seen_max.x, p.x), maxf(seen_max.y, p.y))


# ---------------------------------------------------------------------------
# Probe B — UI multi-touch
# ---------------------------------------------------------------------------

## Keys 1–4 inject a synthetic touch at the matching button's centre. They go
## through `Input.parse_input_event`, which is the same pipeline the OS
## touchscreen feeds, so holding several at once exercises the real routing.
func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey):
		return
	var k := event as InputEventKey
	if k.echo:
		return
	var slot := -1
	match k.keycode:
		KEY_1: slot = 0
		KEY_2: slot = 1
		KEY_3: slot = 2
		KEY_4: slot = 3
		_: return
	_synth_touch(slot, k.pressed)


## Sends one synthetic finger at the centre of both arm-one and arm-two buttons
## for `slot`. They are stacked vertically, so a single index cannot press both;
## two indices are used, offset so they never collide with hardware fingers.
func _synth_touch(slot: int, pressed: bool) -> void:
	if slot >= ability_buttons.size() or slot >= touch_buttons.size():
		return

	var control_at := ability_buttons[slot].get_global_rect().get_center()
	_send_touch(SYNTH_BASE + slot, control_at, pressed)

	var tb := touch_buttons[slot]
	# Input events arrive in viewport coordinates, but the button now lives in
	# the frame's local space, so its centre has to be mapped back out.
	var centre: Vector2 = tb.get_global_transform_with_canvas() * (Vector2(96, 56) * 0.5)
	_send_touch(SYNTH_BASE + ABILITIES + slot, centre, pressed)


func _send_touch(index: int, at: Vector2, pressed: bool) -> void:
	var t := InputEventScreenTouch.new()
	t.index = index
	t.position = at
	t.pressed = pressed
	Input.parse_input_event(t)


func _on_button_gui_input(event: InputEvent, idx: int) -> void:
	if event is InputEventScreenTouch:
		var t := event as InputEventScreenTouch
		if t.pressed:
			gui_held[idx] = t.index
			last_press = "control %d via touch #%d" % [idx + 1, t.index]
		else:
			gui_held.erase(idx)
	elif event is InputEventMouseButton:
		var m := event as InputEventMouseButton
		if m.button_index != MOUSE_BUTTON_LEFT:
			return
		if m.pressed:
			gui_held[idx] = -1
			last_press = "control %d via mouse" % (idx + 1)
		else:
			gui_held.erase(idx)
	else:
		return
	max_gui_held = maxi(max_gui_held, gui_held.size())


func _on_button_down(idx: int) -> void:
	signal_held[idx] = true
	max_signal_held = maxi(max_signal_held, signal_held.size())


func _on_button_up(idx: int) -> void:
	signal_held.erase(idx)


func _on_ts_pressed(idx: int) -> void:
	ts_held[idx] = true
	max_ts_held = maxi(max_ts_held, ts_held.size())
	last_press = "touchscreen %d" % (idx + 1)


func _on_ts_released(idx: int) -> void:
	ts_held.erase(idx)


# ---------------------------------------------------------------------------
# Probe B — automated verdict
# ---------------------------------------------------------------------------

## Presses all four abilities, then releases exactly one. That second phase is
## the #11553 shape: Bevy's widget layer released everything when any one finger
## lifted, because it consulted global touch aggregates. Whatever Godot does
## here, it does it without a phone.
func _run_selftest() -> void:
	for _i in 8:
		await get_tree().process_frame

	# Repeated, because the first pass of this test disagreed with itself
	# between runs. A probe that reports a flake as a finding is worse than no
	# probe, so the run count is part of the instrument.
	var runs := 5
	print("\n=== probe B — four simultaneous presses, %d runs ===" % runs)
	print("%-24s %-12s %-12s %s" % ["phase", "Control gui", "Control sig", "TouchScreen"])

	var control_pass := 0
	var ts_pass := 0
	for r in runs:
		for slot in ABILITIES:
			_synth_touch(slot, true)
		await _settle()
		var d_sig := _held_set(signal_held)
		var d_gui := _held_set(gui_held)
		var d_ts := _held_set(ts_held)

		_synth_touch(1, false)
		await _settle()
		var p_sig := _held_set(signal_held)
		var p_gui := _held_set(gui_held)
		var p_ts := _held_set(ts_held)

		for slot in [0, 2, 3]:
			_synth_touch(slot, false)
		await _settle()
		var u_sig := _held_set(signal_held)
		var u_gui := _held_set(gui_held)
		var u_ts := _held_set(ts_held)

		print("run %d  4 down          %-12s %-12s %s" % [r + 1, d_gui, d_sig, d_ts])
		print("       release #2      %-12s %-12s %s" % [p_gui, p_sig, p_ts])
		print("       all up          %-12s %-12s %s" % [u_gui, u_sig, u_ts])

		# Correct behaviour is exact membership, not a count: after releasing
		# finger 2 the held set must be {1,3,4} and nothing else.
		if d_sig == "1234" and p_sig == "134" and u_sig == "-":
			control_pass += 1
		if d_ts == "1234" and p_ts == "134" and u_ts == "-":
			ts_pass += 1
		await _settle()

	print("")
	print("Control Button     : %d/%d runs correct — %s" % [
		control_pass, runs,
		"multi-touch OK" if control_pass == runs else "NOT dependable for the ability bar"])
	print("TouchScreenButton  : %d/%d runs correct — %s" % [
		ts_pass, runs,
		"multi-touch OK" if ts_pass == runs else "NOT dependable for the ability bar"])
	print("")
	get_tree().quit(0 if (control_pass == runs or ts_pass == runs) else 1)


## Renders a held-button dictionary as sorted 1-based ids, so the log shows
## WHICH button wrongly released rather than only how many did.
func _held_set(d: Dictionary) -> String:
	var ids: Array = d.keys()
	ids.sort()
	if ids.is_empty():
		return "-"
	var out := ""
	for i in ids:
		out += str(int(i) + 1)
	return out


func _settle() -> void:
	for _i in 6:
		await get_tree().process_frame


# ---------------------------------------------------------------------------
# Probe C — safe area and cutouts
# ---------------------------------------------------------------------------

## Godot splits what Bevy's `content_rect()` conflates: the safe area is one
## call and the cutout rectangles are another. That is exactly ticket 02's open
## question — whether the inset accounts for cutouts — so read both and let the
## frame show the answer.
func _read_safe_area() -> void:
	var win := DisplayServer.window_get_size()
	var screen := DisplayServer.screen_get_size()
	var safe: Rect2i = DisplayServer.get_display_safe_area()

	# The safe area is a rect in SCREEN coordinates, so the insets it implies
	# are the window's insets only when the window is the screen. That holds on
	# Android and fails on a desktop window, where subtracting a 480x960 window
	# from the laptop display yields large negative insets and throws the frame
	# — and every bottom-anchored child with it — off the viewport.
	var fullscreen := win == screen
	var left := safe.position.x
	var top := safe.position.y
	var right := screen.x - (safe.position.x + safe.size.x)
	var bottom := screen.y - (safe.position.y + safe.size.y)

	if fullscreen and (left != 0 or top != 0 or right != 0 or bottom != 0):
		safe_area_line = "insets L%d T%d R%d B%d of %dx%d" % [left, top, right, bottom, win.x, win.y]
	else:
		if not fullscreen:
			safe_area_line = "window %dx%d != screen %dx%d — frame is a stand-in" % [
				win.x, win.y, screen.x, screen.y]
		else:
			safe_area_line = "no insets reported (%dx%d) — frame is a stand-in" % [win.x, win.y]
		left = 24
		top = 48
		right = 24
		bottom = 24

	var s := maxf(content_scale, 0.001)
	frame.offset_left = left / s
	frame.offset_top = top / s
	frame.offset_right = -right / s
	frame.offset_bottom = -bottom / s

	if DisplayServer.has_method("get_display_cutouts"):
		var cutouts: Array = DisplayServer.get_display_cutouts()
		if cutouts.is_empty():
			cutout_line = "no cutouts reported"
		else:
			var parts := PackedStringArray()
			for c in cutouts:
				parts.append("%d,%d %dx%d" % [c.position.x, c.position.y, c.size.x, c.size.y])
			cutout_line = "%d cutout(s): %s" % [cutouts.size(), ", ".join(parts)]
	else:
		cutout_line = "get_display_cutouts() unavailable"


# ---------------------------------------------------------------------------
# Probe D — content scale
# ---------------------------------------------------------------------------

func _nudge_content_scale(delta: float) -> void:
	content_scale = clampf(content_scale + delta, 0.25, 4.0)
	get_window().content_scale_factor = content_scale
	# The frame is expressed in scaled units, so it has to be recomputed or it
	# drifts off the safe area as the scale moves.
	_read_safe_area()


# ---------------------------------------------------------------------------
# Probe E — save durability
# ---------------------------------------------------------------------------

## The claim under test, inherited from ticket 01 by way of the Bevy harness:
## that the pause notification arrives early enough for a synchronous write to
## complete. Every notification that arrives is logged, because the Bevy run's
## real finding was about which ones never do.
func _notification(what: int) -> void:
	var label := ""
	match what:
		MainLoop.NOTIFICATION_APPLICATION_PAUSED:
			label = "APPLICATION_PAUSED"
		MainLoop.NOTIFICATION_APPLICATION_RESUMED:
			label = "APPLICATION_RESUMED"
		MainLoop.NOTIFICATION_APPLICATION_FOCUS_IN:
			label = "APPLICATION_FOCUS_IN"
		MainLoop.NOTIFICATION_APPLICATION_FOCUS_OUT:
			label = "APPLICATION_FOCUS_OUT"
		MainLoop.NOTIFICATION_OS_MEMORY_WARNING:
			label = "OS_MEMORY_WARNING"
		NOTIFICATION_WM_CLOSE_REQUEST:
			label = "WM_CLOSE_REQUEST"
		NOTIFICATION_WM_GO_BACK_REQUEST:
			label = "WM_GO_BACK_REQUEST"
		_:
			return

	print("probe E: ", label)
	notifications.append(label)
	if notifications.size() > 6:
		notifications.remove_at(0)

	# Every plausible suspend hook writes, so the log says which one won the
	# race rather than which one we guessed.
	if label in ["APPLICATION_PAUSED", "APPLICATION_FOCUS_OUT", "WM_CLOSE_REQUEST", "WM_GO_BACK_REQUEST"]:
		_save_now(label)


func _save_now(trigger: String) -> void:
	var before := Time.get_ticks_usec()
	var payload := "trigger = %s\nunix_millis = %d\nmax_fingers = %d\nmax_gui_held = %d\nmax_ts_held = %d\n" % [
		trigger,
		int(Time.get_unix_time_from_system() * 1000.0),
		max_fingers,
		max_gui_held,
		max_ts_held,
	]

	var path := "user://bringup-probe.txt"
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		save_report = "save FAILED: open error %d" % FileAccess.get_open_error()
		printerr("probe E: ", save_report)
		return
	f.store_string(payload)
	# NOTE: this is a userspace flush, NOT an fsync. Godot's FileAccess exposes
	# no sync_all() equivalent, so unlike the Bevy harness this cannot prove the
	# bytes reached the disk — only that the write completed before the process
	# was frozen. See README, probe E.
	f.flush()
	f.close()

	var elapsed := (Time.get_ticks_usec() - before) / 1000.0
	save_report = "%s: wrote %d bytes in %.2fms" % [trigger, payload.length(), elapsed]
	# The HUD has one narrow column, so the path goes to the log only — on
	# Android it is the `run-as` path you need, and it is useless on screen.
	print("probe E: ", save_report, " -> ", ProjectSettings.globalize_path(path))


# ---------------------------------------------------------------------------
# Environment, capture, HUD
# ---------------------------------------------------------------------------

func _read_environment() -> void:
	var win := DisplayServer.window_get_size()
	env_lines.append("os %s %s / model %s" % [OS.get_name(), OS.get_version(), OS.get_model_name()])
	env_lines.append("window %dx%d, dpi %d, screen scale %.3f" % [
		win.x, win.y, DisplayServer.screen_get_dpi(), DisplayServer.screen_get_scale()])

	var method := "?"
	if RenderingServer.has_method("get_current_rendering_method"):
		method = str(RenderingServer.get_current_rendering_method())
	var driver := "?"
	if RenderingServer.has_method("get_current_rendering_driver_name"):
		driver = str(RenderingServer.get_current_rendering_driver_name())
	env_lines.append("renderer %s / %s" % [method, driver])
	env_lines.append("adapter %s (%s)" % [
		RenderingServer.get_video_adapter_name(), RenderingServer.get_video_adapter_api_version()])
	env_lines.append("font: %s" % font_note)


## Probe F is judged by eye, so a run has to be able to leave behind a reference
## image to hold a device photo against. Without it nothing here records what
## "correct" is supposed to look like.
func _capture_to(path: String) -> void:
	# Two frames is not enough: theme resolution and container layout each
	# settle a frame late, and a shot taken early shows an unlaid-out HUD.
	for _i in 10:
		await get_tree().process_frame
	var img := get_viewport().get_texture().get_image()
	var err := img.save_png(path)
	if err != OK:
		printerr("capture failed: ", err)
	else:
		print("captured -> ", path)
	get_tree().quit()


func _process(_delta: float) -> void:
	var win := DisplayServer.window_get_size()
	var screen_scale := DisplayServer.screen_get_scale()
	var out := PackedStringArray()

	for line in env_lines:
		out.append(line)

	# Probe D: the arithmetic crispness turns on, plus the live value.
	out.append("[D] content_scale %.3f -> %dpx text at %.1f phys px" % [
		content_scale, HUD_FONT_SIZE, float(HUD_FONT_SIZE) * screen_scale * content_scale])

	# Probe C.
	out.append("[C] " + safe_area_line)
	out.append("    " + cutout_line)

	# Probe A.
	out.append("[A] fingers %d (max %d), synthetic %d (max %d)" % [
		_count_touches(false), max_fingers, _count_touches(true), max_synth])
	var shown := 0
	for idx in touches:
		if idx >= SYNTH_BASE or shown >= 4:
			continue
		var p: Vector2 = touches[idx]
		out.append("      #%s at %.0f,%.0f" % [idx, p.x, p.y])
		shown += 1
	if seen_max.x > -INF:
		out.append("    reach x %.0f..%.0f y %.0f..%.0f (of %dx%d)" % [
			seen_min.x, seen_max.x, seen_min.y, seen_max.y, win.x, win.y])

	# Probe B. Three counts side by side on purpose: the moment they disagree
	# is the finding. Keys 1-4 inject synthetic fingers.
	out.append("[B] last %s   (hold keys 1-4)" % last_press)
	out.append("    Control  gui %d / sig %d   max %d / %d" % [
		gui_held.size(), signal_held.size(), max_gui_held, max_signal_held])
	out.append("    TouchScreenButton %d       max %d" % [ts_held.size(), max_ts_held])

	# Probe E.
	out.append("[E] " + save_report)
	if not notifications.is_empty():
		out.append("    seen: " + ", ".join(notifications))

	hud.text = "\n".join(out)


func _count_touches(synthetic: bool) -> int:
	var n := 0
	for k in touches:
		if (k >= SYNTH_BASE) == synthetic:
			n += 1
	return n
