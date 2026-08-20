# Portrait layout and the desktop frame

Type: prototype
Status: resolved
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

## Prototype

Built 2026-08-20. Throwaway, per `prototype` (UI branch, sub-shape B — a Bevy game has no existing
route to host variants).

- **Flip through it**: https://claude.ai/code/artifact/e76a6248-777f-41e9-86ee-f070973318e9
  Variants at `?variant=A|B|C`, switchable from the floating bar or the ← → keys; the frame toggle
  swaps a 390×790 phone for a 1120×660 monitor. Press `b` to fire ticket 07's return banner.
- **Source**: `.scratch/dot-tower/prototypes/10-portrait-layout/layout.html` — one self-contained
  file, open it directly.
- **Branch**: `prototype/portrait-layout`.

Three structurally different answers to the same question — **where does gold get spent** — running
live, because the budget only bites once things are moving. The tower, ticket 09's strip and the
ability bar are identical across all three; they are settled and are not the subject. A stub sim
drives the motion; it is not ticket 06's model, and no number in it is balanced.

- **A — The deck.** Tower keeps the whole frame (17 rows). Spending slides up over the bottom 54%,
  tabbed *Ranks / Relics / Lock*. Desktop turns the deck into a permanently-open 340px right panel.
- **B — The split.** Three fixed bands, nothing ever modal: tower viewport (12 rows), a permanent
  spend block carrying all three ranks and the lock, then the strip and the ability bar. Desktop
  moves the block to a right panel and gives the height back (15 rows).
- **C — The diegetic rail.** No panel: a 54px rail of type chips down the left edge of the tower,
  the lock drawn as a button on the tower itself, relics as a rail chip. 18 rows. Desktop unfolds
  the rail into a left panel and grows the strip into a scrolling run ledger on the right.

## Answer

**Variant B's structure, with C's diegetic lock, sized to the band rather than to the frame.**
Recorded as [ADR 0008](../../../docs/adr/0008-spending-is-never-modal.md).

### The principle: modality is priced by frequency

The three variants disagree about whether spending may cover the tower, and the mock settles it by
making the cost visible. Opening A's deck covers the wall row — and because the camera sits the
wall low in the frame, what is left visible is *only* the empty floors above the frontier. The
player buys a rank and watches nothing. That is the one purchase whose effect they would want to
see land, so the surface that hides it is the wrong surface.

But the objection is about **frequency**, not about modality. Ranks and locks are bought every few
seconds, so they are always on screen. Relics and prestige are reached a handful of times a run, so
a full-screen surface costs them nothing. A surface may be modal in proportion to how rarely it is
used — which is also the answer to item 4, and a rule the rest of the game's screens inherit.

### Measured: the tower does not want the height A and C give it

B's stated risk was that 12 rows is too few. The mock says the opposite, and this is the finding
that decides the ticket. **In the steady state the tower is almost entirely empty.** Climbers pile
at the wall — ticket 16's crowd-as-armour — so one row carries the fighting, a few carry the walk
up from the lock line, and everything else is unfought floors. The interesting band is bounded by
the lock interval: locking every 10 floors means the lock line never trails the wall by much more
than ten, so **12–14 rows is the whole game**, and A's 17 and C's 18 buy more nothing.

That inverts the trade. B is not paying five floors for always-on prices; it is spending five
floors that were not carrying anything.

### Desktop: width, never information

Each variant's desktop frame was built honestly, and only B's survives. A's panel grows relics, run
stats and best rate; C's grows a scrolling run ledger. Both mean a desktop player reads state a
phone player must go and fetch — and C's ledger is specifically the second temporal surface ticket
09 rejected for the slice, smuggled back in through the wide frame.

So: **the tower column, the strip and the ability bar are the same size in both frames** — the mock
pins the column at 430px on desktop and centres it — and the extra width holds exactly what the
phone reaches through its rare modal surfaces. Nothing is visible on desktop that a phone player
cannot reach in one tap. A desktop-only readout is a bug. This also answers item 5's real question:
the two versions must not play differently, so the panel is a convenience, never a capability.

### The rejected rail

C is the most attractive of the three and the mock is what talks you out of it. The rail overlaps
the tower it annotates, its chips are the smallest touch targets on screen, and it needs a
**left-edge** safe-area inset on top of the top and bottom ones — #7528's edge-coordinate
misreporting applies to every docked edge, and B's layout only ever docks two. Its one idea worth
keeping is the **lock**, which is the single purchase whose effect is spatial: it moves the lock
line and sets climbers sprinting. That button belongs on the tower, next to the sealed marker,
not in a list of prices.

### The original items

1. **The tower column.** 12–14 floors, on both frames. The camera follows the highest floor
   climbers have reached, hard-clamped at the lock line — and the mock surfaced that in a mature
   run **the two clamps meet and the camera stops moving entirely**. Scrolling is an early-run and
   post-prestige phenomenon, not the normal one. The player never scrolls by hand; there is nothing
   below the lock line to scroll to and nothing above the frontier to see.
2. **Permanently visible.** Gold and rate; the highest floor reached; the tower column with the
   combined wall-and-aura row; the three climber types with live rank and price; the next lock and
   its price; the strip; four abilities. Deliberately **not** present: any prestige number. ADR 0007
   keeps the cumulative multiplier off the screen entirely, and the earned one belongs to the
   prestige screen — which is worth about 40px of a budget that had none to spare.
3. **Ability triggers.** Four targets at roughly 88×72 across the bottom, above the gesture inset.
   **Nothing may ever occlude them** — the deck was rebuilt mid-prototype to stop above the bar.
   Covering a cooldown-gated control costs uptime, and uptime is the hero's whole contribution.
4. **The upgrade tree.** There is no tree. Ranks and the lock are permanent and on screen; relics
   and prestige get modal surfaces because they are rare. Watching while upgrading is preserved for
   exactly the purchases whose effect you would watch.
5. **Desktop side panels.** One panel, holding what the phone reaches modally. No divergence.
6. **Safe areas.** Top inset for the notch, bottom for the gesture bar, and rejecting the rail
   means those are the only two edges that dock anything — the simplest possible case for the ~40
   lines of hand-rolled `AndroidApp::content_rect()` work ticket 02 established we have to write.

### Surfaced

- **Relics have nowhere to live and no known frequency.** They are the first thing this ticket tells
  "you are rare enough to be modal", and that only holds if they are actually rare. Added to
  ticket 11, which owns it.
- **The glossary had no word for the screen.** Ticket 09 settled the strip and ticket 10 settles the
  column, and neither was named. Both added to `CONTEXT.md`.
- **`frontier` is used across tickets 03, 06 and 10 but is not a glossary term**, and it is not a
  synonym for the wall — they coincide only once the climb has stalled. Left undefined rather than
  invented here; noted in the map's fog.
