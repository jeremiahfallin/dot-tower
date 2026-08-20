# Ticket 07 — the offline-grant measurement

Throwaway. Not a prototype in the `prototype`-skill sense; it is the measurement behind ticket 07's
cap decision, kept only so the numbers can be re-run rather than taken on trust.

- `grant.mjs` — ticket 06's `model.mjs` with one hook added: `graceGold`, a lump of gold injected
  the first time a run hits the stall condition, after which the run continues. This simulates the
  real scenario — play, quit mid-run, return with offline gold, resume — and `peakAtGrace` records
  where the run had stalled so the floors bought can be read off directly.
- `floors.mjs` — walks a campaign for realistic prestige multipliers, then probes runs 1, 4 and 8
  with grants of 1m … 24h at that run's best rate. Produces the floors-bought table in the ticket.
- `offline.mjs` — the simpler first measurement: total run income expressed as minutes of that
  run's own best rate (3–20 min), which is what made the raw-gold comparison look alarming before
  the log conversion explained it away.

Run with `node floors.mjs` / `node offline.mjs`. Run 8's output is noise — ticket 06 flags its
stall heuristic as ±40%, and at that depth it shows.
