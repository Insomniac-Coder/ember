# Phase 3b, ownership and sharing: A1, A9, A10, A11, A24, A31, A34, A18, A21

Written 2026-10-24 by the ownership-and-sharing agent, under the owner's job (`JOB.md`): speed on par
with C and Rust, memory safety the same as Rust's or better, and Python's ease, all at once, with the
checking done when the program is compiled.

How every claim here can be traced:

- **Ember today** is real `ember check` output (`target/ci/ember.exe`). Every probe and its full output
  are in `target/redesign2/ownership/probes/` (outputs: `probes/ember_check_output.txt`). For two probes
  I also read the C that today's compiler writes, with `ember build --emit c --out-dir <dir>` (C files
  only: no C compiler, nothing run): `probes/a09_emit/` and `probes/a11_emit*/`. No Ember program was
  run.
- **Rust** is rustc 1.98.1. New probes and their output: `target/redesign2/ownership/rust_probes/`
  (`rustc_output.txt`). Older Rust facts come from phase 1, which names its files.
- **Python** behaviour quoted here was run: `target/redesign2/ownership/python/py_checks.py`, run with
  `python -I` (Python 3.14.2), output in `py_checks_output.txt` beside it.
- **Timings** are mine, made under the job's rules and marked PROVISIONAL: other agents were running,
  and the spread between runs was up to 33% for one program (section 4). A decision that rests on a
  difference under 15% waits for the final measurer.
- Numbers from earlier phases name their report and file. Nothing in `compiler/`, `std/`, `docs/` or
  `tests/` was touched.

---

## 0. The short version

Last night's three designs replaced compile-time checks with run-time ones. This report goes the other
way: every answer below is checked when the program is compiled, and the only run-time costs left are
ones Rust pays too, at the same places, and smaller. Three rules carry all nine annoyances:

1. **Field effects: "change anything through any name; never keep a pointer across a change that can
   free what it points to."** The compiler already works out, from each function's body, what the
   function does (Ember's effect sets, `[EFF-1]`, and the field sets of `[BRW-10]`), and already writes
   that into the package's interface (`[EFF-4]`, `[BLD-2]`). The rule adds one fact to it: which fields
   a function may *reshape*, that is, change in a way that can free or move memory (replace a list,
   push to it, change an enum's variant, drop an object). With that fact, every method borrows only
   what it uses, in every package and for classes too (A1); class objects need no run-time access
   words (A9); `mut`, `Cell` and `RefCell` are no longer needed (A10, A31); and a constructor may call
   a helper that only reads fields already set (A18).
2. **Names follow values.** `b = a` makes `b` a second name for the value, as in Python. Rebinding one
   name (`a = [9]`) leaves the other with the old value, as in Python, by moving the old value into
   the other name's storage instead of freeing it. Storing a value into a list or a field leaves the
   old name usable: it now names the place the value went (`root.children.push(child); child.value = 5`
   changes `root.children[0]`, as in Python). All of this is decided when compiling; nothing is counted
   or checked while the program runs (A11, A24).
3. **The lean class.** A class object keeps an 8-byte header (a count and a type index), no access
   words, and its count changes only when a handle is stored somewhere the compiler cannot see through
   (A9, A10, A34). `Box` is implied where a type needs it; `Shared`, `Cell` and `RefCell` go.

| Id | Annoyance | Best answer here | Kind | Cost when running | What is left |
|---|---|---|---|---|---|
| A1 | a method borrowing the whole object | field effects for every method, recorded in the interface | (b) smarter compile-time rule | none | a library body change can change what its callers may do (as Rust's inferred `Send` already does) |
| A9 | shared mutable state | `class` + field effects instead of run-time access words | (b) | the counts Rust's `Rc` also pays; measured 1.08x to 1.12x plain C in three batches, against 1.34x to 1.37x for safe Rust (PROVISIONAL) | a loop over a class's list across a call that may grow *that field of some object of that class* is refused, unless the containers group's loop rule applies |
| A10 | choosing among ownership kinds | two words, `struct` and `class`; `Box` implied; `Cell`, `RefCell`, `Shared` gone; a lint names classes that are never shared | (a) Pythonic spelling | none beyond A9's | the struct-or-class choice itself (copy or share) |
| A11 | moves, use after move, clone | names follow values | (b) | none | one value stored in two owning places needs `copy()` (or a class); a struct the program names, replaced inside a call, needs `copy()` |
| A24 | struct, Box or class decides what compiles and what panics | one rule set for both kinds | (b) | none | the kind still decides copy or share |
| A31 | caching in a read-only method | the Python spelling compiles (field effects), and `@cached` for computed values, invalidated automatically | (b) and (a) | the cache slot and one branch, as hand-written C | none found |
| A34 | clone before move for shared handles | already absent in Ember; unchanged | (a) | one count increment per capture, as Rust's `Arc::clone` | none |
| A18 | initialisation | a helper may run on a half-built `self` if it reads only fields already set | (b) | none | virtual calls stay conservative (as today) |
| A21 | updating a map entry in place | `counts.get(w, 0)`, `Map(default=0)`, and one lookup for `m[k] = m.get(k, d) + 1` | (a) | one lookup, as Rust's entry API | none found |

**Measured** (section 4; PROVISIONAL): a workload in which 1,048,576 watchers share 262,144 objects.
Median of 5 runs, pinned to the P-cores, mains power, ratios to plain C in the same batch:

| | time | peak memory |
|---|---|---|
| plain C (raw pointers) | 1.00 (286.337 ms) | 1.00 (57,352 KB) |
| the lean class (this proposal) | 1.08 | 1.00 |
| Ember today, as its compiler emits this program | 1.75 | 1.14 |
| Ember today, in a program where the access checks stay | 1.87 | 1.14 |
| safe Rust, `Rc<RefCell<Player>>` | 1.37 | 1.08 |
| safe Rust, `Rc<Player>` with `Cell` and one `RefCell` | 1.34 | 1.08 |

All eight programs printed the same checksum on every run.

**What this group needs from the others** (section 5): a loop over a list that survives a change to
that list made inside a call, without a run-time flag (the containers group, A2/A4); a name bound to a
list element that survives the list growing (containers); `cast` of a place being a place (the
references group, A5/A13); the `weak` field (the parent-pointer group, A8); and, for threads, that a
value two threads can reach is never changed and that counts of objects that cross threads are atomic
(the threads group, A15).

**Found in passing** (section 7): today's Ember silently accepts a write to a value after it was moved
(`root.children.push(child); child.value = 5`), which Rust refuses; the write goes to a dead local, so
the program prints the old value where Python prints the new one. And a struct that contains itself by
value (`left: Option[Tree]`) crashes the checker.

---

## 1. Words used in this report

- **Place**: somewhere a value lives: a local variable, a field (`g.log`), a list element (`xs[3]`), a
  map entry (`m[k]`), a field of an object reached through a handle (`cam.target.hp`).
- **Handle**: what a variable of a class type holds: a pointer to a heap object.
- **Count**: the number of handles to an object; the object is freed when it reaches zero (reference
  counting). Rust's `Rc` and Ember's classes both work this way.
- **Loan**: a pointer the compiled program holds into a place for a while: a `for` loop holds one into
  the list it walks, `ref xs[0]` holds one into the element.
- **Reshape**: a change that can free or move memory: replacing a field or variable whose value owns
  memory (`self.items = []` frees the old list), `push`, `insert`, `remove`, `clear` or `sort` on a list
  (a push may move every element to a bigger buffer), changing which variant an enum holds, and
  dropping the last handle to an object. Writing a plain number (`p.hp -= 1`) is not a reshape: it
  cannot free anything.
- **Effect summary**: what the compiler records about a function from its body: which fields it reads,
  writes and reshapes, which it keeps, and what it calls. Ember already computes effect sets this way
  (`[EFF-1]`) and puts them in the package interface (`[EFF-4]`); a method's field set is already
  there too (`[BLD-2]`).
- **Interface**: the summary of a package that its dependants are compiled against.

---

## 2. The three rules

### 2.1 Field effects: change anything through any name; never keep a pointer across a reshape

**What Rust's rule actually protects against.** Rust's "one writer or many readers" rule (`[BRW-1]`
in Ember) is there so that no pointer survives a change that frees or moves what it points to. Phase
1 checked each case: a `push` that moves the buffer a loop walks (A1, A2, A4), a field replaced while
someone holds a pointer into the old value (A9). A write to a plain number cannot free anything; Ember
already relies on this for class fields (`[EXC-17]`).

**The rule.** The compiler keeps Rust's protection and drops the rest:

1. A loan is broken only by a reshape of the place it points into (or of a place containing it).
   Anything else may be written through any name while the loan lives. A reshape that only *moves*
   elements (a list or map growing) breaks loans into the elements' own storage, not loans into memory
   the elements own on the heap (a `Box`'s contents, a `String`'s bytes), which does not move; only
   removing or replacing the element frees that. (This is the references group's "heap contents" fact,
   phase3b-references.md.)
2. Every function's effect summary says which places it may reshape: for its parameters and `self`,
   by field path (`self.log`); for objects reached through handles, by class and field ("reshapes
   `Player.inventory` of some `Player`"). It is computed from the body and the callees' summaries, over
   the call graph, as `[EFF-1]` computes effects today, and it goes into the interface for every
   function, `pub` ones included.
3. A loan used after something that may reshape its place is a compile-time error naming the line that
   may reshape it. For class objects, "may" means: any object of that class, because two handles can
   name one object.
4. A call through `dyn`, a function value, or a `virtual` method whose overrides are not all visible
   takes the effects its type declares, and all effects if it declares none: today's `[EFF-2]` rule.
   An interface or virtual method declares `mut self` when implementations may reshape `self`, as Rust's
   `&mut self` does.
5. A call may not reshape what its own arguments point into: the caller checks the callee's summary
   against the places it passes, as `[BRW-11]` does today for two `mut` arguments.
6. `mut` is inferred from the body. Writing it stays allowed and is checked. A place that may not
   change at all (a `static`, a value frozen to share with threads) cannot be passed where the callee's
   summary says it is written.

**What exists already.** `[EFF-1]` (effects computed from bodies over the call graph), `[EFF-4]`
(effects in the interface, callers re-checked when they change), `[BRW-10]` (a private method borrows
only the fields it uses), `[BLD-2]` (the interface hash already includes a method's field set). The new
parts are: the "reshapes" fact, class-wide effects for objects, and using the summaries across
packages.

**Why it is memory-safe** (single thread; threads are the threads group's, section 5):

- Memory is freed or moved only by a reshape. Every pointer the compiled program holds is a loan the
  checker already tracks. Every reshape is either written in the function or listed in a callee's
  summary, and the summary over-approximates: unknown code has every effect.
- Two handles may name one object, so effects on objects are by class and field, not by variable: a
  reshape of `Player.inventory` through any handle conflicts with a loan into `Player.inventory`
  through any handle.
- An object is not freed under a loan into it, because a loan through a handle keeps the object
  (today's `[RC-5]`); the retain it needs is left out only when no statement in the loan's life may
  drop an object of that class (rule 2 tells).
- A reshape that runs code of the program (a `drop` method run by `clear()`) has that code's effects
  in its own summary. The runtime must detach before it runs such code (take the buffer out, empty the
  list, then drop the elements), so a `drop` that reaches the list sees it empty.

**Attacks I tried, and what stops each** (each program is written the way Python would write it, so
the Python column says what Python does with the same lines):

| Attack | Program | Result | Python does |
|---|---|---|---|
| a loop over a list while a callee pushes to it | `for p in self.players: self.spawn()` where `spawn` pushes to `self.players` | refused at compile time: `spawn`'s summary says it reshapes `self.players` (or runs, under the containers group's loop rule, section 5) | the loop also visits the new players |
| the same through another handle to the same object | `for x in hud.target.inventory: cam.target.inventory.push(x)` | refused, or one compare (section 3.2): "reshapes `Player.inventory`" conflicts with the loan whatever the handles are | never ends (both names are one list) |
| a loan into an enum's payload, then the enum replaced through a second name | `h2 = h` then `match h.state: Walking(path): h2.state = State.Idle; path.pop()` | refused: `h2.state = …` writes the value `h` names (section 2.2) and changes its variant, and `path` points into the old variant | `path` still refers to the old list; `pop` works on it |
| an object freed while a loan into it lives | `x = a.target.items[0]; a.target = other; print(x)` | safe: the loan keeps the old object (`[RC-5]`); the retain stays because `a.target = other` may drop a `Player` | prints the old item |
| a library function holding a loan while calling a user callback | `items.retain(fn(x): other_name_for_items.push(x))` | refused: `retain`'s summary holds a loan into its receiver across calls to its callback, whose summary reshapes the same list | (no `retain`; the same with a comprehension over a list that grows never ends) |
| a `drop` method reshaping the list being cleared | `class Bomb: fn drop(self): WORLD.items.push(…)`, then `WORLD.items.clear()` | safe if the runtime detaches first (requirement above); the `push` lands in the emptied list | the same |
| two arguments that are the same object | `transfer(a, a)` where `transfer` walks `x.items` and pushes to `y.items` | refused at the call (rule 5) for values; for class handles, one compare `a is b` at the call when not provable (section 3.2) | never ends |

**Cost.** Nothing when running: it is a compile-time check. It *removes* run-time work today's Ember
does: the per-field access words of classes (8 bytes in the measured `Player`, `[EXC-19]`) and their
checks where they were not proved away (section 4). Value parameters keep their "not the same place"
guarantee, checked by the caller (rule 5), so the C and LLVM backends may still assume they do not
overlap, as Rust's `&mut` lets LLVM assume. Fields of class objects get no such assumption, as today
(`[EXC-17]`) and as two Rust `Rc`s get none.

**What it gives up.**

- A `pub` method's summary is part of its interface, so a library that changes a method's body to
  touch a new field can stop a dependant from compiling. `[BRW-10]` limits field precision to private
  methods to avoid exactly this. Two facts weigh against that limit. Ember already does it for effects:
  `[EFF-4]` re-checks a caller's contracts (`@noalloc`, say) when a callee's effects change. And Rust
  does it for `Send`: whether a returned closure is `Send` is inferred from the body, and a dependant
  crate that needs `Send` stops compiling when it changes. Probe: `rust_probes/leak_lib.rs` (a library
  returning `impl Fn()` that captures an `Rc`) and `leak_use.rs` (a second crate asking for `Send`):
  `error[E0277]: Rc<usize> cannot be sent between threads safely`, pointing into the library's body
  (`rust_probes/rustc_output.txt`). The package tool should list summary changes when a version is
  published, as it would list a changed signature.
- Some programs today's Ember accepts with a run-time check become compile-time errors (section 3.2),
  unless the containers group's loop rule covers them.

### 2.2 Names follow values

**What Rust's rule protects against.** A value owned by two places would be freed twice; a moved-from
variable would read memory its new owner may already have freed (phase 1, A11).

**The rule.** Ownership stays unique: a value lives in exactly one place. What changes is what a
*name* is.

1. `x = place` makes `x` a name for the value the place holds now. No copy, no move, no count.
   Values of a `Copy` type (numbers, `bool`, and plain structs their author marked `@derive(Copy)`)
   are copied instead, as today and as Python's numbers behave (they never change in place); a copy
   that is changed and never read again gets a warning (the copy-instead-of-share design's E-COPYLOST
   idea, phase3-design-D1.md). `x = something_new` later rebinds the name `x`; it never writes the
   place `x` named, which is Python's meaning (so `for p in players: p = Player()` leaves the list
   alone, the trap phase3-break-D1.md N2 found in that design). Writing *through* the name
   (`x.hp -= 1`, `x.push(…)`) changes the value it names.
2. If the place gets a new value while `x` is still used, `x` keeps the old value. In the same
   function, the compiler moves the old value into `x`'s own storage at that line instead of dropping
   it; for a class handle, the count the place held passes to `x`. If the change can happen inside a
   call (the call's summary says it reshapes the place), a class handle is retained before the call
   (one count pair: Rust's `Rc::clone`); a struct or list value cannot be kept without a copy, so that
   use is a compile-time error whose fix-it is `x = copy(place)`.
3. When a value is stored into another place (pushed to a list, assigned to a field, put in a map,
   passed to a parameter declared `owned` whose summary says where it goes), its old names name the new
   place, as long as the compiler can see the value is still there. Growth of the list does not end
   this (the containers group's rule for element names, section 5); a removal that shifts elements, a
   clear, or a store into a place the compiler cannot follow does, and a later use of the name is a
   compile-time error with fix-its.
4. A value is in one owning place at a time. Storing it into a second one is a compile-time error
   whose fix-its are `copy(x)` or declaring the type a `class`.

**What it looks like** (each Python block was run, `py_checks_output.txt`):

```python
items = [1, 2]; snapshot = items; items.append(3); print(snapshot)     # [1, 2, 3]
a = [1]; b = a; a = [9]; print(b)                                      # [1]
root = Node(0); child = Node(1); root.children.append(child)
child.value = 5; print(root.children[0].value)                         # 5
matrix = []; row = []; matrix.append(row); row.append(1); print(matrix)  # [[1]]
```

```ember
items = [1, 2]; snapshot = items; items.push(3); println(snapshot)      # proposed: [1, 2, 3]   today: E3040
a = [1]; b = a; a = [9]; println(b)                                     # proposed: [1]         today: [1] (b = a moves)
root = Node(value=0); child = Node(value=1); root.children.push(child)
child.value = 5; println(root.children[0].value)                        # proposed: 5           today: 1, silently
matrix = []; row = []; matrix.push(row); row.push(1); println(matrix)   # proposed: [[1]]       today: E3040
```

(Today's results: `probes/a11_names.em`, `a11_rebind.em`, `a11_store_then_use.em`; the "silently" is
section 7.)

**What the compiler emits.** Nothing for rule 1: `snapshot` is compiled as `items`. For rule 2 in one
function, a move instead of a drop:

```c
/* a = [1]; b = a; a = [9]; println(b)   -- written by me: what the proposal emits */
ember_vec a = vec_of_1();          /* b names a: no code */
ember_vec b = a;                   /* a is about to be replaced while b is still used: the old value moves to b */
a = vec_of_9();                    /* instead of: drop(a); a = vec_of_9(); */
print_vec(&b);
```

For rule 3, the index the value went to:

```c
/* root.children.push(child); child.value = 5   -- written by me: what the proposal emits */
size_t child_at = root.children.len;                       /* where push puts it */
ember_vec_push(&root.children, sizeof(Node), &child);
((Node *)root.children.ptr)[child_at].value = 5;           /* child names root.children[child_at] */
```

Today's compiler, for the same lines (`probes/a11_emit/c/debug/c/main.c`, read, not run):

```c
ember_vec_push(&root.children, sizeof(em_4Node), &child);
child.value = 5LL;                 /* writes the moved-from local: root.children[0] keeps 1 */
```

The proposal's code is the indexed store a C programmer writes after a push: no count, no check (the
index is in range because nothing removed from the list since the push; a growth only moves the
buffer, which the index is re-read against).

**Why it is memory-safe.** No value is ever owned by two places (rule 4), so nothing is freed twice.
A name is a compile-time path to a place, plus at most an index or a pointer the compiler keeps, and
the loan rules of 2.1 decide whether that place still holds the value at each use; where they cannot
tell, the program is refused. It is Rust's ownership with the `&` made implicit and the "use after
move" error replaced by "the name went with the value".

**Attacks I tried:**

| Attack | Program | Result | Python does |
|---|---|---|---|
| the named place is replaced in the same function | `old = p.name; p.name = new; log(old)` (the checked-pointers design's worst case, phase3-break-D2.md N1) | works as Python: the old `String` moves into `old` at `p.name = new` | logs the old name |
| the drain idiom | `pending = self.pending; self.pending = []; for job in pending: job.run()` | works: the old list moves into `pending`; `job.run()` changing `self.pending` touches the new list | the same |
| the named place is replaced inside a call | `w = p.weapon; p.upgrade(); print(w.name)` | a struct: refused, fix-it `w = copy(p.weapon)`; a class handle: retained before the call, prints `bow` | prints `bow` (`py_checks_output.txt`, `name_keeps_old_after_call`) |
| the list the value went into is replaced | `items.push(n); items = []; n.value = 1` | refused: "`n` was in `items`, which line N replaced" (fix-its: `n = items.pop()`, or use `n` before) | changes the node, which is no longer in any list |
| stored twice | `a.push(m); b.push(m)` (struct) | refused: rule 4; fix-its `b.push(copy(m))` or `class` | both lists hold one object |
| the same name stored on every loop pass | `for _ in range(n): v.push(template)` | refused: rule 4 | `v` holds n names for one object, a known Python trap (`py_checks_output.txt`, `append_same_n_times`: `[7, 7, 7]`) |
| stored on one path only | `if c: matrix.push(row)` then `row.push(1)` | accepted: `row` is a pointer to whichever place holds it, valid while the loan rules say so | the same |
| given to a function that keeps it somewhere unknown | `send(m); log(m)`, `send` declared `owned` with no destination in its summary | refused, fix-its `log(m)` first, or `send(copy(m))` | both see one object |

**Cost.** None when running (the emitted code above). The old value lives a little longer when a name
keeps it (until that name's last use), which is Python's lifetime for it.

**What happens to programs that compile today.** As far as I can find, none changes meaning: every
line whose meaning changes (`b = a` then using `a`, `n = p.name` then using `p`, `matrix.push(row)`
then `row.push(1)`) is refused today, and `Copy` values keep copying. The one exception is the defect of
section 7 (`child.value = 5` after a push), whose meaning changes from "lost" to Python's. This is my
reading of the rules, not an exhaustive search: UNVERIFIED.

**What is left.** Rules 2 (inside a call, for struct values) and 4 are the places where Python shares
one object and a value cannot. They are compile-time errors with fix-its, never silent copies.

### 2.3 The lean class

**What changes for class objects:**

- **Header 8 bytes**: a 32-bit count and a 32-bit type index (for virtual calls, casts and dropping
  through an interface). Today's 24 bytes (`[OBJ-1]`: strong, weak, access, flags, type pointer) shrink
  because: the access word goes (field effects); the weak count moves to a side record made the first
  time an object gets a weak handle (Swift does this, phase2-swift-mojo.md, S7); the flags fold into
  the top bits of the count or into that side record.
- **No access words**: today a class with one non-`Copy` field carries 8 bytes of them (one word plus a
  zero word, read from the emitted `struct em_obj_6Player` in `probes/a09_emit/c/release/c/main.c`).
- **Fewer count updates**: a handle read from a place into a local is not retained when no statement
  in the local's life may write that place or drop an object of that class (rule 2 of 2.1 tells). Today
  `[RC-2b]` guarantees this only within one expression, and the compiler emitted a retain and a release
  for `t = ws[i].target` on every loop step of the measured program (same file).
- `Box`, `Shared`, `Cell` and `RefCell` stop being things a programmer chooses (A10).

**Cost**, measured in section 4: 1.08x plain C in time (1.12x and 1.09x in two later batches) and the
same peak memory, where today's emitted code is 1.75x and 1.14x and safe Rust is 1.34x to 1.37x and
1.08x (PROVISIONAL). Taken apart in the first batch: the smaller object is worth 28% of the time, the
dropped per-step count pair 13.5% to 13.9%. In the third batch the lean class was 1.09x and the same
object with no counts at all 1.03x, so most of what is left is the count update when a handle is
overwritten.

---

## 3. The annoyances one by one

### 3.1 A1: a method borrowing the whole object

**What Rust's rule protects against** (phase 1, checked there with rustc): in

```rust
for p in self.players.iter_mut() { if p.hp <= 0 { self.note(...) } }   // E0499
```

`note` takes all of `self`, so as far as the caller knows it could push to `self.players` and move the
buffer the loop is walking; the next step would read freed memory.

**The Python program and Ember today.**

```python
def note(self, msg): self.log.append(f"[{self.frame}] {msg}")
def tick(self):
    for p in self.players:
        p.hp -= 1
        if p.hp <= 0: self.note(f"{p.name} died")
```

Ember accepts this for a struct when `note` is private or in the same package (`[BRW-10]`;
`probes/a1_one_package`: accepted). It refuses it across packages, which phase 1 could not test and is
now verified (`probes/a1_two_packages`, a `gamelib` package with `pub fn note`, and an `app` package):

```text
error[E3020]: cannot mutate `*g.players` while it is borrowed by this loop
5 |     for p in g.players.iter_mut():
8 |             g.note(f"{p.name} died")
```

and refuses the class version whose `note` says `mut self` (phase 1's `e01d_whole_object_class_mut.em`,
E3020 again: a `mut self` class method holds every field, `[EXC-15]`).

**Answers.**

- **(b) smarter compile-time rule: field effects (2.1).** The code is the Python code:

  ```ember
  pub fn note(self, msg: String):            # interface records: reshapes self.log; reads self.frame
      self.log.push(f"[{self.frame}] {msg}")
  pub fn tick(self):
      for p in self.players:
          p.hp -= 1
          if p.hp <= 0:
              self.note(f"{p.name} died")    # accepted: note does not reshape self.players
  ```

  The guarantee is Rust's (no pointer into `players` survives a reshape of `players`), checked against
  `note`'s summary instead of its signature. It works in another package, for a class, and with `mut`
  left out. Cost: none when running. Safety: section 2.1. What it gives up: section 2.1 (a library body
  change can change its callers' verdict).
- **(a) Pythonic spelling: declared field sets**: the method says which fields it uses,
  `fn note(mut self{log})` (an idea from Rust's design discussions, "view types"; from memory and not
  in rustc 1.98.1: UNVERIFIED). Same guarantee, no inference across packages. Not chosen: it is a new thing
  to write, and Python writes nothing. Good part kept: an author who wants a `pub` method's summary
  frozen can state it (`@effects(changes=[log])`), and the compiler checks the body against it.
- **(c) new mechanism: loops that cannot be invalidated** (the loop walks by index and re-reads the
  list after a call that may change it; phase 1 A1(b), Swift's cursor, phase2-swift-mojo.md idea 1).
  It belongs to the containers group. For A1 it is not needed when the summary proves no conflict; it
  is what makes the remaining cases work (a `dyn` or `virtual` call whose effects are unknown).

**Choice: field effects.** Zero cost, the Python text, every package.

### 3.2 A9: shared mutable state

**What Rust's rule protects against** (phase 1): one path replacing or growing part of an object while
another path holds a pointer into that part. In phase 1's program, pushing to `inventory` through the
camera while the HUD loops over it would read a freed buffer. Rust's safe answer is `Rc<RefCell<T>>`:
counts, plus a borrow flag checked at run time on every borrow, which locks the whole value (phase 1 ran
it: `RefCell already borrowed` on two different fields).

**The Python program and Ember today.**

```python
p = Player(); cam = Camera(p); hud = Hud(p)
cam.target.hp -= 1
hud.target.inventory.append("key")
for item in hud.target.inventory:         # 1. a number changed during the loop
    cam.target.hp -= len(item)
for item in hud.target.inventory:         # 2. another list of the same object
    cam.target.log.append(item)
for item in hud.target.inventory:         # 3. the same list, through the world
    world.give_to_someone(item)
```

Ember accepts all three with classes (`probes/a09_fields.em`, no error) and leaves loops 2 and 3 to
run-time access words: the emitted C begins a read access on `Player.inventory`'s word around each
loop (`ember_field_begin_read`), a write access on `Player.log`'s word for loop 2
(`ember_field_begin_write`), and checks `Player.inventory`'s word in `give_to_someone`
(`ember_field_check_write`) (`probes/a09_fields_emit/c/release/c/main.c`), so loop 3 panics with an
exclusivity violation (by `[EXC-1]`; not run). The struct twin (`probes/a09_struct_twin.em`) accepts loops 1 and 2 and refuses
the direct form of 3 at compile time (E3020).

**Answers.**

- **(a) Pythonic spelling: `class` is the one word for a shared object.** Ember already has it: no
  `Rc`, `RefCell` or `borrow_mut` in the source. Kept.
- **(b) smarter compile-time rule: field effects replace the access words.** Same source as Python.
  Loop 1: `hp` is a number, nothing reshaped: accepted, no check (as today). Loop 2: `log` is reshaped,
  the loan is on `inventory`: different fields: accepted, no check (today: a run-time word per field).
  Loop 3: `give_to_someone`'s summary says "reshapes `Player.inventory` of some `Player`", which may be
  this one: the loop's loan conflicts. What happens then depends on another group:
  - if the containers group adopts a loop that cannot be invalidated (it re-reads the list after such a
    call), loop 3 compiles and behaves as Python's does (it sees the pushed items, and here never ends,
    as Python's never ends: `py_checks_output.txt`, `same_object_grows_while_iterating`);
  - otherwise it is a compile-time error: "`give_to_someone` may push to any `Player`'s
    inventory, including `hud.target`'s, which this loop is reading", with the fix-it
    `for item in copy(hud.target.inventory):`.

  When both handles are named locals (`for it in a.items: b.items.push(it)`, a transfer between two
  objects), the conflict is exactly "`a` and `b` are the same object": the compiler emits one
  comparison, `if a is b: panic(…)`, before the loop. That is the job `RefCell` does in Rust, at one
  compare per loop instead of a flag write per borrow; Python's transfer works when the objects differ
  (`py_checks_output.txt`, `transfer`).

  Cost, measured (section 4): 1.08x plain C (1.12x and 1.09x in two later batches), against 1.75x for
  today's emitted code and 1.37x for Rust's `Rc<RefCell<Player>>` (PROVISIONAL). Memory: the same peak as plain C here, against 1.14x today and
  1.08x for Rust.
- **(c) new mechanisms considered:**
  - *Checked pointers instead of counts* (each reference carries a generation, compared on use; the
    checked-pointers design, phase3-design-D2.md, and Vale). Measured in phase 2: +0.57 ns (+23%) per
    in-order use, +4.61 ns (+45%) random, 8 more bytes per reference
    (`phase2-vale-austral-pony.md` section 6). Try to save it: last night's version already removed
    most checks by proofs and still measured 1.15x to 1.45x plain C on three workloads
    (`phase3-measurements.md`, section 5), and the object dies with its one owner, so
    `old = p.name; p.name = new; print(old)` panics (phase3-break-D2.md N1). The lean class pays a count
    only where a handle is stored, and never panics on that program. **Discarded for sharing**; good
    part kept: a generation compare is a cheaper `Weak.upgrade` (for the parent-pointer group).
  - *Arenas* (objects of one lifetime freed together; no counts). Fast and safe, but no single object
    can be freed early. Already in Ember as a library (`Arena`, `[ARN-*]`); kept as a library, not as
    the answer to A9.
  - *Keeping today's access word only on fields that need it.* The layout of a class would then depend
    on code anywhere in the program (the "never an error" design's problem, phase3-break-D3.md 2.8).
    **Discarded.**

**Choice: `class` plus field effects** (a + b), with the residue stated above.

### 3.3 A10: choosing among ownership kinds

**What Rust's rule protects against** (phase 1): freeing a value twice or never. Two entities sharing a
`Box<Mesh>` would both free it; so Rust makes the programmer pick `Box`, `Rc`, `RefCell`, `Arc`, `Mutex`,
`Weak`, and change every type that mentions the field when the choice changes.

**The Python program and Ember today.**

```python
m = Mesh([0.0] * 9)
a, b = Entity(m), Entity(m)
a.mesh.verts.append(1.0)
```

Ember compiles the `class MeshC` and `Shared[Mesh]` versions and refuses the `Box` one (phase 1
`a10_ownership_kinds.em`, E3040, re-run: `probes/ember_check_output.txt`). The spec's table still lists
eleven mechanisms (`[SEL-1]`, section IX.0).

**Answers.**

- **(a) Pythonic spelling: two words, the rest implied.**

  | Today (`[SEL-1]`) | Proposed |
  |---|---|
  | `struct`, `enum`, tuple; `Array`, `Map`, `Set`, `String` | values, as today |
  | `class` | shared objects: the lean class (2.3) |
  | `Box[T]` | implied: a field whose type contains itself (`left: Option[Tree]`) is stored behind a pointer, as Swift's `indirect` but unwritten; `dyn Shape` in a field or list is boxed without writing `Box` |
  | `Shared[T]` | a class with one field |
  | `Cell[T]`, `RefCell[T]` | not needed: any field may be written through any name (2.1) |
  | `Weak[O]` | a `weak` field (the parent-pointer group decides it) |
  | `SyncShared[T]`, `Mutex`, `RwLock`, `Atomic` | the threads group |
  | `Arena`, `Pool` + `Handle` | libraries, for programs that want them |

  ```python
  class Tree:                          # Python
      def __init__(self, value, left=None): self.value = value; self.left = left
  ```

  ```ember
  struct Tree:                         # proposed: no Box
      value: int
      left: Option[Tree] = None
  ```

  Today this crashes the checker (`probes/a10_box_inferred.em`: `thread 'main' has overflowed its
  stack`, exit 127; section 7). Cost: the same allocation `Box` makes (Rust's `Box` is the same), so
  zero against writing `Box`. What it gives up: the allocation is not visible in the type;
  `ember inspect --alloc` (`[SEL-1]`) shows it.
- **(b) smarter: one kind, the compiler picks the representation** (phase 1 A10(b), A24(a)). The
  Python programmer writes `class` for everything, and a class whose objects are never shared would be
  stored inline. Try to save it: the obstacle is that "never shared" depends on every function a handle
  reaches, so a distant change (one `self.target = p` in another module) silently changes the layout
  and the speed (the same problem as phase3-break-D3.md 2.8). Making the decision per container does not
  help: a handle taken from the container can still be stored anywhere. **Saved as a lint instead**: the
  compiler, which has the summaries, says "`class Particle` is never shared; as a `struct` it is stored
  inline". The size of the stake is measured: counted objects instead of inline structs cost 3.56x on
  iteration (9.74x scattered) and 2.44x memory (`phase2-rc-elision.md`, `boxed_iter` vs `inline_iter`).
  With names following values (2.2), changing `class` to `struct` changes almost no other line.
- **(c) none better found.**

**Choice: two words plus the lint.** What is left: the choice between copy (`struct`) and share
(`class`). Python has it too, between numbers, strings and tuples (which never change in place) and
objects; it is a choice of meaning, not of machinery.

### 3.4 A11: moves, use after move, clone

**What Rust's rule protects against** (phase 1): two owners of one heap buffer would both free it; a
moved-from variable would read memory its new owner may already have freed.

**The Python program and Ember today.** Phase 1's four (`e11_moves.em`): `send(m); log(m)` (E3040),
`n = p.name; …; keep(p)` (E3042), `for _ in 0..n: v.push(template)` (E3041), `snapshot = items;
items.push(3)` (E3040). New ones, `probes/a11_names.em`: `matrix.push(row); row.push(1)` (E3040), and
`root.children.push(child); child.value = 5`, which compiles and loses the write (section 7). Rust
refuses that last one:

```text
error[E0382]: assign to part of moved value: `child`
6 |     root.children.push(child);
  |                        ----- value moved here
7 |     child.value = 5;
  |     ^^^^^^^^^^^^^^^ value partially assigned here after move
```

(`rust_probes/a11_assign_moved_field.rs`, `rustc_output.txt`.)

**Answers.**

- **(b) smarter compile-time rule: names follow values (2.2).** Every Python line above compiles and
  means what it means in Python, except where Python shares one object between two owners (stored
  twice, stored on every loop pass, given to a function that keeps it somewhere unknown, replaced
  inside a call), which are compile-time errors whose fix-it is `copy(x)` or a class. For `n = p.name;
  keep(p)`, `n` names the field; `keep(p)` moves `p` after `n`'s last use, so it is accepted; if `n` were
  used after, the error would say "`n` named `p.name`, which went to `keep()` at line N". Cost: none
  (2.2's emitted code).
- **(c) copy on write** (lists and dicts as values that share a buffer until one is changed; the
  copy-instead-of-share design, phase3-design-D1.md; Swift; Roc). Measured in phase 2: +10% on random
  writes for the uniqueness test (`unique_set` 103.7 ms against `inline_set` 94.5 ms), and a hidden copy
  of 17.8 ms and twice the peak memory when a shared 64 MiB list is first written (`cow_first_set`;
  `phase2-rc-elision.md`). Try to save it: share only where a move would otherwise be refused
  (phase2-rc-elision.md idea 1), so programs that compile today pay nothing. What no fix removes: the
  meaning. `snapshot = items; items.push(3)` gives `snapshot` the old list, not Python's, and
  `matrix.append(row); row.append(1)` loses the `1` silently (phase3-break-D1.md N1, which no compile-time
  check of that design caught). **Discarded.** Good part kept: "where the compiler proves the move,
  nothing changes" is also how names-follow-values costs nothing.
- **(c) the old name stays a checked reference** (the checked-pointers design's rule that a stored
  owner's old name stays a reference, phase3-design-D2.md section 1.3): the same idea as rule 3 of 2.2,
  but checked while running, and panicking when the owner lets go (phase3-break-D2.md N1). **Discarded** for the run-time check; the idea itself is
  kept, made static.
- **(c) move at the last use, copy at earlier uses** (phase 1 A11(b)). Hidden allocations, and the
  meaning changes (a write to the copy is lost). **Discarded as the default**; kept as a fix-it the
  programmer accepts by writing `copy(x)`.

**Choice: names follow values.** What is left is stated in 2.2.

### 3.5 A24: struct, Box or class decides what compiles and what panics

**What Rust's rule protects against.** Nothing of its own: Rust has one model. The annoyance is
Ember's: two models whose rules differ (phase 1).

**Ember today**, the same lines with the two kinds:

| Lines | `struct` | `class` | Probe |
|---|---|---|---|
| a loop over one list pushes to it through another path | E3020 at compile time | compiles; panics at run time (`[EXC-1]`) | `a09_struct_twin.em`, `a09_fields.em` |
| a loop over one field while another field is pushed to | accepted | accepted, with a run-time word per field | same |
| one mesh stored in two lists | E3040 | compiles; both share it | `a24_two_places.em` |
| a reference returned from a pool found through a map | `Box` pools compile | E3060 | ODR-286's `ecs5/box1.em`, `ecs5/pos.em` (re-run) |

**Answers.**

- **(b) one rule set for both kinds.** Field effects (2.1) and names (2.2) apply to structs and classes
  alike, and the class's run-time access words go. Row 1: both refused at compile time, or both
  accepted under the containers group's loop rule. Row 2: both accepted, nothing checked. Row 4: with
  names binding places, `pool = cast self.pools[k] as Pool[T]` names the place `self.pools[k]`, so the
  returned reference is a loan of `self` in both kinds; this also needs `cast` of a place to be a place
  (the references group; Hylo does this, phase2-hylo-csharp.md H2). Row 3 stays different, because it is
  the meaning of the kind: a value is in one place, an object can be in many:

  ```python
  m = Mesh([1.0]); a.append(m); b.append(m)      # Python: both lists hold one mesh
  ```

  ```ember
  m = MeshS(verts=[1.0]); a.push(m); b.push(m)   # struct: refused, fix-its b.push(copy(m)) or `class`
  m = MeshC(); a.push(m); b.push(m)              # class: both lists hold one mesh, as Python
  ```

  Cost: none.
- **(a) one declaration, representation chosen by the compiler**: A10(b), saved as a lint (3.3).
- **(c) none found** beyond the two above.

**Choice: one rule set.** What is left: copy or share (row 3), which is the reason to have two kinds.

### 3.6 A31: caching in a read-only method

**What Rust's rule protects against** (phase 1): writing a value other readers may be looking at. For
a plain number on one thread nothing can dangle (Ember's own `[EXC-17]` argument). The real hazards are a
cache that owns memory (replacing a cached `String` frees what a reader may point into) and two threads
writing at once.

**The Python program and Ember today.**

```python
@functools.cached_property
def radius(self): return max(abs(v) for v in self.verts)
```

or by hand, `if self._radius is None: self._radius = …`. Ember refuses the struct version (`E3023:
cannot mutate borrowed parameter self`, phase 1 `a31_lazy_cache.em`), accepts the class version, and
declaring `mut self` moves the refusal to every caller that only borrows the mesh
(`probes/a31_mut_inferred_caller.em`: `E3023: cannot mutate borrowed parameter m` in `show`). `@cached`
does not exist (`probes/a31_cached.em`: `E0104`).

**Answers.**

- **(b) smarter compile-time rule: the Python spelling compiles.** Under field effects a method may
  write any field through any name; only a reshape that breaks a live loan is refused. Writing the
  `Option[f32]` cache breaks nothing, so phase 1's struct `radius` compiles as written, also when the
  caller only borrows the mesh. A cache that owns memory (`Option[String]`) is a reshape: refused only
  if a caller holds a pointer into that cache across the call. Cost: none. Threads: a value two threads
  can reach must not be written, and the summary says `radius` writes; the threads group's rule refuses
  that sharing, as Rust's `Cell` makes a type not `Sync`.
- **(a) Pythonic spelling: `@cached`**, Python's `functools.cached_property` idea:

  ```ember
  struct Mesh:
      verts: Array[f32]

      @cached
      fn radius(self) -> f32:
          return max(abs(v) for v in self.verts)
  ```

  The compiler adds a hidden slot, fills it on the first call, and empties it at every write to a field
  the method read (`verts`; the summary says which). That is better than Python, whose cached value
  goes stale: after `m.verts.append(-5.0)`, Python's `m.radius` still says `2.0`
  (`py_checks_output.txt`). Because the slot is written only when empty, it never frees anything, so
  `@cached` also works through an interface method declared without `mut self`, and returning a
  reference into the cached value is safe until a field it read changes. What it emits, written by me:

  ```c
  float Mesh_radius(Mesh *self) {
      if (!self->radius_set) { self->radius = compute_radius(self); self->radius_set = 1; }
      return self->radius;
  }
  /* and at every write to self->verts: */  self->radius_set = 0;
  ```

  That is the lazy cache a C programmer writes by hand, so the cost is the slot (8 bytes for an
  `Option[f32]`) and one branch per call, the same as hand-written C and as Rust's `OnceCell`. Shared
  across threads it needs a once-flag (the threads group).
- **(c) what Rust does: `Cell` or `OnceCell` in the field type.** A wrapper type to choose; not Python.
  Not chosen.

**Choice: both** (the plain spelling compiles; `@cached` for computed values).

### 3.7 A34: clone before move for shared handles

**What Rust's rule protects against** (phase 1): one count serving two owners, so a double release
when both threads finish. Rust makes the programmer write `Arc::clone(&log)` before each closure.

**The Python program and Ember today.**

```python
threading.Thread(target=lambda: log.append("a")).start()
threading.Thread(target=lambda: log.append("b")).start()
```

Ember already compiles the Python shape (`a34_clone_dance.em`, re-run: accepted): copying a handle
retains (`[RC-1]`), so each capture takes its own count.

```ember
fn start(log: Log):                         # Log is a @sync class
    a = thread.spawn(owned fn() => push(log, "a"))
    b = thread.spawn(owned fn() => push(log, "b"))
    a.join()
    b.join()
```

**Answers.** (a) is already in Ember. Nothing in this group changes it, except that the count lives
in the lean 8-byte header. Cost: one count increment per capture, atomic for an object two threads
share, which is Rust's `Arc::clone` (phase 2 measured an atomic pair at about 5.8 ns against 0.84 ns
plain, `phase2-rc-elision.md`); scoped tasks borrow with no count (`[THR-11]`). Which counts are atomic
is the threads group's. One (b) answer belongs there too: Koka marks an object as shared between threads
only when it is sent to another thread, so `@sync` need not be written and one-thread objects keep plain
counts (phase2-rc-elision.md, idea 2). No (c) answer found: nothing cheaper than a count keeps a shared
object alive for as long as its last holder needs it, without a garbage collector.

**Choice: keep.**

### 3.8 A18: initialisation

**What Rust's rule protects against** (phase 1): a method reading a field that was never set reads
garbage.

**The Python program and Ember today.**

```python
def __init__(self, width):
    self.renderer = Renderer(width)
    self.assets = self.load_assets()          # load_assets only reads self.renderer
```

`probes/a18_init_variants.em`: the Python spelling is `E2100: self is used before all struct fields are
initialized`; passing the field to a free function (`load_assets_for(self.renderer)`), Rust's
workaround, compiles; giving every other field a default compiles.

**Answers.**

- **(b) smarter compile-time rule: check what the helper reads, not "self as a whole".** A method may
  run on a half-built `self` if its summary reads only fields already set, writes only those or the
  ones it sets, and does not let `self` escape (store it, pass it whole to code with unknown effects,
  capture it). A `virtual` or `dyn` call stays refused until every field is set (`[CLS-11]`'s rule).
  The Python code compiles as written:

  ```ember
  struct Game:
      renderer: Renderer
      assets: Array[String]

      fn init(self, width: int):
          self.renderer = Renderer(width=width)
          self.assets = self.load_assets()     # accepted: load_assets reads only self.renderer

      fn load_assets(self) -> Array[String]:
          return [f"atlas@{self.renderer.width}px"]
  ```

  Cost: none (a compile-time check).
- **(c) every field starts as zero** (the checked-pointers design's A18 answer). Try to save it: zero is
  not a valid value for a non-null reference or a range type, and the breaker showed it breaks range
  facts that remove bounds checks (phase3-break-D2.md H8); reading an unset number silently gives 0
  where Python raises `AttributeError` (`py_checks_output.txt`, `read_before_set`). **Discarded.**

**Choice: check the helper's reads.**

### 3.9 A21: updating a map entry in place

**What Rust's rule protects against** (phase 1): a lookup that hands back a reference, then an
insertion while that reference lives; inserting may rehash and move the values.

**The Python program and Ember today.** `probes/a21_spellings.em`:

| Spelling | Python | Ember today |
|---|---|---|
| `counts[w] = counts.get(w, 0) + 1` | works | `E2020 … get does not take these arguments`, help: "write `get_or`" |
| `counts[w] = counts.get_or(w, 0) + 1` | — | compiles |
| `counts[w] = counts.get(w).unwrap_or(0) + 1` | — | `E2020: expected ref i64, found an integer` |
| `counts.entry(w).or_default() += 1` | — | `E2140: assignment target is not a place` (phase 1's defect 3) |
| `counts[w] += 1` on a missing key | `KeyError: 'a'` (`py_checks_output.txt`) | panics, as Python raises (`index_mut` in `std/src/collections.em`) |

**Answers.**

- **(a) Pythonic spelling.** `get(k, default)` returns the value when the value type is `Copy`
  (Python's `dict.get`); `Map(default=0)` makes `counts[w] += 1` insert the default (Python's
  `defaultdict(int)`); `unwrap_or(0)` on an `Option[ref int]` takes an `int`; E2140 fixed.

  ```python
  counts = defaultdict(int)
  for w in text.split(): counts[w] += 1
  ```

  ```ember
  counts = Map[String, int](default=0)
  for w in text.split():
      counts[w] += 1
  ```

  Memory safety: no reference is held across an insertion in any of these, as today.
- **(b) smarter: one lookup for `m[k] = m.get(k, d) + 1`.** Today `get_or` and `index_set` each run
  `find` (`std/src/collections.em`, `get_or` at line 811 calls `get`, and `index_set` calls `find`), so the
  Python spelling hashes and probes twice. When the key expression is the same and nothing between the
  two touches `m`, the compiler can emit one find-or-insert and an add in place, which is what Rust's
  `*counts.entry(w).or_insert(0) += 1` does. `counts[w] += 1` on a default map is one lookup already.
  Cost: one lookup, as Rust (no measurement needed: same operations).
- **(c) none needed**: nothing is unsafe here once no reference outlives the statement.

**Choice: both.**

### 3.10 Against the three goals

| Id | Speed (on par with C and Rust) | Memory safety (Rust's or better) | Python's ease | Verdict |
|---|---|---|---|---|
| A1 | met: nothing runs | argued (2.1) and attacked, not proven | met: the Python text | meets all three |
| A9 | 1.08x to 1.12x plain C, 1.34x to 1.37x for safe Rust: on par with C pending the final measurer (under 15%), faster than Rust | argued and attacked, not proven; single thread only | met, except the residue of 3.2 (a compile-time error where Python loops) if the containers group's loop rule is not adopted | meets all three if the containers rule comes; otherwise lacks ease in that one shape |
| A10 | met (A9's cost) | as A9 | two words instead of eleven; the struct-or-class choice remains | lacks ease only in that a choice remains, and it is a choice of meaning |
| A11 | met: nothing runs | argued and attacked (2.2), not proven | met except stored twice, stored every pass, kept somewhere unknown, replaced inside a call (struct): errors with fix-its | lacks ease in those four shapes, each of which Python can only do by sharing |
| A24 | met | as A9 and A11 | met: only copy-or-share differs | meets all three |
| A31 | met: the cost of hand-written C | argued; threads left to the threads group | met, and `@cached` is better than Python's (no stale value) | meets all three |
| A34 | met: Rust's cost | as today | met (already) | meets all three |
| A18 | met: nothing runs | argued | met | meets all three |
| A21 | met: one lookup | unchanged | met | meets all three |

"Argued and attacked, not proven" means: the argument is in section 2, the attacks I tried are listed,
and no adversary has attacked it yet. Last night every design that was attacked had holes; this one
should be attacked before it is believed.

---

## 4. The measurement

**What it measures.** The lean class (2.3) against plain C, today's emitted code and safe Rust, on the
shape A9, A10 and A34 are about: many objects sharing a few.

- 262,144 `Player` objects (`hp`, `x`, and an `inventory` list), each a separate heap block.
- 1,048,576 watchers, each holding a handle to a random player.
- 32 frames. Each frame, each watcher: `hp -= 1`; adds `x * (len(inventory) + 1)` to a total; one in
  eight appends to the inventory and, past eight items, sums and clears it; one in sixteen re-targets to
  a random player (a handle store: the new player's count goes up, the old one's down).
- Integer overflow checks are left out of every program (they are not a memory-model cost; Rust's
  release build has none either).

**Programs** (`target/redesign2/ownership/bench/`):

| Program | What it is | Object size |
|---|---|---|
| `share.c` `-DMODEL=0` | plain C: raw pointers, no header, no counts | 32 bytes |
| `-DMODEL=3` | the lean class: 8-byte header, count changes only on re-target | 40 bytes |
| `-DMODEL=1` | today, as the compiler emits this program: 24-byte header, 8 bytes of access words, a retain and a release of `t = ws[i].target` every step, no access checks (read from `probes/a09_emit/c/release/c/main.c`) | 64 bytes |
| `-DMODEL=2` | today's layout with the checks `[EXC-19]` puts in a program where they are not proved away (the runtime's `ember_field_check_read` / `check_write` / `begin_read`, `runtime/ember_rt/include/ember_rt.h`) | 64 bytes |
| `-DMODEL=4` | today's object without the per-step count pair | 64 bytes |
| `-DMODEL=5` | the lean object with today's per-step count pair | 40 bytes |
| `-DMODEL=6` | the lean class with a 16-byte header (weak count and flags kept in it) | 48 bytes |
| `-DMODEL=7` | the lean object with no counts at all (not safe; only to take the cost apart) | 40 bytes |
| `share_refcell.rs` | safe Rust, `Rc<RefCell<Player>>`, one `borrow_mut` per step | 56 bytes with the counts |
| `share_cell.rs` | safe Rust, `Rc<Player>` with `hp: Cell<i32>` and `inventory: RefCell<Vec<i32>>` | 56 bytes |

Object sizes: `sizes.c` and `sizes.rs` print them (32, 64, 40, 48; Rust 40 + 16 = 56). The
checks-kept program leaves out the runtime's second test in `ember_field_begin_read` (reader-count
overflow) and the check in `ember_field_end_read`, so it slightly flatters today's checked code.

**Commands.** Build: `"C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=<n> share.c -o share_<n>.exe`
(clang 22.1.8); `rustc -C opt-level=2 --edition 2021 share_refcell.rs -o share_refcell.exe`, and the
same for `share_cell.rs`. Run: `powershell -File run.ps1 -Out runs/<file>.txt -Rounds 5 -Exes <list>`:
each round runs every program once, in the listed order, one at a time, pinned to the P-cores
(`cmd /c start /b /wait /affinity C03C03`), after checking `Win32_Battery.BatteryStatus` is 2. Medians:
`python -I medians.py runs/<file>.txt share-plain`. Every run of every program printed checksum
80363660820.

**Results, batch 1** (`runs/share.txt`, summary `runs/share_summary.txt`):

| Program | median ms | min | max | time ratio | peak KB | memory ratio |
|---|---|---|---|---|---|---|
| plain C | 286.337 | 276.242 | 294.578 | 1.00 | 57,352 | 1.00 |
| lean class (proposed) | 310.621 | 305.009 | 357.563 | 1.08 | 57,352 | 1.00 |
| today, as emitted | 500.288 | 479.436 | 583.990 | 1.75 | 65,640 | 1.14 |
| today, checks kept | 535.733 | 501.394 | 601.980 | 1.87 | 65,636 | 1.14 |
| today's object, no per-step count pair | 432.691 | 399.782 | 473.425 | 1.51 | 65,640 | 1.14 |
| lean object, with the per-step count pair | 360.767 | 348.214 | 406.896 | 1.26 | 57,356 | 1.00 |
| Rust `Rc<RefCell<Player>>` | 392.491 | 361.302 | 481.413 | 1.37 | 61,924 | 1.08 |
| Rust `Rc<Player>` + `Cell` | 384.628 | 377.303 | 460.155 | 1.34 | 61,924 | 1.08 |

**Batch 2** (`runs/share16.txt`): plain 315.302 ms; lean class 353.028 (1.12); lean class with a
16-byte header 398.856 (1.26), peak 61,576 KB (1.07).
**Batch 3** (`runs/share_counts.txt`): plain 311.021 ms; lean class 337.587 (1.09); lean object with no
counts 319.222 (1.03).

**What the numbers say, and what they do not.**

- The machine was shared with other agents: plain C's median moved from 286 ms to 315 ms between
  batches, and one program's runs spread by 33% (Rust `RefCell`, 361 to 481 ms). Ratios are taken within
  a batch. All of it is PROVISIONAL.
- **Lean class against today's emitted code: 1.08 against 1.75 (1.61x faster), 14% less memory.** Well
  over 15%: this is the result the proposal rests on, and it holds in every batch.
- Taken apart (batch 1): shrinking the object from 64 to 40 bytes saves 28% of the time with or without
  the per-step count pair (432.7 to 310.6 ms; 500.3 to 360.8 ms); dropping the per-step pair saves 13.5%
  and 13.9% (500.3 to 432.7; 360.8 to 310.6). The second is under 15%: it waits for the final
  measurer. Why 24 bytes cost 28% here (the larger blocks straddle more cache lines) is UNVERIFIED: no
  hardware counters were read.
- **Lean class against plain C: 1.08, 1.12 and 1.09 in three batches**, under 15%, so whether this is
  "on par" is for the final measurer. Batch 3 says most of it is counting when a handle is overwritten
  (1.03 without counts): the old target's count is written, an object plain C never touches. Rust's
  `Rc` does the same work.
- **Lean class against safe Rust: 1.08 against 1.34 to 1.37** (19% to 21% less time), and Rust uses 8%
  more memory. Why Rust is slower here (its borrow flag, its 56-byte block) was not taken apart.
- **A 16-byte header costs 1.26 against 1.12** and 7% more memory (batch 2): the 8-byte header matters.
- Plain C and the lean class have the same peak memory: the Windows heap gave 32- and 40-byte requests
  the same block size (inferred from the equal peaks, not checked in the allocator).
- Not covered: atomic counts (objects shared across threads), a weak handle's side record, and a
  program where field effects refuse something. The model is the code the proposal says the compiler
  emits; no compiler emits it yet.

---

## 5. How the answers fit together, and what they need from other groups

**One program.** `probes/together.em` is a small game written as a Python programmer writes it: a
shared `Player` (A9, A10, A34), a `Game` struct whose `tick` calls `self.note` inside a loop (A1), an
`init` that calls a helper (A18), a cached radius (A31), a kill counter (A21), a node stored and then
changed (A11), and a second name for the log (A11). Today `ember check` reports:

```text
error[E3023]: cannot mutate borrowed parameter `self`             (16: the cache, A31)
error[E2100]: `self` is used before all struct fields are initialized   (40: the helper in init, A18)
error[E2020]: … `get` does not take these arguments               (54: counts.get(name, 0), A21)
error[E3040]: `g` is borrowed after it has been moved out of      (68-69: snapshot = g.log, A11)
```

and it accepts `child.value = 5` after the push, where the write is lost (A11, section 7). Under the
proposal every line compiles as written and means what Python means: the cache by field effects, the
helper by its read set, `get(name, 0)` by the Pythonic `get`, `snapshot` and `child` by names following
values, the loop calling `note` by field effects (accepted today too, because `note` is in the same
package; across packages it is A1's E3020), and the shared `Player` as a lean class.

**How the three rules depend on each other.**

- Names (2.2) need field effects (2.1): whether a name still holds its value after a call is read from
  the call's summary.
- The lean class (2.3) needs field effects: they are what replaces the access words and what proves a
  handle need not be retained.
- A24 is the sum of the three: once structs and classes obey field effects and names alike, only the
  copy-or-share meaning differs.

**What this group needs from the others.**

| From | What | Used by | If not given |
|---|---|---|---|
| containers (A2, A4) | a loop over a list, or a name bound to its element, survives a change to that list made inside a call, decided when compiling (for example: the loop re-reads the list after such a call and walks by index) | A9's loop 3, A1 with `dyn`/`virtual` calls, A24 row 1 | those programs are compile-time errors with fix-its (3.2) instead of running as Python's |
| containers (A2) | a name bound to a list element survives the list growing (re-read by index), and a removal that shifts elements ends it at compile time | names following values into lists (2.2 rule 3) | a name that followed a value into a list ends at the list's next growth |
| containers (A3) | two elements passed together are proved distinct, or one compare is inserted | 2.1 rule 5 for elements | as today (`get_pair_mut`) |
| references (A5, A13) | `cast` of a place is a place | A24 row 4 (ODR-286's pool) | that row stays as today |
| parent pointers (A8) | the `weak` field, and its count kept outside the header | the 8-byte header (2.3) | a 12- to 16-byte header: 1.26x against 1.12x measured (batch 2) |
| strings and views (A22, A25) | whether `String` stays a value or becomes a shared immutable buffer | A11's "replaced inside a call" residue | if strings stay values, `w = p.name; p.rename(); print(w)` needs `copy()`; if shared, it needs nothing |
| threads (A15, A33) | a value two threads can reach is not changed by either (summaries tell which methods change it, A31's cache included); counts of objects that cross threads are atomic; `@cached` shared across threads uses a once-flag | A31, A34, the lean class | not memory-safe across threads; this group's rules are single-thread rules |
| closures (A12, A32) | a closure's effects are its body's (ODR-278 gives every closure its own type, so they are known); a closure in a field or behind `fn(A) -> R` has declared or all effects | 2.1 rule 4 | calls of stored closures are treated as having every effect |
| error messages (A17) | every residue above is an error with a fix-it the checker re-runs before showing | all | |

**What the references group asked of this one** (phase3b-references.md, section 9), and the answer
here: one decision for the `pub` boundary, covering `[BRW-10]`'s field sets and their result
summaries: summaries go into the interface for every function, and the package tool lists their
changes at publishing (2.1). What `x = place` means for a value that cannot be copied: a name for the
place, which takes the old value only when the place is overwritten (2.2 rules 1 and 2), which is the
assumption of their section 7.4. Path-based invalidation that accepts `self.pools.insert` while a
reference into a box's contents is in use: 2.1 rule 1, with the heap-contents refinement. Their
`cast` of a place giving a place (their 3.4) is what row 4 of 3.5 needs.

---

## 6. Discarded answers, and the good parts kept

| Answer | From | Why discarded (what no fix removes) | Good part kept |
|---|---|---|---|
| lists and dicts as values that copy on write | the copy-instead-of-share design (phase3-design-D1.md), Swift, Roc | Python's meaning lost: `matrix.append(row); row.append(1)` loses the 1 silently (phase3-break-D1.md N1); +10% per random write and a hidden 64 MiB copy (`phase2-rc-elision.md`) | "where the move is proven, nothing changes" (2.2 costs nothing for the same reason); its E-COPYLOST idea (catch a lost write at compile time) is what 2.2 does for `child.value = 5` |
| checked pointers for shared objects | the checked-pointers design (phase3-design-D2.md), Vale | +23% in order, +45% random per use, 8 bytes per reference (`phase2-vale-austral-pony.md`); the owner's death makes `old = p.name; p.name = new; print(old)` panic (phase3-break-D2.md N1) | the old name stays usable after a store (2.2 rule 3, made static); a generation compare as a cheap `Weak.upgrade` (for A8) |
| run-time fallbacks where the borrow checker fails | the never-an-error design (phase3-design-D3.md) | the owner of an object depended on line order (phase3-break-D3.md 2.1); layouts depended on distant code (2.8) | "names bind places" and "a name keeps the old value", here made fully static with an error where the static rewrite cannot reach (its break 2.3); the at-most-8-byte header; summaries in the interface |
| today's per-field access words | Ember today (`[EXC-19]`) | 8 bytes per object here and a check per access where not proved away; measured 1.75x to 1.87x of plain C (section 4) | per-field precision, now at compile time |
| move at last use, copy elsewhere | phase 1 A11(b) | hidden allocations; writes to the copy are lost | as a fix-it the programmer writes: `copy(x)` |
| one kind, representation chosen by the compiler | phase 1 A10(b), A24(a) | layout and speed would depend on code anywhere in the program | as a lint that names classes never shared (3.3) |
| zero-initialised fields | the checked-pointers design's A18 | breaks non-null and range types, and the bounds checks range facts remove (phase3-break-D2.md H8) | none |
| declared field sets on `self` | a Rust design idea, "view types" (3.1; UNVERIFIED) | more to write than Python | an optional `@effects(…)` to freeze a `pub` method's summary |
| keeping the access word only where needed | (my own) | a class's layout would depend on code anywhere | none |

---

## 7. Found in passing: defects in today's compiler

1. **A write to a moved-from value is accepted and lost (worse than Rust).** `root.children.push(child);
   child.value = 5` passes `ember check` (`probes/a11_store_then_use.em`, `a11_store_then_read.em`); the
   emitted C writes the dead local (`child.value = 5LL;` after the push, `probes/a11_emit/c/debug/c/main.c`),
   so the program prints `1` where Python prints `5`. A `String` field works the same way: the new
   string is written to the dead local and freed at scope end, and the moved one is not dropped twice
   (`probes/a11_emit_str/`), so it is memory-safe but silently wrong. Rust refuses it (`E0382: assign
   to part of moved value`, `rust_probes/rustc_output.txt`). `[OWN-3]` lists reading, moving and
   borrowing a moved-from place as E3040; assigning a field of it is not listed.
2. **A struct that contains itself by value crashes the checker.** `struct Tree: left: Option[Tree]`
   gives `thread 'main' has overflowed its stack`, exit 127 (`probes/a10_box_inferred.em`). Phase 1 found
   the same crash for `Option[ref Node]`; this is a second trigger. It should be an error naming `Box`
   (or, under 3.3, be accepted).
3. **Phase 1's unverified A1 case is now verified**: a `pub` method of another package borrows all of
   `self` (E3020, `probes/a1_two_packages`), as `[BRW-10]` says.
4. **A count pair the spec allows to remove is emitted.** `t = ws[i].target` used over several
   statements gets a retain and a release every loop step (`probes/a09_emit/c/release/c/main.c`), though
   nothing in `t`'s life writes `Watcher.target`. `[RC-3]` allows removing it; `[RC-2b]` only guarantees
   it within one expression. Measured at 13.5% to 13.9% of the workload's time (section 4, PROVISIONAL).

---

## 8. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/ownership/`:

- `probes/`: the Ember probes (`*.em`, `a1_one_package/`, `a1_two_packages/`, `a09_emit/`,
  `a09_fields_emit/`, `a11_emit/`, `a11_emit_str/`) and `ember_check_output.txt` (every probe, with
  phase 1's probes for these ids re-run).
- `rust_probes/`: `a11_assign_moved_field.rs`, `leak_lib.rs`, `leak_use.rs`, `rustc_output.txt`.
- `python/`: `py_checks.py`, `py_checks_output.txt`.
- `bench/`: `share.c` (eight models), `share_refcell.rs`, `share_cell.rs`, `harness.rs`, `sizes.c`,
  `sizes.rs`, `run.ps1`, `medians.py`; raw runs and summaries in `bench/runs/` (`share.txt`,
  `share16.txt`, `share_counts.txt` and their `_summary.txt`). The C programs include the shared
  `target/redesign/bench/harness.h`.
