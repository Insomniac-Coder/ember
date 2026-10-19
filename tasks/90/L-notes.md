# Job L — layout attributes (`[LAY-2]`): notes

Helper agent L, 2026-10-19. Branch `helper-L`. Everything below is built, tested on Windows (MSVC,
the conformance runner and the Rust suites) and compiled with gcc 15 in WSL (the emitted C of every
new run-pass test and the runtime, never run there), and committed on the branch.

`[LAY-2]` is **not** done: two layouts it defines are refused with `E0900` (section 3), so it keeps an
`unfinished` entry in `docs/phase-status.json`. Everything else the rule says is built and tested.

## 1. What was built, feature by feature

### L1. Where the attributes live (`ember_types`)

* `LayoutAttrs { packed, align }` (align 1 when none is written), recorded per declared name in a
  side table of the type table (`declare_layout`, `layout_attrs(ty)`), as `@must_drop` and
  `@derive(Zeroable)` already are: a generic struct's or enum's instances share it through their
  origin. Decision: a side table rather than a field on `StructDef`/`EnumDef`, which 41 places build;
  the side table touches the four places a declaration is collected and nothing else.
* `TypeTable::layout` applies them: `@packed` places each field right after the one before and makes
  the type's alignment 1 (no trailing padding); `@align(N)` raises the alignment to `max(natural, N)`
  and rounds the size up. A packed payload enum puts its union straight after the tag and packs each
  variant. Types without attributes compute exactly as before.
* `stated_alignment(ty)`: whether a type's inline storage holds an `@align` type or a SIMD vector
  (used by the refusals in section 3).

### L2. Checking the attributes (`ember_typeck`)

* The attribute table marks `@packed` and `@align` built. `@packed` takes no arguments, `@layout`
  takes exactly `c`, `@align` takes one argument: anything else is `E0104`.
* `@align(N)`: `N` is a constant expression (the `const_arg` evaluator array lengths use, so
  `@align(LINE)` works where `LINE` is declared before the type, as for array lengths); not a power of
  two, zero, or above 4096 is `E0104`. Decision: `E0104`, the code the specification uses for an
  attribute's invalid arguments elsewhere (`[ATT-1]`, a module attribute's invalid arguments); the rule
  names no other.
* `@repr(T)`: one of `u8 u16 u32 u64 i8 i16 i32 i64`, by name or alias (`int` is `i64`); anything
  else (`usize`, `u128`, `f32`, …) is `E0104` (it was accepted when `is_integral`). A discriminant that
  does not fit `T`, written or counted on from the previous one, is `E2010` (it was truncated in C:
  `@repr(u8)` with `A = 300` compiled to 44).
* `@repr` on a struct is `E0901` (was `E0900`, "not implemented yet"): `[LAY-2]` gives `@repr` a
  meaning only as an enum's discriminant type, while the attribute table lists the four layout
  attributes for "struct, enum" (owner question Q2).
* `check_declared_layouts`, after every instance is made: the refusals of section 3, and `E2170` at
  the field for a `@derive(Hash)` on a packed type whose field is passed by address.
* `offset_of[T](field)` (`std.mem.offset_of`, also as a bare name like `size_of`): checked in the
  type checker (a struct, a visible field) and folded there to the constant from `TypeTable::layout`,
  so it is a constant wherever one goes (a `const`, an array length); a generic body checked with its
  parameters open gets a placeholder, and each instance its own value. `std/src/mem.em` declares it.
* A derived `hash` of a packed value reads each field that may be unaligned from a local copy
  (`packed_part`), since hashing borrows each part.

### L3. `E2170` (`ember_analysis/src/packed.rs`, `ember_mir::unaligned_place`)

* `unaligned_place(types, body, place)`: the place lies inside a `@packed` value through fields and
  fixed-array elements (a dereference, a list's or a view's element, a class field start aligned
  again) and its type is aligned to more than one byte.
* Every reference MIR forms is an `Rvalue::Ref`: a written `ref`/`ref mut`, and the borrows the
  parameter modes make (a `mut` argument, a borrowed argument passed by address under `[BRW-8]`, a
  method's receiver, a loop over the field, a view of a `String` field). One to an unaligned place is
  `E2170`, reported once per source span, naming the field. Run right after lowering, before drop
  elaboration. A borrowed `Copy` argument passed by value takes no reference and is allowed, as is a
  reference to a one-byte-aligned field (`ref h.tag`; owner question Q3).
* A derived operation that would borrow a non-`Copy` aggregate field of a packed type (a derived
  `clone` calling the field's own `clone(self)`) gets `E2170` at the type, naming the field — the
  same rule Rust enforces for `#[derive]` on packed structs.

### L4. C output (`ember_codegen_c`)

* A packed struct or enum is emitted between `#pragma pack(push, 1)` and `#pragma pack(pop)` (MSVC,
  gcc and clang; the pragma also packs a payload enum's anonymous variant structs). An `@align(N)`
  type is `struct EMBER_ALIGNED(N) name`, the macro defined once per translation unit and per
  exported header (`__declspec(align(N))` on MSVC, `__attribute__((aligned(N)))` elsewhere).
* After each such definition a `_Static_assert` of its size, alignment and every field's offset
  against the compiler's own layout: a C compiler that lays it out otherwise stops the build. Exported
  headers carry a `static_assert`/`alignof` twin under `__cplusplus`. A generic template's struct
  (parameters as `void*`) is not checked.
* Unaligned places are read and written by value, never through their address: checked arithmetic
  into one (`ember_ck_add_u32(a, b, &dest)`) goes through an aligned temporary; drop glue copies a
  packed field out before dropping it (`packed_drop_lines`); the generated equality and `Debug`
  functions compare and print aligned copies of a packed owner's fields (they passed `&(v->inner)`).
  Checked with gcc's `-Waddress-of-packed-member` (made to fire by re-emitting each pragma-packed
  struct with `__attribute__((packed))`): no address of a packed member is taken in any test.

### L5. Over-aligned list buffers (`[HEAP-9]`; runtime and `vec_call`)

* Lists allocate at `EMBER_VEC_ALIGN` (16) and the compiler passes only an element size. A list
  whose element is aligned beyond 16 (`@align(32+)`, and SIMD vectors wider than 16 bytes, which
  were mis-aligned before: defect D-L1) now uses `_aligned` twins of every runtime list operation
  that allocates or frees, taking `_Alignof(T)` after the size: `push`, `reserve_more`,
  `reserve_hint`, `insert`, `from_elems` (literals, `clone`, `drain`, `sorted`) and `free`; the stable
  sort's scratch buffer uses `EMBER_VEC_ALIGN_OF(T)`. `Map`, `Set` and `Pool` are built on `Array`
  and follow.
* Speed: the choice is made by the compiler from the element's layout (`vec_call`), so every other
  list keeps the same calls with the same arguments and the same allocator path; the runtime's plain
  functions call the same static bodies as their twins (no extra call). Checked: 192 existing
  programs' emitted C contains no `_aligned` call, no `EMBER_ALIGNED` and no pragma. A C twin of a
  packed or aligned struct is the emitted C itself (the same pragma or attribute), so there is no
  speed gap to chase; the aligned buffer pays what C's `aligned_alloc` pays.
* Every other place already honoured `_Alignof` (`Box`, `Shared`, class objects, arenas, jobs,
  thread packets, SoA columns, `alloc[T]`); a frame list's stack buffer is only used for alignment 8
  or less.

### L6. Hot reload (`[HR-14]`, optional on the job sheet)

* A type's `@packed`/`@align` are part of its reload schema hash (`layout_schema`; types without
  them keep their hash). Changing them is a schema change: refused for a type a raw pointer can reach
  (the `[HR-13b]` refusal), migrated by name into the new layout otherwise. Test:
  `ember_driver/tests/reload.rs`, `a_changed_layout_attribute_is_a_schema_change` (break-tested).
* Left: `[HR-14]`'s row says "a type used across FFI"; a type that crosses only by value in a C-ABI
  signature is not counted as foreign (`pointed_structs` looks only at raw pointers). Recorded in the
  `HR-14` entry.

## 2. Tests

| Rule | Test |
|---|---|
| LAY-2 (stays `unfinished`, section 3) | `tests/conformance/LAY-2/accept_packed_lays_fields_out_with_no_padding.em` |
| LAY-2, HEAP-9 | `tests/conformance/LAY-2/accept_align_raises_the_alignment_of_every_place.em` |
| LAY-2 | `tests/conformance/LAY-2/accept_repr_chooses_the_discriminant_type.em` |
| LAY-2, TYP-11 | `tests/conformance/LAY-2/accept_layout_c_states_the_default.em` |
| LAY-2 | `tests/conformance/LAY-2/accept_offset_of_is_a_compile_time_function.em` |
| LAY-2 (E2170) | `tests/conformance/LAY-2/reject_a_reference_to_an_unaligned_packed_field.em`, `reject_a_derived_hash_that_borrows_a_packed_field.em` |
| LAY-2 (E0104) | `tests/conformance/LAY-2/reject_an_alignment_that_is_not_a_power_of_two_up_to_4096.em`, `reject_layout_attribute_arguments_outside_the_rule.em` |
| LAY-2 (E2010) | `tests/conformance/LAY-2/reject_a_discriminant_that_does_not_fit_its_repr.em` |
| LAY-2, PHIL-12 (E0901) | `tests/conformance/LAY-2/reject_packed_beside_a_stated_alignment.em` |
| LAY-2, ATT-6 (E0900) | `tests/conformance/LAY-2/reject_layouts_not_built_yet.em` |
| HEAP-9 | `tests/conformance/HEAP-9/accept_a_list_of_wide_vectors_is_aligned_to_their_size.em` |
| HR-14, LAY-2 | `compiler/ember_driver/tests/reload.rs` `a_changed_layout_attribute_is_a_schema_change` |

Every run-pass test passes in `debug`, `development` and `release`; each was break-tested where it
guards a mechanism (the aligned twins off: five of seven buffers mis-aligned; the reload schema
without layout: the refusal missing). Error pages: `docs/errors/E2170.md`, `docs/errors/E0901.md`.
`E0901` joins the code registry (`[PHIL-12]`), and `E2170`'s registry row cites `[LAY-2]`.

Rules this job makes done: none newly counted. `LAY-2` keeps an `unfinished` entry (section 3);
`HEAP-9` and `TYP-11` were already counted (HEAP-9 was in fact broken for wide SIMD vectors, D-L1,
now fixed); `HR-14` stays unfinished with one row fewer.

## 3. Not built, and why (`E0900`, recorded in `docs/phase-status.json`)

* **A SIMD vector in a `@packed` type** (directly or inside another value). `[LAY-2]` says the field is
  unaligned and read and written by byte copies. The C backend cannot say that with `#pragma pack`:
  MSVC keeps the alignment of a member whose type is over-aligned (`__m128`, `_Alignas`) under any
  pack (measured: `offsetof` 16, not 1; gcc packs it to 1). Building it needs a byte-array member
  (`unsigned char v[16]`) and every read and write of such a field through `memcpy`: in `place_in`
  for reads, in `Assign`/`CheckedBinaryOp` for writes (with read-modify-write for a write through a
  struct holding a vector), a MIR pass moving call destinations off such places, the packed copies of
  the generated equality and `Debug` functions, aggregate and static initialisers. Waiting on that
  work; not a question.
* **`@align(N)` on an enum without payloads.** `[TYP-12]` makes such an enum its discriminant integer,
  which is how the C backend holds it everywhere (`typedef uint8_t E;`, casts, `==`, `switch`, the
  `Option` niche, wide-integer casts, reload's unit-enum migration): an integer cannot have a size
  that is a multiple of 64. Building it needs a second C representation (a struct around the tag)
  chosen by "unit-only and not over-aligned" at about 50 sites in the backend and reload. Waiting on
  that work; not a question.

## 4. Owner questions (real choices only)

**Q1. `@packed` together with `@align`, on one type or by holding.** (E0901 meanwhile.)
`[LAY-2]`: "`@packed` — no padding; …" and "`@align(N)` — raises the type's alignment to `N` …;
`size_of` becomes a multiple of `N`, and every place and heap buffer holding the type honours it".
On one type the two disagree whenever the fields' sizes do not add up to a multiple of `N`:

```ember
@packed
@align(8)
struct H:          # fields 5 bytes: size 5 (no padding) or 8 (multiple of 8)?
    tag: u8
    length: u32
```

And a packed type holding an `@align` type cannot both leave no padding and have that field's place
honour `N`:

```ember
@align(64)
struct Line:
    a: u8

@packed
struct P:          # `line` at offset 1 (packed) or 64 (honoured)?
    tag: u8
    line: Line
```

Options: (a) packed wins (C with gcc: `line` at 1; MSVC cannot do this, so the C backend would store
the field as bytes), (b) `@align` wins (`line` at 64, the packed type aligned to 64: MSVC's behaviour),
(c) refuse both combinations (Rust refuses them: E0587, E0588). Built meanwhile: both are `E0901`.

**Q2. `@repr` on a struct.** (E0901 meanwhile.) The attribute table (Part V) lists "`@layout(c)`,
`@packed`, `@align(N)`, `@repr(T)` | struct, enum", but `[LAY-2]` defines `@repr` only as "an enum's
discriminant type". `@repr(u8) struct S: a: u8` has no stated meaning. Options: (a) it does not apply
to a struct (`E0104` naming the positions it applies to, and the table row split), (b) give it a
meaning. Built meanwhile: `E0901`.

**Q3. A reference to a one-byte-aligned field of a packed type.** The code table says `E2170` is
"reference to a field of a packed struct"; the rule says "unaligned fields are read and written by
byte copies; taking a reference to one is `E2170`". A `u8` field is never unaligned:

```ember
@packed
struct H:
    tag: u8
    length: u32

t = ref h.tag      # allowed (the rule's reading) or E2170 (the code table's)?
```

Built meanwhile: allowed (the rule's text is the normative one; the code table summarises).

## 5. Defects found

* **D-L1 (fixed)** `[HEAP-9]`: a list of SIMD vectors wider than 16 bytes (`f32x8`, `f64x8`) was
  allocated 16-aligned (the runtime comment said so). Probe: `Array[f32x8]`, push, then the buffer's
  address `% 32` — 16 for some buffers. Fixed by L5; test
  `tests/conformance/HEAP-9/accept_a_list_of_wide_vectors_is_aligned_to_their_size.em`.
* **D-L2 (fixed)** `[PHIL-12]`, `[LAY-2]`: `@layout(anything)` was accepted and ignored
  (`@layout(rust)` compiled). Now `E0104` unless `c`.
* **D-L3 (fixed)** `[LAY-2]`: `@repr(usize)`, `@repr(u128)`, `@repr(i256)` were accepted (any integral
  type); now only the eight.
* **D-L4 (fixed)** `[LAY-2]`: a discriminant outside `@repr`'s type was truncated in C
  (`@repr(u8) enum E: A = 300` gave 44). Now `E2010`.
* **D-L5 (open, outside this job)** `[TYP-12]`: "the default is the smallest signed type that fits",
  but `enum_repr` picks the smallest *unsigned* type for non-negative discriminants, and caps at 32
  bits. Probe (no `@repr`): `enum Small: A = 200` is `uint8_t` (the rule: `i16`); `enum Big: A =
  5000000000` is `uint32_t`, and `Big.A as i64` prints `705032704` — a silent truncation. Fixing it
  changes the C type and size of every unit enum without `@repr`, so it is left for the owner of
  `[TYP-12]`'s work.
