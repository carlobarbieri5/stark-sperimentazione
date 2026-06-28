// Trace-construction cost probe (winterfell 0.13.1).
//
// The main campaign harness (`src/main.rs`) measures T_P = `prover.prove(trace)` ONLY:
// `build_trace` runs before the timer and is therefore EXCLUDED from T_P (and from the
// native baseline T_C). This probe quantifies that excluded step on the LIGHT instances
// (n up to 2^16, sub-second each), so the paper's methodological note can report it
// without re-running the heavy n = 2^18 / 2^20 configurations.
//
// `build_trace` is the same Rescue hash chain as the native baseline plus the work of
// writing each state into the trace table, so it is O(n) -- like T_C -- whereas
// `prove()` is O(n log n). The build/prove fraction therefore does not grow with n.
//
// Run from the repo root (writes data/trace_cost.csv):
//   cargo run --release --example trace_cost

use std::fs;
use std::time::Instant;

use winterfell::{
    math::fields::f128::BaseElement, BatchingMethod, FieldExtension, ProofOptions, Prover,
};

use stark_campaign::rescue::RescueProver;

type Blake3_256 = winterfell::crypto::hashers::Blake3_256<BaseElement>;

// Light instances only: each builds/proves in well under a second, so this probe is cheap.
const NS: [usize; 4] = [1024, 4096, 16384, 65536];
const REPS: usize = 5; // timed repetitions (median reported), plus one discarded warm-up

fn baseline_options() -> ProofOptions {
    // Same baseline tuple as the campaign (src/main.rs): q=32, blowup=8, grinding=16,
    // FieldExtension::None, FRI folding=8, remainder max degree=31, Linear batching.
    ProofOptions::new(
        32,
        8,
        16,
        FieldExtension::None,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2]
    } else {
        0.5 * (xs[n / 2 - 1] + xs[n / 2])
    }
}

fn main() {
    // Pin to a single thread, matching the campaign's sweep configuration (threads = 1).
    let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();

    let seed = [BaseElement::from(42u8), BaseElement::from(43u8)];

    let mut csv = String::from(
        "n,t_build_ms_median,t_prove_ms_median,build_frac,t_build_ms_min,t_build_ms_max\n",
    );
    println!(
        "{:>8}  {:>14}  {:>14}  {:>10}",
        "n", "build_ms(med)", "prove_ms(med)", "build_frac"
    );

    for &n in &NS {
        // discarded warm-up
        {
            let prover = RescueProver::<Blake3_256>::new(baseline_options());
            let trace = prover.build_trace(seed, n);
            let _ = prover.prove(trace).expect("warm-up prove failed");
        }

        let mut builds = Vec::with_capacity(REPS);
        let mut proves = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            let prover = RescueProver::<Blake3_256>::new(baseline_options());

            let t0 = Instant::now();
            let trace = prover.build_trace(seed, n);
            builds.push(t0.elapsed().as_secs_f64() * 1000.0);

            let t1 = Instant::now();
            let proof = prover.prove(trace).expect("prove failed");
            proves.push(t1.elapsed().as_secs_f64() * 1000.0);
            let _ = proof; // keep the proof alive until after timing
        }

        let b_med = median(builds.clone());
        let p_med = median(proves.clone());
        let b_min = builds.iter().cloned().fold(f64::INFINITY, f64::min);
        let b_max = builds.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let frac = b_med / p_med;

        println!("{n:>8}  {b_med:>14.3}  {p_med:>14.3}  {frac:>10.4}");
        csv.push_str(&format!(
            "{n},{b_med:.6},{p_med:.6},{frac:.6},{b_min:.6},{b_max:.6}\n"
        ));
    }

    fs::write("data/trace_cost.csv", &csv).expect("could not write data/trace_cost.csv");
    println!("\nWrote data/trace_cost.csv");
}
