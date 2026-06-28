// Rp64_256 field-incompatibility demonstration (winterfell 0.13.1).
//
// This example is INTENTIONALLY non-compiling. It exists to make the paper's claim
// (Section 4.5, "The arithmetization-friendly hash that could not be measured")
// reproducible: the Rescue-Prime `Rp64_256` hash is defined ONLY over the 64-bit
// Goldilocks field (`ElementHasher<BaseField = f64::BaseElement>`), whereas the
// workload AIR is over the 128-bit prime field `f128`. The `Prover` impl for
// `RescueProver<H>` requires `H: ElementHasher<BaseField = f128::BaseElement>`, a bound
// `Rp64_256` cannot satisfy, so `prover.prove(trace)` fails to type-check (E0599: the
// method exists but its `Prover` trait bounds are not satisfied).
//
// It is gated behind the `demo_broken` feature so that `cargo build`, `cargo build
// --examples` and `cargo test` stay green. Reproduce the compiler error with:
//
//   cargo build --example rp64_256_field_mismatch --features demo_broken
//
// The captured rustc output is committed at docs/rp64_256_compile_error.txt.

use winterfell::{
    crypto::hashers::Rp64_256, math::fields::f128::BaseElement, BatchingMethod, FieldExtension,
    ProofOptions, Prover,
};

use stark_campaign::rescue::RescueProver;

fn main() {
    let options = ProofOptions::new(
        32,
        8,
        16,
        FieldExtension::None,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    );

    let seed = [BaseElement::from(42u8), BaseElement::from(43u8)];

    // `RescueProver::<Rp64_256>` and `build_trace` only need `H: ElementHasher`, so they
    // compile; the incompatibility surfaces at `prove`, whose `Prover` impl requires
    // `H: ElementHasher<BaseField = f128::BaseElement>` -- not satisfied by the f64-only
    // `Rp64_256`. This is the f64-vs-f128 mismatch described in Section 4.5.
    let prover = RescueProver::<Rp64_256>::new(options);
    let trace = prover.build_trace(seed, 1024);
    let _proof = prover.prove(trace).expect("unreachable: this example never compiles");
}
