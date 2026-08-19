# Progression curve, first pass

Type: prototype
Status: open
Blocked by: 03, 05

## Question

What are the actual numbers, and do they produce the intended feel?

Build a cheap standalone model — a spreadsheet, or a small headless Rust/Python sim, *not* game code — that lets us react to the curve concretely rather than argue about it abstractly.

It should express:

1. **Enemy scaling per floor** — the exponential base.
2. **Climber power per rank**, and **upgrade cost per rank** on a steeper base.
3. **Gold income** from kills, and the guaranteed rate from locked floors.
4. **Lock cost** as a function of floor.
5. **The prestige multiplier** — what it is a function of (peak floor reached?) and how fast it grows.
6. **The hero's contribution**, stationed at a given floor.

Then answer with it:

- How long is a first run before the first prestige feels earned? Minutes or hours?
- Where do walls naturally fall, and are they spaced to feel like rhythm rather than grind?
- Does the second run feel meaningfully faster than the first? By how much?
- Does the curve still work at floor 1,000, or does it break down?

Output the constants in the RON shape they will ship in, so the prototype and the game agree. Link the artifact from this ticket.

## Added by ticket 03

The model gained a knob: **the floor respawn timer**. Cleared floors repopulate fully after it fires, which makes it the gold rate for the entire active region below the wall — and therefore the thing that determines whether stationing the hero below the wall to farm is worth doing at all. Model it explicitly alongside the other constants.

Note the interaction the curve has to balance: a short timer makes farming below the wall strong and pushes players to sit still; a long one pushes them to keep climbing. That tension is the hero-placement tradeoff in ticket 09, expressed as a number.

Also settled: sealed-floor income is **measured at lock time and frozen**, scaled thereafter only by the prestige multiplier. So sealed income does not benefit from later climber upgrades — model it as decaying in relative value.
