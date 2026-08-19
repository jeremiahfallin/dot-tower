# Climber lifetime and the healer

Type: grilling
Status: resolved
Blocked by: 03

## Question

Do climbers persist long enough for healing to be meaningful, and if so, what is the unit of survival?

The tension: climbers are settled as disposable and anonymous, with the climber *type* carrying identity. But a **healer** type only makes sense if the things it heals live long enough to be healed. A stream of mayflies cannot be supported.

Resolve:

1. **Climber lifetime.** Does a climber exist for one floor, several floors, until it dies, or until it reaches the frontier?
2. **Is there a party?** Do climbers travelling together form an implicit group with shared fate, or are they independent entities that happen to be adjacent?
3. **What does the healer actually heal** — individual climbers, the group, or the hero?
4. **Does the hero have HP** and can it die? A stationed hero that can die makes placement genuinely risky; one that cannot makes it purely economic.
5. **Does climber death cost the player anything** beyond lost progress — gold, time, a respawn delay?
6. **Consequences for the simulation boundary.** Persistent climber HP is state that must survive the seam decided in ticket 03. Confirm the answer is compatible.

The legibility constraint applies: whatever the answer, the player must be able to see why a climber died.

## Answer

**Climbers persist until death. The individual is the unit of survival.**

A climber spawns at the lock line, carries its own HP, and climbs until something
kills it. HP **does not regenerate** — not on floor clear, not out of combat, not
over time. A healer is the only source of sustain in the game.

This is what makes the **Wall emergent rather than declared**. Climbers arrive at
a floor already damaged by the floors below it, so the wall is simply where the
stream's surviving HP crosses enemy output. Nothing defines it; it falls out of
the numbers, and it moves when the player invests. It is also visible without
explanation — the wall is the floor where the column piles up and dies.

Population reaches equilibrium on its own: spawn rate balances death rate at the
wall. Ticket 03's ~90 figure is that steady state, not a budget.

**There is no party.** Climbers are independent entities that happen to be
adjacent. `CONTEXT.md`'s "never named, tracked, or directly commanded" stays
intact, and ticket 12 has no group identity to serialise. Grouping was only ever
needed to give the healer something to heal, and proximity supplies that: climbers
bunch at the contested floor by themselves.

**The healer is a pure-support aura.** It heals every damaged friendly in radius
at a rate, with no target selection, and deals **zero damage**.

- Target selection was rejected because it would quietly reintroduce the individual
  identity that "no party" just removed. An aura treats climbers as the anonymous
  mass they are defined to be, needs no per-tick search at 10Hz, and is legible on
  sight: a visible radius, everyone inside recovering.
- "Lowest absolute HP" would have been a trap — the hero's pool dwarfs a climber's,
  so healers would have silently become hero-dedicated.
- **Auras stack linearly**, bounded by tuning the per-healer rate rather than by a
  curve. Two healers heal twice as fast, and the player can learn that by watching.
  Diminishing returns is invisible math and is exactly the failure this project
  exists to avoid.
- Targeting is built as a **swappable policy behind a seam**. The aura is the
  default, not a hardcoded assumption.

**The hero has HP, can die, and respawns on the floor it is stationed on** after a
timer. Because it returns to its post rather than the lock line, the entire cost of
death is **downtime** — which makes the throttle *uptime*. Station at the wall and
your contribution is your survival fraction; station below it and you run at 100%.
That is the "throttle, not wall-breaker" property the map asked for, expressed as a
single tunable number rather than a special rule.

**The hero is a valid heal target with no special-casing.** Healers heal whatever
damaged friendly is in radius. This buys a real link between the two systems for
free: hero placement becomes partly a question of climber composition, and investing
in healer rank widens the uptime throttle rather than removing it.

**Climber death costs progress and nothing else.** Fixed spawn rate, no gold cost
per spawn, no escalating respawn penalty. A dead climber is deleted progress whose
replacement re-walks from the lock line — a throughput tax the player can watch. A
gold cost would turn climbers into a managed resource, contradicting "expendable,
anonymous, never directly commanded"; an escalating penalty would punish the player
hardest exactly when they are already losing.

**The population cap is per climber type, not global.** Each of melee, ranged and
healer fills independently. Two consequences worth stating plainly:

- Healers do **not** compete with damage types for slots. The composition tradeoff
  is therefore **gold**, not population — gold into healer rank is gold not into
  melee rank. This is a materially weaker constraint than slot competition, and it
  is why composition needs its own ticket (15).
- Healers are climbers, so healers die at the wall too. Sustain thins exactly where
  it is most needed, which sharpens the wall into a real boundary rather than a soft
  gradient.

**Legibility.** HP bars appear only on *damaged* climbers, so an undamaged stream
stays visually silent; deaths get a distinct effect; and the wall floor carries a
**persistent aggregate readout** — what floor the wall is at, and the average HP
climbers arrive there with.

The reframe that drove this: "why did *that* climber die" is almost never the
question the player is asking. They are asking **"why am I stuck"**, and the honest
answer is aggregate. Per-unit feedback exists to make the aggregate credible, not to
be read on its own. Floating damage numbers were rejected outright — 90 units of
numbers on a portrait phone is static, not information.

**Player-facing targeting control is deferred**, not rejected. The seam ships; the
UI does not. A standing order set **per climber type** would not violate "never
directly commanded", because type is where identity attaches — per-climber orders
would. That distinction is what makes later exposure consistent rather than a
reversal.

### Item 6 — compatibility with the simulation boundary (verified, not assumed)

Persistent climber HP **does not cross ticket 03's seam**. Climbers exist only in
the active region; the sealed region holds zero entities by construction. When
locking raises the line past trailing climbers they *sprint up* rather than being
absorbed, so a climber is never sealed away with HP that would need preserving.
Ticket 03's "per-enemy HP does not persist across deactivation" governs *enemies*;
climbers are persistent entities and unaffected by it. The immediate-reset rule for
uncontested-but-not-cleared floors still holds, and still blocks attrition.

Three consequences fall out that the boundary did not previously have to answer:

1. **The aura needs a radius rule in `FloorPos` space, which is not Euclidean.**
   `FloorPos { floor: u32, x: NormX }` pairs a discrete floor with a normalised
   0..=1 position, so "radius" has no meaning until it is defined as, for example,
   same floor ± N floors within Δx. This is a genuine gap in ticket 03's coordinate
   system, surfaced here.
2. **Ticket 12 must serialise live climber state** — roughly 90 × (floor, x, hp,
   type). Small, but it is state that did not exist before this ticket.
3. **A hero stationed below a rising lock line is an unhandled case.** Recorded as
   an addendum on ticket 09 rather than answered here.

Status: resolved
