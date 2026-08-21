# Save schema and migration

Type: grilling
Status: resolved
Blocked by: 05, 07

## Question

What exactly is written to disk, and what happens when that shape changes?

Settled: hand-rolled `serde` to RON, one autosave slot plus a rotating backup, no persisted RNG state, no seeded determinism. Not settled:

1. **The full field list.** Timestamp, gold, peak floor, lock line, per-type climber ranks, hero roster with levels, prestige multiplier, relics held. Anything else? Ticket 05 may add persistent climber state; ticket 07 fixes what the offline calculation needs.
2. **Versioning.** A schema version field costs nothing now and is impossible to retrofit. What is the migration policy when a field changes — migrate, or reset with an apology?
3. **Autosave triggers.** On lock, on prestige, on a timer, on Android suspend? Ticket 01 establishes whether suspend gives reliable warning.
4. **Corruption handling.** When the primary save fails to parse, what happens — silent fallback to backup, or tell the player? Silent fallback can quietly discard hours of progress.
5. **What is deliberately *not* saved**, and can it be recomputed on load without the player noticing a discontinuity?
6. **Cheating posture.** A plain-text RON save is trivially editable. For a premium single-player game that may be entirely fine — decide it explicitly.

## Added by ticket 07

**The offline fields are fixed.** Ticket 07 settled offline gold as `best rate × min(elapsed,
12h)`, which costs the save exactly two things:

- a **timestamp** at write, and
- the **best rate** — one `f64`, the high-water mark of rolling 60-second gold/sec for the current
  run, reset on prestige.

No game clock and no RNG state, so `docs/adr/0001` stays affordable.

**What is deliberately not saved** (item 5) also has an answer for the active band: in-flight
climbers, contested floors and floor respawn timers are all dropped. On load the stream respawns at
the lock line and re-walks. The discontinuity is real but unnameable — ticket 05 established that
individual climbers are never tracked — and ticket 07 treats the resulting re-transit as part of
what offline gold pays for.

**Item 6, cheating posture, is partly pre-answered.** Ticket 07 accepted wall-clock manipulation
explicitly (premium, single-player, no leaderboards), hardened only by clamping elapsed to
`0..=cap` so a backwards clock cannot produce negative gold or an overflow. A plain-text RON save
is trivially editable and the same posture presumably applies — but that is this ticket's call, not
ticket 07's.

## Answer

### The partition is the answer to item 1, and it comes before the field list

The ticket asked for a field list. A flat list is the wrong first move: the save is authored as
**two serialised structs mirroring exactly what prestige resets** — `Account` and `Run` — plus an
envelope and a third tier that is never written at all.

The payoff is that prestige stops being a hand-maintained list of fields to zero and becomes
`run = Run::default()`. The set of things prestige resets **is the type**. Ticket 08's itemised
ledger enumerates precisely the losses in `Run`, so the ledger and the reset are generated from one
declaration and cannot drift. The alternative has the same list written down in three places — the
save, the reset, and the ledger — and two of them silently wrong.

`Account` is added to `CONTEXT.md` as the counterpart to `run`. The glossary had **Run** and
**Prestige** and no word for what outlives them, while the map's own fog was already using the term
informally ("does run length grow as the account matures").

**Envelope** — `version: u32`, `written_at: SystemTime`. Ticket 07's offline timestamp;
ticket 01 confirmed `SystemTime` is the correct clock, since `Instant` freezes while suspended.

**`Account`** — survives prestige — cumulative prestige multiplier · best-ever peak floor (drives
both relic milestone grants and ticket 11's new-depth-only currency) · relics held, each with its
rank · relic-currency balance · heroes unlocked · `prestige_count: u32` · `seen: HashSet<MomentId>`.

**`Run`** — cleared by prestige — gold · peak floor this run · lock line · melee/ranged/healer
ranks · per-hero level and experience · stationed hero and its floor · best rate.

**Active band** — never serialised — in-flight climbers and their HP · contested floors and enemy
HP · floor respawn timers · ability cooldowns · hero health and respawn countdown.

The **live position of the climb is not saved** — the field a reader will look for and not find.
Peak floor this run is saved, because it is what prestige scores; the moving edge is dropped, the
stream respawns at the lock line and re-walks, and ticket 03's repopulation timer means the floors
between have refilled anyway. Ticket 07 already ruled that re-transit part of what offline gold
pays for, so this is consistent rather than new. Notably, this ticket never needed the map's
missing word for the moving edge of the climb: nothing persists it.

**One-time moments are a set, not a bool.** Ticket 08's prestige explainer is derivable from
`prestige_count`, but it is the first of a category — first relic, first wall, first offline return
— and legibility has veto power on this project, so there will be more. A `HashSet<MomentId>` is
one line now and makes the second explainer an addition rather than a schema version bump.

**Nothing tuned is ever saved.** Composition, the replacement interval, the caps and the whole
curve are RON tuning data. A save that carries a tuning constant breaks hot reload for every player
who already has one.

### The save is authoritative — [ADR 0012](../../../docs/adr/0012-the-save-is-authoritative.md)

Three of this ticket's decisions turned out to be one decision wearing different clothes, and each
time the redundant-looking option was the right one:

- the **cumulative multiplier is stored, not derived** from a history of past peak floors — deriving
  it makes the earned-multiplier formula load-bearing forever, so a retune in a later patch silently
  moves every existing player's permanent power, which is exactly ADR 0007's nightmare;
- **unknown relic ids are retained verbatim, never pruned** to match the catalogue — ADR 0009 makes
  relics open RON data against a closed enum, so churn is expected, and dropping is irreversible
  while retaining makes a rename mistake recoverable;
- post-release, **schema changes migrate and never reset**.

All three say: no code path may retroactively reduce what the save records the player as having.
Recorded as one ADR with three consequences rather than three ADRs restating the principle. Before
the first public build the migration policy inverts — bump and reset, no migration code, since
there is no player to apologise to and a chain written against a schema still in motion is waste.

Versioning mechanics are cheap: serde ignores unknown fields, so a `struct Probe { version: u32 }`
parses any save well enough to dispatch on. Additive fields need **no** version bump at all —
`#[serde(default)]` covers them. The version exists only for what `default` cannot express: renames,
and fields whose meaning changed while their type did not.

### The failure paths, which is where the real findings were (items 3, 4)

**Corruption on read is never silent.** The map's legibility veto applies to persistence: this
project exists because Thousand Floors leaves players unable to answer *what did that just do for
me*, and a save that quietly rewinds them is the purest instance of that failure. On parse failure —
quarantine the bad file as `.corrupt-<timestamp>` and never delete it (it is the only evidence a
solo dev will get from a bug report), load the backup, and tell the player what they were restored
to **in the game's own terms**: "restored to your last session: floor 412, head start 108 floors",
not "save file corrupted". Same principle as ADR 0007 — state it in floors, not in machinery.

**First launch and corruption are two code paths from the first line.** Both arrive at the same
call site as an `Err` and conflating them fails in both directions: a first launch that apologises
for corruption, or a corrupted save that starts a silent new game and then autosaves over the
backup. Branch on `ErrorKind::NotFound` before attempting any parse. Missing → new game, no
message. Present-but-unparseable → the path above.

**The backup rotates on load, not on write** — and this is the finding that inverts the obvious
implementation. Walk the failure modes: process death mid-write is made unobservable by the atomic
rename, so the backup is not for that; bit-rot is real but rare. The mode that actually bites is a
**bug writing a valid-but-wrong save**, and against that a backup rotating on every write is
worthless — with focus-loss autosave firing constantly, the bad state overwrites the good backup
within seconds. So: at startup, after successfully parsing the primary, copy it to the backup slot.
The backup is then always *the state your last session ended in* — guaranteed to parse, because it
was just parsed — and far enough back to survive a bug that ran for a whole session. It also costs
zero frame budget on Android, happening at load and never inside the `Suspended` handler.

The copy is **byte-for-byte, never a re-serialisation**. Re-serialising would re-stamp `written_at`
and erase the player's offline entitlement on the one path where they are already having a bad day,
and would drop exactly the unknown relic ids ADR 0012 exists to preserve. Accepted consequence:
restoring from backup can pay offline gold for time that includes the lost session. It is bounded
by ticket 07's 12-hour cap and errs generous, which is the right direction on a failure path.

**A save from a newer version disarms every autosave trigger.** A player sideloads an older APK, or
a Steam beta branch rolls back; `version` reads 7 and the build knows 5. Refusing to load is the
obvious and harmless half. The hazard is what happens next — refuse, start a fresh game, and the
60-second timer overwrites the player's real save within a minute. Total, silent, permanent, and it
is the **default behaviour of every component agreed above**. Nothing prevents it accidentally. The
write-lock is the decision; the refusal is only its precondition. No writes at all until the player
upgrades or explicitly chooses to start over.

### Autosave triggers (item 3), largely pre-answered by ticket 01

Ticket 01 killed the two-phase save — `WillSuspend`/`WillResume` are never delivered — and named
`WindowFocused { focused: false }` as primary with `AppLifecycle::Suspended` as backstop. Two gaps
remained, so there are **four** triggers:

1. **Focus loss** — primary.
2. **`Suspended`** — backstop. Ticket 04 still owns confirming the write chain fits its one-frame budget.
3. **A 60-second timer** — the floor, covering a desktop crash or an Android low-memory kill from a
   state where focus was never lost and no lifecycle event fired.
4. **Prestige** — a commit point, not an optimisation. It is the only irreversible destructive act
   the player can perform, and the only write where a crash costs *correctness* (a half-applied
   prestige) rather than progress.

Explicitly **not** on lock or on rank purchase: those fire every few seconds and the timer covers them.

The write ritual is not a choice — temp file, `sync_all()`, atomic rename — so it is recorded rather
than decided.

### Where the file lives — a fact, not a decision

`bevy_android::ANDROID_APP` is a public `OnceLock<AndroidApp>` in 0.19.1 (`bevy_android/src/lib.rs:9`),
and `AndroidApp::internal_data_path()` returns a writable app-private directory — distinct from the
read-only APK asset reader ticket 01 flagged as having no writer. Desktop takes the platform config
directory. Settings live in a **separate `settings.ron`**, not in the save: they are untouched by
prestige, they can safely reset on a parse failure (so they need no migration chain), and — the real
reason — the rotate-on-load backup would otherwise rewind the player's preferences every time it
restored their progress. Restoring progress and restoring preferences are different operations and
must not be welded together.

### Cheating posture (item 6), and the thing it gets confused with

Two answers, deliberately separated, because conflating them produces anti-cheat theatre that stops
neither cheating nor bugs.

**On cheating: nothing.** Plain-text RON, no obfuscation, no checksum, no HMAC, no cheater flag.
Ticket 07 already accepted wall-clock manipulation on the grounds that this is premium,
single-player and has no leaderboards, and the same posture extends to the file. A readable save is
a positive asset for a solo dev — a player can paste it into a bug report.

**On validation: yes, but for robustness.** Ranges are validated on load — not to catch tampering,
but because the economy is `f64` and a single `NaN` from a hand-edited file propagates silently
through every multiplier in the game with no crash to point at. Clamp or refuse, and say which.

### Item 5, what is deliberately not saved

Answered above and by ticket 07: the entire active band. On load the stream respawns at the lock
line and re-walks; the hero loads **alive, on its stationed floor, with all four abilities ready**.
That last is technically exploitable — close and reopen to refresh cooldowns — and it does not
matter: no leaderboards, and relaunching the game to save one cooldown is more tedious than waiting.
Persisting four cooldown timers and a respawn countdown across a suspension would add fields and a
wall-clock reconciliation for zero player-facing gain. The discontinuity remains unnameable because
ticket 05 established individual climbers are never tracked.

Status: resolved
