# What the hero contributes that climbers cannot

Type: grilling
Status: resolved
Blocked by: 06

## Question

The map's charting settlement says hero placement is a **throttle**: below the wall it farms
and escorts, at the wall it pushes. Ticket 06's model says that throttle does not exist.

Measured, across every configuration tried:

- Stationed at the wall, below the wall, or **not stationed at all**, the peak floor moves by
  under 2% — 208 / 205 / 205.
- Giving the hero a patrol zone to hold clear changes nothing, and a wide zone makes it worse
  (±5 → 168, ±20 → 140, against 173 for its own floor alone).
- Health regeneration changes nothing. The hero does not die often enough for uptime to bind.
- The escort fantasy — the hero keeps a floor cleared so climbers pass through free, the
  Dot Heroes II feeling — only appears when climbers arrive badly damaged. With healing off
  entirely (60% arrival health) the spread is real: 142 unstationed against 169 at the wall.
  At the 83% arrival that shipped in ticket 06, it is still flat.
- The hero absorbs **13% of all healing** in a run, and removing it from the healer's
  candidate set *raises* the peak floor from 172 to 178.

The arithmetic behind it: seventy climbers out-damage one hero roughly six to one, and the
hero holds exactly one floor out of a hundred-floor band. Damage cannot make it matter
without making it a wall-breaker, which the map explicitly rules out.

So: **what does the hero do that a climber cannot?**

Resolve:

1. Is the hero's contribution qualitative — a floor-wide effect on climbers passing through,
   something the four abilities do — rather than damage or tanking?
2. Is it economic? A gold effect on its floor would make "station below the wall" a real
   choice without touching combat at all.
3. Does the answer keep placement a *throttle* rather than a wall-breaker, and does the
   tradeoff show up as a number the model can measure?
4. Should the hero stay in the healer's candidate set? Ticket 05 says yes with no
   special-casing; ticket 06 measured that at 13% of sustain for no return.

This must resolve before ticket 09, which is about making the tradeoff *visible* and
currently has no tradeoff to show.

## Answer

**The hero multiplies what climbers do; it never adds to it.** Recorded as
[ADR 0005](../../../docs/adr/0005-the-hero-multiplies-climbers-add.md), because it is the
decision most likely to be "fixed" later by someone who notices the hero feels weak and gives it
a big damage number.

### The mechanism

- **The aura.** Every hero projects the same aura: it multiplies the **damage dealt and the gold
  earned** by climbers on its floor. Reach is **its own floor** — ±0 through ±5 measured
  identically (peak 158/158/159/159/158), because combat concentrates everyone on the contested
  floor, so the narrowest reach is chosen for legibility. It also dissolves the `FloorPos` radius
  gap ticket 05 raised: a reach that never crosses a floor needs no distance metric.
- **One aura, universal.** Archetype is expressed through abilities, health, and taunt — never
  through a different aura kind. One concept to teach, one number to make legible, and one
  feedback treatment for ticket 09 rather than one per hero.
- **Auto-attack stays, explicitly non-load-bearing.** Worth +2 floors. A stationed hero standing
  inert would read as broken, and a unit whose visible behaviour does not explain its value
  violates the legibility constraint from the other direction.
- **Taunt is a per-hero property.** A taunting hero is the primary target on its floor. It pays
  for itself only on a tanky hero: at 1,260 health it buys +3 floors and lifts climber arrival
  health from 53% to 90%; at 140 health it costs 6 floors and a quarter of the experience rate.
  This is what makes the roster differ in the simulation rather than only in the ability list —
  and it delivers the Dot Heroes II escort feeling as a consequence of hero choice.
- **Abilities act on state the aura cannot reach** — enemies, dead climbers, the aura's own
  position — and are per-hero. Not bursts of the aura: an ability that is merely "more multiplier"
  is always best fired immediately, which makes it a cooldown tax rather than a decision. The test
  for each of the four: **is there a wrong moment to press it?**

### Progression

**The hero levels on experience, never on gold.** Experience accrues from enemies defeated inside
its aura while it is alive; kills elsewhere in the tower grant none. Gold buys climbers and locks,
experience grows the hero, and the two economies never touch — so the hero never appears in a
purchase decision at all. Levels raise the hero's own health and damage. **The aura multiplier is
a fixed constant per hero, moved only by relics**, which keeps a multiplier from being bought with
the currency that buys addition.

A coupling falls out for free: a healthier climber stream kills more, which levels the hero faster.

Vocabulary settled and written into `CONTEXT.md`: **a rank is bought, a level is earned.** Rank
applies to a climber type and costs gold; Level applies to the hero and costs nothing. Both reset
on prestige. *Level* is no longer a retired synonym of *Rank*; **Experience** is a new term with
*XP* retired.

### What the model refuted

**"Below the wall it farms and escorts, at the wall it pushes" is retired.** It cannot be made to
work. Stationing at the wall dominates on every axis measured — 7.5× the gold (2.8×10⁹ against
2.8×10⁸) and 3× the experience (2,123 aura kills against 763). The cause is ticket 06's gold curve:
gold scales at 1.095/floor against difficulty at 1.075, so climbing 21 floors higher multiplies
income by ~6.9× and no farming multiplier catches that. It is the same force that killed sealed
income — **depth beats throughput** — and flattening it would remove the gradient that pulls
players upward.

So **placement is not a strategic tradeoff. It is a learnable rule: put the hero where the
fighting is.** *Which* hero and *when* its abilities fire are the decisions.

**Ticket 05's uptime claim was backwards.** It says station at the wall and your contribution is
your survival fraction, below it you run at 100%. Measured without taunt, the reverse holds — 98%
uptime at the wall against 71% below — because seventy climbers bunched at the wall soak the
damage while a hero farming below stands alone against a whole pack. Taunt is what makes the
wall genuinely costly, and therefore what makes ticket 05's intent true.

### Stationing

- **A hero below a rising lock line sprints up**, exactly as trailing climbers do under ticket 03.
  Blocking the lock would make a good purchase conditional on where a unit stands. Ticket 09's
  objection — that silent relocation overrides a strategic choice — is much weaker now that
  placement is a rule rather than a tradeoff, and locking moves the hero toward the wall anyway.
- **The hero stays in the healer's candidate set.** Ticket 05 was right, for a reason that was not
  true when written: hero survival now gates both the aura and the experience, and a taunting hero
  is standing in front of the climbers. Measured, keeping it in gives +2 floors, uptime 73% → 86%,
  and climbers arriving at **90% instead of 86%** — healing spent on the hero returns to the
  climbers. It costs **38.4% of all healing**, which couples healer supply to hero choice tightly
  enough that ticket 15 must know.

### The slice ships the tank

It is the only archetype that exercises the whole mechanism — taunt, absorption, uptime gating
both aura and experience, and healers extending that uptime. Shipping a mage would leave taunt
untested until the roster arrived, which would make the roster the first time the system's central
interaction ever ran.

Prototype extended with the aura, taunt, experience tracking and hero uptime:
`.scratch/dot-tower/prototypes/06-progression-curve/`.
