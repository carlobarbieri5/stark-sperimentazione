# Reproducibility: the exact version behind the document

This file pins the campaign run that the thesis figures and tables are computed from, so a
reader can verify that the published numbers come from a specific execution rather than only
re-running everything from scratch.

## Released version

- **Git tag:** `campagna-sperimentale-v1.0` (annotated). The tagged commit is the version
  used for the document; `git show campagna-sperimentale-v1.0` gives its commit hash and date.
- **Dataset checksums** (SHA-256):
  - `data/results.csv`          → `940930ab1636ed0d3368ec59e2241255832799739035b31416443fe29a638d25`
  - `data/raw_measurements.csv` → `e3c84827f066bd4bfcd544db236ed5ae789b0bb71a0f921cb6f78f055f8e0d06`
  - `data/failures.csv`         → `4c636f600f91ee8509dd6d8b60488572adb149bf425c2be3a2799895f083ffad`

  Verify with `sha256sum data/results.csv` (or `Get-FileHash data/results.csv`). The figures
  in the document are derived only from `data/results.csv`.

## Execution environment

Captured automatically at campaign time in `data/ENV.txt`:

- Winterfell `v0.13.1` (crates.io, pinned `winterfell = "=0.13.1"`, no local patches)
- `rustc 1.96.0`, `cargo 1.96.0`, profile `release`, `opt-level = 3`
- AMD Ryzen 7 PRO 7840U (8 physical / 16 logical cores), 30.7 GB RAM
- Windows 11 Pro 10.0.26200
- 7 timed repetitions per configuration (+ 1 discarded warm-up)

## Exact commands

From the repository root:

```powershell
# 1. Full campaign -> data/results.csv, failures.csv, run_log.txt, ENV.txt
pwsh scripts/run_campaign.ps1

# 2. Aggregate raw rows -> figure data blocks + per-config summary
python scripts/analyze.py

# 3. (optional) Trace-construction cost probe -> data/trace_cost.csv
cargo run --release --example trace_cost

# 4. (optional) Reproduce the Rp64_256 f64-vs-f128 compile error
cargo build --example rp64_256_field_mismatch --features demo_broken
```

`data/run_log.txt` contains the exact per-configuration command line (with timestamps) for
every one of the 25 invocations. A single configuration can be reproduced directly, e.g.:

```sh
target/release/stark_campaign --n 65536 --commit-hash blake3_256 --num-queries 32 \
  --blowup-factor 8 --grinding-factor 16 --field-extension none --fri-folding-factor 8 \
  --threads 1 --repetitions 7 --sweep S1 --machine laptop_ryzen7_7840u_32gb
```

## Determinism note

At `threads=1` the pipeline is deterministic (fixed seed, deterministic Fiat–Shamir and
grinding), so `proof_bytes` and `security_bits` are identical across all 7 repetitions and
reproducible across machines; only wall-clock timings vary (and carry thermal noise on a
single laptop). The 16-thread pass (`MT`) is the sole source of proof nondeterminism, because
parallel grinding finds a different valid nonce per run.
