//! Ticket 12 spent most of its length on the failure paths, so most of the
//! tests are here. Each one is named for the decision it holds in place.

use dot_tower_save::*;
use dot_tower_sim::save::{SaveGame, SCHEMA_VERSION};
use dot_tower_sim::Tuning;
use std::path::PathBuf;

/// A scratch directory that cleans up after itself. Hand-rolled rather than
/// pulled in as a dependency — this crate's whole job is careful filesystem
/// handling, and a harness that hid the filesystem would be testing the wrong
/// thing.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("dot-tower-save-test-{name}-{}", now_unix_millis()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn a_save_at(peak: u32, gold: f64) -> SaveGame {
    let mut s = SaveGame::default();
    s.run.peak = peak;
    s.run.gold = gold;
    s.account.cumulative_prestige_mult = 250.0;
    s
}

#[test]
fn a_first_launch_says_nothing() {
    let dir = Scratch::new("first");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    let loaded = slot.load();
    assert!(matches!(loaded, Loaded::FreshStart));
    assert!(loaded.notice(&Tuning::default()).is_none(), "a first launch must not apologise");
}

#[test]
fn a_save_round_trips() {
    let dir = Scratch::new("round");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    slot.write(&a_save_at(412, 9000.0), Trigger::Prestige).unwrap();

    let loaded = slot.load();
    let save = loaded.save().expect("should have loaded");
    assert_eq!(save.run.peak, 412);
    assert_eq!(save.run.gold, 9000.0);
    assert!(save.written_at_unix_millis > 0, "the write stamps the clock offline gold reads");
    assert!(loaded.notice(&Tuning::default()).is_none(), "a clean load says nothing");
}

/// The finding that inverts the obvious implementation. A backup rotating on
/// every write is worthless against the mode that actually bites — a bug writing
/// a valid-but-wrong save — because autosave overwrites the good copy within
/// seconds.
#[test]
fn the_backup_rotates_on_load_not_on_write() {
    let dir = Scratch::new("rotate");
    let mut slot = SaveSlot::open(dir.path()).unwrap();

    slot.write(&a_save_at(400, 1.0), Trigger::Timer).unwrap();
    assert!(!slot.backup_path().exists(), "no backup yet: nothing has been loaded");

    slot.load();
    assert!(slot.backup_path().exists(), "loading rotates the backup");

    // Now a session runs and writes repeatedly. The backup must NOT follow it —
    // it is the state the last session ended in, not the state of five seconds
    // ago.
    for peak in [401, 402, 403] {
        slot.write(&a_save_at(peak, 1.0), Trigger::Timer).unwrap();
    }
    let backup: SaveGame = ron::from_str(&std::fs::read_to_string(slot.backup_path()).unwrap()).unwrap();
    assert_eq!(backup.run.peak, 400, "the backup followed the writes, which defeats its purpose");
}

#[test]
fn a_corrupt_save_is_quarantined_and_the_backup_is_restored() {
    let dir = Scratch::new("corrupt");
    let mut slot = SaveSlot::open(dir.path()).unwrap();

    slot.write(&a_save_at(412, 5.0), Trigger::Timer).unwrap();
    slot.load(); // rotates the backup
    std::fs::write(slot.primary_path(), "this is not RON at all {{{").unwrap();

    let loaded = slot.load();
    let Loaded::RestoredFromBackup { save, quarantined, .. } = &loaded else {
        panic!("expected a restore from backup, got {loaded:?}");
    };
    assert_eq!(save.run.peak, 412);
    assert!(quarantined.exists(), "the unreadable file is the only evidence a bug report will carry");
    assert!(quarantined.file_name().unwrap().to_string_lossy().contains("corrupt"));

    // And the player is told in the game's own terms, not in machinery.
    let notice = loaded.notice(&Tuning::default()).expect("must say something");
    assert!(notice.contains("floor 412"), "{notice}");
    assert!(notice.contains("head start"), "{notice}");
    assert!(!notice.to_lowercase().contains("parse"), "spoke in machinery: {notice}");
}

/// The hazard is not the refusal, it is what happens next: refuse, start a fresh
/// game, and the 60-second timer overwrites the player's real save within a
/// minute. Total, silent, permanent — and the default behaviour of every other
/// component in the crate.
#[test]
fn a_newer_save_disarms_every_write() {
    let dir = Scratch::new("newer");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    let future = SaveGame { version: SCHEMA_VERSION + 5, ..a_save_at(999, 1.0) };
    let text = ron::ser::to_string_pretty(&future, ron::ser::PrettyConfig::default()).unwrap();
    std::fs::write(slot.primary_path(), &text).unwrap();

    let loaded = slot.load();
    assert!(matches!(loaded, Loaded::RefusedNewerVersion { .. }));
    assert!(!slot.writes_armed());

    for trigger in [Trigger::FocusLost, Trigger::Suspended, Trigger::Timer, Trigger::Prestige] {
        assert!(
            matches!(slot.write(&SaveGame::default(), trigger), Err(Error::WritesDisarmed)),
            "{trigger:?} was still allowed to write over a newer save"
        );
    }
    assert_eq!(
        std::fs::read_to_string(slot.primary_path()).unwrap(),
        text,
        "the newer save was modified"
    );
    // And it must not have been rotated into the backup either.
    assert!(!slot.backup_path().exists());
}

#[test]
fn the_player_can_choose_to_start_over_after_a_newer_save() {
    let dir = Scratch::new("startover");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    let future = SaveGame { version: SCHEMA_VERSION + 1, ..Default::default() };
    std::fs::write(
        slot.primary_path(),
        ron::ser::to_string_pretty(&future, ron::ser::PrettyConfig::default()).unwrap(),
    )
    .unwrap();
    slot.load();

    slot.arm_writes_and_discard_newer_save();
    assert!(slot.write(&a_save_at(1, 0.0), Trigger::Prestige).is_ok());
}

/// Re-serialising would re-stamp `written_at` and erase the player's offline
/// entitlement on the one path where they are already having a bad day, and
/// would drop exactly the unknown relic ids ADR 0012 exists to preserve.
#[test]
fn the_backup_copy_is_byte_for_byte() {
    let dir = Scratch::new("bytes");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    let mut save = a_save_at(300, 2.0);
    save.account.relics.insert("relic-this-build-has-never-heard-of".into(), 3);
    slot.write(&save, Trigger::Timer).unwrap();

    let primary = std::fs::read(slot.primary_path()).unwrap();
    slot.load();
    let backup = std::fs::read(slot.backup_path()).unwrap();
    assert_eq!(primary, backup, "the backup was re-serialised rather than copied");
    assert!(String::from_utf8_lossy(&backup).contains("relic-this-build-has-never-heard-of"));
}

#[test]
fn a_missing_primary_behind_a_good_backup_is_not_a_first_launch() {
    let dir = Scratch::new("orphan");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    slot.write(&a_save_at(500, 3.0), Trigger::Timer).unwrap();
    slot.load();
    std::fs::remove_file(slot.primary_path()).unwrap();

    let loaded = slot.load();
    assert!(
        matches!(loaded, Loaded::RestoredFromBackup { .. }),
        "silently started a new game on top of a live backup: {loaded:?}"
    );
    assert_eq!(loaded.save().unwrap().run.peak, 500);
}

#[test]
fn a_save_full_of_nan_falls_back_rather_than_poisoning_the_economy() {
    let dir = Scratch::new("nan");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    slot.write(&a_save_at(600, 4.0), Trigger::Timer).unwrap();
    slot.load();

    // What a hand-edited file looks like when someone gets it wrong. One NaN
    // propagates through every f64 multiplier in the game with no crash to
    // point at, which is why this is refused rather than loaded.
    std::fs::write(slot.primary_path(), "(version: 1, run: (gold: NaN))").unwrap();
    let loaded = slot.load();
    assert!(matches!(loaded, Loaded::RestoredFromBackup { .. }), "{loaded:?}");
    assert_eq!(loaded.save().unwrap().run.peak, 600);
}

#[test]
fn settings_are_not_rewound_when_progress_is() {
    let dir = Scratch::new("settings");
    let mut slot = SaveSlot::open(dir.path()).unwrap();
    write_settings(dir.path(), &Settings::default()).unwrap();

    slot.write(&a_save_at(700, 5.0), Trigger::Timer).unwrap();
    slot.load();
    std::fs::write(slot.primary_path(), "garbage").unwrap();
    slot.load();

    // The settings file is untouched by any of that — it is not in the save and
    // not covered by the rotation.
    assert!(dir.path().join(SETTINGS_FILE).exists());
    assert_eq!(load_settings(dir.path()), Settings::default());
}

#[test]
fn unreadable_settings_reset_quietly_and_never_quarantine() {
    let dir = Scratch::new("badsettings");
    std::fs::write(dir.path().join(SETTINGS_FILE), "not ron").unwrap();
    assert_eq!(load_settings(dir.path()), Settings::default());
    let stray = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().contains("corrupt"));
    assert!(!stray, "preferences do not get the loud failure path");
}
