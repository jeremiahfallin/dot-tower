//! The column's placeholder art, generated rather than loaded.
//!
//! Every sprite here is the one the ticket 21 study rendered, transcribed from
//! `.scratch/dot-tower/prototypes/21-types/index.html` — the same 16x16 grids
//! and the same palette, because
//! [ADR 0016](../docs/adr/0016-type-reads-as-shape-hue-and-one-tag.md) settled
//! type legibility *on those pixels* and re-drawing them here would quietly
//! re-open a resolved question.
//!
//! They are built into textures at startup rather than shipped as files. The
//! map's standing preference is placeholder primitives until the loop is
//! proven, and generating them keeps the APK's asset pipeline — a separate
//! concern ticket 04 deliberately did not test — out of the game loop's way.
//!
//! The healer's cross is **above** the body, in four reserved rows every type
//! shares. ADR 0016 calls the tag sprite anatomy rather than column furniture,
//! and reserving the rows for all three is what makes that literally true: it
//! is in the texture, not a node hung beside one.

use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use dot_tower_sim::ClimberType;

/// Authored sprite width, in art pixels (ADR 0016).
pub const ART_W: u32 = 16;
/// Authored sprite height, plus the four rows the healer's tag lives in.
pub const ART_H: u32 = 20;
/// Where the 16x16 body starts inside that.
const BODY_TOP: u32 = 4;

/// Floor height in logical pixels. ADR 0016 fixes this at 48, which is what
/// puts 13 floors in ticket 10's 12-14 floor window.
pub const FLOOR_H: f32 = 48.0;
/// Floors drawn at once. Fixed by [`FLOOR_H`]; see ticket 10.
pub const FLOORS_IN_VIEW: u32 = 13;
/// The column is the same width on a phone and on a monitor (ticket 10).
pub const COLUMN_W: f32 = 430.0;

/// Art pixels to logical pixels for a climber, and for the hero.
pub const CLIMBER_SCALE: f32 = 2.0;
pub const HERO_SCALE: f32 = 3.0;

// --------------------------------------------------------------------------
// The palette
// --------------------------------------------------------------------------

/// Colour-blind safety here is by construction: shape carries type redundantly
/// with hue (ADR 0016), and the study's five colour-vision simulations stay in
/// the tree as the check the art pass re-runs. Changing a hue without re-running
/// them is the one thing this table asks you not to do.
const INK: [u8; 4] = rgb(0x14, 0x16, 0x28);

const fn rgb(r: u8, g: u8, b: u8) -> [u8; 4] {
    [r, g, b, 255]
}

/// Fill, secondary, and white/accent for one type, plus the tag colour.
struct Ramp {
    fill: [u8; 4],
    second: [u8; 4],
    white: [u8; 4],
    tag: [u8; 4],
    hat: [u8; 4],
}

const MELEE_RAMP: Ramp = Ramp {
    fill: rgb(0x29, 0xad, 0xf7),
    second: rgb(0x8b, 0x9b, 0xb4),
    white: rgb(0xdf, 0xf3, 0xff),
    tag: INK,
    hat: INK,
};
const RANGED_RAMP: Ramp = Ramp {
    fill: rgb(0xff, 0x8a, 0x3c),
    second: rgb(0xb2, 0x5b, 0x1e),
    white: rgb(0xff, 0xe3, 0xc2),
    tag: INK,
    hat: INK,
};
const HEALER_RAMP: Ramp = Ramp {
    fill: rgb(0xff, 0xf1, 0xe8),
    second: rgb(0xff, 0xcd, 0x75),
    white: rgb(0xff, 0xff, 0xff),
    tag: rgb(0xff, 0xcd, 0x75),
    hat: rgb(0x3a, 0x3f, 0x7a),
};
/// The hero is the study's neutral body in gold, at 3x. It is a placeholder:
/// ticket 16 named the tank but nothing has authored it, so this only has to be
/// unmistakably *not a climber* — which it is by size and hue at once.
const HERO_RAMP: Ramp = Ramp {
    fill: rgb(0xff, 0xd7, 0x6e),
    second: rgb(0xb8, 0x8f, 0x2e),
    white: rgb(0xff, 0xf3, 0xcc),
    tag: INK,
    hat: INK,
};

// --------------------------------------------------------------------------
// The grids
// --------------------------------------------------------------------------

/// `X` ink, `F` fill, `S` secondary, `W` white, `T` tag, `H` hat, `.` empty.
///
/// melee: square shield slab left, squat body.
const MELEE: [&str; 16] = [
    "................",
    "................",
    "......XXXX......",
    ".....XFFFFX.....",
    "....XFFFFFFX....",
    "....XFWWFFFX....",
    "..SSXFFFFFFX....",
    ".SSSXFFFFFFX....",
    ".SSSXFFFFFFX....",
    ".SSSXFFFFFFX....",
    ".SSS.XFFFFFX....",
    ".SSS.XXXXXXX....",
    ".SSS..X...X.....",
    "..S..XX...XX....",
    "................",
    "................",
];

/// ranged: tall thin body, bow string right. Verification flagged that this
/// reads as a pillar rather than a figure at column scale — kept as authored,
/// because ADR 0016 made it the art pass's first target rather than something
/// to quietly redraw here.
const RANGED: [&str; 16] = [
    "................",
    ".......XX....Y..",
    "......XFFX..Y...",
    "......XFFX..Y...",
    "......XWWX..Y...",
    "......XFFX..Y...",
    "......XFFXXYY...",
    "......XFFX..Y...",
    "......XFFX..Y...",
    "......XFFX..Y...",
    "......XFFX..Y...",
    "......X..X..Y...",
    ".....XX..XX.....",
    "................",
    "................",
    "................",
];

/// healer: brimmed hat, gold staff. The cross above the head is [`TAG`].
const HEALER: [&str; 16] = [
    "...T.....TT.....",
    "...T.....TT.....",
    "...T...XXXX.....",
    "...T..XHHHHX....",
    "...T.XHHHHHHX...",
    "...T...XFFX.....",
    "...T..XFFFX.....",
    "...T..XFWFX.....",
    "...T..XFFFX.....",
    "...T.XFFFFFX....",
    "...T..XFFFX.....",
    "...T..XFFFX.....",
    "...T..X..X......",
    "..TTT.XX.XX.....",
    "................",
    "................",
];

/// The hero: broad, shielded, and a head taller than anything else on the row.
const HERO: [&str; 16] = [
    "................",
    ".....XXXXXX.....",
    "....XFFFFFFX....",
    "....XFWWWWFX....",
    "....XFFFFFFX....",
    "..SSXXXXXXXX....",
    ".SSSXFFFFFFX....",
    ".SSSXFFFFFFX....",
    ".SSSXFFWWFFX....",
    ".SSSXFFFFFFX....",
    ".SSSXFFFFFFX....",
    ".SSSXFFFFFFX....",
    ".SSS.XXXXXX.....",
    "..S..XX..XX.....",
    ".....XX..XX.....",
    "................",
];

/// The healer's tag, in the four rows above every body. A plus, and only the
/// healer draws it — ticket 15 measured healer presence as the one type worth
/// noticing from across the column.
const TAG: [&str; 4] = ["................", ".......TT.......", "......TTTT......", ".......TT......."];
const NO_TAG: [&str; 4] = ["................"; 4];

// --------------------------------------------------------------------------
// Rasterising
// --------------------------------------------------------------------------

fn rasterise(body: &[&str; 16], tag: &[&str; 4], ramp: &Ramp) -> Image {
    let mut data = vec![0u8; (ART_W * ART_H * 4) as usize];
    let mut put = |x: u32, y: u32, ch: u8| {
        let colour = match ch {
            b'.' => return,
            b'X' | b'Y' => INK,
            b'F' => ramp.fill,
            b'S' => ramp.second,
            b'W' => ramp.white,
            b'T' => ramp.tag,
            b'H' => ramp.hat,
            _ => INK,
        };
        let i = ((y * ART_W + x) * 4) as usize;
        data[i..i + 4].copy_from_slice(&colour);
    };

    for (row, line) in tag.iter().enumerate() {
        for (col, ch) in line.bytes().enumerate() {
            put(col as u32, row as u32, ch);
        }
    }
    for (row, line) in body.iter().enumerate() {
        for (col, ch) in line.bytes().enumerate() {
            put(col as u32, row as u32 + BODY_TOP, ch);
        }
    }

    Image::new(
        Extent3d { width: ART_W, height: ART_H, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        // The sprites are never read back and never resized, so the CPU copy
        // would be dead weight on a device with 4GB.
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// A 1x1 white pixel, tinted per node. Every flat rectangle on screen is this
/// image — `BackgroundColor` would do as well, but one handle keeps the wall
/// hatch, the aura and the lines on a single code path.
fn hatch() -> Image {
    // A diagonal stripe, tiled across the wall row (ticket 09's hatching).
    const N: u32 = 8;
    let mut data = vec![0u8; (N * N * 4) as usize];
    for y in 0..N {
        for x in 0..N {
            let on = (x + y) % N < 3;
            let i = ((y * N + x) * 4) as usize;
            data[i..i + 4].copy_from_slice(&if on {
                [0xe8, 0x5a, 0x5a, 0xff]
            } else {
                [0, 0, 0, 0]
            });
        }
    }
    Image::new(
        Extent3d { width: N, height: N, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Every generated texture, built once at startup.
#[derive(Resource)]
pub struct Art {
    pub melee: Handle<Image>,
    pub ranged: Handle<Image>,
    pub healer: Handle<Image>,
    pub hero: Handle<Image>,
    pub hatch: Handle<Image>,
}

impl Art {
    pub fn of(&self, ty: ClimberType) -> Handle<Image> {
        match ty {
            ClimberType::Melee => self.melee.clone(),
            ClimberType::Ranged => self.ranged.clone(),
            ClimberType::Healer => self.healer.clone(),
        }
    }
}

pub fn build_art(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    commands.insert_resource(Art {
        melee: images.add(rasterise(&MELEE, &NO_TAG, &MELEE_RAMP)),
        ranged: images.add(rasterise(&RANGED, &NO_TAG, &RANGED_RAMP)),
        healer: images.add(rasterise(&HEALER, &TAG, &HEALER_RAMP)),
        hero: images.add(rasterise(&HERO, &NO_TAG, &HERO_RAMP)),
        hatch: images.add(hatch()),
    });
}

// --------------------------------------------------------------------------
// Screen colours
// --------------------------------------------------------------------------

pub mod ui {
    use bevy::prelude::*;

    pub const BACKDROP: Color = Color::srgb(0.047, 0.051, 0.109);
    pub const FLOOR: Color = Color::srgb(0.055, 0.063, 0.133);
    pub const FLOOR_FIFTH: Color = Color::srgb(0.071, 0.078, 0.169);
    pub const FLOOR_EDGE: Color = Color::srgb(0.114, 0.125, 0.267);
    pub const PANEL: Color = Color::srgb(0.066, 0.071, 0.164);
    pub const CHIP: Color = Color::srgb(0.149, 0.165, 0.302);
    pub const CHIP_LIVE: Color = Color::srgb(0.204, 0.227, 0.416);
    pub const INK: Color = Color::srgb(0.910, 0.902, 0.941);
    pub const DIM: Color = Color::srgb(0.490, 0.518, 0.659);
    pub const GOLD: Color = Color::srgb(1.0, 0.843, 0.431);
    pub const LOCK: Color = Color::srgb(0.329, 0.376, 1.0);
    pub const LOCK_DIM: Color = Color::srgb(0.482, 0.522, 1.0);
    pub const WALL: Color = Color::srgb(0.910, 0.353, 0.353);
    // Blending happens in linear space, so a low alpha over a near-black
    // column lands much brighter than the number suggests: 0.16 reads as solid
    // brown, not as a lit floor.
    pub const AURA: Color = Color::srgba(1.0, 0.804, 0.459, 0.08);
    pub const MELEE: Color = Color::srgb(0.161, 0.678, 0.969);
    pub const RANGED: Color = Color::srgb(1.0, 0.541, 0.235);
    pub const HEALER: Color = Color::srgb(1.0, 0.945, 0.910);

    pub fn of_type(ty: dot_tower_sim::ClimberType) -> Color {
        match ty {
            dot_tower_sim::ClimberType::Melee => MELEE,
            dot_tower_sim::ClimberType::Ranged => RANGED,
            dot_tower_sim::ClimberType::Healer => HEALER,
        }
    }
}
