# Change list — 0.9.9_Hardened_1 → 0.9.9_Hardened_2

The three passes found 21 memory-safety items (MS), 16 consistency items (CS), 9 speed items (SP) and
22 ergonomics items (E). This list turns them into one coherent set of changes, each with its source,
the rules it edits, and why the combination holds together. Every change keeps the three goals in
the same order the passes used: memory safety first, then one meaning and C speed, then Python feel.

**Versioning.** Several changes alter what programs mean or accept. Before 1.0 there is one language
(`[VER-8]`), so `[VER-9]` is amended (C-17) to let a pre-1.0 hardening change the language provided
Appendix H lists every such change and `ember fmt --migrate` rewrites what can be rewritten. Rule ids
keep their policy: ids unknown to 0.9.8 are marked *(new in 0.9.9)*; Appendix H §H.4 lists H1 → H2 by
rule.

## A. Memory safety (all adopted)

| C | Source | Change | Rules |
|---|---|---|---|
| C-01 | MS-01 | the closure given to `thread.spawn` or an unscoped `jobs.submit`, and its result, carry only the static region (`E3063`, help `thread.scope()`) | `[THR-10]`, `[JOB-2]` |
| C-02 | MS-02 | `Shared.get` begins a checked read access, `get_mut` a write access; `Shared[T]` is never `Send`/`Sync`; new `SyncShared[T]` (`T: Sync`) with atomic counts and `get` only | `[HEAP-4]`, `[HEAP-5]`, `[THR-8]`, `[THR-9]`, new `[HEAP-10]`, `[SEL-1]`, §XV |
| C-03 | MS-03 | two-phase initialisation: defaults first; a derived `init` assigns its own undefaulted fields before `super.init`; no use of `self` as a whole before `super.init` returns | `[CLS-2]`, `[CLS-4]`, `[CLS-10]`, new `[CLS-11]` |
| C-04 | MS-04 | retained and once callbacks are `owned fn` with static captures; callback captures are `Send` unless the contract says `threads(main)`/`threads(creator)` | `[FFI-21]`, `[FFI-22]` |
| C-05 | MS-05 | a function in an `unsafe extern` block is safe to call only if declared `safe fn` (contextual keyword), an asserted fact in the trusted-base report | `[FFI-10]`, `[GRM-35]` grammar of `extern_item`, `[LEX-15]`, Part XVI example |
| C-06 | MS-06 | `yield` while a `@must_drop` value is live is `E2231` (the code first proposed, `E2221`, already reports a borrow across `yield`) | new `[CORO-13]` |
| C-07 | MS-07 | `[RT-10]` covers copying a strong handle; `Weak.upgrade` on a `@sync` object is a compare-exchange that fails at zero | `[RT-10]`, `[WK-12]` |
| C-08 | MS-08 | `char32_t` maps to `u32`; `char.from_u32` converts | `[FFI-8]` |
| C-09 | MS-09 | SIMD `load`/`store` check length; gathers and scatters check every lane; unchecked forms only in `unsafe` | new `[SIMD-9]` |
| C-10 | MS-10 | views reached through a class handle or `Shared` get no alias fact unless the loop has no call and no access through another handle; a reference into a `Copy` field of a class object may observe writes through other handles | `[SIMD-3]`, `[CG-C-4]`, new `[EXC-17]` |
| C-11 | MS-11 | the dynamic access a borrow through a class handle begins ends where its loan dies, in whichever function that is | new `[EXC-18]` |
| C-12 | MS-12, CS-10 | calling a callable stored in a class field is a write access to the field for the call; calling any owned callable value needs a mutable place | §VIII.3 table, `[CLO-2]` |
| C-13 | MS-13 | accesses held across a call into C++ stay active and checked; `L3013` warns | `[FFI-39d]`, `[EXC-7]` |
| C-14 | MS-14, SP-09 | stack probes, guard pages, `stack overflow in <function>` | new `[RT-12]`, `[COST-3]` row |
| C-15 | MS-15 | `run_parallel` checks systems not known at compile time before starting them | `[ECS-4]` |
| C-16 | MS-16–21 | proxy borrows; incoherent `Hash`/`Ord` stay memory-safe; exit with detached threads; `keep_alive` for raw pointers; phantom parameters count for `Send`/`Sync`; `read_file` confined to the package | `[SOA-6]`, `[HASH-3]`, `[STD-15]`, `[THR-10]`, `[RC-3]`, `[TYP-35]`, `[CT-2]` |

## B. Consistency (all adopted)

| C | Source | Change | Rules |
|---|---|---|---|
| C-17 | CS-01 | before 1.0 a hardening may change the language when Appendix H lists each change; from 1.0 `[VER-2]` governs | `[VER-9]` |
| C-18 | CS-02, CS-03, SP-03 | contract verdicts assume every listed proof is applied; grouped overflow checks are required in vectorisable-form loops | `[EFF-15]`, `[SIMD-7]`, `[SIMD-5]` |
| C-19 | CS-04 | `debug_assert` named as the one profile-dependent check | `[PRF-1]` |
| C-20 | CS-05 | the two-phase example without a call-site mode | `[BRW-3]` |
| C-21 | CS-06 | `Generator` in the prelude | `[MOD-5]` |
| C-22 | CS-07 | `W2015` only for more digits than the type keeps | `[LEX-17a]` |
| C-23 | CS-08 | `Weak[C]` is `Send`/`Sync` iff `C` is `@sync` | `[THR-8]`, `[THR-9]` |
| C-24 | CS-09 | parallel float reductions: per chunk, then chunks in order | `[PAR-3]` |
| C-25 | CS-11–16 | engine name removed from `[HND-2]`; signature notation in `[OWN-6]`; query iteration borrows; `[ERR-13]` cites `[STD-10]`; the Annex A fixture | `[HND-2]`, `[OWN-6]`, `[ECS-3]`, `[ERR-13]`, `[TST-6]` |

## C. C-like speed (all adopted)

| C | Source | Change | Rules |
|---|---|---|---|
| C-26 | SP-01 | `step_by` joins the guaranteed counted-loop lowering | `[CTL-3]` |
| C-27 | SP-02 | the variable of `for i in a..b` carries the fact `a <= i < b` | `[RNG-4]` |
| C-28 | SP-04, SP-05, SP-08 | cost rows for contraction off, 64-bit `int`, float sorting; lint `L4003` for dense `Array[int]` in hot loops | `[COST-3]`, `[LNT-6]` |
| C-29 | SP-07 | the default hasher is a fast non-cryptographic hash, fixed-seed | `[HASH-2]` |

## D. Ergonomics

| C | Source | Change | Decision | Rules |
|---|---|---|---|---|
| C-30 | E-01, SP-06, CS-14 | per-field access state for each non-`Copy` field of a class; the header word keeps whole-object (`mut self`) accesses; messages name the exact field | adopted: removes false conflicts, costs one word per non-`Copy` field and nothing per access | `[OBJ-1]`, §VIII.3, `[EXC-1]`, `[EXC-6]`, `[EXC-15]`, new `[EXC-19]`, `[COST-3]` |
| C-31 | E-02 | `for k in m:` yields keys; `m.items()` pairs; `m.values()` | adopted: closes a silent Python difference (`[PHIL-14]`) | `[CTL-1]`, `[STD-16]`, Appendix E, examples |
| C-32 | E-03 | top-level statements in the entry file form an implicit `main` | adopted | `[GRM-2]`, `[FN-8]`, `[CLI-4]` |
| C-33 | E-04 | prelude functions `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` with Python's meaning; `len` of a string is `E2073` with both fix-its | adopted: Python spellings with Python meaning are what `[PHIL-14]` asks for; `[STD-13]` narrowed to method names | `[MOD-5]`, new `[STD-26]`, `[STD-13]`, `[DIA-21]`, Appendix E |
| C-34 | E-05 | generator expressions `(e for x in it if c)` | adopted | new `[GRM-38]`, `[TYP-38]` |
| C-35 | E-06 | printing uses `Display`, else `Debug` | adopted: Python's `str` → `repr` fallback | `[STD-9]`, `[LEX-19]` |
| C-36 | E-07, E-17 | `T` converts to `Option[T]` at a coercion site (one level); `None` then a later `T` infers `Option[T]` | adopted | `[TYP-5]`, `[TYP-23]` |
| C-37 | E-08 | falling off the end of a `Result[void, E]` function returns `Ok(())` | adopted | new `[FN-10]` |
| C-38 | E-09 | a lambda written as the argument of an `owned fn` parameter captures by move | adopted | new `[CLO-15]` |
| C-39 | E-10 | standard modules importable without `std.`; a package module of the same name wins, with `W1003` | adopted | `[MOD-3]` |
| C-40 | E-11 | iterator adapters directly on `Iterable` values | adopted | `[STD-19]` |
| C-41 | E-12 | float `Display` is the shortest round-trip text | adopted | `[STD-20]` |
| C-42 | E-13 | shape N13, undeclared field, with a fix-it | adopted | §XVII.6.2 |
| C-43 | E-14–16 | `[DIA-21]` rows: `case`, `global`, `nonlocal`, `*args`, `**kwargs`, `%` formatting, `.format` | adopted | `[DIA-21]` |
| C-44 | E-19, E-20, E-22 | N4 fix-it for int/float mixing; moved-list note; a side-by-side program in Appendix E | adopted | §XVII.6.2, Appendix E |
| — | E-18, E-21 | no change needed | — | — |
| — | rejected in Pass 3 | negative indices, `with … as`, implicit `str` → `String`, fields by assignment | rejected, reasons in PASS3 | — |

## How the changes fit together

* C-02, C-23 and C-01 complete `[THR-13]`: every route across threads is now either a `@sync` object,
  a `SyncShared`, a static, a scoped borrow or an owned value with no borrows.
* C-30 and C-11/C-12 are one design: each non-`Copy` field has its own access state, every long-term
  access (field call, callable call, view) records into it, and it ends where the loan dies.
* C-36 (`T` → `Option[T]`) is what makes C-33's `range(a, b = None)` and many optional parameters read
  like Python, and E-17's `None` inference.
* C-31 changes `Map` iteration; the only other construct iterating a `Map` directly is the `for` loop, so
  `[STD-16]`, `[CTL-1]` and the examples change together, and `ember fmt --migrate` (C-17) rewrites
  `for k, v in m:` to `for k, v in m.items():`.
