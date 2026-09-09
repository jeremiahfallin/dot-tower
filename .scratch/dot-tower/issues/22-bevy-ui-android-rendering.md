# Why bevy_ui does not render on Android

Type: prototype
Status: claimed

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

## Comments

### A mechanism, derived from source, and the build that tests it (session of 2026-09-08)

**No device was attached this session**, so none of the five first moves could
be *completed* — every one of them ends in looking at the phone. Rather than
stop, this session went at the question the moves were meant to answer
indirectly, and found a candidate mechanism in the Bevy source. It is built,
desk-verified and packaged as an APK. **The verdict needs the handset and one
minute of someone's time.**

Ticket stays `claimed` only to match ticket 04's precedent for the same
situation. **Take it over freely — there is no continuity to preserve.**

#### The finding

`bevy_ui` and `bevy_sprite` draw into the same frame, on the same device, with
the same driver. One is correct and one is corrupt. **Their vertex layouts differ
in alignment** — see the caveat below, which ticket 23 was right to press on.

`bevy_sprite_render` hand-writes its layout: five `Float32x4` at bytes 0, 16,
32, 48, 64, stride 80. Every attribute 16-byte aligned, and the stride is a
multiple of 16 so that holds at every vertex index.

`bevy_ui_render` builds its layout with `VertexBufferLayout::from_vertex_formats`,
which packs tightly — `offset += format.size()`, no padding. For the UI
attribute list that yields:

| loc | field | format | offset | 16-aligned |
|-----|-------|--------|--------|------------|
| 2 | color | `Float32x4` | 20 | **no** |
| 4 | border radius | `Float32x4` | 40 | **no** |
| 5 | border thickness | `Float32x4` | 56 | **no** |

Stride **88** — not a multiple of 16, so the misalignment also shifts from one
vertex to the next. **Not one vec4 in the UI layout is 16-byte aligned; every
vec4 in the sprite layout is.** Those numbers are measured by a test that
reproduces upstream's arithmetic, not computed by hand.

**Claim: the PowerVR D-Series driver mis-fetches vec4 vertex attributes that are
not 16-byte aligned, silently.**

It fits every observation ticket 04 recorded, including the two that were
hardest to explain:

- ~~**The silence.**~~ **Withdrawn by [ticket 23](23-upstream-bevy-ui-android.md).**
  This was listed here as evidence and is not. wgpu loads
  `VK_LAYER_KHRONOS_validation` only if it finds it and logs its absence at
  `log::debug!`, below Bevy's filter; stock retail Android does not ship that
  layer; and the modern attribute-alignment VUs (`-10389`, `-10390`) are
  unimplemented in the validation layers anyway. "No validation failure" most
  likely means **"no validation layer."** It discriminates nothing.
- **Buttons rendering as *nothing*, not as garbage.** With `radius` and `border`
  garbage, `sd_inset_rounded_box` returns garbage and both `draw_uinode_*`
  functions end in `saturate(color.a * t)`. A `t` of zero is a fully transparent
  node. Solid-colour nodes vanish; textured text still samples the glyph atlas
  and survives as fragments — which is exactly what was seen.
- **Frame-varying.** The UI vertex buffer is rewritten every frame, so a fetch
  reading the wrong bytes reads *different* wrong bytes each frame.
- **All of `bevy_ui`, not just text.** All five UI pipelines (`ui`,
  `box_shadow`, `gradient`, `ui_texture_slice`, `ui_material`) begin
  `Float32x3, Float32x2`, putting attribute 2 at byte 20, and all five use
  `from_vertex_formats`. The hypothesis predicts the whole crate fails.
- **Renders on macOS/Metal.** Metal requires only 4-byte attribute alignment.

#### It is not #14710

#14710 is a flicker/synchronisation bug. If this holds, **ticket 13's deciding
argument was answering a different question entirely** — the wgpu 29 fix is real
and irrelevant here, and 0.19-versus-0.18 would not be the axis at all. That
sharpens [ticket 23](23-upstream-bevy-ui-android.md) Q1 rather than settling it.

#### Held loosely, and why

**No prior report matching this signature exists that I could find** — not in
Bevy, wgpu or Imagination trackers. The hypothesis rests on the structural
difference above, not on anyone else having hit it. The honest counter-argument:
4-byte-aligned vertex attributes are extremely common, so a driver that
mis-fetched them would break a great deal of software and someone would have
noticed. What is *less* common is a `Float32x4` at a 4-byte-aligned offset with
a stride that is not a multiple of 16 — the exact shape here. Only the device
decides this; reading has taken it as far as it goes.
[bevy#7944 "Corruption on some UI elements"](https://github.com/bevyengine/bevy/issues/7944)
is the closest title in the tracker and was not chased.

#### Corrected by ticket 23, which read the upstream sources

[Ticket 23](23-upstream-bevy-ui-android.md) is resolved and moved two of the
arguments above in opposite directions. Net: roughly even, and the hypothesis
still stands.

- **"The silence is the tell" is withdrawn**, struck above.
- **"Differ in exactly one structural way" was overstated.** They differ in two:
  `bevy_sprite_render` is `VertexStepMode::Instance` and builds positions from
  `@builtin(vertex_index)`, so it **fetches no per-vertex attribute at all**.
  Any bug confined to vertex-rate fetch produces the same sprite/UI split, and
  this patch would not touch it. The prototype README already listed that as the
  first fallback hypothesis; this comment claimed more than the evidence allows.
- **Gained, and worth more than what was lost: prior art for the class, on the
  exact silicon.** This comment recorded that none existed.
  [godot#121005](https://github.com/godotengine/godot/issues/121005) (open,
  2026-07-06) is Pixel 10 Pro XL, "PowerVR D-Series DXT-48-1536, OpenGL ES 3.2
  build **25.1@6794074**": a legal but unusual vertex-attribute configuration is
  **silently mis-fetched**, attributes "reading garbage", the **draw disappears
  entirely**, "no GL errors, no warnings", other draws in the same frame fine,
  not reproducible on desktop. Every structural row matches probe F. It
  corroborates the *class*, not the alignment mechanism, and it is the GLES
  driver rather than the Vulkan one.
- **Sharpened: the layout is legal, so this needs a plain driver conformance
  failure.** `VkVertexInputAttributeDescription` has four valid-usage statements
  and none constrains offset alignment; the rule is
  `VUID-vkCmdDraw-format-10390`, requiring only component-size alignment.
  `wgpu-core` encodes exactly that — `attribute.format.size().min(4)`. That is a
  higher bar than "Bevy is doing something dubious", and worth stating plainly.
- **Adjacent:** Imagination's own PowerVR Graphics Recommendations say "on some
  devices, padding each vertex to **16-byte boundaries** may also improve
  performance" — about *stride*, doubly hedged, filed under performance. It
  supports the 88 → 96 stride change as something Imagination think about; it
  supports no correctness claim about attribute offsets.

**And #14710 is definitively not this bug.** wgpu#8853 was a missing pipeline
barrier between two render passes writing the same colour attachment. Every
device in its thread is Mali or MediaTek/Exynos, the symptom is uniformly
flicker, and it was **never closed**. A barrier bug decides whether the UI pass
*lands*; it cannot scramble geometry within the pass.

#### Two things to do differently on device, from ticket 23

1. **Run the Godot control on `gl_compatibility` first.**
   [godot#115171](https://github.com/godotengine/godot/issues/115171) (open) is
   an Android/Vulkan-Mobile crash on **Pixel 10 Pro, ImgTech DXT-48-1536**, with
   a backtrace through `vulkan.powervr.so (IMG_vkCreatePipelineCache+828)`, from
   Play analytics on a shipping game. The probe's `project.godot` sets
   `rendering_method.mobile="mobile"` — that exact path. `run.sh --renderer
   gl_compatibility` already exists. Run both ways.
2. **Bundle the validation layer, or raise the log filter to `debug`, before any
   further argument rests on silence.** The harness has no diagnostic channel at
   all right now, which is a worse position than "the bug is silent" implies.

#### What is built

[`prototypes/22-ui-vertex-alignment/`](../prototypes/22-ui-vertex-alignment/) —
a vendored `bevy_ui_render` 0.19.1 whose `UiVertex` is reordered to put the
three vec4s at bytes 0, 16 and 32, with the stride padded 88 → 96. **Shader
locations are unchanged, so `ui.wgsl` is untouched**; only byte offsets move.
Nothing about what is drawn changes, so if the corruption is anything other
than alignment the patch changes nothing on screen — a clean negative.

Verified at the desk: `cargo check` clean on desktop, `cargo ndk -t arm64-v8a
check` clean, three passing tests (`cargo test -p bevy_ui_render ticket_22`)
covering `offset_of!` agreement between the CPU struct and the declared GPU
layout, the alignment property itself, and the upstream baseline. **A fresh APK
is built and waiting** at `android/app/build/outputs/apk/debug/app-debug.apk`
(3m32s Rust, 14s Gradle).

`apply.sh` and `revert.sh` add and remove the one `[patch.crates-io]` stanza in
the root `Cargo.toml`; both are idempotent and both were exercised.

**Not verified: anything visual, on either platform.** No device, and
`screencapture` on the dev machine lacks Screen Recording permission — it
returns wallpaper with the menu bar and no windows — so even the macOS control
could not be photographed. **Do the desktop control first** (`cargo run`, 15
seconds): if the desktop is now corrupt the patch itself is wrong and the device
tells you nothing.

#### Reading the result, and what each outcome costs

- **UI renders** → mechanism found, workaround in hand, upstream bug worth
  reporting. Every probe this ticket inherited — B, C, D and A's remainder —
  becomes readable in the same session. Budget for them.
- **Still corrupt, desktop fine** → alignment eliminated, which is the leading
  candidate gone for one minute of device time. Next hypotheses, in the order
  the source suggests: (1) **per-vertex fetch itself** — the sprite pipeline is
  `VertexStepMode::Instance` and builds positions from `@builtin(vertex_index)`,
  so it never fetches a per-vertex attribute at all, and a bug confined to
  `VertexStepMode::Vertex` would produce this same split; (2) **varying count** —
  UI passes 7 locations against sprite's 2. Both separate cheaply with the
  minimal repro that is move 4 on this ticket and that this session did not
  build.
- **Partially fixed** → the fault is in the glyph atlas path, and
  `ui_texture_slice_pipeline` is next.

The Godot control (move 1) is untouched and remains the right first cut if the
phone session has time for both — it answers a different and broader question,
and the two do not overlap.

#### Working tree

**Uncommitted, and on `main`.** `Cargo.toml` carries the experiment stanza and
`Cargo.lock` moved with it; the prototype directory is untracked. Nothing here
should reach `main` as-is — branch it, or run `revert.sh` before committing
anything else.

### Tested on hardware: the alignment hypothesis is wrong (session of 2026-09-08)

**The patch does not fix it.** Verified on device, with the vendored crate
confirmed compiled into the APK. The alignment hypothesis is **eliminated**, and
that is this prototype working exactly as designed — it was built with a
clean-negative property precisely so a null result would mean something.

The device was **not** the bringup handset. It is a **Pixel 11 Pro, Tensor G6,
PowerVR C-Series CXTP-48-1536 MC1, driver `25.3@6908880`**, Android 17 / API 37 —
a different SoC generation, a different GPU series and a different driver branch
from ticket 04's Pixel 10 / Tensor G5 / D-Series DXT.

Evidence in [`prototypes/22-ui-vertex-alignment/evidence-pixel11/`](../prototypes/22-ui-vertex-alignment/evidence-pixel11/).

#### 1. The bug reproduces across two PowerVR generations

Stock `bevy_ui` on this handset shows **ticket 04's signature exactly**: sprite
layer pixel-perfect, UI layer scattered glyph fragments, no buttons, no
safe-area frame, frames differing three seconds apart. Two SoC generations, two
GPU series, two driver branches.

**This is no longer plausibly a driver regression.** It is a standing PowerVR
family failure, and every Pixel from the 10 onward is affected. That raises the
stakes on ticket 22 and it removes "wait for a driver update" as a strategy.

#### 2. The alignment patch changes nothing that matters

Patched build: still corrupt, same character. One frame showed partial grey
button geometry at the bottom right that stock never produced — but the next
frame did not, so it is transient corruption, not a partial fix. **Do not read
it as progress.**

Ticket 23 had already withdrawn the "silence" argument and shown the
sprite/UI contrast under-determines alignment. This closes it out: the layout
was legal, the patch made it *more* legal, and the corruption is indifferent.

#### 3. What is now the leading hypothesis

The prototype README's fallback list, unchanged and now promoted:

1. **Per-vertex attribute fetch itself.** `bevy_sprite_render` is
   `VertexStepMode::Instance` and builds positions from
   `@builtin(vertex_index)` — it **never fetches a per-vertex attribute at
   all**. A driver bug confined to `VertexStepMode::Vertex` produces exactly
   this sprite/UI split and the alignment patch would not touch it. This is now
   the best remaining explanation and it is **directly testable**: draw one
   quad two ways in a minimal repro.
2. **Varying count or `@interpolate(flat)` at width.** UI passes 7 locations,
   4 of them flat; sprite passes 2, 1 flat.

Both separate cheaply with the minimal repro — move 4 on this ticket, still
unbuilt, and now the obvious next thing to build.

#### 4. A bigger finding, which is not this ticket's

Stock Bevy 0.19.1 **does not run at all** on the Pixel 11 — it aborts on launch
inside the PowerVR SPIR-V compiler at `IMG_vkCreateComputePipelines`, because
`bevy_render` identifies the Pixel 10 for its GPU-preprocessing demotion with an
exact string match on `"PowerVR D-Series DXT-48-1536 MC1"`. Details, the failed
workarounds and the working one are on branch `wayfinder/pixel11-bringup`. That
is an upstream Bevy bug with a one-line fix and it deserves its own ticket.

#### 5. Still not established

**Whether any non-PowerVR device is affected.** Every reading to date is
PowerVR. Ticket 04's "one borrowed Adreno phone" is still the cheapest way to
settle whether tickets 02 and 10's UI foundation is in danger, and it is still
unpurchased.

### Parked, with the device line, until the desk work is done (session of 2026-09-08)

Deliberately set aside, not dropped. Everything left on the map — tickets 17,
18, 20, 21 — is desk work, so the effort is finishing it before any further
device sessions. Parking is recorded here as a comment rather than by reopening
the ticket, so the frontier scan does not pick this up as takeable.

Un-parking is the next device session, and the batching the cold-build cost
(8 minutes) demands is now part of the plan:

- the **Godot control**, both renderers, `gl_compatibility` first —
  godot#115171 makes Vulkan-Mobile a known crasher on this exact GPU;
- the **minimal repro** (move 4, still unbuilt), now carrying the
  per-vertex-fetch hypothesis the last session promoted to leading;
- the **borrowed-Adreno test** — the one-minute reading that prices every
  strategy question about Android, including whether the UI foundation in
  tickets 02 and 10 is in danger at all;
- [ticket 14](14-android-performance-budget.md)'s supply half, which is
  logcat-visible and not blocked by the broken HUD.

The alignment experiment's `[patch.crates-io]` stanza is reverted from the root
`Cargo.toml` as part of parking: the hypothesis is eliminated, and every build
on the branch should be stock `bevy_ui_render` 0.19.1 again. The vendored crate
stays in place as evidence.

The Pixel 11 launch abort found during the last session is now
[ticket 24](24-pixel11-launch-abort.md), so it survives the parking
independently of this ticket.
