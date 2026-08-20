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
