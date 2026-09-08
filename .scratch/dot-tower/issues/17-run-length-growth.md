# Does run length grow as the account matures?

Type: prototype
Status: open
Blocked by: 06

## Question

The map has been assuming that runs get longer as peak floor rises — a new account finishes a run
in one sitting, a mature account spreads one across many. Ticket 07 leaned on that assumption when
settling the session shape, and it is currently supported by nothing.

Ticket 06's constants do not produce it. Run lengths across a ten-run campaign go **41, 21, 44,
102, 72, 89, 62, 53, 33, 37** minutes — oscillating around ~50 with no trend. Ticket 06 itself says
run length is really *"when the player stops wanting to push"* and that its own stall heuristic is
±40% noise, so this is the model declining to answer rather than contradicting the claim.

1. **Do we want it?** A run that lengthens as the account matures gives late-game play a different
   rhythm from early-game and makes prestige a considered act rather than a reflex. A flat ~50
   minutes forever is simpler and is what the constants currently do. Decide before tuning.
2. **If yes, what produces it?** Candidates: the prestige multiplier growing slower than enemy
   scaling; lock costs rising faster than income; the replacement interval (ticket 06 found this is
   the run-length dial, not the respawn timer); or something structural that only appears deeper.
3. **How would we know?** The stall heuristic is too noisy to measure a trend this size. Either
   run length needs a less noisy definition, or it needs measuring some other way — this is the
   same instrument problem sitting in the map's *balance methodology* fog.
4. **What does it do to offline gold?** Ticket 07's contract assumed away-time is the dominant
   share of a mature run's wall-clock. That held up under the log conversion regardless, so this
   should not reopen ticket 07 — but confirm it doesn't.

## Comments

### Sharpened by ticket 20 (2026-09-08)

The lock curve *was* the run-length curve and no longer is. Under the shipped
geometric curve, runs ended because locks became unaffordable; priced in time
([ADR 0013](../../../docs/adr/0013-locks-are-priced-in-time.md)), ticket 20's
ten-run campaign gave 40/35/39 min for runs 1–3, 125/146/169 for runs 4–6, and
**runs 7–10 still climbing at a 180-minute cap** (M 4.6×10¹³, peak 1849 — note
the model's f64 horizon is ~floor 2000, so *when* a mature run ends is literally
past what this model can measure). For this ticket that means:

- **Q1 is answered — yes, we want it — and it must now be authored.** Nothing in
  the current design ends a mature run. Ticket 08 already prices prestiging at
  the stall 270× over one more floor, so the pressure exists; either the player
  learns to prestige *while still climbing*, or the design brings stalls back.
  If replacement (ticket 06's run-length dial) is the lever, note the collision:
  raising it thins the crowd, and it is also ADR 0011's crowd dial — one number,
  two jobs.
- **Q2's lock-cost candidate is dead by construction** (time pricing cannot
  outrun income). The remaining candidates are the prestige multiplier's growth
  versus enemy scaling, replacement, or something structural deeper in.
- **Q3 is now the whole ticket.** The stall heuristic is ±40%, the honest answer
  lives past the model's numeric horizon, and ticket 19 already established that
  pinning wall-clock is the only comparison this model can be trusted for. A
  log-spaced or f64-widened model, or resolution by argument from the head-start
  curve rather than simulation, are both on the table.
- **Q4 got sharper, not easier.** Ticket 07 assumed away-time dominates a mature
  run's wall-clock. If a mature run never stalls — climbs for hours — that
  assumption's shape changes under whatever ending rule this ticket lands; the
  log conversion still protects the formula, but the session-shape story needs a
  re-read once an ending rule exists.
