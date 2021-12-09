use crate::prelude::{AgeDistribution10, Real};


///////////////////////////////////////////////////////////////////////////////
// Default param for COVID-19
///////////////////////////////////////////////////////////////////////////////

pub const PROB_ASYMPTOMATIC: Real = 0.42;
pub const PROB_SEVERE: Real = 0.18;
pub const PROB_CRITICAL: Real = 0.22;
pub const PROB_DEATH: Real = 0.49; // = CFR / (PROB_SEVERE * PROB_CRITICAL)
pub const CASE_FATALITY_RATIO: Real = PROB_SEVERE * PROB_CRITICAL * PROB_DEATH;
pub const INFECTION_FATALITY_RATIO: Real = CASE_FATALITY_RATIO * PROB_ASYMPTOMATIC;
pub const ASYMPTOMATIC_INFECTIOUSNESS: Real = 0.50;
pub const INCUBATION_PERIOD: Real = 3.69;
pub const INFECTIOUS_PERIOD: Real = 3.47;
pub const SEVERE_PERIOD: Real = 7.19;
pub const CRITICAL_PERIOD: Real = 17.50 - 7.19;

// Distributions
pub const PROB_ASYMPTOMATIC_DISTRIBUTION: AgeDistribution10 = [
    0.619231, 0.469595, 0.515000, 0.578082, 0.545763, 0.476000, 0.483709, 0.497096, 0.582090,
];
pub const PROB_SEVERE_DISTRIBUTION: AgeDistribution10 = [
    0.000053, 0.000869, 0.020194, 0.059334, 0.077873, 0.171429, 0.243948, 0.333939, 0.316103,
];
pub const PROB_CRITICAL_DISTRIBUTION: AgeDistribution10 = [
    0.500000, 0.347639, 0.060636, 0.050217, 0.077311, 0.148810, 0.333795, 0.526186, 0.865129,
];
pub const PROB_DEATH_DISTRIBUTION: AgeDistribution10 = [PROB_DEATH; 9];
pub const CASE_FATALITY_RATIO_DISTRIBUTION: AgeDistribution10 = [
    0.000026, 0.000148, 0.000600, 0.001460, 0.002950, 0.012500, 0.039900, 0.086100, 0.134000,
];
pub const INFECTION_FATALITY_RATIO_DISTRIBUTION: AgeDistribution10 = [
    0.000016, 0.000069, 0.000309, 0.000844, 0.001610, 0.005950, 0.019300, 0.042800, 0.078000,
];
pub const ASYMPTOMATIC_INFECTIOUSNESS_DISTRIBUTION: AgeDistribution10 = [0.50; 9];
pub const INCUBATION_PERIOD_DISTRIBUTION: AgeDistribution10 = [3.69; 9];
pub const INFECTIOUS_PERIOD_DISTRIBUTION: AgeDistribution10 = [3.47; 9];
pub const SEVERE_PERIOD_DISTRIBUTION: AgeDistribution10 = [7.19; 9];
pub const CRITICAL_PERIOD_DISTRIBUTION: AgeDistribution10 = [17.50 - 7.19; 9];