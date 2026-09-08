# Upstream state of bevy_ui on Android

Type: research
Status: open

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
