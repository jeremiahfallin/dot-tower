# 0015. The run ends at your own record

Date: 2026-09-08
Status: accepted
Resolves: [ticket 18](../../.scratch/dot-tower/issues/18-run-is-over.md)

## Context

Ticket 08 settled the prestige screen and, in settling it, moved the risk to
its trigger: a player who does not notice their run has ended sits at the wall
indefinitely, and no legibility on a screen they never open helps them. The
assumed trigger was stall detection — peak flat for ~6 minutes — which the
model flags as ±40% noisy, and whose false negative is precisely the failure
the trigger exists to prevent.

Ticket 17 then moved the run's value event: the multiplier a run earns pays
for **new territory only** ([ADR 0014](0014-the-multiplier-pays-for-new-territory.md)),
so a run earns ×1.00 until it passes the account's best-ever floor, and
prestiging just past that point is optimal — waiting for the stall is 1.5×
slower. The moment a run becomes worth anything is exact, discrete, already in
the save, and spatial. The stall heuristic was solving the wrong problem.

## Decision

1. **The game never detects stalls.** The trigger is the account's **record** —
   its deepest-ever floor — shown as a line on the tower column, in the
   lock-line / wall-hatch family. Before the run passes it, the line shows the
   distance to beat; on a mature run it is the re-conquest target the run
   climbs toward, giving the lengthening runs of ADR 0014 their face.
2. **The prestige affordance is always present and deliberately dumb** — a
   small permanent control opening ticket 08's modal ledger. It never appears
   or disappears and never issues a verdict; the line's position carries all
   the state. Early prestige needs no discouragement machinery: the ledger
   shows ×1.00 and nothing gained — self-punishing and self-explaining.
3. **The strip gains exactly one line**: the run's running earned-state
   (new floors · ×earned; before passing the record, record vs current peak) —
   the temporal mirror of the spatial line, per ticket 09's split.
4. **The relic currency stays a ledger-only gain.** Its basis is now identical
   to the multiplier's (ADR 0014), so the incentive divergence ticket 11
   feared is structurally impossible; the trigger surfaces never mention it.

## Consequences

- Stall detection survives nowhere in the shipped design. Offline gold already
  uses best rate; run end is the player's prestige. The heuristic remains a
  reporting convenience in the model, nothing more.
- The wall hatching and aggregate deaths (ticket 09) keep their jobs as the
  stall's *texture* — visible coasting — but carry no trigger duty.
- The tower column's visual vocabulary is now closed: lock line, wall hatch,
  aura, record. Ticket 21's type legibility works within these; nothing
  further gets added to the column.
- Run length needs no clock at any depth: the marker is account-relative, so
  the same surface serves a 40-minute early run and a 3-hour mature one.
