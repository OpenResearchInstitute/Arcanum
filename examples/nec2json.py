"""nec2json.py — dump a parsed NEC deck as JSON.

Usage:
    python examples/nec_dump.py path/to/deck.nec
    python examples/nec_dump.py path/to/deck.nec | python -m json.tool

Writes JSON to stdout. Writes parse warnings to stderr.
Exits with status 1 on parse error.

Requires the arcanum extension to be installed (e.g. via `maturin develop`).
"""

import json
import sys

import arcanum


def wire_to_dict(w) -> dict:
    kind = type(w).__name__
    base = {
        "wire_type": kind,
        "tag": w.tag,
        "segment_count": w.segment_count,
        "radius": w.radius,
    }
    if kind == "StraightWire":
        base.update({"x1": w.x1, "y1": w.y1, "z1": w.z1,
                     "x2": w.x2, "y2": w.y2, "z2": w.z2})
    elif kind == "ArcWire":
        base.update({"arc_radius": w.arc_radius,
                     "angle1_deg": w.angle1, "angle2_deg": w.angle2})
    elif kind == "HelixWire":
        base.update({"pitch": w.pitch, "total_length": w.total_length,
                     "radius_start": w.radius_start, "radius_end": w.radius_end,
                     "n_turns": w.n_turns})
    return base


def sim_to_dict(sim) -> dict:
    mesh = sim.mesh_input
    ground_elec = sim.ground_electrical

    t = mesh.transforms
    out = {
        "mesh_input": {
            "gpflag": mesh.gpflag,
            "ground_type": mesh.ground.ground_type,
            "wires": [wire_to_dict(w) for w in mesh.wires],
            "transforms": {
                "gs_scale": t.gs_scale,
                "gm_ops": [
                    {
                        "tag": op.tag,
                        "n_copies": op.n_copies,
                        "rot_x_deg": op.rot_x,
                        "rot_y_deg": op.rot_y,
                        "rot_z_deg": op.rot_z,
                        "trans_x": op.trans_x,
                        "trans_y": op.trans_y,
                        "trans_z": op.trans_z,
                        "tag_increment": op.tag_increment,
                    }
                    for op in t.gm_ops
                ],
            },
        },
        "frequencies_hz": sim.frequencies,
        "sources": [
            {
                "tag": s.tag,
                "segment": s.segment,
                "ex_type": s.ex_type,
                "voltage_real": s.voltage_real,
                "voltage_imag": s.voltage_imag,
            }
            for s in sim.sources
        ],
        "loads": [
            {
                "tag": l.tag,
                "ld_type": l.ld_type,
                "first_seg": l.first_seg,
                "last_seg": l.last_seg,
                "zlr": l.zlr,
                "zli": l.zli,
                "zlc": l.zlc,
            }
            for l in sim.loads
        ],
        "ground_electrical": (
            {
                "permittivity": ground_elec.permittivity,
                "conductivity": ground_elec.conductivity,
                "model": ground_elec.model,
            }
            if ground_elec is not None
            else None
        ),
        "output_requests": {
            "radiation_patterns": [
                {
                    "calc": rp.calc,
                    "n_theta": rp.n_theta,
                    "n_phi": rp.n_phi,
                    "xnda": rp.xnda,
                    "theta_start_deg": rp.theta_start,
                    "phi_start_deg": rp.phi_start,
                    "d_theta_deg": rp.d_theta,
                    "d_phi_deg": rp.d_phi,
                    "rfld": rp.rfld,
                }
                for rp in sim.output_requests.radiation_patterns
            ],
            "near_e_fields": [
                {
                    "nx": nf.nx, "ny": nf.ny, "nz": nf.nz,
                    "xo": nf.xo, "yo": nf.yo, "zo": nf.zo,
                    "dx": nf.dx, "dy": nf.dy, "dz": nf.dz,
                }
                for nf in sim.output_requests.near_e_fields
            ],
            "near_h_fields": [
                {
                    "nx": nf.nx, "ny": nf.ny, "nz": nf.nz,
                    "xo": nf.xo, "yo": nf.yo, "zo": nf.zo,
                    "dx": nf.dx, "dy": nf.dy, "dz": nf.dz,
                }
                for nf in sim.output_requests.near_h_fields
            ],
        },
    }
    return out


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <deck.nec>", file=sys.stderr)
        return 2

    path = sys.argv[1]
    try:
        sim, warnings = arcanum.parse_file(path)
    except arcanum.ParseError as e:
        print(f"parse error: {e}", file=sys.stderr)
        return 1

    for w in warnings:
        print(f"warning [line {w.line}] {w.kind}: {w.message}", file=sys.stderr)

    print(json.dumps(sim_to_dict(sim), indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
