# Gap map 04: functions, effects and checks (CLO, EFF, EXC, CLS, DSP, IFC, ERR, MOD-9, BLD-14)

Agent 04, 2026-10-27. Read-only: nothing was built or run. Every claim about the compiler cites the
file and line read at `ab2b7b76` (branch `pause/redesign-run4-step2`). Scratch scripts and dumps are in
`target/gap-0.9.15/04/` (`extract.py` pulls a rule's text from both spec trees; `map_count.py` and
`design_count.py` count the design tests; `*.txt` are the old tests dumped for reading).

**How the old tests were classified.** All 204 test programs in the 47 `tests/conformance/` folders of
my H.41 rules were read by hand (dumped with comments and expectations, `target/gap-0.9.15/04/{exc_a,
exc_b,clo_a,clo_b,cls,eff,err}.txt`); a regex script only cross-checked that no `keep` holds an old
spelling outside comments. The design-test counts come from `conformance-rule-map.md` by script and
agree with the files' own `#$ rules:` lines.

**In one paragraph.** Today's compiler has the *effect-set* half of this area (kinds such as `Alloc`,
`Io`, computed over the call graph with chains, `ember_analysis/src/effects.rs`) and a narrow
parameter-field access summary for borrowing (`CallableAccessSummary`, `ember_mir/src/lib.rs:548`).
It has **no** place-level summary of what a function changes, reads and ends, so every rule that the
new spec decides "from the summary" (E3020/E3021/E3022/E3060 through calls, E3031, E2100 in `init`,
`@cached`, what a function value or `dyn`/virtual method does) is not built. Class-object exclusivity
is entirely a run-time mechanism today (access words, `BeginAccess`/`EndAccess`, hoisting, L3013): all
of it is removed by 0.9.15 and must be replaced by compile-time rules *before* it is deleted. Captures
are inferred per variable with an `owned fn` prefix and a copy heuristic (CLO-13); 0.9.15 replaces that
with capture words and the real variable. A callable parameter is an implicit generic today; in 0.9.15
it is a two-word function value, and the type-parameter form is the direct call. Helper AJ's function
items (`stopped/AJ-function-items`) are the right base for finding a function value's targets (§3,
SD2).

---

## 1. Rule table

Status words as the brief defines them. "Design" counts the 841 design tests whose spec IDs name the
rule (`conformance-rule-map.md`), with their `R` folders; a rule no design test names by ID is still
exercised by the folders named in its row (by design rule). Old-test counts are
`keep/respell/rewrite/retire`; every non-keep file is named in §1b.

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| BLD-14 | added | interface file records the 7 body facts (changes/reads/ends with chains, only-adds, scoped tasks' effects, parent link set first, view params used after an end, locks, stored function values and C callbacks); a change is an interface change | `ember_build/src/interface.rs:50` schema 14 records signatures, region metadata, effect sets (`:203`), inline bodies, layouts; `ember_driver/src/lib.rs:4056-4060` fills effect sets | partly | schema 15 with the summary (A4-3); kept bodies reinstall it | none | 12 (R8 1, R14 3, R17 8) |
| CLO-1 | changed | a lambda or local function has its own type; kept when given to a type parameter (direct call); a function value wherever a callable type is written | capturing lambda: own env struct (`ember_typeck/src/lib.rs:48432-48470`); capture-free lambda and named function: the shape `fn(A)->R` (`:48297-48330`); AJ gives them items (`TyKind::FnItem`, branch only) | partly | merge AJ (area 9, A9-2); conversion to a function value only where a callable type is written (A4-10) | 4/1/0/0 | 68 (R3 4, R4 2, R5 1, R8 3, R11 4, R12 23, R14 1, R15 28, R17 2) |
| CLO-2 | changed | captures written after the brackets (`copy of`, `owns`, `shares`, `shares mut`, `thread_shared`, `weakref`, `ref`); no word = the real variable; E0120, E0116, E0100 (`with`), E1010 `owned fn`, E0109 `owns fn` | per-variable inference, two probes (`lib.rs:48241-48420`); `owned fn` prefix (`ember_ast/src/lib.rs:837`, `ember_parser/src/body.rs:1650`); real variable as `ref`/`ref mut` env field = today's non-owned capture | partly | capture list in AST/parser (area 7), capture words in typeck, drop `owned fn` (A4-9) | 4/5/5/0 | 2 (R12 2) |
| CLO-3 | changed | a callable type is one type everywhere: a function value; a parameter of callable type receives one and calls through it; direct call only through a type parameter; `type` alias; `extern "C" fn` conversion panics with state | `f: fn(A)->R` parameter is an implicit generic (`lib.rs:1773`, `:1833-1862`, `:5154`); elsewhere an owned callable value (`ember_codegen_c/src/lib.rs:3065`); `extern "C"` conversion built (`CastKind::FnToC`, `ember_mir/src/lib.rs:1199`) | partly | parameter form becomes a function value (A4-10); type-param form stays direct | 2/6/3/0 | 10 (R3 3, R11 2, R12 4, R13 1) |
| CLO-4 | changed | real variables checked in the function that writes the lambda: the value provably ends before them (locals, local lists/objects, callees that keep it no longer by summary), else E3026 naming a word; returned counter needs `owns n` | E3026 for a borrowing lambda stored outside a parameter (`ember_analysis/src/stores.rs:461`); adapter rule (`borrows.rs:3480`) | partly | "provably ends before" from the keeps facts of the summary (A4-2, A4-9); messages | 1/3/0/0 | 9 (R12 6, R15 3) |
| CLO-5 | changed | `shares`/`thread_shared` capture is one more owner; owner loops through function state refused (WK-1), back link `weakref` | a captured class handle is retained (counted) | partly | capture words (A4-9); owner loop check is area 3's E3032 | 0/1/0/0 | 0 (R12 by design rule) |
| CLO-6 | changed | a function that gives a capture away is `once fn`; given where it may run twice E3013; a `once fn` parameter must be `owns f`; second call E3040 | `once fn` built (`lib.rs:39807-39870`, `:5190`); E3030 (`:35644`, `:39868`) | partly | E3030 folded into E3013; plain `f: once fn` is E3013 (A4-10) | 0/11/0/0 | 7 (R1 1, R11 2, R12 3, R15 1) |
| CLO-6a | changed | an owned `once fn` value in an `owns` block or a collection is callable; the call empties its place | built for `Box`, field, `pop_front` (`lib.rs:14719-14721`; codegen C 4 sites) | partly | `owns` block replaces `Box` (area 1, A1-6); codes fold (E3012, E3041) | 0/3/0/0 | 0 |
| CLO-7 | changed | std functions that keep or thread-run a function take it `owns` (`thread.spawn`, `jobs.submit`: `owns f: thread_shared fn()`); nothing captured by a rule of the call; E3026, E7004 for no word | `std/src/thread.em:25` `spawn[R](owned f: fn() -> R)`, `std/src/jobs.em:23`; at-call move capture (`lib.rs:52333-52820`) | partly | std signatures (area 8, A5-12); remove the at-call move rule (A4-9) | 0/1/1/0 | 0 (R15 by design rule) |
| CLO-10 | changed | two words `{code, state}`; state ≤ 1 word inline else one block (`Alloc`); stateless is `Copy`; `=` hands over; call copies two words out; replacing/ending ends the state's owns (drop counted); `thread_shared fn()`; `==` | `struct ember_fn_value`: entry, descriptor, three state words, 40 bytes (`ember_codegen_c/src/lib.rs:3065-3080`); larger state boxed; `+ Send` bit (`ember_types/src/lib.rs:223`) | partly | new representation in both backends, interp, runtime (A4-10) | 0/7/5/0 | 0 (R12 by design rule) |
| CLO-12 | changed | a local function takes the same capture words, passed by name | local functions capture like lambdas (`lib.rs:39895-40050`) | partly | capture words (A4-9) | 0/2/0/0 | 0 |
| CLO-13 | removed | (copy of read-only `Copy` captures where a value is wanted) | `lib.rs:48408-48420` (`by_copy`), assigned-after scan `:29694` | remove | delete; the scan survives only if AJ needs it (area 9 J4) | 0/0/0/2 | 0 |
| CLO-14 | changed | `[F: fn(int) -> int]` above the function; `F` is the exact function, direct call, one instance per function; the call makes what the function changes | explicit callable bound built (`lib.rs:5017`, `:5051`); one instance per capture-free lambda only with AJ | partly | `[F]` above (area 7); summary per instance (A4-2) | 0/6/0/0 | 0 |
| CLO-15 | removed | (lambda at a consumed callable parameter captures by move) | `borrows.rs:3480`; spawn move capture (`lib.rs:52333`) | remove | delete with A4-9 | 0/0/0/2 | 0 |
| CLO-16 | added | `thread_shared fn()`: a function any thread may run; holds only `thread_shared` values or copies (E7015); a `thread_shared` object's fields hold such types | `fn(A)->R + Send` (ODR-264, `TyKind::Fn.send`, `ember_types/src/lib.rs:223`), E7004/E7005 | not built | built by area 5 (A5-10); area 4 keys target sets by it (A4-1) | none | 10 (R12 4, R15 6) |
| CLO-17 | added | a call through a function value makes what every function the program can put there changes; storing changes nothing; a function value counts as an argument changing what its functions change and capture; not stored in a place a real variable it changes owns (E3022) | indirect call: every effect, or D-877's shape guess (`effects.rs:1100-1146`, `:904-936`); no change summary | not built | targets (A4-1), summary (A4-2), call-site use (A4-4) | none | 0 (R12 103, R5 by design rule) |
| CLO-18 | added | a call through a stored value whose functions may change their own state changes its holder (E3023 through a read-only name); such a value to a read-only parameter E3023; one count across a call that may end its own state | every owned callable value called from a mutable place (`lib.rs:44164-44200`, `:47723`) | partly | condition from the summary; count pair (A4-10, A4-11 checks) | none | 0 (R12 by design rule) |
| CLO-19 | added | a lambda parameter says what it receives, matching the type (`fn(owns s)` where the type hands over; E2020 otherwise) | parameter mode taken from the bound silently | not built | A4-9 | none | 0 (R12 1 by content: `reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over`) |
| CLS-1 | changed | object in its own block; `using`; one owner unless `shares`; `=` hands over; `owns` on a class type E2114 | a counted `Copy` handle, strong count 1, retained on copy (`ember_types/src/lib.rs:2122`, `:2658`, as area 1's report cites) | partly | area 1 (A1-9 one owner, A1-14 `using`); E2114 with A1-6 | none | 0 |
| CLS-2 | changed | `init` sets fields through `this`; `this` used as a whole by passing, storing, returning, capturing, link/owner made, or a method whose body lets `this` out; a method may be called once the fields its body reads are set (E2100 messages); E2108 `owns this` on `init` | AST-level check: any method call on `self` is a whole use (`lib.rs:24131-24310`); closure capture (`:48292-48294`) | partly | MIR-level check with summaries (A4-12) | 0/6/0/0 | 3 (R5 3) |
| CLS-4 | changed | `virtual` in a final class E2112; a call through `virtual` counts every override's changes | E2110/E2111 built; W2111 warning (`lib.rs:10987`, `codes.rs:439`); effects of overrides built (`effects.rs:1107-1121`) | partly | E2112 (A4-16); summary over overrides (A4-2) | 0/2/1/0 | 1 (R5 1) |
| CLS-5 | changed | interface calls on a class object static unless the object is a `dyn` value | built (`ember_codegen_c/src/lib.rs:140-149`, test CLS-5) | text only | none (spelling of the test) | 0/1/0/0 | 0 |
| CLS-6 | changed | derived `drop`, base `drop`, fields reverse, block freed once no `weakref` remains; object ended before its `drop` | order built (codegen C 3 sites, `ember_interp/src/object.rs`); weak-count free built | partly | "ended before drop" is area 3 (WK-3) | none | 0 |
| CLS-7 | changed | `this`; what a method does to it worked out from its body; may-change method not called through a read-only name (E3023); `this` never the only name; E2103; `mut this` E0113, `mut self` E1010 | `self` handle; `mut self` holds a whole-object write (`ember_mir/src/lower.rs:296-345`); E2103 (`lib.rs:25845-25849`) | partly | `this` (area 1 A1-7, area 7 parse); summary-based method effect (A4-7) | 0/5/4/0 | 1 (R5 1) |
| CLS-7a | changed | in `drop`, `this` not stored and no owner made (`shares this`); `fn drop():` changes its object as a given | E3016 `ember_analysis/src/drops.rs:477`, `:686` | partly | owner-made case (needs area 3's `shares`) (A4-7) | none | 0 |
| CLS-8 | removed | (`Sync` class fields immutable) | `@sync` class checks (`lib.rs:44425`) | remove | with `@sync` (area 5, A5-9) | 0/0/0/1 | 0 |
| CLS-9 | changed | a `let` field is set only in `init` (E2104) | built (`lib.rs:44440-44460`) | text only | message wording | 0/1/1/0 | 1 (R7 1) |
| CLS-9a | changed | after `init` nothing changes through a `let` field, for every type (deep): no push, no re-point, no change through it (E2104) | opposite: only the binding is fixed (`lib.rs:44441-44444`) | not built | A4-16 | none (rows in CLS-9) | 0 |
| CLS-11 | changed | two-phase construction (wording: set, `this`) | built (`lib.rs:24263-24280`, E2102) | text only | `this` spelling | 0/4/0/0 | 0 |
| CLS-12 | added | `@cached`: result kept, cleared by any change of a field the body reads; `ref T` view for non-`Copy`; once-flag across threads; E2117 cases | nothing | not built | A4-13 | none | 4 (R5 4) |
| DSP-1 | changed | static dispatch when the object's type is final or the method not virtual | built (`ember_mir/src/lower.rs:2570`, `:7330`) | text only | none | 0/1/0/0 | 0 |
| DSP-3 | changed | a `dyn I` call finds the table through the type index, cached per value | through the header's `ti` pointer, ≤ 4 implementers tried first (`ember_codegen_c/src/lib.rs:140-149`, `:879`) | partly | type index (area 3 header); `dyn` values (A4-14) | 0/1/0/0 | 0 |
| DSP-4 | changed | `if s is Circle as c:` with words, `is` without a name, `elif`, E2036, E2037, E2106; one type-id compare | `cast x if D` gives `Option` for classes only (`lib.rs:37614-37700`; parser `body.rs:1067-1107`); runtime walks the base chain (`runtime/ember_rt/src/ember_rt.c:3408-3423`) | not built | A4-14 (with A1-20, area 7 GRM-45) | 0/0/0/0 (no folder) | 2 (R14 2) |
| DSP-5 | changed | devirtualise a call with one implementation; what a virtual/interface call changes is worked out from every override | devirtualisation built (`ember_codegen_c/src/lib.rs:49`, `:869`); effects of overrides built, changes not | partly | summary (A4-2) | 0/1/0/0 | 0 |
| DSP-6 | added | `cast s as Circle` (stops in every profile with the given text); a cast of a place is a place (`ref mut cast …`); E3027 through a plain one; one compare | `cast x as D` for classes, panics (`lib.rs:37614-37700`, `AssertKind::Downcast`) | partly | `dyn` targets, place casts, message, compare (A4-14) | none | 0 (R14 by design rule) |
| EFF-1 | changed | effect set and summary over the call graph; drops count as calls of `drop` | effect sets built (`effects.rs:80-160`, `:841-870`); drops counted (`:1202-1300`); generators built | partly | summary half (A4-2) | 1/2/1/0 | 0 |
| EFF-2 | changed | function value: every function stored into that type in the whole program; type parameter: the one given; interface/`dyn`/virtual: every implementation; `extern "C" fn`: declared effects; contract on a function type checked against each stored function; E4010 | virtual built (`effects.rs:1107-1121`); `dyn`: every effect (`:1123-1127`); stateless fn values by (arity, result) (`:1130-1137`); stateful: every effect (`:1138-1142`); E4010 never emitted (no user of `codes::E4010`) | partly | exact targets (A4-1); contracts on fn types (Q1); E4010 (A4-11) | 2/0/0/0 | 0 |
| EFF-3 | changed | `extern` effects as declared, `FFI` at least; callbacks C keeps are EFF-26's | built (`effects.rs:943-952`); callbacks counted with every effect (`:955-963`) | partly | kept-callback changes (A4-2, with area 6) | 3/0/0/0 | 0 |
| EFF-4 | changed | effects and summaries are interface; written words only across a package edge; summary, chains, scoped tasks, stored function values and kept callbacks recorded; a change re-checks callers | effect sets in `.emif`, re-check built (milestone test `a_callee_gaining_an_effect_rechecks_its_callers_contracts`, `ember_driver/tests/milestones.rs:6169`) | partly | A4-3 | none | 0 (R17 by design rule) |
| EFF-6a | removed | (`@static_safe` parameter forcing a dynamic check) | `lib.rs:19818-19840` | remove | delete with `RefCell` (A4-11) | 0/0/0/1 (in EFF-12) | 0 |
| EFF-9 | changed | `Aliasing` only `assert_disjoint`/`check_disjoint`; `Arithmetic` adds the count-limit compare; `Stale` adds the `weakref` header test | `BeginAccess` → `CHECK_ALIASING` (`effects.rs:980-983`); `RefCellBorrow` → aliasing (`:1076`) | partly | remap kinds (A4-11; checks from area 3) | none | 0 |
| EFF-11 | changed | reasons table: `requested_by_type` for a `weakref` link; `establishes_static_fact` for `assert_disjoint` | side table in codegen C (`ember_codegen_c/src/lib.rs:700-730`), "dynamic exclusivity" rows | partly | A4-11 | none | 0 |
| EFF-12 | changed | `@no_alias_checks` forbids only `assert_disjoint`'s compare (E4030); `@static_safe` E0104 with the rename | `@static_safe` (`ember_analysis/src/static_safe.rs:28`, 84 lines) forbids exclusivity and `RefCell` checks | partly | rename, narrow (A4-11) | 0/1/0/1 | 1 (R16 1) |
| EFF-13 | removed | (class handle accesses are checked dynamically) | `static_safe.rs:40-60` | remove | delete (A4-8) | 0/0/0/2 | 0 |
| EFF-14 | changed | the usual set says `@no_alias_checks` | follows EFF-12 | text only | none beyond EFF-12 | none | 0 |
| EFF-15 | changed | the proof list is OPT-2, RNG-4, SIMD-7 and Part VIII's parent-link proof | verdict after the proofs (`effects.rs:6-8`), with EXC-3/EXC-8 proofs feeding it (driver `lib.rs:5410-5442`) | partly | drop EXC proofs (A4-8), add the parent-link proof (area 3) | 1/1/0/0 | 0 |
| EFF-16 | changed | count-limit compare is a `RuntimeCheck` allowed under `@nopanic(explicit)`; messages say so | built for bounds/arithmetic/aliasing/stale | partly | count compare kind, message (A4-11) | 1/1/0/0 | 0 |
| EFF-18 | changed | `with x.lock:` on a `Lock` has `Sync + Lock + Block` | `Mutex.lock` effects built (`effects.rs:1393-…`) | partly | `Lock` is area 5 (A5-16); its builtin effects here (A4-11) | none | 0 |
| EFF-24 | added | every function's summary: changes/reads/ends with chains, through callees, function values, `dyn`/virtual, threads/tasks/jobs, kept callbacks, drops; elements touched; only-adds; parent link set first; view params used after an end; other names while a `ref mut` is used; locks; no clause on a first line (`changes` E0100) | parameter-field access summary only (`CallableAccessSummary`, `ember_mir/src/lib.rs:548-556`; `borrows.rs:2763-2832`); no `changes` clause exists (never in 0.9.10) | not built | A4-1, A4-2; E0100 text (area 7) | none | 0 (R5 177, R12 196, R17 22 by design rule) |
| EFF-25 | added | the summary decides E3020, E3021, E3031, E3060, E3022 (stored function value), E7018, task effects, E2100, `@cached`; nothing checked at run time; messages name the chain | none of these consult a callee's body | not built | A4-4, A4-5, A4-12, A4-13 (E7018 area 5) | none | 0 (E3020 through calls: 35 tests; E3031: 10) |
| EFF-26 | added | a call that runs a function makes its changes (spawn, scope.spawn, jobs, `for_each`, driving an adapter, `callback(retained|once)`); storing does not; a foreign call may run every kept callback unless `@ffi(no_callbacks)` | `@parallel` closures counted at the call (`effects.rs:1053-1061`); FFI callbacks: every effect (`:955-963`) | not built | A4-2 (contract words from area 6) | none | 0 (R12, R15 by design rule) |
| EFF-27 | added | `@cached` depends on the fields its body reads; clears at each line that may change one; a `sharedref mut`/`shares mut` link into what it reads E2117 | nothing | not built | A4-13 | none | 0 (CLS-12's 4) |
| ERR-2 | changed | wording ("handed over with no call") | built (`ember_typeck/src/errors.rs`) | text only | none | 1/3/0/0 | 0 |
| ERR-3 | changed | `fn source() -> Option[ref dyn Error]` | built | text only | std spelling (area 8) | 1/2/0/0 | 0 |
| ERR-4 | changed | helpers take the function as a type parameter, `owns this`, `owns T` payload; lambdas say `owns` | helpers built in std (`std/src/core.em:2085-2140`), routed by `lib.rs:27828-27860` | partly | CLO-19 lambda words (A4-9); std spelling (area 8) | 3/8/0/0 | 0 |
| ERR-8 | changed | `AnyError` holds `owns dyn Error` | built over `Box[dyn Error]` (`std/src/core.em:92`) | text only | std spelling with A1-6 | 2/3/0/0 | 0 |
| ERR-9 | changed | example uses `StrView` | built | text only | none | 0/1/0/0 | 0 |
| ERR-13 | changed | example text "is out of bounds for a length of" | runtime already says it (`runtime/ember_rt/src/ember_rt.c:9135`) | text only | none | none | 0 |
| EXC-1 | changed | a change of a field while a read-only view into it (through another name that may reach the object) is used: E3021; a reshape ends the view (E3060); `ref mut` name then E3022; when compiling | run-time panic via access words (`ember_mir/src/lib.rs:1274-1310`, `ember_rt.h:1953-1963`, `:2674-2682`) | not built | A4-6 | 0/1/3/0 | 0 (R8 by design rule) |
| EXC-2 | changed | a read through another name while a `ref mut` name of the field is used: E3021 | run-time panic | not built | A4-6 | 0/1/1/0 | 0 |
| EXC-3 | changed | which names may reach one object: same class family and one goes through a link or kept view, unless known different (classes, fresh, `if a is not b:`); refused with that line | proof that removes run-time checks (`ember_analysis/src/access.rs`, 534 lines) and E3080 for one local (`borrows.rs:4699`) | not built | A4-6 | 0/0/4/1 | 0 |
| EXC-3a | removed | (removed checks recorded) | `access.rs:24-78`, inspect side table | remove | A4-8 | none | 0 |
| EXC-4 | removed | (`let` field same access rules) | access words cover `let` fields | remove | A4-8 | none | 0 |
| EXC-5 | removed | (reborrows inside `mut self`) | `StmtKind::LendAccess` (`ember_mir/src/lib.rs:1300-1310`), `lower.rs:737-831`, `:2573` | remove | A4-8 | 0/0/0/4 | 0 |
| EXC-6 | removed | (panic names both accesses; debug access stack) | `ember_interp/src/object.rs`, runtime debug stack | remove | A4-8 | 0/0/0/4 | 0 |
| EXC-7 | removed | (L3013) | `ember_analysis/src/long_access_lint.rs` (227 lines), `codes.rs:457` | remove | A4-8 | 0/0/0/3 | 0 |
| EXC-8 | removed | (one check per loop) | `ember_analysis/src/loop_access.rs` (791), `check_hoisting.rs` (352) | remove | A4-8 | 0/0/0/6 | 0 |
| EXC-9 | removed | (hoisting conditions) | `loop_access.rs` | remove | A4-8 | none | 0 |
| EXC-10 | removed | (outer loop reuses access) | `loop_access.rs:52-120` | remove | A4-8 | 0/0/0/3 | 0 |
| EXC-11 | removed | (hoisted access invisible) | `ember_mir/src/verify.rs` | remove | A4-8 | none | 0 |
| EXC-12 | removed | (`STATIC`/`DYNAMIC_*` classes in inspect) | `ember_codegen_c/src/lib.rs:706` | remove | A4-8 | none | 0 |
| EXC-13 | removed | (perf suite for hoisting) | milestone tests in `milestones.rs` | remove | A4-8 | none | 0 |
| EXC-14 | removed | (`exclusivity = "unchecked"` removed) | referenced in `loop_access.rs`, driver | remove | A4-8 | none | 0 |
| EXC-15 | changed | method and its object: effect worked out from the body; may-change method not through a read-only name (E3023); `this` never noalias; E3031 for a call whose object the method may end before its last use of `this` | `mut self` whole-object write (`lower.rs:345`, `:831`); no restrict on receivers (built) | partly | A4-7, A4-5; remove whole-object write (A4-8) | 0/1/2/1 | 0 (R5 by design rule) |
| EXC-16 | changed | giving a memory-owning field a new value is a reshape: E3020 in a loop over it, E3060 for a view of the old value | write access at run time (`borrows.rs:1474`) | partly | A4-6 (with area 2's reshape) | none | 0 |
| EXC-17 | changed | writing a number or `bool` field is not a reshape; a write through another name while a read-only view of that field is used: E3021; `sharedref mut` gives no alias fact | `Copy` fields have no access word (built, `ember_types/src/lib.rs:1426-1430`) | partly | A4-6 | none | 0 |
| EXC-18 | removed | (dynamic access carried with a returned view) | `borrows.rs:431-560`, `:1081`; codegen C 3 sites | remove | A4-8 | 0/0/0/16 | 0 |
| EXC-19 | changed | fields are separate places; a method call touches the fields its body touches | per-field access words (`ember_types/src/lib.rs:2429`, codegen C `:4939`, `:5522`, `:8165-8330`, `:14491`) | partly | compile-time form (A4-6) using summaries; words removed (A4-8) | 0/2/4/2 | 0 |
| IFC-5 | added | `owns dyn I`, `shares dyn I`, `shares mut dyn I`, `dyn I` parameter view; bare interface as a type E2173; generic method with a `ref`/`ref mut` type parameter gets one `dyn` entry, one that returns/stores it E2050; kind tests; `dyn` methods' changes from every value made into it | `Box[dyn I]` (`TyKind::Dyn`, `ember_types/src/lib.rs:228`), interface-typed class handles (`ClassInterface`, `:183`); any generic method makes it not `dyn`-usable (`lib.rs:13021-13023`) | not built | A4-14 (owns: A1-6; shares: area 3) | none | 3 (R3 1, R14 2) |
| IFC-6 | added | non-`pub` generics may leave bounds unwritten (worked out from the body; E2072); `pub` ones write them (E2040, one message listing every bound) | bounds must be written; body checked against them (E2072 sites `lib.rs:49915-50522`) | not built | A4-15 | none | 2 (R14 1, R17 1) |
| MOD-9 | added | a package's edge: written words count; every other fact is in the interface file; a change is an interface change | as BLD-14 | partly | A4-3, A4-15 | none | 0 (R17 by design rule) |

**Rules the compiler implements under a removed name, in this area** (no extra rows; each is the
`remove`/`partly` half of a row above): `owned fn` (CLO-2), the CLO-13 copy heuristic, the CLO-15 move
capture, `mut self`'s whole-object write (EXC-15), `@static_safe` (EFF-12), `fn(A)->R + Send` and E7005
(CLO-16, ODR-264), `cast x if D` / `as?` (DSP-4), E3030 (CLO-6), E3080 (EXC-3), L3013 (EXC-7).

**Status counts:** built 0, partly 39, not built 15, remove 18, text only 10 (82 rows).

### 1b. Old tests: every non-keep, by folder

Totals over the 204 programs in my H.41 folders: **keep 26, respell 95, rewrite 35, retire 48.**
Common respellings (named per file only where something else changes): `self` → `this` and the
receiver dropped from the parameter list; `mut self` dropped; `owned x` → `owns x`; `mut x` parameter →
`ref mut x` and `ref mut` at the call; `owned fn` → the capture word after the brackets (`fn() owns
values`, `fn() copy of f`); generic brackets on a line above (`[T]`); `str` → `StrView`; lambda
parameter `x: ref int` → `ref x: int` or `x` (E0117 otherwise); `Box[dyn I]`/`Box(v)` → `owns dyn I`;
`+ Send` → `thread_shared`; E3030 → E3013, E3012 → E3013, E3041 → E3040 (folded codes); type names in
messages `int` not `i64` (R18); class aliasing `c = b` → `b = shares mut Bag()` / `c = shares mut b`.

**CLO-1** (respell 1): `accept_a_lambda_giving_a_borrowed_parameter_gives_its_reference` (`w: ref String`
→ word on the name).

**CLO-2** (respell 5, rewrite 5):
- respell: `accept_a_capture_shadows_an_item_of_its_name` (`[T: Display]` above); `accept_a_closure_in_a_method_reads_self`;
  `accept_calling_a_captured_callable_local` (`owned fn` → `fn() copy of f`);
  `accept_mutating_capture_through_mut_callable_parameter` (`ref mut f`; now accepted with a plain `f` too, since the
  lambda changes a real variable, not its own state); `accept_owned_closure_mutates_its_moved_capture` (`fn() owns values`).
- rewrite: `reject_a_callable_value_called_through_a_shared_reference` and
  `reject_calling_an_owned_callable_value_from_a_borrowed_place` (the stored functions change no state of their own, so
  CLO-18 accepts both; to keep a refusal, store a handler that changes its own state: E3023);
  `reject_lending_one_owned_callable_value_twice` (E3022 → E3023: a stateful value given to a read-only parameter, CLO-18);
  `reject_mutating_capture_through_borrowed_callable_parameter` (accepted now: real variable, not own state);
  `reject_outer_write_while_mutating_capture_is_live` (code becomes E3022 under CLO-2's "a place holding the value reaches
  the real variable", to confirm with area 2).

**CLO-3** (respell 6, rewrite 3):
- respell: `accept_a_capture_given_to_a_mut_parameter`, `accept_a_constructor_takes_a_callable`,
  `accept_a_consumed_callable_parameter_is_stored`, `accept_an_owned_callable_given_to_an_owned_parameter_is_dropped`
  (also `owns f: once fn`), `accept_lambdas_in_generic_bodies`, `accept_passing_a_callable_parameter_on`.
- rewrite: `accept_a_function_argument_is_called_directly` (a plain `f: fn(int) -> int` is now a function value: write
  `[F: fn(int) -> int]` and re-derive the `assert-c-count` symbols); `reject_a_borrowing_lambda_stored_as_a_value` (a
  local object that never leaves `main` may hold it: accepted now, CLO-4); `reject_a_stored_callable_parameter_that_is_lent_or_called_once`
  (push of a plain parameter is E3013; the `owns s` lambda is `once fn` given to a `fn` place, E3013; both were E2020/E3030).

**CLO-4** (respell 3): `accept_a_borrowing_closure_at_an_owned_callable_parameter`; `reject_a_borrowing_lambda_stored_in_a_list`
and `reject_mutating_a_captured_container` (same code and line; message text).

**CLO-5** (respell 1): `accept_a_closure_keeps_a_captured_object_alive` (`fn() owns c -> int`; without the word it is
E3026 now).

**CLO-6** (respell 11): every file — `accept_a_once_fn_parameter_takes_either_kind`,
`accept_an_owned_callable_parameter_is_called_any_number_of_times`, `accept_owned_callable_parameter_is_consumed_once`,
`accept_owned_closure_moves_capture_once`, `accept_owned_closure_mutates_then_moves_capture_once`,
`reject_a_once_closure_given_to_an_owned_fn_parameter` (E3013), `reject_a_once_closure_stored_as_a_value` (E3013),
`reject_a_once_closure_where_fn_is_required_names_once_fn` (E3013, help `owns f: once fn()`),
`reject_a_once_fn_parameter_called_twice` (`owns f`), `reject_once_closure_cannot_be_called_twice`,
`reject_once_closure_cannot_satisfy_callable` (E3013).

**CLO-6a** (respell 3): `accept_an_owned_once_fn_value_is_called_and_emptied` (`Box` → `owns`),
`reject_a_once_fn_value_called_twice_or_from_a_borrowed_place` (E3012 → E3013, E3041 → E3040),
`reject_a_once_fn_value_where_fn_is_wanted` (E3030 → E3013).

**CLO-7** (respell 1, rewrite 1): respell `accept_spawn_takes_its_callback_over` (`fn() owns message`); rewrite
`reject_a_callback_that_borrows` (E3026 → E7004: an unscoped thread uses a name with no capture word).

**CLO-10** (respell 7, rewrite 5):
- respell: `accept_a_callable_type_written_send_holds_only_what_is_send` (`thread_shared fn`),
  `accept_a_closure_stored_in_a_field_owns_its_captures`, `accept_a_lambda_returning_a_literal`,
  `accept_an_owned_callable_value_is_send_when_its_captures_are`, `accept_an_owned_callable_value_keeps_its_state`,
  `accept_lists_of_owned_lambdas`, `reject_large_captured_state_in_noalloc_code` (message: more than one word).
- rewrite: `accept_a_function_is_a_value_with_nothing_in_it` (`assert-c` names the 40-byte struct);
  `accept_small_captured_state_in_noalloc_code` and `accept_wide_captured_state_is_held_inline` (three words, a `u128`:
  more than one word, a block, `Alloc`, now E4001); `reject_a_send_callable_type_takes_only_what_is_send` and
  `reject_an_owned_callable_value_capturing_what_is_not_send` (a class object handed over with `owns` may go to another
  thread; `+ Send`/`+ Sync` gone; E7004/E7005 → E7015 cases).

**CLO-12** (respell 2): `accept_a_local_function_writes_a_capture_recursively`,
`reject_a_capture_written_while_the_local_function_lives` (message).

**CLO-13** (retire 2): `accept_a_lambda_copies_what_it_only_reads`, `reject_a_copy_of_a_variable_assigned_later`.

**CLO-14** (respell 6): all six (`[F: …]` above; `str` → `StrView`; messages `int`).

**CLO-15** (retire 2): `accept_a_lambda_at_an_adapter_keeps_its_borrows`, `reject_what_an_adapter_holds_is_still_borrowed_or_moved`.

**CLS-2** (respell 6): all six (`fn init(owns name: String)`; `fn drop()`; message `this`).

**CLS-4** (respell 2, rewrite 1): respell `accept_an_override_may_be_overridden`, `reject_a_method_replacing_an_inherited_one`;
rewrite `accept_an_inherited_mut_self_method_reads_a_base_typed_copy` (its `assert-c` checks the address-of-handle ABI
of `mut self`, which goes).

**CLS-5** (respell 1): `accept_interface_calls_on_a_class_are_static_unless_dyn` (`owns dyn Speak`).

**CLS-7** (respell 5, rewrite 4):
- respell: `accept_plain_class_method_writes_copy_field`, `accept_plain_class_method_writes_its_field`,
  `reject_assigning_self_in_a_class_method` (`this =`), `reject_repointing_self_through_a_mut_parameter`
  (`repoint(ref mut this, …)`), `reject_repointing_self_through_mem_swap`.
- rewrite: `accept_a_field_is_written_through_any_handle` and `accept_a_mut_self_method_writes_its_object` (rely on
  `alias = h` aliasing; now a hand-over); `run_fail_a_field_written_through_another_handle_while_viewed` and
  `run_fail_plain_method_writes_a_viewed_field` (compile-fail E3060 at the view's later use).

**CLS-8** (retire 1): `accept_a_sync_class_drop_reads_its_fields`.

**CLS-9** (respell 1, rewrite 1): respell `reject_a_let_field_assigned_outside_init`; rewrite
`accept_a_let_field_set_in_init_and_changed_through` (pushing to a `let` list is E2104 now, CLS-9a).

**CLS-11** (respell 4): all four.

**DSP-1, DSP-3, DSP-5** (respell 1 each): `accept_a_call_on_a_final_class_is_direct` (no `base: Base = child`
hand-over), `accept_interface_calls_try_the_implementing_classes_first` (`Array[owns dyn Shape]`), `accept_a_call_with_one_implementation_is_direct`;
their `assert-c` strings are re-derived after area 3's header.

**EFF-1** (respell 2, rewrite 1): respell `accept_a_call_through_a_plain_fn_value_has_the_effects_of_its_functions`
(lambda parameter words), `reject_an_effect_through_callees_and_drops` (`fn drop()`); rewrite
`reject_a_plain_fn_value_of_a_shape_some_function_allocates_in` (each lambda is its own instance: accepted; AJ renames it).

**EFF-12** (respell 1, retire 1): respell `accept_static_safe_allows_bounds_and_arithmetic` (`@no_alias_checks`); retire
`reject_a_parameter_forcing_a_dynamic_check` (EFF-6a, `RefCell`).

**EFF-13** (retire 2): `accept_a_check_proved_away_is_static_safe`, `reject_a_static_safe_function_keeping_a_dynamic_check`.

**EFF-15** (respell 1): `reject_the_same_verdict_in_every_profile` (`ref mut xs`). **EFF-16** (respell 1):
`reject_a_written_panic_says_what_is_allowed` (message adds the count compare).

**ERR-2** (respell 3): `accept_question_mark_converts_with_from`, `reject_option_and_result_crossed`,
`reject_question_mark_in_a_function_returning_neither`. **ERR-3** (respell 2): `accept_a_written_error_with_a_source`,
`reject_misplaced_or_incomplete_error_texts`. **ERR-4** (respell 8): `accept_as_ref_and_as_mut_reach_the_payload_where_it_is`,
`accept_methods_taking_a_function` (`fn(owns s)`), `accept_option_and_result_methods`, `accept_option_as_ref_and_as_mut`,
`accept_the_helpers_take_a_once_fn`, `reject_map_consumes_its_receiver`, `run_fail_expect_on_err`, `run_fail_unwrap_on_err`.
**ERR-8** (respell 3): `accept_a_parse_error_converts_into_any_error`, `accept_an_io_error_converts_into_any_error`,
`reject_any_error_is_not_an_error`. **ERR-9** (respell 1): `accept_result_defaults_to_any_error`.

**EXC-1** (respell 1, rewrite 3): respell `accept_a_write_after_the_view_ends` (sharing words); rewrite
`run_fail_a_write_while_a_view_of_the_field_lives`, `run_fail_clearing_while_a_span_of_the_field_lives`,
`run_fail_pushing_while_iterating_the_field` (compile-fail E3060 / E3020).

**EXC-2** (respell 1, rewrite 1): respell `accept_reads_of_one_field_share_it`; rewrite
`run_fail_a_read_while_a_mut_self_call_writes` (needs a method whose body changes `this`: E3021 at the call, BRW-11).

**EXC-3** (rewrite 4, retire 1): rewrite `reject_a_conflict_through_one_handle_is_e3080` (E3080 → E3060 twice),
`reject_a_map_field_is_named_as_the_program_writes_it` (E3080 → E3021), `run_fail_a_read_during_a_field_write_held_by_a_call`
(E3021, overlapping arguments), `run_fail_a_read_during_a_write_held_elsewhere` (needs a changing `update`: E3022/E3021);
retire `accept_a_check_nothing_held_could_fail_is_removed` (EXC-3a).

**EXC-5** (retire 4), **EXC-6** (retire 4), **EXC-7** (retire 3), **EXC-8** (retire 6), **EXC-10** (retire 3): every
file in each (`EXC-7/ember.toml` goes with its folder).

**EXC-15** (respell 1, rewrite 2, retire 1): respell `accept_a_drop_calls_mut_self_methods_on_itself`; rewrite
`run_fail_a_drop_holds_its_objects_write` and `run_fail_an_interface_call_while_a_field_is_viewed` (compile-fail E3060,
through `reset`'s and every implementation's summary); retire `accept_a_mut_self_call_through_an_interface_writes_once`.

**EXC-18** (retire 16): every file. Ten of them (`run_fail_*`) describe cases 0.9.15 refuses when compiling (E3060
after a call whose summary ends what a returned view points into); area 2 may re-home them as rewrites under BRW-17.

**EXC-19** (respell 2, rewrite 4, retire 2): respell `accept_accesses_to_different_fields_never_conflict`,
`accept_packed_access_words_in_a_base_and_a_derived_class` (sharing words; comment about words dropped); rewrite
`run_fail_a_derived_mut_self_call_while_a_base_field_is_viewed`, `run_fail_a_method_that_only_calls_quiet_functions_is_still_checked`,
`run_fail_a_quiet_mut_self_call_while_a_field_is_viewed` (each method changes a field the loop does not walk: run-pass
now) and `run_fail_a_mut_self_call_while_a_field_is_viewed` (E3060); retire `accept_a_loop_check_runs_once_before_the_loop`,
`run_fail_a_loop_check_before_the_loop_still_panics` (EXC-8).

### 1c. Folders of my families whose rules did not change

Not in H.41, so outside the table, but their programs use old spellings and must be respelled with
the slices that change the words: CLO-11 (1 of 2), CLS-10 (2 of 2), EFF-5 (2/3), EFF-6 (2/2), EFF-7
(3/5), EFF-8 (2/2), EFF-23 (1/1), ERR-5 (2/2), ERR-7 (2/2), ERR-10 (1/2), IFC-2 (2/3), IFC-3 (7/9), IFC-4
(50/51) (regex count of `self`, `owned`, `mut x:`, `str`, `Box`, generic brackets on the `fn` line).
Folders to delete with their rules: `EXC-5`, `EXC-6`, `EXC-7`, `EXC-8`, `EXC-10`, `EXC-18`, `CLO-13`,
`CLO-15`, `CLS-8`, `EFF-13` (the TST-4a check in `milestones.rs:7029-7037` requires an `accept_` file in
every folder, so a folder cannot be left empty). Folders to create, each from design tests: CLO-16..19,
CLS-12, DSP-4, DSP-6, EFF-24..27, EXC-16, EXC-17, IFC-5, IFC-6, MOD-9, BLD-14.

---

## 2. Work items

Ids `A4-n`. Sizes S (< 1 day), M (days), L (a week or more). Each item updates the old tests it
names in the same change, so every slice leaves the suite green.

### Slice 1 — whole-program facts, nothing new refused (after area 9's A9-2, AJ merged)

**A4-1 One call graph with every target** (M)
- Changes: new `ember_analysis/src/targets.rs`; `effects.rs:744-760`, `:904-936`, `:1100-1146`;
  `stores.rs:30-60` (`dynamic_bodies`).
- What: one graph shared by effects and summaries. Edges: direct calls; a call through a function
  value → every function converted to **that exact callable type** anywhere in the program (AJ's item
  → shape conversions, `Const::Fn` operands, and `AggregateKind::Callable`, `ember_mir/src/lib.rs:1182`),
  plus conversions between callable types (`thread_shared fn()` → `fn()`); a `dyn` call → every
  implementation in the program's `InterfaceUpcast`/`ClassInterfaceUpcast` casts (`stores.rs:52-55`
  already reads them); a virtual call → the slot's overrides in the class family (`effects.rs:1107`);
  ending a value → its drop glue (`effects.rs:1214-1300`); a spawn/`scope.spawn`/job → its function;
  a foreign call → every callback handed to C with `callback(retained|once)`, unless the declaration
  says `@ffi(no_callbacks)`. D-877's `plain_function_values` is deleted.
- Rules: EFF-2 (targets), CLO-17 and DSP-5 and IFC-5 (target half), EFF-26 (edges).
- Tests: EFF-1 `reject_a_plain_fn_value…` (rewrite, §1b); EFF-2 keeps 2 green.
- Depends: area 9 A9-2 (AJ); area 6 for the callback contract words (until then every foreign call
  counts every callback, which is the safe side).

**A4-2 The summary engine** (L)
- Changes: new `ember_analysis/src/summaries.rs`; `ember_mir/src/lib.rs` (a `FunctionSummary` on
  `Body`, next to `callable_regions` at `:141`, serde like it); driver: a pass between drop elaboration
  (`ember_driver/src/lib.rs:5302`) and the borrow check (`:5336`); kept bodies reinstall theirs as
  `install_callable_regions_kept` does (`:3923`).
- What: the facts of EFF-24 for every body, over A4-1's graph, SCC by SCC to a fixed point (shared
  decision SD1): changes / reads / reshapes / ends per abstract place with a witness for the chain;
  per view parameter (`this` included) the places it may end or move before its last use; per `ref mut`
  parameter what it reaches by other names while live; per `ref mut` list parameter "only adds";
  elements touched; where it keeps a view or function-value parameter (the `StoreSummary` of
  `regions.rs` generalised); whether it lets `this` out and which fields of `this` it reads; whether a
  function's body may end its own state and use it after. Slots for area 3 (parent link set before read)
  and area 5 (locks taken / needed, `reduce(op)`, `under(lock)`) filled by their transfer functions.
- Rules: EFF-24, EFF-1 (summary half), EFF-26 (counting), DSP-5 (changes from overrides), CLS-4
  (virtual counts overrides), CLO-14 (per instance: MIR is already monomorphised).
- Tests: unit tests in `ember_analysis` (no user-visible change yet). Design tests pass only with
  their consumers (A4-4 …).
- Depends: A4-1; area 1 A1-17 (`static mut` as a safe module variable: statics become places);
  area 1 A1-21 consumes the drop half.

**A4-3 Summaries in interface files** (M)
- Changes: `ember_build/src/interface.rs` (schema 15: per callable its summary bytes with each chain
  witness as file and line; per module the function values it converts into each callable type and
  the callbacks it hands to C); `ember_driver/src/lib.rs:4056-4100` (fill, hash);
  `ember_driver/src/incremental.rs` (a changed summary is an interface change: dependants re-check).
- Rules: EFF-4, BLD-14, MOD-9 (facts half).
- Tests: R17 (13 in the folder; BLD-14 named by 12); the EFF-4 milestone test extended with a
  summary change (a library's two lines swapped).
- Depends: A4-2.

### Slice 2a — what calls change, checked when compiling (after area 1's A1-7, A1-8, A1-9 and area 2's view model)

**A4-4 Summaries applied at calls** (L; built jointly with area 2)
- Changes: the borrow checker (`ember_analysis/src/borrows.rs`, area 2's rework of it): one entry
  point maps a callee's summary onto the caller's places (parameters to arguments, `this` to the
  object, statics to themselves, links by class family, a function value's captured real variables to
  the places that hold it) and raises: E3020 (a walk and a call that may change it), E3021 (a read-only
  name and a call that may change what it reads), E3060 (a view across a call that may end or reshape
  what it points into), E3022 (a function value stored in a place a real variable it changes owns,
  CLO-17; a function value counted as an argument changing what its functions change, BRW-11), E3023
  (CLO-18's stateful function value through a read-only holder or to a read-only parameter); every
  message names the chain (R18), built from the summary witnesses as `effects.rs:804-838` builds effect
  chains today.
- Rules: EFF-25, EFF-26 (consumer), CLO-17, CLO-18 (checks), EXC-16 (calls).
- Tests: the 35 design tests with E3020 through a function or function value (R5/R12 design rules);
  E3021, E3022, E3060 tests of R5, R8, R10, R12 whose change comes through a call; old tests CLO-2
  rewrites (4), EXC-15 rewrites (2), CLS-7 run_fail rewrites (2).
- Depends: A4-2; area 2 (BRW-1, BRW-11, BRW-17, BRW-18, R8's loans).

**A4-5 The call check E3031** (M)
- Changes: same entry point as A4-4; the fact from A4-2.
- What: an argument that points into what the callee may end or move before its last use of that
  parameter (plain, `Span`, `ref mut`, the method's object) is E3031; for a `ref mut` parameter also
  what the callee reaches by another name while it is live, unless one whole variable proved (area 2's
  verdict-88 copies); a function stored into a function value that does so with what the value's calls
  give it is refused at the store line (shape B16). Where E3021 and E3031 both fit, E3031 (R18).
- Rules: BRW-21 (area 2's row; built here), EXC-15 (E3031 half), EFF-25.
- Tests: the 10 design tests with E3031 (R4 4, R5 1, R8 3, R11 1, R17 1).
- Depends: A4-2, A4-4; area 2 (A2 item for verdict 88, BRW-22 noalias).

**A4-6 Class objects checked when compiling** (M)
- Changes: the may-alias query (SD5) used by area 2's checker; messages with the `if t is not a:` line.
- What: EXC-3's rule (same class family, at least one path through a link or kept view, unless known
  different: different classes, a fresh object, inside `if a is not b:`); fields separate (EXC-19);
  a memory-owning field given a new value is a reshape (EXC-16); a number/`bool` write is not, but is
  E3021 against a live read-only view through another name (EXC-17); `sharedref mut` views get no
  alias fact.
- Rules: EXC-1, EXC-2, EXC-3, EXC-16, EXC-17, EXC-19.
- Tests: old EXC-1 (1 respell, 3 rewrite), EXC-2 (1+1), EXC-3 (4 rewrite), EXC-19 (2 respell, 4 rewrite);
  R8's class-field tests by design rule.
- Depends: A4-4; area 3 (links, `shares mut`, `sharedref mut`); area 1 A1-9.

**A4-7 Methods and `this`** (M)
- Changes: typeck method-call sites; `lower.rs:296-345`, `:737-831` (the receiver's whole-object
  write goes in A4-8); E2103 messages (`lib.rs:25845`).
- What: what a method does to its object comes from its summary; a may-change method through a
  read-only name is E3023 (message names the line, `for ref mut p in …`); `this` is never noalias
  (stays as today); `drop` changes its object as a given; `this` stored or an owner made in `drop`:
  E3016 (owner half needs area 3).
- Rules: EXC-15, CLS-7, CLS-7a, DSP-5 (the summary half).
- Tests: CLS-7 (5 respell, 2 rewrite: the aliasing ones), EXC-15 respell 1, R5's method tests (E3023,
  R5 3).
- Depends: A4-2, A4-4; area 1 A1-7 (`this`, `fn drop():`).

### Slice 2b — the run-time exclusivity machinery removed (directly after 2a, never before)

**A4-8 Remove run-time exclusivity** (L, mostly deletion)
- Deletes: `StmtKind::BeginAccess`, `BeginAccessTransfer`, `EndAccess`, `EndAccessTransfer`,
  `LendAccess` and `HoistedAccess` (`ember_mir/src/lib.rs:181`, `:327-360`, `:1274-1310`) and every
  match arm on them (`verify.rs`, `coroutines.rs`, `inline.rs`, `drops.rs`, `regions.rs`, `copies.rs`,
  `loop_version.rs`, `range_facts.rs`, `frame_objects.rs`, `stack_lists.rs`, …; 40 files mention them);
  `ember_analysis/src/access.rs`, `loop_access.rs`, `check_hoisting.rs`, `long_access_lint.rs`;
  `borrows.rs:316-1900`'s access materialisation and E3080 (`:4699`); `lower.rs:2652`, `:2750`, `:2841`;
  driver passes `lib.rs:5410-5426` and the L3013 lint (`:5350`); access words in the class layout
  (`ember_types/src/lib.rs:1426`, `:2429`), C (`ember_codegen_c/src/lib.rs:4939`, `:5522`, `:7521`,
  `:8165-8330`, `:14491`), LLVM (`func.rs`, `glue.rs`, `runtime.rs`, `dispatch.rs`), the evaluator
  (`ember_interp/src/object.rs`), the runtime (`ember_rt.h:1953-1963`, `:2674-2682`, `ember_rt.c`, the
  two templates); the `DYNAMIC_*` rows of the safety table (`ember_codegen_c/src/lib.rs:706`); codes
  E3080 and L3013 marked retired.
- Rules: EXC-3a, EXC-4..EXC-14, EXC-18 (remove), EFF-13 (remove), EXC-15's whole-object write.
- Tests: retire 40 EXC files and EFF-13's 2; delete their folders; milestone tests for EXC-8/10/12/13.
- Cost: removes a load and compare per field access, a store per held access and 4 bytes per
  non-`Copy` field; the class layout loses its access words in the same change as area 3's header
  (one layout change, not two).
- Depends: A4-4, A4-5, A4-6, A4-7; area 3's header item (same release).

### Slice 3 — function values (after area 7's capture-list parse, area 1's `owns`/`copy of`, area 3's `shares`/`weakref`, area 5's A5-10)

**A4-9 Capture words and the real variable** (L)
- Changes: typeck `synth_lambda_fresh` (`lib.rs:48236-48600`), `CaptureWatch`, local functions
  (`:39895-40050`), thread-spawn capture rules (`:52333-52820`); delete CLO-13 (`:48408`, `:29694`) and
  CLO-15 (`borrows.rs:3480`) and `lambda.is_owned` (`ember_ast/src/lib.rs:837`, parser `body.rs:1650`).
- What: a name with no word is the real variable (an address in the state, `ref` or `ref mut` by use);
  `copy of` copies; `owns` hands over (numbers included; E3040 after); `shares`, `shares mut`,
  `thread_shared` make one more owner; `weakref` a link; `ref` a view of what the name views now (a
  loop's element, verdict 93); E0120, E0116, E0100 (`with`), E1010 (`owned fn`), E0109 (`owns fn`) with
  the spec's messages (parse by area 7); CLO-4's "provably ends before" from A4-2's keeps facts, E3026
  naming the word; CLO-19 lambda parameter words (E2020).
- Rules: CLO-1, CLO-2, CLO-4, CLO-5, CLO-7 (no rule of the call), CLO-12, CLO-19, ERR-4 (lambda words);
  removes CLO-13, CLO-15.
- Tests: old CLO-1..CLO-7, CLO-12 respells (§1b), CLO-13/CLO-15 retired; design: CLO-2 2, CLO-4 9, and
  the capture-word errors (E0100 4 in R12, E0116 4 in R3/R13, E0120 1 in R12, E3026 6 in R12/R15).
- Depends: A4-2; area 7 (capture list after the brackets, GRM-42); area 1 (A1-1, A1-2, A1-5); area 3
  (`shares`, `weakref`).

**A4-10 Function values: two words, own state, `once`** (L)
- Changes: `ember_types` (`TyKind::Fn`: `send` → `thread_shared` with area 5, stateless/stateful
  distinction kept), typeck callable parameters (`lib.rs:1773`, `:1833-1862`, `:5154`: the implicit
  generic goes), MIR `AggregateKind::Callable`, `ember_codegen_c/src/lib.rs:3065-3180`, `:5680`,
  `:6369`, `ember_codegen_llvm/src/closures.rs`, `ember_interp/src/call.rs`, the runtime.
- What: SD3's two-word value; state ≤ one word inline, larger in one block (`Alloc`); `=` hands a
  stateful value over; a call through a list element or field copies the two words out; replacing or
  ending one ends what its state owns (drop counted, for A4-2); `==` on both words; a plain
  `f: once fn` is E3013, E3030 folded; CLO-18's one count around a call whose body may end its own
  state (placed everywhere by default, removed by the summary, SD7); `extern "C"` conversion panics
  with state.
- Rules: CLO-3, CLO-6, CLO-6a, CLO-10, CLO-14 (the parameter form vs the type-parameter form), CLO-18
  (count), CLO-16 (target side; the type is A5-10's).
- Tests: CLO-3 rewrites (3), CLO-6 (11), CLO-6a (3), CLO-10 (7 respell, 5 rewrite), CLO-14 (6); design
  CLO-3 10, CLO-6 7, R12's dispatch tests.
- Cost: 16 bytes per value (40 today); a call copies two words and makes one indirect call (plus one
  load of the descriptor under SD3's choice (a)); no allocation for a state of one word or less.
- Depends: A4-1, A4-2, A4-9; area 3 (block header for the count); area 5 A5-10.

### Slice 4 — effects and contracts revised (after A4-8 and area 3's run-time checks)

**A4-11 Effects and contracts of 0.9.15** (M)
- Changes: `effects.rs:978-983`, `:1076` (kinds), `static_safe.rs` (rewritten as `@no_alias_checks`),
  typeck attribute handling (`@static_safe` E0104 with the rename; EFF-6a's check `lib.rs:19818-19840`
  deleted), the side table (`ember_codegen_c/src/lib.rs:700-730`), the E4040 message.
- What: `Aliasing` = `assert_disjoint`/`check_disjoint` only; the count-limit compare is `Arithmetic`;
  the `weakref` header test is `Stale` with reason `requested_by_type`; the proof list of EFF-15 (EXC
  proofs out, parent-link proof in, area 3); `with x.lock:` effects (area 5's `Lock`); E4010 for an
  implementation missing its interface's contract (not built today); contracts written on a function
  type checked against every stored function (blocked on Q1).
- Rules: EFF-1, EFF-2, EFF-3, EFF-9, EFF-11, EFF-12, EFF-14, EFF-15, EFF-16, EFF-18; removes EFF-6a.
- Tests: EFF-12 respell 1 / retire 1, EFF-15 and EFF-16 respell 1 each; design EFF-12 1 (R16).
- Depends: A4-1, A4-8; area 3 (count compare, `weakref` test, parent-link proof); area 5 (A5-16).

### Slice 5 — `init` and `@cached`

**A4-12 What `init` may call** (M)
- Changes: move the whole-`this` check from `lib.rs:24131-24310` to a MIR pass over `init` bodies
  (next to `definite_init.rs`), reading A4-2's "fields of `this` read" and "lets `this` out".
- What: a method may be called once every field its body reads is set; one that lets `this` out
  only after every field is set; capturing `this`, `weakref this`, `shares this` are whole uses; E2108
  for `owns this` on `init`; the spec's messages with line numbers.
- Rules: CLS-2. Tests: CLS-2 respell 6; design CLS-2 3 (R5; E2100 2, E2108 1).
- Depends: A4-2; area 1 A1-7; area 3 (`weakref this`).

**A4-13 `@cached`** (L)
- Changes: typeck attribute check; `ember_types` layout (a hidden flag and kept value per `@cached`
  method); MIR lowering of the method (flag test, fill); a pass placing clears; both backends;
  `ember_interp`.
- What: E2117 for a method with parameters, one whose body changes anything (summary), one that reads a
  `shares`/`shares mut`/`thread_shared`/`weakref`/`sharedref mut`/`Cell` field or a field changed under a
  lock; E2117 at a `sharedref mut`/`shares mut` link made into anything such a method reads; a clear at
  every line that may change a read field through any name (a store, a `ref mut` argument, a call whose
  summary changes it); a non-`Copy` result as a `ref T` view into the kept value, ended by a clear
  (E3060 through A4-4); a once-flag on a `thread_shared` object; a reload clears (area 6).
- Rules: CLS-12, EFF-27. Tests: design CLS-12 4 (R5: E2117 4) and `R5/run_a_cached_result_is_cleared_by_a_change_to_what_it_reads`.
- Cost: one flag test per call, one store per change of a read field (the rules' cost table).
- Depends: A4-2, A4-4; area 3 (links); area 6 (reload).

### Slice 6 — interface values, kind tests, casts, bounds, class rules

**A4-14 Interface values, kind tests and casts** (L)
- Changes: `ember_types` (`Dyn` with owner words; `ClassInterface` folded into `dyn` values), typeck
  (`lib.rs:13390`, `:14787`, `:37614-37700`), `dyn_incompatibility` (`:12993-13040`), runtime
  `ember_downcast` (`ember_rt.c:3408`), dispatch in both backends.
- What: `owns dyn I`, `shares dyn I`, `shares mut dyn I`, `dyn I` as a view parameter; E2173 for a bare
  interface name as a type; a generic method whose type parameter is only a `ref`/`ref mut`
  parameter's type gets one `dyn` entry (its instance with the parameter as `dyn` of its bound), one
  that returns or stores it E2050; `if s is T as name:` with words (a place stays a place), `elif`,
  `else`, E2036, E2037, E2106; `cast s as T` on `dyn` values and base classes with the spec's stop
  text; `ref mut cast place as T`; E3027 through a plain one; `cast x if D`/`as?` E1010 with the line;
  one type-id compare (SD6); `dyn` tables through the type index (DSP-3).
- Rules: IFC-5, DSP-3, DSP-4, DSP-6, CLS-5 (text); EXP-9 with area 1's A1-20.
- Tests: DSP-1/3/5 respell 3, CLS-5 respell 1; design R14 39 (DSP-4 2, IFC-5 3 by ID; E2036 1, E2037 1,
  E2106 1, E2050 1, E2173 2).
- Depends: area 1 A1-6 (`owns dyn`), A1-20; area 3 (header with type index, `shares dyn`); area 7
  (GRM-44/45 parse).

**A4-15 Bounds of a generic function** (M)
- Changes: typeck generic signature collection and body check (E2072 sites `lib.rs:49915-50522`).
- What: a non-`pub` function's unwritten bounds inferred from its body (each method called on `T`
  → the interface in scope that declares it; an operator → its operator interface; `copy of` →
  `Clone`); none declares it: E2072 with the spec's message; a `pub` function with a missing bound:
  one E2040 on the first line of the body that needs it, listing every bound.
- Rules: IFC-6, MOD-9 (bounds half), BLD-14 (written words).
- Tests: design IFC-6 2 (R14 1, R17 1) and the R14/R17 private-generic accepts (4).
- Depends: area 7 (`[T]` above); Q2.

**A4-16 Class rules** (S)
- `virtual` in a final class E2112 (was W2111, `lib.rs:10987`); `owns` on a class type E2114 (with
  A1-6); `let` fields deep (CLS-9a: no push, no re-point, no change through them; E2104 message);
  `@sync` class rule deleted with area 5's A5-9 (CLS-8).
- Rules: CLS-4, CLS-1 (E2114), CLS-9, CLS-9a, CLS-8, CLS-11 (text).
- Tests: CLS-4 respell 2, rewrite 1; CLS-9 respell 1, rewrite 1; CLS-8 retire 1; CLS-11 respell 4;
  design CLS-4 1, CLS-9 1.
- Depends: A4-2 (what changes through a `let` field, for calls); area 1 A1-6.

**A4-17 ERR follow-ups** (S)
- Nothing in the compiler beyond A4-9's lambda words; std spellings are area 8's (`AnyError` over
  `owns dyn Error`, the helpers' `[F: once fn(owns T) -> U]` above `fn map(owns f: F) owns this`).
- Tests: ERR-2 respell 3, ERR-3 2, ERR-4 8, ERR-8 3, ERR-9 1.
- Depends: A4-9; area 8.

**Order:** A4-1 → A4-2 → A4-3 (slice 1); A4-4 → A4-5, A4-6, A4-7 (slice 2a); A4-8 (2b); A4-9 → A4-10
(3); A4-11 (4); A4-12, A4-13 (5); A4-14, A4-15, A4-16, A4-17 (6, independent of each other).

---

## 3. Shared decisions

**SD1. What a summary holds and where it lives.**
- *Where:* a `FunctionSummary` on each MIR `Body` (beside `callable_regions`), serialised with kept MIR
  and in `.emif` schema 15; computed by `ember_analysis::summaries` after drop elaboration and before
  the borrow check, because every borrow rule at a call now reads it (today the borrow check at
  `ember_driver/src/lib.rs:5336` runs before effects at `:5442`).
- *Abstract places* (what an entry is about): `Param(i, path)` (into the argument: fields, elements,
  payloads, its owned parts, never through a link), `This(path)`, `Static(symbol, path)`,
  `Linked(class family root or type, field path)` for any object reached through a `shares`/`weakref`/
  `sharedref mut`/`thread_shared` link, `Captured(k, path)` for a lambda's real variable k (mapped to the
  maker's variable where the lambda is made), `HeldBy(i, path)` for the real variables used by function
  values stored under a parameter's place (CLO-2: the holder reaches them), `OwnState` for a function
  value's state. Paths are k-limited (proposal: 4 steps); a longer path widens to its prefix, which
  only refuses more, never less.
- *Kinds:* read, change, reshape (may move or free memory), end (may end an object), and area 5's
  `reduce(op)` and `under(lock)` (its §4). Each entry carries a witness (the line in this body and the
  callee it came through), so a message rebuilds "`tick()` (line 13) adds to `LOG` (through `note`,
  line 4)" by following callees, as `effects.rs:804-838` already does for effect chains.
- *Per-parameter facts:* the places it may end/move before its last use of each view parameter
  (`this` included); what it reaches by other names while each `ref mut` parameter is live; "only
  adds" per `ref mut` list parameter; elements touched; where it keeps each view or function-value
  parameter; lets `this` out; fields of `this` read; may end its own state and then use it. Slots for
  area 3 (sets the parent link first) and area 5 (locks taken, locks needed held).
- *Fixed point:* SCCs of A4-1's graph in reverse topological order; a recursive group takes the union
  (EFF-1); monotone sets over a finite universe, so it terminates. Generic code is summarised per
  instance because MIR bodies are instances (CLO-14, EFF-4's last sentence).
- *Cost against C:* nothing at run time. Compile time: one more fixed point over the program, of the
  same shape as `infer_callable_summaries` (`borrows.rs:2763-2832`) and the effect fixed point; per-SCC
  bodies run on the driver's threads as those do (`[BLD-7]`).

**SD2. How a function value's possible targets are found.** By the exact callable type (parameter
words, result, `once`, `unsafe`, `thread_shared`): every function *converted* into that type anywhere in
the whole program (and through conversions between callable types). Helper AJ's function items are the
right base and should be merged first (area 9's A9-2): with them a function or lambda keeps a type of its
own until a callable type is written, so the conversion is one explicit MIR operation naming the
function, and the target set is exactly the functions converted to that type. Without AJ every
`Const::Fn` operand counts (D-877's guess by arity and result, `effects.rs:904-936`), which is both
imprecise and keyed wrongly. For `dyn`: the implementations in the program's upcast casts (already in
MIR, `stores.rs:52-55`); for `virtual`: the slot's overrides in the class family; for C: the callbacks
handed over with a keeping contract. Costs nothing at run time; type-based, not flow-based (the spec's
words "stored into that type anywhere in the program").

**SD3. A function value's layout.** Two words `{code, state}`. The spec makes the state word hold a
state of one word or less, and a larger state lives in one block. A plain `fn()` type cannot tell its
holder how to end an inline state, so I propose (a): `code` points to a read-only per-function record
`{call, drop, c_entry}` (shared by every value of that function); a call loads `call` from it, one
extra load against an entry pointer, on the same line for a hot loop of one handler; `drop` is NULL for
a state needing none; `==` compares both words; a block state starts with area 3's 8-byte header (its
count gives CLO-18's "one count across that call"). Alternative (b): `code` is the entry itself and
every state needing a drop lives in a headered block, so only `Copy` states of one word stay inline;
(b) saves the load but cannot tell an inline `Copy` word from a block pointer when the value ends, so
it needs a tag the spec's layout has no room for. Recommend (a), and measure the load in step B against
run 3's "two words copied out of the list, dispatch 1.045 and 1.037 of plain C".

**SD4. Who builds the call-site checks.** Area 4 computes the summaries (A4-2) and provides one
mapping API (callee summary → caller places, with witnesses); area 2's checker owns the loans and
raises the codes; E3031 (BRW-21) and the CLO-17/CLO-18 cases are built in area 4's items on that API.

**SD5. "May these two names reach one object?"** One query (EXC-3) used by every borrow rule: two paths
to objects of one class family where at least one goes through a link or a kept view, minus the facts
that tell them apart (different classes, a fresh object, the `if a is not b:` branch). Fields are
separate places (EXC-19).

**SD6. `dyn` values and kind tests.** A `dyn` value of any kind is one pointer to a headered block
(class objects already are; `owns dyn I` of a struct gets the same header), so DSP-3's "table through
the type index" is uniform. A kind test is one compare: type indexes numbered in preorder of the whole
program's class tree, so "is a `C` or below" is `(u32)(index - lo[C]) <= span[C]`; a final target is
`index == C`. With area 3's header (32-bit count, 32-bit type index). Today's base-chain walk
(`ember_rt.c:3408`) costs a load per level.

**SD7. "A fact never changes what a line does" (EFF-25, BLD-14) and the two places where a body fact
decides emitted code.** CLO-18's count around a call that may end the function's own state, and
`@cached`'s clears, are read as their conservative form being the meaning (a count around every call
through a stored stateful function value; a clear at every change that could reach a read field), and
the summary only *removes* those that cannot matter, which is "removes a test that would always pass".
Under that reading the summary never changes a program's behaviour, only its cost.

**SD8. Removal order.** The run-time exclusivity machinery is removed only in the slice right after the
compile-time rules land (2a then 2b). Removing it first, or with area 1's one-owner classes but without
A4-4..A4-7, leaves class-field conflicts unchecked (use-after-free in safe code).

---

## 4. Questions for the owner

**Q1. How is a contract written on a function type?** `[EFF-2]`: "A contract written on a function type
is checked against every function stored into it, with the contract's code." The grammar has no place
for it: `fn_type := ["once"] ["unsafe"] ["extern" STRING] "fn" "(" … ")" ["->" type]` (p03, line 172),
and no rule or example writes one. Blocks: building EFF-2 completely (A4-11). Options: (a) a contract
above a `type` alias (`@noalloc` above `type Handler = fn()`); (b) a contract inside the type
(`@noalloc fn()`), which needs a grammar change; (c) drop the sentence.

**Q2. Which bound does a private generic get when two interfaces in scope declare the method its body
calls?** `[IFC-6]`: "a method that no interface in scope declares is `E2072` …; the bound follows". It
says nothing when two interfaces in scope both declare `weight` (or two operator interfaces could
supply `+`), nor whether a function may write some bounds and leave others to inference. Blocks A4-15.
Options: (a) refuse with a message giving both bounds to write; (b) take both as bounds (the argument
must implement both); (c) prefer the interface declared in the function's own module.

---

## 5. Risks

1. **Type-keyed targets are coarse.** Every `fn()` value anywhere in a program is a possible target of
   every call through any `fn()`, std's own included: one handler that pushes to `LOG` makes every
   `fn()` call in every library a change of `LOG` in that program. This is the spec's chosen rule
   (CLO-17, EFF-2); its false refusals on large programs are unmeasured. Measure on the 841 design
   tests, `std` and `examples/` once A4-1/A4-2 exist.
2. **Summary precision.** k-limited paths and class-family abstraction for objects behind links must
   still accept every design accept test (R5 177, R12 196, R17 22 by design rule). The fixed point
   itself is cheap; the calibration is the unknown.
3. **Pipeline order.** Today the type checker decides E2100 on the AST and the borrow check runs before
   effects. Both move: summaries before the borrow check (SD1), E2100 into a MIR pass (A4-12).
4. **The callable parameter stops being generic.** Every user `f: fn(A) -> R` parameter becomes an
   indirect call through a function value (CLO-3). Std already uses the type-parameter form except
   `thread.spawn` and `jobs.submit` (`std/src/thread.em:25`, `std/src/jobs.em:23`); tests and examples
   that relied on the direct call (CLO-3's `assert-c-count`) change.
5. **A4-8 is wide.** About 40 files match the access statements; the class layout loses its words in
   the same release as area 3's header, and the two must be one layout change.
6. **Foreign calls.** Until area 6 lands `@ffi(no_callbacks)` and the callback contracts, every foreign
   call counts every kept callback's changes (EFF-26), which refuses loops around C calls in FFI-heavy
   code. Correct, but noisy.
7. **AJ's open points** (its Q1, a local assigned two functions; Q3, `match` arms) stay unanswered by
   0.9.15's text; area 9 carries them. AJ's defect 3 (a zero-sized C member making a state larger in C
   than the analysis thinks) matters more under CLO-10's one-word limit (area 9's A9-6).
8. **`@cached` on structs.** A `Copy` struct with a `@cached` method returning a non-`Copy` result, and
   generic types, need a layout decision for the kept value (copied with the struct). Not settled by
   the spec text; resolvable in A4-13 without the owner.
9. **E4010 is not built today** (no emitter for `codes::E4010`), a gap older than 0.9.15.
10. **Not verified by running anything.** Test outcomes under 0.9.15 are read from the spec and the
    programs; codes for two rewrites are marked "to confirm" (CLO-2's outer write, EXC-3's
    `update`). The design-test counts are by script; the old-test classes were read by hand.
