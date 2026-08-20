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
- The hero is a **throttle**, not a wall-breaker — but *placement* is not the throttle. *Amended by ticket 16: "below the wall it farms, at the wall it pushes" is retired; stationing at the wall dominates on gold and experience alike, because depth beats throughput. Placement is a learnable rule — put the hero where the fighting is. The throttle is **uptime**, and the decisions are which hero and when its abilities fire.*
- Climbers are disposable and anonymous as individuals; the **climber type** carries identity. Three types in the slice — melee, ranged, healer — with data keyed by type from day one.
- **Rank** is the upgrade axis for climbers. Hero levels and climber ranks both reset on prestige.
- **Lock** costs gold, sets the spawn floor, is permanent within a run, and is undone by prestige. Every 10 floors. *Amended by ticket 06: it yields no meaningful passive income — locking is a spawn-point and travel-time mechanic, not an economic pillar.*
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
- [Progression curve, first pass](issues/06-progression-curve-first-pass.md) — the first-pass constants, in RON. Enemy health and damage ×1.075/floor against gold ×1.095, which makes deeper floors strictly more gold-efficient and is the engine pulling the player upward; past ~1.12 gold the economy runs away. Healer targeting switches from the aura to **single-target, lowest percentage** — the aura fully out-healed the climb, so climbers reached the wall at 100% and the Wall was declared rather than emergent. **Prestige compounds** (a best-ever-floor multiplier dead-ends at floor ~220), and must be shown as *floors of head start*, never as ×10¹¹. Replacement rate, not the respawn timer, is the run-length dial. f64 holds to ~floor 2,000. Runs land in a 30–60 minute band; floor 1,000 arrives around run 10.
- [What the hero contributes that climbers cannot](issues/16-hero-contribution.md) — **the hero multiplies what climbers do, never adds to it** ([ADR 0005](../../docs/adr/0005-the-hero-multiplies-climbers-add.md)). One universal **aura** on its own floor multiplying climber damage and gold; a ×2 aura buys ~+21 floors where every additive hero tried bought +2, and the gain is sub-linear, so it shifts a wall and can never break one. **Taunt is per-hero**, paying for itself only on a tanky one — which is what makes the roster differ in simulation and delivers the escort feeling. The hero levels on **experience from kills in its aura**, never gold, so the two economies never touch. Aura multiplier is relic-only. Ticket 05's uptime claim was backwards: the wall is *safer* without taunt, because the crowd is armour. The slice ships the tank.
- [How the player reads the hero](issues/09-hero-placement-legibility.md) — **the tower says where and now; the strip says how it has been going.** Everything spatial is diegetic — the aura is a lit floor with its multiplier called out, the hero carries a draining health ring and a respawn countdown, experience fills it from the bottom, the wall is hatched. Everything temporal is one strip above the ability bar: a 45s holding sparkline and a record of ability presses with a plain-language verdict on the last. Nothing duplicated between them. Surfaced: the aura row *is* the wall row in the common case, so they need one combined treatment rather than two decorations. Ticket 16's wrong-moment test is satisfiable in a single line of copy.

## Not yet specified

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- **Hero roster beyond the slice.** Ticket 16 settled the *shape*: heroes share one universal aura and differ by abilities, health, and whether they taunt — with tank, fighter, mage and cleric named as the intended roles and the tank shipping in the slice. What remains is how many exist, how they are unlocked, and what each one's four abilities actually do. Waits on the tank proving the aura-and-uptime loop in play.
- **Enemy and floor variety.** What actually lives on floor 400 that didn't live on floor 40, and whether floors have visual or mechanical themes. Hangs on the curve.
- **Setting, tone, and narrative framing.** Thousand Floors leans on the Classic of Mountains and Seas. dot-tower has no setting yet. Not urgent, but it shapes art direction.
- **Art pipeline past placeholder.** Palette, sprite dimensions, animation budget, whether to buy a pack or commission. Waits on the loop being proven fun.
- **Exposing healer targeting to the player.** Ticket 06 built the policy seam and settled single-target lowest-percentage as the shipped default, so what remains is whether the player ever chooses — a standing order set per climber type — and whether a second policy earns its place. Waits on watching the shipped one in play.
- **Audio.** Entirely unexamined.
- **Accessibility.** Colourblind-safe type differentiation and text legibility on a portrait phone. Likely to bite once three climber types must be told apart at a glance.
- **Balance methodology past the first curve.** How tuning gets validated — headless sim runs, telemetry, playtesting. Ticket 06's model is the obvious seed; whether it becomes a kept tool or stays throwaway is unresolved.
- **Where offline return gets its substance.** Ticket 06 removed sealed income as a source, so ticket 07 has no premise. Whether offline return is about gold at all, or about something else entirely, is now open.
- **Steam release mechanics.** Store page, pricing, wishlists, build pipeline.

## Out of scope

Ruled beyond this destination. Never graduates; returns only if the destination is redrawn as a fresh effort.

- **PvP / Ranking Tower** — Dot Heroes II's competitive ladder. Beyond a single-player vertical slice, and it drags in backend infrastructure.
- **Microtransactions and any IAP economy** — ruled out by the project's premise.
- **Live-ops, seasons, battle passes, daily login economies.**
- **iOS and console ports.**
- **Multiplayer or co-op of any kind.**
- **Extracting shared idle/simulation machinery into a crate with guild-forge** — abstraction fitted to one use case. Revisit only if a third project appears.
