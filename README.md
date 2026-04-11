# Arcanum
Open Source MoM Solver for Antenna Design and Analysis

## Statement of Work: Open Source CMoM Antenna Simulation Engine

**Project:** Conformal Method of Moments (CMoM) Antenna Simulation Engine  
**Organization:** Open Research Institute (ORI)  
**License:** CERN-OHL-S-2.0 (hardware) / GPL-3.0-or-later (software)  
**Status:** Pre-development, Design and Documentation Phase  
**Revision:** 0.1

---

## 1. Background and Motivation

The Method of Moments (MoM) is the foundational numerical technique for wire antenna simulation. The dominant open source implementation, NEC-2, was released into the public domain by Lawrence Livermore National Laboratory and remains the engine underlying most accessible antenna modeling tools. However, NEC-2 uses straight wire segments and a thin-wire kernel approximation, which introduces systematic errors in the following categories. 

- Curved antennas (helics, loops, spirals)
- Closely spaced parallel wires (open-wire transmission lines, tightly wound helics)
- Thick conductors (if the wire radius a significant fraction of segment length)
- Bent wires at acute angles (segment-to-segment near-field interaction artifacts makes a mess)

The Conformal Method of Moments (CMoM) with Exact Kernel addresses all of these limitations. It replaces straight segments with curved conformal segments that follow the actual wire geometry, and replaces the thin-wire kernel with the full cylindrical surface integral, evaluated with a technique called adaptive numerical quadrature.

To the best of ORI's knowledge, no open source implementation of CMoM with Exact Kernel currently exists. The only known production implementation is proprietary (AN-SOF Antenna Simulator, Golden Engineering LLC). The goal of this project is to produce the first open source CMoM engine, making high-fidelity curved-wire antenna simulation available to the amateur radio satellite service, academic researchers, and the broader RF engineering community.

---

## 2. Project Scope

This project phase covers the **design and documentation work only**. No production code will be written until the design documents for all phases are complete, reviewed, and approved. The output of this phase is a repository of design documents that fully specify the architecture, mathematics, interfaces, and validation strategy for each development phase.

The subsequent development phases (not yet in scope) will implement the engine described. This means we are following a 1. design 2. document 3. code and 4. test process.

---

## 3. Computational Structure

The MoM/CMoM engine decomposes naturally into four sequential computational phases. Each phase is a candidate for independent development, testing, and validation. This decomposition drives the project structure and makes it more possible for open source contributors to step in and participate. Many hands make light work. 

### Phase 1 Geometry Discretization

**What it does:** Accepts a wire structure description and produces a segment mesh. This is a set of conformal (curved) cylindrical segments, each described by its geometry, material properties, and connectivity.

**Inputs:** Wire descriptions (endpoints, radius, material, number of segments), source locations, load locations.

**Outputs:** Ordered list of conformal segments with parametric descriptions; connectivity graph; source and load maps.

**Key design decisions:**
- Native wire description format (internal representation)
- Parametric representation of curved segments (arc, helix, spline)
- Segment length guidelines relative to wavelength
- Handling of wire junctions and T-connections
- NEC input deck parsing (see Section 6!)

**Validation:** Segment counts, geometric continuity at junctions, visual inspection of discretized geometry against analytic wire descriptions.

---

### Phase 2 Matrix Fill

**What it does:** Computes every element Z[m,n] of the N×N impedance matrix, where Z[m,n] is the interaction between testing function m and basis function n via the exact kernel Green's function integral. Like matrices? Good! Because that's what this is all about. 

**Inputs:** Segment mesh from Phase 1; frequency.

**Outputs:** Dense complex N×N impedance matrix [Z].

**Computational character:** O(N²). Embarrassingly parallel. Every element is independent. This is the primary target for parallelization via Rayon.

#### What is Rayon?

Glad you asked!

Rayon is a Rust library (crate) for data parallelism. It lets you parallelize iterator-based code with minimal changes — often just replacing .iter() with .par_iter().
The classic example is this:

```
// Sequential
segments.iter().for_each(|s| compute(s));

// Parallel with Rayon — one word change
segments.par_iter().for_each(|s| compute(s));
```

Rayon handles all the thread pool stuff, the so-called "work stealing", and load balancing all under the hood. You don't spawn threads manually or manage synchronization. You just describe what's independent and Rayon figures out how to distribute it across cores. This is why we're doing this in Rust instead of C++. 

Why it matters for Arcanum specifically?

The matrix fill is the perfect Rayon workload. You have N² elements to compute, every single one is independent of every other, and each computation (a Gauss-Legendre quadrature integral) is non-trivial enough to justify the parallelism overhead. In pseudocode:

```
z_matrix.par_iter_mut()
    .enumerate()
    .for_each(|(idx, element)| {
        let m = idx / n_segments;
        let n = idx % n_segments;
        *element = compute_zmn(segments[m], segments[n], frequency);
    });
```

Every core on your machine fills its share of the matrix simultaneously. On an 8-core machine you get roughly 8x speedup on the dominant bottleneck essentially for free. This is a really great thing. 

The D&D analogy (you knew one was coming): if the matrix fill were a dungeon map that you had to draw, sequential iteration is one cartographer drawing every room alone. Rayon is handing out sections of the map to every cartographer you can find, all simultaneously, with no coordination needed because the rooms don't depend on each other.

It's one of the most loved crates in the Rust ecosystem precisely because the payoff-to-effort ratio is extraordinary. We are going to take advantage of that ratio in Arcanum.

**Key design decisions:**
- Exact kernel formulation (full cylindrical surface integral vs. thin-wire approximation?)
- Adaptive Gauss-Legendre quadrature order for non-singular elements
- Special treatment of self-impedance (singular) and near-neighbor (near-singular) elements
- Parallelization strategy (element-level, row-level? some other level?)
- Numerical precision targets (how good is good enough?)

**Validation:** Self-impedance of a thin straight segment against analytic thin-wire result (should converge to thin-wire limit as radius goes to 0); mutual impedance between parallel segments against published tables; matrix symmetry checks.

---

### Phase 3 Matrix Solve

**What it does:** Solves the linear system [Z][I] = [V] for the current vector [I], given the excitation vector [V] assembled from source definitions. Sounds easy, right?

**Inputs:** Impedance matrix [Z] from Phase 2; excitation vector [V] from source definitions. Now we're cooking. 

**Outputs:** Complex current vector [I] — one current amplitude per segment. 

**Computational characteristics:** Complexity is O(N³) for direct LU factorization. For antenna sizes relevant to ORI (ground station antennas maybe, terrestrial antennas and feeds, yagis, modest arrays for 219 MHz, etc.), direct solve via LU factorization is the initial target. Iterative solvers (GMRES) are a future extension.

**Key design decisions:**
- LU factorization via `faer` crate (Rust dense linear algebra)
- Excitation vector assembly (delta-gap voltage source model; current source model)
- Multiple right-hand sides (frequency sweep efficiency)
- Condition number monitoring and ill-conditioning diagnostics (can't have it go off the rails)

**Validation:** Input impedance of a half-wave dipole is 73 + j42.5 Ω (classical result); small loop radiation resistance has a closed-form analytic result; two-wire transmission line impedance to exact transmission line theory.

---

### Phase 4 Post-Processing

**What it does:** Computes all observable antenna parameters from the solved current distribution [I]. 

**Inputs:** Current vector [I] from Phase 3; segment mesh from Phase 1; frequency; observation geometry.

**Outputs:** Input impedance, VSWR, radiated power, gain, directivity, radiation patterns (2D and 3D), near fields, radar cross section (RCS). This is the stuff we care about. 

**Computational character:** Fast relative to Phases 2 and 3. Near-field computation at many observation points can be parallelized but is not the bottleneck.

**Key design decisions:**
- Far-field integration method?
- Near-field computation grid specification?
- Pattern normalization conventions??
- Output format (native? interface to plotting layer so people can use their own??)

**Validation:** Half-wave dipole pattern shape (figure-8 in E-plane, omnidirectional in H-plane); isotropic radiator gain = 0 dBi; energy conservation check (radiated power vs. input power). The Hello World of atenna design. 

---

## 4. Architecture

### 4.1 Language and Runtime

Best of both worlds with Rust and Python. 

| Layer | Technology | Rationale |
|---|---|---|
| Core engine | Rust | Performance, memory safety, parallelism via Rayon |
| Python bindings | PyO3 | Jupyter notebook usability, integration with existing ORI tooling |
| Scripting / post-processing | Python | NumPy, Matplotlib, existing link budget notebooks |
| Linear algebra | `faer` (Rust) | Dense LU, benchmarks competitive with LAPACK |
| Parallelism | `rayon` (Rust) | Data-parallel matrix fill with zero-synchronization |
| Quadrature | `gauss-quad` (Rust) | Gauss-Legendre nodes and weights |
| Geometry | `nalgebra` (Rust) | 3D vector math, parametric curves |

### 4.2 Layered Architecture

```
┌─────────────────────────────────────┐
│   Python API / Jupyter interface    │  gives user-facing: notebooks, scripts, visuals
├─────────────────────────────────────┤
│         PyO3 binding layer          │  has type conversion, error mapping
├─────────────────────────────────────┤
│         Rust core library           │
│  ┌───────────┐  ┌────────────────┐  │
│  │  Phase 1  │  │    Phase 2     │  │
│  │ Geometry  │→ │  Matrix Fill   │  │
│  │   (mesh)  │  │ (Rayon parallel│  │
│  └───────────┘  │  exact kernel) │  │
│                 └───────┬────────┘  │
│  ┌───────────┐          │           │
│  │  Phase 3  │←─────────┘           │
│  │   Solve   │                      │
│  │  (faer LU)│                      │
│  └─────┬─────┘                      │
│        ↓                            │
│  ┌───────────┐                      │
│  │  Phase 4  │                      │
│  │   Post-   │                      │
│  │ Processing│                      │
│  └───────────┘                      │
└─────────────────────────────────────┘
```

### 4.3 Interface Philosophy

Each phase exposes a clean data boundary with inputs and outputs defined like ports in VHDL modules. Phases communicate through well-defined structs, not shared mutable state. This mirrors the AXI-Stream philosophy used in ORI's FPGA modem work. We work hard to have well-defined handshake points between independent processing stages.

---

## 5. Validation Strategy

Validation is not an afterthought. Each phase has analytic ground truth available from classical antenna theory. The validation suite is part of the design document for each phase and must be specified before implementation begins. The answers are findable and well described. We won't have to guess that it works because we can validate at each phase.  

### Primary Validation Cases

| Case | Phase | Expected Result | Source |
|---|---|---|---|
| Half-wave thin dipole impedance | 2, 3 | 73.1 + j42.5 Ω | Classical; King-Middleton |
| Small circular loop radiation resistance | 2, 3 | Closed form: R = 20π²(C/λ)⁴ | Classical |
| Small square loop radiation resistance | 2, 3 | Same limit as circular | Shape independence |
| Two-wire transmission line impedance | 2, 3 | Z₀ = (120/√εᵣ)·ln(D/d) | Transmission line theory |
| Half-wave dipole far-field pattern | 4 | cos²(θ/2)/sin(θ) shape | Classical |
| Helix axial mode impedance | 2, 3, 4 | ~140 Ω (Kraus) | Kraus, Antenna Theory |

**Note on NEC-2 as a validation reference:** NEC-2 is a different solver (straight-segment, thin-wire kernel MoM) and is not used as a primary validation target. In the degenerate limit of straight wires with very small radius-to-segment-length ratios, CMoM must converge to the same result as the thin-wire approximation. This is a necessary but minimal sanity check, not a correctness standard. All primary validation is against analytic closed-form results from classical antenna theory.

### Convergence Testing

For each validation case, results must be demonstrated to converge monotonically as segment count N increases. Convergence plots are required deliverables for each phase design document.

---

## 6. NEC Input Deck Import

### 6.1 Motivation

The `.nec` file format is the lingua franca of wire antenna modeling. Decades of community antenna models exist as `.nec` files. These files come from EZNEC users, 4NEC2 users, academic NEC-2 runs, and direct NEC card deck authors. Supporting `.nec` import means every one of those models can be opened in this engine immediately, with no remodeling effort, and simulated with higher accuracy than the original NEC-2 solver could provide. 

This is a first-class adoption requirement, not a compatibility convenience.

### 6.2 What a .nec File Is

A `.nec` file is a complete simulation input deck. It is **not** only a geometry description. It contains all information needed to run a complete simulation: geometry, excitation, loads, ground model, frequency sweep, and output requests. It uses a card-based format inherited from punched-card computing, where each line begins with a two-letter card mnemonic. 

The `.nec` format is entirely independent of the NEC-2 solver. It is a file interchange format that this engine reads and maps onto its own internal data structures. We do not use NEC-2 as a solver at any point.

### 6.3 Card Types and Phase Mapping

Each card type maps to a specific phase's data structures. The parser is a frontend that fans out into the pipeline.

| Card | Description | Phase |
|---|---|---|
| `GW` | Straight wire segment | 1 — Geometry |
| `GA` | Arc (curved wire) | 1 — Geometry |
| `GH` | Helix | 1 — Geometry |
| `GM` | Geometry move/rotate/scale | 1 — Geometry |
| `GS` | Geometry scale | 1 — Geometry |
| `GE` | Geometry end (required terminator) | 1 — Geometry |
| `EX` | Excitation (voltage or current source) | 3 — Matrix Solve |
| `LD` | Load (impedance on segment) | 3 — Matrix Solve |
| `FR` | Frequency (single or sweep) | 2, 3 — Matrix Fill / Solve |
| `GN` | Ground definition | 1, 2 — Geometry / Matrix Fill |
| `RP` | Radiation pattern output request | 4 — Post-Processing |
| `NE` | Near electric field output request | 4 — Post-Processing |
| `NH` | Near magnetic field output request | 4 — Post-Processing |
| `EN` | End of input deck | Parser |

### 6.4 Design Requirements

- The parser must be a standalone module with no dependency on phase implementations. It produces intermediate data structures that each phase consumes independently.
- Unknown or unsupported cards must produce a warning, not a silent failure or crash.
- The parser must round-trip: a model loaded from `.nec` and re-exported should produce a valid `.nec` file that NEC-2 can also read (export is a secondary goal but the round-trip property constrains the internal representation).
- `GW` (straight wire) is the minimum viable card set for initial implementation. `GA` and `GH` are high priority for ORI use cases (helix ground station antennas).

### 6.5 What NEC Import Is Not

NEC import does not imply NEC-2 solver compatibility. The CMoM engine will produce different (more accurate) results than NEC-2 on the same input geometry. This is expected and desirable — particularly for curved geometry cards (`GA`, `GH`) where NEC-2's straight-segment approximation degrades accuracy. Users should expect improved results, not identical results.

### 6.6 Repository Structure Segement

```
docs/
├── nec-import/
│   ├── design.md       Parser architecture, card routing, error handling
│   ├── card-reference.md  Supported cards with field definitions
│   └── validation.md   Round-trip tests, known-good .nec reference decks
```

---

## 7. Repository Structure (Design Phase)

```
cmom-engine/
├── README.md
├── LICENSE
├── docs/
│   ├── sow.md                        This document
│   ├── nec-import/
│   │   ├── design.md                 Parser architecture, card routing, error handling
│   │   ├── card-reference.md         Supported cards with field definitions
│   │   └── validation.md             Round-trip tests, known-good .nec reference decks
│   ├── phase1-geometry/
│   │   ├── design.md                 Wire format, segment types, junction handling
│   │   ├── math.md                   Parametric curve representations
│   │   └── validation.md             Geometric test cases
│   ├── phase2-matrix-fill/
│   │   ├── design.md                 Kernel formulation, quadrature strategy
│   │   ├── math.md                   Exact kernel integral derivation
│   │   └── validation.md             Impedance matrix test cases
│   ├── phase3-matrix-solve/
│   │   ├── design.md                 LU solver, excitation assembly
│   │   ├── math.md                   EFIE formulation, excitation models
│   │   └── validation.md             Dipole impedance, loop resistance
│   ├── phase4-postprocessing/
│   │   ├── design.md                 Field integrals, pattern computation
│   │   ├── math.md                   Far-field and near-field formulas
│   │   └── validation.md             Pattern shape, energy conservation
│   └── references/
│       └── bibliography.md           Key literature
├── examples/                         Placeholder for future worked examples
└── CONTRIBUTING.md
```

---

## 8. Key References

Key reference materials for citations and review. Thank you to Microwave Update for the list. 

- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — foundational MoM text
- Burke & Poggio — *NEC-2 Method of Moments Code* (1981) — NEC-2 reference implementation
- Fikioris, G. — exact kernel formulations for cylindrical antennas
- Vande Ginste et al. — higher-order and conformal MoM formulations
- Kraus, J.D. — *Antennas* — validation reference for helix and loop cases
- King & Middleton — thin dipole impedance analytic results
- Wait, J.R. — ground loss models for LF/MF antennas

---

## 9. Success Criteria for Design Phase

The design phase is complete when:

1. All four phase design documents (geometry, matrix fill, matrix solve, post-processing) are written, reviewed, and merged to main.
2. The NEC import design document is written, reviewed, and merged to main, including the card reference and validation cases.
3. All mathematical derivations are present in the `math.md` files with sufficient detail that an independent implementer could write the code from the document alone.
4. All validation cases are specified with expected numerical results and convergence criteria.
5. The repository structure is established and documented.
6. At least two ORI contributors have reviewed each design document.

---

*Document maintained by Open Research Institute. Contributions welcome under our participant and developer policies at https://www.openresearch.institute/developer-and-participant-policies/.*
