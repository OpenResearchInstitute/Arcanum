// arcanum-geometry — Phase 1: Geometry Discretization
//
// Accepts MeshInput (and the associated GroundElectrical) from arcanum-nec-import
// and produces a Mesh: a complete, validated segment mesh with connectivity and
// ground descriptor.
//
// Public API:
//   pub fn build_mesh(
//       input: MeshInput,
//       ground_electrical: Option<GroundElectrical>,
//   ) -> Result<(Mesh, GeometryWarnings), GeometryError>

pub mod errors;
pub mod mesh;

pub(crate) mod discretize;
pub(crate) mod images;
pub(crate) mod junctions;
pub(crate) mod tagmap;
pub(crate) mod transforms;

#[cfg(test)]
mod tests;

use arcanum_nec_import::{GroundElectrical, GroundType as NecGroundType, MeshInput};

pub use errors::{
    GeometryError, GeometryErrorKind, GeometryWarning, GeometryWarningKind, GeometryWarnings,
};
pub use mesh::{
    ArcParams, CurveParams, CurveType, EndpointSide, GroundDescriptor, GroundType, HelixParams,
    Junction, LinearParams, Material, Mesh, Segment, SegmentEndpoint, TagMap,
};

/// Build a segment mesh from a parsed NEC MeshInput.
///
/// `ground_electrical` carries the lossy ground parameters from the GN card
/// (conductivity and permittivity). Phase 1 stores them in the GroundDescriptor
/// for Phase 2 to consume; Phase 1 itself does not use them.
///
/// Returns `(Mesh, GeometryWarnings)` on success. Returns `GeometryError` on
/// any hard error.
pub fn build_mesh(
    input: MeshInput,
    ground_electrical: Option<GroundElectrical>,
) -> Result<(Mesh, GeometryWarnings), GeometryError> {
    let mut warnings = GeometryWarnings::new();

    // Step 1: Discretize all declared wires into segments.
    let (mut segments, mut tag_map) = discretize::discretize_wires(&input.wires, &mut warnings)?;

    // Step 2: Apply GS scale and GM transformations.
    transforms::apply(&mut segments, &mut tag_map, &input.transforms, &input.wires)?;

    // Step 3: Build the ground descriptor.
    let mut ground = build_ground_descriptor(&input, ground_electrical);

    // Step 4: Generate PEC image segments if required.
    if ground.ground_type == GroundType::PEC {
        images::generate(&mut segments, &mut warnings);
        ground.images_generated = true;
    }

    // Step 5: Detect junctions.
    let (junctions, endpoint_junction) = junctions::detect(&segments, &mut warnings);

    Ok((
        Mesh {
            segments,
            junctions,
            endpoint_junction,
            ground,
            tag_map,
        },
        warnings,
    ))
}

fn build_ground_descriptor(
    input: &MeshInput,
    ground_electrical: Option<GroundElectrical>,
) -> GroundDescriptor {
    let ground_type = match input.ground.ground_type {
        NecGroundType::PEC => GroundType::PEC,
        NecGroundType::Lossy | NecGroundType::Sommerfeld => GroundType::Lossy,
        NecGroundType::FreeSpace => GroundType::None,
    };

    let (conductivity, permittivity) = match ground_type {
        GroundType::Lossy => {
            if let Some(ge) = ground_electrical {
                (Some(ge.conductivity), Some(ge.permittivity))
            } else {
                (None, None)
            }
        }
        _ => (None, None),
    };

    GroundDescriptor {
        ground_type,
        conductivity,
        permittivity,
        images_generated: false,
    }
}
