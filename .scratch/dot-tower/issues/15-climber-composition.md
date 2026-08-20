# Climber composition and the per-type caps

Type: grilling
Status: open
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
