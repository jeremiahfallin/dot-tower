# Bevy hello-world on a physical Android device

Type: task
Status: open
Blocked by: 01

## Question

Nothing to decide. Get a minimal Bevy 0.18 2D app — a sprite and a touch handler that moves it — building, installing, and running on a real Android device, using whatever toolchain ticket 01 identified.

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
