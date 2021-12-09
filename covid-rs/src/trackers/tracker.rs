use crate::{prelude::Real, utils::Sample};
pub type DynTracker<T> = Box<dyn Tracker<T>>;
pub type TrackerList<T> = Vec<(usize, DynTracker<T>)>;

/// Trait that implements a method that tracks some value of type T and
/// perform some action like collecting statistics, health metrics, emit signals,
/// etc.
///
/// The tracker receives an immutable reference to the tracked value. For a
/// mutable tracker that can modify the tracked content, use TrackerMut.
///
/// Functions, boxed functions, closures, etc can be executed interpreted as
/// trackers using an Into<Tracker<T>> trait.
pub trait Tracker<T> {
    /// Track value.
    fn track(&mut self, value: &T);

    /// Choose to execute tracker at iteration n if tracker must execute every
    /// freq frames with given possibly non-null offset.
    fn maybe_track(&mut self, value: &T, n: usize, freq: usize, offset: usize) {
        if ((n - offset) % freq) == 0 {
            self.track(value);
        }
    }

    // /// Return a boxed version of itself. If object Clone, it should always
    // /// return Some(Box).
    // ///
    // /// This method is declared in the trait implementation to make it possible
    // /// to implement something similar to Clone for dynamic Track objects.
    // fn as_dyn_tracker(&self) -> Option<Box<dyn Tracker<T>>> {
    //     // Some(self.as_clone()?)
    //     None
    // }
}

/// A mutable tracker: receives a mutable reference to the tracked value and may
/// opt to modify it during the tracking stage.
pub trait TrackerMut<T> {
    /// Track value.
    fn track_mut(&mut self, value: &mut T);

    /// Choose to execute tracker at iteration n if tracker must execute every
    /// freq frames with given possibly non-null offset.
    fn maybe_track_mut(&mut self, value: &mut T, n: usize, freq: usize, offset: usize) {
        if ((n - offset) % freq) == 0 {
            self.track_mut(value);
        }
    }

    // /// Return a boxed version of itself. If object Clone, it should always
    // /// return Some(Box).
    // ///
    // /// This method is declared in the trait implementation to make it possible
    // /// to implement something similar to Clone for dynamic Track objects.
    // fn as_dyn_tracker_mut(&self) -> Option<Box<dyn TrackerMut<T>>> {
    //     None
    // }
}

/////////////////////////////////////////////////////////////////////////////
// Tracker instances
/////////////////////////////////////////////////////////////////////////////

impl<T, R1, R2> Tracker<T> for (R1, R2)
where
    R1: Tracker<T>,
    R2: Tracker<T>,
{
    fn track(&mut self, value: &T) {
        self.0.track(value);
        self.1.track(value);
    }

    // fn as_dyn_tracker(&self) -> Option<Box<dyn Tracker<T>>> {
    //     let t1 = self.as_dyn_tracker()?;
    //     let t2 = self.as_dyn_tracker()?;
    //     return;
    // }
}

impl<T, R1, R2, R3> Tracker<T> for (R1, R2, R3)
where
    R1: Tracker<T>,
    R2: Tracker<T>,
    R3: Tracker<T>,
{
    fn track(&mut self, value: &T) {
        self.0.track(value);
        self.1.track(value);
        self.2.track(value);
    }
}

impl<T, R1, R2, R3, R4> Tracker<T> for (R1, R2, R3, R4)
where
    R1: Tracker<T>,
    R2: Tracker<T>,
    R3: Tracker<T>,
    R4: Tracker<T>,
{
    fn track(&mut self, value: &T) {
        self.0.track(value);
        self.1.track(value);
        self.2.track(value);
        self.3.track(value);
    }
}

impl<T> Tracker<T> for () {
    fn track(&mut self, _: &T) {}

    // fn as_dyn_tracker(&self) -> Option<Box<dyn Tracker<T>>> {
    //     Some(())
    // }
}

impl<T> Tracker<T> for TrackerList<T> {
    fn track(&mut self, value: &T) {
        for (_, r) in self.iter_mut() {
            r.track(value);
        }
    }
}

impl<T> Tracker<T> for fn(&T) {
    fn track(&mut self, value: &T) {
        self(value);
    }
}

// #[macro_export]
macro_rules! sampling_tracker {
 ($($ty:ty),*) => {
$(impl Tracker<Real> for $ty
    {
        fn track(&mut self, value: &Real) {
            self.observe(*value);
        }
    })*
 };
}

sampling_tracker!(Vec<Real>);

/////////////////////////////////////////////////////////////////////////////
// Wrappers
/////////////////////////////////////////////////////////////////////////////

/// Target wrapper for IntoTracker on Sampling types
pub struct SamplingTracker<T: Sample>(T);

impl<S: Sample> Tracker<Real> for SamplingTracker<S> {
    fn track(&mut self, value: &Real) {
        self.0.observe(*value)
    }
}

pub struct FnTracker<F>(F);

impl<T, F> Tracker<T> for FnTracker<F>
where
    F: FnMut(&T),
{
    fn track(&mut self, value: &T) {
        (&mut self.0)(value)
    }
}

impl<T, F> TrackerMut<T> for FnTracker<F>
where
    F: FnMut(&mut T),
{
    fn track_mut(&mut self, value: &mut T) {
        (&mut self.0)(value)
    }
}
