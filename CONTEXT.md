# dot-tower

A pixel-art idle tower-climbing auto-battler. Climbers stream endlessly upward through a tower while the player upgrades them, stations a hero, locks floors behind them, and periodically resets for a permanent multiplier.

## Language

### The tower

**Floor**:
A single position in the tower, numbered from 1 upward without limit. Enemies, locking, and hero stationing are all expressed in floors.
_Avoid_: Level, stage, storey

**Tower**:
The unbounded vertical sequence of floors. There is exactly one.
_Avoid_: Dungeon, run map

**Lock**:
A permanent-within-a-run gold purchase applied every 10 floors that raises the spawn point, so
every climber's walk to the front starts higher. It is a spawn-point and travel-time mechanic:
the floors beneath a lock are sealed and produce **no** income. The price is set in **time** —
a fixed number of seconds of recent income, so it tracks the economy by construction — and a
lock below the **account**'s deepest-ever floor is free, because head start is conquered
territory.
_Avoid_: Seal, lockdown, gate

**Lock line**:
The highest locked floor. Climbers spawn here; nothing below it is rendered or simulated.
_Avoid_: Checkpoint, spawn floor

**Wall**:
The floor at which climbers can no longer reliably win, halting upward progress until the player intervenes.
_Avoid_: Gate, plateau, softlock

### Units

**Climber**:
An expendable, anonymous unit that spawns at the lock line and fights its way upward. Individual climbers are never named, tracked, or directly commanded.
_Avoid_: Unit, summon, minion, hero

**Climber type**:
A named behavioural archetype shared by many climbers — melee, ranged, or healer. Type is where identity, rank, and relic affinity attach.
_Avoid_: Class, role, job

**Rank**:
A **purchased** upgrade tier. A rank is bought; a level is earned — the two are not interchangeable,
see **Level**. Two things have ranks, and they differ in currency and in what resets: a **climber
type**'s rank is bought with gold and resets on prestige; a **relic**'s rank is bought with the
currency prestige grants and does *not* reset.
_Avoid_: Tier, star

**Level**:
The progression tier of the hero, **earned with experience** rather than bought. There is no
gold cost and no purchase decision. Levels are per hero and reset on prestige.
_Avoid_: Rank, hero rank

**Experience**:
The quantity a hero accumulates from enemies defeated inside its aura, and its only source of
levels. Enemies killed elsewhere in the tower grant none, so experience reflects where the hero
has actually fought. Not a currency: it cannot be spent, only accrued.
_Avoid_: XP, exp, points

**Hero**:
The single persistent unit the player selects and stations on a floor it repeats across. Its
contribution is its **aura**, which multiplies what climbers on that floor already do; it also
fights automatically, but that damage is deliberately not where its value lies. Its abilities are
triggered by hand. It gains **levels** from **experience**, never from gold.
_Avoid_: Champion, leader, commander, climber

**Aura**:
A continuous effect applied to every friendly within a radius, with no target selection. The
healer's heal is one; the hero's damage-and-gold multiplier is another. An aura applies only while
its source is alive, which is what makes hero uptime matter.
_Avoid_: Buff, field, pulse

**Taunt**:
A hero property, not a universal rule: a taunting hero is the primary target on its floor and
absorbs damage that would otherwise fall on climbers. Whether a hero taunts is part of what
distinguishes one from another, alongside its abilities and its health.
_Avoid_: Aggro, threat, provoke

**Type cap**:
A **ceiling** on the number of live climbers of a single climber type. Each type has its own; there
is no cap across the stream as a whole. It is not a lever: it is never bought with gold, and under
a healthy lock curve it does not bind at all — the stream sits well below it.
_Avoid_: Population cap, unit limit, squad size

**Composition**:
The ratio of climber types in the stream. It is **authored** — a fixed design statement the player
never sets, chosen to make the tower read as an army rather than tuned to an optimum. Composition
is not one of the game's decisions, and describing it as a choice is a category error.
_Avoid_: Party, loadout, army mix, squad

**Replacement**:
The single global interval at which a fallen climber is replaced. One interval for the whole
stream, applied against the authored **composition** — deliberately not one per climber type,
which would make the ratio emergent from six numbers instead of stated in one. It is the dial that
sets run length, and the rate at which climbers die is a consequence of it rather than of combat.
_Avoid_: Replacement rate, replacement interval (both retired in favour of the bare term), respawn (that is the enemy repopulation timer), spawn rate

**Ability**:
One of four cooldown-gated actions a hero possesses, fired manually by the player. There is no mana or energy resource.
_Avoid_: Skill, spell, art, power

### The screen

**Tower column**:
The vertical strip of floor rows the player watches, showing a window of the tower centred on the
highest floor climbers have reached and clamped so it can never scroll below the lock line. It is
the same width on a phone and on a monitor; a wider frame adds panels beside it rather than
enlarging it.
_Avoid_: Tower view, viewport, playfield, stage

**Strip**:
The single band above the ability bar carrying everything temporal — hero uptime over the last
45 seconds, a record of ability presses, and a plain-language verdict on the most recent one.
Everything spatial belongs on the **tower column** instead, and nothing appears in both.
_Avoid_: HUD, status bar, panel, ticker

### Progression

**Run**:
The span of play between two prestiges.
_Avoid_: Session, attempt, life

**Account**:
Everything that outlives a **prestige** — the cumulative prestige multiplier, the deepest floor
ever reached, and the relics held with their ranks. It is the counterpart to **run**: between them
they partition all progress, so every quantity the player owns belongs to exactly one of the two,
and prestige is precisely the act of discarding the run and keeping the account. What the player
watches mature across many runs.
_Avoid_: Profile, meta, save, permanent progress

**Prestige**:
Deliberately ending a run to convert its progress into a permanent multiplier. Resets gold, floor progress, locks, climber ranks, and hero levels.
_Avoid_: Rebirth, ascension, reset

**Prestige multiplier**:
The single permanent scalar applied to climber and hero health and damage. The only source of
permanent raw power. Two readings matter and must not be confused. The **earned** multiplier is
what one prestige is worth on its own, a function of that run's peak floor; it stays small and is
shown to the player. The **cumulative** multiplier is the running product of every earned
multiplier so far; it grows without bound and is never displayed — see **Head start**.
_Avoid_: Ascension bonus, legacy bonus

**Head start**:
The cumulative **prestige multiplier** expressed as a number of floors: how far the whole tower has
dropped. With a head start of *h*, floor *f* now fights exactly as floor *f − h* did before,
because enemy health and damage scale per floor while the multiplier scales climbers — so it is an
exact restatement, not a friendly approximation. This is the only form in which accumulated
prestige is shown.
_Avoid_: Floors skipped, free floors, offset, ascension level

**Relic**:
A permanent qualitative modifier, granted the first time the climb reaches a milestone floor. Each
relic is unique, is never lost, and has its own **rank**. Relics never grant uniform health or
damage — that is the prestige multiplier's sole job — and never a flat percentage of gold, lock
cost or rank cost, which the curve erases.
_Avoid_: Artifact, trinket, rune

**Effect kind**:
One entry in the closed set of things a relic is able to modify. A relic is a *condition*, an
effect kind, and a magnitude; the set of relics is open and lives in tuning data, the set of effect
kinds is closed and changing it is a design decision. No two relics share an effect kind.
_Avoid_: Effect type, modifier type, stat

**Offline gold**:
The gold granted for time spent with the game closed. It is the only thing earned while away — the
climb does not advance — and it is computed rather than simulated, as **best rate** multiplied by
elapsed time up to a cap.
_Avoid_: Offline progress, offline return, idle gold, AFK earnings

**Best rate**:
The highest sustained gold-per-second reached during the current run, and the rate **offline gold**
pays at. It is a high-water mark rather than a recent average, so it cannot be sampled at an
unrepresentative moment, and it resets on prestige.
_Avoid_: Holding rate (holding is hero uptime), peak rate (peak is the floor), offline rate
