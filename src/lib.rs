//! dot-tower — Android bringup harness.
//!
//! This is not the game. It is the instrument for wayfinder ticket 04: a
//! minimal Bevy app whose only job is to put a real Android device in the loop
//! and answer, empirically, the questions that tickets 01, 02 and 13 could only
//! settle from source. Each probe below is labelled with the question it
//! answers so the readings can be transcribed straight onto the ticket.
//!
//! Probes:
//!   A. Touch transport   — does input arrive, per-finger, at the coordinates we expect? (#7528)
//!   B. UI picking        — do `Pointer<..>` observers fire per-finger on UI nodes? (#11553 ban)
//!   C. Safe area         — does `AndroidApp::content_rect()` account for cutouts? (#23003)
//!   D. Text crispness    — what `UiScale` yields crisp pixel text at ~2.625x density?
//!   E. Save durability   — does a synchronous write inside `AppLifecycle::Suspended` complete?
//!   F. UI flicker        — does #14710 stay fixed on real hardware with wgpu 29?

use bevy::{
    input::touch::Touches,
    log::{Level, LogPlugin},
    prelude::*,
    render::view::Msaa,
    text::{FontSize, FontSmoothing},
    ui_render::UiAntiAlias,
    window::{AppLifecycle, WindowMode},
    winit::WinitSettings,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// The `bevy_main` proc macro generates the `android_main` entry point that
/// GameActivity's `System.loadLibrary` call ends up invoking. On desktop it
/// expands to nothing and `src/main.rs` calls this directly.
#[bevy_main]
pub fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                // On Android these are routed to the native logger, so they
                // surface under `adb logcat -s dot-tower RustStdoutStderr`.
                level: Level::INFO,
                filter: "wgpu=error,naga=warn,bevy_render=info".to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "dot-tower bringup".to_string(),
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
    // Lets winit idle harder when nothing is happening — meaningful for battery
    // on a device that is meant to be left running.
    .insert_resource(WinitSettings::mobile())
    .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.06)))
    .insert_resource(Probes::default())
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            probe_touch_transport,
            probe_safe_area,
            probe_save_durability,
            refresh_hud,
        ),
    );

    app.run();
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Everything the HUD reports. One resource so the readings stay together and
/// can be dumped to the log in one line when the app suspends.
#[derive(Resource)]
struct Probes {
    /// A. Live touch positions, keyed by finger id.
    touches: Vec<(u64, Vec2)>,
    /// A. High-water mark of simultaneous fingers seen this session.
    max_fingers: usize,
    /// B. Last UI button press, as `(button index, pointer id)`.
    last_press: Option<(usize, String)>,
    /// B. How many distinct pointers are holding a button down right now.
    buttons_held: usize,
    /// C. `content_rect()` insets vs the physical window, in physical pixels.
    safe_area: Option<Insets>,
    /// E. Result of the last suspend-time save attempt.
    save_report: String,
}

impl Default for Probes {
    fn default() -> Self {
        Self {
            touches: Vec::new(),
            max_fingers: 0,
            last_press: None,
            buttons_held: 0,
            safe_area: None,
            save_report: "no suspend yet".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
struct Insets {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    window_w: i32,
    window_h: i32,
}

/// The sprite dragged by probe A.
#[derive(Component)]
struct Marker;

/// The HUD text updated every frame.
#[derive(Component)]
struct Hud;

/// One of the four ability buttons (probe B).
#[derive(Component)]
struct AbilityButton(usize);

/// The frame drawn at the reported safe-area inset (probe C).
#[derive(Component)]
struct SafeAreaFrame;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands) {
    // `Msaa::Off` + `UiAntiAlias::Off` + `FontSmoothing::None` are the pixel-art
    // triple the map fixes from day one. They are also the configuration in
    // which #14710 (probe F) was reported, so this is the honest test.
    commands.spawn((Camera2d, Msaa::Off, UiAntiAlias::Off));

    // Probe A: a plain coloured sprite. No image asset, so nothing here depends
    // on APK asset loading — that is deliberately a separate concern.
    commands.spawn((
        Marker,
        Sprite::from_color(Color::srgb(0.95, 0.75, 0.2), Vec2::splat(96.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Probe C: a frame inset to the reported safe area. If `content_rect()`
    // accounts for a display cutout, this frame clears the notch. If it only
    // accounts for system bars, the frame will run underneath it — which is the
    // reading we need.
    commands.spawn((
        SafeAreaFrame,
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        BorderColor::all(Color::srgb(0.9, 0.2, 0.35)),
        BackgroundColor(Color::NONE),
    ));

    // Probe D + the HUD. Text is pinned in physical-pixel terms via `FontSize::Px`
    // so that changing `UiScale` visibly rescales it — that is the knob under test.
    commands.spawn((
        Hud,
        Text::new("starting..."),
        TextFont {
            font_size: FontSize::Px(13.0),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.9, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(48),
            left: px(16),
            right: px(16),
            ..default()
        },
    ));

    spawn_ability_bar(&mut commands);
    spawn_ui_scale_controls(&mut commands);
}

/// Probe B: four buttons along the bottom, thumb-reachable, wired through
/// `bevy_picking` observers rather than `Interaction`.
///
/// This is the multi-touch surface the whole `Interaction` ban exists to
/// protect. Pressing two or more at once must register two or more distinct
/// `PointerId::Touch(..)` values; if it does not, the ban is insufficient and
/// the input path needs rethinking before any hero abilities are designed.
fn spawn_ability_bar(commands: &mut Commands) {
    let bar = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: px(24),
            left: px(12),
            right: px(12),
            height: px(88),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .id();

    for i in 0..4 {
        commands
            .spawn((
                AbilityButton(i),
                Node {
                    width: percent(23),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(ability_idle_color()),
                ChildOf(bar),
            ))
            .with_child((
                Text::new(format!("{}", i + 1)),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::BLACK),
                // The label must not swallow the press, or the parent button
                // would never see it.
                Pickable::IGNORE,
            ))
            .observe(on_ability_press)
            .observe(on_ability_release);
    }
}

fn ability_idle_color() -> Color {
    Color::srgb(0.55, 0.58, 0.68)
}

fn on_ability_press(
    press: On<Pointer<Press>>,
    buttons: Query<&AbilityButton>,
    mut colors: Query<&mut BackgroundColor>,
    mut probes: ResMut<Probes>,
) {
    let entity = press.entity;
    let Ok(button) = buttons.get(entity) else {
        return;
    };
    if let Ok(mut color) = colors.get_mut(entity) {
        color.0 = Color::srgb(0.95, 0.85, 0.3);
    }
    probes.buttons_held = probes.buttons_held.saturating_add(1);
    probes.last_press = Some((button.0, format!("{:?}", press.pointer_id)));
    info!(
        "probe B: ability {} pressed by {:?}, {} held",
        button.0 + 1,
        press.pointer_id,
        probes.buttons_held
    );
}

fn on_ability_release(
    release: On<Pointer<Release>>,
    mut colors: Query<&mut BackgroundColor, With<AbilityButton>>,
    mut probes: ResMut<Probes>,
) {
    if let Ok(mut color) = colors.get_mut(release.entity) {
        color.0 = ability_idle_color();
    }
    probes.buttons_held = probes.buttons_held.saturating_sub(1);
}

/// Probe D: step `UiScale` so the human can find the value that renders crisp
/// pixel text at the device's density. The chain under test is
/// `font_size x scale_factor x UiScale`.
fn spawn_ui_scale_controls(commands: &mut Commands) {
    let row = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(8),
            right: px(12),
            column_gap: px(8),
            ..default()
        })
        .id();

    for (label, delta) in [("-", -0.25_f32), ("+", 0.25)] {
        commands
            .spawn((
                Node {
                    width: px(44),
                    height: px(32),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.34, 0.42)),
                ChildOf(row),
            ))
            .with_child((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ))
            .observe(move |_: On<Pointer<Press>>, mut scale: ResMut<UiScale>| {
                scale.0 = (scale.0 + delta).clamp(0.25, 4.0);
                info!("probe D: UiScale now {:.2}", scale.0);
            });
    }
}

// ---------------------------------------------------------------------------
// Probes
// ---------------------------------------------------------------------------

/// Probe A. Drives the sprite from raw touch, and records every finger.
///
/// Deliberately reads `Touches` rather than picking: winit's Android backend
/// emits `WindowEvent::Touch` and never mouse events, so this is the narrowest
/// possible test of that transport. Raw coordinates are reported to the HUD so
/// that dragging to each screen edge exercises #7528 (edge misreporting).
fn probe_touch_transport(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut marker: Single<&mut Transform, With<Marker>>,
    mut probes: ResMut<Probes>,
) {
    let (camera, camera_transform) = *camera;

    probes.touches = touches.iter().map(|t| (t.id(), t.position())).collect();
    probes.max_fingers = probes.max_fingers.max(probes.touches.len());

    // The first finger down drags the sprite; the rest are only counted. On
    // desktop, held mouse falls back in so the same build stays testable there.
    let drag_point = probes.touches.first().map(|(_, p)| *p).or_else(|| {
        if mouse.pressed(MouseButton::Left) {
            windows.iter().next().and_then(|w| w.cursor_position())
        } else {
            None
        }
    });

    if let Some(screen_point) = drag_point
        && let Ok(world) = camera.viewport_to_world_2d(camera_transform, screen_point)
    {
        marker.translation.x = world.x;
        marker.translation.y = world.y;
    }
}

/// Probe C. Reads `AndroidApp::content_rect()` and lays the frame out to match.
///
/// `content_rect` is in physical pixels and `Node` is laid out in logical ones,
/// so the insets are divided by the window's scale factor on the way in.
fn probe_safe_area(
    windows: Query<&Window>,
    mut frame: Single<&mut Node, With<SafeAreaFrame>>,
    mut probes: ResMut<Probes>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let scale = window.scale_factor().max(0.001);
    let (win_w, win_h) = (
        window.physical_width() as i32,
        window.physical_height() as i32,
    );

    let insets = read_content_rect().map(|rect| Insets {
        left: rect.0,
        top: rect.1,
        right: (win_w - rect.2).max(0),
        bottom: (win_h - rect.3).max(0),
        window_w: win_w,
        window_h: win_h,
    });

    // On desktop there is no content rect, so the frame stands in at a fixed
    // inset. That keeps the layout code on the same path in both builds.
    let shown = insets.unwrap_or(Insets {
        left: 24,
        top: 48,
        right: 24,
        bottom: 24,
        window_w: win_w,
        window_h: win_h,
    });

    frame.position_type = PositionType::Absolute;
    frame.left = px(shown.left as f32 / scale);
    frame.top = px(shown.top as f32 / scale);
    frame.right = px(shown.right as f32 / scale);
    frame.bottom = px(shown.bottom as f32 / scale);
    frame.border = UiRect::all(px(2));

    probes.safe_area = insets;
}

/// Returns the raw `content_rect` as `(left, top, right, bottom)` in physical px.
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

/// Probe E. The claim under test, from ticket 01: `AppLifecycle::Suspended` is
/// delivered synchronously while Android's main thread is blocked in
/// `surfaceDestroyed`, so a write plus `sync_all()` there will complete.
///
/// Every link in that chain is verified in source; the composition is
/// inference. This logs a timestamp either side of the write so the elapsed
/// time can be compared against the lifecycle lines in logcat.
fn probe_save_durability(
    mut lifecycle: MessageReader<AppLifecycle>,
    mut probes: ResMut<Probes>,
) {
    for event in lifecycle.read() {
        info!("probe E: AppLifecycle::{:?}", event);

        // `WillSuspend` is in the enum but ticket 01 found it is never
        // delivered. If this line ever appears in logcat, that finding is wrong
        // and ticket 12's save design has to be revisited.
        if matches!(event, AppLifecycle::WillSuspend | AppLifecycle::WillResume) {
            warn!("probe E: {:?} WAS delivered — ticket 01 finding is wrong", event);
        }

        if !matches!(event, AppLifecycle::Suspended) {
            continue;
        }

        let before = SystemTime::now();
        let payload = format!(
            "suspended_at_unix_millis = {}\nmax_fingers = {}\n",
            before
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            probes.max_fingers,
        );

        probes.save_report = match write_durably(&payload) {
            Ok(path) => {
                let elapsed = before.elapsed().map(|d| d.as_millis()).unwrap_or(0);
                let report = format!("saved in {elapsed}ms -> {path}");
                info!("probe E: {report}");
                report
            }
            Err(err) => {
                let report = format!("save FAILED: {err}");
                error!("probe E: {report}");
                report
            }
        };
    }
}

/// Writes and flushes to disk synchronously, returning the path written.
fn write_durably(payload: &str) -> std::io::Result<String> {
    use std::io::Write;

    let mut path = save_dir();
    path.push("bringup-probe.txt");

    let mut file = std::fs::File::create(&path)?;
    file.write_all(payload.as_bytes())?;
    // The whole point of the probe: without this the bytes may only be in the
    // page cache when the process is killed.
    file.sync_all()?;

    Ok(path.display().to_string())
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
    std::env::temp_dir()
}

/// Renders every reading. Probe F is the one thing not reported here: whether
/// this HUD flickers against the sprite behind it is something only eyes can
/// judge, and it is exactly the symptom of #14710.
fn refresh_hud(
    probes: Res<Probes>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window>,
    mut hud: Single<&mut Text, With<Hud>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };

    let mut out = String::new();
    out.push_str(&format!(
        "window {}x{} phys, scale_factor {:.3}\n",
        window.physical_width(),
        window.physical_height(),
        window.scale_factor(),
    ));

    // Probe D: the arithmetic the crispness question turns on.
    out.push_str(&format!(
        "[D] UiScale {:.2}  ->  13px text renders at {:.1} phys px\n",
        ui_scale.0,
        13.0 * window.scale_factor() * ui_scale.0,
    ));

    // Probe C.
    match probes.safe_area {
        Some(i) => out.push_str(&format!(
            "[C] content_rect insets L{} T{} R{} B{} of {}x{}\n",
            i.left, i.top, i.right, i.bottom, i.window_w, i.window_h
        )),
        None => out.push_str("[C] no content_rect (desktop) — frame is a stand-in\n"),
    }

    // Probe A.
    out.push_str(&format!(
        "[A] fingers now {} (max {})\n",
        probes.touches.len(),
        probes.max_fingers
    ));
    for (id, pos) in probes.touches.iter().take(5) {
        out.push_str(&format!("      #{id} at {:.0},{:.0}\n", pos.x, pos.y));
    }

    // Probe B.
    match &probes.last_press {
        Some((button, pointer)) => out.push_str(&format!(
            "[B] last ability {} via {pointer}, {} held\n",
            button + 1,
            probes.buttons_held
        )),
        None => out.push_str("[B] no ability pressed yet\n"),
    }

    // Probe E.
    out.push_str(&format!("[E] {}", probes.save_report));

    ***hud = out;
}
