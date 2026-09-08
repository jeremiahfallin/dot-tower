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

## Answer

Worked before 17, per this ticket's own question 4: 17 cannot be trusted until the lock curve
is settled, because today the lock curve *is* the run-length curve. Decision recorded in
[ADR 0013](../../../docs/adr/0013-locks-are-priced-in-time.md); evidence in
[`prototypes/20-lock-curve/`](../prototypes/20-lock-curve/) (README walks the arc).

### The excavation that had to come first

Every "fixed lock curve" reading before this ticket — including the 1129/1220 peaks and the
9-climb stream quoted in the Question above — was riding an artifact: the model had been paying
gold from sealed floors all along. Ticket 06's withdrawal ("frozen floors emit nothing") was
never implemented. It measured 0.0% of income under the shipped curve, which is why it hid, but
under *any* fixed curve it is 99.3% — 119 seals each paying their frozen 60-second snapshot
forever. Ticket 15's conclusions stand (composition is authored under both curves; the caps
genuinely bind under the shipped one), but its fixed-curve *numbers* are artifacts. Withdrawal
is now implemented (`sealedIncome: false`, default).

With sealed income out, depth is curve-independent: run-6 pinned-60min peak is 746–789 under
every curve tried. Locks never bought depth. They only ever shaped the stream.

### Q1 — the relationship

No geometric base holds a constant relationship to income, because **income compounds faster
than per-kill gold**: kill rate grows with ranks, so `goldBase^10 = 2.478` per lock interval
under-states income growth. Measured with withdrawal in force: base 2.478 under-matches (locks
fall to 4% of income at a 22s cadence, stream collapses onto the wall, pop 11); base 3.0
over-matches (run-6 walk 109, rising monotonically to 230 at the shipped 4.0); the apparent
match at 2.7 moved when sealed income came out and would move again on any rank retune. A
geometric match is tuned-then-rot.

**Price locks in time: K seconds of rolling-60s income** (the same window offline gold's best
rate uses). Matched by construction — it cannot diverge on retune or fall to zero as depth
compounds. K = 30s is the pick:

| K | run 1 | run 6 |
|---|---|---|
| 15s | walk 7.1, pop 17, lock share 3% — trivial | walk 10.9, pop 13 |
| **30s** | **walk 24.7, pop 57, 13 locks @ 35s, share 17% — a real decision** | **walk 12.5, pop 15, skips 0** |
| 60s | walk 102, 3 locks @ 448s — run 1 cannot afford them | walk 42 |
| 120s | walk 115, 0 locks — broken | walk 59 |

And one more rule: **locks below the account's best-ever floor are free.** Head start is
conquered territory; re-locking it is bookkeeping, not a decision. Without this, a mature
account's blitz outruns a 20-floors/min lock line and the road balloons to 268 floors mid-run
(run 11, M = 4.2×10¹³); with it, walk 17.4, pop 53. The gold price lives at the frontier. The
save already holds deepest-floor-ever in `Account`.

### Q2 — the crowd and the walk

The two numbers decouple once the lock line is bounded: **walk is ~K and account age; crowd is
replacement and climb speed.**

- Walk: 11–25 floors across the ten-run campaign at K = 30 (25 early, 12.5 at run 6, 17 at run
  11) — inside ticket 10's column window, sized 12–14, give or take the early run.
- Replacement is the clean crowd dial: pop 7/15/35 as replacement goes 3.33/1.67/0.83s, with
  walk flat ~13 and peak flat. ADR 0011's dial works under the fixed curve — the inertness
  ticket 14 measured was the broken curve's, not the dial's.
- Climb speed dials pop 61/15/10 (quarter/half/full speed) at ~12% depth cost at the far end.
- The integers for the map's fog: at replacement 1.67s, crowd runs **~57 in a fresh account,
  15–53 at depth by account age**; caps 40/30/20 stay generous — zero cap skips at the shipped
  replacement, mild (1.2/10s) only if replacement is pushed to 0.83s; composition ratio 4/3/2
  reproduces (26/19/13 early, 7/5/3 at run 6, exactly 24/18/12 at run 11).

### Q3 — is travel time a mechanic?

Yes, and locking is its pump. The road is the stream's visible shape, the lock purchase is the
spatial decision — the surge is visible feedback — and its stakes are shape, not depth: never
locking (`noLock`) costs 8–9% of peak (145→133 run 1, 776→703 run 6) but explodes the road to
607 floors at run 6 and drives the population flat onto the caps (40/30/20 alive exactly,
skips 5/10s). A purchase a player evaluates by eye, with forgiving stakes, bounded to the
column window. That is the decision content of locking; there is no interesting lock-vs-rank
arithmetic to tune it for, and none is needed.

### Q4 — run length: handed to 17, sharpened

Under the shipped curve, runs ended because locks became unaffordable — the lock curve *was*
the run-length curve. Priced in time, runs 1–6 of a campaign still end naturally (40/35/39,
then 125/146/169 min — lengthening), but **runs 7–10 are still climbing at a 180-minute cap**
(cumulative M 4.6×10¹³ by run 10; peak 1849, brushing the model's f64 limit). The natural
stall recedes past what the model can see. What ends a mature run is now unowned —
prestige-while-climbing must become the norm (ticket 08 already prices prestiging at the stall
270× over one more floor), or replacement rises to bring stalls back. Ticket 17's brief, noted
on that ticket.

### Q5 — the prestige curve survives

Head start holds at **16% of peak in run 1 and 24–28% from run 2 through run 10** (ticket 08's
~26% stands, slightly tighter); floor 1,000 arrives at **run 7** (shipped curve: run 8; ticket
06 promised ~10 and is now conservative). No retune forced downstream.

### Model changes

Flag-gated in `06-progression-curve/model.mjs`, defaults byte-identical — verified by diffing
`19-ranged-reach/reach.mjs` and `15-composition/composition.mjs` outputs before and after:
`lockPricing: 'time' | 'geometric'` (default geometric), `lockTimeCost: 30`,
`lockFreeBelowBest: false` (default), `sealedIncome: true` (default — withdrawal is opt-in so
every prior ticket's reproduction stays exact). Probes must pass `bestEverFloor` explicitly;
the free band only exists relative to an account history.
