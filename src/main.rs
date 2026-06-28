// STARK measurement-campaign harness (winterfell 0.13.1).
//
// One invocation = one configuration. Builds the Rescue hash-chain trace for `n`,
// measures the native chain (T_C baseline), then runs prove()/verify() `repetitions`
// times (plus one discarded warm-up) and prints one CSV row per repetition to stdout
// (no header). The PowerShell orchestrator assembles results.csv / failures.csv.
//
// Workload: the VERBATIM winterfell rescue example AIR (f128 base field).
// Backends: Blake3_256, Blake3_192, Sha3_256 (Rp64_256 is f64-only -> incompatible).

use std::time::Instant;

use clap::Parser;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher, MerkleTree},
    math::fields::f128::BaseElement,
    AcceptableOptions, BatchingMethod, FieldExtension, ProofOptions, Prover,
};

use stark_campaign::rescue::{self, PublicInputs, RescueAir, RescueProver};

// Hash backends, monomorphized over the f128 base field of the AIR.
type Blake3_256 = winterfell::crypto::hashers::Blake3_256<BaseElement>;
type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;
type Sha3_256 = winterfell::crypto::hashers::Sha3_256<BaseElement>;

#[derive(Parser, Debug)]
#[command(about = "STARK measurement-campaign harness (Rescue hash chain, winterfell 0.13.1)")]
struct Args {
    /// Workload size: number of Rescue hash-chain steps (must be a power of two).
    #[arg(long)]
    n: usize,
    /// Commitment / Fiat-Shamir hash backend: blake3_256 | blake3_192 | sha3_256.
    #[arg(long)]
    commit_hash: String,
    #[arg(long)]
    num_queries: usize,
    #[arg(long)]
    blowup_factor: usize,
    #[arg(long)]
    grinding_factor: u32,
    /// Field extension for the composition polynomial: none | quad.
    #[arg(long)]
    field_extension: String,
    #[arg(long)]
    fri_folding_factor: usize,
    /// Number of threads for the Rayon global pool (controls the prover's parallelism).
    #[arg(long)]
    threads: usize,
    /// Timed repetitions (one extra warm-up run is always discarded first).
    #[arg(long)]
    repetitions: usize,
    /// Sweep tag (S1..S6, MT) echoed into the CSV.
    #[arg(long)]
    sweep: String,
    /// Short machine description echoed into the CSV.
    #[arg(long)]
    machine: String,
}

fn main() {
    let args = Args::parse();

    // Configure the Rayon global pool; this governs winterfell's `concurrent` parallelism.
    // build_global() fails if the pool is already initialized -> safe to ignore.
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global();

    let field_extension = match args.field_extension.as_str() {
        "none" => FieldExtension::None,
        "quad" => FieldExtension::Quadratic,
        other => {
            eprintln!("invalid field_extension '{other}' (expected none|quad)");
            std::process::exit(2);
        }
    };

    let options = ProofOptions::new(
        args.num_queries,
        args.blowup_factor,
        args.grinding_factor,
        field_extension,
        args.fri_folding_factor,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    );

    match args.commit_hash.as_str() {
        "blake3_256" => run::<Blake3_256>(&args, options),
        "blake3_192" => run::<Blake3_192>(&args, options),
        "sha3_256" => run::<Sha3_256>(&args, options),
        other => {
            eprintln!("unsupported commit_hash '{other}' (Rp64_256 is f64-only; AIR is f128)");
            std::process::exit(3);
        }
    }
}

fn lambda_bits(commit_hash: &str) -> u32 {
    match commit_hash {
        "blake3_192" => 192,
        _ => 256,
    }
}

fn run<H>(args: &Args, options: ProofOptions)
where
    H: ElementHasher<BaseField = BaseElement> + Sync,
{
    let seed = [BaseElement::from(42u8), BaseElement::from(43u8)];
    let lambda = lambda_bits(&args.commit_hash);
    let fe = args.field_extension.as_str();

    // T_C: native hash chain with no proving. Measured ONCE per invocation (brief step 2);
    // the single value is reported in every row. `native_res` is the public output.
    let t0 = Instant::now();
    let native_res = rescue::compute_hash_chain(seed, args.n);
    let t_native_ms = t0.elapsed().as_secs_f64() * 1000.0;

    // ---- discarded warm-up (prove + verify) ----
    {
        let prover = RescueProver::<H>::new(options.clone());
        let trace = prover.build_trace(seed, args.n);
        let proof = prover.prove(trace).expect("warm-up prove failed");
        let pub_inputs = PublicInputs { seed, result: native_res };
        let acceptable = AcceptableOptions::OptionSet(vec![options.clone()]);
        winterfell::verify::<RescueAir, H, DefaultRandomCoin<H>, MerkleTree<H>>(
            proof,
            pub_inputs,
            &acceptable,
        )
        .expect("warm-up verify failed");
    }

    // ---- timed repetitions ----
    for run_index in 1..=args.repetitions {
        // Build the execution trace (not part of the prove timing).
        let prover = RescueProver::<H>::new(options.clone());
        let trace = prover.build_trace(seed, args.n);

        // T_P: prove.
        let t1 = Instant::now();
        let proof = prover.prove(trace).expect("prove failed");
        let t_prove_ms = t1.elapsed().as_secs_f64() * 1000.0;

        let proof_bytes = proof.to_bytes().len();
        let security_bits = proof.conjectured_security::<H>().bits();

        // T_V: verify (always single context; verifier is single-threaded by nature).
        let pub_inputs = PublicInputs { seed, result: native_res };
        let acceptable = AcceptableOptions::OptionSet(vec![options.clone()]);
        let t2 = Instant::now();
        winterfell::verify::<RescueAir, H, DefaultRandomCoin<H>, MerkleTree<H>>(
            proof,
            pub_inputs,
            &acceptable,
        )
        .expect("verify failed");
        let t_verify_ms = t2.elapsed().as_secs_f64() * 1000.0;

        // One CSV row. Column order MUST match results.csv header:
        // sweep,workload,n,commit_hash,lambda_bits,num_queries,blowup_factor,grinding_factor,
        // field_extension,fri_folding_factor,threads,machine,run_index,
        // t_native_ms,t_prove_ms,t_verify_ms,proof_bytes,security_bits
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{},{}",
            args.sweep,
            "rescue_chain",
            args.n,
            args.commit_hash,
            lambda,
            args.num_queries,
            args.blowup_factor,
            args.grinding_factor,
            fe,
            args.fri_folding_factor,
            args.threads,
            args.machine,
            run_index,
            t_native_ms,
            t_prove_ms,
            t_verify_ms,
            proof_bytes,
            security_bits,
        );
    }
}
