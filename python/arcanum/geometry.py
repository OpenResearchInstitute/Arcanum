# arcanum.geometry
#
# Python interface for Phase 1 — Geometry Discretization.
# The implementation is in crates/arcanum-geometry (Rust), exposed via the
# native extension built by crates/arcanum-py.
#
# Public API (all importable directly from arcanum):
#
#   arcanum.build_mesh(mesh_input, ground_electrical=None)
#       -> (Mesh, list[GeometryWarning])
#
#   arcanum.Mesh
#       .segments          -> list[Segment]
#       .junctions         -> list[Junction]
#       .ground            -> GroundDescriptor
#       .tag_entries       -> list[(tag, first, last)]
#       .segment_count     -> int
#       .real_segment_count -> int
#       .image_segment_count -> int
#
#   arcanum.Segment
#       .start, .end       -> (x, y, z)  meters
#       .wire_radius       -> float  meters
#       .tag               -> int
#       .segment_index     -> int
#       .wire_index        -> int
#       .is_image          -> bool
#       .curve_type        -> 'Linear' | 'Arc' | 'Helix'
#
#   arcanum.Junction
#       .junction_index    -> int
#       .endpoints         -> list[(segment_index, 'Start'|'End')]
#       .is_self_loop      -> bool
#
#   arcanum.GroundDescriptor
#       .ground_type       -> 'None' | 'Lossy' | 'PEC'
#       .conductivity      -> float | None   S/m
#       .permittivity      -> float | None   εr
#       .images_generated  -> bool
#
#   arcanum.GeometryWarning
#       .kind              -> 'NearCoincidentEndpoints' | 'WireInGroundPlane'
#       .message           -> str
#
#   arcanum.GeometryError  (exception)
#       .kind, .wire_index, .message
