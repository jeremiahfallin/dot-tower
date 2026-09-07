# Why bevy_ui does not render on Android

Type: prototype
Status: open

## Question

`bevy_ui` does not render on the bringup handset. The sprite layer is correct
throughout; the UI layer is scattered, frame-varying corruption — no buttons, no
safe-area frame, glyph fragments that differ every frame — and it is silent: no
wgpu error, no Vulkan validation failure, no naga warning.

Split out of [ticket 04](04-android-device-bringup.md), whose own question was
answered. Everything here was established there and is not in doubt:

- **Not the harness.** The same build renders correctly on macOS/Metal.
- **Not the render settings.** Reverting `UiAntiAlias::Off` + `Msaa::Off` +
  `FontSmoothing::None` to Bevy defaults changed the anti-aliasing (so the
  change took effect) and left the corruption identical in character.
- **Specific to the UI pass**, on one device, one GPU, one build: Pixel 10 Pro,
  Tensor G5, PowerVR D-Series DXT-48-1536 MC1, Vulkan, Bevy 0.19.1.

What is *not* distinguished: whether this is **#14710 unfixed** or a **distinct
PowerVR-specific bug**. Both fit every observation. Find out, and find the fix
or the workaround.

## What this ticket inherits

Ticket 04 could not read these because the HUD they are printed on is the thing
that is corrupt. They travel here rather than dying with that ticket.

- **Probe B — UI picking per finger.** Whether `Pointer<..>` observers fire
  per-finger on UI nodes. This is the reading [ticket 02](02-portrait-ui-approach.md)'s
  `Interaction` ban rests on, and it is still an inference.
- **Probe C — safe area.** Whether `AndroidApp::content_rect()` accounts for
  display cutouts or only system bars. **Added to ticket 04 by ticket 02**, and
  it must not be lost with it. One finding here already stands regardless: winit
  logs `TODO: handle Android InsetsChanged` and `TODO: find a way to notify
  application of content rect change`, so insets are **never delivered** and the
  ~40 lines ticket 02 budgeted must poll rather than listen.
- **Probe D — text crispness.** What `UiScale` yields crisp pixel text at
  ~2.625× density. **Also added by ticket 02.**
- **Probe A's remainder.** Edge-coordinate sanity (#7528) and the multi-finger
  high-water mark. `max_fingers = 1` is the only reading captured.

## First moves

Cheapest and most discriminating first. None needs hardware the project lacks.

1. **The Godot control — built, waiting only on the phone.** A Godot probe
   harness carrying the same probes A–F now exists at
   [`prototypes/04-godot-probe/`](../prototypes/04-godot-probe/), exporting a
   signed arm64 APK under `dev.dottower.godotprobe` — deliberately a different
   package id from the Bevy harness's `dev.dottower`, so both sit on the phone
   at once and can be compared back to back in one session. The export chain is
   verified end to end on the dev machine without a device.

   It answers one question and it is the question in the way: **is this device
   capable of compositing a UI layer over a canvas at all?** If Godot's UI
   renders, the device and its driver are exonerated and the fault is Bevy's. If
   Godot corrupts too, this stops being a Bevy problem entirely and the whole
   line of inquiry changes. No second handset, no upstream knowledge, no build
   surgery.

   What it does **not** do is identify the mechanism — only which side of the
   line it falls on. That is still worth more than anything else on this list
   for the effort involved.

2. **Vulkan versus GLES.** Compiled backends are `vulkan`, `metal`, `dx12`,
   `webgl` — **no `gles`** — so every observation so far is Vulkan-only.
   Enabling wgpu's `gles` feature (via a direct `wgpu` dependency for feature
   unification; Bevy does not expose it) says whether this is a Vulkan-path bug.
   The strongest narrowing available from the desk.

3. **Bevy 0.18 versus 0.19 on the same device.** Goes straight at
   [ticket 13](13-bevy-version-target.md)'s deciding argument. If 0.18 corrupts
   identically, #14710 was never the operative issue and 0.19.1 is simply not
   worse. If 0.18 is clean, this is a 0.19 regression and ticket 13's decision is
   actively harmful.

4. **A minimal repro** — one solid-colour UI node, no text. The buttons never
   rendered either, so this is likely all of `bevy_ui` rather than the glyph
   atlas, but an upstream report needs the smallest case.

5. **A second GPU vendor, cheaply.** The corruption is visible to the naked eye,
   so mailing `android/app/build/outputs/apk/debug/app-debug.apk` to anyone with
   a Snapdragon (Adreno) or MediaTek/Exynos (Mali) handset and asking for a photo
   is the whole test — no adb, no loan. Firebase Test Lab's free tier is the
   fallback. That same device is also **the only realistic way to ever test the
   Adreno `Material2d` crash (#22925)** that this PowerVR handset structurally
   cannot, which is what made one borrowed Adreno phone the cheapest next move
   in ticket 04's reckoning.

## What this ticket owns downstream

Ticket 04 was holding these; they block here now.

- **[Ticket 13](13-bevy-version-target.md)'s reason, not its choice.** 0.19.1 was
  chosen because #14710 was fixed transitively via wgpu 29. On hardware that fix
  is not observable. This does not argue for going back — 0.19.1 remains at least
  as good as 0.18 — but whether ticket 13 needs amending waits on the vendor
  test, since a PowerVR-only bug leaves its reasoning intact.
- **The UI foundation in [tickets 02](02-portrait-ui-approach.md) and
  [10](10-portrait-layout-desktop-frame.md).** Both build on `bevy_ui` +
  `bevy_picking`. Neither is invalidated yet. If a second device also corrupts,
  the foundation is in question with no obvious replacement, because ticket 02
  banned the `Interaction` path for multi-touch reasons (#11553) that still hold.

## Not a downgrade

`bevy_ui` failing on Android is serious and **must be fixed regardless of which
vendors it affects**. The Pixel is a mainstream handset and a shipping target, so
a PowerVR-only bug is still a shipping bug. Splitting moved this somewhere it can
be worked properly; it did not lower its priority.
