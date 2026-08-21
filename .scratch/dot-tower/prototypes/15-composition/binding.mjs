// PROTOTYPE — throwaway. Ticket 15, follow-up.
// composition.mjs found that at depth EVERY cap split returns exactly 749 — 70/10/10 and
// 10/70/10 alike. Identical to the floor means something other than the climbers is setting
// the pace. This finds out what.
// Run: node binding.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  return c;
};
const warm = simulateCampaign(base(), 5, { maxSeconds: 90 * 60 });
const M5 = warm[4].prestigeMultOut;

function probe(name, mutate, M, min) {
  const c = base(); mutate(c);
  const r = simulateRun(c, { maxSeconds: min * 60, prestigeMult: M });
  const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));
  const avg = (f) => tail.reduce((a, s) => a + f(s), 0) / (tail.length || 1);
  const pop = ['melee', 'ranged', 'healer'].map((t) => Math.round(avg((s) => s.popByType[t]))).join('/');
  const back = avg((s) => (s.backByType.melee + s.backByType.ranged + s.backByType.healer) / 3);
  console.log('  ' + name.padEnd(36) + String(r.peak).padStart(5) +
    r.goldEarned.toExponential(2).padStart(11) +
    `  pop ${pop}`.padEnd(16) +
    `  lock ${String(r.lockLine).padStart(4)}  ${(r.peak - r.lockLine).toFixed(0).padStart(4)} floors to walk` +
    `  avg ${back.toFixed(0).padStart(4)} behind`);
}

for (const [label, M, min] of [['Run 1', 1, 40], [`Run 6 (M=${M5.toExponential(1)})`, M5, 60]]) {
  console.log(`\n### ${label} — what moves the peak floor?`);
  probe('baseline', () => {}, M, min);
  console.log('  -- combat power --');
  probe('all climber damage x2', (c) => { for (const t of ['melee','ranged','healer']) c.types[t].dps0 *= 2; }, M, min);
  probe('all climber damage x10', (c) => { for (const t of ['melee','ranged','healer']) c.types[t].dps0 *= 10; }, M, min);
  probe('all caps x3', (c) => { for (const t of ['melee','ranged','healer']) c.types[t].cap *= 3; }, M, min);
  console.log('  -- getting there --');
  probe('climb speed x2', (c) => { c.climbSpeed *= 2; }, M, min);
  probe('climb speed x5', (c) => { c.climbSpeed *= 5; }, M, min);
  probe('lock cost base 4.0 -> 2.5', (c) => { c.lockCostBase = 2.5; }, M, min);
  probe('lock margin 15 -> 3', (c) => { c.lockMargin = 3; }, M, min);
  console.log('  -- supply --');
  probe('replacement 5s -> 2s', (c) => { for (const t of ['melee','ranged','healer']) c.types[t].spawnInterval = 2; }, M, min);
  probe('replacement 5s -> 15s', (c) => { for (const t of ['melee','ranged','healer']) c.types[t].spawnInterval = 15; }, M, min);
  probe('respawn timer 20s -> 5s', (c) => { c.respawnTimer = 5; }, M, min);
}
