# Target Bevy 0.18 or 0.19?

Research for ticket `.scratch/dot-tower/issues/13-bevy-version-target.md`.

- **Date of investigation:** 2026-08-18
- **Versions read:** `bevy` `v0.18.1` tag and `v0.19.1` tag (source diffed side by side), plus `bevy` `main` at commit `a5cbc9e`. `wgpu` `v29.0.0` CHANGELOG. `bevy-website` `main`.
- **Transitive versions read:** `winit 0.30` (both), `android-activity 0.6` (both), `wgpu 27` (0.18) vs `wgpu 29.0.3` (0.19), `accesskit_winit 0.29` → `0.32`, `bevy_image_font 0.11.0`.
- **Method:** reading primary source at pinned tags via `raw.githubusercontent.com`, the GitHub REST API for issue/PR/milestone/compare state, the crates.io API for release metadata, the official `0.18-to-0.19` migration guide source in `bevyengine/bevy-website`, and the Bevy 0.19 release blog. **Nothing was compiled or run on a device.** Every claim is tagged `[VERIFIED]` (read directly from a named primary source) or `[INFERENCE]` (my reasoning across sources).
- **Not consulted:** Bevy's Discord (not reachable from this environment). Physical Adreno hardware.

---

## Summary — recommendation

**Target Bevy 0.19 (pin `0.19.1`). Change the build config now, before any code exists.**

The decision does *not* rest on the Adreno crash — that question came back negative. It rests on a different Android bug that 0.19 does fix, and on the fact that 0.18 is already an abandoned branch.

1. **#22925 (Adreno `No map for format` crash) is NOT fixed in 0.19.** `[VERIFIED]` The issue is still `open`, still `P-Crash` / `S-Needs-Investigation`, with zero linked PRs. It was **reproduced on `main` at commit `a5cbc9e` on 2026-04-08**, and that commit already carried `wgpu 29.0.1` — the same wgpu major line 0.19.1 ships. So this is not a "0.19 already fixed it" case; it is a "0.19 was tested and still crashes" case. This constraint from ticket 01 survives intact on both versions: **do not use custom `Material2d`.** See [§1](#1-issue-22925-adreno-material2d-crash).
2. **But 0.19 *does* fix #14710 — the bevy_ui-flickers-on-Android bug — transitively, and this is the real decider.** `[VERIFIED]` The root cause was traced to `gfx-rs/wgpu#8853` (Vulkan backend incorrectly skips barriers between render passes sharing a colour attachment — i.e. main 2D pass and the UI pass). It was fixed by wgpu PR #8924, whose body says verbatim that it "Resolves #8853 (and thus bevyengine/bevy#14710)". That PR merged 2026-03-15 and shipped in **wgpu 29.0.0** (2026-03-18). **Bevy 0.18 pins `wgpu = "27"`. Bevy 0.19 pins `wgpu = "29.0.3"`.** A reporter confirmed the patched wgpu eliminated flickering across four Android devices. This bug hits *exactly* our stack — bevy_ui + sprites + Mesh2d-with-texture on Android — and 0.18 can never get the fix, because it is pinned to a wgpu major that predates it. See [§3.4](#34-14710-ui-flicker-on-android--fixed-in-019-via-wgpu-29).
3. **0.18 is a dead branch.** `[VERIFIED]` `0.18.1` (2026-03-04) is the only patch 0.18 ever received, and nothing has landed on it in 5½ months while 0.19.1 shipped 58 bugfix commits. Bevy does not backport to previous minors. Choosing 0.18 means choosing a version that will receive no further fixes, ever.
4. **`experimental_bevy_ui_widgets` was stabilised.** `[VERIFIED]` Renamed to `bevy_ui_widgets`, folded into the `ui` feature collection, and `UiWidgetsPlugins` + `InputDispatchPlugin` are now in `DefaultPlugins`. It also gained `list`, `scrollarea` and `text_input` widgets. Our locked UI decision gets *cheaper and less experimental* on 0.19.
5. **The build config must change, and not in the obvious way.** `[VERIFIED]` In 0.19, `2d` no longer implies `ui`, no longer implies `audio`, and `default_platform` no longer implies `android-game-activity` or `android_shared_stdcxx` (the latter was deleted outright). **`features = ["2d"]` on 0.19 gives you a project with no `bevy_ui` at all and no Android activity backend.** This directly invalidates finding #2 of research 01 ("`features = ["2d"]` is already Android-complete"). New config in [§2.1](#21-the-cargo-config-must-change).
6. **The two locked Android decisions are unchanged by 0.19 — verified, not assumed.** `AppLifecycle`'s enum and the `WillSuspend → Suspended` state machine in `bevy_winit/src/state.rs` are functionally byte-identical between the two tags. `WillSuspend`/`WillResume` are **still never delivered** in 0.19. `Suspended`-as-save-hook still works, for the same reason it worked in 0.18. See [§2.5](#25-android-lifecycle--applifecycle).
7. **The `Interaction` ban stands.** `[VERIFIED]` `crates/bevy_ui/src/focus.rs` at `v0.19.1` still calls `touches_input.any_just_pressed()`, `any_just_released()` and `first_pressed_position()` — the exact global-aggregate calls that make #11553 (multi-touch) broken. #11553 is still `open`. Nothing relaxes here. See [§2.4](#24-interaction-and-11553--still-broken).
8. **Android platform primitives did not move.** `[VERIFIED]` Both 0.18.1 and 0.19.1 depend on `winit 0.30` and `android-activity 0.6`. #7528 (edge touch coordinates), #23003 (safe-area insets), #4506 (winit Android `safe_area`) are all still open and, in #23003's case, still `S-Blocked` on winit 0.31 — which is still in beta. The hand-rolled `AndroidApp::content_rect()` safe-area plan from research 02 is still required. See [§3](#3-android-specifically).
9. **Ecosystem cost: `bevy_image_font` has no 0.19 release.** `[VERIFIED]` `0.11.0` (2026-02-20, Bevy 0.18) is the newest; last repo push 2026-02-20; no "update to 0.19" issue has even been filed. This is the one genuine cost of going to 0.19 — but it is a *fallback* dependency, not a load-bearing one, and 0.19's new `FontSize::Rem` + `RemSize` gives a cleaner version of the Fix A (pin the effective scale) that research 02 already preferred over `bevy_image_font`. See [§4](#4-ecosystem-readiness).
10. **Timing is on 0.19's side.** `[VERIFIED]` Bevy's own README says a breaking release lands "approximately once every 3 months"; the measured gaps for the last six minors are 105–159 days (median ≈ 146). 0.19.0 shipped 2026-06-19, so 0.20 is most likely **November 2026 – January 2027**. The 0.20 milestone is 78/118 closed with no due date. Starting on 0.18 means paying *two* migrations (0.18→0.19→0.20) instead of one. See [§5](#5-stability-posture).

### What changes in the project's locked decisions

| Locked decision | Verdict | Action |
| --- | --- | --- |
| `bevy = { version = "0.18", default-features = false, features = ["2d"] }` | **Invalidated** | Retarget to `0.19.1`; feature list must add `ui` and `android-game-activity`. |
| Research 01 finding #2: "`features = ["2d"]` is already Android-complete" | **Invalidated for 0.19** | True on 0.18, false on 0.19. |
| Avoid custom `Material2d` (#22925) | **Confirmed, unchanged** | Keep the constraint. Neither version fixes it. |
| `bevy_ui` + `experimental_bevy_ui_widgets` via `bevy_picking` observers | **Confirmed, strengthened** | Feature renamed to `bevy_ui_widgets`; drop the manual `UiWidgetsPlugins` add — it is in `DefaultPlugins` now. |
| `Interaction`/`InteractionPalette` banned (#11553) | **Confirmed, unchanged** | Keep the ban; defect still in 0.19.1 source. |
| `AppLifecycle::Suspended` as synchronous save hook; `WillSuspend`/`WillResume` never delivered | **Confirmed, unchanged** | Keep the design exactly as charted. |
| `UiAntiAlias::Off` + `Msaa::Off` + `FontSmoothing::None` | **Confirmed, unchanged** | All three still exist in 0.19.1 with the same semantics. |
| Hand-roll safe areas from `AndroidApp::content_rect()` | **Confirmed, unchanged** | #23003 still `S-Blocked`. |
| `bevy_image_font` as pixel-font fallback | **Weakened** | No 0.19 release. Treat as unavailable for now; prefer the `UiScale`/`RemSize` pinning route. |
| Save on `WindowFocused{focused:false}` as earlier trigger | **Confirmed** | Unaffected. |

**New risk introduced by 0.19:** `bevy_text` swapped its text engine from `cosmic-text` to **Parley** (PR #22879). This is the single largest change to a subsystem we depend on. It is mostly invisible at the API level, but it is a whole-engine swap in its first release. Mitigations and the one real caveat are in [§2.6](#26-text-the-parley-swap).

---

## 1. Issue #22925 (Adreno `Material2d` crash)

### 1.1 Current status — verified

`[VERIFIED]` GitHub REST API, `repos/bevyengine/bevy/issues/22925`, read 2026-08-18:

```
state:        open
state_reason: null
closed_at:    null
created_at:   2026-02-12
comments:     5
labels:       C-Bug, A-Rendering, P-Crash, O-Android, S-Needs-Investigation
milestone:    null
```

Title: *"Crash on certain Android devices with adreno gpus with `No map for format` errors when using custom Material2d"*.

**It is open. It has no milestone. It is not assigned to any release.** No fix landed in 0.19.0 or 0.19.1.

`[VERIFIED]` A GitHub code/issue search for `repo:bevyengine/bevy 22925 in:body,title` returns `total_count: 0` — **no PR anywhere in the Bevy repository references this issue.** Nobody has attempted a fix.

### 1.2 The decisive detail: it was tested against the 0.19 line and still crashes

This is the part that settles the question, and it is easy to miss.

`[VERIFIED]` Comment by the reporter (`leomeinel`) on 2026-04-08 ([#issuecomment-4206344181](https://github.com/bevyengine/bevy/issues/22925#issuecomment-4206344181)):

> This now happens on current main if using the mobile example: `bevyengine/bevy@a5cbc9e6a3b49775964b2c43e857888295aac4fd`

`[VERIFIED]` `crates/bevy_render/Cargo.toml` at that exact commit `a5cbc9e`:

```toml
wgpu = { version = "29.0.1", ... }
wgpu-types = { version = "29.0.1", ... }
naga = { version = "29.0.1", features = ["wgsl-in"] }
```

`[VERIFIED]` `crates/bevy_render/Cargo.toml` at `v0.19.1`: `wgpu = { version = "29.0.3", ... }`.

`[INFERENCE]` The reproduction was therefore performed on the same wgpu major line (29) that Bevy 0.19.1 ships, on Bevy `main` two months before 0.19.0 was tagged, using Bevy's *own* mobile example rather than a third-party crate. There is no plausible reading in which 0.19 silently fixed this.

Note also that the repro moved from "third-party crate using custom `Material2d`" to "Bevy's own mobile example", which widens the blast radius somewhat — though the mobile example was itself being rewritten at the time (PR #23491, still open), so `[INFERENCE]` it is not certain the example repro is the same defect.

### 1.3 Workarounds discussed

`[VERIFIED]` From the issue body, the reporter's own successful workaround:

> after I have switched to a custom render pipeline and did not use any custom `Material2d`s it worked flawlessly.

`[VERIFIED]` The crate that triggers it is tiny — `bevy_fast_light@0.2.0`, 343 SLoC — and the same crash was independently produced by `bevy_lit`. Two independent crates, one shared ingredient: a custom `Material2d`.

`[VERIFIED]` One hypothesis was raised and then rejected. `beicause` (2026-04-08) suggested the Adreno 660 driver bug `gfx-rs/wgpu#5318`; `Elabajaba` (2026-05-12) replied:

> That issue is webgl2 only, while this seems to be happening on native.

That 2026-05-12 reply is the **last activity on the issue** — three months of silence.

### 1.4 What this means for dot-tower

`[INFERENCE]` The constraint from research 01 is unchanged and applies to *both* candidate versions: build the 2D look out of `Sprite`, `Mesh2d` + built-in `ColorMaterial`, and `bevy_ui`, and do not write a custom `Material2d`. For a pixel-art idle game this is not a real sacrifice — there was no lighting or shader-effect requirement in the charter. If a custom 2D shader ever becomes necessary, the reporter's escape hatch (a custom render pipeline / a `Node` in the render graph, bypassing `Material2d`) is the documented workaround.

**#22925 does not decide this ticket.** It is a wash between the two versions.

---

## 2. Migration cost 0.18 → 0.19

Since no code exists, "migration cost" is really "what does the starting config and first-day API look like". The full guide is 103 entries / 2342 lines; below are only the ones touching our surface.

Source for all of §2: `bevyengine/bevy-website`, `content/learn/migration-guides/0.18-to-0.19.md` @ `main`, plus the corresponding source at the two Bevy tags.

### 2.1 The Cargo config must change

This is the highest-value finding in the section, because it is silent — the wrong config compiles and simply lacks features.

`[VERIFIED]` `Cargo.toml` at `v0.18.1`, line 131:

```toml
2d = ["default_app", "default_platform", "2d_bevy_render", "ui", "scene", "audio", "picking"]
```

`[VERIFIED]` `Cargo.toml` at `v0.19.1`, line 137:

```toml
2d = ["default_app", "default_platform", "2d_bevy_render", "scene", "picking"]
```

`ui` and `audio` are gone. Two migration entries explain why:

> **`ui` feature is now no longer implied by the `3d` or `2d` features** (PR #23180) — "Swapping the UI framework for your Bevy project is a common form of customization… the `ui` feature collection is now no longer implied by the `3d` or `2d` feature collection."

> **`audio` feature is now no longer implied by the `3d`, `2d`, or `ui` features** (PR #23126) — `audio` is now a top-level default instead.

`[VERIFIED]` And `default_platform` changed too. At `v0.18.1` (line 192) it contained `"android-game-activity"` and `"android_shared_stdcxx"`. At `v0.19.1` (line 182) it contains neither; it gained `bevy_clipboard` and `custom_cursor`.

> **Remove android game activity from default** (PR #23708) — "Both options are no longer part of `default-features`, but they need to be added explicitly… For apps using `GameActivity` you need to add the `android-game-activity` feature."

> **Rodio 0.22 Update** (PR #20323) — "The `android_shared_stdcxx` feature was removed, as `cpal`'s `oboe-shared-stdcxx` feature was also removed in favor of Android NDK audio APIs. Keep in mind that if you are using `bevy_audio` the minimum supported Android API version is now 26 (Android 8/Oreo)."

`[VERIFIED]` `android_shared_stdcxx` no longer appears anywhere in `Cargo.toml` at `v0.19.1` (present at `v0.18.1` line 553).

`[VERIFIED]` One trap ruled out: could `2d`'s `picking` collection pull `bevy_ui` back in by accident? No. `picking = ["bevy_picking", "mesh_picking", "sprite_picking", "ui_picking"]`, and in `crates/bevy_internal/Cargo.toml` line 350: `ui_picking = ["bevy_picking", "bevy_ui?/bevy_picking", "bevy_input_focus?/bevy_picking"]`. The `bevy_ui?/` weak-dependency syntax means it activates the feature **only if `bevy_ui` is already enabled by something else**. With `features = ["2d"]` alone on 0.19, `bevy_ui` is genuinely absent.

**Concretely:**

```toml
# 0.18 (charted)
bevy = { version = "0.18", default-features = false, features = ["2d"] }

# 0.19 (recommended)
bevy = { version = "0.19", default-features = false, features = [
  "2d",
  "ui",                      # NEW: no longer implied by 2d
  "android-game-activity",   # NEW: no longer in default_platform
] }
# add "audio" if/when we want bevy_audio; it is no longer implied either.
# do NOT add "android_shared_stdcxx" — the feature no longer exists.
```

`[VERIFIED]` `ui` in 0.19 expands to `["default_app", "default_platform", "ui_api", "ui_bevy_render", "scene", "picking", "bevy_ui_widgets"]` — so `bevy_ui_widgets` comes free with `ui`, and `bevy_input_focus` comes via `ui_api`.

`[INFERENCE]` `android-game-activity` is harmless to list unconditionally: it resolves to `bevy_winit/android-game-activity` → `winit/android-game-activity`, which is a no-op on non-Android targets.

### 2.2 `experimental_bevy_ui_widgets` — stabilised and folded in

`[VERIFIED]` Migration entry, **`experimental_ui_widgets` feature is no longer experimental** (PR #22934), quoted in full:

> The `experimental_bevy_ui_widgets` feature has been renamed to `bevy_ui_widgets`.
>
> The `bevy_ui_widgets` feature has been added to the `ui` feature collection (and thus `bevy`'s default features) for ease of use.
>
> This crate remains immature, and is subject to heavy breaking changes, even relative to Bevy's pre-1.0 standards. However, it is useful enough to see wider adoption, and this change substantially improves the user experience when setting up new projects and running Bevy examples.

Note the honest caveat: **stabilised in name and packaging, not in API stability.** Bevy is explicit that breaking changes will continue. `[INFERENCE]` This is fine for us — we are building on it from scratch and will migrate once at 0.20 regardless.

`[VERIFIED]` Migration entry, **`UiWidgetsPlugins` and `InputDispatchPlugin` are now in `DefaultPlugins`** (PR #23346):

```rust
// 0.18
App::new().add_plugins(DefaultPlugins, UiWidgetsPlugins).run();
// 0.19
App::new().add_plugins(DefaultPlugins).run(); // "Puff!"
```

`[VERIFIED]` Migration entry, **`Core` prefix removed from UI widget components** (PRs #23612, #23938): `CoreScrollbarThumb` → `ScrollbarThumb`, `CoreScrollbarDragState` → `ScrollbarDragState`, `CoreSliderDragState` → `SliderDragState`. Additionally `ScrollbarThumb` entities no longer carry a `Node`; layout is done by a new `update_scrollbar_thumb` system after `ui_layout_system`, and the only styling knobs are `ScrollbarThumb`'s new `border` / `border_radius` fields.

`[VERIFIED]` The widget set grew. `crates/bevy_ui_widgets/src/lib.rs` module list:

| `v0.18.1` | `v0.19.1` |
| --- | --- |
| button, checkbox, menu, observe, popover, radio, scrollbar, slider | button, checkbox, **list**, menu, observe, popover, radio, **scrollarea**, scrollbar, slider, **text_input** |

`[INFERENCE]` `list` and `scrollarea` are directly useful to an idle game with a relic/upgrade list in a portrait column. Getting them built-in is a small but real win.

### 2.3 `bevy_picking` — API unchanged, one feature-flag change

`[VERIFIED]` The migration guide contains **exactly one** `bevy_picking` entry, and it is about features, not API:

> **`bevy_picking` feature flag no longer includes `bevy_input_focus`** (PRs #22933, #22990) — "this functionality is now tied to the existing `bevy/ui_picking` feature, which is itself part of the `ui` feature collection. In most cases, you should add the `ui` feature collection to your project if you are using `bevy_ui`."

Since our config adds `ui`, this is a non-event.

`[VERIFIED]` The observer API our whole UI plan rests on is unchanged. `crates/bevy_picking/src/events.rs` @ `v0.19.1` still defines `Over`, `Out`, `Press`, `Release`, `Click`, `Move`, `DragStart`, `Drag`, `DragEnd`, `Cancel`, `Scroll`. `crates/bevy_picking/src/pointer.rs` @ `v0.19.1` still defines:

```rust
pub enum PointerId {
    #[default] Mouse,
    Touch(u64),
    Custom(Uuid),
}
```

`[INFERENCE]` The chain established in research 02 — Android touch → `TouchInput` → `PointerId::Touch(id)` → `Pointer<Click>` → widget — is intact in 0.19. Zero migration cost here.

### 2.4 `Interaction` and #11553 — still broken

`[VERIFIED]` `repos/bevyengine/bevy/issues/11553` ("Bevy UI multiple button partial touch release", opened 2024-01-27): `state: open`, no milestone, 2 comments, labels `C-Bug`, `A-Input`, `A-UI`.

`[VERIFIED]` The defect is still visible in the 0.19.1 source. `crates/bevy_ui/src/focus.rs` @ `v0.19.1` (368 lines) still contains:

- line 175: `mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();`
- line 187: `mouse_button_input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed();`
- line 208: `.first_pressed_position()`

All three are **global aggregates over all fingers**, exactly as research 02 documented for 0.18. `[INFERENCE]` Multi-touch through `Interaction` is still broken in 0.19. **The `Interaction` / `InteractionPalette` ban stays.**

### 2.5 Android lifecycle / `AppLifecycle`

`[VERIFIED]` `crates/bevy_window/src/event.rs`. The `AppLifecycle` enum at `v0.18.1` (line 459) and `v0.19.1` (line 461) are identical, variant for variant and doc-comment for doc-comment:

```rust
pub enum AppLifecycle { Idle, Running, WillSuspend, Suspended, WillResume }
```

`is_active()` is likewise identical. The only diff in the surrounding file is that `WindowEvent`'s `#[reflect(...)]` attribute gained `Message` — cosmetic.

`[VERIFIED]` `crates/bevy_winit/src/state.rs`. The lifecycle state machine is unchanged in behaviour:

| behaviour | `v0.18.1` | `v0.19.1` |
| --- | --- | --- |
| `suspended()` sets `lifecycle = WillSuspend` | line 496–498 | line 497–499 |
| `redraw_requested` flips `WillSuspend → Suspended`, forces `should_update = true`, then removes `RawHandleWrapper` from the primary window on Android | line 531 | line 532 |
| `redraw_requested` flips `WillResume → Running`, forces an update and a redraw | line 551 | line 553 |
| `Suspended` blocks redraw | line 724 | line 726 |

`[VERIFIED]` A full `diff` of the two files shows the only changes are unrelated refactors: `MessageWriter`-based `SystemState` replaced by an event-send path, `create_windows` gaining `.unwrap()`, `MouseWheel` now carrying a converted `phase`, and `requested_resume` becoming an `Option`. None touch the lifecycle transitions.

`[INFERENCE]` Therefore **everything research 01 established about 0.18's suspend behaviour transfers to 0.19 unchanged**:

- `WillSuspend` and `WillResume` are **still never observable by user code** — the runner overwrites them with `Suspended`/`Running` *before* it triggers the update. This is a source-level fact, not a bug report.
- `AppLifecycle::Suspended` is still delivered inside exactly one forced `app.update()` that runs while Android's Java main thread is blocked in `surfaceDestroyed`. The synchronous-save-on-`Suspended` design is still safe, and still budgeted at one frame.
- Bevy still has no `onPause` / `onStop` / `onSaveInstanceState` / `onDestroy` hook. The `WindowFocused{focused:false}` earlier-autosave recommendation still applies.

**ADR-0002 and the save-hook design need no change for 0.19.**

One adjacent 0.19 change worth knowing: `WindowPlugin` exit systems moved to `Last`.

> **`WindowPlugin` exit systems moved to `Last`** (PR #23624) — `close_when_requested`, `exit_on_all_closed`, `exit_on_primary_closed` all moved into `Last` "to prevent systems that run after `Update` and rely on windows existing from panicking on the last frame". They are also now in a new `ExitSystems` set.

`[INFERENCE]` This makes a desktop-side "save on quit" system easier to order correctly than it was in 0.18. Mildly in 0.19's favour.

### 2.6 Text: the Parley swap

This is the largest change to a subsystem we touch, and the main thing to be honest about.

`[VERIFIED]` Migration entry, **`bevy_text` migration from Cosmic Text to Parley** (PR #22879):

> `bevy_text` now uses Parley for its text layout. For the most part, this change should be invisible to users of `bevy_text` and Bevy more broadly. However, some low-level public methods and types (such as `FontAtlasKey`) have changed to map to `parley`'s distinct API.

Migration steps that matter to us:

> - System font discovery now requires you to enable the `bevy/system_font_discovery` feature. Users on Linux will need the `fontconfig` library for this.
> - The various methods for setting the fallback font … now return a `Result`.

`[INFERENCE]` We load a pixel font as an asset and never ask for a system font, so `system_font_discovery` stays off — which also means no `fontconfig` build dependency on Linux and no extra Android font plumbing. This is a non-issue for us and a build simplification.

`[VERIFIED]` The pixel-art trio is intact in 0.19.1:

- `FontSmoothing::None` still exists in `crates/bevy_text/src/text.rs` (line 1183) with a doc comment that still reads: *"Combine this with `UiAntiAlias::Off` and `Msaa::Off` on your 2D camera for a fully pixelated look."* The caveat *"Due to limitations of the underlying text rendering library, this may require specially-crafted pixel fonts to look good, especially at small sizes"* is present in **both** 0.18.1 and 0.19.1, word for word.
- `UiAntiAlias::{On, Off}` still exists in `crates/bevy_ui_render/src/lib.rs` (line 161) and is still read per-camera (line 785).

`[VERIFIED]` `TextFont`'s two most-used fields changed shape (PRs #22156, #22614):

```rust
// 0.18
TextFont { font: asset_server.load("x.ttf"), font_size: 35., ..default() }
// 0.19
TextFont { font: asset_server.load("x.ttf").into(), font_size: FontSize::Px(35.), ..default() }
```

`font` is now a `FontSource` (`Handle` or `Family`); `font_size` is now a `FontSize` enum.

`[VERIFIED]` And `FontSize` is genuinely richer — `crates/bevy_text/src/text.rs` @ `v0.19.1` line 487:

```rust
pub enum FontSize { Px(f32), Vw(f32), Vh(f32), VMin(f32), VMax(f32), Rem(f32) }
```

with `eval(logical_viewport_size, rem_size)`. Its doc comment retains the warning that matters to us: *"The vertical height of rasterized glyphs in the font atlas in pixels. **This is multiplied by the scale factor**, but not the text entity transform or camera projection."* and *"The viewport variants are not supported by `Text2d`."*

`[INFERENCE]` **This is a direct improvement for the pixel-font problem in research 02 finding #8.** `FontSize::Rem(n)` against a single global `RemSize` resource gives one knob to keep every text size an exact integer multiple of the pixel font's design size across devices — cleaner than the 0.18 approach of overriding `UiScale` or `WindowResolution::set_scale_factor_override` and hoping every call site cooperates. The underlying trap (scale factor still multiplies in) is unchanged, so the *fix* still has to be applied deliberately; 0.19 just gives a better place to apply it.

**Caveats, stated plainly:**

- `[VERIFIED]` Issue **#25385** (2026-08-13, open): "`FontSize` implements `Component` but does not function when used as a component" — `S-Needs-Design`. The type is half-wired. Use it inside `TextFont`, not as a standalone component.
- `[VERIFIED]` Issue **#25453** (2026-08-18, open, `S-Blocked`): "Text looks blurry/unsharp/bad unless the font size is absurdly large", filed against 0.19.1. **This is *not* a 0.19 regression.** The reporter states that `cosmic-text` produces output *"pixel-for-pixel identical to bevy for some letters"*, and maintainer `alice-i-cecile` concluded *"This appears to be an upstream issue at this point"* — in `swash`, the rasteriser both engines share. `[INFERENCE]` 0.18 has the same problem. It also barely applies to us: it concerns antialiased vector-font rendering, and we render a pixel font with `FontSmoothing::None`.
- `[VERIFIED]` Other 0.19 text-related bugs (`FontAtlas`, `PositionedGlyph`, `Measure`, `TextRoot`→`TextSection`) are all low-level API changes for people writing custom text widgets. We are not.

### 2.7 2D rendering — essentially nothing to migrate

`[VERIFIED]` A search of the whole migration guide for `Material2d`, `Mesh2d`, and `Sprite` produces only two hits, both internal:

- `Core2dSystems`' `PostProcess` set was split into `EarlyPostProcess` + `PostProcess`, and 2D gained a `Prepass`.
- `Mesh2dPipelineKey` gained `STRIP_INDEX_FORMAT_*` bits (a wgpu 29 requirement).

`[INFERENCE]` For a game using `Sprite`, `Mesh2d` with built-in materials, and `Camera2d`, the 2D API surface is unchanged between 0.18 and 0.19. Migration cost here is zero — which is what you would hope, given the 0.19 release notes list "Unified 2D and 3D rendering internals" as a **0.20** goal, not a shipped one.

### 2.8 Broader breaking changes we will meet on day one

Not scoped to our subsystems, but unavoidable when writing the first file. `[VERIFIED]`, all from the migration guide:

- **Resources as Components** — the single largest ECS change in 0.19 (opens the guide, 200 lines).
- **`Replace` renamed to `Discard`**; `ComponentHooks::on_replace` → `on_discard`; `#[component(on_replace = …)]` → `#[component(on_discard = …)]`.
- **`DefaultErrorHandler` renamed to `FallbackErrorHandler`.**
- **`set_executor` replaced `ExecutorKind`.**
- **`bevy_scene` is now `bevy_world_serialization`** (the `scene` feature collection = `["bevy_world_serialization", "bevy_scene"]`).
- **`bevy_reflect` reorganised** to de-clutter the crate root.
- **BSN (Bevy Scene Notation)** is the headline 0.19 feature — a new way to declare scenes and, per the release notes, the direction Feathers widgets are moving in. `[INFERENCE]` We are not obliged to adopt BSN, but starting on 0.19 means our hand-rolled UI builder is written *alongside* the idiom Bevy is converging on, rather than one release behind it.

`[INFERENCE]` With zero existing code, all of §2.8 costs nothing — we simply write against 0.19 semantics from the start. This is precisely why the ticket's framing is right: the question is which foundation, not how expensive the move.

---

## 3. Android specifically

### 3.1 The stack did not move

`[VERIFIED]` Dependency versions read directly from crate manifests at both tags:

| Dependency | `v0.18.1` | `v0.19.1` |
| --- | --- | --- |
| `winit` (`crates/bevy_winit/Cargo.toml` line 66) | `"0.30"` | `"0.30"` |
| `android-activity` (`crates/bevy_android/Cargo.toml` line 12) | `"0.6"` | `"0.6"` |
| `wgpu` (`crates/bevy_render/Cargo.toml`) | `"27"` | `"29.0.3"` |
| `accesskit_winit` | `"0.29"` | `"0.32"` |

**winit and android-activity are unchanged. wgpu jumped two majors.** That asymmetry explains everything in this section: the *windowing/input* Android issues are all still open, and the *rendering* Android issue got fixed for free.

### 3.2 #7528 — touch coordinates near screen edges: still open, still stale

`[VERIFIED]` `repos/bevyengine/bevy/issues/7528`: `state: open`, created 2023-02-06, **last updated 2023-02-06**, `comments: 0`, labels `C-Bug`, `O-Android`. Three and a half years with zero activity.

`[VERIFIED]` The issue body (quoting winit maintainer `rib`) identifies the cause as winit's conflated notion of a window's "inner" size:

> On Android that "inner" size should probably be the inset size … Here's the Winit issue for this: `rust-windowing/winit#2308`

`[INFERENCE]` Since Bevy 0.19 is on the same `winit 0.30` as 0.18, **nothing changed**. This remains a live Android risk for touch targets near screen edges, and it is the same shape of problem as the safe-area gap (§3.3) — both stem from winit reporting an un-inset window size on Android. `[INFERENCE]` Design implication unchanged from research 02: keep interactive touch targets away from the extreme screen edges, and derive the usable rect from `AndroidApp::content_rect()` rather than trusting window size.

### 3.3 #23003 — safe-area insets: still open, still `S-Blocked`

`[VERIFIED]` `repos/bevyengine/bevy/issues/23003`: `state: open`, created 2026-02-17, labels `C-Feature`, `A-Windowing`, `A-UI`, `O-Android`, `O-iOS`, **`S-Blocked`**, `X-Uncontroversial`, `D-Straightforward`.

`[VERIFIED]` The full comment thread (5 comments, all Feb 2026) establishes the blocker precisely:

- `xremming`: winit logs `WARN winit::platform_impl::android: TODO: handle Android InsetsChanged notification`.
- `mockersf`: "related to `rust-windowing/winit#3910` — available on iOS but not yet on Android."
- `mockersf`, on an iOS-first PR: "A PR for it would be welcomed!"
- `xremming`, the decisive one: **"Looked into it *but* it requires `winit 0.31`, which is still in beta (last release is `0.31.0-beta.2 (2025-11-16)`). I tried upgrading `bevy_winit` to it but (surprise surprise) it's not trivial. When `winit 0.31` is released and part of Bevy I can take a new look into this."**

`[VERIFIED]` Bevy 0.19.1 is on `winit 0.30`. `[INFERENCE]` The blocker is therefore intact: **0.19 does not unblock safe areas, and could not have.** The research-02 plan — build the safe-area rect by hand from `bevy::android::ANDROID_APP` → `AndroidApp::content_rect()` — is still the only option, on either version.

`[VERIFIED]` The same thread also notes that the `bevy_android` crate "is also missing from the documentation that is available at docs.rs (it's exposed as `bevy::android`)" — and there is a separate open issue for exactly that, **#24521** (2026-06-03, `C-Docs`, `A-Build-System`, `O-Android`, `O-iOS`): "Docs for `bevy::asset::io::android` are not generated". `[INFERENCE]` Expect to read `crates/bevy_android/src/lib.rs` directly rather than docs.rs. Unchanged between versions.

### 3.4 #14710 — UI flicker on Android: **fixed in 0.19 via wgpu 29**

This is the finding that decides the ticket, so here is the full chain, each link verified.

**Link 1 — the Bevy issue.** `[VERIFIED]` `repos/bevyengine/bevy/issues/14710`, "UI elements randomly disappear for some frames on specific android devices": `state: open`, created 2024-08-11, 35 comments, labels `C-Bug`, `A-Rendering`, `A-Windowing`, `A-UI`, `O-Android`, `S-Needs-Investigation`. Reproduced across Mali-G76 / Mali-G57 / Helio G99 devices on Bevy 0.18-dev. Reporter `analytik` narrowed it (2026-01-24): flicker occurs with **UI, sprites, and `ColorMaterial` meshes that have a texture**, but not with untextured `ColorMaterial` meshes; `bevy_egui` flickers too.

**Link 2 — the root cause.** `[VERIFIED]` `AlbinBernhardssonARM` (2026-01-09):

> This may be caused by `gfx-rs/wgpu#8853`, as wgpu is missing a barrier between render passes rendering to the same color attachment (such as main pass and UI).

**Link 3 — confirmed empirically before any fix landed.** `[VERIFIED]` `NiklasEi` (2026-01-24) asked the reporter to test a patched `wgpu-types`. `analytik` (2026-01-25):

> I am so very grateful, indeed that fixes things! Tested 4 different Android devices (phone, tablet, eink) and there's no flickering at all. Added egui, bevy_ui, text, sprites, mesh2d with a texture and with colormaterial, all displayed correctly.

**Link 4 — the wgpu issue and its fix.** `[VERIFIED]` `repos/gfx-rs/wgpu/issues/8853` — "Vulkan backend incorrectly skips barriers", labels `type: bug`, `area: correctness`, `backend: vulkan`: **`state: closed`, `state_reason: completed`, `closed_at: 2026-03-15`.**

`[VERIFIED]` `repos/gfx-rs/wgpu/pulls/8924` — "Move ordered usages to hals", **`merged: true`, `merged_at: 2026-03-15`**, base `trunk`. Its body states verbatim:

> **Connections** — Resolves #8853 (and thus `bevyengine/bevy#14710`)
>
> Currently, ordered usage is defined globally for all hals. According to #8853 this is problematic and the current ordered usages are not correct for Vulkan. This PR moves the ordered usages into the different hals. It also removes the two wrong ordered usages for Vulkan.

**Link 5 — which wgpu release carries it.** `[VERIFIED]` `wgpu` `CHANGELOG.md` at tag `v29.0.0`, under the header `## v29.0.0 (2026-03-18)`, in two places:

- under *Changes → Hal*: "Make ordered texture and buffer uses hal specific. By @NiklasEi in #8924."
- under *Bug Fixes → Vulkan*: "Remove incorrect ordered texture uses. By @NiklasEi in #8924."

`[VERIFIED]` crates.io release dates: `wgpu 27.0.0` 2025-10-01, `28.0.0` 2025-12-18, **`29.0.0` 2026-03-19**, `29.0.3` 2026-05-02.

**Conclusion.** `[INFERENCE]` The fix landed in wgpu 29.0.0. **Bevy 0.18 pins `wgpu = "27"` and 0.18 receives no further releases, so 0.18 can never get this fix. Bevy 0.19 pins `wgpu = "29.0.3"` and therefore has it.**

Caveat, stated plainly: `[VERIFIED]` **Bevy issue #14710 is still marked `open`** — nobody closed it after the wgpu fix shipped, and the last comment on it is from 2026-01-25, before the merge. So this is an inference from the wgpu maintainer's own words plus version arithmetic, **not** a Bevy maintainer's confirmation, and **not** something anyone has re-tested on released 0.19 on the affected hardware. `[INFERENCE]` The inference is strong — the PR author names the Bevy issue explicitly, and a device-level test of the same patch was confirmed by the original reporter — but it deserves verification on real hardware during ticket 04 (Android device bringup). **Treat "0.19 fixes the flicker" as high-confidence, not certain.**

Why this matters more than #22925 for us: `[INFERENCE]` #22925 requires a custom `Material2d`, which we have already decided not to write. #14710 requires only `bevy_ui` + textured sprites on Android — i.e. **the default state of this project on day one**, with no way to design around it.

### 3.5 Other open Android issues (unchanged between versions)

`[VERIFIED]` 42 open issues carry `O-Android`. The ones bearing on this project:

| # | Opened | Labels | Relevance |
| --- | --- | --- | --- |
| **#23754** | 2026-04-10 | `C-Bug`, `A-Rendering`, `A-Windowing`, `O-Android`, `S-Needs-Design`, `C-Machine-Specific` | "Bevy crashes on Google Pixel 10" — PowerVR D-Series GPU, running the mobile example on `main`. Open. A newer device-specific crash, unrelated to Adreno. |
| **#16798** | 2024-12-13 | `C-Dependencies`, `O-Android`, `S-Blocked` | "(Android) Crash when there's more than eight touches at once." `S-Blocked` on the dependency stack. `[INFERENCE]` A real multi-touch ceiling; unlikely to bite an idle game, but worth knowing given we deliberately support multi-touch via `PointerId::Touch`. |
| **#20638** | 2025-08-18 | `C-Bug`, `A-Rendering`, `A-Windowing`, `O-Android`, `O-iOS` | `Camera.computed.target_info` is `None` during `PostUpdate` only on mobile. `[INFERENCE]` Relevant if we compute layout from viewport size in `PostUpdate`. |
| **#11402** | 2024-01-18 | `C-Feature`, `A-Windowing`, `O-Android` | No API for Android immersive mode / hiding nav buttons. `[INFERENCE]` Relevant to a portrait-primary game; still hand-rolled via JNI or Gradle theme. |
| #24926 | 2026-07-09 | `A-Rendering`, `O-Android`, `A-Animation` | Adreno + skinned GLB corruption — 3D only, not our surface. |

`[INFERENCE]` None of these differ between 0.18 and 0.19; they are the standing cost of shipping Bevy on Android on either version. #23754 is worth noting only because it postdates 0.18 and shows the Android device matrix is still turning up crashes.

Also unchanged and still true from research 01: Bevy's example Gradle config targets an API level below Google Play's 2026-08-31 requirement of API 36. `[VERIFIED]` PR **#23491** ("examples: Use single up-to-date android example", opened 2026-03-24) is **still open** — the modernised Kotlin/GameActivity example with updated dependencies has not merged into either release. `[INFERENCE]` We will be writing our own Gradle config regardless.

---

## 4. Ecosystem readiness

### 4.1 `bevy_image_font` — no 0.19 release

`[VERIFIED]` crates.io metadata for `bevy_image_font`:

| Version | Published | Targets |
| --- | --- | --- |
| **0.11.0** | 2026-02-20 | Bevy 0.18 (`bevy_ui ^0.18`, `bevy_sprite ^0.18`, … — all deps `^0.18`) |
| 0.10.0 | 2026-01-06 | Bevy 0.17 |
| 0.9.0 | 2025-05-13 | Bevy 0.16 |

`[VERIFIED]` `github.com/ilyvion/bevy_image_font`: `pushed_at: 2026-02-20`, 12 stars, 2 open issues, not archived. Latest commits are the 0.11.0 release prep. **No branch, PR, or issue exists for a 0.19 update** — the issue list (all states, 15 most recent) contains "Update to bevy 0.18" (#26, closed) and "Update bevy 0.17" (#23, closed) but nothing for 0.19.

`[VERIFIED]` The historical lag pattern, measured from each Bevy release to the matching `bevy_image_font` release:

| Bevy | Released | `bevy_image_font` | Lag |
| --- | --- | --- | --- |
| 0.16.0 | 2025-04-24 | 0.9.0 @ 2025-05-13 | 19 days |
| 0.17.0 | 2025-09-30 | 0.10.0 @ 2026-01-06 | 98 days |
| 0.18.0 | 2026-01-13 | 0.11.0 @ 2026-02-20 | 38 days |
| 0.19.0 | 2026-06-19 | — | **60 days and counting** |

`[VERIFIED]` Both the 0.17 and 0.18 updates arrived as **external contributor PRs** (#24 and #27 from `GiantBlargg`), not maintainer-initiated work.

`[INFERENCE]` 60 days is inside the historical range (0.17 took 98), so this is a lagging crate rather than an abandoned one — but it is a one-maintainer, 12-star crate that depends on drive-by contributions to follow Bevy. It should not be treated as a dependency we can count on being there.

**Why this matters less than it looks.** Research 02 already ranked `bevy_image_font` as *Fix B* — the fallback for the pixel-font scaling problem, behind *Fix A* (pin the effective scale so `font_size × scale_factor` lands on an integer multiple). `[INFERENCE]` 0.19's `FontSize::Rem` + the `RemSize` resource make Fix A materially better than it was on 0.18 (§2.6). So 0.19 improves the option we prefer while temporarily removing the option we ranked second. On balance this is close to neutral, not a reason to stay on 0.18. If Fix A fails in practice on real hardware, the fallback is to port `bevy_image_font` ourselves — `[INFERENCE]` a small crate whose Bevy-facing surface is asset loading plus a sprite/UI render path, and whose 0.19 port would mainly mean absorbing the `TextFont`/`FontSize` and `FontAtlas` changes from §2.6.

### 4.2 `bevy_lunex` — the maintenance signal the ticket asked about

`[VERIFIED]` crates.io: latest is `0.6.0`, published **2026-01-22** (Bevy 0.18 era). Nothing since. Repo `bytestring-net/bevy-lunex`.

`[INFERENCE]` This confirms and sharpens research 02's read: `bevy_lunex` is now **two Bevy releases behind** (no 0.19 release, seven months quiet). It was already ruled out; this is simply further evidence the ruling was right. It is a signal about that crate, not about 0.19.

### 4.3 `bevy_egui` — the counter-example

`[VERIFIED]` crates.io: `0.40.0` published **2026-06-19 — the same day as Bevy 0.19.0** — followed by `0.40.1`, `0.41.0`, `0.41.1`, and `0.42.0` (2026-08-16).

`[INFERENCE]` We are not using it, but it establishes the baseline: a healthy Bevy ecosystem crate had 0.19 support on release day and five releases since. The ecosystem as a whole is on 0.19; `bevy_image_font`'s absence is a property of that crate, not of 0.19's readiness.

### 4.4 Anything else

`[INFERENCE]` Per the charter, the only third-party crates in scope were the UI candidates (all resolved in favour of built-in `bevy_ui`) and the pixel-font fallback. Serialisation (`ron`/`serde`) and save-file handling are Bevy-version-independent. Nothing else in the dependency plan is gated on the Bevy version. If `bevy_seedling` is ever considered for audio instead of `bevy_audio`, note that 0.19's feature split (§2.1) makes opting out of `bevy_audio` trivial — `[VERIFIED]` the migration guide names `bevy_seedling` explicitly as the motivation for that change.

---

## 5. Stability posture

### 5.1 Is 0.19.x settled?

`[VERIFIED]` Release history from crates.io and the GitHub releases API:

| Version | Date |
| --- | --- |
| 0.19.0-rc.1 | 2026-05-13 |
| 0.19.0-rc.2 | 2026-05-22 |
| 0.19.0-rc.3 | 2026-06-10 |
| **0.19.0** | **2026-06-19** |
| **0.19.1** | **2026-08-13** |

**One point release in two months.** `[VERIFIED]` Its scale, via `repos/bevyengine/bevy/compare/v0.19.0...v0.19.1`: **58 commits, 151 files changed.** Every commit message is a fix; there are no features. Notable entries touching our surface:

- **`Fix 2D flicker by always dequeueing retained phase items (#25163) (#25253)`** — see §5.2.
- `Another text measurement fix (#24669)`, `Text performance regression fix (#24663)`, `bevy_text: give swash a stable font id (#24710)` — Parley-migration follow-ups.
- `Fix mismatch in UI color conversions (#24886)`, `Don't cull rotated UI text glyphs (#24999)`, `Wrong UI camera fix (#24982)`, `implement Clone for IsDefaultUiCamera (#24729)`, `clip_check_recursive fix (#24684)`, `ImageMeasure border-box sizing fix (#24674)`, `Always centered radio marks (#24749)` — bevy_ui polish.
- `Fix handling of WindowResolution change (#24746)`, `Fix uninitialized-drawable pink screen on iOS (#25176)` — windowing/mobile.

`[INFERENCE]` This is the profile of a release **settling**, not churning: a dense single patch two months in, concentrated in exactly the areas 0.19 changed most (text/Parley and UI). The absence of a 0.19.2 in the five days since is not evidence either way.

`[VERIFIED]` A GitHub search for `repo:bevyengine/bevy is:issue is:open label:C-Regression` returns **`total_count: 0`** — no open regressions are tracked at all.

### 5.2 The one honest mark against 0.19.0

`[VERIFIED]` Issue **#25163**, "Sprite flickering", opened 2026-07-26, labels `C-Bug`, `A-Rendering`, **`P-Regression`**, filed against:

```toml
bevy = { version = "0.19.0", default-features = false, features = ["2d", "bevy_ui", "bevy_ui_render"] }
```

on Metal (M1 Pro), reproduced on iOS and macOS. **`state: closed`** — fixed by #25253, shipped in 0.19.1.

`[INFERENCE]` This is exactly the kind of thing to weigh: 0.19.0 shipped with a 2D sprite flicker regression in a config nearly identical to ours, and it took ~2.5 weeks from report to fix and ~7 weeks to reach a release. It argues for **pinning `0.19.1`, not `0.19`** — and it is also a reminder that a `.0` Bevy release is not the same as a `.1`. We have the luxury of starting on the `.1`.

### 5.3 0.18's posture, for comparison

`[VERIFIED]` 0.18.0 shipped 2026-01-13; **0.18.1 shipped 2026-03-04 and is the last 0.18 release.** Five and a half months with no further patch, while 0.19 received 58 fixes.

`[VERIFIED]` For historical contrast, 0.17 received three patches (0.17.1, 0.17.2, 0.17.3, the last on 2025-11-17 — i.e. after 0.18 was already in RC).

`[INFERENCE]` Bevy does not maintain old minors once the next one is out. **0.18 is a frozen branch.** Choosing it means choosing a version where every bug found from here on — including anything found during our own Android bringup — is permanently unfixed except by forking. That is a stronger argument than any single issue in this document.

### 5.4 Cadence and when 0.20 lands

`[VERIFIED]` Bevy's own `README.md` at `v0.19.1`, line 15:

> A new version of Bevy containing breaking changes to the API is released [approximately once every 3 months](https://bevy.org/news/bevy-0-6/#the-train-release-schedule). We provide migration guides, but we can't guarantee migrations will always be easy.

`[VERIFIED]` Measured gaps between consecutive `.0` releases (crates.io publish dates):

| From → To | Gap |
| --- | --- |
| 0.13.0 → 0.14.0 | 138 d |
| 0.14.0 → 0.15.0 | 148 d |
| 0.15.0 → 0.16.0 | 146 d |
| 0.16.0 → 0.17.0 | 159 d |
| 0.17.0 → 0.18.0 | 105 d |
| 0.18.0 → 0.19.0 | 157 d |

`[INFERENCE]` The stated 3 months (≈90 d) is optimistic; the lived median is ≈146 days (≈4.8 months), range 105–159. From 0.19.0 on 2026-06-19 that projects **0.20.0 somewhere between 2026-10-02 and 2026-11-25**, most likely **mid-November 2026**. Add an RC period of 3–5 weeks that gives useful early warning.

`[VERIFIED]` The **0.20 milestone** (`milestone/43`) was created 2026-03-30 and is `open` with **78 closed / 40 open (66%)**, **no due date set**. `[INFERENCE]` A 66%-complete milestone with no due date four months into the cycle is consistent with an autumn/early-winter release and gives no reason to expect an unusually early or late one.

`[VERIFIED]` The 0.19 release blog's "What's Next?" section lists for the next cycle: `.bsn` scene file assets, **"Unified 2D and 3D rendering internals" to improve 2D performance**, an entity inspector, assets-as-entities, WESL shaders, and a much more complete Bevy book. `[INFERENCE]` "Unified 2D and 3D rendering internals" is the 0.20 item most likely to affect us, and it is a reason to expect 0.20 to be a *meaningful* 2D migration rather than a trivial one — which is an argument for arriving at that migration from 0.19 rather than doing 0.18→0.19→0.20 back to back.

### 5.5 "Would starting on 0.19 mean migrating again soon?"

`[INFERENCE]` Yes — in roughly 3 months, and again every ~5 months after that. That is unavoidable on Bevy at any version. The question is how many migrations, not whether:

- **Start on 0.18:** two migrations before end of 2026 (0.18→0.19 now-or-later, then 0.19→0.20), on a frozen branch in the meantime, with a known Android UI flicker bug that can never be fixed.
- **Start on 0.19.1:** one migration (0.19→0.20, ~November 2026), on the actively-patched branch, with the flicker fix already in.

There is no version of this where starting on 0.18 involves fewer migrations.

---

## 6. Residual uncertainty — stated plainly

Things I could not establish, in descending order of how much they could change the recommendation:

1. **`[UNKNOWN]` Whether #14710 is actually fixed on real Android hardware running released 0.19.1.** The chain (§3.4) is: wgpu maintainer's PR body names the Bevy issue → the same patch was device-confirmed by the original reporter → the fix is in wgpu 29.0.0 → Bevy 0.19 pins wgpu 29.0.3. Each link is verified. But **no one has re-tested released 0.19 on an affected device**, and the Bevy issue is still open with its last comment predating the merge. This should be an explicit check in ticket 04 (Android device bringup).
2. **`[UNKNOWN]` Whether the Adreno crash in #22925 also affects non-`Material2d` code paths.** The 2026-04-08 comment says the crash "now happens on current main if using the mobile example", which — if it is the same defect — would be much worse than "avoid custom `Material2d`". But the mobile example was being rewritten in the same window (PR #23491), the issue is `S-Needs-Investigation` with nobody assigned, and no one has triaged that comment. **This is the single biggest open risk to Android delivery on either version**, and it is version-neutral. Worth a targeted test on an Adreno device early.
3. **`[UNKNOWN]` Whether `FontSmoothing::None` + a pixel font renders identically under Parley vs cosmic-text.** Both go through `swash` for rasterisation `[VERIFIED via #25453 discussion]`, and the `FontSmoothing::None` doc text is unchanged, so `[INFERENCE]` the rasterised output should be equivalent — but layout (advance widths, line breaking, baseline placement) is Parley's job now, not cosmic-text's, and small differences there are plausible. Cheap to verify with a screenshot comparison once there is a running app.
4. **`[UNKNOWN]` When `bevy_image_font` will support 0.19, if ever.** No signal either way; no issue filed.
5. **`[UNKNOWN]` Bevy's Discord.** Not reachable from this environment. It is where mobile-specific workarounds usually surface first, so the practitioner consensus on Adreno may exist there and is not reflected here.
6. **`[NOT DONE]` Nothing in this document was compiled or run.** Every claim is from reading source, metadata, or issue text.

---

## Sources

All URLs read on 2026-08-18.

**Bevy source, read at pinned tags**
- `Cargo.toml` @ [`v0.18.1`](https://raw.githubusercontent.com/bevyengine/bevy/v0.18.1/Cargo.toml) and [`v0.19.1`](https://raw.githubusercontent.com/bevyengine/bevy/v0.19.1/Cargo.toml) — feature collections `default`, `2d`, `ui`, `default_platform`, `picking`, `ui_picking`, `android-*`
- `crates/bevy_internal/Cargo.toml` @ `v0.19.1` — `ui_picking` weak-dependency wiring (line 350), `bevy_ui` (238), `android-game-activity` (154)
- `crates/bevy_winit/Cargo.toml` @ both tags — `winit = "0.30"` (line 66), `wgpu-types`, `accesskit_winit`
- `crates/bevy_android/Cargo.toml` @ both tags — `android-activity = "0.6"` (line 12)
- `crates/bevy_render/Cargo.toml` @ both tags and @ [`a5cbc9e`](https://raw.githubusercontent.com/bevyengine/bevy/a5cbc9e6a3b49775964b2c43e857888295aac4fd/crates/bevy_render/Cargo.toml) — wgpu 27 / 29.0.1 / 29.0.3
- `crates/bevy_winit/src/state.rs` @ both tags — lifecycle state machine (diffed in full)
- `crates/bevy_window/src/event.rs` @ both tags — `AppLifecycle` enum
- `crates/bevy_ui/src/focus.rs` @ `v0.19.1` — `ui_focus_system` global-aggregate touch reads
- `crates/bevy_text/src/text.rs` @ both tags — `FontSmoothing`, `FontSize`, `TextFont`
- `crates/bevy_ui_render/src/lib.rs` @ `v0.19.1` — `UiAntiAlias`
- `crates/bevy_ui_widgets/src/lib.rs` @ both tags — widget module list
- `crates/bevy_picking/src/events.rs`, `src/pointer.rs` @ `v0.19.1` — `Pointer` events, `PointerId`
- `README.md` @ `v0.19.1` line 15 — release cadence statement

**Migration guide & release notes**
- [`bevy-website` `content/learn/migration-guides/0.18-to-0.19.md`](https://raw.githubusercontent.com/bevyengine/bevy-website/main/content/learn/migration-guides/0.18-to-0.19.md) — 103 entries; PRs cited inline: #23180, #23126, #22488, #23708, #20323, #22934, #23346, #23612, #23938, #22933, #22990, #22879, #22156, #22614, #23012, #23568, #23605, #23624, #23723, #23878, #22789, #24049, #21931
- [Bevy 0.19 release blog](https://bevy.org/news/bevy-0-19/) — 261 contributors, 1185 PRs; "What's Next?" roadmap
- [GitHub releases](https://github.com/bevyengine/bevy/releases) — tag dates for v0.18.0 … v0.19.1
- [`compare/v0.19.0...v0.19.1`](https://github.com/bevyengine/bevy/compare/v0.19.0...v0.19.1) — 58 commits, 151 files

**Bevy issues & PRs (GitHub REST API)**
- [#22925](https://github.com/bevyengine/bevy/issues/22925) — Adreno `Material2d` crash — **open**, `P-Crash`, 5 comments
- [#11553](https://github.com/bevyengine/bevy/issues/11553) — multi-button partial touch release — **open**
- [#7528](https://github.com/bevyengine/bevy/issues/7528) — Android touch position — **open**, no activity since 2023-02-06
- [#23003](https://github.com/bevyengine/bevy/issues/23003) — safe-area insets — **open**, `S-Blocked` on winit 0.31
- [#14710](https://github.com/bevyengine/bevy/issues/14710) — UI flicker on Android — **open**, 35 comments
- [#25163](https://github.com/bevyengine/bevy/issues/25163) — Sprite flickering, `P-Regression` — **closed**, fixed in 0.19.1
- [#25453](https://github.com/bevyengine/bevy/issues/25453) — blurry text — **open**, `S-Blocked`, upstream in `swash`
- [#25385](https://github.com/bevyengine/bevy/issues/25385) — `FontSize` as `Component` non-functional — **open**
- [#23754](https://github.com/bevyengine/bevy/issues/23754) — Pixel 10 crash — **open**
- [#24521](https://github.com/bevyengine/bevy/issues/24521), [#16798](https://github.com/bevyengine/bevy/issues/16798), [#20638](https://github.com/bevyengine/bevy/issues/20638), [#11402](https://github.com/bevyengine/bevy/issues/11402), [#24926](https://github.com/bevyengine/bevy/issues/24926) — other open `O-Android` issues
- [#13038](https://github.com/bevyengine/bevy/issues/13038) — closed 2024 Android segfault, referenced by #22925
- [PR #23491](https://github.com/bevyengine/bevy/pull/23491) — modernised Android example — **open**
- [Milestone 0.20](https://github.com/bevyengine/bevy/milestone/43) — open, 78/118, no due date
- Search `repo:bevyengine/bevy is:issue is:open label:C-Regression` → 0 results
- Search `repo:bevyengine/bevy 22925 in:body,title` → 0 results

**wgpu**
- [gfx-rs/wgpu#8853](https://github.com/gfx-rs/wgpu/issues/8853) — "Vulkan backend incorrectly skips barriers" — **closed 2026-03-15, completed**
- [gfx-rs/wgpu#8924](https://github.com/gfx-rs/wgpu/pull/8924) — "Move ordered usages to hals" — **merged 2026-03-15**; body: "Resolves #8853 (and thus bevyengine/bevy#14710)"
- [`CHANGELOG.md` @ `v29.0.0`](https://raw.githubusercontent.com/gfx-rs/wgpu/v29.0.0/CHANGELOG.md) — `## v29.0.0 (2026-03-18)`, #8924 listed under *Hal* and *Bug Fixes → Vulkan*
- [gfx-rs/wgpu#5318](https://github.com/gfx-rs/wgpu/issues/5318) — Adreno 660 driver bug, raised and rejected as the cause of #22925

**crates.io release metadata** (via `crates.io/api/v1`)
- `bevy` — 0.18.0 (2026-01-13), 0.18.1 (2026-03-04), 0.19.0 (2026-06-19), 0.19.1 (2026-08-13); full `.0` history back to 0.10.0 for cadence
- `wgpu` — 27.0.0 (2025-10-01), 28.0.0 (2025-12-18), 29.0.0 (2026-03-19), 29.0.3 (2026-05-02)
- `bevy_image_font` — 0.11.0 (2026-02-20), deps all `^0.18`
- `bevy_lunex` — 0.6.0 (2026-01-22), nothing since
- `bevy_egui` — 0.40.0 (2026-06-19) … 0.42.0 (2026-08-16)

**Third-party repos**
- [`ilyvion/bevy_image_font`](https://github.com/ilyvion/bevy_image_font) — `pushed_at 2026-02-20`, 12 stars, issue list (all states)
- [`rust-windowing/winit#3910`](https://github.com/rust-windowing/winit/issues/3910), [`#2308`](https://github.com/rust-windowing/winit/issues/2308), `#4506` — cited from within Bevy issue threads (not fetched directly)
