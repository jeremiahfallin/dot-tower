//! The closed-form curves. Every one of them is quoted by at least one ticket,
//! so they live apart from the tick that consumes them and are testable alone.
//!
//! The engine of the whole game is the gap between two of these: enemy health
//! and damage scale at 1.075/floor against gold at 1.095, which makes deeper
//! floors strictly more gold-efficient and is what pulls the player upward.
//! Ticket 06 measured that past ~1.12 gold the economy runs away.

use crate::tuning::{PrestigeMode, Tuning};
use crate::types::ClimberType;

/// Health of a single enemy on `floor`.
pub fn enemy_hp(t: &Tuning, floor: u32) -> f64 {
    t.enemy_hp0 * t.enemy_hp_base.powf(floor as f64 - 1.0)
}

/// Damage per second of a single enemy on `floor`.
pub fn enemy_dps(t: &Tuning, floor: u32) -> f64 {
    t.enemy_dps0 * t.enemy_dps_base.powf(floor as f64 - 1.0)
}

/// Gold for one kill on `floor`, before any hero aura multiplier.
pub fn gold_per_kill(t: &Tuning, floor: u32) -> f64 {
    t.gold_per_kill0 * t.gold_base.powf(floor as f64 - 1.0)
}

/// Gold cost of taking a climber type from `rank` to `rank + 1`.
pub fn rank_cost(t: &Tuning, ty: ClimberType, rank: u32) -> f64 {
    t.types.get(ty).rank_cost0 * t.rank_cost_base.powf(rank as f64 - 1.0)
}

/// Gold cost of the `nth` lock, counting from 1.
///
/// This function is [ticket 20](../../../.scratch/dot-tower/issues/20-travel-time-and-the-lock-curve.md)'s
/// entire subject, and it resolved to a relationship rather than a number:
/// `lock_cost_base == gold_base ^ 10`, so a lock costs a **constant number of
/// kills at its depth, forever**. See [`Tuning::lock_outrun`].
///
/// Before that it grew ×4.00 per ten floors against income's ×2.478 — outrunning
/// income by ×1.614 per lock and compounding, so lock 1 cost 74 kills at its
/// depth and lock 47 cost 2.7×10¹¹. The lock line fell permanently behind the
/// wall and travel time grew without bound.
pub fn lock_cost(t: &Tuning, nth: u32) -> f64 {
    t.lock_cost0 * t.lock_cost_base.powf(nth as f64 - 1.0)
}

/// Gold cost of taking the hero from `level` to `level + 1`.
///
/// Note the tension this name carries: `CONTEXT.md` reserves **level** for what
/// the hero *earns* with experience, and ticket 16 settled that the hero levels
/// on kills in its aura and never on gold, so the two economies never touch.
/// This curve is the throwaway model's gold-bought hero, kept only so ported
/// runs stay comparable — the shipping progression is experience, and replacing
/// this is part of building the hero for real.
pub fn hero_cost(t: &Tuning, level: u32) -> f64 {
    t.hero_cost0 * t.hero_cost_base.powf(level as f64 - 1.0)
}

/// The multiplier a single prestige earns, as a function of that run's peak
/// floor. Ticket 08 measured ×5.18 at floor 147 and only ×38.6 at floor 1,000
/// — log-ish in peak floor, which is why this number stays legible forever and
/// is the one shown to the player.
pub fn prestige_mult_for(t: &Tuning, peak: u32) -> f64 {
    (1.0 + peak as f64 / t.prestige_divisor).powf(t.prestige_exponent)
}

/// Folds a run's earned multiplier into the account's cumulative one.
pub fn compound_prestige(t: &Tuning, cumulative: f64, earned: f64, best_peak: u32) -> f64 {
    match t.prestige_mode {
        PrestigeMode::Compound => cumulative * earned,
        PrestigeMode::BestPeak => prestige_mult_for(t, best_peak),
    }
}

/// The cumulative multiplier restated as **head start**: how many floors the
/// whole tower has dropped.
///
/// This is an exact restatement, not a friendly approximation — enemy health
/// and damage scale per floor while the multiplier scales climbers, so a head
/// start of `h` means floor `f` now fights exactly as floor `f − h` did.
/// [ADR 0007](../../../docs/adr/0007-prestige-is-shown-as-floors.md) makes this
/// the *only* form in which accumulated prestige is ever shown; ticket 08
/// measured it settling at ~26% of peak floor and staying there.
pub fn head_start_floors(t: &Tuning, cumulative_mult: f64) -> f64 {
    if cumulative_mult <= 1.0 {
        return 0.0;
    }
    cumulative_mult.ln() / t.enemy_hp_base.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> Tuning {
        Tuning::default()
    }

    #[test]
    fn floor_one_is_the_base_value() {
        assert_eq!(enemy_hp(&t(), 1), 36.0);
        assert_eq!(gold_per_kill(&t(), 1), 3.0);
        assert_eq!(lock_cost(&t(), 1), 1000.0);
        assert_eq!(rank_cost(&t(), ClimberType::Healer, 1), 45.0);
    }

    #[test]
    fn gold_outruns_enemy_health_which_is_the_engine() {
        // Ticket 06: deeper floors are strictly more gold-efficient, and that
        // is what pulls the player upward. If this ever inverts the game has no
        // reason to climb.
        let t = t();
        let efficiency = |f: u32| gold_per_kill(&t, f) / enemy_hp(&t, f);
        assert!(efficiency(100) > efficiency(1));
        assert!(efficiency(1000) > efficiency(100));
    }

    #[test]
    fn a_lock_costs_a_constant_number_of_kills_at_its_depth() {
        // The property ticket 20 settled on, stated the way a designer would
        // read it rather than as a ratio of two bases.
        let t = t();
        let kills_for = |n: u32| lock_cost(&t, n) / gold_per_kill(&t, 10 * n);
        let first = kills_for(1);
        for n in [1, 5, 20, 60, 120] {
            let k = kills_for(n);
            assert!((k / first - 1.0).abs() < 1e-9, "lock {n} costs {k} kills against {first}");
        }
        // At `lock_cost0` 1000 that constant is ~147 kills. It is the crowd
        // dial: ticket 20 measured the crowd at 6 when a lock costs ~18 kills
        // and 59 when it costs ~590, at a cost of 4% in peak floor.
        assert!((first - 147.0).abs() < 1.0, "a lock costs {first:.1} kills at its depth");
    }

    #[test]
    fn the_curve_ticket_20_replaced_outran_income_and_compounded() {
        // Kept because it is the failure the decision is against, and it should
        // stay reproducible: ×4.00 per ten floors against income's ×2.478.
        let before = Tuning { lock_cost_base: 4.0, ..t() };
        assert!((before.income_per_lock() - 2.478).abs() < 0.01);
        assert!((before.lock_outrun() - 1.614).abs() < 0.01, "{}", before.lock_outrun());
        // Which is what made lock 47 cost 2.7e11.
        assert!(lock_cost(&before, 47) > 1e11);
    }

    #[test]
    fn head_start_is_an_exact_restatement() {
        // The defining property: with head start h, floor f fights as f - h.
        let t = t();
        let m = 250.0;
        let h = head_start_floors(&t, m);
        let f = 400.0;
        // Enemy health at f, divided by the multiplier scaling our climbers,
        // must equal enemy health at f - h.
        let scaled = t.enemy_hp0 * t.enemy_hp_base.powf(f - 1.0) / m;
        let shallower = t.enemy_hp0 * t.enemy_hp_base.powf(f - h - 1.0);
        assert!((scaled - shallower).abs() / shallower < 1e-12);
    }

    #[test]
    fn a_fresh_account_has_no_head_start() {
        assert_eq!(head_start_floors(&t(), 1.0), 0.0);
    }

    #[test]
    fn the_earned_multiplier_stays_legible_at_depth() {
        // Ticket 08's two measured points, which are why this number is the one
        // shown to the player.
        let t = t();
        assert!((prestige_mult_for(&t, 147) - 5.18).abs() < 0.02);
        assert!((prestige_mult_for(&t, 1000) - 38.6).abs() < 0.2);
    }
}
