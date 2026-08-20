import { DEFAULTS, simulateCampaign } from '../06-progression-curve/model.mjs';

// rolling gold/sec over a WINDOW, as a high-water mark
function peakRate(samples, windowSecs) {
  const step = samples[1].t - samples[0].t;
  const n = Math.max(1, Math.round(windowSecs / step));
  let best = 0, bestT = 0;
  for (let i = n; i < samples.length; i++) {
    const r = (samples[i].goldEarned - samples[i - n].goldEarned) / (samples[i].t - samples[i - n].t);
    if (r > best) { best = r; bestT = samples[i].t; }
  }
  return { best, bestT };
}

const runs = simulateCampaign(DEFAULTS, 10, { maxSeconds: 6 * 3600 });
console.log('run  mins   peak  totalGold      peakRate/s  peakAt(min)  run==N min of peakRate');
for (const [i, r] of runs.entries()) {
  const { best, bestT } = peakRate(r.samples, 60);
  const equivMin = best > 0 ? (r.goldEarned / best) / 60 : 0;
  console.log(
    String(i + 1).padStart(3),
    (r.seconds / 60).toFixed(1).padStart(6),
    String(r.peak).padStart(6),
    r.goldEarned.toExponential(2).padStart(10),
    best.toExponential(2).padStart(14),
    (bestT / 60).toFixed(1).padStart(12),
    equivMin.toFixed(1).padStart(10)
  );
}
