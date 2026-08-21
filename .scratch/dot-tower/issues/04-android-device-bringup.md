# Bevy hello-world on a physical Android device

Type: task
Status: claimed
Blocked by: 01

## Question

Nothing to decide. Get a minimal Bevy 2D app (0.19.1 — ticket 13 superseded the 0.18 assumption this was written under) — a sprite and a touch handler that moves it — building, installing, and running on a real Android device, using whatever toolchain ticket 01 identified.

This unblocks decisions rather than delivering the destination: until a real device is in the loop, every downstream decision about touch input, portrait layout, performance budget, and suspend/resume is guesswork.

The agent drives what it can (project scaffolding, build config, scripts). The human handles what needs physical access and their own credentials: device developer mode, USB debugging authorisation, and any Android SDK licence acceptance.

Record on resolution: the exact toolchain and commands that worked, the device and Android version tested, build times, and any workaround needed. Later tickets depend on these facts.

## Added by ticket 02

Two questions that could not be settled from source and need real hardware:

- Does `AndroidApp::content_rect()` account for display cutouts, or only system bars? (The reporter on #23003 was unsure too.) We need ~40 lines of safe-area handling built on it, so this shapes that code.
- What `UiScale` value actually yields crisp pixel text at roughly 2.625× phone density? The blur mechanism is verified as `font_size × scale_factor × UiScale`; whether the arithmetic *looks* right needs eyes on a screen.

## Added by ticket 01

Verify the save-durability chain empirically: log timestamps either side of the save write in the `AppLifecycle::Suspended` handler and compare against logcat lifecycle lines. Each link is verified in source; the composition is inference.

Also note Bevy's example Gradle targets API 33, while Google Play requires API 36 for updates from **2026-08-31**. Bringup should target API 36 rather than inheriting the example's config.

## Comments

### Agent-side work complete — awaiting device (session of 2026-08-18)

Everything not requiring physical hardware or a human licence acceptance is
done and verified. The ticket stays `claimed`, not `resolved`: its deliverable
is "runs on a real Android device", and that has not happened yet.

**Toolchain — one hard blocker found and cleared.** Bevy 0.19.1 declares
`rust-version = "1.95.0"`; this machine was on **1.89.0** and could not have
compiled it at all. Now:

| | |
|---|---|
| rustc | 1.97.1 (was 1.89.0) |
| targets | `aarch64-linux-android`, `x86_64-linux-android` |
| cargo-ndk | 4.1.2 |
| JDK / SDK / NDK / adb | **not installed** — human steps, see below |

**Scaffolded.**

- `Cargo.toml` — bevy 0.19.1, `default-features = false`, features
  `["2d", "ui", "ui_picking", "android-game-activity"]`. `[lib] crate-type =
  ["lib", "cdylib"]`. Dev profile builds deps at `opt-level = 3`.
- `rust-toolchain.toml` — pins channel + Android targets.
- `src/lib.rs` — the bringup harness, six labelled probes (A–F), documented at
  the top of the file. `#[bevy_main]` supplies `android_main`.
- `android/` — Gradle + GameActivity project. AGP **8.13.2** (newest 8.x;
  compileSdk 36 needs 8.9+), Gradle wrapper 8.14.3, `games-activity` **4.4.0**
  (matches Bevy's tested pin against `android-activity` 0.6.1 — a mismatch here
  fails at runtime, not build time), compileSdk/targetSdk **36**, minSdk 31.
- `scripts/android-setup.sh` — 8-stage wizard for the human half.
- `scripts/android-build.sh`, `scripts/android-run.sh` — unattended build,
  install, launch, and probe-log tail.

**Verified without a device.**

- `cargo check --all-targets` — clean.
- `cargo build` — clean, **4m39s** cold (Apple M3, 8 cores), 145 MB debug binary.
- Ran on desktop for 15s: window created, Metal adapter, GPU preprocessing
  supported, `probe E: AppLifecycle::Running` logged, no panics. This matters
  beyond the desktop: it proves the trimmed feature set produces a *coherent
  plugin set at runtime*, which a type-check alone does not.
- Native library name (`dot_tower`), Java package, `namespace`, `applicationId`
  and the `adb` app id were cross-checked to agree across all five files.

**Finding: ticket 13's build config was incomplete.** `ui_picking` is not part
of the `ui` collection, and `UiPickingPlugin` is gated behind it. Because
ticket 02 banned `Interaction`, picking observers are this project's only UI
input path — the previously-stated config would have produced an app whose
ability buttons silently never fire. Recorded in full on ticket 13.

**Finding: the Android target cannot even be `cargo check`ed without an NDK.**
`android-activity` builds its vendored C++ GameActivity glue through `cc-rs`,
which needs `aarch64-linux-android-clang++`. So there is no way to validate the
`cfg(target_os = "android")` code paths — including the `content_rect()` safe
area probe and `internal_data_path()` save location — until the NDK is
installed. Those paths are currently **compile-unverified**.

**Remaining, human only:** run `./scripts/android-setup.sh`. It installs JDK 17,
the Android command-line tools, API 36 + build-tools + NDK, and Gradle; walks
licence acceptance (deliberately not automated); generates the Gradle wrapper;
then covers developer mode, USB debugging and device authorisation. It ends by
offering to run `./scripts/android-run.sh`.

**Open knob:** `minSdk 31` follows Bevy's own tested example, which excludes
pre-Android-12 handsets. If ticket 14's low-end performance budget wants older
hardware, this is the line to change.

**Still to record here when it runs on hardware**, per the original question and
the addenda from tickets 01 and 02:

1. Device model, Android version, API level; cold and incremental build times.
2. Probe A — does touch drag the sprite; do coordinates stay sane at the screen
   edges (#7528); max simultaneous fingers observed.
3. Probe B — do two ability buttons pressed at once report two distinct
   `PointerId::Touch(..)` values. If not, the `Interaction` ban is insufficient.
4. Probe C — does `content_rect()` account for the display cutout, or only the
   system bars? Read the HUD's inset numbers against where the red frame sits.
5. Probe D — which `UiScale` value gives crisp pixel text at ~2.625× density.
6. Probe E — does `probe E: saved in Nms` appear in logcat after backgrounding,
   and does `WillSuspend` ever appear (it should not; if it does, ticket 01 is
   wrong and ticket 12's save design changes).
7. Probe F — any `bevy_ui` flicker against the sprite layer (#14710 on wgpu 29).
8. Any workaround needed to get there.

### Claim transferred; first run on hardware (session of 2026-08-21)

The session that claimed this on 2026-08-18 is gone; the dev confirmed it no
longer exists and reassigned the ticket here. Still `claimed`, **not
`resolved`** — probe F is failing and it has taken probes B, C and D down with
it.

**It runs.** APK built, installed and launched on a physical device. The build
chain works end to end.

#### 1. Device and build

| | |
|---|---|
| Device | Pixel 10 Pro |
| OS | Android 17, API **37** (ahead of the compileSdk/targetSdk 36 we build against) |
| SoC / GPU | Tensor G5 — **PowerVR D-Series DXT-48-1536 MC1**, Vulkan backend |
| CPU / RAM | 8 cores / 15.2 GiB |
| `.so` (debug, unstripped) | 1.4 GB |
| `.so` in APK (AGP-stripped) | 75 MB |
| APK | 79 MB |
| Gradle | 16s incremental (36 tasks) |
| Rust cdylib, cold for `aarch64-linux-android` | **not precisely timed** — record on a clean rebuild |

**The Android code paths compile.** Ticket 04 previously recorded every
`cfg(target_os = "android")` path as compile-unverified for want of an NDK.
They now build, including `content_rect()` and `internal_data_path()`.

#### 2. Probe A — touch transport: **PASS (partial)**

Press and drag moves the sprite; confirmed by hand on the device.
`max_fingers = 1` is the only reading captured so far. **Edge-coordinate
behaviour (#7528) and the multi-finger high-water mark are not yet assessed** —
both are read off the HUD, which probe F has destroyed.

#### 3. Probe B — UI picking per finger: **BLOCKED by probe F**

The four ability buttons do not render at all, so two-finger `PointerId::Touch`
distinctness is untested. This is the reading ticket 02's `Interaction` ban
depends on, and it remains an inference.

#### 4. Probe C — safe area: **BLOCKED by probe F**, plus a finding that stands regardless

The red frame does not render, so `content_rect()` versus the cutout is
unanswered. Independently, winit logs on this device:

```
WARN winit::platform_impl::android: TODO: handle Android InsetsChanged notification
WARN winit::platform_impl::android: TODO: find a way to notify application of content rect change
```

So insets and content-rect changes are **never delivered to the application**.
Whatever safe-area handling ticket 02 budgeted (~40 lines) has to **poll**
`AndroidApp::content_rect()`; it cannot be event-driven. That holds no matter
how probe F resolves.

#### 5. Probe D — text crispness: **BLOCKED by probe F**

No legible text to judge.

#### 6. Probe E — save durability: **PASS, and it closes ticket 01's inference**

```
probe E: AppLifecycle::Suspended
probe E: saved in 2ms -> /data/user/0/dev.dottower/files/bringup-probe.txt
```

File verified present on device (57 bytes, correct contents) via `run-as`.

- `AppLifecycle::Suspended` **is** a genuine synchronous save hook. Ticket 01
  verified each link in source and called the composition inference; it is now
  measured on hardware.
- **`WillSuspend` was never delivered**, as ticket 01 predicted. Had it appeared,
  ticket 12's design would have needed reworking. It stands.
- **2 ms**, comfortably inside a 16.7 ms frame — the budget ticket 01 flagged as
  unquantified by any source.
- `internal_data_path()` resolves to `/data/user/0/dev.dottower/files/`,
  confirming on hardware the writable location ticket 12 asserted from source.

#### 7. Probe F — bevy_ui flicker: **FAIL**

**The sprite layer renders correctly; the `bevy_ui` layer does not.** HUD text
appears as scattered glyph fragments that differ on every frame; the four
ability buttons and the red safe-area frame do not render at all. Three
screenshots three seconds apart hash differently, so this is live corruption
rather than a static glyph bug. **No wgpu error, no Vulkan validation failure,
no naga warning** — logcat filtered to the app's pid is silent. The harness is
not the cause: the HUD is a single `Text` whose string is rebuilt each frame,
not a respawn loop.

This is bigger than a bringup detail. **Ticket 13 chose 0.19.1 specifically
because #14710 (bevy_ui Android flicker) was fixed transitively via wgpu 29** —
that was the argument that rejected 0.18. Tickets 02 and 10 then built the whole
UI approach on `bevy_ui` + `bevy_picking`.

Not yet generalisable: **one device, one GPU, one build.** Candidate causes are
#14710 unfixed, a PowerVR-specific bug, or an interaction with the harness's
`UiAntiAlias::Off` + `Msaa::Off` + `FontSmoothing::None`. Cheapest isolating
tests, in order: run the same build on desktop and check HUD legibility; then
flip those three settings to defaults on device; then a second handset on a
different GPU vendor.

#### The Adreno constraint cannot be tested on this device

The map's standing preference — *no custom `Material2d`, it crashes on Adreno
GPUs* (#22925) — is unverifiable here. Tensor G5 is **PowerVR**, not Adreno. The
bringup handset cannot validate the hardest GPU constraint in the Notes, and
buying that assurance needs a second device.

#### 8. Workarounds required to get this far

Four defects blocked the first run. Three were scripting faults that only appear
on a real machine; none were Bevy, the NDK, or the device.

1. **Gradle wrapper was never generated.** Homebrew installs Gradle **9.7.1**;
   AGP 8.13.2 uses `org.gradle.api.problems.internal.InternalProblems`, removed
   in Gradle **9.6.0**. `gradle wrapper` inside `android/` died applying the
   Android plugin — before it could create the 8.14.3 wrapper that would itself
   have been compatible. Fixed by bootstrapping the wrapper in a scratch
   directory with no build script to configure, then copying it in.
   `scripts/android-setup.sh` now does this.
2. **`.env` values were written unquoted.** `write_env` produced
   `BRINGUP_DEVICE=Pixel 10 Pro (Android 17, API 37)`; an unquoted `(` is a hard
   bash syntax error, and both build scripts `source .env` under `set -euo
   pipefail`. The device-detection stage meant to help later steps was breaking
   them. `write_env` now quotes.
3. **macOS bash 3.2 and empty arrays.** `/usr/bin/env bash` resolves to
   `/bin/bash` 3.2.57, where expanding an empty array under `set -u` is an
   unbound-variable error (fixed in bash 4.4). `"${RELEASE_FLAG[@]}"` aborted
   every debug build before `cargo ndk` ran. Both sites in
   `scripts/android-build.sh` now use the `${arr[@]+"${arr[@]}"}` guard.
4. **Kotlin stdlib duplicate classes.** `appcompat 1.7.0` →
   `lifecycle-common 2.6.2` → `kotlinx-coroutines-android 1.6.4` requests
   `kotlin-stdlib-jdk7/jdk8:1.6.21`, whose classes were merged into
   `kotlin-stdlib` in Kotlin 1.8.0; alongside the 1.8.22 stdlib this fails
   `:app:checkDebugDuplicateClasses`. Constrained both to 1.8.22 (empty shims
   delegating to `kotlin-stdlib`) in the version catalogue. This project uses no
   Kotlin — the pin exists only to settle a transitive graph.

Also of note: Gradle warns the build uses features *"incompatible with Gradle
9.0"*. That is the same AGP-8.13-vs-Gradle-9 fault line seen from the other
side, so the 8.14.3 wrapper pin is load-bearing — a future upgrade must move AGP
and Gradle together.

**Files changed:** `scripts/android-setup.sh`, `scripts/android-build.sh`,
`android/app/build.gradle`, `android/gradle/libs.versions.toml`, plus new
`android/gradlew`, `android/gradlew.bat`, `android/gradle/wrapper/`. The wrapper
jar is currently untracked and conventionally should be committed, or a fresh
clone repeats the bootstrap. `android/gradle.properties` picked up
`ndkVersion=30.0.15729638` from the setup script.

#### What this ticket still owes

Probes B, C and D, all gated behind probe F; probe A's edge coordinates and
multi-finger maximum, same gate; and a properly timed cold Rust build. Probe F
is the critical path — nothing else can be read until `bevy_ui` renders.

### Probe F narrowed: not the harness, not the render settings (session of 2026-08-21)

Two controlled runs, and between them they eliminate both benign explanations.

**Control 1 — same build on desktop: renders correctly.** macOS 15.1, Apple M3,
Metal backend, "GPU preprocessing is fully supported on this device" (against
"Some GPU preprocessing are limited" on the Pixel). HUD text legible, all four
ability buttons present, sprite drags on click. Identical Bevy version,
identical UI code, identical settings. **The harness is exonerated** — this is
not a mistake in how the UI is built.

**Control 2 — Bevy default render settings on device: still corrupt.**
`Msaa::Off` + `UiAntiAlias::Off` + all three `FontSmoothing::None` reverted to
defaults, rebuilt, reinstalled. Glyph edges came back anti-aliased, confirming
the change took effect — and the corruption is unchanged in character: scattered
fragments differing frame to frame, no buttons, no safe-area frame. **The
pixel-art triple is exonerated.** Ticket 02's day-one rendering settings are not
the cause, so they need no revisiting on this account.

Experiment reverted; `src/lib.rs` is back to its committed state.

#### Where that leaves probe F

`bevy_ui` is broken on this device **independent of render configuration and
independent of the harness**. The sprite layer renders correctly throughout, so
this is specific to the UI pass. Still silent — no wgpu error, no Vulkan
validation failure, no naga warning.

What is *not* yet distinguished: whether this is **#14710 unfixed**, or a
**distinct PowerVR-specific bug**. Both are consistent with everything observed.
Separating them needs a second handset on a different GPU vendor — and that same
device would also settle the Adreno `Material2d` crash (#22925) that this
PowerVR device cannot test. **One borrowed Adreno phone answers both questions**,
which makes it the cheapest next move by some distance.

#### Consequence for ticket 13, which is closed

Ticket 13 chose 0.19.1 over 0.18 on a single deciding argument: that #14710
(bevy_ui Android flicker) was fixed transitively via wgpu 29, which 0.18 pins too
old to receive. On hardware, `bevy_ui` does not render correctly on Android.

This does **not** overturn the version choice — 0.19.1 remains at least as good
as 0.18, and nothing here argues for going back. What it overturns is the
*reason*: the fix that justified the upgrade is not observable on this device.
Whether ticket 13 needs amending waits on the second-device test, since a
PowerVR-only bug would leave its reasoning intact.

Tickets 02 and 10 both build on `bevy_ui` + `bevy_picking`. Neither is
invalidated yet — but if the second device also corrupts, the UI foundation is
in question with no obvious replacement, because ticket 02 banned the
`Interaction` path for multi-touch reasons (#11553) that still hold.
