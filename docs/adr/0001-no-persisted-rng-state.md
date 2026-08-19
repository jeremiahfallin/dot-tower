# No persisted RNG state and no seeded determinism

Combat randomness in dot-tower (crit rolls, spawn jitter) is cosmetic: no outcome is ever reconstructed from a seed, because offline progress is computed in closed form rather than replayed (see ADR-0002). We therefore do not seed the RNG, do not persist RNG state in the save, and do not take a determinism dependency such as `bevy_turborand`.

This is recorded because it is a deliberate divergence from the sibling project `guild-forge`, whose GDD persists RNG state and game clock so that missions can be replayed deterministically. A reader familiar with that project will assume the same pattern applies here and may try to "restore" it.

## Consequences

- Combat cannot be replayed or reproduced from a save. Bug reports about combat will need logs or video, not a seed.
- If a future feature ever requires reconstructing past simulation — a replay viewer, a verified leaderboard, server-side validation — this decision must be revisited *before* that feature, not during it.
