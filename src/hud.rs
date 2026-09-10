//! Everything on screen that is not the tower: the top bar, the spend block,
//! the strip and the ability bar.
//!
//! The frame is ticket 10's variant B, and its two rules are worth stating
//! because they are easy to erode:
//!
//! - **Spending is never modal** ([ADR 0008](../docs/adr/0008-spending-is-never-modal.md)).
//!   Ranks are bought every few seconds, so their prices are permanently on
//!   screen; a surface may be modal only in proportion to how rarely it is
//!   used, which is why prestige gets an overlay and ranks do not.
//! - **The frame is the same on a phone and a monitor.** Extra width holds
//!   nothing a phone cannot reach; a desktop-only readout is a bug. So the
//!   whole frame is capped at the column's width and centred, and layout
//!   branches on aspect ratio rather than on `target_os` — which keeps the
//!   portrait layout testable by resizing a desktop window.
//!
//! And the split ticket 09 settled, which decides what may go where: **the
//! tower says where and now; the strip says how it has been going.** Nothing
//! appears in both.

use bevy::prelude::*;
use bevy::text::{FontSize, FontSmoothing};
use dot_tower_sim::{curves, ClimberType};

use crate::art::{ui, Art, COLUMN_W};
use crate::session::{Session, HOLDING_WINDOW};

/// One updating piece of text. An enum rather than a marker component each,
/// because Bevy resolves system access statically and a dozen
/// `Query<&mut Text, With<..>>` in one system is a conflict, not a decomposition.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Readout {
    Gold,
    Rate,
    Peak,
    Rank(ClimberType),
    Holding,
    Earned,
    /// Ticket 07's return banner and ticket 12's load notice share this line:
    /// both are transient, both are temporal, and only one can be true at once.
    Verdict,
}

#[derive(Component, Clone, Copy)]
pub struct RankChip(pub ClimberType);

#[derive(Component)]
pub struct SparkBar(pub usize);

/// Four cooldown-gated ability buttons, reserved and inert.
///
/// Ticket 10 fixed their size and position and gave them one absolute rule —
/// **nothing may ever occlude them**, because covering a cooldown-gated control
/// costs uptime and uptime is the hero's whole contribution. What they *do* is
/// not specified anywhere: the hero roster past the slice is still in the map's
/// fog, and the simulation has no ability API. So the bar is built to the
/// settled layout and does nothing, rather than inventing four abilities here.
#[derive(Component)]
pub struct AbilityButton(pub usize);

const SPARK_BARS: usize = 90;

/// Every string on screen is ASCII, deliberately: Bevy's built-in font carries
/// a small glyph set and renders an em dash or a middle dot as a missing-glyph
/// box. That is a placeholder constraint — the pixel font this game wants does
/// not exist yet — but a readout that shows a tofu box is worse than one that
/// spells its separator with a hyphen.
pub fn label(size: f32, colour: Color) -> impl Bundle {
    (
        TextFont { font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(colour),
        Pickable::IGNORE,
    )
}

/// Builds the whole screen and returns nothing: every system below finds its
/// own pieces by component.
pub fn spawn_screen(commands: &mut Commands, art: &Art) {
    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(ui::BACKDROP),
            Pickable::IGNORE,
        ))
        .id();

    let frame = commands
        .spawn((
            crate::Frame,
            Node {
                width: percent(100),
                max_width: px(COLUMN_W),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(root),
        ))
        .id();

    spawn_top_bar(commands, frame);
    // Slack goes above the tower, never below the ability bar: ticket 10 puts
    // the four targets at the bottom edge, above the gesture inset, and a gap
    // under them would move them out of the thumb's reach on a tall screen.
    commands.spawn((
        Node { width: percent(100), flex_grow: 1.0, ..default() },
        Pickable::IGNORE,
        ChildOf(frame),
    ));
    crate::column::spawn_column(commands, frame, art);
    spawn_spend_block(commands, frame);
    spawn_strip(commands, frame);
    spawn_ability_bar(commands, frame);
}

/// Gold, rate, the deepest floor this run, and the prestige affordance —
/// which is "always present and deliberately dumb" (ADR 0015): it never appears
/// or disappears and never issues a verdict, because the record line on the
/// column already carries all the state.
///
/// Deliberately absent: any prestige *number*. ADR 0007 keeps the cumulative
/// multiplier off the screen entirely and the earned one belongs to the ledger.
fn spawn_top_bar(commands: &mut Commands, frame: Entity) {
    let bar = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(40),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(10),
                padding: UiRect::horizontal(px(10)),
                ..default()
            },
            BackgroundColor(ui::PANEL),
            ChildOf(frame),
        ))
        .id();

    commands.spawn((Readout::Gold, Text::new("0"), label(15.0, ui::GOLD), ChildOf(bar)));
    commands.spawn((Readout::Rate, Text::new(""), label(11.0, ui::DIM), ChildOf(bar)));
    commands.spawn((
        Node { flex_grow: 1.0, ..default() },
        Pickable::IGNORE,
        ChildOf(bar),
    ));
    commands.spawn((Readout::Peak, Text::new(""), label(11.0, ui::INK), ChildOf(bar)));
    commands
        .spawn((
            Node {
                padding: UiRect::axes(px(8), px(4)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ui::CHIP),
            ChildOf(bar),
        ))
        .with_child((Text::new("prestige"), label(11.0, ui::INK)))
        .observe(crate::prestige::open_ledger);
}

/// The three ranks, always on screen with their live prices. The hue on each
/// chip is the type's own (ADR 0016), so the price is tied to the silhouette it
/// pays for rather than to a word.
///
/// There is no hero-level button, and that is deliberate: `hero_cost` is the
/// throwaway model's gold-bought hero, and `CONTEXT.md` reserves **level** for
/// what the hero earns with experience. Putting a price on it here would ship
/// the wrong economy.
fn spawn_spend_block(commands: &mut Commands, frame: Entity) {
    let block = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(56),
                flex_shrink: 0.0,
                column_gap: px(6),
                padding: UiRect::all(px(6)),
                ..default()
            },
            BackgroundColor(ui::PANEL),
            ChildOf(frame),
        ))
        .id();

    for ty in ClimberType::ALL {
        commands
            .spawn((
                RankChip(ty),
                Node {
                    flex_grow: 1.0,
                    flex_basis: px(0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    padding: UiRect::axes(px(8), px(4)),
                    border: UiRect::left(px(4)),
                    ..default()
                },
                BackgroundColor(ui::CHIP),
                BorderColor::all(ui::of_type(ty)),
                ChildOf(block),
            ))
            .with_children(|chip| {
                chip.spawn((Text::new(ty.name()), label(11.0, ui::INK)));
                chip.spawn((Readout::Rank(ty), Text::new(""), label(11.0, ui::DIM)));
            })
            .observe(move |_: On<Pointer<Press>>, mut session: ResMut<Session>| {
                // Straight through to the world rather than through
                // `Player::spend`: that is called once per simulated second,
                // and a tap must change the price on the frame it happened.
                session.world.buy_rank(ty);
            });
    }
}

/// Everything temporal, and nothing else (ticket 09).
///
/// Three lines: the 45-second holding sparkline, the run's earned-state, and
/// one line for whatever the game most recently has to say. The ability-press
/// timeline and its plain-language verdict are the fourth and fifth, and they
/// wait on abilities existing at all.
fn spawn_strip(commands: &mut Commands, frame: Entity) {
    let strip = commands
        .spawn((
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(ui::PANEL),
            ChildOf(frame),
        ))
        .id();

    commands.spawn((Readout::Holding, Text::new(""), label(10.0, ui::DIM), ChildOf(strip)));

    let spark = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(22),
                align_items: AlignItems::FlexEnd,
                column_gap: px(1),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(strip),
        ))
        .id();
    for i in 0..SPARK_BARS {
        commands.spawn((
            SparkBar(i),
            Node { flex_grow: 1.0, flex_basis: px(0), height: percent(20), ..default() },
            BackgroundColor(ui::CHIP),
            Pickable::IGNORE,
            ChildOf(spark),
        ));
    }

    commands.spawn((Readout::Earned, Text::new(""), label(11.0, ui::GOLD), ChildOf(strip)));
    commands.spawn((Readout::Verdict, Text::new(""), label(10.0, ui::INK), ChildOf(strip)));
}

/// Four thumb-reachable targets across the bottom. Nothing may overlap them.
fn spawn_ability_bar(commands: &mut Commands, frame: Entity) {
    let bar = commands
        .spawn((
            crate::AbilityBar,
            Node {
                width: percent(100),
                height: px(76),
                flex_shrink: 0.0,
                column_gap: px(6),
                padding: UiRect::all(px(6)),
                ..default()
            },
            BackgroundColor(ui::PANEL),
            ChildOf(frame),
        ))
        .id();

    for i in 0..4 {
        commands
            .spawn((
                AbilityButton(i),
                Node {
                    flex_grow: 1.0,
                    flex_basis: px(0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(ui::CHIP),
                ChildOf(bar),
            ))
            .with_child((Text::new(format!("{}", i + 1)), label(16.0, ui::DIM)));
    }
}

// --------------------------------------------------------------------------
// Updates
// --------------------------------------------------------------------------

pub fn draw_readouts(session: Res<Session>, mut readouts: Query<(&Readout, &mut Text)>) {
    let world = &session.world;
    for (readout, mut text) in &mut readouts {
        let want = match *readout {
            Readout::Gold => compact(world.gold()),
            Readout::Rate => format!("{}/s", compact(world.current_rate())),
            Readout::Peak => format!("floor {}", world.peak()),
            Readout::Rank(ty) => {
                let cost = curves::rank_cost(&session.tuning, ty, world.ranks()[ty]);
                format!("rank {} - {}", world.ranks()[ty], compact(cost))
            }
            Readout::Holding => format!(
                "HOLDING - LAST {}s        {:.0}%",
                HOLDING_WINDOW as u32,
                session.holding() * 100.0
            ),
            Readout::Earned => earned_state(&session),
            Readout::Verdict => match (&session.banner, &session.notice) {
                (Some((line, _)), _) => line.clone(),
                (None, Some(notice)) => notice.clone(),
                (None, None) if session.writes_refused => {
                    "Progress is not being saved - see the log.".to_string()
                }
                _ => String::new(),
            },
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}

/// Ticket 18's one line: the temporal mirror of the record line on the column.
/// Before the run passes the record it counts down to it; after, it states what
/// the run has earned. It never issues a verdict about whether to prestige.
fn earned_state(session: &Session) -> String {
    let save = session.snapshot();
    let best = save.account.best_peak;
    let peak = save.run.peak;
    if peak <= best {
        format!("record {best} - peak {peak} - {} to beat", best.saturating_sub(peak) + 1)
    } else {
        format!(
            "+{} new floors - x{:.2} earned",
            peak - best,
            save.earned_multiplier(&session.tuning)
        )
    }
}

pub fn draw_rank_chips(session: Res<Session>, mut chips: Query<(&RankChip, &mut BackgroundColor)>) {
    for (chip, mut colour) in &mut chips {
        let cost = curves::rank_cost(&session.tuning, chip.0, session.world.ranks()[chip.0]);
        colour.0 = if cost <= session.world.gold() { ui::CHIP_LIVE } else { ui::CHIP };
    }
}

/// The holding sparkline: hero uptime over the last 45 seconds, oldest at the
/// left. It answers a different question from the health bar on the tower —
/// that one says *this second*, this one says *all minute* — which is exactly
/// why both exist and neither is a duplicate of the other.
pub fn draw_sparkline(session: Res<Session>, mut bars: Query<(&SparkBar, &mut Node, &mut BackgroundColor)>) {
    let samples = session.holding_samples();
    let per_bar = (samples.len() as f32 / SPARK_BARS as f32).max(1.0);
    for (bar, mut node, mut colour) in &mut bars {
        let from = (bar.0 as f32 * per_bar) as usize;
        let to = ((bar.0 + 1) as f32 * per_bar).ceil() as usize;
        let slice: Vec<bool> = samples.iter().copied().skip(from).take(to.saturating_sub(from)).collect();
        let up = if slice.is_empty() {
            None
        } else {
            Some(slice.iter().filter(|u| **u).count() as f32 / slice.len() as f32)
        };
        match up {
            Some(frac) => {
                node.height = percent(15.0 + 85.0 * frac);
                colour.0 = if frac > 0.6 { ui::GOLD } else { ui::WALL };
            }
            None => {
                node.height = percent(15);
                colour.0 = ui::CHIP;
            }
        }
    }
}

// --------------------------------------------------------------------------
// Numbers
// --------------------------------------------------------------------------

/// Gold reaches 10^31 in a mature account, so nothing may print in full.
pub fn compact(value: f64) -> String {
    const SUFFIX: [&str; 11] =
        ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No"];
    if !value.is_finite() {
        return "-".to_string();
    }
    let v = value.abs();
    if v < 1000.0 {
        return format!("{}{:.0}", if value < 0.0 { "-" } else { "" }, v);
    }
    let tier = ((v.log10() / 3.0).floor() as usize).min(SUFFIX.len() - 1);
    let scaled = v / 1000f64.powi(tier as i32);
    let sign = if value < 0.0 { "-" } else { "" };
    if tier == SUFFIX.len() - 1 && v >= 1e33 {
        return format!("{sign}{v:.2e}");
    }
    format!("{sign}{scaled:.2}{}", SUFFIX[tier])
}

#[cfg(test)]
mod tests {
    use super::compact;

    #[test]
    fn compact_stays_short_at_every_scale() {
        assert_eq!(compact(0.0), "0");
        assert_eq!(compact(999.4), "999");
        assert_eq!(compact(1_500.0), "1.50K");
        assert_eq!(compact(4_200_000.0), "4.20M");
        // The economy holds to ~floor 2,000; nothing may ever print in full.
        assert!(compact(1e31).len() <= 9, "{}", compact(1e31));
        assert!(compact(1e40).len() <= 10, "{}", compact(1e40));
    }
}
