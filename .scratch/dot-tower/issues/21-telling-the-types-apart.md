# Telling the climber types apart

Type: prototype
Status: open
Blocked by: 15

## Question

Melee, ranged and healer must be distinguishable at a glance, in a pixel-art column, on a portrait
phone, for a player who may be colourblind. How?

Graduated out of the map's fog by ticket 15, which promoted it from "likely to bite" to
load-bearing. [ADR 0011](../../../docs/adr/0011-composition-is-authored.md) settled that
composition is **authored and never shown as a number** — the tower column is the only place the
mix is represented. If the three types cannot be told apart there, the mix has no representation
anywhere in the game, and the reason the three types exist at all (legibility — the tower reading
as an army fighting) fails silently.

The constraints are unusually tight, and mostly already settled elsewhere:

- The **tower column** is the same width on a phone and on a monitor (ticket 10), so this must
  work at phone scale and gets no extra room on desktop.
- `UiAntiAlias::Off`, `Msaa::Off`, `FontSmoothing::None` from day one (ticket 02), so there is no
  anti-aliasing to lean on for silhouette detail.
- Under a healthy lock curve the whole stream is a **handful of climbers** — 42 early, 9 at depth
  (ticket 15) — so this is a legibility problem about a few sprites, not a crowd.
- The hero already carries a health ring, a respawn countdown, an experience fill and an aura
  (ticket 09), and the wall is hatched. The column's visual vocabulary is close to full.

Resolve:

1. **What carries type — silhouette, colour, or both?** Colour alone fails the accessibility
   constraint. Silhouette alone is hard at pixel-art scale in a narrow column.
2. **Does a climber's state need to be readable too**, or only its type? Ticket 05 settled that
   deaths are legible in aggregate at the wall rather than per unit, which may mean individual
   health does not need to show at all.
3. **What is the smallest sprite that works**, and does that dictate the column's floor height?
   This is the one constraint that runs backwards into ticket 10's layout.
4. **Does the healer need to be distinguishable from further away than the others?** It is the
   type whose presence the player has the most reason to notice, and ticket 15 measured it as the
   only type whose numbers have marginal value.

Prototype rather than grilling: this is "what should it look like", so it wants variations to
react to. Blocked on ticket 15 only for its premise, which is now settled — takeable immediately.
