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

/// Gold cost of the `nth` lock under [`LockPricing::Geometric`], counting from 1.
///
/// **Superseded as a shipping mechanism** by
/// [ADR 0013](../../../docs/adr/0013-locks-are-priced-in-time.md): a lock is priced in
/// *time*, K seconds of recent income, because no geometric base can hold a
/// constant relationship to income. Income compounds faster than per-kill gold —
/// kill rate grows with ranks — so any base fitted to the current rank ladder
/// drifts the moment that ladder is retuned.
///
/// Retained because the reference model still defaults to it and every reading
/// taken before ADR 0013 needs it to reproduce. At the pre-ADR shipped values it
/// grew ×4.00 per ten floors against per-kill gold's ×2.478, so the lock line
/// fell permanently behind the wall and travel time grew without bound: lock 1
/// cost 74 kills at its depth and lock 47 cost 2.7×10¹¹.
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
        assert_eq!(lock_cost(&t(), 1), 500.0);
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

    /// Why geometric pricing was rejected, stated as a property rather than as
    /// a story: holding lock cost to a constant number of *kills* at its depth
    /// does not hold it to a constant share of *income*, because income is kills
    /// per second times gold per kill and only the second term is being matched.
    /// A rank ladder that doubles kill rate halves the lock's real price.
    #[test]
    fn matching_per_kill_gold_does_not_match_income() {
        let t = Tuning { lock_cost_base: Tuning::default().income_per_lock(), ..t() };

        // Per-kill gold: matched exactly, which is what made the idea appealing.
        let kills_for = |n: u32| lock_cost(&t, n) / gold_per_kill(&t, 10 * n);
        assert!((kills_for(20) / kills_for(1) - 1.0).abs() < 1e-9, "per-kill gold is matched");

        // Income also rises with kill rate, and kill rate rises with rank. Two
        // ranks of climber damage is a doubling of income that the lock price
        // never sees.
        let dps_at = |rank: u32| t.rank_power_base.powf(rank as f64 - 1.0);
        let income_growth = dps_at(20) / dps_at(1);
        assert!(income_growth > 10.0, "ranks move income by x{income_growth:.1}");
        // So the lock's share of income has fallen by that whole factor.
        assert!(
            kills_for(20) / income_growth < kills_for(1) * 0.1,
            "a base matched to per-kill gold still collapses as a share of income"
        );
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
