# K05: May a thread_shared value take its memory from an arena or another memory manager, and if so under which rule?

## Design 1

**question**: K05: May a `thread_shared` value take its memory from an arena or another memory manager, and if so under which rule? (Angle: speed first, the smallest cost while the program runs, then safety.)

**summary**: Yes, under [TYP-46a] as the only gate: `thread_shared T using A` is allowed for every manager, the value is counted away from its making line ([RC-4] point 4, so no thread ever asks which thread made it), and it is refused (E7015 at the making line) only when its end or growth would call a manager that is not `ThreadShared`; a one-thread `Arena`, which frees nothing one by one, is accepted, so the gap sweep's own program compiles. The current text says both things at once (five places refuse with E2114: [TYP-46], [OBJ-4], [ALC-1], [OPT-1], the E2114 row; six allow: [TYP-46a], [OBJ-1], [RC-4], [ALC-5], [COST-3], the E7015 row), so an implementer cannot say whether `stats = thread_shared Stats() using level` compiles; the edits keep the allowing half, remove the refusing half, and sharpen the cost: where the place's type says `using A` the compiler emits the atomic path from the type, with no test of the handed-over mark. The method-3 team's shape A (E2114 for every such `using`) is safe too but removes the fastest shape for shared values made in bulk and adds a cost (one runtime chunk allocation and free per value) that the programmer cannot avoid; shape B costs programs that do not write the word nothing.

- EDIT rule=registry row E2114
  - old: ; `thread_shared` before a value made `using m`, whose block would carry no making-thread record | `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[LT-48]`, `[OBJ-4]` |
  - new:  | `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[LT-48]` |
  - why: After the edit below [OBJ-4] defines no E2114, and `using` after `thread_shared` is decided by [TYP-46a] (E7015), one code per fact.

- EDIT rule=registry row E7015
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: `thread_shared T using A` whose value holds a `ref A` (`[TYP-46a]`) and `A` is not `ThreadShared`;
  - why: The row must claim what [TYP-46a] says after its correction: a value made in a manager that frees nothing one by one and does not grow holds nothing, so no manager is called on another thread and nothing is refused.

- EDIT rule=LT-4
  - old: that address is a kept `sharedref mut`
  link to the instance (`[LT-46]`): the program may use the instance while values made in it live,
  - new: that address is a kept `sharedref mut`
  link to the instance (`[LT-46]`) for the exclusivity rules, and counts as a `ref A` for `Send` and
  `ThreadShared` (`[TYP-46a]`): the program may use the instance while values made in it live,
  - why: Verdict 164 makes a type holding a `sharedref mut` link neither `Send` nor `ThreadShared`; read literally, [LT-4] would make every `using` value one-thread and `thread_shared T using A` impossible for any `A`. [TYP-46a] already models the word as a `ref A` for the thread markers; [LT-4] now says so at the place that names the link.

- EDIT rule=ALC-1
  - old:   `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String`; never a
  `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`) names
  - new:   `shares`, `shares mut` or `thread_shared` value, a class object, an `Array`, `Map`, `Set` or
  `String`; a `thread_shared` value made in any manager but `Global` is counted away from its
  making line, `[RC-4]` point 4, and is `ThreadShared` as `[TYP-46a]` says) names
  - why: [ALC-1]'s list contradicted [RC-4] point 4, [OBJ-1], [ALC-5] and [COST-3], which all describe a `thread_shared` value made `using` an instance. The list is restored to the 0.9.15 list with the property that decides the case beside it.

- EDIT rule=OBJ-1
  - old:   never changes. A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new:   never changes. A `thread_shared` value made `using` any manager but `Global` (`[ALC-1]`; `System`
  and a one-instance manager a program declares included) lies in memory the runtime did not lay
  out, so it is marked handed over at the line that makes it (`[RC-4]` point 4) and no thread asks
  which thread made it.
  - why: The chunk record exists only for blocks the runtime takes for `Global` (the sentence before: "the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks"). A `System` block comes from the platform's allocator and has no record, so [RC-4]'s "a value made in `Global` or `System` keeps the two counts" would read a record that is not there; one property, stated once.

- EDIT rule=§H.52 change log (the Hardened_2 entry under "Holes in the memory-safety rules")
  - old: * A `thread_shared` value is never placed in a frame or a manager's instance (`[OPT-1]`,
  `[OBJ-4]`, `[ALC-1]`).
  - new: * A `thread_shared` value is never placed in a frame; one made in a manager's instance is counted
  away from its making line (`[OPT-1]`, `[OBJ-4]`, `[ALC-1]`, `[RC-4]`).
  - why: The log entry claimed the refusing half; it now claims what the rules say.

- EDIT rule=OPT-1
  - old: A `thread_shared` value is never placed in a frame, and never in a
  memory manager's instance: its making thread is read from its chunk (`[RC-4]`), which only the
  runtime allocator's blocks have.
  - new: A `thread_shared` value is never placed in a frame: made in `Global`,
  its making thread is read from its chunk (`[OBJ-1]`), which only the runtime's blocks have, and
  made `using` another manager it lies in that manager's memory and is counted away from its making
  line (`[RC-4]` point 4).
  - why: "never in a memory manager's instance" contradicted [RC-4] point 4 and [OBJ-1]; the frame rule stays (a frame slot has no chunk record and no manager, and the gain would be nil for a value made to be shared).

- EDIT rule=OBJ-4
  - old: A `thread_shared` value's block always comes from the runtime allocator
  (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is
  `E2114` ("a `thread_shared` value's block carries its making thread's record, which only the
  runtime allocator keeps; remove `using m`: `names = thread_shared Names()`").
  - new: A `thread_shared` value made in `Global` takes its block from the runtime's
  chunks, which carry its making thread's record (`[OBJ-1]`); one made `using m` takes it from `m`
  by calling `m.allocate(…)`, keeps its instance word as any value made in `m` does, and is counted
  away from its making line (`[RC-4]` point 4), so its record is never needed.
  - why: This was the defining sentence of the refusal. The cause of the gap sweep's question was that the making thread could not be found for such a block; [RC-4] point 4 answers it by never asking, so the refusal has no reason left. The place where the problem is caused is the block's source, which this rule states.

- EDIT rule=TYP-46
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It may follow an `owns`,
  `shares`, `shares mut` or `thread_shared` type, a class type, or an `Array`, a `Map`, a `Set` or a
  `String` (`name: String using LevelArena = String("") using current`); a `thread_shared T using A`
  is a `thread_shared T` whose block comes from an `A` and whose every owner is counted atomically
  from its making line (`[RC-4]` point 4), and it is `ThreadShared` as `[TYP-46a]` says; after any
  other type it is `E2114`
  - why: The list is the type-level statement of the same property as [ALC-1] and must agree with it and with [TYP-46a], which already names `thread_shared T using A` as a type that exists.

- EDIT rule=§X.4 [COST-3] row "A `thread_shared` value's two counts"
  - old: a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4);
  - new: a value made `using` any manager but `Global` counts every owner atomically from its making line (`[RC-4]` point 4), the path chosen from the place's type, so it pays no test of the handed-over mark and never the queue; its block costs its manager's `allocate` (a bump for an arena) and its end that manager's `deallocate`, or nothing in a manager that frees nothing one by one (`[ALC-1]`), so against a value made in `Global` it gives up the plain home count (25% on a thread's own values, measured) for the manager's allocation and free, a trade the programmer makes per value by the word;
  - why: Every cost has a row: the row now prices the shape and says that nothing runs for it in a program that does not write the word, and that the mark test is not paid where the type decides.

- EDIT rule=RC-4 point 4
  - old: A `thread_shared` value made `using` an instance
     of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that
     makes it: its instance may be reset or end as soon as the compiler sees its last owner end
     (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: A `thread_shared` value made `using` any manager
     but `Global` (`[ALC-1]`; `System` and a one-instance manager a program declares included) is
     marked handed over at the line that makes it: its block lies in memory the runtime did not lay
     out and names no making thread (`[OBJ-1]`), and its instance may be reset or end as soon as the
     compiler sees its last owner end (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point
     would miss; so every owner of such a value is counted in its away count, its end is never
     delayed, and where the place's type says `using A` the compiler emits the away path with no
     test of the mark (where it does not, the mark decides, as after any hand-over); a value made in
     `Global`, with no `using` or `using Global`, keeps the two counts, its block waits for the check
     point and nothing reuses it.
  - why: Two corrections at the cause: a one-instance manager's block (`System`, or a fieldless manager the program declares) has no chunk record either, so "may have several instances" and "or `System`" were wrong; and the cost is stated: the type at the line, which every package sees ([BLD-16]), chooses the path, so the mark test and the queue are never paid for such a value.

- EDIT rule=TYP-46a
  - old: `thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making
  line ("`st` would end on whichever thread holds its last owner, and `current`, an `Arena`, is used
  by one thread; make it in the built-in manager, `st = thread_shared Stats()`, or in a manager any
  thread may call").
  - new: `thread_shared T using A` is a `thread_shared T` counted away from its making line (`[RC-4]`
  point 4); it is `E7015` at the making line when the value holds a `ref A` and `A` is not
  `ThreadShared` ("`stats` would end on whichever thread holds its last owner, giving its block back
  to `pool`, a `BlockPool`, which is used by one thread (`deallocate`, line 18, frees blocks one by
  one); make it in the built-in manager, `stats = thread_shared Stats()`, in an arena, `stats =
  thread_shared Stats() using frame` with `frame = Arena(1 << 20)`, or in a manager any thread may
  call"). So `st = thread_shared Stats() using frame` with `frame` an `Arena` compiles: the value
  holds nothing (`[ARN-2]`: the arena frees nothing one by one), its end on any thread runs its
  `drop` and touches the arena not at all, and the arena is reset or ends only after the compiler
  sees its last owner end (`[ALC-5]`, `[LT-4]`).
  - why: The rule's own first sentences say a value made in a manager that frees nothing one by one and does not grow holds nothing, and [ARN-2] says an `Arena` frees nothing one by one; so the example refused exactly the program the property accepts. The example now names a manager whose `deallocate` does something, and the accepted shape, the fastest one, is stated.

**refused_program**: from std.mem import Allocator, Layout, System
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

struct BlockPool:                          # one thread's pool of 64-byte blocks, no lock
    free: Array[*mut u8] = []

unsafe extend BlockPool implements Allocator:
    # SAFETY: every block is 64 bytes from `System`, handed out once until it is given back.
    fn allocate(layout: Layout) -> *mut u8:
        if layout.size > 64: return null[*mut u8]()
        match this.free.pop():
            Some(p): return p
            None: return System.allocate(Layout(64, 16))
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        this.free.push(p)                  # gives blocks back one by one, so a value made here keeps `ref BlockPool`
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null[*mut u8]()

fn main():
    pool = BlockPool()
    stats = thread_shared Stats() using pool
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")

Refused at line 24 with E7015 ([TYP-46a]): "`stats` would end on whichever thread holds its last owner, giving its block back to `pool`, a `BlockPool`, which is used by one thread (`deallocate`, line 18, frees blocks one by one); make it in the built-in manager, `stats = thread_shared Stats()`, in an arena, `stats = thread_shared Stats() using frame` with `frame = Arena(1 << 20)`, or in a manager any thread may call". (A `BlockPool` whose `free` list sits under a `Mutex`, so that the type is `ThreadShared`, makes the same line compile.)

**accepted_program**: from std.arena import Arena
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    level = Arena(1 << 20)                        # a one-thread arena; it frees nothing one by one ([ARN-2])
    stats = thread_shared Stats() using level     # its block is a bump in `level`; every owner is counted atomically from here
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))   # the task holds the last owner and ends it on its thread
    println("done")

Compiles and prints `done`. The value holds no `ref Arena` ([TYP-46a]: a manager that frees nothing one by one, a value that does not grow), so it is `ThreadShared` and goes to the task; its end on the task's thread runs `drop` and calls nothing on `level`; `level` is reset or ends only after the scope's end, where the compiler sees the last owner end ([ALC-5], [LT-4]: the value carries `level`'s region, so keeping it past `level` would be E3063). The same program with `thread_shared Stats()` (no `using`) also compiles, with the two counts and a block from the runtime's chunks; the programmer chooses by the word.

**runtime_cost**: For a program that writes no `using` after `thread_shared`: nothing new (the handed-over-mark test on the making thread is [RC-4] point 4's existing cost, paid for the ordinary hand-over, not for this). For a `thread_shared T using A` value: at its making, one call of `A.allocate` (a bump for an arena, `System`'s `malloc` for `System`) and the header stores, the away word written with one owner and the handed-over mark; the 32-byte header as for every `thread_shared` value ([OBJ-1], verdict 101); every owner made or ended is one atomic `fetch_add` with the limit test on the returned value, or one atomic subtract ([RT-10]), the end one compare-exchange on the away word ([RC-4] point 2), emitted from the place's type with no load-and-branch on the mark; never the queue and never a check point's work; at its end `drop`, the atomic weak-count subtract, then `A.deallocate` through the kept instance word, or by address for `FoundByAddress`, or nothing for a manager that frees nothing one by one ([ALC-1], verdict 107). Against a value made in `Global` it gives up the plain home count (the brc report: a thread's own values 25% faster with two counts) for the manager's allocation and no runtime free. §X.4 row: `[COST-3]` "A `thread_shared` value's two counts", edited as above. The checks (E7015, E3063, E3021, E2020) run when compiling and nothing runs for them.

**implementer_steps**: 1. Grammar and types: `using A` is accepted after `thread_shared T` in a type and after a value on a making line ([GRM-43] binds `using` tighter than the word, so `thread_shared Stats() using level` is `thread_shared (Stats() using level)`); `thread_shared T using A` is a type distinct from `thread_shared T`, as `owns T using A` is from `owns T` (E2020 for a `Global` value given to a `using A` place, [ALC-1]).
2. At the making line, read from `A`'s declaration or interface file ([BLD-14] item 14) whether its `deallocate` is empty and whether it implements `FoundByAddress`; the value holds a `ref A` exactly when its end or growth calls the instance ([TYP-46a]); infer `ThreadShared` for the value from its components with that `ref A` included; refuse with E7015 when it is not (verdict 165: at the making line). No whole-program fact is read ([BLD-16]).
3. Region check as for any `using` value: the value carries the instance's region ([LT-4] E3063, [ALC-5] E3021, [ALC-5a] for links); a hand-over to an unscoped thread is allowed only where the instance lives as long as the process (a one-instance `static` manager), else E3063; scoped tasks follow [THR-11] with growth and end as changes of the instance only where the value holds the `ref A`.
4. Code for the making line: `p = inst.allocate(Layout(32 + size_of[T], align))`, panic on null as [ALC-6] says; write header: home 0, type index, weak 1, away = one owner with the handed-over mark set, next null; store the instance word after the header where [ALC-1] says the value keeps one; run the constructor or hand the value in.
5. Code for count lines: where the place's type carries `using A`, emit the away path unconditionally (relaxed atomic add plus limit test, acquire-release subtract, the end's compare-exchange, [RT-8], [RT-10]); where the place's type is plain `thread_shared T`, emit today's code (the mark test chooses). Opening a `weakref thread_shared T using A` link likewise takes the away path from the type.
6. Drop glue for the type: `drop`, the atomic weak subtract, and when it reaches zero `A.deallocate(block, layout)` through the kept word, by address ([ALC-2]'s mask and load) or no call for an empty `deallocate`, exactly the end of an `owns T using A` reached from the away end step; the runtime's queue code is untouched, since the away count of such a value never goes below zero.
7. Runtime and FFI: `ember_rt_object_release` and every runtime entry that takes a bare header read the mark, so a using-made value handed to C ([FFI-35a], [FFI-54]) ends correctly on any thread; a host's allocation functions ([FFI-27]) still feed only `Global`'s chunks. Hot reload: registration is [HR-12]'s, on whichever thread makes or ends it; the block is one live instance ([RC-1]).
8. Tests: the two programs above; `thread_shared Stats() using System` (compiles, atomic from birth, `free` on any thread); `thread_shared Stats() using Global` equals no `using`; a `ThreadShared` `FoundByAddress` pool; the E7015 message; `ember inspect --cost` reports the row; a stress run with ends on several threads for a value in a one-thread `Arena`, the arena reset after the scope.

**departs_from_verdict**: None. Verdict 19 (`using` names a manager) and 24 (the place names the kind, the making line the instance) are kept for `thread_shared` as for every other owner word. Verdict 101 is kept whole: the header stays 32 bytes, the two numbers stay for every value made in `Global`, and a value made `using` a manager is in the state fix 2 defines ("a hand-over the compiler sees moves A into B at that moment"), the making line being a hand-over the compiler sees. Verdict 107 is used as written (a manager that frees nothing one by one gives values that carry no word, which is what makes a one-thread `Arena` acceptable). Verdict 164 is respected by the [LT-4] clarification, which follows [TYP-46a]. Verdict 165 is the shape of the refusal (at the making line). The ruling departs from the method-3 team's recommendation A (not a verdict) and from the Hardened_2 lead's "a value made in `Global` or `System` keeps the two counts" (not a verdict; [OBJ-1] lays out chunks for `Global` only).

**trade_offs**: Speed: shape B adds nothing to a program that does not write the word, and for one that does it offers the fastest shape for shared values made in bulk (a bump allocation, no free, no queue), at the price of the plain home count on the making thread (the brc report's 25% on a thread's own values); which side is cheaper depends on the program's ratio of count operations to makings, and the word makes it the programmer's choice per value, visible in the type (the owner's rule: build the variants and let the program pick). Shape A would force one runtime chunk allocation and free on every shared value and forbid the arena for the block itself; its only run-time merit, one counting path for all shared values, is already lost to [RC-4] point 4's hand-over state, which every implementation must have. Readability: one meaning for `using` after every owner word, no exception to learn; the refusal, where it comes, names the manager and the three lines to write. Safety: unchanged and all when compiling: the end on another thread never calls a one-thread manager (E7015), the value never outlives or is reset under its instance (E3063, E3021), nothing reads a making thread for such a value, and the mark is set at making so even a path the type does not decide stays correct. Consistency: removes a contradiction between eleven places and the two codes for one fact (E2114 against E7015); E2114 keeps its one meaning, a type with no heap memory of its own. Given up: the chance to use the home word for something else in such a value (kept 4 bytes unused, so one header layout and one `TypeInfo`), and a value made `using System` loses the two counts (a second per-thread chunk pool for a rarely written form was judged more machinery than the plain count is worth).

**remaining_doubt**: needs_owner: both shapes are safe, so safety does not decide; the recommendation is B as edited above (allow, atomic from the making line, [TYP-46a] the only gate, a one-thread `Arena` accepted), since it is cheaper at run time for the programs that want it and free for the rest, and since A would drop a form rather than make it work. Open points: (1) the E2020 row states only one direction ("a value given to a `using A` place that was not made in an instance of `A`"); the no-mark-test path assumes a `using A`-made value is never given to a plain `thread_shared T` place, as the instance word already forces for `owns T using A`; safety does not depend on it (the mark is set), only the saved test does, so K34 should state both directions. (2) Hot reload: [HR-15] migrates an instance to a new block when the new size does not fit; for a block made in a manager's instance the new block must come from the same instance (or the manager's `allocate` through the kept word); this holds for every `using` value, not only `thread_shared`, and is not settled in the text. (3) The trade (chunk allocation and biased counts against an arena bump and atomic counts) is priced here from the brc report and verdict 107's measurements, not measured as one program; the method-3 team's E1-4 measurement stands. (4) The refused program's `Allocator` body uses `this`, `null[*mut u8]()` and `match` as the text spells them elsewhere; [ALC-2] prints the method signatures without a receiver, so the exact receiver spelling of `allocate` through a `ref` for a `ThreadShared` manager (its change made under its own lock or atomics) belongs to P01, not here.

## Design 2

**question**: K05: May a `thread_shared` value take its memory from an arena or another memory manager, and if so under which rule? (J5-05; the current text says both "never, E2114" in [TYP-46], [ALC-1], [OBJ-4], [OPT-1] and the E2114 registry row, and "yes when A is ThreadShared, counted away from its making line" in [TYP-46a], [OBJ-1], [RC-4] point 4, [ALC-5] and the E7015 registry row.)

**summary**: Yes: `thread_shared T using A` is allowed, and the deciding rule is [TYP-46a]'s own property, not a flat test on A. The value is accepted when what it would hold is `Send` and `ThreadShared` (it holds nothing, as in an `Arena`, which "frees nothing one by one" [ARN-2]; or it holds a link to a manager any thread may call: a `ThreadShared` type whose `deallocate` and `reallocate` read `this`), and E7015 at the making line otherwise; its block comes from the instance, so it is counted in its away count from its making line ([RC-4] point 4) and no thread ever asks which thread made it, chosen by the type at every line with no run-time test. The four refusing sentences (E2114) are removed, [RC-4] point 4's property is corrected from "a manager that may have several instances" to "any manager but Global" (a `System` or `static` manager's block has no chunk header either), and [TYP-46a]'s "A is ThreadShared" is tightened to "any thread may call", which the current text lacks and which lets a race through today.

- EDIT rule=registry row E2114 (§XVI table, line 9873)
  - old: ; `thread_shared` before a value made `using m`, whose block would carry no making-thread record | `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[LT-48]`, `[OBJ-4]` |
  - new:  | `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[LT-48]` |
  - why: [OBJ-4] no longer emits E2114 for `thread_shared … using m`; one code, one defining rule, and the row must claim only what its rules say.

- EDIT rule=registry row E7015 (§XVI table, line 10004)
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: `thread_shared T using A`, at the making line or at the type, where the value would hold a link to a manager that not every thread may call;
  - why: The row follows [TYP-46a]'s corrected condition (below): the test is on what the value holds, not on A's marker alone.

- EDIT rule=§H.52 change log (line 14568)
  - old: * A `thread_shared` value is never placed in a frame or a manager's instance (`[OPT-1]`,
  `[OBJ-4]`, `[ALC-1]`).
  - new: * A `thread_shared` value is never placed in a frame; one made `using` a manager is counted away from
  its making line, chosen by its type at every line (`[OPT-1]`, `[OBJ-4]`, `[ALC-1]`, `[RC-4]`, `[TYP-46a]`).
  - why: The log line restated the sentence this ruling removes; a cross-reference claims what its rule says.

- EDIT rule=§IV marker table, row `Send` (line 1859)
  - old: the `ref A` a value made in a manager's instance holds, unless `A` is `ThreadShared`, `[TYP-46a]`)
  - new: the link to its instance a value made in a manager's instance holds, unless `A` is a manager any thread may call, `[TYP-46a]`)
  - why: The table repeated [TYP-46a]'s condition; it follows the corrected one.

- EDIT rule=§IV marker table, row `ThreadShared` (line 1860)
  - old: (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`)
  - new: (the link to its instance a value made in a manager's instance holds included: a `ref A` for a manager any thread may call, else a `sharedref mut` link, `[TYP-46a]`)
  - why: Same: the held link is a `ref A` only when A's methods read `this`; otherwise it is [LT-4]'s `sharedref mut` link, which is not `ThreadShared` ([LT-46], verdict 164).

- EDIT rule=OPT-1
  - old: A `thread_shared` value is never placed in a frame, and never in a
  memory manager's instance: its making thread is read from its chunk (`[RC-4]`), which only the
  runtime allocator's blocks have.
  - new: A `thread_shared` value is never placed in a frame: one made with no `using` has its making
  thread read from its chunk (`[OBJ-1]`), which only the runtime's chunks carry, and one made `using`
  an instance stays in the block that instance gave, whose `allocate` and `deallocate` are the
  program's own work (`[RC-4]` point 4).
  - why: "never in a memory manager's instance" contradicted [OBJ-1], [RC-4] point 4 and [ALC-5], which describe exactly such a value. The frame rule stays: a frame block has no chunk header and skipping a program's own `allocate` would be observable.

- EDIT rule=OBJ-4
  - old: A `thread_shared` value's block always comes from the runtime allocator
  (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is
  `E2114` ("a `thread_shared` value's block carries its making thread's record, which only the
  runtime allocator keeps; remove `using m`: `names = thread_shared Names()`").
  - new: A `thread_shared` value's block comes from the runtime's chunks (`[OBJ-1]`) when it is made
  with no `using`, and from the instance named when it is made `using` one (`[TYP-46a]`): the first
  has its making thread read from its chunk's header, the second is counted away from the line that
  makes it and asks no thread which made it (`[RC-4]` point 4).
  - why: The refusing sentence is one of the two answers the merge left standing; the reason it gave ("only the runtime allocator keeps" the record) is true and is why such a value is counted away from birth, not why it must be refused.

- EDIT rule=ALC-1
  - old: (an `owns`,
  `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String`; never a
  `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`)
  - new: (an `owns`,
  `shares`, `shares mut` or `thread_shared` value, a class object, an `Array`, `Map`, `Set` or
  `String`; a `thread_shared` value's block then comes from the instance and never from the
  runtime's chunks, so it is counted away from its making line, `[RC-4]` point 4, `[TYP-46a]`)
  - why: ALC-1 repeats TYP-46's list; both lists return to the owner's shape (verdict 19: `using` after the owner words) and point to the rule that decides a `thread_shared` value's manager.

- EDIT rule=TYP-46
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It may follow an `owns`,
  `shares`, `shares mut` or `thread_shared` type (a `thread_shared` value only where `[TYP-46a]`
  allows), a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name: String using
  LevelArena = String("") using current`); after any other type it is `E2114`
  - why: The list is where the contradiction with [TYP-46a] is caused; `thread_shared` returns to it, as in 0.9.15, and the rule that decides which manager is named.

- EDIT rule=§X.4 row "A `thread_shared` value's two counts" (COST-3 table, line 6193)
  - old: a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4);
  - new: a value made `using` any manager but `Global` counts every owner atomically from its making line, chosen by the type of the place at each line with no test of the mark and no read of a chunk (`[RC-4]` point 4), and gives its block back through its manager's `deallocate` on the thread that ends it;
  - why: The row must name what runs: for such a value the away path only, decided when compiling, plus the manager's own free; "an instance" is widened to every manager but `Global`, since a `System` or `static` manager's block has no chunk header either.

- EDIT rule=OBJ-1
  - old: A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new: A `thread_shared` value made `using` any manager but `Global` (`[ALC-1]`, `[TYP-46a]`) lies in
  memory the runtime did not lay out, so it is marked handed over at the line that makes it
  (`[RC-4]` point 4) and no thread asks which thread made it; it never enters a queue, so where it
  keeps its instance's address (`[ALC-1]`) that address stands in the header's queue place, and the
  value costs no word beyond the 32 bytes.
  - why: "an instance" left `using System` and a program's own `static` manager (no instance on the line, one instance, no chunk header) to the two-count path, which would read a making thread from memory that has none; and the implementer needs to know where the instance word of verdicts 47 and 107 goes: the 8-byte queue place is free for a value that is never queued.

- EDIT rule=TYP-46a (the Send/ThreadShared clause)
  - old: it is `Send`, and `ThreadShared`, only
  when `A` is `ThreadShared` (`[THR-8]`, `[THR-9]`), since those calls run on whichever thread holds
  or ends it.
  - new: it is `Send`, and `ThreadShared`, only
  when `A` is a manager any thread may call (below; `[THR-8]`, `[THR-9]`), since those calls run on
  whichever thread holds or ends it.
  - why: `ThreadShared` says several threads may hold and read a value at once ([THR-9]); it says nothing about a `deallocate` that changes `this`, which the ending thread would run while the owner thread allocates: a `struct Pool: slab: Array[u8]; free: Array[usize]` is inferred `ThreadShared` and the current text accepts the race.

- EDIT rule=TYP-46a (the E7015 sentence)
  - old: `thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making
  line ("`st` would end on whichever thread holds its last owner, and `current`, an `Arena`, is used
  by one thread; make it in the built-in manager, `st = thread_shared Stats()`, or in a manager any
  thread may call").
  - new: A manager **any thread may call** is a `ThreadShared` type whose `deallocate` and `reallocate`
  read `this` (`[EXC-15]`, a fact of its summary, `[BLD-14]` item 1; a `static` manager's do,
  `[ALC-1]`, and so do those of a type that locks inside, `inner: Mutex[Slab]`); a value whose end or
  growth would call a method that changes its instance holds the `sharedref mut` link of `[LT-4]`
  instead, which is neither `Send` nor `ThreadShared` (`[LT-46]`, verdict 164), whatever the
  instance's type. So `thread_shared T using A` is `E7015` at the making line when the value made
  would hold a link to an instance that is not one any thread may call: a value that grows made in
  an `Arena`, or any value in a manager whose `deallocate` changes `this` ("`msg` would end on
  whichever thread holds its last owner, and that end gives its block back through
  `Pool.deallocate` (line 12), which changes `pool`; make it in the built-in manager, `msg =
  thread_shared Msg()`, or in a manager any thread may call, whose `deallocate` reads `this` and
  locks inside"); a place of such a type (`log: thread_shared Log using Pool`) is `E7015` at the
  type, since no value could fill it. A `thread_shared` value that holds nothing (one that does not
  grow, made in a manager that frees nothing one by one: `stats = thread_shared Stats() using level`
  in an `Arena`, `[ARN-2]`) is `Send` and `ThreadShared` when `T` is, and is accepted: its end on
  any thread runs its `drop` and touches the arena not at all, every owner of it is counted in its
  away count from its making line (`[RC-4]` point 4), and the arena is neither reset nor ended
  while it lives (`[ALC-5]`, `[LT-4]`).
  - why: The flat test on A refused the J5-05 program for a reason untrue of it (nothing in the value's end touches the arena) and accepted a pool whose `deallocate` changes `this` from two threads. Stating the property (what the value holds, and whether its instance's methods may run on any thread) decides every manager the same way, including a program's own, and takes no feature away.

- EDIT rule=TYP-46a (the THR-11 clause)
  - old: unless `A` is `ThreadShared`. For the exclusivity rules `[ALC-5]` stands:
  - new: unless `A` is a manager any thread may call, whose methods change nothing. For the exclusivity
  rules `[ALC-5]` stands:
  - why: Two scoped tasks growing values in a `ThreadShared` bump manager whose `reallocate` changes `this` both change it; the exemption must name the property that makes the growth a read.

- EDIT rule=RC-4 point 4
  - old: A `thread_shared` value made `using` an instance
     of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that
     makes it: its instance may be reset or end as soon as the compiler sees its last owner end
     (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: A `thread_shared` value made `using` any manager but `Global` (`System`, a `static` manager
     and every instance alike, `[ALC-1]`, `[TYP-46a]`) is marked handed over at the line that makes
     it, in the header's first store, before any other thread can reach it: its block lies in memory
     the runtime did not lay out, so no chunk header names its making thread (`[OBJ-1]`), and its
     instance may be reset or end as soon as the compiler sees its last owner end (`[ALC-5]`,
     `[LT-4]`), which an end delayed to a check point would miss; so every owner of such a value is
     counted in its away count, its end is never delayed, and the thread that ends it gives the
     block back through the manager's `deallocate` there. The compiler reads this from the type of
     the place at the line (`thread_shared T using A`, `[BLD-16]`) and emits the away path with no
     test of the mark and no read of a chunk; the runtime's own routines, which see no type
     (`ember_rt_object_release`, the hand-over walk of `[THR-2]`, a reload's registration,
     `[HR-12]`), test the mark before anything reads a chunk header, so a block the runtime did not
     lay out is never asked for its making thread. A value made in `Global`, with or without `using
     Global` written, keeps the two counts, its block waits for the check point and nothing reuses
     it.
  - why: This is where the run-time behaviour is defined. "a manager that may have several instances" left `System` and a `static` manager on the two-count path with no chunk to read; the order "mark before chunk" is what makes a block without a chunk header safe in the routines that cannot know the type; and the type-chosen path is what keeps the generated code at C speed, with no new test.

**refused_program**: from std.sync import Atomic
from std.mem import Allocator, Layout
import std.thread

struct Pool:                      # every field is ThreadShared, so Pool is inferred ThreadShared ([THR-9])
    slab: Array[u8]
    free: Array[usize]

# SAFETY: allocate hands out a slot no live value holds; deallocate returns it to `free`
unsafe extend Pool implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        i = this.free.pop()                                   # changes `this`
        return this.slab.ptr_at(i * 64)
    unsafe fn deallocate(p: *mut u8, layout: Layout):         # line 12
        this.free.push(owns this.slab.index_of(p))            # changes `this`
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null                                           # a slot never grows

class Msg:
    hits: Atomic[int] = Atomic(0)

fn main():
    pool = Pool(slab=[], free=[])
    msg = thread_shared Msg() using pool
    with scope = thread.scope():
        scope.spawn(owns fn() owns msg: msg.hits.fetch_add(1))
    println("done")

# E7015 at `msg = thread_shared Msg() using pool`:
# "`msg` would end on whichever thread holds its last owner, and that end gives its block back through
# `Pool.deallocate` (line 12), which changes `pool`; make it in the built-in manager, `msg = thread_shared Msg()`,
# or in a manager any thread may call, whose `deallocate` reads `this` and locks inside"
# (The current text accepts this program, `Pool` being `ThreadShared`, and the task's end pushes to `free`
# while `main` may pop it: a data race in Safe Ember. The method bodies are illustrative.)

**accepted_program**: from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    level = Arena.with_capacity(64 * 1024)          # frees nothing one by one ([ARN-2]), so the value holds nothing
    stats = thread_shared Stats() using level       # its block from `level`; every owner counted in its away count from here
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))   # the task holds the last owner and ends it on its thread
    println("done")

# Compiles and prints done. The task's end of the last owner is one atomic subtract to zero and the ended step
# ([RC-4] point 2), `Stats`'s drop runs there, `deallocate` does nothing, and `level` ends after the scope, which is
# after the compiler saw the last owner end ([ALC-5], [LT-4]). Written `stats = thread_shared Stats() using System`,
# or `using pool` for a `struct MsgPool: inner: Mutex[Slab]` whose `allocate` and `deallocate` lock inside and read
# `this`, it compiles the same way. `thread_shared [0, 0] using level` (a value that grows) is E7015 by the same rule.

**runtime_cost**: For a `thread_shared T using A` value with A not `Global`: every +1 and -1 is the atomic away path from its making line (one `fetch_add` and the limit test, or one atomic subtract and the ended step), chosen by the type at the line, so no test of the handed-over mark and no read of a chunk header; one `allocate` at its making and one `deallocate` on the thread that ends it, the manager's own work (an arena: a bump, nothing freed one by one); the handed-over mark is part of the header's first store, nothing. A `thread_shared` value with no `using` is unchanged: two counts, its chunk header, the mark test on its making thread. The runtime's own out-of-line routines (`ember_rt_object_release`, the hand-over walk, a reload's registration) test the mark before the chunk header: one load and one branch, already in the row, now ordered. §X.4 rows: "A `thread_shared` value's two counts" (edited above) and "A value made `using` a memory manager" (unchanged: "each allocation and free of the value is the manager's own work ... none for a value that does not grow in a manager that frees nothing one by one"). Against C++: `std::allocate_shared` over a `pmr` resource counts atomically from birth too, so the arena-made value pays what C++ pays, and the method-3 team's "one allocation and one free more per object" for the refusal is not paid.

**implementer_steps**: 1. Types: `using A` is accepted after `thread_shared T` in a type and on a making line (the 0.9.15 list); the E2114 arm for it is removed from the type checker and from [OBJ-4]'s check. 2. For every type implementing `Allocator`, compute "any thread may call": `ThreadShared(A)` and the summaries of its `deallocate` and `reallocate` do not change `this` ([EXC-15], [EFF-24]); across a package's edge read it from the interface file's summaries ([BLD-14] item 1); `Global`, `System` and every `static` manager have it ([STA-1]: `allocate` reads `this`). 3. For a value made `using` an instance decide what it holds, by [ALC-1] and [TYP-46a]: nothing (empty `deallocate` and the type does not grow, or `FoundByAddress` with any-thread-callable methods); a `ref A` (keeps the word or `FoundByAddress`, A any thread may call); else a `sharedref mut` link, which makes the value neither `Send` nor `ThreadShared`. 4. At `thread_shared X(...) using a` and at a type `thread_shared T using A`: if the value holds a `sharedref mut` link, E7015 with the message of [TYP-46a]; verdict 165's `ThreadShared(T)` check runs beside it. In a generic body the property comes from the bounds only (see remaining_doubt). 5. Code generation for a place whose type is `thread_shared T using A`, A not `Global`: emit the away-path inline fast paths of [RT-10] (`fetch_add` + limit test; atomic subtract + the ended compare-exchange), never the home path, never the chunk read; the making line calls `a.allocate(Layout(32 + size_of[T], max(align, 8)))`, writes the header once: type index, weak count 1, home count 0, away word = count 1 with the handed-over mark set, and the instance's address in the queue place where [ALC-1] says the value keeps one; the end runs `drop`, decrements the weak count atomically and, at zero, calls `deallocate` through the word in the queue place (or the `FoundByAddress` mask and load, or nothing for an empty `deallocate`) on the ending thread. 6. Runtime: in `ember_rt_object_release`, the [THR-2] hand-over walk driven by `TypeInfo`, the reload registration ([HR-12]) and the leak report, read the away word and test the handed-over mark before computing the chunk address; a marked value takes the away path and is never queued; `[HR-46]`'s rewrite of instance addresses reads a `thread_shared` value's word from the queue place. 7. `weakref thread_shared T using A` links carry the `using` in their type ([GRM-3a]: the target's own type) and open on the away count ([RC-5] point 2) with no mark test. 8. Tests: the accepted program prints done; the refused program is E7015 at line 23 with the message; `thread_shared [0, 0] using level` is E7015; `thread_shared Stats() using System` compiles and its counts are atomic from birth; break tests: drop the mark from the first store and call `ember_rt_object_release` from a C thread on an arena-made value (reads a making thread from arena bytes); make `Pool.deallocate` read `this` through a `Mutex` and see the refused program compile. 9. Record nothing new at the package edge: items 1 and 14 of [BLD-14] already carry the summaries and the empty-`deallocate`/`FoundByAddress` facts.

**departs_from_verdict**: None. Touches, without overriding: 101 (the maker is read from the memory chunk, fix 1: a value made `using` reads no maker at all, and the 32-byte header is kept), 107 (no word in a manager with an empty `deallocate` or `FoundByAddress`: kept; the word, where kept, goes in the header's queue place), 47 (an arena-made value carries a pointer to its arena: kept, in that place, 0 bytes beyond the header), 19 and 24 (`using` names the kind, the making line the instance: the list returns to the owner's shape with `thread_shared` in it), 164 (a `sharedref mut` link never goes to another thread: used as the model of a value whose instance's methods change `this`), 165 (refused where the value is made: E7015 at the making line). The method-3 team's edits (E2114 for every `using` naming a `thread_shared` header) are not taken; they were never an owner verdict.

**trade_offs**: Against answer A (E2114 everywhere): A needs no property and no ordering rule, but it makes `thread_shared` the one exception to "`using` follows every type whose value has heap memory of its own" (what K34 wants to state as a property), drops a feature C++ (`std::allocate_shared`) and Rust (`Arc::new_in`) have, and refuses the J5-05 program for a hazard (the delayed end) that counting the value away from birth removes; the owner's condition on the delegation (never the cheaper shape, never dropping a feature to get past a check) weighs against it. Speed: a value made `using` loses the home-count fast path (25% on a thread's own values, [COST-3]) and pays atomics from birth, as C++'s `shared_ptr` always does; the ordinary `thread_shared` value and every program without `using` are unchanged, and the generated code gains no test. `using System` loses the two-count path too; the alternative, the runtime laying out a second set of per-thread chunks from `System`, was not taken for a rare spelling. Safety: the tightened property refuses a `ThreadShared`-by-inference pool whose `deallocate` changes `this`, which the current text accepts with a data race; the fix for such a program is a `Mutex` inside the manager, one line. Readability: `using` means the same after every word; the refused programs' messages give the line to write. Consistency: the two answers in one text become one; the implementer has one place for the instance word and one order for the mark and the chunk.

**remaining_doubt**: needs_owner: safety does not decide between A (refuse every `using` after `thread_shared`) and this shape, since both are safe once the text is consistent; the recommendation is this shape, which the owner's standing condition on the delegation (the better shape, no feature dropped to pass a check) already points to, and the J5-05 program the gap sweep asked about then compiles. Open beside it: (1) in a generic body over `[A: needs (Allocator, ThreadShared)]` the bound does not say that A's `deallocate` reads `this`, and [ALC-2]'s declarations of `allocate`/`deallocate` name no receiver mode while [ALC-1] says `allocate` "changes the instance"; until K34 or the lead fixes `Allocator`'s receivers, a making line `thread_shared T using a` in such a body is E7015 unless the instance is checked per monomorphised instance ([TYP-16]); (2) the queue-place word is this ruling's addition; the lead may keep verdict 47's word beside the value instead, at 8 bytes more; (3) [OPT-1]'s frame placement of any value made `using` an instance (a program's own `allocate` is observable) is a general gap of P02, not of `thread_shared`; (4) `into_raw()` of a value made `using` an instance handed to C, and the instance's life, is the "other places" item of P02, untouched here; (5) whether the compiler can always see the type `thread_shared T using A` at a count line (a `dyn I` or `some I` place holding such an object) rests on [TYP-46]'s E2020 making the manager part of every place's type; if a place can hold values of two managers, the generated code must test the mark there, one load and one branch, which the current row already prices.

## Judgement 1

**winner**: 2

**scores**: [
 {
  "design": 7,
  "safety": 6,
  "speed": 9,
  "readability": 8,
  "consistency": 7,
  "generality": 7,
  "holes": "Breaks the data-race promise ([THR-13]: \"every place two threads can reach is one of these, and each is ordered or only read\") with a program it accepts: `struct Pool: slab: Array[u8]; free: Array[usize]` with `unsafe extend Pool implements Allocator` whose `deallocate` runs `this.free.push(owns this.slab.index_of(p))`; `pool = Pool(slab=[], free=[])`, `msg = thread_shared Msg() using pool`, `with scope = thread.scope(): scope.spawn(owns fn() owns msg: msg.hits.fetch_add(1))`, and in the parent, before the scope ends, `other = thread_shared Msg() using pool` (or `println(pool.free.len())`). `Pool` is inferred `ThreadShared` ([THR-9]: \"Structural types are `ThreadShared` when every component is\"), its `deallocate` is not empty and it is not `FoundByAddress`, so under design 1's gate (its [TYP-46a]: \"`E7015` ... when the value holds a `ref A` and `A` is not `ThreadShared`\") the line compiles; the task's end of the last owner runs `pool.deallocate`, which writes `pool.free` on the task's thread while the parent's `allocate` pops it; [THR-11] does not catch it because design 1 keeps the clause \"unless `A` is `ThreadShared`\" that exempts the end from the task's changes. The cause is modelling the instance word as a `ref A` whatever the methods do ([LT-4] edit: \"counts as a `ref A` for `Send` and `ThreadShared`\"), which contradicts [EXC-15] (\"A method that may change its object is not called through a name that only reads it\"): a `ref A` can only call methods that read `this`. Secondary: the instance word is \"stored after the header\" (8 bytes per value, where the never-used queue place is free); `using System` keeps 4 unused header bytes. No hole under separate compilation or hot reload beyond design 2's shared open points."
 },
 {
  "design": 9,
  "safety": 9,
  "speed": 9,
  "readability": 7,
  "consistency": 8,
  "generality": 9,
  "holes": "No accepted program found that breaks a promise. The Pool program above is refused (E7015 at the making line, `Pool.deallocate` changes `this`); a pool that locks inside (`inner: Mutex[Slab]`, methods read `this`) compiles and is ordered by its lock ([STA-1]: \"shared state that changes goes through an `Atomic`, `Mutex`, `RwLock` or `Once` in it\"). Weak points, not safety: (1) false refusal: its [THR-11] clause keeps \"a value's growth and end are changes of its instance\" for every value, so `level = Arena(1 << 20); stats = thread_shared Stats() using level; with scope: spawn(owns fn() owns stats: ...); more = Stats() using level` (the parent allocating from the arena before the join) is E7007 although the task's end calls nothing on `level` ([ALC-1]: a value in a manager that frees nothing one by one keeps no word, its `deallocate` \"does nothing\"); fixed in the merge. (2) Leaves [LT-4]'s flat \"that address is a kept `sharedref mut` link\" and [ALC-1]'s \"it holds a `ref A` to the instance for `Send` and `ThreadShared`\" unreconciled with its own property; fixed in the merge. (3) A generic body over `[A: needs (Allocator, ThreadShared)]` cannot know that `A.deallocate` reads `this` from the bound (open, K34/P01). (4) `ember_rt_object_release` from C on a `using`-made value depends on the mark-before-chunk order it adds to [RC-4]; the order is stated, so it holds."
 }
]

**grafts**: Design 2 (the property-worded shape: `thread_shared T using A` allowed, counted away from its making line, refused only where the value would hold a link to an instance that not every thread may call) wins: it is the only one of the two that closes the race a `ThreadShared`-by-inference pool lets through, it puts the instance word in the header's never-used queue place (0 bytes beyond verdict 101's 32), and it orders the mark before the chunk read in the runtime routines that see no type. Grafted from design 1: (a) the [LT-4] cross-reference at the place that names the link, corrected so it does not assert `ref A` flatly but points to [TYP-46a]'s property; (b) the [RC-4] clause \"where the place's type does not carry `using A`, the mark decides, as after any hand-over\", so the type-chosen path is stated together with its fallback; (c) the [COST-3] pricing of the trade (the home count's plain adds given up for the manager's allocation and no runtime free, nothing for a manager that frees nothing one by one) so the row prices the shape and not only the path; (d) design 1's open point (1) applied as a clause in [TYP-46]: a place with no `using` is a `using Global` place, so E2020 holds in both directions and every place's type names its values' manager, which is the premise of emitting the away path from the type. Added by the judge: the [THR-11] clause in [TYP-46a] applies where growth and end call the instance's methods (a value that holds nothing changes it not at all), removing design 2's false E7007; and [ALC-1]'s between-threads sentence restated to the property. Not taken from the method-3 team's shape A (E2114 for every `using` after `thread_shared`): it is safe but makes `thread_shared` the one owner word `using` cannot follow, forces a runtime chunk allocation and free on every shared value made in bulk, and refuses the gap sweep's program for a hazard ([RC-4] point 4's delayed end) that counting away from birth removes; the owner's condition on the delegation (never drop a feature to get past a check) weighs against it. Taken from the method-3 team: the observation that on a making line `using` after a struct value made shared names the block the word makes, as `owns Mesh(points=[0.0]) using current` does ([TYP-46]: \"The line that makes such a value names the instance after the value it makes\"); no edit needed, the analogy already holds.

- FINAL EDIT rule=registry row E2114 (§XVI table)
  - old: ; `thread_shared` before a value made `using m`, whose block would carry no making-thread record | `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[LT-48]`, `[OBJ-4]` |
  - new:  | `[TYP-45]`, `[TYP-46]`, `[ALC-1]`, `[LT-48]` |
  - why: After the [OBJ-4] edit no rule emits E2114 for `thread_shared … using m`; the row claims only what its rules say, and `using` after `thread_shared` is decided by [TYP-46a] (E7015): one code, one defining rule.

- FINAL EDIT rule=registry row E7015 (§XVI table)
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: `thread_shared T using A`, at the making line or at the type, where the value would hold a link to an instance of a manager that not every thread may call;
  - why: The row follows [TYP-46a]'s corrected condition: the test is on what the value holds and whether the instance's methods may run on any thread, not on `A`'s marker alone.

- FINAL EDIT rule=§H.52 change log, the Hardened_2 entry under the memory-safety holes
  - old: * A `thread_shared` value is never placed in a frame or a manager's instance (`[OPT-1]`,
  `[OBJ-4]`, `[ALC-1]`).
  - new: * A `thread_shared` value is never placed in a frame; one made `using` a manager is counted away
  from its making line, the path chosen by its type (`[OPT-1]`, `[OBJ-4]`, `[ALC-1]`, `[RC-4]`,
  `[TYP-46a]`).
  - why: The entry restated the refusing half this ruling removes; a cross-reference claims what its rules say. (The lead may instead leave the Hardened_2 entry and record the change in Hardened_3's own entry; one of the two.)

- FINAL EDIT rule=§IV marker table, row `Send`
  - old: the `ref A` a value made in a manager's instance holds, unless `A` is `ThreadShared`, `[TYP-46a]`)
  - new: the link to its instance a value made in a manager's instance holds, unless `A` is a manager any thread may call, `[TYP-46a]`)
  - why: The table repeated [TYP-46a]'s flat condition; it follows the corrected one.

- FINAL EDIT rule=§IV marker table, row `ThreadShared`
  - old: (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`)
  - new: (the link to its instance a value made in a manager's instance holds included: a `ref A` for a manager any thread may call, else a `sharedref mut` link, `[TYP-46a]`)
  - why: The held link is a `ref A` only when the methods it calls read `this`; otherwise it is [LT-4]'s `sharedref mut` link, which is not `ThreadShared` ([LT-46], verdict 164).

- FINAL EDIT rule=OPT-1
  - old: A `thread_shared` value is never placed in a frame, and never in a
  memory manager's instance: its making thread is read from its chunk (`[RC-4]`), which only the
  runtime allocator's blocks have.
  - new: A `thread_shared` value is never placed in a frame: one made with no `using` has its making
  thread read from its chunk (`[OBJ-1]`), which a frame has not, and one made `using` an instance
  stays in the block that instance gave, since the instance's `allocate` and `deallocate` are the
  program's own work (`[RC-4]` point 4).
  - why: "never in a memory manager's instance" contradicted [OBJ-1], [RC-4] point 4 and [ALC-5], which describe exactly such a value. The frame rule stays: a frame block has no chunk header, and skipping a program's own `allocate` would be observable.

- FINAL EDIT rule=OBJ-4
  - old: A `thread_shared` value's block always comes from the runtime allocator
  (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is
  `E2114` ("a `thread_shared` value's block carries its making thread's record, which only the
  runtime allocator keeps; remove `using m`: `names = thread_shared Names()`").
  - new: A `thread_shared` value's block comes from the runtime's chunks (`[OBJ-1]`) when it is made
  with no `using`, and from the instance named when it is made `using` one, by calling
  `m.allocate(…)` as for any value made in `m` (`[TYP-46a]` says when): the first has its making
  thread read from its chunk's header, the second keeps its instance word as `[ALC-1]` says, is
  counted away from the line that makes it and asks no thread which made it (`[RC-4]` point 4).
  - why: This was the defining sentence of the refusal, one of the two answers the merge left standing; its reason (only the runtime's chunks carry the record) is true and is why such a value is counted away from birth, not why it must be refused. The rule states the block's source, which is where the question was caused.

- FINAL EDIT rule=ALC-1 (the list)
  - old:   `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String`; never a
  `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`) names
  - new:   `shares`, `shares mut` or `thread_shared` value, a class object, an `Array`, `Map`, `Set` or
  `String`; a `thread_shared` value's block then comes from the instance and never from the
  runtime's chunks, so it is counted away from its making line, `[RC-4]` point 4, `[TYP-46a]`) names
  - why: [ALC-1]'s list contradicted [RC-4] point 4, [OBJ-1], [ALC-5] and [COST-3], which all describe a `thread_shared` value made `using` an instance; the list returns to the owner's shape (verdict 19: `using` after the owner words) with the deciding property beside it.

- FINAL EDIT rule=ALC-1 (the between-threads sentence)
  - old: Between threads a value made in an instance
  follows its manager: it holds a `ref A` to the instance for `Send` and `ThreadShared`, and its
  growth and end are changes of the instance for `[THR-11]` (`[TYP-46a]`).
  - new: Between threads a value made in an instance
  follows its manager: for `Send` and `ThreadShared` it holds a `ref A` to the instance when `A` is
  a manager any thread may call, else the `sharedref mut` link of `[LT-4]`, and its growth and end
  are changes of the instance for `[THR-11]` where they call its methods (`[TYP-46a]`).
  - why: The sentence stated the `ref A` flatly; a `ref A` can call only a method that reads its object ([EXC-15]), so the link is a `ref A` exactly when the manager's methods do. It now says what [TYP-46a] says.

- FINAL EDIT rule=TYP-46 (the list)
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It may follow an `owns`,
  `shares`, `shares mut` or `thread_shared` type (a `thread_shared T using A` is a `thread_shared T`
  whose block comes from an `A` and whose owners are counted away from its making line, `[RC-4]`
  point 4, allowed where `[TYP-46a]` says), a class type, or an `Array`, a `Map`, a `Set` or a
  `String` (`name: String using LevelArena = String("") using current`); after any other type it is
  `E2114`
  - why: The list is the type-level statement of the same property as [ALC-1] and must agree with it and with [TYP-46a], which already names `thread_shared T using A` as a type that exists.

- FINAL EDIT rule=TYP-46 (the E2020 sentence)
  - old: A value given to a `using A` place that was not made in an
  instance of `A` is `E2020`
  - new: A value given to a `using A` place that was not made in an
  instance of `A` (a place with no `using` is a `using Global` place, so a place's type always names
  its values' manager) is `E2020`
  - why: The sentence stated one direction; the other (a value made in an instance given to a plain place) follows once a plain place is read as a `using Global` place, which the rule's own "Without `using`, the memory comes from the built-in manager (`Global`)" already implies. The type-chosen counting path of [RC-4] point 4 rests on every place's type naming its values' manager, so the premise is stated where places are typed.

- FINAL EDIT rule=§X.4 [COST-3] row "A `thread_shared` value's two counts"
  - old: a value made `using` an instance counts every owner atomically, as if handed over at its making line (`[RC-4]` point 4);
  - new: a value made `using` any manager but `Global` counts every owner atomically from its making line, the path chosen from the type of the place at each line, so it pays no test of the mark, no read of a chunk and never the queue (`[RC-4]` point 4), and gives its block back through its manager's `deallocate` on the thread that ends it, nothing for a manager that frees nothing one by one (`[ALC-1]`); against a value made in `Global` it gives up the home count's plain adds (a thread's own values 25% faster, below) for the manager's allocation (a bump for an arena) and no runtime free, a trade the programmer makes per value by the word;
  - why: Every cost has a row: the row now names what runs for such a value (the away path only, decided when compiling, plus the manager's own free), says that nothing runs for it in a program that does not write the word, and prices the trade against a value made in `Global`; "an instance" is widened to every manager but `Global`, since a `System` or `static` manager's block has no chunk header either.

- FINAL EDIT rule=OBJ-1
  - old: never changes. A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new: never changes. A `thread_shared` value made `using` any manager but `Global` (`[ALC-1]`,
  `[TYP-46a]`; `System` and a one-instance manager a program declares included) lies in memory the
  runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it; it is never queued, so where it keeps its instance's
  address (`[ALC-1]`) that address stands in the header's queue place, and the value costs no word
  beyond the 32 bytes.
  - why: The chunk record exists only for blocks the runtime takes for `Global` (the sentence before); "an instance" left `using System` and a program's own `static` manager to the two-count path, which would read a making thread from memory that has none. The implementer needs one place for the instance word of verdicts 47 and 107: the 8-byte queue place is free for a value that is never queued.

- FINAL EDIT rule=TYP-46a (the Send/ThreadShared clause)
  - old: it is `Send`, and `ThreadShared`, only
  when `A` is `ThreadShared` (`[THR-8]`, `[THR-9]`), since those calls run on whichever thread holds
  or ends it.
  - new: it is `Send`, and `ThreadShared`, only
  when `A` is a manager any thread may call (below; `[THR-8]`, `[THR-9]`), since those calls run on
  whichever thread holds or ends it.
  - why: `ThreadShared` says several threads may hold and read a value at once ([THR-9]); it says nothing about a `deallocate` that changes `this`, which the ending thread would run while the owner thread allocates: a `struct Pool: slab: Array[u8]; free: Array[usize]` is inferred `ThreadShared` and the current text accepts the race.

- FINAL EDIT rule=TYP-46a (the E7015 sentence)
  - old: `thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making
  line ("`st` would end on whichever thread holds its last owner, and `current`, an `Arena`, is used
  by one thread; make it in the built-in manager, `st = thread_shared Stats()`, or in a manager any
  thread may call").
  - new: A manager **any thread may call** is a `ThreadShared` type whose `deallocate` and `reallocate`
  read `this` (`[EXC-15]`, a fact of its summary, `[BLD-14]` item 1; a `static` manager's do,
  `[ALC-1]`, and so do those of a type that locks inside, `inner: Mutex[Slab]`), since a method a
  `ref A` may call is one that reads its object; a value whose end or growth would call a method
  that changes its instance holds the `sharedref mut` link of `[LT-4]` instead, which is neither
  `Send` nor `ThreadShared` (`[LT-46]`, verdict 164), whatever the instance's type. So
  `thread_shared T using A` is `E7015` at the making line when the value made would hold a link to
  an instance that is not one any thread may call: a value that grows made in an `Arena`, or any
  value in a manager whose `deallocate` changes `this` ("`msg` would end on whichever thread holds
  its last owner, and that end gives its block back through `Pool.deallocate` (line 12), which
  changes `pool`; make it in the built-in manager, `msg = thread_shared Msg()`, or in a manager any
  thread may call, whose `deallocate` reads `this` and locks inside"); a place of such a type
  (`log: thread_shared Log using Pool`) is `E7015` at the type, since no value could fill it. A
  `thread_shared` value that holds nothing (one that does not grow, made in a manager that frees
  nothing one by one: `stats = thread_shared Stats() using level` with `level` an `Arena`,
  `[ARN-2]`) is `Send` and `ThreadShared` when `T` is, and is accepted: its end on any thread runs
  its `drop` and touches the arena not at all, every owner of it is counted in its away count from
  its making line (`[RC-4]` point 4), and the arena is neither reset nor ended while it lives
  (`[ALC-5]`, `[LT-4]`).
  - why: The flat test on `A` refused the gap sweep's program for a reason untrue of it (nothing in the value's end touches the arena: the rule's own earlier sentence says such a value "holds nothing") and accepted a pool whose `deallocate` changes `this` from two threads. Stating the property (what the value holds, and whether the instance's methods may run on any thread) decides every manager the same way, a program's own included, and takes no feature away; the message names the line and the lines to write.

- FINAL EDIT rule=TYP-46a (the THR-11 clause, first part)
  - old: For `[THR-11]` a value's growth and end are changes of its instance, so two
  scoped tasks growing values made in one instance are `E7007`
  - new: For `[THR-11]` a value's growth and end are changes of its instance where they call its
  methods (a value that holds nothing changes it not at all), so two scoped tasks growing values
  made in one instance are `E7007`
  - why: [ALC-5] counts as changes of the instance "the `Allocator` methods that value's growth and end call"; the end of a value that keeps no word in a manager whose `deallocate` does nothing calls none, so a parent allocating from the arena while a task holds such a value is not a conflict. Without this, the accepted shape is refused (E7007) as soon as the parent touches the arena before the join.

- FINAL EDIT rule=TYP-46a (the THR-11 clause, the exemption)
  - old: unless `A` is `ThreadShared`. For the exclusivity rules `[ALC-5]` stands:
  - new: unless `A` is a manager any thread may call, whose `reallocate` and `deallocate` read the
  instance. For the exclusivity rules `[ALC-5]` stands:
  - why: Two scoped tasks growing values in a `ThreadShared` bump manager whose `reallocate` changes `this` both change it; the exemption must name the property that makes the growth and the end reads of the instance.

- FINAL EDIT rule=RC-4 point 4
  - old: A `thread_shared` value made `using` an instance
     of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that
     makes it: its instance may be reset or end as soon as the compiler sees its last owner end
     (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss, so every owner of such
     a value is counted in its away count and its end is never delayed; a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: A `thread_shared` value made `using` any manager
     but `Global` (`System`, a `static` manager and every instance alike, `[ALC-1]`, `[TYP-46a]`) is
     marked handed over at the line that makes it, in the header's first store, before any other
     thread can reach it: its block lies in memory the runtime did not lay out, so no chunk header
     names its making thread (`[OBJ-1]`), and its instance may be reset or end as soon as the
     compiler sees its last owner end (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point
     would miss; so every owner of such a value is counted in its away count, its end is never
     delayed, and the thread that ends it gives the block back through the manager's `deallocate`
     there. Where the type of the place at the line says `using A` (`[TYP-46]`, `[BLD-16]`) the
     compiler emits the away path with no test of the mark and no read of a chunk; where it does
     not, the mark decides, as after any hand-over; the runtime's own routines, which see no type
     (`ember_rt_object_release`, the hand-over walk of `[THR-2]`, a reload's registration,
     `[HR-12]`), test the mark before anything reads a chunk header, so a block the runtime did not
     lay out is never asked for its making thread. A value made in `Global`, with or without `using
     Global` written, keeps the two counts, its block waits for the check point and nothing reuses
     it.
  - why: This is where the run-time behaviour is defined. "a manager that may have several instances" and "or `System`" left `System` and a `static` manager on the two-count path with no chunk to read; the order "mark before chunk" is what makes a block without a chunk header safe in the routines that cannot know the type; the type-chosen path, with the mark as the fallback where a place's type does not say, is what keeps the generated code at C speed with no new test.

- FINAL EDIT rule=LT-4
  - old: that address is a kept `sharedref mut`
  link to the instance (`[LT-46]`): the program may use the instance while values made in it live,
  - new: that address is a kept `sharedref mut`
  link to the instance (`[LT-46]`) for the exclusivity rules, and between threads what `[TYP-46a]`
  says (a `ref A` to a manager any thread may call, else this link, which never leaves its
  thread): the program may use the instance while values made in it live,
  - why: Verdict 164 makes a type holding a `sharedref mut` link neither `Send` nor `ThreadShared`; read alone, [LT-4] would make every `using` value one-thread and `thread_shared T using A` impossible for any `A`, while [TYP-46a] models the link as a `ref A` where the manager's methods read `this`. The rule that names the link now says which reading holds for which check.

**final_cost_row**: | A `thread_shared` value's two counts | guaranteed present | blocks made in `Global` come in chunks of the configured allocator, each chunk's header naming its thread, read with one load on the away path (`[OBJ-1]`); a value made `using` any manager but `Global` counts every owner atomically from its making line, the path chosen from the type of the place at each line, so it pays no test of the mark, no read of a chunk and never the queue (`[RC-4]` point 4), and gives its block back through its manager's `deallocate` on the thread that ends it, nothing for a manager that frees nothing one by one (`[ALC-1]`); against a value made in `Global` it gives up the home count's plain adds (a thread's own values 25% faster, below) for the manager's allocation (a bump for an arena) and no runtime free, a trade the programmer makes per value by the word; on the making thread, one load and one branch on the handed-over mark before each count operation and each opening of a `weakref thread_shared T` link (`[RC-4]` point 4); a 32-byte header, 16 bytes more than one atomic count's (`[OBJ-1]`); a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over. Measured against one atomic count (`tasks/96/redesign/measure-0.9.15/brc/REPORT.md`, 15 runs per cell): a thread's own values 25% faster on 1 to 8 threads, the camera program 89 ms against 544 ms; reading another thread's values 3% slower on 1 thread and 17% to 28% faster on 2 to 8; one thread handing copies to all 0.93x to 1.05x; a random mix 9% to 13% slower; a pipeline (made on one thread, ended on the next) 27% slower on 1 thread and the same on 2 to 8 |

**owner_verdicts_touched**: None overridden. Touched, each kept as written: 19 (`using` names a manager, after the owner words: `thread_shared` returns to the list, as in 0.9.15) and 24 (the place names the kind, the making line the instance: unchanged for `thread_shared`); 47 (an arena-made value carries a pointer to its arena: kept where [ALC-1] keeps it, placed in the header's queue place for a `thread_shared` value, so 0 bytes beyond the header; verdict 47 said \"accepted for now\" and asked the measuring phase to drop the pointer where it can, which this does without dropping it); 101 (two counts, fix 1 the maker read from the memory chunk, fix 2 \"a hand-over the compiler sees moves A into B at that moment\": a value made `using` is in fix 2's state from its making line, reads no maker, and the header stays 32 bytes; every value made in `Global` keeps both numbers); 107 (no word in a manager with an empty `deallocate` or `FoundByAddress`: used as written, which is what makes a one-thread `Arena` acceptable); 164 (a `sharedref mut` link never goes to another thread: used as the model of a value whose instance's methods change `this`); 165 (refused where the value is made: E7015 at the making line, and at a type no value could fill). The method-3 team's recommendation A (E2114 for every `using` after `thread_shared`) and the Hardened_2 lead's \"a value made in `Global` or `System` keeps the two counts\" (ruling N84) were not owner verdicts; the latter is corrected because [OBJ-1] lays out chunks for `Global` only.

**remaining_doubt**: needs_owner: safety does not decide between shape A (refuse every `using` after `thread_shared`, the method-3 team's answer) and shape B (allow, counted away from the making line, refused only where the value would hold a link to an instance not every thread may call), since both are safe once the text says one thing; the recommendation is B as merged above, which the owner's standing condition on the delegation (the better shape, no feature dropped to pass a check) already points to, and under which the gap sweep's own program (`stats = thread_shared Stats() using level`, `level` an `Arena`) compiles while a pool whose `deallocate` changes `this` is refused with the line to write. Between the two B designs safety does decide (design 2's property closes a race design 1 accepts). Open beside it: (1) a generic body over `[A: needs (Allocator, ThreadShared)]` cannot read from the bound that `A.deallocate` reads `this`, and [ALC-2] prints `allocate`/`deallocate` without a receiver mode while [ALC-1] says `allocate` \"changes the instance\"; until K34 or P01 fixes `Allocator`'s receivers (or adds a bound such as a marker for a manager any thread may call), `thread_shared T using a` in such a body is E7015 unless checked per monomorphised instance ([TYP-16]). (2) The queue-place word is design 2's addition; the lead may keep verdict 47's word beside the value instead, at 8 bytes more, with no other change. (3) A `thread_shared` value made in a local instance that holds nothing (an `Arena`) handed to an unscoped thread: the text relies on the region it carries ([LT-4] E3063, [ALC-5]'s \"one handed to an unscoped thread ... lives, for this rule, as long as the instance\") and on [THR-8]'s rule for a value holding a `ref A`; the lead should confirm that a spawn's capture counts as \"kept where it would outlive the instance\" for a region-bearing value with no view in it, a gap of P02, not of `thread_shared`. (4) Hot reload: [HR-46] migrates a `using` value in place and refuses a move; it covers a `thread_shared` value made `using` with no further text, provided [HR-46]'s rewrite of the instance address reads a `thread_shared` value's word from the queue place (an implementer note, not a rule). (5) The trade (a chunk allocation and biased counts against an arena bump and atomic counts) is priced from the brc report and verdict 107's measurements, not measured as one program; the method-3 team's E1-4 measurement stands. (6) `using System`, a `static` manager and a program's own one-instance manager lose the two-count path; a second set of per-thread chunks from `System` was not taken for a rare spelling, and the row says so by \"any manager but `Global`\".
