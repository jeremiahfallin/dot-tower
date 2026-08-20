// PROTOTYPE — throwaway. Ticket 19, second pass.
//
// reach.mjs compares runs of wildly different LENGTHS (13 min to 240 min), and a longer run
// reaches a deeper floor for free. This re-measures every variant at FIXED WALL-CLOCK TIMES,
// which is the only comparison that isolates the mechanic from the stall heuristic — the same
// heuristic ticket 06 flagged as +/-40% and ticket 07 hit the limits of.
//
// It also tests the sensitivity of the headline result: does front-to-back targeting still win
// when the pack CONCENTRATES on the weakest front-liner (lineFocus) instead of sharing the hit
// out across the whole front rank?
//
// Run: node normalise.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const HOURS = 4 * 3600;
const MARKS = [10, 20, 30, 45, 60, 90];

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  return c;
};

function probe(name, mutate, { prestigeMult = 1 } = {}) {
  const c = base(); mutate(c);
  const r = simulateRun(c, { prestigeMult, maxSeconds: HOURS });
  const at = (min) => {
    const s = [...r.samples].reverse().find((s) => s.t <= min * 60);
    return s && r.seconds >= min * 60 - 30 ? s : null;
  };
  return { name, r, peakAt: MARKS.map(at).map((s) => (s ? s.peak : null)),
           goldAt: MARKS.map(at).map((s) => (s ? s.goldEarned : null)) };
}

function table(title, rows) {
  console.log(`\n### ${title}`);
  console.log('  ' + 'variant'.padEnd(30) + MARKS.map((m) => `${m}m`.padStart(7)).join('') +
              '  |' + 'ran'.padStart(6) + 'endPeak'.padStart(8) + '  gold@30m'.padStart(11));
  for (const p of rows) {
    console.log('  ' + p.name.padEnd(30) +
      p.peakAt.map((v) => (v === null ? '–' : String(v)).padStart(7)).join('') +
      '  |' + `${(p.r.seconds / 60).toFixed(0)}m`.padStart(6) + String(p.r.peak).padStart(8) +
      '  ' + (p.goldAt[2] === null ? '–' : p.goldAt[2].toExponential(2)).padStart(9));
  }
}

table('Run 1, floor reached by wall-clock minute', [
  probe('baseline (threat, no reach)', () => {}),
  probe('attack reach 2', (c) => { c.reach.ranged = 2; }),
  probe('attack reach 3', (c) => { c.reach.ranged = 3; }),
  probe('standoff 2', (c) => { c.reach.ranged = 2; c.standoff = true; }),
  probe('standoff 3', (c) => { c.reach.ranged = 3; c.standoff = true; }),
  probe('line (spread)', (c) => { c.targeting = 'line'; }),
  probe('line (focus)', (c) => { c.targeting = 'line'; c.lineFocus = true; }),
  probe('line focus + standoff 2', (c) => { c.targeting = 'line'; c.lineFocus = true; c.reach.ranged = 2; c.standoff = true; }),
]);

const warm = simulateCampaign(base(), 5, { maxSeconds: HOURS });
const M5 = warm[4].prestigeMultOut;
table(`Run 6 (M=${M5.toExponential(2)}), floor reached by wall-clock minute`, [
  probe('baseline (threat, no reach)', () => {}, { prestigeMult: M5 }),
  probe('attack reach 2', (c) => { c.reach.ranged = 2; }, { prestigeMult: M5 }),
  probe('standoff 2', (c) => { c.reach.ranged = 2; c.standoff = true; }, { prestigeMult: M5 }),
  probe('line (spread)', (c) => { c.targeting = 'line'; }, { prestigeMult: M5 }),
  probe('line (focus)', (c) => { c.targeting = 'line'; c.lineFocus = true; }, { prestigeMult: M5 }),
  probe('line focus + standoff 2', (c) => { c.targeting = 'line'; c.lineFocus = true; c.reach.ranged = 2; c.standoff = true; }, { prestigeMult: M5 }),
]);

// Does the formation ORDER matter, or only that damage is concentrated somewhere?
table('Does the order matter, or only the concentration? (run 1)', [
  probe('threat weights (baseline)', () => {}),
  probe('line: melee front', (c) => { c.targeting = 'line'; c.lineFocus = true; }),
  probe('line: ranged front', (c) => { c.targeting = 'line'; c.lineFocus = true; c.formation = { ranged: 0, melee: 1, healer: 2 }; }),
  probe('line: healer front', (c) => { c.targeting = 'line'; c.lineFocus = true; c.formation = { healer: 0, melee: 1, ranged: 2 }; }),
  probe('line: all one rank', (c) => { c.targeting = 'line'; c.lineFocus = true; c.formation = { melee: 0, ranged: 0, healer: 0 }; }),
]);
