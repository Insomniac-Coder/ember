# Phase 3b: attacking and saving the threads group's answers (A15, A33, A19, A14, A20)

Written 2026-10-24 for the owner. This attacks the answers in `phase3b-threads.md` (called "the
answers report" below): threads (A15), references out of a lock (A33), global variables (A19), async
and generators (A14), and C callbacks and `unsafe` (A20). For every problem I found I tried to save the
answer first, measured the saved version where it costs anything, and attacked it again.

Nothing in `compiler/`, `std/`, `docs/` or `tests/` was changed. My programs are under
`target/redesign2/threads/attack/`: Ember programs in `probes/`, Rust in `rust/`, Python in `python/`,
timing programs in `bench/`. No Ember program was run (only `ember check`).

---

## 0. The short version

Four of the answers would have let a program the rules accept use freed memory or race. All four are
saved with compile-time rules or with checks that cost about what C or Rust pays for the same work. One
answer the group discarded (each task changing its own class object) is saved: it now costs within the
noise of plain C instead of 1.44x to 1.78x. Two speed claims were wrong and are corrected.

| Annoyance | Answer (what it does) | Verdict | What changed |
|---|---|---|---|
| A15 | Tasks in a scope are checked against each other from their effect summaries ("the scope rule") | **saved** | A handle taken out of a lock field could outlive its object (`y01`). Fixed: a handle read through any lock leaves it only as a counted handle, and everything stored into a lock is marked shared. Tasks may also keep handles to the parent's objects in their results, paying Rust's `Arc` price (1.10x and 1.26x of no counts, measured). |
| A15 | `total += n` and `results.push(x)` in tasks become per-task accumulators ("reductions") | **saved** | A reduction written inside a function the task calls cannot work as described (`y06`). Fixed: reductions only in the task's own body; then they compile to a C local (measured 1.00x; through a pointer it would be 1.61x). Nested spawns get a fixed combine order. |
| A15 | A `lock` field guards the other fields; Python's `with self.lock:` compiles as written | **saved** | Writing a field in `init` after `self` was shared races (`y02`); today's `[THR-1]` refuses it and the answer dropped that clause. Restored. A helper called with the lock held was refused (`y05`); now the helper's summary says "needs the lock" and callers are checked. |
| A15 | One bit in the count marks an object shared with another thread; only those objects pay atomic counts ("the shared flag") | **saved** | One object inside two locks, written under each, raced under one reading of the rule (`y03`); the rule is made exact (Rust refuses the same program, `r05`). "Configure, then share" was refused (`y04`); now allowed. The atomic path gets the overflow check it lacked. The marking walk is on par with Rust's way of building a shared graph (1.03x against 1.07x). |
| A15 | Each task changes its own class object (discarded by the group as "claimed objects", 1.73x) | **saved** | A count of 1 already proves no other handle exists; only objects with a higher count take a claim. Measured 0.96x to 1.23x of plain C in four batches, against 1.44x to 1.78x for the discarded version; no lost update with duplicates (thousands without the check). |
| A33 | A reference taken through a lock lives no longer than the lock; what leaves is a copy | **saved** | The copy of a class handle was said to be safe "because anything inside a lock is shared"; that was not true for lock fields. Fixed by the A15 rule above. The guard's limit itself: no hole found. |
| A33 | An accessor that keeps the lock while the caller uses the part | **saved** | Its speed claim (0.55x of a copy) holds only for a very short use. With a longer use it is 1.74x and 7.1x of a copy, measured: the lock is held for the caller's whole use. The claim is corrected and a warning is added. |
| A19 | Module variables with Python's spelling (`FRAME = 0`, `global FRAME`) | **saved** | A constant read by a worker thread was refused (`y07`; today's `static` and Rust allow it); now allowed. Module variables whose start value is not known when compiling paid an out-of-line call on every use (1.49x, measured) and ran at first use, not at start-up as in Python; now they are set at start-up in dependency order. |
| A19 | The 0.55x "emit module variables through a base pointer" recommendation | **discarded** | It is a CPU effect on one variable changed on every call, not a property of the layout: a 6x difference on a bare chain of adds, 1.00x when the function does 16 multiplications of other work (measured). |
| A14 | Loans into a generator's own heap contents may live across `yield` | **saved** | A local `Map` or `Set` iterated in a generator stays refused, because their iterators point at the map's header, which moves with the frame (`x02`, today's E2221). Fixed in the standard library: those iterators point at the entries' heap buffer, as `Chars` already points at the string's bytes. |
| A14 | A generator consumed in place may walk a field of a class object | **kept** | No hole found. |
| A14 | Async tasks live in task groups; their frames never move | **saved** (design only) | A started future that is stored or passed to something like `select` can move after it has started. Loans of inline locals across `await` are allowed only in task frames and in futures awaited directly. |
| A20 | Checked trampolines: C holds a slot number and a generation, not an address | **saved** | Three holes: a callback C runs on its own thread can be unregistered under it (shown: the callback wrote into freed state); a user-data value from another callback runs this trampoline on the wrong closure; a growing table can move slots under a running call. Fixed: the calling thread announces the slot with two plain stores and unregistering waits, 1.46x of plain C on an empty callback, against 16.8x for an atomic count; the type is checked in the same compare as the generation (1.05x, against 1.04x without it); the table never moves. |
| A20 | Callback contracts inferred from the C header | **kept** | No hole found; with the checks above a wrong guess panics or runs a closure of the right type. |
| A20 | A reference made from a raw pointer lives no longer than its `unsafe` block | **saved** | As written, a safe wrapper could not return a reference into memory it owns (`y13`); Rust can (`r06`). The reference now borrows the place the pointer was read from. |

All timings below are PROVISIONAL: other agents ran at the same time, and the spread between runs was
large (for one program, 62 to 89 ms). Where a verdict would rest on a difference under 15%, I say so;
it waits for the final measurer.

---

## 1. Words used in this report

- **Thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
  **Task**: a function handed to another thread to run.
- **Scope**: `with scope = thread.scope():`; tasks spawned in it may use the parent's local variables,
  because the block does not end until every task has finished (the **join**). An **unscoped thread**
  (`thread.spawn`) may outlive the function that started it.
- **Data race**: two threads touch one memory location at the same time, at least one writes, and
  nothing orders them. In C and Rust this is undefined behaviour; in practice it loses updates, or frees
  memory that is still in use.
- **Count**: the number of handles to a class object; the object is freed when it reaches zero.
  **Handle**: what a variable of a class type holds (a counted pointer). **Atomic** operation: a
  read-modify-write that two threads cannot interleave; several times dearer than a plain one.
- **Lock** and **guard**: `with g = m.lock():` waits until no other thread holds `m`, gives `g`, and
  releases `m` at the block's end. A **lock field** is the answers report's `lock: Lock` field in a
  class, used as `with self.lock:`.
- **Effect summary** (summary): what the compiler records about a function from its body: which places
  it reads, writes and **reshapes** (changes in a way that can free or move memory). For class objects
  the place is "field `hp` of some `Player`", because two handles can name one object.
- **View**: a reference, or a value holding one, that the compiler limits to the life of what it
  points into. A **handle view** is a handle the compiler did not count, because it can prove the
  object outlives it.
- **Reduction**: per-task results combined after the join (`total += n` from several tasks).
- **The shared flag**: the answers report's bit at the top of an object's count, set when the object
  first becomes reachable from another unscoped thread; flagged objects get atomic counts. The **walk**
  sets it on every object reachable from the shared one.
- **Frame**: the struct a generator is compiled into (its state and the locals that live across a
  `yield`). **Heap contents**: the bytes of a `String`, the elements of an `Array`; they live in a separate
  heap buffer, so they stay put when the frame moves. **Header**: the part of a `String`, `Array` or `Map`
  stored in the frame itself (pointer, length, capacity).
- **Trampoline**: the small function Ember generates for a C library to call; it finds the Ember
  closure and calls it. **User data**: the `void*` a C library hands back to its callback. **Slot**,
  **generation**: the answers report's table entry for a registered callback, and the number in it
  that changes when the callback is unregistered.
- **Store-to-load forwarding**: the CPU handing a value just stored to a following load of the same
  address before the store reaches the cache. **False sharing**: two threads writing different
  variables that share one 64-byte cache line, which the CPU then moves back and forth between cores.
- **Kinds (a), (b), (c)**: the job's three acceptable answers: a Pythonic spelling of Rust's guarantee;
  a smarter compile-time solution; a new mechanism.

---

## 2. How I checked

- **Ember today**: `target/ci/ember.exe check` (`ember 0.1.0`, `language: 0.9.10`). Output:
  `attack/probes/ember_check_output.txt`. Programs named `x*.em` are today's syntax and were checked;
  programs named `y*.em` use the answers report's proposed syntax, which today's compiler cannot
  check, so they are attack programs to read, not compiler output.
- **Rust**: rustc 1.98.1, `rustc -C opt-level=2 --edition 2021 --emit=metadata`; output
  `attack/rust/rustc_output.txt`.
- **Python**: `python -I attack/python/py_attack.py` (Python 3.14.2); every behaviour quoted below is an
  `assert` in that file; output `py_attack_output.txt` ("all Python checks passed").
- **Timing**: `attack/bench/run_group.sh <batch> <base> <programs...>` calls the ownership group's
  `run.ps1` (mains power checked before every round: `(Get-CimInstance Win32_Battery).BatteryStatus`
  was 2, and no round aborted; 5 rounds, each running every program once, one at a time, in the given
  order, pinned with `start /affinity C03C03`, the 8 P-cores of the Core Ultra 9 275HX) and its
  `medians.py` (median of 5, ratio to the base). Raw lines `bench/runs/<batch>.txt`, tables
  `bench/runs/<batch>_summary.txt`. C is `clang -O2` (22.1.8); every build line is in `bench/build.sh`.
  Every program of a workload printed the same checksum on every run.
- **All timings are PROVISIONAL.** Other agents were running. The same plain C program took 58.6 ms in
  one batch and 83.3 ms in another (`claim2_b2_summary.txt`, `claim2_summary.txt`).

---

## 3. A15: threads

### 3.1 The scope rule (answers report 4.3): saved

**What it says.** Inside a scope, every task's effect summary is checked against every other task's
and against the parent's code until the join; a write that meets another task's read or write of the
same place is a compile-time error. A handle a task reads from the parent's data is a view: the task
never changes the count of an object it did not make. Operations on locks and lock-guarded fields are
not conflicts.

#### Attack 1: a handle read out of a lock field outlives its object (memory safety)

```ember
# y01_lockfield_view.em (proposed syntax)
class Player:
    name: String = "ann".to_string()

class Board:
    lock: Lock = Lock()
    best: Player = Player()

fn run(board: Board):
    with scope = thread.scope():
        scope.spawn(fn():
            with board.lock:
                p = board.best              # a view: no count (rule 4)
            thread.sleep_ms(10)
            println(p.name))                # used after the lock
        scope.spawn(fn():
            with board.lock:
                board.best = Player(name="bob".to_string()))   # releases "ann", its last handle
```

```python
# py_attack.py, check 7: the handle keeps "ann" alive; Python prints "ann"
with b.lock:
    best = b.best
with b.lock:
    b.best = Player("bob")
print(best.name)          # ann
```

The answers report gives three rules that disagree here:

- Rule 4 (4.3): `p` is a view of a parent object, so it holds no count.
- Rule 3 (4.3): writes to lock-guarded fields are not conflicts, so task 2 may replace `board.best`.
  It releases "ann"; that was the last counted handle, so "ann" is freed while task 1 still holds `p`.
  Rule 2 does not catch it: task 2 writes "the count of some `Player`" and task 1 reads
  "`name` of some `Player`", two different places.
- Section 5.3 rule 2 says a class handle leaves a lock "as a retain (atomic: anything inside a lock is
  flagged)". If that rule applies, `p` is counted and the program is safe; but the claim it rests on is
  false for a lock *field*: stores into a lock field of an object that no unscoped thread has seen do
  not set the flag (4.6 only flags stores into objects already shared). Then the retain and release are
  plain, and task 1's release after the lock races with task 2's changes under the lock.

So, depending on which rule wins, the program reads freed memory, races on a count, or (rule 5, "a
reference to a guarded field does not leave the `with` block", if it is read as covering handles) is
refused although Python runs it.

**Saving it.** Three rules, none with a cost on code that does not use locks:

1. **A handle read through a lock leaves it only as a counted handle.** Through a `Mutex` guard or a lock
   field alike: used inside the `with` block it is a view (no count); kept past the block it is retained
   while the lock is held.
2. **Everything stored into a lock is flagged at the store.** A `Mutex`'s value or a lock field, whether
   or not the object holding it has been shared. Then "anything inside a lock is flagged" is true by
   construction, the retain in rule 1 is atomic, and the compiler knows it statically (a handle read
   through a lock is flagged), so it emits the atomic instruction with no test.
3. **What the object's own fields may do** is the shared flag's rule, made exact in section 3.4.

Cost: the walk when an unflagged object is stored into a lock (the answers report measured 20.8 ns per
object on a cold, scattered graph; section 3.4 measures a fresh one), and an atomic count pair for a
handle that leaves a lock, which is what Rust's `Mutex<Arc<T>>` pays for the same program.

**Attacking the saved version.** Task 1 now holds a counted handle to "ann"; task 2's release brings the
count from 2 to 1; "ann" lives until task 1 drops `p`. Both changes are atomic (rule 2), so they do not
race. Reading `p.name` after the lock: `name` is never written after `init`, so the share rule allows it.
If `Player.name` were written somewhere, `y01` would be refused at the store into the lock field (section
3.4), as Rust refuses writing through an `Arc`.

#### Attack 2: a scope handed to a function

```ember
# x01_scope_param.em (today's syntax)
fn start_reader(scope: thread.Scope, xs: Array[int]):
    scope.spawn(fn(): println(xs[0] + xs[1]))       # the task outlives start_reader

fn main():
    xs = [1, 2, 3]
    with scope = thread.scope():
        start_reader(scope, xs)
        xs.push(4)                                    # grows xs while the task may read it
```

```python
from concurrent.futures import ThreadPoolExecutor
def start_reader(ex, xs):
    ex.submit(lambda: print(xs[0] + xs[1]))
xs = [1, 2, 3]
with ThreadPoolExecutor() as ex:
    start_reader(ex, xs)
    xs.append(4)
```

If this compiled, the parent's `xs.push(4)` would run while the task reads `xs`, but the scope rule only
treats "the parent's statements from each spawn" as concurrent, and the spawn is inside the callee.
Today it does not compile: `error[E2072]: std.thread.Scope has no method named spawn`
(`ember_check_output.txt`). **Requirement added**: keep that, or put the spawned task's effects in the
callee's summary as "running until the scope ends", so the caller's later statements are checked
against them. No cost either way. (The message itself is misleading: section 9.)

#### Attack 3: tasks making result objects that point at their inputs (ease)

```ember
# y12_results_point_at_inputs.em (proposed syntax)
fn score_all(players: Array[Player]) -> Array[Match]:
    results: Array[Match] = []
    with scope = thread.scope():
        for p in players:
            scope.spawn(fn(): results.push(Match(player=p, score=p.name.len())))
    return results
```

```python
# py_attack.py, check 6: every match points at its input object
with ThreadPoolExecutor(4) as ex:
    matches = list(ex.map(lambda p: Match(p, len(p.name)), players))
```

As written this is refused: storing `p` in a new `Match` is a retain of a parent object, a write to
"the count of some `Player`", and two tasks do it. The answers report lists it as a residue ("a task
cannot keep a parent's object past the scope"). Rust accepts it with a lifetime on `Match`
(`r04_results_borrow_inputs.rs`, `Match<'a>` holding `&'a Player`, compiles).

**Saving it** (rule 4b): in a task's own code, a retain of a parent object is emitted as an atomic
increment, and is not a conflict, as long as nothing else changes counts of that class during the
scope with plain instructions: the parent's statements, other tasks' shared code (a `drop` that
releases a `Player`), and the shared flag's walk all count as plain count writes. A task may not release
a parent object it retained (the result must leave by a reduction or a lock); that is a compile-time
error naming the release. After the join the counts are exact, because the join orders every task's
atomic increments before the parent's next plain operation.

Measured (`bench/matches.c`; 1,048,576 parent objects, 8 tasks, one 24-byte `Match` per handle, the
parent appends the task lists in spawn order, uses and frees the matches; 16 rounds;
`runs/matches_summary.txt`, `runs/matches_hot_summary.txt`):

| Program | What it is | median ms | ratio |
|---|---|---|---|
| `matches-0-plainC` | no counts (also Rust's `&'a Player` version, which has none) | 1103.641 | 1.00 |
| `matches-1-atomic-in-task` | rule 4b: the task retains with an atomic increment (Rust's `Arc::clone`) | 1217.819 | 1.10 |
| `matches-2-deferred-at-join` | no count in the task; the parent retains each player while it appends | 1341.307 | 1.22 |
| `matches-0-plainC-hot` | every match points at one of 8 players | 419.046 | 1.00 |
| `matches-1-atomic-in-task-hot` | rule 4b, 8 hot objects (8 threads incrementing 8 counts) | 528.472 | 1.26 |
| `matches-2-deferred-at-join-hot` | deferred, 8 hot objects | 468.089 | 1.12 |

Rule 4b costs what Rust's `Arc` costs for the same program; it is not as cheap as Rust's borrowed
version, which has no count at all. Ember can reach that too, with a view field
(`@view struct Match: player: ref Player`, the references group's "an object holding a view carries the
view's limit"), at the price of writing `ref`. The deferred variant is not chosen: slower when the
results point at many objects (1.22x), faster only when they point at a few (1.12x against 1.26x).

**Verdict for the scope rule: saved** (lock rules 1 and 2, the scope-parameter requirement, rule 4b).

### 3.2 Reductions (answers report 4.4): saved

**What it says.** In a scope, a captured place that tasks only change with `+=`, `push` and similar,
and that nobody reads until the join, gets one accumulator per task, combined at the join in spawn
order. It says this compiles to exactly what a C programmer writes: a local counter per thread.

#### Attack 1: a reduction inside a function the task calls

```ember
# y06_reduce_in_helper.em (proposed syntax)
WORDS = 0

fn add_words(t: String):
    global WORDS
    WORDS += t.split_whitespace().count()

fn count(texts: Array[String]):
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): add_words(t))
    add_words("one more".to_string())
```

```python
# py_attack.py, check 9 (with a lock, as Python needs): WORDS == 6
def add_words(t):
    global WORDS
    with words_lock:
        WORDS += len(t.split())
```

The answers report says module variables changed by tasks are reductions (6.3 rule 4, 6.5).
`add_words` is compiled once. Called from `main` it must add to `WORDS`; called from a task it must
add to that task's accumulator, which it cannot name. Compiled as written, it adds to the global from
eight threads: a data race. The only ways to make it work are two compiled copies of every such
function (impossible for a function in a package compiled separately), or an accumulator passed by
pointer (or found through a thread-local pointer). The pointer version is what the cost claim then
rests on, so I measured it (`bench/reduce2.c`, the answers report's workload: 64 MiB of text, 8 tasks,
`+= 1` per word; `runs/reduce2_summary.txt`):

| Program | Where the accumulator is | median ms | ratio |
|---|---|---|---|
| `reduce2-0-local-accumulator` | a C local in the task's function, stored once at the end (today's `@parallel(reduce=...)` does this: the chunk function returns its partial, `emit_parallel_helpers` in `compiler/ember_codegen_c/src/lib.rs`) | 47.693 | 1.00 |
| `reduce2-3-through-pointer-packed` | in the task record, reached through a pointer, records side by side | 76.957 | 1.61 |
| `reduce2-4-through-pointer-padded` | the same, each record on its own cache line | 51.054 | 1.07 |

Through a pointer the compiler keeps the counter in memory (the text's byte loads might alias it:
`inc qword ptr [r8]` per word, `bench/asm/reduce2_3.s`), and records side by side share cache lines
(false sharing): 1.61x.

**Saving it.** A reduction is recognised only when every `x += e` (or `push`) is written in the task's
own body. Then the accumulator is a C local, as today's `@parallel` reductions are (1.00x). A call
from a task to a function whose summary writes the variable is a conflict, with the fix-it of returning
the amount: `scope.spawn(fn(): WORDS += count_words(t))`. No cost. What it gives up: Python's helper
with `global WORDS` needs that one-line change in a task.

#### Attack 2: "deterministic" is not true with nested spawns

A task that spawns into the same scope (`scope.spawn` inside a task) makes the spawn order depend on
timing, so "combined in spawn order" gives a list whose order changes between runs. Not a memory
problem. **Fix**: combine in spawn-tree order (the parent's spawns in order, each task's own spawns right
after it, in their order). No cost.

Other attacks found nothing: reading `total` inside a task is refused; `x = max(x, e)` on floats with
NaN gives the same result per task and combined (Python's `max(a, b)` keeps `a` unless `b > a`, so a NaN
never replaces a value); integer overflow can panic in a partial where the serial sum would not (the
answers report says so).

**Verdict: saved** (reductions only in the task's own body; spawn-tree order).

### 3.3 The lock field (answers report 4.5): saved

**What it says.** In a class with a `lock: Lock` field, every other field written after `init` may be
touched only inside `with x.lock:` on the same handle, once the object is shared. Fields never written
after `init` need no lock.

#### Attack 1: a field written in `init` after `self` was shared (memory safety)

```ember
# y02_init_escape.em (proposed syntax)
class Worker:
    name: String = "".to_string()

    fn init(self, name: String):
        thread.spawn(fn(): println(self.name))   # self is shared here
        self.name = name                         # a write in init: no lock needed, says the rule
```

```python
# py_attack.py, check 1: the thread reads "" or "bob"; Python never crashes
class Worker:
    def __init__(self, name):
        self.name = ""
        t = threading.Thread(target=lambda: print(self.name)); t.start()
        self.name = name
```

`name` is a `String`: a pointer, a length and a capacity. The thread can read the new pointer with the
old length, or the old pointer after `self.name = name` freed the old buffer: an out-of-bounds read or a
use after free. Today's `[THR-1]` refuses this: "inside `init`, a field assignment after `self` has been
used as a whole is `E7003`". Checked: `x04_sync_init_escape.em` gives
`error[E7003]: cannot write field name of an @sync class after init or after using self as a whole`.
The answers report replaces `[THR-1]` and does not keep that clause.

**Saving it.** Keep the clause: for the lock rule and the share rule, `init` ends at the first use of
`self` as a whole (passing it, capturing it, storing it). No cost.

#### Attack 2: a helper called with the lock held (ease)

```ember
# y05_helper_under_lock.em (proposed syntax)
class Stats:
    lock: Lock = Lock()
    words: int = 0

    fn _bump(self):
        self.words += 1            # summary: "unguarded write of Stats.words"

    fn add(self):
        with self.lock:
            self._bump()
```

```python
# py_attack.py, check 3: 8 threads, words == 8
def _bump(self): self.words += 1
def add(self):
    with self.lock:
        self._bump()
```

The rule records per function whether its accesses were guarded; `_bump`'s are not, so `Stats` may not
be shared, though every call holds the lock. Rust accepts the same shape (`r02_helper_under_lock.rs`:
`fn bump(s: &mut Inner)` called with the guard).

**Saving it.** A function that touches a guarded field of its parameter `x` without a `with` gets the
precondition "needs `x.lock` held" in its summary (what Clang's thread-safety annotations write by hand
as `REQUIRES(mu)`, here inferred); each caller must hold that lock on the argument it passes, checked the
same way as the `with` rule. No cost.

**Attacking the saved version.** A caller that passes a different object (`with a.lock: b._bump()`) is
refused (it holds `a.lock`, not `b.lock`); a closure that reassigns the variable inside the block is
caught because the closure's summary writes the variable; re-taking the lock inside `_bump` is the
existing self-wait error. A deadlock through a call whose effects are unknown is still possible; it is
not a memory error (the runtime's lock on Windows is an SRW lock, `runtime/ember_rt/include/ember_rt.h`,
and elsewhere its own futex lock; taking either twice on one thread waits forever, it does not corrupt
memory).

**Verdict: saved** (the `init` clause kept; inferred "needs the lock" preconditions).

### 3.4 The shared flag (answers report 4.6): saved

**What it says.** Objects start with plain counts. When an object first becomes reachable from another
unscoped thread, a walk sets the top bit of the count of every object reachable from it; flagged objects
use atomic counts. A class object shared with an unscoped thread may have no plain field written after
the share: "every write to `C.f` outside `init` must be guarded by a lock field or be an
`Atomic`/`Mutex` operation".

#### Attack 1: one object inside two locks (memory safety, under one reading)

```ember
# y03_two_mutexes.em (proposed syntax)
p = Player()
a: Mutex[Array[Player]] = Mutex([p])
b: Mutex[Array[Player]] = Mutex([p])
t1 = thread.spawn(fn():
    with g = a.lock():
        g[0].hp -= 1)              # "under a Mutex"
t2 = thread.spawn(fn():
    with g = b.lock():
        g[0].hp -= 1)              # under another Mutex: one field, no common lock
```

```python
p = Player(); a = [p]; b = [p]; la = threading.Lock(); lb = threading.Lock()
def t1():
    with la: a[0].hp -= 1
def t2():
    with lb: b[0].hp -= 1          # Python runs it (its interpreter lock serialises the two)
```

If a write made while holding *some* `Mutex` counts as "a `Mutex` operation", both writes pass and race.
A `Mutex` guards the value it holds (here a list of handles), not the objects those handles point to.
Rust refuses the program: `r05_object_in_two_mutexes.rs` gives
`error[E0594]: cannot assign to data in an Arc` on both writes.

**Saving it** (the exact rule):

- **Unscoped sharing**: a class object reachable from a lock, a channel or a thread's captures is
  shared; its fields must be never written after `init`, or guarded by its own lock field. A `Mutex`'s
  guard covers the `Mutex`'s value only. This is Rust's rule for `Mutex<Vec<Arc<Player>>>`.
- **Scoped tasks**: writes through a `Mutex` guard to fields of objects inside it are allowed when every
  access to that field in the scope goes through the same `Mutex` (the scope rule sees all of them). So
  `with g = m.lock(): g[0].hp -= 1` from several scoped tasks compiles.

Fix-its for the unscoped case: make `Player` a struct (then the list holds values and the `Mutex` guards
them), or give `Player` a lock field. No cost.

#### Attack 2: configure, then share (ease)

```ember
# y04_configure_then_share.em (proposed syntax)
cfg = Config()
cfg.scale = 3                                  # a write outside init
worker = thread.spawn(fn(): println(cfg.scale))
```

```python
# py_attack.py, check 2: the thread reads 3
cfg = Config(); cfg.scale = 3
t = threading.Thread(target=lambda: print(cfg.scale)); t.start()
```

"Every write to `C.f` outside `init`" is checked for the class over the whole build, so this one write
makes `Config` impossible to share. Rust accepts it (`r01_configure_then_share.rs`: write, then
`Arc::new`).

**Saving it.** A write through a handle the compiler can prove is not yet shared does not count: the
object was made in this function and has not yet been stored anywhere but local variables, passed to a
function whose summary may keep it, or shared. The share is the first such escape, and the thread
start orders the earlier writes before the thread's reads. No cost.

#### Attack 3: speed claims

- **The atomic path has no overflow check.** In `counts.c` model 6 the flagged retain is a bare
  `lock xadd`; a flagged count past `0xFFFFFFFF` wraps to 0 and loses the flag, and the next release
  frees a live object. Ember's `[RC-1]` requires an overflow panic, and Rust's `Arc` aborts when the old
  count is past its limit. Fixed in `bench/counts2.c` model 7 (compare the old value, panic at
  `0xFFFFFFF0` or above). Measured with a retain and release on every step, every object shared
  (`runs/counts2_summary.txt`):

  | Program | median ms | ratio |
  |---|---|---|
  | `counts-4s-atomic-always` (Swift's way, Rust's `Arc`) | 392.372 | 1.00 |
  | `counts-6s-flag-shared-inline` (the answers report's saved version) | 482.737 | 1.23 |
  | `counts-7s-flag-shared-inline-ovf` (with the overflow check) | 527.783 | 1.35 |

  The check adds 9% in this worst case (under 15%: waits for the final measurer). The answers report
  had 1.15x for model 6 against always-atomic; this batch gives 1.23x.

  **Saved further, at no cost:** where the compiler knows a handle came from a share (read through a
  lock, which rule 2 of section 3.1 makes always flagged; captured by an unscoped thread; received from a
  channel; read from a `static`), it emits the atomic instruction directly, with no test: that is
  `counts-4s`'s code, 1.00x of Rust's `Arc`. The tested path stays only where the compiler cannot tell.
- **The release test assumes one flag bit.** The answers report's release fast path is
  `(int32_t)count > 1`, a sign test. The ownership group folds more flags into the top bits of the same
  word (`phase3b-ownership.md` 2.3). With a second flag bit set and the shared bit clear, the count is
  positive and large, so the fast path decrements it and never sees it reach 1: the object leaks, or with
  a different bit layout is freed early. **Fix**: an unsigned range test on both paths, as retain already
  does (`count - 2 < LIMIT - 2`), one more `lea`; not measured.
- **The walk against Rust's way.** Rust builds a graph it will share out of `Arc`s from the start, an
  atomic increment per link; the flag builds with plain increments and walks once at the share. Measured
  (`bench/walkbuild.c`: 262,144 objects, each linked to a random earlier one, built, shared, freed, 32
  rounds; `runs/walkbuild_summary.txt`):

  | Program | median ms | ratio |
  |---|---|---|
  | `walkbuild-0-plain-noshare` (never shared) | 642.045 | 1.00 |
  | `walkbuild-1-plain-then-walk` (the flag) | 661.969 | 1.03 |
  | `walkbuild-2-atomic-links` (Rust's `Arc`) | 688.396 | 1.07 |

  On par (both under 15% of each other): no change needed.
- **Every store into a lock must set the flag before the stored handle becomes visible** (before the
  lock is released); otherwise another thread can take a still-plain handle out. An ordering
  requirement, no cost.

**Verdict: saved** (the exact rule for objects inside locks; configure-then-share; the overflow check;
direct atomics for handles known to be shared; range tests on both paths).

### 3.5 Each task changes its own class object (answers report 4.7, discarded there): saved

**What it says.** The group discarded "claimed objects": before a task writes an object it takes a
claim byte with an atomic exchange, because the list may hold one object twice. Measured 1.73x of plain
C; its reason for discarding: "one atomic instruction per object per pass is the minimum to detect two
tasks on one object", and proving distinct objects at compile time "needs no other handle to a `Player`
exists, which the lean class cannot know in general".

```ember
# y11_parallel_class_objects.em (proposed syntax)
@parallel
for p in players:            # players: Array[Player], Player a class; the list may hold one object twice
    p.hp -= 1
    p.x += p.hp & 1
```

```python
# py_attack.py, check 10: one object twice in the list; both changes land (hp 10 -> 8)
with ThreadPoolExecutor(4) as ex:
    list(ex.map(hurt, lst))
```

**Why the reason does not hold.** The object's count already says whether another handle exists. A
count of 1 means the list's slot holds the only counted handle, so no other slot of this list holds the
object, and no other task can reach it to write it: other paths to `Player` fields in the tasks are
compile-time conflicts (writes to "`hp` of some `Player`" anywhere but through the loop element). The
count is on the cache line the task writes anyway. Only objects with a count above 1 need a claim.

**The saved mechanism.** Each task reads the element's count (a plain load, which on x86 is also a
relaxed atomic load). 1: write with plain instructions. Above 1: take the object's claim with an atomic
exchange, **waiting** while another task holds it (a body touches only its own element, so claims never
nest and cannot deadlock), then release it. Waiting instead of panicking gives Python's result when the
list holds one object twice.

Why it is sound: during the scope no thread changes these counts with plain instructions (the scope
rule's rule 4: tasks do not retain parent objects; the parent may not, because it would conflict with the
tasks' reads of "the count of some `Player`"; a share's walk counts as a write). The list must be an owned
list (its slots counted); a list of views gives no such guarantee, and then every element takes the
claim. An object shared with an unscoped thread has the flag set, so its count is never 1 (and its class
cannot have unguarded writes anyway). The claim bit can live in the count word itself (bit 30), because
nothing else changes these counts during the scope; that halves the count's range again (to 2^30 - 2).
The models below keep a separate claim byte on the same cache line; the instructions are the same.

Measured (`bench/claim2.c`, the answers report's workload: 1,048,576 shuffled handles, 8 tasks, 64
passes; models 0 and 1 are the answers report's; `claim2-7` is the count test with the list's buffer
in a local and the claim path out of line, `bench/asm/claim2_7.s`; `-8` is the same with one object in
16 counted twice):

| Batch (summary file) | plain C ms | count test (7) | count test, 1 in 16 claimed (8) | atomic claims (1) |
|---|---|---|---|---|
| `claim2_b4_summary.txt` | 70.592 | 1.00 | 1.23 | 1.44 |
| `claim2_b5_summary.txt` | 67.876 | 1.23 | 1.39 | 1.56 |
| `claim2s_summary.txt` (8 sweeps per pass) | 550.601 | 0.96 | 1.17 | 1.63 |
| `claim2s_b2_summary.txt` (8 sweeps per pass) | 515.070 | 1.18 | 1.35 | 1.78 |

Earlier batches with the first, looser version of the count test (`claim2-3`, which reloads the list
pointer on every step): 1.18x, 1.14x, 1.19x, 1.06x, 1.13x, 1.05x, 1.33x, against 1.58x, 1.54x, 1.65x for
atomic claims (`claim2_summary.txt`, `claim2_b2`, `claim2_b3`, and the same later batches).

The machine was noisy: plain C ranged 60.8 to 84.2 ms inside one batch (`claim2_b5_summary.txt`). The
count test is within the noise of plain C (0.96x to 1.23x); whether it is within 15% waits for the final
measurer. Its gap to the atomic claims is above 15% in every batch. The code it adds to the loop is one
load from the line the body writes, a compare and a branch (`asm/claim2_7.s`).

Correctness (`claim2-5` and `claim2-6`: one slot in 16 re-pointed at an object another task's part also
holds; `runs/claim2_dups.txt`): with the count test, 0 decrements lost in each of 3 runs; with no check,
2,988, 4,528 and 6,768 lost.

**Verdict: saved.** The answers report's residue "class objects written by several tasks need a lock
or must be structs" is removed for the loop-element case (`@parallel for p in players`,
`for p in players.iter_mut(): scope.spawn(...)`). Good part of the discarded version, kept: the waiting
claim is its slow path.

---

## 4. A33: references out of a lock

### 4.1 A guard's limit, and a copy of an immutable value (answers report 5.3): saved

**What it says.** A reference read through a guard lives no longer than the guard (this closes today's
hole, which the answers report re-checked: `a33_lock.em` and three more compile today). What leaves a
lock is a copy (`return g[0]` of a `String`) or, for a class handle, a retain "atomic: anything inside a
lock is flagged". A list or struct does not leave silently.

```ember
fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0]              # a copy leaves the lock
```

```python
def top(scores):
    with lock:
        return scores[0]
```

**Attacks.** On the guard's limit: returning a closure that captured a view from the guard, returning
an iterator over `g`, or storing `g[0]` into an object, are all views carrying the guard's limit under the
references group's rules (saved by its attacker), so all are refused. No hole found.

On the copy: the class-handle half rested on "anything inside a lock is flagged", which was false for lock
fields (section 3.1, attack 1). With section 3.1's rules 1 and 2 it is true. Objects reached through a
handle inside the lock follow section 3.4's exact rule.

On the cost claim "0.88x of Rust's `.clone()`": the C models use the SRW lock and Rust's standard `Mutex`
is a different implementation (the answers report says so). Under 8-thread contention the lock dominates,
so the 0.88x is about the lock, not the answer. The fair statement is "the same code as C copying the same
way".

**Verdict: saved** (by section 3.1's rules; the cost statement corrected).

### 4.2 An accessor that keeps the lock while the caller uses the part (answers report 5.4): saved

**What it says.** `yield g[0]` inside `with g = scores.lock():`, used as `with name = top(board.scores):`;
the lock is released after the caller's last use of the part. Claimed 0.55x of the copy-out, "zero copies".

```ember
fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        yield g[0]

with name = top(board.scores):
    println(name)
```

```python
@contextmanager
def top(scores):
    with lock:
        yield scores[0]

with top(board.scores) as name:
    print(name)
```

**Safety attacks** (a part held across `yield` or `await`: refused by its rules; a call that takes the
same lock while the part lives: refused when the summary says so, a deadlock when the callee is unknown,
which is not a memory error, section 3.3; every exit runs the release, and a panic aborts): no hole found.

**Speed attack.** The accessor holds the lock for the caller's whole use. Under contention that serialises
the callers' work, so it wins only when the use is shorter than the copy. Measured with the answers
report's `lock.c`, unchanged except a longer use (`bench/lock2.c`, made from `lock.c` by `sed`; 8 threads,
one lock; "reps" is how many times the caller hashes the name; `runs/lock2_summary.txt`):

| Use (hashes of the name) | calls per thread | copy-out ms | accessor ms | accessor / copy-out |
|---|---|---|---|---|
| 4 (the answers report's case) | 2,000,000 | 2628.502 | 1402.476 | 0.53 |
| 16 | 1,000,000 | 1554.017 | 2698.602 | 1.74 |
| 64 | 250,000 | 539.612 | 3834.393 | 7.11 |

The answers report's 0.55x is reproduced (0.53x) and holds only for that very short use.

**Saving it.** The mechanism stays; the claim and the guidance change: copy-out is the default
spelling; the accessor is for a use that is short or must see the value while it is locked. Because
the accessor hides the lock from the caller (as Python's context manager does), the compiler warns when
the part of a lock-holding accessor stays live across a loop or a call that is not known to be short:
"the lock taken in `top` is held until line N; copy the value out if this work is long". A compile-time
lint, no cost.

**Verdict: saved** (claim corrected; warning added).

---

## 5. A19: global variables

### 5.1 Module variables with Python's spelling (answers report 6.3): saved

**What it says.** `FRAME = 0` at module level declares a module variable; `global FRAME` is needed to
assign it; tasks in a scope may use them like the parent's locals; a function reachable from an unscoped
thread may not touch any module variable (rule 5). Initialisers are evaluated when compiling where they
can be, otherwise on first access (`[STA-3]`). Claimed cost: "plain C globals: nothing is counted or
checked".

#### Attack 1: a constant read by a worker thread (ease; a step back from today)

```ember
# y07_module_constant_thread.em (proposed syntax)
DEBUG = true

fn main():
    worker = thread.spawn(fn(): println(DEBUG))     # refused by rule 5
    worker.join()
```

```python
# py_attack.py, check 4: the worker reads True
DEBUG = True
t = threading.Thread(target=lambda: print(DEBUG)); t.start()
```

Today the same program written with `static DEBUG: bool = true` compiles (`x03_static_read_thread.em`,
exit 0), and so does Rust (`r03_static_read_thread.rs`). Under the answers report's rule 5 the Python
spelling is refused.

**Saving it.** A module variable that is never assigned after its initialiser and never changed through
its name anywhere in the build (no `global X` assignment, no `X.push`, no write to its fields) is a
constant: it is today's `static`, readable from any thread. If it holds class handles, those objects are
flagged once at start-up. No cost.

#### Attack 2: initialisers that run on first use (speed, and Python's order)

The answers report keeps `[STA-3]`: an initialiser the compiler cannot evaluate (`CONFIG =
load_config()`, `START = time.now()`) runs at the first access. Today's C backend reaches such a static
through an accessor that calls the runtime's out-of-line `ember_rt_static_begin` on every access
(`lazy_static_definitions` in `compiler/ember_codegen_c/src/lib.rs`; `ember_rt_static_begin` is declared
out of line in `runtime/ember_rt/include/ember_rt.h`). So "nothing is checked" holds only for
initialisers known when compiling. Measured (`bench/globals2.c`, the answers report's `globals.c`
workload with the lazy accessor; `runs/globals2_summary.txt`):

| Program | median ms | ratio |
|---|---|---|
| `globals2-4-plain-base-pointer` (plain globals through a register, the same addressing as the accessor's result) | 103.376 | 1.00 |
| `globals2-6-lazy-accessor` (three accessor calls per `tick()`) | 153.920 | 1.49 |
| `globals2-0-plain-eager` (plain globals addressed from the instruction pointer) | 194.165 | 1.88 |

The last row is slower than the accessor for the reason section 5.2 isolates, which is why the fair
comparison is against the base-pointer row.

The order also differs from Python: Python runs module-level assignments when the module is imported,
in order, before `main` (`py_attack.py` check 8: `['A', 'B', 'main']`); `[STA-3]` runs `START =
time.now()` when something first reads `START`.

**Saving it.** Module variables are initialised at start-up, before `main`, in dependency order (which
initialiser reads which module variable is in the summaries; a cycle is a compile-time error naming it,
as Go does for package variables). Then every access is a plain global (no accessor), and side effects
happen in Python's order. `[STA-2]`'s "no initialisation-order problem" holds because the order is
computed and cycles are refused. Today's `static` keeps `[STA-3]`.

#### Attack 3: tasks changing module variables through a called function

Section 3.2, attack 1 (`y06`) is this answer's rule 4. Saved there: reductions only in the task's own
body.

**Verdict: saved** (constants readable by any thread; start-up initialisation in dependency order;
reductions only in task bodies).

### 5.2 "Emit module variables through a base pointer: 0.55x" (answers report 6.3 and 11.7): discarded

The answers report found plain C globals 1.8x slower than the same globals reached through a base
pointer (`globals-0` 192.219 ms against `globals-4` 105.666 ms), called the cause UNVERIFIED, and
recommended that "the C backend should emit a module's variables as one struct reached through a base
pointer". I isolated the cause (`bench/ripchain.c`; `runs/ripchain_summary.txt`; assembly in
`bench/asm/ripchain_*.s`):

| Program | What it is | median ms |
|---|---|---|
| `ripchain-0-loop-rip` | 1,000,000,000 × `inc qword ptr [rip + VG]` | 1197.070 |
| `ripchain-1-loop-base` | the same through a register: `inc qword ptr [rax]` | 197.700 |
| `ripchain-2-call-rip` | 300,000,000 calls of `tick()`: `inc qword ptr [rip + G]` | 455.225 |
| `ripchain-3-call-base` | the same, `tick()` loads a base pointer first | 121.364 |
| `ripchain-4-call-rip-work` | as 2, and `tick()` also does 16 multiplications on a local | 533.449 |
| `ripchain-5-call-base-work` | as 3 with the same work | 537.570 |

A chain of adds to one memory location runs about 1 cycle per add through a register and about 6 cycles
when addressed from the instruction pointer: on this CPU the store-to-load forwarding is nearly free in
one case and not in the other. Others have measured near-zero-latency forwarding on Intel's recent
P-cores (Anton Ertl's add-chain measurements, https://www.complang.tuwien.ac.at/anton/additions/; a
Redwood Cove review, https://jia.je/hardware/2025/04/23/intel-redwood-cove/); that it is skipped for
instruction-pointer addressing is my measurement, not a documented fact (UNVERIFIED in the manuals). As
soon as the function does some other work, the difference is gone (533.449 against 537.570 ms).

**Discarded**, because the effect is the CPU's, appears only when the whole function is one
read-modify-write of one variable, and costs a register and a load in every function otherwise. No
fix makes it a property of the layout. **Good part kept**: microbenchmarks that change one global on every
call measure this effect instead of what they mean to; the threads group's `globals-1`, `globals-2`
ratios (3.10x, 7.15x) and this report's `globals2` table are affected, and comparisons must use the same
addressing on both sides.

One more note on `globals.c`: its model of today's workaround (`globals-1`) takes the lock with the
runtime's one-thread shortcut, which exists only off Windows; on Windows Ember's `Mutex` is an SRW lock
(`ember_rt.h`, `[THR-3]` comment). It is a baseline, so no verdict rests on it.

---

## 6. A14: generators and async

### 6.1 Loans into the frame's own heap contents across `yield` (answers report 7.3 rule 1): saved

**What it says.** A loan into the heap contents of the frame's own local (a `String`'s bytes, an
`Array`'s elements) may be held across `yield`, because moving the frame moves only the header; a loan
into inline storage stays refused.

**Attack: dictionaries and sets** (ease).

```ember
# x02_gen_dict.em (today's syntax)
gen fn keys_of() -> Generator[int]:
    m: Map[int, int] = {}
    m[1] = 10
    m[2] = 20
    for k in m.keys():
        yield k
```

```python
# py_attack.py, check 5: [1, 2]
def keys_of():
    d = {1: 10, 2: 20}
    for k in d:
        yield k
```

Today: `error[E2221]: a borrow of m, a local of this generator, is held across this yield`. Under the
proposed rule it stays refused, and correctly so: `MapKeys` holds `owner: ref Map[K, V, H]`
(`std/src/collections.em` line 907), a pointer to the map's header, which lives in the frame and moves
with it. `SetIter` holds `owner: ref Map[T, bool, H]` the same way (line 1071). `Chars` and `Bytes` hold a `str`, a
pointer into the string's heap bytes (`std/src/string.em` lines 128 to 165), so `for c in s.chars()`
qualifies. The rule is only as good as each iterator's representation.

**Saving it.** State the rule by what the loan points to in the emitted code (heap contents, not a
header), and change `MapKeys`, `MapValues`, `MapItems` and `SetIter` to hold a view of the entries'
heap buffer (the map's `entries: Array[...]`, line 417) and a position, as `Chars` holds a `str`. The map
cannot be reshaped while the iterator lives (the ordinary loan rule), so the buffer stays put. No cost
(one pointer fewer to follow per step). A user-written iterator holding `ref self` stays refused, with the
fix-it to hold a view of the heap part.

**Verdict: saved.**

### 6.2 A generator consumed in place may walk a class object's field (answers report 7.3 rule 2): kept

```ember
for n in door.replay():           # replay walks self.log across yield
    println(n)
```

```python
for n in door.replay():
    print(n)
```

Attacks: the loop body replaces `door` (`door = Door()`), which may drop the object the generator walks:
the body's summary says it may drop a `Door`, which breaks the loan into `Door.log`, so it is refused
(or the frame's handle to `self` keeps it, if counted); passing the generator to `zip` makes it "passed",
so `E2229` stays; another thread changing `door.log` needs `Door` shared, and then `log` is guarded and
the generator's unguarded read is refused. No hole found. **Kept.**

### 6.3 Async tasks in task groups (answers report 7.3 rule 3, design only): saved

**What it says.** An async task's frame is allocated once when spawned and never moves, so loans of its
inline locals across `await` are allowed; the scope rule replaces `Send`.

**Attack.** Task frames never move, but a future that is not a task can be started and then moved: a
`select` (or timeout) that resumes two futures, finishes with one and hands back the other, still
suspended with a loan into its own inline local, which the caller then pushes into a list.

```ember
# proposed async syntax (design only)
winner, other = await select(fetch_a(), fetch_b())   # `other` was started, then returned by value
pending.push(other)                                  # moves a started frame
```

```python
done, pending = await asyncio.wait({asyncio.create_task(fetch_a()), asyncio.create_task(fetch_b())},
                                   return_when=asyncio.FIRST_COMPLETED)
```

**Saving it.** Inline loans across `await` are allowed in task frames and in futures awaited directly
(`await f()`: the callee's frame is inside the caller's, which does not move); a future that is stored,
passed or returned (to `select`, say) keeps today's refusal of inline loans, as generators keep `E2229`
under rule 2. Python's own `asyncio.wait` works on tasks, which are task frames here. No cost.

**Verdict: saved** (design only, as the answers report says).

The cost claim for generators ("1.00x of today's workaround") is true by construction: the model of the
proposed frame (`gen.c` model 1) reads the string through the frame's header and an index, which is the
same code as today's index workaround. It is a correct statement of what the rule may emit; nothing to
change.

---

## 7. A20: C callbacks and `unsafe`

### 7.1 Checked trampolines (answers report 8.3): saved

**What it says.** Registration puts the closure in a table slot and gives C `slot | generation << 32` as
the user data; unregistering bumps the generation. The trampoline checks the slot is in range, compares
the generation (a stale call does nothing), compares the thread for `threads(creator)` callbacks, then
runs the closure. A running count is added only for closures that may unregister themselves. Claimed:
the facts today's contracts assert are checked, "better than Rust". Measured 1.03x of plain C.

#### Attack 1: unregistered while C runs it on another thread (memory safety)

```ember
# y09_ffi_any_unregister.em (proposed syntax)
@ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
extern "C" fn register_cb(cb: fn(ud: *mut void), ud: *mut void) -> CallbackToken

fn main():
    game = Game()
    token = register_cb(fn(): game.score += 1)
    thread.sleep_ms(20)
    drop(token)                 # unregister: frees the closure's state, releases `game`
```

```python
CB = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
cb = CB(lambda ud: setattr(game, "score", game.score + 1))
lib.register_cb(cb, None)
time.sleep(0.02)
lib.unregister(); del cb        # the ctypes documentation warns that a callback object freed
                                # while C can still call it crashes the program
```

For a `threads(any)` callback, C's own thread can be inside the closure when the main thread drops the
token. The generation check passed before the drop; nothing makes the drop wait, and only closures that
unregister *themselves* get the running count. Shown in C (`bench/ffi_toctou.c`, `FIX=0`: the library's
thread enters the callback, the callback signals and works for 200 ms, the main thread unregisters, and
"freeing" overwrites the Game with a poison value): `FIX=0: the callback wrote into freed state: YES`
(`runs/ffi_toctou.txt`). The same happens for a `threads(creator)` callback whose token is dropped on
another thread.

#### Attack 2: another callback's user data (memory safety)

```ember
# y10_ffi_wrong_userdata.em (proposed syntax)
a = c.on_event(fn(): game.score += 1)              # captures a Game handle
b = c.on_text(fn(s: str): log.push(s.to_string()))  # captures an Array[String]
c.mix_up()                                         # C calls on_text's pointer with on_event's user data
```

```python
# ctypes has the same hazard: each CFUNCTYPE object is a separate thunk, but a void* user-data value
# handed to the wrong function is whatever the receiving callback casts it to
```

The value is a live slot with the right generation, so range and generation pass, and `on_text`'s
trampoline runs its body on `on_event`'s closure state: a type confusion. A C bug or a wrongly inferred
contract (section 7.2) is enough.

#### Attack 3: the table grows

The model reads `table` (a growable runtime table, per its own comment). If registering on thread A
reallocates the table while thread B is inside a trampoline holding `&table[i]`, B reads the freed old
table.

**Saving it.**

1. **Announce the running call** (for `threads(any)`, and for any token that can be dropped on another
   thread). The trampoline writes the slot into its thread's record (a small stack, for callbacks that
   call back into C and are called again) with plain stores, and withdraws it on exit. Unregistering,
   which is rare, bumps the generation, calls `FlushProcessWriteBuffers()` (on Linux
   `membarrier`), which makes every running thread's earlier stores visible, then waits until no other
   thread's record announces the slot, and only then frees. A call that announced before the barrier is
   seen and waited for; one that announces after it sees the new generation and does not run. A callback
   that unregisters itself (its own thread announces the slot) defers the free to its trampoline's exit.
   Shown: `FIX=1: the callback wrote into freed state: no` (`runs/ffi_toctou.txt`). An unattached C
   thread's record pointer starts at a full shared record, which sends it to the slow path that attaches
   it, so the fast path has no extra test.
2. **Check the type in the same compare as the generation.** The slot stores `{generation, type}` as
   one 64-bit word; the trampoline compares it with `{its own type, the handle's generation}` in one
   instruction and only on a mismatch looks at which half differs (stale: return; another callback's:
   panic).
3. **The table never moves**: its address range is reserved once and pages are committed as it grows.
4. Unregistering calls C's own unregister function first and bumps the generation after, so a library
   that calls the callback from inside its unregister function (to flush events) is not mistaken for a
   stale call.

Measured (`bench/ffi2.c`, the answers report's workload: a C function not inlined calls the callback
200,000,000 times, the callback adds 1 to `game.score`; `runs/ffi2_summary.txt` and
`runs/ffi2_b2_summary.txt`):

| Program | What it is | batch 1 ms (ratio) | batch 2 ms (ratio) |
|---|---|---|---|
| `ffi2-0-plainC` | user data is `Game*` | 161.922 (1.00) | 156.947 (1.00) |
| `ffi2-2-checked` | the answers report's trampoline | 164.925 (1.02) | 163.270 (1.04) |
| `ffi2-5-checked-typed` | type check as a separate compare | 199.990 (1.24) | 199.664 (1.27) |
| `ffi2-8-checked-typed-one-compare` | **fix 2**: generation and type in one compare | | 164.886 (1.05) |
| `ffi2-6-any-atomic-running` | `threads(any)` protected by an atomic count (compare-exchange in, decrement out) | 2719.654 (16.80) | |
| `ffi2-7-any-announced` | **fix 1**: `threads(any)` protected by announcing (two plain stores in, one out) | 229.274 (1.42) | 229.630 (1.46) |
| `ffi2-9-any-announced-one-compare` | fix 1 with fix 2's compare | | 245.132 (1.56) |

The separate type compare cost a quarter on this empty callback (one more compare-and-branch in a
loop that does nothing else); folded into the generation compare it costs nothing measurable (1.05x
against 1.04x). The announcement costs 0.36 ns per call ((229.630 - 156.947) ms / 200,000,000), 1.46x
on an empty callback, against 12.8 ns per call (16.8x) for the atomic running count. Unregistering with 7
other threads calling callbacks: 3.093 µs median of 5 pinned runs (2.587 to 4.169 µs;
`runs/ffi_unregister.txt`, `ffi_toctou.c` `FIX=2`).

Is 1.46x on par? Only `threads(any)` callbacks pay it, and the ratio is on a callback that does one add;
any real work dilutes the 0.36 ns. Rust's safe-outside version pays nothing because it does not protect
against this at all (`&mut *(ud as *mut Game)`, trusting the library). The version that matches Rust's
cost is to trust the library's promise that its unregister function waits for running calls, which is
today's asserted contract; the announcement is what makes the claim "checked, better than Rust" true.

**Attacking the saved version.** Nested callbacks deeper than the record's stack (14 here) take the
counted path. Thread records stay valid while scanned (a thread detaches under the registry lock). Waiting
in unregister while holding a lock the running callback needs is a deadlock, not a memory error (the
same as Windows' `UnregisterWaitEx` in its waiting mode). On macOS there is no `FlushProcessWriteBuffers`
or `membarrier`; a substitute exists (UNVERIFIED for this use). On ARM the light side needs the compiler
barrier only, as here, because the heavy side is a real barrier on every core (the published
asymmetric-fence technique; not run on ARM).

**Verdict: saved** (announcement for callbacks that can be unregistered from another thread; one compare
for generation and type; a table that never moves; unregister order). Discarded: the atomic running count
for `threads(any)` (16.8x); good part: it needs no operating-system barrier, so it is the fallback for
nesting deeper than the record's stack.

### 7.2 Contracts inferred from the C header (answers report 8.4): kept

A wrong guess about which `void*` is the user data now reaches the trampoline's checks: an arbitrary
pointer value fails the range check or the generation and type compare (a panic), or, by coincidence,
names a live slot of the same closure type (a wrong call, memory-safe). A callback inferred as `retained`
that C only calls during the call (a `qsort_s` comparator) costs one registration per call to C, not per
callback (not measured). No hole found. **Kept.**

### 7.3 A reference made from a raw pointer lives no longer than its `unsafe` block (answers report 8.5): saved

```ember
# y13_unsafe_ref_return.em (proposed rule)
struct Wrapper:
    raw: *const int

    fn get(self) -> ref int:
        unsafe:
            # SAFETY: raw points at memory this Wrapper owns for as long as it lives
            return ref *self.raw          # refused: the reference's limit is the unsafe block
```

```python
# Python has no counterpart (ctypes pointers are unchecked everywhere)
```

As written, no safe wrapper can return a reference into memory it owns, which is what most FFI
wrappers do. Rust allows it, tying the reference to `&self` (`r06_raw_ref_borrows_self.rs` compiles).
**Saving it**: a reference made from a raw pointer borrows the place the raw pointer was read from
(`self` here), never more; it still never gets an unbounded limit, which was the point. No cost.
**Verdict: saved.**

---

## 8. What the saved answers need from other groups

| From | What | Used by |
|---|---|---|
| ownership | the count's flag bits tested by unsigned range on retain and release; one more bit (30) for the claim | 3.4, 3.5 |
| ownership | "init ends at the first use of `self` as a whole" kept in the share and lock rules | 3.3 |
| ownership | "a fresh object not yet stored or shared" as a summary fact (for configure-then-share) | 3.4 |
| ownership | "needs `x.lock` held" as a precondition in summaries | 3.3 |
| references | handle views read through a lock end with the guard; a view field (`ref Player`) in a struct or class carries its limit | 3.1 |
| standard library | `MapKeys`, `MapValues`, `MapItems`, `SetIter` hold a view of the entries' heap buffer | 6.1 |
| runtime | thread records with an announcement stack; unregister with a process-wide barrier; a reserved, never-moving callback table | 7.1 |
| C backend | reductions as C locals in the task function, as `@parallel` partials are today; module variables initialised before `main` in dependency order | 3.2, 5.1 |

---

## 9. Found in passing (today's compiler; nothing was changed)

1. **A misleading message for a scope passed to a function.** `x01_scope_param.em`:
   `error[E2072]: std.thread.Scope has no method named spawn`. `Scope` has `spawn`; what is refused is
   spawning through a `Scope` that is not the `with` binding (`[THR-5]`). The message should say that.
2. **Today's `[THR-1]` init clause works** (`x04_sync_init_escape.em`: `E7003`), which is why section 3.3
   keeps it.
3. **`ember check` writes an incremental cache** (`target/debug/incremental/`, 285 KB) into the folder of
   the file it checks (here `attack/probes/target/`). Not a defect, but worth knowing for disk use.

---

## 10. Files

All under `target/redesign2/threads/attack/`.

- `probes/`: `x01_scope_param.em` to `x04_sync_init_escape.em` (today's syntax, checked; output
  `ember_check_output.txt`); `y01_lockfield_view.em` to `y13_unsafe_ref_return.em` (proposed syntax,
  attack programs).
- `rust/`: `r01_configure_then_share.rs` to `r06_raw_ref_borrows_self.rs`; output `rustc_output.txt`.
- `python/py_attack.py` and `py_attack_output.txt`.
- `bench/`: `claim2.c`, `reduce2.c`, `lock2.c` (made from `../../bench/lock.c`), `ffi2.c`,
  `ffi_toctou.c`, `ripchain.c`, `counts2.c` (made from `../../bench/counts.c`), `walkbuild.c`,
  `globals2.c`, `matches.c`; `build.sh` (every build line, and every timing command); `run_group.sh`
  (calls `target/redesign2/ownership/bench/run.ps1` and `medians.py`); `asm/` (assembly quoted above);
  `runs/` (every run and every summary; `claim2_dups.txt`, `ffi_toctou.txt`, `ffi_unregister.txt` for the
  functional checks).

---

## 11. The final answer list for the group

Each line is the answer as it now stands; "saved" answers replace the originals.

1. **A15, the scope rule (saved).** Tasks in a scope are checked against each other and against the
   parent from their effect summaries; no `Send`, `Sync` or `@sync` is written. Added: a handle read
   through any lock (a `Mutex`'s value or a lock field) is a view inside the `with` block and a counted
   (atomic) handle past it; everything stored into a lock is flagged at the store, before it becomes
   visible; a scope handed to a function cannot spawn (as today) unless the callee's summary carries the
   spawned task's effects until the scope ends; in a task's own code a parent object may be retained
   with an atomic increment when nothing else changes counts of that class with plain instructions during
   the scope, and such objects leave only by a reduction or a lock (Rust's `Arc` price: 1.10x and 1.26x of
   no counts); a view field (`ref Player`) gives Rust's borrowed price.
2. **A15, reductions (saved).** `x += e`, `x = max(x, e)`, `push`, `extend`, `add` written in the task's
   own body become a C local per task, combined at the join in spawn-tree order (1.00x of plain C; a
   reduction reached through a pointer would be 1.61x). A call whose summary writes the variable is a
   conflict, with the fix-it of returning the amount.
3. **A15, the lock field (saved).** `with self.lock:` guards the other fields once the object is shared;
   `init` ends at the first use of `self` as a whole (today's `[THR-1]` clause); a function that touches
   guarded fields without its own `with` gets "needs `x.lock` held", checked at each call.
4. **A15, the shared flag (saved).** Plain counts until an object is reachable from another unscoped
   thread, then the flag and atomic counts. Objects reachable from a shared lock, channel or thread capture
   have fields that are never written after `init` or are guarded by their own lock field (a `Mutex` guards
   its value, not the objects its handles point to); scoped tasks may write through one `Mutex`'s guard
   when every access to that field in the scope goes through it. Writes through a handle not yet shared
   (fresh in this function) do not count. The atomic retain checks overflow; handles known to be shared
   get atomic instructions with no test; both fast paths use unsigned range tests. The walk is on par with
   building with `Arc` (1.03x against 1.07x).
5. **A15, each task changes its own class object (saved, was discarded).** In `@parallel for p in
   players` or `iter_mut` over an owned list of class handles, a count of 1 means plain writes; a higher
   count takes a waiting claim (bit 30 of the count). 0.96x to 1.23x of plain C in four noisy batches,
   against 1.44x to 1.78x for claiming every object; duplicates lose no update and give Python's result.
6. **A33, a guard's limit and copy-out (saved).** A reference read through a guard ends with the guard;
   numbers and strings leave as copies (the same code as C copying the same way); class handles leave as
   atomic retains (true now, by answer 1); lists and structs need `copy(...)`, work inside the block, or the
   accessor.
7. **A33, the accessor (saved).** `yield` inside the `with` keeps the lock until the caller's last use of
   the part. It beats copy-out only for very short uses (0.53x at 4 hashes, 1.74x at 16, 7.1x at 64, under
   8-thread contention); copy-out is the default, and the compiler warns when a lock-holding part stays live
   across a loop or a long call.
8. **A33, snapshots (not chosen; not re-attacked).** Kept as the answers report says, a library option for
   data read far more than written.
9. **A19, module variables (saved).** Python's spelling, `global` to assign; module variables never
   changed after their initialiser are constants readable by any thread; all module variables are
   initialised at start-up in dependency order (a cycle is a compile-time error), so every access is a
   plain global; scoped tasks use them like the parent's locals, with reductions only in task bodies;
   unscoped threads may not touch the others (a `static` with a lock field or `Atomic`, as in Rust).
10. **A19, the base-pointer layout for module variables (discarded).** The 0.55x was the CPU's
    store-to-load forwarding on a one-variable chain (6x on a bare chain, 1.00x with a little other work in
    the function). Good part: benchmarks that change one global per call must use the same addressing on
    both sides.
11. **A14, heap contents across `yield` (saved).** Allowed when the loan points into heap contents in the
    emitted code; `Map` and `Set` iterators changed to hold a view of the entries' buffer, so `for k in d:
    yield k` compiles; loans into inline storage stay refused unless the frame never moves.
12. **A14, generators consumed in place (kept).** A loan into a class object's field may cross `yield` when
    the generator is consumed by a `for` loop in the same function and the loop body neither reshapes that
    field nor may drop that class's objects.
13. **A14, async in task groups (saved, design only).** Task frames never move, so inline loans across
    `await` are allowed there and in futures awaited directly; a future that is stored, passed (to
    `select`) or returned keeps the refusal; the scope rule replaces `Send`.
14. **A20, checked trampolines (saved).** C holds `{slot, generation}`; the trampoline checks range, then
    generation and closure type in one compare, then the thread for `threads(creator)`, in a table that
    never moves (1.05x of plain C). Callbacks that can be unregistered while running on another thread
    announce the running slot with plain stores, and unregistering uses a process-wide barrier and waits
    (1.46x on an empty callback, 0.36 ns per call; about 3 µs per unregister). C's unregister runs before
    the generation is bumped.
15. **A20, inferred contracts (kept).** `callback(retained, user_data=...)` and `threads(creator)` inferred
    from the header; a wrong guess is caught by the checks of answer 14 or is a memory-safe wrong call; the
    remaining asserted fact is listed by `ember tcb`.
16. **A20, `unsafe` references (saved).** A reference made from a raw pointer borrows the place the pointer
    was read from, never more; `unsafe:` blocks keep their `# SAFETY:` notes.
