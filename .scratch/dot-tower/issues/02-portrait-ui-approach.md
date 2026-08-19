# Portrait-first UI approach in Bevy 0.18

Type: research
Status: resolved

## Question

What is the right UI layer for a portrait-primary game that must also present a landscape desktop frame, on Bevy 0.18?

Answer specifically:

1. **The options.** Bevy's built-in `bevy_ui` in 0.18, `bevy_egui`, `bevy_lunex`, or something else current. For each: does it support touch input properly, does it handle safe areas and notches, and is it maintained against 0.18?
2. **Prior art in this repo's neighbourhood.** `/Users/tacit/dev/bevy-shenji` uses `bevy_immediate` and the `experimental_bevy_ui_widgets` feature; `/Users/tacit/dev/shenji-bevy` uses `bevy_egui` 0.38; `/Users/tacit/dev/guild-forge` uses a local `bevy_declarative` styling crate (hosted on the user's other PC, not present here). What did each choose and what does that suggest?
3. **Responsive layout.** What is the idiomatic way to express "one portrait tower column, with desktop spending extra width on side panels" — one layout with conditional chrome, or two layouts?
4. **Text legibility on a phone.** Bitmap/pixel fonts at varying DPI is a known trap in pixel-art games. What are the options?

Note the constraint: the game is pixel art, so UI must not fight the aesthetic with a smoothed vector look.

Capture findings as a Markdown file in the repo and link it from this ticket.

## Answer

Findings: [research/02-portrait-ui-approach.md](../research/02-portrait-ui-approach.md).

**Decision: built-in `bevy_ui` + `experimental_bevy_ui_widgets`, driven exclusively through `bevy_picking` observers**, with a thin local styling builder in the `bevy_declarative` shape (~25 methods).

1. **The options.** The touch criterion settles it. `bevy_picking`'s `touch_pick_events` reads `WindowEvent::TouchInput` directly and spawns one `PointerId::Touch(id)` **per finger**; `UiPlugin` registers its picking backend unconditionally; every `bevy_ui_widgets` widget is built purely on `Pointer<...>` observers, reading no `ButtonInput<MouseButton>` or `CursorMoved` anywhere. No touch shim needed. `bevy_ui` is also already compiled in — `features = ["2d"]` expands to include `ui` and `picking` → `ui_api`, `ui_bevy_render`, `ui_picking` — so every alternative is a dependency stacked on a UI layer already shipping. `experimental_bevy_ui_widgets` is one extra flag: 8 headless modules imposing zero styling, carrying a "likely to change substantially" warning but no breaking entry in the 0.17→0.18 migration guide. `bevy_lunex` is the one crate betting against: silent since 2026-02-24 and never updated for Bevy 0.19.
2. **Prior art converges on `bevy_ui`.** Three of four codebases land there: `bevy_immediate` and `bevy_declarative` are both *wrappers over it* — `bevy_declarative`'s `.on_click` takes `IntoObserverSystem<Pointer<Click>, ...>` and raw `BorderRadius`/`Button` pass straight through `.insert()`. Their method vocabularies are near-identical Tailwind-shaped builders. `bevy_egui` was tried in `shenji-bevy` and abandoned. But none of the three is portrait, touch-first, or pixel-art — that is all new ground.
3. **Responsive layout**: one layout with conditional chrome, branching on **window aspect ratio, never `target_os`** — so the portrait layout stays testable by resizing a desktop window. Frame in `Percent`/`VMin`, chrome in `Px`, since `UiScale` multiplies only `Val::Px` and that gives one clean knob.
4. **Text legibility**: `UiAntiAlias::Off` + `Msaa::Off` + `FontSmoothing::None` from day one. The blur mechanism is `font_size × scale_factor × UiScale`; `bevy_image_font` is the fallback, with real no-newline/no-wrap limits that happen not to bite for HUD numerals.

**The trap the prior art walks into.** `bevy_ui`'s legacy `ui_focus_system` consults `Touches` only through global aggregates — `first_pressed_position()`, `any_just_pressed()`, `any_just_released()` — not per finger. Two fingers down, releasing one releases everything (open issue #11553, still present in v0.18.0 source). **guild-forge's `InteractionPalette` pattern is exactly this and must not be copied**: four thumb-reachable ability buttons are precisely a multi-touch surface. `Interaction`-based UI is banned in dot-tower.

**Consequences for other tickets:**

- **Ticket 10 item 6 has a definite answer: nobody has safe areas.** #23003 is `S-Blocked` on winit 0.31 (still beta) and winit's Android impl is itself open (#4506). It is a wash across all options, so not a selection criterion — but it is ~40 lines we must write from `AndroidApp::content_rect()`. Insetting edge-docked targets also mitigates edge-coordinate bug #7528, buying two fixes at once.
- **Ticket 04 gains two must-verify-on-hardware items**: whether `content_rect()` accounts for display cutouts or only system bars, and what `UiScale` value actually yields crisp pixel text at ~2.625× phone density.
- **Bevy 0.19 shipped 2026-06-19 (0.19.1 on 2026-08-13)** while this project's premises assume 0.18. Surfaced as ticket 13.

Two things could reopen this, neither resolvable by reading: `bevy_ui` proving unreliable on target Android hardware (#14710), or the pixel-font problem defeating both `UiScale` tuning and `bevy_image_font`.

Status: resolved

## Amended by ticket 13

On 0.19.1: `experimental_bevy_ui_widgets` is now **`bevy_ui_widgets`**, folded into the `ui` feature collection, with `UiWidgetsPlugins` + `InputDispatchPlugin` shipping in `DefaultPlugins` — no manual add. The recommendation itself is unchanged and the `Interaction` ban still stands (verified against `bevy_ui/src/focus.rs` @ v0.19.1).

Two amendments: `bevy_text` swapped cosmic-text → Parley, so `TextFont.font` is `FontSource` and `font_size` is `FontSize`. And `bevy_image_font` — the pixel-font fallback — has no 0.19 release, weakening plan B; 0.19's new `FontSize::Rem` + `RemSize` strengthen plan A in partial compensation.
