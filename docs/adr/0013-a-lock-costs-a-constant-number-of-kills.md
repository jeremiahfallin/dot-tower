# A lock costs a constant number of kills

`lock_cost_base` is **not an independent constant**. It is `gold_base ^ 10`, so the price of a
lock, expressed in kills at the depth it seals, is the same for the first lock and the fiftieth.
The tuning file states a number because it must; the number is a consequence.

The quantity that matters is the ratio between them — `lock_outrun` in the code — and it wants to
be exactly 1.00. Both sides of that are failures, and they are different failures:

| `k` | peak | crowd | transit | back | gold wait | bought on sight | ceiling binds at depth |
|---|---|---|---|---|---|---|---|
| 0.81 | 642 | 6 | 10 | 2 | 1.2s | **90%** | no |
| 0.91 | 641 | 6 | 10 | 2 | 2.4s | **74%** | no |
| **1.00** | **631** | **7** | **12** | **2** | **41s** | **0%** | **no** |
| 1.11 | 607 | 83 | 64 | 28 | 64s | 0% | **yes** |
| 1.21 | 592 | 86 | 102 | 43 | 65s | 0% | **yes** |
| 1.61 | 565 | 88 | 180 | 74 | 97s | 0% | **yes** |
| 1.82 | 563 | 87 | 203 | 90 | 93s | 0% | **yes** |

*Run 6 of six 60-minute runs, wall-clock pinned, `lock_cost0` 500. `transit` is floors from the
lock line to the frontier; `back` is how far behind the frontier the average climber sits.*

**Above 1.00 the lock line falls permanently behind the wall.** Cost outruns income and compounds,
so locks become geometrically unaffordable, travel time grows without bound, and the crowd pins
against the per-type ceiling — which is [ADR 0011](0011-composition-is-authored.md)'s central
prohibition. The transition is sharp rather than gradual: between k = 1.00 and k = 1.11 the crowd
goes 7 → 83 and the average climber's distance behind the frontier goes 2 floors → 28.

**Below about 0.90 locking stops being a purchase.** At k = 0.81 nine locks in ten are bought
within one decision cycle of the climb earning the right to them: gold is never the constraint, and
the lock degenerates into something that happens to the player rather than something they buy.

**At exactly 1.00 it is a purchase and the line keeps up.** This is the result that was not
predictable from the arithmetic, and it answers the objection that motivated the whole question: a
lock cost that merely matches income does *not* become automatic, because gold is contested with
climber ranks. The measured wait is 41 seconds and no lock arrives already affordable.

## Consequences

- **`lock_cost0` is the crowd dial, and it is nearly free.** Across a 32× sweep at k = 1.00 the
  crowd goes 6 → 59 and the transit 10 → 29 floors, while peak floor moves 643 → 616. How big the
  crowd is and how far it walks are the same number — population is spawn rate times lifetime, and
  lifetime is mostly walking — so this one constant sets both. It is authoring, and it is the
  number to move if the tower does not read as an army.
- **`lock_margin` must not be used as a second crowd dial**, though it looks like one. Raising it
  to 40 grows the crowd to 58, but by gating the lock on depth instead of on gold: 91% of locks
  then arrive already affordable, and the purchase decision disappears exactly as it does at
  k < 0.9. Price the lock; do not delay it.
- **[ADR 0011](0011-composition-is-authored.md) is no longer conditional.** Ticket 14 found that
  its claim — the per-type ceiling does not bind — held only under a lock curve nobody had landed,
  and that under the shipped one the ceiling *was* the crowd dial. At k = 1.00 the ceiling stops
  binding from the second run onward and the live mix settles on the authored ratio. It still
  grazes during run 1, when the account has no prestige and the climb is slowest.
- **Travel time is a legibility parameter, not an economic one.** Across a 20× swing in transit —
  10 floors to 205 — peak floor moves 17%, while the share of climber-ticks spent walking never
  drops below 78%. The stream is overwhelmingly walking in every configuration, so travel is not
  something the lock curve introduces; it is what a stream that spawns at a line and fights at a
  frontier is. What the lock curve chooses is how much of it the player watches.
- **Retuning `gold_base` requires moving `lock_cost_base` with it.** This is the way the decision
  is most likely to be undone by accident, because the two constants sit in different sections of
  the tuning file and only their ratio is meaningful. `Tuning::warnings()` says so at load.
- **Entity demand fell.** The landed curve peaks at 142 entities against 178 before it, which puts
  the game further below the 200–450 band
  [ticket 14](../../.scratch/dot-tower/issues/14-android-performance-budget.md) sized its device
  session around.
