# Ticket 08 — the prestige before/after screen

Throwaway, per `prototype` (UI branch, sub-shape B — a Bevy game has no existing route to host
variants). Four structurally different answers to *what does the player see when they prestige*.

- `prestige.html` — the prototype. One self-contained file; open it directly, or serve the
  `prototypes/` directory and hit `/08-prestige-screen/prestige.html`. Variants at
  `?variant=A|B|C|D`, switchable from the floating bar or the ← → keys.
- `dump.mjs` — regenerates the run table baked into `prestige.html`. It calls ticket 06's
  `model.mjs` unmodified, as a seven-run campaign with the ticket-16 hero (×2 aura, taunting,
  stationed at the wall), and emits peak / minutes / gold / ranks / multipliers / the peak-vs-time
  curve for each run. Run with `node dump.mjs`.

Every number on the screen is measured, not invented — including the projected next run, which is
the *actual* next run of the same campaign. That matters here more than in most prototypes: the
whole question is whether the screen can make a true statement the player can act on, and a screen
built on made-up numbers cannot be judged for honesty.

Runs 1, 2 and 5 are offered as account states (first prestige, second, mature). Run 7's stall
detection is noise — ticket 06 flags the heuristic at ±40% and at that depth it shows — so the
mature case stops at prestiging out of run 5 into run 6.
