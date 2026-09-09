//! The dot-tower simulation.
//!
//! Pure Rust. No Bevy, no rendering, no I/O beyond reading a tuning file — and
//! that boundary is enforced by the crate graph rather than by discipline.
//! [Ticket 22](../../.scratch/dot-tower/issues/22-bevy-ui-android-rendering.md)
//! leaves `bevy_ui` in question on a shipping handset with no obvious
//! replacement, and ticket 04 deliberately declined to front-run the engine
//! choice. If that resolves badly it should cost the presentation layer and
//! nothing else, which is what this crate is for.
//!
//! It has a second job. Ticket 19 caught the throwaway JS model *inverting a
//! headline result*, and ticket 14 found it had never implemented ticket 03's
//! floor-reset rule at all. The map's fog says outright that if that model
//! becomes a kept tool it needs checking against the decisions it is used to
//! defend. So this is both the game's simulation and the instrument for
//! checking it — and it earns that by reproducing the reference model rather
//! than by replacing it. See `sweeps/model-parity.ron`.
//!
//! **A reading is only meaningful with its method attached.** [`play`] pins the
//! wall clock, because ticket 19 showed the stall heuristic cannot be compared
//! across variants. But a pin shorter than a mature run's natural length reports
//! the stream's opening transient and calls it depth — under ADR 0013 a mature
//! run lasts 100-180 minutes, so an hour is a third of one. Pin to compare
//! curves; run long to characterise one.
//!
//! ```no_run
//! use dot_tower_sim::{autoplay::Greedy, play, Tuning, World};
//!
//! let outcome = play(World::new(Tuning::default(), 1.0), &mut Greedy::default(), 3600.0);
//! println!("peak floor {}", outcome.peak);
//! ```

pub mod autoplay;
pub mod curves;
pub mod geometry;
pub mod metrics;
pub mod save;
pub mod tuning;
pub mod types;
pub mod world;

pub use geometry::{Floor, FloorPos, NormX};
pub use metrics::{LockEvent, Sample, Totals};
pub use save::{Account, Ledger, Run, SaveGame};
pub use tuning::Tuning;
pub use types::{ClimberType, PerType};
pub use world::{Climber, Player, World, DT};

use curves::prestige_mult_for;

/// Plays a world for `seconds` of simulated time, sampling every 10s.
///
/// A free function rather than a type: `CONTEXT.md` gives **run** a specific
/// meaning — the span of play between two prestiges — and that noun belongs to
/// [`Run`], the thing prestige resets. A second type competing for it was worth
/// no more than the function it wraps.
///
/// Fixed length is the point. Ticket 19: a variant that grinds rather than
/// stalls reaches a deeper floor *for that reason alone*, which made one result
/// look like +29 floors until every run was pinned to identical time, at which
/// point it reversed. Any comparison this crate is used for must hold time
/// constant.
pub fn play(world: World, player: &mut impl Player, seconds: f64) -> Outcome {
    play_sampling_every(world, player, seconds, 10.0)
}

pub fn play_sampling_every(
    mut world: World,
    player: &mut impl Player,
    seconds: f64,
    sample_every: f64,
) -> Outcome {
    let prestige_mult_in = world.prestige_mult_pub();
    let mut samples = Vec::new();
    let mut last_sample = 0.0;

    while world.elapsed() < seconds {
        // A sample is the state *after* the tick that began at `tick_start`,
        // labelled with that start time. Testing the interval against the
        // post-tick clock instead would fire two ticks early and label every
        // sample one tick late — small enough to look like rounding, and enough
        // to make a trace disagree with the reference model for no real reason.
        let tick_start = world.elapsed();
        world.step(player);
        if tick_start - last_sample >= sample_every {
            let mut s = world.sample();
            s.t = tick_start;
            samples.push(s);
            last_sample = tick_start;
        }
    }

    let peak = world.peak();
    Outcome {
        seconds: world.elapsed(),
        peak,
        gold_earned: world.totals().gold_earned,
        totals: world.totals().clone(),
        prestige_mult_in,
        prestige_mult_earned: prestige_mult_for(world.tuning(), peak),
        run: world.run().clone(),
        samples,
        locks: world.locks().to_vec(),
    }
}

/// What a run produced.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub seconds: f64,
    pub peak: Floor,
    pub gold_earned: f64,
    pub totals: Totals,
    pub prestige_mult_in: f64,
    /// What this run's peak floor is worth as a single prestige. Shown to the
    /// player; the cumulative product never is (ADR 0007).
    pub prestige_mult_earned: f64,
    /// The run as the save would write it — which is how a campaign carries
    /// state from one run to the next.
    pub run: Run,
    pub samples: Vec<Sample>,
    /// Every lock bought, with what the purchase was waiting on.
    pub locks: Vec<LockEvent>,
}

/// A sequence of runs with prestige between them.
///
/// Prestige is [`SaveGame::prestige`] and nothing more (ticket 12): the set of
/// things it resets **is** [`SavedRun`], so there is no field list here to keep
/// in step with the ledger. Running a campaign through the save type rather than
/// through a loose `f64` is what keeps that true — if the two ever disagree, it
/// shows up here first.
pub fn campaign(
    tuning: &Tuning,
    runs: usize,
    seconds_per_run: f64,
    make_player: impl Fn() -> Box<dyn Player>,
) -> Vec<Outcome> {
    let mut out = Vec::with_capacity(runs);
    let mut save = SaveGame::default();

    for _ in 0..runs {
        let mut player = make_player();
        let world = World::resume(tuning.clone(), save.account.clone(), save.run.clone());
        let outcome = play(world, &mut player, seconds_per_run);

        save.run = outcome.run.clone();
        save.prestige(tuning);
        out.push(outcome);
    }
    out
}

impl Player for Box<dyn Player> {
    fn station(&mut self, world: &World) -> Option<Floor> {
        (**self).station(world)
    }
    fn spend(&mut self, world: &mut World) {
        (**self).spend(world)
    }
}
