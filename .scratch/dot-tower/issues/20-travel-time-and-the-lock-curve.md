# Travel time and the lock curve

Type: prototype
Status: open
Blocked by: 06

## Question

At depth the game is **travel time, not combat**. What should the lock curve be, and what is the
stream supposed to look like once it is fixed?

Raised by ticket 15, which could not explain why composition was inert at depth until it measured
what actually moves the peak floor there. Nothing about the climbers does:

| run 6, 60 minutes | peak floor |
|---|---|
| baseline | 749 |
| ×2 all climber damage | 749 |
| **×10 all climber damage** | **749** |
| ×3 all type caps | 756 |
| replacement 5s → 2s | 749 |
| replacement 5s → 15s | 748 |
| **lock cost base 4.0 → 2.5** | **1129** |

The cause is arithmetic and structural rather than a tuning wobble:

```
lock cost grows      x4.00 per 10 floors   (lockCostBase)
gold per kill grows  x2.48 per 10 floors   (goldBase 1.095 ^ 10)
=> lock cost outruns income by x1.614 per lock, compounding
```

Lock 1 costs 74 kills at its depth. Lock 47 costs 2.7×10¹¹. Locks become geometrically
unaffordable, the lock line falls permanently behind the wall, and travel time grows without
bound. At depth climbers walk **278 floors** from the lock line to the wall and sit on average
108 floors behind it — so the units that reach the front are a trickle, and making that trickle
stronger changes nothing.

**The obvious fix is not obviously right.** Dropping `lockCostBase` to 2.5 reaches floor 1129
instead of 749, but it collapses the stream: 88 climbers strung over 108 floors becomes **9
climbers, all within 0.7 floors of the wall**, and the per-type caps stop binding entirely
(40/30/20 with 3/3/3 alive). Run 1 goes from 80 climbers over 17 floors to 42 over 4. The shipped
curve and the provisional fix are opposite failures — a long walk with a big crowd, against no
walk and almost no crowd — and neither is "an endless stream of climbers".

Resolve:

1. **What relationship should lock cost hold to income?** It cannot outrun it, or travel time
   diverges. If it merely matches it, does locking become automatic — a purchase with no decision
   in it, bought the moment it is affordable?
2. **How big is the crowd meant to be, and how far does it walk?** These are the same number:
   population is spawn rate times lifetime, and lifetime is mostly walking. Ticket 10 sized the
   tower column at 12–14 floors of interesting band on the assumption climbers pile at the wall.
   Nine climbers on one floor is a different picture, and 88 strung over 108 floors is another.
3. **Is travel time supposed to be a mechanic at all?** Ticket 03 called locking "a spawn-point
   and travel-time mechanic". It is currently the *only* mechanic at depth. Either that is the
   game — an idle logistics curve, which is a defensible thing to be — or travel is overhead and
   the lock line should track the wall closely.
4. **What does this do to run length?** Ticket 17 owns run length and cannot be trusted until this
   is settled: at depth the run ends when locks become unaffordable, so the lock curve *is* the
   run-length curve today.
5. **Does the prestige curve survive the fix?** Ticket 08 measured head start at ~26% of peak
   floor and ticket 06 put floor 1,000 at around run 10. With locks fixed, run 6 reaches 1230.
   Both numbers move.

Ticket 15 is resolved and does not wait on this — composition is authored under both lock curves
(0.7% spread at depth *with* the fix). But ticket 15's caps and ratio are left unnumbered in the
map's fog pending question 2, and ticket 17 is downstream of question 4.

The measurement harness already exists: `.scratch/dot-tower/prototypes/15-composition/binding.mjs`
sweeps what moves the peak floor, and `authored.mjs` runs the fixed-lock comparison.
