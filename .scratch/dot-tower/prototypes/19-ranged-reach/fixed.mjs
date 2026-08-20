// PROTOTYPE — throwaway. Ticket 19, third pass — the decisive one.
//
// The first two passes were contaminated by the stall heuristic: variants that GRIND rather than
// stall run four times as long and reach a deeper floor for that reason alone. This pass disables
// the stall detector entirely and runs every variant for exactly the same wall-clock time, so the
// only thing that can differ is the mechanic.
//
// Run: node fixed.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;                 // the run never "ends"; it just keeps going
  return c;
};

const VARIANTS = [
  ['baseline — threat weights, no reach', () => {}],
  ['attack reach 1', (c) => { c.reach.ranged = 1; }],
  ['attack reach 2', (c) => { c.reach.ranged = 2; }],
  ['attack reach 3', (c) => { c.reach.ranged = 3; }],
  ['attack reach 5', (c) => { c.reach.ranged = 5; }],
  ['standoff 1', (c) => { c.reach.ranged = 1; c.standoff = true; }],
  ['standoff 2', (c) => { c.reach.ranged = 2; c.standoff = true; }],
  ['standoff 3', (c) => { c.reach.ranged = 3; c.standoff = true; }],
  ['formation: melee front', (c) => { c.targeting = 'line'; c.lineFocus = true; }],
  ['formation: ranged front', (c) => { c.targeting = 'line'; c.lineFocus = true; c.formation = { ranged: 0, melee: 1, healer: 2 }; }],
  ['formation: healer front', (c) => { c.targeting = 'line'; c.lineFocus = true; c.formation = { healer: 0, melee: 1, ranged: 2 }; }],
  ['formation: no order at all', (c) => { c.targeting = 'line'; c.lineFocus = true; c.formation = { melee: 0, ranged: 0, healer: 0 }; }],
  ['formation (spread) melee front', (c) => { c.targeting = 'line'; }],
  ['melee front + standoff 2', (c) => { c.targeting = 'line'; c.lineFocus = true; c.reach.ranged = 2; c.standoff = true; }],
];

function run(label, minutes, prestigeMult) {
  console.log(`\n### ${label} — every run exactly ${minutes} minutes, stall detector off`);
  console.log('  ' + 'variant'.padEnd(34) + 'peak'.padStart(6) + '   vs base' +
              'gold'.padStart(11) + 'pop m/r/h'.padStart(12) + 'deaths m/r/h'.padStart(16) +
              'heal% m/r/h/H'.padStart(15) + 'up%'.padStart(5));
  let basePeak = null;
  for (const [name, mutate] of VARIANTS) {
    const c = base(); mutate(c);
    const r = simulateRun(c, { prestigeMult, maxSeconds: minutes * 60 });
    if (basePeak === null) basePeak = r.peak;
    const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));
    const m = (k, ty) => tail.reduce((a, s) => a + s[k][ty], 0) / (tail.length || 1);
    const heal = r.healToHero + r.healToClimbers;
    console.log('  ' + name.padEnd(34) + String(r.peak).padStart(6) +
      `${r.peak === basePeak ? '' : (r.peak > basePeak ? '+' : '')}${r.peak === basePeak ? '   —' : (((r.peak / basePeak - 1) * 100).toFixed(1) + '%')}`.padStart(10) +
      r.goldEarned.toExponential(2).padStart(11) +
      ['melee', 'ranged', 'healer'].map((t) => Math.round(m('popByType', t))).join('/').padStart(12) +
      `${r.deathsByType.melee}/${r.deathsByType.ranged}/${r.deathsByType.healer}`.padStart(16) +
      ['melee', 'ranged', 'healer', 'hero'].map((k) => ((r.healByKind[k] / heal) * 100).toFixed(0)).join('/').padStart(15) +
      (r.heroUptime * 100).toFixed(0).padStart(5));
  }
}

run('Run 1', 40, 1);
run('Run 1, longer', 90, 1);

const warm = simulateCampaign(base(), 5, { maxSeconds: 90 * 60 });
const M5 = warm[4].prestigeMultOut;
run(`Run 6 (M=${M5.toExponential(2)})`, 60, M5);
