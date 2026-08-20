# The hero multiplies, climbers add

The hero's contribution to the tower is a **multiplier applied to what climbers already do**,
never a quantity added alongside them. Its aura multiplies the damage dealt and the gold earned
by climbers standing on its floor. Its own auto-attack exists for identity and is explicitly not
load-bearing.

This was measured, not assumed. Every additive contribution tried — damage, tanking, healing, a
patrol zone that held floors clear, health regeneration — moved the peak floor by under 2%.
Stationed at the wall, below it, or **not stationed at all**, the run ended in the same place:
208 / 205 / 205. The cause is structural rather than a tuning failure. One entity added to a
stream of seventy is one seventieth, and the ratio only worsens as population caps rise, so no
amount of tuning an additive hero can outrun the stream it is competing with.

A multiplicative contribution escapes this because its value scales *with* the stream instead of
against it, and it stays as relevant on floor 5,000 as on floor 50. Measured, a ×2 aura buys
about +21 floors where an additive hero bought +2.

## Consequences

- The hero remains a **throttle rather than a wall-breaker**, and provably so: the gain is
  sub-linear in the multiplier (×2 → +21 floors, ×3 → +29, ×5 → +49). Multiplicative in effect,
  but a bounded floor offset in outcome. It can shift a wall, never break one open.
- **The aura multiplier is not bought with gold.** Buying a multiplier with the same currency
  that buys addition makes the hero the only sane purchase. It is a fixed constant per hero,
  moved only by relics — which is exactly the qualitative modifier
  [0003](0003-relics-modify-prestige-multiplies.md) reserves relics for.
- The hero levels on **experience earned from kills inside its aura**, never on gold, so the two
  economies never touch and the hero never appears in a purchase decision.
- Hero uptime is genuinely load-bearing for the first time: a multiplier that only applies while
  the hero lives means a hero at 40% uptime delivers 40% of its contribution. This is what ticket
  05 claimed the throttle was; it was not true while the contribution was 2%.
- This is the decision most likely to be "fixed" by a future contributor who notices the hero
  feels weak and gives it a large damage number. That has been tried. It is worth 2%.
