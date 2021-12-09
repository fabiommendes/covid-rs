use super::{
    bind::{Bind, BindRef},
    MultiComponent,
};
use crate::prelude::Real;

/// A set of epidemiological parameters that may depend on some state. If no such
/// dependency exists, the trait can thought as EpiParams<()> and
/// an automatic derivation of EpiLocalParams<()> is provided.
///
/// This is useful in several situations: epidemiological parameters may depend
/// on age, gender, vaccination status, time from infection, socio-economic
/// factors etc. Instead of anticipating all possible dependencies, we abstract
/// Two situations:
///
///     1. The global param set: implement this trait
///     2. The global param set specialized to some set of properties that
///        usually depend on each agent: implement EpiLocalParams.
///
/// We can convert a global param to a local one using the LocalBind trait that
/// maps a EpiParams instance to an EpiLocalParams via some agent or some
/// properties of that agent.
///
/// State must be feed as an additional argument to the getter functions of
/// this trait.
pub trait EpiParamsT<S> {
    /// Incubation period is the average duration in the "Exposed" category.
    /// In this stage, agents are infected but *CANNOT* yet infect other agents.
    fn incubation_period(&self, obj: &S) -> Real;

    /// Infectious period is the average duration in the "Infectious" category"
    /// In this stage, agents *CAN* infect other agents.
    fn infectious_period(&self, obj: &S) -> Real;

    /// Average duration of a "severe" case.
    ///
    /// A value of zero  is equivalent to disabling the clinical evolution in the
    /// CH compartments, effectively transforming SEICHAR to SEIR.
    fn severe_period(&self, _obj: &S) -> Real;

    /// Average duration of a "critical" case.
    ///
    /// Like severe_period() a null value makes it coincide with SEIR;
    fn critical_period(&self, _obj: &S) -> Real;

    /// Probability that agent has some extra protection against infection (e.g., due to vaccination or masks).
    fn prob_protect(&self, _obj: &S) -> Real;

    /// Relative infectiousness of asymptomatic agents.
    ///
    /// Make it equal to 1.0 to coincide with SEIR;
    fn asymptomatic_infectiousness(&self, _obj: &S) -> Real;

    /// Probability of an exposed agent not developing any symptoms (E to A).
    ///
    /// The complement is the probability for transitioning from E to I.
    ///
    /// A value of 0.0 makes it coincide with SEIR;
    fn prob_asymptomatic(&self, _obj: &S) -> Real;

    /// Probability of an infectious agent develop severe symptoms (I to H).
    ///
    /// The complement is the probability for transitioning from I to R.
    ///
    /// A value of 1.0 makes it coincide with SEIR and keep the fatality rate.
    /// A value of 0.0 imposes a transition to R, with zero chance of deaths.
    fn prob_severe(&self, _obj: &S) -> Real;

    /// Probability of a severe agent develop critical symptoms (S to C).
    ///
    /// The complement is the probability for transitioning from S to R.
    ///
    /// Must adopt the same values of prob_severe to coincide with SEIR.
    fn prob_critical(&self, _obj: &S) -> Real;

    /// Probability of a critical agent to die (C to D).
    ///
    /// The complement is the probability for transitioning from C to R.
    /// The default value uses CFR and the transition probabilities I -> S and
    /// S -> I to compute this probability.
    #[inline]
    fn prob_death(&self, obj: &S) -> Real {
        let factor = self.prob_critical(obj) * self.prob_severe(obj);
        return self.case_fatality_ratio(obj) / factor;
    }

    /// Probability of death for (symptomatic) cases.
    /// Defaults to zero.
    fn case_fatality_ratio(&self, _obj: &S) -> Real;

    /// Probability of death for all infections (symptomatic or not)
    ///
    /// Impls should usually override case_fatality_ratio() and prob_asymptomatic()
    /// and use the default implementation of this method.
    #[inline]
    fn infection_fatality_ratio(&self, obj: &S) -> Real {
        self.case_fatality_ratio(obj) * (1.0 - self.prob_asymptomatic(obj))
    }

    /// Probability of transition E -> (A or I) in a single day.
    fn incubation_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.incubation_period(obj))
    }

    /// Probability of transition I -> (H or R) in a single day.
    fn infectious_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.infectious_period(obj))
    }

    /// Probability of transition S -> (C or R) in a single day.
    fn severe_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.severe_period(obj))
    }

    /// Probability of transition C -> (D or R) in a single day.
    fn critical_transition_prob(&self, obj: &S) -> Real {
        self.daily_probability(self.severe_period(obj))
    }

    /// A helper method that computes the daily transition probability from the
    /// transition period.
    #[inline]
    fn daily_probability(&self, value: Real) -> Real {
        daily_probability(value)
    }

    /// Creates a bound SimpleSEIRParams object bound to the given state.
    ///
    /// It binds to a reference to self, which is more efficient, but may
    /// create problems with lifetimes.
    fn bind<'a>(&'a self, bind: S) -> BindRef<'a, Self, S>
    where
        Self: Sized,
        S: Clone,
    {
        BindRef::new(self, bind)
    }

    /// Creates a bound SimpleSEIRParams object bound to the given state.
    ///
    fn bind_copy(&self, obj: &S) -> Bind<Self, S>
    where
        Self: Clone,
        S: Clone,
    {
        Bind::new(self.clone(), obj.clone())
    }

    /// Creates a bound SimpleSEIRParams object bound to the given state.
    ///
    /// It binds to a reference to self, which is more efficient, but may
    /// create problems with lifetimes.
    fn with_bounded_params<R>(&self, bind: S, f: impl FnOnce(&BindRef<'_, Self, S>) -> R) -> R
    where
        Self: Sized,
    {
        // let ptr: *mut S = obj;
        let params = BindRef::new(self, bind);
        f(&params)
        // // Safety: object cannot be moved
        // unsafe {
        //     f(&params, &mut *ptr);
        // }
        // drop(obj);
    }
}

/// A trait for objects tha expose the internal data representation T of the
/// EpiLocalParams param set.
///
/// This is useful in conversions between parameters but is not very useful
/// to end users.
pub trait EpiParamsData<T> {
    fn with_incubation_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;
    fn with_infectious_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;
    fn with_severe_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;
    fn with_critical_period_data<S>(&self, f: impl FnOnce(&T) -> S) -> S;

    /// Helper method that may make it easier to implement with_*_data() methods
    /// for missing values.]
    fn with_scalar_data<R, S>(&self, scalar: R, f: impl FnOnce(&T) -> S) -> S
    where
        T: MultiComponent<Elem = R>,
    {
        let data = T::from_component(scalar);
        f(&data)
    }
}

/// Computes the daily transition probability from the transition period.
#[inline(always)]
pub(crate) fn daily_probability(value: Real) -> Real {
    1.0 - (-1. / value).exp()
}
