# Save schema and migration

Type: grilling
Status: open
Blocked by: 05, 07

## Question

What exactly is written to disk, and what happens when that shape changes?

Settled: hand-rolled `serde` to RON, one autosave slot plus a rotating backup, no persisted RNG state, no seeded determinism. Not settled:

1. **The full field list.** Timestamp, gold, peak floor, lock line, per-type climber ranks, hero roster with levels, prestige multiplier, relics held. Anything else? Ticket 05 may add persistent climber state; ticket 07 fixes what the offline calculation needs.
2. **Versioning.** A schema version field costs nothing now and is impossible to retrofit. What is the migration policy when a field changes — migrate, or reset with an apology?
3. **Autosave triggers.** On lock, on prestige, on a timer, on Android suspend? Ticket 01 establishes whether suspend gives reliable warning.
4. **Corruption handling.** When the primary save fails to parse, what happens — silent fallback to backup, or tell the player? Silent fallback can quietly discard hours of progress.
5. **What is deliberately *not* saved**, and can it be recomputed on load without the player noticing a discontinuity?
6. **Cheating posture.** A plain-text RON save is trivially editable. For a premium single-player game that may be entirely fine — decide it explicitly.

## Added by ticket 07

**The offline fields are fixed.** Ticket 07 settled offline gold as `best rate × min(elapsed,
12h)`, which costs the save exactly two things:

- a **timestamp** at write, and
- the **best rate** — one `f64`, the high-water mark of rolling 60-second gold/sec for the current
  run, reset on prestige.

No game clock and no RNG state, so `docs/adr/0001` stays affordable.

**What is deliberately not saved** (item 5) also has an answer for the active band: in-flight
climbers, contested floors and floor respawn timers are all dropped. On load the stream respawns at
the lock line and re-walks. The discontinuity is real but unnameable — ticket 05 established that
individual climbers are never tracked — and ticket 07 treats the resulting re-transit as part of
what offline gold pays for.

**Item 6, cheating posture, is partly pre-answered.** Ticket 07 accepted wall-clock manipulation
explicitly (premium, single-player, no leaderboards), hardened only by clamping elapsed to
`0..=cap` so a backwards clock cannot produce negative gold or an overflow. A plain-text RON save
is trivially editable and the same posture presumably applies — but that is this ticket's call, not
ticket 07's.
