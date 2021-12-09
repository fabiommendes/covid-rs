use crate::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;

/// Sample n ages from a non-empty vector of probabilities given as pairs
/// (age_group, prob). Probabilities do not need to be normalized and age
/// groups are iterpreted the range from the given number to the next value.
///
/// The last age group is assumed to have the same size as the penultimate one.
pub fn random_ages(n: usize, rng: &mut impl Rng, probs: AgeDistribution10) -> Vec<Age> {
    let distrib = WeightedIndex::new(&probs).unwrap();
    return (0..n)
        .map(|_| (10 * distrib.sample(rng) + rng.gen_range(0..10)) as Age)
        .collect();
}

/// Compute R0 from iterator over agents and the number of secondary infections
/// produced by each agent.
pub fn r0<M: EpiModel>(it: impl IntoIterator<Item = (usize, M)>) -> Real {
    let mut total = 0;
    let mut acc = 0;
    for pair in it {
        match pair {
            (0, st) => {
                if st.is_recovered() {
                    total += 1;
                }
            }
            (n, _) => {
                total += 1;
                acc += n;
            }
        }
    }
    return (acc as Real) / (total as Real);
}

/// Default random number generator
pub fn default_rng() -> SmallRng {
    SmallRng::from_entropy()
}

/// Default random number generator with numeric seed
pub fn seeded_rng(n: impl Into<u64>) -> SmallRng {
    SmallRng::seed_from_u64(n.into())
}
