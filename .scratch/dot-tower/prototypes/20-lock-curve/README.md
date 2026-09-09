# Ticket 20 — travel time and the lock curve

**The prototype is not throwaway, and that is deliberate.** Every earlier ticket
measured on `06-progression-curve/model.mjs`; ticket 19 caught that model
inverting a headline result and ticket 14 found it had never implemented ticket
03's floor-reset rule, so the map's fog asks for an instrument that can be
checked against the decisions it defends. This ticket was measured on
[`crates/sim/`](../../../../crates/sim) — the shipping simulation — which is
verified tick for tick against the JS model on both lock curves.

## Reproducing

```
cargo run --release -p dot-tower-sim --bin ticket20     # the sweeps in readings.txt
cargo run -p dot-tower-sim --bin headless               # the landed curve
cargo run -p dot-tower-sim --bin headless -- --tuning sweeps/pre-ticket-20.ron
cargo test -p dot-tower-sim --release                   # incl. model parity
```

## Method

- **Wall-clock pinned.** Six runs of 60 minutes each; readings are run 6.
  Ticket 19 established that a stall heuristic cannot be compared across
  variants, because a variant that grinds rather than stalls buys depth from run
  length alone. The harness has no stall detector at all.
- **`sealed_income` off.** The JS model still pays frozen income from sealed
  floors, which ticket 06 withdrew. Leaving it on overstates the case for a
  cheaper lock by 59% — see the ticket's first comment.
- **Crowd, transit and back are means over the run's last 20 minutes.** A single
  instant of a small stream lands wherever the last death did.
- **Two buy policies.** `cheapest` is what every recorded number used. But under
  it a lock priced above a rank is passed over until ranks grow past it, so the
  gate is rank cost rather than affordability — and ticket 20's question 1 is
  precisely whether the lock is bought *the moment it is affordable*. `lockfirst`
  exists to make that readable.

## Columns

| column | meaning |
|---|---|
| `k` | `lock_cost_base / gold_base^10`. Above 1, lock cost outruns income and compounds. |
| `transit` | floors from the lock line to the frontier — the walk every climber re-makes |
| `back` | how far behind the frontier the average climber sits |
| `goldwait` | seconds a lock sat permitted-by-depth and unaffordable |
| `auto` | share of locks bought within one decision cycle of becoming permitted |
| `walk` | share of climber-ticks spent walking rather than fighting |
| `skips e/l` | replacement slots skipped at a type ceiling, first third / last third of the run |
| `head` | head start as a share of peak floor (ticket 08 measured ~26%) |
| `+10m` | floors gained in the run's final ten minutes — is the climb still moving? |
