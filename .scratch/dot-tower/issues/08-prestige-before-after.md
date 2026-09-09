# The prestige before/after screen

Type: prototype
Status: resolved
Blocked by: 06

## Question

What does the player see when they prestige?

This is the single highest-leverage screen in the game. The specific complaint that sank the reference game is a player prestiging and not knowing what it bought. Under the legibility constraint, this screen is a shipping requirement.

Make a rough mockup — paper, a static image, an HTML sketch, anything reactable — and use it to answer:

1. **What comparison is shown?** Climber damage before vs after, health before vs after, the multiplier itself, projected time-to-reach-previous-floor?
2. **Before committing or after?** A preview that says *"prestige now and your multiplier goes 1.4x to 1.9x"* is far stronger than a post-hoc report, and it turns prestige into a judgement rather than a leap of faith.
3. **What is lost?** Gold, floors, locks, climber ranks, hero levels all reset. Is the loss shown as explicitly as the gain? Hiding it is dishonest; showing it badly makes prestige feel punishing.
4. **How is "was it worth it" answered on the *next* run** — does anything persist to compare against, or does the player have to remember?
5. **First prestige specifically.** The player has never seen this before and has no baseline. Does it need explaining, and where?

Link the mockup from this ticket.

## Added by ticket 06

Settled: **the multiplier compounds** — each run's multiplier multiplies the last. This was
forced rather than chosen. Deriving it from best-ever floor instead reaches a fixed point around
floor 220 and the game stops permanently, because floor buys multiplier buys floor and the loop
closes.

The cost lands squarely on this ticket. Compounding reaches **×10¹¹ by floor 1,000** and keeps
going, which is precisely the illegible number this project exists to avoid. The model's
suggestion: present it as **floors of head start** — `ln(M) / ln(enemy_hp_base)` — so ×10¹¹
reads as "worth about 350 floors" instead. The raw multiplier should probably never appear.

Also relevant: the multiplier buys floors logarithmically while floors cost exponentially, so
run two re-reaches run one's ceiling in about a third of the time and that ratio shrinks every
run after. Whatever this screen promises has to stay honest as the returns flatten.

## Answer

**The ledger, itemised, before committing, every time — with a one-time explainer ahead of the
first.** The screen names every line that resets at its real value, and prices the gain in two
numbers only: the multiplier *this run earns*, and the head start in floors. Nothing is collapsed
behind a disclosure and nothing is softened into "you'll have it back in ten minutes".

### The two multipliers

The decision that unlocks the rest. There are two, and only one is legible:

- **Earned** — `M = (1 + peak/50)^1.2`, what *this* prestige is worth on its own. ×5.18 at floor
  147, ×7.10 at 206, ×22.24 at 613, and still only **×38.6 at floor 1,000**. It is a log-ish
  function of peak floor, so it stays two or three digits essentially forever. This is the number
  on the screen.
- **Cumulative** — the running product. ticket 06 measured it at **×10¹¹ by floor 1,000**, and it
  keeps going. It appears **nowhere**, ever.

The cumulative multiplier is carried instead by **head start in floors**, `ln(M) / ln(1.075)`.
This is not a friendly translation of an unfriendly number — it is *exact*: with head start `h`,
floor `f` next run fights precisely like floor `f − h` did, because enemy health and damage both
scale on 1.075 per floor while the multiplier scales climbers. ×10¹¹ is "worth about 350 floors",
and the sentence is true rather than approximate.

**Measured: head start settles at ~26% of peak floor** — 15%, 24%, 27%, 27%, 27%, 26% across runs
1–6. The relationship the screen promises is stable, so it does not need re-explaining as the
account matures, and there is no point at which the number stops meaning what it meant.

### The screen

Two blocks, losses above gains, struck-through old value beside new.

**YOU GIVE UP** — gold, peak floor, locks, melee rank, ranged rank, healer rank, hero level. At
the end of run 1 that reads `36.75m → 0`, `147 → 1`, `9 × 10 floors → none`, `56 → 1`, `54 → 1`,
`54 → 1`, `50 → 1`. Deliberately unsoftened. The alternative tested — pricing the whole loss as
"you'll be back here in about ten minutes" — is true, and was rejected: it is the game deciding
for the player how to feel about their own run.

**YOU KEEP** — `This run earns ×5.18`, `Head start 0 floors → 23 floors`, `Relics 3 → 3`.

**The footer** carries the summary the ledger cannot: *every floor in the tower is now 23 floors
easier; you should be back at floor 147 in about 10 min*. This is the mitigation for the ledger's
one real risk — that a wall of numbers is itself the illegibility this project exists to avoid.
Both figures are measured, not projected: 10 min is when run 2 actually crossed floor 147.

### Before committing (ticket Q2)

**Before.** The screen is the preview; confirming is a second act. It shows `PRESTIGE` and
`KEEP CLIMBING`, and reappears unchanged after the fact only as a post-hoc report if the player
wants it. A leap of faith was never on the table under the legibility constraint.

### When to prestige is a stall question, not a multiplier question

Measured at run 1's stall: **one more floor costs 1.4 minutes and buys +0.084 floors of head
start.** Prestiging buys 22.8. The margin is three orders of magnitude, and it never narrows —
run 5: one floor costs 3.3 min and buys +0.025, against 42.9 for prestiging.

This kills the "is my multiplier big enough yet" framing entirely. There is no threshold to wait
for; there is only *have I stopped climbing*. Consequence for the screen: the trigger matters more
than the content. Ticket 06's stall condition (peak flat for ~6 minutes) is what should surface
the prestige affordance, and the run header line — `RUN 1 · 41 MIN` — is where the stall reads.
The ledger itself does not argue; it does not need to.

### The first prestige (ticket Q5)

**A one-time explainer, shown once, ahead of the first ledger.** It says the run is over, names
everything that resets, states the exchange in floors, and — the load-bearing line — promises
*"You'll see it every time, so you never have to take this on trust."* Then `SHOW ME`, and the
ledger. It never appears again.

This is a tutorial panel in a project that has otherwise avoided tutorials. It earns the exception
because the first prestige is the only moment where the player has no baseline at all, and it is
the exact moment the reference game loses people.

### Was it worth it, on the next run (ticket Q4)

**Nothing persists to compare against, and nothing needs to.** Head start is on the ledger every
time, so run *n+1*'s screen shows `23 floors → 50 floors` — the previous prestige's result is the
starting value of the next one's. The player never has to remember a number; the ledger is its own
history.

The stronger treatment — a chart of last run against this run, with the real line drawn over the
projection — was built and rejected for the slice as a second temporal surface competing with
ticket 09's strip. Noted in the map's fog rather than discarded.

### What was rejected

- **B, the diegetic tower drop** — a lit band on the tower showing every floor sliding down 23,
  no modal at all. The clearest single statement of what prestige does, and consistent with ticket
  09's spatial/temporal split. It cannot carry the loss: the costs end up as chips in a sheet,
  which is the ledger again with less room.
- **C, two runs** — this run and the projected next on a floors-against-minutes chart. The flat
  tail of the current line *is* the stall, shown rather than stated, and it answers Q4 for free.
  Rejected as a second sparkline surface; the projection is also a promise the game must then keep.
- **D, the verdict** — one sentence, the loss priced in minutes, ledger collapsed. Most legible,
  and the reason it lost is the reason it was tempting: it tells the player what to think about a
  decision that is theirs.

## Prototype

Built 2026-08-20. Throwaway, per `prototype` (UI branch, sub-shape B — a Bevy game has no existing
route to host variants).

- **Flip through it**: https://claude.ai/code/artifact/640ab93c-f216-4158-80a3-74b1b602d9cd
  Variants at `?variant=A|B|C|D`, switchable from the floating bar or the ← → keys. Controls for
  account state (1st prestige / 2nd / mature) and moment (first time only / before committing /
  just after / next run).
- **Source**: `.scratch/dot-tower/prototypes/08-prestige-screen/` — `prestige.html` is one
  self-contained file; `dump.mjs` regenerates the run table it is built on.
- **Branch**: `prototype/prestige-screen`.

Every number is measured. `dump.mjs` walks ticket 06's `model.mjs` unmodified as a seven-run
campaign with the ticket-16 hero (×2 aura, taunting, stationed at the wall); the "projected next
run" is the *actual* next run of that campaign. This mattered more here than in most prototypes:
the whole question is whether the screen can make a true statement the player can act on, and a
screen built on invented numbers cannot be judged for honesty.

Run 7's stall detection is noise — ticket 06 flags the heuristic at ±40% and at that depth it
shows — so the mature case stops at prestiging out of run 5.

## Comments

### Amended by ticket 17 (2026-09-08)

The multiplier a run earns now pays for peak beyond the account's best-ever
floor ([ADR 0014](../../../docs/adr/0014-the-multiplier-pays-for-new-territory.md)).
For this ticket's screen that means:

- The ledger's earned-multiplier line is unchanged for a fresh account (Δ =
  peak, so the ×5.18-at-147 numbers stand), and at maturity it shows the
  honest small number: ×2.2–3.0 per mature run for 48–76 new floors. A run
  that never passed the account's best shows **×1.00** — the itemised
  before-committing design already makes that visible rather than a surprise,
  which is exactly why this basis is safe to ship.
- "Prestige is a stall decision" is retired at maturity: prestiging before
  passing the best earns nothing, just past it is optimal (1.5× faster than
  waiting for the stall). The trigger this screen serves is now the
  **past-your-best moment**, and marking it belongs to ticket 18's surface.
- Head start settles at ~21% of peak at maturity (26% early) — the promise
  still never needs restating.
