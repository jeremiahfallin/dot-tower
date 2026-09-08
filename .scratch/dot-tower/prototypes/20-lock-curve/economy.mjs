// PROTOTYPE — throwaway. Ticket 20, Q1: is locking a decision or a reflex?
//
// Reconstructs where the run's gold went from the end state alone (ranks and hero
// costs are closed-form over their final levels; lock events carry their floor).
// The identity earned == rank + hero + lock + leftover is printed as a check.
//
// Run: node economy.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone, rankCost, heroCost, lockCost } from '../06-progression-curve/model.mjs';

const MATCH = Math.pow(DEFAULTS.goldBase, 10);

const base = (B, mutate) => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  c.lockCostBase = B;
  if (mutate) mutate(c);
  return c;
};

function breakdown(c, M, min) {
  const r = simulateRun(c, { maxSeconds: min * 60, prestigeMult: M });
  let rank = 0;
  for (const ty of ['melee', 'ranged', 'healer'])
    for (let i = 1; i < r.ranks[ty]; i++) rank += rankCost(c, ty, i);
  let hero = 0;
  for (let l = 1; l < r.heroLevel; l++) hero += heroCost(c, l);
  const lock = r.events.filter((e) => e.kind === 'lock').reduce((a, e) => a + lockCost(c, e.floor / 10), 0);
  const id = r.goldEarned - (rank + hero + lock + r.gold);
  const pct = (x) => ((100 * x) / r.goldEarned).toFixed(1).padStart(5) + '%';
  console.log(`  peak ${String(r.peak).padStart(5)}  ranks ${r.ranks.melee}/${r.ranks.ranged}/${r.ranks.healer} hero ${String(r.heroLevel).padStart(3)}` +
    `  locks ${String(r.lockLevel).padStart(3)}  | gold -> rank ${pct(rank)} hero ${pct(hero)} lock ${pct(lock)} POOL ${pct(r.gold)}` +
    `  (identity residual ${id.toExponential(1)})`);
  return r;
}

for (const B of [MATCH, 2.7, 4.0]) {
  const warm = simulateCampaign(base(B), 5, { maxSeconds: 90 * 60 });
  const M5 = warm[4].prestigeMultOut;
  console.log(`\n### lockCostBase ${B === MATCH ? B.toFixed(3) + ' (matched)' : B}`);
  console.log('  run 1:'); breakdown(base(B), 1, 40);
  console.log('  run 6:'); breakdown(base(B), M5, 60);
}
