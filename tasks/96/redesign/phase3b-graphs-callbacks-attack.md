# Phase 3b: graphs, callbacks and dynamic types, attacked and saved (second attack)

Written 2026-10-24 by the second adversary of `phase3b-graphs-callbacks.md` ("the report" below), under
the owner's job (`JOB.md`): speed on par with C and Rust, memory safety the same as Rust's or better,
Python's ease, with the checking done when the program is compiled; and before anything counts as
failed, try to save it.

**This replaces the first attack's report**, which is kept unchanged as
`phase3b-graphs-callbacks-attack-first-run.md` (its programs are in
`target/redesign2/graphs-callbacks-attack/`). I did not take it on trust: I re-ran its eleven attack
programs (`attacks.exe 1` to `11`; each reproduces exactly what it reports), re-read the spec lines it
cites, and re-measured the claims it rests on. Its findings are carried into this report, marked
**(first attack)**, with what I checked. Everything not so marked is new here.

How every claim here can be traced (paths under `C:/Users/ism19/Code/ember/target/redesign2/` unless
they say otherwise):

- **Memory-safety attacks**: small C programs that do what the rules would make the compiler emit, each
  with the Ember program it stands for in a comment above it, in `graphs-callbacks-attack2/c/attacks2.c`
  (14 modes: each odd mode is the rule as written, the even mode after it is the fix). Built with
  AddressSanitizer, a clang option that stops a program with a report the moment it reads or writes
  memory it must not:
  `"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address -I ../../../redesign/bench attacks2.c -o attacks2.exe`
  (its runtime DLL copied from `C:/Program Files/LLVM/lib/clang/22/lib/windows/`; nothing downloaded),
  run as `attacks2.exe <1..14>` with `ASAN_OPTIONS=abort_on_error=0`; all output in
  `c/attacks2_output.txt`. The program calls `SetErrorMode` first (the shared `harness.h`), so no
  dialog can open.
- **Python**: `graphs-callbacks-attack2/python/py_attack2.py`, run with `python -I` (Python 3.14.2),
  output `py_attack2_output.txt`; `python/graph.py` (output `graph_output.txt`). Every "Python gives"
  below comes from those files.
- **Ember today**: `graphs-callbacks-attack2/probes/*.em`, checked with
  `C:/Users/ism19/Code/ember/target/ci/ember.exe check <file>` by `probes/run_checks.sh`; output
  `probes/ember_check_output.txt`. No Ember program was built or run.
- **Timings**: mine, under the job's rules. `(Get-CimInstance Win32_Battery).BatteryStatus` was 2
  before every batch and the runner checks it before every round; every run pinned to the performance
  cores from its start (`start /affinity C03C03`); one program at a time; 5 rounds with the order
  reversed in every second round; medians. Runner and median script: the group's `bench/run.ps1` and
  `bench/medians.py`, copied unchanged into `graphs-callbacks-attack2/bench/`. Builds:
  `graphs-callbacks-attack2/bench/build.sh` (clang 22.1.8 `-O2`; rustc 1.98.1
  `-C opt-level=2 --edition 2021`), which builds the group's models from the group's own source files
  (read, not changed) into my `bench/out/`. Raw results and summaries: `graphs-callbacks-attack2/bench/runs/`.
  Other agents were running: the processor load read 0% to 26% before batches and spreads reached 74%
  on the tree workload. **Every timing is PROVISIONAL**; a decision that rests on a difference under
  15% waits for the final measurer.
- Nothing in `compiler/`, `std/`, `docs/` or `tests/` was touched; no git command was run.

---

## 0. The short version

No answer of the group is discarded. Every one that had a hole is saved by a fix that costs nothing on
the paths that were measured, except where this report says what it costs. Six of the holes are new
here; each was a use after free, a write into freed memory, or a read of the wrong type in the C that
the rules as written would emit, caught by AddressSanitizer, and each fix removed it. One save made by
the first attack (section 5.2) and one made here (section 4.2) were attacked again and needed a further
fix.

| Annoyance | The report's answer | Verdict | What changed (first attack; this attack) |
|---|---|---|---|
| A8, trees with parent pointers | **back links** (`parent: Option[Node] = backlink(children)`) | **saved** | (first attack) an indexed put of a node already in the list panics, `swap()` for the swap line, no mutable views, hidden writes in effect summaries, Python's `child.parent = self` accepted when the block makes it true. (Here) a link assigned the value it holds freed that value (`attacks2.exe 7`); the rules now make one membership change in a fixed order, which also turns Python's filter idiom from quadratic (44x to 91x plain C) into linear (1.13x to 1.42x) |
| A8, other back references | **`weak` fields cleared by registration** | **saved**, with a cost left | (first attack) the holder's death unregisters; weakly held classes stay on their thread. (Here) a `weak` list registered slot by slot wrote into freed memory after a removal (`attacks2.exe 9`); registering the list by its buffer fixes it (`attacks2.exe 14`); on churn it costs 1.20x to 1.27x plain C, 1.09x to 1.10x if an object's header can point at its side record |
| A8, graphs | lists of `weak` edges kept in step both ways | **saved** | (first attack: open, no rules) the rules are written here (4.4); measured 0.97x and 1.08x plain C with the same reverse lists |
| A12, closures capturing mutable state | **captures work as in Python**: copy, move, pointer or box | **saved** | (first attack) a boxed variable is a place in effect summaries; "copy" only for `Copy`. (Here) the same hole with a variable captured by **pointer** (`attacks2.exe 11`): any variable a closure writes is such a place. The box's cost in a hot loop was checked: none (0.99x and 1.00x) |
| A32, callbacks that remove themselves | **a stored closure is an object, protected where it is used** | **saved** | (first attack) a closure in a variable another closure writes gets a box; the body's retain is dropped when the field's closures are known. (Here) a stored closure called with an argument that removes it was freed before its call (`attacks2.exe 3`), and one lending its own captured list to a call that frees it (`attacks2.exe 5`): both are protected by the ownership group's own rules, which the report's narrower wording left out |
| A13, downcasting | **`isinstance` narrows** | **saved** | (first attack) narrowing ends at hidden writes; constant-time subclass test; flow-sensitive. (Here) narrowing of a local that a closure writes read a `Square` as a `Circle` (`attacks2.exe 1`): such a local is narrowed like a place |
| A13, generic methods through `dyn` | a `dyn` table entry instantiated with `V = dyn Visitor` | **saved** | (first attack) each implementation's body is checked for using `V` only through references |
| A13, registry by type | the references group's pool table and place-cast | **kept** | as saved by the references group's attacker |
| A16, bounds | **bounds inferred from the body** | **saved** | (first attack) a bound naming an interface others cannot see is refused. (Here) Python's duck-typed method call and its `acc = 0` sum are still refused: two compile-time extensions proposed (section 8) |
| A16, orphan rule | a program nobody depends on may implement anything | **kept** | no problem found by either attack |
| A16, overlap | the most specific implementation wins | **open** | still unproven; (first attack) with separate compilation two implementations can run for one type |

**Measured** (PROVISIONAL; median of 5 pinned runs; ratio to plain C doing the same work in the same
batch; two batches each):

| What | This attack | The report |
|---|---|---|
| tree, back links with the 8-byte header, against plain C laid out the same way | 1.202, 1.309 (`runs/w2_a.txt`, `w2_b.txt`) | 1.103 |
| tree, back links without the header (the link's own work) | 1.025, 1.149 | 1.078 |
| tree, safe Rust's index arena | 1.519, 1.357 | 1.261 |
| tree walk calling a stored callback on every step, back links, against the same walk in plain C | no protection needed: 1.166, 1.030; with the count pair the ownership group's rule adds: 1.477, 1.329 (`runs/w2v_a.txt`, `w2v_b.txt`) | not measured |
| callbacks, the answer, against plain C doing exactly its work with the same slab | 1.045, 1.037 (`runs/w4_a.txt`, `w4_b.txt`) | "1.00" (against `malloc`); 1.08 against the slab |
| callbacks, safe Rust, same baseline | 1.490, 1.472 | 1.36 against `malloc` |
| a boxed captured variable written in a hot loop, against a C local | 0.990, 0.998 (`runs/micro_a.txt`, `micro_b.txt`) | not measured |
| Python's filter idiom on a back-linked list, rules as written / fixed | 43.9x, 44.5x at 10,000 children; 85.1x, 90.6x at 20,000 / 1.27x, 1.42x; 1.13x, 1.24x (`runs/filter_e.txt`, `filter_f.txt`) | not measured |
| a `weak` observer list under churn, registered by slot (made safe) / by buffer / by buffer with the record found through the object | 25.1x, 25.2x / 1.27x, 1.26x / 1.09x, 1.10x of plain C that tracks its lists (`runs/weak_g.txt`, `weak_h.txt`); safe Rust 0.92x, 0.94x | not measured |
| a graph with edges kept in step both ways | 0.969, 1.081 (`runs/graph_a.txt`, `graph_b.txt`) | argued, not measured |

---

## 1. Words used

- **Handle**: what a variable of a `class` type holds, a pointer to an object on the heap. **Count**:
  how many owning handles an object has; it is freed when the count reaches zero. A **count pair** is
  one increment and one decrement around something (Rust's `Rc::clone` and its drop).
- **Header**: the words at the start of each object; the ownership group proposes 8 bytes, a 32-bit
  count and a 32-bit type index, "the lean class" (`phase3b-ownership.md` 2.3).
- **Back link**: the report's answer for parent pointers: a field the language keeps equal to "the
  object whose list holds me". Its rules are numbered 1 to 7 in the report's section 3.3.
- **Holder**: the object whose field holds another (a parent for its children).
- **Box**: a small counted heap block holding a closure's captured state, or one captured variable
  shared by a function and its closures.
- **Stored closure**: a closure kept in a list or a field to be called later (a handler, a callback).
- **Effect summary**: what a function may change, computed from its body (the ownership group's field
  effects, `phase3b-ownership.md` 2.1). **Reshape**: a change that can free or move memory.
- **Loan**: a pointer the compiled program holds into a place for a while.
- **Narrowing**: after `if isinstance(x, C):`, the compiler treats `x` as a `C` inside the block.
- **Registration** (for `weak`): storing into a `weak` field records the field's address in a **side
  record** belonging to the target, found through a **side table** keyed by the target's address; when
  the target dies, every recorded field is set to `None` (the report's 3.3, after Objective-C).
- **Walkers count**: the containers group's per-list count of running loops; while it is above zero,
  removing from the list panics (`phase3b-containers.md` 4.3, rule 9).
- **Panic**: a deterministic stop with a message; never a read of the wrong memory (`[PAN-1]`).
- **Use after free**: reading or writing memory that was freed. **Buffer overflow**: reading past the end
  of an object. **Type confusion**: reading an object as a type it is not. All three are what Rust's
  rules exist to prevent.
- **Monomorphise**: compile a separate copy of a generic function for each type it is used with.
- **Slab**: an allocator handing out same-size blocks from large chunks (Python's own allocator does).

---

## 2. Do the C models do exactly what the compiler would emit, and exactly the work?

I read every timed model of the report line by line against its rules, and its baselines against the
Python programs.

**Tree workload** (`graphs-callbacks/bench/w2/w2_backlink.c` against `w2_plain_malloc.c -DSLAB -DARRAY24`):

- The walk is `for (Node *a = n->parent; a; a = a->parent) depth++;` in both: the same instructions.
  The rules put no count, check or test on it, and nothing in it is a call, so the ownership group's
  retain rule adds nothing. Correct.
- The re-parent is search, order-keeping remove, push and one store in both. The model changes no
  count when a node moves; that is right only because rule 2 *moves* the handle from the old list to
  the new (a release followed by a retain would free a node whose only owner is the old list, between
  the two). The rule text says "no count changes"; the compiler must emit it fused, as the model does.
- Differences that are not the memory model: the Ember model's `nodes` list grows by `push` from 4
  elements and bounds-checks each random index, where the baseline allocates the million pointers once
  and checks nothing (the first attack measured this at 2% to 8%); the baseline's slab clears each node
  with `memset` and the model's does not (in the baseline's disfavour). Both are small and go both ways.

**Callback workload** (`w4/w4_ember_point.c` against `w4/w4_plain.c`):

- The Ember model does what rules 1 to 3 say: `{function, box}` copied out, no count on the call, the
  body's count pair only on the rare path around `unregister_current`.
- The baseline does slightly *less* work than the Python program and the Ember model: no `v >= 0` test,
  no returned counter, and its final sum reads each environment's counter directly instead of calling
  each handler with `v = -1`. I wrote `graphs-callbacks-attack2/bench/w4_plain_same.c` (the group's file
  with exactly those three things added) and measured it with the same slab: 21.366 and 21.308 ms,
  against 21.991 and 21.484 ms for the group's slab baseline (`runs/w4_a.txt`, `w4_b.txt`). The
  difference is under 3%: the group's numbers stand, and the tables here use the same-work baseline.
- The report's headline "1.00x plain C" is against the `malloc` baseline, which is slower than the
  slab baseline of the same table; against the same-work slab baseline it is 1.045 and 1.037 (section 6.4).

**`weak` model** (`w2/w2_zweak.c`): never frees a node, so it says nothing about the cost of a death or a
holder's death, and it has no `weak` lists (the first attack noted the first; section 4.2 measures the
second).

---

## 3. Back links (A8)

### 3.1 The first attack's findings, checked

| Finding (first attack) | Program | What I checked | Fix (cost) |
|---|---|---|---|
| an indexed put of a node already in the same list frees a node still in the list | `p.children[1] = p.children[0]` (Python: `['a', 'a']`) | `attacks.exe 1` reproduces (`heap-use-after-free`). The spec's own order makes this the emitted order: "an assignment evaluates its right side first, ... then the target places, then stores" (`[EXP-2]`, spec section VI.1), so the slot is found before rule 2 shifts the list | the put panics naming both positions; `xs[i], xs[j] = xs[j], xs[i]` compiles to `xs.swap(i, j)` (none: the test is on the path rule 2 already takes) |
| a mutable view walks around rule 6 | `blank_first(p.children)` with `fn blank_first(xs: MutSpan[Node])` | an `Array` becomes a `MutSpan` at a `mut` site (`[SPN-1]`, spec line 918: read) | rule 6 closes every way to replace an element without the link's own operations (none) |
| the link's hidden writes must be in effect summaries | `new_parent.children.push(node)` writes `node.parent` and reshapes the old parent's list | used by narrowing (`attacks.exe 6`/`7` reproduce) | the writes of rules 2, 3 and 4 are in summaries (none when running) |
| Python's `child.parent = self` is refused by rule 5 | `def add(self, child): child.parent = self; self.children.append(child)` | the reason for rule 5 holds | the write is accepted and removed when the same block makes it true before any read (none) |
| a tree shared with a thread | a root dropped on one thread writes children's `parent` | follows from the hidden writes | the threads group's share check refuses it (none) |

### 3.2 A link assigned the value it already holds (memory safety, new)

Rule 2 says a put "first takes the object out of its current holder's `g`", rule 3 that a replaced value
gets its link set to `None`. The usual code for an assignment reads the old value, stores the new one,
then releases the old one. For a one-to-one back link (the report's doubly linked list), assigning a
field the value it already holds then frees that value:

```ember
class LNode:
    v: int
    next: Option[LNode] = None
    prev: Option[LNode] = backlink(next)

a.next = c
a.next = a.next            # or any `x.next = y` where y already is x.next
println(a.next.v)
```

```python
a.next = c; c.prev = a
a.next = a.next
print(a.next.v)            # 2   (py_attack2_output.txt, link_self_assign)
```

As written: the old value `c` is read; rule 2 takes `c` out of its holder (`a.next = None`, its count
moving with it); `a.next = c`; then rule 3 on the old value `c`: link cleared and the count released,
which frees `c`, still in `a.next`. `attacks2.exe 7`: `AddressSanitizer: heap-use-after-free ... READ of
size 8` at `a.next.v`.

**The fix.** Rules 2 and 3 are one membership change, in this order: (1) unlink the old value (its link
to `None`; no release yet); (2) link the new value: if it is still in another holder, take it out there
(its count moves); if it is in none, it gets a count for its new place; (3) release the old value.
`attacks2.exe 8`: `a.next.v = 2`, as Python. **Cost**: none; the same operations in a fixed order, and
the test in step 2 is the one rule 2 already makes.

### 3.3 Python's filter idiom (speed, new)

The same ordering decides the cost of Python's most common list idiom on a back-linked list:

```ember
p.children = [c for c in p.children if c.alive]
```

```python
p.children = [c for c in p.children if c.alive]       # dropped children keep a stale .parent in Python
                                                       # (py_attack2_output.txt, filter_idiom_children)
```

Read as written, rule 2 applies to every element of the new list: each kept child's current holder is
`p` itself, so each is searched for in the old list and removed from it with a memory move, then rule 3
clears what is left. That is quadratic. With the order of 3.2, step 1 has cleared every old element's
link, so step 2 finds no holder to search, and the whole assignment is linear.

`graphs-callbacks-attack2/bench/filter.c` (10 rounds: filter a third of the children out, then push
them back; `-DLITERAL`, `-DORDERED`, `-DORDERED -DNOCOUNT` with the count updates left out to take the
cost apart, `-DPLAIN` for plain C), `runs/filter_e.txt`, `filter_f.txt`:

| Program | 10,000 children | | 20,000 children | |
|---|---|---|---|---|
| plain C | 1.050 ms (1.00) | 1.029 ms (1.00) | 2.287 ms (1.00) | 2.123 ms (1.00) |
| the rules as written | 46.051 (43.9x) | 45.761 (44.5x) | 194.722 (85.1x) | 192.368 (90.6x) |
| the rules with the order of 3.2 | 1.335 (1.27x) | 1.462 (1.42x) | 2.591 (1.13x) | 2.629 (1.24x) |
| the same without count updates | 1.203 (1.15x) | 1.143 (1.11x) | 2.503 (1.09x) | 2.516 (1.19x) |

Doubling the list quadruples the time as written (O(n²)); with the fixed order it doubles. Of what is
left, the link's own work is 1.09x to 1.19x of plain C (it clears every old element's link and sets
every kept element's; plain C writes only the dropped ones), and the count updates of the two new lists
are the rest, the ownership group's cost for any list of class handles. These are 1 to 3 ms programs
and spreads reached 52%: under 15% or near it, for the final measurer. A compiler can go further and
compile "a field assigned a filter of itself" as the containers group's in-place filter (0.98x plain C,
`phase3b-containers.md` 4.3), which leaves kept elements and their links untouched (argued, not built).

### 3.4 Python's two-line unlink (readability, new)

```python
node.prev.next = node.next
node.next.prev = node.prev            # (py_attack2_output.txt, unlink_two_lines: [1, 3], c.prev is a)
```

```ember
node.prev.unwrap().next = node.next    # all of the unlink: moves c into a.next, sets c.prev = a, frees b's place
node.next.prev = node.prev             # refused: prev is a back link (rule 5); node.next is None by now
```

The first line already does everything; the second is refused, and the first attack's acceptance rule
(3.1) does not help, because `node.next` was moved out by the first line. The error should say "the line
above already set `prev`". A shortfall in Python's habit, not in safety.

### 3.5 A walk that calls unknown code on every step (speed, new)

The report: "Reading `n.parent` is a plain pointer load." True for the measured walk, which calls
nothing. A walk that reports each ancestor to a stored callback is common:

```ember
class Tree:
    nodes: Array[Node] = []
    on_visit: fn(Node) = fn(n: Node): pass       # a stored callback

    fn depth_total(self) -> int:
        depth = 0
        for n in self.nodes:
            a = n.parent
            while a is not None:
                self.on_visit(a)                 # stored: its effects are unknown, so it may drop a Node
                depth += 1
                a = a.parent                     # `a` is used after the call: retained across it
        return depth
```

```python
def depth_total(self):
    depth = 0
    for n in self.nodes:
        a = n.parent
        while a is not None:
            self.on_visit(a)
            depth += 1
            a = a.parent
    return depth
```

Under the ownership group's rule (a handle read from a place is retained when anything in its life may
drop an object of its class, `phase3b-ownership.md` 2.3), each step gets a count pair. I added the call
to the tree workload (`bench/w2v_plain.c` and `bench/w2v_backlink.c`: the group's files with one change,
a call through a function pointer the C compiler cannot see on every step; `-DRETAIN` adds the count
pair), `runs/w2v_a.txt`, `w2v_b.txt`, `w2_a.txt`, `w2_b.txt`:

| Program | w2v_a | w2v_b | w2_a | w2_b |
|---|---|---|---|---|
| plain C, same layout, same call | 892.7 ms (1.000) | 1031.1 (1.000) | 1168.3 (1.000) | 899.8 (1.000) |
| back links, no protection (the callee's effects are known) | 1040.9 (1.166) | 1062.0 (1.030) | 1257.0 (1.076) | 1039.0 (1.155) |
| back links, with the count pair | 1318.3 (1.477) | 1370.2 (1.329) | 1624.6 (1.391) | 1373.6 (1.527) |
| back links, a per-thread stack of protected objects instead (`-DHAZARD`) | 1300.4 (1.457) | 1344.2 (1.304) | | |

The count pair costs 1.27x, 1.29x, 1.29x and 1.32x of the same walk without it. **Trying to save it**:
a cheaper protection for the same purpose, pushing `a` on a per-thread stack that a release reaching zero
checks before freeing (no write to the node), costs the same (1.25x and 1.27x): not cheaper, discarded.
What removes it is knowing the callee's effects:

- **A callback passed as a parameter costs nothing**: today's `[CLO-3]` already compiles a `fn(Node)`
  *parameter* as an implicit generic, one copy per function passed, and `[EFF-2]` gives each copy that
  function's exact effects. `def walk(tree, visit)` written Python's way is that case.
- **A stored callback** (`self.on_visit`) or a `dyn` one has "all effects" (`[EFF-2]`), so the count
  pair stays unless the field's closures are all known (the first attack's save in 6.1 below) or its
  type declares what it may not do: `[EFF-2]` already lets a stored callable's type declare contracts
  (`@noalloc fn(A) -> R`); a contract "drops no `Node`" would be one more (proposed spelling:
  `on_visit: @keeps(Node) fn(Node)`; UNVERIFIED that the effect system can express it).

Against Rust: its index arena pays nothing here (the walk holds the arena borrowed, so the callback
cannot remove a node), and its `Rc` tree pays a count pair on every step anyway. This cost is the
ownership group's rule, not the link's; it is passed to them.

### 3.6 The tree workload re-measured

`runs/w2_a.txt`, `w2_b.txt` (spreads up to 74% and 42%), ratio to plain C laid out the same way:

| Program | w2_a | w2_b |
|---|---|---|
| plain C, same layout (`w2_plain_malloc.c -DSLAB -DARRAY24`) | 611.5 ms (1.000) | 537.5 (1.000) |
| back links, 8-byte header (`w2_backlink.c -DSLAB`, the answer as emitted) | 735.0 (1.202) | 703.7 (1.309) |
| back links, no header (`-DNOHEADER`) | 626.6 (1.025) | 617.5 (1.149) |
| plain C, all nodes in one array (`w2_plain_keep.c`) | 525.8 (0.860) | 518.8 (0.965) |
| safe Rust, index arena (`w2_rust_index.rs`) | 928.6 (1.519) | 729.5 (1.357) |

The link's own work is within 15% of plain C (1.03, 1.15). The 8-byte header again costs more than the
report's 2% (+17% and +14% here; the first attack measured +23%, +15%, +2%, +7%); a 4-byte header for
classes that need no type index measured 0.964 and 1.024 in the first attack and was adopted by the
combined report (`phase3b-combined.md` section 4.4). Back links are faster than Rust's index arena in
both batches (0.79x and 0.96x of its time with the header).

### 3.7 Verdict: saved

Back links, with the first attack's four fixes (3.1) and the membership change in a fixed order (3.2),
which also makes the filter idiom linear (3.3). Memory-safe by the report's invariant with those doors
closed. Speed: the link's own work is within 15% of plain C; the header is the ownership group's
choice; a walk calling a stored callback pays the ownership group's count pair unless the callback's
effects are known (3.5). What it lacks against Python: a cycle of owning links leaks (as in Rust); a
child of a dead parent sees `None`; the two-line unlink (3.4).

---

## 4. `weak` fields and lists (A8)

### 4.1 The first attack's findings, checked

| Finding (first attack) | Program | What I checked | Fix (cost) |
|---|---|---|---|
| the holder dies before its target; the target's death writes into the freed holder | `cam = Camera(target=player); cam = None; player = None` | `attacks.exe 4` reproduces (`heap-use-after-free ... WRITE`); `5` is clean | a holder's drop stores `None` into its `weak` fields, which unregisters them (a lookup per holder death; not measured) |
| a weakly held object shared with a thread is cleared from another thread | a `Player` shared with an unscoped thread | follows from the report's "per thread" side table | a class that is the target of any `weak` field cannot be shared with an unscoped thread; `SyncWeak[T]` at Rust's price (none otherwise) |

### 4.2 `weak` lists (memory safety and speed, new)

The report: "an Array of weak elements (`Array[weak T]`) registers its elements again when its buffer
moves". A removal moves elements too, without moving the buffer:

```ember
class Subject:
    observers: Array[weak Observer] = []

s.observers.push(o1); s.observers.push(o2); s.observers.push(o3)
s.observers.remove(0)                                   # o2 and o3 shift down one slot
s.observers.push(o4); s.observers.push(o5); s.observers.push(o6)   # the buffer moves
o2 = None                                               # o2 dies: its registered fields become None
```

```python
observers = [weakref.ref(o1), weakref.ref(o2), weakref.ref(o3)]
del observers[0]
observers += [weakref.ref(o4), weakref.ref(o5), weakref.ref(o6)]
del o2
print([None if r() is None else r().id for r in observers])    # [None, 3, 4, 5, 6]
```

**As written** (slots registered, re-registered when the buffer moves): after the removal, o2's record
names its old slot; the growth re-registers each element by its current slot, finds no record for o2's,
and frees the old buffer; o2's death then writes `None` into it. `attacks2.exe 9`:
`heap-use-after-free ... WRITE of size 8`.

**First save: re-register every shifted slot too.** Safe, but a removal from a list of n observers then
costs n side-table lookups. **Second save: register the list, not its slots** (the address of its
`{ptr, len, cap}` inside the holder object): shifts and growth change nothing; a death scans each list
that registered with it. `attacks2.exe 10`: `observers: None 3 4 5 6`, as Python.

**That save attacked again**: under "names follow values" (`phase3b-ownership.md` 2.2 rule 2) a list can
move out of its field:

```ember
s.observers.push(o1)
old = s.observers           # `old` names the list ...
s.observers = []            # ... which moves into `old` here instead of being dropped
o1 = None
println(old[0])             # Python: None
```

The registration still names `s.observers`, now an empty list; `old`'s slot keeps the dead `o1`.
`attacks2.exe 13`: `heap-use-after-free ... READ`. **Third save: register the buffer.** The buffer moves
with the list value; it carries a one-word prefix with the live length, kept by the list's own `push`
and `remove`; a growth that moves the buffer re-registers each element once (one lookup per element
per growth, so one per push on average); a death scans each registered buffer. `attacks2.exe 14`:
`old[0] = None`. Attacked again: sorting, swapping two lists, moving a list into another field or
another list (the buffer goes with it), `clear()` (each element unregisters), dropping the list (each
element unregisters) and the containers group's in-place filter (it scans live slots, all valid memory):
no further hole found.

**Cost**, on churn (`bench/weakarr.c`: 1,000 observers; 300,000 operations at random positions, 99 in
100 an unsubscribe and resubscribe, 1 in 100 a death and a new observer; every program prints the same
checksum, 104237838987), `runs/weak_e.txt` to `weak_h.txt`:

| Program | weak_g | weak_h | against Rust |
|---|---|---|---|
| plain C, each observer tracks the lists it is in (`-DPLAIN_TRACKED`; what C writes when an observer can be in several lists) | 9.125 ms (1.000) | 9.003 (1.000) | 1.089, 1.069 |
| plain C, each observer knows its one subject (`-DPLAIN`) | 8.747 (0.959) | 8.553 (0.950) | 1.044, 1.016 |
| slots registered, re-registered on every shift (`-DSLOTS`, the first save) | 228.685 (25.061) | 226.961 (25.209) | 27.283, 26.961 |
| the buffer registered (`-DBUFFER`, the third save) | 11.551 (1.266) | 11.362 (1.262) | 1.378, 1.350 |
| the same with a one-entry cache of the last record found (`-DBUFFER -DCACHE1`) | 10.905 (1.195) | 10.938 (1.215) | 1.301, 1.299 |
| the same with the record found through the object (`-DBUFFER -DDIRECT`) | 9.945 (1.090) | 9.896 (1.099) | 1.186, 1.176 |
| safe Rust, `Vec<Weak<Obs>>` (`bench/weakarr.rs`) | 8.382 (0.919) | 8.418 (0.935) | 1.000 |

What is left is the side-table lookup on each store and removal (the lean header has no room for a
pointer to the record). With the record reachable from the object, as Swift does by turning a weakly
referenced object's count word into a pointer to its side table, the buffer registration is 1.09x and
1.10x of plain C (under 15%) and 1.18x of Rust. Rust is faster on churn because its `Weak` registers
nothing; it pays on every read instead (an upgrade: a test and a count pair), where registration reads
with a plain load. That header decision is the ownership group's (and the combined report's, which has
since reopened registration against a read-time test, `phase3b-combined.md` 4.3).

### 4.3 Verdict: saved, with a cost left open

`weak` fields and lists cleared by registration, with the first attack's two fixes and the buffer
registration for lists. Reads stay a plain load. Left open: on churn, 1.20x to 1.27x of plain C with the
side table, 1.09x to 1.10x if the header can point at the side record; that choice belongs to the
ownership group.

### 4.4 Graphs whose edges both ways are kept in step: saved (rules written here)

The first attack left this open: the report gave no rules (back-link rule 2, "take it out of its holder
first", cannot apply to a city with many roads in). The rules:

1. `into: Array[weak C] = backlink(roads)` in class `C` pairs with `roads: Array[weak C]`, both
   non-owning. Every city in either list is alive, so their elements read as `City`, never `None`.
2. Putting `b` into `a.roads` (push, insert, an indexed put, assigning the whole field) also puts `a`
   into `b.into`; taking `b` out of `a.roads` (remove, pop, clear, a replaced element, replacing the
   field) takes one occurrence of `a` out of `b.into`. A road twice is two entries on each side.
3. `into` is read-only (fix-it: change `roads`), and neither list can be lent as a mutable view
   (back-link rule 6, with the first attack's widening).
4. When a city dies, before its memory is freed, it is taken out of `y.roads` for every `y` in its
   `into`, and out of `z.into` for every `z` in its `roads` (a road to itself once).
5. These hidden changes are in effect summaries: "the death of a `City` reshapes `City.roads` and
   `City.into` of some `City`" (as the first attack's 3.3 for back links).

Invariant: `b` appears in `a.roads` as many times as `a` appears in `b.into`, and no list names a freed
city. Attacked: a loop over `b.roads` that kills one of its cities (`for r in b.roads: cities.remove(r)`)
reshapes `b.roads` through a death, which the summaries of rule 5 expose: the containers group's loop
rule 9 applies (a panic, where Python's hand-written cleanup would skip elements); duplicates and roads
to itself are handled by rule 2 and rule 4 as written.

```ember
class City:
    id: int
    roads: Array[weak City] = []
    into: Array[weak City] = backlink(roads)

cities[0].roads.push(cities[1])          # also cities[1].into.push(cities[0])
cities[k] = City(id=next_id)             # the old city dies: taken out of every list that names it
```

```python
def road_push(a, b): a.roads.append(b); b.into.append(a)
def city_die(x):
    for y in x.into:
        if y is not x: y.roads.remove(x)
    for z in x.roads:
        if z is not x: z.into.remove(x)
```

**Cost**, measured (`bench/graph.c`: 100,000 cities, 8 random roads each; 10 passes, each walking every
road and replacing 1% of the cities with new ones with 8 new roads; plain C keeps the same reverse lists
by hand, and `-DEMBER` adds the header, the bounds checks and the count; the Python twin
`python/graph.py` prints the same checksum, 383684462149), `runs/graph_a.txt`, `graph_b.txt`: plain C
74.8 and 71.2 ms, the rules 72.5 (0.969) and 77.0 (1.081); memory 1.044 (the header). **Left**: an
undirected graph written Python's way (`a.roads.push(b); b.roads.push(a)`) keeps four entries per edge
(two roads, two `into`) where C keeps two; a field paired with itself as a set
(`roads: Set[weak City] = backlink(roads)`, where adding `b` to `a.roads` adds `a` to `b.roads` and a
second add is a no-op) would match C (argued, not measured).

---

## 5. Captures as in Python (A12)

### 5.1 The first attack's findings, checked

| Finding (first attack) | Program | What I checked | Fix (cost) |
|---|---|---|---|
| a boxed list emptied by a stored closure while a loop walks it | `bus.on("reset", fn(): xs.clear())`, then `for x in xs: bus.emit("reset")` | `attacks.exe 8` reproduces (`heap-use-after-free`), `9` panics | a boxed variable is a place in effect summaries; the containers group's rule 9 applies (its cost, only in such loops) |
| "copied" and "written after" need exact meanings | `get = fn() => n; set = fn(v: int): n = v` | `[CLO-13]` copies only `Copy` variables (spec line 2070: read) | copy only `Copy`; writes by other closures, partial writes and mutable uses count (none) |
| a stored closure's own box must move with it to a worker thread | `jobs.submit(fn(): work(x))` | consistent with `[CLO-3]` (owned callable values move) | the threads group treats it as part of the moved value (none) |
| a loop variable is new on each pass, unlike Python | `[lambda: i for i in range(3)]` | the report argues it (Python's FAQ, Go 1.22) | a deliberate difference, the owner's to accept |

### 5.2 The same hole with a variable captured by pointer (memory safety, new)

The first attack's fix names *boxed* variables. A variable the compiler proves outlives every closure
that captures it is captured by **pointer** (the report's rule 2), and the same program then breaks:

```ember
fn main():
    xs = ["ab", "cde", "f"]
    hs = [fn(): xs.clear()]       # hs dies before xs: xs is captured by pointer, not boxed
    total = 0
    for x in xs:
        hs[0]()                   # runs xs.clear()
        total += len(x)
    println(total)
```

```python
def main():
    xs = ["ab", "cde", "f"]
    def clear(): xs.clear()
    hs = [clear]
    total = 0
    for x in xs:
        hs[0]()
        total += len(x)
    return total                  # 2   (list_cleared_by_closure_during_loop)
```

Nothing in the loop's own text changes `xs`, so the loop keeps a pointer and a length.
`attacks2.exe 11`: `heap-use-after-free ... READ of size 2` (the loop reads a string `clear()` freed).

**The fix.** Any variable that a closure writes, wherever it lives (box or pointer), is a place in effect
summaries: the closure's writes to it are its effects, and a call with unknown effects may have them.
`attacks2.exe 12`: `panic: xs changed while a loop over it was running (line 6)`, the containers group's
trade for removals by unseen code (Python ends the loop with 2). **Cost**: none beyond rule 9's, only in
loops over such variables that call unseen code. The same fix serves narrowing (section 7.2).

### 5.3 The box's cost in a hot loop (speed, checked: holds)

The report: "The box is ... one pointer load per access; plain C's callback environment and Rust's
`Rc<Cell<i64>>` pay the same", measured only where every access is already through a pointer. The case
that could cost more is a hot loop in the function that owns the boxed variable:

```ember
total = 0
callbacks.push(fn() -> int: total)        # stored and escaping, total written after: a box
for rep in range(2000):
    for x in xs:                          # 65,536 numbers
        total += x
```

```python
total = 0
def read(): return total
callbacks.append(read)
for x in xs: total += x                    # (boxed_variable_hot_loop)
```

`bench/boxloop.c`, with the box and the list's buffer taken from an allocator in its own translation unit
(`bench/rtalloc.c`), as Ember's `ember_alloc` is (declared with no `malloc` attribute in
`runtime/ember_rt/include/ember_rt.h` line 1323), so the C compiler cannot assume they do not overlap;
`runs/micro_a.txt`, `micro_b.txt`, ratio to a C local accumulator:

| Program | micro_a | micro_b |
|---|---|---|
| plain C, a local accumulator | 11.155 ms (1.000) | 11.026 (1.000) |
| plain C, the callback's variable updated in the loop | 11.179 (1.002) | 11.003 (0.998) |
| the box, every access through its pointer (as the rule says) | 11.042 (0.990) | 11.005 (0.998) |
| the box, kept in a local across the loop | 11.047 (0.990) | 11.056 (1.003) |
| safe Rust, `Rc<Cell<i64>>` | 11.206 (1.005) | 11.116 (1.008) |

The claim holds: clang tests once whether the box overlaps the list and runs the vector loop when it
does not (`bench/out/boxloop_ember_naive.s`: an overlap test before the loop, a scalar copy of the loop
for the overlapping case). For loops too complex for that test, Ember already outlines loops into
functions whose list parameters are `restrict` (`ember_rt.h` line 371, ADR-079); a boxed variable can be
passed the same way. No fix needed.

### 5.4 Verdict: saved

Captures as in Python, with the first attack's three fixes and the widening of 5.2. The box costs
nothing measurable (W4: 1.018 and 1.052 against the pointer version's 1.045 and 1.037; the hot loop:
0.99 and 1.00).

---

## 6. A stored closure is an object (A32)

### 6.1 The first attack's findings, checked

| Finding (first attack) | Program | What I checked | Fix (cost) |
|---|---|---|---|
| a closure held in a variable that another closure overwrites is freed during its own call | `on_tick = fn(): reset(); println(name)` where `reset` assigns `on_tick` | `attacks.exe 10` reproduces, `11` prints `first` | a closure whose holder a call may overwrite (list element, field, or a variable some closure writes) keeps its state in a counted box (a box only for such variables) |
| the body's count pair on every event when the handler calls unseen code | `d.log(k)` before using the state | consistent with my 3.5 (1.27x to 1.32x for a count pair per step on the tree) | no count pair when every closure stored in the field is known, by the field's recorded closure types (first attack: 0.95x and 0.99x of plain C with, 1.09x and 1.12x without) |
| hot reload could undo the "no protection" path (report 5.4) | a reloaded handler that removes handlers | reasoning checked | a reload that would change the choice is refused (none) |
| a `for` dispatch whose handler removes itself panics | `for h in self.handlers: h(self)` | the report says so itself (5.5) | waits on the containers group (a shortfall, stated) |

### 6.2 A stored closure called with an argument that removes it (memory safety, new)

Rule 2: "Calling a stored closure copies `{function, box}` out of its slot and calls. The caller uses
nothing of it after the call, so it protects nothing." But the arguments are evaluated after the callee:
`[EXP-1]` (spec section VI.1) evaluates operands left to right, completely, before the call, and Python
evaluates the callee first.

```ember
class Bus:
    handlers: Array[fn(int)] = []
    fn clear_and_count(self) -> int:
        n = len(self.handlers)
        self.handlers.clear()
        return n

name = "first"
bus.handlers.push(fn(n: int): println(f"{name} {n}"))
bus.handlers[0](bus.clear_and_count())
```

```python
bus.handlers.append(lambda n: print(f"{name} {n}"))
bus.handlers[0](bus.clear_and_count())     # first 1   (callee_evaluated_before_args)
```

The callee's `{function, box}` is copied out; evaluating the argument clears the list and frees the box;
the call then runs on freed state. `attacks2.exe 3`: `heap-use-after-free ... READ of size 8 ... in
print_body`.

**The fix.** The copied-out callee is a name like any other, so the ownership group's rule for names
applies (`phase3b-ownership.md` 2.2 rule 2: a class handle whose place may change inside a call is
retained before that call): when an argument's evaluation may remove a closure stored where the callee
came from, the callee's box gets a count pair around that argument. `attacks2.exe 4`: `first 1`, as
Python. **Cost**: a count pair only in that shape. Evaluating the arguments first instead would cost
nothing but change Python's meaning (the call would find an empty list and panic).

### 6.3 A handler lending its own captured state to a call that frees it (memory safety, new)

Rule 3: the body retains its box across a call that may release it when it "uses its box after it".
A body can also use its box *during* the call, by lending part of its state:

```ember
class Bus2:
    handlers: Array[fn(Bus2)] = []
    fn report(self, xs: Array[int]):          # a read-only loan of the caller's list
        self.handlers.clear()                 # drops the running handler's box, and xs with it
        println(xs[0])

seen = [1, 2, 3]                              # used later only by the closure: moved into its box
bus.handlers.push(fn(b: Bus2): b.report(seen))
bus.handlers[0](bus)
```

```python
bus.handlers.append(lambda b: b.report(seen))
bus.handlers[0](bus)                          # prints 1   (closure_lends_its_state)
```

`attacks2.exe 5`: `heap-use-after-free ... READ of size 8 ... in lend_body`.

**The fix.** The report says the ownership group's rule "applies to the box as to any object"; that rule
already covers loans: "An object is not freed under a loan into it, because a loan through a handle keeps
the object; the retain it needs is left out only when no statement in the loan's life may drop an object
of that class" (`phase3b-ownership.md` 2.1). Rule 3's wording becomes "used during or after the call".
`attacks2.exe 6`: `xs[0] = 1`. **Cost**: a count pair on calls that receive a loan into the box and may
release it; when that is every event (a handler passing its captured name to a stored logger), it is
the first attack's per-event measurement (1.09x and 1.12x of plain C), removed the same way when the
field's closures are known.

### 6.4 The callback workload re-measured

`runs/w4_a.txt`, `w4_b.txt`, ratio to plain C doing exactly the same work with the same slab (section 2):

| Program | w4_a | w4_b |
|---|---|---|
| plain C, same work, same slab (`w4_plain_same.c -DSLAB`) | 21.366 ms (1.000) | 21.308 (1.000) |
| the group's plain C with the slab | 21.991 (1.029) | 21.484 (1.008) |
| the group's plain C with `malloc` | 22.844 (1.069) | 22.902 (1.075) |
| **the answer** (`w4_ember_point.c`) | 22.320 (1.045) | 22.100 (1.037) |
| the answer with `total` boxed (`-DBOXED`) | 21.750 (1.018) | 22.408 (1.052) |
| safe Rust (`w4_rust.rs`) | 31.846 (1.490) | 31.355 (1.472) |

The answer is within 5% of plain C and 0.70x of safe Rust's time in both batches; memory 4,256 KB
against 4,080 KB (each box carries a count). Checksums 34946993000 on every run.

### 6.5 Verdict: saved

A stored closure is an object, with the first attack's three fixes, the callee protected while its own
arguments are evaluated (6.2), and the body's protection covering loans of its state (6.3), both of which
are the ownership group's existing rules applied without the report's narrower wording.

---

## 7. Dynamic types (A13)

### 7.1 The first attack's findings, checked

| Finding (first attack) | Program | What I checked | Fix (cost) |
|---|---|---|---|
| narrowing outlives a hidden write of a back link | `if isinstance(n.parent, Folder): other.children.push(n); n.parent.icon` | `attacks.exe 6` reproduces (`heap-buffer-overflow`), `7` re-tests | narrowing of a place ends at any write of that field in any object, hidden writes included (none) |
| the subclass test needs the whole program | `isinstance(x, OpenClass)` with separate compilation | argument checked; not measured | each type record holds its depth and its ancestors by depth: a few loads and two compares, no loop (HotSpot's primary supertypes) |
| narrowing should follow the flow | `if not isinstance(s, Circle): return 0.0` then `s.r` | consistent with mypy | early exits and `assert` narrow (none) |
| a `dyn` entry for a generic method needs every implementation to use `V` only through references | `seen: Array[V] = []` in an implementation | reasoning checked | checked per implementation; error at the conversion to `dyn` (none) |

### 7.2 Narrowing a local that a closure writes (memory safety, new)

The report: "For a local, narrowing lasts until the local is assigned." Under A12's rule 1 a closure can
assign the local, and no assignment appears in the function's own text:

```ember
open class Shape: pass
class Circle(Shape):
    r: float = 1.5
class Square(Shape): pass

s: Shape = Circle()
hs = [fn(): s = Square()]        # a closure that writes s
if isinstance(s, Circle):
    hs[0]()                      # no assignment to s written here
    println(s.r)                 # as written: s is still narrowed to Circle
```

```python
if isinstance(s, Circle):
    hs[0]()
    print(s.r)                   # AttributeError: 'Square' object has no attribute 'r'
```

`attacks2.exe 1`: `heap-buffer-overflow ... READ of size 8`: `r` is read past the end of a `Square`. With
a field of another type at that offset, it would read a float as a pointer: type confusion.

**The fix.** A local that any closure writes is narrowed like a place (section 5.2's fix gives it
effects): the narrowing ends at any call that may run such a closure, and a later `s.r` is a
compile-time error asking for the test again, with the fix-it `c = s` before the test (a name keeps the
old value when the place changes inside a call, `phase3b-ownership.md` 2.2 rule 2, so `c` stays the
`Circle`, as Python's `c` would). `attacks2.exe 2`: the re-test sees a `Square`. **Cost**: none when
running.

### 7.3 Generic classes and interfaces in `isinstance` (readability, new)

- Today's `cast x if Holder[int]` compiles (`probes/a13_generic_cast.em`), and today's type information is
  per instance: a class's type-information symbol is "its path, mangled, then its instance hash"
  (`compiler/ember_types/src/lib.rs` lines 3292 to 3293). So the header's type index must name the
  instance, not the class, or `isinstance(x, Holder[int])` would accept a `Holder[String]`. Python's
  `isinstance(x, Holder)` (no type argument) works and `isinstance(x, Holder[int])` is a `TypeError`
  (`isinstance_generic`). For Ember's `isinstance(x, Holder)`: the type record carries its generic
  class (one more load), and the narrowed type is "a `Holder` of some type", whose `T`-typed fields need
  a further test. Proposed; not built.
- Python's `isinstance(x, Iterable)` tests an interface. The report's rule covers classes only. Narrowing
  to `dyn I` needs the type record's list of implemented interfaces (Go's type assertion to an
  interface works this way); cost argued, not measured. Left as a shortfall until built.

### 7.4 Verdict: saved

`isinstance` narrowing with the first attack's fixes and the captured-local rule of 7.2; generic
methods through `dyn` as the first attack saved them; the references group's pool table and place-cast
**kept** as their attacker saved them (`phase3b-references-attack.md` section 0).

---

## 8. Generics (A16)

### 8.1 The first attack's findings, checked

- **Bounds inferred from the body**: a `pub` function whose inferred bound names an interface other
  packages cannot see silently accepts only the package's own types; fix: refused with a request to
  export or write it (Rust refuses a private trait in a public interface). Checked by reasoning.
- **A program nobody depends on may implement anything**: kept; no problem found by either attack.
- **The most specific implementation wins**: open. Its counterexample (a library function compiled
  with the general implementation, a program choosing a more specific one for the same type: two
  implementations of one interface for `i32` in one program) is right by reading.

### 8.2 Python's duck-typed method call (readability, new)

```ember
class Crate:
    kg: int = 5
    fn weight(self) -> int: return self.kg

fn heaviest[T](xs: Array[T]) -> int:
    best = 0
    for x in xs:
        w = x.weight()
        if w > best: best = w
    return best
```

```python
def heaviest(xs):
    best = 0
    for x in xs:
        w = x.weight()
        if w > best: best = w
    return best                     # [5, 2] for crates and sacks   (duck_typed_method)
```

Today: `error[E2040]: T has no method weight; its bounds do not provide one`
(`probes/a16_duck_method.em`). Under the report's rule it stays an error ("a method name that no
interface in scope declares is an error in the body"). **Save proposed**: such a name becomes a
structural requirement in the inferred bounds ("`T` has `fn weight(self) -> R`"), checked when each
instance is compiled, as every instance already is; the error lands at the call and names the body line
(today's E2040 shape). **Cost**: none when running (each instance is a direct call). **Gives up**: a typo
in such a name surfaces at the first call rather than in the body, which is how C++ templates behave and
what nominal bounds avoid; the inferred requirement is listed by `ember inspect --bounds` and in the
package interface like any bound. The owner may prefer the error; it is a readability trade, not a
safety or speed one.

### 8.3 A sum that starts from 0 (readability, new)

```ember
fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:
    acc = 0
    for x in xs:
        acc = acc + x
    return acc
```

```python
def total(xs):
    acc = 0
    for x in xs: acc = acc + x
    return acc                       # total([1.5, 2.5]) == 4.0
```

Today, even with the bound written: `E2020: + cannot be applied to i64 and T` and `expected T, found
i64` (`probes/a16_sum_from_zero.em`). **Save proposed**: an integer literal assigned to a variable that
later holds a `T` takes the type `T`, with the inferred bound "made from an integer literal" (Haskell
types numeric literals this way; Rust needs `T::zero()`). **Cost**: none (the constant is made per
instance when compiling). UNVERIFIED that it fits Ember's inference order; a literal whose later uses
disagree stays an error.

### 8.4 Verdicts

Inferred bounds: **saved** (the first attack's fix; the two extensions above proposed, not required for
safety or speed). Program exemption from the orphan rule: **kept**. Most specific wins: **open**.

---

## 9. The report's "found in passing" defect, checked

The report: today's compiler accepts dispatch by index (`h = self.handlers.get_mut(i).unwrap(); h(self)`)
and its emitted C holds a pointer into the list's buffer across the call, then copies the closure's
state back into it. Read in `graphs-callbacks/probes/a32_emit_uaf/c/release/c/main.c`: line 233 takes
`&((...)_9.v.ptr)[_12]`, line 251 calls through it, and the trampoline (lines 131 to 137) copies the
state into a local, calls the body (passing the copy by value), then `memcpy(_env, &_l, sizeof _l)`
back to the same address. The claim stands (by reading; not run). The first attack found the same.

---

## 10. Pieces replaced, and their good parts

| Replaced piece | From | Why replaced | Good part kept |
|---|---|---|---|
| rules 2 and 3 of back links as two separate steps | the report, A8 | `a.next = a.next` freed `a.next`; the filter idiom was quadratic | rule 2's "the handle moves, no count changes", now step 2 of one membership change |
| re-registering a `weak` list's slots when they move | the report's words for growth; my first save for shifts | unsafe after a shift as written; 25x plain C when made safe | the side record per target, kept |
| registering a `weak` list by its header | my second save | unsafe when the list value moves out of its field | "nothing to do when slots move", kept by registering the buffer |
| a per-thread stack of protected objects instead of a count pair | tried here (3.5) | costs the same as the count pair (1.25x and 1.27x against 1.27x and 1.29x) | none beyond the measurement: the cost is the extra work around an opaque call, not the write to the node |
| "the caller uses nothing after the call, so it protects nothing" | the report's A32 rule 2 | false when the arguments run after the callee is copied out | true when they cannot remove it: the common case still pays nothing |
| "the body retains its box when it uses it after the call" | the report's A32 rule 3 | misses loans of the state into the call | the retain stays limited to calls that may release the box |
| "a local is narrowed until it is assigned" | the report's A13 | a closure can assign it | narrowing of locals at all, with no run-time cost |
| the first attack's "a boxed variable is a place in summaries" | the first attack's A12 fix | misses variables captured by pointer | the same rule, widened to every variable a closure writes |
| first-run pieces replaced by the first attack | `phase3b-graphs-callbacks-attack-first-run.md` section 10 | as stated there | as stated there |

---

## 11. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/graphs-callbacks-attack2/`.

- `c/attacks2.c` (modes 1 to 14), `c/attacks2_output.txt`, `c/attacks2.exe` with its sanitizer DLL.
- `python/py_attack2.py`, `python/py_attack2_output.txt`; `python/graph.py`, `python/graph_output.txt`.
- `probes/a16_duck_method.em`, `probes/a16_sum_from_zero.em`, `probes/a13_generic_cast.em`,
  `probes/run_checks.sh`, `probes/ember_check_output.txt`.
- `bench/build.sh` (builds every timed program into `bench/out/`; the group's from their own sources);
  `bench/ember_model.h` (the group's, copied); `bench/run.ps1`, `bench/medians.py` (the group's, copied);
  `bench/w4_plain_same.c`; `bench/w2v_plain.c`, `bench/w2v_backlink.c` (`-DRETAIN`, `-DHAZARD`);
  `bench/boxloop.c`, `bench/rtalloc.c`, `bench/boxloop.rs`; `bench/weakarr.c` (`-DPLAIN`,
  `-DPLAIN_TRACKED`, `-DSLOTS`, `-DARRAY`, `-DBUFFER`, `-DCACHE1`, `-DDIRECT`), `bench/weakarr.rs`;
  `bench/filter.c` (`-DPLAIN`, `-DLITERAL`, `-DORDERED`, `-DNOCOUNT`, `-DNCH=20000`); `bench/graph.c`
  (`-DEMBER`).
- `bench/runs/`: `w2_a.txt`, `w2_b.txt`, `w2v_a.txt`, `w2v_b.txt`, `w4_a.txt`, `w4_b.txt`, `micro_a.txt`
  to `micro_d.txt`, `weak_e.txt` to `weak_h.txt`, `filter_e.txt`, `filter_f.txt`, `graph_a.txt`,
  `graph_b.txt`, each with a `*_summary.txt` made by `python -I medians.py <file> <baselines>`. (The
  filter rows of `micro_a.txt` and `micro_b.txt` merge two list sizes under one name and are not used.)
- Timing command (PowerShell): `powershell -NoProfile -ExecutionPolicy Bypass -File
  C:/Users/ism19/Code/ember/target/redesign2/graphs-callbacks-attack2/bench/run.ps1 -Out <file> -Exes <a.exe,b.exe,...>`.
- The first attack's report: `C:/Users/ism19/Code/ember/tasks/96/redesign/phase3b-graphs-callbacks-attack-first-run.md`.

---

## 12. The final answer list for the graphs, callbacks and dynamic types group

| Id | Annoyance | Final answer | Kind | Cost when running (source; PROVISIONAL) | What is left | Verdict |
|---|---|---|---|---|---|---|
| A8 | trees with parent pointers, linked lists | **back links**: `parent: Option[Node] = backlink(children)`. A put into `children` is one membership change: unlink the old value, link the new one (taking it out of its old holder, its count moving with it), release the old value; removal or the holder's death clears the link. An indexed put of a node already in the list panics and the swap line compiles to `swap()`; no mutable view of the list; the hidden writes are in effect summaries; a link write the same block makes true is accepted and removed | (c) new mechanism | link work 1.03x and 1.15x of plain C laid out the same way, 1.20x and 1.31x with the 8-byte header (`runs/w2_a.txt`, `w2_b.txt`); 0.79x and 0.96x of Rust's index arena; filter idiom 1.13x to 1.42x of plain C (`runs/filter_*.txt`) | a walk calling a stored callback pays the ownership group's count pair (1.27x to 1.32x, `runs/w2v_*.txt`) unless the callback's effects are known; the header size is the ownership group's; a cycle of owning links leaks (as in Rust); Python's two-line unlink is one line | saved |
| A8 | other back references (observers, a camera's target) | **`weak` fields and lists cleared by registration**: a field registers its own address, a list registers its buffer; a target's death clears fields and scans lists; a holder's death unregisters; weakly held classes stay on their thread (`SyncWeak[T]` otherwise) | (a) Pythonic spelling | reads: a plain load; on churn 1.20x to 1.27x of plain C with the side table, 1.09x to 1.10x with the record reachable from the object; Rust 0.92x to 0.94x (`runs/weak_*.txt`) | the header room for a side-record pointer (ownership group); the cost of a holder's death not measured | saved |
| A8 | graphs | **edges kept in step both ways**: `roads: Array[weak City]`, `into: Array[weak City] = backlink(roads)`; a road added or removed changes both sides; a dying city leaves every list that names it; elements read as `City` | (c) | 0.969 and 1.081 of plain C keeping the same reverse lists; memory 1.044 (`runs/graph_*.txt`) | an undirected graph written Python's way keeps four entries per edge (a self-paired set field would keep two; argued) | saved |
| A12 | closures capturing mutable state | **captures work as in Python**: copied (only `Copy`, never written after by anything), moved, pointed to, or boxed; any variable a closure writes is a place in effect summaries; a closure's own box moves with it to a thread | (a) with (b) | none measurable: boxed `total` 1.018 and 1.052 against 1.045 and 1.037 by pointer (`runs/w4_*.txt`); boxed variable in a hot loop 0.99 and 1.00 of a C local (`runs/micro_a.txt`, `micro_b.txt`) | removal by unseen code during a loop over such a variable panics (containers rule 9); loop variables are per pass, unlike Python | saved |
| A32 | a callback receiving its holder; a handler removing itself | **a stored closure is an object**: state in a counted box whenever its holder may be overwritten during the call; the call copies two words out; the callee is protected while its own arguments run if they may remove it; the body keeps its box across calls that may release it while it uses the box during or after them, which known field closures rule out | (b) smarter compile time | 1.045 and 1.037 of plain C doing the same work; Rust 1.490 and 1.472 (`runs/w4_*.txt`); a count pair per event when a handler calls unseen code with its state (first attack: 1.09x to 1.12x) | `for h in handlers` with self-removal waits on the containers group | saved |
| A13 | trait objects and downcasting | **`isinstance` narrows**, flow-sensitive; ends at any write of the place, hidden writes included, and at any call that may run a closure writing a narrowed local; subclass test by an ancestor array; type index per generic instance; the references group's pool table and place-cast | (a) | one load and compare for a final class; a few loads and two compares otherwise (argued) | `isinstance` with an interface or an unparameterised generic class: proposed, not built | saved |
| A13 | generic methods through `dyn` | **a `dyn` table entry instantiated with `V = dyn I`**, for implementations that use `V` only through references; otherwise an error at the conversion to `dyn` | (a) | the indirect calls Rust's hand-written workaround makes | a type parameter returned or stored by value stays E2050 | saved |
| A16 | generics friction: bounds | **bounds inferred from the body**, in the package interface; a bound on an interface others cannot see is refused; proposed: undeclared method names as structural requirements, integer literals typed by use | (a) and (b) | none | the two proposals are the owner's to accept (a typo in a duck-typed name surfaces at the first call) | saved |
| A16 | generics friction: orphan rule | **a program nobody depends on may implement any interface for any type** | (b) | none | a dependency adding the same implementation breaks the program at upgrade | kept |
| A16 | generics friction: overlap | the most specific implementation wins | (b) | none | unproven; with separate compilation two implementations can run for one type | open |
