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
