# Phase 3b: attacking and saving the threads group's answers (second attack)

Written 2026-10-24 for the owner. This attacks the answers in `phase3b-threads.md` (the threads group's
report, called "the answers report" below): threads (A15), references out of a lock (A33), global
variables (A19), async and generators (A14), and C callbacks and `unsafe` code (A20). For every problem I
tried to save the answer first, measured the saved version where it costs anything, and attacked it
again.

This is the **second** attack on these answers. The first one (written at 08:05 today) is kept unchanged
beside this file as `phase3b-threads-attack-first-run.md` (called "the first attack" below). This second
attack was started because the first one ran under a problem the owner raised ("what about the agents
that already ran with that problem?"). I was not told which problem it was, so I took none of the first
attack's findings on trust: I re-ran every one of its Ember probes, Rust twins and Python checks, read
every C model it wrote, and re-timed the measurements its verdicts rest on (section 2). Each of its
findings I keep below says "first attack, rechecked". Then I looked for what it missed. This file
replaces it as the group's attack report; its final answer list (section 12) replaces the first one's.

Nothing in `compiler/`, `std/`, `docs/` or `tests/` was changed. My programs are under
`C:/Users/ism19/Code/ember/target/redesign2/threads/attack2/` (called `attack2/` below). No Ember program
was run; only `ember check`.

---

## 0. The short version

The first attack's findings all reproduce. It missed seven memory-safety holes: in rules the answers report
wrote (values that must stay on their thread; a callback C runs during a later C call), in rules the first
attack kept (a generator consumed by a loop that itself yields; a guessed callback contract), and in the first
attack's own fixes (a lock held by the parent "protecting" its tasks; a function value that drops "needs the
lock"; a claim dropped with a plain store). It also missed seven Python programs the answers refuse, and two C
models (one of the first attack's, one of the answers report's) were not what the compiler would emit. Every
hole is saved at no run-time cost. All answers but one are saved; the one discarded is the one the first
attack discarded.

| Annoyance | Answer (what it does) | Verdict | What changed in this attack |
|---|---|---|---|
| Threads (A15) | Tasks in a scope are checked against each other from their effect summaries ("the scope rule"); no `Send` or `Sync` | **saved** | First attack's fixes, rechecked. New: today, `Send` and `Sync` are also what stop a lock guard, a scope and a raw pointer from reaching another thread (`x05`, `x08`, `x09`); the proposal drops them and nothing replaced that. Saved with a "thread-bound" fact the compiler infers. New: a lock held by the parent does not protect tasks it spawns (`y14`: a data race the rules accept). New: tasks may release objects the parent owns (Python's `p.target = ...` in a parallel loop, `y22`); count changes are netted per task: 1.05x of plain C on that program, where Rust's `Arc` costs 1.80x. |
| Threads (A15) | `total += n` in tasks becomes a per-task total ("reductions") | **saved** | First attack's fixes, rechecked. New: Python's `counts[w] += 1` and `hist[b] += 1` are refused; saved as element reductions (per-task copies added at the join): 3.5x faster than one thread, 91x faster than an atomic add per element. |
| Threads (A15) | A `lock` field guards the other fields (Python's `with self.lock:`) | **saved** | First attack's fixes, rechecked. New: the `y14` race; a function value loses the "needs the lock" fact (`y18`); two locks in a class, a module lock, a lock in a local are not covered (`y19`, `y24`); Python's re-entrant lock habit is refused (`y20`); no way to wait (Python's `Condition`); every object stored under any lock became atomically counted even with no second thread (`y25`). All saved at no run-time cost: one rule, "a lock guards what is only touched while holding it", on the thread that holds it. |
| Threads (A15) | One bit in the count marks an object shared with another thread ("the shared flag") | **saved** | First attack's fixes, rechecked (its atomic orders are Rust's). New: flag only objects of classes that can reach another thread (`y25`). |
| Threads (A15) | Each task changes its own class object; a count of 1 proves no other task can reach it ("claims") | **saved, narrowed** | First attack's idea, rechecked with the real 8-byte header. Its model had a claim byte the header does not have; in the count word the claim must be dropped with an atomic instruction, or count increments are lost (1,136 to 1,387 lost per run, measured: a later use after free). Cost re-measured: 1.12x to 1.36x of plain C when the list is the objects' only holder (the first attack said 0.96x to 1.23x; the final measurer decides), and 1.9x to 4.0x when the objects are also held elsewhere (a filtered list), cheaper than Rust's only safe spelling for that (`Arc<Mutex<T>>`, 4.39x in the answers report) but not on par with C. |
| References out of a lock (A33) | A reference through a guard ends with the guard; numbers and strings leave as copies | **saved** | First attack's fix, rechecked. New: the guard's limit belongs to the thread holding it (`y14`), and a wait on the lock ends every view of what it guards. |
| References out of a lock (A33) | An accessor keeps the lock while the caller uses the part | **saved** | First attack's correction (it beats a copy only for very short uses), rechecked by reading its program. |
| Global variables (A19) | Python's `FRAME = 0` and `global FRAME` | **saved** | First attack's fixes, rechecked. New: Python's module lock and module list used by worker threads (`y24`) now compile; a callback C runs on the main thread during a later C call can grow a list the caller is walking (`y16`, `x06`): a use after free unless every foreign call's summary includes the callbacks C may run. |
| Global variables (A19) | Emit module variables through a base pointer (the answers report's "0.55x") | **discarded** | First attack's verdict, rechecked: the measurement reproduces within 1% (a store-to-load effect of the processor on one variable). |
| Async and generators (A14) | A loan into a generator's own heap contents may cross `yield` | **saved, replaced** | New: the answers report's measured model never holds an address across `yield`; it keeps a position and re-reads the string's header. Holding the address buys nothing (82.020 against 82.222 ms, measured). Keeping a position is safe without the rule, needs no change to the `Map` iterators (the first attack's fix), and keeps working if strings get an inline short form, which the answers report itself suggests for A33 and which breaks the rule as written. |
| Async and generators (A14) | A generator consumed in place by a `for` loop may walk a class object's field | **saved** (was kept) | New hole (`y15`): the loop's body is a `yield`, so the consumer's consumer runs while the walk is suspended and clears the list: a use after free. Saved by the same position walk: Python's behaviour (the walk stops), no refusal, no measurable cost. |
| Async and generators (A14) | Async tasks live in task groups whose frames never move | **saved** (design only) | First attack's fix (a started future passed to `select`), rechecked by reading. New: Python's asyncio habit of changing shared state between `await`s with no lock (`y23`) is refused by the scope rule; on a group whose tasks share one thread, conflicts are checked per `await`. |
| C callbacks (A20) | Checked trampolines: C holds a slot number and a generation, not an address | **saved** | First attack's fixes, rechecked (its unregister protocol is in the right order; 1.05x of plain C reproduced). Its final list said 1.46x for the `threads(any)` version; the version it chose measures 1.51x to 1.58x (1.42x with the type checked separately). New: the re-entrant callback of `y16`. |
| C callbacks (A20) | The callback contract read from the C header | **saved** (was kept) | New hole (`y17`): a guessed user-data parameter that C writes through (a read buffer) makes C write over memory at the address formed by the slot number and the generation, with no `unsafe` anywhere. Shown: 4,096 bytes of an unrelated mapping overwritten. Saved: the value C holds is not a valid address (two high bits set), so C's first write faults; the trampoline's instructions are unchanged (1.02x to 1.05x). |
| C callbacks (A20) | A reference made from a raw pointer lives no longer than the place the pointer was read from | **saved** | First attack's fix, rechecked (`r06` compiles in Rust). New: raw pointers stay thread-bound (`x09`), as today. |

All timings are PROVISIONAL: other agents were running, and two programs running the same loop differed by
up to 17% in one batch (section 2). No verdict here rests on a difference under 15%; where a cost claim
does, I say it waits for the final measurer.

---

## 1. Words used in this report

- **Thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
  **Task**: a function handed to another thread. **Scope**: `with scope = thread.scope():`; tasks spawned
  in it may use the parent's locals, because the block does not end until every task has finished (the
  **join**). An **unscoped thread** (`thread.spawn`) may outlive the function that started it.
- **Data race**: two threads touch one memory location at the same time, at least one writes, and nothing
  orders them. In C and Rust this is undefined behaviour; in practice it loses updates or frees memory still
  in use.
- **Count**: the number of counted handles to a class object; the object is freed at zero. **Handle**: what
  a variable of a class type holds. **Atomic** instruction: a read-modify-write that two threads cannot
  interleave (on this processor a `lock`-prefixed instruction); several times dearer than a plain one.
- **Lock** and **guard**: `with g = m.lock():` waits until no other thread holds `m`, gives `g`, and lets go
  at the block's end. A **lock field** is the answers report's `lock: Lock` field, used as `with self.lock:`.
- **Effect summary** (summary): what the compiler records about a function from its body: which places it
  reads, writes and **reshapes** (changes in a way that can free or move memory). For class objects the place
  is "field `hp` of some `Player`", because two handles can name one object.
- **View**: a reference, or a value holding one, that the compiler limits to the life of what it points into.
- **Reduction**: per-task results combined after the join (`total += n` from several tasks).
- **The shared flag**: the answers report's top bit of an object's count, set when the object first becomes
  reachable from another unscoped thread; flagged objects get atomic counts.
- **Claim**: the first attack's bit in an object's count word that a task sets (atomically) to say "I am
  changing this object"; another task that finds it set waits.
- **Frame**: the struct a generator is compiled into. **Header**: the part of a `String` or `Array` stored
  in the frame or object itself (address of the contents, length, capacity). **Heap contents**: the bytes or
  elements, in a separate heap block. **By position**: a loop that keeps an index, not an address, and reads
  the header again before each step.
- **Trampoline**: the small function Ember generates for a C library to call; **user data**: the `void*` a C
  library hands back to its callback; **slot**, **generation**: the runtime table entry for a registered
  callback, and the number in it that changes when the callback is unregistered.
- **Thread-bound value** (new here): a value that must stay on the thread that made it: a lock guard, a
  scope, a raw pointer, a C handle whose contract says `threads(creator)`.
- **Kinds (a), (b), (c)**: the job's three acceptable answers: a Pythonic spelling of Rust's guarantee; a
  smarter compile-time solution; a new mechanism.

---

## 2. How I checked

**The first attack, rechecked.**

| What | How | Result |
|---|---|---|
| Its four Ember probes in today's syntax (`x01` to `x04`) | copied to `attack2/probes/recheck/`, `target/ci/ember.exe check` (`ember 0.1.0`, `language: 0.9.10`) | the same four results: `E2072`, `E2221`, exit 0, `E7003` (`recheck/recheck_output.txt`) |
| Its six Rust twins (`r01` to `r06`) | `rustc 1.98.1 -C opt-level=2 --edition 2021 --emit=metadata --crate-type lib` | the same: five compile, `r05` gives two `E0594` (`attack2/rust/recheck/rustc_recheck.txt`) |
| Its Python checks | `python -I` (3.14.2) on a copy | "all Python checks passed", the same ten values (`attack2/python/py_attack_recheck_output.txt`) |
| Its C models | read every file in `threads/attack/bench/` | faithful except `claim2.c` (a claim byte the real header lacks, section 3.5); its unregister protocol (`ffi_toctou.c`) announces before it checks, which is the right order (section 7.1); `lock2.c` differs from the group's `lock.c` only in the use length (diff); `globals2.c`'s lazy accessor is what `lazy_static_definitions` emits (`compiler/ember_codegen_c/src/lib.rs` line 7224 calls the out-of-line `ember_rt_static_begin` on every access) |
| Its timings that verdicts rest on | rebuilt from its sources, re-timed | `ripchain` reproduces within 1% (section 5.2); the checked trampoline 1.05x reproduces (section 7.1); the claims' count test does **not** reproduce its 0.96x to 1.23x (section 3.5) |
| The atomic orders in its count models | read `counts2.c`, `walkbuild.c`, `matches.c` | increments relaxed, decrements acquire-release: what Rust's `Arc` does; correct on processors that reorder more than this one |

**New programs.**

- **Ember today**: `attack2/probes/x05` to `x09` in today's syntax, checked; output
  `attack2/probes/ember_check_output.txt`. Programs named `y14` to `y25` use the answers report's proposed
  syntax, which today's compiler cannot check: they are attack programs to read, not compiler output.
- **Rust**: `attack2/rust/r07a` to `r11`, same command as above; output `attack2/rust/rustc_output.txt`.
- **Python**: `python -I attack2/python/py_attack2.py`; every behaviour quoted below is an `assert` there;
  output `py_attack2_output.txt` ("all Python checks passed").
- **Timing**: `attack2/bench/run_group.sh <batch> <base> <programs...>` calls the ownership group's
  `run.ps1` (mains power checked before every round: `(Get-CimInstance Win32_Battery).BatteryStatus` was 2,
  and no round aborted; 5 rounds, each running every program once, one at a time, in the given order, pinned
  with `start /affinity C03C03`, the 8 performance cores) and its `medians.py` (median of 5). Raw lines
  `bench/runs/<batch>.txt`, tables `bench/runs/<batch>_summary.txt`. C is `clang -O2` (22.1.8); every build
  line and every timing command is in `bench/build.sh`. Every program of a workload printed the same checksum
  on every run.
- **All timings are PROVISIONAL.** The processor load read 29% to 32% before my batches (other agents). Two
  programs that run the same timed loop, `claim3-0-plainC` and the first attack's `claim2-0-plainC`, took
  55.328 and 64.790 ms in one batch (`claim3_summary.txt`) and 358.538 and 410.507 ms in another
  (`claim3s_b2_summary.txt`): 14% to 17% apart. That is the noise floor of this machine today.

---

## 3. A15: threads

### 3.1 The scope rule (answers report 4.3): saved

**What it says.** Inside a scope, every task's summary is checked against every other task's and against
the parent's code until the join; a write meeting another task's read or write of the same place is a
compile-time error. "No `Send`, `Sync` or `@sync` is ever written" (its rule 5): what a task may touch is
decided by the summaries alone. Writes to lock-guarded fields and operations on `Mutex`, `Atomic` and
channels are not conflicts.

**First attack, rechecked.** Its four additions stand: a handle read through a lock leaves it as a counted
handle and everything stored into a lock is flagged (section 3.4 narrows the second); a scope handed to a
function cannot spawn (`x01`, still `E2072` today); a task may retain a parent object with an atomic
increment ("rule 4b"; this attack generalises it below).

#### Attack 1 (new): what `Send` and `Sync` stop today that the summaries do not

The answers report removes `Send` and `Sync` as if their only job were stopping data races on counted
objects. Today they also keep three kinds of value on their thread. Probes (`attack2/probes/`):

```ember
# x05_guard_in_two_tasks.em (today's syntax)
m: Mutex[Array[int]] = Mutex([])
with g = m.lock():
    with scope = thread.scope():
        scope.spawn(fn(): g.push(1))
        scope.spawn(fn(): g.push(2))
```

```text
error[E7004]: the task borrows mutably `g`, a `std.sync.MutexGuard[Array[i64]]`, which is not Send
```

```ember
# x08_nested_spawn.em: a task that spawns into its own scope
with scope = thread.scope():
    scope.spawn(fn(): scope.spawn(fn(): println(xs[0])))
    println(xs[1])
```

```text
error[E7005]: the task borrows `scope`, a `std.thread.Scope`, which is not Sync
```

```ember
# x09_rawptr_wrapper_to_thread.em: a wrapper around a C window that its library allows only on the
# thread that opened it
struct Window:
    raw: *mut u8
...
w = open()
h = thread.spawn(owned fn(): w.swap())
```

```text
error[E7004]: the thread's function captures `w`, a `Window`, which is not Send
```

```python
# Python has no counterpart for the guard and the raw pointer; for the first program it runs both
# appends (py_attack2.py check 1: ['#a', '#b'], the interpreter lock serialises them)
```

Rust refuses two of them for the same reasons: `r07b` (a guard captured by a scoped task) gives `E0277
MutexGuard cannot be sent between threads safely`, `r08` (the raw-pointer wrapper) gives `E0277 *mut u8
cannot be sent between threads safely`. The third, a nested spawn, Rust accepts (`r09` compiles), because the
nested task's borrows are checked by the scope's lifetime. Under the answers report's rules nothing refuses `x09` (a raw
pointer is not a counted object and has no summary), so a safe program could drive a thread-bound C library
from the wrong thread, which can corrupt the library's memory. `x05` is caught only if the scope rule's
"writes through a guard are not conflicts" is read narrowly (attack 2 shows it is not, under the first
attack's own wording). `x08` is caught by nothing: the nested task's effects are in no summary.

**Saving it.** Keep the fact, not the word. A value is **thread-bound** when it is or holds a lock guard,
a `Scope`, a raw pointer, or a C handle whose contract says `threads(creator)`. The compiler infers it;
ordinary Python-style code never meets it, because class objects, lists, strings and numbers are never
thread-bound. A task or thread may not capture or receive a thread-bound value (error naming the value, with
the fix-it of passing the data instead of the guard). The author of `unsafe` code may lift it with today's
spelling, `unsafe extend Raw implements Send:` with a `# SAFETY:` note
(`tests/conformance/THR-8/accept_send_written_by_hand.em`), exactly as a Rust author writes `unsafe impl
Send`. Two refinements where Rust is easier than today:

- A task may use the **data** behind the parent's guard, as a `ref mut` place, which the scope rule then
  checks like any other place: one task writing compiles, two conflict. Rust accepts one (`r07c`, through
  `&mut *g`) and refuses two (`r07a`, `E0499`).
- A task may spawn into its own scope when its summary carries the spawned task's effects as "running until
  the scope ends", so the scope rule checks them against everything after the spawn. Rust accepts `x08`'s
  twin (`r09` compiles).

Cost: none at run time.

#### Attack 2 (new): a lock held by the parent does not protect the tasks it spawns (memory safety)

```ember
# y14_lock_held_tasks_write.em (proposed syntax)
fn count(stats: Stats, texts: Array[String]):
    with stats.lock:
        with scope = thread.scope():
            for t in texts:
                scope.spawn(fn():
                    for w in t.split_whitespace():
                        stats.words += 1
                        if w.starts_with("#"):
                            stats.tags.push(w.to_string()))
```

```python
# py_attack2.py check 1: Python runs it; its interpreter lock keeps the appends whole
with s.lock:
    ts = [threading.Thread(target=lambda w=w: s.tags.append(w)) for w in ("#a", "#b")]
```

Every access to `stats.words` and `stats.tags` is "lexically inside `with stats.lock:` on the same handle"
(the lock-field rule, answers report 4.5), and the scope rule does not count writes to lock-guarded fields as
conflicts (its rule 3). Under the first attack's saved lock rule the closure gets "needs `stats.lock` held",
and the spawn site holds it. So the program is accepted. But the lock is held by the parent; the tasks run at
the same time on other threads: a data race on `words`, and on `tags`, whose `push` can reallocate the
buffer another task is writing into. The same hole is in the first attack's saved `Mutex` rule (its 3.4:
"writes through a Mutex guard ... are allowed when every access to that field in the scope goes through the
same Mutex"), with `g.push(1)` and `g.push(2)` from two tasks under one parent guard (the second function in
`y14`; today `x05`'s `E7004`).

**Saving it.** "Held" is a fact about a thread. A lock taken by `with` (a guard, or a lock field) protects
only accesses made by the thread that took it, in that block; a task body starts with no locks held, and a
spawn does not pass a held lock or a "needs the lock" fact to the task. Inside a task, the parent's guarded
places are ordinary places for the scope rule. Then `y14` is two writes to "`words` of some `Stats`" from two
tasks: refused, with the fix-it of taking the lock in the task (`with stats.lock:` inside the lambda) or a
reduction. Cost: none.

#### Attack 3 (new): tasks that release objects the parent owns (Python's `p.target = ...`, readability)

```ember
# y22_parallel_retarget.em (proposed syntax)
fn aim(players: Array[Player], enemies: Array[Enemy]):
    @parallel
    for p in players:
        p.target = nearest(enemies, p.x)      # releases the old target, retains the new one
```

```python
# py_attack2.py check 7: every player gets its nearest enemy
with ThreadPoolExecutor(4) as ex: list(ex.map(aim, players))
```

The answers report's rule 4: "A task that drops handles it owns to parent objects writes 'the count of some
C', so two such tasks conflict." The first attack's rule 4b lets a task retain a parent object with an atomic
increment but says it "may not release" one. Overwriting `p.target` releases the old target, so this
everyday game loop is refused. Rust accepts it with `Arc` targets (`r10_parallel_retarget.rs` compiles),
paying an atomic increment and an atomic decrement per re-target.

**Saving it.** Three ways to emit the count changes, measured (`attack2/bench/retarget.c`: 1,048,576 players
in 8 tasks, 16 passes; 64 enemies; models 1 to 3 skip the count changes when the target does not change;
`runs/retarget_summary.txt`):

| Program | What it is | `nearest` search over 64 enemies (y22 as written) | bare re-target, 64 enemies | bare re-target, 65,536 enemies |
|---|---|---|---|---|
| plain C | a pointer store, no counts (not safe) | 60.882 ms (1.00) | 8.982 ms (1.00) | 9.079 ms (1.00) |
| atomic in the task | Rust's `Arc` price: atomic increment of the new target, atomic decrement of the old | 109.709 (1.80) | 217.763 (24.2) | 28.891 (3.18) |
| deferred to the join | the task logs (old, new); the parent changes the counts after the join | 63.228 (1.04), memory 1.45x | 55.965 (6.23) | 63.604 (7.01) |
| **netted** | each task adds +1/-1 into a 1,024-entry table of its own; increases are applied with an atomic add when an entry is pushed out or the task's loop ends; decreases only after every task has finished its loop (a barrier), with an atomic subtract that may free | **63.668 (1.05)**, memory 1.01x | **18.360 (2.04)** | 32.159 (3.54) |

Why atomic counts are so slow with 64 enemies: every task reads all 64 enemies in its search, and every count
change writes the line those reads need, so the line moves between cores on every change. Rust's `Arc` keeps
its count in the same block as the data and pays the same.

**The rule chosen (netted).** In a task, a retain or release of a parent object is added to the task's own
table; increases are applied atomically when the task's loop ends (or an entry is pushed out); decreases only
after every task of the scope has finished its loop. Why it is safe: no count decreases while any task is in
its loop, so nothing a task can reach is freed under it; after the barrier every increase has been applied,
so a count that reaches zero then is truly zero, and the object's `drop` runs in that task, whose summary the
scope rule already checks. The parent may not change counts of that class during the scope (as rule 4b
requires), and neither may a share's walk. Cost: on the program as Python writes it, 1.05x of plain C (under
15%: the final measurer decides) and 0.58x of Rust's `Arc` price; on a bare re-target loop 2.0x with hot
objects (Rust's price 24x) and 3.5x with cold ones (Rust's 3.2x, the same within 15%). It replaces the first
attack's rule 4b, which is its retain half (the first attack measured that half at 1.10x and 1.26x, its
`matches_summary.txt` and `matches_hot_summary.txt`). The deferred version is not chosen: its log costs 1.45x memory and its serial apply
7.0x on the cold loop. **Not on par with C** on a loop that does nothing but re-target, as Rust is not
either; the C-speed spelling is a view field (`target: ref Enemy`, the references group's) or an index, as in
Rust.

**Attacking the saved version.** A dead enemy held only by targets: its count can reach zero only after the
barrier, when no task is in its loop. A task that reads another player's target (`q.target.hp`): reading
another player's fields conflicts with that player's own task (the scope rule). The claim's count test
(section 3.5) reads counts of the loop's own class during the scope; netting only delays decreases and adds
increases, so a list that holds an object twice still shows a count of at least 2. No hole found.

**Verdict for the scope rule: saved** (first attack's fixes; thread-bound values; locks held per thread;
nested spawns carry their effects; netted count changes for parent objects in tasks).

### 3.2 Reductions (answers report 4.4): saved

**First attack, rechecked.** Reductions only in the task's own body (a reduction through a pointer would be
1.61x, its `reduce2_summary.txt`), combined in spawn-tree order. Read again; stands.

#### Attack (new): Python's dict of counts and histogram are refused (readability)

```ember
# y21_dict_reduction.em (proposed syntax)
with scope = thread.scope():
    for t in texts:
        scope.spawn(fn():
            for w in t.split_whitespace():
                counts[w.to_string()] = counts.get(w, 0) + 1)

@parallel
for b in data:
    hist[b] += 1
```

```python
# py_attack2.py check 6: {'a': 3, 'b': 2, 'c': 1}; the histogram is right
def count(t):
    for w in t.split():
        with clock: counts[w] = counts.get(w, 0) + 1
```

The rule recognises `x += e` on one place and `push`/`extend`/`add` on lists and sets. `counts[w] += 1` and
`hist[b] += 1` write a different place each time; two tasks writing one map or array conflict: refused.

**Saving it: element reductions.** A captured map or array that tasks only change by `m[k] op= e` (or
`m[k] = m.get(k, d) op e`), and that nobody reads until the join, gets a private copy per task (an empty map,
or an array of the identity), combined at the join key by key in spawn order. That is what a C programmer
writes, and OpenMP's `reduction(+: hist[:256])`. Measured for the array (`attack2/bench/hist.c`: 32 MiB of
random bytes, 8 tasks, 256 buckets; `runs/hist_summary.txt`):

| Program | median ms | against the reduction |
|---|---|---|
| element reduction (8 private histograms, added at the join) | 2.264 | 1.00 |
| plain C on one thread | 8.019 | 3.54 |
| one shared histogram, an atomic add per byte | 205.210 | 90.6 |
| one shared histogram, a lock per byte (Python's spelling) | 1615.658 | 713.6 |

The map version is not measured (UNVERIFIED: its merge can cost more when the tasks see many different keys,
which a C programmer's per-thread maps pay too). Memory: one private copy per task, as in C. The map's
insertion order is spawn order on every run (Python's threaded order differs between runs). Attacks: reading
`counts` in a task (refused, as for `total`); a value read and written back in a form other than `op=`
(refused, a write); a key type whose hash calls code with effects (that code's summary is checked like any
other). No hole found.

**Verdict: saved** (first attack's fixes; element reductions).

### 3.3 The lock field (answers report 4.5): saved

**What it says.** "In a class with one field of type `Lock`, every other field that is written after `init`
is guarded by it"; each access must be lexically inside `with x.lock:` on the same handle. The first attack
added the `init` clause (rechecked: `x04` is still `E7003` today) and "needs `x.lock` held" for helpers called
under the lock (rechecked: Rust accepts the twin, `r02`).

Six new problems:

1. **`y14` (section 3.1, attack 2)**: a lock held by the parent is taken as protecting tasks it spawns. A
   memory-safety hole.
2. **A function value loses the precondition** (`y18_needs_lock_fn_value.em`): `bump = s._bump` makes a
   value of type `fn()`; a task calls `bump()` with no lock while another task holds it and writes `words`.
   The precondition is checked at calls to `_bump`, and this is not one. A memory-safety hole in the first
   attack's fix.
3. **Two locks** (`y19_two_locks.em`): `stats_lock` guards `hits`, `cache_lock` guards `cache`, a common
   Python shape (py check 4 runs it). The rule speaks of "one field of type `Lock`".
4. **A lock that is not a field** (`y24_module_lock_worker.em`, a module lock and a module list, Python's
   most common threading shape; py check 9 runs it), or a local lock captured by tasks.
5. **Python's re-entrant lock** (`y20_rlock_helper.em`): `deposit_all` holds `self.lock` and calls
   `deposit`, which takes it too. Python writes `threading.RLock()` for this (py check 5: balance 6; a plain
   `Lock` taken twice is refused, `acquire(blocking=False)` gives `False`). The answers report makes it a
   compile-time error, and the first attack's "needs the lock" does not apply, because `deposit` is also
   called from outside with no lock held.
6. **No way to wait.** Python's producer and consumer use `threading.Condition` (py check 10); a lock field
   has no `wait`. Today's `std.sync` has no condition variable at all (`std/src/sync.em`: `Mutex`, `RwLock`,
   `Atomic`, `Once`, channels), so this is a gap, not a hole; but whoever adds it must know that `wait`
   releases the lock in the middle of the block, so a view of a guarded list taken before the wait would point
   into memory another thread has since changed.

**Saving it: one rule for every lock, the way Python programmers think of them.**

- **A lock guards what is only touched while holding it.** For every place two threads may reach (a field of
  an object that is shared or touched by tasks, a module variable touched by unscoped threads, a local captured
  by tasks), the compiler takes the set of locks held at every access to it, from the summaries, and keeps the
  ones common to all accesses (the lockset idea of the Eraser race detector, Savage et al. 1997, from memory,
  UNVERIFIED as a citation; here done when compiling). A lock is named by a path that cannot change: `x.f` for
  a lock field of the object holding the place (the same handle), a module variable, or a captured local, each
  never reassigned. A written place with no common lock is an error naming two accesses that hold none in
  common. One lock field (today's 4.5), two lock fields (`y19`), a module lock (`y24`) and a local lock are all
  this one rule.
- **Held is per thread** (problem 1): see section 3.1.
- **Preconditions travel with function values** (problem 2): a function value's type carries "needs `x.lock`
  held" as it carries declared effects (`[EFF-2]` puts effects on function types); making a value from a
  method with such a precondition is an error unless the value's type states it, with the fix-it
  `fn(): with s.lock: s._bump()`. A `dyn` method may not add the precondition (as for contracts, `E4010`).
- **Re-entering a lock this thread already holds** (problem 5): when a call happens while the caller is known
  to hold `x.lock` (the compiler sees the `with` and the handle), and the callee's summary takes `x.lock`, the
  call goes to a second entry of the callee that skips the acquire and the release: the callee compiled
  under "needs `x.lock` held", which the first attack's rule already defines. Every call site knows statically
  which entry it needs, so no owner or depth is kept when running (Python's `RLock` keeps both). A call through
  a value or a `dyn` whose effects may take the lock stays an error (it could wait for itself).
- **Waiting** (problem 6): `x.lock.wait()` (or `wait_until(cond)`), as Python's `Condition` wraps a lock. A wait
  counts as a reshape of every place the lock guards, so the ordinary loan rule ends every view of them at the
  wait. Cost: the operating system's condition variable, which C pays.
- **Flag only what can reach another thread** (problem `y25`): section 3.4.

Cost: none at run time beyond the locks (each the lock C takes; the re-entrant case takes one lock where
Python's `RLock` takes a counted one).

**Attacking the saved version.** A lock reached through another object's field (`self.parent.lock`
guarding `self.items`) is allowed only when every field on the path is never written after `init`, so the
lock's identity cannot change. Reading `RESULTS` in `main` after joining the workers, without the lock: the
compiler cannot prove every thread that touches it was joined, so it asks for the lock (or a scope); a
residue, safe. Deadlocks (two locks taken in opposite orders) are not memory errors (the runtime's lock is an
SRW lock on Windows, `runtime/ember_rt/include/ember_rt.h`). No hole found.

**Verdict: saved** (first attack's fixes; one lockset rule; locks held per thread; preconditions in function
types; static re-entry; waits end views).

### 3.4 The shared flag (answers report 4.6): saved

**First attack, rechecked.** Its exact rule for objects inside locks (Rust refuses its `y03` twin, `r05`:
`E0594`, rechecked), configure-then-share (`r01` compiles, rechecked), the overflow check on the atomic path,
direct atomic instructions for handles known to be shared, and range tests on both paths. Its models'
atomic orders are Rust's (section 2). Back links and `weak` fields, which the walk does not follow: the graphs
group's attack refuses sharing a class that is the target of a `weak` field, and a back-linked tree's `parent`
field is written when its root is dropped, so the share check refuses it (`phase3b-graphs-callbacks-attack.md`
3.5 and 4.2; `phase3b-combined.md` 4.7). So the walk never misses a reachable object. Rechecked by reading.

#### Attack (new): every object stored under any lock pays atomic counts, even with one thread (speed)

```ember
# y25_lockfield_single_thread.em (proposed syntax)
class Cache:
    lock: Lock = Lock()
    entries: Array[Entry] = []

    fn put(self, e: Entry):
        with self.lock:
            self.entries.push(e)        # flagged here, though no thread exists

fn main():
    c = Cache()
    for i in 0..1000000:
        e = Entry(key=f"k{i}")
        c.put(e)
        e.hits += 1                     # and the shared-object rule now refuses this write
```

```python
# Python's library classes often carry a lock for whoever needs it (logging.Handler does); a program with
# one thread pays one uncontended lock and nothing else
```

The first attack's rule 2 (its 3.1): "Everything stored into a lock is flagged at the store ... whether or not
the object holding it has been shared." Then a class written to be safe for threads, used by one thread,
walks and flags every object stored in it; every count change on those objects is atomic (the answers report
measured 7.6 ns per atomic pair against 1.6 ns plain, its `counts_summary.txt` batch 1), and every later write
to their fields is refused by the share rule.

**Saving it.** Flag at the store only when the object holding the lock belongs to a class that can reach
another thread: a whole-build fact the share check already computes (some share site, channel or unscoped
capture reaches the class). Inside a scope, objects under a lock that tasks reach follow section 3.1's task
rules (netted counts), which need no flag. Cost: none; the atomic cost is paid only where a thread can see the
object, as Rust pays `Arc` only where the programmer chose it.

**Verdict: saved** (first attack's fixes; flag only classes that can reach another thread; thread-bound
values from 3.1).

### 3.5 Each task changes its own class object ("claims"; discarded by the group, saved by the first attack): saved, narrowed

**What the first attack says.** In `@parallel for p in players` over a list of class handles, a count of 1
proves the list's slot is the only counted handle, so no other task can reach the object: plain writes. A
higher count takes a claim, "bit 30 of the count", waiting if another task holds it. It measured 0.96x to
1.23x of plain C with a separate claim byte, "the instructions are the same".

```ember
# the first attack's y11 (proposed syntax)
@parallel
for p in players:            # players: Array[Player], Player a class
    p.hp -= 1
    p.x += p.hp & 1
```

```python
with ThreadPoolExecutor(4) as ex:
    list(ex.map(hurt, lst))      # first attack's py check 10: both changes land when an object is in twice
```

#### Attack 1 (new): the claim must be dropped atomically, or counts are lost (memory safety)

The lean header is a 32-bit count with its flags in the top bits and a 32-bit type index
(`phase3b-ownership.md` 2.3); there is no free byte, so the claim is bit 30 of the count word. The first
attack's own rule 4b lets other tasks retain objects with atomic increments during the scope. A claim dropped
with a plain store (what a separate byte allowed) is then a read-modify-write of the count word that can
overwrite another task's increment: the count ends lower than the number of handles, and the object is freed
while a handle remains. Shown (`attack2/bench/claim3.c` models 5 and 6: one slot in 16 holds an object
another task's part also holds, and tasks retain those objects with atomic increments while the claims run;
three runs each, `runs/claim3_dups.txt`):

| Claim dropped with | count increments lost (of 4,194,304) | decrements of `hp` lost |
|---|---|---|
| an atomic AND (`lock and`) | 0, 0, 0 | 0, 0, 0 |
| a plain read-modify-write (`and dword ptr [rcx], ...`, `asm/claim3_6.s`) | 1,387, 1,136, 1,272 | 0, 0, 0 |

**Saving it.** Take the claim with `lock bts` and drop it with `lock and` (`asm/claim3_1.s`). This costs only
on the claim path, which is taken only for objects with a count above 1.

#### Attack 2 (new): the cost, measured with the real header

`claim3.c` is the first attack's workload (1,048,576 shuffled handles, 8 tasks, 64 passes) with the lean
header. Ratios to `claim3-0-plainC` in the same batch:

| Program | `claim3` | `claim3_b2` | `claim3s` (8 sweeps per pass) | `claim3s_b2` |
|---|---|---|---|---|
| count test, every count 1 (the list is the only holder) | 1.25 | 1.19 | 1.27 | 1.36 |
| count test, one object in 16 also held elsewhere | 1.42 | 1.37 | 1.51 | |
| count test, every object also held elsewhere (a filtered list, a selection, a grid) | 3.48 | 3.19 | 3.98 | |
| claim every element, no count test (the group's discarded answer, this header) | 2.11 | 1.91 | 2.56 | |
| first attack's `claim2-7` (its count test) | 1.31 | 1.28 | | 1.33 |
| first attack's `claim2-0` (plain C, the same timed loop as `claim3-0`) | 1.17 | 1.04 | | 1.14 |

The fast path is the same three instructions in both attacks' models (`mov eax, [rcx]`, `cmp eax, 1`,
`jne`), so the two count tests measure alike (1.28x to 1.33x against `claim3-0`; against `claim2-0` in the
same batches, 1.12x, 1.23x and 1.16x). Its cost against plain C is 1.12x to 1.36x here and 0.96x to 1.23x in
the first attack's batches, with identical plain programs 4% to 17% apart: **whether it is on par with C
waits for the final measurer.**

The case the first attack did not measure: when the objects are also held by another list, every count is at
least 2 and every element takes the claim. That costs 3.2x to 4.0x of plain C, more than claiming every
element without the test (1.9x to 2.6x; the load just before the locked instruction on the same line costs
more than I expected, cause not isolated, UNVERIFIED). **Saving that case:** where the compiler knows the
list is a selection of objects another list holds (a list of views, or a list built by filtering another),
it emits the claim with no count test (1.9x to 2.6x); where it cannot tell, the count test. Neither is on par
with C. For this sharing, Rust's only safe spelling is a lock per object (`Arc<Mutex<Player>>`, 4.39x in the
answers report's `claim_summary.txt`), so claims stay cheaper than Rust; the C-speed spelling is a struct
element or a list that is the objects' only holder.

**Verdict: saved, narrowed** (atomic drop of the claim; the claim emitted directly for selections; the cost
claim corrected: near C, final measurer to confirm, only for lists that are their objects' only holder).

---

## 4. A33: references out of a lock

### 4.1 A guard's limit and copy-out (answers report 5.3): saved

**What it says.** A reference read through a guard lives no longer than the guard; numbers and strings leave
as copies; class handles leave as counted handles (first attack: everything stored into a lock is flagged,
narrowed in 3.4 above); lists and structs need `copy(...)`.

```ember
fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0]
```

```python
def top(scores):
    with lock:
        return scores[0]
```

**First attack, rechecked by reading.** Its attacks on the guard's limit (a closure, an iterator, a stored
view) and its correction of the "0.88x of Rust" claim stand.

**New.** Two gaps, both closed by rules above at no cost: the guard's limit belongs to the thread that took
the lock (section 3.1 attack 2: tasks spawned inside the `with` see the data as ordinary places), and a wait
on the lock ends every view of what it guards (section 3.3).

**Verdict: saved.**

### 4.2 The accessor that keeps the lock while the caller uses the part (answers report 5.4): saved

**First attack, rechecked by reading.** Its `lock2.c` is the group's `lock.c` with only the use length
changed (diff in section 2); its table (0.53x of copy-out at 4 hashes, 1.74x at 16, 7.11x at 64) and its
warning stand. **New**: a task spawned while the caller holds the part follows section 3.1's rule (the part
is an ordinary place for the task). **Verdict: saved** (the first attack's version).

### 4.3 Snapshots (answers report 5.5): not chosen, not re-attacked

Kept as the answers report says, a library option for data read far more than written.

---

## 5. A19: global variables

### 5.1 Module variables with Python's spelling (answers report 6.3): saved

**First attack, rechecked.** Constants readable from any thread (`x03` still compiles today with `static`;
Rust's twin `r03` compiles); start-up initialisation instead of today's lazy accessor, whose cost it measured
(1.49x) and whose code I confirmed in the C backend (section 2); reductions only in task bodies. One
addition: where the order is not fixed by dependencies, ties are broken by the order in the file, which is
Python's (its py check 8: `['A', 'B', 'main']`).

#### Attack 1 (new): the module lock and the worker threads (readability)

```ember
# y24_module_lock_worker.em (proposed syntax)
RESULTS_LOCK = Lock()
RESULTS: Array[int] = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():
    hs = [thread.spawn(fn(): worker(i)) for i in 0..4]
```

```python
# py_attack2.py check 9: [0, 1, 4, 9]
RESULTS_LOCK = threading.Lock(); RESULTS = []
def worker(x):
    r = x * x
    with RESULTS_LOCK: RESULTS.append(r)
```

Rule 5 of the answer: "a function reachable from `thread.spawn` ... that touches a module variable is a
compile-time error", with the fix-it of a `static` and a lock field. Rust's spelling is a `static
Mutex<Vec<i64>>` (`r11` compiles).

**Saving it.** Two rules, no run-time cost: a module variable of a synchronised type (`Lock`, `Mutex`,
`RwLock`, `Atomic`, a channel end) that is never reassigned is readable and usable from any thread (today's
`static` rule, in Python's spelling); and module variables guarded by a module lock follow section 3.3's
lockset rule. Then `y24` compiles as Python writes it, and an unguarded `RESULTS.push(r)` in `worker` is an
error naming the write and the lock to take.

#### Attack 2 (new): a callback C runs during a later C call grows a list the caller is walking (memory safety)

```ember
# y16_reentrant_callback_module_var.em (proposed syntax); today's twin x06 uses a class field
LOG: Array[String] = []

unsafe extern "C":
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(off_key))
    safe fn on_key(cb: extern "C" fn(*mut void, i32), ud: *mut void)
    fn off_key(cb: extern "C" fn(*mut void, i32), ud: *mut void)
    @ffi(effects=[Io])
    safe fn poll_events()          # C: reads the window's events and calls the key callback

fn main():
    token = on_key(fn(k: i32): LOG.push(f"key {k}"))
    LOG.push("start")
    for line in LOG:
        poll_events()              # the callback runs here and pushes to LOG
        println(line)
```

```python
# py_attack2.py check 3: the loop sees what the callback appended: ['start', 'key 1', 'key 2', 'key 3']
for line in LOG:
    poll_events()
    seen.append(line)
```

This is how event libraries work (a window library's `poll_events` calls the key callback registered
earlier). The callback is `threads(main)`, so A19 rule 5 lets it touch `LOG`. The loop holds the address of
`LOG`'s buffer across `poll_events()`. Today the twin with a class field (`x06_reentrant_callback.em`, and
`x06b` with `@ffi(effects=[Io])` written on `poll_events`) passes `ember check` (exit 0); at run time today
the field's access word catches the push (`[EXC-1]`, `[EXC-19]`). The proposed model removes access words
(`phase3b-ownership.md` 2.3). What remains is `poll_events`'s summary. The combined report gives a foreign
function "the effects its type declares, or every effect" (`phase3b-combined.md` 3.2, rule 1). "Every effect"
is safe (the loop then re-reads the list after the call, its section 4.5 case 2), but then every loop that calls
any C function pays that re-read and every view held across any C call is refused; a declaration such as `@ffi(effects=[Io])` states effect kinds, and
read as "reshapes no Ember place" it lets the callback reallocate `LOG` under the loop: a use after free. The
author of `poll_events`'s declaration cannot know which callbacks a program registers.

**Saving it.** Every foreign call's summary includes the summaries of the callbacks C may run on that thread
before it returns: every callback the program registers with a `threads(main)` or `threads(creator)` contract,
and every `@export`ed function. A declaration narrows what C itself does, never this part. The set is a
whole-build fact, worked out at link time like the share check; a reload that adds a callback re-checks its
callers or is refused (as the answers report does for summaries). Then `y16` takes the combined report's case
2: the loop walks `LOG` by position and reads its length again after each `poll_events()`, which is Python's
behaviour (py check 3), and a loop over a list no callback touches keeps its plain form. Cost: none beyond the
re-read the combined report already measured for that case (its section 4.5).

**Verdict: saved** (first attack's fixes; synchronised module variables and module locks; foreign calls
include the callbacks C may run; file order as the tie-break).

### 5.2 "Emit module variables through a base pointer: 0.55x" (answers report 6.3, 11.7): discarded

**First attack, rechecked by re-timing** (`ripchain.c` built from its source; `runs/ripchain_summary.txt`
against its own):

| Program | first attack, ms | this attack, ms |
|---|---|---|
| loop of `inc` on a global addressed from the instruction pointer | 1197.070 | 1199.511 |
| the same through a register | 197.700 | 196.997 |
| `tick()` called 300,000,000 times, global from the instruction pointer | 455.225 | 453.473 |
| the same through a base pointer | 121.364 | 119.817 |
| as the first, `tick()` also doing 16 multiplications | 533.449 | 532.661 |
| as the second, with the same work | 537.570 | 540.564 |

Reproduced within 1%. The 0.55x is the processor's on a function that is one read-modify-write of one
variable; with a little other work it is gone (1.01x). **Discarded** (the claim; the layout itself costs
nothing measurable and can be left to the C backend). **Good part** (first attack's): benchmarks that change
one global per call must use the same addressing on both sides.

---

## 6. A14: generators and async

### 6.1 Loans into the frame's own heap contents across `yield` (answers report 7.3 rule 1): saved, replaced

**What it says.** A loan into the heap contents of the generator's own local may be held across `yield`,
because moving the frame moves only the header. The first attack found that `Map` and `Set` iterators point
at the map's header, which moves with the frame, and saved the rule by changing those iterators in the
standard library.

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    for s in buf:                 # today E2221
        yield s.clone()
```

```python
def lines():
    buf = ["a", "b"]
    for s in buf:
        yield s
```

#### Attack 1 (new): the rule contradicts the answers report's own A33 suggestion

The answers report suggests (its 5.4 and its section 9 table, strings row) that an inline short form for
strings would make copy-out cheaper. A string stored inline keeps its bytes in its header, which lives in the
frame and moves with it: under rule 1, `for c in s.chars()` across `yield` would then point into the frame's
old place. The references group states the same constraint for its own rule (`phase3b-references.md`
section 5.3, attack 11: rule 1 "needs the guarantee that `String` and `Array` never store their contents
inline"), and the containers group lists a short-string layout among its good parts (1.46x instead of 2.34x for
owned words, `phase3b-containers.md` section 8). The two cannot both stand as written.

#### Attack 2 (new): what the measured model actually does

The cost the answers report gives for rule 1 ("today's frame, measured 1.00x of today's workaround") comes
from `threads/bench/gen.c` model 1, which reads `f->s.p[f->i]` and tests `f->i < f->s.len` on every
resume: a position and a fresh read of the header, not an address held across `yield`. The first attack noted
the same ("true by construction"). So the rule was never measured; its model is the by-position walk. I
measured both (`attack2/bench/gen2.c`, the same workload: a generator over a 1 MiB string, 100 runs,
104,857,600 values; `runs/gen2_summary.txt`):

| Program | median ms | ratio to plain C |
|---|---|---|
| plain C loop | 69.943 | 1.00 |
| frame keeps a position, re-reads the header each resume, `next()` inlined (gen.c model 2) | 82.222 | 1.18 |
| frame keeps an address and an end address across `yield` (what rule 1 allows), inlined | 82.020 | 1.17 |
| the string is a class object's field, walked by position, re-reading the field each resume, inlined | 71.308 | 1.02 |
| frame keeps a position, `next()` not inlined (gen.c model 1) | 208.907 | 2.99 |
| field walked by position, `next()` not inlined | 145.834 | 2.09 |

Holding the address buys nothing measurable (82.020 against 82.222 ms).

**Saving it: walk by position.** A `for` loop over a list, string, map or set whose body contains `yield`
(or `await`) keeps an index in the frame and, after each resume, reads the collection's header again from
where it lives now (the frame, or the object's field) and checks the index against the current length. No
address is held across the suspension, so moving the frame is harmless with or without an inline short form,
and `Map` and `Set` need no new iterators (they walk the entries' array by position). Inside one step (between
two resumes) the ordinary loan rule still applies. For a map, Python raises an error if the dict changed size
during the walk; Ember's walk stays memory-safe either way (whether it should also stop with an error, as
Python's does, is the containers group's choice). What still needs
a loan across `yield` is a walk that has no position (a linked structure held by references), which keeps
`E2221`, and a reference to an inline local (`r = ref n`), which stays refused unless the frame never moves.
Cost: the measured model's own (1.17x to 1.18x inlined, against plain C's loop; the gap is the generator
protocol, as the answers report says).

**Verdict: saved, replaced** (the by-position walk instead of the heap-contents rule; the first attack's
`Map`/`Set` change is no longer needed).

### 6.2 A generator consumed in place may walk a class object's field (answers report 7.3 rule 2): saved (was kept)

**What it says.** A loan into `self.log` may cross `yield` when the generator is consumed by a `for` loop in
the same function, never stored or passed, and the loop's body does not reshape `Door.log`. The first attack
found no hole and kept it.

#### Attack (new): the loop's body is itself a `yield` (memory safety)

```ember
# y15_gen_consumer_yields.em (proposed syntax)
class Door:
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:          # a loan into door.log's buffer, held across yield
            yield entry.len()

gen fn lengths(door: Door) -> Generator[int]:
    for n in door.replay():             # consumed in place; the body's only statement is a yield
        yield n

fn main():
    door = Door()
    door.log.push("open".to_string())
    door.log.push("shut".to_string())
    for n in lengths(door):
        door.log.clear()                # runs while replay is suspended inside lengths
        println(n)
```

```python
# py_attack2.py check 2: [4], the walk stops when the list is cleared; no crash
for n in lengths(d):
    d.log.clear()
    got.append(n)
```

`replay` is consumed in place by `lengths`'s loop, whose body (`yield n`) reshapes nothing. But while
`lengths` is suspended at that `yield`, its own consumer runs, and `main` clears `door.log`; when `lengths`
resumes, `replay` reads through the freed buffer. Rule 2 looks at the loop's body and misses what runs at a
`yield` inside it. Today this is refused only because `replay` is `E2229` (`x07`, checked).

**Saving it.** The by-position walk of 6.1 removes the need for rule 2: `replay` re-reads `self.log` after
each resume (the frame keeps `self` counted, `[CORO-12]`), so whatever the consumer, or the consumer's
consumer, does to the field, the walk stays inside it, and the program behaves as Python's (the walk stops
when the list is cleared, an appended entry is visited). Generators over a field may then also be stored or
passed, which rule 2 refused. Cost: measured in 6.1 (the field walked by position, 1.02x of plain C inlined,
2.09x not inlined against 2.99x for the frame-local walk). One difference from Python remains, the one the
combined report states for loops (its 4.5): if the consumer replaces the whole field (`door.log = [...]`),
Python goes on over the old list and Ember over the new one. The alternative fix, counting a `yield` in the
loop's body as code the compiler cannot see, also closes the hole but refuses `y15`; not chosen.

**Verdict: saved** (was kept by the first attack).

### 6.3 Async tasks in task groups (answers report 7.3 rule 3, design only): saved

**First attack, rechecked by reading.** A future started and then passed to something like `select` can move;
inline loans across `await` are allowed only in task frames and futures awaited directly. Stands. Loops across
`await` use 6.1's walk.

#### Attack (new): Python's asyncio habit (readability)

```ember
# y23_asyncio_shared_state.em (proposed syntax; async is reserved for v2)
async fn crawl(url: String, seen: Set[String], out: Array[String]):
    page = await fetch(url)
    if url not in seen:
        seen.add(url)
        if len(seen) <= 100:
            out.push(page.title)
```

```python
# py_attack2.py check 8: ['a', 'b', 'c']; tasks of one event loop change `seen` between awaits, no lock
async def crawl(url, seen, res):
    await asyncio.sleep(0)
    if url not in seen:
        seen.add(url)
        if len(seen) <= 3: res.append(url)
```

The answers report checks task groups with the scope rule "on one thread or many": two tasks that add to
`seen` and read its length conflict, so this is refused. On asyncio's single thread, only one task runs between
two `await`s, so it is race-free.

**Saving it.** A task group whose tasks all run on the thread that made it (the default, as in asyncio) is
checked per `await`: two tasks conflict only when one holds a loan across an `await` into a place another task
may reshape (the generator logic of 6.1 and 6.2). A group that may run tasks on several threads
(`TaskGroup(threads=8)`) keeps the scope rule. Cost: none. Design only, as the answers report says.

**Verdict: saved** (design only).

---

## 7. A20: C callbacks and `unsafe`

### 7.1 Checked trampolines (answers report 8.3): saved

**First attack, rechecked.** Its three holes and fixes stand: a `threads(any)` callback unregistered while C
runs it on another thread (fixed by announcing the running slot and making unregister wait; I read
`ffi_toctou.c`: the trampoline stores the announcement, then a compiler barrier, then loads the generation,
and unregister bumps the generation, calls `FlushProcessWriteBuffers`, then scans; that is the order the
asymmetric barrier needs); another callback's user data (fixed by checking the closure type in the same
compare as the generation); a table that moves (fixed: reserved once). Re-timed with its own code
(`attack2/bench/ffi3.c`, models copied from its `ffi2.c`; `runs/ffi3_summary.txt`, `runs/ffi3_b2_summary.txt`):

| Program | batch 1 ms (ratio) | batch 2 ms (ratio) |
|---|---|---|
| plain C | 156.179 (1.00) | 163.129 (1.00) |
| checked: range, then generation and type in one compare, then the thread | 164.575 (1.05) | |
| `threads(any)`, announced, type checked separately (its `ffi2-7`) | | 231.639 (1.42) |
| `threads(any)`, announced, one compare (the version its final list chose) | 243.460 (1.56) | 248.779 (1.53) |

**A number corrected.** The first attack's final list says "1.46x on an empty callback" for the announced
version. That was its model 7 (type checked separately); the version it chose (one compare) measured 1.56x in
its own batch 2 (`ffi2_b2_summary.txt`: 245.132 ms) and 1.53x to 1.56x here. Model 7 is 7% faster (under 15%:
the final measurer decides which compare `threads(any)` trampolines use).

**New**: the re-entrant callback of section 5.1 attack 2 is this answer's too: the checks here make a call
safe against a stale, forged or wrong-thread value, but not against a callback that changes what the
interrupted Ember code is walking. Fixed there (foreign calls' summaries include the callbacks).

**Verdict: saved** (first attack's fixes; callbacks in foreign calls' summaries; number corrected).

### 7.2 The callback contract read from the C header (answers report 8.4): saved (was kept)

**What it says.** "When a C function takes a function pointer and a `void*`, and the function pointer's own
parameters include a `void*`, the importer infers `callback(retained, user_data=<that parameter>)`" with
`threads(creator)`, so the call site has no annotation. The first attack attacked a wrong guess from the
trampoline's side (a wrong value reaches its checks) and kept it.

#### Attack (new): a wrong guess that C writes through (memory safety)

```ember
# y17_misinferred_userdata.em (proposed syntax)
#     void async_read(int fd, void *buf, size_t n, void (*done)(void *buf));
import c "aio.h" as aio

fn main():
    fd = aio.open_log()
    aio.async_read(fd, 4096, fn(): println("done"))     # the signature the inference produced
```

```python
# ctypes would pass whatever the program gives; Python has no inference to get wrong
```

The header has the inferred shape, but `buf` is the buffer C reads into, and the callback gets it back. The
importer takes `buf` out of the Ember signature and passes the trampoline's handle in its place; C writes up to
`n` bytes at the address `slot | generation << 32`. None of the trampoline's checks runs: the damage is done
before any callback, and no `unsafe` was written. (`[FFI-21]`'s rule that the callback has exactly one
parameter of the user data's type does not exclude this header: each side has one `void*`.) Shown
(`attack2/bench/tagfault.c`, `runs/tagfault.txt`): with slot 5 and generation 9 the handle is
`0x0000000900000005`, an ordinary user address; where something is mapped there (a heap segment or a mapped
file can be), C's write lands in it:

```text
untagged user data 0000000900000005: exception 0x00000000, bytes of an unrelated mapping overwritten: 4096
tagged user data   4080000900000005: exception 0xC0000005 (0xC0000005 = access violation), nothing written
```

**Saving it.** Make the value C holds unusable as an address: its top 16 bits are `0x4080` (bit 62 and bit 55
set, bit 63 clear), which is not a canonical address on x86-64, so C's first read or write through it faults
before anything is stored (shown above), and the kernel refuses it in a system call. On AArch64 bit 55 puts it
in the kernel half (UNVERIFIED: not run on ARM). The tag is stored in the slot as part of the generation word,
so the trampoline's compare is the same instruction against a different constant: the assembly of `tramp` is
identical with and without the tag (`diff` of `asm/ffi3_8.s` against `asm/ffi3_10.s`, and of `ffi3_9.s`
against `ffi3_11.s`: no difference). Measured: 163.755 ms tagged against 164.575 untagged (1.05x of plain C
both, `ffi3_summary.txt`), 1.02x in batch 2. The generation keeps 16 bits; a slot whose generation reaches
65,536 is retired, as the answers report already does at 2^32. The inferred fact stays listed by `ember tcb`;
a wrong guess now stops the program instead of corrupting it.

**Verdict: saved** (was kept).

### 7.3 A reference made from a raw pointer (answers report 8.5): saved

**First attack, rechecked.** A reference made from a raw pointer borrows the place the pointer was read from
(Rust's twin `r06` compiles, rechecked). **New**: raw pointers are thread-bound (section 3.1 attack 1) unless
the `unsafe` author writes today's `unsafe extend T implements Send:`. **Verdict: saved.**

---

## 8. The C models, checked against what the compiler would emit

| Model | Faithful? | What I did |
|---|---|---|
| the first attack's `claim2.c` | no: a claim byte where the lean header's type index is; a plain-store drop that is only safe in a byte of its own | `claim3.c`: the real header, claim in bit 30, atomic take and drop (section 3.5) |
| the answers report's `gen.c` model 1 ("the proposed frame") | it is the by-position walk, not the rule it was quoted for | `gen2.c`: both forms (section 6.1) |
| the first attack's `ffi2.c` | yes; its final list quoted the wrong model's number | re-timed (section 7.1) |
| the first attack's `ffi_toctou.c` | yes: announce, compiler barrier, then check; bump, process-wide barrier, then scan | read |
| the first attack's `lock2.c`, `ripchain.c`, `reduce2.c`, `globals2.c`, `walkbuild.c`, `matches.c`, `counts2.c` | yes (diff, the C backend's accessor, atomic orders as Rust's `Arc`) | read; `ripchain` re-timed |
| mine: `retarget.c`, `hist.c`, `claim3.c`, `ffi3.c`, `gen2.c`, `tagfault.c` | the netted table, barrier and private histograms are run-time code the compiler would call; each model does the same work, checked by equal checksums | |

---

## 9. What the saved answers need from other groups (added to the first attack's table)

| From | What | Used by |
|---|---|---|
| ownership | the thread-bound fact (guards, scopes, raw pointers, `threads(creator)` handles), inferred; `unsafe extend T implements Send` kept for `unsafe` authors | 3.1, 7.3 |
| ownership | "held lock" facts per thread; "needs `x.lock` held" in function types | 3.1, 3.3 |
| ownership (lean class) | the claim in bit 30, taken and dropped atomically; count range then 2^30 minus the other flag bits | 3.5 |
| runtime | per-task count tables and a barrier for scopes whose tasks retain or release parent objects | 3.1 |
| runtime | the generation word's top 16 bits as a non-address tag | 7.2 |
| C backend | generator loops across `yield` by position; element reductions as private copies | 6.1, 3.2 |
| FFI | foreign calls' summaries include every `threads(main)`/`threads(creator)` callback and `@export`ed function | 5.1, 7.1 |
| standard library | `Lock.wait`/`notify` (a lock with a condition), whose wait ends views | 3.3 |
| standard library | the first attack's new `Map`/`Set` iterators: not needed with 6.1 | 6.1 |

---

## 10. Found in passing (today's compiler; nothing was changed)

1. **Today's thread safety for guards, scopes and raw-pointer wrappers is all `Send`/`Sync`** (`x05`, `x05b`,
   `x08`, `x09`). Not a defect today; it is what the proposal must replace (section 3.1).
2. **A re-entrant callback passes `ember check`** (`x06`, `x06b`) and is caught only when running, by the
   field's access word. Not a defect today; the proposed model must catch it when compiling (section 5.1).
3. **`from std import mem` fails** with `E1061 module 'std' has no item named mem` and an `E0100 expected a
   declaration` pointing into `std/src/lib.em` line 31 (`pass`), met while writing `x06`. The references
   group's second attack reports the same defect (`phase3b-references-attack.md` section 10, item 5).

---

## 11. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/threads/attack2/`.

- `probes/`: `x05_guard_in_two_tasks.em`, `x05b_guard_in_one_task.em`, `x06_reentrant_callback.em`,
  `x06b_reentrant_callback_declared.em`, `x07_gen_consumer_yields.em`, `x08_nested_spawn.em`,
  `x09_rawptr_wrapper_to_thread.em` (today's syntax, checked; `ember_check_output.txt`); `y14` to `y25`
  (proposed syntax, attack programs); `recheck/` (the first attack's `x01` to `x04`, re-checked).
- `rust/`: `r07a` to `r11`, `rustc_output.txt`; `recheck/` (the first attack's `r01` to `r06`).
- `python/py_attack2.py`, `py_attack2_output.txt`; `py_attack_recheck.py` (the first attack's, copied) and its
  output.
- `bench/`: `claim3.c`, `ffi3.c`, `tagfault.c`, `retarget.c`, `hist.c`, `gen2.c`; `build.sh` (every build line
  and timing command); `run_group.sh`; `asm/`; `runs/` (every run and summary; `claim3_dups.txt` and
  `tagfault.txt` for the functional checks).
- The first attack's report, unchanged: `tasks/96/redesign/phase3b-threads-attack-first-run.md`.

---

## 12. The final answer list for the group

Each line is the answer as it now stands; "saved" answers replace the originals and the first attack's list.

1. **A15, the scope rule (saved).** Tasks in a scope are checked against each other and against the parent
   from their effect summaries; ordinary code writes no `Send`, `Sync` or `@sync`. Kept from the first attack:
   a handle read through a lock is a view inside the `with` and a counted handle past it; a scope handed to a
   function cannot spawn unless its effects travel with it. New: **thread-bound values** (guards, scopes, raw
   pointers, `threads(creator)` handles) never reach another thread, inferred, with `unsafe extend T
   implements Send` for `unsafe` authors; a task may use the data behind a parent's guard as an ordinary place;
   a task may spawn into its scope, carrying the spawned task's effects; **a held lock protects only the
   thread that holds it**; tasks may retain and release objects the parent owns, with **count changes netted
   per task** (increases applied at the task's end, decreases after every task's loop): 1.05x of plain C on
   Python's re-target loop where Rust's `Arc` costs 1.80x (PROVISIONAL).
2. **A15, reductions (saved).** `x op= e`, `push`, `extend`, `add` written in the task's own body become a
   local per task, combined at the join in spawn-tree order (1.00x of plain C); **`m[k] op= e` on a map or an
   array becomes a private copy per task, added key by key at the join** (the array: 3.5x faster than one
   thread, 91x faster than an atomic add per element; the map: not measured).
3. **A15, locks (saved).** **A lock guards what is only touched while holding it** (one rule for a lock field,
   several lock fields, a module lock and a local lock); `init` ends at the first use of `self` as a whole;
   "needs `x.lock` held" is inferred for helpers and carried in function types; a `with` on a lock this thread
   is known to hold enters the callee without taking it again (Python's `RLock` habit, no run-time owner); a
   wait on the lock ends every view of what it guards.
4. **A15, the shared flag (saved).** Plain counts until an object can be reached from another unscoped thread,
   then the flag and atomic counts (the first attack's exact rules, overflow check, direct atomics and range
   tests kept); **only objects of classes that can reach another thread are flagged when stored under a lock**.
5. **A15, claims (saved, narrowed).** In a parallel loop over a list of class handles, a count of 1 means
   plain writes and a higher count takes a claim, bit 30 of the count word, **taken and dropped with atomic
   instructions** (a plain drop lost 1,136 to 1,387 count increments per run). Where the compiler knows the
   list is a selection of objects held elsewhere, the claim is taken with no count test. Cost: 1.12x to 1.36x of
   plain C for a list that is its objects' only holder (the first attack: 0.96x to 1.23x; the final measurer
   decides); 1.9x to 4.0x when the objects are also held elsewhere, against 4.39x for Rust's only safe spelling;
   a struct element stays the C-speed form.
6. **A33, a guard's limit and copy-out (saved).** A reference read through a guard ends with the guard, on the
   thread that holds it; numbers and strings leave as copies; class handles as counted handles; lists and
   structs need `copy(...)`, work inside the block, or the accessor.
7. **A33, the accessor (saved, the first attack's version).** `yield` inside the `with` keeps the lock until the
   caller's last use of the part; it beats copy-out only for very short uses, and the compiler warns when the
   part stays live across a loop or a long call.
8. **A33, snapshots (not chosen).** A library option for data read far more than written.
9. **A19, module variables (saved).** Python's spelling, `global` to assign; constants and **synchronised
   module variables (a `Lock`, `Mutex`, `Atomic` or channel end never reassigned) are usable from any thread**;
   module variables guarded by a module lock follow answer 3; initialised at start-up in dependency order, ties
   in file order; scoped tasks use them like the parent's locals with reductions in task bodies; **every foreign
   call's summary includes the callbacks C may run on that thread**, so a loop over a list a callback may grow
   re-reads it after the call, as Python's does.
10. **A19, the base-pointer layout (discarded, rechecked).** The 0.55x was the processor's store-to-load
    behaviour on one variable (reproduced within 1%; 1.01x with a little other work). Good part: benchmarks
    that change one global per call must use the same addressing on both sides.
11. **A14, loops across `yield` (saved, replaced).** A loop over a list, string, map or set whose body yields
    walks **by position**, reading the collection's header again after each resume: no address crosses a
    suspension, so frames may move, strings may get an inline short form, `Map` and `Set` need no new
    iterators, and the measured cost is the answers report's own (holding an address measured the same, 82.020
    against 82.222 ms). A walk with no position, or a reference to an inline local, stays refused unless the
    frame never moves.
12. **A14, generators over a class object's field (saved, was kept).** The same by-position walk, re-reading the
    field after each resume, so the consumer, or its consumer, may change the field (the hole `y15` closed) and
    the generator may be stored or passed; behaves as Python's except when the whole field is replaced.
13. **A14, async in task groups (saved, design only).** Task frames never move, so inline loans across `await`
    are allowed there and in futures awaited directly; a future passed to `select` keeps the refusal; a group
    whose tasks share one thread is checked per `await` (Python's asyncio habit compiles); a group on several
    threads uses the scope rule.
14. **A20, checked trampolines (saved).** C holds a tagged `{slot, generation}`; the trampoline checks the range,
    then generation and closure type in one compare, then the thread for `threads(creator)`, in a table that
    never moves (1.02x to 1.05x of plain C). Callbacks that can be unregistered while running on another thread
    announce the running slot and unregister waits: 1.51x to 1.58x on an empty callback with the one compare,
    1.42x with the type checked separately (the final measurer picks). C's unregister runs before the
    generation is bumped.
15. **A20, contracts read from the header (saved, was kept).** Inferred as the answers report says, listed by
    `ember tcb`; the user data's top 16 bits are `0x4080`, so a wrong guess that C uses as an address faults
    before anything is written (shown), at no instruction cost.
16. **A20, `unsafe` references (saved).** A reference made from a raw pointer borrows the place the pointer was
    read from; raw pointers are thread-bound unless the `unsafe` author says otherwise; `unsafe:` blocks keep
    their `# SAFETY:` notes.

**Good parts of what was not chosen, with where they came from:**

- Deferred count changes applied by the parent after the join (this attack, `retarget.c` model 2): no atomic
  instruction in the tasks; 1.04x on Python's re-target loop, but 1.45x memory and a serial apply (7.0x on a
  bare loop).
- Counting a `yield` in a consumer loop's body as code the compiler cannot see (this attack, 6.2): a smaller
  fix for `y15` if the by-position walk is ever not wanted; it refuses where Python runs.
- The type checked by a separate compare in `threads(any)` trampolines (the first attack's `ffi2-7`): 7% faster
  than the one compare on that path (PROVISIONAL).
- The answers report's heap-contents fact (`phase3b-references.md`): still true and still useful to the
  references group for views into a value's own heap contents; the generator rule no longer needs it.
