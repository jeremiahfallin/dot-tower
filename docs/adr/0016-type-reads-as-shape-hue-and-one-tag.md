# 0016. Type reads as shape, hue, and one tag

Date: 2026-09-08
Status: accepted
Resolves: [ticket 21](../../.scratch/dot-tower/issues/21-telling-the-types-apart.md)

## Context

ADR 0011 made composition authored and invisible as a number, which leaves the
tower column as the mix's *only* representation — if the three types cannot be
told apart there, the reason they exist fails silently. The constraints are
tight and mostly inherited: the column is the same width on a phone and a
monitor (ticket 10), anti-aliasing is off from day one (ticket 02), the crowd
spans ~57 climbers early down to a handful at depth (ticket 20), and ADR 0015
has closed the column's visual vocabulary at four marks.

Ticket 21 was resolved by prototype ([`prototypes/21-types/`](../../.scratch/dot-tower/prototypes/21-types/)):
four design directions — silhouette-only, colour-only, glyph tags, hybrid —
each under five colour-vision simulations and an authored-resolution ladder,
rendered in the closed column vocabulary. Verdicts returned on the prototype
itself: the hybrid, all four questions as recommended.

## Decision

1. **A climber's type is carried by silhouette and hue together** — each type
   owns a distinct shape (melee's shield slab, ranged's bow column, healer's
   hat and staff) *and* a colour-blind-safe hue (the study's palette: melee
   blue, ranged ember-orange, healer pale with a gold staff). Colour alone is
   dead by construction — identical bodies in red/green collapse under
   deuteranopia; glyph tags above every head are redundant once shape and hue
   both carry type.
2. **The healer alone carries a tag** — a small gold cross above its head —
   the one type ticket 15 measured as having marginal value, legible from
   across the column and still visible at 1× authored pixels. It is sprite
   anatomy, not column furniture: ADR 0015's four-mark closure governs the
   column and holds unamended.
3. **A climber's state never renders.** Per-unit health pips were prototyped
   deliberately, to be caught failing: at phone scale they read as noise.
   Ticket 05's deaths-in-aggregate at the wall is now the individual's entire
   state surface.
4. **Sprites are authored at 16×16; the 8×8 hand-authored grid is the
   demonstrated readability floor** (each type still reads, and the healer's
   tag survives, at 1×). Floor height is 48px — 13 floors in view, inside
   ticket 10's 12–14-floor window.

## Consequences

- The art-pipeline fog item inherits fixed constraints: authored grid sizes,
  floor height, and a colour-blind-safe palette. The ranged silhouette is the
  art pass's first target — verification flagged it reading as a pillar rather
  than a figure at column scale (it is ADR 0010's "arrow as sprite flavour"
  delivered, and it distinguishes, but it does not read as a body). Melee's
  shield trim can parse as a companion sprite in dense clusters — an art
  problem, not a structural one.
- The palette is colour-blind-safe by construction — shape carries type
  redundantly with hue. The prototype's simulations stay in the tree as the
  check the art pass re-runs against final assets.
- Ticket 10's layout question is answered at 48px floors; the column window,
  the lock interval, and the sprite budget now agree on one number.
- Nothing per-climber ever appears above, below, or beside a sprite. If state
  ever needs to show, the wall's aggregate — not the sprite — is where it goes.
