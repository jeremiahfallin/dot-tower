# Upstream state of bevy_ui on Android

Research for ticket `.scratch/dot-tower/issues/23-upstream-bevy-ui-android.md`.

- **Date of investigation:** 2026-09-08. No hardware; desk only.
- **Primary sources read:** `bevyengine/bevy` issues #14710, #7944, #8894, #23754 and PRs #9169, #9237 (GitHub REST API, full comment threads paginated); `gfx-rs/wgpu` issue #8853 and PR #8924 (body, comments, and the `.diff`); `wgpu` `CHANGELOG.md` at tag `v29.0.0`; `wgpu` source tarball at tag **`v29.0.3`** (`wgpu-core`, `wgpu-hal`, `wgpu-types`, grepped in full); `bevy` source at tag **`v0.19.1`** (`bevy_render/Cargo.toml`, `bevy_ui_render/src/pipeline.rs`, `bevy_sprite_render/src/render/mod.rs`, `bevy_mesh/src/vertex.rs`, `examples/mobile/src/lib.rs`, root and `bevy_internal` `Cargo.toml`); **Vulkan specification** `KhronosGroup/Vulkan-Docs` at tag **`v1.4.362`** (`chapters/fxvertex.adoc`, `chapters/commonvalidity/draw_vertex_binding.adoc`); `KhronosGroup/Vulkan-ValidationLayers` source (`layers/core_checks/cc_drawdispatch.cpp`, `layers/best_practices/*`) and issues #9065, #3733; `KhronosGroup/Vulkan-Docs` issues #1277, #1661; `godotengine/godot` issues #115171, #121005 and PR #111329, plus `drivers/gles3/storage/config.cpp`; Imagination Technologies' own **PowerVR Graphics Recommendations** at `docs.imgtec.com`; Mesa's `docs/drivers/powervr.rst` at `main`; `supertuxkart/stk-code` issue #5388; `forums.imgtec.com` thread 3987; crates.io release metadata for `wgpu`.
- **Method:** GitHub REST API via authenticated `gh` for issue/PR state and full comment threads; `raw.githubusercontent.com` and release tarballs at pinned tags for source; the Vulkan spec read as asciidoc from the Khronos repository at a pinned tag rather than from the rendered registry (`registry.khronos.org` returned HTTP 403/301 to the fetch tool); `grep` over the complete `wgpu` v29.0.3 tree. **Nothing was compiled and nothing was run on a device.** Every claim is tagged `[VERIFIED]` (read directly from a named primary source) or `[INFERENCE]` (my reasoning across sources).
- **Not consulted / could not reach:**
  - **Bevy's Discord.** Not reachable from this environment. Ticket 23 Q2 names it explicitly; that surface is unsearched.
  - **Imagination's Partner Portal and the D-Series driver release notes.** Gated behind licensee login. The 26.1 driver blog post is public and was read, but it contains no bug-fix list, so nothing can be said about whether a newer driver fixes anything.
  - **`registry.khronos.org` rendered spec.** Blocked (403). The same normative text was read from `KhronosGroup/Vulkan-Docs` at tag `v1.4.362`, which is the source that generates it.
  - **Google Play.** No systematic survey of shipped Bevy Android titles was possible; §7's negative result is bounded by the surfaces named there.
  - **RenderDoc / a GPU capture.** Needs the device.
  - **Physical hardware of any kind.**

---

## Summary

**The alignment hypothesis survives, but for weaker reasons than ticket 22 recorded, and one of its two supporting arguments has to be withdrawn.** The "silence" argument is not evidence — validation almost certainly was never running. In exchange, a much better piece of corroboration turned up that ticket 22 did not have: **the same GPU and the same driver build silently mis-fetch vertex attributes in Godot, today, in an open bug.**

1. **The #14710 → wgpu#8853 → wgpu#8924 → wgpu 29 chain is real, and every link verifies.** `[VERIFIED]` Research 13 got this right in every particular. wgpu#8853 is `closed`/`completed` at 2026-03-15 by `cwfitzgerald`; PR #8924 is `merged: true` at 2026-03-15 and its body says verbatim "Resolves #8853 (and thus `bevyengine/bevy#14710`)"; the `v29.0.0` CHANGELOG carries it twice; crates.io dates `29.0.0` at 2026-03-19 and `29.0.3` at 2026-05-02; `bevy_render/Cargo.toml` at `v0.19.1` pins `wgpu = "29.0.3"`; and the fixed code is present in the pinned tag — `wgpu-hal/src/vulkan/adapter.rs:3057` at `v29.0.3` returns `TextureUses::INCLUSIVE` only, citing issue 8853 in a comment. See [§1](#1-14710-and-the-wgpu-29-fix-verified-in-full).
2. **But it fixed a different bug from ours, and the difference is not marginal.** `[VERIFIED]` `[INFERENCE]` #8853 is a **missing pipeline barrier between two render passes that write the same colour attachment** — a whole-pass write-after-write hazard on tiled GPUs. Every device in #14710's 35-comment thread is **Mali or MediaTek/Exynos**; the reported symptom is uniformly *flicker* ("randomly disappear for some frames"); and the two community mitigations that worked — Android's "Disable HW Overlays" and inserting a no-op post-process pass — are both pass-composition remedies. A missing barrier decides whether the UI pass's output *lands*; it cannot scramble geometry *within* that pass, cannot make solid nodes vanish in **every** frame while textured glyphs partly survive, and on a tiler would corrupt at tile granularity, not glyph-fragment granularity. **Two different failures.** See [§2](#2-is-it-the-same-class-of-bug-no).
3. **#14710 is still open, was never closed, and nobody has reported a regression — or anything at all — since before the fix merged.** `[VERIFIED]` `state: open`, `state_reason: null`, `closed_by: null`, `updated_at: 2026-01-25`. A GitHub search for `14710` in the body of any Bevy issue or PR returns `total_count: 0` — no Bevy PR has ever referenced it. So: not closed, not reopened, no regression report, and **no one has re-tested released 0.19 on affected hardware.** Unchanged from research 13's read three weeks ago. See [§1.4](#14-the-current-state-of-14710-itself).
4. **No prior report of this signature exists that I could find.** `[VERIFIED]` (as a negative). Nothing in Bevy, wgpu, naga, or the Khronos trackers describes sprite-correct/UI-corrupt on PowerVR, Imagination or Tensor hardware. **"Tensor G5" returns zero hits in both the Bevy and wgpu trackers.** `bevy#7944` — the closest title, and unread until now — is a **different bug on different hardware**: AMD Radeon RDNA2 desktop, Vulkan-only, fixed by DX12, root-caused to an **MSAA sample-count mismatch** between the UI pipeline and the view target (PR #9169), then re-broken when #9169 was reverted by #9237 for breaking tonemapping. It is desktop-only and has nothing to do with Android. See [§3](#3-existing-reports-matching-the-signature-negative).
5. **PowerVR is not a target wgpu has any exposure to. It carries literally zero PowerVR correctness workarounds.** `[VERIFIED]` Grepping the complete `wgpu` v29.0.3 tree for `imgtec|powervr|imagination` yields exactly three code hits: a vendor-id constant (`0x1010`), a GLES vendor-string match, and one *limits-reporting* exception. Qualcomm, NVIDIA, Intel and MoltenVK all have named correctness workarounds in the Vulkan backend; Imagination has none. Meanwhile `wgpu#7669` — "wgpu crashes on Motorola G54 with PowerVR BXM-8-256", labelled `external: driver-bug` — has sat **open with zero comments since 2025-05-05**. See [§4](#4-powervr-as-a-wgpu-target-thin-to-the-point-of-absent).
6. **Godot, on the identical GPU and the identical driver build, has an open bug in which legal-but-unusual vertex attribute state is silently mis-fetched and the geometry disappears entirely.** `[VERIFIED]` `godotengine/godot#121005`, opened 2026-07-06, still open: Pixel 10 Pro XL, "PowerVR D-Series DXT-48-1536, OpenGL ES 3.2 build **25.1@6794074**". A `MultiMeshInstance2D` draw is **invisible** whenever the instance count exceeds 256 and is not a power of two. "No GL errors, no warnings; non-instanced canvas items in the same frame render normally." The reporter's diagnosis: a large non-power-of-two `glVertexAttribDivisor` value is mishandled, "leaving the item transform/modulate attributes reading garbage". **This is the strongest corroboration available for the class of the hypothesis** — same silicon, same driver, silent attribute mis-fetch, total disappearance, other draws in the same frame fine. It is *not* corroboration of the alignment mechanism specifically. See [§5.4](#54-prior-art-for-the-class-of-bug-two-real-cases-one-of-them-on-this-exact-driver).
7. **Core Vulkan explicitly permits a `Float32x4` vertex attribute at a 4-byte-aligned offset. The layout is legal, and the alignment hypothesis therefore requires a plain driver conformance failure.** `[VERIFIED]` `VkVertexInputAttributeDescription` has exactly four valid-usage statements and **none of them constrains offset alignment**. The alignment rule lives at draw time, as **`VUID-vkCmdDraw-format-10390`**: for a non-packed format, `attribAddress` "must: be a multiple of the ... component size of the pname:format". `VK_FORMAT_R32G32B32A32_SFLOAT` is not a packed format, so its required alignment is **4 bytes**. `VK_KHR_portability_subset` adds a *stride* alignment VU (`-04456`) but still adds no offset-alignment VU. See [§5.1](#51-what-vulkan-actually-requires).
8. **wgpu's own validation encodes exactly that rule, and would accept offset 20 for a `Float32x4` without complaint.** `[VERIFIED]` `wgpu-core/src/device/resource.rs:4049` (v29.0.3): `let required_offset_alignment = attribute.format.size().min(4);` — `16.min(4) == 4`, and `20 % 4 == 0`. `wgpu-types` documents `VERTEX_ALIGNMENT = 4` for buffer offsets and strides, so stride 88 is accepted too. **wgpu imposes no alignment requirement on `VertexAttribute::offset` beyond 4 bytes and documents none.** See [§5.2](#52-what-wgpu-requires).
9. **Ticket 22's "the silence is the tell" argument has to be withdrawn. The silence is almost certainly meaningless.** `[VERIFIED]` `[INFERENCE]` wgpu enables `VK_LAYER_KHRONOS_validation` only if it finds the layer in the enumerated instance layers (`wgpu-hal/src/vulkan/instance.rs:651-713`); when the flag is set and the layer is absent it logs at **`log::debug!`**, below Bevy's default filter. A stock retail Android device does not ship that layer — it has to be bundled in the APK or pushed via the debug-layer mechanism. On top of that, the two modern attribute-alignment VUs (`-10389`, `-10390`) are **still unimplemented in the validation layers** — `Vulkan-ValidationLayers#9065`, open since 2024-12-20, labelled `Incomplete`. So the layout would not have been flagged even if validation *had* been running. **"No Vulkan validation failure" in ticket 04 most likely means "validation was never loaded."** See [§6](#6-why-the-silence-proves-much-less-than-it-looks).
10. **Imagination's own guidance does single out 16-byte vertex boundaries — but as a performance suggestion, not a correctness requirement.** `[VERIFIED]` PowerVR Graphics Recommendations, *Optimising Vertex and Index Buffers*: "On some devices, padding each vertex to 16-byte boundaries may also improve performance." That is about **stride**, it is hedged twice ("some devices", "may"), and it is filed under performance. It corroborates the prototype's stride change 88 → 96 as a thing Imagination themselves think about; it does not support a correctness claim about individual attribute offsets. Nothing in Imagination's public documentation states an alignment requirement for correctness. See [§5.3](#53-what-imagination-actually-says).
11. **People do ship `bevy_ui` on Android — that is not the reason it is broken here.** `[VERIFIED]` Bevy's own `examples/mobile/src/lib.rs` at `v0.19.1` contains a `// Test ui` `Button`/`Node`/`Text`; the canonical Android template `NiklasEi/bevy_game_template` (1,147★) builds its menu from `Node`/`Button`/`Text` and ships Android APK and Play-Store AAB CI jobs; and #14710's thread is direct evidence of **nine distinct developers** running `bevy_ui` on physical Android handsets between 2024-08 and 2026-01. What I could **not** find is a named, shipped commercial Bevy Android title. Bevy collaborator `NthTensor`, 2025-09-13: "It is *possible* to ship games to iOS and Android, but not easy." See [§7](#7-what-other-bevy-projects-actually-do-on-android).
12. **The Godot control probe is likely to crash on this handset before it renders anything, and ticket 22 should know that before it spends device time.** `[VERIFIED]` `godotengine/godot#115171`, open since 2026-01-20: "Android / Vulkan Mobile crash on Pixel 10 Pro (ImgTech DXT 48-1536) when creating pipeline cache", with a backtrace through `/vendor/lib64/hw/vulkan.powervr.so (IMG_vkCreatePipelineCache+828)`, from Play Store analytics on a shipping game with 10k+ installs. The probe at `prototypes/04-godot-probe/project.godot` sets `renderer/rendering_method.mobile="mobile"` — the Vulkan Mobile renderer, the exact path in that report. `run.sh --renderer gl_compatibility` is the fallback and the probe already supports it. See [§8.1](#81-run-the-godot-probe-on-gl_compatibility-first).

### Verdict on the alignment hypothesis

**Not contradicted. Not confirmed. Its evidential base has changed shape.**

| Ticket 22's argument | Status after this research |
| --- | --- |
| "Core Vulkan permits 4-byte attribute offsets, so validation has nothing to say" | **`[VERIFIED]`, and sharper than stated** — `VUID-vkCmdDraw-format-10390` requires only component-size (4-byte) alignment, and both wgpu and the validation layers encode exactly that. |
| "The silence is the tell" | **Withdraw.** Validation was almost certainly never loaded, and the relevant modern VUs are unimplemented anyway. The silence discriminates nothing. |
| "Sprite correct, UI corrupt, and alignment is the only structural difference" | **Overstated.** It is not the only one: the sprite pipeline is `VertexStepMode::Instance` and fetches no per-vertex attribute at all. Any bug confined to vertex-rate fetch produces the same split. The prototype README says this; the ticket comment does not. |
| "No prior report matching this signature exists" | **Confirmed as a negative**, across a wider surface than was searched before — and one adjacent case was found that is worth more than a matching one would have been (Godot #121005, §5.4). |
| "It is not #14710" | **`[VERIFIED]`.** §2. |

The honest position: the hypothesis is **plausible, cheap to test, and now has one strong class-level precedent on the exact silicon**, but the observation that made it feel compelling (the silence) has been removed, and the sprite/UI contrast under-determines it. The prototype's clean-negative property is what makes it worth running regardless.

---

## 1. #14710 and the wgpu 29 fix, verified in full

Research 13 §3.4 asserted five links. I re-read each from the source that owns it. **All five hold.** What follows adds the two things research 13 did not establish: what the fix actually *does*, and what #14710 actually *is*.

### 1.1 The wgpu issue and its content

`[VERIFIED]` `repos/gfx-rs/wgpu/issues/8853`, read 2026-09-08:

```
title:        Vulkan backend incorrectly skips barriers
state:        closed
state_reason: completed
closed_at:    2026-03-15T03:23:34Z
closed_by:    cwfitzgerald
created_at:   2026-01-09
labels:       type: bug, area: correctness, backend: vulkan
```

The body — the thing research 13 summarised in one clause — is worth reading in full, because it names the mechanism precisely. `AlbinBernhardssonARM` writes that `wgpu-core` skips barriers when a usage is "ordered", that `COLOR_TARGET | DEPTH_STENCIL_WRITE` was in the global `ORDERED` set, and that:

> [Vulkan makes very few execution ordering guarantees.](https://registry.khronos.org/vulkan/specs/latest/html/vkspec.html#synchronization-implicit) The only guarantees are: order of primitives in the graphics pipeline, the order of image layout transitions ... and the order of pipeline stages within one command ... For instance, on a tiled architecture, two render passes will execute in parallel, regardless of submission order, unless some synchronization primitive is used to ensure the order.

and then, on the case that matters:

> This is the case in Bevy's Android example, which contains two render passes: 1. Render pass 1 renders the 3D scene. 2. Render pass 2 renders a 2D overlay. Both render passes render to the same color attachment. This is a write-after-write hazard, which requires a pipeline barrier ... wgpu incorrectly skips the barrier since both uses of the color attachment are `COLOR_TARGET`, which is considered ordered. This leads to these rendering artifacts on **Immortalis-G715**.

Source: <https://github.com/gfx-rs/wgpu/issues/8853>

**So: a whole-pass write-after-write hazard between the main pass and the UI pass, observed on a Mali Immortalis-G715.**

### 1.2 The fix, and what it changed

`[VERIFIED]` `repos/gfx-rs/wgpu/pulls/8924`: `merged: true`, `merged_at: 2026-03-15T03:23:33Z`, base `trunk`, author `NiklasEi`, merge commit `62c2f5da97c1aef44109dabc069d09429a845fef`. Body verbatim:

> **Connections** — Resolves #8853 (and thus bevyengine/bevy#14710)
>
> **Description** — Currently, ordered usage is defined globally for all hals. According to #8853 this is problematic and the current ordered usages are not correct for Vulkan. This PR moves the ordered usages into the different hals. It also removes the two wrong ordered usages for Vulkan.

`[VERIFIED]` From the PR's `.diff`, `wgpu-types/src/texture.rs` **deletes** the global constant outright:

```
-        const ORDERED = Self::INCLUSIVE.bits() | Self::COLOR_TARGET.bits() | Self::DEPTH_STENCIL_WRITE.bits() | Self::STORAGE_READ_ONLY.bits();
```

and each hal gained a `get_ordered_texture_usages()` / `get_ordered_buffer_usages()` (17 files, `wgpu-hal/src/{vulkan,dx12,metal,gles,noop,dynamic}/`).

Source: <https://github.com/gfx-rs/wgpu/pull/8924>

### 1.3 The fix is present in the exact version Bevy 0.19.1 ships

`[VERIFIED]` `wgpu-hal/src/vulkan/adapter.rs` at tag **`v29.0.3`**, lines 3053-3059 — read directly, not inferred from the changelog:

```rust
// Vulkan makes very few execution ordering guarantees
// see https://registry.khronos.org/vulkan/specs/latest/html/vkspec.html#synchronization-implicit
// We just don't want to insert barriers between inclusive uses
// See https://github.com/gfx-rs/wgpu/issues/8853
fn get_ordered_texture_usages(&self) -> wgt::TextureUses {
    wgt::TextureUses::INCLUSIVE
}
```

`[VERIFIED]` `CHANGELOG.md` at tag `v29.0.0`, header `## v29.0.0 (2026-03-18)`, lists #8924 twice — under *Changes → Hal* ("Make ordered texture and buffer uses hal specific") and under *Bug Fixes → Vulkan* ("Remove incorrect ordered texture uses").

`[VERIFIED]` crates.io: `wgpu 27.0.0` 2025-10-01, `28.0.0` 2025-12-18, `29.0.0` **2026-03-19**, `29.0.1` 2026-03-26, `29.0.3` **2026-05-02**.

`[VERIFIED]` `crates/bevy_render/Cargo.toml` at `v0.19.1`, line 98: `wgpu = { version = "29.0.3", ... }`.

**Research 13's chain is sound end to end.** The fix is not merely in a release Bevy might resolve to; it is in the pinned patch version, and I read the fixed function.

### 1.4 The current state of #14710 itself

`[VERIFIED]` `repos/bevyengine/bevy/issues/14710`, read 2026-09-08:

```
title:        UI elements randomly disappear for some frames on specific android devices
state:        open
state_reason: null
closed_at:    null
closed_by:    null
created_at:   2024-08-11
updated_at:   2026-01-25T08:52:36Z
comments:     35
labels:       C-Bug, A-Rendering, A-Windowing, A-UI, O-Android, S-Needs-Investigation
milestone:    null
```

- **Not closed.** `closed_by` is null; it has never been closed, so it cannot have been reopened.
- **No regression report.** `updated_at` is 2026-01-25 — *before* the fix merged (2026-03-15). Nothing has been posted to it in nearly eight months.
- **No Bevy PR ever referenced it.** `[VERIFIED]` `search/issues?q=repo:bevyengine/bevy 14710 in:body` → `total_count: 0`.

So nobody upstream has confirmed the fix on released 0.19, nobody has refuted it, and nobody has reported it coming back. Research 13's explicit caveat ("high-confidence, not certain") was correctly placed and remains exactly as unresolved as it was.

### 1.5 What #14710 actually looks like on hardware

This is the part research 13 did not extract, and it is what decides §2. From the full 35-comment thread:

`[VERIFIED]` Issue body (`MarcoMeijer`, Bevy 0.14.1, Samsung Galaxy A34): a `NodeBundle` with a `UiImage` and a `SpriteBundle` with the same texture, side by side. "**The ui image is flickering, but the regular sprite is not flickering.**" And: "This issue happened once to me, but never again after that. But on this persons phone it seems to happen almost always."

`[VERIFIED]` Affected hardware named across the thread — **every one of them Mali, MediaTek or Exynos, and not one PowerVR**:

| Reporter | Date | Device / SoC / GPU |
| --- | --- | --- |
| `MarcoMeijer` | 2024-08-11 | Samsung Galaxy A34 |
| `breadbyte` | 2025-06-04 | Redmi 13C — MediaTek Helio G85 |
| `sakateka` | 2025-08-18 | POCO M6 (Helio G91-Ultra), Galaxy A33 5G (Exynos 1280) |
| `WilledgeR` | 2025-11-10 | realme c75 — Mali-G52 MC2 |
| `eyozk` | 2025-11-25 | Galaxy A25 5G (Exynos 1280), Redmi Note 10S (MT6785V) |
| `Litttlefish` | 2025-11-26 | Dimensity 6020 affected; Dimensity 8400 not |
| `analytik` | 2025-12-09 | Oukitel WP17 (Mali-G76 MC4 / Helio G95), Lenovo Yoga Tab 11 (Mali-G76 MC4 / Helio G90T), TCL Nxtpaper 14 (Mali-G57 MC2 / Helio G99) |
| `suprohub` | 2025-12-19 | Infinix NOTE 30 — Helio G99 |
| `AlbinBernhardssonARM` | 2026-01-09 | Mali Immortalis-G715 (in wgpu#8853) |

`[VERIFIED]` Two community mitigations worked, and both are pass-composition remedies:

- `sakateka`, 2025-08-18: "I suddenly found a fix for this issue: before that, the app was just unusable, but after enabling the '**Disable HW Overlays**' setting in Developer Options, it now runs smoothly 99% of the time."
- `eyozk`, 2025-11-25: "I think I got rid of the UI flickering problem by **adding a post-processing effect** ... The shader code doesn't actually do anything." Minimal repro published at <https://github.com/eyozk/bevy-android-ui-flicker>. (It did not work for everyone — `analytik`, 2025-12-09: "adding the eyozk's no-op shader and `WinitSettings::mobile()` did not help.")

`[VERIFIED]` `analytik`, 2026-01-24, narrowing it: "The bug happens with 1. UI, 2. Sprites and 3. meshes with ColorMaterials **IF they have a texture**, but it does not happen to just ColorMaterial meshes."

`[VERIFIED]` `analytik`, 2026-01-25, after testing `NiklasEi`'s patched `wgpu-types`: "I am so very grateful, indeed that fixes things! Tested 4 different Android devices (phone, tablet, eink) and there's **no flickering at all**. Added egui, bevy_ui, text, sprites, mesh2d with a texture and with colormaterial, **all displayed correctly**."

Source: <https://github.com/bevyengine/bevy/issues/14710>

---

## 2. Is it the same class of bug? No.

This is the question ticket 23 asked that research 13 did not. Setting the two side by side:

| | **#14710 / wgpu#8853** | **dot-tower on Pixel 10 Pro** |
| --- | --- | --- |
| Mechanism | Missing image memory barrier between two render passes writing the same colour attachment | Unknown |
| Failure unit | The whole UI **pass** — its output lands or it does not | Individual **primitives** — glyph fragments scattered, solid nodes absent |
| Time behaviour | Intermittent. "randomly disappear for **some frames**"; one reporter saw it once and never again | **Persistent.** Never correct in any frame observed |
| Sprites | **Also affected** (`analytik`, and the issue body's own repro is sprite-vs-UI *both* flickering in some configs) | **Correct throughout** |
| Solid-colour nodes | Flicker like everything else | **Never render at all**, in any frame |
| Hardware | Mali G52/G57/G76/Immortalis-G715, MediaTek Helio/Dimensity, Exynos 1280 | PowerVR D-Series DXT-48-1536 MC1 — **appears nowhere in the thread** |
| Mitigations that worked | Disable HW Overlays; a no-op post-process pass; the wgpu patch | None tried yet |
| Status in our build | **Fixed.** wgpu 29.0.3 code read at §1.3 | Present |

`[INFERENCE]` The structural argument, which I think is decisive:

A missing barrier between pass A and pass B is a **hazard on the shared attachment**. It determines whether B's writes are correctly ordered against A's. Its failure modes are: B's contribution is lost (UI vanishes), A's is lost (background vanishes), or they interleave at whatever granularity the tiler resolves at — **tiles**, on Mali and PowerVR alike. It operates entirely *outside* the passes. It has no access to the contents of B's vertex buffer, no way to select which draws inside B survive, and no reason to treat a textured glyph quad differently from an untextured solid quad, because at the point the hazard occurs both are already pixels.

The observed failure does all three of those things: it varies *within* the UI pass, it discriminates by draw kind (textured survives as fragments, untextured never appears), and its granularity is glyph-sized, not tile-sized. And it is unconditional, where the barrier bug is a race and therefore intermittent by construction — `MarcoMeijer` saw it once and never again on his own handset.

**Conclusion: `[INFERENCE]` these are two different bugs, and the wgpu 29 fix is real, shipped, and irrelevant to what ticket 04 saw.** Ticket 22's comment reached the same conclusion from the Bevy source; it also holds from the upstream side.

One caveat, stated so it is not lost: `[INFERENCE]` I cannot rule out that a barrier bug on a tiler could *look* like scattered fragments if the tiles were small enough and the resolve granular enough. What rules it out is not the scatter — it is the two facts a hazard cannot reach: solid nodes never appearing in **any** frame, and sprites being correct in **every** frame. Those need a mechanism inside the pass.

---

## 3. Existing reports matching the signature (negative)

**No report matching the signature — sprite layer correct, UI layer corrupt, silence, PowerVR/Imagination/Tensor — exists in any tracker I could reach.** Stating the surface so the next person does not repeat it:

`[VERIFIED]` **Bevy tracker** (`repo:bevyengine/bevy`, issue search, all states):

| Query | `total_count` | Relevant hits |
| --- | --- | --- |
| `PowerVR` | 6 | Only **#23754** is a genuine PowerVR report (§4.2). The others are incidental word matches. |
| `Imagination` | 7 | Only **#23754**. The rest match the ordinary English word. |
| `DXT-48` | 1 | **#23754**, and only #23754. |
| `Tensor G5` | **0** | — |
| `Pixel 10` | 220 (tokenised, mostly "pixel") | Only **#23754**. |
| `ui corruption` | 6 | #7944, #8355, #8894 — all desktop; see below. |
| `label:O-Android label:A-UI is:open` | 4 | #14710, #23003 (safe area), #25341 (soft keyboard on web), #9116 (WASM buttons). |
| `vertex alignment` | 23 | Nothing about vertex-attribute alignment. |
| `from_vertex_formats` | 5 | Nothing about padding or alignment. |

`[VERIFIED]` **wgpu tracker** (`repo:gfx-rs/wgpu`): `PowerVR` → 12 hits, one real (**#7669**, §4.1); `Imagination` → 4, none new; `DXT-48` → **0**; `IMG Tec` → **0**; `Tensor G5` → **0**; `android corruption` → **0**; `vertex attribute alignment` → 24, none on point; `unaligned vertex` → 3, none on point. The full `external: driver-bug` label listing (123 issues) was scanned: it contains Adreno, Mali, Intel, AMD, radv, asahi, Apple, WARP and Raspberry Pi entries, and exactly one PowerVR entry (#7669).

`[VERIFIED]` **naga**: `repo:gfx-rs/naga` returns `total_count: 0` for both `PowerVR` and `Imagination`. (naga has lived in the `gfx-rs/wgpu` repository for some years, so the wgpu searches above cover it.)

**Not searched: the Bevy Discord.** Ticket 23 Q2 named it and I could not reach it. If this file is used to decide anything, that gap is real.

### 3.1 bevy#7944 "Corruption on some UI elements" — read, and it is a different bug

`[VERIFIED]` `repos/bevyengine/bevy/issues/7944`: `state: open`, `state_reason: reopened`, created 2023-03-07, 15 comments, last activity **2024-11-28**, labels `C-Bug`, `A-Rendering`, `A-UI`, `S-Needs-Investigation`. **No `O-Android` label.**

Reported by `AmionSky` on **Windows 11 + AMD Radeon RX 6750 XT (discrete desktop), Vulkan backend**, Bevy 0.10.0. Running `contributors`, `blend_modes`, `game_menu`: "Some UI elements have some sort of texture corruption?" and "In the `contributors` example it's happening all the time and the corruption is flickering/changing every frame."

The parts that look like our bug, and the parts that do not:

- **Similar:** UI-only, Vulkan-only, corruption that changes every frame, silent.
- **Different:** desktop AMD RDNA2, not mobile; **`[VERIFIED]` forcing the DX12 backend fixes it** (`AmionSky` 2023-04-02, `nishusb` 2023-05-22, `georgecjl` 2023-08-22) — we have no such escape on Android; and **it was root-caused**.

`[VERIFIED]` The root cause, from PR **#9169** ("Fix UI corruption for AMD gpus with Vulkan", `ickshonpe`, merged 2023-07-19, "Fixes #8894, Fixes #7944"):

> The UI pipeline's `MultisampleState::count` is set to 1 whereas the `MultisampleState::count` for the camera's ViewTarget is taken from the `Msaa` resource, and corruption occurs when these two values are different.

`[VERIFIED]` It was reverted six days later by PR **#9237** (`Elabajaba`, merged 2023-07-25, "Fixes #9234 / re-breaks: The issues that were linked in #9169"): "Any passes that are post msaa resolve need to use the main textures, not the msaa texture." `hymm` reopened #7944 the same day. So the bug is understood, has a known-bad fix, and has been parked since.

`[INFERENCE]` **Not our bug.** An MSAA sample-count mismatch between the UI pipeline and the view target is a concrete, non-alignment mechanism, and ticket 04's control 2 already ran with Bevy-default MSAA on device and saw identical corruption. It is worth a footnote only as further evidence that **`bevy_ui`'s Vulkan path has a history of silent, driver-specific corruption that the sprite path does not share** — which is a pattern, not a diagnosis.

Sources: <https://github.com/bevyengine/bevy/issues/7944>, <https://github.com/bevyengine/bevy/pull/9169>, <https://github.com/bevyengine/bevy/pull/9237>

---

## 4. PowerVR as a wgpu target: thin to the point of absent

### 4.1 wgpu has zero PowerVR correctness workarounds

`[VERIFIED]` I downloaded the `gfx-rs/wgpu` source tarball at tag **`v29.0.3`** — the exact version Bevy 0.19.1 pins — and grepped the entire tree, case-insensitively, for `imgtec|powervr|imagination`. **Three code hits, total:**

| File | What it is |
| --- | --- |
| `wgpu-hal/src/auxil/mod.rs:24` | `pub mod imgtec { pub const VENDOR: u32 = 0x1010; }` — a vendor-id constant. (0x1010 = 4112, matching the `vendor: 4112` in bevy#23754's adapter dump.) |
| `wgpu-hal/src/gles/adapter.rs:164` | `} else if vendor.contains("imgtec") { db::imgtec::VENDOR }` — GLES vendor-string parsing. |
| `wgpu-hal/src/vulkan/adapter.rs:1482` | Membership in `ignore_max_fragment_combined_output_resources`, alongside Intel/NVIDIA/AMD, because "maxFragmentCombinedOutputResources ... is not reported correctly". **A limits-reporting exception, not a correctness workaround.** |

(The only other match anywhere in the tree is a shell comment in `examples/features/src/skybox/images/generation.bash` naming `PVRTexToolCLI`.)

For contrast, `[VERIFIED]` the full set of Vulkan-backend correctness workarounds in the same tree (`wgpu-hal/src/vulkan/mod.rs:400-430`, applied at `adapter.rs:2153-2164`):

```rust
pub struct Workarounds: u32 {
    /// Only generate SPIR-V for one entry point at a time.
    const SEPARATE_ENTRY_POINTS = 0x1;
    /// Qualcomm OOMs when there are zero color attachments but a non-null pointer
    /// to a subpass resolve attachment array. This nulls out that pointer in that case.
    const EMPTY_RESOLVE_ATTACHMENT_LISTS = 0x2;
    /// ... we need to make sure all calls to vkCmdFillBuffer are aligned to 16 bytes
    /// if they cover a range of 4096 bytes or more.
    const FORCE_FILL_BUFFER_WITH_SIZE_GREATER_4096_ALIGNED_OFFSET_16 = 0x4;
}
```

gated on `vendor_id == db::qualcomm::VENDOR` and `vendor_id == db::nvidia::VENDOR` respectively, plus a MoltenVK special case and an Intel outdated-driver check. **Qualcomm, NVIDIA, Intel and Apple each have named quirks. Imagination has none.**

`[INFERENCE]` This is the single most useful framing finding for ticket 22. It is not that wgpu has tried to work around PowerVR and failed; it is that **wgpu has never encountered PowerVR closely enough to quirk it.** If the Pixel 10's driver misbehaves in some legal-but-unusual corner, there is no reason to expect wgpu to have noticed.

`[VERIFIED]` The one open PowerVR issue confirms the same: `gfx-rs/wgpu#7669`, "wgpu crashes on Motorola G54 with PowerVR BXM-8-256 Mobile GPU", opened 2025-05-05, labels `type: bug`, `external: driver-bug`, `platform: android`, `tag: crash`, **still open, zero comments** in sixteen months. The reporter tried four separate wgpu applications (their own, `wgpu-in-app`, `wgpu_winit_example`, Bevy's mobile examples): "All of them crash immediately, as soon as the first command buffer is submitted." And, tellingly: "**Interestingly, rendering seems to work when rendering to a offscreen texture**" — and "the official Vulkan samples ... are able to run on this device."

Source: <https://github.com/gfx-rs/wgpu/issues/7669>

### 4.2 Bevy already has an open Pixel 10 / DXT-48-1536 crash

`[VERIFIED]` `bevyengine/bevy#23754`, "Bevy crashes on Google Pixel 10", opened 2026-04-10 by `mockersf`, **still open**, labels `C-Bug`, `A-Rendering`, `A-Windowing`, `O-Android`, `S-Needs-Design`, `C-Machine-Specific`. Adapter dump from the issue body:

```
AdapterInfo { name: "PowerVR D-Series DXT-48-1536 MC1", vendor: 4112, device: 1896223250,
  device_type: IntegratedGpu, driver: "PowerVR D-Series Vulkan Driver",
  driver_info: "24.3@6660496", backend: Vulkan, ... }
```

— the exact GPU in ticket 04's handset. Running Bevy's mobile example on `main`, it SIGABRTs. `[VERIFIED]` `SkiFire13` reproduced it and isolated the log line immediately preceding the abort:

```
spvcompiler   E  Unhandled sampler flag combo
libc          A  Fatal signal 6 (SIGABRT), code -1 (SI_QUEUE) in tid 25364 (Async Compute T)
```

i.e. **the PowerVR driver's own SPIR-V compiler aborting the process.** `mockersf`, 2026-04-10: "shader compilation that makes android crash... yay for a new phone with bad Vulkan support 🎉".

`[VERIFIED]` `jordandavidson`, 2026-05-16, on a later driver: same crash with `driver_info: "25.1@6794074"`, and "Imagination have released a new 26.1 driver update this week, and that should hopefully get rolled out to Pixel 10 devices at some point. The driver behaviour differences between 24.3 and 25.1, and potentially 26.1 may be something else we see."

`[VERIFIED]` That 26.1 blog post (Imagination, 2026-05-11) was read. It announces Vulkan Roadmap 2026 support, `VK_EXT_graphics_pipeline_library`, and "preview support for Android 17 Vulkan requirements". **It contains no bug-fix list, no known-issues list, and no mention of D-Series.** Distribution is "through Partner Portal", i.e. licensees only. `[INFERENCE]` Nothing can be concluded from it about whether the Pixel 10's driver improves; but it does establish that at least three driver generations (24.3, 25.1, 26.1) are in play and that the device's driver version is a variable ticket 22 should record.

`[INFERENCE]` #23754 is a *different* failure from ours (a hard crash in shader compilation, on the 3D mobile example, which our 2D-only build does not exercise). What it establishes is that **Bevy already knows this specific GPU is broken, and has for five months, with no fix and `S-Needs-Design`.**

Sources: <https://github.com/bevyengine/bevy/issues/23754>, <https://blog.imaginationtech.com/imagination-gpu-driver-26.1-vulkan-advancements-and-android-17-preview>

### 4.3 Godot, by contrast, ships vendor-wide PowerVR workarounds — and has two open Pixel 10 bugs

`[VERIFIED]` `godotengine/godot`, `drivers/gles3/storage/config.cpp`:

```cpp
} else if (rendering_device_name.contains("PowerVR")) {
    disable_transform_feedback_shader_cache = true;
}
```

`[VERIFIED]` Landed as PR **#111329**, "Add all PowerVR devices to the transform feedback shader cache ban list", merged 2025-10-20 by `clayjohn` (a Godot rendering maintainer). His rationale, verbatim:

> Based on the feedback from @kisg and the comments in #94915 it seems safest to just disable the shader cache for transform feedback shaders **for all PowerVR devices**. From what we have learned the latest devices may no longer have the bug that causes this crash. But for users it is much better for us to put a stop to this crash once and for all ... so that we can quickly help reduce crash rates for released games

`[INFERENCE]` That is what a mature engine's PowerVR relationship looks like: a vendor-wide ban list, driven by crash analytics from shipped titles. wgpu has nothing comparable, because it has no comparable exposure.

The two open Godot bugs on **the exact GPU in ticket 04's handset** are §5.4 (#121005) and §8.1 (#115171).

### 4.4 Vendor-independent corroboration, weakly

`[VERIFIED]` **Mesa's `docs/drivers/powervr.rst`** (read at `main`) says its PowerVR Vulkan driver is conformant only on `BXM-4-64` and `BXS-4-64`, requires `PVR_I_WANT_A_BROKEN_VULKAN_DRIVER=1` for anything else, and warns that for partially-supported hardware "instability and corruption are to be expected until additional feature support and workarounds are in place". **D-Series does not appear in any of its three tables.** *Caveat, and it matters:* this is Mesa's own open-source driver, not the proprietary `vulkan.powervr.so` blob running on the Pixel 10. It says nothing directly about our device. It is evidence about how thin Imagination's Vulkan surface is in the open ecosystem, and nothing more. (Ticket 22's prototype README cited this; the citation is accurate but should carry this caveat.)

`[VERIFIED]` **`supertuxkart/stk-code#5388`** (cited by ticket 22's README, verified here): "Random bugs/glitches when using IMG PowerVR GPU with Vulkan rendering", PowerVR Rogue GE8322 on a Unisoc SC9863A, Android 10, with video. The reporter also names Helio P22/G35, P90/P95 and Dimensity 930/7020 as "very poor compatibility when testing native games/benchmark with Vulkan" — 3DMark Wild Life, Warzone Mobile, Asphalt Legends Unite. Closed 2026-05-02. Rogue, not D-Series; weak, vendor-level corroboration only.

`[VERIFIED]` **`forums.imgtec.com` thread 3987** (linked from wgpu#7669): broken Vulkan on Motorola G54 / BXM-8-256 across Android 14 QPR2/QPR3 and 15 DSU. Imagination's own reply is the notable part: "GPU driver updates are not managed by Imagination Technologies, but by the SoC manufacturer." `[INFERENCE]` On the Pixel 10 that means Google, not Imagination, controls when a fixed driver ships — which is the same situation, and it means the Imagination 26.1 announcement does not translate into a delivery date.

---

## 5. The alignment hypothesis, against the specifications

### 5.0 The layout, re-derived independently

Before testing the hypothesis I re-derived its numbers from `v0.19.1` rather than trusting the prototype.

`[VERIFIED]` `crates/bevy_mesh/src/vertex.rs:909-929` at `v0.19.1` — `VertexBufferLayout::from_vertex_formats`, doc comment "Creates a new **densely packed** `VertexBufferLayout`":

```rust
let mut offset = 0;
for (shader_location, format) in vertex_formats.into_iter().enumerate() {
    attributes.push(VertexAttribute { format, offset, shader_location: shader_location as u32 });
    offset += format.size();
}
VertexBufferLayout { array_stride: offset, step_mode, attributes }
```

No padding, ever. `[VERIFIED]` `crates/bevy_ui_render/src/pipeline.rs` feeds it `[Float32x3, Float32x2, Float32x4, Uint32, Float32x4, Float32x4, Float32x2, Float32x2]` with `VertexStepMode::Vertex`. Evaluating: offsets **0, 12, 20, 36, 40, 56, 72, 80**; stride **88**. The three `Float32x4`s land at **20, 40, 56** — none 16-aligned — and 88 is not a multiple of 16, so the misalignment rotates per vertex index. **Ticket 22's table is exactly right.**

`[VERIFIED]` `crates/bevy_sprite_render/src/render/mod.rs:215-...` at `v0.19.1` hand-writes `array_stride: 80`, five `Float32x4` at 0/16/32/48/64 — and, in the same literal, **`step_mode: VertexStepMode::Instance`**.

`[INFERENCE]` **That last word is the problem with the sprite/UI contrast as evidence.** The two pipelines differ in *at least two* structural ways — attribute alignment **and** step mode — and the prototype's patch varies only one. The prototype README says this plainly under "UI still corrupt, desktop still fine"; ticket 22's comment does not, and the comment is what the next reader will see. A negative result from the patch eliminates alignment cleanly, which is its value. A positive result confirms alignment. But the observation "sprite correct, UI corrupt" on its own supports *any* hypothesis about vertex-rate attribute fetch equally well, and should not be presented as pointing at alignment.

### 5.1 What Vulkan actually requires

**Question: does core Vulkan permit a `VK_FORMAT_R32G32B32A32_SFLOAT` vertex attribute at a 4-byte-aligned offset? Yes, unambiguously.**

`[VERIFIED]` `KhronosGroup/Vulkan-Docs` at tag **`v1.4.362`**, `chapters/fxvertex.adoc` lines 384-407. `VkVertexInputAttributeDescription` has **exactly four** valid-usage statements, quoted in full:

> * [[VUID-VkVertexInputAttributeDescription-location-00620]] pname:location must: be less than sname:VkPhysicalDeviceLimits::pname:maxVertexInputAttributes
> * [[VUID-VkVertexInputAttributeDescription-binding-00621]] pname:binding must: be less than sname:VkPhysicalDeviceLimits::pname:maxVertexInputBindings
> * [[VUID-VkVertexInputAttributeDescription-offset-00622]] pname:offset must: be less than or equal to sname:VkPhysicalDeviceLimits::pname:maxVertexInputAttributeOffset
> * [[VUID-VkVertexInputAttributeDescription-format-00623]] The <<resources-buffer-view-format-features,format features>> of pname:format must: contain ename:VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT

**Not one of them mentions alignment.** `offset` is bounded above and nothing else.

`[VERIFIED]` The one `VK_KHR_portability_subset` addition to that struct, same block:

> * [[VUID-VkVertexInputAttributeDescription-vertexAttributeAccessBeyondStride-04457]] If the `apiext:VK_KHR_portability_subset` extension is enabled, and slink:VkPhysicalDevicePortabilitySubsetFeaturesKHR::pname:vertexAttributeAccessBeyondStride is ename:VK_FALSE, the sum of pname:offset plus the size of the vertex attribute data described by pname:format must: not be greater than pname:stride ...

That is about reading past the stride, not alignment. `[VERIFIED]` The portability subset's only alignment VU is on the *binding*, not the attribute — `VUID-VkVertexInputBindingDescription-stride-04456`: stride must be a multiple of `minVertexInputBindingStrideAlignment`. `[INFERENCE]` So even the strictest profile Vulkan defines adds no attribute-offset alignment rule; and since MoltenVK reports `minVertexInputBindingStrideAlignment = 4`, stride 88 is legal there too — consistent with the macOS control rendering correctly.

**The alignment rule does exist — but it is a draw-time rule, and it requires 4 bytes, not 16.**

`[VERIFIED]` `chapters/fxvertex.adoc` lines 1140-1175, §*Vertex Input Extraction*:

```
attribAddress = bufferBindingAddress + effectiveVertexOffset + attribDesc.offset;
```

> If ... pname:format is a packed format, `attribAddress` must: be a multiple of the size in bytes of the size of the format as described in <<formats-packed,Packed Formats>>. Otherwise, ... `attribAddress` must: be a multiple of the size in bytes of the **component type** indicated by pname:format (see <<formats,Formats>>).

`[VERIFIED]` This is normative as **`VUID-vkCmdDraw-format-10389`** (packed) and **`VUID-vkCmdDraw-format-10390`** (non-packed), in `chapters/commonvalidity/draw_vertex_binding.adoc`:

> * [[VUID-{refpage}-format-10390]] For each vertex attribute accessed by this command, if its slink:VkVertexInputAttributeDescription::pname:format ... is not a <<formats-packed,packed format>>, ... the value of `attribAddress` ... must: be a multiple of the <<formats,component size of the pname:format>>

`VK_FORMAT_R32G32B32A32_SFLOAT` has no `_PACK` suffix, so it is not a packed format; its component type is a 32-bit float. **Required alignment: 4 bytes.**

`[VERIFIED]` This reading is confirmed by Hans-Kristian Arntzen answering exactly this question on `KhronosGroup/Vulkan-Docs#1661` (2021-11-15):

> `R32G32B32_SFLOAT` is not a packed format (no `*_PACK` enum), so the alignment required is `R32_SFLOAT`, i.e. **4 bytes**.
>
> \> `buffer[binding].baseAddress`
>
> For purposes of alignment, this one is basically just 0, and should be ignored. For purposes of alignment, the `VkBuffer` itself will always be well aligned for any usage required of it ...

**Applying it to Bevy's UI layout:** `attribAddress = bufferBindingAddress + vertexIndex × 88 + 20`. wgpu requires vertex buffer binding offsets to be multiples of 4 (§5.2), the buffer base is well-aligned by construction, `88 % 4 == 0` and `20 % 4 == 0`. `attribAddress` is therefore a multiple of 4 at every vertex index. **`VUID-vkCmdDraw-format-10390` is satisfied. The layout is legal Vulkan.**

`[INFERENCE]` So the hypothesis's premise — "the layout is legal, so validation has nothing to say" — is **verified**. But that cuts both ways: the hypothesis now requires the driver to fail a plain, normative, twenty-year-old requirement on the most ordinary vertex format there is. That is a large claim about a driver that passes Vulkan conformance. It is not impossible (§5.4 shows this driver doing something adjacent, and shows AMD having done something similar), but it should be held as the strong claim it is.

Sources: <https://github.com/KhronosGroup/Vulkan-Docs/blob/v1.4.362/chapters/fxvertex.adoc>, <https://github.com/KhronosGroup/Vulkan-Docs/blob/v1.4.362/chapters/commonvalidity/draw_vertex_binding.adoc>, <https://github.com/KhronosGroup/Vulkan-Docs/issues/1661>

### 5.2 What wgpu requires

**Question: does wgpu impose or document any alignment requirement on `VertexAttribute::offset`? Only 4 bytes, and it would accept offset 20 for a `Float32x4` without complaint.**

`[VERIFIED]` `wgpu-core/src/device/resource.rs:4047-4056` at `v29.0.3`, in `create_render_pipeline`, under a comment citing the WebGPU spec's `validating GPUVertexBufferLayout`:

```rust
let required_offset_alignment = attribute.format.size().min(4);
if attribute.offset % required_offset_alignment != 0 {
    return Err(pipeline::CreateRenderPipelineError::InvalidVertexAttributeOffset {
        location: attribute.shader_location,
        offset: attribute.offset,
    });
}
```

`VertexFormat::Float32x4.size()` is 16; `16.min(4)` is **4**; `20 % 4 == 0`. **Accepted.** The only other offset check in the function is `attribute.offset >= 0x10000000`.

`[VERIFIED]` The stride check immediately above it:

```rust
if vb_state.array_stride % wgt::VERTEX_ALIGNMENT != 0 {
    return Err(pipeline::CreateRenderPipelineError::UnalignedVertexStride { ... });
}
```

and `[VERIFIED]` `wgpu-types/src/lib.rs:187-191`:

```rust
/// [Vertex buffer offsets] and [strides] have to be a multiple of this number.
pub const VERTEX_ALIGNMENT: BufferAddress = 4;
```

**88 % 4 == 0. Accepted.** (Had `VERTEX_ALIGNMENT` been 16, wgpu would itself have rejected Bevy's UI pipeline — worth noting as a sanity check that nothing upstream considers stride 88 remarkable.)

`[VERIFIED]` `wgpu-types/src/vertex.rs:96-104` documents `offset` as, in full, "Byte offset of the start of the input". **No alignment is documented anywhere on the type.**

`[INFERENCE]` So wgpu is not merely permissive here by accident — it implements the WebGPU rule deliberately, cites the spec, and the WebGPU rule and the Vulkan rule agree at 4 bytes for this format. There is no layer between Bevy and the driver that would object.

### 5.3 What Imagination actually says

**Question: is there a known driver requirement that vertex attributes be naturally or 16-byte aligned? For Imagination: a hedged performance suggestion about stride, and nothing about correctness.**

`[VERIFIED]` Imagination Technologies, *PowerVR Graphics Recommendations* → *Optimising Geometry* → *Optimising Vertex and Index Buffers* (<https://docs.imgtec.com/performance-guides/graphics-recommendations/html/topics/optimising-vertex-and-index-buffers.html>), first paragraph, quoted in full:

> For optimal performance on PowerVR graphics cores, a mesh with static attribute data should:
>
> - Use indexed triangle lists.
> - Interleave vertex buffer object (VBO) attribute data.
> - Not include unused attributes.
>
> For optimal vertex shader execution performance, meshes transformed by the same vertex shader, even if compiled into different shader programs, must have the same VBO attribute data layout. **On some devices, padding each vertex to 16-byte boundaries may also improve performance.**

`[VERIFIED]` The adjacent page, *Choosing Attribute Data Types*, was also read in full. It covers FP16/FP32/INT throughput, implicit conversion cost in the USC, and precision trade-offs. **It says nothing about alignment.**

`[INFERENCE]` Reading this honestly:

- It is about **stride** ("padding each vertex"), which is precisely what the prototype's 88 → 96 change does. So the prototype's stride change is the thing Imagination themselves name.
- It is **doubly hedged** — "on some devices", "may also improve performance" — and it sits in a performance guide, under a heading about performance, in a list of performance recommendations. Reading it as a latent correctness requirement is a stretch, and this file should not do so.
- It says nothing at all about the alignment of *individual attributes within* the stride, which is the other half of the prototype's change.

**No public Imagination document I could reach states an alignment requirement for correctness.** The Partner Portal driver documentation is not reachable (see "Not consulted"), so this is a bounded negative.

`[VERIFIED]` For completeness on the other vendors ticket 23 asked about: the Khronos **Vulkan best-practices validation layer** — which does carry vendor-specific advice, including an `kBPVendorIMG` flag alongside Arm, AMD and NVIDIA (`layers/best_practices/bp_constants.h:108-112`) — has **no check of any kind relating to vertex attribute alignment**, for IMG or for anyone. Its IMG-specific checks are: `BestPractices-IMG-vkCreateImage-too-large-sample-count`, `BestPractices-IMG-Texture-Format-PVRTC-Outdated`, and shared Arm/IMG checks about attachment readback, depth-only passes, small indexed draw calls, and mip levels. The only vertex-related performance check in `bp_pipeline.cpp` is `BestPractices-vkCreateGraphicsPipelines-too-many-instanced-vertex-buffers`. `[INFERENCE]` If any GPU vendor had told Khronos that unaligned vec4 attributes were a hazard, this is where it would be, and it is not there.

### 5.4 Prior art for the class of bug: two real cases, one of them on this exact driver

**Question: has anyone reported a vertex-attribute-alignment bug against any Vulkan driver with this shape? Not with this exact shape. But two adjacent cases exist, and one is on the same GPU and driver build as ticket 04's handset.**

#### The strong one — Godot #121005, same GPU, same driver build, silent attribute mis-fetch

`[VERIFIED]` `godotengine/godot#121005`, "2D MultiMesh draws nothing on PowerVR DXT (GL Compatibility) when instance count is >256 and not a power of two", opened 2026-07-06 by `ElodinLaarz`, **still open**, labels `bug`, `topic:rendering`, `topic:2d`, `needs testing`. System information from the issue body:

> Google Pixel 10 Pro XL (mustang), Android 16, **PowerVR D-Series DXT-48-1536**, OpenGL ES 3.2 build **25.1@6794074**, gl_compatibility renderer. Not reproducible on desktop NVIDIA (OpenGL Compatibility) with identical scenes — device-specific.

The symptom, verbatim:

> On this driver, a `MultiMeshInstance2D` draw (TRANSFORM_2D + colors + custom data, ShaderMaterial canvas_item shader) is invisible whenever the drawn instance count is **greater than 256 and not an exact power of two**. **No GL errors, no warnings; non-instanced canvas items in the same frame render normally.**

With a measured sweep: 128/200/256/512/1024/2048/4096 visible; 257/300/511/513/735/1023/1025/1536/2047/2049/3072/4095/4097/5120 black. And the reporter's diagnosis:

> Suspected mechanism: `RasterizerCanvasGLES3::_enable_attributes()` binds the per-item batch attributes (locations 8-15) with `glVertexAttribDivisor(i, p_rate)` where `p_rate == instance_count` ... but **large non-power-of-two divisor values appear to be mishandled by this driver, leaving the item transform/modulate attributes reading garbage** (world transform collapses / modulate zero), **so the whole draw disappears**. ... **GLES 3.2 permits arbitrary divisors, matching the absence of errors.** The 3D path (divisor 1) is unaffected, which fits this being 2D-only.

Workaround: pad the instance count to a power of two. A public minimal repro was posted in the comments.

`[INFERENCE]` Line up the structure against ticket 04's observation:

| | Godot #121005 | dot-tower probe F |
| --- | --- | --- |
| GPU | PowerVR D-Series DXT-48-1536 | PowerVR D-Series DXT-48-1536 MC1 |
| Driver build | `25.1@6794074` | not recorded — **record it** |
| API state | legal but unusual (`glVertexAttribDivisor` with a large non-pow2 divisor) | legal but unusual (`Float32x4` at 4-aligned offset, stride not a multiple of 16) |
| Diagnostics | none — "No GL errors, no warnings" | none — no wgpu error, no validation failure, no naga warning |
| Failure | vertex attributes "reading garbage" → **draw disappears entirely** | radius/border garbage → `saturate(color.a * t)` with `t == 0` → **solid nodes disappear entirely** |
| Other draws in the same frame | "non-instanced canvas items in the same frame render normally" | sprite layer renders correctly |
| Vendor-independent? | "Not reproducible on desktop NVIDIA" | not reproducible on macOS/Metal |

**Every row matches.** The *specific* mechanism is different — divisor handling in the GLES path, not offset alignment in the Vulkan path — so this is not proof of the alignment hypothesis. What it proves is the thing the hypothesis needed and did not have: **this driver, on this silicon, at this driver version, does silently mis-fetch vertex attributes under legal-but-unusual configurations, and the failure mode is total disappearance rather than visible garbage.** Ticket 22 built its hypothesis with the honest note that "no prior report matching this signature exists". This is the closest thing to one, and it strengthens the case materially.

Source: <https://github.com/godotengine/godot/issues/121005>

#### The other one — an AMD driver, via the Vulkan spec's own ambiguity report

`[VERIFIED]` `KhronosGroup/Vulkan-Docs#1277`, "ambiguity regarding vkCmdBindVertexBuffers offsets", opened 2020-05-15, closed 2024-12-20:

> recently we ran into an issue with **broken drawcalls and/or gpu hang on amd hardware** (os+gpu dependent, details below). Tracked it down to `vkCmdBindVertexBuffers` **offset requiring alignment** (in testing it seems 4 was required). Reading through the spec I couldn't really find much about this ...
>
> On windows I only got broken drawcalls with either a 580 or 5700. On linux (radv) 580 seemed fine with lower alignment, however **5700 would hang and crash Xorg**.

`[VERIFIED]` `jeffbolznv` (NVIDIA) answered by quoting the `attribAddress` rule; `oddhack` closed it with "This should be fixed in the 1.4.304 spec update" — which is where `VUID-vkCmdDraw-format-10389/10390` came from (§5.1, and §6's `Vulkan-ValidationLayers#9065` cites the same merge request).

`[INFERENCE]` This is genuine prior art for the *class* — a shipping Vulkan driver producing broken draw calls and GPU hangs from insufficiently aligned vertex data, silently, with the spec unclear enough that the reporter had to guess. But note the direction: there the application was **below** the required 4-byte alignment. Here the application **meets** it. So #1277 does not show a driver demanding more than the spec; it shows one enforcing what the spec (obscurely) already said.

**Searched and found nothing:** no report anywhere in `gfx-rs/wgpu`, `bevyengine/bevy`, `KhronosGroup/Vulkan-Docs` or `KhronosGroup/Vulkan-ValidationLayers` describes a driver that mis-fetches a *spec-legal* `Float32x4` vertex attribute because it is not 16-byte aligned. If ticket 22's patch works on device, **that report does not exist yet and should be written** — against Imagination first (this would be a driver conformance bug against `VUID-vkCmdDraw-format-10390`), and against Bevy second as a workaround request.

---

## 6. Why the silence proves much less than it looks

Ticket 22's comment calls the silence "the tell" and the prototype README calls it the first item of supporting evidence. **On the sources, it is not evidence at all.** Three independent reasons, each verified:

**(a) The validation layer was almost certainly never loaded.** `[VERIFIED]` `wgpu-hal/src/vulkan/instance.rs:651-713` at `v29.0.3`: wgpu enumerates instance layers, looks for `VK_LAYER_KHRONOS_validation`, and only pushes it if found. When `InstanceFlags::VALIDATION` is set but the layer is absent:

```rust
} else {
    log::debug!(
        "InstanceFlags::VALIDATION requested, but unable to find layer: {}",
        validation_layer_name.to_string_lossy()
    );
}
```

`[INFERENCE]` `log::debug!` is below Bevy's default log filter, so the "I could not find the layer" message would not have appeared in logcat either — the absence of validation is itself silent. And on Android the layer only exists in-process if it is packaged into the APK's `jniLibs` (or pushed via the per-app debug-layer property). Ticket 04's build chain was Gradle + `cargo-ndk` with no mention of bundling validation layers. **The overwhelmingly likely reading of "no Vulkan validation failure" is "no validation layer."**

**(b) Even loaded, it would not have flagged an alignment problem of the modern kind.** `[VERIFIED]` `KhronosGroup/Vulkan-ValidationLayers#9065`, "Add new vertex attribute alignment VUs", opened 2024-12-20, **still open**, labelled `Incomplete`, zero comments. Body in full:

> Added in 1.4.304 from https://gitlab.khronos.org/vulkan/vulkan/-/merge_requests/6991 (https://github.com/KhronosGroup/Vulkan-Docs/issues/1277)
>
> - [ ] `VUID-vkCmdDraw-format-10389`
> - [ ] `VUID-vkCmdDraw-format-10390`

Both boxes unchecked, twenty-one months on. `[VERIFIED]` A code search for `10390` across the whole repository hits only `scripts/vk_validation_stats.py` — i.e. the VUID is known to the tooling and implemented nowhere.

**(c) There *is* an older attribute-address alignment check, and our layout passes it.** `[VERIFIED]` `layers/core_checks/cc_drawdispatch.cpp` (lines ~3084-3110) still implements the pre-10390 formulation under the `-02721` VUID:

```cpp
const VkDeviceSize attrib_address = vertex_buffer_offset + vertex_buffer_binding->stride + attr_desc.offset;
VkDeviceSize vtx_attrib_req_alignment = GetVertexInputFormatSize(attr_desc.format);
if (!vkuFormatIsPacked(...) && !vkuFormatIsCompressed(...) && ...) {
    vtx_attrib_req_alignment = SafeDivision(vtx_attrib_req_alignment, vkuFormatComponentCount(attr_desc.format));
}
if (!IsPointerAligned(attrib_address, vtx_attrib_req_alignment)) { /* LogError */ }
```

For `Float32x4`: `16 / 4 = 4`. `attribAddress = (4-aligned) + 88 + 20`, a multiple of 4. **Passes.** (Historical confirmation that this check is live and fires on real bugs: `Vulkan-ValidationLayers#3733`, 2022, where a developer's `vkCmdBindVertexBuffers` offset of `1` produced exactly this error on `VK_FORMAT_R32G32B32_SFLOAT`.)

`[INFERENCE]` **Net:** the silence is consistent with the alignment hypothesis, and equally consistent with every other hypothesis, including ones nobody has thought of. It should be struck from the list of supporting arguments. And it hands ticket 22 a concrete cheap action — **bundle the validation layer** (§8.2) — because right now the harness has no diagnostic channel at all, which is a worse position than "the bug is silent" implies.

---

## 7. What other Bevy projects actually do on Android

**Short answer: `bevy_ui` on Android is not exotic. It is what everyone does, including Bevy itself. Nobody I could find has shipped a commercial Android title with it.**

`[VERIFIED]` **Bevy's own mobile example uses `bevy_ui`.** `examples/mobile/src/lib.rs` at `v0.19.1`, line 144 onward, contains a section commented `// Test ui` spawning `Button` + `Node` + `Text::new("Test Button")` + `TextFont` + `TextColor` + `TextLayout`, with an `Interaction`-driven colour system. So upstream's only mobile example exercises the UI path deliberately. `[INFERENCE]` It is not much of a test bed, though: that same example is the one that SIGABRTs on the Pixel 10 in #23754, and it is 3D (`Camera3d`), so it would not isolate a 2D UI-versus-sprite split.

`[VERIFIED]` **The canonical Android-capable template uses `bevy_ui`.** `NiklasEi/bevy_game_template` (1,147★, last pushed 2026-07-25, "Template for a Bevy game including CI/CD for web, Windows, Linux, macOS, iOS and Android"): `src/menu.rs` builds its menu from `Node`, `Button`, `Text`, `TextFont`, `TextColor`; `Cargo.toml` comments "Bevy's default features are `2d`, `3d`, `ui` and `audio`. We take all of them except for `audio`"; and `.github/workflows/release.yaml` has a `build-for-Android` job (`cargo apk build --package mobile`, `aarch64-linux-android`) plus a referenced `release-android-google-play` workflow for AABs. `[INFERENCE]` This is the most-used starting point for Bevy-on-Android, it is maintained by the same person who fixed wgpu#8853, and it takes `bevy_ui` for granted.

`[VERIFIED]` **Nine developers ran `bevy_ui` on physical Android handsets in #14710's thread** between 2024-08 and 2026-01 (table in §1.5), on Bevy 0.14 through 0.18-dev, across at least twelve distinct devices. They filed a bug about flicker; none of them reported the UI failing to render at all. `[INFERENCE]` That is a meaningful negative on the broadest question ticket 22 is worried about: `bevy_ui` on Android is not categorically broken. Something specific to this device is.

`[VERIFIED]` **One dissenting practitioner.** `WilledgeR`, #14710, 2025-11-27: "I dont think it's related to `GameActivity` or `NativeActivity`. Im using my own window manager and my own android Activity ... the issue still exists it's something related to `bevy_render`. **I have another project which uses my own text rendering and ui crate which uses `wgpu` with no such a problem.** ... This issue is bothering since `bevy 14` or even earlier and beside **poor performance of bevy ui on android** I decided to use only `bevy { default-features = false }`." `[INFERENCE]` One data point that a hand-rolled wgpu UI on Android avoids both the flicker and a performance problem — relevant to tickets 02/10 only if the foundation ever has to be reconsidered, and not otherwise.

`[VERIFIED]` **Upstream's own posture.** `bevyengine/bevy` discussion #20998 ("Future support for exporting Bevy games to Android and iOS", opened 2025-09-13). Collaborator `NthTensor`, 2025-09-13: "It is *possible* to ship games to iOS and Android, but not easy" — adding that improvement is unlikely without more mobile development activity, the project lacking developers debugging mobile ergonomics. Collaborator `Viridia`, 2025-09-14, on vendor engagement: "it's not so much that Bevy is beneath their notice, but rather that Rust is."

**Negative result, bounded.** `[VERIFIED]` A GitHub repository search for `bevy android game` sorted by stars returns twelve results, of which the top two are a template and a `scrcpy` client; the rest are ≤4★ hobby projects. I could find **no named, shipped, commercially distributed Bevy Android title**. Surfaces searched: GitHub repository search, the Bevy issue tracker, `bevyengine/bevy-assets` (one Android-adjacent entry, `bevy-in-app`), and general web search. **Not searched: Google Play systematically, itch.io, or the Bevy Discord.** The contrast with Godot is instructive: Godot #115171 is filed *from Play Store crash analytics on a shipping title with 10k+ installs*. Nothing equivalent exists for Bevy on Android that I could find.

`[INFERENCE]` **For ticket 22 this is reassuring in one direction and not the other.** Nobody has found `bevy_ui` unusable on Android, so the foundation in tickets 02 and 10 is not in question on the evidence. But the population of people who would have noticed a Pixel 10 problem is very small, the Tensor G5 is four months old, and its GPU already has an open crash in the Bevy tracker with `S-Needs-Design` and no owner. **Waiting for upstream is not a plan.**

---

## 8. What this hands ticket 22

Five concrete things, in the order I would spend device minutes on them.

### 8.1 Run the Godot probe on `gl_compatibility` first, and expect Vulkan Mobile to crash

`[VERIFIED]` `godotengine/godot#115171`, opened 2026-01-20 by `akien-mga`, **still open**: "Android / Vulkan Mobile crash on Pixel 10 Pro (ImgTech DXT 48-1536) when creating pipeline cache (analytics report)". Reproducible in 4.5.stable and in a 4.5.1 + Vulkan-Mobile-fixes build. Backtrace frame #00:

```
#00  pc 0x00000000000a831c  /vendor/lib64/hw/vulkan.powervr.so (IMG_vkCreatePipelineCache+828)
#01  ...  RenderingDeviceDriverVulkan::pipeline_cache_create(...)
#02  ...  RenderingDevice::initialize(...)
#03  ...  DisplayServerAndroid::DisplayServerAndroid(...)
```

— i.e. it dies inside the PowerVR driver during display-server construction, before anything is drawn. From Play Store analytics on `Draknek`'s *Spooky Express* (10k+ installs). `[VERIFIED]` No fix references it: a search for `115171` in the Godot repo returns only the issue itself and a sibling report.

`[VERIFIED]` `prototypes/04-godot-probe/project.godot` sets `renderer/rendering_method.mobile="mobile"` — Vulkan Mobile, the exact path in that report — with a comment explaining the choice as apples-to-apples with the Bevy harness. `scripts/run.sh --renderer gl_compatibility` switches it.

**Recommendation.** Run the probe **both ways**, and budget for the Vulkan run failing at launch. If it does, that is not a wasted run: a Godot Vulkan crash on this device *and* a Bevy Vulkan corruption on this device, from two independent engines, would be a strong statement that the fault is in `vulkan.powervr.so`. And `gl_compatibility` still answers the probe's actual question ("can this device composite a UI layer over a canvas at all?"). One caveat: `[VERIFIED]` that renderer has its own confirmed silent bug on this exact device (#121005), though it is scoped to `MultiMesh` with non-power-of-two instance counts, which a Control-node UI probe will not hit.

### 8.2 Bundle the validation layer, because right now there is no diagnostic channel

Per §6, the harness almost certainly ran with no validation layer at all, and wgpu says so only at `debug` level. Two cheap actions:

1. Raise the Rust log filter to `debug` for `wgpu_hal` and look for `InstanceFlags::VALIDATION requested, but unable to find layer` — that one line settles whether validation was ever running.
2. Ship `libVkLayer_khronos_validation.so` in the APK (`jniLibs/arm64-v8a/`) and set `InstanceFlags::VALIDATION`. `[VERIFIED]` The layer implements the `attribAddress` alignment check (§6c), so if it loads and stays quiet, the layout is confirmed conformant *on device* and one hypothesis is bounded. If it starts producing errors nobody has seen, that is worth more than the whole reading exercise.

**Until one of those is done, "it is silent" should not appear in any argument.**

### 8.3 Record the driver version. It is a variable, and it is moving

`[VERIFIED]` Three D-Series driver builds are in evidence and they behave differently: `24.3@6660496` (bevy#23754, Android 16), `25.1@6794074` (bevy#23754 follow-up **and** godot#121005 — the build that silently mis-fetches attributes), and 26.1 (announced 2026-05-11, Partner Portal only, "preview support for Android 17 Vulkan requirements"). Ticket 04's handset is on **Android 17 / API 37**, ahead of every report above. `wgpu::AdapterInfo::driver_info` carries it; log it and put it in the ticket. If the device is on 26.1 and still corrupts, that is a different (and worse) finding than if it is on 25.1.

### 8.4 The minimal repro (move 4) is now the highest-value unbuilt thing

`[INFERENCE]` Given §5.0 — that the sprite pipeline is `VertexStepMode::Instance` and fetches no per-vertex attribute at all — the sprite/UI contrast cannot distinguish alignment from step mode from varying count. A minimal repro can, cheaply, and it is the thing ticket 22 listed and did not build. The discriminating shape is one solid-colour quad drawn four ways in one frame: (a) stock UI layout, stride 88; (b) the patched layout, stride 96; (c) the same attributes at vertex rate with everything 16-aligned but a different *count* of locations; (d) the same data at instance rate. Godot #121005's methodology is worth copying exactly — a **sweep** with frame readback, not a single yes/no. `adb exec-out screencap -p` per configuration.

### 8.5 If the patch works, write the report — nobody has

Per §5.4, no report of this shape exists anywhere I searched. If ticket 22's prototype flips the UI to correct on device, the finding is: a Vulkan driver failing **`VUID-vkCmdDraw-format-10390`** by requiring more than component-size alignment for `VK_FORMAT_R32G32B32A32_SFLOAT`. That is a driver conformance bug, and the primary report belongs with Imagination — though note `[VERIFIED]` their forum position that "GPU driver updates are not managed by Imagination Technologies, but by the SoC manufacturer", so a parallel report to Google is warranted. Secondary reports: `gfx-rs/wgpu` (which has no PowerVR quirk infrastructure at all and would be establishing it — see §4.1, and Godot PR #111329 as the model), and `bevyengine/bevy` against `from_vertex_formats`, where a padding-aware constructor would fix all five UI pipelines at once. Attach the sweep from §8.4; a bare description will not move a driver vendor.

---

## 9. Does ticket 13 need amending?

**No. It needs one footnote, and it does not need its decision revisited.**

`[VERIFIED]` Ticket 13's finding #2 is factually correct in every particular — I re-verified all five links independently in §1, including reading the fixed function in `wgpu-hal` at the pinned tag `v29.0.3`. Nothing in it was wrong.

`[INFERENCE]` What is wrong is the **weight** the ticket put on it. Finding #2 is labelled "**this is the real decider**", and ticket 04 correctly flagged that on hardware the fix is not observable. §2 now establishes *why*: #14710 is a render-pass synchronisation bug on Mali/MediaTek tilers, our failure is something else entirely, and the two would never have been fixed by the same patch. Ticket 13's deciding argument was answering a question that turned out not to be ours.

**But the decision survives on its other legs, all of which are untouched:**

- `[VERIFIED]` **Finding #3 stands and is sufficient on its own.** 0.18 is a dead branch — `0.18.1` (2026-03-04) is the only patch it ever received, and Bevy does not backport to previous minors. `[INFERENCE]` Even in a world where #14710 had never existed, choosing a version that will receive no further fixes ever, in order to avoid a bug that is not the one you have, is not a trade worth making.
- `[VERIFIED]` **Findings #4, #5, #7, #8, #10** (widget stabilisation, the feature-collection changes, the `Interaction` ban, the unchanged Android platform primitives, the release-cadence argument) are all independent of #14710 and were not touched by this research.
- `[VERIFIED]` **The fix in 0.19.1 is real and 0.18 genuinely cannot have it.** So 0.19.1 remains strictly better on this axis, even though the axis turned out not to matter to us.
- `[INFERENCE]` **Going back to 0.18 would not help and might hurt.** If §2 is right, 0.18 would corrupt identically *and* reintroduce a flicker bug we currently do not have.

**The precise amendment.** Ticket 13's finding #2 should keep its `[VERIFIED]` status and gain a dated addendum, something like:

> **Addendum, 2026-09-08 (research 23 §2).** This finding is factually correct: the fix is real, it is in wgpu 29.0.0, and Bevy 0.19.1 pins wgpu 29.0.3, whose `wgpu-hal/src/vulkan/adapter.rs` was read directly to confirm it. **It is, however, not the bug dot-tower has.** #14710 is a missing-barrier synchronisation defect between render passes sharing a colour attachment, reported exclusively on Mali/MediaTek/Exynos hardware, producing intermittent flicker. What ticket 04 observed on PowerVR is persistent, within-pass corruption in which solid nodes never render at all. Two different failures. **This finding should no longer be described as "the real decider"** — finding #3 (0.18 is a dead branch) carries the decision on its own. The version choice is unchanged.

`[VERIFIED]` Ticket 13's own "Open questions" #1 already said the right thing — "`[UNKNOWN]` Whether #14710 is actually fixed on real Android hardware running released 0.19.1 ... **no one has re-tested released 0.19 on an affected device**" — and that remains true today (§1.4). It is worth noting that **that question is still open in its own terms**: our handset is not an affected device, so probe F cannot answer it either way. If a Mali or MediaTek handset ever passes through the project (ticket 22's move 5), running the harness on it answers ticket 13's open question #1 and the vendor-generalisation question in a single session.

**One thing ticket 13 could not have known and should now be recorded somewhere durable:** `[VERIFIED]` ticket 13 §3.5 already listed bevy#23754 ("Bevy crashes on Google Pixel 10", PowerVR D-Series) in its table of other open Android issues, with the note that it "shows the Android device matrix is still turning up crashes". That was three weeks before ticket 04 put a Pixel 10 Pro on the desk. **The bringup handset was already named in an open upstream Bevy crash report at the time it was chosen.** Not a reason to have chosen differently — it is a mainstream shipping target and the crash is in the 3D example — but the project should not be surprised by this device, and the map should say so.
