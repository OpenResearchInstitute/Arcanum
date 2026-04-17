// arcanum-py
//
// PyO3 Python bindings for Arcanum. This is the only cdylib in the workspace
// and the only crate that depends on pyo3. All other crates are rlib.
//
// NEC import types and functions are registered in this file.
// Phase 1–4 types and functions are added here as each phase is implemented.

// pyo3 0.22's create_exception! macro internally uses cfg(gil-refs), which
// rustc flags as unexpected. This is a known pyo3 issue; allow it crate-wide.
#![allow(unexpected_cfgs)]

use arcanum_geometry as geo;
use arcanum_nec_import as nec;
use pyo3::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// ParseError — Python exception raised on hard parse failure
// ─────────────────────────────────────────────────────────────────────────────

pyo3::create_exception!(
    arcanum,
    ParseError,
    pyo3::exceptions::PyException,
    "Hard parse error from the NEC import pipeline.\n\n\
     Attributes:\n\
     - kind (str): error category (e.g. 'MissingGeCard')\n\
     - line (int): 1-based line number where the error was detected\n\
     - message (str): human-readable description"
);

/// Convert a Rust ParseError into a Python ParseError exception, attaching
/// `kind` and `line` as instance attributes so tests can assert on them.
fn nec_err_to_pyerr(py: Python<'_>, e: nec::ParseError) -> PyErr {
    let err = ParseError::new_err(format!(
        "[line {}] {}: {}",
        e.line,
        e.kind.as_str(),
        e.message
    ));
    // value_bound borrows err by shared ref; we drop the Bound before returning err.
    {
        let exc = err.value_bound(py);
        let _ = exc.setattr("kind", e.kind.as_str());
        let _ = exc.setattr("line", e.line);
        let _ = exc.setattr("message", e.message.as_str());
    }
    err
}

// ─────────────────────────────────────────────────────────────────────────────
// ParseWarning
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "ParseWarning")]
struct PyParseWarning {
    inner: nec::ParseWarning,
}

#[pymethods]
impl PyParseWarning {
    /// Warning category string (e.g. 'UnknownCard').
    #[getter]
    fn kind(&self) -> &str {
        self.inner.kind.as_str()
    }

    /// 1-based line number where the condition was detected.
    #[getter]
    fn line(&self) -> usize {
        self.inner.line
    }

    /// Human-readable description.
    #[getter]
    fn message(&self) -> &str {
        &self.inner.message
    }

    fn __repr__(&self) -> String {
        format!(
            "ParseWarning(kind={:?}, line={}, message={:?})",
            self.inner.kind.as_str(),
            self.inner.line,
            self.inner.message
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Wire types
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "StraightWire")]
struct PyStraightWire {
    inner: nec::StraightWire,
}

#[pymethods]
impl PyStraightWire {
    /// Wire tag number from the GW card ITAG field.
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }
    /// Number of segments the wire is divided into.
    #[getter]
    fn segment_count(&self) -> u32 {
        self.inner.segment_count
    }
    #[getter]
    fn x1(&self) -> f64 {
        self.inner.x1
    }
    #[getter]
    fn y1(&self) -> f64 {
        self.inner.y1
    }
    #[getter]
    fn z1(&self) -> f64 {
        self.inner.z1
    }
    #[getter]
    fn x2(&self) -> f64 {
        self.inner.x2
    }
    #[getter]
    fn y2(&self) -> f64 {
        self.inner.y2
    }
    #[getter]
    fn z2(&self) -> f64 {
        self.inner.z2
    }
    /// Wire cross-section radius in the same units as coordinates.
    #[getter]
    fn radius(&self) -> f64 {
        self.inner.radius
    }
    fn __repr__(&self) -> String {
        format!(
            "StraightWire(tag={}, segment_count={}, ({},{},{})→({},{},{}))",
            self.inner.tag,
            self.inner.segment_count,
            self.inner.x1,
            self.inner.y1,
            self.inner.z1,
            self.inner.x2,
            self.inner.y2,
            self.inner.z2
        )
    }
}

#[pyclass(name = "ArcWire")]
struct PyArcWire {
    inner: nec::ArcWire,
}

#[pymethods]
impl PyArcWire {
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }
    #[getter]
    fn segment_count(&self) -> u32 {
        self.inner.segment_count
    }
    /// Radius of the circular arc in the XZ plane.
    #[getter]
    fn arc_radius(&self) -> f64 {
        self.inner.arc_radius
    }
    /// Start angle of the arc in degrees.
    #[getter]
    fn angle1(&self) -> f64 {
        self.inner.angle1
    }
    /// End angle of the arc in degrees.
    #[getter]
    fn angle2(&self) -> f64 {
        self.inner.angle2
    }
    /// Wire cross-section radius.
    #[getter]
    fn radius(&self) -> f64 {
        self.inner.radius
    }
    fn __repr__(&self) -> String {
        format!(
            "ArcWire(tag={}, segment_count={}, arc_radius={}, {}°→{}°)",
            self.inner.tag,
            self.inner.segment_count,
            self.inner.arc_radius,
            self.inner.angle1,
            self.inner.angle2
        )
    }
}

#[pyclass(name = "HelixWire")]
struct PyHelixWire {
    inner: nec::HelixWire,
}

#[pymethods]
impl PyHelixWire {
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }
    #[getter]
    fn segment_count(&self) -> u32 {
        self.inner.segment_count
    }
    /// Axial distance per turn.
    #[getter]
    fn pitch(&self) -> f64 {
        self.inner.pitch
    }
    /// Total axial length of the helix.
    #[getter]
    fn total_length(&self) -> f64 {
        self.inner.total_length
    }
    /// Helix radius at the start end.
    #[getter]
    fn radius_start(&self) -> f64 {
        self.inner.radius_start
    }
    /// Helix radius at the far end.
    #[getter]
    fn radius_end(&self) -> f64 {
        self.inner.radius_end
    }
    /// Wire cross-section radius.
    #[getter]
    fn radius(&self) -> f64 {
        self.inner.radius
    }
    /// Derived: total_length / pitch (number of turns).
    #[getter]
    fn n_turns(&self) -> f64 {
        self.inner.n_turns
    }
    fn __repr__(&self) -> String {
        format!(
            "HelixWire(tag={}, segment_count={}, n_turns={})",
            self.inner.tag, self.inner.segment_count, self.inner.n_turns
        )
    }
}

/// Convert a WireDescription into the appropriate concrete Python wire type.
fn wire_to_py(py: Python<'_>, w: nec::WireDescription) -> PyResult<PyObject> {
    match w {
        nec::WireDescription::Straight(sw) => {
            Ok(Py::new(py, PyStraightWire { inner: sw })?.into_any())
        }
        nec::WireDescription::Arc(aw) => Ok(Py::new(py, PyArcWire { inner: aw })?.into_any()),
        nec::WireDescription::Helix(hw) => Ok(Py::new(py, PyHelixWire { inner: hw })?.into_any()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Ground types
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "GeometricGround")]
struct PyGeometricGround {
    inner: nec::GeometricGround,
}

#[pymethods]
impl PyGeometricGround {
    /// Ground type string: 'FreeSpace', 'Lossy', 'PEC', or 'Sommerfeld'.
    #[getter]
    fn ground_type(&self) -> &str {
        self.inner.ground_type.as_str()
    }

    fn __repr__(&self) -> String {
        format!(
            "GeometricGround(ground_type={:?})",
            self.inner.ground_type.as_str()
        )
    }
}

#[pyclass(name = "GroundElectrical")]
struct PyGroundElectrical {
    inner: nec::GroundElectrical,
}

#[pymethods]
impl PyGroundElectrical {
    /// Relative permittivity (εr).
    #[getter]
    fn permittivity(&self) -> f64 {
        self.inner.permittivity
    }
    /// Electrical conductivity in S/m.
    #[getter]
    fn conductivity(&self) -> f64 {
        self.inner.conductivity
    }
    /// Computation model string: 'ReflectionCoeff' or 'Sommerfeld'.
    #[getter]
    fn model(&self) -> &str {
        match self.inner.model {
            nec::GroundModel::ReflectionCoeff => "ReflectionCoeff",
            nec::GroundModel::Sommerfeld => "Sommerfeld",
        }
    }
    fn __repr__(&self) -> String {
        format!(
            "GroundElectrical(permittivity={}, conductivity={}, model={:?})",
            self.inner.permittivity,
            self.inner.conductivity,
            match self.inner.model {
                nec::GroundModel::ReflectionCoeff => "ReflectionCoeff",
                nec::GroundModel::Sommerfeld => "Sommerfeld",
            }
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GeometryTransforms
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "GmOperation")]
struct PyGmOperation {
    inner: nec::GmOperation,
}

#[pymethods]
impl PyGmOperation {
    /// ITAG — wire tag to transform. 0 = all wires.
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }
    /// NRPT — number of additional copies to generate. 0 = transform in place.
    #[getter]
    fn n_copies(&self) -> u32 {
        self.inner.n_copies
    }
    /// ROX — rotation about x-axis, degrees.
    #[getter]
    fn rot_x(&self) -> f64 {
        self.inner.rot_x
    }
    /// ROY — rotation about y-axis, degrees.
    #[getter]
    fn rot_y(&self) -> f64 {
        self.inner.rot_y
    }
    /// ROZ — rotation about z-axis, degrees.
    #[getter]
    fn rot_z(&self) -> f64 {
        self.inner.rot_z
    }
    /// XS — translation along x-axis.
    #[getter]
    fn trans_x(&self) -> f64 {
        self.inner.trans_x
    }
    /// YS — translation along y-axis.
    #[getter]
    fn trans_y(&self) -> f64 {
        self.inner.trans_y
    }
    /// ZS — translation along z-axis.
    #[getter]
    fn trans_z(&self) -> f64 {
        self.inner.trans_z
    }
    /// ITS — tag increment per generated copy.
    #[getter]
    fn tag_increment(&self) -> u32 {
        self.inner.tag_increment
    }
    fn __repr__(&self) -> String {
        format!(
            "GmOperation(tag={}, n_copies={}, rot=({},{},{}), trans=({},{},{}))",
            self.inner.tag,
            self.inner.n_copies,
            self.inner.rot_x,
            self.inner.rot_y,
            self.inner.rot_z,
            self.inner.trans_x,
            self.inner.trans_y,
            self.inner.trans_z,
        )
    }
}

#[pyclass(name = "GeometryTransforms")]
struct PyGeometryTransforms {
    inner: nec::GeometryTransforms,
}

#[pymethods]
impl PyGeometryTransforms {
    /// GS scale factor, or None if no GS card was present.
    #[getter]
    fn gs_scale(&self) -> Option<f64> {
        self.inner.gs_scale
    }
    /// GM operations in deck order.
    #[getter]
    fn gm_ops(&self) -> Vec<PyGmOperation> {
        self.inner
            .gm_ops
            .iter()
            .cloned()
            .map(|op| PyGmOperation { inner: op })
            .collect()
    }
    fn __repr__(&self) -> String {
        format!(
            "GeometryTransforms(gs_scale={:?}, gm_ops={})",
            self.inner.gs_scale,
            self.inner.gm_ops.len()
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MeshInput
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "MeshInput")]
struct PyMeshInput {
    inner: nec::MeshInput,
}

#[pymethods]
impl PyMeshInput {
    /// List of wire elements. Each item is a StraightWire, ArcWire, or HelixWire.
    /// Coordinates are raw (pre-transformation); apply transforms before use.
    #[getter]
    fn wires(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.inner
            .wires
            .iter()
            .cloned()
            .map(|w| wire_to_py(py, w))
            .collect()
    }

    /// Geometric ground plane boundary condition.
    #[getter]
    fn ground(&self) -> PyGeometricGround {
        PyGeometricGround {
            inner: self.inner.ground.clone(),
        }
    }

    /// Ground plane flag from GE card (0 = no ground, 1 = ground present).
    #[getter]
    fn gpflag(&self) -> i32 {
        self.inner.gpflag
    }

    /// GS and GM transformations to be applied by Phase 1.
    #[getter]
    fn transforms(&self) -> PyGeometryTransforms {
        PyGeometryTransforms {
            inner: self.inner.transforms.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "MeshInput(wires={}, gpflag={})",
            self.inner.wires.len(),
            self.inner.gpflag
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Source and load definitions
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "SourceDefinition")]
struct PySourceDefinition {
    inner: nec::SourceDefinition,
}

#[pymethods]
impl PySourceDefinition {
    /// Excitation type: 0 = delta-gap voltage, 5 = current slope discontinuity.
    #[getter]
    fn ex_type(&self) -> i32 {
        self.inner.ex_type
    }
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }
    #[getter]
    fn segment(&self) -> u32 {
        self.inner.segment
    }
    /// Real part of the complex source voltage.
    #[getter]
    fn voltage_real(&self) -> f64 {
        self.inner.voltage_real
    }
    /// Imaginary part of the complex source voltage.
    #[getter]
    fn voltage_imag(&self) -> f64 {
        self.inner.voltage_imag
    }
    fn __repr__(&self) -> String {
        format!(
            "SourceDefinition(tag={}, segment={}, ex_type={})",
            self.inner.tag, self.inner.segment, self.inner.ex_type
        )
    }
}

#[pyclass(name = "LoadDefinition")]
struct PyLoadDefinition {
    inner: nec::LoadDefinition,
}

#[pymethods]
impl PyLoadDefinition {
    /// Load type: 0=series RLC, 1=parallel RLC, 4=distributed, 5=conductivity.
    #[getter]
    fn ld_type(&self) -> i32 {
        self.inner.ld_type
    }
    /// Tag of the loaded wire (0 = all wires).
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }
    /// First segment in the loaded range (1-indexed).
    #[getter]
    fn first_seg(&self) -> u32 {
        self.inner.first_seg
    }
    /// Last segment in the loaded range (0 = same as first_seg).
    #[getter]
    fn last_seg(&self) -> u32 {
        self.inner.last_seg
    }
    /// Series/parallel resistance in Ω (or conductivity in S/m for type 5).
    #[getter]
    fn zlr(&self) -> f64 {
        self.inner.zlr
    }
    /// Series/parallel inductance in H.
    #[getter]
    fn zli(&self) -> f64 {
        self.inner.zli
    }
    /// Series/parallel capacitance in F.
    #[getter]
    fn zlc(&self) -> f64 {
        self.inner.zlc
    }
    fn __repr__(&self) -> String {
        format!(
            "LoadDefinition(tag={}, ld_type={}, segs={}..{})",
            self.inner.tag, self.inner.ld_type, self.inner.first_seg, self.inner.last_seg
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Output requests
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "RadiationPatternRequest")]
struct PyRadiationPatternRequest {
    inner: nec::RadiationPatternRequest,
}

#[pymethods]
impl PyRadiationPatternRequest {
    #[getter]
    fn calc(&self) -> i32 {
        self.inner.calc
    }
    #[getter]
    fn n_theta(&self) -> u32 {
        self.inner.n_theta
    }
    #[getter]
    fn n_phi(&self) -> u32 {
        self.inner.n_phi
    }
    #[getter]
    fn xnda(&self) -> i32 {
        self.inner.xnda
    }
    #[getter]
    fn theta_start(&self) -> f64 {
        self.inner.theta_start
    }
    #[getter]
    fn phi_start(&self) -> f64 {
        self.inner.phi_start
    }
    #[getter]
    fn d_theta(&self) -> f64 {
        self.inner.d_theta
    }
    #[getter]
    fn d_phi(&self) -> f64 {
        self.inner.d_phi
    }
    /// Radial distance in wavelengths; 0.0 = far field.
    #[getter]
    fn rfld(&self) -> f64 {
        self.inner.rfld
    }
    fn __repr__(&self) -> String {
        format!(
            "RadiationPatternRequest(n_theta={}, n_phi={})",
            self.inner.n_theta, self.inner.n_phi
        )
    }
}

#[pyclass(name = "NearFieldRequest")]
struct PyNearFieldRequest {
    inner: nec::NearFieldRequest,
}

#[pymethods]
impl PyNearFieldRequest {
    #[getter]
    fn nx(&self) -> u32 {
        self.inner.nx
    }
    #[getter]
    fn ny(&self) -> u32 {
        self.inner.ny
    }
    #[getter]
    fn nz(&self) -> u32 {
        self.inner.nz
    }
    #[getter]
    fn xo(&self) -> f64 {
        self.inner.xo
    }
    #[getter]
    fn yo(&self) -> f64 {
        self.inner.yo
    }
    #[getter]
    fn zo(&self) -> f64 {
        self.inner.zo
    }
    #[getter]
    fn dx(&self) -> f64 {
        self.inner.dx
    }
    #[getter]
    fn dy(&self) -> f64 {
        self.inner.dy
    }
    #[getter]
    fn dz(&self) -> f64 {
        self.inner.dz
    }
    fn __repr__(&self) -> String {
        format!(
            "NearFieldRequest(nx={}, ny={}, nz={})",
            self.inner.nx, self.inner.ny, self.inner.nz
        )
    }
}

#[pyclass(name = "OutputRequests")]
struct PyOutputRequests {
    inner: nec::OutputRequests,
}

#[pymethods]
impl PyOutputRequests {
    /// Far-field radiation pattern requests (from RP cards).
    #[getter]
    fn radiation_patterns(&self) -> Vec<PyRadiationPatternRequest> {
        self.inner
            .radiation_patterns
            .iter()
            .cloned()
            .map(|r| PyRadiationPatternRequest { inner: r })
            .collect()
    }
    /// Near electric-field requests (from NE cards).
    #[getter]
    fn near_e_fields(&self) -> Vec<PyNearFieldRequest> {
        self.inner
            .near_e_fields
            .iter()
            .cloned()
            .map(|r| PyNearFieldRequest { inner: r })
            .collect()
    }
    /// Near magnetic-field requests (from NH cards).
    #[getter]
    fn near_h_fields(&self) -> Vec<PyNearFieldRequest> {
        self.inner
            .near_h_fields
            .iter()
            .cloned()
            .map(|r| PyNearFieldRequest { inner: r })
            .collect()
    }
    fn __repr__(&self) -> String {
        format!(
            "OutputRequests(radiation_patterns={}, near_e={}, near_h={})",
            self.inner.radiation_patterns.len(),
            self.inner.near_e_fields.len(),
            self.inner.near_h_fields.len()
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SimulationInput — top-level parse result
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "SimulationInput")]
struct PySimulationInput {
    inner: nec::SimulationInput,
}

#[pymethods]
impl PySimulationInput {
    /// Wire geometry and ground boundary condition (consumed by Phase 1).
    #[getter]
    fn mesh_input(&self) -> PyMeshInput {
        PyMeshInput {
            inner: self.inner.mesh_input.clone(),
        }
    }

    /// Frequency list in Hz (consumed by Phase 2 and Phase 3).
    #[getter]
    fn frequencies(&self) -> Vec<f64> {
        self.inner.frequencies.clone()
    }

    /// Voltage/current source definitions (consumed by Phase 3).
    #[getter]
    fn sources(&self) -> Vec<PySourceDefinition> {
        self.inner
            .sources
            .iter()
            .cloned()
            .map(|s| PySourceDefinition { inner: s })
            .collect()
    }

    /// Impedance load definitions (consumed by Phase 3).
    #[getter]
    fn loads(&self) -> Vec<PyLoadDefinition> {
        self.inner
            .loads
            .iter()
            .cloned()
            .map(|l| PyLoadDefinition { inner: l })
            .collect()
    }

    /// Ground electrical parameters (consumed by Phase 2). None for free space or PEC.
    #[getter]
    fn ground_electrical(&self) -> Option<PyGroundElectrical> {
        self.inner
            .ground_electrical
            .clone()
            .map(|g| PyGroundElectrical { inner: g })
    }

    /// Far-field and near-field output requests (consumed by Phase 4).
    #[getter]
    fn output_requests(&self) -> PyOutputRequests {
        PyOutputRequests {
            inner: self.inner.output_requests.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SimulationInput(wires={}, freqs={}, sources={}, loads={})",
            self.inner.mesh_input.wires.len(),
            self.inner.frequencies.len(),
            self.inner.sources.len(),
            self.inner.loads.len()
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public functions
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a NEC input deck from a string.
///
/// Returns ``(SimulationInput, [ParseWarning, ...])`` on success.
/// Raises ``ParseError`` on the first hard error encountered.
#[pyfunction]
fn parse(py: Python<'_>, input: &str) -> PyResult<(PySimulationInput, Vec<PyParseWarning>)> {
    let (sim, warnings) = match nec::parse(input) {
        Ok(v) => v,
        Err(e) => return Err(nec_err_to_pyerr(py, e)),
    };
    let py_warns = warnings
        .into_vec()
        .into_iter()
        .map(|w| PyParseWarning { inner: w })
        .collect();
    Ok((PySimulationInput { inner: sim }, py_warns))
}

/// Parse a NEC input deck from a file path.
///
/// Reads the file then calls ``parse``. Raises ``ParseError`` on any failure
/// (I/O errors are wrapped as ``ParseError`` with line 0).
#[pyfunction]
fn parse_file(py: Python<'_>, path: &str) -> PyResult<(PySimulationInput, Vec<PyParseWarning>)> {
    let (sim, warnings) = match nec::parse_file(std::path::Path::new(path)) {
        Ok(v) => v,
        Err(e) => return Err(nec_err_to_pyerr(py, e)),
    };
    let py_warns = warnings
        .into_vec()
        .into_iter()
        .map(|w| PyParseWarning { inner: w })
        .collect();
    Ok((PySimulationInput { inner: sim }, py_warns))
}

// ─────────────────────────────────────────────────────────────────────────────
// GeometryError — Python exception raised on hard geometry error
// ─────────────────────────────────────────────────────────────────────────────

pyo3::create_exception!(
    arcanum,
    GeometryError,
    pyo3::exceptions::PyException,
    "Hard geometry error from the Phase 1 mesh builder.\n\n\
     Attributes:\n\
     - kind (str): error category (e.g. 'ZeroLengthWire')\n\
     - wire_index (int): 0-based wire index where the error was detected\n\
     - message (str): human-readable description"
);

fn geo_err_to_pyerr(py: Python<'_>, e: geo::GeometryError) -> PyErr {
    let err = GeometryError::new_err(format!(
        "[wire {}] {}: {}",
        e.wire_index,
        e.kind.as_str(),
        e.message
    ));
    {
        let exc = err.value_bound(py);
        let _ = exc.setattr("kind", e.kind.as_str());
        let _ = exc.setattr("wire_index", e.wire_index);
        let _ = exc.setattr("message", e.message.as_str());
    }
    err
}

// ─────────────────────────────────────────────────────────────────────────────
// GeometryWarning
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "GeometryWarning")]
struct PyGeometryWarning {
    inner: geo::GeometryWarning,
}

#[pymethods]
impl PyGeometryWarning {
    /// Warning category string (e.g. 'NearCoincidentEndpoints').
    #[getter]
    fn kind(&self) -> &str {
        self.inner.kind.as_str()
    }

    /// Human-readable description.
    #[getter]
    fn message(&self) -> &str {
        &self.inner.message
    }

    fn __repr__(&self) -> String {
        format!(
            "GeometryWarning(kind={:?}, message={:?})",
            self.inner.kind.as_str(),
            self.inner.message
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Segment
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "Segment")]
struct PySegment {
    inner: geo::Segment,
}

#[pymethods]
impl PySegment {
    /// Precomputed start point as (x, y, z) in meters.
    #[getter]
    fn start(&self) -> (f64, f64, f64) {
        let p = self.inner.start();
        (p.x, p.y, p.z)
    }

    /// Precomputed end point as (x, y, z) in meters.
    #[getter]
    fn end(&self) -> (f64, f64, f64) {
        let p = self.inner.end();
        (p.x, p.y, p.z)
    }

    /// Wire cross-section radius in meters. Not scaled by GS.
    #[getter]
    fn wire_radius(&self) -> f64 {
        self.inner.wire_radius
    }

    /// NEC wire tag number.
    #[getter]
    fn tag(&self) -> u32 {
        self.inner.tag
    }

    /// Global index of this segment in the mesh segment list.
    #[getter]
    fn segment_index(&self) -> usize {
        self.inner.segment_index
    }

    /// Index of the wire (GW/GA/GH card) this segment belongs to.
    #[getter]
    fn wire_index(&self) -> usize {
        self.inner.wire_index
    }

    /// True if this is a PEC ground image segment (not addressable by EX/LD).
    #[getter]
    fn is_image(&self) -> bool {
        self.inner.is_image
    }

    /// Curve type string: 'Linear', 'Arc', or 'Helix'.
    #[getter]
    fn curve_type(&self) -> &str {
        match &self.inner.curve {
            geo::CurveParams::Linear(_) => "Linear",
            geo::CurveParams::Arc(_) => "Arc",
            geo::CurveParams::Helix(_) => "Helix",
        }
    }

    fn __repr__(&self) -> String {
        let s = self.inner.start();
        let e = self.inner.end();
        format!(
            "Segment(index={}, tag={}, type={}, ({:.4},{:.4},{:.4})→({:.4},{:.4},{:.4}){})",
            self.inner.segment_index,
            self.inner.tag,
            match &self.inner.curve {
                geo::CurveParams::Linear(_) => "Linear",
                geo::CurveParams::Arc(_) => "Arc",
                geo::CurveParams::Helix(_) => "Helix",
            },
            s.x, s.y, s.z,
            e.x, e.y, e.z,
            if self.inner.is_image { " [image]" } else { "" }
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Junction
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "Junction")]
struct PyJunction {
    inner: geo::Junction,
}

#[pymethods]
impl PyJunction {
    /// Unique index of this junction.
    #[getter]
    fn junction_index(&self) -> usize {
        self.inner.junction_index
    }

    /// All segment endpoints at this junction as list of (segment_index, side) tuples.
    /// side is 'Start' or 'End'.
    #[getter]
    fn endpoints(&self) -> Vec<(usize, String)> {
        self.inner
            .endpoints
            .iter()
            .map(|ep| {
                let side = match ep.side {
                    geo::EndpointSide::Start => "Start",
                    geo::EndpointSide::End => "End",
                };
                (ep.segment_index, side.to_string())
            })
            .collect()
    }

    /// True if this junction is a self-loop (start and end of the same wire).
    #[getter]
    fn is_self_loop(&self) -> bool {
        self.inner.is_self_loop
    }

    fn __repr__(&self) -> String {
        format!(
            "Junction(index={}, endpoints={}, is_self_loop={})",
            self.inner.junction_index,
            self.inner.endpoints.len(),
            self.inner.is_self_loop
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GroundDescriptor
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "GroundDescriptor")]
struct PyGroundDescriptor {
    inner: geo::GroundDescriptor,
}

#[pymethods]
impl PyGroundDescriptor {
    /// Ground type string: 'None', 'Lossy', or 'PEC'.
    #[getter]
    fn ground_type(&self) -> &str {
        match self.inner.ground_type {
            geo::GroundType::None => "None",
            geo::GroundType::Lossy => "Lossy",
            geo::GroundType::PEC => "PEC",
        }
    }

    /// Electrical conductivity in S/m. None for PEC or free space.
    #[getter]
    fn conductivity(&self) -> Option<f64> {
        self.inner.conductivity
    }

    /// Relative permittivity εr. None for PEC or free space.
    #[getter]
    fn permittivity(&self) -> Option<f64> {
        self.inner.permittivity
    }

    /// True if Phase 1 generated PEC image segments for this mesh.
    #[getter]
    fn images_generated(&self) -> bool {
        self.inner.images_generated
    }

    fn __repr__(&self) -> String {
        format!(
            "GroundDescriptor(ground_type={:?}, images_generated={})",
            match self.inner.ground_type {
                geo::GroundType::None => "None",
                geo::GroundType::Lossy => "Lossy",
                geo::GroundType::PEC => "PEC",
            },
            self.inner.images_generated
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mesh
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "Mesh")]
struct PyMesh {
    inner: geo::Mesh,
}

#[pymethods]
impl PyMesh {
    /// All segments in order: real segments first, image segments last.
    #[getter]
    fn segments(&self) -> Vec<PySegment> {
        self.inner
            .segments
            .iter()
            .cloned()
            .map(|s| PySegment { inner: s })
            .collect()
    }

    /// All junctions detected in the mesh.
    #[getter]
    fn junctions(&self) -> Vec<PyJunction> {
        self.inner
            .junctions
            .iter()
            .cloned()
            .map(|j| PyJunction { inner: j })
            .collect()
    }

    /// Ground plane descriptor.
    #[getter]
    fn ground(&self) -> PyGroundDescriptor {
        PyGroundDescriptor {
            inner: self.inner.ground.clone(),
        }
    }

    /// Tag map entries as (tag, first_segment_index, last_segment_index) tuples.
    /// Image segments are excluded.
    #[getter]
    fn tag_entries(&self) -> Vec<(u32, usize, usize)> {
        self.inner.tag_map.iter().collect()
    }

    /// Total number of segments (real + image).
    #[getter]
    fn segment_count(&self) -> usize {
        self.inner.segments.len()
    }

    /// Number of real (non-image) segments.
    #[getter]
    fn real_segment_count(&self) -> usize {
        self.inner.real_segment_count()
    }

    /// Number of PEC image segments.
    #[getter]
    fn image_segment_count(&self) -> usize {
        self.inner.image_segment_count()
    }

    fn __repr__(&self) -> String {
        format!(
            "Mesh(segments={}, real={}, images={}, junctions={})",
            self.inner.segments.len(),
            self.inner.real_segment_count(),
            self.inner.image_segment_count(),
            self.inner.junctions.len()
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Geometry functions
// ─────────────────────────────────────────────────────────────────────────────

/// Build a discretized segment mesh from a parsed MeshInput.
///
/// Returns ``(Mesh, [GeometryWarning, ...])`` on success.
/// Raises ``GeometryError`` on any hard error (zero-length wire, etc.).
///
/// ``ground_electrical`` carries the lossy GN card parameters (conductivity,
/// permittivity). Pass ``None`` (the default) for PEC or free-space models.
#[pyfunction]
#[pyo3(signature = (mesh_input, ground_electrical=None))]
fn build_mesh(
    py: Python<'_>,
    mesh_input: &PyMeshInput,
    ground_electrical: Option<&PyGroundElectrical>,
) -> PyResult<(PyMesh, Vec<PyGeometryWarning>)> {
    let ge = ground_electrical.map(|g| g.inner.clone());
    match geo::build_mesh(mesh_input.inner.clone(), ge) {
        Ok((mesh, warnings)) => {
            let py_warnings = warnings
                .into_vec()
                .into_iter()
                .map(|w| PyGeometryWarning { inner: w })
                .collect();
            Ok((PyMesh { inner: mesh }, py_warnings))
        }
        Err(e) => Err(geo_err_to_pyerr(py, e)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Module registration
// ─────────────────────────────────────────────────────────────────────────────

/// Arcanum — Open Source CMoM Antenna Simulation Engine.
#[pymodule]
fn arcanum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    // Exception types
    m.add("ParseError", py.get_type_bound::<ParseError>())?;
    m.add("GeometryError", py.get_type_bound::<GeometryError>())?;

    // NEC import types
    m.add_class::<PyParseWarning>()?;
    m.add_class::<PyGmOperation>()?;
    m.add_class::<PyGeometryTransforms>()?;
    m.add_class::<PyStraightWire>()?;
    m.add_class::<PyArcWire>()?;
    m.add_class::<PyHelixWire>()?;
    m.add_class::<PyGeometricGround>()?;
    m.add_class::<PyGroundElectrical>()?;
    m.add_class::<PyMeshInput>()?;
    m.add_class::<PySourceDefinition>()?;
    m.add_class::<PyLoadDefinition>()?;
    m.add_class::<PyRadiationPatternRequest>()?;
    m.add_class::<PyNearFieldRequest>()?;
    m.add_class::<PyOutputRequests>()?;
    m.add_class::<PySimulationInput>()?;

    // NEC import functions
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;

    // Geometry types
    m.add_class::<PyGeometryWarning>()?;
    m.add_class::<PySegment>()?;
    m.add_class::<PyJunction>()?;
    m.add_class::<PyGroundDescriptor>()?;
    m.add_class::<PyMesh>()?;

    // Geometry functions
    m.add_function(wrap_pyfunction!(build_mesh, m)?)?;

    Ok(())
}
