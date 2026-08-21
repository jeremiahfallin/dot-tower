// PROTOTYPE — throwaway. Ticket 15, round 2 facts.
//   1. Composition was judged inert partly because the late game is travel, not combat. Does it
//      become a REAL decision once the lock curve is fixed? If so, "authored" needs revisiting.
//   2. Cap is now a relic-only axis. Does raising ONE type's cap do anything worth a relic?
// Run: node authored.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const base = (fixLocks) => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  if (fixLocks) c.lockCostBase = 2.5;      // the provisional fix, so the late game fights again
  return c;
};
const go = (fixLocks, mutate, min, M) => {
  const c = base(fixLocks); mutate(c);
  return simulateRun(c, { maxSeconds: min * 60, prestigeMult: M });
};

function sweep(label, fixLocks, min, M) {
  const splits = [[40,30,20],[20,40,30],[45,15,30],[30,30,30],[60,20,10],[20,60,10],
                  [10,20,60],[70,10,10],[10,70,10],[10,10,70]];
  const rows = splits.map(([m,r_,h]) => {
    const res = go(fixLocks, (c) => {
      c.types.melee.cap = m; c.types.ranged.cap = r_; c.types.healer.cap = h;
    }, min, M);
    return [`${m}/${r_}/${h}`, res];
  }).sort((a,b) => b[1].peak - a[1].peak);
  const best = rows[0][1].peak, worst = rows[rows.length-1][1].peak;
  console.log(`\n### ${label}`);
  for (const [n, r] of rows) {
    console.log('  caps ' + n.padEnd(12) + String(r.peak).padStart(6) +
      ((r.peak/worst - 1)*100).toFixed(1).padStart(7) + '%' + r.goldEarned.toExponential(2).padStart(11));
  }
  console.log(`  --> best/worst spread: ${((best/worst - 1)*100).toFixed(1)}%`);
}

sweep('Run 1, lock curve AS SHIPPED (4.0)', false, 40, 1);
sweep('Run 1, lock curve FIXED (2.5)',      true,  40, 1);

const warm = simulateCampaign(base(true), 5, { maxSeconds: 90*60 });
const M5 = warm[4].prestigeMultOut;
sweep(`Run 6 (M=${M5.toExponential(1)}), lock curve FIXED (2.5)`, true, 60, M5);

console.log('\n### Is a SINGLE-type cap relic worth anything? (lock curve fixed, run 1)');
const b = go(true, () => {}, 40, 1);
console.log('  baseline 40/30/20'.padEnd(34) + String(b.peak).padStart(5));
for (const ty of ['melee','ranged','healer']) {
  for (const k of [1.5, 2, 3]) {
    const r = go(true, (c) => { c.types[ty].cap = Math.round(c.types[ty].cap * k); }, 40, 1);
    console.log(`  ${ty} cap x${k}`.padEnd(34) + String(r.peak).padStart(5) +
      ((r.peak/b.peak - 1)*100).toFixed(1).padStart(7) + '%');
  }
}
