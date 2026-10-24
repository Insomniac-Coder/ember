# Phase 3b: one memory model for Ember, put together from the surviving answers (second run)

Written 2026-10-24 for the owner, under the job in [JOB.md](JOB.md): speed on par with C and Rust,
memory safety the same as Rust's or better, and Python's ease of reading and writing, all at once, with
the checking done when the program is compiled.

**Why there is a second run of this step.** The first combined report was written at 08:42 from the
first runs of the five attacks. All five were then run again (four as second attacks, the containers
attack as a second pass in the same file), because some agents had started their programs through
`cmd /c start`, which opened "Windows cannot find" boxes on the owner's screen
(`phase3b-ownership-attack.md`, "Why there is a second attack"); the owner asked "what about the agents
that already ran with that problem?". The second runs replaced the first runs' reports and corrected some
of the numbers the first combined report rested on. This report is built from the final
reports. The first combined report is kept unchanged as `superseded/phase3b-combined-first-run.md`, and
its C files as `target/redesign2/combined/first-run/` (only their `#include` line was changed, so they
still build from the deeper folder). Section 2.1 answers the owner's question for every input this
report uses; section 0.4 lists what changed from the first run.

What I read: `JOB.md`; the five group reports (`phase3b-ownership.md`, `phase3b-containers.md`,
`phase3b-references.md`, `phase3b-graphs-callbacks.md`, `phase3b-threads.md`); their final attacks
(`phase3b-ownership-attack.md`, `phase3b-containers-attack.md` (two passes), `phase3b-references-attack.md`,
`phase3b-graphs-callbacks-attack.md`, `phase3b-threads-attack.md`, each ending with its group's final
answer list) and, where they cite them, the first runs (`phase3b-*-attack-first-run.md`,
`superseded/phase3b-ownership-attack-first-run.md`); `phase3b-salvage.md`; the first combined report;
last night's `FINDINGS.md` and `phase3-measurements.md`; and the C models and run files those reports
name.

What I made (all under `C:/Users/ism19/Code/ember/target/redesign2/combined/`, called `combined/` below):

- `w1.c` to `w4.c`: what this model's compiler would emit for the four workloads, using the shared harness
  `target/redesign/bench/harness.h` (`h_start()` first, `h_report("combined-wN", checksum)` last).
  `w3_registration.c`: W3 with the other `weak` mechanism, kept for the measurer (section 4.3).
- Self-check builds: `w1.c -DCURSOR_CHECK` (and `-DNO_CURSOR_RULE`, which must fail), `w2.c -DLINK_CHECK`,
  `w3.c -DPOISON`, `w4.c -DPOISON` (section 2.2).
- `ember/w1_main.em`, `w2.em`, `w3.em`, `w4.em`: the four programs in this model's syntax, with what
  today's `ember check` says about them (`ember/ember_check_output.txt`).
- `python/run_twins.ps1`: I ran the Python twin of every workload once; each printed its known checksum
  (`python/run_twins_log.txt`).
- `run_once.ps1` and `run_once_output.txt`: one run of every model, self-check and same-layout baseline,
  for the checksums and the peak memory.

**No timing was done.** Every time ratio below comes from an earlier report, names its file, and is
PROVISIONAL there (other agents were running). What I measured is the checksum and the **peak memory**
(the harness's `peak_kb`, the process's peak working set) from one run of each program, started by
PowerShell's `Start-Process -NoNewWindow` (no `cmd`, no `start`, no window), pinned to the efficiency cores
(affinity mask 0x3FC3FC) to stay off the cores other agents time on, after
`(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 (mains). The `ms=` field each program prints in
those runs is not a measurement and is not used anywhere here. Nothing in `compiler/`, `std/`, `docs/` or
`tests/` was touched; no Ember program was built or run (only `ember check`); no git command was run.

---

## 0. The short version

### 0.1 The model in six sentences

1. There are **two kinds of values**: a `struct` (and every list, map, string and number) lives in one
   place and is never shared, and a `class` object lives on the heap and is shared through counted
   handles, as every Python object is.
2. `b = a` makes `b` a **second name** for the same value, as in Python; the compiler keeps a name as a
   path fixed when the name is made ("element 3 of `enemies`", "the value of key `"a"` in `d`"), holds
   an address only while nothing can change what it points into, and looks the place up again after
   something may.
3. Every function has an **effect summary**, worked out from its body and kept in its package's
   interface, saying which places it may **reshape** (change in a way that frees or moves memory); no
   pointer is ever kept across a call that may reshape what it points into. That one rule replaces
   Rust's "one writer or many readers", its lifetimes, `RefCell`, and today's run-time access words.
4. A link that must not own is written as Python writes it: `parent: Option[Node] = backlink(children)`
   (the language keeps both ends in step; reading it is a plain load) or `target: weak GameObject`
   (reads as `None` once the target is gone; one bit test per read).
5. Closures capture variables as Python's do (the compiler picks copy, move, pointer or a counted box);
   a stored closure protects its own state only around the rare call that may remove it.
6. Threads are checked against each other from the same effect summaries: no `Send`, `Sync` or `Arc` is
   written; a lock guards what is only touched while holding it; `+=` from tasks becomes a per-task total.

### 0.2 The kinds the programmer sees, and what runs

- **Values**: `struct` and `class`. **Links the programmer writes**: none for names and handles (they are
  what `=` makes); `backlink(...)` and `weak` for links that must not own. Everything else a Rust
  programmer writes (`&`, `&mut`, lifetimes, `Box`, `Rc`, `RefCell`, `Cell`, `Weak`, `Send`, `Sync`,
  `Arc`, `move`) is gone or decided by the compiler. Views (`str`, `Span`) stay as types, with their
  limits inferred.
- **Pointers in the compiled program**: four kinds. A name or loan (an address or a position the
  compiler holds only while it is valid), a counted handle, a back link (a plain pointer the language
  keeps in step) and a weak pointer (a plain pointer whose target's header is tested on read).
- **What runs while the program runs**: a count change when a class handle is stored where the compiler
  cannot see through; a bounds check on every index the compiler cannot prove; a compare and a subtract
  when a list loses an element while a name into it is in use; the search-and-move C does when a
  back-linked node changes parent; a weak count change on stores into `weak` fields and a one-bit test
  on reads; one counted box per stored closure; locks, per-task totals and atomic counts only where tasks
  or threads really share. Section 3.3 lists each with its measured cost.

### 0.3 The four workloads

Checksums and memory: my one run of each program this session (`combined/run_once_output.txt`). Time:
the earlier provisional measurement of the same mechanism, with its source; this model's own times are
for the final measurer (section 7).

| Workload | C model | Checksum (known) | Peak memory, model / plain C in the same layout | Earlier time of the same mechanism, ratio to plain C in the same layout (PROVISIONAL) | Safe Rust, same work (PROVISIONAL) |
|---|---|---|---|---|---|
| W1, ECS in EnTT's owning group | `combined/w1.c` | 2111831520 | 39,156 / 39,156 KB = **1.00** | the salvage's index-cursor model in this layout: 1.057 with two bounds checks per entity, 1.008 without (salvage `runs/SUMMARY.txt`, `d3_w1_group`, `d3_w1_group_nobounds`); this model proves both away, and its inner loop is plain C's (section 6.1) | 1.166 (salvage `rust_w1_group`) |
| W2, tree with parent pointers | `combined/w2.c` | 647034814 | 76,724 / 76,684 KB = **1.00** | back links with this model's 4-byte header: 0.964 and 1.024 (first graphs attack, `graphs-callbacks-attack/bench/runs/w2_attack3_summary.txt`, `w2_attack4_summary.txt`); with no header 1.025 and 1.149, with an 8-byte header 1.202 and 1.309 (second graphs attack, `graphs-callbacks-attack2/bench/runs/w2_a.txt`, `w2_b.txt`) | 1.519 and 1.357, index arena (second graphs attack, same files) |
| W3, game objects referring to each other | `combined/w3.c` | 9288233970 | 9,468 / 9,752 KB = **0.97** | a `weak` read by a test of the target's count, objects from a slab, dispatch by class id: 0.822 (salvage `d1_w3`, with the weak count in a 12-byte header; this model's side-record version is UNMEASURED) | 1.283 (salvage `rust_w3`) |
| W4, callbacks that remove themselves | `combined/w4.c` | 34946993000 | 4,208 / 4,076 KB = **1.03** | a stored closure as an object: 1.045 and 1.037 of plain C doing exactly the same work with the same slab (second graphs attack, `runs/w4_a.txt`, `w4_b.txt`) | 1.490 and 1.472 with the workload's shape (same files); 0.712 with a different shape, the handler returning "remove me" (salvage `rust_w4`) |

### 0.4 What changed from the first run of this step

1. **Loops that remove from their own list and call code the compiler cannot see** (section 4.5). The
   first run refused them at compile time, saying the run-time mode that would run them costs 1.20x to
   1.29x of plain C per such call. The containers attack's second pass showed that number was measured
   against C that knows its callback only reads the length; against C written to be correct for any
   callback, the mode costs 1.00x and 1.02x (`containers-attack2/runs/a4_cb_summary.txt`,
   `a4_cb_2_summary.txt`). That reason is withdrawn. The choice stays the compile-time rule, for the
   reasons in 4.5, and it is listed as an annoyance with no answer meeting all three goals.
2. **The class's speed is open** (section 4.9). With objects from a slab, as this model allocates them,
   the shared-object workload runs at 1.16x and 1.17x of plain C (three batches,
   `ownership-attack2/bench/runs/share_summary.txt`, `share2_summary.txt`, `alloc2_summary.txt`); the first
   run quoted 1.05x to 1.10x. The cost is the count increment on the new target when a handle is stored;
   the one fix tried did not help.
3. **The compare that keeps a name's old value** costs 1.00x of correct C, not "0.88x" (the first
   attack's baseline computed a different answer; `ownership-attack2/bench/runs/alias_name_summary.txt`).
4. **W4** is 1.045 and 1.037 of plain C doing exactly the same work with the same slab, not 1.002 and
   1.032 (the earlier baseline did slightly less work; second graphs attack section 2).
5. **`weak`**: the test on read is kept, with a stronger reason. The registration mechanism needed three
   successive fixes for lists of weak elements, and on churn it costs 1.09x to 1.27x of plain C where
   Rust's `Weak`, which is a test on read, costs 0.92x and 0.94x (second graphs attack 4.2,
   `runs/weak_g.txt`, `weak_h.txt`). `w3_registration.c` now finds its records through the object (its
   fastest form), so the measurer compares the two mechanisms at their best. Side-record counts are 64 bits
   (ownership attack 5.5).
6. **New memory-safety holes closed** by the second attacks, all at compile time or on rare paths: a map
   value named by address, a generator's loan kept while its list moves (ownership); a write through an
   index while views of the element live, a handle's drop reading a view whose text is gone (references);
   a slice passed with the list a callee grows, a slice kept across `clear()` through another name, a list
   inside a list walked across unseen code, a word kept across `clear()` of a list that took its text
   along (containers); a back link assigned the value it holds, a stored closure freed while its own
   arguments run, a handler lending its state to a call that frees it, narrowing of a local a closure
   writes, a variable captured by pointer cleared during a loop (graphs); guards, scopes and raw pointers
   reaching other threads, a parent's lock "protecting" its tasks, a function value losing "needs the
   lock", a claim dropped with a plain store, a generator whose consumer yields, a C callback growing a
   list the caller walks, a misread user-data parameter that C writes through (threads).
7. **The ECS cursor rule** moved into the group's one "leave" operation, which `destroy` and
   `remove[T]` both call (containers attack 7.5); `combined/w1.c` and its self-check now include it.
8. **A walk that calls a stored callback on every step** pays a count pair, 1.27x to 1.32x of the same
   walk without it, unless the callback's effects are known (second graphs attack 3.5); not covered by the
   first run.

### 0.5 Where no answer meets all three goals

Said plainly; each has its reason in sections 4 and 5.

1. **A loop that removes from the list it walks and also calls code the compiler cannot see that may
   change that list** is a compile-time error, Rust's rule. The run-time mode that runs it is on par with C
   in the loop, but needs a walking state that other code tests; its cost on whole programs is unmeasured
   and it panics where Python works (section 4.5).
2. **A struct element's name used after such a call** is a compile-time error (copy it first, or make the
   element a class); Python keeps the object.
3. **One value in two owning places** needs `copy(x)` or a `class`; Python shares one object.
4. **A cycle of owning links leaks**, as in Rust and Swift; Python's collector frees it. A child of a dead
   parent sees `None`; a weakly referenced object that dies frees its fields at once but keeps its block
   until the last weak field to it lets go, as Rust's does.
5. **Shared class objects re-pointed often**: 1.16x to 1.17x of plain C with a slab (Rust's `Rc` 1.31x to
   1.42x). Above 15%, so not on par with C; no fix found. The C-speed spelling is structs in an `Array`
   with positions (section 4.9).
6. **A walk that calls a stored callback with unknown effects** pays a count pair per step (1.27x to
   1.32x), as Rust's `Rc` tree does; Rust's index arena pays nothing because it forbids the callback from
   changing the tree.
7. **Threads**: a class object written by several tasks needs a lock, or a claim at 1.12x to 1.36x of
   plain C (1.9x to 4.0x when the objects are also held elsewhere); a bare loop that only re-targets parent
   objects from tasks is 2.0x to 3.5x of plain C (Rust's `Arc` 3.2x to 24x).
8. **Generics**: "the most specific implementation wins" is unproven and stays open; Python's duck-typed
   method call and its `acc = 0` sum have proposed rules the owner must accept or refuse.

---

## 1. Words used

- **Place**: somewhere a value lives: a variable, a field (`p.hp`), a list element (`xs[3]`), a map entry
  (`d[k]`), a field reached through a handle (`cam.target.hp`).
- **Name**: what `x = place` makes in this model: a second name for the value in that place, not a copy.
  The compiler keeps it as a **path** (how to find the place again).
- **Position**: a name for a list element kept as its index ("element 3 of `xs`"), moved when elements
  before it are removed.
- **Handle**: what a variable of a `class` type holds: a pointer to an object on the heap. **Count**: how
  many owning handles an object has; it is freed when the count reaches zero (reference counting, as
  Python, Swift and Rust's `Rc` do). **Count pair**: one increment and one decrement around something.
- **Header**: the bytes at the start of an object that hold its count and, when needed, its type.
  **Count word**: the 32-bit word in the header that holds the count and its flag bits.
- **Reshape**: a change that can free or move memory: replacing a value that owns memory
  (`self.items = []`), `push`, `insert`, `remove`, `clear` or `sort` on a list (a push may move every
  element to a bigger block), changing an enum's variant, dropping the last handle to an object. Writing a
  number (`p.hp -= 1`) is not a reshape.
- **Effect summary**: what the compiler records about a function from its body: which places it reads,
  writes and reshapes, by field path (`self.log`) and, for objects reached through handles, by class and
  field ("reshapes `Player.inventory` of some `Player`").
- **Loan**: a pointer the compiled program holds into a place for a while (a loop's pointer into the list
  it walks).
- **View**: an address and a length into memory something else owns (`str`, a slice `xs[a..b]`). Its
  **limit** is how long what it points into lives; the compiler works it out.
- **Bounds check**: the test `i < len` before `xs[i]`. **Panic**: a stop with a message; Ember's panics
  never unwind (`[PAN-1]`), so no half-finished change is ever seen after one.
- **Slab**: an allocator that hands out same-size blocks from a large chunk and reuses freed ones from a
  list; Python's own object allocator works this way.
- **Side record**: a small record kept outside an object, found through the object's count word, for the
  rare objects that need more than the header (here: objects a `weak` field points at, or with a count too
  large for the header).
- **Back link**: a field the language keeps equal to "the object whose list holds me". **Weak field**: a
  field that does not keep its object alive and reads `None` once the object is gone.
- **Box** (for closures): a small counted heap block holding a closure's captured state.
- **Code the compiler cannot see**: a call through a function value whose closures are not all known, a
  virtual method whose overrides are not all known, a foreign (C) function, a `yield` (the consumer runs
  there). Its summary is what its type declares, or "may do anything".
- **Thread-bound value**: a value that must stay on the thread that made it (a lock guard, a scope, a raw
  pointer).
- **Same layout**: plain C doing the same work with the same data layout, so a ratio measures the memory
  model, not a different data structure. **Plain C**: C written for speed with no safety checks. **Safe
  Rust**: Rust with no `unsafe`.
- The job's three kinds of answer: **a Pythonic spelling** (Rust's compile-time guarantee written the way
  Python writes it), **a smarter compile-time rule** (one that makes Rust's rule unnecessary), and **a new
  mechanism**.
- Last night's three designs, by what they did: **"copy instead of share"** (lists and dicts as
  copy-on-write values), **"checked pointers"** (objects in slots that never move, references carrying a
  generation number), **"today's rules, but never an error"** (today's borrow checker with run-time
  fallbacks).
- **PROVISIONAL**: measured while other agents were running; a decision resting on a difference under 15%
  waits for the final measurer. **UNVERIFIED**: believed, not checked.

---

## 2. How this was put together

Each group's final answer list (the last section of each final attack report) is the input; the salvage
report's good parts are candidates. Where two answers cover the same ground, I kept the one that meets
more of the three goals, and when both meet all three, the one that adds fewer kinds of things to the
language. Where two answers contradict each other, section 4 says which went and why. Section 5 lists
every annoyance with the answer this model keeps.

### 2.1 The agents that ran under the error-box problem

The problem: a program started through `cmd /c start ... /affinity` could open a "Windows cannot find"
box on the owner's screen, and a start that fails runs nothing (`phase3b-ownership-attack.md`, "Why there
is a second attack"). What a failed start would do to a report is leave a timing missing, not make one up.
So the question for every number I use is: did the program that produced it actually run?

Which inputs started programs that way, read from their runners: the salvage (`salvage/runs/ab.ps1` line
12), the graphs group and both its attacks (`graphs-callbacks/bench/run.ps1`; the second attack copied it,
`graphs-callbacks-attack2/bench/run.ps1` line 15), the ownership group (`ownership/bench/run.ps1`), and both
threads attacks (`threads/attack/bench/run_group.sh`, `threads/attack2/bench/run_group.sh`). The containers
attack and the references attack started theirs with PowerShell's `Start-Process -NoNewWindow`; the second
ownership attack started its programs directly from PowerShell (its section 7).

What I checked: every line with a time (`ms=`) in the run folders this report cites also carries the
program's checksum, which a program prints only after it has done all its work.

| Run folder (under `target/redesign2/`) | lines with a time | of them with a checksum |
|---|---|---|
| `salvage/runs/` | 748 | 748 |
| `graphs-callbacks/bench/runs/` | 290 | 290 |
| `graphs-callbacks-attack/bench/runs/` (the first graphs attack; the 4-byte header's numbers) | 220 | 220 |
| `graphs-callbacks-attack2/bench/runs/` | 670 | 670 |
| `ownership/bench/runs/` | 72 | 72 |
| `containers/bench/runs/` | 340 | 340 |
| `ownership-attack2/bench/runs/` | 271 | 271 |
| `containers-attack/runs/` | 205 | 205 |
| `containers-attack2/runs/` | 50 | 50 |
| `threads/attack/bench/runs/` (the first threads attack; the reductions' 1.00x) | 378 | 378 |
| `threads/attack2/bench/runs/` | 312 | 312 |
| `references/attack/bench/out/` (the first references attack; A7's 1.046, A23's 1.015) | 139 | 139 |
| `references/attack2/bench/out/` | 242 | 211; the other 31 are the median summaries, each listing the checksum its runs printed (`checksums=1 16779647745309`) |

Command: for each folder, `cat <folder>/*.txt | grep -c "ms="` and the same piped through
`grep -c "checksum="`. A search of every `.txt` file under `target/redesign2/` for "cannot find",
"cannot execute" and "The system cannot" found them only in `ember check` outputs (compiler messages about
Ember names), never in a run file. The four salvage files also each end with a `DONE` line and have one
checksum line per program per round (91, 91, 77 and 91 lines).

So every timing cited here came from a program that ran to its end. Whether a box appeared on the owner's
screen while those batches ran cannot be read from the files; the runners that use `cmd /c start` pass
`/b /wait`, which opens no console window of its own. **The first run of this step** read the first-run
attacks; its own runs used PowerShell's `Start-Process` (its section 2). Its conclusions are replaced here
because its inputs changed (section 0.4), not because its runs failed. **My runs** use PowerShell's
`Start-Process -NoNewWindow` (`combined/run_once.ps1`, `python/run_twins.ps1`), and every C model calls
`SetErrorMode` first through the shared harness.

### 2.2 Checks I made (none of them a timing)

| Check | Command | Result |
|---|---|---|
| each C model gives the known checksum | `"C:/Program Files/LLVM/bin/clang.exe" -O2 -Wall -Wextra wN.c -o wN.exe` (clang 22.1.8), then one run each through `run_once.ps1` | all four equal the known checksums; `w3_registration.exe` too; no warnings |
| the ECS cursor rule keeps "every entity once" in the owning group, with components taken away and added inside the loop | `clang -O2 -DCURSOR_CHECK w1.c -o w1_cursor_check.exe`, run | `cursor-check ok` (5,000 random loops that destroy the current entity, entities not yet reached and entities already visited, take `Velocity` from entities that live on, give it to entities that had only `Position`, and create entities with both) |
| the self-check catches a library without the rule | the same with `-DNO_CURSOR_RULE` | `panic: check: visited twice, or an entity that joined during the loop was visited`, exit 101 |
| the new `w1.c` emits the same main loop as the first run's | `clang -O2 -S` of both, `diff` of the `main` function | no difference |
| the W1 inner loop is plain C's | `clang -O2 -S` of `combined/w1.c` and `salvage/plain/w1_group.c` | the same entity load, two float load-add-store pairs, the `% 100` multiply-shift test and its branch; plain C also reloads its three array pointers from its globals on every step, the model keeps them in registers |
| every back link names its holder, every node is held once (W2) | `clang -O2 -DLINK_CHECK w2.c -o w2_link_check.exe`, run | `link check ok`, known checksum |
| no weak field ever points at a freed block (W3) | `clang -O2 -DPOISON w3.c -o w3_poison.exe` (freed blocks never reused, marked, every weak read checks the mark), run | `poison check ok: 20 blocks freed, none read after free; side records in use at the end 53499, at most 53499`, known checksum |
| how many objects died in W3 | a scratch copy of the poison build that prints `destroyed` | 9,173 |
| no closure box is used after it is freed (W4) | `clang -O2 -DPOISON w4.c -o w4_poison.exe`, run | `poison check ok: no box used after it was freed`, known checksum |
| the Python twins do the same work | `python/run_twins.ps1`: `C:\Python314\python.exe -I <file>` (Python 3.14.2), once each, efficiency cores | `w1_dicts.py` and `w1.py` 2111831520, `w2.py` 647034814, `w3.py` 9288233970, `w4.py` 34946993000 (`python/run_twins_log.txt`; the files are the first run's, kept unchanged) |
| what today's compiler says about the four programs in this model's syntax | `target/ci/ember.exe check <file>` (ember 0.1.0, language 0.9.10) | refused, as expected for a proposal; the errors that matter are quoted in section 6 (`ember/ember_check_output.txt`) |
| peak memory of models and baselines | `run_once.ps1`, one run each | the table in 0.3; also `w3_registration` 11,960 KB (the first run's hash-table version 14,272 KB in the same batch) |

One observation on the memory figures: one run of the same program can differ by a few hundred KB (the
first run's `w2.exe` printed 77,056 KB in my batch, the new one 76,724 KB, its link-check build 76,684
KB). Differences of that size are noise.

---

## 3. The model

### 3.1 What a programmer writes

Each block is the Ember program under this model beside the Python a programmer would write. Where a
Python result is quoted, its source is named.

**Values and names.** A list is a value with one owner; a second name is not a copy.

```python
matrix = []; row = []
matrix.append(row); row.append(1)
print(matrix)                      # [[1]]

a = [1]; b = a; a = [9]
print(b)                           # [1]
```

```ember
matrix = []; row = []
matrix.push(row); row.push(1)      # row now names matrix[0]
println(matrix)                    # [[1]]

a = [1]; b = a; a = [9]            # the old list moves into b's storage at `a = [9]`
println(b)                         # [1]
```

The compiler emits `matrix[0].push(1)` for the second line and a move instead of a free for `a = [9]`
(ownership group 2.2; its Python results in `ownership/python/py_checks_output.txt`). Nothing is counted.
What is left: one value stored into two owning places (`a.push(m); b.push(m)`) is a compile-time error
whose fix-its are `b.push(copy(m))` or making the type a `class`.

**Classes** are shared objects, as in Python:

```python
class Player:
    def __init__(self): self.hp = 10; self.inventory = []
p = Player(); cam_target = p; hud_target = p
cam_target.hp -= 1
print(hud_target.hp)               # 9
```

```ember
class Player:
    hp: int = 10
    inventory: Array[String] = []

p = Player(); cam_target = p; hud_target = p
cam_target.hp -= 1
println(hud_target.hp)             # 9
```

No `Rc`, `RefCell` or `Shared`. The count changes when a handle is stored somewhere the compiler cannot
see through; a handle read into a local is not counted when nothing in the local's life may drop an object
of that class.

**Methods that use part of the object.** No method "borrows all of `self`":

```python
def note(self, msg): self.log.append(f"[{self.frame}] {msg}")
def tick(self):
    for p in self.players:
        p.hp -= 1
        if p.hp <= 0: self.note(f"{p.name} died")
```

```ember
fn note(self, msg: String):        # summary: reshapes self.log; reads self.frame
    self.log.push(f"[{self.frame}] {msg}")
fn tick(self):
    for p in self.players:
        p.hp -= 1
        if p.hp <= 0:
            self.note(f"{p.name} died")   # accepted: note does not reshape self.players
```

`mut` is inferred. Today this is `E3020` across packages (ownership group 3.1).

**Holding an element while the list grows or shrinks**, and the swap idiom (containers group A2, saved by
its attack, 3.3):

```python
boss = enemies[0]
enemies.append(Enemy(boss.x + 1.0, boss.hp // 2))
boss.hp //= 2

tmp = xs[i]; xs[i] = xs[j]; xs[j] = tmp
```

```ember
boss = enemies[0]                  # the name "element 0 of enemies"
enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
boss.hp //= 2                      # the buffer is read again after the push

tmp = xs[i]; xs[i] = xs[j]; xs[j] = tmp     # replacing xs[i] moves its old value into tmp first
```

**A name for a dict value** (ownership attack 4.2):

```python
d = {}; c1 = Cell(); d["a"] = c1
for k in range(100): d[str(k)] = Cell()
c1.x = 5
print(d["a"].x)                    # 5
```

```ember
d = Map[String, Cell]()
c1 = Cell(x=0); d["a"] = c1        # c1 now names the entry for "a"
for k in range(100): d[str(k)] = Cell(x=0)
c1.x = 5                           # the entry's position, found again: never an old address
println(d["a"].x)                  # 5
```

**Changing a list while looping over it** (containers group A4): the loop becomes a one-pass in-place
filter. Python skips the element after a removed one; Ember does not copy that skip.

```python
for e in enemies:
    e.hp -= 1
    if e.hp <= 0:
        if e.splits: enemies.append(Enemy(5))
        enemies.remove(e)          # Python then skips the next enemy
```

```ember
for e in enemies:
    e.hp -= 1
    if e.hp <= 0:
        if e.splits: enemies.push(Enemy(hp=5, splits=false))
        enemies.remove(e)          # the current element goes; nothing is skipped
```

The Python habit of walking a copy works too, and makes no copy (containers attack 5.7):

```python
for e in es[:]:
    e.hp -= 1
    if e.hp <= 0:
        es.remove(e)
        es.append(E(9))            # leaves [E2, E4, E9, E9]
```

```ember
for e in enemies[..]:              # the same walk, bounded by the elements there at the start
    e.hp -= 1
    if e.hp <= 0:
        enemies.remove(e)
        enemies.push(Enemy(hp=9))  # leaves [E2, E4, E9, E9]
```

**Views** (references group A5, A6, A22; containers group A25). Nothing about lifetimes is written:

```python
words = []
for line in f:
    words.extend(line.lower().split())
```

```ember
words = []
for line in f:
    words.extend(line.to_lower().split())   # views; the list takes each line's text along
```

**Parent pointers, other links that must not own, and graphs** (graphs group A8):

```python
class Node:
    def __init__(self, value): self.value = value; self.parent = None; self.children = []
def reparent(node, new_parent):
    node.parent.children.remove(node)
    node.parent = new_parent
    new_parent.children.append(node)

import weakref
class Camera:
    def __init__(self, target): self._target = weakref.ref(target)
    @property
    def target(self): return self._target()     # None once the player is gone
```

```ember
class Node:
    value: int
    parent: Option[Node] = backlink(children)    # kept equal to the node whose children hold me
    children: Array[Node] = []

fn reparent(node: Node, new_parent: Node):
    new_parent.children.push(node)                # takes it out of its old parent first, sets parent

class Camera:
    target: weak Player                           # reads as Option[Player]; None once it is gone
```

A graph whose edges are kept in step both ways (second graphs attack 4.4):

```python
def road_push(a, b): a.roads.append(b); b.into.append(a)
def city_die(x):
    for y in x.into:
        if y is not x: y.roads.remove(x)
    for z in x.roads:
        if z is not x: z.into.remove(x)
```

```ember
class City:
    id: int
    roads: Array[weak City] = []
    into: Array[weak City] = backlink(roads)      # read-only; kept in step with every roads list

cities[0].roads.push(cities[1])                   # also cities[1].into.push(cities[0])
cities[k] = City(id=next_id)                      # the old city leaves every list that names it
```

**Closures** (graphs group A12): captures work as Python's do, without `nonlocal`, `owned` or a cell
type; a handler may remove itself from inside its own call (section 6.4 is the full program).

```python
def make_counter():
    count = 0
    def inc():
        nonlocal count
        count += 1
        return count
    return inc
```

```ember
fn make_counter() -> fn() -> int:
    count = 0
    fn inc() -> int:
        count += 1                 # count lives in a box shared by the function and the closure
        return count
    return inc
```

**Threads** (threads group A15, A19, as saved by its second attack): no `Send`, `Sync` or `Arc`; tasks in a
scope are checked against each other; `+=` from tasks is a per-task total joined at the end; a lock guards
what is only touched while holding it.

```python
import threading
def count_words(texts):
    total = 0; lock = threading.Lock()
    def work(t):
        nonlocal total
        with lock:
            total += len(t.split())
    ts = [threading.Thread(target=work, args=(t,)) for t in texts]
    for th in ts: th.start()
    for th in ts: th.join()
    return total

RESULTS_LOCK = threading.Lock(); RESULTS = []
def worker(x):
    r = x * x
    with RESULTS_LOCK: RESULTS.append(r)
```

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())   # a reduction: no lock
    return total

RESULTS_LOCK = Lock()
RESULTS: Array[int] = []
fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)            # an unguarded push here is a compile-time error naming the lock
```

**Generators** (threads attack 6.1, ownership attack 3.8): a loop whose body yields walks by position and
reads its list again after each resume, so the caller may grow the list between `next()` calls, as in
Python.

```python
def walk(xs):
    for x in xs:
        yield x
items = [1, 2]; g = walk(items); print(next(g))
items.append(99)
print(list(g))                     # [2, 99]
```

```ember
gen fn walk(xs: Array[int]) -> Generator[int]:
    for x in xs:
        yield x

items = [1, 2]; g = walk(items); println(g.next())
items.push(99)                     # allowed: the walk keeps a position, not an address
println(g.collect())               # [2, 99]
```

### 3.2 What the compiler checks

All of this is decided when the program is compiled. Each rule names the final answer it is; section 4
says where two answers met.

1. **Effect summaries** (ownership attack, A1). For every function, from its body and its callees'
   summaries: which places it reads, writes and reshapes, by field path for its parameters and by the class
   that declares the field for objects reached through handles. They go into the package interface for
   every function, `pub` ones included. A call through a function value, a virtual method whose overrides
   are not all visible, or a foreign function takes the effects its type declares, or every effect. A body
   with `unsafe` or a foreign call uses its declared effects, and **every** effect, module variables
   included, when it declares none. **Every foreign call's summary includes the summaries of the callbacks
   C may run on that thread before it returns** (threads attack 5.1). A hot reload that widens a summary a
   running frame relies on is refused. A `drop` may change only what its object owns unless its class says
   more. Assigning a field stores the new value before the old one is dropped, and `clear()` of a list whose
   elements run program code in their `drop` detaches the buffer first and gives it back after.
2. **Loans.** A loan is broken only by a reshape of the place it points into, or of a place containing it.
   A loan used after something that may reshape its place is a compile-time error naming the line. For
   objects, "may" means "any object of that class", because two handles can name one object. A loan stored
   inside a value (a generator frame, a struct holding a view) pins its place: the place may not be moved,
   replaced or dropped while that loan lives.
3. **Two arguments that may be one place** (containers attack A3; ownership attack 3.5): functions are
   compiled assuming no overlap; a call that cannot rule out that two arguments are one place compares them
   and calls a second compiled copy that assumes nothing. In that copy a view of a string's characters used
   after the string is replaced keeps the old buffer alive until its last use, and a view of a list's
   elements used after a removal, replacement or move of them is copied just before that write. A view
   passed to a call whose summary may change the view's source is passed as a copy (Python's slice is a
   copy); for a callee that changes one known list, a range compare and a copy only on overlap.
4. **Names** (ownership attack A11; containers attack A2). `x = place` names the place; handles and keys on
   the path are taken when the name is made, so rebinding a variable on the path does not move the name. A
   list-element name is a position: the buffer is read again after a push; a removal before it moves it
   (`j -= (k < j)`); removing or replacing the named element moves its value into the name; a second name
   for the same element follows the first (a one-bit flag). A map-value name is the entry's position, found
   again by key after anything that may compact the map. A name passed to a function that may change its
   list passes a place (list and position), and the function's removals move it after comparing the list
   by identity. A position across a sort, swap or reversal of its list is a compile-time error. A struct
   element's name used after code the compiler cannot see that may remove from its list is a compile-time
   error (copy it first, or make it a class). A struct replaced inside a call while a name to it is used is
   a compile-time error that offers both meanings ("`copy()` keeps it as it was before the call; or name it
   after the call"). `@derive(Copy)` structs are named like any other value.
5. **One owner per value** (ownership attack A11). A value is in one owning place at a time; storing it into
   a second is an error with `copy()` and `class` as fix-its. A write that may replace a named place
   through another handle keeps the old value for the name, with one compare, in a register.
6. **View limits are inferred** (references attack, all twelve final answers): what a function's result
   borrows is read from its body; containers and objects holding views carry their limits, one per object,
   kept through type erasure and `weak` handles; dropping such a handle counts as a use of the limit when
   the object's `drop` reads the view; a value may hold views of its own heap contents, and a write through
   an index holds the container until the views are rebuilt; a temporary lives as long as its views; the
   owner follows a view into a local; a returned view reuses the temporary's buffer only when it fills at
   least half of its **capacity**; a field held exclusively (a `mut` parameter, `mut self`, a local
   `ref mut` such as a loop element) may be emptied if every path refills it; assigning to a whole binding
   rebinds it, as in Python. A list of views whose text would die takes the text along in an owner block,
   if the list is never shrunk while it holds taken texts (containers attack A25).
7. **Loops** (containers attack A4, with section 4.5 here). A loop whose body may change the list it walks
   is an in-place filter with a read position `r` and a write position `w`; appended elements are visited;
   `enemies.remove(e)` drops the current element without skipping the next; `enumerate`'s index is `e`'s
   current position; reads and writes through the list by position compare the position with `r`;
   inserting, sorting or reversing the walked list in the body is a compile-time error. A loop over a copy
   or a slice of the list it changes is the same walk with the copy's bounds. A loop whose body calls code
   the compiler cannot see that may change the list walks by position and reads the list's length and
   buffer again after each such call (Python's own iterator behaviour); if the body also removes from the
   list, the loop is a compile-time error with fix-its. A loop whose body yields (or awaits) walks by
   position too, reading the collection's header again from where it lives after each resume.
8. **Back links** (second graphs attack, A8): a put into the paired list is one membership change: unlink
   the replaced value, link the new one (taking it out of its old holder, its count moving with it), release
   the replaced value. An indexed put of a node already in the list panics; the swap line compiles to
   `swap()`; the paired list cannot be lent as a mutable list or view; a back link cannot be assigned except
   where the same block makes it true; the hidden writes are in effect summaries.
9. **`weak`** (section 4.3): a weak field or list element is a plain pointer holding a weak count on its
   target; a read tests the target's DEAD bit. A class that a `weak` field points at is not shared with an
   unscoped thread (`SyncWeak[T]`, at Rust's price, for programs that need it). Edges kept both ways
   (`backlink(roads)` on an `Array[weak City]`) take a dying city out of every list that names it.
10. **Closures** (second graphs attack, A12 and A32): per captured variable, the cheapest place that keeps
    Python's meaning (copy only for `Copy` values nothing writes later, move, pointer when every closure
    provably dies first, else a counted box). Any variable a closure writes, boxed or not, is a place in
    effect summaries. A stored closure keeps its state in a counted box when its holder may be overwritten
    during its call. A call of a stored closure copies two words out; when evaluating the arguments may
    remove that closure, the callee's box gets a count pair around them. The body keeps its box across a
    call that may release it when it uses the box during or after that call; known field closures rule it
    out. A closure that captures a box is not sent to an unscoped thread.
11. **Narrowing** (second graphs attack, A13): after `if isinstance(s, Circle):` or `if m is not None:`,
    `s` is a `Circle` (and `m` not `None`) in the block, following early exits and `assert`, until any
    write of that place in any object (hidden writes included) or any call that may run a closure writing a
    narrowed local. The type index names the generic instance.
12. **Threads** (second threads attack, all sixteen final answers): the scope rule (tasks against each other
    and against the parent, from summaries); thread-bound values never reach another thread; a held lock
    protects only the thread that holds it; tasks may retain and release the parent's objects with counts
    netted per task; reductions only in a task's own body, element reductions for `m[k] op= e`; one lockset
    rule for every lock, "needs the lock" carried in function types, static re-entry, waits that end views;
    the shared flag only for classes that can reach another thread; claims taken and dropped atomically;
    module variables with Python's spelling, synchronised ones usable from any thread; generator and async
    loops by position; checked trampolines for C callbacks, with a tagged user-data value.
13. **Generics** (second graphs attack, A16): bounds are inferred from the body and recorded in the
    interface; a bound on an interface other packages cannot see is refused; a program nobody depends on may
    implement any interface for any type.

### 3.3 What runs while the program runs, and what it costs

Every cost is a ratio to plain C doing the same work in the same layout, PROVISIONAL, from the file named.

| Part | When it runs | What it does | Measured cost |
|---|---|---|---|
| counts on class handles | a handle stored where the compiler cannot see through, or overwritten; not when read into a local whose life cannot drop that class | a plain increment or decrement with a range test (a count past 28 bits moves to the side record, 3.4); atomic only for objects with the shared flag | sharing workload: 1.09x and 1.11x with one `malloc` per object, **1.16x and 1.17x with a slab** (open, 4.9); Rust's `Rc` 1.31x to 1.42x; today's emitted code 1.71x and 1.78x (`ownership-attack2/bench/runs/share_summary.txt`, `share2_summary.txt`, `alloc2_summary.txt`) |
| bounds checks | indexes the compiler cannot prove in range | `i < len`, panic otherwise | what safe Rust pays; in W1 the per-entity checks are proven away (with them 1.057, without 1.008: salvage `d3_w1_group`, `d3_w1_group_nobounds`) |
| element names | a removal, or a write that may replace the named place through another handle | a compare and a subtract, or a compare and a move | the compare: 1.00x of correct C in its register form; Rust's copy 1.28x; the pointer form 1.53x, never emitted where the register form works (`ownership-attack2/bench/runs/alias_name_summary.txt`) |
| two arguments that may be one place | a call that cannot rule it out | one compare, then the no-overlap or the overlap copy | element pairs free; spans 1.08x of `restrict` C with bounds checks, safe Rust 1.13x (`containers-attack/runs/a3_span.txt`); two list parameters 0.98x (`a3_lists.txt`) |
| the one-pass loop filter | a loop that changes the list it walks | read and write positions; removed elements dropped at the end of their step | 0.97x of plain C (`containers-attack2/runs/a4_base.txt`); 1.01x with reads by position (`containers-attack/runs/a4_read.txt`), 0.98x with writes by position (`containers-attack2/runs/a4_write.txt`); Rust's idiom 1.62x (`containers/bench/runs/a4.txt`) |
| a loop across unseen calls | a loop whose body calls code the compiler cannot see that may change its list (and removes nothing itself) | reads the list's length and buffer again after each such call | the reload any C compiler makes after a call that may write memory; not timed |
| lists of views take their text along | a list of views outlives its text | the text's buffer moves into the list's owner block (an 8-byte pointer in the list) | 1.00x; Rust's workaround 1.13x; owned words 1.77x (`containers-attack/runs/a25_lines.txt`) |
| back links | a back-linked node put into or taken out of its list; the holder dies | search, order-keeping remove, push, one pointer store (what C writes) | the link's own work 1.03x and 1.15x (no header); 0.964 and 1.024 with the 4-byte header (sources in 0.3); Python's filter idiom on a back-linked list 1.13x to 1.42x (`graphs-callbacks-attack2/bench/runs/filter_e.txt`, `filter_f.txt`, 1 to 3 ms programs) |
| edges kept both ways | a road added or removed; a city dies | the reverse list kept in step | 0.969 and 1.081 of plain C keeping the same reverse lists; memory 1.044 (`graphs-callbacks-attack2/bench/runs/graph_a.txt`, `graph_b.txt`) |
| `weak` fields | a store; a read; the death of an object a weak field points at | store: weak count +1 / -1 in the side record; read: one bit test; death: drop the fields, keep the block until the weak count is 0 | the read test with the weak count in a 12-byte header: W3 0.822 (salvage `d1_w3`); the test alone on a chain of dependent loads +18% (salvage `d1_w2` against `d1_w2_proven`), which is why trees use back links; this model's side-record version: UNMEASURED |
| closure boxes | a closure stored in a list or field; a variable some escaping closure writes | one counted box from the slab for its size; the body keeps its box across a call that may release it | W4 1.045 and 1.037; Rust 1.490 and 1.472 (`graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`); a boxed variable in a hot loop 0.990 and 0.998 of a C local (`micro_a.txt`, `micro_b.txt`) |
| a count pair around an unseen call | a handle used after a call whose effects may drop its class (a walk calling a stored callback) | retain before, release after | 1.27x to 1.32x of the same walk without it (`graphs-callbacks-attack2/bench/runs/w2v_a.txt`, `w2v_b.txt`); none when the callback is a parameter or its closures are known |
| class objects from slabs | every class allocation | a block from a per-size chunk, no allocator header | slabs gave W3 -19% and W4 -17% (salvage 2.5) |
| the pool table | `reg.get[T](e)` | one slot per type, the slot number from the compiler, read once per function when given at load time | 1.05x to 1.11x of plain C (bounds checks); safe Rust's `downcast_mut` 1.57x to 1.64x (`references/attack2/bench/out/summary_run1.txt` to `summary_run3.txt`) |
| threads | only where tasks or threads really share | per-task totals; private copies for element reductions; netted count changes for parent objects; locks where written; atomic counts for flagged objects; claims in parallel loops over handles with count above 1 | reductions 1.00x (first threads attack); element reductions 3.5x faster than one thread (`threads/attack2/bench/runs/hist_summary.txt`); netted counts 1.05x on Python's re-target loop, Rust's `Arc` 1.80x (`retarget_summary.txt`); claims 1.12x to 1.36x, 1.9x to 4.0x when the objects are held elsewhere (`claim3*_summary.txt`) |
| generator and async loops | each resume | read the collection's header again, check the position | 1.17x and 1.18x of a plain C loop with `next()` inlined, the generator protocol's cost, the same whether an address or a position is kept (`threads/attack2/bench/runs/gen2_summary.txt`) |
| C callbacks | every callback from C | check slot, generation and closure type in one compare; the thread | 1.05x and 1.02x; 1.42x to 1.58x on an empty callback that can be unregistered from another thread (`threads/attack2/bench/runs/ffi3_summary.txt`, `ffi3_b2_summary.txt`) |

Not memory-model costs, and left out of every model and baseline, as in phase 3: integer overflow and
zero-divisor checks.

### 3.4 The object header and the count word

- **A class that is not `open` and has no base class gets a 4-byte header: its count word.** No handle to
  it ever needs its type at run time: every handle has the class as its static type, and a `dyn I` value
  carries its method table beside the pointer, with the type id in that table (references attack 3.2 fix
  1 compares an id found there). That the C backend can always do without the type index for such classes
  is my reading, UNVERIFIED in the compiler. W2's `Node` packs the count with its 4-byte `value`: 40 bytes,
  plain C's node size.
- **Every other class gets 8 bytes**: the count word and a 32-bit type index, used for virtual calls,
  casts and `isinstance` (W3's `GameObject`). The index costs 0.99x to 1.03x of today's type pointer on
  virtual calls (`ownership-attack2/bench/runs/vcall_summary.txt`).
- **The count word's bits**: 31 "shared with another thread" (the shared flag); 30 "claimed by a parallel
  task", always taken with an atomic bit-test-and-set and dropped with an atomic AND (threads attack 3.5);
  29 SIDE: the counts live in a side record and the low bits are its index (Swift allocates a side table
  only when an object first gets a weak reference, `phase2-swift-mojo.md` line 96); 28 DEAD: the object's
  fields are dropped and only weak fields keep its block; 0 to 27 the count, or the side record's index.
  Every flag sends the fast retain and release to the slow path through the range test they already make.
- **Side records** hold `{strong u64, weak u64}`: a count that would pass 2^28 - 1 moves there, and 64
  bits cannot overflow in practice (ownership attack 5.5).
- **Objects come from per-size slabs** (graphs group 8; salvage good part 9).

---

## 4. Where the answers conflicted, and what was chosen

### 4.1 What a list is: a shared object, a copy-on-write value, or a value with names

- Python: a list is a shared object; `b = a` shares it.
- "Copy instead of share" (salvaged): a list is a value that copies on write. Fast enough after the
  salvage (W1 1.21, W2 1.24 of same-layout C, W3 0.82, W4 1.04; salvage section 0), but
  `matrix.append(row); row.append(1)` gives `[[]]` where Python gives `[[1]]`, silently, and the salvage
  found no fix that keeps the rule (salvage 2.4).
- "Names follow values" (ownership group, as saved by its attacks): a list is a value with one owner, and
  `row` becomes a name for `matrix[0]`. Python's result in the common cases, nothing counted, and a
  compile-time error with fix-its where it cannot follow.
- Making every list a counted object would give Python's meaning everywhere, at the price phase 2 measured
  for counted objects against inline values: 3.56x on iteration, 9.74x when scattered, 2.44x the memory
  (`phase2-rc-elision.md`, quoted in `phase3b-ownership.md` 3.3, lines 540-541). That price falls on every
  list.

**Chosen: names follow values.** It is the only one of the three that is fast, safe, and never silently
different from Python; what it lacks is a compile-time error with fix-its, never a silent difference.

### 4.2 An element returned from a container: a loan, or a name looked up again

- The references group (A5): `reg.get_mut[T](e)` returns a loan of the registry, so while it is in use the
  registry cannot change (Rust's rule, at no cost).
- The containers group (ECS rule 2): `pos` is "the Position of `e`", looked up again after anything that
  may change the pools.
- The ownership attack (4.6): a name's path is fixed when the name is made, so "looked up again" must mean
  "with the same handles and keys", or `n = p.items; p = Player(); n.push(1)` would push into the new
  player's list where Python's `n` is the old one's.

**Chosen: both, by what the path is, with the path fixed when the name is made.** A name built from keys or
from a read-only function that returns a place (`p = reg.get[Position](e)`, `d[k]`) is looked up again
after a change by running the same path with the same registry handle and the same `e`: the cost is the
lookup Rust's workaround writes by hand, and only after a change. A name built from a position (what a
view loop yields) follows pushes and removals, and across a swap, sort or reversal of that list it is a
compile-time error whose fix-it is the keyed lookup; the ECS `destroy` swaps, so its summary says so. A
function with side effects cannot be run again, so a name from it keeps the loan rule. This gives Python's
habit (`p = reg.get(e, Position); reg.add(...); p.x += 1` works) without a lookup on the hot path of a view
loop.

### 4.3 `weak`: cleared by registration, or a test on read

- **Registration** (graphs group, A8, as saved by both its attacks): a store records the field's address
  in a side record of the target; a dying target sets every recorded field to `None`; reads are a plain
  load. For lists of weak elements the second attack found that registering slots, then the list's header,
  each left a write into freed memory (`attacks2.exe 9`, `13`), and saved it by registering the list's
  buffer (`attacks2.exe 14`). On churn (observers subscribing and leaving) it costs 1.266 and 1.262 of plain
  C with a table keyed by address, 1.090 and 1.099 with the record found through the object
  (`runs/weak_g.txt`, `weak_h.txt`).
- **A test on read** ("copy instead of share"'s object half, salvaged): a weak field is a plain pointer
  holding a weak count on its target; a read tests the target's DEAD bit; a dead object's block waits for
  its weak count. W3 at 0.822 of plain C (salvage `d1_w3`). Rust's `Weak<T>` is this mechanism, and on the
  same churn it costs 0.919 and 0.935 of plain C (the same two files, `weakarr_rust`).

**Chosen: the test on read, with the weak count in a side record found through the count word.** Reasons:

1. **Safety by construction.** An `Array[weak T]` is a list of such pointers; a removal, a shift, a growth,
   a move of the list or a copy of it changes nothing a target needs to know (a copy adds one weak count per
   element, as Rust's `Vec<Weak<T>>` clone does). The family of holes the graphs attack found in
   registration (slots that move, lists that move out of their field) cannot occur.
2. **Speed where it matters.** On churn, the mechanism is Rust's, which measured below plain C that tracks
   its lists; registration measured 1.09x to 1.27x. On reads, it is one bit test of a header the program is
   about to read anyway (W3 reads `t.x` right after); registration saves that test. On a chain of dependent
   weak reads the test costs +18% (salvage), which is why trees use back links.
3. **Memory.** One run each, this session: `w3.c` 9,468 KB, `w3_registration.c` 11,960 KB (record found
   through the object), plain C 9,752 KB (`combined/run_once_output.txt`). Registration needs a list of field
   addresses per targeted object.
4. **A read writes nothing**, so reads of a shared holder stay reads for the thread rules.

What it costs and lacks: the bit test per read; one more load on stores to reach the side record (Rust's
`Rc` keeps its weak count in the allocation; this model keeps the 8-byte header and pays the indirection on
the rare store; UNMEASURED); and a dead object's block, though not its fields, stays until the last weak
field to it is overwritten or dies, as Rust's allocation does. In W3, 20 of the 9,173 dead objects' blocks
went back to the slab by the end (the poison build); the other 9,153 are held by weak fields of objects
that never re-target: 9,153 x 40 bytes = 366,120 bytes, inside the 9,468 KB above. The side records in use
at the end, 53,499 of 16 bytes, are about 836 KB of it.

**The time comparison waits for the final measurer**: if `combined-w3` is more than 15% slower than
`combined-w3-registration`, the choice should be looked at again with both numbers.

### 4.4 The header: 8 bytes, or 4

The ownership group proposed 8 bytes for every class. The graphs attacks measured the 8-byte header on W2
at +23%, +15%, +2% and +7% (first attack) and +17% and +14% (second attack, `w2_a.txt`, `w2_b.txt`), and a
4-byte header at 0.964 and 1.024 (first attack). **Chosen: 4 bytes for a class that is not `open` and has
no base, 8 otherwise** (3.4). W2's memory with it equals plain C's in the same layout (76,724 against
76,684 KB, one run each). The ownership attack (5.3) found that the header's size is about 3% of the class's
cost in its sharing workload; the rest is counting (4.9).

### 4.5 Loops and code the compiler cannot see

Two complete answers exist for a loop that removes from its own list and calls code the compiler cannot
see that may change that list; the containers attack (5.8) lays them side by side and leaves the pick to
the owner, and the second ownership attack (3.7) chose the compile-time side.

- **The run-time mode** (containers attack 5.5 and 5.7): the loop holds a walking state for its path,
  publishes `r`, `w` and flags before each unseen call, and other code that touches the walked list sees
  Python's list (length, reads, writes, removals, appends) through a mapping; replacing, moving, dropping or
  reordering anything on the path, holding a view of the list, and a second removing loop over it panic at
  run time. Its mapping is checked by `containers-attack/a4/walker_check.c` (3,000 random walks). Cost in
  the loop: 1.02x and 1.00x of plain C written to be correct for an unknown callback, 1.26x and 1.34x of C
  that knows its callback only reads the length (`containers-attack2/runs/a4_cb_summary.txt`,
  `a4_cb_2_summary.txt`). Cost elsewhere: other code that may run during a walk tests a walking state on
  accesses to list fields, hoisted out of loops with no unseen call (argued; its cost on a whole program is
  UNMEASURED), and the state needs a home (a per-field word, which the lean class removed, or a count-word bit
  plus a per-thread record of walks, UNVERIFIED that the bit can be spared).
- **The compile-time rule** (the first combined run, 4.5): three cases. (1) Every call in the body is
  visible: the one-pass filter. (2) The body calls unseen code that may change the list and removes nothing
  itself: the loop walks by position and reads the list's length and buffer again after each such call,
  exactly as Python's list iterator does; an element appended by that code is visited, and a removal by it
  may make the loop skip one element, as in Python; a class element used after the call is retained across
  it; a struct element used after it is a compile-time error. (3) The body also removes from the list: a
  compile-time error with fix-its (collect the removals and apply them after the loop; or state the
  callee's effects with `@effects(...)`).

**Chosen: the compile-time rule.** The first run's reason (the mode is slower than C) is withdrawn: against
C that is correct for an unknown callback, the mode is on par. The reasons now:

1. It costs nothing in any program; the mode puts a test into other code whose cost nobody has measured,
   and the job's lesson from last night is that run-time checks spread into programs that never needed them.
2. Both are as safe as Rust (the mode by its mapping and path rule, the rule by refusing).
3. Neither is fully Python: the mode runs the program (without Python's skip) but panics at run time when
   the unseen code inserts, sorts or replaces the walked list, which Python allows; the rule refuses the
   program at compile time, naming the line and the fix-its.

So **this annoyance has no answer that meets all three goals**: case 3 is refused where Python runs it. The
case is narrow (a loop that both removes from its own list and calls code from elsewhere that may change
that list), and every other loop pays nothing. If the owner prefers running case 3, the mode is ready to
build, with its files: `containers-attack/a4/ember_cb_walker.c`, `walker_check.c`,
`containers-attack2/a4/nested_attack.c` (the path rule). The same choice decides a name held across unseen
calls (containers attack 3.6): with the rule, a struct element's name used after such a call is an error.

Case 2 now covers three more sources of unseen code than the first run named: a C function, whose summary
includes the callbacks C may run (threads attack 5.1: `for line in LOG: poll_events()` re-reads `LOG`, as
Python's loop sees the appended lines); a `yield`, at which the consumer runs (4.6); and a stored closure
whose closures are not all known. One difference from Python remains in case 2: if the unseen code replaces
the whole list (`self.handlers = []`), Python's loop goes on over the old list and this one over the new
one, because the loop walks the field, not a value taken out of it.

### 4.6 Generators: a loan pinned in the frame, or a walk by position

- The threads group (A14 rule 1) let a loan into a generator's own heap contents cross `yield`; its second
  attack found that the measured model never held an address (it kept a position), that holding one buys
  nothing (82.020 against 82.222 ms, `threads/attack2/bench/runs/gen2_summary.txt`), and that a generator
  consumed by a loop whose body itself yields lets the outer consumer free the list under it (`y15`).
- The ownership attack (3.8) found that a generator holding a loan to a list the caller then stores
  elsewhere reads freed memory, and fixed it by pinning the stored loan's place and treating `yield` as a
  call the compiler cannot see.

**Chosen: both, which agree.** A loop whose body yields walks by position and re-reads its collection's
header after each resume (threads attack 6.1, 6.2), which is case 2 of 4.5 with `yield` as the unseen code;
a loan stored in the frame pins its place, so the list may grow between `next()` calls but may not be moved
into another place (ownership attack 3.8; fix-it "store it after the generator's last use"). A generator
walking a class field re-reads the field after each resume, so it may also be stored or passed.

### 4.7 A loop variable captured by a closure

The graphs group gives each pass of a `for` loop its own variable, so closures made in a loop keep that
pass's value; Python shares one variable and its FAQ lists the result as a trap. Kept as the graphs group
chose it, and stated as a difference from Python.

### 4.8 Threads and the other answers

- `weak` and threads: a class that a `weak` field points at is not shared with an unscoped thread; a dying
  object frees its fields while another thread might be reading them. `SyncWeak[T]` with an atomic upgrade
  at Rust's price for programs that need it.
- The count word holds the shared flag and the claim bit (3.4); with netted counts (threads attack 3.1) the
  tasks' increases reach the count word with atomic adds while claims may be held, which is why the claim
  must be dropped with an atomic AND (threads attack 3.5: a plain drop lost 1,136 to 1,387 increments per
  run).
- The owner block of a list that took its text along carries a count when lists are copied; it is atomic
  only when the block can reach another thread (containers attack 6.6, using the shared flag).
- Names, positions and the ECS cursor are single-thread facts; they rely on the thread rules: a list another
  thread can reach is moved, frozen or locked.

### 4.9 Two costs that are open, not chosen

**The class re-pointed often.** The ownership attack (5.3) measured its sharing workload (1,048,576
watchers re-targeting among 262,144 players) with both programs taking objects from one block: the lean
class 1.17x, 1.17x and 1.16x of plain C. The same objects with no counts at all ran at 1.03x, so counting is
about 13% and the bigger object about 3%. The cause: re-targeting raises the new target's count, touching a
cache line plain C never touches. Counts kept beside the objects instead of in them gave 1.16x again. **No fix
found; above 15%, so not on par with C in this workload.** Rust's `Rc` costs 1.31x to 1.42x on the same
work. The C-speed spelling is a different layout, which Ember can write and the containers group makes
Pythonic (`t = players[ws[i]]` names the element): 0.71x and 0.77x of plain C's pointer version in C, 0.77x
and 0.79x in safe Rust. One direction not tried (UNVERIFIED): a handle stored into a list that provably dies
before the owner of its targets, while that owner never shrinks, needs no count; its ceiling is the no-count
program (1.02x to 1.03x).

```python
t = ws[i].target          # objects shared by handle
t.hp -= 1
```

```ember
t = ws[i].target          # class Player: 1.09x to 1.17x of plain C here
t.hp -= 1
```

```ember
t = players[ws[i]]        # struct Player in an Array, a position per watcher: plain C's fastest layout
t.hp -= 1
```

**A walk that calls a stored callback.** Under the count rule, a handle used after a call that may drop an
object of its class is retained across that call. A parent walk that reports each ancestor to a stored
callback whose effects are unknown pays a count pair per step: 1.27x, 1.29x, 1.29x and 1.32x of the same walk
without it (second graphs attack 3.5). A per-thread stack of protected objects instead costs the same (1.25x,
1.27x). It costs nothing when the callback is a parameter (each passed function gets its own compiled copy
with its exact effects, `[CLO-3]`, `[EFF-2]`), or when every closure stored in the field is known. A declared
contract on the callback's type ("drops no `Node`") is a proposal (UNVERIFIED that the effect system can
express it). Rust's `Rc` tree pays the count pair every step anyway; Rust's index arena pays nothing because
the walk holds the arena borrowed and the callback cannot remove a node.

### 4.10 What was taken from last night's three designs

From "copy instead of share": counted classes, the test-on-read `weak`, objects from slabs, dispatch by
class id, "protect at the point of use" (W4's handler), a generator consumed by a `for` compiled into the
loop (the ECS view), copy-on-write only for immutable strings shared between objects (an option, not used by
the workloads). Not taken: lists and dicts as values (4.1).

From "checked pointers": the ECS cursor published before calls that may destroy, an object running its own
method kept until it returns (now the general "protect at the point of use"), owned fields stored inline,
and slots checked by a generation number, which the C callback trampolines use. Not taken: generational
references for ordinary links (a check on every use where C has none; W2 1.24 to 1.58, salvage 3.5).

From "today's rules, but never an error": effect summaries per function; static borrowing first with a
report of every run-time check (`ember inspect --checks`); "a name keeps the value of a removed element".
Not taken: pins, tombstones and the settle (the one-pass filter and positions replace them), and ownership
by "first place stored" (the breaker's line-order trap).

---

## 5. The annoyances, one line each

"Kind" is the job's kind of answer. Costs are the measurements named in section 3.3 (PROVISIONAL), or
"none" where the answer is compile time only. "From" names the final report whose answer list holds it.

| Annoyance | What this model does | Kind | Cost when running | What is left | From |
|---|---|---|---|---|---|
| A1, a method borrowing the whole object | effect summaries per method, in the interface; unsafe and foreign code use declared effects, or every effect | smarter compile-time rule | none | a library body change can change what its callers may do and what a running program may reload | ownership attack |
| A2, holding an element while adding to the list | the name is a position, read again after a push, moved after a removal; a removed or replaced element's value moves into its name; a callee that may change the list gets a place | smarter compile-time rule | a compare and a subtract or copy per change while a name is used | sorting, swapping or reversing the list while the name is needed is an error; across unseen code that may remove: error for struct elements (4.5) | containers attack |
| A3, two elements of one list in one call | allowed, `i == j` included; two compiled copies and a compare; views copied where the callee may change their source | smarter compile-time rule | spans 1.08x (Rust 1.13x); two list parameters 0.98x; copies only where Python copies | a view into an inline enum payload across a write through the other name is an error; two written overlapping spans get C's element-by-element result | containers attack |
| A4, changing a list while looping over it | the one-pass filter; reads and writes by position compared with `r`; `enumerate`'s index is the current position; a loop over `xs[..]` is the same walk | smarter compile-time rule | 0.97x to 1.01x | Python's skip not copied; with unseen code that may change the list: section 4.5 (one case refused) | containers attack, choice in 4.5 |
| A5, returning references | the body decides what a result borrows; `-> ref T` lends without `ref` at the `return`; a pool table keyed by the type | Pythonic spelling, and a smarter rule for the table | the table: 1.05x to 1.11x of plain C (bounds checks) | an error can come from a line in a callee's body (the message must name it) | references attack |
| A6, structs holding references | a container of views is a view; an object holding a view carries the view's limit, kept through erasure and `weak`; its drop counts as a use | Pythonic spelling | none | keeping a view longer than its source is refused, as in Rust | references attack |
| A7, a value that points into itself | a value may hold views of its own heap contents; a write through an index holds the container until the views are rebuilt | smarter compile-time rule | none over the same program with the text outside (1.046 of plain C, bounds checks) | editing the text means rebuilding the views before the next use | references attack |
| A8, trees with parent pointers | `backlink(children)`, one membership change in a fixed order | new mechanism | link work 1.03x and 1.15x; with the 4-byte header 0.964 and 1.024 | a child of a dead parent sees `None`; a cycle of owning links leaks; Python's two-line unlink is one line (the second is refused) | graphs attack |
| A8, other back references | `weak` fields and lists, tested on read, counts in a side record | Pythonic spelling | one bit test per read; a side-record update per store | the dead object's block waits for its weak fields (4.3) | graphs attack, changed in 4.3 |
| A8, graphs with edges both ways | `into: Array[weak City] = backlink(roads)` | new mechanism | 0.969 and 1.081 of plain C; memory 1.044 | an undirected graph written Python's way keeps four entries per edge (a self-paired set field would keep two; argued) | graphs attack |
| A9, shared mutable state | `class` plus effect summaries; no access words, no `RefCell` | Pythonic spelling and a smarter rule | 1.09x to 1.11x with `malloc`, **1.16x to 1.17x with a slab**; Rust's `Rc` 1.31x to 1.42x | the slab gap (4.9, open); a walk calling a stored callback pays a count pair (4.9) | ownership attack |
| A10, choosing among ownership kinds | two words, `struct` and `class`; `Box` implied; a lint names classes never shared | Pythonic spelling | none | the copy-or-share choice itself | ownership attack |
| A11, moves, use after move, clone | names follow values; map-value names are positions or keys; paths fixed when made | smarter compile-time rule | one compare, register form: 1.00x of correct C; Rust's copy 1.28x | one value in two owning places needs `copy()` or a class; a struct replaced inside a call is an error offering both meanings | ownership attack |
| A12, closures capturing mutable state | captures as in Python; any variable a closure writes is a place in summaries | Pythonic spelling with smarter rules | a box costs nothing measurable (0.990, 0.998 of a C local) | loop variables are per pass (4.7); a boxed capture does not cross to unscoped threads | graphs attack |
| A13, trait objects and downcasting | `isinstance` and `is not None` narrow, flow-sensitive; a downcast of a place is a place; a generic method callable through `dyn` | Pythonic spelling | one load and compare for a final class (argued) | `isinstance` with an interface or an unparameterised generic class: proposed, not built; a type parameter returned or stored by value through `dyn` stays refused (E2050) | graphs attack, references attack |
| A14, async and generators | loops across `yield` and `await` walk by position; a loan stored in a frame pins its place; async tasks of one thread checked per `await` | smarter compile-time rule | the generator protocol (1.17x, 1.18x inlined) | a walk with no position, or a reference to an inline local, across `yield` stays refused unless the frame never moves | threads attack, ownership attack |
| A15, threads | the scope rule; thread-bound values; locks guard what is only touched while held; reductions and element reductions; netted counts; the shared flag; claims | smarter rule, Pythonic spellings, new mechanisms | reductions 1.00x; netted 1.05x (Rust's `Arc` 1.80x); claims 1.12x to 1.36x | a class written by several tasks needs a lock or a claim (1.9x to 4.0x when held elsewhere) | threads attack |
| A16, generics friction | bounds inferred from the body; a program nobody depends on may implement anything | Pythonic spelling and a smarter rule | none | "most specific wins" open; duck-typed methods and `acc = 0` proposed, the owner's call | graphs attack |
| A17, error messages that do not say what to do | not a memory-model question; every refusal here has fix-its named in the reports | | | the defects found in passing (each report's last sections; section 6 here) | |
| A18, initialisation | a helper may run on a half-built object if it reads only fields already set; a second compiled copy for the constructor | smarter compile-time rule | none (code size) | `virtual`, `dyn` and other packages' helpers wait until every field is set | ownership attack |
| A19, global mutable state | Python's `FRAME = 0` and `global FRAME`; start-up initialisation in dependency order; module locks; foreign calls include their callbacks | Pythonic spelling | plain globals | an unguarded write from an unscoped thread is an error naming the lock | threads attack |
| A20, unsafe code and C callbacks | checked, tagged trampolines; contracts read from the header; a reference from a raw pointer borrows the place it was read from; raw pointers thread-bound | new mechanism | 1.02x to 1.05x per callback | an inferred contract is still an asserted fact (`ember tcb` lists it) | threads attack |
| A21, updating a map entry | `get(k, default)`, `setdefault`, `Map(default=...)`; one lookup when the key has no effects | Pythonic spelling | one lookup, as Rust's entry API | a `Map(default=...)` read of a missing key inserts it (Python's `defaultdict`) | ownership attack |
| A22, a view of a temporary | the temporary lives as long as its views; the owner follows the view into a local; a returned view reuses the buffer only at half its capacity | Pythonic spelling with smarter rules | memory equal to a copy | a field or `mut` parameter cannot hold a hidden owner (refused, `String(w)` offered) | references attack |
| A23, moving a field out from behind `mut self` | a field held exclusively (including a loop element) may be emptied if every path refills it; assigning a whole binding rebinds it | smarter compile-time rule | within 1.5% of plain C (1.015); Rust's `mem::replace` 1.204 | relies on panics never unwinding | references attack |
| A24, struct or class decides what compiles | one rule set for both | smarter compile-time rule | none | the kind still decides copy or share | ownership attack |
| A25, a list of string views | the list carries its text's limit, or takes the text along; a word kept across `clear()` moves the free to its last use | Pythonic spelling with a smarter rule | 1.00x (Rust's workaround 1.13x) | a list that is shrunk cannot take texts along (owned words, 1.12x to 1.77x) | containers attack |
| A31, caching in a read-only method | the plain spelling compiles; `@cached` over the object's own fields | smarter rule and Pythonic spelling | the cache slot and one branch | a cached value depending on another object is written by hand | ownership attack |
| A32, a callback receiving its holder; a handler removing itself | a stored closure is an object; protected while its own arguments run if they may remove it; its body keeps its box across calls that may release it | smarter compile-time rule | 1.045 and 1.037 of plain C doing the same work | in a `for` over handlers from another package: case 2 of 4.5 (Python's skip) | graphs attack |
| A33, references out of a lock | a reference through a guard ends with the guard, on the thread that holds it; numbers and strings leave as copies; an accessor keeps the lock while the part is used | Pythonic spelling with a smarter rule | copy-out 1.00x; the accessor beats a copy only for very short uses | returning a list out of a lock needs `copy(...)` | threads attack |
| A34, clone before move for shared handles | copying a handle retains; nothing to write | Pythonic spelling | one count increment per capture | none | ownership attack |
| ECS: create and destroy inside a view loop | the library loop walks backwards by position; the cursor is published before calls that read it; every operation that takes an entity out of the walked range (`destroy`, `remove[T]`) applies the cursor rule | smarter compile-time rule | W1: section 6.1 | an inner loop over one pool that destroys entities other than its own current one keeps "every entity once" only for itself | containers attack |

---

## 6. The workloads in this model's syntax, beside Python, and the C each compiles to

The Ember programs are in this model's syntax, which today's checker refuses; `combined/ember/*.em` hold
them, and `ember/ember_check_output.txt` what today's `ember check` says. Every Python program here is a
file in `combined/python/` that prints the known checksum. Random numbers: the harness's generator, written
in each program as `rand()`:

```python
state = 42                                     # 7 for W3
def rand():
    global state
    state = (state * 6364136223846793005 + 1442695040888963407) & 0xFFFF_FFFF_FFFF_FFFF
    return state >> 33
```

```ember
state: u64 = 42
fn rand() -> int:
    global state
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
    return (state >> 33) as int
```

Today this is refused: `E0100: global is Python's; Ember declares no names as shared` (the threads group's
A19 answer makes it Python's spelling), and the module-level `state` gives `E1030: main is already declared`
because today's module-level statements form an implicit `main` (`ember/ember_check_output.txt`).

### 6.1 W1, the ECS, and the registry's `get` and `view`

**The program.**

```python
reg = Registry()
for i in range(N):
    e = reg.create()
    reg.add(e, Position(float(i % 1000), float(i // 1000)))
    if i % 2 == 0:
        reg.add(e, Velocity((i % 7) * 0.5, (i % 5) * 0.25))
for s in range(100):
    for e, pos, vel in reg.view(Position, Velocity):
        pos.x += vel.dx
        pos.y += vel.dy
        if e % 100 == s:
            reg.destroy(e)
            n = reg.create()
            reg.add(n, Position(0.0, 0.0))
            reg.add(n, Velocity(1.0, 1.0))
positions = reg.pools[Position]
print(sum(int(p.x * 4.0) + int(p.y * 4.0) for p in positions.values()) + len(positions))
```

```ember
struct Position:
    x: f32
    y: f32

struct Velocity:
    dx: f32
    dy: f32

fn main():
    reg = Registry()
    reg.group[(Position, Velocity)]()              # the owning group, made before the entities (EnTT's advice)
    for i in 0..N:
        e = reg.create()
        reg.add(e, Position(x=(i % 1000) as f32, y=(i // 1000) as f32))
        if i % 2 == 0:
            reg.add(e, Velocity(dx=(i % 7) as f32 * 0.5, dy=(i % 5) as f32 * 0.25))
    for s in 0..100:
        for e, pos, vel in reg.group[(Position, Velocity)]():
            pos.x += vel.dx
            pos.y += vel.dy
            if e.id % 100 == s:
                reg.destroy(e)
                n = reg.create()
                reg.add(n, Position(x=0.0, y=0.0))
                reg.add(n, Velocity(dx=1.0, dy=1.0))
    checksum: u64 = 0
    for pos in reg.each[Position]():
        checksum += ((pos.x * 4.0) as i64 + (pos.y * 4.0) as i64) as u64
    println(checksum + len(reg.each[Position]()) as u64)
```

The order of visits does not change the checksum: every spawn in a step is the same `Position(0, 0)`,
`Velocity(1, 1)`, and the step's spawns take the same set of new ids in any order, which is why Python's
forward walk prints the same number as the C model's backward walk (`python/w1_dicts.py`, 2111831520).

What the compiler checks here: `pos` and `vel` are names by position (slot `k` of each pool); they are not
used after `reg.destroy(e)`, whose summary says it swaps the pools, so nothing is refused. Had the body
written `pos.x += 1.0` after `reg.destroy(e)`, that line would be an error naming `destroy`, with the fix-it
`pos = reg.get[Position](e)` (4.2). Today's checker, given the same loop over today's `std.ecs.World` in
place of the `Registry` below (`ember/w1_main.em`), stops earlier: the library has no `group`
(`E2072: std.ecs.World has no method named group`) and today's `std.ecs` asks components to implement its
`Component` interface (`E2040`).

**The registry's `get` and `view`**, beside a Python registry of dicts (`python/w1_dicts.py`; the
sparse-set twin `python/w1.py` follows the Ember library line for line):

```python
class Registry:
    def __init__(self):
        self.pools = {}            # component class -> {entity: component}
        self.next_id = 0

    def get(self, e, T):
        return self.pools[T][e]

    def destroy(self, e):
        for pool in self.pools.values():
            pool.pop(e, None)

    def view(self, A, B):
        a, b = self.pools[A], self.pools[B]
        for e in list(a):                  # a copy of the keys: the loop body may add and destroy
            if e in a and e in b:          # skip what the body destroyed before its turn
                yield e, a[e], b[e]
```

```ember
NONE: u32 = 0xFFFF_FFFF

struct Entity:
    id: u32

struct Pool[T]:                        # EnTT's sparse set
    sparse: Array[u32] = []            # entity id -> slot, or NONE
    ents: Array[Entity] = []           # slot -> entity
    dense: Array[T] = []               # slot -> component
    cursor: int = 0                    # a running view loop's position; 0 when none runs

    fn slot(self, e: Entity) -> int:   # -1 when e has no T
        if e.id < len(self.sparse) and self.sparse[e.id] != NONE:
            return self.sparse[e.id] as int
        return -1

    fn leave(self, i: int):            # every removal from the walked range comes through here
        if i < self.cursor:            # a running loop has not reached slot i yet
            self.cursor -= 1
            self.swap(i, self.cursor)  # the last entity not yet reached takes its slot
            i = self.cursor            # the leaving entity now sits in a slot already visited
        self.swap(i, len(self.ents) - 1)
        self.sparse[self.ents[len(self.ents) - 1].id] = NONE
        self.ents.pop()
        self.dense.pop()

    fn remove(self, e: Entity):
        i = self.slot(e)
        if i >= 0:
            self.leave(i)

class Registry:
    pools: TypeTable = TypeTable()     # slot Pool[T] holds only a Pool[T] (references group, A5)
    next_id: u32 = 0

    fn get[T](self, e: Entity) -> ref T:          # a name: "the T of e", found again after changes
        pool = self.pools[Pool[T]]
        i = pool.slot(e)
        if i < 0:
            panic(f"entity {e.id} has no {T}")
        return pool.dense[i]

    fn destroy(self, e: Entity):
        for pool in self.pools.values():          # each pool's remove, through the table
            pool.remove(e)

    fn view[A, B](self) -> Iterator[(Entity, ref A, ref B)]:
        a = self.pools[Pool[A]]                   # (the library walks the smaller pool; A here)
        b = self.pools[Pool[B]]
        saved = a.cursor                          # loops nest: keep the outer loop's cursor
        a.cursor = len(a.ents)
        while a.cursor > 0:
            a.cursor -= 1
            e = a.ents[a.cursor]
            j = b.slot(e)
            if j >= 0:
                yield e, a.dense[a.cursor], b.dense[j]
        a.cursor = saved

    fn group[A, B](self) -> Iterator[(Entity, ref A, ref B)]:   # EnTT's full-owning group (W1)
        g = self.groups[(A, B)]          # made on first use; the entities with both sit in slots [0, g.glen)
        a = self.pools[Pool[A]]
        b = self.pools[Pool[B]]
        saved = g.cursor
        g.cursor = g.glen
        while g.cursor > 0:
            g.cursor -= 1
            yield a.ents[g.cursor], a.dense[g.cursor], b.dense[g.cursor]
        g.cursor = saved

    fn each[T](self) -> ref Array[T]:   # every T, in pool order
        return self.pools[Pool[T]].dense
```

`add` and `destroy` also keep the group: an entity that gets both components is swapped to slot `glen` of
both pools and `glen` grows; an entity that leaves the group, by `destroy` or by `remove[T]` of one of the
two, goes through the group's own `leave`, which applies the same cursor rule as `Pool.leave` before it
swaps the entity to slot `glen - 1` (`group_leave` in `combined/w1.c`). The containers attack showed why the
rule must sit in that one operation: a `remove[Velocity]` written as a plain swap with the last group slot
moved an entity that had joined during the loop into the part not yet reached, and the loop visited it
(`containers-attack2/ecs/group_check_output.txt`, seed 2).

What the compiler does with `view` and `group`: the generator is consumed by a `for` whose body it can see,
so it is compiled into that loop (salvage good part 5). The loop writes the cursor on every step, but only
`leave` reads it, so the compiler keeps the cursor in a register and stores it just before a call whose
summary reads it, and reads it back after. The pools' arrays are kept in registers on the common path, where
nothing can change a pool, and read again after the rare branch. The yielded `a.dense[g.cursor]` is a name by
position (4.2).

**What `combined/w1.c` emits**, against plain C in the same layout (`salvage/plain/w1_group.c`), with the same
library structure (the pools called directly; a registry that reaches them through its type table adds one
load on the rare path, not modelled):

- **The common path is plain C's loop.** From `clang -O2 -S` of both: one load of the entity id, two float
  load-add-store pairs, the `% 100` test and its branch. Plain C also reloads its three array pointers from
  its global pools on every step (its compiler cannot tell that only the rare branch changes them); the model
  keeps them in registers. No bounds check on the common path: the position only decreases from a value
  checked against the three arrays' lengths when the loop starts. The second run's `w1.c` compiles `main` to
  the same instructions as the first run's (section 2.2).
- **The rare path adds**: the cursor store before `destroy` and its read-back after; one check of the
  read-back position against the three lengths; bounds checks on every index inside the library's `swap`,
  `leave` and `add` that the compiler cannot prove (indexes read out of `sparse` and `ents`); the compare of
  the leaving entity's slot with the cursor; the 32-bit id counter's exhaustion check in `create`.
- **No counts, no generations, no pins, no tombstones**: nothing in W1 is a class object.

Memory: 39,156 KB, the same as plain C's 39,156 KB (one run each). Earlier timings of this design in other
layouts: the containers attack's sparse-set version at 1.09x of plain C in that layout
(`containers-attack/runs/ecs_2.txt`), safe Rust 1.13x; the salvage's owning-group model with two per-entity
checks 1.057, without 1.008. This model's own time is for the final measurer. The archetype layout for
reference: plain C 42.523 ms against 88.537 ms in the owning group (salvage `runs/SUMMARY.txt`), the layout's
price before any memory model.

### 6.2 W2, the tree with parent pointers

```python
class Node:
    __slots__ = ("value", "parent", "children")
    def __init__(self, value):
        self.value = value
        self.parent = None
        self.children = []

nodes = [Node(0)]
for i in range(1, N):
    n = Node(i % 1000)
    nodes.append(n)
    p = nodes[rand() % i]
    n.parent = p
    p.children.append(n)
depth_sum = 0
for p in range(10):
    for n in nodes:
        m = n.parent
        while m is not None:
            depth_sum += 1
            m = m.parent
    for i in range(1, N):
        if (i + p) % 100 == 0:
            node = nodes[i]
            new_parent = nodes[rand() % i]
            node.parent.children.remove(node)
            node.parent = new_parent
            new_parent.children.append(node)
print(depth_sum + sum(n.value * len(n.children) for n in nodes))
```

```ember
class Node:
    value: i32
    parent: Option[Node] = backlink(children)
    children: Array[Node] = []

fn main():
    nodes = [Node(value=0)]
    for i in 1..N:
        n = Node(value=(i % 1000) as i32)
        nodes.push(n)
        nodes[rand() % i].children.push(n)            # also sets n.parent
    depth_sum = 0
    for p in 0..10:
        for n in nodes:
            m = n.parent
            while m is not None:
                depth_sum += 1
                m = m.parent
        for i in 1..N:
            if (i + p) % 100 == 0:
                nodes[rand() % i].children.push(nodes[i])   # out of the old parent's children, then in
    checksum = depth_sum
    for n in nodes:
        checksum += n.value as int * len(n.children)
    println(checksum)
```

The Ember re-parent is one line where Python's is three, and it cannot leave a node in two lists (the bug
Python's version invites when its first line is forgotten, graphs group 3.2). Today's checker refuses
`backlink` (`E1010: cannot find backlink in this scope`) and does not narrow `m` after `while m is not None:`
(`E2020: Option[Node] has no field parent` at `m = m.parent`), which this model's narrowing rule (3.2 rule 11)
covers.

**What `combined/w2.c` emits** (from the graphs group's `w2_backlink.c -DSLAB`, with the 4-byte header):

- `Node` is not `open` and has no base: a 4-byte header (the count), packed with `value`; 40 bytes, the size
  of plain C's node in the same layout. Nodes come from a slab.
- The walk is plain C's: `for (m = n->parent; m; m = m->parent) depth++`. No count, no check, no weak test:
  nothing in the walk can drop a `Node` (no call in it).
- A re-parent is one membership change: nothing in `children` is replaced, so only its middle step runs:
  search the old parent's list, order-keeping remove (Python's `list.remove`), push, store `parent`. The
  handle moves between lists: no count change, which is right only because the move is one fused step (a
  release before the retain could free a node whose only other owner is the old list; second graphs attack
  section 2).
- One retain per new node (it is in `nodes` and in its parent's `children`).
- Bounds checks on `nodes[...]`, which the compiler cannot prove (`len(nodes) == i` is a fact about the loop
  it does not track); `remove_at` checks its position.

Memory 76,724 KB, against 76,684 KB for plain C in the same layout (the graphs group's
`w2_plain_malloc.c -DSLAB -DARRAY24`: one object per node from a slab, a list of node pointers, Ember's list
header), one run each. Plain C keeping the nodes themselves in one array uses 54,940 KB (salvage `plain_w2`);
the difference is the Python program's shape (a list of objects; Ember's 24-byte list header), not the
memory model (salvage 1.3).

### 6.3 W3, game objects referring to each other

```python
import weakref

class GameObject:
    __slots__ = ("id", "x", "y", "health", "_target", "__weakref__")
    def __init__(self, id, x, y, health):
        self.id = id; self.x = x; self.y = y; self.health = health
        self._target = None
    @property
    def target(self):
        return self._target() if self._target is not None else None
    @target.setter
    def target(self, t):
        self._target = weakref.ref(t) if t is not None else None
    def update(self):
        pass

class Ship(GameObject):
    __slots__ = ()
    def update(self):
        t = self.target
        if t is not None:
            self.x += sign(t.x - self.x)
            self.y += sign(t.y - self.y)

class Rock(GameObject):
    __slots__ = ()
    def update(self):
        self.x = (self.x + 1) % 1000

class Turret(GameObject):
    __slots__ = ()
    def update(self):
        t = self.target
        if t is not None and abs(t.x - self.x) + abs(t.y - self.y) < 300:
            t.health -= 1

KINDS = (Ship, Rock, Turret)
def make(id):
    x = rand() % 1000
    y = rand() % 1000
    return KINDS[id % 3](id, x, y, 100 + id % 50)

objects = []
for i in range(M):
    o = make(i)
    if i > 0:
        o.target = objects[rand() % i]
    objects.append(o)
next_id = M; destroyed = 0
for frame in range(1000):
    for o in objects:
        o.update()
    before = len(objects)
    objects = [o for o in objects if o.health > 0]   # the dead die here, in id order
    dead = before - len(objects)
    destroyed += dead
    for _ in range(dead):
        o = make(next_id); next_id += 1
        if objects:
            o.target = objects[rand() % len(objects)]
        objects.append(o)
print(sum(o.x + o.y + o.health for o in objects) + 1_000_000 * destroyed)
```

```ember
open class GameObject:
    id: int
    x: i32
    y: i32
    health: i32
    target: weak GameObject = None
    virtual fn update(self):
        pass

class Ship(GameObject):
    override fn update(self):
        t = self.target
        if t is not None:
            self.x += sign(t.x - self.x)
            self.y += sign(t.y - self.y)

class Rock(GameObject):
    override fn update(self):
        self.x = (self.x + 1) % 1000

class Turret(GameObject):
    override fn update(self):
        t = self.target
        if t is not None and (t.x - self.x).abs() + (t.y - self.y).abs() < 300:
            t.health -= 1

fn sign(d: i32) -> i32:
    if d > 0:
        return 1
    if d < 0:
        return -1
    return 0

fn make(id: int) -> GameObject:
    x = (rand() % 1000) as i32
    y = (rand() % 1000) as i32
    health = (100 + id % 50) as i32
    if id % 3 == 0:
        return Ship(id=id, x=x, y=y, health=health)
    if id % 3 == 1:
        return Rock(id=id, x=x, y=y, health=health)
    return Turret(id=id, x=x, y=y, health=health)

fn main():
    objects: Array[GameObject] = []
    for i in 0..M:
        o = make(i)
        if i > 0:
            o.target = objects[rand() % i]
        objects.push(o)
    next_id = M
    destroyed = 0
    for frame in 0..1000:
        for o in objects:
            o.update()
        dead = 0
        for o in objects:
            if o.health <= 0:
                objects.remove(o)          # the one-pass filter: the object dies at the end of its step
                dead += 1
        destroyed += dead
        for _ in 0..dead:
            o = make(next_id)
            next_id += 1
            if len(objects) > 0:
                o.target = objects[rand() % len(objects)]
            objects.push(o)
    total = destroyed * 1_000_000
    for o in objects:
        total += (o.x + o.y + o.health) as int
    println(total)
```

The Ember `weak` field is one line where Python's is a slot, a property and a setter. Python's loop
`for o in objects: if o.health <= 0: objects.remove(o)` would skip the object after each removed one, which
is why the Python twin uses a list comprehension; the Ember loop is the containers group's filter and skips
nothing. `virtual` on the base method is today's spelling (without it today's checker says
`E2110: override of a method that is not virtual`); Python has no such word. Today's checker refuses `weak`
(`E1010: cannot find type weak`), and its `remove` takes a position, not the element
(`E2020: expected an integer, found GameObject`); `remove(e)` of the loop's current element is the containers
group's A4 spelling.

What the compiler checks: the overrides of `update` are all in this program, and their summaries say none can
drop a `GameObject` or change `objects`, so the update loop holds no count and needs no reload; `t` is used
inside its statement with nothing in between that could drop it, so the weak read takes no count (with a call
that may drop a `GameObject` between the read and the use, `t` would be retained across it: Rust's
`upgrade`).

**What `combined/w3.c` emits:**

- `GameObject` is `open`: the 8-byte header {count word, type index}; the virtual call indexes the method
  table by the type index (plain C's kind table). 40 bytes; plain C's object is 56 bytes, because it keeps a
  watcher list in every object.
- Objects from a 40-byte slab with a free list.
- A weak read: the target's DEAD bit tested, then the plain loads.
- A weak store: the target's first weak reference moves its count into a side record `{strong u64,
  weak u64}` (SIDE bit and index in the count word); its weak count goes up; the old target's goes down (and
  its count moves back to the header when it reaches zero).
- The filter: survivors copied down, the dead released. A dead object that no weak field points at goes back
  to the slab at once; one that weak fields point at has its fields dropped (its own weak target let go) and
  DEAD set, and its block goes back to the slab when its weak count reaches zero.
- Bounds checks: on `objects[rand() % i]` while building; none on `objects[rand() % len(objects)]` (Ember's
  `%` is floor modulo, `[TYP-28]`, so a positive length bounds it).

Memory: 9,468 KB against plain C's 9,752 KB, one run each; `w3_registration.c` 11,960 KB (4.3). The poison
build ran to the known checksum: 20 blocks freed, none read after free, 53,499 side records in use at the end.

### 6.4 W4, callbacks that remove themselves

The Python is the graphs group's program (`graphs-callbacks/python/w4.py`, copied to `combined/python/w4.py`).
A handler returns its counter, and the end reads every counter by calling the handler with `v = -1`, which
changes nothing; the C models and baselines do the same work.

```python
class Dispatcher:
    def __init__(self):
        self.handlers = []
        self.current = 0
    def register(self, h):
        self.handlers.append(h)
    def unregister_current(self):
        del self.handlers[self.current]
    def dispatch(self, k):
        self.current = k % len(self.handlers)
        self.handlers[self.current](self, k, k % 1000)

def main():
    total = 0
    def make_handler(m):
        counter = 0
        def handler(d, k, v):
            nonlocal counter, total
            if v >= 0:
                counter += v * m
                total += 1
                if k % 100_000 == 99_999:
                    d.unregister_current()             # removes itself, from inside its own call
                    d.register(make_handler(k % 17 + 1))
            return counter
        return handler
    d = Dispatcher()
    for i in range(10_000):
        d.register(make_handler(i % 13 + 1))
    for k in range(10_000_000):
        d.dispatch(k)
    s = 0
    for h in list(d.handlers):
        s += h(d, 0, -1)
    print(total + s)
```

```ember
class Dispatcher:
    handlers: Array[fn(Dispatcher, int, int) -> int] = []
    current: int = 0
    fn register(self, h: fn(Dispatcher, int, int) -> int):
        self.handlers.push(h)
    fn unregister_current(self):
        self.handlers.remove(self.current)
    fn dispatch(self, k: int):
        self.current = k % len(self.handlers)
        h = self.handlers[self.current]
        h(self, k, k % 1000)

fn main():
    total = 0
    fn make_handler(m: int) -> fn(Dispatcher, int, int) -> int:
        counter = 0
        fn handler(d: Dispatcher, k: int, v: int) -> int:
            if v >= 0:
                counter += v * m
                total += 1
                if k % 100_000 == 99_999:
                    d.unregister_current()
                    d.register(make_handler(k % 17 + 1))
            return counter
        return handler
    d = Dispatcher()
    for i in 0..10_000:
        d.register(make_handler(i % 13 + 1))
    for k in 0..10_000_000:
        d.dispatch(k)
    s = 0
    for h in d.handlers:
        s += h(d, 0, -1)
    println(total + s)
```

`dispatch` is written in two lines where Python writes one: today's parser reads
`self.handlers[self.current](...)` as a generic method call with a type argument (`E2072: Dispatcher has no
method named handlers`), a grammar question, not a memory one; both spellings would emit the same code (`h`
is not used after the call, so nothing protects it). Today's checker also refuses `h = self.handlers[...]`
(`E3012: cannot move out of a class field`; under this model `h` is a name) and the returned closure
(`E3026: this lambda borrows what it captures, so it may not be stored`; under this model its captures are
boxed).

What the compiler decides: `counter` is written by `handler` after it is made and `handler` is stored in a
list, so `counter` lives in the closure's box; `m` is never written, so it is copied into the box; `total` is
written by `main` and every handler, but every closure that captures it is stored only in `d`, a local made
after `total`, so all of them die before it: a pointer, no box (graphs group 4.3). `total` is written by
closures, so it is a place in effect summaries (second graphs attack 5.2). Evaluating `dispatch`'s arguments
(`self, k, k % 1000`) cannot remove a handler, so the call protects nothing (attack 6.2). The handler lends no
part of its box to `d.unregister_current()` (attack 6.3) but uses it after that call (`st->total`,
`counter`), so it keeps its box across it. The final loop over `d.handlers` calls handlers whose summary may
remove from `Dispatcher.handlers` and removes nothing itself: case 2 of 4.5.

**What `combined/w4.c` emits** (the graphs group's `w4_ember_point.c`, with the 4-byte header on the box and the
dispatcher, and without the dispatch's bounds check):

- The list element is {function, box}, 16 bytes; the box is {count, counter, m, &total}, 32 bytes, from a
  32-byte slab.
- `dispatch` copies the two words out and calls; nothing is counted or marked. No bounds check: `k % len` is
  below the length (floor modulo by a positive length) and nothing changes the list between the two lines.
- The handler's rare branch retains its own box before `d.unregister_current()` and releases it after; the
  box is freed there. Once per 100,000 events.
- The final loop reads the list's length again after every call.
- `remove` checks its position; the retain checks its count for overflow.

Memory: 4,208 KB against 4,076 KB for plain C doing the same work with the same slab
(`graphs-callbacks-attack2/bench/w4_plain_same.c -DSLAB`), one run each: each box carries a count, 32 bytes
against plain C's 24-byte environment.

---

## 7. For the final measurer

Every program prints `<name> ms=... peak_kb=... checksum=...` through the shared harness. All builds are
`"C:/Program Files/LLVM/bin/clang.exe" -O2 <file>.c -o <file>.exe`, run from `combined/`.

| Model | Compare with (same layout) | Also |
|---|---|---|
| `combined/w1.c` (`combined-w1`) | `salvage/plain/w1_group.c` (built with `-I target/redesign/bench`; the salvage's `bin/plain_w1_group.exe`) | `salvage/rust/w1_group.rs` (safe Rust, same layout); `salvage/plain/w1_arch.c` (archetype layout, for reference) |
| `combined/w2.c` (`combined-w2`) | `graphs-callbacks/bench/w2/w2_plain_malloc.c -DSLAB -DARRAY24` (`graphs-callbacks-attack2/bench/out/w2_plain_slab_array24.exe`) | `salvage/plain/w2_emberlist.c`, `salvage/plain/w2.c`, `graphs-callbacks/bench/w2/w2_rust_index.rs`, `salvage/rust/w2.rs` |
| `combined/w3.c` (`combined-w3`) | `salvage/plain/w3.c` | `combined/w3_registration.c` (`combined-w3-registration`: the other `weak`, 4.3); `salvage/rust/w3.rs` |
| `combined/w4.c` (`combined-w4`) | `graphs-callbacks-attack2/bench/w4_plain_same.c -DSLAB` (exactly the same work, same slab; `out/w4_plain_same_slab.exe`) | `salvage/plain/w4.c`; `graphs-callbacks/bench/w4/w4_rust.rs` (the workload's shape); `salvage/rust/w4.rs` (a differently shaped program, salvage 1.2) |

Decisions that wait on these times:

1. `weak` (4.3): if `combined-w3` is more than 15% slower than `combined-w3-registration`, the choice is open
   again.
2. Whether the 4-byte header removes the W2 header cost the graphs attacks saw (4.4).
3. The lean class from a slab (4.9): `ownership-attack2/bench/share_slab.c` against `share_slab0.c`; this
   model's verdict "not on par" rests on 1.16x to 1.17x, near enough to 15% that the final measurer should
   confirm it.
4. The claim's count test (threads attack 3.5): 1.12x to 1.36x here, 0.96x to 1.23x in the first attack's
   batches, with identical plain programs 4% to 17% apart.
5. The compare used by `threads(any)` trampolines (threads attack 7.1: one compare 1.53x to 1.56x, the type
   checked separately 1.42x).

---

## 8. Good parts kept from what this model did not take

| Good part | From | Where it may help |
|---|---|---|
| Registration of weak fields: reads cost nothing, memory comes back at once; the buffer registration for lists | graphs group A8 and its second attack 4.2; Objective-C's zeroing weak references | a program that reads weak fields on a chain of dependent loads; `combined/w3_registration.c` (record found through the object) is the starting point |
| The run-time "walked" loop mode: other code sees Python's list while a loop removes from it, on par with C written for an unknown callback | containers attack 5.5, 5.7, 5.8 | if the owner prefers running the refused loop of 4.5; `walker_check.c` is its self-check, `nested_attack.c` its path rule |
| Copy-on-write values | "copy instead of share" | immutable strings shared between objects |
| Generational references packed in one word {48-bit address, 16-bit generation} | "checked pointers", salvage 3.5 | C callbacks (the trampolines use slot + generation, tagged); handles that cross a C boundary |
| A generation compare per target use on the target's own cache line: W3 0.969 of plain C with a 40-byte object | "today's rules, but never an error", salvage `d3_w3` | an alternative to `weak` where a stale link should be detected rather than kept alive |
| Pins, tombstones and the settle | "today's rules, but never an error" | none left in this model; kept on record |
| Snapshots of locked data | threads group A33 | data read far more than written (a library) |
| Deferred count changes applied by the parent after the join: no atomic instruction in tasks, 1.04x on Python's re-target loop, but 1.45x memory | threads attack 3.1 | a scope whose tasks re-target few objects and memory is plentiful |
| Counting a `yield` in a consumer loop's body as unseen code | threads attack 6.2 | a smaller fix for the generator hole if the by-position walk is ever not wanted |
| A region that owns a whole graph | graphs group A8 | build-once graphs (meshes, parse trees), as a library |
| Pages that never move: appending 0.71x of doubling | phase 2, containers group 8 | an append-heavy `PagedArray`, if wanted |
| Counts beside the objects, the class known from its chunk | ownership attack 5.3 | would save the type index of `open` classes if every chunk holds one class (UNVERIFIED) |

---

## 9. Files and commands

All under `C:/Users/ism19/Code/ember/target/redesign2/combined/`:

| File | What it is |
|---|---|
| `w1.c` | W1 as emitted; `-DCURSOR_CHECK` builds the cursor self-check instead of the workload; with `-DNO_CURSOR_RULE` the check must fail |
| `w2.c` | W2 as emitted; `-DLINK_CHECK` adds the back-link and count check at the end |
| `w3.c` | W3 as emitted (weak by a test on read, side records); `-DPOISON` is the freed-block check |
| `w3_registration.c` | W3 with weak fields cleared by registration, the record found through the object (the alternative of 4.3) |
| `w4.c` | W4 as emitted; `-DPOISON` is the freed-box check |
| `*.exe` | the default builds and the self-check builds of the files above |
| `run_once.ps1`, `run_once_output.txt` | one run of every build and the four same-layout baselines (and three first-run builds, for the memory comparison) |
| `ember/w1_main.em`, `w2.em`, `w3.em`, `w4.em`, `ember_check_output.txt` | the programs in this model's syntax and today's `ember check` output |
| `python/w1_dicts.py`, `w1.py`, `w2.py`, `w3.py`, `w4.py` | the Python twins (the first run's files, unchanged); `run_twins.ps1`, `run_twins_log.txt`, `*_output.txt` my run of each |
| `first-run/` | the first run of this step: its C files (include path adjusted), builds, `run_once_output.txt` and Python outputs |

Commands (Git Bash, in `combined/`):

```text
"C:/Program Files/LLVM/bin/clang.exe" -O2 -Wall -Wextra w1.c -o w1.exe          # and w2, w3, w3_registration, w4
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DCURSOR_CHECK w1.c -o w1_cursor_check.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DCURSOR_CHECK -DNO_CURSOR_RULE w1.c -o w1_cursor_check_norule.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DLINK_CHECK w2.c -o w2_link_check.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DPOISON w3.c -o w3_poison.exe        # and w4.c -o w4_poison.exe
powershell -NoProfile -ExecutionPolicy Bypass -File run_once.ps1 > run_once_output.txt
powershell -NoProfile -ExecutionPolicy Bypass -File python/run_twins.ps1 > python/run_twins_log.txt
C:/Users/ism19/Code/ember/target/ci/ember.exe check ember/w2.em                  # and w1_main, w3, w4
```

Sources of every number not measured here: `phase3b-salvage.md` and `salvage/runs/SUMMARY.txt`;
`phase3b-graphs-callbacks-attack.md` (`graphs-callbacks-attack2/bench/runs/`) and its first run
(`graphs-callbacks-attack/bench/runs/w2_attack3_summary.txt`, `w2_attack4_summary.txt`);
`phase3b-ownership-attack.md` (`ownership-attack2/bench/runs/`); `phase3b-containers-attack.md`
(`containers-attack/runs/`, `containers-attack2/runs/`) and `phase3b-containers.md` (`containers/bench/runs/`);
`phase3b-references-attack.md` (`references/attack2/bench/out/`); `phase3b-threads-attack.md`
(`threads/attack2/bench/runs/`); `phase2-rc-elision.md` via `phase3b-ownership.md` 3.3.
