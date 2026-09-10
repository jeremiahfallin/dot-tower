//! dot-tower.
//!
//! This file is assembly: the plugin set, the window, the platform workarounds
//! ticket 04 paid for on hardware, and the schedule that joins the simulation
//! to the screen. The game itself is in four modules, split along the seams the
//! tickets drew rather than by engine convention:
//!
//! - [`session`] — the loop. One `World`, one save slot, and the clock.
//! - [`column`] — the tower column: where and now.
//! - [`hud`] — the top bar, the spend block, the strip: how it has been going.
//! - [`prestige`] — the one modal surface, and the only irreversible act.
//!
//! The simulation lives in its own crate and knows nothing about Bevy. That is
//! ticket 22's insurance: `bevy_ui` is still in question on a shipping handset,
//! and if it resolves badly it must cost the presentation layer and nothing
//! else. Nothing in `crates/sim` may ever `use bevy::`.
//!
//! What is deliberately not here yet, so it is not mistaken for an oversight:
//! **the four hero abilities**. Ticket 10 fixed the bar's size and position and
//! ticket 09 showed the verdict line can carry "was that the right moment" —
//! but what the four abilities *do* is still in the map's fog, and the
//! simulation has no ability API. The bar is built to the settled layout and
//! does nothing.

pub mod art;
pub mod column;
pub mod hud;
pub mod prestige;
pub mod session;

use bevy::{
    image::ImagePlugin,
    log::{Level, LogPlugin},
    prelude::*,
    render::view::Msaa,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    ui_render::UiAntiAlias,
    window::WindowMode,
    winit::WinitSettings,
};
use dot_tower_sim::DT;

/// The frame everything sits in: the column's width, centred, in both frames.
#[derive(Component)]
pub struct Frame;

/// The four ability targets. Marked so the safe-area inset can keep them clear
/// of the gesture bar — ticket 10's one absolute rule about them.
#[derive(Component)]
pub struct AbilityBar;

/// The `bevy_main` proc macro generates the `android_main` entry point that
/// GameActivity's `System.loadLibrary` call ends up invoking. On desktop it
/// expands to nothing and `src/main.rs` calls this directly.
#[bevy_main]
pub fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            // PIXEL 11 WORKAROUND. On Tensor G6 the PowerVR driver's SPIR-V
            // compiler aborts inside `IMG_vkCreateComputePipelines` while
            // mangling an image type, killing the process on launch before
            // anything renders. Bevy already downgrades the Pixel 10 away from
            // the compute-driven culling path, but it does so with an exact
            // name match on "PowerVR D-Series DXT-48-1536 MC1"
            // (`bevy_render::get_pixel10_driver_version`), and this device
            // reports "PowerVR C-Series CXTP-48-1536 MC1", so the workaround
            // never fires and Bevy selects `GpuPreprocessingMode::Culling`.
            //
            // Constraining storage *textures* trips Bevy's `limit_support`
            // check (it wants >= 12) and lands on
            // `GpuPreprocessingMode::PreprocessingOnly` — exactly the mode the
            // Pixel 10 is demoted to, and a configuration known to run. The
            // game uses no storage textures. Remove once upstream broadens that
            // match. See ticket 24.
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                    // `limits` is only a *request* — Bevy resolves it against
                    // what the adapter reports and a constraint is lost.
                    // `constrained_limits` is applied afterwards via
                    // `or_worse_values_from`, so it wins regardless of adapter
                    // support (bevy_render/src/renderer/mod.rs:321).
                    constrained_limits: Some(bevy::render::settings::WgpuLimits {
                        max_storage_textures_per_shader_stage: 4,
                        ..default()
                    }),
                    ..default()
                })),
                ..default()
            })
            // Pixel art, so no filtering anywhere. The sprites are authored
            // 16x16 and drawn at 2x and 3x (ADR 0016); linear sampling would
            // undo the entire ticket 21 study.
            .set(ImagePlugin::default_nearest())
            .set(LogPlugin {
                // On Android these are routed to the native logger, so they
                // surface under `adb logcat -s dot-tower RustStdoutStderr`.
                level: Level::INFO,
                filter: "wgpu=error,naga=warn,bevy_render=info".to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "dot-tower".to_string(),
                    // Portrait phone shape, so the desktop window is a stand-in
                    // for the device. Ticket 02 settled that layout branches on
                    // aspect ratio, never `target_os` — this keeps that testable.
                    resolution: (480u32, 960u32).into(),
                    mode: if cfg!(target_os = "android") {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                    } else {
                        WindowMode::Windowed
                    },
                    ..default()
                }),
                ..default()
            }),
    )
    // Focused, the app runs at 60Hz; unfocused it idles at 1Hz — and that is
    // safe *because* losing focus stops the world outright (ticket 07: the
    // climb does not advance while away). If the sim ever ran unfocused, this
    // line would silently slow it down instead.
    .insert_resource(WinitSettings::mobile())
    // The tick is the simulation's, not the renderer's. Ticket 03 fixed 10Hz
    // and every number in `crates/sim` was measured at it, so the frame rate
    // must not be able to change what the tower does.
    .insert_resource(Time::<Fixed>::from_seconds(DT))
    .insert_resource(ClearColor(art::ui::BACKDROP))
    .init_resource::<column::View>()
    .init_resource::<prestige::Ledger>()
    .add_systems(Startup, (session::open_session, art::build_art, setup).chain())
    .add_systems(FixedUpdate, session::tick)
    .add_systems(
        Update,
        (
            (session::on_focus, session::on_lifecycle),
            safe_area,
            column::track_view,
            (
                column::draw_rows,
                column::draw_marks,
                column::draw_unit_images,
                column::draw_mark_text,
                column::draw_lock_chip,
                hud::draw_readouts,
                hud::draw_rank_chips,
                hud::draw_sparkline,
                prestige::draw_ledger,
            ),
        )
            .chain(),
    );

    app.run();
}

fn setup(mut commands: Commands, art: Res<art::Art>) {
    // `Msaa::Off` + `UiAntiAlias::Off` + `FontSmoothing::None` are the pixel-art
    // triple ticket 02 fixed from day one. `UiScale` stays at 1.0: probe D's
    // reading is still owed by the parked device tickets, and guessing a value
    // here would bake in a number nobody has looked at on a screen.
    commands.spawn((Camera2d, Msaa::Off, UiAntiAlias::Off));
    hud::spawn_screen(&mut commands, &art);
    prestige::spawn_ledger(&mut commands);
}

/// Insets the frame to the display's safe area.
///
/// Ticket 02 established there is no upstream option — bevy#23003 is blocked on
/// a winit beta — so these are the ~40 hand-rolled lines it predicted. Only two
/// edges dock anything (rejecting ticket 10's rail is what bought that), and
/// insetting them also mitigates the edge-coordinate misreporting in
/// winit#7528, so it is worth doing for two reasons.
fn safe_area(windows: Query<&Window>, mut frame: Query<&mut Node, With<Frame>>) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let Ok(mut node) = frame.single_mut() else {
        return;
    };
    let scale = window.scale_factor().max(0.001);
    let (top, bottom) = match read_content_rect() {
        Some((_, rect_top, _, rect_bottom)) => (
            rect_top as f32 / scale,
            ((window.physical_height() as i32 - rect_bottom).max(0)) as f32 / scale,
        ),
        None => (0.0, 0.0),
    };
    if node.padding.top != px(top) {
        node.padding.top = px(top);
    }
    if node.padding.bottom != px(bottom) {
        node.padding.bottom = px(bottom);
    }
}

/// The raw `content_rect` as `(left, top, right, bottom)` in physical pixels.
#[cfg(target_os = "android")]
fn read_content_rect() -> Option<(i32, i32, i32, i32)> {
    let app = bevy::android::ANDROID_APP.get()?;
    let rect = app.content_rect();
    Some((rect.left, rect.top, rect.right, rect.bottom))
}

#[cfg(not(target_os = "android"))]
fn read_content_rect() -> Option<(i32, i32, i32, i32)> {
    None
}
