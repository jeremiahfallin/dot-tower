//! Tests that check the simulation against the decisions it is used to defend.
//!
//! The map's fog says it plainly: *if this model becomes a kept tool it needs
//! to be checked against the decisions it is used to defend.* Ticket 19 found
//! the throwaway model inverting a headline result and ticket 14 found it had
//! never implemented ticket 03's floor-reset rule. So these are not unit tests
//! of arithmetic — they are assertions that the recorded findings still hold.
//!
//! They are slow by unit-test standards (each plays an hour of simulated time).
//! That is the price of testing a claim about an hour of play.

use dot_tower_sim::autoplay::{BuyPolicy, Greedy, Station};
use dot_tower_sim::{campaign, ClimberType, Player, Tuning};

const HOUR: f64 = 3600.0;

fn greedy() -> Box<dyn Player> {
    Box::new(Greedy::default())
}

/// Late-run cap skips under a given tuning. Named apart because the test that
/// uses it is asserting a *contrast*, and inlining it twice hid that.
fn campaign_before(t: &Tuning) -> u64 {
    let c = campaign(t, 6, HOUR, greedy);
    let deep = c.last().unwrap();
    deep.samples[deep.samples.len() * 2 / 3..].iter().map(|s| s.cap_skips).sum()
}

fn peaks(t: &Tuning, runs: usize) -> Vec<u32> {
    campaign(t, runs, HOUR, greedy).iter().map(|o| o.peak).collect()
}

/// The port is exact. These are the JS model's peaks for six 60-minute runs,
/// read off `model.mjs` directly, on both lock curves.
///
/// `sealed_income` is on because the model pays it and the design does not —
/// see [`the_withdrawn_sealed_income_is_negligible_on_the_shipped_curve`].
/// Parity is checked *with* the model's own rules, not the game's.
#[test]
fn the_port_reproduces_the_reference_model_exactly() {
    let mut t = Tuning {
        sealed_income: true,
        lock_cost0: 500.0,
        lock_cost_base: 4.0,
        ..Default::default()
    };
    assert_eq!(peaks(&t, 6), vec![133, 184, 254, 343, 448, 569], "the model's lock curve");

    t.lock_cost_base = 2.5;
    assert_eq!(peaks(&t, 6), vec![138, 206, 322, 505, 738, 998], "ticket 20's proposed curve");
}

/// Ticket 06 withdrew sealed-floor income after measuring a sealed band at
/// 0.04% of the money. On the shipped curve that holds: turning the withdrawn
/// mechanic on or off moves peak floor by at most two floors in a run, which is
/// 1.1% at its worst and under 0.2% by the end of a six-run campaign.
#[test]
fn the_withdrawn_sealed_income_is_negligible_on_the_shipped_curve() {
    let off = Tuning { lock_cost0: 500.0, lock_cost_base: 4.0, ..Default::default() };
    let on = Tuning { sealed_income: true, ..off.clone() };

    let (a, b) = (peaks(&off, 6), peaks(&on, 6));
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        let drift = (*x as f64 - *y as f64).abs() / *y as f64;
        assert!(drift < 0.02, "run {}: {x} vs {y} is {:.1}%", i + 1, drift * 100.0);
    }
    // What actually matters is where the campaign ends up, since prestige
    // compounds every run's error into the next.
    let end = (*a.last().unwrap() as f64 - *b.last().unwrap() as f64).abs() / *b.last().unwrap() as f64;
    assert!(end < 0.005, "campaign end drifted {:.2}%", end * 100.0);
}

/// **And on ticket 20's proposed curve it does not hold at all.**
///
/// Cheap locks are bought ~98 times a run instead of ~11, and every lock freezes
/// a rate that then pays for the rest of the run, scaled by prestige. So most of
/// the case for dropping `lock_cost_base` is an income source the design
/// removed, not the lock curve. This test exists to stop that being rediscovered
/// the hard way.
#[test]
fn the_withdrawn_sealed_income_dominates_the_proposed_curve() {
    let off = Tuning { lock_cost0: 500.0, lock_cost_base: 2.5, ..Default::default() };
    let on = Tuning { sealed_income: true, ..off.clone() };

    let (without, with) = (*peaks(&off, 6).last().unwrap(), *peaks(&on, 6).last().unwrap());
    assert!(
        with as f64 > without as f64 * 1.4,
        "expected the withdrawn mechanic to dominate: {without} without, {with} with"
    );

    // And the honest comparison — both curves without it — is a far smaller win
    // than ticket 20's headline suggests.
    let shipped =
        *peaks(&Tuning { lock_cost0: 500.0, lock_cost_base: 4.0, ..Default::default() }, 6)
            .last()
            .unwrap();
    let gain = (without as f64 - shipped as f64) / shipped as f64;
    assert!((0.0..0.25).contains(&gain), "fixing the lock curve alone buys {:.0}%", gain * 100.0);
}

/// Ticket 03's load-bearing claim, and the one the whole no-cap decision rests
/// on: entity count stays in the low hundreds whether the band is 20 floors or
/// 5,000, because cost tracks contested floors rather than band height.
///
/// The contrast has to be drawn on the **pre-ticket-20** curve, because that is
/// the one that produces a long band — climbers strung over hundreds of floors
/// behind a lock line that has fallen away. Ticket 20 landed a curve that keeps
/// the lock line near the frontier, so on the shipped tuning the band never
/// gets long enough to be a test of anything.
#[test]
fn band_height_buys_no_entities() {
    let t = Tuning { lock_cost0: 500.0, lock_cost_base: 4.0, ..Default::default() };
    let shallow = campaign(&t, 1, HOUR, greedy);
    let deep = campaign(&t, 8, HOUR, greedy);

    let spread =
        |o: &dot_tower_sim::Outcome| o.samples.iter().map(|s| s.climber_spread).max().unwrap_or(0);
    let entities =
        |o: &dot_tower_sim::Outcome| o.samples.iter().map(|s| s.entities).max().unwrap_or(0);

    let (s0, s1) = (spread(&shallow[0]), spread(deep.last().unwrap()));
    let (e0, e1) = (entities(&shallow[0]), entities(deep.last().unwrap()));

    assert!(s1 > s0 * 2, "the band should grow a lot: {s0} -> {s1}");
    assert!(
        (e1 as f64) < e0 as f64 * 1.5,
        "but entities should barely move: {e0} -> {e1} while the band went {s0} -> {s1}"
    );
}

/// And the consequence of ticket 20 for ticket 14's device session: the landed
/// curve does not merely stay inside the 200-450 entity range that ticket was
/// sized to test, it sits well under the bottom of it.
#[test]
fn the_landed_curve_stays_far_inside_the_device_budget() {
    let peak = campaign(&Tuning::default(), 8, HOUR, greedy)
        .iter()
        .flat_map(|o| &o.samples)
        .map(|s| s.entities)
        .max()
        .unwrap_or(0);
    assert!(peak < 250, "entity demand grew past what ticket 14 measured: {peak}");
}

/// ADR 0011: composition is authored, and the per-type ceiling is never the
/// crowd dial. Ticket 14 found that conditional on the lock curve and handed it
/// to ticket 20; ticket 20 landed `lock_outrun == 1.0`, and this is the
/// assertion that it holds.
///
/// The ceiling still binds in the opening minutes of a run, before any lock has
/// been bought and while the stream is filling from nothing. That is not the
/// failure ADR 0011 is about — what matters is that it does not bind **at
/// depth**, where the crowd is supposed to be the authored ratio and nothing
/// else.
#[test]
fn the_type_ceiling_does_not_bind_at_depth() {
    let landed = Tuning::default();
    let campaign = campaign(&landed, 6, HOUR, greedy);
    let deep = campaign.last().unwrap();

    let tail = &deep.samples[deep.samples.len() * 2 / 3..];
    let late: u64 = tail.iter().map(|s| s.cap_skips).sum();
    assert_eq!(late, 0, "the ceiling is binding at depth, so it is setting the crowd size");

    // And under the curve ticket 20 replaced, it bound constantly — which is
    // the finding, not an accident of this configuration.
    let before = Tuning { lock_cost0: 500.0, lock_cost_base: 4.0, ..Default::default() };
    let c = campaign_before(&before);
    assert!(c > 0, "the pre-ticket-20 curve should bind at depth; it is why the curve moved");
}

/// Ticket 15 measured buy-cheapest, buy-even and all-in-on-one-type spanning
/// 140 to 148 peak floor, because rank cost forces all three to be bought
/// regardless of intent. Gold competition is not a real decision.
#[test]
fn no_spending_strategy_pulls_away_from_the_others() {
    let t = Tuning::default();
    let run = |buy: BuyPolicy| {
        let mut p = Greedy::new(buy, Station::Frontier);
        dot_tower_sim::Run::new(t.clone(), 1.0).play(&mut p, HOUR).peak as f64
    };
    let results = [
        run(BuyPolicy::Cheapest),
        run(BuyPolicy::Even),
        run(BuyPolicy::Focus(ClimberType::Melee)),
        run(BuyPolicy::Focus(ClimberType::Ranged)),
    ];
    let lo = results.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = results.iter().cloned().fold(0.0, f64::max);
    assert!((hi - lo) / lo < 0.20, "spending strategies spread {lo}..{hi}, which is a real decision");
}
