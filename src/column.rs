//! The tower column: the band the player actually watches.
//!
//! The camera rule, the floor height and the four marks are all settled and are
//! not re-decided here:
//!
//! - **13 floors of 48px, the same width on a phone and a monitor** — ticket 10
//!   measured 12-14 as the whole game (the interesting band is bounded by the
//!   lock interval) and [ADR 0016](../docs/adr/0016-type-reads-as-shape-hue-and-one-tag.md)
//!   fixed 48px from the smallest sprite that reads.
//! - **The camera follows the highest floor climbers have reached, hard-clamped
//!   at the lock line.** The player never scrolls by hand: there is nothing
//!   below the lock line to scroll to and nothing above the frontier to see.
//!   In a mature run the two clamps meet and the column stops moving.
//! - **Four marks and no more** — lock line, wall hatch, aura, record
//!   ([ADR 0015](../docs/adr/0015-the-run-ends-at-your-own-record.md) closed the
//!   vocabulary). Type reads on the climbers themselves, and **nothing
//!   per-climber ever renders** (ADR 0016), which is why there is no health pip
//!   in this file and must not be one.
//!
//! The one control drawn on the tower is the **lock**, because it is the single
//! purchase whose effect is spatial — it moves the line and sets climbers
//! sprinting (ticket 10, keeping variant C's one good idea).

use bevy::prelude::*;
use bevy::text::{FontSize, FontSmoothing};
use bevy::ui::widget::NodeImageMode;
use dot_tower_sim::{ClimberType, Floor};

use crate::art::{ui, Art, CLIMBER_SCALE, COLUMN_W, FLOORS_IN_VIEW, FLOOR_H, HERO_SCALE};
use crate::session::Session;

/// How high in the frame the frontier rides. Three rows of headroom above it,
/// nine of walk-up below — so the lock line stays on screen through the part of
/// a run where it is still moving.
const HEADROOM: Floor = 3;

/// The most climbers drawn at once. Ticket 20's crowd is ~57 early and a
/// handful at depth; the pool is sized well past that and the front of the
/// column is drawn first, so an overflow loses the rear rather than the fight.
const POOL: usize = 96;


/// Everything on the column that is not a floor row, tagged so one query can
/// move all of them. Bevy resolves system access statically, so a dozen
/// `Query<&mut Node, With<..>>` in one system is a runtime conflict rather than
/// a tidy decomposition; an enum is both cheaper and the honest description —
/// these are all the same kind of thing, a mark placed at a floor.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Aura,
    AuraCallout,
    WallHatch,
    RecordLine,
    RecordLabel,
    LockLine,
    LockButton,
    LockLabel,
    Climber(usize),
    Hero,
    HeroHealth,
    HeroCountdown,
}

#[derive(Component)]
pub struct ColumnBand;

#[derive(Component)]
pub struct FloorRow(u32);

#[derive(Component)]
pub struct FloorLabel(u32);

/// What the column is showing this frame.
///
/// A resource because two things need it and neither owns it: a tap on a row
/// has to turn back into a floor number, and the row does not know which floor
/// it is showing. The visible climbers are gathered here once for the same
/// reason — the drawing systems would otherwise each re-filter and re-sort the
/// whole stream, which is the same answer computed twice.
#[derive(Resource, Default)]
pub struct View {
    pub bottom: Floor,
    /// Climbers inside the window, front first, as `(type, floor, x)`. Front
    /// first so a crowd that overflows the sprite pool loses its rear rather
    /// than the fight the player is watching.
    visible: Vec<(ClimberType, Floor, f32)>,
}

fn tiny(size: f32, colour: Color) -> impl Bundle {
    (
        TextFont { font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(colour),
        // Labels must never swallow a tap meant for the floor beneath them.
        Pickable::IGNORE,
    )
}

fn absolute() -> Node {
    Node { position_type: PositionType::Absolute, ..default() }
}

/// Builds the column into `parent`. Rows first, then the marks, then the
/// units — child order is draw order, and the climbers belong on top of the
/// marks that describe where they are standing.
pub fn spawn_column(commands: &mut Commands, parent: Entity, art: &Art) {
    let band = commands
        .spawn((
            ColumnBand,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                // Never taller than the 13 floors it draws: a taller band would
                // add empty backdrop above the top row rather than more tower.
                max_height: px(FLOORS_IN_VIEW as f32 * FLOOR_H),
                flex_direction: FlexDirection::Column,
                // Rows stack from the bottom, so the band clips at the top when
                // it is shorter than 13 floors — a short window loses the empty
                // headroom rather than the fighting.
                justify_content: JustifyContent::FlexEnd,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(ui::BACKDROP),
            ChildOf(parent),
        ))
        .id();

    for i in 0..FLOORS_IN_VIEW {
        commands
            .spawn((
                FloorRow(i),
                Node {
                    width: percent(100),
                    height: px(FLOOR_H),
                    flex_shrink: 0.0,
                    border: UiRect::top(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    padding: UiRect::right(px(6)),
                    ..default()
                },
                BackgroundColor(ui::FLOOR),
                BorderColor::all(ui::FLOOR_EDGE),
                ChildOf(band),
            ))
            .with_child((FloorLabel(i), Text::new(""), tiny(10.0, ui::DIM)))
            .observe(station_on_press);
    }

    // --- the four marks ---

    commands.spawn((
        Mark::Aura,
        Node { left: px(0), right: px(0), ..absolute() },
        BackgroundColor(ui::AURA),
        Pickable::IGNORE,
        ZIndex(1),
        ChildOf(band),
    ));
    commands.spawn((
        Mark::AuraCallout,
        Text::new(""),
        tiny(10.0, ui::GOLD),
        Node { right: px(6), ..absolute() },
        ZIndex(2),
        ChildOf(band),
    ));

    commands.spawn((
        Mark::WallHatch,
        Node { left: px(0), right: px(0), height: px(FLOOR_H), ..absolute() },
        ImageNode {
            image: art.hatch.clone(),
            image_mode: NodeImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
            color: ui::WALL.with_alpha(0.0),
            ..default()
        },
        Pickable::IGNORE,
        ZIndex(2),
        ChildOf(band),
    ));

    // The record is dashed, which is what tells it from the lock line at a
    // glance: one is where the account has been, the other where the climb may
    // not go.
    let record = commands
        .spawn((
            Mark::RecordLine,
            Node {
                left: px(0),
                right: px(0),
                height: px(2),
                column_gap: px(4),
                overflow: Overflow::clip(),
                ..absolute()
            },
            Pickable::IGNORE,
            ZIndex(3),
            ChildOf(band),
        ))
        .id();
    for _ in 0..40 {
        commands.spawn((
            Node { width: px(6), height: px(2), flex_shrink: 0.0, ..default() },
            BackgroundColor(ui::GOLD),
            Pickable::IGNORE,
            ChildOf(record),
        ));
    }
    commands.spawn((
        Mark::RecordLabel,
        Text::new(""),
        tiny(10.0, ui::GOLD),
        Node { left: px(8), ..absolute() },
        ZIndex(3),
        ChildOf(band),
    ));

    commands.spawn((
        Mark::LockLine,
        Node { left: px(0), right: px(0), height: px(3), ..absolute() },
        BackgroundColor(ui::LOCK),
        Pickable::IGNORE,
        ZIndex(3),
        ChildOf(band),
    ));

    // --- the lock, drawn on the tower it moves ---
    commands
        .spawn((
            Mark::LockButton,
            Node {
                left: px(8),
                height: px(26),
                padding: UiRect::axes(px(8), px(4)),
                align_items: AlignItems::Center,
                ..absolute()
            },
            BackgroundColor(ui::CHIP),
            // Above the climbers: it is a control with a price on it, and a
            // sprite walking across the digits is the difference between a
            // price and a guess.
            ZIndex(7),
            ChildOf(band),
        ))
        .with_child((Mark::LockLabel, Text::new(""), tiny(11.0, ui::LOCK_DIM)))
        .observe(buy_lock_on_press);

    // --- the units ---
    for i in 0..POOL {
        commands.spawn((
            Mark::Climber(i),
            Node {
                width: px(crate::art::ART_W as f32 * CLIMBER_SCALE),
                height: px(crate::art::ART_H as f32 * CLIMBER_SCALE),
                ..absolute()
            },
            ImageNode::new(art.melee.clone()),
            Visibility::Hidden,
            Pickable::IGNORE,
            ZIndex(5),
            ChildOf(band),
        ));
    }

    commands.spawn((
        Mark::Hero,
        Node {
            width: px(crate::art::ART_W as f32 * HERO_SCALE),
            height: px(crate::art::ART_H as f32 * HERO_SCALE),
            ..absolute()
        },
        ImageNode::new(art.hero.clone()),
        Pickable::IGNORE,
        ZIndex(6),
        ChildOf(band),
    ));
    // Ticket 09 asks for a ring draining with the hero's health; a bar is the
    // placeholder for that shape, and the countdown replacing it while the hero
    // is down is the half carrying the uptime story.
    commands.spawn((
        Mark::HeroHealth,
        Node { height: px(3), ..absolute() },
        BackgroundColor(ui::GOLD),
        Pickable::IGNORE,
        ZIndex(6),
        ChildOf(band),
    ));
    commands.spawn((
        Mark::HeroCountdown,
        Text::new(""),
        tiny(11.0, ui::WALL),
        absolute(),
        ZIndex(6),
        ChildOf(band),
    ));
}

/// Tapping a floor stations the hero on it (ticket 09: placement is a learnable
/// rule, so the interaction can be as plain as this). The world refuses the
/// move while the hero is dead and clamps it to the lock line; neither check is
/// repeated here, because there is one authority for both.
fn station_on_press(
    press: On<Pointer<Press>>,
    rows: Query<&FloorRow>,
    view: Res<View>,
    mut session: ResMut<Session>,
) {
    let Ok(row) = rows.get(press.entity) else {
        return;
    };
    let floor = view.bottom + (FLOORS_IN_VIEW - 1 - row.0);
    session.hands.station = Some(floor);
    session.world.station_hero(floor);
}

/// Buying the lock is the one purchase with a spatial effect, so its control
/// lives on the tower. [`World::buy_lock`] refuses it when the climb has not
/// cleared the margin or the gold is not there.
fn buy_lock_on_press(_press: On<Pointer<Press>>, mut session: ResMut<Session>) {
    session.world.buy_lock();
}

/// Where the band sits this frame, and who is in it.
pub fn track_view(session: Res<Session>, mut view: ResMut<View>) {
    let lock = session.world.lock_line();
    let top = session.world.frontier() + HEADROOM;
    view.bottom = top.saturating_sub(FLOORS_IN_VIEW - 1).max(lock);

    let (bottom, ceiling) = (view.bottom, view.bottom + FLOORS_IN_VIEW);
    view.visible.clear();
    view.visible.extend(
        session
            .world
            .climbers()
            .iter()
            .filter(|c| c.pos.floor >= bottom && c.pos.floor < ceiling)
            .map(|c| (c.ty, c.pos.floor, c.pos.x.get())),
    );
    view.visible.sort_by_key(|(_, floor, _)| std::cmp::Reverse(*floor));
    view.visible.truncate(POOL);
}

/// Distance in pixels from the band's bottom edge to the floor line of `floor`.
fn y_of(view: &View, floor: Floor) -> f32 {
    (floor as f32 - view.bottom as f32) * FLOOR_H
}

pub fn draw_rows(
    view: Res<View>,
    mut rows: Query<(&FloorRow, &mut BackgroundColor)>,
    mut labels: Query<(&FloorLabel, &mut Text)>,
) {
    for (row, mut colour) in &mut rows {
        let floor = view.bottom + (FLOORS_IN_VIEW - 1 - row.0);
        colour.0 = if floor.is_multiple_of(5) { ui::FLOOR_FIFTH } else { ui::FLOOR };
    }
    for (label, mut text) in &mut labels {
        let floor = view.bottom + (FLOORS_IN_VIEW - 1 - label.0);
        // Floor 0 is real to the simulation — `lock_line()` is 0 until the
        // first lock is bought — but `CONTEXT.md` numbers floors from 1, so it
        // is drawn and never named. See the note in `track_view`'s ticket.
        let want = if floor > 0 && floor.is_multiple_of(5) {
            floor.to_string()
        } else {
            String::new()
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}

/// Places every mark. One system and one query, because they all answer the
/// same question — at what height does this belong — against one view.
pub fn draw_marks(
    session: Res<Session>,
    view: Res<View>,
    mut marks: Query<(&Mark, &mut Node, &mut Visibility)>,
) {
    let world = &session.world;
    let top = view.bottom + FLOORS_IN_VIEW;
    let radius = session.tuning.hero_aura_floors;
    let lit_aura = session.tuning.hero_aura_mult > 1.0;
    let hero_floor = world.hero_floor();
    let hero_w = crate::art::ART_W as f32 * HERO_SCALE;
    let hero_x = COLUMN_W - hero_w - 10.0;
    let hero_on_screen = hero_floor >= view.bottom && hero_floor < top;
    let sprite_w = crate::art::ART_W as f32 * CLIMBER_SCALE;
    let best = world.account().best_peak;

    for (mark, mut node, mut vis) in &mut marks {
        let mut show = true;
        match *mark {
            // The lit floors the hero multiplies — drawn only when it actually
            // multiplies. The shipped curve has `hero_aura_mult` at 1.0, so the
            // aura is inert until a relic turns it on (ticket 16), and lighting
            // a row for an inert aura would promise a multiplier that is not
            // there. The band and its callout appear together or not at all.
            Mark::Aura => {
                show = lit_aura && world.hero_alive() && hero_on_screen;
                node.height = px((2 * radius + 1) as f32 * FLOOR_H);
                node.bottom = px(y_of(&view, hero_floor.saturating_sub(radius)));
            }
            Mark::AuraCallout => {
                show = lit_aura && world.hero_alive() && hero_on_screen;
                node.bottom = px(y_of(&view, hero_floor) + FLOOR_H - 14.0);
            }
            // The wall carries no trigger duty — ADR 0015 retired stall
            // detection from the design. It is the stall's *texture*, so it
            // fades in with how long the climb has been stuck and decides
            // nothing.
            Mark::WallHatch => {
                node.bottom = px(y_of(&view, world.frontier()));
            }
            // The line the run climbs toward, and the moment prestige starts
            // being worth anything (ADR 0014).
            Mark::RecordLine | Mark::RecordLabel => {
                show = best > 0 && best >= view.bottom && best < top;
                let lift = if *mark == Mark::RecordLabel { 3.0 } else { 0.0 };
                node.bottom = px(y_of(&view, best) + FLOOR_H + lift);
            }
            Mark::LockLine => {
                show = world.lock_line() > 0;
                node.bottom = px(y_of(&view, world.lock_line()));
            }
            Mark::LockButton => {
                node.bottom = px(y_of(&view, world.lock_line()) + 6.0);
            }
            Mark::LockLabel => {}
            Mark::Climber(i) => match view.visible.get(i) {
                Some(&(ty, floor, x)) => {
                    node.left = px(x * (COLUMN_W - sprite_w - 72.0) + lane(ty));
                    node.bottom = px(y_of(&view, floor) + 2.0);
                }
                None => show = false,
            },
            Mark::Hero => {
                show = hero_on_screen && world.hero_alive();
                node.left = px(hero_x);
                node.bottom = px(y_of(&view, hero_floor) + 2.0);
            }
            Mark::HeroHealth => {
                let frac = world.hero_health_fraction() as f32;
                show = hero_on_screen && world.hero_alive() && frac > 0.0;
                node.left = px(hero_x);
                node.width = px(hero_w * frac);
                node.bottom = px(y_of(&view, hero_floor) + FLOOR_H - 4.0);
            }
            Mark::HeroCountdown => {
                show = hero_on_screen && !world.hero_alive();
                node.left = px(hero_x);
                node.bottom = px(y_of(&view, hero_floor) + 14.0);
            }
        }
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
}

/// The type each climber slot is currently showing, and the wall's opacity.
pub fn draw_unit_images(
    session: Res<Session>,
    view: Res<View>,
    art: Res<Art>,
    mut images: Query<(&Mark, &mut ImageNode)>,
) {
    for (mark, mut image) in &mut images {
        match *mark {
            Mark::Climber(i) => {
                if let Some(&(ty, _, _)) = view.visible.get(i) {
                    let want = art.of(ty);
                    if image.image != want {
                        image.image = want;
                    }
                }
            }
            Mark::WallHatch => {
                let stuck = (session.seconds_stuck() / 20.0).clamp(0.0, 1.0) as f32;
                image.color = ui::WALL.with_alpha(0.35 * stuck);
            }
            _ => {}
        }
    }
}

/// The text on the column: floor callouts, the record label, the lock's price
/// and the hero's respawn countdown.
pub fn draw_mark_text(
    session: Res<Session>,
    mut texts: Query<(&Mark, &mut Text, &mut TextColor)>,
) {
    let world = &session.world;
    let (next_floor, price) = world.next_lock();
    let deep_enough = world.peak() >= next_floor + session.tuning.lock_margin;
    let affordable = price <= world.gold();

    for (mark, mut text, mut colour) in &mut texts {
        let want = match *mark {
            Mark::AuraCallout => format!("x{:.0}", session.tuning.hero_aura_mult),
            Mark::RecordLabel => format!("record {}", world.account().best_peak),
            Mark::LockLabel => {
                // Free at or below the account's deepest-ever floor, because
                // head start is conquered territory (ADR 0013) — and saying
                // "free" is how the player finds that out.
                colour.0 = if deep_enough && affordable { ui::GOLD } else { ui::LOCK_DIM };
                if !deep_enough {
                    let needed = (next_floor + session.tuning.lock_margin).saturating_sub(world.peak());
                    format!("lock {next_floor} - {needed} floors off")
                } else if price <= 0.0 {
                    format!("lock {next_floor} - free")
                } else {
                    format!("lock {next_floor} - {}", crate::hud::compact(price))
                }
            }
            Mark::HeroCountdown => match world.hero_respawn_in() {
                Some(secs) => format!("{:.0}s", secs.ceil()),
                None => String::new(),
            },
            _ => continue,
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}

/// The lock chip's own background, which tracks affordability.
pub fn draw_lock_chip(session: Res<Session>, mut chips: Query<(&Mark, &mut BackgroundColor)>) {
    let world = &session.world;
    let (floor, price) = world.next_lock();
    let ready = world.peak() >= floor + session.tuning.lock_margin && price <= world.gold();
    for (mark, mut colour) in &mut chips {
        if *mark == Mark::LockButton {
            colour.0 = if ready { ui::CHIP_LIVE } else { ui::CHIP };
        }
    }
}

/// Keeps the three types from stacking into one file of sprites. Purely
/// presentational: combat is floor-local and the simulation has no notion of
/// where across a floor anything stands beyond `NormX` (ADR 0010).
fn lane(ty: ClimberType) -> f32 {
    match ty {
        ClimberType::Melee => 6.0,
        ClimberType::Ranged => 30.0,
        ClimberType::Healer => 54.0,
    }
}
