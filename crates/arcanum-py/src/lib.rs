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
// MeshInput
// ─────────────────────────────────────────────────────────────────────────────

#[pyclass(name = "MeshInput")]
struct PyMeshInput {
    inner: nec::MeshInput,
}

#[pymethods]
impl PyMeshInput {
    /// List of wire elements. Each item is a StraightWire, ArcWire, or HelixWire.
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
// Module registration
// ─────────────────────────────────────────────────────────────────────────────

/// Arcanum — Open Source CMoM Antenna Simulation Engine.
#[pymodule]
fn arcanum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    // Exception type
    m.add("ParseError", py.get_type_bound::<ParseError>())?;

    // NEC import types
    m.add_class::<PyParseWarning>()?;
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

    Ok(())
}
