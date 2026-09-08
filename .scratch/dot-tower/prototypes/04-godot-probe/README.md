# Godot probe harness — ticket 04's probes, rebuilt

The Godot counterpart of the Bevy bringup harness in `src/lib.rs`, carrying the
same probes A–F. It exists to answer one question that reading cannot settle:
**how much of what is currently hard about this project is Bevy, and how much is
Android?**

Built against Godot **4.7.2**. Nothing here is game code.

## Running it

```bash
./scripts/run.sh              # interactive: drag the sprite, hold keys 1-4
./scripts/run.sh selftest     # probe B's automated verdict, prints and exits
./scripts/run.sh shot         # writes build/probe-desktop.png and exits
```

Keys `1`–`4` inject synthetic fingers onto the ability buttons, so multi-touch
is testable without a touchscreen. See probe B.

## The probes

| | Question | Bevy issue it mirrors |
|---|---|---|
| A | Does input arrive per-finger, at the coordinates we expect? | #7528 |
| B | Do four buttons register simultaneous presses? | #11553 |
| C | Does the safe area account for display cutouts? | #23003 |
| D | What scale yields crisp pixel text at ~2.625× density? | — |
| E | Does a write inside the suspend hook complete? | — |
| F | Does the UI layer render at all, and without flicker? | #14710 |

## Findings — desktop, session of 2026-09-07

### Probe B is settled, and it is the one that matters

Probe B looked like it needed hardware and does not. *Does the widget layer
aggregate pointers the transport kept apart* is a question about engine code,
not about a touchscreen: synthetic `InputEventScreenTouch`es pushed through
`Input.parse_input_event` travel the same viewport GUI path a real finger does.
`./scripts/run.sh selftest` presses all four abilities, releases exactly one,
and checks the held set by **membership, not count** — because "3 are held" does
not distinguish correct behaviour from the wrong three.

Both arms were measured: Godot's `Button` (a Control, the direct `bevy_ui`
analogue) and `TouchScreenButton` (a Node2D, Godot's documented answer for
on-screen action buttons).

**Result: Godot's Controls do multi-touch correctly — but only with a default
turned off.**

| `emulate_mouse_from_touch` | 4 down | release #2 | verdict |
|---|---|---|---|
| `true` (Godot's default) | `1234` | **`34`** — button 1 released itself | intermittent, 1 bad run in 5 |
| `false` | `1234` | `134` | 5/5 correct, both arms |

That bad row is *exactly* Bevy's #11553 shape: one finger lifting releases
another finger's button. Godot emulates a mouse from the first touch only, and a
mouse has one pointer, so with the default on, a four-finger press can reach the
widget layer as one cursor. It is **intermittent**, which is worse than
deterministic — it is the kind of thing that ships.

The difference that matters for the engine question is not the bug, it is the
fix. In Bevy this lives in `ui_focus_system`, has an open upstream issue, and no
user-side remedy — which is why ticket 02 banned the `Interaction` path outright.
In Godot it is one line in `project.godot`. `TouchScreenButton` was correct
either way, so there is also a second, independent route to the ability bar.

**Consequence for ticket 02**: its `Interaction` ban has no Godot equivalent
that costs anything. The four ability buttons are not a reason to prefer either
engine, and Godot does not need a picking-observer workaround to have them.

### Probe D: the arithmetic works, the eye test is still owed

`13px × screen_scale 2.0 × content_scale 1.0 = 26 physical px` on this Mac, and
the `scale -` / `scale +` buttons walk `content_scale_factor` live. Font
smoothing is off via `antialiasing`, `subpixel_positioning` and `hinting` on a
duplicated fallback font — the `FontSmoothing::None` analogue. Whether the
result *looks* crisp at 2.625× is still a question only a phone screen answers.

### Probe E: the hook fires, but Godot cannot prove durability

`APPLICATION_FOCUS_OUT` and `WM_CLOSE_REQUEST` arrive on desktop and the write
completes in **0.13–0.21 ms**. Every notification that arrives is logged, because
the Bevy run's real finding was about which ones never do.

**One place Godot is genuinely weaker.** `FileAccess.flush()` is a userspace
flush, not an `fsync`. The Bevy harness called `sync_all()` and could state that
the bytes reached the disk; this cannot. For [ADR 0012](../../../../docs/adr/0012-the-save-is-authoritative.md)
— *no code path may retroactively reduce what the save says the player has* —
that gap is real and would need closing, most likely by writing the save through
a small platform call rather than through `FileAccess`.

### Probe F: renders clean here, and that proves nothing

The HUD, the safe-area frame, both button rows and the sprite all render
correctly on macOS / Metal / Forward+. So did Bevy. Ticket 04's whole point is
that macOS was clean and the Pixel was not. `build/probe-desktop.png` is the
reference image to hold a device photo against — it is not evidence.

## What desktop cannot settle

- **Probe C.** A desktop window is not the screen, so there are no real insets.
  The frame falls back to a stand-in and says so. Godot does split what Bevy's
  `content_rect()` conflates — `get_display_safe_area()` and
  `get_display_cutouts()` are separate calls — so the harness reads both, and
  ticket 02's cutout question should get a cleaner answer than Bevy could give.
- **Probe E on Android.** Whether `APPLICATION_PAUSED` arrives early enough is
  the actual claim; desktop focus-out is a different event on a different path.
- **Probe F, which is the reason any of this exists.** Undecided.

## Finishing it on hardware

Blocked on one thing: Godot's **export templates**, a ~1 GB download this
machine does not have. Everything else is already installed from the Bevy
bringup — SDK build-tools 36, platform android-36, and `~/.android/debug.keystore`.

```bash
./scripts/android-export.sh   # errors with the exact download command if templates are missing
```

That script also writes the SDK/JDK/keystore paths into Godot's editor settings,
which a CLI export reads from there rather than from the environment — a failure
mode with an unhelpful error message.

## What this harness does not prove

- **targetSdk.** `gradle_build/use_gradle_build=false` uses Godot's prebuilt
  APK template, which is what makes the export one command with no Gradle in the
  loop. It also means targetSdk is whatever the template ships. Fine for a probe;
  to be revisited before any Play Store upload, which requires API 36 from
  2026-08-31.
- **Anything about the game.** No simulation, no ECS question, no performance
  budget. Ticket 14's ~1,100-entity model is untouched, and Godot has no ECS —
  that trade is argued nowhere in this directory and should not be inferred from
  a probe passing.
- **The renderer comparison.** Every reading here is Metal. The argument that
  Godot's Compatibility (GLES3) renderer is a fallback Bevy structurally lacks
  is untested; `renderer/rendering_method.mobile` in `project.godot` is where it
  would be switched.
