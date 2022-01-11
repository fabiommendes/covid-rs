use std::{rc::Rc, sync::Arc};

use crate::{
    prelude::{Age, Real},
    sim::HasAge,
};

use super::ParamSet;

////////////////////////////////////////////////////////////////////////////////
// TRAIT DECLARATIONS
////////////////////////////////////////////////////////////////////////////////

/// A trait that provide descriptions of basic epidemiological parameters independently
/// from any agent state.
pub trait EpiParams {
    fn incubation_period(&self) -> Real;
    fn infectious_period(&self) -> Real;
    fn severe_period(&self) -> Real;
    fn critical_period(&self) -> Real;
    fn asymptomatic_infectiousness(&self) -> Real;
    fn prob_asymptomatic(&self) -> Real;
    fn prob_severe(&self) -> Real;
    fn prob_critical(&self) -> Real;
    fn prob_protect(&self) -> Real;
    fn case_fatality_ratio(&self) -> Real;

    fn prob_death(&self) -> Real {
        let factor = self.prob_critical() * self.prob_severe();
        return self.case_fatality_ratio() / factor;
    }

    fn infection_fatality_ratio(&self) -> Real {
        self.case_fatality_ratio() * (1.0 - self.prob_asymptomatic())
    }

    fn incubation_transition_prob(&self) -> Real {
        self.daily_probability(self.incubation_period())
    }

    fn infectious_transition_prob(&self) -> Real {
        self.daily_probability(self.infectious_period())
    }

    fn severe_transition_prob(&self) -> Real {
        self.daily_probability(self.severe_period())
    }

    fn critical_transition_prob(&self) -> Real {
        self.daily_probability(self.severe_period())
    }

    /// A helper method that computes the daily transition probability from the
    /// transition period.
    #[inline]
    fn daily_probability(&self, value: Real) -> Real {
        daily_probability(value)
    }
}

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
pub trait PartialEpiParams<S> {
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

    /// Probability that an exposed agent does not develop any symptoms (E to A).
    ///
    /// The complement is the probability for transitioning from E to I.
    ///
    /// A value of 0.0 makes SEAIR coincide with SEIR;
    fn prob_asymptomatic(&self, _obj: &S) -> Real;

    /// Probability that an infectious agent develops severe symptoms (I to H).
    ///
    /// The complement is the probability for transitioning from I to R.
    ///
    /// A value of 1.0 makes it coincide with SEIR and keep the fatality rate.
    /// A value of 0.0 imposes a transition to R, with zero chance of deaths.
    fn prob_severe(&self, _obj: &S) -> Real;

    /// Probability that a severe agent develops critical symptoms (S to C).
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
    ///
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
}

/// Computes the daily transition probability from the transition period.
#[inline(always)]
pub(crate) fn daily_probability(value: Real) -> Real {
    1.0 - (-1. / value).exp()
}

////////////////////////////////////////////////////////////////////////////////
// CONCRETE TYPES AND IMPLEMENTATIONS
////////////////////////////////////////////////////////////////////////////////

/// A simple struct that implements a
pub struct PartialEpiParamsClosure<P, B>
where
    P: PartialEpiParams<B>,
{
    pub params: P,
    pub bind: B,
}

impl<P, B> PartialEpiParamsClosure<P, B>
where
    P: PartialEpiParams<B>,
{
    /// Create a new closure bind.
    pub fn new(params: P, bind: B) -> Self {
        return PartialEpiParamsClosure { params, bind };
    }
}

macro_rules! closure_method {
    ($name:ident) => {
        fn $name(&self) -> Real {
            return self.params.$name(&self.bind);
        }
    };
}

impl<S, P> EpiParams for PartialEpiParamsClosure<P, S>
where
    P: PartialEpiParams<S>,
{
    closure_method!(incubation_period);
    closure_method!(infectious_period);
    closure_method!(severe_period);
    closure_method!(critical_period);
    closure_method!(asymptomatic_infectiousness);
    closure_method!(prob_asymptomatic);
    closure_method!(prob_severe);
    closure_method!(prob_protect);
    closure_method!(prob_critical);
    closure_method!(prob_death);
    closure_method!(case_fatality_ratio);
    closure_method!(infection_fatality_ratio);
    closure_method!(incubation_transition_prob);
    closure_method!(infectious_transition_prob);
    closure_method!(severe_transition_prob);
    closure_method!(critical_transition_prob);
}

impl<ST: HasAge, P: PartialEpiParams<Age>> ParamSet<ST> for Rc<P> {
    type BoundParams = PartialEpiParamsClosure<Self, Age>;

    fn bind(&self, st: &ST) -> Self::BoundParams {
        PartialEpiParamsClosure {
            params: self.clone(),
            bind: st.age(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trait implementations
////////////////////////////////////////////////////////////////////////////////

// TRAIT IMPLEMENTATIONS FOR BOXED TYPES ///////////////////////////////////////

macro_rules! delegate_to_ref {
    ($name:ident, bind=$ty:ident) => {
        fn $name(&self, obj: &$ty) -> Real {
            return self.as_ref().$name(obj);
        }
    };
    ($name:ident) => {
        fn $name(&self) -> Real {
            return self.as_ref().$name();
        }
    };
}

impl<T: EpiParams> EpiParams for Rc<T> {
    delegate_to_ref!(incubation_period);
    delegate_to_ref!(infectious_period);
    delegate_to_ref!(severe_period);
    delegate_to_ref!(critical_period);
    delegate_to_ref!(asymptomatic_infectiousness);
    delegate_to_ref!(prob_asymptomatic);
    delegate_to_ref!(prob_severe);
    delegate_to_ref!(prob_critical);
    delegate_to_ref!(prob_protect);
    delegate_to_ref!(case_fatality_ratio);
}

impl<T: EpiParams> EpiParams for Arc<T> {
    delegate_to_ref!(incubation_period);
    delegate_to_ref!(infectious_period);
    delegate_to_ref!(severe_period);
    delegate_to_ref!(critical_period);
    delegate_to_ref!(asymptomatic_infectiousness);
    delegate_to_ref!(prob_asymptomatic);
    delegate_to_ref!(prob_severe);
    delegate_to_ref!(prob_critical);
    delegate_to_ref!(prob_protect);
    delegate_to_ref!(case_fatality_ratio);
}

impl<T: EpiParams> EpiParams for Box<T> {
    delegate_to_ref!(incubation_period);
    delegate_to_ref!(infectious_period);
    delegate_to_ref!(severe_period);
    delegate_to_ref!(critical_period);
    delegate_to_ref!(asymptomatic_infectiousness);
    delegate_to_ref!(prob_asymptomatic);
    delegate_to_ref!(prob_severe);
    delegate_to_ref!(prob_critical);
    delegate_to_ref!(prob_protect);
    delegate_to_ref!(case_fatality_ratio);
}

impl<S, T> PartialEpiParams<S> for Rc<T>
where
    T: PartialEpiParams<S>,
{
    delegate_to_ref!(incubation_period, bind = S);
    delegate_to_ref!(infectious_period, bind = S);
    delegate_to_ref!(severe_period, bind = S);
    delegate_to_ref!(critical_period, bind = S);
    delegate_to_ref!(prob_protect, bind = S);
    delegate_to_ref!(asymptomatic_infectiousness, bind = S);
    delegate_to_ref!(prob_asymptomatic, bind = S);
    delegate_to_ref!(prob_severe, bind = S);
    delegate_to_ref!(prob_critical, bind = S);
    delegate_to_ref!(prob_death, bind = S);
    delegate_to_ref!(case_fatality_ratio, bind = S);
    delegate_to_ref!(infection_fatality_ratio, bind = S);
    delegate_to_ref!(incubation_transition_prob, bind = S);
    delegate_to_ref!(infectious_transition_prob, bind = S);
    delegate_to_ref!(severe_transition_prob, bind = S);
    delegate_to_ref!(critical_transition_prob, bind = S);
}

impl<S, T> PartialEpiParams<S> for Arc<T>
where
    T: PartialEpiParams<S>,
{
    delegate_to_ref!(incubation_period, bind = S);
    delegate_to_ref!(infectious_period, bind = S);
    delegate_to_ref!(severe_period, bind = S);
    delegate_to_ref!(critical_period, bind = S);
    delegate_to_ref!(prob_protect, bind = S);
    delegate_to_ref!(asymptomatic_infectiousness, bind = S);
    delegate_to_ref!(prob_asymptomatic, bind = S);
    delegate_to_ref!(prob_severe, bind = S);
    delegate_to_ref!(prob_critical, bind = S);
    delegate_to_ref!(prob_death, bind = S);
    delegate_to_ref!(case_fatality_ratio, bind = S);
    delegate_to_ref!(infection_fatality_ratio, bind = S);
    delegate_to_ref!(incubation_transition_prob, bind = S);
    delegate_to_ref!(infectious_transition_prob, bind = S);
    delegate_to_ref!(severe_transition_prob, bind = S);
    delegate_to_ref!(critical_transition_prob, bind = S);
}
