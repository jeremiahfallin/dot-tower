//! Reading and writing the save, including every way it can go wrong.
//!
//! [Ticket 12](../../.scratch/dot-tower/issues/12-save-schema.md) spent most of
//! its length on the failure paths, and so does this crate. The happy path is a
//! serialise and a rename; the decisions are all in what happens when the file
//! is missing, corrupt, or from a build that does not exist yet.
//!
//! Three of those decisions are load-bearing and easy to undo by accident:
//!
//! - **The backup rotates on load, not on write.** Process death mid-write is
//!   already unobservable thanks to the atomic rename, so the backup is not for
//!   that. The mode that actually bites is a bug writing a *valid-but-wrong*
//!   save, and against that a backup rotating on every write is worthless —
//!   focus-loss autosave overwrites the good copy within seconds.
//! - **The copy is byte-for-byte, never a re-serialisation.** Re-serialising
//!   would re-stamp `written_at` and erase the player's offline entitlement on
//!   the one path where they are already having a bad day, and would drop
//!   exactly the unknown relic ids [ADR 0012] exists to preserve.
//! - **A save from a newer build disarms every write.** Refusing to load is the
//!   harmless half; the hazard is what happens next, because a fresh game plus
//!   a 60-second timer overwrites the player's real save within a minute. That
//!   is the default behaviour of every other component here, so the write lock
//!   is the decision and the refusal is only its precondition.
//!
//! This crate takes the directory as a parameter and never asks the platform
//! where it is. On Android that path comes from `bevy_android::ANDROID_APP`,
//! which would drag Bevy into the save layer; the game resolves it and passes
//! it in. [`default_desktop_dir`] covers the desktop case.
//!
//! [ADR 0012]: ../../docs/adr/0012-the-save-is-authoritative.md

use dot_tower_sim::curves::head_start_floors;
use dot_tower_sim::save::{Repairs, SaveGame, VersionProbe, SCHEMA_VERSION};
use dot_tower_sim::Tuning;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SAVE_FILE: &str = "save.ron";
pub const BACKUP_FILE: &str = "save.backup.ron";
pub const SETTINGS_FILE: &str = "settings.ron";

/// The floor under every other trigger, covering a desktop crash or an Android
/// low-memory kill from a state where focus was never lost and no lifecycle
/// event fired.
pub const TIMER_INTERVAL_SECS: f64 = 60.0;

/// When the game writes.
///
/// Ticket 01 killed the two-phase save — `WillSuspend` is never delivered — and
/// ticket 04 confirmed on hardware that a write plus `sync_all()` inside
/// `Suspended` completes in 2 ms. These four are the whole policy, and the
/// exclusions are as deliberate as the inclusions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// Primary. `WindowFocused { focused: false }`.
    FocusLost,
    /// Backstop. `AppLifecycle::Suspended`, delivered synchronously while
    /// Android's main thread is blocked in `surfaceDestroyed`.
    Suspended,
    /// Every [`TIMER_INTERVAL_SECS`].
    Timer,
    /// A commit point, not an optimisation: prestige is the only irreversible
    /// destructive act the player can perform, and the only write where a crash
    /// costs *correctness* — a half-applied prestige — rather than progress.
    Prestige,
    // Deliberately absent: on lock, and on rank purchase. Both fire every few
    // seconds and the timer covers them.
}

/// What happened when the save was read.
#[derive(Debug)]
pub enum Loaded {
    /// No save and no backup. A new game, and **nothing is said about it** —
    /// conflating this with corruption produces a first launch that apologises.
    FreshStart,
    Restored { save: Box<SaveGame>, repairs: Repairs },
    /// The primary was unreadable. This is the state the last session ended in.
    RestoredFromBackup { save: Box<SaveGame>, quarantined: PathBuf, repairs: Repairs },
    /// The primary was unreadable and there was no usable backup behind it.
    Lost { quarantined: PathBuf },
    /// Written by a build that does not exist yet. **Writes are disarmed.**
    RefusedNewerVersion { found: u32, known: u32 },
}

impl Loaded {
    /// The save to play, or `None` if the player is starting fresh.
    pub fn save(&self) -> Option<&SaveGame> {
        match self {
            Self::Restored { save, .. } | Self::RestoredFromBackup { save, .. } => Some(save),
            _ => None,
        }
    }

    /// What to tell the player, **in the game's own terms**.
    ///
    /// Ticket 12 is specific about this and it is the same principle as
    /// ADR 0007: "restored to your last session: floor 412, head start 108
    /// floors", not "save file corrupted". A save that quietly rewinds someone
    /// is the purest instance of the failure this whole project exists against.
    pub fn notice(&self, tuning: &Tuning) -> Option<String> {
        match self {
            // A first launch says nothing at all. Conflating this with
            // corruption produces a game that apologises for a save that never
            // existed.
            Self::FreshStart => None,
            Self::Restored { repairs, .. } if repairs.is_empty() => None,
            Self::Restored { repairs, .. } => Some(format!(
                "Your save needed repairing before it could be loaded: {}.",
                repairs.0.join("; ")
            )),
            Self::RestoredFromBackup { save, .. } => Some(format!(
                "Your save could not be read, so you have been restored to your last session: \
                 floor {}, head start {:.0} floors.",
                save.run.peak,
                head_start_floors(tuning, save.account.cumulative_prestige_mult),
            )),
            Self::Lost { .. } => Some(
                "Your save could not be read and there was no backup behind it, so this is a new \
                 tower. The unreadable file has been kept."
                    .to_string(),
            ),
            Self::RefusedNewerVersion { found, known } => Some(format!(
                "This save was written by a newer version of the game (save {found}, this build \
                 reads {known}). It has not been loaded and nothing will be written over it. \
                 Update the game to continue."
            )),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    /// The save is from a newer build, so writing would destroy it.
    WritesDisarmed,
    Io(std::io::Error),
    Serialise(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WritesDisarmed => f.write_str(
                "writes are disarmed because the save on disk is from a newer version of the game",
            ),
            Self::Io(e) => write!(f, "{e}"),
            Self::Serialise(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// One save directory, and whether it may be written to.
pub struct SaveSlot {
    dir: PathBuf,
    writes_armed: bool,
}

impl SaveSlot {
    /// Opens a slot. Creates the directory if it does not exist.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, Error> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir, writes_armed: true })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn primary_path(&self) -> PathBuf {
        self.dir.join(SAVE_FILE)
    }
    pub fn backup_path(&self) -> PathBuf {
        self.dir.join(BACKUP_FILE)
    }
    /// False once a newer-version save has been seen. Nothing re-arms it except
    /// the player explicitly choosing to start over.
    pub fn writes_armed(&self) -> bool {
        self.writes_armed
    }

    /// Deliberate, player-initiated "start a new tower" — the only thing that
    /// re-arms writes after a newer-version save has locked them.
    pub fn arm_writes_and_discard_newer_save(&mut self) {
        self.writes_armed = true;
    }

    /// Reads the save, rotating the backup on success.
    pub fn load(&mut self) -> Loaded {
        let primary = self.primary_path();
        let bytes = match std::fs::read(&primary) {
            Ok(b) => b,
            // First launch and corruption are two code paths from the first
            // line. Branching here, before any parse, is what keeps a first
            // launch from apologising and a corrupt save from silently starting
            // a new game and then autosaving over the backup.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return match self.read_and_parse(&self.backup_path()) {
                    // A primary that has gone missing while a backup survives is
                    // not a first launch, whatever the missing file suggests.
                    Some(Ok((save, repairs))) => Loaded::RestoredFromBackup {
                        save: Box::new(save),
                        quarantined: primary,
                        repairs,
                    },
                    _ => Loaded::FreshStart,
                };
            }
            Err(_) => {
                // Present but unreadable for some other reason — permissions, a
                // bad mount. Treated as corruption rather than as a fresh start,
                // and writes are disarmed, because the one thing that must not
                // happen is overwriting a save that is probably still intact.
                self.writes_armed = false;
                return match self.read_and_parse(&self.backup_path()) {
                    Some(Ok((save, repairs))) => Loaded::RestoredFromBackup {
                        save: Box::new(save),
                        quarantined: primary,
                        repairs,
                    },
                    _ => Loaded::Lost { quarantined: primary },
                };
            }
        };

        let text = String::from_utf8_lossy(&bytes).into_owned();

        // Version before parse: a newer save may contain shapes this binary
        // cannot represent, and `serde` ignoring unknown fields means the full
        // parse would *succeed* and silently drop them.
        if let Ok(probe) = ron::from_str::<VersionProbe>(&text)
            && probe.version > SCHEMA_VERSION
        {
            self.writes_armed = false;
            return Loaded::RefusedNewerVersion { found: probe.version, known: SCHEMA_VERSION };
        }

        match parse(&text) {
            Ok((save, repairs)) => {
                // Rotate the backup only now, and only from the bytes that were
                // just proven to parse.
                let _ = std::fs::copy(&primary, self.backup_path());
                Loaded::Restored { save: Box::new(save), repairs }
            }
            Err(_) => {
                let quarantined = self.quarantine(&primary);
                match self.read_and_parse(&self.backup_path()) {
                    Some(Ok((save, repairs))) => Loaded::RestoredFromBackup {
                        save: Box::new(save),
                        quarantined,
                        repairs,
                    },
                    _ => Loaded::Lost { quarantined },
                }
            }
        }
    }

    fn read_and_parse(&self, path: &Path) -> Option<Result<(SaveGame, Repairs), String>> {
        let text = std::fs::read_to_string(path).ok()?;
        if let Ok(probe) = ron::from_str::<VersionProbe>(&text)
            && probe.version > SCHEMA_VERSION
        {
            return Some(Err("backup is from a newer version".into()));
        }
        Some(parse(&text))
    }

    /// Moves an unreadable save aside. **Never deleted** — it is the only
    /// evidence a solo developer will get from a bug report.
    fn quarantine(&self, path: &Path) -> PathBuf {
        let dest = self.dir.join(format!("{SAVE_FILE}.corrupt-{}", now_unix_millis()));
        // Rename rather than copy, so the next load does not walk into the same
        // failure and the game can recover on its own.
        if std::fs::rename(path, &dest).is_err() {
            let _ = std::fs::copy(path, &dest);
            let _ = std::fs::remove_file(path);
        }
        dest
    }

    /// Writes the save. Temp file, `sync_all()`, atomic rename — the ritual is
    /// not a choice, so it is recorded rather than decided.
    pub fn write(&self, save: &SaveGame, _trigger: Trigger) -> Result<(), Error> {
        if !self.writes_armed {
            return Err(Error::WritesDisarmed);
        }

        let mut stamped = save.clone();
        stamped.version = SCHEMA_VERSION;
        stamped.written_at_unix_millis = now_unix_millis();

        let text = ron::ser::to_string_pretty(&stamped, ron::ser::PrettyConfig::default())
            .map_err(|e| Error::Serialise(e.to_string()))?;

        let tmp = self.dir.join(format!("{SAVE_FILE}.tmp"));
        {
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(text.as_bytes())?;
            // Without this the bytes may only be in the page cache when the
            // process is killed. Ticket 04 measured the whole write at 2 ms on
            // the bringup handset, inside `Suspended`'s one-frame budget.
            file.sync_all()?;
        }
        std::fs::rename(&tmp, self.primary_path())?;

        // Best effort, and genuinely optional: the rename is already atomic, and
        // this only narrows the window in which the *directory entry* is not yet
        // durable. Windows cannot open a directory for sync at all.
        if let Ok(dir) = std::fs::File::open(&self.dir) {
            let _ = dir.sync_all();
        }
        Ok(())
    }
}

fn parse(text: &str) -> Result<(SaveGame, Repairs), String> {
    let mut save: SaveGame = ron::from_str(text).map_err(|e| e.to_string())?;
    let repairs = save.validate().map_err(|e| e.to_string())?;
    Ok((save, repairs))
}

pub fn now_unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The platform config directory, plus the game's own folder.
///
/// Android is deliberately not handled here — its path comes from
/// `AndroidApp::internal_data_path()`, which lives behind Bevy.
pub fn default_desktop_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let dir = if cfg!(target_os = "macos") {
        home?.join("Library/Application Support")
    } else if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)?
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|h| h.join(".config")))?
    };
    Some(dir.join("dot-tower"))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// Player preferences, in their **own file**.
///
/// Ticket 12 gives three reasons and the third is the real one: settings are
/// untouched by prestige; they can safely reset on a parse failure, so they need
/// no migration chain; and the rotate-on-load backup would otherwise rewind the
/// player's preferences every time it restored their progress. Restoring
/// progress and restoring preferences are different operations and must not be
/// welded together.
///
/// What actually goes in here is open — it overlaps audio and accessibility and
/// waits on the UI being real, which is why the struct is empty rather than
/// speculative. `#[serde(default)]` means adding a field later costs nothing and
/// needs no version.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {}

/// Reads settings, falling back to defaults on **any** failure.
///
/// Deliberately infallible. There is no version field and no migration chain: a
/// preference that cannot be read is worth exactly one shrug, and the loud
/// failure path belongs to progress, not to preferences.
pub fn load_settings(dir: &Path) -> Settings {
    std::fs::read_to_string(dir.join(SETTINGS_FILE))
        .ok()
        .and_then(|t| ron::from_str(&t).ok())
        .unwrap_or_default()
}

/// Writes settings. Uses the same durable ritual as the save — a preferences
/// file torn in half by a power cut is a worse experience than a missing one,
/// and the cost is a few microseconds on a file written when a menu closes.
///
/// Not gated on [`SaveSlot::writes_armed`]: the write lock exists to protect a
/// newer *save* from an older build, and preferences carry no progress.
pub fn write_settings(dir: &Path, settings: &Settings) -> Result<(), Error> {
    let text = ron::ser::to_string_pretty(settings, ron::ser::PrettyConfig::default())
        .map_err(|e| Error::Serialise(e.to_string()))?;
    let tmp = dir.join(format!("{SETTINGS_FILE}.tmp"));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, dir.join(SETTINGS_FILE))?;
    Ok(())
}
