# The save is authoritative and never re-derived

The save file is the record of what the player **has**, not a set of inputs from which their
holdings are recomputed. No code path may retroactively reduce it: not a formula retune, not a
tuning-data rename, not a schema change. Where a value could be either stored or derived, it is
stored; where a value is no longer understood, it is kept anyway.

This is one decision, and it surfaced three times while settling the save schema (ticket 12) —
each time looking like a separate, local call, and each time the redundant-looking option was the
right one. It pairs with [0007](0007-prestige-is-shown-as-floors.md): 0007 governs how permanent
progress is *shown*, this governs how it is *kept*.

## Consequences

- **The cumulative prestige multiplier is stored, not derived from a history of past peak
  floors.** Deriving it makes the earned-multiplier formula load-bearing forever: retune it in a
  later patch and every existing player's permanent power silently moves, for reasons that
  happened months ago and with no event to attribute it to. A peak-floor history may still be
  recorded for the run-comparison surface still sitting in the map's fog — but as a *record*. Where
  the two disagree, the stored multiplier wins.

- **Relic ids are stable strings, and the save is never pruned to match the catalogue.**
  [0009](0009-relic-effects-avoid-axes-the-curve-erases.md) makes relics open RON data against a
  closed effect enum, so the catalogue is *expected* to churn while the vocabulary does not. A save
  will therefore eventually name a relic the running build has never heard of. It is retained
  verbatim and ignored at runtime, never dropped — dropping is irreversible, and re-adding or
  un-renaming the relic restores it. A rename mistake becomes recoverable instead of a permanent
  silent theft of something ticket 11 declared unique and permanent.

- **Post-release, schema changes migrate; they never reset.** A chain of `v_n → v_n+1` functions,
  each one permanent once shipped. Before the first public build the policy inverts — bump and
  reset, no migration code — because there is no player to apologise to and a migration chain
  written against a schema still in motion is waste. The inversion happens once, on release day.

- **A save from a newer version disarms every autosave trigger.** Refusing to load it is the
  obvious half of the answer and the harmless half. The dangerous half is what happens next: a
  fresh game plus a 60-second autosave timer overwrites the player's real save within a minute.
  Nothing else in the design prevents this, so the write-lock is the decision and the refusal is
  merely its precondition.

- **The backup is copied byte-for-byte, never re-serialised.** Re-serialising would re-stamp the
  write timestamp — erasing the player's offline entitlement on the one path where they are already
  having a bad day — and would drop exactly the unknown relic ids this ADR exists to preserve. A
  save the build cannot fully understand still has to survive being handled.

- **This costs storage and it costs migration code, and both are the point.** A future contributor
  will notice that the multiplier is recomputable and the dead relic ids are inert, and will
  propose deleting them. Neither is redundancy; both are the refusal to let the present rewrite the
  past.
