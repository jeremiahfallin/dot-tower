# Ticket 19 — the ranged-reach measurement

Throwaway. Like ticket 07's, this is not a prototype in the `prototype`-skill sense — there is
nothing to click. It is the measurement behind ticket 19's decision, kept so the numbers can be
re-run rather than taken on trust.

Ticket 06's `model.mjs` was extended in place (as ticket 16 did) with four knobs, **all defaulting
to off**, so every earlier measurement still reproduces — ticket 08's run-1 peak of 147 included:

- `reach: {melee, ranged, healer}` — floors above its own a type can attack across. A climber
  attacks the nearest floor with enemies within reach, own floor first, and never two floors, so
  reach buys range rather than output.
- `standoff` — a type with reach halts as soon as the fighting comes into reach instead of walking
  into it. This is the knob that turns statistical safety into positional safety.
- `targeting: 'threat' | 'line'` plus `formation` and `lineFocus` — whether incoming damage splits
  by threat weight (ticket 06) or falls front-to-back through a formation, and whether a rank
  shares the hit out or the weakest unit is finished off first.
- `failedFloorReset` — ticket 03's rule that a floor nobody is fighting restores its pack. The
  model has never done this. `false` is what tickets 06–16 actually measured.

Run in order; each pass corrects the one before it.

- `reach.mjs` — the first sweep, and the one that is **wrong**. Kept deliberately: it makes
  front-to-back targeting look like +29 floors and 14× gold, which is entirely an artifact of the
  stall heuristic. Variants that grind rather than stall run 174 minutes against the baseline's
  41 and reach a deeper floor for that reason alone.
- `normalise.mjs` — re-reads the same runs at fixed wall-clock marks, which reverses the headline.
- `fixed.mjs` — the decisive pass. Stall detector disabled, every variant run for identical
  wall-clock time, so the mechanic is the only thing that can differ. **Read this one.**
- `why.mjs` — checks the two stories: that the standoff gold collapse is the hero's ±0 aura
  (widen it to ±2 and gold goes 0.84× → 1.10×), and that attack reach is a damage buff
  (reach ±2 = 2.22× gold; +100% ranged damage = 2.01×).
- `safety.mjs` — the finding that outgrew the ticket. Per-capita death rate by type, which is what
  a player actually watches. Ranged is not safer; replacement rate sets the death rate and
  compresses a 10× health swing into a 1.3× survival swing.

`node fixed.mjs` is the one to run if you only run one. Roughly 25s each; the model is
deterministic, so differences between rows are real for that configuration — but they are
tuning-sensitive, not structural, wherever the ticket says so.
