# Bevy 0.18 Android: build toolchain and lifecycle

Type: research
Status: resolved

## Question

What is the current state of shipping a Bevy 0.18 2D game to Android, and what does it constrain?

Answer specifically:

1. **Toolchain.** What actually builds an APK/AAB today — `cargo-apk`, `xbuild`, `cargo-ndk` plus Gradle, or something newer? Which is maintained against 0.18? What does Bevy's own Android example use, and does it still work unmodified?
2. **Lifecycle.** How does Bevy handle Android suspend/resume? What happens to the render surface when the app backgrounds — is it destroyed and recreated, and does that require special handling? This is load-bearing: closed-form offline progress (ADR-0002) depends on knowing exactly when the app can write a timestamp and when it can be killed without warning.
3. **Save durability.** Where do files live on Android, and what guarantees exist that a write completes before the process is killed? The rotating-backup decision assumes writes can be interrupted.
4. **Input.** Does touch input arrive through the same events as mouse, or does it need separate handling? Any known issues with hold/long-press.
5. **Known sharp edges** for 2D specifically — audio, text rendering, asset loading from the APK.

Capture findings as a Markdown file in the repo and link it from this ticket.

## Answer

Findings: [research/01-bevy-android-support.md](../research/01-bevy-android-support.md) (587 lines, every claim tagged `[VERIFIED]` against pinned source or `[INFERENCE]`). Sources read at pinned versions: Bevy `v0.18.0`, winit `v0.30.12`, android-activity `v0.6.1` including its vendored AGDK glue. Nothing was run on a device.

1. **Toolchain**: `cargo-ndk` + Gradle with `GameActivity`. This is what Bevy's own example uses and what its `validation-jobs.yml` compiles on every merge-queue run. `cargo-apk` is deprecated in Bevy's README (no release since 2023-11-30, `NativeActivity` only); `xbuild` self-describes as unmaintained; `cargo-apk2` is alive but still `NativeActivity`/APK-only.
2. **Lifecycle**: the render surface *is* destroyed and recreated, and Bevy already handles it — `RawHandleWrapper` is removed on suspend and a fresh winit window is built for the same entity on resume. No game-code handling needed.
3. **Save durability**: `AppLifecycle::Suspended` is a genuine synchronous hook — it is delivered inside one forced `app.update()` while Android's Java main thread is blocked in `surfaceDestroyed`. A synchronous write plus `sync_all()` there will complete. Budget is one frame; no source quantifies it.
4. **Input**: touch does **not** arrive as mouse. winit's Android backend emits `WindowEvent::Touch` only, with a literal `// TODO mouse events`. No gesture recognition — long-press must be hand-rolled.
5. **Sharp edges**: audio does not auto-pause on background; no safe-area inset API (#23003); assets are read whole-file into memory from the APK with no writer and no hot reload.

**Consequences for other tickets:**

- `features = ["2d"]` is already Android-complete — it pulls `default_platform`, which includes `android-game-activity` and `android_shared_stdcxx`. No extra features required.
- **ADR-0002 is confirmed safe**, and `SystemTime` is the correct clock — `Instant` freezes while suspended and `CLOCK_MONOTONIC` stops in deep sleep.
- **`WillSuspend` / `WillResume` are never delivered** despite their doc comments. They exist in the enum and the winit runner collapses both to `Suspended`/`Running` before the send site. Do not design a two-phase save — this answers ticket 12 item 3.
- **Autosave needs two triggers.** `Suspended` fires on surface destruction, not `onPause`/`onStop` (winit drops all four Android lifecycle callbacks with `TODO` log lines; there is no `onDestroy` hook). Use `WindowFocused { focused: false }` as the primary trigger and `Suspended` as the backstop. Feeds ticket 12.
- **Avoid custom `Material2d`.** Issue #22925 is open and `P-Crash`, filed against Bevy 0.18.1 with `features = ["2d"]`: crashes on Adreno GPUs — the majority Android GPU — reproduced by two independent crates. Constrains ticket 03.
- **Play Store API level**: Bevy's example Gradle targets API 33; Google Play requires API 36 for updates from **2026-08-31**. It builds, but is not shippable as configured. Feeds ticket 04.
- Touch coordinates are misreported near screen edges (#7528, open since 2023) and there is no safe-area API (#23003). Both constrain tickets 02 and 10.

The one claim worth confirming empirically in ticket 04 is the save-durability chain: log timestamps either side of the write in the `Suspended` handler and compare against logcat lifecycle lines. Each link is verified; the composition is inference.

Status: resolved

## Superseded in part by ticket 13

The finding that `features = ["2d"]` is Android-complete was **true for 0.18 only**. The project now targets 0.19.1, where `2d` no longer implies `ui`, and `default_platform` no longer implies `android-game-activity`. Correct config is `features = ["2d", "ui", "android-game-activity"]`. Everything else in this ticket's answer was re-verified against v0.19.1 and stands unchanged — including the `AppLifecycle::Suspended` save hook and the undelivered `WillSuspend`/`WillResume`.
