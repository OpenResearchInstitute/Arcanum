# Bibliography

**Project:** Arcanum  
**Document:** `docs/references/bibliography.md`  
**Status:** DRAFT  
**Revision:** 0.2

---

## 1. Purpose

This document lists the foundational literature for the Arcanum project. Every reference cited in any Arcanum design document must appear here with complete publication details. References are organized by category.

---

## 2. Foundational MoM and Antenna Theory

### Harrington, R.F.
*Field Computation by Moment Methods*  
Macmillan, New York, 1968. Reprinted by IEEE Press, 1993.  
ISBN: 0-7803-1014-4  
**Role:** Foundational text for the Method of Moments. The mathematical basis for the EFIE formulation, Galerkin testing, and impedance matrix assembly used throughout Arcanum. Chapters 3–5 directly relevant to Phase 2 and Phase 3.  
**Cited in:** phase2-matrix-fill/math.md, phase2-matrix-fill/design.md, phase3-matrix-solve/math.md

---

### Balanis, C.A.
*Antenna Theory: Analysis and Design*, 3rd Edition  
Wiley-Interscience, Hoboken, NJ, 2005.  
ISBN: 0-471-66782-X  
**Role:** Primary reference for classical antenna results used in validation: half-wave dipole impedance (73.1 + j42.5 Ω), pattern formulas, directivity, near-field and far-field regions, radiation efficiency. Chapter 4 (linear wire antennas) and Chapter 2 (fundamental parameters) directly relevant.  
**Cited in:** phase3-matrix-solve/validation.md, phase4-postprocessing/math.md, phase4-postprocessing/validation.md

---

### Kraus, J.D. and Marhefka, R.J.
*Antennas: For All Applications*, 3rd Edition  
McGraw-Hill, New York, 2002.  
ISBN: 0-07-232103-2  
**Role:** Primary reference for axial-mode helix antenna design: gain formula, half-power beamwidth formula, input impedance approximation, axial ratio. Chapter 7 (helical antennas) directly relevant. Also reference for small loop radiation resistance formula.  
**Cited in:** phase3-matrix-solve/validation.md, phase4-postprocessing/math.md, phase4-postprocessing/validation.md, sow.md

---

### King, R.W.P. and Middleton, D.
"The Cylindrical Antenna; Current and Impedance"  
*Quarterly of Applied Mathematics*, Vol. 14, No. 2, pp. 175–188, 1956.  
**Role:** Analytic thin-wire dipole impedance results used as validation ground truth. The 73.1 + j42.5 Ω classical result is derived from this work.  
**Cited in:** phase3-matrix-solve/validation.md, sow.md

---

## 3. NEC-2 Reference Implementation

### Burke, G.J. and Poggio, A.J.
*Numerical Electromagnetics Code (NEC) — Method of Moments*  
Lawrence Livermore National Laboratory, UCID-18834, 1981.  
Available: public domain, distributed with 4NEC2 and xnec2c  
**Role:** Authoritative reference for the NEC-2 solver, card format definitions, and thin-wire kernel formulation. The baseline that Arcanum improves upon. The card field definitions in `docs/nec-import/card-reference.md` are derived from this document.  
**Cited in:** nec-import/card-reference.md, nec-import/design.md, phase2-matrix-fill/math.md, sow.md

---

### Burke, G.J.
*Numerical Electromagnetics Code (NEC-4) — Method of Moments*  
Lawrence Livermore National Laboratory, UCRL-MA-109338, 1992.  
**Role:** NEC-4 extends NEC-2 with improved ground modeling and other enhancements. Not open source but referenced for comparison with Arcanum's approach. The NEC-4 license restriction (Lawrence Livermore approval required) is one motivation for an open source CMoM alternative.  
**Cited in:** sow.md (background)

---

## 4. Boundary Element Methods — Ghent University Group

### Cools, K., Bogaert, I., Fostier, J., Peeters, J., Vande Ginste, D., Rogier, H., and De Zutter, D.
"Accurate and Efficient Algorithms for Boundary Element Methods in Electromagnetic Scattering — A Tribute to the Work of Professor F. Olyslager"  
*Radio Science*, draft manuscript, Ghent University, January 2012.  
**Role:** Overview of BEM/MoM research at Ghent University's Electromagnetics Group. Covers conformal discretization schemes for surface current methods (EFIE/MFIE), MLFMA, and parallel algorithms. The "conformal" discussed here refers to conforming finite element spaces for surface mesh BEM — distinct from Arcanum's curved wire segment CMoM. Included as background on the Ghent group and the broader conformal discretization literature.  
**Note:** This is NOT a wire antenna CMoM reference. The foundational curved-wire CMoM papers are Champagne et al. (1992) and Rogers & Butler (2001) — see Section 5 below. The original SOW reference to "Vande Ginste et al." for conformal wire MoM was incorrect and has been resolved.  
**Cited in:** bibliography.md (background context)

---

## 5. Curved Wire MoM — Foundational References

These are the papers on which Tony Golden's CMoM implementation in AN-SOF is based, as identified from his 2020 white paper (Section 7 below). They are the primary foundational references for Arcanum's Phase 2 matrix fill.

### Champagne, N.J. II, Williams, J.T., and Wilton, D.R.
"The Use of Curved Segments for Numerically Modeling Thin Wire Antennas and Scatterers"  
*IEEE Transactions on Antennas and Propagation*, Vol. 40, No. 6, pp. 682–689, June 1992.  
DOI: 10.1109/8.144597  
**Role:** The foundational paper introducing curved (conformal) segments for wire antenna MoM. Replaces straight-segment approximation with parametric curved segments, resolving the geometric modeling error for helices, loops, and arcs. This is the direct precursor to Arcanum's CMoM approach and is cited as reference [12] in Tony Golden's 2020 white paper.  
**Cited in:** phase2-matrix-fill/math.md, phase2-matrix-fill/design.md, sow.md

---

### Rogers, S.D. and Butler, C.M.
"An Efficient Curved-Wire Integral Equation Solution Technique"  
*IEEE Transactions on Antennas and Propagation*, Vol. 49, No. 1, pp. 70–79, January 2001.  
DOI: 10.1109/8.910530  
**Role:** Efficient implementation of the curved-wire integral equation. Develops practical quadrature strategies for the curved segment MoM, directly relevant to Phase 2's matrix fill implementation. Cited as reference [13] in Tony Golden's 2020 white paper alongside Champagne et al.  
**Cited in:** phase2-matrix-fill/math.md, phase2-matrix-fill/design.md

---

### Popovic, B.D. and Kolundzija, B.M.
*Analysis of Metallic Antennas and Scatterers*  
The Institution of Electrical Engineers, London, UK, 1994.  
**Role:** Comprehensive MoM reference for both wire and surface antenna analysis. Tony Golden cites this as reference [10] in his 2020 white paper as a primary MoM reference alongside Harrington. Directly relevant to the EFIE formulation and discretization strategy used in Arcanum.  
**Cited in:** phase2-matrix-fill/math.md (background)

---

### Song, J.M. and Chew, W.C.
"Moment Method Solutions Using Parametric Geometry"  
*Journal of Electromagnetic Waves and Applications*, Vol. 9, No. 1/2, pp. 71–83, January–February 1995.  
**Role:** Parametric geometry formulation for MoM — the mathematical framework for describing curved wire geometry as a parametric function, which underlies the conformal segment approach. Cited as reference [11] in Tony Golden's 2020 white paper.  
**Cited in:** phase2-matrix-fill/math.md (background)

---

## 6. Exact Kernel

### Fikioris, G. and Wu, T.T.
"On the Application of Numerical Methods to Hallén's Equation"  
*IEEE Transactions on Antennas and Propagation*, Vol. 49, No. 3, pp. 383–392, March 2001.  
DOI: 10.1109/8.918612  
**Role:** Exact kernel formulation and singularity extraction for self-impedance of cylindrical dipoles. The near-singular integration treatment in Phase 2 is based on this work. The self-impedance extraction formula referenced in `phase2-matrix-fill/math.md` Section 5.2 must be derived from this paper before Phase 2 implementation begins.  
**Cited in:** phase2-matrix-fill/math.md, phase2-matrix-fill/design.md

---

### Fikioris, G.
"The Exact Integral Equation for the Current on an Infinite Cylindrical Antenna"  
*IEEE Transactions on Antennas and Propagation*, Vol. 54, No. 7, pp. 2188–2191, July 2006.  
DOI: 10.1109/TAP.2006.877170  
**Role:** Further development of exact kernel treatment for cylindrical conductors. Companion to the 2001 paper.  
**Cited in:** phase2-matrix-fill/math.md

---

## 7. Golden Engineering White Paper

### Golden, T.
"Equivalent Wire-Grids for the Electromagnetic Modeling of Conducting Surfaces"  
Golden Engineering R&D, July 2020. Public Domain Mark 1.0.  
Available: https://archive.org/details/an-sof-equivalent-wire-grids  
**Role:** Tony Golden's own technical paper describing the CMoM implementation underlying AN-SOF. Establishes the Curvilinear Method of Moments (CMoM) framework for wire-grid surface modeling, derives the equivalent radius formula, and validates against NEC-2. The reference list in this paper (particularly references [10]–[13]) identifies the foundational curved-wire MoM literature on which AN-SOF's engine is based — and therefore on which Arcanum's Phase 2 is based. Key citations: Champagne et al. [12], Rogers & Butler [13], Popovic & Kolundzija [10], Song & Chew [11].  
**Cited in:** bibliography.md (provenance of foundational references)

---

## 8. Ground Models

### Wait, J.R.
*Electromagnetic Waves in Stratified Media*, 2nd Edition  
Pergamon Press, Oxford, 1970.  
ISBN: 978-0080165912  
**Role:** Theoretical foundation for the Sommerfeld/Wait ground loss model used for lossy ground in NEC (IPERF = 2). The electrical parameters (conductivity, permittivity) stored by Phase 1 and consumed by Phase 2's lossy ground extension are evaluated using this theory.  
**Cited in:** sow.md, phase1-geometry/design.md (GN card, IPERF = 2)

---

## 9. Numerical Methods

### Abramowitz, M. and Stegun, I.A.
*Handbook of Mathematical Functions*  
National Bureau of Standards, Applied Mathematics Series 55, 1964.  
Available: public domain  
**Role:** Reference for Gauss-Legendre quadrature nodes and weights (Table 25.4), and for special function evaluations. The `gauss-quad` Rust crate implements these tables.  
**Cited in:** phase2-matrix-fill/math.md, phase2-matrix-fill/design.md

---

### Press, W.H., Teukolsky, S.A., Vetterling, W.T., and Flannery, B.P.
*Numerical Recipes: The Art of Scientific Computing*, 3rd Edition  
Cambridge University Press, 2007.  
ISBN: 978-0521880688  
**Role:** General numerical methods reference: LU factorization, condition number estimation, adaptive quadrature. Chapter 2 (solution of linear algebraic equations) relevant to Phase 3.  
**Cited in:** phase3-matrix-solve/math.md

---

## 10. Software and Crates

### faer
Rust dense linear algebra library.  
Repository: https://github.com/sarah-ek/faer-rs  
**Role:** Dense matrix storage, LU factorization with partial pivoting, multiple right-hand side solve. Used in Phase 2 (ZMatrix storage) and Phase 3 (solver).  
**Cited in:** phase2-matrix-fill/design.md, phase3-matrix-solve/design.md

---

### rayon
Rust data parallelism library.  
Repository: https://github.com/rayon-rs/rayon  
Crates.io: https://crates.io/crates/rayon  
**Role:** Data-parallel matrix fill (Phase 2) and parallel pattern computation (Phase 4). The `par_iter()` pattern used throughout the hot paths.  
**Cited in:** phase2-matrix-fill/design.md, phase4-postprocessing/design.md, sow.md

---

### gauss-quad
Rust Gauss-Legendre quadrature crate.  
Repository: https://github.com/cgubbin/gauss-quad  
Crates.io: https://crates.io/crates/gauss-quad  
**Role:** Gauss-Legendre nodes and weights for numerical integration in Phase 2 matrix fill.  
**Cited in:** phase2-matrix-fill/design.md, sow.md

---

### nalgebra
Rust linear algebra library.  
Repository: https://github.com/dimforge/nalgebra  
Crates.io: https://crates.io/crates/nalgebra  
**Role:** 3D vector arithmetic, rotation matrices, coordinate transformations. Used in Phase 1 geometry and Phase 4 spherical coordinate projections.  
**Cited in:** phase2-matrix-fill/design.md, sow.md

---

### PyO3
Rust–Python bindings.  
Repository: https://github.com/PyO3/pyo3  
Crates.io: https://crates.io/crates/pyo3  
**Role:** Python bindings for the Arcanum Rust core. Enables Jupyter notebook access and integration with ORI's existing Python toolchain.  
**Cited in:** sow.md

---

## 11. NEC Reference Decks

The following NEC input deck collections are used as reference material for the `docs/nec-import/reference-decks/` test suite.

### NEC-2 Distribution Decks
*Numerical Electromagnetics Code (NEC) Example Input Decks*  
Lawrence Livermore National Laboratory, distributed with NEC-2, 1981.  
Available: public domain; bundled with 4NEC2, xnec2c  
**Role:** Original NEC-2 sample decks providing canonical reference inputs and expected outputs. Used as Tier 1 reference decks in the NEC import validation suite. Provenance: public domain, authoritative.

---

### Lewallen, R. (W7EL)
*EZNEC Antenna Software Example Models*  
Distributed with EZNEC v7.0, 2021. Now freeware.  
Available: https://www.eznec.com  
**Role:** Community-validated antenna models in NEC format. W7EL's example decks represent decades of practical antenna modeling and cover a wide range of antenna types. Used as Tier 2 reference decks. Provenance: freeware, W7EL credited.

---

### Open Research Institute
*ORI Arcanum Reference Decks*  
Synthesized by ORI contributors for Arcanum validation.  
Repository: `docs/nec-import/reference-decks/ori/`  
**Role:** ORI-specific antenna models purpose-built for Arcanum validation. Parameters derived from ORI's own antenna designs and modeling work. Provenance: ORI original work.  
**Planned decks:**
- `half-wave-dipole.nec` — standard half-wave dipole, Phase 3/4 validation
- `yagi-3el.nec` — 3-element Yagi-Uda, pattern validation
- `helix-axial.nec` — 5-turn axial-mode helix over ground, ORI primary use case
- `dumbbell-ori.nec` — ORI Dumbbell compact meander-loaded HF dipole (see below)

---

### Thompson, M. et al. (Open Research Institute)
*ORI Dumbbell Antenna — Compact Meander-Loaded HF Dipole*  
Open Research Institute, 2023. Repository: https://github.com/openresearchinstitute/dumbbell  
**Role:** ORI's compact HF antenna design using non-inductive meander loading sections to shorten a half-wave dipole. The radiating element is λ/6 on each side; the remaining wire is folded into 5 meander humps per arm wound around a cylindrical cowling. Primary design frequency 14.2 MHz (20m band); tested across 7–28 MHz. Field test data (SWR measurements) from Washington DC testing by Samudra Haque's team provides real measured performance to compare against. MATLAB Antenna Toolbox model (`dumbbell.m`) provides exact geometric parameters.

The Arcanum reference deck (`dumbbell-ori.nec`) is derived directly from the MATLAB model parameters:
- radiatorLength = λ/6 = 3.5187 m per arm
- NotchWidth (hump height) = 0.3265 m
- NotchLength (hump width) = 0.0254 m (1 inch)
- Wire radius = 0.001312 m

The dumbbell is a complementary test case to the helix: it is a pure straight-segment (GW) structure with dense junctions — 42 wires, approximately 80 junction points. It exercises Phase 1 junction handling and connectivity, Phase 3 impedance over a multi-band sweep, and Phase 4 pattern comparison against NEDA measurement data.

**Why this is a compelling ORI reference deck:**
- Pure wire structure — no dielectrics, no patches
- Dense junction network tests Phase 1 at scale
- Real measured SWR data exists for validation
- ORI's own design with full provenance
- Complementary to helix: straight-segment dense-junction vs curved-segment sparse-junction

**Cited in:** nec-import/validation.md (V-REAL reference decks), phase4-postprocessing/validation.md

---

## 12. Open Items

The following bibliography entries are incomplete and must be resolved before the document is marked final:

1. **Fikioris singularity extraction** — confirm that the 2001 IEEE TAP paper (Section 6) contains the specific self-impedance extraction formula needed for Phase 2 implementation. If a different Fikioris paper contains the relevant formula, add that entry and update the citation in `phase2-matrix-fill/math.md`. This is the Phase 2 implementation gate item.

2. **NEC-2 distribution deck inventory** — identify the specific decks in the NEC-2 distribution that will serve as Tier 1 reference decks and list them here with deck names and antenna types.

**Resolved items:**

- ~~Vande Ginste et al. CMoM wire reference~~ — Resolved. The correct foundational curved-wire MoM references are Champagne, Williams, Wilton (1992) and Rogers & Butler (2001), identified from Tony Golden's 2020 white paper. The SOW's reference to "Vande Ginste et al." for conformal wire MoM was incorrect. Section 5 has been updated accordingly.

---

*Arcanum — Open Research Institute*
