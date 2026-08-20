import { DEFAULTS, simulateCampaign, clone } from '../06-progression-curve/model.mjs';
const cfg = clone(DEFAULTS);
cfg.heroAuraMult = 2.0; cfg.heroTaunt = true; cfg.heroStation = 'wall';
const runs = simulateCampaign(cfg, 7, { maxSeconds: 4*3600 });
const out = runs.map((r,i)=>{
  const pts=[]; let lastT=-999;
  for (const s of r.samples){ if (s.t-lastT>=60){ pts.push([+(s.t/60).toFixed(1), s.peak]); lastT=s.t; } }
  pts.push([+(r.seconds/60).toFixed(1), r.peak]);
  const cut=r.seconds-600, s0=r.samples.find(s=>s.t>=cut);
  return { run:i+1, min:+(r.seconds/60).toFixed(1), peak:r.peak, gold:r.gold,
    ranks:r.ranks, heroLevel:r.heroLevel, lockLevel:r.lockLevel,
    Min:r.prestigeMultIn, earned:r.prestigeMultEarned, Mout:r.prestigeMultOut,
    matchPrevMin: r.secondsToMatchPrevPeak!=null? +(r.secondsToMatchPrevPeak/60).toFixed(1):null,
    last10:r.peak-(s0?s0.peak:1), pts };
});
console.log(JSON.stringify(out));
