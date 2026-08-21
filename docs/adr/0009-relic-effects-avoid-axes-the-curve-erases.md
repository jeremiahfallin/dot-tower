# Relic effects go on axes the curve does not erase

Extends [0003](0003-relics-modify-prestige-multiplies.md).

A relic may grant a flat `+N%` **only on an axis that is flat in depth** — ability cooldown,
replacement interval, type cap, aura reach and persistence, climb speed, hero respawn. On any axis
the progression curve already drives exponentially — gold, lock cost, rank cost — a relic must take
**conditional** form instead: *"gold from kills inside the aura is doubled"*, never *"+20% gold"*.

This is arithmetic, not taste. Gold rises 9.5% per floor, so a `+15%` income relic is worth **1.54
floors** — roughly three seconds of climbing. `−20%` lock cost buys 1.6 floors of lock line;
`−20%` rank cost buys 0.85 of one rank. The danger was never that these break the economy: forty
stacked income relics move the effective `gold_base` from 1.0950 to 1.0970, against a runaway
threshold near 1.12. The danger is that they sit **beneath the economy's noise floor**. A rare
depth milestone whose reward is worth less than the next two floors is precisely the *what did that
just do for me* failure this project exists to prevent — arriving by arithmetic rather than by a
soup of stacked modifiers.

## Consequences

- **The surviving axes are the saturating ones.** Income absorbs unlimited relics without noticing
  them; `−20%` cooldown forty times over is no cooldown. So flat-axis levers must be **few and
  substantial**, and catalogue growth has to come from **conditionals**, where each condition
  carves out an axis of its own and nothing saturates. This is why the relic count can grow large
  while the effect vocabulary stays small.
- **Magnitudes compose multiplicatively, never additively.** Additive percentages on a saturating
  axis reach zero and then absurdity; `×0.8` five times is `×0.33` and never `×0`.
- **A future contributor will propose a flat income or lock-cost relic**, and it will look
  completely reasonable. It is not wrong because it is unbalanced — it is wrong because it is
  invisible. Convert it to a conditional, or drop it.
- **This is why relic ranks exist and are not just more relics.** Depth on a flat axis has to come
  from ranking one relic up, since a second relic on the same axis is forbidden by the
  one-relic-per-axis rule.

## Amendment — `type cap` retired (ticket 15)

`type cap` was one of the two axes reserved for type affinity. Measured, it is beneath this ADR's
own noise floor for two of the three types: at **three times** the cap, melee is worth 0.0% and
ranged 0.0%. Only the healer's does anything (×2 → +4.1%, ×3 → +5.4%), because single-target
healing means the twentieth healer still adds a full unit of sustain while damage saturates
against the difficulty curve.

It is **retired from the closed enum** rather than kept as a healer-only entry. A closed
vocabulary earns its closedness by every entry being real, and "cap, but only the healer's"
documents a tuning accident rather than a design axis. The healer's marginal value is kept as a
finding — it is where the slack in the column is — but not as vocabulary.

This leaves type affinity with neither axis intact: [0010](0010-combat-is-floor-local-and-has-no-reach.md)
already narrowed `type stat` to damage by showing a survivability relic is beneath the noise floor
too, since replacement compresses a 10× health swing into 1.3× of survival. What a relic that
"favours ranged climbers" can actually do is now an open question, and it sits in the map's fog
with the relic catalogue.
