# 20 — travel time and the lock curve

PROTOTYPE — throwaway. Answers
[ticket 20](../../issues/20-travel-time-and-the-lock-curve.md); decision recorded
in [ADR 0013](../../../../docs/adr/0013-locks-are-priced-in-time.md).

**Verdict:** no geometric lock curve can hold a constant cost-to-income
relationship (income compounds faster than per-kill gold, because kill rate
grows with ranks). Price locks in **time** — K seconds of rolling-60s income —
and make them **free below the account's best-ever floor** (head start is
conquered territory). K = 30s is the pick. Runs stop ending on their own at
maturity, which is ticket 17's problem now, sharpened.

Model support (all flag-gated, defaults byte-identical — verified against
`19-ranged-reach/reach.mjs` and `15-composition/composition.mjs` before and
after): `lockPricing: 'time'`, `lockTimeCost: 30`, `lockFreeBelowBest: true`,
`sealedIncome: false` (ticket 06's withdrawal, which had never actually been
implemented).

## The arc

1. `walk.mjs` — geometric base sweep with sealed income withdrawn
   (`walk-readings-noseal.txt`): 2.478 = `goldBase^10` under-matches (locks →
   ~4% of income, 20s cadence, stream collapses onto the wall, pop 11);
   3.0 over-matches (walk diverges to 160 at run 6). No stable middle — the
   apparent match at 2.7 in the earlier sealed-era readings
   (`economy-readings.txt`) moved once sealed income came out.
2. `diagnose.mjs` — the sealed-income diagnosis (`diagnose-readings.txt`): under
   the shipped curve sealed floors pay 0.0% of income (why the bug hid), under
   any *fixed* curve 99.3% — 119 seals each paying their frozen 60-second
   snapshot forever. Every "fixed lock curve" reading before this point was
   riding that artifact.
3. `time.mjs` — K sweep (`time-readings-final.txt`): K=15 makes locks trivial
   early (3% share); K=60/120 break run 1 outright (walk 102/115, the account
   can't afford its first locks). K=30: run 1 locks 13 @ 35s cadence at 17% of
   income — a real rank-vs-lock decision; run 6 walk 12.5, pop 15, skips 0.
4. `stream.mjs` — the dials and the stakes (`stream-readings.txt`): replacement
   is the clean crowd dial (pop 7/15/35 as replacement goes 3.33/1.67/0.83s,
   walk flat ~13, peak flat); climbSpeed dials pop 61/15/10 at ~12% depth cost;
   `noLock` costs 8–9% of peak and explodes the road to 607 floors with the caps
   binding — locking is load-bearing for *shape*, not depth. And the
   mature-account check: without free territory, run 11's road balloons to ~268
   floors mid-run because the blitz outruns a 20-floors/min lock line; with it,
   walk 17.4, pop 53.
5. `campaign.mjs` — ten-run campaigns (`campaign-readings.txt`), time-30 vs
   shipped: head start 16% in run 1, 24–28% of peak thereafter (ticket 08's
   ~26% survives); floor 1000 at run 7 (shipped: 8); entities max 157 mean 42
   (shipped: 213/137) — the range ticket 14's parked device session should test
   against. **Runs 7–10 hit the 180-minute cap still climbing** — the lock curve
   no longer ends runs. Ticket 17 inherits.

## Caveats

- Run length rides ticket 06's stall heuristic (±40% noise); the honest
  run-length instrument is ticket 17's job. The "still climbing at cap" readings
  are real though — peak-gain resets track actual climber floors, not just
  lock-line teleports.
- f64 tops out around floor 2000; the campaign model can't see far past run 10
  (peak 1849 at run 10).
- Probes pass `bestEver` from a warm 5-run (or 10-run) campaign — the free
  territory only exists relative to an account history, never in isolation.
