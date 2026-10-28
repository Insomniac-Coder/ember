# Gap map 07: syntax, types and diagnostics

Area 07 of the 0.9.15 gap map (brief: `tasks/96/redesign/compiler-0.9.15/inputs/brief.md`). Families:
GRM, LEX, TYP, TXT, DIA, PHIL, COST, PRF, TST, and the small ones no other area takes (RT, MOD but
MOD-9, MNG, DET, CLI, CG-C, CG-LL, SEL, IDE, HASH, BLD but BLD-14, ATT, SER, RFL, LNT, IMP, FMT, DRV,
DOC, CT, BEN). `BLD-FFI-*` is read as the FFI area's family and is not here.

Read-only work: nothing was built or run except `python -I tools/rule_index.py` (it only reads; it
reported the six `[COST-5]` lines quoted below) and small read-only scripts kept in
`target/gap-0.9.15/07/` (`rules.py` word diffs of every rule, `codes_check.py` the `codes.rs` registry
test re-done in Python against the pinned target, `classify.py`/`final.py` the old-test sort,
`p17a_diff.txt` the code table old against new).

## 0. The picture in five lines

- The compiler is 0.9.10 throughout. Of the 0.9.15 words, only `cast x as D` exists
  (`ember_parser/src/body.rs:1047`); `owns`, `shares`, `thread_shared`, `weakref`, `sharedref`, `this`,
  `using`, `exists as`, `is T as`, `copy of`, capture words, `from`, `guarded by`, type-parameter lines,
  `static fn`, `StrView` appear nowhere in `compiler/` (grep of every `.rs`).
- The pin already points at 0.9.15 (`docs/spec-source/development-target.json`, commit `ab2b7b76`), so
  the spec-tied tests are red now. Milestone 0 (section 2, A7-1 to A7-3) makes the ones that only read
  tables green without any language change; the version test, Annex A and the spec examples follow the
  grammar and the rest of the model.
- The registry (`compiler/ember_diag/src/codes.rs`) lacks 34 codes the target names and cites the wrong
  rule for 9 (E2173, E3023, E3063, E3095, E3060, E3062, E5060, E7004 twice); 59 rows changed meaning,
  19 codes were retired (section 7).
- `ember inspect --cost` prints six labels with no row in the new table: four are renames, two
  (`Dynamic exclusivity check`, `` `RefCell` borrow ``) leave only with the run-time exclusivity checks
  and `RefCell` themselves (other areas).
- The parser change is one slice with a migrator: every removed spelling is refused with a
  machine-applicable migration (`[CLI-20]` requires exactly this tool), and the same slice runs it over
  `std/` (9,081 lines; 2,242 `self`, 224 `owned`, 150 generic `fn f[T]`), the tests, `docs/book/` and
  `docs/errors/`, so no slice carries two grammars.

## 1. Rule table

128 rules of these families are in H.41 (all `changed` or `added`; none `removed`), plus three rows
whose rule text did not change but whose error code's row did (`GRM-2`/`E0100`, `MOD-3`/`E1010`,
`TYP-4`/`E2020`; they carry design tests). Item ids point to section 2. Old-test cells give counts;
the file names of every non-keep test are in section 6. Design-test cells count the 841 tests whose
`#$ rules:` names the rule; nearly all 841 also need this area's grammar (section 2, A7-6/A7-7).

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| `[ATT-1]` | changed | an attribute Ember no longer has (`@sync`, `@static_safe`, `@fp(contract)`, `@borrows`, `@no_runtime_checks`) is `E0104` naming the one to write | `ember_typeck/src/lib.rs:60305-60336` still admits `@static_safe`, `@sync`, `@borrows`, `@fp(contract)`; `@no_runtime_checks` is in `RESERVED_ATTRIBUTES` (`:60367`), refused as reserved | not built | drop the four rows from the attribute table; add a gone-attribute list whose `E0104` gives the replacement text of the rule; move `@no_runtime_checks` there (A7-15) | keep 6, respell 1 (§6) | 5 (R3 1, R6 1, R15 1, R16 2) |
| `[ATT-2]` | changed | the attribute table: `@no_alias_checks`, `@fp(allow_fma)`, `@unsafe(export(...))` (plain `@export` is `E3100`), `@view` checked, `@cached`; no `@sync`, `@borrows` | table at `ember_typeck/src/lib.rs:60305-60367` is 0.9.10's (`@export` plain at `:60320`) | partly | rename/replace the rows; `@cached` row with the CLS area; plain `@export` refused `E3100` with the FFI area (A7-15) | no folder | 0 |
| `[BEN-6]` | changed | thresholds: `shares` code ≤ 1.15x an intrusive count, atomic for `thread_shared` | bench thresholds name counted objects and `@sync` (`tests/perf`) | text only | rename the benchmark rows when the `shares`/`thread_shared` benchmarks exist (area of RC) (A7-22) | no folder | 0 |
| `[BLD-2]` | changed | a module's cache key covers the facts of `[BLD-14]` callers' checks rely on | the key covers signatures, layouts, effect sets, inline bodies and borrow summaries (`ember_build`) | partly | key the cache on the BLD-14 interface facts once that area defines them (A7-22) | no folder | 0 |
| `[CG-C-4]` | changed | every `ref mut` parameter (a `MutSpan` one included) is a `restrict` pointer, never `this`; the `[MONO-10]` copy passes one pointer | `restrict` only on bodies marked `restrict_views` (`ember_codegen_c/src/lib.rs:434-436`, `ember_mir/src/lib.rs:192`) | partly | a MIR parameter flag `exclusive` from the `ref mut` word, emitted as `restrict`; sound only with BCK-8's call proof (A7-19) | keep 1, respell 1 (§6) | 0 |
| `[CG-C-9]` | changed | debugger visualisers show class objects and their links (`owns`, `shares`, `weakref`) | natvis is generated and linked (`ember_build/src/lib.rs:1024`) for 0.9.10's handles, `Box`, `Shared` | partly | new entries for the 0.9.15 header and the link kinds (A7-20) | no folder | 0 |
| `[CG-C-11]` | changed | `@fp(allow_fma)` functions go in their own unit | done for `@fp(contract)` (`ember_typeck/src/lib.rs:2004`, `:2081`) | partly | rename the attribute; `@fp(contract)` becomes `E0104` (A7-8) | no folder | 0 |
| `[CG-LL-3]` | changed | `noalias` on every `ref mut` parameter (a `MutSpan` one included), never `this` | `noalias` only on `restrict_views` spans and `sret` (`ember_codegen_llvm/src/func.rs:98-101`, `:214`) | partly | as CG-C-4, through `abi::Ext::NoAlias` (A7-19) | no folder | 0 |
| `[CLI-2]` | changed | `--emit c` writes `<dir>/cflags.txt` (the stack-probe flag among them) | no `cflags.txt` anywhere in `compiler/` | not built | write the flags of `gnu_flags`/`msvc_flags` for the target beside the sources (A7-18) | no folder | 0 |
| `[CLI-18]` | changed | `explain --cycle` and the build analyse one graph of owners: each edge an owner edge or not, the shortest loop | `explain_cycle` (`ember_driver/src/lib.rs:1224`) classifies strong, weak, unknown | partly | read the WK area's owner graph (`E3032`) (A7-22) | keep 0, rewrite 1 (§6) | 0 |
| `[CLI-20]` | changed | `fmt --migrate` rewrites 0.9.10 source (Box, Weak, owned, mut params, self, str, clone, if Some, cast if, as?/as!, @borrows, @static_safe, @fp(contract), assert_disjoint); lists what needs a choice | `migrate_file` (`ember_driver/src/lib.rs:3210-3290`) knows 0.9.8 → 0.9.9 only; help text says so (`:73`) | not built | every M1 refusal carries a migration; run the checker under migratable parse errors; edit call sites in other files (A7-9) | no folder | 0 |
| `[COST-3]` | changed | the cost table; what runs is only what it lists | `ember inspect --cost` prints six labels with no row: `check_cost_row` (`ember_driver/src/lib.rs:1557-1569`), `:2673`, `:2716`; `python tools/rule_index.py` reports them | partly | rename four labels; the other two go with the run-time exclusivity checks and `RefCell`; report the new rows (owner count, `weakref` link, kind test, function value, lock, stack probe) as their areas land (A7-2) | no folder | 0 |
| `[COST-6]` | added | the stop texts of the run-time checks, in the program's names | bounds text lacks the `.get(i)` hint (`runtime/ember_rt/templates/ember_rt.c.in:9134`); downcast says `invalid downcast` (`ember_codegen_c/src/lib.rs:8718`); counts say `reference count overflow on %s` (`ember_rt.c.in:3037`) | not built | per-site static descriptors passed on the cold path; texts of the table (A7-16) | no folder | 0 |
| `[CT-5]` | changed | `comptime(e)` of an owning type is constant data read through a read-only view (a change is `E3023`); `copy of comptime(e)` copies; `static` vs `static mut` placement | each use materialises a fresh value (`ember_typeck/src/const_eval.rs:115`), the 0.9.10 clone | not built | materialise once as constant data, type the use as a read-only view; `static mut` initialises a heap value (A7-23) | keep 1, rewrite 1 (§6) | 0 |
| `[DET-2]` | changed | an object's address as an integer is `Nondet`; a spawn-order reduction of scoped tasks is not | address case at `ember_analysis/src/effects.rs:992` | partly | reductions with the THR area (`[THR-21]`) (A7-22) | keep 3 | 0 |
| `[DET-3]` | changed | a call through a function value counts every function the program can put there; a type-parameter function per instance | `effects.rs:1130-1141`: plain `fn` values by shape; any other function value is all effects | partly | use the function-value target sets (EFF/CLO area) (A7-22) | no folder | 0 |
| `[DET-5]` | changed | `@fp(allow_fma)` inside `@deterministic` is `E4072` with its text; `@fp(contract)` is `E0104` | `E4072` for `@fp(contract)` (`effects.rs:1328`) | partly | rename; message texts (A7-8) | keep 2 | 0 |
| `[DIA-2]` | changed | every refusal names what was refused in the program's names (`int` not `i64`, `fn()`), why in one line, and the exact line to write | only titles are style-checked (`ember_diag/src/codes.rs:511`); messages are per site | partly | type printer keeps the written spelling; the fix API of A7-14; one review per emitting area (A7-14) | no folder | 0 |
| `[DIA-3]` | changed | ownership and view errors label the later use (`used here`) and give their shape's fix | labels exist as `later used here` in the borrow checker | partly | label text; shape fixes of the new table (A7-14) | no folder | 0 |
| `[DIA-7]` | changed | every ownership and view error is classified into a 0.9.15 shape | `ember_diag/src/shapes.rs:135-161` maps to 0.9.10's shapes (O3, O4, O5, B8, B14, B15, X1) | partly | new shapes B16, B17, N15-N18; `E3028`→B10, `E3031`→B16, `E3032`→B17; old shapes go when their codes stop being emitted (A7-1) | no folder | 0 |
| `[DIA-8]` | changed | `ember explain --view <file>:<line>`: each view in use, where made, what it points into, the use that keeps it | `--borrow` (`ember_driver/src/lib.rs:618`) with loan/region wording | partly | rename the flag and the wording; read the view facts of the BRW area (A7-2) | no folder | 0 |
| `[DIA-9]` | changed | never suggest `unsafe` when a fix without it compiles; never `shares`/`shares mut` alone where a one-owner fix compiles | forbidden suggestions are `unsafe`, `Cell`, `RefCell`, `Shared`, `clone()` (help policy in the borrow checker; `not-help` tests) | partly | new policy; verified by A7-14's fix-compiles check (A7-14) | no folder | 0 |
| `[DIA-10]` | changed | help names the concrete construct as the line to write (`for ref mut x in xs:`, `weakref`, `owns n`) | helps name `Weak`, `owned fn` | partly | rewrite the help texts per shape (A7-14) | no folder | 0 |
| `[DIA-11]` | changed | shape S1 names the `assert_disjoint` call and the lines that need no check, or removing `@no_alias_checks` | `E4030` reports why a dynamic check was kept (`[EFF-11]` reasons) | not built | new meaning with `E4030`'s new row (DSJ area) (A7-14) | keep 0, respell 1, rewrite 1, retire 2 (§6) | 0 |
| `[DIA-16]` | changed | a suggestion of `shares`/`shares mut` notes its run-time cost (a count per owner, a compare per +1) | a note names the cost of `Shared`/`RefCell` | partly | rewrite the note (A7-14) | no folder | 0 |
| `[DIA-21]` | changed | Python habits P14 (`self` → `this`), P18, P22 (`owns dyn Shape`), P26/P27 (`StrView`, `String(name)`) | P14 says `add self` (`ember_parser/src/python.rs:286`); habits table `ember_diag/src/habits.rs` | partly | reverse P14; new texts for P18, P22, P26, P27 (A7-15) | keep 2 | 0 |
| `[DIA-22]` | changed | a diagnostic caused by a callee's inferred fact is reported at the call, the callee as secondary span | done for contracts and signatures | partly | extend to BLD-14 facts (A7-14) | keep 1, respell 1 (§6) | 0 |
| `[DIA-23]` | changed | a run-time stop names Ember entities (the stop texts of `[PRF-4]`) | panics name functions; texts are 0.9.10's (see COST-6) | partly | the COST-6 texts (A7-16) | no folder | 0 |
| `[DIA-25]` | added | the line a message gives compiles; several lines replaced are all named; a moved line says where; `mut path` is `E0113` | no check that a fix compiles; suggestions are span edits (`ember_diag/src/lib.rs:169-192`) | not built | line-edit fixes; the test runner applies each fix and checks again (A7-14) | no folder | 0 |
| `[DIA-26]` | added | one mistake, one message, on the program's line; a fault inside a stored function is reported where it is stored | no de-duplication in `Sink::emit` (`ember_diag/src/lib.rs:219`) | not built | arbitration per primary line; store-site attribution from the CLO area (A7-14) | no folder | 0 |
| `[DIA-27]` | added | a message about a change a call makes names the chain of calls | effect diagnostics name one callee | not built | witness edges in the inferred summaries (EFF area) (A7-14) | no folder | 0 |
| `[DIA-28]` | added | two errors on one line: the one whose fix compiles (`E7018` over `E3020`, `E3020` over `E7007`, `E3060` over `E3022`, `E3031` over `E3021`) | `Sink::ordered` (`ember_diag/src/lib.rs:316`) sorts and shows all | not built | a precedence table applied per line (A7-14) | no folder | 0 |
| `[DOC-3]` | changed | `ember doc` shows each parameter's word, `from`, and what the compiler worked out (changes, reads, ends, chain, effects, allocation) | `ember doc` shows a generated contract from the signature | partly | read the BLD-14 interface facts (A7-22) | no folder | 0 |
| `[DRV-1]` | changed | `@derive(Copy)` gives `Clone`, never on a class (`E2080`); `Clone` copies a link field as one more link; no `Component` derive | derives in `ember_typeck/src/derives.rs`; `Component` derive exists | partly | Copy-implies-Clone, class refusal, link-field cloning (with RC/WK area), drop `Component` (ECS area) (A7-10) | keep 2 | 0 |
| `[FMT-1]` | changed | `ember fmt` never changes a word in front of a name or a capture word | the formatter prints 0.9.10's modes (`ember_fmt/src/lib.rs`) | partly | print the 0.9.15 AST (A7-22) | no folder | 0 |
| `[GRM-3]` | changed | type words before a type (`owns`, `shares`, `shares mut`, `thread_shared`, `weakref`, `ref`, `ref mut`, `sharedref mut`), `using` after one; fn-type parameter words; `thread_shared fn()` | `parse_type_nested` (`ember_parser/src/decls.rs:1454-1600`) knows `ref`, `*`, `fn(mut T)`, `fn() + Send` | not built | parse the words into the AST of A7-5; types per SD1 (A7-7) | keep 0, respell 1 (§6) | 0 |
| `[GRM-4]` | changed | `x = e` puts the value into the place `x` stands for; a declaration in a loop body is a new name each round | assignment and declaration are built; per-round names are not distinguished by the hand-over check | partly | per-round declaration identity for the OWN area's hand-over check (A7-8) | keep 2 | 0 |
| `[GRM-8a]` | changed | inside `[ ]` a type is parsed after `owns`, `shares`, `thread_shared`, `weakref`, `sharedref` too | the lookahead list lacks the new words (`ember_parser/src/body.rs`) | not built | add the words (A7-7) | keep 1 | 0 |
| `[GRM-8b]` | changed | an interface named bare as a type argument is `E2173` | `E2173` only for interfaces no class implements (ODR-182, `ember_typeck/src/habits.rs`) | partly | refuse every bare interface in a type position (A7-10) | no folder | 0 |
| `[GRM-13]` | changed | pattern names are read-only views (a number copied); `ref mut` names change the part; `mut` alone is `E0113`; `match owns e` | `Bind { by_ref, mutable }` (`ember_ast/src/lib.rs:988`); `mut x` accepted (`ember_parser/src/body.rs:1873-1880`); `match owned` (`:1173`) | partly | words on pattern names; `E0113`; read-only checks with the BRW area (A7-7) | keep 2, respell 4, rewrite 2 (§6) | 0 |
| `[GRM-15]` | changed | `owns e` required on the line making a value for an `owns` place (`E2123`), allowed before a `for`/comprehension iterable, a `match` scrutinee and a number; elsewhere `E0109` | `ExprKind::Owned` only for `for`/`match` (`ember_ast/src/lib.rs:760`; `body.rs:511`) | not built | value word `owns`; `E2123` with the OWN area (A7-7) | no folder | 0 |
| `[GRM-19]` | changed | `Some(...)` as an `if`/`elif`/`while` pattern is `E0115`; other variants keep the pattern form | `parse_condition` accepts any pattern (`ember_parser/src/body.rs:436`) | not built | refuse `Some` there with the `exists as` line (A7-7) | keep 0, respell 2 (§6) | 3 (R13 2, R14 1) |
| `[GRM-27]` | changed | a clause over `owns xs` hands the items over; over a collection read in place, items are read-only views; `copy of` copies | built with `owned` (`body.rs:1583`) | partly | the word `owns`; read-only views with the BRW area (A7-7) | keep 4, respell 4 (§6) | 0 |
| `[GRM-28]` | changed | a local function takes the lambda's capture words and is a value passed by its name | local functions capture as 0.9.10 lambdas; usable as values | partly | capture words (CLO area semantics) (A7-7) | keep 12, respell 6 (§6) | 0 |
| `[GRM-30]` | changed | `cast s as Circle` stops when it is not one; `as?`, `as!`, `cast s if`, `cast s when` are `E1010` with the kind test | `cast as`/`cast if` parsed (`ember_parser/src/body.rs:1047-1092`); `as?`/`as!` are `E0100` (`:1094-1107`) | partly | `cast if/when` and `as?/as!` → `E1010`; place casts (A7-12) | keep 1, respell 3, rewrite 1 (§6) | 0 |
| `[GRM-35]` | changed | overlay grammar: `with (header_hash=...)`, attributes on overlay class methods, trailing comma in contracts | overlay parser lacks the three (`ember_parser/src/decls.rs`) | not built | parse them; FFI-55 meaning is the FFI area's (A7-6) | keep 4, respell 1 (§6) | 0 |
| `[GRM-36]` | changed | `ref e`, `ref mut e`, `const ref mut e` name a place: also a call returning `ref`, a cast of a place, a span's elements | `RefOf { mutable, place }` (`body.rs:1053-1062`) with 0.9.10's place forms | partly | `const ref mut`; the new place forms (A7-7) | keep 1, respell 1 (§6) | 0 |
| `[GRM-37]` | changed | `unsafe import c` before a C source file | `import c` only | not built | parse `unsafe import c` (FFI-51 meaning: FFI area) (A7-6) | keep 7, respell 2 (§6) | 0 |
| `[GRM-38]` | changed | a generator expression reads what it iterates in place | built | text only | none (-) | keep 5, rewrite 1 (§6) | 0 |
| `[GRM-40]` | added | type parameters on a line of their own above a fn/struct/class; brackets after the name are `E0100` | brackets after the name (`decls.rs:972`, `:1058`, class `:1067`) | not built | bracket-line lookahead (the one place the parser looks past a line end) (A7-6) | no folder | 0 |
| `[GRM-41]` | added | a first line: parameters with words, `owns this` or captures, `-> T`, `from`; `E0118` out of order; `E0117` word in the type; `mut` alone `E0113`; no `self` (`E1010`); `static fn` (`E2115` outside a type); `changes`/`reads` clause `E0100` | `parse_param` takes `mut`/`owned`/`self` (`ember_parser/src/decls.rs:1014-1053`) | not built | new first-line parser; implicit object per SD2 (A7-6) | no folder | 2 (R5 2) |
| `[GRM-42]` | added | capture words after a lambda's brackets; `with` is `E0100`; `ref mut` capture `E0120`; a name with no word after a worded one `E0116` | `owned fn` (`Lambda.is_owned`, `ember_ast/src/lib.rs:838`; `body.rs:1649`) | not built | parse capture lists (CLO area semantics) (A7-7) | no folder | 8 (R2 1, R3 3, R5 1, R12 2, R13 1) |
| `[GRM-43]` | added | value words before a value bind as prefix `-`; `copy`/`of` contextual; `sharedref` alone `E0114`; `using` binds tighter | only `ref`/`ref mut` prefixes (`body.rs:1053`) | not built | prefix words and `using` level in the precedence table (A7-7) | no folder | 0 |
| `[GRM-44]` | added | `if/while x exists as p:` opens an `Option` or `weakref`; words on the name; `E2036` when it always holds; `E2106` name in use; `Some` in a header `E0115` | absent; `Condition::Pattern` (`ember_ast/src/lib.rs:639`) | not built | parse; checker for `Option` (A7-12); `weakref` with the WK area (A7-12) | no folder | 32 (R3 3, R5 1, R8 4, R10 1, R11 10, R13 12, R15 1) |
| `[GRM-45]` | added | `if s is Circle as c:` names it; `s is Circle` a `bool`; named form only in `if`/`elif` | `is` is identity only (`ember_parser/src/body.rs:948`) | not built | parse; resolve type-or-value after `is` (A7-12) | no folder | 1 (R14 1) |
| `[GRM-46]` | added | `from` after a view field type or a result type; `guarded by` after a field or `static mut` type | absent (`@borrows` instead, `ember_typeck/src/lib.rs:4068`) | not built | parse; `from` meaning with the LT area, `guarded by` with the THR area (A7-6) | no folder | 0 |
| `[HASH-1]` | changed | `Hasher` is not `Copy`; `finish() owns this -> u64` | `fn finish(owned self)` (`std/src/collections.em:28`) | partly | respell by the migrator (A7-9) | keep 2, respell 8 (§6) | 0 |
| `[HASH-3]` | changed | a `hash`/`eq` that may change or end the map it is called from is refused (`E3031`); nothing checked at run time | a `RefCell` borrow panics at run time | not built | EFF inference + `E3031` (BRW area); retire the `RefCell` path (A7-22) | keep 0, respell 1 (§6) | 0 |
| `[IDE-4]` | changed | editor shows inferred facts from the last `ember check`, never blocking | borrow/effect facts shown the same way | partly | show the BLD-14 facts (A7-22) | no folder | 0 |
| `[IDE-6]` | changed | an identifier's text is a view into the session's text | built | text only | none (-) | no folder | 0 |
| `[IMP-11]` | changed | the verification covers the checking of views and changes | built | text only | none (-) | no folder | 0 |
| `[LEX-14]` | changed | the 52-keyword table (adds `owns`, `shares`, `sharedref`, `this`, `thread_shared`, `using`, `weakref`; removes `owned`, `self`) | `Kw` has `owned` and `self`, lacks the seven (`ember_lexer/src/token.rs:223-235`; test `:351` pins 47) | not built | new table; `owned`/`self` as reserved non-words (A7-4) | keep 1 | 1 (R15 1) |
| `[LEX-15]` | changed | contextual `from` (result/field), `copy`/`of`, `exists`, `cast`, `guarded`/`by`; `owned` and `self` lexed and refused with `E1010` | contextual `from` (import only), `cast` (`token.rs:281`, `body.rs:1047`) | partly | the new contextual positions; `E1010` messages (A7-4) | keep 5, respell 3 (§6) | 0 |
| `[LEX-20]` | changed | a string literal is a `StrView` | type `str` (`TyKind::Str`, `ember_types/src/lib.rs:145`) | partly | rename `str` → `StrView`, `cstr` → `CStrView` (A7-8) | keep 1 | 0 |
| `[LEX-21]` | changed | `as?`/`as!` lexed only to be refused with `E1010` | refused with `E0100` (`body.rs:1094-1107`) | partly | code and message (A7-4) | keep 2 | 0 |
| `[LEX-23]` | changed | safety notes before `unsafe:`, `unsafe extend/extern/overlay`, `unsafe import c`, `@unsafe(...)`; none on an `unsafe fn` (`W3013`) | an `unsafe fn` with a body requires a note (`ember_parser/src/decls.rs:983-985`) | partly | reverse the `unsafe fn` case, add `W3013`, add the new constructs (A7-6) | keep 2 | 0 |
| `[LNT-6]` | changed | `L2001` unnecessary `copy of`; `L3002` a view in use too long; no `L3001` (a loop of owners is `E3032`) | `L2001` on `clone()`, `L3001` emitted (2 sites) | partly | retarget `L2001`; retire `L3001` (A7-22) | keep 3, respell 5 (§6) | 0 |
| `[MNG-1]` | changed | built-in paths under `std.core` (`std.core.StrView`); `[MONO-10]`'s copy is an instance | `std.core.str` | partly | rename; MONO-10 instances with the MONO area (A7-21) | keep 1, respell 4 (§6) | 0 |
| `[MNG-2]` | changed | `@unsafe(export("name"))` sets the symbol | `@export("name")` (`ember_typeck/src/lib.rs:20592`) | not built | new attribute form (FFI-26 with the FFI area) (A7-21) | no folder | 0 |
| `[MNG-6]` | changed | canonical spelling writes type words (`owns T`, `shares mut T`, `weakref T`, `using A`, `thread_shared fn()`) and MONO-10 pairs | spelling of 0.9.10 types | not built | spell the SD1 kinds (A7-10) | no folder | 0 |
| `[MOD-1]` | changed | a declaration may shadow `int`, `float`, `Never`, `StrView` | shadowing of `str` (`ember_typeck/src/lib.rs:1723, :1735`) | partly | rename (A7-8) | keep 3, respell 3 (§6) | 0 |
| `[MOD-5]` | changed | the prelude: `StrView`, `Cell`, `ThreadShared`; no `Box`, `Shared`, `Weak`, `RefCell`, `Sync`, `str` | `COMPILER_KNOWN_TYPES` lists `Box`, `Shared`, `Weak`, `RefCell` (`ember_typeck/src/lib.rs:1716-1719`) | not built | new prelude; each removal lands with its replacement's slice (A7-8) | keep 5, respell 4 (§6) | 0 |
| `[PHIL-2]` | changed | allocation only where a construct says so (an `owns T` place, `shares x`, `copy of`), listed by `inspect --alloc` | allocation sites from `ember_analysis::allocation_sites` for 0.9.10's forms | partly | report the new forms as their areas land (A7-22) | keep 0, rewrite 1 (§6) | 0 |
| `[PHIL-3]` | changed | no implicit copy: `copy of x`; a `Copy` type holds no owner or link | a class handle is copied with a retain (`ember_types/src/lib.rs:171-176`) | partly | class objects not `Copy` (OWN area); `copy of` (A7-8) | keep 0, respell 2 (§6) | 0 |
| `[PHIL-4]` | changed | synchronisation only through `Mutex`, `RwLock`, `Atomic`, channels, `guarded by`, `thread_shared` counts | `@sync` classes and `SyncShared` synchronise | partly | THR area (-) | keep 0, respell 1 (§6) | 0 |
| `[PHIL-8]` | changed | every rule on who may change or read a place is checked when compiling; run-time checks only for computed data | run-time exclusivity checks and `RefCell` remain (cost rows `ember_driver/src/lib.rs:1563-1564`) | partly | EXC and CELL areas remove them (-) | keep 1, respell 1, retire 1 (§6) | 0 |
| `[PHIL-9]` | changed | the header holds the counts and the type, nothing else | `ember_obj_header` holds `strong`, `weak`, `access`, `flags`, type pointer (`runtime/ember_rt/templates/ember_rt.h.in:1425-1437`) | not built | area of OBJ/RC (OBJ-1 layout) (-) | keep 0, respell 1 (§6) | 0 |
| `[PHIL-10]` | changed | safe code cannot end a value without clean-up or leak through a loop of owners | cycles leak (`L3001` lint) | partly | WK area (`E3032`) (-) | no folder | 0 |
| `[PHIL-11]` | changed | what remains possible includes a count reaching its limit; no reference-cycle leak | count overflow panics; cycles may leak | partly | RC/WK areas (-) | no folder | 0 |
| `[PHIL-15]` | changed | costs named and reported per site | `inspect --cost` (`ember_driver/src/lib.rs:1574`) | partly | the new rows (A7-2) | no folder | 0 |
| `[PHIL-16]` | added | the words say what the code does; inferred facts only decide whether a line compiles | 0.9.10 words | partly | the whole grammar and checker (A7-6) | no folder | 0 |
| `[PRF-1]` | changed | every profile performs the same run-time checks and stops with the same text | true for 0.9.10's checks | partly | with PRF-4 (A7-16) | no folder | 0 |
| `[PRF-4]` | added | the list of run-time checks, in every profile; no run-time exclusivity check | run-time exclusivity checks exist (row `Dynamic exclusivity check`, `ember_driver/src/lib.rs:1563`) | partly | EXC removal (other area); stop texts here (A7-16) | no folder | 0 |
| `[RFL-2]` | changed | `h.type_info()` on a class object | built | text only | none (-) | keep 1, respell 1 (§6) | 0 |
| `[RT-7]` | changed | a count is compared with its maximum before each +1 (one-thread 4,294,967,295; `thread_shared` stops at 2,147,483,647); texts; 8-bit test counts | overflow panics `reference count overflow on %s` (`runtime/ember_rt/templates/ember_rt.c.in:3037`); no half-range rule, no 8-bit build | partly | texts and the count-width build (A7-17); limit compares with the RC area (A7-17) | no folder | 0 |
| `[RT-8]` | changed | `thread_shared` counts atomic; `shares` counts plain | `@sync` counts atomic (`ember_rt.h.in:1550-1560`) | partly | RC/THR area (-) | no folder | 0 |
| `[RT-10]` | changed | counting only where a line makes/ends a `shares`/`thread_shared` owner or `weakref` link; one owner never counted; `fetch_add` then limit test | every class-handle copy retains | partly | RC area (-) | keep 0, rewrite 1 (§6) | 0 |
| `[RT-12]` | changed | stack probes on every target for Clang/GCC; `E9014` for a toolchain that cannot probe; flags in `cflags.txt` | `-fstack-clash-protection` on Linux only (`ember_build/src/lib.rs:1983-1986`) | partly | every target; probe-capability check; `cflags.txt` (A7-18) | keep 2, respell 1 (§6) | 0 |
| `[SEL-1]` | changed | views (`ref T`, `ref mut T`, a span, `StrView`) classify as `view`; function values as `callable` | built with `str` | partly | rename (A7-8) | no folder | 0 |
| `[SEL-2]` | changed | `shares`/`weakref` and `CppShared`/`CppWeak` never interconvert (`E5065`) | built for `Shared`/`Weak` | partly | with the type words (A7-10) | keep 0, respell 2 (§6) | 0 |
| `[SER-1]` | changed | derived `serialize`/`deserialize` with `[W: ...]` lines, `ref mut` writer/reader, `static fn deserialize` | generated with 0.9.10 signatures (`ember_typeck/src/derives.rs`) | partly | generate the new forms (A7-13) | keep 1 | 0 |
| `[TST-6]` | changed | Annex A is generated from a compiled fixture | `annex_a_is_generated_from_a_fixture_that_compiles` (`ember_driver/tests/milestones.rs:7680`) | text only | green when the fixture compiles (A7-25) | no folder | 0 |
| `[TST-11]` | changed | the suite's scenario list (hand-over, views and changes, walks, call check, function values, kept views, managers, `shares`/`weakref`, loops of owners, threads) | 0.9.10's list (exclusivity, runtime cycles) in `tests/` | not built | move the 841 design tests into `tests/conformance`; retire the 0.9.10 scenarios (A7-24) | no folder | 0 |
| `[TST-31]` | added | `#$ count_bits: 8` compiles a test with 8-bit counts, used only by the two count-limit tests | the runner refuses unknown keys (`ember_driver/tests/milestones.rs:268-275`) | not built | runner key + build define (A7-17) | no folder | 0 |
| `[TXT-1]` | changed | `StrView` is a read-only view of valid UTF-8 | built as `str` | partly | rename (A7-8) | no folder | 0 |
| `[TXT-2]` | changed | conversions return `Result[StrView, Utf8Error]` | built as `str` | partly | rename (A7-8) | keep 1, respell 3 (§6) | 0 |
| `[TXT-3]` | changed | C boundary uses `CStrView`/`CString` | built as `cstr` | partly | rename (A7-8) | no folder | 0 |
| `[TXT-4]` | changed | `s.get(a..b) -> Option[StrView]` | built as `str` | partly | rename (A7-8) | keep 3, respell 1 (§6) | 0 |
| `[TXT-5]` | changed | `String(s)` makes a `String` | `to_string`/`String.from` | partly | add `String(s)` (A7-8) | no folder | 0 |
| `[TXT-6]` | changed | C ABI of `StrView` | built as `str` | partly | rename (A7-8) | no folder | 0 |
| `[TXT-8]` | changed | a `StrView` lives no longer than its text; a reshape ends it; a `String` place reads as a `StrView` | built with regions | partly | reshape wording with the BRW area (A7-8) | keep 0, respell 1, rewrite 1 (§6) | 0 |
| `[TXT-9]` | changed | a literal is a `StrView` view or a `String`; `String(s)` | built as `str` | partly | rename; `String(s)` (A7-8) | keep 1, respell 3 (§6) | 0 |
| `[TXT-10]` | changed | `StrView` operations; methods give views into `s` | built as `str` | partly | rename (A7-8) | keep 18, respell 13, rewrite 3 (§6) | 0 |
| `[TXT-11]` | changed | `String` operations with `String(s)`, `as_str`; `+` takes `a` over; changes are reshapes | built | partly | `String(s)`; reshape ends views (BRW area) (A7-8) | keep 16, respell 10 (§6) | 0 |
| `[TYP-1]` | changed | properties include `ThreadShared`, not `Sync` | `is_sync` (`ember_types/src/lib.rs:1761`) | partly | rename the property (A7-10) | keep 2, respell 1 (§6) | 0 |
| `[TYP-5]` | changed | coercions: `owns T` read as `T`; a value to `dyn I` keeping its word (`owns dyn I`, `shares dyn I`); `MutSpan` given with `ref mut` | 0.9.10 list (`Box`, class handle to interface) | partly | the SD1 kinds (A7-10) | keep 6, respell 4 (§6) | 0 |
| `[TYP-6]` | changed | a downcast is `cast h as D` or `if h is D as d:` | `Downcast { forced }` (`ember_ast/src/lib.rs:730`) | partly | drop the unforced form (A7-12) | keep 7 | 0 |
| `[TYP-9]` | changed | `@fp(allow_fma)`; `@fp(contract)` is `E0104` | `@fp(contract)` (`ember_typeck/src/lib.rs:2081`) | partly | rename (A7-8) | keep 5, respell 4 (§6) | 0 |
| `[TYP-9b]` | changed | `@fp(allow_fma)` enables FMA in that function | as `@fp(contract)` | partly | rename (A7-8) | no folder | 0 |
| `[TYP-13]` | changed | niche for class objects, `owns T`, `StrView`, `CStrView` | built for handles, `Box`, `str`, `cstr` | partly | the new kinds (A7-10) | keep 6, respell 8 (§6) | 0 |
| `[TYP-14]` | changed | `owns T` read as `T` (`.get()` is `E2072`); `=` on a `ref mut` name writes its place; `r = ref mut b` re-points it; `const ref mut` never re-points | a reference local is never re-seated; `Box` read-through | partly | new assignment rules (with the BRW area's `E3023`) (A7-27) | keep 0, respell 5 (§6) | 4 (R3 2, R11 2) |
| `[TYP-15]` | changed | a view may be stored anywhere; what holds it lives no longer than what it points into; class view fields say `from` (`E2118`) | only `static` views stored in heap places (0.9.10 rule; tests `TYP-15/reject_*`) | not built | LT area's region engine; `from` fields and `E2118` here (A7-26) | keep 3, respell 8, rewrite 16 (§6) | 9 (R6 1, R7 8) |
| `[TYP-17]` | changed | bounds on the line above; a `pub` function writes them (one `E2040`); a non-`pub` one has them worked out from its body (`E2072`, `E2070`) | bounds after the name; missing bound always `E2040` | not built | bounds inference for non-`pub` generics (A7-13) | keep 3, respell 33, rewrite 6 (§6) | 2 (R14 1, R17 1) |
| `[TYP-19]` | changed | no specialisation (example uses `StrView`) | built | text only | none (-) | keep 0, respell 7 (§6) | 0 |
| `[TYP-21]` | changed | operators read operands in place; `String +` takes its left operand over | built | text only | none (-) | keep 1, respell 9 (§6) | 0 |
| `[TYP-22]` | changed | `dyn`-compatible: every method has an object; generic only over a viewed type parameter (one entry); `E2050` gives the line | generic methods refused (`E2050`) | partly | the viewed-type-parameter exception; message (A7-13) | keep 2, respell 106, rewrite 2 (§6) | 1 (R14 1) |
| `[TYP-23]` | changed | a lambda parameter's word is written, never inferred; mismatch is `E2020` | `owned` inferred from the expected type (ODR-025) | partly | stop inferring; `E2020` (retires `E2228`) (A7-8) | keep 6, respell 2, rewrite 1 (§6) | 0 |
| `[TYP-24]` | changed | marker table: `Copy` never for class objects or links; `Send`/`ThreadShared` inferred | class handles `Copy`; `Sync` marker | partly | with SD1 and the OWN area (A7-10) | keep 1, respell 13 (§6) | 0 |
| `[TYP-31]` | changed | an index out of range stops with the bounds text | text without the `.get(i)` hint | partly | stop text (A7-16) | keep 6 | 0 |
| `[TYP-34]` | changed | view types include `sharedref mut T`; `from static` fields do not make a view type; a class is never a view; `@view` checked | view inference and `E2030` built (`ember_typeck/src/lib.rs:9958`) | partly | new members and message (A7-26) | keep 2, respell 1 (§6) | 2 (R7 2) |
| `[TYP-35]` | changed | phantom parameter written `[Tag]` above; counts for `ThreadShared` | `Handle[Tag]` after the name | partly | with GRM-40 and TYP-1 (A7-6) | keep 0, respell 2, rewrite 1 (§6) | 0 |
| `[TYP-36]` | changed | `copy of x` by `Clone` (`E2116` when it cannot); `owns`/link fields copied as more owners/links; `ref T` implements as `T` | `.clone()`; `Box` compare/clone | partly | `copy of`; link copying with the RC/WK area (A7-10) | keep 4, respell 17 (§6) | 0 |
| `[TYP-38]` | changed | `[owns Circle(r=1.0)]`; `[0.0, 1.0] using current` | literals of 0.9.10 | partly | the words in literals (A7-11) | keep 8, respell 2 (§6) | 0 |
| `[TYP-39]` | changed | an `owns T` displays as its `T` | `Box` prints as `T` (`ember_types/src/lib.rs:2031`) | partly | with SD1 (A7-10) | keep 3, respell 4 (§6) | 0 |
| `[TYP-43]` | changed | `Send`/`ThreadShared` inferred; compiler types recognised by module never by name; `unsafe extend T implements Send` with a SAFETY note; `T: Sync` is `E1010` | recognition by name prefix (`ember_types/src/lib.rs:1874-1879`, `ember_typeck/src/lib.rs:7961`); `Sync` marker | partly | SD1 kinds; module-based recognition (A7-10) | no folder | 0 |
| `[TYP-44]` | added | kind tests and casts: `E2036`, `E2037`, `E2106`, cast of a place, one type-index compare, the stop text | `ember_downcast` walks the base chain (`runtime/ember_rt/templates/ember_rt.c.in:3407`) | partly | SD4 (A7-12) | no folder | 24 (R3 2, R5 1, R14 20, R17 1) |
| `[TYP-45]` | added | who owns a value, in its type (`T`, `owns T`, `shares`, `thread_shared`, `weakref`, views, `owns dyn I`, function types); `E2114`, `E2123`, `E2173`, `E2020` | `Box[T]`, `Shared[T]`, `Weak[T]`, `SyncShared[T]` as compiler structs (`ember_types/src/lib.rs:701-702`, `:2006-2015`) | not built | SD1 (A7-10) | no folder | 4 (R3 3, R14 1) |
| `[TYP-46]` | added | `using A` after an owning type; the making line names the instance; `E2114`, `E2020`, `E3063` | `Array[T, A]`, `Box[T, A]` | not built | parse and type formation here; allocation with the ALC area (A7-11) | no folder | 6 (R3 6) |
| `[GRM-2]` | (code row) | (not in H.41; `E0100`'s row changed) `global`, a statement at module level outside the entry file, `changes`/`reads`/`with` after a function's brackets | `E0100` for `global` (`ember_parser/src/python.rs:156`) | partly | the new cases (A7-6) | no folder | 10 (R3 1, R5 2, R12 5, R15 2) |
| `[MOD-3]` | (code row) | (not in H.41; `E1010`'s row changed) gone names (`self`, `Box`, `RefCell`, `Weak`, `Shared`, `owned`, `Sync` property, `cast if`, `upgrade`, `mem.forget`, `.clone()`, `.copied()`) are `E1010` with the word to write | `E1010` is name resolution only (`cannot find name`) | partly | the gone list with messages (A7-15) | no folder | 32 (R1 1, R3 12, R4 1, R5 6, R9 2, R11 1, R12 2, R13 1, R14 5, R15 1) |
| `[TYP-4]` | (code row) | (not in H.41; `E2020`'s row changed) a lambda word mismatch, an unopened `weakref`, an object stored in a `weakref` place without the word, a value from the wrong manager | `E2020` mismatched types | partly | the new cases (A7-8) | no folder | 6 (R3 3, R11 2, R13 1) |

Status of the 128 H.41 rows: built 0, partly 85, not built 35, remove 0, text only 8. No rule of these families is `removed` in H.41; the features the spec dropped that this area's code holds (`Box`, `str`, `owned`, `self`, `as?`/`as!`, `cast if`, `@fp(contract)`, `@static_safe`, `@borrows`, `@export`, the `--borrow` flag, the six cost labels) are handled in the rows of the rules that replace them.

## 2. Work items

Grouped into slices; each slice leaves the compiler built, its tests green and nothing half-removed.
Areas are named by the families they hold (the brief's numbering of other areas is not in my inputs):
OWN/BRW (hand-over, views, changes), RC/WK/OBJ (counts, `weakref`, header), EFF/BLD-14 (inferred
summaries, interface files), CLO (function values), THR, FFI, ALC/ARN, LT, EXC/CELL (the run-time
checks being removed), MONO, DSJ.

### Slice M0: the spec-tied tests read 0.9.15 (no language change)

**A7-1 The code registry and the shape table at 0.9.15** (S)
- Files: `compiler/ember_diag/src/codes.rs`, `compiler/ember_diag/src/shapes.rs`.
- Add the 34 codes (E0113-E0118, E0120, E2037, E2106, E2108, E2112, E2114-E2119, E2123, E3028, E3031,
  E3032, E5013, E5069, E7007, E7008, E7012-E7018, E9014, W3013) with the target's rows; fix the 9 rule
  citations; give the 59 changed rows their 0.9.15 titles (E0109 is no longer about `owned`, E3023 no
  longer "two writers", E4030 no longer `@static_safe`...). Retired codes stay registered (never reused).
- Shapes: add B16 (`E3031`), B17 (`E3032`), N15-N18; `E3028` joins B10. O3, O4, O5, B8, B14, B15 and
  X1 stay while their (retired) codes are still emitted, and go with the emitting area's slice.
- Rules: DIA-7 (table part), and `[DIA-6a]`/`[DIA-7a]` (unchanged rules, new table).
- Tests: `codes.rs` `the_registry_and_the_specification_name_the_same_codes` green (verified in
  Python: after this, 0 unregistered, 0 citation mismatches); `shapes.rs` tests stay green.
- Depends on: nothing.

**A7-2 `explain --view` and the cost labels** (S)
- Files: `compiler/ember_driver/src/lib.rs` (`:618` flag, `:71-73` help, `:1557-1588` `check_cost_row`
  and its class match, `:2673`, `:2716`), `compiler/ember_driver/tests/milestones.rs` (`:925-956` cost
  test strings).
- `--borrow` becomes `--view` with DIA-8's wording. Labels: `` `assert_disjoint` comparison `` →
  `` `assert_disjoint` ``; `Downcast check` → `Kind test or cast`; `Owned callable value` → `A call
  through a function value (a parameter, field, list element or local of a function type)`;
  `Retain / release, surviving` → `Owner count +1 / -1` (classes in the table match).
- Rules: DIA-8, COST-3 (labels), PHIL-15.
- Tests: `help_lists_every_command_and_flag_of_the_specification` green; `rule_index.py` from 6
  `[COST-5]` problems to 2. The last two (`Dynamic exclusivity check`, `` `RefCell` borrow ``) are
  removed by the EXC and CELL areas' slices that delete those checks; mapping them to another row
  would misreport a cost.
- Depends on: nothing.

**A7-3 The spec tools follow the pin** (S)
- Files: `tools/spec_check.py` (reads `docs/spec-source/ember-spec.md`, which is 0.8.5: once the grammar
  changes its old blocks fail beyond the baseline), `tools/spec_check_baseline.json`,
  `compiler/ember_driver/tests/milestones.rs:8697` (`BASELINE` of `every_example_of_the_specification_checks`).
- `spec_check.py` reads the pinned target, as its `--emit-appendix` already does; its baseline is
  re-recorded for the new revision (`--allow-growth`, the case `[TST-4c]` names). The 49 ` ```ember `
  blocks of the target get a heading baseline in the Rust test, reported as `baselined` (`[TST-29]`),
  shrinking as slices land.
- Rules: TST-6/TST-7 (unchanged TST-7; TST-6 text).
- Depends on: nothing.

### Slice M1: the 0.9.15 grammar, the spelling-only changes, and the migrator (one commit)

A7-4 to A7-9 land together: the new parser refuses every removed spelling with a migration, the
migrator rewrites the repository, and the version moves to 0.9.15. Constructs parsed here whose
meaning belongs to an area not yet landed (`shares`, `shares mut`, `thread_shared`, `weakref`,
`sharedref mut`, `guarded by`, `using`, capture words) are refused by name with `E0900`
(`[CLI-19]`), and their 0.9.10 forerunners (`Shared`, `Weak`, `SyncShared`, `RefCell`, `@sync`,
`owned fn`, `mem.forget`, `.upgrade()`) stay until the slice that builds the replacement removes them
and migrates their users: that is the owner's "nothing half-removed" applied per feature. The word
`owned` stays a keyword only for `owned fn` until the CLO area's capture slice; every other use of it
is `E1010` here.

**A7-4 Lexer** (S)
- `ember_lexer/src/token.rs:223-281`: the 52-keyword table; `owned` and `self` move to a "reserved,
  not words" class refused with `E1010` and the word to write; contextual `from` (after a result or
  field type), `copy`/`of`, `exists`, `guarded`/`by`; `as?`/`as!` → `E1010` with the kind test or cast.
  The lexer test pinning 47 (`:351`) moves to 52.
- Rules: LEX-14, LEX-15, LEX-21. Old tests: LEX-15 3 respell. Design: every test with `this` (258).

**A7-5 The syntax tree for the new forms** (M)
- `ember_ast/src/lib.rs` (+`print.rs`): `Mode` → `ParamWord { Ref, RefMut, ConstRefMut, Owns }`
  (423 uses of `Mode::*` across the workspace, renamed in place); `FnDecl { is_static, owns_this,
  from, captures, type_params }` with no `ParamKind::Receiver` in source (SD2); `TypeKind::{Owns,
  Shares{mutable}, ThreadShared, WeakRef, SharedRefMut, Using}` and `Fn { thread_shared }` (replacing
  `send`); `ExprKind::{ValueWord, Using}`, `Downcast` forced only; `Condition::{Exists, KindTest}`;
  pattern names with a word; `FieldDecl { from, guarded_by }`, `StaticDecl { guarded_by }`;
  `Lambda { captures }` replacing `is_owned`.
- Depends on: SD1, SD2.

**A7-6 Parser: items and first lines** (L)
- `ember_parser/src/decls.rs:931-1053` and the item loop: type-parameter lines (GRM-40, the one
  lookahead past a line end), first lines in order with `E0118`/`E0117`/`E0113`, `static fn`, `owns this`,
  `-> T from p`, `changes`/`reads`/`with` clauses → `E0100` with the first line to write (GRM-41,
  GRM-2), `from`/`guarded by` on fields and statics (GRM-46), overlay additions (GRM-35),
  `unsafe import c` (GRM-37), safety notes on the new `unsafe` constructs and `W3013` on an `unsafe fn`
  (LEX-23; reverses `decls.rs:983-985`), phantom `[Tag]` (TYP-35).
- Design tests: 17 use a type-parameter line (R14 12, R12 1, R15 2, R17 2); GRM-41 2 (R5); GRM-2 10
  (R12 5, R15 2, R3 1, R5 2).

**A7-7 Parser: expressions, conditions, patterns, captures** (M)
- `ember_parser/src/body.rs`: value words as prefixes at `-`'s level and `using` above calls (GRM-43,
  `E0114`); type words (GRM-3, GRM-8a); `owns e` positions and syntactic `E0109` (GRM-15, GRM-27);
  `exists as` with worded names, `Some(...)` in a header `E0115` (GRM-44, GRM-19); `is T as name`
  in `if`/`elif` only (GRM-45); `cast` operand rules and `E1010` for `cast if/when` (GRM-30); `ref`,
  `ref mut`, `const ref mut` operands (GRM-36); pattern words and `E0113` (GRM-13); capture lists with
  `E0100` (`with`), `E0120`, `E0116` (GRM-42, GRM-28).
- Design tests: GRM-44 32 (R11 10, R13 12, R8 4, R3 3, R10 1, R15 1, R5 1), GRM-42 8, GRM-19 3,
  GRM-45 1; 94 use `exists as`, 116 a capture word, 78 `copy of`, 144 `ref mut` at a call.

**A7-8 Checker for the spelling-only forms** (L)
- `ember_typeck` (68k lines), `ember_hir`, `std/`: a method's object is implicit (`this`; `E1010`
  outside a method or in a `static fn`); `static fn` called on the type (`E2115` outside a type body);
  parameter words onto the existing modes (`ref` = borrow, `ref mut` = `mut`, `owns` = `owned`; new
  `const ref mut` never re-pointed); `copy of` through `Clone` (`E2116` for what it cannot copy);
  `exists as` on an `Option` (names read-only, `copy of v` a copy of one's own); lambda parameter words
  written, not inferred (TYP-23: drop ODR-025's inference, `E2020`, retire `E2228`); `str`→`StrView`,
  `cstr`→`CStrView`, `String(s)` (TXT-1 to TXT-11, LEX-20, MOD-1, MNG-1, SEL-1); `@fp(allow_fma)`,
  `@no_alias_checks`, `from` replacing `@borrows`, gone attributes and names with `E0104`/`E1010` and
  their texts (DET-5, TYP-9, TYP-9b, CG-C-11, with A7-15); the prelude without `Box` and `str`
  (MOD-5, the rest with their areas); `LANGUAGE_VERSION` "0.9.15"
  (`ember_parser/src/lib.rs:21`).
- The object's change-mode: 0.9.10 read it from `mut self`; 0.9.15 works it out from the body. This
  item uses the EFF area's "may change `this`" summary (EFF-24/BLD-14) and does not grow its own.
- Tests: `the_three_versions_are_separate` green; 4 files with `#! language "0.9.10"` respelled
  (FFI-21, GRM-37, LEX-10, VER-8).
- Depends on: EFF area's first summary slice (object change-mode); FN area's call-site words
  (`E3028`, FN-2a).

**A7-15 Gone names and attributes, with their texts** (S)
- `ember_typeck/src/lib.rs:60305-60367` (attribute table), `ember_diag/src/habits.rs`,
  `ember_parser/src/python.rs:286`: `E0104` for `@sync`, `@static_safe`, `@fp(contract)`, `@borrows`,
  `@no_runtime_checks` with the rule's texts; plain `@export` `E3100`; `E1010` for every gone name of
  its row with the word to write; Python habits P14 (reversed: `this`), P18, P22, P26, P27.
- Rules: ATT-1, ATT-2, MOD-3, DIA-21. Old tests: ATT-1 1 respell. Design: ATT-1 5 (R15 1, R16 2,
  R3 1, R6 1), MOD-3 32 (R3 12, R5 6, R14 5, ...), PHIL-12 5.
- Each gone name whose replacement is another area's (`Shared`, `Weak`, `RefCell`, `@sync`,
  `mem.forget`, `upgrade`) is switched on in that area's slice.

**A7-9 The migrator, run over the repository** (L)
- `ember_driver/src/lib.rs:3210-3290` (`migrate_file`): migrations for every 0.9.10 form CLI-20
  lists; the checker runs when the only errors carry migrations (today it is skipped on any error,
  `:3253`), so `mut x` parameters can add `ref mut` at their calls, in every file of the package (today
  only the named file).
- Run over: `std/src/*.em`; `tests/**` (3,147 conformance files and the 511 others); `docs/book/`
  (52 samples, `guide.rs`); `docs/errors/` (section 7); `docs/spec-source/` is already 0.9.15. The
  ~220 Ember programs inside Rust tests (`milestones.rs` 107, `reload.rs` 61, `package.rs` 14, ...)
  are moved by a script that runs the migrator on each string.
- Rules: CLI-20, HASH-1 (std signature), and the respell column of section 6 (322 files of this
  area's folders).
- Depends on: A7-4 to A7-8.

### Slice M2: ownership words in types (with RC/WK/OBJ and OWN)

**A7-10 The type representation of the words** (M)
- `ember_types/src/lib.rs`: SD1's kinds replace the compiler structs `Box`, `Shared`, `Weak`,
  `SyncShared` (`:701-702`, `:2006-2015`, `:2307-2349`, `:2653`); `Send`/`ThreadShared` over the kinds
  (`:1761-1910`), compiler types recognised by declaring module, never by name (the `Cell_`/`RefCell_`
  prefix test at `:1874-1879` and `ember_typeck/src/lib.rs:7961` go: TYP-43); display (`owns T` prints
  as `T`, TYP-39); niche (TYP-13); canonical spelling (MNG-6); coercion row 8 (TYP-5); `E2114`, `E2173`
  for a bare interface (GRM-8b, TYP-45), `.get()` on `owns` `E2072` (TYP-14); `copy of` copying links
  as more owners/links (TYP-36, DRV-1); `E5065` for the new kinds (SEL-2); markers (TYP-1, TYP-24).
- `owns T` is the 0.9.10 `Box[T]` machinery (one owner, its own block, nullable-pointer niche) under
  its new spelling, so it lands here; `shares`/`weakref`/`thread_shared`/`sharedref mut` land when the
  RC/WK/OBJ area's counts and header land, in the same slice that removes `Shared`, `Weak`,
  `SyncShared`, `@sync` and migrates their tests (TYP-22's 67 `Weak[Shared[...]]` files among them).
- Depends on: RC/WK/OBJ (count header, OBJ-1), OWN (`E2123`, `[OWN-10]`).

**A7-11 `using` memory managers** (M): parse and type formation (TYP-46, TYP-38 `using m` on a
literal, GRM-43 precedence), `E2114`, `E2020` for a value from the wrong manager; allocation through
the instance is the ALC/ARN area's. Design tests: 9 (all R3). Depends on: ALC/ARN.

### Slice M3: opening and kind tests

**A7-12 `exists as`, `is T as`, `cast`** (M)
- Checker: GRM-44 for `Option` (the `weakref` half with WK), GRM-45, TYP-44 (`E2036`, `E2037`,
  `E2106`, a cast of a place is a place, `E3027` for `ref mut` through a plain one), TYP-6, GRM-30;
  backend: one type-index compare (SD4) replacing `ember_downcast`'s base-chain walk
  (`runtime/ember_rt/templates/ember_rt.c.in:3407`); the cast's stop text with the program's names.
- Design tests: TYP-44 24 (R14 20, R3 2, R5 1, R17 1); 14 use `is T as`, 8 `cast as` (all R14).
- Depends on: OBJ-1's 32-bit type index (RC/WK/OBJ).

### Slice M4: generics

**A7-13 Bounds and `dyn` entries** (M)
- TYP-17: bounds of a non-`pub` generic worked out from its body (each method or operator used on a
  parameter's value gives the interface in scope declaring it; two are `E2070`; none `E2072`); a `pub`
  one gets one `E2040` naming every missing bound. TYP-22: a method generic only over a type used as a
  viewed parameter gets one `dyn` entry; `E2050` with the line. SER-1's generated signatures.
- Old tests: TYP-17 6 rewrite, TYP-22 2 rewrite. Design: TYP-17 2 (R14, R17), TYP-22 1 (R14).
- Depends on: A7-6. Owner question Q1 (partial bounds).

### Slice M5: diagnostics

**A7-14 The message framework** (M)
- `ember_diag/src/lib.rs`: fixes as line edits ("instead of ..., write ..."; "move line 17 below the
  loop") rendered as DIA-25 asks; per-line arbitration in `Sink` (DIA-26, DIA-28) by a precedence table
  (SD3); the chain of calls from the EFF summaries' witness edges (DIA-27, DIA-22); type names as the
  program wrote them (DIA-2); shape helps (DIA-3, DIA-7, DIA-9, DIA-10, DIA-11, DIA-16).
- The test runner applies every machine-applicable fix of a `compile-fail` test and checks again: the
  fix must compile (DIA-25's "never runs into another error"), so the rule is tested, not reviewed.
- Depends on: EFF (witness edges), CLO (where a stored function was stored, DIA-26), each emitting area
  for its own texts.

**A7-16 Stop texts in every profile** (S): COST-6's table, DIA-23, TYP-31 (`; use .get(i) for an
Option`), PRF-1, PRF-4; per-site static descriptors (SD5). Count texts with A7-17, `assert_disjoint`'s
with DSJ, the trampoline's with FFI.

### Slice M6: the count width for the limit tests

**A7-17 `#$ count_bits:`** (S)
- `ember_driver/tests/milestones.rs:157-275` (new key, allowed only on a `run-fail` or `run-pass` test;
  known-keys list), `ember_build` (`EMBER_COUNT_BITS` → `-DEMBER_COUNT_BITS=8` for the program and
  the runtime object, whose cache key is already the hashed command, `ember_build/src/lib.rs:2087`),
  runtime count type and limits, the numbers in the texts with thousands separators.
- Rules: TST-31, RT-7 (texts and test width). Design: R9 1, R11 1 (`#$ count_bits: 8`) and
  `R9/run_real_programs_keep_32_bit_counts.em`.
- Depends on: RC/WK/OBJ (the counts and their limit compare).

### Slice M7: toolchain and backends

**A7-18 Probes everywhere, `E9014`, `cflags.txt`** (S): `ember_build/src/lib.rs:1983-1986` (every
target for Clang and GCC), a probe-capability check giving `E9014`, `--emit c` writing
`cflags.txt` from the same flag function. RT-12, CLI-2.

**A7-19 `restrict`/`noalias` on every `ref mut` parameter** (M): a MIR parameter flag from the word
(never `this`), emitted as `restrict` (C) and `noalias` (LLVM); the MONO-10 copy passes one pointer.
CG-C-4, CG-LL-3. Depends on: BRW's call proof (`[BCK-8]`), MONO-10. Cost: none; it is what lets C
compilers vectorise as they do for C with `restrict`.

**A7-20 Visualisers** (S): natvis for the 0.9.15 header and links (CG-C-9). Depends on: OBJ-1.

**A7-21 Mangling** (S): MNG-1 (in M1 for `StrView`), MNG-2 `@unsafe(export(...))` with FFI-26,
MONO-10 instances (MNG-1, MNG-6) with MONO.

### Slice M8: tools

**A7-22 Lints, formatter, doc, IDE, cache, determinism** (M): LNT-6 (`L2001` on `copy of`, `L3002`
wording, `L3001` retired), FMT-1 (prints the new tree), CLI-18 (owner edges from WK's analysis),
DOC-3, IDE-4 and BLD-2 (BLD-14 facts), DET-2 (reductions, THR), DET-3 (function-value targets, EFF/CLO),
HASH-3 (`E3031`, BRW), PHIL-2 (new allocation forms), BEN-6 rows.

**A7-23 `comptime` as a read-only view** (S): CT-5 at `ember_typeck/src/const_eval.rs:115`;
`copy of comptime(e)`; `static mut` placement. Old test: CT-5 1 rewrite. Depends on: BRW (`E3023`).

**A7-26 Views kept anywhere** (M with LT's engine): TYP-15 (`from` on class view fields, `E2118`,
`from static`), TYP-34. Old tests: TYP-15 16 rewrite. Design: TYP-15 9 (R7 8, R6 1), TYP-34 2 (R7).
Depends on: LT (region engine, `[LT-45]`).

**A7-27 Names for places** (M): TYP-14's `=` through a `ref mut` name, re-pointing with `ref mut`,
`const ref mut`; GRM-4's per-round names. Depends on: BRW (`E3023`). Design: TYP-14 4 (R11 2, R3 2).

**A7-24 Old tests and error pages of this area** (M): the 39 rewrites and 3 retirements of section 6;
`docs/errors/`: 21 pages of retired codes retired, about 16 pages whose code's meaning changed
rewritten, the rest respelled by A7-9, a page for each new code when it is first emitted
(`tools/error_pages.py` requires pages only for emitted codes); TST-11's scenarios as the design tests
moved into `tests/conformance`.

### Slice M9: close-out

**A7-25 Annex A, the spec's examples, the guide** (S once the rest is in): TST-6's fixture
(`docs/spec-source/appendix-a.em` uses `weakref`, `thread_shared`, `guarded by`, `from`, `is T as`,
`exists as copy of`, `Lock`), the 49 blocks with an empty baseline (`[TST-7]`), `guide.rs`.
Depends on: every area.

Order: M0 → M1 → (M2, M3, M4 in parallel with the areas they need) → M5 → M6 → M7 → M8 → M9.

## 3. Shared decisions

**SD1. The ownership words are kinds of type, not library structs.** `TyKind::Owns { inner, alloc }`,
`Shares { inner, mutable, alloc }`, `ThreadShared { inner, alloc }`, `WeakRef { inner }`,
`SharedRefMut { inner }`; `alloc: Option<Ty>` is `using A` (also on `Class`, `Vec` and the `Map`/`Set`
structs); `Fn { thread_shared }` replaces `send`. Why: MOD-5 says they are "words of the language, not
library types"; TYP-43 forbids recognising compiler types by name, which the struct-with-origin scheme
does today (`ember_types/src/lib.rs:1822-1879`); MNG-6 spells them as words. Cost against C: none,
it is a compile-time representation; `owns T` is one pointer (C's `T*` from `malloc`), the others are
the RC/WK/OBJ area's layouts.

**SD2. A method's object is implicit in the tree and explicit in HIR.** The parser stores no receiver;
lowering adds one with mode `owns` (for `owns this`) or a view whose "may change" bit is the EFF
area's inferred summary. Downstream analyses keep their receiver handling (42 `ParamKind::Receiver`
sites) instead of being rewritten. Cost: none.

**SD3. Diagnostics carry line edits, and the sink picks one per line.** A fix is a list of
`(line range, replacement | move-to)`; the runner applies it to check DIA-25. The sink keeps every
diagnostic and renders, per primary line, the one first in a precedence table built from DIA-28
(`E7018` > `E3020` > `E7007`; `E3060` > `E3022`; `E3031` > `E3021`) and DIA-26. The chain of calls
(DIA-27) comes from one witness edge stored per fact in the EFF summary (the first call found to
produce it), followed at report time: no cost unless a message is printed. Cost while running: none.

**SD4. A kind test is one compare.** OBJ-1 gives a 32-bit type index. Number each class family in
pre-order at link time (the build sees the whole program); `x is C` is
`(uint32_t)(tid - first[C]) <= span[C]`, one compare and a branch, for open classes with descendants;
for a class with no descendants it is `tid == index[C]`. An interface value already carries the
object's type index. C++'s `dynamic_cast` walks; Rust has no subclassing. Today's base-chain walk
(`ember_rt.c.in:3407`) goes.

**SD5. Stop texts are static per-site data.** Each check site passes, on its failure path only, a
pointer to a static record (line, the expression's text, names); the runtime adds the dynamic part
(the object's type name from its index, the index and length). The hot path is unchanged: C's
compare and branch.

**SD6. The test count width is a build define.** `EMBER_COUNT_BITS=8` in the environment of a build
compiles the program and the runtime with 8-bit owner and weak counts in the header, stored and compared
as the 32-bit ones are; the runtime object is cached per command already. Nothing in the language
changes; no real build sets it.

**SD7. Migration-first transitions.** A removed spelling is never accepted beside the new one; it is
refused with a migration, and the slice that removes it runs the migrator over the repository. A
parsed construct whose meaning is another area's is `E0900` until that area lands (`[CLI-19]`).

## 4. Questions for the owner

**Q1. A non-`pub` generic that writes some bounds (TYP-17 against `E2072`'s row).** `[TYP-17]`: "A
function that is not `pub` may leave them out: its bounds are worked out from its body, each method
called on a parameter giving the interface in scope that declares it ... and a method no interface in
scope declares is `E2072`." The registry row of `E2072` (p17a): "in a generic body that is not `pub`,
a method no bound's interface declares". For `[T: Lookalike]` above a non-`pub` `fn direct_add(a: T,
b: T) -> T: return a + b` (`tests/conformance/TYP-17/reject_lookalike_operator_bounds_on_parameters_and_projections.em`),
does the body add `Add` to the written bound (TYP-17's reading: any interface in scope), or is it
`E2072`/`E2040` because the written bounds do not declare `add` (the row's reading)? It decides 6 old
tests and the bound inference of A7-13.

## 5. Risks

- **The object's change-mode moves from a word to inference.** 0.9.10's borrow checker reads `mut
  self`; 0.9.15 has no word. M1 cannot land before the EFF area's first summary exists, and every
  method call in `std/` changes from a declared to an inferred mode. If the inference is late, M1
  waits; a local fixed point over each type's methods would duplicate EFF's work and is not proposed.
- **The checker is 68k lines written around 0.9.10's names.** `Mode::*` 423 uses, `ParamKind::Receiver`
  42, `SelfExpr`/`SelfValue` 30, `is_owned` 36, name tests for `Box`/`Shared`/`Weak`/`RefCell`
  (`ember_types/src/lib.rs:1825-1879`, `ember_typeck/src/lib.rs:1717`, `:7961`, `:15127`). Renames
  are mechanical; the name tests are not.
- **`is` before a name.** `x is Name` is a kind test when `Name` is a type and identity when it is a
  value (TYP-44); the parser cannot know, so resolution decides (as `IndexOrInstantiate` already does).
  A name that is both (a class used as its constructor) needs a fixed order: the type reading first.
- **Error recovery for the migrator.** The checker must run on a tree the parser rebuilt after
  refusing old forms; today `migrate_file` skips checking on any error. Getting cross-file call-site
  edits right is the largest unknown in A7-9.
- **Inline programs in Rust tests** (about 220) are strings, not files; the migrator must be driven
  over them by a script, and assertions on exact output (mangled names, `assert-c` strings, message
  texts) will need hand edits.
- **`TYP-15` reverses a rule.** 0.9.10 allowed only `static` views in heap places; 0.9.15 allows any
  place that lives no longer than the source. 16 of 27 old tests flip; the LT area's engine carries it.
- **Not verified:** the classification of 555 old tests was by script (old spellings in code, codes
  in annotations, words in expected messages, runtime names in `assert-c`), with overrides after reading
  about 15 files in full and the headers or annotations of about 60 (TYP-15, TYP-17, TYP-22, GRM-13, GRM-28, GRM-30,
  GRM-38, CT-5, PHIL-8, DIA-11, TYP-23); some "keep" files may still expect a message whose wording
  DIA-2 changes. The count of rows whose text changed is exact (Python diff of both trees); "text only"
  was decided from those diffs. No build or test was run, so nothing here is measured.

## 6. Old tests of these rules, file by file

555 files in 68 folders: keep 191, respell 322, rewrite 39, retire 3. Respell names what changes (`self`: `this` and no object parameter; `generic-fn`/`generic-type`: brackets to a line above; `mut-param`: `ref mut` on the name and at the call; `owned`: `owns`; `str`/`cstr`: `StrView`/`CStrView`; `clone`: `copy of`; `to_string`: `String(s)` where the target spells it; `if-Some`: `exists as`; `upgrade`: `exists as`; `Box`: `owns`; `Shared`/`Weak`/`@sync`: `shares`/`weakref`/`thread_shared`, a choice the migrator lists; `mut-pattern`: `ref mut` on the name; `cast-if`, `as?!`: `if x is D as d`; `static_safe`, `fp-contract`, `borrows`, `export`: the new attribute; `message text`: the expected message's wording; `assert-c names`: mangled or runtime names in `#$ assert-c`). Folders with only keeps are not listed.

**ATT-1**
- respell 1: `accept_the_listed_attributes.em` (borrows)

**CG-C-4**
- respell 1: `accept_views_not_proved_apart_or_reassigned_keep_plain_pointers.em` (mut-param)

**CLI-18**
- rewrite 1: `accept_cycle_analysis_root.em` — expects `L3001` (retired): a loop of owners is now `E3032`

**CT-5**
- rewrite 1: `accept_where_results_live.em` — `comptime(e)` is a read-only view now; pushing to it is `E3023`

**DIA-11**
- retire 2: `reject_a_per_access_check_names_its_reason.em` — per-access run-time exclusivity checks are gone; `reject_a_refcell_check_names_its_reason.em` — `RefCell` is gone
- rewrite 1: `reject_a_loop_check_names_its_reason_and_no_restructuring.em` — `E4030` now means `assert_disjoint` inside `@no_alias_checks` (S1)
- respell 1: `accept_the_suggested_data_structure_is_static_safe.em` (mut-param, static_safe)

**DIA-22**
- respell 1: `reject_calls_against_their_signatures.em` (message text, self)

**GRM-13**
- rewrite 2: `reject_ref_mut_binding_of_a_borrowed_parameter.em` — `E3023` becomes `E3027` (`ref mut` over what the function may not change); `run_fail_a_write_while_a_class_field_binding_lives.em` — a run-time exclusivity stop becomes a compile-time refusal
- respell 4: `accept_matching_a_class_field_binds_by_reference.em` (mut-pattern, self); `accept_matching_a_place_binds_by_reference.em` (owned); `reject_moving_out_of_a_matched_place.em` (message text, owned); `reject_using_a_place_after_match_owned.em` (owned)

**GRM-19**
- respell 2: `accept_a_dotted_path_in_a_pattern_condition.em` (if-Some); `accept_a_pattern_condition_binds_in_its_block.em` (if-Some)

**GRM-27**
- respell 4: `accept_a_comprehension_takes_its_list_over_or_copies.em` (clone, mut-pattern, owned, self); `accept_set_and_map_comprehensions.em` (str, to_string); `reject_a_borrowed_item_kept_whole.em` (message text); `reject_a_list_taken_over_is_not_used_after.em` (message text, owned)

**GRM-28**
- respell 6: `accept_a_generic_local_function_is_made_for_each_use.em` (generic-fn); `accept_a_generic_local_generator_is_made_for_each_use.em` (generic-fn); `accept_a_local_generator_calls_itself_and_nests.em` (self); `reject_a_generic_local_function_used_as_a_value.em` (generic-fn); `reject_a_local_generator_body_reports_once.em` (message text); `reject_a_local_generator_borrows_what_it_captures_while_it_lives.em` (message text, owned)

**GRM-3**
- respell 1: `accept_generic_arguments_on_a_path_segment.em` (generic-type)

**GRM-30**
- rewrite 1: `reject_the_old_downcast_spellings.em` — `as?`/`as!` are `E1010` (were `E0100`), message gives `if s is D as d`
- respell 3: `accept_an_asserted_downcast_holds_at_any_address.em` (Box, self); `accept_cast_reads_a_field_an_element_and_a_comprehension.em` (cast-if, self); `accept_checked_and_asserted_downcasts.em` (cast-if, self)

**GRM-35**
- respell 1: `accept_overlay_grammar.em` (self)

**GRM-36**
- respell 1: `reject_ref_of_a_value.em` (message text)

**GRM-37**
- respell 2: `accept_creator_threads_directive.em` (export); `accept_language_and_threads_directives.em` (export)

**GRM-38**
- rewrite 1: `reject_a_later_for_clause_over_owned_or_uncopyable_items.em` — `E0109` no longer covers a later clause; `owns` before a comprehension iterable is allowed (GRM-15)

**HASH-1**
- respell 8: `accept_default_hasher_surface_is_deterministic_and_does_not_retain_spans.em` (mut-param); `accept_hash_for_the_table_types.em` (generic-fn); `accept_hash_is_a_prelude_interface.em` (generic-fn, mut-param, self); `accept_hash_prelude_without_an_explicit_std_import.em` (generic-fn); `accept_static_generic_hash_and_default_hasher.em` (generic-fn, mut-param, owned, self); `reject_concrete_hasher_in_a_hash_implementation.em` (mut-param, self); `reject_hash_needs_hashable_parts.em` (generic-fn); `reject_use_after_default_hasher_finish.em` (message text)

**HASH-3**
- respell 1: `accept_a_constant_hash_still_works.em` (generic-fn, mut-param, self)

**LEX-15**
- respell 3: `accept_extend_is_contextual.em` (generic-type, self); `accept_is_is_a_method_name.em` (self); `accept_not_is_a_method_name.em` (self)

**LNT-6**
- respell 5: `accept_a_copying_conversion_at_the_ffi_boundary_is_reported.em` (cstr); `accept_a_large_copy_value_passed_by_value_is_reported.em` (owned); `accept_an_unnecessary_clone_is_reported.em` (clone, message text, owned); `accept_an_unsafe_extern_declaration_without_a_contract_is_reported.em` (cstr); `accept_dynamic_dispatch_on_a_final_type_is_reported.em` (self)

**MNG-1**
- respell 4: `accept_a_programs_own_str_and_the_built_in_one_are_two_symbols.em` (self, str); `accept_a_struct_a_b_and_a_method_a_b_are_two_symbols.em` (self); `grid.em` (generic-fn); `reject_two_items_of_one_path_are_refused_before_their_instances.em` (generic-type, self)

**MOD-1**
- respell 3: `accept_a_module_and_an_item_of_one_name_in_two_scopes.em` (self); `reject_a_type_named_like_a_built_in_type.em` (self); `reject_an_item_and_a_module_of_one_path.em` (self)

**MOD-5**
- respell 4: `accept_a_programs_int_float_and_str_shadow_the_prelude.em` (str, to_string); `accept_a_programs_own_weak_type.em` (Weak); `accept_a_programs_type_shadows_a_prelude_type.em` (Box, generic-type, self); `reject_a_programs_box_is_named_as_written.em` (Box, generic-type, message text)

**PHIL-2**
- rewrite 1: `accept_every_allocating_form_is_listed.em` — the allocating forms are `owns`, `shares`, `copy of`, not `Shared`/`owned fn`

**PHIL-3**
- respell 2: `accept_a_copy_is_its_bits_and_its_retains.em` (clone, mut-pattern, self); `reject_no_implicit_copy_of_a_value_that_is_not_copy.em` (message text)

**PHIL-4**
- respell 1: `accept_no_synchronisation_the_types_do_not_ask_for.em` (assert-c names)

**PHIL-8**
- retire 1: `run_fail_a_check_where_no_proof_exists.em` — a `RefCell` run-time borrow panic; `RefCell` is gone
- respell 1: `accept_a_static_proof.em` (mut-param)

**PHIL-9**
- respell 1: `accept_a_value_type_has_no_hidden_state.em` (assert-c names)

**RFL-2**
- respell 1: `accept_type_info_keeps_a_description_for_run_time.em` (generic-fn, generic-type)

**RT-10**
- rewrite 1: `accept_counting_is_inline_and_an_upgrade_never_revives.em` — `@sync`/`Shared`/`Weak` become `thread_shared`/`weakref`; counting only at the lines that make owners

**RT-12**
- respell 1: `run_fail_a_thread_overflow_names_its_function.em` (owned, owned-fn)

**SEL-2**
- respell 2: `accept_shared_and_cpp_shared_each_keep_their_own_count.em` (Shared, clone); `reject_shared_and_cpp_shared_never_convert.em` (Shared, Weak)

**TXT-10**
- rewrite 3: `reject_a_trimmed_temporary_does_not_outlive_it.em` — a view of a temporary is `E3063`, not `E3060`; `reject_split_once_parts_of_a_temporary_do_not_outlive_it.em` — as above; `reject_the_bytes_of_a_temporary_do_not_outlive_it.em` — as above
- respell 13: `accept_ascii_runs_step_around_the_decoder.em` (str); `accept_driven_text_iterators_keep_signed_cursor_state.em` (assert-c names); `accept_native_text_cursors_keep_signed_indices.em` (assert-c names, owned); `accept_split_once_and_split_share_search_postconditions.em` (assert-c names, str); `accept_split_once_evaluates_the_text_before_the_separator.em` (mut-param, str); `accept_split_once_view_copy_chains.em` (str); `accept_split_once_wrapper_tag_discard_and_stored_split.em` (str); `accept_text_iterators_in_loops_and_through_next.em` (str); `accept_text_loops_change_way_between_english_mixed_and_short.em` (str); `reject_split_once_after_only_keeps_the_text_borrowed.em` (message text); `reject_split_once_before_only_keeps_the_text_borrowed.em` (message text); `reject_split_once_parts_keep_the_text_borrowed.em` (message text); `reject_user_access_to_unchecked_decoder.em` (message text)

**TXT-11**
- respell 10: `accept_checked_append_keeps_evaluation_and_bytes.em` (str); `accept_string_sizes_erase_integer_ranges.em` (mut-param); `reject_adding_two_strs.em` (message text); `reject_inserting_text_instead_of_a_char.em` (message text); `reject_string_argument_mutation_during_reservation.em` (message text, mut-param); `reject_string_clear_while_a_view_is_live.em` (message text); `reject_string_insert_while_a_view_is_live.em` (message text); `reject_string_remove_while_a_view_is_live.em` (message text); `reject_string_reserve_while_a_view_is_live.em` (message text); `reject_string_truncate_while_a_view_is_live.em` (message text)

**TXT-2**
- respell 3: `accept_bytes_stay_bytes.em` (clone, generic-fn); `reject_bytes_becoming_text_without_validation.em` (message text, str); `reject_mutating_bytes_while_validated_text_lives.em` (message text)

**TXT-4**
- respell 1: `accept_slices_at_stepped_cursors_are_not_checked_again.em` (assert-c names)

**TXT-8**
- rewrite 1: `reject_a_str_of_a_local_string_escaping.em` — a view returned past its text is `E3063`/`E3062`, not `E3060`
- respell 1: `accept_a_str_lives_as_long_as_what_it_views.em` (str)

**TXT-9**
- respell 3: `accept_a_str_literal_in_noalloc_code.em` (str); `accept_string_literal_initialises_string.em` (owned); `accept_to_string_and_string_from.em` (to_string)

**TYP-1**
- respell 1: `accept_128_bit_integers.em` (str, to_string)

**TYP-13**
- respell 8: `accept_bool_option_uses_third_byte_value.em` (str); `accept_box_option_uses_its_null_pointer.em` (Box); `accept_class_option_is_one_nullable_pointer.em` (mut-pattern, self); `accept_cstr_option_uses_null_pointer.em` (cstr); `accept_dyn_ref_option_uses_empty_fat_pointer.em` (self); `accept_mut_span_option_move_chain_through_projected_storage.em` (owned); `accept_pop_of_an_empty_array_is_none_for_every_niche.em` (owned, str); `accept_str_option_preserves_empty_view.em` (str)

**TYP-14**
- respell 5: `accept_a_box_operand_is_read_through.em` (Box); `accept_a_boxed_payload_is_passed_to_a_borrowed_parameter.em` (Box); `accept_a_reference_to_a_box_is_read_through.em` (Box, self); `accept_indexing_through_a_ref_field.em` (generic-type); `reject_a_boxed_payload_moved_into_a_value.em` (Box, message text)

**TYP-15**
- rewrite 16: `reject_a_borrowed_str_pushed_into_an_array.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_callback_view_result_stored_in_a_box.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_closure_storing_a_parameter_in_a_capture.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_local_view_stored_in_the_callers_place.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_mixed_region_view_in_box.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_non_static_view_in_a_class_object.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_non_static_view_in_a_static.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_non_static_view_through_an_element.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_slice_stored_in_an_array.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_view_stored_by_mem_replace_or_swap.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_view_stored_through_a_method_or_reference_into_the_heap.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_a_view_stored_through_a_mut_parameter.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_arena_backed_view_in_box.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_nonstatic_view_in_box.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_nonstatic_view_in_shared.em` — `static`-only storage became "lives no longer than its source" (TYP-15); `reject_swapped_fields_outliving_their_source.em` — `static`-only storage became "lives no longer than its source" (TYP-15)
- respell 8: `accept_a_multi_region_static_view_in_box.em` (Box, str); `accept_a_static_holds_a_view_made_of_literals.em` (str); `accept_a_static_region_view_in_a_static.em` (str); `accept_an_owning_container_at_a_view_type.em` (str); `accept_static_region_view_in_box.em` (Box, str); `accept_static_region_view_provenance_in_box.em` (Box, str); `accept_static_views_stored_every_way.em` (generic-fn, mut-param, owned, str); `reject_a_value_holding_its_owner_and_a_view_of_it.em` (message text, str, to_string)

**TYP-17**
- rewrite 6: `reject_a_generic_types_method_uses_only_its_bounds.em` — a non-`pub` generic now has its bounds worked out from its body (Q1 for written bounds); `reject_a_method_no_bound_provides.em` — a non-`pub` generic now has its bounds worked out from its body (Q1 for written bounds); `reject_an_associated_function_no_bound_provides.em` — a non-`pub` generic now has its bounds worked out from its body (Q1 for written bounds); `reject_an_uncalled_generic_method_of_a_generic_type.em` — a non-`pub` generic now has its bounds worked out from its body (Q1 for written bounds); `reject_comparisons_without_parameter_or_projection_bounds.em` — a non-`pub` generic now has its bounds worked out from its body (Q1 for written bounds); `reject_literal_left_comparisons_without_right_parameter_bounds.em` — a non-`pub` generic now has its bounds worked out from its body (Q1 for written bounds)
- respell 33: `accept_a_binding_names_another_parameters_associated_type.em` (generic-fn, generic-type, mut-pattern, owned, self); `accept_a_bound_call_reaches_the_interface_method.em` (generic-fn, self); `accept_a_bound_on_a_generic_interface.em` (clone, generic-fn, generic-type, self, str); `accept_a_bounded_extension_on_an_instance_over_a_parameter.em` (generic-type, owned, self); `accept_a_default_method_returning_a_generic_of_self.em` (generic-type, mut-pattern, owned, self); `accept_a_generic_body_meets_an_extension_through_its_own_bound.em` (generic-fn, generic-type, self, to_string); `accept_a_generic_interfaces_defaults_and_its_methods_own_parameters.em` (generic-fn, self); `accept_a_generic_types_method_calls_its_bound.em` (generic-fn, generic-type, self); `accept_a_receiverless_interface_member.em` (generic-fn); `accept_a_where_clause_on_an_associated_type.em` (generic-fn, owned); `accept_a_written_method_replaces_a_default_in_an_extend.em` (self); `accept_an_imported_bound.em` (generic-fn, self); `accept_an_inherent_next_is_the_iterators_for_a_bound_call.em` (copied, mut-pattern, self); `accept_comparisons_with_parameter_and_projection_bounds.em` (generic-fn); `accept_compiler_provided_capabilities_meet_bounds.em` (clone, generic-fn, owned); `accept_indexing_through_a_bound.em` (generic-fn, mut-param, str); `accept_literal_left_comparisons_with_float_parameters.em` (generic-fn); `accept_nominal_operator_bounds_and_ordinary_lookalike_methods.em` (generic-fn, self); `accept_the_sum_example.em` (generic-fn, self); `accept_where_clauses_bound_their_parameters.em` (generic-fn, generic-type, self); `reject_a_bound_naming_no_interface.em` (generic-fn, generic-type); `reject_a_default_body_uses_only_what_its_interface_provides.em` (self, str); `reject_a_generic_body_error_is_reported_once.em` (generic-fn, generic-type, to_string); `reject_a_generic_body_sees_only_its_own_bounds.em` (generic-fn, generic-type, self, to_string); `reject_a_generic_interface_bound_not_met.em` (generic-fn, self); `reject_a_generic_interface_callable_contract_mismatch.em` (generic-fn, self); `reject_a_where_clause_on_an_associated_type_is_met.em` (generic-fn, owned); `reject_an_extension_a_parameter_does_not_meet.em` (generic-type, owned, self); `reject_an_extension_whose_associated_binding_a_type_misses.em` (generic-type, mut-pattern, self, str); `reject_an_index_bound_not_met.em` (generic-fn, message text, str); `reject_an_interface_generic_method_with_a_nongeneric_implementation.em` (generic-fn, self); `reject_an_output_the_bound_leaves_unsaid.em` (generic-fn, self); `reject_lookalike_operator_bounds_on_parameters_and_projections.em` (generic-fn, mut-param, self)

**TYP-19**
- respell 7: `accept_an_extension_target_takes_its_type_defaults.em` (generic-type, self); `accept_implementations_that_cannot_meet_are_both_kept.em` (generic-type, self, str); `accept_implementations_their_bounds_keep_apart.em` (generic-type, mut-pattern, self, str); `reject_a_bound_met_through_a_blanket_overlaps.em` (generic-type, self); `reject_an_extension_with_a_defaulted_target_overlaps.em` (generic-type, self); `reject_overlapping_implementations_are_found_where_declared.em` (Box, generic-type, message text, self, str); `reject_overlaps_whose_interface_names_self_or_an_associated_type.em` (generic-fn, generic-type, mut-pattern, self)

**TYP-21**
- respell 9: `accept_a_generic_class_implements_two_instances_of_an_interface.em` (generic-type, self); `accept_a_generic_type_implements_two_instances_of_an_interface.em` (generic-fn, generic-type, self); `accept_a_number_on_the_left.em` (self); `accept_a_numbers_operator_method_beside_another_instance.em` (self); `accept_indexing_through_the_interfaces.em` (owned, self); `accept_numbers_implement_the_operator_interfaces.em` (generic-fn); `accept_operators_call_their_interfaces.em` (self); `reject_an_operator_the_type_does_not_implement.em` (self); `reject_indexing_without_the_interface.em` (self)

**TYP-22**
- rewrite 2: `reject_dyn_generic_interface_generic_method.em` — as above; `reject_generic_dyn_interface_method.em` — a method generic over a viewed parameter is now `dyn`-compatible
- respell 106: `accept_a_box_dyn_of_an_interface_requiring_clone_clones.em` (Box, clone, self); `accept_a_box_dyn_of_an_interface_requiring_eq_compares.em` (Box, self); `accept_dyn_interface_bool_literal.em` (Box, assert-c names, self); `accept_dyn_interface_box_class.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_box_class_mut.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_box_enum.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_box_enum_inherited.em` (Box, assert-c names, self); `accept_dyn_interface_box_enum_multi_interface.em` (Box, assert-c names, self); `accept_dyn_interface_box_enum_mut.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_box_generic_base_mut_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_interface_box_generic_base_shared_weak_field.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_interface_box_generic_base_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_interface_box_generic_class.em` (Box, assert-c names, generic-type, self); `accept_dyn_interface_box_generic_class_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_interface_box_generic_class_concrete_base.em` (Box, assert-c names, generic-type, self); `accept_dyn_interface_box_generic_class_drop.em` (Box, assert-c names, generic-type, mut-pattern, self); `accept_dyn_interface_box_generic_class_generic_base.em` (Box, assert-c names, generic-type, self); `accept_dyn_interface_box_generic_class_multi_interface.em` (Box, assert-c names, generic-type, self); `accept_dyn_interface_box_generic_class_mut.em` (Box, assert-c names, generic-type, mut-pattern, self); `accept_dyn_interface_box_generic_class_mut_weak_field_return.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_interface_box_generic_class_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_dyn_interface_box_generic_class_shared_weak_field.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_interface_box_generic_class_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_interface_box_generic_class_weak_field_return.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_interface_box_generic_enum.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_box_inherited_class.em` (Box, assert-c names, self); `accept_dyn_interface_box_inherited_class_drop.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_box_inherited_class_mut.em` (Box, assert-c names, mut-pattern, self); `accept_dyn_interface_enum_default.em` (Box, assert-c names, self); `accept_dyn_interface_ref_enum.em` (assert-c names, self); `accept_dyn_interface_ref_enum_mut.em` (assert-c names, mut-pattern, self); `accept_dyn_interface_scalar.em` (Box, assert-c names, self); `accept_dyn_interface_type_formation.em` (Box, owned, self); `accept_dyn_multi_interface_box_generic_base_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_multi_interface_box_generic_base_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, owned, self, upgrade); `accept_dyn_multi_interface_box_generic_base_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_multi_interface_box_generic_class_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_multi_interface_box_generic_class_mut_weak_field.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_multi_interface_box_generic_class_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_dyn_multi_interface_box_generic_class_shared_weak_field.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_dyn_multi_interface_box_generic_class_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_multi_interface_box_generic_class_weak_field_return.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_dyn_multi_interface_box_inherited_class_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, mut-pattern, self, upgrade); `accept_dyn_multi_interface_box_inherited_class_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, mut-pattern, owned, self, upgrade); `accept_expired_dyn_interface_box_generic_base_mut_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_dyn_interface_box_generic_base_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_dyn_interface_box_generic_class_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_dyn_interface_box_generic_class_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_expired_dyn_interface_box_generic_class_weak_class_field_return.em` (Box, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_dyn_interface_box_generic_class_weak_field_return.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_dyn_multi_interface_box_generic_base_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_dyn_multi_interface_box_generic_base_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, mut-pattern, owned, self, upgrade); `accept_expired_dyn_multi_interface_box_generic_class_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_dyn_multi_interface_box_generic_class_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_expired_dyn_multi_interface_box_inherited_class_borrowed_weak_argument.em` (Box, Shared, Weak, assert-c names, mut-pattern, self, upgrade); `accept_expired_dyn_multi_interface_box_inherited_class_owned_weak_argument.em` (Box, Shared, Weak, assert-c names, mut-pattern, owned, self, upgrade); `accept_expired_ref_dyn_generic_base_weak_class_field_return.em` (Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_dyn_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_dyn_generic_class_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_expired_ref_dyn_generic_class_weak_class_field_return.em` (Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_dyn_generic_class_weak_field_return.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_dyn_multi_interface_generic_base_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_dyn_multi_interface_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_mut_dyn_generic_base_weak_class_field_return.em` (Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_ref_mut_dyn_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_expired_ref_mut_dyn_generic_class_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_expired_ref_mut_dyn_generic_class_weak_class_field_return.em` (Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_ref_mut_dyn_generic_class_weak_field_return.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_ref_mut_dyn_multi_interface_generic_base_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_ref_mut_dyn_multi_interface_generic_base_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, owned, self, upgrade); `accept_expired_ref_mut_dyn_multi_interface_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_expired_ref_mut_dyn_multi_interface_generic_class_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, owned, self, upgrade); `accept_ref_dyn_generic_base_weak_class_field_return.em` (Weak, assert-c names, generic-type, self, upgrade); `accept_ref_dyn_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_ref_dyn_generic_class_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_ref_dyn_generic_class_weak_class_field_return.em` (Weak, assert-c names, generic-type, self, upgrade); `accept_ref_dyn_generic_class_weak_field_return.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_ref_dyn_multi_interface_generic_base_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_ref_dyn_multi_interface_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_ref_dyn_multi_interface_generic_class_shared_weak_field.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_ref_mut_dyn_generic_base_weak_class_field_return.em` (Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_ref_mut_dyn_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, self, upgrade); `accept_ref_mut_dyn_generic_class_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, owned, self, upgrade); `accept_ref_mut_dyn_generic_class_weak_class_field_return.em` (Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_ref_mut_dyn_generic_class_weak_field_return.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_ref_mut_dyn_multi_interface_generic_base_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_ref_mut_dyn_multi_interface_generic_base_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, owned, self, upgrade); `accept_ref_mut_dyn_multi_interface_generic_base_weak_class_field_return.em` (Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_ref_mut_dyn_multi_interface_generic_class_borrowed_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `accept_ref_mut_dyn_multi_interface_generic_class_owned_weak_argument.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, owned, self, upgrade); `accept_ref_mut_dyn_multi_interface_generic_class_weak_field.em` (Shared, Weak, assert-c names, generic-type, mut-pattern, self, upgrade); `reject_an_interface_naming_itself_that_is_not_dyn_compatible.em` (self); `reject_bare_dyn_value_parameter.em` (self); `reject_clone_where_an_interface_or_a_box_payload_is_not_clone.em` (Box, clone, message text, self); `reject_dyn_generic_interface_returning_self.em` (message text, self); `reject_dyn_generic_interface_sized_default_call.em` (clone, self); `reject_dyn_generic_interface_with_owned_receiver.em` (message text, owned, self); `reject_dyn_inherited_sized_default_call.em` (self); `reject_dyn_interface_box_class_after_transfer.em` (Box, message text, self); `reject_dyn_interface_box_generic_class_after_transfer.em` (Box, generic-type, message text, self); `reject_dyn_interface_returning_self.em` (message text, self); `reject_dyn_interface_with_owned_receiver.em` (message text, owned, self); `reject_dyn_multi_interface_ambiguous_method.em` (self); `reject_dyn_sized_default_call.em` (clone, self); `reject_eq_where_an_interface_or_a_box_payload_is_not_eq.em` (Box, message text, self); `reject_noalloc_compares_dyn_boxes_through_their_table.em` (Box, self)

**TYP-23**
- rewrite 1: `accept_owned_and_types_flow_into_a_lambda.em` — a lambda parameter word is no longer inferred: `fn(v)` against `fn(owns T)` is `E2020`
- respell 2: `accept_empty_list_arguments_of_a_generic_constructor.em` (generic-type); `accept_expected_type_solves_generic_calls.em` (generic-fn, generic-type)

**TYP-24**
- respell 13: `accept_a_bound_instance_is_chosen_or_named_in_a_generic_body.em` (generic-fn, self); `accept_a_generic_interface_named_without_its_arguments.em` (self); `accept_an_exact_instance_goes_before_a_coerced_one.em` (self); `accept_an_inherent_and_an_interface_method_share_a_name.em` (self); `accept_an_instance_of_a_generic_interface_names_the_call.em` (self); `accept_an_interface_names_its_method.em` (self); `accept_sibling_instances_associated_functions_chosen_by_arguments.em` (generic-fn); `reject_a_call_two_instances_both_fit.em` (self); `reject_an_ambiguous_call_once_and_nothing_after.em` (generic-fn, self); `reject_an_ambiguous_method.em` (self); `reject_an_interface_is_named_as_a_program_writes_it.em` (message text, self, str); `reject_an_operator_two_instances_fit_equally.em` (self); `reject_two_bounds_offer_a_method.em` (generic-fn, self)

**TYP-34**
- respell 1: `accept_view_through_a_type_declared_later.em` (str)

**TYP-35**
- rewrite 1: `reject_a_phantom_parameter_that_is_not_sync.em` — expects `E7001` (retired); `ThreadShared` in place of `@sync`/`Sync`
- respell 2: `accept_a_phantom_parameter_sets_type_identity.em` (generic-type); `reject_a_phantom_parameter_mixing_instances.em` (generic-type)

**TYP-36**
- respell 17: `accept_a_borrow_of_a_void_value_compiles.em` (clone); `accept_a_cell_of_a_copy_value_holding_a_handle_prints.em` (mut-param, self); `accept_a_class_handle_prints_its_class_and_address.em` (mut-pattern, self); `accept_a_written_display.em` (Box, generic-fn, generic-type, mut-param, self, to_string); `accept_arrays_order_lexicographically.em` (generic-fn); `accept_default_of_builtin_types.em` (generic-fn); `accept_display_debug_and_copy_bounds.em` (generic-fn); `accept_every_number_has_a_default.em` (generic-fn); `accept_mut_span_option_take_without_hash_requirement.em` (owned); `accept_shared_span_and_option_keep_structural_hash.em` (generic-fn); `accept_tuples_order_and_references_implement_as_their_referent.em` (generic-fn, to_string); `accept_two_boxes_compare_what_they_hold.em` (Box, generic-fn, self); `accept_void_has_a_default.em` (generic-fn); `reject_a_tuple_or_reference_without_its_parts_interface.em` (generic-fn); `reject_an_array_whose_element_is_not_ord.em` (generic-fn); `reject_bounds_the_table_does_not_meet.em` (generic-fn); `reject_mut_span_does_not_inherit_its_elements_hash.em` (generic-fn)

**TYP-38**
- respell 2: `accept_a_str_key_makes_a_map_of_str.em` (str); `accept_class_handles_in_literals_and_boxes.em` (Box, Shared)

**TYP-39**
- respell 4: `accept_a_dyn_value_inside_another.em` (Box); `accept_box_and_cell_print_as_what_they_hold.em` (Box, to_string); `accept_collections_print_as_python_shows_them.em` (str); `reject_a_dyn_value_without_debug_inside_another.em` (Box, message text, mut-param, self)

**TYP-5**
- respell 4: `accept_a_mut_parameter_is_auto_borrowed.em` (generic-fn, mut-param, mut-pattern, self); `accept_a_place_is_auto_borrowed_for_a_ref.em` (generic-fn, str); `reject_writing_a_place_its_auto_borrow_still_reads.em` (message text); `reject_writing_through_mut_self_while_it_is_auto_borrowed.em` (message text, mut-pattern, self)

**TYP-9**
- respell 4: `accept_a_default_keeps_its_declarations_float_mode.em` (self); `accept_a_mut_default_keeps_its_declarations_float_mode.em` (mut-param, self); `accept_a_relaxed_function_is_built_in_its_own_unit.em` (fp-contract, self); `reject_a_float_mode_on_a_method_without_a_body.em` (fp-contract, self)


## 7. Error codes and error pages

Measured by `target/gap-0.9.15/07/codes_check.py` (the `codes.rs` test re-done in Python against the
pinned target) and `p17a_diff.txt` (the two code tables compared row by row).

**The registry today against 0.9.15** (`compiler/ember_diag/src/codes.rs`: 261 codes; target: 265 live
rows, 33 retired, 298 codes named):
- Codes the target names and the registry lacks (34): E0113, E0114, E0115, E0116, E0117, E0118, E0120,
  E2037, E2106, E2108, E2112, E2114, E2115, E2116, E2117, E2118, E2119, E2123, E3028, E3031, E3032,
  E5013, E5069, E7007, E7008, E7012, E7013, E7014, E7015, E7016, E7017, E7018, E9014, W3013. Of these
  this area emits E0113-E0120, E2036/E2037/E2106 (opening and kind tests), E2114, E2115, E2116, E2118
  (with LT), E9014, W3013; the rest are other areas' (E2108 CLS, E2112 CLS, E2117 CLS/EFF, E2119 WK,
  E2123 OWN, E3028 FN, E3031 BRW, E3032 WK, E5013/E5069 FFI, E70xx THR).
- Registered codes citing a rule their 0.9.15 row does not (9): E2173 (`[GRM-8b]` → `[TYP-45]`,
  `[IFC-5]`, `[OBJ-2]`), E3023 (`[BRW-1]` → `[BRW-14]`, `[BRW-15]`), E3063 (`[TYP-15]` → `[LT-45]`,
  `[BRW-19]`, `[ALC-1]`), E3095 (`[DSJ-1]` → `[DSJ-4]`), E3060 (`[BRW-8]` → `[BRW-17]`, `[LT-3]`), E3062
  (`[LT-1]` → `[LT-1a]`), E5060 (`[FFI-39]` → `[FFI-39c]`), E7004 (`[THR-11]`, `[FFI-22]` →
  `[THR-17]`, `[THR-10]`, `[FFI-21]`).
- Newly retired (19), still registered so never reused, each to stop being emitted by the slice that
  removes its feature: E2228 (this area, A7-8), E2229, E3012, E3025, E3030, E3041, E3042, E3061, E3065,
  E3080, E7001, E7003, E7005 (emitted by this area's parser for `+ Sync`, `ember_parser/src/decls.rs:1635`;
  goes with `thread_shared fn()`), E7020, L3001 (A7-22), L3011, L3013, L3014, W2111. Emission sites
  found by `grep codes::<code>`: 32 across the workspace.
- Rows whose meaning changed (59): E0100, E0104, E0109, E1010, E1030, E2020, E2031, E2036, E2040, E2050,
  E2072, E2100, E2102, E2103, E2104, E2140, E2150, E2173, E2200, E2221, E3010, E3011, E3013, E3014,
  E3016, E3020, E3021, E3022, E3023, E3024, E3026, E3027, E3040, E3050, E3060, E3062, E3063, E3095,
  E3100, E3105, E4030, E4072, E5010, E5015, E5020, E5054, E5060, E5063, E5064, E5065, E7002, E7004,
  E7010, E9030, L2001, L3002, L3015, L3017, W2190. Those this area owns the meaning of: E0100, E0104,
  E0109 (syntactic part), E1010, E2020 (lambda words, `weakref` use, wrong manager), E2036, E2040,
  E2050, E2072, E2150, E2173, E2200 (with OWN), E3105, E4072, L2001, L3002, W2190.

**Error pages** (`docs/errors/`, 177 pages; `tools/error_pages.py` builds both examples of each):
- retire 21 (their codes are retired): E2228, E2229, E3012, E3025, E3030, E3041, E3042, E3061, E3064,
  E3065, E3080, E7001, E7003, E7005, E7020, E9033, L3001, L3011, L3013, L3014, W2111;
- rewrite 16 (the code's meaning changed): E2100, E2104, E2150, E2200, E3013, E3016, E3020, E3023,
  E3026, E3027, E3095, E4030, E5060, E7004, E7010, L3017;
- respell 42 (0.9.10 spellings in their examples; the migrator of A7-9): E0110, E2042, E2043, E2050,
  E2101, E2102, E2103, E2110, E2111, E2120, E2121, E2221, E2223, E2260, E3010, E3011, E3015, E3024,
  E3070, E3105, E4001, E4002, E4003, E4020, E5015, E5018, E5020, E5055, E5056, E5057, E5058, E5059,
  E5065, E7011, E9041, L2001, L2002, L4002, L5001, L5002, W2190, W2220;
- keep 98 (no 0.9.10 spelling found in their examples by the script; their prose was not read);
- new: a page for each of the 34 new codes when the compiler first emits it (the gate requires pages
  only for emitted codes, `tools/error_pages.py:61`).

## 8. The spec-tied Rust tests and tools at the new target

| check | reads | red today because | green after |
|---|---|---|---|
| `codes.rs` `the_registry_and_the_specification_name_the_same_codes` | the pinned target's XVII.9 | 34 unregistered codes, 9 citations | A7-1 (M0) |
| `shapes.rs` `every_ownership_code_has_a_shape` | the registry | green now; would go red when A7-1 adds E3028/E3031/E3032 without shapes | A7-1 (same commit) |
| `milestones.rs` `help_lists_every_command_and_flag_of_the_specification` (`:1150`) | XVII.1's listing | `--view` is missing (`--borrow` today) | A7-2 (M0) |
| `milestones.rs` cost test (`:925-962`) and `tools/rule_index.py` `[COST-5]` | `[COST-3]`'s table | 6 labels with no row | A7-2 (4 labels, M0); EXC and CELL removal (2 labels) |
| `milestones.rs` `the_three_versions_are_separate` (`:7646`) | `**Version:**` | `LANGUAGE_VERSION` is `0.9.10` (`ember_parser/src/lib.rs:21`) | A7-8 (M1) |
| `milestones.rs` `annex_a_is_generated_from_a_fixture_that_compiles` (`:7680`) | `p19-annex-a.md`, `appendix-a.em` | the fixture is 0.9.15 code | A7-25 (M9) |
| `milestones.rs` `every_example_of_the_specification_checks` (`:8697`) | 49 ` ```ember ` blocks | 0.9.15 code | baselined at M0 (A7-3); empty baseline at M9 |
| `milestones.rs` test runner (`parse_expectations`, `:123-290`) | `#$` keys | refuses `#$ count_bits:` | A7-17 (M6) |
| `guide.rs` `every_sample_of_the_guide_runs...` | `docs/book/` | green today; red the moment M1 changes the grammar | A7-9 respells the guide in M1 |
| `tools/spec_check.py` | `ember-spec.md` (0.8.5) | green today; red when M1 drops 0.8.5's spellings | A7-3 (reads the pin) |
| `tools/rule_index.py` registry citations | adopted spec + pinned target | green; stays green (every new citation is in the target, checked) | - |
