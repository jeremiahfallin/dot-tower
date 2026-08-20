// PROTOTYPE — throwaway. Answers ticket 19 "Ranged attack reach".
//
// QUESTION: does combat have positional reach, and does giving ranged climbers one change
// anything? Today ranged safety is `threat: 1.0` against melee's `3.0` — a damage share with no
// position. Ticket 16 measured hero aura reach at ±0 through ±5 and found them identical,
// because combat concentrates everyone on the contested floor. That null result is the live
// hypothesis here too.
//
// Three independent things are being called "reach", and they are measured separately:
//   ATTACK REACH  — ranged fires at a floor above its own (`reach.ranged`), still walking to the wall.
//   STANDOFF      — ranged HALTS as soon as the fighting comes into reach (`standoff`), so it is
//                   never standing where the enemies are. Positional safety, not statistical.
//   FORMATION     — incoming damage falls front-to-back through the line (`targeting: 'line'`)
//                   instead of being split by threat weight. Positional safety WITHIN a floor.
//
// Run: node reach.mjs
import { DEFAULTS, simulateRun, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall'; // the slice's shipped hero
  return c;
};

const HOURS = 4 * 3600;
const mean = (xs) => xs.reduce((a, b) => a + b, 0) / (xs.length || 1);

function probe(name, mutate, { prestigeMult = 1 } = {}) {
  const c = base(); mutate(c);
  const r = simulateRun(c, { prestigeMult, maxSeconds: HOURS });
  const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));   // the mature run
  const pick = (k, ty) => mean(tail.map((s) => s[k][ty]));
  return {
    name,
    peak: r.peak,
    min: +(r.seconds / 60).toFixed(1),
    gold: r.goldEarned,
    // composition: who is alive, how healthy, and how far behind the wall they stand
    pop: ['melee', 'ranged', 'healer'].map((ty) => Math.round(pick('popByType', ty))).join('/'),
    hp: ['melee', 'ranged', 'healer'].map((ty) => (pick('hpByType', ty) * 100).toFixed(0)).join('/'),
    back: ['melee', 'ranged', 'healer'].map((ty) => pick('backByType', ty).toFixed(1)).join('/'),
    deaths: `${r.deathsByType.melee}/${r.deathsByType.ranged}/${r.deathsByType.healer}`,
    heal: ['melee', 'ranged', 'healer', 'hero']
      .map((k) => ((r.healByKind[k] / (r.healToHero + r.healToClimbers)) * 100).toFixed(0)).join('/'),
    uptime: (r.heroUptime * 100).toFixed(0),
    auraKills: r.killsInAura,
  };
}

function table(title, rows) {
  console.log(`\n### ${title}`);
  console.log('  ' + [
    'variant'.padEnd(34), 'peak'.padStart(5), 'min'.padStart(6), 'gold'.padStart(9),
    'pop m/r/h'.padStart(11), 'hp% m/r/h'.padStart(11), 'back m/r/h'.padStart(13),
    'deaths m/r/h'.padStart(15), 'heal% m/r/h/H'.padStart(15), 'up%'.padStart(4),
  ].join(' '));
  for (const r of rows) {
    console.log('  ' + [
      r.name.padEnd(34), String(r.peak).padStart(5), r.min.toFixed(1).padStart(6),
      r.gold.toExponential(2).padStart(9), r.pop.padStart(11), r.hp.padStart(11),
      r.back.padStart(13), r.deaths.padStart(15), r.heal.padStart(15), r.uptime.padStart(4),
    ].join(' '));
  }
}

// ---------------------------------------------------------------------------
// 1. Attack reach alone. Ranged shoots further; it still walks to the wall.
//    This is ticket 16's aura-reach experiment, rerun on the climbers.
// ---------------------------------------------------------------------------
table('Attack reach only — ranged fires N floors up, still stands at the wall', [
  probe('baseline (no reach)', () => {}),
  ...[1, 2, 3, 5].map((n) => probe(`reach ±${n}`, (c) => { c.reach.ranged = n; })),
]);

// ---------------------------------------------------------------------------
// 2. Standoff. Ranged halts as soon as the fighting is within reach, so the pack
//    on the contested floor can never touch it. Positional safety, across floors.
// ---------------------------------------------------------------------------
table('Standoff — ranged halts short and fires up', [
  probe('baseline (no reach)', () => {}),
  ...[1, 2, 3, 5].map((n) => probe(`standoff ${n}`, (c) => { c.reach.ranged = n; c.standoff = true; })),
  probe('standoff 2, ranged+healer', (c) => { c.reach.ranged = 2; c.reach.healer = 2; c.standoff = true; }),
]);

// ---------------------------------------------------------------------------
// 3. Formation. No cross-floor reach at all — the line is WITHIN the floor, and
//    incoming damage falls front-to-back instead of splitting by threat weight.
// ---------------------------------------------------------------------------
table('Formation — incoming falls front-to-back inside the floor', [
  probe('baseline (threat weights)', () => {}),
  probe('line: melee/ranged/healer', (c) => { c.targeting = 'line'; }),
  probe('line + reach 1', (c) => { c.targeting = 'line'; c.reach.ranged = 1; }),
  probe('line + standoff 1', (c) => { c.targeting = 'line'; c.reach.ranged = 1; c.standoff = true; }),
  probe('line + standoff 2', (c) => { c.targeting = 'line'; c.reach.ranged = 2; c.standoff = true; }),
]);

// ---------------------------------------------------------------------------
// 4. Ticket 03's rule. The model has never restored a floor nobody is fighting,
//    so attrition CAN beat a wall in it. Reach is the mechanic most exposed to
//    that, so measure both variants under the rule as actually specified.
// ---------------------------------------------------------------------------
table("Under ticket 03's rule — a floor nobody fights restores its pack", [
  probe('baseline', (c) => { c.failedFloorReset = true; }),
  probe('reach 2', (c) => { c.failedFloorReset = true; c.reach.ranged = 2; }),
  probe('standoff 2', (c) => { c.failedFloorReset = true; c.reach.ranged = 2; c.standoff = true; }),
  probe('line', (c) => { c.failedFloorReset = true; c.targeting = 'line'; }),
  probe('line + standoff 2', (c) => { c.failedFloorReset = true; c.targeting = 'line'; c.reach.ranged = 2; c.standoff = true; }),
]);

// ---------------------------------------------------------------------------
// 5. Deep in a campaign. Ticket 16 found effects that vanish at depth; check
//    that whatever survives above still survives at run 5's prestige multiplier.
// ---------------------------------------------------------------------------
const warm = simulateCampaign(base(), 5, { maxSeconds: HOURS });
const M5 = warm[4].prestigeMultOut;
console.log(`\n(campaign warm-up: run 5 peak ${warm[4].peak}, carrying M=${M5.toExponential(2)} into run 6)`);
table('Run 6, at depth', [
  probe('baseline', () => {}, { prestigeMult: M5 }),
  probe('reach 2', (c) => { c.reach.ranged = 2; }, { prestigeMult: M5 }),
  probe('standoff 2', (c) => { c.reach.ranged = 2; c.standoff = true; }, { prestigeMult: M5 }),
  probe('line', (c) => { c.targeting = 'line'; }, { prestigeMult: M5 }),
  probe('line + standoff 2', (c) => { c.targeting = 'line'; c.reach.ranged = 2; c.standoff = true; }, { prestigeMult: M5 }),
]);
