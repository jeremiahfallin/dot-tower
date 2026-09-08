# 0013. Locks are priced in time, and free in conquered territory

Date: 2026-09-08
Status: accepted
Resolves: [ticket 20](../../.scratch/dot-tower/issues/20-travel-time-and-the-lock-curve.md)

## Context

A lock costs gold and raises the spawn point by 10 floors. Ticket 06 priced it
geometrically — `lockCost0 · lockCostBase^(k−1)` — and ticket 15 discovered the
consequence at depth: with `lockCostBase` 4.0 against per-kill gold growing
`goldBase^10 = 2.478` per lock interval, lock cost outruns income by ×1.61 per
lock, compounding. The lock line falls permanently behind the wall, travel time
grows without bound, and the stream strings out over hundreds of floors while
the units that reach the front are a trickle too thin for anything to matter.

The obvious fix — drop the base until it matches income — does not exist. Income
compounds **faster than per-kill gold**, because kill rate grows with ranks, so
no geometric base holds a constant relationship to income. Measured
(ticket 20): base 2.478 under-matches (locks fall to ~0% of income and the
stream collapses onto the wall); base 3.0 over-matches (travel diverges); the
apparent match at 2.7 was a coincidence of the current rank-ladder constants and
moved when sealed income was withdrawn. Any geometric match is tuned-then-rot.

Relatedly: the model had been paying gold from sealed floors all along, left in
because it measured 0.0% under the shipped curve. Under *any* fixed curve it is
~99% of income — 119 seals each paying their frozen 60-second snapshot forever.
Ticket 06's withdrawal ("frozen floors emit nothing", per `CONTEXT.md`'s Lock)
turns out to have been never implemented, and every "fixed lock curve" reading
before ticket 20 was riding the artifact.

## Decision

1. **A lock costs time, not a gold magnitude.** Its price is K seconds of
   recent income (rolling 60s, the same window offline gold's best rate uses).
   Matched to income by construction: it cannot diverge when the rank ladder is
   retuned and cannot fall to zero as depth compounds. K is tuning data in RON.
2. **Locks below the account's deepest-ever floor are free.** Head start is
   conquered territory; re-locking it is bookkeeping, not a decision. Without
   this rule a mature account's blitz through its head start outruns any
   time-priced lock line (K = 30s caps it at 20 floors/min) and the road
   balloons to 260+ floors mid-run. The gold price — and the decision — live at
   the frontier. The save already holds deepest-floor-ever in `Account`.
3. **Sealed floors emit nothing, enforced.** Ticket 06's withdrawal is now
   implemented. Locking is a spawn-point and travel-time mechanic, not an
   economic pillar.

## Consequences

- Locking is load-bearing on the stream's *shape*, not on depth: never locking
  costs ~8–9% of peak floor but explodes the road (603 floors at run 6) and
  forces the caps to bind. The purchase a player evaluates by eye — the road
  shortens — with forgiving stakes. That is the decision content of locking;
  there is no interesting lock-vs-rank arithmetic to tune it for.
- The walk stays inside the tower column's window (25 floors in a fresh account,
  12–17 at depth; ticket 10 sized that window at 12–14). Crowd size is dialed
  by replacement and climb speed, not by the lock curve.
- **Runs stop ending.** Under the shipped curve, runs ended because locks
  became unaffordable — the lock curve *was* the run-length curve. Priced in
  time, runs 7–10 of a campaign are still climbing at a three-hour cap and the
  natural stall recedes past the model's f64 horizon. What ends a mature run is
  now an open question, owned by
  [ticket 17](../../.scratch/dot-tower/issues/17-run-length-growth.md).
- The prestige curve survives unchanged: head start holds 16% of peak in run 1
  and 24–28% from run 2 through run 10, and floor 1,000 arrives at run 7 rather
  than ticket 06's promised ~10.
