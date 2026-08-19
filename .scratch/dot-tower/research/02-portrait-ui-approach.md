# Portrait-first UI approach in Bevy 0.18

Research for ticket `.scratch/dot-tower/issues/02-portrait-ui-approach.md`.

- **Date of investigation:** 2026-08-18
- **Bevy version read:** `v0.18.0` tag (released 2026-01-13). Patch `0.18.1` released 2026-03-04.
- **Transitive versions read:** `bevy_egui v0.39.1` (the last 0.18-compatible release), `bevy_lunex 0.6.0` / repo `main`, `bevy_immediate` 0.5.x–0.8.0 metadata, `bevy_image_font 0.11.0`, `android-activity 0.6.1`, `winit 0.30`.
- **Method:** reading primary source at pinned tags (Bevy repo, bevy_egui repo, bevy-lunex repo), crates.io release + dependency metadata via the API, the GitHub issue tracker, and the three prior-art repos on this machine. **Nothing here was compiled or run on a device.** Every claim is tagged `[VERIFIED]` (read directly from a primary source, with the file named) or `[INFERENCE]` (my reasoning across sources).
- **Not consulted:** Bevy's Discord (not reachable). `bevy_declarative` source (lives on the user's other PC — its shape is inferred from call sites, as the ticket instructed).

---

## Summary — what this means for dot-tower

1. **`bevy_ui` is already compiled into this project and costs nothing extra.** `features = ["2d"]` expands to include `ui` *and* `picking`, which in turn include `ui_api`, `ui_bevy_render`, and `ui_picking`. There is no "should we pay for bevy_ui" question — we are already paying. Every third-party option is a cost *on top* of a UI layer we already ship.
2. **Touch works natively through `bevy_picking`, and this settles the hardest constraint.** `bevy_picking`'s input plugin reads `WindowEvent::TouchInput` directly and spawns one `PointerId::Touch(id)` pointer per finger. `bevy_ui` registers its picking backend unconditionally. `bevy_ui_widgets` is built *entirely* on `Pointer<...>` observers. So the chain Android touch → `TouchInput` → `PointerId::Touch` → `Pointer<Click>` → widget is complete with no shim. **The single most important evaluation criterion is satisfied by the built-in option.**
3. **But do not use `Interaction`.** Bevy's legacy `ui_focus_system` *does* read `Touches` — however it only ever reads `first_pressed_position()`, `any_just_pressed()` and `any_just_released()`. Those are global, not per-finger. Two-finger use breaks (open issue #11553, and the defect is still visible in the 0.18 source). **guild-forge's `InteractionPalette` pattern is exactly this trap** and should not be copied to a touch-first game. Use `Pointer<Over>/<Out>/<Press>/<Release>` observers instead.
4. **Nobody solves safe areas. This is a wash, not a differentiator.** Bevy issue #23003 is open and labelled `S-Blocked`; it is blocked on `winit 0.31` (still beta), and winit's Android implementation of `Window::safe_area` is itself still an open issue (#4506). No option in this comparison offers safe-area insets. We must build it by hand from `bevy::android::ANDROID_APP` → `AndroidApp::content_rect()`.
5. **Bevy 0.19 shipped on 2026-06-19 (0.19.1 on 2026-08-13).** We are one release behind. This matters mainly as a maintenance filter: it separates crates that track Bevy from crates that have stalled.
6. **`bevy_lunex` has not been updated for 0.19 and its repo has been quiet since February.** It is the only candidate that has not followed Bevy forward. That is a poor bet for a project that will want to move to 0.19.
7. **`bevy_egui` is the healthiest third-party crate but the wrong tool here.** It handles touch properly and its maintainer is extremely responsive (released for 0.19 within days). But it is a vector-look immediate-mode UI with its own rasterizer and no pixelated-text mode — it directly fights the pixel-art constraint — and its README lists "Desktop and web platforms support", never native mobile. shenji-bevy already used it; the user moved away from it in every project since.
8. **Pixel fonts have exactly one trap and two fixes.** `TextFont::font_size` is multiplied by the window scale factor and `UiScale` before rasterisation, so a pixel font at `font_size: 8` becomes 21 physical px on a 2.625× phone — a non-integer multiple of its design size, which looks wrong even with antialiasing off. Fix A: pin the effective scale (`UiScale`, or `WindowResolution::set_scale_factor_override`) so the product is an integer multiple. Fix B: bypass the glyph rasteriser entirely with `bevy_image_font` 0.11 (Bevy 0.18, has a `ui` feature). Fix A is cheaper and should be tried first.
9. **There is no responsive/media-query system in Bevy, in any option.** The idiomatic 0.18 answer is *one tree with conditional chrome*: a single root, `Val::Vw/Vh/VMin/VMax` for the frame, and one system that watches window aspect and flips `Node.display` / `flex_direction` on the side panels. Two separate layouts is the wrong shape for a game whose portrait column is identical in both.

**Recommended: built-in `bevy_ui` + `experimental_bevy_ui_widgets`, driven exclusively through `bevy_picking` observers, with a thin local styling builder.** Reasoning and risks in [§6](#6-recommendation).

---

## 1. The options

### 1.0 What we already have — the feature accounting

`[VERIFIED]` `Cargo.toml` at `v0.18.0`, lines 131–177:

```toml
2d = [
  "default_app",
  "default_platform",
  "2d_bevy_render",
  "ui",
  "scene",
  "audio",
  "picking",
]

ui = [
  "default_app",
  "default_platform",
  "ui_api",
  "ui_bevy_render",
  "scene",
  "audio",
  "picking",
]

# COLLECTION: Features used to enable picking in Bevy apps.
picking = ["bevy_picking", "mesh_picking", "sprite_picking", "ui_picking"]
```

and lines 260, 280:

```toml
ui_api = ["default_app", "common_api", "bevy_ui"]

# Provides an implementation for picking UI
ui_picking = ["bevy_internal/ui_picking"]
```

`[VERIFIED]` So `bevy = { version = "0.18", default-features = false, features = ["2d"] }` **already enables `bevy_ui`, `bevy_ui_render`, `bevy_picking` and the UI picking backend.** The ticket's question "what extra Bevy features does each option need" has a surprising answer for the built-in option: **none**.

`[VERIFIED]` `experimental_bevy_ui_widgets` is a separate opt-in, `Cargo.toml` line 399:

```toml
# Experimental headless widget collection for Bevy UI.
experimental_bevy_ui_widgets = ["bevy_internal/bevy_ui_widgets"]
```

`[VERIFIED]` `experimental_bevy_feathers` exists too and *depends on* `experimental_bevy_ui_widgets` (lines 402–406). Feathers is the styled editor-tooling layer on top of the headless widgets — its own crate description reads "A collection of UI widgets for building editors and utilities in Bevy". `[INFERENCE]` Feathers is not for us: it ships its own Fira Sans/Fira Mono TTFs and a fixed dark desktop theme (`crates/bevy_feathers/src/{dark_theme,palette,font_styles}.rs`), which is the exact "smoothed vector look" the ticket rules out.

### 1.1 `bevy_ui` (built-in, 0.18)

**Touch: yes, natively, and multi-touch-correct — *if* you use picking.**

`[VERIFIED]` `crates/bevy_picking/src/input.rs` at `v0.18.0`. The plugin registers both backends separately (lines 95–111):

```rust
mouse_pick_events.run_if(PointerInputSettings::is_mouse_enabled),
touch_pick_events.run_if(PointerInputSettings::is_touch_enabled),
```

and `touch_pick_events` (lines 198–262) reads `WindowEvent::TouchInput` and creates a **distinct pointer entity per finger**:

```rust
if let WindowEvent::TouchInput(touch) = window_event {
    let pointer = PointerId::Touch(touch.id);
    ...
    TouchPhase::Started => {
        commands.spawn((pointer, PointerLocation::new(location.clone())));
```

This is the exact counterpart to ticket 01's finding that Android delivers only `WindowEvent::Touch`. `bevy_picking` consumes it directly — it does not go looking for a cursor.

`[VERIFIED]` `crates/bevy_ui/src/lib.rs` line 172 — the backend is added unconditionally by `UiPlugin`:

```rust
app.add_plugins(picking_backend::UiPickingPlugin)
```

`[VERIFIED]` `crates/bevy_ui/src/picking_backend.rs` header confirms it is pointer-generic, not mouse-specific: *"It will look for any pointers using the same render target as the UI camera, and run hit tests on the UI node tree."*

**Touch: partially, and buggily — if you use `Interaction`.**

`[VERIFIED]` `crates/bevy_ui/src/focus.rs` at `v0.18.0`. `ui_focus_system` *does* consult touch (lines 155–156, 176, 188):

```rust
mouse_button_input: Res<ButtonInput<MouseButton>>,
touches_input: Res<Touches>,
...
let mouse_released =
    mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();
...
let mouse_clicked =
    mouse_button_input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed();
```

and derives position by falling back to touch (lines 205–209):

```rust
window
    .physical_cursor_position()
    .or_else(|| {
        touches_input
            .first_pressed_position()
            .map(|pos| pos * window.scale_factor())
    })
```

`[VERIFIED]` **`first_pressed_position()` / `any_just_released()` are global aggregates, not per-touch-id.** So with two fingers down, releasing either one releases *every* pressed node, and hover position always tracks whichever finger landed first. This is exactly the complaint in open issue #11553 ("Bevy UI multiple button partial touch release", 2024-01-27, still open), and the defect is visible in the 0.18 source above — it has not been fixed. Related open issues: #2333 (2021-06-12, `ui_focus_system` touch handling), #5985 (touch propagation), #7371 (`Interaction` lacks `just_pressed`/`just_released`).

`[INFERENCE]` **This is a decisive, actionable finding for ticket 10 item 3.** Four thumb-reachable ability buttons is precisely a multi-touch surface. Build them on `Pointer<Press>`/`Pointer<Release>` observers, never on `Interaction`.

`[VERIFIED]` Bevy's own mobile example still uses the buggy path — `examples/mobile/src/lib.rs` has `button_handler` querying `(Changed<Interaction>, With<Button>)`. It works because that example has exactly one button. Do not read it as an endorsement.

**Safe areas: no.** See [§1.5](#15-safe-areas-nobody-has-them).

**Maintained against 0.18 / cadence:** `[VERIFIED]` it *is* Bevy. Released in lockstep; `bevy_ui_widgets` on crates.io shows `0.18.0` (2026-01-13), `0.18.1` (2026-03-04), `0.19.0` (2026-06-19), `0.19.1` (2026-08-13) — Bevy's ~5-month major cadence with patch releases.

**Pixel-art fit:** `[VERIFIED]` first-class. `FontSmoothing::None` and `UiAntiAlias::Off` exist specifically for this — see [§4](#4-text-legibility).

### 1.2 `experimental_bevy_ui_widgets` — what it actually provides, and how stable

`[VERIFIED]` `crates/bevy_ui_widgets/src/lib.rs` at `v0.18.0`, crate docs verbatim:

```
//! This crate provides a set of standard widgets for Bevy UI, such as buttons, checkboxes, and sliders.
//! These widgets have no inherent styling, it's the responsibility of the user to add styling
//! appropriate for their game or application.
//!
//! ## Warning: Experimental
//!
//! This crate is currently experimental and under active development.
//! The API is likely to change substantially: be prepared to migrate your code.
```

`[VERIFIED]` The full module list is exactly eight: `button`, `checkbox`, `menu`, `observe`, `popover`, `radio`, `scrollbar`, `slider`. `UiWidgetsPlugins` is a `PluginGroup` adding `PopoverPlugin, ButtonPlugin, CheckboxPlugin, MenuPlugin, RadioGroupPlugin, ScrollbarPlugin, SliderPlugin`.

`[VERIFIED]` **It is headless — logic and state only, zero visual output.** You supply every pixel. That is precisely what a pixel-art game wants: it means the crate cannot impose a vector look on you.

`[VERIFIED]` **Everything is picking-driven, hence touch-native.** Imports at the top of each widget:

- `button.rs:15` — `use bevy_picking::events::{Cancel, Click, DragEnd, Pointer, Press, Release};`
- `slider.rs:24` — `use bevy_picking::events::{Drag, DragEnd, DragStart, Pointer, Press};`
- `scrollbar.rs:12` — `use bevy_picking::events::{Cancel, Drag, DragEnd, DragStart, Pointer, Press};`

There is **no `ButtonInput<MouseButton>` or `CursorMoved` read anywhere in the widget input paths** — the only `ButtonState` references are keyboard activation handlers. `[INFERENCE]` So these widgets respond to touch correctly, including multi-touch, because each `PointerId::Touch(n)` is an independent pointer. This is the strongest single argument for the built-in stack.

`[VERIFIED]` State is external by design (crate docs): *"the widgets do not automatically update their own internal state, but instead rely on the app to update the widget state ... in response to a change event emitted by the widget."* Widgets emit `Activate` and `ValueChange<T>` `EntityEvent`s.

**How stable, really?** `[INFERENCE]` The "likely to change substantially" warning is real but bounded: the API is small (8 modules), it is in-tree so every change ships with a first-party migration guide, and the 0.17→0.18 migration guide contains no breaking entry for `bevy_ui_widgets` at all — the 0.18 release notes describe *additions* (`Popover`, `MenuPopup`, radio improvements). `[INFERENCE]` The practical risk is one afternoon of migration per Bevy release, and it is containable by wrapping every widget spawn behind our own helper functions — which is what bevy-shenji already does (see [§2.1](#21-bevy-shenji-bevy-0173-bevy_immediate-04--experimental_bevy_ui_widgets)).

**Cost:** one extra Cargo feature. `[INFERENCE]` No new dependency tree, no version-skew risk, no separate release cadence.

### 1.3 `bevy_egui`

**Touch: yes, thoroughly.** `[VERIFIED]` `src/input.rs` at `v0.39.1` has `write_window_touch_messages_system` and `write_non_window_touch_messages_system`, an `EguiContextPointerTouchId` component tracking the active touch, and a `write_touch_message` fn mapping the full phase set:

```rust
event: egui::Event::Touch {
    device_id: egui::TouchDeviceId(message.window.to_bits()),
    id: touch_id,
    phase: match message.phase {
        bevy_input::touch::TouchPhase::Started => egui::TouchPhase::Start,
        bevy_input::touch::TouchPhase::Moved  => egui::TouchPhase::Move,
        bevy_input::touch::TouchPhase::Ended  => egui::TouchPhase::End,
        bevy_input::touch::TouchPhase::Canceled => egui::TouchPhase::Cancel,
```

including force/pressure translation. It additionally synthesises egui pointer events from the *first* active touch so ordinary widgets respond.

**Safe areas: no.**

**Maintained against 0.18: yes, but 0.18 is now a back-version.** `[VERIFIED]` crates.io dependency metadata:

| bevy_egui | targets |
|---|---|
| 0.38.x | bevy 0.17 |
| **0.39.0 (2026-01-14), 0.39.1 (2026-02-06)** | **bevy 0.18** |
| 0.40.x (2026-06-19) | bevy 0.19 |
| 0.41.x, 0.42.0 (2026-08-16) | bevy 0.19 |

`[VERIFIED]` **Cadence is excellent** — `0.39.0` landed 2026-01-14, *one day* after Bevy 0.18. Repo last pushed 2026-08-17, 1,400 stars, 33 open issues. This is the most actively maintained third-party option by a distance.

**Why it is still wrong here:** `[VERIFIED]` egui renders through its own rasteriser and font atlas, entirely bypassing `bevy_text`; there is no `FontSmoothing::None` equivalent, so the pixel-art constraint is unmet by construction. `[VERIFIED]` The README's feature list opens with "Desktop and web platforms support" and the only mobile item is "Mobile web virtual keyboard (still rough around the edges)" — **native Android is not a claimed platform.** `[INFERENCE]` The touch code exists and is careful, but shipping on an unclaimed platform means we own every bug we find.

### 1.4 `bevy_lunex`

**Touch: yes, inherited from `bevy_picking`.** `[VERIFIED]` `crate/src/picking.rs` — Lunex ships a *picking backend*, not an input source:

```rust
use bevy_picking::backend::prelude::*;
use bevy_picking::{backend::PointerHits, Pickable};
...
fn lunex_2d_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
```

`[INFERENCE]` Because it consumes `PointerId`/`PointerLocation` rather than raw input, it gets `PointerId::Touch(n)` for free from `bevy_picking`'s touch backend. Touch is fine.

`[VERIFIED]` **One caveat:** `crate/src/cursor.rs` (the `SoftwareCursor` / `GamepadCursor` module) synthesises `PointerInput` from `MouseButtonInput` and `GamepadButtonChangedEvent` only — `PointerId::Mouse` hardcoded at lines 449, 465, 503, 519. `[INFERENCE]` That module is opt-in (for gamepad-driven virtual cursors) and irrelevant on a phone; just do not enable it.

**Safe areas: no.**

**Maintained against 0.18: yes — and *only* 0.18, which is the problem.** `[VERIFIED]` `bevy_lunex 0.6.0` (2026-01-22) depends on `bevy_app ^0.18` etc. It is the newest release. `[VERIFIED]` Repo `bytestring-net/bevy-lunex`: last push **2026-02-24**, 946 stars, 13 open issues, not archived. Recent commits: `2026-02-24 Add HUD example and Fix tracing ANSCII`, `2026-01-22 Updated to 0.18`, `2025-10-13 Bumped to bevy 0.17`.

`[VERIFIED]` **Cadence: roughly one release per Bevy version, arriving days-to-weeks after — but Bevy 0.19 shipped 2026-06-19 and two months later there is still no 0.19 release and no commits since February.** `[INFERENCE]` This is the weakest maintenance signal of the four. Historically the author has caught up (0.16 → 0.17 → 0.18 all landed), so it may well resume; but adopting it now means betting that it does.

**What it would buy us:** `[VERIFIED]` README: *"Any aspect ratio: Lunex is designed to support ALL window sizes out of the box without deforming. The built in layout types react nicely and intuitively to aspect ratio changes."* `[VERIFIED]` It provides units `Ab` (absolute), `Rl`/`Rw`/`Rh` (relative to parent, optionally cross-axis), `Em`, and `Vp`/`Vw`/`Vh` (viewport). `[INFERENCE]` The genuinely distinctive `Rw`/`Rh` — "proportional to a width measure even when used in a height field" — is what makes square things stay square across aspect ratios. Bevy's `Val` can approximate this with `VMin`, and `Node.aspect_ratio` covers the square case outright. **The differentiator is real but narrow, and does not justify a stale dependency.** Its other headline feature, worldspace/diegetic 3D UI, is irrelevant to a 2D portrait game.

### 1.5 Safe areas: nobody has them

`[VERIFIED]` Bevy issue **#23003** "Device safe area insets (for iOS and Android)", opened 2026-02-17, **open**, labels `C-Feature, A-Windowing, A-UI, O-Android, O-iOS, S-Blocked, X-Uncontroversial, D-Straightforward`. It asks for a `SafeAreaInsets` resource. The reporter notes winit already logs `WARN winit::platform_impl::android: TODO: handle Android InsetsChanged notification`.

`[VERIFIED]` The blocker, from the issue thread:

- mockersf: safe area is *"available on iOS but not yet on Android"* in winit, pointing at winit issue #3910.
- The reporter, after attempting it: *"Looked into it but it requires `winit 0.31`, which is still in beta (last release is `0.31.0-beta.2 (2025-11-16)`). I tried upgrading `bevy_winit` to it but (surprise surprise) it's not trivial."*

`[VERIFIED]` winit's own tracker: **#3910 "Finish safe area implementation" (open, 2024-09-11)**, **#4506 "Android: implement `Window::safe_area`" (open, 2026-03-07)**, #3911 "Add event for changes to the safe area" (open). So even on winit 0.31 the Android side is unimplemented.

`[VERIFIED]` Bevy 0.18 uses winit 0.30 (established in ticket 01). `[INFERENCE]` **Safe areas are therefore not arriving in Bevy 0.18, and probably not in 0.19 either.**

**The workaround, and it is a good one:** `[VERIFIED]` `android_activity 0.6.1` exposes `AndroidApp::content_rect()`, documented as *"Queries the current content rectangle of the window; this is the area where the window's content should be placed to be seen by the user."* `[VERIFIED]` Bevy exposes the `AndroidApp` as `bevy::android::ANDROID_APP` (ticket 01, `crates/bevy_android/src/lib.rs`). `[INFERENCE]` So we can write a small `SafeAreaInsets` resource ourselves: poll `content_rect()` each frame (or on `WindowResized`/config change), diff against the window's physical size, and expose four inset values. This is maybe 40 lines and is **the same amount of work regardless of which UI crate we pick** — it is not a reason to choose one over another. Caveat `[INFERENCE]`: `content_rect` reflects the GameActivity content rect, which tracks system bar insets but is not documented as accounting for display cutouts/notches specifically; it should be validated on a notched device before being trusted (folds into ticket 04, device bringup).

**Compounding hazard:** `[VERIFIED]` open issue #7528 (touch coordinates misreported near screen edges, open since 2023 — from ticket 01). `[INFERENCE]` The bottom ability bar and any top status strip are exactly edge-docked UI. **Inset every touch target away from the physical edge.** Conveniently, honouring safe-area insets does this anyway — so implementing insets mitigates two problems at once. Treat the inset as a minimum of `max(safe_area, ~8dp)` rather than raw safe area.

### 1.6 Anything else current?

`[INFERENCE]` The other names that come up are not live options for this project:

- **`bevy_feathers`** (`experimental_bevy_feathers`) — in-tree, but it is a *styled* editor/tooling theme with bundled Fira fonts. Wrong aesthetic. Useful only as a reference implementation of how to style `bevy_ui_widgets`.
- **`bevy_immediate`** — not a separate UI stack; see [§2.1](#21-bevy-shenji-bevy-0173-bevy_immediate-04--experimental_bevy_ui_widgets). It is a wrapper *over* `bevy_ui`.
- **`bevy_declarative`** — the user's own local crate; likewise a wrapper over `bevy_ui`. See [§2.3](#23-guild-forge-bevy-018-local-bevy_declarative).

---

## 2. Prior art in this user's own projects

The headline result: **three of the four codebases converge on `bevy_ui`.** Two of them reach it through a thin ergonomic wrapper, and the wrappers are strikingly similar. The one that does not (shenji-bevy/egui) is the oldest and was abandoned in favour of the others.

### 2.1 bevy-shenji (Bevy 0.17.3, `bevy_immediate` 0.4, `experimental_bevy_ui_widgets`)

`[VERIFIED]` `/Users/tacit/dev/bevy-shenji/Cargo.toml`:

```toml
bevy = { version = "0.17.3", features = ["hotpatching", "experimental_bevy_ui_widgets"] }
bevy_immediate = { version = "0.4", features = ["hotpatching"] }
lucide-icons = "0.563.0"
```

`[VERIFIED]` **`bevy_immediate` is a wrapper over `bevy_ui`, not an alternative to it.** From crates.io dependency metadata, `bevy_immediate_ui 0.8.0` depends on: `bevy_ui`, `bevy_ui_widgets`, `bevy_feathers`, `bevy_picking`, `bevy_input_focus`, `bevy_text`. `[INFERENCE]` It therefore inherits `bevy_ui`'s touch story *exactly* — same picking backend, same widgets, same `FontSmoothing`. Choosing `bevy_immediate` is choosing `bevy_ui` plus an immediate-mode authoring layer.

`[VERIFIED]` `docs/STYLING.md` (33 KB, written for humans *and* agents) documents a complete Tailwind-shaped design system over it — a palette (`GRAY_800`, `PRIMARY_500`, semantic `HEADER_TEXT`/`LABEL_TEXT`), a 4px spacing scale, typography presets, borders, layout/visual/text/image primitives, style presets, and 12 widgets (Label, Icon, Button, Badge, Divider, ProgressBar, Tabs, Tooltip, Modal, List, ListItem, Table). Core pattern:

```rust
ui.ch()                          // spawn a child entity, returns ImmEntity
    .flex_col()
    .p(SPACE_4)
    .bg(GRAY_800)
    .add(|ui| { ui.ch().label("Hello"); });
```

with `.style(|n: &mut Node| { ... })` documented as the escape hatch to raw Bevy `Node` — `[INFERENCE]` confirming from the docs alone that the output is ordinary `bevy_ui` entities.

`[VERIFIED]` `experimental_bevy_ui_widgets` is used sparingly and always behind a local wrapper — only two files touch it, `theme/widgets/slider.rs` and `theme/widgets/radio.rs`, with comments like *"Bevy 0.17 provides `bevy_ui_widgets::Slider` which handles all input logic"*. `[INFERENCE]` **This is the containment pattern to copy**: adopt the experimental crate for the fiddly stateful widgets (slider, radio, scrollbar) where hand-rolling is genuinely expensive, wrap it locally, and leave the API-churn surface tiny.

`[VERIFIED]` `src/app_caps.rs` shows the framework's extension model — an `AppCaps` capability set combining `bevy_immediate` core capabilities (`CapabilityUiBase/Text/Interaction/Look`) with project-local ones (`CapabilityUiLayout`, `CapabilityUiTextStyle`, `CapabilityUiVisuals`, `CapabilityButton`, `CapabilityObserver`, `CapabilityUiImage`). The file's own doc comment says the theme is *"intentionally outside the `theme` module so the theme stays reusable across projects"*.

`[VERIFIED]` Layout (`src/game/ui/layout.rs`) is a **desktop** shape: sidebar row + right column (scrolling main view + bottom bar) + absolute context-menu overlay. Fonts are vector TTFs (`GoogleSans.ttf`, `Kenney Space.ttf`), no `FontSmoothing` setting anywhere, no touch handling, no responsive logic. `[INFERENCE]` This project is not portrait or pixel-art prior art — it is *styling-system* prior art, and it is the most sophisticated of the three.

`[VERIFIED]` `bevy_immediate` version map (crates.io): 0.4.0/0.5.x → bevy 0.18 for 0.5.1, **0.6.x → bevy 0.18.1**, 0.7.0/0.8.0 → bevy 0.19. Repo `PPakalns/bevy_immediate` last pushed 2026-07-15, 151 stars, 2 open issues. `[VERIFIED]` Cadence is fast — 0.7.0 landed 2026-06-20, one day after Bevy 0.19. `[INFERENCE]` Small but healthy and clearly tracking Bevy closely. For Bevy 0.18 we would pin `bevy_immediate 0.6.x`.

### 2.2 shenji-bevy (Bevy 0.17.2, `bevy_egui` 0.38)

`[VERIFIED]` `/Users/tacit/dev/shenji-bevy/Cargo.toml` — 5 dependencies total, `bevy_egui = "0.38.0"`. `[VERIFIED]` The UI is plain egui immediate mode: `egui::SidePanel::left("shared_sidebar").default_width(200.0).resizable(true)`, `egui::CentralPanel`, `egui::ScrollArea::vertical()`, `ui.selectable_label(...)` for navigation, `egui::FontId::default()` for text. Window is `resolution: (1280, 720)`.

`[INFERENCE]` **This is the "get something on screen fast" project and it shows** — hardcoded desktop resolution, default fonts, resizable side panels, no theming layer, `src/ui/components.rs` hand-drawing shapes with `egui::Stroke`. It is the oldest of the three and the user has not returned to egui in either successor. `[INFERENCE]` The lesson to carry forward is a negative one: egui gets you moving quickly but leaves you with no design system and a look you cannot make pixel-art.

### 2.3 guild-forge (Bevy 0.18, local `bevy_declarative`)

This is the closest analogue to dot-tower — same Bevy version, same feature line.

`[VERIFIED]` `/Users/tacit/dev/guild-forge/Cargo.toml`:

```toml
bevy = { version = "0.18", default-features = false, features = ["2d"] }
bevy_declarative = { path = "../bevy_declarative" }
```

`[VERIFIED]` Note: **`experimental_bevy_ui_widgets` is NOT enabled here.** The `2d`-only feature line is identical to dot-tower's.

**Inferring `bevy_declarative`'s shape from call sites** (the crate itself is on the user's other PC, as expected):

`[VERIFIED]` Import surface, consistent across all 8 call sites:

```rust
use bevy_declarative::element::div::{Div, div};
use bevy_declarative::element::text::{TextEl, text};
use bevy_declarative::style::styled::Styled;
use bevy_declarative::style::values::{pct, px};
```

`[VERIFIED]` The complete set of builder methods actually used across `src/screens/`, `src/ui/`, `src/theme/widgets.rs`:

- **layout**: `.abs()`, `.absolute()`, `.row()`, `.col()`, `.w()`, `.h()`, `.w_full()`, `.h_full()`, `.flex_1()`, `.gap()`, `.p()`, `.items_center()`, `.justify_center()`, `.justify_between()`, `.overflow_y_scroll()`, `.overflow_y_hidden()`
- **visual**: `.bg()`, `.rounded()`, `.border_radius()`, `.with_z()`
- **text**: `.font_size()`, `.color()`
- **ECS bridge**: `.insert(bundle)`, `.child(el)`, `.add_child()`, `.add_children()`, `.on_click(observer)`, `.spawn(&mut commands)`, `.spawn_as_child_of(&mut commands, entity)`
- **values**: `px(f32)`, `pct(f32)`

`[VERIFIED]` It is unambiguously a **retained-mode builder that spawns ordinary `bevy_ui` entities**, because raw Bevy types pass straight through `.insert(...)` and `.border_radius(...)`:

```rust
.insert((Name::new("Button"), Button, InteractionPalette { ... }))
.border_radius(BorderRadius::MAX)
.on_click(action)   // impl IntoObserverSystem<Pointer<Click>, B, M>
```

`[VERIFIED]` `.on_click` takes `impl IntoObserverSystem<Pointer<Click>, B, M> + Sync + 'static` (from `theme/widgets.rs` signatures) — i.e. it is a `bevy_picking` observer registration. `[INFERENCE]` **So guild-forge's click handling is already touch-correct**, and `bevy_declarative` is essentially the same idea as `bevy_immediate`'s API expressed in retained rather than immediate style — the method vocabulary (`.w_full()`, `.flex_1()`, `.gap()`, `.p()`, `.bg()`, `.rounded()`, `.items_center()`) is almost a subset of what `STYLING.md` documents.

`[VERIFIED]` **But guild-forge also demonstrates the trap.** `theme/interaction.rs`'s `InteractionPalette { none, hovered, pressed }` is applied to every button, and `screens/sidebar.rs` drives highlight state from `&mut BackgroundColor` keyed on `Interaction`. `[INFERENCE]` Per [§1.1](#11-bevy_ui-built-in-018), that is the single-touch-only path. It is fine for guild-forge, which is a desktop game with a mouse. **It must not be copied into dot-tower.**

`[VERIFIED]` Layout is desktop-shaped and fixed: `const SIDEBAR_WIDTH: f32 = 220.0;`, `game_button` at `w(px(380.0)).h(px(80.0))` with `font_size(40.0)`, `Window { fit_canvas_to_parent: true }` (a web concern, not a mobile one). **No responsive logic, no `pct`-based frame, no touch, no pixel fonts.** `[INFERENCE]` guild-forge answers "what does the user's 0.18 UI code look like" but not "how should a portrait game be laid out" — that question is genuinely unanswered in the prior art.

### 2.4 What the prior art suggests

`[INFERENCE]`

1. **The user's settled preference is a chainable Tailwind-shaped builder over `bevy_ui`.** Two independent implementations (`bevy_immediate` + the `STYLING.md` theme; `bevy_declarative`) converge on nearly the same method vocabulary. Fighting this would be a mistake.
2. **The choice between immediate and retained is not the interesting axis** — both wrappers produce `bevy_ui` entities and inherit identical touch, text and layout behaviour. Pick on ergonomics and dependency risk, not capability.
3. **egui was tried and left behind.** Do not relitigate.
4. **Nothing in the prior art is portrait, touch-first, or pixel-art.** All three are desktop landscape with vector fonts and mouse `Interaction`. dot-tower is new ground on every axis the ticket asks about, and the prior art's `Interaction`-based patterns are actively wrong here.
5. **`bevy_declarative` is a real asset but a real risk.** It is the user's own code, matches their taste, and already works on Bevy 0.18 — but it is unavailable on this machine and is a single-maintainer, unpublished crate. `[INFERENCE]` Its used surface is ~25 methods; reimplementing it against raw `Node` is a day's work if it ever becomes a blocker. Do not architect around its absence, and do not block on its presence.

---

## 3. Responsive layout

### 3.1 What Bevy 0.18 gives you

`[VERIFIED]` `crates/bevy_ui/src/geometry.rs` lines 31–57 — `Val` has viewport units:

```rust
pub enum Val {
    Auto,
    Px(f32),        // logical pixels
    Percent(f32),   // % of parent's length along the axis
    Vw(f32),        // % of viewport width
    Vh(f32),        // % of viewport height
    VMin(f32),      // % of the viewport's smaller dimension
    VMax(f32),      // % of the viewport's larger dimension
}
```

`[VERIFIED]` `crates/bevy_ui/src/ui_node.rs` — `Node` carries `display`, `position_type`, `overflow`, `flex_direction`, `min_width`/`max_width` (and height equivalents), and `aspect_ratio: Option<f32>` (line 599). Full CSS-flexbox plus CSS-grid semantics via Taffy.

`[VERIFIED]` `UiScale(pub f32)` — `crates/bevy_ui/src/lib.rs` lines 113–125, with the important restriction in its own doc: *"**Note:** This will only affect fixed ui values like `Val::Px`"*.

`[VERIFIED]` **There is no media-query, breakpoint, or orientation mechanism anywhere in `bevy_ui`.** `[VERIFIED]` No `A-UI` issue about orientation or portrait exists on the tracker (a search for open orientation+android issues returns only #14710, which is about UI nodes flickering on some Android devices). `[INFERENCE]` This is not an oversight to wait out — it is a gap every Bevy game fills itself, in about 20 lines.

### 3.2 One layout with conditional chrome, or two layouts?

`[INFERENCE]` **One layout with conditional chrome.** The reasoning is specific to this game rather than general:

- The ticket's own framing already settles it: *"desktop spending its extra width on side panels rather than restaging the scene"*. The tower column is **identical** in both presentations. Two layouts would duplicate the one part that must not diverge, and the duplication would be invisible until the two drift.
- The transition is continuous, not discrete. A desktop window dragged narrow, a tablet, a foldable, and a phone in landscape are all points on one axis. Two layouts forces a hard breakpoint that will be wrong for some of them; a flex root with `display: None` side panels degrades smoothly through all of them.
- Bevy's flexbox already does the work. A root `flex_direction: Row` containing `[left panel] [tower column] [right panel]`, where the tower column is `width: VMin(100)`-ish and `max_width: Px(...)`, and the panels are `flex_grow: 1.0` — collapses to exactly the portrait column when the panels are hidden, with no repositioning code.
- One tree means one set of entity markers, so ticket 10's "what must be permanently visible" budget is expressed once.

**The idiomatic shape** `[INFERENCE]`, assembled from verified primitives:

```rust
// One root, always. Portrait column is the flex centre; panels are optional siblings.
#[derive(Resource, PartialEq, Eq, Clone, Copy)]
enum Frame { Portrait, Wide }

// Drive it off window aspect, not off target_os — a narrow desktop window
// must behave like a phone, and a tablet in landscape must behave like desktop.
fn update_frame(
    windows: Query<&Window, Changed<Window>>,
    mut frame: ResMut<Frame>,
    mut panels: Query<&mut Node, With<SidePanel>>,
) {
    let Ok(window) = windows.single() else { return };
    let next = if window.width() / window.height() >= 1.2 { Frame::Wide } else { Frame::Portrait };
    if *frame == next { return; }
    *frame = next;
    for mut node in &mut panels {
        node.display = if next == Frame::Wide { Display::Flex } else { Display::None };
    }
}
```

`[INFERENCE]` Three notes on getting this right:

- **Branch on aspect ratio, never on `cfg!(target_os = "android")`.** A resized desktop window is the cheapest possible test rig for the portrait layout — the ticket-10 prototype can be driven entirely on desktop if the condition is aspect-based. Hard-coding the platform throws that away.
- **`Changed<Window>` is the right trigger.** Do not recompute every frame; do not use a `WindowResized` message alone, since it does not fire on initial layout.
- **Add hysteresis if flicker appears.** A single threshold at exactly 1.0 will thrash while a window is dragged across it. `[INFERENCE]` Untested — only add it if observed.

`[VERIFIED]` For working at both 9:16 and 16:9 without deforming, the units to reach for are `VMin`/`VMax` and `Node.aspect_ratio`. `VMin` sizes relative to the **smaller** dimension, which is the width in portrait and the height in landscape — so `width: VMin(100)` gives "as wide as the narrow axis allows" in both orientations, which is exactly the tower column's rule. `[INFERENCE]` This is Bevy's native answer to the problem `bevy_lunex` markets its `Rw`/`Rh` units for, and it is sufficient here.

`[INFERENCE]` **Keep the frame in relative units and the chrome in `Px`.** Because `UiScale` multiplies only `Val::Px` ([§3.1](#31-what-bevy-018-gives-you)), a layout whose *structure* is `Percent`/`Vw`/`VMin` and whose *pixel-art chrome* (borders, icon boxes, bar heights) is `Px` gives you a single knob — `UiScale` — that scales the art without touching the layout. That is a genuinely useful property and it falls out for free if the units are chosen deliberately. It also dovetails with the font fix in [§4.3](#43-fix-a-pin-the-effective-scale-preferred).

`[VERIFIED]` Scrolling the tower column on touch is a solved pattern in-tree: `examples/ui/drag_to_scroll.rs` at `v0.18.0` uses `Overflow::scroll()` + `ScrollPosition` + a `Pointer<Drag>` observer, and — importantly — divides by `UiScale`:

```rust
.observe(|drag: On<Pointer<Drag>>, ui_scale: Res<UiScale>, ...| {
    scroll_position.0 = (start.0 - drag.distance / ui_scale.0).max(Vec2::ZERO);
})
```

`[INFERENCE]` Because it is a `Pointer<Drag>` observer it is touch-native, and it is the direct answer to ticket 10's camera/scroll question for the UI-side approach. Note it must divide by `UiScale` or drag distance will not match finger distance.

---

## 4. Text legibility

### 4.1 What Bevy 0.18 provides for pixel text

`[VERIFIED]` `crates/bevy_text/src/text.rs` lines 716–738:

```rust
/// Determines which antialiasing method to use when rendering text. By default, text is
/// rendered with grayscale antialiasing, but this can be changed to achieve a pixelated look.
pub enum FontSmoothing {
    /// No antialiasing. Useful for when you want to render text with a pixel art aesthetic.
    ///
    /// Combine this with `UiAntiAlias::Off` and `Msaa::Off` on your 2D camera for a fully pixelated look.
    ///
    /// **Note:** Due to limitations of the underlying text rendering library,
    /// this may require specially-crafted pixel fonts to look good, especially at small sizes.
    None,
    #[default]
    AntiAliased,
}
```

`[VERIFIED]` `crates/bevy_ui_render/src/lib.rs` lines 163–171 — `UiAntiAlias { On, Off }`, a **component on the camera**, with the doc example spawning `(Camera2d, UiAntiAlias::Off)`.

`[VERIFIED]` `Msaa::Off` on the 2D camera is already recommended by Bevy's own mobile example (ticket 01).

`[INFERENCE]` **So the three-part recipe is documented by Bevy itself and needs no third-party crate:** `TextFont { font_smoothing: FontSmoothing::None, .. }` on text, `UiAntiAlias::Off` + `Msaa::Off` on the camera. That is the baseline; do it from day one, because retrofitting it after art is authored means re-tuning every font size.

### 4.2 The actual trap

`[VERIFIED]` `TextFont::font_size` doc, `crates/bevy_text/src/text.rs` lines 261–268:

> The vertical height of rasterized glyphs in the font atlas in pixels.
>
> **This is multiplied by the window scale factor and `UiScale`**, but not the text entity transform or camera projection.
>
> A new font atlas is generated for **every combination of font handle and scaled font size** which can have a strong performance impact.

`[INFERENCE]` **Both halves of that doc are the trap, and they are separate problems.**

- **Blurriness.** The rasterised size is `font_size × window.scale_factor() × UiScale`. A pixel font is designed at one native size (say 8px) and only looks right at integer multiples of it. Android scale factors are ugly — commonly 2.625 (xxhdpi ~420dpi), 2.75, 3.0, 3.5 — so `font_size: 8` on a 2.625× device rasterises at 21px, which is 2.625× the design size. Turning antialiasing off does not fix this; it makes it *worse*, because now the glyph is being nearest-sampled at a fractional scale and stems land on different pixel counts. **This is why "just set `FontSmoothing::None`" is not the whole answer** and why the doc warns it "may require specially-crafted pixel fonts to look good".
- **Atlas churn.** One atlas per (font, scaled size) pair. `[INFERENCE]` If a responsive design computes font sizes from viewport units, or `UiScale` animates, you generate atlases continuously. Keep font sizes to a small fixed set of constants — which the `STYLING.md` typography scale already does — and never derive them from window size.

### 4.3 Fix A: pin the effective scale (preferred)

`[VERIFIED]` Two levers exist:

- `UiScale(pub f32)` — resource, multiplies `Val::Px` and text size. `crates/bevy_ui/src/lib.rs:113`.
- `WindowResolution::set_scale_factor_override(Option<f32>)` — *"Set the window's scale factor, this will be used over what the backend decides."* `crates/bevy_window/src/window.rs:1023–1030`.

`[INFERENCE]` The approach: at startup (and on `Changed<Window>`), read the real scale factor, then pick an **integer** `n` such that `n ≈ scale_factor` and set `UiScale` so the product `font_size × scale_factor × ui_scale` lands on `n × design_size`. Concretely, with a font designed at 8px and a device scale factor of 2.625, choose `n = 3` and set `UiScale = 3.0 / 2.625 ≈ 1.143` — every `Px` value and every font size then rasterises at a clean 3× the design grid.

`[INFERENCE]` **`set_scale_factor_override` is the blunter instrument** — it changes the window's logical size too, so a fixed override makes the logical viewport a fixed physical/override ratio. That may be desirable (it gives a predictable design canvas) but it interacts with the layout in [§3](#3-responsive-layout) and with touch coordinates. Prefer `UiScale`, which touches only sizes, and only reach for the override if `UiScale` proves insufficient. `[INFERENCE]` **Untested either way** — this needs the ticket-10 prototype on a real device to confirm, and it is the single highest-value thing to check during ticket 04 device bringup.

`[INFERENCE]` Caveat worth flagging: because `UiScale` affects only `Val::Px`, changing it rescales chrome and text but *not* a `Percent`/`Vw`-based frame. That is the desired behaviour per [§3.2](#32-one-layout-with-conditional-chrome-or-two-layouts) — but it means the frame must be authored in relative units for the trick to work cleanly.

### 4.4 Fix B: bypass the rasteriser with an image font

`[VERIFIED]` `bevy_image_font 0.11.0`, released 2026-02-20, repo `ilyvion/bevy_image_font`, ~5,790 downloads. Dependencies are `bevy_app/bevy_asset/bevy_ecs/bevy_image/bevy_render/bevy_sprite/... ^0.18` — **Bevy 0.18 compatible**. README: *"Add the following to your Cargo.toml: `bevy = "0.18"`, `bevy_image_font = "0.11"`"*.

`[VERIFIED]` It renders fonts stored as a single PNG with each glyph at a predefined position, and it supports UI: an entity gets `ImageFontText` plus one of

- `Sprite` + `ImageFontPreRenderedText` (world-space), or
- **`ImageNode` + `ImageFontPreRenderedUiText`** (`bevy_ui`), or
- `ImageFontSpriteText` (atlas-based, animatable per-glyph).

`[VERIFIED]` Feature flags: `ui` (gates `ImageFontPreRenderedUiText`, pulls `bevy/bevy_ui`), `rendered`, `atlas_sprites` — all default-on, individually disableable.

`[VERIFIED]` **Known limitations, stated by the crate:** no newlines, no automatic line wrapping (both "Out of Scope"/"Known Limitations"), space characters require a blank texture region, and non-`Center` sprite anchors are advised to avoid blurry odd-dimensioned sprites.

`[INFERENCE]` This gives *guaranteed* crispness — glyphs are image blits at whatever integer scale you choose, with `ImageSampler::nearest`, and the DPI problem disappears because nothing is rasterised at runtime. The price is real: no wrapping and no newlines means every multi-line string must be laid out by hand as separate nodes. `[INFERENCE]` **For an idle game that is a smaller price than it sounds** — the permanently-visible HUD of ticket 10 (gold, floor number, lock line, cooldowns, prestige multiplier) is all short single-line numeric strings, which is exactly this crate's sweet spot. Long prose (relic descriptions, tooltips, settings) can stay on `bevy_text` with `FontSmoothing::None`.

`[INFERENCE]` **Recommended sequencing:** ship on Fix A alone. Adopt Fix B *only for the HUD numerals* if Fix A does not survive contact with a real device. Do not adopt it wholesale — it is a single-maintainer crate (last release 2026-02-20, no 0.19 release yet) and would become a migration dependency.

### 4.5 Sizing for thumbs, not just for pixels

`[INFERENCE]` A legibility point the ticket does not ask but ticket 10 needs: at `UiScale` tuned for a 3× pixel grid, an 8px-design-size font is 24 physical px tall — comfortably readable — but a button sized to fit it is *not* comfortably tappable. Android's guidance is a 48dp minimum touch target. `[INFERENCE]` Size ability buttons and upgrade rows by touch target first (`Px` values that land ≥48dp after `UiScale`), then fit the pixel art into them, rather than the reverse. Combined with the edge-inset rule from [§1.5](#15-safe-areas-nobody-has-them), this is the whole mobile-ergonomics budget.

---

## 5. Comparison table

Criteria are the ticket's, ordered by the ticket's own stated priority — touch first.

| | **`bevy_ui` (+ `experimental_bevy_ui_widgets`)** | **`bevy_egui`** | **`bevy_lunex`** | **`bevy_immediate` / `bevy_declarative`** |
|---|---|---|---|---|
| **Touch: consumes Bevy touch events natively?** | **Yes.** `bevy_picking` reads `WindowEvent::TouchInput` → `PointerId::Touch(id)`; all `bevy_ui_widgets` are `Pointer<...>` observers `[VERIFIED]` | **Yes.** `TouchInput` → `egui::Event::Touch` with full phase + force mapping `[VERIFIED]` | **Yes**, indirectly — it is a `bevy_picking` *backend*, consumes `PointerId`/`PointerLocation` `[VERIFIED]` | **Yes** — depends on `bevy_ui` + `bevy_ui_widgets` + `bevy_picking`; identical to column 1 `[VERIFIED]` |
| **Multi-touch correct?** | **Yes via picking** (one pointer per finger). **No via `Interaction`** — global `first_pressed_position()`/`any_just_released()`, issue #11553 still live in 0.18 source `[VERIFIED]` | Yes for `egui::Event::Touch`; synthesised pointer follows one touch `[VERIFIED]` | Yes (inherits picking) `[INFERENCE]` | Same as column 1 `[INFERENCE]` |
| **Mouse-only shim needed on Android?** | **No** `[VERIFIED]` | **No** `[VERIFIED]` | No — but do not enable the `SoftwareCursor`/`GamepadCursor` module, which is `MouseButtonInput`-only `[VERIFIED]` | **No** `[INFERENCE]` |
| **Safe areas / notches** | **No** (#23003 open, `S-Blocked` on winit 0.31; winit #4506 Android unimplemented) `[VERIFIED]` | **No** `[VERIFIED]` | **No** `[VERIFIED]` | **No** `[INFERENCE]` |
| **Maintained against 0.18** | Is Bevy `[VERIFIED]` | `0.39.1`, 2026-02-06 `[VERIFIED]` | `0.6.0`, 2026-01-22 `[VERIFIED]` | `bevy_immediate 0.6.x`; `bevy_declarative` already on 0.18 `[VERIFIED]` |
| **Has followed to 0.19?** | **Yes** (0.19.0 2026-06-19, 0.19.1 2026-08-13) `[VERIFIED]` | **Yes** (0.40.0 on 0.19 release day) `[VERIFIED]` | **No — none, 2 months on. Repo quiet since 2026-02-24** `[VERIFIED]` | `bevy_immediate` **yes** (0.7.0 one day after) `[VERIFIED]`; `bevy_declarative` unknown `[INFERENCE]` |
| **Release cadence** | Bevy's ~5-month major + patches `[VERIFIED]` | Per-Bevy-release within days; frequent minors (7 releases in 2026) `[VERIFIED]` | ~1 per Bevy version, days-to-weeks — currently lapsed `[VERIFIED]` | `bevy_immediate` very fast, small crate `[VERIFIED]` |
| **Repo health** | n/a | pushed 2026-08-17, 1.4k★, 33 issues `[VERIFIED]` | pushed 2026-02-24, 946★, 13 issues `[VERIFIED]` | pushed 2026-07-15, 151★, 2 issues `[VERIFIED]` |
| **Pixel-art fit** | **Best.** `FontSmoothing::None` + `UiAntiAlias::Off` + `Msaa::Off` documented together `[VERIFIED]` | **Worst.** Own rasteriser, no pixelated text mode; vector look by construction `[VERIFIED]` | Good — you draw everything; but text goes through `bevy_text` anyway `[INFERENCE]` | **Best** — same `bevy_text` path as column 1 `[INFERENCE]` |
| **Extra Bevy features needed beyond `["2d"]`** | **None** for `bevy_ui`/picking (already in `2d`); `experimental_bevy_ui_widgets` is 1 flag `[VERIFIED]` | None `[INFERENCE]` | None `[INFERENCE]` | None `[INFERENCE]` |
| **Native Android a claimed platform?** | Yes (Bevy CI builds Android) `[VERIFIED]` | **No** — README says "Desktop and web" `[VERIFIED]` | Not stated `[INFERENCE]` | Not stated `[INFERENCE]` |
| **Responsive/aspect support** | `Vw/Vh/VMin/VMax`, `aspect_ratio`, flex+grid; no media queries `[VERIFIED]` | Panels auto-fit; no design system `[VERIFIED]` | Markets "any aspect ratio"; `Rw`/`Rh` cross-axis units `[VERIFIED]` | Inherits `bevy_ui`; wrappers expose only `px`/`pct` `[VERIFIED]` |
| **Prior art in this user's projects** | Indirect (via both wrappers) | shenji-bevy — **abandoned** | None | bevy-shenji, guild-forge — **current practice** |
| **API stability risk** | Core stable; widgets crate says *"likely to change substantially"* `[VERIFIED]` | Stable, but tracks egui's own churn `[INFERENCE]` | **Stalled — the worst kind of risk** `[INFERENCE]` | Small crates, fast-moving; `bevy_declarative` is single-maintainer + unpublished `[INFERENCE]` |

---

## 6. Recommendation

### 6.1 The call

**Build the UI on Bevy 0.18's built-in `bevy_ui`, with `experimental_bevy_ui_widgets` enabled, driven exclusively through `bevy_picking` observers, and wrapped in a thin project-local styling builder.**

```toml
bevy = { version = "0.18", default-features = false, features = ["2d", "experimental_bevy_ui_widgets"] }
```

with these standing rules:

1. **All interaction through `Pointer<...>` observers.** `Pointer<Click>` for taps, `Pointer<Press>`/`Pointer<Release>` for held ability buttons, `Pointer<Drag>` for the tower scroll. **`Interaction` and `InteractionPalette` are banned** — they are single-touch-only ([§1.1](#11-bevy_ui-built-in-018)). This is the one place dot-tower must *not* copy guild-forge.
2. **Pixel-art baseline from day one:** `Camera2d` gets `UiAntiAlias::Off` and `Msaa::Off`; every `TextFont` gets `FontSmoothing::None`.
3. **One layout, conditional chrome** ([§3.2](#32-one-layout-with-conditional-chrome-or-two-layouts)). Branch on window aspect ratio, never on `target_os`. Frame in `Percent`/`VMin`, chrome in `Px`.
4. **Write our own `SafeAreaInsets`** from `AndroidApp::content_rect()`, and inset all edge-docked touch targets by `max(safe_area, ~8dp)` — which also mitigates the edge-coordinate bug #7528 ([§1.5](#15-safe-areas-nobody-has-them)).
5. **Confine `bevy_ui_widgets` behind local wrappers**, exactly as bevy-shenji does — use it for `Scrollbar`, `Slider`, `Checkbox`, `RadioGroup`, and `Button`'s `Activate` event; hand-roll anything trivial.

### 6.2 Why

`[INFERENCE]`

- **It wins the criterion the ticket says matters most, outright.** The touch chain is verified end-to-end in 0.18 source with no shim, and the widgets crate contains no mouse-only code path at all. Nothing else beats it here; two options merely tie.
- **It is free.** `features = ["2d"]` already compiles `bevy_ui`, `bevy_ui_render`, `bevy_picking` and `ui_picking` ([§1.0](#10-what-we-already-have--the-feature-accounting)). Every alternative is a dependency *added on top of* a UI layer we ship regardless. For a solo project shipping to a store, dependency count is a real cost.
- **It is the only option with a first-party migration path.** Bevy 0.19 is already out and 0.20 will follow. In-tree code comes with an official migration guide every release; the 0.17→0.18 guide has no breaking `bevy_ui_widgets` entry at all. `bevy_lunex` has not made the 0.19 jump in two months.
- **It is the only option that supports the pixel-art constraint natively.** `FontSmoothing::None` exists for exactly this and Bevy's own docs spell out the three-part recipe. egui cannot do this at all.
- **It matches what the user already does.** Both `bevy_immediate` and `bevy_declarative` are wrappers over this same stack — so this recommendation is not a departure from their practice, it is naming the substrate their practice already sits on. Whether a builder goes on top is a separate, reversible, ergonomics-only decision.

### 6.3 On the styling builder

`[INFERENCE]` **Recommended: hand-roll a small local builder in the `bevy_declarative` shape rather than adding a dependency**, but treat this as low-stakes and revisit once `bevy_declarative` is reachable.

- The used surface in guild-forge is ~25 methods ([§2.3](#23-guild-forge-bevy-018-local-bevy_declarative)) over `Node`/`TextFont`/`BackgroundColor`/`BorderRadius`. That is a day of work, has no version-skew risk, and can be extended with the two things dot-tower actually needs and neither existing wrapper has: **viewport-unit helpers** (`.vw()`, `.vh()`, `.vmin()`) and **safe-area-aware padding**.
- Do **not** block on `bevy_declarative` being on the other PC, and do not adopt `bevy_immediate 0.6.x` just to get a builder — it pulls `bevy_feathers` transitively, whose whole purpose is a themed desktop look we do not want.
- `docs/STYLING.md` in bevy-shenji is the specification to work from. Its palette/spacing/typography scales port directly; its widget list is a superset of what an idle game needs.

### 6.4 Risks, and what would change the call

| Risk | Severity | Mitigation / trigger to revisit |
|---|---|---|
| `bevy_ui_widgets` API churn — the crate says "likely to change substantially" `[VERIFIED]` | Medium | Confine behind local wrappers (bevy-shenji's proven pattern). In-tree = official migration guide each release. Only 8 modules. |
| **No safe-area API anywhere; `content_rect()` may not account for display cutouts** `[VERIFIED]` / `[INFERENCE]` | **High** — a notch over the gold counter is a shipping bug | Build `SafeAreaInsets` from `content_rect()`; **validate on a notched device during ticket 04**. If `content_rect` proves cutout-blind, fall back to JNI `WindowInsets.getDisplayCutout()`. |
| Touch misreported near edges (#7528, open since 2023) `[VERIFIED]` | Medium | Inset all edge-docked targets. Never place the only hit region of an ability button flush to the bezel. |
| **Pixel fonts blur at fractional device scale factors** `[VERIFIED]` (mechanism) / `[INFERENCE]` (severity) | **High** — it is the ticket's stated known trap | Fix A (`UiScale` pinned to an integer grid). Trigger for Fix B (`bevy_image_font` for HUD numerals only): Fix A fails on a real device. **Untested — verify during ticket 04.** |
| Font atlas churn if sizes are computed dynamically `[VERIFIED]` from doc | Low | Fixed typography constants only. Never derive `font_size` from viewport size. |
| Bevy 0.19 already out; we are a version behind | Low | The recommendation *reduces* this risk — in-tree code migrates with the engine. Revisit the 0.19 bump after ticket 04, not before. |
| `bevy_declarative` unavailable | Low | Reimplement ~25 methods, or write raw `Node` first. Not on the critical path either way. |
| UI nodes flicker on some Android devices (#14710, open) `[VERIFIED]` | Unknown | Device-specific; nothing to do pre-emptively. Watch for it during ticket 04. Note it affects `bevy_ui` specifically — if it bites hard, that is the one finding that would reopen this decision. |

`[INFERENCE]` **What would genuinely change the recommendation:** only two things. (a) `bevy_ui` rendering proving unreliable on target Android hardware (#14710) — in which case egui becomes the pragmatic fallback despite the aesthetic cost, since it renders through its own pipeline. (b) The pixel-font problem proving unsolvable within `bevy_text` *and* `bevy_image_font`'s no-wrapping limitation proving intolerable — a much less likely combination. Neither can be settled from source; both need ticket 04.

### 6.5 Open questions this research could not close

`[INFERENCE]` Stated plainly, because guessing here would be worse than admitting it:

1. **Does `AndroidApp::content_rect()` account for display cutouts, or only system bars?** The Android docs the issue thread links are themselves ambiguous, and the reporter of #23003 said the same (*"I'm not sure what it's supposed to be as even the official Android documentation is unclear"*). **Unknown. Needs a notched device.**
2. **What `UiScale` value actually produces crisp pixel text on a 2.625× phone?** The mechanism is verified; the value is arithmetic; whether the result *looks* right is not something source reading can establish.
3. **Whether `bevy_lunex` will resume.** Two months of silence after a 0.19 release is suggestive but not conclusive — the author has caught up before.
4. **`bevy_declarative`'s actual API beyond the ~25 methods guild-forge exercises**, including whether it has viewport units or any responsive support. Inferred from call sites only, as instructed.

---

## Sources

Bevy (all at tag `v0.18.0` unless noted):

- https://github.com/bevyengine/bevy/blob/v0.18.0/Cargo.toml — `2d`/`ui`/`picking`/`ui_api`/`ui_picking` collections; `experimental_bevy_ui_widgets`, `experimental_bevy_feathers`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui_widgets/src/lib.rs — experimental warning, module list, `UiWidgetsPlugins`, external state management
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui_widgets/src/button.rs — `Pointer<Press/Release/Click/Cancel/DragEnd>` observers
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui_widgets/src/slider.rs — `Pointer<Drag/DragStart/DragEnd/Press>`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui_widgets/src/scrollbar.rs — `Pointer<Press/Drag/DragStart/DragEnd/Cancel>`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_picking/src/input.rs — `touch_pick_events`, `PointerId::Touch`, `PointerInputSettings`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui/src/picking_backend.rs — pointer-generic UI hit testing
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui/src/focus.rs — `ui_focus_system`, `Touches` aggregates, `first_pressed_position()`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui/src/lib.rs — `UiPlugin`, `UiPickingPlugin` registration, `UiScale`, `UiSystems`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui/src/interaction_states.rs — `Pressed`, `Checked`, `InteractionDisabled`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui/src/geometry.rs — `Val::{Px,Percent,Vw,Vh,VMin,VMax}`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui/src/ui_node.rs — `Node` fields, `aspect_ratio`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_ui_render/src/lib.rs — `UiAntiAlias`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_text/src/text.rs — `FontSmoothing`, `TextFont::font_size` scaling + atlas-per-size note
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_window/src/window.rs — `WindowResolution::set_scale_factor_override`
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/ui/drag_to_scroll.rs — touch-native scrolling via `Pointer<Drag>` + `UiScale`
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/ui/ui_scaling.rs — `UiScale` usage
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/mobile/src/lib.rs — mobile example still uses `Changed<Interaction>`
- https://github.com/bevyengine/bevy/tree/v0.18.0/crates/bevy_feathers/src — bundled Fira fonts, `dark_theme.rs`, `palette.rs`
- https://bevy.org/news/bevy-0-18/ — release notes: `Popover`, `MenuPopup`, `AutoDirectionalNavigation`, font variations, pickable text sections, `IgnoreScroll`
- https://github.com/bevyengine/bevy-website/blob/main/content/learn/migration-guides/0.17-to-0.18.md — `bevy_ui_picking_backend` → `ui_picking`; `BorderRadius` onto `Node`; `LineHeight` split out; no breaking `bevy_ui_widgets` entry

Bevy issues:

- https://github.com/bevyengine/bevy/issues/23003 — safe area insets (open, `S-Blocked`), incl. maintainer comments on winit 0.31
- https://github.com/bevyengine/bevy/issues/11553 — multi-button partial touch release (open, 2024-01-27)
- https://github.com/bevyengine/bevy/issues/2333 — `ui_focus_system` touch handling (open, 2021-06-12)
- https://github.com/bevyengine/bevy/issues/7371 — `Interaction` lacks `just_pressed`/`just_released` (open)
- https://github.com/bevyengine/bevy/issues/5985 — no way to prevent touch propagation (open)
- https://github.com/bevyengine/bevy/issues/7528 — touch position misreported (open) [via ticket 01]
- https://github.com/bevyengine/bevy/issues/14710 — UI elements disappear on some Android devices (open)

Upstream:

- https://github.com/rust-windowing/winit/issues/3910 — "Finish safe area implementation" (open)
- https://github.com/rust-windowing/winit/issues/4506 — "Android: implement `Window::safe_area`" (open, 2026-03-07)
- https://github.com/rust-windowing/winit/issues/3911 — safe-area change event (open)
- https://github.com/rust-mobile/android-activity/blob/v0.6.1/android-activity/src/lib.rs — `AndroidApp::content_rect()` doc
- https://docs.rs/android-activity/0.6.1/android_activity/struct.AndroidApp.html — method list

Third-party crates:

- https://github.com/vladbat00/bevy_egui/blob/v0.39.1/src/input.rs — `write_window_touch_messages_system`, `EguiContextPointerTouchId`, `write_touch_message`
- https://github.com/vladbat00/bevy_egui/blob/v0.39.1/README.md — "Desktop and web platforms support"
- https://github.com/bytestring-net/bevy-lunex/blob/main/crate/src/picking.rs — `UiLunexPickingPlugin` as a `bevy_picking` backend
- https://github.com/bytestring-net/bevy-lunex/blob/main/crate/src/cursor.rs — `SoftwareCursor`, `MouseButtonInput`-only pointer synthesis
- https://github.com/bytestring-net/bevy-lunex/blob/main/crate/src/units.rs — `Ab`/`Rl`/`Rw`/`Rh`/`Em`/`Vp`/`Vw`/`Vh`
- https://github.com/bytestring-net/bevy-lunex/blob/main/README.md — "Any aspect ratio" claim
- https://github.com/ilyvion/bevy_image_font/blob/main/README.md — Bevy 0.18 support, `ImageFontPreRenderedUiText`, `ui`/`rendered`/`atlas_sprites` features, limitations
- https://api.github.com/repos/{bytestring-net/bevy-lunex, vladbat00/bevy_egui, PPakalns/bevy_immediate} — `pushed_at`, stars, open issue counts (read 2026-08-18)
- https://crates.io/api/v1/crates/{bevy_egui,bevy_lunex,bevy_immediate,bevy_immediate_core,bevy_immediate_ui,bevy_ui_widgets,bevy_feathers,bevy_image_font} and `/{version}/dependencies` — release dates and Bevy version mapping quoted throughout

This user's projects (read on this machine, 2026-08-18):

- /Users/tacit/dev/bevy-shenji/Cargo.toml, docs/STYLING.md, src/app_caps.rs, src/game/ui/layout.rs, src/theme/widgets/{slider,radio}.rs, src/theme/primitives/text.rs
- /Users/tacit/dev/shenji-bevy/Cargo.toml, src/main.rs, src/ui/systems.rs, src/ui/components.rs
- /Users/tacit/dev/guild-forge/Cargo.toml, src/main.rs, src/screens/sidebar.rs, src/theme/widgets.rs, src/theme/interaction.rs (referenced), and all `bevy_declarative` call sites under src/screens/ and src/ui/

Prior research:

- /Users/tacit/dev/dot-tower/.scratch/dot-tower/research/01-bevy-android-support.md — touch-only input on Android, winit's `// TODO mouse events`, `2d` feature completeness, issues #7528/#23003/#14710
