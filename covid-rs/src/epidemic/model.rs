use crate::{
    prelude::Real,
    sim::{HasEpiModel, Id, Population},
};
use paste::paste;
use rand::{prelude::SliceRandom, Rng};

/// Basic trait for all compartment-like epidemic models. This includes all the
/// SIR family of models and possibly other more generic cases.
///
/// An Epi model has a defined CARDINALITY that designates the number of
/// different basic states an agent can be in. Those states can have further
/// attached information (e.g., is infected with some given severity level) which
/// are not accounted for the CARDINALITY of a model.
///
/// The index() method converts a state to the corresponding integer representation
pub trait EpiModel: Sized + Clone {
    /// Maximum number of epidemiological states
    const CARDINALITY: usize;
    const CSV_HEADER: &'static str;

    /// Return the vector of column names for each compartment
    fn column_names() -> Vec<String> {
        let columns = Self::CSV_HEADER.split(',').map(|s| s.to_string()).collect();
        return columns;
    }

    /// Return the vector of column names for each compartment
    fn column_names_short() -> Vec<String> {
        let columns = Self::CSV_HEADER
            .split(',')
            .map(|s| {
                if let Some(c) = s.chars().nth(0) {
                    c.to_uppercase().to_string()
                } else {
                    "".to_string()
                }
            })
            .collect();
        return columns;
    }

    /// Index of the susceptible state.
    const S: usize;

    /// Index of the dead state.
    const D: usize;

    /// A type that represents the disease associated with the epidemiological
    /// model. Use the unity type to signal an abstract epidemiological model
    /// not associated with a concrete disease.
    type Disease;

    /// An additional clinical state that may describe other parameters of the
    /// contamination like severity, access to healthcare, etc.
    type Clinical;

    /// The index associated with the current epidemiological state. This is a
    /// simple mapping that usually depends on the epidemic model. For a SIR-like
    /// model, for instance, can be something like susceptible => 0,
    /// infectious => 1, recovered => 2, dead => 3.
    fn index(&self) -> usize;

    /// Create new infectious individual with the given clinical parameters.
    fn new_infectious_with(clinical: &Self::Clinical) -> Self;

    /// Create a new infectious individual with the default infectious state.
    /// In a SIR-like model, this creates an individual to I.
    fn new_infectious() -> Self
    where
        Self::Clinical: Default,
    {
        Self::new_infectious_with(&Self::Clinical::default())
    }

    /// Force element into an infectious state, when this is possible.
    ///
    /// This usually means that it will migrate any contaminated state back to
    /// infectious and the susceptible state will be left as is.
    fn force_infectious(&mut self, force_dead: bool) -> bool;

    /// Return the relative probability of contamination from self. Usually this
    /// should be equal to 1.0 for the default infectious state some ratio
    /// compared to this state.
    fn contagion_odds(&self) -> Real;

    /// Return true if one agent can contaminate the other. This must return true
    /// if contagion is, in principle, possible. Further external restrictions
    /// (like, e.g., physical distance) may make the infection impossible, but
    /// this should be treated later in the pipeline.
    fn can_contaminate(&self, other: &Self) -> bool {
        self.is_contagious() && other.is_susceptible()
    }

    /// Transfer contamination state from other and return a boolean telling if
    /// the contamination occurred or not. This method should force contamination
    /// even when it is not clinically possible (e.g., self is recovered).
    /// It is used by contaminate_from(), which checks basic clinical plausibility
    /// and uses this method to transfer contamination to a susceptible agent
    /// if contamination is possible.
    ///
    /// This method should not simply clone the contamination state from `other`
    /// but rather modify self to the proper state that describes the start
    /// of an infection. For instance, in a model like SEIR, if other is in some
    /// infected state, like `SEIR::Recovered(params)`, self will be modified
    /// to `SEIR::Exposed(params)`, independently from its current state.
    fn transfer_contamination_from(&mut self, other: &Self) -> bool;

    /// Contaminate `self` from `other` if contamination is plausible.
    ///
    /// Return a boolean telling if contamination occurred.
    fn contaminate_from(&mut self, other: &Self) -> bool {
        if other.can_contaminate(self) {
            self.transfer_contamination_from(other);
            return true;
        }
        return false;
    }

    /// Return a copy of agent after interacting with other assuming that
    /// contamination should occur. This method return None if `other` is not able
    /// to contaminate `self`
    fn contaminated_from(&self, other: &Self) -> Option<Self> {
        if other.can_contaminate(self) {
            let mut new = self.clone();
            new.transfer_contamination_from(other);
            return Some(new);
        }
        return None;
    }

    /// Return true if agent has been contaminated with disease irrespective of
    /// the current clinical state.
    ///
    /// The default implementation simply negates self.is_susceptible().
    fn is_contaminated(&self) -> bool {
        !self.is_susceptible()
    }

    /// Return true if agent is susceptible to be contaminated with disease
    /// irrespective of the current clinical state.
    fn is_susceptible(&self) -> bool {
        self.index() == Self::S
    }

    /// Return true if agent is able to contaminate other agents. It must return
    /// true even if the probability of contamination is very low.
    fn is_contagious(&self) -> bool {
        self.contagion_odds() > 0.0
    }

    /// Return true if agent is recovered from disease.
    fn is_recovered(&self) -> bool;

    /// Return true if agent is dead from disease.
    fn is_dead(&self) -> bool {
        self.index() == Self::D
    }

    /// Return true if agent is alive. This is just a negation of is_dead and
    /// traits should generally implement only the former.
    fn is_alive(&self) -> bool {
        !self.is_dead()
    }
}

/// Create methods each_<comp>, each_<comp>_mut and n_<comp> for querying agents
/// in each compartment of an epidemiological model
macro_rules! compartment_methods {
    ($id:ident, for=$typ:ty) => {
        paste! {
            fn [<each_ $id>](&self, f: impl FnMut(Id, &Self::State))
            where
                Self::State: $typ,
            {
                let mut g = f;
                self.each_agent(&mut |id, st: &Self::State| {
                    if st.[<is_ $id>]() {
                        g(id, st)
                    }
                })
            }

            fn [<each_ $id _mut>](&mut self, f: impl FnMut(Id, &mut Self::State))
            where
                Self::State: $typ,
            {
                let mut g = f;
                self.each_agent_mut(&mut |id, st: &mut Self::State| {
                    if st.[<is_ $id>]() {
                        g(id, st)
                    }
                })
            }

            fn [<n_ $id>](&self) -> usize
            where
                Self::State: $typ,
            {
                let mut n = 0;
                self.each_agent(&mut |_, st|
                    if st.[<is_ $id>]() {
                        n += 1;
                    }
                );
                return n;
            }
        }
    };
}

/// A population with some epidemic model
pub trait EpiModelPopulationExt: Population {
    // Methods for SIR-based populations //////////////////////////////////////
    compartment_methods!(susceptible, for=EpiModel);
    compartment_methods!(contaminated, for=EpiModel);
    compartment_methods!(contagious, for=EpiModel);
    compartment_methods!(recovered, for=EpiModel);
    compartment_methods!(dead, for=EpiModel);
    compartment_methods!(infectious, for=SEIRLike);
    compartment_methods!(exposed, for=SEIRLike);
    compartment_methods!(asymptomatic, for=SEICHARLike);
    compartment_methods!(severe, for=SEICHARLike);
    compartment_methods!(critical, for=SEICHARLike);

    /// Return the fraction of population that is susceptible
    fn susceptible_ratio(&self) -> Real
    where
        Self::State: EpiModel,
    {
        self.n_susceptible() as Real / self.count() as Real
    }

    /// Return the fraction of population that is contaminated
    fn attack_ratio(&self) -> Real
    where
        Self::State: EpiModel,
    {
        self.n_contaminated() as Real / self.count() as Real
    }

    /// Count the number of (Susceptible, Infectious, Recovered, Total)
    /// individuals.
    fn count_sir(&self) -> [usize; 4]
    where
        Self::State: SEIRLike,
    {
        let (mut s, mut i, mut r, mut n) = (0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return [s, i, r, n];
    }

    /// Count the number of (Susceptible, Exposed, Infectious, Recovered, Total)
    /// individuals.
    fn count_seir(&self) -> [usize; 5]
    where
        Self::State: SEIRLike,
    {
        let (mut s, mut e, mut i, mut r, mut n) = (0, 0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_exposed() {
                e += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return [s, e, i, r, n];
    }

    /// Count the number of (Susceptible, Exposed, Asymptomatic, Infectious, Recovered, Total)
    /// individuals.
    fn count_seair(&self) -> [usize; 6]
    where
        Self::State: SEICHARLike,
    {
        let (mut s, mut e, mut a, mut i, mut r, mut n) = (0, 0, 0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_exposed() {
                e += 1;
            } else if st.is_asymptomatic() {
                a += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return [s, e, a, i, r, n];
    }

    /// Count the number of (Susceptible, Exposed, Infectious, Critical,
    /// Severe/Hospitalized, Asymptomatic, Recovered, Total) individuals.
    fn count_seichar(&self) -> [usize; 8]
    where
        Self::State: SEICHARLike,
    {
        let (mut s, mut e, mut i, mut c, mut h, mut a, mut r, mut n) = (0, 0, 0, 0, 0, 0, 0, 0);
        self.each_agent(&mut |_, st: &Self::State| {
            if st.is_susceptible() {
                s += 1;
            } else if st.is_exposed() {
                e += 1;
            } else if st.is_infectious() {
                i += 1;
            } else if st.is_critical() {
                c += 1;
            } else if st.is_severe() {
                h += 1;
            } else if st.is_asymptomatic() {
                a += 1;
            } else if st.is_recovered() {
                r += 1;
            }
            n += 1;
        });
        return [s, e, a, i, c, h, r, n];
    }

    /// Contaminate n individuals at random as if contaminated from given
    /// (possibly) infectious agent.
    fn contaminate_at_random_from<R: Rng>(
        &mut self,
        infectious: &Self::State,
        n: usize,
        only_susceptible: bool,
        rng: &mut R,
    ) -> &mut Self
    where
        Self::State: EpiModel,
    {
        let mut count = n;
        for _ in 0..5 {
            if count == 0 {
                return self;
            }
            self.map_randoms_mut(count, rng, |_, ag| {
                if !only_susceptible || ag.is_susceptible() {
                    count -= ag.transfer_contamination_from(infectious) as usize;
                }
            });
        }
        return self;
    }

    /// Contaminate n individuals at random with some variant
    fn contaminate_at_random<R: Rng>(
        &mut self,
        n: usize,
        only_susceptible: bool,
        rng: &mut R,
    ) -> &mut Self
    where
        Self::State: EpiModel,
        <Self::State as EpiModel>::Clinical: Default,
    {
        let infectious = Self::State::new_infectious();
        return self.contaminate_at_random_from(&infectious, n, only_susceptible, rng);
    }

    /// Contaminate n individuals at random with some specific clinical
    /// state/variant
    fn contaminate_at_random_with<R: Rng>(
        &mut self,
        clinical: &<Self::State as EpiModel>::Clinical,
        n: usize,
        only_susceptible: bool,
        rng: &mut R,
    ) -> &mut Self
    where
        Self::State: EpiModel,
    {
        let infectious = Self::State::new_infectious_with(clinical);
        return self.contaminate_at_random_from(&infectious, n, only_susceptible, rng);
    }

    /// Force all contaminated agents into an infectious state possibly even
    /// including dead elements.
    fn force_infectious(&mut self, force_dead: bool) -> &mut Self
    where
        Self::State: EpiModel,
    {
        self.each_agent_mut(|_, ag| {
            ag.force_infectious(force_dead);
        });
        return self;
    }

    /// FIXME: does it work? Is it better than the current strategy?
    fn _contaminate_at_random(
        &mut self,
        n: usize,
        rng: &mut impl Rng,
        f: impl Fn(usize, &mut Self::State) -> bool,
    ) -> usize
    where
        Self::State: EpiModel,
    {
        let from_list = |n, rng, pop: &mut Self| {
            let mut susceptibles = vec![];
            pop.each_agent(&mut |id, st| {
                if st.is_susceptible() {
                    susceptibles.push(id);
                }
            });
            (susceptibles.len() > n).then(|| susceptibles.shuffle(rng));

            let mut size = susceptibles.len().min(n);
            for (_, id) in (0..size).into_iter().zip(susceptibles) {
                pop.map_agent_mut(id, |st| {
                    if f(id, st) {
                        size -= 1;
                    }
                });
            }
            return size;
        };

        let mut cases = 0;
        let mut tries = 0;

        while cases < n {
            let (id, _) = self.random(rng);
            self.map_agent_mut(id, |st| {
                if st.is_susceptible() && f(id, st) {
                    cases += 1;
                }
            });

            tries += 1;
            if tries >= 3 * n && tries > 15 {
                return cases + from_list(n - cases, rng, self);
            }
        }
        return n;
    }
}

macro_rules! is_state {
    ($name:ident, index=$idx:ident) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool {
                self.index() == Self::$idx
            }
        }
    };
    ($name:ident) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool;
        }
    };
    ($name:ident, $expr:expr) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool { $expr }
        }
    };
    ($name:ident, |$var:ident| $expr:expr) => {
        paste! {
            #[doc = "Return true if agent is in the  `" $name "` state."]
            fn [<is_ $name>](&self) -> bool {
                let $var = self; $expr
             }
        }
    };
}

/// The classical Susceptible, Exposed, Infectious, Recovered model. This crate
/// also assumes a distinct Dead state which is usually grouped with Recovered
/// in the classical SIR model.
pub trait SEIRLike: EpiModel {
    const E: usize;
    const I: usize;
    const R: usize;

    is_state!(exposed, index = E);
    is_state!(infectious, index = I);

    /// Force exposure of agent to infection using the given clinical
    /// parameters.
    fn expose(&mut self, with: &Self::Clinical) {
        self.infect(with)
    }

    /// Force exposure of agent to infection using the default clinical
    /// parameters, if they exist.
    fn expose_default(&mut self)
    where
        Self::Clinical: Default,
    {
        self.expose(&Self::Clinical::default())
    }

    /// Force exposure of agent to infection using the given clinical
    /// parameters. Differently from expose() this method always puts agent
    /// into a infectious state.
    fn infect(&mut self, with: &Self::Clinical);

    /// Like infect(), but uses the default infection parameters.
    fn infect_default(&mut self)
    where
        Self::Clinical: Default,
    {
        self.infect(&Self::Clinical::default())
    }
}

/// Extends the SEIR with asymptomatic agents and a severity model in which
/// agents may require healthcare.
///
/// Asymptomatics do not develop symptoms and severity of disease never
/// aggravates, but they may transmit it to other agents (even if with a
/// reduced transmissibility).
///
/// SEICHAR consider severity levels:
///
/// Severe/Hospitalized:
///     Agents that require access to simple healthcare facilities.
/// Critical/ICU:
///     Cases that are severe enough to require intensive care.    
pub trait SEICHARLike: SEIRLike {
    const C: usize;
    const H: usize;
    const A: usize;
    is_state!(asymptomatic, index = A);
    is_state!(severe, index = H);
    is_state!(critical, index = C);
}

////////////////////////////////////////////////////////////////////////////////
// Trait implementations
////////////////////////////////////////////////////////////////////////////////
impl<T: HasEpiModel + Clone> EpiModel for T
where
    Self: Default,
    T::Model: EpiModel,
{
    const CARDINALITY: usize = T::Model::CARDINALITY;
    const CSV_HEADER: &'static str = T::Model::CSV_HEADER;
    const S: usize = T::Model::S;
    const D: usize = T::Model::D;
    type Disease = <<T as HasEpiModel>::Model as EpiModel>::Disease;
    type Clinical = <<T as HasEpiModel>::Model as EpiModel>::Clinical;

    default fn index(&self) -> usize {
        self.epimodel().index()
    }

    fn force_infectious(&mut self, force_dead: bool) -> bool {
        self.epimodel_mut().force_infectious(force_dead)
    }

    default fn new_infectious_with(clinical: &Self::Clinical) -> Self {
        let mut new = Self::default();
        new.set_epimodel(<Self as HasEpiModel>::Model::new_infectious_with(clinical));
        return new;
    }

    default fn is_contaminated(&self) -> bool {
        self.epimodel().is_contaminated()
    }

    default fn is_susceptible(&self) -> bool {
        self.epimodel().is_susceptible()
    }

    default fn is_contagious(&self) -> bool {
        self.epimodel().is_contagious()
    }

    default fn contagion_odds(&self) -> Real {
        self.epimodel().contagion_odds()
    }

    default fn can_contaminate(&self, other: &Self) -> bool {
        self.epimodel().can_contaminate(other.epimodel())
    }

    default fn contaminated_from(&self, other: &Self) -> Option<Self> {
        self.epimodel()
            .contaminated_from(other.epimodel())
            .map(|m| {
                let mut new = self.clone();
                new.set_epimodel(m);
                return new;
            })
    }

    default fn transfer_contamination_from(&mut self, other: &Self) -> bool {
        self.epimodel_mut()
            .transfer_contamination_from(other.epimodel())
    }

    default fn contaminate_from(&mut self, other: &Self) -> bool {
        self.epimodel_mut().contaminate_from(other.epimodel())
    }

    default fn is_dead(&self) -> bool {
        self.epimodel().is_dead()
    }

    default fn is_recovered(&self) -> bool {
        self.epimodel().is_recovered()
    }

    default fn is_alive(&self) -> bool {
        self.epimodel().is_alive()
    }
}

impl<P> EpiModelPopulationExt for P
where
    P: Population,
    P::State: EpiModel,
{
}
