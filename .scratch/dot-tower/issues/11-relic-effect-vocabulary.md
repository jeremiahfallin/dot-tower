# Relic effect vocabulary

Type: grilling
Status: resolved
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

## Answer

### The vocabulary is a seam, not a list

The ticket asked for a closed list. The answer is that **the list of effect *kinds* is closed and
the list of *relics* is open**, because a relic is a `(condition, effect, magnitude)` triple
defined in RON against a fixed enum of effect kinds in Rust. Adding a relic is one RON line;
adding a new *kind* is a Rust change and a design decision. A handful of conditions times a
handful of effects multiplies out into a large, varied catalogue, which is how the relic count
grows without the vocabulary growing.

Embedded scripting was rejected — maximum freedom, a large dependency, and a bad bet while still
learning Bevy. Hand-writing each relic in Rust was rejected because it forecloses exactly the
data-driven tinkering relics are for.

Starting effect kinds: **ability cooldown, replacement interval, type cap, type stat, aura reach,
aura persistence, climb speed, hero respawn, conditional gold** — plus **attack reach** once the
combat model has one (see *Routed out*). Two carry warnings into the enum itself: **replacement
interval is ticket 06's run-length dial for the whole game**, and **type cap is contested by
ticket 15**.

### The axis rule, which is the real finding

**A flat percentage on an axis the curve already drives is arithmetically invisible.** Income
rises 9.5%/floor, so `+15%` income is worth **1.54 floors** — about three seconds of climbing.
`−20%` lock cost buys 1.6 floors of lock line. `−20%` rank cost buys 0.85 of one rank. These
never threatened the economy: forty stacked income relics move the effective `gold_base` from
1.0950 to 1.0970, nowhere near the ~1.12 runaway threshold. They are beneath its noise floor,
which is worse — a rare depth reward worth less than the next two floors is ticket 06's
legibility failure arriving by arithmetic instead of by soup.

The axes that survive are the ones **flat in depth**: cooldowns, replacement interval, type caps,
aura reach, climb speed, respawn. A percentage there is worth the same at floor 20 and floor
2,000. But those are exactly the axes that **saturate** — `−20%` cooldown forty times over is no
cooldown. So income can absorb unlimited relics without noticing them, and cooldown notices every
one and runs out of room.

That tension sets the catalogue's shape:

- **`+N%` is permitted only on flat-in-depth axes.** Anything touching gold, lock cost or rank
  cost must take **conditional** form — *"gold from kills inside the aura is doubled"* survives
  because it changes where you station rather than what a number reads.
- **Few and substantial, then many and narrow.** The flat-axis levers saturate after a handful, so
  they are spent early and rarely; conditionals don't saturate, because each condition carves out
  its own axis. One milestone schedule, two kinds of payload. The collection grows in **breadth**
  over a player's lifetime, not in magnitude.

Recorded as [ADR 0009](../../../docs/adr/0009-relic-effects-avoid-axes-the-curve-erases.md).

### Type affinity, and the carve-out in ADR 0003

Type affinity acts on **behaviour** first and **spawn share / cap** second. Rank economics is dead
on arrival — `rank_cost_base` is 1.30, so a cheaper-ranks relic buys 0.85 of a rank.

A **type stat** relic (melee health, ranged damage) is permitted, but only at a magnitude that
changes composition. `+15%` melee health is **exactly 1.0 ranks** — a purchase the player makes
every few seconds and re-buys every run for pocket change. Doubling melee health is different in
kind: melee currently absorbs 66% of all healing, so it moves healer demand and therefore the mix.
The line is that a type-stat relic must be large enough to change composition and never small
enough to read as a rank. [ADR 0003](../../../docs/adr/0003-relics-modify-prestige-multiplies.md)
forbids this in letter and is **amended** with the carve-out rather than retired — its core claim,
that prestige owns the one number, stands.

### Acquisition, ranks, and the currency

- **Relics are granted at first-time floor milestones**, on a schedule of increasing rarity.
  Acquisition is a **grant, not a decision** — no draft. Bundling relics into the prestige reward
  was rejected to keep ADR 0007's "what did prestige buy" answer exact.
- **Relics accumulate permanently and are all obtainable.** Every relic is unique; there are no
  duplicates and no two relics on one axis, so **stacking never arises between relics**. Where a
  magnitude does compose, it composes **multiplicatively, never additively** — additive
  percentages on a saturating axis reach zero and then absurdity.
- **Relics have ranks**, bought with a currency **granted at prestige**. This widens *Rank* rather
  than borrowing *Level*: the glossary's rule is that a rank is bought and a level is earned, so a
  purchased relic tier is a rank. Unlike climber ranks, **relic ranks do not reset on prestige**.
- **The currency scales with peak floor beyond your previous best.** Only new depth pays. Flat
  per-prestige was rejected because it makes **prestige-spam optimal** — ticket 08 measured
  re-climbing at 3× speed, so a loop of shallow runs would out-earn one deep run and fight the
  stall logic 08 established. Scaling with raw peak floor restates what the multiplier already
  says. "New depth only" is the same rule that already governs relic acquisition, so relics and
  relic ranks come from one rule instead of two.

This was chosen **structurally rather than by measurement, on purpose**: ticket 07 already hit the
limits of the stall heuristic for detecting run-length *trends*, which is ticket 17's open problem.
"New depth only" is the option that does not need the measurement we cannot yet make.

### Legibility

Accumulate-everything means a mature player owns the whole catalogue, so *"which relics are
active"* is answerable and useless — all of them. The work happens elsewhere:

- **Attribution at the moment of change.** A relic's effect is announced when it is acquired or
  ranked, and not narrated again. Under the currency rule that burst happens roughly once per run.
- **Diegetic where possible.** A relic that changes something visible in the tower is read off the
  tower; only the invisible ones need a readout.
- **A catalogue as a reference surface**, for lookup — not as the answer to *what did that do*.

Q13's structure carries weight here: because every relic is `condition × effect`, its text writes
itself in a uniform shape, and the **condition is the part the player can watch for**.

### The screen — ADR 0008 holds

Relic currency arrives **at prestige**, so relic spending is a burst once per run, adjacent to a
surface that is already modal and already rare. **Relics get a modal surface reached at prestige**,
and [ADR 0008](../../../docs/adr/0008-spending-is-never-modal.md) holds unamended; its open
consequence — *"relics have no home yet, and ticket 11 has to say how rare they actually are"* — is
now closed. **The caveat is load-bearing: if relic currency ever becomes earnable during a run,
relics become a moment-to-moment spend and ticket 10 reopens.**

### How many in the slice

Not zero. Relics were the most droppable system on the board, but ticket 16 made the hero's aura
relic-only, and the milestone grant is what gives depth a qualitative reward. **Roughly 8–12
milestone relics across the slice's depth range, weighted to flat-axis levers early, with a
conditional tail.** The exact catalogue is authoring rather than deciding, and waits on the attack
reach ticket and on ticket 15.

### Routed out

**Ranged attack reach is baseline combat identity, not a relic effect, and it does not exist.**
Today ranged safety is entirely `threat: 1.0` against melee's `3.0` — a damage-share with no
position — and ticket 16 set aura reach to ±0 precisely because combat concentrates everyone on
the contested floor. Relics can only modify a reach that already exists. Filed as ticket 19; once
baseline reach exists, the ranged-affinity relic gets its obvious legible form: **extend it**.

### Unnamed

**The prestige-granted currency has no name.** *Vestige* and similar read as near-synonyms of
*relic*, which is the kind of collision this glossary works to avoid, and *ascension* is already
retired. Left to whoever writes the prestige screen copy; noted in the map's fog.
