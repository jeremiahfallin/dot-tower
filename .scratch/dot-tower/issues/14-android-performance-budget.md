# Low-end Android performance budget

Type: task
Status: open
Blocked by: 04

## Question

Nothing to decide until something is measured. Ticket 03 made the performance model concrete and falsifiable, so measure it on real hardware and find where it breaks.

The model to test: entity count is bounded by **contested floors**, not band height, because floors activate lazily as climbers approach and deactivate behind them. Predicted worst case at project scale is ~1,100 entities (a 200-floor band at ~5 enemies/floor, plus 90 climbers, plus the hero) ticking at 10Hz.

Measure on the lowest-end device available:

1. **Does the prediction hold?** Instrument entity count and frame time with a synthetic band. Where does frame time actually degrade — 1,000 entities, 5,000, 20,000?
2. **What is the real climber spread?** The bound assumes climbers clump behind the wall. If they spread thin across 90 floors, activation churn is much higher than predicted.
3. **Activation churn cost.** Spawning and despawning a floor's enemies as climbers pass is the mechanism the whole model rests on. What does one activation cost, and how many per second happen at normal play?
4. **The 10Hz tick under thermal throttling.** Does a sustained session throttle, and does the fixed timestep start accumulating catch-up ticks? Catch-up spirals are the classic fixed-timestep failure.
5. **Memory** for inert floor data at a large band — 5,000 floors of seed + flag + timer.
6. **Battery draw** over a 30-minute session. An idle game people leave running has a battery reputation to protect.

Record the numbers, the device, and where the model first fails. If it fails, the cap rejected in ticket 03 is the fallback — reopen that decision with data rather than reasoning.
