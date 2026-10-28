# 24 Faithfulness reviewer: is the C of W1-W4 what the final model would emit? (run 3, phase 4)

Power: `PowerLineStatus` printed `Online` at the start, before each of my two timing sets, and before
writing this report. No compiler change, no git command, no Ember program run. Nothing another agent wrote
was edited: I copied `w3.c`, `w4.c` and `ember_rt.h` from `target/redesign3/23-c-writer/` into
`target/redesign3/24-faithfulness/` with `cp` and changed only the copies.

Inputs: the model `RUN3/combined-v4.md`, the C writer's report `RUN3/23-c-writer.md`, its code
`target/redesign3/23-c-writer/` (`w1.em`..`w4.em`, `w1.c`..`w4.c`, `ember_rt.h`), and the spec
`docs/spec-source/Ember_v0.9.10_Hardened_35.md` for the language checks.

## 0. Terms

- **check**: code that runs while the program runs and stops it when something is wrong (an index past
  the end, an overflow, a zero divisor).
- **memory-model work**: what the model itself adds while running: a weak count changed, the "ended"
  (DEAD) test on reading a `weak` link, a count held across a call, a box for a captured variable.
- **range fact**: a number the compiler knows is inside a range, which removes a check. The spec's list
  (`[RNG-4]`, spec line 1047): literals, `min`, `max`, `clamp`, the arms of a comparison, arithmetic on
  known ranges, and the variable of a `for i in a..b` loop. A function's result is not on the list.
- **floor `%`**: Ember's `%` on signed integers rounds like Python (`[TYP-28]`, line 969): one compare and
  one add more than C's `%` unless the left side is known to be `>= 0`.
- **favours the model**: the C leaves out something the model would emit, so the model looks faster than
  it is. **unfair to the model**: the C has something the model would not emit.
- **model contradiction**: two places in the model say different things, so no C can follow both; the
  lead (or the owner) must rule, not agent 25.

## 1. Answer in one paragraph

23's C is faithful on almost every line: every language check the spec keeps is there (bounds
`[TYP-31]`, overflow `[TYP-8]`, zero divisor `[TYP-28]`, `unwrap`, float-to-int saturation `[TYP-6]`,
count overflow `[RT-7]`, spec line 6076), the memory-model work the model's table 1.2 lists is there, and
nothing is there that the model would not emit apart from four small items. I found **ten gaps**: four
that favour the model (one of them measurable, W4's boxes, about 1.10x), three that are unfair to it (all
too small to see), two model contradictions that 25 cannot settle (W2's per-hop DEAD test, which decides
W2's ratio, 1.40x; and W3's noalias promise, which is a safety question with no speed effect), and one
spec gap (no rule says a length is `>= 0`). Plus two cautions for the timer about the same-layout plain C.

## 2. The gaps

Each: where, what the model says, what the C does, which way it leans, what it costs (measured where it
could matter), and what to do. Timings are single runs through `run.ps1` (pinned, mains checked), three
rounds, to size a gap, not the timer's numbers (section 4).

### Gap 1 (W2, model contradiction, decides W2's ratio): the DEAD test on every hop of the walk

- Where: `w2.c` line 110, `for (Node *m; (m = weak_read(a)) != NULL; a = m->parent)`.
- The model says two things. R11 (`combined-v4.md` line 399): "`weak T` reads as `Option` ..., one test
  of the target's header per read", and table 1.2 (line 539) "one test of the target's header | each read
  of a `weak` link". `a = copied n.parent` is a `weak Node` of the walk's own (R2, line 87-88), not a
  `set by` link, so every `while Some(m) = a:` reads a `weak` link and tests. But AN-62, whose walk W2's
  is (line 8415-8419, the same `a = copied n.parent` / `a = copied p.parent`), says "the C unchanged"
  (line 8423), and that C reads `a` with no test (`11-area-D/c/an62_defer_asan.c` lines 82-88,
  `an62_walk_bench.c` line 125); its cost claim, 0.944 of plain C (line 8510, and table 1.2 line 538), was
  measured without the test. AN-62's own prose needs the test, though: "marked dead, so `weak` reads of it
  give `None`" (line 8440).
- 23's C follows R11, the rule's literal text. That is faithful to the rules; it is not a C-writer error.
- Cost of the test (my runs, `24-faithfulness/variant_runs.txt`): `model_w2` 1088.5, 1069.7, 1103.0 ms;
  23's `diag_w2_hop_proved` (only the hop test removed, every other check kept) 773.8, 786.8, 776.5 ms.
  Medians 1088.5 / 776.5 = **1.40**. 23 measured the same (`23-c-writer/diag_w2_output.txt`).
- Is the test needed in W2? No: `a` is copied from a `set by` link, whose target is alive while it holds
  the node (D7 rule 5, line 7417), and no line of the walk can end a `Node` (R10 point 1, line 387). The
  model has no rule that removes a test proved this way. 23's section 4 proposes one; it needs no new
  word.
- To do: **the lead rules** which C is the model's W2: R11 as written (`model_w2`), or AN-62's C (then
  R11 needs the sentence 23 proposes, and `diag_w2_hop_proved` is the model). The timer should time both,
  labelled. AN-62's 0.944 and table 1.2's "none in AN-62's walk" do not include this test either way.

### Gap 2 (W4, favours the model, measurable): `counter` and `m` in the handler's state block, not a box each

- Where: `w4.c` lines 41-43 (the `State` struct) and 91-92 (`make_handler`); 23's choice C2.
- The model: D1 (line 7364) "a box otherwise (the function value is returned, ...)"; the box is "a small
  counted heap block holding **one** captured variable" (line 7316); table 1.2 (line 543) "one allocation
  per captured variable instance"; R12 (line 408) the same. Nothing in the model says that variables only
  one function value captures may share its state block. AN-53's case 2 (`11-area-D/c/an53_captures.c`
  lines 49-55) makes the one box the state, for a function value with one capture; it says nothing about
  three.
- The C puts `counter` and `m` in the 32-byte state block and only `total` in a box. The model's text
  gives each a box: two more slab blocks per handler made and one more load per call for `counter`.
- Cost (`variant_runs.txt`): my `v24_w4_box_per_capture` (`24-faithfulness/w4.c -DBOXES`: the same 32-byte
  state holding three box pointers; `counter` and `m` each in a 16-byte counted box from a slab, released
  with the state) ran 29.6, 28.9, 28.9 ms against `model_w4` 26.8, 25.5, 26.2 ms in the same rounds:
  medians 28.95 / 26.16 = **1.11**. Peak memory 4,912 / 4,536 / 4,536 KB against 4,628 / 4,268 / 4,264 KB.
  Checksum 34946993000 in every run.
- 23's placement cannot change what the program does (only this handler reaches `counter` and `m` once
  `make_handler` returns), so it is a good save; but it is not written in the model.
- To do: either the lead adds the placement to D1 (one sentence, no word: "a captured variable that no
  other function value and no live frame shares lives in the function value's own state block"), and
  23's `w4.c` stays the model; or 25 makes the box-per-capture build the model's W4 (mine can be used as
  it is). The timer should time both until then.

### Gap 3 (W2 and W3, favours the model, negligible): floor `%` left out where the left side is a call's result

- Where: `w2.c` lines 96 and 123 (`h_rand() % i`); `w3.c` lines 89, 90 (`rand_int() % 1000`), 107
  (`rand_int() % i`) and 131 (`rand_int() % objects.len`).
- `rand()` returns `int`. `[RNG-4]` gives no range fact for a function's result, and 23 says so himself
  (report section 0: "a `bool` returned by a call proves nothing in the caller"; `w1.c` line 81). So the
  sign of `rand()` is unknown at these lines and Ember's floor `%` needs its compare-and-add (`FMOD`). The
  C uses C's `%` with the comment "rand() >= 0", a fact only `rand`'s body knows. At `w3.c` line 131 the
  missing bounds check is proved only by floor `%` (a positive length gives a result in `[0, len)`), so
  with C's `%` the proof itself rests on the missing fact.
- Cost: one compare and add per node built and per re-parent (W2), per object made (W3); outside the hot
  loops. Not measured; nothing in the hot loops changes.
- To do (25): `FMOD` at those six sites. The zero-divisor proofs stay (they come from `i >= 1`, `i > 0`
  and `len(objects) > 0`, all comparison or loop facts).

### Gap 4 (W4, favours the model, small): the bounds check of `self.handlers[self.current]`

- Where: `w4.c` line 121-122. 23: "current is `k % len`, just stored, below `len`".
- The proof needs `len(self.handlers) > 0`, of which the zero-divisor check gives only `!= 0`; `> 0` needs
  "a length is `>= 0`", which `[RNG-4]` does not list (gap 10), and the fact must survive the store to
  the field `self.current` and its load (`[RNG-4]` speaks of expressions; it says nothing about fields).
  Read literally, the check stays.
- Cost (`variant_runs2.txt`): `v24_w4_dispatch_bounds` (`w4.c -DDISPATCH_BOUNDS`, one compare per
  dispatch) 27.8, 26.6, 27.4 ms against the same file without it 26.7, 26.3, 26.5 ms: medians 27.43 /
  26.48 = **1.04**.
- To do: if the lead accepts gap 10's rule ("a length is `>= 0`") and facts through a local, 25 can keep
  the elision by writing `current = k % len(self.handlers)` to a local first; otherwise 25 adds the check.

### Gap 5 (W2, the model emits it, never taken): the drop of `n` when the build's `nodes[...]` read gives `None`

- Where: `w2.c` line 99, `if (p) children_push(p, n);`. In `w2.em` line 28-29, when `Some(ref mut p)` does
  not match, `n` is not handed over and ends at the end of the loop body: the model emits its drop
  (`else node_drop(n)`), as 23 did for `moved` at line 127.
- Cost: none (never taken in W2; one branch already present).
- To do (25): add `else node_drop(n);` for completeness.

### Gap 6 (W2, unfair to the model, negligible): the bounds check of `nodes[i]` in the re-parent loop

- Where: `w2.c` line 116, `[K8]`, per iteration that passes `(i + p) % 100 == 0`.
- `[OPT-2]` (spec line 3389): for a counted loop `a..b` indexing at `i + c`, with the list's base and
  length invariant in the loop, the compiler "MUST emit one entry test". `for i in 1..N:` indexes
  `nodes[i]`; the body changes only `children` lists, never `nodes`. So the model emits one test per pass
  (`N - 1 < len(nodes)`), as 23 did for W1 (`[K14]`, `w1.c` line 109).
- Cost: about 10,000 compares per pass, against 1,000,000-node walks. Not measurable.
- To do (25): one test before the loop, none inside.

### Gap 7 (W4, unfair to the model, negligible): `total` in a box

- Where: `w4.c` lines 39, 43, 88-89, 131-133; 23's choice C3.
- D1 (line 7360): "an address when the compiler proves the function value ends before the variable
  does: it is kept only in locals, local lists, or objects that never leave the function, and every
  method it is handed to says what it changes". For `total` (main's variable) that holds: every handler
  lives in `d`, a local of `main` declared after `total` (so it ends first), handed only to `register`
  (`changes handlers`) and `dispatch` (`mut self`). The bullet "the function value is returned" is about
  the function that owns the variable; `make_handler` returns the handler to `main`, which owns `total`.
  So by D1's own test the address is allowed; last run used it (`phase3b-combined.md` 6.4).
- Cost: one count add per handler made (10,100) and one subtract per handler dropped; per call the load
  count is the same (`st->total->value` against `*st->total`). Not measurable.
- To do (25): `total` by address. Optional; it changes nothing measurable.

### Gap 8 (W3 and W4, model contradiction on W3, no speed effect): noalias on `mut self` and `ref mut` parameters

- Where: `w3.c` lines 70, 77, 80 (`ship_update`, `rock_update`, `turret_update` take `Obj *self`); `w4.c`
  lines 96 and 118 (`Dispatcher *d`); 23's choice C8.
- The model: R4 (line 244) "Every `ref mut` parameter is compiled with noalias"; open point 7 (line 813)
  treats `mut self` as `ref mut self`. In `Turret.update` and `Ship.update`, `t` is reached through
  `self.target`, a `weak GameObject`, and may be `self` (a turret targeting itself: never in W3, allowed by
  the language). R9 (line 375) "Only parameters keep the 'different places' promise", and R4's compare
  covers two `ref mut` arguments of a call or "two `ref mut` element names" (line 245), not a parameter
  and a name reached through a link. So the model as written emits noalias on `self` and no compare: C
  with undefined behaviour for a self-targeting turret. That is a hole in the model, for the lead, not a
  C-writer gap. 23 emitted neither noalias nor a compare (a third form the model does not describe).
- Cost (`variant_runs.txt`, `variant_runs2.txt`): `v24_w3_noalias_only` (`w3.c -DR4=1`) 449.4, 417.6,
  434.7 ms; `v24_w3_noalias_compare` (`-DR4=2`, noalias plus one compare `t != self` per read) 446.1,
  466.6, 426.3 ms; `model_w3` 426.8, 434.9, 425.2 and 418.0, 415.8, 442.9 ms; my unmodified copy of
  `w3.c` 462.3, 427.0, 466.3 ms. W3's run-to-run spread is about 10%, larger than any difference here: the
  choice does not show in W3's time. W4 not measured (the handler body is small; W3 suggests nothing).
- To do: **the lead rules** (the sound reading is noalias plus R4's compare whenever a `ref mut` or
  `mut self` parameter and a name reached through a `shared`/`weak` link of the same type are both used).
  25 need change nothing for timing.

### Gap 9 (W1, W3: unclear, rare paths): overflow checks on `len - 1` and `before - len`

- Where: `w1.c` line 55 (`last = len(self.ents) - 1`, per remove) and `w3.c` line 125
  (`dead = before - len(objects)`, per frame). 23 cites "`[STD-19]` known numbers" for line 55; `[STD-19]`
  (the iterator adapters) says nothing about lengths.
- Both are safe only if a length is known `>= 0`, which `[RNG-4]` does not list (gap 10). Read literally,
  each needs one overflow check.
- Cost: one compare per despawn (W1) and per frame (W3). Not measurable.
- To do: follows gap 10's ruling; if none, 25 adds the two checks.

### Gap 10 (spec, for the lead): no rule says a length is `>= 0`

Gaps 4 and 9 turn on it. Every Ember list's `len()` is `int` (`[TYP-31]`) and can never be negative, but
`[RNG-4]`'s list of facts does not include it. Every real compiler knows it (Rust's `len()` is `usize`).
One line in `[RNG-4]` ("a container's `len()` is in `[0, int.MAX]`") settles gaps 4 and 9 in the model's
favour, with no word added.

## 3. Checked and found faithful

- **W1** (`w1.c`): every bounds and overflow check listed in 23's table 3.1 is where the spec puts it;
  `[OPT-2]`'s entry test `[K14]` matches the spec (its "checked body otherwise" would stop at the first
  index anyway, since the loop runs from the top index down, so a panic before the loop does the same);
  `as u32` and `as int` between integers keep the low bits and need no check (`[TYP-6]`, line 939);
  float to int saturates (`[K17]`, `[TYP-6]` line 940); `pos` (`ref mut reg.pos.dense[k]`) and `vel` (of
  `reg.vel.dense`, another list) are different places, proved, so R4 needs no compare; the `dead` list and
  the despawn after the loop are R8 (c) as agreed (AN-04, AN-06). Nothing missing, nothing extra.
- **W2**: no strong count anywhere (one owner per node, hand-over between `children` lists), the `set by`
  stores on `push` and `remove` (D7 rule 2), the plain load of `node.parent` (D7 rule 5), one weak count
  per `nodes` element, no weak count on the walk's copy `a` (R2 as 21 wrote it, line 90: R10 point 1
  covers its whole life), no count pair and no region (R10 point 1: no line of the walk or of the
  re-parent's views can end a `Node` while they are used; `moved` ends only on the branch where none of
  them is used). Language checks as 23's table 3.2, except gaps 3, 5, 6.
- **W3**: one weak count per stored link, the DEAD test per `self.target` read (R11, AN-65), no count on
  `t` (R10 point 1), the end of a dead object (DEAD set, its own link's weak count given back, the block
  kept while weak links remain, R2), two passes (loop, then `retain`): AN-04's fusion condition (3) fails
  because the `retain` test reads `health`, which turrets change during the loop (lines 1566-1567), and (5) fails
  because ending an object sets DEAD, which later weak reads would see. Language checks as 23's table
  3.3, except gap 3.
- **W4**: the two-word copy out of the list per call with no count (D4, R12, AN-60 line 8199), the
  handler's hold on its own state only across `unregister_current` when it uses the state after it (D4,
  R12 line 412, as AN-60's own C, lines 8205-8208), the drop of the removed value at `remove`, the
  zero-divisor check of `k % len` (no fact gives `len != 0`), floor `%` on the parameter `k`. Language
  checks as 23's table 3.4, except gaps 2, 4, 7.
- **All**: the 12-byte header (choice C1) costs nothing: with packing `Node` is 48 bytes and `GameObject`
  40, the sizes an 8-byte header gives, so the model's R9 is met in size; the model should still say where
  the weak count lives. Count overflow checks are the spec's (`[RT-7]`, line 6076). Not emitting `main`'s
  drops after its last line (C10) matches plain C. 22's open problems G-01 to G-05 do not touch these
  programs (G-01 needs a target that ended before the region; no `Node` ends in W2's walk).
- Every checksum in my runs equals plain C's: W1 2111831520, W2 647034814, W3 9288233970, W4 34946993000.

## 4. Two cautions for the timer (26) and the lead

1. **W2's same-layout plain C is not a floor.** In my rounds `plain_same_w2` (no checks, no weak work)
   ran 892.7, 862.6, 831.9 ms, slower than `diag_w2_hop_proved` (which keeps the overflow checks, the
   weak counts and every other DEAD test) at 773.8, 786.8, 776.5 ms; 23 saw the same (746.5-814.4 against
   677.7-706.6, `diag_w2_output.txt`). A program doing strictly more work ran faster, so code placement,
   not work, decides part of W2's time; a ratio to `plain_same_w2` alone can mislead. Report W2 against
   both `plain_same_w2` and the baseline `bench/w2.c`.
2. **W3's `-DPLAIN` keeps the model's weak counts and DEAD tests** (23's `w3.c` header lines 25-27), so
   `model_w3 / plain_same_w3` measures only the language checks, not the model's link mechanism. The
   model's cost of links shows only against the baseline `bench/w3.c` (watcher lists).

## 5. What 25 should do, in order

| # | workload | change | from gap |
|---|---|---|---|
| 1 | W4 | build the box-per-capture W4 as the model's (or keep 23's if the lead writes the placement into D1); my `24-faithfulness/w4.c -DBOXES` is that build | 2 |
| 2 | W2, W3 | `FMOD` at `w2.c` 96, 123 and `w3.c` 89, 90, 107, 131 | 3 |
| 3 | W4 | the bounds check at `w4.c` 122, unless the lead rules gap 10 | 4 |
| 4 | W2 | `else node_drop(n);` at `w2.c` 99 | 5 |
| 5 | W2 | `[OPT-2]`: one entry test before the re-parent loop instead of `[K8]` per iteration | 6 |
| 6 | W4 | `total` by address (optional, not measurable) | 7 |
| 7 | W1, W3 | overflow checks at `w1.c` 55 and `w3.c` 125, unless the lead rules gap 10 | 9 |

For the lead, not 25: gap 1 (which W2 is the model's; time both), gap 8 (the noalias hole in R4/R9),
gap 10 (lengths `>= 0`).

## 6. Files

All under `C:\Users\ism19\Code\ember\target\redesign3\24-faithfulness\`:

| file | what |
|---|---|
| `ember_rt.h` | copied unchanged from 23 |
| `w3.c` | 23's `w3.c` copied; `-DR4=1` adds noalias on `self`, `-DR4=2` noalias plus the compare `t != self` (gap 8); without either it is 23's program |
| `w4.c` | 23's `w4.c` copied; `-DBOXES` one box per captured variable (gap 2); `-DDISPATCH_BOUNDS` the bounds check of gap 4; without either it is 23's program |
| `bin/` | the builds (`clang -O2 -I target/redesign/bench`): `model_w1..w4.exe`, `plain_same_w2..w4.exe`, `diag_w2_hop_proved.exe` from 23's sources; `v24_*.exe` from mine |
| `variant_runs.txt`, `variant_runs2.txt` | the two sets of three rounds through `target/redesign2/bench/run.ps1` quoted above |
