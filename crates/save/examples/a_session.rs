//! Plays a session, saves it, closes the game, comes back later, and prestiges.
//!
//! The point is to exercise the seam end to end outside a game loop: what the
//! save carries, what it deliberately does not, and what the player is told.
//!
//! `cargo run -p dot-tower-save --example a_session`

use dot_tower_save::{Loaded, SaveSlot, Trigger};
use dot_tower_sim::autoplay::Greedy;
use dot_tower_sim::save::SLICE_HERO;
use dot_tower_sim::{play, Tuning, World};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tuning = Tuning::default();
    let dir = std::env::temp_dir().join(format!("dot-tower-session-{}", std::process::id()));
    let mut slot = SaveSlot::open(&dir)?;
    println!("save directory: {}\n", dir.display());

    // --- first launch ---
    let loaded = slot.load();
    println!("first launch      -> {loaded:?}");
    println!("  told the player : {:?}\n", loaded.notice(&tuning));

    let mut save = loaded.save().cloned().unwrap_or_default();

    // --- a session ---
    let world = World::resume(tuning.clone(), save.account.clone(), save.run.clone());
    let outcome = play(world, &mut Greedy::default(), 30.0 * 60.0);
    save.run = outcome.run.clone();
    println!("played 30 minutes -> floor {}, {:.0} gold", save.run.peak, save.run.gold);
    println!("  best rate       : {:.0} gold/sec (what offline pays at)", save.run.best_rate);
    println!(
        "  hero            : {SLICE_HERO} level {}, stationed on floor {}\n",
        save.run.heroes[SLICE_HERO].level,
        save.run.stationed.as_ref().map_or(0, |(_, f)| *f),
    );

    slot.write(&save, Trigger::FocusLost)?;

    // --- away for six hours ---
    let reopened = slot.load();
    let mut save = match reopened {
        Loaded::Restored { save, .. } => *save,
        other => panic!("expected a clean load, got {other:?}"),
    };
    let six_hours_later = save.written_at_unix_millis + 6 * 3600 * 1000;
    let grant = save.apply_offline_grant(six_hours_later, &tuning);
    println!("away six hours    -> {:.0} gold ({} the cap)", grant.gold, if grant.capped { "over" } else { "under" });
    println!("  the climb did not advance: still floor {}\n", save.run.peak);

    // --- what prestige would cost, shown before committing ---
    let ledger = {
        let mut preview = save.clone();
        preview.prestige(&tuning)
    };
    println!("prestige ledger   -> earns x{:.2} this run", ledger.earned_multiplier);
    println!(
        "  head start      : {:.0} floors -> {:.0} floors",
        ledger.head_start_before, ledger.head_start_after
    );
    println!("  first time      : {}", ledger.first_time);
    println!("  gives up:");
    for (what, change) in &ledger.losses {
        println!("      {what:<16} {change}");
    }

    save.prestige(&tuning);
    slot.write(&save, Trigger::Prestige)?;
    println!("\nafter prestige    -> floor {}, {:.0} gold, {} prestiges",
        save.run.peak, save.run.gold, save.account.prestige_count);
    println!("  account kept    : head start {:.0} floors",
        dot_tower_sim::curves::head_start_floors(&tuning, save.account.cumulative_prestige_mult));

    std::fs::remove_dir_all(&dir).ok();
    Ok(())
}
