use crate::prelude::Real;
use rand::Rng;
use std::convert::{From, TryFrom};

/// Baseline/Variant of concern.
/// A bool-like enum that defines if a variant is a Variant of concern
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Variant {
    Baseline,
    VoC,
}

impl Variant {
    /// Create variants at random with a given probability for a VoC
    pub fn random(rng: &mut impl Rng, prob_voc: Real) -> Self {
        if rng.gen_bool(prob_voc) {
            Variant::VoC
        } else {
            Variant::Baseline
        }
    }

    pub fn is_baseline(self) -> bool {
        return self == Variant::Baseline;
    }
    pub fn is_voc(self) -> bool {
        return self == Variant::VoC;
    }

    pub fn csv(self) -> String {
        format!("{}", usize::from(self))
    }

    pub fn select<T>(self, baseline: T, voc: T) -> T {
        match self {
            Variant::Baseline => baseline,
            Variant::VoC => voc,
        }
    }
}

impl From<bool> for Variant {
    fn from(is_voc: bool) -> Self {
        if is_voc {
            Variant::VoC
        } else {
            Variant::Baseline
        }
    }
}

impl Default for Variant {
    fn default() -> Self {
        Variant::Baseline
    }
}

impl TryFrom<usize> for Variant {
    type Error = &'static str;
    fn try_from(n: usize) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(Self::Baseline),
            1 => Ok(Self::VoC),
            _ => Err("invalid index for variant"),
        }
    }
}

impl From<Variant> for u8 {
    fn from(value: Variant) -> u8 {
        match value {
            Variant::Baseline => 0,
            Variant::VoC => 1,
        }
    }
}

impl From<Variant> for usize {
    fn from(value: Variant) -> usize {
        u8::from(value) as usize
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Variants {
    mask: u8,
}

impl Variants {
    pub fn add(mut self, v: Variant) {
        self.mask |= 1 >> u8::from(v);
    }
}
