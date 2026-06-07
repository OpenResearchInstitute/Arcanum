# NEC2 Input Format Parsing — Strategy Summary

## What the docs already specify

**[design.md](design.md)** states free-field whitespace-delimited is the primary parsing strategy, with this explicit requirement:

> "Arcanum must handle both" column-based and free-field formats.

**[validation.md](validation.md)** test case V-FMT-002 specifies: "Fields padded to column positions. Pass criterion: fields parsed identically to V-FMT-001." — meaning the parser must handle column-format input and produce the same result as free-field input.

---

## Why whitespace tokenization already satisfies "handle both"

A column-format file with explicit values is just a free-field file with wider whitespace. A whitespace tokenizer handles it transparently:

```
GW    1   10 0.000000  0.000000  0.000000  1.000000  ...  ← column format
GW 1 10 0.0 0.0 0.0 1.0 ...                              ← free-field format
```

Both tokenize to the same sequence: `["GW", "1", "10", "0.0", ...]`. No heuristic, no caller parameter needed for this case.

---

## The one real edge case: blank fields in column format

Whitespace tokenization silently **shifts** when a field is genuinely blank in column format. If field 3 is blank (all spaces in its column slice), the tokenizer sees field 4's value and assigns it to field 3. This is silent corruption, not a parse error.

Whether this matters for Arcanum depends on which fields can legitimately be blank in NEC2 column-format files. Looking at Arcanum's supported card set against the blank-vs-zero analysis:

| Card | Field | NEC2 blank means | Arcanum's definition |
|------|-------|-----------------|---------------------|
| EX | F3 | normalize to max | **Not in Arcanum's EX field list** (6 fields: EXTYPE, tag, seg, unused, V_re, V_im) |
| NE/NH | I2/I3/I4 | default 1 | All required; hard error if missing |
| GW | I1 (NS) | reset symmetry | Hard error if NS < 1 |
| FR | I2 (NFRQ) | 1 frequency | Hard error if NFRQ < 1 |
| GS | I1/I2 | not used | Card reference says "set to 0" explicitly |

Arcanum's simplified card definitions eliminate most of these. The remaining risk is NE/NH counts — if a legacy column-format file has blank count fields meaning "default to 1," the tokenizer would shift and corrupt subsequent float fields silently.

---

## Recommendation

**Primary path: whitespace tokenizer.** It handles both formats for all practical inputs (xnec2c-generated files, the reference decks in the repo). No heuristic detection needed.

**Residual risk: blank fields in strict NEC2 column-format files.** For Arcanum's supported card set this reduces to NE/NH count fields. The design doc's "hard error on missing required fields" would catch the shift as a type error (a float where an int is expected), so it fails loudly rather than silently.

**If strict NEC2 column-format import is needed:** add an optional `InputFormat` parameter to `parse()` with variants `Auto`, `FreeField`, and `ColumnBased`. The `ColumnBased` path uses fixed slice offsets and returns `None` for blank fields; the caller (or the card-specific validator) maps `None` to the NEC2 spec defaults. This would be an explicit opt-in for users known to be working with legacy files.

Given that Arcanum's reference decks and card-reference.md are written for xnec2c-style free-field output, `Auto` defaulting to whitespace tokenization is the right call. The `ColumnBased` variant can be deferred until a real use case surfaces.
