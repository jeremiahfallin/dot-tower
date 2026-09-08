# Ticket 14 — the demand side of the performance budget

`node entities.mjs` (readings saved in `readings.txt`)

## What this is

Ticket 14 asks where frame time degrades on a low-end Android device. That is
two questions wearing one coat:

- **Demand** — how many entities and how much activation churn the game
  actually produces. A property of the simulation, not of the phone.
- **Supply** — what the device can absorb. Needs the device.

Ticket 06's model already knows the demand side; it tracks every climber's floor
and a map of floor records. It had never been asked. This adds the entity
accounting to `model.mjs` (additively — new fields on the existing sample
objects, no behaviour change, in the same way tickets 15, 16 and 19 extended it)
and reports it here.

Measuring demand first is not a consolation prize for lacking hardware. It tells
the hardware session which entity counts to test instead of guessing 1,000 /
5,000 / 20,000, and it can falsify ticket 03's model on its own — if predicted
demand were already enormous, no device measurement would rescue it.

## What it found

- **Max 205 entities** across 6,533 samples of a 14-run campaign reaching floor
  1,647 — **18.6% of the 1,100 the model predicts as its worst case.**
- **Entity count is decoupled from band height, decisively.** Driving prestige
  from ×1 to ×10¹² grows the band 6× (67 → 399 floors) while entities go
  *down*, 164 → 153. This is ticket 03's load-bearing claim and it holds.
- **Climbers do spread thin** — up to 581 floors — which is exactly what ticket
  14 Q2 feared. It costs nothing, and the reason is worth keeping: an active
  floor holds ~1.4 enemies against a pack size of 4, because trailing climbers
  walk through ground the leaders already cleared. **Spread buys activation
  events, not entities**, and churn peaks at 5.2/s.
- **Contested floors: 2–8**, against ticket 03's "at most ~90" bound. It
  *decreases* with depth as the column stretches out.
- **Inert floor data is a non-issue**: 16 B per floor puts ticket 14's
  hypothetical 5,000-floor band at 78 KiB. The deepest band actually reached was
  399 floors — the 5,000-floor case does not occur, because band height is
  bounded by how far one run climbs.

## The finding that is not about performance

Population sits at **exactly 90 in every run** — the sum of the three per-type
caps. Raising the ceiling shows where it stops binding: 180 → 270 entities,
360 → 366, and then it **saturates**, with 720 and 1440 producing byte-identical
runs because population is spawn rate times lifetime and the ceiling no longer
touches it.

So `CONTEXT.md`'s claim that the type cap "does not bind at all — the stream sits
well below it" is **false at the current value of 90** and true from roughly 360
up. Two caveats before anyone edits the glossary on this:

1. The model still runs the **superseded** mechanism — three caps and three
   spawn intervals. Ticket 15 replaced that with one global replacement interval
   against an authored ratio. The numbers here are of the old machine; the
   *shape* of the question survives the change, the values do not.
2. How big the crowd should be is **ticket 20's**, not this ticket's.

The number ticket 20 will want: left to spawn interval and lifetime with no
ceiling, the crowd settles near **405 climbers** and the whole simulation costs
**~436 entities** — still under 40% of the 1,100 prediction with nothing
capping it.

## What this cannot answer

Needs the device, and stays on the ticket:

- **Q1's cost half.** Where frame time actually degrades. Demand says the
  interesting range is 200–450 entities, not 20,000.
- **Q3's cost half.** What one activation costs. Demand says the rate is 1–5/s.
- **Q4.** Thermal throttling and fixed-timestep catch-up over a sustained session.
- **Q6.** Battery draw over 30 minutes.

Q2 and Q5 are answered here and do not need hardware.
