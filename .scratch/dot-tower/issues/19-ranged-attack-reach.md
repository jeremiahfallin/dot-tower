# Ranged attack reach

Type: prototype
Status: open

## Question

Does combat have positional reach, and does giving ranged climbers one change anything?

Today it does not. In ticket 06's model everyone on the contested floor fights, and incoming
damage is split by **threat weight** — ranged carries `threat: 1.0` against melee's `3.0`. Ranged
safety is therefore *statistical*, not positional: a ranged climber is less likely to be hit, but
it is standing in the same place as everything else. Ticket 03 put positions in a continuous,
clamped, **floor-local** space, and ticket 16 set the hero's aura reach to ±0 floors precisely
because combat concentrates everyone on the contested floor.

Raised by ticket 11. Relics can only modify a reach that already exists, so the ranged-affinity
relic has no legible form until this is settled — with it, the obvious one is **extend it**.

Resolve:

1. **Does ranged get real reach** — firing before enemies arrive, and firing while melee absorbs —
   or is threat-weighting the intended abstraction with reach as sprite flavour?
2. **What does reach cost the model?** Floor-local positions are already continuous, so
   within-floor reach may be nearly free; reach that *crosses* a floor reopens the distance-metric
   gap ticket 05 raised and ticket 16 closed by choosing ±0.
3. **Does it change the outcome?** Measure it in ticket 06's model the way ticket 16 measured aura
   reach — which found ±0 through ±5 identical, and that null result is the live hypothesis here
   too.
4. **What does it do to composition?** If ranged becomes positionally safe rather than
   statistically safe, the melee/ranged balance moves and **ticket 15 needs the answer**.
5. **Is it legible?** A climber that is safe because of where it stands can be seen; a climber
   that is safe because of a threat weight cannot.
