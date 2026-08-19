# Bevy 0.18 on Android: toolchain, lifecycle, save durability, input

Research for ticket `.scratch/dot-tower/issues/01-bevy-android-support.md`.

- **Date of investigation:** 2026-08-18
- **Bevy version read:** `v0.18.0` tag (released 2026-01-13). Latest patch referenced in issue reports is `0.18.1`.
- **Transitive versions read:** `winit 0.30` (`v0.30.12` source), `android-activity 0.6` (`v0.6.1` source), `rodio 0.20` + `cpal 0.15`, `cargo-ndk 4.1.2`.
- **Method:** reading primary source at pinned tags (Bevy repo, winit, android-activity, the vendored AGDK `android_native_app_glue.c`), Bevy's own CI workflow, Bevy's GitHub issue tracker, crates.io release metadata, and developer.android.com. **Nothing here was tested on a physical device.** Every claim is tagged `[VERIFIED]` (read directly from a primary source, with the file named) or `[INFERENCE]` (my reasoning across sources).
- **Not consulted:** Bevy's Discord (not reachable from this environment). Where Discord is the usual home of a known problem, I say so.

---

## Summary — what this means for dot-tower

1. **The toolchain question has a clear answer: `cargo-ndk` + Gradle with `GameActivity`.** That is what Bevy's own example uses and what Bevy's CI builds on every merge-queue run. `cargo-apk` is deprecated by Bevy's own README, has had no release since 2023-11-30, and only supports `NativeActivity`. `xbuild`'s own repository description reads "(unmaintained)". Use Gradle. It is also the only path to an AAB.
2. **`features = ["2d"]` is already Android-complete.** The `2d` collection pulls in `default_platform`, which pulls in `android-game-activity` and `android_shared_stdcxx`. No extra Cargo features are needed for Android. This is the useful side-finding the ticket asked for.
3. **There *is* a reliable "about to be suspended" hook, and it is stronger than expected.** `AppLifecycle::Suspended` is delivered inside exactly one forced `app.update()`, and that update runs while Android's Java main thread is *blocked* in `surfaceDestroyed` waiting for the native thread. A synchronous file write in that frame will complete before the process can be torn down. This is verified through four layers of source. **ADR-0002's closed-form offline progress is safe.**
4. **The budget for that hook is one frame, not one second.** Blocking the Java main thread risks an ANR (android-activity documents this explicitly). A RON save of the size ticket 12 describes is fine; anything expensive is not. Do not do a full re-serialise-and-fsync of a large blob there — but our save is small.
5. **The catch: `Suspended` fires on *surface destruction*, not on `onPause`/`onStop`.** Bevy has no `onPause`, `onStop`, `onSaveInstanceState` or `onDestroy` hook at all — winit 0.30 explicitly drops all four with a `TODO` log line. Recommendation: **also autosave on `WindowFocused { focused: false }`**, which fires earlier (on Android's `LostFocus`) and is cheap. Treat `Suspended` as the backstop, not the only trigger.
6. **The app is completely frozen while backgrounded.** No systems run, `Time` does not advance. This is exactly what a closed-form offline design wants — but it means you must compute elapsed time from wall clock (`SystemTime`), never from `Time`/`Instant`. ADR-0002 already does this.
7. **Touch is *not* mouse.** Android motion events become `TouchInput` only; Bevy never synthesises `MouseButtonInput` or `CursorMoved` from them. Anything reading `ButtonInput<MouseButton>` is dead code on Android. Use `bevy_picking` observers, which unify mouse and touch into `PointerId`.
8. **Two sharp edges that should change plans:** (a) custom `Material2d` crashes on some Adreno GPUs — reported on Bevy 0.18.1 with literally `features = ["2d"]` (issue #22925, open); avoid custom 2D materials until that resolves. (b) Bevy's example Gradle config targets API 33, which is **below Google Play's requirement of API 36 as of 2026-08-31** — the example builds but is not shippable as-is.
9. **Rotating backups remain justified**, but for a different reason than "suspend writes get interrupted" — the suspend write is actually safe. They protect against crash/kill during the *other* autosave triggers, and against corruption. Keep them.

---

## 1. Toolchain

### 1.1 What Bevy itself uses — verified

`[VERIFIED]` Bevy's `examples/README.md` at `v0.18.0` documents exactly one supported Android path and calls the alternative deprecated:

Setup:

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
```

with `ANDROID_SDK_ROOT` set to the SDK root, and `ANDROID_NDK_ROOT` set when using "NDK (Side by side)".

Build:

```sh
cargo ndk -t arm64-v8a -o android_example/app/src/main/jniLibs build
cd examples/mobile/android_example && ./gradlew build
```

Debug:

```sh
adb logcat | grep 'RustStdoutStderr\|bevy\|wgpu'
adb uninstall org.bevyengine.example   # fixes "unknown activity" errors
```

On `cargo-apk`, the README says verbatim that it is "a simpler and deprecated tool which doesn't support `GameActivity`".

`[VERIFIED]` `.github/workflows/validation-jobs.yml` at `v0.18.0` contains a `build-android` job that runs on every merge group:

```yaml
- name: Set up JDK 17            # actions/setup-java@v5, temurin
- run: rustup target add aarch64-linux-android
- run: cargo install --force cargo-ndk
- run: cargo ndk -t arm64-v8a -o android_example/app/src/main/jniLibs build --package bevy_mobile_example
- run: cd examples/mobile/android_example && chmod +x gradlew && ./gradlew build
```

So the `cargo-ndk` + Gradle + `GameActivity` path is **continuously verified to compile** against 0.18. It is not verified to *run* — CI never installs on a device or emulator.

### 1.2 Maintenance status of each candidate

| Tool | Latest crates.io release | Repo last push | Verdict |
|---|---|---|---|
| `cargo-ndk` | **4.1.2, 2025-08-09** | 2026-06-12 | `[VERIFIED]` Actively maintained. This is the one. |
| `cargo-apk` | 0.10.0, **2023-11-30** | 2026-04-08 | `[VERIFIED]` No release in 2.5 years. README: "Ideal for apps that provide a `NativeActivity`" — no `GameActivity`. Bevy calls it deprecated. |
| `xbuild` | 0.2.0, **2022-12-21** | 2026-07-21 | `[VERIFIED]` GitHub repo description literally reads `Cross compile rust to any platform (unmaintained)`. Do not use. |
| `cargo-apk2` | **1.4.0, 2026-08-13** | active | `[VERIFIED]` Actively maintained community fork of `cargo-apk`, explicitly created because "`cargo-apk` has stagnated". Still `NativeActivity`-only, still APK-only. `[INFERENCE]` Usable as a fast no-Gradle path for throwaway device testing; not viable for Play Store (no AAB, no `GameActivity`). |

`[INFERENCE]` For a Play Store release you need an **AAB**, which means Gradle (`./gradlew bundleRelease`). There is no Rust-native tool producing AABs. This settles it: Gradle is not optional for shipping, so adopt it from the start rather than migrating later.

### 1.3 Cargo.toml / project configuration required

`[VERIFIED]` `examples/mobile/Cargo.toml` at `v0.18.0` is remarkably minimal — the Gradle path needs **no `[package.metadata.android]` at all**:

```toml
[lib]
name = "bevy_mobile_example"
crate-type = ["lib", "cdylib"]

[dependencies]
bevy = { path = "../../" }
```

`[VERIFIED]` The entry point uses `#[bevy_main]` on `pub fn main()` in `src/lib.rs`, with a thin `src/main.rs` for desktop. Expanding `crates/bevy_derive/src/bevy_main.rs` shows what the macro generates:

```rust
#[unsafe(no_mangle)]
#[cfg(target_os = "android")]
fn android_main(android_app: bevy::android::android_activity::AndroidApp) {
    let _ = bevy::android::ANDROID_APP.set(android_app);
    main();
}
```

So `#[bevy_main]` is mandatory: without it `bevy::android::ANDROID_APP` is never populated, and `bevy_winit` and the Android asset reader both `.expect("Bevy must be setup with the #[bevy_main] macro on Android")`.

`[VERIFIED]` The Gradle side (`examples/mobile/android_example/`):

- `app/build.gradle`: `compileSdk 34`, `minSdk 31`, `targetSdk 33`, `abiFilters 'arm64-v8a'`, `buildFeatures { prefab true }`, `externalNativeBuild.cmake` with `-DANDROID_STL=c++_shared`, and
  ```groovy
  sourceSets.main {
      assets.srcDirs += files('../../../../assets')
      res.srcDirs    += files('../../../../assets/android-res')
  }
  ```
  — i.e. the Rust crate's `assets/` directory is wired straight into the APK's assets.
- `gradle/libs.versions.toml`: AGP `8.4.0`, `androidx.games:games-activity:2.0.2`.
- `gradle-wrapper.properties`: Gradle `8.6`.
- `MainActivity.java`: `extends com.google.androidgamesdk.GameActivity`, `System.loadLibrary("bevy_mobile_example")`, plus immersive-mode flags in `onWindowFocusChanged`.
- `AndroidManifest.xml`: `<meta-data android:name="android.app.lib_name" android:value="bevy_mobile_example" />` and a wide `android:configChanges` list (`orientation|screenSize|density|uiMode|...`) so the Activity is **not** recreated on rotation.

`[INFERENCE]` That `configChanges` list matters more than it looks: `ANDROID_APP` is a `OnceLock`, and `#[bevy_main]` sets it with `let _ = ...set(...)`, silently ignoring failure. If the Activity were ever destroyed and recreated in the same process, `android_main` would run again and the `OnceLock` would keep the **stale** `AndroidApp`. android-activity's own docs warn against exactly this: "It's not recommended to store an `AndroidApp` as global static state". Keep the `configChanges` list; do not let the Activity be recreated. For a portrait-locked game, also add `android:screenOrientation="portrait"`.

### 1.4 `libc++_shared.so`

`[VERIFIED]` Bevy's README: "Bevy may require `libc++_shared.so` to run on Android, as it is needed by the `oboe` crate, but typically `cargo-ndk` does not copy this file automatically." The example works around it with a CMake `dummy.cpp` + `packagingOptions { exclude 'lib/*/libdummy.so' }` hack.

`[VERIFIED]` `cargo-ndk`'s current README documents a much simpler fix: **`cargo ndk build --link-libcxx-shared`**. Prefer this over the CMake hack — it removes the need for `externalNativeBuild`, `CMakeLists.txt`, `dummy.cpp` and `prefab` entirely.

### 1.5 Which Cargo features Android needs — the side-finding

`[VERIFIED]` From `Cargo.toml` at `v0.18.0`:

```toml
2d = ["default_app", "default_platform", "2d_bevy_render", "ui", "scene", "audio", "picking"]

default_platform = [
  "std", "android-game-activity", "android_shared_stdcxx",
  "bevy_gilrs", "bevy_winit", "default_font", "multi_threaded",
  "webgl2", "x11", "wayland", "sysinfo_plugin",
]
```

**`bevy = { version = "0.18", default-features = false, features = ["2d"] }` already enables everything Android needs.** Specifically:

- `android-game-activity` → `winit/android-game-activity` (the GameActivity backend; matches Bevy's Gradle example).
- `android_shared_stdcxx` → `bevy_audio/android_shared_stdcxx` → `cpal/oboe-shared-stdcxx` (this is *why* `libc++_shared.so` is needed).
- `default_font` → the embedded fallback font, so text renders before you ship a font asset.

`[VERIFIED]` Input source features are also covered: `bevy_window`'s `Cargo.toml` unconditionally enables `bevy_input`'s `["gestures", "keyboard", "mouse", "touch"]`, so the 0.18 "input sources are now behind features" migration note does not bite us.

`[VERIFIED]` Choosing `NativeActivity` instead would require dropping `default_platform` and hand-listing features, since `android-native-activity` and `android-game-activity` are mutually exclusive alternatives. Don't.

### 1.6 Does Bevy's example still work unmodified?

`[VERIFIED]` It **compiles** unmodified — CI proves that.

Caveats, all verified:

- The example is a **3D** scene (cube, sphere, `PointLight`, `Camera3d`). It is a build-system reference, not a 2D reference.
- Its Gradle config is stale: `targetSdk 33`, `compileSdk 34`, AGP 8.4.0, Gradle 8.6, `games-activity:2.0.2`.
- Open Bevy issue **#23491** ("examples: Use single up-to-date android example", opened 2026-03-24, `S-Waiting-on-SME`) documents this from the inside: deprecated `setSystemUiVisibility`, "most Android dependencies are very out of date", and a proposal to move to `targetSdk = 37` / `compileSdk = 37`.
- Open issue **#19021** ("No instructions on building for Android or iOS under examples/mobile", still open 2026-02-04) — the instructions live in `examples/README.md`, not in `examples/mobile/`, which is why they are easy to miss.
- The `android_basic` (cargo-apk) readme still says `bevy = { version = "0.14" }` — clear evidence that path is unmaintained.
- **Sources disagree on the minimum API level for `GameActivity`.** Bevy's README says 31; issue #23491 asserts that is "incorrect. It should be 23", citing androidx versioning. Unresolved. Treat 31 as the safe assumption until measured on a device (ticket 04).

### 1.7 Play Store constraints (not Bevy's fault, but load-bearing)

`[VERIFIED, developer.android.com, fetched 2026-08-18]` As of **2026-08-31**, new apps and app updates on Google Play must target **Android 16 (API 36)** or higher. Existing apps must target API 35+ to stay available to new users. An extension to 2026-11-01 can be requested. Bevy's example `targetSdk 33` is well below this.

`[VERIFIED, developer.android.com]` Since **2025-11-01**, apps targeting API 35+ must support **16 KB memory pages** on 64-bit devices. For native `.so` files this means all `LOAD` segments aligned to `2**14`, `GNU_RELRO` present, and 16 KB zip alignment. **NDK r28+ does this by default.** For NDK r27 and lower you must pass `-Wl,-z,max-page-size=16384 -Wl,-z,common-page-size=16384`.

`[INFERENCE]` For a `cargo-ndk` build that means either using NDK r28+ (simplest — `cargo-ndk` auto-detects the newest installed NDK), or exporting `RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-z,common-page-size=16384"`. I found **no Bevy issue** about 16 KB pages, which is consistent with "NDK r28+ makes it a non-issue" rather than with "Bevy is broken here". Verify with:
```sh
llvm-objdump -p libdot_tower.so | grep LOAD     # expect align 2**14
zipalign -v -c -P 16 4 app-release.apk
```

---

## 2. Lifecycle

### 2.1 The type

`[VERIFIED]` `crates/bevy_window/src/event.rs`:

```rust
pub enum AppLifecycle {
    Idle,          // The application is not started yet.
    Running,       // The application is running.
    WillSuspend,   // "Applications have one frame to react to this event before being paused"
    Suspended,     // The application was suspended.
    WillResume,    // "one extra frame to react to this event before being fully resumed"
}
impl AppLifecycle { pub fn is_active(&self) -> bool { /* false for Idle | Suspended */ } }
```

In 0.18 this is a **`Message`**, not an `Event` — read it with `MessageReader<AppLifecycle>`. It is also wrapped in `WindowEvent::AppLifecycle(_)` if you need ordering against other window events.

### 2.2 **`WillSuspend` and `WillResume` are never actually delivered**

`[VERIFIED]` This contradicts the doc comments above, so it is worth stating plainly. A full grep of `crates/` at `v0.18.0` finds `WillSuspend`/`WillResume` in only two files: the enum definition and `bevy_winit/src/state.rs`. In `state.rs`:

- `fn suspended()` sets `self.lifecycle = AppLifecycle::WillSuspend` (line 498) — it does **not** send anything.
- At the top of `redraw_requested()` (line 531), `if self.lifecycle == WillSuspend { self.lifecycle = Suspended; ... }`.
- The single send site (line 598-600) is *after* that transition:
  ```rust
  if self.lifecycle != self.previous_lifecycle {
      self.previous_lifecycle = self.lifecycle;
      self.bevy_window_events.send(self.lifecycle);
  }
  ```

The same collapse happens on the resume side (`WillResume` → `Running` at line 551). **Game code therefore only ever observes `AppLifecycle::Running` and `AppLifecycle::Suspended`.** Bevy's own mobile example is consistent with this: its `handle_lifetime` treats `WillSuspend`/`WillResume` as no-ops and does the real work in the `Suspended` and `Running` arms.

Practical consequence: do not write a two-phase "prepare on WillSuspend, commit on Suspended" save. There is one phase.

### 2.3 What "suspend" actually means on Android

`[VERIFIED]` `winit 0.30` `src/platform_impl/android/mod.rs`:

```rust
MainEvent::InitWindow      { .. } => callback(Event::Resumed,   ...),
MainEvent::TerminateWindow { .. } => callback(Event::Suspended, ...),
MainEvent::Start   => warn!("TODO: forward onStart notification to application"),
MainEvent::Pause   => { self.running = false; }      // not forwarded
MainEvent::Stop    => warn!("TODO: forward onStop notification to application"),
MainEvent::SaveState { .. } => warn!("TODO: forward saveState notification ..."),
MainEvent::Destroy => warn!("TODO: forward onDestroy notification to application"),
```

So:

- **`Suspended` == the native render surface was destroyed.** It is not `onPause` and not `onStop`.
- **There is no `onPause`, `onStop`, `onSaveInstanceState` or `onDestroy` hook in Bevy at all.** This was noted back in issue #9057 (2023) and is still true in winit 0.30 as shipped with Bevy 0.18.
- `MainEvent::LostFocus` **is** forwarded, as `WindowEvent::Focused(false)`, which `bevy_winit/src/state.rs:362` turns into a `WindowFocused { window, focused: false }` message. This is the earliest signal Bevy gives you that the app is going away.

### 2.4 The render surface *is* destroyed and recreated — Bevy handles it

`[VERIFIED]` `bevy_winit/src/state.rs`, inside `redraw_requested()`:

On suspend (line 531 onwards):
```rust
self.lifecycle = AppLifecycle::Suspended;
should_update = true;                      // force one last update
self.ran_update_since_last_redraw = false;
#[cfg(target_os = "android")]
{
    // Remove the `RawHandleWrapper` from the primary window.
    // This will trigger the surface destruction.
    ...remove::<RawHandleWrapper>();
}
```

On resume (line 551 onwards): it queries for `(With<CachedWindow>, Without<RawHandleWrapper>)`, calls `winit_windows.create_window(...)` to make a **new** winit window for the **same** `Entity`, and re-inserts a fresh `RawHandleWrapper`.

`[VERIFIED]` `bevy_render/src/view/window/mod.rs` closes the loop — `extract_windows` reads `RemovedComponents<RawHandleWrapper>` and drops both the `ExtractedWindow` and the `wgpu` `Surface` for that entity.

`[INFERENCE]` What survives across suspend/resume: the `Window` entity id, all components on it except `RawHandleWrapper`, the whole main-world ECS, the `wgpu` `Device`/`Queue` and everything uploaded to the GPU (textures, buffers, pipelines). Only the `Surface` is torn down and rebuilt. Cameras targeting `RenderTarget::Window` are keyed by entity, so they reattach automatically. **No special handling is required from game code for the surface.** This matches the design intent of #9057's resolution.

`[INFERENCE]` One thing to be aware of: the surface is destroyed *before* the final `Suspended` update runs. Anything in your `Suspended` handler that assumes a live render target (e.g. reading `Camera.computed.target_info`, doing a screen-to-world conversion) may see `None`. Keep the handler to pure CPU work.

### 2.5 While suspended, nothing runs

`[VERIFIED]` `should_update()` ends with `handle_event && self.lifecycle.is_active()`, and `is_active()` is `false` for `Suspended`. Redraws are also gated: `if self.redraw_requested && self.lifecycle != AppLifecycle::Suspended`.

`[VERIFIED]` Bevy issue **#9057, "Bevy cannot run in the background on Android"** (`P-High`, still open, last substantive comment 2024-03-19) confirms this is a known, deliberate, unfinished state. mockersf: "To run in background, we need to be able to run the main app while skipping part of the rendering app."

`[INFERENCE]` For dot-tower this is a feature, not a bug: the closed-form design never wanted background ticks. But it has two hard consequences:

- **`Time` does not advance while suspended.** Elapsed offline time must come from wall clock.
- `[INFERENCE, high confidence, not verified from a doc in this session]` `std::time::Instant` on Android is `CLOCK_MONOTONIC`, which **stops during device deep sleep**. So `Instant` is doubly wrong for offline time — it misses both the frozen-app period and deep sleep. `std::time::SystemTime` / `UNIX_EPOCH` is the correct source, which is what ADR-0002 already specifies. Worth an explicit note in the save code so nobody "optimises" it to `Instant` later.

### 2.6 Resume sequence

`[VERIFIED]` `resumed()` sets `WillResume` and calls `create_windows`. The next `redraw_requested()` flips to `Running`, recreates the winit window + `RawHandleWrapper`, sets `redraw_requested = true`, sends `AppLifecycle::Running`, and runs an update. So game code gets exactly one update where `Running` is readable — that's where you re-read the wall clock, compute offline progress, and resume audio.

---

## 3. Save durability

### 3.1 Where files live

`[VERIFIED]` `bevy_asset/src/io/source.rs`, `get_default_writer()`:

```rust
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
return None;
```

**Bevy's `AssetWriter` does not exist on Android.** You cannot use Bevy's asset IO to write saves. Use `std::fs` directly. (Reading is fine — see §5.3.)

`[VERIFIED]` The save directory comes from `android-activity`, reachable from anywhere via the global Bevy sets up:

```rust
#[cfg(target_os = "android")]
let dir: Option<std::path::PathBuf> = bevy::android::ANDROID_APP
    .get()
    .and_then(|app| app.internal_data_path());
```

`[VERIFIED]` `android-activity` v0.6.1 `game_activity/mod.rs`:
```rust
pub fn internal_data_path(&self) -> Option<std::path::PathBuf> {
    self.game_activity.with_locked_app(|app_ptr| {
        if app_ptr.is_null() { log::error!("... after GameActivity was destroyed"); return None; }
        unsafe { try_get_path_from_ptr((*(*app_ptr).activity).internalDataPath) }
    })
}
```
and `GameActivity.cpp` sets `internalDataPath` from the `internalDataDir` string handed in from Java. `[INFERENCE]` That is the Java `Context.getFilesDir()`, i.e. `/data/data/<package>/files` — app-private, backed up by default, wiped on uninstall. Correct place for a save.

`[VERIFIED]` It returns `Option`, and it returns `None` once the Activity is destroyed. Handle that; don't `unwrap()`.

`[VERIFIED]` `external_data_path()` and `obb_path()` also exist if ever needed.

### 3.2 The guarantee — this is the important part

The chain, verified layer by layer:

1. `[VERIFIED]` **`android_native_app_glue.c`** (vendored in android-activity v0.6.1, from the AGDK GameActivity sources). `android_app_set_window()` runs on the **Java main thread** when the surface is destroyed:
   ```c
   if (android_app->pendingWindow != NULL) { android_app_write_cmd(android_app, APP_CMD_TERM_WINDOW); }
   android_app->pendingWindow = window;                      // NULL
   ...
   while (android_app->window != android_app->pendingWindow) {
       pthread_cond_wait(&android_app->cond, &android_app->mutex);   // <-- BLOCKS
   }
   ```
   and it is only released by `android_app_post_exec_cmd(APP_CMD_TERM_WINDOW)` on the native thread, which sets `android_app->window = NULL` and broadcasts.

2. `[VERIFIED]` **android-activity** `game_activity/mod.rs` calls, in order: `android_app_pre_exec_cmd(cmd)` → **`callback(PollEvent::Main(MainEvent::TerminateWindow{}))`** → `android_app_post_exec_cmd(cmd)`. The Java thread stays blocked for the whole duration of that callback.

3. `[VERIFIED]` **winit** `platform_impl/android/mod.rs`: that callback runs `single_iteration(Some(TerminateWindow))`, which dispatches `Event::Suspended` and then, at the end of the *same* function, `callback(Event::AboutToWait)` — the comment there reads "This is always the last event we dispatch before poll again".

4. `[VERIFIED]` **Bevy** `bevy_winit/src/state.rs`: `about_to_wait()` calls `redraw_requested()`, which flips `WillSuspend → Suspended`, forces `should_update = true` and `ran_update_since_last_redraw = false`, sends the `AppLifecycle::Suspended` message, and calls `run_app_update()` → `app.update()`.

**Conclusion `[INFERENCE, from four verified layers]`: a system that reads `MessageReader<AppLifecycle>`, sees `Suspended`, and performs a blocking `std::fs` write + `sync_all()` will complete that write while Android's UI thread is blocked in `surfaceDestroyed`. The write is not racing the process teardown.** This is a genuine synchronous hook, stronger than the "best-effort onPause" you get in most engines.

Corroboration `[VERIFIED]`: Bevy's own `examples/mobile/src/lib.rs` relies on exactly this, pausing audio in the `AppLifecycle::Suspended` arm — which only works if user systems really do run in that frame.

### 3.3 How much time does it give?

`[VERIFIED]` android-activity's crate docs: "Some `Activity` lifecycle callbacks on the Java main thread will block until the next time `poll_events()` is called, so if you don't call `poll_events()` regularly you may trigger an ANR dialog and cause users to force close your application." And, on the `on_create` entrypoint: "Blocking the Java main thread for too long may cause an 'Application Not Responding' (ANR) dialog".

`[INFERENCE]` No source I found states a numeric budget for the surface-destroy path specifically. **Treat the budget as "one frame" — target well under 100 ms, and never exceed a couple hundred.** A small RON blob plus `sync_all()` is comfortably inside that on flash storage. What would *not* be safe: serialising a large world, compressing, doing a directory scan, or awaiting an async task (there is no later frame to await into).

I deliberately did **not** invent AOSP timeout constants here. If an exact number matters, measure it on device in ticket 04 by logging timestamps either side of the write.

### 3.4 What has *no* warning at all

`[VERIFIED]` winit drops `MainEvent::Destroy` with `warn!("TODO: forward onDestroy notification to application")` and never exits the loop. So:

- **Process killed while already suspended** (memory pressure, user swipes from Recents after backgrounding): no notification. Harmless — the save was written at suspend.
- **`onDestroy` / Back-button finish**: no hook. `[INFERENCE]` In the normal Activity teardown order the surface is destroyed before `onDestroy`, so `Suspended` should still have fired first — but this is exactly the kind of thing to confirm on a device in ticket 04.
- **Crash / panic / ANR kill**: no hook. This is the real justification for the rotating backup.
- **Low memory**: winit *does* forward `MainEvent::LowMemory` as `Event::MemoryWarning`, but `[VERIFIED]` Bevy's `WinitAppRunnerState` has no handler for it — it is silently dropped. No hook.

### 3.5 Concrete recommendation for ticket 12

`[INFERENCE]` Autosave triggers, in order of reliability:

1. **`AppLifecycle::Suspended`** — the backstop. Synchronous, guaranteed to complete. Always write the timestamp here.
2. **`WindowFocused { focused: false }`** — fires earlier (Android `LostFocus`), cheap, and covers cases where the surface is not destroyed (notification shade, split-screen, a dialog over the app). Add this; it costs nothing.
3. Domain triggers already planned (on lock, on prestige, on a timer).

Do **not** rely on: `WillSuspend` (never sent), any `onPause`/`onStop`/`onDestroy` hook (doesn't exist), or `AppExit` (Android never sends it).

`[INFERENCE]` Durable-write recipe, unchanged from desktop practice but worth writing down because Bevy gives you nothing here:

```
write  save.ron.tmp
file.sync_all()               // <- required; a rename alone does not flush data
rename save.ron.tmp -> save.ron   // atomic within the same directory
(optionally) open the directory and sync_all() it
```
Rotate the previous `save.ron` to `save.ron.bak` before the rename. `File::sync_all()` is the Rust API; skipping it is the classic way to end up with a zero-length RON file after a power loss.

`[INFERENCE]` **Verdict on the rotating backup decision:** keep it, but for the right reason. The suspend write is safe. The backup protects against (a) a kill or crash during a *timer* autosave, (b) a partially-flushed write on a device that lies about `fsync`, and (c) a schema bug producing unparseable RON. That third one is arguably the most likely in practice.

---

## 4. Input

### 4.1 Touch does not arrive as mouse

`[VERIFIED]` `winit 0.30` `platform_impl/android/mod.rs`, `handle_input_event`:

```rust
InputEvent::MotionEvent(motion_event) => {
    let phase = match motion_event.action() {
        MotionAction::Down | MotionAction::PointerDown => Some(TouchPhase::Started),
        MotionAction::Up   | MotionAction::PointerUp   => Some(TouchPhase::Ended),
        MotionAction::Move                             => Some(TouchPhase::Moved),
        MotionAction::Cancel                           => Some(TouchPhase::Cancelled),
        _ => { None }   // TODO mouse events
    };
    ... WindowEvent::Touch(Touch { device_id, phase, location, id, force: Some(Force::Normalized(pressure)) })
}
```

**Android motion events become `WindowEvent::Touch` and nothing else.** No `CursorMoved`, no `MouseInput`. Note the literal `// TODO mouse events` — even a connected USB mouse / stylus hover produces nothing on Android through this backend.

`[VERIFIED]` Bevy turns that into `TouchInput { phase, position, window, force, id }` messages plus the `Touches` resource.

`[INFERENCE]` So: **anything in dot-tower that reads `ButtonInput<MouseButton>`, `CursorMoved`, or `window.cursor_position()` will be silently dead on Android.** This needs to be a codebase rule from day one, not a port-time discovery.

### 4.2 The portable path is `bevy_picking`

`[VERIFIED]` `bevy_picking/src/input.rs` runs `mouse_pick_events` and `touch_pick_events` side by side. Touches become `PointerId::Touch(id)` pointers with their own press/move/release/drag actions; mouse becomes the single `PointerId::Mouse`. `PointerInputSettings { is_touch_enabled, is_mouse_enabled }` both default to `true`. Touch pointers are deactivated on lift (`deactivate_touch_pointers`), so no phantom hover state lingers.

`[VERIFIED]` `picking` (including `ui_picking` and `sprite_picking`) is part of the `2d` feature collection, so `bevy_ui`'s `Interaction` component works from touch out of the box — Bevy's mobile example's `button_handler` is driven purely by touch.

`[INFERENCE]` **Recommendation: build all of dot-tower's input on picking observers (`Pointer<Press>`, `Pointer<Release>`, `Pointer<Drag>`) plus `Interaction` for UI.** That is the only layer that behaves identically on desktop and Android. Reach for raw `TouchInput` only for gestures picking doesn't model.

`[VERIFIED]` Minor doc discrepancy worth knowing: `TouchInput::force`'s doc comment says pressure "is only available on **iOS** 9.0+ and **Windows** 8+", but winit's Android backend *does* populate it with `Force::Normalized(pointer.pressure())`. Don't design around it either way.

### 4.3 Hold / long-press

`[VERIFIED]` `bevy_input/src/gestures.rs` — `PinchGesture`, `RotationGesture`, `DoubleTapGesture` and `PanGesture` are each documented "Only available on **`macOS`** and **`iOS`**". `[VERIFIED]` Bevy's mobile example comments confirm it: `recognize_rotation_gesture: true, // ... This doesn't work on Android` and `// Rotation gestures only work on iOS`.

**There is no gesture recognition on Android in Bevy 0.18. Long-press must be implemented by hand:** track `TouchPhase::Started` (or `Pointer<Press>`) with its position and a wall/`Time` stamp, cancel on `Ended`/`Cancelled` or on movement beyond a slop radius (Android's own `ViewConfiguration` uses ~8 dp touch slop and a 500 ms long-press timeout — reasonable values to copy), and fire when the timer elapses.

`[INFERENCE]` For an idle game with hold-to-buy-repeatedly interactions this is straightforward but must be built, and must be built once in a shared place rather than per-button.

### 4.4 Known touch issues

- `[VERIFIED]` **#16798 — "(Android) Crash when there's more than eight touches at once"** (open, `S-Blocked`, `C-Dependencies`). Panic in `android-activity-0.6.0/src/game_activity/input.rs:136`: `Pointer index 8 is out of bounds`. Still `android-activity 0.6` in Bevy 0.18. `[INFERENCE]` A portrait idle game is unlikely to see 9 simultaneous fingers, but a child mashing the screen will. Cheap mitigation: nothing you can do in Bevy — it's an upstream panic. Worth knowing it exists.
- `[VERIFIED]` **#7528 — "on Android, touch position may not be correctly reported"** (open since 2023-02-06). Root cause per the issue: winit conflates a window's "inner" size with the surface size; on Android the inner size should arguably be the *inset* size. Links to `rust-windowing/winit#2308`. `[INFERENCE]` This is directly relevant to a portrait game with a display cutout and a gesture navigation bar — touch coordinates and rendered coordinates can disagree near screen edges. Test this early on a real notched device (ticket 04); it may constrain how close to the edges tap targets can sit.
- `[VERIFIED]` **#23003 — "Device safe area insets (for iOS and Android)"** (open, 2026-02-23). Bevy has **no safe-area / inset API**. `[INFERENCE]` Load-bearing for tickets 02 and 10: a portrait layout must reserve top and bottom margins by hand, or query insets through JNI. Bevy's example manifest goes fullscreen-immersive and simply ignores the problem.
- `[VERIFIED]` **#20638 — "`Camera.computed.target_info` is none during `PostUpdate` only on mobile"** (open, 2025-08-18). Do world↔screen conversions in `Update`, not `PostUpdate`, on mobile.

---

## 5. Known sharp edges for 2D

### 5.1 Rendering — the one that should change plans

`[VERIFIED]` **#22925 — "Crash on certain Android devices with adreno gpus with `No map for format` errors when using custom `Material2d`"** (open, `P-Crash`, `S-Needs-Investigation`, filed 2026-02-12 against **Bevy 0.18.1** with features literally `"2d", "webp"`). Reporter: crashes on Adreno 660 (Android 16), works on a Pixel 6a and in the emulator; two independent crates using custom `Material2d` (`bevy_fast_light`, `bevy_lit`) hit it; switching to a custom render pipeline avoided it.

`[INFERENCE]` **Recommendation: do not build dot-tower's look on custom `Material2d`.** Adreno is the majority Android GPU. Stick to `Sprite`, `Text2d`, and `bevy_ui` for the vertical slice. If a lighting/shader effect becomes necessary, treat "does it survive Adreno" as an explicit gate.

`[VERIFIED]` **#8229 — "Android: panic on some devices when MSAA is enabled"** (open since 2023). Bevy's own example inserts `Msaa::Off` under `#[cfg(target_os = "android")]` with a comment pointing at that issue. `[INFERENCE]` Harmless for us — pixel art wants `Msaa::Off` and `ImagePlugin::default_nearest()` anyway. Set it explicitly.

`[VERIFIED]` **#14710 — "UI elements randomly disappear for some frames on specific android devices"** (open, Samsung Galaxy A34, 0.14-era, minimal repro is a `Sprite` + a UI image). Not confirmed on 0.18. Watch for it.

`[VERIFIED]` **#23754 — "Bevy crashes on Google Pixel 10"** (open, 2026-04-10, `main`, PowerVR D-Series GPU): the mobile example dies with only `Process ... has died: fg TOP` in logcat. `[INFERENCE]` Newer/unusual GPUs are still a live risk; do not assume "if it runs on my phone it runs".

`[VERIFIED]` Bevy does carry Adreno/Mali workaround plumbing — `bevy_render::get_adreno_model()` and `get_mali_driver_version()`, used to disable GPU preprocessing on Adreno ≤730 and Mali driver <r48. So the engine is at least aware of the hardware landscape.

`[VERIFIED]` `#[cfg(not(target_os = "android"))] shadows_enabled` in the example, referencing #8214 (segfault with shadows on some devices) — 3D only, irrelevant to us.

### 5.2 Audio

`[VERIFIED]` `bevy_audio` = `rodio 0.20`, with `cpal 0.15` as an explicit Android-only dependency, and `android_shared_stdcxx = ["cpal/oboe-shared-stdcxx"]`. So the path is rodio → cpal → **oboe** → `libc++_shared.so` (see §1.4). `vorbis` is enabled by the `audio` collection, so `.ogg` works; `.mp3`/`.wav`/`.flac` need extra features.

`[VERIFIED]` **Audio does not pause itself when the app backgrounds.** Bevy's example says so in a comment and handles it manually:

```rust
fn handle_lifetime(mut r: MessageReader<AppLifecycle>, sink: Single<&AudioSink>) {
    for e in r.read() {
        match e {
            AppLifecycle::Idle | AppLifecycle::WillSuspend | AppLifecycle::WillResume => {}
            AppLifecycle::Suspended => sink.pause(),
            AppLifecycle::Running   => sink.play(),
        }
    }
}
// "Pause audio when app goes into background and resume when it returns.
//  This is handled by the OS on iOS, but not on Android."
```

`[INFERENCE]` Because the app is frozen while suspended (§2.5), you cannot fade out — the pause is instantaneous and must happen in the same `Suspended` frame as the save. Bundle both into one "app is going away" system, and make sure it handles the case where no `AudioSink` exists yet (the example gates on `any_with_component::<AudioSink>`).

`[VERIFIED]` Open issue #2705 ("Improve user experience for audio on Linux") shows `bevy_audio` is generally considered rough; many projects use `bevy_kira_audio` instead. I did not verify Kira's Android status — out of scope, flag for later if `bevy_audio` disappoints.

### 5.3 Assets from the APK

`[VERIFIED]` `bevy_asset/src/io/source.rs`: on Android the default reader is `AndroidAssetReader`; the default *writer* is `None`; the watcher is `None` with the message "Android does not currently support watching assets."

`[VERIFIED]` `bevy_asset/src/io/android.rs`: reads through Android's `AssetManager` obtained from `bevy_android::ANDROID_APP`. Notable properties:
- `read()` calls `opened_asset.buffer()?` then `bytes.to_vec()` — **the whole asset is read into memory, twice, with no streaming.** Fine for pixel-art sprites and small RON; a consideration if audio files get large.
- `read_directory()` *is* implemented (filters out `.meta` files), so `load_folder` may work despite the old open issue #9591 ("`AssetServer::load_folder` dont work on wasm and android", 2023, unverified against 0.18).
- `is_directory()` is a documented "HACK" using `open_dir` + `open`.
- Paths are relative to the APK's `assets/` root — the Gradle `assets.srcDirs` line in §1.3 is what makes the Rust `assets/` folder land there.

`[VERIFIED]` No asset hot-reloading on Android; the `file_watcher`/`dev` features are inert there.

`[VERIFIED]` Open issue #24521: "Docs for `bevy::asset::io::android` are not generated" — the module is real but invisible on docs.rs. Read the source, not the docs.

### 5.4 Text

`[VERIFIED]` `default_font` is part of `default_platform` and therefore of `2d`, so there is an embedded fallback font on Android with no extra work.

`[INFERENCE]` Bevy has no access to Android system fonts — everything must be shipped as an asset, which is what a pixel-art game wants anyway. I found **no open `O-Android` issue about text or font rendering**, which is a mildly encouraging negative result. The one text-adjacent Android issue (#25341) is about `EditableText` and the soft keyboard on the *web*, and #13822 asks for IME/soft-keyboard support on mobile — relevant only if dot-tower ever needs text entry (it shouldn't).

`[VERIFIED]` 0.18 changed `TextLayoutInfo::section_rects` → `run_geometry`; not Android-specific, just noting it if any layout code is ported from 0.17 examples.

### 5.5 Other

- `[VERIFIED]` `WinitSettings::mobile()` = `Reactive { wait: 1/60s }` focused, `reactive_low_power(1s)` unfocused. Bevy's example sets it with the comment "Make the winit loop wait more aggressively when no user input is received / This can help reduce cpu usage on mobile devices". `[INFERENCE]` Correct default for an idle game; use it, and use `RequestRedraw` if a system ever needs to force a frame.
- `[VERIFIED]` `bevy_gilrs` and `sysinfo_plugin` come along with `default_platform`. Neither has a current open Android issue. `[INFERENCE]` Leave them; disabling them would mean abandoning the feature collection.
- `[VERIFIED]` `bevy_log` has an `android_tracing` module — logs land in logcat, and Bevy's mobile example raises `LogPlugin` to `Level::DEBUG` with `filter: "wgpu=error,bevy_render=info,bevy_ecs=trace"`.
- `[VERIFIED]` Open #10213: "Running bevy with default features and/or `android_shared_stdcxx` as a feature makes the app freeze before any systems can run" (2023, open, unverified against 0.18). If a device bring-up hangs with a blank screen and no logs, this is the issue to re-read.
- `[VERIFIED]` Open #11402: no built-in way to hide navigation buttons — Bevy's example does it in Java (`setSystemUiVisibility`), which #23491 notes is deprecated API. `[INFERENCE]` Immersive mode will need a small amount of Kotlin/Java in `MainActivity` using `WindowInsetsController`.

---

## Confidence and gaps

**High confidence (read directly from pinned source):** the toolchain, the feature collections, the `AppLifecycle` state machine including the `WillSuspend`/`WillResume` dead-end, surface destruction/recreation, the absence of `onPause`/`onStop`/`onDestroy` hooks, the touch-is-not-mouse behaviour, gesture unavailability, the audio manual-pause requirement, and the asset reader's behaviour.

**High confidence but assembled across four codebases (§3.2):** that the `Suspended` frame runs while the Java main thread is blocked. Each link is verified; the composition is my reasoning. It is the single most load-bearing claim in this document and the one most worth confirming empirically in ticket 04 — log a timestamp before and after the save write in the `Suspended` handler and compare with logcat's activity-lifecycle lines.

**Genuinely uncertain:**
- The ANR budget for that frame. No source gives a number; I declined to invent one.
- Whether `Suspended` reliably precedes `onDestroy` in every teardown path (back button, Recents swipe, "Force stop", `killProcess` under memory pressure). Needs a device.
- Minimum API level for `GameActivity`: Bevy's README says 31, Bevy issue #23491 says 23. Sources disagree.
- Whether the older open issues (#9591 `load_folder`, #10213 freeze, #14710 disappearing UI, #7528 touch position) still reproduce on 0.18. All are open, but most were last touched years ago.
- Bevy's Discord was not reachable, so anything that is common knowledge there but never filed as an issue is missing from this document.

**Deliberately not investigated:** startup/shader-compilation time, memory footprint, APK size, battery drain, Play Console signing/upload mechanics, and `bevy_kira_audio` as a `bevy_audio` replacement.

---

## Sources

Bevy (all at tag `v0.18.0` unless noted):

- https://bevy.org/news/bevy-0-18/ — release notes; release date 2026-01-13; cargo feature collections
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/README.md — Android setup, build, debug, `cargo-apk` deprecation
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/mobile/src/lib.rs — `#[bevy_main]`, `WinitSettings::mobile()`, `Msaa::Off`, `handle_lifetime`
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/mobile/Cargo.toml
- https://github.com/bevyengine/bevy/tree/v0.18.0/examples/mobile/android_example — `app/build.gradle`, `AndroidManifest.xml`, `MainActivity.java`, `gradle/libs.versions.toml`
- https://github.com/bevyengine/bevy/blob/v0.18.0/examples/mobile/android_basic/readme.md — the deprecated `cargo-apk` path
- https://github.com/bevyengine/bevy/blob/v0.18.0/.github/workflows/validation-jobs.yml — the `build-android` CI job
- https://github.com/bevyengine/bevy/blob/v0.18.0/Cargo.toml — `2d` / `default_platform` feature collections
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_window/src/event.rs — `AppLifecycle`, `WindowEvent`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_winit/src/state.rs — suspend/resume state machine, surface teardown, `should_update`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_winit/src/winit_config.rs — `WinitSettings::mobile()`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_render/src/view/window/mod.rs — `RemovedComponents<RawHandleWrapper>` → surface drop
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_android/src/lib.rs — `ANDROID_APP`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_derive/src/bevy_main.rs — what `#[bevy_main]` expands to
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_asset/src/io/android.rs — `AndroidAssetReader`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_asset/src/io/source.rs — no `AssetWriter`, no watcher on Android
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_audio/Cargo.toml — rodio/cpal/oboe, `android_shared_stdcxx`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_input/src/gestures.rs — gestures are macOS/iOS only
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_input/src/touch.rs — `TouchInput`
- https://github.com/bevyengine/bevy/blob/v0.18.0/crates/bevy_picking/src/input.rs — mouse and touch pointers
- https://github.com/bevyengine/bevy-website/blob/main/content/learn/migration-guides/0.17-to-0.18.md — feature collections, `bevy_input` source features

Bevy issues:

- https://github.com/bevyengine/bevy/issues/9057 — cannot run in the background on Android (open, P-High)
- https://github.com/bevyengine/bevy/issues/22925 — custom `Material2d` crash on Adreno (open, 0.18.1)
- https://github.com/bevyengine/bevy/issues/23491 — modernise the Android example (open)
- https://github.com/bevyengine/bevy/issues/23754 — crash on Pixel 10 (open)
- https://github.com/bevyengine/bevy/issues/23003 — safe area insets (open)
- https://github.com/bevyengine/bevy/issues/20638 — `target_info` None in `PostUpdate` on mobile (open)
- https://github.com/bevyengine/bevy/issues/19021 — no build instructions under `examples/mobile` (open)
- https://github.com/bevyengine/bevy/issues/16798 — crash with >8 simultaneous touches (open)
- https://github.com/bevyengine/bevy/issues/14710 — UI elements disappear on some devices (open)
- https://github.com/bevyengine/bevy/issues/8229 — MSAA panic on some devices (open)
- https://github.com/bevyengine/bevy/issues/7528 — touch position may be misreported (open)
- https://github.com/bevyengine/bevy/issues/10213 — `android_shared_stdcxx` freeze (open)
- https://github.com/bevyengine/bevy/issues/9591 — `load_folder` on Android (open)
- https://github.com/bevyengine/bevy/issues/24521 — `bevy::asset::io::android` docs not generated (open)
- https://github.com/bevyengine/bevy/issues/11402 — hide navigation buttons (open)

Upstream:

- https://github.com/rust-windowing/winit/blob/v0.30.12/src/platform_impl/android/mod.rs — `MainEvent` → winit event mapping; dropped `Pause`/`Stop`/`SaveState`/`Destroy`; `MotionEvent` → `Touch`
- https://github.com/rust-mobile/android-activity/blob/v0.6.1/android-activity/src/lib.rs — crate docs on blocking lifecycle callbacks, ANR, `AndroidApp` as global state, `internal_data_path`
- https://github.com/rust-mobile/android-activity/blob/v0.6.1/android-activity/src/game_activity/mod.rs — `poll_events`, `pre_exec_cmd` → callback → `post_exec_cmd`
- https://github.com/rust-mobile/android-activity/blob/v0.6.1/android-activity/android-games-sdk/game-activity/prefab-src/modules/game-activity/src/game-activity/native_app_glue/android_native_app_glue.c — `android_app_set_window` blocking on `pthread_cond_wait`
- https://github.com/bbqsrc/cargo-ndk — README, `--link-libcxx-shared`
- https://github.com/rust-mobile/xbuild — repo description "(unmaintained)"
- https://github.com/rust-mobile/cargo-apk — README, `NativeActivity` only
- https://github.com/mzdk100/cargo-apk2 — maintained fork
- https://crates.io/api/v1/crates/{cargo-apk,xbuild,cargo-ndk,cargo-apk2} — release dates quoted in §1.2

Android platform:

- https://developer.android.com/google/play/requirements/target-sdk — API 36 required from 2026-08-31
- https://developer.android.com/guide/practices/page-sizes — 16 KB page support, NDK r28 default, linker flags, verification commands
- https://developer.android.com/games/agdk/game-activity — `GameActivity`
- https://developer.android.com/guide/components/activities/activity-lifecycle — Activity lifecycle ordering
