# Spending is never modal, and desktop adds width rather than information

The portrait frame cannot hold the tower and a spend surface at full size, so something has to
give. Ticket 10's mock measured what each option costs: a modal deck keeps the **tower column** at
17 floors but covers the wall row — the exact row the player wants to watch when a rank purchase
lands — and leaves only the empty floors above the frontier visible. A permanent spend block keeps
every price on screen at the cost of five floors. We took the permanent block.

The rule generalises past this one screen: **modality is priced by frequency.** Climber ranks and
locks are bought every few seconds, so they are always on screen. Relics and prestige are reached
a handful of times a run, so a full-screen surface for them costs nothing. A surface may be modal
in proportion to how rarely it is used.

The same decision fixes what desktop is allowed to be. The tower column, the **strip** and the
ability bar are the same size in both frames; the extra desktop width goes to one panel holding
exactly what the phone reaches through its rare modal surfaces. **Nothing is ever visible on
desktop that a phone player cannot reach.** A desktop-only readout is a bug, not a feature.

## Consequences

- **The tower column is sized to the band, not to the frame.** Twelve to fourteen floors, because
  the interesting band is bounded by the lock interval — locking every 10 floors means the lock
  line never trails the wall by more than about ten floors, and the mock shows the rest of the
  column empty in the steady state. A taller column shows more nothing.
- **The camera is stationary in the common case.** It follows the frontier with a hard lower clamp
  at the lock line, and once the lock line is within a screen of the frontier the two clamps meet
  and the view pins. Scrolling is an early-run and post-prestige phenomenon, not the normal one.
- **The ability bar is the one region nothing may ever occlude.** Abilities are cooldown-gated, so
  covering them costs uptime — the throttle [0005](0005-the-hero-multiplies-climbers-add.md) makes
  the hero's whole contribution.
- **Relics are modal, reached at prestige** (settled by ticket 11). Relic currency is granted at
  prestige, so relic spending is a burst once per run, next to a surface already modal and already
  rare. The caveat is load-bearing: if relic currency ever becomes earnable *during* a run, relics
  become a moment-to-moment spend and this rule sends them back onto the permanent block.
- Layout still branches on window aspect ratio and never on `target_os`, so both frames stay
  testable by resizing a desktop window.
