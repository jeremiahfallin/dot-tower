//! The loop's contract, run without a window.
//!
//! These are the three joins in `src/session.rs` that are expensive to get
//! wrong and invisible when they are: the tick advancing at 10Hz, the world
//! stopping the moment focus is lost, and the save being written on ticket 12's
//! triggers. All three are wired to Bevy messages, so they are tested by
//! sending those messages rather than by clicking a window.
//!
//! The systems under test are the shipping ones — no test-only variant — with
//! the save directory pointed at a scratch path through [`SaveDir`], which is
//! the only seam.

use bevy::prelude::*;
use bevy::window::{AppLifecycle, WindowFocused};
use dot_tower::session::{on_focus, on_lifecycle, open_session, tick, SaveDir, Session};
use dot_tower_save::{SAVE_FILE, TIMER_INTERVAL_SECS};
use dot_tower_sim::DT;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("dot-tower-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn harness(dir: &std::path::Path) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<WindowFocused>()
        .add_message::<AppLifecycle>()
        .insert_resource(SaveDir(dir.to_path_buf()))
        .add_systems(Startup, open_session)
        .add_systems(FixedUpdate, tick)
        .add_systems(Update, (on_focus, on_lifecycle));
    app.update();
    app
}

/// Ticks the simulation directly, so the assertions are about the loop rather
/// than about how much wall-clock time a test runner happened to take.
fn sim_ticks(app: &mut App, n: usize) {
    for _ in 0..n {
        app.world_mut().run_schedule(FixedUpdate);
    }
}

fn elapsed(app: &App) -> f64 {
    app.world().resource::<Session>().world.elapsed()
}

#[test]
fn the_tick_is_ten_hertz_and_the_renderer_does_not_get_a_vote() {
    let dir = scratch("hz");
    let mut app = harness(&dir);

    sim_ticks(&mut app, 100);

    // 100 ticks is 10 simulated seconds at ticket 03's fixed timestep, however
    // long the frames took.
    assert!((elapsed(&app) - 100.0 * DT).abs() < 1e-9, "{}", elapsed(&app));
}

#[test]
fn losing_focus_stops_the_world_and_writes_the_save() {
    let dir = scratch("focus");
    let mut app = harness(&dir);
    sim_ticks(&mut app, 50);
    let before = elapsed(&app);

    let window = Entity::PLACEHOLDER;
    app.world_mut().write_message(WindowFocused { window, focused: false });
    app.update();

    // Ticket 12's primary trigger.
    assert!(dir.join(SAVE_FILE).exists(), "focus loss did not write the save");

    // ADR 0002 and ticket 07: the climb does not advance while away. If these
    // ticks moved the clock, away time would be paid for twice — once by the
    // simulation and again by the offline grant on return.
    sim_ticks(&mut app, 50);
    assert_eq!(elapsed(&app), before, "the world kept climbing while away");

    app.world_mut().write_message(WindowFocused { window, focused: true });
    app.update();
    sim_ticks(&mut app, 50);
    assert!(elapsed(&app) > before, "the world did not restart on return");
}

#[test]
fn android_suspend_is_the_same_door_as_focus_loss() {
    let dir = scratch("suspend");
    let mut app = harness(&dir);
    sim_ticks(&mut app, 10);
    let before = elapsed(&app);

    app.world_mut().write_message(AppLifecycle::Suspended);
    app.update();
    sim_ticks(&mut app, 30);

    assert!(dir.join(SAVE_FILE).exists(), "suspend did not write the save");
    assert_eq!(elapsed(&app), before, "the world kept climbing while suspended");

    app.world_mut().write_message(AppLifecycle::Running);
    app.update();
    sim_ticks(&mut app, 10);
    assert!(elapsed(&app) > before, "resuming did not restart the world");
}

#[test]
fn the_timer_is_the_floor_under_every_other_trigger() {
    let dir = scratch("timer");
    let mut app = harness(&dir);

    // Whatever startup wrote, the timer must write again on its own.
    let _ = std::fs::remove_file(dir.join(SAVE_FILE));
    sim_ticks(&mut app, (TIMER_INTERVAL_SECS / DT) as usize + 2);

    assert!(dir.join(SAVE_FILE).exists(), "the 60-second timer never fired");
}

/// Ticket 12: a run survives a restart in full — gold, ranks, lock line, peak.
/// The active band deliberately does not, which is ticket 07's rule and the
/// reason a resumed tower re-walks from the lock line.
#[test]
fn a_run_survives_being_closed_and_reopened() {
    let dir = scratch("resume");
    let mut app = harness(&dir);
    sim_ticks(&mut app, 600);

    let (peak, gold) = {
        let session = app.world().resource::<Session>();
        (session.world.peak(), session.world.gold())
    };
    assert!(peak > 1, "nothing climbed in a minute of simulated time");

    app.world_mut().write_message(WindowFocused { window: Entity::PLACEHOLDER, focused: false });
    app.update();
    drop(app);

    let reopened = harness(&dir);
    let session = reopened.world().resource::<Session>();
    assert_eq!(session.world.peak(), peak, "the peak floor did not survive");
    assert!(session.world.gold() >= gold, "gold did not survive");
    assert_eq!(session.world.elapsed(), 0.0, "the reopened run kept the old clock");
}
