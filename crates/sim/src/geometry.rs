//! Where a unit is, in the only coordinate space the simulation admits.
//!
//! [Ticket 03](../../../.scratch/dot-tower/issues/03-simulation-boundary.md)
//! settled this and the reasoning is worth restating, because the whole point
//! is what the type makes *unrepresentable*: a climber rendering happily inside
//! a wall is a silent wrong value, and silent wrong values are the class of bug
//! this project exists to not ship. World-space coordinates are derived for
//! rendering and are never authoritative.

use serde::{Deserialize, Serialize};

/// A position across the width of a floor, clamped to `0.0..=1.0` on
/// construction so no code path can produce an out-of-bounds value — not
/// knockback, not a bad interpolation, not a `NaN`.
///
/// `f32` per ticket 03. The economy needs `f64` (ticket 06 measured it holding
/// to ~floor 2,000); a position across one floor does not.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(from = "f32", into = "f32")]
pub struct NormX(f32);

impl NormX {
    pub const START: Self = Self(0.0);
    pub const END: Self = Self(1.0);

    /// Clamps, and maps `NaN` to the start of the floor. `f32::clamp` panics on
    /// a `NaN` bound but *propagates* a `NaN` value, so the explicit test is
    /// load-bearing rather than defensive noise.
    #[inline]
    pub fn new(x: f32) -> Self {
        Self(if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) })
    }

    #[inline]
    pub fn get(self) -> f32 {
        self.0
    }

    /// Advances by `delta`, returning whether the far edge was reached. The
    /// caller decides what crossing means — climbers advance a floor, a
    /// knockback impulse simply stops at the wall.
    #[inline]
    pub fn advance(&mut self, delta: f32) -> bool {
        let raw = self.0 + delta;
        *self = Self::new(raw);
        raw >= 1.0
    }
}

impl From<f32> for NormX {
    fn from(x: f32) -> Self {
        Self::new(x)
    }
}

impl From<NormX> for f32 {
    fn from(x: NormX) -> Self {
        x.0
    }
}

/// A floor number, counted from 1 upward without limit.
pub type Floor = u32;

/// A position in the tower: which floor, and where across it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FloorPos {
    pub floor: Floor,
    pub x: NormX,
}

impl FloorPos {
    pub fn entering(floor: Floor) -> Self {
        Self { floor, x: NormX::START }
    }

    /// Walks across the current floor, stepping up when the far edge is
    /// reached. Returns the new floor if one was entered.
    pub fn walk(&mut self, delta: f32) -> Option<Floor> {
        if self.x.advance(delta) {
            self.x = NormX::START;
            self.floor += 1;
            Some(self.floor)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_range_is_unrepresentable() {
        assert_eq!(NormX::new(-5.0).get(), 0.0);
        assert_eq!(NormX::new(9.0).get(), 1.0);
        assert_eq!(NormX::new(f32::NAN).get(), 0.0);
        assert_eq!(NormX::new(f32::INFINITY).get(), 1.0);
        assert_eq!(NormX::new(f32::NEG_INFINITY).get(), 0.0);
    }

    #[test]
    fn a_deserialised_position_is_clamped_too() {
        // The save is authoritative (ADR 0012), but authoritative is not the
        // same as trusted: a corrupt file must not be able to smuggle a
        // position through the constructor's back.
        let x: NormX = ron::from_str("40.0").unwrap();
        assert_eq!(x.get(), 1.0);
    }

    #[test]
    fn walking_a_floor_at_the_shipped_speed_takes_twenty_ticks() {
        // climb_speed 0.5 floors/s at the 10Hz tick.
        let mut pos = FloorPos::entering(7);
        let mut ticks = 0;
        loop {
            ticks += 1;
            if let Some(f) = pos.walk(0.5 * 0.1) {
                assert_eq!(f, 8);
                break;
            }
            assert!(ticks < 100, "never crossed");
        }
        assert_eq!(ticks, 20);
        assert_eq!(pos.x.get(), 0.0);
    }
}
