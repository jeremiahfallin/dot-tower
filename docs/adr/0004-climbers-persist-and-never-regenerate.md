# Climbers persist until death and never regenerate

A climber carries its own HP from the lock line until something kills it. There is no
regeneration of any kind — not over time, not out of combat, not on clearing a floor.
The healer is the only source of sustain in the game.

This exists to make the **Wall emergent rather than declared**. Because climbers arrive
at a floor already damaged by the floors beneath it, the wall is simply where surviving
HP crosses enemy output — nothing defines it, it falls out of the numbers, and it moves
whenever the player invests. It is also visible without being explained: the wall is the
floor where the column piles up and dies.

## Considered options

Full heal on floor clear, and slow out-of-combat regeneration, were both rejected. Either
would erase the "climbers arrive chewed up" property, collapsing the wall back into
something that has to be defined by hand and tuned as a special case. They would also
demote the healer from a structural pillar to an accelerant, leaving the third climber
type without a reason to exist.

## Consequences

- Healer rank becomes a direct, legible lever on where the wall sits.
- Healers are climbers, so healers die at the wall too. Sustain thins exactly where it is
  most needed, sharpening the wall into a boundary rather than a gradient.
- Live climber HP is state the save schema must carry (see ticket 12). It does not cross
  the sealed/active seam — sealed floors hold no entities — so this costs the simulation
  boundary nothing.
- This is the decision most likely to be "fixed" by a future contributor adding an obvious
  regeneration trickle. That change would silently flatten the wall and strand the healer.
