use super::{
    table_tracker::TableTracker,
    tracker::{Tracker, TrackerList},
};
use crate::prelude::EpiModel;
use crate::sim::Population;
use getset::{CopyGetters, Getters};
use std::fmt::Debug;

/// Epicurve reporter that can be extended with an arbitrary list of FnMut()
/// reporters.
#[derive(Default, Getters, CopyGetters)]
pub struct EpiTracker<P> {
    #[getset(get_copy = "pub")]
    n_iter: usize,
    #[getset(get = "pub")]
    epicurves: TableTracker<usize>,
    #[getset(get = "pub")]
    reporters: TrackerList<P>,
}

impl<P> EpiTracker<P> {
    pub fn new(population: &P) -> Self
    where
        P: Population,
        P::State: EpiModel,
    {
        let mut new = EpiTracker {
            n_iter: 0,
            reporters: vec![],
            epicurves: TableTracker::new(P::State::CARDINALITY),
        };
        new.epicurves.update(population, true);
        return new;
    }

    /// Return a CSV string with the content of the Epicurves.
    pub fn render_epicurve_csv(&self, head: &str) -> String {
        self.epicurves.render_csv(head, ',')
    }

    /// Return an array with the last row of epicurves.
    pub fn tip(&self) -> Vec<usize> {
        self.epicurves.tip()
    }

    /// Return the i-th row of epicurves.
    pub fn row(&self, i: usize) -> Option<Vec<usize>> {
        self.epicurves.row(i)
    }

    /// Return the i-th epicurve.
    pub fn col(&self, i: usize) -> Option<Vec<usize>> {
        self.epicurves.col(i)
    }

    /// Return a copy of reporter, ignoring the user defined ones
    pub fn copy(&self) -> Self {
        EpiTracker {
            n_iter: self.n_iter,
            epicurves: self.epicurves.clone(),
            reporters: vec![],
        }
    }
}

impl<P> Debug for EpiTracker<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EpiReporter")
            .field("epicurves", &self.epicurves)
            .finish()
    }
}

impl<P> Tracker<P> for EpiTracker<P>
where
    P: Population,
    P::State: EpiModel,
{
    fn track(&mut self, value: &P) {
        self.epicurves.track(value);
        self.reporters.track(value);
        self.n_iter += 1;
    }
}
