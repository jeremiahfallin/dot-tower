# Prototype 22 — is `bevy_ui`'s vertex layout misaligned for PowerVR?

Throwaway. Built for [ticket 22](../../issues/22-bevy-ui-android-rendering.md).
Answers one question: **is the corruption a vertex-attribute alignment bug?**

**Status: built and desk-verified, verdict outstanding — it needs the phone.**
No device was attached in the session that built this.

## The hypothesis

`bevy_ui` and `bevy_sprite` draw into the same frame, on the same device, with
the same driver. One is correct and one is corrupt. Their vertex layouts differ
in alignment — though **not only** in alignment; see "What the contrast does not
settle" below.

**Sprite — renders correctly.** Hand-written layout, `bevy_sprite_render/src/render/mod.rs:216`:

| loc | format | offset | 16-aligned |
|-----|--------|--------|------------|
| 0–4 | `Float32x4` ×5 | 0, 16, 32, 48, 64 | yes, all |

Stride **80** — a multiple of 16, so that holds at every vertex index.

**UI — corrupt.** Built by `VertexBufferLayout::from_vertex_formats`, which packs
tightly (`offset += format.size()`, no padding), `bevy_ui_render/src/pipeline.rs:58`:

| loc | field | format | offset | 16-aligned |
|-----|-------|--------|--------|------------|
| 0 | position | `Float32x3` | 0 | — |
| 1 | uv | `Float32x2` | 12 | — |
| 2 | **color** | **`Float32x4`** | **20** | **no** |
| 3 | flags | `Uint32` | 36 | — |
| 4 | **border radius** | **`Float32x4`** | **40** | **no** |
| 5 | **border thickness** | **`Float32x4`** | **56** | **no** |
| 6 | size | `Float32x2` | 72 | — |
| 7 | point | `Float32x2` | 80 | — |

Stride **88** — *not* a multiple of 16, so the misalignment also shifts from one
vertex to the next.

Not one vec4 in the UI layout is 16-byte aligned; every vec4 in the sprite
layout is. Those offsets are measured, not hand-computed — see
`ticket_22_stock_baseline` in `bevy_ui_render/src/lib.rs`.

**The claim:** the PowerVR D-Series driver mis-fetches vec4 vertex attributes
that are not 16-byte aligned, and does it silently.

## Why it fits every observation ticket 04 recorded

- ~~**Silent.**~~ **Withdrawn by ticket 23.** The layout *is* legal — core
  Vulkan requires only component-size alignment
  (`VUID-vkCmdDraw-format-10390`), and `wgpu-core` encodes exactly that
  (`attribute.format.size().min(4)`). But the silence is not evidence for this
  hypothesis over any other: wgpu loads `VK_LAYER_KHRONOS_validation` only if
  present and logs its absence below Bevy's filter, stock retail Android does
  not ship it, and the modern alignment VUs are unimplemented anyway. Most
  likely there was **no validation layer**. What survives is the sharper point:
  the layout being legal means this hypothesis requires a plain **driver
  conformance failure**.
- **Sprite layer correct, UI layer corrupt, same frame.** Alignment is *a*
  structural difference between them — not the only one. See "What the contrast
  does not settle" below.
- **Buttons render as nothing at all, rather than as garbage.** With `radius`
  and `border` garbage, `sd_inset_rounded_box` returns garbage, and both
  `draw_uinode_background` and `draw_uinode_border` end in
  `saturate(color.a * t)` — a `t` of zero is a fully transparent node, not a
  visible artefact. A solid-colour node disappears; textured text still samples
  the glyph atlas and survives as fragments. That is exactly what was seen.
- **Frame-varying.** The UI vertex buffer is rewritten every frame (the HUD
  string is rebuilt each frame), so a fetch reading the wrong bytes reads
  *different* wrong bytes each frame.
- **All of `bevy_ui`, not just text.** Every UI pipeline —
  `ui`, `box_shadow`, `gradient`, `ui_texture_slice`, `ui_material` — starts
  `Float32x3, Float32x2`, putting attribute 2 at byte 20, and all five use
  `from_vertex_formats`. The hypothesis predicts the whole crate fails, which
  is what ticket 04 saw.
- **Renders on macOS/Metal.** Metal requires 4-byte attribute alignment; this
  layout is legal there. Consistent with the desktop control passing.

## What it is not

Not #14710, which is a flicker/synchronisation bug. If this hypothesis holds,
ticket 13's deciding argument was answering a different question entirely — the
wgpu 29 fix is real but irrelevant here. That is a finding for
[ticket 23](../../issues/23-upstream-bevy-ui-android.md), not a conclusion of
this prototype.

## Prior art: one strong class-level case, on this exact silicon

Ticket 23 searched the upstream sources properly and changed this section. What
follows supersedes the "none found" this file originally recorded.

**[godot#121005](https://github.com/godotengine/godot/issues/121005)** (open,
2026-07-06) — Pixel 10 Pro XL, **"PowerVR D-Series DXT-48-1536, OpenGL ES 3.2
build 25.1@6794074"**. A `MultiMeshInstance2D` draw is **invisible** whenever the
instance count exceeds 256 and is not a power of two. A legal but unusual vertex
attribute configuration (a large non-power-of-two `glVertexAttribDivisor`) is
**silently mis-fetched**, leaving attributes "reading garbage"; **"no GL errors,
no warnings"**; non-instanced canvas items in the same frame render normally; not
reproducible on desktop. **Every structural row matches probe F.**

It corroborates the **class** — this driver silently mis-fetches vertex
attributes under legal-but-unusual configurations — not the alignment mechanism
specifically, and it is the GLES driver rather than the Vulkan one.

Still true, and still worth knowing:

- **No report matching probe F's signature exists.** Nothing in the Bevy, wgpu,
  naga or Khronos trackers describes sprite-correct/UI-corrupt on PowerVR,
  Imagination or Tensor hardware. **"Tensor G5" returns zero hits in both the
  Bevy and wgpu trackers.**
- **wgpu carries zero PowerVR correctness workarounds** — a vendor-id constant,
  a GLES vendor-string match and one limits exception, against named workarounds
  for Qualcomm, NVIDIA, Intel and MoltenVK. `wgpu#7669` (PowerVR crash,
  `external: driver-bug`) has been open with zero comments since 2025-05-05.
  The thin layer is upstream of Bevy.
- [bevy#7944](https://github.com/bevyengine/bevy/issues/7944) is a **different
  bug**: AMD RDNA2 desktop, Vulkan-only, fixed by DX12, root-caused to an MSAA
  sample-count mismatch. Now read; nothing to do with Android.
- **Imagination's own guidance** says "on some devices, padding each vertex to
  **16-byte boundaries** may also improve performance" — about *stride*, doubly
  hedged, under performance. It supports the 88 → 96 stride change as something
  they think about; it supports no correctness claim about attribute offsets.

## What the contrast does not settle

**The sprite/UI contrast under-determines alignment**, and this file should have
said so where it stated the difference. The two paths differ in *two* ways, not
one: `bevy_sprite_render` is also `VertexStepMode::Instance` and builds positions
from `@builtin(vertex_index)`, so it **fetches no per-vertex attribute at all**.
Any driver bug confined to vertex-rate fetch produces exactly the same split, and
this patch would not touch it. That is fallback hypothesis 1 below, and it is the
reason the clean-negative property matters more than the argument does.

## The patch

Vendored `bevy_ui_render` 0.19.1 with two edits, both marked `TICKET 22 PATCH`:

1. **`src/lib.rs`** — `UiVertex` fields reordered so the three vec4s sit at
   bytes 0, 16 and 32, plus a `_pad: [f32; 2]` rounding the stride 88 → 96 so
   the alignment survives every vertex index. The two struct literals that
   build vertices use named fields, so reordering needed nothing from them but
   the new `_pad`.
2. **`src/pipeline.rs`** — the layout is hand-written from exported constants
   instead of `from_vertex_formats`. **Shader locations are unchanged, so
   `ui.wgsl` is untouched** — only byte offsets and the stride move.

Nothing about what is drawn changes. If the corruption is anything other than
alignment, this patch changes nothing on screen, and that is a clean negative.

## Desk verification (done)

```bash
cargo test -p bevy_ui_render ticket_22
```

Three tests, all passing:

- `cpu_struct_matches_declared_gpu_layout` — `offset_of!` on every `UiVertex`
  field against the exported layout constants. The struct and the layout are
  two independent declarations of the same bytes in two files; if they drift,
  nothing fails to compile and the GPU silently reads the wrong fields.
- `every_attribute_is_naturally_aligned_at_every_vertex_index` — the property
  the patch exists to establish, stride included.
- `stock_layout_misaligns_every_vec4` — reproduces upstream's arithmetic and
  pins the baseline at color @ 20, radius @ 40, border @ 56, stride 88.

Also verified: `cargo check` clean on desktop, `cargo ndk -t arm64-v8a check`
clean for the device, and the APK builds.

**Not verified: anything visual.** The session that built this had no device
attached, and `screencapture` on the dev machine lacks Screen Recording
permission (it returns wallpaper with the menu bar and no windows), so even the
macOS control could not be photographed. The desktop control is 15 seconds of
someone's time and should be done first — see below.

## Before spending device time — two warnings from ticket 23

1. **Run the Godot control on `gl_compatibility` first, not the default.**
   [godot#115171](https://github.com/godotengine/godot/issues/115171) (open) is
   an Android/Vulkan-Mobile crash on **Pixel 10 Pro, ImgTech DXT-48-1536**, with
   a backtrace through `vulkan.powervr.so (IMG_vkCreatePipelineCache+828)`, from
   Play analytics on a shipping game. The probe's `project.godot` sets
   `rendering_method.mobile="mobile"` — that exact path, so it may well crash
   before it renders anything. `run.sh --renderer gl_compatibility` already
   exists. Run both ways and record which one got further.
2. **Bundle `VK_LAYER_KHRONOS_validation` into the APK, or at minimum raise the
   log filter to `debug`.** The harness currently has no diagnostic channel at
   all — worse than "the bug is silent" implies — and nothing further should
   rest on silence until it does.

## Running it

```bash
./apply.sh                 # adds the [patch.crates-io] stanza to Cargo.toml
cargo run                  # desktop control — must still look correct
./scripts/android-run.sh   # build, install, launch, tail the probe log
```

```bash
./revert.sh                # restores stock bevy_ui_render
```

Both scripts are idempotent. `apply.sh` appends one `[patch.crates-io]` stanza
to the root `Cargo.toml`; `revert.sh` removes it and leaves the vendored copy
on disk.

## Reading the result

**Do the desktop control first.** `cargo run` with the patch applied must look
exactly as it did before. If the desktop is now corrupt, the patch itself is
wrong — the struct and the layout disagree — and nothing about the device tells
you anything. The tests above are meant to make this impossible, but they check
the declaration, not the pixels.

Then on device:

- **UI renders correctly** → hypothesis confirmed. The mechanism is vertex
  attribute alignment, the workaround is in hand, and this is an upstream bug
  worth reporting against `bevy_ui_render` (and probably against wgpu or
  Imagination, depending on where the fault is judged to sit). Every probe
  ticket 22 inherited — B, C, D and A's remainder — becomes readable in the
  same session, so budget time for them.
- **UI still corrupt, desktop still fine** → alignment is exonerated and this
  prototype has done its job by eliminating the leading candidate. The next
  hypotheses, in the order the source suggests:
  1. **Per-vertex attribute fetch itself.** The sprite pipeline is
     `VertexStepMode::Instance` and builds positions from
     `@builtin(vertex_index)`; it never fetches a per-vertex attribute at all.
     A driver bug confined to `VertexStepMode::Vertex` would produce the same
     split and this patch would not touch it.
  2. **Varying count or `@interpolate(flat)`.** UI passes 7 locations, 4 of
     them flat; sprite passes 2, 1 flat. Flat interpolation is therefore
     already exonerated in kind, but not at that width.
  Both are cheaply separable with a minimal repro — which is move 4 on the
  ticket and which this prototype does not build.
- **Partially fixed** — e.g. solid nodes appear but text is still wrong — the
  fault is in the glyph atlas path rather than the layout, and
  `ui_texture_slice_pipeline` is the next place to look.

Whichever way it goes, capture it with `adb exec-out screencap -p > shot.png`
rather than a description; ticket 04's readings were lost to prose.
