# How the game says a run is over

Type: grilling
Status: open
Blocked by: 17

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
