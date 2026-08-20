# Offline gold pays your best rate for up to 12 hours

Supersedes [0002](0002-closed-form-offline-progress.md).

While the player is away, dot-tower grants **gold only**, computed as `best rate × min(elapsed,
12h)` — where **best rate** is a high-water mark of the rolling 60-second gold/sec achieved during
the current run, reset on prestige. No efficiency fraction, no decay curve. The climb still does
not advance: floor progress requires the player to be present.

0002 had the same shape but the wrong source. It paid from **sealed** floors below the lock line,
and ticket 06 measured those at 0.04% of frontier income — a sealed band is always far behind the
frontier because gold per kill scales at 1.095/floor against enemy health at 1.075. The active
band is where the money is, and its steady state at a wall is *gold accrues, the floor does not
move*, which is exactly what an offline grant is supposed to represent.

## Considered options

**A trailing average of recent income** was rejected: quit during a re-transit, just after a
prestige, or mid-wipe at the wall, and the snapshot is a rate that isn't yours. That is the same
failure that killed 0002's measure-and-freeze rule, and no choice of window length fixes it. A
high-water mark cannot be sampled at a bad moment, so the trap is closed by construction rather
than by tuning.

**Recomputing the rate closed-form** from peak floor, ranks and the prestige multiplier was
rejected as a second implementation of the economy sitting beside the real one — the divergence
hazard [0001](0001-no-persisted-rng-state.md) and the simulation boundary were both shaped to
avoid.

**An efficiency fraction and a hand-built decay curve** were both rejected on measurement.
Injecting a grant into ticket 06's model at the moment a run stalls, away time rising 96× (5
minutes → 8 hours) buys roughly 3× the floors: rank costs compound at 1.30 while rank power
compounds at 1.15 against enemy scaling at 1.075, so **gold converts to floors logarithmically**
and each doubling of away time is worth ~2–4 floors. The diminishing returns already exist inside
the economy; a second, invisible curve on top is double-dipping.

## Consequences

- **The cap is an expectation-setting device, not a balance lever.** That is what makes 12 hours
  affordable — the difference between an 8-hour and a 12-hour cap is worth about four floors, and
  a player who sleeps nine hours and opens the app at lunch should not be punished for it.
- **The formula never names the wall.** Ticket 05 made the wall emergent rather than declared, so
  it is not a value the simulation holds. It doesn't need to be: because deeper is always richer,
  best sustained gold/sec is definitionally where the player stalled.
- **Locking loses the second job 0002 gave it.** Locking before putting the phone down no longer
  affects what you earn while away. It remains a spawn-point and travel-time mechanic, which
  ticket 06 showed is enough to earn its cost.
- **Elapsed is clamped to `0..=cap`.** Wall-clock cheating stays accepted — premium,
  single-player, no leaderboards — but a backwards clock must never produce negative gold or an
  overflow.
- **Desktop-left-running beats the cap, and that is accepted.** Desktop ticks when unfocused;
  Android destroys the surface and suspends. Capping foreground accrual would mean a visibly
  running game that has stopped paying, which is the legibility failure this project exists to
  avoid.
- **The save needs one more number**: the best rate, alongside the timestamp. Still no RNG state
  and no game clock, so [0001](0001-no-persisted-rng-state.md) stays affordable.
- **Prestige resets the best rate**, so quitting immediately after a prestige earns nothing. This
  requires a line of copy on the prestige screen; a silent zero is the failure mode this project
  is built against.
