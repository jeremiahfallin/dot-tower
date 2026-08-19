# What the hero contributes that climbers cannot

Type: grilling
Status: open
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
