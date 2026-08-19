# Offline progress is closed-form, gold only, and does not advance the climb

While the player is away, dot-tower grants gold computed as a closed-form function of locked floors, their guaranteed rate, and elapsed wall-clock time, capped at roughly 8-12 hours. The climb itself does not advance: floor progress requires the player to be present.

## Considered options

Simulating forward at an accelerated tick rate — the approach `guild-forge` takes for unobserved missions — was rejected. That cost is O(elapsed time × floors), which is affordable for a handful of concurrent missions and unaffordable across an overnight Android suspension against a tower of thousands of floors.

Granting offline floor progress was rejected separately: it would require simulating combat to know how far the climb got, dragging the same cost back in.

## Consequences

- This trades a genre convention — idle players expect the main number to rise while away — for comprehensibility. The return state is expressible in one sentence: *"Away 6h 12m. Floors 1-120 earned 4.2M gold."*
- Locking becomes the deliberate act a player performs before putting the phone down, which gives the mechanic a second job beyond its gold sink.
- The save needs a timestamp and a gold rate rather than an RNG state and a game clock, which is what makes ADR-0001 affordable.
- Wall-clock time is player-controllable, so the offline reward is trivially cheatable by changing the device clock. Accepted for a premium single-player game with no leaderboards.
