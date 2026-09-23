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
