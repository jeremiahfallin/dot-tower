# Stock Bevy 0.19.1 aborts on launch on the Pixel 11 Pro

Type: research
Status: open

## Question

Get the Pixel 11 launch abort fixed upstream, or at least reported, so the
workaround in `src/lib.rs` can come out. Found during [ticket
22](22-bevy-ui-android-rendering.md)'s hardware session; filed separately
because it is a different bug, in `bevy_render` rather than `bevy_ui`, and it
deserves its own upstream thread.

## The bug

Stock Bevy 0.19.1 does not run at all on the Pixel 11 Pro — SIGABRT on the
"Async Compute T" thread inside the PowerVR SPIR-V compiler:

```
IMG_vkCreateComputePipelines -> CompileShaders
  -> ComputeShaderCompileState -> spvcompiler::getMangledImageTypeString
```

Root cause is upstream and is one line. `bevy_render` already demotes the Pixel
10 off the compute-driven culling path, but it identifies it with an exact
string match:

```rust
if adapter_info.name != "PowerVR D-Series DXT-48-1536 MC1" { return None; }
```

The Pixel 11 reports `"PowerVR C-Series CXTP-48-1536 MC1"`, so the match fails,
Bevy selects `GpuPreprocessingMode::Culling`, builds the culling compute
pipelines, and the driver dies compiling one. The tell in the logs: the Pixel 10
prints "Some GPU preprocessing are limited" while this device prints "GPU
preprocessing is fully supported."

Device: Pixel 11 Pro, Tensor G6, PowerVR C-Series CXTP-48-1536 MC1, driver
`25.3@6908880`, Android 17 / API 37.

Likely the same fault as
[bevy#23754](https://github.com/bevyengine/bevy/issues/23754) ("Bevy crashes on
Google Pixel 10", open, S-Needs-Design), which ticket 13 listed and ticket 23
re-flagged.

## The workaround (shipped on `wayfinder/pixel11-bringup`)

Commit `13f1607` constrains `max_storage_textures_per_shader_stage` in
`WgpuSettings::constrained_limits`, which trips Bevy's `limit_support` check and
lands on `PreprocessingOnly` — the same mode the Pixel 10 is demoted to, and a
configuration known to run. The harness uses no storage textures.

Two failed approaches are recorded because both are instructive:

- `WgpuSettings::limits` is only a request and is resolved away against the
  adapter, so the constraint never landed.
- Zeroing `max_compute_workgroup_size_x` does land, but is too blunt: it also
  invalidates Bevy's own sparse-buffer-update pipeline, whose entry point
  declares a workgroup size of 256, and Bevy quits on the validation error.

## Reported upstream (2026-09-08)

Steps 1 and 2 below are **done**. This ticket was written without visibility
into the session that did them; recording the outcome here.

**Sameness confirmed.** bevy#23754's thread carries the same signature —
`SIGABRT` on the `Async Compute T` thread, PowerVR, traced by its reporters to
shader compilation. Two contributors were actively bisecting it and a
maintainer was engaged, so the finding belonged on that thread rather than in a
new issue, which is where it went.

**Comment posted:**
[bevy#23754 (comment)](https://github.com/bevyengine/bevy/issues/23754#issuecomment-5586670782).
It carries the symbolised driver frames (the thread had the crash but not the
resolved backtrace), the root cause, the two-row table showing both handsets on
0.19.1 diverging on the GPU-preprocessing log line, the `constrained_limits`
workaround with both failed attempts, and the suggested fix — match the vendor
id `0x1010` or a name prefix rather than one literal product string, since as
written every future PowerVR part regresses to a launch crash the day it ships.

Deliberately scoped: it does **not** claim to close the Pixel 10 case (a
reporter there still hit it on driver `25.1`, and this project no longer has a
Pixel 10 to test a fix against), and it keeps the `bevy_ui` corruption out of
the thread — one paragraph flags it as a separate fault, since two bugs in one
thread helps nobody. An offer to test patches on the Pixel 11 is on the record.

## What remains

1. **Track the fix.** When a Bevy release carries a broadened match, delete the
   workaround from `src/lib.rs` and re-verify launch on the handset. Testing a
   maintainer's patch before then was offered and needs the device.

Nothing here is blocked by the parking of tickets 22 and 14, but the
re-verification does need the phone, so it batches with that session.
