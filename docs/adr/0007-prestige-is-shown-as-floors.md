# Prestige is shown as floors, not as a multiplier

The prestige multiplier compounds — [0006](0006-offline-gold-pays-the-best-rate.md)'s campaign
model measures it reaching **×10¹¹ by floor 1,000** and continuing — and a number of that shape is
exactly the illegibility this project exists to avoid. So the **cumulative** multiplier is never
displayed anywhere in the game. It is shown only as **head start**: `ln(M) / ln(1.075)`, the
number of floors by which the whole tower has dropped.

This is an exact restatement rather than a friendly approximation. Enemy health and damage both
scale on 1.075 per floor while the multiplier scales climbers, so with head start `h`, floor `f`
fights precisely as floor `f − h` did. "Worth about 350 floors" is true in the same sense that
×10¹¹ is true, and the player can act on it.

The **earned** multiplier — what a single prestige is worth on its own, `(1 + peak/50)^1.2` — is a
different number and *is* shown, on the prestige screen, beside the head start. It is a log-ish
function of peak floor: ×5.18 at floor 147, ×22.24 at floor 613, and still only ×38.6 at floor
1,000. It stays two or three digits for the lifetime of an account, so it costs nothing in
legibility and gives the player a figure they can feel this run earning.

## Consequences

- **Head start is measured to settle at ~26% of peak floor** (15%, 24%, 27%, 27%, 27%, 26% across
  runs 1–6), so the relationship stays stable as the account matures. The framing does not need
  re-explaining later, and there is no depth at which it stops meaning what it meant.
- The save schema still stores the cumulative multiplier — head start is derived at display time
  and never persisted. Ticket 12 owns that, but it is not a stored field.
- **Head start is a presentation of the multiplier, not a second mechanic.** Nothing is computed
  from it. Any future code that treats it as an input rather than an output is a bug.
- 1.075 is enemy scaling from [0006](0006-offline-gold-pays-the-best-rate.md)'s curve. If that
  constant is retuned, the head-start conversion moves with it — they are the same number and must
  read from the same RON key, never be written twice.
- This is the decision most likely to be undone by a contributor who wants to show the player a
  big number. The big number is ×10¹¹.
