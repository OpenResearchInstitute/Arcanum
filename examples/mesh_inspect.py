"""mesh_inspect.py — parse a NEC deck, build the Phase 1 mesh, and summarise.

Usage:
    python examples/mesh_inspect.py path/to/deck.nec

Writes a structured summary to stdout:
  - NEC parse overview (wires, ground, transforms)
  - Mesh summary (segment counts, ground descriptor)
  - Tag map
  - Junctions (if any)
  - Segment table
  - Geometry and parse warnings

Requires the arcanum extension to be installed (e.g. via `maturin develop`).
"""

import math
import sys

# Ensure Unicode line-drawing characters render correctly on Windows.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

import arcanum

# ── formatting helpers ────────────────────────────────────────────────────────

SECTION = "━" * 60
DIVIDER = "─" * 40


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


def fmt_pt(xyz) -> str:
    x, y, z = xyz
    return f"({x:+.6f}, {y:+.6f}, {z:+.6f})"


def seg_length(seg) -> float:
    sx, sy, sz = seg.start
    ex, ey, ez = seg.end
    return math.sqrt((ex - sx) ** 2 + (ey - sy) ** 2 + (ez - sz) ** 2)


# ── NEC parse summary ─────────────────────────────────────────────────────────

def print_nec_summary(sim, path: str) -> None:
    mesh = sim.mesh_input
    wires = mesh.wires
    heading(f"NEC INPUT  —  {path}")
    field("wires declared", len(wires))
    field("ground type", mesh.ground.ground_type)

    transforms = mesh.transforms
    if transforms.gs_scale is not None:
        field("GS scale factor", transforms.gs_scale)
    if transforms.gm_ops:
        field("GM operations", len(transforms.gm_ops))

    freqs = sim.frequencies
    if freqs:
        mhz_list = ", ".join(f"{f / 1e6:.3f}" for f in freqs[:4])
        suffix = f"  …+{len(freqs) - 4} more" if len(freqs) > 4 else ""
        field("frequencies MHz", mhz_list + suffix)

    if sim.sources:
        for s in sim.sources:
            field(f"  source tag={s.tag} seg={s.segment}", f"EX type {s.ex_type}")

    if sim.ground_electrical:
        ge = sim.ground_electrical
        field("ground εr", ge.permittivity)
        field("ground σ S/m", ge.conductivity)


# ── mesh summary ──────────────────────────────────────────────────────────────

def print_mesh_summary(mesh) -> None:
    heading("MESH SUMMARY")
    field("total segments", mesh.segment_count)
    field("real segments", mesh.real_segment_count)
    if mesh.image_segment_count:
        field("image segments (PEC)", mesh.image_segment_count)
    field("junctions", len(mesh.junctions))

    g = mesh.ground
    field("ground type", g.ground_type)
    if g.images_generated:
        field("PEC images generated", "yes")
    if g.conductivity is not None:
        field("conductivity S/m", g.conductivity)
    if g.permittivity is not None:
        field("permittivity εr", g.permittivity)


# ── tag map ───────────────────────────────────────────────────────────────────

def print_tag_map(mesh) -> None:
    entries = mesh.tag_entries
    if not entries:
        return
    heading(f"TAG MAP  ({len(entries)} wire{'s' if len(entries) != 1 else ''})")
    print(f"    {'tag':>4}  {'first seg':>9}  {'last seg':>8}  {'count':>5}")
    print(f"    {'───':>4}  {'─────────':>9}  {'────────':>8}  {'─────':>5}")
    for tag, first, last in entries:
        count = last - first + 1
        print(f"    {tag:>4}  {first:>9}  {last:>8}  {count:>5}")


# ── junctions ─────────────────────────────────────────────────────────────────

def print_junctions(mesh) -> None:
    junctions = mesh.junctions
    if not junctions:
        return
    heading(f"JUNCTIONS  ({len(junctions)})")
    segments = mesh.segments
    for j in junctions:
        loop_label = "  [self-loop]" if j.is_self_loop else ""
        subheading(f"Junction {j.junction_index}  —  {len(j.endpoints)} endpoints{loop_label}")

        # Collect distinct wire indices contributing to this junction (valence).
        wire_indices = {segments[si].wire_index for si, _side in j.endpoints}
        field("valence (wires)", len(wire_indices))

        # Print position from the first endpoint's segment start/end.
        first_si, first_side = j.endpoints[0]
        pos = segments[first_si].start if first_side == "Start" else segments[first_si].end
        field("position m", fmt_pt(pos))

        for si, side in j.endpoints:
            seg = segments[si]
            field(f"  seg {si} {side}", f"tag={seg.tag}  wire={seg.wire_index}")


# ── segment table ─────────────────────────────────────────────────────────────

# Maximum segments to print individually before switching to a compact summary.
_SEG_DETAIL_LIMIT = 40


def print_segments(mesh) -> None:
    segments = mesh.segments
    total = len(segments)
    heading(f"SEGMENTS  ({total} total)")

    if total > _SEG_DETAIL_LIMIT:
        print(f"  (showing first {_SEG_DETAIL_LIMIT} of {total} segments)\n")

    hdr = f"  {'idx':>4}  {'tag':>3}  {'type':>6}  {'wire':>4}  {'start (x,y,z) m':<36}  {'end (x,y,z) m':<36}  {'len m':>9}"
    bar = f"  {'────':>4}  {'───':>3}  {'──────':>6}  {'────':>4}  {'─' * 36:<36}  {'─' * 36:<36}  {'─────────':>9}"
    print(hdr)
    print(bar)

    for seg in segments[:_SEG_DETAIL_LIMIT]:
        image_flag = "*" if seg.is_image else " "
        length = seg_length(seg)
        print(
            f"  {seg.segment_index:>4}{image_flag} {seg.tag:>3}  {seg.curve_type:>6}  {seg.wire_index:>4}"
            f"  {fmt_pt(seg.start):<36}  {fmt_pt(seg.end):<36}  {length:>9.6f}"
        )

    if total > _SEG_DETAIL_LIMIT:
        shown = _SEG_DETAIL_LIMIT
        remaining = total - shown
        real_rem = sum(1 for s in segments[shown:] if not s.is_image)
        img_rem  = remaining - real_rem
        parts = []
        if real_rem:
            parts.append(f"{real_rem} real")
        if img_rem:
            parts.append(f"{img_rem} image")
        print(f"\n  … {remaining} more segments ({', '.join(parts)})")

    if any(s.is_image for s in segments):
        print("\n  * = PEC image segment")


# ── warnings ──────────────────────────────────────────────────────────────────

def print_geometry_warnings(geo_warnings) -> None:
    if not geo_warnings:
        return
    heading(f"GEOMETRY WARNINGS  ({len(geo_warnings)})")
    for w in geo_warnings:
        field(w.kind, w.message)


def print_parse_warnings(parse_warnings) -> None:
    if not parse_warnings:
        return
    heading(f"PARSE WARNINGS  ({len(parse_warnings)})")
    for w in parse_warnings:
        field(f"line {w.line}  {w.kind}", w.message)


# ── main ──────────────────────────────────────────────────────────────────────

def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <deck.nec>", file=sys.stderr)
        return 2

    path = sys.argv[1]

    try:
        sim, parse_warnings = arcanum.parse_file(path)
    except arcanum.ParseError as e:
        print(f"parse error: {e}", file=sys.stderr)
        return 1

    ge = sim.ground_electrical  # None for free space / PEC
    try:
        mesh, geo_warnings = arcanum.build_mesh(sim.mesh_input, ge)
    except arcanum.GeometryError as e:
        print(f"geometry error: {e}", file=sys.stderr)
        return 1

    print_nec_summary(sim, path)
    print_mesh_summary(mesh)
    print_tag_map(mesh)
    print_junctions(mesh)
    print_segments(mesh)
    print_geometry_warnings(geo_warnings)
    print_parse_warnings(parse_warnings)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
