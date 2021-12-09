use std::fmt::Debug;

pub mod seair;
pub mod seichar;
pub mod seir;
pub mod simple;
pub mod sir;
pub use seair::*;
pub use seichar::*;
pub use seir::*;
pub use simple::*;
pub use sir::*;

impl<C: Debug> Debug for SIR<C> {
    default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Susceptible => write!(f, "S"),
            Self::Infectious(c) => write!(f, "I({:?})", c),
            Self::Recovered(c) => write!(f, "R({:?})", c),
            Self::Dead(c) => write!(f, "D({:?})", c),
        }
    }
}

impl<C: Debug> Debug for SEIR<C> {
    default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Susceptible => write!(f, "S"),
            Self::Exposed(c) => write!(f, "E({:?})", c),
            Self::Infectious(c) => write!(f, "I({:?})", c),
            Self::Recovered(c) => write!(f, "R({:?})", c),
            Self::Dead(c) => write!(f, "D({:?})", c),
        }
    }
}

impl<C: Debug> Debug for SEAIR<C> {
    default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Susceptible => write!(f, "S"),
            Self::Exposed(c) => write!(f, "E({:?})", c),
            Self::Asymptomatic(c) => write!(f, "A({:?})", c),
            Self::Infectious(c) => write!(f, "I({:?})", c),
            Self::Recovered(c) => write!(f, "R({:?})", c),
            Self::Dead(c) => write!(f, "D({:?})", c),
        }
    }
}

impl<C: Debug> Debug for SEICHAR<C> {
    default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Susceptible => write!(f, "S"),
            Self::Exposed(c) => write!(f, "E({:?})", c),
            Self::Infectious(c) => write!(f, "I({:?})", c),
            Self::Critical(c) => write!(f, "C({:?})", c),
            Self::Severe(c) => write!(f, "H({:?})", c),
            Self::Asymptomatic(c) => write!(f, "A({:?})", c),
            Self::Recovered(c) => write!(f, "R({:?})", c),
            Self::Dead(c) => write!(f, "D({:?})", c),
        }
    }
}

/// Implements a debug
macro_rules! implDebug {
    (SIR: $ty:ty) => {implDebug!(SIR<$ty> { });};
    (SEIR: $ty:ty) => {implDebug!(SEIR<$ty> { Exposed: "E" });};
    (SEAIR: $ty:ty) => {implDebug!(SEAIR<$ty> { Exposed: "E", Asymptomatic: "A" });};
    (SEICHAR: $ty:ty) => {implDebug!(SEICHAR<$ty> { Exposed: "E", Asymptomatic: "A", Severe: "H", Critical: "C" });};

    ($ty:ty { $($st:ident: $opt:literal),* $(,)? }) => {
        impl Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Susceptible => write!(f, "S"),
                    $(
                        Self::$st(_) => write!(f, $opt),
                    )*
                    Self::Infectious(_) => write!(f, "I"),
                    Self::Recovered(_) => write!(f, "R"),
                    Self::Dead(_) => write!(f, "D"),
                }
            }
        }
    };

}

implDebug!(SIR: ());
implDebug!(SEIR: ());
implDebug!(SEAIR: ());
implDebug!(SEICHAR: ());

/// Type alias for simple SIR model enumeration
pub type SIRm = SIR<()>;

/// Type alias for simple SEIR model enumeration
pub type SEIRm = SEIR<()>;

/// Type alias for simple SEAIR model enumeration
pub type SEAIRm = SEAIR<()>;

/// Type alias for simple SEICHAR model enumeration
pub type SEICHARm = SEICHAR<()>;

/// Type alias for simple SIR agent
pub type SirAgent<V> = SimpleAgent<SIRm, V>;

/// Type alias for simple SIR agent
pub type SeirAgent<V> = SimpleAgent<SEIRm, V>;

/// Type alias for simple SIR agent
pub type SeairAgent<V> = SimpleAgent<SEAIRm, V>;

/// Type alias for simple SIR agent
pub type SeicharAgent<V> = SimpleAgent<SEICHARm, V>;
