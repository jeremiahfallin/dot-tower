# Ranged attack reach

Type: prototype
Status: resolved

## Question

Does combat have positional reach, and does giving ranged climbers one change anything?

Today it does not. In ticket 06's model everyone on the contested floor fights, and incoming
damage is split by **threat weight** — ranged carries `threat: 1.0` against melee's `3.0`. Ranged
safety is therefore *statistical*, not positional: a ranged climber is less likely to be hit, but
it is standing in the same place as everything else. Ticket 03 put positions in a continuous,
clamped, **floor-local** space, and ticket 16 set the hero's aura reach to ±0 floors precisely
because combat concentrates everyone on the contested floor.

Raised by ticket 11. Relics can only modify a reach that already exists, so the ranged-affinity
relic has no legible form until this is settled — with it, the obvious one is **extend it**.

Resolve:

1. **Does ranged get real reach** — firing before enemies arrive, and firing while melee absorbs —
   or is threat-weighting the intended abstraction with reach as sprite flavour?
2. **What does reach cost the model?** Floor-local positions are already continuous, so
   within-floor reach may be nearly free; reach that *crosses* a floor reopens the distance-metric
   gap ticket 05 raised and ticket 16 closed by choosing ±0.
3. **Does it change the outcome?** Measure it in ticket 06's model the way ticket 16 measured aura
   reach — which found ±0 through ±5 identical, and that null result is the live hypothesis here
   too.
4. **What does it do to composition?** If ranged becomes positionally safe rather than
   statistically safe, the melee/ranged balance moves and **ticket 15 needs the answer**.
5. **Is it legible?** A climber that is safe because of where it stands can be seen; a climber
   that is safe because of a threat weight cannot.

## Answer

**No. Combat is floor-local and has no reach; threat weighting is the intended abstraction and
the arrow is sprite flavour.** Recorded as
[ADR 0010](../../../docs/adr/0010-combat-is-floor-local-and-has-no-reach.md), because a future
contributor will look at ranged climbers standing in the melee and "fix" it.

Three separate mechanics get called "reach", and they were measured separately. The first two
passes were contaminated — variants that *grind* rather than stall run four times as long and
reach a deeper floor for that reason alone, which made front-to-back targeting look like +29
floors and 14× gold. Every number below comes from the third pass, which disables the stall
detector and runs every variant for identical wall-clock time. That correction reversed the
headline: normalised, the same variant is **worse**.

### 1. Attack reach is a damage buff in a positional costume

Firing at a floor above your own while still walking to the wall. Reach ±2 buys **+2.7% peak and
2.22× gold**; leaving reach at zero and **doubling ranged damage** buys +2.0% and 2.01×. The
honest lever reproduces it. It is non-monotonic — ±5 is worse than ±2, because fire thins across
floors — and it moves safety in the *wrong* direction: ranged deaths rise 238 → 264. Widening the
ranged cap 20% reproduces none of it (1.01× gold), so it is damage, not bodies.

Q1's second half is therefore answered by the first: reach is not an alternative to threat
weighting, it is a flat damage modifier under another name, and ADR 0003 exists to stop those.

### 2. Standoff works, and costs

Halting as soon as the fighting comes into reach — real positional safety. It does what it
promises: ranged deaths 172 → 114, ranged's share of all healing **23% → 2%**. Peak floor falls
−4.1% in run 1 and **−15.1% at depth**, because a climber that cannot be hit is also a body no
longer absorbing anything at the wall; melee inherits it and its healing share rises to 93%.

**It breaks even only by reopening ticket 16.** Standing off walks ranged out of the hero's aura,
which ticket 16 fixed at ±0 — verified directly: gold 0.84× of baseline at ±0, 1.10× once the
aura widens to ±2. So positional safety for ranged is not a ranged decision; it is a hero
decision, and it costs the ±0 that ticket 16 chose for legibility.

### 3. Formation ordering is indistinguishable from randomising it

Incoming falling front-to-back within the floor, no cross-floor reach. Melee in front **811**,
ranged in front **811**, healer in front **812**, no formation order at all **815**. An
order-blind rule and an order-sensitive rule land within 0.5%. Q3's null hypothesis — carried
over from ticket 16's ±0-through-±5 aura result — reproduces, and harder: it is not merely that
reach is neutral, it is that the variants which *aren't* neutral are neutral-to-negative.

### 4. What it would have cost the model (Q2)

Not free, even at ±1. Three seams appear: an attacker→target assignment, so a climber within
reach of two contested floors does not fire at both and turn range into output; per-climber aura
evaluation rather than per-floor, which is what exposes the ticket-16 coupling above; and a
definition of *contested* that answers whether a floor being shot at from a distance counts —
forced by ticket 03's floor-reset rule, which reach is the mechanic most exposed to. All three
are avoided entirely by keeping combat floor-local, and ticket 05's distance-metric gap stays
closed rather than being reopened.

### 5. The premise is false, and this is the real finding (Q4, Q5)

Q5 asks whether ranged's safety can be *seen*. It cannot, but not for the reason the ticket
assumes — **it is not there to see**.

| | melee | ranged | healer | melee vs ranged |
|---|---|---|---|---|
| as shipped (threat 3.0 / 1.0 / 0.5) | 9.6 | 8.9 | 7.6 | **1.08×** |
| ranged threat cut 4× to 0.25 | 10.1 | 9.5 | 8.3 | 1.07× |
| threat weighting **off entirely** | 9.4 | 9.7 | 7.9 | **0.97×** |
| standoff 2 — literal invulnerability | 9.7 | 9.4 | 6.6 | 1.03× |

*(deaths per live climber, 40-minute run)*

Melee dies 1.08× as often as ranged. Deleting threat weighting altogether changes that to 0.97×.
The system the ticket set out to make legible **is already inert**.

The cause is that **per-type replacement sets the death rate**. A type sitting at its cap that
dies faster simply respawns faster, so deaths-per-capita converges on the replacement interval
regardless of what combat did. It is heavily damping rather than perfectly flat, and the
compression is severe: a **10× swing in ranged health produces a 1.3× swing in survival**
(hp 30 → 300 moves 8.9 → 6.7; hp 30 → 3 moves 8.9 → 18.0).

So Q4 inverts. Reach does not enrich composition — standoff *removes* a composition decision, by
taking ranged out of the sustain economy entirely (2% of healing) and making it fire-and-forget.
And the deeper problem is that **no climber type has a survival identity to move**, because the
cap-and-replacement machinery ticket 06 identified as the run-length dial is also the survival
dial, and it overwrites the combat model's output. Appended to ticket 15, which owns the caps and
the replacement interval.

### Surfaced in passing

**The model has never implemented ticket 03's rule that a floor nobody is fighting restores its
pack**, so attrition can beat a wall in it — a floor chipped to 10% stays there while the
climbers that chipped it die and respawn. Reach is the mechanic most exposed to this, which is
how it came to light. Measured impact on the baseline is **147 → 144, inside the noise**, so
tickets 06–16 are not invalidated; but it is a settled decision the measurement tool silently
contradicts, and it is now a knob (`failedFloorReset`) rather than an absence.

Prototype: `.scratch/dot-tower/prototypes/19-ranged-reach/`. Ticket 06's `model.mjs` extended
with reach, standoff, formation targeting and per-type instrumentation, all defaulting off —
ticket 08's run-1 peak of 147 still reproduces exactly.

## Amended by ticket 15's mechanism landing (session of 2026-09-07)

`model.mjs` now runs one global replacement interval against an authored ratio
(ADR 0011) instead of three per-type spawn intervals. **This ticket's headline
lever no longer exists**: "per-type replacement sets the death rate" was measured
through `types.ranged.spawnInterval`, and ADR 0011 abolished per-type supply.

The question survives the change and was re-asked through the only per-type
supply knob left, the **ratio**. `safety.mjs` is updated accordingly, and the
finding holds — more strongly than before:

- **As shipped, melee now dies 1.00× as often as ranged** (was 1.08×). Ranged is
  not safer, which was this ticket's central claim.
- **Supply still sets the death rate.** Ratio `ranged 3 → 9` drives deaths per
  live climber from 8.3 to 11.5 *for every type*; a 100× ranged health swing
  (300 vs 3) moves it 7.0 to 11.6. Survival is a property of the stream, not of
  the climber type.

**Depth results are intact**, which is where this ticket's conclusions live: at
run 6, baseline peak 827 → 821, reach ±2 828 against a baseline of 821 (still
inert), standoff 2 680 → 685 (still ~17% down).
[ADR 0010](../../docs/adr/0010-combat-is-floor-local-and-has-no-reach.md) stands.

Run-1 figures moved around considerably. That is expected rather than alarming:
this ticket already established that run-1 comparisons are untrustworthy unless
pinned to identical wall-clock time, and it is the reason the resolution rests on
the depth numbers.
