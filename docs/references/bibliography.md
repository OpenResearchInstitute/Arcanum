# Bibliography

**Project:** Arcanum  
**Document:** `docs/references/bibliography.md`  
**Status:** DRAFT  
**Revision:** 0.1

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

## 4. Exact Kernel and Conformal MoM

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

### Vande Ginste, D., Franchois, A., and De Zutter, D.
"A Broadband Equivalent Transmission Line Model for the Analysis of Shielded Coaxial Cables"  
*IEEE Transactions on Advanced Packaging*, Vol. 30, No. 2, pp. 218–226, May 2007.  
DOI: 10.1109/TADVP.2007.896000  
**Note:** Cite the Vande Ginste et al. paper directly relevant to conformal MoM wire formulations — the specific paper referenced in the SOW should be identified and this entry updated with the correct title and DOI. The above is a placeholder entry.  
**Status:** ⚠ INCOMPLETE — correct paper citation required before this document is marked final.  
**Cited in:** sow.md

---

## 5. Ground Models

### Wait, J.R.
*Electromagnetic Waves in Stratified Media*, 2nd Edition  
Pergamon Press, Oxford, 1970.  
ISBN: 978-0080165912  
**Role:** Theoretical foundation for the Sommerfeld/Wait ground loss model used for lossy ground in NEC (IPERF = 2). The electrical parameters (conductivity, permittivity) stored by Phase 1 and consumed by Phase 2's lossy ground extension are evaluated using this theory.  
**Cited in:** sow.md, phase1-geometry/design.md (GN card, IPERF = 2)

---

## 6. Numerical Methods

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

## 7. Software and Crates

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

## 8. NEC Reference Decks

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
**Role:** ORI-specific antenna models purpose-built for Arcanum validation, including ORI's own antenna designs. Parameters chosen to match test conditions in `docs/nec-import/validation.md` and `docs/phase4-postprocessing/validation.md`. Provenance: ORI original work.  
**Planned decks:**
- `half-wave-dipole.nec` — standard half-wave dipole, Phase 3/4 validation
- `yagi-3el.nec` — 3-element Yagi-Uda, pattern validation
- `helix-axial.nec` — 5-turn axial-mode helix over ground, ORI primary use case
- Additional ORI-specific antenna designs (see GitHub Discussions)

---

## 9. Open Items

The following bibliography entries are incomplete and must be resolved before the document is marked final:

1. **Vande Ginste et al. CMoM paper** — the specific paper on conformal MoM wire formulations referenced in the SOW must be properly identified and the entry in Section 4 updated with the correct title, journal, and DOI. 

2. **Fikioris singularity extraction** — confirm that the 2001 IEEE TAP paper (Section 4) contains the specific self-impedance extraction formula needed for Phase 2 implementation. If a different Fikioris paper contains the relevant formula, add that entry and update the citation in `phase2-matrix-fill/math.md`. There are several different versions floating around. 

3. **NEC-2 distribution deck inventory** — identify the specific decks in the NEC-2 distribution that will serve as Tier 1 reference decks and list them here with deck names and antenna types. 

---

*Arcanum — Open Research Institute*
