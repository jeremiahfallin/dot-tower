# The simulation boundary

Type: grilling
Status: resolved

## Question

Exactly where does simulated combat end and computed math begin, and what contract must hold across that seam?

This is the most expensive decision on the map to reverse. The hybrid model is settled in principle: real simulation where the player is looking, closed-form math everywhere else. What is not settled:

1. **What is the simulated region?** The frontier floor only? A window of N floors around the camera? Everything above the lock line?
2. **What does a locked floor do?** Locking sets the spawn point and kills rendering below it. Does a locked floor still simulate anything at all, or does it collapse entirely to a gold rate?
3. **What happens between the lock line and the frontier** — floors that are unlocked but not being watched?
4. **The agreement contract.** The simulated and computed paths must produce consistent results, or the player sees the game cheat. What exactly must agree — gold per second, kill rate, time-to-clear-a-floor? What tolerance is acceptable?
5. **Transitions.** What happens when the player scrolls the camera to a floor that was being computed, or when a floor gets locked while simulating?
6. **The failure mode to design against.** "Units go outside the walls" in Thousand Floors is a simulation bug. What invariants does the simulated region need so this class of bug is structurally impossible rather than fixed case by case?

Resolution should name the boundary precisely enough that ticket 12 can define a save schema from it.

## Answer

**The partition: two regions, not three.**

- **Sealed** — everything below the lock line. Zero entities, zero simulation, zero rendering. Contributes exactly one number: a gold rate.
- **Active** — everything from the lock line to the frontier. Fully simulated, and **uncapped**.

The middle band the ticket worried about does not exist, and no cap is needed. Simulation cost tracks **contested floors, not band height**: an enemy on a floor with no climbers has nothing to do. A floor exists as *data* (a seed, a cleared flag, a respawn timer) until a climber comes within a floor or two, at which point its enemies spawn as entities and despawn again behind them. With ~90 climbers, at most ~90 floors can be contested and realistically far fewer, so entity count stays in the low hundreds whether the band is 20 floors or 5,000. Arithmetic at the project's stated scale: a 200-floor band is ~1,000 enemies + 90 climbers ≈ 1,100 entities, ~11k component updates/sec at 10Hz — an order of magnitude inside Bevy's comfort. Rendering is bounded by the camera, not the band.

**Positions are continuous, inside a constrained coordinate space.**

```rust
struct FloorPos { floor: u32, x: NormX }   // NormX wraps f32, clamped to 0.0..=1.0
```

`NormX::new` clamps on construction, so no code path can produce an out-of-bounds value — not knockback, not a bad interpolation, not a NaN. World-space `Vec2` is *derived* for rendering only and is never authoritative. This makes the "units go outside the walls" class **unrepresentable rather than fixed**, while keeping smooth movement and real knockback (an impulse on `x`, clamped; vertical knockback decrements `floor`).

**Time**: fixed timestep in Bevy's `FixedUpdate` at **10Hz**, rendering interpolates between ticks. Frame-rate independence across a 144Hz desktop and a throttled phone is non-negotiable; the coarse rate is 6× cheaper on mobile and makes the closed-form sealed math plain arithmetic rather than an approximation of a continuous sim.

**There is no agreement contract.** The ticket assumed two paths computing the same thing within a tolerance. The settled partition eliminates that: sealed floors run no shadow simulation, they emit a rate. The seam is a **one-way, one-time conversion at lock time**. The divergence-looks-like-cheating failure mode is gone by construction.

The conversion: **measure the floor's actual output while active, freeze it at lock time, and scale it thereafter only by the prestige multiplier.** Honest (it's what you achieved there), legible (the player watched it being earned), and freezing prevents a feedback loop where every climber upgrade retroactively inflates every sealed floor. The intended consequence is that sealed floors decay in relative value, keeping pressure on locking higher.

**Floor lifecycle — two states, not one.**

- **Cleared** (all enemies dead) → stays empty, respawn timer starts, repopulates **fully** when it fires.
- **Uncontested but not cleared** (the wave died mid-fight) → **immediate full reset.**

This resolves the edge case where a floor is cleared, the last climber advances, and a trailing climber would otherwise arrive to a freshly-respawned full floor. It also blocks attrition on a wall: a floor you *failed* to clear snaps back the moment your last climber there dies, so the answers to a wall remain upgrading or stationing the hero, never waiting.

Per-enemy HP does **not** persist across deactivation. Only the cleared flag and the timer — an integer and a float per floor.

**Enemies respawn, and that is the farming mechanic.** Surfaced while resolving the above: the hero stationed below the wall farms gold, which requires something to farm. If cleared floors stayed cleared there would be nothing there. The respawn timer sets the gold rate for the whole active region below the wall, making it a first-class tuning knob — see ticket 06.

**Transitions.**

- **Locking past trailing climbers**: they **sprint to the new entry floor**. Nothing disappears, so nothing needs explaining. Costs one transient non-combat "sprinting" state. (Rejected: absorbing them, even with a notice — a disappearance is a thing the player must be taught, a sprint is not.)
- **Scrolling below the lock line**: **not permitted.** The camera has a hard lower bound at the lock line, with a visual indicator on the lowest visible floor showing the tower is sealed beneath. Simpler than rendering a dead region, and removes the empty-tower aesthetic problem entirely.

**Invariant discipline — wrap position and gold, nothing else.** The rule: wrap a value when a wrong one would be **silent**. A climber rendering happily inside a wall is silent; a gold underflow to a nonsense balance is silent. A negative cooldown fires an ability instantly and a wrong enemy count is visible on screen — both self-announce, so a wrapper buys ceremony rather than safety.

Status: resolved

## Amended by ticket 06

**Sealed floors produce no meaningful income, and the measure-and-freeze conversion is
withdrawn.**

Modelling the curve showed the rule cannot work as written. You only lock a floor once you
have safely climbed past it, so the band below the new line has been empty for minutes and
its measured 60-second output reads zero — four of eleven locks in a default run froze an
income of exactly zero.

Changing the measurement window does not rescue it. Gold per kill scales at 1.095/floor
against enemy health at 1.075, so a sealed band is always far behind the frontier in value:
the final frozen rate was 1.0×10⁴/s against a frontier income of 2.6×10⁷/s — **0.04% of the
money**. Even the closed-form ceiling for a band (what it would yield if farmed continuously)
came to 0.35%.

Accepted rather than fixed. **Locking is a spawn-point and travel-time mechanic, not an
economic pillar.** It still earns its cost — peak floor 176 with locks against 155 without —
because raising the lock line shortens the transit every climber re-walks.

Everything else in this ticket stands: the two-region partition, the uncapped active band,
`FloorPos` with clamped `NormX`, `FixedUpdate` at 10Hz, the floor lifecycle, and the transition
rules. The seam is still one-way; it just carries no gold across it.
