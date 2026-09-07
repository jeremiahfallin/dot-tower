# Target Bevy 0.18 or 0.19?

Type: research
Status: resolved

## Question

This map was charted assuming Bevy 0.18, because that is what `guild-forge` pins. Ticket 02 established that **Bevy 0.19 shipped 2026-06-19, with 0.19.1 on 2026-08-13**. Starting a new project one minor version behind current needs to be a decision rather than an accident.

Establish:

1. **Does 0.19 fix issue #22925** — the `P-Crash` custom `Material2d` failure on Adreno GPUs, filed against 0.18.1? Ticket 01 flagged this as a constraint on the whole rendering approach, and Adreno is the majority Android GPU. If 0.19 fixes it, that alone likely decides this.
2. **Migration cost 0.18 → 0.19** for the surface this project actually touches: `bevy_ui`, `bevy_picking`, `experimental_bevy_ui_widgets`, 2D rendering, and the Android lifecycle/`AppLifecycle` API. Read the official migration guide.
3. **Android specifically.** Did anything change in the winit/android-activity stack? Are #7528 (edge touch coordinates) or #23003 (safe areas) resolved or newly unblocked?
4. **Ecosystem readiness** for anything still under consideration — and note that `bevy_lunex` has *not* followed 0.19, though ticket 02 already rules it out.
5. **Stability posture.** Is 0.19.x settled, or is there a pattern of point releases suggesting waiting? What is the release cadence, and when is 0.20 likely?

The decision follows fairly mechanically from the facts, so gather them and state the recommendation. Note that no code exists yet, which makes migration cost close to zero — the real question is whether 0.19 is *stabler*, not whether switching is cheap.

## Answer

Findings: [research/13-bevy-version-target.md](../research/13-bevy-version-target.md).

**Decision: target Bevy 0.19, pinned to `0.19.1`.**

1. **#22925 does not decide it.** Still `open`, `P-Crash`, `S-Needs-Investigation`, no milestone, zero PRs referencing it. Decisively: the reporter re-tested on `main` at `a5cbc9e` (2026-04-08), which already carried `wgpu 29.0.1` — the same wgpu major 0.19.1 ships. 0.19 was effectively tested and still crashes. The workaround is unchanged and applies to both versions: avoid custom `Material2d`, use a custom render pipeline.
2. **#14710 decides it** — `bevy_ui` flickering on Android, which 0.19 fixes transitively. Chain: root cause `gfx-rs/wgpu#8853` (Vulkan skips barriers between passes sharing a colour attachment — the main 2D pass and the UI pass); fixed by wgpu PR #8924, whose body reads "Resolves #8853 (and thus bevyengine/bevy#14710)"; merged 2026-03-15, shipped in wgpu 29.0.0. **Bevy 0.18 pins `wgpu = "27"`, Bevy 0.19 pins `wgpu = "29.0.3"`.** The reporter device-confirmed the patch on four Android devices. Unlike #22925, this needs only `bevy_ui` plus textured sprites — our day-one default, with nothing to design around.
3. **0.18 is a frozen branch.** `0.18.1` (2026-03-04) is its only patch; nothing in five and a half months, against 58 bugfix commits in 0.19.1. Bevy does not backport, so 0.18 can never receive the wgpu 29 fix.
4. **Cadence**: measured median gap 146 days; 0.20 expected ~Nov 2026. Starting on 0.18 means two migrations before year-end instead of one.

**This invalidates the build config, silently.** In 0.19, `2d` no longer implies `ui` (PR #23180) or `audio` (#23126), and `default_platform` no longer implies `android-game-activity` (#23708) or `android_shared_stdcxx` (deleted, #20323). `features = ["2d"]` compiles fine on 0.19 and yields **no `bevy_ui` and no Android activity backend**. The `ui_picking` weak-dep (`bevy_ui?/bevy_picking`) does not sneak it back in.

**Corrected config**: `bevy = { version = "0.19.1", default-features = false, features = ["2d", "ui", "android-game-activity"] }`

**Other changes:**
- `experimental_bevy_ui_widgets` → **`bevy_ui_widgets`**, folded into the `ui` collection. `UiWidgetsPlugins` + `InputDispatchPlugin` now ship in `DefaultPlugins` — drop the manual add. Gained `list`, `scrollarea`, `text_input`. Still documented as unstable.
- **`bevy_text` swapped cosmic-text → Parley.** Mostly invisible, but `TextFont.font` is now `FontSource` and `font_size` is now `FontSize` — every text call site changes shape.
- `bevy_image_font` has **no 0.19 release** (0.11.0 is 0.18-only; repo quiet since 2026-02-20). The pixel-font fallback weakens. Partly offset by 0.19's new `FontSize::Rem` + `RemSize`, which improve the scale-pinning approach ticket 02 already ranked first.

**Verified unchanged** — ADR-0002 and both prior tickets otherwise stand: the `AppLifecycle` enum and `bevy_winit` state machine are functionally identical, so `WillSuspend`/`WillResume` are still never delivered and `Suspended`-as-save-hook still works. The `Interaction` ban stands — `bevy_ui/src/focus.rs` @ v0.19.1 still calls the global-aggregate touch methods. `bevy_picking`'s observer API and `PointerId::Touch(u64)` are identical. `UiAntiAlias::Off` / `Msaa::Off` / `FontSmoothing::None` intact. winit stays 0.30, android-activity stays 0.6 — #7528 and #23003 unchanged, hand-rolled `content_rect()` safe areas still required.

**Two caveats, flagged not smoothed:** #14710 is still marked `open` and nobody re-tested *released* 0.19 on affected hardware — high-confidence inference, not certainty; verify in ticket 04. And 0.19.0 shipped with a `P-Regression` sprite flicker (#25163) in a config near-identical to ours, fixed in 0.19.1 — hence pinning `0.19.1`, not `0.19`.

Status: resolved

## Corrected by ticket 04

The stated build config is **incomplete**. Adding `ui_picking`:

```toml
bevy = { version = "0.19.1", default-features = false, features = [
    "2d", "ui", "ui_picking", "android-game-activity",
] }
```

This ticket read the `ui_picking` weak-dep correctly but drew the conclusion only
about `android-game-activity`. Verified against the vendored 0.19.1 source:

- `bevy-0.19.1/Cargo.toml` — the `ui` collection is `["default_app",
  "default_platform", "ui_api", "ui_bevy_render", "scene", "picking",
  "bevy_ui_widgets"]`. **`ui_picking` is not in it.**
- `bevy_ui-0.19.1/src/lib.rs:178` — `UiPickingPlugin` is added under
  `#[cfg(feature = "bevy_picking")]`, which only `bevy_internal/ui_picking`
  turns on.

The `picking` in the `ui` collection is `bevy_picking` itself — the pointer
abstraction and observer machinery. It is not the **UI backend** that decides
which UI node a pointer is over. Without `ui_picking` the app compiles, runs,
and `Pointer<..>` observers on UI nodes simply never fire.

This is load-bearing rather than cosmetic because ticket 02 banned `Interaction`
(#11553 aggregates touches globally, breaking the four-button multi-touch
surface). Picking observers are therefore the *only* UI input path in this
project — so `features = ["2d", "ui", "android-game-activity"]` would have
yielded an app with **no working UI input at all**, failing silently.

Also confirmed while building against it: **Bevy 0.19.1 declares
`rust-version = "1.95.0"`.** This machine was on 1.89.0 and could not have
compiled it. Now on 1.97.1; `rust-toolchain.toml` pins the channel.

## Amended by ticket 04: the deciding reason is unverified on hardware

This ticket chose 0.19.1 over 0.18 on a single argument — that #14710 (bevy_ui
Android flicker) was fixed transitively via wgpu 29, which 0.18 pins too old to
receive. On the bringup handset, **`bevy_ui` does not render correctly at all**.

The choice is not overturned: 0.19.1 remains at least as good as 0.18, and
nothing observed argues for going back. What is in doubt is the *reason*. The fix
that justified the upgrade is not observable on this device.

Whether this ticket needs a real amendment now waits on
[ticket 22](22-bevy-ui-android-rendering.md), not on ticket 04 — specifically on
its 0.18-versus-0.19 comparison and its second-GPU-vendor test. A PowerVR-only
bug would leave this ticket's reasoning intact; a 0.19 regression would make the
decision actively harmful.
