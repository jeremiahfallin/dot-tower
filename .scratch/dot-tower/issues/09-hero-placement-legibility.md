# Making the hero placement tradeoff visible

Type: prototype
Status: open
Blocked by: 03, 06, 16

## Question

How does the player see whether their hero is in the right place?

Hero placement is a real strategic decision, not a fixed answer: stationed below the wall the hero farms gold and escorts climbers upward; stationed at the wall it pushes. The player re-evaluates this constantly. Under the legibility constraint, a decision the player cannot see the consequences of is a decision they are guessing at.

Sketch something reactable and answer:

1. **What does the player compare?** Gold per second at the current station vs a candidate one? Climber survival rate? Effective floor progress per minute?
2. **Is it predictive or observed?** Showing projected outcomes before moving is more useful but risks being wrong; showing measured results after is honest but slow.
3. **How is the wall itself indicated?** The player needs to see where climbers are failing before placement means anything.
4. **What is the interaction?** Drag the hero up and down the tower, tap a floor to assign, or a number input? On a portrait touchscreen with the tower scrolling, this is not obvious.
5. **Does the game ever suggest a placement?** An auto-place button is a legibility aid and a depth-remover at the same time.

Link the sketch from this ticket.

## Added by ticket 05

The hero can now **die and respawn on the floor it is stationed on** after a timer,
so the whole cost of hero death is downtime and the placement throttle is *uptime*.
Whatever makes the tradeoff visible must therefore show survival fraction, not just
position — a hero parked at the wall at 40% uptime is contributing less than the
player will assume from looking at where it stands.

Healers also heal the hero with no special-casing, so hero survival is partly a
function of climber composition passing through its floor. That coupling needs to be
visible too, or the player will read a hero surviving longer as the hero getting
stronger.

**One unhandled case, surfaced while verifying ticket 05 against ticket 03:** what
happens to a hero stationed *below* a lock line that has just risen past it? Sealed
floors hold zero entities, so the hero cannot stay there. The established precedent
is climbers sprinting up to the new entry floor, but the hero's floor is a deliberate
player choice, and silently relocating it changes that choice without telling them.
Options are to block locking past the stationed floor, relocate with an explicit
notice, or have the hero sprint up exactly as climbers do.


## Added by ticket 06

The premise does not currently hold: there is no placement tradeoff to make visible.
Stationed at the wall, below it, or not at all, the peak floor moves by under 2%. Patrol
zones and hero regeneration change nothing. Ticket 16 now has to establish what the hero
contributes before this ticket has a subject.

## Answered by ticket 16 — this ticket now has a subject

The premise is settled, and it is not what this ticket assumed. **There is no placement tradeoff
to visualise.** Stationing at the wall dominates on gold and experience alike, so placement is a
learnable rule — put the hero where the fighting is — not a decision the player re-evaluates.

Item 1 ("what does the player compare?") and item 5 ("does the game ever suggest a placement?")
are therefore moot: there is nothing to compare, and the right placement is always the same.

What replaces them, and what this ticket should now be about:

1. **Uptime.** The aura multiplies only while the hero lives, so a hero at 74% uptime is
   delivering 74% of its contribution. This is the number the player must be able to read, and it
   is invisible without help — the hero looks the same standing there whether it is thriving or
   dying every twenty seconds.
2. **The aura itself.** It applies to one floor, so it wants one highlighted row and a legible
   statement of what it is doing to the climbers standing in it.
3. **Experience.** It accrues only from kills inside the aura, which is a rule the player has to
   learn by seeing it, not by reading it.
4. **Ability moments.** Abilities act on state the aura cannot reach, and the design test is
   whether there is a wrong moment to press each one. If the feedback cannot show a player that
   they pressed one at the wrong moment, the ability has no moment.
5. **Item 3 stands unchanged** — the wall still needs indicating, and it matters more now, since
   "put the hero where the fighting is" is unfollowable if the player cannot see where that is.

**The unhandled lock-line case is resolved**: the hero **sprints up**, exactly as trailing
climbers do under ticket 03. Blocking the lock would make a good purchase conditional on where a
unit stands; and silent relocation is a far smaller sin now that placement is a rule rather than a
strategic choice, particularly since locking moves the hero toward the wall anyway.

**Ticket 05's addendum above is superseded in part**: it says a hero parked at the wall runs at a
survival fraction while one below runs at 100%. Measured, that is backwards without taunt — 98% at
the wall against 71% below, because seventy climbers at the wall soak the damage and a farming hero
stands alone. Taunt is what makes the wall costly, and taunt is a per-hero property.
