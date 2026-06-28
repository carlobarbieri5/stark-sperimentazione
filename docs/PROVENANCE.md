# Provenance and development history

This repository is published as a clean **final release** of the measurement campaign that
backs the bachelor's thesis *Post-Quantum Security of Merkle-Based Transparent Proof
Systems*. Its Git history is intentionally short; this note records where the code, the
workload, and the published numbers come from, so that the release can be audited without a
long commit trail.

## Where the workload comes from

The proving workload is **not original**. The Rescue–Prime AIR, prover, and hash
permutation under `src/rescue/` plus the constraint helpers in `src/utils.rs` are the
upstream **Winterfell `examples/src/rescue`** sources at tag **`v0.13.1`**, copied verbatim
or lightly adapted (only the example CLI / `Example`-trait glue was stripped, and the native
`compute_hash_chain` baseline was kept for the $T_C$ measurement). They retain their original
`Copyright (c) Facebook, Inc.` MIT headers. A reference copy of the upstream files used
during development is kept locally under `.archive/_vendor_ref/` (not tracked).

Original work in this repository:
- `src/main.rs` — the measurement harness (timing, CSV emission, parameter parsing);
- `src/lib.rs` — exposes the workload to the harness and the auxiliary examples;
- `scripts/run_campaign.ps1` — the campaign orchestrator;
- `scripts/analyze.py` — the raw → published aggregation (see below);
- `examples/trace_cost.rs`, `examples/rp64_256_field_mismatch.rs` — the two methodological
  probes added for reproducibility.

## How the published numbers were produced

The chain from raw measurements to the figures and tables is fully in-repository:

1. **Raw data.** `scripts/run_campaign.ps1` builds the release binary and runs every
   configuration, writing one row per timed repetition to `data/results.csv` (175 rows: 7
   repetitions × 25 configurations), plus `data/failures.csv`, `data/run_log.txt`,
   `data/ENV.txt`. `data/raw_measurements.csv` is a reshaped per-run export of the same data.
   Nothing is averaged or fabricated in these files.
2. **Aggregation.** `scripts/analyze.py` reads `data/results.csv`, computes the
   per-configuration medians / minima / maxima, and re-derives the pgfplots data blocks
   (`scaling.dat`, `queries.dat`, `digest.dat`, `grinding.dat`, `blowup.dat`) and the derived
   comparisons quoted in §4.4. Its output matches the `filecontents*` data blocks used by the
   paper figures.
3. **Document.** The thesis LaTeX source (`paper/paper_full.tex`, tracked here; only the
   compiled PDF is delivered separately) draws the figures with pgfplots directly from those
   embedded data blocks; the generated figures it produces are kept under `paper/figures/`.

During development the data passed through earlier drafts retained locally (e.g. an earlier
`results_chapter.tex`, field-correction snippets, a draft `.docx`/`.pdf`); these intermediate
artifacts are kept under `.archive/` and are not part of the release. The substantive history
that matters for reproducibility — raw data, environment, exact commands, and the aggregation
script — is all tracked. See `docs/REPRODUCIBILITY.md` for the exact run recipe and the
dataset checksum.
