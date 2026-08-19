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
A permanent-within-a-run gold purchase applied every 10 floors that raises the spawn point and guarantees a passive income rate from the floors beneath it.
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
The upgrade tier of a climber type, purchased with gold. Rank is per type, and resets on prestige.
_Avoid_: Level, tier, star

**Hero**:
The single persistent unit the player selects and stations on a floor it repeats across. It fights automatically; its abilities are triggered by hand.
_Avoid_: Champion, leader, commander, climber

**Aura**:
A continuous effect applied to every friendly within a radius, with no target selection. The healer's heal is one.
_Avoid_: Buff, field, pulse

**Type cap**:
The maximum number of live climbers of a single climber type. Each type has its own; there is no cap across the stream as a whole.
_Avoid_: Population cap, unit limit, squad size

**Ability**:
One of four cooldown-gated actions a hero possesses, fired manually by the player. There is no mana or energy resource.
_Avoid_: Skill, spell, art, power

### Progression

**Run**:
The span of play between two prestiges.
_Avoid_: Session, attempt, life

**Prestige**:
Deliberately ending a run to convert its progress into a permanent multiplier. Resets gold, floor progress, locks, climber ranks, and hero levels.
_Avoid_: Rebirth, ascension, reset

**Prestige multiplier**:
The single permanent scalar applied to climber and hero health and damage. The only source of permanent raw power.
_Avoid_: Ascension bonus, legacy bonus

**Relic**:
A permanent qualitative modifier — income, lock cost, cooldowns, spawn rate, climber-type behaviour. Relics never grant raw health or damage; that is the prestige multiplier's sole job.
_Avoid_: Artifact, trinket, rune
