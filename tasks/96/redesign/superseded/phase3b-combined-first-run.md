# Phase 3b: one memory model for Ember, put together from the surviving answers

Written 2026-10-24 for the owner, under the job in [JOB.md](JOB.md): speed on par with C and Rust,
memory safety the same as Rust's or better, and Python's ease of reading and writing, all at once, with
the checking done when the program is compiled.

What I read: `JOB.md`; the five group reports (`phase3b-ownership.md`, `phase3b-containers.md`,
`phase3b-references.md`, `phase3b-graphs-callbacks.md`, `phase3b-threads.md`); the five attacks on them
(`phase3b-*-attack.md`, each ending with its group's final answer list); `phase3b-salvage.md`; last
night's `FINDINGS.md` and `phase3-measurements.md`; and the C models and baselines those reports name.

What I made (all under `C:/Users/ism19/Code/ember/target/redesign2/combined/`, called `combined/` below):

- `w1.c` to `w4.c`: what this model's compiler would emit for the four workloads, using the shared
  harness `target/redesign/bench/harness.h`. Each was built once with
  `"C:/Program Files/LLVM/bin/clang.exe" -O2 wN.c -o wN.exe` and run once: every one prints its known
  checksum (section 6). `w3_registration.c` is W3 with the other `weak` mechanism, kept for the measurer
  (section 4.3).
- `python/w1_dicts.py`, `python/w2.py`, `python/w3.py`, `python/w4.py`: the Python twin of each Ember
  program in section 6 (`w4.py` is the graphs group's, copied). `python/w1.py` is a second W1 twin, a
  sparse-set registry written line for line like the Ember library, by another agent working in the same
  folder (section 2). Each was run once with `C:/Python314/python.exe -I` (Python 3.14.2) and printed the
  known checksum (section 2).
- Self-check builds, each run once (section 2): the ECS cursor rule (`w1.c -DCURSOR_CHECK`), the back
  links (`w2.c -DLINK_CHECK`), no weak field pointing at a freed object (`w3.c -DPOISON`), no closure box
  used after it is freed (`w4.c -DPOISON`).

**No timing was done.** The task says the final measurer times; every time ratio below comes from an
earlier report, names its file, and is PROVISIONAL there (other agents were running). What I did measure
is **peak memory**, one run of each program, started from PowerShell with `Start-Process` and pinned to
the efficiency cores (affinity mask 0x3FC3FC) to stay off the cores other agents time on, after
`(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 (mains). Peak memory is the harness's
`peak_kb` (the process's peak working set). Nothing in `compiler/`, `std/`, `docs/` or `tests/` was
touched; no Ember program was built or run.

---

## 0. The short version

**The model in five sentences.**

1. There are **two kinds of values**: a `struct` (and every list, map, string and number) lives in one
   place and is never shared, and a `class` object lives on the heap and is shared through counted
   handles, as every Python object is.
2. `b = a` makes `b` a **second name** for the same value, as in Python; a name is kept by the compiler
   as a path ("element 0 of `enemies`", "the Position of `e`"), it holds the address only while nothing
   can change what it points into, and it looks the place up again after something may.
3. Every function has an **effect summary**, worked out from its body and kept in its package's
   interface, saying which fields it may **reshape** (change in a way that frees or moves memory); no
   pointer is ever kept across a call that may reshape what it points into, and that one rule replaces
   Rust's "one writer or many readers", its lifetimes, `RefCell`, and today's run-time access words.
4. A link that must not own is written as Python writes it: `parent: Option[Node] = backlink(children)`
   (the language keeps both ends in step; reading it is a plain load) or `target: weak GameObject`
   (reads as `None` once the target is gone; one bit test per read).
5. Closures capture variables as Python's do (the compiler picks copy, move, pointer or a counted box),
   a stored closure protects its own state only on the rare call that may remove it, and threads are
   checked against each other from the same effect summaries.

**The kinds the programmer sees**: two kinds of values (`struct`, `class`) and two kinds of
non-owning links (`backlink`, `weak`). Everything else a Rust programmer writes (`&`, `&mut`,
lifetimes, `Box`, `Rc`, `RefCell`, `Cell`, `Weak`, `Send`, `Sync`, `move`) is gone or decided by the
compiler. The kinds of pointer the compiled program uses: names (compile-time paths), counted handles,
and the two non-owning links.

**What runs while the program runs**: a count change when a class handle is stored where the compiler
cannot see through; a bounds check on every index the compiler cannot prove; bookkeeping on stores into
`weak` fields and a one-bit test when one is read; the search-and-move C does when a back-linked node
changes parent; one counted box per stored closure; locks and atomic counts only for objects that
really cross threads. Section 3.3 lists each with its measured cost.

**The four workloads** (section 6). Checksums: my one run of each model. Memory: one run of each model
and of plain C in the same layout, this session (section 2). Time: the earlier provisional measurement
of the same mechanism, with its source; this model's own times are for the final measurer.

| Workload | C model | Checksum | Peak memory, model / plain C same layout | Earlier time of the same mechanism, ratio to plain C same layout (PROVISIONAL) | Safe Rust, same work (salvage `runs/SUMMARY.txt`) |
|---|---|---|---|---|---|
| W1, ECS in EnTT's owning group | `combined/w1.c` | 2111831520 (known) | 39,336 / 39,188 KB = **1.00** | the "today's rules, but never an error" salvage model in the same layout, without its two per-entity bounds checks, which this model proves away: 1.008; with them 1.057 (salvage `runs/SUMMARY.txt`) | 1.166 |
| W2, tree with parent pointers | `combined/w2.c` | 647034814 (known) | 76,684 / 76,648 KB = **1.00** | back links with this model's 4-byte header: 0.964 and 1.024 (graphs attack, `bench/runs/w2_attack3.txt`, `w2_attack4.txt`) | 1.354 (against Ember's list layout) |
| W3, game objects referring to each other | `combined/w3.c` | 9288233970 (known) | 9,148 / 9,756 KB = **0.94** | a `weak` read by a count test, objects from a slab, dispatch by class id: 0.822 (salvage `d1_w3`; it kept the weak count in a 12-byte header, this model keeps it in a side record: this model's time is UNMEASURED) | 1.283 |
| W4, callbacks that remove themselves | `combined/w4.c` | 34946993000 (known) | 4,256 / 4,076 KB = **1.04** | a stored closure as an object: 1.002 and 1.032 against plain C with the same slab (graphs attack, `bench/runs/w4_attack.txt`, `w4_attack2.txt`) | 0.712 (Rust's handler cannot reach the dispatcher; a different program, salvage 1.2) |

**Where no answer meets all three goals** (said plainly; each is in section 5 with its reason):

1. A loop that removes from the list it walks **and** calls code the compiler cannot see that may
   change that list is a compile-time error (Rust's rule). The run-time mode that would run it costs
   1.20x to 1.29x of plain C per such call in its worst case and needs per-field access words that this
   model removes (section 4.5).
2. A struct element's name used after such a call is a compile-time error (copy it first or make it a
   class); Python keeps the object.
3. One value in two owning places needs `copy(x)` or a `class`; Python shares it.
4. A cycle of owning links leaks, as in Rust and Swift; Python's collector frees it.
5. A child whose parent has died sees `None`, and a weakly referenced object that dies frees its fields
   at once but keeps its block until the last weak field to it lets go (as Rust's `Weak` does); Python
   keeps the parent alive.
6. "The most specific implementation wins" (overlapping generic implementations) and graphs whose edges
   are kept in step both ways are still open: no rules written, nothing measured.
7. A class object written by several threads needs a lock field (or must be a struct); a task cannot
   keep a parent's object past its scope without paying Rust's `Arc` price.

---

## 1. Words used

- **Place**: somewhere a value lives: a variable, a field (`p.hp`), a list element (`xs[3]`), a map
  entry (`d[k]`), a field reached through a handle (`cam.target.hp`).
- **Name**: what `x = place` makes in this model: a second name for the value in that place, not a
  copy. The compiler keeps it as a **path** (how to find the place again: "element 3 of `xs`").
- **Handle**: what a variable of a `class` type holds: a pointer to an object on the heap.
  **Count**: how many owning handles an object has; the object is freed when it reaches zero
  (reference counting, as Python, Swift and Rust's `Rc` do). **Header**: the words at the start of an
  object that hold its count and, when needed, its type.
- **Reshape**: a change that can free or move memory: replacing a value that owns memory
  (`self.items = []`), `push`, `insert`, `remove`, `clear` or `sort` on a list (a push may move every
  element to a bigger block), changing an enum's variant, dropping the last handle to an object.
  Writing a number (`p.hp -= 1`) is not a reshape.
- **Effect summary** (also **field effects**): what the compiler records about a function from its
  body: which places it reads, writes and reshapes, by field path (`self.log`) and, for objects reached
  through handles, by class and field ("reshapes `Player.inventory` of some `Player`").
- **Loan**: a pointer the compiled program holds into a place for a while (a loop's pointer into the
  list it walks).
- **View**: an address and a length into memory something else owns (`str`, a slice). Its **limit** is
  how long what it points into lives; the compiler works it out.
- **Bounds check**: the test `i < len` before `xs[i]`. **Panic**: a stop with a message; Ember's panics
  never unwind (`[PAN-1]`), so no half-finished change is ever seen after one.
- **Slab**: an allocator that hands out same-size blocks from a large chunk and reuses freed ones from
  a list; Python's own object allocator works this way.
- **Side record**: a small record kept outside an object, found from the object, for the rare objects
  that need more than the header (here: objects that a `weak` field points at).
- **Back link**: a field the language keeps equal to "the object whose list holds me".
  **Weak field**: a field that does not keep its object alive and reads `None` once the object is gone.
- **Box** (for closures): a small counted heap block holding a closure's captured state.
- **Code the compiler cannot see**: a call through a function value from another package, a virtual
  method whose overrides are not all known, a foreign (C) function. Its summary is what its type
  declares, or "may do anything".
- **Same layout**: plain C doing the same work with the same data layout, so that a ratio measures the
  memory model and not a different data structure.
- **Plain C**: C written for speed with no safety checks. **Safe Rust**: Rust with no `unsafe`.
- The job's three kinds of answer: **a Pythonic spelling** (Rust's compile-time guarantee written the
  way Python writes it), **a smarter compile-time rule** (one that makes Rust's rule unnecessary), and
  **a new mechanism**.
- Last night's three designs are named by what they did: **"copy instead of share"** (lists and dicts
  as copy-on-write values), **"checked pointers"** (objects in slots that never move, references with a
  generation number), **"today's rules, but never an error"** (today's borrow checker with run-time
  fallbacks).
- **PROVISIONAL**: measured while other agents were running; a decision resting on a difference under
  15% waits for the final measurer. **UNVERIFIED**: believed, not checked.

---

## 2. How this was put together

Each group's final answer list (the last section of each attack report) is the input; the salvage
report's good parts are candidates. Where two answers cover the same ground, I kept the one that meets
more of the three goals, and when both met all three, the one that adds fewer kinds of things to the
language. Where two answers contradict each other, section 4 says which one went and why. Section 5
lists every annoyance with the answer this model keeps.

Checks I made myself, none of them a timing:

| Check | Command | Result |
|---|---|---|
| each C model gives the known checksum | `clang -O2 wN.c -o wN.exe`, then `wN.exe` once | all four equal the known checksums; `w3_registration.exe` too |
| the ECS cursor rule keeps "every entity once" | `clang -O2 -DCURSOR_CHECK w1.c -o w1_cursor_check.exe`, run | `cursor-check ok` (3,000 random loops destroying the current entity, entities not yet reached and entities already visited, and creating entities) |
| the self-check catches a broken rule | the same build with the cursor test replaced by `if (0)` (a copy in my scratch folder) | `panic: check: visited twice or not expected` |
| no weak field ever points at a freed object (W3) | `clang -O2 -DPOISON w3.c -o w3_poison.exe`: freed blocks are never reused and are marked, and every weak read checks the mark | `poison check ok: 20 blocks freed, none read after free`, known checksum; the other dead objects' blocks are still held by weak fields at the end (section 6.3) |
| the same for the registration version | a poison copy of `w3_registration.c` in my scratch folder (the same three changes) | no failure; 9,173 objects freed |
| every back link names its holder, every node is held once (W2) | `clang -O2 -DLINK_CHECK w2.c -o w2_link_check.exe` | `link check ok`, known checksum |
| no closure box is used after it is freed (W4) | `clang -O2 -DPOISON w4.c -o w4_poison.exe`: freed boxes are marked and never reused; the handler checks its box on entry and before its last use | `poison check ok: no box used after it was freed`, known checksum |
| the Python twins compute the same work | `python -I <file>.py`, once each, efficiency cores | `w1_dicts.py` 2111831520, `w2.py` 647034814, `w3.py` 9288233970, `w4.py` 34946993000 (`python/w2_output.txt` .. `w4_output.txt`); the other agent's `w1.py`, which it was still changing as I finished, 2111831520 in my run and again in its own later run (`python/w1_output.txt`) |
| the W1 inner loop is plain C's loop | `clang -O2 -S` of `combined/w1.c` and `salvage/plain/w1_group.c` | the same load, two float load-add-store pairs, the `% 100` test and branch; plain C also reloads the three array pointers every step, the model keeps them in registers (section 6.1) |
| all models build without warnings | `clang -O2 -Wall -Wextra -fsyntax-only` | no warnings |
| peak memory, models and plain C | `Start-Process` from PowerShell, affinity 0x3FC3FC, one run each | the table in section 0; W3's registration version 13,868 KB |

**A second agent worked in the same folder.** While I wrote this report, another agent added the
`-DLINK_CHECK` build to `w2.c` and the `-DPOISON` builds to `w3.c` and `w4.c` (inside `#ifdef`, leaving
the default builds' code unchanged), wrote `run_once_output.txt`, and built `w2_link_check.exe`,
`w3_poison.exe` and `w4_poison.exe` (the files changed at 08:36). I read those changes, rebuilt every
default build and every self-check from the current sources, and ran each once: the rows above are my
runs. The `-DPOISON` build of `w3.c` replaces my scratch copy of the same check, which gave the same
result.

One observation about the memory figures: the same program reports about 500 KB more peak memory when
started from the Git Bash shell than from PowerShell (`combined/w4.exe`: 4,800 KB from Bash, 4,256 KB
from PowerShell; my PowerShell run of the salvage's `plain_w4.exe` gave 4,076 KB, as in the salvage
report). Every comparison in this report uses runs started the same way (PowerShell), in the same
session.

---

## 3. The model

### 3.1 What a programmer writes

Each block is the Ember program under this model beside the Python a programmer would write. The
Python results quoted come from the group reports' Python checks, which name their files.

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

The compiler emits `matrix[0].push(1)` for the second line and a move instead of a free for
`a = [9]` (ownership group, section 2.2; Python results in its `python/py_checks_output.txt`). Nothing
is counted. What is left: one value stored into two owning places (`a.push(m); b.push(m)`) is a
compile-time error whose fix-its are `b.push(copy(m))` or making the type a `class`; Python would
share one object.

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
see through; a handle read into a local is not counted when nothing in the local's life may drop an
object of that class (ownership group, 2.3).

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

`mut` is inferred. Today this is `E3020` across packages (ownership group 3.1, `probes/a1_two_packages`).

**Holding an element while the list grows or shrinks** (containers group, A2):

```python
boss = enemies[0]
enemies.append(Enemy(boss.x + 1.0, boss.hp // 2))
boss.hp //= 2
```

```ember
boss = enemies[0]                  # the name "element 0 of enemies"
enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
boss.hp //= 2                      # the buffer is read again after the push: no use of freed memory
```

**Changing a list while looping over it** (containers group, A4): the loop becomes a one-pass in-place
filter. Python skips the element after a removed one; Ember does not copy that bug.

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
        enemies.remove(e)          # the current element is not kept; nothing is skipped
```

**Views** (references group, A5, A6, A22; containers group, A25). Nothing about lifetimes is written:

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

**Parent pointers and other links that must not own** (graphs group, A8):

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

**Closures** (graphs group, A12 and A32): captures work as Python's do, without `nonlocal`, `owned` or
a cell type; a handler may remove itself from inside its own call (section 6.4 is the full program).

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

**Threads** (threads group, A15): no `Send`, `Sync` or `Arc`; tasks in a scope are checked against
each other; `+=` from tasks is a per-task total joined at the end.

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
```

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())   # a reduction: no lock
    return total
```

### 3.2 What the compiler checks

All of this is decided when the program is compiled. Each rule names the group whose final answer it
is; section 4 says where I changed one.

1. **Effect summaries** (ownership group, A1). For every function, from its body and its callees'
   summaries: which places it reads, writes and reshapes, by field path for its parameters and by class
   and field for objects reached through handles. They go into the package interface for every
   function, `pub` ones included. A call through a function value, a virtual method whose overrides are
   not all visible, or a foreign function takes the effects its type declares, or every effect. A body
   with `unsafe` or a foreign call keeps its declared `mut self` as its summary. A hot reload that widens
   a summary a running caller relies on is refused. A `drop` may change only what its object owns,
   unless its class says more.
2. **Loans.** A loan is broken only by a reshape of the place it points into, or of a place containing
   it. A loan used after something that may reshape its place is a compile-time error that names the
   line. For objects, "may" means "any object of that class", because two handles can name one object;
   effects are keyed by the class that declares the field (ownership attack, 2.3).
3. **Names are paths** (containers group, A2; ownership group, A11). `x = place` names the place. The
   compiler keeps the address while nothing on the path can change. After a push it reads the buffer
   again; after a removal it moves the position (`j -= (k < j)`); if the named element itself is
   removed or replaced, its value moves into the name (Python keeps the object you removed). A name
   built from keys or from a call to a read-only function that returns a place (`reg.get[Position](e)`,
   `d[k]`) is looked up again by running the path again. A position across a sort, a swap or a reversal
   of its list is a compile-time error, as is a position across code the compiler cannot see that may
   remove from that list (section 4.5).
4. **One owner per value** (ownership group, A11). A value is in one owning place at a time; storing it
   into a second is an error with `copy()` and `class` as fix-its. A write that may replace a named
   place through another handle keeps the old value for the name, with one compare at run time.
5. **View limits are inferred** (references group). What a function's result points into is read from
   its body; a list, map or object holding views carries their limit; a temporary lives as long as the
   views made from it; a value may hold views of its own heap contents (A7); a list of views whose text
   would die takes the text along (containers group, A25).
6. **The back-link doors** (graphs group, A8, as saved by its attacker): a back link cannot be assigned
   (its fix-it is the push that makes it true), except where the same block makes it true; the list it
   pairs with cannot be lent as a plain mutable list or view; the link's hidden writes are in effect
   summaries.
7. **Closure captures** (graphs group, A12): per captured variable, the cheapest place that keeps
   Python's meaning (copy, move, pointer when every closure provably dies first, else a counted box); a
   boxed variable is a heap place in effect summaries; a closure that captures a box is not sent to an
   unscoped thread.
8. **Narrowing** (graphs group, A13): after `if isinstance(s, Circle):`, `s` is a `Circle` in the block,
   until any write of that field in any object, hidden writes included.
9. **Threads** (threads group): the scope rule (tasks against each other and against the parent, from
   summaries); reductions only in a task's own body; a lock field guards the other fields once the
   object is shared; a reference taken through a lock lives no longer than the lock; objects reachable
   from another unscoped thread are marked shared at the point of sharing; a class that a `weak` field
   points at is not shared with an unscoped thread (`SyncWeak` for that).
10. **Generics** (graphs group, A16): bounds are inferred from the body and recorded in the interface.

### 3.3 What runs while the program runs, and what it costs

| Part | When it runs | What it does | Measured cost (PROVISIONAL unless said) |
|---|---|---|---|
| counts on class handles | a handle stored where the compiler cannot see through, or overwritten; not when read into a local whose life cannot drop that class | a plain increment or decrement with a range test (a count past 28 bits moves to the side record, 3.4); atomic only for objects with the shared bit | the class mechanism at 1.05x and 1.10x of plain C with the same objects; Rust's `Rc` 1.29x to 1.39x; today's emitted code 1.60x (ownership attack, `bench/runs/share_summary.txt`, `share_slab_summary.txt`) |
| bounds checks | indexes the compiler cannot prove in range | `i < len`, panic otherwise | what safe Rust pays; in W1 the per-entity checks are proven away (1.057 with them, 1.008 without, salvage `d3_w1_group`, `d3_w1_group_nobounds`) |
| element names | a removal, or a write that may replace the named place through another handle | a compare and a subtract, or a compare and a move | the compare: 0.88x of the same code without it, emitted well; first version 1.34x; Rust's copy 1.12x (ownership attack, `bench/runs/alias_name_summary.txt`) |
| two names that may be one place (A3) | a call that may pass one place twice | compare, then the no-alias or the alias copy of the function | element pairs free; spans 1.08x of `restrict` C with bounds checks, safe Rust 1.13x (containers attack, `runs/a3_span.txt`) |
| the one-pass loop filter (A4) | a loop that changes the list it walks | read and write positions; removed elements dropped at the end of their step | 0.98x of the fastest plain C (containers, `bench/runs/a4.txt`); 1.01x with reads by position (containers attack, `runs/a4_read.txt`); Rust's idiom 1.62x |
| lists of views take their text along (A25) | a list of views outlives the text | the text's buffer moves into the list's owner block (an 8-byte pointer in the list) | 1.00x, against Rust's own workaround 1.13x and owned words 1.77x (containers attack, `runs/a25_lines.txt`) |
| back links | a back-linked node is put into or taken out of its list; the holder dies | search, order-keeping remove, push, one pointer store (what C writes) | the link's own work 0.96x to 1.05x of plain C laid out the same way (graphs attack, `bench/runs/w2_attack*.txt`) |
| `weak` fields | a store into one; a read; the death of an object a weak field points at | store: weak count +1 / -1 in the side record; read: one bit test; death: drop the fields, keep the block until the weak count is 0 | the same read test with the weak count in the header: W3 0.822 of plain C (salvage `d1_w3`); the test alone on a pointer chase +18% (salvage `d1_w2` 718.3 ms against `d1_w2_proven` 606.5 ms). This model's side-record version: UNMEASURED |
| closure boxes | a closure stored in a list or field; a closure variable some escaping closure writes | one counted box per closure from the slab for its size; the body retains its box across a call that may remove it | W4 1.002 and 1.032 of plain C with the same slab; Rust 1.43x to 1.46x (graphs attack, `bench/runs/w4_attack.txt`, `w4_attack2.txt`); a boxed captured variable 0.95x of a pointer (graphs, `runs/w4_final.txt`) |
| class objects from slabs | every class allocation | a block from a per-size chunk, no allocator header | one `malloc` per node made plain C itself 1.50x slower on W2 (graphs, `runs/w2_final.txt`: 801.4 against 535.7 ms); slabs gave W3 -19% and W4 -17% (salvage 2.5) |
| threads | only where tasks or threads really share | per-task totals joined at the end; locks where written; atomic counts for objects with the shared bit; a claim bit for parallel loops over handles with count above 1 | reductions 1.00x of plain C; tasks keeping a parent's objects 1.10x and 1.26x (Rust's `Arc` price); claims 0.96x to 1.23x (threads attack 0) |
| locks handing out parts (A33) | a value read out of a lock | a copy of a number or string, or the lock held while the part is used | copy-out 1.00x; the held lock 0.53x of a copy at 4 hashes, 1.74x at 16, 7.1x at 64 (threads attack 4.2) |
| C callbacks (A20) | every callback from C | check slot, generation and closure type in one compare; the thread | 1.05x of plain C; 1.46x on an empty callback that can be unregistered from another thread (threads attack 7.1) |

Not memory-model costs and left out of every model and baseline, as in phase 3: integer overflow and
zero-divisor checks.

### 3.4 The object header and the count word

- **A class that is not `open` and has no base class gets a 4-byte header: its count.** No handle to
  it ever needs its type at run time: every handle has the class as its static type, and a `dyn I` value
  carries its method table beside the pointer (references group 3.4 quotes the spec's type table:
  "pointer + table"). That the backend can always do without the type index for such classes is my
  reading, UNVERIFIED in the compiler. W2's `Node` packs the count with its 4-byte `value`: 40 bytes,
  the size of plain C's node (graphs attack 3.6 measured this header at 0.964 and 1.024).
- **Every other class gets 8 bytes**: the count word and a 32-bit type index, used for virtual calls,
  casts and `isinstance` (W3's `GameObject`).
- **The count word's bits**: 31 "shared with another thread" and 30 "claimed by a parallel task" (the
  threads group's); 29 SIDE: the count lives in a side record and the low bits are its index (Swift
  allocates a side table only when the first `weak` reference to an object is formed,
  `phase2-swift-mojo.md` line 96; that its count word then points at that table is from memory,
  UNVERIFIED here); 28 DEAD: the object's fields
  are dropped and only weak fields keep its block; 0 to 27 the count, or the side record's index. Every
  flag sends the fast retain and release to the slow path with the range test they already make
  (ownership attack 4.3). A count that would pass 2^28 - 1 moves into the side record, whose counts are
  32 bits.
- **Objects come from per-size slabs** (graphs group 8; salvage good part 9).

---

## 4. Where the answers conflicted, and what was chosen

### 4.1 What a list is: a shared object, a copy-on-write value, or a value with names

- Python: a list is a shared object; `b = a` shares it.
- "Copy instead of share" (salvaged): a list is a value that copies on write. Fast enough after the
  salvage (W1 1.21, W2 1.24 of same-layout C, W3 0.82, W4 1.04; salvage section 2), but
  `matrix.append(row); row.append(1)` gives `[[]]` where Python gives `[[1]]`, silently, and the
  salvage found no fix that keeps the rule (salvage 2.4).
- "Names follow values" (ownership group, as saved by its attacker): a list is a value with one owner,
  and `row` becomes a name for `matrix[0]`. Python's result in the common cases, nothing counted, and a
  compile-time error with fix-its where it cannot follow (two owning places).
- Making every list a counted object would give Python's meaning everywhere, at the price phase 2
  measured for counted objects against inline values: 3.56x on iteration, 9.74x when scattered, 2.44x
  the memory (`phase2-rc-elision.md`, quoted by the ownership group 3.3). That price falls on every
  list.

**Chosen: names follow values.** It is the only one of the three that is fast, safe and never silently
different from Python; what it lacks is stated, never silent.

### 4.2 An element returned from a container: a loan, or a name looked up again

- The references group (A5): `reg.get_mut[T](e)` returns a loan of the registry, so while it is in use
  the registry cannot change (Rust's rule, at no cost).
- The containers group (ECS rule 2): `pos` is "the Position of `e`", looked up again after anything
  that may change the pools.

**Chosen: both, by what the path is.** A name built from keys or from a call to a read-only function
that returns a place (`p = reg.get[Position](e)`) is looked up again after a change, by running the
path again: the cost is the lookup Rust's workaround writes by hand, and only after a change. A name
built from a position (`a.dense[k]`, what a view loop yields) follows pushes, but across a swap,
sort or reversal of that list it is a compile-time error whose fix-it is the keyed lookup; the ECS
`destroy` swaps, so its summary says so. A function with side effects cannot be run again, so a name
from it keeps the loan rule. This gives Python's habit (`p = reg.get(e, Position); reg.add(...);
p.x += 1` works) without a lookup on the hot path of a view loop.

### 4.3 `weak`: cleared by registration, or a test on read

- The graphs group chose **registration** (a store records the field's address with the target; a
  dying target clears every recorded field; reads are a plain load), because the ownership group's
  8-byte header had no room for a weak count. Its attacker saved it (the holder's death unregisters;
  no sharing with unscoped threads).
- The salvage's best W3 came from **a test on read** ("copy instead of share"'s object half: a weak
  field is a plain pointer, a read tests the target's strong count, a dead object's block waits for its
  weak count): 0.822 of plain C in time and 0.856 in memory (salvage `runs/SUMMARY.txt`).

I built W3 both ways, with the same objects and the same slab (`w3.c`, `w3_registration.c`), and
measured memory, one run each from PowerShell (section 2): **registration 13,868 KB, the test on read
9,148 KB, plain C 9,756 KB.** Registration costs 1.42x plain C's memory here, because every targeted
object needs a table entry and a list of field addresses. The test on read keeps the weak count in a
side record reached by an index in the count word (3.4), so the 8-byte header needs no room for it,
and W3's object stays 40 bytes.

**Chosen: the test on read, with the side record.** Reasons: memory (a 1.52x difference between the
two, from one run each, which no timing noise explains); no table lookup on any read or store; a read
writes nothing, so reads of a shared holder stay reads for the thread rules. What it costs and what it
lacks: one bit test per read (the salvage isolated a weak test at +18% on a chain of dependent loads,
which is why trees use back links, not `weak`); and a dead object's block, though not its fields, stays
until the last weak field to it is overwritten or dies, as Rust's `Rc`/`Weak` allocation does. In W3,
20 of 9,173 dead objects' blocks went back to the slab by the end (the poison build); the others are
held by objects that never re-target: 9,153 x 40 bytes = 366,120 bytes, about 358 KB (the harness's KB
are 1,024 bytes), inside the 9,148 KB above.
**The time comparison waits for the final measurer**: if `combined-w3` turns out more than 15% slower
than `combined-w3-registration`, the choice should be looked at again with both numbers (my single
runs, which are not timings, gave no reason to expect that or the opposite).

### 4.4 The header: 8 bytes, or 4

The ownership group proposed 8 bytes for every class. Its graphs attacker measured the 8 bytes at +23%,
+15%, +2% and +7% on W2 in four noisy batches, and a 4-byte header at 0.964 and 1.024 (graphs attack
3.6), and passed the question back. **Chosen: 4 bytes for a class that is not `open` and has no base,
8 otherwise** (3.4). W2's memory with it equals plain C's in the same layout (76,684 against 76,648 KB);
the graphs group's 8-byte model used 84,548 KB in my run of it this session.

### 4.5 Loops and code the compiler cannot see

The containers attack saved a run-time "walked" mode for a loop over a class field's list whose body
calls code the compiler cannot see: the loop keeps its state in the field's access word, and every
other access to that field's list sees the list as Python would (positions mapped around the loop's
gap). The ownership group's lean class removes those access words; they and their checks were part of
what made today's emitted code 1.60x against the lean class's 1.10x (ownership attack 4.2). The two
cannot both stand.

**Chosen: no run-time loop state; three compile-time cases.**

1. If every call in the body is code the compiler can see, the containers group's one-pass filter
   applies (0.98x of plain C, section 3.3).
2. If the body calls code the compiler cannot see that may change the list, and the body itself does
   not remove from the list, the loop walks by position and reads the list's length and buffer again
   after each such call, exactly as Python's own list iterator does: an element appended by that code
   is visited, and a removal by it may make the loop skip one element, as in Python. Memory stays safe
   (no address is held across the call; positions are checked against the length read after it). The
   cost is the reload a C compiler also makes after a call that may write memory. A class element used
   after the call is retained across it; a struct element used after it is a compile-time error (fix-its:
   copy it first, or make it a class). This covers Python's `for h in self.handlers: h(self)`, with a
   handler that removes itself (Python's skip included). One difference from Python: if that code
   replaces the whole list (`self.handlers = []`), Python's loop goes on over the old list and this one
   over the new one, because the loop walks the field, not a value taken out of it.
3. If the body also removes from the list it walks, the loop is a compile-time error with fix-its
   (collect the removals and apply them after the loop; or state the callee's effects with
   `@effects(...)` so the compiler can see them).

What is lost against the containers attack's version: case 3 (it ran, at 1.20x to 1.29x of plain C per
unseen call in the worst case, containers attack 5.6), and in case 2 Python's skip instead of "Python
without the skip". **This is an annoyance with no answer that meets all three goals**: the run-time
mode is slower than C and needs per-field state on every object; the compile-time rule refuses a
Python program. I chose the compile-time rule because the case is narrow (a loop that both removes from
its own list and calls code from another package that may change that list) and every other loop pays
nothing.

### 4.6 A loop variable captured by a closure

The graphs group gives each pass of a `for` loop its own variable, so closures made in a loop keep that
pass's value; Python shares one variable and its FAQ lists the result as a trap. Kept as the graphs
group chose it, and stated as a difference from Python, not as Python's meaning.

### 4.7 Threads and the other answers

- `weak` and threads: a class that a `weak` field points at is not shared with an unscoped thread
  (graphs attack 4.2); `SyncWeak[T]` with an atomic upgrade, at Rust's price, for programs that need
  it. Kept; with the test on read the reason is the same (a dying object frees its fields while another
  thread may be reading them).
- The count word holds both groups' bits (3.4).
- Names, positions and the ECS cursor are single-thread facts; they rely on the thread rules: a list
  another thread can reach is moved, frozen or locked (containers group 7).

### 4.8 What was taken from last night's three designs

From "copy instead of share": counted classes, the test-on-read `weak`, objects from slabs, dispatch by
class id, "protect at the point of use" (W4's handler), a generator consumed by a `for` compiled into
the loop (the ECS view), copy-on-write only for immutable strings shared between objects (an option,
not used by the workloads). Not taken: lists and dicts as values (4.1).

From "checked pointers": the ECS cursor published before calls that may destroy (L2, and its breaker's
N7 for nested loops), an object running its own method kept until it returns (now the general
"protect at the point of use"), owned fields stored inline, and slots checked by a generation number, which the threads group's C
callback trampolines also use. Not taken: generational references for ordinary links (a check on
every use where C has none; W2 1.24 to 1.58, salvage 3.5).

From "today's rules, but never an error": effect summaries per function; static borrowing first with a
report of every run-time check (`ember inspect --checks`); "a name keeps the value of a removed
element". Not taken: pins, tombstones and the settle (the one-pass filter and positions replace them;
the ECS model with pins was 1.63x to 1.90x, containers group 6.3), and ownership by "first place
stored" (the breaker's line-order trap).

---

## 5. The annoyances, one line each

"Kind" is the job's kind of answer. Costs are the groups' measurements (PROVISIONAL) or "none" where
the answer is compile time only. "From" names the report whose final answer list holds it.

| Annoyance | What this model does | Kind | Cost when running | What is left | From |
|---|---|---|---|---|---|
| A1, a method borrowing the whole object | effect summaries per method, in the interface | smarter compile-time rule | none | a library body change can change what its callers may do | ownership + attack |
| A2, holding an element while adding to the list | the name is a position, read again after a push, moved after a removal; a removed element's value moves into its name | smarter compile-time rule | a compare and a subtract after a removal while a name is used | sorting, swapping or reversing the list while the name is needed is an error; across unseen code that may remove: error for struct elements (4.5) | containers + attack |
| A3, two elements of one list in one call | allowed, `i == j` included; the call compares and picks the no-alias or alias copy | smarter compile-time rule | spans 1.08x (Rust 1.13x) | a view into an inline enum payload across a write through the other name is an error | containers + attack |
| A4, changing a list while looping over it | the one-pass filter; Python's skip not copied | smarter compile-time rule | 0.98x to 1.01x | with unseen code: section 4.5 (one case refused) | containers + attack, changed in 4.5 |
| A5, returning references | the body decides what a result borrows; `-> ref T` lends without `ref` at the `return`; a pool table keyed by the type needs no cast | Pythonic spelling, and a smarter rule for the table | the table: bounds checks only, 1.171 of plain C (references attack) | an interface method has no body to read: it keeps today's rule | references + attack |
| A6, structs holding references | a container of views is a view; an object holding a view carries its limit | Pythonic spelling | none | keeping a view longer than its source, with no way to move the source in, is refused (as Rust) | references + attack |
| A7, a value that points into itself | a value may hold views of its own heap contents | smarter compile-time rule | 1.046 of plain C, from bounds checks (references attack) | editing the text in place means rebuilding the views | references + attack |
| A8, trees with parent pointers | `backlink(children)` | new mechanism | link work 0.96x to 1.05x; W2 with the 4-byte header 0.964, 1.024 | a child of a dead parent sees `None`; a cycle of owning links leaks | graphs + attack |
| A8, other back references | `weak` fields, tested on read, counts in a side record | Pythonic spelling | one bit test per read; a side-record update per store | the dead object's block waits for its weak fields (4.3) | graphs + attack, changed in 4.3 |
| A8, graphs with edges both ways | open | | | rules for many-to-many links not written; not measured | graphs attack 4.4 |
| A9, shared mutable state | `class` plus effect summaries; no access words, no `RefCell` | Pythonic spelling and a smarter rule | the class at 1.05x to 1.10x of plain C with the same objects; Rust's `Rc` 1.29x to 1.39x | the fastest layout (structs in an `Array`, positions) is written by the programmer, as in Rust | ownership + attack |
| A10, choosing among ownership kinds | two words, `struct` and `class`; `Box` implied; a lint names classes never shared | Pythonic spelling | none | the copy-or-share choice itself | ownership + attack |
| A11, moves, use after move, clone | names follow values | smarter compile-time rule | one compare where a write may replace a named place through another handle (0.88x of the code without it) | one value in two owning places needs `copy()` or a class | ownership + attack |
| A12, closures capturing mutable state | captures as in Python; the compiler picks copy, move, pointer or box | Pythonic spelling with smarter rules | a box costs nothing measurable (0.95x of a pointer) | loop variables are per pass (4.6); a boxed capture does not cross to unscoped threads | graphs + attack |
| A13, trait objects and downcasting | `isinstance` narrows; a downcast of a place is a place; a generic method callable through `dyn` | Pythonic spelling | one load and compare for a final class (argued) | a type parameter returned or stored by value through `dyn` stays refused (E2050) | graphs + attack, references + attack |
| A14, async and generators | loans into a frame's heap contents may cross `yield`; generators consumed in place; async only in task groups | smarter compile-time rule | none | a loan into a frame's inline local across `yield` stays refused unless the frame never moves | threads + attack |
| A15, threads | the scope rule, reductions, the lock field, the shared bit, claims for parallel loops over handles | smarter rule, Pythonic spellings, one new mechanism | reductions 1.00x; tasks keeping parents' objects 1.10x, 1.26x; claims 0.96x to 1.23x | a class written by several threads needs a lock field | threads + attack |
| A16, generics friction | bounds inferred from the body; a program nobody depends on may implement anything | Pythonic spelling and a smarter rule | none | "the most specific implementation wins" is open (unproven) | graphs + attack |
| A17, error messages that do not say what to do | not a memory-model question; every refusal in this model has fix-its named in the group reports | | | the defects found in passing (each report's last sections) | |
| A18, initialisation | a helper may run on a half-built object if it reads only fields already set; the constructor calls a second compiled copy of it | smarter compile-time rule | none (one more compiled copy) | `virtual` and other packages' helpers wait until every field is set | ownership + attack |
| A19, global mutable state | Python's `FRAME = 0` and `global FRAME`; initialised at start-up in dependency order | Pythonic spelling | plain globals, 1.00x | an unscoped thread uses a `static` with a lock | threads + attack |
| A20, unsafe code and C callbacks | checked trampolines; contracts read from the C header; a reference from a raw pointer borrows the place it was read from | new mechanism | 1.05x of plain C per callback | an inferred contract is still an asserted fact (`ember tcb` lists it) | threads + attack |
| A21, updating a map entry | `get(k, default)`, `setdefault`, `Map(default=...)`, one lookup | Pythonic spelling | one lookup, as Rust | whether reading a missing key of a default map inserts it (Python does) is the owner's call | ownership + attack |
| A22, a view of a temporary | the temporary lives as long as its views; the owner follows the view; a returned owned string reuses the buffer when the result fills half of it | Pythonic spelling with smarter rules | memory equal to a copy; 0.983 to 0.849 of the copy's time | views of temporaries pushed into a list that outlives them are refused with the copy offered | references + attack |
| A23, moving a field out from behind `mut self` | a field held exclusively may be emptied if every path refills it; names follow their use | smarter compile-time rule | within 3.3% of plain C; Rust's `mem::replace` idiom 1.204 | relies on panics never unwinding | references + attack |
| A24, struct or class decides what compiles | one rule set for both | smarter compile-time rule | none | the kind still decides copy or share | ownership + attack |
| A25, a list of string views | the list carries its text's limit, or takes the text along | Pythonic spelling with a smarter rule | 1.00x (Rust's workaround 1.13x) | one list viewing two texts with different lives cannot be returned without copies | containers + attack |
| A31, caching in a read-only method | the plain spelling compiles; `@cached` for values over the object's own fields | smarter rule and Pythonic spelling | the cache slot and one branch | a cached value depending on another object is written by hand | ownership + attack |
| A32, a callback receiving its holder; a handler removing itself | a stored closure is an object; its body keeps its own box across a call that may remove it | smarter compile-time rule | 1.002, 1.032 of plain C with the same slab | in a `for` loop over handlers from another package: section 4.5 case 2 (Python's skip) | graphs + attack |
| A33, references out of a lock | a reference through a guard ends with the guard; numbers and strings leave as copies; an accessor keeps the lock while the part is used | Pythonic spelling with a smarter rule | copy-out 1.00x; the accessor beats a copy only for very short uses | returning a list out of a lock needs `copy(...)` | threads + attack |
| A34, clone before move for shared handles | copying a handle retains; nothing to write | Pythonic spelling | one count increment per capture | none | ownership + attack |
| ECS: create and destroy inside a view loop | the library loop walks backwards by position; the loop's cursor is published before calls that read it; `destroy` of an entity not yet reached keeps "every entity once" | smarter compile-time rule | W1 owning group: see 6.1 | an inner loop over one pool that destroys entities other than its own current one keeps "every entity once" only for itself | containers + attack |

---

## 6. The workloads in this model's syntax, beside Python, and the C each compiles to

The Ember programs are written in this model's syntax (proposed; today's checker refuses `backlink`,
`weak`, and most of the shapes, as the group reports show with `ember check` output). Every Python
program here is a file in `combined/python/` that prints the known checksum. Random numbers: the
harness's generator, written in each program as `rand()`:

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
`Velocity(1, 1)` and the step's spawns take the same set of new ids in any order, which is why Python's
forward walk prints the same number as the C model's backward walk (`python/w1_dicts.py`, section 2).

What the compiler checks here: `pos` and `vel` are names by position (slot `k` of each pool); they are
not used after `reg.destroy(e)`, whose summary says it swaps the pools, so nothing is refused. Had the
body written `pos.x += 1.0` after `reg.destroy(e)`, that line would be an error naming `destroy`, with
the fix-it `pos = reg.get[Position](e)` (4.2).

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

    fn remove(self, e: Entity):
        i = self.slot(e)
        if i < 0:
            return
        if i < self.cursor:            # a running loop has not reached e yet
            self.cursor -= 1
            self.swap(i, self.cursor)  # the last entity not yet reached takes e's slot
            i = self.cursor            # e now sits in a slot already visited
        self.swap(i, len(self.ents) - 1)
        self.sparse[e.id] = NONE
        self.ents.pop()
        self.dense.pop()

class Registry:
    pools: TypeTable = TypeTable()     # slot Pool[T] holds only a Pool[T] (references group, A5)
    next_id: u32 = 0

    fn get[T](self, e: Entity) -> ref T:          # a name: "the T of e", looked up again after changes
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

`add` and `destroy` also keep the group: an entity that gets both components is swapped to slot `glen`
of both pools and `glen` grows; one that leaves is swapped to slot `glen - 1` first, with the same
cursor rule as `Pool.remove` (`group_enter` and `reg_destroy` in `combined/w1.c`).

What the compiler does with `view` (and with the owning group's loop, which is the same loop over
slots `[0, glen)` of both pools): the generator is consumed by a `for` whose body it can see, so it is
compiled into that loop (salvage good part 5). The loop writes `a.cursor` on every step, but only
`remove` reads it, so the compiler keeps the cursor in a register and stores it just before a call
whose summary reads `Pool.cursor`, and reads it back after: this is the containers group's "publish
the cursor before calls that may destroy", obtained from effect summaries instead of written by hand.
The pools' arrays are kept in registers on the common path, where nothing can change a pool, and read
again after the rare branch (containers group 1.4). The yielded `a.dense[a.cursor]` is a name by
position (4.2).

**What `combined/w1.c` emits**, against plain C in the same layout (`salvage/plain/w1_group.c`), with
the same library structure (the pools called directly; a registry that reaches them through its type
table adds one load on the rare path, not modelled):

- **The common path is plain C's loop.** From `clang -O2 -S` of both: one load of the entity id, two
  float load-add-store pairs, the `% 100` test and its branch. Plain C also reloads the three array
  pointers from its global pools on every step (its compiler cannot tell that only the rare branch
  changes them); the model keeps them in registers. No bounds check on the common path: the position
  only decreases from a value checked against the three arrays' lengths when the loop starts.
- **The rare path adds**: the cursor store before `destroy` and its read-back after; one check of the
  read-back position against the three lengths; bounds checks on every index inside the library's
  `swap`, `remove` and `add` that the compiler cannot prove (indexes read out of `sparse` and `ents`);
  the compare of the destroyed entity's slot with the cursor; the 32-bit id counter's overflow check in
  `create`.
- **No counts, no generations, no pins, no tombstones**: nothing in W1 is a class object.

Memory: 39,336 KB against plain C's 39,188 KB (section 2). The earlier timings of this design in other
layouts: the containers attack's sparse-set version at 1.09x of plain C in that layout (`runs/ecs_2.txt`),
safe Rust 1.13x; the salvage's owning-group model with two per-entity checks 1.057, without 1.008.
This model's own time: for the final measurer. The archetype layout for reference: plain C 42.523 ms
against 88.537 ms in the owning group (salvage `runs/SUMMARY.txt`), the layout's price before any
memory model.

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

The Ember re-parent is one line where Python's is three, and it cannot leave a node in two lists (the
bug Python's version invites when its first line is forgotten, graphs group 3.2).

**What `combined/w2.c` emits** (from the graphs group's `w2_backlink.c -DSLAB`, with the 4-byte
header):

- `Node` is not `open` and has no base: a 4-byte header (the count), packed with `value`; 40 bytes,
  the size of plain C's node in the same layout. Nodes come from a slab.
- The walk is plain C's: `for (m = n->parent; m; m = m->parent) depth++`. No count, no check, no
  weak test.
- A re-parent: search the old parent's list, order-keeping remove (Python's `list.remove`), push,
  store `parent`. The handle moves between lists: no count change.
- One retain per new node (it is in `nodes` and in its parent's `children`).
- Bounds checks on `nodes[...]`, which the compiler cannot prove (`len(nodes) == i` is a fact about the
  loop it does not track); `remove_at` checks its position.

Memory 76,684 KB, against 76,648 KB for plain C in the same layout (the graphs group's
`w2_plain_malloc.c -DSLAB -DARRAY24`: one object per node from a slab, a list of node pointers, Ember's
list header) and 54,936 KB for plain C keeping the nodes themselves in one array (`salvage/plain/w2.c`).
The difference between those two baselines is the Python program's shape (a list of objects, 8 MB of
pointers; Ember's 24-byte list header), not the memory model (salvage 1.3).

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
    fn update(self):
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
`for o in objects: if o.health <= 0: objects.remove(o)` would skip the object after each removed one,
which is why the Python twin uses a list comprehension; the Ember loop is the containers group's filter
and skips nothing.

What the compiler checks: the overrides of `update` are all in this program, and their summaries say
none can drop a `GameObject` or change `objects`, so the update loop holds no count and needs no
reload; `t` is used inside its statement with nothing in between that could drop it, so the weak read
needs no count (with a call that may drop a `GameObject` between the read and the use, `t` would be
retained across it: Rust's `upgrade`).

**What `combined/w3.c` emits:**

- `GameObject` is `open`: the 8-byte header {count word, type index}; the virtual call indexes the
  method table by the type index (plain C's kind table). 40 bytes; plain C's object is 56 bytes,
  because it keeps a watcher list in every object.
- Objects from a 40-byte slab with a free list.
- A weak read: the target's DEAD bit tested, then the plain loads.
- A weak store: the target's first weak reference moves its count into a side record (SIDE bit and
  index in the count word); its weak count goes up; the old target's goes down (and its count moves
  back to the header when it reaches zero).
- The filter: survivors copied down, the dead released. A dead object that no weak field points at
  goes back to the slab at once; one that weak fields point at has its fields dropped (its own weak
  target let go) and DEAD set, and its block goes back to the slab when its weak count reaches zero.
- Bounds checks: on `objects[rand() % i]` while building; none on `objects[rand() % len(objects)]`
  (Ember's `%` is floor modulo, `[TYP-28]`, so a positive length bounds it).

Memory: 9,148 KB against plain C's 9,756 KB (`w3_registration.c`: 13,868 KB; section 4.3). The
`-DPOISON` build of `w3.c` (freed blocks never reused, marked, every weak read checks the mark) ran to
the known checksum with no failure and 20 blocks freed; my scratch copy of the same check also printed
the side records in use at the end: 53,499.

### 6.4 W4, callbacks that remove themselves

This is the graphs group's program (`graphs-callbacks/python/w4.py`, copied to `combined/python/w4.py`),
unchanged. A handler returns its counter, and the end reads every counter by calling the handler with
`v = -1`, which changes nothing; the C models and baselines do the same work.

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
        self.handlers[self.current](self, k, k % 1000)

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

What the compiler decides: `counter` is written by `handler` only after it is made, and `handler` is
stored in a list, so `counter` lives in the closure's box; `m` is never written, so it is copied into
the box; `total` is written by `main` and every handler, but every closure that captures it is stored
only in `d`, a local made after `total`, so all of them die before it: a pointer, no box (graphs group
4.3, rule 2). The final loop over `d.handlers` calls handlers whose summary may remove from
`Dispatcher.handlers`; the loop walks by position and reads the list again after each call (4.5,
case 2); it removes nothing itself.

**What `combined/w4.c` emits** (the graphs group's `w4_ember_point.c`, with the 4-byte header on the
box and the dispatcher, and without the dispatch's bounds check):

- The list element is {function, box}, 16 bytes; the box is {count, counter, m, &total}, 32 bytes, from
  a 32-byte slab.
- `dispatch` copies the two words out and calls; nothing after the call uses them, so nothing is counted
  or marked. No bounds check: `k % len` is below the length (floor modulo by a positive length) and
  nothing changes the list between the two lines.
- The handler's rare branch retains its own box before `d.unregister_current()` (whose summary may
  remove a stored closure, possibly this one) and releases it after; the box is freed there. Once per
  100,000 events.
- `remove` checks its position; the retain checks its count for overflow.

Memory: 4,256 KB against plain C's 4,076 KB: each box carries a count, 32 bytes against plain C's
24-byte environment (graphs group 5.3 found the same: 4,260 against 4,076 KB).

---

## 7. For the final measurer

Every program prints `<name> ms=... peak_kb=... checksum=...` through the shared harness.

| Model | Build | Compare with (same layout) | Also |
|---|---|---|---|
| `combined/w1.c` (`combined-w1`) | `clang -O2 w1.c -o w1.exe` | `salvage/plain/w1_group.c` (built with `-I target/redesign/bench`; the salvage's `bin/plain_w1_group.exe`) | `salvage/rust/w1_group.rs` (safe Rust, same layout); `salvage/plain/w1_arch.c` (archetype layout, for reference) |
| `combined/w2.c` (`combined-w2`) | `clang -O2 w2.c -o w2.exe` | `graphs-callbacks/bench/w2/w2_plain_malloc.c -DSLAB -DARRAY24` (`out/w2_plain_slab_array24.exe`) | `salvage/plain/w2_emberlist.c`, `salvage/plain/w2.c`, `salvage/rust/w2.rs` |
| `combined/w3.c` (`combined-w3`) | `clang -O2 w3.c -o w3.exe` | `salvage/plain/w3.c` | `combined/w3_registration.c` (`combined-w3-registration`: the other `weak`, section 4.3); `salvage/rust/w3.rs` |
| `combined/w4.c` (`combined-w4`) | `clang -O2 w4.c -o w4.exe` | `salvage/plain/w4.c`; `graphs-callbacks/bench/w4/w4_plain.c -DSLAB` (the same slab) | `salvage/rust/w4.rs` (a differently shaped program, salvage 1.2) |

Decisions that wait on these times: section 4.3 (`weak`: if `combined-w3` is more than 15% slower than
`combined-w3-registration`, the choice is open again); whether the 4-byte header removes the W2 header
cost the graphs attacker saw (4.4).

---

## 8. Good parts kept from what this model did not take

| Good part | From | Where it may help |
|---|---|---|
| Registration of weak fields: reads cost nothing, memory comes back at once | graphs group A8, ObjC's zeroing weak references | a program that reads weak fields on a chain of dependent loads; its memory needs the record layout fixed first (`w3_registration.c` is the starting point) |
| The "walked" loop mode: other code sees Python's list while a loop removes from it | containers attack 5.5 | if the owner prefers running the refused loop of 4.5 at 1.20x to 1.29x per unseen call; `walker_check.c` is its self-check |
| Copy-on-write values | "copy instead of share" | immutable strings shared between objects |
| Generational references packed in one word {48-bit address, 16-bit generation} | "checked pointers", salvage 3.5 | C callbacks (the threads group uses slot + generation); handles that cross a C boundary |
| Pins, tombstones and the settle | "today's rules, but never an error" | none left in this model; kept on record |
| Snapshots of locked data | threads group A33 | data read far more than written (a library) |
| A region that owns a whole graph | graphs group A8 | build-once graphs (meshes, parse trees), as a library |
| Pages that never move: appending 0.71x of doubling | phase 2, containers group 8 | an append-heavy `PagedArray`, if wanted |
| A per-class slab: 7% on the ownership attack's sharing workload (under 15%) | ownership attack 4.2 | the runtime's allocator (this model already uses per-size slabs) |

---

## 9. Files and commands

All under `C:/Users/ism19/Code/ember/target/redesign2/combined/`:

| File | What it is |
|---|---|
| `w1.c` | W1 as emitted; `-DCURSOR_CHECK` builds the cursor self-check instead of the workload |
| `w2.c` | W2 as emitted; `-DLINK_CHECK` adds the back-link and count check at the end |
| `w3.c` | W3 as emitted (weak by a test on read, side records); `-DPOISON` is the freed-block check |
| `w3_registration.c` | W3 with weak fields cleared by registration (the alternative of 4.3) |
| `w4.c` | W4 as emitted; `-DPOISON` is the freed-box check |
| `w1.exe`, `w2.exe`, `w3.exe`, `w3_registration.exe`, `w4.exe` | the default builds |
| `w1_cursor_check.exe`, `w2_link_check.exe`, `w3_poison.exe`, `w4_poison.exe` | the self-check builds |
| `run_once_output.txt` | the other agent's one run of each build (section 2), not mine |
| `python/w1_dicts.py`, `w2.py`, `w3.py`, `w4.py`, and `w1_dicts_output.txt`, `w2_output.txt` .. `w4_output.txt` | the Python twins and their printed checksums |
| `python/w1.py` | the other agent's sparse-set twin of W1 (section 2) |

Commands (Git Bash, in `combined/`):

```text
"C:/Program Files/LLVM/bin/clang.exe" -O2 w1.c -o w1.exe                      # and w2, w3, w3_registration, w4
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DCURSOR_CHECK w1.c -o w1_cursor_check.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DLINK_CHECK w2.c -o w2_link_check.exe
"C:/Program Files/LLVM/bin/clang.exe" -O2 -DPOISON w3.c -o w3_poison.exe      # and w4.c -o w4_poison.exe
./w1.exe                                                                       # once each: the checksum
```

Python twins and memory (PowerShell): `Start-Process -FilePath <exe or C:\Python314\python.exe -I <file>.py>
-NoNewWindow -PassThru`, then `$p.ProcessorAffinity = [IntPtr]0x3FC3FC` and `WaitForExit()`; peak
memory is the `peak_kb` each program prints. The poison check of `w3_registration.c` and the cursor
check with its rule removed were copies in my scratch folder, made from these files with the changes
described in section 2.

Sources of every number not measured here: `phase3b-salvage.md` and its `runs/SUMMARY.txt`;
`phase3b-graphs-callbacks.md` (`bench/runs/w2_final.txt`, `w4_final.txt`) and its attack
(`bench/runs/w2_attack*.txt`, `w4_attack*.txt`); `phase3b-ownership-attack.md`
(`bench/runs/share_summary.txt`, `share_slab_summary.txt`, `alias_name_summary.txt`);
`phase3b-containers.md` (`bench/runs/a4.txt`, `ecs_*.txt`) and its attack (`runs/a3_span.txt`,
`a4_read.txt`, `a25_lines.txt`, `ecs_2.txt`); `phase3b-references-attack.md` (`bench/out/summary.txt`);
`phase3b-threads-attack.md` (`bench/runs/`); `phase2-rc-elision.md` via `phase3b-ownership.md` 3.3.
