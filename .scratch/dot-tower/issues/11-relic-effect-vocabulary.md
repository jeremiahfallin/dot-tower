# Relic effect vocabulary

Type: grilling
Status: open
Blocked by: 05, 06

## Question

What kinds of effects can a relic have, and how many exist in the slice?

Settled: relics modify, prestige multiplies. No relic reads "+15% damage" — that is prestige's job. Relics grant qualitative effects and may favour a specific climber type. Not settled:

1. **The effect vocabulary.** Income rate, lock cost reduction, ability cooldown, spawn rate, climber lifetime, type-specific behaviour. What is the closed list, and is it closed?
2. **Type affinity.** What does "a relic that favours ranged climbers" actually change — their damage profile, their spawn share, their positioning, their survivability?
3. **How are relics obtained?** Milestone chests at floor thresholds, as in the reference game? Prestige rewards? Both?
4. **Do they stack?** Two relics affecting spawn rate — additive, multiplicative, or does the better one win? Stacking multipliers is how the "what did that do" problem returns through a side door.
5. **How many in the slice?** Possibly zero. Relics are the most droppable system on the board, and the slice may prove the loop without them.
6. **Legibility.** How does a player see which relics are active and what each is currently contributing?

## Added by ticket 16

A concrete relic target, and a constraint. The hero's **aura multiplier is a fixed constant per
hero, moved only by relics** — deliberately kept off the gold economy, because buying a multiplier
with the currency that buys addition makes the hero the only sane purchase. So "widen the aura" and
"raise the aura multiplier" are exactly the qualitative effects this ticket is cataloguing, and
they are the hero's *only* route to getting stronger beyond levels.

Note the aura multiplies gold as well as damage, so a relic touching it is an income relic and a
combat relic at once. Whether that counts as one effect or two is this ticket's problem.

## Added by ticket 10

Relics have no place on the screen, and ticket 10's layout rule means they only get one by being
**rare**. [ADR 0008](../../../docs/adr/0008-spending-is-never-modal.md) prices modality by frequency:
ranks and locks are permanently on screen because they are bought every few seconds, and relics are
assigned a modal surface on the assumption that they are reached a handful of times a run.

So this ticket now has to answer one more thing, and the layout depends on it: **how often does the
player acquire or interact with a relic?** If the answer is "often enough to be part of the moment
to moment", the modal surface is wrong and relics need permanent space the portrait frame does not
have — which would reopen ticket 10 rather than extend it.
