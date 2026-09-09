//! What is written to disk: two structs mirroring exactly what prestige resets.
//!
//! [Ticket 12](../../../.scratch/dot-tower/issues/12-save-schema.md) settled the
//! shape, and the shape *is* the argument. A flat field list would have the same
//! enumeration written down in three places — the save, the prestige reset, and
//! ticket 08's itemised ledger — and two of them silently wrong. Here prestige
//! is [`Run::default`], and the ledger is generated from the same declaration.
//!
//! The counterpart to that is [`Account`]: everything that outlives a prestige.
//! Between them they partition all progress, so every quantity the player owns
//! belongs to exactly one of the two.
//!
//! A third tier is **never serialised at all** — in-flight climbers, contested
//! floors, respawn timers, ability cooldowns, hero health. On load the stream
//! respawns at the lock line and re-walks; ticket 07 already ruled that
//! re-transit part of what offline gold pays for.
//!
//! I/O lives in the `dot-tower-save` crate, not here. These are game state.

use crate::curves::prestige_mult_for;
use crate::geometry::Floor;
use crate::tuning::Tuning;
use crate::types::PerType;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Bumped only for what `#[serde(default)]` cannot express: renames, and fields
/// whose meaning changed while their type did not. Additive fields need no bump.
///
/// Before the first public build the migration policy is **inverted** — bump and
/// reset, no migration code — because there is no player to apologise to and a
/// chain written against a schema still in motion is waste. After it,
/// [ADR 0012](../../../docs/adr/0012-the-save-is-authoritative.md) applies and
/// schema changes migrate.
pub const SCHEMA_VERSION: u32 = 1;

/// A relic's identity. **Open data, not a closed set.**
///
/// [ADR 0009](../../../docs/adr/0009-relic-effects-avoid-axes-the-curve-erases.md)
/// makes the catalogue churn by design — relics are RON data against a closed
/// enum of effect *kinds* — so a save will routinely name relics this build has
/// never heard of. They are kept verbatim and never pruned: dropping is
/// irreversible, retaining makes a rename mistake recoverable.
pub type RelicId = String;

/// One of the moments the game explains exactly once.
///
/// A set rather than a bool per moment. Ticket 08's prestige explainer is
/// derivable from [`Account::prestige_count`], but it is the first of a category
/// — first relic, first wall, first offline return — and legibility has veto
/// power on this project, so there will be more. A set makes the second
/// explainer an addition rather than a schema bump.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MomentId(pub String);

impl MomentId {
    /// The project's one tutorial, earned because the first prestige is the only
    /// moment with no baseline to compare against.
    pub fn first_prestige() -> Self {
        Self("first-prestige".into())
    }
    pub fn first_relic() -> Self {
        Self("first-relic".into())
    }
    pub fn first_wall() -> Self {
        Self("first-wall".into())
    }
    pub fn first_offline_return() -> Self {
        Self("first-offline-return".into())
    }
}

/// A hero's progression within the current run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HeroProgress {
    /// Earned with experience, never bought
    /// ([ADR 0005](../../../docs/adr/0005-the-hero-multiplies-climbers-add.md)).
    pub level: u32,
    /// Accumulated from enemies defeated inside this hero's aura, and its only
    /// source of levels. Not a currency: it cannot be spent.
    pub experience: f64,
}

impl Default for HeroProgress {
    fn default() -> Self {
        // A hero the player has is level 1, not level 0 — and prestige resets it
        // to 1 rather than deleting it.
        Self { level: 1, experience: 0.0 }
    }
}

/// The one hero the slice ships. Ticket 16 settled that heroes differ by
/// abilities, health and whether they taunt, and that the tank is the one that
/// proves the aura-and-uptime loop; the roster beyond it is in the map's fog.
/// The save is already keyed by hero so that adding the second one is data.
pub const SLICE_HERO: &str = "tank";

/// Everything that outlives a prestige.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Account {
    /// **Stored, not derived** from a history of past peak floors. Deriving it
    /// would make the earned-multiplier formula load-bearing forever, so a
    /// retune in a later patch would silently move every existing player's
    /// permanent power — exactly ADR 0007's nightmare.
    ///
    /// Never shown to the player. It appears only as head start in floors.
    pub cumulative_prestige_mult: f64,
    /// Drives relic milestone grants and ticket 11's new-depth-only currency.
    pub best_peak: Floor,
    /// Relic id to rank. Unknown ids are retained verbatim.
    pub relics: BTreeMap<RelicId, u32>,
    /// The currency prestige grants, earned on peak floor beyond the previous
    /// best. Still unnamed — see the map's fog.
    pub relic_currency: f64,
    pub heroes_unlocked: BTreeSet<String>,
    pub prestige_count: u32,
    /// One-time moments already shown. `BTreeSet` rather than `HashSet` so the
    /// file has a stable order and a diff between two saves is readable — this
    /// is a plain-text save a player may paste into a bug report.
    pub seen: BTreeSet<MomentId>,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            // A fresh account has no head start, and the multiplier is a
            // product: its identity is 1, not 0.
            cumulative_prestige_mult: 1.0,
            best_peak: 0,
            relics: BTreeMap::new(),
            relic_currency: 0.0,
            heroes_unlocked: BTreeSet::new(),
            prestige_count: 0,
            seen: BTreeSet::new(),
        }
    }
}

/// Everything prestige clears.
///
/// **Adding a field here adds it to the prestige reset and to ticket 08's ledger
/// automatically** — that is the entire reason this is a struct rather than a
/// list of fields on the save. If you add one, add a line to [`Run::losses`];
/// there is a test that fails if you do not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Run {
    pub gold: f64,
    /// Peak floor **this run** — what prestige scores. The live position of the
    /// climb is deliberately absent; see the module docs.
    pub peak: Floor,
    /// Not the lock line: the count of locks bought. The line is derived, so the
    /// two cannot disagree.
    pub lock_level: u32,
    pub ranks: PerType<u32>,
    pub heroes: BTreeMap<String, HeroProgress>,
    /// Which hero is stationed, and on which floor.
    pub stationed: Option<(String, Floor)>,
    /// Ticket 07's high-water mark of rolling 60-second gold/sec, and the rate
    /// offline gold pays at. A high-water mark rather than a recent average, so
    /// it cannot be sampled at an unrepresentative moment.
    pub best_rate: f64,
}

impl Default for Run {
    fn default() -> Self {
        Self {
            gold: 0.0,
            peak: 1,
            lock_level: 0,
            ranks: PerType::splat(1),
            heroes: BTreeMap::new(),
            stationed: None,
            best_rate: 0.0,
        }
    }
}

impl Run {
    /// The highest locked floor. Climbers spawn here.
    pub fn lock_line(&self) -> Floor {
        10 * self.lock_level + if self.lock_level > 0 { 1 } else { 0 }
    }

    /// What prestige would destroy, named at its real value.
    ///
    /// Ticket 08 settled that the losses are itemised at their real values
    /// (`melee rank 56 → 1`) rather than softened into minutes-to-recover, and
    /// shown **before committing, every time**. This is that list, and it lives
    /// next to the fields it enumerates so the two cannot drift apart.
    pub fn losses(&self) -> Vec<(&'static str, String)> {
        let fresh = Run::default();
        let mut out = vec![
            ("gold", format!("{:.0} → 0", self.gold)),
            ("floor reached", format!("{} → {}", self.peak, fresh.peak)),
            ("locks", format!("{} → 0", self.lock_level)),
            ("melee rank", format!("{} → {}", self.ranks.melee, fresh.ranks.melee)),
            ("ranged rank", format!("{} → {}", self.ranks.ranged, fresh.ranks.ranged)),
            ("healer rank", format!("{} → {}", self.ranks.healer, fresh.ranks.healer)),
        ];
        let fresh_hero = HeroProgress::default();
        for (hero, p) in &self.heroes {
            out.push((
                "hero level",
                format!(
                    "{hero} level {} → {} ({:.0} experience → 0)",
                    p.level, fresh_hero.level, p.experience
                ),
            ));
        }
        if let Some((hero, floor)) = &self.stationed {
            out.push(("stationed", format!("{hero} on floor {floor} → unstationed")));
        }
        out.push(("best rate", format!("{:.0}/s → 0", self.best_rate)));
        out
    }
}

/// The file, envelope and all.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveGame {
    pub version: u32,
    /// Unix milliseconds at write. Ticket 01 confirmed `SystemTime` is the
    /// correct clock — `Instant` freezes while suspended — and this is that
    /// clock in a form a human can read in a pasted bug report.
    pub written_at_unix_millis: u64,
    pub account: Account,
    pub run: Run,
}

impl Default for SaveGame {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            written_at_unix_millis: 0,
            account: Account::default(),
            run: Run::default(),
        }
    }
}

/// Just enough of a save to decide what to do with it.
///
/// `serde` ignores unknown fields, so this parses any save well enough to
/// dispatch on — including one written by a newer build whose full shape this
/// binary cannot represent.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct VersionProbe {
    #[serde(default)]
    pub version: u32,
}

/// What load-time validation had to change, so the game can say which.
///
/// Ticket 12 separates two things that get conflated. On **cheating**: nothing —
/// plain text, no checksum, no obfuscation, no cheater flag. A readable save is
/// an asset for a solo dev. On **validation**: yes, but for robustness, because
/// the economy is `f64` and a single `NaN` from a hand-edited file propagates
/// through every multiplier in the game with no crash to point at.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Repairs(pub Vec<String>);

impl Repairs {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A save that could be parsed but not trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Untrustworthy(pub String);

impl std::fmt::Display for Untrustworthy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Untrustworthy {}

impl SaveGame {
    /// Validates ranges, clamping what is merely nonsensical and refusing what
    /// is not a number at all.
    ///
    /// The split follows ADR 0012: **no code path may retroactively reduce what
    /// the save records the player as having.** So a value that is absurdly
    /// large is accepted as-is — it is what the save says the player has — while
    /// a value below the floor of its own type is raised to that floor, which
    /// takes nothing away. A non-finite value is refused outright: it is not a
    /// quantity at all, and clamping it would invent one.
    pub fn validate(&mut self) -> Result<Repairs, Untrustworthy> {
        let mut nonfinite = Vec::new();
        for (name, v) in [
            ("account.cumulative_prestige_mult", self.account.cumulative_prestige_mult),
            ("account.relic_currency", self.account.relic_currency),
            ("run.gold", self.run.gold),
            ("run.best_rate", self.run.best_rate),
        ] {
            if !v.is_finite() {
                nonfinite.push(format!("{name} is {v}"));
            }
        }
        for (hero, p) in &self.run.heroes {
            if !p.experience.is_finite() {
                nonfinite.push(format!("run.heroes.{hero}.experience is {}", p.experience));
            }
        }
        if !nonfinite.is_empty() {
            return Err(Untrustworthy(nonfinite.join("; ")));
        }

        let mut repairs = Repairs::default();
        let mut raise = |name: &str, v: &mut f64, floor: f64| {
            if *v < floor {
                repairs.0.push(format!("{name} was {v}, raised to {floor}"));
                *v = floor;
            }
        };
        // A multiplier below 1 would make prestige *subtract* permanent power.
        raise("cumulative prestige multiplier", &mut self.account.cumulative_prestige_mult, 1.0);
        raise("gold", &mut self.run.gold, 0.0);
        raise("best rate", &mut self.run.best_rate, 0.0);
        raise("relic currency", &mut self.account.relic_currency, 0.0);

        for ty in crate::types::ClimberType::ALL {
            if self.run.ranks[ty] < 1 {
                repairs.0.push(format!("{} rank was 0, raised to 1", ty.name()));
                self.run.ranks[ty] = 1;
            }
        }
        if self.run.peak < 1 {
            repairs.0.push(format!("peak floor was {}, raised to 1", self.run.peak));
            self.run.peak = 1;
        }
        // The account's best-ever peak cannot be behind this run's, or a relic
        // milestone already granted could be granted again.
        if self.account.best_peak < self.run.peak {
            self.account.best_peak = self.run.peak;
        }
        Ok(repairs)
    }

    /// What this run is worth as a single prestige. Shown to the player; the
    /// cumulative product never is.
    ///
    /// [ADR 0014](../../../docs/adr/0014-the-multiplier-pays-for-new-territory.md):
    /// this pays for **new territory only**, so it reads the account's
    /// deepest-ever floor and must be called *before* prestige folds this run's
    /// peak into it.
    pub fn earned_multiplier(&self, tuning: &Tuning) -> f64 {
        prestige_mult_for(tuning, self.new_territory(tuning))
    }

    /// The floors this run is paid for.
    pub fn new_territory(&self, tuning: &Tuning) -> Floor {
        match tuning.prestige_basis {
            crate::tuning::PrestigeBasis::BeyondBest => {
                self.run.peak.saturating_sub(self.account.best_peak)
            }
            crate::tuning::PrestigeBasis::Peak => self.run.peak,
        }
    }

    /// Ends the run, keeping the account. **This is the whole of prestige.**
    ///
    /// The set of things it resets is [`Run`], so there is no field list here to
    /// keep in step with anything.
    pub fn prestige(&mut self, tuning: &Tuning) -> Ledger {
        let earned = self.earned_multiplier(tuning);
        let ledger = Ledger {
            earned_multiplier: earned,
            losses: self.run.losses(),
            head_start_before: crate::curves::head_start_floors(
                tuning,
                self.account.cumulative_prestige_mult,
            ),
            head_start_after: crate::curves::head_start_floors(
                tuning,
                self.account.cumulative_prestige_mult * earned,
            ),
            first_time: !self.account.seen.contains(&MomentId::first_prestige()),
        };

        self.account.best_peak = self.account.best_peak.max(self.run.peak);
        self.account.cumulative_prestige_mult = crate::curves::compound_prestige(
            tuning,
            self.account.cumulative_prestige_mult,
            earned,
            self.account.best_peak,
        );
        self.account.prestige_count += 1;
        self.account.seen.insert(MomentId::first_prestige());
        self.run = Run::default();
        ledger
    }
}

/// What the game owes the player for time spent away.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OfflineGrant {
    pub gold: f64,
    /// Real time away, before the cap.
    pub elapsed_secs: f64,
    /// Time actually paid for.
    pub paid_secs: f64,
    pub capped: bool,
}

impl SaveGame {
    /// Ticket 07: **best rate × min(elapsed, cap)**, and gold only — the climb
    /// does not advance while away.
    ///
    /// Computed rather than simulated, which is what makes it affordable: an
    /// accelerated logic-only tick would be O(time × floors) and could not
    /// survive an overnight suspension.
    ///
    /// Elapsed time is clamped to `0..=cap` rather than trusted. Ticket 07
    /// accepted wall-clock manipulation explicitly — premium, single-player, no
    /// leaderboards — so this is not an anti-cheat measure; it is there so a
    /// clock that jumps backwards produces no gold instead of negative gold, and
    /// one that jumps forward by a century does not overflow the economy.
    pub fn offline_grant(&self, now_unix_millis: u64, tuning: &Tuning) -> OfflineGrant {
        let cap = tuning.offline_cap_hours * 3600.0;
        let elapsed = (now_unix_millis.saturating_sub(self.written_at_unix_millis)) as f64 / 1000.0;
        let paid = elapsed.clamp(0.0, cap);
        OfflineGrant {
            gold: self.run.best_rate * paid,
            elapsed_secs: elapsed,
            paid_secs: paid,
            capped: elapsed > cap,
        }
    }

    /// Applies the grant. Returns what was granted so the caller can show it —
    /// ticket 07 puts the report in ticket 09's strip, thresholded on duration
    /// and never on amount, so a two-minute absence says nothing.
    pub fn apply_offline_grant(&mut self, now_unix_millis: u64, tuning: &Tuning) -> OfflineGrant {
        let grant = self.offline_grant(now_unix_millis, tuning);
        self.run.gold += grant.gold;
        grant
    }
}

/// Ticket 08's before/after, shown before committing, every time.
#[derive(Debug, Clone, PartialEq)]
pub struct Ledger {
    /// The small, legible number: ×5.18 at floor 147, ×38.6 at floor 1,000.
    pub earned_multiplier: f64,
    pub losses: Vec<(&'static str, String)>,
    /// Head start in floors, which is the **only** form accumulated prestige
    /// takes on screen (ADR 0007). Its before-value is last prestige's result,
    /// so nothing needs remembering between screens.
    pub head_start_before: f64,
    pub head_start_after: f64,
    /// Whether the one-time explainer precedes this ledger.
    pub first_time: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prestige_is_the_type_and_nothing_else() {
        let mut s = SaveGame::default();
        s.run.gold = 9_000.0;
        s.run.peak = 412;
        s.run.lock_level = 30;
        s.run.ranks = PerType::new(56, 54, 51);
        s.run.best_rate = 1234.0;
        s.account.relic_currency = 7.0;

        let before = s.account.cumulative_prestige_mult;
        s.prestige(&Tuning::default());

        assert_eq!(s.run, Run::default(), "prestige is `run = Run::default()`");
        assert!(s.account.cumulative_prestige_mult > before);
        assert_eq!(s.account.best_peak, 412, "the account keeps the peak it scored");
        assert_eq!(s.account.relic_currency, 7.0, "and everything else it owns");
        assert_eq!(s.account.prestige_count, 1);
    }

    /// Ticket 12's structural claim: the ledger enumerates precisely the losses
    /// in `Run`, generated from one declaration so the two cannot drift. Without
    /// a derive macro that is a promise rather than a guarantee — so this is the
    /// guarantee. Add a field to `Run` and this fails until you have visited
    /// [`Run::losses`] and decided what the player is told they are losing.
    #[test]
    fn the_ledger_names_every_field_prestige_destroys() {
        let run = Run {
            gold: 1.0,
            peak: 2,
            lock_level: 3,
            ranks: PerType::new(4, 5, 6),
            heroes: BTreeMap::from([("tank".into(), HeroProgress { level: 7, experience: 8.0 })]),
            stationed: Some(("tank".into(), 9)),
            best_rate: 10.0,
        };
        let text = ron::ser::to_string_pretty(&run, ron::ser::PrettyConfig::default()).unwrap();

        // Top-level fields only — one indent level in. Nested keys belong to a
        // field that is already accounted for.
        let fields: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("    ") && !l.starts_with("     "))
            .filter_map(|l| l.trim().split_once(':'))
            .map(|(k, _)| k.trim())
            .collect();

        assert_eq!(
            fields.len(),
            7,
            "`Run` now serialises {} fields ({fields:?}). `Run::losses` was written against 7. \
             Add the new one to the ledger and update this count.",
            fields.len()
        );

        // And the ledger is not empty for any of them at these values.
        let losses = run.losses();
        for probe in ["gold", "floor", "lock", "melee", "ranged", "healer", "hero", "stationed", "rate"] {
            assert!(
                losses.iter().any(|(label, _)| label.contains(probe)),
                "the ledger says nothing about {probe}: {losses:?}"
            );
        }
    }

    #[test]
    fn offline_gold_pays_the_best_rate_up_to_the_cap() {
        let t = Tuning::default();
        let mut s = SaveGame::default();
        s.run.best_rate = 100.0;
        s.written_at_unix_millis = 1_000_000;

        // An hour away.
        let g = s.offline_grant(1_000_000 + 3_600_000, &t);
        assert_eq!(g.gold, 360_000.0);
        assert!(!g.capped);

        // A week away pays twelve hours.
        let g = s.offline_grant(1_000_000 + 7 * 24 * 3_600_000, &t);
        assert_eq!(g.gold, 100.0 * 12.0 * 3600.0);
        assert!(g.capped);
    }

    #[test]
    fn a_clock_that_jumps_backwards_pays_nothing_rather_than_negative() {
        let t = Tuning::default();
        let mut s = SaveGame::default();
        s.run.best_rate = 100.0;
        s.written_at_unix_millis = 5_000_000;
        let g = s.offline_grant(1_000, &t);
        assert_eq!(g.gold, 0.0);
        assert_eq!(g.paid_secs, 0.0);
    }

    /// ADR 0014's own guarantee: a fresh account is identical under either
    /// basis, so every early-game number from tickets 06 and 08 stands.
    #[test]
    fn a_fresh_account_is_unaffected_by_the_new_basis() {
        let mut s = SaveGame::default();
        s.run.peak = 147;
        let beyond = s.earned_multiplier(&Tuning::default());
        let peak = s.earned_multiplier(&Tuning {
            prestige_basis: crate::tuning::PrestigeBasis::Peak,
            ..Default::default()
        });
        assert_eq!(beyond, peak);
        // Ticket 08's measured value at floor 147, still standing.
        assert!((beyond - 5.18).abs() < 0.02, "{beyond}");
    }

    /// The structural kill. Ticket 17 measured the old rule compounding to
    /// 4.2e110 in twelve hours of five-minute cycles that fought nothing new;
    /// re-conquered ground must be worth exactly nothing.
    #[test]
    fn re_conquering_old_ground_earns_nothing() {
        let t = Tuning::default();
        let mut s = SaveGame::default();
        s.run.peak = 400;
        s.prestige(&t);
        let after_first = s.account.cumulative_prestige_mult;
        assert!(after_first > 1.0);

        // Ten more runs that never pass the record.
        for _ in 0..10 {
            s.run.peak = 399;
            let ledger = s.prestige(&t);
            assert_eq!(ledger.earned_multiplier, 1.0, "a run inside the record paid out");
        }
        assert_eq!(s.account.cumulative_prestige_mult, after_first, "prestige spam compounded");
        assert_eq!(s.account.best_peak, 400, "and the record did not move");
    }

    #[test]
    fn a_nan_is_refused_rather_than_clamped() {
        let mut s = SaveGame { run: Run { gold: f64::NAN, ..Run::default() }, ..Default::default() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn validation_raises_but_never_reduces() {
        // ADR 0012: no code path may retroactively reduce what the save records
        // the player as having. An implausibly rich save is still the player's.
        let mut s = SaveGame::default();
        s.run.gold = 1e300;
        s.account.cumulative_prestige_mult = 0.5;
        let repairs = s.validate().unwrap();
        assert_eq!(s.run.gold, 1e300, "a large value is what the save says they have");
        assert_eq!(s.account.cumulative_prestige_mult, 1.0);
        assert!(!repairs.is_empty());
    }

    #[test]
    fn unknown_relics_survive_a_round_trip() {
        // ADR 0009 makes the catalogue churn by design, so a save will name
        // relics this build has never heard of. Dropping them is irreversible.
        let mut s = SaveGame::default();
        s.account.relics.insert("a-relic-from-a-later-patch".into(), 4);
        let text = ron::ser::to_string_pretty(&s, ron::ser::PrettyConfig::default()).unwrap();
        let back: SaveGame = ron::from_str(&text).unwrap();
        assert_eq!(back.account.relics["a-relic-from-a-later-patch"], 4);
    }

    #[test]
    fn a_partial_save_fills_in_from_defaults() {
        // Additive fields need no version bump; `#[serde(default)]` covers them.
        let s: SaveGame = ron::from_str("(version: 1)").unwrap();
        assert_eq!(s.run, Run::default());
        assert_eq!(s.account.cumulative_prestige_mult, 1.0);
    }

    #[test]
    fn a_newer_save_still_reveals_its_version() {
        let probe: VersionProbe =
            ron::from_str("(version: 99, account: (some_field_from_the_future: 3))").unwrap();
        assert_eq!(probe.version, 99);
    }
}
