# Phase 3b: graphs, callbacks and dynamic types, attacked and saved

Written 2026-10-24 by the adversary of `phase3b-graphs-callbacks.md` (called "the report" below), under
the owner's job (`JOB.md`): speed on par with C and Rust, memory safety the same as Rust's or better,
Python's ease, with the checking done when the program is compiled; and before anything counts as
failed, try to save it.

How every claim here can be traced (all paths under
`C:/Users/ism19/Code/ember/target/redesign2/graphs-callbacks-attack/` unless they say otherwise):

- **What the rules would make the compiler emit**: small C programs written by me, in `c/attacks.c`,
  each with the Ember program it models in a comment above it. Memory errors were caught with
  AddressSanitizer (a clang option, `-fsanitize=address`, that stops a program with a report the moment
  it reads or writes memory it must not). Built with
  `"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address -I ../../../redesign/bench attacks.c -o attacks.exe`,
  run as `attacks.exe <1..11>` with `ASAN_OPTIONS=abort_on_error=0` (the sanitizer's runtime DLL was
  copied next to it from the LLVM install; nothing was downloaded). Every run's output is in
  `c/attacks_output.txt`. The program calls `SetErrorMode` first (the shared harness), so no dialog
  could open.
- **Python**: `python/py_attack.py`, run with `python -I` (Python 3.14.2), output in
  `python/py_attack_output.txt`; `python/w4log.py` (output `python/w4log_output.txt`). Every "Python
  gives" below is from those files.
- **Timings**: mine, made under the job's rules: `(Get-CimInstance Win32_Battery).BatteryStatus` was 2
  before every round (the report's runner checks it), pinned to the performance cores
  (`start /affinity C03C03`), one program at a time, 5 rounds, order reversed in every second round,
  medians. Runner: the report's own `graphs-callbacks/bench/run.ps1`; medians: its
  `graphs-callbacks/bench/medians.py`. Builds: `bench/build.sh` (clang 22.1.8 `-O2`, rustc 1.98.1
  `-C opt-level=2 --edition 2021`). Other agents were running: the processor load read 6% to 44%
  between batches, and the tree workload's spreads reached 55%. **Every timing is PROVISIONAL**; a
  decision that rests on a difference under 15% waits for the final measurer.
- **Ember today**: I ran no Ember program and changed nothing in `compiler/`, `std/`, `docs/` or
  `tests/`. The report's `ember check` outputs (`graphs-callbacks/probes/ember_check_output.txt`) were
  read and match what the report quotes; its "found in passing" defect was checked by reading the
  emitted C it names (section 9).

---

## 0. The short version

No answer of the group is discarded. Seven are saved by fixes, two are kept as they were, two stay open
(they were proposals without rules or proof, and still are). Five of the fixes close memory-safety holes
that programs the rules accept would hit; AddressSanitizer caught each one in the C model, and each fix
removed it. A sixth (section 3.2) breaks the same invariant as one of them and is argued, not built. None of the fixes costs anything on the measured paths.

| Annoyance | The report's answer | Verdict | What changed |
|---|---|---|---|
| A8, trees with parent pointers | **back links** (`parent: Option[Node] = backlink(children)`; the language keeps both ends in step) | **saved** | an indexed put (`xs[i] = v`, `insert`) of a node already in the same list freed a node still in the list (`attacks.exe 1`); now it panics, and Python's swap line compiles to `swap()`; mutable views (`MutSpan`) of the list are refused like `mut Array`; the hidden writes of the link are in effect summaries; Python's `child.parent = self` line is accepted when the same block makes it true |
| A8, other back references | **`weak` fields cleared by registration** | **saved** | a holder that died before its target left its field registered, and the target's death wrote into freed memory (`attacks.exe 4`); now a holder's drop unregisters; a class held by `weak` cannot be shared with an unscoped thread |
| A8, graphs | lists of `weak` edges kept in step both ways | **open** | the rules for many-to-many links were never written (rule 2 of back links does not apply), and the cost is argued, not measured |
| A12, closures capturing mutable state | **captures work as in Python**; the compiler picks copy, move, pointer or box | **saved** | a variable living in a box could be emptied by a stored closure while a loop walked it (`attacks.exe 8`); now a boxed variable is a heap place in effect summaries, so the containers group's loop rule applies; "copy" only for `Copy` types |
| A32, callbacks that remove themselves | **a stored closure is an object, protected where it is used** | **saved** | a closure held in a variable that another closure writes was freed during its own call (`attacks.exe 10`); now such a closure keeps its state in a counted box too; the body's retain costs 1.09x to 1.12x plain C when it happens on every event, and is removed when the field's closures are known (0.95x to 0.99x); hot reload may not undo the "no protection" path |
| A13, downcasting | **`isinstance` narrows** | **saved** | the narrowing outlived a hidden write of a back link and read a plain `Node` as a `Folder` (`attacks.exe 6`); now narrowing ends at hidden writes too; a constant-time subclass test that needs no whole-program numbering |
| A13, generic methods through `dyn` | **a `dyn` table entry instantiated with `V = dyn Visitor`** | **saved** | the rule looked only at the interface; an implementation body that needs `V` by value cannot be instantiated with `dyn`; now checked, with the error at the conversion to `dyn` |
| A13, registry by type | the references group's pool table and place-cast, adopted | **kept** | as saved by the references group's attacker (four free fixes, `phase3b-references-attack.md` 0) |
| A16, bounds | **bounds inferred from the body** | **saved** | an inferred bound could name an interface other packages cannot see; now refused with a request to write it |
| A16, orphan rule | **a program nobody depends on may implement anything** | **kept** | no problem found |
| A16, overlap | **the most specific implementation wins** | **open** | still unproven (the report said so); one more counterexample here: with separate compilation, two implementations run for one type in one program |

**Measured** (PROVISIONAL; ratios to plain C laid out the same way, in the same batch):

| | my batches | the report's |
|---|---|---|
| back links, the link's own work (no header) | 1.006, 1.045, 0.973, 1.044 | 1.078 |
| back links with the 8-byte header (the answer as emitted) | 1.285, 1.161, 0.996, 1.119 | 1.103 |
| safe Rust's index arena | 1.891, 1.415, 1.329, 1.434 | 1.261 |
| stored closures (callback workload), against plain C with the same slab | 1.002, 1.032 | 1.08 (the report's headline 1.00 was against plain C with `malloc`) |
| stored closures when every handler call first calls an unknown callback | 1.123, 1.086 | (not measured) |
| the same with the body's retain removed (the save) | 0.985, 0.954 | |
| safe Rust, same | 1.464, 1.430 | |

---

## 1. Words used

- **Handle**: what a variable of a `class` type holds, a pointer to an object on the heap. **Count**:
  how many owning handles an object has; it is freed at zero.
- **Header**: the words at the start of each object (count and type). The ownership group proposes 8
  bytes, a 32-bit count and a 32-bit type index, "the lean class" (`phase3b-ownership.md` 2.3).
- **Back link**: the report's answer for parent pointers: a field the language keeps equal to "the
  object whose list holds me". Its rules are numbered 1 to 7 in the report's section 3.3; I quote the
  one I attack each time.
- **Holder**: the object whose field holds another (a parent for its children; a camera for its
  `weak` target).
- **Indexed put**: a change that puts a value at a position: `xs[i] = v`, `xs.insert(i, v)`.
- **Mutable view**: a pointer and length through which elements can be replaced, `MutSpan[T]` in Ember;
  an `Array` turns into one at a `mut` site (`[SPN-1]`, spec line 918).
- **Effect summary**: what a function may change, computed from its body (the ownership group's field
  effects, `phase3b-ownership.md` 2.1). **Reshape**: a change that can free or move memory.
- **Box** (A12 and A32): a small counted heap block holding a closure's captured state, or one captured
  variable shared by a function and its closures.
- **Narrowing**: after `if isinstance(x, C):`, the compiler treats `x` as a `C` inside the block.
- **Walkers count**: the containers group's per-list count of running loops; while it is above zero,
  removing from the list panics (`phase3b-containers.md` 4.3, rule 9).
- **Panic**: a deterministic stop with a message; Ember does not unwind (`[PAN-1]`).
- **Use after free**: reading or writing memory that was freed. **Buffer overflow**: reading past the
  end of an object. Both are what Rust's rules exist to prevent.

---

## 2. What I attacked, and how

For each answer: programs its rules accept that still reach freed memory or read the wrong type; every
cost claim, re-measured where it rests on a measurement; the C models, checked line by line against
what the rules say the compiler emits; and Python programs a user would write that the answer refuses
or runs differently. For each problem: the cheapest fix, its cost, and the fixed version attacked again.

The C models of the report were rebuilt from their own sources by `bench/build.sh`, so every binary I
timed matches a source file. Checksums matched on every run (647034814 for the tree, 34946993000 for
the callbacks, 34951993000 for the callbacks with a log; the last is also what `python/w4log.py`
prints).

---

## 3. Back links (A8): attacks and saves

### 3.1 An indexed put of a node already in the same list (memory safety)

The report's rule 2: "Every operation that puts an object into `g` (push, insert, assigning an element,
assigning the whole field) first takes the object out of its current holder's `g` ... then sets its
back link." When the current holder is the same list, taking the node out shifts every later element
down by one, so the position the program named now holds a different node. The rule does not say
whether the position is found before or after that shift.

```ember
p = Node(name="p")
p.children.push(Node(name="a"))
p.children.push(Node(name="b"))
p.children[1] = p.children[0]
```

```python
p.children[1] = p.children[0]
print([c.name for c in p.children])        # ['a', 'a']   (py_attack_output.txt, assign_same_list)
```

**Position found first** (the place is evaluated, and bounds-checked, before rule 2 runs): rule 2 takes
`a` out (the list becomes `[b]`, but slot 1 still holds a stale copy of `b`), the store replaces that
stale copy, and rule 3 releases what it replaced: `b`, whose only count was the list. `b` is freed while
`p.children[0]` still holds it, and `a` sits past the list's length. `attacks.exe 1`:
`AddressSanitizer: heap-use-after-free ... READ of size 8` when the program reads `p.children[0].name`.

**Position found after** (no memory error): Python's swap line fails instead.

```ember
p.children[0], p.children[2] = p.children[2], p.children[0]
```

```python
p.children[0], p.children[2] = p.children[2], p.children[0]
print([c.name for c in p.children])        # ['c', 'b', 'a']   (swap_idiom)
```

`attacks.exe 3`: the first assignment takes `c` out (`[a, b]`) and replaces `a` (`[c, b]`); the second
asks for position 2 of a list of two: `panic: index out of range`.

**The fix.** An indexed put of a node whose back link already names this list's holder panics with a
message naming both positions ("a is already at children[0]; a node has one place: use swap(), insert()
or push()"), unless it is the node's own slot (nothing to do). The tuple swap of two elements of one
back-linked list (`xs[i], xs[j] = xs[j], xs[i]`) is compiled as `xs.swap(i, j)`, which changes no
membership and so no link. `push` of a node already in the list keeps the report's meaning (moved to the
end): it names no position, so nothing shifts under it. In every other case the position is found after
the take-out, as the bounds check must be.

`attacks.exe 2`: the panic message for `p.children[1] = p.children[0]` (the demo reports it and goes
on), then `p.children = [c, b, a]` for the swap, as Python.

**Cost**: none. Rule 2 already loads the node's link and tests it (`if (old)` in the report's
`children_push`); the new test, `old == holder`, is inside that branch, on the path that already does a
search and a memory move.

**Python lost**: `p.children[1] = p.children[0]` makes Python's list hold one node twice, which a node
with one parent cannot be; Ember panics where Python silently builds a tree that is not a tree. The
fix-it names `copy` for a second node or `swap` for a move.

**Attacked again**: `insert` of a node already in the list panics the same way (the fix-it:
`xs.insert(i, xs.pop(j))`); a put of a node from a *different* list shifts only that list, so the
position is unaffected; the assignment of the whole field (`p.children = other_list`) names no position.

### 3.2 A mutable view walks around the link (memory safety)

Rule 6: "it cannot be lent as a plain `mut Array[Node]` to a function that knows nothing of the link".
An `Array` also becomes a `MutSpan` at a `mut` site (`[SPN-1]`), and a `MutSpan` can replace elements:

```ember
fn blank_first(xs: MutSpan[Node]):        # a library function; knows nothing of back links
    xs[0] = Node(name="blank")

kept = p.children[0]                      # a second owner of the first child
blank_first(p.children)                   # not an `Array` parameter: rule 6, read literally, allows it
p = None                                  # p dies; rule 4 clears the links of p.children only
println(kept.parent)                      # still names p, which is freed
```

```python
def blank_first(xs): xs[0] = Node("blank")
blank_first(p.children)                   # Python keeps p alive through kept.parent; no error
```

The replaced child (`kept`) is no longer in `p.children` but its link still names `p`; when `p` dies,
rule 4 walks only `p.children`, so `kept.parent` is left pointing at freed memory. (Not built as a
separate attack: it is the same broken invariant that `attacks.exe 4` shows for `weak` fields.)

**The fix.** Rule 6 covers every way to replace an element without the link's own operations: a
`MutSpan`, `ref mut` of an element, `split_at_mut`, and a parameter whose inferred summary replaces
elements. Reading (`Span`, iteration, writing fields of the child objects themselves) stays allowed.
**Cost**: none.

### 3.3 The link's hidden writes must be in effect summaries

`new_parent.children.push(node)` writes `node.parent` and reshapes the *old* parent's `children`; the
death of any `Node` writes the `parent` field of each of its children (rule 4). None of these writes is
in the source. Three other rules read effect summaries and would miss them: loans (a loan into the old
parent's list across the push), narrowing (section 7.1, where it is a memory error), and the threads
group's share check ("no plain field written after the share", `phase3b-threads.md` 4.6).

**The fix.** The operations of rules 2, 3 and 4 have their writes in summaries: a put into `C.g` writes
`D.f` (the link) and reshapes `C.g` of some `C`; any release that may drop a `C` writes `D.f` of some
`D`. **Cost**: none when running.

### 3.4 Python's own tree code writes both ends (readability)

Rule 5: "The back link cannot be assigned (`node.parent = p` is a compile error whose fix-it is
`p.children.push(node)`)". The first line of the Python every programmer writes is then an error:

```python
def add(self, child):
    child.parent = self                   # refused by rule 5
    self.children.append(child)

def reparent(node, new_parent):
    node.parent.children.remove(node)
    node.parent = new_parent              # refused by rule 5
    new_parent.children.append(node)
```

(`py_attack_output.txt`, `python_add_writes_both_ends`: `('p', ['c'])`.) The report's reason for rule 5
holds: if assigning the link *meant* a push, Python's four-line linked-list insert would move nodes
twice and lose one. So the save keeps the meaning and accepts the line:

**The fix.** A write to a back link is accepted when it states what the same block makes true: on every
path from the write, before any read of that field and before any call that may read it, the paired
operation puts the node into that holder's list (or, for `= None`, takes it out of its list). The
compiler checks it and removes the write; the link stays maintained by the language. Any other write
is today's rule 5 error with its fix-it. With it, Python's `add` and `reparent` compile as written:

```ember
fn add(self, child: Node):
    child.parent = self                   # accepted: the next line makes it true; removed
    self.children.push(child)

fn reparent(node: Node, new_parent: Node):
    node.parent.unwrap().children.remove_value(node)
    node.parent = new_parent              # accepted: the push below makes it true; removed
    new_parent.children.push(node)
```

(`remove_value` stands for Python's remove-by-value, whatever Ember's list calls it.) Refused, as
before: `c.parent = p` with no push, and Python's doubly linked insert, whose third line reads
`a.next.prev` after the first line has already moved `c` out of `a.next`.

**Cost**: none when running (the write is removed). The check is a definite-assignment-style walk over
one block, argued here and not built (UNVERIFIED that it accepts every shape a Python programmer
writes; Python's detach loop, `for k in p.children: k.parent = None` then `p.children = []`, needs the
check to see across the loop).

### 3.5 Threads

A tree with back links shared with an unscoped thread: when its root's last owner releases it on one
thread, rule 4 writes the `parent` field of every child, which another thread may be reading. With 3.3,
that write is in the summaries, so the threads group's own share check refuses the share at compile time
("`Node.parent` is written ... while a thread shares the object"). Scoped tasks reading a tree while
another task drops a `Node` conflict under their rule 2 for the same reason. That is Rust's position
(an `Rc` tree is not `Send`). No extra fix beyond 3.3.

### 3.6 The cost, re-measured and taken apart

The report's model (`graphs-callbacks/bench/w2/w2_backlink.c`) does what the rules say: the walk is a
pointer chase with no count, and a re-parent is the search, order-keeping remove, push and link store
that plain C does. One difference from its "same layout" plain C is not the link: the Ember model's
`nodes` list grows by `push` from 4 and checks every index, where the plain C allocates the million
pointers once and checks nothing. I took the model apart (`bench/w2_backlink_parts.c`: `-DNOHEADER`,
`-DPLAINNODES` for plain C's `nodes`, `-DPLAINCHILD` for plain C's unchecked search, `-DHDR4` for a
4-byte header).

Four batches (`bench/runs/w2_attack.txt` to `w2_attack4.txt`, summaries beside them), median ms and
ratio to plain C laid out the same way (`w2_plain_malloc.c -DSLAB -DARRAY24`) in the same batch:

| Program | batch 1 | batch 2 | batch 3 | batch 4 |
|---|---|---|---|---|
| plain C, one array, order-keeping remove (`w2_plain_keep.c`) | 603.0 (0.864) | 627.6 (0.888) | 681.8 (0.845) | 577.4 (0.889) |
| plain C, same layout | 697.9 (1.000) | 706.6 (1.000) | 806.7 (1.000) | 649.6 (1.000) |
| back links, 8-byte header (**the answer as emitted**) | 896.7 (1.285) | 820.4 (1.161) | 803.4 (0.996) | 727.2 (1.119) |
| back links, no header | 729.6 (1.045) | 711.1 (1.006) | 785.2 (0.973) | 678.0 (1.044) |
| back links, 4-byte header (`-DHDR4`) | | | 777.9 (0.964) | 665.5 (1.024) |
| back links, no header, plain C's `nodes` | 677.9 (0.971) | 688.8 (0.975) | | 667.5 (1.028) |
| the same, plain C's search too | 704.7 (1.010) | 681.2 (0.964) | | |
| safe Rust, index arena (`w2_rust_index.rs`) | 1319.6 (1.891) | 999.8 (1.415) | 1071.8 (1.329) | 931.7 (1.434) |

Spreads (slowest run over fastest) were 25% to 47% in batch 1, 5% to 20% in batch 2, 18% to 55% in
batch 3, 1% to 22% in batch 4: other agents' work.

What it shows (PROVISIONAL):

- **The back link itself costs nothing**: with plain C's `nodes` list and no header it is 0.96x to
  1.03x of plain C in every batch. The report's claim stands.
- **Ember's growing, checked `nodes` list** adds about 2% to 8% (no header, batch by batch: 1.045
  against 0.971, 1.006 against 0.975, 1.044 against 1.028 with plain C's list). Rust's index arena pays its own checks too.
- **The 8-byte header** is the rest, and it is not stable: the report measured about 2%; my batches give
  +23%, +15%, +2% and +7% over the same program without it. The node grows from 40 to 48 bytes, and
  this tree (77 to 85 MB peak working set) is larger than the processor's cache, so the extra bytes may
  cost more when other programs use the cache too (my reading of the spreads; UNVERIFIED: no cache
  counters were read and the cache size was not checked).
  A class that is not `open` and has no base class may need no type index (the index serves virtual
  calls, casts and drops through an interface, `phase3b-ownership.md` 2.3); its header can then be the
  4-byte count, which packs with a 4-byte field: `-DHDR4` is 40 bytes and measured 0.964 and 1.024.
  Whether Ember's `dyn` values carry their table beside the pointer, which this needs, is UNVERIFIED;
  this is passed to the ownership group, whose header it is.
- **Against safe Rust's index arena**: faster in every batch (0.68x, 0.82x, 0.75x, 0.78x of its time;
  the report's 0.87x).
- Every difference against plain C here is under 15% or swings across 15% between batches: the final
  measurer decides.

### 3.7 Verdict: saved

Back links with: indexed puts of a node already in the list panic, the swap line compiles to `swap()`
(3.1); every mutable view is refused (3.2); the hidden writes are in summaries (3.3); a back-link write
that the same block makes true is accepted and removed (3.4). Memory-safe by the report's invariant
with those doors closed; the link costs nothing measurable; the header's 8 bytes are the ownership
group's open question. What it lacks against Python is unchanged: a cycle of owning links leaks (as in
Rust), and a child held after its parent died sees `None` where Python keeps the parent alive.

---

## 4. `weak` fields cleared by registration (A8): attacks and saves

### 4.1 The holder dies first (memory safety)

The report: "storing into a weak field registers the field's address with the target ... an object
that dies clears every registered field, then is freed at once". It says what happens when the target
dies, not when the holder does. Its model (`graphs-callbacks/bench/w2/w2_zweak.c`) never frees a node.

```ember
player = Player()
cam = Camera(target=player)          # cam.target is registered with player
cam = None                           # the camera dies first
player = None                        # the player dies: it clears every registered field
```

```python
pl = Player(); cam = Camera(pl)      # Camera keeps weakref.ref(pl)
del cam; del pl                      # no error   (weak_holder_dies_first)
```

`attacks.exe 4`: `AddressSanitizer: heap-use-after-free ... WRITE of size 8`: the player's death writes
`None` into the camera's freed memory (in a real heap, into whatever object reused it).

**The fix.** A holder's drop stores `None` into each of its `weak` fields that holds a target, which
unregisters the field (the same code as overwriting the field). Objective-C's zeroing weak references
do this in each object's compiler-made destructor (from memory of its runtime, `objc_destroyWeak`;
UNVERIFIED here). `attacks.exe 5`: both die, nothing is written after a free.

**Cost**: a side-table lookup and a search of the target's list per holder death with a set `weak`
field. Not measured: the tree workload kills no node (UNVERIFIED; it is the same work as one weak store,
which the report did measure as part of `w2_zweak.c`).

### 4.2 Threads

The report asks for "the zeroing-weak side table ... per thread" (its section 8). The threads group lets
an object cross to another thread with a shared flag and atomic counts (`phase3b-threads.md` 4.6). A
`Player` shared that way can die on any thread, and its death clears `Camera.target` fields that belong
to objects on other threads, which read them with a plain load: a data race, and a use after free when
the reader has just loaded the pointer. Rust's answer for this case is `std::sync::Weak`, whose upgrade
counts atomically.

**The fix.** At compile time, at the share: a class that is the target of any `weak` field in the build
cannot be shared with an unscoped thread (the error names the `weak` field), the same whole-build check
the threads group already makes at a share. A standard-library `SyncWeak[T]` with Rust's mechanics (a
weak count, an atomic upgrade) serves the rare program that needs it, at Rust's cost. **Cost**: none
for everything else.

### 4.3 Verdict: saved

`weak` cleared by registration, with the holder's drop unregistering its fields, and weakly held classes
kept on their thread. Reads stay a plain load.

### 4.4 Graphs with back-linked `weak` lists: open

The report's graph answer (`roads: Array[weak City]`, `into: Array[weak City] = backlink(roads)`) is
argued, not measured, and its rules were never written: back-link rule 2 ("first takes the object out
of its current holder") cannot apply to a city that has many roads in, and the rules for duplicates
(two roads to one city) and for the death of a city are not stated. It stays **open**: it needs its own
rules and a measurement before it can be kept or discarded.

---

## 5. Captures as in Python (A12): attacks and saves

### 5.1 A captured list emptied by a stored closure while a loop walks it (memory safety)

The report's rules decide where a captured variable lives (copy, move, pointer, box). They do not say
how a variable in a box appears to the other rules. A box is reachable from every closure that captures
it, and those closures can run inside any call to code the function cannot see.

```ember
fn main():
    xs = ["ab", "cde", "f"]
    bus = Bus()
    bus.on("reset", fn(): xs.clear())        # stored and escaping: xs lives in a box
    total = 0
    for x in xs:
        bus.emit("reset")                    # runs the stored closure
        total += len(x)
    println(total)
```

```python
def boxed_loop():
    xs = ["ab", "cde", "f"]
    handlers = {"reset": lambda: xs.clear()}
    total = 0
    for x in xs:
        handlers["reset"]()
        total += len(x)
    return total                             # 2   (boxed_list_cleared_in_loop)
```

If `xs` is compiled as the function's own local, nothing the loop calls "can" change it, so the loop
keeps a pointer and a length. `attacks.exe 8`: `AddressSanitizer: heap-use-after-free ... READ of size
2` (the loop reads a string `clear()` freed).

**The fix.** A variable that lives in a box is a heap place in effect summaries: "reshapes the captured
`xs` of `main`" is an effect of every closure that writes it, and a call with unknown effects may have
it. The containers group's rule 9 then applies (positions, the walkers count, a panic if unseen code
removes). `attacks.exe 9`: `panic: xs changed while a loop over it was running (line 6)`. Python ends
the loop (total 2); Ember panics: that is the containers group's chosen trade for removals by unseen
code (`phase3b-containers.md` 4.4), not a new one. **Cost**: rule 9's, which the containers group
measured at +1.4% on `push` (their `runs/count.txt`), only in loops over boxed variables that call
unseen code.

### 5.2 "Copied" and "written after" need their exact meaning

- The report says a variable "read by the closure and never written after the closure is made" is
  **copied** ("today's `[CLO-13]`"). `[CLO-13]` copies only `Copy` variables (spec line 2070). A list
  or a string copied into every closure that reads it would be a hidden O(n) copy and a second owner,
  which the ownership group forbids (`phase3b-ownership.md` 2.2, rule 4). Fixed wording: copied only if
  `Copy`; otherwise moved, pointed to, or boxed (shared, as Python shares it).
- "Written after" must count writes by other closures, partial writes (`p.x = 5`) and mutable uses
  (`xs.push(4)`), not only assignments in the function:

  ```ember
  n = 1
  get = fn() => n
  set = fn(v: int): n = v
  set(2)
  println(get())          # must be 2: `set` writes n, so n is boxed, not copied into get
  ```

  ```python
  n = 1
  get = lambda: n
  def set_(v):
      nonlocal n; n = v
  set_(2); print(get())   # 2
  ```

  The report's own attack table ("two closures share a counter ... both capture one box") shows this is
  its intent; the rule's text must say it. **Cost**: none.

### 5.3 Closures moved to threads

The report's rule 6 ("a closure that captures a box is not sent to another thread") is about the box of a
*shared variable*: right, and it agrees with the threads group (a written field of a shared object is
refused at the share). A32's boxes are different: a stored closure's own state box. Closure values move
(`[CLO-3]`; a value is in one owning place at a time, `phase3b-ownership.md` 2.2 rule 4), so that box
has one owner and its count exists only for the body's own retain (section 6). A job queue
(`jobs.submit(fn(): work(x))`) moves the closure, and its box with it, to the worker. The threads
group's walk must treat a closure's own box as part of the moved value, not as a shared object;
otherwise a stateful job (Rust's `Box<dyn FnMut() + Send>`, accepted by Rust) would be refused.
Requirement passed to the threads group; **cost**: none.

### 5.4 The box's cost claim

The report: "The box is one allocation per variable instance and one pointer load per access". Measured
on the callback workload, `total` in a box against by pointer (`bench/runs/w4_attack.txt`): 23.066 and
22.236 ms, 1.04x; the report had 0.95x. Both are under 15%: the claim "nothing measurable here" holds.
Not measured: a hot loop in the *enclosing* function writing a boxed variable while also storing to
other memory, where each access is a load and store through the box (as with C's environment struct or
Rust's `Rc<Cell<i64>>`, which pay the same); argued on par with them, UNVERIFIED.

### 5.5 What differs from Python, stated as differences

A `for` loop variable is a new variable on each pass (the report's rule 4), so closures made in a loop
see their own pass's value; Python's closures all see the last value. The report argues it from Python's
own FAQ and Go 1.22. It is a deliberate difference from Python, and the owner's to accept.

### 5.6 Verdict: saved

Captures as in Python, with boxed variables visible to effect summaries (5.1), the exact meaning of
"copied" and "written after" (5.2), and closures' own boxes moving with them to threads (5.3).

---

## 6. A stored closure is an object (A32): attacks and saves

### 6.1 A closure replaced while it runs, held in a variable (memory safety)

The report keeps `[CLO-10]`'s inline state "for closures in locals and parameters, which no call can
remove from under themselves". Under A12's own rules that is false for a local that another closure
writes: the local lives in a box, and calling any closure that writes it replaces the value.

```ember
fn main():
    name = "first".to_string()
    on_tick = fn(): println("noop")
    fn reset():
        on_tick = fn(): println("second")    # writes on_tick: on_tick lives in a box
    on_tick = fn():
        reset()                               # replaces, and drops, the closure that is running
        println(name)                         # then uses its own captured state
    on_tick()
```

```python
def main():
    name = "first"
    on_tick = lambda: print("noop")
    def reset():
        nonlocal on_tick
        on_tick = lambda: print("second")
    def tick():
        reset()
        print(name)
    on_tick = tick
    on_tick()                                 # prints first   (closure_replaced_while_running)
```

The running closure's state (`name`, and `reset`'s box: more than three words, so one heap block owned
by the closure value, `[CLO-10]`) is dropped when `reset` stores the new value; the body then reads
`name`. `attacks.exe 10`: `AddressSanitizer: heap-use-after-free ... READ of size 8`. Rule 3 (the body
retains its own box) does not help: rule 1 gave this closure no counted box.

**The fix.** Rule 1 becomes: a closure keeps its state in a counted box whenever the place that holds it
may be overwritten or dropped by a call during the closure's own run: a list element, a field, **or a
variable that some closure writes**. Inline state stays for locals no closure writes and for
parameters. Rule 3 then applies: `reset()` may release a closure stored in `on_tick`'s box, the body
uses its state after it, so the body retains its box across the call. `attacks.exe 11`: prints `first`,
as Python. **Cost**: a box only for closures held in such variables, which no measured program has.

### 6.2 The body's retain, when it happens on every call (speed)

The report measured its answer on a handler that calls code it cannot see only on its rare path (one
event in 100,000). A handler that logs through a stored callback, then updates its state, is common:

```ember
fn handler(d: Dispatcher, k: int, v: int) -> int:
    if v >= 0:
        d.log(k)                      # a stored callback: may do anything, so may remove this handler
        counter += v * m              # the state is used after it: the body retains its box (rule 3)
        total += 1
        ...
```

```python
def handler(d, k, v):
    nonlocal counter, total
    if v >= 0:
        d.log(k)
        counter += v * m
        total += 1
        ...
```

I added that one line to the report's callback workload in C (`bench/w4log_plain.c`, plain C with the
same slab and the same `v >= 0` test as the Python program), Ember (`bench/w4log_ember.c`, the report's
`w4_ember_point.c` with the retain around `d.log(k)` on every event; `-DNORETAIN` without it), and
safe Rust (`bench/w4log_rust.rs`). The log closure is chosen at run time from two identical functions so
no compiler can see through the call. Two batches (`bench/runs/w4_attack.txt`, `w4_attack2.txt`),
median ms and ratio to plain C:

| Program | batch 1 | batch 2 |
|---|---|---|
| plain C, same slab | 24.936 (1.000) | 25.082 (1.000) |
| **Ember as the report's rules emit it** (retain and release around `d.log`) | 28.010 (1.123) | 27.241 (1.086) |
| Ember without the retain (`-DNORETAIN`) | 24.553 (0.985) | 23.939 (0.954) |
| safe Rust (take the handler out, call, put it back) | 36.518 (1.464) | 35.856 (1.430) |

The retain and release cost 14.1% and 13.8% against the same program without them: one count increment
and one decrement-and-test per event, on a five-instruction handler. Ember stays 0.77x and 0.76x of
safe Rust's time. Against plain C it is under 15%, so the final measurer decides; but it is a cost C
does not pay, and the job asks to try to remove it.

**The save.** The retain is needed only if `d.log(k)` may release this closure's box. The ownership
group's rule 4 already gives a call through a function value "the effects its type declares, and all
effects if it declares none". Add one fact to field effects: for each field or list of callables, the
closure types stored into it (each closure has its own type, ODR-278, so each has a summary), recorded
in the package interface like any other summary. When every closure that can reach `Dispatcher.log` is
known (the field is private, or the program's own build stores them all), the call has the union of
their effects. Here `log` writes only `logged`, so it cannot release a handler, and the body emits no
retain: the `-DNORETAIN` row, 0.985x and 0.954x of plain C. When a closure from an unknown package can
be stored there, the retain stays (1.09x to 1.12x, still faster than Rust). A hot reload that widens
that union under compiled callers is refused, as the ownership group's attacker fixed for summaries
(`phase3b-ownership-attack.md` 2.4).

### 6.3 Hot reload and the "no protection" path

The report's 5.4 calls a stored closure in place, with no protection (0.905x plain C), when the
compiler sees every closure stored in the list and none of them may change it. A hot reload that adds a
closure with a wider summary, or edits one to remove handlers, makes every such call site wrong, running
or not, because the choice is in the compiled code and in the list's element layout. **The fix**: a
reload that would turn that choice is refused, naming the field (the same comparison the ownership
group's attacker added for summaries). **Cost**: none when running.

### 6.4 The `for` dispatch and the containers group's loop

The report asks the containers group to let unseen code remove the element a `for` loop is on (its 5.5)
by keeping the loop's position in a per-thread record. Two gaps:

- A handler that re-enters the dispatch starts a second loop over the same list; one record per thread
  cannot hold both positions. The extension must apply only when the walkers count is 1 and the record
  names this list; otherwise rule 9's panic stays.
- After the first such removal the loop has a gap (positions between `w` and `r`); a later handler that
  reads `len(self.handlers)` or indexes the list sees the gap, filled with removed elements by rule 10,
  where Python sees a compact list.

A simpler extension: when unseen code removes the element at the loop's recorded position (walkers count
1), the removal closes up the list at once (Python's own memory move) and moves the recorded position
back by one, so the next pass visits the element that followed (Python skips it, a known Python bug the
containers group chose not to copy). Unseen code always sees a compact list. Cost: a store of the
position per pass in loops that call unseen code, and the memory move Python makes. Argued, not
measured; it is the containers group's mechanism to accept. Until then, a handler that removes itself
from inside `for h in self.handlers` panics where Python runs it: a shortfall, not a design choice.

### 6.5 The headline numbers, re-measured

The report's first table calls `w4_plain.c` (one `malloc` per environment, 23.6 ms) "plain C, fastest
layout" and gives the answer 1.00x against it, while the same table's "laid out as the Ember program
is" row (the slab) is faster, 0.92. Against the slab, which is both the fastest and the same layout,
the report's own text gives 1.08x. Re-measured (`bench/runs/w4_attack.txt`, `w4_attack2.txt`):

| Program | batch 1 | batch 2 |
|---|---|---|
| plain C, same slab (`w4_plain.c -DSLAB`) | 22.187 (1.000) | 21.963 (1.000) |
| plain C, `malloc` (`w4_plain.c`) | 23.941 (1.079) | |
| **the answer** (`w4_ember_point.c`) | 22.236 (1.002) | 22.660 (1.032) |
| the answer with `total` boxed (`-DBOXED`) | 23.066 (1.040) | |
| safe Rust (`w4_rust.rs`) | 32.170 (1.450) | |

The claim "on par with plain C" holds (1.00x to 1.08x across three batches, all under 15%); the table
should name the slab as the baseline.

### 6.6 Verdict: saved

A stored closure is an object, with: a counted box for any closure whose holder a call may overwrite,
including a variable another closure writes (6.1); the body's retain dropped when the field's closures
are known (6.2); reloads that would undo the in-place path refused (6.3). The `for` dispatch with
self-removal waits on the containers group (6.4).

---

## 7. Dynamic types (A13): attacks and saves

### 7.1 Narrowing across a hidden write (memory safety)

The report: narrowing of a place lasts "until a write to it or a call whose effect summary may write
it". A back link is written by `push` on a *different* field, and a `weak` field by the death of a
different object. Neither is a write in the source.

```ember
open class Node:
    name: String
    parent: Option[Node] = backlink(children)
    children: Array[Node] = []

class Folder(Node):
    icon: String

if isinstance(n.parent, Folder):
    other.children.push(n)             # rule 2 sets n.parent = other (a plain Node)
    println(n.parent.icon)             # as written: still narrowed to Folder
```

```python
if isinstance(n.parent, Folder):
    reparent(n, other)
    print(n.parent.icon)               # AttributeError   (narrowed_parent_changes)
```

`attacks.exe 6`: `AddressSanitizer: heap-buffer-overflow ... READ of size 8`: `icon` is read past the
end of the plain `Node`. Python stops with a clean error; Ember, as written, reads the wrong memory.

**The fix.** Section 3.3's summaries (hidden writes of back links and of `weak` fields are writes of
those fields), and narrowing of a place ends at any write of that field in *any* object of the class
(two handles can name one object, so `other.shape = Square()` ends the narrowing of `self.shape` when
`other` may be `self`; the report's "places by field effects" says this, its "a write to it" should
too). `attacks.exe 7`: the narrowing ends at the push; the re-test says `n.parent is other, not a
Folder`. **Cost**: none.

### 7.2 The subclass test without the whole program (speed)

The report: a test against an `open` class is "a range compare ... which needs the class tree of the
whole program at link time; otherwise today's walk up the base chain". Ember compiles packages
separately and reloads code while running (`[HR-1]`), so the fallback, a loop, would be the common
case. A test that needs no numbering: each class's type record holds its depth and a fixed-size array of
its ancestors by depth; `isinstance(x, C)` is "depth(x) >= depth(C) and ancestors(x)[depth(C)] is C":
a few loads (the header's type index, that type's record, one ancestor entry) and two compares, with
no loop. The HotSpot Java virtual machine checks class (not interface) subtypes this
way, with an array of primary supertypes indexed by depth (`_primary_supers` in its `Klass`, described in
the OpenJDK mailing list thread on JDK-8180450,
https://mail.openjdk.org/pipermail/hotspot-dev/2024-March/085895.html). A class deeper than the array
falls back to the walk. Not measured; the cost is argued from the instructions. Adding a class at reload
adds a record and changes no other class's.

### 7.3 Python habits the rule should accept

```python
def area(s):
    if not isinstance(s, Circle):
        return 0.0
    return 3.14159 * s.r * s.r          # narrowed after the early return (as mypy does; UNVERIFIED here)
```

```ember
fn area(s: Shape) -> float:
    if not isinstance(s, Circle):
        return 0.0
    return 3.14159 * s.r * s.r
```

The report narrows "in the branch where it is true". Narrowing should follow the flow: after an early
`return`, `continue`, `break` or `raise`, and after `assert isinstance(s, Circle)` (a panic if false).
`isinstance(s, (Circle, Square))` narrows to their nearest common base. **Cost**: none.
(`py_attack_output.txt`, `early_exit_isinstance`.)

### 7.4 Verdict for narrowing: saved

### 7.5 Generic methods through `dyn`: the implementation's body must allow it

The report's rule looks at the interface: a method whose type parameter `V` "appears only as a
parameter's type, passed by reference or lent". The table entry is built "from the generic body
instantiated with `V = dyn Visitor`", but that body is each *implementation's*, and an implementation
can use `V` in ways a `dyn` type cannot satisfy:

```ember
class Pair implements Inspect:
    a: int
    b: int
    fn inspect[V: Visitor](self, mut v: V):
        seen: Array[V] = []             # an Array of V needs V's size: fine for Sum, impossible for dyn Visitor
        v.visit(self.a)
        v.visit(self.b)
```

```python
class Pair:
    def inspect(self, v):               # duck typed: any visitor, always
        v.visit(self.a); v.visit(self.b)
```

**The fix.** An implementation of such a method (and every generic function its body calls with `V`) is
checked for using `V` only through references; a body that needs `V` by value or as a type argument
of another generic type cannot be instantiated with `dyn`, and converting that type to `dyn Inspect` is
an error naming the line. Types never used through `dyn` are not affected. This is Rust's `?Sized`,
inferred instead of written. **Cost**: none.

**Verdict: saved.** The pool table and place-cast the group adopted are **kept** as the references
group's attacker saved them (`phase3b-references-attack.md`, section 0: compare a type id, never the
method-table address; erasure keeps the limit; the table never hands out its slots; slot numbers from
the compiler).

---

## 8. Generics (A16)

### 8.1 Bounds inferred from the body: saved

A public generic function whose body calls a method of an interface the package does not export gets a
bound nobody outside can implement; the function silently accepts only the package's own types:

```ember
interface Weigh:                      # not `pub`
    fn weight(self) -> int

pub fn heaviest[T](xs: Span[T]) -> int:    # inferred bound: Weigh, which callers cannot see
    best = 0
    for x in xs:
        best = max(best, x.weight())
    return best
```

```python
def heaviest(xs): return max(x.weight() for x in xs)    # any object with .weight()
```

**The fix.** An inferred bound of a `pub` function that names an interface other packages cannot see is
an error asking for it to be exported or written (Rust refuses a private trait in a public interface).
Interface methods have no body, so their bounds stay written. **Cost**: none. No other problem found:
each instance is the same code as with written bounds; a hot reload that changes a body's bounds
re-instantiates and re-checks it when the package is rebuilt.

### 8.2 A program nobody depends on may implement anything: kept

Attacked: a library's code cannot see the program's implementations, so it cannot be compiled with one
(a library that needs `Vec3: Hash` must find it in its own view, and then the program's is a duplicate,
E2041); generic library code instantiated by the program is monomorphised with the program's one
implementation. Without overlap (8.3), an extra implementation cannot change which code runs, only let
more code compile. No problem found.

### 8.3 The most specific implementation wins: open

The report already marks it unproven. One more counterexample, from separate compilation:

```ember
# package lib
pub interface Key:
    fn key(self) -> String
extend[T: Debug] T implements Key:
    fn key(self) -> String: return f"{self:?}"
pub fn index(xs: Array[i32]) -> Map[String, i32]:       # not generic: compiled in lib, with the general one
    m = Map()
    for x in xs:
        m[x.key()] = x
    return m

# the program
extend i32 implements Key:                               # allowed by 8.2, chosen for i32 by 8.3
    fn key(self) -> String: return f"int {self}"
m = lib.index([1, 2])
println(m.get(1.key()))                                  # None: lib stored "1", the program asks for "int 1"
```

```python
from functools import singledispatch
@singledispatch
def key(x): return repr(x)
@key.register
def _(x: int): return f"int {x}"
# Python dispatches when called: lib's calls and the program's both see the int version
```

Two implementations of `Key` for `i32` run in one program, the incoherence `[TYP-20]` exists to prevent
(the report discarded package-private implementations for the same reason). A candidate fix: the
interface files record which implementation each compiled non-generic use chose, and a more specific
implementation that would override one already used elsewhere is E2041. It stays **open**, behind the
soundness proof the report asks for.

---

## 9. The report's "found in passing" defect, checked

The report says today's compiler accepts dispatch by index (`h = self.handlers.get_mut(i).unwrap();
h(self)`) and that the emitted C holds a pointer into the list's buffer across the call, then copies the
closure's state back into it. Read in `graphs-callbacks/probes/a32_emit_uaf/c/release/c/main.c`: line
233 takes `&(...)_9.v.ptr)[_12]` into the buffer, line 251 calls the closure through it, and the
trampoline (lines 131 to 137) copies the state into a local, calls the body, and copies it back to the
same address (`memcpy(_env, &_l, sizeof _l)`). The claim stands as the report states it (by reading;
not run).

---

## 10. Good parts of what was replaced

Nothing was discarded whole. Pieces replaced by fixes, and what was good in each:

| Replaced piece | From | Good part kept |
|---|---|---|
| rule 5 of back links as a flat refusal | the report, A8 | the reason for it: assigning a link must never *mean* a move, or Python's linked-list insert loses nodes |
| "lent as a plain `mut Array`" as the only door rule 6 closes | the report, A8 | that a back-linked list may still be read and walked freely |
| inline closure state "for locals and parameters" | `[CLO-10]`, the report's A32 rule 1 | inline state for locals no closure writes: no allocation for the common local lambda |
| the body's retain for every call through an unknown function | the report's A32 rule 3 | it is the only per-call cost and only on calls that may release; with known field effects it is gone |
| the range-numbered subclass test | the report's A13 | one compare for a final class |
| "a write to it" ends narrowing | the report's A13 | narrowing of places at all (mypy's habit) |

---

## 11. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/graphs-callbacks-attack/`.

- `c/attacks.c` (modes 1 to 11), `c/attacks_output.txt`, `c/attacks.exe` with its sanitizer DLL.
- `python/py_attack.py`, `python/py_attack_output.txt`; `python/w4log.py`, `python/w4log_output.txt`.
- `bench/build.sh` (builds every timed program into `bench/out/`, the report's from their own sources);
  `bench/w2_backlink_parts.c`; `bench/w4log_plain.c`, `bench/w4log_ember.c` (`-DNORETAIN`),
  `bench/w4log_rust.rs`.
- `bench/runs/w2_attack.txt`, `w2_attack2.txt`, `w2_attack3.txt`, `w2_attack4.txt`, `w4_attack.txt`,
  `w4_attack2.txt`, each with a `*_summary.txt` made by
  `python -I ../../graphs-callbacks/bench/medians.py <file> <baselines>`.
- Timing command (PowerShell): `powershell -NoProfile -ExecutionPolicy Bypass -File
  C:/Users/ism19/Code/ember/target/redesign2/graphs-callbacks/bench/run.ps1 -Out <file> -Exes <a.exe,b.exe,...>`.

---

## 12. The final answer list for the graphs, callbacks and dynamic types group

| Id | Annoyance | Final answer | Kind | Cost when running (source) | What is left | Verdict |
|---|---|---|---|---|---|---|
| A8 | trees with parent pointers, linked lists | **back links**: `parent: Option[Node] = backlink(children)`; a put into `children` moves the node from its old parent and sets the link; a removal or the parent's death clears it; an indexed put of a node already in the list panics and the swap line compiles to `swap()`; no mutable view of the list; the hidden writes are in effect summaries; a link write the same block makes true is accepted and removed | (c) | the link's own work 0.96x to 1.05x of plain C laid out the same way; with the 8-byte header 1.00x to 1.29x; 0.68x to 0.82x of safe Rust's index arena (`bench/runs/w2_attack*.txt`; PROVISIONAL) | a cycle of owning links leaks (as in Rust); a child of a dead parent sees `None`; the header's cost is the ownership group's question (a 4-byte header for classes with no type index measured 0.96x and 1.02x) | saved |
| A8 | other back references (observers, a camera's target) | **`weak` fields cleared by registration**: a store registers the field with its target; the target's death clears it; the holder's death unregisters it; a weakly held class is not shared with unscoped threads (`SyncWeak` for that) | (a) | reads: a load; stores and deaths: a table lookup (the report's `w2_zweak.c`: 1.21x back links on the tree, 1.70x its memory) | the holder-death cost is not measured | saved |
| A8 | graphs | lists of `weak` edges kept in step both ways | (c) | argued as C's, not measured | rules for many-to-many links not written | open |
| A12 | closures capturing mutable state | **captures work as in Python**: copied (only `Copy`, never written after by anything), moved, pointed to, or boxed; a boxed variable is a heap place in effect summaries; a closure's own box moves with it to a thread | (a) with (b) | none measurable: boxed `total` 1.04x of by pointer (`bench/runs/w4_attack.txt`) | a removal by unseen code during a loop over a boxed list panics (containers' rule 9); loop variables are per pass, unlike Python | saved |
| A32 | a callback receiving its holder; a handler removing itself | **a stored closure is an object**: state in a counted box whenever its holder may be overwritten during the call (list element, field, or a variable some closure writes); the call copies two words out; the body retains its box only across calls that may release it, which known field effects rule out | (b) | 1.00x and 1.03x plain C; Rust 1.45x (`w4_attack.txt`, `w4_attack2.txt`); with a log call per event 1.09x and 1.12x, or 0.95x and 0.99x when the log's closures are known; Rust 1.43x and 1.46x | `for h in handlers` with self-removal waits on the containers group (section 6.4) | saved |
| A13 | trait objects and downcasting | **`isinstance` narrows**, flow-sensitive (early exit, `assert`), ended by any write of the field in any object, hidden writes included; subclass test by an ancestor array indexed by depth; the references group's pool table and place-cast as saved by its attacker | (a) | one load and compare for a final class; a few loads and two compares otherwise, no loop (argued, not measured) | none found | saved |
| A13 | generic methods through `dyn` | **a `dyn` table entry instantiated with `V = dyn I`**, for implementations whose bodies use `V` only through references; otherwise an error at the conversion to `dyn` | (a) | the indirect calls Rust's hand-written workaround makes | a type parameter returned or stored by value stays E2050 | saved |
| A16 | generics friction: bounds | **bounds inferred from the body**, in the package interface; a bound on an interface others cannot see is refused | (a) and (b) | none | a library body change can change what callers may pass | saved |
| A16 | generics friction: orphan rule | **a program nobody depends on may implement any interface for any type** | (b) | none | a dependency adding the same implementation breaks the program at upgrade | kept |
| A16 | generics friction: overlap | the most specific implementation wins | (b) | none | unproven; with separate compilation two implementations can run for one type (section 8.3) | open |
