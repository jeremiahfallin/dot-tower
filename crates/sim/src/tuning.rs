//! Every tuned constant, loaded from RON.
//!
//! The map's standing rule: *tuning constants live in hot-reloadable RON, never
//! in Rust — an idle game is 90% tuning.* [Ticket 12](../../../.scratch/dot-tower/issues/12-save-schema.md)
//! adds the other half: **nothing tuned is ever saved**, because a save that
//! carries a tuning constant breaks hot reload for every player who already has
//! one. So this type is the boundary — it loads from disk and it never
//! round-trips through a save.
//!
//! [`Default`] carries the first-pass curve from
//! [ticket 06](../../../.scratch/dot-tower/issues/06-progression-curve-first-pass.md),
//! and the whole struct is `#[serde(default)]`, so `tuning.ron` may override one
//! field or all of them. That is what makes a sweep a three-line file.
//!
//! # Deliberately absent
//!
//! - **Attack reach, standoff, formation order and line targeting.** Measured
//!   and rejected by [ticket 19](../../../.scratch/dot-tower/issues/19-ranged-attack-reach.md);
//!   combat is floor-local and threat weighting is the abstraction
//!   ([ADR 0010](../../../docs/adr/0010-combat-is-floor-local-and-has-no-reach.md)).
//!   Their machinery is not ported. The prototype that measured them still
//!   exists if the question is ever reopened.
//! - **Per-type spawn intervals.** Retired by ticket 15: the stream has one
//!   global [`Tuning::replacement`] against an authored [`Tuning::composition`]
//!   ([ADR 0011](../../../docs/adr/0011-composition-is-authored.md)).
//! - **Buy policy, hero stationing, and the stall heuristic.** Not tuning —
//!   they are a *scripted player*. See [`crate::autoplay`].

use crate::types::{ClimberType, PerType};
use serde::{Deserialize, Serialize};

/// How the healer chooses what to mend.
///
/// Ticket 06 built this seam and settled [`Self::LowestFraction`] as the
/// shipped default: the aura fully out-healed the climb, so climbers reached
/// the wall at 100% and the Wall was *declared* rather than emergent. Whether
/// the player ever picks is still open in the map's fog — the seam is what
/// ticket 06 promised, not a reopening of that decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealPolicy {
    /// Every wounded unit in radius at once. Rejected as the default.
    Aura,
    /// The whole rate onto whoever is furthest from full, proportionally.
    #[default]
    LowestFraction,
    /// The whole rate onto whoever has the fewest points left.
    LowestAbsolute,
    /// The whole rate onto the nearest, breaking ties by lowest fraction.
    Nearest,
}

/// How one prestige's earned multiplier becomes the account's cumulative one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PrestigeMode {
    /// Each run's multiplier multiplies the last. Ticket 06 settled this: a
    /// best-ever-peak multiplier dead-ends at floor ~220.
    #[default]
    Compound,
    /// A function of best-ever peak, which does not compound. Kept only so the
    /// dead end stays reproducible.
    BestPeak,
}

/// Per-climber-type constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TypeTuning {
    pub hp0: f64,
    pub dps0: f64,
    pub heal0: f64,
    /// A **ceiling**, never a lever and never bought (ADR 0011). Under a
    /// healthy lock curve it should not bind at all — and it currently does,
    /// constantly, which is [ticket 20](../../../.scratch/dot-tower/issues/20-travel-time-and-the-lock-curve.md)'s
    /// problem. [`crate::metrics::Sample::cap_skips`] is how you see it bind.
    pub cap: u32,
    pub rank_cost0: f64,
    /// Share of a floor's incoming damage this type draws, relative to the
    /// others standing on it. Ticket 06's abstraction, and after ticket 19 the
    /// *only* positional idea in combat.
    pub threat: f64,
}

impl Default for TypeTuning {
    fn default() -> Self {
        Self { hp0: 70.0, dps0: 5.5, heal0: 0.0, cap: 40, rank_cost0: 25.0, threat: 3.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tuning {
    // --- the tower ---
    pub enemy_hp0: f64,
    pub enemy_hp_base: f64,
    pub enemy_dps0: f64,
    pub enemy_dps_base: f64,
    pub pack_size: u32,
    pub gold_per_kill0: f64,
    pub gold_base: f64,
    /// Seconds before a cleared floor repopulates. Ticket 03 surfaced that this
    /// *is* the farming mechanic: without it there would be nothing below the
    /// wall to farm, so it sets the gold rate for the whole band.
    pub respawn_timer: f64,

    // --- climbers ---
    pub rank_power_base: f64,
    /// Steeper than [`Self::rank_power_base`], by design.
    pub rank_cost_base: f64,
    /// Prestige multiplies **health and damage** only. Healing is not raw
    /// power, so it is off by default.
    pub heal_scales_with_prestige: bool,
    pub types: PerType<TypeTuning>,
    /// Floors per second across a *cleared* floor.
    pub climb_speed: f32,

    // --- healing ---
    /// A healer mends its own floor ± this many.
    pub aura_floors: u32,
    pub heal_policy: HealPolicy,
    pub heal_self: bool,
    /// Is the hero in the candidate set at all?
    pub heal_hero: bool,

    // --- hero ---
    pub hero_hp0: f64,
    pub hero_dps0: f64,
    pub hero_power_base: f64,
    pub hero_cost0: f64,
    pub hero_cost_base: f64,
    pub hero_respawn: f64,
    pub hero_threat: f64,
    /// Floors either side of its post the hero keeps clear.
    pub hero_zone: u32,
    /// The aura multiplies what climbers in it already do — damage *and* gold.
    /// It never adds ([ADR 0005](../../../docs/adr/0005-the-hero-multiplies-climbers-add.md)),
    /// and ticket 16 made the multiplier relic-only, so `1.0` is correct for a
    /// fresh account rather than a placeholder.
    pub hero_aura_mult: f64,
    pub hero_aura_floors: u32,
    /// Per-hero, not a universal rule. Ticket 16: taunt pays for itself only on
    /// a tanky hero, and the slice ships the tank.
    pub hero_taunt: bool,
    pub hero_regen_pct: f64,

    // --- the stream (ticket 15, ADR 0011) ---
    /// The single global interval at which a fallen climber is replaced, in
    /// seconds. Ticket 06 found this is the run-length dial; ticket 15 made it
    /// the *only* thing it is.
    pub replacement: f64,
    /// The authored ratio. Never tuned to an optimum and never shown to the
    /// player. The integers themselves wait on ticket 20.
    pub composition: PerType<u32>,

    // --- locking ---
    /// What the first lock costs. **This is the crowd dial**, and ticket 20
    /// measured it as a nearly free one: across a 32x sweep at a scale-invariant
    /// growth rate the crowd went 6 -> 59 while peak floor moved 643 -> 616.
    /// How big the crowd is and how far it walks are the same number, so this
    /// sets both.
    pub lock_cost0: f64,
    /// How lock cost grows per lock. Only meaningful against
    /// [`Self::income_per_lock`] — see [`Self::lock_outrun`], which is the
    /// number ticket 20 is actually about.
    pub lock_cost_base: f64,
    /// Only buy the lock at floor 10k once peak floor is this far above it.
    pub lock_margin: u32,

    // --- prestige ---
    pub prestige_divisor: f64,
    pub prestige_exponent: f64,
    pub prestige_mode: PrestigeMode,

    // --- rules the design settled but the model has never run ---
    /// Ticket 03: a floor nobody is fighting restores its pack, so attrition
    /// cannot beat a wall. **The throwaway model never implemented this**, and
    /// tickets 14 and 19 found the omission is inside the noise (peak 147 → 144).
    /// Defaulted to the model's behaviour so the ported numbers are comparable;
    /// ticket 03's actual rule is `true` and turning it on is how that gets
    /// checked on the real sim.
    pub failed_floor_reset: bool,
    /// Ticket 03 originally froze a sealed floor's measured output as passive
    /// income. **Ticket 06 withdrew it** — a band you have safely climbed past
    /// has been empty for minutes and measures zero, and the closed-form
    /// ceiling is 0.35% of the money. Locking is a spawn-point and travel-time
    /// mechanic, not an economic pillar. Kept as a flag, off, so the withdrawal
    /// stays falsifiable rather than merely asserted.
    pub sealed_income: bool,

    // --- instrumentation (ticket 14) ---
    /// How near a climber must be for a floor's pack to exist as entities
    /// rather than as data. Affects measurement only; no rule reads it.
    pub activation_radius: u32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            enemy_hp0: 36.0,
            enemy_hp_base: 1.075,
            enemy_dps0: 2.2,
            enemy_dps_base: 1.075,
            pack_size: 4,
            gold_per_kill0: 3.0,
            gold_base: 1.095,
            respawn_timer: 20.0,

            rank_power_base: 1.15,
            rank_cost_base: 1.30,
            heal_scales_with_prestige: false,
            types: PerType::new(
                TypeTuning { hp0: 70.0, dps0: 5.5, heal0: 0.0, cap: 40, rank_cost0: 25.0, threat: 3.0 },
                TypeTuning { hp0: 30.0, dps0: 9.0, heal0: 0.0, cap: 30, rank_cost0: 35.0, threat: 1.0 },
                TypeTuning { hp0: 36.0, dps0: 0.0, heal0: 3.5, cap: 20, rank_cost0: 45.0, threat: 0.5 },
            ),
            climb_speed: 0.5,

            aura_floors: 1,
            heal_policy: HealPolicy::LowestFraction,
            heal_self: true,
            heal_hero: true,

            hero_hp0: 420.0,
            hero_dps0: 38.0,
            hero_power_base: 1.18,
            hero_cost0: 120.0,
            hero_cost_base: 1.30,
            hero_respawn: 20.0,
            hero_threat: 5.0,
            hero_zone: 0,
            hero_aura_mult: 1.0,
            hero_aura_floors: 0,
            hero_taunt: false,
            hero_regen_pct: 0.0,

            // 1.67s reproduces the aggregate inflow of the three per-type
            // intervals it replaced, so ticket 15's mechanism change is not
            // silently also a tuning change.
            replacement: 1.67,
            // 4/3/2 mirrors the old 40/30/20 ceilings for the same reason.
            composition: PerType::new(4, 3, 2),

            // Ticket 20. Both of these moved, and they moved for different
            // reasons — one is structural, the other is authoring.
            //
            // `lock_cost_base` is NOT a free constant: it is `gold_base ^ 10`,
            // so a lock costs the same number of kills at its depth forever.
            // Above that the ceiling binds at depth and ADR 0011 fails; below
            // it, locking stops being a purchase. If `gold_base` is ever
            // retuned this must follow it — `lock_outrun` is what to check.
            lock_cost0: 1000.0,
            lock_cost_base: 1.095_f64.powf(10.0),
            lock_margin: 15,

            prestige_divisor: 50.0,
            prestige_exponent: 1.2,
            prestige_mode: PrestigeMode::Compound,

            failed_floor_reset: false,
            sealed_income: false,

            activation_radius: 2,
        }
    }
}

/// What is wrong with a tuning file, in terms a human can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuningError(pub String);

impl std::fmt::Display for TuningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TuningError {}

impl Tuning {
    /// What income multiplies by across one lock interval of ten floors.
    pub fn income_per_lock(&self) -> f64 {
        self.gold_base.powf(10.0)
    }

    /// How much faster lock cost grows than the income that pays for it.
    ///
    /// This is ticket 20's whole subject, and the measured result is that it
    /// wants to be **exactly 1.0**. Above 1 the lock line falls permanently
    /// behind, travel time diverges, and the per-type ceiling starts setting the
    /// crowd size — which [ADR 0011](../../../docs/adr/0011-composition-is-authored.md)
    /// says it must never do. Below about 0.9 the lock is affordable the moment
    /// depth allows it and stops being a purchase at all.
    pub fn lock_outrun(&self) -> f64 {
        self.lock_cost_base / self.income_per_lock()
    }

    /// Parses RON and validates. Hot reload means a human edits this file with
    /// the game running, so a bad edit must produce a message, never a `NaN`
    /// that propagates silently through every `f64` multiplier downstream —
    /// ticket 12 made that argument about saves and it applies here with more
    /// force, because saves are not edited by hand.
    pub fn from_ron_str(s: &str) -> Result<Self, TuningError> {
        let t: Tuning = ron::from_str(s).map_err(|e| TuningError(format!("tuning.ron: {e}")))?;
        t.validate()?;
        Ok(t)
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, TuningError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .map_err(|e| TuningError(format!("{}: {e}", path.display())))?;
        Self::from_ron_str(&text)
    }

    /// Things that load and run but are probably a mistake.
    ///
    /// Separate from [`Self::validate`] because none of these is wrong enough to
    /// refuse — the sweeps under `sweeps/` deliberately set some of them, and a
    /// tuning pass should be able to try a bad idea. They exist because a
    /// decision that was expensive to reach is cheap to undo by accident.
    pub fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        let k = self.lock_outrun();
        if k > 1.0001 {
            out.push(format!(
                "lock cost outruns income by x{k:.3} per lock (ticket 20 settled x1.000): the lock \
                 line will fall permanently behind the wall, and the per-type ceiling will set the \
                 crowd size in violation of ADR 0011"
            ));
        } else if k < 0.9 {
            out.push(format!(
                "lock cost trails income at x{k:.3} per lock (ticket 20 settled x1.000): locks \
                 become affordable the moment depth allows them, so locking stops being a purchase"
            ));
        }
        if self.sealed_income {
            out.push(
                "sealed_income is ON, which ticket 06 withdrew from the design; every reading will \
                 include income the game does not have"
                    .to_string(),
            );
        }
        out
    }

    pub fn validate(&self) -> Result<(), TuningError> {
        let mut bad = Vec::new();

        let mut finite = |name: &str, v: f64| {
            if !v.is_finite() {
                bad.push(format!("{name} is {v}"));
            }
        };
        finite("enemy_hp0", self.enemy_hp0);
        finite("enemy_dps0", self.enemy_dps0);
        finite("gold_per_kill0", self.gold_per_kill0);
        finite("hero_hp0", self.hero_hp0);
        finite("hero_dps0", self.hero_dps0);
        finite("replacement", self.replacement);
        finite("lock_cost0", self.lock_cost0);

        // `is_nan() || <= 0.0` rather than `!(v > 0.0)`: same truth table, but
        // it says out loud that a NaN is meant to land here.
        let mut positive = |name: &str, v: f64| {
            if v.is_nan() || v <= 0.0 {
                bad.push(format!("{name} must be > 0, got {v}"));
            }
        };
        positive("enemy_hp0", self.enemy_hp0);
        positive("enemy_hp_base", self.enemy_hp_base);
        positive("gold_base", self.gold_base);
        positive("rank_power_base", self.rank_power_base);
        positive("rank_cost_base", self.rank_cost_base);
        positive("respawn_timer", self.respawn_timer);
        // A replacement of zero spawns a climber every tick forever.
        positive("replacement", self.replacement);
        positive("prestige_divisor", self.prestige_divisor);

        if self.pack_size == 0 {
            bad.push("pack_size must be >= 1".into());
        }
        if !(self.climb_speed.is_finite() && self.climb_speed > 0.0) {
            bad.push(format!("climb_speed must be > 0, got {}", self.climb_speed));
        }
        if self.hero_aura_mult < 1.0 {
            // ADR 0005: the hero multiplies. A multiplier below 1 would make it
            // subtract, which is not a tuning value, it is a sign error.
            bad.push(format!("hero_aura_mult must be >= 1.0, got {}", self.hero_aura_mult));
        }
        if self.composition.iter().all(|(_, n)| *n == 0) {
            bad.push("composition is all zero — the stream would never spawn".into());
        }
        for t in ClimberType::ALL {
            let ty = self.types.get(t);
            if !(ty.hp0 > 0.0 && ty.hp0.is_finite()) {
                bad.push(format!("types.{}.hp0 must be > 0, got {}", t.name(), ty.hp0));
            }
            if ty.threat < 0.0 {
                bad.push(format!("types.{}.threat must be >= 0", t.name()));
            }
        }

        if bad.is_empty() { Ok(()) } else { Err(TuningError(bad.join("; "))) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ticket 20's structural result, pinned. If `gold_base` moves and
    /// `lock_cost_base` does not follow it, this is what says so.
    #[test]
    fn the_shipped_lock_curve_neither_outruns_income_nor_trails_it() {
        assert!((Tuning::default().lock_outrun() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_shipped_defaults_validate() {
        Tuning::default().validate().unwrap();
    }

    #[test]
    fn a_partial_file_overrides_only_what_it_names() {
        // This is the shape every ticket-20 sweep will be written in.
        let t = Tuning::from_ron_str("(lock_cost_base: 2.5)").unwrap();
        assert_eq!(t.lock_cost_base, 2.5);
        assert_eq!(t.gold_base, Tuning::default().gold_base);
        assert_eq!(t.types.melee.hp0, 70.0);
    }

    #[test]
    fn a_nan_is_refused_rather_than_propagated() {
        let err = Tuning::from_ron_str("(replacement: NaN)").unwrap_err();
        assert!(err.0.contains("replacement"), "{err}");
    }

    #[test]
    fn an_aura_that_subtracts_is_refused() {
        let err = Tuning::from_ron_str("(hero_aura_mult: 0.5)").unwrap_err();
        assert!(err.0.contains("hero_aura_mult"), "{err}");
    }

    #[test]
    fn round_trips_through_ron() {
        let t = Tuning::default();
        let s = ron::ser::to_string_pretty(&t, ron::ser::PrettyConfig::default()).unwrap();
        assert_eq!(Tuning::from_ron_str(&s).unwrap(), t);
    }
}
