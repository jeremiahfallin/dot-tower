// PROTOTYPE — throwaway. Ticket 17: does the beyondBest campaign hold past floor 1000?
// (f64 ceiling ~2000; stop before gold overflows.) Run: node long.mjs
import { DEFAULTS, simulateRun, clone } from '../06-progression-curve/model.mjs';
const c = clone(DEFAULTS);
c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
c.sealedIncome = false; c.lockPricing = 'time'; c.lockTimeCost = 30;
c.lockFreeBelowBest = true; c.prestigeBasis = 'beyondBest';
let M = 1, best = 0, f1000 = null;
for (let i = 1; i <= 20; i++) {
  const r = simulateRun(c, { prestigeMult: M, maxSeconds: 4 * 3600, bestEverFloor: best });
  M *= r.prestigeMultEarned;
  const h = Math.log(M) / Math.log(DEFAULTS.enemyHpBase);
  if (f1000 === null && r.peak >= 1000) f1000 = i;
  console.log(`run ${String(i).padStart(2)}: ${String(Math.round(r.seconds / 60)).padStart(3)} min  peak ${String(r.peak).padStart(4)}  new ${String(r.peak - best).padStart(4)}  earned x${r.prestigeMultEarned.toFixed(2)}  M ${M.toExponential(2)}  h ${String(Math.round(h)).padStart(3)} (${Math.round(100 * h / r.peak)}% of peak)`);
  best = Math.max(best, r.peak);
  if (best > 1900) { console.log('(stopping: f64 ceiling)'); break; }
}
console.log(`floor 1000 at run ${f1000}`);
