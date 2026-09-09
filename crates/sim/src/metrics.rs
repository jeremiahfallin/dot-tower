//! What the simulation reports about itself.
//!
//! Two shapes, and the split matters. [`Totals`] are run-to-date counters the
//! world keeps; [`Sample`] is an instant, read on whatever cadence the harness
//! chooses. Ticket 14's entity accounting is here in full because it is the
//! reading the whole no-cap decision rests on — ticket 03 claimed entity count
//! stays in the low hundreds whether the band is 20 floors or 5,000, and this
//! is what checks it.

use crate::geometry::Floor;
use crate::types::PerType;
use crate::world::World;

/// Counters that only ever go up over a run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Totals {
    pub gold_earned: f64,
    pub kills: u64,
    /// Kills inside the hero's aura — the only source of hero experience
    /// (ticket 16), so this is where levels come from and gold is not involved.
    pub kills_in_aura: u64,
    pub deaths_by_type: PerType<u64>,
    pub hero_deaths: u64,
    pub heal_to_climbers: f64,
    pub heal_to_hero: f64,

    /// Climber-ticks spent walking across a cleared floor, against ticks spent
    /// fighting on one that still has enemies. Ticket 20's question 3 turns on
    /// the ratio: if the stream is mostly walking, travel time *is* the game at
    /// depth, whether or not that was intended.
    pub ticks_walking: u64,
    pub ticks_fighting: u64,
}

/// One lock, and what the player was actually waiting on to buy it.
///
/// Ticket 20's question 1 asks whether a lock cost that merely matches income
/// makes locking automatic — a purchase with no decision in it. That is exactly
/// [`Self::gold_wait`]: the time the lock sat allowed-by-depth and unaffordable.
/// A gold wait of zero means the lock was bought the instant the climb earned
/// the right to it, and gold never entered into it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LockEvent {
    pub t: f64,
    pub floor: Floor,
    pub cost: f64,
    /// Seconds this lock was permitted by [`crate::Tuning::lock_margin`] but
    /// could not be afforded.
    pub gold_wait: f64,
}

impl Totals {
    /// Share of climber-ticks spent walking rather than fighting.
    pub fn walking_fraction(&self) -> f64 {
        let total = self.ticks_walking + self.ticks_fighting;
        if total == 0 { 0.0 } else { self.ticks_walking as f64 / total as f64 }
    }

    pub fn deaths(&self) -> u64 {
        self.deaths_by_type.melee + self.deaths_by_type.ranged + self.deaths_by_type.healer
    }
}

/// One instant of the run.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub t: f64,
    pub gold: f64,
    pub gold_earned: f64,
    /// Gold per second, run-to-date.
    pub gps: f64,
    pub peak: Floor,
    /// The highest floor holding live climbers. See [`World::frontier`] on why
    /// this is not simply "the wall".
    pub frontier: Floor,
    /// Mean health fraction of the climbers at the front. Ticket 06 read the
    /// Wall off this: an aura healer held it at 1.0, which is what made the
    /// Wall declared rather than emergent.
    pub frontier_hp: f64,
    pub lock_line: Floor,
    pub lock_level: u32,
    pub sealed_rate: f64,

    /// Run-to-date, so a trace can be lined up against the reference model
    /// without re-deriving them from the totals.
    pub kills: u64,
    pub deaths: u64,

    pub pop: usize,
    pub pop_by_type: PerType<usize>,
    pub hp_by_type: PerType<f64>,
    /// How far behind the frontier each type sits, on average. Ticket 20's
    /// subject: at depth this reached 108 floors.
    pub back_by_type: PerType<f64>,

    pub ranks: PerType<u32>,
    pub hero_level: u32,
    pub hero_floor: Floor,
    pub hero_uptime: f64,
    pub prestige_mult: f64,

    // --- ticket 14: what this instant would cost ---
    pub active_floors: usize,
    pub occupied_floors: usize,
    /// Floors holding both climbers and live enemies. Ticket 03 bounded cost by
    /// these and put the bound at ~90; ticket 14 measured 2–8, *decreasing* with
    /// depth, because an active floor holds ~1.4 enemies against a pack of 4 —
    /// trailing climbers walk through ground the leaders already cleared.
    pub contested_floors: usize,
    /// Lowest to highest climber, inclusive. Ticket 14 saw this reach 581.
    pub climber_spread: u32,
    pub enemy_entities: u32,
    /// The number the device session has to absorb. Ticket 14 measured a max of
    /// 201 across a 14-run campaign against a prediction of 1,100, which is what
    /// narrowed the range worth testing to 200–450.
    pub entities: u32,
    pub floor_records: usize,
    pub activations_per_sec: f64,
    pub deactivations_per_sec: f64,

    /// Replacement slots skipped because a type was at its ceiling. ADR 0011
    /// says this should be **zero** — a binding ceiling means the cap is acting
    /// as the crowd dial, which is the one thing it must never be. It is not
    /// currently zero, and that is ticket 20's problem.
    pub cap_skips: u64,
}

impl Sample {
    pub(crate) fn take(w: &World) -> Self {
        let frontier = w.frontier();
        let climbers = w.climbers();

        let at_front: Vec<&crate::world::Climber> =
            climbers.iter().filter(|c| c.pos.floor + 1 >= frontier).collect();
        let frontier_hp = if at_front.is_empty() {
            1.0
        } else {
            at_front.iter().map(|c| c.hp / c.max_hp).sum::<f64>() / at_front.len() as f64
        };

        let mut pop_by_type = PerType::splat(0usize);
        let mut hp_by_type = PerType::splat(0.0f64);
        let mut back_by_type = PerType::splat(0.0f64);
        for cl in climbers {
            pop_by_type[cl.ty] += 1;
            hp_by_type[cl.ty] += cl.hp / cl.max_hp;
            back_by_type[cl.ty] += (frontier - cl.pos.floor.min(frontier)) as f64;
        }
        for ty in crate::types::ClimberType::ALL {
            let n = pop_by_type[ty].max(1) as f64;
            hp_by_type[ty] /= n;
            back_by_type[ty] /= n;
        }

        let act = w.active_floors();
        let enemy_entities = w.enemy_entities_in(&act);

        let mut occupied = std::collections::HashSet::new();
        for cl in climbers {
            occupied.insert(cl.pos.floor);
        }
        let contested = occupied
            .iter()
            .filter(|f| w.floors().get(f).is_none_or(|s| s.count > 0))
            .count();

        let spread = match (
            climbers.iter().map(|c| c.pos.floor).min(),
            climbers.iter().map(|c| c.pos.floor).max(),
        ) {
            (Some(lo), Some(hi)) => hi - lo + 1,
            _ => 0,
        };

        let (activations, deactivations, elapsed) = w.churn();

        Self {
            t: w.elapsed(),
            gold: w.gold(),
            gold_earned: w.totals().gold_earned,
            gps: w.gold_rate(),
            peak: w.peak(),
            frontier,
            frontier_hp,
            lock_line: w.lock_line(),
            lock_level: w.lock_level(),
            sealed_rate: w.sealed_rate(),

            kills: w.totals().kills,
            deaths: w.totals().deaths(),

            pop: climbers.len(),
            pop_by_type,
            hp_by_type,
            back_by_type,

            ranks: w.ranks(),
            hero_level: w.hero_level(),
            hero_floor: w.hero_floor(),
            hero_uptime: w.hero_uptime(),
            prestige_mult: w.prestige_mult(),

            active_floors: act.len(),
            occupied_floors: occupied.len(),
            contested_floors: contested,
            climber_spread: spread,
            enemy_entities,
            entities: enemy_entities + climbers.len() as u32 + u32::from(w.hero_alive()),
            floor_records: w.floors().len(),
            activations_per_sec: activations as f64 / elapsed,
            deactivations_per_sec: deactivations as f64 / elapsed,

            cap_skips: w.cap_skips(),
        }
    }
}
