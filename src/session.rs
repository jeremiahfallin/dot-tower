//! The loop: one [`World`], one save slot, and the clock that joins them.
//!
//! Everything here is wiring rather than design — every rule it enforces was
//! settled by a ticket, and the comments say which, because the cost of getting
//! one of them subtly wrong is a save that quietly rewinds someone.
//!
//! Three joins are load-bearing:
//!
//! - **The tick is fixed at 10Hz and is the sim's, not the renderer's.**
//!   [`DT`] is 0.1s and the world's numbers were all measured at it, so the
//!   frame rate must not be able to change what the tower does. Bevy's
//!   `FixedUpdate` runs the catch-up; `Time<Virtual>`'s max delta bounds it.
//! - **Away time is paid for, never simulated.** ADR 0002 and ticket 07: the
//!   climb does not advance while away, so losing focus *pauses* the world and
//!   regaining it credits [`SaveGame::offline_grant`]. Letting the sim catch up
//!   instead would advance the climb — cheap for a minute, wrong at every
//!   length, and a different game after an overnight.
//! - **The save is written on ticket 12's four triggers and no others.**
//!   Focus loss, suspend, a 60-second timer, and prestige.

use bevy::prelude::*;
use bevy::window::{AppLifecycle, WindowFocused};
use dot_tower_sim::save::{OfflineGrant, SaveGame};
use dot_tower_sim::{curves, ClimberType, Floor, Player, Tuning, World, DT};
use dot_tower_save::{now_unix_millis, SaveSlot, Trigger, TIMER_INTERVAL_SECS};

/// How much of the uptime sparkline the strip shows (ticket 09).
pub const HOLDING_WINDOW: f64 = 45.0;
const HOLDING_SAMPLES: usize = (HOLDING_WINDOW / DT) as usize;

/// Ticket 07: under five minutes away, credit silently. Over it, say so.
const BANNER_AFTER_SECS: f64 = 300.0;

/// What the player's hands do to the world.
///
/// Only [`Player::station`] is implemented. Purchases deliberately do **not**
/// go through [`Player::spend`]: the world calls that once per simulated second
/// because the scripted player's spend curve is sensitive to that cadence, and
/// a human who taps a rank must see the price change on the same frame they
/// touched it. So taps call [`World::buy_rank`] and friends directly, between
/// ticks, which is the same seam by another door.
#[derive(Default)]
pub struct Hands {
    /// Where the player has put the hero, or `None` before they have chosen.
    /// Returned every tick rather than once, so the world's own clamping (a
    /// rising lock line, a dead hero refusing to move) stays authoritative.
    pub station: Option<Floor>,
}

impl Player for Hands {
    fn station(&mut self, _world: &World) -> Option<Floor> {
        self.station
    }
}

/// Where the save lives.
///
/// A resource rather than a call to [`save_dir`] inside the startup system, so
/// the loop can be run against a scratch directory in a test. Everything else
/// about the save path is identical in both cases — a test that has to take a
/// different code path proves less than one that does not.
#[derive(Resource, Clone)]
pub struct SaveDir(pub std::path::PathBuf);

impl Default for SaveDir {
    fn default() -> Self {
        Self(save_dir())
    }
}

/// Why the world is not ticking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asleep {
    /// The window lost focus, or Android suspended us.
    Away,
    /// The prestige ledger is open. A modal that keeps running underneath is a
    /// modal the player is punished for reading (ticket 08).
    Deciding,
}

/// The one thing everything else reads.
#[derive(Resource)]
pub struct Session {
    pub world: World,
    pub hands: Hands,
    pub tuning: Tuning,
    slot: SaveSlot,
    /// Set while the world is not advancing, with the wall clock at which it
    /// stopped — which is what the offline grant is measured from.
    asleep: Option<(Asleep, u64)>,
    since_save: f64,
    /// Ticket 12's notice, in the game's own terms. Shown once.
    pub notice: Option<String>,
    /// Ticket 07's return banner, and when it appeared.
    pub banner: Option<(String, f64)>,
    /// Hero aliveness, one entry per tick, oldest first. The strip's holding
    /// sparkline; nothing else in the game keeps history.
    pub holding: std::collections::VecDeque<bool>,
    /// Whether the last write was refused, so the UI can say so rather than
    /// pretending it saved.
    pub writes_refused: bool,
    /// The highest floor holding climbers, and how long it has been there.
    ///
    /// The wall's hatching is drawn from this rather than from
    /// [`World::seconds_since_peak_gain`], and the difference matters on the
    /// path players take most: after a resume the active band is dropped and
    /// re-walks (ticket 07), so the run's *peak* cannot move for minutes while
    /// the climb is plainly advancing. Reading the peak there would hatch a
    /// wall that is not there.
    frontier: dot_tower_sim::Floor,
    frontier_since: f64,
}

impl Session {
    /// Fraction of the last [`HOLDING_WINDOW`] the hero was alive for.
    pub fn holding(&self) -> f64 {
        if self.holding.is_empty() {
            return 1.0;
        }
        self.holding.iter().filter(|up| **up).count() as f64 / self.holding.len() as f64
    }

    pub fn holding_samples(&self) -> &std::collections::VecDeque<bool> {
        &self.holding
    }

    /// How long the climb's leading edge has been stuck, in simulated seconds.
    /// Texture only: ADR 0015 retired stall detection from the design, so
    /// nothing may branch on this except how strongly the wall is drawn.
    pub fn seconds_stuck(&self) -> f64 {
        (self.world.elapsed() - self.frontier_since).max(0.0)
    }

    pub fn asleep(&self) -> Option<Asleep> {
        self.asleep.map(|(why, _)| why)
    }

    /// The save as it would be written this instant.
    pub fn snapshot(&self) -> SaveGame {
        SaveGame {
            account: self.world.account().clone(),
            run: self.world.run().clone(),
            ..SaveGame::default()
        }
    }

    /// What this run is worth if prestiged now, without committing to it.
    /// Ticket 08 shows the ledger before every prestige, so the preview and the
    /// commit must be the same computation — running it on a clone is what
    /// guarantees that rather than hoping two code paths agree.
    pub fn prestige_preview(&self) -> dot_tower_sim::Ledger {
        self.snapshot().prestige(&self.tuning)
    }

    /// Ends the run. The only irreversible act in the game, so it writes before
    /// it returns (ticket 12's fourth trigger).
    pub fn prestige(&mut self) -> dot_tower_sim::Ledger {
        let mut save = self.snapshot();
        let ledger = save.prestige(&self.tuning);
        self.write(&save, Trigger::Prestige);
        self.world = World::resume(self.tuning.clone(), save.account, save.run);
        self.hands.station = None;
        self.frontier = 0;
        self.frontier_since = 0.0;
        self.holding.clear();
        self.since_save = 0.0;
        ledger
    }

    fn write(&mut self, save: &SaveGame, trigger: Trigger) {
        match self.slot.write(save, trigger) {
            Ok(()) => {
                self.writes_refused = false;
                debug!("saved on {trigger:?}");
            }
            Err(err) => {
                // Disarmed writes are ticket 12's newer-save lock and are not a
                // bug; anything else is, and both must be visible rather than
                // swallowed, because the failure mode is silent lost progress.
                self.writes_refused = true;
                warn!("save on {trigger:?} refused: {err}");
            }
        }
    }

    fn save_now(&mut self, trigger: Trigger) {
        let save = self.snapshot();
        self.write(&save, trigger);
        self.since_save = 0.0;
    }

    /// Puts the world down, saving as it goes. Idempotent: focus loss and
    /// suspend both arrive on Android and the first one wins, so the offline
    /// clock starts at the moment the player actually left.
    fn sleep(&mut self, why: Asleep, trigger: Option<Trigger>) {
        if self.asleep.is_some() {
            return;
        }
        self.asleep = Some((why, now_unix_millis()));
        if let Some(trigger) = trigger {
            self.save_now(trigger);
        }
    }

    /// Wakes the world, paying for the time away.
    fn wake(&mut self) {
        let Some((why, at)) = self.asleep.take() else {
            return;
        };
        if why != Asleep::Away {
            return;
        }

        // The grant is measured from the moment we stopped ticking, not from
        // the save's own stamp: the two are milliseconds apart on this path,
        // and using our own is what keeps a refused write from also costing the
        // player their away time.
        let mut probe = self.snapshot();
        probe.written_at_unix_millis = at;
        let grant = probe.offline_grant(now_unix_millis(), &self.tuning);
        if grant.gold <= 0.0 {
            return;
        }
        self.world.credit_offline(grant.gold);
        if grant.elapsed_secs >= BANNER_AFTER_SECS {
            self.banner = Some((banner_line(&grant, &self.tuning, self.world.ranks()), 0.0));
        }
    }
}

/// Ticket 07's return line, and its comparator is **ranks** — gold is
/// meaningless at 10^31/s, "worth 8 hours of climbing" is circular, and floors
/// need the log conversion that reintroduces a second economy model.
fn banner_line(grant: &OfflineGrant, tuning: &Tuning, ranks: dot_tower_sim::PerType<u32>) -> String {
    // Cheapest-first, which is how the gold would actually be spent.
    let mut ranks = ranks;
    let mut purse = grant.gold;
    let mut bought = 0u32;
    while bought < 999 {
        let (ty, cost) = ClimberType::ALL
            .into_iter()
            .map(|ty| (ty, curves::rank_cost(tuning, ty, ranks[ty])))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .expect("three climber types");
        if cost > purse {
            break;
        }
        purse -= cost;
        ranks[ty] += 1;
        bought += 1;
    }

    let mins = (grant.paid_secs / 60.0) as u64;
    let away = if mins >= 60 {
        format!("{}h {:02}m", mins / 60, mins % 60)
    } else {
        format!("{mins}m")
    };
    let ranks = match bought {
        0 => "not yet a rank".to_string(),
        1 => "enough for 1 rank".to_string(),
        n => format!("enough for {n} ranks"),
    };
    let capped = if grant.capped { " (capped)" } else { "" };
    format!(
        "Away {away}{capped} - the tower kept fighting. {} gold, {ranks}.",
        crate::hud::compact(grant.gold)
    )
}

// --------------------------------------------------------------------------
// Startup
// --------------------------------------------------------------------------

/// Opens the save, pays for the time away, and builds the world to play.
///
/// Failure here is not fatal and must not be: a slot that cannot be opened
/// still yields a playable tower with writes refused, which is strictly better
/// than a game that will not start.
pub fn open_session(mut commands: Commands, dir: Option<Res<SaveDir>>) {
    let tuning = load_tuning();
    for warning in tuning.warnings() {
        warn!("tuning: {warning}");
    }

    let dir = dir.map(|d| d.0.clone()).unwrap_or_else(save_dir);
    let mut slot = match SaveSlot::open(&dir) {
        Ok(slot) => slot,
        Err(err) => {
            error!("save directory {} unusable: {err}", dir.display());
            // `SaveSlot::open` created nothing, so writes will keep failing and
            // keep saying so. The tower still runs.
            SaveSlot::open(std::env::temp_dir().join("dot-tower")).expect("temp dir is writable")
        }
    };

    let loaded = slot.load();
    let notice = loaded.notice(&tuning);
    let mut banner = None;
    let (account, run) = match loaded.save() {
        Some(save) => {
            let mut save = save.clone();
            let grant = save.apply_offline_grant(now_unix_millis(), &tuning);
            if grant.elapsed_secs >= BANNER_AFTER_SECS && grant.gold > 0.0 {
                banner = Some((banner_line(&grant, &tuning, save.run.ranks), 0.0));
            }
            info!(
                "resumed: floor {}, lock line {}, head start {:.0} floors",
                save.run.peak,
                save.run.lock_line(),
                curves::head_start_floors(&tuning, save.account.cumulative_prestige_mult),
            );
            (save.account, save.run)
        }
        None => {
            info!("new tower");
            Default::default()
        }
    };

    let station = run.stationed.as_ref().map(|(_, floor)| *floor);
    commands.insert_resource(Session {
        world: World::resume(tuning.clone(), account, run),
        hands: Hands { station },
        tuning,
        writes_refused: !slot.writes_armed(),
        slot,
        asleep: None,
        since_save: 0.0,
        notice,
        banner,
        holding: std::collections::VecDeque::with_capacity(HOLDING_SAMPLES),
        frontier: 0,
        frontier_since: 0.0,
    });
}

/// `tuning.ron` beside the binary or in the working directory, else the shipped
/// defaults — which state the same numbers, so the two cannot disagree about
/// what the game is. Android has no such file and always takes the default.
fn load_tuning() -> Tuning {
    for path in ["tuning.ron", "../tuning.ron"] {
        if std::path::Path::new(path).exists() {
            match Tuning::load(path) {
                Ok(tuning) => {
                    info!("tuning from {path}");
                    return tuning;
                }
                Err(err) => warn!("{path} unreadable ({err}); using shipped defaults"),
            }
        }
    }
    Tuning::default()
}

#[cfg(target_os = "android")]
fn save_dir() -> std::path::PathBuf {
    bevy::android::ANDROID_APP
        .get()
        .and_then(|app| app.internal_data_path())
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(not(target_os = "android"))]
fn save_dir() -> std::path::PathBuf {
    dot_tower_save::default_desktop_dir().unwrap_or_else(|| std::env::temp_dir().join("dot-tower"))
}

// --------------------------------------------------------------------------
// The tick
// --------------------------------------------------------------------------

/// One 10Hz tick, in `FixedUpdate`.
pub fn tick(mut session: Option<ResMut<Session>>) {
    let Some(session) = session.as_mut() else {
        return;
    };
    if session.asleep.is_some() {
        return;
    }
    let mut hands = std::mem::take(&mut session.hands);
    session.world.step(&mut hands);
    session.hands = hands;

    let front = session.world.frontier();
    if front > session.frontier {
        session.frontier = front;
        session.frontier_since = session.world.elapsed();
    }

    let alive = session.world.hero_alive();
    session.holding.push_back(alive);
    while session.holding.len() > HOLDING_SAMPLES {
        session.holding.pop_front();
    }

    session.since_save += DT;
    if session.since_save >= TIMER_INTERVAL_SECS {
        session.save_now(Trigger::Timer);
    }

    if let Some((_, age)) = session.banner.as_mut() {
        *age += DT;
    }
    if session.banner.as_ref().is_some_and(|(_, age)| *age > 12.0) {
        session.banner = None;
    }
}

/// Ticket 12's first trigger, and the pause that makes ticket 07 exact.
pub fn on_focus(mut focus: MessageReader<WindowFocused>, mut session: ResMut<Session>) {
    for event in focus.read() {
        if event.focused {
            session.wake();
        } else {
            session.sleep(Asleep::Away, Some(Trigger::FocusLost));
        }
    }
}

/// Ticket 12's second trigger. On Android this arrives synchronously while the
/// main thread is blocked in `surfaceDestroyed`; ticket 04 measured the write
/// plus `sync_all()` at 2ms on a Pixel 11, so it completes.
pub fn on_lifecycle(mut lifecycle: MessageReader<AppLifecycle>, mut session: ResMut<Session>) {
    for event in lifecycle.read() {
        match event {
            AppLifecycle::Suspended => session.sleep(Asleep::Away, Some(Trigger::Suspended)),
            AppLifecycle::Running => session.wake(),
            _ => {}
        }
    }
}

/// Stops the world while the ledger is open, and starts it again after.
pub fn set_deciding(session: &mut Session, deciding: bool) {
    if deciding {
        session.sleep(Asleep::Deciding, None);
    } else if session.asleep() == Some(Asleep::Deciding) {
        session.asleep = None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use dot_tower_sim::save::OfflineGrant;

    /// Ticket 07 rejected gold ("4.2M" is meaningless at 10^31/s) and hours
    /// ("worth 8 hours of climbing" is circular) in favour of **ranks**, which
    /// are exact, free to compute, and point at the decision the player is
    /// about to make. This is that arithmetic.
    #[test]
    fn the_return_line_counts_ranks_and_never_gold() {
        let tuning = Tuning::default();
        let grant = OfflineGrant {
            gold: curves::rank_cost(&tuning, ClimberType::Melee, 1) * 2.5,
            elapsed_secs: 8.0 * 3600.0 + 4.0 * 60.0,
            paid_secs: 8.0 * 3600.0 + 4.0 * 60.0,
            capped: false,
        };
        let line = banner_line(&grant, &tuning, dot_tower_sim::PerType::splat(1));

        assert!(line.starts_with("Away 8h 04m"), "{line}");
        assert!(line.contains("the tower kept fighting"), "{line}");
        assert!(line.contains("ranks"), "{line}");
    }

    #[test]
    fn a_grant_that_buys_nothing_still_says_so() {
        let tuning = Tuning::default();
        let grant = OfflineGrant {
            gold: 1.0,
            elapsed_secs: 400.0,
            paid_secs: 400.0,
            capped: false,
        };
        let line = banner_line(&grant, &tuning, dot_tower_sim::PerType::splat(1));
        // Suppressing on *amount* is the failure ticket 07 named: it hides the
        // mechanic exactly when the player is least sure it exists.
        assert!(line.contains("not yet a rank"), "{line}");
    }
}
