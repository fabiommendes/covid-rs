/*
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum Healthcare {
    None,
    Improvised,
    Proper,
}

impl Healthcare {
    pub fn can_infect(self) -> bool {
        self != Healthcare::Proper
    }
    pub fn csv(self) -> String {
        format!("{}", usize::from(self))
    }
}

impl Default for Healthcare {
    fn default() -> Self {
        Healthcare::Proper
    }
}

impl From<Healthcare> for usize {
    fn from(value: Healthcare) -> usize {
        match value {
            Healthcare::None => 0,
            Healthcare::Improvised => 1,
            Healthcare::Proper => 2,
        }
    }
}

impl TryFrom<usize> for Healthcare {
    type Error = &'static str;

    fn try_from(n: usize) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(Healthcare::None),
            1 => Ok(Healthcare::Improvised),
            2 => Ok(Healthcare::Proper),
            _ => Err("integer outside bounds"),
        }
    }
}
*/
