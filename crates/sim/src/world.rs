//! The simulation. One [`World::step`] is one 10Hz tick.
//!
//! # What this is a port of, and what was changed on the way
//!
//! The throwaway JS model at `.scratch/dot-tower/prototypes/06-progression-curve/model.mjs`
//! produced nearly every number the resolved tickets quote. This is that model
//! moved into the shipping language, because
//! [ticket 19](../../../.scratch/dot-tower/issues/19-ranged-attack-reach.md)
//! caught it *inverting a headline result* and ticket 14 found it had never
//! implemented ticket 03's floor-reset rule at all. An instrument used to defend
//! decisions has to be checkable against them, and a second implementation in a
//! language with types is the cheapest check available.
//!
//! Deliberate departures, each of them a settled decision rather than a
//! liberty taken:
//!
//! - **Rejected mechanics are not ported.** Attack reach, standoff, formation
//!   order and line targeting were measured and rejected by ticket 19
//!   ([ADR 0010](../../../docs/adr/0010-combat-is-floor-local-and-has-no-reach.md)).
//!   Combat is floor-local, so the model's `attackers` map collapses into "who
//!   is standing here", which is why no such map appears below.
//! - **The player is not part of the world.** The model had a buy policy, a
//!   hero-stationing rule and a stall heuristic baked into the tick. Those are
//!   a *scripted player* and they live behind [`Player`]. The shipping game's
//!   player is taps on a screen; the harness's is
//!   [`crate::autoplay::Greedy`]. Ticket 18 owns the stall condition and it is
//!   still open, so nothing in here claims to know when a run is over.
//! - **Iteration order is pinned.** JS objects and `Map`s iterate in insertion
//!   order and several tie-breaks resolve on "first wins", so a `HashMap` where
//!   the model had a `Map` would silently change results. Ordering is explicit
//!   wherever it is load-bearing, and said so.

use crate::curves::*;
use crate::geometry::{Floor, FloorPos};
use crate::metrics::{LockEvent, Sample, Totals};
use crate::save::{Account, HeroProgress, Run, SLICE_HERO};
use crate::tuning::{HealPolicy, Tuning};
use crate::types::{ClimberType, PerType};
use std::collections::{HashMap, HashSet};

/// The fixed timestep, in seconds. Ticket 03 settled 10Hz: frame-rate
/// independence across a 144Hz desktop and a throttled phone is non-negotiable,
/// and the coarse rate is 6× cheaper on mobile.
pub const DT: f64 = 0.1;

/// One live climber. Anonymous by design — `CONTEXT.md` is explicit that
/// individual climbers are never named, tracked or directly commanded, so this
/// carries no identity beyond its type.
#[derive(Debug, Clone)]
pub struct Climber {
    pub ty: ClimberType,
    pub pos: FloorPos,
    pub hp: f64,
    pub max_hp: f64,
}

/// A floor's enemies. Ticket 03: a floor is inert *data* until a climber comes
/// near, and only the count and the timer survive deactivation — per-enemy
/// health does not persist.
#[derive(Debug, Clone)]
pub struct FloorState {
    pub count: u32,
    /// The pack's health as one pool. Individual enemies are not entities until
    /// they need to be drawn.
    pub pool_hp: f64,
    /// When the pack returns, or `None` if it is not cleared.
    pub respawn_at: Option<f64>,
}

/// Who a healer is mending. Indices into the live climber list, which is
/// stable for the length of one tick.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HealTarget {
    Climber(usize),
    Hero,
}

struct HealUnit {
    target: HealTarget,
    floor: Floor,
    hp: f64,
    max_hp: f64,
}

/// Applies healing to both the snapshot and the real value, so the two cannot
/// drift within a tick. Free rather than a method because it needs three
/// disjoint pieces of the world at once.
fn apply_heal(
    units: &mut [HealUnit],
    climbers: &mut [Climber],
    hero_hp: &mut f64,
    totals: &mut Totals,
    i: usize,
    amount: f64,
) {
    let got = amount.min(units[i].max_hp - units[i].hp);
    if got <= 0.0 {
        return;
    }
    units[i].hp += got;
    match units[i].target {
        HealTarget::Climber(c) => {
            climbers[c].hp += got;
            totals.heal_to_climbers += got;
        }
        HealTarget::Hero => {
            *hero_hp += got;
            totals.heal_to_hero += got;
        }
    }
}

/// What the simulation asks of whoever is playing.
///
/// Both methods are called at the exact points in the tick the model spent its
/// numbers at, so a scripted player reproduces them and a human player slots
/// into the same seam.
pub trait Player {
    /// Where the hero should stand, called every tick. `None` leaves it be.
    fn station(&mut self, _world: &World) -> Option<Floor> {
        None
    }

    /// Called once per simulated second. Spend through [`World`]'s purchase
    /// methods — they are the only way gold leaves the run.
    fn spend(&mut self, _world: &mut World) {}
}

/// A player who does nothing. The simulation runs without one.
pub struct Idle;
impl Player for Idle {}

pub struct World {
    tuning: Tuning,

    // --- the clock ---
    t: f64,

    /// Everything prestige clears, and the thing the save writes.
    ///
    /// Held as the struct rather than as loose fields deliberately: ticket 12
    /// made prestige `run = Run::default()`, and that only holds if the world's
    /// idea of a run and the save's are the same declaration. A cached copy of
    /// any field here would be the drift that ticket was written against.
    run: Run,
    /// Everything that outlives a prestige.
    account: Account,

    // --- the hero's live state, which is never saved ---
    hero_hp: f64,
    /// `Some(t)` while dead. Uptime is the hero's whole contribution (ticket 16).
    hero_dead_until: Option<f64>,

    // --- the active band ---
    climbers: Vec<Climber>,
    floors: HashMap<Floor, FloorState>,
    next_replacement: f64,
    last_decision: f64,

    // --- sealed income, withdrawn by ticket 06 but kept falsifiable ---
    sealed_rate: f64,
    /// Rolling `(t, floor, gold)`. Only maintained when `sealed_income` is on;
    /// it exists solely to measure a band at lock time.
    band_window: Vec<(f64, Floor, f64)>,

    /// Gold credited in each of the last 600 ticks, as a ring buffer, with its
    /// running sum. Ticket 07's best rate is the high-water mark of the rolling
    /// 60-second average, and a high-water mark cannot be recovered from a
    /// coarser record after the fact — so it is accumulated exactly, per tick,
    /// rather than sampled.
    gold_ring: Box<[f64; RATE_WINDOW_TICKS]>,
    gold_ring_at: usize,
    gold_ring_sum: f64,
    gold_this_tick: f64,

    // --- instrumentation ---
    totals: Totals,
    locks: Vec<LockEvent>,
    /// When the next lock first became permitted by depth. `None` means the
    /// climb has not yet earned it.
    margin_eligible_since: Option<f64>,
    last_peak_gain_at: f64,
    prev_active: HashSet<Floor>,
    activations: u64,
    deactivations: u64,
    churn_since: f64,
    cap_skips: u64,
    hero_ticks: u64,
    hero_alive_ticks: u64,
    /// Scratch reused every tick so a 4-hour run does not allocate 144,000 times.
    scratch_order: Vec<Floor>,
    scratch_seen: HashSet<Floor>,
}

const WINDOW: f64 = 60.0;

/// Ticket 07's rolling window, in ticks.
const RATE_WINDOW_TICKS: usize = (WINDOW / DT) as usize;

impl World {
    /// A fresh run. `prestige_mult` is the account's cumulative multiplier —
    /// `1.0` for a brand-new account.
    pub fn new(tuning: Tuning, prestige_mult: f64) -> Self {
        let hero_hp = tuning.hero_hp0 * prestige_mult;
        Self {
            tuning,
            t: 0.0,
            run: Run {
                // The slice's one hero exists from the first frame, stationed
                // at the bottom of the tower.
                heroes: std::iter::once((SLICE_HERO.to_string(), HeroProgress::default()))
                    .collect(),
                stationed: Some((SLICE_HERO.to_string(), 1)),
                ..Run::default()
            },
            account: Account { cumulative_prestige_mult: prestige_mult, ..Account::default() },
            hero_hp,
            hero_dead_until: None,
            climbers: Vec::new(),
            floors: HashMap::new(),
            next_replacement: 0.0,
            last_decision: 0.0,
            sealed_rate: 0.0,
            band_window: Vec::new(),
            gold_ring: Box::new([0.0; RATE_WINDOW_TICKS]),
            gold_ring_at: 0,
            gold_ring_sum: 0.0,
            gold_this_tick: 0.0,
            totals: Totals::default(),
            locks: Vec::new(),
            margin_eligible_since: None,
            last_peak_gain_at: 0.0,
            prev_active: HashSet::new(),
            activations: 0,
            deactivations: 0,
            churn_since: 0.0,
            cap_skips: 0,
            hero_ticks: 0,
            hero_alive_ticks: 0,
            scratch_order: Vec::new(),
            scratch_seen: HashSet::new(),
        }
    }

    /// Rebuilds a world from a save.
    ///
    /// The active band is not restored, because it is not in the save: the
    /// stream respawns at the lock line and re-walks, and ticket 07 already
    /// ruled that re-transit part of what offline gold pays for. The hero comes
    /// back **alive, on its stationed floor, with every ability ready** — which
    /// is technically exploitable by relaunching, and does not matter, because
    /// there are no leaderboards and relaunching to save one cooldown is more
    /// tedious than waiting.
    pub fn resume(tuning: Tuning, account: Account, run: Run) -> Self {
        let mut w = Self::new(tuning, account.cumulative_prestige_mult);
        w.run = run;
        w.account = account;
        if w.run.stationed.is_none() {
            w.set_hero_floor(w.lock_line().max(1));
        }
        w.hero_hp = w.hero_max_hp();
        w.hero_dead_until = None;
        w
    }

    // ----------------------------------------------------------------- reads

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }
    pub fn elapsed(&self) -> f64 {
        self.t
    }
    pub fn gold(&self) -> f64 {
        self.run.gold
    }
    pub fn peak(&self) -> Floor {
        self.run.peak
    }
    pub fn ranks(&self) -> PerType<u32> {
        self.run.ranks
    }
    pub fn hero_level(&self) -> u32 {
        self.hero().level
    }

    fn hero(&self) -> HeroProgress {
        self.run.heroes.get(SLICE_HERO).copied().unwrap_or_default()
    }

    /// Read-only view of what the save would write.
    pub fn run(&self) -> &Run {
        &self.run
    }
    pub fn account(&self) -> &Account {
        &self.account
    }
    pub fn hero_floor(&self) -> Floor {
        self.run.stationed.as_ref().map_or(1, |(_, f)| *f)
    }
    pub fn hero_alive(&self) -> bool {
        self.hero_dead_until.is_none_or(|until| self.t >= until)
    }
    pub fn climbers(&self) -> &[Climber] {
        &self.climbers
    }
    pub fn floors(&self) -> &HashMap<Floor, FloorState> {
        &self.floors
    }
    pub fn lock_level(&self) -> u32 {
        self.run.lock_level
    }
    pub fn totals(&self) -> &Totals {
        &self.totals
    }
    /// Ticket 07's best rate — the highest sustained gold/sec this run.
    pub fn best_rate(&self) -> f64 {
        self.run.best_rate
    }
    /// Every lock bought this run, in order.
    pub fn locks(&self) -> &[LockEvent] {
        &self.locks
    }
    pub fn seconds_since_peak_gain(&self) -> f64 {
        self.t - self.last_peak_gain_at
    }

    /// The highest locked floor. Climbers spawn here and nothing below it is
    /// rendered or simulated.
    pub fn lock_line(&self) -> Floor {
        10 * self.run.lock_level + if self.run.lock_level > 0 { 1 } else { 0 }
    }

    /// The floor the next lock would seal, and what it costs.
    pub fn next_lock(&self) -> (Floor, f64) {
        (10 * (self.run.lock_level + 1), lock_cost(&self.tuning, self.run.lock_level + 1))
    }

    /// Where the column piles up: the highest floor holding live climbers.
    ///
    /// This is the **wall** only once the climb has stalled — they coincide most
    /// of the time but not at the interesting moment, which is why the map still
    /// owes a word for the moving edge of the climb.
    pub fn frontier(&self) -> Floor {
        self.climbers.iter().map(|c| c.pos.floor).max().unwrap_or(self.lock_line()).max(self.lock_line())
    }

    /// Gold per second averaged over the whole run so far.
    pub fn gold_rate(&self) -> f64 {
        self.totals.gold_earned / self.t.max(1.0)
    }

    // ------------------------------------------------------------- purchases

    /// Buys the next rank of `ty` if it is affordable. Returns whether it was.
    pub fn buy_rank(&mut self, ty: ClimberType) -> bool {
        let cost = rank_cost(&self.tuning, ty, self.run.ranks[ty]);
        if cost > self.run.gold {
            return false;
        }
        self.run.gold -= cost;
        self.run.ranks[ty] += 1;
        true
    }

    /// See [`hero_cost`] on why this is the throwaway model's hero and not the
    /// shipping one.
    pub fn buy_hero_level(&mut self) -> bool {
        let cost = hero_cost(&self.tuning, self.hero_level());
        if cost > self.run.gold {
            return false;
        }
        self.run.gold -= cost;
        self.run.heroes.entry(SLICE_HERO.to_string()).or_default().level += 1;
        true
    }

    /// Buys the next lock. Fails if it is unaffordable *or* if the climb has
    /// not yet cleared it by [`Tuning::lock_margin`].
    pub fn buy_lock(&mut self) -> bool {
        let (floor, cost) = self.next_lock();
        if self.run.peak < floor + self.tuning.lock_margin || cost > self.run.gold {
            return false;
        }
        self.run.gold -= cost;
        self.do_lock(floor);
        true
    }

    fn do_lock(&mut self, new_line: Floor) {
        self.locks.push(LockEvent {
            t: self.t,
            floor: new_line,
            cost: lock_cost(&self.tuning, self.run.lock_level + 1),
            gold_wait: self.margin_eligible_since.map_or(0.0, |since| self.t - since),
        });
        self.margin_eligible_since = None;

        // Ticket 03 froze a sealed band's measured output as passive income;
        // ticket 06 withdrew it after finding four of eleven locks froze a rate
        // of exactly zero. Off by default, and measured here only when asked.
        if self.tuning.sealed_income {
            let cutoff = self.t - WINDOW;
            let sum: f64 = self
                .band_window
                .iter()
                .filter(|(ts, f, _)| *ts >= cutoff && *f < new_line)
                .map(|(_, _, g)| g)
                .sum();
            self.sealed_rate += sum / WINDOW;
        }

        self.run.lock_level = new_line / 10;
        let line = self.lock_line();

        // Trailing climbers sprint to the new entry floor (ticket 03). Nothing
        // disappears, so nothing needs explaining — a disappearance is a thing
        // the player would have to be taught, a sprint is not.
        for cl in &mut self.climbers {
            if cl.pos.floor < line {
                cl.pos = FloorPos::entering(line);
            }
        }
        self.floors.retain(|f, _| *f >= line);
        if self.hero_floor() < line {
            self.set_hero_floor(line);
        }
    }

    /// Stations the hero. Refused while it is dead — respawn happens on its
    /// post, and moving a corpse would hand the player a way to dodge the
    /// downtime that *is* the throttle.
    pub fn station_hero(&mut self, floor: Floor) {
        if self.hero_alive() {
            let line = self.lock_line();
            self.set_hero_floor(floor.max(line));
        }
    }

    // ------------------------------------------------------------------ tick

    fn set_hero_floor(&mut self, floor: Floor) {
        self.run.stationed = Some((SLICE_HERO.to_string(), floor));
    }

    fn hero_max_hp(&self) -> f64 {
        self.tuning.hero_hp0 * self.tuning.hero_power_base.powf(self.hero_level() as f64 - 1.0)
            * self.account.cumulative_prestige_mult
    }

    fn climber_hp(&self, ty: ClimberType, rank: u32) -> f64 {
        self.tuning.types.get(ty).hp0 * self.tuning.rank_power_base.powf(rank as f64 - 1.0)
            * self.account.cumulative_prestige_mult
    }

    fn climber_dps(&self, ty: ClimberType, rank: u32) -> f64 {
        self.tuning.types.get(ty).dps0 * self.tuning.rank_power_base.powf(rank as f64 - 1.0)
            * self.account.cumulative_prestige_mult
    }

    fn climber_heal(&self, ty: ClimberType, rank: u32) -> f64 {
        let m = if self.tuning.heal_scales_with_prestige { self.account.cumulative_prestige_mult } else { 1.0 };
        self.tuning.types.get(ty).heal0 * self.tuning.rank_power_base.powf(rank as f64 - 1.0) * m
    }

    fn floor_state(&mut self, f: Floor) -> &mut FloorState {
        let t = &self.tuning;
        self.floors.entry(f).or_insert_with(|| FloorState {
            count: t.pack_size,
            pool_hp: t.pack_size as f64 * enemy_hp(t, f),
            respawn_at: None,
        })
    }

    fn credit(&mut self, f: Floor, g: f64) {
        self.run.gold += g;
        self.totals.gold_earned += g;
        self.gold_this_tick += g;
        if self.tuning.sealed_income {
            self.band_window.push((self.t, f, g));
            if self.band_window.len() > 20_000 {
                self.band_window.drain(0..10_000);
            }
        }
    }

    /// One 10Hz tick.
    pub fn step(&mut self, player: &mut impl Player) {
        // Aliveness is decided ONCE per tick and then carried. The hero can die
        // partway through combat, and re-reading it would mean its aura applied
        // to the floors resolved before it fell and not to those after — an
        // ordering artefact rather than a rule. Harmless while the aura
        // multiplier is 1.0, which is every account without the relic; wrong the
        // moment one has it, and the aura is the hero's whole contribution.
        let hero_alive = self.hero_alive();

        self.replace_a_fallen_climber();
        self.respawn_cleared_floors();
        let hero_target = self.hero_target();
        self.fight(hero_target, hero_alive);
        self.restore_uncontested_floors();
        self.heal(hero_alive);
        self.regenerate_hero(hero_alive);
        self.remove_the_dead();
        self.walk();
        self.pay_sealed_income();

        // Depth-eligibility for the *next* lock, read after movement has
        // settled this tick's peak and before the player may spend.
        if self.margin_eligible_since.is_none() {
            let (floor, _) = self.next_lock();
            if self.run.peak >= floor + self.tuning.lock_margin {
                self.margin_eligible_since = Some(self.t);
            }
        }

        if let Some(floor) = player.station(self) {
            self.station_hero(floor);
        }
        // A purchase decision once per simulated second. The spend curve is
        // sensitive to the cadence, so it is tracked explicitly rather than
        // inferred from the clock — `t` accumulates in steps of 0.1 and never
        // lands on a whole second exactly.
        if self.t - self.last_decision >= 1.0 {
            player.spend(self);
            self.last_decision = self.t;
        }

        self.account_for_churn();
        self.record_best_rate();
        self.t += DT;
    }

    /// Ticket 07. The rate offline gold pays at: a **high-water mark**, not a
    /// recent average, so it cannot be sampled at an unrepresentative moment —
    /// a player who quits at the wall leaves a system whose steady state is
    /// gold-without-progress, and that is what they are owed for.
    ///
    /// Before a full window has elapsed the divisor is the elapsed time rather
    /// than 60, or the opening seconds of a run would report a rate diluted by
    /// time that never happened.
    fn record_best_rate(&mut self) {
        self.gold_ring_sum -= self.gold_ring[self.gold_ring_at];
        self.gold_ring[self.gold_ring_at] = self.gold_this_tick;
        self.gold_ring_sum += self.gold_this_tick;
        self.gold_ring_at = (self.gold_ring_at + 1) % RATE_WINDOW_TICKS;
        self.gold_this_tick = 0.0;

        let span = (self.t + DT).min(WINDOW);
        if span <= 0.0 {
            return;
        }
        let rate = self.gold_ring_sum / span;
        if rate > self.run.best_rate {
            self.run.best_rate = rate;
        }
    }

    /// Ticket 15 / ADR 0011. One interval for the whole stream, against an
    /// authored ratio.
    fn replace_a_fallen_climber(&mut self) {
        if self.t < self.next_replacement {
            return;
        }
        self.next_replacement = self.t + self.tuning.replacement;
        let ty = self.neediest_type();
        let alive = self.climbers.iter().filter(|c| c.ty == ty).count() as u32;

        // If that type is at its ceiling the slot is SKIPPED, never passed to
        // another type: passing it would let the replacement scheduler quietly
        // rewrite the authored composition. A binding ceiling is therefore
        // visible as a starved stream, which is the point — it should never bind.
        if alive >= self.tuning.types.get(ty).cap {
            self.cap_skips += 1;
            return;
        }
        let hp = self.climber_hp(ty, self.run.ranks[ty]);
        self.climbers.push(Climber { ty, pos: FloorPos::entering(self.lock_line()), hp, max_hp: hp });
    }

    /// Whichever type sits furthest below its authored share of the **live**
    /// stream. `CONTEXT.md` defines composition as the ratio of types in the
    /// stream, so targeting the live mix rather than the spawn sequence is what
    /// corrects for differential mortality instead of letting it drift the
    /// realised ratio away from the authored one.
    fn neediest_type(&self) -> ClimberType {
        let mut live = PerType::splat(0usize);
        for cl in &self.climbers {
            live[cl.ty] += 1;
        }
        let n = self.climbers.len();
        let total: u32 = ClimberType::ALL.iter().map(|t| self.tuning.composition[*t]).sum();

        let mut best = ClimberType::Melee;
        let mut best_deficit = f64::NEG_INFINITY;
        for ty in ClimberType::ALL {
            let share = self.tuning.composition[ty] as f64 / total as f64;
            let have = if n > 0 { live[ty] as f64 / n as f64 } else { 0.0 };
            // Strictly greater, so ties resolve to the earlier type. Fixed
            // order, fixed tie-break: this is why `ClimberType::ALL` is ordered.
            if share - have > best_deficit {
                best_deficit = share - have;
                best = ty;
            }
        }
        best
    }

    fn respawn_cleared_floors(&mut self) {
        let t = self.t;
        let pack = self.tuning.pack_size;
        let hp0 = self.tuning.enemy_hp0;
        let base = self.tuning.enemy_hp_base;
        for (f, s) in self.floors.iter_mut() {
            if s.count == 0 && s.respawn_at.is_some_and(|at| t >= at) {
                s.count = pack;
                s.pool_hp = pack as f64 * (hp0 * base.powf(*f as f64 - 1.0));
                s.respawn_at = None;
            }
        }
    }

    /// The hero holds a stretch, not a point: it fights the lowest floor in its
    /// zone that still has enemies, so floors behind it stay clear and climbers
    /// transit them free.
    fn hero_target(&self) -> Floor {
        if !self.hero_alive() || self.tuning.hero_zone == 0 {
            return self.hero_floor();
        }
        let lo = self.hero_floor().saturating_sub(self.tuning.hero_zone).max(self.lock_line());
        let hi = (self.hero_floor() + self.tuning.hero_zone).min(self.run.peak);
        for f in lo..=hi {
            if self.floors.get(&f).is_none_or(|s| s.count > 0) {
                return f;
            }
        }
        self.hero_floor()
    }

    fn fight(&mut self, hero_target: Floor, hero_alive: bool) {
        self.hero_ticks += 1;
        if hero_alive {
            self.hero_alive_ticks += 1;
        }

        // Contested floors, in first-seen climber order. Order is load-bearing:
        // gold accrues into one running sum and hero damage can trigger a death
        // mid-loop, so a `HashMap`'s iteration order would silently change
        // results run to run.
        let mut order = std::mem::take(&mut self.scratch_order);
        let mut seen = std::mem::take(&mut self.scratch_seen);
        order.clear();
        seen.clear();
        for cl in &self.climbers {
            let f = cl.pos.floor;
            // An unvisited floor still holds a full pack; a cleared one is
            // walked across rather than fought.
            if self.floors.get(&f).is_some_and(|s| s.count == 0) {
                continue;
            }
            if seen.insert(f) {
                order.push(f);
            }
        }
        if hero_alive && seen.insert(hero_target) {
            order.push(hero_target);
        }

        for &f in &order {
            self.fight_one_floor(f, hero_alive && hero_target == f, hero_alive);
        }

        self.scratch_order = order;
        self.scratch_seen = seen;
    }

    fn fight_one_floor(&mut self, f: Floor, hero_here: bool, hero_alive: bool) {
        if self.floor_state(f).count == 0 {
            return;
        }

        // Combat is floor-local (ADR 0010): whoever stands here fights here.
        // `standing` is therefore both the attackers and the targets, which is
        // the whole of what ticket 19 settled.
        let standing: Vec<usize> = (0..self.climbers.len())
            .filter(|&i| self.climbers[i].pos.floor == f)
            .collect();
        if standing.is_empty() && !hero_here {
            return;
        }

        // The aura multiplies what climbers already do and never adds
        // (ADR 0005). It is keyed on where the climber *stands*.
        let aura_on = hero_alive && self.tuning.hero_aura_mult > 1.0;
        let in_aura = |fl: Floor| {
            aura_on && fl.abs_diff(self.hero_floor()) <= self.tuning.hero_aura_floors
        };

        let mut dps = 0.0;
        let mut gold_weighted = 0.0;
        for &i in &standing {
            let cl = &self.climbers[i];
            let m = if in_aura(cl.pos.floor) { self.tuning.hero_aura_mult } else { 1.0 };
            let d = self.climber_dps(cl.ty, self.run.ranks[cl.ty]) * m;
            dps += d;
            gold_weighted += d * m;
        }
        if hero_here {
            // The hero's own damage is deliberately not where its value lies,
            // and it does not multiply itself — but the gold it earns is
            // weighted like everyone else's.
            let m = if in_aura(self.hero_floor()) { self.tuning.hero_aura_mult } else { 1.0 };
            let d = self.tuning.hero_dps0 * self.tuning.hero_power_base.powf(self.hero_level() as f64 - 1.0)
                * self.account.cumulative_prestige_mult;
            dps += d;
            gold_weighted += d * m;
        }
        let gold_mult = if dps > 0.0 { gold_weighted / dps } else { 1.0 };

        // --- damage the pack ---
        let e_hp = enemy_hp(&self.tuning, f);
        let respawn_at = self.t + self.tuning.respawn_timer;
        let s = self.floor_state(f);
        let before = s.count;
        s.pool_hp -= dps * DT;
        if s.pool_hp <= 0.0 {
            s.pool_hp = 0.0;
            s.count = 0;
            s.respawn_at = Some(respawn_at);
        } else {
            s.count = (s.pool_hp / e_hp).ceil() as u32;
        }
        let after = s.count;
        let killed = before - after;

        if killed > 0 {
            self.totals.kills += killed as u64;
            let g = killed as f64 * gold_per_kill(&self.tuning, f) * gold_mult;
            self.credit(f, g);
            // Experience comes only from kills inside the aura, so it reflects
            // where the hero has actually fought — enemies killed elsewhere in
            // the tower grant none (ticket 16). This is also why the hero's two
            // economies never touch: experience is never bought.
            if hero_alive && f.abs_diff(self.hero_floor()) <= self.tuning.hero_aura_floors {
                self.totals.kills_in_aura += killed as u64;
                // NOTE: experience accrues, but nothing yet turns it into a
                // level — `hero_cost` is still the throwaway model's gold-bought
                // hero. The experience-to-level curve is ticket 16's and is not
                // specified anywhere, so inventing one here would be tuning by
                // accident. The field is real so the save carries it.
                self.run.heroes.entry(SLICE_HERO.to_string()).or_default().experience +=
                    killed as f64;
            }
        }

        // --- the pack hits back, and it has no reach of its own ---
        let incoming = ((before + after) as f64 / 2.0) * enemy_dps(&self.tuning, f) * DT;

        // Taunt is per-hero, not a universal rule. Ticket 16 found the wall is
        // *safer* without it, because the crowd is armour — which is exactly
        // why it only pays for itself on a tanky hero.
        if self.tuning.hero_taunt && hero_here {
            self.hit_hero(incoming);
            return;
        }

        let mut total_threat: f64 = standing.iter().map(|&i| self.tuning.types.get(self.climbers[i].ty).threat).sum();
        if hero_here {
            total_threat += self.tuning.hero_threat;
        }
        if total_threat <= 0.0 {
            return;
        }
        for &i in &standing {
            let share = self.tuning.types.get(self.climbers[i].ty).threat / total_threat;
            self.climbers[i].hp -= incoming * share;
        }
        if hero_here {
            self.hit_hero(incoming * (self.tuning.hero_threat / total_threat));
        }
    }

    fn hit_hero(&mut self, d: f64) {
        self.hero_hp -= d;
        if self.hero_hp <= 0.0 {
            self.totals.hero_deaths += 1;
            self.hero_dead_until = Some(self.t + self.tuning.hero_respawn);
            self.hero_hp = self.hero_max_hp();
        }
    }

    /// Ticket 03: a floor nobody is fighting restores its pack, so attrition
    /// cannot beat a wall — the answers to a wall stay upgrading or stationing
    /// the hero, never waiting. The throwaway model never implemented this and
    /// tickets 14 and 19 both measured the omission as inside the noise, so it
    /// is a flag rather than an assumption.
    fn restore_uncontested_floors(&mut self) {
        if !self.tuning.failed_floor_reset {
            return;
        }
        let occupied: HashSet<Floor> = self.climbers.iter().map(|c| c.pos.floor).collect();
        let t = &self.tuning;
        for (f, s) in self.floors.iter_mut() {
            if s.count == 0 || occupied.contains(f) {
                continue;
            }
            let full = t.pack_size as f64 * enemy_hp(t, *f);
            if s.pool_hp < full {
                s.count = t.pack_size;
                s.pool_hp = full;
            }
        }
    }

    fn heal(&mut self, hero_alive: bool) {
        let heal_rate = self.climber_heal(ClimberType::Healer, self.run.ranks.healer) * DT;
        if heal_rate <= 0.0 {
            return;
        }
        let hero_max = self.hero_max_hp();

        // A snapshot the healers mutate as they go, so a second healer sees what
        // the first already mended rather than both committing to the same
        // dying unit. Kept in step with the real values on every application.
        let mut units: Vec<HealUnit> = self
            .climbers
            .iter()
            .enumerate()
            .map(|(i, cl)| HealUnit {
                target: HealTarget::Climber(i),
                floor: cl.pos.floor,
                hp: cl.hp,
                max_hp: cl.max_hp,
            })
            .collect();
        if hero_alive && self.tuning.heal_hero {
            // The hero is a valid target with no special-casing.
            units.push(HealUnit {
                target: HealTarget::Hero,
                floor: self.hero_floor(),
                hp: self.hero_hp,
                max_hp: hero_max,
            });
        }

        let mut per_floor: HashMap<Floor, Vec<usize>> = HashMap::new();
        for (i, u) in units.iter().enumerate() {
            per_floor.entry(u.floor).or_default().push(i);
        }

        let reach = self.tuning.aura_floors;
        let policy = self.tuning.heal_policy;
        let heal_self = self.tuning.heal_self;
        let healers: Vec<usize> = (0..self.climbers.len())
            .filter(|&i| self.climbers[i].ty == ClimberType::Healer)
            .collect();

        let mut in_range: Vec<usize> = Vec::new();
        for h in healers {
            let h_floor = self.climbers[h].pos.floor;
            in_range.clear();
            // Low floor first, and within a floor in `units` order. Both orders
            // decide ties in the policies below, so both are deliberate.
            for f in h_floor.saturating_sub(reach)..=(h_floor + reach) {
                let Some(idxs) = per_floor.get(&f) else { continue };
                for &i in idxs {
                    // A unit at or below zero is still a candidate: deaths are
                    // resolved after healing, so a healer can pull one back.
                    if units[i].hp >= units[i].max_hp {
                        continue;
                    }
                    if !heal_self && units[i].target == HealTarget::Climber(h) {
                        continue;
                    }
                    in_range.push(i);
                }
            }
            if in_range.is_empty() {
                continue;
            }

            match policy {
                // The aura treats everyone in radius at once, at a rate each.
                // Ticket 06 rejected it as the default: it fully out-healed the
                // climb, climbers reached the wall at 100%, and the Wall became
                // declared rather than emergent.
                HealPolicy::Aura => {
                    for &i in &in_range {
                        apply_heal(
                            &mut units,
                            &mut self.climbers,
                            &mut self.hero_hp,
                            &mut self.totals,
                            i,
                            heal_rate,
                        );
                    }
                }
                // Every other policy commits the healer's whole rate to one unit.
                _ => {
                    let mut best = in_range[0];
                    for &i in &in_range[1..] {
                        let better = match policy {
                            HealPolicy::LowestFraction => {
                                units[i].hp / units[i].max_hp < units[best].hp / units[best].max_hp
                            }
                            HealPolicy::LowestAbsolute => units[i].hp < units[best].hp,
                            HealPolicy::Nearest => {
                                let da = units[best].floor.abs_diff(h_floor);
                                let db = units[i].floor.abs_diff(h_floor);
                                db < da
                                    || (db == da
                                        && units[i].hp / units[i].max_hp
                                            < units[best].hp / units[best].max_hp)
                            }
                            HealPolicy::Aura => unreachable!("handled above"),
                        };
                        // Strictly better wins, so an earlier candidate holds a
                        // tie. That is what makes the two orders above matter.
                        if better {
                            best = i;
                        }
                    }
                    apply_heal(
                        &mut units,
                        &mut self.climbers,
                        &mut self.hero_hp,
                        &mut self.totals,
                        best,
                        heal_rate,
                    );
                }
            }
        }
    }

    fn regenerate_hero(&mut self, hero_alive: bool) {
        if !hero_alive || self.tuning.hero_regen_pct <= 0.0 {
            return;
        }
        let max = self.hero_max_hp();
        self.hero_hp = (self.hero_hp + max * self.tuning.hero_regen_pct * DT).min(max);
    }

    /// Deaths are legible in aggregate at the wall, not per unit (ticket 05) —
    /// so this counts by type and says nothing about any individual.
    fn remove_the_dead(&mut self) {
        let mut i = self.climbers.len();
        while i > 0 {
            i -= 1;
            if self.climbers[i].hp <= 0.0 {
                let ty = self.climbers[i].ty;
                self.totals.deaths_by_type[ty] += 1;
                self.climbers.remove(i);
            }
        }
    }

    fn walk(&mut self) {
        let speed = self.tuning.climb_speed * DT as f32;
        for i in 0..self.climbers.len() {
            let f = self.climbers[i].pos.floor;
            // Materialises the floor, matching the model — which is also what
            // makes `floor_records` a real memory reading rather than a guess.
            if self.floor_state(f).count != 0 {
                self.totals.ticks_fighting += 1;
                continue;
            }
            self.totals.ticks_walking += 1;
            if let Some(entered) = self.climbers[i].pos.walk(speed)
                && entered > self.run.peak
            {
                self.run.peak = entered;
                self.last_peak_gain_at = self.t;
            }
        }
    }

    fn pay_sealed_income(&mut self) {
        if self.sealed_rate > 0.0 {
            let g = self.sealed_rate * self.account.cumulative_prestige_mult * DT;
            self.run.gold += g;
            self.totals.gold_earned += g;
            self.gold_this_tick += g;
        }
    }

    /// Ticket 14. Counted every tick, not every sample: activation cost is paid
    /// per transition, so sampling it at 10s intervals would miss almost all
    /// of it.
    fn account_for_churn(&mut self) {
        let act = self.active_floor_set();
        for f in &act {
            if !self.prev_active.contains(f) {
                self.activations += 1;
            }
        }
        for f in &self.prev_active {
            if !act.contains(f) {
                self.deactivations += 1;
            }
        }
        self.prev_active = act;
    }

    fn active_floor_set(&self) -> HashSet<Floor> {
        let r = self.tuning.activation_radius;
        let line = self.lock_line();
        let mut act = HashSet::new();
        let add = |centre: Floor, act: &mut HashSet<Floor>| {
            for f in centre.saturating_sub(r).max(line)..=(centre + r) {
                act.insert(f);
            }
        };
        for cl in &self.climbers {
            add(cl.pos.floor, &mut act);
        }
        if self.hero_alive() {
            add(self.hero_floor(), &mut act);
        }
        act
    }

    /// Reads the instrumentation and resets the per-interval counters, exactly
    /// as the model did at each sample.
    pub fn sample(&mut self) -> Sample {
        let s = Sample::take(self);
        self.activations = 0;
        self.deactivations = 0;
        self.churn_since = self.t;
        self.cap_skips = 0;
        s
    }

    // Accessors the metrics module needs without making the fields public.
    pub(crate) fn churn(&self) -> (u64, u64, f64) {
        (self.activations, self.deactivations, (self.t - self.churn_since).max(DT))
    }
    pub(crate) fn cap_skips(&self) -> u64 {
        self.cap_skips
    }
    pub(crate) fn hero_uptime(&self) -> f64 {
        if self.hero_ticks == 0 { 1.0 } else { self.hero_alive_ticks as f64 / self.hero_ticks as f64 }
    }
    pub(crate) fn active_floors(&self) -> HashSet<Floor> {
        self.active_floor_set()
    }
    pub(crate) fn enemy_entities_in(&self, act: &HashSet<Floor>) -> u32 {
        act.iter()
            .map(|f| self.floors.get(f).map_or(self.tuning.pack_size, |s| s.count))
            .sum()
    }
    pub(crate) fn prestige_mult(&self) -> f64 {
        self.account.cumulative_prestige_mult
    }
    /// The account's cumulative multiplier coming into this run. Never shown to
    /// a player — ADR 0007 makes head start in floors the only form it takes on
    /// screen — but the harness needs it to label a campaign.
    pub fn prestige_mult_pub(&self) -> f64 {
        self.account.cumulative_prestige_mult
    }
    pub(crate) fn sealed_rate(&self) -> f64 {
        self.sealed_rate
    }
}
