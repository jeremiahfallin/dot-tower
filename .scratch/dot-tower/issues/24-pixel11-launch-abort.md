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

## What remains

1. **Confirm bevy#23754 sameness** — read the thread against the backtrace
   above; if it is the same fault, the fix belongs there rather than in a new
   issue.
2. **Report upstream**, with the one-line fix: match the PowerVR family (both
   D-Series and C-Series strings, or a prefix match on `"PowerVR"` combined with
   whatever else the demotion actually keys on) rather than one exact device
   string. **This is an outward-facing act — publishing to the Bevy tracker —
   and waits for an explicit go-ahead.** Desk work otherwise.
3. **Track the fix**, and when a Bevy release carries it, delete the
   workaround from `src/lib.rs` and re-verify launch on the handset.

Not blocked by the parking of tickets 22 and 14: steps 1 and 2 need no device.
