//! The closed set of climber types, and the container that keeps it closed.
//!
//! Three types exist and the set does not grow at runtime: `CONTEXT.md` makes
//! **climber type** the thing identity, rank and relic affinity attach to, and
//! [ADR 0011](../../../docs/adr/0011-composition-is-authored.md) makes the mix
//! between them authored rather than chosen. Modelling that as a map keyed by
//! string — which the throwaway JS model did — allows a fourth type to appear
//! by typo, and allows a lookup to fail. Neither is representable here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClimberType {
    Melee,
    Ranged,
    Healer,
}

impl ClimberType {
    /// Iteration order for everything that must be reproducible. Fixed
    /// deliberately: several tie-breaks in the tick resolve on "first wins".
    pub const ALL: [ClimberType; 3] = [Self::Melee, Self::Ranged, Self::Healer];

    pub fn name(self) -> &'static str {
        match self {
            Self::Melee => "melee",
            Self::Ranged => "ranged",
            Self::Healer => "healer",
        }
    }
}

/// One value per climber type. Total by construction — there is no missing key
/// and no `Option` to unwrap at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PerType<T> {
    pub melee: T,
    pub ranged: T,
    pub healer: T,
}

impl<T> PerType<T> {
    pub fn new(melee: T, ranged: T, healer: T) -> Self {
        Self { melee, ranged, healer }
    }

    #[inline]
    pub fn get(&self, t: ClimberType) -> &T {
        match t {
            ClimberType::Melee => &self.melee,
            ClimberType::Ranged => &self.ranged,
            ClimberType::Healer => &self.healer,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, t: ClimberType) -> &mut T {
        match t {
            ClimberType::Melee => &mut self.melee,
            ClimberType::Ranged => &mut self.ranged,
            ClimberType::Healer => &mut self.healer,
        }
    }

    /// Pairs in [`ClimberType::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (ClimberType, &T)> {
        ClimberType::ALL.into_iter().map(move |t| (t, self.get(t)))
    }
}

impl<T: Copy> PerType<T> {
    pub fn splat(v: T) -> Self {
        Self::new(v, v, v)
    }
}

impl<T: Default> Default for PerType<T> {
    fn default() -> Self {
        Self { melee: T::default(), ranged: T::default(), healer: T::default() }
    }
}

impl<T> std::ops::Index<ClimberType> for PerType<T> {
    type Output = T;
    fn index(&self, t: ClimberType) -> &T {
        self.get(t)
    }
}

impl<T> std::ops::IndexMut<ClimberType> for PerType<T> {
    fn index_mut(&mut self, t: ClimberType) -> &mut T {
        self.get_mut(t)
    }
}
