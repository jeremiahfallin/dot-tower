# Composition is authored

The three climber types spawn in a **fixed ratio the player never sets**. Composition is a design
statement, not a decision. The game has four decisions — climber rank, locks, which hero is
stationed, and when its abilities fire — and this is deliberately not a fifth.

The types exist for **legibility**. The tower has to read as an army fighting: melee falling at
the front, archers behind them, healers keeping them up. A homogeneous stream would make the wall
unreadable, and the wall is the thing the player is watching. The division of labour is real but
incidental — melee is the health pool, ranged the damage, healer the sustain — and it is what
makes one gold axis legible: rank buys *more army*, and the army has parts.

This was measured, not assumed. Across eleven splits of a fixed 90-climber budget, with every run
pinned to identical wall-clock time:

| | run 1, locks as shipped | run 1, locks fixed | run 6, locks fixed |
|---|---|---|---|
| best split | 149 | 156 | 1232 |
| worst split | 122 | 131 | 1224 |
| **spread** | **22.1%** | **19.1%** | **0.7%** |

**Composition matters while the player is weak and stops mattering once they are strong.** As a
decision it would be one that *expires*, which is the worst available shape in an idle game whose
player is meant to still be deciding things at depth. The 0.7% was measured specifically against
the suspicion that the late game's inertness was a lock-curve artifact; it is not, and fixing the
lock curve makes composition **more** inert rather than less.

Gold competition was the constraint this was resting on, and it does not bite. Across spending
strategies as different as buy-cheapest, buy-even, and all-in on one type for an entire run, the
spread is 140 to 148 in run 1 and 738 to 749 at depth — because rank cost at 1.30/rank rises
fast enough that income forces all three to be bought regardless of intent. Ranks land at
55/54/53 whatever the player does.

Each type does earn its place: removing one costs 12.9% (melee), 11.6% (ranged), 20.4% (healer).
The stream needs all three. It does not need the player to choose between them.

## Consequences

- **The type cap is never bought with gold.** Raising every cap is a monotonic, saturating power
  increase (×5 caps → +15.6%) that is indifferent to the split — so as a gold axis it competes
  with rank while doing rank's job. Rank remains the single gold-bought power axis, on the same
  reasoning that keeps the hero's aura off the gold economy in
  [0005](0005-the-hero-multiplies-climbers-add.md).
- **One global replacement interval plus an authored ratio**, rather than three caps and three
  intervals. Six numbers were steering three unrelated things: the ratio, the run-length dial
  ([ticket 06](../../.scratch/dot-tower/issues/06-progression-curve-first-pass.md)) and the
  survival dial ([0010](0010-combat-is-floor-local-and-has-no-reach.md)). Caps survive as a
  per-type ceiling that, under a healthy lock curve, does not bind at all.
- **The ratio is authored to read as an army, not tuned to the optimum.** Near-equal measures
  best, and an army that is one-third medics does not read like one. The mechanical difference
  expires; the fantasy does not. Same trade the aura took in
  [0005](0005-the-hero-multiplies-climbers-add.md), for the same reason.
- **The mix is never a number on screen.** The tower column is the display, and under a healthy
  lock curve the whole stream is 42 climbers early and 9 at depth — directly countable. This makes
  telling the three types apart at a glance load-bearing rather than a nicety.
- **`type cap` is retired as a relic effect kind**, and type affinity as a whole needs rethinking.
  See [0009](0009-relic-effects-avoid-axes-the-curve-erases.md).
- This is the decision most likely to be "fixed" by a future contributor who finds three climber
  types with no strategic difference between them and concludes the player should be given control
  of the mix. The control would be real; the decision behind it would not.
