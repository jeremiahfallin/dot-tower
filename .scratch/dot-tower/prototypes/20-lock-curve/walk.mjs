// PROTOTYPE — throwaway. Ticket 20: travel time and the lock curve.
//
// lockCost(k) = lockCost0 * B^(k-1), one lock per 10 floors, while gold per kill
// grows goldBase^10 = 2.478 per 10 floors. B > 2.478 compounds unaffordability
// per lock (walk diverges); B < 2.478 compounds affordability (lock line rides
// the wall); B = 2.478 makes a lock cost constant *time* at depth. This sweep
// measures the stream each family actually produces: walk (wall - lock line),
// spread (floors covered by live climbers), crowd, and whether caps bind.
//
// Protocol: pinned wall-clock (ticket 19 -- the only trusted comparison), tail
// = last 40% of samples. Each B warms its own 5-run campaign, so "run 6" is run
// 6 of a campaign actually played under that curve.
//
// Run: node walk.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone, lockCost } from '../06-progression-curve/model.mjs';

const base = (mutate) => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;               // pinned-time protocol; no stall cutoff
  c.sealedIncome = false;             // ticket 06's withdrawal, implemented (see diagnose.mjs:
                                      // sealed income is 99% of a fixed-curve run otherwise)
  if (mutate) mutate(c);
  return c;
};

const MATCH = Math.pow(DEFAULTS.goldBase, 10);   // 2.478: income per lock interval

function probe(c, M, min) {
  const r = simulateRun(c, { maxSeconds: min * 60, prestigeMult: M });
  const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));
  const avg = (f) => tail.reduce((a, s) => a + f(s), 0) / (tail.length || 1);
  const pop = ['melee', 'ranged', 'healer'].map((t) => Math.round(avg((s) => s.popByType[t]))).join('/');
  const lockEvents = r.events.filter((e) => e.kind === 'lock');
  const lockGold = lockEvents.reduce((a, e) => a + lockCost(c, e.floor / 10), 0);
  const dts = lockEvents.slice(1).map((e, i) => e.t - lockEvents[i].t);
  dts.sort((a, b) => a - b);
  const cadence = dts.length ? dts[Math.floor(dts.length / 2)] : NaN;
  return {
    peak: r.peak,
    walk: avg((s) => s.wall - s.lockLine),
    spread: avg((s) => s.climberSpread),
    back: avg((s) => (s.backByType.melee + s.backByType.ranged + s.backByType.healer) / 3),
    pop: avg((s) => s.pop), popSplit: pop,
    capSkips: avg((s) => s.capSkips),
    locks: lockEvents.length, lockGold, share: lockGold / r.goldEarned, cadence,
    entMax: Math.max(...tail.map((s) => s.entities)),
    endLock: r.lockLine,
  };
}

function row(label, p) {
  console.log('  ' + label.padEnd(22) +
    String(Math.round(p.peak)).padStart(5) +
    `  walk ${p.walk.toFixed(1).padStart(6)}` +
    `  spread ${p.spread.toFixed(1).padStart(6)}` +
    `  back ${p.back.toFixed(1).padStart(5)}` +
    `  pop ${String(Math.round(p.pop)).padStart(3)} (${p.popSplit})`.padEnd(16) +
    `  skips ${p.capSkips.toFixed(2)}` +
    `  locks ${String(p.locks).padStart(3)} @ ${p.cadence.toFixed(0)}s` +
    `  gold ${ (p.share * 100).toFixed(0).padStart(3)}%` +
    `  ent ${p.entMax}`);
}

for (const B of [MATCH, 2.55, 2.7, 3.0, 3.5, 4.0]) {
  const warm = simulateCampaign(base((c) => { c.lockCostBase = B; }), 5, { maxSeconds: 90 * 60 });
  const M5 = warm[4].prestigeMultOut;
  const label = B === MATCH ? `${B.toFixed(3)} (matched)` : `${B}`;
  console.log(`\n### lockCostBase ${label}   [M5 = ${M5.toExponential(2)}, warm peak ${warm[4].peak}]`);
  row('run 1', probe(base((c) => { c.lockCostBase = B; }), 1, 40));
  row('run 6', probe(base((c) => { c.lockCostBase = B; }), M5, 60));
}
