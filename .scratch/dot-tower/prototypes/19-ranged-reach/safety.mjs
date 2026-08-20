// PROTOTYPE — throwaway. Ticket 19, final pass.
// The ticket's question 5 assumes ranged IS safer and asks whether that safety can be SEEN.
// This checks the assumption first: per-capita death rate by type, which is what a player
// actually watches. threat 3.0/1.0/0.5 is set against hp0 70/30/36 -- the two may cancel.
// Run: node safety.mjs
import { DEFAULTS, simulateRun, clone } from '../06-progression-curve/model.mjs';

const base = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.stallMinutes = 1e9;
  return c;
};

function rates(name, mutate) {
  const c = base(); mutate(c);
  const r = simulateRun(c, { maxSeconds: 40 * 60 });
  const tail = r.samples.slice(Math.floor(r.samples.length * 0.6));
  const pop = (ty) => tail.reduce((a, s) => a + s.popByType[ty], 0) / (tail.length || 1);
  const per = (ty) => r.deathsByType[ty] / pop(ty);           // deaths per live climber, per run
  const [m, rg, h] = ['melee', 'ranged', 'healer'].map(per);
  console.log('  ' + name.padEnd(40) + String(r.peak).padStart(5) +
    '   deaths per live climber  ' +
    [m, rg, h].map((v) => v.toFixed(1).padStart(6)).join('') +
    `   melee dies ${(m / rg).toFixed(2)}x as often as ranged`);
}

console.log('\n### Is ranged actually safer? (m/r/h, higher = dies more)');
rates('as shipped — threat 3/1/0.5, hp 70/30/36', () => {});
rates('threat only: ranged threat 1 -> 0.25', (c) => { c.types.ranged.threat = 0.25; });
rates('hp only: ranged hp 30 -> 70 (= melee)', (c) => { c.types.ranged.hp0 = 70; });
rates('both: threat 0.25 and hp 70', (c) => { c.types.ranged.threat = 0.25; c.types.ranged.hp0 = 70; });
rates('threat weighting OFF (all threat 1.0)', (c) => {
  c.types.melee.threat = 1; c.types.ranged.threat = 1; c.types.healer.threat = 1;
});
rates('standoff 2 — positional safety instead', (c) => { c.reach.ranged = 2; c.standoff = true; });
rates('standoff 2 + aura widened to +/-2', (c) => { c.reach.ranged = 2; c.standoff = true; c.heroAuraFloors = 2; });

console.log('\n### Then what DOES set the death rate? (ranged column is the one to watch)');
rates('as shipped', () => {});
rates('ranged hp 30 -> 300 (10x tankier)', (c) => { c.types.ranged.hp0 = 300; });
rates('ranged hp 30 -> 3 (10x squishier)', (c) => { c.types.ranged.hp0 = 3; });
rates('ranged replacement 5s -> 15s', (c) => { c.types.ranged.spawnInterval = 15; });
rates('ranged replacement 5s -> 2s', (c) => { c.types.ranged.spawnInterval = 2; });
rates('ranged cap 30 -> 10', (c) => { c.types.ranged.cap = 10; });
