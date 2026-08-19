# Offline gold formula and the return-screen contract

Type: grilling
Status: open
Blocked by: 06

## Question

What exactly does the player earn while away, and what does the game say when they come back?

Settled: gold only, closed-form, capped at roughly 8-12 hours, no floor progress. Not settled:

1. **The formula.** Gold as a function of which locked floors, their rate, and elapsed wall-clock time. Does every locked floor contribute, or only the highest band?
2. **The cap.** Where exactly, and is it a hard stop or diminishing returns past the threshold? A hard stop is more legible; diminishing returns is kinder to someone who sleeps nine hours.
3. **Clock trust.** Wall-clock time on a device the player controls invites trivial cheating by changing the system clock. Do we care? For a premium single-player game with no leaderboards the honest answer may be "no" — but decide it deliberately rather than by omission.
4. **The return screen.** Under the legibility constraint this must be answerable in one sentence: *"Away 6h 12m. Floors 1-120 earned 4.2M gold."* What exactly does it show, and does it appear as a modal, a banner, or a ledger entry?
5. **Zero-progress honesty.** If the player was away 20 minutes and earned almost nothing, does the screen still appear? Suppressing it hides the mechanic; always showing it becomes noise.
6. **First-session behaviour.** Before anything is locked, offline earns nothing. Does the game explain why, or silently show zero? Silently showing zero is exactly the Thousand Floors failure.
