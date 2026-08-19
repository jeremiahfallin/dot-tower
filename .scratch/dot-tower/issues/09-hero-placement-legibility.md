# Making the hero placement tradeoff visible

Type: prototype
Status: open
Blocked by: 03, 06

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
