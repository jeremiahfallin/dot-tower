// PROTOTYPE — throwaway. Facts for ticket 15 "Climber composition and the per-type caps".
// Not a decision — the measurements the grilling needs so its questions are about design
// rather than about arithmetic anyone could have run.
//
// Every run is pinned to identical wall-clock time with the stall detector off. Ticket 19
// established that this is the only comparison this model can be trusted for.
// Run: node composition.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  return c;
};
const go = (mutate, min = 40, M = 1) => {
  const c = base(); mutate(c);
  return simulateRun(c, { maxSeconds: min * 60, prestigeMult: M });
};
const row = (name, r, ref, extra = '') => {
  const d = ref ? ((r.peak / ref.peak - 1) * 100) : 0;
  console.log('  ' + name.padEnd(38) + String(r.peak).padStart(5) +
    (ref ? (d >= 0 ? '+' : '') + d.toFixed(1) + '%' : '    —').padStart(9) +
    r.goldEarned.toExponential(2).padStart(11) +
    `  ranks ${r.ranks.melee}/${r.ranks.ranged}/${r.ranks.healer}`.padEnd(20) + extra);
};

// --- A. Does each type earn its place at all? ---
console.log('\n### A. Marginal value of each type (cap -> 0 removes it entirely)');
const b = go(() => {});
row('baseline  caps 40/30/20', b, null);
row('no melee', go((c) => { c.types.melee.cap = 0; }), b);
row('no ranged', go((c) => { c.types.ranged.cap = 0; }), b);
row('no healer', go((c) => { c.types.healer.cap = 0; }), b);

// --- B. Is there an interior optimum? Fixed population budget of 90, split three ways. ---
console.log('\n### B. Fixed budget of 90 climbers, split three ways — is any split better?');
const splits = [
  [40, 30, 20], [60, 20, 10], [20, 60, 10], [10, 20, 60],
  [30, 30, 30], [70, 10, 10], [10, 70, 10], [10, 10, 70],
  [45, 35, 10], [45, 15, 30], [20, 40, 30],
];
const bud = [];
for (const [m, r_, h] of splits) {
  const res = go((c) => { c.types.melee.cap = m; c.types.ranged.cap = r_; c.types.healer.cap = h; });
  bud.push([`${m}/${r_}/${h}`, res]);
}
bud.sort((x, y) => y[1].peak - x[1].peak);
for (const [name, res] of bud) row(`caps ${name}`, res, bud[bud.length - 1][1]);

// --- C. Does raising every cap simply win? ---
console.log('\n### C. Does more of everything just win? (composition as a spending axis)');
row('baseline 40/30/20 (90 total)', b, b);
for (const k of [1.5, 2, 3, 5]) {
  row(`every cap x${k} (${Math.round(90 * k)} total)`,
      go((c) => { for (const t of ['melee', 'ranged', 'healer']) c.types[t].cap = Math.round(c.types[t].cap * k); }), b);
}

// --- D. Is gold competition a real decision? Spread across spending strategies. ---
console.log('\n### D. Gold competition — spread across spending strategies');
for (const pol of ['cheapest', 'even', 'melee', 'ranged', 'healer', 'noLock']) {
  const r = go((c) => { c.buyPolicy = pol; });
  row(`buy: ${pol}`, r, b, `lock ${r.lockLevel}`);
}

// --- E. Same, at depth. Ticket 19 found effects that flip sign between run 1 and run 6. ---
const warm = simulateCampaign(base(), 5, { maxSeconds: 90 * 60 });
const M5 = warm[4].prestigeMultOut;
console.log(`\n### E. At depth (run 6, M=${M5.toExponential(2)}), 60 minutes`);
const b6 = go(() => {}, 60, M5);
row('baseline 40/30/20', b6, null);
for (const [m, r_, h] of [[70, 10, 10], [10, 70, 10], [10, 10, 70], [30, 30, 30]]) {
  row(`caps ${m}/${r_}/${h}`, go((c) => {
    c.types.melee.cap = m; c.types.ranged.cap = r_; c.types.healer.cap = h;
  }, 60, M5), b6);
}
for (const pol of ['even', 'melee', 'ranged', 'healer']) {
  row(`buy: ${pol}`, go((c) => { c.buyPolicy = pol; }, 60, M5), b6);
}
