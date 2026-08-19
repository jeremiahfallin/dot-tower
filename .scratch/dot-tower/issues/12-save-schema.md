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
