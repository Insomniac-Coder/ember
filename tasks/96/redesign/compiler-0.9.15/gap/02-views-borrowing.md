# Gap map 02: views and borrowing (BRW, LT, DSJ, SPN, CTL, SOA, SIMD, OPT, STR)

Agent 02, 2026-10-27. Read-only: nothing was built or run. The compiler is at `ab2b7b76` ("Move the
development target to spec 0.9.15_Hardened_1"); it still implements 0.9.10_Hardened_35.

**How this was made.** The 72 rules of these families that the H.41 table lists were extracted from both
spec trees with a script (`target/gap-0.9.15/02/extract.py`) and compared by hand, with
`run4/v5/rules.md` R4 and R8, XVIII.4 (`[BCK-1]` to `[BCK-8]`), `[CG-C-4]`, `[MONO-10]` and the
verdicts 53, 54, 81, 82, 86 to 89, 95 and 98 of `JOB.md`. Every claim about the compiler was checked with
`rg` on the source; file:line is given for each. **Old tests:** the 250 `.em` files in the 49 folders of
these rules were classified by hand: I read every compile-fail and run-fail test and every run-pass test
in the BRW, CTL-1, CTL-2, DSJ, LT, OPT, SIMD, SOA, SPN and STR folders; the 22 `CTL-3b` tests were sorted
by a scan for `iter_mut`, `.copied()` and `reborrow` (`target/gap-0.9.15/02/oldtests.py`), and the 7 it
flagged were read. The verdicts, with a reason per file, are in `target/gap-0.9.15/02/verdicts.py`
(section 1b reproduces them). **Design tests:** counts by `conformance-rule-map.md`
(`target/gap-0.9.15/02/design.py`); that map names most of these rules zero times (347 of its 841 rows
are mapped only by a "main" ID), so a content count over `run4/v5/conformance/` is added where it matters.
394 of the 841 design tests name at least one rule of these families.

**Overall.** The borrow checker is a working NLL loan checker (`ember_analysis/src/borrows.rs`, 5,620
lines, and `regions.rs`, 2,827 lines): liveness by last use, two-phase reservation, field disjointness,
region contracts per callee kept across builds. That machinery survives. What the new spec changes is
**what a conflict is and where it is reported** (a reshape ends an address and is reported at its *later
use*, E3060, instead of refused at the change; a list element named by index is a *position* that a push
keeps), **what a name is** (a part name is a view; `=` re-points a view name; parameters and loop names
keep their place; a plain name changes nothing), and **calls** (every call is checked against a summary
of the callee's body: E3031, the loop rule through calls, module variables, function values by every
target; nothing is checked while running). None of the interprocedural part exists: today a call is seen
only through its signature's region contract (`borrows.rs:2592-2711`). The run-time exclusivity
machinery inside the borrow pass (`insert_shared_accesses_all`, `borrows.rs:327-2017`, about 1,700
lines, and the `RefCell` guard loans) goes. No `ref mut` parameter is `restrict`/`noalias` today; only
outlined list kernels are (`ember_codegen_c/src/lib.rs:6523-6535`, `ember_codegen_llvm/src/func.rs:98`).
There is **no run-time index compare** to remove (verdict 98): two `ref mut` elements with variable
indexes are already refused when compiling (`borrows.rs:5143-5180`, `overlaps`); what is missing is the
proofs that let them through (`if i != j:`, `for j in i + 1..`).

Area numbers used below are those the other reports use: area 1 ownership (A1-n), area 3 links and
classes (`shares`, `weakref`, RefCell removal), area 4 effects, closures and the run-time exclusivity
removal, area 5 threads and unsafe (A5-n), area 7 codes and error pages, area 9 helper branches (A9-n).

## 1. Rule table

Status: `built`, `partly`, `not built`, `remove`, `text only`. Old tests: k/s/w/r = keep, respell,
rewrite, retire (every non-keep is named in section 1b). Design tests: by the rule map, `+n by content`
where the map undercounts.

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| BRW-1 | changed | a `ref mut` name is alone (E3021 read, E3022 change, by another name that may name the place, links of one type included) and a read-only name promises no change of what it reads (E3021 at the change; a reshape ends it, E3060 at its later use); checks for a `ref mut` parameter against module variables made at each call; nothing at run time | NLL loans and `check_point` (`ember_analysis/src/borrows.rs:4564-4950`): every write overlapping a live loan is refused at the write (E3021/E3022); no position/address split; no link "may be one" rule; class fields checked at run time (`insert_shared_accesses_all`, `borrows.rs:327`; E3080 classification `:4702`) | partly | BCK-3 conflict table (A2-4); per-call module-variable check (A2-10); run-time access removal (A2-11) | 6: k1 w5 | 34 (R2:1 R4:2 R6:3 R7:3 R8:17 R9:1 R11:1 R12:5 R17:1) |
| BRW-2 | changed | a view is in use until the last use of anything made from it; scope does not matter | built: backward liveness over regions (`regions.rs:1-40`, `borrows.rs:1-30`) | built | wording only | 4: k1 s2 w1 | 0 |
| BRW-3 | changed | the place may be read while a call's arguments are evaluated; a view still used at the call that overlaps a changeable argument is E3021 | built: reservation window and activation (`borrows.rs:4253`, `:4300`, `:4481`) | built | `ref mut` words at the call are area 1's (A1-8) | 6: k1 s5 | 0 |
| BRW-4 | changed | different fields are different places at any depth, for struct values and class objects alike | struct fields: `overlaps` (`borrows.rs:5143-5151`); class fields are dynamic (access words, `borrows.rs:1481-1626`) | partly | class object fields static (A2-4 with area 4's EXC removal, A2-11) | 4: k2 s1 w1 | 0 |
| BRW-5 | changed | `ref mut a[i]` beside `ref mut a[j]` (names or arguments) is E3022 unless proved different: constants, `j` from `i + 1..`, inside `if i != j:`; nothing compared at run time | constant indexes only (`borrows.rs:5152-5153`); help names `get_pair_mut` (`:4916`); `get_pair_mut` in std (`std/src/core.em:2344`, `:2397`) | partly | index proofs (A2-4); message names `if i != j:` | 10: k2 s5 w3 | 0 (+8 by content: R4:4 R8:3 R16:1) |
| BRW-6 | changed | a read-only view of `r.f` keeps `r` from changing `r.f`; `ref mut r.f` lends the part; passing a `ref mut` name to a `ref mut` parameter lends it | built (reborrow through `Deref` places, field-precise `overlaps`) | built | wording only | 2: s2 | 0 |
| BRW-7 | changed | using an unassigned place is E3050; one a hand-over may have reached is E3040 | built: `definite_init.rs`, `drops.rs` | text only | wording ("used", "hand-over") | 3: k1 s2 | 0 |
| BRW-8 | changed | plain and `ref mut` parameters by address; views and small Copy values without a Cell as copies; `owns` receives the value; a returned view into a copy parameter's own storage is E3063 | by-address rule built (`ember_codegen_c/src/lib.rs:12218`, `Ref` is `T*`); the escape is E3060 (`borrows.rs:3754-3905`) | partly | E3063 (A2-2); `owns` mode is area 1's | 3: k1 s1 w1 | 0 |
| BRW-9 | changed | a view in Safe code is never null, dangling or unaligned | built | text only | none | 2: k1 w1 | 0 |
| BRW-10 | changed | what a method does to its object is worked out from its body for every method, `pub` included, through its callees; recorded in the interface file | only private, non-virtual struct/enum methods, and only fields used: `borrows_used_fields` (`ember_typeck/src/lib.rs:25125-25136`), `receiver_fields` (`borrows.rs:2079-2130`); others take all of `self` (E3025, `borrows.rs:4837`) | partly | every method, change/read split, through callees and overrides, in the summary (A2-9, A2-10); E3025 goes | 4: s1 w3 | 0 |
| BRW-11 | changed | two arguments of one call, one changeable (a function value counts as changing what its targets change), may not overlap unless proved apart (constants, `i + 1..`, `if i != j:`, `assert_disjoint`); E3022/E3021; exception 1 one whole variable (copy of the callee), exception 2 a `thread_shared` object changed only under its lock | all argument loans live together (`check_call_activation`, `borrows.rs:4481`); no proofs, no function-value rule, no exceptions | partly | A2-10 (rule and proofs), A2-12 (`assert_disjoint`), A2-13 with A1-22 (exception 1), area 5's A5-16 (exception 2) | 2: s1 w1 | 19 (R4:11 R8:4 R12:4) |
| BRW-12 | added | a name bound to a part, or to a call whose result is a view, is a read-only view (a part with no fields of its own is copied); `ref mut`, `copy of`, link words and `ref` before the part say otherwise | `boss = enemies[0]` copies a Copy element and moves (E3011, `ember_analysis/src/drops.rs:1017-1022`) a non-Copy one | not built | part names become views (A2-3; with A1-3/A1-5 for `=` and `copy of`) | none | 39 (R1:3 R2:25 R3:1 R4:5 R6:1 R8:1 R13:1 R14:2) |
| BRW-13 | added | `=` on a view name re-points it (checked against the new target); `ref mut` names re-point with `r = ref mut b`; a `ref` given to a `ref mut` name is E3023; `const ref mut` never re-points (E3023) | the opposite: a reference local is never re-seated, `m = ref mut y` writes through (ADR-010, `ember_typeck/src/lib.rs:44143-44160`); regions assume it (`regions.rs:34-40`) | not built | A2-3 (typeck, lowering, region kills) | none | 0 (+7 `const ref mut`: R2:2 R4:5; +6 re-pointing lines) |
| BRW-14 | added | parameters, loop, pattern and opened names keep their place: `=` on one is E3023 (except `exists as copy of v`); a `ref mut` one writes or re-points the caller's place | writes to a borrowed parameter are E3023 (`reject_borrowed_parameter_write`, `ember_typeck/src/lib.rs:44291-44350`); loop and pattern names may be assigned | partly | A2-3 | none | 23 (R1:1 R2:10 R3:2 R4:4 R12:4 R13:2) |
| BRW-15 | added | a plain name never changes what it names or leads to (Cell field, `shares` target, MutSpan elements, a method that may change its object): E3023 with the line to write; locks, atomics, channels, `guarded by` fields excepted | writes through a shared `ref` are E3021 (`lib.rs:44160-44248`), through a shared span E3021 (`:44226`), to a borrowed parameter E3023 (`:44348`); Cell, links and methods through plain names allowed | partly | A2-3 (local cases), A2-10 (methods by their summary) | none | 42 (R1:2 R2:10 R3:2 R4:8 R5:3 R9:5 R12:7 R13:2 R14:3) |
| BRW-16 | added | a name for a list element made by indexing (list reached through owning locals, parameters, `this`, one-owner fields) is a position: found again at each use; adding keeps it; remove/insert/reorder/clear/replace ends it (E3060 at a later use) | no positions: an element view is a loan on the list, and a push while it lives is E3022 (`borrows.rs:4564-4950`) | not built | A2-4 (checker), A2-5 (MIR, backends, evaluator) | none | 84 (R1:1 R2:4 R4:3 R5:3 R6:4 R7:7 R8:39 R9:6 R12:6 R15:6 R17:5) |
| BRW-17 | added | every other view is an address, ended by any change that may reshape what it points into, through any name or call (worked out from code); two paths of one type through a link or kept view may be one unless proved apart; a push does not end a view of a class object in its own block | a loan refuses the reshape itself (E3021/E3022 at the change); no "reshape" notion separate from "write"; no link paths; no interprocedural reshape | partly | A2-4 (intra), A2-10 (through calls) | none | 49 (R2:2 R4:1 R6:2 R7:5 R8:18 R9:1 R10:10 R12:9 R17:1) |
| BRW-18 | added | while a `for` walks a collection (or a generator that walks it is still used), nothing changes it, whatever the change, through its own name or any call (function values, library calls given functions, `spawn`, foreign callbacks, `drop`): E3020 at the change; `for ref mut x in xs` changes the item; a counting loop may write and add but not remove | E3020 for a write or `mut` borrow of the iterable while the `for` iterator's loan lives (`borrows.rs:4790-4848`, `loan_held_by_for_iterator` `:4981`, `ember_hir/src/lib.rs:103`); nothing through calls; counting loops unchecked; no `for ref mut` handle | partly | A2-6 (direct), A2-10 (through calls) | none | 78 (R1:1 R2:1 R5:6 R6:1 R7:3 R8:23 R9:3 R12:28 R14:3 R15:7 R17:2) |
| BRW-19 | added | a temporary lives until the last use of every view made from it; returning or storing such a view past it is E3063; a changing method on a temporary whose only effect is that change is E2140 | temporaries end with the statement (`ember_mir/src/lower.rs:1020-1045`), a later use is E3060 (`borrows.rs:4752-4790`); E2140 only for assignment to a value (`ember_typeck/src/lib.rs:16449`) | partly | area 1's A1-18 and A1-19, on A2-4's view liveness; E3063 by A2-2 | none | 11 (R4:1 R6:1 R8:9) |
| BRW-20 | added | a view made through a lock guard, `read()`, `write()` or a `guarded by` field does not leave its `with` block (E3063) | not checked: JOB.md row 4 notes `return ref g[0]` compiles; guards are `@view` structs (`std/src/sync.em:53-80`) | not built | area 5's A5-19 makes the guard the source; E3063 is A2-2's | none | 0 (+6 by content: R8) |
| BRW-21 | added | the call check: from each body, which view parameters (`this` included) are used after a line that may end or move what they view, kept with the function (interface file), checked at each call (E3031); for a `ref mut` parameter also what the callee reads or changes by another name while it is used; the refusal for a stored function stands where it is stored | nothing: calls are seen through region contracts only (`borrows.rs:2592-2711`, `regions.rs`) | not built | A2-9 (facts), A2-10 (check, messages); E3031 joins the registry (area 7) | none | 15 (R4:6 R5:1 R8:6 R11:1 R17:1) (+10 E3031 by content) |
| BRW-22 | added | every `ref mut` parameter, a `MutSpan` one included, is compiled with noalias; one-variable copies have one name; never `this`, never `sharedref mut` | none: `Ref` is a plain `T*` (`ember_codegen_c/src/lib.rs:12218`); `noalias` only on outlined kernels' views (`ember_codegen_llvm/src/func.rs:98-103`) | not built | A2-14 (after A2-10 and A2-13) | none | 0 (+2 by content: the R4 `add_both` and `rename(ref mut LOG)` runs) |
| CTL-1 | changed | loop names are read-only views; `for ref mut x in e`, `for k, ref mut v in m`; `for x in owns e`; a `Map` gives keys, two names give key and value; a plain loop name changes nothing (E3023) | `check_for` (`ember_typeck/src/lib.rs:28854`), `check_for_iterator` (`:34341`); `iter_mut()` gives `ref mut` items; `for ... in owned` | partly | A2-6; `owns` word is area 1's | 12: k3 s7 w2 | 0 (+68 use `for ref mut` by content) |
| CTL-2 | changed | while a `for` walks a collection nothing changes it but the loop's handle (BRW-18); counting loops may write and add, not remove; `while` on the current length may do both | see BRW-18 | partly | A2-6, A2-10 | 3: k1 s2 | 78 (as BRW-18) |
| CTL-3b | changed | counted loops over ranges, spans, arrays, SoA columns and adapters, `copy_each` in place of `copied` | built for today's adapters (`check_for` and `loop_version.rs`); `copied` is the std name | partly | rename (std area); changing elements through adapters needs Q3 | 22: k15 s7 | 0 |
| CTL-7 | changed | `defer` uses locals as they are when it runs; they are not dropped before it | built: defers are lowered at each exit, before the drops (`ember_mir/src/lower.rs:860-865`, `:1214-1217`), so no loan is held | built | none | none | 0 |
| CTL-11 | added | `exists as` and kind-test headers: the name is in scope in the block only, keeps its place (BRW-14), plain reads, `copy of v` may change | no `exists as`, no `is T as c` (parser: `ember_parser/src/body.rs:1538`) | not built | scope rule in A2-3, after the opening area (GRM-44) and A1-20 | none | 0 |
| DSJ-1 | changed | `check_disjoint(a, b) -> Result[void, void]` reads both, takes neither, `@must_use` (W2190); in `Ok` the names are known apart | `assert_disjoint(owned a, owned b) -> Result[(A, B), (A, B)]` (`std/src/mem.em:76`), synthesised in typeck (`ember_typeck/src/lib.rs:16250-16330`) | not built | A2-12 | 1: s1 | 0 (+7 by content: R4:6 R16:1) |
| DSJ-2 | changed | the fact belongs to the two names (after `assert_disjoint`, inside `check_disjoint`'s `Ok`); re-pointing either drops it | the fact rides on the returned tuple (`ember_analysis/src/kernels.rs:839-846`, `origin`) | partly | A2-12 | none | 0 |
| DSJ-3 | changed | where the fact holds the names may be given to one call with one changeable (BRW-11) and are `restrict`/`noalias` | `restrict` in outlined kernels only (`kernels.rs:25-31`); no BRW-11 use | partly | A2-12, A2-14 | 1: s1 | 0 |
| DSJ-5 | changed | `assert_disjoint(a, b)` stops the program with the names; `RuntimeCheck(Aliasing)`, removed where known apart | `assert_disjoint_or_panic` (`mem.em:85`); known-apart elision built (`kernels.rs:748`, `CheckProof::KnownDisjoint` `ember_mir/src/lib.rs:273`) | partly | rename, stop text (A2-12) | 2: s2 | 0 |
| DSJ-6 | changed | usable in `@noalloc`/`@nosync`; inside `@no_alias_checks` each is E4030 | usable in `@static_safe` (`ember_analysis/src/static_safe.rs:68`) | partly | the attribute is area 4's (EFF-12); A2-12 marks the site | 1: w1 | 1 (R16) (+5 R16 by content) |
| DSJ-7 | changed | `assume_disjoint(a, b)`, an `unsafe fn`, gives the names the fact | returns the pair (`mem.em:91`) | partly | A2-12 | 2: s2 | 0 |
| DSJ-9 | changed | `check_disjoint_all` and `assert_disjoint_all`, 2 to 8 views | `assert_disjoint_all` with a Result (`mem.em:81`) | partly | A2-12 | 1: s1 | 0 |
| DSJ-10 | added | making a view and comparing ranges read no element; an element read through the read-only one before the proof is E3021 | not modelled | not built | A2-12 with A2-4 | none | 0 |
| LT-1 | changed | a call's view result and a view put in through `ref mut` point into their sources; rule 1 for every method (`owns this` too); "into" covers what it owns; a `ref`/`ref mut` result is a place; a generator result walks what it views | elision rules 1 to 3 (`borrows.rs:3010` `elision_of`, `:2592` `legacy_elision`), rule 1 only for a borrowed receiver; E3062 (`:3178`) | partly | A2-1 (rule 1 for every method), A2-6 (generators), A2-10 (`ref mut` put-in) | 21: k2 s9 w10 | 18 (R4:3 R5:1 R6:5 R7:2 R8:4 R14:1 R15:2) |
| LT-1a | changed | `-> T from name` (a parameter or a bare field of `this`) replaces `@borrows`; E3062 outside it; E2031; E2118 for a name that is both | `@borrows(...)` attribute: `check_borrows_attribute` (`ember_typeck/src/lib.rs:4060-4304`); `from` is only an import keyword (`ember_parser/src/decls.rs:19`) | not built | A2-1; see Q1 | 14: s9 w2 r3 | 1 (R6) (+24 use `from` by content: R5:1 R6:9 R7:4 R8:7 R15:3) |
| LT-1b | removed | (the L3014 lint) | L3014 emitted at `ember_typeck/src/lib.rs:4048` and `:9974`; registry `ember_diag/src/codes.rs:458` | remove | A2-1 | 4: r4 | 0 |
| LT-4 | changed | a value made in a memory manager's instance lives no longer than the instance (E3063); the instance address is a kept `sharedref mut` link; arena allocations point into the arena | arena regions (`borrows.rs:3902-3921`); E3061 (`:3862`) | partly | E3061 into E3063 (A2-2); `using` instances are area 1's (A1-14, D7) | 2: k1 w1 | 0 |
| LT-5 | changed | regions by non-lexical liveness; messages in source terms ("the view of `x`") | built | text only | wording | none | 0 |
| LT-6 | changed | no named lifetimes; the help names `from` | built; helps name `@borrows` | text only | wording (A2-1) | 2: k1 s1 | 0 |
| LT-7 | changed | calls through a function type get fresh regions for their sources | built (`regions.rs`, callable contracts `borrows.rs:2018-2065`) | text only | none | 10: k4 s3 w3 | 0 |
| LT-14 | changed | one region slot per view field | built (`regions.rs:2577` `view_region_paths`) | text only | none | 2: k2 | 0 |
| LT-18 | changed | every view field is an ordinary view under BRW-1 to BRW-22 | built under the old BRW | text only | follows A2-4 | 2: k1 w1 | 0 |
| LT-22 | changed | the result summary from the body; where it needs more than the first line allows, rule 3 or `from` (E3065 gone) | summary built (`borrows.rs:2871` `inferred_result_summary`); E3065 (`:3358`) | partly | drop E3065 for the rule-3 fallback (A2-1) | 5: k2 s1 w2 | 0 |
| LT-23 | changed | with `from` on a multi-region result every field points into what `from` names (E3062 otherwise) | `@borrows` keeps each field on its own source; E3065 | not built | A2-1 | 3: s1 w2 | 0 |
| LT-25 | changed | a view field may not point into another field of its value (E3024) | built (`borrows.rs:4720-4749`) | text only | none | 2: k1 s1 | 0 |
| LT-26 | changed | a view keeps nothing alive; a view of an object through a link is checked not to outlive it | a handle temporary is retained for the borrow (ODR-065, `borrows.rs:4767-4778` help) | partly | link views per BCK-7 (A2-4 with area 3's links) | none | 0 |
| LT-27 | changed | noalias only from a `ref mut` parameter, DSJ and SIMD-3 | DSJ and kernels only | not built | A2-14 | none | 0 |
| LT-34 | changed | one-source view types act as single-region views | built | text only | none | 2: k1 w1 | 0 |
| LT-35 | changed | which fields a function uses; through a function value, `dyn`, interface or virtual method, every function the program can put there; foreign: everything | direct calls summarised (`borrows.rs:2763-2870`); dynamic targets assume every field | partly | target sets (A2-9, area 4's shared decision) | 3: k1 s1 w1 | 0 |
| LT-36 | changed | treating a view as a whole needs every region | built | text only | none | none | 0 |
| LT-39 | changed | a run-time field selection needs every region (E3021) | built | text only | wording | 2: k1 w1 | 0 |
| LT-42 | changed | a function value using a real variable or a `ref` capture of a view records its regions and may not outlive them (E3026); an `owns` capture of a view needs `static` | E3026/E3063 for stored closures (`ember_analysis/src/stores.rs:461-462`, `ember_typeck/src/lib.rs:4415-4428`); `owned fn` | partly | capture words are area 4's; view liveness A2-4 | 7: k1 s4 w2 | 0 |
| LT-44 | changed | an arena parameter is a source; `from arena` only beside other sources | built with `@borrows` | partly | `from` (A2-1) | 1: s1 | 0 |
| LT-45 | added | kept views: a struct holding a view lives no longer than its targets; a value holding views (a list of views) too (E3063); class view fields say `from <let field>` or `from static` (E2118) | `@view` structs and E3063 for stored views (`ember_typeck/src/lib.rs:4428`, `stores.rs:522`); E2030 built; no class view fields with `from` | partly | A2-8 | none | 42 (R3:3 R4:1 R6:1 R7:31 R8:4 R9:2) |
| LT-46 | added | three kept views: `ref T`, `ref mut T` (alone; second holder E3022), `sharedref mut T` (several holders, no noalias); E0114 | `ref`/`ref mut` fields only; no `sharedref` | not built | A2-8 (with area 3) | none | 1 (R7) (+22 use `sharedref mut`: R7:15 R9:7) |
| LT-47 | added | a view put in through `ref mut` points into what it did or into another argument; a callee's local is E3063; a use after an argument changed is E3060 | LT-21's replaced-field handling in `regions.rs:34-40` | partly | A2-8 (callee side), A2-10 (caller side) | none | 0 |
| OPT-1 | changed | frame placement when no owner, link or view (a `weakref` included) outlives the function | escape analysis over counted handles (`ember_analysis/src/frame_objects.rs:1-20`) | partly | A2-15 after A1-9 and area 3 | 2: s1 w1 | 0 |
| SIMD-3 | changed | alias facts derived only: (a) to (d) as before, (e) every `ref mut` parameter; views through links and `this` only with a proof that no other path reaches the object; plain parameters prove nothing | (a) to (d) in `kernels.rs:777-899` (`origin`, `apart`); links: `loop_version.rs:25-30` relies on run-time access checks (`[HEAP-5]`, `[EXC-1]`) | partly | A2-14 | 1: s1 | 0 |
| SIMD-9 | changed | `v.store(ref mut s)`, `v.scatter(ref mut s, idx)` (E3028 without) | stores take the span plainly | partly | A2-7; E3028 is A1-8's | 5: k1 s4 | 0 (+1 by content: R15) |
| SOA-2 | changed | a column is a `MutSpan` view, written only through `ref mut` names or arguments where `ps` itself may change (E3027, E3023); a length change reshapes all of `ps` (E3060 later, E3020 in a walk) | columns typed `MutSpan`/`Span` by `ps`'s mutability (`ember_typeck/src/lib.rs:50775-50833`); a length change borrows all of `ps` (E3022) | partly | A2-7 | 3: s2 w1 | 0 |
| SOA-6 | changed | `ps[i]` an `SoARef`, `ref mut ps[i]` an `SoAMut`; a proxy is a position; `ps.get(i)` a proxy; `copy of ps[i]` gathers, `load()` at a use E1010; `for ref mut p in ps` | proxies by mutability (`lib.rs:15109-15118`, `:34412`, `check_for_soa` `:29551`); `load()` public; `get` gathers | partly | A2-7, positions from A2-5 | 3: w3 | 0 |
| SPN-1 | changed | `Array`/`String` convert to `Span`/`StrView` (and `MutSpan` where `ref mut` says so); the conversion makes a view under BRW-1 | built as a borrow (`lib.rs:55599` area, `TYP-5` coercions) | partly | follows A2-3, A2-4 | 14: k1 s5 w7 r1 | 0 |
| SPN-3 | changed | `MutSpan` not Copy; writing names and parameters say `ref mut`, plain ones only read (E3023); `ref mut` before a span made in the call; `reborrow()` gone | `reborrow()` (`ember_typeck/src/lib.rs:55599`); a `mut` parameter accepts `as_mut_span()` | partly | A2-7 | 13: k1 s6 w5 r1 | 0 |
| SPN-4 | changed | `iter()` gives read-only views; `for ref mut x in s` over a `MutSpan` a `ref mut` name holds | `iter_mut()` items | partly | A2-6 | 2: s2 | 0 |
| SPN-5 | changed | `split_at` on a `MutSpan` lends it and gives two disjoint `MutSpan`s | built | text only | none | 11: k3 s4 w4 | 0 |
| STR-1 | changed | memberwise constructor; an `owns` field's argument says `owns` | built without the word | partly | A1-6 (area 1) | 5: k4 s1 | 0 |
| STR-3 | changed | not `Copy` = `=` hands over; `@derive(Copy)` gives `Clone`; class objects, links, `ref mut`, `MutSpan` not Copy | class handles are Copy (`ember_types/src/lib.rs:2122`) | partly | A1-9; links area 3 | none | 0 |
| STR-5 | changed | field-wise `Clone` copies what it owns and each link as one more link | derives in `ember_typeck/src/derives.rs`; links do not exist | partly | A1-5 and area 3 | 14: k6 s7 w1 | 0 |
| STR-6 | changed | `fn init(...)` sets fields through `this`; `owns this` on `init` is E2108 | `init(mut self, ...)` | partly | A1-7 | 3: s3 | 0 |
| STR-7 | changed | `Self` inside a type body; the example writes `[T]` above `struct Pair:` | built | text only | `[T]` above is the grammar area's (GRM-40) | 1: s1 | 0 |
| (removed) E3025, shape B8 | - | "a method takes all of `self`" no longer exists: what a method does is worked out (BRW-10) | `borrows.rs:4810-4837`, `codes.rs:297` | remove | A2-10 | in BRW-4, LT-1 above | 0 |
| (removed) `reborrow()` | - | not in SPN-3 any more | `ember_typeck/src/lib.rs:55599` (`reborrow_span`) and its callers | remove | A2-7 | in SPN-3, SPN-5 above | 0 |
| (removed) RefCell guard loans and L3011 in the borrow pass | - | RefCell is gone (area 3) | `ReferenceKind::RuntimeGuard` handling `borrows.rs:4605-4630`, `check_refcell_call` `:4006-4075`, `is_guard_loan` `:5371` | remove | A2-11 with area 3 | none here | 0 |
| (removed) run-time access brackets and E3080 in the borrow pass | - | no run-time exclusivity check (BRW-1, E3080 retired) | `insert_shared_accesses_all` `borrows.rs:327-2017` (driver `ember_driver/src/lib.rs:5410`); E3080 choice `borrows.rs:4702`; `access.rs`, `loop_access.rs`, `long_access_lint.rs` (driver `:5350`, `:5414-5418`) | remove | A2-11 with area 4 (A9-5 deletes the branch work) | `SPN-1/run_fail_string_field_view_keeps_exclusivity` r | 0 |

### 1a. Status counts

76 rows: the 72 H.41 rules and 4 removed features the borrow pass implements. built 4 (BRW-2, BRW-3,
BRW-6, CTL-7); partly 41; not built 13 (BRW-12, BRW-13, BRW-16, BRW-20, BRW-21, BRW-22, CTL-11, DSJ-1,
DSJ-10, LT-1a, LT-23, LT-27, LT-46); remove 5 (LT-1b and the four rows above); text only 13 (BRW-7,
BRW-9, LT-5, LT-6, LT-7, LT-14, LT-18, LT-25, LT-34, LT-36, LT-39, SPN-5, STR-7).

Old tests in the 49 folders: **250 files: keep 63, respell 111, rewrite 67, retire 9.** Outside the H.41
rows, two `DSJ-4` tests use `assert_disjoint`'s old result and need the same respell as DSJ-1.

The respells are mechanical (`this`, `ref mut` parameters and call words, `owns`, `StrView`, `[T]`
above, `from`, `.copy_each()`, `check_disjoint`): `ember fmt --migrate` (`[CLI-20]`) rewrites all of them
once it exists; until then they ride with area 1's renames. The 67 rewrites, by the change of meaning
behind each:

* a reshape ends an address and is reported at its later use (E3060), not at the change (E3021/E3022):
  28;
* names re-point and keep their place (BRW-13, BRW-14), plain names change nothing (BRW-15): 10;
* a returned or kept view that would outlive its target is E3063, not E3060 or E3061: 8;
* E3025 and E3065 retire (what a method does is worked out; rule 3 replaces E3065): 5;
* removed library forms (`reborrow()` 3, `@static_safe` 1, SoA's `load()` and proxies 1): 5;
* LT-1's rule 1 for every method, and `from` naming one thing: 4;
* positions survive a push: 2; one-owner classes (area 1): 2; a method's changes found through its
  callee: 1; one whole variable twice: 1; a temporary living to its view's last use: 1.

Nine of them turn a compile-fail test into run-pass.

### 1b. Old tests, file by file (every non-keep)

- **BRW-1** (6): keep 1, respell 0, rewrite 5, retire 0
  - rewrite `accept_writing_through_a_mut_ref`: `m = ref mut y` now re-points `m` (BRW-13): prints 1 5, not 5 5
  - rewrite `reject_a_reference_is_not_re_pointed`: `hottest = r` re-points a view name (BRW-13): now run-pass
  - rewrite `reject_assigning_an_owning_field_while_borrowed_once`: replacing `h.arr` ends `r` (BRW-17): E3060 at `println(r)`; `mut h` -> `ref mut h`
  - rewrite `reject_mutating_a_viewed_container`: push is a reshape: E3060 at `println(v[0])`, not E3021 at the push
  - rewrite `reject_writing_through_a_shared_ref`: `r = 99` on a `ref` name: E3023 (BRW-15), not E3021
- **BRW-2** (4): keep 1, respell 2, rewrite 1, retire 0
  - respell `accept_a_loop_keeps_an_element_across_its_turns`: `str` -> `StrView`
  - rewrite `reject_a_loop_writes_the_element_it_keeps`: `xs[0] = "z"` replaces a String `best` may view: E3060 at `println(best)`; `StrView`
  - respell `reject_a_write_before_the_last_use`: message wording only (E3021 at the write stays)
- **BRW-3** (6): keep 1, respell 5, rewrite 0, retire 0
  - respell `accept_an_argument_copied_before_activation`: `mut v` -> `ref mut v`, call `f(ref mut v, v[0])`
  - respell `accept_mut_argument_two_phase`: `ref mut a` and at the call
  - respell `accept_two_phase_borrows_across_conditional_arguments`: `ref mut values` and at the call
  - respell `reject_a_borrow_live_at_activation`: `ref mut` words; messages (E3021 stays, BRW-3/BRW-11)
  - respell `reject_a_conditional_borrow_live_at_activation`: `ref mut` words; messages
- **BRW-4** (4): keep 2, respell 1, rewrite 1, retire 0
  - rewrite `reject_method_defeats_disjoint_fields`: E3025 retired: `set_x` changes only `this.x` (BRW-10): E3022 at the call; `this`
  - respell `reject_two_mutable_borrows_of_one_field`: message wording (E3022 at the second `ref mut` stays)
- **BRW-5** (10): keep 2, respell 5, rewrite 3, retire 0
  - respell `accept_a_mutable_view_splits_in_two`: names that write a MutSpan say `ref mut` (SPN-3)
  - respell `accept_get_pair_mut_gives_two_elements`: `ref mut` params and calls; `Some((ref mut a, ref mut b))`; `StrView`
  - respell `accept_get_pair_mut_of_a_mutable_view`: `ref mut view`; pattern names `ref mut`
  - rewrite `reject_a_pair_used_after_its_array_changes`: push ends the pair: E3060 at its later use, not E3022; `owns`; returned view of a local is E3063
  - rewrite `reject_growing_while_an_element_is_borrowed`: `a` is a position (BRW-16): push allowed, now run-pass printing 10
  - rewrite `reject_mutating_array_while_split_view_lives`: E3060 at `println(left[0])`, not E3022 at the push
  - respell `reject_the_same_constant_index_twice`: message ("both are `v[0]`"); E3022 stays
  - respell `reject_two_variable_indices`: message; E3022 stays if a local holding a constant is not a constant (Q2)
- **BRW-6** (2): keep 0, respell 2, rewrite 0, retire 0
  - respell `accept_a_reborrow_is_not_a_move`: `ref mut n`, `bump(ref mut r)`
  - respell `reject_writing_while_a_shared_reborrow_is_live`: message wording (E3021 at `x = 2` stays)
- **BRW-7** (3): keep 1, respell 2, rewrite 0, retire 0
  - respell `reject_borrowing_a_moved_place`: E3040 message ("handed over")
  - respell `reject_borrowing_an_unassigned_or_moved_local`: E3050/E3040 messages
- **BRW-8** (3): keep 1, respell 1, rewrite 1, retire 0
  - rewrite `accept_a_copy_struct_holding_a_cell_is_passed_by_address`: a plain parameter changes nothing behind a `Cell` (BRW-15): `t.count.set` is E3023
  - respell `reject_an_instance_value_that_takes_by_address`: `[T]` above (GRM-40); `StrView`
- **BRW-9** (2): keep 1, respell 0, rewrite 1, retire 0
  - rewrite `reject_returning_a_reference_to_a_local`: a returned view of a local is E3063 (TYP-15, BCK-4), not E3060
- **BRW-10** (4): keep 0, respell 1, rewrite 3, retire 0
  - rewrite `accept_a_method_borrows_only_the_fields_it_uses`: `this`; second half is E3021 under LT-1 rule 1 (result points into all of `this`)
  - rewrite `reject_a_method_handing_its_receiver_elsewhere_borrows_all_of_it`: `touch` changes only `count`, found through the callee: now run-pass
  - respell `reject_a_method_using_the_iterated_field`: `this`; E3020 message names the chain
  - rewrite `reject_a_view_from_a_method_keeps_its_field_borrowed`: push ends the view: E3060 at `println(r)`, not E3021
- **BRW-11** (2): keep 0, respell 1, rewrite 1, retire 0
  - respell `accept_disjoint_mut_arguments`: `ref mut` params and call words
  - rewrite `reject_overlapping_mut_arguments`: one whole variable twice (exception 1): now run-pass
- **CTL-1** (12): keep 3, respell 7, rewrite 2, retire 0
  - respell `accept_a_for_over_a_type_parameter`: `[I: Iterator]` above; `owns`; `this`
  - respell `accept_a_kept_map_key_borrows_the_map`: `from owner`; `this`; `StrView`
  - respell `accept_a_view_iterator_loop_is_counted`: `for ref mut p in ps:`
  - respell `accept_an_adapters_items_borrow_what_it_reads`: `[I: Iterator]` above; `owns`
  - respell `accept_array_place_iteration_borrows_and_yields_refs`: `fn drop()`
  - respell `accept_owned_iteration_consumes_the_collection`: `for k in owns m`; `fn drop()`
  - respell `reject_a_collection_used_after_owned_iteration`: `owns`; E3040 message
  - rewrite `reject_writing_a_map_while_a_kept_key_lives`: E3060 at the kept key's later use (BRW-17), not E3021; the E3020 stays
  - rewrite `reject_writing_a_map_while_an_adapters_kept_item_lives`: E3060 at the later use, not E3021
- **CTL-2** (3): keep 1, respell 2, rewrite 0, retire 0
  - respell `reject_mutation_while_a_loop_binding_nothing_borrows_it`: E3020 message
  - respell `reject_mutation_while_for_loop_borrows_iterable`: E3020 message
- **CTL-3b** (22): keep 15, respell 7, rewrite 0, retire 0
  - respell `accept_a_backwards_enumerate_zipped_with_nothing_overflows_nothing`: `.copy_each()`
  - respell `accept_a_for_over_a_chain_is_one_loop_per_part`: `.copy_each()`; `ref mut` loop names over adapters (Q3)
  - respell `accept_a_for_over_rev_is_a_counted_loop`: `ref mut m` over `.rev()` (Q3); `.copy_each()`
  - respell `accept_a_mutable_view_iterates_its_elements`: `for ref mut x in view`, `view = ref mut xs.as_mut_span()`
  - respell `accept_a_provably_safe_enumerate_checks_no_number`: `for i, ref mut x in ...` (Q3)
  - respell `accept_an_adapter_chain_is_one_counted_loop`: `.copy_each()`; `for i, ref mut z in ...` (Q3)
  - respell `accept_an_enumerate_under_two_revs_numbers_forwards`: `.copy_each()`
- **DSJ-1** (1): respell `accept_assert_disjoint_gives_both_views_back`: `check_disjoint` (Result[void, void], names kept); `ref mut`; `StrView`
- **DSJ-3** (1): respell `accept_views_proved_apart_loop_in_a_restrict_function`: `check_disjoint`; `ref mut` params
- **DSJ-5** (2): respell `accept_or_panic_gives_the_pair` (`assert_disjoint(p, q)`, no result); respell `run_fail_or_panic_on_overlap` (name and stop text)
- **DSJ-6** (1): rewrite `accept_assert_disjoint_in_noalloc_nosync_static_safe`: `@static_safe` is E0104 and `@no_alias_checks` refuses the check (E4030)
- **DSJ-7** (2): respell `accept_assume_disjoint_in_unsafe` (gives the names the fact, returns nothing); respell `reject_assume_disjoint_outside_unsafe` (no result to destructure; E3100 stays)
- **DSJ-9** (1): respell `accept_assert_disjoint_all_compares_every_pair`: `check_disjoint_all`
- **LT-1** (21): keep 2, respell 9, rewrite 10, retire 0
  - respell `accept_a_borrow_through_a_reference_to_a_view`: `StrView`
  - respell `accept_a_generic_body_means_the_same_for_every_instance`: `[T]` above; `this`
  - respell `accept_a_generic_container_parameter_is_a_source`: `[T]` above
  - respell `accept_a_generic_parameter_counts_as_copy`: `[T]` above
  - respell `accept_a_mut_copy_parameter_is_not_a_source`: `ref mut pos`, call word; `StrView`
  - rewrite `accept_an_owned_receiver_is_under_rule_3`: rule 1 covers every method, `owns this` too: returning `a` is E3062 without `from a`
  - respell `accept_array_and_string_parameters_are_sources`: `StrView`
  - respell `accept_rule_1_takes_any_borrowed_receiver`: `this`, `StrView`
  - rewrite `reject_a_local_views_target_escaping`: E3060 -> E3063
  - rewrite `reject_a_reborrowed_local_view_escaping`: E3060 -> E3063
  - rewrite `reject_a_reference_into_a_view_parameters_own_storage`: E3060 -> E3063 (BRW-8); `owns P`, `owns h`
  - rewrite `reject_a_reference_to_a_view_parameter_itself`: E3060 -> E3063; `StrView`
  - respell `reject_a_view_of_a_generic_parameter`: `[T]` above; E3062 message
  - respell `reject_a_view_of_a_mut_copy_parameter_without_borrows`: `ref mut n`; help names `from n`
  - rewrite `reject_a_view_outliving_a_temporary_argument`: the temporary lives to `t`'s last use (BRW-19): now run-pass
  - rewrite `reject_items_of_a_non_copy_iterator_keep_it_borrowed`: E3025 retired: the second `w.next()` changes what `x` points into: E3021
  - rewrite `reject_mutating_an_argument_of_an_owned_receiver_call`: rule 1 covers `owns this`: the result points into `this`, not `ys`
  - rewrite `reject_mutating_an_argument_while_the_result_lives`: E3060 at `println(s[0])`, not E3021
  - rewrite `reject_using_another_parameter_while_the_result_lives`: E3060 at `println(v[0])`, not E3021
- **LT-1a** (14): keep 0, respell 9, rewrite 2, retire 3
  - respell `accept_borrows_names_a_mut_copy_parameter`: `-> ref int from n`, `ref mut n`
  - respell `accept_borrows_names_a_view_field_of_the_receiver`: `from owner`; `[T]` above; `this`
  - rewrite `accept_borrows_naming_the_receiver`: `from` names one thing: `longer` must become a free function (rule 3)
  - respell `accept_borrows_narrows_the_region`: `-> Span[i32] from a`
  - rewrite `reject_a_reference_to_a_named_view_parameters_slot`: E3060 -> E3063; `from a`
  - respell `reject_borrows_names_a_copy_struct_holding_a_cell`: `from c`; E2031 message
  - respell `reject_borrows_names_a_field_but_returns_the_receiver`: `from owner`; E3062 message
  - respell `reject_borrows_names_a_field_that_is_not_a_view`: `from at` etc.; E2031 messages; `[T]` above
  - respell `reject_borrows_names_a_generic_field_but_an_instance_returns_another`: `from inner`; `[..]` above; `.copy_each()`
  - respell `reject_borrows_names_a_mut_ref_field`: `from owner`; E3062 message
  - respell `reject_borrows_names_an_owned_arena`: `from arena`, `owns arena`
  - retire `reject_empty_borrows_attribute`: `@borrows()` gone (E0104, ATT-1)
  - retire `reject_named_borrows_argument`: `@borrows(x = x)` gone
  - retire `reject_non_name_borrows_argument`: `@borrows(0)` gone
- **LT-1b** (4): retire all: `accept_explicit_intersection`, `warns_on_implicit_intersection`, `warns_on_rule_3_over_source_parameters`, `warns_once_for_an_interface_default_method` (L3014 and `@borrows` removed; the folder's `ember.toml` goes with them)
- **LT-4** (2): keep 1 (`accept_an_allocation_within_its_arena_region`, if `Arena.alloc` survives ARN-1); rewrite `reject_an_arena_allocation_escaping_its_arena`: E3061 folded into E3063
- **LT-6** (2): keep 1; respell `reject_a_named_lifetime`: `StrView`
- **LT-7** (10): keep 4, respell 3, rewrite 3, retire 0
  - respell `accept_a_call_through_a_value_borrows_only_its_sources`: `fn(ref mut int, StrView)`, call words
  - rewrite `reject_a_callback_result_that_borrows_the_callees_local`: E3060 -> E3063
  - respell `reject_a_function_whose_borrows_reach_past_the_callable_type`: `from n`, `ref mut`; E2020 stays
  - respell `reject_a_lambda_viewing_its_mut_parameter`: `fn(ref mut x)`; help names `from`
  - rewrite `reject_a_result_that_holds_a_closures_capture`: `mem.drop(other)` ends what `r` views: E3060 at `println(r)` (or E3062 at the lambda)
  - rewrite `reject_mutating_a_source_argument_through_a_value_call`: E3060 at `println(v, n)`, not E3021
- **LT-14** (2): keep 2
- **LT-18** (2): keep 1; rewrite `reject_mutation_while_the_matching_field_region_is_live`: E3060 at the later use, not E3021
- **LT-22** (5): keep 2; respell `accept_literal_arguments_to_a_multi_region_result` (`StrView`); rewrite `reject_opaque_multi_region_local_result` and `reject_opaque_multi_region_result_provenance` (E3065 retired: now run-pass)
- **LT-23** (3): respell `accept_borrows_keeps_each_field_on_its_own_source` (drop `@borrows(left, right)`: the body's summary keeps each field on its source); rewrite `reject_borrows_cannot_stand_in_for_unknown_field_provenance` (E3065 retired: run-pass, or `from` naming one parameter with E3062); rewrite `reject_the_field_that_borrows_the_changed_source` (E3060 at the later use, not E3021)
- **LT-25** (2): keep 1; respell `reject_borrowing_an_owner_moved_into_the_same_view`: E3024 message
- **LT-34** (2): keep 1; rewrite `reject_changing_the_one_source_while_a_field_is_live`: E3060 at the later use, not E3021
- **LT-35** (3): keep 1; respell `accept_direct_method_summary_uses_only_selected_field` (`this`); rewrite `reject_mutation_before_callable_accesses_that_field` (E3060 at `sum(pair)`, not E3021)
- **LT-39** (2): keep 1; rewrite `reject_a_field_selected_at_run_time_needs_every_region`: E3060 at the later uses, not E3021
- **LT-42** (7): keep 1, respell 4, rewrite 2
  - respell `accept_owned_closure_allows_static_view_capture`: `fn() owns pair`; `StrView`
  - respell `accept_owned_closure_moves_an_array_capture`: `fn() owns data`
  - rewrite `reject_closure_capture_of_whole_aggregate`: E3060 at the later call, not E3021
  - rewrite `reject_closure_capture_unions_selected_fields`: E3060 at the later call, not E3021
  - respell `reject_owned_closure_cannot_capture_a_nonstatic_view`: `fn() owns pair`; E3063 stays
  - respell `reject_owned_closure_capture_moves_the_source`: `fn() owns data`; E3040 message
- **LT-44** (1): respell `accept_one_arena_parameter_needs_no_borrows`: `ref mut arena`, call word
- **OPT-1** (2): rewrite `accept_a_handle_that_may_outlive_its_function_stays_on_the_heap` (one-owner classes: `return self` and `Weak(w)` need `shares`/`weakref`; the C asserts re-derived); respell `accept_an_object_no_handle_outlives_is_made_in_the_frame` (`this`; `store(ref mut list, k)` with `owns k`)
- **SIMD-3** (1): respell `accept_distinct_soa_columns_loop_in_a_restrict_function`: `ref mut ps`; `xs = ref mut ps.x`
- **SIMD-9** (5): keep 1 (`run_fail_a_gather_names_its_lane`); respell `accept_a_load_at_the_induction_variable_needs_no_check_in_the_loop` (`ref mut out`, `.store(ref mut out[i..i + 8])`), `accept_checked_and_unchecked_memory_operations` (`store(ref mut ...)`, `scatter(ref mut ...)`), `reject_the_unchecked_forms_outside_unsafe` (`ref mut` before the stored span, or E3028 joins), `run_fail_a_short_store_panics` (`ref mut out`, `store(ref mut out[1..])`)
- **SOA-2** (3): respell `accept_two_columns_borrowed_at_once` (`ref mut pos`, `integrate(ref mut ps.pos, ps.vel, 0.016)`); rewrite `reject_a_length_change_while_a_column_is_borrowed` (E3060 at `println(scores.len())`, not E3022); respell `reject_writing_a_borrowed_soa` (E3023 message)
- **SOA-6** (3): rewrite `accept_element_proxies` (`p.score += 10` through a plain proxy is E3023; `load()` is E1010; `ns.get` gives a proxy; `for ref mut q in ns`); rewrite `reject_a_length_change_while_a_proxy_lives` (a proxy is a position: push keeps it, now run-pass); rewrite `reject_writing_through_an_soaref` (E3023, not E3021)
- **SPN-1** (14): keep 1, respell 5, rewrite 7, retire 1
  - respell `accept_array_and_fixed_array_coerce`: `ref mut xs`, `fill(ref mut a, 9)`
  - respell `accept_as_mut_span_writes_through`: `s = ref mut a.as_mut_span()`
  - respell `accept_returning_an_implicit_str_of_a_borrowed_parameter`: `StrView`
  - respell `accept_string_coerces_to_str`: `StrView`
  - respell `accept_string_descriptor_fields`: `StrView`; `ref mut reads`
  - rewrite `reject_a_view_returned_from_a_call`, `reject_as_span_still_borrows`, `reject_mutating_around_a_live_str_view`, `reject_mutating_around_an_implicit_str_view`, `reject_mutating_while_a_slice_lives`: E3060 at the later use, not E3021 at the change
  - rewrite `reject_writing_through_a_shared_span`: E3023 (BRW-15), not E3021
  - rewrite `reject_writing_through_a_slice_inside_its_loop`: E3023 or E3020 by DIA-28, not E3021
  - retire `run_fail_string_field_view_keeps_exclusivity`: the run-time exclusivity check is removed
- **SPN-3** (13): keep 1, respell 6, rewrite 5, retire 1
  - respell `accept_a_mutspan_converts_where_a_span_is_expected`: `m = ref mut a.as_mut_span()`; `[T: Copy]` above
  - rewrite `accept_mut_span_reborrow`: `reborrow()` gone
  - respell `accept_mut_span_split_at_reborrows`: `ref mut` names
  - respell `accept_shared_span_split_at`: `[T]` above
  - rewrite `reject_array_mutation_while_direct_reborrow_is_live`: `reborrow()` gone; E3060 at the later use
  - rewrite `reject_array_mutation_while_direct_split_is_live`: E3060 at the later use, not E3022
  - rewrite `reject_array_mutation_while_shared_split_is_live`: E3060 at the later use, not E3021
  - rewrite `reject_parent_mut_span_reborrow_while_child_is_live`: `reborrow()` gone; E3022 at a second `ref mut` name
  - respell `reject_parent_mut_span_use_while_split_live`: `ref mut` names; E3022 stays
  - retire `reject_shared_span_reborrow`: `reborrow()` gone from every span
  - respell `reject_use_after_move_of_a_mut_span`: E3040 message
  - respell `reject_writing_a_mutspan_while_its_span_lives`: `ref mut` name; E3021 stays
- **SPN-4** (2): respell `accept_named_span_iterators_implement_iterator` (`[I: Iterator]` above, `ref mut iterator`), `accept_span_iterators_and_chunks` (`[T]` above, `ref mut span`)
- **SPN-5** (11): keep 3, respell 4, rewrite 4
  - rewrite `accept_disjoint_mutable_iterator_items_can_coexist`: `a = ref mut replacement` on a pattern name is E3023 (BRW-14)
  - rewrite `accept_iterator_items_preserve_source_region`: a `ref mut` name given another ref re-points (BRW-13)
  - respell `accept_iterator_option_preserves_source_region`: `[T]` above
  - rewrite `accept_mut_span_reuse_after_iterators_end`: `=` on a loop name is E3023 (BRW-14)
  - respell `accept_mut_span_split_halves_are_disjoint`: `ref mut` names
  - respell `accept_views_of_a_view_parameters_target`: `ref mut s`, `owns h`
  - respell `reject_parent_mutation_while_shared_iterator_lives`: `ref mut` name; E3021 stays
  - rewrite `reject_parent_use_while_yielded_item_lives`: `reborrow()` gone
- **STR-1** (5): keep 4; respell `accept_an_enums_associated_function`: `static fn`
- **STR-5** (14): keep 6, respell 7, rewrite 1
  - respell `accept_a_box_field_derives_clone_recursive_types_included` (`Box[T]` -> `owns T`), `accept_a_written_debug_replaces_the_implicit_one` (`this`, `ref mut f`), `accept_implicit_clone` (`this`), `accept_implicit_debug` (`[T]` above), `accept_implicit_eq_over_a_written_eq` (`this`), `reject_a_written_eq_beside_a_part_without_eq` (`this`), `reject_implicit_clone_of_a_type_with_drop` (`fn drop()`; help `fn clone() -> Guard`)
  - rewrite `accept_tuples_options_and_results_clone`: `b = a` hands the class object over: `a.name` after it is E3040
- **STR-6** (3): respell all: `accept_an_init_assigning_fields_without_defaults`, `accept_an_init_constructor`, `reject_a_field_left_unassigned_by_init` (`fn init(...)`, `this`; E2100 message)
- **STR-7** (1): respell `accept_self_inside_another_type`: `[T]` above; `static fn`; `owns other`, `owns this`

## 2. Work items

Ordered; grouped into slices that each leave the compiler built, its tests green and nothing half
removed. Sizes: S under a day, M days, L a week or more. "Design" counts are tests the item makes pass
on its own share; most design tests also need other areas' words (`this`, `ref mut` at the call,
`owns`), so they turn green only when those slices have landed too.

### Slice A: words and codes, no change of meaning (after area 1's renames A1-1, A1-7, A1-8)

**A2-1 `from` replaces `@borrows`; rule 1 for every method; E3065 and L3014 go** (M)
- Changes: parser: `-> T from name` after the result type (`ember_parser/src/decls.rs`, where the
  signature ends; `from` is already a contextual keyword, `decls.rs:19`); `ember_ast` field on the
  function; `ember_typeck/src/lib.rs:4060-4304` (`check_borrows_attribute*`) re-keyed to `from`, with
  E2118 for a name that is both a parameter and a field and E0104 for `@borrows` (ATT-1, with the
  attributes owner); rule 1 for every method, `owns this` included (`ember_analysis/src/borrows.rs:3010`
  `elision_of`, `:2592` `legacy_elision`, `:3095` `is_method_body`); LT-22's fallback to rule 3 in place
  of E3065 (`borrows.rs:3334-3383`); L3014 removed (`ember_typeck/src/lib.rs:4048`, `:9974`,
  `ember_diag/src/codes.rs:458`); helps that named `@borrows` name `from` (LT-6).
- Completes: LT-1a, LT-1b, LT-6, LT-22, LT-23, LT-44; LT-1's rule 1.
- Tests: old LT-1a s9 w2 r3, LT-1b r4, LT-22 w2, LT-23 s1 w1, LT-1 w2 (owned receiver), LT-7 s2, LT-44
  s1, CTL-1 s1. Design: the 24 tests that write `from` (R6:9, R8:7, R7:4, R15:3, R5:1), of which the
  generator ones wait for Q1.
- Depends: A1-7 (`this`, `owns this`); Q1 for `from <non-view field>`.

**A2-2 A view that would outlive its target is E3063** (S)
- Changes: `borrows.rs:3754-3905` (`check_escapes`: E3060 and E3061 become E3063 for returned or kept
  views of locals, of a copy parameter's own storage and of temporaries), `:3862` (E3061), `stores.rs:522`
  wording; E3061 retired in the registry (area 7).
- Completes: BRW-8's E3063, LT-4's code, BCK-4's second half; the E3063 that BRW-19 (A1-18) and BRW-20
  (A5-19) rely on.
- Tests: old BRW-9 w1, LT-1 w4, LT-1a w1, LT-7 w1, LT-4 w1. Design: E3063 in 16 files (R3:1 R4:1 R6:1
  R7:6 R8:7), with A2-8 for the kept-view ones.
- Depends: nothing.

### Slice B: the view model inside one function (lands as one; about 40 old tests change with it)

**A2-3 What a name is: part names are views, re-pointing, names that keep their place, plain names change nothing** (L)
- Changes: typeck: a name bound to a part or to a view-returning call is a read-only view (a number,
  `bool` or payloadless variant is copied), `ref`, `ref mut`, `copy of` (A1-5) before a part; `=` on a
  view name re-points it and `ref mut` names re-point with `r = ref mut b` (removes ADR-010's
  write-through, `ember_typeck/src/lib.rs:44138-44260`, `reject_repointing_self` `:44249`); `const ref
  mut`; E3023 for `=` on parameters, loop, pattern and opened names (extends `reject_borrowed_parameter_write`
  `:44291`) and for a change through a plain name of a `Cell`, a `shares` target, `MutSpan` elements
  (replaces the E3021 of `:44160-44248`); CTL-11's scope rule for opened and kind-test names.
  Lowering (`ember_mir/src/lower.rs:1388` `lower_assign`) writes the reference local itself on re-point.
  `regions.rs`: the forward value-state already separates a replaced view field's old and new values
  (LT-21, `regions.rs:34-40`); extend it to whole view locals so a re-point kills the old target.
  `drops.rs:1017-1022`: E3011 no longer fires for a part name, which is now a view (with A1-3, which
  owns `=` from a whole name).
- Completes: BRW-12, BRW-13, BRW-14, BRW-15 (the cases a body shows), CTL-11 (with its owners), SPN-1.
- Tests: old BRW-1 w3, BRW-8 w1, SPN-1 w2, SPN-5 w3, SOA-6 w1, plus respells. Design: BRW-12 39, BRW-14
  23, BRW-15 42 (the method part of BRW-15 waits for A2-10), the 13 re-pointing tests.
- Depends: A1-3 and A1-5 (hand-over by `=` of a whole name, `copy of`), A1-8 (`ref mut` call words,
  `const ref mut`), area 3 for the `shares` case, GRM-44 and A1-20 for CTL-11.

**A2-4 The view checker: positions and addresses, BCK-3's conflicts, index proofs, links** (L)
- Changes: `borrows.rs`: the `Loan` (`:54-80`) gets a kind (position, address) and a "may reshape" access
  separate from "write"; `check_point` (`:4564-4950`) follows BCK-3: a change of what a live read-only
  view reads is E3021 at the change, a reshape ends an address and its later use is E3060, a push keeps
  positions and ends addresses of inline elements, a `ref mut` view refuses reads (E3021) and changes
  (E3022) through other names; `overlaps` (`:5143`) adds index proofs (constants, `for j in i + 1..`,
  under `if i != j:`; Q2), SoA columns and split halves as disjoint, and BCK-3's link rule (two paths of
  one type through a link or kept view may be one unless different types, a fresh object or under
  `if a is not b:`); class objects in their own block survive a push of the list that owns them; the
  messages of R18 (each names the line to write; DIA-28 picks the one whose fix compiles). E3025 and the
  E3080 classification go (`:4702`, `:4810-4837`).
- Completes: BRW-1 (in one body), BRW-2, BRW-4 (with A2-11), BRW-5, BRW-16 (check), BRW-17 (in one body),
  LT-5, LT-18, LT-25, LT-26 (with area 3), LT-34, LT-36, LT-39; BCK-1 to BCK-4 and BCK-6.
- Tests: the 28 "E3060 at the later use" rewrites (BRW-1 w2, BRW-2 w1, BRW-5 w2, BRW-10 w1, CTL-1 w2,
  LT-1 w2, LT-7 w2, LT-18 w1, LT-23 w1, LT-34 w1, LT-35 w1, LT-39 w1, LT-42 w2, SPN-1 w5, SPN-3 w3, SOA-2
  w1), LT-1's E3025 iterator case and BRW-10's rule-1 case (with A2-1), and the E3021/E3022 message
  respells. Design: BRW-1 34, BRW-17 49, BRW-16 84 and
  the 8 index-proof tests, for the cases that need no call summary (about half; the rest need A2-10).
- Depends: A2-3; area 3's link kinds for the link rule; A2-5 in the same slice.

**A2-5 Positions at run time** (M)
- Changes: a position lowers to the list place and its index, not an element pointer: each use re-reads
  the list's buffer pointer where an add may have intervened, and uses the pointer directly where none
  can (`ember_mir/src/lower.rs`, index lowering); no bounds check again (the list only grew); C and LLVM
  emit nothing new beyond the reload; the evaluator (`ember_interp`) does the same; `range_facts.rs` and
  `list_locals.rs`/`reserve_pushes.rs` keep their facts across a reload.
- Completes: BRW-16 at run time, SOA-6's proxy as a position.
- Tests: old BRW-5 w1 and SOA-6 w1 (become run-pass). Design: the 16 R8 position tests and the R12 ones.
- Depends: A2-4.

### Slice C: loops, spans and kept views

**A2-6 Loops: the loop's handle, the walk rule through the collection's own name, counting loops, generators** (M)
- Changes: typeck `check_for` (`ember_typeck/src/lib.rs:28854`) and `check_for_iterator` (`:34341`):
  `for ref mut x in xs`, `for k, ref mut v in m`, `for k, v in m` for maps; a plain loop name changes
  nothing (E3023, help `for ref mut`); lowering gives the `ref mut` handle the element pointer (the walk
  cannot reshape the list, so no position is needed). `borrows.rs:4790-4848`: E3020 for any change of
  the walked collection, element writes and map value writes included, with the two-line messages; a
  counting loop whose end read `xs.len()` refuses removal of `xs` (a new fact on the loop, beside
  `for_iterator` in `ember_hir/src/lib.rs:103`); a generator result walks what it views
  (`ember_analysis/src/coroutines.rs` regions, E3020 while it will still be used, E3021 for other
  changes unless `from` narrows).
- Completes: CTL-1, CTL-2, SPN-4, BRW-18 through the collection's own name, LT-1's generator sentence.
- Tests: old CTL-1 s1, CTL-2 s2, CTL-3b s5 (with Q3), BRW-10 s1. Design: BRW-18/CTL-2 78 for the direct
  cases (R8:23 mostly), the 68 tests using `for ref mut`.
- Depends: A2-3, A2-4; Q3.

**A2-7 Spans, SoA columns and proxies, SIMD stores** (M)
- Changes: a `MutSpan` is written only through a `ref mut` name or parameter (E3023), `ref mut` before
  a span made in the call; `reborrow()` removed (`ember_typeck/src/lib.rs:55599` and callers); SoA
  columns are `MutSpan` views written through `ref mut` names, E3027 where `ps` is reached through a
  read-only name, a length change is a reshape of all of `ps` (A2-4's E3060, A2-6's E3020); proxies:
  `ps[i]` read-only, `ref mut ps[i]` an `SoAMut`, `ps.get(i) -> Option[SoARef[T]]`, `copy of ps[i]` by
  `load()`, `load()` at a use E1010 (`lib.rs:15109-15118`, `:29551`, `:34412`, `:50775-50833`); SIMD
  `store`/`scatter` take `ref mut s` (`std/src/simd.em`, typeck builtins).
- Completes: SPN-3, SPN-5, SOA-2, SOA-6, SIMD-9.
- Tests: old SPN-3 w2 r1, SPN-5 w1, SOA-2 s2, SOA-6 w1, SIMD-9 s4, SIMD-3 s1, BRW-5 s3, SPN-1 s2.
  Design: R15 1 (no SoA tests in the design set: SoA needs new tests).
- Depends: A2-3, A2-4, A2-6; A1-8 (E3028).

**A2-8 Kept views: `sharedref mut`, `ref mut` fields alone, class view fields with `from`, values that hold views** (M)
- Changes: `sharedref mut T` type and value word (parser, `ember_types`, typeck; E0114); a second holder
  of a `ref mut` field is E3022; a class's view field needs `from <let field>` or `from static` (E2118
  otherwise; E3063 for a `from static` field given other text), reusing A2-1's `from`; a value holding
  views (a list of views, a struct a call made) lives no longer than its targets (E3063 through A2-2);
  inside a callee only an argument's view or static text may be put into a `ref mut` view parameter
  (E3063); `stores.rs`, `regions.rs:2577` (`view_region_paths`).
- Completes: LT-42, LT-45, LT-46, LT-47 (callee side), LT-26 (with area 3).
- Tests: old LT-42 s4. Design: LT-45 42 (R7:31), the 22 `sharedref mut` tests, LT-46 1.
- Depends: A2-1, A2-4; area 3 (`shares`, `weakref`, class objects with one owner).

### Slice D: calls checked by summaries (with area 4's summaries and the run-time checks' removal)

**A2-9 The borrow facts in each function's summary** (L)
- Changes: on top of area 4's summary (what a function changes, reads and ends, with the chain): per
  view parameter (`this` included) the places whose end or move precedes a later use of it, with the
  lines; per `ref mut` parameter the places it reaches by another name while the parameter is used;
  per list parameter whether the function only adds to it (keeps positions); which fields of `this` a
  method changes and reads (generalises `receiver_fields`, `borrows.rs:2079-2130`, and
  `borrows_used_fields`, `ember_typeck/src/lib.rs:25125`, to every method, `pub` and virtual included);
  where a stored function is the one at fault. Least fixed point over the whole call graph, function
  values and `dyn`/virtual calls by every target (shared decision D3), installed like the region
  contracts (`borrows.rs:112-140`, `install_callable_regions_kept`) and kept in the interface record.
- Completes: BCK-8's borrow half; BRW-10's facts; BRW-21's facts; LT-35's targets.
- Tests: none on its own. Depends: area 4 (EFF-1 summary, EFF-4 interface records, function-value
  targets), A1-21 (a `drop` counts as a call).

**A2-10 Calls checked against their callees' summaries** (L)
- Changes: `borrows.rs:4481` (`check_call_activation`) and the call point of `check_point`: E3031 for an
  argument pointing into what the callee may end or move before its last use of that parameter, and for
  a `ref mut` argument into what the callee reaches by another name, the refusal of a stored function at
  its storing line; BRW-11 (two arguments, one changeable, a function value counting its targets'
  changes; proofs from A2-4 and A2-12; E3021 for two spans of maybe-one lists); BRW-18 through calls
  (function values, library calls given functions, `spawn`, foreign callbacks, a `drop`); BRW-17's
  reshape through calls; BRW-1 (f)'s check of a `ref mut` parameter against module variables; BRW-15's
  method that may change its object through a plain name (E3023); LT-47 on the caller's side; messages
  name the chain ("through `click`, which runs the handler stored on line 9"). E3031 enters the
  registry (area 7).
- Completes: BRW-1, BRW-10, BRW-11 (minus its exceptions), BRW-15, BRW-17, BRW-18, BRW-21, LT-35, LT-47.
- Tests: old BRW-4 w1, BRW-10 w2, LT-1 w1 (E3025 cases). Design: BRW-21 15 (E3031 in 10 files), BRW-11
  19, the through-call share of BRW-18 (R12:28, R5:6, R15:7, R14:3), BRW-17 (R10:10, R12:9), BRW-16's
  "a call that reads and removes" tests.
- Depends: A2-9, A2-4, A2-6; A1-8.

**A2-11 The run-time access and RefCell machinery leaves the analyses** (M)
- Changes: delete `insert_shared_accesses_all` and its helpers (`borrows.rs:320-2017`), the
  `RuntimeGuard` loans (`:4605-4630`, `:5350-5380`), `check_refcell_call` and L3011 (`:3714-3725`,
  `:3924-4075`), `access.rs`, `loop_access.rs`, `long_access_lint.rs` and their driver passes
  (`ember_driver/src/lib.rs:5350`, `:5410-5418`, `:5424`), the `BeginAccess`/`EndAccess` statements
  (`ember_mir/src/lib.rs:1274-1300`) once no producer remains; the evaluator's access checks
  (`ember_interp/src/object.rs`) and the runtime's (`runtime/ember_rt/src/ember_rt.c:9165-9341`) go with
  area 4's EXC slice.
- Completes: the four "removed" rows; with area 4, EXC's removed rules.
- Tests: old SPN-1 r1; the EXC and CELL folders are areas 3 and 4's.
- Depends: A2-4 and A2-10 must be in (the compile-time checks that replace them); area 3 (RefCell gone),
  area 4 (EXC slice), A9-5 (branch deletions).

### Slice E: alias facts

**A2-12 `check_disjoint`, `assert_disjoint`, `assume_disjoint`: facts on names** (M)
- Changes: `std/src/mem.em:72-92` (new signatures: read both, `Result[void, void]` with `@must_use`, or
  nothing); typeck `synth_assert_disjoint` (`ember_typeck/src/lib.rs:16250-16330`) and the E3095 checks
  (`:16276`, `:16395`); the fact keyed to the two names (after `assert_disjoint`, in `check_disjoint`'s
  `Ok` arm), dropped on re-point, read by A2-10's BRW-11 proof and by `kernels.rs:777-899` (`origin`,
  which today follows the returned tuple); DSJ-10 (making views and comparing reads no element; an
  element read before the proof is E3021); the stop text naming the two names; `RuntimeCheck(Aliasing)`
  sites and `elide_known_disjoint_comparisons_all` (`kernels.rs:748`) unchanged; E4030 inside
  `@no_alias_checks` (the attribute is area 4's, `static_safe.rs:68` today).
- Completes: DSJ-1, DSJ-2, DSJ-3, DSJ-5, DSJ-6, DSJ-7, DSJ-9, DSJ-10.
- Tests: old DSJ-1 s1, DSJ-3 s1, DSJ-5 s2, DSJ-6 w1, DSJ-7 s2, DSJ-9 s1 (and DSJ-4 s2). Design: 7 (R4:6
  R16:1), the 5 `@no_alias_checks` tests in R16 with area 4.
- Depends: A2-3 (re-point), A2-10.

**A2-13 One whole variable under two names: the proof and the exemption** (S)
- Changes: in A2-10's argument check, prove when two `ref mut` arguments, or a `ref mut` argument and a
  module variable the callee's summary names, are one whole variable (the same local or static, no
  projection; parts with unknown positions stay E3022), mark the call for area 1's copy (A1-22, shared
  decision D5 of area 1) instead of refusing it, and give the copy one name for the merged place so A2-14
  emits no `restrict` between them.
- Completes: BRW-11 exception 1, BRW-22's copy clause (with A1-22: MONO-10).
- Tests: old BRW-11 w1 (run-pass). Design: R4's 2 runs (`add_both`, `rename(ref mut LOG)`).
- Depends: A2-10, A1-22.

**A2-14 noalias on every `ref mut` parameter; alias facts the passes read** (M)
- Changes: C: a `ref mut` parameter is `T *restrict` (`ember_codegen_c/src/lib.rs:6521-6560`
  `signature`, `:12218`); a `ref mut` `MutSpan` parameter also gets its element pointer hoisted into a
  `T *restrict` local, reloaded after a re-point; a `const ref mut` `MutSpan` may be passed split
  (pointer, length) as kernels are (D5 below); never on `this` or `sharedref mut`. LLVM:
  `noalias` on the parameter (`ember_codegen_llvm/src/func.rs:78-110`; `abi::Ext::NoAlias`
  `abi.rs:137-150`; `Ext::Ref` if helper branch AI lands, A9-10); scoped metadata for hoisted element
  pointers. Re-justify the passes that leaned on run-time checks: `loop_version.rs:25-30` (invariance
  through a reference parameter "panics before anything is changed"), `kernels.rs:1-41` and `origin`
  (`:777-899`), `check_hoisting.rs`, `uncounted_handles.rs`, with SIMD-3's new (e) and its link rule.
- Completes: BRW-22, LT-27, SIMD-3, CG-C-4, CG-LL-3's `noalias`.
- Tests: new `assert-c` tests (`restrict` on a `ref mut` parameter, none on `this`, none in a one-name
  copy); the LLVM test named in area 9's I9. Design: none assert C.
- Depends: A2-10 and A2-13 **must** be in first: `restrict` is sound only when every call is checked.
  A2-12 for (d).

### Slice F: after area 1 and area 3's class work

**A2-15 Frame objects for one-owner classes** (S)
- Changes: `ember_analysis/src/frame_objects.rs`: escape by hand-over, `shares` and `weakref` links (a
  `weakref` made escapes) instead of counted handle copies; views cannot escape (BCK-4).
- Completes: OPT-1. Tests: old OPT-1 s1 w1. Depends: A1-9, area 3.

Not separate items: BRW-19 is area 1's A1-18 and A1-19 on A2-4's view liveness and A2-2's E3063;
BRW-20 is area 5's A5-19 on A2-2; BRW-11's exception 2 is area 5's A5-16; STR-1, STR-3, STR-5, STR-6 are
area 1's A1-6, A1-9, A1-5, A1-7 (STR-7 is the grammar area's `[T]`).

| slice | items | must land together because |
|---|---|---|
| A | A2-1, A2-2 | words and codes only |
| B | A2-3, A2-4, A2-5 | the checker accepts a push beside a position only if codegen re-finds it; re-pointing changes run-time meaning and region kills at once |
| C | A2-6, A2-7, A2-8 | each alone is consistent; order as listed |
| D | A2-9, A2-10, A2-11 | the run-time checks may go only when the call checks exist |
| E | A2-12, A2-13, A2-14 | `restrict` needs the call checks, the one-name copies and the names' disjointness facts |
| F | A2-15 | waits for area 1 and area 3 |

## 3. Shared decisions

**D1. What the borrow half of a summary holds, and where it lives.** Proposal: area 4's per-function
summary (changes, reads, ends, each with a chain) gains, in the same record:
`view_params: [(param, [(place, first_line)])]` (places whose end or move precedes a later use of that
parameter), `ref_mut_reach: [(param, [place])]` (places read or changed by another name while the
parameter is used), `list_touch: [(param, AddsOnly or Reshapes)]`, `this_fields: (changed, read)`, and
the stored-function fault lines. A *place* is a root (parameter i, `this`, a module variable, or "an
object of type T reached through a link") plus a field path and an element mark. Lives on `Body` next
to `CallableRegionMetadata`, installed by one whole-program fixpoint, serialised in the interface record
(`ember_build/src/interface.rs`, schema per area 9's K1). Why: every one of BRW-10, -11, -17, -18, -21
and LT-35 is a lookup into it; one record avoids five passes over the call graph. Cost against C:
nothing at run time; compile time is one more fixpoint over the call graph, the same shape as
`loop_version.rs:834-905` (`Summaries`).

**D2. Positions are (list place, index).** A view's loan carries a kind, position or address. A
position lowers to its list place and index; a use after a possible add reloads the buffer pointer, a
use with no add between uses the pointer it has. Why: BRW-16 says the element is found again at each
use; a pointer would dangle after a reallocating push. Cost against C: zero where no add intervenes;
one load per use after an add, which is what correct C does (an index kept across a `realloc`).

**D3. A function value's possible targets** (with area 4, whose decision it is). Proposal: by the
function type including its parameter words, `thread_shared` and result, the set of named functions and
lambdas converted to that type anywhere in the program, libraries included through their interface
records; a `dyn`/interface/virtual call by every implementation or override of that slot. Why: BRW-11,
BRW-18 and BRW-21 all say "every function the program can put there"; the type is the only whole-program
key that needs no flow analysis, and area 9's J6 says AJ's index makes it exact. Cost: none at run time;
imprecision shows as refusals, never as unsoundness.

**D4. Index proofs are a closed syntactic list** (pending Q2): both indexes constants (literals or
`const` items) and different; `j` the counter of `for j in i + 1..` with `i` unchanged in the loop; the
use inside `if i != j:` (or its `else` of `if i == j:`) with neither reassigned. Why: XVIII.4 says every
implementation accepts the same programs (`[PHIL-13]`); a proof from range facts would make acceptance
depend on the optimiser. Cost: none at run time (verdict 98 removes the compare).

**D5. noalias ABI.** A `ref mut` parameter is passed as `T *restrict`; for a `ref mut` `MutSpan` the
place is passed by address and the callee hoists `elem *restrict p = dst->ptr` (reloaded after a
re-point); a `const ref mut` `MutSpan` never re-points, so it may be passed split as
`(elem *restrict, size_t)` (BRW-8 allows the choice by the first line). LLVM: `noalias` on the pointer
parameter; the hoisted element pointer gets a scope from `llvm.experimental.noalias.scope.decl`. Cost
against C: one load per call for a re-pointable span, zero for `const ref mut`; the gain is the one run 3
measured for the call compiled with noalias, 1.00x to 1.07x of plain C against 1.53x without
(`run4/v5/rules.md` section 2, row 2).

**D6. Disjointness facts are pairs of names with a validity range** (the points after `assert_disjoint`,
the `Ok` arm of `check_disjoint`), killed by a re-point of either name, read by BRW-11's check and by
`kernels.rs`'s `origin`. Cost: two compares where the program writes the call (DSJ), nothing else.

## 4. Questions for the owner

**Q1. May `from` name a field of `this` that is not a view?** `[LT-1a]`: "A bare field name names a
field of `this` whose type is a view ... Naming a field when there is no `this`, or a field of neither
kind, is `E2031`." `[LT-1]`: "any other change of `d` ... is ... `E3021`, unless `-> Generator[int] from
log` narrows the view to `d.log`", where `log` is an `Array[String]`, and seven design tests write it
(`R8/accept_another_field_changes_while_a_from_log_generator_is_used.em`, `R6/...`, `R15/...`). Under
LT-1a's text those are E2031; under LT-1's they narrow the result to that field and what it owns. It
blocks A2-1's E2031 check and A2-6's generator rule.

**Q2. Is the list of index proofs closed?** `[BRW-5]`: "unless the compiler proves `i` and `j` differ:
both constants and different, `j` from `i + 1..`, or the line inside `if i != j:`" (the same list in
`[BRW-11]` and `[BCK-3]`). Is `i = 0`, `j = 1` (locals never changed) "constants"? Does `if i == j:
return` above the line count, or `while i != j:`? XVIII.4 says every implementation must accept the same
programs, which suggests a closed syntactic list (D4); the words "the compiler proves" suggest any proof.
It decides whether the old test `BRW-5/reject_two_variable_indices` stays E3022 and whether `range_facts`
feeds the checker.

**Q3. How does a loop change elements through an adapter?** `[CTL-3b]` MUST compile `enumerate`, `zip`,
`rev`, `chain`, `take`, `skip`, `step_by` over a `MutSpan` or `Array` to counted loops; `[CTL-1]` gives
only `for ref mut x in e:` and says `for v in xs.iter_mut(): v += 1` is `E3023`; `[SPN-4]` no longer
mentions `iter_mut`. Is `for i, ref mut x in xs.iter_mut().enumerate():` the spelling (so `iter_mut()`
stays in std), or `for i, ref mut x in xs.enumerate():`, or is changing through an adapter not
allowed? Seven old CTL-3b tests do it; the CTL-3b MUST needs the answer.

**Q4. Does a plain parameter's promise cover a number written through another name?** `[BRW-1]`: while
a read-only view, "a plain parameter" included, "is still used, what it points into is not changed
through any other name (`E3021`, at the change)". `[BRW-21]`: the call check refuses only what "may end or
move" a view parameter, and "Writing a number or a `bool` ends and moves nothing, so a method that
writes `this.health` through a link that may lead to its own object (a turret aimed at itself) is
allowed". Inside a callee, a plain parameter may therefore see its target's numbers change through a
module variable or a link, which BRW-1 says cannot happen. Which holds: is BRW-1's promise for a
parameter limited to "not ended or moved" (and E3021 only for changes in the caller's own body), or must
the call check also refuse a callee that changes, by another name, what a plain parameter reads while it
is used? It decides A2-10's check and whether a plain parameter's elements may be kept in registers
across such writes (SIMD-3 says a plain parameter proves nothing, which fits the first reading).

## 5. Risks

* **The interprocedural core is all new.** A2-9 and A2-10 have no precursor beyond region contracts and
  BRW-10's field sets; their precision (by-type function-value targets, link paths by type) decides how
  many of the ~150 design tests that go through calls are accepted as the design expects. Size unknown
  until area 4's summary exists; I rate A2-9 and A2-10 L each with low confidence.
* **`restrict` turns any hole in the call checks into a miscompile.** Today an unproved alias costs
  speed; after A2-14 it is undefined behaviour in C. A2-14 must stay behind A2-10 and A2-13, and needs a
  test set that tries to break BRW-21 (stored functions, virtual calls, foreign callbacks, generators,
  `drop`s) before it lands.
* **Re-pointing reverses a design assumption.** ADR-010 ("a reference local is not re-seated, which is
  what lets regions be location-insensitive", `ember_typeck/src/lib.rs:44143`) is cited in typeck,
  `regions.rs:34` and `coroutines.rs:299`. `regions.rs` already tracks replaced view fields point by point
  (LT-21), so I expect M, not L, but the generator regions (`coroutines.rs`) are untested against it.
* **Reporting at the later use changes most messages.** 33 old tests move their error from the change to
  the later use; R18's messages compute the line to write and DIA-28 picks among two errors. The
  diagnostic share of A2-4 and A2-10 is large and unmeasured.
* **Positions touch the list optimisations.** `list_locals.rs`, `reserve_pushes.rs`, `stack_lists.rs`
  and `range_facts.rs` reason about a list's buffer; a position's reload must keep their facts valid.
* **Passes that leaned on run-time checks.** `loop_version.rs:25-30` justifies invariance by the
  run-time access check panicking first; `uncounted_handles.rs` and `check_hoisting.rs` touch access
  statements. Each must be re-argued with compile-time facts (A2-14) or it becomes unsound when A2-11
  removes the checks.
* **The evaluator.** `ember_interp` must implement positions, re-pointing and one-name copies the same
  way; I did not read its reference model.
* **Not verified:** that the old tests behave as I classified them (nothing was run); the exact C
  emitted for `ref mut` parameters beyond `c_type` (`lib.rs:12218`); `loop_version.rs` beyond its header
  and `Summaries`; the std's own uses of `@borrows`, `reborrow` and `iter_mut` (the std area's).
