# M6a: calls checked by the summaries; one name for one variable; messages that name the chain (plan, 2026-11-20)

MILESTONES.md's M6a row: A4-4 to A4-7 (`gap/04`), A2-10, A2-13 (`gap/02`), A1-21, A1-22 (`gap/01`), A3-6, A3-7, A3-8
(`gap/03`), A7-14 (`gap/07`); 5 L, 6 M, 1 S. The bank written for it is `tasks/96/redesign/jobs/ahead/M6a/` (INDEX.md:
44 rules, 119 files); a rule's folder moves to `tests/conformance/<RULE>/` when it passes. Spec: Hardened_14. A2-9 (the
borrow facts) went into M2's A4-2 (`M2-plan.md` slice 4, built); what is left of it is steps 1-2 here. A2-11 is not in
this row: its `RefCell` part is M6b's, and the run-time access machinery goes with A4-8 in M6b, after these checks
(SD8). Written by a read-only planner: nothing was built or run but `target/ci/ember.exe check` on copies of the bank.

## Where it stands (probed 2026-11-20)

- **The facts exist and nothing reads them.** `ember_analysis/src/summaries.rs` (A4-2, M2 slices 2-4): changes, reads,
  reshapes, adds, removes, ends, `used_after`, `read_while`/`changed_while`, `lets_this_out`, each `Fact` with a chain
  of (callee body index, call span). Only `--emit mir` computes them (`ember_driver/src/lib.rs:6131`). M2's slices 1
  (in the pipeline, kept, verified), 5 (`.emif`) and 7 (`static mut` with `E7012`) are open: "they go with the first
  check that reads a summary", which is this milestone (W8's FN-9 was reverted to wait for it).
- **Today's `summarize` cannot go into the pipeline as it is.** It re-walks every body each round until nothing grows
  (`summaries.rs:178-204`), keeps every fact twice (`known`), clones a `String` per static fact (`Root::Static`), and
  `borrows::regions_all` (`borrows.rs:177`) holds every body's `Regions` at once: a second region inference of the
  whole program and a peak set by the program, not by its biggest module (ODR-281, `tools/memory_curve.py`).
- **Calls today:** seen through region contracts and the argument borrows only (`check_call_activation`,
  `borrows.rs:4750`). A method's receiver is narrowed by `receiver_fields` (`borrows.rs:2294`) for private struct and
  enum methods only (`borrows_used_fields`, `ember_typeck/src/lib.rs:28426`); a class method borrows its whole object.
  A position survives only a built-in add to the first argument (`positions::only_adds`, `positions.rs:209`).
- **The bank against today's compiler:** 57 of 119 files use `static mut` and stop at `E1010` (M2 slice 7). Of the
  62 others, 31 give the expected codes today (texts and outputs not checked); 31 do not:

| Cause | Files | Step |
|---|---|---|
| class objects checked while running, not when compiling (EXC-1, -2, -3, -16, BCK-7.1, RC-5) | 7 | 12, 13 |
| a method borrows its whole object (BRW-10, EXC-19), its object not reserved (BCK-5 `c.add(c.n)`) | 3 | 5 |
| a changing method through a read-only name missed (FN-9.4; CLS-7.1 gives 3 of 4) | 2 | 5 |
| a position across a written function's call: `E3022` at the call | 2 | 8 |
| stored functions, function values (BRW-21.9, DIA-26, CLO-17 x2) | 4 | 9 |
| a call's two `ref mut` elements under `for j in i + 1..` (M5's proof misses it); spans copied; one variable twice | 3 | 10 |
| regions through calls (LT-35 accept `E3060`; LT-47 accept is A2-8's callee side) | 2 | 15 |
| other milestones: capture words (CLO-18 x3, M8), Q8's `this.handlers[i](…)` and `E2103` (CLS-7 x2), `sharedref mut` (EXC-17.2, M5's A2-8), `assert_disjoint` (DIA-11, M6c), a cascade `E3021` (DIA-25.2) | 8 | 16, later |

## Progress (the lead)

- **Step 1, lean (2026-11-20):** `summaries::summarize_flow`, stage 1 with no region facts, a worklist over the
  callers (a body walked again only when a callee's summary grew), worked out once per build in `run_checks` and
  given to each `Form` (`Form::summaries`); each fact keeps the line that does it (`Fact::at`). Not yet: the
  strongly connected groups, bounded facts, keeping in `BorrowFacts`, `[VERIFY-3]`, and the measurement
  (`tools/compile_budgets.py`, `tools/memory_curve.py`) before it reaches main. `positions` works them out a
  second time in the driver (a `ponytail:` note).
- **First consumers (parameters only; statics wait for step 0):** a `ref mut` given to a written function whose
  summary reshapes what it points at is a reshape (`reshapes_at`: `E3060` at a view's later use, `wipe(ref mut
  xs)`, `xs.retain(keep)`), "may move" when it only adds; a position survives a written function that only adds
  (`positions::added_arguments`, any argument); `E3020` through a written function names its verb and chain:
  "`note(ref mut xs)` (line 16) adds to `xs` (through `grow`, line 8) while this loop walks `xs` (line 15)".
- **Step 0's `E7012` (2026-11-20):** `summaries::check_module_variables_on_threads`, at each `thread.spawn` and
  `jobs.submit` outside a scope: the bodies the call runs (a named function, a function value's targets) and the
  `drop`s what it is given may run (`drop_runs`), by their summaries; the spec's text, the declarations written by
  `ember_diag::guarded_static_mark` from the program's own lines. A method is named by its type (`Ticket.drop`,
  `TICKET_LOCK`), a lambda by the variable (`FRAME_LOCK`). Not yet: `guarded by` (M9), C's callbacks and
  `export(threads=...)` (M12).
- **Step 4, both halves (2026-11-20):** one walk of what ending a value runs (`drop_runs.rs`: a type's own `drop`,
  its parts', a class's and those below it, a lambda's state's, what a `dyn` box or an interface handle was made
  from, the object a counted link may end; `effects::drop_of` reads it too, so `Box[dyn I]`'s end is its values'
  `drop`s, not `Effects::ALL`). A `StmtKind::Drop` and a clear, a truncation or an SoA element ended in place are a
  call of each `drop` in the summaries (`ended_facts`: a `drop`'s facts on statics, and those leaving its object
  through a pointer; what it does to its own object ends with it).
- **Step 3, statics (2026-11-20):** `summaries::runs_of` (each checked form: its calls' bodies, its ends' `drop`s)
  and `borrows::summary_accesses`: what they do to module variables is an access at the call or the end, met with
  the body's loans (`check_point_with`, a `Cause` for the text): "`tick()` (line 19) adds to `LOG` (through
  `note`, line 10) while this loop walks `LOG` (line 17); call `tick()` after the loop: move line 19 below the
  loop", and DRP-8's "`enemies.clear()` (line 16) ends every `Enemy` in `enemies`, and an `Enemy`'s `drop` pushes
  to `LOG` (its line 22), ...". Facts carry their built-in's verb (`Fact::op`). Parameters' facts still meet the
  loans where the arguments are borrowed (`reshapes_at`); other conflicts than a walk's keep the generic texts
  until step 7.
- **Step 6's plan (the lead, 2026-11-20):** the call check reads stage 2 (`used_after`, `read_while`,
  `changed_while`), which needs every body's regions (`summaries::summarize` already makes them with
  `borrows::regions_all`). `regions_all` builds again the contracts, capture contracts and store summaries
  `run_checks` already has; so `run_checks` infers each body's regions once, with its own `call_contract`, and
  gives them to both the worklist (`summarize_flow_with` with regions: `body_facts` already takes them) and
  `check_body` for a body's plain form (no `dyn` call made direct, the common case); only the other forms infer
  their own. Measured with `tools/compile_budgets.py` (B1, B3, B9) before it lands. `check_call_summary` after `check_call_activation`: each
  argument's loans (`regions.holders` of its temporary) against the callee's `used_after(i)` (E3031 with
  BRW-21's three texts: a view, the method's object, a `ref mut` read by another name while used); DIA-28
  orders E3031 before E3021 on one line. Bank: BRW-21 (10), FN-9.1, EXC-15.1 (2), CLS-7.2, DIA-22 (after its
  E3028 case moves to a file of its own: a type error stops the borrow check, and the spec does not say it
  should not), DIA-28, BCK-8.2.

## 1. The summary

What each function's summary carries (`summaries.rs`; stage 1 needs no regions, stage 2 does):

```rust
pub enum Root { Param(u16), Static(u32), Linked(ClassId) }  // Static: the program's static table; Linked: step 12
pub struct Reach { root: Root, len: u8, path: [Step; MAX_PATH] }     // inline: no heap per fact
pub struct Fact {
    reach: Reach,
    op: Op,                    // Push, Extend, Insert, Clear, Remove, Pop, Truncate, Sort, …, Write, Replace, Drop, HandOver
    at: Span,                  // the line that does it, in the last function of the chain; every hop keeps it
    via: Option<(u32, Span)>,  // the first call it comes through; the rest is found again (below), not stored
}
pub struct Summary {
    changes, reads, reshapes, adds, removes, ends, lets_this_out: Vec<Fact>,   // stage 1 (as today)
    used_after: Vec<(u16, Fact, (Span, UseKind))>,   // stage 2: the parameter's next use after the line: read/change/write
    read_while, changed_while: Vec<(u16, Fact, Span)>, // stage 2: the use that keeps the `ref mut` parameter live
    ends_families, changes_families: Vec<ClassId>,    // step 13: families whose objects it may end or change
}
```

Worked out, not stored: the fields of `this` it changes and reads (the first `Field` of `Param(0)` facts),
`only_adds(reach)` (exists), "changes its object". The chain text ("(through `note`, line 4)", the handler form) is
rebuilt on demand by following `via` into the callee's facts with the same `at`, as `effects.rs` `Graph::chain`
(`effects.rs:997`) does: one hop per fact in memory, any length in a message. A fact whose place lies inside another
fact of the same kind and root is not added; past a cap per (root, kind) the facts widen to their common prefix (only
refuses more): the facts per body stay bounded by its parameters and statics, whatever the program's size.

**Where it is computed.** `run_checks` (`borrows.rs:215`), the borrow check's own driver, so `Regions` are inferred
once per body:
1. **Stage 1** (`summaries::summarize_flow`), before `closure_capture_contracts` (`borrows.rs:2472`), whose receiver
   narrowing reads it: over the strongly connected groups of one call graph (`targets.rs` gains `call_edges`, the
   callees `body_facts` resolves today, `summaries.rs:543-603`; Tarjan's lifted out of `regions::loop_groups`,
   `regions.rs:2710`, into one helper), callees first; inside a group the worklist of `effects::fixed_point`
   (`effects.rs:1039`). A body is walked once, again only when a callee in its own group grew.
2. **Stage 2**, inside `check_forms` (`borrows.rs:340`): bodies run level by level (a body after its callees' groups;
   `crate::parallel::each` per level, the 16-thread limit kept); each form's `Regions` from `check_body`
   (`borrows.rs:3719`) give its stage-2 facts (the union over forms, so no extra inference), then the body's checks run
   with its callees' summaries complete, then its `Regions` are dropped. A recursive group's members iterate together.
   `regions_all` is deleted; `--emit mir` prints the installed summaries.

**Where it is kept.** `BorrowFacts` (`borrows.rs:154`) gains `summaries: HashMap<String, Summary>` beside `stores`,
with `via` by symbol in the record (P2). Incremental (`ember_driver/src/incremental.rs`): `Form` and `KeptFacts` carry
them as they carry `stores`; a kept body's summary is its record's, never inferred again, as
`install_callable_regions_kept` (`borrows.rs:112`) does for contracts; `facts_of` (`incremental.rs:857`) hashes
(contract, stores, summary), so a kept caller of a body whose summary changed is checked again (`Plan::verify`,
`Attempts::again`); a kept body's record keeps the targets of its calls through values and `dyn` (as `kinds` does,
`borrows.rs:337`), so a function stored anywhere new checks it again. Spans relocate with the record's
(`ember_span::codec::Relocation`). `SCHEMA` 5 to 6. `[VERIFY-3]`: in debug builds (the `ci` profile) the summaries are
made again from the MIR and compared, as `verify_callable_regions_or_panic` (`ember_driver/src/lib.rs:314`) does. The
`.emif` record is step 17.

## 2. Consumers, and where each plugs in

| Consumer | Code, rules | Plug point | Step |
|---|---|---|---|
| a call's facts as accesses | all below | `check_body`'s `Terminator::Call` arm (`borrows.rs:3910`): new `call_accesses` maps each callee fact (the union over `Targets` for a value, `dyn` or `virtual` call; a function-value argument's targets for a built-in that runs it) onto caller places, `Param(i)` through argument i (`summaries::mapped`), `Static(s)` onto each in-scope loan whose root local points at `s` (`pointers_of`'s `Target::Static`); each access carries its cause into `check_point` (`borrows.rs:4833`) | 3 |
| a walk and a call that may change it | E3020; BRW-18, EFF-25, DIA-27, EFF-26, DSP-5, CLO-17.1, BRW-10.2 | `check_point`'s walk branch: `walk_change_message` (`borrows.rs:5900`) gets the chain form: "`tick()` (line 13) adds to `LOG` (through `note`, line 4) while this loop walks `LOG` (line 11); call `tick()` after the loop: move line 13 below the loop" | 3 |
| a call that ends or reshapes what a live view points into; one that changes what it reads | E3060, E3021; BRW-17, BRW-1, BCK-3, EXC-16 | `reshaped` (`borrows.rs:5118`) is `reshapes_at(...)` or a reshaping cause; the E3021 branch (`borrows.rs:5233`) names the chain | 7 |
| a written function that reshapes (`retain`, `wipe(ref mut xs)`) | E3060 | the same `reshaped` test: the argument's `ref mut` to a callee whose summary removes ends a position or an address (`E3060` at the later use, not `E3022` at the call) | 7, 8 |
| positions across a written function that only adds | BRW-16, BCK-8.1 | `positions::only_adds` becomes `added_list(func, args, summaries) -> Option<usize>` (any argument, not only the first; a static it adds to by name, Q-E); `plan` (`positions.rs:149`), `repointed_add_at` (`:187`); `repoint_positions_all` (driver `lib.rs:5821`) is given the summaries | 8 |
| the call check | E3031; BRW-21, EXC-15, FN-9.1, DIA-22 | new `check_call_summary` after `check_call_activation` (`borrows.rs:3911`): argument i's loans (its temporary's, `regions.holders`) against the callee's `used_after(i)`, and for `ref mut` its `read_while(i)`/`changed_while(i)`; the method's object is argument 0; E3031 over E3021 on one line (DIA-28) | 6 |
| the arguments against each other | E3022, E3021; BRW-11, BCK-5 | `check_call_activation`: a function-value argument's targets' changes are accesses against the reserved changeable arguments (`LOG.retain(keep)`); the method's object reserved, then activated | 5, 10 |
| one name for one variable | BRW-11.4, MONO-10, BRW-21.8, BRW-22.2 | `one_whole_variable(body, args, callee) -> Option<Merge>` in `check_call_activation` and `check_call_summary` exempts the call; the copy is a pass (`ember_analysis/src/one_name.rs`) | 10, 11 |
| what a method does to its object | BRW-10, EXC-19, A4-7 | `receiver_fields` (`borrows.rs:2294`) reads stage 1 for every method (class, `pub`, `virtual` by its overrides); `NarrowedBorrow` (`regions.rs:188`) gains `reads`, so changed fields are lent `ref mut` and read ones `ref` | 5 |
| a stored function | E3022, E3031 at the store; CLO-17.2, BRW-21.9, DIA-26 | `targets.rs` keeps each place's store lines; a call through a value checks each target alone and reports a failing one at its store; at a store, the function's facts against the holder's place | 9 |

## 3. The order

Each step lands alone: its bank folders move as they pass (`python tasks/impl-0.9.9/annotations.py
tests/conformance/<RULE>`), the old tests named follow the spec, one quick check before each merge to `main`. A rule
is credited when all its clauses pass; most need several steps (INDEX §0 says which other milestones too).

0. **M2's slice 7 first (P1): `static mut` as a place, and `E7012`** through stage 1's spawn and job edges (a
   `static mut` reached by a function a non-scoped thread may run). Unblocks 57 bank files. Old: STA-1, STA-3, FN-8.
   The static's place must be one root the borrow check can compare (two temporaries from `LazyStatic(LOG)` are one
   place): say so in its design, since step 3 maps `Static` facts onto it.
   **Built (A1-17, 2026-11-20), but for `E7012`.** A use of a static is `Deref` of a compiler temporary its
   access fills (`_t = lazy_static()`, `constant_static()`, `foreign_static()`; each use makes one). The borrow
   check maps every such temporary to the first one of the same static in the body
   (`borrows::module_variable_roots`, kept per form in `Form::statics`, applied in `call_argument_place`), so
   `(*_7)` and `(*_2)` are one root; `positions::list_of` takes the root too. Step 3 maps a callee's
   `Root::Static(s)` fact onto the root of `s` in the caller's map (a caller that never names `s` holds no
   loan of it). A `static mut` with no heap memory is the image's writable data (`TypeTable::
   is_writable_static`, a plain C global, LLVM `internal global`); one that owns heap memory, or whose
   initialiser reads a `static mut`, is built on first access. Not built: `E7012` (no spawn or job edges
   exist yet; stage 1's), `guarded by` (M9), and a call that changes a module variable by its name while
   the caller holds a view of it (steps 3 and 7).
1. **The summaries in the pipeline (§1), with nothing read yet.** Stage 1 and 2 as §1, kept and verified. Tests: the
   `--emit mir` summary tests unchanged; a new incremental test (kept summaries equal a clean build's; `note`'s two
   lines swapped checks `main` again). **Measured before it lands:** `tools/compile_budgets.py` (B1, B3, B6, B9) and
   `tools/memory_curve.py`, the before and after compilers interleaved: no figure worse beyond its spread (`[BUD-3]`),
   the curve flat (ODR-281). Bank: none.
2. **The facts messages need:** `op`, `at`, `via`, the next use of a parameter (§1); `summaries::chain_text`.
   Tests: `--emit mir` lines, unit tests. Bank: none.
3. **E3020 through calls** (A4-4; A2-10's BRW-18 part): `call_accesses`, the chain message. A `ref mut` argument stays
   a change by its word (Q-H); one message per loan and call (`reported`, `borrows.rs:4957`), the summary's first.
   Bank: EFF-25 (25.1, 25.3, 25.2's E3020), DIA-27, BRW-18, EFF-26 (4), DSP-5 (2), CLO-17.1 (2), DIA-22's E3020. Old:
   `BRW-18/reject_a_call_that_grows_the_list_its_loop_walks`, `BRW-10/reject_a_method_using_the_iterated_field` keep
   E3020 with the chain text.
4. **A `drop` counts as a call** (A1-21): `StmtKind::Drop` (`summaries.rs:521`) and the built-ins that end elements
   (clear, truncate, a replacement) add the summary of each `drop` the type reaches (`effects.rs` `drop_of`, `:1398`,
   lifted to one function both read); a `drop`'s facts on its own object are dropped. Bank: DRP-8 (3).
5. **A method's object from its summary** (A4-7): `receiver_fields` and `NarrowedBorrow` as §2; the object of a
   changing method reserved then activated (BCK-5); `borrows_used_fields` goes for this package's and std's methods (a
   dependency's `pub` method keeps its whole object until step 17); find why FN-9.4's `p.change()` and one of CLS-7.1's
   four names give no `E3023` (typeck `objects.rs` decides the mode). Bank: BRW-10 (3), EXC-19, BCK-5 (2), CLS-7.1 (2),
   FN-9.4 (2), EXC-15's accept. Old: BRW-10's five and BRW-4's E3022 cases re-run; none expected to change.
6. **The call check E3031** (A4-5): `check_call_summary`, BRW-21's texts. Bank: BRW-21 (10 of 14: not 21.8, 21.9's
   two, 21.3's `is not` accept), FN-9.1, EXC-15.1 (2), CLS-7.2's Q6 file, DIA-22, DIA-28, BCK-8.2.
7. **A call that reshapes, ends or changes what a live view reads** (A2-10's BRW-17 and BRW-1 through calls). Bank:
   BCK-3 (2), EXC-16's `reset`, EFF-25.2's E3021 and E3060, BCK-1 and BCK-2 (codes pass today: credit when texts do).
8. **Positions survive a written function that only adds** (M5's left item): `added_list`; a callee that removes
   ends the position. Bank: BCK-8.1 (2), BCK-6.
9. **Function values: the fault at the store** (A4-4/A4-5's CLO and BRW-21.9 halves): store lines in `Targets`;
   per-target checks; CLO-17.2's store refusal by the function's facts mapped through its environment
   (`summaries::task_mapped`, `summaries.rs:712`), with INDEX Q3 and Q4. Bank: BRW-21.9 (2), DIA-26, CLO-17.2 (2;
   Q3's file alone), BRW-18's store. Old: CLO-2's 4 rewrites (gap/04 §1b).
10. **Arguments against each other, and one whole variable** (A2-10's BRW-11, A2-13): function-value arguments in
    `check_call_activation`; find why M5's `for j in i + 1..` proof misses a call's two `ref mut` elements;
    `one_whole_variable`, and E3022 with MONO-10.4's text where no body can be had. Bank: BRW-11 (8), MONO-10.4 (2),
    BRW-21.8, and MONO-10.1-10.3 and BRW-22.2, which print the same with or without the copy until noalias (M6c). Old:
    `BRW-11/reject_overlapping_mut_arguments` becomes a run-pass; `OWN-6/reject_swap_of_the_same_place` by Q-J.
11. **The copy** (A1-22): `one_name.rs`, after the borrow check, over every body (a kept one too: its marks found
    again from its MIR and the summaries, never from the check): the callee cloned with the two places one, named as
    an instance (Q-D), deduplicated by (callee, pair), copied down the chain (MONO-10.3), the call retargeted. Test:
    `--emit mir` or assert-c that the call names the copy. `ember inspect`'s list waits (not in the bank).
12. **Class objects checked when compiling** (A4-6, A3-8's "may be one"): `overlaps` (`borrows.rs:5645`) asks one
    query: two places through handles of one class family, one through a link or a kept view, unless known different
    (other families, a fresh object, the line under `if a is not b:`, `index_proofs.rs`'s dominating-arm test reused);
    `Root::Linked(family)` facts mapped at calls; the `is not` fact also clears `used_after` (BRW-21.3). The run-time
    words stay until M6b. Bank: EXC-1 (2), EXC-2 (2), EXC-3 (2), EXC-16, EXC-17.1, BCK-7.1 (2), BRW-21.3's accept.
    Old: the 13 run_fail files of EXC-1, EXC-2, EXC-3, EXC-19 and CLS-7's 2 move to compile time (gap/04 §1b).
13. **Views through links** (A3-7): `ends_families`/`changes_families`; a link view across a line or call that may
    end its family is E3060 at the later use, across one that may change it E3021 (BCK-7.2); `keep_handle_alive`
    and the write-backs go (`ember_mir/src/lower.rs:7858` and its callers, ODR-065): a codegen change, benched before
    the merge. Bank: RC-5 (2 of 3; the stored-function file is M8's), BCK-7.2 (2). Old: RC-5 (3 respell, 3 rewrite,
    1 retire), BCK-7 (2 rewrite).
14. **A `drop`'s object through calls** (A3-6's second half): `check_drop_self_escapes_all` (`drops.rs:502`) follows
    `lets_this_out` and stores of `this` through callees, with the chain; WK-3's count 0 before `drop`, the run-time
    test deleted. Bank: OBJ-5 (2 of 3; `thread_shared this` is M9), WK-3.
15. **Regions through calls** (A2-10's LT parts): a by-value struct argument needs only the regions of the fields
    the callee's contract reads (`regions.rs:1832-1850`, `:2234-2245`), through a value by the union of its targets'
    contracts. Bank: LT-35 (2), LT-47 (2 of 3).
16. **The message framework** (A7-14): DIA-28's precedence per line in `ember_diag::Sink`, DIA-26, DIA-7's shape
    helps (B1, B2, B3, B4, B7, B16), DIA-2's names as written (`TypeTable::written`), DIA-9/10's helps. Bank: DIA-2
    (INDEX Q10), DIA-3, DIA-7, DIA-9, DIA-10, DIA-25.2, DIA-28. The runner that applies each fix: later.
17. **Summaries in interface files** (A4-3, M2 slice 5; may run beside the others, it touches `ember_build` only):
    `CallableInterfaceContract` (`ember_build/src/interface.rs:200`) gains the summary (hops as symbol and line),
    `component_hashes` (`:820`) hashes it, `SCHEMA_VERSION` 15 to 16; `borrows_used_fields` goes for dependencies
    too; MONO-10's carried bodies (`[BLD-2]`). Test: the EFF-4 test with a library's two lines swapped. Bank: none.

## 4. Risks

- **Compile time and memory.** Every build now pays stage 1 over the whole program and stage 2 per body. Bounds kept:
  one region inference per body (today's `regions_all` would make two, and hold all); `Regions` of one level's bodies
  at a time; facts inline, subsumed and capped; a body re-walked only when a callee in its own group grew. Not known
  without a build: how deep std's call levels go (a deep chain serialises `parallel::each`; the fallback is stage 2
  over small per-body liveness tables instead of levels) and what the cap costs in false refusals. The debug-only
  verification doubles the cost in the `ci` profile, as the region verifier does. Nothing changes at run time but step
  8's buffer re-read after a written add (what correct C does), step 11's copies (code size), step 13's removed copies.
- **Coarse function-value targets** (gap/04 risk 1): one `fn()` stored anywhere counts at every call of every `fn()`.
  The spec's rule; measure the false refusals on `tests/`, `examples/` and std after step 3.
- **Incremental correctness.** A kept summary is trusted while its callees' facts hash the same and its targets are the
  same; a missed input (a new store into a callable type in a fresh module, a `drop` added to a type a kept body ends,
  an override added) is a stale fact and a wrong verdict. The `[VERIFY-3]` recompute catches it in ci builds; add an
  incremental test for each of those three inputs. An edit that changes a widely called function's summary re-checks
  every caller (B1 slower), as EFF-4 requires.
- **Holes become miscompiles in M6c.** A fact missed here only accepts a wrong program while the run-time checks stand
  (until M6b); with noalias (M6c) it is undefined behaviour in C. Before M6c: a test set that tries to break BRW-21
  (stored functions, virtual calls, generators, `drop`s, foreign callbacks) (gap/02 risk 2).
- **Message churn.** Steps 3, 6, 7, 12 and 13 move old tests' codes and texts; each step runs the folders it can reach.
- **Static places** depend on step 0's lowering; **`dyn_kinds` forms** give stage 2 per form (unioned): a body checked
  in no form (a kept one) has only its record's facts.

## 5. Questions

Planning questions for the lead (P), then spec questions (Q, none decided; INDEX §3's Q1-Q11 also apply: Q2 and Q9
decide the chain texts of steps 2 and 9).

| # | Question | Readings | Recommended |
|---|---|---|---|
| P1 | Step 0 is M2's slice 7, not in M6a's row; 57 bank files wait on it | (a) first in M6a; (b) its own milestone slot before | (a): no other order lets the bank run |
| P2 | Where the summary lives: M2-plan slice 1 says on `Body`, serialised with kept MIR | (a) `Body`; (b) `BorrowFacts`, as `stores` | (b): one pattern for kept facts, `facts_of` hashes it, `ember_mir` gets no analysis types |
| Q-A | E3031's "its line 3" when the end is in the callee's callee (`note` calls `reset`, which clears `LOG`) | (a) `note`'s line of `reset()`, "(through `reset`, line M)"; (b) `reset`'s line | (a): "its line" is `note`'s, and DIA-27 names the chain |
| Q-B | which later use E3031 names when the parameter is used more than once after the end | (a) the first; (b) the last | (a): the one the fix "use `s` before `LOG.clear()`" moves first |
| Q-C | the verb: "`tick()` adds to `LOG`" (EFF-25) but "`log_click`, which pushes to `LOG`" (CLO-17), "`note` clears `LOG`" (BRW-21) | (a) after a call line, the kind of change (adds to, removes from, changes, ends); after a function's name, its line's own operation; (b) always the operation | (a): it fits all five spec texts |
| Q-D | MNG-6 gives no canonical spelling for a MONO-10 copy, which packages must agree on to deduplicate | (a) one more level after the arguments naming the merged pair (`[=0,1]`, a static by its path); (b) a component of its own | (a): an instance like the others, hashed by MNG-1 |
| Q-E | a position of `ENEMIES[0]` across a call that only pushes to `ENEMIES` by name: BCK-8 and EFF-24 say "a list parameter"; BRW-16 counts module variables and says the list may grow | (a) kept; (b) ended (E3060) | (a): BRW-16 is the rule on positions; BCK-8 names one case |
| Q-H | `look(ref mut xs)` in `for x in xs:`, where `look` only reads: EFF-25 says the summary decides E3020; BCK-5 and BRW-11 call a `ref mut` argument changeable by its word | (a) E3020 as today; (b) allowed by the summary | (a): the word is the promise the caller reads, and BRW-11 lists `ref mut` apart from "a changing method's object" |
| Q-J | `mem.swap(ref mut v, ref mut v)`: MONO-10 refuses only where no body can be had and does not name built-ins | (a) allowed, the one-place copy of `swap` does nothing (gap/01 A1-22); (b) E3022 as today | (a), with a P-row in MONO-10 naming built-ins |
