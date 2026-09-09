//! Runs the simulation with no engine attached, and prints what it found.
//!
//! Every run is played to a **fixed wall-clock length**. Ticket 19's hardest
//! lesson: a variant that grinds rather than stalls reaches a deeper floor for
//! that reason alone, and comparing on a stall heuristic inverted a headline
//! result. Time is held constant here and there is no stall detector.
//!
//! ```text
//! headless                          # a 6-run campaign on the shipped curve
//! headless --runs 10 --minutes 60
//! headless --tuning sweeps/fixed-lock.ron
//! ```

use dot_tower_sim::autoplay::Greedy;
use dot_tower_sim::curves::head_start_floors;
use dot_tower_sim::{campaign, Player, Tuning};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runs = 6usize;
    let mut minutes = 60.0f64;
    let mut path: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--runs" => runs = args.next().ok_or("--runs needs a number")?.parse()?,
            "--minutes" => minutes = args.next().ok_or("--minutes needs a number")?.parse()?,
            "--tuning" => path = Some(args.next().ok_or("--tuning needs a path")?),
            "-h" | "--help" => {
                eprintln!("headless [--runs N] [--minutes M] [--tuning FILE]");
                return Ok(());
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }

    let tuning = match &path {
        Some(p) => Tuning::load(p)?,
        None => Tuning::default(),
    };
    println!(
        "tuning: {}   lock outrun x{:.3}   replacement {}s   composition {}/{}/{}",
        path.as_deref().unwrap_or("<shipped defaults>"),
        tuning.lock_outrun(),
        tuning.replacement,
        tuning.composition.melee,
        tuning.composition.ranged,
        tuning.composition.healer,
    );
    for w in tuning.warnings() {
        println!("  ! {w}");
    }
    println!("{runs} runs of {minutes:.0} minutes each, wall-clock pinned\n");

    let outcomes = campaign(&tuning, runs, minutes * 60.0, || {
        Box::new(Greedy::default()) as Box<dyn Player>
    });

    println!(
        "{:>4}  {:>7}  {:>9}  {:>6}  {:>5}  {:>5}  {:>7}  {:>6}  {:>5}  {:>5}",
        "run", "peak", "head strt", "crowd", "m/r/h", "back", "entities", "cntstd", "skips", "locks",
    );
    for (i, o) in outcomes.iter().enumerate() {
        let tail = &o.samples[o.samples.len() * 2 / 3..];
        let tail_mean = |f: fn(&dot_tower_sim::Sample) -> f64| {
            tail.iter().map(f).sum::<f64>() / tail.len() as f64
        };
        let last = o.samples.last();
        let peak_entities = o.samples.iter().map(|s| s.entities).max().unwrap_or(0);
        let (crowd, mix, back, contested, skips, locks) = match last {
            Some(s) => (
                tail_mean(|s| s.pop as f64),
                format!(
                    "{:.0}/{:.0}/{:.0}",
                    tail_mean(|s| s.pop_by_type.melee as f64),
                    tail_mean(|s| s.pop_by_type.ranged as f64),
                    tail_mean(|s| s.pop_by_type.healer as f64)
                ),
                tail_mean(|s| {
                    (s.back_by_type.melee + s.back_by_type.ranged + s.back_by_type.healer) / 3.0
                }),
                s.contested_floors,
                tail.iter().map(|s| s.cap_skips).sum::<u64>(),
                s.lock_level,
            ),
            None => (0.0, "-".into(), 0.0, 0, 0, 0),
        };
        println!(
            "{:>4}  {:>7}  {:>9.1}  {:>6.0}  {:>5}  {:>5.0}  {:>7}  {:>6}  {:>5}  {:>5}",
            i + 1,
            o.peak,
            head_start_floors(&tuning, o.prestige_mult_in),
            crowd,
            mix,
            back,
            peak_entities,
            contested,
            skips,
            locks,
        );
    }

    let max_entities = outcomes.iter().flat_map(|o| &o.samples).map(|s| s.entities).max().unwrap_or(0);
    // Late-run only. The ceiling always binds in the opening minutes, while the
    // stream fills from nothing and no lock has been bought yet; ADR 0011 is
    // about whether it binds at depth.
    let total_skips: u64 = outcomes
        .iter()
        .flat_map(|o| &o.samples[o.samples.len() * 2 / 3..])
        .map(|s| s.cap_skips)
        .sum();
    println!();
    println!("peak entities across the campaign: {max_entities}  (ticket 14 measured 201; the device range to test is 200-450)");
    println!("replacement slots skipped at a type ceiling, at depth: {total_skips}");
    if total_skips > 0 {
        // ADR 0011 says the ceiling must never be the crowd dial. Saying so here
        // is cheaper than rediscovering it in a sweep six weeks from now.
        println!("  ^ ADR 0011 requires this to be zero at depth. A non-zero count means the");
        println!("    per-type ceiling is setting the crowd size rather than the authored ratio.");
    }
    Ok(())
}
