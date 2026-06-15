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
with the `f128` AIR.

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

Outputs (written to `data/`, not tracked by git): `results.csv` (one row per run),
`failures.csv` (errors/timeouts with reason), `run_log.txt`, `ENV.txt`.

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
the measurement harness (`src/main.rs`) and the campaign orchestrator
(`scripts/run_campaign.ps1`). See [LICENSE](LICENSE).

## License

MIT — see [LICENSE](LICENSE).
