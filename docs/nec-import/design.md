# NEC Import — Design

**Project:** Arcanum  
**Document:** `docs/nec-import/design.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document describes the architecture of the NEC import parser. This is the front door to Arcanum. The parser accepts a `.nec` input deck and produces a `SimulationInput` struct that all four phases consume. It also produces a `ParseWarnings` list of non-fatal issues encountered during parsing.

The parser is a pipeline with two cleanly separated stages:

1. **Lexical stage** — reads raw text, identifies card mnemonics, parses fields into typed values. Produces a `ParsedDeck`.
2. **Semantic stage** — validates card ordering and cross-references, applies geometry transformations, splits the GN card, and routes each card's data to the correct phase's input structure. Produces a `SimulationInput`.

This two-stage design means the lexical parser and the semantic router can be developed, tested, and reasoned about independently. The `ParsedDeck` is the boundary between them.

---

## 2. Entry Point

The public API of the NEC import module is a single function:

```rust
pub fn parse(input: &str) -> Result<(SimulationInput, ParseWarnings), ParseError>
```

- `input` — the full text of the `.nec` deck as a string slice. The caller is responsible for reading the file; the parser does not do I/O.
- On success: returns `(SimulationInput, ParseWarnings)`. Warnings may be non-empty even on success.
- On hard error: returns `Err(ParseError)` describing the first fatal error encountered. Parsing aborts at the first hard error.

A thin file-reading wrapper is provided for convenience:

```rust
pub fn parse_file(path: &Path) -> Result<(SimulationInput, ParseWarnings), ParseError>
```

This reads the file to a string and calls `parse`. It is not independently tested. All parser tests use `parse` with string literals.

---

## 3. Stage 1 — Lexical Parsing

### 3.1 Responsibilities

The lexical stage reads the input line by line and produces an ordered list of typed card structs. It is responsible for:

- Normalizing line endings (`\r\n` → `\n`)
- Stripping and ignoring CM/CE comment lines
- Identifying the two-character card mnemonic on each line
- Tokenizing the remaining fields as whitespace-delimited tokens
- Parsing integer fields to `i32` and float fields to `f64`
- Returning `NecCard::Unknown(mnemonic)` for unrecognized card types
- Hard-erroring on malformed fields (non-numeric where numeric expected)

The lexical stage does not validate card ordering, check tag references, or apply transformations. It does only what is listed above.

### 3.2 The NecCard Enum

Each recognized card type maps to a strongly-typed struct variant:

```rust
enum NecCard {
    Gw(GwCard),
    Ga(GaCard),
    Gh(GhCard),
    Gm(GmCard),
    Gs(GsCard),
    Ge(GeCard),
    Gn(GnCard),
    Ex(ExCard),
    Ld(LdCard),
    Fr(FrCard),
    Rp(RpCard),
    Ne(NeCard),
    Nh(NhCard),
    En,
    Unknown(String),   // mnemonic preserved for warning emission
}
```

Each card struct holds exactly the fields defined in `card-reference.md` as typed Rust values. No raw strings survive past the lexical stage.

### 3.3 The ParsedDeck

The output of the lexical stage is:

```rust
struct ParsedDeck {
    cards: Vec<(usize, NecCard)>,   // (line number, card) — line number for error reporting
}
```

The line number is preserved alongside each card so that semantic-stage error messages can identify the source line in the original input.

### 3.4 Field Parsing Rules

- Leading/trailing whitespace on each line is stripped before tokenizing.
- Fields are separated by one or more whitespace characters (spaces or tabs).
- Integer fields are parsed with `i32::from_str`. Float fields are parsed with `f64::from_str`, which handles scientific notation (`2.5E-1`, `1.0e3`).
- If a required field is missing (too few tokens on the line) or cannot be parsed as the expected type, the lexical stage emits a hard error identifying the card mnemonic, line number, field position, and the offending token (or "missing").
- Trailing fields beyond the expected count are silently ignored (common in NEC-2 decks that pad lines).

---

## 4. Stage 2 — Semantic Routing

### 4.1 Responsibilities

The semantic stage consumes the `ParsedDeck` and produces a `SimulationInput`. It is responsible for:

- Validating card ordering (geometry cards before GE; simulation cards after GE)
- Checking that GE is present (hard error if absent)
- Building the tag registry from GW/GA/GH cards and checking for duplicate tags
- Applying GS and GM transformations to wire geometry
- Resolving EX and LD tag/segment references against the tag registry
- Splitting the GN card: geometric boundary condition → `MeshInput`; electrical parameters → `GroundElectrical`
- Assembling the frequency list from FR cards
- Assembling output requests from RP/NE/NH cards
- Emitting warnings for unknown cards, unsupported card types, and suspicious conditions
- Routing each card's data to the correct field of `SimulationInput`

### 4.2 The SimulationInput Struct

```rust
pub struct SimulationInput {
    pub mesh_input: MeshInput,                        // → Phase 1
    pub frequencies: Vec<f64>,                        // → Phase 2, Phase 3 (Hz)
    pub sources: Vec<SourceDefinition>,               // → Phase 3
    pub loads: Vec<LoadDefinition>,                   // → Phase 3
    pub ground_electrical: Option<GroundElectrical>,  // → Phase 2
    pub output_requests: OutputRequests,              // → Phase 4
}
```

Each field is consumed independently by the relevant phase. No phase receives the full `SimulationInput`. Each phase receives only its own field. This enforces the phase boundary documented in `docs/phase1-geometry/design.md` Section 9.

### 4.3 The MeshInput Struct

`MeshInput` is the input to Phase 1's geometry processor. It contains the raw wire descriptions and ground boundary condition, before discretization:

```rust
pub struct MeshInput {
    pub wires: Vec<WireDescription>,         // From GW, GA, GH (after GS/GM applied)
    pub ground: GeometricGround,             // From GN (geometric portion only)
    pub gpflag: i32,                         // From GE
}
```

`WireDescription` is a union type covering straight, arc, and helix wires. It carries the tag number, segment count, geometric parameters, and wire radius. It does not carry source or load information. Those are in `sources` and `loads`.

### 4.4 GN Card Split

The GN card is split in the semantic stage, not in the lexical stage. After lexical parsing, the `GnCard` struct holds all fields. The semantic stage splits it:

```rust
// Geometric portion → MeshInput
mesh_input.ground = GeometricGround {
    ground_type: map_iperf(gn.iperf),
};

// Electrical portion → SimulationInput (for Phase 2)
if gn.iperf == 0 || gn.iperf == 2 {
    simulation_input.ground_electrical = Some(GroundElectrical {
        permittivity: gn.epse,
        conductivity: gn.sig,
        model: map_ground_model(gn.iperf),
    });
}
```

The `GnCard` struct is consumed entirely in the semantic stage and does not appear in `SimulationInput`. Neither Phase 1 nor Phase 2 sees the raw card. They see only their respective derived structs. This is the implementation of the split documented in `docs/phase1-geometry/design.md` Section 6.1.

### 4.5 Transformation Application

GS and GM cards are applied eagerly in the semantic stage before `MeshInput` is constructed. The transformation pipeline is:

1. Collect all wire descriptions from GW/GA/GH cards in deck order.
2. Apply GS scale factor to all wire endpoint coordinates (not radii).
3. Apply GM transformations in deck order, each referencing wires by tag.

After this pipeline, `MeshInput.wires` contains final transformed coordinates. No transformation records are preserved in `MeshInput`. Phase 1 sees only the result.

### 4.6 Tag Registry

The semantic stage builds a tag registry as it processes geometry cards:

```rust
struct TagRegistry {
    entries: HashMap<u32, TagEntry>,
}

struct TagEntry {
    wire_index: usize,    // index into MeshInput.wires
    segment_count: u32,   // NS from the card
    line_number: usize,   // for error reporting
}
```

Duplicate tags produce a hard error immediately when the duplicate is encountered. EX and LD cards are validated against the registry after all geometry cards are processed. The tag and segment number must exist. Unknown tags and out-of-range segment numbers are hard errors.

### 4.7 Frequency Assembly

Multiple FR cards are valid. The semantic stage assembles all FR cards into a single frequency list in the order they appear:

```rust
pub struct FrequencyList(Vec<f64>);   // Hz, not MHz. The conversion is applied here
```

MHz to Hz conversion is applied in the semantic stage. All downstream phases work in Hz. No phase should contain MHz-to-Hz conversion logic.

---

## 5. Error and Warning Architecture

### 5.1 ParseError

Hard errors abort parsing immediately and return `Err(ParseError)`. The `ParseError` type carries:

```rust
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub line: usize,
    pub message: String,    // human-readable, includes card type and field context
}

pub enum ParseErrorKind {
    MissingGeCard,
    DuplicateTag,
    ZeroSegmentCount,
    ZeroLengthWire,
    UnknownTagReference,
    SegmentOutOfRange,
    FieldParseFailure,
    InvalidFieldValue,
    MultipleGnCards,
    GeometryAfterGe,
}
```

### 5.2 ParseWarnings

Warnings are collected throughout parsing and returned alongside the successful result:

```rust
pub struct ParseWarnings(Vec<ParseWarning>);

pub struct ParseWarning {
    pub kind: ParseWarningKind,
    pub line: usize,
    pub message: String,
}

pub enum ParseWarningKind {
    UnknownCard,
    UnsupportedExType,
    UnsupportedCard,
    NradlIgnored,
    MissingEnCard,
    NearCoincidentEndpoints,
    WireInGroundPlane,
}
```

`ParseWarnings` implements `is_empty()` and iteration. The caller decides whether to display, log, or ignore warnings. The parser never suppresses a warning silently.

---

## 6. What the Parser Does Not Do

The parser does not:

- **Discretize wire segments.** That is Phase 1's job. The parser produces `WireDescription` structs with segment counts; Phase 1 produces the actual `Segment` mesh.
- **Compute geometry.** No arc lengths, no helix endpoints, no junction detection. The parser reads numbers off cards and stores them.
- **Do any electromagnetic computation.** The parser has no knowledge of frequency-dependent behavior, even though it reads FR cards.
- **Re-parse cards downstream.** Once a card is parsed and routed, it is consumed. No phase accesses raw card text.
- **Perform I/O beyond reading the input string.** File reading is the caller's responsibility.

---

## 7. Relationship to Phase 1

The parser and Phase 1 are adjacent but distinct modules. Their boundary is `MeshInput`:

```
parse(input: &str)
    → SimulationInput {
        mesh_input: MeshInput,   ← parser produces this
        ...
    }

phase1::discretize(mesh_input: MeshInput)
    → Mesh                       ← Phase 1 produces this
```

Phase 1 does not call the parser. The parser does not call Phase 1. The caller (the top-level simulation runner, or a test) calls them in sequence.

This boundary means the parser and Phase 1 can be tested independently. Parser tests (this document) feed `MeshInput` directly to assertions. Phase 1 tests (`docs/phase1-geometry/validation.md`) construct `MeshInput` programmatically without going through the parser. Trust us. This will make writing this a lot easier.

---

## 8. Anticipated Repository Layout

```
src/
├── nec_import/
│   ├── mod.rs              ← pub fn parse(), pub fn parse_file()
│   ├── lexer.rs            ← Stage 1: text → ParsedDeck
│   ├── cards.rs            ← NecCard enum and card structs
│   ├── router.rs           ← Stage 2: ParsedDeck → SimulationInput
│   ├── tag_registry.rs     ← Tag registry used by router
│   ├── errors.rs           ← ParseError, ParseWarnings
│   └── tests/
│       ├── parse_tests.rs  ← V-PARSE-XXX cases
│       ├── fmt_tests.rs    ← V-FMT-XXX cases
│       ├── route_tests.rs  ← V-ROUTE-XXX cases
│       ├── error_tests.rs  ← V-ERR-XXX cases
│       ├── warn_tests.rs   ← V-WARN-XXX cases
│       └── real_tests.rs   ← V-REAL-XXX cases (loads from reference-decks/)
```

---

## 9. Open Questions

1. **Encoding:** NEC-2 decks are ASCII. Should the parser reject non-ASCII bytes or silently replace them? Most real-world decks are clean ASCII but some have been through word processors. Recommendation: reject non-ASCII with a hard error and a clear message.

2. **Maximum deck size:** No limit is specified. A deck with millions of wire segments would produce a very large `MeshInput`. Should the parser enforce a maximum segment count as a safety check? Recommendation: warn (not error) above a configurable threshold.

3. **GR card (geometry rotate):** GR generates copies of a wire by rotation, similar to GM. It is listed as unsupported in `card-reference.md`. However, GR appears in many real-world Yagi and log-periodic decks. It may need to be promoted from deferred to initial scope. Recommendation: open a GitHub Discussion, post to mailing list, and ask on Slack, etc.

---

## 10. References

- `docs/nec-import/card-reference.md` — field definitions for all supported cards
- `docs/nec-import/validation.md` — test cases
- `docs/phase1-geometry/design.md` — Phase 1 interface; MeshInput consumption
- Burke & Poggio, *NEC-2 Method of Moments Code* (1981)

---

*Arcanum — Open Research Institute*
