//! The prestige ledger: the one surface allowed to be modal, because it is
//! reached a handful of times a run (ticket 10's frequency rule).
//!
//! [Ticket 08](../.scratch/dot-tower/issues/08-prestige-before-after.md) settled
//! what it shows and, more importantly, what it does not:
//!
//! - **Before and after, shown before committing, every time.** No "are you
//!   sure" — the numbers are the confirmation.
//! - **Losses itemised at their real values** (`melee rank 56 -> 1`), never
//!   softened into minutes-to-recover. [`Run::losses`] is that list and lives
//!   next to the fields it enumerates, so this file states none of them itself.
//! - **No verdicts.** The ledger never says whether now is a good time. An
//!   early prestige shows x1.00 and nothing gained — self-punishing and
//!   self-explaining ([ADR 0015](../docs/adr/0015-the-run-ends-at-your-own-record.md)).
//! - **Accumulated prestige appears only as head start, in floors**
//!   ([ADR 0007](../docs/adr/0007-prestige-is-shown-as-floors.md)). The
//!   cumulative multiplier is never displayed, here or anywhere.
//!
//! The world stops while this is open. A modal that keeps running underneath is
//! a modal the player is taxed for reading.

use bevy::prelude::*;

use crate::art::ui;
use crate::hud::label;
use crate::session::{set_deciding, Session};

#[derive(Resource, Default)]
pub struct Ledger {
    pub open: bool,
}

#[derive(Component)]
pub struct LedgerOverlay;

#[derive(Component)]
pub struct LedgerBody;

pub fn spawn_ledger(commands: &mut Commands) {
    let overlay = commands
        .spawn((
            LedgerOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                // Stops above the ability bar. Ticket 10: nothing may ever
                // occlude those four targets, because covering a cooldown-gated
                // control costs uptime and uptime is the hero's contribution.
                bottom: px(76),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(14)),
                row_gap: px(10),
                ..default()
            },
            // Opaque: at anything less the top bar's gold reads through the
            // ledger's own first line, and this is the one screen that must not
            // be ambiguous about what it is telling you.
            BackgroundColor(ui::BACKDROP),
            Visibility::Hidden,
            GlobalZIndex(10),
        ))
        .id();

    commands.spawn((
        Text::new("End this run"),
        label(16.0, ui::GOLD),
        ChildOf(overlay),
    ));
    commands.spawn((LedgerBody, Text::new(""), label(11.0, ui::INK), ChildOf(overlay)));

    let buttons = commands
        .spawn((
            Node { column_gap: px(8), margin: UiRect::top(px(6)), ..default() },
            ChildOf(overlay),
        ))
        .id();

    commands
        .spawn((
            Node { padding: UiRect::axes(px(14), px(8)), ..default() },
            BackgroundColor(ui::CHIP_LIVE),
            ChildOf(buttons),
        ))
        .with_child((Text::new("prestige"), label(13.0, ui::GOLD)))
        .observe(confirm);

    commands
        .spawn((
            Node { padding: UiRect::axes(px(14), px(8)), ..default() },
            BackgroundColor(ui::CHIP),
            ChildOf(buttons),
        ))
        .with_child((Text::new("keep climbing"), label(13.0, ui::INK)))
        .observe(cancel);
}

/// Opens the ledger and stops the world.
pub fn open_ledger(_press: On<Pointer<Press>>, mut ledger: ResMut<Ledger>, mut session: ResMut<Session>) {
    ledger.open = true;
    set_deciding(&mut session, true);
}

fn confirm(_press: On<Pointer<Press>>, mut ledger: ResMut<Ledger>, mut session: ResMut<Session>) {
    // The ledger is the confirmation (ticket 08 wants no second gate), so this
    // only guards against a press reaching a surface that is not open.
    if !ledger.open {
        return;
    }
    let applied = session.prestige();
    info!(
        "prestiged: x{:.2} earned, head start {:.0} -> {:.0} floors",
        applied.earned_multiplier, applied.head_start_before, applied.head_start_after
    );
    ledger.open = false;
    set_deciding(&mut session, false);
}

fn cancel(_press: On<Pointer<Press>>, mut ledger: ResMut<Ledger>, mut session: ResMut<Session>) {
    ledger.open = false;
    set_deciding(&mut session, false);
}

/// Shows the ledger, and fills it from the same computation the commit uses.
pub fn draw_ledger(
    ledger: Res<Ledger>,
    session: Res<Session>,
    mut overlay: Query<&mut Visibility, With<LedgerOverlay>>,
    mut body: Query<&mut Text, With<LedgerBody>>,
) {
    for mut vis in &mut overlay {
        *vis = if ledger.open { Visibility::Inherited } else { Visibility::Hidden };
    }
    if !ledger.open {
        return;
    }

    let preview = session.prestige_preview();
    let mut out = String::new();

    // The only moment with no baseline to read the numbers against.
    if preview.first_time {
        out.push_str(
            "Prestige ends this run and keeps the account. The tower drops by the head start \
             below, so the floors you fought for become the floors you start past.\n\n",
        );
    }

    let peak = session.world.peak();
    let best = session.world.account().best_peak;
    out.push_str(&format!("peak floor {peak}    record {best}\n"));
    out.push_str(&format!(
        "new territory {} floors  ->  x{:.2} earned\n",
        peak.saturating_sub(best),
        preview.earned_multiplier
    ));
    out.push_str(&format!(
        "head start {:.0} floors  ->  {:.0} floors\n\n",
        preview.head_start_before, preview.head_start_after
    ));

    out.push_str("this run ends:\n");
    for (what, change) in &preview.losses {
        out.push_str(&format!("  {what}: {change}\n"));
    }

    let mut text = match body.single_mut() {
        Ok(text) => text,
        Err(_) => return,
    };
    if text.0 != out {
        text.0 = out;
    }
}
