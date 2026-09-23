# Deep research findings — 2026-09-23

Written incrementally during a solo read of the whole specification and a probe
of the compiler. Nothing here has been applied. Nothing here edits the spec:
per COLD-START §2 every item is a *proposal* or a *defect record*; spec items
need an owner ruling before any text moves.

**The goal these are judged against (owner, 2026-09-23):** a language that is
**fast like C**, has **types like Python** (readable, light annotation,
inference), and **Rust-like memory safety minus the annoying parts of Rust**.

## Sources

| What | Where / version |
|---|---|
| Spec reviewed | `docs/spec-source/Ember_v0.9.8_Hardened_3.md` (current frozen development target; sha256 in `development-target.json`) |
| Adopted normative | `docs/spec-source/ember-spec.md` = v0.8.5_Hardened_1 (split into `docs/spec/`) |
| Compiler probed | clean worktree at `761ce7c` (the six uncommitted `ember_analysis` edits in the main tree were **not** built or touched) |
| Earlier audit | `tasks/audit/repro/*.em` (40 programs, 24 task ids; their index `tasks/audit/TASKS.md` was never pushed) |

## How to read an entry

Each finding has an id `F-nnn`, and:

* **Kind** — one of
  * `DEFECT` — the rule is clear, the compiler is wrong (goes to `docs/DEFECTS.md`)
  * `ERRATUM` — the document contradicts itself or is ambiguous (goes to `docs/spec-errata.md`; nobody moves until the owner rules)
  * `GAP` — the document is silent where a program can reach (owner question, `docs/OWNER-QUEUE.md`)
  * `DESIGN` — a proposed change to the language against the goal above (owner decision, language version bump)
  * `PERF` — something that stops Ember being as fast as C
  * `TOOLING` / `DOCS` / `PROCESS` — tools, documentation, how the project is run
  * `WITHDRAWN` — kept so the id is never reused; the entry says why
* **Severity** — S1 (unsound / memory-unsafe / miscompile), S2 (wrong result or blocks real programs), S3 (friction, cost, clarity), S4 (cosmetic)
* **Evidence** — the rule quoted by id and line in the reviewed file, and/or a probe program with its observed output
* **Proposal** — what to do; for spec items, the proposed direction only

---

## Summary (read this first)

**213 findings** (ids F-001–F-214; F-069 withdrawn after a probe disproved it) from a solo read of the whole 0.9.8_Hardened_3 specification
(9,390 lines) and about 150 probe programs run against a clean build of
`761ce7c`. Rough split by kind: ~70 `ERRATUM`, ~52 `DEFECT`, ~38 `DESIGN`,
~23 `GAP`, the rest `DOCS`/`TOOLING`/`PROCESS`/`PERF`. Plus the 39 audit
defects (Part D), **38 of which still reproduce** (the 39th is hidden by the
Windows C compiler, not fixed).

All findings are grouped into 16 categories by what fixing them takes — see
**[Findings by category](#findings-by-category)** below the owner questions.

### What matters most, in the order I would do it

1. **Memory-safety holes in Safe Ember.** The 16 SAFE reproducers (Part D);
   F-079 (assigning a non-`Copy` field through a class handle is unclassified
   — a live use-after-free); F-078 + F-104 (the text allows a data race on a
   `Sync` class); F-190/F-191 (runtime buffer arithmetic unchecked; `Array`
   under-aligned for over-aligned types).
2. **Silent miscompiles and UB in generated C.** UB-1..5 (Part D), F-102
   (release uses signed C arithmetic against `[CG-C-1]`), F-139 (float
   contraction never disabled — breaks determinism on ARM).
3. **Things accepted and silently ignored.** F-161 (`@noalloc`,
   `@deterministic`, `@static_safe` and *made-up* attributes all accepted and
   unenforced), F-192 (`@align` ignored), F-004/F-123 (imports of missing
   modules and `import c` silently accepted).
4. **Accepted programs that produce broken C.** F-012/F-184 (`println` of an
   f-string, a `String`, a tuple, an `Option`, a struct or a range type), audit
   CG-2 and FE-3 — and F-013, the missing gate that would catch the class.
5. **Compiler crashes and hangs.** F-185 (a parent/child class pair with a
   constructor overflows the compiler's stack), F-136 (non-injective
   mangling → ICE), audit PERF-2 (two hangs).
6. **A "first programs" milestone** (F-115): F-029 (`Array[i32]()`), F-080
   (`String` construction/concatenation), F-180 (`a if c else b`), F-181
   (`Ord` on scalars), F-182 (`Option.unwrap_or`, `Array.pop`, `str.len`),
   F-203/F-204 (generic self-type; recursive enum traversal), F-058 (closures
   cannot be returned/boxed), F-003 (`import a.b.c`), F-002 (`()` as `void`),
   F-137 (`for x in span`), F-100 (`ember new`, `ember test`).
7. **Python-feel quick wins, mostly diagnostics:** F-198 (help for `True`,
   `len(x)`, `append`, `range`…), F-148–F-150 (`xs[-1]`, `is None`, `let`),
   F-031 (string literal → `String`), F-155 (integer `/`), F-049 (chained
   comparisons), F-014 (names assigned in every branch), F-093 (ordered
   `Map`), F-067 (method calls borrowing only the fields they use).
8. **Simplifications that remove Rust-style friction:** F-168 (retire
   `with_views*` + `@latebound` — identity forwarders whose only effect is to
   reject safe programs), F-046 (eight language-version selectors before
   1.0), F-036 (`@view` required though inferred), F-025 (drop the v2
   named-lifetime reservation), F-070 (`@borrows(arena)` on every wrapper),
   F-018 (`::` vs `.`).
9. **Measure the goal instead of asserting it:** F-140 (no performance suite
   — `tests/perf/` is empty), F-144 (the first-week newcomer corpus
   `[TST-8]` specifies does not exist), F-171 (the per-rule implementation
   matrix the spec requires does not exist), F-085 (half the spec's examples
   do not parse and the gate only checks the old spec).
10. **Process:** F-199 (ledger intake — the ledgers say "none open"), F-200
    (freeze the spec except errata until 1–6 are done), F-173/F-156 (split the
    870 KB spec; write the user guide), F-159 (Phase 1 marked done with
    Part II–VI features missing).

### Questions only the owner can answer

Each of these is written up with evidence in its entry; none has been acted on.

| Finding | Question |
|---|---|
| F-078, F-104 | Does a `Sync` class get an atomic access word, or must its non-`let` fields all be synchronised types? |
| F-130, F-147 | Should `exclusivity = "unchecked"` and `gpu.validate = false` exist, and if so spelled as `unsafe`? |
| F-102 | Release overflow: keep Rust's wrap, or Swift's trap? |
| F-168 | Retire `with_views*` and source-level `@latebound` (revisits ODR-005/006/007/015)? |
| F-046 | Drop language-version selectors until 1.0? |
| F-155, F-065 | Integer `/` and `%`: C semantics, or Python's `//`? |
| F-093 | Insertion-ordered `Map`/`Set`? |
| F-031, F-032 | String literal → `String`; `Array[str]` of literals (`[TYP-15]` vs `[TYP-15a]`)? |
| F-037 | Default integer `i32` or `i64`? |
| F-040, F-052, F-174 | Orphan rule per module or per package? |
| F-043, F-038, F-181 | Which interfaces floats and scalars implement; does `<` go through `PartialOrd`? |
| F-014 | Hoist a name every `if`/`else` arm declares (refines OQ-2)? |
| F-018 | Drop `::` (reopens OQ-15)? |
| F-059, F-061 | Meaning of `fn(A) -> R` outside a parameter; callable boxed once-closures in v1? |
| F-179, F-200 | Decouple 1.0 from the RageV migration; freeze the spec except errata? |
| F-157, F-089, F-101 | Resolve three self-contradictions (C++ exception default, SAFETY comments vs `[LEX-11]`, shipping stale-handle checks) |

### How to use this file later

Per the project's process (`docs/COLD-START.md` §2), nothing here edits the
specification. When work resumes: `DEFECT` entries become open rows in
`docs/DEFECTS.md` (with the probe as the reproducer), `ERRATUM` entries go
to `docs/spec-errata.md`, owner questions go to `docs/OWNER-QUEUE.md`, and
`DESIGN` entries wait for a ruling. The probe programs quoted in entries are
small enough to paste into `tests/` as the failing case first.

<!-- CATEGORY-INDEX-BEGIN (generated; regenerate rather than hand-edit) -->
## Findings by category

Every finding is in exactly one category, chosen by **what fixing it takes**
(who moves, and where the change lands). Kind and severity are the entry's own.
The full evidence for each id is in its entry under *Findings* below.

| # | Category | Count | S1 | S2 | S3 | S4 | Where the fix lands |
|---|---|---|---|---|---|---|---|
| C1 | [Memory safety and soundness](#c1) | 7 | 3 | 3 | 1 | 0 | compiler + spec; owner rulings for the spec items |
| C2 | [Miscompiles and undefined behaviour in the generated C](#c2) | 2 | 0 | 2 | 0 | 0 | compiler (`DEFECTS.md`) |
| C3 | [Accepted but silently ignored or broken](#c3) | 6 | 3 | 3 | 0 | 0 | compiler (`DEFECTS.md`) |
| C4 | [Compiler crashes and hangs](#c4) | 2 | 1 | 1 | 0 | 0 | compiler (`DEFECTS.md`) |
| C5 | [Core language features missing or wrong in the compiler](#c5) | 17 | 0 | 13 | 4 | 0 | compiler (`DEFECTS.md`) |
| C6 | [Standard library to build](#c6) | 8 | 0 | 3 | 5 | 0 | `std/` (`DEFECTS.md` where a rule names the API) |
| C7 | [Diagnostics and error pages](#c7) | 22 | 0 | 2 | 15 | 5 | compiler diagnostics + `docs/errors/` |
| C8 | [Performance](#c8) | 5 | 1 | 1 | 3 | 0 | compiler backend + runtime; needs the perf suite first |
| C9 | [Language design — Python feel](#c9) | 16 | 0 | 6 | 8 | 2 | owner decision (`OWNER-QUEUE.md`), then a language version |
| C10 | [Language design — remove Rust-style friction / simplify](#c10) | 14 | 1 | 2 | 9 | 2 | owner decision, then a language version |
| C11 | [Language design — other semantic choices](#c11) | 6 | 0 | 0 | 5 | 1 | owner decision |
| C12 | [Spec contradictions that need an owner ruling](#c12) | 18 | 0 | 2 | 14 | 2 | `spec-errata.md`; nobody moves until ruled |
| C13 | [Spec gaps — the text is silent](#c13) | 17 | 0 | 2 | 11 | 4 | `OWNER-QUEUE.md` |
| C14 | [Spec editorial fixes — no change of meaning](#c14) | 47 | 0 | 0 | 17 | 30 | hardening-class edits (`spec-amendments.md`) |
| C15 | [Tooling, tests and CI gates](#c15) | 11 | 1 | 5 | 4 | 1 | `tools/`, `tests/`, CI |
| C16 | [Docs and project process](#c16) | 15 | 1 | 5 | 4 | 5 | docs + process (owner for the process items) |
| C17 | [Withdrawn](#c17) | 1 | 0 | 0 | 0 | 0 | — |
| | **Total** | **214** | 11 | 50 | 100 | 52 | |

The 39 audit defects of Part D sit in the same scheme: SAFE-1..6 and RC-1..3 → C1 (RC are leaks, not
unsafety, but they live in the same drop/move code); UB-1..5 → C2; CG-1, CG-2, FE-3 → C3; PERF-2 → C4;
FE-1, FE-2 → C5; DIAG-1..4 → C7.

**Suggested order of work:** C1 → C2 → C3 → C4 (correctness; no owner input needed except C1's
spec items) → C15's gates (so the classes above cannot come back) → C5 + C6 (the first-programs
milestone) → C7 → C8 once the perf suite exists. C9–C13 are one owner session: a batch of
rulings. C14 can be done any time as one hardening. C16 alongside.

### C1
**Memory safety and soundness** — 7 · lands in: compiler + spec; owner rulings for the spec items

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-078 | S1 | ERRATUM | a `Sync` class may have no dynamic exclusivity — the text contradicts itself (data race) |
| F-079 | S1 | GAP | assigning a non-`Copy` field through a handle is neither an "instantaneous" nor a "long-term" access |
| F-104 | S1 | ERRATUM | `[THR-1]` admits a `Sync` class with a plain mutable scalar field (data race) |
| F-130 | S2 | ERRATUM | two package settings turn memory-safety checks off for Safe code, and `[PHIL-10]` lists no such exception |
| F-190 | S2 | DEFECT | growable-buffer arithmetic is unchecked |
| F-191 | S2 | DEFECT | `Array[T]` storage is aligned to 16 bytes whatever `T` needs |
| F-147 | S3 | DESIGN | the example manifest makes shipping builds UB on an exclusivity violation by default |

### C2
**Miscompiles and undefined behaviour in the generated C** — 2 · lands in: compiler (`DEFECTS.md`)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-102 | S2 | PERF | "release wraps and emits nothing" is only true if the C is written so that wrapping is defined |
| F-139 | S2 | DEFECT | float contraction is never disabled |

### C3
**Accepted but silently ignored or broken** — 6 · lands in: compiler (`DEFECTS.md`)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-012 | S1 | DEFECT | `println` of a `String`/f-string/tuple/`Option`/struct emits C that does not compile |
| F-161 | S1 | DEFECT | every contract attribute is accepted and ignored, and so is any made-up attribute |
| F-184 | S1 | DEFECT | range types reach the C compiler broken |
| F-004 | S2 | DEFECT | importing a module that does not exist is silently accepted |
| F-123 | S2 | DEFECT | Ember cannot call C at all today, and the C-import directive is silently ignored |
| F-192 | S2 | DEFECT | `@align(N)` is silently ignored |

### C4
**Compiler crashes and hangs** — 2 · lands in: compiler (`DEFECTS.md`)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-185 | S1 | DEFECT | a parent/child class pair with a constructor overflows the compiler's stack |
| F-136 | S2 | DEFECT | the mangling scheme is not injective, and a collision is an internal compiler error |

### C5
**Core language features missing or wrong in the compiler** — 17 · lands in: compiler (`DEFECTS.md`)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-002 | S2 | DEFECT | `()` is not accepted as a value of type `void` |
| F-003 | S2 | DEFECT | `import a.b.c` does not bind `c` |
| F-029 | S2 | DEFECT | `xs = Array[i32]()` is rejected: the explicit type argument is ignored |
| F-058 | S2 | DEFECT | a closure cannot be returned or boxed; the spec's own form is rejected |
| F-073 | S2 | DEFECT | writing a class field through a handle from outside a method fails |
| F-137 | S2 | DEFECT | `for x in span:` is rejected |
| F-148 | S2 | DEFECT | `xs[-1]` compiles and panics at run time with index 18446744073709551615 |
| F-160 | S2 | DEFECT | indexing does not produce a place that can be returned by reference, so milestone M2 cannot be written as specified |
| F-180 | S2 | DEFECT | the conditional expression `a if c else b` is not implemented |
| F-181 | S2 | DEFECT | built-in scalars do not implement `Ord`, so no generic comparison can be written |
| F-203 | S2 | DEFECT | a generic struct's methods cannot name their own type |
| F-204 | S2 | DEFECT | the textbook recursive enum cannot be traversed |
| F-208 | S2 | DEFECT | range slicing `xs[a..b]` is not implemented |
| F-183 | S3 | DEFECT | a single-statement block lambda inside brackets is rejected |
| F-196 | S3 | DEFECT | redeclaring a name in the same block is accepted |
| F-197 | S3 | GAP | raw strings, byte strings and f-string format specs are not implemented |
| F-205 | S3 | DEFECT | calling a parameter whose type is a generic bounded by `Callable` fails |

### C6
**Standard library to build** — 8 · lands in: `std/` (`DEFECTS.md` where a rule names the API)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-080 | S2 | DEFECT | `String(literal)` does not exist |
| F-088 | S2 | GAP | almost none of the Part IX.1 heap library exists |
| F-182 | S2 | DEFECT | `Option`/`Array`/`str` basics missing (`unwrap_or`, `pop`, `str.len`, iterating `[T; N]`) |
| F-077 | S3 | DEFECT | spec examples call APIs that do not exist (`sum`, `Array[f32]([…])`, `retain`) |
| F-114 | S3 | GAP | `[DRV-1]`: every derive is specified as a hand-written `extend` in `std/derive/*.em`, and the generator "MUST produce the same MIR" |
| F-162 | S3 | DEFECT | milestone M3 fails: `mem` is not in the prelude |
| F-189 | S3 | DEFECT | `Array.iter()` does not exist |
| F-211 | S3 | DEFECT | `enumerate` over an `Array` cannot be written |

### C7
**Diagnostics and error pages** — 22 · lands in: compiler diagnostics + `docs/errors/`

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-198 | S2 | DESIGN | the ten most common Python habits get no help at all |
| F-206 | S2 | GAP | 32 error pages for 210 registered codes |
| F-005 | S3 | DEFECT | every E2020 type mismatch carries a note about numeric conversion |
| F-008 | S3 | DEFECT | one bad character reports once per byte |
| F-009 | S3 | DEFECT | "not implemented in this phase" reuses `E1010` |
| F-010 | S3 | DEFECT | cascades after a failed import |
| F-011 | S3 | DEFECT | cascade after a private constructor |
| F-015 | S3 | DEFECT | `if x = 5:` (a `==` typo) gets a misleading fix |
| F-023 | S3 | DEFECT | `E1020` means two things in the compiler |
| F-082 | S3 | DESIGN | reference cycles leak silently in release, and Python users build cycles |
| F-083 | S3 | DESIGN | objects may die before their last syntactic use |
| F-149 | S3 | DEFECT | `x is None` gives two wrong errors instead of the fix |
| F-150 | S3 | DEFECT | `let x = 5` gives a raw parse error |
| F-169 | S3 | DEFECT | diagnostics that originate in a callee's contract are reported at the callee, not the caller |
| F-186 | S3 | DEFECT | the runtime exclusivity panic names a C symbol, not the Ember object |
| F-187 | S3 | DEFECT | `L3011` fires on calls that cannot re-enter the cell |
| F-188 | S3 | DEFECT | a view escaping its source gets two errors, the first wrong |
| F-006 | S4 | DEFECT | N1 suggestion quality |
| F-098 | S4 | DESIGN | `@realtime` forbids `x / n` via `@nopanic(explicit)`; needs fix-its |
| F-151 | S4 | DEFECT | `a < b < c` fix-it is incomplete |
| F-153 | S4 | ERRATUM | N1's threshold (≤ 2 *or* ≤ ⅓ length) admits `io`→`Eq`, `min`→`main` |
| F-195 | S4 | DEFECT | reference-count overflow reports the wrong reason |

### C8
**Performance** — 5 · lands in: compiler backend + runtime; needs the perf suite first

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-140 | S1 | PERF | there is no performance suite; "fast like C" is unmeasured |
| F-193 | S2 | PERF | every reference-count operation is an out-of-line call, and a `Sync` retain is a CAS loop |
| F-138 | S3 | DEFECT | `[CTL-3b]`'s guaranteed lowering is not implemented |
| F-141 | S3 | DEFECT | several promised backend/runtime pieces do not exist |
| F-194 | S3 | PERF | every allocation goes through `_aligned_malloc` (MSVC) / `aligned_alloc`, and `ember_realloc` always allocates-copies-frees |

### C9
**Language design — Python feel** — 16 · lands in: owner decision (`OWNER-QUEUE.md`), then a language version

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-014 | S2 | DESIGN | a name assigned in every branch of an `if`/`else` is not visible after it |
| F-031 | S2 | DESIGN | strings are the biggest Python-ergonomics gap |
| F-048 | S2 | DESIGN | `Map` and `Set` are not in the prelude |
| F-055 | S2 | DESIGN | no runtime-initialised globals in v1 |
| F-093 | S2 | DESIGN | `Map`/`Set` iterate in unspecified order; Python's `dict` iterates in insertion order |
| F-155 | S2 | DESIGN | integer `/` is the silent Python trap |
| F-038 | S3 | DESIGN | floats have no `Ord`, so sorting a list of floats needs ceremony |
| F-049 | S3 | DESIGN | Python chained comparison is rejected |
| F-054 | S3 | DESIGN | every struct needs `@derive(...)` boilerplate |
| F-060 | S3 | DESIGN | calling a callable field needs a temporary |
| F-068 | S3 | DESIGN | the Python swap `a[i], a[j] = a[j], a[i]` is a parse error |
| F-074 | S3 | GAP | no way to return an iterator without naming its concrete type |
| F-113 | S3 | DESIGN | a default error type would shorten most signatures |
| F-209 | S3 | DESIGN | comprehensions |
| F-065 | S4 | DESIGN | `%` follows C (sign of the dividend) while the syntax follows Python |
| F-210 | S4 | DESIGN | interfaces are nominal only |

### C10
**Language design — remove Rust-style friction / simplify** — 14 · lands in: owner decision, then a language version

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-168 | S1 | DESIGN | `with_views*` and `@latebound`: a chain of features whose net effect is to reject safe programs |
| F-046 | S2 | DESIGN | eight selectable language versions before 1.0 |
| F-067 | S2 | DESIGN | a method call borrows all of `self`, so the classic Rust partial-borrow error is back |
| F-018 | S3 | DESIGN | two path separators, `.` and `::` |
| F-024 | S3 | DESIGN | `[GRM-14]` adds a third kind of generic parameter (`access P`) for one library type, `std.ecs.Query` |
| F-025 | S3 | DESIGN | `[LEX-22]` reserves `'a` for "v2 named lifetimes" |
| F-036 | S3 | DESIGN | `@view` is required on a struct the compiler already knows is a view |
| F-040 | S3 | DEFECT | the orphan rule `[TYP-20]` is not enforced, and as written it is stricter than Rust's |
| F-061 | S3 | DESIGN | a boxed once-callable is not callable (`[CLO-6a]`) |
| F-070 | S3 | DESIGN | `@borrows(arena)` is mandatory on every Arena wrapper, at every nesting level |
| F-087 | S3 | DESIGN | `Shared[T]` overlaps `class` almost completely |
| F-095 | S3 | DESIGN | `[EFF-2]`: a `fn(A) -> R` parameter "is assumed to carry all effects unless written `@noalloc fn(A) -> R`" |
| F-041 | S4 | DEFECT | "interface methods need the interface imported" (`[TYP-24]` step 2) is not enforced |
| F-045 | S4 | DESIGN | three iteration interfaces |

### C11
**Language design — other semantic choices** — 6 · lands in: owner decision

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-019 | S3 | DESIGN | `as` binds tighter than unary minus |
| F-037 | S3 | DESIGN | the default integer is `i32` |
| F-064 | S3 | DESIGN | temporaries in an `if`-condition live through the `else` |
| F-090 | S3 | ERRATUM | the `assert_disjoint` example uses `dst` after moving it |
| F-133 | S3 | DESIGN | `migrate_from` may not contain a single integer `+` |
| F-107 | S4 | DESIGN | async is v2 |

### C12
**Spec contradictions that need an owner ruling** — 18 · lands in: `spec-errata.md`; nobody moves until ruled

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-062 | S2 | ERRATUM | a class `gen fn` with `mut self` contradicts `[CORO-6]` or `[CLS-7]`, and the text does not say which |
| F-081 | S2 | DEFECT | a borrowed class-handle parameter cannot be written through (`E3023`) — text and example disagree |
| F-030 | S3 | ERRATUM | `min`, `max` and `clamp` are used as free functions and declared nowhere |
| F-032 | S3 | ERRATUM | `[TYP-15]` and `[TYP-15a]` disagree about containers of static views |
| F-033 | S3 | ERRATUM | `<<` overflow is undefined by the text |
| F-043 | S3 | ERRATUM | which interface `<` uses is unclear, and `Ord` excludes floats |
| F-047 | S3 | ERRATUM | the `[MOD-5]` prelude list is incomplete against the rest of the document |
| F-052 | S3 | ERRATUM | impl granularity is inconsistent |
| F-086 | S3 | ERRATUM | `[ARN-4]` makes `@noalloc`-cleanliness a property of a value, which the effect system cannot see |
| F-089 | S3 | ERRATUM | `[UNS-8]` and `[LEX-11]` contradict each other about whether a comment can produce a warning |
| F-101 | S3 | ERRATUM | is the stale-handle check removed in `shipping`? |
| F-108 | S3 | ERRATUM | `[CT-1]` excludes allocation from comptime and then supports allocating types |
| F-112 | S3 | ERRATUM | `@derive(SoA)` generates a different `SoA[T]` per `T`, which is specialisation |
| F-143 | S3 | ERRATUM | effect analysis is said to run both after and before monomorphisation |
| F-157 | S3 | ERRATUM | C++ exception policy: "no default" vs a default |
| F-174 | S3 | ERRATUM | `[GEN-COH-1]` makes impl ownership a *package* matter |
| F-122 | S4 | ERRATUM | the `[MOD-5]` prelude list omits names Part XV puts in the prelude |
| F-176 | S4 | ERRATUM | `[HR-IMPL-2]` describes reclaiming an old image |

### C13
**Spec gaps — the text is silent** — 17 · lands in: `OWNER-QUEUE.md`

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-059 | S2 | GAP | what a `fn(A) -> R` type means outside a parameter is unspecified |
| F-109 | S2 | GAP | what happens when comptime-materialised heap data is mutated at run time |
| F-042 | S3 | GAP | no table says which standard interfaces the built-in types implement |
| F-063 | S3 | ERRATUM | `Coroutine[R]` names one type parameter; a coroutine has two |
| F-066 | S3 | GAP | integer `**` with a run-time negative exponent or overflow |
| F-094 | S3 | GAP | is `DefaultHasher` seeded per process? |
| F-106 | S3 | GAP | what `return`, `break`, `continue` and `?` mean inside `with scope = thread.scope():` |
| F-110 | S3 | ERRATUM | `comptime` is used as an expression with a block value, which the grammar does not have |
| F-119 | S3 | GAP | `print`/`println` have no signature |
| F-126 | S3 | GAP | the overlay language has no grammar |
| F-128 | S3 | GAP | the "five count contracts" are never listed |
| F-134 | S3 | GAP | is a base class's `init` inherited? |
| F-175 | S3 | GAP | what happens when a `Pool` runs out of generations |
| F-044 | S4 | GAP | `Formatter`, `FmtError`, `Ordering` and `Into` are used by the interface sketch and declared nowhere |
| F-056 | S4 | GAP | `fn main(args: Span[str])` |
| F-092 | S4 | GAP | phantom type parameters |
| F-213 | S4 | GAP | `i32.MIN % -1` |

### C14
**Spec editorial fixes — no change of meaning** — 47 · lands in: hardening-class edits (`spec-amendments.md`)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-007 | S3 | ERRATUM | the showcase program passes a `str` literal where a `String` is required |
| F-017 | S3 | ERRATUM | the EBNF still contains what `[GRM-16]` and `[GRM-11]` delete |
| F-021 | S3 | ERRATUM | `@nopanic` listed as v2 while `@nopanic(explicit)` is a current contract |
| F-035 | S3 | ERRATUM | `as?` / `as!` are used in IV.6 and absent from the grammar |
| F-050 | S3 | ERRATUM | `[GRM-23]` gives `a in b in c` code `E0104` |
| F-051 | S3 | ERRATUM | `extend[T: Display] Array[T] implements Display` (V.6) is not in the grammar |
| F-099 | S3 | ERRATUM | `@nopanic(explicit)` is missing from the attribute table |
| F-105 | S3 | ERRATUM | the job-system example breaks two rules |
| F-116 | S3 | ERRATUM | `min`/`max`/`clamp`/`abs`/`sqrt`/`sin` are declared in `std.math`, but the spec's examples call them unimported |
| F-120 | S3 | ERRATUM | `unsafe(reason = …)` is not in the grammar; three annotation channels for one fact |
| F-124 | S3 | ERRATUM | two more unapplied editorial instructions inside normative rules |
| F-145 | S3 | ERRATUM | `E9010` has two meanings |
| F-164 | S3 | ERRATUM | the Stage 0 example uses syntax and semantics the rest of the document rejects |
| F-167 | S3 | ERRATUM | the quick reference does not type-check, and one of its errors is in the reference itself |
| F-170 | S3 | ERRATUM | the headline multi-region example does not compile under the document's own rules |
| F-172 | S3 | ERRATUM | Part XXIV's normative rules have no category |
| F-177 | S3 | ERRATUM | Appendix B is a "mandatory CI" checklist that fails against the file it is in |
| F-016 | S4 | ERRATUM | the 0.8.2c change log says there is no if-let; the grammar has one |
| F-020 | S4 | ERRATUM | `@deprecated` has two signatures |
| F-022 | S4 | ERRATUM | a pasted editorial instruction inside `[BLD-11]` |
| F-027 | S4 | ERRATUM | `[LEX-15]` still says "The reserved set therefore has 48 entries" |
| F-028 | S4 | ERRATUM | the Part I.4 table still promises "~2 ns per access pair" |
| F-034 | S4 | ERRATUM | `[TYP-13]` guarantees a niche for `*fn` |
| F-039 | S4 | DOCS | `[RNG-10]` carries `[RNG-10a]`, `[RNG-10b]` and `[RNG-10c]` inline in one bullet |
| F-053 | S4 | ERRATUM | three rules stated twice, verbatim, inside themselves |
| F-057 | S4 | ERRATUM | undeclared names in normative examples |
| F-071 | S4 | ERRATUM | the `@borrows` example contradicts the rule and its own semantics |
| F-072 | S4 | ERRATUM | `[DRP-4]` is unenforceable as written |
| F-075 | S4 | ERRATUM | `split_at_mut` is named by `[BRW-5]` and `[TST-25]` (item 8); the API is `split_at` |
| F-076 | S4 | ERRATUM | VII.7's example has statements at file scope |
| F-084 | S4 | ERRATUM | the VIII.7 idiom uses three things the compiler lacks or rejects |
| F-091 | S4 | ERRATUM | `[ALC-3]` declares `static ALLOC: dyn Allocator` |
| F-096 | S4 | ERRATUM | `RuntimeCheck` has four kinds; `[EFF-17]` says `@no_runtime_checks` "excludes all five kinds" |
| F-097 | S4 | ERRATUM | `[EFF-16]`'s last two sentences have no clear antecedent |
| F-111 | S4 | ERRATUM | more example syntax the grammar lacks (`SoA[T].Ref`, `mut` at call sites, …) |
| F-117 | S4 | ERRATUM | `[STD-8]`'s mandated help text is Rust syntax |
| F-118 | S4 | ERRATUM | `[TXT-2]` still names `CppString.as_str()` |
| F-125 | S4 | ERRATUM | `std::string` → `CppString` "with `.as_str()`" |
| F-129 | S4 | ERRATUM | the `[FFI-39]` example uses constructor syntax the grammar does not have |
| F-131 | S4 | ERRATUM | XVII.5 writes `mut` at a call site |
| F-132 | S4 | ERRATUM | the mandated refusal report suggests a signature the rules reject |
| F-135 | S4 | ERRATUM | wrong or loose citations |
| F-152 | S4 | ERRATUM | `[DIA-7a]`'s table is flattened into one line |
| F-154 | S4 | ERRATUM | stale conditional and dead codes in the diagnostics part |
| F-158 | S4 | ERRATUM | more amendment text appended instead of applied |
| F-163 | S4 | ERRATUM | XXI.5 lists `book/` as "(user guide, v1.1)" |
| F-165 | S4 | ERRATUM | stale lists in XXIII |

### C15
**Tooling, tests and CI gates** — 11 · lands in: `tools/`, `tests/`, CI

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-144 | S1 | TOOLING | the instrument that measures the goal does not exist |
| F-013 | S2 | TOOLING | "accepted by Ember, rejected by the C compiler" is a whole defect class with no gate |
| F-085 | S2 | TOOLING | the development target's examples are not checked by any gate, and half of them do not parse |
| F-100 | S2 | GAP | the first-hour path does not exist |
| F-171 | S2 | GAP | the machine-readable implementation matrix the spec requires does not exist |
| F-207 | S2 | TOOLING | the six "green" gates are green because their baselines absorb almost everything |
| F-001 | S3 | TOOLING | `tasks/audit/TASKS.md` (the audit's index) was never pushed |
| F-142 | S3 | TOOLING | `compiler/ember_typeck/src/lib.rs` is 20,935 lines in one file |
| F-146 | S3 | GAP | most of the CLI in XX.1 is missing |
| F-214 | S3 | TOOLING | the test suite is green (263 tests) but has a flaky UI pair, and `cargo test` hides everything after the first failure |
| F-026 | S4 | TOOLING | `[LEX-2]`: the formatter writes the platform's line ending by default |

### C16
**Docs and project process** — 15 · lands in: docs + process (owner for the process items)

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-115 | S1 | DESIGN | the build order is inverted against the goal |
| F-156 | S2 | GAP | no user guide and no "coming from Python" chapter |
| F-159 | S2 | PROCESS | Phase 1 is recorded complete but Parts II–VI features are missing |
| F-173 | S2 | DOCS | one 870 KB file is three documents |
| F-199 | S2 | PROCESS | the ledgers say "none open" while ~80 defects reproduce |
| F-200 | S2 | PROCESS | specification churn far outpaces the compiler |
| F-127 | S3 | DESIGN | scope: the C++ importer is a project the size of the rest of the compiler |
| F-166 | S3 | PROCESS | three owner decisions bear directly on the goal and should be re-put with the evidence from this pass |
| F-179 | S3 | PROCESS | 1.0 is tied to a full RageV production migration |
| F-201 | S3 | DOCS | `docs/HANDOFF.md` is 10,698 lines |
| F-103 | S4 | DOCS | `[TOOL-1]`–`[TOOL-4]` sit inside X.3 "Inspection" |
| F-121 | S4 | DOCS | sections filed under the wrong Part |
| F-178 | S4 | DOCS | Appendix H is 730 lines of audit records that "bind nobody" |
| F-202 | S4 | DOCS | `examples/` has only `hello.em` |
| F-212 | S4 | DOCS | two errata are marked open that the grammar already fixed |

### C17
**Withdrawn** — 1 · lands in: —

| Id | Sev | Kind | Finding |
|---|---|---|---|
| F-069 | - | WITHDRAWN | (withdrawn: unmarked callbacks already accept local views — see F-168) |

<!-- CATEGORY-INDEX-END -->

---

## Findings

### Part A — carried over: the 2026-09-23 audit reproducers

The 40 programs in `tasks/audit/repro/` cover 24 audit task ids (CG-1, CG-2,
DIAG-1..4, FE-1..3, PERF-2, RC-1..3, SAFE-1..6, UB-1..5). One program,
`safe5_control_plain_name`, is a control that already behaves correctly, so
they record **39 defects**. Their index `tasks/audit/TASKS.md` was never
pushed; PERF-1 has no reproducer, so the missing index held more than the
programs show. Re-run status against `761ce7c` is recorded in Part D.

* **F-001** `TOOLING` S3 — `tasks/audit/TASKS.md` is cited by the header of every
  reproducer and exists on no branch. Recover it from whoever ran the audit, or
  rebuild it from the reproducer headers (this file's Part D table is that
  rebuild).

### Part B — found by this pass

#### B.1 The spec's own showcase program (Part I.5) does not compile

Extracted verbatim from lines 1125–1170 and run through `ember check` at
`761ce7c`: **12 errors**. Some are phase gaps (no `std.math`, no `std.io`); four
are real.

* **F-002** `DEFECT` S2 — **`()` is not accepted as a value of type `void`.**
  Part IV.2 table (line 1620): *"`void` | 0 | the unit type; value `()`"*.
  ```
  fn main():
      x: void = ()        # E2020 expected `void`, found `()`
  ```
  Also breaks the ordinary `return Ok(())` from a `Result[void, E]` function —
  the idiom every fallible `main` in the spec uses.
* **F-003** `DEFECT` S2 — **`import a.b.c` does not bind `c`.** `[MOD-3]`:
  *"`import a.b.c` binds `c` as a namespace"*. `import std.collections` then
  `collections.DefaultHasher.new()` → `E1010 cannot find 'collections' in this
  scope`. Only `from … import` works. Every `import std.io` / `io.println` in
  the spec fails the same way.
* **F-004** `DEFECT` S2 — **importing a module that does not exist is silently
  accepted.** `import std.nonexistent` followed by any program compiles and runs
  (exit 0). Same for `import std.io`, which is not implemented. It should be an
  unresolved-module error at the import.
* **F-005** `DEFECT` S3 — **every E2020 type mismatch carries a note about
  numeric conversion.** `expected 'String', found 'str'` and `expected 'void',
  found '()'` both print `note: Ember does not convert between numeric types
  implicitly [TYP-4]`. The note should appear only when both sides are numeric.
* **F-006** `DEFECT` S4 — **N1 suggestion quality**: `cannot find 'io'` suggests
  `did you mean 'Eq'?` (distance 2 on a two-letter name). With an
  `import std.io` line in the file the right help is "`import a.b.c` binds `c`"
  or "module `std.io` is not available". Suggest a distance cap proportional to
  name length (Python uses a 1/3-of-length cutoff).
* **F-007** `ERRATUM` S3 — **the showcase program passes a `str` literal where a
  `String` is required.** `Emitter("fountain")` calls `fn init(mut self, name:
  String)`. `[LEX-20]`: *"String literals have type `str` … `String(literal)` or
  `literal.to_string()` produces an owned `String`"*, and `[PHIL-2]` forbids
  implicit heap allocation. So either the example is wrong or there is an
  undocumented `str → String` conversion. (Design side: F-031 in B.4 proposes
  allowing a **literal** to initialise a `String` parameter, since that is the
  single most common Python-user stumble.)
* **F-008** `DEFECT` S3 — **one bad character reports once per byte.** A line
  starting with three backticks prints `E0100 unexpected character` three times
  plus a fourth "expected a declaration". Coalesce a run of invalid characters
  into one diagnostic.
* **F-009** `DEFECT` S3 — **"not implemented in this phase" reuses `E1010`**
  (the "cannot find" code): `this type is not supported yet in this phase of the
  compiler`, `mutable class-field access requires a 'mut self' class method in
  this phase`. A user cannot tell "your program is wrong" from "the compiler is
  unfinished", and tooling that keys on codes miscounts. Give phase limits their
  own code (e.g. `E9xxx not yet implemented`) and list them in one place.
* **F-010** `DEFECT` S3 — **cascades after a failed import.** The unresolved
  `from std.math import Vec3` is reported, then every later use of `Vec3` is
  reported again (4 extra errors). A name whose import failed should be
  poisoned silently (same family as audit DIAG-1).
* **F-011** `DEFECT` S3 — **cascade after a private constructor.** `DefaultHasher()`
  outside its module reports `E1020 constructor is private` **and** `E2020 field
  'state' of 'DefaultHasher' has no value` for the same call.

#### B.2 Printing, strings and f-strings

* **F-012** `DEFECT` **S1-class (accepted program, broken C)** — **`println` of any
  `String`, including every f-string, emits C that does not compile.**
  ```
  fn main():
      a = 3
      println(f"{a}")      # passes `ember check`; clang: passing 'ember_vec' to
                           # parameter of incompatible type 'ember_str'
  ```
  `s = f"a={a}"` then `println(s)` fails the same way: the backend calls
  `ember_println_str(<String>)` without the `String → str` view that D-038 added
  for assignments and arguments. **No conformance case uses an f-string**
  (`grep 'f"' tests/` is empty), which is why nothing caught it. `[LEX-19]`
  f-strings are the Python-user's first printing tool, so this is a first-hour
  failure. **Broader than strings:** `println(x)` passes type checking for
  *any* `x` and falls back to `ember_println_str` for every type outside a
  built-in set — probed: `println((1, 2))`, `println(Some(3))`,
  `println(P(x=1))` and `println(p)` for a range type `p: Percent` all end in
  `error: the C compiler failed`; `bool`, `char`, integers and floats work.
  `println` of a non-`Display` type should be a type error (`E2040`) naming
  `Display`/`Debug`.
* **F-013** `TOOLING` S2 — **"accepted by Ember, rejected by the C compiler" is a
  whole defect class with no gate.** F-012, audit CG-2 (C keywords as field
  names) and FE-3 (`i128`) are all this shape. Proposal: a CI job that compiles
  every accepted `tests/**/*.em` and every spec ```` ```ember ```` block with
  `-std=c11 -pedantic -Werror` under both clang and MSVC, and a small
  grammar-driven generator of well-typed programs (csmith-style) that must never
  reach a C error. `[CMP-3]`'s "the C backend does not define the language" is
  only enforceable with such a gate.

#### B.3 Grammar (Part III) and declaration-by-assignment

* **F-014** `DESIGN` S2 — **a name assigned in every branch of an `if`/`else` is
  not visible after it.** Python users write:
  ```
  if n > 1:
      result = 10
  else:
      result = 20
  println(result)        # E1010 cannot find `result` in this scope (no help)
  ```
  `[GRM-4]` declares `result` separately inside each block and block scoping
  ends both. The typo half of this footgun is already covered and works
  (`[LNT-2]` `L1002` "did you mean to assign to `total`?", `[LNT-3]` `L1001`
  "never read" — both probed, both fire). The branch half is not. Proposals:
  (a) **cheap:** when `E1010` names a binding that exists in sibling branches
  just above, the help says *"`result` is declared inside the `if`/`else`
  above; declare it first with `result: i32`, or write `result = 10 if n > 1
  else 20`"* (deferred initialisation `result: i32` already works — probed;
  the conditional expression does not yet, F-180);
  (b) **the Python feel:** if every arm of an exhaustive `if/elif/else` or
  `match` declares the same name at the same type and none reads an outer one,
  the binding belongs to the enclosing block. Definite-initialisation analysis
  already exists to check it. (Note: `OQ-2`/ADR-002 chose block scoping over
  Python function scoping; (b) keeps block scoping and only hoists a name every
  arm declares, so it refines that decision rather than reversing it — the
  owner should still rule on it as a semantic change.)
* **F-015** `DEFECT` S3 — **`if x = 5:` (a `==` typo) gets a misleading fix.**
  `E2036 this pattern always matches` with `help: write 'x = e' on the preceding
  line`. When the pattern is a bare identifier already bound in scope, the help
  should be `did you mean 'x == 5'?` first.
* **F-016** `ERRATUM` S4 — **the 0.8.2c change log says there is no if-let; the
  grammar has one.** Line 781: *"`if Some(p) = …` has no if-let to parse into"*.
  Line 1442: `condition := expression | pattern "=" expression (* 'if Some(x) =
  opt:' *)`, and `[GRM-19]` constrains it. The change-log sentence is stale. (The
  compiler: `E1010 a pattern condition is not supported yet in this phase`, plus
  a cascade `cannot find 'x'` — F-009/F-010 again.)
* **F-017** `ERRATUM` S3 — **the EBNF still contains what `[GRM-16]` and `[GRM-11]`
  delete.** `[GRM-16]` says the `small_stmt` alternatives `"return"`, `"break"`,
  `"continue"` are removed and `block_expr` is deleted from `atom`; lines
  1431–1432 and 1496 still show all of them, and `small_stmt` carries a
  placeholder `"defer" ":" ...`. A reader implementing from the EBNF builds the
  wrong grammar. Rewrite the productions to match the rules.
* **F-018** `DESIGN` S3 — **two path separators, `.` and `::`.** `path_expr :=
  identifier "::" identifier` (line 1503; OQ-15 kept it), but `path_type` uses
  `.`, imports use `.`, and **every example in the spec uses `.`**
  (`io.println`, `Vec3.ZERO`, `Type.name(...)`, `DefaultHasher.new()`); no
  ```` ```ember ```` block uses `::`. Python has one separator and resolves by
  what the left side is. Proposal: drop `::` from Ember source (keep it only
  inside C++ strings), or record why a user would ever need it. **This reopens
  an owner decision** (`OQ-15`: *"Stays"*); the new evidence is that no
  example in 0.9.8 uses it.
* **F-019** `DESIGN` S3 — **`as` binds tighter than unary minus.** Level 13
  prefix `-` is below level 15 `as`, so `-x as u32` is `-(x as u32)`. C, C++,
  Rust, Swift and Go all apply the sign first. With untyped literals,
  `-1 as u8` becomes "negate an unsigned", which is exactly the family of audit
  FE-2. Proposal: make prefix `-`/`~` bind tighter than `as`, or reject an
  unparenthesised `-x as T` with a fix-it.
* **F-020** `ERRATUM` S4 — **`@deprecated` has two signatures** in the attribute
  table: `@deprecated("msg")` (fn, type) and `@deprecated(since, note)` (any
  item).
* **F-021** `ERRATUM` S3 — **`@nopanic` is listed as v2 in the attribute table,
  while the 0.8.4_Hardened_1 change log calls `@nopanic(explicit)` one of Ember's
  current contracts** and `[EFF-22]` specifies it. One of them is stale.
* **F-022** `ERRATUM` S4 — **a pasted editorial instruction inside `[BLD-11]`**
  (line 5744): *"— replace "(`E1020`)" with "(`E1021 name is not linked…`)" …
  Add `docs/errors/E1021.md`"*. This is the fourth instance of the defect class
  the 0.8.1 change log names; `[DIA-6a]`'s completeness pass was supposed to
  grow a check for imperative prose in rule bodies and evidently did not.
* **F-023** `DEFECT` S3 — **`E1020` means two things in the compiler.** The
  registry (`ember_diag/src/codes.rs:154`) and `[GRM-4]` make it *"name is
  already declared in this block"*; `ember_typeck` also emits `E1020` for
  "memberwise constructor is private" and private-field access
  (`lib.rs:1931, 9440, 14361`). `[BLD-11]` says `E1020` *"MUST NOT be reused"*.
  Privacy needs its own code.
* **F-024** `DESIGN` S3 — **`[GRM-14]` adds a third kind of generic parameter
  (`access P`) for one library type, `std.ecs.Query`.** `[OQ8-*]` says the ECS
  has *"no ECS compiler magic"*; an access-mode generic kind that exists only
  so `Query[(mut Position, Velocity)]` parses is language surface for one
  library. Either generalise it (read/write capability as an ordinary
  type-level concept usable by any container) or give `Query` an ordinary
  signature (`Query[Write[Position], Read[Velocity]]`).
* **F-025** `DESIGN` S3 — **`[LEX-22]` reserves `'a` for "v2 named lifetimes".**
  The goal excludes Rust's annoying parts, and named lifetimes are the first
  thing on that list. `[LT-15]` already forbids *requiring* named-lifetime
  syntax for views. Proposal: drop the v2 reservation and state as a
  non-goal that Ember never exposes lifetime names; the escape hatches
  (`@borrows`, `@latebound`) cover the cases (F-168 argues `@latebound`
  should go; F-070 that `@borrows` should be needed less).
* **F-026** `TOOLING` S4 — **`[LEX-2]`: the formatter writes the platform's
  line ending by default.** That makes `ember fmt` output differ between Windows
  and Linux checkouts of one repository and fights `.gitattributes`. Default to
  LF everywhere; CRLF only by explicit `ember.toml` setting.
* **F-027** `ERRATUM` S4 — **`[LEX-15]` still says "The reserved set therefore
  has 48 entries"** and `[LEX-15b]` overrides it to 49. The table has 49. Fix the
  number in `[LEX-15]` instead of keeping a rule whose only job is to correct
  another rule's arithmetic. Same for `[LEX-15a]`, whose first sentence is
  repeated verbatim at its end.
* **F-028** `ERRATUM` S4 — **the Part I.4 table still promises "~2 ns per access
  pair"** for object exclusivity, which the 0.5 change log (row 9, `OQ-16`)
  withdrew as a promise.

#### B.4 Type system (Part IV)

* **F-029** `DEFECT` **S2** — **`xs = Array[i32]()` is rejected: the explicit type
  argument is ignored.** `E2060 cannot tell what this 'Array' holds; annotate
  the variable`. `[TYP-18]` names exactly this form as the required spelling
  (*"explicit instantiation … required when no argument mentions the parameter
  (`Array[f32]()`)"*). `xs: Array[i32] = Array[i32]()` works, so the
  constructor's own type arguments are dropped when nothing else supplies them.
  This is the first line of almost every program that uses a list.
* **F-030** `ERRATUM`/`GAP` S3 — **`min`, `max` and `clamp` are used as free
  functions and declared nowhere.** The IV.2a example writes
  `y = min(max(x, 0.0), 1.0)`; `[RNG-3a]`, `[RNG-4]` and `[RNG-5a]` build rules on
  them. They are not in the `[MOD-5]` prelude list, and the compiler says
  `cannot find 'min'` (suggesting `main`). Declare them (prelude, generic over an
  ordering bound, float-aware per `[TYP-9]`) or rewrite the examples.
* **F-031** `DESIGN` S2 — **strings are the biggest Python-ergonomics gap.**
  Probed at `761ce7c`: `Emitter("fountain")` into a `String` parameter,
  `names.push("alice")` into `Array[String]`, and `t: String = "x"` are all
  `E2020 expected 'String', found 'str'`. Every Python user hits this in the
  first ten minutes. `[PHIL-2]` (no implicit heap allocation) is why. Proposal:
  let a **string literal** (not an arbitrary `str`) initialise a `String` at a
  coercion site, since the allocation is visible at the literal and costs the
  same as the `String("…")` the user must type anyway; keep arbitrary
  `str → String` explicit. Record it as a named, documented `[PHIL-2]`
  exception. (Alternative: make `Array[str]` of static literals legal, which
  `[TYP-15]`'s own principle already allows — see F-032.)
* **F-032** `ERRATUM` S3 — **`[TYP-15]` and `[TYP-15a]` disagree about containers
  of static views.** `[TYP-15]`: container elements have no bounding region, *"so
  they may hold only a view whose every region slot is `static`"* — which admits
  `Array[str]` of literals. `[TYP-15a]`: *"Arbitrary owning containers
  instantiated at a view type (`Array[str]`, …) remain rejected"* — outright.
  S1 (ERR-044) resolved the same tension for class fields in favour of static
  views. Owner ruling needed; the ergonomic answer is to follow `[TYP-15]`.
* **F-033** `ERRATUM` S3 — **`<<` overflow is undefined by the text.** `[TYP-8]`
  lists `<<` among the operations that *"wrap"* in release, which implies it can
  overflow and so panics in debug; `[TYP-10]` defines only the
  shift-*amount* check. Probe: `x: i32 = 1; x << 31` gives `-2147483648` in
  debug with no panic (Rust semantics: bits shifted out are not checked), while
  `x << 32` panics in debug and gives `1` in release (masked). The compiler
  follows Rust; the text should say so, and should say what a **negative
  signed shift amount** does (audit UB-3 found the debug check is a signed
  `n >= 32`, so `a >> -1` runs). Proposal: *"the amount is taken as unsigned;
  amount ≥ width panics in debug and is masked in release; bits shifted out are
  not an overflow"*, and emit `(unsigned)` shifts in C so no profile has UB.
* **F-034** `ERRATUM` S4 — **`[TYP-13]` guarantees a niche for `*fn`**, a type the
  grammar does not have (raw function pointers are `extern "C" fn(...)`).
* **F-035** `ERRATUM` S3 — **`as?` / `as!` are used in IV.6 and absent from the
  grammar.** Part III's `postfix` has only `"as" type`, and II.6 lists `?` and `!`
  as separate tokens, so `h as? Derived` reads as `(h as ...)?`. The compiler
  accepts `a as? Dog` (probed, prints `4`), so the grammar is what is missing.
* **F-036** `DESIGN` S3 — **`@view` is required on a struct the compiler already
  knows is a view** (`E2030` with a fix-it, "required on the declaration as
  documentation"). An annotation the compiler can infer and insists on is the
  kind of ceremony the goal excludes. Proposal: infer it; let `ember fmt` or the
  IDE show it; keep `@view` optional as documentation.
* **F-037** `DESIGN` S3 — **the default integer is `i32`.** `[LEX-16]`: an
  untyped literal with no context becomes `i32`. For a "types like Python"
  goal, `i32` overflow at 2.1 billion is the trap Python users never meet
  (sums, products, timestamps, file sizes). Swift and Go default to a 64-bit
  `Int`; C speed is unaffected on 64-bit targets for scalar code. The f32 float
  default (Part 0 row 12) has an engine rationale; the i32 default has none
  written down. Worth an owner decision either way, with the reason recorded.
* **F-038** `DESIGN` S3 — **floats have no `Ord`, so sorting a list of floats
  needs ceremony.** `[TYP-9]`: *"`Ord` is not implemented for floats — use
  `partial_cmp` or `total_cmp`"*. Rust's `PartialOrd`/`Ord` split is on most
  lists of Rust annoyances. Proposal: keep IEEE `<` as the operator, and make
  `sort()`/`min`/`max` on floats use `total_cmp` by default (NaN sorts last),
  documented, so `xs.sort()` just works; also note that the `[MOD-5]` prelude
  has `Eq`/`Ord` but not the `PartialEq`/`PartialOrd` that `[RNG-5a1]` generates
  impls of — another undeclared name.
* **F-039** `DOCS` S4 — **`[RNG-10]` carries `[RNG-10a]`, `[RNG-10b]` and
  `[RNG-10c]` inline in one bullet** (line 1706), the structural shape ODR-002
  found the rule extractor could not read. Split them into their own bullets.

* **F-040** `DEFECT`+`DESIGN` S3 — **the orphan rule `[TYP-20]` is not enforced, and
  as written it is stricter than Rust's.** `[TYP-20]`: an implementation of `I`
  for `T` *"may appear only in the module that declares `I` or the module that
  declares `T`"* — and a module is **one file** (`[MOD-1]`). Probe: `lib/shapes.em`
  declares `P`, `lib/iface.em` declares `Named`, `main.em` writes `extend P
  implements Named` → **accepted, prints 8**. A duplicate impl in a second
  module is caught (`E2041`), so whole-program coherence holds today; what is
  missing is the rule that makes separate compilation safe. Design point: a
  per-*file* orphan rule forbids organising one package's impls into their own
  file, which even Rust (per-crate) allows, and Rust's orphan rule is itself on
  the annoyances list. Proposal: owner picks **package** granularity (matches
  `[BLD-2]`'s unit of separate compilation), then the compiler enforces it.
* **F-041** `DEFECT`+`DESIGN` S4 — **"interface methods need the interface
  imported" (`[TYP-24]` step 2) is not enforced.** `main2.em` calls `p.tag()`
  with neither `Named` nor the impl's module imported by name (only `import
  lib.impl`) → accepted. Rust's "trait not in scope" error is a well-known
  papercut. Proposal: make the spec match the compiler — resolve an interface
  method without an import when exactly one visible impl provides it; keep
  `E2070` for true ambiguity.
* **F-042** `GAP` S3 — **no table says which standard interfaces the built-in
  types implement.** Floats have `==` with `NaN != NaN`; do they implement
  `Eq` (and so qualify as `Map` keys under `[HASH-4]`) and `Hash`? Do tuples,
  fixed arrays, `char`, `bool`, `str`, `Option`, `Result` get `Eq`/`Ord`/`Hash`/
  `Debug`/`Display`/`Default`/`Clone`? Every generic program depends on this.
  Proposal: one normative table in IV.8.
* **F-043** `ERRATUM` S3 — **which interface `<` uses is unclear, and `Ord`
  excludes floats.** The IV.8 sketch puts `# < > <= >=` on `Ord`, and floats have
  no `Ord` (`[TYP-9]`), yet floats need `<`. `PartialOrd: Eq` exists with only
  `partial_cmp`. `PartialEq` is used by `[RNG-5a1]` and never declared.
  Consequence: a generic `fn largest[T: Ord](xs: Span[T])` cannot take `f32`,
  the most common element type in engine code. Proposal: `<` on generics goes
  through `PartialOrd`; `Ord` refines it; declare `PartialEq` or drop it from
  `[RNG-5a1]`.
* **F-044** `GAP` S4 — **`Formatter`, `FmtError`, `Ordering` and `Into` are used
  by the interface sketch and declared nowhere**, and none is in the `[MOD-5]`
  prelude, so a user cannot write `implements Display` from the text alone.
* **F-045** `DESIGN` S4 — **three iteration interfaces** (`IntoIterator`,
  `Iterable`, `IterableMut`) plus `Iterator`. The `for x in v` / `for x in owned
  v` split maps to two of them; worth checking whether `IterableMut` can be
  folded into `Iterable` with a `mut self` overload of `iter` once modes are
  part of resolution. Low priority.

#### B.5 Declarations (Part V)

* **F-046** `DESIGN` **S2** — **eight selectable language versions before 1.0.**
  `[MOD-6]` requires the compiler to accept `0.8.3`, `0.8.4`, `0.8.5`, `0.9`,
  `0.9.5`, `0.9.6`, `0.9.7` and `0.9.8`, each selecting *"that version's
  accepted-program and semantic rules"*. There are no external users; every
  additive revision multiplies the test matrix and every checker needs a
  version switch. Rust, Swift and Go support no selectors before 1.0.
  Proposal: before 1.0, a source file selects nothing and gets the current
  development language; the `#! language` machinery starts at 1.0. The
  history stays in the change logs.
* **F-047** `ERRATUM` S3 — **the `[MOD-5]` prelude list is incomplete against the
  rest of the document.** Missing: `Cell`, `RefCell` (the 0.5 change log row 6,
  `OQ-10`: *"`Cell` and `RefCell` join the prelude"* — and the compiler does
  accept `Cell(3)` with no import, probed), `RangeError` (`[RNG-3]`: *"a prelude
  type"*), `min`/`max`/`clamp` (F-030), `PartialOrd`, `From`, `Error`,
  `IntoIterator`, `Callable`. One list should be authoritative.
* **F-048** `DESIGN` S2 — **`Map` and `Set` are not in the prelude.** Python's
  `dict`/`set` are built-in; in Ember they need `from std.collections import
  Map`. Proposal: add `Map`, `Set` (and `min`, `max`, `abs`, `range`-style
  helpers if any) to the prelude.
* **F-049** `DESIGN` S3 — **Python chained comparison is rejected.** `0 <= x < 10`
  → `E0102 chained comparison` (probed; good help text). The rejection guards
  against C's `a < b < c` reading, which cannot arise in Ember because `bool`
  never compares with an integer. Proposal: accept Python's meaning
  (`lo <= x and x < hi`, middle operand evaluated once) for `<`, `<=`, `>`,
  `>=`, `==`, `!=`; keep `in` non-associative.
* **F-050** `ERRATUM` S3 — **`[GRM-23]` gives `a in b in c` code `E0104`** — the
  unknown-attribute code. III.5 gives chaining `E0102`, and the compiler emits
  `E0102` for comparisons. `[GRM-23]` should say `E0102`.
* **F-051** `ERRATUM` S3 — **`extend[T: Display] Array[T] implements Display` (V.6)
  is not in the grammar**: `extend_decl := "extend" type [implements_clause]
  [where_clause] ":" type_body` has no generic parameter list. Either add
  `["[" generic_params "]"]` after `extend` or rewrite with `where`.
* **F-052** `ERRATUM` S3 — **impl granularity is inconsistent.** `[IFC-1]` lets any
  module in the package add inherent methods with `extend T:`; `[TYP-20]` lets
  only the declaring module add an interface impl. Supports F-040's package
  granularity.
* **F-053** `ERRATUM` S4 — **three rules stated twice, verbatim, inside
  themselves**: `[GRM-20]`, `[GRM-8d]` and `[FFI-34a]` (lines 2151, 2155, 2156)
  each repeat their whole text a second time. Also `[CLS-7a]` sits after
  `[CLS-9]`, and `[GRM-20]`–`[GRM-23]` live in Part V's attributes section.
* **F-054** `DESIGN` S3 — **every struct needs `@derive(...)` boilerplate.**
  `[STR-3]`/`[STR-5]`: `Copy`, `Eq`, `Ord`, `Hash`, `Debug` are never implicit.
  The spec's own `Vec3` needs `@derive(Copy, Debug, Eq)`. Keeping `Copy`
  explicit has a stated reason (adding a field must not silently change
  semantics). `Debug` and field-wise `Eq` have no such risk and are what a
  Python `@dataclass` gives by default. Proposal: auto-derive `Debug` and `Eq`
  (and `Clone` when all fields are `Clone`) for plain structs whose fields
  support them, with `@no_derive(...)` to opt out; keep `Copy`/`Ord`/`Hash`
  explicit.
* **F-055** `DESIGN` S2 — **no runtime-initialised globals in v1.** V.7:
  *"`static lazy` v2; in v1 non-comptime statics are `E2130`"*, and statics must
  be comptime-constructible. A registry `static REG: Mutex[Map[str, Handler]]`
  or any global holding heap data cannot be written in Safe Ember v1. Python
  module-level state is ordinary. Proposal: `static lazy` (thread-safe
  once-init, `Sync` required) in v1, or allow comptime heap values to be
  materialised into static storage. (The compiler today also restricts `const`
  to literals — audit DIAG-1.)
* **F-056** `GAP` S4 — **`fn main(args: Span[str])`** (`[FN-8]`) promises
  UTF-8 `str` for OS arguments, which on Linux are arbitrary bytes and on
  Windows UTF-16. Say what happens to a non-UTF-8 argument (fail at startup,
  lossy replace, or a `Span[OsStr]` form).
* **F-057** `ERRATUM` S4 — **undeclared names in normative examples**: `PI`
  (V.4 `Shape.area`), `sqrt` (V.3 `Vec3.length`), `Entity` used both as a
  class (`Weak[Entity]`, V.5) and elsewhere as an ECS id. `[TST-7]` compiles
  spec blocks; these presumably ride on an ignore marker — check that the
  showcase blocks are actually compiled by CI.

#### B.6 Expressions, closures, coroutines (Part VI)

* **F-058** `DEFECT` **S2** — **a closure cannot be returned or boxed; the spec's own
  form is rejected.** `[CLO-3]`: *"A boxed dynamic closure is `Box[dyn fn(A) ->
  R]`"*. Probes at `761ce7c`:
  ```
  fn make_adder(n: i32) -> Box[dyn fn(i32) -> i32]:
      return Box(owned fn(x: i32) => x + n)
  # E1010 a `dyn` bound must name an interface
  # E2020 expected `Box_error`, found `Box_closure0_env`   <- internal names leak
  # E1010 cannot find `add5`                               <- cascade

  fn make_adder(n: i32) -> fn(i32) -> i32:
      return owned fn(x: i32) => x + n
  # E2020 expected `fn(i32) -> i32`, found `closure0_env`
  ```
  So there is **no way to return a closure or keep a capturing one in a
  collection**: callbacks, event handlers, factories, job queues. (Audit DIAG-2
  already records the `closure0_env` name leak; this is the missing feature
  under it.)
* **F-059** `GAP` S2 — **what a `fn(A) -> R` type means outside a parameter is
  unspecified.** `[CLO-3]` defines it only for parameters (*"an implicit generic
  parameter bounded by `Callable`"*). The compiler accepts `struct Button:
  on_click: fn() -> i32`, and `f = b.on_click; f()` prints 3 — so a field
  position means *something* (an implicitly generic struct?). Return position
  and local annotations are also unspecified. Proposal (Python feel): in a
  field, local, return or container-element position, `fn(A) -> R` means an
  **owned, type-erased callable** (`Box[dyn fn(A) -> R]` with small-closure
  inline storage), so `struct Button: on_click: fn()` just works and costs one
  indirect call; in parameter position it stays the zero-cost generic bound.
  Say it in the spec either way.
* **F-060** `DESIGN` S3 — **calling a callable field needs a temporary.**
  `b.on_click()` → `E1010 'Button' has no method named 'on_click'`; `f =
  b.on_click; f()` works. Python calls it directly. Proposal: when no method of
  that name exists and a field of callable type does, `b.on_click()` calls the
  field (or give the error a help naming `(b.on_click)()`).
* **F-061** `DESIGN` S3 — **a boxed once-callable is not callable (`[CLO-6a]`).**
  The text itself calls the `Option`/`mem.take` workaround "the Option dance"
  and defers the fix to v2. Rust fixed exactly this (`Box<dyn FnOnce>`
  callable) in 1.35. One-shot tasks — thread spawns, job queues, deferred
  callbacks — are the main users. Proposal: allow `owned self` through a
  `Box[dyn …]` receiver in v1 (the box is consumed, so the vtable call is
  sound).
* **F-062** `ERRATUM` **S2** — **a class `gen fn` with `mut self` contradicts
  `[CORO-6]` or `[CLS-7]`, and the text does not say which.** The VI.5a example
  is `gen fn open_door(mut self) -> Coroutine[()]` with two `yield`s. `[CLS-7]`:
  *"`mut self` grants a dynamically checked write access for the duration of
  the method"*. Either that access is held across the suspensions — so any other
  write to the door during the two-second animation panics with an exclusivity
  violation — or it is released at each `yield`, in which case `[CORO-6]`
  (*"no borrow may be held across a `yield`"*) needs a class-access analogue
  and the method must re-acquire on resume. This is the headline gameplay
  example for coroutines.
* **F-063** `ERRATUM` S3 — **`Coroutine[R]` names one type parameter; a
  coroutine has two** (`CoroutineState[R, Y]`, `Resumable.Return`/`.Yield`).
  `Coroutine[()]` in the example yields `wait(0.5)` and `animate(...)` — values
  of (presumably) different types — and returns nothing. Say how the yield
  type is written or inferred, and whether two `yield`s of different types are
  legal. (Coroutines are not implemented: `cannot find type 'Coroutine'`.)
* **F-064** `DESIGN` S3 — **temporaries in an `if`-condition live through the
  `else`** (`[EXP-4]`: *"temporaries bound by `with`/`for`/`if`-conditions live
  to the end of the construct"*). With `RefCell` this reproduces Rust's classic
  footgun: `if Some(x) = cell.borrow().get(k): … else: cell.borrow_mut()…`
  panics at run time. Rust 2024 changed if-let to drop the scrutinee's
  temporaries before `else`. Proposal: same.
* **F-065** `DESIGN` S4 — **`%` follows C (sign of the dividend) while the syntax
  follows Python.** `(i - 1) % n` is `-1` at `i = 0`; in Python it is `n - 1`.
  Keep C semantics for speed, but add a lint where a `%` result feeds an index
  and the dividend can be negative, pointing at `rem_euclid`.
* **F-066** `GAP` S3 — **integer `**` with a run-time negative exponent.** VI.3 says
  *"integer `**` with negative exponent is `E2151`"* — a compile-time code — but
  says nothing for an exponent only known at run time, nor about overflow of
  the result (`2 ** 40` in `i32`). (`**` is unimplemented: `E1010 this operator
  is not supported yet in this phase`.)

#### B.7 Ownership, borrowing, lifetimes (Part VII §1–§6)

* **F-067** `DESIGN` **S2** — **a method call borrows all of `self`, so the
  classic Rust partial-borrow error is back.** Probe:
  ```
  struct World:
      names: Array[i32]
      count: i32
      fn bump(mut self):
          self.count += 1

  for n in w.names:
      w.count += n      # accepted: disjoint fields ([BRW-4])
  for n in w.names:
      w.bump()          # E3020 cannot mutate `w.names` while it is borrowed by this loop
  ```
  `[BRW-4]`: disjointness does not hold *"through a method call — a method takes
  all of `self`"*. This is on every list of Rust annoyances. Ember is unusually
  well placed to fix it: `90059c8` already infers **parameter-field access
  summaries** for direct bodies to a fixpoint (COLD-START §5). Proposal: for
  non-`virtual`, non-`pub`-across-package methods, the receiver borrows only
  the fields its summary touches (public/virtual methods keep whole-`self`
  borrows so a body change is not a caller-visible break). The E3020 text is
  also misleading here — it says `w.names` is mutated when `bump` only writes
  `count`.
* **F-068** `DESIGN` S3 — **the Python swap `a[i], a[j] = a[j], a[i]` is a parse
  error** (`E0100 expected the end of the line` at the second comma). With
  parentheses, `a[i], a[j] = (a[j], a[i])`, it works and prints `2`. The
  assignment grammar already takes a `target_list`; the right-hand side should
  accept an unparenthesised tuple too (`return a, b` likewise, if not already).
* **F-069** `WITHDRAWN` — **premise disproved by probe; see F-168.** I first
  read `[LT-7]` as saying an unmarked `fn(Span[T]) -> R` callback cannot
  receive a view of the callee's local storage without `@latebound`. Probe:
  ```
  fn with_local(f: fn(Span[i32]) -> i32) -> i32:
      tmp: Array[i32] = Array[i32]()
      tmp.push(41)
      return f(tmp.as_span())      # accepted; prints 42 / 82
  ```
  Unmarked callbacks already accept invocation-local views (they are
  monomorphised, `[CLO-3]`). So `@latebound` adds no capability — it only
  adds the restriction F-168 describes. Kept here so the id is not reused.
* **F-070** `DESIGN` S3 — **`@borrows(arena)` is mandatory on every Arena wrapper,
  at every nesting level** (`[LT-4]`, `[LT-4a]`: *"Nested wrappers MUST repeat
  `@borrows(arena)`"*). Elision cannot supply it because `Arena` is not a view
  type. Proposal: extend `[LT-1]` so a non-view parameter that can *produce*
  views (an `Arena`) is an elision candidate: if the only such parameter is one
  Arena, the returned view borrows it — no annotation, and `Arena` still is not
  a view type.
* **F-071** `ERRATUM` S4 — **the `@borrows` example contradicts the rule and its
  own semantics.** `[LT-1]` writes `fn longest(a: str, b: str) -> str
  @borrows(a)` — trailing — while `[LT-1a]` says `@borrows` is *"written on its
  own line preceding the function declaration"*. And a `longest` can return `b`,
  which `@borrows(a)` rejects (`E3062`); `longest` is precisely the function
  that *needs* the default intersection. Use a function that always returns a
  view of `a`.
* **F-072** `ERRATUM` S4 — **`[DRP-4]` is unenforceable as written**: *"drop bodies
  MUST NOT panic in `abort` mode without accepting process termination"*. Either
  state the behaviour (a panic in `drop` aborts the process — which `[PAN-1]`
  already implies) or make it a lint.

* **F-073** `DEFECT` **S2** — **writing a class field through a handle from outside
  a method fails.** The spec's own beginner example (VII.8, `heal_all`) writes
  `p.health = 100.0` through a class handle and comments *"handle access; fine
  (Part VIII)"*. The compiler: `E1010 mutable class-field access requires a
  'mut self' class method in this phase` **and** `E3021 cannot write through a
  shared reference` on the same line. `fn heal(p: Player): p.health = 100.0`
  gives `E3023 cannot mutate borrowed parameter 'p'` (whether a borrowed
  *handle* may be written through is settled in Part VIII — see B.8). For
  class-based, Python-style code this is the most basic operation there is.
* **F-074** `GAP` S3 — **no way to return an iterator without naming its concrete
  type.** `[SPN-10]`: *"No opaque iterator-return mechanism is introduced"*.
  With closures in adapters (`xs.iter().map(f)`), the type is unnameable, so a
  function cannot return a filtered/mapped view of its data and a struct cannot
  hold one. Two proposals: (a) opaque return types (`-> some Iterator[Item =
  T]`, like Rust's `impl Trait`), or better for this goal, (b) **make `gen fn`
  a Python generator**: a `gen fn` whose frame implements `Iterator[Item = Y]`
  so `for x in evens(10):` works. `[CORO-5]` already makes frames
  allocation-free and compile-time sized, so it costs what a hand-written
  iterator costs.
* **F-075** `ERRATUM` S4 — **`split_at_mut` is named by `[BRW-5]` and `[TST-25]`
  (item 8); the API is `split_at`** (VII.7 example, `SPN-API-1`). One name.
* **F-076** `ERRATUM` S4 — **VII.7's example has statements at file scope**
  (`buf = …`, `normalize(…)`, `left, right = …`), which `[GRM-2]` forbids — the
  same defect the 0.9.7_Hardened_3 change log used to refuse a proposed example.
  Either the block is excluded from `[TST-7]` or the gate is not running it.
* **F-077** `DEFECT`/`GAP` S3 — **spec examples call APIs that do not exist:**
  `Iterator.sum()` (`E1010 'SpanIter_f32' has no method named 'sum'` — also
  leaks the mangled name `SpanIter_f32`), `Array[f32]([1, 2, 3])` (`E2020
  'Array()' takes no arguments`), `Array.retain`, `SpanIter`'s adapters. These
  belong to Part XV's library surface — tracked there — but the examples in
  Parts I and VII are where users look first.

#### B.8 Classes and reference counting (Part VIII)

* **F-078** `ERRATUM` **S1** — **the text contradicts itself on whether a `Sync`
  class has dynamic exclusivity at all, which decides whether Safe Ember has a
  data race.** `[OBJ-1]`'s header table (line 2510): `access_state … (dynamic
  exclusivity …; !Sync classes only)`. `[THR-7]` (line 3252): *"`[PHIL-10]`'s
  data-race guarantee holds because those rules apply to a `Sync` class exactly
  as to any other"*. If a `Sync` class has no access word, two threads may each
  begin a long-term write access (a `mut self` call, an assignment of a
  non-`Copy` field) on one object and nothing stops them. Needs an owner ruling:
  either `Sync` classes get an **atomic** access word, or a `Sync` class's
  fields are all effectively `let` and mutation goes only through
  `Mutex`/`RwLock`/`Atomic` fields (Rust `Arc` / Swift `Sendable` model).
* **F-079** `GAP` **S1** — **assigning a non-`Copy` field through a handle is
  neither an "instantaneous" nor a "long-term" access.** VIII.3 defines
  instantaneous as reading or writing *"a single scalar/`Copy` field"* and lists
  the long-term kinds (`mut self` call, `mut` argument, `ref`/`ref mut`, `for`).
  `h.children = Array()` drops the old `Array` — while a `for c in h.children:`
  loop may be walking it. The audit's `safe4_class_field_view_alias_replace`
  is exactly this and is a heap-use-after-free today. The spec must classify
  the assignment (it should be a write access, checked against active readers)
  and the compiler must implement it.
* **F-080** `DEFECT` **S2** — **`String(literal)` does not exist.** `[LEX-20]`:
  *"`String(literal)` or `literal.to_string()` produces an owned `String`"*.
  Probes: `String("x")` → `E2020 'String()' takes no arguments`;
  `"abc".to_string()` → `E1010 'str' has no method named 'to_string'`. The only
  route is `t: String = String()` then `t.push_str("hi")`. `String + String` →
  `E2020 '+' cannot be applied to 'String'`. (Part XV is where the full string
  API is judged; this is the one the lexical rule itself promises.)
* **F-081** `DEFECT` S2 — **(update to F-073)** `[FN-1]`'s *"the callee cannot
  mutate or move `a`"* is about the handle, and VIII.3 says a scalar field write
  through a handle is *"performed directly with no check"*; VII.8's example
  relies on it. So `fn heal(p: Player): p.health = 100.0` → `E3023 cannot mutate
  borrowed parameter 'p'` is the compiler being stricter than the text. If the
  owner *intends* borrowed handles to be read-only, the VII.8 example and VIII.3
  must say so; either way one of them moves.
* **F-082** `DESIGN` S3 — **reference cycles leak silently in release, and Python
  users build cycles** (parent/child, observer lists). `[WK-1]` gives `Weak`, a
  debug `--leak-check` and the static lint `L3001`. Proposal: make `ember run`
  in `debug` report leaked cycles at exit **by default** (opt-out flag), so a
  leak is visible the first time it happens, not only when someone knows to ask.
* **F-083** `DESIGN` S3 — **objects may die before their last syntactic use**
  (`[RC-3]`, Swift semantics). A class used as a scope guard (lock guard,
  profiler zone, "undo group") can run its `drop` early. `with h:` /
  `mem.keep_alive(h)` pin it, but nothing tells the user. Proposal: a class
  whose `drop` has observable side effects and whose handle is bound but never
  read gets a lint suggesting `with`.
* **F-084** `ERRATUM` S4 — **the VIII.7 idiom uses three things the compiler
  lacks or rejects**: `Option[Box[dyn fn(Entity)]]` (F-058), `if Some(cb) =
  self.on_death:` (if-let, F-016), and calling a borrowed `Box[dyn fn]`.

#### B.9 The spec's own examples, and what the gates check

* **F-085** `TOOLING` **S2** — **the development target's examples are not checked
  by any gate, and half of them do not parse.** `tools/spec_check.py` hard-codes
  `SPEC = docs/spec-source/ember-spec.md` (the adopted 0.8.5), so the
  0.9.8_Hardened_3 target is never examined. Running the same tool against H3
  at `761ce7c`: **46 ` ```ember ` blocks in Parts I–XVII and Appendix A; 22 parse,
  24 do not** (Parts V, VI, VII, IX×5, XI×4, XII×3, XIII, XIV, XVI×4, XVII×3).
  The adopted file carries a baseline of 25 accepted failures
  (`tools/spec_check_baseline.json`). Also, the gate checks **parsing only** (`[TST-7]` itself says
  `--syntax-only`, while the 0.4 change log row 10 describes it as "every fenced
  `ember` block … is compile-checked"); the I.5 showcase parses and has 12
  type errors (F-002 ff.). Proposals: (1) point the gate at
  `development-target.json`'s path as well; (2) type-check, not just parse,
  every block not marked `ignore`; (3) mark signature sketches (`fn
  ArenaArray.with_capacity[T](…)`, `len(self) -> usize` API lists) as
  `ember,ignore` with the permitted reason, since they are not Ember.

#### B.10 Memory facilities (Part IX §0–§2)

* **F-086** `ERRATUM` S3 — **`[ARN-4]` makes `@noalloc`-cleanliness a property of a
  value, which the effect system cannot see.** *"it is `@noalloc`-clean only if
  the arena is `@noalloc`-declared (`Arena.fixed(buffer: MutSpan[u8])`, which
  never grows …) … The type `FixedArena` is provided for hot paths."* If
  `Arena.fixed` returns an `Arena`, the checker cannot know at a call site that
  `alloc` will not grow. Only a distinct type (`FixedArena`) can carry the
  fact. Proposal: `Arena.fixed` returns `FixedArena`; drop the
  "`@noalloc`-declared arena" wording.
* **F-087** `DESIGN` S3 — **`Shared[T]` overlaps `class` almost completely.** Both
  are counted heap objects with the same header, both use dynamic exclusivity
  for long-term mutable access, both have `Weak` companions; IX.0 itself says
  *"Advanced; prefer `class`"*. Twelve storage mechanisms is a lot to teach a
  Python programmer. Worth asking whether `Shared[T]` pays for its surface, or
  whether "a `class` wrapping a `T`" is enough. (Low priority: the owner
  selected its API in 0.9.8.)
* **F-088** `GAP` **S2** — **almost none of the Part IX.1 heap library exists.**
  `std/src/*.em` totals **386 lines** (`borrow`, `collections`, `core`, `lib`,
  `math`, `mem`). No `Map`, `Set`, `Deque`, `BitSet`, `SmallArray`, `Pool`; the
  `Array` methods the examples use (`retain`, `sort`, `extend`, …) and the
  `String` API are largely absent (F-080). For a Python-feel language the
  standard library *is* the ergonomics; see the Part XV section for the full
  list.

#### B.11 Memory facilities (Part IX §3–§8)

* **F-089** `ERRATUM` S3 — **`[UNS-8]` and `[LEX-11]` contradict each other about
  whether a comment can produce a warning.** `[LEX-11]`: a `##` comment not
  followed by a declaration *"is discarded in silence: a comment never affects
  compilation, and that includes producing a warning"*. `[UNS-8]`: the compiler
  *"MUST emit `W3012 unsafe block with no SAFETY note` … when the block is not
  preceded by a `## SAFETY:` doc comment"* — a `##` before a statement, which
  `[LEX-11]` discards. And the spec's own examples write the note as a
  same-line `# SAFETY:` (IX.4, `[DSJ-7]`'s block), so both would warn.
  Proposal: `[UNS-8]` names the comment form explicitly (a `# SAFETY:` line
  comment on or directly above the `unsafe:` line), and `[LEX-11]` gets an
  exception clause for it; the lexer's side table of comment spans (II.7)
  already records what is needed.
* **F-090** `ERRATUM` S3 — **the `assert_disjoint` example uses `dst` after
  moving it.** `[DSJ-1]`: *"It **consumes** `a` and `b`"*; `dst` is a
  `MutSpan[f32]`, move-only (`[SPN-3]`). The `None:` arm then calls
  `fallback_overlapping_blend(dst, src)` — `E3040 use of moved value`. Also the
  pattern `Some(d, s)` does not match the documented `Some((a', b'))` tuple.
  Proposal: return the views on failure too — `Result[(A, B), (A, B)]` — or take
  reborrows so the originals survive the `None` path.
* **F-091** `ERRATUM` S4 — **`[ALC-3]` declares `static ALLOC: dyn Allocator`**, an
  unsized type in a static, which IV.9 forbids (`dyn I` only behind a pointer).
  And `[PHIL-11]` cites `[ALC-1]` for "panics on exhaustion"; that is `[ALC-4]`.
* **F-092** `GAP` S4 — **phantom type parameters.** `struct Handle[Tag]` never uses
  `Tag` in a field. Say whether an unused type parameter is legal (it should be
  — Python-feel means no `PhantomData` ceremony) and what variance/`Send`/
  `Sync` it gets.

#### B.12 Effects and contracts (Part X §1–§2a)

Effects are **inferred** and contracts are opt-in, which is the right shape for
the goal (no Rust-style annotation burden). The findings are about edges.

* **F-093** `DESIGN` **S2** — **`Map`/`Set` iterate in unspecified order; Python's
  `dict` iterates in insertion order.** `[DET-2]` makes `Map`/`Set` iteration a
  `Nondet` source, and `[ARN-5d]` says *"A stable-insertion-order map would be a
  separate API revision"*. Python users rely on dict order, and deterministic
  iteration also takes `Map` iteration **out** of `Nondet`, so `@deterministic`
  code (lockstep, replay) can iterate maps. A compact insertion-ordered layout
  (CPython's since 3.6, Rust's `indexmap`) is as fast or faster for iteration
  and smaller. Proposal: `Map`/`Set` are insertion-ordered by specification;
  an unordered variant can exist for the rare case that wants it.
* **F-094** `GAP` S3 — **is `DefaultHasher` seeded per process?** `[HASH-2]` leaves
  the algorithm implementation-defined. A random seed (HashDoS protection)
  makes every map operation order `Nondet` and breaks replay; a fixed seed makes
  maps attackable from untrusted input. The spec must pick (fixed seed, and
  F-093's insertion order makes the seed invisible to iteration either way).
* **F-095** `DESIGN` S3 — **`[EFF-2]`: a `fn(A) -> R` parameter "is assumed to carry
  all effects unless written `@noalloc fn(A) -> R`"** — but `[CLO-3]` makes that
  parameter a monomorphised generic, so the actual closure and its effects are
  known at each instantiation. As written, a `@noalloc fn apply(f: fn(i32) ->
  i32)` cannot call `f` without annotating the function type, even when every
  caller passes a non-allocating closure. Proposal: for the statically
  dispatched (generic) form, check contracts after instantiation, and require
  the annotation only for `dyn`/boxed callables.
* **F-096** `ERRATUM` S4 — **`RuntimeCheck` has four kinds; `[EFF-17]` says
  `@no_runtime_checks` "excludes all five kinds"** (X.2 says "all four").
* **F-097** `ERRATUM` S4 — **`[EFF-16]`'s last two sentences have no clear
  antecedent**: *"It is therefore outside `[EFF-17]`'s `@nopanic(explicit)` and
  outside `[EFF-19]`'s default `@realtime` set, so a frame-path function MAY
  carry contracts."* — "It" appears to mean the `RuntimeCheck`-covered panics,
  but reads as `Panic(Explicit)`, which is the opposite. Rewrite.
* **F-098** `DESIGN` S4 — **`@realtime`'s default set includes
  `@nopanic(explicit)`, and `[EFF-16]` makes every division by a non-constant,
  every shift by a variable, and every `unwrap` a `Panic(Explicit)`.** A
  realtime audio/physics function cannot write `x / n`. That may be intended;
  if so, the diagnostic should offer `checked_div`/`n.max(1)`-style fixes, and
  the `docs/errors` page should say so up front.
* **F-099** `ERRATUM` S3 — **(refines F-021)** the Part III §7 attribute table lists
  `@nopanic`(v2) and **omits `@nopanic(explicit)`**, which `[EFF-17]` makes a v1
  contract and `[EFF-19]` puts in `@realtime`'s default set.

#### B.13 Inspection, first hour, cost model (Part X §3–§4)

* **F-100** `GAP`/`DEFECT` **S2** — **the first-hour path does not exist.** `[TOOL-4]`
  makes M0 — `install → ember new hello → cd hello → ember run` in under five
  minutes — a Phase 0 exit criterion whose regression *"blocks a release"*.
  At `761ce7c`: `ember new` → `unknown command`; `ember test` → unknown;
  `ember toolchain install cc` (`[TOOL-2]`, *"a committed deliverable"*) →
  unknown; `ember --version` prints only `ember 0.1.0`, not the language
  version, backend and resolved C compiler/linker `[TOOL-4]` requires;
  `ember inspect --cost` (`[COST-4]`) → unknown option. `ember fmt` exists.
  For a Python-feel language the first hour is the product. Proposal: treat
  `ember new`, `ember test`, the full `--version` line and the bundled C
  toolchain as the next milestone, ahead of further ownership refinements.
* **F-101** `ERRATUM` S3 — **is the stale-handle check removed in `shipping`?**
  Part I.4 table: resource handle liveness is checked *"always outside
  `shipping`"*. `[COST-3]`: *"Stale-handle (generation) check — guaranteed
  required — `[HND-1]`'s guarantee is the check; `get_unchecked` is the
  `unsafe` opt-out"*. One says shipping drops it, the other says only
  `unsafe` does.
* **F-102** `PERF`/`DEFECT` S2 — **"release wraps and emits nothing" is only true
  if the C is written so that wrapping is defined.** (`[CG-C-1]` already requires
  it: *"wrapping ops use unsigned arithmetic and cast back"* — so the compiler is
  wrong, not the text; the release C for `s += x` is a plain signed `+`.) `[COST-3]`'s overflow row
  claims zero cost in release. In C, signed `+` that overflows is UB, and the
  audit (UB-1: `ub1_overflow_wrap_miscompile`, `ub1_wrap_loop_miscompile`)
  shows the backend emits plain signed arithmetic, so clang/MSVC assume no
  overflow and **miscompile** (reproduced on MSVC/clang at `761ce7c`: debug
  `false`, release `true`; the wrap loop returns `-1`). Doing it right
  (unsigned arithmetic with casts, or `-fwrapv`) forgoes some of the loop
  optimisations C compilers get from UB. Proposal: state the real cost in the
  row, and put a decision to the owner: keep Rust's "wrap in release", or take
  Swift's "trap in release too" (safer, Python users never see silent
  wraparound, typically 1–3% cost, and the C backend then emits
  `__builtin_*_overflow` checks that are well defined).
* **F-103** `DOCS` S4 — **`[TOOL-1]`–`[TOOL-4]` sit inside X.3 "Inspection"**, in
  the effects Part; they are toolchain rules and belong in Part XX. `[TOOL-4]`
  also carries the whole M0 milestone definition in one run-on bullet.

#### B.14 Concurrency (Part XI)

(Nothing in Part XI is implemented: `std` has no `thread`, `sync` or `jobs`
module, so these are text findings.)

* **F-104** `ERRATUM` **S1** — **`[THR-1]` admits a `Sync` class with a plain
  mutable scalar field, and scalar field writes through a handle are
  unchecked.** `[THR-1]`: a class is `Sync` iff every field's type is
  interior-synchronised *"or a deeply immutable value type"*. `i32` is a
  deeply immutable value type — the *type* has no mutating API — but a non-`let`
  field of that type can be assigned through any handle, and VIII.3 performs
  scalar field writes *"directly with no check"*. So:
  ```
  @sync class Counter:
      n: i32 = 0
  # two threads each run `c.n = c.n + 1` on one handle: a data race in Safe Ember
  ```
  Together with F-078 (no access word on `Sync` classes) this breaks
  `[PHIL-10]`. Proposal: the test is per **field**, not per type — a `Sync`
  class's non-`let` fields must be interior-synchronised (`Atomic`, `Mutex`,
  `RwLock`, channel end); a `let` field must be of a deeply immutable type. (`[THR-1]`
  also ends with a dangling clause, *"— because handles alias, a plain mutable
  field shared across threads would be a data race."*)
* **F-105** `ERRATUM` S3 — **the job-system example breaks two rules.** XI.5:
  `jobs.submit(fn(): build_visibility(scene, mut visible))`. (1) a non-`owned`
  closure borrowing `scene`/`visible` outside a `JobScope`, which `[JOB-2]`
  forbids and `[CLO-7]` (`jobs.submit` *"MUST declare … `owned f`"*) rejects;
  (2) `mut visible` **at a call site**, which `[FN-2a]`/`OQ-13` rule out (*"A
  call site never writes the mode"*). Same `mut` at a call site in
  `submit_after`.
* **F-106** `GAP` S3 — **what `return`, `break`, `continue` and `?` mean inside
  `with scope = thread.scope():`.** `[THR-5]` defines the `with` form as *"sugar
  for `thread.scope(fn(scope): …)`"*, so a `return x` in the block would return
  from the lambda, not the function the user sees — the opposite of every other
  `with` block. Specify it (forbid with a diagnostic, or desugar so the jump
  leaves the enclosing function after the join). Same for `jobs.scope`.
* **F-107** `DESIGN` S4 — **async is v2.** Python users know `async/await`, but
  for Ember's workloads coroutines plus jobs cover it; no change proposed beyond
  making sure F-074's "gen fn as generator" does not paint async into a corner.

#### B.15 Data-oriented programming, errors, compile time (Parts XII–XIV)

* **F-108** `ERRATUM` S3 — **`[CT-1]` excludes allocation from comptime and then
  supports allocating types.** *"Any Ember function whose transitive effect set
  ⊆ `{Panic}` … may be called at compile time"*, then *"Supported: …
  `Array`/`String`/`Map` (interpreted heap)"* — every one of which carries
  `Alloc` (X.1). Proposal: `⊆ {Panic, Alloc}` with allocation served by the
  interpreter heap.
* **F-109** `GAP` S2 — **what happens when comptime-materialised heap data is
  mutated at run time.** `[CT-5]` materialises comptime `Array`/`String`/`Map`
  results as static data. A `static REG: Mutex[Array[T]]` whose array is
  static data and then `push`es will grow — and the runtime will `free` or
  `realloc` a pointer into the data segment. Specify it (copy-on-first-grow,
  or statics of owning types are immutable). Ties to F-055.
* **F-110** `ERRATUM` S3 — **`comptime` is used as an expression with a block
  value, which the grammar does not have.** XIV.1: `const SIN_TABLE: [f32; 256] =
  comptime:` followed by a block whose last line is `t`. Part III has only
  `comptime_block` (item) and `comptime_stmt` (statement), and `[GRM-11]`
  excludes block expressions from v1. Either add `comptime_expr` with a
  defined result rule, or rewrite the example with a `comptime fn`.
* **F-111** `ERRATUM` S4 — **more syntax in examples that the grammar lacks:**
  `SoA[Particle].Ref` (generic arguments mid-path; `path_type` allows them only
  at the end), `Io(@from io.Error)` (an attribute inside `variant_fields`),
  `lanes = f32xN` (a type used as a value), `r.store(mut out[i..i+8])` and
  similar (`mut` at a call site, against `[FN-2a]`), `particles.get(i) ->
  Option[Particle]` (a signature inside a non-`ignore` ```` ```ember ```` block).
* **F-112** `ERRATUM` S3 — **`@derive(SoA)` generates a different `SoA[T]` per `T`,
  which is specialisation** — `[TYP-19]` says there is none in v1. Say what
  mechanism produces a per-`T` layout for a generic name (a compiler-known
  type constructor, like `Cell`/`RefCell` under ADR-019, is the honest answer).
* **F-113** `DESIGN` S3 — **a default error type would shorten most signatures.**
  Python code raises anything. In Ember every fallible function spells
  `Result[T, SomeError]`, and the "any error" form is `Result[T, Box[dyn
  Error]]`. Proposal: `Result[T, E = Box[dyn Error]]` (a default type
  parameter, which IV.7 already has), so `fn load(path: str) -> Result[Texture]`
  works and `?` erases through `[ERR-8]`.
* **F-114** `GAP` S3 — **`[DRV-1]`: every derive is specified as a hand-written
  `extend` in `std/derive/*.em`, and the generator "MUST produce the same MIR".**
  There is no `std/derive/` directory (`std/src/` has six files). The
  reference the rule depends on does not exist yet.

#### B.16 Standard library (Part XV)

* **F-115** `DESIGN`/`PROCESS` **S1 (strategic)** — **the build order is inverted
  against the goal.** What `std/src/` implements today (386 lines): the
  `with_views2/3/4` and `with_views2/3/4_mut` callback helpers, the hashing
  protocol, `ArenaArray`/`ArenaMap`, the four Span iterator types, core
  interfaces. What it does not: `Map`, `Set`, the `String` API (no
  concatenation, no `to_string`, no `String("x")` — F-080), `std.math`'s
  `min`/`max`/`sqrt`/`Vec3`, `std.io`, `std.fs`, `std.time`, `std.process`,
  iterator adapters (`map`, `filter`, `sum`, `collect`), `print` of a `String`
  (F-012). Meanwhile the compiler carries multi-region views, `@latebound`,
  callable mode vectors, EMIF artifact schemas 1–5 and eight language-version
  selectors. A Python programmer's first program — read a file, split lines,
  count words in a dict, print an f-string — cannot be written. Proposal: a
  "first programs" milestone ahead of further borrow-system work: `ember new`,
  f-string printing, `String` ops, `Map`/`Set`, `Array` methods, iterator
  adapters, `std.io`/`std.fs` basics, `std.math` scalars, and the I.5 showcase
  compiling and running.
* **F-116** `ERRATUM` S3 — **`min`/`max`/`clamp`/`abs`/`sqrt`/`sin` are declared in
  `std.math`, but the spec's examples call them unimported** (IV.2a, V.3, V.5,
  XIV.1's `sin(...)`). Either they are prelude names (add to `[MOD-5]`) or every
  example needs `from std.math import …`. (Refines F-030.)
* **F-117** `ERRATUM` S4 — **`[STD-8]`'s mandated help text is Rust syntax**:
  *"`coll.iter().any(|e| e == x)`"*. The Ember form is `coll.iter().any(fn(e) =>
  e == x)`. A normative `help` that does not compile fails `[PHIL-8a]`'s own
  test (the help, applied literally, must produce a compiling program).
* **F-118** `ERRATUM` S4 — **`[TXT-2]` still names `CppString.as_str()`**, which the
  0.8.2b change log (row 1) says was corrected to the fallible `.to_str()`.
* **F-119** `GAP` S3 — **`print`/`println` have no signature.** `[STD-2]` says
  `println("literal")` and `println(some_str)` do not allocate, and the compiler
  also prints integers, bools and floats, so it is overloaded in some
  compiler-known way — while `[TYP-26]` forbids overloading. Specify it
  (`println[T: Display](x: T)`), say whether several arguments are allowed
  (Python's `print(a, b)`), and include `String` (F-012).
* **F-120** `ERRATUM` S3 — **`unsafe(reason = "…")` (`[UNS-9]`) is not in the
  grammar**: `unsafe_stmt := "unsafe" ":" block`. Every unsafe block now wants
  a reason category (`L3018`), a `SAFETY` note (`W3012`, F-089), and, for
  `unsafe fn`, `@safety(...)` (`L3015`) — three annotation channels for one
  fact. Consider folding the reason category into the `SAFETY` note
  (`# SAFETY(ffi): …`).
* **F-121** `DOCS` S4 — **sections filed under the wrong Part**: `XV.4a` is fine,
  but `VIII.5a Cycle diagnosis` and `IX.5a Unsafe categorisation` sit inside
  Part XV, after the string model. `[STD-4]` ends with an unrelated sentence
  about `debug_assert*`.
* **F-122** `ERRATUM` S4 — **the `[MOD-5]` prelude list omits names Part XV puts in
  the prelude**: `eprintln`, `todo`, `unreachable`, `Ordering`, the `mem.*`
  helpers, `Range*`. (Adds to F-047.)

#### B.17 Foreign function interface (Part XVI)

* **F-123** `DEFECT`/`GAP` **S2** — **Ember cannot call C at all today, and the
  C-import directive is silently ignored.** `import c "math.h"` compiles and
  runs with no effect (same silent-accept class as F-004). A manual block
  ```
  unsafe extern "C":
      fn abs(x: i32) -> i32
  fn main():
      unsafe:
          println(abs(0 - 5))     # E1010 cannot find `abs` in this scope
  ```
  fails to resolve the declared function. "Fast like C" for a real program
  means calling C libraries; `[FFI-10]` manual declarations are the smallest
  useful piece and should come first, before any C++ work.
* **F-124** `ERRATUM` S3 — **two more unapplied editorial instructions inside
  normative rules** (F-022 was the first): `[FFI-34]` (line 3873) contains
  *"Replace "Claiming a grade whose evidence is absent is `E5050`" with: …"*,
  and `[FFI-38]` (line 3885) contains *"Strike "or exception behaviour" … and
  add: …"*. So the rule text says two contradictory things (`E5050` and, in
  the pasted replacement, `W5050` + `E5050` only under `--require-evidence`).
  Apply both, and make the completeness gate grep for `Replace "`, `Strike "`,
  `insert:`, `— replace` in rule bodies.
* **F-125** `ERRATUM` S4 — **`std::string` → `CppString` "with `.as_str()`"** in the
  `[FFI-17a]` table (line 3840); every other mention says the fallible
  `.to_str()`. (With F-118, two stale copies.)
* **F-126** `GAP` S3 — **the overlay language has no grammar.** Overlays
  (`overlay c "vulkan/vulkan.h":` … `fn vkCreateBuffer(device: borrowed, …) ->
  status`, `rename … as …`, `hide …`) carry every safety fact of the FFI, and
  `[TST-7]` exempts their blocks because *"Part III defines no
  `overlay_decl`"*. Contract words appear where types go (`device: borrowed`,
  `-> status`), attributes share a line with declarations (against `[ATT-4]`),
  and grades are written inside attribute arguments (`effects=[FFI]
  @instrumented`). Before any importer work, give `overlay_decl` a production.
* **F-127** `DESIGN` S3 — **scope: the C++ importer is a project the size of the
  rest of the compiler.** Part XVI specifies libclang parsing in MSVC mode,
  generated C++ thunks compiled by the project's compiler, a P0/P1/P2 standard
  library mapping, trampoline subclassing of C++ bases, exception
  translation, grades, instrumentation runs and a C++ importer corpus
  (`[CXX-*]`). For the stated goal (C speed, Python types, Rust safety) the C
  side — manual `extern`, then `import c` of plain headers (Zig's best
  feature) — delivers most of the value. Proposal: explicitly schedule C++
  import after a usable C FFI and a usable standard library, and keep the
  C++ rules out of the 1.0 conformance gate unless RageV integration is the
  1.0 goal.

* **F-128** `GAP` S3 — **the "five count contracts" are never listed.** `[FFI-11]`
  (line 4068) ends *"MUST carry a **count** axis:"* and the list that should
  follow is missing; `[FFI-11a]` then requires `E5012` to *"list the five count
  contracts"*. Only `one` is ever named. Recover the list (probably `one`,
  `span(len_of(p))`, `nullable`, a fixed `N`, NUL-terminated) as a SOURCE
  RECOVERY, or ask the owner.
* **F-129** `ERRATUM` S4 — **the `[FFI-39]` example uses constructor syntax the
  grammar does not have**: `init(name: CppString)` inside `extern class` and
  `init(self):` inside an Ember class, both without `fn` — `[CLS-2]` says the
  constructor is `fn init(mut self, …)`. `[FFI-36a]` says *"`[LEX-15]`'s reserved
  set stays at 48 entries"* (it is 49, F-027). `## XX.13` (the C++ corpus) is
  filed in the middle of Part XVI.

#### B.18 GPU host model (Part XVII) and "safety off" settings

* **F-130** `ERRATUM` **S2** — **two package settings turn memory-safety checks off
  for Safe code, and `[PHIL-10]` lists no such exception.** `[EXC-1]`:
  `exclusivity = "unchecked"` (*"shipping only, and then it is UB"*);
  `[GPU-1]`: stale GPU handles are *"unchecked in shipping only with
  `gpu.validate = false`"* — a stale handle then reaches a destroyed or reused
  GPU object. `[PHIL-10]` says a program with no `unsafe` cannot perform a
  use-after-free or access through an invalid reference, with no carve-out
  for manifest keys, and `[DSJ-8]` rejects exactly this shape for
  `assert_disjoint` (*"would move a memory-safety property into a build
  setting, and would be invisible in review because it does not contain the
  word `unsafe`"*). XXIII.2's non-goals are explicit that the exclusivity
  setting is *"the one exception the owner has granted … and it does not extend
  to any other mechanism"* — so `[GPU-1]`'s `gpu.validate = false` contradicts
  XXIII.2 outright. Proposal: either these keys are spelled as `unsafe`
  (e.g. `[profile.shipping] unsafe_unchecked_exclusivity = true`) and counted
  by `ember tcb` as unsafe surface, and `[PHIL-10]` names them; or they go.
* **F-131** `ERRATUM` S4 — **XVII.5 writes `mut` at a call site**
  (`gpu.History[TextureHandle].new(mut device, …)`) — `[FN-2a]` again (see F-111).

#### B.19 Hot reload (Part XVIII)

Carefully designed; nothing of it is implemented, and none of it is on the
critical path for the stated goal. Text findings only.

* **F-132** `ERRATUM` S4 — **the mandated refusal report suggests a signature the
  rules reject**: `[HR-18]`'s example tells the user to *"add `fn
  migrate_from(old: ref OldEnemy) -> Enemy`"*, while `[HR-35]` fixes it as
  `-> Result[Self, ReloadError]`. A diagnostic copied from the spec would
  steer the user into `E2225`/a type error.
* **F-133** `DESIGN` S3 — **`migrate_from` may not contain a single integer `+`**
  unless the compiler proves it cannot overflow. `[HR-35]` forbids any `Panic`
  in the effect set, and `[EFF-15]`'s contract profile makes every unproven
  integer overflow check a panic source; so is every index and every
  division. The spec concedes indexing (`.get(i)`) and division
  (`checked_div`) but not ordinary arithmetic, which is what migrations do
  (`hp * 100 / max`). Proposal: inside `migrate_from`, integer arithmetic is
  wrapping or saturating by definition (it is `@overflow(wrap)` implicitly),
  or overflow maps to `ReloadError.Migration` automatically.
* **F-134** `GAP` S3 — **is a base class's `init` inherited?** XVIII.7 calls
  `Enemy(old.entity)` on `class Enemy(Script)`, where `Enemy` declares no
  `init` and `Script` declares `fn init(mut self, entity: Entity)`. `[CLS-3]`
  synthesises a memberwise constructor when no `init` is declared, but says
  nothing about a derived class whose base has one. Python inherits
  `__init__`; say what Ember does.
* **F-135** `ERRATUM` S4 — **wrong or loose citations**: `[HR-3]` cites
  "`[THR-1]`'s thread spawn" (THR-1 is the `Sync` rule); `[HR-34]` calls its
  approach "`[PHIL-3]`'s" (PHIL-3 is about implicit copies); XVIII.7 puts
  `@static_safe @noalloc` on one line against `[ATT-4]`; `[HR-2]` has "a a
  migration".

#### B.20 Compiler architecture, C backend, performance (Part XIX)

* **F-136** `DEFECT`+`ERRATUM` **S2** — **the mangling scheme is not injective, and a
  collision is an internal compiler error.** `[MNG-1]`: `em_<pkg>_<module path
  with '_'>_<item>`. Module `lib/m_x.em` with `fn f` and module `lib/m.em` with
  `fn x_f` both mangle to `em_lib_m_x_f`. Probe: `error: internal compiler
  error: [BLD-2] import-visible callable 'em_lib_m_x_f' has more than one
  declaration contract`. Underscores are ordinary identifier characters, so
  any `_`-joined scheme collides. Proposal: length-prefix each component
  (Itanium style, `em_3lib3m_x1f`) or escape `_` inside components; and an ICE
  is never the right diagnostic for a user-reachable condition.
* **F-137** `DEFECT` **S2** — **`for x in span:` is rejected.** `[CTL-1]`: `for x in
  v` where `v` is a place borrows `v` and yields `ref T`; `[CTL-3b]` names
  iteration over `Span[T]` explicitly. Probe: `fn total(xs: Span[i64])` with
  `for x in xs:` → `E2040 'Span[i64]' cannot be iterated: it has no 'next'
  method`. `for x in xs.iter():` works. (`Array` works directly; a `Span`
  parameter — the common case in a function — does not.)
* **F-138** `DEFECT` S3 (perf contract) — **`[CTL-3b]`'s guaranteed lowering is not
  implemented.** For `for x in xs.iter():` the release C declares an iterator
  struct (`em_std_collections_SpanIter_i64 __it`), materialises an
  `em_Option_ref_i64` per element and `switch`es on its tag — the exact shape
  `[CTL-3b]` says MUST NOT appear (*"no iterator object in memory, no `next`
  call"*, *"MUST NOT depend on the host compiler's inlining"*). Similarly
  `for i in 0..xs.len(): s += xs[i]` keeps the bounds check inside the loop,
  where `[OPT-2]`'s loop versioning and §4.12's "guaranteed" bounds-check
  elimination say it MUST go. clang cleans both up at `-O2` (see F-140), but
  MSVC `/O2` and debug builds pay it, and the spec made it a guarantee.
* **F-139** `DEFECT` **S2** — **float contraction is never disabled.** `[TYP-9a]`
  requires `#pragma STDC FP_CONTRACT OFF` in every TU plus
  `-ffp-contract=off` (clang/gcc) and `/fp:precise` + `fp_contract(off)`
  (MSVC). `grep` finds none of these in `compiler/` or `runtime/`, and the
  release flags are just `-O2`/`-O3` or `/O2`. On AArch64 (clang's default is
  `-ffp-contract=on`) `a*b + c` becomes an FMA, so results differ between
  machines — which breaks `[TYP-9]`, `[RNG-6]`'s range facts and every
  `@deterministic` claim.
* **F-140** `PERF` **S1 (strategic)** — **there is no performance suite; "fast like
  C" is unmeasured.** `tests/perf/` and `tests/ffi/` are **empty**. XIX.10
  promises `cargo-fuzz` targets, `insta` snapshots and a performance regression
  suite gating releases; none exists (no `fuzz/`, no `insta`/`proptest` in any
  `Cargo.toml`). One crude probe this pass: summing a 20M-element `Array[i64]`
  20 times, Ember `--profile release` (clang `-O2` backend) ~0.5–0.9 s vs.
  hand-written C `-O2` ~0.44–0.51 s — comparable for this loop, because clang
  removes what `ember_opt` does not (F-138). Proposal: a small suite — the
  Benchmarks Game set (nbody, spectral-norm, fannkuch, mandelbrot,
  binary-trees for RC/allocation), an RC-heavy class graph, an f-string/String
  workload, a `Map` workload — each with a C reference, run in CI in
  `release` and `shipping` on clang and MSVC, reporting the ratio.
* **F-141** `DEFECT` S3 — **several promised backend/runtime pieces do not exist**:
  the `[CG-C-3]` cross-module inline header (no `_inline.h` emitted; every
  cross-module call is opaque to the C compiler, which matters for "fast like
  C" once programs have more than one module); `mimalloc` (`[RT-1]`,
  `[OBJ-4]`: *"vendored and used as the default"* — not in the tree);
  backtraces (`[RT-4]`; the runtime prints *"note: backtraces are not
  available in this build"*); the `ember_opt`, `ember_mono`, `ember_resolve`,
  `ember_interp`, `ember_ffi`, `ember_abi` crates of XIX.1's layout.
* **F-142** `TOOLING` S3 — **`compiler/ember_typeck/src/lib.rs` is 20,935 lines in
  one file** (the whole compiler is ~60k). That is where most future edits
  land, and where two agents collide. Split it by concern (names, calls,
  closures, cells/spans, ranges, generics, diagnostics).
* **F-143** `ERRATUM` S3 — **XIX says effect analysis both runs after
  monomorphisation (XIX.1: *"Effect analysis runs **after**
  monomorphisation"*) and checks generics before it (§4.10: *"Before
  monomorphisation, generic functions are checked once with their bounds'
  declared effects"*).** Which verdict decides a contract on a generic? It
  determines whether F-095's `@noalloc` higher-order helpers are writable.

#### B.21 Toolchain, manifest, tests (Part XX §1–§5)

* **F-144** `TOOLING` **S1 (strategic)** — **the instrument that measures the goal
  does not exist.** `[TST-8]`–`[TST-10]` specify `tests/firstweek/`: at least 24
  first-draft programs (text adventure, CSV summariser, scene graph, event bus,
  inventory, path finder …) written by people who have read only Part I and
  Appendix A, committed unmodified, with an acceptance rate published per
  release and an unclassified rejection a release blocker. That is exactly a
  "does it feel like Python" test, and `tests/firstweek/` does not exist.
  Proposal: build it now, before more borrow-system features. The authors
  should be humans (or, as a stopgap, fresh agents given only Part I +
  Appendix A and no other context); record who wrote each.
* **F-145** `ERRATUM` S3 — **`E9010` has two meanings.** `[TYP-9c]`: the toolchain
  cannot honour `@fastmath`/`@fp(...)`; `[MAN-3]`: an unknown key in `[lints]`.
  The compiler registry merges them into one entry ("the toolchain or the
  manifest names something it cannot honour"). One code, one meaning.
* **F-146** `GAP` S3 — **most of the CLI in XX.1 is missing** (extends F-100):
  besides `new`, `test`, `toolchain`, the listing promises `bench`, `lint`,
  `doc`, `clean`, `bind`, `shader-bind`, `inspect <item>`, `inspect --alloc`,
  `--cost`, `--deterministic`, `explain <rule-id>`, `explain --borrow`, `build
  --reload`, `run --hot`, `--timings`, `--build-id`, `--report=…`. The shipped
  `--help` lists `build`, `run`, `check`, `explain <CODE>`, `explain --cycle`,
  `inspect --safety`, `inspect --cycle`. `[CLI-15]` requires every listed
  surface to appear in `ember --help`; the gate that should catch the gap is
  the one `[DIA-6a]` describes. Proposal: mark unbuilt commands in the spec's
  listing (e.g. `(Phase n)`), so the help text and the spec agree.
* **F-147** `DESIGN` S3 — **the example manifest makes shipping builds UB on an
  exclusivity violation by default** (`[profiles.shipping] exclusivity =
  "unchecked"`). A reader copies the example. See F-130; at minimum the
  example should show the safe default and the unchecked form commented out.

#### B.22 Diagnostics (Part XX §6)

`[DIA-17]` ("Python-form fix-its") is the most goal-aligned rule in the
document. None of its five fix-its is implemented; probes at `761ce7c`:

* **F-148** `DEFECT` **S2** — **`xs[-1]` compiles and panics at run time with index
  18446744073709551615.** `[DIA-17]`: *"`xs[-1]` on a `usize`-indexed container
  suggests `xs.last()`"*; `[LEX-16]` + `E2010` say a literal that does not fit
  its target type is an error. `ember check` accepts it; `ember run` panics
  `index 18446744073709551615 is out of bounds for a length of 1`. This is the
  audit's FE-2 (negative literal accepted as unsigned) in the place a Python
  user will hit it first.
* **F-149** `DEFECT` S3 — **`x is None` gives two wrong errors instead of the fix.**
  `[DIA-17]`: suggest `x.is_none()`. Actual: `E2060 cannot tell which 'Option'
  this 'None' is` and `E2020 'is' requires related class handles, found
  'Option_i32' and '<error>'` — the second leaks a mangled name and reports on
  an error type, which `[DIA-14]` forbids. **Design proposal:** simply *accept*
  `x is None` / `x is not None` on an `Option` — it is unambiguous (`is` on an
  `Option` has no other meaning), it is what every Python user types, and the
  spec already knows the translation.
* **F-150** `DEFECT` S3 — **`let x = 5` gives a raw parse error** (`E0100 expected
  an expression`, `E0100 expected the end of the line`). `[DIA-17]`: suggest `x =
  e` with a note about `let` fields.
* **F-151** `DEFECT` S4 — **`a < b < c` fix-it is incomplete**: the help says
  `write 'a < b and b < c'` but `[DIA-17]` also requires the note that `b` is then
  evaluated twice. (Moot if F-049 is adopted.)
* **F-152** `ERRATUM` S4 — **`[DIA-7a]`'s table is flattened into one line**
  (line 5506): a Markdown table written inline after the rule text, which
  renders as prose and which any tool reading it row-by-row misparses. It is
  the table `rule_index.py` is supposed to check against `ember_diag::codes`.

* **F-153** `ERRATUM` S4 — **(the rule behind F-006)** N1 admits a candidate at
  *"Damerau–Levenshtein distance ≤ 2 (or ≤ ⅓ of the name's length)"*. With
  "or", every name of length ≤ 6 accepts distance 2, so `io` → `Eq` and `min` →
  `main`. Python's `difflib`-style suggestions use a ratio cutoff. Proposal:
  `distance ≤ max(1, min(2, len/3))`.
* **F-154** `ERRATUM` S4 — **stale conditional and dead codes in the diagnostics
  part**: shape O5's row still carries *"If RFC-023 is declined, the required
  text is instead …"* although `[CLO-6]` adopted it; N8 demands *"the one-token
  fix at the call site"* for a mode mismatch although `[FN-2a]` says a call
  site never writes a mode; the 0.6 registry paragraph still lists
  `E4050`–`E4057`/`E4060`–`E4064` "(contracts and verification)", a layer 0.6.2
  removed — mark them reserved-retired like `E4071`.

* **F-155** `DESIGN` **S2** — **integer `/` is the silent Python trap.** `[DOC-2]`
  itself lists `/` first among the *silent* differences a "coming from Python"
  chapter must disclose: `7 / 2` is `3` in Ember and `3.5` in Python 3. Python
  changed exactly this in 3.0 (PEP 238) because it caused real bugs. A
  disclosure table does not stop the bug. Options for the owner: (a) add `//`
  (floor division, Python meaning) and make `/` on two integers a compile
  error whose fix-its are `//` or a float conversion; (b) keep C semantics and
  lint `int / int` whose result flows into a float. (a) is the "types like
  Python" answer at zero runtime cost; (b) is the "C-like" answer. Pair with
  F-065 (`%` sign).
* **F-156** `GAP` S2 — **no user guide and no "coming from Python" chapter.**
  `[DOC-2]` makes `docs/book/` a 1.0 artefact and the landing page, with a
  mandated Python-migration table and a list of what behaves exactly like
  Python. `docs/book/` does not exist; the only prose is the 870 KB
  specification, which is not a way to learn a language. For the stated goal
  this is as important as any compiler feature.
* **F-157** `ERRATUM` S3 — **C++ exception policy: "no default" vs a default.**
  `[FFI-43]`: *"there is no default: an import whose policy the header and
  overlay both leave unstated is `E5061`"*. `[FFI-24a]`: where the
  specification is potentially-throwing or dependent, *"the fact is `unknown`
  and `[FFI-24]`'s `Result[T, CppError]` shape applies"* — which is a default —
  and `[FFI-38]`'s pasted amendment says *"`[FFI-24]`'s catch-all establishes
  it universally"*. Owner ruling needed.
* **F-158** `ERRATUM` S4 — **more amendment text appended instead of applied**:
  `[TST-13]` ends *"add: an instrumented run in which …"* (line 5754), a fourth
  pasted instruction (F-022, F-124); `[TCB-5]` says the evidence record *"is
  stored under `target/<profile>/ffi-evidence/`"* and, two sentences later,
  that its default location is *"`.ember/ffi-evidence/` … not under
  `target/`"*; `[TCB-4]` still lists a "solver" category and `[TCB-3]` "every
  assumption a proof relied on", both from the removed prover.

#### B.23 Implementation plan and milestones (Part XXI)

* **F-159** `PROCESS` **S2** — **Phase 1 is recorded as complete, but its exit
  criterion ("conformance for Parts II–VI except closures/generics") is not
  met.** Phase 1's list includes f-strings (F-012: broken end to end),
  `String`/`str` with SSO (F-080: no construction, no concatenation),
  modules/imports across files (F-003: `import a.b.c` does not bind),
  `Option`/`Result` and `?` (fine), plus things this pass found missing from
  Parts II–VI: `**` (F-066), `if`-let conditions (F-016), `()` as `void`
  (F-002), `import c`/`extern` (F-123). Proposal: re-audit the Phase 1 exit
  against the grammar and Parts II–VI rule by rule, and keep a
  machine-checked list of "specified but not implemented" instead of phase
  labels. (`COLD-START` §4's own words — "a directory named for a rule is not
  coverage of the rule" — apply to phases too.)
* **F-160** `DEFECT` **S2** — **indexing does not produce a place that can be
  returned by reference, so milestone M2 cannot be written as specified.**
  XXI.3 says milestone tests *"must exist verbatim"*; M2 is `fn first(xs:
  Array[i32]) -> ref i32: return xs[0]`. Probe: `E2020 expected 'ref i32',
  found 'i32'` (with or without the `Array[i32]([1, 2])` constructor, which is
  also missing, F-077). `tests/milestones/m2_…em` was rewritten over `ref
  i32` parameters with a comment deferring the real form to "block D". IV.8's
  `Index.index(self, i) -> ref Output` is the rule; M2 as written is the
  acceptance test for it.

* **F-161** `DEFECT` **S1-class (silent contract)** — **every contract attribute is
  accepted and ignored, and so is any made-up attribute.** Probes at
  `761ce7c`:
  ```
  @noalloc
  fn f():
      a: Array[i32] = Array[i32]()
      a.push(1)              # accepted, runs — milestone M4 expects E4001
  @deterministic @static_safe @nosync   (each on its own line) — accepted, no check
  @totally_made_up
  fn k(): pass             # accepted — [ATT-1] requires E0104
  ```
  The spec names this exact failure as the worst one (`[TYP-9c]`: *"a silently
  ignored … attribute is the worst outcome, because the programmer believes
  the contract holds"*). Until effects are implemented (Phase 4), every
  contract attribute must be rejected with a "not implemented yet" code
  (F-009), and `[ATT-1]`'s unknown-attribute check must run now — it is a
  table lookup.
* **F-162** `DEFECT` S3 — **milestone M3 fails: `mem` is not in the prelude.**
  `mem.drop(a)` → `E1010 cannot find 'mem'`, with the suggestion *"did you
  mean `Res`?"* (F-153's threshold again). Part XV puts `mem.{take, replace,
  swap, drop, forget, size_of, align_of}` in `std.core`'s prelude; `[MOD-5]`
  does not list `mem` (F-047).
* **F-163** `ERRATUM` S4 — **XXI.5 lists `book/` as "(user guide, v1.1)"**;
  `[DOC-2]`: *"The user guide is a 1.0 artefact, not v1.1"*. `[GATE-1]` defines
  "known soundness defects" as those *"recorded in `docs/spec-errata.md"`* — the
  document-errata ledger — while compiler soundness defects live in
  `docs/DEFECTS.md` (and today in `tasks/audit/repro/`); `[GATE-3]` covers
  them, but `[GATE-1]`'s wording should point at both.

#### B.24 RageV integration plan (Part XXII)

* **F-164** `ERRATUM` S3 — **the Stage 0 example uses syntax and semantics the
  rest of the document rejects**: `unsafe:` as an **expression** (`ok = unsafe:
  (…)(e.id, …)`) — the grammar has only `unsafe_stmt`; `ref mut out` of an
  **uninitialised** `out: Vec3`, which `[BRW-7]` makes `E3050`; `*const
  c.NativeApi` (Ember spells a const raw pointer `*T`); C-style `/*script*/`
  comments; `@field`, `@callable`, `@derive(Script)` — none in Part III §7's
  attribute table or a registered plugin namespace (`[ATT-1]` → `E0104`);
  `Layer[T]`… `Layout[T]`-compatible arrays (XXII.3) — `Layout[T]` is defined
  nowhere. An "unsafe expression" form is worth considering on its own
  merits (Rust has it; it keeps `unsafe` scopes tight, which `[UNS-3]`'s
  L3010 wants), but it must be in the grammar first.

#### B.25 Owner decisions, non-goals, glossary (Part XXIII)

* **F-165** `ERRATUM` S4 — **stale lists in XXIII**: the glossary's *Effect* row
  lists `Alloc, Sync, Panic, Unsafe, FFI, Block` (missing `Lock`, `Io`,
  `Nondet`, `RuntimeCheck(k)`); its *Reason code* row omits
  `establishes_static_fact`; `OQ-26` says the reserved set has 48 entries (49,
  F-027).
* **F-166** `PROCESS` S3 — **three owner decisions bear directly on the goal and
  should be re-put with the evidence from this pass**, not silently reopened:
  `OQ-1`/ADR-001 (f32 float default; fine for engine code, and `[LEX-17a]`'s
  W2015 already guards it — no change proposed), `OQ-2`/ADR-002 (block
  scoping; F-014 proposes a narrow refinement), `OQ-15` (`::`; F-018), and
  `OQ-7`/ADR-007 (cycles leak; F-082 proposes only on-by-default reporting).
  Each proposal in this file that touches an `OQ` names it, so the owner can
  rule with the history in view.

#### B.26 Appendix A (the syntax quick reference)

* **F-167** `ERRATUM`+`DEFECT` S3 — **the quick reference does not type-check, and
  one of its errors is in the reference itself.** `[TST-6]`: the fixture *"is
  annotated `#$ test: compile-pass`"*; `docs/spec-source/appendix-a.em` is
  annotated `#$ test: syntax-pass`. `ember check` on it: **31 errors**. Most are
  names the snippet never declares (`Entity`, `Player`, `Cmd`, `count`, …),
  which a quick reference may do — but: `fn demo(n: usize, out: MutSpan[f32],
  …)` then `out[i] = f(inp[i])` is `E3023 cannot mutate borrowed parameter
  'out'`, a **real error in the reference** (it needs `mut out`), and
  `Display`/`Debug` — `[MOD-5]` prelude names — resolve to nothing (`E1010
  cannot find interface 'Debug'`; only `std.core`'s `Default`, `Eq`, … are
  wired). The fixture also selects `#! language "0.9.6"` and puts `@noalloc
  @simd` on one line (`[ATT-4]`). Proposal: declare the missing names in a
  hidden prelude of the fixture, fix `mut out`, make the gate type-check it,
  and wire `Display`/`Debug` into the prelude.
  (The `compile-pass` shortfall itself is already on record as ERR-020,
  deferred; the `mut out` error and the unresolvable prelude interfaces are
  new.)

#### B.27 Part XXIV — the 0.9-series additions

* **F-168** `DESIGN` **S1 (strategic)** — **`with_views*` and `@latebound`: a chain of
  features whose net effect is to reject safe programs.** Evidence:
  1. The six helpers in `std/src/borrow.em` are identity forwarders — every
     body is `return f(a, b)` (or `f(a, b, c)`, …). Their only content is the
     `@latebound` marker on `f`.
  2. `@latebound` (`[FN-6b]`, `[LT-7]`, `[LT-10]`) forbids the callback's result
     from containing any input region. Probe:
     ```
     fn first(a: Span[i32], b: Span[i32]) -> Span[i32]: return a
     direct  = first(a.as_span(), b.as_span())               # accepted
     escaped = with_views2(a.as_span(), b.as_span(), first)  # E3062
     ```
     `escaped` points into `a`, which lives in `main` — memory-safe. The
     direct call is accepted; the helper rejects it. This is ODR-015's own
     "historical minimal reproduction".
     And the reverse capability is not needed: an **unmarked** callback
     already receives views of the callee's local storage (probe in F-069),
     so `@latebound` enables nothing that was not already possible.
  3. The error is reported **inside the standard library** (`std\src\borrow.em:9:1`),
     not at the user's call.
  4. To make the `_mut` helpers expressible the language grew callable
     parameter modes in function types (`fn(mut MutSpan[A])`, `[FN-6]`
     amended), the mode vector carried through `Callable[Args, R]` as hidden
     type identity (`[FN-6a]`), a new code `E2228`/shape B15, `[LT-8]`–`[LT-13]`,
     `[LT-11a]`, `[TST-20]`, `[TST-21]` and the EMIF callable-summary schemas.
  5. XXIV's own H14 header says the problem the helpers solved — composing
     views of independent lifetimes — *"is independently superseded by the
     source-language feature in `0.9.5_Hardened_1`"* (multi-region views).
  Against the goal ("Rust-like safety minus the annoying parts") this is the
  clearest single simplification available. Proposal for the owner:
  retire the `with_views*` family and the source-level `@latebound` modifier;
  keep late-bound callback regions as an internal property of the few APIs
  whose soundness needs them (`thread.scope`, `jobs.scope` — `[THR-5]`,
  `[JOB-2]` — which already carry the region through the `Scope` view type);
  revisit whether callable parameter modes in `fn(...)` types are still
  needed once `_mut` helpers are gone. Classified per `[BUD-5]`:
  **SIMPLIFIES**. (Owner decisions ODR-005/006/007/015 are what this would
  revisit; the evidence above is new since they were taken.)
* **F-169** `DEFECT` S3 — **diagnostics that originate in a callee's contract are
  reported at the callee, not the caller.** The `E3062` above points at
  `borrow.em:9:1` with no label on the user's line 13. `[DIA-1]` requires a
  primary span; for a contract violated *by an argument*, the primary span
  must be the call site, with the callee's declaration as a secondary label.

* **F-170** `ERRATUM` S3 — **the headline multi-region example does not compile
  under the document's own rules.** MRV-6: `fn integrate(v:
  TransformPhysicsView, dt: f32): … v.positions[i].x += …` — `v` is a borrowed
  parameter, and writing through its `MutSpan` field is a mutation of `v`
  (`[FN-1]`: the callee *"cannot mutate"* a borrowed parameter). The compiler
  agrees: the same shape gives `E3023 cannot mutate borrowed parameter 'v'`.
  It needs `mut v`. (It also calls `min` unimported — F-030.) Multi-region
  views themselves are **the right design for the goal** — inference only,
  no lifetime syntax, erased at runtime — so this is an example fix, not a
  feature concern.

* **F-171** `GAP` **S2** — **the machine-readable implementation matrix the spec
  requires does not exist.** XXIV *Implementation Completeness Matrix*: *"The
  repository MUST maintain a machine-readable implementation matrix with one
  row per normative rule"* (parser/resolver/typeck/borrowck/…/tests/docs status
  per rule). No such file is in the repository (`tools/` has none; no
  `*matrix*` outside one test). It is the instrument that would have
  prevented F-159 (Phase 1 marked done with Parts II–VI features missing) and
  F-161 (contracts silently unenforced). Proposal: generate it from
  `rule_index.py` + the conformance tree, with `NOT_STARTED` as the honest
  default, and make "not implemented" diagnostics (F-009) cite the row.
* **F-172** `ERRATUM` S3 — **Part XXIV's normative rules have no category.**
  `[CAT-2]` assigns default categories by Part — I–XVIII, XIX, XX, XXII, Part 0
  — and never mentions Part XXIV, although `[IMP-8]` makes it normative. Most
  of XXIV (`[IMP-*]`, `[COMP-1]`, `[VERIFY-*]`, `[LAY-1]`, the canonical-fact
  names `TypeIdentity`/`BorrowCapability`/…) describes **this** compiler's
  architecture, which `[CAT-1]` calls `REFERENCE-IMPLEMENTATION` and `[CAT-3]`
  says must not be cited as the reason for a language rule. Assign XXIV rules
  categories explicitly; most are reference-implementation.
* **F-173** `DOCS` **S2 (strategic)** — **one 870 KB file is three documents.**
  It carries (1) the language (roughly Parts I–XVII), (2) the reference
  compiler's design (XIX, most of XXIV), and (3) project process and history
  (Part 0's ~880 lines of change logs, hardening classes, amendment
  procedure, Appendix H). A Python programmer who opens it meets 1,000 lines
  of process before the first line of Ember. `[DOC-2]`'s user guide (F-156)
  is the real answer for users; for implementers, splitting the file into a
  *Language Reference*, a *Compiler Design* and a *Process & History* record —
  each versioned together, the rule index spanning all three — would make
  every other finding in this file cheaper to act on. (The 0.8.3 change log
  rejected moving history out because "optional is how that stops working";
  keeping it in the same release, in a separate file, answers that.)
* **F-174** `ERRATUM` S3 — **`[GEN-COH-1]` makes impl ownership a *package*
  matter** (*"the implementation has exactly one package owner"*), while
  `[TYP-20]` makes it a *module* matter — a second normative statement of the
  orphan rule at a different granularity (supports F-040). Its example is
  also not Ember syntax: `extend Type: Interface { ... }`.
* **F-175** `GAP` S3 — **what happens when a `Pool` runs out of generations.**
  `[RT-9]`: a slot that exhausts its generation space *"MUST permanently
  retire"*. IX.6's `Handle` has a 12-bit generation (RageV's 20/12 split). A
  pool recycling one hot slot (bullets, particles) at 60k allocations/s
  retires 4,096-reuse slots continuously; all 2^20 slots are gone after ~4
  billion allocations (~19 hours at that rate), after which the pool cannot
  allocate. Specify the failure (a `CapacityError`, not a panic), and give
  `Pool` a wider default generation (e.g. 32/32 in a `u64` handle), keeping
  20/12 only where RageV compatibility needs it (`Entity`).
* **F-176** `ERRATUM` S4 — **`[HR-IMPL-2]` describes reclaiming an old image**
  (*"An old image MUST NOT be reclaimed until every participating thread has
  … advanced to the new epoch"*), while `[HR-4]` says *"Old images are never
  unloaded"*. Reconcile (epochs may reclaim *data*, not code, perhaps — say
  so).

#### B.28 Appendices B and H

* **F-177** `ERRATUM` S3 — **Appendix B is a "mandatory CI" checklist that fails
  against the file it is in.** Item 1: *"Exactly one current version
  declaration exists and names `0.9.7_Hardened_1`"* — the file is
  `0.9.8_Hardened_3`. Item 2 names predecessor `0.9.6_Hardened_6`; item 3's
  selector list lacks `0.9.8`; two items are numbered 35. The closing line
  says *"A failure of any checklist item is a specification CI failure"* —
  so either nothing runs it or CI would be red. Either re-target it at the
  current revision each cut (and run it), or move it to Appendix H as history.
* **F-178** `DOCS` S4 — **Appendix H is 730 lines of audit records that "bind
  nobody"**, plus Part XXIV's own history paragraphs; together with Part 0's
  change logs, roughly a quarter of the file is provenance. See F-173.
* **F-179** `PROCESS` S3 — **1.0 is tied to a full RageV production migration.**
  XXIV's *1.0 Readiness Gate* ends *"The production reference workload MUST
  successfully exercise the complete supported migration path"*, and *RageV
  Final Audit* lists Vulkan, OpenGL, Jolt, ImGui, yaml-cpp, cgltf, editor,
  hot reload…. That makes a language release wait on an engine port. For the
  stated goal the language could reach 1.0 on its own conformance, first-week
  corpus and performance suite, with RageV as a separate integration
  milestone. Owner call.

### Part C — Compiler sweep of everyday features (clean build at `761ce7c`)

Twenty small programs covering what a newcomer writes in week one. **Works:**
enums + expression `match` (F-001's showcase aside), interfaces + generic bound
dispatch, `?` on `Result`, class inheritance + `virtual`/`override`, drop order
(reverse declaration), `while … else`, labelled `break`/`continue`, `defer`,
tuple return + destructuring, `RefCell` + `with`, `Weak` + `upgrade`, `Box`
auto-deref, `mut self` methods on structs, `Arena.alloc`. **Fails:**

* **F-180** `DEFECT` **S2** — **the conditional expression `a if c else b` is not
  implemented** (`E1010 this expression is not supported yet in this phase`).
  It is level 1 of III.5's precedence table and the idiom F-014's help would
  point users to.
* **F-181** `DEFECT` **S2** — **built-in scalars do not implement `Ord`, so no
  generic comparison can be written.** `fn largest[T: Ord + Copy](a: T, b: T)`
  called with `largest(3, 7)` → `E2040 'i32' does not implement
  'std.core.Ord', which 'T' requires`. (With F-042/F-043: which scalars
  implement which interfaces needs one table, and the compiler needs to
  follow it.)
* **F-182** `DEFECT` S2 — **`Option`/`Array`/`str` basics are missing:**
  `Option.unwrap_or` (`[ERR-4]`'s combinator set — `map`, `and_then`,
  `unwrap_or`, `ok_or`, `expect`, `is_some`, …), `Array.pop` (IX.1's list:
  `pop`, `insert`, `remove`, `retain`, `sort`, …), `str.len()` (!), and
  iterating a fixed array `for x in a:` with `a: [i32; 4]` (`E2040 cannot be
  iterated`; `[CTL-3b]` names `[T; N]`).
* **F-183** `DEFECT` S3 — **a single-statement block lambda inside brackets is
  rejected.** `apply(fn(x): total += x, 4)` → `E0106 a multi-statement closure
  cannot be written inside brackets`, but the body is one `small_stmt`, which
  `[LEX-6a]` explicitly allows (*"a lambda's `:` body is a single
  `small_stmt`"*). The `=>` form (`fn(x) => total += x`) is correctly a parse
  error (assignment is not an expression) but its message is a bare
  `expected ')'`; it should say "use `fn(x): total += x`".
* **F-184** `DEFECT` **S1-class (accepted program, broken C)** — **range types reach
  the C compiler broken** — see F-012's broadened note: `println(p)` for `p:
  Percent` emits `ember_println_str(uint8_t)`.

### Part D — The 40 audit reproducers, re-run at `761ce7c`

Run with `python tasks/audit/run_repros.py` from a clean worktree (Windows,
clang/MSVC; no gcc, so the ASan/UBSan column of the runner did not run).
"Still" = the recorded wrong behaviour reproduces.

| Task | Reproducer | Expected (from header) | Observed now | Status |
|---|---|---|---|---|
| CG-1 | cg1_trigraphs_in_string_literals | text verbatim | verbatim on clang/MSVC (no trigraphs by default) | **masked on this host** — still wrong under `gcc -std=c11`; escape `?` in emitted strings |
| CG-2 | cg2_c_keyword_field_names | 10 | C compile fails (exit 1) | still |
| DIAG-1 | diag_const_cascade | one E2130 | E2130 + cascade | still |
| DIAG-2 | diag_duplicate_borrow_error | one E3021 | duplicate E3021 | still |
| DIAG-2 | diag_internal_place_names | names `h.items` | names `h.0` | still |
| DIAG-3 | diag_internal_type_names | user type names | `closure0_env` | still (also F-058, F-149) |
| DIAG-4 | diag_uninit_also_reported_as_moved | one E3050 | E3050 + E3040 | still |
| FE-1 | fe1_capture_callable_parameter | 11 | E1010 cannot find `f` | still |
| FE-1 | fe1_forward_callable_parameter | 6 | E1010 cannot find `f` | still |
| FE-2 | fe2_negative_literal_unsigned | reject E2010 | prints 4294967295 | still (also F-148 `xs[-1]`) |
| FE-3 | fe3_i128_codegen | 25 or clear diagnostic | C compile fails | still |
| PERF-2 | perf2_closure_self_instantiation_hang | diagnostic | `ember check` timeout | still |
| PERF-2 | perf2_polymorphic_recursion_hang | depth diagnostic | `ember check` timeout | still |
| RC-1 | rc1_match_temp_early_return_leak | 3, 103, 2 | 3, 2 | still |
| RC-1 | rc1_weak_upgrade_return_leak | 7, no leak | 7 (leak not measurable without LSan) | presumed still |
| RC-2 | rc2_value_iterator_tuple_handles_leak | 7, 8, 107/108, 0 | 7, 8, 0 | still |
| RC-3 | rc3_move_field_out_of_drop_type | reject E3010 | accepted, prints 1, 0 | still |
| SAFE-1 | safe1_move_out_of_array_index | reject E3011 | accepted | still |
| SAFE-1 | safe1_move_out_of_fixed_array | reject E3011 | accepted | still |
| SAFE-1 | safe1_move_out_of_mutspan | reject E3011 | accepted; crashes (exit 1) | still |
| SAFE-1 | safe1_move_out_of_span | reject E3011 | accepted; crashes | still |
| SAFE-1 | safe1_span_element_into_owned_call | reject E3011 | accepted; crashes | still |
| SAFE-2 | safe2_move_out_of_class_field_alias | reject E3012 | accepted | still |
| SAFE-3 | safe3_mut_and_shared_int | reject E3021 | accepted, prints 1 | still |
| SAFE-3 | safe3_mut_arg_and_element_ref | reject E3021 | accepted | still |
| SAFE-3 | safe3_mut_arg_and_span_same_call | reject E3021 | accepted | still |
| SAFE-3 | safe3_mut_self_and_span_of_field | reject E3021 | accepted | still |
| SAFE-4 | safe4_class_field_view_alias_grow | reject or panic | accepted | still |
| SAFE-4 | safe4_class_field_view_alias_replace | reject or panic | accepted | still (spec gap F-079) |
| SAFE-5 | safe5_control_plain_name | reject E3022 | rejected E3022 | **correct (control)** |
| SAFE-5 | safe5_struct_named_refcell_prefix | reject E3022 | accepted | still |
| SAFE-6 | safe6_arena_capacity_overflow | panic or reject | abort (exit 3) | still (heap corruption before abort) |
| UB-1 | ub1_overflow_wrap_miscompile | false everywhere | debug false, release/shipping **true** | still |
| UB-1 | ub1_u16_multiply_promotes_to_int | 1 without UB | release 1 (UB in C) | still |
| UB-1 | ub1_wrap_loop_miscompile | 2 | release **-1** | still |
| UB-2 | ub2_negate_min | debug panic | no panic | still |
| UB-3 | ub3_negative_shift_amount | debug panic | runs | still (spec gap F-033) |
| UB-3 | ub3_signed_left_shift_overflow | no C UB | C UB | still |
| UB-4 | ub4_float_to_int_cast | saturating | release prints values outside i32 range (`306501908904`) | still |
| UB-5 | ub5_inclusive_range_to_max | 3 | debug panic; release/shipping **infinite loop** | still |

**Summary: 38 of 39 defects still reproduce; the 39th (CG-1) is hidden by the
host C compiler, not fixed.** The 16 SAFE programs are memory-safety holes
in Safe Ember and should come before any feature work.

### Part E — Additional soundness and robustness probes (beyond the audit)

Fourteen further ownership/aliasing probes. **Correctly rejected or checked:**
push while iterating (`E3020`), push while a `Span` is live (`E3021`), returning
`ref` to a local (`E3060`), use after moving a `Box` (`E3040`), move in a loop
(`E3041`), `Arena.reset` with a live allocation (`E3021`), double
`RefCell.borrow_mut` (runtime panic naming both sites), `Shared.get_mut`
while a `get` view is live (runtime exclusivity panic). **New problems:**

* **F-185** `DEFECT` **S1-class (compiler crash on the basic OOP pattern)** —
  **a parent/child class pair with a constructor overflows the compiler's
  stack.**
  ```
  class Tree:
      children: Array[Node] = Array[Node]()
  class Node:
      parent: Weak[Tree]
      fn init(mut self, t: Tree):
          self.parent = Weak(t)
  ```
  `ember check` / `ember run` → `thread 'main' has overflowed its stack` (the
  Rust compiler process dies; no diagnostic). Minimal trigger: two classes
  referring to each other where one has an `init` that assigns the field
  (`class K: held: Option[N]` + `class N: k: K; fn init(mut self, k: K):
  self.k = k`). Without the `init`, both compile. This is the scene
  graph / observer / tree shape `[TST-8]`'s first-week list names explicitly.
* **F-186** `DEFECT` S3 — **the runtime exclusivity panic names a C symbol, not
  the Ember object.** `panic at u11b.em:6:9: exclusivity violation:
  overlapping access to em_main`. `[EXC-6]`/`[EXC-1]` require `<Class>.<field>`
  and, in debug, the location of the *other* active access.
* **F-187** `DEFECT` S3 — **`L3011` fires on calls that cannot re-enter the
  cell.** With a live `RefCell` guard, `a.push(1)` (a method on the guard
  itself) and `println(0)` each get *"a RefCell guard for 'c' is live across
  this call"*. `[CELL-7]`: the lint is for a call *"that could re-enter the same
  cell"*. Three warnings for a four-line program trains users to ignore it.
* **F-188** `DEFECT` S3 — **a view escaping its source gets two errors, the first
  wrong.** Returning `V(s=xs.as_span())` for a local `xs` reports `E3021 'xs'
  cannot be written while it is borrowed` (nothing writes `xs`) and then the
  correct `E3060 'xs' does not live long enough`. The drop at scope end is
  being reported as a "write". One diagnostic, shape B7 (`[DIA-7a]`).
* **F-189** `DEFECT` S3 — **`Array.iter()` does not exist** (`E1010 'Array[i32]'
  has no method named 'iter'`), though `[SPN-5]`, IX.1 and the `Iterable`
  interface make it the canonical way to iterate; `for x in xs:` works.

### Part F — Runtime (`runtime/ember_rt`, C11) review

* **F-190** `DEFECT` **S2 (memory safety)** — **growable-buffer arithmetic is
  unchecked.** `ember_vec_reserve` (`ember_rt.c:1159`) doubles `cap` with no
  overflow check and passes `cap * elem_size` to `ember_realloc` unchecked;
  `ember_vec_extend` computes `v->len + count` unchecked; `raw_alloc` rounds
  `(size + align - 1) & ~(align - 1)` unchecked. A wrapped product allocates
  a small buffer that later writes overrun — the same defect class the audit
  found in the Arena (SAFE-6). Use checked multiplication and panic "capacity
  overflow" (`[ALC-4]`).
* **F-191** `DEFECT` **S2** — **`Array[T]` storage is aligned to 16 bytes whatever
  `T` needs.** `#define EMBER_VEC_ALIGN (sizeof(void*) * 2)` with the comment
  *"The compiler passes an element size but not an alignment … over-aligning is
  always sound"* — but for `T` with alignment > 16 (SIMD types, `[SIMD-1]`'s
  `@align(width)` vectors, cache-line–aligned structs) it *under*-aligns, and
  aligned vector loads on misaligned data are UB / fault. Pass `align_of[T]`
  to the vec routines. **Latent today**: no type the compiler currently
  builds has alignment > 16, because `@align` is ignored (F-192) and SIMD
  types do not exist yet — it becomes live the day either lands.
* **F-192** `DEFECT` S2 — **`@align(N)` is silently ignored.** `@align(64) struct
  Wide` compiles, and the emitted C has no `_Alignas`; `size_of`/`align_of`
  are wrong for it and FFI/SIMD/GPU layout (`[TYP-11]`, `[FFI-5]`, IX.5)
  silently disagree with the declaration. Same shape as F-161 (an attribute
  accepted and not honoured).
* **F-193** `PERF` **S2** — **every reference-count operation is an out-of-line
  call, and a `Sync` retain is a CAS loop.** `ember_retain`/`ember_release`
  are `static inline` wrappers around `ember_obj_retain`/`ember_obj_release`,
  which live in `ember_rt.c` — a separate translation unit, so the host C
  compiler can inline nothing without LTO. Each retain also re-loads the
  flags word to check `DEINITIALISING` and, for `Sync` objects, loops on
  compare-exchange, where `[RT-8]` asks only for a relaxed increment. Swift and
  C++ `shared_ptr` inline the fast path. Proposal: put the fast path (one
  increment, overflow trap) in the header; keep the deinit/resurrection
  checks on the release-to-zero slow path, where `[OBJ-5]` needs them. `[BEN-6]`
  gates RC-heavy code at ≤ 1.15× an intrusive C++ count — measure this first.
  **Crude measurement (noisy — a `cargo test` run was using the machine):**
  a loop that returns one of two class handles from a function and reads a
  field, 50M iterations, one surviving retain + release per iteration in the
  emitted C: Ember release (clang `-O2`) ~108–211 ms vs. the same loop in C with
  an intrusive inline count ~45–63 ms — roughly **2×**, against a 1.15×
  gate. Needs a clean re-run under `[BEN-1]`'s protocol.
* **F-194** `PERF` S3 — **every allocation goes through `_aligned_malloc`
  (MSVC) / `aligned_alloc`, and `ember_realloc` always allocates-copies-frees.**
  No `mimalloc` (`[RT-1]`), no in-place `realloc`, and global allocation
  counters (`g_stats.live_bytes += size`) updated non-atomically on every
  allocation — a data race as soon as threads exist, and a shared cache line
  under load. Use plain `malloc`/`realloc` when `align <= alignof(max_align_t)`,
  and per-thread or atomic stats (or debug-only).
* **F-195** `DEFECT` S4 — **reference-count overflow reports the wrong reason.**
  At `strong == UINT32_MAX` `object_try_strong_retain` returns false and the
  caller panics *"retain of deinitialising or invalid object"*; `[RT-7]` makes
  overflow a distinct fatal error and the message should say so.

### Part G — Lexer/parser edges and Python habits

**Works:** tab indentation is `E0002`, `1f32`, `0x_ff`, `1_000_000`, a
non-ASCII identifier (`café`), triple-quoted strings, `;` as `E0105`.

* **F-196** `DEFECT` S3 — **redeclaring a name in the same block is accepted.**
  `x: i32 = 1` then `x: i32 = 2` in one block compiles and prints 2 (with an
  `L1001` warning on the first). `[GRM-4]`: *"redeclaring in the same block is
  `E1020`"*.
* **F-197** `GAP` S3 — **raw strings, byte strings and f-string format specs
  are not implemented** (`E1010 this literal is not supported yet`, `E1010 a
  format spec is not supported yet`) — all Part II (`[LEX-19]`), which Phase 1
  covers (F-159).
* **F-198** `DESIGN` **S2** — **the ten most common Python habits get no help at
  all.** Probed: `True`, `len(x)`, `xs.append(1)`, `range(3)` — each a bare
  `E1010 cannot find …` / `has no method named …` with **no** `help` line
  (and `print(a, b)` with two arguments is presumably the same). `[DIA-17]`
  lists five Python fix-its (none implemented, F-148–F-151); its list should
  grow to cover, at minimum: `True`/`False`/`None` → `true`/`false`/`None` as
  an `Option`, `len(x)` → `x.len()`, `xs.append(v)` → `xs.push(v)`,
  `range(n)` → `0..n`, `range(a, b)` → `a..b`, `str(x)` → `f"{x}"`,
  `int(s)` → `s.parse[i32]()`, `x ** 0.5` → `sqrt`, `elif`-less `else if` →
  `elif`, `def` → `fn`, `self.x` in a non-method → the `self` parameter,
  `print(a, b)` → `println(f"{a} {b}")`. Each is a table lookup at the point
  N1/N3 already fire; this is the cheapest "types like Python" win in the
  whole list. (Also consider simply accepting `True`/`False` as aliases.)

### Part H — Project process and ledgers

* **F-199** `PROCESS` **S2** — **the ledgers say "none open" while ~80 defects
  reproduce.** `docs/COLD-START.md` (dated 2026-09-15): *"125 defects recorded,
  **none open**"*; `docs/DEFECTS.md` has 151 rows, every one fixed or
  closed. None of the 39 audit defects (Part D) or the defects in this file
  is in the ledger. The project's own rule (memory: *a defect fix updates four
  documents*; COLD-START §2's sort) says a clear-rule/wrong-compiler finding
  belongs in `DEFECTS.md` as **open**. Proposal: before fixing anything, an
  intake pass — every `DEFECT` entry here and every audit reproducer becomes an
  open `D-nnn` row pointing at its reproducer, and every `ERRATUM` becomes an
  `ERR-nnn` in `spec-errata.md` — so the "none open" line can be true again.
* **F-200** `PROCESS` **S2** — **specification churn far outpaces the compiler.**
  `docs/spec-source/` holds **28 files, ~20 MB** — full snapshots of 0.6
  through 0.9.8_Hardened_3 — plus ~40 hardening cycles recorded in Part 0 and
  Appendix H, against **1 of 9 phases complete** (COLD-START §4). The last
  three revisions (0.9.8 H1–H3) were a `Shared` API, a CLI root rule and N1
  suggestion ordering; in the same period `println` of an f-string does not
  compile (F-012). Proposal for the owner: a **spec freeze on everything but
  errata** until the "first programs" milestone (F-115) and the SAFE holes
  (Part D) are closed; keep frozen snapshots in git history (tags) rather
  than as 20 MB of files in the tree.
* **F-201** `DOCS` S3 — **`docs/HANDOFF.md` is 10,698 lines.** A handoff that
  long is an archive. COLD-START already does the handoff job; move HANDOFF's
  historical sections to an `docs/history/` file per phase and keep HANDOFF
  to the current state (the RageV-style "current state / done / not done /
  invariants / traps" that XXI.1 rule 4 asks for).
* **F-202** `DOCS` S4 — **`examples/` has only `hello.em`**; XXI.5 lists
  `particles`, `ecs_demo`, `vulkan_triangle`. A handful of small, compiling,
  CI-run examples (FizzBuzz, word count with `Map`, a class tree, a particle
  loop) would double as the Python-user tour and as regression tests.

### Part I — Generics and recursive data (compiler sweep, continued)

**Works:** `Array[Box[dyn Shape]]` with an interface default method; a
`fn(i32) -> i32` parameter receiving a capturing closure.

* **F-203** `DEFECT` **S2** — **a generic struct's methods cannot name their own
  type.** Inside `struct Pair[T]:`, `fn swap(self) -> Pair[T]` → `E1010 cannot
  find type 'Pair' in this scope`; `-> Self` → `E2020 'Self' names the type
  being declared, and there is none here` and `cannot find 'Self'`. So a
  generic container cannot have a method that returns or builds another
  instance of itself (`map`, `clone`, `split`, `swap` …).
* **F-204** `DEFECT` **S2** — **the textbook recursive enum cannot be traversed.**
  ```
  enum Tree:
      Leaf(v: i32)
      Node(l: Box[Tree], r: Box[Tree])
  fn sum(t: Tree) -> i32:
      return match t:
          Leaf(v) => v
          Node(l, r) => sum(l) + sum(r)    # E2020 expected 'Tree', found 'Box_Tree'
  ```
  IX.1 says `Box` derefs automatically and IV.4 says a reference is read
  through wherever a `T` is wanted; neither happens for a `Box` in argument
  position (and the message leaks the mangled `Box_Tree`). `sum(l.get())`
  instead gives `E3013 cannot move out of a borrowed parameter` — the pattern
  binding is treated as a move although `[GRM-13]` makes it a `ref` binding.
  Expression trees, ASTs, JSON values and scene hierarchies all have this
  shape.
* **F-205** `DEFECT` S3 — **calling a parameter whose type is a generic bounded
  by `Callable` fails**: `fn apply_twice[F: Callable[(i32), i32]](f: F, x: i32):
  return f(f(x))` → `E1010 cannot find 'f' in this scope` (audit FE-1's shape
  through an explicit bound). The `f: fn(i32) -> i32` spelling works.

### Part J — Tooling surface

**Works:** `--json` diagnostics (structured, with spans, helps, and the
rendered text); `ember fmt` is idempotent on the files tried; `ember explain
<code>` resolves a code to its rule.

* **F-206** `GAP` S2 — **32 error pages for 210 registered codes.** `[DIA-6]`:
  *"`docs/errors/EXXXX.md` exists for every code with an example and its
  fix"*; `[DOC-1]` makes pages a per-phase exit criterion. `docs/errors/` has
  32 files; `compiler/ember_diag/src/codes.rs` registers 210 codes. `ember
  explain E3040` — **use of moved value**, the most common ownership error —
  prints *"No extended description has been written for this code yet."* For
  a Python-feel language the error page for the first ownership error is
  the tutorial. Proposal: pages for the top 20 codes a newcomer hits (E1010,
  E2020, E2060, E3040, E3021, E3020, E3023, E3060, E3050, E3041, E2040, E0102,
  E2035, E0105, E0002, E2010, E3013, E3062, E1020, E2090) before any new code
  is added.

* **F-207** `TOOLING` **S2** — **the six "green" gates are green because their
  baselines absorb almost everything.** Run on the clean tree: all pass. Their
  baselines: `rule_index_baseline.json` — **752 rules without tests** (of ~873
  in the index, so roughly 14% of rules have a test directory), **188 codes
  without an error page**, **149 codes without a test**, 5 codes not in the
  registry, 4 dangling references; `spec_check_baseline.json` — 25 failing
  spec blocks (for the *old* spec; F-085); `error_pages_baseline.json` — 59.
  "No new problems" is accurate and says nothing about the size of the old
  ones. Proposal: print the baseline sizes in the gate output and in
  COLD-START's status block, and set a shrink target per phase so the number
  is watched the way `[TST-4c]` intends ("a work list with a fixed ceiling").

* **F-214** `TOOLING` S3 — **the test suite is green but has a flaky pair, and
  `cargo test` hides everything after the first failure.** Clean tree at
  `761ce7c`, `cargo test --workspace --no-fail-fast`: **263 tests, all pass**
  (13 analysis, 3 branding, 10 build, 4 codegen, 16 diag, 35 milestones/
  conformance in ~7 min, 9 UI, 5+41 lexer, 21 MIR, 69 parser, 12 span, 8
  typeck, 17 types). COLD-START still says 210. On the first full run, two UI
  tests (`committed_basic_name_suggestion_matches_and_fixed_source_compiles`,
  `committed_struct_field_name_suggestion_…`) failed under full parallel load
  and pass alone and on three reruns — a shared-output or timing race in
  `compiler/ember_driver/tests/ui.rs`. Without `--no-fail-fast` that failure
  also stopped every later test binary from running. Proposal: give each UI
  case its own temp output dir, and run CI with `--no-fail-fast`.

### Part K — More Python-shaped gaps

* **F-208** `DEFECT` S2 — **range slicing `xs[a..b]` is not implemented**, and
  its failure cascades: `s = xs[0..1]` → `E1010 this expression is not
  supported yet in this phase`, then `'i32' has no method named 'len'` on the
  next line (the failed expression was typed as the element type — a cascade
  `[DIA-14]` forbids). VI.3 specifies slicing (`a[i..j]`, `a[..j]`, `a[i..]`).
  Python's `xs[0:1]` gives two raw parse errors; add it to F-198's list
  (`xs[a:b]` → `xs[a..b]`).
* **F-209** `DESIGN` S3 — **comprehensions.** The 0.8.2c change log defers list
  comprehensions ("the review defers them itself, correctly"). They are among
  the most-used Python constructs, and once iterator adapters exist
  (`map`/`filter`/`collect` — themselves missing, F-182) they are pure
  syntax: `[f(x) for x in xs if p(x)]` ≡ `xs.iter().filter(fn(x) => p(x)).map(fn(x)
  => f(x)).collect[Array[_]]()`, with no new semantics and the same cost.
  Proposal: schedule list/map/set comprehensions for right after the
  adapters, as sugar only.
* **F-210** `DESIGN` S4 — **interfaces are nominal only.** Python is duck-typed;
  Ember requires `implements`/`extend … implements` (`[TYP-17]`, "There is no
  duck typing"). Go-style implicit satisfaction is the usual middle ground,
  but it interacts with coherence (F-040) and with `dyn` tables. Recorded as
  an idea to weigh, not a recommendation.
* **F-211** `DEFECT` S3 — **`enumerate` over an `Array` cannot be written**
  (`Array.iter()` is missing, F-189), so `for i, x in …enumerate():` — the
  Python `for i, x in enumerate(xs)` — has no working form today.

* **F-212** `DOCS` S4 — **two errata are marked open that the grammar already
  fixed.** `docs/spec-errata.md` ERR-036 (*"Part III admits no item-level
  `extern "C" fn` declaration"*) and ERR-037 (*"no `extern class`
  declaration"*) are **Status: open**, but 0.8.4_Hardened_1 amendments A8/A9
  added exactly those productions (`fn_header := ["extern" string_lit] …`,
  `extern_class := …`, H3 lines 1342 and 1388). Close them with a pointer to
  A8/A9. (Checked all errata/owner-queue entries still marked open —
  ERR-020, ERR-034, ERR-042, ERR-043 and ODR-003 — none duplicates a finding
  here except ERR-020, cross-referenced in F-167.)

* **F-213** `GAP` S4 — **`i32.MIN % -1`.** Probed: panics `integer overflow in '/'`
  in both profiles (the message names `/` for a `%`). `[TYP-8]` specifies only
  `i32.MIN / -1`. The mathematical result of `MIN % -1` is 0 and fits (Python
  gives 0); it panics only because x86 `idiv` traps. Say which one Ember
  means, and fix the operator named in the message. (Division by zero and
  `MIN / -1` behave as specified in every profile — probed.)

