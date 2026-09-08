# Prototype 22 — is `bevy_ui`'s vertex layout misaligned for PowerVR?

Throwaway. Built for [ticket 22](../../issues/22-bevy-ui-android-rendering.md).
Answers one question: **is the corruption a vertex-attribute alignment bug?**

**Status: built and desk-verified, verdict outstanding — it needs the phone.**
No device was attached in the session that built this.

## The hypothesis

`bevy_ui` and `bevy_sprite` draw into the same frame, on the same device, with
the same driver. One is correct and one is corrupt. Their vertex layouts differ
in exactly one structural way.

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

- **Silent.** Core Vulkan permits 4-byte attribute offsets, so the validation
  layers have nothing to report. A conformant-looking-but-wrong fetch trips no
  error, no wgpu log, no naga warning. The silence is the tell.
- **Sprite layer correct, UI layer corrupt, same frame.** The only structural
  difference between them is the one above.
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

## Prior art: none found

Searched before building, so the next person does not repeat it. **No report
matching this signature exists that I could find**, which is a reason to hold
the hypothesis loosely — it rests on the source-level difference above, not on
anyone else having hit it.

- No Bevy, wgpu or Imagination issue describes `bevy_ui` corruption on PowerVR
  or Tensor hardware, and none describes a vec4 vertex-attribute alignment bug
  on PowerVR specifically.
- [bevy#7944 "Corruption on some UI elements"](https://github.com/bevyengine/bevy/issues/7944)
  is the closest title in the tracker and is worth a read by whoever picks this
  up; it was not chased here.
- Weak, general corroboration only: PowerVR is reported as a poorly-supported
  Vulkan target for native games, with corruption expected on some hardware —
  e.g. [supertuxkart#5388](https://github.com/supertuxkart/stk-code/issues/5388),
  [Mesa's PowerVR driver docs](https://docs.mesa3d.org/drivers/powervr.html).
  Consistent with the Tensor G5 being the first Google SoC on Imagination
  D-Series, and with ticket 23's suspicion that the Rust graphics stack has
  little exposure to it.

The counter-argument deserves stating: 4-byte-aligned vertex attributes are
extremely common, so a driver that mis-fetches them would break a great deal of
software, and someone would likely have noticed. What is less common is a
`Float32x4` at a 4-byte-aligned offset with a stride that is not a multiple of
16 — which is the exact shape here. That is why the on-device test decides this
and reading does not.

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
