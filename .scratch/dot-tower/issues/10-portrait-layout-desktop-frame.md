# Portrait layout and the desktop frame

Type: prototype
Status: open
Blocked by: 02

## Question

What is on screen, and where, on a phone and on a monitor?

Settled: a portrait-primary tower column, with desktop spending its extra width on side panels rather than restaging the scene. Not settled: what actually occupies those regions.

Sketch both layouts and answer:

1. **The tower column.** How many floors are visible at once? Does the camera follow the frontier, the hero, or the player's scroll?
2. **What must be permanently visible** — gold, current floor, lock line, hero abilities with cooldowns, prestige multiplier? On a phone this is a brutal budget.
3. **Ability triggers on touch.** Four cooldown-gated abilities need to be reachable by thumb without covering the action.
4. **Where does the upgrade tree live?** A full-screen mode on phone, a side panel on desktop? If it is modal on phone, the player cannot watch while upgrading — is that acceptable?
5. **What does desktop put in the side panels** that phone has to reach through menus, and does that make the two versions play differently?
6. **Safe areas and notches** on the portrait layout.

Link the sketches from this ticket.

## Added by ticket 02

Item 6 (safe areas) has a definite answer: **no UI option has them.** Bevy issue #23003 is `S-Blocked` on winit 0.31 (still beta), and winit's Android implementation is itself open (#4506). We must write ~40 lines against `AndroidApp::content_rect()` ourselves. Insetting edge-docked targets also mitigates the edge-coordinate misreporting in #7528, so it is worth doing for two reasons.

Layout branches on **window aspect ratio, never `target_os`** — this keeps the portrait layout testable by resizing a desktop window, which matters a great deal for iteration speed. Frame dimensions in `Percent`/`VMin`, chrome in `Px`.

## Added by ticket 03

Item 1 (camera) is now partly settled: **the camera has a hard lower bound at the lock line** — the player cannot scroll into sealed territory. The lowest visible floor needs a visual treatment indicating the tower is sealed beneath it. That indicator is the only thing conveying accumulated progress, since the sealed region is never shown, so it carries more weight than a decorative border.

Also needs a visual: climbers **sprinting** to the new entry floor when locking raises the line past them. Transient, non-combat, and it must read as a reward rather than a malfunction.
