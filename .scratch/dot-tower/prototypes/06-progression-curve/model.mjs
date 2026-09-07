// PROTOTYPE — throwaway. Answers ticket 06 "Progression curve, first pass".
// Pure model: no DOM, no timers, no I/O. The page (and node) call in; nothing flows out.

export const DEFAULTS = {
  // --- the tower ---
  enemyHp0: 36, enemyHpBase: 1.075,
  enemyDps0: 2.2, enemyDpsBase: 1.075,
  packSize: 4,
  goldPerKill0: 3, goldBase: 1.095,
  respawnTimer: 20,          // s before a cleared floor repopulates

  // --- climbers (per type) ---
  rankPowerBase: 1.15,       // hp, dps and heal all scale on this
  rankCostBase: 1.30,        // steeper than power, by design
  healScalesWithPrestige: false, // prestige multiplies HEALTH and DAMAGE only
  types: {
    melee:  { hp0: 70, dps0: 5.5, heal0: 0,   cap: 40, rankCost0: 25, spawnInterval: 5.0, threat: 3.0 },
    ranged: { hp0: 30, dps0: 9.0, heal0: 0,   cap: 30, rankCost0: 35, spawnInterval: 5.0, threat: 1.0 },
    healer: { hp0: 36, dps0: 0,   heal0: 3.5, cap: 20, rankCost0: 45, spawnInterval: 5.0, threat: 0.5 },
  },
  climbSpeed: 0.5,           // floors/s across a cleared floor

  // --- reach (ticket 19) ---
  targeting: 'threat',       // 'threat' = incoming split by threat weight (ticket 06's abstraction)
                             // 'line'   = incoming falls front-to-back through the formation
  formation: { melee: 0, ranged: 1, healer: 2 }, // rank in the line under 'line'; 0 is the front
  lineFocus: false,          // within a rank: false spreads the hit across it, true finishes off
                             // the weakest unit first. The forgiving reading against the harsh one.
  reach: { melee: 0, ranged: 0, healer: 0 },     // floors ABOVE its own a type can attack across
  standoff: false,           // does a type with reach halt short of the fighting and fire up?
  failedFloorReset: false,   // ticket 03 says a floor nobody is fighting restores its pack.
                             // The model has never done this; false is what tickets 06-16 measured.
  auraFloors: 1,             // heal reaches same floor +/- N
  healPolicy: 'lowestFraction',        // which targeting policy the healer runs -- see HEAL_POLICIES
  buyPolicy: 'cheapest',     // ticket 15: how the player spends. 'cheapest' is what 06 measured --
                             // buy whatever you can afford, forever. The others are the strategies
                             // a player would actually try, and the spread between them is the
                             // answer to "is gold competition a real decision?"
                             // 'melee' | 'ranged' | 'healer' = rank one type first, always
                             // 'even'  = keep the three ranks level
                             // 'noLock' = cheapest, but never buy a lock
  healSelf: true,            // may a healer be its own target?
  healHero: true,            // is the hero in the candidate set at all?

  // --- hero ---
  heroHp0: 420, heroDps0: 38,
  heroPowerBase: 1.18, heroCost0: 120, heroCostBase: 1.30,
  heroRespawn: 20,
  heroThreat: 5.0,
  heroStation: 'wall',       // 'wall' | 'below' | 'off'
  heroBelowOffset: 6,        // floors below the wall when stationing 'below'
  heroZone: 0,               // floors either side of its post the hero patrols, keeping them clear
  heroAuraMult: 1.0,         // multiplies damage dealt AND gold earned by climbers in the aura
  heroAuraFloors: 0,         // aura reaches its floor +/- N
  heroTaunt: false,          // hero is the primary target on its floor while it lives
  heroRegenPct: 0,           // fraction of max HP regenerated per second

  // --- locking ---
  lockCost0: 500, lockCostBase: 4.0,
  lockMargin: 15,            // only lock 10k when peak floor is this far above it

  // --- prestige ---
  prestigeDivisor: 50, prestigeExponent: 1.2, // M = (1 + peak/divisor)^exponent
  prestigeMode: 'compound', // 'bestPeak' = M is a function of best-ever peak (does not compound)
                            // 'compound' = each run's multiplier multiplies the last
  stallMinutes: 6,           // peak flat this long => the run is over

  // --- entity accounting (ticket 14) ---
  // Ticket 03: a floor is inert DATA until a climber comes "within a floor or
  // two", at which point its pack spawns as entities and despawns behind them.
  // This is that radius, made explicit so the bound can be measured rather than
  // asserted. It affects instrumentation only -- no simulation behaviour reads it.
  activationRadius: 2,
};

const clone = (o) => JSON.parse(JSON.stringify(o));

// ---------- closed-form curves ----------
export const enemyHp   = (c, f) => c.enemyHp0  * Math.pow(c.enemyHpBase,  f - 1);
export const enemyDps  = (c, f) => c.enemyDps0 * Math.pow(c.enemyDpsBase, f - 1);
export const goldPerKill = (c, f) => c.goldPerKill0 * Math.pow(c.goldBase, f - 1);
export const rankCost  = (c, t, r) => c.types[t].rankCost0 * Math.pow(c.rankCostBase, r - 1);
export const lockCost  = (c, k) => c.lockCost0 * Math.pow(c.lockCostBase, k - 1);
export const heroCost  = (c, l) => c.heroCost0 * Math.pow(c.heroCostBase, l - 1);
export const prestigeMultFor = (c, peak) =>
  Math.pow(1 + peak / c.prestigeDivisor, c.prestigeExponent);

const climberHp  = (c, t, r, M) => c.types[t].hp0  * Math.pow(c.rankPowerBase, r - 1) * M;
const climberDps = (c, t, r, M) => c.types[t].dps0 * Math.pow(c.rankPowerBase, r - 1) * M;
// Healer targeting policies. 'aura' treats everyone in radius at once; every other policy
// commits the healer's whole rate to one chosen unit. Ticket 05 settled the aura as the
// default -- this is the seam it promised, not a change to that decision.
export const HEAL_POLICIES = {
  aura: null,                                  // handled inline: all of them, at a rate
  lowestFraction: (us) => us.reduce((a, b) => (b.hp / b.maxHp < a.hp / a.maxHp ? b : a)),
  lowestAbsolute: (us) => us.reduce((a, b) => (b.hp < a.hp ? b : a)),
  nearest: (us, h) => us.reduce((a, b) => {
    const da = Math.abs(a.floor - h.floor), db = Math.abs(b.floor - h.floor);
    return db < da || (db === da && b.hp / b.maxHp < a.hp / a.maxHp) ? b : a;
  }),
};

const climberHeal = (c, t, r, M) =>
  c.types[t].heal0 * Math.pow(c.rankPowerBase, r - 1) * (c.healScalesWithPrestige ? M : 1);

// ---------- one run ----------
export function simulateRun(cfg, { prestigeMult = 1, maxSeconds = 4 * 3600, dt = 0.1, sampleEvery = 10 } = {}) {
  const c = cfg;
  const M = prestigeMult;

  let t = 0, gold = 0, sealedRate = 0;
  let lockLevel = 0;                       // lock line = 10 * lockLevel
  const ranks = { melee: 1, ranged: 1, healer: 1 };
  let heroLevel = 1, heroFloor = 1, heroHp = c.heroHp0 * M, heroDeadUntil = -1;
  let peak = 1, lastPeakGainAt = 0;
  let deaths = 0, kills = 0, goldEarned = 0, healToHero = 0, healToClimbers = 0;
  const deathsByType = { melee: 0, ranged: 0, healer: 0 };
  let heroAliveTicks = 0, heroTicks = 0, heroDeaths = 0;
  let killsInAura = 0, killsTotal = 0;   // kill COUNT, not kill value -- the XP question
  const healByKind = { melee: 0, ranged: 0, healer: 0, hero: 0 };

  const climbers = [];                      // {type, floor, prog, hp}
  const nextSpawn = { melee: 0, ranged: 0, healer: 0 };
  const floors = new Map();                 // f -> {count, poolHp, respawnAt}
  const bandWindow = [];                    // rolling [t, floor, gold] for lock measurement
  const WINDOW = 60;

  const samples = [];
  const events = [];
  const wallHistory = [];

  // Ticket 14 instrumentation. Read-only over `floors`: materialising a floor
  // record here would change the memory profile and the respawn behaviour we
  // are trying to measure, so an untouched floor is counted as a full pack
  // without being inserted.
  const R = c.activationRadius ?? 2;
  let prevActive = new Set();
  let activations = 0, deactivations = 0, churnSince = 0;

  function activeFloorSet() {
    const act = new Set();
    for (const cl of climbers) {
      for (let f = Math.max(lockLine(), cl.floor - R); f <= cl.floor + R; f++) act.add(f);
    }
    if (t >= heroDeadUntil) {
      for (let f = Math.max(lockLine(), heroFloor - R); f <= heroFloor + R; f++) act.add(f);
    }
    return act;
  }

  function enemyEntitiesIn(act) {
    let n = 0;
    for (const f of act) {
      const st = floors.get(f);
      n += st ? st.count : c.packSize;   // untouched floor => full pack, not materialised
    }
    return n;
  }

  const lockLine = () => 10 * lockLevel + (lockLevel > 0 ? 1 : 0);

  function floorState(f) {
    let s = floors.get(f);
    if (!s) { s = { count: c.packSize, poolHp: c.packSize * enemyHp(c, f), respawnAt: -1 }; floors.set(f, s); }
    return s;
  }

  function credit(f, g) {
    gold += g; goldEarned += g;
    bandWindow.push([t, f, g]);
    if (bandWindow.length > 20000) bandWindow.splice(0, 10000); // rolling, bounded
  }

  function purchase() {
    // Greedy: buy the cheapest thing you can afford, forever. What a real player does.
    for (let guard = 0; guard < 50; guard++) {
      const opts = [];
      for (const ty of ['melee', 'ranged', 'healer']) opts.push({ kind: 'rank', ty, cost: rankCost(c, ty, ranks[ty]) });
      opts.push({ kind: 'hero', cost: heroCost(c, heroLevel) });
      const nextLockFloor = 10 * (lockLevel + 1);
      if (peak >= nextLockFloor + c.lockMargin) opts.push({ kind: 'lock', cost: lockCost(c, lockLevel + 1), floor: nextLockFloor });
      opts.sort((a, b) => a.cost - b.cost);
      // The spending strategy reorders the options; affordability still decides.
      const pol = c.buyPolicy;
      if (pol === 'noLock') { const i = opts.findIndex((o) => o.kind === 'lock'); if (i >= 0) opts.splice(i, 1); }
      else if (pol === 'even') {
        opts.sort((a, b) => (a.kind === 'rank' ? ranks[a.ty] : 1e9) - (b.kind === 'rank' ? ranks[b.ty] : 1e9));
      } else if (pol === 'melee' || pol === 'ranged' || pol === 'healer') {
        opts.sort((a, b) => (a.kind === 'rank' && a.ty === pol ? -1 : 0) - (b.kind === 'rank' && b.ty === pol ? -1 : 0));
      }
      const pick = opts.find((o) => o.cost <= gold);
      if (!pick) return;
      gold -= pick.cost;
      if (pick.kind === 'rank') ranks[pick.ty]++;
      else if (pick.kind === 'hero') heroLevel++;
      else doLock(pick.floor);
    }
  }

  function doLock(newLine) {
    // Measure what the band below the new line actually produced, then freeze it.
    const cutoff = t - WINDOW;
    let sum = 0;
    for (const [ts, f, g] of bandWindow) if (ts >= cutoff && f < newLine) sum += g;
    const measured = sum / WINDOW;
    sealedRate += measured;
    lockLevel = newLine / 10;
    // trailing climbers sprint to the new entry floor (ticket 03)
    for (const cl of climbers) if (cl.floor < lockLine()) { cl.floor = lockLine(); cl.prog = 0; }
    for (const f of [...floors.keys()]) if (f < lockLine()) floors.delete(f);
    if (heroFloor < lockLine()) heroFloor = lockLine();
    events.push({ t, kind: 'lock', floor: newLine, sealedRate, measured });
  }

  function currentWall() {
    // Where the column piles up: the highest floor with live climbers that still has enemies.
    let w = lockLine();
    for (const cl of climbers) if (cl.floor > w) w = cl.floor;
    return w;
  }

  let lastDecision = 0, lastSample = 0;

  while (t < maxSeconds) {
    // --- spawn ---
    for (const ty of ['melee', 'ranged', 'healer']) {
      const alive = climbers.reduce((n, cl) => n + (cl.type === ty ? 1 : 0), 0);
      if (alive < c.types[ty].cap && t >= nextSpawn[ty]) {
        const hp = climberHp(c, ty, ranks[ty], M);
        climbers.push({ type: ty, floor: lockLine(), prog: 0, hp, maxHp: hp });
        nextSpawn[ty] = t + c.types[ty].spawnInterval;
      }
    }

    // --- respawns ---
    for (const [f, s] of floors) {
      if (s.count === 0 && s.respawnAt >= 0 && t >= s.respawnAt) {
        s.count = c.packSize; s.poolHp = c.packSize * enemyHp(c, f); s.respawnAt = -1;
      }
    }

    // --- group by floor ---
    const byFloor = new Map();
    for (const cl of climbers) {
      if (!byFloor.has(cl.floor)) byFloor.set(cl.floor, []);
      byFloor.get(cl.floor).push(cl);
    }
    const heroAlive = c.heroStation !== 'off' && t >= heroDeadUntil;
    if (c.heroStation !== 'off') { heroTicks++; if (heroAlive) heroAliveTicks++; }
    // The hero holds a stretch, not a point: each tick it fights the lowest floor in its zone
    // that still has enemies, so floors behind it stay clear and climbers transit them free.
    let heroTarget = heroFloor;
    if (heroAlive && c.heroZone > 0) {
      for (let f = Math.max(lockLine(), heroFloor - c.heroZone); f <= Math.min(peak, heroFloor + c.heroZone); f++) {
        const s2 = floors.get(f);
        if (s2 ? s2.count > 0 : true) { heroTarget = f; break; }
      }
    }

    // --- reach: who is shooting at which floor (ticket 19) ---
    // A climber attacks the NEAREST floor with enemies at or above its own, within its type's
    // reach. Its own floor wins, and nobody attacks two floors, so reach buys range, never output.
    const attackers = new Map();                 // floor -> climbers firing INTO it
    for (const cl of climbers) {
      const r = c.reach[cl.type] || 0;
      for (let d = 0; d <= r; d++) {
        const f = cl.floor + d;
        const s2 = floors.get(f);
        if (s2 && s2.count === 0) continue;      // an unvisited floor still holds a full pack
        if (!attackers.has(f)) attackers.set(f, []);
        attackers.get(f).push(cl);
        break;
      }
    }

    // --- combat, floor by floor ---
    const contested = new Set([...attackers.keys()]);
    if (heroAlive) contested.add(heroTarget);
    for (const f of contested) {
      const s = floorState(f);
      if (s.count === 0) continue;
      const group = attackers.get(f) || [];      // who is shooting at this floor
      const standing = byFloor.get(f) || [];     // who is standing in it, and can therefore be hit
      const heroHere = heroAlive && heroTarget === f;
      if (group.length === 0 && !heroHere) continue;

      // The hero's aura multiplies what climbers already do. It is keyed on where the CLIMBER
      // stands, not on what it is firing at -- so standing off walks out of the aura.
      const auraOn = heroAlive && c.heroAuraMult > 1;
      const inAura = (fl) => auraOn && Math.abs(fl - heroFloor) <= c.heroAuraFloors;
      let dps = 0, goldWeighted = 0;
      for (const cl of group) {
        const m = inAura(cl.floor) ? c.heroAuraMult : 1;
        const d = climberDps(c, cl.type, ranks[cl.type], M) * m;
        dps += d; goldWeighted += d * m;
      }
      if (heroHere) {
        const m = inAura(heroFloor) ? c.heroAuraMult : 1;
        const d = c.heroDps0 * Math.pow(c.heroPowerBase, heroLevel - 1) * M;
        dps += d; goldWeighted += d * m;
      }
      const goldMult = dps > 0 ? goldWeighted / dps : 1;

      // damage the pack
      const eHp = enemyHp(c, f);
      const before = s.count;
      s.poolHp -= dps * dt;
      if (s.poolHp <= 0) { s.poolHp = 0; s.count = 0; s.respawnAt = t + c.respawnTimer; }
      else s.count = Math.ceil(s.poolHp / eHp);
      const killed = before - s.count;
      if (killed > 0) {
        kills += killed; credit(f, killed * goldPerKill(c, f) * goldMult);
        killsTotal += killed;
        if (heroAlive && Math.abs(f - heroFloor) <= c.heroAuraFloors) killsInAura += killed;
      }

      // The pack hits back -- and it has no reach of its own. Only what STANDS here is hit.
      const incoming = ((before + s.count) / 2) * enemyDps(c, f) * dt;
      const hitHero = (d) => {
        heroHp -= d;
        if (heroHp <= 0) { heroDeaths++; heroDeadUntil = t + c.heroRespawn;
          heroHp = c.heroHp0 * Math.pow(c.heroPowerBase, heroLevel - 1) * M; }
      };
      // Taunt: the hero eats the floor's damage instead of sharing it out.
      if (c.heroTaunt && heroHere) { hitHero(incoming); continue; }

      if (c.targeting === 'line') {
        // Front-to-back: the pack hits the nearest rank of the formation and spills backward
        // only once that rank has nothing left to absorb. Position IS the protection.
        const tiers = new Map();
        const push = (k, u) => { if (!tiers.has(k)) tiers.set(k, []); tiers.get(k).push(u); };
        for (const cl of standing) push(c.formation[cl.type] ?? 0, { get: () => cl.hp, hit: (d) => { cl.hp -= d; } });
        if (heroHere) push(0, { get: () => heroHp, hit: hitHero });
        let remaining = incoming;
        for (const k of [...tiers.keys()].sort((a, b) => a - b)) {
          if (remaining <= 0) break;
          const tier = tiers.get(k).filter((u) => u.get() > 0);
          const pool = tier.reduce((a, u) => a + u.get(), 0);
          if (pool <= 0) continue;
          const take = Math.min(remaining, pool);
          if (c.lineFocus) {
            // The pack concentrates on whoever is nearest to falling, rather than sharing out.
            let left = take;
            for (const u of tier.sort((a, b) => a.get() - b.get())) {
              if (left <= 0) break;
              const bite = Math.min(left, u.get());
              u.hit(bite); left -= bite;
            }
          } else {
            for (const u of tier) u.hit(take * (u.get() / pool));
          }
          remaining -= take;
        }
      } else {
        let totalThreat = 0;
        for (const cl of standing) totalThreat += c.types[cl.type].threat;
        if (heroHere) totalThreat += c.heroThreat;
        if (totalThreat > 0) {
          for (const cl of standing) cl.hp -= incoming * (c.types[cl.type].threat / totalThreat);
          if (heroHere) hitHero(incoming * (c.heroThreat / totalThreat));
        }
      }
    }

    // --- a floor nobody is fighting recovers (ticket 03's rule, off by default) ---
    if (c.failedFloorReset) {
      for (const [f, s] of floors) {
        if (s.count === 0 || contested.has(f)) continue;
        const full = c.packSize * enemyHp(c, f);
        if (s.poolHp < full) { s.count = c.packSize; s.poolHp = full; }
      }
    }

    // --- healing, through the targeting seam ---
    const healRate = climberHeal(c, 'healer', ranks.healer, M) * dt;
    if (healRate > 0) {
      const heroMax = c.heroHp0 * Math.pow(c.heroPowerBase, heroLevel - 1) * M;
      // One candidate list per tick. The hero is a valid target with no special-casing.
      const units = [];
      for (const cl of climbers) units.push({ cl, floor: cl.floor, hp: cl.hp, maxHp: cl.maxHp, hero: false, kind: cl.type });
      if (heroAlive && c.healHero) units.push({ cl: null, floor: heroFloor, hp: heroHp, maxHp: heroMax, hero: true, kind: 'hero' });
      const perFloor = new Map();
      for (const u of units) {
        if (!perFloor.has(u.floor)) perFloor.set(u.floor, []);
        perFloor.get(u.floor).push(u);
      }
      const apply = (u, amount) => {
        const got = Math.min(amount, u.maxHp - u.hp);
        if (got <= 0) return;
        u.hp += got;
        if (u.hero) { heroHp += got; healToHero += got; } else { u.cl.hp += got; healToClimbers += got; }
        healByKind[u.kind] += got;
      };
      const policy = HEAL_POLICIES[c.healPolicy];
      for (const h of climbers) {
        if (h.type !== 'healer') continue;
        const inRange = [];
        for (let d = -c.auraFloors; d <= c.auraFloors; d++) {
          for (const u of perFloor.get(h.floor + d) || []) {
            if (u.hp >= u.maxHp) continue;
            if (!c.healSelf && u.cl === h) continue;
            inRange.push(u);
          }
        }
        if (!inRange.length) continue;
        if (!policy) for (const u of inRange) apply(u, healRate);   // the aura
        else apply(policy(inRange, h), healRate);                    // one chosen unit
      }
    }

    // --- hero regeneration ---
    if (heroAlive && c.heroRegenPct > 0) {
      const heroMax = c.heroHp0 * Math.pow(c.heroPowerBase, heroLevel - 1) * M;
      heroHp = Math.min(heroMax, heroHp + heroMax * c.heroRegenPct * dt);
    }

    // --- deaths ---
    for (let i = climbers.length - 1; i >= 0; i--) {
      if (climbers[i].hp <= 0) { deathsByType[climbers[i].type]++; climbers.splice(i, 1); deaths++; }
    }

    // --- movement: a cleared floor is walked across ---
    for (const cl of climbers) {
      const s = floorState(cl.floor);
      if (s.count !== 0) continue;
      // Standoff: a type with reach stops as soon as the fighting comes INTO reach, rather
      // than walking into it. This is what turns statistical safety into positional safety.
      if (c.standoff) {
        let held = false;
        for (let d = 1; d <= (c.reach[cl.type] || 0); d++) {
          const s2 = floors.get(cl.floor + d);
          if (s2 ? s2.count > 0 : true) { held = true; break; }
        }
        if (held) continue;
      }
      cl.prog += c.climbSpeed * dt;
      if (cl.prog >= 1) { cl.prog = 0; cl.floor++; if (cl.floor > peak) { peak = cl.floor; lastPeakGainAt = t; } }
    }

    // --- sealed income ---
    if (sealedRate > 0) { const g = sealedRate * M * dt; gold += g; goldEarned += g; }

    // --- hero station ---
    if (c.heroStation !== 'off') {
      const w = currentWall();
      const target = c.heroStation === 'wall' ? w : Math.max(lockLine(), w - c.heroBelowOffset);
      if (target !== heroFloor && t >= heroDeadUntil) heroFloor = target;
    }

    // --- decisions ---
    if (t - lastDecision >= 1) { purchase(); lastDecision = t; }

    // --- entity churn (ticket 14) ---
    // Counted every tick, not every sample: activation cost is paid per
    // transition, so sampling it at 10s intervals would miss almost all of it.
    {
      const act = activeFloorSet();
      for (const f of act) if (!prevActive.has(f)) activations++;
      for (const f of prevActive) if (!act.has(f)) deactivations++;
      prevActive = act;
    }

    // --- sampling ---
    if (t - lastSample >= sampleEvery) {
      const w = currentWall();
      const atWall = climbers.filter((cl) => cl.floor >= w - 1);
      const wallHp = atWall.length ? atWall.reduce((a, cl) => a + cl.hp / cl.maxHp, 0) / atWall.length : 1;
      // per type: how many are alive, how healthy they are, and how far back they stand
      const popByType = { melee: 0, ranged: 0, healer: 0 };
      const hpByType = { melee: 0, ranged: 0, healer: 0 };
      const backByType = { melee: 0, ranged: 0, healer: 0 };
      for (const cl of climbers) {
        popByType[cl.type]++;
        hpByType[cl.type] += cl.hp / cl.maxHp;
        backByType[cl.type] += w - cl.floor;
      }
      for (const ty of ['melee', 'ranged', 'healer']) {
        const n = popByType[ty] || 1;
        hpByType[ty] /= n; backByType[ty] /= n;
      }
      // --- ticket 14: what this instant would cost as entities ---
      const act = activeFloorSet();
      const enemyEntities = enemyEntitiesIn(act);
      const occupied = new Set(climbers.map((cl) => cl.floor));
      let contested = 0;
      for (const f of occupied) {
        const st = floors.get(f);
        if (!st || st.count > 0) contested++;   // climbers present AND enemies present
      }
      let lo = Infinity, hi = -Infinity;
      for (const cl of climbers) { if (cl.floor < lo) lo = cl.floor; if (cl.floor > hi) hi = cl.floor; }
      const elapsed = Math.max(t - churnSince, dt);

      samples.push({
        t, gold, goldEarned, peak, wall: w, wallHp, lockLine: lockLine(), sealedRate,
        pop: climbers.length, popByType, hpByType, backByType, deaths, kills,
        ranks: { ...ranks }, heroLevel, heroFloor,
        gps: goldEarned / Math.max(t, 1),
        // ticket 14
        activeFloors: act.size,
        occupiedFloors: occupied.size,
        contestedFloors: contested,
        climberSpread: climbers.length ? hi - lo + 1 : 0,
        enemyEntities,
        entities: enemyEntities + climbers.length + (t >= heroDeadUntil ? 1 : 0),
        floorRecords: floors.size,
        activationsPerSec: activations / elapsed,
        deactivationsPerSec: deactivations / elapsed,
      });
      activations = 0; deactivations = 0; churnSince = t;
      wallHistory.push(currentWall());
      lastSample = t;
    }

    // --- run over? ---
    if (t - lastPeakGainAt > c.stallMinutes * 60 && t > 300) break;

    t += dt;
  }

  return {
    seconds: t, gold, goldEarned, peak, deaths, deathsByType, kills, healToHero, healToClimbers, healByKind,
    heroUptime: heroTicks ? heroAliveTicks / heroTicks : 1, heroDeaths, killsInAura, killsTotal,
    lockLevel, lockLine: lockLine(), sealedRate, ranks: { ...ranks }, heroLevel,
    prestigeMultEarned: prestigeMultFor(c, peak),
    samples, events, pop: climbers.length,
  };
}

// ---------- a campaign of runs ----------
export function simulateCampaign(cfg, runs = 3, opts = {}) {
  const out = [];
  let M = 1, bestPeak = 0;
  for (let i = 0; i < runs; i++) {
    const r = simulateRun(cfg, { ...opts, prestigeMult: M });
    // how long did this run take to match the PREVIOUS run's peak?
    if (i > 0) {
      const prevPeak = out[i - 1].peak;
      const s = r.samples.find((s) => s.peak >= prevPeak);
      r.secondsToMatchPrevPeak = s ? s.t : null;
    }
    r.prestigeMultIn = M;
    out.push(r);
    bestPeak = Math.max(bestPeak, r.peak);
    M = cfg.prestigeMode === 'compound' ? M * r.prestigeMultEarned : prestigeMultFor(cfg, bestPeak);
    r.prestigeMultOut = M;
  }
  return out;
}

export { clone };
