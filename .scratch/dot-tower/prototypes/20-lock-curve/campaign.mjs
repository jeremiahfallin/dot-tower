// PROTOTYPE — throwaway. Ticket 20, Q4 + Q5 (+ ticket 14's entity range).
//
// Ten-run campaigns, natural stall cutoff (ticket 06's heuristic -- noisy at
// +-40%, flagged in the ticket answer; the honest run-length instrument is
// ticket 17's problem). Compares the shipped curve against time-priced locks at
// K=30, both with sealed income withdrawn. Reports per run: length, peak, locks,
// cumulative multiplier, head start in floors (h: 1.075^h = M_cum) and its ratio
// to peak; then floor-1000 arrival, and the entity range ticket 14's parked
// device session should test against.
//
// Run: node campaign.mjs
import { DEFAULTS, simulateCampaign, clone } from '../06-progression-curve/model.mjs';

const shipped = () => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.sealedIncome = false;
  return c;
};
const time30 = () => {
  const c = shipped();
  c.lockPricing = 'time'; c.lockTimeCost = 30;
  c.lockFreeBelowBest = true;         // head start is conquered territory
  return c;
};

const HP_BASE = DEFAULTS.enemyHpBase;
const headStart = (M) => Math.log(M) / Math.log(HP_BASE);

for (const [name, cfg] of [['shipped curve (geometric B=4.0)', shipped()], ['time-priced locks (K=30s)', time30()]]) {
  const runs = simulateCampaign(cfg, 10, { maxSeconds: 3 * 3600 });
  console.log(`\n### ${name}`);
  let floor1000 = null, entMax = 0, entSum = 0, entN = 0;
  runs.forEach((r, i) => {
    if (floor1000 === null && r.peak >= 1000) floor1000 = i + 1;
    for (const s of r.samples) { if (s.entities > entMax) entMax = s.entities; entSum += s.entities; entN++; }
    const h = headStart(r.prestigeMultOut);
    console.log(`  run ${String(i + 1).padStart(2)}: ${String(Math.round(r.seconds / 60)).padStart(3)} min` +
      `  peak ${String(r.peak).padStart(5)}  locks ${String(r.lockLevel).padStart(3)}` +
      `  M ${r.prestigeMultOut.toExponential(2)}  head start ${String(Math.round(h)).padStart(4)} floors (${(100 * h / r.peak).toFixed(0)}% of peak)`);
  });
  console.log(`  --> floor 1000 at run ${floor1000 ?? 'never (peak max ' + Math.max(...runs.map((r) => r.peak)) + ')'}` +
    `; entities max ${entMax}, mean ${Math.round(entSum / entN)}`);
}
