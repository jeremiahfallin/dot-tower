// PROTOTYPE — throwaway. Ticket 17, follow-ups to chop.mjs under prestigeBasis
// 'beyondBest' (earned multiplier pays only for peak past the account's best).
//
// Q1 (the original question): does the optimal run lengthen as the account
// matures? chop.mjs found the optimum at ~60 min at 6-run maturity -- the
// shortest interval whose runs still reach new territory. If run length is
// mostly re-conquest (walk + rank rebuild through conquered floors), and
// best-ever grows, the optimum should drift upward. This warms two maturities
// (6 and 12 runs) and re-runs a finer interval sweep at each.
//
// Also prints the warm campaigns run-by-run: length, peak, new territory
// (peak - previous best), earned multiplier -- the shape the player sees.
//
// Run: node maturity.mjs
import { DEFAULTS, simulateRun, clone } from '../06-progression-curve/model.mjs';

const design = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.sealedIncome = false;
  c.lockPricing = 'time'; c.lockTimeCost = 30;
  c.lockFreeBelowBest = true;
  c.prestigeBasis = 'beyondBest';
  return c;
};

function warm(n, log) {
  const c = design();
  let M = 1, best = 0;
  for (let i = 0; i < n; i++) {
    const r = simulateRun(c, { prestigeMult: M, maxSeconds: 4 * 3600, bestEverFloor: best });
    const isNew = r.peak - best;
    M *= r.prestigeMultEarned;
    if (log) console.log(`  run ${String(i + 1).padStart(2)}: ${String(Math.round(r.seconds / 60)).padStart(3)} min` +
      `  peak ${String(r.peak).padStart(4)}  new ${String(isNew).padStart(4)}  earned x${r.prestigeMultEarned.toFixed(2)}` +
      `  M ${M.toExponential(2)}  head start ${String(Math.round(Math.log(M) / Math.log(DEFAULTS.enemyHpBase))).padStart(3)}`);
    best = Math.max(best, r.peak);
  }
  return { M, best };
}

function play(M, best, X, budget) {
  let remaining = budget * 60, runs = 0, wall = 0;
  const c = { ...design(), stallMinutes: 1e9 };
  while (remaining > 60) {
    const r = simulateRun(c, { prestigeMult: M, maxSeconds: Math.min(X * 60, remaining), bestEverFloor: best });
    M *= r.prestigeMultEarned;
    best = Math.max(best, r.peak);
    wall += r.seconds; remaining -= r.seconds; runs++;
    if (r.seconds <= 0) break;
  }
  return { runs, wall: wall / 60, M, best, lnM: Math.log(M) };
}

for (const n of [6, 12]) {
  console.log(`\n===== warm campaign, ${n} runs (beyondBest) =====`);
  const w = warm(n, true);
  console.log(`\ninterval sweep from M = ${w.M.toExponential(2)}, best ${w.best} (8h budget):`);
  console.log('  policy          runs   wall    M_cum          ln M/h    frontier/h');
  for (const X of [50, 60, 70, 80, 90, 120, 150, 180, 240]) {
    const p = play(w.M, w.best, X, 8 * 60);
    console.log(`  every ${String(X).padStart(3)} min` +
      String(p.runs).padStart(6) +
      `  ${String(Math.round(p.wall)).padStart(4)}m` +
      `  ${p.M.toExponential(3).padStart(14)}` +
      `  ${(p.lnM / (p.wall / 60)).toFixed(4).padStart(8)}` +
      `  ${((p.best - w.best) / (p.wall / 60)).toFixed(1).padStart(9)}`);
  }
}
