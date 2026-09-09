//! Dumps one run's end state in the same shape as the throwaway JS model, so
//! the port stays checkable against the numbers the resolved tickets quote.
//!
//! The reference is `.scratch/dot-tower/prototypes/06-progression-curve/model.mjs`.
//! Run both pinned to the same wall clock — never to the stall heuristic, which
//! ticket 19 caught inverting a result.

use dot_tower_sim::autoplay::Greedy;
use dot_tower_sim::{play, Tuning, World};

fn main() {
    let minutes: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(60.0);
    let o = play(World::new(Tuning::default(), 1.0), &mut Greedy::default(), minutes * 60.0);
    let s = o.samples.last().expect("a run this long samples");

    println!("{{");
    println!("  \"seconds\": {:.1},", o.seconds);
    println!("  \"peak\": {},", o.peak);
    println!("  \"kills\": {},", o.totals.kills);
    println!("  \"deaths\": {},", o.totals.deaths());
    println!("  \"goldEarned\": {:.4e},", o.gold_earned);
    println!("  \"pop\": {},", s.pop);
    println!(
        "  \"ranks\": {{ \"melee\": {}, \"ranged\": {}, \"healer\": {} }},",
        s.ranks.melee, s.ranks.ranged, s.ranks.healer
    );
    println!("  \"heroLevel\": {},", s.hero_level);
    println!("  \"lockLevel\": {},", s.lock_level);
    println!(
        "  \"popByType\": {{ \"melee\": {}, \"ranged\": {}, \"healer\": {} }},",
        s.pop_by_type.melee, s.pop_by_type.ranged, s.pop_by_type.healer
    );
    println!("  \"entities\": {},", s.entities);
    println!("  \"contested\": {},", s.contested_floors);

    // Means over the whole run, which is the fair comparison: a single instant
    // of a cap-bound stream lands wherever the last death did.
    let n = o.samples.len() as f64;
    let mean = |f: fn(&dot_tower_sim::Sample) -> f64| o.samples.iter().map(f).sum::<f64>() / n;
    println!("  \"meanPop\": {:.2},", mean(|s| s.pop as f64));
    println!("  \"maxPop\": {},", o.samples.iter().map(|s| s.pop).max().unwrap_or(0));
    println!("  \"meanEntities\": {:.2},", mean(|s| s.entities as f64));
    println!("  \"maxEntities\": {},", o.samples.iter().map(|s| s.entities).max().unwrap_or(0));
    println!("  \"meanContested\": {:.2},", mean(|s| s.contested_floors as f64));
    println!("  \"meanSpread\": {:.2},", mean(|s| s.climber_spread as f64));
    println!("  \"capSkipsPerSample\": {:.2}", mean(|s| s.cap_skips as f64));
    println!("}}");
}
