# How the game says a run is over

Type: grilling
Status: resolved
Blocked by: 17

*Claimed 2026-09-08; grilling round presented to the user the same day — ticket 17's ADR 0014
moved the value event from the stall to the past-your-best moment, which retires question 1's
heuristic entirely and reshapes the rest.*

## Question

What tells the player their run has ended, and where does the prestige affordance come from?

Ticket 08 settled the prestige screen and, in settling it, moved the hard part somewhere else.
Measured at run 1's stall: **one more floor costs 1.4 minutes and buys +0.084 floors of head
start, against 22.8 for prestiging now.** At run 5: 3.3 minutes and +0.025, against 42.9. The
margin is three orders of magnitude and it never narrows.

So there is no "is my multiplier big enough yet" decision to make. There is only *have I stopped
climbing* — which means the prestige screen's content was never the risk. **The trigger is.** A
player who does not notice they have stalled sits at the wall indefinitely, and no amount of
legibility on a screen they never open will help them.

Not settled:

1. **What is the stall condition in the shipped game?** Ticket 06's model uses "peak flat for
   ~6 minutes", and tickets 07 and 17 both flag that heuristic as too noisy to lean on — ±40% at
   depth. A false positive tells a climbing player to quit; a false negative is the failure this
   ticket exists to prevent. Is peak-flat even the right signal, or is it something the player can
   see directly — deaths at the wall, gold-per-second flattening, best rate stopping moving?

2. **Is the prestige affordance always present, or does it appear?** A button that is always there
   invites the player to prestige far too early, which ticket 08 measured as strictly bad. A button
   that appears is the game issuing a verdict, which ticket 08 deliberately declined to do on the
   screen itself. Does that reasoning survive being moved to the trigger, or does it invert?

3. **Where does it live?** Ticket 09 gave the strip above the ability bar to the temporal — a 45s
   holding sparkline and the record of ability presses. A stall is temporal. Does it belong in that
   strip, and can the strip carry it without becoming two things? Ticket 10 owns the region.

4. **What does the stall look like before it is named?** Under the legibility constraint the player
   should be able to see the run ending before the game says so. Ticket 09 already hatches the wall
   and shows deaths in aggregate there. Is that enough, and if it is, what is left for the trigger
   to add beyond a route to the screen?

5. **Does this change at depth?** Ticket 17 is measuring whether runs get longer as the account
   matures. If a mature run spans days and many pickups, a six-minute flat peak is a normal
   Tuesday, not a stall.

## Added by ticket 11

Prestige now grants a **currency spent on relic ranks**, which puts a second gain on the prestige
side of the trigger this ticket owns.

It was scaled deliberately to avoid breaking that trigger: the currency pays on **peak floor beyond
your previous best**, so only new depth earns it. Flat-per-prestige was rejected because it makes
**prestige-spam optimal** — ticket 08 measured re-climbing at 3× speed, so a loop of shallow runs
would out-earn one deep run and directly fight the stall logic ticket 08 established.

Two things for this ticket to carry:

1. **The choice was structural, not measured.** Ticket 07 hit the limits of the stall heuristic for
   detecting run-length *trends* (ticket 17's problem), so "new depth only" was chosen because it
   needs no measurement. If ticket 17 produces a tool that *can* measure trends, this is worth
   re-checking rather than assuming.
2. **The prestige ledger now has a third gain line** — earned multiplier, head start in floors, and
   relic currency. Ticket 08 judged line count safe because each has its own units, but this ticket
   owns whether the trigger stays legible with three things on it.

## Comments

### Premise settled by ticket 17 (2026-09-08)

Ticket 17 decided *when* a run is over in value terms: the multiplier pays
for new territory only ([ADR 0014](../../../docs/adr/0014-the-multiplier-pays-for-new-territory.md)),
so a run is worth something from the moment it passes the account's
best-ever floor and the optimal prestige lands shortly after — 1.5× faster
than waiting for the stall. The measured facts this ticket's surface must
carry: the **past-your-best moment** (concrete, spatial, on the tower), the
×1.00 state before it (a ledger that says "this run has earned nothing yet"),
and run lengths that lengthen with maturity (~40 min early, 2.5–3 h deep) so
"the run is over" cannot lean on a fixed session length.

## Answer

Grilled 2026-09-08 — all five questions settled as recommended. Decision recorded in
[ADR 0015](../../../docs/adr/0015-the-run-ends-at-your-own-record.md); **Record** added to
`CONTEXT.md`.

1. **The trigger is passing the record; stall detection retires from the design entirely.**
   ADR 0014 made the run's value event exact, discrete, and saved: ×1.00 until the peak passes
   the account's best-ever floor. No heuristic anywhere in the shipped game — offline gold
   already uses best rate, and run end is the player's prestige. A false negative is now
   impossible, which was this ticket's founding failure mode. The wall hatching and aggregate
   deaths keep their jobs as the stall's *texture* (visible coasting), carrying no trigger
   duty. Works at every depth, because the marker is account-relative, not clock-relative —
   question 5's depth worry dissolves with the clock.
2. **The record is a line on the tower column** — the lock-line / wall-hatch family. Before
   the run passes it, the line shows the distance to beat; on a mature run it is the
   re-conquest target the run climbs toward, giving ticket 17's lengthening runs their face.
3. **The prestige affordance is always present and deliberately dumb** — a small permanent
   control opening ticket 08's modal ledger. It never appears or disappears and never issues a
   verdict; ticket 08's "no verdicts" reasoning holds at the trigger because the line's
   position carries all the state. Early prestige needs no discouragement: the ledger shows
   ×1.00, nothing gained — self-punishing and self-explaining, and the one-time explainer
   covers the only moment with no baseline.
4. **The strip gains exactly one line** — the run's running earned-state (new floors · ×earned;
   before passing the record, record vs current peak). The temporal mirror of the spatial
   line, per ticket 09's split; noted as an amendment on ticket 09.
5. **Ticket 11's currency: no re-check needed, and its gain stays ledger-only.** The currency's
   basis (peak beyond previous best) is now *identical* to the multiplier's — ADR 0014 unified
   them — so the incentive divergence ticket 11 feared is structurally impossible; the
   measurement tool 11 wanted arrived and answered for free. The third gain line stays on the
   ledger (each gain has its own units); the trigger surfaces — the record line and the strip
   line — never mention currency, so the moment stays singular.

One closure with teeth: the tower column's visual vocabulary is now **closed** — lock line,
wall hatch, aura, record. Ticket 21's type legibility must work within these four; nothing
further is added to the column.
