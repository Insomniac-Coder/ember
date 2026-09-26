# HANDOFF — writing Ember 0.9.9_Hardened_1

**Read this first after any context reset.** It is updated after every Part is written.

## The task (owner, 2026-09-23)

> Use the 212 findings and your understanding of the language to write the **v9.9 Hardened 1**
> document — the most coherent, well-defined language spec that takes care of memory safety, C-like
> speed and Python-like ergonomics. Complete freedom to make any decision that satisfies that goal.
> Small workflows only if absolutely necessary, and **ask permission first, stating the agent count**
> (the only thing that needs permission). Keep a context-reset-safe handoff and memory file updated
> periodically.

* "v9.9" is read as **0.9.9_Hardened_1** (next after 0.9.8; semantic changes ⇒ new language
  version, hardening resets to 1). Stated in the doc's front matter.
* Findings: `tasks/audit/FINDINGS.md` (F-001–F-214, categorised C1–C16 in its index).
* The owner delegated the owner decisions: this document decides C9–C13 itself and records each in
  Appendix H (changes) and Appendix G (finding → resolution).
* No workflow has been used. None planned unless a real need appears (then ask, with agent count).

## AFTER H1 IS BUILT — owner's follow-on plan (2026-09-23, binding)

> "after you are done creating 0.9.9 H1 I need you to do 3 passes on the document"

0. **FIRST (owner, 2026-09-23): findings-coverage audit of H1.** Every F-001..F-214 must be accounted
   for in H1 — Appendix G maps each to its resolution and the rule ids that carry it; a script checks
   every cited id is defined in H1; then read each resolution against the rule text to confirm the
   text really does it (not just claims it). Fix H1 for any gap, record the audit in
   `passes/PASS0-findings-coverage.md`. Only then start the passes below.
   **Also (owner, 2026-09-23): feature parity with 0.9.8.** Nothing 0.9.8 has may be missing from 0.9.9
   unless something better replaced it. Check: every retired 0.9.8 rule id (Appendix H list) →
   PRESENT (folded, cite) / REPLACED (the better thing, cite) / INTERNAL (compiler internals or plan,
   not a language feature) / RESTORED (added back to H1). Plus mechanical inventories diffed 0.9.8 vs
   0.9.9: grammar productions, keywords, attributes, CLI commands/flags, manifest keys, std modules
   and names, error codes. Record in `passes/PASS0-feature-parity.md`; restore anything missing.
1. **Memory-safety pass** (loopholes / failures) → its own findings document
   `tasks/spec-0.9.9/passes/PASS1-memory-safety.md`.
2. **Consistency / C-like-speed pass** → `tasks/spec-0.9.9/passes/PASS2-consistency-speed.md`.
3. **Ergonomics pass** → `tasks/spec-0.9.9/passes/PASS3-ergonomics.md`.
4. From the three: the most reasonable, coherent **change list** (`passes/CHANGES-H2.md`), then write
   **0.9.9_Hardened_2** (`docs/spec-source/Ember_v0.9.9_Hardened_2.md`; H1 stays as built).
5. **Sanity check on H2** (`passes/SANITY-H2.md`): re-run the end checks + a read-through.

Passes are solo (no workflow unless asked with an agent count). Status of these steps is tracked in
the progress log below.

## Where things are

| Path | What |
|---|---|
| `tasks/spec-0.9.9/parts/pNN-*.md` | the document, one file per Part, in order |
| `tasks/spec-0.9.9/HANDOFF.md` | this file |
| `tasks/spec-0.9.9/categorize.py`, `titles.tsv` | finding → category data (script's `SP` path points at the session scratchpad; re-point if rerun) |
| final output (not yet built) | `docs/spec-source/Ember_v0.9.9_Hardened_1.md` = concatenation of parts |

Build: `cat tasks/spec-0.9.9/parts/p*.md > docs/spec-source/Ember_v0.9.9_Hardened_1.md`.
**Do not commit** (owner commits only when asked). Do not touch the six uncommitted
`compiler/ember_analysis/src/*.rs` files in the main tree (someone else's work).

## Plan of the document (Parts → files)

| File | Part | Status |
|---|---|---|
| p00-front.md | front matter, goal, reading guide, versioning (`VER-1..4,8,9`) | **done** |
| p01-overview.md | I overview, principles (PHIL), Safe Ember invariant, tiers, showcase | **done** |
| p02-lexical.md | II lexical | **done** |
| p03-grammar.md | III grammar (complete EBNF) | **done** |
| p04-types.md | IV types | **done** |
| p05-declarations.md | V declarations | **done** |
| p06-expressions.md | VI expressions, closures, generators | **done** |
| p07-ownership.md | VII ownership, borrowing, regions, views | **done** |
| p08-classes.md | VIII classes & RC | **done** |
| p09-memory.md | IX memory facilities | **done** |
| p10-effects.md | X effects, contracts, determinism, cost model | **done** |
| p11-concurrency.md | XI concurrency | **done** |
| p12-dod.md | XII SoA, SIMD, ECS | **done** |
| p13-errors.md | XIII errors | **done** |
| p14-comptime.md | XIV comptime, reflection, derives | **done** |
| p15-stdlib.md | XV standard library surface | **done** |
| p16-ffi.md | XVI FFI (C) | **done** |
| p17-toolchain.md | XVII toolchain, diagnostics, conformance | **done** |
| p18-implementation.md | XVIII requirements on implementations | **done** |
| p19-annex-a.md | Annex A quick reference | **done** |
| p20-annex-b.md | Annex B hot reload | **done** |
| p21-annex-c.md | Annex C C++ interop | **done** |
| p22-annex-d.md | Annex D GPU host model | **done** |
| p23-appx-e.md | Appendix E coming from Python | **done** |
| p24-appx-f.md | Appendix F glossary | todo |
| p25-appx-g.md | Appendix G findings → resolutions (all F-001..F-214) | todo |
| p26-appx-h.md | Appendix H changes from 0.9.8_H3 | todo |
| p27-appx-i.md | Appendix I rule index (generate by script at the end) | todo |

## Checks to run at the end

0. **New-ID check:** every id marked *(new in 0.9.9)* MUST NOT exist anywhere in
   `docs/spec-source/Ember_v0.9.8_Hardened_3.md` (FFI-43 and IMP-1 collisions were caught by hand);
   every unmarked id MUST exist there (else it is new and needs the marker).

1. Rule-id uniqueness (each `[XXX-n]` defined once).
2. Every cited rule id is defined (or listed as retired in Appendix H).
3. Every F-001..F-214 appears in Appendix G.
4. Every ` ```ember ` block: run `ember check --syntax-only` with the clean build
   (`<scratchpad>/ember-clean/target/debug/ember.exe`) — failures are expected only for
   constructs new in 0.9.9 (list them); anything else is a mistake in the example.
5. Read-through for contradictions (solo). Workflow only if truly needed — ask first with count.

## Rule-ID policy

Keep 0.9.8 ids where the rule survives (mark *(changed in 0.9.9)* if meaning changed). New rules take
numbers above 0.9.8's max per family: ABI:5 ALC:4 ARN:13 ATT:5 BEN:8 BLD:13 BRW:9 CELL:12 CG-C:10
CLI:18 CLO:7 CLS:9 CORO:11 COST:5 CT:5 CTL:9 DET:9 DIA:19 DRP:6 DSJ:9 ECS:7 EFF:22 ENM:4 ERR:8 EXC:14
EXP:7 FFI:48 FN:8 GRM:23 HASH:4 HEAP:7 HND:2 HR:43 IFC:4 LEX:22 LNT:5 LT:43 MAN:7 MNG:5 MOD:7 OBJ:5
OWN:8 PAN:3 PAR:4 PHIL:11 PRF:2 RC:6 RNG:10 RT:9 SEL:2 SIMD:6 SOA:5 SPN:10 STA:2 STD:8 STR:5 THR:7
TST:26 TXT:8 TYP:26 UNS:10 VER:7 WK:14. **Allocated so far (new):** VER-8, VER-9, PHIL-12, PHIL-13,
PHIL-14, PHIL-15, LEX-23, LEX-24, LEX-25, GRM-24..GRM-34 (GRM-33/34 used), CTL-10 (planned p06), TYP-28 (int `/`), TYP-31 (negative index), TYP-32 (`some`), TXT-9 (literal→String), CLO-10 (callable inline capacity), CT-6 (`comptime(e)`), DIA-20 (error-token runs).
**New diagnostic codes so far:** E0008 (unterminated char), E0110 (bodiless fn), E0900 (not yet implemented), E0901 (unspecified), E1060 (unknown module), E2011 (negative literal index), E2240 (int `/`), E2250 (bad format spec), E2260 (`some` outside return).

## THE DECISIONS (binding for the whole draft — keep every Part consistent with these)

### Safety (C1)
1. **Sync classes**: every field of a `Sync` class is implicitly `let` (assignable only in `init`);
   each field's type must be `Sync`; mutation only through `Atomic`/`Mutex`/`RwLock`/channels. So
   no plain data race is expressible; no access word needed on Sync objects. (F-078, F-104)
2. **Assigning a non-`Copy` field through a class handle is a write access** (dynamically checked
   against live long-term accesses). Scalar/`Copy` field reads/writes stay instantaneous. (F-079)
3. **No profile or manifest key removes a safety check.** `exclusivity = "unchecked"` and
   `gpu.validate = false` are deleted. The only opt-outs are `unsafe` APIs. (F-130, F-147)
4. Runtime: checked capacity arithmetic; element alignment honoured. (F-190, F-191)
5. **Class methods do not need `mut self` to mutate fields** (reference semantics like Python);
   each field access is checked individually; `mut self` on a class method means "hold one write
   access for the whole call". Borrowed class-handle parameters may be written through. (F-073, F-081)

### Correctness of generated code (C2–C4)
6. **Integer overflow panics in every profile.** `@overflow(wrap)` on a fn/module and
   `wrapping_*`/`checked_*`/`saturating_*` methods opt out. Profiles never change meaning — `[PRF-1]`
   has no exceptions any more. C backend emits UB-free arithmetic. (F-102, UB-1..5)
7. FP contraction always off except explicit `fma`/`@fp(contract)`; C backend passes the flags. (F-139)
8. **No silent acceptance** (`[PHIL-12]`): every attribute, import, directive and construct either has
   its specified effect or is rejected (`E0900` not-yet-implemented, `E0901` unspecified, `E0104`
   unknown attribute, `E1060` unknown module). (F-004, F-123, F-161, F-192)
9. Accepted programs never reach a C error; mangling is injective (length-prefixed); an internal
   compiler error is always a defect. (F-012, F-136, F-185)

### Numbers (C9/C11)
10. `int` = `i64`, `float` = `f64` (prelude aliases). Untyped int literal defaults to `int`, float
    literal to `float`; literals take their type from context first (so engine f32 math is unchanged).
    Reverses 0.9.8's f32 default. (F-037)
11. **Sizes and indices are `int`.** `len()` returns `int`; indexing takes any integer type; negative
    or ≥ len panics. `usize` stays for FFI/raw memory. A negative *literal* index is a compile error
    with `xs.last()`/`xs[xs.len() - 1]` fix-its. (F-148)
12. **Division:** `/` is true division on floats only; on two integers it is `E2240` with fix-its `//`
    or float conversion. `//` = floor division (Python), `%` = floor modulo (sign of divisor, Python).
    `div_trunc`/`rem_trunc` give C semantics. `MIN // -1` panics (overflow); `x % -1 == 0`. (F-155,
    F-065, F-213)
13. Shifts: amount must be in `0..width`, else panic in every profile; bits shifted out are not an
    overflow; `>>` is arithmetic on signed. C emits unsigned shifts. (F-033)
14. `as` on numbers: int→int truncates; float→int saturates, NaN→0; unary minus binds tighter than
    `as`. (F-019)
15. `**` integer power: negative exponent panics (compile error if constant); overflow panics. (F-066)

### Python-feel (C9)
16. Literals: `[a, b]` → `Array[T]` unless context wants `[T; N]`/`Span`; `{k: v}` → `Map`; `{a, b}`
    → `Set`; `{}` needs a type. List/set/map **comprehensions** (sugar over iterator adapters). (F-209)
17. **String literal initialises a `String`** at any coercion site (allocation visible at the
    literal). `String + str` concatenation; `str.len()` bytes; `Array[str]` of static literals legal.
    (F-031, F-032)
18. **Chained comparisons** Python-style (middle operand evaluated once); `in` stays non-assoc. (F-049)
19. `x is None` / `x is not None` legal on `Option`. No truthiness (`if xs:` stays an error + fix-it).
20. Unparenthesised tuples on assignment RHS and `return a, b`. (F-068)
21. **A name assigned in every arm** of an exhaustive `if/elif/else` or `match` at the same type is
    declared in the enclosing block. (F-014)
22. `Map`/`Set` are **insertion-ordered** (deterministic; not `Nondet`). Fixed-seed hashing;
    `RandomState` hasher optional for untrusted keys. (F-093, F-094)
23. Structs/enums auto-derive `Eq`, `Debug`, `Clone` when all fields support them; `@no_derive(...)`
    opts out; `Copy`, `Ord`, `Hash`, `Default` stay explicit. (F-054)
24. `static` may have a run-time initialiser: lazy, thread-safe, once. (F-055, F-109)
25. **Generators**: `gen fn f(...) -> Generator[Y]` implements `Iterator[Item = Y]`; frame is a sized
    value (no allocation). `some I` opaque return types. (F-074, F-063)
26. `Result[T, E = Error]`: `Result[T]` means any-error (`Error` = boxed `dyn` error). (F-113)
27. `print`/`println` accept one or more `Display` args separated by spaces (compiler-known, the only
    variadic calls); `sep=`, `end=` named args. Non-`Display` arg is a type error. (F-012, F-119)
28. Calling a callable field `b.on_click()` works. (F-060)
29. Floats implement `Eq`/`Ord`: operators are IEEE; `Ord.cmp` is IEEE totalOrder (so `sort()`
    works); floats are not `Hash`. No `PartialEq`/`PartialOrd`. (F-038, F-043)
30. Prelude (authoritative list in §V.1) includes `Map`, `Set`, `min`, `max`, `abs`, `clamp`, `mem`,
    `Cell`, `RefCell`, `RangeError`, `int`, `float`, `Error`, `Ordering`, … (F-030, F-047, F-048, F-122)

### Removing Rust-style friction (C10)
31. **No `::`**; `.` everywhere. (F-018)
32. **No named lifetimes, ever**; `'ident` is not a token. (F-025)
33. `@view` optional (inferred). (F-036)
34. **Coherence per package**: an `extend T implements I` may appear in any module of the package
    that declares `T` or `I`. Interface methods resolve without importing the interface when exactly
    one visible impl provides them. (F-040, F-041, F-052, F-174)
35. **`with_views*` and `@latebound` removed.** Callable types elide regions like function signatures
    (Rust's `Fn(&T)` behaviour): each call gets fresh regions for borrowed/view params; result may
    borrow from args per `[LT-1]`. (F-168, F-069)
36. **Callable types**: `fn(A) -> R` in a parameter = zero-cost generic bound; in any other position
    (field, local, return, element) = an owned callable value (capture-free: code pointer; small
    captures inline; else boxed, `Alloc` reported). `once fn(A) -> R` for single-use (callable through
    a box). Parameter modes (`mut`, `owned`) allowed in fn types. No `Callable[Args,R]` in source. (F-058,
    F-059, F-061)
37. **Private methods borrow only the fields they touch** (non-`pub`-outside-package, non-virtual):
    receiver borrow is field-sensitive from the method's access summary. (F-067)
38. Arena elision: a view returned from a function whose only view-producing parameter is one
    `Arena` borrows that arena — no `@borrows`. (F-070)
39. Contracts on generic fns checked per instantiation for statically-dispatched callables. (F-095, F-143)
40. Iteration interfaces: `Iterator`, `Iterable` (`iter`), `IntoIterator` (owned); `iter_mut` is a
    method convention, not an interface. (F-045)
41. ECS `Query[(Mut[Position], Velocity)]` with ordinary marker type `Mut[T]` — no access-mode
    generic kind. (F-024)
42. Pre-1.0: no language-version selectors (`[VER-8]`). (F-046)

### Other semantics (C11–C13)
43. `if`-condition temporaries drop before `else`. (F-064)
44. `assert_disjoint(a, b) -> Result[(A, B), (A, B)]` (views returned on failure). (F-090)
45. Class `gen fn` methods take `self` (handle retained by the frame); no class long-term access may
    span a `yield`. (F-062)
46. `FixedArena` is the `@noalloc` arena type; `Arena.fixed` returns it. (F-086)
47. `# SAFETY:` line comment immediately before/on an `unsafe` line is recognised; `[LEX-11]` has
    that exception. `unsafe` blocks carry no separate reason syntax (category goes in the comment:
    `# SAFETY(ffi): …`). (F-089, F-120)
48. Stale-handle checks in every profile. (F-101)
49. Comptime may allocate (interpreter heap); results in statics are immutable; expression form
    is `comptime(e)` (GRM-32). (F-108, F-109, F-110)
50. `SoA[T]` is a compiler-known type constructor. (F-112)
51. C++ exceptions: importer derives `noexcept`; potentially-throwing → `Result[T, CppError]` by
    default (there IS a default). (F-157)
52. Old hot-reload images never unloaded; epochs reclaim data only. (F-176)
53. `fn main(args: Array[String])` args lossy-decoded (U+FFFD); `std.process.args_os()` raw. (F-056)
54. Phantom type params legal. (F-092)
55. Control flow inside `with scope = thread.scope():` leaves the enclosing function after the join.
    (F-106)
56. Derived class with no `init` inherits base `init` params; derived fields need defaults. (F-134)
57. `Pool` handles: 64-bit (32 index / 32 generation) default; exhausted slot retires; all retired →
    `CapacityError`. `Entity` keeps 20/12 for RageV. (F-175)
58. Count contracts (FFI): `one`, `count(param)`, `len_of(param)`, `nul_terminated`, `fixed(N)`. (F-128)
59. `migrate_from` integer arithmetic overflow → `ReloadError.Migration`. (F-133)
60. Leak (cycle) report on by default in `debug` runs; `with h:` pin + lint for early deinit. (F-082, F-083)
61. `True`/`False`/`len(x)`/`append`/`range()`/`def`/`xs[a:b]`/`with e as x` etc. → fix-its (not
    accepted). (F-198)
62. RageV integration plan and compiler-architecture internals are **out of this document** (Part
    XVIII keeps only user-visible obligations). C++ interop, hot reload, GPU → annexes.

### Added while writing (binding too)
63. **`Sync` classes are explicit**: only `@sync class` is `Sync`/`Send`; all other classes are neither
    (plain counts, whatever their fields). `@sync` fields immutable after `init` (E7003); no `mut self`
    methods on them. `@thread_local` retired. (THR-1/THR-2, p11)
64. **Copy capture fallback** (CLO-13): a read-only capture of a `Copy` variable whose storage would end
    while the closure lives, and that is never assigned afterwards, is copied — so loop variables work in
    spawned closures without `owned fn`.
65. Scoped tasks are plain `with` blocks: the `Scope`'s drop joins; `return`/`break`/`?` keep their
    meaning (THR-5, F-106). Two dependent job phases = two scopes (THR-12). JOB-3/JOB-4 debug-only
    access-set verification REMOVED (profile-dependent safety); `submit_after` only orders.
66. `@parallel` results independent of worker count (PAR-5: fixed chunk boundaries, chunk-order
    reduction) ⇒ not `Nondet`. Job system started before `main` from the manifest; `@parallel` has no
    `Alloc` (PAR-4).
67. `Atomic.fetch_add` panics on overflow (after the store); `checked_fetch_add`, `wrapping_fetch_add`.
    `join()` returns `R` (panics abort, no payload).

68. `SoA[T]` for any struct, no derive; columns are disjoint places, so `columns_mut` and SOA-5's
    field-name argument kind are RETIRED; proxies `SoARef[T]`/`SoAMut[T]`; one heap block, 64-byte
    aligned columns (SOA-7). Vector integer lanes panic on overflow like scalars (SIMD-8).
69. **Grouped overflow checks** (SIMD-7): in loops without Sync/Io/FFI/Unsafe/Block, overflow may be
    checked once per vector group (not observable: private writes + abort); checked integer reductions
    vectorise only when no partial sum can overflow (versioned on trip count). This is what keeps
    overflow-panics-everywhere at C speed.

70. **Refines 26:** the any-error type is **`AnyError`** (prelude struct: boxed `dyn Error` + context
    chain; does NOT implement `Error`); `Error` stays the interface. `Result[T, E = AnyError]`.
    `.context(msg)`; main prints the `caused by:` chain (ERR-12). `?` Option↔Result mismatch = E2181.

71. Comptime: no `comptime fn` marker; allowed effects ⊆ {Alloc, Panic(Explicit), RuntimeCheck} (F-108).
    Heap results: read-only static data only where no `ref mut` path exists; interior-mutable statics
    init at run time; `comptime(e)` of a heap type clones per evaluation (F-109). User attributes are
    `@attribute` structs, typed like constructor calls (RFL-3). Debug format is Python-dataclass style
    `Point(x=1.0, y=2.0)`. Derive `SoA`/`PartialOrd` gone. Serialize generic over Writer (SER-1/2 new).

72. **Stdlib naming**: one `snake_case` convention (`starts_with`, `push`, `trim`, `lines`, `find ->
    Option`); Python's differing names are NOT aliases — `E2072` no-method with a fix-it (Appx E table).
    Python names kept only with Python meaning (`partition`, `items`, `count`). `split()` no-arg is
    `split_whitespace()`. `Map`: `m[k]` panics on missing, `get_or(k, d)`, `m[k] = v` via `IndexSet`
    (STD-17), `str` keys via `AsKey` (STD-12). `round()` is half-to-even. `input()` panics at EOF
    (console failures fatal, STD-10).

73. FFI: scalar-only functions in an `unsafe extern` block are safe to call (FFI-10, F-123).
    Overlay grammar GRM-35 (F-126): contract words space-separated in type position; contextual
    keywords `overlay`/`rename`/`hide`/`c`. Count axis: `one`, `count(n)`, `nul_terminated`,
    `fixed(N)`, `inout_count(p)` (F-128; `len_of` folded into `count`). Keyword-named C identifiers use
    `r#name`. TCB/instrumentation grades (FFI-37*) and all C++ rules → Annex C. `cstr` (not CStr).

74. **Borrow checking is specified exactly and location-sensitively** (BCK-1..7, §XVIII.4, new family
    BCK): loans flow with values; a loan is live only on paths to a later use; a returned loan is live
    only on paths reaching that return ⇒ NLL problem case #3 (conditional return of a borrow) is
    ACCEPTED (BCK-6 example). Same acceptance set for every implementation (PHIL-13).
75. Mangling is length-prefixed (MNG-1, F-136); hash collision = E9040 not ICE. Runtime counting fast
    path inline in ember_rt.h, @sync retain = one fetch_add (RT-10, F-193); system malloc (RT-1, F-194).
    `@fastmath` functions go to a separate TU with relaxed flags (CG-C-11). Shared generics only above
    the instantiation ceiling (MONO-6).
76. `ref e` / `ref mut e` expressions were missing from the grammar — added (GRM-36, E0111 non-place).

77. `migrate_from`: every run-time check failure inside it becomes `ReloadError.Migration` (HR-35,
    generalises decision 59); explicit panics there are E2225. GPU stale/in-flight checks every profile.
    C++ exception default: provably-noexcept called directly, else `Result[T, CppError]` (FFI-24, F-157).

## Progress log

* 2026-09-23: decisions fixed (above); p00 written.
* p01 (PHIL-12 no silent acceptance, PHIL-13 one meaning, PHIL-14 Python spelling=Python meaning, PHIL-15 costs named), p02 (keywords unchanged 49; contextual abstract/from/gen/once/some; future async/await/macro/union; `as?`/`as!` single tokens; never type is `Never`, not `!`), p03 (full grammar; precedence: as below */ and above unary; `**` above unary; comparisons chain; map/set literals; comprehensions; local fns; `comptime(e)`) written. 
* p04 types done: TYP-27 void/(), TYP-28 `/` `//` `%`, TYP-29 float //,%, TYP-30 `**`, TYP-31 int indices, TYP-32 `some`, TYP-34 view inferred, TYP-35 phantom, TYP-36 interface table, TYP-37 float Eq/Ord, HASH-2 fixed seed + RandomState; TYP-5 = complete coercion list. Forward refs still to DEFINE later: TYP-38 (collection literal typing → p06), TXT-9 (p15), CLO-10/CLO-11 (p06), CTL-10 (p06), CT-6 (p14), CORO-3 (p06), OPT-2 (p07/p10), CG-C-1/CG-C-11 (p18), MONO-2 (p17), STR-5 (p05), STD-9/STD-11 (p15), SPN-1 (p07), THR-1 (p11), OBJ-2/CLS-7 (p05/p08), DIA-20 (p17), UNS-8 (p09), E2261 (some mismatch). 
* p05 done: MOD-3 import binds last seg + E1060/E1061, MOD-8 glob imports, MOD-5 authoritative prelude table, FN-6 callable types (modes; fresh regions per call via LT-7), FN-8 main(args: Array[String]) lossy, FN-9 handle params, STR-2 defaults any expr per construction, STR-5 implicit Eq/Debug/Clone + @no_derive, STR-6 struct init, CLS-7 class methods mutate via self (no mut), CLS-10 init inheritance, STA-1/STA-3 lazy statics (T: Sync, E7002), ATT-1/4/6 + full attribute table (LAY-2 for layout attrs → define in p09).
* p06 done: EXP-2 swap, EXP-4 cond temporaries, EXP-9 is None, TYP-38 collection literals, CTL-1 for rules (Map yields (k,v), str yields chars), CTL-10 hoisting (E2230 type mismatch), CLO-3 position-dependent fn types, CLO-10 3-word inline captures, CLO-6/6a once fn, CLO-11 callable fields, CLO-12 local fns, CORO-1..12 generators (Generator[Y,R] opaque frame implementing Iterator; CORO-6 frame-local borrows across yield forbidden, param views allowed; CORO-12 class gen methods E2229), PAN-1..3. Added TYP-39 (collections Display like Python str()).
* **TODO for later Parts:** p15 must define Map indexing: `m[k]` read panics on missing key (KeyError-like), `m[k] = v` insert-or-replace via interface `IndexSet[Idx, V]` (add to IV.8 list + prelude), `Map[String, V]` accepts `str` keys for lookup/insert; `Iterator.sum()`, `.copied()`, `split()` on str returning iterator of str; `format`, `input`. p15 must define TXT-9, STD-8 Contains, STD-9 print/println variadic, STD-11 ordered Map. 
* p07 done: BRW-3 two-phase (live borrow at activation = E3021), BRW-10 field-sensitive private methods, BRW-11 all call borrows live together, LT-1/1a/1b elision + @borrows, LT-4/LT-44 arena elision, LT-6 no lifetimes ever, LT-7 callable fresh regions, LT-8..13 RETIRED (with_views), LT-14..43 multi-region condensed (kept ids 14,16,17,18,20,21,22,23,24,25,26,27,30,34,35,36,38,39,42,43; LT-15/19/28/29/31/31a/32/33/37/40/41 referenced or folded — list retired/folded ones in Appx H), DRP-7 dropck, SPN-5 split_at only. Refers to §XVIII.4 for the NLL algorithm → p18 must have XVIII.4. 
* p08 done: OBJ-1..5, RC-1..6 (+RC-2e loop handles; L3019 early-deinit lint), EXC-1/2 every profile, EXC-16 non-Copy field assign = write access, EXC-8..12 hoisting split into bullets, EXC-14 unchecked mode removed, EXC-15 mut self whole-object access, DSP-1..5, WK-1..6,9,11..15 (WK-15 leak report default in debug; WK-7/8/10 folded), OPT-1. Tree example (parent Weak + init-less class). 
* p09 done: SEL-1/2 table, HEAP-1..9 (HEAP-8 checked capacity, HEAP-9 alignment), ARN-1..8,10,11 (ARN-4 FixedArena type; ARN-5 condensed a..g; ARN-9/12/13 folded into ARN-8), ALC-1..4 (ALC-3 manifest global_allocator), UNS-1..8,10 (UNS-8 SAFETY comment note, UNS-9 category folded into note), LAY-2 layout attrs honoured, HND-1..3 (64-bit handles, Split20 for RageV, retirement), CELL-1..10,12 (CELL-11 folded into MOD-5), DSJ-1..9 (DSJ-1 Result). Next: p10 effects.
* p10 done: EFF-1..4,9..11,18 effects (EFF-2 callable params per instantiation; EFF-9 four RuntimeCheck kinds Aliasing/Bounds/Stale/Arithmetic); contracts table + EFF-5,6,6a,7,8,12..17,19..21, EFF-23 (NEW: unbuilt contract = E0900); EFF-15 one verdict in every profile; EFF-16 @nopanic(explicit) allows RuntimeCheck; DET-1..7 + DET-10 (NEW: FP environment); Map order not Nondet; COST-1..5 table (overflow + stale checks every profile, floor // cost, literal allocations, owned callable, generator); OPT-2 loop versioning + OPT-3 (NEW). Split inline definitions to bullets (WK-14, TYP-11/12/14/22, CLS-9a) so the index script can take "bullet-leading `[ID]`" as THE definition form. Forward refs still to define: MONO-6/MONO-7 (p17/p18), STD-4 NonZero (p15), BLD-13 (p17), RT-7/RT-8 (p18), RFL-2/DRV-1 (p14), THR-6 (p11).
* **Definition convention (for the end checks):** a rule is defined where `* \`[ID]\`` starts a bullet (or `[ID]` starts an indented continuation in a numbered list). Everything else is a citation.
* p11 done: THR-8 Send, THR-9 Sync (new), THR-1 explicit @sync + immutable fields (E7001/E7003), THR-2, THR-7, THR-13 data-race guarantee (new), THR-10 spawn/join (new), THR-3 Mutex/RwLock/Once, THR-14 Atomic (new, E7006 bad order), THR-15 channels (new), THR-4, THR-6, THR-5 scope-as-with, THR-11 spawn borrows (new; E7004 not Send, E7005 not Sync), THR-12 two scopes (new), PAR-1..5 (PAR-5 new), JOB-1,2,3(changed),5; JOB-4 retired. Added CLO-13 to p06. p05 attribute table: `@thread_local` removed. New codes: E7003, E7004, E7005, E7006.
* p12 done: SOA-1 (compiler-known; E2071 non-struct), SOA-2 columns as places, SOA-3, SOA-4 ArenaSoA, SOA-6 proxies (new), SOA-7 one block + 64B alignment (new); SOA-5 RETIRED. SIMD-1..6 revised, SIMD-7 grouped overflow (new), SIMD-8 lane rules (new); p10 cost row updated. ECS-1..7 (ECS-3 Mut[T] marker, no access kind — F-024; ECS-4 compile-time sets, E7020).
* p13 done: ERR-1 (changed), ERR-2 (E2180 kept, E2181 new), ERR-3 derive(Error) with variant-level `@from` (F-111 fix), ERR-4, ERR-5, ERR-6, ERR-7, ERR-8 AnyError, ERR-9..13 new (default E, context, cost, main chain, panic-vs-error convention). p05 prelude + p10 cost row updated. Forward refs: DIA-21 (Python try/except/raise fix-its) → p17; Map str-key insert, `str.partition/lines/parse`, `fs.read_to_string` → p15.
* p14 done: CT-1..5 changed, CT-6 comptime(e), CT-7 comptime blocks (new; E6003 unavailable op, E6004 compile-time panic, E6005 run-time local), RFL-1..3, DRV-1 table (spec text is the reference — F-114), DRV-2, SER-1/2 (new family SER).
* p15 done: STD-1..8b kept/changed, new STD-9 print, STD-10 input, STD-11 ordered Map, STD-12 AsKey, STD-13 naming (E2072), STD-14, STD-15 Array, STD-16 Map ops, STD-17 IndexSet, STD-18 format, STD-19 iterators, STD-20 numbers, STD-21 math, STD-22 io, STD-23 time, STD-24 process, STD-25 random; TXT-1..8 (TXT-4 every profile), TXT-9..11 new. IndexSet added to p04 sketch + p05 prelude; p06 example uses get_or; p11 uses split_whitespace. TODO for p23 (Appx E): the Python-name → Ember-name table referenced by STD-13.
* p16 done: FFI-1,2,2a,4,5,5a,6,6a,6b,7,8 table,9,10(changed),11(changed: 5 axes),11a,11b,11c,12,13,14,15,16,20a,21,22,23,25(changed),26,27,28,29,29a,29b,29c,30,30a,30b,31,31a,31b,33,35a,36a,36b, FFI-49 (new link_name; FFI-43 is 0.9.8 C++ exception policy → Annex C); GRM-35 overlay grammar (in p16; also list in p03? — p03 references it via LEX-15); BLD-FFI-1,1a,3. p02 LEX-15 contextual keywords extended. p15 CStr→cstr. Moved to Annex C (p21): FFI-3, 17*, 24, 32*, 34, 36*, 37*, 38, 39*, 40-42, BLD-FFI-1b/2/4/5, CXX-*, TCB-*.
* 2026-09-23: owner added the post-H1 plan (3 passes → change list → H2 → sanity check), recorded at the top.
* p17 done: CLI-1,4,5,6,7,9,10 + CLI-19 matrix (new; unimplemented = E0900); MAN-1,2,3,7,8(new),VER-5; BLD-1,2,6,8..11,13; PRF-1 (no exceptions), PRF-2, PRF-3 profile table (new); TST-0..4a,5,6,7(changed: ember check, `ember,fragment` marker),8,9,10 + TST-27 C gate, TST-28 perf gate, TST-29 honest baselines, TST-30 all failures/quarantine (new); DIA-1..7,9,12,14,16 + DIA-20..24 (new: byte runs, Python habits table, caller-site, Ember names in panics, N1 threshold ≤1 for ≤4 chars); shapes tables O1-O9,B1-B12,B15,X1,S1,A1,R1,N1-N12; code ranges; FMT-1 LF (new), FMT-2; LNT-1..5; DOC-1,2; CONF-1..5; ABI-1; VER-4. NOTE: `ember,fragment` block marker introduced by TST-7 — use it in the end checks for examples that name undeclared items.
* p18 done: CG-C-1,2 (changed), CG-C-3,3a,3b,4..10, CG-C-11 FP flags (new); MNG-1 (changed, injective), MNG-2..4; MONO-1,2,3,5,6,7,8; BCK-1..7 (new family, §XVIII.4 — p07 LT-5 points here); RT-1 (changed), RT-2,3,4,6,7 (changed), RT-8, RT-10, RT-11 (new); IMP-11 (new; IMP-1..10 are 0.9.8 plan rules, retired). New codes: E9040, E0111. p03: GRM-36 ref expressions + precedence row.
* p19 done: Annex A rewritten for 0.9.9 syntax (marked `ember,fragment`; no `#! language`). TODO at end checks: update p00 "Examples" paragraph to mention `ember,fragment` (TST-7).
* p20/p21/p22 done (Annexes B, C, D). `tools/split_inline.py` finds/fixes mid-bullet definitions (run it before the end checks; clean as of p22).
* p23 done (Appendix E: carry-over table, STD-13 name table, what's new).
* 2026-09-23: owner added step 0 — findings-coverage audit of H1 before the passes (recorded at the top).
* p24–p27 done; tools written: check_findings.py (G), appx_h.py (H), appx_i.py (I), check_ids.py, check_citations.py, split_inline.py, check_examples.py. H1 BUILT at docs/spec-source/Ember_v0.9.9_Hardened_1.md (6217 lines, 417 KB) — rebuild with `cat parts/p*.md` after any fix, and regenerate G/H/I first. Checks green: ids (798 defined, 0 problems), citations (0 undefined), findings (214 mapped, 0 problems). check_examples: 10/43 blocks fail the 0.9.8-era parser — classify (new syntax vs mistake); XI.4 block is a real mistake (file-scope loop).
* 2026-09-23: owner added feature-parity requirement (recorded at the top, step 0).
* **STEP 0 DONE (2026-09-23).** `passes/PASS0-findings-coverage.md` (214/214 mapped; S1/S2 rule text read against resolutions via `passes/_s1s2_evidence.md`; 9 gaps fixed) and `passes/PASS0-feature-parity.md` (inventories + 244 retired rules + 212 shrunk rules reviewed; large restoration list applied to H1; replacements table). New tools: `code_registry.py` (generates §XVII.9 into `parts/p17a-codes.md`), `check_examples.py --desugar`. H1 rebuilt: 7.2k lines, 913 rules; ids/citations/registry/findings/examples all green.
* **NEXT: PASS1 memory safety** → `passes/PASS1-memory-safety.md` (loopholes/failures in H1), then PASS2 consistency + C speed, PASS3 ergonomics, then CHANGES-H2, H2, SANITY-H2.
* Rebuild recipe: `python tools/code_registry.py --write; python tools/check_findings.py --write; python tools/appx_h.py; python tools/appx_i.py; cat parts/p*.md > ../../docs/spec-source/Ember_v0.9.9_Hardened_1.md` then run check_ids, check_citations, split_inline, check_examples (needs PYTHONIOENCODING=utf-8).
* **PASSES DONE (2026-09-23):** `passes/PASS1-memory-safety.md` (21: 8 S1), `passes/PASS2-consistency-speed.md` (16 CS + 9 SP), `passes/PASS3-ergonomics.md` (22 E + 4 rejected), `passes/CHANGES-H2.md` (C-01..C-44, the binding H2 change list).
* **H2 IN PROGRESS.** H1 sources frozen in `parts-h1/` (H1 build = docs/spec-source/Ember_v0.9.9_Hardened_1.md, do not overwrite). `parts/` is now the H2 source; build to docs/spec-source/Ember_v0.9.9_Hardened_2.md. Apply C-01..C-44 in order; tick them in the H2 progress line below; add Appendix H §H.4 (H1 → H2 by rule) and retitle front matter to 0.9.9_Hardened_2.
* **H2 BUILT (2026-09-23):** C-01..C-44 all applied; docs/spec-source/Ember_v0.9.9_Hardened_2.md (7.6k lines, 926 rules: 13 added, 68 changed vs H1). New codes E2073, E2102, E2231 (E2221 was taken), W1003, L4003 (in code_registry.py NEW). appx_h.py now also writes §H.4 (hand table of language changes + generated H1->H2 rule diff against parts-h1/). check_examples.py desugars `safe fn`, generator expressions and top-level statements (lift_script). All checks green: ids 0 problems, citations 0 undefined, registry 237 live/0 problems, findings 214/0, examples 46/0 failing.
* H2 rebuild recipe: same as above but `cat parts/p*.md > ../../docs/spec-source/Ember_v0.9.9_Hardened_2.md`.
* **SANITY-H2 DONE (2026-09-23):** `passes/SANITY-H2.md`. 14 defects found and fixed (SH-01 `unsafe overlay` grammar + E5067; SH-02 `mut self` marks the dynamic class's field words; SH-03..14 smaller). Final checks: 926 rules/0 problems, citations 0 undefined, registry 238 live/0, findings 214/0, examples 46/0, every H1->H2 text change marked. H2 = docs/spec-source/Ember_v0.9.9_Hardened_2.md is now the frozen 0.9.9 target; copy parts/ to parts-h2/ before any H3 edit.
* **OWNER 2026-09-23 — NEXT PHASE: implement 0.9.9 in the compiler.** Ambiguities found while implementing -> ODR-021, ODR-022, ... in `docs/OWNER-QUEUE.md` style; I decide each (owner delegated it) and cut 0.9.9_Hardened_3, _4, ... (diff against the frozen predecessor, list in Appendix H). Continue until complete or told to stop. Work in a separate worktree/branch (another agent's six uncommitted `compiler/ember_analysis/src/*.rs` files are in the main tree). No commits unless the owner says so. Implementation plan and progress: `tasks/impl-0.9.9/PLAN.md` (to be written).
* **H3 CUT (2026-09-23):** `docs/spec-source/Ember_v0.9.9_Hardened_3.md` (sources frozen in `parts-h3/`; `parts/` stays the working source for H4). Records ODR-021 (float `//`/`%` are Python's exact results, `[TYP-29]`) and ODR-022 (`x = 0` is `int` at the declaration, `[TYP-23]`), both in `docs/OWNER-QUEUE.md` on branch `impl/0.9.9`. `appx_h.py` writes §H.5 (ODR table `ODRS` + generated diff vs `parts-h2/`); for H4, add the ODR rows to `ODRS` and change `H2_PARTS` to the latest frozen predecessor's folder as a new section. All checks green (926 rules). Implementation state lives in `Code/ember-099/docs/MIGRATION-0.9.9.md`.
* **LOCATION (2026-09-23):** this folder is committed on `main` (`Code/ember/tasks/spec-0.9.9/`); build hardenings to `Code/ember/docs/spec-source/`. H26 is the latest (ODR-066 to ODR-068, the owner's Part II adoptions: SP-007's const-argument equality, SP-016's `AsKey`/`ToKey` split, SP-031's `get_pair_mut`, 2026-09-26; H25 carried ODR-065, the review of SP-010's access question: a borrow through a handle stored in an object keeps its own copy, 2026-09-26; H24 carried ODR-049 to ODR-064, the owner's simplification pass, `docs/proposals/Ember_Simplification_Pass_Revised.md`, 2026-09-26; H23 carried ODR-048, how an instance of a generic passes and elides its parameters, 2026-09-26; H22 carried ODR-047, `NonZero[T]`'s API and its division, 2026-09-26; H21 carried ODR-046, the form of `std.math.det`'s functions, 2026-09-26; H20 carried ODR-043 to ODR-045: `std.math`'s vectors, matrices, rotations, shapes and `KahanSum` as `[STD-28]`; `[CG-C-11]`'s pragma where the compiler implements it; a type's constants and the order of constants, 2026-09-26; H19 carried ODR-042, blanket implementations, a bound's parents, and the collections' `Index`, 2026-09-26; H18 carried ODR-041, `f16` is a `Number` answering in `f32`, and has the float constants, 2026-09-26; H17 carried ODR-040, the operator interfaces' methods and the numbers' implementations of them, 2026-09-26; H16 carried ODR-039, `[STD-20]`'s integer methods and scalar constants, 2026-09-26; H15 carried ODR-038, owner-ruled: `std.math` takes any number type through `Number` and `T.Real`, 2026-09-25; H14 carried ODR-037, `std.math.Float`; H13 ODR-032 to ODR-036; H12 ODR-031, H11 ODR-030, H10 ODR-029, H9 ODR-028); `parts/` is the working source for H27, and `parts-h26/` is frozen. `appx_h.py` now writes the front matter's Version and Supersedes lines itself. To cut the next: add the ODR row to `ODRS` in `tools/appx_h.py`, run the rebuild recipe above with `cat parts/p*.md > ../../docs/spec-source/Ember_v0.9.9_Hardened_N.md`, copy `parts/` to `parts-hN/`, and repoint `docs/spec-source/development-target.json` (its SHA-256 is of the file's LF bytes).
