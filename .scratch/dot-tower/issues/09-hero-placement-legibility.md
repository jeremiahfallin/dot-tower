# How the player reads the hero

Type: prototype
Status: resolved
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

## Prototype

Built 2026-08-20. Throwaway, per `prototype` (UI branch, sub-shape B — a Bevy game has no
existing route to host variants).

- **Flip through it**: https://claude.ai/code/artifact/4e6ad1f3-15e0-4229-8b10-bb55bb3d2252
  Variants at `?variant=A|B|C`, switchable from the floating bar or the ← → keys.
- **Source**: `.scratch/dot-tower/prototypes/09-hero-legibility/hero-hud.html` — one self-contained
  file, open it directly.

Three structurally different answers, running **live** in a 390×790 portrait frame, because uptime
and "was that the right moment to press" cannot be judged from a still image. The tower, the
climbers and the ability bar are identical across all three and are deliberately not the subject —
ticket 10 owns layout. A throwaway stub sim drives the motion; it is not ticket 06's model.

- **A — Instrument panel.** A docked readout above the ability bar: uptime as a bar, aura as a
  multiplier, experience as a fill, the wall as a floor number. One place to look, tower stays
  clean. Risk: a wall of numbers disconnected from the thing they describe.
- **B — Read it off the tower.** No HUD. The aura is a lit floor, uptime is a ring draining on the
  hero itself with a respawn countdown in its place, experience fills the hero from the bottom, the
  wall is hatched where climbers die. Risk: nothing is a number, so slow trends are invisible.
- **C — The ledger.** The recent past rather than the present: a 45-second uptime sparkline, every
  ability press marked green or red on a timeline, and a plain-language verdict on the last one.
  Risk: most screen space, most to learn, says nothing about right now.

The four tank abilities are stubbed with real good/bad conditions so ticket 16's design test —
*is there a wrong moment to press it?* — is demonstrable rather than theoretical. Pressing Bulwark
at 93% health reports **"✗ BULWARK — WASTED — YOU WERE NEAR FULL HEALTH"**.

Awaiting a pick. The useful answer is usually "the uptime treatment from C with the aura from B".

## Answer

**Variant D — the tower says where and now; the strip says how it has been going.**

Chosen from the prototype as C's uptime treatment combined with B's aura. The two halves divide on
a principle rather than on taste, and the principle is the part worth keeping:

- **Everything spatial lives on the tower**, where the player is already looking. The aura is a lit
  floor with its multiplier called out beside it. The hero carries a ring that drains with its
  health, replaced by a respawn countdown when it is down. Experience fills the hero from the
  bottom. The wall is hatched where climbers are dying.
- **Everything temporal lives in one strip** above the ability bar: a 45-second holding sparkline,
  every ability press marked green or red on a timeline, and a plain-language verdict on the last
  one.
- **Nothing is duplicated between them.** The ring says the hero is at 30% health *this second*;
  the sparkline says it has been dying *all minute*. Those are different questions, and neither
  treatment answers both — which is why the winning design is a combination rather than either
  variant alone.

Rejected: **A (instrument panel)** states everything as numbers in one docked place, but severs
each number from the thing it describes — it reads "uptime 74%" while the hero it refers to may be
off-screen. **B alone** puts everything on the tower but makes every value instantaneous, so slow
trends vanish; you cannot tell 74% uptime from 85% by watching a ring flicker. **C alone** answers
the trend questions well and says nothing about right now.

### What the mock surfaced

**The aura row and the wall marker are the same row.** Ticket 16 established the hero belongs at
the wall, so in the common case a lit blue aura, red wall hatching, the gold hero, its ring and a
callout all stack on one 30px band. These cannot be designed as independent decorations that happen
to coexist — they are the default state, not an edge case, and the implementation needs one
combined treatment for "the hero is standing at the wall".

**The wrong-moment test is satisfiable.** The four tank abilities were stubbed with real good/bad
conditions, and the verdict line reports things like *"✗ WAR CRY — WASTED — BARELY ANYONE WAS
STANDING IN THE AURA"*. Ticket 16 set the design test — is there a wrong moment to press it? — and
the prototype shows the feedback can carry that answer in one line of plain language. All four
tank abilities passed the test; that is a bar the remaining roster's abilities must also clear.

### Items retired from the original question

Items 1, 2 and 5 are moot, per ticket 16. There is nothing to compare (stationing at the wall
dominates), so no predictive-versus-observed choice arises, and an auto-place button would suggest
the only placement there is. Item 3 — indicating the wall — stands and is answered above. Item 4,
the interaction for choosing a floor, is deliberately **not** answered here: placement is now a
learnable rule rather than a decision the player re-evaluates, so the interaction can be as plain as
tapping a floor. Ticket 10 owns the layout it sits in.

The ticket has been retitled: it was "Making the hero placement tradeoff visible", which named a
tradeoff ticket 16 proved does not exist.
