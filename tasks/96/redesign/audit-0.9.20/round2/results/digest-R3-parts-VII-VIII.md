# R3-parts-VII-VIII

could_not_cover: Nothing in the thirteen clusters was left unread; every rule each cluster names was read in CURRENT-SPEC.md with its registry row, together with JOB.md verdicts 1, 5, 10, 18, 23, 24, 25, 27, 32, 34, 43, 47, 53, 60, 68, 78, 79, 93, 101, 103, 107, 118, 119, 137, 148 and 164 to 166, decisions.md §12, §13 and the area-2 rulings, and the K04 and K05 design notes. Not verified against the compiler (the brief forbids running anything): whether today's compiler accepts `thread_shared X() using inst`, and the LT-22 conformance test `rest = this.rest` that K26 cites, both as the K04/K05 notes and the J4-08 checker reported them. Two things found on the way that no cluster of mine asks: [THR-19]'s holding forms miss a guard that is a temporary in any statement, not only a `match` subject (written into K23's edits because the K23 program needs it); and [DSP-4]'s interface-type list leaves out `shares mut dyn I` that [TYP-44] has (written into K35's edit). Verdicts 164 to 166 touch none of these clusters beyond K05, where verdict 165's `thread_shared [..] using frame` names a list's buffer, which [TYP-46] still allows and [TYP-46a] decides, so the K05 edits leave it as it is.

## K04-weakref-outlives-arena  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: ALC-5a: "A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one, since the target's block stays until the link ends (`[WK-11]`, `[OBJ-3]`): the link carries the instance's region from the line that makes it (`w = weakref n`) through every place it is copied or stored, as a kept view does (`[LT-45]`); keeping it where it would outlive the instance is `E3063` ("`w` links to a `Node` made in `frame` (line 10), and `frame` ends when `main` returns; end the link first: write `w = None` above the return"), and while it lives a change of the instance that `[ALC-5]` does not allow is `E3021`"; ALC-5a: "A field or element of a `weakref` type whose target was made in an instance carries that instance's region as a kept view field does (`[LT-45]`), so the object or list that holds it lives no longer than the instance (`E3063`). Nothing runs for this: the block a link keeps is given back by the manager's own `deallocate` when the weak count reaches 0, as any other, which for a manager that frees nothing one by one is a no-op, so this check is what keeps the block from reuse."; LT-4: "A value made in a memory manager's instance (`owns Mesh(points=[0.0, 1.0]) using current`, `[ALC-1]`), and every value that holds one, lives no longer than that instance, wherever it is kept: keeping or returning it where it would outlive the instance is `E3063`"; OBJ-3: "the block is freed when the weak count reaches 0, which for a value no `weakref` link may reach is at once."; E3063 registry row: "a `weakref` link, or a value holding one, kept where it would outlive the instance its target was made in"

**program**: 

**remaining_doubt**: J5-02's program is now E3063 at `cam.target = weakref boss` (the link in `cam`, a `ref mut` parameter of the caller's object, would outlive `level`), and the level-arena layout with the arena made in the caller compiles, which is the K04 design note's option A as written into [ALC-5a] in Hardened_2. The only words left loose are [LT-4]'s own, which still say "every value that holds one" with [ALC-5a] doing the extension beside it; one defining rule is kept because [ALC-5a] names [LT-4] and [ALC-5] as the rules it extends and the registry rows of E3063 and E3021 cite it.

**REFUTER reason**: [ALC-5a] decides it and decides it on the refusing side: "A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one ... keeping it where it would outlive the instance is `E3063`", and "A field or element of a `weakref` type whose target was made in an instance carries that instance's region as a kept view field does (`[LT-45]`), so the object or list that holds it lives no longer than the instance (`E3063`)". J5-02's `cam.target = weakref boss` stores the link through the `ref mut cam` parameter into the caller's object, which the E3063 registry row already covers ("a view of the callee's own locals put into the caller's view through `ref mut`" and "a `weakref` link, or a value holding one, kept where it would outlive the instance its target was made in"), and a reset under a live link is `E3021` ("while it lives a change of the instance that `[ALC-5]` does not allow is `E3021`"). The give-back is defined too: "the block a link keeps is given back by the manager's own `deallocate` when the weak count reaches 0, as any other, which for a manager that frees nothing one by one is a no-op". Nothing runs. [LT-4]'s own words ("every value that holds one") are extended by name from [ALC-5a], so one defining rule stands; the reader's remaining doubt is wording only.

## K05-thread-shared-using  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: TYP-46: "after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`"; ALC-1: "never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`"; OBJ-4: "A `thread_shared` value's block always comes from the runtime allocator (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is `E2114` ("a `thread_shared` value's block carries its making thread's record, which only the runtime allocator keeps; remove `using m`: `names = thread_shared Names()`")."; OPT-1: "A `thread_shared` value is never placed in a frame, and never in a memory manager's instance: its making thread is read from its chunk (`[RC-4]`), which only the runtime allocator's blocks have."; OBJ-1: "A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4) and no thread asks which thread made it."; RC-4: "A `thread_shared` value made `using` an instance of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that makes it: its instance may be reset or end as soon as the compiler sees its last owner end (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such a value is counted in its away count and its end is never delayed; a value made in `Global` or `System` keeps the two counts, its block waits for the check point and nothing reuses it."; TYP-46a: "`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making line ("`st` would end on whichever thread holds its last owner, and `current`, an `Arena`, is used by one thread; make it in the built-in manager, `st = thread_shared Stats()`, or in a manager any thread may call")."; ALC-5: "A `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4), so the rule is checked as written"

**program**: The implementer's question, which the current text answers three ways. For `stats = thread_shared Stats() using System` (System: one instance, `ThreadShared`): [OBJ-4], [TYP-46], [ALC-1], [OPT-1] and the E2114 registry row ("`thread_shared` before a value made `using m`, whose block would carry no making-thread record") refuse it with E2114, "whatever manager its class names"; [RC-4] point 4 describes it as accepted ("a value made in `Global` or `System` keeps the two counts"), [OBJ-1] describes a `thread_shared` value made `using` an instance as "marked handed over at the line that makes it", [TYP-46a] refuses it only for an `A` that is not `ThreadShared` (E7015, with `using System` implied accepted), and [ALC-5] describes how such a value is ended. Which code, if any, does the line get, and does `thread_shared Stats() using level` (an `Arena`) get E2114 ([OBJ-4]) or E7015 ([TYP-46a])? The E7015 registry row also lists "`thread_shared T using A` for an `A` that is not `ThreadShared`" beside E2114's row for the same line.

- READER EDIT rule=OBJ-1
  - old: A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new: No `thread_shared` value is made `using` an instance (`E2114`, `[OBJ-4]`), so every `thread_shared` block lies in a chunk whose header names its making thread.
  - why: The lead's N84 wrote both halves of O8 into the text: the refusal ([OBJ-4], [OPT-1], [TYP-46], [ALC-1]) and a run-time path for the refused construct ([OBJ-1], [RC-4] point 4). The refusal is the safe and cheaper answer (one way to find the making thread, no handed-over-at-birth path, no manager `deallocate` on a foreign thread, the K05 design note's option A); the description of the other path must go so that the rules agree.

- READER EDIT rule=RC-4
  - old: A `thread_shared` value made `using` an instance
     of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that
     makes it: its instance may be reset or end as soon as the compiler sees its last owner end
     (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: Every `thread_shared` value's block comes from the runtime allocator (`[OBJ-4]`; `using` after `thread_shared` is `E2114`), so its block waits for the check point and nothing reuses it.
  - why: Point 4 describes counting for a value [OBJ-4] refuses; an implementer cannot build a path for a program the compiler never accepts, and the sentence implies `using System` is accepted.

- READER EDIT rule=TYP-46a
  - old: `thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making
  line ("`st` would end on whichever thread holds its last owner, and `current`, an `Arena`, is used
  by one thread; make it in the built-in manager, `st = thread_shared Stats()`, or in a manager any
  thread may call").
  - new: `thread_shared T using A` is `E2114` for every `A` (`[OBJ-4]`, `[TYP-46]`), so no `thread_shared` value holds a `ref A`.
  - why: One line, one code: [OBJ-4] gives E2114 to the same line for every manager; the E7015 sentence contradicts it and implies `using System` compiles.

- READER EDIT rule=ALC-5
  - old: A
  `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4),
  so the rule is checked as written; one handed to an unscoped thread (`[THR-2]`) lives, for this
  rule, as long as the instance, so no change of this rule's kind is made to that instance after the
  hand-over (`E3021`, naming the hand-over's line).
  - new: No `thread_shared` value is made in an instance (`E2114`, `[OBJ-4]`), so every value made in one is ended on the thread that holds its owner and the rule is checked as written; a value made in an instance and handed to an unscoped thread (`[THR-2]`, which `[TYP-46a]` allows only for a `ThreadShared` manager) lives, for this rule, as long as the instance, so no change of this rule's kind is made to that instance after the hand-over (`E3021`, naming the hand-over's line).
  - why: The sentence describes the end of a value the text refuses to make; the hand-over clause is kept for the one-owner value [TYP-46a] lets cross threads.

- READER EDIT rule=registry row E7015
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: 
  - why: The row names a refusal that the E2114 row already gives the same line; one code per line ([DIA-6a]).

**remaining_doubt**: The choice between refusing every manager (the four refusing rules, the K05 design note's option A) and allowing a `ThreadShared` one-instance manager (what [RC-4] point 4 and [TYP-46a] imply) is safe either way; the edits follow the refusing rules because they are four against three, carry the E2114 registry row, and keep verdict 101's one making-thread path. If the lead meant the allowing design, the edits go the other way ([OBJ-4], [OPT-1], [TYP-46], [ALC-1] and the E2114 row), and [OBJ-1]'s handed-over-at-birth sentence then needs the `deallocate`-on-another-thread rule [TYP-46a] gives. [THR-1]'s program of verdict 165 is untouched: `thread_shared [..] using frame` names the list's buffer, which [TYP-46] still allows and [TYP-46a] decides by `Send`/`ThreadShared` of the manager.

**REFUTER reason**: The contradiction is real. Four rules and the registry refuse: [OBJ-4] "A `thread_shared` value's block always comes from the runtime allocator (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is `E2114`"; [TYP-46] "after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`"; [ALC-1] "never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`"; [OPT-1] "never in a memory manager's instance". Five places describe the accepted path: [OBJ-1] "A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it"; [RC-4] point 4 "A `thread_shared` value made `using` an instance of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that makes it ... a value made in `Global` or `System` keeps the two counts"; [TYP-46a] "`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making line"; [ALC-5] "A `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4)"; and, which the reader missed, the §X.4 row "A `thread_shared` value's two counts": "a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4)". An implementer cannot give `thread_shared Stats() using System` a code. Both designs are safe (the allowing one counts away from birth and never reads a chunk header), so the choice is consistency with [OBJ-1]'s one path and the K05 design note's option A (decided, 7 agents), which the reader follows rightly. Two narrowings. (1) [OBJ-4]'s sentence is wider than its intent: by [GRM-43] row 18 `thread_shared [String("a")] using frame` is `thread_shared ([..] using frame)`, "a value made `using m`" with `thread_shared` before it, so [OBJ-4]'s words give `E2114` to the list of verdict 165 that the design note keeps ("a list's own buffer ... decided by Send and ThreadShared as `new_in(frame)` is") and that the reader says [TYP-46a] still decides; after the reader's deletion of [TYP-46a]'s `E7015` sentence nothing decides it but [OBJ-4]'s over-wide `E2114`. The sentence must name whose block the word names, and [TYP-46a]'s sentence is reworded for the list case instead of deleted. (2) The §X.4 row keeps the allowing path and must change with [RC-4] point 4. The reader's edits to [OBJ-1], [RC-4] and [ALC-5] stand.

- BETTER EDIT rule=OBJ-1
  - old: A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new: No `thread_shared` value is made `using` an instance (`E2114`, `[OBJ-4]`), so every `thread_shared` block lies in a chunk whose header names its making thread.
  - why: The reader's edit, kept: the refusal is the design chosen and the handed-over-at-birth path is the other half of N84.

- BETTER EDIT rule=RC-4
  - old: A `thread_shared` value made `using` an instance
     of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that
     makes it: its instance may be reset or end as soon as the compiler sees its last owner end
     (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: Every `thread_shared` value's block comes from the runtime allocator (`[OBJ-4]`; `using` before it is `E2114`), so its block waits for the check point and nothing reuses it.
  - why: The reader's edit, kept.

- BETTER EDIT rule=ALC-5
  - old: A
  `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4),
  so the rule is checked as written; one handed to an unscoped thread (`[THR-2]`) lives, for this
  rule, as long as the instance, so no change of this rule's kind is made to that instance after the
  hand-over (`E3021`, naming the hand-over's line).
  - new: No `thread_shared` value is made in an instance (`E2114`, `[OBJ-4]`), so every value made in one is ended on the thread that holds its owner and the rule is checked as written; a value made in an instance and handed to an unscoped thread (`[THR-2]`, which `[TYP-46a]` allows only for a `ThreadShared` manager) lives, for this rule, as long as the instance, so no change of this rule's kind is made to that instance after the hand-over (`E3021`, naming the hand-over's line).
  - why: The reader's edit, kept.

- BETTER EDIT rule=OBJ-4
  - old: whatever manager its class names: `thread_shared` before a value made `using m` is
  `E2114`
  - new: whatever manager its class names, `System` and `Global` included: `thread_shared` before a value whose own block `using m` would name (a class object, `thread_shared Stats() using level`; a struct the word puts in a block, `thread_shared Mesh(points=[0.0]) using level`) is `E2114`
  - why: The sentence as written also hits `thread_shared [..] using frame`, where by [GRM-43] row 18 the word names the list's buffer and the shared block still comes from the runtime allocator; the refusal names whose block the word names, so verdict 165's list is decided by [TYP-46a], not refused here.

- BETTER EDIT rule=OBJ-4
  - old: 
  - new: Before a list, map, set or string made `using m` the word names their buffer (`[GRM-43]` row 18; `[TYP-46]`: what a value holds comes from where its own type says), the shared block still comes from the runtime allocator, and the line is decided by `[TYP-46a]`, as `thread_shared Array[String].new_in(m)` is.
  - why: States the one case the refusal does not reach, so [OBJ-4] and the design note agree.

- BETTER EDIT rule=TYP-46a
  - old: `thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making
  line ("`st` would end on whichever thread holds its last owner, and `current`, an `Arena`, is used
  by one thread; make it in the built-in manager, `st = thread_shared Stats()`, or in a manager any
  thread may call").
  - new: `thread_shared` before a list, map, set or string made `using A` (the word names their buffer, `[OBJ-4]`) with an `A` that is not `ThreadShared` is `E7015` at the making line ("`names` would grow and end on whichever thread holds its last owner, and `current`, an `Arena`, is used by one thread; make the list in the built-in manager, `names = thread_shared [String("a")]`, or in a manager any thread may call"); `thread_shared T using A` for a value whose own block the word would name is `E2114` for every `A` (`[OBJ-4]`), so no such value holds a `ref A`.
  - why: Reworded, not deleted: the `E7015` case is real for the list whose buffer is in a one-thread manager, and the `E2114` case is named beside it so the two codes never meet on one line.

- BETTER EDIT rule=§X.4 row "A `thread_shared` value's two counts"
  - old: (`[OBJ-1]`); a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4); on the making thread
  - new: (`[OBJ-1]`), and every `thread_shared` block is such a block (`[OBJ-4]`: `using` before it is `E2114`); on the making thread
  - why: The cost row still describes the handed-over-at-birth path the edits remove; a row that names a path the compiler never emits is a cost with no program.

- BETTER EDIT rule=registry row E7015
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: `thread_shared` before a list, map, set or string made `using A` for an `A` that is not `ThreadShared`;
  - why: The row follows the reworded [TYP-46a] instead of vanishing.

## K06-sharing-a-local  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: RC-1: "`shares x` and `shares mut x` are written before a new value, a local of this function that owns its value (which becomes one of the owners: a value that is not a class object is then made in a block of its own from the line that declares the local (`xs = make()` puts the result in that block), as if that line had said `shares`, so nothing moves and no view of it ends at the `shares` line, and the local's type is the shared type throughout the function) or a name that is already a `shares` or `shares mut` owner"; RC-1: "`thread_shared x` is written only before a new value or a name that is already a `thread_shared` owner (`E7017`, `[OWN-13]`, `[THR-1]`), since its header and its making thread's record are written where the value is made"; THR-1: "it is written before a new value (`st = thread_shared Stats()`) or on a value already `thread_shared` (`fn() thread_shared st:`), and on a value made otherwise it is `E7017`, as `shares` or `shares mut` on a `thread_shared` owner is (`[RC-1]`) ("`names` is `share_and_keep`'s own object, which it may change; to share it between threads, create it shared: `names = thread_shared Names()`")"; OWN-13: "`thread_shared x`, one more owner on any thread, written only before a new value (`names = thread_shared Names()`) or a value already `thread_shared` (`E7017` otherwise"; PHIL-2: "a value made shared (`shares x`, `shares mut x`, `thread_shared x`): the block is made where the value is made, so for a local shared on a later line the allocation is the local's own line, which `ember inspect --alloc` lists with the sharing line beside it (`[RC-1]`)"; RC-7: "After `other = shares mut items` (or `other = shares items`), `items` may still change the list if it could before, and so may `other`"

**program**: 

**remaining_doubt**: J5-06 is decided as its recommendation B (N57, Hardened_1): `g = ref inv.gold` survives `shop = shares mut inv`, `inv` is in its block from its declaring line on every path, and `inv.gold += 1` is allowed by [RC-7]. J4-05 is decided as its recommendation A: `shared = thread_shared names` is E7017 in [RC-1], [THR-1] and [OWN-13] alike, with [RC-1] giving the reason (the header and the making thread's record are written where the value is made), and E2122's row now names only `shares`/`shares mut` of a one-owner place. One loose end for K24: [RC-1] says "a local of this function that owns its value", and whether a loop or pattern name handed its value (`for x in owns xs:`) is such a local is the K24 question, not this one.

**REFUTER reason**: [RC-1] decides both members. J5-06: "a local of this function that owns its value (which becomes one of the owners: a value that is not a class object is then made in a block of its own from the line that declares the local (`xs = make()` puts the result in that block), as if that line had said `shares`, so nothing moves and no view of it ends at the `shares` line, and the local's type is the shared type throughout the function)", so `g = ref inv.gold` survives `shop = shares mut inv`, the one-path `other = shares inv` changes nothing (one type throughout), and `inv.gold += 1` is [RC-7]'s "`items` may still change the list if it could before, and so may `other`". J4-05: "`thread_shared x` is written only before a new value or a name that is already a `thread_shared` owner (`E7017`, `[OWN-13]`, `[THR-1]`), since its header and its making thread's record are written where the value is made", the same in [THR-1] and [OWN-13], and the E2122 row names only `shares`, `shares mut` or `thread_shared` "of a place that has one owner and is not a local", which no longer contradicts it. Safe: the block exists from the declaring line, so no view dangles; a 32-byte header is never retrofitted.

## K11-unsafe-drop-override  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: grammar fn_header: "fn_header     := ["extern" STRING] ["unsafe"] ["static" | "virtual" | "override"] ["gen"]"; UNS-12: "**An `unsafe fn` is unsafe as a whole** (verdict 118: "the function literally says that it is unsafe"). Its body is unsafe code, as the inside of an `unsafe:` block is."; UNS-1: "An `unsafe` context is required to: dereference, read or write through `*T`/`*mut T`; offset a pointer; cast between pointers or pointers and integers; call an `unsafe fn`"; DRP-8: "Anything that may end a value whose type has a `drop` (a list cleared, a value replaced, a local ending with its function, a function value or an `owns dyn I` value replaced or ended with what its state owns, `[CLO-10]`) makes the changes that `drop` makes, wherever a value of its type may end"; CLS-7a: "A `drop`'s first line is `fn drop():`; it changes its object as a given (`[DRP-1]`)."; CLS-4: "Methods are non-virtual unless `virtual`; replacing one requires `override`; overriding a non-virtual method is `E2110`."; FN-6: "an `unsafe fn` coerces only to an `unsafe` callable type (`E2020` otherwise)"; PHIL-10: "A program containing no `unsafe` boundary (`[TIER-1]`) and no false foreign contract (`[FFI-1]`) cannot perform an invalid ownership operation, a use-after-free, a double release"

**program**: # SAFETY: these declarations match window.h
unsafe extern "C":
    safe fn create_window() -> *mut u8
    @safety("`w` came from `create_window` and is destroyed once")
    unsafe fn destroy_window(w: *mut u8)

class Window:
    handle: *mut u8

    @safety("`this.handle` came from `create_window` and is destroyed only here")
    unsafe fn drop():
        destroy_window(this.handle)

open class Widget:
    virtual fn show():
        pass

class Raw(Widget):
    p: *mut u8

    @safety("`this.p` points to a writable byte")
    override unsafe fn show():
        p.write(1)

fn main():
    w = Window(handle=create_window())
    println("open")
    ws: Array[owns Widget] = [owns Raw(p=create_window())]
    ws[0].show()        # a safe virtual call reaches an `unsafe fn` body: nobody vouches
    # `w` ends here, in safe code: its `unsafe fn drop()` runs with no `unsafe:` block and no `# SAFETY:` note

The grammar accepts `unsafe` before `fn drop()` and before `override fn`; [UNS-12] makes each body unsafe code with its own `@safety`; [UNS-1] asks unsafe code of a caller that names the function, and the end of `w` and the virtual call name no function. [CLS-7a]'s "A `drop`'s first line is `fn drop():`" gives no code for `unsafe fn drop():`, and no rule matches an override's or an implementation's `unsafe` against the method it replaces. So the program has no refusal, and unsafe code runs from a safe program with no `unsafe` boundary, against [PHIL-10]'s premise.

- READER EDIT rule=UNS-12
  - old: 
  - new: A function that safe code runs without naming it is never an `unsafe fn`: `drop`, which runs wherever a value ends (`[DRP-1]`, `[DRP-8]`); an `override` of a method that is not `unsafe`, which a virtual call through the base reaches (`[DSP-2]`); and a method implementing an interface method the interface declares without `unsafe`, which a call through `dyn I` or through a bound reaches. Each written `unsafe fn` is `E2020`, an `unsafe fn` where a safe one is wanted (`[FN-6]`): "`drop` runs wherever a `Window` ends, in safe code, so it cannot be an `unsafe fn`; write `fn drop():` and put `destroy_window(this.handle)` in an `unsafe:` block with a `# SAFETY:` note"; "`show` overrides `Widget.show`, which safe code calls through any `Widget`; write `override fn show():` and put `p.write(1)` in an `unsafe:` block with a `# SAFETY:` note". A `virtual` method or an interface method declared `unsafe fn` is called in unsafe code through the base or the interface as any `unsafe fn` is (`[UNS-1]`), with its `@safety` (`[UNS-7]`); a safe override or implementation of one is allowed, as a safe function converts to an `unsafe fn(…)` type (`[FN-6]`).
  - why: The property: whoever vouches must be able to see the call. A `drop`, an override and an implementation are reached by code that names only the base's or the interface's declaration, so the vouching belongs inside them, in an `unsafe:` block with its note, as Rust keeps `Drop::drop` safe. The refusal sits where the mismatch is written, with [FN-6]'s existing code for an `unsafe fn` where a safe one is wanted, so no new code.

- READER EDIT rule=registry row E2020
  - old: 
  - new: an `unsafe fn` written as `drop`, as an `override` of a method that is not `unsafe`, or as the implementation of an interface method the interface declares without `unsafe` (`[UNS-12]`)
  - why: The registry names every case of a code with its rule ([DIA-6a]).

**remaining_doubt**: Verdicts 118 and 119 say what an `unsafe fn` is, not where one may be declared, so the edit extends them without touching them. Whether the right code is E2020 (the mismatch, [FN-6]) or E3100 (the unsafe call made by the value's end, [UNS-1]) is a choice of defining rule; E2020 is used because the refusal is at a declaration, not at an operation. The critic's note asks for a rule that an implementation's first line matches the interface's declaration; the text has E2110/E2111 for overrides and nothing explicit for an implementation's words, and this edit covers only the `unsafe` word, which is the safety part.

**REFUTER reason**: The grammar accepts both (`fn_header := ["extern" STRING] ["unsafe"] ["static" | "virtual" | "override"] ["gen"]`). [UNS-12] makes the body unsafe code ("Its body is unsafe code, as the inside of an `unsafe:` block is"), and [UNS-11] says who must vouch: "before a declaration (`unsafe fn`, ...) it says that whoever uses it must vouch, with `@safety` saying what they must make true". The user of a `drop` is the value's end, which [DRP-1] places ("runs exactly once per value, at the end of its life") and no line names; [UNS-1]'s `E3100` is for "call an `unsafe fn`" outside unsafe code, and no rule says whether the end of `w` in safe code is such a call. [CLS-7a]'s "A `drop`'s first line is `fn drop():`" gives no code for `unsafe fn drop():`. For the override, [CLS-4] checks only `virtual`/`override` (`E2110`, `E2111`) and [DSP-2] sends the safe call `ws[0].show()` to the unsafe body with no one vouching. So the text accepts a program in which `@safety`'s obligation is made true by nobody, which is what [UNS-11]'s "possible only inside an `unsafe` block" promises against. The reader's edit is at the cause (where the mismatch is declared), refuses only the three forms safe code reaches without naming them, keeps an `unsafe` virtual or interface method callable as any `unsafe fn` is, and adds no code; `E2020` is [FN-6]'s "an `unsafe fn` where a safe one is wanted", a fair fit, and which code is the lead's (JOB.md: messages and codes are the lead's). No verdict is touched: 118 and 119 say what an `unsafe fn` is, not where one may be declared. It also covers `Display.fmt` run by `println` and `Add.add` run by `+`, which are implementations of interface methods declared without `unsafe`.

## K17-read-only-parameter-promise  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: WK-11: "That count is a number in the target's header that no name reads, so making or ending a link is not a change of the target: any name may make one (`up = weakref p` from a read-only `p`; `child.parent = weakref this` in a method that otherwise changes only `this.children`)."; GRM-44: "plain `p` is a read-only view and never changes (`E3023`); `if ui exists as ref mut me:` may change what it names"; BRW-21: "Writing a number or a `bool` ends and moves nothing, so a method that writes `this.health` through a link that may lead to its own object (a turret aimed at itself) is allowed"; BRW-21: "Since a `ref mut` parameter is compiled with noalias (`[BRW-22]`), the check is wider for it: while it is used, the callee reaches its place through no other name, so an argument for it that points into a place the callee reads or changes by another name while the parameter is used is refused too"; EXC-15: "That a method may change its object promises nothing about other names: the same object may be reached and changed through a link while the method runs (a turret aimed at itself: `if this.target exists as ref mut t: t.health -= 1`, accepted with no check), so `this` is never the only name of its object and is never compiled with noalias (`[BRW-22]`)."; BCK-3: "Two places **may be one** when one is a prefix or an extension of the other (distinct fields, distinct `SoA` columns and the halves of a split are not, `[BRW-4]`); when they are elements of one list whose indexes are not proved different (different constants, `j` from `i + 1..`, under `if i != j:`); or when they lead to values of one type and at least one path goes through a link or a kept view, unless the compiler knows they differ"; BRW-22: "Every `ref mut` parameter, a `MutSpan` one included, is compiled with noalias (C's `restrict`): while it is used, nothing else the function uses reaches its place (`[BRW-1]`, `[BRW-21]`)"; decisions.md §(area 2 Q4), the lead's ruling: "across a call, only "not ended or moved" is checked (BRW-21); a change through a link is allowed, and the compiler never assumes the value unchanged (verdict 18, the turret)"

**program**: 

- READER EDIT rule=FN-1
  - old: and what `a` reads does not
    change while `a` is used (`[BRW-1]`).
  - new: and what `a` reads is not ended or moved while `a` is used (`[BRW-1]`, `[BRW-21]`); a number another name of the same place writes across the call (a module variable, a name opened `ref mut` from a link, `[WK-11]`) shows through `a`, since the compiler never assumes a value unchanged where it cannot prove the names apart (`[EXC-15]`).
  - why: The bullet promises more than [BRW-21] checks and than the lead's ruling decides; an implementer reading [FN-1] and [BRW-21] cannot tell which holds. The words state the ruling where the promise is made.

- READER EDIT rule=BRW-21
  - old: inside it, the parameter makes the promise of `[BRW-1]`,
  and the compiler holds the callee to it.
  - new: inside it, the parameter makes the promise of `[BRW-1]`, and the compiler holds the callee to it: for a plain or `Span` parameter, that what it views is not ended or moved; for a `ref mut` parameter, that its place is reached by no other name (below).
  - why: The sentence says the whole promise is held and the rest of the rule checks only ending and moving for a plain parameter; the one sentence that names both scopes removes the contradiction J4-14 found.

**remaining_doubt**: J5-07: the text, read as written, accepts the program (`inspect(boss)` prints 0), which the lead ruled in area 2 Q4 ("a change through a link is allowed") and which the finder, the checker and the recommendation (A: E3023, the link follows the name's rights) did not expect. It is safe: every view the caller holds is checked against the callee's summary by the type rule of [BCK-3] and [BRW-17] (same class through a link may be one place), a plain parameter has no noalias ([BRW-22]), and inside the callee a use of `e` after `x.hp = 0` is E3021 by the same rule. It runs against the spirit of verdicts 1 and 32 (a call that changes an argument says so), which name owners and the call word, not links; A would need either a second link type that says "reads" (a new word, the owner's) or the link's provenance tracked through fields across packages, which [BLD-16] forbids, so if the owner wants A it is an owner question, not a text edit. J4-14 (`report(PARTY[0])` with `PARTY[0].hp = 5` inside): decided the same way by [BRW-21] (end or move only for a plain parameter) and the ruling; prints 5; the edits above make [FN-1] say so.

**REFUTER reason**: The text decides both programs as the lead ruled (decisions.md §4, area 2 Q4: "across a call, only 'not ended or moved' is checked (BRW-21); a change through a link is allowed, and the compiler never assumes the value unchanged (verdict 18, the turret)"). J5-07: [WK-11] "any name may make one (`up = weakref p` from a read-only `p` ...)", [GRM-44] "`if ui exists as ref mut me:` may change what it names"; inside `inspect`, `e` is not used after `w = weakref e` (a link is not a view, so [BRW-2]'s liveness does not extend `e`), and [BCK-3]'s "when they lead to values of one type and at least one path goes through a link" would give `E3021` only while `e` is live; the caller holds no view of `boss`. So it compiles and prints 0. J4-14: [BRW-21] "Writing a number or a `bool` ends and moves nothing" and "Since a `ref mut` parameter is compiled with noalias (`[BRW-22]`), the check is wider for it", so for a plain `p` the call check is end-or-move only, and [BRW-17] defers the parameter/module-variable case to the call ("A view parameter (a plain, `Span` or `ref mut` one), or `this`, and what its function may end or move may be one place too; that is checked at each call"); prints 5. The promise [FN-1] states, "what `a` reads does not change while `a` is used (`[BRW-1]`)", is therefore wider than what is checked, and the reader's two edits write the ruling where the promise is made. Safe: a scalar written through another name dangles nothing, every view the caller holds is checked by the type rule of [BCK-3] and [BRW-17] against the callee's summary, and a plain parameter has no noalias. The edits narrow verdict 5 ("A read-only name is a promise that what it reads does not change while it is in use") across a call only, which is the lead's standing ruling, so no owner verdict is overridden; the lead should still mark it in the change log as the ruling's consequence.

## K23-match-subject-temporaries  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: EXP-4: "A temporary created while evaluating the **condition** of an `if`, `elif` or `while` is dropped before the chosen block runs, unless the condition names what it holds (`exists as`, a kind test, a pattern): the temporary the name refers to, and every temporary of the condition that the name's value views, directly or through what it holds (`[BRW-19]`: `if names().first() exists as p:` keeps the list `names()` made while `p` views a string in it), then live to the end of that block only, never into an `elif` or `else`; the rest are dropped before the block runs. A temporary bound by `with` lives to the end of the `with` block; one created by a `for` iterable lives to the end of the loop."; DRP-3: "Temporaries drop at the end of the statement that created them (`[EXP-4]`)."; GRM-15: "before the iterable of a `for` or of a comprehension's `for` clause (`[GRM-27]`) and before the scrutinee of a `match`, where the default is a view, so that the loop or the branches take the value over (`for x in owns xs:`, `match owns e:`)"; THR-19: "While this thread holds a lock, inside its `with` block, or from `g = m.lock()`, `g = rw.read()` or `g = rw.write()` to the last use of `g` or of a value holding it (`[BRW-2]`), these are `E7018`, with no loop needed and nothing checked while the program runs: a call through a function value, or through a `dyn` or `virtual` method, that may run a function taking a lock of the kind this thread holds"; registry row E7018: "a wait for a lock this thread holds, while this thread holds that lock (its `with` block, or a guard in a local until its last use)"

**program**: from std.sync import Mutex

static JOBS: Mutex[Array[Job]] = Mutex([])

class Job:
    id: int

    fn run():
        JOBS.lock().push(owns Job(id=this.id + 1))   # follow-up work: takes JOBS's lock again

fn run_next():
    match owns JOBS.lock().pop():        # the guard is a temporary of the subject
        Some(job):
            job.run()                    # under [DRP-3] the guard lives to the end of the match: the lock is held here
        None:
            pass

fn main():
    JOBS.lock().push(owns Job(id=1))
    run_next()
    println("done")

[EXP-4] names no `match` subject, so [DRP-3] decides: the guard lives to the end of the `match` statement, through `job.run()`. [THR-19]'s holding forms are a `with` block and a guard in a local (`g = m.lock()`), and the E7018 row says the same; a temporary guard is neither, so nothing refuses `job.run()`, and `JOBS.lock()` inside it waits for the lock this thread holds for ever. [THR-19]'s title promises "A wait for a lock this thread holds is refused when compiling" (verdict 43), and this program breaks it. The same hole is `JOBS.lock().pop().run()` on one line, whose guard lives to the end of the statement.

- READER EDIT rule=EXP-4
  - old: A temporary created while evaluating the **condition** of an `if`, `elif`
  or `while` is dropped before the chosen block runs, unless the condition names what it holds
  (`exists as`, a kind test, a pattern):
  - new: A temporary created while evaluating the **condition** of an `if`, `elif` or `while`, or the **subject** of a `match` (`match owns JOBS.lock().pop():` holds the lock for the pop and lets the guard go before the arm runs), is dropped before the chosen block or arm runs, unless the condition or the arm's pattern names what it holds (`exists as`, a kind test, a pattern):
  - why: One rule for every header that tests a value: the subject of a `match` is a header like a condition, and a lock is held only while something still looks into what it guards; `match owns e:` hands the value to the arm's names, so nothing of the guard is viewed and it ends; `match JOBS.lock().first():` keeps the guard to the end of the arm whose name views its contents, by the same words.

- READER EDIT rule=EXP-4
  - old: then
  live to the end of that block only, never into an `elif` or `else`; the rest are dropped before
  the block runs.
  - new: then live to the end of that block or arm only, never into an `elif`, an `else` or another arm; the rest are dropped before the block or arm runs.
  - why: The arm is the block of a `match`; the words name it.

- READER EDIT rule=THR-19
  - old: While this thread holds a lock, inside its `with` block, or from `g =
  m.lock()`, `g = rw.read()` or `g = rw.write()` to the last use of `g` or of a value holding it
  (`[BRW-2]`), these are `E7018`,
  - new: While this thread holds a lock, inside its `with` block, from `g = m.lock()`, `g = rw.read()` or `g = rw.write()` to the last use of `g` or of a value holding it (`[BRW-2]`), or for as long as a guard made as a temporary lives (`[EXP-4]`: to the end of its statement, or of the block or arm whose name views what it guards), these are `E7018`,
  - why: The property is "while this thread holds a lock"; a guard that is a temporary holds it too, for the life [EXP-4] gives it, and the forms listed were read as the whole list. Checked when compiling from the temporary's life, which the compiler already tracks ([BRW-19]); nothing runs.

- READER EDIT rule=registry row E7018
  - old: while this thread holds that lock (its `with` block, or a guard in a local until its last use)
  - new: while this thread holds that lock (its `with` block, a guard in a local until its last use, or a guard made as a temporary for its life, `[EXP-4]`)
  - why: The row lists the holding forms.

**remaining_doubt**: The finder's two options are both safe only once [THR-19] covers a temporary guard; without that edit the current text (option B by [DRP-3]) lets the deadlock compile, which is why the status is not_answered rather than answered_differently. The choice of A (the subject's temporaries end before the arm, as a condition's do) over B (held through the match, the program refused E7018 and rewritten on two lines) is consistency and the accepted program, not safety; if the lead keeps B, only the [THR-19] and E7018 edits are needed, plus one sentence in [EXP-4] saying a `match` subject's temporaries live to the end of the statement.

**REFUTER reason**: [EXP-4] names the lives of temporaries of "an expression statement", of "the **condition** of an `if`, `elif` or `while`", of `with` and of a `for` iterable, and never a `match` subject; the grammar makes `match_stmt` a `compound_stmt`, not an expression statement, so [DRP-3] decides ("Temporaries drop at the end of the statement that created them"): the guard lives through `job.run()`. [THR-19] then refuses only while "this thread holds a lock, inside its `with` block, or from `g = m.lock()`, `g = rw.read()` or `g = rw.write()` to the last use of `g` or of a value holding it", and the E7018 row repeats the two forms ("its `with` block, or a guard in a local until its last use"); a guard that is a temporary is neither, so `job.run()`, "a direct call that may take the held lock and is not proved to enter past it", is not refused and the program waits for ever, against the rule's title "A wait for a lock this thread holds is refused when compiling" (verdict 43). Deadlock is permitted by [PHIL-11] in general, but this one is the kind verdict 43 promises to refuse, so the text is not answered. The reader's edits are safe and at the cause: [THR-19] gains the missing holding form (the compiler already tracks a temporary's life, [BRW-19]), and [EXP-4] gives the `match` subject the condition rule, under which `match owns JOBS.lock().pop():` lets the guard go before the arm (the arm's name owns the popped value and views nothing of the guard) while `match JOBS.lock().first():` keeps it to the end of the arm whose name views it. Nothing runs. Note for the lead: whether `owns` before a temporary subject is `E0109` is K22's question; the [EXP-4] edit holds either way.

## K24-owning-names-changeable  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: BRW-14: "A parameter, a loop name, a pattern name and a name opened with `exists as` (`[GRM-44]`) or named by a kind test (`[DSP-4]`) stand for what the caller, the loop, the opening or the test gave, and keep it for their whole scope: `=` on one is `E3023`"; BRW-15: "A plain parameter, a plain loop, pattern or opened name, and a plain local that holds a view never change what they name or what it leads to"; CTL-1: "`for x in owns e:`, or a value `e` (a temporary) whose type is `IntoIterator`: hands `e` over, and each element to the loop's name"; CTL-1: "A plain loop name changes nothing: `for v in xs.iter_mut(): v += 1` is `E3023`, whose help gives `for ref mut v in xs:`."; GRM-13: "`match owns e:` hands `e` over, and each name of the branch that runs owns its part."; FN-1: "`owns c: C`: **handed over**, for every type, numbers included. The caller's name ends, and a later use is `E3040`"; RC-1: "for an `owns` parameter, which this function owns from its first line"; VII.1 terms: "**view**: a name or value that points into a place someone else owns and may not outlive it: `ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, a plain loop or pattern name, a name opened with `exists as`"

**program**: struct Job:
    name: String
    retries: int

fn retry_all(owns jobs: Array[Job]) -> Array[Job]:
    again: Array[Job] = []
    for job in owns jobs:
        job.retries += 1          # `job` owns its element ([CTL-1]); `job` has no word, so a "plain loop name" ([BRW-15], [CTL-1]'s last sentence): E3023 or allowed?
        again.push(owns job)
    return again

fn label(owns name: String) -> String:
    if name.len() == 0:
        name = String("unnamed")  # [BRW-14]: `=` on "a parameter" is E3023, though [FN-1] and [RC-1] make `name` the callee's own
    return name

fn main():
    jobs = retry_all(owns [Job(name=String("a"), retries=0)])
    println(jobs[0].retries, label(owns String("")))

The implementer cannot answer whether `job.retries += 1` and `name = String("unnamed")` compile: [BRW-14] refuses `=` on every parameter, loop and pattern name without excepting the ones handed their value, while [FN-1], [RC-1], [CTL-1] and [GRM-13] make those names owners, and [BRW-15]'s and [CTL-1]'s "plain" is read either as "no word on the name" (then `job` is plain and E3023) or as "a view" (then `job` owns and may change). No verdict covers it.

- READER EDIT rule=BRW-14
  - old: A parameter, a loop name, a pattern
  name and a name opened with `exists as` (`[GRM-44]`) or named by a kind test (`[DSP-4]`) stand for
  what the caller, the loop, the opening or the test gave, and keep it for their whole scope:
  - new: A parameter that is not `owns`, a loop or pattern name that views its value (one under `for x in e:` or `match e:`) and a name opened with `exists as` (`[GRM-44]`) or named by a kind test (`[DSP-4]`) stand for what the caller, the loop, the opening or the test gave, and keep it for their whole scope:
  - why: The rule is for names that stand for someone else's value; the names handed their value are not among them, and the first sentence must say so where the refusal is defined.

- READER EDIT rule=BRW-14
  - old: 
  - new: A name that was handed its value owns it, as a local does, and keeps no place: an `owns` parameter (`[FN-1]`), a loop name under `for x in owns e:` (`[CTL-1]`) and a name of the branch that runs under `match owns e:` (`[GRM-13]`) may be changed and given a new value with no word (`job.retries += 1` under `for job in owns jobs:`; `name = String("unnamed")` on `owns name: String` ends the string the caller handed over, as `=` on a local does, `[OWN-3]`), and `kept = job` hands it over.
  - why: The property: an owner may change and replace its value, a view may not; it is stated once for every name that owns, so no second kind of owner exists. Checked when compiling as for a local; nothing runs.

- READER EDIT rule=BRW-15
  - old: A plain parameter, a plain
  loop, pattern or opened name, and a plain local that holds a view never change what they name or
  what it leads to
  - new: A plain parameter, a plain loop, pattern or opened name (one that views its value; a name handed its value is an owner, `[BRW-14]`), and a plain local that holds a view never change what they name or what it leads to
  - why: "Plain" is given its meaning where the rule uses it.

- READER EDIT rule=CTL-1
  - old: A plain loop name changes nothing: `for v in xs.iter_mut(): v += 1` is `E3023`, whose help gives
    `for ref mut v in xs:`.
  - new: A loop name that views its element changes nothing: `for v in xs.iter_mut(): v += 1` is `E3023`, whose help gives `for ref mut v in xs:`; a loop name under `for x in owns e:` owns its element and may be changed (`[BRW-14]`).
  - why: The sentence refused by the word "plain" what the same rule's second bullet hands over; it now says which loop names view and which own.

**remaining_doubt**: The edits take the finder's option A (an owning name is a local owner), which follows [FN-1], [RC-1], [CTL-1], [GRM-13] and the hand-on `counts.insert(owns u, owns 0)` the text already writes under `for u in owns upper:`; option B (change through, never `=`) would make an `owns` parameter the one owner that cannot be given a new value, a second kind of owner the text would then have to describe. Both are compile-time.

**REFUTER reason**: The rules pull apart. [BRW-14]: "A parameter, a loop name, a pattern name ... stand for what the caller, the loop, the opening or the test gave, and keep it for their whole scope: `=` on one is `E3023`" names every parameter, while [FN-1] makes `owns c: C` "**handed over** ... The caller's name ends" and [RC-1] speaks of "an `owns` parameter, which this function owns from its first line". [CTL-1] hands the element over ("`for x in owns e:` ... hands `e` over, and each element to the loop's name") and in the same rule says "A plain loop name changes nothing: `for v in xs.iter_mut(): v += 1` is `E3023`", where `job` under `for job in owns jobs:` carries no word and so is "plain" on one reading and an owner on the other; [BRW-15] has the same word. [GRM-13] decides the `match` case ("`match owns e:` hands `e` over, and each name of the branch that runs owns its part") but says nothing about changing the part. [OWN-3] lists "a plain parameter, a loop, pattern or opened name" as views, and the VII.1 term says "a plain parameter, a plain loop or pattern name", which reads as "no word", so an implementer cannot decide `job.retries += 1` or `name = String("unnamed")`. The reader's option A is the one the text already implies for owners (a local is changed and given a new value with no word; [OWN-3]'s `=` ends the old value) and costs nothing; it is safe because a handed-over value is reached by no other name, and a class object with links is changed through its owner anyway. The edits give "plain" its meaning where used and keep the `const ref mut` and `ref mut` sentences of [BRW-14] untouched.

## K25-owns-takes-a-place-out  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: GRM-15: "and in `b = owns n` for a value `=` would copy (a number, a `bool`, a `StrView`, `[OWN-9]`), so that `n` ends there."; GRM-15: "Before a name whose place already hands the value over with no word (`e.mesh = m`, `b = a` for a value `=` hands over, `return a`), and before `fn` anywhere but an argument for an `owns` parameter, it is `E0109`, whose message says to remove it"; BRW-12: "A name bound to a part (`n = p.name`, `boss = enemies[0]`), or to a call whose result is a view (`ref T`, `StrView`, `Span[T]`, a `ref` or `ref mut` place, `[LT-1]`), such as `word = line.to_lower().trim()` or `best = a.as_span()`, is a read-only view of it: never a hand-over, and never a silent copy"; OWN-11: "A part of a struct value stored somewhere else (`push`, `return`, a field, an `owns` argument) is handed over on its own: for a struct value `p`, `names.push(owns p.name)` moves `p`'s name into the list, and `p.hp` stays usable"; TYP-44: "`cast s as Circle` gives the `Circle`, on `owns dyn` and `shares dyn` values too, and stops the program when `s` is not one, with the text "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`", in the program's own names and line. A cast of a place is a place: the name bound from it is read-only unless written `ref mut` (`pool = ref mut cast this.pools[k] as Pool[T]`), and `ref mut` taken through a plain one (`return ref mut pool.dense[pool.sparse[e]]`) is `E3027`."; FN-2a: "`return a` and a constructor's argument take no word: they hand a value over as `=` does (`[OWN-9]`)."; GRM-43: "`copy of`, `owns`, `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut`, `ref`, `ref mut` and `const ref mut` stand directly before the value they act on and bind as a prefix `-` does, wherever they stand, in a call's argument and on the right of `=` as anywhere else"; grammar unary: "unary         := ("-" | "~") unary | value_word unary | "cast" postfix_expr "as" type | power"

**program**: struct Request:
    path: String
    body: String

fn take_body(owns req: Request) -> String:
    body = owns req.body             # [GRM-15] lists no `owns` before a part on `=`; [BRW-12] makes `body = req.body` a view; [OWN-11] lists push, return, a field, an argument, never `=`
    println(req.path)
    return body

interface Shape:
    fn area() -> float

class Circle:
    r: float

extend Circle implements Shape:
    fn area() -> float:
        return 3.0 * this.r * this.r

fn keep_circles(owns shapes: Array[owns dyn Shape]) -> Array[owns Circle]:
    circles: Array[owns Circle] = []
    for s in owns shapes:
        if s is Circle:
            circles.push(owns cast s as Circle)   # parses (`value_word unary`, `cast … as` at row 16); no rule says what it gives
    return circles

open class Unit:
    name: String

class Player(Unit):
    score: int = 0

fn promote(owns u: Unit) -> Player:
    return cast u as Player          # `return` hands over "as `=` does", and `=` from a cast of a place binds a view: the Player handed over, or a view of the local `u` (E3063)?

fn main():
    println(take_body(owns Request(path=String("/u"), body=String("hi"))))
    println(keep_circles(owns [owns Circle(r=1.0)]).len())
    p = promote(owns Player(name=String("ann")))
    println(p.name)

Three lines an implementer cannot decide: `body = owns req.body` (hand-over, or E0109, or E3028?), `owns cast s as Circle` (the same block as an `owns Circle`, a refusal, or a copy?), and `return cast u as Player` (a hand-over or a returned view of a local). Verdict 78 decided a field hand-over on `push`, verdicts 25 and 27 the cast's spelling only, verdict 148 that `return` takes no word.

- READER EDIT rule=GRM-15
  - old: so that `n`
  ends there.
  - new: so that `n` ends there, and before a part on the right of `=` (`body = owns req.body`), which hands the part over into the new name as `names.push(owns req.body)` does (`[OWN-11]`, with its refusals: a field of a type with `drop` `E3010`, an element `E3011`, a field of a class object or a part reached through a name that does not own it `E3013`), where `=` alone makes a view (`[BRW-12]`).
  - why: The property: `owns` before a value on the right of `=` hands over what `=` alone would copy or view. The number case is written; the part case is the same word with the same meaning, and the refusals are [OWN-11]'s own.

- READER EDIT rule=OWN-11
  - old: A part of a struct value stored somewhere else (`push`, `return`, a field, an `owns` argument) is handed over on its own:
  - new: A part of a struct value stored somewhere else (`push`, `return`, a field, an `owns` argument, `body = owns req.body` on the right of `=`, `[GRM-15]`) is handed over on its own:
  - why: The defining rule lists where a part is handed over; `=` with `owns` joins the list.

- READER EDIT rule=TYP-44
  - old: and `ref mut`
  taken through a plain one (`return ref mut pool.dense[pool.sparse[e]]`) is `E3027`.
  - new: and `ref mut` taken through a plain one (`return ref mut pool.dense[pool.sparse[e]]`) is `E3027`. The words before a value act on a cast as on any value (`[GRM-43]`), after the kind test, which runs first and stops the program before anything is handed over: `owns cast s as Circle` hands the whole owner `s` over as its kind, the same block and no copy (an `owns Circle` from an `owns dyn Shape`; a `Circle` object with one owner from a `Unit`), `shares cast s as Circle` makes one more owner typed `shares Circle` from a `shares dyn Shape` or a `shares Shape`, and `return cast u as Player` hands `u` over as a `Player`, as `return u` would (`[FN-2a]`); a cast of a part is a part (`[OWN-11]`), and a cast of a name that does not own its value is handed over by nobody (`E3013`). A cast of a temporary gives the value, held as the temporary was.
  - why: The words before a value already mean the same before every value ([GRM-43]); giving them their meaning before a cast adds no word, costs nothing beyond the type-id compare the cast already makes, and gives the downcast of an owner its only spelling. Decided when compiling; the only run-time step is the cast's own compare.

- READER EDIT rule=DSP-6
  - old: Each cast is one type-id compare.
  - new: Each cast is one type-id compare; what the words before a cast hand over or share is `[TYP-44]`'s.
  - why: [DSP-6] repeats [TYP-44]'s cast sentence; a pointer keeps one defining rule.

**remaining_doubt**: The edits take the three findings' option A, which is the one that gives `owns` and `shares` one meaning before every value and sends no program to `mem.take` or to a copy. The exact `old` for [DSP-6] is a single sentence that also ends [TYP-44]'s neighbour text ("Each test and each cast is one compare of type indices" in [TYP-44] differs), so it is unique to [DSP-6]. Whether `owns cast u as Player` on a class object with one owner should be E2114 like `owns` on a class type ([TYP-45]) is a wording question: the edit treats `owns` before a value as the hand-over word ([OWN-9]), not as the type word.

**REFUTER reason**: `body = owns req.body`: [BRW-12] makes `body = req.body` "a read-only view of it: never a hand-over, and never a silent copy" and lists the words that say otherwise ("`ref mut` ..., `copy of` ..., the link words ..., and `ref`") with no `owns`; [GRM-15] lists where `owns` is written and has only "in `b = owns n` for a value `=` would copy (a number, a `bool`, a `StrView`, `[OWN-9]`)"; its `E0109` is for "a name whose place already hands the value over with no word", which a part on `=` is not; [OWN-11] hands a part over only "somewhere else (`push`, `return`, a field, an `owns` argument)". Three readings (hand-over, `E0109`, `E3028`) and no rule picks one. `owns cast s as Circle`: [GRM-43] binds the words "wherever they stand" and the grammar (`unary := ... value_word unary | "cast" postfix_expr "as" type`) parses it, but [TYP-44] gives a cast only as a place ("A cast of a place is a place: the name bound from it is read-only unless written `ref mut`"). `return cast u as Player`: [FN-2a] "`return a` ... hand a value over as `=` does" against a cast that `=` binds as a view. Not answered. The reader's edits are at the cause and safe: the kind test runs first and stops the program before anything is handed over; `owns dyn I` is "pointer + table" (§IV.9 table), so the data pointer handed over as an `owns Circle` is the same block with no copy; the part case inherits [OWN-11]'s own refusals (`E3010`, `E3011`, `E3013`). Verdicts 78 (a field handed over on its own) and 148 (`return` takes no word) are kept; 25 and 27 decided the spelling only. No cost while running beyond the cast's compare.

## K26-copying-a-view  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_proposed_text

**answered_by**: BRW-12: "is a read-only view of it: never a hand-over, and never a silent copy, except that a part with no fields of its own (a number, a `bool`, a variant with no payload) is copied, as Python binds a number."; OWN-9: "A value `=` copies (a number, a `bool`, a `char`, a `StrView`, a `ref T`, a `Span[T]`, a struct made only of these) is copied with `=` too, so `b = owns n` is how `=` hands one over"; OWN-7: "A value of a `Copy` type is duplicated bit for bit by `=` and on use, and the source stays valid; the duplicate is independent, and no copy runs any code."; VII.1 terms: "**`Copy` type**: a type whose `=` makes such a copy bit for bit: the numbers, `bool`, `ref T`, `Span[T]`, `StrView`, and structs made only of these."; OWN-8: "`copy of` is `E2116` before what it cannot copy as itself: ... a value of a view type (`StrView`, `Span[T]`, a struct holding a `ref mut` field): "`w` is a view of `line`'s text; to make your own copy write `String(w)`""; OWN-8: "Every `Copy` type has `Clone` (`@derive(Copy)` gives it), so `b = copy of a` works for a `Point` marked only `Copy`."; TYP-36: "`copy of x` makes a value with `Clone` (`[OWN-8]`), and every `Copy` type is `Clone`; before what it cannot copy as itself (a `weakref` link, a value of a view type, a type with no `Clone`) it is `E2116`."; TYP-36 table: "| `StrView` | ✓ | ✓ | ✓ | by bytes | ✓ | `""` | ✓ | ✓ |"; STR-3: "it gives `Clone` too, so `copy of` works on every `Copy` struct (`[OWN-8]`)"; §IV.8 marker table, Copy row: "always for scalars, `ref T`, `Span`, `StrView`, raw pointers, `extern fn`, tuples and fixed arrays of `Copy` elements"

**program**: struct Lexer:
    rest: StrView

    fn next_word() -> Option[StrView]:
        w = this.rest                  # [BRW-12]: a part is a view unless it has no fields of its own (a StrView is not listed); [OWN-7]/[OWN-9]/VII.1: a StrView is copied by `=`
        if w.find(" ") exists as i:
            this.rest = w[i + 1..]     # E3021 if `w` views the field; allowed if `w` is a copy of the view
            return Some(w[..i])
        this.rest = ""
        return Some(w)

@derive(Copy)
struct Token:
    text: StrView
    line: int

fn main():
    src = String("let x = 1")
    lx = Lexer(rest=src.as_str())
    println(lx.next_word())
    tok = Token(text=src.as_str(), line=1)
    t2 = copy of tok                   # [STR-3]: works on every Copy struct; [OWN-8]: a struct holding a view?
    w = src.as_str()
    w2 = copy of w                     # [TYP-36] table: StrView is Clone and Copy, and [STR-3]/verdict 68 say every Copy type works; [OWN-8] and [TYP-36]'s sentence: E2116
    println(t2.line, w2)

Two lines an implementer cannot decide: `w = this.rest` (a view of the field by [BRW-12], a copy by [OWN-7], [OWN-9] and VII.1) and `copy of w` (E2116 by [OWN-8] and [TYP-36]'s sentence, allowed by the same rule's table, by [STR-3] and by verdict 68).

- READER EDIT rule=BRW-12
  - old: except that a part with no fields of its own (a number, a `bool`, a variant
  with no payload) is copied, as Python binds a number.
  - new: except that a part of a `Copy` type (a number, a `bool`, a `char`, a variant with no payload, a `StrView`, a `Span[T]`, a `ref T`, a `Copy` struct) is copied, as `=` copies a whole name of that type (`[OWN-7]`, `[OWN-9]`): `w = this.rest` for `rest: StrView` is a `StrView` of its own that looks at what the field looks at, so `this.rest = w[i + 1..]` while `w` is used changes nothing `w` reads, and a copied `Copy` struct is not then changed through the new name while the source is used (`[OWN-7]`, `E3023`).
  - why: One meaning of `=` for a `Copy` value, whole name or part: the words of [OWN-7], [OWN-9] and VII.1 already say `=` copies these types, and copying a view copies no data, so it is no silent copy; the lexer then reads as in C++ and Rust, and the compiler's own conformance test (LT-22, `rest = this.rest`) already accepts it.

- READER EDIT rule=OWN-8
  - old:   * a value of a view type (`StrView`, `Span[T]`, a struct holding a `ref mut` field): "`w` is a view of
    `line`'s text; to make your own copy write `String(w)`";
  - new:   * a `ref mut` view, a `MutSpan`, or a struct holding one, since no second changing view of a place can be made: "`d` is the one name that writes `xs`'s elements; a copy of your own is `copy of xs`". Before a bare read-only view (`StrView`, `Span[T]`, `ref T`) `copy of` copies what it views, as before a link it copies the target: a `String` from a `StrView` (what `String(w)` makes), an `Array[T]` from a `Span[T]`, a `T` from a `ref T`, one allocation and one copy where the viewed value owns memory (`[PHIL-2]`), so `copy of w` is text of one's own, untouched by a later change of `src` (verdict 93: `copy of t` on a loop's view of a text gives the task its own copy). A view inside a struct is copied as one more view of the same place, as a link inside a struct is copied as a link to the same target: `t2 = copy of tok` for a `Token` holding `text: StrView` is a `Token` whose `text` looks at the same text;
  - why: Verdict 68 says a `Copy` type always works with `copy of` and E2116 is never given for it; `StrView`, `Span[T]` and `ref T` are `Copy` by the marker table and `Clone` by [TYP-36]'s table, so the E2116 bullet departs from the verdict. The edit gives `copy of` before a view the meaning it has before a link (the thing itself, [OWN-8]'s own words for links, N68), and keeps E2116 only where no copy can mean anything (a second changing view).

- READER EDIT rule=TYP-36
  - old: before what it cannot copy as itself (a `weakref` link, a value of a view
  type, a type with no `Clone`) it is `E2116`.
  - new: before what it cannot copy as itself (a `weakref` link, a `ref mut` view or a `MutSpan`, a type with no `Clone`) it is `E2116`; before a read-only view (`StrView`, `Span[T]`, `ref T`) it copies what the view looks at (`[OWN-8]`).
  - why: The sentence must agree with the table beside it, which marks `StrView` `Clone` and `Copy`.

- READER EDIT rule=registry row E2116
  - old: a value of a view type (a struct holding a `ref mut` field included)
  - new: a `ref mut` view, a `MutSpan`, or a struct holding one
  - why: The row lists the cases of the code.

- READER EDIT rule=PHIL-2
  - old: `copy of x` of a value that owns memory (`[OWN-8]`);
  - new: `copy of x` of a value that owns memory, or of a read-only view of one (`[OWN-8]`);
  - why: The allocating forms are listed in full; `copy of` before a view of a `String` allocates the copy.

**cost_row**: the §X.4 / [PHIL-2] `copy of` row: `copy of` before a read-only view of a value that owns memory is one allocation and one copy of what it views, the same cost as `String(w)`, which it replaces; `copy of` before a view of a `Copy` value, and `=` on a view part, copy a pointer and a length and allocate nothing

**departs_from_verdict**: 68: "A `Copy` type always works with `copy of` ... E2116 is not given for it"; the current [OWN-8] and [TYP-36] give E2116 for `StrView` and `Span[T]`, which the marker table and [TYP-36]'s own table make `Copy`; the edits bring the text to the verdict

**remaining_doubt**: For J2-01 the edits take option A (copy what the view looks at), which keeps verdict 10 (the copy word copies the thing itself, never a link), verdict 79 (a link or view inside a struct is copied as one more), verdict 93 (a copy of a loop's text is text of one's own) and verdict 68 at once; option B (a bit copy, another view) also meets verdict 68 and is what the type's `Clone` (`fn clone() -> Self`) returns, but it makes `copy of w` a second view that a later `src.clear()` ends (E3060), which is not what the word says. If the lead prefers B, [OWN-8]'s sentence "by its type's `Clone`" stands and only the E2116 bullet, [TYP-36]'s sentence and the registry row change. For J4-08 the alternative (a view of the field for `Copy` structs with fields, a copy only for field-less types and views) would keep [BRW-12]'s list closer to today's but leave `q = p.pos` a view while `q = pos` copies, two meanings of `=` for one type.

**REFUTER reason**: Both members are undecided today. J4-08: [BRW-12] copies only "a part with no fields of its own (a number, a `bool`, a variant with no payload)" and makes every other part a view, while [OWN-9] says "A value `=` copies (a number, a `bool`, a `char`, a `StrView`, a `ref T`, a `Span[T]`, a struct made only of these) is copied with `=` too" and the VII.1 term makes `StrView` a `Copy` type; whether `w = this.rest` views the field (then `this.rest = w[i + 1..]` is `E3021`) or copies the view is not said; the reader's [BRW-12] edit is right and safe (the copy carries the field's region, [LT-20]). J2-01: [OWN-8] and [TYP-36] give `E2116` for "a value of a view type (`StrView`, `Span[T]`, ...)" while [TYP-36]'s table marks `StrView` `Clone ✓ Copy ✓`, the marker table says `Copy` "always for scalars, `ref T`, `Span`, `StrView`", [STR-3] says "`copy of` works on every `Copy` struct" and verdict 68 "A `Copy` type always works with `copy of` ... E2116 is not given for it", so `copy of tok` on a `@derive(Copy)` `Token` holding a `StrView` is refused by one rule and allowed by another. But the reader's option A opens a hole: [OWN-8] defines `copy of x` as "a copy of the thing itself, by its type's `Clone` (`fn clone() -> Self`, the explicit deep copy)", and [TYP-36] says "`copy of x` makes a value with `Clone`"; under A, `copy of w` for `w: StrView` is a `String`, so in `[T: needs Clone]` code `return copy of x` with `T = StrView` returns a `String` where a `T` is declared, which no monomorphised body can type, and a `Token`'s `clone()` could not be field-wise. A also adds an allocation to a word that [PHIL-3] says copies bits for a `Copy` value ("A copy of a `Copy` value is its bits ... no copy runs any other code"), the row the reader had to add. Option B is the one every rule already states: a read-only view is `Copy`, its `Clone` returns `Self`, so `copy of w` is one more view of the same text (as `w2 = w` is) and text of one's own stays `String(w)`; `E2116` then keeps only what has no `Clone` (`ref mut T`, `MutSpan[T]`, a struct holding one), which are not `Copy`, so verdict 68 is met and verdicts 10, 79 and 93 untouched (93's `copy of t` copies the `String` the loop name `t` names, by [OWN-13]'s "a word acts on the object the value names", not a `StrView`). No cost row is needed.

- BETTER EDIT rule=BRW-12
  - old: except that a part with no fields of its own (a number, a `bool`, a variant
  with no payload) is copied, as Python binds a number.
  - new: except that a part of a `Copy` type (a number, a `bool`, a `char`, a variant with no payload, a `StrView`, a `Span[T]`, a `ref T`, a `Copy` struct) is copied, as `=` copies a whole name of that type (`[OWN-7]`, `[OWN-9]`): `w = this.rest` for `rest: StrView` is a `StrView` of its own that looks at what the field looks at, so `this.rest = w[i + 1..]` while `w` is used changes nothing `w` reads, and a copied `Copy` struct is not then changed through the new name while the source is used (`[OWN-7]`, `E3023`).
  - why: The reader's J4-08 edit, kept: one meaning of `=` for a `Copy` value, whole name or part.

- BETTER EDIT rule=OWN-8
  - old:   * a value of a view type (`StrView`, `Span[T]`, a struct holding a `ref mut` field): "`w` is a view of
    `line`'s text; to make your own copy write `String(w)`";
  - new:   * a `ref mut T` view, a `MutSpan[T]`, or a struct holding one, which have no `Clone` (`[TYP-36]`), since no second changing view of a place can be made: "`d` is the one name that writes `xs`'s elements, and a copy of it would be a second; for a copy of the elements write `copy of xs`"; a read-only view (`StrView`, `Span[T]`, `ref T`), and a `Copy` struct holding one, is `Copy` (verdict 68), so `copy of w` is one more view of the same text, as `w2 = w` is, and text of one's own is `String(w)`;
  - why: Keeps `copy of x` as `fn clone() -> Self` for every type, so generic `Clone` code types, and gives `E2116` only where the type has no `Clone`.

- BETTER EDIT rule=TYP-36
  - old: before what it cannot copy as itself (a `weakref` link, a value of a view
  type, a type with no `Clone`) it is `E2116`.
  - new: before what it cannot copy as itself (a `weakref` link, a `ref mut T` view, a `MutSpan[T]`, a struct holding one, a type with no `Clone`) it is `E2116`; a read-only view is `Copy`, so `copy of` before one gives one more view of the same place (`[OWN-8]`).
  - why: The sentence agrees with the table beside it, which marks `StrView` `Clone` and `Copy`.

- BETTER EDIT rule=registry row E2116
  - old: a value of a view type (a struct holding a `ref mut` field included)
  - new: a `ref mut T` view, a `MutSpan[T]`, or a struct holding one
  - why: The row lists the cases of the code.

## K35-weakref-dyn  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: WK-16: "A `weakref` of a value held `dyn I`, `owns dyn I`, `shares dyn I` or `some I`, and the type `weakref dyn I`, are `E2119` too: the kind of the object is known only while the program runs, and whether its header holds a weak count is its class's to fix (`[OBJ-1]`): "`weakref dyn Listener` (line 5) names a kind known only while the program runs, and a `weakref` link needs a class that fixes its header; link to a base class: write `open class Listener:` with `virtual fn notify(msg: StrView): pass` in place of lines 1 and 2, `class Button(Listener):` in place of line 8, and `listeners: Array[weakref Listener] = []` in place of line 5". A link to the base class is one pointer, opened with one header test, and every object of the family carries the count."; OBJ-1: "a `pub(package)` type, and a private type its package never links to, carry none, and no `weakref` link anywhere can name a value of one: no line outside the package holds such a value by its type (an object held `dyn I` or `some I` is never linked, `E2119`, `[WK-16]`), and no line inside links to one."; registry row E2119: "and of a value whose kind is known only while the program runs (`dyn I`, `owns dyn I`, `shares dyn I`, `some I`; the type `weakref dyn I` too), whose header the link cannot know"; BLD-16 (via N76): "No fact of the whole program decides a package's code or layout"

**program**: 

- READER EDIT rule=DSP-4
  - old: A value of an interface type (`dyn I`, `owns dyn
  I`, `shares dyn I`) or of a base class is tested for its kind with `is`:
  - new: A value of an interface type (`dyn I`, `owns dyn I`, `shares dyn I`, `shares mut dyn I`) or of a base class is tested for its kind with `is`:
  - why: [TYP-44] lists `shares mut dyn I` among the interface types a kind test takes and [DSP-4] leaves it out; the critic flagged the slip, and the two lists should agree (wording only).

**remaining_doubt**: Decided as the finder's option B (refused, E2119, with the base-class rewrite in the message), not the recommended A, by N76 in Hardened_2: under [BLD-16] a package lays out its objects from its own text, and a `weakref dyn I` would make every implementer in every package carry a weak count for a link it cannot see, so A would need a whole-program fact; B is safe (no link is made to an object whose header may lack the count) and costs nothing. `sharedref mut dyn I` is not named anywhere; [TYP-45a] admits `ref` and `ref mut` before a `dyn` bound and [TYP-45] lists `owns`, `shares` and `shares mut` with `dyn`, so an implementer would refuse `sharedref mut dyn I` as a type with no rule; that is the critic's wording note, not this cluster's question, and safe.

**REFUTER reason**: [WK-16] decides it, on the refusing side: "A `weakref` of a value held `dyn I`, `owns dyn I`, `shares dyn I` or `some I`, and the type `weakref dyn I`, are `E2119` too: the kind of the object is known only while the program runs, and whether its header holds a weak count is its class's to fix (`[OBJ-1]`)", with the base-class rewrite in the message, and [OBJ-1] agrees ("an object held `dyn I` or `some I` is never linked, `E2119`, `[WK-16]`"). It is the only answer [BLD-16] allows ("No fact of the whole program decides a package's code or layout"): a link to an implementer declared in another package cannot make that package lay out a weak count. Safe and free. The [DSP-4] edit only aligns its list with [TYP-44]'s ("`dyn I`, `owns dyn I`, `shares dyn I`, `shares mut dyn I`"). `sharedref mut dyn I` is a separate small gap, listed under missed.

## K42-split-mutspan-halves  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: SPN-5: "`split_at(i)` on a `Span` returns two `Span`s; on a `MutSpan` it lends the view and returns two disjoint `MutSpan`s. `i > len` panics. (There is no separate `split_at_mut`: an `Array` is split for changing as `a.as_mut_span().split_at(i)`.)"; SPN-3: "A name or parameter that writes a `MutSpan`'s elements says `ref mut` (`d = ref mut a.as_mut_span()`, `fn blend(ref mut dst: MutSpan[float], src: Span[float]):`), and a plain one only reads them (`E3023`, `[BRW-15]`). A `MutSpan` is lent to a call with `ref mut` before the argument (`blend(ref mut d, s)`), which before a span names its elements, so it may stand before a span made in the call (`integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`)"; BRW-12: "and before a `MutSpan` a call gives it says the name may write the span's elements (`d = ref mut tracks[dst_name].as_mut_span()`)"; GRM-5: "`a, b = e` destructures a tuple, a struct or a fixed array. The right side is evaluated once, into a temporary, before any target is written (`[EXP-2]`). Each target follows `[GRM-4]` on its own: a name not in scope is declared with its part's type"; grammar target: "target        := IDENT | postfix_expr | "_" | "(" target_list ")""; FN-2a: "`ref mut` before something this function may not change (a plain parameter, a temporary, a place reached through a read-only name) is `E3027`"

**program**: fn crossfade(ref mut samples: MutSpan[float]):
    mid = samples.len() // 2
    left, right = samples.split_at(mid)          # two MutSpans in a tuple; `left` and `right` are plain names ([GRM-5], the target grammar has no word)
    for i in 0..mid:
        right[i] = (left[i] + right[i]) / 2.0    # [SPN-3]: a plain name holding a MutSpan only reads: E3023, and the help has no line to give

fn main():
    xs = [1.0, 2.0, 3.0, 4.0]
    crossfade(ref mut xs.as_mut_span())
    println(xs)

[SPN-5] promises two writable halves; [SPN-3] and [BRW-12] let `ref mut` stand only before "a `MutSpan` a call gives" and before an argument, and the destructuring grammar takes bare targets, so `left, right = ref mut samples.split_at(mid)` is `ref mut` before a tuple temporary (E3027 by [FN-2a]) and `ref mut left, ref mut right = ...` does not parse. No line writes both halves; verdict 34 says the name says `ref mut` but not where the word goes on a destructuring line.

- READER EDIT rule=SPN-3
  - old: which before a span names its elements, so it may stand before a span made in
  the call (`integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`);
  - new: which before a span names its elements, so it may stand before a span made in the call (`integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`) and before a tuple of spans a call gives, on an `=` line whose targets bind them: `left, right = ref mut samples.split_at(mid)` gives each name the right to write its half's elements, as `d = ref mut a.as_mut_span()` does for one span, and without `ref mut` the names only read (`E3023`, whose help gives that line);
  - why: The property: `ref mut` before a `MutSpan` value names its elements, whether the call gives one span or several; the word stays before the value, where every `=` line puts it ([GRM-43], verdict 34's own example), so no second place for words on an assignment is added. Checked when compiling; the halves are disjoint by [BRW-4] and [BCK-3], so both names may be used together.

- READER EDIT rule=BRW-12
  - old: and before a `MutSpan` a call gives it says the name may write
  the span's elements (`d = ref mut tracks[dst_name].as_mut_span()`);
  - new: and before a `MutSpan`, or a tuple of them, a call gives it says the names bound from it may write the spans' elements (`d = ref mut tracks[dst_name].as_mut_span()`, `left, right = ref mut xs.split_at(mid)`, `[SPN-3]`);
  - why: [BRW-12] repeats the `MutSpan` case of [SPN-3] and must say the same.

**remaining_doubt**: The edits take the finder's option A (the word before the value); option B (a word before each target, `ref mut left, ref mut right = …`) would mirror `if pair exists as (ref mut a, ref b):` ([GRM-44]) and the comma rule of [OWN-13], but it needs a grammar change to `target` and gives an `=` line a second place for words. Both are compile-time and safe. The exact `old` text of the [BRW-12] edit crosses a line break at "may write\n  the span's elements" as the file wraps it (line 3698).

**REFUTER reason**: [SPN-5] promises "on a `MutSpan` it lends the view and returns two disjoint `MutSpan`s"; [SPN-3] requires "A name or parameter that writes a `MutSpan`'s elements says `ref mut` (`d = ref mut a.as_mut_span()` ...), and a plain one only reads them (`E3023`, `[BRW-15]`)"; [BRW-12] lets `ref mut` stand "before a `MutSpan` a call gives", one span; [GRM-5]'s targets are bare (`target := IDENT | postfix_expr | "_" | "(" target_list ")"`), so `ref mut left, ref mut right = ...` does not parse and `left, right = ref mut samples.split_at(mid)` is `ref mut` before a tuple that no rule names. The one-branch `match` with `(ref mut left, ref mut right)` is plausible under [GRM-13] but the text does not say a `ref mut` pattern name on a temporary's `MutSpan` part writes its elements, and it is not the line `E3023`'s help would give. So an implementer cannot write both halves and the question stands. The reader's edit is safe (the halves are disjoint by [SPN-5] and [BRW-4], both names may be live together, checked when compiling), costs nothing, keeps the word on the value as verdict 34's own example does (`d = ref mut tracks[dst_name].as_mut_span()`), and adds no second place for words on an `=` line.

## K43-weakref-into-cached  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_proposed_text

**answered_by**: EFF-27: "A change that does not pass through the method's object cannot be cleared that way, so a `sharedref mut` link, or a `shares mut` owner, made into anything a `@cached` method of its holder reads is `E2117` at the line that makes it ("`s.items[0]` is read by `Shelf.first_name`, which is `@cached` and would not see a change made through this link; link the shelf instead, `sharedref mut s`, or remove `@cached`"); a link to the whole holder is allowed, since every change through it passes through the holder."; CLS-12: "gives it again until a change, through any name, of a field the method's body reads clears it (`[EFF-27]`). A `Copy` result is returned by value, any other as a `ref T` view into the kept value, which clearing it ends (a later use is `E3060`)."; CLS-12: "For a method that reads a value its other owners may change: E2117 ("`radius` reads `mesh`, which its other owners may change without `Mesh` seeing it, so a kept result could be stale; remove `@cached` (line 13)")."; WK-16: "A `weakref` of a part (a field, or an element stored inline), of a value held `owns`, or of a new value, whose target does not live on its own (`[WK-11]`, `[OWN-13]`), is `E2119`"; GRM-44: "`if ui exists as ref mut me:` may change what it names"; OWN-13: "a function that makes a `weakref T` link to what a parameter names, `this` included, directly or through a call, records it, and a call that gives that parameter a value reached through a `thread_shared` value is `E7015` at the call, naming the chain"

**program**: class Book:
    name: String

class Shelf:
    items: Array[Book] = []

    @cached
    fn first_name() -> String:
        return copy of this.items[0].name

fn main():
    shelf = Shelf()
    shelf.items.push(owns Book(name=String("Dune")))
    w = weakref shelf.items[0]        # a class object in its own block: not a part, so [WK-16] allows it; [EFF-27] refuses only `sharedref mut` and `shares mut` here
    println(shelf.first_name())       # Dune, kept
    if w exists as ref mut b:         # [GRM-44]
        b.name = String("Emma")       # does not pass through `shelf`: no clear is placed
    println(shelf.first_name())       # prints the kept "Dune"

The text accepts it and the second call gives a stale result: [EFF-27]'s own reason (a change that does not pass through the method's object cannot be cleared) covers a `weakref` link opened `ref mut` exactly as it covers a `sharedref mut` link, and only the two are named. A stale kept result is not a memory fault, but [CLS-12] promises the result is cleared by "a change, through any name" of what the body reads, and here it is not.

- READER EDIT rule=EFF-27
  - old: so a `sharedref mut` link, or a `shares mut` owner, made into anything a `@cached` method of
  its holder reads is `E2117` at the line that makes it
  - new: so a `sharedref mut` link, a `shares mut` owner, or a `weakref` link, which may be opened `ref mut` (`[GRM-44]`), made into anything a `@cached` method of its holder reads is `E2117` at the line that makes it (for the link, "`shelf.items[0]` is read by `Shelf.first_name`, which is `@cached` and would not see a change made through this link; link the shelf instead, `weakref shelf`, or remove `@cached`"); across a call the fact `[OWN-13]` records, that a function makes a `weakref` link to what a parameter names, gives the same `E2117` at the call that passes it such a part
  - why: The rule's own reason names every link that can change the object without passing through the holder; the `weakref` link gets the same line, code and help as the `sharedref mut` link, decided when compiling, and the summary fact that already exists for the `thread_shared` check carries it across a call, so no second way of tracking links is added.

- READER EDIT rule=registry row E2117
  - old: a `sharedref mut` link or `shares mut` owner made into what a `@cached` method reads, at the line that makes it
  - new: a `sharedref mut` link, a `shares mut` owner or a `weakref` link made into what a `@cached` method reads, at the line that makes it (or at the call that passes such a part to a function that links it)
  - why: The row lists the cases of the code.

**remaining_doubt**: The finder's option B (allow the link, refuse the `ref mut` opening where it is made) needs the link's provenance kept through fields, which K17's ruling does not give and [BLD-16] forbids across packages; A is the one that follows the sentence beside it. A `shares` (read-only) owner into the read set stays allowed, since nothing changes through it; a `ref mut` view made from `shelf.items[0]` passes through `shelf`'s name where it is made, so the compiler places the clear at changes through that view by the same local flow it uses for views ([LT-1]).

**REFUTER reason**: The hole is real. [OWN-13] and [WK-16] allow the link (`shelf.items[0]` is a class object in its own block, not "an element stored inline"), [GRM-44] opens it `ref mut`, and [EFF-27] refuses only "a `sharedref mut` link, or a `shares mut` owner, made into anything a `@cached` method of its holder reads", though its own reason covers the link ("A change that does not pass through the method's object cannot be cleared that way"); [CLS-12] promises the result is kept "until a change, through any name, of a field the method's body reads clears it", and `b.name = String("Emma")` is such a change that clears nothing. But the reader's fix leaves the same hole open: a `weakref` link is made from any name at any time ("any name may make one", [WK-11]) and survives the object's hand-over into the holder, so `b = Book(name=String("Dune"))`, `w = weakref b`, `shelf.items.push(owns b)` puts a linked book under the cached read with no line that makes a link "into anything a `@cached` method of its holder reads"; the summary fact of [OWN-13] names only a parameter's chain, and [BLD-16] forbids following a link's provenance across packages. The cause is that a class object has identity and may be changed through a link that passes through no name of the holder, so the refusal belongs where the holder reads it, decided from the holder's own text: the class's weak-count fact, which [OBJ-1] fixes in the declaring package ("a type carries a weak count when ... The fact is part of the type's layout in its package's interface file") and which says exactly whether any `weakref` link anywhere can name the object ("no `weakref` link anywhere can name a value of one" that carries none). This keeps the `sharedref mut` and `shares mut` sentence of [EFF-27] as it is (those are refused where they are made) and costs nothing.

- BETTER EDIT rule=CLS-12
  - old: 
  - new: A `@cached` body reads nothing a link may change without passing through the holder: reading a field of a class object reached through a field or element of `this` (not `this` itself), of a class that carries a weak count (`[OBJ-1]`), is `E2117` ("`first_name` reads `items[0].name`, and a `Book` may be changed through a `weakref Book` link, made from any name before or after the book entered `shelf` (`[WK-11]`) and opened `ref mut` (`[GRM-44]`), without `Shelf` seeing it, so a kept result could be stale; remove `@cached` (line 7), or keep the names in `Shelf` itself: `names: Array[String]`"); a class that carries no weak count is named by no link, and a `sharedref mut` link or `shares mut` owner into the read set is refused where it is made (`[EFF-27]`).
  - why: At the cause: the one fact that says whether a link can reach the object is the class's own, read from its interface, so the check holds across packages and for a link made before the object entered the holder.

- BETTER EDIT rule=EFF-27
  - old: a link to the whole holder is allowed, since every change
  through it passes through the holder.
  - new: a link to the whole holder is allowed, since every change through it passes through the holder; a `weakref` link may be made from any name at any time, so the class objects it may name are kept out of a `@cached` body's reads instead (`[CLS-12]`).
  - why: Points from the rule whose reason covers the link to the rule that refuses it.

- BETTER EDIT rule=registry row E2117
  - old: a `sharedref mut` link or `shares mut` owner made into what a `@cached` method reads, at the line that makes it
  - new: a `sharedref mut` link or `shares mut` owner made into what a `@cached` method reads, at the line that makes it; a `@cached` body that reads a field of a class object of a class carrying a weak count, reached through a field or element of `this`
  - why: The row lists the cases of the code.

## K63-copied-array-tuple-changed  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: OWN-7: "A `Copy` struct copied with `=` from a name that is still used afterwards is not then changed through the new name (`E3023`): the line says `copy of` (a copy of your own) or `ref mut` (the original), so that no copy is changed by mistake."; OWN-7: "A value of a `Copy` type is duplicated bit for bit by `=` and on use, and the source stays valid; the duplicate is independent, and no copy runs any code."; §IV.8 marker table, Copy row: "always for scalars, `ref T`, `Span`, `StrView`, raw pointers, `extern fn`, tuples and fixed arrays of `Copy` elements"; TYP-36 table: "| tuples, `[T; N]` | if all | if all | if all | lexicographic, if all | if all | if all | if all `Debug` | if all |"; GRM-5: "`a, b = e` destructures a tuple, a struct or a fixed array."

**program**: 

- READER EDIT rule=OWN-7
  - old: A `Copy` struct copied with `=` from a name that is still used afterwards is not then
  changed through the new name (`E3023`):
  - new: A `Copy` value with parts (a struct, a tuple, a fixed array) copied with `=` from a name that is still used afterwards is not then changed through its parts or whole through the new name (`E3023`):
  - why: The rule's reason, that no copy is changed by mistake, holds for every `Copy` value whose parts can be changed, and most of all for a fixed array, which Python's `=` would share; one rule wherever the same mistake can be made. Checked when compiling, as today for structs.

**remaining_doubt**: The text as written decides the finder's option B: the rule names only a `Copy` struct, so `q = p` then `q[0] += 1.0` on a `[float; 3]`, and `u = t` then `u.0 += 1`, compile and change the copy while `p` and `t` keep their values (prints 0.0 0.0 0). That is safe ([OWN-7]: the duplicate is independent) and only inconsistent with the rule's stated reason; the edit extends the refusal, so it narrows the accepted set and is a hardening, not a safety fix. A number, `bool`, `char` or view copied with `=` has no part to change and is untouched.

**REFUTER reason**: The text decides it: [OWN-7] refuses only "A `Copy` struct copied with `=` from a name that is still used afterwards is not then changed through the new name (`E3023`)", and tuples and fixed arrays are `Copy` by the marker table ("tuples and fixed arrays of `Copy` elements") and [TYP-36]'s table ("tuples, `[T; N]` | if all | if all"), so `q = p` then `q[0] += 1.0` and `u = t` then `u.0 += 1` compile and change the independent copy ("the duplicate is independent, and no copy runs any code"); `p[0]` and `t.0` print 0.0 and 0. Safe as it stands. The reader's edit extends the refusal to every `Copy` value with parts for the rule's own stated reason ("so that no copy is changed by mistake"), which is a hardening of consistency, not a safety fix; it narrows the accepted set, is checked when compiling as for structs, touches no verdict (the rule is the redesign's R1), and the help lines (`copy of`, `ref mut`) already compile for arrays and tuples.

# MISSED

## MISSED: A part handed out of a value that has several owners leaves a hole the other owners read

**rules**: OWN-11, EXP-6, RC-1, RC-7

**program**:
```
struct Inventory:
    gold: int
    items: Array[String]

fn main():
    inv = Inventory(gold=5, items=[String("sword")])
    shop = shares mut inv                  # `inv` is in a block of its own from its declaring line, two owners ([RC-1])
    names: Array[Array[String]] = []
    names.push(owns inv.items)             # [OWN-11]: a part of a struct value, handed over on its own; `inv` owns it
    println(shop.items.len())              # the other owner reads the hole: a moved-out list

[OWN-11] hands a part over "for a struct value `p`" and refuses only "A field of a class object ... since other names may reach the object and would see the hole (`E3013`)" and "A hand-over out of a name or place that does not own the value"; `inv` is an owner (one of several, [RC-7]: "`items` may still change the list if it could before"), so the words accept the line, and `shop.items` reads freed memory, against [PHIL-10]. A `shares` owner only reads (`E3023`), a `thread_shared` owner changes nothing directly (`E7014`), and a `sharedref mut` holder or a name opened from a link does not own, so `shares mut` is the one owner the refusal misses.
```

- MISSED EDIT rule=OWN-11
  - old: A field of a class object is not handed out this way, since
  other names may reach the object and would see the hole (`E3013`:
  - new: A field of a class object, and a part of a value that has several owners (`shares mut inv`, `[RC-1]`), is not handed out this way, since other names may reach the value and would see the hole (`E3013`, with the same help, `names.push(owns mem.take(ref mut inv.items))`:
  - why: The reason the rule gives for a class object holds for every value another owner reaches; `mem.take` leaves a value in the place, which the other owners may read.

- MISSED EDIT rule=EXP-6
  - old: a field of a class object, which other names may reach and would see the hole, and
  anything reached through a name that does not own it, are `E3013` (`[OWN-11]`).
  - new: a field of a class object, a part of a value with several owners (`shares mut`), which other names may reach and would see the hole, and anything reached through a name that does not own it, are `E3013` (`[OWN-11]`).
  - why: The two defining places say the same.

## MISSED: A `@cached` body that reads a `static mut` or a static with its own operations goes stale

**rules**: CLS-12, EFF-27, STA-1

**program**:
```
static mut SCALE: float = 1.0

class Circle:
    r: float

    @cached
    fn area() -> float:
        return SCALE * this.r * this.r

fn main():
    c = Circle(r=1.0)
    println(c.area())      # 1.0, kept
    SCALE = 2.0            # no field of `c` changes, so no clear is placed ([EFF-27]: the result "depends on the fields its body reads")
    println(c.area())      # prints the kept 1.0

[CLS-12] refuses a body that "changes anything" or reads a `shares`, `weakref`, `Cell` or thread-changed field, and [EFF-27] clears the result at "every change, through any name, to one of them", them being "the fields its body reads"; a `static mut`, or a `static` `Mutex`, `Atomic` or `guarded by` variable, is read by the body, changes through no name of the object, and is cleared by nothing. Not a memory fault, but [CLS-12]'s promise ("gives it again until a change ... of a field the method's body reads clears it") is broken by a program the text accepts.
```

- MISSED EDIT rule=CLS-12
  - old: 
  - new: `@cached` is `E2117` too on a method whose body reads anything not reached through `this` that may change (a `static mut`, or a `static` `Mutex`, `RwLock`, `Atomic`, `Cell`, channel end or `guarded by` variable, `[STA-1]`, directly or through a call, `[EFF-24]`), since a change of it passes through no name of the object and clears nothing ("`area` reads `SCALE`, a `static mut` that may change without `Circle` seeing it, so a kept result could be stale; remove `@cached` (line 5), or keep the scale in the object: `scale: float`").
  - why: The kept result follows only what passes through the object; the rule names the one other source of change the body can read. Decided from the body's summary, which already exists.

- MISSED EDIT rule=registry row E2117
  - old: on a `virtual` method or an `override`;
  - new: on a `virtual` method or an `override`; on one whose body reads a `static mut` or a `static` with its own operations;
  - why: The row lists the cases of the code.

## MISSED: `sharedref mut dyn I` is grammatical by [TYP-45a] and decided by no rule

**rules**: TYP-45, TYP-45a, IFC-5, WK-16

**program**:
```
interface Listener:
    fn on_event(code: int)

class Hud:
    name: String

extend Hud implements Listener:
    fn on_event(code: int):
        println(this.name, code)

class Bus:
    listeners: Array[sharedref mut dyn Listener] = []    # [TYP-45a]: a word of [TYP-45] "stands before ... a `dyn` bound"; [TYP-45] names only `owns`, `shares` and `shares mut` with `dyn`

fn main():
    hud = Hud(name=String("hud"))
    bus = Bus()
    bus.listeners.push(sharedref mut hud)
    bus.listeners[0].on_event(1)

[TYP-45a] admits the form ("A word of `[TYP-45]` stands before a type that has no word of its own: a `path_type`, a tuple, a fixed array, a function type or a `dyn` bound"), [TYP-45] gives `dyn` only three words ("`owns dyn I`, `shares dyn I` and `shares mut dyn I` ... `dyn I` alone is a parameter's type") and [IFC-5] names `ref dyn I` and `ref mut dyn I`; whether the list above is a type, and what its element is, is not said. The K35 edit of [WK-16] decided only `weakref dyn I`.
```

- MISSED EDIT rule=TYP-45
  - old: `dyn I` alone is a parameter's type, a read-only view of the value (`ref mut
    v: dyn I` may change it).
  - new: `dyn I` alone is a parameter's type, a read-only view of the value (`ref mut v: dyn I` may change it); anywhere else a view of such a value is `ref dyn I`, `ref mut dyn I` or `sharedref mut dyn I` (`[IFC-5]`, `[TYP-45a]`), one pointer and one table, governed as every view of its word is (`[LT-46]`, `[TYP-15]`).
  - why: A view of a `dyn` value needs no header the link cannot know (the table travels with the pointer), so the three view words apply as to any type; the rule says so where the `dyn` forms are listed.
