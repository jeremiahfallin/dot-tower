# Low-end Android performance budget

Type: task
Status: claimed
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

## Comments

### Demand measured; supply awaits the device (session of 2026-09-07)

Ticket 04 resolved and this became the frontier. **Still `claimed`, not
`resolved`** — this ticket's question is where frame time degrades *on hardware*,
and no device is attached. What follows is everything answerable without one,
which turned out to be more than expected.

The ticket's six questions are really two, and they need different instruments:

- **Demand** — how many entities and how much churn the game produces. A
  property of the simulation, not of the phone. **Ticket 06's model already knew
  this and had never been asked.**
- **Supply** — what the device absorbs. Needs the device.

Prototype: [`prototypes/14-performance-budget/`](../prototypes/14-performance-budget/),
with readings in `readings.txt`. The entity accounting was added to
`06-progression-curve/model.mjs` additively — new fields on existing sample
objects, read-only over the floor map, no simulation behaviour reads any of it.
Ticket 19's `reach.mjs` output is **byte-identical** before and after, and ticket
15's three prototypes run unchanged.

#### Q1, demand half — the prediction is conservative by 5×

**Max 205 entities** across 6,533 samples of a 14-run campaign reaching floor
1,647. The predicted worst case is 1,100. The measured maximum is **18.6% of
it**, and the mean is 129.

This does not answer where frame time degrades. It does something better for the
device session: it says the range worth testing is **200–450 entities**, not the
1,000 / 5,000 / 20,000 the ticket guessed at. Testing 20,000 would have been
measuring a case the game never produces.

#### Q2 — climbers spread thin, and it is free

The ticket's worry was that a thin spread would drive churn far above prediction.
**They do spread thin** — up to 581 floors — and it costs nothing. Activation
churn peaks at **5.2/s**, mean 2.0/s.

The reason is worth keeping, because it is not what ticket 03 argued. Ticket 03
bounded cost by *contested* floors. The real mechanism is that an active floor
holds **~1.4 enemies against a pack size of 4**: trailing climbers walk through
ground the leaders already cleared. **Spread buys activation events, not
entities.** Contested floors run 2–8 against ticket 03's "at most ~90" bound, and
*decrease* with depth as the column stretches out.

#### The load-bearing claim, tested directly

Ticket 03: *"entity count stays in the low hundreds whether the band is 20 floors
or 5,000."* Driving prestige ×1 → ×10¹² grows the band **6×** (67 → 399 floors)
while entities go **down**, 164 → 153. The claim holds, and it is the one the
whole no-cap decision rests on.

Also settled: **the 5,000-floor band does not occur.** Band height is bounded by
how far a single run climbs, and peak floor is logarithmic in prestige — ×10¹²
reaches floor 1,097. The largest band seen was 399.

#### Q5 — memory is a non-issue

16 B per floor record (f64 timer + u32 count + u32 seed/flags). Ticket 14's
hypothetical 5,000 floors is **78 KiB**; 50,000 floors is 781 KiB. Nothing to
budget.

#### A finding that is not about performance, and belongs to ticket 20

Population sits at **exactly 90 in every run** — the sum of the three per-type
caps. Raising the ceiling: 180 → 270 entities, 360 → 366, then it **saturates**,
with 720 and 1440 producing identical runs because population is spawn rate times
lifetime and the ceiling has stopped touching it.

So `CONTEXT.md`'s **Type cap** entry — *"under a healthy lock curve it does not
bind at all — the stream sits well below it"* — is **false at the current value
of 90** and true from roughly 360 up. **Not acted on, and the glossary is
deliberately not edited**, for two reasons:

1. The model still runs the **superseded** mechanism — three caps and three spawn
   intervals. [Ticket 15](15-climber-composition.md) replaced it with one global
   replacement interval against an authored ratio. The *shape* of the question
   survives that change; the values do not.
2. Crowd size belongs to [ticket 20](20-travel-time-and-the-lock-curve.md),
   which is open and owns "how big the crowd is and how far it walks".

The number ticket 20 will want: with no ceiling binding, the crowd settles near
**405 climbers** and the whole simulation costs **~436 entities** — still under
40% of the 1,100 prediction. Whoever updates the model to ticket 15's mechanism
should re-run `entities.mjs`; it is the check that would catch this claim being
wrong in the shipped design rather than the superseded one.

#### What remains, and it all needs the device

- **Q1, cost half.** Where frame time degrades. Test 200–450 entities.
- **Q3, cost half.** What one activation costs. The rate is 1–5/s.
- **Q4.** Thermal throttling and fixed-timestep catch-up over a sustained session.
- **Q6.** Battery draw over 30 minutes.

None is blocked by [ticket 22](22-bevy-ui-android-rendering.md): all four are read
off logcat, not off the HUD that does not render.

#### The cold build ticket 04 owed

Measured here because it needed no device. **8m 07s** wall for a cold
`aarch64-linux-android` debug build (2,896s CPU across ~5.9 cores), producing the
1.4 GB unstripped `.so` ticket 04 already recorded. Built into a separate
`CARGO_TARGET_DIR` so the working cache was not destroyed to get the number;
11 GB of intermediates, discarded afterwards.

Ticket 04's table is updated in place. The practical consequence for anyone
working this ticket: an incremental Rust change is cheap, but a `cargo clean`, a
toolchain bump or a `Cargo.toml` feature edit costs eight minutes before Gradle's
16s even starts — so batch the device experiments rather than rebuilding between
each one.
