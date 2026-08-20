# Combat is floor-local and has no reach

Combat happens **on a floor, between everything standing on it**. No unit attacks a floor it is
not standing on, and no unit is protected by where it stands. Incoming damage is shared out by
**threat weight** — melee 3.0, ranged 1.0, healer 0.5 — which is a damage share, not a position.

Ranged climbers fire across a gap as **animation**. The arrow is sprite flavour over a floor-local
exchange, and nothing in the simulation reads it.

This was measured against three separate mechanics, all of which are commonly meant by "reach".

**Attack reach — firing at a floor above your own — is a damage buff wearing a positional
costume.** Reach ±2 buys +2.7% peak floor and 2.22× the gold. Leaving reach at zero and simply
**doubling ranged damage** buys +2.0% and 2.01× — the same result from the honest lever. It moves
nothing about safety in either direction: ranged deaths *rise*, 238 → 264, because a climber that
shoots further pulls more floors into the fight. Widening the ranged population cap by 20%
reproduces none of it (1.01× gold), so the effect is specifically damage, not bodies. Adding it
would be adding a large flat damage modifier under another name, which is what
[0003](0003-relics-modify-prestige-multiplies.md) exists to prevent.

**Standoff — halting short of the fighting so the pack can never reach you — works, and costs.**
It delivers exactly what it promises: ranged deaths fall 172 → 114 and ranged's share of all
healing collapses from 23% to 2%. The peak floor falls with it — −4.1% in run 1, **−15.1% at
depth** — because a climber that cannot be hit is also a body that is no longer at the wall
absorbing anything, and melee inherits all of it (melee's share of healing rises to 93%). It
breaks even only if the hero's aura widens to cover the standoff distance, which is measurable
(gold 0.84× at ±0 against 1.10× at ±2) and would reopen [0005](0005-the-hero-multiplies-climbers-add.md)'s
±0 reach.

**Formation — position deciding who gets hit within a floor — has no effect that position
explains.** Melee in front scores 811, ranged in front 811, healer in front 812, and **no
formation order at all 815**. An order-blind rule and an order-sensitive rule land within 0.5% of
each other, so whatever the variant changes, it is not being caused by who is standing where.

## Consequences

- **[0005](0005-the-hero-multiplies-climbers-add.md)'s ±0 aura reach stands**, and for a second
  independent reason. It was chosen for legibility once combat concentrated everyone on the
  contested floor; the same concentration is why no other reach pays either.
- **Ticket 03's floor-local clamped positions stay a rendering concern.** Position is never read
  by combat, so no distance metric is needed and wall-escape stays unrepresentable. This is the
  gap ticket 05 raised and 0005 closed, and it stays closed.
- **A "ranged reach" relic is dead.** Relics can only modify a reach that exists, and none does.
  The ranged-affinity relic must find its form in the `type stat` or `type cap` effect kinds
  ([0009](0009-relic-effects-avoid-axes-the-curve-erases.md)), at a magnitude that moves
  composition.
- **Ranged is not currently safer than melee, and no reach mechanic makes it so.** Melee dies
  1.08× as often per live climber; with threat weighting switched off entirely, 0.97×. Standoff
  — literal invulnerability from the contested floor — moves it to 1.03×. The cause is that
  **per-type replacement sets the death rate**: a type at its cap that dies faster simply
  respawns faster, and a **10× swing in climber health compresses to a 1.3× swing in survival**.
  Type identity cannot be expressed through survivability while that machinery stands. This is a
  live problem, owned by the composition ticket, not by combat.
- This is the decision most likely to be "fixed" by a future contributor who notices that ranged
  climbers stand in the melee and adds reach to fix it. All three forms have been tried. The one
  that changes nothing is a damage buff, the one that works costs 15% of the peak floor, and the
  third is indistinguishable from randomising it.
