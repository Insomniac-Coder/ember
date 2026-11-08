# R5-parts-XI-XVII

could_not_cover: Nothing in the eight clusters was left unread. The exact words of the new messages are the lead's to settle (JOB.md, "Messages and tests: the lead's to settle"); the edits give the property and the line to write, and copy verdict 164's and 165's own messages where they exist. I did not read docs/HANDOFF.md §0.355 beyond its role in CLAUDE.md, since this task edits nothing and runs nothing.

## K01-sharedref-send  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: THR-8: "A type is **`Send`** when a value of it may be handed over to another thread: nothing reachable from it (its fields, its elements, what its `owns` parts hold, a function value's state) is a one-thread owner (`shares T`, `shares mut T`), a `weakref T` link (one to a value on one thread, `[WK-11]`), a `sharedref mut` link (a view several holders on one thread may change, `[LT-46]`), or bound to the thread that made it"; THR-9: "`ref mut T`, `MutSpan[T]`, `sharedref mut T`, `Cell`, `UnsafeCell`, raw pointers, a `Retained[ForeignCallback]` token (`[FFI-23]`), `shares T`, `shares mut T`, and a `weakref T` link are not"; LT-46: "it never reaches another thread (it is neither `Send` nor `ThreadShared`, `[THR-8]`), and the scoped tasks of its thread change its target only under a lock, through `Atomic`, or by a reduction (§XI.3, `[THR-13]`)"; THR-15: "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`), so a one-thread owner never crosses through a channel: an end of a channel of such a `T` given to another thread is `E7015` at that line, as any value that is not `Send` (`[THR-2]`)"; THR-1: "everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`"; THR-13: "a `sharedref mut` target: a scoped task changes it directly only where neither another task nor the parent may touch it before the join, or by a reduction (`E7007`); an unscoped thread never holds one (`E7015`)"; marker table §IV.8, row Send: "nothing reachable from the value is a one-thread owner (`shares`, `shares mut`), a `weakref T` link, a `sharedref mut` link (a view several holders on one thread may change)"

**program**: 

- READER EDIT rule=THR-15
  - old: `Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]`
  is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`), so a one-thread owner never crosses
  through a channel: an end of a channel of such a `T` given to another thread is `E7015` at that
  line, as any value that is not `Send` (`[THR-2]`).
  - new: A channel carries its values to another thread, so `T` MUST be `Send` (`[THR-8]`): `channel[T]` or `channel_unbounded[T]` for a `T` that is not is `E7015` at the line that makes the channel, naming what keeps `T` on one thread (verdict 164: "`Watcher` holds `target`, a `sharedref mut Player` link, which other names on this thread may change, so no channel carries a `Watcher`; let a scoped task use `w` where it is (`scope.spawn(owns fn(): ...)`) instead of sending it"); so `Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, for every channel, and a one-thread owner never crosses through one (`[THR-2]`).
  - why: Verdict 164 shows `channel[Watcher]` refused at the channel line with `E7015`; the current text refuses only the end's crossing (and the `[T: needs Send]` bound, `E2040`). A channel of a `T` no thread may receive has no use, and refusing it where it is written is the verdict's letter and the more readable message.

- READER EDIT rule=registry row E7015
  - old: an end of a channel of a `T` that is not `Send` given to another thread;
  - new: a channel of a `T` that is not `Send`, at the line that makes it;
  - why: The row follows the rule it cites (`[THR-15]`).

**departs_from_verdict**: 164 in its letter only: the verdict shows `channel[Watcher](capacity=1)` refused (`E7015`); the current text accepts that line and refuses the end's hand-over (`[THR-15]`) and the `Send` bound at `hand_off(owns w, tx)` (`E2040`, `[TYP-43]`). The property the verdict states (a `sharedref mut` link never goes to another thread) is met: `[THR-8]`, `[THR-9]`, `[LT-46]`, the marker table and `[THR-1]`'s second bullet all list the link.

**remaining_doubt**: The finder's program is refused today by two routes (the `[T: needs Send]` bound, and `[THR-15]` if an end were given to a thread), and no route reaches a race, so the lead may judge verdict 164 satisfied by the property and skip the edit; N80 (decisions.md §13) chose 'Send exactly when T is Send' without the verdict in hand. `[THR-1]`'s second bullet lists examples (owner, `weakref T`, plain `fn()`) and not the `sharedref mut` link, but its clause 'everything reachable from it is `Send`' decides it through `[THR-8]`.

**REFUTER reason**: The property is met in the current text: `[THR-8]` "nothing reachable from it ... is ... a `sharedref mut` link (a view several holders on one thread may change, `[LT-46]`)", `[LT-46]` "it never reaches another thread (it is neither `Send` nor `ThreadShared`, `[THR-8]`)", and `[THR-15]` "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`) ... an end of a channel of such a `T` given to another thread is `E7015` at that line". The finder's `hand_off(owns w, tx)` is refused by the `[T: needs Send]` bound (`E2040`) and an end crossing a thread by `[THR-15]`. But verdict 164 reads "`channel[Watcher]` ... is refused", and the owner answered "yes a" to an option whose code line was `tx, rx = channel[Watcher](capacity=1)   # refused`; the current text accepts that line, so the text departs from the verdict's letter (N80 in decisions.md §13 is a lead ruling and cannot override it). The reader's edit restores the letter at the cause: a channel exists to carry values between threads, and a channel of a `T` no thread may receive has no use the text names. It is also safer than the current wording, which refuses only an end "given to another thread": a scoped task uses the parent's `rx` with no hand-over (`[THR-11]` "it uses the caller's real variables"), so today a `Watcher` can travel to a task through a channel whose ends never move, and only `[THR-13]`'s last bullet ("a scoped task changes it directly only where neither another task nor the parent may touch it before the join") stands between the received link and a race, with no rule saying how a write through a link taken out of a channel is judged. Refusing the channel at its making line closes that route with no analysis. No cost is added (a refusal), no other rule contradicts it (`[THR-9]`'s "a channel's `Sender[T]` for a `Send` `T`" stays true, every channel's `T` now being `Send`), and the registry row follows. The message's words are the lead's (JOB.md, "Messages and tests").

## K02-thread-shared-not-threadshared  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: THR-1: "everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`"; THR-1: "its state changes only through an `Atomic`, a `Mutex`, an `RwLock`, a channel end, or a field `guarded by` a lock, under that lock (`[THR-18]`)"; THR-8: "a `thread_shared T`, and a `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`; `dyn I` when `I` requires it (`interface I: needs Send:`). Of the function types, `thread_shared fn()` is `Send` (`[THR-1]`)"; THR-9: "`RwLock[T]` and `thread_shared T` for a `T` that is `Send` and `ThreadShared` (several threads hold a `ref T` at once: `RwLock[Cell[int]]` is not `ThreadShared`)"; CELL-3: "`Cell[T]` is not `ThreadShared`; it is `Send` if `T` is."; THR-10: "so a function that holds a one-thread value is `E7015` (`[THR-1]`, `[THR-2]`), and `R` MUST be `Send` (`E7015`, `E7008`)"; THR-2: "A `shares` or `shares mut` owner counts its owners with plain adds, and every object that is not `thread_shared` counts its `weakref` links with plain adds. So no such owner, and no `weakref T` link, reaches another thread by any route"; registry row E7015: "a `thread_shared` object of a class pinned to one thread or with a `weakref T` field; `thread_shared T using A` for an `A` that is not `ThreadShared`"

**program**: import std.thread
from std.sync import RwLock

class Cache:
    hits: RwLock[Cell[int]] = RwLock(Cell(0))

fn bump(c: thread_shared Cache):
    with g = c.hits.read():
        g.set(g.get() + 1)

fn main():
    c = thread_shared Cache()                               # accepted: THR-1 asks only E7014 and 'everything reachable is Send'; Cache is Send (CELL-3)
    h = thread.spawn(owns fn() thread_shared c: bump(c))    # accepted: the function is a `thread_shared fn()`, which THR-8 makes Send with no condition, and it holds a `thread_shared` owner, which THR-1's third bullet allows; a `thread_shared Cache` is not a 'one-thread value' of THR-2, so THR-10's E7015 does not name it
    bump(c)
    h.join()
    # two threads hold read guards of one RwLock at once and both `set` the Cell through them: a data race in Safe Ember, against [PHIL-10]; E7014 is met to the letter ('through an RwLock')

- READER EDIT rule=THR-1
  - old:   * everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a
    plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`
  - new:   * everything reachable from it is `Send` and `ThreadShared` (`[THR-8]`, `[THR-9]`), since several threads hold and read the value at once: a one-thread owner, a `weakref T` link, a plain `fn()` value, or a part that is not safe for several threads to read at once (a `Cell`, an `RwLock` of one, a raw pointer, `[THR-9]`) reachable from it is `E7015` at the line that makes it `thread_shared`, naming the part and the line to write (verdict 165: "`FrameStats.draws` is an `RwLock` of a `Cell`, and threads holding its read lock at once would change the `Cell` together; write `draws: Atomic[int] = Atomic(0)` in `FrameStats` (with `from std.sync import Atomic` at the top)"); a `guarded by` field counts as the `Mutex` it is (`[THR-18]`), `ThreadShared` when its type is `Send`
  - why: Verdict 165: `thread_shared` is refused where the value is made when something in it is not `ThreadShared`. The current bullet asks for `Send` alone, and `[THR-8]`'s unconditional `Send` for `thread_shared fn()` then carries a non-`ThreadShared` `thread_shared` owner to another thread; checked at the making line, every `thread_shared T` has a `ThreadShared` `T`, which is also what `[THR-8]`'s and `[THR-9]`'s conditions on `thread_shared T` already assume.

- READER EDIT rule=THR-1
  - old:   * **`thread_shared fn()`** is a function any thread may run. It holds only `thread_shared` owners,
    values handed over to it and copies, never a one-thread owner or a `weakref T` link (`E7015`:
  - new:   * **`thread_shared fn()`** is a function any thread may run. It holds only `thread_shared` owners, values handed over to it and copies, never a one-thread owner or a `weakref T` link; and where the type is not a `once fn`, several threads may run one value of it at once, so what it holds is `ThreadShared` too (`[THR-9]`; a `copy of` a `Cell` in a function given to `handlers: Array[thread_shared fn()]` is `E7015` where it is given, `[CLO-16]`) (`E7015`:
  - why: The same cause one step on: a function value stored where any thread may run it is read by several threads at once, so `Send` alone leaves a captured `Cell` racing; a `once fn` runs once on one thread and needs only `Send`.

- READER EDIT rule=registry row E7015
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: a `thread_shared` value of a type that is not `ThreadShared`, at the line that makes it, and a function given to a `thread_shared fn()` type (not a `once fn`) whose state is not `ThreadShared`; `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - why: The row lists the rule's new case (`[THR-1]`).

**departs_from_verdict**: 165: the verdict refuses `stats = thread_shared FrameStats()` at that line (`E7015`, naming the field and `draws: Atomic[int]`); `[THR-1]`'s checks at that line are `E7014` and 'everything reachable from it is `Send`', and no rule asks for `ThreadShared` there, so the program above compiles and races.

**remaining_doubt**: `[THR-9]` does not list function types among what is `ThreadShared`; the second edit states the fact where the type is defined (`[THR-1]`) and `[THR-9]` could repeat it in one clause. The message's fix (`Atomic[int]`) is the verdict's; its words are the lead's to settle (JOB.md, 'Messages and tests').

**REFUTER reason**: The hole is real. `[THR-1]`'s checks at the making line are `E7014` and "everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a plain `fn()` value reachable from it is `E7015`"; `[CELL-3]` "`Cell[T]` is not `ThreadShared`; it is `Send` if `T` is", so `c = thread_shared Cache()` with `hits: RwLock[Cell[int]]` compiles, against verdict 165 ("`stats = thread_shared FrameStats()` with `draws: RwLock[Cell[int]]` is `E7015` at that line"). Whether the spawn is then refused is undecidable: `[THR-8]` "Of the function types, `thread_shared fn()` is `Send` (`[THR-1]`)" with no condition and `[THR-1]` "It holds only `thread_shared` owners, values handed over to it and copies" accept it, while `[THR-8]`'s "a `thread_shared T` ... when `T` is `Send` and `ThreadShared`" and `[FFI-22]`'s "every value it holds is `Send`" (said for callbacks) point the other way; `[THR-10]`'s "a function that holds a one-thread value is `E7015`" names `[THR-2]`'s owners and links, not a `thread_shared` owner of a non-`ThreadShared` type. Two threads then hold read guards of one `RwLock` (`[THR-9]` "several threads hold a `ref T` at once: `RwLock[Cell[int]]` is not `ThreadShared`") and `set` the `Cell`: a data race, against `[PHIL-10]`. The reader's fix is at the cause and follows the verdict. Narrowed for three things: (1) the proposed bullet's parenthetical "(a `Cell`, an `RwLock` of one, a raw pointer, `[THR-9]`)" reads as if a `Cell` inside a `Mutex` were refused, where `[THR-9]` makes "`Mutex[T]` for a `Send` `T`" `ThreadShared` whatever `T` holds; the markers are judged by components and the bullet should say so; (2) the proposed text requires every part to be `ThreadShared`, and `[THR-9]` does not list `thread_shared fn()` among what is `ThreadShared`, so `[THR-1]`'s own fix line `handlers: Array[thread_shared fn()] = []` in a `thread_shared Bus` would be refused by the new bullet read literally; `[THR-9]` must say it (the reader saw this in its doubt and left it); (3) the same requirement closes a second hole the reader did not name: `[THR-1]`'s first bullet lets a `thread_shared` object's state change through "a channel end", and a `Receiver` is `Send` (`[THR-15]`) but "one consumer" (`[THR-11]`) and not `ThreadShared` (`[THR-20]`), so today two threads may `recv()` through one `thread_shared` object's `Receiver` field; with `ThreadShared` required at the making line the field refuses the object, and the bullet should name a `Sender`, not "a channel end". The reader's second edit (a `thread_shared fn()`'s state is `ThreadShared` unless the type is a `once fn`) and the registry row stand.

- BETTER EDIT rule=THR-1
  - old:   * everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a
    plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`
    ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any
    thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`");
  - new:   * its type is `Send` and `ThreadShared` (`[THR-8]`, `[THR-9]`, each judged by the type's components as the marker is: a `Mutex[T]` of a `Send` `T` is `ThreadShared` whatever `T` holds), since owners on several threads hold and read the value at once: a one-thread owner, a `weakref T` link, a plain `fn()` value, or a part that is not `ThreadShared` (a `Cell`, or an `RwLock` of one, outside a `Mutex`; a raw pointer; a `Receiver`; `[THR-9]`) reachable from it is `E7015` at the line that makes it `thread_shared`, naming the part and the line to write ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`"; verdict 165: "`FrameStats.draws` is an `RwLock` of a `Cell`, and threads holding its read lock at once would change the `Cell` together; write `draws: Atomic[int] = Atomic(0)` in `FrameStats` (with `from std.sync import Atomic` at the top)"); a `guarded by` field is the `Mutex` it stands for (`[THR-18]`), `ThreadShared` when its type is `Send`;
  - why: Verdict 165 refuses `thread_shared` at the making line when the type is not `ThreadShared`; stated on the type and its components, the rule cannot be read as refusing a `Cell` behind a `Mutex`, and it also refuses a `Receiver` field, which two threads would otherwise consume through one object.

- BETTER EDIT rule=THR-1
  - old: its state changes only through an `Atomic`, a `Mutex`, an `RwLock`, a channel
    end, or a field `guarded by` a lock, under that lock (`[THR-18]`);
  - new: its state changes only through an `Atomic`, a `Mutex`, an `RwLock`, a `Sender` (a `Receiver` is one consumer's, `[THR-11]`, and not `ThreadShared`, `[THR-20]`), or a field `guarded by` a lock, under that lock (`[THR-18]`);
  - why: "a channel end" admitted a `Receiver`, which several owners on several threads would take values out of at once, against `[THR-15]`'s one-consumer queue.

- BETTER EDIT rule=THR-1
  - old:   * **`thread_shared fn()`** is a function any thread may run. It holds only `thread_shared` owners,
    values handed over to it and copies, never a one-thread owner or a `weakref T` link (`E7015`:
  - new:   * **`thread_shared fn()`** is a function any thread may run. It holds only `thread_shared` owners, values handed over to it and copies, never a one-thread owner or a `weakref T` link; and where the type is not a `once fn`, several threads may run one value of it at once, so its state is `ThreadShared` too (`[THR-9]`; a `copy of` a `Cell` in a function given to `handlers: Array[thread_shared fn()]` is `E7015` where it is given, `[CLO-16]`), while a `thread_shared once fn()` is handed over and run once (`[THR-10]`) and needs only `Send` state (`E7015`:
  - why: The reader's second edit, kept: a stored function any thread may run is read by several threads at once; the `once fn` case is stated so `thread.spawn`'s function needs no more than today.

- BETTER EDIT rule=THR-9
  - old: a `weakref
  thread_shared T` for such a `T`, and a channel's `Sender[T]` for a `Send` `T` are (`[THR-15]`).
  - new: a `weakref thread_shared T` for such a `T`, a channel's `Sender[T]` for a `Send` `T` (`[THR-15]`), and `thread_shared fn()`, whose state is `ThreadShared` (`[THR-1]`), are; a `thread_shared once fn()` is `Send` and not `ThreadShared`, since it is handed over and run once (`[CLO-6]`, `[THR-10]`).
  - why: With `ThreadShared` required of a `thread_shared` value's type, `[THR-1]`'s own fix line `handlers: Array[thread_shared fn()] = []` compiles only if `[THR-9]` lists the function type; the reader left this to a doubt.

- BETTER EDIT rule=registry row E7015
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: a `thread_shared` value of a type that is not `ThreadShared`, at the line that makes it, and a function given to a `thread_shared fn()` type (not a `once fn`) whose state is not `ThreadShared`; `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - why: The reader's row edit, kept: the row lists the rule's new case.

## K03-task-ends-shares-owners  reader=answered  refuter=narrowed  should_be=answered  safety=none

**answered_by**: THR-11: "**A task touches no one-thread count.** Making or ending a `shares` or `shares mut` owner, making, ending or opening a `weakref T` link, and ending an object whose class family carries a weak count (`[OBJ-1]`), each write a header with plain instructions (`[RC-4]`, `[WK-11]`), which the parent may write at the same moment. So a task does each of them only for a `thread_shared` target (`[THR-2]`), worked out from its body and every function it may run (`[EFF-24]`, `[DRP-8]`): a task that may end a `shares` owner, or may end an object of a class family that carries a weak count, is `E7015` at the spawn ("the task started on line 11 clears `a`, which holds `shares Player` owners, whose count is kept with plain adds on one thread; clear it after the join: move `a.clear()` below the `with scope` block"; "the task started on line 18 may end the `Player` that `w` links (`q = Player()`), while this thread may test `w`; give `q` its new value after the `with scope` block")"; THR-11: "A value a task or a job takes with `owns` is handed to another thread: `[THR-2]`'s compare and move happen at the spawn, so every link on this thread then finds it ended, and the task ends it with no header this thread reads."; JOB-2: "`jobs.scope()` follows `[THR-5]` and `[THR-11]`: `s.submit` on a `JobScope` takes a plain function (`owns f: once fn()`), whose jobs may use the caller's own variables, and the block's exit joins them."; registry row E7015: "a scoped task or job that may end a `shares` owner or an object of a class family that carries a weak count"; THR-2: "**Making, ending or opening a `weakref` link touches its target's header** (its weak count, and the test that it still lives): in a scoped task, a job, or any function that may run on another thread than its target's owner, it is allowed only through a `weakref thread_shared T`"

**program**: 

**remaining_doubt**: Verdict 166 is met in substance and in its message: the finder's `scope.spawn(owns fn(): level.enemies.clear())` is `E7015` at the spawn ('a task that may end a `shares` owner ... is `E7015` at the spawn'), the fix moves the clear below the block, and the second message covers an object `weakref` links point at. One wording gap, not a hole: the `E7015` registry row says 'may end a `shares` owner' while the rule refuses making one too ('a task does each of them only for a `thread_shared` target'); the row could read 'may make or end'. The parent's `hud_target = shares mut boss` needs no rule of its own since the task is refused first.

**REFUTER reason**: Answered as the reader says. `[THR-11]`: "**A task touches no one-thread count.** Making or ending a `shares` or `shares mut` owner, making, ending or opening a `weakref T` link, and ending an object whose class family carries a weak count (`[OBJ-1]`), each write a header with plain instructions ... So a task does each of them only for a `thread_shared` target (`[THR-2]`) ... a task that may end a `shares` owner, or may end an object of a class family that carries a weak count, is `E7015` at the spawn", with the message that moves `a.clear()` below the `with scope` block, which is verdict 166's letter and fix; `[JOB-2]` "`jobs.scope()` follows `[THR-5]` and `[THR-11]`" carries it to jobs, and `[PAR-2]` "what it uses from outside follows the rules of a scoped task (`[THR-11]`, `[THR-2]`)" to parallel bodies. The parent's `hud_target = shares mut boss` needs no rule since the spawn is refused first. The one thing to apply is the consistency item the reader left in its doubt: the `E7015` row reads "a scoped task or job that may end a `shares` owner or an object of a class family that carries a weak count" while the rule refuses making one too ("a task does each of them only for a `thread_shared` target"), and the brief asks that every cross-reference claim what its rule says.

- BETTER EDIT rule=registry row E7015
  - old: a scoped task or job that may end a `shares` owner or an object of a class family that carries a weak count;
  - new: a scoped task or job that may make or end a `shares` owner, make, end or open a `weakref T` link, or end an object of a class family that carries a weak count;
  - why: The row follows `[THR-11]`, which refuses each plain-count write a task may make, not only the end of a `shares` owner.

## K10-queued-drop-check-points  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_proposed_text

**answered_by**: RC-4 point 3: "**Check points.** The compiler places them: at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends"; RC-4 point 3: "A value whose making thread has ended is ended at once by the thread that ends its last owner: a thread's end empties and closes its queue in one atomic step"; RC-4 point 5: "**So a value's end is delayed only** when its last owner ends on a thread that did not make it and the value was not handed over at a place of point 4 (it travelled inside another object): it is ended at its making thread's next check point."; WK-3: "A `thread_shared` value whose last owner ends on a thread that did not make it, and that was not handed over, is ended at its making thread's next check point (`[RC-4]` point 5), and its `drop` runs there."; THR-19: "**A wait for a lock this thread holds is refused when compiling.**"; DRP-8: "A `drop` may change things outside its object (a log line, leaving a list)."; BRW-18: "and the end of an object whose `drop` changes it, `[DRP-8]`). It is `E3020`, at the change"; OWN-2: "A value always ends with its clean-up: nothing ends an owner without running it"; HR-3a: "Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`), `thread.spawn` threads, and job workers."; BLD-16: "not by which of its values are made `thread_shared`, not by which of its classes a `weakref` names"

**program**: Two programs the current text accepts. (1) J7-01, a deadlock the text promises to refuse, and with `free` unguarded a push into a list this thread walks (`[BRW-18]`, a use after free):

import std.thread
from std.sync import Lock

class Pool:
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:
    id: int
    pool: thread_shared Pool
    fn drop():
        with this.pool.lock:
            this.pool.free.push(owns this.id)

class Batch:
    conns: Array[thread_shared Conn] = []

fn main():
    pool = thread_shared Pool()
    b = Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)])
    h = thread.spawn(owns fn() owns b: b.conns.len())   # the Conn travels inside b: no hand-over (RC-4 point 5), so its end is queued to main
    with pool.lock:
        for id in pool.free:          # RC-4 point 3 puts a check point at this back edge; the queued Conn's drop runs here and waits for pool.lock, which this thread holds; no rule refuses it (THR-19 lists calls, joins, statics, a second read())
            println(id)
    h.join()

(2) J7-02, a clean-up that never runs (`[OWN-2]`):

import std.jobs

class Asset:
    file: File
    fn drop():
        this.file.close()

class Bundle:
    assets: Array[thread_shared Asset] = []

fn load_all(paths: Array[String]) -> Array[Bundle]:
    out: Array[Bundle] = []
    with s = jobs.scope():
        for p in paths:
            s.submit(owns fn() ref p: out.push(owns Bundle(assets=[thread_shared Asset(file=File.open(p))])))
    return out

fn main():
    bundles = load_all([String("a.bin"), String("b.bin")])
    bundles.clear()      # each Asset's last owner ends on main; each is queued to the worker that made it (point 2); a worker waiting for work in the runtime reaches no check point of point 3 and does not end until the process does, where THR-16 runs no drop: the files are never closed
    # the same for a value made on a host thread (FFI-22) that has returned to C

- READER EDIT rule=RC-4 point 3
  - old:   3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line.
  - new:   3. **Check points.** The compiler places them where nothing of this thread's is open for a queued value's `drop` to clash with: a check point stands only where this thread holds no lock (outside every `with` block of a lock, and after a guard's last use, `[THR-19]`'s extent) and walks no `static mut` and holds no view into one (`[BRW-18]`, `[LT-3]`), since a `drop` may take a lock or change a module variable (`[DRP-8]`) and the types a thread's queue may hold are not known where a line is compiled (`[BLD-16]`). Within that: at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends (for `main`, before its return completes, `[THR-16]`); the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line. A loop that runs wholly inside a lock's block has no check point of its own: its queued values wait for the check point after the block, so the `drop` of a value this thread made never waits for a lock this thread holds and never changes what this thread walks. A thread that runs no compiled code of the program still empties its queue: a job worker (§XI.5) empties it before each wait for work, and a value put in the queue of a waiting worker wakes it to do so; a host thread attached by `[FFI-22]` may return to C and never run Ember code again, so a value made on one is marked handed over at its making line (point 4), as a value made in a manager's instance is, and nothing is ever queued to it. So every queued value is ended within one check point, one job or one callback of its last owner's end.
  - why: The placement was a lead choice (decisions.md §6), not a verdict; verdict 101 asks only for 'its next check point ... always before the thread ends'. Placing the points where the thread holds nothing keeps `[THR-19]`'s and `[BRW-18]`'s promises with nothing checked while running and refuses no program (the finder's option B, bounded as the critic asked); a worker's queue test sits where it already stops, and a host thread's values pay the atomics of a handed-over value instead of waiting for a thread that may never return (the finder's A and B, each where its cost is smallest).

- READER EDIT rule=§X.4 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point, and on a job worker before each wait for work (`[RC-4]` point 3), and the queue's work for a value whose last owner ends on another thread without a hand-over; every owner of a value made on a host thread counted atomically, as for a value made in a manager's instance (point 4).
  - why: Every cost has a row (BRIEF): the worker's test and the host thread's atomic counts are new costs.

**cost_row**: §X.4 row "A `thread_shared` value's two counts": a queue test on a job worker before each wait for work, and atomic counts for every owner of a value made on a host thread (edit above).

**remaining_doubt**: The alternative for host threads is to empty the queue when the outermost Ember call returns to C and at detach; it leaves a value queued after that return waiting for the thread's next call, which may never come, so the hand-over at making was chosen. Whether a long loop that holds a lock for the life of the program may delay queued ends indefinitely is a resource matter (`[PHIL-11]`), not a safety one; the end is only delayed, never lost. `[WK-3]`'s last sentence and `[THR-20]`'s 'A `drop` runs where the last owner ends' agree with the edit and need no change.

**REFUTER reason**: Both programs are accepted and break promises. (1) `[RC-4]` point 3 "The compiler places them: at every loop's back edge", point 5 "it is ended at its making thread's next check point", `[WK-3]` "its `drop` runs there": `Conn`'s `drop` takes `pool.lock` inside `main`'s `with pool.lock:` loop, which the head of `[THR-19]` promises away ("**A wait for a lock this thread holds is refused when compiling.** ... with no loop needed and nothing checked while the program runs") and none of its listed shapes catches (a call through a function value, a join, a direct call, a `static` initialiser, a second `read()`); `[PHIL-11]` excuses deadlock in general but not one the text promises to refuse. The reader is right that the memory-safety routes are closed today (`[THR-1]` `E7014` for a direct push into a `thread_shared` object, `[THR-20]` `E7012` for an unguarded `static mut` a `drop` a function on another thread may run changes), so the hole is the promise of `[THR-19]`. (2) `[RC-4]` point 3 places check points only in compiled code and "when the thread ends"; a job worker waiting for work and a host thread back in C (`[FFI-22]`, `[HR-3a]`) reach none, and `[THR-16]` runs no `drop` at the process's end, so the `Asset`'s `close` never runs, against `[OWN-2]` "A value always ends with its clean-up" and point 5's own promise. Confirmed as not answered. The reader's fix is narrowed because its placement rule is lexical ("outside every `with` block of a lock, and after a guard's last use") while the property it claims ("where this thread holds no lock") is not: a function with a loop called from inside `with pool.lock:` has a back edge the compiler places a check point at, since the callee cannot see its caller's lock (`[BLD-16]`: no fact outside the package's own text decides its code), and the queued `Conn`'s `drop` waits there for the lock this thread holds; the same for a blocking call in such a callee. So the proposed text reopens J7-01 one call down, or forces an implementer to add a thread-local lock-depth test the cost table has no row for. The better fix moves the decision to the type, where the declaring package can see it and `[BLD-16]` allows it: a `thread_shared` value whose `drop` (or the `drop` of anything it owns) may take a lock, join a thread or end a scope, the things `[THR-19]` refuses under a held lock, is marked handed over at its making line, as a value made `using` an instance already is (`[RC-4]` point 4, N84), so every owner of it is counted atomically and "**A `drop` runs where the last owner ends**" (`[THR-20]`) is literally true for it: the `drop` runs on the thread that ends the last owner, where `[THR-19]`, `[THR-11]` and `[DRP-8]` judge it as the call it is, and the queue holds only values whose `drop` waits for nothing, so a check point anywhere, under a lock or in a callee, waits for nothing. This refuses no program, adds nothing checked while running, keeps biased counting for every type whose `drop` only frees, closes files or touches atomics, and costs atomic counts for the lock-taking types alone (a row below). It also removes the contradiction that sank the earlier design (its arbiter's E3-1, a scoped task's last pending destructors against the scope's end): nothing lock-taking is ever deferred to a scope's end. The reader's two J7-02 provisions are kept (a worker empties its queue before each wait for work and is woken by a value put in it; a value made on a host thread is marked handed over, which the runtime does when it gives out the block, since it knows the thread from its registration and the compiler emits nothing new), as is "for `main`, before its return ends the process". A `thread.spawn` thread blocked for a long time in a receive or a join delays the lock-free values queued to it (new-gaps NG-11, NG-47); that is the delay verdict 101 itself states ("at its next check point") and `[PHIL-11]`'s resource matter, never a lost clean-up, so no edit.

- BETTER EDIT rule=RC-4 point 3
  - old:      a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end
     makes a value a joined task ended on its own thread end before the parent's next line.
  - new:      a channel's receive, a sleep) and when the thread ends (for `main`, before its return ends the process, `[THR-16]`); the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line. A queued value's `drop` takes no lock, joins no thread and ends no scope (point 4 keeps every value whose `drop` may out of the queue), so a check point inside a lock's `with` block, under a guard or in a function called there waits for nothing, and `[THR-19]`'s refusals stay complete with nothing checked while the program runs; what such a `drop` changes is judged as the `drop` of any value a function on another thread may end (`[THR-20]`, `[DRP-8]`). A job worker (§XI.5) empties its queue before each wait for work, and a value put in the queue of a waiting worker wakes it to do so. A thread attached by `[FFI-22]` may return to C and never run Ember code again, so nothing is queued to it: a value made on it is marked handed over when the runtime gives out its block (point 4).
  - why: Point 3 is where the hidden ends are placed; saying what the queue may hold lets every placement stand, in callees too, and gives the worker and host thread cases, which have no check point of their own.

- BETTER EDIT rule=RC-4 point 4
  - old: a value made in `Global` or
     `System` keeps the two counts, its block waits for the check point and nothing reuses it.
  - new: a value made in `Global` or `System` keeps the two counts, its block waits for the check point and nothing reuses it. A `thread_shared` value of a type whose `drop`, or the `drop` of anything it owns, may take a lock, join a thread or end a `with scope` block, worked out from the type's own code and the functions it may run (`[EFF-24]`; a call through a function value, a `dyn` or `virtual` method, or a foreign call not declared `@ffi(no_callbacks)` counts as one, as `[THR-19]` counts them; the fact is read from the type's declaration, `[BLD-16]`), is marked handed over at the line that makes it in the same way: every owner is counted in its away count, its `drop` runs where its last owner ends, on that thread (`[THR-20]`), where `[THR-19]` and `[THR-11]` judge it as the call it is (`[DRP-8]`), and never at a check point of a thread that holds a lock it needs. So is a value made on a thread attached by `[FFI-22]`, marked by the runtime when it gives out the block, since such a thread may never run Ember code again.
  - why: The decision is made where the declaring package can see it (the type's `drop`), not where the loop happens to be, so no caller's lock is missed and no program is refused; the cost falls on the lock-taking types alone.

- BETTER EDIT rule=§X.4 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point, and on a job worker before each wait for work (`[RC-4]` point 3), and the queue's work for a value whose last owner ends on another thread without a hand-over; every owner counted atomically, as for a value made `using` an instance, for a value whose type's `drop` may take a lock, join a thread or end a scope, and for a value made on a thread attached by `[FFI-22]` (point 4).
  - why: Every cost has a row: the worker's test and the atomic counts of the two marked kinds are new.

## K36-unsafe-extend-vouching  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: TYP-43: "A program writes `Send` by hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend T implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler cannot check"; TYP-43: "`ThreadShared` is written only in bounds, `Copy` comes from `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`, as is an `unsafe extend` of any interface but `Send` or an `unsafe interface` (ODR-111)."; THR-8: "the author of a type that holds a raw pointer declares it with `unsafe extend T implements Send:` and a `# SAFETY:` note"; THR-9: "raw pointers, a `Retained[ForeignCallback]` token (`[FFI-23]`), `shares T`, `shares mut T`, and a `weakref T` link are not; an `extern type`, an opaque C++ class or a `@ffi(handle)` type is only where the program declares it (`[FFI-8]`)"; FFI-8: "each is `Send` unless its contract says `threads(creator)` or `threads(main)`, so one thread at a time uses an object of it, and `ThreadShared` only where the program declares it, `unsafe extend Ctx implements ThreadShared:` with a `# SAFETY:` note saying that the library reads one object from several threads at once (as `Send` is declared for a type holding a raw pointer, `[THR-8]`; `E3100` without `unsafe`), an asserted fact `ember tcb` lists"; FFI-8: "A `handle` is an opaque pointer-sized `Copy` value with no destructor of its own"; registry row E3100: "`Send` declared by hand without `unsafe extend`; ... `ThreadShared` declared for an `extern type` without `unsafe`"; STA-1: "`T` MUST be `ThreadShared` (`E7002`)"

**program**: J2-12, which two rules decide two ways (an implementer cannot choose):

class GpuTexture:
    handle: *mut u8
    size: int

# SAFETY: the driver's texture handles may be used from any thread
unsafe extend GpuTexture implements Send:    # TYP-43: only 'a struct or enum of its own package' -> refused; THR-8: 'the author of a type that holds a raw pointer declares it' -> accepted
    pass

fn main():
    tex = GpuTexture(handle=null[*mut u8](), size=0)
    h = thread.spawn(owns fn() owns tex: tex.size)
    println(h.join())

J7-06 is decided, another way than the finder expected: the vouching for `ThreadShared` exists, on the foreign type itself (`[FFI-8]`: an `extern type`, an opaque C++ class or a `@ffi(handle)` type), not on a program's struct holding a raw pointer, which `[THR-9]` keeps off `ThreadShared`; a wrapper several threads read holds a `@ffi(handle)` type vouched `ThreadShared` and is then `ThreadShared` structurally, usable as a `static` and read by scoped tasks through `ref`. But `[TYP-43]`'s sentence 'an `unsafe extend` of any interface but `Send` or an `unsafe interface` is `E2040`' contradicts `[FFI-8]` and the `E3100` row, so `unsafe extend Ctx implements ThreadShared:` is refused by one rule and required by another.

- READER EDIT rule=TYP-43
  - old: A program writes `Send` by
  hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend T
  implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler
  cannot check
  - new: A program writes `Send` by hand, for a struct, enum or class of its own package that holds a raw pointer (the vouching is about the pointer, not the kind of type holding it, `[THR-8]`), only as `unsafe extend T implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler cannot check; the declaration vouches for the type's raw pointers alone, and every other component is inferred as for any type, so a `shares` owner or a `weakref T` link in such a type still keeps it on one thread (`E7015`)
  - why: `[TYP-43]` says 'struct or enum' and `[THR-8]` 'a type', with no reason for leaving a class out (a `shares` owner of the class stays one-thread whatever the class's fields are); and no rule said whether the vouching covers the type's other components, which a `# SAFETY:` note cannot make safe.

- READER EDIT rule=TYP-43
  - old: each of these written by hand is `E2040`, as is an
  `unsafe extend` of any interface but `Send` or an `unsafe interface` (ODR-111).
  - new: each of these written by hand is `E2040`, as is an `unsafe extend` of any interface but `Send`, an `unsafe interface`, or `ThreadShared` on an `extern type`, an opaque C++ class or a `@ffi(handle)` type, whose contents the compiler cannot see (`[FFI-8]`; `E3100` without `unsafe`) (ODR-111). A struct holding a raw pointer is never `ThreadShared` (`[THR-9]`): a wrapper several threads read at once holds the foreign object as a `@ffi(handle)` type vouched `ThreadShared`, not as a raw pointer.
  - why: `[FFI-8]` and the `E3100` row allow and require the declaration that this sentence refuses; the second sentence answers J7-06's 'how does a program let several threads read such a value' in the rule that refuses the other spelling.

**remaining_doubt**: The finder's option A (vouch `ThreadShared` on any struct holding a raw pointer) is not taken: `[FFI-8]` puts the asserted fact on the foreign type whose library documentation gives it, which `ember tcb` lists once; a struct route through `@ffi(handle)` reaches the same programs. If the lead prefers A, the `E2040` sentence is the one place to widen. Whether a `static LOGGER: CLogger` can be given a C handle at compile time is a `[STA-3]` question outside this cluster.

**REFUTER reason**: Both contradictions are in the text. `[TYP-43]`: "A program writes `Send` by hand, for a struct or enum of its own package that holds a raw pointer" against `[THR-8]`: "the author of a type that holds a raw pointer declares it with `unsafe extend T implements Send:`" and "class objects are `Send` when every component is", so J2-12's class is refused by one rule and accepted by the other. `[TYP-43]`: "each of these written by hand is `E2040`, as is an `unsafe extend` of any interface but `Send` or an `unsafe interface` (ODR-111)" against `[FFI-8]`: "`ThreadShared` only where the program declares it, `unsafe extend Ctx implements ThreadShared:` with a `# SAFETY:` note ... (`E3100` without `unsafe`)" and the `E3100` row "`ThreadShared` declared for an `extern type` without `unsafe`", so the declaration `[FFI-8]` requires is refused by `[TYP-43]`. The reader's edits are at the cause (`[TYP-43]`, the one rule on hand-written markers) and open no hole: the vouching covers the raw pointers alone and every other component is inferred, so a `shares` owner in the class still keeps it on one thread; `ThreadShared` stays off a struct holding a raw pointer (`[THR-9]` "raw pointers ... are not") and goes on the foreign type `[FFI-8]` already names, which `ember tcb` lists once, so J7-06's wrapper holds a `@ffi(handle)` type vouched `ThreadShared` and is `ThreadShared` by its components; nothing runs while the program runs; no verdict touches the question. The finder's option A (vouching `ThreadShared` on any struct) is a lead choice the reader rightly leaves open. One hole in these rules the reader did not raise is listed under missed (a `threads(creator)` type vouched `ThreadShared`).

## K37-parallel-array-elements  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: PAR-2: "The loop is accepted only if (a) each has the form `i + k` for a constant `k`; (b) all of them share **one** `k`; and (c) `P` is the elements of a `MutSpan` or an `SoA` column the loop uses, not a place reached through a link (`shares`, `shares mut`, `sharedref mut`, `thread_shared`), a `Cell` or a raw pointer."; PAR-1: "A `@parallel` loop splits a range (`for i in 0..n:`) or a list walked by element (`for ref mut p in players:`) into chunks"; SPN-1: "An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]` or `StrView` where one is expected, and to `MutSpan[T]` where one is expected (a `ref mut` parameter, a `ref mut` name, or a `MutSpan`-typed field made with `ref mut`, `[BRW-12]`)."; PAR-2: "where an `Atomic`, `chunks_mut` or `@parallel(reduce=…)` makes the loop compile, the message gives that line."

**program**: The implementer's question, with the line that cannot be decided:

fn brighten(ref mut pixels: Array[float], gain: float):
    @parallel
    for i in 0..pixels.len():
        pixels[i] *= gain        # P = pixels[i] is written at index i + 0 (clauses (a) and (b) hold); clause (c) names 'the elements of a MutSpan or an SoA column', and `pixels` is an Array: SPN-1 converts it only where a MutSpan is expected, which indexing in a body does not, so (c) neither accepts nor names it. Accept, or E7010 whose message gives `px = ref mut pixels.as_mut_span()`? No rule says.

- READER EDIT rule=PAR-2
  - old: (c) `P` is the elements of a `MutSpan` or an `SoA` column the
  loop uses, not a place reached through a link (`shares`, `shares mut`, `sharedref mut`,
  `thread_shared`), a `Cell` or a raw pointer.
  - new: (c) `P` is the elements of a contiguous list the loop reaches by a name that may change it, an `Array[T]`, a `[T; N]`, a `MutSpan[T]` or an `SoA` column (`fn brighten(ref mut pixels: Array[float], gain: float):` with `pixels[i] *= gain` compiles as the `MutSpan` form does), which the body neither grows nor shrinks (a `push`, `pop`, `insert`, `remove` or `clear` of the list in the body is a write of the list itself, reached through no index, which (a) refuses), not a place reached through a link (`shares`, `shares mut`, `sharedref mut`, `thread_shared`), a `Cell` or a raw pointer.
  - why: The two loops are equally safe: distinct chunks write distinct elements of one list that cannot change shape during the loop, which is the property clause (c) protects; refusing the `Array` form would make the programmer write a conversion the compiler can see is the same (BRIEF: Python-like, no line written only for the compiler; a rule states the property, not a shape).

**remaining_doubt**: Safety does not decide between the finder's A and B; the owner's readability rules (JOB.md 'What Python-like means'; AUTOPILOT: the better shape, never the cheaper one) point to A, and the lead holds the delegation for it. An `Array` reached through a `shares mut` owner stays refused by the link clause.

**REFUTER reason**: The line cannot be decided from the text. `[PAR-2]` (c): "`P` is the elements of a `MutSpan` or an `SoA` column the loop uses"; `[SPN-1]`: an `Array[T]` converts to `MutSpan[T]` only "where one is expected (a `ref mut` parameter, a `ref mut` name, or a `MutSpan`-typed field made with `ref mut`, `[BRW-12]`)", and `pixels[i]` in a body expects none, so `fn brighten(ref mut pixels: Array[float], gain: float)` with `pixels[i] *= gain` is neither accepted by (c) nor given an `E7010` message (the rule's message sentence names "an `Atomic`, `chunks_mut` or `@parallel(reduce=…)`", not `as_mut_span()`), while `[PAR-1]` accepts the by-element walk of the same `Array`. Confirmed. The reader's edit is safe: the property (c) protects is that distinct chunks write distinct elements of one buffer that cannot change shape during the loop, and a `push`, `pop`, `insert`, `remove` or `clear` is a write of `P` through no index, which (a) refuses; a list reached through a link stays refused by the link clause; `[THR-20]` keeps a `static mut` list under its own rules. Narrowed for one gap the edit opens: `[PAR-2b]` says "A call it cannot see into that receives a captured `MutSpan` or column defeats the proof", and with an `Array` or `[T; N]` admitted by (c) a call the analysis cannot see into that receives the list must defeat the proof the same way, or a callee that reshapes it escapes the check; the rule must name the list too.

- BETTER EDIT rule=PAR-2
  - old: (c) `P` is the elements of a `MutSpan` or an `SoA` column the
  loop uses, not a place reached through a link (`shares`, `shares mut`, `sharedref mut`,
  `thread_shared`), a `Cell` or a raw pointer.
  - new: (c) `P` is the elements of a contiguous list the loop reaches by a name that may change it, an `Array[T]`, a `[T; N]`, a `MutSpan[T]` or an `SoA` column (`fn brighten(ref mut pixels: Array[float], gain: float):` with `pixels[i] *= gain` compiles as the `MutSpan` form does), which the body neither grows nor shrinks (a `push`, `pop`, `insert`, `remove` or `clear` of the list in the body is a write of the list itself, reached through no index, which (a) refuses), not a place reached through a link (`shares`, `shares mut`, `sharedref mut`, `thread_shared`), a `Cell` or a raw pointer.
  - why: The reader's edit, kept: the rule states the property (one buffer of fixed shape, distinct elements per chunk), not the shape of the name.

- BETTER EDIT rule=PAR-2b
  - old: A call it cannot see into that receives a captured `MutSpan`
  or column defeats the proof for everything reachable through that argument (`E7010`) (ODR-062).
  - new: A call it cannot see into that receives a captured list, `MutSpan` or column, by `ref mut` or by a name that may change it, defeats the proof for everything reachable through that argument (`E7010`) (ODR-062).
  - why: With an `Array` admitted by clause (c), a call the analysis cannot read through must defeat the proof for it as for a `MutSpan`, or a callee that reshapes the list is not seen.

## K38-parallel-reductions  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: THR-21: "A task's statement of one of these forms on a place `P` that the task otherwise neither reads nor writes, `P op= e` for `+`, `-`, `*`, `|`, `&`, `^`, `P = max(P, e)` and `P = min(P, e)`, `P.push(owns e)`, `P.extend(owns e)` and `P.add(owns e)`, and `P[k] op= e` on a `Map`, where `P` is a local, a `static mut` (`[THR-20]`), or a field of an object the tasks reach"; THR-21: "A plain assignment from a task (`a.target.hp = 5`) and a task that reads the current value (`if stats.total < 10: stats.total += x`) need a lock (`E7007`)."; PAR-2: "what it uses from outside follows the rules of a scoped task (`[THR-11]`, `[THR-2]`). In a loop over a range, for each place `P` written in the body the compiler collects every index expression through which `P` is accessed in the body, **reads included**. The loop is accepted only if (a) each has the form `i + k` for a constant `k`"; PAR-2: "A loop walked by element hands each chunk its own elements through the loop's `ref mut` name, and writes nothing else but declared reductions (`[PAR-3]`)"; PAR-2: "where an `Atomic`, `chunks_mut` or `@parallel(reduce=…)` makes the loop compile, the message gives that line."; PAR-3: "`@parallel(reduce=[total: +, best: max])` declares variables combined after the loop; inside the body each chunk has its own copy. Each chunk reduces its iterations left to right, and the chunk results are then combined in chunk order, so the result is the same on every machine (`[PAR-5]`); for floats it can differ from the serial left-to-right sum, which is why a float reduction must be declared."; SIMD-5: "and every floating-point reduction is declared by `@parallel(reduce=…)` or the function is `@fastmath`"; THR-11: "two of them changing one place, or one reading a place another changes, is `E7007`, other than by a reduction (`[THR-21]`), through the lock that guards the place (a `guarded by` field inside `with` its lock, `[THR-18]`; a `Mutex`'s or `RwLock`'s content through its guard, `[THR-3]`; a `with` block of a lock that does not guard the place is no exception), through an `Atomic`, or through a `Sender` by its own operations (`[THR-15]`)"

**program**: 

- READER EDIT rule=PAR-2
  - old: and what it uses from outside follows the
  rules of a scoped task (`[THR-11]`, `[THR-2]`). In a loop over a range, for each place `P` written in
  the body the compiler collects
  - new: and what it uses from outside follows the rules of a scoped task (`[THR-11]`, `[THR-2]`), except that a total the iterations combine is declared (`[PAR-3]`): a task's undeclared `total += x` is a reduction, and a loop body's is `E7010`, whose message gives the `reduce=` line. In a loop over a range, for each place `P` written in the body, other than a declared reduction variable, an `Atomic`, a `Sender`, or a `guarded by` field inside `with` its lock (each changed by its own operations, `[THR-11]`'s exceptions), the compiler collects
  - why: The clauses (a) to (c) read literally refuse an `Atomic` (`hits.fetch_add(1)`: a written place with no index), which the rule's own message sentence says makes the loop compile, and the cross-reference to `[THR-11]` gives an undeclared total two answers (a reduction there, `E7010` here); the edit states the exceptions once and says which answer holds.

- READER EDIT rule=PAR-3
  - old: declares variables combined
  after the loop; inside the body each chunk has its own copy.
  - new: declares the variables the loop's iterations combine, whole numbers and lists included, unlike a scoped task's (`[THR-21]`): a `@parallel` body runs once per iteration, and the declaration is what `[SIMD-5]` reads for a float total. Inside the body each chunk has its own accumulator, which starts and is combined as a task's does (`[THR-21]`).
  - why: 'a float reduction must be declared' left the whole-number case implied; `[PAR-2]`'s by-element sentence and clause (c) already decide it (declared), so the rule that defines the declaration says so, and 'its own copy' is replaced by the accumulator `[THR-21]` defines (K39).

- READER EDIT rule=PAR-3
  - old: for floats it can differ from the serial left-to-right sum, which is
  why a float reduction must be declared.
  - new: for floats it can differ from the serial left-to-right sum, which is one reason every reduction is declared.
  - why: Follows the first edit.

**remaining_doubt**: J7-09 is answered: `[THR-21]` now lists `P = max(P, e)` and `P = min(P, e)` as the written forms (H.53: 'the written forms of a reduction'), so `best = max(best, t.len())` in a task compiles and is not the refused plain assignment. J7-10 is decided the other way from the finder's A: the text keeps declared reductions in loops (`[PAR-2]`'s 'writes nothing else but declared reductions', clause (c), `[PAR-3]`, `[SIMD-5]`), which is safe; the finder's A (infer loop totals as tasks do and drop `reduce=`) would need `[SIMD-5]`'s clause rewritten and is the lead's to weigh against 'no line written only for the compiler'. Verdicts 77 and 92 decided tasks only.

**REFUTER reason**: The reader's reading of the text is right and its status holds. J7-09 is answered: `[THR-21]` lists "`P = max(P, e)` and `P = min(P, e)`" among the written forms "on a place `P` that the task otherwise neither reads nor writes", so `best = max(best, t.len())` is the reduction and not the refused "plain assignment from a task (`a.target.hp = 5`)". J7-10 is decided for declared reductions in loops by `[PAR-2]` "writes nothing else but declared reductions (`[PAR-3]`)", `[PAR-3]`, and `[SIMD-5]` "every floating-point reduction is declared by `@parallel(reduce=…)`", which is safe (a declaration refuses, never races), and the finder's option A would rewrite `[SIMD-5]`; verdicts 77 and 92 decided tasks only. The letter is inconsistent as the reader says: in a range loop clause (c) "`P` is the elements of a `MutSpan` or an `SoA` column" refuses an `Atomic` (`hits.fetch_add(1)`, a written place with no index) that the same rule's "where an `Atomic`, `chunks_mut` or `@parallel(reduce=…)` makes the loop compile" admits, and "follows the rules of a scoped task (`[THR-11]`, `[THR-2]`)" hands an undeclared `total += x` to `[THR-21]`, which makes it a reduction. The edits say the exceptions once. Narrowed for two gaps: (1) the exception list omits what `[THR-11]` also excepts, "a `Mutex`'s or `RwLock`'s content through its guard, `[THR-3]`", so `with g = m.lock(): g.push(owns x)` in a body would be refused by (c) and allowed by the task rules the body follows; and the by-element sentence "writes nothing else but declared reductions" is left refusing the same `Atomic` in a by-element body; (2) `[PAR-3]`'s new "whole numbers and lists included" promises a `push` reducer the `reduce=` syntax nowhere shows; "whole numbers included" says what the rule decides.

- BETTER EDIT rule=PAR-2
  - old: and what it uses from outside follows the
  rules of a scoped task (`[THR-11]`, `[THR-2]`). In a loop over a range, for each place `P` written in
  the body the compiler collects
  - new: and what it uses from outside follows the rules of a scoped task (`[THR-11]`, `[THR-2]`), except that a total the iterations combine is declared (`[PAR-3]`): a task's undeclared `total += x` is a reduction, and a loop body's is `E7010`, whose message gives the `reduce=` line. In a loop over a range, for each place `P` written in the body, other than a declared reduction variable or a place `[THR-11]` excepts, changed by its own operations (an `Atomic`, a `Sender`, a `Mutex`'s or `RwLock`'s content through its guard, a `guarded by` field inside `with` its lock), the compiler collects
  - why: The reader's edit with `[THR-11]`'s exceptions complete, so the body's lock-guarded changes are not refused by a clause meant for indexed elements.

- BETTER EDIT rule=PAR-2
  - old: through the loop's `ref mut` name, and writes nothing else but declared reductions (`[PAR-3]`); its
  - new: through the loop's `ref mut` name, and writes nothing else but declared reductions (`[PAR-3]`) and the places `[THR-11]` excepts, each by its own operations; its
  - why: The by-element sentence carried the same contradiction with the message sentence for an `Atomic` in the body.

- BETTER EDIT rule=PAR-3
  - old: declares variables combined
  after the loop; inside the body each chunk has its own copy.
  - new: declares the variables the loop's iterations combine, whole numbers included, unlike a scoped task's (`[THR-21]`): the declaration is what `[SIMD-5]` reads for a float total. Inside the body each chunk has its own accumulator, which starts and is combined as a task's does (`[THR-21]`).
  - why: The reader's edit without "lists", which the `reduce=` syntax does not show; "its own copy" is replaced by the accumulator `[THR-21]` defines (K39).

- BETTER EDIT rule=PAR-3
  - old: for floats it can differ from the serial left-to-right sum, which is
  why a float reduction must be declared.
  - new: for floats it can differ from the serial left-to-right sum, which is one reason every reduction is declared.
  - why: The reader's edit, kept: follows the first.

## K39-reduction-start-values  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: THR-21: "is a reduction: each task adds into a private accumulator, and the accumulators are combined into the place at the join, in spawn order."; THR-21: "Each task's accumulator has the place's type, and its operations are checked as the place's are, and so are the combining operations at the join (verdict 105): a whole-number total is the one adding one by one gives whenever no task's own total overflows (`a.target.hp -= 1` and `b.target.hp -= 2` from two tasks, `a` and `b` linking one player whose `hp` is 10, leave 7 every time), and a task whose own total overflows stops the program there (`[TYP-8]`)"; PAR-3: "inside the body each chunk has its own copy. Each chunk reduces its iterations left to right, and the chunk results are then combined in chunk order"; TYP-8: "An arithmetic operation (`+ - * // % **`, unary `-`, and the compound assignments) whose mathematical result does not fit its type panics with `integer overflow in '<op>'`"; §X.4 row "Scoped tasks and their reductions": "one private accumulator per task and one combine per task at the join, as C's per-thread counter; no atomic per change (`[THR-21]`)"

**program**: The implementer's question, with the line that cannot be decided:

import std.thread

class Player:
    hp: u8 = 10

fn main():
    p = Player()
    with scope = thread.scope():
        scope.spawn(owns fn(): p.hp -= 1)    # THR-21: a reduction; 'each task adds into a private accumulator' of 'the place's type' (u8), 'checked as the place's are'. From what value does it start, and what does `-= 1` do to it? Starting at 0 and applying `-= 1` is 0 - 1, which TYP-8 stops; yet the rule promises 7 for this very example ('leave 7 every time'). Nothing says the accumulator gathers 1 and the join subtracts it.
        scope.spawn(owns fn(): p.hp -= 2)
    println(p.hp)

The same for `*=` (start 1 or 0?), `&=`, `max`/`min` (start from what?), and `[PAR-3]`'s 'own copy' (a copy of the variable's value would count it once per chunk).

- READER EDIT rule=THR-21
  - old: is
  a reduction: each task adds into a private accumulator, and the accumulators are combined into the
  place at the join, in spawn order.
  - new: is a reduction: each task gathers its changes in a private accumulator, which starts from the operator's neutral value (0 for `+`, `-`, `|` and `^`, 1 for `*`, every bit set for `&`, an empty collection for `push`, `extend` and `add`, an empty `Map` for `P[k] op= e`); for `-=` the accumulator gathers what the task takes away (`acc += e`) and the join takes it from the place (`P -= acc`), so an unsigned place never goes below zero in a task whose one-by-one result would not (`hp: u8 = 10` with `-= 1` and `-= 2` from two tasks leaves 7, and a task taking away more than 255 in all stops there); for `P = max(P, e)` and `P = min(P, e)` the accumulator holds the first `e` the task gives and then the larger (smaller) of itself and each later `e`, and a task that gave none leaves the place as it is at the join; the accumulators are combined into the place at the join, in spawn order, each by the place's own operator. All the tasks' statements on one place use one operator (`+=` and `-=` counting as one): `+=` in one task and `*=` in another is `E7007`, whose message gives the lock, since the joined result would depend on which task's statement is applied first.
  - why: The rule promised the one-by-one total (verdict 105: 'equal when no task's total overflows') but never said what a total starts from or how `-=` is gathered, so a `u8` place stopped the program on the rule's own example; the neutral start and 'gather what is taken away' keep the promise for every integer type and operator with the same one accumulator and one combine per task (§X.4 row unchanged). One operator per place keeps the joined result the one-by-one one; a place given two operators has no one-by-one result to promise.

**remaining_doubt**: `[PAR-3]`'s 'own copy' is covered by K38's second edit (an accumulator that starts and is combined as a task's). Refusing two operators on one place is a design choice safety does not force (the text's spawn-order join already gives a deterministic result); the lead may keep the join in spawn order and drop that sentence, but then `(P + a) * b` is a result no program meant. For a float `max` with a NaN, the first-value start avoids a start of `-inf`; `[DET-2]`'s 'same on every run' holds since each task's `e` sequence is its own.

**REFUTER reason**: Confirmed as undecidable. `[THR-21]`: "each task adds into a private accumulator, and the accumulators are combined into the place at the join, in spawn order ... Each task's accumulator has the place's type, and its operations are checked as the place's are, and so are the combining operations at the join (verdict 105)" and the example "`a.target.hp -= 1` and `b.target.hp -= 2` from two tasks ... leave 7 every time"; nothing says what the accumulator starts from, nor which operation the join applies (for `-`, an accumulator of `0 - 1 - 2` joined by `P -= acc` gives 13, by `P += acc` 7), nor what one task's `P += 1; P *= 2` gathers into one accumulator, nor what a `max` with no statement run leaves; with `hp: u8` the natural reading (start 0, `-=`) stops the program under `[TYP-8]` where the rule promises 7. The reader's fix keeps verdict 105 (the task's own total, in the place's type, stops on overflow) and verdict 77's example. Narrowed for three things: (1) "`+=` and `-=` counting as one" does not work with "the accumulator gathers what the task takes away" on an unsigned place (`P -= 3; P += 5` on a `u8` would gather `0 - 3` and stop where one by one gives 12), so the two must count as one operator only on a signed place, where the accumulator holds the signed change from 0; (2) for `max`, `min`, `&` and `|` a flag for "gave none" is not needed: these are idempotent, so the accumulator starts from the place's value at the spawn, which nothing changes before the join (`[THR-21]` "Nothing reads the place before the join", `[THR-11]`), and a task whose statement never runs leaves the place as it is; this also settles a float `max` with a `NaN` without a `-inf` start and keeps `[DET-2]`; (3) the Map sentence "inserting a key absent from `P` with the accumulator's value" gives the wrong sign once `-=` gathers what is taken away, and must say what applying the accumulator to the type's zero gives. One operator per place (with the lock as the fix) is kept: it is the only way one accumulator can hold a task's statements, and a place given two operators has no one-by-one result the rule could promise.

- BETTER EDIT rule=THR-21
  - old: is
  a reduction: each task adds into a private accumulator, and the accumulators are combined into the
  place at the join, in spawn order.
  - new: is a reduction: each task gathers its changes in a private accumulator, and the accumulators are combined into the place at the join, in spawn order, each by the place's own operator. The accumulator of `+`, `*` and `^` starts from the operator's neutral value (0, 1, 0), that of `push`, `extend` and `add` empty, that of `P[k] op= e` an empty `Map`; for `-=` it gathers what the task takes away (`acc += e`) and the join takes it from the place (`P -= acc`), so an unsigned place never goes below zero in a task whose one-by-one result would not (`hp: u8 = 10` with `-= 1` and `-= 2` from two tasks leaves 7, and a task taking away more than 255 in all stops there); on a signed place `+=` and `-=` are one operator, the accumulator holding the signed change from 0; for `max`, `min`, `&` and `|`, which applied twice give what applied once gives, the accumulator starts from the place's value at the spawn, which nothing changes before the join, so a task whose statement never runs leaves the place as it is. All the tasks' statements on one place use one operator: `+=` in one task and `*=` in another, or `+=` and `-=` on an unsigned place, is `E7007`, whose message gives the lock, since one accumulator holds one operator's changes and the joined result would otherwise depend on which task's statement is applied first.
  - why: The rule promised the one-by-one total (verdict 105) but never said what an accumulator starts from or how `-=` is gathered and joined, so a `u8` place stopped the program on the rule's own example; the neutral start, the gathered subtraction, and the idempotent start from the place keep the promise for every type and operator with one accumulator and one combine per task (§X.4 row unchanged).

- BETTER EDIT rule=THR-21
  - old: inserting a key absent from `P` with the accumulator's value.
  - new: inserting a key absent from `P` with what the operator applied to the value type's zero and the accumulator gives (the accumulator itself for `+`, `|` and `^`, `0 - acc[k]` for `-`).
  - why: Once `-=` gathers what is taken away, inserting the accumulator as it is would give the wrong sign for a key absent from the map.

# MISSED

## MISSED: A thread-bound foreign type vouched `ThreadShared` sends its view to another thread

**rules**: FFI-8, THR-8, THR-9, E3100 row

**program**:
```
# overlay: `@ffi(handle) type Ctx` with the contract `threads(creator)` (one thread uses an object of it, [FFI-8])
# SAFETY: the library reads one context from several threads at once   <- contradicts the contract, and nothing refuses it
unsafe extend Ctx implements ThreadShared:

fn main():
    ctx = make_ctx()                      # a Copy handle of this thread
    with scope = thread.scope():
        scope.spawn(owns fn(): use_ctx(ctx))    # the task reads the parent's `ctx`: `ref Ctx` is Send since Ctx is ThreadShared ([THR-8] "`ref T` and `Span[T]` when `T` is `ThreadShared`"); C is called on another thread than the creator, which the contract forbids; [FFI-53]'s trampoline check covers callbacks, not a handle's use
        use_ctx(ctx)
```

- MISSED EDIT rule=FFI-8
  - old: `E3100` without `unsafe`), an asserted fact `ember tcb` lists;
  - new: `E3100` without `unsafe`), an asserted fact `ember tcb` lists, and never for a type whose contract says `threads(creator)` or `threads(main)`, which binds every object to one thread: the declaration is `E7008` at its line ("`Ctx` is used by the thread that made it (`threads(creator)`), so no other thread may read one; remove the `unsafe extend`, or change the contract if the library allows any thread");
  - why: `[FFI-8]` lets a program declare `ThreadShared` on a type whose own contract makes it thread-bound; `[THR-8]` then makes a `ref` of it `Send`, and a scoped task reads the object on another thread. The two asserted facts contradict each other and the text must refuse the pair.

## MISSED: `[THR-9]` and the marker table leave `thread_shared fn()` outside `ThreadShared`, so a `thread_shared` class holding handlers has no consistent judgement

**rules**: THR-9, marker table §IV.8 row ThreadShared, THR-1, CLO-16

**program**:
```
class Bus:
    handlers: Array[thread_shared fn()] guarded by lock = []
    lock: Lock = Lock()

fn main():
    bus = thread_shared Bus()     # [THR-1] and [CLO-16] give `handlers: Array[thread_shared fn()]` as the line to write, yet neither [THR-9] nor the marker table says a `thread_shared fn()` is ThreadShared ("a type is ThreadShared when several threads may hold and read one value of it at once"); once K02's edit asks ThreadShared of the type, the rule's own fix line is refused unless [THR-9] lists it, and the marker table must agree with [THR-9] (ODR-052's condition)
```

- MISSED EDIT rule=marker table §IV.8 row ThreadShared
  - old: a `weakref thread_shared T` link is when `T` is `Send` and `ThreadShared` (`[THR-9]`) |
  - new: a `weakref thread_shared T` link is when `T` is `Send` and `ThreadShared`, and `thread_shared fn()` is, its state being `ThreadShared` (`[THR-9]`, `[THR-1]`) |
  - why: The table claims to agree with `[THR-9]`; the `[THR-9]` clause is given under K02, and the table must carry the same fact.
