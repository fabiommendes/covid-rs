
/// Vaccine applied to agent. First and second doses are treated as different
/// vaccines.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Vaccine {
    None,
    CoronaVac1,
    CoronaVac2,
    Oxford1,
    Oxford2,
    Pfzer1,
    Pfzer2,
    Sputnik1,
    Sputnik2,
    JnJ,
}
impl Default for Vaccine {
    fn default() -> Self {
        Vaccine::None
    }
}
