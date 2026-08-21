# Climber composition and the per-type caps

Type: grilling
Status: resolved
Blocked by: 06

## Question

Who decides how many of each climber type are in the stream, and what makes that
a real choice?

Ticket 05 established that the population cap is **per climber type**, not global.
That removed the constraint this project was implicitly relying on: healers do not
compete with melee or ranged for slots, so "more healers" costs nothing in damage
output. The only remaining competition is **gold** — rank is per type, so gold into
healer rank is gold not into melee rank.

That is a materially weaker constraint, and it may not be enough. Resolve:

1. **Are the per-type caps fixed, or upgradeable?** If upgradeable, cap is a second
   spending axis alongside rank, and the composition lever. If fixed, composition is
   authored and the player only controls rank.
2. **What are the caps at the start of a run**, and are they equal across types? An
   equal split is a design statement, not a default.
3. **Is gold competition actually sufficient** to make composition contested? If
   rank is cheap relative to income, the player maxes everything and composition
   stops being a decision.
4. **Does the player see composition at all** — is the melee/ranged/healer mix a
   number on screen, or something inferred from watching the stream? The legibility
   constraint applies: if composition is a decision, its effect must be visible.
5. **Do relics move composition?** ADR-0003 permits relics to favour specific
   climber types. A relic that shifts the mix is qualitative and legal — confirm
   that is the intended lever rather than an accident.
6. **What happens at the cap** when a type is full — is the spawn skipped, or does
   the spawn slot pass to another type?

Blocked on ticket 06 because whether gold competition bites at all depends on the
income and rank cost curves.

## Added by ticket 06

The ground under this ticket moved. The per-type caps (melee 40, ranged 30, healer 20) were
sized against an **aura**, whose value scaled with how many wounded units sat in radius. The
healer is now **single-target**, so a healer's output is one unit's worth regardless of how many
are hurt — the marginal value of the twentieth healer is completely different from what the cap
was chosen for.

Two measurements worth carrying in:

- Healing lands roughly **66% melee, 17% ranged, 5% healers, 13% hero**. Melee dominates because
  it carries the highest threat weight and eats most of the incoming damage.
- Replacement rate is per type and only fires while that type is under its cap — settled at 5s.
  It turned out to be the run-length dial for the whole game, so composition and pacing are
  coupled through it.

## Added by ticket 16

Healer supply is now coupled to hero choice. A taunting tank absorbs **38.4% of all healing** in a
run — it stands in front of the climbers, so healing spent on it comes back to them (climbers reach
the wall at 90% rather than 86%, and the hero's uptime rises 73% → 86%). But a third of your
sustain is a large standing commitment, and it is a commitment the player makes by *picking a
hero*, not by buying healers.

So the healer cap and rank question this ticket owns cannot be answered independently of which
hero is stationed. A non-taunting hero takes 3.5%.

## Added by ticket 11

Relics move composition, confirmed — this ticket's question 5 is answered as **yes, deliberately**.
The levers are the `type cap` and `type stat` effect kinds, and they arrive with constraints
attached:

- **Type cap is a relic effect kind**, so the caps this ticket sets are a floor that relics raise,
  not a fixed authored value. If this ticket makes caps player-upgradeable too, cap becomes a
  *third* thing moving one number and the attribution problem returns.
- **A type-stat relic must be large enough to change composition** — `+15%` melee health is
  exactly 1.0 ranks and is banned as noise, while doubling melee health moves healer demand
  because melee absorbs 66% of all healing
  ([ADR 0003 amendment](../../../docs/adr/0003-relics-modify-prestige-multiplies.md)). So relics
  can only shift the mix in *large* steps, which means they cannot be this ticket's fine-grained
  composition dial.
- **Rank-cost relics are dead** — `rank_cost_base` 1.30 means a −20% relic buys 0.85 of one rank.
  Gold competition cannot be relieved by relics, so if this ticket finds gold competition
  insufficient, relics are not the escape hatch.
- **Replacement interval is also an effect kind**, and ticket 06 found it is the run-length dial
  for the whole game — so a spawn-rate relic is a run-length relic, and this ticket's coupling of
  composition to pacing runs through it.

Ticket 19 (ranged attack reach) was split out of ticket 11 and is upstream of this ticket's
question: if ranged becomes positionally safe rather than statistically safe, the mix moves.

## Added by ticket 19

Ticket 19 went looking for whether ranged should have positional reach, found the answer was no
([ADR 0010](../../../docs/adr/0010-combat-is-floor-local-and-has-no-reach.md)), and found a
larger problem that lands squarely on this ticket: **no climber type has a survival identity, and
the machinery this ticket owns is why.**

Measured, as deaths per live climber over a 40-minute run:

| | melee | ranged | healer | melee vs ranged |
|---|---|---|---|---|
| as shipped (threat 3.0 / 1.0 / 0.5) | 9.6 | 8.9 | 7.6 | **1.08×** |
| ranged threat cut 4× to 0.25 | 10.1 | 9.5 | 8.3 | 1.07× |
| threat weighting **off entirely** | 9.4 | 9.7 | 7.9 | **0.97×** |
| standoff — ranged literally untouchable | 9.7 | 9.4 | 6.6 | 1.03× |

Melee dies 1.08× as often as ranged. Delete threat weighting entirely and it is 0.97×. Make
ranged *invulnerable on the contested floor* and it is 1.03×.

The cause is **per-type replacement**. A type sitting at its cap that dies faster simply respawns
faster, so deaths-per-capita converges on the replacement interval no matter what combat did. The
damping is severe rather than total: **a 10× swing in ranged health produces a 1.3× swing in
survival** (hp 30 → 300 moves 8.9 → 6.7; hp 30 → 3 moves 8.9 → 18.0).

What this does to the questions above:

- **Question 3 gains a second front.** Gold competition may or may not be sufficient to make
  composition contested; either way, composition currently cannot be *expressed* through
  survivability, because the caps and the 5s interval overwrite it. Whatever lever this ticket
  chooses has to clear that compression, and `hp0` at ten times its value does not.
- **Question 6 is load-bearing, not a detail.** Whether the spawn is skipped or passes to another
  type at the cap is what decides whether death rate is a combat output or a scheduling artifact.
  Today it is a scheduling artifact.
- Ticket 06 found the replacement interval is the **run-length dial for the whole game**. It is
  the **survival dial** too, so this ticket cannot move it for composition reasons without moving
  run length — which is ticket 17's. The two are coupled through one constant.
- **Question 5's answer narrows.** Ticket 11 already ruled that a type-stat relic must be large
  enough to move composition; the compression measured here says how large. A relic that doubles
  melee health buys about a 1.06× change in how often melee dies, so a type-stat relic aimed at
  *survivability* is beneath the noise floor for the same reason ticket 11's income relics were.
  Type-stat relics have to act on damage, or on the cap.
- Ranged reach is **not** available as a composition lever, and the reason is worth carrying: it
  makes ranged fire-and-forget, dropping its share of all healing from 23% to 2% and removing it
  from the sustain economy — deleting a composition interaction rather than creating one.

## Answer

**Composition is authored, not chosen.** The three climber types spawn in a fixed ratio the player
never sets, and the game has four decisions — rank, locks, which hero, when its abilities fire —
not five. Recorded as
[ADR 0011](../../../docs/adr/0011-composition-is-authored.md).

This ticket has been carrying the assumption since charting that composition is a player decision.
Two tickets' worth of measurement now say it never was, and the machinery to make it one does not
exist: ticket 19 showed type identity cannot be expressed through survival, and the measurements
below show it cannot be expressed through the mix either.

### The mix is a wide flat basin, and at depth it is nothing

Eleven splits of a fixed 90-climber budget, every run pinned to identical wall-clock time:

| | run 1, locks as shipped | run 1, locks fixed | run 6, locks fixed |
|---|---|---|---|
| best split | 149 (20/40/30) | 156 (30/30/30) | 1232 (70/10/10) |
| baseline 40/30/20 | 147 | 148 | 1230 |
| worst split | 122 (10/70/10) | 131 (10/10/70) | 1224 (10/70/10) |
| **spread** | **22.1%** | **19.1%** | **0.7%** |

Across spending strategies — buy-cheapest, buy-even, all-in on one type for a whole run — the
spread is 140 to 148 in run 1 and 738 to 749 at depth. **Question 3 is answered: gold competition
is not sufficient, and it is not close.** Rank cost at 1.30/rank rises fast enough that income
forces you to buy all three regardless of intent; ranks land at 55/54/53 whatever the player does.

The `0.7%` is the number that decides the ticket. **Composition matters while you are weak and
stops mattering once you are strong**, so as a decision it would be one that *expires* — the worst
possible shape in an idle game whose player is meant to still be deciding things at depth. This
was checked specifically against the suspicion that the late game's inertness was the lock curve's
fault; it is not. Fixing the lock curve makes composition **more** inert, not less.

Each type does earn its place — removing one costs 12.9% (melee), 11.6% (ranged), 20.4% (healer).
The stream needs all three. It does not need the player to decide the mix.

### What the types are for (question 4, inverted)

**Legibility.** The tower has to read as an army fighting: melee falling at the front, archers
behind, healers keeping them up. A homogeneous stream would make the wall unreadable, and the wall
is the thing the player is watching. That is a legitimate reason under this map's standing rule
that legibility has veto power — the same reasoning that took ticket 16's aura to ±0 when ±0
through ±5 measured identically — but it has to be written down, or a future contributor will find
three types with no strategic difference and conclude they should be merged or given one.

The division of labour is real but incidental: melee is the health pool, ranged is the damage,
healer is the sustain. It makes one gold axis legible — rank buys *more army*, and the army has
parts.

### Settled

- **Caps are never purchasable with gold** (question 1). Raising every cap is a monotonic,
  saturating power increase (×5 → +15.6%) that is indifferent to the split, so as a gold axis it
  competes with rank while doing rank's job. Rank stays the single gold-bought power axis, exactly
  as [ADR 0005](../../../docs/adr/0005-the-hero-multiplies-climbers-add.md) keeps the aura off it.
- **One global replacement interval plus an authored ratio**, replacing three caps and three
  intervals. Six numbers were quietly steering three unrelated things — the ratio, the run-length
  dial (ticket 06) and the survival dial (ticket 19). One interval decouples them, so ticket 17 can
  move run length without moving composition. Caps stay per type as a **ceiling**, and under the
  fixed lock curve they stop binding entirely (40/30/20 with 3/3/3 alive).
- **At the cap the spawn is skipped** (question 6) — and with a ratio there is nothing to skip.
  Passing the slot to another type would let an invisible rule rewrite the authored ratio, drifting
  it toward whatever the replacement machinery is churning: composition set by an accident of
  scheduling.
- **The ratio is authored to read as an army, not tuned to the optimum** (question 2). Near-equal
  measures best and an army that is one-third medics does not read like one; the mechanical
  difference expires and the fantasy does not. The integers themselves are authoring rather than
  deciding, and wait on ticket 20 to say how big the crowd is — left in the map's fog.
- **The player never sees the mix as a number** (question 4). No panel, no readout; the tower
  column is the display. With locks fixed the whole stream is 42 climbers in run 1 and **9 at
  depth**, all within a floor of the wall — directly countable. This makes telling the three types
  apart at a glance load-bearing rather than a nicety, graduated out of the fog as ticket 21.

### Question 5 is withdrawn rather than answered

Ticket 11 confirmed relics move composition through `type cap` and `type stat`. Both are now
measured as near-inert. Ticket 19 took `type stat` down to damage-only — a survivability relic is
beneath the noise floor, because replacement compresses a 10× health swing into 1.3× of survival.
And `type cap` is worth **0.0% for melee and 0.0% for ranged** at three times the cap; only the
healer's does anything (×2 → +4.1%, ×3 → +5.4%), because single-target healing means the twentieth
healer still adds a full unit of sustain while damage saturates against the curve.

**`type cap` is retired from the closed enum.** A closed vocabulary earns its closedness by every
entry being real, and "cap, but only the healer's" documents a tuning accident rather than a design
axis. [ADR 0009](../../../docs/adr/0009-relic-effects-avoid-axes-the-curve-erases.md) amended. The healer's marginal
value is kept as a *finding* — it is where the slack in the column is — but not as vocabulary.
This leaves ticket 11's type-affinity story needing a rethink, since both of its axes are now gone
or narrowed; that is in the fog with the relic catalogue, not solved here.

### Carried to ticket 20

The lock curve was going to block this ticket and no longer does — composition is inert under both
curves, so the answer did not need it. But the finding that came out of chasing it is the largest
on the board, and it is ticket 20's: **at depth the game is travel time, not combat.** ×10 climber
damage moves the peak floor by nothing (749 → 749). The only thing that moves it is the lock line.
The second half matters as much as the first: the provisional fix trades a long walk with a big
crowd for **no walk and almost no crowd** (9 climbers at depth), and neither is "an endless stream
of climbers".

Prototype: `.scratch/dot-tower/prototypes/15-composition/`.
