# Phase 3b: threads, async, globals, locks and unsafe code (A14, A15, A19, A20, A33)

Written 2026-10-24 by the threads-group agent, under the owner's job (`JOB.md`): speed on par with C
and Rust, memory safety the same as Rust's or better, and Python's ease, all at once, with the
checking done when the program is compiled. Nothing in `compiler/`, `std/`, `docs/` or `tests/` was
touched. My files are under `target/redesign2/threads/` (section 12 lists them).

How every claim here can be traced:

- **Ember today** is real `ember check` output (`target/ci/ember.exe`, which prints `ember 0.1.0`,
  `language: 0.9.10`). Probes and their full output: `target/redesign2/threads/probes/`
  (`ember_check_output.txt`). No Ember program was run, and I did not use `ember build`.
- **Rust** is rustc 1.98.1. New probes: `target/redesign2/threads/rust_probes/` (`rustc_output.txt`).
  Older Rust facts come from phase 1 (`phase1-sharing.md`), which names its files.
- **Python** behaviour quoted here was run: `target/redesign2/threads/python/py_checks.py`, with
  `python -I` (Python 3.14.2); output in `py_checks_output.txt` beside it ("all Python checks passed").
- **Timings** are mine (section 2). **All of them are PROVISIONAL**: other agents were running at the
  same time. A choice that rests on a difference under 15% waits for the final measurer; I say so where
  it applies.
- Numbers from earlier phases and from the sibling groups name their report.

---

## 0. The short version

Last night's designs replaced compile-time checks with run-time ones. Here, everything that decides
whether a program is safe is decided when it is compiled; the few run-time costs left are ones C and
Rust pay for the same work (a lock, an atomic count on an object two threads really share), and each
is measured. Five rules carry all five annoyances:

1. **The scope rule.** Inside `with scope = thread.scope():` the compiler runs every task's effect
   summary (what it reads, writes and frees; the ownership group's "field effects") against every other
   task's and against the parent's code until the join. A conflict is a compile-time error naming both
   lines. No `Send`, `Sync` or `@sync` is written anywhere: a class object that tasks only read needs
   nothing, and tasks never change the reference count of the parent's objects, so no count is atomic.
2. **Reductions.** `total += n` and `results.push(x)` written in tasks compile to one private
   accumulator per task, combined at the join in spawn order. This is what a C programmer writes by
   hand, and it is the only spelling of Python's "add to a shared total" that runs at C speed.
3. **A lock field guards its object.** Python's `with self.lock:` compiles as written: in a class with
   a `lock` field, every other mutable field may be touched only inside `with obj.lock:` once the object
   is shared with threads. Checked when compiling; the lock costs what C's lock costs.
4. **The shared flag.** An object that really crosses to an unscoped thread (`thread.spawn`, a channel,
   a lock's contents) gets one bit set in its count (Koka's idea, `phase2-rc-elision.md` idea 2); from
   then on its count changes are atomic. Every other object keeps plain counts, and the test costs no
   extra instruction (the instructions are identical; measured 1.00x to 1.04x, within noise).
5. **A guard's region.** A reference taken through a lock's guard lives no longer than the guard. This
   closes today's soundness hole (A33). What leaves a lock is a copy of an immutable value, or the lock
   stays held for as long as the caller uses the part (an accessor).

| Id | Annoyance | Best answer here | Kind (the job's three) | Cost when running (PROVISIONAL) | What is left |
|---|---|---|---|---|---|
| A15 | threads: `Send`/`Sync`, sharing, scoped threads | the scope rule, reductions, the lock field, the shared flag | (b) smarter compile-time, with (a) spellings and one (c) mechanism | scoped tasks: none; `+=` from 8 tasks 1.00x plain C (Rust's `AtomicU64` spelling 10.35x); counts of unshared objects 1.00x to 1.04x (same instructions) | class objects *written* by several tasks need a lock or must be structs; a task cannot keep a parent's object past the scope |
| A33 | references out of a lock or `RefCell` guard (unsound today) | a guard's region; `return g[0]` copies a string or number; `with name = top(board):` keeps the lock while the name is used | (a) Pythonic spelling, with (b) for the accessor | copy-out 1.00x plain C copying the same way, 0.88x Rust's `.clone()`; accessor 0.55x of copy-out (no copy) | returning a *list* out of a lock needs `copy(...)` or the accessor (Python returns the shared list, which then races) |
| A19 | global mutable state | Python's own spelling: `FRAME = 0` at module level, `global FRAME` to assign; one thread plus scoped tasks; `+=` from tasks is a reduction | (a) | plain C globals 1.00x; 0.55x if emitted through a base pointer; today's Ember workaround 3.10x (one thread) and 7.15x; Rust's `AtomicU64` + `Mutex` 7.44x | an unscoped thread cannot touch a module variable (it uses a `static` with a lock) |
| A14 | async: `Pin`, `Send`, borrows across `await` | the heap-contents rule for frames, in-place generators, async only in task groups | (b) | none: the frame is today's (measured 1.00x of today's workaround) | a reference to a frame's *inline* local across `yield` stays refused unless the frame never moves |
| A20 | unsafe code and FFI | checked trampolines (C holds a slot number and a generation, not an address; the thread is checked) and contracts inferred from the header | (c) new mechanism, with (a) | 1.03x plain C per callback (today's trampoline 1.47x; Rust's raw-pointer workaround 0.97x) | a callback that may unregister itself pays a running count (1.29x); an inferred contract is still an asserted fact |

**Better than Rust in safety, in one place:** a callback C calls after it was unregistered, or from
the wrong thread, is caught (A20). Rust's workaround is an `unsafe` cast with an unbounded lifetime.
**Equal to Rust elsewhere**, by the arguments and attacks in each section; none is a proof.

**What this group needs from the others** (section 9): field effects with summaries in package
interfaces (ownership); the count of a class as one more "field" in those summaries; one free bit at
the top of the lean class's 32-bit count; `@cached` slots initialised with a once-flag; "a handle read
from a borrowed place is a view" (references); the heap-contents fact (references); yield-accessors
(references); a lambda passed where the callee keeps it captures by retain without `owned`
(closures).

**Found in passing** (section 11): the A33 hole is still present and also passes through `RwLock`'s
read guard; today's FFI trampoline makes an out-of-line call on every callback (1.47x plain C); the
natural `Atomic` spelling in a task is refused by the old "cannot find" bug; two misleading
messages; a missing `ref String` to `str` conversion; and C globals that ran 1.8x slower addressed one
way than another.

---

## 1. Words used in this report

- **Thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
- **Task**: a function handed to another thread to run.
- **Scope, scoped task**: `with scope = thread.scope():` is a block; tasks spawned in it may use the
  parent's local variables, because the block does not end until every task has finished (the
  **join**). `[THR-5]`, `[THR-11]` in the spec.
- **Unscoped thread**: one started with `thread.spawn(...)` or `jobs.submit(...)`; it may outlive the
  function that started it.
- **Data race**: two threads touch one memory location at the same time, at least one writes, and
  nothing orders them. In C and Rust a data race is undefined behaviour; in practice it loses updates
  or frees memory still in use.
- **Count** (reference count): the number of handles to a class object; the object is freed when it
  reaches zero. **Atomic** operation: a read-modify-write that two threads cannot interleave (on this
  CPU a `lock`-prefixed instruction); several times dearer than a plain one.
- **Lock** (mutex) and **guard**: `with g = m.lock():` waits until no other thread holds `m`, binds
  `g` to the protected value, and releases `m` at the block's end.
- **Effect summary**: what the compiler records about a function from its body: which places (a
  variable, a field, a list element, or "field `hp` of some `Player`") it reads, writes and
  **reshapes** (changes in a way that can free or move memory: replace, `push`, `clear`, drop). Ember
  already computes effect sets (`[EFF-1]`) and puts them in package interfaces (`[EFF-4]`); the
  ownership group (`phase3b-ownership.md` section 2.1) adds field-level reshape facts. I build on that.
- **Handle**: what a variable of a class type holds: a pointer to a counted heap object.
- **View**: a reference, or a value holding one, that the compiler limits to the life of what it
  points into (`phase3b-references.md` section 1).
- **Reduction**: combining per-task results with an operation such as `+` after the tasks end.
- **Frame**: the struct a generator (or an async function) is compiled into: its state number and the
  locals that live across a `yield`. **Heap contents**: the bytes of a `String`, the elements of an
  `Array`, the inside of a `Box`; they live in a separate heap buffer, so they do not move when the
  frame (or any value holding the header) moves (`phase3b-references.md` section 1).
- **Trampoline**: the small C function Ember generates for a C library to call; it finds the Ember
  closure and calls it. **User data**: the `void*` a C library hands back to its callback.
- **Generation**: a number stored in a table slot and in every reference to it; a mismatch means the
  slot was reused.
- **Kind (a), (b), (c)**: the job's three acceptable answers: a Pythonic spelling of Rust's guarantee;
  a smarter compile-time solution; a new mechanism.

---

## 2. How the costs were measured

- **Machine**: the owner's laptop, on mains power: `(Get-CimInstance Win32_Battery).BatteryStatus`
  returned 2, and the runner checks it before every round (no run aborted: `grep -c ABORT
  bench/runs/*.txt` gives 0 for every file).
- **Compilers**: `"C:/Program Files/LLVM/bin/clang.exe"` (clang 22.1.8) with `-O2`; `rustc 1.98.1` with
  `-C opt-level=2 --edition 2021 -A unexpected_cfgs`. Each C file builds every model of one workload
  with `-DMODEL=n`; each Rust file builds its variants with `--cfg`. The build lines are at the top of
  each file.
- **Harness**: the shared `target/redesign/bench/harness.h` (it calls `SetErrorMode`, so no dialog can
  open; it prints time, peak working set and a checksum). The Rust twins use the ownership group's
  `target/redesign2/ownership/bench/harness.rs` (same random numbers, same output line).
- **Runner**: `bench/run_group.sh <group> <base> <programs...>` calls the ownership group's
  `run.ps1` (5 rounds; in each round every program runs once, one at a time, in the given order, so A
  and B alternate; pinned with `start /affinity C03C03`, the P-cores) and then its `medians.py`
  (median of 5 for time and for peak memory, ratio to the named base). Raw lines:
  `bench/runs/<group>.txt`; tables: `bench/runs/<group>_summary.txt`.
- Every program of a workload printed the same checksum on every run (the summaries list them).
- Threaded models use 8 threads (the mask `C03C03` has 8 logical processors).
- Not covered: integer overflow checks (`[TYP-8]`) are left out of every C model and of plain C, as in
  earlier phases; Rust builds without overflow checks at this opt level. Teardown is skipped.
- Where I argue "no cost" from the emitted code, the assembly is kept in `bench/asm/` (built with
  `-O2 -S -masm=intel`).

---

## 3. The model in one page

What the five rules have in common: **every place two threads could both touch is known when the
program is compiled, from the effect summaries; the run-time mechanisms (locks, atomic counts) appear
only where two threads really share something, which is also where C and Rust put them.**

- Inside a scope (rule 1), the parent's data is *lent* to the tasks: tasks read it, or write disjoint
  parts of it, or add to it through a reduction (rule 2), or change it under a lock (rule 3). Handles
  to the parent's objects are views inside tasks: no task ever changes the count of an object it did not
  create, so counts need nothing.
- Beyond a scope, a value *moves* to the thread (the sender's name ends, as with the ownership group's
  "names follow values"), and a class object is *shared*: the shared flag (rule 4) makes its counts
  atomic, and its plain fields must not be written by anyone after the share unless a lock field guards
  them (rule 3).
- Locks hand out nothing that outlives them (rule 5).
- Module variables (A19) are lent to scoped tasks like the parent's locals, and refused to unscoped
  threads.
- Generators and async frames (A14) keep views into their own heap contents across `yield`/`await`;
  async tasks live in task groups, which are scopes, so rule 1 replaces `Send`.
- C callbacks (A20) find their closure through a checked table slot, and check their thread, so the
  facts today's contracts assert are checked instead.

---

## 4. A15: threads (`Send`/`Sync`, sharing, scoped threads)

### 4.1 What Rust's rule protects against

Phase 1 (`phase1-sharing.md`, A15, section 5), checked against its probes:

- two threads updating `total` at once: lost updates, a data race (`E0499` in `a15_threads.rs`);
- a thread reading `texts` after the function returned and freed it (`E0373`, `E0521`);
- without `Send`: two threads updating an `Rc`'s plain count at once, so the vector is freed while one
  thread still uses it (`E0277`).

### 4.2 The Python program, and what Ember does with it today

```python
# Python (py_checks.py runs this shape: the total is 6 whatever the order)
total = 0; lock = threading.Lock()
def work(t):
    global total
    with lock:
        total += len(t.split())
threads = [threading.Thread(target=work, args=(t,)) for t in texts]
```

```ember
# Ember today: a15_threads.em (phase 1), re-checked
with scope = thread.scope():
    for t in texts:
        scope.spawn(fn(): total += t.split_whitespace().count())
```

```text
error[E3021]: `t` cannot be written while it is borrowed
error[E3022]: `total` is already mutably borrowed
error[E3060]: `t` does not live long enough
error[E7004]: the thread's function captures `names`, a `Names`, which is not Send
   = help: declare it `@sync class Names`: its fields are all `Send` and `Sync` [THR-1]
```

New probes (`probes/`, all in `ember_check_output.txt`) separate the causes:

| Probe | What it does | Ember today | Rust (`rust_probes/`) |
|---|---|---|---|
| `p15a_loopvar_read.em` | tasks only read their loop variable | **compiles** | (Rust needs `move`) |
| `p15g_write_only.em` | tasks write `total`, do not read `t` | `E3022` only | `E0499` (phase 1) |
| `p15h_read_t_atomic.em` | `total.fetch_add(t.split_whitespace().count())` with an `Atomic` | `E1010 cannot find t` (a compiler bug, section 11) | compiles with `AtomicU64` |
| `p15i_join_sum.em` | each task returns its count; the parent adds the joins | compiles | compiles |
| `p15e_push.em` | tasks `results.push(i * i)` | `E3021`, `E3022` | `E0499`, `E0373` (`r15c_push_scoped.rs`) |
| `p15b_class_read.em` | tasks only read a plain class object | `E7004` "not Send", help `@sync` | `E0277` for `Rc<Config>` (`r15b_rc_scoped_read.rs`) |
| `p15c_struct_elems.em` | each task changes its own struct element (`iter_mut`) | `E3022`, `E3021`, `E3060` | **compiles** (`r15a_iter_mut_scoped.rs`) |
| `p15d_parent_writes.em` | the parent grows a list a task reads | `E3021` (correct: a race) | refused |
| `p15f_sync_class.em` | an unscoped thread changes a plain class's field | `E7004`, help `@sync` | refused |
| `p15k_guarded_today.em` | today's spelling: `@sync class` with `Atomic` and `Mutex[...]` fields | compiles | (Rust: `Arc<...>` with atomics and `Mutex`) |

So the errors on `t` in `a15_threads.em` are knock-on errors of the write to `total` (alone, `t` is
fine), and Ember is stricter than Rust for disjoint element writes (`p15c`).

### 4.3 Answer (b): the scope rule. Kind (b), a smarter compile-time solution

**What the user writes** (the Python spelling, no thread words):

```ember
# a class object only read by tasks: today E7004 (p15b), proposed: compiles
class Config:
    scale: int = 2

fn run(cfg: Config, xs: Array[int]) -> int:
    with scope = thread.scope():
        a = scope.spawn(fn() => xs[0] * cfg.scale)
        b = scope.spawn(fn() => xs[1] * cfg.scale)
        return a.join() + b.join()

# each task its own element: today refused (p15c), Rust accepts (r15a), proposed: compiles
fn hurt_all(players: Array[Player]):        # Player a struct
    with scope = thread.scope():
        for p in players.iter_mut():
            scope.spawn(fn(): p.hp -= 1)
```

```python
from concurrent.futures import ThreadPoolExecutor
def run(cfg, xs):
    with ThreadPoolExecutor() as ex:
        a = ex.submit(lambda: xs[0] * cfg.scale)
        b = ex.submit(lambda: xs[1] * cfg.scale)
        return a.result() + b.result()

def hurt_all(players):
    with ThreadPoolExecutor() as ex:
        for p in players:
            ex.submit(lambda p=p: setattr(p, "hp", p.hp - 1))
```

**The rule the compiler applies:**

1. Each task's effect summary is known: its body is visible (ODR-278 gives every closure its own type)
   and every function it calls has a summary in its interface. Unknown code (a call through `dyn`, a
   function value, a C function) has the effects its type declares, or all effects.
2. The tasks of one scope, and the parent's statements from each spawn to the block's end, are treated
   as running at the same time. Two of them **conflict** when one writes or reshapes a place that another
   reads, writes or reshapes. For a class object the place is "field `f` of some object of class `C`",
   because two handles can name one object (the ownership group's rule 3 of section 2.1). A conflict is
   a compile-time error naming both lines.
3. Not conflicts: operations on `Atomic`, `Mutex`, `RwLock`, channel ends and lock-guarded fields
   (section 4.5); reductions (section 4.4); writes to different elements handed out by `iter_mut`,
   `chunks_mut` or `split_at` (one element per task, as Rust's `iter_mut` gives).
4. **Counts are a field too.** A retain or release of an object of class `C` is a write to "the count
   of some `C`". A handle a task reads from the parent's data is a view inside the task (the
   references group's "a list of views is a view", applied to handles): using it, or keeping it in the
   task's own locals and lists, changes no count. A task changes counts only of objects it created
   itself (the summary tracks fresh allocations separately, so two tasks that each make and drop their
   own `Node`s do not conflict). Storing a parent's handle anywhere that outlives the scope is refused,
   with fix-its: push it in a reduction (it is appended, and retained, by the parent after the join), or
   keep its index. A task that drops handles it owns to parent objects writes "the count of some `C`",
   so two such tasks conflict.
5. No `Send`, `Sync` or `@sync` is ever written. `[THR-1]`'s "a class is `Sync` only when declared
   `@sync`" goes; what a task may touch is decided by rules 2 to 4.

**Why it is memory-safe.** A data race needs two threads, one location, a write, and no ordering.
Every location a task can touch is in its summary (rule 1 over-approximates: unknown code has every
effect), and so is every location the parent touches meanwhile. Two summaries that share a written
location are refused (rule 2), unless the location is synchronised (rule 3). Counts are locations
(rule 4), so a count is never changed by two threads at once. Nothing a task reads can be freed during
the scope, because freeing is a reshape or a count write, and both conflict with a read or a count write
elsewhere. Nothing the scope lends outlives it, because the scope joins before its block ends
(`[THR-5]`).

**Attacks I tried:**

| Attack | Program | Result |
|---|---|---|
| the parent grows a list a task reads | `p15d_parent_writes.em`: `s.spawn(fn() => println(xs[0] + xs[1]))` then `xs.push(4)` | refused (the parent reshapes `xs`, a task reads it); refused today too |
| two tasks write one object through two handles | `scope.spawn(fn(): a.target.hp -= 1)`, `scope.spawn(fn(): b.target.hp -= 1)` | refused: both write "`hp` of some `Player`" |
| a task reads an object the parent changes through another handle | task: `print(cfg.scale)`; parent: `other.scale = 3` | refused: "`scale` of some `Config`" |
| a hidden write in a getter | a task calls `p.area()`, a `@cached` property whose cache slot is written on first read (ownership A31) | refused, unless the cache slot uses a once-flag (section 9 asks the ownership group for it) |
| a call whose effects are unknown | `scope.spawn(fn(): self.on_hit())`, `on_hit` a stored function value | refused unless the field's type declares its effects |
| two tasks free the same object | each task clears a list it owns; one object is in both lists | refused: both write "the count of some `Item`" (rule 4); fix-its: clear after the join, or make `Item` a struct |
| a task keeps a parent's object | `scope.spawn(fn(): cache.push(players[0]))` where `cache` outlives the scope | refused as a store, unless `cache.push` is a reduction (then the parent appends and retains after the join) |
| a race on initialisation | two tasks read a module variable whose initialiser runs on first access | the initialiser runs once under `[STA-3]`'s `Once`, as today's statics |
| a reload widens a summary | hot reload adds a write to a function a task calls | the reload must be refused (the fix proposed by the attacker of "today's rules, but never an error", `phase3-break-D3.md` 1.6); section 9 |

**Cost.** Nothing when running. The emitted code is the code of today's scoped tasks (a closure with
pointers to what it captures, run on a thread or job, joined at the block's end), which is the code of
Rust's `std::thread::scope` and of C threads given pointers. It *removes* atomic counts from every class
object a scoped task reads, which `@sync` requires today (`[RC-4]`).

**What it gives up.** Class objects written by several tasks: refused (Python runs it with no check;
Rust refuses them as `Rc`). The fix-its are a struct element (zero cost,
`iter_mut`), a lock field (section 4.5), or a reduction. The new-mechanism answer that tried to allow
it is in section 4.7.

### 4.4 Answer (a): reductions, and no thread words. Kind (a), a Pythonic spelling

**What the user writes:**

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())   # today E3022; proposed: a reduction
    return total

fn squares() -> Array[int]:
    results: Array[int] = []
    with scope = thread.scope():
        for i in 0..4:
            scope.spawn(fn(): results.push(i * i))                    # today E3021, E3022 (p15e)
    return results                                                     # always [0, 1, 4, 9]
```

```python
def count_words(texts):
    total = 0; lock = threading.Lock()
    def work(t):
        nonlocal total
        with lock:
            total += len(t.split())
    ts = [threading.Thread(target=work, args=(t,)) for t in texts]
    for t in ts: t.start()
    for t in ts: t.join()
    return total

results = []
ts = [threading.Thread(target=lambda i=i: results.append(i * i)) for i in range(4)]
# py_checks.py: every value arrives; Python does not promise the order
```

**The rule.** In a scope, a captured place `x` that tasks only change with `x += e`, `x -= e`,
`x *= e`, `x |= e`, `x &= e`, `x ^= e`, `x = max(x, e)` or `x = min(x, e)` (numbers), or with `push`,
`extend` or `add` (lists, sets), and that neither the tasks nor the parent read or write any other way
until the join, is compiled as a reduction: each task gets an accumulator holding the operation's
identity (0, 1, an empty list) in its own state; at the join the parent combines them into `x` in spawn
order. Reading `total` inside a task is refused ("`total` is the sum of every task, known after the
scope"). `+=` on a float is accepted with a lint: the sum is the same on every run, but when a task adds
more than once its grouping differs from a serial sum (the reason `[PAR-3]` makes float reductions
explicit); with one add per task it is bit-identical to the serial sum.

**Why it is memory-safe.** No location is shared during the scope: each accumulator belongs to one
task, and the join orders every task's writes before the parent's combine. A list reduction appends
each task's list in spawn order on the parent's thread, so the result is the result of running the
tasks one after another in spawn order (deterministic, where Python's order is not). Attacks: a task
that reads `total` (refused), the parent reading `total` inside the scope (refused: a conflict), a
non-associative update `total = total * 2 + 1` (not a reduction form; it is a write, refused). One
difference from a serial run: with integers that check overflow (`[TYP-8]`), a per-task partial of
mixed-sign additions can overflow where the serial sum would not (a panic, never a wrong value).

**No thread words.** With rules 1 to 4 and the shared flag (section 4.6), nothing asks for `Send`,
`Sync` or `@sync`: errors name the write that conflicts and offer a reduction, a lock field, `Atomic`,
or a struct.

**Cost, measured** (`bench/reduce.c`, `bench/reduce.rs`; 64 MiB of text in 8 parts, 8 tasks, `total
+= 1` for each of 37,278,612 words; `runs/reduce_summary.txt`; PROVISIONAL):

| Program | What it is | median ms | ratio |
|---|---|---|---|
| `reduce-0` | plain C as a C programmer writes it: a local counter per thread, summed after the join. The reduction compiles to exactly this, so it is one row | 43.548 | 1.00 |
| `reduce-1` | one shared atomic counter, `lock xadd` per word: today's Ember spelling (`total.fetch_add(1)`) | 444.946 | 10.22 |
| `reduce-2` | one counter under a lock per word: Python's `with lock: total += 1` written literally | 1453.022 | 33.37 |
| `reduce-rust-joinsum` | safe Rust: each thread returns its count, summed after the joins | 46.888 | 1.08 |
| `reduce-rust-atomic` | safe Rust: `AtomicU64::fetch_add(1, Relaxed)` (phase 1's workaround) | 450.761 | 10.35 |
| `reduce-rust-mutex` | safe Rust: `*total.lock().unwrap() += 1` | 2479.383 | 56.93 |

Peak memory is the text (69,112 KB for C, 69,564 to 69,580 KB for Rust). When each task adds only
once (Python's `total += len(t.split())`), every spelling costs the same, one add or one atomic per
task; the reduction is never slower.

### 4.5 Answer (b): a lock field guards its object. Kind (b), a smarter compile-time solution

For an object several threads change, Python programmers write a lock field and `with self.lock:`.
Rust makes them move the fields into a `Mutex<Inner>`; Ember today makes them write `@sync` and wrap
every field (`p15k_guarded_today.em`, compiles). Proposed: the Python code compiles as written.

```ember
from std.sync import Lock                 # proposed: a lock with no value inside (Mutex[void])

class Stats:
    lock: Lock = Lock()
    words: int = 0
    tags: Array[String] = []

    fn add(self, w: str):
        with self.lock:
            self.words += 1
            if w.starts_with("#"):
                self.tags.push(w.to_string())
```

```python
class Stats:
    def __init__(self):
        self.lock = threading.Lock(); self.words = 0; self.tags = []
    def add(self, w):
        with self.lock:
            self.words += 1
            if w.startswith("#"):
                self.tags.append(w)
```

Today (`p15j_guarded_lock.em`): `E1061 module 'std.sync' has no item named Lock`, and `E7004` for
sharing `stats` with tasks, help `@sync`.

**The rule.** In a class with one field of type `Lock`, every other field that is written after `init`
is *guarded* by it. Wherever an object of the class may be reached by two threads (a scope's tasks, or
an unscoped share), each read or write of a guarded field must be lexically inside `with x.lock:` on
the same handle `x` (the same variable, not reassigned in the block; `self` in methods). Each function's
summary records whether its accesses to `C.f` were guarded; a class whose summaries all say "guarded"
may be shared. Fields never written after `init` need no lock. A reference to a guarded field obeys rule
5 (section 5): it does not leave the `with` block. A class used by one thread only needs no lock at all.
Calling, while holding `x.lock`, a function whose summary takes `x.lock` again is a compile-time
error (it would wait for itself; Python's `Lock` is not re-entrant either).

**Why it is memory-safe.** Every write to a guarded field happens with the lock held, so two writes,
or a write and a read, are ordered by the lock. The "same handle" requirement stops `with a.lock:
b.words += 1`. Attacks: an access outside the `with` block in a method called from a task (refused: the
method's summary says "unguarded"); a reference to `self.tags` returned out of the block (refused by
rule 5); a guarded field holding a handle, used after the block (the handle's own object then follows
its own class's rules; its count is atomic if it was shared, section 4.6); two locks taken in opposite
orders by two threads (a deadlock, not a memory error; `[THR-4]`'s debug detector reports it).

**Cost.** The lock, as in C and Rust. Measured in section 5.4 (`lock-0`: lock, use, unlock in C,
1479.479 ms; safe Rust's `Mutex` doing the same, 2170.016 ms). No field is wrapped, so the object is
the size of the plain fields plus the lock word.

### 4.6 Answer (c): the shared flag, for objects that cross to unscoped threads. Kind (c), a new mechanism

**What the user writes** (a background worker, a channel, a shared configuration):

```ember
cfg = Config(scale=2)                                # a class; nothing written after init
worker = thread.spawn(fn(): serve(cfg, requests))   # cfg is shared; requests (a list value) moves
```

```python
cfg = Config(scale=2)
worker = threading.Thread(target=serve, args=(cfg, requests)); worker.start()
```

**The mechanism.** Today a class chooses atomic or plain counts once, by being `@sync` or not
(`[RC-4]`). Proposed, from Koka (`phase2-rc-elision.md` idea 2): every object starts with plain counts.
When an object first becomes reachable from another unscoped thread (captured by `thread.spawn` or
`jobs.submit`, sent on a channel, stored into a lock's contents or a `static`), the runtime walks the
class objects reachable from it and sets one bit, the top bit of the lean class's 32-bit count
(`phase3b-ownership.md` section 2.3), stopping at objects already flagged. A flagged object's counts are
changed with atomic instructions.

The bit costs no instruction on the plain path, because it moves a flagged count out of the range the
plain path already tests. The ownership group's lean retain tests `strong - 1 < 2^32 - 2` (the
runtime's `ember_retain_plain`); with the flag it tests `strong - 1 < 2^31 - 2`. From
`bench/asm/counts_1.s` and `counts_2.s`:

```asm
; lean, plain counts (model 1)            ; lean, with the shared flag (model 2)
mov  eax, dword ptr [r13]                  mov  eax, dword ptr [r14]
lea  edx, [rax - 1]                        lea  ecx, [rax - 1]
cmp  edx, -2                               cmp  ecx, 2147483645
jae  <slow path>                           ja   <slow path: atomic if flagged>
inc  eax                                   inc  eax
mov  dword ptr [r13], eax                  mov  dword ptr [r14], eax
; release:                                 ; release:
mov  eax, dword ptr [rcx]                  mov  eax, dword ptr [rbx]
cmp  eax, 2                                cmp  eax, 2
jb   <slow path>                           jl   <slow path>     ; signed: a flagged count is negative
dec  eax                                   dec  eax
```

The data-race side of sharing: a class object shared with an unscoped thread must have no plain field
written after the share, by anyone. The compiler checks it at the share from the summaries of the whole
build: every write to `C.f` outside `init` must be guarded by a lock field (section 4.5) or be an
`Atomic`/`Mutex` operation. This is `[THR-1]`'s "every field immutable after `init`", inferred instead
of declared, and with Python's lock field allowed. The error names the write: for `p15f`'s
`ps.words += 1`, "`PlainStats.words` is written at line 18 while a thread shares the object: guard it
with a `lock` field, or make it `Atomic[int]`".

**Why it is memory-safe.**

- Invariant: every class object reachable from a flagged object is flagged. It holds at the share (the
  walk), and stays true because a shared object's plain fields do not change after the share, and every
  store into a lock's contents, a channel or a static flags what it stores. So an unflagged object is
  reachable only from the thread that is walking it, and the walk itself races with nothing.
- Count changes on flagged objects are atomic; on unflagged objects only one thread can make them.
- The count race found by the attacker of the copy-instead-of-share design (`phase3-break-D1.md` B3: two threads copying a value out of one lock
  and racing on its count) cannot happen: anything inside a lock was flagged when it was stored.
- Values (structs, lists, strings as values) are not counted and not walked: a value moves to the thread
  and the sender's name ends there (names follow values, `phase3b-ownership.md` 2.2).

Attacks: a shared object whose field is written later (refused at the share); an object linked into a
shared object through its lock field after the share (flagged by the store); a handle copied out of a
shared object into a thread's own list (flagged object: atomic retain); a value moved to a thread that
contains handles the sender also holds (the walk flags those objects); the weak count of a flagged object
(must be atomic too: section 9 asks the parent-pointer group).

**Cost, measured** (`bench/counts.c`, `bench/counts.rs`; 262,144 objects held by 1,048,576 watchers,
32 frames; each step reads and writes `hp`; one watcher in 16 re-targets, a retain and a release; the
`-s` rows also retain and release on every step, 33,554,432 extra pairs, to make the per-operation cost
visible; `runs/counts_summary.txt` (first batch) and `runs/counts_shared_summary.txt` (second batch);
PROVISIONAL):

| Program | What it is | batch 1 median ms (ratio to plain C) | batch 2 median ms (ratio to lean plain) |
|---|---|---|---|
| `counts-0` | plain C, no header, no counts | 89.259 (1.00) | |
| `counts-1` | lean class, plain counts (the ownership proposal) | 119.361 (1.34) | 123.406 (1.00) |
| `counts-2` | lean class with the shared flag, objects not shared | 121.059 (1.36) | 128.329 (1.04) |
| `counts-3` | the flag, every object shared (atomic path out of line) | 136.234 (1.53) | 151.522 (1.23) |
| `counts-6` | the flag, every object shared, atomic path inline (saved version of 3) | | 140.675 (1.14) |
| `counts-4` | every count atomic, always (Swift's way, Rust's `Arc`) | 130.909 (1.47) | 125.635 (1.02) |
| `counts-1s` | lean plain, a pair every step | 173.617 (1.95) | 176.258 (1.43) |
| `counts-2s` | flag, not shared, a pair every step | 173.939 (1.95) | 178.752 (1.45) |
| `counts-3s` | flag, shared, out of line, a pair every step | 491.826 (5.51) | 497.262 (4.03) |
| `counts-6s` | flag, shared, inline, a pair every step | | 444.569 (3.60) |
| `counts-4s` | always atomic, a pair every step | 385.598 (4.32) | 386.218 (3.13) |
| `counts-rust-rc` / `-arc` | safe Rust, `Rc` / `Arc` (40-byte objects) | 145.424 (1.63) / 155.156 (1.74) | |
| `counts-rust-rc-s` / `-arc-s` | the same with a clone and drop every step | 169.536 (1.90) / 397.738 (4.46) | |
| `counts-5` | only the marking walk over all 262,144 objects | 5.451 | |

Reading:

- **Objects that are not shared pay nothing measurable**: 121.059 against 119.361 ms and 173.939
  against 173.617 ms in batch 1; 1.04x and 1.01x in batch 2. The instructions are the same (above), so
  the differences are noise (under 15%, provisional).
- **Objects that are shared** pay the atomic instructions, as Rust's `Arc` does: per pair about
  (385.598 - 130.909) / 33,554,432 = 7.6 ns atomic against (173.617 - 119.361) / 33,554,432 = 1.6 ns
  plain (batch 1). The first flag version put the atomic path in an out-of-line function (`counts-3s`,
  1.27x of always-atomic in batch 1). **Saved** by writing the atomic path inline after the one
  compare (`counts-6s`): 444.569 against 386.218 ms, 1.15x of always-atomic, which matches safe
  Rust's `Arc` (397.738 ms in batch 1) within that ratio. The remaining difference is the plain load and
  compare before the atomic instruction; it falls only on objects really shared with unscoped threads
  that are also counted on every step, which the lean class avoids in most code.
- **The walk**: 5.451 ms for 262,144 objects, 20.8 ns per object, once per object per program.
- Memory: the flag uses an existing bit (26,096 KB in every lean model). The maximum count halves
  (2^31 - 2), with today's overflow panic (`[RC-1]`) at the new limit.

### 4.7 Answer (c), discarded: claimed objects

The residue of 4.3 is the common Python shape "each task changes its own object", for class objects:

```ember
@parallel
for p in players:            # players: Array[Player], Player a class
    p.hp -= 1
    p.x += p.hp & 1
```

```python
with ThreadPoolExecutor(8) as ex:
    ex.map(hurt, chunks(players, 8))
```

The list may hold one object twice, and then two tasks write one object. **The mechanism:** before a
task writes an object it swaps a claim byte in the object's header to 1 (an atomic exchange); finding
1 means another task holds it: panic. It stores 0 when done. The pair also orders a later task's writes
after this one's, so the access is race-free.

**Measured** (`bench/claim.c`, `bench/claim.rs`; 1,048,576 shuffled handles, 8 tasks, 64 passes;
`runs/claim_summary.txt`; PROVISIONAL):

| Program | What it is | median ms | ratio | peak KB |
|---|---|---|---|---|
| `claim-0` | plain C, no check (32-byte objects, the lean layout) | 75.004 | 1.00 | 44,564 |
| `claim-1` | atomic claims per object | 129.658 | 1.73 | 44,572 |
| `claim-2` | saving attempt: the parent claims every object before the tasks start (plain stores, one thread), tasks clear | 316.701 | 4.22 | 44,576 |
| `claim-rust-refs-chunks` | safe Rust with values: `Vec<&mut Player>` split by `chunks_mut` (24-byte structs, no header) | 58.526 | 0.78 | 36,848 |
| `claim-rust-arc-mutex` | safe Rust with shared objects: `Vec<Arc<Mutex<Player>>>`, a lock per object | 328.983 | 4.39 | 86,328 |

**Saving it.** The cost is one atomic instruction per object per pass; no layout removes that, because
two tasks may reach one object at once and only an atomic instruction can tell. Moving the claims to the
parent (`claim-2`) removes the atomics but adds a serial pass over a million scattered headers: worse.
Proving at compile time that the list holds each object once would remove the claims, but that needs
"no other handle to a `Player` exists", which the lean class cannot know in general.
**Discarded**: 1.73x of plain C is not on par with C. The answer kept is the scope rule's: a struct
element costs nothing (Rust's 0.78x row is the value layout: smaller objects, no header), and a class
object changed by several tasks takes its lock field. **Good part kept**: a per-object claim that
panics is 2.5x cheaper than Rust's lock per object (129.658 against 328.983 ms) and half its memory;
if Ember ever needs exclusive access to shared objects from tasks, this is the mechanism.

### 4.8 Choice for A15

**The scope rule (b), with reductions (a), the lock field (b) and the shared flag (c).**

- Speed: scoped tasks cost nothing; reductions are plain C (Rust's own best spelling 1.08x, its
  `Atomic` workaround 10.35x); the flag costs unshared objects nothing (the same instructions; measured 1.00x
  to 1.04x) and shared objects Rust's `Arc` price within 1.15x in the count-heavy worst case (under
  15%, provisional); locks cost what C's lock costs.
- Safety: data-race freedom by the summary check, argued and attacked, not proven. The same guarantee
  as Rust's `Send`/`Sync`, reached without the words.
- Ease: the Python programs of 4.2 compile (the loop, `+=`, `push`, read-only class objects, a lock
  field, a background thread with moved values). What is left: a class object changed by several
  tasks needs its lock or a struct (claims were too slow); a task cannot keep a parent's object past the
  scope; an object shared with an unscoped thread must have its changing fields guarded. Each is a place
  where Python races under its interpreter lock, and each error offers a fix-it.

---

## 5. A33: references out of a lock or `RefCell` guard

### 5.1 What Rust's rule protects against

Reading lock-protected data after the lock is released, while another thread (or later code on this
thread) changes or frees it (`phase1-sharing.md` A33; `a33_lock.rs`: `E0515 cannot return value
referencing temporary value`).

### 5.2 The Python program, and what Ember does with it today

```python
def top(scores):
    with lock:
        return scores[0]       # py_checks.py: still "ann" after the list is cleared (str is immutable)
```

Ember today accepts the unsound version (`a33_lock.em`, re-checked: exit 0):

```ember
fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]
```

Re-checked today (`probes/ember_check_output.txt`): `a33_lock.em`, `a33_lock_uaf2.em` (one thread: `r =
top(m); clear(m); println(r)`, a use after free), `a33_lock_race.em` (a scoped task clears while the
main thread reads: a data race) and the new `p33d_rwlock.em` (the same through `RwLock.read()`) all
compile with exit 0; `a09_refcell_escape.em` compiles with only the `L3011` warning. A reference kept
in a local of the same function past the guard is refused (`p33c_local_escape.em`, `E3060`), so the hole
is the region a reference gets when it leaves through a function's result: it is taken from the
parameter `scores`, not from the guard (phase 1's diagnosis). `p33a_clone_fix.em`
(`return g[0].clone()`) compiles; `p33b_return_place.em` (`return g[0]` with a `String` result) is
`E3011 cannot move out of an element`.

### 5.3 Answer (a): a guard's region, and a copy of an immutable value. Kind (a)

**What the user writes:**

```ember
fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0]            # proposed: a copy of the String leaves the lock (today E3011)
```

```python
def top(scores):
    with lock:
        return scores[0]
```

**The rules.**

1. **A guard's region** (the fix of the hole, needed by every answer here): a reference read through a
   `MutexGuard`, `RwLockReadGuard`, `RwLockWriteGuard`, `RefCell` guard, or a lock-guarded field (section
   4.5) has the region of the guard: the `with` block, or the statement of a guard temporary. Returning
   it, storing it, or using it after the guard ends is a compile-time error. The guard types are already
   known to the compiler by name (`std/src/sync.em` lines 47 to 49, quoted in phase 1), so this is the
   region given to a read-through, as the references group's inferred limits do elsewhere.
2. **What leaves a lock is a value.** `return g[0]` (or `name = g[0]` used after the block) of a
   number, a `str`/`String`, or a tuple of those is a copy. That is Python's meaning: Python returns the
   same immutable `str`, which nothing can change, so a copy cannot be told apart (`py_checks.py`: the
   name is still "ann" after `clear()`). A class handle leaves as a retain (atomic: anything inside a
   lock is flagged, section 4.6). A list, map or struct does **not** leave silently: Python returns the
   shared list, and changing it outside the lock changes the guarded one (`py_checks.py`: `['ann',
   'bob']`), which is a race in Python and would be a silent lost update with a copy. That case is an
   error with fix-its: `copy(g.items)`, do the change inside the `with` block, or the accessor (5.4).

**Why it is memory-safe.** No reference outlives the lock it came through (rule 1), so nothing can
read memory another thread frees or changes. A copy is made while the lock is held. The four
probes above become errors: `a33_lock.em` (a guard-region reference returned), `a33_lock_uaf2.em` and
`a33_lock_race.em` (the same function), `p33d_rwlock.em` (the read guard), `a09_refcell_escape.em` (and
`RefCell` itself goes if the ownership group's field effects are adopted, `phase3b-ownership.md` 2.3).

**Cost**, measured below (5.4): the copy, which C pays when it copies the same way and Rust pays as
`.clone()`.

### 5.4 Answer (b): an accessor that keeps the lock while the part is used. Kind (b)

Copying is the price of Python's meaning. When the caller only needs to look, nothing needs to be
copied: the lock can stay held until the caller's last use of the part. This is the references group's
yield-accessor (`phase3b-references.md` 3.6, "kept, not chosen", which names lock guards as its use):

```ember
fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        yield g[0]                         # the caller uses the part here; the lock is released after

with name = top(board.scores):
    println(name)
```

```python
from contextlib import contextmanager
@contextmanager
def top(scores):
    with lock:
        yield scores[0]

with top(board.scores) as name:
    print(name)
```

Today: `p33e_accessor.em` is `E2220 yield outside a gen fn` (and `E2182`): new syntax.

**The rules**: the references group's (yields once on every path; the part lives in the caller like a
returned reference; the code after `yield` runs at the caller's last use of the part), plus one: **lock
effects**. A function's summary records the locks it takes ("takes the lock of `Board.scores`"). While
a part from an accessor holding lock `L` is live, a call whose summary takes `L` is a compile-time
error: it would wait for itself. A part may not be held across a `yield` or `await` (a suspended frame
would hold the lock indefinitely).

**Why it is memory-safe**: the lock is held for the whole life of the reference, so no other thread can
change what it points to; the same thread cannot either, because changing it needs the lock (refused by
lock effects) or a reference the loan rules track.

**Cost, measured** (`bench/lock.c`, `bench/lock.rs`; a table of 64 names behind a lock, 8 threads each
call `top()` 2,000,000 times and hash the name 4 times, thread 0 replaces `scores[0]` every 1024th call;
`runs/lock_summary.txt`; PROVISIONAL; this is heavy contention, 16 million lock acquisitions on one
lock):

| Program | What it is | median ms | ratio to copy-out |
|---|---|---|---|
| `lock-1` | the copy leaves the lock: lock, `malloc` and copy, unlock, hash, free. **5.3's answer**, and C doing the same | 2700.330 | 1.00 |
| `lock-0` | the name is used inside the lock: **5.4's accessor**, and C's "lock, use, unlock" | 1479.479 | 0.55 |
| `lock-2` | the copy goes to a stack buffer (C with a small known size; Ember only with an inline small-string form) | 1978.159 | 0.73 |
| `lock-3` | snapshots (5.5) | 2106.676 | 0.78 |
| `lock-rust-clone` | safe Rust: `scores.lock().unwrap()[0].clone()` | 3075.552 | 1.14 |
| `lock-rust-inplace` | safe Rust: use `&g[0]` while the guard lives | 2170.016 | 0.80 |

Peak memory 3,588 to 3,612 KB in C, 4,068 to 4,092 KB in Rust. The copy-out answer equals C copying the
same way and is 0.88x of Rust's `.clone()` (2700.330 against 3075.552 ms); the accessor equals C's
in-place use. The copy's cost here is mostly the `malloc` inside the lock, which lengthens the time the
lock is held under contention; an inline small form for strings (the strings group's choice, section 9)
would bring copy-out to `lock-2`'s 0.73.

### 5.5 Answer (c), not chosen: snapshots

Readers take the current version of the table with a shared lock and an atomic count increment, then
use it with no lock; a writer copies the table, swaps the pointer, and the last reader of the old
version frees it. This is exactly Python's meaning (a name taken out stays valid however the table
changes later, with no copy of the name) and readers never wait for each other. `lock-3`: 2106.676 ms,
0.78x of copy-out, 1.42x of in-place use; each write copies all 64 names. **Not chosen as the
language answer**: it is a library type (no rule changes), it pays an atomic pair per read and a full
copy per write, and for A33's problem (a reference leaving a guard) rules 1 and 2 are cheaper and need
nothing new. **Kept** as a library option for data that is read far more than written.

### 5.6 Choice for A33

**The guard's region with copy-out of immutable values (a), plus the accessor (b) for zero copies.**
Speed: copy-out = C copying the same way, 0.88x of Rust; the accessor = C's in-place use, 0.68x of
Rust's in-place (1479.479 against 2170.016 ms; Rust's `Mutex` differs from the SRW lock the C models
use). Safety: the hole closes; the same guarantee as Rust's `E0515`. Ease: `return g[0]` works for
names and numbers; returning a list needs `copy(...)` or the accessor, because Python's version races.
Meets all three goals.

---

## 6. A19: global mutable state

### 6.1 What Rust's rule protects against

Two threads updating `FRAME` at once (lost updates), and, on one thread, a function holding a reference
into `LOG` while a function it calls pushes to `LOG` and reallocates it (a use after free)
(`phase1-sharing.md` A19; `a19_globals.rs`: `E0133`, `E0596`).

### 6.2 The Python program, and what Ember does with it today

```python
FRAME = 0; LOG = []
def tick():
    global FRAME
    FRAME += 1; LOG.append("tick")
```

Ember today (`a19_globals.em`, `static`): `E1010 cannot find FRAME` (misleading) and `E3021`.
`p19c_module_assign.em`, Python's exact spelling, gives `E1030: main is already declared in this
module` at `FRAME = 0` (module-level statements form an implicit `main`), `E0100: global is Python's;
Ember declares no names as shared`, and `E1010` for both names. `p19a_var.em` (`var FRAME`) gives
`E0100 var is not needed` and the same `E1030`. The workaround (`a19_workaround.em`: `static FRAME:
Atomic[int]`, `static LOG: Mutex[Array[String]]`) compiles.

### 6.3 Answer (a): module variables, Python's spelling. Kind (a)

```ember
FRAME = 0                         # a module variable: an assignment at module level
LOG: Array[String] = []

fn tick():
    global FRAME                  # needed to assign, as in Python
    FRAME += 1
    LOG.push("tick".to_string())  # changing through the name needs nothing, as in Python
```

```python
FRAME = 0
LOG = []
def tick():
    global FRAME
    FRAME += 1
    LOG.append("tick")
```

**The rules.**

1. An assignment at module level declares a module variable. Its initialiser is evaluated when the
   program is compiled where it can be (`0`, `[]`, most of them), and otherwise on first access with
   `[STA-3]`'s run-once rule, so `[STA-2]`'s "no initialisation-order problem" still holds. In a file
   that has no `fn main`, top-level statements still run as the implicit `main`, and their assignments
   are module variables too (Python's meaning).
2. Assigning a module variable inside a function needs `global NAME` (Python's rule). Writing
   `FRAME += 1` without it is a compile-time error whose fix-it adds the line (Python raises
   `UnboundLocalError` when it runs); `FRAME = 1` without it declares a local, with a warning that it
   hides the module variable.
3. **One thread, loans**: a module variable is a place like a field of the module, in every effect
   summary (the ownership group's field effects). A loan into `LOG` across a call whose summary
   reshapes `LOG` is a compile-time error naming the call (or runs, under the containers group's loop
   rule).
4. **Scoped tasks**: module variables are lent to tasks exactly like the parent's locals (section 4.3),
   so tasks may read them while nobody writes them, and `WORDS += n` or `LOG.push(x)` from tasks is a
   reduction (section 4.4; this is the (c)-kind part of the answer, section 6.5).
5. **Unscoped threads**: a function reachable from `thread.spawn`, `jobs.submit` or a C callback whose
   contract is not `threads(main)` that touches a module variable is a compile-time error (its summary
   says so), with fix-its: pass the value in, or a `static` with a lock field or `Atomic`.

**Why it is memory-safe.** Rule 3 is the loan rule every place already has; rule 4 is the scope rule;
rule 5 means only the main thread and its scoped tasks ever touch a module variable, and the scope rule
orders those. Attacks: a loop over `LOG` calling a function that pushes to `LOG` (refused, rule 3); an
unscoped thread printing `FRAME` (refused, rule 5); a C callback on another thread touching `LOG`
(refused unless its contract says `threads(main)`, which the checked trampoline of section 8 then
checks); two scoped tasks writing `FRAME` other than by a reduction (refused, a conflict).

**Cost.** The compiler emits plain C globals: nothing is counted or checked. Measured anyway, with a
surprise (`bench/globals.c`, `bench/globals.rs`; `tick()` not inlined, called 100,000,000 times: `FRAME +=
1`, `LOG.push(FRAME)`, the log summed and cleared every 4096 pushes; `runs/globals_summary.txt`;
PROVISIONAL):

| Program | What it is | median ms | ratio |
|---|---|---|---|
| `globals-0` | plain C globals as clang emits them (one global per variable) | 192.219 | 1.00 |
| `globals-5` | the module's variables as one struct global, named directly | 194.921 | 1.01 |
| `globals-4` | the same struct, reached through a base pointer loaded once per call | 105.666 | 0.55 |
| `globals-3` | thread-local globals (`__declspec(thread)`) | 110.600 | 0.58 |
| `globals-1` | today's Ember workaround (`Atomic` + `Mutex`) in a one-thread program (the runtime's lock skips its atomics then) | 595.569 | 3.10 |
| `globals-2` | the same after a second thread has started | 1374.172 | 7.15 |
| `globals-rust-threadlocal` | safe Rust: `thread_local!` with `Cell` and `RefCell` | 200.234 | 1.04 |
| `globals-rust-atomic-mutex` | safe Rust: `AtomicU64` (Relaxed) and `Mutex<Vec<i64>>` (phase 1's workaround) | 1430.346 | 7.44 |

The surprise: `globals-0`, `globals-5` and `globals-4` run the same instructions on the same data
(`bench/asm/globals_0.s`, `globals_4.s`, `globals_5.s`); only the addressing differs (from the instruction
pointer, against from a base register), and the base-register form took 0.55x the time on this CPU,
the same as the thread-local form, which uses a base register too. Why the CPU treats the two forms so
differently here is **UNVERIFIED** (not isolated further). It means the C backend should emit a module's
variables as one struct reached through a base pointer; then the proposal is 0.55x of clang's default
and 0.53x of Rust's thread-local idiom. The same may apply to today's `static`s (section 11).

### 6.4 Answer (b), discarded as the default: thread-local module variables

Every thread gets its own copy of each module variable, so no race can exist. Measured: `globals-3`,
0.58x, as fast as the best plain form. **Discarded as the meaning of a module variable** because it is
not Python's: in Python a worker thread sees the main thread's `FRAME` (`py_checks.py`: the worker reads
41), and a thread-local version would read 0. No fix keeps both meanings. **Good part kept**: a
thread-local variable costs nothing here, so `@thread_local` on a module variable is a free option when
that meaning is wanted.

### 6.5 Answer (c): merged at the join; and one discarded

- **Kept: module variables changed by scoped tasks are merged at the join.** This is section 4.4's
  reduction applied to module variables (rule 4 above). Python's `global WORDS` with a lock becomes:

  ```ember
  WORDS = 0
  fn count(texts: Array[String]):
      with scope = thread.scope():
          for t in texts:
              scope.spawn(fn(): WORDS += t.split_whitespace().count())    # a reduction
  ```

  ```python
  WORDS = 0; lock = threading.Lock()
  def count(texts):
      def work(t):
          global WORDS
          with lock: WORDS += len(t.split())
      ...
  ```

  Today: `p19b_var_reduce.em` (with `var`) fails on `var` itself. Cost: section 4.4's table (1.00x of
  plain C; the per-word lock is 33.37x).
- **Discarded: worker threads post their changes to the main thread.** `LOG.push(x)` in an unscoped
  worker would be sent as a message and applied by the main thread later. Counterexample no fix
  removes: the worker cannot read its own write (`LOG.push(x); print(len(LOG))` prints the old length),
  which Python does (`py_checks.py`: the worker reads 1). Not measured, because the counterexample
  decides it. **Good part**: deferred command lists are how games apply changes from workers; that is a
  library pattern over channels (`[THR-15]`), not a rule.

### 6.6 Choice for A19

**Module variables with Python's spelling (a), merged at the join in scoped tasks (c).** Speed: plain
globals (and 0.55x of clang's default if emitted through a base pointer), against today's workaround at
3.10x and 7.15x and Rust's synchronised workaround at 7.44x. Safety: the loan rule, the scope rule, and
no unscoped thread may touch them: Rust's guarantee. Ease: Python's text, `global` included. What is
left: an unscoped thread uses a `static` with a lock field or `Atomic`, as in Rust. Meets all three goals.

---

## 7. A14: async (`Pin`, `Send`, borrows across `await`)

### 7.1 What Rust's rule protects against

(`phase1-sharing.md` A14, section 5): **`Pin`**: a suspended frame moving while a pointer into itself
lives (without it, `for s in &buf { ... .await }` holds `&buf` into the frame's old location after the
executor moves the future); **`Send`**: a task resumed on another thread touching non-atomic counts;
**`'static`**: a task outliving the stack it borrows.

### 7.2 The Python program, and what Ember does with it today

```python
async def serve(cache, out):
    n = await fetch()
    out.append(len(cache) + n)
async def main():
    out = []; cache = [1, 2, 3]
    async with asyncio.TaskGroup() as tg:          # py_checks.py: [4, 4]
        tg.create_task(serve(cache, out)); tg.create_task(serve(cache, out))

def lines():
    buf = ["a", "b"]
    for s in buf:
        yield s                                      # py_checks.py: ['a', 'b']
```

Ember today: `async` and `await` are reserved (`p14c_async.em`: `E0005 async is reserved for v2`).
Generators share async's frame lowering (`[CORO-4]`, §XI.6), and refuse: `a14_generators.em` gives
`E2221` (a generator walking its own local list) and `E2229` (a loop over a class field across `yield`);
the new `p14a_heap_contents.em` (`for c in s.chars()` over its own local `String`) is `E2221` too, and
`p14b_inline_local.em` (`r = ref n` held across `yield`) is `E2221`.

### 7.3 Answer (b): the heap-contents rule, and in-place generators. Kind (b)

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    for s in buf:                 # today E2221; proposed: allowed
        yield s.clone()

class Door:
    log: Array[String] = []
    gen fn replay(self) -> Generator[int]:
        for entry in self.log:    # today E2229; proposed: allowed when consumed in place (rule 2)
            yield entry.len()

for n in door.replay():           # the loop body does not change Door.log
    println(n)
```

```python
def lines():
    buf = ["a", "b"]
    for s in buf:
        yield s

class Door:
    def replay(self):
        for entry in self.log:
            yield len(entry)
```

**The rules.**

1. **Heap contents.** A loan into the heap contents of the frame's own local (a `String`'s bytes, an
   `Array`'s elements, a `Box`'s inside) may be held across `yield` and `await`. Moving the frame moves
   only the headers (pointer, length, capacity); the buffer stays where it is, so the loop's position
   still points into it. The local itself may not be reshaped while the loan lives (the ordinary loan
   rule). A loan into the frame's *inline* storage (`r = ref n` for a local `n`, a fixed array stored in
   the frame) stays refused (`p14b`), because that storage does move. This is the references group's
   heap-contents fact (`phase3b-references.md` section 9 names this use).
2. **In-place consumption.** A loan into a class object's field (`for entry in self.log`) is broken only
   by a reshape of that field by code that runs while the generator is suspended. When the generator is
   consumed by a `for` loop in the same function and never stored or passed on, that code is the loop's
   body, whose summary the compiler has: if it does not reshape `Door.log`, the loan is allowed. If the
   generator is stored, passed or returned, `E2229` stays.
3. **Async, when it comes**: an async task lives in a task group (`async with tasks = TaskGroup():`, Python
   3.11's own spelling), which is a scope. Its frame is allocated once when spawned and never moves, so a
   loan of its own inline locals across `await` is allowed too (no `Pin` in the language); `Send` is the
   scope rule of section 4.3 (tasks that may run at the same time are checked against each other, on one
   thread or many); `'static` is not needed, because the group joins before its block ends. Fire-and-forget
   tasks are unscoped threads' rules (section 4.6).

**Why it is memory-safe.** Rule 1: the only memory that moves with a frame is the frame; heap contents
are freed only by a reshape or drop of the local, which the loan rule refuses while the loan lives, and
dropping the frame drops the loan with it. Rule 2: between two resumes only the consumer's loop body runs,
and its summary is checked against the loan. Rule 3: a frame that never moves has no dangling self-loan,
and the group's join is a scope's join. Attacks: `for c in s.chars(): s.push('x')` inside the generator
(refused, a reshape under a loan); yielding a reference into the frame's own data to the consumer (stays
refused: the consumer could keep it past the next resume, Rust's "lending iterator" problem); storing a
suspended generator in a list that grows (heap contents do not move: safe; inline loans were refused);
the consumer's loop calling a function that pushes to `door.log` (refused by rule 2, naming the call).

**Cost.** Nothing: the frame is today's frame. Measured to confirm (`bench/gen.c`; a generator over
the bytes of its own 1 MiB local `String`, 100 runs, 104,857,600 yields; `runs/gen_summary.txt`;
PROVISIONAL):

| Program | What it is | median ms | ratio |
|---|---|---|---|
| `gen-0` | plain C: a loop over the bytes, no generator | 70.990 | 1.00 |
| `gen-2` | the proposed frame, `next()` inlined into the consumer's loop | 80.767 | 1.14 |
| `gen-1` | the proposed frame, `next()` not inlined (1.97 ns per value) | 206.186 | 2.90 |
| `gen-4` | today's workaround (E2221's help: iterate by index, bounds-checked), not inlined | 204.695 | 2.88 |

The proposal costs what today's workaround costs (206.186 against 204.695 ms, under 1%); the gap to plain
C is the generator protocol itself (a call per value when not inlined), not this rule.

### 7.4 Answer (c), discarded: stackful coroutines

Give each generator or task its own stack (a Windows fiber here), and `yield` switches stacks. No
function colouring, no `Pin`, any loan across a suspension, blocking calls work. **Measured**
(`bench/gen.c` model 3; `runs/gen_summary.txt` and `runs/genmem_summary.txt`; PROVISIONAL):

| | stackless frame | fiber |
|---|---|---|
| 104,857,600 yields | 206.186 ms (not inlined), 80.767 ms (inlined) | 4791.856 ms: 45.7 ns per value, 67.5x plain C |
| 10,000 generators suspended at once | 4,772 KB peak (about 135 bytes each over plain C's 3,452 KB) | 58,812 KB peak (about 5.4 KB each over the frames) |

**Saving it.** A smaller stack still costs at least one committed page per suspended coroutine (4 KB on
x86-64), against a frame of exactly its live locals; a hand-written switch would be cheaper than
`SwitchToFiber` (not measured: it needs assembly); growable stacks need either stack copying, which must
find and fix every pointer into the stack (a garbage collector's pointer maps, which the C backend cannot
produce), or segmented stacks, which thrash at a segment boundary. The memory counterexample is not removed
by any of them. **Discarded.** **Good parts**: no function colouring, and blocking code runs unchanged
inside a coroutine; worth remembering if Ember ever targets servers with many blocking calls.

### 7.5 Choice for A14

**The heap-contents rule and in-place generators (b) now; task groups with frames that never move (a)
and the scope rule for `Send` when async comes.** Speed: today's frame, measured equal to today's
workaround. Safety: the same as Rust's `Pin` + `Send` + `'static`, by the rules above. Ease: Python's
generator text compiles; async would be Python 3.11's `TaskGroup` text. What is left: a reference to an
inline local across `yield` (rare: a reference to a local number) stays refused unless the frame never
moves. Meets all three goals for generators; async is design only, because `async` is reserved for v2.

---

## 8. A20: unsafe code and FFI

### 8.1 What Rust's rule protects against

C calling a callback after the state it captured is gone (a use after free), or from another thread (a
data race) (`phase1-sharing.md` A20). Rust's workaround (`a20_ffi.rs`) is an `extern "C" fn` that turns
the user-data pointer back into `&'static mut Game` inside `unsafe`, a reference whose life nothing
bounds.

### 8.2 The Python program, and what Ember does with it today

```python
CB = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
lib.register_cb(CB(lambda ud: setattr(game, "score", game.score + 1)), None)
# py_checks.py runs the same callback three times: score 3
```

Ember today (`a20_ffi.em`, re-checked): without a contract, `E5040` with a help that names the contract;
with `@ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))` on the
declaration, `token = register_cb(owned fn(): game.score += 1)` compiles with no `unsafe`. The contract is
an asserted fact, listed by `ember tcb` (`[FFI-22]`, `[TCB-1]`): if C keeps calling after `unregister`, or
calls from another thread, Ember trusts it. A declaration with no contract (`p20b_inferred.em`) gives
`E2020 register_cb takes 2 arguments, found 1` and `E5002`, whose help writes the `TODO(...)` contract out.

The trampoline today's compiler writes (`emit_ffi_callback`, `compiler/ember_codegen_c/src/lib.rs` line
6833) first calls `ember_rt_thread_attach()`, an out-of-line function in `runtime/ember_rt/src/ember_rt.c`
(line 2425) that compares a thread-local generation with a global one, then calls the closure with the
record at the user-data pointer.

### 8.3 Answer (c): checked trampolines. Kind (c), a new mechanism

**What the user writes**: the same as today (or less, with 8.4):

```ember
token = c.register_cb(fn(): game.score += 1)
```

```python
lib.register_cb(CB(lambda ud: setattr(game, "score", game.score + 1)), None)
```

**The mechanism** (phase 1's A20 (b), "C holds handles, never Ember addresses", made fast):

- Registration puts the closure's state in a slot of a runtime table and hands C, as the user data, not
  an address but `slot | generation << 32`. Unregistering (dropping the token) bumps the slot's
  generation, so every value C still holds is stale.
- The trampoline checks the slot number is inside the table (C can hand back any value), compares the
  generation (a stale call runs nothing and panics with the callback's name), compares the calling
  thread with the slot's creator when the contract is `threads(creator)` or `threads(main)` (a thread-local
  id; a mismatch panics), then runs the closure. The thread compare also proves the thread is attached,
  so today's out-of-line attach call goes.
- A closure whose body may drop its own token (its summary says it drops a callback token) is counted
  as running around the call, and unregistering frees its state only at a count of 0. Other closures
  skip this, decided when compiling.
- A slot whose generation reaches its maximum is retired, never reused.

**Why it is memory-safe, and better than Rust.** C can only hand back a number. A stale number fails the
generation compare, a made-up one fails the range check, a wrong thread fails the thread compare; none
of them reaches freed memory or races. The facts today's contract *asserts* (C stops calling after
`unregister`; C calls from the creator thread) are *checked*. Rust's workaround cannot check them: its
reference is `unsafe` with an unbounded lifetime. Attacks: a call after the token is dropped
(generation); a forged user-data value (range); a call from C's worker thread under `threads(creator)`
(thread); a callback that unregisters itself (the running count, emitted because its summary shows the
token drop); 2^32 registrations through one slot (retired). `threads(any)` callbacks skip the thread
compare and share their captures (section 4.6's rules: the flag, guarded fields).

**Cost, measured** (`bench/ffi.c`, `bench/ffi.rs`; a C "library" function, not inlined, calls the callback
through a pointer it cannot see through, 200,000,000 times; the callback adds 1 to `game.score`;
`runs/ffi_summary.txt`; PROVISIONAL; assembly in `bench/asm/ffi_1.s`, `ffi_2.s`, `ffi_4.s`):

| Program | What it is | median ms | ratio |
|---|---|---|---|
| `ffi-0` | plain C: the user data is `Game*` | 158.406 | 1.00 |
| `ffi-1` | today's trampoline: out-of-line attach, record, closure | 233.083 | 1.47 |
| `ffi-2` | the checked trampoline: range, generation, thread, closure | 162.857 | 1.03 |
| `ffi-4` | the checked trampoline with the running count (a body that may drop its own token) | 204.247 | 1.29 |
| `ffi-3` | `ffi-2` plus today's out-of-line attach (if the attach could not be folded in) | 258.517 | 1.63 |
| `ffi-rust-rawptr` | safe-outside, `unsafe`-inside Rust: `&mut *(ud as *mut Game)` | 154.223 | 0.97 |

The checked trampoline is within 3% of plain C and faster than today's (the loads it adds are off the
loop's critical path, and it drops the out-of-line call). The running count costs 1.29x because its
increment and decrement chain through memory from one call to the next; it is emitted only for bodies
whose summary shows they may drop a callback token. Memory: one 24-byte slot per registered callback
(the model's `Slot`) instead of today's record.

### 8.4 Answer (a): the contract inferred from the header. Kind (a)

When a C function takes a function pointer and a `void*`, and the function pointer's own parameters
include a `void*`, the importer infers `callback(retained, user_data=<that parameter>)` with
`threads(creator)`, the safe defaults, so the call site has no annotation (the example in 8.3). Cost:
none beyond the trampoline. What it gives up: an inferred fact is still a guess; a `void*` that is not
user data would be wrong. With 8.3, retention and the thread are checked, so the only asserted fact left
is "this `void*` is the user data"; it is listed by `ember tcb` like a written one, and a shape that does
not match exactly stays `E5040` with today's help. The call site also needs the closures group's "a
lambda passed where the callee keeps it captures by retain" (today `owned fn` is required, `[FFI-21]`).

### 8.5 The `unsafe` half, and an answer not chosen

- **Raw pointers.** Ember already has `unsafe:` blocks with `# SAFETY:` notes and `ember tcb`
  (`[UNS-8]`). One rule makes it easier than Rust's: a reference made from a raw pointer gets the
  region of the enclosing `unsafe` block, never an unbounded one (Rust's unbounded lifetimes are the
  Rustonomicon's main hazard, phase 1). Cost: none (a compile-time region). Python has no equivalent
  (`ctypes` is unchecked everywhere).
- **Not chosen, kind (b): read the C library's source.** The importer already runs Clang; where the C
  source is available, it could see whether the function stores the callback (retained) or only calls it
  during the call (borrowed), and on which thread. Feasibility is UNVERIFIED (function pointers through
  casts, unions and globals defeat it; binary-only libraries have no source); not measured, not chosen.

### 8.6 Choice for A20

**Checked trampolines (c) with inferred contracts (a).** Speed: 1.03x plain C per callback (under 15%,
provisional), faster than today's 1.47x; 1.29x only for a callback that may unregister itself. Safety:
better than Rust's (the retention and thread facts are checked, not trusted). Ease: Python's call site.
Meets all three goals.

---

## 9. How the answers fit together, and what they need from other groups

How they fit: one mechanism decides every thread question, the effect summary. The scope rule (A15) is
also what makes module variables safe in tasks (A19), what replaces `Send` for async task groups (A14),
and what decides which callbacks may touch module variables (A20). Lock effects (A33's accessor) and
lock-guarded fields (A15) are two facts in the same summaries. The shared flag (A15) is what makes a
handle copied out of a lock (A33) safe, and what `threads(any)` callbacks (A20) use. The count is just
another field in the summaries, so the ownership group's "names follow values" and the lean class carry
over to threads with no new run-time machinery except the flag.

| From | What | Used by | If not given |
|---|---|---|---|
| ownership (A1, A9) | field effects: read, write and reshape facts per field, class-wide for objects, in every function's summary and in package interfaces | the scope rule, lock fields, module variables, in-place generators | the scope rule falls back to today's whole-variable borrows (`[THR-11]`), `@sync` stays |
| ownership | "the count of some `C`" as one more field in summaries, with fresh allocations tracked separately | rule 4 of the scope rule | counts of parent objects in tasks must be atomic, i.e. `@sync` stays |
| ownership (lean class) | one free bit at the top of the 32-bit count | the shared flag | an extra header word, or Rust's choice of atomic counts per type |
| ownership (A31) | a `@cached` slot written on first read uses a once-flag (plain while one thread runs, like the runtime's lock shortcut) | tasks calling `@cached` getters | such calls conflict in tasks |
| references (A5, A6) | a handle read from a borrowed place, kept within that place's life, is a view: no count | the scope rule's rule 4 | tasks retain parent objects: atomic counts again |
| references (A7, A22) | the heap-contents fact | A14 rule 1 | `E2221` stays |
| references (3.6) | yield-accessors | A33's accessor | A33 keeps only the copy-out answer |
| containers (A3) | `iter_mut`, `chunks_mut`, `split_at` hand out distinct elements | the scope rule's disjoint writes | `p15c` stays refused |
| closures (A12, A32) | a closure's effects are its body's; a lambda passed where the callee keeps it captures by retain, with no `owned` | the scope rule, A20 (a) | `owned fn` at FFI call sites |
| strings (A22, A25) | whether `String` stays a value (then copy-out copies, `lock-1`) or becomes a shared immutable buffer (then copy-out is a retain, flagged); an inline small form would make the copy `lock-2`'s 0.73 | A33 rule 2 | copy-out stays a heap copy |
| parent pointers (A8) | the weak count, when it lives in a side record, uses atomics for flagged objects | the shared flag | weak handles cannot cross threads |
| hot reload (Annex B) | a reload that widens a summary a scope check relied on is refused (`phase3-break-D3.md` 1.6's fix) | the scope rule | a reload could add a race |
| error messages (A17) | every residue above is an error with a fix-it the checker re-runs before showing | all | |

What I answer for the ownership group's request (`phase3b-ownership.md` section 5, the threads row): "a
value two threads can reach is not changed by either" is the scope rule (scoped) and the share rule of
section 4.6 (unscoped); "counts of objects that cross threads are atomic" is the shared flag; "`@cached`
shared across threads uses a once-flag" is in the table above.

---

## 10. Discarded answers, and the good parts kept

| Answer | From | Why discarded (what no fix removes) | Good part kept |
|---|---|---|---|
| claimed objects (per-object atomic claim for tasks) | mine | 1.73x plain C on a tiny body (section 4.7); one atomic instruction per object per pass is the minimum to detect two tasks on one object; the parent-side version is 4.22x | 2.5x cheaper than Rust's lock per object, half its memory |
| thread-local module variables by default | phase 1 A19 (b) | not Python's meaning: a worker reads its own `FRAME` (0) instead of the main thread's (41, `py_checks.py`) | free when wanted: `@thread_local` (0.58x, section 6.4) |
| workers post their global changes to the main thread | mine | a worker cannot read its own write (`py_checks.py` shows Python can) | deferred command lists as a library pattern over channels |
| stackful coroutines (fibers) | Go, Java's virtual threads (from memory, UNVERIFIED) | 45.7 ns per value and about 5.4 KB per suspended task against about 135 bytes (section 7.4); a stack needs at least one page | no function colouring; blocking code runs unchanged |
| per-class atomic counts decided for the whole program | today's `@sync` inferred | every object of a shared class pays atomics even when only one is shared; with separately compiled packages the choice is a load and a branch per count anyway (no better than the flag) | none beyond the flag |
| biased reference counting (an owner thread per object) | Swift's BRC paper (`phase2-swift-mojo.md` S4) | a thread-id test on every count operation and more header bytes, where the flag's test is free (section 4.6) | an object stays cheap for its owner after sharing |
| "move it or freeze it" with an isolation check at every spawn | Pony's `iso`/`val`, last night's "today's rules, but never an error" design (`phase3-design-D3.md` 1.7) | proving a moved graph has no outside handle needs a capability discipline that the checked-pointers design's attacker called "an A15 annoyance in another form" (`phase3-break-D2.md` H2) | values move freely (kept: names follow values); "frozen" survives as "no plain field written after the share", inferred |
| values that copy on write between threads | the copy-instead-of-share design (`phase3-design-D1.md` D1-9) | its own A33 example races on a count (`phase3-break-D1.md` B3); tasks cannot share closure state (N6) | deep copies only where a value leaves a lock (A33 rule 2) |
| per-thread stores with generation-checked references | the checked-pointers design (`phase3-design-D2.md` 1.7) | references resolve against the wrong thread's store (`phase3-break-D2.md` H2) | a generation check is the right tool across the C boundary (A20) |
| snapshots as the A33 answer | RCU, `arc-swap` (from memory, UNVERIFIED) | a library type, an atomic pair per read and a full copy per write; rules 1 and 2 are cheaper | kept as a library option for read-mostly data (`lock-3`, 0.78x of copy-out) |
| reading the C library's source to derive callback facts | mine | feasibility UNVERIFIED; binary libraries have no source | could turn more asserted facts into derived ones |

---

## 11. Found in passing (today's compiler; nothing was changed)

1. **The A33 soundness hole is still there, and also passes through `RwLock`.** `a33_lock.em`,
   `a33_lock_uaf2.em`, `a33_lock_race.em` and the new `p33d_rwlock.em` (a reference from `read()`
   returned, then `clear` under `write()`) pass `ember check` with exit 0. A reference kept in a local of
   the same function is caught (`p33c_local_escape.em`, `E3060`).
2. **Knock-on errors on the wrong variable.** In `a15_threads.em`, `E3021 t cannot be written while it is
   borrowed` and `E3060 t does not live long enough` appear only because the same lambda also writes
   `total`: reading `t` alone compiles (`p15a_loopvar_read.em`), and writing `total` alone gives only
   `E3022` (`p15g_write_only.em`).
3. **The "cannot find" bug in lambdas still blocks the natural `Atomic` spelling** (phase 1's defect 2):
   `p15h_read_t_atomic.em`, `fn(): total.fetch_add(t.split_whitespace().count())`, gives `E1010 cannot
   find t in this scope`.
4. **A confusing message for module-level assignments.** `FRAME = 0` (or `var FRAME: int = 0`) at module
   level in a file with `fn main` gives `E1030: main is already declared in this module`, pointing at
   `FRAME` (`p19a_var.em`, `p19c_module_assign.em`). The cause is that module-level statements form an
   implicit `main`; the message should say so.
5. **`ref String` is not accepted where `str` is expected** at a call argument (`E2020 expected str,
   found ref String`, first version of `p15j`/`p15k`, kept at the end of `ember_check_output.txt`).
6. **Today's FFI trampoline makes an out-of-line call on every callback** (`ember_rt_thread_attach`):
   1.47x plain C per callback (`ffi-1`), where the checked trampoline, which proves attachment with an
   inline thread-local compare, is 1.03x (section 8.3).
7. **Globals addressed from the instruction pointer ran 1.8x slower than through a base pointer** on this
   CPU for a load-modify-store per call (`globals-0` 192.219 ms against `globals-4` 105.666 ms, same
   instructions otherwise; cause UNVERIFIED). Today's `static`s are emitted as plain C globals, so they may
   be affected; worth a look by whoever owns the C backend.

---

## 12. Files

All under `target/redesign2/threads/` unless a path says otherwise.

- `probes/`: Ember probes. Re-checked from phase 1: `a14_generators.em`, `a15_threads.em`,
  `a15_workaround4.em`, `a19_globals.em`, `a19_workaround.em`, `a20_ffi.em`, `a33_lock.em`,
  `a33_lock_uaf2.em`, `a33_lock_race.em`, `a09_refcell_escape.em`. New: `p14a_heap_contents.em`,
  `p14b_inline_local.em`, `p14c_async.em`, `p15a_loopvar_read.em` to `p15k_guarded_today.em`,
  `p19a_var.em`, `p19b_var_reduce.em`, `p19c_module_assign.em`, `p20b_inferred.em`,
  `p33a_clone_fix.em` to `p33e_accessor.em`. All output: `probes/ember_check_output.txt`.
- `rust_probes/`: `r15a_iter_mut_scoped.rs`, `r15b_rc_scoped_read.rs`, `r15c_push_scoped.rs`;
  output `rustc_output.txt`.
- `python/py_checks.py` and `py_checks_output.txt`.
- `bench/`: `counts.c`/`counts.rs` (the shared flag), `reduce.c`/`reduce.rs` (reductions),
  `globals.c`/`globals.rs` (module variables), `ffi.c`/`ffi.rs` (trampolines), `lock.c`/`lock.rs`
  (A33), `gen.c` (generators and fibers), `claim.c`/`claim.rs` (claimed objects); `run_group.sh`
  (calls `target/redesign2/ownership/bench/run.ps1` and `medians.py`); `bin/` (the programs); `asm/`
  (assembly quoted in sections 4.6, 6.3 and 8.3); `runs/<group>.txt` (every run) and
  `runs/<group>_summary.txt` (the tables).
