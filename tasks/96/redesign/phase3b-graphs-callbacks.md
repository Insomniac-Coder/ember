# Phase 3b: graphs, callbacks and dynamic types (A8, A12, A32, A13, A16)

Written 2026-10-24 by the graphs-and-callbacks agent, under the owner's job (`JOB.md`): speed on par
with C and Rust, memory safety the same as Rust's or better, and Python's ease, all at once, with the
checking done when the program is compiled.

How every claim here can be traced:

- **Ember today** is real `ember check` output (`target/ci/ember.exe`, Ember 0.1.0, language 0.9.10).
  Every probe and its output are in `target/redesign2/graphs-callbacks/probes/`
  (`ember_check_output.txt`, made by `run_checks.sh` there). For two probes I also read the C that
  today's compiler writes, with `ember build --emit c --profile release --out-dir c` (C files only; no C
  compiler ran on them and nothing was run): `probes/a32_emit/`, `probes/a32_emit_uaf/`,
  `probes/a13_emit/`. No Ember program was run.
- **Rust** is rustc 1.98.1. My probes and their output: `target/redesign2/graphs-callbacks/rust/`
  (`rustc_output.txt`). Older Rust facts come from phase 1, which names its files
  (`artifacts/phase1/sharing/rustc_output.txt`, `artifacts/phase1/borrowing/rustc_output.txt`).
- **Python** behaviour quoted here was run with Python 3.14.2: `python -I py_checks.py` in
  `target/redesign2/graphs-callbacks/python/` prints `all Python checks passed`
  (`py_checks_output.txt`). The callback workload written in Python (`python/w4.py`) prints
  34946993000, the checksum every C and Rust model of it prints.
- **Timings** are mine, made under the job's rules, and all are **PROVISIONAL**: other agents were
  running at the same time; spreads reached 22% on the tree workload. A decision that rests on a
  difference under 15% waits for the final measurer. Section 2 gives the method.
- Nothing in `compiler/`, `std/`, `docs/` or `tests/` was touched.

---

## 0. The short version

Last night's three designs replaced compile-time checks with run-time checks and came out slower than
C. Every answer chosen here is decided when the program is compiled; the run-time work left is the work
a C programmer writes for the same job, and it was measured against that C.

| Id | Annoyance | Chosen answer | Kind | Cost (provisional) | Meets all three goals? |
|---|---|---|---|---|---|
| A8 | trees with parent pointers, graphs, cycles | **back links**: `parent: Option[Node] = backlink(children)`. The language keeps both ends of the link in step: a push into `children` takes the node out of its old parent and sets `parent`; a parent that dies clears its children's `parent`. Reading `parent` is a plain pointer load. | (c) new mechanism | tree workload: 1.10x plain C laid out the same way (1.04x to 1.14x over three runs), 1.14x C's fastest layout, 0.87x safe Rust's index arena, 0.23x safe Rust's `Rc`/`Weak`, 0.24x today's Ember | **Yes for trees, lists and other one-owner links.** Left: a cycle of *owning* links written Python's way still leaks, as in Rust (Python's collector frees it); the debug build names it. Non-structural weak references cost a table lookup per store (`weak`, 3.3). |
| A12 | closures capturing mutable state | **captures work as in Python**: a closure sees and changes the variable itself. The compiler stores each captured variable in the cheapest safe place: copied, moved, pointed to (when it proves the closure dies first), or in a small counted box (when it does not). No `owned fn`, no E3026. | (a) Pythonic spelling, with (b) proofs | callback workload: the box costs nothing measurable (0.95x against the pointer version); the whole program 1.00x plain C | **Yes.** Left: a closure crossing threads may not capture a boxed variable (as Rust's `Rc` may not cross). |
| A32 | a callback receiving the object that holds it; a handler removing itself | **a stored closure is an object, protected where it is used**: a closure stored in a list or field keeps its state in a counted box; calling it copies {function, box} out; the closure's body keeps its own box alive only across a call that may remove it (the ownership group's rule). The handler may add, remove or replace handlers, itself included, as in Python. | (b) smarter compile time | callback workload: **1.00x plain C**, 0.74x safe Rust (Rust's take-and-put-back was 1.36x plain C) | **Yes.** Left: inside a `for` loop over the handler list, a handler removing itself needs one extension of the containers group's loop rule (section 5.5); with an index loop it works as in Python. |
| A13 | trait objects and downcasting | the references group's answers (a downcast of a place is a place; a pool table keyed by the type) plus two here: **`isinstance(s, Circle)` narrows `s`** inside the block, compiled to one compare of the header's type index; and **a generic method may be called through `dyn`**: the compiler makes the table entry that takes the bound as `dyn`, which is Rust's hand-written workaround | (a) | narrowing: one load and one compare (today: a call that walks the base chain); `dyn` generic methods: the same code as Rust's workaround | **Yes.** Left: a generic method whose type parameter is returned or stored by value stays refused (E2050), as in Rust. |
| A16 | generics friction | **bounds inferred from the body** (recorded in the package interface, printed on request); **a program nobody depends on may implement any interface for any type**; **the most specific implementation wins** where two overlap (Python's `singledispatch`) | (a) and (b) | none at run time: each instance compiles to the same code as with written bounds | **Yes for the first two.** The third rests on a soundness argument not yet proved for Ember (UNVERIFIED). |

**Measured** (section 2, PROVISIONAL), median of 5 pinned runs, ratio to plain C in the same batch:

| | tree with parent pointers (phase 3's W2) | callbacks that remove themselves (phase 3's W4) |
|---|---|---|
| plain C, fastest layout | 1.00 (532.0 ms) | 1.00 (23.6 ms) |
| plain C, laid out as the Ember program is (one object per node or closure) | 1.04 | 0.92 |
| **this report's answer** | **1.14** (back links) | **1.00** (stored closure as an object) |
| safe Rust, the shared-object way (`Rc`, `Weak`, `RefCell`; `Box<dyn FnMut>`, `Rc<Cell>`) | 5.01 | 1.36 |
| safe Rust, the index arena | 1.31 | n/a |
| Ember today, as the spec lays it out | 4.71 | refused (and its one accepted spelling is unsound, section 10) |

Every program printed the same checksum on every run (647034814 for the tree, 34946993000 for the
callbacks).

**Found in passing** (section 10): today's Ember accepts a handler list dispatched by index
(`h = self.handlers.get_mut(i).unwrap(); h(self)`) and the C it emits holds a pointer into the list's
buffer across the call, with no access check, then copies the closure's state back into that pointer
after the call: a write into freed memory when the handler adds to the list, and a doubly owned
`String` when it removes itself (by reading the emitted C; not run).

---

## 1. Words used in this report

- **Handle**: what a variable of a `class` type holds: a pointer to an object on the heap.
- **Count**: the number of handles that own an object; the object is freed when it reaches zero
  (reference counting). Rust's `Rc` and Ember's classes both work this way.
- **Header**: the words at the start of every object that hold its count and its type. The ownership
  group proposes an 8-byte header, a 32-bit count and a 32-bit type index (`phase3b-ownership.md` 2.3,
  "the lean class"); today's is 24 bytes (`[OBJ-1]`).
- **Owning link**: a field or list element whose handle counts (`children: Array[Node]`). A
  **non-owning link** does not count (a parent pointer must not, or parent and child keep each other
  alive forever: a **cycle**, which counting never frees).
- **Back link**: this report's proposal for A8: a non-owning field that the language keeps equal to
  "the object whose owning field holds me" (section 3.3).
- **Weak reference**: a non-owning link that reads as "nothing" once its object has died. Rust's
  `Weak`, Ember's `Weak[T]`, Python's `weakref`.
- **Dangling**: a pointer to memory that was freed. Using one is the use-after-free that Rust's rules
  exist to prevent.
- **Slab**: an allocator that hands out same-size blocks from a large chunk, in order, then reuses
  freed blocks; Python's own object allocator works this way. **malloc**: the C library allocator.
- **Closure**: a function value that uses variables of the function that made it (its **captures**).
  A **stored closure** is one kept in a list or a field to be called later (a callback, a handler).
- **Box** (in this report): a small counted heap block holding captured state.
- **Effect summary**, **field effects**, **reshape**: from the ownership group (`phase3b-ownership.md`
  1 and 2.1). The compiler records, from each function's body, which fields it may *reshape*: change in
  a way that can free or move memory (push to a list, remove from it, replace it, drop an object). A
  pointer is never kept across a call that may reshape what it points into.
- **Loan**: a pointer the compiled program holds into a place for a while (a `for` loop holds one into
  the list it walks).
- **Panic**: a deterministic stop with a message (`[PAN-1]`); never a read of the wrong memory.
- **Interface table** (vtable): the table of function pointers a `dyn I` value or a class handle uses
  for calls whose target is chosen at run time. **Downcast**: asking whether a value typed by an
  interface or a base class is really of a given type.
- **Bound**: what a generic function requires of its type parameter (`T: Add`). **Monomorphise**:
  compile a separate copy of a generic function for each type it is used with (Ember does, `[TYP-16]`).
- **Orphan rule**, **coherence**: the rule that an implementation of an interface for a type may only
  be written in the package of the interface or of the type, so that two packages cannot give one type
  two different implementations (`[TYP-20]`; Rust's E0117).
- **Same layout**: plain C that does the same work in the same shape as the Ember program (one heap
  object per node, a list of pointers to them, Ember's list layout), so that a ratio measures the memory
  model and not a different data structure.

The three designs of last night are named by what they did: **"copy instead of share"**
(`phase3-design-D1.md`), **"checked pointers"** (`phase3-design-D2.md`), and **"today's rules, but never
an error"** (`phase3-design-D3.md`).

---

## 2. How the costs were measured

- **Machine**: the owner's laptop, Intel Core Ultra 9 275HX, on mains power:
  `(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 before every batch, and the runner checks it
  before every round.
- **Compilers**: clang 22.1.8, `"C:/Program Files/LLVM/bin/clang.exe" -O2 -I target/redesign/bench -I ..
  <file>.c -o out/<name>.exe` (with the `-D` switches named below); rustc 1.98.1,
  `rustc -C opt-level=2 --edition 2021 <file>.rs -o out/<name>.exe`.
- **Harness**: the shared `target/redesign/bench/harness.h` (it calls `SetErrorMode`, so no dialog can
  open, and prints time, peak working set and a checksum); its Rust twin `bench/harness.rs` (copied
  from the ownership group's; the same random sequence and output line). The pieces of Ember's runtime
  the models use are written out in `bench/ember_model.h` (the list layout `{ptr, len, cap}` with
  64-bit length and capacity and first growth to 4, from `runtime/ember_rt/src/ember_rt.c`
  `vec_reserve_at`; bounds checks; the slab).
- **Runner**: `bench/run.ps1`: 5 rounds; in each round every program runs once, one at a time, pinned
  to the performance cores from its start (`cmd /c start "" /b /wait /affinity C03C03 <exe>`); the
  order is forward in odd rounds and reversed in even rounds, so A and B alternate. **Medians**:
  `python -I bench/medians.py <results> <baseline>`.
- **Batches**: tree workload `bench/runs/w2.txt`, `w2_parts.txt`, `w2_final.txt` (the tables use
  `w2_final.txt`; the other two are the consistency check); callback workload `bench/runs/w4.txt`,
  `w4_b2.txt`, `w4_final.txt` (the tables use `w4_final.txt`). Summaries beside them (`*_summary.txt`).
- **Checksums**: every run of every program printed 647034814 (tree) or 34946993000 (callbacks).
- **Left out of every model and every baseline**, as in phase 3: integer overflow checks and
  zero-divisor checks (they are not memory-model costs; Rust's release build has none either).
- **The workloads** are phase 3's (`phase3-measurements.md`): the tree (1,000,000 nodes, each under a
  random earlier node; 10 passes, each walking every node up to the root and moving 1% of nodes to a
  new random parent) and the callbacks (10,000 handlers, 10,000,000 events; each handler adds to its
  own counter and to a shared total; every 100,000th event the handler removes itself and registers a
  replacement).

---

## 3. A8: trees with parent pointers, graphs, cycles

### 3.1 What Rust's rule protects against

Phase 1 (`phase1-borrowing.md` A8, checked with rustc there): a back pointer must not dangle when its
target is freed, and a cycle of owners must not leak or be freed twice. Rust's direct spelling (a
`&Node` parent) is refused three times (E0502, E0505, E0515); the workarounds are `Rc` with
`RefCell<Weak<Node>>` parents, or an arena of nodes linked by index. A `Weak` that is read after its
object died returns `None`; an `Rc` cycle leaks (memory, not safety: Rust counts a leak as safe).

### 3.2 The Python program, and what Ember does with it today

```python
class Node:
    def __init__(self, name):
        self.name = name
        self.parent = None
        self.children = []

    def add(self, child):
        child.parent = self
        self.children.append(child)

def reparent(node, new_parent):
    node.parent.children.remove(node)
    node.parent = new_parent
    new_parent.children.append(node)
```

Python keeps a parent alive while a child refers to it (`py_checks_output.txt`,
`root_kept_alive_by_child_parent`), and its cycle collector frees the parent-child cycle
(`cycle_freed_by_gc`). Forgetting the first line of `reparent` leaves the node in two child lists
(`reparent_forgot_remove_in_both`), a bug Python programmers write.

Ember today, four probes:

- The Python spelling, `parent: Option[Node] = None` (`probes/a8_tree_python.em`): **accepted**, with a
  warning whose advice names the wrong edge:

  ```text
  warning[L3001]: potential reference cycle: Node.children -> Node
   --> a8_tree_python.em:4:5
    = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
  ```

  `children` is the owning edge; making it weak would free every child. With a strong `parent` every
  parent-child pair is a cycle that leaks (`[WK-1]`).
- Today's working spelling (`probes/a8_tree_weak_today.em`, the spec's Part VIII example):
  `parent: Weak[Node] = Weak.empty()`, `child.parent = Weak(self)`, `match n.parent.upgrade():`. It
  compiles, with the same misleading L3001.
- The re-parent in today's spelling (`probes/a8_reparent_python.em`) has eight lines of body against
  Python's three: upgrade, match, search, remove, a new `Weak(...)`, push.
- A doubly linked list (`probes/a8_dlist.em`) and a graph of cities with roads both ways
  (`probes/a8_graph_cycle.em`) compile with L3001 and leak (by `[WK-1]`; not run).
- A graph in today's `Arena` (`probes/a8_arena_graph.em`) is refused: `E3090: City needs drop and
  cannot be allocated with Arena.alloc` (an `Array` field needs drop).
- A plain struct tree with `parent: Option[ref Node]` crashes the checker (phase 1 defect 1).

### 3.3 Answers

#### (c) Back links: the language keeps both ends of a link in step (chosen)

```python
class Node:
    def __init__(self, name):
        self.name = name
        self.parent = None
        self.children = []

    def add(self, child):
        child.parent = self
        self.children.append(child)

def reparent(node, new_parent):
    node.parent.children.remove(node)
    node.parent = new_parent
    new_parent.children.append(node)
```

```ember
class Node:
    name: String
    parent: Option[Node] = backlink(children)
    children: Array[Node] = []

    fn add(self, child: Node):
        self.children.push(child)            # also sets child.parent

fn reparent(node: Node, new_parent: Node):
    new_parent.children.push(node)           # takes node out of its old parent's children first
```

`probes/a8_tree_backlink.em` is this program; today it is `E1010: cannot find backlink in this scope`.
The spelling is a proposal (SQLAlchemy calls the same idea `back_populates`; its documentation says a
change on one side of such a relationship affects the other,
https://docs.sqlalchemy.org/en/20/orm/backref.html; Python's own `xml.dom.minidom.appendChild` moves a
node that already has a parent, `py_checks_output.txt`, `dom_appendChild_moves`).

**The rules.**

1. A field declared `f: Option[C] = backlink(g)` in class `D` pairs with an owning field `g` of class
   `C` whose type is `Array[D]` (one parent, many children) or `Option[D]` (one to one: `next` and
   `prev`). It does not count: it is not an owning edge in the ownership graph (`[WK-5]`), so it closes
   no cycle and gets no L3001.
2. Every operation that puts an object into `g` (push, insert, assigning an element, assigning the
   whole field) first takes the object out of its current holder's `g` (search, then the
   order-keeping remove, as Python's `list.remove`), then sets its back link to the new holder. The
   handle moves from one list to the other: no count changes.
3. Every operation that takes an object out of `g` (remove, pop, clear, replacing the whole field)
   sets its back link to `None`.
4. When the holder dies, the drop of `g` walks it anyway (to release each element); it first sets each
   element's back link to `None`. A child that something else still holds survives with no parent.
5. The back link cannot be assigned (`node.parent = p` is a compile error whose fix-it is
   `p.children.push(node)`). Allowing it would make Python's four-line linked-list insert silently
   mean something else once the first line had moved the node.
6. A field named by a back link is changed only through its own operations: it cannot be lent as a
   plain `mut Array[Node]` to a function that knows nothing of the link (it is still readable as a
   list). This is what keeps rules 2 and 3 complete.
7. Reading `n.parent` gives `Option[Node]`: a load, and the `None` test the program writes anyway.

**Why it is memory-safe.** The invariant: if `x.parent` is `p`, then `x` is an element of
`p.children` and `p` has not been freed. It holds because every way `x` can leave `p.children` is an
operation of rule 2 or 3, rule 6 closes the other doors, and `p` cannot be freed while it holds
`x` without running rule 4 first. `p.children` holds a count of `x`, so `x` cannot be freed while it is
there. No back link can therefore point at freed memory, and nothing is counted, checked or tagged to
make it so.

Attacks I tried:

| Attack | Program | Result |
|---|---|---|
| the parent dies while a child is held elsewhere | `leaf = tree.find("x"); tree = None; print(leaf.parent)` | `None` (rule 4). Python prints the old parent, which its child kept alive; Rust's `Weak` and Swift's `weak` give `None` too |
| re-parent without removing first (Python's bug) | `new.children.push(node)` while `node` is in `old.children` | taken out of `old.children` first (rule 2): never in two lists |
| a library function changes the list behind the link's back | `shuffle_remove(mut xs: Array[Node])` called with `p.children` | refused (rule 6); passing it read-only is allowed |
| the whole list is moved out | `kids = p.children; p.children = []` | the old list moves into `kids` (names follow values, `phase3b-ownership.md` 2.2) and its nodes' `parent` becomes `None` (rule 3): safe; Python would leave stale parents |
| a loop over `old.children` that moves each child to `new` | `for c in old.children: new.children.push(c)` | `push` reshapes `Node.children` of some node (field effects), which may be `old.children`: the containers group's loop rule applies (their rule 9, `phase3b-containers.md` 4.3) or it is a compile-time error with the fix-it `new.children.extend(old.children.drain())` |
| a cycle through the owning edge | `a.children.push(b); b.children.push(a)` | the first push makes `b` a child of `a`; the second takes `a` out of its own parent (if any) and makes it a child of `b`. Each now owns the other: a cycle of owners, which leaks as any strong cycle does (memory, not safety; Rust's `Rc` leaks the same way), and L3001 reports it. A check that the new parent is not a descendant (a walk up its back links) would refuse it at run time; not proposed by default (Python allows it too) |
| the old D3 breaker's line-order trap (`phase3-break-D3.md` 2.1) | `root.children.push(c); c.parent = Some(root)` | the second line is refused (rule 5); ownership never depends on which line came first |
| a doubly linked list insert | `new.next = a.next; a.next = new` | rule 2 moves `c` from `a.next` into `new.next` (`c.prev = new`), then `new` into `a.next` (`new.prev = a`): `a, new, c` |

**What the compiler emits** for the walk and the re-parent (`bench/w2/w2_backlink.c`, written by me as
the model of what the compiler would emit):

```c
for (Node *a = n->parent; a; a = a->parent) depth++;          /* the walk: plain C's own loop */

static inline void children_push(Node *holder, Node *c) {      /* new_parent.children.push(node) */
    Node *old = c->parent;
    if (old) evec_remove_at(&old->children, evec_index_of(&old->children, c));
    evec_push(&holder->children, c);
    c->parent = holder;
}
```

This is the code a C programmer writes for a re-parent (`plain/w2.c` in phase 3 does the same search,
remove, push and parent store), with no count change and no check.

**Cost, measured** (`runs/w2_final.txt`; summary `runs/w2_final_summary.txt`):

| Program | What it is | Median ms | Peak KB | Time vs plain C fastest | Time vs plain C same layout |
|---|---|---|---|---|---|
| `w2_plain_keep.c` | plain C, all nodes in one array, order-keeping remove | 532.038 | 54,944 | 1.000 | 0.965 |
| `w2_plain_swap.c` | phase 3's baseline (swap-remove) | 549.184 | 54,936 | 1.032 | 0.996 |
| `w2_plain_malloc.c` | plain C, one `malloc` per node, a list of node pointers | 801.414 | 79,012 | 1.506 | 1.453 |
| `w2_plain_malloc.c -DSLAB` | plain C, nodes from a slab, a list of node pointers | 535.728 | 62,900 | 1.007 | 0.971 |
| `w2_plain_malloc.c -DSLAB -DARRAY24` | the same with Ember's list layout: **plain C, same layout** | 551.472 | 76,652 | 1.037 | **1.000** |
| `w2_backlink.c -DSLAB` | **back links**, lean class (8-byte header), nodes from a slab | 608.154 | 84,544 | **1.143** | **1.103** |
| `w2_backlink.c -DSLAB -DNOHEADER` | the same without the header (not safe for shared objects; to take the cost apart) | 594.433 | 76,692 | 1.117 | 1.078 |
| `w2_backlink.c` | back links, nodes from `malloc` | 790.815 | 100,256 | 1.486 | 1.434 (0.987 of plain C with `malloc`) |
| `w2_weak16.c -DSLAB` | a `weak` parent read by a count test, 16-byte header (3.3 below) | 877.377 | 92,392 | 1.649 | 1.591 |
| `w2_zweak.c -DSLAB` | a `weak` parent cleared by registration (3.3 below) | 738.924 | 143,312 | 1.389 | 1.340 |
| `w2_today.c` | Ember today as the spec lays it out: 24-byte header, access words, `Weak.upgrade()` with a retain and release per step | 2,505.127 | 115,964 | 4.709 | 4.543 |
| `w2_rust_rc.rs` | safe Rust, `Rc<Node>`, `RefCell<Weak<Node>>` parent | 2,666.502 | 116,320 | 5.012 | 4.835 |
| `w2_rust_index.rs` | safe Rust, all nodes in a `Vec`, links as `u32` indices | 695.453 | 52,336 | 1.307 | 1.261 |

How to read it:

- **The back link itself costs nothing on the walk**: the loop is the same instructions as plain C's.
  Against plain C in the same layout, the model is 1.103 here, 1.143 in the first batch
  (`runs/w2.txt`) and 1.035 in the second (`runs/w2_parts.txt`): under 15% each time, so the final
  measurer decides. Without the header it is 1.078 (here) and 1.032; the header costs about 2% of time
  here and 10% of memory (8 bytes per node: 84,544 against 76,692 KB).
- **The layout costs more than the memory model.** One `malloc` per node makes plain C itself 1.50x
  slower than plain C with a slab (801.4 against 535.7 ms), and the back-link model 1.30x slower (790.8
  against 608.2 ms). A runtime that allocates class objects from size-class slabs, as Python's own
  allocator does, is a requirement this group passes on (section 8). Ember's list header (24 bytes
  against C's 16) costs 2.9% more (551.5 against 535.7 ms).
- **Against safe Rust**: 0.23x the time of the `Rc`/`Weak` program Rust's errors lead to, and 0.87x the
  index arena Rust programmers are told to write. The arena's walk computes an address from an index on
  every step: in its assembly (`bench/w2/out/w2_rust_index.s`, loop `.LBB4_57`) each step is a shift, a
  load, a widening move and a bounds compare, where a pointer walk is one load. That this is why it is
  1.31x plain C is UNVERIFIED (no hardware counters read).
- **Against today's Ember**: 0.24x (4.1x faster), from the smaller object and no count or upgrade per
  step.
- The salvage report took last night's designs to 1.24x ("copy instead of share", with a `weak` test
  per step), 1.24x ("checked pointers", packed references) and 1.07x ("today's rules, but never an
  error", raw pointers that need the whole-program fact that no node ever dies; 1.95x without it)
  against its own same-layout plain C (`phase3b-salvage.md` section 0). Back links need no
  whole-program fact.

#### (a) A Pythonic `weak` field (kept, for links that are not part of a structure)

Not every back reference has an owning list to pair with: a camera that watches a player, a cache, an
observer list. For those, a `weak` field, read without ceremony:

```python
import weakref
class Camera:
    def __init__(self, target): self._target = weakref.ref(target)
    @property
    def target(self): return self._target()      # None once the player is gone
```

```ember
class Camera:
    target: weak Player                           # reads as Option[Player]; None once the player is gone
```

Today `weak` is a parse error (`E0100: expected ':'`, the ownership group's `probes/a10_weak_field.em`;
mine, `probes/a8_graph_links.em`: `E1010: cannot find type weak`). Two implementations were measured,
on the tree workload, against back links:

- **A count test on every read** (`w2_weak16.c`): a weak reference is a plain pointer; an object whose
  count reaches 0 while weak references remain is kept as an empty block until they are gone (today's
  `[OBJ-3]`); each read tests the target's count. With the weak count in a 16-byte header it is 1.44x
  back links (1.43x and 1.38x in the other batches). The salvage report isolated the test alone at +18%
  (`phase3b-salvage.md`, its "copy instead of share" W2 row, `-DD1_PROVEN`). It also needs a weak count,
  which the ownership group's 8-byte header has no room for.
- **Clearing by registration** (`w2_zweak.c`, Objective-C's zeroing weak references): storing into a
  weak field registers the field's address with the target, in a side record found through a table
  keyed by the object's address; an object that dies clears every registered field, then is freed at
  once. One bit of the count word says "has a side record", so other objects pay nothing. Reads are a
  plain load. On the tree workload: 1.21x back links in time and **1.70x in memory** (143,312 against
  84,544 KB): the registration lists duplicate the child lists. A field's address must not move, so
  `weak` is allowed in class fields and in weak collections (`Array[weak T]`, which registers its
  elements again when its buffer moves), not in a struct value (fix-it: make the holder a class).

**Choice for `weak`**: clearing by registration. Reads cost nothing, there is no weak count in the
header (the ownership group's request, `phase3b-ownership.md` section 5), and a dead object's memory is
returned at once. Its cost is on stores: a table lookup per store into a weak field and per death of a
weakly referenced object. For a tree, back links are better in time and memory, so the compiler's
L3001 fix-it should offer `backlink(...)` when the cycle is a parent-child pair, and `weak` otherwise.

#### (c) Graphs whose edges both ways are kept in step (argued, not measured)

For a real graph (cities and roads), the nodes are owned by one list and the edges must not own. The
same mechanism as back links, for many-to-many: the edges are `weak`, and a second field lists who
points here, kept in step by the language, so a city that dies is taken out of every list that names it
and no side table is needed.

```python
class City:
    def __init__(self, name): self.name = name; self.roads = []
a, b = City("a"), City("b")
a.roads.append(b); b.roads.append(a)              # Python's collector frees the cycle
```

```ember
class City:
    name: String
    roads: Array[weak City] = []
    into: Array[weak City] = backlink(roads)       # who has a road to me

cities = [City("a"), City("b")]                   # the owner of every city
cities[0].roads.push(cities[1])
cities[1].roads.push(cities[0])
```

Walking `roads` is a pointer load per edge; adding an edge is two pushes; removing a city removes it
from its neighbours' lists (a search in each). That is the C code for an undirected graph that can
delete nodes, so the cost is argued as C's; not measured. Today: `probes/a8_graph_links.em` is
`E1010: cannot find type weak`.

#### (b) A region that owns a whole graph (considered, kept as a library)

A graph allocated in one region whose nodes point at each other freely, freed all at once (Rust's
typed arenas; phase 1 A8(b)). Zero cost on every access, but no node can be freed before the region,
and today's `Arena` refuses any node type with a list (`E3090`, `probes/a8_arena_graph.em`). It
needs the references group's limits carried by a value's own fields (a node holding views limited to
its region), which their report leaves open. Kept as a library idea for build-once graphs (meshes,
parse trees); not the answer for Python programs, which delete nodes.

#### A collector for cycles (considered, not chosen)

Python, Nim's ORC and CPython free unreachable cycles with a collector that runs only on types that
can form cycles (`phase2-rc-elision.md` idea 4). It costs a registration on every count decrement of
such a type and extra header bytes; it was not measured here (no Nim on this machine; phase 2 found no
C baseline for it). With back links, trees, lists and parent-child pairs are no longer cycles at all,
so the collector would only serve owning cycles. Not chosen; the debug build's leak report
(`[WK-15]`) stays, with L3001's advice fixed (section 10).

### 3.4 Choice for A8

**Back links** for every link that is the reverse of an owning field (trees, doubly linked lists,
owner and owned), with **`weak`** (cleared by registration) for the rest, and back-linked `weak` lists
for graphs. Reading any of them is a plain load; the write-side work is what C does. What it lacks
against Python: a cycle of *owning* links still leaks, as in Rust, where Python's collector frees it;
and a child held after its parent died sees `None`, where Python keeps the parent alive.

---

## 4. A12: closures capturing mutable state

### 4.1 What Rust's rule protects against

Phase 1 (`phase1-sharing.md` A12): a stored closure writing a variable whose function has returned
(a write to a dead stack slot). rustc refuses `b.on_click(|| score += 1)` with E0373 and E0503; its
help (`move`) compiles and returns 0, because the closure then bumps its own copy; the working spelling
is `Rc<Cell<i32>>` plus a clone moved into each closure.

### 4.2 The Python program, and what Ember does with it today

```python
def game():
    score = 0
    def on_click():
        nonlocal score
        score += 1
    b = Button(); b.on_click(on_click); b.click()
    return score                                  # 1
```

```ember
fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    return score
```

Today (`probes/a12_button_python.em`, where `Button` stores the handler in `handlers: Array[fn()]` and
`click` runs `for h in self.handlers: h()`), three errors:

```text
error[E2020]: expected `fn() -> void`, found `Callable0`                 (6:28, inside Button.on_click)
error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`  (6:28)
  = help: write it `owned fn`, which moves or copies each capture into the value
error[E3021]: cannot write through a shared reference                    (10:13, `h()` in click)
  = help: ... iterate a collection of them with `.iter_mut()`
```

With `owned f` on the parameter (`probes/a12_button_owned.em`) the error is still E3026, reported at
the `push` inside `on_click`, not at the line that wrote the lambda. Following the help (`owned fn`)
compiles and, by `[CLO-2]`, bumps a copy: `game()` returns 0 with no warning (phase 1,
`a12_closures_owned.em`; from the spec, not run). The other everyday shapes, all checked:

| Program (Python's result) | Ember today |
|---|---|
| handlers in a local list that dies before `score` (`probes/a12_local_list.em`; Python 1) | E3026, "this place has no bounding region" |
| a counter factory, `make_counter()` (`a12_counter_factory.em`; Python 2) | E3026 at `return inc` |
| two closures sharing one counter (`a12_two_closures.em`; Python 2) | E3026 twice |
| an escaping closure reading a variable written after it was made (`a12_read_later.em`; Python 2) | E3026 |
| closures made in a loop, each returning its loop variable (`a12_loop_capture.em`; Python `[2, 2, 2]`) | compiles; each closure gets its own copy (`[CLO-13]`): `0 + 1 + 2` (from the spec, not run) |

### 4.3 Answers

#### (a) with (b): captures work as in Python; the compiler picks where each captured variable lives (chosen)

The Ember programs above are written exactly as shown (no `owned`, no `mut`, no cell type). The rules:

1. A closure captures variables, not values: it sees later writes to them and its writes are seen
   (Python's `nonlocal` meaning, without the word).
2. Where each captured variable lives is decided when compiling, per variable. The choice never
   changes what the program does, only what it costs:
   - read by the closure and never written after the closure is made: **copied** into it (today's
     `[CLO-13]`);
   - used after the closure is made only by that closure: **moved** into it;
   - otherwise, if the compiler proves every closure that captures it dies before the variable's
     scope ends (it is stored only in locals, local lists, or objects that carry the variable's
     limit: the references group's 4.3 and 4.4): a **pointer** to the variable;
   - otherwise: the variable lives in a **box** shared by the function and every closure that captures
     it, freed with the last of them. This is what Python's cells, Swift's boxed captures and C#'s
     hoisted locals do (`phase2-swift-mojo.md` A12 row, `phase2-hylo-csharp.md` A12 row).
3. A local function used by a closure contributes its own captures, not itself (a "flat" closure: the
   handler that calls `make_handler` captures `total` once, not `make_handler`'s state as well). This
   is the merge the "today's rules, but never an error" breaker found its callback model assumed without
   saying (`phase3-break-D3.md` 3.4).
4. A `for` loop variable is a new variable on each pass, so a closure made in a loop keeps that pass's
   value. This keeps today's `[CLO-13]` result and differs from Python on purpose: Python's FAQ lists
   the opposite as a trap (its entry on lambdas made in a loop that all return the same result,
   https://docs.python.org/3/faq/programming.html), and Go gave `for` loop variables per-pass scope in
   Go 1.22 (https://go.dev/blog/loopvar-preview, 2023-09-19).
5. A closure that is stored in a list or a field keeps its state in a box (A32's rule, section 5.3).
6. A closure that captures a box is not sent to another thread (the box's count is not atomic): the
   threads group's rule (`[THR-8]` today).

**Why it is memory-safe.** Copies and moves are owned by the closure. A pointer is used only where the
compiler proved that the closure dies first (the references group's limit rules, which are
compile-time). A box is counted, so it lives as long as the function or any closure still uses it.

Attacks I tried:

| Attack | Program | Result |
|---|---|---|
| an escaping closure reads a variable written later (`phase3-break-D1.md` B8) | `n = 1; f = fn() => n; n = 2; return f` | `n` is written after capture and the closure escapes: a box; `f()` is 2, as Python |
| two closures share a counter, and a debug print is added (`phase3-break-D2.md` N2) | `make_pair()` returning `inc`, `get`; then `println(count)` after making them | both capture one box; the print changes nothing (the choice of place never changes behaviour; at worst a move becomes a box) |
| a stored closure clears the list it lives in while running (`phase3-break-D1.md` B2) | `hs.push(make()); hs[0]()` where the closure does `hs.clear(); n += 1` | the closure's state is a box (rule 5); its body uses the box after `hs.clear()`, a call that may release it, so the body holds it across that call (section 5.3); `n += 1` writes live memory |
| a closure captures a list element's name (`phase3-break-D3.md` 2.5) | `boss = enemies[0]; button.on_click(fn(): boss.hp -= 10)` | a class element: the handle is captured (Python's meaning); a struct element: `boss` names a place that a later push may move, so capturing it in an escaping closure is a compile-time error with the fix-its "capture `enemies` and the index" or "make `Enemy` a class" |
| a closure stored in the object it captures | `self.button.on_click(fn(): self.count += 1)` | the box holds a counted handle to `self`, and `self` owns the button: an owning cycle that leaks, as in Swift and Rust; L3001 should report it. Python's collector frees it. The leak-free spelling passes the object as an argument instead of capturing it (section 5) |
| a closure sent to a thread | `thread.spawn(fn(): total += 1)` | refused (rule 6), with the fix-its `Atomic` or a channel |

**Cost.** Copies, moves and pointers cost nothing (the same code C writes, or less). The box is one
allocation per variable instance and one pointer load per access; plain C's callback environment and
Rust's `Rc<Cell<i64>>` pay the same. Measured on the callback workload (`runs/w4_final.txt`), the shared
`total` as a pointer (`w4_ember_point.c`) against in a box (`-DBOXED`): 23.542 and 22.473 ms, a
difference under 15% and in the box's favour: nothing measurable.

#### (b) Callbacks that cannot outlive what they borrow (kept as an option)

Phase 1 A12(b): scoped registration (`with b.on_click(...):`) or handlers that receive the state as an
argument (`b.on_click(fn(g: Game): g.score += 1)`). Zero cost and no box; both remain available, and
the second is how a handler avoids an owning cycle with its owner. Not chosen as *the* answer because
Python's habit is to capture.

#### Discarded: `owned fn` with a warning

Keep today's rules and add Rust's warning when an `owned fn` writes a copy nobody reads. It removes the
silent wrong answer but keeps the annoyance (E3026, `owned`, and a box written by hand as a class).

### 4.4 Choice for A12

Captures work as in Python, and the compiler chooses copy, move, pointer or box. Nothing is written,
nothing is refused except a box crossing threads, and the measured cost of the box is nil on the
callback workload.

---

## 5. A32: a callback receiving the object that holds it; a handler removing itself

### 5.1 What Rust's rule protects against

Phase 1 (`phase1-sharing.md` A32): a handler pushing to `handlers` while the loop holds a pointer into
the list (the push moves the list's buffer and frees the old one, which the loop and the running
closure live in), or a handler removing or replacing itself and freeing its own captured state while it
runs. rustc refuses `for h in self.handlers.iter_mut() { h(self) }` with E0499; the workaround is
`std::mem::take` of the whole list (handlers added during the loop are lost unless merged by hand) or
taking each closure out of its slot and putting it back.

### 5.2 The Python program, and what Ember does with it today

```python
class Ui:
    def __init__(self): self.handlers = []; self.clicks = 0
    def fire(self):
        for h in self.handlers:            # a handler may append; the loop then calls it too
            h(self)
```

```ember
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers:
            h(self)
```

Python calls a handler added during the dispatch in the same dispatch
(`handler_added_during_dispatch_runs_same_dispatch`: 2 clicks), lets a handler remove itself (and then
skips the next one: `remove_self_skips_next_in_same_dispatch`), and lets a handler re-enter the
dispatch (`reentrant_handler`).

Ember today:

- `probes/a32_fire_python.em`: `E3021: cannot write through a shared reference` at `h(self)` (a
  stored callable must be called from a mutable place, `[CLO-2]`).
- `probes/a32_fire_python_itermut.em` follows that help (`.iter_mut()`): `E3080: self.handlers cannot be
  read while it is mutably borrowed` at `h(self)`. The struct version (`probes/a32_struct.em`) is E3021.
- `probes/a32_fire_index.em` and `probes/a32_remove_self.em`, dispatching by index
  (`h = self.handlers.get_mut(i).unwrap(); h(self)`): **accepted**, and unsound (section 10).

### 5.3 Answers

All the safe answers below were measured on the callback workload, the program a Python programmer
writes (`python/w4.py`, which prints the same checksum):

```python
class Dispatcher:
    def __init__(self): self.handlers = []; self.current = 0
    def register(self, h): self.handlers.append(h)
    def unregister_current(self): del self.handlers[self.current]
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
```

```ember
class Dispatcher:
    handlers: Array[fn(Dispatcher, int, int) -> int] = []
    current: int = 0
    fn register(self, h: fn(Dispatcher, int, int) -> int): self.handlers.push(h)
    fn unregister_current(self): self.handlers.remove(self.current)
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
    ...
```

#### (b) A stored closure is an object, protected where it is used (chosen)

1. A closure stored in a list element or a field keeps its captured state in a box: a small counted
   object (a count and the captures), from the slab for its size. The element is
   `{function, box}`, 16 bytes; a closure without captures has no box. This is plain C's `env` and
   Rust's `Box<dyn FnMut>`. (`[CLO-10]`'s three inline words stay for closures in locals and
   parameters, which no call can remove from under themselves.)
2. Calling a stored closure copies `{function, box}` out of its slot and calls. The caller uses nothing
   of it after the call, so it protects nothing: no count, no mark, no copy back.
3. The closure's body is compiled like a method of its box, and the ownership group's rule applies to
   the box as to any object: a handle used after a call that may release it is retained across that
   call (`phase3b-ownership.md` 2.1 and 2.3). `d.unregister_current()` may remove a closure stored in
   `Dispatcher.handlers` (its effect summary says so; a call through an unknown function may remove
   anything), and the handler uses its box after it (`counter`, `total`), so the body retains its own
   box across that call and releases it after; if the removal dropped the last other count, the box is
   freed there.
4. Re-entering a running closure (a handler that dispatches to itself) is allowed, as in Python: both
   calls use one box, and the general rules protect it like any object's fields.

This is the salvage report's "protect at the point of use" (`phase3b-salvage.md`, its W4 rows; its
model `target/redesign2/salvage/d3/w4.c`), written here with a count instead of a pin, and with A12's
capture rules in front of it.

```c
/* bench/w4/w4_ember_point.c: what the compiler emits */
Callable c = d->hs.ptr[i];                        /* copied out; not used after the call */
c.fn(c.st, d, k, k % 1000);

static int64_t handler(State *st, Dispatcher *d, int64_t k, int64_t v) {
    if (v >= 0) {
        st->counter += v * st->m;
        *st->total += 1;
        if (k % 100000 == 99999) {
            st->rc++;                             /* used after a call that may release it */
            unregister_current(d);
            register_handler(d, make_handler(k % 17 + 1, st->total));
            int64_t c = st->counter;
            state_release(st);
            return c;
        }
    }
    return st->counter;
}
```

**Why it is memory-safe.** The running closure's state is in a box that never moves (only
`{function, box}` moves when the list grows or shifts), and the box is freed only when its count reaches
zero. The caller holds no pointer into the list during the call. The body holds a count on its box
across every call that may release it, and the effect summaries over-approximate (unknown code may do
anything), so no path frees the box while the body will still use it. Attacks:

| Attack | Result |
|---|---|
| a handler appends handlers until the list's buffer moves | the list moves `{function, box}` pairs; the running closure's state is not in the list |
| a handler removes itself, then reads and writes its state | its body retained the box across the removal: the state is live until the body releases it |
| a handler clears or replaces the whole list | the same: every box dropped with the list is freed unless counted elsewhere; the running one is held by its body |
| a handler re-enters itself through the dispatcher | allowed (Python allows it); a loop over a list in its own state across that call follows the containers group's loop rules |
| a handler that removes itself and does not use its state afterwards | the body retains nothing; the removal frees the box at once; nothing reads it again |
| the caller calls the same stored closure twice through a local name | `h = self.handlers[i]; h(self); h(self)`: `h` is used after a call that may remove it, so the same rule retains the box across the first call (a count pair, only in this shape) |
| today's unsound shape (section 10) written this way | the caller copies two words out and keeps no pointer into the buffer, so there is nothing to write back |

**Cost, measured** (`runs/w4_final.txt`; summary `runs/w4_final_summary.txt`):

| Program | What it is | Median ms | Peak KB | vs plain C | vs safe Rust |
|---|---|---|---|---|---|
| `w4_plain.c` | phase 3's plain C: `malloc`'d environments; the handler frees its own as its last act | 23.592 | 4,076 | 1.000 | 0.738 |
| `w4_plain.c -DSLAB` | the same with environments from a slab | 21.803 | 4,080 | 0.924 | 0.682 |
| `w4_ember_point.c` | **the answer** (state boxes from a slab; `total` by pointer) | **23.542** | 4,260 | **0.998** | **0.736** |
| `w4_ember_point.c -DBOXED` | the answer with `total` in a box (A12's fallback) | 22.473 | 4,260 | 0.953 | 0.703 |
| `w4_rust.rs` | safe Rust: `Vec<Option<Box<dyn FnMut>>>`, `Rc<Cell<i64>>`; take the closure out, call, put it back | 31.975 | 4,476 | 1.355 | 1.000 |

Against plain C with the same slab it is 1.08 (point) and 1.03 (boxed): under 15%, for the final
measurer. Memory is 4.5% above plain C (each box carries a count: 32 bytes against plain C's 24-byte
environment). The salvage report measured its version at 0.96x plain C (`phase3b-salvage.md` section 0).

#### Other mechanisms tried, and what each cost (all safe, all measured in the same batch)

- **Lend the closure out of its slot** (phase 1 A32(a); Rust's take-and-put-back done by the compiler):
  the closure, with three inline words of state, is moved to the caller's frame for the call and a
  stub left in its slot; it is moved back if the stub is still there, searched for if the list
  shifted, dropped if the handler removed it; a call that reaches the stub (re-entry) panics.
  First version (`w4_ember.c`): **2.461x**. Saving it: the assembly (`bench/w4/out/w4_ember.s`) showed
  the copy back reading 16 bytes the handler had just written as 8 (a store the processor cannot
  forward to the load, which then waits); copying word by word (`-DWORDCOPY`) took it to **1.455x**.
  A variant that leaves the state in place and redirects today's trampoline write-back through a
  per-thread "running record" that list operations update (`w4_ember_runrec.c`): **1.400x**. Both are
  close to safe Rust (1.355x), whose code is the same idea. Not chosen: they cost about 1 ns per call
  that the chosen answer does not, and they forbid re-entry.
- **A "running" flag in the state box** (`w4_ember_mark.c`): set before every call, cleared after; a
  removal of a running closure defers the free to the caller. **1.306x**: two writes per call, where
  the chosen answer writes only on the handler's rare path.
- **Lending out a closure whose state is already boxed** (`-DWORDCOPY -DHEAPSTATE`): 2.026x; boxing and
  lending both, the worst of both.
- **No protection at all** (`-DNOLEND`, not safe; only to take the cost apart): 0.905x. The inline state
  is that fast when nothing can remove the closure during its call (section 5.4).

#### (b) Changes made during a dispatch are applied after it (considered)

EnTT's signals and phase 1 A32(b): `push` and `remove` on the handler list during a dispatch go to a
pending list merged when the dispatch ends. Fast, but a handler added during the dispatch does not run
in it, which Python's does. Not chosen as the language rule; a library may offer it.

### 5.4 When no protection is needed

When the compiler can see every closure stored in the list (a private field, all registrations in the
package) and none of them may change the list, the call keeps the closure's state inline and calls it
in place: the `-DNOLEND` code, 0.905x plain C, which is then safe. This is field effects applied to
closure types (each closure has its own type, ODR-278, so its effects are known); a field whose
closures come from other packages takes the conservative path above, which already costs nothing
measurable here.

### 5.5 With the containers group's loop

`for h in self.handlers: h(self)` is a loop over a list whose body calls code the compiler cannot see.
The containers group's rule for that case (`phase3b-containers.md` 4.3, rule 9) visits handlers
appended during the loop (Python's behaviour), and **panics** if unseen code removes, reorders or
replaces the list while the loop runs. My rule keeps the running closure's state safe whatever the list
does; their rule decides what the loop does next. So, together, a handler that removes itself from
inside a `for` dispatch panics, where Python runs it (and skips the next handler). With an index loop
(`while i < len(self.handlers)`, the callback workload's shape) it works. I ask the containers group for one extension:
removal, by unseen code, of the element the loop is on means "not kept" (their rule 3) instead of a
panic. The loop can tell, cheaply: in loops whose body calls unseen code it keeps its position in a
per-thread record (one store per pass), and `remove` compares the position it removes with it. Not
measured; it is their mechanism to accept or not.

### 5.6 Choice for A32

**A stored closure is an object, protected where it is used**: 1.00x plain C and 0.74x safe Rust on
the worst case for it (a dispatch of a five-instruction handler), with Python's meaning for adding,
removing and replacing handlers, re-entry included. The `for` loop case needs the extension in 5.5.

---

## 6. A13: trait objects and downcasting

### 6.1 What Rust's rule protects against

Phase 1 (`phase1-sharing.md` A13): reading a value as the wrong type (without the type check on a
downcast, `get_mut[f32]` on the `int` pool would reinterpret its bytes), and a table that would need a
slot for every possible instance of a generic method (`dyn` compatibility, E0038).

### 6.2 The Python program, and what Ember does with it today

```python
pools = {int: Pool(), float: Pool()}
pools[int].dense.append(10)                    # a dict keyed by the type: no cast at all

for s in shapes:
    if isinstance(s, Circle):
        t += s.r
```

Ember today:

- The registry with `Box[dyn AnyPool]` pools and a cast (`probes/a13_box_cast.em`): `E2020: cast … as
  and cast … if need related class handles, found Box[dyn AnyPool] and Pool[T]`.
- A map keyed by a type (`probes/a13_registry_python.em`): `type` is not an expression (E0100, E1010).
- `isinstance` over class objects (`probes/a13_isinstance.em`) compiles in today's spelling, five lines
  for Python's two:

  ```ember
  for s in shapes:
      match cast s if Circle:
          Some(c):
              t += c.r
          None:
              pass
  ```

  It compiles to a call, `ember_downcast(object, &em_ti_6Circle)` (`probes/a13_emit/c/release/c/main.c`
  line 332), which walks the object's base chain (`runtime/ember_rt/src/ember_rt.c` line 3408). Writing
  `if c = cast s if Circle:` gets `E2036: this pattern always matches` with the help "did you mean
  `c == cast s if Circle`?" (`probes/a13_if_cast.em`).
- An interface with a generic method used through `dyn` (`probes/a13_dyn_generic.em`):
  `E2050: interface Inspect is not dyn-compatible … method inspect is generic`. The hand-written
  workaround, passing the visitor as `ref mut dyn Visitor` (`probes/a13_dyn_param.em`), compiles; so
  does its Rust twin (`rust/r13_dyn_param.rs`).

### 6.3 Answers

#### The registry: the references group's answers (adopted, not repeated)

`phase3b-references.md` 3.4: `cast <place> as T` on a `Box[dyn I]` place gives a reference (kind (a));
and a standard-library **pool table keyed by the type**, `self.pools[Pool[T]]`, whose insert puts a
value only in its own type's slot, so reading needs no check (kind (b)). They measured it on 32,000,000
gets: the table 1.208x (inlined) and 1.084x (out of line) plain C without bounds checks, the cast 7%
more, safe Rust's `downcast_mut` 1.710x and 1.606x (their `bench/out/results.txt`). The Python spelling
above is the table's spelling.

#### (a) `isinstance` narrows (chosen)

```python
for s in shapes:
    if isinstance(s, Circle):
        t += s.r
```

```ember
for s in shapes:
    if isinstance(s, Circle):
        t += s.r
```

The rules: `isinstance(x, C)` (or a tuple of classes) is a `bool`; in the branch where it is true, `x`
has type `C`, as Python's type checker mypy narrows a variable after an `isinstance` test (its "Type
narrowing" page, https://mypy.readthedocs.io/en/stable/type_narrowing.html). For a local, narrowing lasts until
the local is assigned; for a place (`self.shape`), until a write to it or a call whose effect summary
may write it. `cast … if` and `cast … as` stay. Today `probes/a13_isinstance_narrow.em` is
`E1010: cannot find isinstance` and `E1010: cannot find field r on Shape`.

**Cost**, argued from what the compiler emits: the lean header holds a 32-bit type index. A test
against a final class is one load and one compare, inline. A test against an open class is a range
compare (two compares) when the program's classes are numbered so each class's descendants follow it,
which needs the class tree of the whole program at link time; otherwise today's walk up the base chain.
A C programmer tests a kind field the same way. Safe Rust needs `as_any()` and `downcast_ref`
(`rust/r13_downcast.rs` compiles), whose call through the table and 16-byte compare the references
group saw in its assembly.

**Safety**: the narrowed type holds only where nothing can have changed the value since the test
(locals by assignment, places by field effects), so a field access through the narrowed type always
reads an object of that class.

#### (a) A generic method may be called through `dyn` (chosen)

```python
class Leaf:
    def inspect(self, v): v.visit(self.n)      # any visitor, duck typed
```

```ember
interface Inspect:
    fn inspect[V: Visitor](self, mut v: V)

fn walk(xs: Array[Box[dyn Inspect]]) -> int:   # today: E2050
    s = Sum(total=0)
    for x in xs:
        x.inspect(s)
    return s.total
```

The rule: when a method's type parameter appears only as a parameter's type, passed by reference or
lent (`mut v: V`), and its bound (`Visitor`) is itself `dyn`-compatible, the interface table gets one
entry for the method with the parameter as `dyn Visitor`; the compiler builds it from the generic body
instantiated with `V = dyn Visitor`. A call through `dyn Inspect` passes `s` as `dyn Visitor`. A call
on a known type stays a direct, monomorphised call. A type parameter that is returned, stored by value
or used in another generic position keeps E2050, naming the use.

**Cost**: through `dyn`, each `v.visit` in the body is an indirect call: the same code as Rust's
hand-written workaround (`rust/r13_dyn_param.rs`), which is what Rust programmers write today. Nothing
changes for static calls.

**Safety**: the entry is an ordinary instance of the generic body; the type checker checks it as it
checks any instance.

#### (b) One table slot per instance the program uses (considered, not chosen)

With the whole program visible, each `dyn Inspect` table could get one slot per `V` actually used
(phase 1 A13(b)): zero indirect calls. It gives up separate compilation of tables and adding instances
by hot reload (`[HR-1]`); the `dyn`-parameter entry above works everywhere and costs what Rust's
workaround costs. Good part kept: as an optimisation the compiler may apply when it sees every use.

#### `Box` written by hand around `dyn`

`shapes: Array[dyn Shape]` without `Box` is the ownership group's (implied `Box`,
`phase3b-ownership.md` 3.3; their probe `a10_dyn_inferred.em` shows today's E2020).

### 6.4 Choice for A13

The references group's pool table and place-cast, plus `isinstance` narrowing and `dyn`-callable
generic methods. All compile-time; the run-time work is one type compare where C and Rust also test a
type, and an indirect call where Rust's workaround makes the same one.

---

## 7. A16: generics friction

### 7.1 What Rust's rule protects against

Phase 1 (`phase1-sharing.md` A16): two packages each implementing `Display for Array[Vec3]`, so that
which one runs depends on link order (incoherence); and a generic body using an operation some type
lacks, found only when someone uses it with that type, far from the cause.

### 7.2 The Python program, and what Ember does with it today

```python
def sum_all(xs):
    acc = xs[0]
    for x in xs[1:]: acc = acc + x
    return acc

@singledispatch
def describe(x): return repr(x)
@describe.register
def _(x: int): return f"int {x}"
```

Python finds a missing `+` when it runs (`duck_sum_error_at_run_time`: "unsupported operand type(s) for
+: 'object' and 'object'") and picks the most specific registered type (`singledispatch_most_specific`:
`describe(True)` is `"int True"`, since `bool` is a kind of `int`).

Ember today:

- `probes/a16_bounds.em` (no bounds): `E2040: T's bounds do not provide a matching Add operator`. With
  bounds written (`a16_bounds_written.em`) it compiles; a type without `+` then fails at the call with
  `E2040: Name does not implement Add[Name], which T requires` (`a16_bound_misuse.em`).
- `probes/a16_orphan.em`, a one-file **program** giving `Array[Vec3]` the `Neg` interface:
  `E2121: Array[Vec3] and Neg are both another package's`. Rust refuses the same even in a binary
  crate that nothing can depend on (`rust/r16_orphan_bin.rs`: E0117).
- `probes/a16_overlap.em`, a general implementation plus a special one for `i32`:
  `E2041: i32 already implements Describe`.

### 7.3 Answers

#### (a) Bounds inferred from the body (chosen)

```python
def sum_all(xs):
    acc = xs[0]
    for x in xs[1:]: acc = acc + x
    return acc
```

```ember
fn sum[T](xs: Span[T]) -> T:            # bounds inferred: Add[Output = T] + Copy
    acc = xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc
```

The rules: every operation the body performs on `T` is mapped to the one interface in scope that
provides it (operators to their interfaces, `[TYP-21]`; a method name to the interface declaring it);
the bounds are the set of those interfaces. A method name that no interface in scope declares is an
error in the body, at its line (interfaces are nominal, `[TYP-40]`, so this keeps errors local); one
that two interfaces declare asks for the bound to be written. Written bounds stay allowed and are
checked. Inferred bounds go into the package interface, as the ownership group's effect summaries do,
and the publishing tool lists a changed bound as an interface change. `ember inspect --bounds` prints
them. A use with a type that lacks one is today's E2040 at the call, naming the line that needs it.

**Cost**: none at run time; each instance compiles to the same code as with written bounds. **Safety**:
unchanged: the body is checked against its bounds, every instance against the bounds. **Gives up**: a
library body change can change what its callers may pass (the same trade the ownership group accepted
for effect summaries, and Rust accepts for inferred `Send`).

#### (b) A program nobody depends on may implement any interface for any type (chosen)

A package whose kind is `bin` (or a test) cannot be depended on, so no other package can see its
implementations: the orphan rule's reason (two packages writing the same implementation) cannot arise.
It may implement any interface for any type; two implementations of one interface for one type
anywhere in the program are still E2041. **Cost**: none. **Gives up**: moving such code into a library
later means moving the implementation too; a future version of a dependency that adds the same
implementation breaks the program at upgrade (the program's author, not a third party, fixes it).

#### (b) The most specific implementation wins (proposed; soundness UNVERIFIED)

```python
@singledispatch
def describe(x): return repr(x)
@describe.register
def _(x: int): return f"int {x}"
```

```ember
extend[T: Debug] T implements Describe:
    fn describe(self) -> String: return f"{self:?}"

extend i32 implements Describe:                # today E2041; proposed: chosen for i32
    fn describe(self) -> String: return f"int {self}"
```

The rule: two implementations may overlap when one is strictly more specific (its types are an
instance of the other's and its bounds imply the other's); each instance uses the most specific one
that applies; overlap with no most specific one stays E2041; a more specific implementation may not
change associated types. **Cost**: none (the choice is made when monomorphising). **Gives up**: adding
an implementation in a dependency can change which one runs (Python's `singledispatch` has the same
property; the build tool should report it). **Safety: UNVERIFIED.** Rust's specialisation is unstable
because of soundness problems usually attributed to lifetimes in implementation selection (phase 1
A16(b), with the Rust project's 2026 goal page); Ember's regions are inferred and are not part of types,
so that cause may not arise, but nobody has proved it for Ember's rules. It should not be adopted
before that proof.

#### (c) Implementations visible only inside one package (discarded)

Scoped implementations would let a library add an interface to another's type privately. Two such
packages could give one key type two `Hash` implementations and pass a `Map` between them, corrupting
it (the reason `[TYP-20]` exists). Discarded; the counterexample is the shared map.

### 7.4 Choice for A16

Inferred bounds and the program exemption from the orphan rule now; most-specific-wins after a
soundness proof. None of them costs anything when the program runs.

---

## 8. How the answers fit together, and what they need from the other groups

**Within this group.**

- A12 and A32 are one rule: a closure is a function plus captured state; the state lives where the
  compiler can show it is safe (inline, a pointer, or a box), and a stored closure's box is an object
  like any other, protected by the same retain-where-used rule.
- A8's back links and A32's stored closures meet in the observer pattern: an observer list is
  `Array[weak Observer]` (A8's `weak`) or a list of handlers that receive the subject as an argument
  (A32), which avoids the owning cycle a captured `self` makes (section 4.3).
- A13's narrowing and A16's inferred bounds both use only the lean header's type index and the
  monomorphiser; nothing runs that C would not run.

**What this group needs from the others.**

| From | What | Used by | If not given |
|---|---|---|---|
| ownership (A1, A9, A10) | the lean class: an 8-byte header with a 32-bit count, one bit of which says "has a side record" (zeroing `weak`) | A8 `weak`, A32 boxes | `weak` needs a weak count in a 16-byte header: 1.44x back links on the tree (measured) |
| ownership | field effects that say "may release an object of class C" and "may remove a callable stored in field F", and the rule that a handle used after such a call is retained across it | A32's protection, A8's reads of `parent` into a local | A32 needs a per-call mark: 1.31x (measured) |
| ownership | class objects allocated from size-class slabs by the runtime | A8 (1.30x with `malloc`, measured), A32 boxes | the tree with back links is 1.49x plain C instead of 1.14x |
| containers (A2, A4) | a loop over a list that unseen code may change (their rule 9), extended so that removing the element the loop is on means "not kept" (5.5) | A32 with `for h in self.handlers` | a handler that removes itself from a `for` dispatch panics |
| containers | a loop over `children` while back links move children (their rule 9 or a compile-time error) | A8 | as stated in 3.3's attack table |
| references (A5, A6) | limits carried by local containers and by objects (their 4.3, 4.4), including class handles | A12's "pointer" place for captures | such captures go to a box: no measurable cost on the callback workload (0.95x) |
| references | the pool table keyed by type and the place-cast | A13 | as their report says |
| threads (A15) | a box (captured variable or stored closure) has a non-atomic count and stays on its thread unless frozen; the zeroing-`weak` side table is per thread | A12, A32, A8 `weak` | not memory-safe across threads |
| error messages (A17) | L3001 offering `backlink(...)` for a parent-child pair and naming the right edge; E3026's replacement errors reported at the caller | A8, A12 | as today |

**What the others asked of this group.** The ownership group asked for "the `weak` field, and its
count kept outside the header": `weak` needs no weak count at all under clearing by registration, only
one bit of the count word, and back links need nothing in the header. They also asked that a closure's
effects be its body's and that a closure behind `fn(A) -> R` have declared or all effects: agreed, and
section 5.4 uses exactly that.

---

## 9. Discarded answers, and the good parts kept

| Answer | From | Why discarded (what no fix removes) | Good part kept |
|---|---|---|---|
| generation-checked parent pointers | "checked pointers" (`phase3-design-D2.md`), Vale | a compare per step: +23% in order, +45% random (`phase2-vale-austral-pony.md` section 6); the salvage report's packed version is still 1.24x on the tree (`phase3b-salvage.md`); back links need no check | a stale use is a clean panic, never a read of freed memory: right for handles into pools (`[HND-*]`) |
| raw parent pointers proved safe by a whole-program fact | "today's rules, but never an error" (`phase3-design-D3.md`) | the fact breaks with one `pop()` anywhere, separate compilation or hot reload (1.95x without it, `phase3b-salvage.md`); its "first store is the home" rule made line order decide ownership (`phase3-break-D3.md` 2.1) | the walk is a plain pointer chase: back links get it with no whole-program fact |
| `weak` read by a count test | today's `Weak`, "copy instead of share", Swift | +18% per walk (salvage), 1.44x back links here; needs a weak count in the header | kept as the reading of `weak` in a program that cannot use registration (a foreign object) |
| inferring which edge is the back edge | phase 1 A8(a) | a wrong guess changes when objects die, silently; `next` and `prev` are symmetric | the L3001 fix-it that proposes `backlink(...)` |
| lending the closure out of its slot | phase 1 A32(a); Rust's take-and-put-back | 1.40x to 1.46x after the stall was fixed (2.46x before); forbids re-entry | the word-by-word copy (any compiler-made copy back after a call should avoid wide loads of just-written words) |
| a running flag per call | my own | 1.31x: two writes per call | the deferred free of a running closure's state |
| a per-thread running record that list operations update | my own | 1.40x; changes every operation of lists of callables | it shows today's trampoline write-back (section 10) can be redirected instead of removed |
| pinning the handler list for every call | "today's rules, but never an error" | ties the state to the list; the salvage report replaced it with the chosen answer (1.03x to 0.96x) | none beyond the salvage's |
| a count pair around every stored-closure call | "copy instead of share" | +11% (salvage, its W4 row) | "the caller uses nothing after the call, so it protects nothing" |
| closures capturing copies (`owned fn`) | Ember today, Rust's `move` | the silent wrong answer (`game()` returns 0) | copies stay the choice for variables never written after capture |
| one table slot per instance for `dyn` generic methods | phase 1 A13(b) | needs the whole program, breaks hot reload | an optimisation when the whole program is visible |
| scoped (package-private) implementations | (my own) | two `Hash` implementations corrupt a shared `Map` | none |
| a region that owns a graph | phase 1 A8(b), Rust arenas | no node freed before the region; needs limits on a value's own fields | for build-once graphs, as a library |

---

## 10. Found in passing: defects in today's compiler (nothing was changed)

1. **Unsound: a handler list dispatched by index writes freed memory.** In a class method,
   `h = self.handlers.get_mut(i).unwrap(); h(self)` passes `ember check` (`probes/a32_fire_index.em`,
   `a32_remove_self.em`, `a32_uaf_today.em`, `a32_remove_self_today.em`), though `for h in
   self.handlers.iter_mut(): h(self)` is E3080. In the emitted C
   (`probes/a32_emit_uaf/c/release/c/main.c`):
   - line 233 takes a pointer into the list's buffer (`_14 = &(...)_9.v.ptr)[_12]`), and line 251 calls
     the closure with that pointer as its state, with no access check (the field's access word
     `_access_handlers` appears only in the struct and the descriptor, lines 71 and 161; the debug
     profile, `probes/a32_emit/c/debug/c/main.c`, has none either);
   - the closure's trampoline (lines 131 to 137) copies the inline state in, runs the body, then
     copies it back to the same address (line 136, `memcpy(_env, &_l, sizeof _l)`).

   When the handler pushes to the list and the buffer moves, that copy back writes 24 bytes into freed
   memory. When the handler removes itself, the removal drops its state (freeing the captured
   `String`'s buffer) and shifts the next handler into its slot; the copy back then overwrites the next
   handler's state with the dropped one, so one `String` buffer is owned twice and freed twice. By
   reading the emitted C; not run. `[EXC-18]` requires a write access on `handlers` for as long as `h`
   lives, which would have turned the push into a panic.
2. **L3001 names the owning edge** for Python's tree spelling as well as for today's `Weak` spelling
   (`probes/a8_tree_python.em`, `a8_tree_weak_today.em`): "if `Node.children` is a back-reference, make
   it hold a Weak handle". Phase 1 found it for the `Weak` case.
3. **The Python tree compiles and leaks.** `parent: Option[Node]` strong gives only the (misleading)
   warning; every parent-child pair is then a cycle (`probes/a8_tree_python.em`).
4. **E3026 is reported inside the callee** (`probes/a12_button_owned.em`: line 6, the push in
   `on_click`, not line 16, where the lambda is written), as phase 1 found; and a lambda passed to a
   plain (not `owned`) callable parameter that is stored is first `E2020: expected fn() -> void, found
   Callable0` (`a12_button_python.em`), a compiler-internal type name.
5. **Python's dispatch loop is refused twice.** `for h in self.handlers: h()` is E3021; its help
   (`.iter_mut()`) leads to E3080 when the handler gets `self` (`probes/a32_fire_python.em`,
   `a32_fire_python_itermut.em`).
6. **A closure stored in a local list that dies first is still E3026** ("this place has no bounding
   region", `probes/a12_local_list.em`): the container-of-views limits are not applied to closures.
7. **`if c = cast s if Circle:`** gets E2036 "this pattern always matches" with the help "did you mean
   `c == cast s if Circle`?", which is not what the writer meant (an `if let`); the fix is
   `match cast s if Circle:` (`probes/a13_if_cast.em`).

---

## 11. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/graphs-callbacks/`.

- `probes/*.em`: the Ember probes named above; `probes/run_checks.sh` runs `ember check` on each and
  writes `probes/ember_check_output.txt`. Emitted C (not compiled, not run): `probes/a32_emit/`,
  `probes/a32_emit_uaf/`, `probes/a13_emit/`.
- `rust/r16_orphan_bin.rs`, `r13_dyn_param.rs`, `r13_downcast.rs`; output `rust/rustc_output.txt`.
- `python/py_checks.py` (output `py_checks_output.txt`); `python/w4.py` (output `w4_output.txt`:
  34946993000).
- `bench/ember_model.h` (Ember's list, bounds checks, slab), `bench/harness.rs`, `bench/run.ps1`,
  `bench/medians.py`.
- Tree workload, `bench/w2/`: `w2_plain_swap.c`, `w2_plain_keep.c`, `w2_plain_malloc.c` (`-DSLAB`,
  `-DARRAY24`), `w2_backlink.c` (`-DSLAB`, `-DNOHEADER`), `w2_weak16.c`, `w2_zweak.c`, `w2_today.c`,
  `w2_rust_rc.rs`, `w2_rust_index.rs`; binaries and `w2_rust_index.s` in `bench/w2/out/`;
  `bench/w2/diag/` holds a phase-timer copy used only to locate the time (not in any table).
- Callback workload, `bench/w4/`: `w4_plain.c` (`-DSLAB`), `w4_ember_point.c` (`-DBOXED`),
  `w4_ember.c` (`-DWORDCOPY`, `-DBOXED`, `-DHEAPSTATE`, `-DNOLEND`), `w4_ember_runrec.c`,
  `w4_ember_mark.c`, `w4_rust.rs`; binaries and `w4_ember.s` in `bench/w4/out/`.
- Raw timings and summaries: `bench/runs/` (`w2.txt`, `w2_parts.txt`, `w2_final.txt`, `w4.txt`,
  `w4_b2.txt`, `w4_final.txt`, and a `*_summary.txt` for each).
