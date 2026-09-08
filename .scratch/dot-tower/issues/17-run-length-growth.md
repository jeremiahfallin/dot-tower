# Does run length grow as the account matures?

Type: prototype
Status: resolved
Blocked by: 06

*Claimed 2026-09-08, immediately after ticket 20 resolved — unblocked by it. The question is no
longer "does run length grow" but "what ends a mature run at all", since the lock curve no
longer does.*

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

## Answer

Decision recorded in [ADR 0014](../../../docs/adr/0014-the-multiplier-pays-for-new-territory.md);
evidence in [`prototypes/17-run-length/`](../prototypes/17-run-length/). The instrument problem
(question 3) dissolved rather than got solved: instead of measuring run length through the ±40%
stall heuristic, the sweep pins **total wall-clock** and varies the prestige interval — ticket
19's trusted comparison — so the model never has to detect a stall to answer.

### What the sweep found first: the shipped rule is a spam engine

`chop.mjs`: from a mature state, 12 h under eleven prestige policies. Under the shipped rule
(earned multiplier = f(run's peak), `(1 + peak/50)^1.2`), **prestiging every 5 minutes
dominates everything**: M = 4.2×10¹¹⁰, ln M compounding at 21.2/hour against 2.6 for the
stall policy. The 5-minute runs never re-reach the account's best (walk-limited to ~170
floors); they climb easy conquered floors and bank a multiplier for territory the account
already owned. Head start bought with one prestige, resold every cycle. Ticket 11 feared
exactly this and priced the relic currency on peak-beyond-best to kill it — but the
multiplier itself paid for re-conquest, and the multiplier is raw power.

### The decision: pay for new territory only

`prestigeBasis: 'beyondBest'` — earned = `(1 + Δ/50)^1.2`, Δ = max(0, peak − best-ever). A run
that never passes the account's best earns ×1.00. A fresh account is unchanged (Δ = peak), so
tickets 06 and 08's early-game numbers stand. The principle is now uniform with ticket 11's
relic currency: **the account pays for new territory only.**

### The four questions

1. **Do we want lengthening runs? Yes — and now it is produced, not assumed.** Under
   `beyondBest`, early runs are pure frontier (40/35/45 min); a mature run must first
   re-conquer its head start, a walk that grows with best-ever, and natural runs land at
   2.5–3 h (136–202 min, runs 9–20 of the 20-run campaign). The *optimal* interval drifts
   upward too: 60–70 min at best-ever 412, 70–80 min at 771 (`maturity.mjs`). The map's
   assumption was right; it needed a mechanism, and the re-conquest walk is it.
2. **What produces it: the earned-multiplier basis**, not lock costs (dead by construction,
   ticket 20) and not replacement (which would thin the crowd — ticket 06's run-length dial
   collides with ADR 0011's crowd dial; leave it alone). The structural mechanism is the one
   question 2 speculated ("something that only appears deeper"): conquered territory only
   exists on a mature account, and only a mature run has to re-walk it.
3. **How we know: pinned-wall-clock policy sweeps.** The stall heuristic is retired from
   anything load-bearing. Natural-stall lengths remain reportable texture at ±40%.
4. **Offline gold: confirmed, not reopened.** Under this rule away time cannot earn
   multiplier *at all* — new territory requires play — so ticket 07's gold-only, climb-frozen
   contract is the only shape consistent with the economy, not merely a convenient one.

### The costs, on the table

- Floor 1,000 arrives at run 16 (run 7 under the old basis; ticket 06 promised ~10). New
  territory holds steady at 48–76 floors/run through run 20 with no dead-end (unlike ticket
  06's `bestPeak` mode, which dead-ended at ~220 — increments compound here), so this is
  pacing, not a wall; the divisor/exponent can pull it back, and they are tuning data.
- Head start settles at ~21% of peak at maturity (24–28% under the old basis); ~26% still
  holds for the early account.
- Prestige timing flips from ticket 08's "stall decision" to a **past-your-best decision**:
  prestiging before passing the account's best earns nothing, just past it is optimal, and
  waiting for the stall is ~1.5× slower (ln M/h 1.26 vs 0.82). That moment — passing your
  best — is concrete, spatial, and ticket 18's surface must mark it. Noted on that ticket.

Model: `prestigeBasis` flag-gated in `06-progression-curve/model.mjs`, default `peak`
byte-identical (verified against `reach.mjs` and `composition.mjs` before and after).
