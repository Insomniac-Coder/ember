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
| CELL | CELL-6a, CELL-11 | folded into `[CELL-6]` and the prelude (`[MOD-5]`) |
| CLI | CLI-3, CLI-14, CLI-16, CLI-17 | condensed into the command listing of §XVII.1 and `[CLI-19]` |
| CLO-ABI | CLO-ABI-1, CLO-ABI-2 | compiler internals |
| CMP | CMP-1, CMP-2, CMP-3 | compiler internals |
| COMP | COMP-1 | compiler internals |
| CONF | CONF-6 | folded into `[CONF-1]` |
| COR-ABI | COR-ABI-1, COR-ABI-2 | compiler internals |
| CTL | CTL-3a, CTL-3c | folded into `[CTL-3]`; its conformance obligations are `[TST-4a]` |
| CTR | CTR-11 | retired in 0.6.2 |
| CXX | CXX-2, CXX-3, CXX-4, CXX-5, CXX-6, CXX-7 | C++ corpus details; `[CXX-1]` in Annex C |
| DET-IMPL | DET-IMPL-1, DET-IMPL-2 | compiler internals |
| DIA | DIA-17, DIA-19 | condensed into §XVII.6 (shape tables, `[DIA-21]`, `[DIA-24]`) |
| EFF | EFF-11a, EFF-19a, EFF-19b, EFF-22 | folded into `[EFF-11]`, `[EFF-16]` and `[EFF-19]` |
| EXP | EXP-7 | replaced by floor semantics, `[TYP-28]` |
| FFI | FFI-11d, FFI-11e, FFI-18, FFI-19, FFI-20, FFI-24a, FFI-32a, FFI-32b, FFI-32c, FFI-32d, FFI-32e, FFI-34, FFI-34a, FFI-35, FFI-37e, FFI-37f, FFI-43a | C++ interop details (Annex C) or folded into the five-axis contract `[FFI-11]` |
| FFI-CB | FFI-CB-1 | folded into `[FFI-21]` |
| FFI-IMPL | FFI-IMPL-1, FFI-IMPL-3 | compiler internals |
| FN | FN-6a, FN-6b | removed with `@latebound` (F-168); callable types elide regions per call (`[LT-7]`) |
| GATE | GATE-1, GATE-2, GATE-3, GATE-4, GATE-5, GATE-6, GATE-7, GATE-8, GATE-8a | implementation plan; out of the language document |
| GEN-COH | GEN-COH-1, GEN-COH-2 | folded into `[TYP-20]` |
| GRM | GRM-9, GRM-14, GRM-22 | folded into Part III (`GRM-14` retired with the access-mode generic kind, F-024) |
| HIR | HIR-1, HIR-2 | compiler internals |
| HOT | HOT-1, HOT-3, HOT-5, HOT-10 | compiler internals of hot reload |
| HR | HR-12b, HR-21a, HR-22a, HR-23a, HR-26, HR-40 | condensed into Annex B |
| HR-IMPL | HR-IMPL-1, HR-IMPL-3 | compiler internals of hot reload |
| IDE | IDE-1, IDE-2, IDE-5, IDE-7, IDE-10 | language-server design; out of the language document |
| IMP | IMP-1, IMP-2, IMP-3, IMP-7, IMP-8, IMP-9, IMP-10 | implementation plan; out of the language document (`[IMP-11]` is new) |
| JOB | JOB-4 | debug-only access-set verification removed (a check in one profile is not a guarantee, `[PHIL-13]`) |
| LAY | LAY-1 | replaced by `[LAY-2]` |
| LEX | LEX-14a, LEX-15a, LEX-15b | folded into `[LEX-14]` and `[LEX-15]` |
| LT | LT-2, LT-2a, LT-4a, LT-4b, LT-8, LT-8a, LT-9, LT-10, LT-11, LT-11a, LT-12, LT-13, LT-15, LT-19, LT-28, LT-29, LT-31, LT-31a, LT-32, LT-33, LT-37, LT-40, LT-41 | `with_views*` removed (F-168); multi-region rules condensed into §VII.4 |
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
Hardened_28 adds ODR-070, the nesting limit (D-331).

| ODR | Ruling | Rules |
|---|---|---|
| ODR-021 | float `//` and `%` are Python's: the exact floor modulo rounded once, and the floor quotient consistent with it | `[TYP-29]` |
| ODR-022 | an untyped literal is never left open: `total = 0` declares an `int` whatever the later uses | `[TYP-23]` |
| ODR-023 | a function returning a value that can reach the end of its body is `E2182` (Hardened_4) | `[FN-10]`, §XVII.9 |
| ODR-024 | a borrowed or `mut` parameter whose type is not `Copy` is a source parameter a returned view may borrow, and a borrowed parameter is passed by address except a view or a `Copy` value holding no `Cell` (Hardened_5) | `[LT-1]`, `[LT-1a]`, `[LT-1b]`, `[LT-7]`, `[LT-44]`, `[FN-1]`, `[FN-3]`, `[FN-6]`, `[BRW-8]`, `[CORO-6]`, §VIII.3, §XVII.6 B7, §XVII.9, Appendix F |
| ODR-025 | `[ERR-4]`'s methods that take a function are eager, move the payload in (`filter` borrows it) and take `once fn`; an unannotated lambda parameter takes `owned` from the expected callable type, never `mut` (Hardened_6) | `[ERR-4]`, `[CLO-7]`, `[TYP-23]` |
| ODR-026 | a type that declares `drop` is not implicitly `Clone`; `@derive(Clone)` or a written `clone` gives it one (Hardened_7) | `[STR-5]` |
| ODR-027 | a range is a value: the prelude's range types are structs with public bounds, `Copy` when the bound is, and a `for` over one counts over a copy of its bounds, leaving it unchanged; `a..` overflows at its type's maximum (Hardened_8) | `[CTL-3]`, `[STD-8]`, `[STD-26]` |
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

Generated: every rule whose text differs from Hardened_2.

| Rule | H2 → now |
|---|---|
| `[ALC-1]` | changed |
| `[ARN-3]` | changed |
| `[ARN-11]` | changed |
| `[BLD-2]` | changed |
| `[BLD-8]` | changed |
| `[BRW-5]` | changed |
| `[BRW-8]` | changed |
| `[CELL-2]` | changed |
| `[CG-C-11]` | changed |
| `[CLO-7]` | changed |
| `[CLS-4]` | changed |
| `[CORO-6]` | changed |
| `[CT-4]` | changed |
| `[CTL-1]` | changed |
| `[CTL-3]` | changed |
| `[DET-2]` | changed |
| `[DET-4]` | changed |
| `[DRP-6]` | changed |
| `[EFF-1]` | changed |
| `[EFF-7]` | changed |
| `[EFF-16]` | changed |
| `[ERR-1]` | changed |
| `[ERR-4]` | changed |
| `[ERR-10]` | changed |
| `[ERR-12]` | changed |
| `[EXC-17]` | changed |
| `[FMT-1]` | changed |
| `[FN-1]` | changed |
| `[FN-1a]` | changed |
| `[FN-3]` | changed |
| `[FN-6]` | changed |
| `[FN-9]` | changed |
| `[FN-10]` | changed |
| `[GRM-34]` | changed |
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
| `[RT-10]` | changed |
| `[SIMD-5]` | changed |
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
| `[TXT-10]` | changed |
| `[TYP-13]` | changed |
| `[TYP-15]` | changed |
| `[TYP-21]` | changed |
| `[TYP-23]` | changed |
| `[TYP-26]` | changed |
| `[TYP-29]` | changed |
| `[TYP-34]` | changed |
| `[TYP-37]` | changed |
| `[TYP-38]` | changed |
| `[TYP-39]` | changed |
| `[TYP-41]` | added |
