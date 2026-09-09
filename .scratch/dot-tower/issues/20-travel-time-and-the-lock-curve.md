# Travel time and the lock curve

Type: prototype
Status: resolved
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

## Comments

### The measured case for the fix is mostly a withdrawn mechanic (session of 2026-09-09)

Not a resolution — the five questions stand. But the instrument this ticket
names moved, and moving it turned up something that changes what its headline
number means.

**The harness is now Rust.** `crates/sim/` is the shipping simulation, and it is
also what this ticket should be measured on. It is verified **tick for tick**
against `prototypes/06-progression-curve/model.mjs` — 1,974 consecutive samples
identical on peak, lock line, crowd, ranks, hero level and gold, on both lock
curves, and a six-run campaign reproducing the model's peaks exactly
(133/184/254/343/448/569 at `lock_cost_base` 4.0; 138/206/322/505/738/998 at
2.5). Reproduce with `sweeps/model-parity.ron`.

Getting there needed one fix that was squarely the port's fault — a sampling
window that fired two ticks early and labelled every sample one tick late, enough
to make a trace disagree for no real reason. It also needed the curves to use
`powf` rather than `powi`: repeated squaring disagrees with `Math.pow` on 53 of
the first 60 floors and its error compounds with depth, which matters on its own
account for a game whose engine is 1.075 against 1.095 over hundreds of floors.

**The finding: the model pays sealed-floor income, and the design does not.**
Ticket 06 withdrew frozen income from sealed floors after measuring a sealed band
at 0.04% of the money — locking is a spawn-point and travel-time mechanic, not an
economic pillar. `model.mjs` still implements it: `doLock` measures the band and
`sealedRate` pays every tick thereafter, scaled by prestige. So every number this
ticket quotes includes an income source the game will not have.

On the **shipped** curve that does not matter, and ticket 06's finding stands
exactly: at most two floors in a run, under 0.5% by the end of a six-run
campaign. On the **proposed** curve it dominates.

| run 6, 60 min pinned | with the withdrawn income | without it |
|---|---|---|
| `lock_cost_base` 4.0 (shipped) | 569 | 568 |
| `lock_cost_base` 2.5 (proposed) | **998** | **627** |

The mechanism is the obvious one once seen: at 2.5 the run buys ~98 locks instead
of ~11, and every lock freezes a rate that then pays for the rest of the run. The
fix was not buying travel time so much as buying a passive income stream.

**So the honest comparison between the two curves is 568 → 627 — about +10%, not
the +50% to +75% the ticket opens with.** That does not clear the shipped curve:
lock cost still outruns income by ×1.614 per lock, the lock line still falls
permanently behind the wall, and travel time still diverges. The *arithmetic*
argument in the question is untouched. What is weaker than recorded is the
measured case that dropping `lock_cost_base` is the fix, and question 3 — *is
travel time supposed to be a mechanic at all?* — is now carrying more of the
ticket than it was.

Question 2 is unaffected, and if anything sharper. With the withdrawn income
taken out of both sides, run 6 at 60 minutes gives **3 climbers sitting on the
frontier** under the proposed curve, against **80 trailing an average of 52
floors** under the shipped one. Both failures the question describes are real,
still opposite, and neither is "an endless stream of climbers".

**One methodological note for whoever takes this.** The numbers in the question
(baseline 749, fixed 1129) were taken with runs ending on the stall heuristic.
Ticket 19 established that the heuristic cannot be compared across variants,
because a variant that grinds rather than stalls buys depth from run length
alone. Every number above is pinned to identical wall-clock time; the Rust
harness has no stall detector at all, deliberately, and ticket 18 still owns what
the shipped game will use.

## Answer

**The lock curve is a relationship, not a number: `lock_cost_base == gold_base ^ 10`, so a lock
costs a constant number of kills at the depth it seals, forever.** Recorded as
[ADR 0013](../../../docs/adr/0013-a-lock-costs-a-constant-number-of-kills.md). The second half of
the question — what the stream is supposed to look like — turns out to be a separate dial that
costs almost nothing, and the two were tangled because only one constant was being varied.

Measured on [`crates/sim/`](../../../crates/sim) rather than the JS model, wall-clock pinned, with
`sealed_income` off. Readings and method in [`prototypes/20-lock-curve/`](../prototypes/20-lock-curve/).

### 1. What relationship should lock cost hold to income?

**Exactly matching, and the ticket's own objection to that is answered by measurement.** The
question worried that if lock cost merely matches income, locking becomes automatic — bought the
moment it is affordable, with no decision in it. It does not, because gold is contested with
climber ranks: at k = 1.00 a lock waits **41 seconds** between becoming permitted and becoming
affordable, and **not one lock in a six-run campaign** arrives already affordable.

Reading that at all needed a new buy policy. Under buy-cheapest — which is what every recorded
number in this project used — a lock priced above a rank is passed over until ranks grow past it,
so the gate is rank cost and affordability can never be observed. `BuyPolicy::LockFirst` exists to
ask the question the ticket actually asked.

The transition is sharp on both sides, and the full table is in the ADR. Above 1.00 the lock line
falls behind and the per-type ceiling starts setting the crowd size; below ~0.90, nine locks in ten
are bought on sight.

### 2. How big is the crowd, and how far does it walk?

**Set by `lock_cost0`, and nearly free.** At k = 1.00, sweeping it 125 → 4000:

| `lock_cost0` | kills per lock | peak | crowd | transit | back |
|---|---|---|---|---|---|
| 125 | 18 | 643 | 6 | 10 | 2 |
| 500 | 74 | 631 | 7 | 12 | 2 |
| **1000** | **147** | **634** | **18** | **16** | **7** |
| 2000 | 295 | 627 | 46 | 22 | 11 |
| 4000 | 590 | 616 | 59 | 29 | 15 |

A 32× price range moves the crowd 6 → 59 and peak floor 4%. So the question's framing — a long walk
with a big crowd *against* no walk and almost no crowd — was a false dichotomy created by varying
one constant. They are separate knobs.

**Landed at `lock_cost0` 1000**: ~18 climbers trailing the frontier by ~7 floors over a ~16-floor
transit, which fits [ticket 10](10-portrait-layout-desktop-frame.md)'s 12–14 row tower column with
the stream mostly on screen. This integer is **authoring, not arithmetic** — 2000 doubles the crowd
for 1% of peak floor but pushes the transit to 22 floors, past what the column shows. Whoever does
the art pass owns that trade, and it is cheap to make.

The mix at depth is now the authored 4/3/2 rather than the ceiling's 40/30/20, which is
[ADR 0011](../../../docs/adr/0011-composition-is-authored.md) working for the first time.

### 3. Is travel time supposed to be a mechanic at all?

**The dichotomy is false, and the measurement says so plainly: the stream is 78–96% walking in
every configuration tested**, from a 10-floor transit to a 205-floor one. Travel is not something
the lock curve introduces and it cannot be removed — it is what a stream that spawns at a line and
fights at a frontier *is*.

And it is not economic. Across that same 20× swing in transit, peak floor moves 17% (651 → 558) and
moves *against* travel, so a longer walk is not a trade that buys anything either. What travel time
sets is what the tower looks like: how many climbers are on screen and how spread out they are.

**So travel time is a legibility parameter.** On this project that is a promotion, not a demotion —
legibility has veto power — but it means the lock curve is answerable to the tower column and to
ticket 21, not to the economy. The lock line should track the wall about as closely as k = 1.00
makes it, and the crowd should be priced to fill the column.

### 4. What does this do to run length?

**Nothing, and the premise was wrong.** The question states that at depth the run ends when locks
become unaffordable, so the lock curve *is* the run-length curve. It is not. Over four hours at a
mature account the floors-gained-per-10-minutes decays on the same curve at k = 1.00, 1.21 and
1.61, reaching 697 / 662 / 641 — different depths, identical shape. There is no cliff where locks
stop being affordable; there is a smooth asymptote.

```
k=1.00, 4 hours:  272 221  83  24  18  12  11   7   6   6   3   5 ...  2   1   3   1
```

**~90% of a run's depth arrives in the first hour**, and the remaining three hours add ~10%.

Two consequences outside this ticket. **[Ticket 17](17-run-length-growth.md) is no longer
downstream of this one** and can be taken immediately. And for
**[ticket 18](18-run-is-over.md)**: there is no stall *event* to detect. The climb decays smoothly
and never quite reaches zero, so any "has the run ended" signal is choosing a threshold on a
continuous curve rather than noticing a discontinuity. That is a harder problem than ticket 18
currently assumes, and it is the reason its question 1 matters more than its question 2.

### 5. Does the prestige curve survive the fix?

**Yes, untouched.** Head start sits at **24–27% of peak floor across the entire sweep** — every
lock curve, every price, every margin. [Ticket 08](08-prestige-before-after.md)'s ~26% is not a
property of the lock curve at all, which is a stronger result than it had before.

Ticket 06 put floor 1,000 at around run 10; under the landed curve it arrives in **run 9**
(140, 194, 271, 371, 489, 630, 790, 954, 1117, 1258). Close enough that ticket 06's shape stands.

### What moved

`tuning.ron`: `lock_cost_base` 4.0 → 2.4782276135 (`= 1.095^10`), `lock_cost0` 500 → 1000.
`lock_margin` stays at 15 — it also grows the crowd, but by making 91% of locks arrive already
affordable, which buys the crowd by destroying the decision.

Before and after, six runs of 60 minutes:

| | peak (run 6) | crowd | back | ceiling binds at depth |
|---|---|---|---|---|
| before (k = 1.614) | 568 | 88 | 82 floors | yes, 2147 skips |
| after (k = 1.000) | 630 | 19 | 7 floors | no, 0 skips from run 2 |

### Left open

- **The `composition` integers and the cap ceilings**, which the map's fog parked here. This ticket
  says how big the crowd is (~18 at depth, set by `lock_cost0`) but not what 4/3/2 should be. That
  is authoring and it now waits on ticket 21 and the art pass, not on arithmetic.
- **Run 1 still grazes the ceiling** — 40 skipped replacement slots in its final third, against
  zero in every later run. A fresh account with no prestige climbs slowly enough for the stream to
  fill. Not worth a lever; worth knowing before someone reads a run-1 trace and reopens ADR 0011.

Status: resolved
