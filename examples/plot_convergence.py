#!/usr/bin/env python3
"""Generate Phase 2 validation convergence plots from CSV data.

Reads CSV files produced by `cargo run --example convergence_data -p arcanum-matrix-fill`
and produces PNG figures for the validation spec (docs/phase2-matrix-fill/validation.md,
section 12).

Usage:
    python examples/plot_convergence.py [--data-dir DIR] [--out-dir DIR]

Defaults:
    --data-dir docs/phase2-matrix-fill/figures
    --out-dir  docs/phase2-matrix-fill/figures
"""

import argparse
import csv
import os
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np


def read_csv(path):
    """Read a CSV file and return a dict of column_name -> list of floats."""
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        columns = {key: [] for key in reader.fieldnames}
        for row in reader:
            for key in reader.fieldnames:
                columns[key].append(float(row[key]))
    return columns


def plot_v_thin_001(data_dir, out_dir):
    """V-THIN-001: |Z(a) - Z_ref| vs a/delta, log-log scale.

    Z_ref is the thinnest-radius result (last row).
    """
    data = read_csv(os.path.join(data_dir, "v_thin_001.csv"))
    a_over_delta = np.array(data["a_over_delta"])
    re_z = np.array(data["re_z"])
    im_z = np.array(data["im_z"])

    # Reference: thinnest radius (last entry)
    re_ref = re_z[-1]
    im_ref = im_z[-1]

    # Exclude reference point itself from plot
    x = a_over_delta[:-1]
    re_err = np.abs(re_z[:-1] - re_ref)
    im_err = np.abs(im_z[:-1] - im_ref)

    fig, ax = plt.subplots(figsize=(7, 5))
    ax.loglog(x, re_err, "o-", label=r"|Re($Z$) $-$ Re($Z_\mathrm{ref}$)|", color="tab:blue")
    ax.loglog(x, im_err, "s-", label=r"|Im($Z$) $-$ Im($Z_\mathrm{ref}$)|", color="tab:red")
    ax.set_xlabel(r"$a / \Delta$")
    ax.set_ylabel(r"|$Z(a) - Z_\mathrm{ref}$| ($\Omega$)")
    ax.set_title("V-THIN-001: Self-Impedance Thin-Wire Convergence")
    ax.legend()
    ax.grid(True, which="both", alpha=0.3)
    ax.invert_xaxis()  # thinnest radius on the right

    fig.tight_layout()
    out_path = os.path.join(out_dir, "v_thin_001.png")
    fig.savefig(out_path, dpi=150)
    plt.close(fig)
    print(f"  -> {out_path}")


def plot_v_quad(data_dir, out_dir, name, title):
    """V-QUAD-00x: |Z(p) - Z(p=64)| vs p, semi-log scale.

    Reference is the highest-order result (last row, p=64).
    """
    data = read_csv(os.path.join(data_dir, f"{name}.csv"))
    order = np.array(data["order"])
    re_z = np.array(data["re_z"])
    im_z = np.array(data["im_z"])

    # Reference: highest order (last entry)
    re_ref = re_z[-1]
    im_ref = im_z[-1]

    # Exclude reference point from plot
    x = order[:-1]
    re_err = np.abs(re_z[:-1] - re_ref)
    im_err = np.abs(im_z[:-1] - im_ref)

    fig, ax = plt.subplots(figsize=(7, 5))
    ax.semilogy(x, re_err, "o-", label=r"|Re($Z$) $-$ Re($Z_\mathrm{ref}$)|", color="tab:blue")
    ax.semilogy(x, im_err, "s-", label=r"|Im($Z$) $-$ Im($Z_\mathrm{ref}$)|", color="tab:red")
    ax.set_xlabel("Quadrature order $p$")
    ax.set_ylabel(r"|$Z(p) - Z(p{=}64)$| ($\Omega$)")
    ax.set_title(title)
    ax.set_xticks(order)
    ax.legend()
    ax.grid(True, which="both", alpha=0.3)

    fig.tight_layout()
    out_path = os.path.join(out_dir, f"{name}.png")
    fig.savefig(out_path, dpi=150)
    plt.close(fig)
    print(f"  -> {out_path}")


def main():
    parser = argparse.ArgumentParser(description="Generate Phase 2 convergence plots")
    parser.add_argument(
        "--data-dir",
        default="docs/phase2-matrix-fill/figures",
        help="Directory containing CSV data files",
    )
    parser.add_argument(
        "--out-dir",
        default="docs/phase2-matrix-fill/figures",
        help="Directory for output PNG files",
    )
    args = parser.parse_args()

    os.makedirs(args.out_dir, exist_ok=True)

    print("Generating convergence plots...")
    plot_v_thin_001(args.data_dir, args.out_dir)
    plot_v_quad(args.data_dir, args.out_dir, "v_quad_001",
                "V-QUAD-001: Self-Impedance Quadrature Convergence")
    plot_v_quad(args.data_dir, args.out_dir, "v_quad_002",
                "V-QUAD-002: Near-Neighbor Mutual Impedance Quadrature Convergence")
    plot_v_quad(args.data_dir, args.out_dir, "v_quad_003",
                "V-QUAD-003: Far Off-Diagonal Mutual Impedance Quadrature Convergence")
    print("\nAll plots generated.")


if __name__ == "__main__":
    main()
