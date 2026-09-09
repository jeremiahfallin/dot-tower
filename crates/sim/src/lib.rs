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
//! [ticket 20](../../.scratch/dot-tower/issues/20-travel-time-and-the-lock-curve.md),
//! which is the frontier for desktop.
//!
//! ```no_run
//! use dot_tower_sim::{autoplay::Greedy, tuning::Tuning, Run};
//!
//! let outcome = Run::new(Tuning::default(), 1.0).play(&mut Greedy::default(), 60.0 * 60.0);
//! println!("peak floor {}", outcome.peak);
//! ```

pub mod autoplay;
pub mod curves;
pub mod geometry;
pub mod metrics;
pub mod tuning;
pub mod types;
pub mod world;

pub use geometry::{Floor, FloorPos, NormX};
pub use metrics::{LockEvent, Sample, Totals};
pub use tuning::Tuning;
pub use types::{ClimberType, PerType};
pub use world::{Climber, Player, World, DT};

use curves::{compound_prestige, prestige_mult_for};

/// A run played to a fixed wall-clock length, with samples taken along the way.
///
/// Fixed length is the point. Ticket 19: a variant that grinds rather than
/// stalls reaches a deeper floor *for that reason alone*, which made one result
/// look like +29 floors until every run was pinned to identical time, at which
/// point it reversed. Any comparison this crate is used for must hold time
/// constant.
pub struct Run {
    world: World,
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
    pub samples: Vec<Sample>,
    /// Every lock bought, with what the purchase was waiting on.
    pub locks: Vec<LockEvent>,
}

impl Run {
    pub fn new(tuning: Tuning, prestige_mult: f64) -> Self {
        Self { world: World::new(tuning, prestige_mult) }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Plays for `seconds` of simulated time, sampling every 10s.
    pub fn play(self, player: &mut impl Player, seconds: f64) -> Outcome {
        self.play_sampling_every(player, seconds, 10.0)
    }

    pub fn play_sampling_every(
        mut self,
        player: &mut impl Player,
        seconds: f64,
        sample_every: f64,
    ) -> Outcome {
        let prestige_mult_in = self.world.prestige_mult_pub();
        let mut samples = Vec::new();
        let mut last_sample = 0.0;

        while self.world.elapsed() < seconds {
            // A sample is the state *after* the tick that began at `tick_start`,
            // labelled with that start time. Testing the interval against the
            // post-tick clock instead would fire two ticks early and label
            // every sample one tick late — small enough to look like rounding,
            // and enough to make a trace disagree with the reference model for
            // no real reason.
            let tick_start = self.world.elapsed();
            self.world.step(player);
            if tick_start - last_sample >= sample_every {
                let mut s = self.world.sample();
                s.t = tick_start;
                samples.push(s);
                last_sample = tick_start;
            }
        }

        let peak = self.world.peak();
        Outcome {
            seconds: self.world.elapsed(),
            peak,
            gold_earned: self.world.totals().gold_earned,
            totals: self.world.totals().clone(),
            prestige_mult_in,
            prestige_mult_earned: prestige_mult_for(self.world.tuning(), peak),
            samples,
            locks: self.world.locks().to_vec(),
        }
    }
}

/// A sequence of runs with prestige between them.
///
/// Prestige is `run = Run::default()` and nothing more (ticket 12): the set of
/// things it resets **is** the type, which is why there is no field list here
/// to keep in step with the ledger.
pub fn campaign(
    tuning: &Tuning,
    runs: usize,
    seconds_per_run: f64,
    make_player: impl Fn() -> Box<dyn Player>,
) -> Vec<Outcome> {
    let mut out = Vec::with_capacity(runs);
    let mut cumulative = 1.0;
    let mut best_peak = 0;

    for _ in 0..runs {
        let mut player = make_player();
        let outcome = Run::new(tuning.clone(), cumulative).play(&mut player, seconds_per_run);
        best_peak = best_peak.max(outcome.peak);
        cumulative = compound_prestige(tuning, cumulative, outcome.prestige_mult_earned, best_peak);
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
