# Phase 3b: an attack on the combined memory model, where its parts meet

Written 2026-10-24 for the owner, under the job in [JOB.md](JOB.md). The model attacked is
[phase3b-combined.md](phase3b-combined.md) (its second run, written at 10:12), called "the combined
report" below. Its parts were attacked one group at a time; this attack looks only at what breaks when
they meet: an answer from one group that undoes another group's safety argument, a cost that appears only
when two rules apply together, and a Python program that hits two rules at once. It also checks that the
combined report's C models do what its compiler would emit and exactly the work.

What I read: `JOB.md`; the combined report, all of it; its C models `combined/w1.c` to `w4.c`; the
baselines they are compared with (`salvage/plain/w1_group.c`, `salvage/plain/w3.c`,
`graphs-callbacks/bench/w2/w2_plain_malloc.c`); the parts of the group reports the problems below touch
(`phase3b-graphs-callbacks-attack.md` sections 3 to 7 and 12, `phase3b-threads-attack.md` sections 3.1 to
3.5, `phase3b-containers-attack.md` sections 7 and 9, `phase3b-containers.md` section 7,
`phase3b-ownership-attack.md` on names); the threads attack's `retarget.c` and the ownership attack's
`share_slab.c`, whose code two of the problems below rest on; and EnTT's and mimalloc's own pages for two
facts (named where used).

What I made, all under `C:/Users/ism19/Code/ember/target/redesign2/combined-attack/` (called `attack/`
below): one C program per problem that emits what the combined model's compiler would emit, built as
written and with the fix; a Python twin for each Ember program; three timed workloads; run files.
Nothing in `compiler/`, `std/`, `docs/`, `tests/` or `target/redesign2/combined/` was changed; no git
command was run; no Ember program was built or run.

**Timings are PROVISIONAL**: other agents were running. Each was taken on mains power
(`(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 before every round), pinned to the performance
cores (mask 0xC03C03), one program at a time, 5 runs each, the order reversed every other round, median
(`attack/run.ps1`, `attack/medians.py`). Every timed program prints its checksum, and every ratio names
its run file. A decision that rests on a difference under 15% waits for the final measurer.

---

## 0. The short version

Six problems where two parts of the model meet. Four are memory-safety holes, one breaks "every entity
once" in the ECS (memory stays safe), one is a cost. Each has a fix that was written, run, attacked again
and, where it costs anything, timed. **All six are fixed; none is left open.**

| # | What breaks | The two parts that meet | Kind | The fix | Its cost (PROVISIONAL) | Verdict |
|---|---|---|---|---|---|---|
| 1 | `if cam.target is not None:` then a line that kills the Player, then `cam.target.inventory.push(1)`: writes into the dead Player's freed list | narrowing (graphs group) and the combined report's own choice of `weak` (a test on read, 4.3) | memory safety | a statement that may drop a `Player` counts as a write of every `weak Player` place, so the narrowing ends there | none (compile time) | fixed |
| 2 | tasks in a parallel loop re-target handles to objects that some `weak` field points at: their count words now name the wrong side records; tasks that make first weak references crash | the threads group's netted counts, and the combined report's count word, which moves a weakly referenced object's counts into a global side table | memory safety | **weak counts beside the objects**: the strong count never leaves the header; the weak count lives in an array beside the object's slab slot; weak count changes in tasks are netted like strong ones; slabs per thread | W3: 0.946 and 0.956 of plain C, against 0.942 and 0.933 for the model as written; memory 9,232 KB against 9,508 KB | fixed |
| 3 | a `view` loop over a pool that an EnTT group owns, whose body adds a moving entity (W1's own spawn), visits some new entities and skips old ones; W1's body written as a view gives a different checksum | the ECS cursor rule (containers group) and the owning-group layout (the owner's choice for W1) | "every entity once" (memory stays safe) | a view walks a pool no group owns when it has one; new entities' move into the group waits until the walk ends; a body that changes existing entities' membership walks a copy of the ids | pending move: 0.942 and 1.016 of plain C; copy of ids: 1.11 and 1.09 of plain C with the same meaning | fixed |
| 4 | a stored closure that captures `boss = enemies[0]`: either writes into a freed buffer, or silently hits another enemy | names (ownership and containers groups) and captures (graphs group); the containers group asked the closures group to decide, and nobody did | memory safety, and a silent difference from Python | a class element's name is captured as its handle; a struct element's name may be captured only by a closure that dies before anything else can change the list, otherwise a compile-time error with fix-its | none (one count per capture, Python's meaning) | fixed |
| 5 | a shared object that is also weakly referenced (a camera's target) pays a side-record load on every count change: the sharing workload goes from 1.14x and 1.16x to 1.52x and 1.56x of plain C | counted class handles (ownership group) and the side record (combined report 3.4) | cost | the fix of problem 2 | 1.153 and 1.163 of plain C, the same as with no weak referrer (1.141 and 1.162) | fixed |
| 6 | `for c in old.children: if c.value > 5: new.children.push(c)` frees a child still in `new.children` if the loop drops what the push moved | back links' hidden removal (graphs group) and the one-pass loop filter (containers group) | memory safety | a removal whose value goes on living elsewhere closes the gap without dropping it | none | fixed |

Problems 2 and 5 share their fix, which replaces the combined report's side records for `weak`. The rules
the combined report should change are written out in section 5. **The C models**: all four do the work of
their baselines; I found no shortcut that changes a timing (section 4). One finding for the measurer: the
combined report's W3 was UNMEASURED there; in my batches it ran at 0.942 and 0.933 of plain C.

**The owner's question, "what about the agents that already ran with that problem?"** (the error boxes
opened by `cmd /c start`): for this step's inputs, I recounted the combined report's table and found it
right, and found one more fact: no run line in any of those folders lacks a time, so no start failed and
left a gap. My own runs never used `cmd`; three of my test starts failed because a library was missing,
and section 2.1 says what that means for the screen.

---

## 1. Words used

- **Place**: somewhere a value lives (a variable, a field `p.hp`, a list element `xs[3]`). **Name**: what
  `x = place` makes in this model, a second name for the value in that place; for a list element it is kept
  as a **position** ("element 0 of `enemies`").
- **Handle**: what a `class` variable holds, a pointer to a heap object. **Count** (strong count): how many
  owning handles an object has; it is freed at zero. **Weak field**: a field written `target: weak Player`;
  it does not keep its object alive and reads `None` once the object is gone. **Weak count**: how many weak
  fields point at an object.
- **Header**: the bytes at the start of an object; here 4 bytes (the count word) for a class with no
  subclasses and no base, 8 bytes otherwise. **Count word**: the 32-bit word holding the count and flag bits.
  **DEAD bit**: set when an object's fields have been dropped but weak fields still point at its block.
- **Side record**: in the combined report, a 16-byte record `{strong, weak}` in one global table; the first
  weak reference to an object moves both its counts there and puts the record's number in the count word
  (the SIDE bit). Swift does the same with its side table.
- **Narrowing**: after `if x is not None:` (or `isinstance`), the compiler treats `x` as not `None` in the
  block, so it emits no test on later uses. **Effect summary**: what the compiler records about a function
  from its body: which places it reads, writes and reshapes, and which classes of object it may drop.
- **Task**: work run on another thread inside a `thread.scope()` or a `@parallel` loop. **Netted counts**
  (threads attack 3.1): a task does not change a shared object's count at once; it adds +1 and -1 into a
  table of its own and applies the sums with atomic instructions, increases when its loop ends, decreases
  only after every task has finished its loop (the barrier).
- **Slab**: an allocator that hands out same-size blocks from big chunks. **Chunk**: one such big block.
- **Owning group** (EnTT): for `Position` and `Velocity`, the entities that have both sit at the front of
  both pools in the same order. **Pool**: one component type's arrays. **View**: a loop over the entities
  that have some components. **Cursor**: the position a running view or group loop has reached, which other
  operations read so that "every entity once" holds.
- **One-pass filter** (containers group): how a loop that removes from the list it walks is compiled: a read
  position and a write position walk the list once and close the gaps.
- **Back link**: a field written `parent: Option[Node] = backlink(children)`, kept equal to "the node whose
  `children` holds me"; pushing a node into a new parent's `children` takes it out of the old one's first.
- **AddressSanitizer**: clang's checker that stops a program at its first read or write of freed or
  out-of-range memory (`-fsanitize=address`).
- **Plain C**: C written for speed with no safety checks. **PROVISIONAL**: measured while other agents ran.
  **UNVERIFIED**: believed, not checked.

---

## 2. How this was done

### 2.1 The agents that ran with the error-box problem

The problem: a program started through `cmd /c start` from Git Bash could open a "Windows cannot find" box
on the owner's screen, and a start that fails runs nothing.

**The combined report's inputs.** Its section 2.1 says which runners used `cmd /c start` (from PowerShell)
and that every line with a time in their run folders also carries a checksum, which a program prints only
after all its work. I checked both:

- The runners: `salvage/runs/ab.ps1` line 12, `graphs-callbacks-attack2/bench/run.ps1` line 15 and
  `ownership/bench/run.ps1` line 12 do contain `& cmd /c "start `"`" /b /wait /affinity C03C03 ..."`.
- The counts: for each of its 13 run folders, `cat <folder>/*.txt | grep -c "ms="` and the same with
  `checksum` give exactly the combined report's numbers (748/748 for `salvage/runs`, 670/670 for
  `graphs-callbacks-attack2/bench/runs`, 312/312 for `threads/attack2/bench/runs`, and so on; for
  `references/attack2/bench/out` both counts are 242, because its 31 summary lines say `checksums=`).
- One fact the combined report did not check: whether any start failed and left a round **without** a
  time. In all 13 folders, `grep "exe="` (or `prog=`, or `round=`) lines without `ms=`: **0**. Every program
  in every round ran to its end.

So the answer the combined report gave stands, and is now stronger: none of the numbers it uses comes
from, or is missing because of, a failed start.

**My own runs.** Every program was started from Git Bash directly (`./prog.exe`) or by `attack/run.ps1`
from PowerShell (`& $exe`); no `cmd`, no `start`. Every C program calls `h_start()` first, which calls
`SetErrorMode` so a crash opens no box. Two things to report:

- Three starts of AddressSanitizer builds failed (`narrow_weak_as_written.exe` twice,
  `narrow_weak_saved.exe` once) because the sanitizer's library was not on the search path; Git Bash
  printed `error while loading shared libraries: clang_rt.asan_dynamic-x86_64.dll`, exit 127. Windows can
  show a "System Error: ... was not found" box when a program's library is missing; that box waits for a
  click, and these commands returned at once, so I believe no box was shown, but I cannot see the screen
  (UNVERIFIED). Every later run put the library's folder on `PATH` first.
- One start of `threads_side_race.exe` ended with exit 127 for a reason I did not find (its message line
  was overwritten when I rewrote that output file); I rebuilt it and its next three runs ended as expected.
  Windows Defender's log shows no detection in the last three hours (`Get-WinEvent` on
  `Microsoft-Windows-Windows Defender/Operational`, events 1006, 1007, 1015 and 1116 to 1119: none).
- The race build and the AddressSanitizer builds are meant to crash; they did, after `h_start()`.

### 2.2 How each problem was shown

For each problem: the Ember program in the combined model's syntax (today's `ember check` refuses this
syntax, as it does the combined report's own examples); the Python twin, run with
`C:\Python314\python.exe -I` (Python 3.14.2); and a C program that emits what the combined model's
compiler would emit, built as the combined report's rules read today and built with the fix, under
AddressSanitizer where memory is at stake: `"C:/Program Files/LLVM/bin/clang.exe" -O1 -g
-fsanitize=address` (clang 22.1.8). Timed programs are built with `-O2`. Each program's output is in a
`*_output.txt` file next to it.

---

## 3. The problems

### 3.1 Problem 1: narrowing a `weak` field survives the death of its target (memory safety)

**The two parts.** The graphs group's narrowing (combined rule 11): after `if m is not None:` a place is
treated as not `None` "until any write of that place in any object (hidden writes included)". The graphs
group designed it with **its** `weak`, cleared by registration: a target's death wrote `None` into every
weak field pointing at it, a hidden write that ended the narrowing. The combined report replaced that `weak`
with a test on read (its 4.3): a death writes nothing into the weak field; reads test the target's DEAD
bit. Nothing ends the narrowing any more.

```python
# attack/narrow_weak.py
import weakref
class Player:
    def __init__(self): self.hp = 100; self.inventory = []
class Camera:
    def __init__(self): self._target = None
    @property
    def target(self): return self._target() if self._target is not None else None
    @target.setter
    def target(self, p): self._target = weakref.ref(p)

players = [Player()]
players[0].inventory.append(5)
cam = Camera(); cam.target = players[0]
if cam.target is not None:
    players.pop()                          # the last reference: the Player is freed here
    cam.target.inventory.append(1)         # AttributeError: 'NoneType' object has no attribute 'inventory'
```

```ember
class Player:
    hp: int = 100
    inventory: Array[int] = []
class Camera:
    target: weak Player = None

players = [Player()]
players[0].inventory.push(5)
cam = Camera(); cam.target = players[0]
if cam.target is not None:                 # narrowed: cam.target is a Player in this block
    players.pop()                          # the only strong handle goes: the Player's fields are dropped
    cam.target.inventory.push(1)           # as written: still narrowed, so no DEAD test is emitted
```

`attack/narrow_weak.c` emits the second read as a plain load (narrowed) and the death as the combined
model's `w3.c` does (fields dropped, DEAD set, block kept for the weak field). Output
(`attack/narrow_weak_output.txt`):

```text
== narrow_weak_as_written.exe
==19872==ERROR: AddressSanitizer: heap-use-after-free on address 0x12265e2a07c8 ...
WRITE of size 8 at 0x12265e2a07c8 thread T0
    #0 ... in main ...\combined-attack\narrow_weak.c:78
== narrow_weak_saved.exe
fixed: cam.target is None after the pop
== python narrow_weak.py
python: AttributeError: 'NoneType' object has no attribute 'inventory'
```

The push writes into the inventory buffer the death freed. Reading `cam.target.hp` instead would silently
read a dead object where Python raises.

**The fix.** For narrowing, and for every rule that asks "may this place have changed?", a statement or
call whose effect summary may drop an object of class `T` counts as a write of every `weak T` field and
`Array[weak T]` element. After `players.pop()` the type of `cam.target` is `Option[Player]` again, so
`cam.target.inventory` is a compile-time error naming the pop, with two fix-its: test again
(`if cam.target is not None:`, which finds `None`, Python's outcome without the exception), or take a name
first (`t = cam.target`, a strong handle; the count rule keeps the Player alive across the pop, which is
also what Python does: `narrow_weak.py`'s second half prints `inventory [1] camera sees a Player`). The
saved build emits the test again and finds `None`.

**The fix attacked.**
- What "may drop a `T`" covers: overwriting a `T` handle, removing from a list of `T`, a local going out of
  scope, the drop of anything that owns `T` handles, a call whose summary does any of these. The count rule
  already needs exactly this fact ("a handle read into a local is not counted when nothing in the local's
  life may drop an object of that class", combined 0.2), so no new analysis is needed.
- Threads: a task's decreases happen only after the barrier, at the scope's end, whose summary carries the
  tasks' effects; under the fix those drops are writes of `weak T` places, so the scope rule sees a parent
  read of `cam.target` during the scope as conflicting with them. A class a weak field points at is not
  shared with an unscoped thread (combined 4.8), so no other drop can happen behind the compiler's back.
- `yield` and code the compiler cannot see "may do anything", so they end the narrowing too.
- It refuses more than it must: the death of a fresh `Player` that no weak field could point at also ends
  the narrowing. The user writes the test again, a one-bit test, which Python's property call pays anyway.

**Cost**: none when running. **Verdict: fixed.**

### 3.2 Problem 2: netted counts, weak stores in tasks, and the side records (memory safety)

**The two parts.** The threads group lets tasks retain and release objects the parent owns, with counts
netted per task; the netted increase is `__atomic_fetch_add(&count, delta)` and the decrease
`if (__atomic_fetch_sub(&count, n) == n) free(...)` (`threads/attack2/bench/retarget.c` lines 59 and 106),
written for a plain count. The combined report's count word (its 3.4) moves the counts of any object a
weak field points at into a side record and puts the **record's number** in the count word's low bits. The
same atomic add then adds to the record's number. Separately, a weak store runs `combined/w3.c`'s
`weak_retain`, which allocates side records from one global table that `realloc` may move, with plain
increments: hidden shared state that no effect summary names, so the scope rule cannot see two tasks touch
it.

```python
# attack/threads_side.py (Camera's weak property as in narrow_weak.py)
players = [Player(i) for i in range(8)]
cams = [Camera() for _ in range(64)]
for i, c in enumerate(cams): c.target = players[i % 8]        # every player weakly referenced
ws = [Watcher(players[0]) for _ in range(64)]
def aim(i):
    ws[i].target = players[1 + i % 7]
    cams[i].target = players[(i + 3) % 8]
with ThreadPoolExecutor(4) as ex: list(ex.map(aim, range(64)))
# prints: references per player (list + watchers): [1, 11, 10, 10, 10, 10, 10, 10]
```

```ember
@parallel                                     # 4 tasks
for i, w in enumerate(ws):
    w.target = players[1 + i % 7]             # netted: +1 at the loop's end, -1 after the barrier
    cams[i].target = players[(i + 3) % 8]     # a weak store in a task (each task writes its own cameras)
```

`attack/threads_side.c` runs this with real threads, the combined model's count word and `w3.c`'s side
records copied unchanged, and the threads attack's netted instructions. Output
(`attack/threads_side_output.txt`):

```text
== threads_side_as_written.exe
as written: player 0 count word 1fffffc0 (SIDE 0, index 268435392); its count should be 1, it reads 268435392
as written: player 1 count word 2000000b (SIDE 1, index 11); its count should be 11, it reads -1
as written: player 2 count word 2000000b (SIDE 1, index 11); its count should be 10, it reads -1
...
as written: 1's count word names side record 11, past the 8 that exist
exit=3
```

Player 1's record number went from 1 to 11 (ten netted increases added to it), player 2's from 2 to 11:
two objects now share one record. Player 0's 64 netted decreases borrowed through the SIDE bit, set DEAD
(bit 28) on a live object, and left a record number of 268,435,392. Every later release reads or frees
through the wrong record.

The second case needs no shared object at all: eight tasks each make the first weak reference to 100,000
objects only they touch (`-DRACE`):

```text
== threads_side_race.exe          (three runs, -O2)
exit=139 (Segmentation fault)  x3
== threads_side_race_asan.exe
ERROR: AddressSanitizer: heap-buffer-overflow ... WRITE of size 8 ... thread T2
    #0 ... in race_task ...\threads_side.c:175
... located 0 bytes after 16384-byte region ... allocated by thread T1 here: ... realloc
```

The slab allocator has the same flaw by reading: `combined/w3.c` and `w4.c` keep one free list per size,
and under netted counts a task frees the objects whose count it brings to zero after the barrier, on its
own thread, while other tasks free theirs.

**Trying the smallest fix first.** Keep the side records and make every count change on a weakly
referenced object take a compare-and-swap that checks SIDE, make the table never move, and allocate
records with atomic instructions. That repairs safety, but problem 5 below shows the side record also
costs 1.34x on ordinary counting in a single thread, so I looked for one change that fixes both.

**The fix: weak counts beside the objects.**
1. The strong count never leaves the header. (A count past 28 bits, about 268 million handles to one
   object, still moves to a side record, entered and left with compare-and-swap; not reached by any
   workload.) The count word keeps DEAD; SIDE means only that overflow.
2. The weak count lives beside the object: each slab chunk (256 KiB, aligned to 256 KiB, so an object's
   chunk is its address with the low bits cleared) gets, the first time any of its objects becomes a weak
   target, an array of 32-bit weak counts, one per slot. The weak count counts the weak fields plus one for
   all strong handles together while any weak field exists (Rust's `Arc` keeps the same extra one), so
   exactly one decrement reaches zero and frees the block, whichever side lets go last.
3. In tasks, weak count changes are netted exactly like strong ones; a chunk's array is created with a
   compare-and-swap on the chunk's header; a netted decrease compares the count bits, `(old & LOW) == n`,
   not the whole word with its flags.
4. Slabs are per thread; a block freed on another thread goes onto a separate list for such frees with
   one compare-and-swap, which is mimalloc's design ("there is one list for thread-local free operations,
   and another one for concurrent free operations"; "Free-ing from another thread can now be a single CAS",
   https://github.com/microsoft/mimalloc README). Not built here; the saved test uses `malloc`.
5. Classes too big for a slab keep their weak count in a word before the header, decided per class when
   compiling (not built, UNVERIFIED that every such class can be found statically; a `dyn` handle finds its
   size in its method table).

```text
== threads_side_saved.exe
saved: 0 players with a wrong count after the scope; players freed 8 of 8
```

Under AddressSanitizer: no error, every count right, every player freed exactly once.

**The fix attacked.**
- The claim test (threads attack 3.5): a parallel loop over a list of handles writes an object without a
  claim when its count is 1. With the side record, the count word of every weakly referenced object is
  not 1, so every such element takes the claim; the threads attack measured that path at 3.19x to 3.98x of
  plain C ("count test, every object also held elsewhere", its section 3.5 table; that the cost carries over
  exactly is argued, UNVERIFIED). With the fix the count word holds the real count again.
- Memory in the worst case: one weak target in every chunk costs 4 bytes per slot of each chunk, 26 KiB per
  256 KiB chunk of 40-byte objects (about 10%). Rust's `Rc` keeps both counts, 16 bytes, in every object
  always (its `RcBox`, from memory, UNVERIFIED). In W3, 17 chunks got an array (`attack/checks_output.txt`).
- A dead block's slot is reused only when its weak count is 0, so a new object never inherits a count.
- A weak read still tests DEAD in the header, and DEAD can only be set after the barrier in tasks.

**Cost** (`attack/w3_beside.c`, the combined W3 with only this change; runs `w3_beside_a.txt`,
`w3_beside_b.txt`, ratio to `salvage/plain/w3.c`):

| Program | batch a | batch b | peak memory |
|---|---|---|---|
| plain C | 437.693 ms (1.000) | 438.306 (1.000) | 9,756 KB |
| combined W3 as written (side records) | 412.187 (0.942) | 408.842 (0.933) | 9,508 KB |
| weak counts beside the objects | 414.141 (0.946) | 418.961 (0.956) | 9,232 KB |

The fix is within 3% of the model as written (under 15%: the final measurer decides) and uses less memory.
The poison build (freed blocks never reused, every weak read checked) prints `poison check ok: 20 blocks
freed, none read after free`, the same 20 as the combined report's, with the known checksum.

**Verdict: fixed.** The netted weak counts and the per-thread slab are argued and not timed; they use the
mechanisms the threads attack and mimalloc already measured.

### 3.3 Problem 3: a view over a pool that a group owns ("every entity once"; memory stays safe)

**The two parts.** The containers group's ECS rule keeps "every entity once" with a cursor that every
removal from the walked range respects. It was built for EnTT's sparse layout. For W1 the owner chose
EnTT's owning group, and the combined report's library (its 6.1) has both loops: `view` (a pool walked
backwards with the pool's cursor, `Pool.leave` applying the rule) and `group` (the group's range with the
group's cursor). The group's own operations reorder the pools it owns: an entity that gets both components
is swapped to the group's end, one that loses one is swapped out. Neither swap looks at a running view's
cursor, so a `view` over `Position` (owned) moves visited and unvisited entities across its cursor.

EnTT documents the same limit: for an owned component iterated outside its group, "adding to an entity all
the components required to get it into the group may invalidate the iterators", and "full consistency for
owned components is guaranteed only when they are iterated as part of their groups"
(https://github.com/skypjack/entt/wiki/Entity-Component-System, "What is allowed and what is not").

```python
# with the registry of dicts in combined/python/w1_dicts.py, whose view walks a copy of the keys
# (this loop itself not run; its meaning is the one ecs_view_cost.c's models 0, 1, 3 and 4 share)
for e, pos in reg.view(Position):
    pos.x += 0.5
    if e % 100 == s:
        reg.destroy(e)
        n = reg.create(); reg.add(n, Position(0, 0)); reg.add(n, Velocity(1, 1))
```

```ember
for e, pos in reg.view[Position]():        # every entity with a Position; the group owns Position
    pos.x += 0.5
    if e.id % 100 == s:
        reg.destroy(e)
        n = reg.create()
        reg.add(n, Position(x=0.0, y=0.0))
        reg.add(n, Velocity(dx=1.0, dy=1.0))  # n joins the group: swapped to its end, across the cursor
```

`attack/ecs_view_group.c` copies `combined/w1.c`'s library and adds section 6.1's pool cursor, `Pool.leave`
and `view`, plus a third component, `Health`, that no group owns. 5,000 random loops per kind; each step
changes other entities (destroy, take `Velocity` away, give it, create with both, create with
`Position` only). Output (`attack/ecs_view_group_output.txt`):

```text
== ecs_view_group_as_written.exe
as written: view[Position]: 5000 loops, 27911 visits wrong (an entity visited twice, visited after joining, or skipped); first failing seed 1
as written: view[Position, Health]: 5000 loops, 12102 visits wrong ...; first failing seed 2
== ecs_view_group_spawn_only.exe     (only W1's own changes: destroy, create with both)
as written: view[Position]: 5000 loops, 39602 visits wrong ...; first failing seed 1
== ecs_view_group_saved.exe and ecs_view_group_spawn_only_saved.exe
saved: view[Position]: 5000 loops, 0 visits wrong ...
saved: view[Position, Health]: 5000 loops, 0 visits wrong ...
```

W1's body itself is enough. Written as a view over `Position` (`attack/ecs_view_cost.c`), it prints
checksum 170232906 where the same work done right prints 56628502. W1 as the combined report writes it uses
`reg.group[(Position, Velocity)]()`, which is safe; the problem is the other spelling, which a Python
programmer is as likely to write.

**The fix**, in the library and from the body's effect summary:
1. A view walks a pool that no group owns when it has one (`view[Position, Health]` walks `Health`). Free.
2. A view over exactly a group's components is that group's loop (already the containers attack's rule 1).
3. A view whose pools are all owned, whose body only creates entities and destroys the current one (W1's
   body): a new entity gets its components at once (`reg.get` finds them), but its move into the group
   waits in a pending list until the walk ends; a group loop that starts inside the walk also walks that
   list.
4. A view whose pools are all owned, whose body may give an existing entity, or take from it, a component
   the owning group owns: the view walks a copy of the walked pool's ids taken when it starts, and looks each
   one up (Python's `for e in list(a)`).

**The fix attacked.** Deferring the *leave* as well was tried on paper and fails: removing `Velocity` from
an entity still in the group's range would break the group's order in the `Velocity` pool at once, and
keeping it is what the "today's rules, but never an error" design's tombstones did. Moving an entity across
the group's boundary while keeping its "visited" state is impossible with positions alone: a joining entity
that was visited must end up in front of the cursor, where everything counts as unvisited. Hence the copy
for case 4. A pending entity destroyed before the walk ends is skipped at the flush (it no longer has both).
Both saved builds print 0 wrong visits.

**Cost** (`attack/ecs_view_cost.c`: W1's entities, W1's body as a view over `Position`, 1,000,000 entities,
100 steps; runs `ecs_view_a.txt`, `ecs_view_b.txt`; ratio to plain C that walks the pool and keeps the
spawns for after the loop, as EnTT requires):

| Program | batch a | batch b | peak memory | checksum |
|---|---|---|---|---|
| plain C, spawns kept for after the loop | 179.527 ms (1.000) | 168.817 (1.000) | 61,416 KB | 56628502 |
| fix case 3: the pending move into the group | 169.077 (0.942) | 171.466 (1.016) | 61,576 KB | 56628502 |
| plain C walking a copy of the ids | 228.972 (1.275) | 237.620 (1.408) | 65,372 KB | 56628502 |
| fix case 4: a copy of the ids, with bounds checks | 254.965 (1.420) | 258.652 (1.532) | 65,324 KB | 56628502 |
| the cursor walk as written (not the same work) | 169.453 (0.944) | 166.891 (0.989) | 61,020 KB | 170232906 |

Case 3 is on par with plain C. Case 4 is 1.11x and 1.09x of plain C that keeps the same meaning (the copy);
plain C reaches 1.00x only by deferring the change, which changes what the rest of the loop sees. Case 4
applies only when a body changes the group membership of entities that already existed.

**Verdict: fixed.** One more case of the same root, noted and not built: the cursor is one per pool or
group, so two walks of one pool that are not nested (two stored generators advanced in turn) protect only
whichever wrote the cursor last. The combined report already lists nested loops as a limit; a small list
of running cursors per pool, with a removal moving the gap through each cursor in order, would cover both
(argued, UNVERIFIED).

### 3.4 Problem 4: a stored closure that captures a name (memory safety, or a silent difference)

**The two parts.** Names (ownership group A11, containers group A2): `boss = enemies[0]` names element 0
of `enemies`, a position moved by removals the compiler sees. Captures (graphs group A12, combined rule
10): per captured variable, copy, move, pointer or box. A name is neither a value nor a plain variable. The
containers group asked the closures group to decide (`phase3b-containers.md` section 7, item 6: "the
simplest rule is that capturing a name captures a copy of the element ... or the container and the
position"); neither graphs report answers, and the combined report's rule 10 does not mention names.

```python
# attack/closure_name.py
def setup(g):
    boss = g.enemies[0]
    def on_hit(): boss.hp -= 1
    g.buttons.append(on_hit)             # stored: called long after setup returns
def wave(g):
    g.enemies.pop(0)                     # the boss leaves the list
    for i in range(64): g.enemies.append(Enemy(i))
# prints: enemies[0].hp = 20 ; the boss (out of the list) hp = 9
```

```ember
fn setup(g: Game):
    boss = g.enemies[0]                  # a name: element 0 of g.enemies
    g.buttons.push(fn(): boss.hp -= 1)   # the closure escapes into g.buttons
fn wave(g: Game):                        # knows nothing of the closure
    g.enemies.remove(0)
    for i in range(64): g.enemies.push(Enemy(hp=i))
```

`attack/closure_name.c` emits each reading of rule 10 (Enemy a struct, then a class). Output
(`attack/closure_name_output.txt`):

```text
== closure_name_1.exe   (the box holds the element's address)
ERROR: AddressSanitizer: heap-use-after-free ... READ of size 8 ...
== closure_name_2.exe   (the box holds the path: the game handle and position 0)
reading 2 (path): enemies[0].hp = 19
== closure_name_3.exe   (the box holds a copy of the element)
reading 3 (copy): enemies[0].hp = 20
== closure_name_4.exe   (Enemy a class: the box holds the handle, counted)
reading 4 (handle (class Enemy)): enemies[0].hp = 20, the boss (out of the list) hp = 9
== python closure_name.py
python: enemies[0].hp = 20 ; the boss (out of the list) hp = 9
```

The address reading reads freed memory after the wave's pushes move the buffer. The path reading cannot
be moved by `wave`, which does not know the box exists, so the click hits the new first enemy (19 where
Python leaves 20). The copy reading changes nothing anyone can see: the program meant the boss.

A second case of the same meeting, by reading: `items = []; ui.on_click(fn(): items.push(1));
bag.push(items)`. `items` lives in the closure's box; "names follow values" would move the list into
`bag` and leave the box with a moved-from list (or two owners of one buffer).

**The fix.**
- A closure that captures the name of a **class** element captures the handle, with one count (rule 10's
  "copying a handle retains"). This is Python's meaning exactly: reading 4 matches Python.
- A closure that captures the name of a **struct** element (or a value named inside a container, `row`
  after `matrix.push(row)`) keeps the path only when it provably dies before anything other than visible
  code of the name's own function can change the name's list (the same proof rule 10 uses for pointer
  captures; the function's own removals move the name as rule 4 says). Otherwise it is a compile-time error
  with fix-its: `copy(boss)` (a snapshot, said out loud), make `Enemy` a class, or capture the list and an
  index.
- Moving a value out of a variable an escaping closure captures is the one-owner error of rule 5, with its
  fix-its (`copy(items)` or a class): the box is an owning place.

**The fix attacked.** `matrix = []; row = []; matrix.push(row); f = fn(): row.push(1); f()` compiles (the
closure dies first) and gives Python's `[[1]]`. A non-escaping closure passed to a function that also
changes the list (`apply(enemies, fn(): boss.hp -= 1)`) cannot keep the path, because `apply`'s removals
would not move it; the rule's "anything other than visible code of the name's own function" refuses it.
**What is left against Python:** a stored closure over a struct element's name is refused; Python has no
structs, and the class spelling is Python's own. A run-time alternative (the list keeps a record of
positions held by closures and moves them on removal) would keep the struct spelling at a cost on every
removal from such a list; not built.

**Cost**: none, or one count per capture of a class element. **Verdict: fixed.**

### 3.5 Problem 5: counting a weakly referenced object goes through the side record (cost)

**The two parts.** The ownership group's counted class handles, and the combined report's side record. The
combined report (4.3) says the side record costs "one more load on stores to reach the side record ...
UNMEASURED", but its count word moves **both** counts into the record, so every strong retain and release
of a weakly referenced object takes the slow path and loads a record from a separate table. W3 does not
show it (its loop changes no strong count); the ownership attack's sharing workload does, once its players
have a weak referrer, as a camera, a HUD or an observer list gives them.

```python
t = ws[i].target            # 1,048,576 watchers share 262,144 players; one in 16 re-targets per frame
t.hp -= 1
```

`attack/share_weak.c` is `ownership-attack2/bench/share_slab.c`'s work line for line, with the combined
count word, and one weak referrer per player set up before the timed loop (runs `share_weak_a.txt`,
`share_weak_b.txt`; ratio to plain C with the same slab, `share_slab0.c`'s program):

| Program | batch a | batch b | peak memory |
|---|---|---|---|
| plain C | 232.175 ms (1.000) | 234.107 (1.000) | 53,284 KB |
| the combined count word, no weak referrer | 269.829 (1.162) | 267.112 (1.141) | 55,332 KB |
| the combined count word, every player weakly referenced (side records) | 363.225 (1.564) | 356.876 (1.524) | 59,432 KB |
| weak counts beside the objects (problem 2's fix), every player weakly referenced | 270.106 (1.163) | 270.027 (1.153) | 56,360 KB |

The side record adds 35% and 34% to the lean class's own cost (363.225 / 269.829, 356.876 / 267.112); the
fix removes it. All four print checksum 80363660820. The remaining 1.14x to 1.16x is the lean class's own
open cost (combined 4.9), unchanged.

**Verdict: fixed**, by problem 2's fix.

### 3.6 Problem 6: a back link's hidden removal inside the loop filter (memory safety)

**The two parts.** A push into a back-linked `children` first takes the node out of its old holder's
list; "its count moving with it" (combined rule 8). A loop whose body may change the list it walks is the
one-pass filter, and "a removed element is dropped at the end of its step" (rule 7). The graphs group
handed this loop to the containers group's rule (`phase3b-graphs-callbacks.md` line 279); neither rule says
whether a hidden removal is a drop or a move.

```python
# attack/backlink_filter.py
for c in old.children:
    if c.value > 5: reparent(c, new)        # Python skips the child after each move: [7] [3, 9, 2]
for c in list(old.children):
    if c.value > 5: reparent(c, new)        # over a copy: [7, 9] [3, 2]
```

```ember
for c in old.children:                      # the body may change old.children: the one-pass filter
    if c.value > 5:
        new.children.push(c)                # c leaves old.children: a hidden removal of the walked list
```

`attack/backlink_filter.c` maps the hidden removal onto the filter both ways (`backlink_filter_output.txt`):

```text
== backlink_filter_as_remove.exe     (mapped as remove: dropped at the end of its step)
ERROR: AddressSanitizer: heap-use-after-free ... READ of size 4 ...
== backlink_filter.exe               (mapped as a move out of the walked list)
new.children: 7 9   old.children: 3 2
both trees released
```

Dropped as a removal, the count is one too low: the push moved the handle without a new count. As a move,
the result is Python's walk over a copy (the filter does not copy Python's skip, as rule 7 already says).

**The fix.** A removal from the walked list whose value goes on living elsewhere (a back link's move into
another holder, `x = xs.pop(i)`, a removed element moving into a name) closes the gap and drops nothing;
only a removal that discards the value drops it at the end of its step. **Attacked**: a back-linked
`replace` (`old.children[k] = other`) inside the walk unlinks and releases the replaced child (step 3 of
rule 8's membership change): a discard, dropped at the end of the step, correct. **Cost**: none.
**Verdict: fixed.**

---

## 4. Do the C models emit what the compiler would, and exactly the work?

| Model | Same work as its baseline | What I checked | Findings |
|---|---|---|---|
| `combined/w1.c` against `salvage/plain/w1_group.c` | yes | the setup, the step loop, `destroy`, `group_enter`/`leave`, the checksum loop line by line | The Ember library text in combined 6.1 is incomplete: its `destroy` loops `pool.remove(e)` over every pool and never calls the group's leave, which the C (rightly) calls first; and its `Pool.leave` compares with the pool's cursor, which the C's `pool_del` leaves out. That compare is one test per destroy on the rare path (at least 500,000 destroys in all: each of the 500,000 entities that start in the group is destroyed once, at the step equal to its id modulo 100, plus some spawned ones), never taken in W1: no shortcut that moves a timing. The registry's type-table lookup is left out, as the combined report says. |
| `combined/w2.c` against `w2_plain_malloc.c -DSLAB -DARRAY24` | yes | random calls in the same order, the re-parent order (remove from the old parent, then push), the checksum | The model's `nodes` list grows by doubling (Python's list) where the baseline allocates 1,000,000 pointers once: the model does slightly more work, not less. |
| `combined/w3.c` against `salvage/plain/w3.c` | yes | random calls in `make` and for targets in the same order, the update dispatch, the filter, the checksum | None. Its time was UNMEASURED in the combined report; my batches: 0.942 and 0.933 of plain C (`runs/w3_beside_*_summary.txt`). Problem 2's fix replaces it with `attack/w3_beside.c`. |
| `combined/w4.c` | yes (the second graphs attack checked it against `w4_plain_same.c`) | the dispatch, the handler's rare branch (it reads `counter` before releasing its box), the final loop reading the length after every call | None. `k % len` with a length of 0 would divide by zero; zero-divisor checks are left out of every model and baseline, as stated. |

The Python twins print the known checksums (the combined report's `run_twins_log.txt`); I did not rerun
them. The combined report's other checks (cursor self-check, link check, poison builds) I did not repeat,
except that my `ecs_view_group.c` exercises `w1.c`'s group operations again and finds them consistent.

---

## 5. What the combined model should change

Each line names the combined report's rule it amends.

1. **Rule 9, `weak`** (problems 2 and 5): a weak field or list element is a plain pointer; a read tests
   the target's DEAD bit (unchanged). The strong count never leaves the header; only a count past 28 bits
   moves to a side record, entered and left by compare-and-swap. The weak count lives beside the object, in
   an array of 32-bit counts per slab chunk created when the chunk first holds a weak target; it counts the
   weak fields plus one for the strong handles together. A strong count reaching zero with a weak count set
   marks DEAD, drops the fields and lets go of the extra one; whoever brings the weak count to zero frees the
   block. Classes too big for a slab keep the weak count in a word before the header. Section 3.4's count
   word: bits 31 shared, 30 claim, 29 SIDE (overflow only), 28 DEAD, 0 to 27 the count.
2. **Rule 11, narrowing** (problem 1): a statement or call that may drop an object of class `T` counts as a
   write of every `weak T` field and list element, for narrowing and for every rule that asks whether a
   place may have changed.
3. **Rule 10, closures** (problem 4): a captured name of a class element is the handle, counted; a captured
   name of a struct element (or of a value named inside a container) keeps its path only in a closure that
   dies before anything other than visible code of the name's function can change the list, and is
   otherwise a compile-time error with fix-its (`copy(x)`, `class`, the list and an index); moving a value
   out of a variable an escaping closure captures is rule 5's error.
4. **Rule 7, loops** (problem 6): a removal from the walked list whose value lives on elsewhere closes the
   gap without dropping; only a discarding removal drops at the end of its step.
5. **Rule 12, threads** (problem 2): weak count changes in tasks are netted like strong ones; a netted
   decrease compares the count bits, not the flags; a chunk's weak-count array is created by
   compare-and-swap; slabs are per thread, with a concurrent-free list per chunk (one compare-and-swap per
   free from another thread).
6. **The ECS library** (problem 3): a view walks a pool no group owns when it has one; a view over exactly
   a group's components is that group's loop; a view whose pools are all owned keeps new entities' move into
   the group in a pending list until it ends (a group loop started inside it walks the list too) and, when its
   body may change the group membership of entities that already existed, walks a copy of the ids. Section
   6.1's `destroy` text should show the group's leave.
7. **Section 4.3's reasons** change: reason 3 (memory) gets better (W3 9,232 KB against 9,508 KB in my
   batches); "one more load on stores" becomes "one address mask, one load and an increment on a weak store,
   nothing on strong counts".

Annoyance rows this changes (combined section 5): A8 "other back references" (cost: one bit test per read,
nothing extra on strong counts; the side record is gone), A12 (the name rule above), A13 (narrowing ends at
a possible drop of a weak target), ECS (the view rules above).

---

## 6. For the final measurer

| Program | Compare with | Why |
|---|---|---|
| `attack/w3_beside.c` (`w3-weak-beside`) | `salvage/plain/w3.c` and `combined/w3.c` | the W3 the amended model emits; within 3% of `combined/w3.c` here |
| `attack/share_weak.c -DMODEL=9` (weak counts beside) and `-DMODEL=8` (side records) | `-DMODEL=0` (plain C) and `-DMODEL=3` (no weak referrer) | the cost problem 5 removes |
| `attack/ecs_view_cost.c -DMODEL=3` and `-DMODEL=1` | `-DMODEL=0` and `-DMODEL=4` | the two paths of problem 3's fix; model 2 is not the same work and is there only to show the wrong checksum |

Builds: `"C:/Program Files/LLVM/bin/clang.exe" -O2 [-DMODEL=n] <file>.c -o <file>[_n].exe` in `attack/`.
`combined/w1.c`, `w2.c` and `w4.c` are unchanged by this attack.

---

## 7. Good parts of what this attack replaced

| Replaced | From | Replaced by | Good part kept |
|---|---|---|---|
| side records for every weakly referenced object, found through the count word | the combined report 3.4 and 4.3, after Swift's side table (`phase2-swift-mojo.md` line 96) | weak counts beside the objects | memory only for what is weakly referenced, and no header growth; the record stays for counts past 28 bits |
| a global free list per object size | the combined models, salvage good part 9 | per-thread slabs with a concurrent-free list (mimalloc) | the slab itself: W3 -19%, W4 -17% (salvage 2.5) |
| a view's cursor walk over an owned pool when the body changes the group | the containers group's ECS rule in EnTT's owning-group layout | the pending move into the group, or a copy of the ids | the cursor walk, still used for every view over a pool no group owns and for every group loop |
| "a captured variable is copied, moved, pointed to or boxed" applied to names | combined rule 10 | the handle for class elements, the path only when provably safe, otherwise a refusal | the box and pointer captures for every plain variable |

The counts-beside-the-objects idea is the ownership attack's own good part ("counts beside the objects,
the class known from its chunk", its 5.3, listed in combined section 8), used here for the weak count only.

---

## 8. Files and commands

All under `C:/Users/ism19/Code/ember/target/redesign2/combined-attack/`:

| File | What it is |
|---|---|
| `narrow_weak.c`, `narrow_weak.py`, `narrow_weak_output.txt` | problem 1: as written and `-DSAVED`, under AddressSanitizer; the Python twin |
| `threads_side.c`, `threads_side.py`, `threads_side_output.txt` | problem 2: as written, `-DSAVED`, `-DRACE` (at `-O2` and under AddressSanitizer); the Python twin |
| `w3_beside.c`, `checks_output.txt` | problems 2 and 5's fix in W3; its `-DPOISON` self-check and checksums |
| `ecs_view_group.c`, `ecs_view_group_output.txt` | problem 3's check, as written, `-DSAVED`, and `-DSPAWN_ONLY` with each |
| `ecs_view_cost.c` | problem 3's timed models 0 to 4 |
| `closure_name.c`, `closure_name.py`, `closure_name_output.txt` | problem 4: `-DREADING=1` to `4`; the Python twin |
| `share_weak.c` | problem 5's timed models 0, 3, 8, 9 |
| `backlink_filter.c`, `backlink_filter.py`, `backlink_filter_output.txt` | problem 6: `-DAS_REMOVE` and the fix; the Python twin |
| `run.ps1`, `medians.py`, `runs/*.txt`, `runs/*_summary.txt` | the timing runner and every run (`share_weak_a`, `_b`; `w3_beside_a`, `_b`; `ecs_view_a`, `_b`) |

Commands (Git Bash, in that folder; AddressSanitizer runs need
`PATH="/c/Program Files/LLVM/lib/clang/22/lib/windows:$PATH"`):

```text
"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address [-DSAVED] narrow_weak.c -o narrow_weak_{as_written,saved}.exe
"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address [-DSAVED|-DRACE] threads_side.c -o threads_side_{as_written,saved,race_asan}.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DRACE threads_side.c -o threads_side_race.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 [-DPOISON] w3_beside.c -o w3_beside[_poison].exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 [-DSAVED] [-DSPAWN_ONLY] ecs_view_group.c -o ecs_view_group_....exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n ecs_view_cost.c -o ecs_view_cost_n.exe       # n = 0..4
"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address -DREADING=n closure_name.c -o closure_name_n.exe   # n = 1..4
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=n share_weak.c -o share_weak_n.exe           # n = 0, 3, 8, 9
"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address [-DAS_REMOVE] backlink_filter.c -o backlink_filter[_as_remove].exe
C:\Python314\python.exe -I narrow_weak.py          # and threads_side.py, closure_name.py, backlink_filter.py
powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 -Out runs\share_weak_a.txt -Rounds 5 -Exes share_weak_0.exe,share_weak_3.exe,share_weak_8.exe,share_weak_9.exe
powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 -Out runs\w3_beside_a.txt -Rounds 5 -Exes ..\salvage\bin\plain_w3.exe,..\combined\w3.exe,w3_beside.exe
powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 -Out runs\ecs_view_a.txt -Rounds 5 -Exes ecs_view_cost_0.exe,ecs_view_cost_3.exe,ecs_view_cost_4.exe,ecs_view_cost_1.exe,ecs_view_cost_2.exe
C:\Python314\python.exe -I medians.py runs\<file>.txt <baseline exe>      # the batches marked b ran in reverse order
```

The debug-information files of the AddressSanitizer builds were deleted after the runs to keep the folder
small (8.8 MB).
