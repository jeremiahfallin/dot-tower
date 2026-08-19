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
