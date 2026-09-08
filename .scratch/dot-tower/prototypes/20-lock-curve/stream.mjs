// PROTOTYPE — throwaway. Ticket 20, Q2 and Q1.
//
// Q2: K (lock time price) couples walk and crowd — dearer lock, longer road,
// bigger crowd. But the map's fog wants crowd and walk as SEPARATE authored
// numbers: road length ~ the tower column's 12-14 floor window (ticket 10), crowd
// big enough to read as an army. climbSpeed sets road TIME at fixed road LENGTH
// (slow walk = more climbers alive on the same road); replacement sets inflow.
// This checks those dials actually move crowd without moving walk, at K=30.
//
// Q1: is locking load-bearing at all? buyPolicy 'noLock' vs 'cheapest' vs 'even'
// at K=30 -- if never locking barely costs depth, locks are decoration; if it
// costs everything, they are a mandatory tax; in between is a real decision.
//
// Run: node stream.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const base = (mutate) => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  c.sealedIncome = false;
  c.lockPricing = 'time'; c.lockTimeCost = 30;
  c.lockFreeBelowBest = true;        // head start is conquered territory; price lives at the frontier
  if (mutate) mutate(c);
  return c;
};

function probe(c, M, min, bestEver = 0) {
  const r = simulateRun(c, { maxSeconds: min * 60, prestigeMult: M, bestEverFloor: bestEver });
  const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));
  const avg = (f) => tail.reduce((a, s) => a + f(s), 0) / (tail.length || 1);
  const locks = r.events.filter((e) => e.kind === 'lock');
  return {
    peak: r.peak,
    walk: avg((s) => s.wall - s.lockLine),
    spread: avg((s) => s.climberSpread),
    pop: avg((s) => s.pop),
    popSplit: ['melee', 'ranged', 'healer'].map((t) => Math.round(avg((s) => s.popByType[t]))).join('/'),
    capSkips: avg((s) => s.capSkips),
    locks: locks.length,
    entMax: Math.max(...tail.map((s) => s.entities)),
  };
}

function row(label, p) {
  console.log('  ' + label.padEnd(28) +
    String(Math.round(p.peak)).padStart(5) +
    `  walk ${p.walk.toFixed(1).padStart(6)}` +
    `  spread ${p.spread.toFixed(1).padStart(6)}` +
    `  pop ${String(Math.round(p.pop)).padStart(3)} (${p.popSplit})`.padEnd(16) +
    `  skips ${p.capSkips.toFixed(2)}` +
    `  locks ${String(p.locks).padStart(3)}` +
    `  ent ${p.entMax}`);
}

const warm = simulateCampaign(base(), 5, { maxSeconds: 90 * 60 });
const M5 = warm[4].prestigeMultOut;
console.log(`K=30 throughout, M5 = ${M5.toExponential(2)}\n`);

console.log('### Q2a — road TIME at fixed road length (climbSpeed), run 6');
const best5 = Math.max(...warm.map((r) => r.peak));
for (const cs of [0.25, 0.5, 1.0]) row(`climbSpeed ${cs}`, probe(base((c) => { c.climbSpeed = cs; }), M5, 60, best5));
console.log('### Q2b — inflow (replacement), run 6');
for (const rep of [0.83, 1.67, 3.33]) row(`replacement ${rep}s`, probe(base((c) => { c.replacement = rep; }), M5, 60, best5));

console.log('\n### Q1 — is locking load-bearing? (buyPolicy at K=30)');
for (const [label, M, min] of [['run 1', 1, 40], ['run 6', M5, 60]]) {
  console.log(`  ${label}:`);
  for (const pol of ['cheapest', 'noLock', 'even']) row(`  policy ${pol}`, probe(base((c) => { c.buyPolicy = pol; }), M, min, M === 1 ? 0 : best5));
}

console.log('\n### Q2c — does the shape hold at run-10 depth? (campaign to M10, then one run)');
const deep = simulateCampaign(base(), 10, { maxSeconds: 90 * 60 });
const M10 = deep[9].prestigeMultOut;
row(`run 11 (M=${M10.toExponential(1)})`, probe(base(), M10, 60, Math.max(...deep.map((r) => r.peak))));
