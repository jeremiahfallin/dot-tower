// PROTOTYPE — throwaway. Ticket 19, verification pass.
// fixed.mjs produced two results with a story attached. This checks the stories.
//   A) Standoff collapses gold by 4-5 orders of magnitude. Claim: because standing off walks
//      ranged OUT of the hero's aura, which ticket 16 fixed at +/-0 floors.
//   B) Attack reach raises gold with no change to who dies. Claim: it is a damage buff, not a
//      safety mechanic — so a plain ranged damage increase should reproduce it.
// Run: node why.mjs
import { DEFAULTS, simulateRun, clone } from '../06-progression-curve/model.mjs';

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  return c;
};
const go = (mutate, min = 40) => {
  const c = base(); mutate(c);
  const r = simulateRun(c, { maxSeconds: min * 60 });
  return r;
};
const show = (name, r, ref) => console.log('  ' + name.padEnd(44) + String(r.peak).padStart(5) +
  r.goldEarned.toExponential(2).padStart(11) +
  (ref ? `  (gold ${(r.goldEarned / ref.goldEarned).toFixed(2)}x base)` : '') +
  `  ranged deaths ${r.deathsByType.ranged}`);

console.log('\n### A) Is the standoff gold collapse the hero aura?');
const b = go(() => {});
show('baseline', b, null);
show('standoff 2 (aura reach +/-0, as shipped)', go((c) => { c.reach.ranged = 2; c.standoff = true; }), b);
show('standoff 2, aura widened to +/-2', go((c) => { c.reach.ranged = 2; c.standoff = true; c.heroAuraFloors = 2; }), b);
show('standoff 2, hero aura off entirely', go((c) => { c.reach.ranged = 2; c.standoff = true; c.heroAuraMult = 1; }), b);
show('baseline, hero aura off entirely', go((c) => { c.heroAuraMult = 1; }), b);

console.log('\n### B) Is attack reach just a ranged damage buff?');
show('baseline', b, null);
show('attack reach 2', go((c) => { c.reach.ranged = 2; }), b);
for (const pct of [0.1, 0.25, 0.5, 1.0]) {
  show(`no reach, ranged dps +${(pct * 100).toFixed(0)}%`,
       go((c) => { c.types.ranged.dps0 *= 1 + pct; }), b);
}

console.log('\n### C) Does reach do anything a WIDER type cap does not?');
show('baseline', b, null);
show('attack reach 2', go((c) => { c.reach.ranged = 2; }), b);
show('no reach, ranged cap 30 -> 36 (+20%)', go((c) => { c.types.ranged.cap = 36; }), b);
