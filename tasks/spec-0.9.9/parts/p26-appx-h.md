---

# Appendix H — Changes from 0.9.8_Hardened_3

## H.1 What changed, by goal

**Memory safety.**
* Only `@sync` classes are shareable, and their fields are immutable after `init`; Safe Ember has no
  data race (`[THR-1]`, `[THR-13]`).
* Assigning a non-`Copy` field through a class handle is a checked write access (`[EXC-16]`).
* No setting removes a safety check: `exclusivity = "unchecked"` and `gpu.validate` are gone, overflow
  and stale-handle checks run in every profile (`[PRF-1]`, `[TYP-8]`, `[HND-1]`, `[GPU-1]`).
* Checked capacity arithmetic and honoured alignment in the runtime (`[HEAP-8]`, `[HEAP-9]`).
* Compile-time heap data can never be grown or freed at run time (`[CT-5]`).

**C-like speed.**
* UB-free C, floating-point flags, injective mangling, inline count fast paths (`[CG-C-1]`, `[CG-C-11]`,
  `[MNG-1]`, `[RT-10]`).
* Grouped overflow checks keep checked arithmetic vectorisable (`[SIMD-7]`).
* A performance gate against equivalent C (`[TST-28]`).

**Python ergonomics.**
* `int` is `i64`, `float` is `f64`; integer `/` is rejected, `//` and `%` are floor (`[TYP-1]`,
  `[TYP-28]`).
* String literals initialise `String`; collection literals and comprehensions; chained comparisons;
  `x is None`; unparenthesised tuples; names assigned in every branch; implicit `Eq`/`Debug`/`Clone`;
  lazy statics; generators and `some` returns; `Result[T]` with `AnyError`; ordered `Map`; `print`
  with several arguments; callable fields; Python-habit fix-its (`[TXT-9]`, `[TYP-38]`, `[GRM-25]`,
  `[EXP-9]`, `[GRM-29]`, `[CTL-10]`, `[STR-5]`, `[STA-3]`, `[CORO-1]`, `[TYP-32]`, `[ERR-9]`,
  `[STD-11]`, `[STD-9]`, `[CLO-11]`, `[DIA-21]`).
* Rust-style friction removed: no `::`, no lifetime syntax ever, `@view` inferred, coherence per
  package, `with_views*` and `@latebound` removed, field-sensitive private methods, arena elision,
  location-sensitive borrow checking (`[GRM-24]`, `[LEX-22]`, `[TYP-34]`, `[TYP-20]`, `[LT-7]`,
  `[BRW-10]`, `[LT-44]`, `[BCK-2]`).

**Coherence.**
* No silent acceptance; one meaning per program; Python spelling means Python meaning; costs are named
  (`[PHIL-12]`–`[PHIL-15]`).
* One language before 1.0 (`[VER-8]`). The document is the language only: compiler architecture, the
  implementation plan and the host-engine plan are no longer in it; C++, hot reload and GPU are
  annexes.

## H.2 Removed constructs

`::`; `'a` lifetime reservation; `with_views`, `with_views2..4`, `@latebound`; `@thread_local`;
`@derive(SoA)` and `columns_mut`; `Callable[…]` in source; `PartialEq`/`PartialOrd`; `unsafe(reason =
…)`; `#! language` selectors other than the current version; the manifest keys `exclusivity`,
`overflow`, `bounds_checks` and `gpu.validate`; job access-set declarations verified only in debug.

## H.3 Rule identifiers no longer defined

Generated: every 0.9.8 identifier that this revision does not define, grouped by family. An identifier
is never reused for another meaning.

| Family | Identifiers | Why |
|---|---|---|
| ARN | ARN-5a, ARN-5b, ARN-5e, ARN-5f, ARN-8a, ARN-9, ARN-12, ARN-13 | sub-rules folded into `[ARN-5]` and `[ARN-8]` |
| AST | AST-1, AST-2 | compiler internals; out of the language document |
| ATT | ATT-5 | folded into the attribute table and `[GRM-20]` |
| BEN | BEN-7 | benchmark plan; replaced by the performance gate `[TST-28]` |
| BLD | BLD-3, BLD-12 | build-system internals; the user-visible parts are §XVII.3 |
| BLD-FFI | BLD-FFI-4a | C++ build integration; see Annex C |
| BUD | BUD-1a, BUD-2a, BUD-4, BUD-4a, BUD-5a, BUD-6, BUD-7, BUD-8 | compile-time budget details; condensed into `[BLD-10]` |
| CAT | CAT-1, CAT-2, CAT-3, CAT-4, CAT-5 | rule categories retired; conformance profiles (`[CONF-*]`) replace them |
| CELL | CELL-5, CELL-6, CELL-6a, CELL-7, CELL-8, CELL-9, CELL-10, CELL-11, CELL-12 | folded into the prelude (`[MOD-5]`); in 0.9.15 `RefCell` and its run-time checks are gone, `Cell` stays (`[CELL-13]`), and the rest is `shares mut`, `thread_shared`, `Mutex` and `Atomic` |
| CLI | CLI-3, CLI-14, CLI-16, CLI-17 | condensed into the command listing of §XVII.1 and `[CLI-19]` |
| CLO-ABI | CLO-ABI-1, CLO-ABI-2 | compiler internals |
| CLS | CLS-8 | in 0.9.15 a class needs no mark to be shared between threads: `thread_shared` says it on the value (§XI) |
| CMP | CMP-1, CMP-2, CMP-3 | compiler internals |
| COMP | COMP-1 | compiler internals |
| CONF | CONF-6 | folded into `[CONF-1]` |
| COR-ABI | COR-ABI-1, COR-ABI-2 | compiler internals |
| CTL | CTL-3a, CTL-3c | folded into `[CTL-3]`; its conformance obligations are `[TST-4a]` |
| CTR | CTR-11 | retired in 0.6.2 |
| CXX | CXX-2, CXX-3, CXX-4, CXX-5, CXX-6, CXX-7 | C++ corpus details; `[CXX-1]` in Annex C |
| DET-IMPL | DET-IMPL-1, DET-IMPL-2 | compiler internals |
| DIA | DIA-17, DIA-19 | condensed into §XVII.6 (shape tables, `[DIA-21]`, `[DIA-24]`) |
| EFF | EFF-6a, EFF-11a, EFF-13, EFF-19a, EFF-19b, EFF-22 | folded into `[EFF-11]`, `[EFF-16]` and `[EFF-19]`; in 0.9.15 written effect clauses are gone: what a function changes is worked out from its body (`[EFF-1]`) |
| EXC | EXC-3a, EXC-4, EXC-5, EXC-6, EXC-7, EXC-8, EXC-9, EXC-10, EXC-11, EXC-12, EXC-13, EXC-14 | performance gate folded into `[TST-28]`; in 0.9.15 the run-time exclusivity checks are gone: who may change or read a place is checked when compiling (§VII) |
| EXP | EXP-7 | replaced by floor semantics, `[TYP-28]` |
| FFI | FFI-11d, FFI-11e, FFI-18, FFI-19, FFI-20, FFI-24a, FFI-32a, FFI-32b, FFI-32c, FFI-32d, FFI-32e, FFI-34, FFI-34a, FFI-35, FFI-37e, FFI-37f, FFI-43a | C++ interop details (Annex C) or folded into the five-axis contract `[FFI-11]` |
| FFI-CB | FFI-CB-1 | folded into `[FFI-21]` |
| FFI-IMPL | FFI-IMPL-1, FFI-IMPL-3 | compiler internals |
| FN | FN-6a, FN-6b | removed with `@latebound` (F-168); callable types elide regions per call (`[LT-7]`) |
| GATE | GATE-1, GATE-2, GATE-3, GATE-4, GATE-5, GATE-6, GATE-7, GATE-8, GATE-8a | implementation plan; out of the language document |
| GEN-COH | GEN-COH-1, GEN-COH-2 | folded into `[TYP-20]` |
| GRM | GRM-9, GRM-14, GRM-22 | folded into Part III (`GRM-14` retired with the access-mode generic kind, F-024) |
| HEAP | HEAP-4, HEAP-5 | in 0.9.15 `Shared[T]` and its checked `get`/`get_mut` are gone: several owners are `shares` and `shares mut` (§VIII), with no run-time access check |
| HIR | HIR-1, HIR-2 | compiler internals |
| HOT | HOT-1, HOT-3, HOT-5, HOT-10 | compiler internals of hot reload |
| HR | HR-12b, HR-21a, HR-22a, HR-23a, HR-26, HR-40 | condensed into Annex B |
| HR-IMPL | HR-IMPL-1, HR-IMPL-3 | compiler internals of hot reload |
| IDE | IDE-1, IDE-2, IDE-5, IDE-7, IDE-10 | language-server design; out of the language document |
| IMP | IMP-1, IMP-2, IMP-3, IMP-7, IMP-8, IMP-9, IMP-10 | implementation plan; out of the language document (`[IMP-11]` is new) |
| JOB | JOB-4 | debug-only access-set verification removed (a check in one profile is not a guarantee, `[PHIL-13]`) |
| LAY | LAY-1 | replaced by `[LAY-2]` |
| LEX | LEX-14a, LEX-15a, LEX-15b | folded into `[LEX-14]` and `[LEX-15]` |
| LT | LT-1b, LT-2, LT-2a, LT-4a, LT-4b, LT-8, LT-8a, LT-9, LT-10, LT-11, LT-11a, LT-12, LT-13, LT-15, LT-19, LT-28, LT-29, LT-31, LT-31a, LT-32, LT-33, LT-37, LT-40, LT-41 | `with_views*` removed (F-168); multi-region rules condensed into §VII.4 |
| MAN | MAN-4, MAN-5, MAN-6 | folded into `[MAN-8]` |
| MIR | MIR-1, MIR-2, MIR-3, MIR-4, MIR-5, MIR-6 | compiler internals |
| MIR-REG | MIR-REG-1 | compiler internals |
| MNG | MNG-5 | folded into `[MNG-1]` |
| MOD | MOD-6, MOD-6a | language-version selectors removed (`[VER-8]`) |
| MONO | MONO-4 | folded into `[MONO-2]` and `[MONO-8]` |
| OPT | OPT-2a | conformance obligation folded into `[TST-4a]` |
| OQ8 | OQ8-1, OQ8-4, OQ8-7 | open questions closed by this revision |
| PRV | PRV-8 | retired with the prover in 0.6.2 |
| RNG | RNG-5a, RNG-10c | folded into `[RNG-5]` and `[RNG-10]` |
| RT | RT-9 | folded into `[RT-1]` and `[HND-3]` |
| RV | RV-1, RV-2, RV-3, RV-4 | host-engine integration plan; out of the language document |
| SOA | SOA-5 | `columns_mut` retired: columns are disjoint places (`[SOA-2]`) |
| SPN | SPN-6, SPN-7, SPN-9, SPN-10 | folded into `[SPN-4]` and Part XV |
| STD | STD-7a | folded into `[STD-7]` |
| STD-IMPL | STD-IMPL-1, STD-IMPL-2 | compiler internals |
| TST | TST-4c, TST-12, TST-13, TST-14, TST-15, TST-16, TST-17, TST-18, TST-19, TST-20, TST-21, TST-22, TST-23, TST-24, TST-25, TST-26 | condensed into §XVII.5 |
| UNS | UNS-9, UNS-9a, UNS-10b | the reason category moved into the `# SAFETY(…):` note (`[LEX-23]`, `[UNS-8]`) |
| VER | VER-7 | folded into `[VER-2]` |
| VERIFY | VERIFY-1, VERIFY-2, VERIFY-3 | compiler internals |
| WK | WK-10 | folded into `[WK-4]`, `[WK-6]` and `[WK-15]` |

## H.4 Changes from 0.9.9_Hardened_1 to 0.9.9_Hardened_2

Hardened_2 applies the change list drawn from three passes over Hardened_1 (memory safety; consistency
and speed; ergonomics). Before 1.0 a hardening may change the language when this log lists each such
change (`[VER-9]`). These change which programs are accepted, or what they do:

| Change | Effect on existing programs | Rules |
|---|---|---|
| a thread or unscoped job carries only the static region | a spawn capturing a view of a local is rejected; use `thread.scope()` | `[THR-10]`, `[JOB-2]` |
| `Shared[T]` is one-thread; `SyncShared[T]` crosses threads; `get`/`get_mut` are checked accesses | sending a `Shared` is rejected; overlapping `get_mut` panics | `[HEAP-4]`, `[HEAP-5]`, `[HEAP-10]`, `[THR-8]`, `[THR-9]` |
| two-phase construction | a derived `init` that calls `super.init` before assigning its own fields is rejected (`E2102`) | `[CLS-2]`, `[CLS-4]`, `[CLS-10]`, `[CLS-11]` |
| retained and once C callbacks own static captures; callback captures are `Send` unless the contract names a thread | such callbacks capturing borrows or non-`Send` handles are rejected | `[FFI-21]`, `[FFI-22]` |
| foreign functions are safe to call only when asserted: `safe fn`, or an overlay entry | calls to unlisted header functions and to extern-block functions not marked `safe` need `unsafe` | `[FFI-1]`, `[FFI-2]`, `[FFI-10]` |
| `yield` while a `@must_drop` value is live | rejected (`E2231`) | `[CORO-13]` |
| `char32_t` is `u32` | foreign signatures using it change type | `[FFI-8]` |
| SIMD memory operations are bounds-checked | out-of-range `load`/`store`/`gather`/`scatter` panic | `[SIMD-9]` |
| calling a callable field is a write access; owned callable values are called from a mutable place | a callback that replaces itself while running panics | §VIII.3, `[CLO-2]` |
| accesses held across a call into C++ stay checked | a conflicting re-entrant override panics | `[FFI-39d]`, `[EXC-7]` |
| `run_parallel` checks systems given as values | a conflict panics before any system starts | `[ECS-4]` |
| phantom parameters count for `Send`/`Sync`; `comptime.read_file` stays inside the package | fewer types are `Send`; reads outside the package are `E6010` | `[TYP-35]`, `[CT-2]` |
| exclusivity is tracked per field | reading one field while writing another no longer panics | `[EXC-19]`, `[EXC-1]`, `[EXC-2]`, `[EXC-15]` |
| a `Map` iterates its keys | `for k, v in m:` becomes `for k, v in m.items():` (`ember fmt --migrate`) | `[CTL-1]`, `[STD-16]` |
| the entry file may hold top-level statements | scripts need no `main` | `[GRM-2]`, `[FN-8]` |
| `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` in the prelude | accepted with Python's meaning; `len` of a string is `E2073` | `[STD-26]`, `[MOD-5]` |
| generator expressions | accepted | `[GRM-38]` |
| printing falls back from `Display` to `Debug` | more values print; none prints differently | `[STD-9]`, `[LEX-19]` |
| `T` converts to `Option[T]`; `None` then `T` infers `Option[T]` | accepted | `[TYP-5]`, `[TYP-23]` |
| a `Result[void, E]` function returns `Ok(())` at its end | accepted | `[FN-10]` |
| a lambda passed to a consumed callable parameter captures by move | accepted without `owned fn` | `[CLO-15]` |
| standard modules importable without `std.` | accepted; a package module of the same name wins (`W1003`) | `[MOD-3]` |
| iterator adapters on any `Iterable` | accepted | `[STD-19]` |
| float `Display` is the shortest round-trip text | printed floats may change | `[STD-20]` |
| `W2015` only for more digits than the type keeps | fewer warnings | `[LEX-17a]` |
| contract verdicts assume every listed proof applied; grouping required in vectorisable loops | verdicts agree between implementations | `[EFF-15]`, `[SIMD-7]`, `[SIMD-5]` |

Generated: every rule whose text differs between the two hardenings.

| Rule | H1 → H2 |
|---|---|
| `[BRW-3]` | changed |
| `[CG-C-4]` | changed |
| `[CLI-4]` | changed |
| `[CLI-20]` | changed |
| `[CLO-2]` | changed |
| `[CLO-4]` | changed |
| `[CLO-7]` | changed |
| `[CLO-15]` | added |
| `[CLS-2]` | changed |
| `[CLS-4]` | changed |
| `[CLS-7]` | changed |
| `[CLS-10]` | changed |
| `[CLS-11]` | added |
| `[CORO-12]` | changed |
| `[CORO-13]` | added |
| `[CT-2]` | changed |
| `[CTL-1]` | changed |
| `[CTL-3b]` | changed |
| `[DIA-21]` | changed |
| `[ECS-3]` | changed |
| `[ECS-4]` | changed |
| `[EFF-15]` | changed |
| `[ERR-13]` | changed |
| `[EXC-1]` | changed |
| `[EXC-2]` | changed |
| `[EXC-6]` | changed |
| `[EXC-7]` | changed |
| `[EXC-15]` | changed |
| `[EXC-17]` | added |
| `[EXC-18]` | added |
| `[EXC-19]` | added |
| `[FFI-1]` | changed |
| `[FFI-2]` | changed |
| `[FFI-8]` | changed |
| `[FFI-10]` | changed |
| `[FFI-21]` | changed |
| `[FFI-22]` | changed |
| `[FFI-39d]` | changed |
| `[FN-8]` | changed |
| `[FN-10]` | added |
| `[GRM-2]` | changed |
| `[GRM-35]` | changed |
| `[GRM-38]` | added |
| `[HASH-2]` | changed |
| `[HASH-3]` | changed |
| `[HEAP-4]` | changed |
| `[HEAP-5]` | changed |
| `[HEAP-7]` | changed |
| `[HEAP-10]` | added |
| `[HND-2]` | changed |
| `[JOB-2]` | changed |
| `[LEX-15]` | changed |
| `[LEX-17a]` | changed |
| `[LEX-19]` | changed |
| `[LNT-6]` | changed |
| `[MOD-3]` | changed |
| `[MOD-5]` | changed |
| `[OBJ-1]` | changed |
| `[OWN-6]` | changed |
| `[PAR-3]` | changed |
| `[PHIL-13]` | changed |
| `[PRF-1]` | changed |
| `[RC-3]` | changed |
| `[RC-4]` | changed |
| `[RNG-4]` | changed |
| `[RT-10]` | changed |
| `[RT-12]` | added |
| `[SIMD-3]` | changed |
| `[SIMD-5]` | changed |
| `[SIMD-7]` | changed |
| `[SIMD-9]` | added |
| `[SOA-6]` | changed |
| `[STD-9]` | changed |
| `[STD-13]` | changed |
| `[STD-15]` | changed |
| `[STD-16]` | changed |
| `[STD-19]` | changed |
| `[STD-20]` | changed |
| `[STD-26]` | added |
| `[THR-1]` | changed |
| `[THR-6]` | changed |
| `[THR-8]` | changed |
| `[THR-9]` | changed |
| `[THR-10]` | changed |
| `[THR-13]` | changed |
| `[THR-16]` | added |
| `[TIER-1]` | changed |
| `[TST-6]` | changed |
| `[TYP-5]` | changed |
| `[TYP-23]` | changed |
| `[TYP-35]` | changed |
| `[TYP-38]` | changed |
| `[VER-9]` | changed |
| `[WK-11]` | changed |
| `[WK-12]` | changed |

## H.5 Changes from 0.9.9_Hardened_2

Each ambiguity found while implementing 0.9.9 is an owner decision request (`docs/OWNER-QUEUE.md`),
ruled under the owner's delegation and recorded here. Hardened_3 carries ODR-021 and ODR-022;
Hardened_4 adds ODR-023; Hardened_5 adds ODR-024; Hardened_6 adds ODR-025; Hardened_7 adds ODR-026;
Hardened_8 adds ODR-027; Hardened_9 adds ODR-028;
Hardened_10 adds ODR-029; Hardened_11 adds ODR-030; Hardened_12 adds ODR-031; Hardened_13 adds ODR-032 to ODR-036;
Hardened_14 adds ODR-037; Hardened_15 adds ODR-038; Hardened_16 adds ODR-039; Hardened_17 adds ODR-040;
Hardened_18 adds ODR-041; Hardened_19 adds ODR-042; Hardened_20 adds ODR-043 to ODR-045;
Hardened_21 adds ODR-046; Hardened_22 adds ODR-047; Hardened_23 adds ODR-048; Hardened_24 adds
ODR-049 to ODR-064, the owner's simplification pass (`docs/proposals/Ember_Simplification_Pass_Revised.md`);
Hardened_25 adds ODR-065, the review of SP-010's access question; Hardened_26 adds ODR-066 to
ODR-068, the owner's Part II adoptions SP-007, SP-016 and SP-031; Hardened_27 adds ODR-069, SP-013;
Hardened_28 adds ODR-070, the nesting limit (D-331); Hardened_29 adds ODR-071 and ODR-072, found
closing the last open defects (D-355, D-202); Hardened_30 adds ODR-073, the hand-declared C pointer
contract syntax and safe surface; Hardened_31 adds ODR-074, the ABI witness for a hand-declared
counted pointer; Hardened_32 adds ODR-075 and ODR-076, the safe reference-to-pointer spellings and
the exact pointer type of `null[P]()`; Hardened_33 adds ODR-077, the input length source for a
borrowed counted C pointer result; Hardened_34 adds ODR-078, the exclusive counted result's
mutable view and C pointer; Hardened_35 adds ODR-079, the nullable counted input's distinction
between absent and present-empty views; Hardened_36 adds ODR-080, nullable borrowed C strings and
their null-pointer niche; Hardened_37 adds ODR-081, the fallible owned C-string conversion and
explicit borrowed view; Hardened_38 adds ODR-082, the C qualifier asserted by an immutable
foreign static; Hardened_39 adds ODR-083, the initialization anchor for static export creator
contracts; Hardened_40 adds ODR-084, module-attribute syntax and lexical integer overflow policy;
Hardened_41 adds ODR-085, what the dynamic exclusivity checks cost and the packed access words;
Hardened_42 adds ODR-086 and ODR-087, running totals checked block by block and the LLVM backend;
Hardened_43 adds ODR-088, integer division no vector instruction set does keeps a loop out of
vectorisable form; Hardened_44 adds ODR-089, the iterator adapters' counts and ownership;
Hardened_45 adds ODR-090, where a float mode holds; Hardened_46 adds ODR-091, which iterators
run backwards; Hardened_47 adds ODR-092, when two implementations overlap; Hardened_48 adds
ODR-093, what a range type implements; Hardened_49 adds ODR-096, how a call chooses between two
instances of one generic interface; Hardened_50 adds ODR-095, which operators a range value has,
and ODR-097, where an implementation may be written; Hardened_51 adds ODR-094, the owner's
ruling on `collect`'s target, `peekable`'s methods, and how `Box` and `Cell` show; Hardened_52 adds
ODR-098, the byte offsets, UTF-8 checks and capacity contracts of `String` mutation methods.
0.9.10_Hardened_3 adds ODR-099, a pointer into a parameter C passes as a copy (§H.8);
0.9.10_Hardened_4 adds ODR-100, what a lambda written at an adapter captures (§H.9);
0.9.10_Hardened_5 adds ODR-101, a `main` of another form (§H.10);
0.9.10_Hardened_6 adds ODR-102, owned callable values (§H.11);
0.9.10_Hardened_7 adds ODR-103, a lambda outliving its captures is `E3026` (§H.12);
0.9.10_Hardened_8 adds ODR-104, a constructor's `return` and its receiver (§H.13);
0.9.10_Hardened_9 adds ODR-105, `E2104` for a `let` field assigned outside `init` (§H.14);
0.9.10_Hardened_10 adds ODR-106 to ODR-112, the owner's `is` and `?.` and the agents' open
questions (§H.15); 0.9.10_Hardened_11 adds ODR-113 to ODR-115, the owner's rulings on what printing,
a use after a move and an `unsafe fn` value are (§H.16); 0.9.10_Hardened_12 adds ODR-116, a
`MutSpan` where a `Span` is expected (§H.17); 0.9.10_Hardened_13 adds ODR-117 to ODR-126, the
owner's answers to the 85% push's questions and the profiles' new names (§H.18); 0.9.10_Hardened_14
adds ODR-127 to ODR-131, the owner's answers to helper C's questions (§H.19); 0.9.10_Hardened_15 adds
ODR-132, the type of `Retained.ptr()` (§H.20).

| ODR | Ruling | Rules |
|---|---|---|
| ODR-021 | float `//` and `%` are Python's: the exact floor modulo rounded once, and the floor quotient consistent with it | `[TYP-29]` |
| ODR-022 | an untyped literal is never left open: `total = 0` declares an `int` whatever the later uses | `[TYP-23]` |
| ODR-023 | a function returning a value that can reach the end of its body is `E2182` (Hardened_4) | `[FN-10]`, §XVII.9 |
| ODR-024 | a borrowed or `mut` parameter whose type is not `Copy` is a source parameter a returned view may borrow, and a borrowed parameter is passed by address except a view or a `Copy` value holding no `Cell` (Hardened_5) | `[LT-1]`, `[LT-1a]`, `[LT-1b]`, `[LT-7]`, `[LT-44]`, `[FN-1]`, `[FN-3]`, `[FN-6]`, `[BRW-8]`, `[CORO-6]`, §VIII.3, §XVII.6 B7, §XVII.9, Appendix F |
| ODR-025 | `[ERR-4]`'s methods that take a function are eager, move the payload in (`filter` borrows it) and take `once fn`; an unannotated lambda parameter takes `owned` from the expected callable type, never `mut` (Hardened_6) | `[ERR-4]`, `[CLO-7]`, `[TYP-23]` |
| ODR-026 | a type that declares `drop` is not implicitly `Clone`; `@derive(Clone)` or a written `clone` gives it one (Hardened_7) | `[STR-5]` |
| ODR-027 | a range is a value: the prelude's range types are structs with public bounds, `Copy` when the bound is, and a `for` over one counts over a copy of its bounds, leaving it unchanged; `a..` overflows past its type's maximum (Hardened_8; since 0.9.10 it gives the maximum first, D-526) | `[CTL-3]`, `[STD-8]`, `[STD-26]` |
| ODR-028 | a method named like an inherited one replaces it: without `override` over a virtual method it is `E2111`, over a non-virtual one `E2110`; an `override` is itself virtual (Hardened_9) | `[CLS-4]` |
| ODR-029 | `parse[T]()` reads the whole text strictly (no white space; an optional sign and digits for integers, Rust's float grammar with `inf`/`nan`, `true`/`false`, one character), and `ParseError` is `Empty`, `Invalid` or `Overflow` (Hardened_10) | `[TXT-10]` |
| ODR-030 | `extend` is a contextual keyword, a keyword only at the start of an item before the type it extends, so `Array` can have its `extend` method (Hardened_11) | `[LEX-15]` |
| ODR-031 | `Array`'s `sort_by(cmp: fn(T, T) -> Ordering)` and `sort_by_key[K: Ord](f: fn(T) -> K)` are stable, and `f` is called once per element; `windows(n)` yields shared, overlapping views one step apart, none when `n > len`, and panics on `0`; `drain(r) -> Array[T]` takes any integer range and panics outside `0..=len` (Hardened_12) | `[STD-15]` |
| ODR-032 | `Map[K, V, H, A]` and `Set[T, H, A]`: the hasher before the allocator; the full method lists with signatures; `AsKey[K]: Hash` has `is_key` and `to_key`, and every `K: Eq + Hash` is `AsKey[K]`; owned iteration of a `Map` yields its keys (Hardened_13) | `[STD-11]`, `[STD-12]`, `[STD-16]`, `[ALC-1]`, `[CTL-1]` |
| ODR-033 | the `Hasher` methods, and users may implement it; `DefaultHasher`'s values are fixed for one version on every target; making a `RandomState` is `Nondet`; a panicking `hash` or `eq` aborts; `Map`'s constant time is amortised (Hardened_13) | `[HASH-1]`, `[HASH-2]`, `[HASH-3]`, `[DET-2]`, `[STD-11]` |
| ODR-034 | an empty `Map` prints `{}` and an empty `Set` `set()`; strings `Debug` as Python's `repr`; `Map`/`Set` `==` compares as sets; a repeated literal key keeps its first position and last value; a list literal is never a `Set`; `sorted(m)` sorts the keys (Hardened_13) | `[TYP-39]`, `[STD-16]`, `[TYP-38]` |
| ODR-035 | `union` is contextual: reserved only where an item would begin `union` and a name, so `Set` can have its `union` method (Hardened_13) | `[LEX-15]` |
| ODR-036 | a `Map`'s keys and values and a `Set`'s elements are not views (`E3063`); text in a `{…}` literal with no context is `String` (Hardened_13) | `[STD-11]`, `[TYP-38]` |
| ODR-037 | `std.math.Float`, implemented by `f32` and `f64` only (`E2042`), is the bound of the generic float functions and provides their operators, typed literals and methods; `PI`, `TAU` and `E` are untyped constants (Hardened_14) | `[STD-21]`, `[STD-27]`, V.7 |
| ODR-038 | ruled by the owner: `std.math`'s functions take any number type through `std.math.Number`, answering in `T.Real` (`f64` for an integer, `f32` for an `f32`); `T.Name` names a type parameter's associated type (Hardened_15) | `[IFC-4]`, `[STD-21]`, `[STD-27]` |
| ODR-039 | `[STD-20]`'s integer methods: `checked_`, `wrapping_`, `saturating_` and `overflowing_` forms of `add`, `sub`, `mul`, `floordiv`, `rem`, `pow` and `neg` (the shifts too, but not saturating), built into the integer types; bit counts are `int`s; a float's `MIN` is `-MAX` (Hardened_16) | `[STD-20]` |
| ODR-040 | the operator interfaces' methods are `add`, …, `floordiv`, `bitand`, …, `neg` and `not` (a method name after `fn` and `.`), each `…Assign` form's with `_assign`; every number type implements the interface of each operator it has, in `std.core`; a type has an operator only through its interface; one `type Output` serves every interface of an `extend` block; a binding is written only in a bound (Hardened_17) | `[TYP-21]`, `[IFC-4]`, `[LEX-15]` |
| ODR-041 | `f16` is a `Number` whose `Real` is `f32`; every float type, `f16` included, has `INF`, `NAN`, `EPSILON`, `MAX` and `MIN` (Hardened_18) | `[STD-20]`, `[STD-21]`, `[STD-27]` |
| ODR-042 | an `extend` parameter that only the implemented interfaces name makes a blanket implementation; a bound brings its parents; indexing is only through `Index`, `IndexMut` and `IndexSet`, which `Array` and the views implement through their built-in indexing and `Map` for every `Q: AsKey[K]` (Hardened_19) | `[GRM-34]`, `[IFC-3]`, `[TYP-21]`, `[STD-17]` |
| ODR-043 | `std.math` has the module table's vectors, matrices, `Quat`, `Transform` and shapes, with public components, the operators and the methods graphics libraries agree on: matrices column-major, projections right-handed with depth in `[0, 1]`, a ray's intersections the least `t ≥ 0`; `KahanSum` is an `f64` accumulator with Neumaier's correction (Hardened_20) | `[STD-21]`, `[STD-28]`, `[STD-5]`, `[STD-3]` |
| ODR-044 | the `FP_CONTRACT` pragma is emitted where the compiler implements it: clang's spelling, MSVC's own; gcc, which warns about it, has `-ffp-contract=off` alone (Hardened_20) | `[CG-C-11]`, `[CG-C-1]` |
| ODR-045 | a type's `const` is named `T.NAME` (`Self.NAME` inside it) and is private unless `pub`; constants name one another in any order, a cycle is `E6001` and a panic while evaluating one `E6004` (Hardened_20) | `[CT-1]`, `[CT-7]`, `[MOD-2]` |
| ODR-046 | `std.math.det`'s functions take `f32` or `f64` and answer in it, an `f32` result being the `f64` one rounded once; each within one unit in the last place (`atan2` 1.3); no integers (Hardened_21) | `[DET-4]` |
| ODR-047 | `NonZero[T]` is `std.core`'s, read with `get()`, its field and constructor private; `T` is an integer type, through a private `Integer`; `x // d` and `x % d` take a `NonZero` of `x`'s type and answer in it (Hardened_22) | `[STD-4]`, `[TYP-13]` |
| ODR-048 | an instance of a generic passes a parameter its result may point into as the declaration does (`Holder[str]`'s receiver by address, as `Holder[T]`'s); a parameter an instance's type makes a view is a source of that instance (Hardened_23) | `[BRW-8]`, `[LT-1]` |
| ODR-049 | an associated type's default fills in an implementation that does not state it, meeting its bounds; it is not an equality a bound or a default method body may assume; a cycle of defaults is `E2043` — SP-003 (Hardened_24) | `[IFC-4]` |
| ODR-050 | `Result[T, E]` bounds neither parameter; each operation says what it needs of `E`; `main`'s error prints its chain when it has one — SP-009 (Hardened_24) | `[ERR-1]`, `[ERR-10]`, `[ERR-12]` |
| ODR-051 | a `Copy` value's copy is its bits, retaining each counted handle in it; a struct holding handles may be `Copy`; an `owned` handle argument is the caller's moved, or a retained copy — SP-010 (Hardened_24) | `[PHIL-3]`, `[OWN-7]`, `[STR-3]`, `[FN-9]` |
| ODR-052 | `RwLock[T]` and `SyncShared[T]` are `Sync` for a `Send + Sync` `T`, `Mutex[T]` for a `Send` one; a `@sync` class's fields are `Send` and `Sync`; the marker table agrees with `[THR-8]` — SP-011 (Hardened_24) | `[THR-8]`, `[THR-9]`, `[THR-1]`, `[HEAP-10]` |
| ODR-053 | a view is what carries a region; a guard is a view that needs drop, and dropping it ends its access — SP-012 (Hardened_24) | `[DRP-6]`, `[TYP-34]` |
| ODR-054 | `@must_drop` is structural: a type owning such a value is one too — SP-015 (Hardened_24) | `[THR-6]` |
| ODR-055 | a float's comparison operators are IEEE in generic code too; `cmp`, sorting, `min` and `max` are total; an algorithm keeps to one relation — SP-018 (Hardened_24) | `[TYP-37]`, `[DRV-1]` |
| ODR-056 | compile time runs `std.math.det`'s functions when the source names them, and never puts one in place of the platform's — SP-019 (Hardened_24) | `[CT-4]` |
| ODR-057 | an operation's effects include what it runs: callbacks, copies, releases, drops, argument evaluation; a generator's by phase; `@nopanic(explicit)` without checks says no modelled panic — SP-020 (Hardened_24) | `[EFF-1]`, `[EFF-16]`, `[STD-18]`, `[STD-19]`, `[CELL-2]` |
| ODR-058 | a body edit re-checks the callers that use a fact of it that changed: effects, inline bodies, borrow summaries — SP-023 (Hardened_24) | `[BLD-2]`, `[BLD-8]` |
| ODR-059 | `ember fmt` changes layout only; rewriting source is `--migrate`'s — SP-024 (Hardened_24) | `[FMT-1]` |
| ODR-060 | the formatter writes LF with no manifest switch; `Weak.upgrade` never raises a count from zero; the attribute table lists every attribute; `@assume_noalloc` is an expression inside `unsafe`; `format_to` is variadic — SP-026 (Hardened_24) | `[LEX-2]`, `[RT-10]`, `[EFF-7]`, `[TYP-26]`, `[ATT-1]` |
| ODR-061 | a build's acceptance depends on its semantic inputs; build policy and budgets stop a build as themselves, never as a language error — SP-027 (Hardened_24) | `[PHIL-13]` |
| ODR-062 | the parallel and vectorisation analyses read through calls by the inline header's specified expansion, never by the C compiler's inlining — SP-028 (Hardened_24) | `[PAR-2b]`, `[SIMD-5]` |
| ODR-063 | an object is deinitialised when its last owner ends as the source says, never earlier; `L3019` is retired — SP-014 (Hardened_24) | `[RC-3]` |
| ODR-064 | `alloc_array` initialises with `Default`, `alloc_zeroed` fills with zeros for a `Zeroable` `T`; a struct is `Zeroable` by `@derive(Zeroable)` — SP-017 (Hardened_24) | `[ARN-3]`, `[ARN-11]` |
| ODR-065 | a borrow through a handle stored in an object, or a call made on one, goes through a retained copy that lasts to the end of the statement; kept longer it is `E3060`; `mem.drop` moves a handle — review of SP-010 (Hardened_25) | `[RC-5]`, `[EXC-17]`, `[FN-9]`, `[OWN-6]` |
| ODR-066 | a const argument is any compile-time expression; two are equal by value when known, else by a stated integer normal form (`+`, `-`, `*`; other operations opaque), and not shown equal is a mismatch that says so — SP-007 (Hardened_26) | `[TYP-41]` |
| ODR-067 | `AsKey[K]` only matches; `ToKey[K]: AsKey[K]` makes the key, and `m[q] = v` needs it; every `K: Eq + Hash` is `AsKey[K]`, and `ToKey[K]` when also `Clone` — SP-016 (Hardened_26) | `[STD-12]`, `[STD-17]` |
| ODR-068 | `get_pair_mut(i, j)` on an `Array` or `MutSpan` gives two mutable references, or `None` for an index out of range or equal indices, after checking — SP-031 (Hardened_26) | `[BRW-5]`, `[STD-15]`, `[SPN-5]` |
| ODR-069 | one storage rule for every owning container: a `Map`, `Set` or `Array` may hold `static` views, checked at each store wherever it happens (through a `mut` parameter or reference, or in a callee, whose callers answer for its parameters); an instance ties a result to a parameter its type names — SP-013 (Hardened_27) | `[TYP-15]`, `[STD-11]`, `[TYP-38]`, `[LT-1]` |
| ODR-070 | every compiler accepts nesting 256 levels deep and states its own limit (this one 1,024); passing it is `E0112`, never a crash (Hardened_28) | `[GRM-39]` |
| ODR-071 | the type `()` is `void`; a tuple type has two or more elements (Hardened_29) | `[TYP-27]` |
| ODR-072 | in a class method `self` is the object the method was called on for the whole call; assigning to it is `E2103`, and re-pointing a caller's handle is a `mut` parameter's (Hardened_29) | `[CLS-7]`, `[FN-9]`, `[EXC-15]` |
| ODR-073 | hand-declared pointer facts use `@ffi(param(name, words…), result(words…))`; a `safe fn` exposes the mapped safe Ember type, never a raw pointer (Hardened_30) | `[FFI-10]`, `[FFI-11]` |
| ODR-074 | `count(n)` on a hand-declared safe C function writes `n` at its C ABI position as an integer witness, hides it from the callable signature, and supplies it from a checked span length (Hardened_31) | `[FFI-10]`, `[FFI-11]` |
| ODR-075 | `ref_to_ptr(r)` remains a safe reference-to-raw-pointer conversion, and `r as *T` is safe when it preserves the pointee type and does not strengthen mutability (Hardened_32) | §IV.5, `[TYP-7]` |
| ODR-076 | `null[P]()` takes a complete raw-pointer type and returns that exact pointer type, including its mutability (Hardened_32) | §IV.5 |
| ODR-077 | a safe hand-declared borrowed `count(n)` result requires an input span sharing `n`; its length supplies the hidden C count and result length (Hardened_33) | `[FFI-10]`, `[FFI-11]` |
| ODR-078 | an `exclusive` borrowed `count(n)` result is `MutSpan[T]` (nullable as `Option[MutSpan[T]]`) through mutable C `T*`, using ODR-077's input witness length (Hardened_34) | `[FFI-10]`, `[FFI-11]` |
| ODR-079 | a nullable hand-declared borrowed `count(n)` input sends `None` as null pointer and zero, but preserves `Some(empty)` with a non-null aligned static sentinel and zero (Hardened_35) | `[FFI-10]`, `[FFI-11]` |
| ODR-080 | `Option[cstr]` has a null-pointer niche; nullable borrowed NUL-terminated C inputs and results use it without changing their lifetime contract (Hardened_36) | `[TYP-13]`, `[FFI-11]`, `[FFI-15]` |
| ODR-081 | `str.to_cstring()` returns `Result[CString, NulError]`, checking for interior NUL before allocation; `.as_cstr()` is an explicit borrow of its owner (Hardened_37) | `[TXT-5]`, `[FFI-15]` |
| ODR-082 | `@ffi(immutable)` asserts a `const` C object for a hand-declared foreign static, with a matching C declaration and safe `Sync` read (Hardened_38) | `[FFI-8]` |
| ODR-083 | the `creator` thread contract of a static free-function export is anchored to successful module initialization; idempotent init does not transfer it and reinitialization after shutdown establishes a new creator (Hardened_39) | `[FFI-33]`, `[FFI-33c]`, `[GRM-37]` |
| ODR-084 | `#! module name(args)` attaches existing module attributes; overflow is lexical to declarations, defaults, and comptime expressions, while integer methods retain fixed contracts (Hardened_40) | `[GRM-37]`, `[ATT-4]`, `[TYP-8]`, `[TYP-28]`, `[FN-5]`, `[CT-4]`, `[STD-20]` |
| ODR-085 | an access that is not held is checked but not counted; a `mut self` call to a quiet function is only checked; a check that nothing held anywhere could fail may be removed; each class's access words stand side by side before its fields, a zero word after an odd count (Hardened_41) | `[EXC-3]`, `[EXC-19]`, `[OBJ-1]`, `[COST-3]` |
| ODR-086 | a running total is not grouped; it is vectorised under checked arithmetic when its widths prove it safe or, for a signed total of at least 16 bits, when a run-time test over each block of 64 iterations does; a block that fails the test runs checked, and so does the rest of the loop (Hardened_42) | `[SIMD-5]`, `[SIMD-7]` |
| ODR-087 | LLVM is restored as the reference implementation's second backend over the same MIR, begun once the C backend passes the conformance suite and the default once it also passes the performance suite; `--backend c|llvm` (Hardened_42) | `[CG-LL-1]`, `[CG-LL-2]`, `[CG-LL-3]`, `[SIMD-1]`, `[SIMD-3]`, `[BLD-6]` |
| ODR-088 | a loop that divides integers by a divisor that is not a constant, or integers of 64 bits or more by a constant that is not a power of two, is not in vectorisable form: no vector instruction set does either, so its overflow checks stay one per operation (Hardened_43) | `[SIMD-5]`, `[SIMD-7]` |
| ODR-089 | an adapter or consumer takes the iterator it wraps; `take`, `skip`, `nth` and `step_by` count in `int`, and a negative count or index, or a step of zero or less, panics when the adapter is made or the consumer called; `enumerate(start=0)` as `[STD-26]` (Hardened_44) | `[STD-19]`, `[STD-14]`, `[STD-26]` |
| ODR-090 | a float mode is lexical, as the overflow policy is: a lambda takes its function's; a parameter default keeps its declaration's and a field default is strict wherever evaluated; inside `@fastmath` no range fact about a float is derived, and integer facts are unaffected; neither mode relaxes range values, `as`, the float classifications, or makes a NaN undefined behaviour (Hardened_45) | `[TYP-9]`, `[TYP-9b]`, `[RNG-4]`, `[CG-C-11]` |
| ODR-091 | an iterator runs backwards when it implements `DoubleEndedIterator` (`next_back`), whose method `rev` is; `ExactSizeIterator` gives `len`; the ranges with an end and the views' element iterators are both, and each adapter over them where it can tell its last item (Hardened_46) | `[STD-19]`, `[CTL-3b]`, `[MOD-5]` |
| ODR-092 | two implementations overlap where one type could be both, each one's type parameters standing for any type, found where they are declared; a bound separates two only where the type it bounds is written out in full and does not meet it (Hardened_47) | `[TYP-19]`, `[TYP-20]` |
| ODR-093 | a range type is `Clone`, `Hash`, `Display` and `Debug` when its representation is, its `Eq` and `Ord` are its representation's, and a range value shows as its representation's value; it is never `Default` (Hardened_48) | `[RNG-8]`, `[RNG-5a1]`, `[TYP-39]` |
| ODR-096 | of two instances of one generic interface a call's or an operator's arguments fit, one whose parameter types are the arguments' own (an untyped literal as its default type) is chosen before one reached through a coercion, for every type; two that fit equally are `E2070`, and an operator chooses as its method does (Hardened_49) | `[TYP-24]`, `[RNG-5a2]`, `[TYP-21]` |
| ODR-095 | a range type has every operator its representation implements over itself, `**` and the bitwise operators included, each generated as `[RNG-5a1]`'s are with the representation as its `Output` (Hardened_50) | `[RNG-5]`, `[RNG-5a1]` |
| ODR-097 | an implementation may also be written in the package that declares one of its interface's arguments, where, reading its type and then those arguments, the first type the package declares comes before any of the implementation's type parameters standing alone; anywhere else it is `E2121` (Hardened_50) | `[TYP-20]` |
| ODR-094 | the owner's: `collect[C]()` builds any `C` implementing `std.core.FromIterator[Item]` (`Array`, `Set`, `Map` from pairs, `String` from `char`s and `String`s, and a program's types), `C` written or the type the call is expected to give; `peekable` gives `peek`, `peek_mut`, `next_if` and `next_if_eq`; `Option`'s `as_ref` and `as_mut` reference the payload where it is; `Box[T]` and `Cell[T]` show as the `T` they hold (Hardened_51) | `[STD-19]`, `[ERR-4]`, `[TYP-39]` |
| ODR-098 | String capacity is measured in bytes; reserve asks for additional bytes; insert takes a byte offset and a char, remove returns that char, truncate keeps a byte prefix, and shrinking retains capacity, with UTF-8 boundary and integer-size checks in every profile (Hardened_52) | `[TXT-11]`, `[TXT-4]`, `[TYP-31]`, `[HEAP-8]` |
| ODR-099 | the owner's direction (2026-10-11), its detail under the delegation: where a program turns a callable type into `extern "C" fn`, a function whose value has that type returns no raw pointer made from a reference to a parameter it borrows by value, which a call from C passes as a copy that ends with the call (`E5042`); followed through copies, conversions and aggregates, not calls; a value read through the parameter and a pointer stored in it are not into the copy | `[FFI-21a]`, `[FN-6]`, `[FN-1]` |
| ODR-100 | the owner's (2026-10-11): the iterator adapters take over the callable and the iterator they keep (`owned`); a lambda written at one keeps the captures `[CLO-2]` infers and the adapter lives no longer than they do; only a consumed callable parameter whose captured state must carry the static region (`thread.spawn`, `jobs.submit`, registries) makes a lambda written there capture by move | `[CLO-15]`, `[CLO-4]`, `[CLO-7]`, `[STD-19]` |
| ODR-101 | ruled under the delegation (2026-10-11, the owner: "fix that autonomously"): the entry module's `main` in a form `[FN-8]` does not list (another parameter or more than one, another return type, type parameters) is `E2183` at its name; the one parameter may have any name and mode, its type `Array[String]`; an `E` that is not `Display` is `E2040` | `[FN-8]` |
| ODR-102 | ruled under the delegation (2026-10-11, autopilot): a function or capture-free lambda used as a value is a plain `Copy` value that becomes an owned callable value where one is wanted; an owned callable value moves and drops its state, is lent mutably to a borrowed parameter (`E3023` from a place that is not mutable, `E3022` twice at once), is neither `Send` nor `Sync`, equals another calling the same function with the same state, and given to C panics when it holds state; a lambda copies what it only reads (`[CLO-13]`) where it becomes an owned callable value, "assigned after" being later in the body or anywhere in a loop around it | `[CLO-3]`, `[CLO-10]`, `[CLO-13]`, `[CLO-2]`, `[THR-8]`, `[THR-9]` |
| ODR-103 | ruled under the delegation (2026-10-12, autopilot): a lambda that borrows, stored or sent where no view may go, is shape B9's `E3026` (a closure outlives what it captures), help `owned fn` or `thread.scope()`; `E3063` stays for other views | `[CLO-3]`, `[CLO-4]`, `[THR-10]` |
| ODR-104 | ruled under the delegation (2026-10-12, autopilot): a `return` from a class `init`, like the end of its body, hands the object to its caller, so every field without a default is assigned on that path (`E2100`); `mut self` in a class `init` is accepted and means the same as `self` | `[CLS-2]`, `[STR-6]` |
| ODR-105 | ruled under the delegation (2026-10-12, autopilot): a `let` field assigned outside its `init` is `E2104`, a code of its own beside `E2103` | `[CLS-9]` |
| ODR-106 | ruled under the delegation (2026-10-16, the owner: "allow ?.is()", "make it an error"): `is` is a method name after `fn` in a type's body, `.` and `?.`, as `not` is (ODR-040), so `AnyError` has `is[E]()`; a function standing alone may take neither `not` nor `is` (`E0005`) | `[LEX-15]`, `[ERR-8]` |
| ODR-107 | ruled under the delegation (2026-10-16): optional chaining `a?.f`, `a?.m[T](…)` on an `Option`: `None` when `a` is, else the member wrapped in `Some` unless it is an `Option` (then it is the result) or nothing; `a` read as an unconsumed place, a non-`Copy` field lent | `[EXP-10]` |
| ODR-108 | ruled under the delegation (2026-10-16, the owner: "fix all the stuff yourself"): `[SEL-1]` reports every local, parameter and field by the row its type picks and every type by its kind; a type with no row is a `value`, a `view`, a `callable` or a `raw pointer` | `[SEL-1]` |
| ODR-109 | ruled under the delegation (2026-10-16): a `Cell` shows a copy of its value, as `get` reads it; a non-`Copy` value that writes its own `Display` or `Debug` is not shown from a `Cell` (`E2040`), since a `Cell` lends its value to nothing | `[TYP-39]`, `[CELL-2]` |
| ODR-110 | ruled under the delegation (2026-10-16): `[MAN-8]` names the keys of `[c]` (`compiler`, `flags`), `[link]` (`libs`), `[jobs]` (`workers`), `[realtime]` (`contracts`) and `[ffi]` (`evidence`); any other key or section is `E9001` | `[MAN-8]` |
| ODR-111 | ruled under the delegation (2026-10-16): the marker interfaces are `std.core`'s, usable as bounds; `unsafe extend T implements Send|Sync:` writes one by hand for a struct or enum of the package (generic ones per their parameters' marker bounds), `E3100` without `unsafe`; a class only as `@sync`, `Copy` only by `@derive(Copy)`, `Sized` never (`E2040`) | `[TYP-43]`, Part IV §8, grammar `extend_decl` |
| ODR-112 | ruled under the delegation (2026-10-16): an extension may target a parameter standing alone with blanket parameters besides (`extend[T, U] T implements Wrap[U]`); a call leaving a blanket parameter open takes it from the one instance of the interface the receiver's type implements through the block | `[GRM-34]` |
| ODR-113 | ruled by the owner (2026-10-16): `[STD-9]`'s example of a type with neither `Display` nor `Debug` is a `RefCell`, not a class: a class handle always has `Debug`, its class and address (`[TYP-36]`) | `[STD-9]` |
| ODR-114 | ruled by the owner (2026-10-16): a moved-from place, read, moved again or borrowed, is `E3040` with `[OWN-3]`'s help; `E3050` is a place uninitialised on some path, one never assigned there | `[OWN-3]`, `[BRW-7]` |
| ODR-115 | ruled by the owner (2026-10-16): `unsafe fn(…) -> R` is a callable type whose call is an `unsafe` operation; an `unsafe fn` coerces only to one, any function to one; as a bound it is met by any function | `[FN-6]`, `[UNS-1]`, grammar `fn_type` |
| ODR-116 | ruled by the owner (2026-10-16): a `MutSpan[T]` converts to `Span[T]` where one is expected, a shared reborrow; `[TYP-5]`'s complete list lacked it, and Part XII.1's `integrate(ps.pos, ps.vel, …)` passes a mutable column to a `Span` parameter | `[TYP-5]` rule 6, `[SPN-3]`, `[SOA-2]` |
| ODR-117 | ruled by the owner (2026-10-19): the profiles are `debug`, `development` and `release`; `release` is the build users get (formerly `shipping`), `development` the optimised build still debuggable (formerly `release`) | `[PRF-1]`, `[PRF-2]`, `[PRF-3]`, `[MAN-7]`, `[PAN-1]`, `[RT-4]`, `[HR-27]`, `[BUD-2]` |
| ODR-118 | ruled by the owner (2026-10-19): a `release` build ignores `[build] reload` and prints the note `hot reload is off in release builds`; `E9033` is retired | `[MAN-7]`, `[PRF-2]`, Annex B |
| ODR-119 | ruled by the owner (2026-10-19): in a hot-reload build a reloadable function is never inlined, whatever `@inline` says; builds without hot reload inline it | `[CG-C-3a]`, `[HR-9a]` |
| ODR-120 | ruled by the owner (2026-10-19): a migrated instance's old version is freed field by field without the type's own `drop`, since the instance lives on in its new form | `[HR-2a]` |
| ODR-121 | ruled by the owner (2026-10-19): `ReloadError`'s parts are `type_name: String`, `reason: String`, `instance: u64`, `message: String`, `error: String` | `[HR-38]`, `[HR-35]` |
| ODR-122 | ruled by the owner (2026-10-19): a foreign type names its destructor with `@ffi(destroys(f))` on its declaration | `[FFI-36a]` |
| ODR-123 | ruled by the owner (2026-10-19): one live object adopted twice through one local in one function is `E5068` at compile time | `[FFI-36b]` |
| ODR-124 | ruled by the owner (2026-10-19): a status enum is marked `@ffi(status=E, ok=X, error=N)`, naming its error type; `[ERR-6]` and `[FFI-16]` disagreed on the spelling and the name | `[ERR-6]`, `[FFI-16]`, `[FFI-11]` |
| ODR-125 | ruled by the owner (2026-10-19): an `inout_count` function with a result of its own returns a tuple, that result first, then each pointer output in the C declaration's order | `[FFI-11b]` |
| ODR-126 | ruled by the owner (2026-10-19): when an instance moves, handles inside tuples, `Box` contents and closure captures are rewritten too | `[HR-15]` |
| ODR-127 | ruled by the owner (2026-10-19): a dependency `{ git, version }` resolves against the versions its repository has tagged `vMAJOR.MINOR.PATCH`; a requirement with no source is for a registry this version does not have (`E0900`) | `[VER-5]`, `[MAN-8]` |
| ODR-128 | ruled by the owner (2026-10-19): a field of an `@export_table` holds the function its default names (`= door_created`), else the module's function of its own name | `[FFI-26]` |
| ODR-129 | ruled by the owner (2026-10-19): an `@export_table`'s symbol carries its protocol version, `Name_vN`, with no separate marker | `[FFI-26]`, `[ABI-1]` |
| ODR-130 | ruled by the owner (2026-10-19): a field of an `@export_table` takes the thread contract of the function it holds | `[FFI-33c]` |
| ODR-132 | ruled under the delegation (2026-10-19, autopilot), the owner having chosen a typed pointer: `tok.ptr()` is a `std.ffi.ObjectPtr[C]` for a class `C`, a `*mut U` for a `Box[U]`, a `*mut u8` for a `Box[dyn I]`; an `ObjectPtr` is a class object's address with no count of its own, one pointer to C | `[FFI-23]` |
| ODR-131 | ruled by the owner (2026-10-19): a `cdylib` exports the marker `ember_module_protocol_vN`, which its host looks up before calling into it; `[FFI-28]` and `[ABI-1]` disagreed | `[FFI-28]`, `[FFI-31]`, `[ABI-1]` |
| ODR-133 | ruled by the owner (2026-10-19, A1: the data is fixed at a checkpoint, so layout changes apply live too), built under the delegation: each function keeps a record of the call it is in and the image lists what the rest of it depends on; a change no frame below the checkpoint depends on applies (instances migrate, one a frame references in place), one a frame depends on is refused, naming it | `[HR-3]`, `[HR-3b]`, `[HR-33]` |
| ODR-134 | ruled under the delegation by the owner's standing rule (2026-09-08, ERR-017: a reserved word that blocks a name the standard library must declare becomes contextual, never a rename or `r#`): `open` is a keyword only before `class`, so `File.open` (`[STD-22]`) can be declared; the reserved set has 47 words | `[LEX-15]`, `[STD-22]` |
| ODR-135 | ruled under the delegation: `[build] max_instantiations` applies to the package's own generics; the standard library's instances are not the package's to budget, and counting them made any small ceiling warn on code the program never wrote | `[MONO-3]` |
| ODR-136 | ruled under the delegation: `Nondet` for padding is recorded at the reinterpreting cast of a pointer to a type with padding; reading uninitialised memory is undefined (`[PHIL-10]` lists it among what Safe Ember cannot do), an `unsafe` obligation, not a nondeterministic result | `[DET-2]`, `[PHIL-10]` |
| ODR-137 | ruled under the delegation: `@non_exhaustive` is a C enum's (unit-only, with `@repr`), holding any value of its integer type (`match` needs `_`, no niche, `E(9)`/`9` for an unnamed value) and crossing the boundary as that integer; on a hand declaration `param(p, out, one)` is `mut p: T` returned by the call; `@ffi(status=E, ok=X, error=N)` is written on `E` itself, and `T` in `Result[T, N]` is `void`, the one output's type, or the tuple of several | `[FFI-8]`, `[FFI-11]`, `[FFI-11b]`, `[FFI-16]`, `[ERR-6]` |
| ODR-138 | ruled under the delegation: `@packed` beside `@align(N)` on one type, or a `@packed` type holding a field whose type states an alignment with `@align`, is `E0104`; no padding and a size that is a multiple of `N` cannot both hold | `[LAY-2]` |
| ODR-139 | ruled under the delegation: `@repr(T)` applies to enums only (`E0104` on a struct); the attribute table's row is split | `[LAY-2]`, `[ATT-1]` |
| ODR-140 | ruled under the delegation: `E2170` is a reference to an unaligned field of a packed type (alignment above one byte), as `[LAY-2]`'s text says; the code table's wording follows it | `[LAY-2]` |
| ODR-141 | ruled under the delegation: `run(system, arg)` takes one argument after the system (a tuple for several, `()` for none); `Not[D]`'s item is `()`; `Entity` is a query parameter and `Option[P]` applies to any parameter; the ECS's compiler support is the conversion of a function to a `System` and the compile-time check of `run_parallel`'s literal | `[ECS-3]`, `[ECS-4]` |
| ODR-142 | ruled under the delegation: a query none of whose parameters requires a component iterates every live entity, in the world's order (creation order with swap-remove holes) | `[ECS-3]`, `[ECS-6]` |
| ODR-143 | ruled under the delegation: a shared method called on `self` inside a `mut self` method is lent the method's access as a read: its reads are not checked again, and a write to the object under it is | `[EXC-5]` |
| ODR-144 | ruled under the delegation: `some I` is legal as a function's or a method's return type; in an interface's method (declared or default) it is `E2260`, and an associated type bounded by `I` states what each implementation returns | `[GRM-31]`, `[TYP-32]` |
| ODR-145 | ruled under the delegation: a raw pointer made from an integer is a compile-time value and a `const` of pointer type may hold one; a pointer into the evaluator's heap may not leave it (`E6010`); a null-pointer or pointer-cast macro is imported as such a `const` | `[FFI-6a]`, `[CT-2]` |
| ODR-146 | ruled under the delegation: an object-like macro that expands to nothing is not a declaration and is not reported | `[FFI-6a]` |
| ODR-147 | ruled under the delegation: an overlay entry marked `@ffi(macro_fn)` writes one Ember type for each parameter and for the result | `[FFI-6b]`, `[GRM-35]` |
| ODR-148 | ruled under the delegation: `ember bind --init <header>` writes `<stem>.overlay.em` beside the header | `[CLI-5]` |
| ODR-149 | ruled under the delegation: `overlay` takes one file or a list applied in order; `override` before an item replaces an earlier file's entry, any other second entry for one name is `E5010` | `[FFI-30c]`, `[GRM-35]` |
| ODR-150 | ruled under the delegation: the examples are corrected to the rules (`mut` on written `MutSpan` parameters, `[FN-1]`; `ember,fragment` where a block names undeclared items; XVII.5's expected diagnostic); every block is checked as `[TST-7]` says | `[TST-7]`, `[FN-1]`, `[TST-1]` |
| ODR-151 | ruled under the delegation: a compiler declares its profile as `ember --version`'s `profile:` line; each profile holds the one before it; Ember Core's part of XV is XV.1 and XV.3–XV.6, and XV.2 and XV.7 (the io layer) join Ember Systems; Ember Dynamic adds Annex B to Ember Native | `[CONF-1]`, `[CONF-2]`, `[CONF-3]`, `[CONF-5]` |
| ODR-152 | ruled under the delegation: the cost table gains the rows `Virtual method call` (conditionally elided, `[DSP-2]`, `[DSP-5]`), `Downcast check` (`[DSP-4]`), `assert_disjoint` comparison (`[DSJ-5]`), `Allocation` (`[EFF-1]`) and `contains` (`[STD-8a]`), the five costs `ember inspect --cost` reported with no row; `rule_index.py` compares the rows the compiler reports with the table | `[COST-3]`, `[COST-5]` |
| ODR-153 | ruled with the owner's approval: each benchmark builds each side at several code layouts, the same for both sides, each moving the code to other addresses and changing nothing else, keeps each side's fastest by a short interleaved run, then times the two as before | `[BEN-1]` |
| ODR-154 | ruled with the owner's approval: the performance gate's verdict of record is the one measured on the recorded reference machine before a change is published; CI on shared machines reports the results without failing (`--report`) | `[TST-28]` |
| ODR-155 | ruled under the delegation: Annex A's map comprehension names its type (`index: Map[str, int] = …`), and its `unsafe:` block carries a safety note; the annex is generated from `docs/spec-source/appendix-a.em`, which compiles | `[TST-6]`, `[TYP-38]`, `[UNS-8]` |
| ODR-156 | the owner's answer: a C++ parameter that keeps the pointer takes a `Retained` token the program holds (`tok = Retained.pin(self)`, then `tok`); `self` there is `E5060`; a parameter borrowed for the call takes `self` | `[FFI-39c]`, `[FFI-23]` |
| ODR-157 | ruled under the delegation: a `@packed` type holding a SIMD vector is `E0104`, as one holding an `@align` type is (ODR-138); a vector's alignment is its size (`[SIMD-1]`) | `[LAY-2]`, `[SIMD-1]` |
| ODR-158 | ruled under the delegation: `std.ser`'s descriptions own their text (`String`) and arrays; a field names its type by name; a class field's offset counts from the end of the header; attribute arguments are `(name, AttrValue)` pairs and an `@attribute` struct's fields are numbers, text, `bool` or `char` (`E0104`); `@reflect` implements `std.ser.Reflect` (`type_info` needs it, `E2040`), inherited by derived classes; `reflect[T]()` outside compile time is `E6010` | `[RFL-1]`, `[RFL-2]`, `[RFL-3]`, `[RT-3]` |
| ODR-159 | ruled under the delegation: the serialisation interfaces and formats are as §XIV.2 says (`Writer`/`Reader`, the version, JSON's `"$version"`/`"$variant"`/`null`, a self-describing binary form, defaults for `@ser(default)`/`@ser(since=N)`) | `[SER-1]`, `[SER-2]` |
| ODR-160 | the owner's answer: a `const` method of a class with a `mutable` member is `self` when its visible body writes no field, else `mut self`; an overlay `class C:` method line wins; to be built | `[FFI-40a]` |
| ODR-161 | the owner's answer: a C++ static data member reads and writes as a global, through the address thunk; to be built | `[FFI-41]` |
| ODR-162 | the owner's answer: an `@export` function may hand C++ an object of a `@sync` class, presented by the `--cpp` header as a class of the same name; to be built | `[BLD-FFI-5a]`, `[FFI-31b]` |
| ODR-163 | the owner's ruling: a foreign static read only in `unsafe` need not be `Sync`; a C or C++ variable holding a pointer imports as any other, used in `unsafe` | `[STA-1]`, `[FFI-41]`, `[FFI-10]` |
| ODR-164 | the owner's ruling, reversing ODR-021: float `//` is `floor(a / b)` and `%` is `a - b * floor(a / b)`, each step rounded | `[TYP-29]` |
| ODR-165 | delegated: an inline namespace adds nothing to a C++ name in an import | `[FFI-41]`, `[FFI-17b]` |
| ODR-166 | delegated: a C++ function returning an opaque class by value gives a `ForeignBox[T]` | `[FFI-32]`, Annex C's import table |
| ODR-167 | delegated: `[CXX-1]`'s corpus is nlohmann/json until the owner approves more downloads | `[CXX-1]` |
| ODR-168 | delegated: `format`'s spec is a string literal, checked where it is written | `[STD-18]`, `[LEX-19]` |
| ODR-169 | delegated: `lto = "thin"` is `on` where the toolchain does only `off` and `on` (cl, gcc) | `[BLD-6]` |
| ODR-170 | delegated: the build record is `target/<profile>/inspect/<name>.build.json`, its `lto` keys `asked`, `used`, `why` | `[BLD-6]` |
| ODR-171 | delegated: objects another link reads take no LTO with cl or Clang, fat LTO objects with gcc | `[BLD-6]` |
| ODR-172 | delegated: a figure's spread is its IQR over its median; both gates apply below 15 % | `[BUD-3b]` |
| ODR-173 | delegated: the Vulkan-sized header is generated in Vulkan's shape and not counted; B6's cold cache is a new global cache; B7 and B9 build with the program cache off | `[BUD-1]`, `[BUD-2]` |
| ODR-174 | delegated: `W2192`, use of a deprecated item, at each use outside its module; to be built | `[VER-3]`, §XVII.9 |
| ODR-175 | delegated: `pub unsafe interface Allocator` with `allocate`, `deallocate`, `reallocate` over `Layout`; `[TYP-43]` admits an `unsafe interface`; to be built | `[ALC-2]`, `[TYP-43]`, III.2 |
| ODR-176 | delegated: an ended region a run-time selection needs is a loan conflict, `E3021` | `[LT-39]` |
| ODR-177 | delegated: `[CLS-1]` and `[OBJ-4]` hold except as `[OPT-1]` permits; a host's allocator count is not behaviour | `[CLS-1]`, `[OBJ-4]`, `[OPT-1]` |
| ODR-178 | delegated: Visual Studio shows a lone local collection's length only; a C struct per collection instance with the backend's next rework | `[CG-C-9]` |
| ODR-179 | delegated: each `[DIA-21]` row is a catalogued shape with an id and codes, named by marks and `tests/ui/` | `[DIA-21]`, `[TST-9]` |
| ODR-180 | delegated: an `AnyError` function fails with a message through `AnyError.msg(text)`, which a catalogued help names | `[ERR-8]`, `[DIA-21]` |
| ODR-181 | delegated: `ref T` implements `Eq`, `Ord`, `Hash`, `Display`, `Debug` as `T` does | `[TYP-36]` |
| ODR-182 | delegated: a bare interface name in a type position, read with `[OBJ-2]` by ODR-226 | `[OBJ-2]`, `[GRM-8b]`, `[DIA-21]` |
| ODR-183 | delegated: `char` has `is_digit`, `is_alphabetic`, `is_alphanumeric`, `is_whitespace` and `is_ascii_*`, with Appendix E rows | `[TXT-10]`, Appendix E |
| ODR-184 | delegated: `float(x)` and `int(x)` are answered by the argument's type | `[DIA-21]` |
| ODR-185 | delegated: sequence repetition is refused with a catalogued help | `[PHIL-14]`, `[DIA-21]` |
| ODR-186 | delegated: a single-quoted string is a `[DIA-21]` row whose fix-it is the double-quoted string | `[DIA-21]` |
| ODR-187 | delegated: a list operation on an iterator has the help `.collect[Array[…]]()` | `[DIA-21]` |
| ODR-188 | delegated: the published rate counts the catalogue's helps; the gap to the printed ones is a defect | `[TST-10]` |
| ODR-189 | delegated: a consumed callable parameter is an owned callable value in its function and may be stored | `[CLO-3]` |
| ODR-190 | delegated: MSVC gets no vectorisation pragma | `[CG-C-6]` |
| ODR-191 | delegated: a changed header is any change to it or a file it includes | `[HR-23]`, `[HR-41]` |
| ODR-192 | delegated: `[HR-23]` covers `import c` headers | `[HR-23]`, `[HR-41]` |
| ODR-193 | delegated: `[HR-14]`'s FFI row reads "layout or fields changed on a type used across FFI by value" | `[HR-14]` |
| ODR-194 | delegated: `on_relocate` is 0.8.5's spelling with `ObjectPtr[C]`, called after commit publishes and before reclaim; to be built | `[HR-20]` |
| ODR-195 | delegated: `[HR-24]` and `[HR-41]`'s GPU row go after 1.0 with Annex D | `[HR-24]`, `[HR-41]` |
| ODR-196 | delegated: `@borrows(self.f)` names a view field of the receiver; std's iterators declare it, so a map's keys borrow the map | `[LT-1a]`, `[CTL-1]` |
| ODR-197 | delegated: `borrowed one exclusive` is written `p: ref mut T` as well as `mut p: T` | `[FFI-11]` |
| ODR-198 | delegated: `std::variant` crosses as an enum named by `@ffi(variant=[…])` on a `type` entry; to be built | `[FFI-17a]` |
| ODR-199 | delegated: `@ffi(std_function)` makes a `std::function` an owned callable value; to be built | `[FFI-17a]` |
| ODR-200 | delegated: a fixed-extent `std::span<T, N>` crosses as `ref [T; N]` / `ref mut [T; N]`; to be built | `[FFI-17a]` |
| ODR-201 | delegated: a module and an item of one name in one scope, or a type named like a built-in type, is `E1032` | `[MOD-1]`, `[MNG-1]`, §XVII.9 |
| ODR-202 | delegated: a transliterated component is marked `u` before its length | `[MNG-3]` |
| ODR-203 | delegated: `em_obj_`, `em_vt_`, `em_ti_` take the mangled form without its own `em_` | `[MNG-4]` |
| ODR-204 | delegated: a root package's items have no package component in a symbol | `[MNG-1]` |
| ODR-205 | delegated: the canonical spelling of generic arguments is written into the spec, as built | `[MNG-6]`, `[MNG-1]`, `[MONO-1]` |
| ODR-206 | delegated: an interface-qualified method, a structural owner and `[CLO-3]`'s copies are spelled as built | `[MNG-1]` |
| ODR-207 | delegated: `sorted` over an iterator of references gives clones; to be built | `[STD-26]` |
| ODR-208 | delegated: a lambda given to a `mut` callable parameter is `E3027`, shape B10 | `[CLO-3]` |
| ODR-209 | delegated: a same-line `:` lambda body ends at the statement's newline; to be built | `[GRM-17]` |
| ODR-210 | delegated: std's consumers that take a callable take it over; to be built | `[STD-19]` |
| ODR-211 | delegated: `@borrows(self.f)` may name a field of a type-parameter type, checked per instance; to be built | `[LT-1a]` |
| ODR-212 | delegated: `@borrows(self.f)` holds for a receiver in any mode | `[LT-1a]` |
| ODR-213 | delegated: an ASCII `_` beginning an escape-like run is `_u005F_` in every component, which is then transliterated; C member names alike; to be built | `[MNG-3]` |
| ODR-214 | delegated: `E1032` is name shape N14 | §XVII.6.2 |
| ODR-215 | delegated: a built-in type's path is under `std.core` in a symbol; to be built | `[MNG-1]` |
| ODR-216 | delegated: `E1032` compares the build's modules | `[MOD-1]` |
| ODR-217 | delegated: `E1032` covers an `import` binding | `[MOD-1]` |
| ODR-218 | the owner's answer: `run_parallel` runs systems on threads; a function value is `Send` when its captures are; a component must be `Send`; to be built | `[ECS-4]`, `[THR-8]`, `[CLO-10]` |
| ODR-219 | the owner's answer: a class with a C++ base is built in Ember's order, the C++ constructor at `super.init(…)` | `[FFI-39b]` |
| ODR-220 | the owner's answer: the catching C++ calls on `@noalloc` or main-thread paths are listed by `ember inspect --cost`; to be built | `[FFI-24c]` |
| ODR-221 | the owner's answer: an Ember class inherits from a C++ class with no overlay entry; the trampoline passes on the overridden virtuals; to be built | `[FFI-39]`, `[FFI-48]` |
| ODR-222 | the owner's answer: releases are published on the Ember repository's GitHub Releases by the release workflow; to be built | `[TOOL-1]` |
| ODR-223 | the owner's answer: the LLVM backend is part of 1.0; Ember's download carries Clang and LLD; to be built | `[TOOL-2]` |
| ODR-224 | the owner's answer: the macOS download is built and tested on GitHub's macOS machines and stays in 1.0; to be built | `[TOOL-1]` |
| ODR-225 | delegated: `[DIA-21]` gains P21..P30 as built, an id column, and Appendix E rows | `[DIA-21]`, Appendix E |
| ODR-226 | delegated: a bare interface name is the class handle; where no class implements the interface it is `E2173` (P22) | `[OBJ-2]`, `[GRM-8b]`, `[DIA-21]`, §XVII.9 |
| ODR-227 | delegated: `x == None` compiles where the payload is `Eq`; the habit row is where it has none | `[DIA-21]` |
| ODR-228 | delegated: `[LEX-26]`: a malformed literal is `E0009`, an unclosed one `E0101` | `[LEX-26]`, §XVII.9 |
| ODR-229 | delegated: `char`'s classes have Python's meaning; `is_ascii_*` restrict them to ASCII | `[TXT-10]` |
| ODR-230 | delegated: `AnyError.msg(owned text: String)` is an error whose text is `text` | `[ERR-8]` |
| ODR-231 | delegated: `int(s)`'s fix-it ends in `?` or `.unwrap()`, `max(xs)`'s in `.unwrap()` | `[DIA-21]` |
| ODR-232 | delegated: an unknown field is `E2072` (shape N3), not `E1010`; to be built | §XVII.6.2 |
| ODR-233 | delegated: a type name is a value candidate only before `.` | `[DIA-24]` |
| ODR-234 | delegated: O2's help offers the borrow first, then `.clone()` and `owned` | §XVII.6.1 |
| ODR-235 | delegated: B4's help adds a new local for a rebound parameter | §XVII.6.1 |
| ODR-236 | the owner's answer: a comprehension may take its list over and move the items, or copy them with `.clone()`; to be built | `[GRM-27]`, `[GRM-15]` |
| ODR-237 | the owner's answer: `Box[T]` is `Clone` when `T` is; an interface may require `Clone`, and then `Box[dyn I]` is `Clone`; to be built | `[TYP-36]`, `[TYP-22]`, IX.1 |
| ODR-239 | delegated, confirmed by the owner: `std.mem.System`, the platform's allocator, for a global allocator to wrap | `[ALC-2]` |
| ODR-240 | the owner: std's callable-taking functions borrow a named callable; a lambda written at the call is given (replaces ODR-210) | `[STD-19]` |
| ODR-241 | the owner: `Box == Box` as what they hold; `Box[dyn I]` when `I` requires `Eq`, two kinds never equal | `[TYP-36]` |
| ODR-242 | the owner: no fixed times; the figures are a gate by before-and-after comparison on one machine | `[BLD-10]`, `[BUD-1]`, `[BUD-2]`, `[BUD-3]`, `[BUD-3a]` |
| ODR-243 | delegated: a `const` value by the compiler's own evaluator, never a built C program | `[CT-4]` |
| ODR-244 | the owner: `debug` one C file per module, recompiled as it changes; `release` one C unit | `[BLD-1]` |
| ODR-245 | delegated: `user_data=p@N` names the callback parameter that receives the user data | `[FFI-21]` |
| ODR-246 | delegated: `destroys(f)` may take the registering call's result, which the token keeps | `[FFI-11]` |
| ODR-247 | delegated: `threads(creator)` and `handle` on a function are `E5002` | `[FFI-1]` |
| ODR-248 | delegated: `@layout(c, union)` and `@layout(c, flexible)`, as built | `[FFI-8]` |
| ODR-249 | delegated: `@ffi(newtype)` a nominal type with the typedef's representation, `as` only | `[FFI-8]` |
| ODR-250 | delegated: a variadic call's further arguments typed; a promoted one `E5015`, as built | `[FFI-8]` |
| ODR-251 | delegated: a `unique_ptr` whose deleter holds state is `E5034` | `[FFI-17a]` |
| ODR-252 | delegated: `operator*`/`->` import `get()` and `get_mut()` | `[FFI-42]` |
| ODR-253 | delegated: a `nullable` callback with user data is an `Option` of a callable value | `[FFI-21]` |
| ODR-254 | delegated: the non-panicking twin is an accessor's; a constructor's or mutation's bug panics with none | `[STD-14]` |
| ODR-255 | delegated: `dyn I` is `Send`/`Sync` when `I` requires it; shared reads in `run_parallel` need `Sync` | `[THR-8]`, `[ECS-4]` |
| ODR-256 | delegated: the identifier interner is the session's | `[IDE-6]` |
| ODR-257 | delegated: the module cache serves `ember check` as `ember build` | `[BLD-2]` |
| ODR-258 | delegated: Ninja carried beside Clang and LLD; from a checkout on `PATH`, else an error naming the install | `[BLD-4]` |
| ODR-259 | the owner: `debug` without Ember's speed rewrites unless `rewrites = true` | `[PRF-3]` |

Generated: every rule whose text differs from Hardened_2 in Hardened_52, the last
0.9.9.

| Rule | H2 → Hardened_52 |
|---|---|
| `[ALC-1]` | changed |
| `[ARN-3]` | changed |
| `[ARN-11]` | changed |
| `[ATT-4]` | changed |
| `[BLD-2]` | changed |
| `[BLD-6]` | changed |
| `[BLD-8]` | changed |
| `[BRW-5]` | changed |
| `[BRW-8]` | changed |
| `[CELL-2]` | changed |
| `[CG-C-11]` | changed |
| `[CG-LL-1]` | added |
| `[CG-LL-2]` | added |
| `[CG-LL-3]` | added |
| `[CLO-7]` | changed |
| `[CLS-4]` | changed |
| `[CLS-7]` | changed |
| `[CORO-6]` | changed |
| `[CT-4]` | changed |
| `[CTL-1]` | changed |
| `[CTL-3]` | changed |
| `[DET-2]` | changed |
| `[DET-4]` | changed |
| `[DRP-6]` | changed |
| `[DRV-1]` | changed |
| `[EFF-1]` | changed |
| `[EFF-7]` | changed |
| `[EFF-16]` | changed |
| `[ERR-1]` | changed |
| `[ERR-4]` | changed |
| `[ERR-10]` | changed |
| `[ERR-12]` | changed |
| `[EXC-3]` | changed |
| `[EXC-17]` | changed |
| `[EXC-19]` | changed |
| `[FFI-8]` | changed |
| `[FFI-10]` | changed |
| `[FFI-11]` | changed |
| `[FFI-15]` | changed |
| `[FFI-33]` | changed |
| `[FMT-1]` | changed |
| `[FN-1]` | changed |
| `[FN-1a]` | changed |
| `[FN-3]` | changed |
| `[FN-5]` | changed |
| `[FN-6]` | changed |
| `[FN-9]` | changed |
| `[FN-10]` | changed |
| `[GRM-34]` | changed |
| `[GRM-37]` | changed |
| `[GRM-39]` | added |
| `[HASH-1]` | changed |
| `[HASH-2]` | changed |
| `[HASH-3]` | changed |
| `[HEAP-10]` | changed |
| `[IFC-3]` | changed |
| `[IFC-4]` | changed |
| `[LEX-2]` | changed |
| `[LEX-15]` | changed |
| `[LT-1]` | changed |
| `[LT-1a]` | changed |
| `[LT-1b]` | changed |
| `[LT-7]` | changed |
| `[LT-44]` | changed |
| `[OWN-6]` | changed |
| `[OWN-7]` | changed |
| `[PAR-2b]` | changed |
| `[PHIL-3]` | changed |
| `[PHIL-13]` | changed |
| `[RC-3]` | changed |
| `[RC-5]` | changed |
| `[RNG-4]` | changed |
| `[RNG-5a1]` | changed |
| `[RNG-5a2]` | changed |
| `[RNG-8]` | changed |
| `[RT-10]` | changed |
| `[SIMD-1]` | changed |
| `[SIMD-3]` | changed |
| `[SIMD-5]` | changed |
| `[SIMD-7]` | changed |
| `[SPN-5]` | changed |
| `[STD-4]` | changed |
| `[STD-5]` | changed |
| `[STD-11]` | changed |
| `[STD-12]` | changed |
| `[STD-15]` | changed |
| `[STD-16]` | changed |
| `[STD-17]` | changed |
| `[STD-18]` | changed |
| `[STD-19]` | changed |
| `[STD-20]` | changed |
| `[STD-21]` | changed |
| `[STD-27]` | added |
| `[STD-28]` | added |
| `[STR-3]` | changed |
| `[STR-5]` | changed |
| `[THR-1]` | changed |
| `[THR-6]` | changed |
| `[THR-8]` | changed |
| `[THR-9]` | changed |
| `[TXT-5]` | changed |
| `[TXT-10]` | changed |
| `[TXT-11]` | changed |
| `[TYP-8]` | changed |
| `[TYP-9]` | changed |
| `[TYP-13]` | changed |
| `[TYP-15]` | changed |
| `[TYP-19]` | changed |
| `[TYP-20]` | changed |
| `[TYP-21]` | changed |
| `[TYP-23]` | changed |
| `[TYP-24]` | changed |
| `[TYP-26]` | changed |
| `[TYP-27]` | changed |
| `[TYP-28]` | changed |
| `[TYP-29]` | changed |
| `[TYP-34]` | changed |
| `[TYP-37]` | changed |
| `[TYP-38]` | changed |
| `[TYP-39]` | changed |
| `[TYP-41]` | added |

## H.6 Changes in 0.9.10 (from 0.9.9_Hardened_52)

**0.9.10_Hardened_1** is the owner's design G8-4 (`docs/proposals/G8-4-counts-and-positions.md`,
decided 2026-10-05, built with `docs/MIGRATION-0.9.10.md`). It changes what programs are accepted,
so it is a new language version (`[VER-9]`):

* Every iterator counts and numbers its items in a type that holds every count and number it can
  have: `Iterator.Count` and `Iterator.Position`, `int` unless the iterator says otherwise; ranges
  of 64-bit numbers count in `u64` (`..`) and `u128` (`..=`) and number in `i128`; ranges of
  128-bit numbers count in `u128` (`..`) and `u256` (`..=`) and number in `i256` (`[STD-19]`).
  This replaces ODR-089's "counts are `int`s" for what Ember hands back (what a caller passes stays
  an `int`) and ODR-091's backwards-`enumerate` panic.
* `chain`, `flatten` and `flat_map` step up only as far as their counts need (`u256` the limit).
* The visible-numbers rule: where a range's numbers are written in place, `enumerate`, `len` and
  `count` give the smallest type that holds the values (`[STD-19]`, `[STD-26]`).
* `skip_front`, `skip_back` and `nth_back` leave out items at once through the adapters, so none
  passes a gigantic run of items one at a time (`[STD-19]`).
* `u256` and `i256`, the 256-bit counts (`[TYP-42]`).
* `a..` gives its type's maximum and panics only when the value after it is asked for, in a `for`
  and through its iterator, which leaves values out at once (`[CTL-3]`, D-526; ODR-027 said the
  loop overflowed on reaching the maximum).
* `for i in a..=T.MAX` ends after `T.MAX`, as `[CTL-3]` said (D-525: it ran forever).
* An index or a size wider than `usize` that `usize` does not hold is out of bounds (`[TYP-31]`,
  D-527: its low bits were used).
* `i256` and `u256` implement `Add`, `Sub`, `Neg`, `AddAssign` and `SubAssign` (ODR-040); `parse`
  does not read them (`[TYP-42]`).
* A loop numbering in a type wider than 64 bits has a copy computing the numbers as `int`s, taken
  when every number fits one (`[CTL-3b]`, the owner's decision C).
* A program that put such a count or number into an `int` converts it (`as int`, or `to_int()` in
  generic code); `docs/MIGRATION-0.9.10.md` lists the cases.

Generated: every rule whose text differs from Hardened_52.

| Rule | Hardened_52 → 0.9.10_Hardened_1 |
|---|---|
| `[CTL-3]` | changed |
| `[CTL-3b]` | changed |
| `[STD-19]` | changed |
| `[STD-26]` | changed |
| `[TYP-31]` | changed |
| `[TYP-42]` | added |

## H.7 Changes in 0.9.10_Hardened_2 (from 0.9.10_Hardened_1)

**0.9.10_Hardened_2** records the owner's rulings of 2026-10-06 on three decisions of the audit of
the delegated rulings, and on the type of `enumerate`'s numbers in a function written for any type
(`docs/DECISIONS.md`, ADR-140). It changes what programs are accepted
(`docs/MIGRATION-0.9.10.md`):

* `parse` reads `u256` and `i256` over their whole range, with the statuses the narrower integers
  give; `Overflow` only past the real top or below the real bottom (`[TXT-10]`, `[TYP-42]`). D-528
  had refused them (`E2020`).
* The known-numbers rule replaces the visible-numbers rule: `enumerate`'s numbers, and `len` and
  `count`, are in the smallest type that holds every value, read from the numbers the compiler
  knows: numbers written as numbers, names set once and never changed, a loop's counter,
  arithmetic on known numbers, and lengths (`[STD-19]`, `[STD-26]`). Before, a variable counted as
  any value of its type.
* `enumerate` numbers in a type that holds every number it can give, stepping up from its
  iterator's `Position` where the numbers could pass that type's top, so no number of an
  `enumerate` overflows; a counted loop computes its numbers with no check (`[STD-19]`,
  `[CTL-3b]`). This replaces G8-4 decision F (a number past `int`'s top panicked) and ODR-091's
  backwards panic.
* A whole number converts by itself to another whole-number type where the compiler knows every
  value it can have fits, at a coercion site and in an operator (`[TYP-5]` rule 12, `[TYP-4]`):
  after `n = 5`, `y: u64 = n` is accepted. Floats, `bool` and `char` keep their rules.
* In a function written for any type, where the type of `enumerate`'s numbers depends on the item
  type, that type and the lines using the numbers are checked for each use, which numbers by what
  it knows; a use a line does not fit is refused there, naming it (`[TYP-17]`, `[STD-19]`).

Generated: every rule whose text differs from 0.9.10_Hardened_1.

| Rule | 0.9.10_Hardened_1 → 0.9.10_Hardened_2 |
|---|---|
| `[CTL-3b]` | changed |
| `[STD-19]` | changed |
| `[STD-26]` | changed |
| `[TXT-10]` | changed |
| `[TYP-4]` | changed |
| `[TYP-5]` | changed |
| `[TYP-17]` | changed |
| `[TYP-42]` | changed |

## H.8 Changes in 0.9.10_Hardened_3 (from 0.9.10_Hardened_2)

**0.9.10_Hardened_3** records ODR-099, the owner's choice of 2026-10-11 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-151). It changes what programs are accepted:

* Where a program turns a callable type into `extern "C" fn`, a function whose value has that type
  returns no raw pointer into a parameter it borrows by value: C passes the argument as a copy that
  ends when the call returns (`[FFI-21a]`, `E5042`). The same function called from Ember borrows
  the caller's value, as before.

Generated: every rule whose text differs from 0.9.10_Hardened_2.

| Rule | 0.9.10_Hardened_2 → 0.9.10_Hardened_3 |
|---|---|
| `[FFI-21a]` | added |

## H.9 Changes in 0.9.10_Hardened_4 (from 0.9.10_Hardened_3)

**0.9.10_Hardened_4** records ODR-100, the owner's choice of 2026-10-11 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-153). It changes what programs are accepted:

* The iterator adapters take over the callable and the other iterator they keep (`owned`); a named
  iterator given to `zip` or `chain` is moved (`[STD-19]`).
* A lambda written at an adapter keeps the captures `[CLO-2]` infers; the adapter lives no longer
  than they do. Only a consumed callable parameter whose captured state must carry the static region
  (`thread.spawn`, `jobs.submit`, registries) makes a lambda written there capture by move
  (`[CLO-15]`, `[CLO-4]`, `[CLO-7]`).

Generated: every rule whose text differs from 0.9.10_Hardened_3.

| Rule | 0.9.10_Hardened_3 → 0.9.10_Hardened_4 |
|---|---|
| `[CLO-4]` | changed |
| `[CLO-7]` | changed |
| `[CLO-15]` | changed |
| `[STD-19]` | changed |

## H.10 Changes in 0.9.10_Hardened_5 (from 0.9.10_Hardened_4)

**0.9.10_Hardened_5** records ODR-101, ruled under the owner's delegation on 2026-10-11
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-157). It changes what programs are accepted:

* The entry module's `main` in a form `[FN-8]` does not list is `E2183`, reported at its name: another
  parameter than one `Array[String]` (any name, any mode), more than one, another return type than
  `void` or `Result[void, E]`, or type parameters. An `E` that is not `Display` is `E2040`, an unmet
  bound. Such a `main` used to be accepted and run wrongly (D-551).

Generated: every rule whose text differs from 0.9.10_Hardened_4.

| Rule | 0.9.10_Hardened_4 → 0.9.10_Hardened_5 |
|---|---|
| `[FN-8]` | changed |

## H.11 Changes in 0.9.10_Hardened_6 (from 0.9.10_Hardened_5)

**0.9.10_Hardened_6** records ODR-102, ruled under the owner's delegation on 2026-10-11 while building
owned callable values (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-159, D-554). It changes what
programs are accepted:

* A function, or a lambda that captures nothing, used as a value is a plain `Copy` value; it becomes
  an owned callable value where one is wanted. An owned callable value moves and drops its state
  (`[CLO-3]`).
* An owned callable value given to a borrowed parameter is lent mutably: from a place that is not
  mutable it is `E3023`, and the same value given twice to one call is `E3022` (`[CLO-2]`, `[CLO-3]`).
* An owned callable value is neither `Send` nor `Sync`; two are equal when they call the same function
  with the same state (`[CLO-10]`). Converting one to `extern "C" fn` reads its C entry without moving
  it, and panics if it holds captured state (`[CLO-3]`).
* A lambda copies what it only reads where it becomes an owned callable value; "assigned after" is an
  assignment later in the body or anywhere in a loop around it (`[CLO-13]`).

Generated: every rule whose text differs from 0.9.10_Hardened_5.

| Rule | 0.9.10_Hardened_5 → 0.9.10_Hardened_6 |
|---|---|
| `[CLO-3]` | changed |
| `[CLO-10]` | changed |
| `[CLO-13]` | changed |

## H.12 Changes in 0.9.10_Hardened_7 (from 0.9.10_Hardened_6)

**0.9.10_Hardened_7** records ODR-103, ruled under the owner's delegation on 2026-10-12
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-161). It changes a diagnostic's code only:

* A lambda that borrows, stored where no view may go (an owned callable value, a list, a class
  field) or sent to a thread, is `E3026`, shape B9 (a closure outlives what it captures), with the
  help `owned fn` or `thread.scope()`; it was `E3063`, which stays for every other view
  (`[CLO-3]`, `[CLO-4]`, `[THR-10]`).

Generated: every rule whose text differs from 0.9.10_Hardened_6.

| Rule | 0.9.10_Hardened_6 → 0.9.10_Hardened_7 |
|---|---|
| `[CLO-3]` | changed |
| `[THR-10]` | changed |

## H.13 Changes in 0.9.10_Hardened_8 (from 0.9.10_Hardened_7)

**0.9.10_Hardened_8** records ODR-104, ruled under the owner's delegation on 2026-10-12
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-165):

* A `return` from a class `init`, like the end of its body, hands the object to its caller, so every
  field without a default is assigned on that path (`E2100`) (`[CLS-2]`).
* `mut self` is accepted in a class `init` and means the same as `self`, as for a struct (`[CLS-2]`,
  `[STR-6]`).

It also corrects the overview's `Emitter` example, which moved a borrowed `String` into a field
(`[FN-1]`): its parameter is `owned name: String`.

Generated: every rule whose text differs from 0.9.10_Hardened_7.

| Rule | 0.9.10_Hardened_7 → 0.9.10_Hardened_8 |
|---|---|
| `[CLS-2]` | changed |

## H.14 Changes in 0.9.10_Hardened_9 (from 0.9.10_Hardened_8)

**0.9.10_Hardened_9** records ODR-105, ruled under the owner's delegation on 2026-10-12
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-166). It gives a diagnostic a code only:

* A `let` field assigned outside its `init` is `E2104` (`[CLS-9]`); the code table gave the rule
  none.

Generated: every rule whose text differs from 0.9.10_Hardened_8.

| Rule | 0.9.10_Hardened_8 → 0.9.10_Hardened_9 |
|---|---|
| `[CLS-9]` | changed |

## H.15 Changes in 0.9.10_Hardened_10 (from 0.9.10_Hardened_9)

**0.9.10_Hardened_10** records ODR-106 to ODR-112, ruled under the owner's delegation on 2026-10-16
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-191):

* `is` is a method name after `fn` in a type's body, `.` and `?.`, as `not` is; a function standing
  alone may take neither (`[LEX-15]`, ODR-106).
* Optional chaining is `[EXP-10]`: `None` propagates, an `Option` member is the result, a field that
  is not `Copy` is lent (ODR-107).
* What a declaration is, and what a type with no row of IX.0's table is reported as, in
  `[SEL-1]` (ODR-108).
* A `Cell` shows a copy of its value; a non-`Copy` value with its own `Display` or `Debug` is not
  shown from one (`[TYP-39]`, ODR-109).
* The keys of `[c]`, `[link]`, `[jobs]`, `[realtime]` and `[ffi]`, in `[MAN-8]` (ODR-110).
* The marker interfaces are `std.core`'s and usable as bounds; `Send` and `Sync` are written by
  hand with `unsafe extend` (`[TYP-43]`, the grammar's `extend_decl`, ODR-111).
* An extension over a parameter standing alone may have blanket parameters, taken at a call from
  the receiver's one instance of the interface (`[GRM-34]`, ODR-112).

Generated: every rule whose text differs from 0.9.10_Hardened_9.

| Rule | 0.9.10_Hardened_9 → 0.9.10_Hardened_10 |
|---|---|
| `[EXP-10]` | added |
| `[GRM-34]` | changed |
| `[LEX-15]` | changed |
| `[MAN-8]` | changed |
| `[SEL-1]` | changed |
| `[TYP-39]` | changed |
| `[TYP-43]` | added |

## H.16 Changes in 0.9.10_Hardened_11 (from 0.9.10_Hardened_10)

**0.9.10_Hardened_11** records ODR-113 to ODR-115, ruled by the owner on 2026-10-16
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-198):

* The example of a type with neither printer is a `RefCell`: a class handle always has `Debug`
  (`[STD-9]`, `[TYP-36]`, ODR-113).
* A moved-from place, read, moved again or borrowed, is `E3040`; `E3050` is a place never assigned on
  some path (`[OWN-3]`, `[BRW-7]`, ODR-114).
* `unsafe fn(…) -> R` is a callable type whose call needs `unsafe`; an `unsafe fn` coerces only to one
  (`[FN-6]`, `[UNS-1]`, the grammar's `fn_type`, ODR-115).

Generated: every rule whose text differs from 0.9.10_Hardened_10.

| Rule | 0.9.10_Hardened_10 → 0.9.10_Hardened_11 |
|---|---|
| `[BRW-7]` | changed |
| `[FN-6]` | changed |
| `[OWN-3]` | changed |
| `[STD-9]` | changed |
| `[UNS-1]` | changed |

## H.17 Changes in 0.9.10_Hardened_12 (from 0.9.10_Hardened_11)

**0.9.10_Hardened_12** records ODR-116, ruled by the owner on 2026-10-16 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-200):

* A `MutSpan[T]` converts to `Span[T]` where one is expected, a shared reborrow of the same elements.
  `[TYP-5]` gives its list as complete and lacked it, while Part XII.1's `integrate(ps.pos, ps.vel, …)`
  passes a mutable column to a `Span` parameter (`[TYP-5]` rule 6, `[SPN-3]`, `[SOA-2]`, ODR-116).

Generated: every rule whose text differs from 0.9.10_Hardened_11.

| Rule | 0.9.10_Hardened_11 → 0.9.10_Hardened_12 |
|---|---|
| `[SOA-2]` | changed |
| `[SPN-3]` | changed |
| `[TYP-5]` | changed |

## H.18 Changes in 0.9.10_Hardened_13 (from 0.9.10_Hardened_12)

**0.9.10_Hardened_13** records ODR-117 to ODR-126, ruled by the owner on 2026-10-19 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-204, `tasks/85/owner-answers.md`):

* The profiles are `debug`, `development` and `release`. `release` is the build users get (formerly
  `shipping`: optimisation 3, stripped, full LTO, no backtrace); `development` is optimised and still
  debuggable (formerly `release`: optimisation 2, line information, symbols, thin LTO). Every rule,
  table and example names them so (`[PRF-3]`, ODR-117).
* A `release` build ignores `[build] reload` with a note, and `E9033` is retired; hot reload is a mode of
  `debug` and `development` (`[MAN-7]`, `[PRF-2]`, ODR-118).
* In a hot-reload build a reloadable function is never inlined, whatever `@inline` says (`[CG-C-3a]`,
  `[HR-9a]`, ODR-119).
* A migrated instance's old version runs no user `drop`: the instance lives on in its new form
  (`[HR-2a]`, ODR-120).
* `ReloadError`'s parts have types: `type_name: String` (the name `type` is reserved), `reason`,
  `message` and `error` are `String`, `instance` is `u64` (`[HR-38]`, ODR-121).
* A foreign type names its destructor with `@ffi(destroys(f))` (`[FFI-36a]`, ODR-122).
* One live object adopted twice through one local is `E5068` at compile time (`[FFI-36b]`, ODR-123).
* A status enum is marked `@ffi(status=E, ok=X, error=N)`, which names its error type (`[ERR-6]`,
  `[FFI-16]` agreed, ODR-124).
* An `inout_count` function with a result of its own returns that result first, then each pointer
  output (`[FFI-11b]`, ODR-125).
* Handles inside tuples, `Box` contents and closure captures are rewritten when an instance moves
  (`[HR-15]`, ODR-126).

Generated: every rule whose text differs from 0.9.10_Hardened_12.

| Rule | 0.9.10_Hardened_12 → 0.9.10_Hardened_13 |
|---|---|
| `[BUD-2]` | changed |
| `[CG-C-3a]` | changed |
| `[ERR-6]` | changed |
| `[FFI-11]` | changed |
| `[FFI-11b]` | changed |
| `[FFI-16]` | changed |
| `[FFI-36a]` | changed |
| `[FFI-36b]` | changed |
| `[HR-2a]` | changed |
| `[HR-9a]` | changed |
| `[HR-15]` | changed |
| `[HR-27]` | changed |
| `[HR-35]` | changed |
| `[HR-38]` | changed |
| `[MAN-7]` | changed |
| `[PAN-1]` | changed |
| `[PRF-2]` | changed |
| `[PRF-3]` | changed |
| `[RT-4]` | changed |

## H.19 Changes in 0.9.10_Hardened_14 (from 0.9.10_Hardened_13)

**0.9.10_Hardened_14** records ODR-127 to ODR-131, ruled by the owner on 2026-10-19 on helper C's
questions (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-207, `tasks/85/owner-answers.md`):

* A dependency `{ git, version }` resolves against its repository's version tags; a requirement with
  no source waits for a registry (`[VER-5]`, `[MAN-8]`, ODR-127).
* A field of an `@export_table` holds the function its default names, else the function of its own
  name (`[FFI-26]`, ODR-128).
* An `@export_table` is exported as `Name_vN`, its protocol version in its own name (`[FFI-26]`,
  `[ABI-1]`, ODR-129).
* A table's field takes its function's thread contract (`[FFI-33c]`, ODR-130).
* A `cdylib` exports the marker `ember_module_protocol_vN`, which its host looks up before calling into
  it (`[FFI-28]`, `[FFI-31]`, `[ABI-1]`, ODR-131).

Generated: every rule whose text differs from 0.9.10_Hardened_13.

| Rule | 0.9.10_Hardened_13 → 0.9.10_Hardened_14 |
|---|---|
| `[ABI-1]` | changed |
| `[FFI-26]` | changed |
| `[FFI-28]` | changed |
| `[FFI-31]` | changed |
| `[FFI-33c]` | changed |
| `[MAN-8]` | changed |
| `[VER-5]` | changed |

## H.20 Changes in 0.9.10_Hardened_15 (from 0.9.10_Hardened_14)

**0.9.10_Hardened_15** records ODR-132, ruled under the delegation on 2026-10-19 after the owner chose a
typed pointer (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-208):

* `tok.ptr()` is typed by what the token holds: a `std.ffi.ObjectPtr[C]` for a class `C`, a `*mut U` for
  a `Box[U]`, a `*mut u8` for a `Box[dyn I]` (`[FFI-23]`, ODR-132).

Generated: every rule whose text differs from 0.9.10_Hardened_14.

| Rule | 0.9.10_Hardened_14 → 0.9.10_Hardened_15 |
|---|---|
| `[FFI-23]` | changed |

## H.21 Changes in 0.9.10_Hardened_16 (from 0.9.10_Hardened_15)

**0.9.10_Hardened_16** records ODR-133, the owner's A1 ("fix data at checkpoint"), built under the delegation with the constraint the old code below a checkpoint sets (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-209):

* At `std.hot.checkpoint()` a layout change applies when no frame below it depends on the changed type after its call returns; one a frame depends on is refused, naming the function and the line (`[HR-3]`, `[HR-3b]` new, ODR-133).

Generated: every rule whose text differs from 0.9.10_Hardened_15.

| Rule | 0.9.10_Hardened_15 → 0.9.10_Hardened_16 |
|---|---|
| `[HR-3]` | changed |
| `[HR-3b]` | added |

## H.22 Changes in 0.9.10_Hardened_17 (from 0.9.10_Hardened_16)

**0.9.10_Hardened_17** records ODR-134, ruled under the delegation by the owner's standing rule for a keyword that collides with a name the standard library must declare (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-212):

* `open` is a contextual keyword, a keyword only before `class`, as `abstract` is, so `std.fs.File` can have the `open` that `[STD-22]` names; the reserved set has 47 words (`[LEX-15]`, ODR-134).

Generated: every rule whose text differs from 0.9.10_Hardened_16.

| Rule | 0.9.10_Hardened_16 → 0.9.10_Hardened_17 |
|---|---|
| `[LEX-15]` | changed |

## H.23 Changes in 0.9.10_Hardened_18 (from 0.9.10_Hardened_17)

**0.9.10_Hardened_18** records ODR-135, ruled under the delegation while `[MONO-3]` was built (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-214):

* `[build] max_instantiations` is a ceiling on the package's own generics: the standard library's (the prelude alone instantiates `NonZero` twelve times) are not the package's to budget (`[MONO-3]`, ODR-135).

Generated: every rule whose text differs from 0.9.10_Hardened_17.

| Rule | 0.9.10_Hardened_17 → 0.9.10_Hardened_18 |
|---|---|
| `[MONO-3]` | changed |

## H.24 Changes in 0.9.10_Hardened_19 (from 0.9.10_Hardened_18)

**0.9.10_Hardened_19** records ODR-136, ruled under the delegation while the rest of `[DET-2]` was built (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-216):

* Reading padding bytes is `Nondet` where the compiler sees them become readable: a pointer to a type with padding reinterpreted as a pointer to another type. Reading uninitialised memory is not `Nondet` but undefined, outside Safe Ember, an `unsafe` obligation (`[DET-2]`, `[PHIL-10]`, ODR-136).

Generated: every rule whose text differs from 0.9.10_Hardened_18.

| Rule | 0.9.10_Hardened_18 → 0.9.10_Hardened_19 |
|---|---|
| `[DET-2]` | changed |

## H.25 Changes in 0.9.10_Hardened_20 (from 0.9.10_Hardened_19)

**0.9.10_Hardened_20** records ODR-137, ruled under the delegation while `[FFI-16]` and `[ERR-6]` were built on hand declarations (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-219):

* `@non_exhaustive` is a C enum's: a unit-only enum with its `@repr`. Any value of its integer type is a value of the enum, so a `match` needs `_`, `Option` of it keeps a tag, `Debug` shows an unnamed value as `E(9)` and `Display` as `9`; it crosses the C boundary as its integer, where an exhaustive enum does not (`[FFI-8]`).
* On a hand declaration (ODR-073), `param(p, out, one)` is written on `mut p: T`: C gets a `T*`, `p` leaves the callable signature and the call returns the value, several in a tuple in C's order, after C's own result when it has one (`[FFI-11]`, `[FFI-11b]`).
* `@ffi(status=E, ok=X, error=N)` is written on the C enum `E` itself, `X` one of its variants; `N` is declared with the enum's visibility, `Copy`, with `Display` the code's `Debug`. `result(status)` makes the result `Result[T, N]`, `T` the outputs: `void`, the one output's type, or a tuple of several; the outputs are not read on an error code (`[FFI-16]`, `[ERR-6]`).

Generated: every rule whose text differs from 0.9.10_Hardened_19.

| Rule | 0.9.10_Hardened_19 → 0.9.10_Hardened_20 |
|---|---|


## H.26 Changes in 0.9.10_Hardened_21 (from 0.9.10_Hardened_20)

**0.9.10_Hardened_21** records ODR-138..ODR-143, ruled under the delegation on questions helpers L (layout attributes) and M (the ECS) raised (`tasks/90/owner-questions.md`, `docs/DECISIONS.md` ADR-225):

* `@packed` and `@align(N)` are not written on one type, and a `@packed` type holds no field whose type states an alignment with `@align`: `E0104` (ODR-138).
* `@repr(T)` is an enum's discriminant type only; on a struct it is `E0104`, and the attribute table gives it its own row (ODR-139).
* `E2170` is a reference to an *unaligned* field of a packed type, as `[LAY-2]`'s text says; the code table now says so (ODR-140).
* The ECS library's shape: `run(system, arg)` passes one argument (a tuple for several, `()` for none); `Not[D]`'s item is `()`; `Entity` is a query parameter yielding the entity and `Option[P]` applies to any parameter; and Part XII.3 names its two pieces of compiler support (the conversion of a function to a `System`, `run_parallel`'s literal checked for `E7020`) (ODR-141).
* A query with no required component iterates every live entity, in the world's order (ODR-142).
* Under `[EXC-5]`, a shared method called on `self` inside a `mut self` method is lent the method's access as a read: its reads are not checked again, a write to the object under it is (ODR-143).

Generated: every rule whose text differs from 0.9.10_Hardened_20.

| Rule | 0.9.10_Hardened_20 → 0.9.10_Hardened_21 |
|---|---|
| `[ECS-3]` | changed |
| `[ECS-4]` | changed |
| `[EXC-5]` | changed |
| `[LAY-2]` | changed |

## H.27 Changes in 0.9.10_Hardened_22 (from 0.9.10_Hardened_21)

**0.9.10_Hardened_22** records ODR-144, ruled under the delegation while `[TYP-32]` was built (`docs/DECISIONS.md` ADR-229):

* A `some` type is a function's or a method's return type, never the return type of a method an interface declares (a declaration or a default): each implementation would choose its own type, which an associated type bounded by the interface already states. Such a `some` is `E2260` (ODR-144).

Generated: every rule whose text differs from 0.9.10_Hardened_21.

| Rule | 0.9.10_Hardened_21 → 0.9.10_Hardened_22 |
|---|---|
| `[GRM-31]` | changed |
| `[TYP-32]` | changed |

## H.28 Changes in 0.9.10_Hardened_23 (from 0.9.10_Hardened_22)

**0.9.10_Hardened_23** records ODR-145..ODR-149, ruled under the delegation on the questions helper N raised building the C header importer (`tasks/90/owner-questions.md`, `docs/DECISIONS.md` ADR-231):

* A raw pointer made from an integer is a compile-time value (an address, read by nothing), so a `const` of pointer type may hold one; a null-pointer or pointer-cast macro is imported as such a `const` (ODR-145).
* An object-like macro that expands to nothing declares nothing and is not reported (ODR-146).
* An overlay entry marked `@ffi(macro_fn)` writes Ember types for the macro's parameters and result (ODR-147).
* `ember bind --init <header>` writes `<stem>.overlay.em` beside the header (ODR-148).
* `overlay` takes one file or a list applied first to last; a later file's `override` entry replaces an earlier one, any other second entry for one name is `E5010` (ODR-149).

Generated: every rule whose text differs from 0.9.10_Hardened_22.

| Rule | 0.9.10_Hardened_22 → 0.9.10_Hardened_23 |
|---|---|
| `[CLI-5]` | changed |
| `[CT-2]` | changed |
| `[FFI-6a]` | changed |
| `[FFI-6b]` | changed |
| `[FFI-30c]` | changed |
| `[GRM-35]` | changed |

## H.29 Changes in 0.9.10_Hardened_24 (from 0.9.10_Hardened_23)

**0.9.10_Hardened_24** records ODR-150, ruled under the delegation when `[TST-7]`'s check was built: every ```` ```ember ```` block is now checked with `ember check`, and five examples did not say what the rules do (`docs/DECISIONS.md` ADR-232):

* Part XII's `integrate` and `scale` write through a `MutSpan` parameter, which `[FN-1]` makes `mut` (a borrowed parameter is not assigned through); XI.5's `step` likewise.
* XI.5's job example and XVI.3's Vulkan import name items the document does not declare (`integrate`, `age`, a header and an overlay file): they are `ember,fragment`.
* XVII.5's compile-fail test expects the diagnostic the compiler gives, as `[TST-1]` asserts it.
* VII.7's `heal_all`, IX.4's raw pointer reads and Part XIII's `?` into `AnyError` were the compiler's to fix (D-747, D-748, D-749), not the examples'.

Generated: every rule whose text differs from 0.9.10_Hardened_23.

| Rule | 0.9.10_Hardened_23 → 0.9.10_Hardened_24 |
|---|---|


## H.30 Changes in 0.9.10_Hardened_25 (from 0.9.10_Hardened_24)

**0.9.10_Hardened_25** records ODR-151 and ODR-152, ruled under the delegation when `[CONF-1]`'s declaration and `[COST-5]`'s check were built (`docs/DECISIONS.md` ADR-236, ADR-237):

* A compiler declares (`[CONF-1]`) its profile as `ember --version`'s `profile:` line (`none`, `core`, `systems`, `native` or `dynamic`), and each profile holds the one before it.
* Ember Core (`[CONF-2]`) named XV's core and alloc layers, which are not sections of Part XV: they are XV.1 and XV.3–XV.6. `[CONF-3]` takes XV.2 and XV.7, the io layer's console, files, time and processes, which no profile named.
* Ember Dynamic (`[CONF-5]`) adds Annex B to Ember Native.
* The cost table (`[COST-3]`) gains five rows, the costs `ember inspect --cost` already reported without one: `Virtual method call`, `Downcast check`, `assert_disjoint` comparison, `Allocation` and `contains`. `[COST-5]`'s check found them.

Generated: every rule whose text differs from 0.9.10_Hardened_24.

| Rule | 0.9.10_Hardened_24 → 0.9.10_Hardened_25 |
|---|---|
| `[CONF-1]` | changed |
| `[CONF-2]` | changed |
| `[CONF-3]` | changed |
| `[CONF-5]` | changed |
| `[COST-3]` | changed |

## H.31 Changes in 0.9.10_Hardened_26 (from 0.9.10_Hardened_25)

**0.9.10_Hardened_26** records ODR-153 and ODR-154, ruled with the owner's approval (2026-10-20) when D-755 showed that a small loop's speed depends on where its code lies and on which processor runs it (`docs/DECISIONS.md` ADR-239):

* The performance gate (`[BEN-1]`) builds each side at several code layouts, the same for both, and compares each side at its fastest. The C program of `w1_sum_loop` ran 31.9 ms at one layout and 55.3 ms at another on CI's runner, with no change to its code: a single build per side measured where the linker put the loop as much as the program.
* The gate's verdict of record (`[TST-28]`) is the one measured on the recorded reference machine before a change is published. CI's shared runners were AMD EPYC processors of three generations, changing from run to run, and each failed a different benchmark; there CI reports the gate's results (`ember bench --compare --report`) without failing on them.

Generated: every rule whose text differs from 0.9.10_Hardened_25.

| Rule | 0.9.10_Hardened_25 → 0.9.10_Hardened_26 |
|---|---|
| `[BEN-1]` | changed |
| `[TST-28]` | changed |

## H.32 Changes in 0.9.10_Hardened_27 (from 0.9.10_Hardened_26)

**0.9.10_Hardened_27** records ODR-155, ruled under the delegation when Annex A was made a fixture that compiles (`[TST-6]`, `docs/DECISIONS.md` ADR-246). Compiling the annex found five implementation defects, fixed (ADR-244), and two places where the annex broke the specification's own rules:

* The map comprehension `index = {name: i for i, name in NAMES.iter().enumerate()}` named no type, so its keys were `String` (`[TYP-38]`: a map written with no type owns its text), and a `str` that is not a literal does not convert to one (`[TYP-5]` rule 4). It now says `index: Map[str, int] = …`, the static `str`s `NAMES` holds.
* The `unsafe:` block carried a comment, not the safety note `[UNS-8]` requires of every one (`W3012`). It now carries `# SAFETY: …`.

Generated: every rule whose text differs from 0.9.10_Hardened_26.

| Rule | 0.9.10_Hardened_26 → 0.9.10_Hardened_27 |
|---|---|


## H.33 Changes in 0.9.10_Hardened_28 (from 0.9.10_Hardened_27)

**0.9.10_Hardened_28** records ODR-156, the owner's answer to helper N4's question on `[FFI-39c]`, and ODR-157, ruled under the delegation when `[LAY-2]`'s last two layouts were built (`docs/DECISIONS.md` ADR-249):

* A C++ parameter that keeps the pointer it is given takes a `Retained` token the program holds: `tok = Retained.pin(self)` upcasts `self` to the C++ base and the call is given `tok`; passing `self` itself there is `E5060`, and a parameter borrowed for the call alone takes `self` with no token (ODR-156, the owner's option (a)).
* A SIMD vector is aligned to its size (`[SIMD-1]`), an alignment stated as `@align` states one, so a `@packed` type holding one is `E0104`, as ODR-138 rules for `@align` types: no padding and that alignment cannot both hold, and MSVC keeps the alignment under `#pragma pack` (ODR-157).

Generated: every rule whose text differs from 0.9.10_Hardened_27.

| Rule | 0.9.10_Hardened_27 → 0.9.10_Hardened_28 |
|---|---|
| `[FFI-39c]` | changed |
| `[LAY-2]` | changed |

## H.34 Changes in 0.9.10_Hardened_29 (from 0.9.10_Hardened_28)

**0.9.10_Hardened_29** records ODR-158, ruled under the delegation when reflection was built (`[RFL-1]`..`[RFL-3]`, `[RT-3]`, `docs/DECISIONS.md` ADR-250). The rules name `TypeDesc`, `TypeInfo` and their descriptors without declaring them; §XIV.2 now says what `std.ser` declares:

* A description owns its text and arrays (`String`, `Array`), so it is no view and lives in a `static`, not a `const`; `TypeInfo` is `TypeDesc`.
* A field names its type by name, since a description of each field's type would not end for a recursive type; a class field's offset is counted from the end of the runtime's object header.
* An attribute is its name and `(name, AttrValue)` pairs: a user attribute's arguments named by the fields they set, a built-in's positional ones by position, a name argument as its text; an `@attribute` struct's fields are `int`, `float`, `str`, `String`, `bool` or `char` (`E0104`).
* `@reflect` implements `std.ser.Reflect`, which `type_info[T]()` asks for (`E2040`); a class derived from a `@reflect` class is `@reflect`; `reflect[T]()` where compile time does not evaluate is `E6010`.

Generated: every rule whose text differs from 0.9.10_Hardened_28.

| Rule | 0.9.10_Hardened_28 → 0.9.10_Hardened_29 |
|---|---|


## H.35 Changes in 0.9.10_Hardened_30 (from 0.9.10_Hardened_29)

**0.9.10_Hardened_30** records ODR-159, ruled under the delegation when serialisation was built (`[SER-1]`, `[SER-2]`, `[RNG-10a]`, `docs/DECISIONS.md` ADR-251), and the owner's answers of 2026-10-21 on three Annex C questions (ADR-252), whose rule texts change when they are built:

* `std.ser`'s `Writer` and `Reader` shapes, a struct's version (the highest `@ser(since=N)`), the JSON form (`"$version"`, `"$variant"`, `null`) and the self-describing binary form are as §XIV.2's paragraph on serialisation says (ODR-159).
* For `[FFI-40a]`: a `const` method of a class with a `mutable` member takes `self` when its body, visible in the header, writes no field, and `mut self` otherwise or when its body is not visible; an overlay `class C:` section restates a method as an Ember method line, which wins (ODR-160, the owner).
* For `[FFI-41]`: a static data member, and a namespace variable, reads and writes as a global (`cpp.engine.Grid.made += 1`), through the address thunk (ODR-161, the owner).
* For `[BLD-FFI-5a]`, `[FFI-31b]`: an `@export` function may hand C++ an object of a `@sync` class, which the `--cpp` header presents as a class of the same name with its `pub` methods, copies sharing the object and the last one freeing it; a class without `@sync` is refused (ODR-162, the owner).

Generated: every rule whose text differs from 0.9.10_Hardened_29.

| Rule | 0.9.10_Hardened_29 → 0.9.10_Hardened_30 |
|---|---|


## H.36 Changes in 0.9.10_Hardened_31 (from 0.9.10_Hardened_30)

**0.9.10_Hardened_31** builds the owner's answers ODR-160 (`[FFI-40a]`) and ODR-161 (`[FFI-41]`), whose rule texts now say what is built, and records two rulings the owner made on 2026-10-21: ODR-163, on variables holding a pointer, and ODR-164, which reverses ODR-021 (`docs/DECISIONS.md` ADR-253):

* For `[FFI-40a]`: the importer reads each `const` method's body when the class has a `mutable` member, its own or a base's; a method line of an overlay's `class C:` section (`[GRM-35]`'s `overlay_class`) restates the method, and its receiver wins; a line that differs from the import's method is `E5010` (ODR-160, the owner).
* For `[FFI-41]`: a static data member, and a variable of a namespace, is a foreign static read and written as a global, through the thunk giving its address (ODR-161, the owner); every variable of a C++ header goes through its thunk, an `inline` one too.
* For `[STA-1]`, `[FFI-41]` and Part XVI's table: a foreign static read only in `unsafe` need not be `Sync`, so a C or C++ variable holding a pointer imports as any other; one read without `unsafe` under `@ffi(immutable)` must be `Sync` (ODR-163, the owner).
* For `[TYP-29]`: float `//` is `floor(a / b)` and `%` is `a - b * floor(a / b)`, each step rounded, so `1.0 // 0.1 == 10.0` and `1.0 % 0.1 == 0.0` (ODR-164, the owner, reversing ODR-021).

Generated: every rule whose text differs from 0.9.10_Hardened_30.

| Rule | 0.9.10_Hardened_30 → 0.9.10_Hardened_31 |
|---|---|
| `[FFI-8]` | changed |
| `[FFI-40a]` | changed |
| `[FFI-41]` | changed |
| `[FFI-42a]` | changed |
| `[GRM-35]` | changed |
| `[STA-1]` | changed |
| `[TYP-29]` | changed |

## H.37 Changes in 0.9.10_Hardened_32 (from 0.9.10_Hardened_31)

**0.9.10_Hardened_32** builds `[CXX-1]`'s corpus (nlohmann/json 3.11.3, `tests/cxx-corpus`) and records three rulings made under the restored delegation, for the owner's review (`docs/DECISIONS.md` ADR-254):

* For `[FFI-41]`: an inline namespace adds nothing to a name, as in C++ (`nlohmann::json_abi_v3_11_3::json` is `cpp.nlohmann.json`), and `instantiate=[…]` names its members so (ODR-165, delegated).
* For `[FFI-32]` and Annex C's import table: a function returning by value a class Ember holds behind a pointer gives a `ForeignBox[T]` owning a copy moved to the heap, freed by the class's destructor, as a constructor's does; an abstract class, or one without a public destructor, stays refused (ODR-166, delegated).
* For `[CXX-1]`: the corpus holds nlohmann/json 3.11.3; more real headers are added as the owner approves their downloads (ODR-167, delegated).
* For `[CLI-6]`'s synopsis: `ember bind` also takes `--outcomes` (each declaration's outcome, `[CXX-1]`), `--emit-cpp [--out <file>]` (`[FFI-50]`), `--flag <flag>` and `--instantiate <template-id>`.

Generated: every rule whose text differs from 0.9.10_Hardened_31.

| Rule | 0.9.10_Hardened_31 → 0.9.10_Hardened_32 |
|---|---|
| `[FFI-32]` | changed |
| `[FFI-41]` | changed |

## H.38 Changes in 0.9.10_Hardened_33 (from 0.9.10_Hardened_32)

**0.9.10_Hardened_33** records rulings made under the restored delegation, for the owner's review:

* For `[STD-18]`: `format`'s spec is a string literal, checked against the value's type where it is written (`E2250`), as an f-string's is (ODR-168, delegated).

Generated: every rule whose text differs from 0.9.10_Hardened_32.

| Rule | 0.9.10_Hardened_32 → 0.9.10_Hardened_33 |
|---|---|
| `[STD-18]` | changed |

## H.39 Changes in 0.9.10_Hardened_34 (from 0.9.10_Hardened_33)

**0.9.10_Hardened_34** records the rulings made under the restored delegation on helpers P to Y's questions and the lead's (ODR-169..ODR-217, ODR-225..ODR-235), for the owner's review, and nine answers the owner gave on 2026-10-22 (ODR-218..ODR-224, ODR-236, ODR-237) (`docs/DECISIONS.md` ADR-256..ADR-267); `[LEX-26]` and `[MNG-6]` are new, and one question stays open for the owner (ODR-238, `[CLO-15]`):

* For `[VER-3]`: each use of a deprecated item outside its own module is `W2192` (use of a deprecated item), naming `since` and `note` (ODR-174, delegated).
* For `[PHIL-14]`: sequence repetition (`s * n`, `[x] * n`) is rejected likewise, with `s.repeat(n)` and `[x for _ in 0..n]` as the fixes (ODR-185, delegated).
* For `[LEX-26]` (new): a literal is written as II.5's grammar gives it, and anything else is `E0009` (malformed literal); a string left unclosed, or an f-string's `{`, is `E0101` (unclosed delimiter or literal) (ODR-228, delegated).
* For III.2's grammar: `interface_decl` takes `unsafe` (`pub unsafe interface Allocator`, ODR-175, delegated).
* For `[GRM-8b]`: an interface named as a type that no class implements is `E2173` (ODR-226, delegated, reading ODR-182 with `[OBJ-2]`).
* For `[GRM-15]`: `owned e` may be the iterable of a comprehension's `for` clause (ODR-236, the owner).
* For `[GRM-17]`: outside brackets, a same-line `:` lambda body ends at the statement's `NEWLINE` (`h = fn(x): total += x`) (ODR-209, delegated).
* For `[GRM-27]`: a comprehension may take its collection over (`[s for s in owned shapes if …]`: the kept items move, the rest are dropped) or copy its items with `.clone()`, the collection staying as it was (ODR-236, the owner).
* For `[TYP-43]`: an `unsafe extend` may implement an `unsafe interface`, which only an `unsafe extend` implements (`E3100` without `unsafe`) (ODR-175, delegated).
* For `[TYP-36]`: a shared `ref T` implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as `T` does (ODR-181, delegated); `Box[T]` is `Clone` when `T` is, and `Box[dyn I]` when `I` requires `Clone` (ODR-237, the owner).
* For `[TYP-22]`: an interface that requires `Clone` is usable as `dyn`; every type implementing it is `Clone`, and its method table copies the value into a new box (ODR-237, the owner).
* For `[MOD-1]`: a module and an item of one name in one scope, an import binding among them, and a type named like a built-in type that the prelude does not let a declaration shadow, are `E1032`; the modules compared are the build's (ODR-201, ODR-216, ODR-217, delegated).
* For `[CLS-1]`, `[OBJ-4]` and `[OPT-1]`: a class object lives on the heap and is made by the runtime allocator except as `[OPT-1]` permits; how many times an embedding host's allocation functions are called is not behaviour (ODR-177, delegated).
* For `[CLO-3]`: a consumed (`owned`) callable parameter is an owned callable value in its function and may be stored (ODR-189, delegated); a lambda given to a `mut` callable parameter is `E3027`, shape B10 (ODR-208, delegated).
* For `[CLO-10]` and `[THR-8]`: an owned callable value is `Send` when everything it captures is `Send`, as a function and a capture-free lambda always are (ODR-218, the owner).
* For `[ECS-4]`: `run_parallel` runs non-conflicting systems at once, on threads; each system and every component its query uses must be `Send` (`E7004`) (ODR-218, the owner).
* For `[LT-1a]`: `@borrows(self.f)` names a view field of the receiver, in any receiver mode, or a field whose type is a type parameter, checked in each instance; the standard library's iterators that read through a shared field declare it, so `for k in m:` and `m.keys()` yield references that borrow the map (ODR-196, ODR-211, ODR-212, delegated).
* For `[LT-39]`: an ended region a run-time selection needs is a loan conflict, `E3021`, not `E3065` (ODR-176, delegated).
* For `[OBJ-2]`: a bare interface name in a type position is a class handle typed by the interface; where no class implements the interface it is `E2173` (P22) (ODR-226, delegated).
* For IX.1's `Box[T]`: `Clone` when `T` is, a new box holding a clone; `Box[dyn I]` when `I` requires `Clone`, through its method table (ODR-237, the owner).
* For `[ALC-2]`: `std.mem`'s `pub unsafe interface Allocator` has `allocate`, `deallocate` and `reallocate` over `Layout` (`size`, `align`), and `Global` is the runtime's allocator (ODR-175, delegated).
* For `[ERR-8]`: `AnyError.msg(owned text: String)` is an error whose text is `text` (ODR-180, ODR-230, delegated).
* For `[STD-26]`: `sorted` over an iterator of references gives an `Array` of clones, as `sorted(m)` does (ODR-207, delegated).
* For `[TXT-10]`: `char` has `is_alphabetic`, `is_digit`, `is_alphanumeric` and `is_whitespace` with Python's meaning, and their `is_ascii_*` forms (ODR-183, ODR-229, delegated).
* For `[STD-19]`: the consumers that apply a callable (`any`, `all`, `find`, …) take it over, as the adapters and `for_each` do (ODR-210, delegated).
* For `[FFI-11]`: `borrowed one exclusive` is written `p: ref mut T` as well as `mut p: T`; an opaque type takes only the first (ODR-197, delegated).
* For `[BLD-6]`: `thin` becomes `on` with cl and gcc; the build record is `target/<profile>/inspect/<name>.build.json` with `lto`'s `asked`, `used` and `why`; a static library's objects take no LTO with cl or Clang and fat ones with gcc (ODR-169, ODR-170, ODR-171, delegated).
* For `[BUD-1]` and `[BUD-2]`: the Vulkan-sized header is a generated 20,000 lines in Vulkan's shape, not counted in the package's lines; B6's cold cache is a new, empty global build cache, and B7 and B9 build with the program cache off (ODR-173, delegated).
* For `[BUD-3b]`: the spread is the interquartile range over the median, and both gates apply only below 15 % (ODR-172, delegated).
* For `[TST-9]`: a mark names its shape by its catalogue id (ODR-179, delegated).
* For `[TST-10]`: the published rate counts the catalogue's helps; a help the compiler does not print is a defect (ODR-188, delegated).
* For `[DIA-21]`: each row is a catalogued shape with an id and its codes (ODR-179); the table gains P21..P30 as built (ODR-180, ODR-182, ODR-185, ODR-186, ODR-187, ODR-225, ODR-226); `float(x)` and `int(x)` are answered by the argument's type and `int(s)` and `max(xs)` end in `?` or `.unwrap()` (ODR-184, ODR-231); `x == None` is a habit only where the `Option` has no `Eq` (ODR-227) (delegated).
* For `[DIA-24]`: a type name is a value candidate only before `.` (`Color` in `Color.Red`) (ODR-233, delegated).
* For §XVII.6's shapes: O2's help offers the borrow first (ODR-234); B4's adds a new local for a rebound parameter (ODR-235); N3 is an unknown method or field, `E2072` (ODR-232); N14 is `E1032` (ODR-214) (delegated).
* For `[TOOL-1]`: releases are published on the Ember repository's GitHub Releases, built by the release workflow, the macOS archive built and tested on GitHub's macOS machines (ODR-222, ODR-224, the owner).
* For `[TOOL-2]`: Ember's download carries LLVM's Clang and LLD, which compile the generated C and the C and C++ glue; Windows' system libraries are the one part it does not carry (ODR-223, the owner; the LLVM backend's own plan is the owner's to give).
* For `[CG-C-6]`: MSVC gets no vectorisation pragma; Clang `loop vectorize(enable)` and GCC `ivdep` stay (ODR-190, delegated).
* For `[MNG-1]`: a root package's items have no package component; a method told apart by its interface, a built-in type's path (under `std.core`), a structural owner and `[CLO-3]`'s copies are spelled as built (ODR-204, ODR-206, ODR-215, delegated).
* For `[MNG-3]`: an ASCII `_` that begins what would otherwise read as an escape is transliterated too, as `_u005F_`, a transliterated component has `u` before its length, and C member names are spelled so too (ODR-202, ODR-213, delegated).
* For `[MNG-4]`: the per-type symbols take the type's mangled form without its own `em_` (`em_obj_6Animal`) (ODR-203, delegated).
* For `[MNG-6]` (new) and `[MONO-1]`: the canonical spelling of a generic instance's arguments is written into the spec, so every compiler spells alike and packages deduplicate instances at link time (ODR-205, delegated).
* For `[HR-14]`: the FFI row reads "layout or fields changed on a type used across FFI by value" (ODR-193, delegated).
* For `[HR-20]`: `on_relocate` is `Retained.pin(handle, on_relocate: Option[extern "C" fn(ObjectPtr[C], ObjectPtr[C])])`, called with the old and the new address after commit publishes and before reclaim (ODR-194, delegated).
* For `[HR-23]` and `[HR-41]`: any change to a C or C++ header, or to a file it includes, refuses the reload (ODR-191, ODR-192, delegated).
* For `[HR-24]` and `[HR-41]`: the GPU row moves into `[HR-24]`, which goes after 1.0 with Annex D in `docs/phase-status.json` (ODR-195, delegated).
* For `[FFI-17a]`: a fixed-extent `std::span<T, N>` crosses as `ref [T; N]` / `ref mut [T; N]`; `std::variant` is named by `@ffi(variant=[…])` on a `type` entry; `@ffi(std_function)` makes an owned callable value (ODR-198, ODR-199, ODR-200, delegated).
* For `[FFI-24c]`: the catching C++ calls on `@noalloc` or main-thread paths are listed by `ember inspect --cost`, not `ember bind --report` (ODR-220, the owner).
* For `[FFI-39]` and `[FFI-48]`: an Ember class inherits from a C++ class with no overlay entry, the trampoline passing on exactly the virtuals the class overrides; `@ffi(trampoline, virtuals=[…])` and its unsupported-construct row are gone (ODR-221, the owner).
* For `[FFI-39b]`: the C++ base constructor runs at `super.init(…)` (`[CLS-11]`) (ODR-219, the owner).
* For §XVII.9: `E0009`, `E1032` and `W2192` are added; `E0101` reads "unclosed delimiter or literal" and `E2173` "an argument or type position that is neither a type nor a constant" (ODR-228, ODR-201, ODR-174, ODR-226).
* For Appendix E: rows for `c.isdigit()` and its kin, `xs.sort(key=f)`, `"%d" % n` and `"{}".format(n)`, `s * n` and `[x] * n` (ODR-183, ODR-225).

Generated: every rule whose text differs from 0.9.10_Hardened_33.

| Rule | 0.9.10_Hardened_33 → 0.9.10_Hardened_34 |
|---|---|
| `[ALC-2]` | changed |
| `[BLD-6]` | changed |
| `[BUD-1]` | changed |
| `[BUD-2]` | changed |
| `[BUD-3b]` | changed |
| `[CG-C-6]` | changed |
| `[CLO-3]` | changed |
| `[CLO-10]` | changed |
| `[CLS-1]` | changed |
| `[DIA-21]` | changed |
| `[DIA-24]` | changed |
| `[ECS-4]` | changed |
| `[ERR-8]` | changed |
| `[FFI-11]` | changed |
| `[FFI-17a]` | changed |
| `[FFI-24c]` | changed |
| `[FFI-39]` | changed |
| `[FFI-39b]` | changed |
| `[FFI-48]` | changed |
| `[GRM-8b]` | changed |
| `[GRM-15]` | changed |
| `[GRM-17]` | changed |
| `[GRM-27]` | changed |
| `[HR-14]` | changed |
| `[HR-20]` | changed |
| `[HR-23]` | changed |
| `[HR-24]` | changed |
| `[HR-41]` | changed |
| `[LEX-26]` | added |
| `[LT-1a]` | changed |
| `[LT-39]` | changed |
| `[MNG-1]` | changed |
| `[MNG-3]` | changed |
| `[MNG-4]` | changed |
| `[MNG-6]` | added |
| `[MOD-1]` | changed |
| `[MONO-1]` | changed |
| `[OBJ-2]` | changed |
| `[OBJ-4]` | changed |
| `[OPT-1]` | changed |
| `[PHIL-14]` | changed |
| `[STD-19]` | changed |
| `[STD-26]` | changed |
| `[THR-8]` | changed |
| `[TOOL-1]` | changed |
| `[TOOL-2]` | changed |
| `[TST-9]` | changed |
| `[TST-10]` | changed |
| `[TXT-10]` | changed |
| `[TYP-22]` | changed |
| `[TYP-36]` | changed |
| `[TYP-43]` | changed |
| `[VER-3]` | changed |

## H.40 Changes in 0.9.10_Hardened_35 (from 0.9.10_Hardened_34)

**0.9.10_Hardened_35** records the owner's answers of 2026-10-22 and 2026-10-23 (ODR-240, ODR-241, ODR-242, ODR-244, ODR-259, and ODR-223 again: the LLVM backend is part of 1.0, begun now) and the rulings on helpers AA, AB and AC's smaller questions (ODR-239, ODR-243, ODR-245..ODR-258, for the owner's review; ODR-239 the owner confirmed) (`docs/DECISIONS.md` ADR-273); the owner also confirmed ODR-175, ODR-207, ODR-209, ODR-211, ODR-213 and ODR-215, and ODR-238 (`[CLO-15]`) stays open:

* For `[ALC-2]`: `Global` is the package's configured allocator and `System` the platform's, which a global allocator may wrap (ODR-239).
* For `[STD-19]`: a callable given to the standard library is borrowed; a lambda written at the call is given (ODR-240, the owner, replacing ODR-210).
* For `[TYP-36]`: two boxes compare with `==` as what they hold does; `Box[dyn I]` when `I` requires `Eq`, two kinds never equal (ODR-241, the owner).
* For `[BLD-10]`, `[BUD-1]`, `[BUD-2]`, `[BUD-3]`, `[BUD-3a]`: no fixed times; the figures are a gate by before-and-after comparison on one machine (ODR-242, the owner).
* For `[CT-4]`: a `const` value is the compiler's own evaluator's, never a built C program's (ODR-243).
* For `[BLD-1]`, `[BLD-2]`, `[BLD-4]`: `debug` writes one C file per module, `release` one unit (ODR-244, the owner); the cache serves `ember check` (ODR-257); Ninja is carried, or found on `PATH` (ODR-258).
* For `[FFI-1]`, `[FFI-8]`, `[FFI-11]`, `[FFI-21]`, `[FFI-17a]`, `[FFI-42]`: helper AB's questions (ODR-245..ODR-253).
* For `[STD-14]`: the non-panicking twin is an accessor's (ODR-254).
* For `[THR-8]`, `[ECS-4]`: `dyn I` is `Send`/`Sync` when `I` requires it; a component two parallel systems read is `Sync` (ODR-255).
* For `[IDE-6]`: the identifier interner is the session's (ODR-256).
* For `[PRF-3]`: a `debug` build makes none of Ember's speed rewrites unless `[profiles.debug] rewrites = true` (ODR-259, the owner).
* For `[CG-LL-1]`: the LLVM backend is part of 1.0, built beside the C backend now (ODR-223, the owner, again on 2026-10-23); the progress table counts it as phase 9, inside 1.0.

Generated: every rule whose text differs from 0.9.10_Hardened_34.

| Rule | 0.9.10_Hardened_34 → 0.9.10_Hardened_35 |
|---|---|
| `[ALC-2]` | changed |
| `[BLD-1]` | changed |
| `[BLD-2]` | changed |
| `[BLD-4]` | changed |
| `[BLD-10]` | changed |
| `[BUD-1]` | changed |
| `[BUD-2]` | changed |
| `[BUD-3]` | changed |
| `[BUD-3a]` | changed |
| `[CG-LL-1]` | changed |
| `[CT-4]` | changed |
| `[ECS-4]` | changed |
| `[FFI-1]` | changed |
| `[FFI-8]` | changed |
| `[FFI-11]` | changed |
| `[FFI-17a]` | changed |
| `[FFI-21]` | changed |
| `[FFI-42]` | changed |
| `[IDE-6]` | changed |
| `[PRF-3]` | changed |
| `[STD-14]` | changed |
| `[STD-19]` | changed |
| `[THR-8]` | changed |
| `[TYP-36]` | changed |

## H.41 Changes in 0.9.15_Hardened_1 (from 0.9.10_Hardened_35)

**0.9.15_Hardened_1** is the memory-model rework the owner ordered on 2026-10-23 and decided, verdict by
verdict, from 2026-10-24 to 2026-10-27 (`tasks/96/redesign/JOB.md`, verdicts 1 to 100; the design in
`tasks/96/redesign/run4/v5/`, with its 842 conformance programs; the unsafe-features inventory,
`run4/v5/unsafe-inventory.md`). Its goal, all at once: as fast as C and Rust; as memory-safe as Rust or
safer, checked when compiling; readable like Python, every meaning written in words on the thing it
describes. It replaces the memory model of 0.9.10:

* Ownership is written in words on the value and in its type: `owns` (one owner, and the hand-over, for
  every type, numbers included), `shares` and `shares mut` (several owners, counted), `thread_shared`
  (several owners on several threads), `weakref` (a link that keeps nothing alive, opened with
  `if x exists as y:`), `ref` and `ref mut` (views), `copy of` (a copy of your own); `Box`, `Shared`,
  `Weak`, `RefCell`, `SyncShared`, `ForeignBox`, `owned`, `@sync`, `@borrows` and `mem.forget` are gone.
* What a function changes and reads is worked out from its body, never written (no `changes`, no
  `reads`); every refusal names the chain of calls that makes a change.
* Every rule about who may change or read a place is checked when compiling; the run-time exclusivity
  checks and the run-time index compare are gone. A run-time check is left only where the property is
  data the program computes (an index, an overflow, a `weakref` target, a downcast, a count's limit).
* Loops follow Rust's way: nothing changes a collection through its own name while a loop walks it;
  the loop's `ref mut` handle changes the item it is on.
* A loop of owners is refused unless the compiler proves it is opened again; anything that can break a
  safety promise is possible only inside `unsafe` (`into_raw()`, C function pointers, `unsafe import c`,
  `@unsafe(export(...))`, an `Allocator`'s free); the inventory's 34 fixes are applied.
* The ECS is EnTT's design (ODR-279, verdict 100): a `Registry` of sparse-set pools and views, with no
  scheduler.

Generated: every rule whose text differs from 0.9.10_Hardened_35.

| Rule | 0.9.10_Hardened_35 → 0.9.15_Hardened_1 |
|---|---|
| `[ALC-1]` | changed |
| `[ALC-2]` | changed |
| `[ALC-5]` | added |
| `[ARN-1]` | changed |
| `[ARN-2]` | changed |
| `[ARN-3]` | changed |
| `[ARN-4]` | changed |
| `[ARN-6]` | changed |
| `[ARN-7]` | changed |
| `[ARN-8]` | changed |
| `[ATT-1]` | changed |
| `[ATT-2]` | changed |
| `[BCK-1]` | changed |
| `[BCK-2]` | changed |
| `[BCK-3]` | changed |
| `[BCK-4]` | changed |
| `[BCK-5]` | changed |
| `[BCK-6]` | changed |
| `[BCK-7]` | changed |
| `[BCK-8]` | added |
| `[BEN-6]` | changed |
| `[BLD-2]` | changed |
| `[BLD-14]` | added |
| `[BLD-FFI-1]` | changed |
| `[BLD-FFI-3]` | changed |
| `[BLD-FFI-5a]` | changed |
| `[BRW-1]` | changed |
| `[BRW-2]` | changed |
| `[BRW-3]` | changed |
| `[BRW-4]` | changed |
| `[BRW-5]` | changed |
| `[BRW-6]` | changed |
| `[BRW-7]` | changed |
| `[BRW-8]` | changed |
| `[BRW-9]` | changed |
| `[BRW-10]` | changed |
| `[BRW-11]` | changed |
| `[BRW-12]` | added |
| `[BRW-13]` | added |
| `[BRW-14]` | added |
| `[BRW-15]` | added |
| `[BRW-16]` | added |
| `[BRW-17]` | added |
| `[BRW-18]` | added |
| `[BRW-19]` | added |
| `[BRW-20]` | added |
| `[BRW-21]` | added |
| `[BRW-22]` | added |
| `[CELL-1]` | changed |
| `[CELL-2]` | changed |
| `[CELL-3]` | changed |
| `[CELL-5]` | removed |
| `[CELL-6]` | removed |
| `[CELL-7]` | removed |
| `[CELL-8]` | removed |
| `[CELL-9]` | removed |
| `[CELL-10]` | removed |
| `[CELL-12]` | removed |
| `[CELL-13]` | added |
| `[CG-C-4]` | changed |
| `[CG-C-9]` | changed |
| `[CG-C-11]` | changed |
| `[CG-LL-3]` | changed |
| `[CLI-2]` | changed |
| `[CLI-18]` | changed |
| `[CLI-20]` | changed |
| `[CLO-1]` | changed |
| `[CLO-2]` | changed |
| `[CLO-3]` | changed |
| `[CLO-4]` | changed |
| `[CLO-5]` | changed |
| `[CLO-6]` | changed |
| `[CLO-6a]` | changed |
| `[CLO-7]` | changed |
| `[CLO-10]` | changed |
| `[CLO-12]` | changed |
| `[CLO-13]` | removed |
| `[CLO-14]` | changed |
| `[CLO-15]` | removed |
| `[CLO-16]` | added |
| `[CLO-17]` | added |
| `[CLO-18]` | added |
| `[CLO-19]` | added |
| `[CLS-1]` | changed |
| `[CLS-2]` | changed |
| `[CLS-4]` | changed |
| `[CLS-5]` | changed |
| `[CLS-6]` | changed |
| `[CLS-7]` | changed |
| `[CLS-7a]` | changed |
| `[CLS-8]` | removed |
| `[CLS-9]` | changed |
| `[CLS-9a]` | changed |
| `[CLS-11]` | changed |
| `[CLS-12]` | added |
| `[CORO-3]` | changed |
| `[CORO-5]` | changed |
| `[CORO-6]` | changed |
| `[CORO-12]` | changed |
| `[CORO-13]` | changed |
| `[CORO-14]` | added |
| `[COST-3]` | changed |
| `[COST-6]` | added |
| `[CT-5]` | changed |
| `[CTL-1]` | changed |
| `[CTL-2]` | changed |
| `[CTL-3b]` | changed |
| `[CTL-7]` | changed |
| `[CTL-11]` | added |
| `[DET-2]` | changed |
| `[DET-3]` | changed |
| `[DET-5]` | changed |
| `[DIA-2]` | changed |
| `[DIA-3]` | changed |
| `[DIA-7]` | changed |
| `[DIA-8]` | changed |
| `[DIA-9]` | changed |
| `[DIA-10]` | changed |
| `[DIA-11]` | changed |
| `[DIA-16]` | changed |
| `[DIA-21]` | changed |
| `[DIA-22]` | changed |
| `[DIA-23]` | changed |
| `[DIA-25]` | added |
| `[DIA-26]` | added |
| `[DIA-27]` | added |
| `[DIA-28]` | added |
| `[DOC-3]` | changed |
| `[DRP-1]` | changed |
| `[DRP-5]` | changed |
| `[DRP-6]` | changed |
| `[DRP-7]` | changed |
| `[DRP-8]` | added |
| `[DRV-1]` | changed |
| `[DSJ-1]` | changed |
| `[DSJ-2]` | changed |
| `[DSJ-3]` | changed |
| `[DSJ-5]` | changed |
| `[DSJ-6]` | changed |
| `[DSJ-7]` | changed |
| `[DSJ-9]` | changed |
| `[DSJ-10]` | added |
| `[DSP-1]` | changed |
| `[DSP-3]` | changed |
| `[DSP-4]` | changed |
| `[DSP-5]` | changed |
| `[DSP-6]` | added |
| `[ECS-1]` | changed |
| `[ECS-2]` | changed |
| `[ECS-3]` | changed |
| `[ECS-4]` | changed |
| `[ECS-5]` | changed |
| `[ECS-7]` | changed |
| `[ECS-8]` | added |
| `[ECS-9]` | added |
| `[EFF-1]` | changed |
| `[EFF-2]` | changed |
| `[EFF-3]` | changed |
| `[EFF-4]` | changed |
| `[EFF-6a]` | removed |
| `[EFF-9]` | changed |
| `[EFF-11]` | changed |
| `[EFF-12]` | changed |
| `[EFF-13]` | removed |
| `[EFF-14]` | changed |
| `[EFF-15]` | changed |
| `[EFF-16]` | changed |
| `[EFF-18]` | changed |
| `[EFF-24]` | added |
| `[EFF-25]` | added |
| `[EFF-26]` | added |
| `[EFF-27]` | added |
| `[ERR-2]` | changed |
| `[ERR-3]` | changed |
| `[ERR-4]` | changed |
| `[ERR-8]` | changed |
| `[ERR-9]` | changed |
| `[ERR-13]` | changed |
| `[EXC-1]` | changed |
| `[EXC-2]` | changed |
| `[EXC-3]` | changed |
| `[EXC-3a]` | removed |
| `[EXC-4]` | removed |
| `[EXC-5]` | removed |
| `[EXC-6]` | removed |
| `[EXC-7]` | removed |
| `[EXC-8]` | removed |
| `[EXC-9]` | removed |
| `[EXC-10]` | removed |
| `[EXC-11]` | removed |
| `[EXC-12]` | removed |
| `[EXC-13]` | removed |
| `[EXC-14]` | removed |
| `[EXC-15]` | changed |
| `[EXC-16]` | changed |
| `[EXC-17]` | changed |
| `[EXC-18]` | removed |
| `[EXC-19]` | changed |
| `[EXP-4]` | changed |
| `[EXP-5]` | changed |
| `[EXP-6]` | changed |
| `[EXP-9]` | changed |
| `[EXP-10]` | changed |
| `[FFI-1]` | changed |
| `[FFI-2a]` | changed |
| `[FFI-8]` | changed |
| `[FFI-10]` | changed |
| `[FFI-11]` | changed |
| `[FFI-11b]` | changed |
| `[FFI-15]` | changed |
| `[FFI-17a]` | changed |
| `[FFI-17d]` | changed |
| `[FFI-17e]` | changed |
| `[FFI-21]` | changed |
| `[FFI-21a]` | changed |
| `[FFI-22]` | changed |
| `[FFI-23]` | changed |
| `[FFI-25]` | changed |
| `[FFI-26]` | changed |
| `[FFI-29b]` | changed |
| `[FFI-30c]` | changed |
| `[FFI-31b]` | changed |
| `[FFI-32]` | changed |
| `[FFI-33]` | changed |
| `[FFI-33a]` | changed |
| `[FFI-33b]` | changed |
| `[FFI-33c]` | changed |
| `[FFI-36]` | changed |
| `[FFI-36a]` | changed |
| `[FFI-37d]` | changed |
| `[FFI-39]` | changed |
| `[FFI-39c]` | changed |
| `[FFI-39d]` | changed |
| `[FFI-40]` | changed |
| `[FFI-40a]` | changed |
| `[FFI-41]` | changed |
| `[FFI-42a]` | changed |
| `[FFI-51]` | added |
| `[FFI-52]` | added |
| `[FFI-53]` | added |
| `[FFI-54]` | added |
| `[FFI-55]` | added |
| `[FFI-56]` | added |
| `[FMT-1]` | changed |
| `[FN-1]` | changed |
| `[FN-1a]` | changed |
| `[FN-2]` | changed |
| `[FN-2a]` | changed |
| `[FN-3]` | changed |
| `[FN-4]` | changed |
| `[FN-6]` | changed |
| `[FN-8]` | changed |
| `[FN-9]` | changed |
| `[GPU-2]` | changed |
| `[GPU-3]` | changed |
| `[GPU-6]` | changed |
| `[GPU-7]` | changed |
| `[GPU-9]` | changed |
| `[GPU-12]` | added |
| `[GRM-3]` | changed |
| `[GRM-4]` | changed |
| `[GRM-8a]` | changed |
| `[GRM-8b]` | changed |
| `[GRM-13]` | changed |
| `[GRM-15]` | changed |
| `[GRM-19]` | changed |
| `[GRM-27]` | changed |
| `[GRM-28]` | changed |
| `[GRM-30]` | changed |
| `[GRM-35]` | changed |
| `[GRM-36]` | changed |
| `[GRM-37]` | changed |
| `[GRM-38]` | changed |
| `[GRM-40]` | added |
| `[GRM-41]` | added |
| `[GRM-42]` | added |
| `[GRM-43]` | added |
| `[GRM-44]` | added |
| `[GRM-45]` | added |
| `[GRM-46]` | added |
| `[HASH-1]` | changed |
| `[HASH-3]` | changed |
| `[HEAP-1]` | changed |
| `[HEAP-3]` | changed |
| `[HEAP-4]` | removed |
| `[HEAP-5]` | removed |
| `[HEAP-6]` | changed |
| `[HEAP-7]` | changed |
| `[HEAP-9]` | changed |
| `[HEAP-10]` | changed |
| `[HND-3]` | changed |
| `[HR-2]` | changed |
| `[HR-2a]` | changed |
| `[HR-3b]` | changed |
| `[HR-6a]` | changed |
| `[HR-8]` | changed |
| `[HR-10]` | changed |
| `[HR-11]` | changed |
| `[HR-12]` | changed |
| `[HR-14]` | changed |
| `[HR-15]` | changed |
| `[HR-15a]` | changed |
| `[HR-16]` | changed |
| `[HR-17a]` | changed |
| `[HR-19]` | changed |
| `[HR-20]` | changed |
| `[HR-35]` | changed |
| `[HR-41]` | changed |
| `[HR-44]` | added |
| `[HR-45]` | added |
| `[HR-46]` | added |
| `[HR-47]` | added |
| `[IDE-4]` | changed |
| `[IDE-6]` | changed |
| `[IFC-5]` | added |
| `[IFC-6]` | added |
| `[IMP-11]` | changed |
| `[JOB-2]` | changed |
| `[JOB-3]` | changed |
| `[LEX-14]` | changed |
| `[LEX-15]` | changed |
| `[LEX-20]` | changed |
| `[LEX-21]` | changed |
| `[LEX-23]` | changed |
| `[LNT-6]` | changed |
| `[LT-1]` | changed |
| `[LT-1a]` | changed |
| `[LT-1b]` | removed |
| `[LT-4]` | changed |
| `[LT-5]` | changed |
| `[LT-6]` | changed |
| `[LT-7]` | changed |
| `[LT-14]` | changed |
| `[LT-18]` | changed |
| `[LT-22]` | changed |
| `[LT-23]` | changed |
| `[LT-25]` | changed |
| `[LT-26]` | changed |
| `[LT-27]` | changed |
| `[LT-34]` | changed |
| `[LT-35]` | changed |
| `[LT-36]` | changed |
| `[LT-39]` | changed |
| `[LT-42]` | changed |
| `[LT-44]` | changed |
| `[LT-45]` | added |
| `[LT-46]` | added |
| `[LT-47]` | added |
| `[MNG-1]` | changed |
| `[MNG-2]` | changed |
| `[MNG-6]` | changed |
| `[MOD-1]` | changed |
| `[MOD-5]` | changed |
| `[MOD-9]` | added |
| `[MONO-10]` | added |
| `[OBJ-1]` | changed |
| `[OBJ-2]` | changed |
| `[OBJ-3]` | changed |
| `[OBJ-4]` | changed |
| `[OBJ-5]` | changed |
| `[OPT-1]` | changed |
| `[OWN-1]` | changed |
| `[OWN-2]` | changed |
| `[OWN-3]` | changed |
| `[OWN-4]` | changed |
| `[OWN-6]` | changed |
| `[OWN-7]` | changed |
| `[OWN-8]` | changed |
| `[OWN-9]` | added |
| `[OWN-10]` | added |
| `[OWN-11]` | added |
| `[OWN-12]` | added |
| `[OWN-13]` | added |
| `[PAR-1]` | changed |
| `[PAR-2]` | changed |
| `[PHIL-2]` | changed |
| `[PHIL-3]` | changed |
| `[PHIL-4]` | changed |
| `[PHIL-8]` | changed |
| `[PHIL-9]` | changed |
| `[PHIL-10]` | changed |
| `[PHIL-11]` | changed |
| `[PHIL-15]` | changed |
| `[PHIL-16]` | added |
| `[PRF-1]` | changed |
| `[PRF-4]` | added |
| `[RC-1]` | changed |
| `[RC-2]` | changed |
| `[RC-2a]` | changed |
| `[RC-2b]` | changed |
| `[RC-2c]` | changed |
| `[RC-2d]` | changed |
| `[RC-2e]` | changed |
| `[RC-3]` | changed |
| `[RC-4]` | changed |
| `[RC-5]` | changed |
| `[RC-6]` | changed |
| `[RC-7]` | added |
| `[RFL-2]` | changed |
| `[RT-7]` | changed |
| `[RT-8]` | changed |
| `[RT-10]` | changed |
| `[RT-12]` | changed |
| `[SEL-1]` | changed |
| `[SEL-2]` | changed |
| `[SER-1]` | changed |
| `[SIMD-3]` | changed |
| `[SIMD-9]` | changed |
| `[SOA-2]` | changed |
| `[SOA-6]` | changed |
| `[SPN-1]` | changed |
| `[SPN-3]` | changed |
| `[SPN-4]` | changed |
| `[SPN-5]` | changed |
| `[STA-1]` | changed |
| `[STA-3]` | changed |
| `[STD-3]` | changed |
| `[STD-6]` | changed |
| `[STD-8]` | changed |
| `[STD-8b]` | changed |
| `[STD-9]` | changed |
| `[STD-11]` | changed |
| `[STD-12]` | changed |
| `[STD-14]` | changed |
| `[STD-15]` | changed |
| `[STD-16]` | changed |
| `[STD-17]` | changed |
| `[STD-18]` | changed |
| `[STD-19]` | changed |
| `[STD-21]` | changed |
| `[STD-25]` | changed |
| `[STD-26]` | changed |
| `[STD-29]` | added |
| `[STR-1]` | changed |
| `[STR-3]` | changed |
| `[STR-5]` | changed |
| `[STR-6]` | changed |
| `[STR-7]` | changed |
| `[TCB-4]` | changed |
| `[THR-1]` | changed |
| `[THR-2]` | changed |
| `[THR-3]` | changed |
| `[THR-5]` | changed |
| `[THR-6]` | changed |
| `[THR-7]` | changed |
| `[THR-8]` | changed |
| `[THR-9]` | changed |
| `[THR-10]` | changed |
| `[THR-11]` | changed |
| `[THR-12]` | changed |
| `[THR-13]` | changed |
| `[THR-15]` | changed |
| `[THR-17]` | added |
| `[THR-18]` | added |
| `[THR-19]` | added |
| `[THR-20]` | added |
| `[THR-21]` | added |
| `[TIER-1]` | changed |
| `[TST-6]` | changed |
| `[TST-11]` | changed |
| `[TST-31]` | added |
| `[TXT-1]` | changed |
| `[TXT-2]` | changed |
| `[TXT-3]` | changed |
| `[TXT-4]` | changed |
| `[TXT-5]` | changed |
| `[TXT-6]` | changed |
| `[TXT-8]` | changed |
| `[TXT-9]` | changed |
| `[TXT-10]` | changed |
| `[TXT-11]` | changed |
| `[TYP-1]` | changed |
| `[TYP-5]` | changed |
| `[TYP-6]` | changed |
| `[TYP-9]` | changed |
| `[TYP-9b]` | changed |
| `[TYP-13]` | changed |
| `[TYP-14]` | changed |
| `[TYP-15]` | changed |
| `[TYP-17]` | changed |
| `[TYP-19]` | changed |
| `[TYP-21]` | changed |
| `[TYP-22]` | changed |
| `[TYP-23]` | changed |
| `[TYP-24]` | changed |
| `[TYP-31]` | changed |
| `[TYP-34]` | changed |
| `[TYP-35]` | changed |
| `[TYP-36]` | changed |
| `[TYP-38]` | changed |
| `[TYP-39]` | changed |
| `[TYP-43]` | changed |
| `[TYP-44]` | added |
| `[TYP-45]` | added |
| `[TYP-46]` | added |
| `[UNS-1]` | changed |
| `[UNS-2]` | changed |
| `[UNS-4]` | changed |
| `[UNS-7]` | changed |
| `[UNS-8]` | changed |
| `[UNS-10]` | changed |
| `[UNS-10a]` | changed |
| `[UNS-11]` | added |
| `[UNS-12]` | added |
| `[WK-1]` | changed |
| `[WK-2]` | changed |
| `[WK-3]` | changed |
| `[WK-4]` | changed |
| `[WK-5]` | changed |
| `[WK-6]` | changed |
| `[WK-7]` | changed |
| `[WK-8]` | changed |
| `[WK-9]` | changed |
| `[WK-11]` | changed |
| `[WK-12]` | changed |
| `[WK-13]` | changed |
| `[WK-14]` | changed |
| `[WK-15]` | changed |
| `[WK-16]` | added |
| `[WK-17]` | added |
| `[WK-18]` | added |

## H.42 Changes in 0.9.15_Hardened_2 (from 0.9.15_Hardened_1)

**0.9.15_Hardened_2** writes in the owner's verdicts 101 to 106 and the lead's rulings for the compiler
work (`tasks/96/redesign/JOB.md`; `tasks/96/redesign/compiler-0.9.15/decisions.md`, sections 4 to 6):

* Verdict 101: a `thread_shared` value's owners are counted in two numbers, its making thread's own
  (plain) and the other threads' (atomic), with a hand-over to the making thread through its queue and
  the compiler's check points, and an immediate hand-over where an owner is stored where other threads
  can reach it (biased counting; `[OBJ-1]`'s 32-byte header, `[RC-4]`, `[RC-5]`, `[WK-3]`, `[WK-12]`,
  `[RT-7]`, `[RT-8]`, `[RT-10]`, `[COST-3]`'s measured row).
* Verdict 102: a C contract writes `owns` for "C takes this object"; `owned` is refused everywhere.
* Verdict 103: `shares`, `shares mut` and `thread_shared` are not made of a place that has one owner and
  is not a local (`E2122`, new).
* Verdict 104: a scoped task's claims end at its join, `h.join()` or the end of the `with scope:` block.
* Verdict 105 (for now): a task's or a chunk's own whole-number total stops the program on overflow.
* Verdict 106: a contract on a function type is written above its `type` line.
* The lead's rulings: `from` names any field of `this`; when a store may close a loop of owners and the
  two proofs that it does not (`E3032`); a deferred-drop region runs the drops at once; a `Retained`
  token is an owner edge; a manager type with no fields has one instance; `=` on a `static` is `E3023`;
  `E3100` for an unverified foreign call outside `unsafe`, `E5002` for a contract's own errors and two
  destructors of one foreign type; two bounds that both declare a method are `E2070`; every function
  has a type of its own (ODR-278); a cdylib exports its class handles' functions; the trampoline's
  number on 32-bit targets; `std.core.type_name[T]()` and the ECS's `Storage[T]`; `@component` gone;
  what `ember fmt --migrate` also rewrites.

Generated: every rule whose text differs from 0.9.15_Hardened_1.

| Rule | 0.9.15_Hardened_1 → 0.9.15_Hardened_2 |
|---|---|
| `[ALC-1]` | changed |
| `[ATT-1]` | changed |
| `[BCK-7]` | changed |
| `[BLD-FFI-5a]` | changed |
| `[CLI-20]` | changed |
| `[CLO-1]` | changed |
| `[ECS-2]` | changed |
| `[EFF-2]` | changed |
| `[FFI-11]` | changed |
| `[FFI-23]` | changed |
| `[FFI-28]` | changed |
| `[FFI-36a]` | changed |
| `[FFI-39c]` | changed |
| `[FFI-53]` | changed |
| `[FFI-54]` | changed |
| `[HEAP-10]` | changed |
| `[LEX-15]` | changed |
| `[LT-1a]` | changed |
| `[OBJ-1]` | changed |
| `[OWN-13]` | changed |
| `[PAR-3]` | changed |
| `[RC-1]` | changed |
| `[RC-4]` | changed |
| `[RC-5]` | changed |
| `[RT-7]` | changed |
| `[RT-8]` | changed |
| `[RT-10]` | changed |
| `[STA-1]` | changed |
| `[THR-1]` | changed |
| `[THR-2]` | changed |
| `[THR-7]` | changed |
| `[THR-11]` | changed |
| `[THR-13]` | changed |
| `[THR-21]` | changed |
| `[TYP-24]` | changed |
| `[TYP-45]` | changed |
| `[WK-3]` | changed |
| `[WK-5]` | changed |
| `[WK-7]` | changed |
| `[WK-12]` | changed |

## H.43 Changes in 0.9.15_Hardened_3 (from 0.9.15_Hardened_2)

**0.9.15_Hardened_3** writes in the owner's verdicts 107 to 117 (`tasks/96/redesign/JOB.md`) and the
lead's details for them (`tasks/96/redesign/compiler-0.9.15/decisions.md`, section 7):

* Verdict 107: a value made in a memory manager keeps no word naming its instance when the manager frees
  nothing one by one (and the value does not grow), or implements `FoundByAddress`, whose pieces start at
  a multiple of 65,536 bytes with the instance's address in their first word (`[ALC-1]`, `[ALC-2]`,
  `[OBJ-4]`, `[COST-3]`).
* Verdicts 108 and 113: a `pub` generic writes everything its type parameters need; one that is not
  `pub` writes all of it or none, and part of a list is `E2040` with the whole line in the message
  (`[TYP-17]`, `[IFC-6]`).
* Verdicts 109 to 112: what a type parameter needs is written after `needs`, one bare and several in
  round brackets (`[T: needs (Add[T, Output = T], Copy)]`), in type-parameter lines, `extend`
  brackets, an interface's parents, associated types and `where` clauses; `+` between needs is gone
  (`E0100`), kept only after `dyn` and `some` (`[TYP-17]`, `[LEX-15]`, Part III).
* Verdict 114: an object handed whole to another thread while a `weakref` link on this thread points
  at it goes, and the links here find it ended: its contents move when a link exists (Rust's
  `Rc::try_unwrap`), objects it owns of `weakref`-target classes too, a view opened from a link is not
  used after it, and an object C keeps never moves (a link to it stops the program) (`[THR-2]`,
  `[RC-5]`, `[WK-12]`, `[FFI-35a]`, `[UNS-4]`; `E7015` no longer covers it).
* Verdicts 115 and 116: typed variadic generics and functions in C++'s spelling, `[Ts...: needs
  Display]` above `fn log(values: Ts...):` (`[TYP-47]`, new); a view lists any number of components,
  `reg.view[Position, Velocity]()` (`[ECS-3]`, `[ECS-4]`); `zip` takes any number (`[STD-26]`).
* Verdict 117: `print`, `println`, `eprint`, `eprintln` and `format_to` are library functions with a
  pack, `[Ts...: needs Printable]`; the compiler implements `Printable` for every type with `Display` or
  `Debug` (`[STD-9]`, `[STD-18]`, `[TYP-26]`, §IV.8).

Generated: every rule whose text differs from 0.9.15_Hardened_2.

| Rule | 0.9.15_Hardened_2 → 0.9.15_Hardened_3 |
|---|---|
| `[ALC-1]` | changed |
| `[ALC-2]` | changed |
| `[ARN-3]` | changed |
| `[ARN-5d]` | changed |
| `[ARN-11]` | changed |
| `[CELL-1]` | changed |
| `[CLI-20]` | changed |
| `[CLO-14]` | changed |
| `[DIA-21]` | changed |
| `[ECS-3]` | changed |
| `[ECS-4]` | changed |
| `[ECS-8]` | changed |
| `[EFF-2]` | changed |
| `[ERR-1]` | changed |
| `[ERR-4]` | changed |
| `[ERR-8]` | changed |
| `[ERR-12]` | changed |
| `[FFI-8]` | changed |
| `[FFI-35a]` | changed |
| `[FN-8]` | changed |
| `[GRM-31]` | changed |
| `[GRM-34]` | changed |
| `[GRM-40]` | changed |
| `[HASH-1]` | changed |
| `[HASH-4]` | changed |
| `[IFC-3]` | changed |
| `[IFC-4]` | changed |
| `[IFC-5]` | changed |
| `[IFC-6]` | changed |
| `[LEX-15]` | changed |
| `[LEX-21]` | changed |
| `[OBJ-4]` | changed |
| `[RC-5]` | changed |
| `[RNG-5a1]` | changed |
| `[SER-1]` | changed |
| `[SOA-6]` | changed |
| `[STD-8]` | changed |
| `[STD-9]` | changed |
| `[STD-11]` | changed |
| `[STD-12]` | changed |
| `[STD-15]` | changed |
| `[STD-16]` | changed |
| `[STD-17]` | changed |
| `[STD-18]` | changed |
| `[STD-19]` | changed |
| `[STD-21]` | changed |
| `[STD-26]` | changed |
| `[STD-27]` | changed |
| `[THR-2]` | changed |
| `[THR-8]` | changed |
| `[THR-9]` | changed |
| `[TYP-17]` | changed |
| `[TYP-19]` | changed |
| `[TYP-22]` | changed |
| `[TYP-26]` | changed |
| `[TYP-36]` | changed |
| `[TYP-37]` | changed |
| `[TYP-43]` | changed |
| `[TYP-47]` | added |
| `[UNS-4]` | changed |
| `[UNS-5]` | changed |
| `[WK-12]` | changed |

## H.44 Changes in 0.9.15_Hardened_4 (from 0.9.15_Hardened_3)

**0.9.15_Hardened_4** writes in the owner's verdicts 118 to 124 from the ODR review of 2026-10-29
(`tasks/96/redesign/JOB.md`) and the lead's details for them (`tasks/96/redesign/compiler-0.9.15/decisions.md`,
section 9):

* Verdict 118: an `unsafe fn` is unsafe as a whole, its body unsafe code; code in it to be checked as
  safe code is written in a `safe:` block (`[UNS-12]`, rewritten; `[UNS-1]`, `[TIER-1]`, Part III, the
  glossary). `L3020` (new): a `safe:` block outside unsafe code; `L3010` also covers an `unsafe:` block
  inside unsafe code (`[UNS-3]`).
* Verdict 119: everything written in unsafe code is unsafe: a function declared there is an
  `unsafe fn` and carries `@safety`, a lambda there is an `unsafe fn(…)` value (`[UNS-12]`, `[UNS-7]`).
* Verdict 120: a C function declared in an `unsafe extern` block carries `@safety` (`[UNS-7]`).
* Verdict 121: the header an overlay was written for is recorded by Ember in `ember.overlays`, a
  fingerprint per declaration the overlay describes, and every overlay is checked against it; the
  overlay line's `header_hash` is gone (`[FFI-55]`, `[GRM-35]`, `[CLI-5]`, `[MAN-2]`, `E5010`).
* Verdict 122: a call through a stored function has the effects of the functions that reach that
  place, field by field; a place none reaches contributes nothing (`[EFF-2]`, `[BLD-14]`).
* Verdict 123: `E5013` names C's promotion and both sizes instead of `as i64` (`[FFI-8]`).
* Verdict 124: a link past its time limit is ended and is `E9042` (`[BLD-15]`, new; `[MAN-8]`'s
  `[link] timeout`).
* The lead's details: the statement rules of `E3100` (one per statement, the block around the whole
  statement) in `[UNS-1]`; a lambda in unsafe code needs no `@safety`; `W3013`'s three wordings.

Generated: every rule whose text differs from 0.9.15_Hardened_3.

| Rule | 0.9.15_Hardened_3 → 0.9.15_Hardened_4 |
|---|---|
| `[BLD-14]` | changed |
| `[BLD-15]` | added |
| `[CLI-5]` | changed |
| `[EFF-2]` | changed |
| `[FFI-8]` | changed |
| `[FFI-55]` | changed |
| `[GRM-35]` | changed |
| `[LEX-15]` | changed |
| `[MAN-2]` | changed |
| `[MAN-8]` | changed |
| `[TIER-1]` | changed |
| `[UNS-1]` | changed |
| `[UNS-3]` | changed |
| `[UNS-7]` | changed |
| `[UNS-11]` | changed |
| `[UNS-12]` | changed |

## H.45 Changes in 0.9.15_Hardened_5 (from 0.9.15_Hardened_4)

**0.9.15_Hardened_5** writes in the owner's verdicts 125 to 131 from the ODR review of 2026-10-29, evening
(`tasks/96/redesign/JOB.md`); the lead's other rulings of Hardened_4 were approved as written
(`tasks/96/redesign/compiler-0.9.15/decisions.md`, "Owner verdicts on §9's rulings"):

* Verdict 125: a function `import c` or `import cpp` writes that its overlay entry leaves unsafe to
  call carries `@safety` in that entry; a call of one without it is `L3015`, once for each function, at
  its first call in unsafe code (`[UNS-7]`).
* Verdict 126: a lambda written in unsafe code carries `@safety`, written just before it, and one
  without it is `L3015` (`[UNS-7]`, the grammar's `lambda`).
* Verdict 127: every program is a package; a build of a file with no `ember.toml` writes a default one
  (`[MAN-9]`, new); the overlay record is always beside it (`[FFI-55]`).
* Verdicts 128 and 129: a callable parameter holds what its callers pass, and a function put into a
  list is followed into the list's place, as `[EFF-2]` says; dropping a function value has the effects
  of dropping the functions that reach its place (`[EFF-2]`).
* Verdict 130: an overlay entry for a function unsafe to call is written `unsafe fn`, with its
  `@safety`; the `unsafe` contract word is gone (`[GRM-35]`, `[FFI-2]`, `[FFI-11]`, `[CLI-5]`).
* Verdict 131: a C++ class member's entry is written under its class; an entry outside a class section
  that names only members is `E5010` (`[GRM-35]`, `[FFI-12]`, `[FFI-40a]`).
* `ember bind <header> --check <overlay> --merge` re-records an overlay once it is checked, and
  `E5010`'s message names that command (`[CLI-7]`, `[FFI-55]`).

Generated: every rule whose text differs from 0.9.15_Hardened_4.

| Rule | 0.9.15_Hardened_4 → 0.9.15_Hardened_5 |
|---|---|
| `[CLI-5]` | changed |
| `[CLI-7]` | changed |
| `[EFF-2]` | changed |
| `[FFI-2]` | changed |
| `[FFI-11]` | changed |
| `[FFI-12]` | changed |
| `[FFI-40a]` | changed |
| `[FFI-55]` | changed |
| `[GRM-35]` | changed |
| `[MAN-9]` | added |
| `[UNS-7]` | changed |

## H.46 Changes in 0.9.15_Hardened_6 (from 0.9.15_Hardened_5)

**0.9.15_Hardened_6** writes in the owner's verdicts 132 to 135 from the review of 2026-10-30, morning
(`tasks/96/redesign/JOB.md`, and `tasks/96/redesign/compiler-0.9.15/decisions.md`, "Owner verdicts on
§11's rulings"):

* Verdict 132: an overlay entry for a C++ method without `@ffi(const)` makes the method one that
  changes its object, whatever the importer reads from the header's body; `@ffi(const)` keeps it one
  that only reads its object (`[FFI-40a]`, `[GRM-35]`). This reverses Hardened_5's reading.
* Verdict 133: a hand-written C declaration unsafe to call is written `unsafe fn`, with `@safety` above
  it, as an overlay entry is; `@ffi(unsafe)`, `param(p, unsafe)` and `result(unsafe)` are `E0104`, whose
  message gives the line to write, and `ember fmt --migrate` rewrites them, with an overlay entry's
  `unsafe` contract words (`[FFI-2]`, `[FFI-10]`, `[FFI-11]`, `[CLI-20]`).
* Verdict 134: `L3015` for an imported function comes once, at its first call by place, with a note
  naming each other call (`[UNS-7]`).
* Verdict 135: the default manifest's note reads "created `<path>`: every program needs one, and this
  folder had none" (`[MAN-9]`).
* The LLVM backend, which passed the conformance and performance suites, is the default; the C
  backend is chosen with `--backend c` (`[CG-LL-1]`, §I.1, the manifest example of §XVII).

Generated: every rule whose text differs from 0.9.15_Hardened_5.

| Rule | 0.9.15_Hardened_5 → 0.9.15_Hardened_6 |
|---|---|
| `[CG-LL-1]` | changed |
| `[CLI-20]` | changed |
| `[FFI-2]` | changed |
| `[FFI-10]` | changed |
| `[FFI-11]` | changed |
| `[FFI-40a]` | changed |
| `[GRM-35]` | changed |
| `[MAN-9]` | changed |
| `[UNS-7]` | changed |
