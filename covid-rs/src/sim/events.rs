use super::Id;

/** EVENTS *******************************************************************/

/// The Event describes a minimal structure that Event enums must have to
/// be used in an efficient event dispatcher struct  
pub trait Event {
    /// Must return a unique id for event type.
    fn id(&self) -> usize;

    /// Maximum id for all events
    fn length() -> usize;
}

/// Event
#[derive(Debug, Clone)]
pub enum EpiEvent {
    StartStep,
    EndStep(usize),
    NewInfection(Id, Id),
}

impl Event for EpiEvent {
    fn id(&self) -> usize {
        match self {
            Self::StartStep => 0,
            Self::EndStep(_) => 1,
            Self::NewInfection(_, _) => 2,
        }
    }

    fn length() -> usize {
        return 3;
    }
}

/** EVENT HANDLERS ***********************************************************/

/// The EventHandler<E> trait describes an object that can process some
/// event of type E (usually an enum type) in a given context Ctx (usually a
/// simulation object).
///
/// Event handlers are useful to collect statistics, and generate reports.
pub trait EventHandler<E>
where
    E: Event,
{
    /// Handle event in the given context. Usually, the context is a reference
    /// to a simulation object. The event handler cannot affect the context.
    fn handle(&mut self, event: &E);

    /// In order to make dispatch more efficient, event handlers are registered
    /// to separate lanes depend on event type. This usually corresponds to a
    /// different Id for each case of a enum type.
    ///
    /// This method should return the handle id for the event handler.
    fn handle_id(&self) -> usize;

    /// Event handlers may produce reports for the user with information
    /// about the handler state. This is usually used in event handlers that collect
    /// statistics about the simulation.
    fn report(&self) -> String;
}

#[derive(Debug, Clone, Default)]
pub struct InfectionTraceHandler {
    pairs: Vec<(Id, Id)>,
}

impl InfectionTraceHandler {
    /// Create new handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Count infections for the given id
    pub fn count_new_infections_by(&self, id: Id) -> usize {
        self.pairs.iter().filter(|(a, _)| id == *a).count()
    }
}

impl EventHandler<EpiEvent> for InfectionTraceHandler {
    fn handle(&mut self, event: &EpiEvent) {
        if let &EpiEvent::NewInfection(a, b) = event {
            self.pairs.push((a, b))
        }
    }

    fn handle_id(&self) -> usize {
        return EpiEvent::NewInfection(0, 0).id();
    }

    fn report(&self) -> String {
        return "".to_string();
    }
}

/// Count the number of infections per step.
///
/// Listen to the EndStep(num_infections) event
#[derive(Debug, Clone, Default)]
pub struct EpiTracker {
    infections: Vec<usize>,
}

impl EpiTracker {
    /// Create new handler
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventHandler<EpiEvent> for EpiTracker {
    fn handle(&mut self, event: &EpiEvent) {
        if let &EpiEvent::EndStep(n) = event {
            self.infections.push(n)
        }
    }

    fn handle_id(&self) -> usize {
        return EpiEvent::EndStep(0).id();
    }

    fn report(&self) -> String {
        return "".to_string();
    }
}

/** EVENT DISPATCHER *********************************************************/
pub struct EventDispatcher<E>
where
    E: Event,
{
    listeners: Vec<Vec<Box<dyn EventHandler<E>>>>,
}

impl<E> EventDispatcher<E>
where
    E: Event,
{
    /// Create a new event dispatcher
    pub fn new() -> Self {
        let mut events = Vec::with_capacity(E::length());
        for _ in 0..E::length() {
            events.push(Vec::new());
        }
        return EventDispatcher { listeners: events };
    }

    /// Trigger all handlers for the given event.
    pub fn trigger(&mut self, event: &E) {
        for ev in &mut self.listeners[event.id()] {
            ev.handle(event)
        }
    }

    /// Register event handler
    pub fn register(&mut self, handler: Box<dyn EventHandler<E>>) {
        let id = handler.handle_id();
        self.listeners[id].push(handler);
    }
}
