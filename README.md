# STARK measurement campaign

Measurement harness for a transparent STARK prover/verifier, built on
[Winterfell](https://github.com/facebook/winterfell) `0.13.1`. It runs a fixed
workload — a **Rescue hash-chain** AIR over the 128-bit field `f128`
(modulus `2^128 − 45·2^40 + 1`) — and records prover time, verifier time, proof
size and conjectured security across a sweep of proof parameters.

This is the experimental code for a bachelor's thesis on the post-quantum
security of Merkle-based transparent proof systems.

## What it measures

Each invocation runs **one** configuration: it builds the trace for `n` steps,
times the native hash chain once (the `T_C` baseline), then runs
`prove()` / `verify()` for `repetitions` timed runs (plus one discarded warm-up)
and prints one CSV row per run to stdout. Swept parameters:

| Parameter | Flag | Meaning |
|---|---|---|
| Workload size | `--n` | Rescue hash-chain steps (power of two) |
| Hash backend | `--commit-hash` | `blake3_256` \| `blake3_192` \| `sha3_256` (Merkle + Fiat–Shamir) |
| Queries | `--num-queries` | FRI query repetitions |
| Blowup | `--blowup-factor` | Reed–Solomon rate (LDE expansion) |
| Grinding | `--grinding-factor` | proof-of-work nonce bits |
| Field extension | `--field-extension` | `none` \| `quad` (composition poly) |
| FRI folding | `--fri-folding-factor` | layer folding factor |
| Threads | `--threads` | Rayon pool size (prover parallelism) |
| Repetitions | `--repetitions` | timed runs (1 warm-up always discarded) |

`Rp64_256` is intentionally unsupported: it is an `f64`-only hash, incompatible
with the `f128` AIR (the type error is reproduced by the `rp64_256_field_mismatch`
example — see *Methodological probes* below).

## Build

Rust stable, edition 2021.

```sh
cargo build --release
```

## Run one configuration

```sh
./target/release/stark_campaign \
  --n 65536 --commit-hash blake3_256 --num-queries 32 \
  --blowup-factor 8 --grinding-factor 16 --field-extension none \
  --fri-folding-factor 8 --threads 1 --repetitions 1 \
  --sweep S0 --machine laptop_ryzen7_7840u_32gb
```

## Run the full campaign

The PowerShell orchestrator runs every configuration, handles timeouts/OOMs, and
collects the results into `data/`:

```powershell
pwsh scripts/run_campaign.ps1
```

Outputs are written to `data/` and tracked in this repository: `results.csv` (one row per
run), `failures.csv` (errors/timeouts with reason), `run_log.txt`, `ENV.txt`.

## Analyse the raw data (medians / figures)

`scripts/analyze.py` is the link between the raw `results.csv` and the published numbers: it
computes the per-configuration medians / minima / maxima and re-derives the pgfplots data
blocks (`scaling`, `queries`, `digest`, `grinding`, `blowup`) embedded in the paper, plus the
derived comparisons quoted in §4.4. Pure standard library (no pandas):

```sh
python scripts/analyze.py
```

It writes `data/summary.csv` and `data/figures/*.dat`, and prints the blocks to stdout for a
direct comparison against the paper's embedded figure data (the thesis document is delivered
separately).

## Methodological probes

- **Trace-construction cost** (`examples/trace_cost.rs`): proving time `t_prove` excludes
  trace construction (`build_trace` runs before the timer). This probe times `build_trace`
  against `prove` on the light instances and writes `data/trace_cost.csv`; it shows trace
  construction is a small, shrinking fraction of `prove`. Run: `cargo run --release --example
  trace_cost`.
- **Rp64_256 field mismatch** (`examples/rp64_256_field_mismatch.rs`): an intentionally
  non-compiling example that reproduces the `f64`-vs-`f128` type error behind the Rp64_256
  exclusion. Gated behind the `demo_broken` feature so it does not break normal builds. Run
  `cargo build --example rp64_256_field_mismatch --features demo_broken`; the captured
  compiler error is committed at `docs/rp64_256_compile_error.txt`.

See `docs/REPRODUCIBILITY.md` for the exact campaign recipe and dataset checksums, and
`docs/PROVENANCE.md` for the development history and the raw → published chain.

## Output columns

`results.csv` opens with a header row written by the campaign script; the
harness binary itself prints only data rows (one per run). Columns:

```
sweep, workload, n, commit_hash, lambda_bits, num_queries, blowup_factor,
grinding_factor, field_extension, fri_folding_factor, threads, machine,
run_index, t_native_ms, t_prove_ms, t_verify_ms, proof_bytes, security_bits
```

`security_bits` is the conjectured security from
`proof.conjectured_security::<H>().bits()`. Nothing is averaged in the CSV — it
holds raw per-run rows; medians/spreads are computed downstream.

## Environment

Measurements in the thesis were taken on an AMD Ryzen 7 PRO 7840U
(8 cores / 16 threads), 32 GB RAM, Windows 11, `winterfell = "=0.13.1"` from
crates.io (no local patches).

## Attribution

The Rescue AIR, prover, hash permutation and constraint helpers under
`src/rescue/` and `src/utils.rs` are the Winterfell `examples/src/rescue` files
at tag `v0.13.1`, copied verbatim or lightly adapted; they keep their original
`Copyright (c) Facebook, Inc.` MIT headers. Original work in this repository:
the measurement harness (`src/main.rs`, `src/lib.rs`), the campaign orchestrator
(`scripts/run_campaign.ps1`), the analysis script (`scripts/analyze.py`), and the
methodological probes (`examples/`). See [LICENSE](LICENSE).

## License

MIT — see [LICENSE](LICENSE).
