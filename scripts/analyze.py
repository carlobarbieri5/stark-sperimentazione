#!/usr/bin/env python3
"""Aggregate the raw STARK campaign data into the published figures and tables.

This is the missing link between data/results.csv (175 raw per-run rows, 7 timed
repetitions per configuration) and the numbers that appear in the paper: it computes the
per-configuration medians / minima / maxima and re-derives, byte-for-byte, the pgfplots
data blocks used by the paper figures (the `filecontents*` blocks for
scaling.dat / queries.dat / digest.dat / grinding.dat / blowup.dat) and the Results
tables. Run it and diff its output against the paper to verify the raw -> published step.
(The thesis document itself is delivered separately and is not tracked in this repository.)

Pure standard library (no pandas), so it runs on a stock Python 3.

Usage (from the repository root):
    python scripts/analyze.py

It reads   data/results.csv
and writes data/figures/{scaling,queries,digest,grinding,blowup}.dat
           data/summary.csv          (per-configuration median/min/max, all sweeps)
while printing the .dat blocks and the derived comparisons to stdout.

Times in results.csv are milliseconds; the figures report seconds for prove/native and
milliseconds for verify, exactly as the paper does.
"""

import csv
import os
import statistics
from collections import OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RESULTS = os.path.join(ROOT, "data", "results.csv")
FIG_DIR = os.path.join(ROOT, "data", "figures")
SUMMARY = os.path.join(ROOT, "data", "summary.csv")

# Configuration key: everything that identifies one swept point (run_index excluded).
KEY_FIELDS = (
    "sweep", "n", "commit_hash", "lambda_bits", "num_queries", "blowup_factor",
    "grinding_factor", "field_extension", "fri_folding_factor", "threads",
)


def load_groups():
    """Return OrderedDict[key_tuple] -> dict of aggregated stats for that configuration."""
    rows_by_key = OrderedDict()
    with open(RESULTS, newline="") as f:
        for row in csv.DictReader(f):
            key = tuple(row[k] for k in KEY_FIELDS)
            rows_by_key.setdefault(key, []).append(row)

    groups = OrderedDict()
    for key, rows in rows_by_key.items():
        prove = [float(r["t_prove_ms"]) for r in rows]
        verify = [float(r["t_verify_ms"]) for r in rows]
        native = {float(r["t_native_ms"]) for r in rows}
        proof = {int(r["proof_bytes"]) for r in rows}
        sec = {int(r["security_bits"]) for r in rows}
        groups[key] = {
            "fields": dict(zip(KEY_FIELDS, key)),
            "reps": len(rows),
            # native time T_C is measured ONCE per configuration and repeated in every row
            "native_ms": next(iter(native)),
            "native_constant": len(native) == 1,
            "prove_med_ms": statistics.median(prove),
            "prove_min_ms": min(prove),
            "prove_max_ms": max(prove),
            "verify_med_ms": statistics.median(verify),
            # proof_bytes / security are constant per config at threads=1 (deterministic);
            # under parallel grinding (MT) proof_bytes varies, so keep the median there.
            "proof_bytes": statistics.median(sorted(proof)) if len(proof) > 1
            else next(iter(proof)),
            "proof_constant": len(proof) == 1,
            "security_bits": next(iter(sec)),
        }
    return groups


def r3(x):
    """Round to 3 decimals (seconds), matching the paper's data blocks."""
    return round(x, 3)


def write_block(lines, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", newline="\n") as f:
        f.write("\n".join(lines) + "\n")


def fmt_table(header, rows):
    print(header)
    for row in rows:
        print(row)
    print()


def main():
    groups = load_groups()

    # --- per-configuration summary (covers every Results table) -----------------------
    os.makedirs(os.path.dirname(SUMMARY), exist_ok=True)
    with open(SUMMARY, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(list(KEY_FIELDS) + [
            "reps", "native_ms", "prove_med_ms", "prove_min_ms", "prove_max_ms",
            "verify_med_ms", "proof_bytes", "security_bits",
        ])
        for key, g in groups.items():
            w.writerow(list(key) + [
                g["reps"], f"{g['native_ms']:.6f}", f"{g['prove_med_ms']:.6f}",
                f"{g['prove_min_ms']:.6f}", f"{g['prove_max_ms']:.6f}",
                f"{g['verify_med_ms']:.6f}", g["proof_bytes"], g["security_bits"],
            ])

    def by_sweep(sweep, extra=lambda g: True):
        return [g for g in groups.values()
                if g["fields"]["sweep"] == sweep and extra(g)]

    # ---- scaling.dat (sweep S1) ------------------------------------------------------
    scaling = sorted(by_sweep("S1"), key=lambda g: int(g["fields"]["n"]))
    lines = ["n        TC        TP        TPep       TPem      ratio  proofKiB  verify  sec"]
    for g in scaling:
        n = int(g["fields"]["n"])
        tc = r3(g["native_ms"] / 1000)
        tp = r3(g["prove_med_ms"] / 1000)
        tp_hi = r3(g["prove_max_ms"] / 1000)
        tp_lo = r3(g["prove_min_ms"] / 1000)
        tpep = round(tp_hi - tp, 3)
        tpem = round(tp - tp_lo, 3)
        ratio = round(tp / tc, 2)
        proof_kib = round(g["proof_bytes"] / 1024, 2)
        verify = r3(g["verify_med_ms"])
        lines.append(f"{n:<8} {tc:<9} {tp:<9} {tpep:<10} {tpem:<9} {ratio:<6.2f} "
                     f"{proof_kib:<9} {verify:<7} {g['security_bits']}")
    write_block(lines, os.path.join(FIG_DIR, "scaling.dat"))
    fmt_table("=== scaling.dat (S1) ===", lines)

    # ---- queries.dat (sweep S4) ------------------------------------------------------
    queries = sorted(by_sweep("S4"), key=lambda g: int(g["fields"]["num_queries"]))
    lines = ["q    sec   proofKiB   TP       verify"]
    for g in queries:
        q = int(g["fields"]["num_queries"])
        lines.append(f"{q:<4} {g['security_bits']:<5} {round(g['proof_bytes']/1024,2):<10} "
                     f"{r3(g['prove_med_ms']/1000):<8} {r3(g['verify_med_ms'])}")
    write_block(lines, os.path.join(FIG_DIR, "queries.dat"))
    fmt_table("=== queries.dat (S4) ===", lines)

    # ---- digest.dat (sweep S3) -------------------------------------------------------
    digest = sorted(by_sweep("S3"), key=lambda g: int(g["fields"]["lambda_bits"]))
    lines = ["lambda  sec   proofKiB"]
    for g in digest:
        lam = int(g["fields"]["lambda_bits"])
        lines.append(f"{lam:<7} {g['security_bits']:<5} {round(g['proof_bytes']/1024,2)}")
    write_block(lines, os.path.join(FIG_DIR, "digest.dat"))
    fmt_table("=== digest.dat (S3) ===", lines)

    # ---- grinding.dat (sweep S5) -----------------------------------------------------
    grinding = sorted(by_sweep("S5"), key=lambda g: int(g["fields"]["grinding_factor"]))
    lines = ["g    sec   TP        proofKiB"]
    for g in grinding:
        gr = int(g["fields"]["grinding_factor"])
        lines.append(f"{gr:<4} {g['security_bits']:<5} {r3(g['prove_med_ms']/1000):<9} "
                     f"{round(g['proof_bytes']/1024,2)}")
    write_block(lines, os.path.join(FIG_DIR, "grinding.dat"))
    fmt_table("=== grinding.dat (S5) ===", lines)

    # ---- blowup.dat (sweep S6, field extension none) ---------------------------------
    blowup = sorted(by_sweep("S6", lambda g: g["fields"]["field_extension"] == "none"),
                    key=lambda g: int(g["fields"]["blowup_factor"]))
    lines = ["b    sec   TP       proofKiB"]
    for g in blowup:
        b = int(g["fields"]["blowup_factor"])
        lines.append(f"{b:<4} {g['security_bits']:<5} {r3(g['prove_med_ms']/1000):<8} "
                     f"{round(g['proof_bytes']/1024,2)}")
    write_block(lines, os.path.join(FIG_DIR, "blowup.dat"))
    fmt_table("=== blowup.dat (S6, none) ===", lines)

    # ---- derived comparisons quoted in the prose (Section 4.4) -----------------------
    def med_s(sweep, pred):
        gs = by_sweep(sweep, pred)
        return r3(gs[0]["prove_med_ms"] / 1000) if gs else None

    print("=== derived comparisons (Section 4.4) ===")
    b3 = med_s("S2", lambda g: g["fields"]["commit_hash"] == "blake3_256")
    sha3 = med_s("S2", lambda g: g["fields"]["commit_hash"] == "sha3_256")
    if b3 and sha3:
        print(f"Blake3-256 {b3}s -> SHA3-256 {sha3}s : +{round((sha3/b3-1)*100,1)}% prove")

    none_g = by_sweep("S6", lambda g: g["fields"]["field_extension"] == "none"
                      and g["fields"]["blowup_factor"] == "8")
    quad_g = by_sweep("S6", lambda g: g["fields"]["field_extension"] == "quad")
    if none_g and quad_g:
        n_tp, q_tp = r3(none_g[0]["prove_med_ms"]/1000), r3(quad_g[0]["prove_med_ms"]/1000)
        n_pb, q_pb = none_g[0]["proof_bytes"], quad_g[0]["proof_bytes"]
        print(f"none {n_tp}s/{n_pb}B -> quad {q_tp}s/{q_pb}B : "
              f"+{round((q_tp/n_tp-1)*100,1)}% time, +{round((q_pb/n_pb-1)*100,1)}% proof")

    st = [g for g in scaling if g["fields"]["n"] == "65536"]
    mt = by_sweep("MT")
    if st and mt:
        st_tp, mt_tp = r3(st[0]["prove_med_ms"]/1000), r3(mt[0]["prove_med_ms"]/1000)
        print(f"1 thread {st_tp}s -> 16 threads {mt_tp}s : {round(st_tp/mt_tp,2)}x speedup")

    print(f"\nWrote {os.path.relpath(SUMMARY, ROOT)} and {os.path.relpath(FIG_DIR, ROOT)}/*.dat")
    print("Compare the .dat blocks above against the filecontents* data blocks used by the "
          "paper figures (the thesis source is delivered separately).")


if __name__ == "__main__":
    main()
