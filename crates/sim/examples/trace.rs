//! Prints a per-sample trace of one run, for lining up against the JS model.
use dot_tower_sim::autoplay::Greedy;
use dot_tower_sim::{Run, Tuning};

fn main() {
    let path = std::env::args().nth(1);
    let secs: f64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(900.0);
    let every: f64 = std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(10.0);
    let tuning = path.map_or_else(Tuning::default, |p| Tuning::load(p).expect("tuning"));
    let o = Run::new(tuning, 1.0).play_sampling_every(&mut Greedy::default(), secs, every);
    for (i, s) in o.samples.iter().enumerate() {
        let _ = i;
        println!("t={:>7.1} peak={:>4} lock={:>4} ranks={}/{}/{} hero={} gold={:.2}", s.t, s.peak, s.lock_line, s.ranks.melee, s.ranks.ranged, s.ranks.healer, s.hero_level, s.gold);
    }
}
