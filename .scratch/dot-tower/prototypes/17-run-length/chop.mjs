// PROTOTYPE — throwaway. Ticket 17: what ends a mature run?
//
// Ticket 20 left this ticket a new question: the lock curve no longer ends
// runs (runs 7-10 of a campaign still climb at a 180-min cap), so run length
// is unowned. The one mechanism that could own it is prestige timing itself:
// prestiging banks M_earned(peak) = (1+peak/50)^1.2, and a fresh mature run
// re-reaches ~76% of the old peak quickly through free conquered territory,
// so rapid cycling might out-compound frontier climbing. This sweeps the
// prestige interval X from a mature state under a FIXED total wall-clock
// budget (12h) -- ticket 19's only trusted comparison -- and reports M_cum
// and frontier (best-ever) per hour for each policy. If some X* dominates,
// the model ends runs there; if flat, the ending must be authored elsewhere.
//
// Run: node chop.mjs
import { DEFAULTS, simulateRun, clone } from '../06-progression-curve/model.mjs';

const design = (basis = 'peak') => {
  const c = clone(DEFAULTS);
  c.heroAuraMult = 2.0; c.heroTaunt = true; c.heroStation = 'wall';
  c.sealedIncome = false;
  c.lockPricing = 'time'; c.lockTimeCost = 30;
  c.lockFreeBelowBest = true;         // ticket 20's shipped design
  c.prestigeBasis = basis;            // ticket 17: 'peak' pays for re-conquest (spam engine),
                                      // 'beyondBest' pays for new territory only
  return c;
};

// Warm to maturity: six natural runs (campaign shape from ticket 20's readings
// -- runs 1-6 end naturally, M ~4e6, best-ever ~841).
function warm(basis) {
  const c = design(basis);
  let M = 1, best = 0;
  for (let i = 0; i < 6; i++) {
    const r = simulateRun(c, { prestigeMult: M, maxSeconds: 3 * 3600, bestEverFloor: best });
    M *= r.prestigeMultEarned; best = Math.max(best, r.peak);
  }
  return { M, best };
}

// Play `budget` minutes from (M, best) prestiging every X minutes
// (X = null -> the ticket-06 stall heuristic, i.e. what the game does today).
function play(c, M, best, X, budget) {
  let remaining = budget * 60, runs = 0, wall = 0;
  const cfg = X === null ? c : { ...c, stallMinutes: 1e9 };
  while (remaining > 60) {
    const cap = X === null ? remaining : Math.min(X * 60, remaining);
    const r = simulateRun(cfg, { prestigeMult: M, maxSeconds: cap, bestEverFloor: best });
    M *= r.prestigeMultEarned;
    best = Math.max(best, r.peak);
    wall += r.seconds; remaining -= r.seconds; runs++;
    if (r.seconds <= 0) break; // degenerate guard
  }
  const h = Math.log(M) / Math.log(DEFAULTS.enemyHpBase); // head start in floors
  return { runs, wall: wall / 60, M, best, lnM: Math.log(M), h };
}

for (const basis of ['peak', 'beyondBest']) {
const w = warm(basis);
console.log(`\n===== prestigeBasis: ${basis} =====`);
console.log(`warm state: M = ${w.M.toExponential(2)}, best-ever ${w.best}, head start ${Math.round(Math.log(w.M) / Math.log(DEFAULTS.enemyHpBase))} floors`);
console.log(`12h of play from there, per prestige policy:\n`);

const BUDGET = 12 * 60;
const policies = [
  ['every 5 min', 5], ['every 10 min', 10], ['every 20 min', 20],
  ['every 30 min', 30], ['every 45 min', 45], ['every 60 min', 60],
  ['every 90 min', 90], ['every 2 h', 120], ['every 3 h', 180],
  ['every 4 h', 240], ['at stall (today)', null],
];
console.log('  policy            runs   wall    M_cum          ln M/h    frontier/h   head start');
for (const [label, X] of policies) {
  const p = play(design(basis), w.M, w.best, X, BUDGET);
  const extraWall = BUDGET - p.wall;
  console.log('  ' + label.padEnd(16) +
    String(p.runs).padStart(4) +
    `  ${String(Math.round(p.wall)).padStart(4)}m` +
    `  ${p.M.toExponential(3).padStart(14)}` +
    `  ${(p.lnM / (p.wall / 60)).toFixed(4).padStart(8)}` +
    `  ${((p.best - w.best) / (p.wall / 60)).toFixed(1).padStart(9)}` +
    `  ${String(Math.round(p.h)).padStart(7)}${extraWall > 1 ? `   (unused ${Math.round(extraWall)}m)` : ''}`);
}
}
