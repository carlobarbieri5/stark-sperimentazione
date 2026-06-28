// STARK measurement-campaign library (winterfell 0.13.1).
//
// Exposes the Rescue hash-chain workload (the verbatim winterfell `examples/rescue`
// AIR/prover/permutation over the f128 base field) plus the native `compute_hash_chain`
// baseline, so that the measurement binary (`src/main.rs`) and the auxiliary examples
// (`examples/`) share one definition of the workload.

pub mod rescue;
pub mod utils;
