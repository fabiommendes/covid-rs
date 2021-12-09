#![feature(
    min_specialization,
    auto_traits,
    negative_impls,
)]
pub mod trackers;
pub mod epidemic;
pub mod models;
pub mod params;
pub mod prelude;
pub mod sampler;
pub mod sim;
pub mod utils;
pub mod config;