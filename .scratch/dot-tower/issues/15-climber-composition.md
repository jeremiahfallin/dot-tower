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
