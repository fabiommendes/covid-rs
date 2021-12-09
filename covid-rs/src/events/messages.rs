use crate::{prelude::Time, sim::Id};

/// Events are summarized by Message values. A message is typically a enum type
/// that characterizes the various types of events that may occur and their
/// respective payloads.  
pub trait Msg {
    /// Must return a unique id for event type.
    fn id(&self) -> usize;

    /// Maximum id for all events
    const EVENT_TYPES_COUNT: usize;
}

/// Main event type for the main simulation
///
/// It register events that may occur during the simulation of an epidemic.
#[derive(Debug, Clone)]
pub enum EpidemicSimulationMsg {
    StartStep(Time),
    EndStep(Time, usize),
    NewInfection(Id, Id),
}

impl EpidemicSimulationMsg {
    pub const START_STEP_ID: usize = 0;
    pub const END_STEP_ID: usize = 1;
    pub const NEW_INFECTION_ID: usize = 2;
}

impl Msg for EpidemicSimulationMsg {
    const EVENT_TYPES_COUNT: usize = 3;

    fn id(&self) -> usize {
        match self {
            Self::StartStep(_) => Self::START_STEP_ID,
            Self::EndStep(_, _) => Self::END_STEP_ID,
            Self::NewInfection(_, _) => Self::NEW_INFECTION_ID,
        }
    }
}
