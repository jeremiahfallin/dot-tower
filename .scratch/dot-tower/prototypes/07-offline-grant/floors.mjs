import { DEFAULTS, simulateRun, prestigeMultFor } from './grant.mjs';

function peakRate(samples, w = 60) {
  const step = samples[1].t - samples[0].t, n = Math.max(1, Math.round(w / step));
  let best = 0;
  for (let i = n; i < samples.length; i++)
    best = Math.max(best, (samples[i].goldEarned - samples[i-n].goldEarned) / (samples[i].t - samples[i-n].t));
  return best;
}

// walk a campaign to get realistic prestige multipliers, then probe run N
const MINUTES = [1, 2, 5, 15, 30, 60, 120, 480, 1440];
let M = 1;
const probes = [1, 4, 8];
for (let i = 1; i <= 8; i++) {
  const base = simulateRun(DEFAULTS, { prestigeMult: M, maxSeconds: 6*3600 });
  if (probes.includes(i)) {
    const rate = peakRate(base.samples);
    const row = [];
    for (const mins of MINUTES) {
      const g = simulateRun(DEFAULTS, { prestigeMult: M, maxSeconds: 8*3600, graceGold: rate * mins * 60 });
      row.push(`${String(mins).padStart(4)}m:+${String(g.peak - g.peakAtGrace).padStart(3)}`);
    }
    console.log(`run ${i}  stall@floor ${String(base.peak).padStart(4)}  peakRate ${rate.toExponential(2)}`);
    console.log('   floors bought:', row.join('  '));
  }
  M *= base.prestigeMultEarned;
}
