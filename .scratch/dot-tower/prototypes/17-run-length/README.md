# 17 — run-length growth

PROTOTYPE — throwaway. Answers
[ticket 17](../../issues/17-run-length-growth.md); decision recorded in
[ADR 0014](../../../../docs/adr/0014-the-multiplier-pays-for-new-territory.md).

**Verdict:** the shipped earned-multiplier rule (a function of the run's peak
floor) is a prestige-spam engine — measured, cycling every 5 minutes compounds
the account to M = 4.2×10¹¹⁰ in 12 hours without fighting a new floor, because
the multiplier pays for re-conquered territory. The fix: **the multiplier pays
for peak beyond the account's best-ever floor** (`prestigeBasis: 'beyondBest'`
in the model, default unchanged and byte-identical). Spam then earns literally
nothing, and run length becomes structural: early runs are pure frontier
(~40 min), mature runs spend most of their length re-conquering a head start
that grows with the account, so runs lengthen to 2.5–3 h and the optimal
prestige interval drifts upward with maturity.

## Scripts

1. `chop.mjs` (`chop-readings.txt`) — the decisive sweep. From a mature state,
   12 h of pinned wall-clock under eleven prestige intervals, both bases side
   by side. Under `peak`: 5-min cycling wins by 8× (ln M/h 21.2 vs 2.6 at
   stall). Under `beyondBest`: every interval below the re-conquest time banks
   *nothing* (M frozen), and the optimum is the shortest interval that still
   reaches new territory.
2. `maturity.mjs` (`maturity-readings.txt`) — the original question. Warms 6-
   and 12-run accounts under `beyondBest`, then finer interval sweeps at each:
   the optimum sits at 60–70 min at best-ever 412 and 70–80 min at 771 — it
   drifts up as the re-conquest walk lengthens.
3. `long.mjs` (`long-readings.txt`) — the dead-end check ticket 06's `bestPeak`
   mode failed. Twenty runs under `beyondBest`: new territory holds at 48–76
   floors/run, peak 1300 at run 20, floor 1,000 at run 16, head start ~21% of
   peak, earned ×2.2–3.0 per run throughout. Increments compound; no
   dead-end.

## Notes

- The re-conquest cliff is real and sharp: at 12-run maturity, a 60-minute run
  never passes best-ever 771 and earns ×1.00; a 70-minute one does. The cliff
  sits at the re-conquest walk + rank-rebuild time and drifts up ~10 min per
  six runs of maturity.
- The stall heuristic (±40%) is not load-bearing anywhere here: comparisons
  are pinned wall-clock policy sweeps, ticket 19's trusted pattern. Natural
  stall lengths are reported as texture only.
- f64 is comfortable: M stays ≤ 4×10⁸ through run 20 under `beyondBest`.
