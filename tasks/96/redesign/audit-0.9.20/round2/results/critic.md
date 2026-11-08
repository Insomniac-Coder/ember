# Critic

## missing

### missing 1
**what**: The text the whole round read is stale. CURRENT-SPEC.md (round2/inputs) is not the parts at HEAD (commit dd99380, 'verdict 165, the gap review's lead fixes'): 44 rules differ, among them [THR-1] (the lead's verdict-165 bullet is in p11 and absent from CURRENT-SPEC, whose second bullet still reads 'everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a plain `fn()` value'), [FN-2a], [OWN-9], [GRM-15] (the verdict 147/160 clause 'a value made in the call with a word that makes an owner or a link ... takes no `owns` before it'), [LT-48] (69 lines: 'a lambda's typed parameter takes it in the same slot', 'it stands before the whole type, the holding word included'), [LT-1], [LT-46], [BRW-18], [HR-44] (the proofs through a reloadable callee, 15 lines), [HR-11], [PAR-1]..[PAR-5] (re-wrapped, `?` added), [STD-15] (`extend(owns items)`), [FFI-52] (`setup(shares mut game)`), [STR-5], [IFC-4], [CTL-10], [EXC-17], [CELL-1/2/13], [HND-2/3], [ARN-5c/d], [SIMD-5], [PHIL-8/11], [LEX-11], [GRM-45], [DIA-24], [GPU-11] and the registry rows of `E0109` and `E7015`.
**why_it_matters**: Every reader's old string and every judged ruling's asserting script was copied from CURRENT-SPEC.md; the K10 ruling says 'The old strings are copied with CURRENT-SPEC.md's wrapping ... each verified to occur exactly once (checked for `[DRP-8]`'s and `[HR-44]`'s)', and [HR-44] at HEAD differs by 81 words, so that script fails; K12/K18/K19 (FN-2a, OWN-9, GRM-15), K58/K59 (LT-48), K37/K38/K39 (PAR-1/2/3), K31/K53 (FFI-52), K09 (LT-1), K02/NG-24 (THR-1) were all judged against text the lead had already changed, and some of their edits duplicate the 35 lead fixes.
**where_to_look**: Run /tmp/claude-0/-home-user-ember/76ebe43b-c2ec-58a2-88ba-aa90c6b1cf0a/scratchpad/bulletdiff.py with the rule ids above (it diffs a bullet in CURRENT-SPEC.md against the concatenated tasks/spec-0.9.9/parts/p*.md); git show dd99380 --stat; the per-part counts: p07 183 lines, p11 54, p05 51, p09 38, p01 35.

### missing 2
**what**: [THR-1]'s second bullet now has three texts and no merge: the lead's (at HEAD, 'everything reachable from it is `Send` and `ThreadShared` (`[THR-8]`, `[THR-9]`) ... (verdict 165: "`FrameStats.draws` is an `RwLock[Cell[int]]` ...")'), R5's K02 (3 edits, status not_answered, 'departs 165 ... no rule asks for `ThreadShared` there') and R9's NG-24 (1 edit, 'brings [THR-1] to the verdict'). The K02 input file itself says 'Status: answered by the owner, verdict 165'.
**why_it_matters**: At HEAD K02 is 'answered' and both reader edits are redundant or conflict in wording with the applied bullet; the digest records K02 as a hole in the current text.
**where_to_look**: tasks/spec-0.9.9/parts/p11-concurrency.md lines 92-106 at HEAD; CURRENT-SPEC.md 6384-6387; round2/inputs/K02-thread-shared-not-threadshared.md line 11; JOB.md verdict 165 (line 1253).

### missing 3
**what**: Five results end in needs_owner and no OPEN ODR records the question: K05 ('needs_owner: safety does not decide between shape A ... and shape B'), K20 ('The owner gave no verdict on K20 itself (J6-16 was routed to him); this ruling is for him'), K21 (status needs_owner; the two-name `at`/`at_mut` split departs from the method-3 lead's decision 4), NG-10 (every return from Ember into C a check point; refuter 'narrowed, needs_owner'), NG-26 (the const/non-const accessor pair, 'a gap for K47').
**why_it_matters**: docs/AUTOPILOT.md §3: 'When something truly needs the owner ... record it as an OPEN ODR and continue'; grep of docs/OWNER-QUEUE.md finds no entry for K05, K20, K21, J6-16, J8-04, J5-05, NG-10 or NG-26.
**where_to_look**: docs/OWNER-QUEUE.md (no match for those keys; last ODRs are ODR-255 at line 1597 and the closed audit trail from line 7101); the digest's `remaining_doubt` of K05, K20, K21; round2/inputs/design-K05-question.md, design-K20-question.md, design-K21-question.md.

### missing 4
**what**: The current text contradicts itself on `thread_shared T using A`, and the K05 ruling's edit list may not reach every side. [OBJ-4]: 'A `thread_shared` value's block always comes from the runtime allocator (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is `E2114`'; [TYP-46]: 'after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`'; [ALC-1]: 'never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`'; against [TYP-46a]: '`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making line', [OBJ-1]: 'A `thread_shared` value made `using` an instance (`[ALC-1]`) ... is marked handed over at the line that makes it', [RC-4] point 4 the same, and the `E2114` registry row: '`thread_shared` before a value made `using m`, whose block would carry no making-thread record'. R6 listed this as missed ('allowed by `[OBJ-1]`, `[RC-4]` and `[TYP-46a]` and `E2114` by `[OBJ-4]` and `[ALC-1]`'); the K05 ruling (shape B) names [TYP-46], [ALC-1], [OBJ-1], [RC-4] and not [OBJ-4], [TYP-46a] or the `E2114` row.
**why_it_matters**: After the ruling one of the six places still refuses what another accepts unless the lead edits all of them in one direction; the critic's note on K05 ('it must be answered before K34') was not honoured, since R4 edited the same [TYP-46]/[ALC-1] sentence for K34 in parallel.
**where_to_look**: CURRENT-SPEC.md [OBJ-4] line 4637, [TYP-46] 1697-1703, [ALC-1] 5413-5415, [TYP-46a] 1716, [OBJ-1] 4618-4620, [RC-4] point 4 at 4788-4798, `E2114` row 9873; round2/inputs/K05-thread-shared-using.md and K34-which-types-take-using.md.

### missing 5
**what**: The refuters' 'missed' items (33) have no status, no edit and no verdict; each is a cluster of its own. R1: a whole name as an element of a collection literal (handed over, viewed or refused); `owns dyn I` and which implementation it has; `Option[Cs]...` not producible ([TYP-47]). R2: a one-word owner capture whose target the body changes through a lock or a `shares mut` link is put in a block for no reason ([CLO-17]/[CLO-18]); [BRW-15]'s 'a plain local that holds a view' read to cover a local owning an iterator; [DRP-1]'s 'It changes its own object as a given' does not except `let` fields; [STR-2]'s 'a construction that omits the field' does not name a written `init`'s call. R3: a part handed out of a value with several owners leaves a hole the other owners read ([OWN-10]/[RC-1]); a `@cached` body reading a `static mut` goes stale ([CLS-12]); `sharedref mut dyn I` grammatical by [TYP-45a] ('a word of `[TYP-45]` stands before ... a `dyn` bound') and decided by no rule. R4: the E4070 row omits the lock's view and `try_recv` that [DET-2] names; the E3096 row names only the parent arena; E2020 for `c as int` gives no line. R5: a thread-bound foreign type vouched `ThreadShared` sends its view; `thread_shared fn()` outside `ThreadShared` in [THR-9] and the marker table. R6: `dyn I` has no `ThreadShared` rule (now partly written by the K20 ruling into [THR-9], to be confirmed); the `thread_shared T using A` contradiction above; E4070's help for a platform function `std.math.det` lacks. R7: `@ffi(std_function)` with a raw pointer in a parameter (K07's fourth spelling, now covered by the K07 ruling's ODR-199 change, to be confirmed); `@ffi(newtype)` over a pointer typedef ('a nominal type with the typedef's representation', [FFI-8]) outside the limit's words `*T`, `*mut T`; [UNS-1] lists only 'access a foreign `static mut` (`[STA-1]`)' while [FFI-8]/[FFI-41] make reading a foreign `const` static unsafe; what a registering call returns when the C function has a result (K31 residue); whether a hand-declared `type` in `unsafe extern "C":` takes `@ffi(handle)` (K46 doubt). R8: a free C++ operator whose first operand is not the class; [FFI-10]'s '64 bits' false on a 32-bit target; [THR-2]'s message shows a safe call [FFI-11] refuses; the ODR-166 row says `ForeignBox[T]` where [FFI-32] says `owns T`. R9: a method cannot know which manager `this` was made in; a `thread_shared` value made `using System` or a one-instance manager has neither chunk header nor handed-over mark (the K05 ruling's row 'any manager but `Global`' answers it only if applied there); `some I + Send` has no meaning. R10: the parameter words an `extern "C" fn` type may carry (NG-41's assumption); how an imported struct's function-pointer field becomes `Option`; the `T&&` override's move is a thunk operation `ember inspect` should list (NG-29); an interface implementation's method need not have the interface's first line (K11's critic note).
**why_it_matters**: Each names a program the text accepts or a question an implementer cannot answer, and the round closed none of them.
**where_to_look**: The digest's `missed` arrays per reader; CURRENT-SPEC.md [TYP-45a] 1675, [UNS-1] 5542, [DRP-1] 4462, [STR-2] 2471, [CLS-12] 2641, [FFI-8] typedef and function-pointer rows, registry rows E4070 9951 and E3096 9939.

### missing 6
**what**: Where a `thread.scope()` task runs is still unstated. [THR-5]: 'The `with` block is ordinary code of the enclosing function ... the scope first joins every task spawned in it' says nothing of the thread; §XI.5 neither. The K10 ruling pins 'a scope's task or job runs on a thread the runtime owns, never on the thread that opened the scope' into [RC-4] point 4 alone and asks the lead to 'check the runtime's design (M9)'; the second judge says the ruling 'holds either way' but the lead 'should pin it for `[HR-3a]`'s registration and the home-thread flag'.
**why_it_matters**: The by-thread mark (a task-thread-made value is never queued) rests on it; if a task may run on the opening thread, a plain value it makes is home-counted on main and the mark lies.
**where_to_look**: CURRENT-SPEC.md [THR-5] 6738, §XI.5 from 6879, [HR-3a]; round2/inputs/design-K10-method4.md; runtime/ember_rt for the scope implementation.

### missing 7
**what**: [CLS-12]'s whole-program fact is flagged by both K20 judges and edited by nobody: 'On an object several threads reach, the kept result is filled once under a once-flag' and 'on an object several threads reach, a kept result is filled under a once-flag and never cleared' are facts of the program, against [BLD-16] ('not by which of its values are made `thread_shared`'). R3's K43 edits [CLS-12] for a `weakref` into what a `@cached` method reads, and R3's missed item adds a `@cached` body reading a `static mut`; three directions on one rule, no merge.
**why_it_matters**: A package compiling the method cannot know whether any object of the class is `thread_shared`; the K20 ruling makes the case reachable through every `Error`-implementing class.
**where_to_look**: CURRENT-SPEC.md [CLS-12] 2641-2668, [BLD-16] 9269; the K20 `remaining_doubt` (1) and (3); round2/inputs/K43-weakref-into-cached.md.

### missing 8
**what**: [FFI-17d]: '`threads=any` requires the class to be `Send` and `ThreadShared` (`E7015` at the class, naming the field ...)'. The K20 ruling makes an open class's markers a header promise ('for an open trampoline class this now means the header word, and the message should name it; FFI-17d is not edited here').
**why_it_matters**: A trampoline class derived from a C++ base is `open` by construction; without the edit its `threads=any` check reads the fields of the declared class only, the gap the K20 ruling closes elsewhere.
**where_to_look**: CURRENT-SPEC.md [FFI-17d] 11224-11241; the K20 ruling's `remaining_doubt` (2) of the second judge.

### missing 9
**what**: Annex B's reload row 'field added with a default | initialised to that default' lets a reload of package A add a one-thread field (`level: shares Level = shares Level()`) to a class whose live objects package B, not rebuilt, made `thread_shared`; both K20 judges name it as 'a separate finding' and K14 (HR-12/13/15, 'answered', 0 edits) and K48 (0 edits) did not take it.
**why_it_matters**: A `thread_shared` object would hold a `shares` owner across threads after a reload, which [THR-1]'s making-line check never sees.
**where_to_look**: CURRENT-SPEC.md Annex B table rows at 10856-10858 and [HR-11] (its schema 'kind, base, layout attributes'); K20 `remaining_doubt` (2) in both judges.

### missing 10
**what**: [DIA-18] and the `E5002` registry row disagree: [DIA-18] 'A foreign call rejected because its contract has unknown facts (`E5002`) names each missing fact', the row '(a call of a function without a verified contract outside `unsafe` is `E3100`)'. The K07 ruling lists it as 'pre-existing, not touched here, for the lead's consistency pass'; no result took it.
**why_it_matters**: One mistake, two codes: the brief's 'one code, one defining rule'.
**where_to_look**: CURRENT-SPEC.md [DIA-18] 9573, `E5002` row 9954; tools/p17a_0915.py.

### missing 11
**what**: [ALC-2]'s `Allocator` prints `fn allocate(layout: Layout) -> *mut u8` with no receiver mode while [ALC-1] says '`allocate` changes the instance (`[EXC-15]`, `[ALC-5]`)' and 'a manager with one instance is a `static` whose `allocate` reads `this`'. The K05 ruling's doubt (1): 'until K34 or P01 fixes `Allocator`'s receivers ... `thread_shared T using a` in such a body is E7015 unless checked per monomorphised instance'. K34's rules are [ALC-1], IV.1, [TYP-46]; nobody edits [ALC-2].
**why_it_matters**: A generic body over `[A: needs (Allocator, ThreadShared)]` cannot read from the bound what `deallocate` does to `this`, so the K05 property cannot be checked there.
**where_to_look**: CURRENT-SPEC.md [ALC-2] 5458-5484, [ALC-1] 5422-5427; K05 `remaining_doubt` (1).

### missing 12
**what**: The K21 ruling adds `rename status as Status` for a type while [FFI-30a] reads 'An overlay changes the signatures, safety and names of *functions* and adds wrappers; it never changes a type's identity, layout or fields' (K21 doubt 4: 'the lead may wish to add "and names of types"'); K75 (R8, FFI-12/FFI-30c/FFI-55/GRM-35, 1 edit) is the only other result on the overlay grammar and does not touch [FFI-30a].
**why_it_matters**: A reader of [FFI-30a] refuses what the K21 examples write.
**where_to_look**: CURRENT-SPEC.md [FFI-30a] 8333; the K21 winners' edits to [GRM-35]/[FFI-12].

### missing 13
**what**: New codes and messages the rulings introduce have no registry row or `codes.rs` entry yet: `E5035` (K21, 'ambiguous C++ member'; no row in CURRENT-SPEC, which has `E2070 | ambiguous interface method | [TYP-24]`), the K07 `E5002`/`E3100` texts, the K05 `E7015` 'at a type no value could fill', the K20 `E2040` at `implements Error` and `E7015` at a derived class. K21 doubt (7): 'if the lead classifies FFI codes there [[DIA-12]], a shape "ambiguous C++ member" with the cast or `rename` line as help is needed'.
**why_it_matters**: AUDIT §7's cut recipe requires `code_registry --check` 0 problems and `codes.rs` to carry every live code.
**where_to_look**: CURRENT-SPEC.md registry rows 9856-10006; tools/p17a_0915.py; compiler/ember_diag/src/codes.rs; docs/AUDIT-0.9.20.md §7 'Applied how'.

### missing 14
**what**: The K07 ruling hands two holes to nobody: (a) [FFI-8]'s table 'has no row for `void*`/`const void*`; `pNext`'s refusal assumes it maps to a raw pointer type ... K46 or the table's owner should add the row' (the table has rows for `typedef` and 'function pointer' only); (b) 'The dual route is not closed here ... an `@unsafe(export(…))` function ([FFI-26]; [FFI-31b] lists no refusal of a raw-pointer result) or an `override fn` of a C++ virtual ([FFI-39]) that returns `ref_to_ptr(ref mut local)` in safe code hands C a dangling address', 'for K08's lead'. K08 was read by R7 (3 edits) with the refuter's 'hole_in_proposed_text', and no judged ruling owns it; NG-17, NG-19, NG-21, NG-32, NG-33, NG-38 and NG-41 are the same route.
**why_it_matters**: The ruling's own statement that 'the route by which safe code handed C a null, dangling, short or forged address under an assertion of safety is closed' is false while (b) is open.
**where_to_look**: CURRENT-SPEC.md [FFI-8] rows (grep 'typedef', 'function pointer'; no 'void'), [FFI-21a], [FFI-26], [FFI-31b], [FFI-39]; round2/inputs/K08-export-arguments-from-c.md; new-gaps.md NG-17 (line 751), NG-19 (832), NG-21 (933), NG-32 (1480), NG-33 (1540), NG-38 (1825), NG-41 (1987).

### missing 15
**what**: docs/AUDIT-0.9.20.md §7's unrun exit checks: 'the exit checks the plan named (an implementer and an attacker over the changed Parts) were not run, for the owner's credits: a small pass may run them'. This round read 76 clusters and 47 gaps, each a question; no result is a whole-document implementer or attacker read over §H.52's 222 changed rules.
**why_it_matters**: The owner's task for the pass is 'any agent can pick it and start implementing without any disruptions'; the implementer read is the check of that.
**where_to_look**: docs/AUDIT-0.9.20.md lines 452-455; tasks/96/redesign/PLAN-spec-0.9.20.md exit criteria; round1/G9.json (the round-1 implementer read, before Hardened_2's 222 changes).

### missing 16
**what**: Owner items carried from Hardened_2 that no result mentions: N83 'reverses half of verdict 135, reported' (the owner's 'better solutions are welcomed' is not a verdict on it; 'The owner may reinstate the written file'); N78/N79 answered the owner's own open questions N38–N40 (O7) by delegation; N87 'touches verdicts 101 and 114, reported'. The round's K59 ('runtime placement beyond verdicts', 1 edit) builds on N78 without the owner's word.
**why_it_matters**: docs/AUTOPILOT.md §3: 'Never overturn what the owner ruled himself'; these stay owner questions until he answers, and the owner list for Hardened_3 should carry them beside K05/K20/K21.
**where_to_look**: tasks/96/redesign/compiler-0.9.15/decisions.md §13 rows N78, N79, N83, N87 (lines 699-724); docs/AUDIT-0.9.20.md §3 O6, O7 (lines 180-187); JOB.md verdict 135 (line 1066).

### missing 17
**what**: The K20 ruling's std check is unverified: 'std must compile once under the new parents ... `SendError[T]` (std/src/sync.em 122) gets its `Error` implementation under `[T: needs (Send, ThreadShared)]`' (judge 1) against 'SendError[T] in std/src/sync.em implements no `Error` today, so nothing to bound' (judge 2); 'pub open class AnyStorage (std/src/ecs.em 101) writes the promise only if a `World` is meant to cross threads (ODR-218); read only, not verified'. The digest speaks of 'all three copies of `Error`'s declaration'; CURRENT-SPEC.md has two (`interface Error: needs (Debug, Display):` at 1958 in the sketch, quoted at 1742), the third being std/src/core.em.
**why_it_matters**: Two winners disagree on a fact of the standard library the ruling edits; the owner's rule 'A standard-library file must make sense on its own' puts the `needs` line in core.em.
**where_to_look**: std/src/sync.em around line 122; std/src/ecs.em line 101; std/src/core.em `interface Error`; CURRENT-SPEC.md 1742, 1958, [ERR-3].

### missing 18
**what**: The K10 ruling's test set is named but not produced: 'the lead's test set should carry the arbiter's E3-1 program as a run-pass test printing `0` then `1`, J7-01's as frames then `1`, J7-02's as two `freed` lines during the clear, and a break test that removes the by-thread mark ... and one that removes the by-type mark'; likewise K07's E5002 'must-refuse' programs (NG-34, NG-37 'Add this program as a must-refuse test') and NG-45's note that two K20 tests 'follow the implementation, not the specification'.
**why_it_matters**: docs/AUTOPILOT.md §4: 'Every fix has a test that fails without it'; JOB.md: 'for test changes make everything in accordance to the spec'.
**where_to_look**: tests/conformance/ (RC-4, THR-19, FFI-10 directories); round2/inputs/design-K10-method4.md; new-gaps.md NG-45 (line 2216).

### missing 19
**what**: K05's doubt (3): a `thread_shared` value made in a local `Arena` handed to an unscoped thread relies on [LT-4]'s region (`E3063`) and [ALC-5]'s 'one handed to an unscoped thread ... lives, for this rule, as long as the instance'; 'the lead should confirm that a spawn's capture counts as "kept where it would outlive the instance" for a region-bearing value with no view in it, a gap of P02'. NG-01 and NG-04 (R9 'answered') show the sibling: a block kept by a `weakref` after its value ends while the instance moves (`moved = counter`).
**why_it_matters**: A kept block's instance word dangles after a move the text allows; R9's 'answered' rests on the K04 design's [LT-4] edit, whose 'with_edit' column in new-gaps.md says 'The same' for NG-04.
**where_to_look**: new-gaps.md NG-01 (line 3) and NG-04 (159), their `with_edit` lines; CURRENT-SPEC.md [LT-4], [ALC-5]; round2/inputs/design-K04-method3.md.

## contradictions_between_results

### contradictions_between_results 1
[THR-1] second bullet: three texts (the lead's verdict-165 bullet at HEAD in p11-concurrency.md 95-103; R5 K02's 3 edits marked not_answered; R9 NG-24's 1 edit), none merged, and K02's status contradicts its own input file ('Status: answered by the owner, verdict 165').

### contradictions_between_results 2
[TYP-46]/[ALC-1] `using` sentence: the K05 ruling puts `thread_shared` back in the list (verdict 19) and refuses only 'where the value would hold a link to an instance not every thread may call', while R4's K34 (3 edits) restates the same closed list as a property for String, Deque, etc.; the K05 input says 'it must be answered before K34: its answer decides whether TYP-46's list may be restated as a property'. HEAD [OBJ-4]/[ALC-1]/[TYP-46] still say `E2114`, [TYP-46a] says `E7015`.

### contradictions_between_results 3
[RC-4] point 4 and [OBJ-1]'s header: the K05 ruling ('a value made `using` is in fix 2's state from its making line ... the header stays 32 bytes', the arena word 'placed in the header's queue place', 'any manager but `Global`', correcting N84's 'a value made in `Global` or `System` keeps the two counts', which is HEAD's text) against the K10 ruling ('a value of a type that is not plain nor ... made on a scoped task's or job's thread, a job worker or a C-owned thread ... count in B from the making line', a by-thread and a by-type mark, 'a host thread's values are never queued') — two rewrites of one point and one header, each reported as touching verdict 101, with no merged text.

### contradictions_between_results 4
[THR-9]/[THR-8]/[TYP-43]: the K20 ruling writes `dyn I`'s `ThreadShared` half into [THR-9] and names the header promise in [TYP-43] 'without `unsafe`'; R5's K01 (2 edits on THR-8/9/TYP-36/marker table), R5's K36 (2 edits on THR-8/9/TYP-43, the `unsafe extend` vouch) and R6's K13 (THR-9) edit the same rules; R5's missed item ('`thread_shared fn()` outside `ThreadShared`') is a fifth direction.

### contradictions_between_results 5
[FFI-10]/[FFI-2]/[FFI-21a]/[FFI-56] direction property: the K07 ruling keeps 'results stay outside the limit (137)' and leaves the export/override route 'for K08's lead', while R9/R10 edited those rules for NG-11, 12, 14, 16, 17, 19, 20, 21, 32, 33, 34, 38, 41 (and R7 for K08) to reach 'what an Ember function given to C returns' and what it writes through a `ref mut` C passed; NG-13 and NG-35 (`@ffi(std_function)`) are edited by R9/R10 and also decided by the K07 ruling's ODR-199 change (`unsafe fn(A…) -> R` by default). The refuters marked NG-11 and NG-14 'hole_in_proposed_text'.

### contradictions_between_results 6
[TYP-26]: the K21 ruling (C++ overloads as several first lines, `_mut` names, typed entries) against R10's NG-27 and NG-31 edits ('one override per inherited C++ declaration of a name') and NG-26 (needs_owner, 'the tie-break K47 chooses will have to amend FFI-39b's several fitting equally'); R8's K47 (FFI-42 operator names) is the third hand on the overlay naming.

### contradictions_between_results 7
The two K07 winners differ: one keeps the `retained` row and [FFI-11c] 'as the current text has it' ('verdict-130 leftovers ... a question of its own, not K07's'), the other writes 'the `retained` row's last silent-unsafe line is written `unsafe fn`'; one says '`safe` is [FFI-56]'s existing word', the other '[FFI-11]'s existing word'.

### contradictions_between_results 8
The two K20 winners differ on codes ('E7015 at the derived class ... against E2040 at an `implements Error` whose type is not `Send` ... design 1 used E2040 for both and the lead may unify') and the second cites 'ODR-027 (a derive on a generic type holds per instance: now stated in DRV-1)', but docs/OWNER-QUEUE.md line 55 has ODR-027 as 'a range is a `Copy` value with public bounds'; both judges note designs 2 and 3 cited verdict 132 for 'a declaration means what it says' when JOB.md's 132 is about `@ffi(const)` overlay entries.

### contradictions_between_results 9
[DRP-8]/[THR-19]/[THR-20]: the K10 ruling edits all three (a thread's last check point before its join is let go; 'the only end no line writes changes nothing outside its object'); R3's K11 (2 edits, DRP-8/DSP-2/UNS-1/UNS-12, `unsafe fn drop`) and R3's K23 (4 edits, THR-19 among its rules, a lock guard in a `match` subject) edit the same rules from the readers' side.

### contradictions_between_results 10
[LT-48]: R4's K58 (9 edits, the `runtime` check's effect kind) and R1's K59 (1 edit) edit a rule whose HEAD text already differs from CURRENT-SPEC.md by 69 lines (lead fixes 'a lambda's typed parameter takes it in the same slot', 'before the whole type, the holding word included').

### contradictions_between_results 11
[TYP-36]: R3's K26 (5 edits, `copy of` a view, toward verdict 68), R1's K30 (6 edits, equality and identity of shared owners) and R5's K01 (marker table) edit one rule; K26's refuter says 'hole_in_proposed_text' and no revised text is recorded.

### contradictions_between_results 12
[CLS-12]: R3's K43 (2 edits, a `weakref` into what `@cached` reads) against the K20 judges' flag of the rule's whole-program fact (S2-2, 'no edit here') and R3's missed `static mut` item.

### contradictions_between_results 13
Reader against refuter on status with no settlement recorded: NG-08 (reader needs_owner, refuter 'refuted, not_a_defect'), NG-28 (reader not_answered with 1 edit, refuter 'refuted, answered'), NG-39 (reader not_answered with 1 edit, refuter 'refuted, not_a_defect'), NG-24 (reader answered, refuter 'hole_in_proposed_text'), K29 (reader answered_differently, refuter 'not_answered'); and 'narrowed' with hole_in_proposed_text on K08, K10, K26, K43, NG-11, NG-14, NG-24 where the digest holds no corrected text.

### contradictions_between_results 14
R2's K12 edits (`thread.spawn(owns push_a)`, `push` takes `owns`, the `F`-bounded lending) against the lead fixes already at HEAD in [FN-2a] ('an `owns` parameter of one of these types takes `owns` as every such argument does (`start_worker(owns rx)` ...)' and 'A function from a variable given to a parameter whose type is a type parameter bounded by a function type ... is lent for the call') and [STD-15] (`extend(owns items)`), which CURRENT-SPEC.md lacks: duplicate or divergent wording of one rule.

### contradictions_between_results 15
The K10 judges differ on the task-thread pin: judge 1 writes into [RC-4] point 4 that a scope's task 'runs on a thread the runtime owns, never on the thread that opened the scope'; judge 2 says the ruling 'holds either way (neither kind keeps a home count)' and asks only that [THR-5] be pinned.

## verdict_clashes

### verdict_clashes 1
Verdict 159 ('A constructor's argument takes no word: it works as `=` does ... `w = Words(names=xs)` hands `xs` over'): R2's K19 departs from it 'in a case it did not consider', lending a value given to a written `init` parameter that only reads instead of handing it over; the reader offers the alternative text that keeps 159's letter ('a value given to an `init` parameter with no word is handed over too and read by `init` for the call, so `Label(caption)` ends `caption`'). The lead must choose and report.

### verdict_clashes 2
Verdicts 150 ('A function passed from a variable to a standard-library function is borrowed; the caller keeps it') and 51 ('fn push_a() ... then `a = thread.spawn(push_a)`'): R2's K12 narrows 150 to a parameter the library runs during the call (`F`-bounded) and rewrites 51's line as `thread.spawn(owns push_a)` under 147 and 149; the same narrowing is already at HEAD in [FN-2a] (H.53) as a lead fix, so Hardened_3 must report it as touching 150 and 51.

### verdict_clashes 3
Verdict 164 (`channel[Watcher](capacity=1)` refused, `E7015`): R5's K01 departs 'in its letter only', accepting the channel line and refusing the end's hand-over ([THR-15]) and the `Send` bound at `hand_off(owns w, tx)` (`E2040`).

### verdict_clashes 4
Verdict 101 ('The thread that made the object counts its owners in one number, A'): the K10 ruling makes it untrue for a value of a non-plain type and for a value made on a scoped task's, a job's or a host thread (counted in B from the making line), and the K05 ruling makes it untrue for a value made `using` any manager but `Global`; both report it as N84 did. Verdict 47 (the arena pointer beside the value): the K05 ruling moves it into the header's queue place.

### verdict_clashes 5
Verdict 165: the K05 ruling's E7015 'at a type no value could fill' and the K20 ruling's E7015 at a derived class extend 165's making-line refusal to declaration sites; R5's K02 and R9's NG-24 both state the current text misses 165 although HEAD carries it.

### verdict_clashes 6
Verdict 61 ('A function type says whether any thread may run its functions'): the K20 ruling extends the owner's choice (b, a word, over the compiler finding it) from function types to a header promise on every `pub open` class, and makes `interface Error: needs (Debug, Display, Send, ThreadShared)` (changing the example line of verdict 110); the owner gave no verdict on J6-16, so the ruling is a proposal the owner has not seen, with the stated cost (an error holding a `Cell` refused as an `Error`).

### verdict_clashes 7
Verdict 60 and ODR-199: the K07 ruling changes ODR-199's shape (a `std_function` value's default type becomes `unsafe fn(A…) -> R`, `safe` on the entry giving `fn(A…) -> R`), 'the one change that refuses programs 0.9.20_Hardened_2 accepts'; a delegated ruling, not an owner verdict, but it lands in the owner's N75 migration rule (`ember fmt --migrate` rewrites nothing).

### verdict_clashes 8
Verdict 86 ('`changes` and `reads` are removed; the compiler works out what every function changes'): the K21 ruling's two-name `_mut` split is chosen because 'every such rule found reads either `this`'s kind (circular) or the result's use', a departure from the method-3 lead's decision 4 (option b), not from the owner; it also introduces names the header never wrote (`at_mut`), which the owner's 'Python-like: what a programmer means stays written down' may or may not accept; no ODR records it.

### verdict_clashes 9
Verdict 135 (the default manifest's note): N83 of Hardened_2 reversed half of it ('reported'); no round-2 result returns to it and the owner has not answered.

### verdict_clashes 10
Verdict 68 ('A `Copy` type always works with `copy of` ... E2116 is not given for it'): R3's K26 brings [OWN-8] and [TYP-36] to the verdict (they give E2116 for `StrView` and `Span[T]` today), but the refuter marks the proposed text 'hole_in_proposed_text' and the digest records no corrected edit, so the text may still clash with 68 after the cut.
