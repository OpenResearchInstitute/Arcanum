"""nec_inspect.py — print a human-readable breakdown of a parsed NEC deck.

Usage:
    python examples/nec_inspect.py path/to/deck.nec

Writes a structured, annotated summary to stdout.
Parse warnings are printed inline after the summary.
Exits with status 1 on parse error.

Requires the arcanum extension to be installed (e.g. via `maturin develop`).
"""

import sys

import arcanum

# ── formatting helpers ────────────────────────────────────────────────────────

SECTION  = "━" * 60
DIVIDER  = "─" * 40

def heading(title: str) -> None:
    print(f"\n{SECTION}")
    print(f"  {title}")
    print(SECTION)

def subheading(title: str) -> None:
    print(f"\n  {DIVIDER}")
    print(f"  {title}")
    print(f"  {DIVIDER}")

def field(label: str, value, indent: int = 4) -> None:
    pad = " " * indent
    print(f"{pad}{label:<28}{value}")

# ── wire printers ─────────────────────────────────────────────────────────────

def print_straight_wire(i: int, w) -> None:
    subheading(f"Wire {i + 1} of {_wire_total}  [GW — straight]  tag={w.tag}")
    field("segments",        w.segment_count)
    field("start (x,y,z) m", f"({w.x1}, {w.y1}, {w.z1})")
    field("end   (x,y,z) m", f"({w.x2}, {w.y2}, {w.z2})")
    field("wire radius m",   w.radius)


def print_arc_wire(i: int, w) -> None:
    subheading(f"Wire {i + 1} of {_wire_total}  [GA — arc]  tag={w.tag}")
    field("segments",        w.segment_count)
    field("arc radius m",    w.arc_radius)
    field("start angle °",   w.angle1)
    field("end angle °",     w.angle2)
    field("wire radius m",   w.radius)


def print_helix_wire(i: int, w) -> None:
    subheading(f"Wire {i + 1} of {_wire_total}  [GH — helix]  tag={w.tag}")
    field("segments",        w.segment_count)
    field("turns",           w.n_turns)
    field("pitch m/turn",    w.pitch)
    field("total length m",  w.total_length)
    field("radius start m",  w.radius_start)
    field("radius end m",    w.radius_end)
    field("wire radius m",   w.radius)


_wire_printers = {
    "StraightWire": print_straight_wire,
    "ArcWire":      print_arc_wire,
    "HelixWire":    print_helix_wire,
}

_wire_total = 0   # set before iterating wires so printers can reference it

# ── section printers ──────────────────────────────────────────────────────────

def print_geometry(sim) -> None:
    global _wire_total
    mesh = sim.mesh_input
    wires = mesh.wires
    _wire_total = len(wires)

    heading("GEOMETRY")
    field("wire elements",   _wire_total)
    field("ground plane",    mesh.ground.ground_type)
    field("GE gpflag",       mesh.gpflag,
          indent=4)

    for i, w in enumerate(wires):
        printer = _wire_printers.get(type(w).__name__)
        if printer:
            printer(i, w)
        else:
            subheading(f"Wire {i + 1} of {_wire_total}  [unknown type]")


def print_ground_electrical(sim) -> None:
    ge = sim.ground_electrical
    if ge is None:
        return
    heading("GROUND — ELECTRICAL PARAMETERS  [GN]")
    field("computation model",  ge.model)
    field("permittivity εr",    ge.permittivity)
    field("conductivity S/m",   ge.conductivity)


def print_frequencies(sim) -> None:
    freqs = sim.frequencies
    heading(f"FREQUENCIES  [FR]  ({len(freqs)} point{'s' if len(freqs) != 1 else ''})")
    for i, hz in enumerate(freqs):
        mhz = hz / 1e6
        field(f"  [{i:>3}]", f"{mhz:.6f} MHz  ({hz:.3f} Hz)")


def print_sources(sim) -> None:
    sources = sim.sources
    if not sources:
        return
    heading(f"EXCITATIONS  [EX]  ({len(sources)})")
    _ex_types = {0: "delta-gap voltage", 5: "current-slope discontinuity"}
    for i, s in enumerate(sources):
        subheading(f"Source {i + 1}  tag={s.tag}  segment={s.segment}")
        field("type",          f"{s.ex_type}  ({_ex_types.get(s.ex_type, 'other')})")
        field("voltage real V", s.voltage_real)
        field("voltage imag V", s.voltage_imag)


def print_loads(sim) -> None:
    loads = sim.loads
    if not loads:
        return
    heading(f"LOADS  [LD]  ({len(loads)})")
    _ld_types = {
        0: "series RLC",
        1: "parallel RLC",
        4: "distributed impedance",
        5: "wire conductivity",
    }
    for i, l in enumerate(loads):
        subheading(f"Load {i + 1}  tag={l.tag}  segs {l.first_seg}–{l.last_seg}")
        field("type",      f"{l.ld_type}  ({_ld_types.get(l.ld_type, 'other')})")
        field("R  Ω",      l.zlr)
        field("L  H",      l.zli)
        field("C  F",      l.zlc)


def print_output_requests(sim) -> None:
    req = sim.output_requests
    rp  = req.radiation_patterns
    ne  = req.near_e_fields
    nh  = req.near_h_fields
    if not (rp or ne or nh):
        return

    heading("OUTPUT REQUESTS")

    if rp:
        print(f"\n  Far-field radiation patterns  [RP]  ({len(rp)})")
        for i, r in enumerate(rp):
            subheading(f"RP {i + 1}  calc={r.calc}  xnda={r.xnda}")
            field("theta points",     r.n_theta)
            field("phi points",       r.n_phi)
            field("theta start °",    r.theta_start)
            field("theta step °",     r.d_theta)
            field("phi start °",      r.phi_start)
            field("phi step °",       r.d_phi)
            field("rfld (0=far)",     r.rfld)

    for label, card, fields_list in (
        ("Near E-field requests  [NE]", ne, ne),
        ("Near H-field requests  [NH]", nh, nh),
    ):
        if not fields_list:
            continue
        print(f"\n  {label}  ({len(fields_list)})")
        for i, nf in enumerate(fields_list):
            subheading(f"NF {i + 1}  grid {nf.nx}×{nf.ny}×{nf.nz}")
            field("origin (xo,yo,zo)", f"({nf.xo}, {nf.yo}, {nf.zo})")
            field("step  (dx,dy,dz)",  f"({nf.dx}, {nf.dy}, {nf.dz})")


def print_warnings(warnings) -> None:
    if not warnings:
        return
    heading(f"PARSE WARNINGS  ({len(warnings)})")
    for w in warnings:
        field(f"line {w.line}  {w.kind}", w.message)


# ── main ──────────────────────────────────────────────────────────────────────

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

    print(f"\nFile: {path}")
    print_geometry(sim)
    print_ground_electrical(sim)
    print_frequencies(sim)
    print_sources(sim)
    print_loads(sim)
    print_output_requests(sim)
    print_warnings(warnings)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
