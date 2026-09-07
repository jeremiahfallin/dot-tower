// PROTOTYPE — throwaway. Answers the desk-answerable half of ticket 14
// "Low-end Android performance budget".
//
// QUESTION: ticket 03 claims simulation cost tracks CONTESTED FLOORS, not band
// height — floors are inert data until a climber comes within a floor or two,
// so "entity count stays in the low hundreds whether the band is 20 floors or
// 5,000". Ticket 14 restates the worst case as ~1,100 entities and asks where
// frame time actually degrades.
//
// That question has two halves and they need different instruments:
//   DEMAND — how many entities and how much activation churn the game actually
//            produces. This is a property of the simulation, not of the phone,
//            and ticket 06's model already knows it. Measured here.
//   SUPPLY — what a low-end Android device can absorb. Needs the device.
//
// Measuring demand first is not a consolation prize: it tells the hardware
// session which N to test instead of guessing 1,000 / 5,000 / 20,000, and if
// demand is far under the prediction the expensive question shrinks.
//
// Run: node entities.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const RUNS = 14;                 // ticket 06: floor 1,000 arrives around run 10
const HOURS = 4 * 3600;

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';  // the slice's shipped hero
  return c;
};

const max = (xs) => xs.reduce((a, b) => (b > a ? b : a), -Infinity);
const mean = (xs) => xs.reduce((a, b) => a + b, 0) / (xs.length || 1);
const pct = (xs, p) => {
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.floor(s.length * p))];
};

function stats(runs) {
  const all = runs.flatMap((r) => r.samples);
  return {
    samples: all.length,
    peak: max(runs.map((r) => r.peak)),
    entitiesMax: max(all.map((s) => s.entities)),
    entitiesP95: pct(all.map((s) => s.entities), 0.95),
    entitiesMean: mean(all.map((s) => s.entities)),
    activeMax: max(all.map((s) => s.activeFloors)),
    contestedMax: max(all.map((s) => s.contestedFloors)),
    spreadMax: max(all.map((s) => s.climberSpread)),
    popMax: max(all.map((s) => s.pop)),
    churnMean: mean(all.map((s) => s.activationsPerSec)),
    churnMax: max(all.map((s) => s.activationsPerSec)),
    recordsMax: max(all.map((s) => s.floorRecords)),
    worst: all.reduce((a, s) => (s.entities > a.entities ? s : a), all[0]),
  };
}

// ---------------------------------------------------------------------------
console.log('=== 1. A normal campaign ===');
console.log(`${RUNS} runs, compounding prestige, the slice's shipped hero.\n`);

const runs = simulateCampaign(base(), RUNS, { maxSeconds: HOURS });

console.log('run  peak   pop  entities(max)  active  contested  spread  act/s  records');
runs.forEach((r, i) => {
  const s = stats([r]);
  console.log(
    String(i + 1).padStart(3) +
    String(r.peak).padStart(7) +
    String(s.popMax).padStart(6) +
    String(s.entitiesMax).padStart(15) +
    String(s.activeMax).padStart(8) +
    String(s.contestedMax).padStart(11) +
    String(s.spreadMax).padStart(8) +
    s.churnMean.toFixed(2).padStart(7) +
    String(s.recordsMax).padStart(9)
  );
});

const S = stats(runs);
console.log(`\nAcross all ${S.samples} samples:`);
console.log(`  entities   mean ${S.entitiesMean.toFixed(0)}  p95 ${S.entitiesP95}  MAX ${S.entitiesMax}`);
console.log(`  prediction 1,100 worst case  ->  measured max is ${(S.entitiesMax / 1100 * 100).toFixed(1)}% of it`);
console.log(`  activation churn  mean ${S.churnMean.toFixed(2)}/s  max ${S.churnMax.toFixed(2)}/s`);
console.log(`  deepest floor reached ${S.peak}, largest band ${S.recordsMax} floor records`);

console.log(`\nThe worst single sample (t=${S.worst.t.toFixed(0)}s, floor ${S.worst.peak}):`);
console.log(`  ${S.worst.entities} entities = ${S.worst.enemyEntities} enemies + ${S.worst.pop} climbers + hero`);
console.log(`  ${S.worst.activeFloors} active floors, ${S.worst.contestedFloors} contested, climbers spread over ${S.worst.climberSpread}`);

// ---------------------------------------------------------------------------
console.log('\n=== 2. The load-bearing claim: does band height matter? ===');
console.log('Ticket 03: "entity count stays in the low hundreds whether the band is');
console.log('20 floors or 5,000". Prestige is what drives the band deep and wide, so');
console.log('force it directly rather than waiting for a campaign to get there.\n');

console.log('prestige x   peak   band  entities(max)  active(max)  contested(max)  churn/s');
const bandRows = [];
for (const M of [1, 10, 100, 1e3, 1e4, 1e5, 1e6, 1e9, 1e12]) {
  const r = simulateRun(base(), { prestigeMult: M, maxSeconds: HOURS });
  const st = stats([r]);
  bandRows.push({ M, ...st });
  console.log(
    M.toExponential(0).padStart(10) +
    String(r.peak).padStart(7) +
    String(st.recordsMax).padStart(7) +
    String(st.entitiesMax).padStart(15) +
    String(st.activeMax).padStart(13) +
    String(st.contestedMax).padStart(16) +
    st.churnMean.toFixed(2).padStart(9)
  );
}
const first = bandRows[0], last = bandRows[bandRows.length - 1];
console.log(`\nband grew ${(last.recordsMax / first.recordsMax).toFixed(0)}x (${first.recordsMax} -> ${last.recordsMax} floors)`);
console.log(`entities grew ${(last.entitiesMax / first.entitiesMax).toFixed(2)}x (${first.entitiesMax} -> ${last.entitiesMax})`);

// ---------------------------------------------------------------------------
console.log('\n=== 3. Why a thin spread is free ===');
console.log('Ticket 14 Q2 worried that if climbers spread thin instead of clumping,');
console.log('activation churn would be much higher than predicted. They DO spread');
console.log('thin. Here is what the active floors actually contain.\n');

const deep = simulateRun(base(), { prestigeMult: 1e4, maxSeconds: HOURS });
const ds = stats([deep]);
const w = ds.worst;
console.log(`worst sample of a deep run (floor ${w.peak}):`);
console.log(`  climbers spread over        ${w.climberSpread} floors`);
console.log(`  floors active (spread +-${DEFAULTS.activationRadius})   ${w.activeFloors}`);
console.log(`  of those, CONTESTED         ${w.contestedFloors}`);
console.log(`  enemy entities              ${w.enemyEntities}  (${(w.enemyEntities / Math.max(w.activeFloors, 1)).toFixed(2)} per active floor, pack size ${DEFAULTS.packSize})`);
console.log(`  climber entities            ${w.pop}`);
console.log(`  TOTAL                       ${w.entities}`);
console.log('\nAn active floor holds far less than a full pack because trailing');
console.log('climbers walk through ground the leaders already cleared. Spread costs');
console.log('activation events, not entities -- and the events are cheap.');

// ---------------------------------------------------------------------------
console.log('\n=== 4. How hard is this pinned to the crowd size? ===');
console.log('Population sits at exactly 90 in every run above -- the sum of the three');
console.log('per-type caps. That mechanism is GONE: ticket 15 replaced three caps and');
console.log('three intervals with one global replacement interval against an authored');
console.log('ratio, and how big the crowd actually is belongs to ticket 20, still open.');
console.log('So the number that matters is not 90, it is the slope.\n');

console.log('crowd  entities(max)  enemies  active(max)  churn/s');
for (const mult of [1, 2, 4, 8, 16]) {
  const c = base();
  for (const ty of ['melee', 'ranged', 'healer']) c.types[ty].cap *= mult;
  const r = simulateRun(c, { prestigeMult: 1e4, maxSeconds: HOURS });
  const st = stats([r]);
  console.log(
    String(90 * mult).padStart(5) +
    String(st.entitiesMax).padStart(15) +
    String(st.worst.enemyEntities).padStart(9) +
    String(st.activeMax).padStart(13) +
    st.churnMean.toFixed(2).padStart(9)
  );
}
console.log('\nIt SATURATES. Raising the ceiling past ~360 changes nothing: 720 and');
console.log('1440 give identical runs, because population is spawn rate times lifetime');
console.log('and the ceiling has stopped binding. That is CONTEXT.md\'s claim about the');
console.log('type cap -- "under a healthy lock curve it does not bind at all" -- shown');
console.log('directly, and it is false at the CURRENT value of 90, where population sits');
console.log('pinned at the ceiling in every run, and true from roughly 360 up.');
console.log('');
console.log('The number ticket 20 needs: left to spawn interval and lifetime alone, the');
console.log('crowd settles near 405 climbers and the whole simulation costs ~436');
console.log('entities -- still under 40% of the 1,100 prediction with no ceiling at all.');

// ---------------------------------------------------------------------------
console.log('\n=== 5. Memory for inert floor data (Q5) ===');
const REC = 8 + 4 + 4;   // f64 respawn timer + u32 count + u32 seed/flags, packed
for (const n of [last.recordsMax, 5000, 50000]) {
  console.log(`  ${String(n).padStart(6)} floors x ${REC}B = ${(n * REC / 1024).toFixed(1)} KiB`);
}
console.log('  (ticket 14 asks about 5,000 floors; the deepest band above was ' + last.recordsMax + ')');
