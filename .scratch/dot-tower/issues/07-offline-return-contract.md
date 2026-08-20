# Offline gold formula and the return-screen contract

Type: grilling
Status: resolved
Blocked by: 06

## Question

What exactly does the player earn while away, and what does the game say when they come back?

Settled: gold only, closed-form, capped at roughly 8-12 hours, no floor progress. Not settled:

1. **The formula.** Gold as a function of which locked floors, their rate, and elapsed wall-clock time. Does every locked floor contribute, or only the highest band?
2. **The cap.** Where exactly, and is it a hard stop or diminishing returns past the threshold? A hard stop is more legible; diminishing returns is kinder to someone who sleeps nine hours.
3. **Clock trust.** Wall-clock time on a device the player controls invites trivial cheating by changing the system clock. Do we care? For a premium single-player game with no leaderboards the honest answer may be "no" — but decide it deliberately rather than by omission.
4. **The return screen.** Under the legibility constraint this must be answerable in one sentence: *"Away 6h 12m. Floors 1-120 earned 4.2M gold."* What exactly does it show, and does it appear as a modal, a banner, or a ledger entry?
5. **Zero-progress honesty.** If the player was away 20 minutes and earned almost nothing, does the screen still appear? Suppressing it hides the mechanic; always showing it becomes noise.
6. **First-session behaviour.** Before anything is locked, offline earns nothing. Does the game explain why, or silently show zero? Silently showing zero is exactly the Thousand Floors failure.

## Added by ticket 06

**Offline gold has no source.** This ticket's premise was that offline progress yields gold
only, closed-form, capped at 8–12 hours. Ticket 06 removed the thing that was going to generate
it: sealed floors produce no meaningful income (0.04% of frontier earnings), and the climb does
not advance while away.

So there is nothing left running. This ticket needs a new premise before it can be worked —
either offline gold comes from somewhere other than sealed floors, or offline return is not
about gold at all.

## Answer

### The premise, restored

The ticket said offline gold had no source. The source had **moved**, not vanished. Ticket 06
killed *sealed* income, but ticket 03's floor lifecycle still stands: cleared floors repopulate on
a 20s timer, failed floors reset immediately, and replacement climbers spawn every 5s per type
under cap. So a player who quits at the wall leaves behind a system whose true steady state is
**gold accrues, the floor does not move** — which is precisely what ADR 0002 wanted to grant.
Offline gold is now a faithful summary of what the active band would really do, rather than a
payout from a region that had stopped earning.

### The contract

**Offline gold = best rate × min(elapsed, 12h).** No efficiency factor, no decay curve.

- **Best rate** is a high-water mark of the rolling 60-second gold/sec achieved **this run**,
  reset on prestige. One `f64` in the save.
- **12 hours, hard stop.** An overnight sleep and a slow morning are fully covered.
- Elapsed is clamped to `0..=cap`, so a backwards clock grants zero rather than something
  nonsensical.
- Credited automatically on load. The return screen reports; it never gates.

### Why a high-water mark rather than a trailing window

A trailing average is ticket 06's bug wearing a new hat — quit during a re-transit, just after a
prestige, or mid-wipe at the wall, and you snapshot a rate that isn't yours. A high-water mark
sidesteps the trap *structurally* rather than by tuning a window: there is no bad moment to sample
because a maximum cannot be sampled at a bad moment.

It also removes the need for the game to know where **the wall** is. Ticket 05 deliberately made
the wall emergent rather than declared, so it is not a value the simulation holds and the formula
cannot reference it. It doesn't need to: gold per kill scales at 1.095/floor against enemy health
at 1.075, so deeper is always richer and **your best sustained gold/sec is definitionally where
you stalled.** The wall gets into the formula without ever being named.

The rejected third option was recomputing the rate closed-form from peak floor, ranks and the
prestige multiplier. Stable, but it is a second implementation of the economy sitting beside the
real one — the "two paths computing the same thing" hazard ticket 03 eliminated by construction.

### The measurement that decided the cap

Scripts kept at `.scratch/dot-tower/prototypes/07-offline-grant/` — `grant.mjs` is ticket 06's
model with a resume-grant hook, `floors.mjs` produces the table below. Throwaway, but re-runnable.

Expressed as raw gold, an 8-hour cap looked catastrophic: a whole run's *total* income is worth
only **3–20 minutes at that run's own peak rate**, because income compounds so hard that
everything before the last stretch is a rounding error. Paying eight hours of frontier rate looked
like handing over dozens of runs.

That comparison is wrong, because gold does not convert to progress linearly. The model was
patched to inject a grant at the moment a run stalls and measure floors gained:

```
run 1 (stalls @132)   5m:+10   30m:+10   2h:+25   8h:+24   24h:+37
run 4 (stalls @348)   5m:+ 5   30m:+15   2h:+18   8h:+29   24h:+37
```

Away time rises **96×** from 5 minutes to 8 hours and buys roughly **3× the floors**. Rank costs
compound at 1.30 while rank power compounds at 1.15 against enemy scaling at 1.075, so
**offline gold converts to floors logarithmically** and each doubling of away-time is worth
~2–4 floors. (Run 8 was discarded — ticket 06 flags its stall heuristic as ±40% noise, and that
is what its numbers look like.)

Three consequences follow, and they are the substance of this ticket:

1. **The efficiency factor is deleted, not tuned.** *"You earn what you were earning, for up to
   12 hours"* is one sentence with no unexplained number in it. A 40% version buys a handful of
   floors in exchange for a percentage the player will read as a punishment.
2. **No hand-built decay curve.** The diminishing returns already exist inside the economy.
   Layering a second, invisible curve on a log curve is double-dipping, and gives the player two
   mechanisms to fail to understand instead of zero.
3. **The cap is an expectation-setting device, not a balance lever.** Which is what makes the
   generous end of ADR 0002's 8–12h range affordable.

### The return screen

Ticket 09 settled that **the tower says where and now; the strip says how it has been going.** A
return report is a temporal statement, so it belongs to the strip above the ability bar, not over
the tower.

- **A transient banner in the strip, not a modal.** A run now spans many sittings, which makes
  this the most-repeated screen in the game; a dismissal gate on it is a tax.
- **The comparator is ranks, not gold and not floors.** "4.2M gold" is meaningless when the rate
  is 10³¹/s. "Worth 8 hours of climbing" is circular — the grant *is* rate × time. "Floors' worth"
  needs the log conversion, which means reintroducing the second economy model rejected above.
  Ranks are exact, free (the game holds every rank price), and point at the decision the player is
  about to make.

  > **Away 8h 04m — the tower kept fighting. 4.2M gold, enough for 6 ranks.**

- **Threshold on duration, never on amount.** Under ~5 minutes away, credit silently. Suppressing
  on *amount* is the Thousand Floors failure directly: it hides the mechanic exactly when the
  player is least sure it exists. Suppressing on *duration* hides nothing, because a player who
  alt-tabbed for ninety seconds knows they alt-tabbed.

### The session shape this assumes

A run is **spread across pickups**, not one sitting — on Android the realistic unit is 5–10
minutes. So putting the phone down mid-run is the common case, and the run survives it **in full**:
gold, per-type ranks, hero level and experience, lock line, locks, relics, prestige multiplier and
peak floor all restore exactly. Quitting is never a soft prestige.

The **active band does not persist**. In-flight climbers, contested floors and respawn timers are
dropped; on load the stream respawns at the lock line and re-walks. Losing ~90 anonymous climbers
costs the player nothing they can name (ticket 05: individuals are never tracked) and keeps ticket
12's save small. The cost is that the first minute after a resume is re-transit rather than
earning — which is part of what offline gold is paying for.

### Accepted asymmetries

- **Desktop-left-running beats the cap.** Desktop keeps ticking when unfocused; Android destroys
  the surface and suspends (ticket 01), so nothing accrues. A desktop player who leaves the game
  open overnight earns real, uncapped, fully-simulated gold. Accepted rather than policed: capping
  foreground accrual would mean a visibly running game that has stopped paying, which is exactly
  the *what did that just do for me* failure this project exists to avoid. Same posture as the
  clock-cheat below.
- **Offline pays the *stalled* rate, not a climbing one.** Parity is with a desktop player who has
  hit a wall and is earning consistently — not one who is still ascending. Offline cannot reach
  new floors, so it cannot earn at a growing rate.
- **Wall-clock cheating remains trivially possible and remains accepted** — premium, single-player,
  no leaderboards. Hardened only by the clamp, which turns "player set their clock back" from a
  possible negative-gold or overflow path into a no-op. A monotonic cross-check was rejected:
  Android kills the process and uptime resets on reboot, so it catches only the cheater who cannot
  be bothered to restart.

### The post-prestige hole

Best rate is per-run, so it resets on prestige. Prestige, close the app, sleep, and you return to
**zero**. Accepted, with **one line of copy on the prestige screen** — *the tower earns nothing
while you rebuild*. It is self-correcting (ticket 06 has run 2 re-reaching the previous ceiling in
10–16 minutes), and both alternatives — carrying the high-water across prestige, or seeding the
new run from a decayed previous one — add a second formula to move a number across a boundary the
entire design says resets. A silent zero would be the failure; an expected one is fine.

The original ticket's first-session worry mostly evaporates: any player who has killed anything
has a best rate, so there is no "nothing is locked yet, so you earn nothing" state to explain.

### Vocabulary

Three names were in play — *offline progress* (ADR 0002), *offline gold* (this ticket), *offline
return* (the map's fog). Settled on **offline gold**, with **best rate** for the high-water mark.
Both added to `CONTEXT.md`. *Best rate* rather than *holding rate*: ticket 09 already uses
**holding** for hero uptime, and rather than *peak rate*, because *peak* is already the floor.

### What this changes elsewhere

- **ADR 0002 is superseded** by [ADR 0006](../../../docs/adr/0006-offline-gold-pays-the-best-rate.md),
  not amended. Its mechanism is wrong twice over: sealed floors don't pay, and its second stated
  benefit — *"locking becomes the deliberate act a player performs before putting the phone down"*
  — is dead, because the offline rate is a frontier high-water mark and locking before you quit
  now does nothing for what you earn while away.
- **Ticket 12** — the offline fields are fixed. Addendum filed there; it is now unblocked.
- **New ticket 17** — the map assumed runs lengthen as the account matures. Ticket 06's constants
  don't produce that. Raised as its own question.

Status: resolved
