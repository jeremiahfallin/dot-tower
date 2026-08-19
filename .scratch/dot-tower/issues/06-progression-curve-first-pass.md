# Progression curve, first pass

Type: prototype
Status: resolved
Blocked by: 03, 05

## Question

What are the actual numbers, and do they produce the intended feel?

Build a cheap standalone model — a spreadsheet, or a small headless Rust/Python sim, *not* game code — that lets us react to the curve concretely rather than argue about it abstractly.

It should express:

1. **Enemy scaling per floor** — the exponential base.
2. **Climber power per rank**, and **upgrade cost per rank** on a steeper base.
3. **Gold income** from kills, and the guaranteed rate from locked floors.
4. **Lock cost** as a function of floor.
5. **The prestige multiplier** — what it is a function of (peak floor reached?) and how fast it grows.
6. **The hero's contribution**, stationed at a given floor.

Then answer with it:

- How long is a first run before the first prestige feels earned? Minutes or hours?
- Where do walls naturally fall, and are they spaced to feel like rhythm rather than grind?
- Does the second run feel meaningfully faster than the first? By how much?
- Does the curve still work at floor 1,000, or does it break down?

Output the constants in the RON shape they will ship in, so the prototype and the game agree. Link the artifact from this ticket.

## Added by ticket 03

The model gained a knob: **the floor respawn timer**. Cleared floors repopulate fully after it fires, which makes it the gold rate for the entire active region below the wall — and therefore the thing that determines whether stationing the hero below the wall to farm is worth doing at all. Model it explicitly alongside the other constants.

Note the interaction the curve has to balance: a short timer makes farming below the wall strong and pushes players to sit still; a long one pushes them to keep climbing. That tension is the hero-placement tradeoff in ticket 09, expressed as a number.

Also settled: sealed-floor income is **measured at lock time and frozen**, scaled thereafter only by the prestige multiplier. So sealed income does not benefit from later climber upgrades — model it as decaying in relative value.

## Prototype

Built 2026-08-19. Throwaway, per `prototype`.

- **Drive it**: https://claude.ai/code/artifact/ff9d3f90-ee35-4a7f-8f20-c49b373ae320
- **Source**: `.scratch/dot-tower/prototypes/06-progression-curve/` — `model.mjs` is the pure,
  liftable model; `template.html` is the shell; `build.sh` inlines one into the other to produce
  the single-file `progression-curve.html`. Open that file directly, or run `./build.sh` after
  editing the model.
- **Captured on**: branch `prototype/progression-curve` (local, not pushed).

Not a spreadsheet in the end. It is a discrete simulation at 10Hz — the same `FixedUpdate` rate
ticket 03 settled — carrying every climber as an individual with its own HP, so the wall emerges
from attrition exactly the way ticket 05 says it should, rather than being asserted by a formula.
Enemies, gold, rank costs and lock costs stay closed-form. A greedy buy-the-cheapest-thing player
policy stands in for the human.

Six guided walkthroughs: the first run, does run two feel faster, the hero throttle, the respawn
timer, locking into dead floors, and floor 1,000. Every constant is a slider, and the current
slider positions render as the RON the game will load.

### Findings carried into the discussion

Two of these contradict things the map currently treats as settled, so they need a decision
rather than a tuning pass:

1. **Sealed income is measured after the money has left.** You only lock a floor once you have
   safely climbed past it, so the band below the line has been empty for minutes and its measured
   60-second output is zero. Four of eleven locks in a default run froze an income of exactly
   zero. Ticket 03's *measure and freeze* rule is sound; the measurement window is what fails.
2. **The hero is a rounding error.** At the wall, below the wall, or not stationed at all, the
   peak floor moves by under 2% (208 / 205 / 205). Seventy climbers out-damage one hero roughly
   six to one, and the hero holds one floor. Hero placement cannot be the throttle the map claims
   while the hero's contribution is damage.

And four that are tuning, not contradiction:

3. **The respawn timer is a traffic knob, not only an income knob.** Repopulated floors sit
   between the lock line and the wall, so a short timer makes every fresh climber re-fight its way
   up and arrive already damaged. It is the single constant that most directly sets wall height.
4. **Healer rate is the most sensitive constant in the model.** Reading the aura as a shared pool
   rather than ticket 05's per-target rate changed deaths tenfold and moved the peak by a third.
5. **A prestige multiplier that does not compound dead-ends.** Deriving it from best-ever floor
   reaches a fixed point near floor 220 and stops forever. Compounding works, and costs a
   multiplier that reads ×10^11 by floor 1,000 — which is the legibility problem this project
   exists to avoid. Expressing it as *floors of head start* (log of the multiplier over the enemy
   scaling base) keeps it readable.
6. **The opening is a blur.** Floors 1–90 and nine locks are gone in under five minutes.

### The four questions, as the model currently answers them

- **First run**: ~33 minutes to floor 208, eleven locks. Minutes, not hours.
- **Walls**: nothing until floor ~90, then a smooth geometric stretch — 0.7, 0.9, 1.2, 1.5, 2.0,
  2.5, 3.0, 3.5, 9.5 minutes per 10-floor band. Rhythm, not grind, up to the last band.
- **Second run**: re-reaches run one's ceiling in ~15 minutes against 33 — about 2.2×. The ratio
  shrinks every run thereafter (run 10 needs ~32 minutes to match run 9).
- **Floor 1,000**: reached around run 10, roughly twelve hours of cumulative play, curve shape
  unchanged. Doubles stay honest to about floor 2,000, beyond which gold overflows and a
  big-number representation is needed.

Awaiting reaction. The numbers above are one defensible tuning, not the decision.

## Answer

### The constants

```ron
// assets/tuning/progression.ron
(
    tower: (
        enemy_hp_floor_1: 36.0,      enemy_hp_base: 1.075,
        enemy_dps_floor_1: 2.2,      enemy_dps_base: 1.075,
        pack_size: 4,
        gold_per_kill_floor_1: 3.0,  gold_base: 1.095,
        respawn_timer_secs: 20.0,
    ),
    climbers: (
        rank_power_base: 1.15,       rank_cost_base: 1.30,
        climb_speed_floors_per_sec: 0.5,
        spawn_interval_secs: 5.0,    // per type, only while that type is under its cap
        aura_reach_floors: 1,
        heal_targeting: "lowest_fraction",
        types: {
            "melee":  (hp: 70.0, dps: 5.5, heal: 0.0, cap: 40, rank_cost_1: 25.0, threat: 3.0),
            "ranged": (hp: 30.0, dps: 9.0, heal: 0.0, cap: 30, rank_cost_1: 35.0, threat: 1.0),
            "healer": (hp: 36.0, dps: 0.0, heal: 3.5, cap: 20, rank_cost_1: 45.0, threat: 0.5),
        },
    ),
    hero: (
        hp_level_1: 420.0, dps_level_1: 38.0, power_base: 1.18,
        level_cost_1: 120.0, level_cost_base: 1.30,
        respawn_secs: 20.0, threat: 5.0,
    ),
    locking: ( cost_first: 500.0, cost_base: 4.0, floors_per_lock: 10 ),
    prestige: ( divisor: 50.0, exponent: 1.2, compounds: true, scales_healing: false ),
)
```

`gold_base` sitting above `enemy_hp_base` is the load-bearing relationship: gold per unit
of enemy health rises with depth (0.25 at floor 1, 1.55 at floor 100, 9.80 at floor 200),
and that gradient is what pulls the player upward. Pushing `gold_base` past ~1.12 against
a `rank_cost_base` of 1.30 makes the economy run away — floor 6,000 inside four hours.

### The four questions

- **First run**: the curve stops yielding somewhere in a **30–60 minute** band, ending
  around floor 130. Not tighter than that: run length here is measured by a heuristic
  (peak floor flat for six minutes), it is non-monotonic across adjacent settings, and
  anything inside ±40% is noise. Run length is not really a constant — a run ends when the
  player stops wanting to push. What these constants fix is when the curve stops *paying*.
- **Walls**: nothing until floor ~90, then a stretching sequence of 10-floor bands. Under
  the aura it was a clean geometric ramp; under single-target targeting it is jagged — a
  wall at 90, two nearly free bands, then a climb. Judged acceptable.
- **Second run**: re-reaches the previous ceiling in 10–16 minutes, roughly 3×. The ratio
  shrinks every run after.
- **Floor 1,000**: around run 10, ~12 hours cumulative, curve shape unchanged. **f64 is the
  representation** — honest to about floor 2,000, past which gold overflows. Revisit then,
  not now.

### The decisions

1. **Healer targeting is single-target, lowest percentage of health**, reach **±1 floor**.
   This replaces the aura settled in ticket 05. Single-target is roughly a five-fold cut in
   sustain, and that cut is the point: under the aura climbers reached the wall at 99–100%
   health, so there was no attrition and the Wall was a flat power check rather than the
   emergent thing ticket 05 describes. At 83% arrival it is emergent again.
2. **Floor respawn timer 20s.** Chosen on feel; at a 5s replacement interval the respawn
   timer barely moves run length (20s→41 min, 45s→36, 60s→62).
3. **Replacement climbers every 5s per type, only while that type is under its cap.** This
   is the run-length dial, not the respawn timer — moving it from 2–3s to 5s cut the first
   run from 87 minutes to 48, where dropping the respawn timer from 45s to 20s cost only 12.
4. **Floor-1 enemy health 36, not 12.** The opening was *transit*-bound: 90 floors at 0.5
   floors/s is three minutes of pure walking out of the 4.5 the opening took. Slowing the
   walk would have fixed the clock and made the whole game sluggish; making early floors
   fight back gives the opening content. Floor 90 now takes 11.2 minutes.
5. **Prestige compounds.** Forced, not chosen: deriving the multiplier from best-ever floor
   reaches a fixed point near floor 220 and the game stops forever, because floor buys
   multiplier buys floor and the loop closes. The cost is a multiplier reading ×10¹¹ by
   floor 1,000 — present it to the player as *floors of head start*
   (`ln(M) / ln(enemy_hp_base)`), never as a raw number.
6. **Prestige does not scale healing**, per `CONTEXT.md` — health and damage only. Under
   single-target targeting this is nearly free: campaigns end at floor 563 against 568.
7. **Sealed floors produce no meaningful income, and that is accepted.** Locking is a
   spawn-point and travel-time mechanic, not an economic pillar. It still earns its cost:
   peak floor 176 with locks against 155 without.

### What this invalidates elsewhere

- **Ticket 03** — sealed income and the measure-and-freeze conversion. Addendum filed there.
- **Ticket 05** — the aura is no longer the policy; its stated reason for rejecting
  lowest-absolute-health is backwards; and keeping the hero in the candidate set is not free.
  Addendum filed there.
- **Ticket 07** — offline gold now has no source. Addendum filed there.
- **Ticket 09** — its premise does not hold; there is no placement tradeoff to make visible
  until the hero contributes something climbers cannot. Now blocked by ticket 16.
- **Tickets 08, 11, 15** — unblocked, with addenda where this resolution changed their ground.

Prototype and full findings: see the Prototype section above.
