//! Ticket 20 — travel time and the lock curve.
//!
//! Every run here is pinned to identical wall-clock time and `sealed_income` is
//! **off**, so these are the design as settled rather than the design plus the
//! mechanic ticket 06 withdrew. See the comment on ticket 20 for why that
//! distinction moved the ticket's headline number by 59%.
//!
//! Run with `cargo run --release -p dot-tower-sim --bin ticket20`.

use dot_tower_sim::autoplay::{BuyPolicy, Greedy, Station};
use dot_tower_sim::curves::head_start_floors;
use dot_tower_sim::{campaign, play, Outcome, Player, Tuning, World};

const HOUR: f64 = 3600.0;
const RUNS: usize = 6;

struct Reading {
    peak: u32,
    crowd: f64,
    transit: f64,
    back: f64,
    locks: usize,
    gold_wait: f64,
    automatic: f64,
    walking: f64,
    /// Split, because "the ceiling binds" means something different if it only
    /// binds in the first minutes of a run before any lock has been bought.
    skips_early: u64,
    skips_late: u64,
    head_start: f64,
    late_gain: u32,
}

fn measure(t: &Tuning, buy: BuyPolicy) -> Reading {
    let outcomes = campaign(t, RUNS, HOUR, || {
        Box::new(Greedy::new(buy, Station::Frontier)) as Box<dyn Player>
    });
    read(t, outcomes.last().unwrap())
}

fn read(t: &Tuning, o: &Outcome) -> Reading {
    // Means over the last third of the run: a single final instant of a
    // cap-bound stream lands wherever the last death did.
    let tail = &o.samples[o.samples.len() * 2 / 3..];
    let n = tail.len() as f64;
    let mean = |f: fn(&dot_tower_sim::Sample) -> f64| tail.iter().map(f).sum::<f64>() / n;

    let locks = &o.locks;
    let (gold_wait, automatic) = if locks.is_empty() {
        (f64::NAN, f64::NAN)
    } else {
        let total: f64 = locks.iter().map(|l| l.gold_wait).sum();
        // One second is the scripted player's decision cadence, so anything
        // under it means gold was never in the way.
        let auto = locks.iter().filter(|l| l.gold_wait <= 1.0).count() as f64;
        (total / locks.len() as f64, auto / locks.len() as f64)
    };

    // Is the climb still moving at the hour mark? Floors gained in the last ten
    // minutes, which is a run-length reading that needs no stall heuristic.
    let ten_min_ago = o.samples.iter().rev().find(|s| s.t <= o.seconds - 600.0);
    let late_gain = o.peak - ten_min_ago.map_or(o.peak, |s| s.peak);

    Reading {
        peak: o.peak,
        crowd: mean(|s| s.pop as f64),
        transit: mean(|s| (s.frontier - s.lock_line) as f64),
        back: mean(|s| (s.back_by_type.melee + s.back_by_type.ranged + s.back_by_type.healer) / 3.0),
        locks: locks.len(),
        gold_wait,
        automatic,
        walking: o.totals.walking_fraction(),
        skips_early: o.samples[..o.samples.len() / 3].iter().map(|s| s.cap_skips).sum(),
        skips_late: tail.iter().map(|s| s.cap_skips).sum(),
        head_start: head_start_floors(t, o.prestige_mult_in),
        late_gain,
    }
}

fn header(title: &str, first_col: &str) {
    println!("\n{title}");
    println!(
        "{:>10}  {:>5}  {:>6}  {:>5}  {:>7}  {:>5}  {:>5}  {:>9}  {:>5}  {:>5}  {:>11}  {:>6}  {:>5}",
        first_col, "k", "peak", "crowd", "transit", "back", "locks", "goldwait", "auto", "walk",
        "skips e/l", "head", "+10m",
    );
}

fn row(label: String, k: f64, r: &Reading) {
    println!(
        "{:>10}  {:>5.2}  {:>6}  {:>5.0}  {:>7.0}  {:>5.0}  {:>5}  {:>9}  {:>5}  {:>4.0}%  {:>11}  {:>6}  {:>5}",
        label,
        k,
        r.peak,
        r.crowd,
        r.transit,
        r.back,
        r.locks,
        if r.gold_wait.is_nan() { "-".into() } else { format!("{:.1}s", r.gold_wait) },
        if r.automatic.is_nan() { "-".into() } else { format!("{:.0}%", r.automatic * 100.0) },
        r.walking * 100.0,
        format!("{}/{}", r.skips_early, r.skips_late),
        format!("{:.0}%", 100.0 * r.head_start / r.peak as f64),
        r.late_gain,
    );
}

fn main() {
    // The sweeps vary the lock curve, so everything else must be held at what it
    // was when the question was asked — including `lock_cost0`, which sweep C
    // then varies on its own.
    let base = Tuning { lock_cost0: 500.0, ..Tuning::default() };
    let per_lock = base.income_per_lock();
    println!("ticket 20 — travel time and the lock curve");
    println!("{RUNS} runs x 60 min, wall-clock pinned, sealed_income OFF (ticket 06 withdrew it)");
    println!("all readings are run {RUNS}; crowd/transit/back are means over its last 20 minutes");
    println!(
        "k = lock_cost_base / gold_base^10 = lock_cost_base / {per_lock:.3}; k>1 means lock cost outruns income"
    );

    // --- A. the growth rate ---
    header(
        "A. lock_cost_base — how lock cost grows against income (buy policy: cheapest)",
        "cost_base",
    );
    for lock_cost_base in [2.0, 2.25, per_lock, 2.75, 3.0, 3.5, 4.0, 4.5] {
        let t = Tuning { lock_cost_base, ..base.clone() };
        row(format!("{lock_cost_base:.3}"), lock_cost_base / per_lock, &measure(&t, BuyPolicy::Cheapest));
    }

    // --- B. the same, by a player who actually wants the lock ---
    header(
        "B. the same curve, buy policy: lock-first — this is where 'automatic' is readable",
        "cost_base",
    );
    for lock_cost_base in [2.0, 2.25, per_lock, 2.75, 3.0, 3.5, 4.0, 4.5] {
        let t = Tuning { lock_cost_base, ..base.clone() };
        row(format!("{lock_cost_base:.3}"), lock_cost_base / per_lock, &measure(&t, BuyPolicy::LockFirst));
    }

    // --- C. the absolute price, at a scale-invariant growth rate ---
    header("C. lock_cost0 — the absolute price, at k = 1.00 (lock-first)", "cost0");
    for lock_cost0 in [125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0] {
        let t = Tuning { lock_cost_base: per_lock, lock_cost0, ..base.clone() };
        row(format!("{lock_cost0:.0}"), 1.0, &measure(&t, BuyPolicy::LockFirst));
    }

    // --- Q4: does the lock curve end the run? ---
    println!("\nE. run length — is the climb still moving? floors gained in each 10-minute window");
    println!("{:>10}  {:>5}  {:>6}  {:>44}", "cost_base", "k", "peak", "gain per 10 min, minutes 0-240");
    for lock_cost_base in [per_lock, 3.0, 4.0] {
        let t = Tuning { lock_cost_base, ..base.clone() };
        // A single long run at the account maturity run 6 represents, so the
        // question is about depth rather than about a fresh account.
        let mature = campaign(&t, 5, HOUR, || {
            Box::new(Greedy::new(BuyPolicy::LockFirst, Station::Frontier)) as Box<dyn Player>
        });
        let m = mature.last().unwrap().prestige_mult_in * mature.last().unwrap().prestige_mult_earned;
        let long = play(
            World::new(t.clone(), m),
            &mut Greedy::new(BuyPolicy::LockFirst, Station::Frontier),
            4.0 * HOUR,
        );
        let mut gains = Vec::new();
        let mut prev = 1u32;
        for w in 1..=24 {
            let at = long.samples.iter().rev().find(|s| s.t <= w as f64 * 600.0);
            let p = at.map_or(prev, |s| s.peak);
            gains.push(format!("{:>4}", p - prev));
            prev = p;
        }
        println!("{:>10.3}  {:>5.2}  {:>6}  {}", lock_cost_base, lock_cost_base / per_lock, long.peak, gains.join(""));
    }

    // --- D. how closely the line is allowed to track the frontier ---
    header("D. lock_margin — how far the climb must clear a floor before sealing it", "margin");
    for lock_margin in [0, 5, 10, 15, 25, 40] {
        let t = Tuning { lock_cost_base: per_lock, lock_margin, ..base.clone() };
        row(format!("{lock_margin}"), 1.0, &measure(&t, BuyPolicy::LockFirst));
    }
}
