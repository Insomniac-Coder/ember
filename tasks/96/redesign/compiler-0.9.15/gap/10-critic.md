# Gap map 10: completeness critic of reports 01 to 09

Read-only. Nothing was built or run. The scripts that did the counting are in `target/gap-0.9.15/10/`:
- `cov.py`: H.41 rules against every report's rule-table rows;
- `codes.py`: `p17a-codes.md` against the 0.9.10 table, and each changed or new code against the reports;
- `folders.py`: conformance folders of H.41 rules against the reports;
- `spell.py`: removed spellings under `tests/`, `std/`, `examples/` and `bench/`, with string literals and comments left out (output in `uncls.txt`);
- `keep.py`: each row's "keep" count against the files of the folder that still hold a removed spelling;
- `deps.py`: the work-item ids each report defines and the ids every report uses (`deps.txt` holds every "Depends" line).

I read by hand every row the scripts flagged, every shared-decisions section, and the items named below.

## 0. Verdict in one paragraph

Coverage is close to complete:
- every one of the 521 H.41 rules has a row;
- every changed or new code is named;
- every conformance folder of an H.41 rule is named;
- no "keep" count is contradicted by a removed spelling in its folder (293 rows checked).

The problems are of three kinds:
1. **Ownership of work.**
   - Nine pieces of work are claimed by two or three areas. The worst cases are the removal of the run-time exclusivity machinery, the summary's per-parameter facts, the kind tests, `noalias`, and respelling the repository.
   - Five HEAP rules are claimed by no area.
2. **Shared decisions that disagree.**
   - Two areas put flags back into the header that OBJ-1 removes.
   - `owns dyn I` has two layouts.
   - The ownership words have two type representations.
   - The interface schema number collides with AK's.
   - Area 3's weak-count decision rests on a false claim that no interface files exist.
3. **Dependency cycles.** Seven cycles cross area 1 and areas 2, 3, 4, 5 and 7. Each needs either a joint slice or a corrected edge.

68 files with removed spellings, in 30 folders, are classified by no report. Four of those folders are not rules in either spec.

## 1. Coverage (by script)

### 1a. H.41 rules

| check | result |
|---|---|
| H.41 rows (changed / added / removed) | 521 (409 / 84 / 28) |
| rules with no row in any report | **none** |
| rules with a rule-table row in two reports | **DRV-1** (07:78 and 08:69, its `Component` row) |
| rules a script counted twice that are not really double | HEAP-10 (05:80 only names it in a removed-name row); RC-5 (01:91 only names it in a removed-name row); EXC-7 (06:113 has a removed-name row "L3013 across C++ calls", not a rule row; but see C1 below on who deletes the lint) |
| rows for rules not in H.41 (correct) | ECS-6 (08, unchanged); GRM-2, MOD-3, TYP-4 (07, rows whose code changed) |

Fix for DRV-1: area 7 keeps the row. Area 8's row becomes a note: "the `Component` entry of `derives.rs:27` goes in A8-3; DRV-1 is A7-10's". The rule then belongs to one area.

### 1b. Non-built rules that no work item names

These rows point the work at another area, and that area's items never name the rule.

| rule | row says | what is missing | fix |
|---|---|---|---|
| HEAP-3, HEAP-6, HEAP-7 | 01:73, :76, :77 "shares area" | 01:385 disclaims them; 03 never names them | area 3: add them to A3-2/A3-3 "Completes", with their old tests (01's 1b already classifies HEAP-6 w1 and HEAP-7 w1) |
| HEAP-4, HEAP-5 (removed) | 01:74-75 "remove with `Shared`" | no item deletes `Shared.get`/`get_mut`; A9-5 retires one AI test of HEAP-5; HEAP-5's 11 retires are owned by no item | area 3: A3-2 deletes them; area 4: A4-8 deletes their access calls |
| HEAP-10 | 01:79 "shares/threads areas" | 05 names it only inside its `@sync` row | area 5: add HEAP-10 to A5-8/A5-11 "Completes", with its 3 rewrites (01's 1b) |
| CLS-6 | 04:61 "'ended before drop' is area 3 (WK-3)" | A3-6 completes WK-3 but does not name CLS-6 | area 3: name CLS-6 in A3-6 |
| FN-3 | 01:46 "lifetime area owns `from`" | no area 2 item names FN-3 | area 2: name FN-3 in A2-2 (E3063 at return) |
| PHIL-3, -4, -8, -10, -11, -16 | 07: "OWN area", "THR area", "EXC and CELL areas", "WK area (E3032)", "RC/WK areas", "A7-6" | none of the named items lists the rule; four rows say "(-)" | area 7: name the items (PHIL-3 → A1-9 + A1-5; PHIL-4 → A5-8; PHIL-8 → A4-8 + A3-5; PHIL-10 → A3-9; PHIL-11 → A3-2/A3-3; PHIL-16 → A7-6..A7-8); each of those areas adds the PHIL rule to its item |
| STD-8, STD-8b, STD-18, STD-25 | 08: "the rename (types area)", "call word (R4 area)" | A7-8 (rename) and A1-8 (E3028) do not name them | areas 7 and 1 add them; area 8 replaces "R4 area" with "A1-8" |
| CELL-6..CELL-10, CELL-12 (removed) | 03: "delete" | no item number on the row | area 3: write "A3-5" in each row (it plainly is A3-5) |

### 1c. Error codes

`p17a-codes.md` has 298 codes against 264 in 0.9.10:
- 34 are new;
- 78 rows differ: 59 have a changed meaning and 19 are newly retired.

Every one of these 112 codes is named in at least one report. Area 7's section 7 (07:700-743) matches this diff exactly: 34 new, 59 changed, 19 retired. I re-counted `codes.rs`: 261 registered codes, as 07 says.

Area 7 hands each new code to an area. One hand-off has an owner dispute:
- E3031 is "BRW" in 07 and is raised in A2-10 (02:477).
- 04's SD4 and A4-5 say area 4 builds it.

This is contradiction C2 below.

### 1d. Conformance folders

- 317 of the 589 folders are rules in H.41, and every one is named by its area's report.
- 37 folders have none of their files named. These are "keep" folders or folders classified as a whole ("all six", "every file"), which is acceptable.
- One exception is 08's STD-19 (08:82): "respell 25" is given without file names, against the brief.

Two re-homings were proposed and then claimed by no area:
- 04:271 says "area 2 may re-home" EXC-18's ten `run_fail_*` programs, which 0.9.15 refuses when compiling (E3060 under BRW-17). Area 2 does not claim them.
- 03:289 says the same for CELL-10's six E3023/E3022 programs. Area 2 does not claim them either.

Also, EXC-18's five `accept_*` files are retired by 04:271, yet they are valid 0.9.15 programs: a getter returning a view of a field is still legal. Fix: area 2 takes the ten `run_fail_*` files as rewrites under BRW-17 and the five `accept_*` files as keeps under BRW-17 or LT-1. Area 4 changes "retire 16" to "move 15, retire 1" (`run_fail_replacing_a_refcell_field_while_its_guard_lives`).

### 1e. Files with removed spellings that no report classifies

The scan covers `Box[`, `Shared[`, `Weak[`, `RefCell`, `SyncShared`, `ForeignBox`, `owned`, `@sync`, `@borrows`, `changes`/`reads` clauses and `mem.forget`, with string literals and comments left out. It found 622 files:
- `bench/` 4, `std/src` 14, `tests/` 604;
- `examples/` 0.

Of these, 516 are named by file and 38 more sit in folders a report classifies as a whole. All of `std`, `bench`, `compile-fail`, `compile-pass`, `run-pass`, `ui`, `firstweek` and `perf` is classified, by area 8's section 6.

**68 files in 30 folders are classified by no report.** Area 7's A7-9 migrator respells "tests/**", but 07:233-236 says it leaves `Shared`, `Weak`, `SyncShared`, `RefCell`, `@sync`, `owned fn`, `mem.forget` to the slices that remove those features. So the files using those spellings have no slice. The `@borrows`, `@static_safe` and `ForeignBox` uses change meaning, not just spelling.

| folder | files | spellings | area that should classify it |
|---|---|---|---|
| TYP-16 | 27 | `Shared[`, `Weak[`, `owned` | 3 (A3-2 migrates `Shared`/`Weak` users); 7 for the rest |
| TYP-18 | 4 | `@borrows`, `owned` | 7 (`@borrows` → `from` is A2-1, so these are rewrites, not respells) |
| LT-4a, LT-4b | 4, 3 | `@borrows` (arena provenance) | 2. **Neither folder is a rule in 0.9.10 or 0.9.15**: they are orphans; re-home them under LT-4/LT-44 or retire them |
| UNS-10b | 2 | `owned`, `@static_safe` (E3105 tests) | 5 (A5-3). **Not a rule in either spec**: re-home under UNS-10 |
| TST-22 | 1 | `@borrows` | 7. **Not a rule in either spec** |
| THR-16 | 1 | `SyncShared`, `mem.forget`, `owned` | 5 (rewrite with `thread_shared`; `mem.forget` is gone, so the test's leak needs `into_raw` in `unsafe` or a different shape) |
| THR-14, PHIL-1 | 1, 1 | `@sync` | 5 (A5-9) |
| CORO-1, CORO-7, CORO-11, JOB-1, JOB-5 | 1, 1, 1, 2, 1 | `owned` | 5 |
| FFI-11f, FFI-42, FFI-44 | 2, 1, 1 | `ForeignBox`, `@borrows` | 6 (A6-5: these are rewrites, `owns T` of a foreign type) |
| FFI-2, FFI-9, FFI-17 | 1, 1, 1 | `owned` | 6 |
| SPN-6, CTL-10 | 2, 2 | `match owned`, `owned` | 2 |
| GRM-17, GRM-34, GRM-8c, TYP-32, DET-10 | 2, 2, 1, 2, 1 | `owned` | 7 |
| HASH-4, RNG-8 | 1, 1 | `@borrows`, `owned` | 8 |

Areas 1, 3 and 4 already list their unchanged-rule folders (01:144-147, 03:123-131, 04:281-290). Areas 2, 5, 6, 7 and 8 do not; each should add the same paragraph.

## 2. Contradictions

### 2a. Shared decisions

**S-1. The header carries no flags (OBJ-1), but two areas put one back.**
- 03 D1/D6 (03:521-590): count u32 + type index u32, weak u32 only where needed; the `flags` word goes, including `PINNED` and `FRAME` (03:405).
- 05 decision 1 (05:395-413) keeps "one header bit, set when an object is made `thread_shared`", and leaves "the bit's place in the 8-byte header" to area 3. Area 3 answered no (03:577-585, a second type index instead). Area 5 never adopted that answer.
- 06 D4 (06:599-602) pins an object by "the header's pinned flag (`ember_obj_pin`, `ember_rt.c.in:3200`)". That flag is `object->flags |= EMBER_OBJ_PINNED` (`ember_rt.c.in:3205-3207`, `ember_rt.h:1421`), in the word OBJ-1 removes. HR-12's 16 reload bytes are the live-list pointers (`ember_rt.h:1435-1436`), not a flag word.
- Fix:
  - area 5 adopts 03 D6 (a twin type index for `thread_shared`);
  - area 6 moves the pin out of the header. Choices: a bit in the reload-only `live_prev` pointer, which is aligned so its low bit is free, or a planner-side set of pinned addresses. Area 3 confirms the choice in A3-4's dependency on area 6.

**S-2. Kind tests by pre-order range vs area 3's twin indexes and reload.**
- 04 SD6 (04:604-609) and 07 SD4 both make `x is C` one compare, `(u32)(tid - first[C]) <= span[C]`, over a whole-program pre-order numbering.
- 03 D6's twin index for `thread_shared` objects falls outside the class's range unless the twins are numbered inside it.
- No report says what hot reload does to the numbering when a reload adds a class or a subclass. A6-18 and A6-19 cover header sizes only.
- Fix:
  - area 3 numbers each twin inside its class's pre-order span (or tests the index with a twin mask);
  - areas 4 and 7 state the formula once (one owner: A4-14, see C4);
  - area 6 adds an item: the type-index table under reload. Indexes stay stable, so new classes take fresh indexes, and the range test needs a fallback for families a reload extends. If no stable scheme keeps one compare, that is a question for the owner.

**S-3. The weak count is decided over the whole program, but area 3 says there are no interface files.**
- 03 D1 (03:533-534): "a search for an interface-file mechanism in `compiler/` finds none". **Wrong.** `compiler/ember_build/src/interface.rs` (1,679 lines, `SCHEMA_VERSION = 14` at `:50`) serialises "resolved callable signatures, region contracts, effect sets, exported inline bodies and type layouts" (`:4-8`). `ember_driver/src/incremental.rs:170` uses it.
- Whether a class family gets the 12-byte header depends on whether *any* `weakref` in the whole program names it. A library compiled to an interface record before its users exist cannot know that. The type layouts that record holds would then be wrong.
- Area 4's A4-3/BLD-14 and area 9's S7 both depend on per-package interface records.
- Fix: area 3 rewrites D1. Either the layout of a weak-countable family is fixed at link time (interface records then hold no field offsets for class families, only for value types), or the question goes to the owner. HR-12a makes reload a whole-process property; nothing does the same for the weak count.

**S-4. The interface schema number.**
- 04 (04:339, 04:555) puts the summary at `.emif` schema 15.
- AK, which A9-1 merges first, already sets `SCHEMA_VERSION = 15` (`git show stopped/AK-compiler-memory:compiler/ember_build/src/interface.rs:53`). AJ adds another bump (09:248).
- 09 S7 says "at least 16".
- Fix: area 4 writes "the schema after AK's and AJ's bumps (16 or later)".

**S-5. What `owns dyn I` is in memory.**
- 01 D1 (01:395-399): `owns T` is today's `Box`, and `owns dyn I` is `{data, vtable}`, two words with no header.
- 04 SD6 (04:604-606): "a `dyn` value of any kind is one pointer to a headered block … `owns dyn I` of a struct gets the same header".
- 08 3.3 relies on a "type-index compare" on `owns dyn AnyPool`.
- 07 SD1 says only "`owns T` is one pointer".
- OBJ-2 fixes the layout for class objects only. For a struct behind `owns dyn` the spec leaves it open, so the two must be reconciled.
- Fix: area 1 owns the decision, and area 4 adapts A4-14. Recommendation: keep 01's two words and put the type index in the vtable (a kind test is then one load and one compare, with no 8-byte header per boxed struct). That cost matters for ECS pools and `Array[owns dyn Shape]`.

**S-6. The type representation of the words.**
- 03 D4 (03:557-561): one `TyKind::Link { kind: Shares | SharesMut | ThreadShared | WeakRef, target }`, with no allocator.
- 07 SD1 (07:436-443): separate `Owns { inner, alloc }`, `Shares { inner, mutable, alloc }`, `ThreadShared { inner, alloc }`, `WeakRef`, `SharedRefMut`.
- A3-2 and A7-10 both build it (C5 below). Area 1's A1-14 needs `shares T using A`, which 03's shape cannot hold.
- Fix: one owner (A7-10 builds the representation; A3-2 builds the meaning) and one shape. 07's shape is the one with `alloc`; area 3 adjusts D4.

**S-7. The effect summary: same home, differing contents.** Every report puts it on the MIR `Body` next to `callable_regions` and serialises it in the interface record (02 D1, 04 SD1, 06 D7, 09 S7). That agrees. The gaps:
- 03 D7 asks for two facts, "may make an owner of parameter k or of `this`" (OBJ-5, D5) and "opens the owner edge `T.f`" (WK-7). 04 SD1 and A4-2 (04:318-337) list neither; they reserve a slot for area 3's parent-link fact only.
- 06 D7 asks for "what it hands to C". It is in A4-3 (04:341) but not in SD1.
- Fix: area 4 adds the three facts to SD1 and A4-2.

**S-8. How a function value's targets are found.**
- 04 SD2 and 09 S2 agree: the key is the exact callable type with its words, result, `once`, `unsafe` and `thread_shared`, over AJ's conversion sites.
- 02 D3 (02:591-597) omits `once` and `unsafe` from the key. For `dyn` it takes "every implementation or override of that slot", where 04 SD2 takes "the implementations in the program's upcast casts".
- The two sets differ, so area 2's checker and area 4's summaries would disagree on the same call.
- Fix: area 2 replaces D3 with "as area 4's SD2".

**S-9. The function value's first word.**
- 04 SD3 option (a) makes `code` point to a per-function record `{call, drop, c_entry}`.
- 09 S3 converts an item to `{code, 0}` with `code` the entry.
- 06 D1 stores `{code, state}` in callback slots.
- Fix: once area 4 settles SD3, areas 9 (A9-7) and 6 (A6-6) cite it rather than restate it.

**S-10. Spelling transition.**
- 08 3.1 (08:351-362): "each language area adds its new spelling **alongside** the old", then one migration commit, then refusal.
- 07 SD7 (07:470-473) and M1 (07:230-238): "A removed spelling is never accepted beside the new one". M1 refuses, migrates `std/` and `tests/**` and moves the version in one commit.
- These are two different plans for the same step. 07's plan keeps "nothing half-removed" and handles `std` by migrating it in the same commit, so 08's reason for a window does not hold.
- Fix: area 8 drops 3.1. A8-9 shrinks to the six non-mechanical `std` rewrites, each landing with the slice that removes its feature.

**S-11. Removing the run-time exclusivity checks.** The order agrees everywhere: compile-time rules first (04 SD8; 02 A2-11 "A2-4 and A2-10 must be in"; 03 Slice 4 "after area 4 removes"). The ownership does not (C1).

### 2b. Work claimed twice, or by neither

| # | work | claimed by | fix |
|---|---|---|---|
| C1 | delete `insert_shared_accesses_all` (`borrows.rs:327`), `access.rs`, `loop_access.rs`, `long_access_lint.rs`, the driver passes, `BeginAccess`/`EndAccess`, the runtime's access helpers (`ember_rt.h:1953-1972`) | A2-11 (02:494-505), A4-8 (04:404-422), A6-8 (L3013, 06:113/426), A3-4 (ember_rt.h access words, 03:405), A9-5 (AI's access code) | A4-8 owns the deletion; A2-11 keeps only the `RefCell` guard loans and `check_refcell_call`; A6-8 and A9-5 depend on A4-8 |
| C2 | the summary's per-parameter facts and the call checks | A2-9 vs A4-2 (the same list: view params ended before last use, `ref mut` reach, "only adds", fields of `this`, stored function at fault); A2-10 vs A4-4/A4-5 (mapping summaries at calls; E3031) | A4-2 computes every fact; A2-9 becomes "A4-2's borrow transfer functions"; A2-10 raises the codes through A4-4's mapping API, as 04 SD4 says; E3031 is A4-5's or A2-10's, not both |
| C3 | `noalias`/`restrict` on `ref mut` parameters | A2-14, A7-19, A9-10 | A2-14 owns it; A7-19 goes; A9-10 only adapts AI's `Ext::Ref`. A7-19 lists only "BRW's call proof" as its prerequisite, so it could land before A2-10/A2-13, which A2-14 marks unsound |
| C4 | kind tests `x is T`, `is T as name`, `cast`, E2036/E2037/E2106, the one-compare test | A1-20, A4-14, A7-12 | A4-14 owns the checker; A7-7 the parse; A1-20 and A7-12's checker half go |
| C5 | the type representation of the words | A3-2 (`Link` variant, 03:352), A7-10 | see S-6 |
| C6 | `into_raw()` built in, only in `unsafe` | A1-11, A5-4 (06:340 notes it) | A1-11 owns it; A5-4 becomes the TCB listing only |
| C7 | `this`, `static fn`, parameter words and call words, `copy of`, the `owns` lexing | A1-1, A1-5, A1-7, A1-8 vs A7-4, A7-7, A7-8 (07:280-288 does "parameter words onto the existing modes", `copy of` through `Clone`, `static fn` E2115, `this`) | one owner each: 07 for lexing and parsing in M1; 01 for the meaning. A7-8 must not also build what A1-5/A1-7/A1-8 build |
| C8 | views kept anywhere, and re-pointing | A2-8 vs A7-26 (TYP-15); A2-3 vs A7-27 (re-pointing through `ref mut`, `const ref mut`) | area 2 owns both; A7-26 and A7-27 shrink to their TYP rows' texts |
| C9 | respelling `std/` and `tests/` | A7-9 (all of `std/src`, `tests/**`), A8-9, A8-10, plus every area's "respell" counts | A7-9 owns every mechanical respell; area items keep only rewrites and retires |
| C10 | HEAP-3/-4/-5/-6/-7/-10; the EXC-18 and CELL-10 re-homing | nobody | section 1b and 1d |

## 3. Dependencies

Every work-item id named anywhere (A1-1 … A9-12, 160 ids) is defined, and each is defined once, in its own report. Area 9's row ids (K1, J4, J6, I9) are defined in 09.

Many "Depends" lines name an area by a family or design folder that is no area: "area ATT" (A5-3), "area ARN/ALC/STA" (A5-4), "the methods area", "lifetimes area" (A5-7), "area RC" (A5-9), "area CLO", "area EFF" (05), "R7 area", "R4/R6 area", "regions area", "loops area", "kind-test area" (08:238-306). Each should name the item:

| written | means |
|---|---|
| ATT | A7-8 |
| ARN | A1-12 |
| ALC | A1-13/A1-14 |
| STA | A1-17 |
| methods | A1-7 |
| lifetimes | A2-1 |
| RC | A3-2/A3-3 |
| CLO | A4-9 |
| EFF | A4-2 |
| R7 | A2-8 |
| R4 | A1-8 |
| R6 | A2-1/A2-2 |
| loops | A2-6 |
| kind-test | A4-14 |

### Cycles

| # | cycle | evidence | fix |
|---|---|---|---|
| Y1 | A1-7 → A4-7 → A1-7 | A1-7 (01:278) needs "what a method does to `this` is inferred, EXC-15"; EXC-15 is A4-7's (04:399), which depends on A1-7 (04:400) | A1-7's edge goes to A4-2 (the summary), not to A4-7 |
| Y2 | A1-17 ↔ A4-2 | A1-17 (01:308) needs EFF-24's module-variable changes; A4-2 (04:335) depends on A1-17 ("statics become places") | split A1-17: (a) `static mut` as a place, before A4-2; (b) its per-call checks, after |
| Y3 | A1-17 ↔ A5-18 | A1-17 needs the threads area's E7012/THR-20 (A5-18); A5-18 (05:332) needs "area STA" = A1-17 | the same split: A5-18 depends on (a); (b) depends on A5-18 |
| Y4 | A1-8 ↔ A2-3, and A1-8 → A2-14 → A2-10 → A1-8 | A1-8 (01:268) needs BRW-15 (A2-3/A2-10) and BRW-22 (A2-14); A2-3 (02:387) and A2-10 (02:492) depend on A1-8 | A1-8 is the words and E3028 only; drop its edge to area 2. BRW-15/BRW-22 depend on A1-8, not the reverse |
| Y5 | A1-20 ↔ A4-14 | A1-20 (01:365) depends on "the dyn/cast area"; A4-14 (04:518) depends on A1-20 | merge, per C4 |
| Y6 | A1-9 → A3-4 → A4-8 → A4-6 → A1-9 | A1-9 (01:291) needs "the class object's count and weak count, OBJ-1/OBJ-3" (A3-4's header); A3-4 (03:419) needs area 4's access words gone (A4-8); A4-8 (04:422) needs A4-6; A4-6 (04:388) needs A1-9 | A1-9 needs only A3-2 (sharing words), as 03:347 says; drop its OBJ-1/OBJ-3 edge. A3-4 and A4-8 are mutual ("same release", 04:422) yet sit in different slices (03 Slice 4, 04 Slice 2b): make them one slice |
| Y7 | slice M1 (A7-4..A7-9, "one commit") → A4-2 and A1-8 → M1 | A7-8 (07:293) needs "EFF area's first summary slice" (A4-2) and "FN area's call-site words (E3028)" (A1-8); A1-8 → A1-1 → the lexer decision (A7-4); A1-7 → GRM-41 (A7-6); A4-2 → A4-1 → A9-2 | M1 is the parser plus refusals plus the migrator, with no checker meaning that needs a summary. The "object change-mode" of A7-8 stays read from today's `mut self` until A4-2 lands; A1-1/A1-7/A1-8 land inside M1 or right after it |

Two more slice inconsistencies:
- **A1-10/A1-11.** A1-10 (Slice 1) "must land in the same slice" as A1-11 (01:176). A1-11 depends on A1-6 and A1-9 (Slice 4 and Slice 6, 01:391). So Slice 1 cannot be consistent. Fix: move A1-10 to A1-11's slice, or keep `mem.forget` until then.
- **A1-9 with A3-2/A3-3.** 03 Slice 3 says they are one milestone (03:347-349). 01 places A1-9 in its own Slice 6. Fix: one joint slice, named in both reports.

## 4. Spot checks (3 or more per report)

| report | claim | checked | result |
|---|---|---|---|
| 01 | `ember_types/src/lib.rs:2122` Class/ClassInterface are `Copy` | inside `copy_with` (`:2084`), `:2122` returns true | right |
| 01 | `lower.rs:871-885` `emit_drops_from` | `:871` | right |
| 01 | `counted_of` at `ember_typeck/src/lib.rs:19462` (01:73, :79) | `fn counted_of` is at **`:19472`**; `:19462` is the doc comment above it | **wrong line** (03 cites `:19473` correctly) |
| 02 | `insert_shared_accesses_all` at `borrows.rs:327` | `:327` | right |
| 02 | `check_point` `borrows.rs:4564` | `:4564` | right |
| 02 | `loop_version.rs:834-905` `Summaries` | `struct Summaries` `:834`, `impl` `:842` | right |
| 03 | "a search for an interface-file mechanism in `compiler/` finds none" (03:533) | `ember_build/src/interface.rs`, 1,679 lines, schema 14 | **wrong** (S-3) |
| 03 | the runtime tests `ti->flags` on every retain/release, `ember_rt.h:1933-1948` | `:1934-1947` | right |
| 03 | class arm of the retain glue `ember_codegen_c/src/lib.rs:8382-8390` | `retain_lines_for_value`, Class arm at `:8384` | right |
| 03 | `ember_types/src/lib.rs:1825-1835` the `Shared`/`SyncShared`/`Weak` cases | there | right |
| 04 | borrow check at driver `lib.rs:5336`, effects at `:5442` | `:5335-5336`, `check_effects_all` `:5441` | right |
| 04 | D-877's guess by arity and result `effects.rs:904-936` | `plain_function_values` `:904` | right |
| 04 | base-chain walk `ember_rt.c:3408` | `ember_downcast` `:3408` | right |
| 04 | summary in "`.emif` schema 15" | AK already uses 15 | **conflict** (S-4) |
| 05 | `EMBER_TI_SYNC` test on every count change `ember_rt.h:1933-1947` | right; `#define` at `:1416` | right |
| 05 | Ember `static mut` refused today, `ember_typeck/src/lib.rs:10259` | `if decl.is_mut` → E1010 at `:10259-10262` | right |
| 05 | Send/Sync table `ember_types/src/lib.rs:1777-1918` | `is_send` `:1757`, the table follows | right |
| 06 | `ForeignBox` is 24 bytes, `std/src/ffi.em:165-168` | pointer, destructor, u32 | right |
| 06 | the adopt "only handle" check `ember_rt.c.in:2496` | `ember_cpp_object_adopt`, strong != 1 | right |
| 06 | `ember tcb` fixed line `ember_driver/src/lib.rs:1736` | `:1736` | right |
| 06 | the pinned flag in the header (D4) | it is the `flags` word OBJ-1 removes | **conflict** (S-1) |
| 07 | `LANGUAGE_VERSION` "0.9.10" `ember_parser/src/lib.rs:21` | right | right |
| 07 | lexer test pins 47 keywords `token.rs:351` | right | right |
| 07 | E7005 emitted for `+ Sync` `decls.rs:1635` | right | right |
| 07 | `codes.rs` registers 261 codes | 261 | right |
| 08 | `derives.rs:27` `WRITTEN` holds `"Component"` | right | right |
| 08 | `MemForget` `ember_hir/src/lib.rs:2063` | right | right |
| 08 | `AnyStorage` flat `ecs.em:100-131` | `sparse: Array[u32]`, swap-remove at `:127-131` | right |
| 09 | branch sizes (AK 5 commits +1385/-301 in 25 files; AJ 4, +1178/-123 in 33; AI 9, +7342/-2343 in 65) | `git diff --shortstat` from each merge base | right |
| 09 | `holds_a_cell` and `class_field_offsets_after(id, header)` on AI | `stopped/AI-llvm-backend:compiler/ember_types/src/lib.rs:1691`, `:2421` | right |
| 09 | D-877's key at `effects.rs:904` | right | right |

## 5. What each area should do (summary)

- **Area 1:**
  - fix the `counted_of` line;
  - drop A1-9's OBJ-1/OBJ-3 edge (Y6), A1-7's edge to A4-7 (Y1), and A1-8's edge to area 2 (Y4);
  - split A1-17 (Y2, Y3);
  - move A1-10 to A1-11's slice;
  - own C6 and S-5;
  - make A1-9 a joint slice with A3-2/A3-3.
- **Area 2:**
  - take EXC-18's 15 and CELL-10's 6 re-homed files;
  - classify SPN-6, CTL-10, LT-4a and LT-4b;
  - adopt 04 SD2 (S-8);
  - shrink A2-9/A2-10/A2-11 (C1, C2);
  - own C3 and C8;
  - name FN-3.
- **Area 3:**
  - rewrite D1's interface-file claim and the weak-count decision (S-3);
  - number twins inside pre-order spans (S-2);
  - take HEAP-3/-4/-5/-6/-7 and CLS-6;
  - put A3-5 in the CELL rows;
  - classify TYP-16's `Shared`/`Weak` users;
  - adopt 07's type shape (S-6);
  - one slice with A4-8 (Y6).
- **Area 4:**
  - schema number (S-4);
  - add 03's two facts and 06's "hands to C" to SD1 (S-7);
  - own C1 and C4;
  - EXC-18 "move 15, retire 1".
- **Area 5:**
  - adopt the twin type index instead of a header bit (S-1);
  - take HEAP-10;
  - classify THR-14, THR-16, PHIL-1, CORO-1/7/11, JOB-1/5 and UNS-10b;
  - name items instead of family "areas" in every Depends line.
- **Area 6:**
  - move the pin out of the header (S-1);
  - add a reload item for the type-index table (S-2);
  - classify FFI-2/9/11f/17/42/44.
- **Area 7:**
  - give up A7-19, A7-12's checker, and A7-26/A7-27's checker halves;
  - make A7-8 lexing and parsing only (C7, Y7);
  - name the PHIL rows' items;
  - classify TYP-18, TYP-32, GRM-17, GRM-34, GRM-8c, DET-10 and TST-22.
- **Area 8:**
  - drop 3.1 (S-10);
  - name STD-19's 25 respells;
  - make the DRV-1 row a note;
  - classify HASH-4 and RNG-8;
  - name items instead of "R4/R6/R7 area".
- **Area 9:** cite 04's SD3 in S3/A9-7 (S-9). Nothing else.

## 6. Confidence

- **Scripted:** the coverage counts (1a, 1c, 1d, 1e) are exact.
- **1e's spelling scan** is a regular expression with string literals and trailing comments stripped. It can still miss a spelling split across lines, or catch an identifier that happens to be `owned`. I read about 25 of the 68 flagged files by hand (all of TYP-32, SPN-6, GRM-34, CORO-1, FFI-42, LT-4a, THR-14, HASH-4, UNS-10b).
- **Cycles:** read from the "Depends" text, with family names resolved to items by me. Where a report's wording was loose, I quoted it.
- **Contradictions:** each was read in both reports' own words, at the lines given.
