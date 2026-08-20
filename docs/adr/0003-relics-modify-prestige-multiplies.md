# Relics modify, prestige multiplies

The prestige multiplier is the sole source of permanent raw health and damage. Relics grant only qualitative effects — income rate, lock cost, ability cooldowns, spawn rate, climber-type behaviour — and never read "+15% damage".

Both systems persist across prestige, so both are permanent power. Left unseparated they would be two systems doing one job, and the player asking "my numbers went up, which system did that?" is precisely the failure this project exists to avoid: the reference game's most common complaint is a player prestiging without knowing what it bought.

## Consequences

- Prestige owns one number with one meaning, which is what makes an honest before/after screen possible.
- Any proposed relic granting flat or percentage health/damage should become a prestige tick instead. This is the decision most likely to be "fixed" by a future contributor adding an obvious damage relic.

## Amendment — type-specific stat relics (ticket 11)

A relic may modify the **health or damage of a single climber type**, at a magnitude large enough
to change composition. This is the patch the consequence above predicts, and it is admitted
deliberately and narrowly.

The distinction is magnitude, and it is measurable. `+15%` melee health is **exactly 1.0 ranks** of
melee — a purchase the player already makes every few seconds and re-buys every run for pocket
change, so it is scale wearing a relic's clothes and the ban still catches it. Doubling melee
health is different in kind: melee absorbs 66% of all healing, so halving its death rate moves
healer demand and therefore the mix. That is a qualitative change whose unit happens to be health.

The rule: **a type-stat relic must be large enough to change composition, and never small enough to
read as a rank.** Uniform health or damage across all types remains forbidden — that is prestige,
and it has no composition effect to justify it. See
[0009](0009-relic-effects-avoid-axes-the-curve-erases.md) for the arithmetic this rests on.
