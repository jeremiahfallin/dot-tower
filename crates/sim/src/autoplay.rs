//! A scripted player, for measuring with.
//!
//! The throwaway model baked its buy policy and hero stationing into the tick.
//! They do not belong there: the shipping game's purchases come from taps, and
//! a simulation that cannot be driven by a human is not the shipping
//! simulation. Everything here implements [`Player`] and the world does not
//! know which kind it has.
//!
//! Note what is *not* here: a stall detector. Ticket 06's "peak flat for six
//! minutes" is ±40% noise at depth, and ticket 19 found it did not merely add
//! noise but **inverted a headline result** — a variant that grinds rather than
//! stalls buys depth from run length alone. Pinning every run to identical
//! wall-clock time is the only comparison the model can be trusted for, so that
//! is what the harness does, and [ticket 18](../../../.scratch/dot-tower/issues/18-run-is-over.md)
//! still owns what the shipped game will use.

use crate::curves::{hero_cost, rank_cost};
use crate::geometry::Floor;
use crate::types::ClimberType;
use crate::world::{Player, World};

/// How a scripted player spends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuyPolicy {
    /// Buy the cheapest thing you can afford, forever. What ticket 06 measured
    /// and what a real player mostly does.
    #[default]
    Cheapest,
    /// Keep the three climber ranks level.
    Even,
    /// Rank one type first, always.
    Focus(ClimberType),
    /// Cheapest, but never buy a lock. The control for ticket 20.
    NoLock,
    /// Buy the lock whenever it is affordable, cheapest otherwise.
    ///
    /// This exists for ticket 20's question 1, which asks whether a lock is
    /// bought *the moment it is affordable*. Under [`Self::Cheapest`] that can
    /// never be observed: a lock priced above a rank is passed over until ranks
    /// grow past it, so the gate is rank cost rather than affordability. Only a
    /// player who actually wants the lock can measure whether gold was ever in
    /// the way.
    LockFirst,
}

/// Where a scripted player stations the hero.
///
/// Ticket 16 retired "below the wall it farms, at the wall it pushes":
/// stationing at the wall dominates on gold *and* experience, because depth
/// beats throughput. Placement is a learnable rule — put the hero where the
/// fighting is — so [`Self::Frontier`] is both the default and the honest one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Station {
    #[default]
    Frontier,
    /// A fixed number of floors below the frontier. Kept so ticket 16's
    /// measurement stays reproducible.
    Below(u32),
    /// No hero at all.
    Off,
}

/// The greedy scripted player: spends every second, stations every tick.
#[derive(Debug, Clone, Default)]
pub struct Greedy {
    pub buy: BuyPolicy,
    pub station: Station,
}

impl Greedy {
    pub fn new(buy: BuyPolicy, station: Station) -> Self {
        Self { buy, station }
    }
}

/// One thing that can be bought, and what it costs right now.
#[derive(Debug, Clone, Copy)]
enum Purchase {
    Rank(ClimberType),
    HeroLevel,
    Lock,
}

impl Player for Greedy {
    fn station(&mut self, world: &World) -> Option<Floor> {
        match self.station {
            Station::Off => None,
            Station::Frontier => Some(world.frontier()),
            Station::Below(n) => Some(world.frontier().saturating_sub(n).max(world.lock_line())),
        }
    }

    fn spend(&mut self, world: &mut World) {
        // Bounded, because buying a rank can make the next rank affordable and
        // a runaway would otherwise be an infinite loop rather than a visible
        // balance problem.
        for _ in 0..50 {
            let mut options: Vec<(Purchase, f64)> = ClimberType::ALL
                .into_iter()
                .map(|ty| (Purchase::Rank(ty), rank_cost(world.tuning(), ty, world.ranks()[ty])))
                .collect();
            options.push((Purchase::HeroLevel, hero_cost(world.tuning(), world.hero_level())));

            let (lock_floor, lock_price) = world.next_lock();
            if world.peak() >= lock_floor + world.tuning().lock_margin {
                options.push((Purchase::Lock, lock_price));
            }

            // Cheapest first; the policy then reorders, and affordability still
            // decides. A stable sort keeps the tie order fixed.
            options.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            match self.buy {
                BuyPolicy::Cheapest => {}
                BuyPolicy::NoLock => options.retain(|(p, _)| !matches!(p, Purchase::Lock)),
                BuyPolicy::Even => {
                    // Ticket 15 measured this against `Cheapest` and all-in-on-
                    // one-type: 140 to 148 peak floor, because rank cost forces
                    // all three to be bought regardless of intent.
                    options.sort_by_key(|(p, _)| match p {
                        Purchase::Rank(ty) => world.ranks()[*ty],
                        _ => u32::MAX,
                    });
                }
                BuyPolicy::LockFirst => {
                    options.sort_by_key(|(p, _)| !matches!(p, Purchase::Lock));
                }
                BuyPolicy::Focus(want) => {
                    options.sort_by_key(|(p, _)| !matches!(p, Purchase::Rank(ty) if *ty == want));
                }
            }

            let gold = world.gold();
            let Some(&(pick, _)) = options.iter().find(|(_, cost)| *cost <= gold) else {
                return;
            };
            let bought = match pick {
                Purchase::Rank(ty) => world.buy_rank(ty),
                Purchase::HeroLevel => world.buy_hero_level(),
                Purchase::Lock => world.buy_lock(),
            };
            if !bought {
                return;
            }
        }
    }
}
