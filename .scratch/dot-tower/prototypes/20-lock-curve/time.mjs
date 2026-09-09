// PROTOTYPE — throwaway. Ticket 20: locks priced in TIME — K seconds of rolling-60s income.
//
// The geometric curve cannot hold a constant relationship to income (income
// compounds faster than per-kill gold because kill rate grows with ranks), so
// every geometric base either goes free at depth (stream collapses onto the
// wall) or diverges (travel grows without bound). Pricing a lock as K seconds
// of current income is matched by construction and survives rank retunes.
//
// This sweeps K and prints the stream it produces plus the gold-flow split,
// because K is also the lock-vs-rank competition dial: one lock costs K seconds
// of whatever else gold was going to buy.
//
// Run: node time.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone, rankCost, heroCost } from '../06-progression-curve/model.mjs';

const base = (mutate) => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  c.sealedIncome = false;             // ticket 06's withdrawal, implemented
  c.lockPricing = 'time';
  c.lockFreeBelowBest = true;         // head start is conquered territory (see stream.mjs Q2c
                                      // for why time pricing alone fails on a mature account)
  if (mutate) mutate(c);
  return c;
};

function probe(c, M, min, bestEver = 0) {
  const r = simulateRun(c, { maxSeconds: min * 60, prestigeMult: M, bestEverFloor: bestEver });
  const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));
  const avg = (f) => tail.reduce((a, s) => a + f(s), 0) / (tail.length || 1);
  const lockEvents = r.events.filter((e) => e.kind === 'lock');
  const dts = lockEvents.slice(1).map((e, i) => e.t - lockEvents[i].t).sort((a, b) => a - b);
  let rank = 0;
  for (const ty of ['melee', 'ranged', 'healer']) for (let i = 1; i < r.ranks[ty]; i++) rank += rankCost(c, ty, i);
  let hero = 0;
  for (let l = 1; l < r.heroLevel; l++) hero += heroCost(c, l);
  const lock = lockEvents.reduce((a, e) => a + (e.cost || 0), 0);
  return {
    peak: r.peak,
    walk: avg((s) => s.wall - s.lockLine),
    spread: avg((s) => s.climberSpread),
    back: avg((s) => (s.backByType.melee + s.backByType.ranged + s.backByType.healer) / 3),
    pop: avg((s) => s.pop),
    popSplit: ['melee', 'ranged', 'healer'].map((t) => Math.round(avg((s) => s.popByType[t]))).join('/'),
    capSkips: avg((s) => s.capSkips),
    locks: lockEvents.length,
    cadence: dts.length ? dts[Math.floor(dts.length / 2)] : NaN,
    share: lock / r.goldEarned, rankShare: rank / r.goldEarned, heroShare: hero / r.goldEarned,
    poolShare: r.gold / r.goldEarned,
    entMax: Math.max(...tail.map((s) => s.entities)),
    lifeSec: avg((s) => s.pop) * c.replacement,   // pop x interval = mean lifetime
  };
}

function row(label, p) {
  console.log('  ' + label.padEnd(14) +
    String(Math.round(p.peak)).padStart(5) +
    `  walk ${p.walk.toFixed(1).padStart(6)}` +
    `  spread ${p.spread.toFixed(1).padStart(6)}` +
    `  pop ${String(Math.round(p.pop)).padStart(3)} (${p.popSplit})`.padEnd(16) +
    `  life ${String(Math.round(p.lifeSec)).padStart(3)}s` +
    `  skips ${p.capSkips.toFixed(2)}` +
    `  locks ${String(p.locks).padStart(3)} @ ${p.cadence.toFixed(0).padStart(3)}s` +
    `  gold: rank ${(p.rankShare * 100).toFixed(0)}% hero ${(p.heroShare * 100).toFixed(0)}% lock ${(p.share * 100).toFixed(0).padStart(3)}% pool ${(p.poolShare * 100).toFixed(0)}%` +
    `  ent ${p.entMax}`);
}

for (const K of [15, 30, 60, 120]) {
  const warm = simulateCampaign(base((c) => { c.lockTimeCost = K; }), 5, { maxSeconds: 90 * 60 });
  const M5 = warm[4].prestigeMultOut;
  console.log(`\n### lockTimeCost = ${K}s   [M5 = ${M5.toExponential(2)}, warm run-5 peak ${warm[4].peak}]`);
  const best5 = Math.max(...warm.map((r) => r.peak));
  row('run 1', probe(base((c) => { c.lockTimeCost = K; }), 1, 40));
  row('run 6', probe(base((c) => { c.lockTimeCost = K; }), M5, 60, best5));
}
