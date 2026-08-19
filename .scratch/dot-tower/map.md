# dot-tower — Wayfinder Map

`wayfinder:map`

## Destination

A **vertical-slice design spec** for dot-tower — one tower, three climber types, one hero, a closed idle loop with floor locking and prestige — plus the Bevy architecture decisions that are expensive to reverse: the simulation boundary, the save schema, and config-driven tuning. The slice is proven when it runs on desktop **and on a physical Android device**.

This map is done when nothing is left to decide before someone sits down and builds that slice.

## Notes

**Domain.** A pixel-art idle tower-climbing auto-battler in Bevy, in the lineage of [Dot Heroes Ⅱ: Nonstop RPG](https://apkcombo.com/dot-heroes-ii-nonstop-rpg/com.mrgames.summonstower/) (six authored towers, positional heroes, four skills each, gold-cost seals, PvP ranking tower) and [Thousand Floors](https://store.steampowered.com/app/4794380/Thousand_Floors/) (endless floors, idle summons, mystic-arts tree, milestone relics, rebirth). Thousand Floors sits at **42% positive**, and the diagnosis driving this project is that its failures are *bugs and illegibility* — a player prestiging without knowing what it bought, units clipping through walls — not shallow mechanics.

**Skills every session should consult.** `grilling` and `domain-modeling` by default. `prototype` for any ticket typed `prototype`. `research` for any ticket typed `research`.

**Standing preferences for this effort.**

- **Plan, don't do.** Tickets resolve decisions. The pull to start building is the signal the map is finished.
- **Legibility has veto power.** No mechanic ships without its feedback. The player must always be able to answer *what did that just do for me*. This is the project's reason to exist; it does not get cut under schedule pressure.
- **Premium, no microtransactions.** Build a game worth playing, sell it later.
- **guild-forge is a quarry, not a foundation.** Mine `/Users/tacit/dev/guild-forge/guild_forge_gdd.md` for document structure and its crate list. Do **not** inherit its simulation strategy — accelerated logic-only ticking is O(time × floors) and cannot survive an overnight Android suspension.
- **Tuning constants live in hot-reloadable RON, never in Rust.** An idle game is 90% tuning.
- **No custom `Material2d`.** It crashes on Adreno GPUs — the majority Android GPU — in both 0.18 and 0.19 (#22925, open, no fix in flight). Use a custom render pipeline where a material would be the obvious reach.
- **No `Interaction`-based UI.** All UI input goes through `bevy_picking` observers. Bevy's legacy `ui_focus_system` aggregates touches globally rather than per finger (#11553), which breaks multi-touch — and the four hero ability buttons are a multi-touch surface.
- **Vocabulary is fixed.** See `CONTEXT.md`. Retired synonyms: *seal*, *lockdown*, *level* (for floors), *summon*, *unit*.

**Settled at charting.** Premises established during the charting grill. These are not ticket resolutions — they are the ground the tickets stand on.

- Spine: Thousand Floors' infinite generated floors, with a positional hero grafted on.
- Platforms: desktop and Android. iOS and console are out.
- The hero is **stationed on a floor it repeats across**; the player chooses which hero and fires its abilities manually. Four abilities, cooldown-gated, no mana or energy resource.
- Hero placement is a **throttle**, not a wall-breaker: below the wall it farms and escorts, at the wall it pushes. The tradeoff must be made visible.
- Climbers are disposable and anonymous as individuals; the **climber type** carries identity. Three types in the slice — melee, ranged, healer — with data keyed by type from day one.
- **Rank** is the upgrade axis for climbers. Hero levels and climber ranks both reset on prestige.
- **Lock** costs gold, sets the spawn floor, is permanent within a run, is undone by prestige, and yields a guaranteed passive income rate. Every 10 floors.
- Prestige grants a **permanent multiplier** on health and damage — the single permanent power axis. It must present an explicit before/after.
- **Relics modify, prestige multiplies.** Relics grant qualitative effects (income, lock cost, cooldowns, spawn rate) and may favour specific climber types. No relic reads "+15% damage".
- Offline progress yields **gold only**, closed-form, capped at roughly 8–12 hours. The climb does not advance while away.
- Save is hand-rolled `serde` to **RON**, one autosave slot plus a rotating backup. No `bevy_save` while still learning Bevy.
- **No persisted RNG state, no seeded determinism.** See `docs/adr/0001`.
- Art runs on placeholder primitives until the loop is proven, then an asset pack, then custom.

## Decisions so far

- [Bevy 0.18 Android: build toolchain and lifecycle](issues/01-bevy-android-support.md) — `cargo-ndk` + Gradle with `GameActivity` is the only maintained path; the surface is destroyed and recreated on suspend and Bevy already handles it; `AppLifecycle::Suspended` is a real synchronous save hook but `WillSuspend`/`WillResume` are never delivered; touch is `WindowEvent::Touch` only, never mouse; `features = ["2d"]` is already Android-complete; custom `Material2d` crashes on Adreno GPUs (#22925).
- [Portrait-first UI approach in Bevy 0.18](issues/02-portrait-ui-approach.md) — built-in `bevy_ui` + `experimental_bevy_ui_widgets` driven through `bevy_picking` observers, which handle touch per-finger natively; `bevy_ui` is already compiled in via `features = ["2d"]`; `Interaction`/`InteractionPalette` is **banned** (global-aggregate touch bug #11553, the pattern guild-forge uses); layout branches on aspect ratio, never `target_os`; `UiAntiAlias::Off` + `Msaa::Off` + `FontSmoothing::None` from day one.
- [Target Bevy 0.18 or 0.19?](issues/13-bevy-version-target.md) — **0.19.1**, decided by #14710 (bevy_ui Android flicker) being fixed transitively via wgpu 29, which 0.18 pins too old to receive and can never be backported. #22925 (Adreno `Material2d` crash) is unfixed in both. Build config is `features = ["2d", "ui", "ui_picking", "android-game-activity"]` — on 0.19 the `2d` collection silently no longer implies `ui` or the Android backend, and `ui` does not imply `ui_picking` (corrected by ticket 04; without it no UI input works at all). Bevy 0.19.1 requires Rust 1.95.
- [The simulation boundary](issues/03-simulation-boundary.md) — two regions: sealed floors emit a frozen gold rate and nothing else, the active band is uncapped with floors activating lazily as climbers approach. No agreement contract exists — locking is a one-way conversion, measured and frozen. Positions are continuous inside a clamped floor-local space, making wall-escape unrepresentable. `FixedUpdate` at 10Hz. Cleared floors repopulate on a timer (which *is* the farming mechanic); failed floors reset immediately, so attrition can't beat a wall.
- [Climber lifetime and the healer](issues/05-climber-lifetime-and-healer.md) — climbers persist **until death** with **no HP regeneration**, which makes the Wall emergent rather than declared. No party; the individual is the unit of survival. The healer is a **pure-support aura**, stacking linearly, targeting built as a swappable policy. The hero has HP, dies, and respawns **on its stationed floor**, so the throttle is *uptime*. Population cap is **per climber type**, so composition is contested in gold, not slots. Deaths are legible in aggregate at the wall, not per-unit.

## Not yet specified

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- **Hero roster beyond the slice.** How many heroes exist, how they are unlocked, whether they differ by role or by ability kit. Blocked on the slice hero proving the station mechanic works at all.
- **Enemy and floor variety.** What actually lives on floor 400 that didn't live on floor 40, and whether floors have visual or mechanical themes. Hangs on the curve.
- **Setting, tone, and narrative framing.** Thousand Floors leans on the Classic of Mountains and Seas. dot-tower has no setting yet. Not urgent, but it shapes art direction.
- **Art pipeline past placeholder.** Palette, sprite dimensions, animation budget, whether to buy a pack or commission. Waits on the loop being proven fun.
- **Healer targeting policies beyond the aura.** The seam ships in the slice but the aura is the only policy, and no UI exposes it. Whether a second policy earns its place — and whether the player should set it per climber type — waits on watching the aura in play.
- **Audio.** Entirely unexamined.
- **Accessibility.** Colourblind-safe type differentiation and text legibility on a portrait phone. Likely to bite once three climber types must be told apart at a glance.
- **Balance methodology past the first curve.** How tuning gets validated — headless sim runs, telemetry, playtesting.
- **Steam release mechanics.** Store page, pricing, wishlists, build pipeline.

## Out of scope

Ruled beyond this destination. Never graduates; returns only if the destination is redrawn as a fresh effort.

- **PvP / Ranking Tower** — Dot Heroes II's competitive ladder. Beyond a single-player vertical slice, and it drags in backend infrastructure.
- **Microtransactions and any IAP economy** — ruled out by the project's premise.
- **Live-ops, seasons, battle passes, daily login economies.**
- **iOS and console ports.**
- **Multiplayer or co-op of any kind.**
- **Extracting shared idle/simulation machinery into a crate with guild-forge** — abstraction fitted to one use case. Revisit only if a third project appears.
