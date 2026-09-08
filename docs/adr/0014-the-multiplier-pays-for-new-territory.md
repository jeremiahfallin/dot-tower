# 0014. A run's multiplier pays for new territory only

Date: 2026-09-08
Status: accepted
Resolves: [ticket 17](../../.scratch/dot-tower/issues/17-run-length-growth.md)

## Context

Ticket 20 priced locks in time and runs stopped ending: nothing in the design
ended a mature run, which made run length — the map's oldest unmeasured
assumption — an open question with no owner.

Measured first ([`prototypes/17-run-length/chop.mjs`](../../.scratch/dot-tower/prototypes/17-run-length/chop.mjs)):
under the shipped rule, the multiplier a run earns is a function of its peak
floor, `(1 + peak/50)^1.2` — and **rapid prestige cycling dominates
everything**. From a mature state, prestiging every 5 minutes for 12 hours
grows the account to M = 4.2×10¹¹⁰, compounding ln M at 21.2/hour against 2.6
for the stall policy. The runs never re-reach the account's best (they are
walk-limited to ~170 floors); they climb easy conquered floors and bank a
multiplier for territory the account had already conquered. The head start —
bought with one prestige — is resold every cycle. This is prestige-spam:
ticket 11 feared it, and priced the relic currency on *peak beyond your
previous best* specifically to kill it — but the multiplier itself paid for
re-conquest, and the multiplier is raw power.

## Decision

The multiplier a run earns is a function of that run's peak **beyond the
account's deepest-ever floor**: `M_earned = (1 + Δ/50)^1.2`, with
Δ = max(0, run peak − best-ever). A run that never passes the account's best
earns ×1.00. A fresh account is identical to the old rule (best-ever 0, Δ =
peak), so every early-game number from tickets 06 and 08 stands unchanged.
The principle is now uniform with ticket 11's relic currency: **the account
pays for new territory only.** The save already holds deepest-floor-ever in
`Account`.

## Consequences

- **Spam dies by arithmetic, not by friction.** Prestige intervals that never
  reach the frontier earn literally nothing — measured: M frozen at the warm
  value across 12 hours of 5-minute cycles.
- **Run length becomes structural, and grows with the account** — the map's
  original assumption, now produced rather than assumed. Early runs are pure
  frontier (~40 min). A mature run must first re-conquer its head start — a
  walk that grows with best-ever — so the optimal interval drifts upward
  (measured: 60–70 min at best-ever 412; 70–80 min at 771) and natural runs
  land at 2.5–3 hours. The lengthening *is* the re-conquest walk.
- **The stall over-waits.** Prestiging just past the account's best grows the
  account ~1.5× faster than waiting for the stall (ln M/h 1.26 vs 0.82 at
  six-run maturity). Prestige stops being a stall decision (ticket 08's
  framing) and becomes a **past-your-best decision** — which is a legible
  moment, and ticket 18's surface must mark it.
- **The campaign slows in floor terms.** Floor 1,000 arrives at run 16 (run 7
  under the re-conquest-paying rule; ticket 06 promised ~10), with new
  territory holding steady at 48–76 floors per run through run 20 — no
  dead-end, unlike ticket 06's `bestPeak` mode, because increments compound.
  The divisor and exponent are tuning data; the basis is the decision.
- **Head start settles at ~21% of peak at maturity** (24–28% under the old
  basis); ticket 08's ~26% holds for the early account.
- **Offline gold needs no reopening — it is confirmed by this rule.** Away
  time cannot earn multiplier at all (new territory requires play), so ticket
  07's gold-only, climb-frozen contract is not merely convenient; it is the
  only shape consistent with the economy.
