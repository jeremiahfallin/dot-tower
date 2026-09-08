// PROTOTYPE — throwaway. Ticket 20 diagnostic: where does run-6 income actually
// come from, and what do time-priced locks really cost? The K sweep showed lock
// share 0% with ~119 locks bought every ~20s at EVERY K — impossible if each
// lock costs K seconds of income. Suspicion: sealed-floor income (which
// CONTEXT.md says should not exist) dominates at depth and prices are computed
// against kill income only.
// Run: node diagnose.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone, rankCost, heroCost } from '../06-progression-curve/model.mjs';

const base = (mutate) => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  c.lockPricing = 'time'; c.lockTimeCost = 30;
  if (mutate) mutate(c);
  return c;
};

const warm = simulateCampaign(base(), 5, { maxSeconds: 90 * 60 });
const M5 = warm[4].prestigeMultOut;
const r = simulateRun(base(), { maxSeconds: 60 * 60, prestigeMult: M5 });

// sealed income: integrate sealedRate * M over time via trapezoid on samples
let sealed = 0;
for (let i = 1; i < r.samples.length; i++) {
  const a = r.samples[i - 1], b = r.samples[i];
  sealed += ((a.sealedRate + b.sealedRate) / 2) * M5 * (b.t - a.t);
}
const locks = r.events.filter((e) => e.kind === 'lock');
const lockSpend = locks.reduce((a, e) => a + e.cost, 0);
console.log(`run 6, K=30s, M=${M5.toExponential(1)}`);
console.log(`  goldEarned      ${r.goldEarned.toExponential(3)}`);
console.log(`  of which sealed ${sealed.toExponential(3)}  (${(100 * sealed / r.goldEarned).toFixed(1)}%)`);
console.log(`  of which kills  ${(r.goldEarned - sealed).toExponential(3)}`);
console.log(`  lock events     ${locks.length}, total spend ${lockSpend.toExponential(3)} (${(100 * lockSpend / r.goldEarned).toFixed(2)}% of earned)`);
for (const e of [locks[0], locks[Math.floor(locks.length / 2)], locks[locks.length - 1]]) {
  console.log(`    t=${e.t.toFixed(0).padStart(5)}s floor ${String(e.floor).padStart(5)} cost ${e.cost.toExponential(2)}  (rate then: ${(e.cost / 30).toExponential(2)}/s)`);
}
// kill income rate at the end, for comparison with the last lock's implied rate
const tail = r.samples.slice(-10);
const killRate = (r.goldEarned - sealed) / r.seconds;
console.log(`  mean kill rate  ${killRate.toExponential(2)}/s;  final sealedRate ${r.samples[r.samples.length - 1].sealedRate.toExponential(2)}/s (xM=${M5.toExponential(1)})`);

// Same question under the SHIPPED broken curve (B=4.0, geometric): how much of
// the readings tickets 06-16 rest on was sealed income? If small there, those
// conclusions survive the withdrawal approximately; if large, they shift too.
const base4 = (() => { const c = clone(DEFAULTS); c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall'; c.stallMinutes = 1e9; return c; })();
const warm4 = simulateCampaign(base4, 5, { maxSeconds: 90 * 60 });
const r4 = simulateRun(base4, { maxSeconds: 60 * 60, prestigeMult: warm4[4].prestigeMultOut });
let sealed4 = 0;
for (let i = 1; i < r4.samples.length; i++) {
  const a = r4.samples[i - 1], b = r4.samples[i];
  sealed4 += ((a.sealedRate + b.sealedRate) / 2) * warm4[4].prestigeMultOut * (b.t - a.t);
}
console.log(`\nshipped curve (B=4.0) run 6: peak ${r4.peak}, ${r4.lockLevel} locks`);
console.log(`  goldEarned ${r4.goldEarned.toExponential(3)}, sealed ${(100 * sealed4 / r4.goldEarned).toFixed(1)}%`);
