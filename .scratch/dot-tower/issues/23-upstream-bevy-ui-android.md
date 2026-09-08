# Upstream state of bevy_ui on Android

Type: research
Status: resolved

## Question

Desk research, no hardware. Runs in parallel with
[ticket 22](22-bevy-ui-android-rendering.md) from the moment it exists, and
should — it may hand that ticket its answer for free.

1. **Is #14710 actually closed, and against what?** [Ticket 13](13-bevy-version-target.md)
   chose Bevy 0.19.1 over 0.18 on the single argument that #14710 (bevy_ui
   Android flicker) was fixed transitively via wgpu 29. Verify that: find the
   commit or wgpu release that closed it, confirm 0.19.1 actually ships it, and
   establish what the fix addressed. If the fix was narrower than "bevy_ui
   flickers on Android", ticket 13's reasoning was thinner than recorded.

2. **Are there existing reports of `bevy_ui` corruption on PowerVR or Tensor
   hardware?** Bevy issues, wgpu issues, and the Bevy Discord. The signature to
   match: sprite layer correct, UI layer scattered and frame-varying, **no
   validation error and no wgpu log at all**. The silence is the distinctive
   part — a driver bug that trips no validation layer narrows the search.

3. **Is PowerVR a known-bad wgpu target more broadly?** Tensor G5 moved Google
   from Mali to PowerVR (Imagination D-Series), which is recent enough that the
   whole Rust graphics stack may have little exposure to it. Look for wgpu
   issues naming Imagination, PowerVR, or `DXT-48`. If wgpu itself is thin here,
   that reframes ticket 22 — the answer would be upstream of Bevy entirely.

4. **What do other Bevy projects shipping on Android actually do about UI?**
   Ticket 02 surveyed UI crates for touch and maintenance, not for whether
   anyone has shipped them on an Android device. If the answer is that nobody
   ships `bevy_ui` on Android, that is worth knowing before ticket 22 spends a
   week finding out the hard way.

Capture findings as a Markdown file under `research/` and link it from this
ticket, per ticket 02's and 13's precedent.

## Why this is separate from ticket 22

Ticket 22 needs the phone; this needs a browser. Keeping them apart means the
device-bound work is never waiting on reading, and reading can happen while the
phone is elsewhere. Research tickets are also the one type a session may resolve
several of, so this can be picked up alongside other work.

## Answer

**Resolved.** Findings in
[`research/23-upstream-bevy-ui-android.md`](../research/23-upstream-bevy-ui-android.md)
(660 lines, house style, every claim tagged `[VERIFIED]` or `[INFERENCE]`).
Desk only, no hardware. Bevy's Discord was **not reachable** and stays unsearched
— Q2 names it explicitly, so that surface is still open to whoever can reach it.

**Q1 — #14710 is real, is verified in every link, and is a different bug from
ours.** Research 13's chain checks out: wgpu#8853 → PR #8924 ("Resolves #8853
(and thus bevyengine/bevy#14710)") → wgpu 29.0.0 → `bevy_render` pins `29.0.3`,
and the fixed code is present at the pinned tag. But #8853 is a **missing
pipeline barrier between two render passes writing the same colour attachment** —
a whole-pass write-after-write hazard on tilers. Every device in #14710's
35-comment thread is **Mali or MediaTek/Exynos — no PowerVR anywhere**; the
symptom is uniformly *flicker*; and both community mitigations that worked
("Disable HW Overlays", a no-op post-process pass) are pass-composition remedies.
A barrier bug decides whether the UI pass's output *lands*. It cannot scramble
geometry within the pass, cannot make solid nodes vanish in **every** frame while
glyphs partly survive, and would corrupt at tile granularity rather than glyph
granularity. **Two different failures.** Also: **#14710 was never closed** —
`state: open`, `closed_by: null`, last touched 2026-01-25, and no Bevy PR has
ever referenced it. Nobody has re-tested released 0.19 on affected hardware.

**Q2 — no report matching the signature exists.** Searched Bevy, wgpu, naga and
the Khronos trackers; **"Tensor G5" returns zero hits in both the Bevy and wgpu
trackers**. `bevy#7944`, the closest title and unread until now, is a **different
bug on different hardware**: AMD RDNA2 desktop, Vulkan-only, fixed by DX12,
root-caused to an MSAA sample-count mismatch (PR #9169) and re-broken by revert
#9237. Nothing to do with Android.

**Q3 — PowerVR is not a target wgpu has meaningful exposure to.** Grepping the
whole wgpu tree for `imgtec|powervr|imagination` finds a vendor-id constant, a
GLES vendor-string match and one limits-reporting exception — **zero correctness
workarounds**, against named ones for Qualcomm, NVIDIA, Intel and MoltenVK.
`wgpu#7669` (PowerVR crash, labelled `external: driver-bug`) has sat open with
**zero comments since 2025-05-05**. Godot, by contrast, ships a vendor-wide
PowerVR ban list. This reframes ticket 22 as the ticket suspected it might: the
thin layer is upstream of Bevy.

**Q4 — shipping `bevy_ui` on Android is the norm, so that is not why it is
broken here.** Bevy's own `examples/mobile` has a `// Test ui` button; the
canonical template `NiklasEi/bevy_game_template` (1.1k★) builds its menu from
`Node`/`Button`/`Text` with Android APK and Play AAB CI; and #14710's thread is
nine developers running `bevy_ui` on twelve physical handsets. No *named shipped
commercial* Bevy Android title was found — a bounded negative, surfaces named.

### What it does to ticket 22's hypothesis

**Not contradicted, not confirmed — its evidential base changed shape.** Two
arguments moved in opposite directions, and the net is roughly even.

*Withdrawn:* **"the silence is the tell."** wgpu loads
`VK_LAYER_KHRONOS_validation` only if it finds it, and logs its absence at
`log::debug!` — below Bevy's filter. Stock retail Android does not ship that
layer. And the two modern attribute-alignment VUs (`-10389`, `-10390`) are
**still unimplemented** in the validation layers (`Vulkan-ValidationLayers#9065`,
open since 2024-12-20, `Incomplete`). "No validation failure" most likely means
**"no validation layer."** The silence discriminates nothing and must come off
the list of supporting arguments.

*Also overstated:* "alignment is the **only** structural difference between the
sprite and UI paths." It is not — `bevy_sprite_render` is
`VertexStepMode::Instance` and fetches no per-vertex attribute at all, so any bug
confined to vertex-rate fetch produces the same split. The prototype README says
this; ticket 22's comment did not, and has been corrected.

*Gained, and worth more than what was lost:* **prior art for the class, on the
exact silicon.** `godotengine/godot#121005` (open, 2026-07-06) — Pixel 10 Pro XL,
"PowerVR D-Series DXT-48-1536, OpenGL ES 3.2 build **25.1@6794074**" — a legal
but unusual vertex-attribute configuration (a large non-power-of-two
`glVertexAttribDivisor`) is **silently mis-fetched**, attributes "reading
garbage", the **draw disappears entirely**, "no GL errors, no warnings", other
draws in the same frame fine, not reproducible on desktop. Every structural row
matches probe F. It is corroboration of the *class*, not of the alignment
mechanism, and it is on the GLES driver rather than the Vulkan one — but ticket
22 recorded that no such prior art existed, and now it does.

*Sharpened:* core Vulkan **explicitly permits** a `Float32x4` at a 4-byte
offset. `VkVertexInputAttributeDescription` has four VUs and none constrains
offset alignment; the rule is `VUID-vkCmdDraw-format-10390`, requiring only
component-size alignment. wgpu encodes exactly that
(`attribute.format.size().min(4)`, verified in `wgpu-core` locally). **So the
layout is legal and the hypothesis requires a plain driver conformance
failure** — a higher bar than "Bevy is doing something dubious", and worth
stating plainly.

*Adjacent:* Imagination's own PowerVR Graphics Recommendations say "on some
devices, padding each vertex to **16-byte boundaries** may also improve
performance" — about **stride**, doubly hedged, filed under performance. It
corroborates the prototype's 88 → 96 stride change as something Imagination
think about; it supports no correctness claim about attribute offsets.

### Two things ticket 22 should do differently

1. **Run the Godot control on `gl_compatibility` first.** `godot#115171` (open,
   2026-01-20) is an Android/Vulkan-Mobile crash on **Pixel 10 Pro, ImgTech
   DXT-48-1536**, backtrace through `vulkan.powervr.so
   (IMG_vkCreatePipelineCache+828)`, from Play analytics on a shipping game. The
   probe's `project.godot` sets `rendering_method.mobile="mobile"` — that exact
   path. `run.sh --renderer gl_compatibility` already exists. Run both ways.
2. **Bundle the validation layer, or raise the log filter to `debug`, before any
   further argument rests on silence.** The harness currently has no diagnostic
   channel at all, which is a worse position than "the bug is silent" implies.

### Ticket 13

**A footnote, not an amendment.** Its finding #2 is factually correct in every
particular — the fix is real and 0.19.1 ships it — but it should lose "this is
the real decider", because the fix addresses a bug this project has not
observed. Finding #3 (0.18 is a dead branch receiving no fixes, ever) carries
the decision on its own, and is untouched. Suggested addendum text is in §9 of
the research file. Separately worth noting: ticket 13 §3.5 already listed
`bevy#23754` — "Bevy crashes on Google Pixel 10", still open, `S-Needs-Design`,
`C-Machine-Specific` — three weeks before this handset was chosen for bringup.

Status: resolved
