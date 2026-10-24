# Phase 3b, ownership and sharing: attacked, then saved

Written 2026-10-24 by the attacker of the ownership-and-sharing group's report
(`phase3b-ownership.md`, "the report" below), under the owner's job (`JOB.md`): speed on par with C
and Rust, memory safety the same as Rust's or better, and Python's ease, all at once, with the
checking done when the program is compiled. The job says to save an answer before discarding it, so
every problem below comes with the fix that removes it at the least cost, and the fix is attacked
again.

How every claim here can be traced (all paths under `C:/Users/ism19/Code/ember/target/redesign2/ownership-attack/`):

- **Ember today**: `ember check` output (`target/ci/ember.exe check <file>`) for every probe in
  `probes/`, all in `probes/ember_check_output.txt`. No Ember program was built or run.
- **What the proposed rules would make the compiler emit**: small C programs written by me, in `c/`,
  each with the Ember program it models in a comment. Memory errors were caught with AddressSanitizer
  (a clang option, `-fsanitize=address`, that makes a program stop with a report the moment it reads
  or writes freed memory): `c/attacks.c`, output in `c/attacks_output.txt`. Built with
  `"C:/Program Files/LLVM/bin/clang.exe" -O1 -g -fsanitize=address attacks.c -o attacks.exe` and run as
  `attacks.exe <1..7>` with `ASAN_OPTIONS=abort_on_error=0` (its runtime DLL copied next to it from the
  LLVM install, nothing downloaded). `c/noalias.c`: `clang -O2`, output `c/noalias_output.txt`.
- **Python**: `python/py_attack.py`, run with `python -I` (Python 3.14.2), output in
  `python/py_attack_output.txt`. Every "Python prints" below is from that file.
- **Timings**: mine, in `bench/`, made under the job's rules: mains power checked before every round,
  pinned to the performance cores (`start /affinity C03C03`), one program at a time, 5 rounds, the
  order reversed in every second round, medians. Other agents were running, so every timing is
  **PROVISIONAL**, and a decision resting on a difference under 15% waits for the final measurer.
- Numbers from the report name its files. Nothing outside `tasks/96/redesign/` (this file) and
  `target/redesign2/` was written.

---

## 0. The short version

The report's three rules hold up as ideas. As written, they have six memory-safety holes, one false
claim that would make the compiled program compute a wrong answer, four places where a Python program
is refused or silently means something else, two overstated claims, one imprecision and one unneeded
run-time check. Each has a fix that costs nothing when the program runs, except one compare in one
case (measured: no slower than the code without it, when emitted well). No answer is discarded.

| Annoyance | The report's answer | Verdict | What changed |
|---|---|---|---|
| A1, a method borrowing the whole object | field effects (a function's summary of what it changes, computed from its body) | **saved** | unsafe and foreign code declare their effects; drop code runs only after the new value is stored; inherited fields are matched by the class that declares them; a hot reload may not grow a summary a running caller relies on |
| A9, shared mutable state | `class` plus field effects | **saved** | as A1; the `a is b` check before a loop is dropped (the containers group's loops make it unneeded); the speed claim restated |
| A10, choosing among ownership kinds | two words, `struct` and `class` | **kept** | the "never shared" hint only for classes not visible outside their package |
| A11, moves, use after move, clone | names follow values | **saved** | element names are the containers group's positions (fixes a use of freed memory); a name keeps the old value when a write that may be to the same place replaces it (one compare); `@derive(Copy)` structs are named like any other value |
| A24, struct or class decides what compiles | one rule set for both | **saved** | follows A1 and A11 |
| A31, caching in a read-only method | the plain spelling compiles; `@cached` | **saved** | `@cached` may read only the object's own fields, and may not change what it reads |
| A34, clone before move for shared handles | unchanged | **kept** | none |
| A18, initialisation | a helper may run on a half-built object if it reads only fields already set | **saved** | the helper is compiled a second time for the constructor, where its stores to unset fields are first stores (fixes a crash, and accepts Python's usual helper that sets fields) |
| A21, updating a map entry | `get(k, default)`, `Map(default=…)`, one lookup | **saved** | `get(k, default)` and `setdefault` also for values that are not plain data |
| the lean class (an 8-byte header) | 1.08x to 1.12x plain C, faster than safe Rust | **kept** | re-measured at 1.05x and 1.10x; "faster than safe Rust" is true against Rust's `Rc`, not against Rust written with one array and positions (0.77x to 0.80x of plain C) |

The problems, by name (each is a section below):

| Problem | Kind | Evidence | Fix | Cost of the fix |
|---|---|---|---|---|
| the unsafe-code hole | memory safety | std's own `CppVector`: a view kept across `push_back`, read after free (`attacks.exe 3`) | a body with `unsafe` uses its declared effects (`mut self` means "may change anything in `self`"), as Rust's `&mut self` does | none |
| the drop-order hole | memory safety | `w.items = []` whose elements' drop walks `w.items`: read after free (`attacks.exe 4`) | every change that runs program code stores the new value before dropping the old | none (the same instructions in another order) |
| the pop hole | memory safety | `root.children.push(child); root.children.pop(); child.children.push(…)`: write after free (`attacks.exe 1`) | element names follow the containers group's position rules; the name keeps a popped value | none beyond what the containers group argues is zero |
| the inherited-field hole | memory safety | a loop over `s.items` while `d.items` grows, `d` a subclass handle to the same object | effects on objects are matched by the class that declares the field | none |
| the hot-reload hole | memory safety | a reload that makes `note` push to `self.players` under a running loop | a reload that grows a summary a running frame or the host relies on is refused | none when running |
| the half-built-helper hole | memory safety | the helper's assignment drops whatever the unset field held (`attacks.exe 6`) | the constructor calls a second copy of the helper | code size only |
| the overlap claim | wrong answer | `reset_then_set(c, c)` gives 0 with the assumed `restrict`, 1 in Python (`noalias.c`) | drop the claim; alias facts only from proofs, as Ember's `[SIMD-3]` already requires | none against today |
| the second-handle refusal | Python refused | `old = p.name; q.name = "bob"; return old` with `q` maybe `p` | one compare at the write; the name keeps the old value when equal | measured 0.88x of the code without the compare (PROVISIONAL) |
| the copied-struct divergence | Python changed silently | `pos = p.pos; pos.x += 1.0` prints 1.0 in Ember (today and as proposed), 2.0 in Python | `@derive(Copy)` structs are named, not copied | none |
| the setting-helper refusal | Python refused | `self._load()` in `__init__` setting fields (E2100 today) | the second copy of the helper, as above | code size only |
| the stale cache | false claim ("never stale") | a cached value reading `self.owner.name` | `@cached` reads only the object's own fields | none |
| the list-default refusal | Python refused | `groups.get("z", [])` (E2072 today), `setdefault` | `get` and `setdefault` for any value type | none |
| the drop-effects imprecision | refusals and needless run-time counts | storing a handle whose class has a `dyn` field has "every effect" | a `drop` may change only what its object owns, unless the class says more | none |
| the duplicate check | an unneeded run-time check | `if a is b: panic` before a loop | dropped; the containers group's position loop is safe without it | removes a check |
| the Rust-comparison claim | overstated | Rust and C with one array and positions run at 0.71x to 0.80x of plain C pointers | restated; a per-class block allocator measured 7% faster (under 15%: for the final measurer) | none |

---

## 1. Words used

- **Place**: where a value lives: a local, a field (`g.log`), a list element (`xs[3]`), a map entry, a
  field reached through a handle (`cam.target.hp`).
- **Handle**: what a variable of a class type holds: a pointer to a heap object with a **count** of
  the handles to it; the object is freed when the count reaches zero.
- **Loan**: a pointer the compiled program holds into a place for a while (a `for` loop's pointer into
  the list it walks, a view of a string's bytes).
- **Reshape**: the report's word for a change that can free or move memory: replacing a value that
  owns memory, `push`, `remove`, `clear`, changing an enum's variant, dropping an object.
- **Effect summary**: what the compiler records about a function from its body: which places it reads,
  writes and reshapes. The report's first rule, **field effects**, checks loans against these.
- **Name**: under the report's second rule, **names follow values**, `x = place` makes `x` a second
  name for the value in the place, not a copy.
- **Drop**: what runs when a value ends: freeing its memory, releasing its handles, and a class's own
  `fn drop(self)`, which is program code.
- **Position**: the containers group's way of naming a list element: "element 3 of `xs`", looked up
  again after the list may have moved (`phase3b-containers.md`, section 2).
- **`restrict` / `noalias`**: a promise to the C compiler (or LLVM) that two pointers never point at
  the same memory, which lets it reorder reads and writes.
- **Slab**: one large block from which all objects of one class are taken, instead of one `malloc` each.
- **Kinds (a), (b), (c)**: the job's three kinds of answer: (a) a Pythonic way of writing what Rust
  checks; (b) a smarter compile-time rule that makes Rust's rule unneeded; (c) a new mechanism.

---

## 2. Field effects: attacks and saves

The rule (report, section 2.1): any field may be written through any name; a loan is broken only by a
reshape of its place; every function's summary says what it may reshape; `mut` is inferred from the
body; for objects reached through handles, effects are "by class and field".

### 2.1 The unsafe-code hole

**The rule as written.** "`mut` is inferred from the body. Writing it stays allowed and is checked"
(rule 6), and a reshape is inferred from types: "replacing a field or variable whose value owns
memory", `push`, `clear`, and so on.

**The program it accepts.** The standard library's own `CppVector` (`std/src/ffi.em`, from line 279)
wraps a C++ `std::vector`. Its `push_back` writes no field of `self`; it calls a C++ function through
a stored pointer, and that function may move the elements to a new block and free the old one:

```ember
pub fn push_back(mut self, value: T):          # std/src/ffi.em
    push = self.push
    unsafe:  # SAFETY(ffi): the thunk copies the value into the vector
        push(self.object, ref_to_ptr(ref value))
```

Inferred from that body, the summary is "reads `self.push` and `self.object`": nothing reshaped. So:

```ember
from std.ffi import CppVector, c_int

fn grow_while_viewing(v: CppVector[c_int]):    # proposed: no `mut` needed
    s = v.span()            # a view of the vector's elements; borrows v
    v.push_back(7)          # summary: reshapes nothing of v, so the view stays valid
    println(s[0])           # reads the block push_back freed
```

The nearest Python, an `array` with a `memoryview` of it, refuses the growth while the view exists:

```python
v = array.array("i", [5]); s = memoryview(v)
v.append(7)        # BufferError: cannot resize an array that is exporting buffers
```

**What happens.** Today the program is refused, only because `push_back` says `mut self`
(`probes/cppvector_span_push.em`: `E3021: *v is borrowed here and mutably borrowed elsewhere`). Rule 6
does not say whether a written `mut` widens the summary or is only checked against the body. Read as
"checked", the summary is the body's, which changes nothing the type system can see, and the program
is accepted; read as "widens", the same method written without `mut`, as the rule invites ("`mut` is
inferred"), is accepted. The C it would run (`attacks.exe 3`): `heap-use-after-free ... READ of size 4`.
A user's own container built on `*mut T` and `Allocator.reallocate` (`[ALC-2]`) has the same hole,
and also through `pop`, whose body only subtracts from a number field and reads the element out.

**The fix.** In a function whose body contains `unsafe` (or calls a foreign function), the inferred
summary is not used. The function's declared effects are its summary: `mut self` means "may reshape
anything in `self`", as Rust's `&mut self` does, and a reference it returns borrows all of `self`
unless it says less. Such a function without a declaration gets "reshapes every parameter it
reaches", which is safe and can only refuse more. The body is still checked to stay within what it
declares. This is the report's own "good part kept", `@effects(…)` to freeze a summary, made required
where inference cannot see.

**Cost.** None when running. `CppVector`'s methods that change the vector (`push_back`, `span_mut`,
`drop`) already say `mut self`; the other `unsafe` bodies in `std/` were not checked one by one.

**Attacked again.** A safe function calling the unsafe one takes the declared summary: covered. Unsafe
code that frees memory reachable from another parameter must name it (`@effects(reshapes=[self,
other])`), as Rust needs `&mut other`; not naming it is a broken `[UNS-4]` obligation, as in Rust.

### 2.2 The drop-order hole

**The rule as written.** "A reshape that runs code of the program (a `drop` method run by `clear()`)
has that code's effects in its own summary. The runtime must detach before it runs such code (take the
buffer out, empty the list, then drop the elements)" (report, section 2.1). Detaching is required for
`clear()` only. Today's compiler releases an old handle before storing the new one
(`target/redesign2/ownership/probes/a09_emit/c/release/c/main.c`, lines 486-488:
`ember_release_plain(…target); …target = _67;`). Whether today's run-time access words (`[EXC-16]`)
catch a drop that reaches a class field during its own assignment was not checked (UNVERIFIED); the
report removes them either way, so the order becomes what keeps the program safe.

**The program it accepts.**

```ember
class Item:
    world: World
    hits: int = 0

    fn drop(self):
        for other in self.world.items:     # reads World.items
            other.hits += 1

class World:
    items: Array[Item] = []

fn reset(w: World):
    w.items = []        # reshapes World.items; no loan is live across this line
```

```python
class Item:
    def __init__(self, world): self.world = world; self.hits = 0
    def __del__(self):
        for other in self.world.items: other.hits += 1
w = World(); w.items = [Item(w) for _ in range(3)]
w.items = []           # the destructors walk the NEW list: 0 elements (replace_list_drop_walks_it: 0)
```

No loan is held across `w.items = []`, so the rule finds nothing to refuse. Emitted in today's order
(drop the old list in place, then store the new one), the second item's drop walks the old list and
writes the first item, already freed: `attacks.exe 4`, `heap-use-after-free ... READ of size 8`.

**The fix.** For every change that may run program code (assignment, `remove`, `pop`, `truncate`,
`clear`, replacing a map entry, changing an enum's variant, the last release of a handle): take the
old value out, store the new value, then drop the old one. A drop that reaches the place sees the new
value, which is Python's order. `attacks.exe 5`: "the drops walked 0 elements", as Python.

**Cost.** None: the same loads and stores in another order. The report's own benchmark already does it
for handles (`share.c`: `old = ws[i].target; ws[i].target = n; release(old)`).

**Attacked again.** A drop that pushes into the place being replaced lands in the new list (Python
does the same). A drop that replaces the place again is an ordinary assignment, done after the first
has finished storing. A name that keeps the old value (section 3) holds the detached value; it is
dropped after the name's last use.

### 2.3 The inherited-field hole

**The rule as written.** "For class objects, 'may' means: any object of that class" (rule 3). Ember
classes have one base and inherit its fields (`[GRM-1]`, `[CLS-4]`).

**The program.**

```ember
open class Script:
    items: Array[int] = []

class Door(Script):
    pass

fn copy_into(s: Script, d: Door):
    for x in s.items:          # a loan into Script.items
        d.items.push(x)        # a reshape of "Door.items"? It is Script.items of the same object.
```

```python
d = Door(); s = d
for it in s.items: d.items.append(it)    # never ends (inherited_field_grows_in_loop, stopped at 6)
```

Today this compiles and is guarded by the field's run-time access word (`probes/inherited_field_loop.em`,
exit 0). If "that class" is read as the class of the handle (`Door`) rather than the class that
declares the field (`Script`), the loan on `Script.items` and the reshape of `Door.items` look
unrelated, the access word is gone, and `copy_into(d, d)` walks a buffer the push frees.

**The fix.** Effects on objects are keyed by the class that declares the field. Fields cannot be
overridden (`[CLS-4]`), so each field has exactly one declaring class. **Cost:** none. **Attacked
again:** interface-typed handles have no fields, only methods, whose effects come from their
declarations (`[EFF-2]`).

### 2.4 The hot-reload hole

**The rule as written.** Summaries go into the interface, and callers are re-checked when they change
(report, section 2.1, "What it gives up"). Ember also replaces function bodies in a running program:
"At a checkpoint the frames below it run their old code on after the swap" (`[HR-3b]`), and calls go
through a thunk to the newest body (`[HR-5]`, `[HR-6]`). The report does not mention reloading.

**The program.**

```ember
pub fn note(self, msg: String):         # version 1: reshapes self.log only
    self.log.push(msg)

pub fn tick(self):
    for p in self.players:              # a loan into self.players
        if p.hp <= 0:
            self.note(f"{p.name} died") # accepted against version 1 of note
```

```python
def note(self, msg): self.log.append(msg)
def tick(self):
    for p in self.players:
        if p.hp <= 0: self.note(f"{p.name} died")
```

While `tick` is paused below a checkpoint, the owner edits `note` to also `self.players.push(…)` and
the game reloads. The rebuilt package re-checks `tick` against the new `note`; the frame that is
already inside the old `tick` does not get re-checked, keeps its loan, and calls the new `note`
through the thunk. Today `mut self` is part of `note`'s signature, so a body edit cannot change what a
caller may hold across it; under inferred summaries it can.

**The fix.** Add summaries to what a reload compares: a reload is refused when a function's summary
grows and a frame below the checkpoint holds a loan across a call to it (the image records, per call,
the places it holds loans into, as `[HR-3b]` already records the types each call depends on), or when
the host calls it (as `[HR-21]` refuses a changed export signature). The refusal names the function
and the line, as `[HR-3b]`'s does. **Cost:** none when running; a comparison at reload. **Attacked
again:** a summary that shrinks is always allowed; a reload that makes `tick` itself walk by position
(the containers group's loop) applies to new calls of `tick`, and the old frame is the one refused.

### 2.5 The overlap claim

**The claim.** "Value parameters keep their 'not the same place' guarantee, checked by the caller
(rule 5), so the C and LLVM backends may still assume they do not overlap, as Rust's `&mut` lets LLVM
assume" (report, section 2.1, "Cost"). Rule 5 checks only reshapes: "A call may not reshape what its
own arguments point into". Writing a number is not a reshape.

**The program.**

```ember
struct Counter:
    n: int

fn reset_then_set(a: Counter, b: Counter) -> int:
    a.n = 0
    b.n = 1
    return a.n

c = Counter(n=5)
println(reset_then_set(c, c))     # accepted by rule 5: no reshape
```

```python
print(reset_then_set(c, c))       # 1 (reset_then_set_same: 1)
```

With the promise the report says the backends may make, clang computes the wrong answer
(`c/noalias.c`, output `c/noalias_output.txt`): "with noalias 0 (c.n is 1), without 1".

**The fix.** Drop the claim. Ember's spec already says alias facts "MUST be derived, never assumed
... Two views being live at once is not a proof, and neither are parameter modes" (`[SIMD-3]`); the
backends get them from proofs (`[DSJ-*]`, `split_at`, distinct owners in one function). Extending rule
5 to writes instead would refuse `f(c, c)`, which Python runs. **Cost:** none against today's Ember
and against every number in the report (none used parameter `restrict`). Against Rust, a loop that
Rust's `&mut` would let LLVM vectorise without a run-time overlap test may get one here; not measured
(UNVERIFIED).

### 2.6 The drop-effects imprecision

**The rule as written.** Dropping the last handle to an object is a reshape and has the effects of the
code it runs. A class with a field of an interface type, or an `open` class whose subclasses live in
other packages, runs unknown `drop` code, which has every effect (`[EFF-2]`).

```ember
for e in self.entities:
    e.target = self.pick()     # may drop the old target; Entity holds components: Array[Component]
```

```python
for e in self.entities:
    e.target = self.pick()
```

Every handle store whose old object might reach unknown drop code then has every effect: it breaks
every live loan, so the loop above becomes a position loop with the containers group's run-time
walkers count (`phase3b-containers.md`, section 4.3, rule 9), and a view held across it is refused.
Not unsafe; imprecise in the most common game code.

**The fix.** A `drop` method may reshape only what its own object owns, unless the class declares more
(`@effects` on `drop`, checked). Ember already restricts `drop` (`[CLS-7a]`: `self` may not be stored
anywhere that outlives the call). Then dropping any object, even through an interface, has bounded
effects: it frees what it owns, and a loan into an object is already protected by the object being
kept alive (`[RC-5]`). **Cost:** none. **What Python loses:** a `__del__` that removes the object from
a global registry needs the declaration; Python itself does not promise when `__del__` runs.

### 2.7 Field effects, fixed

The rule with the five fixes above: declared effects in unsafe and foreign code; new value stored
before any drop runs; effects on objects keyed by the declaring class; reloads refused when a summary
grows under a running caller; drops bounded unless declared; and no alias fact from parameters.
Attacked again as a whole: the report's own attack table (section 2.1) still holds, and each fix was
attacked again above. **Verdict: saved.**

---

## 3. Names follow values: attacks and saves

The rule (report, section 2.2): `x = place` names the value; if the place gets a new value while `x`
is used, `x` keeps the old one; a stored value's old name follows it into the new place; a value is in
one owning place at a time.

### 3.1 The pop hole

**The rule as written.** Rule 3: "Growth of the list does not end this (the containers group's rule for
element names, section 5); a removal that shifts elements, a clear, or a store into a place the
compiler cannot follow does". And the emitted code: "no count, no check (the index is in range because
nothing removed from the list since the push...)". `pop()` removes without shifting, so it is not in
the list of things that end the name.

**The program.**

```ember
root = Node(value=0)
child = Node(value=1)
child.children.push(Node(value=3))
root.children.push(child)          # child now names root.children[child_at]
root.children.pop()                # shifts nothing: the name is not ended; the popped node is dropped
child.children.push(Node(value=7)) # emitted: root.children[child_at].children.push(...), no check
```

```python
root.children.append(child); root.children.pop(); child.children.append(Node(7))
print(len(child.children), len(root.children))     # 2 0 (store_pop_use)
```

`attacks.exe 1`, the code the report says the compiler emits: `heap-use-after-free ... WRITE of size
8`. The slot past the end still holds the popped node's bits, whose child list was freed with it.
`truncate`, `swap_remove`, `insert` (which shifts the other way) and `sort` are missing from the list
too.

**The fix.** A stored value's name is a containers-group position, with all of that group's rules, not
only the growth rule: the position is checked against the length after anything that may shorten the
list; it follows a removal before it; a reorder while the name is needed is a compile-time error; a
change by code the compiler cannot see is caught by the walkers count (`phase3b-containers.md`, section
2.3, rules 2-6). One refinement: where that group makes `pop()` of a named element that owns memory a
compile-time error (its rule 4), the name can take the value instead, because the value goes either to
a temporary that is dropped (the name keeps it) or into a place the compiler sees (the name follows
it, this report's rule 3). `attacks.exe 2` is that version: "2 0", Python's answer.

**Cost.** The containers group argues positions cost nothing against Rust's workaround, from the
emitted code: one reload of the buffer pointer after a push, as Rust's and C's code must do (their
section 2.3; not timed there or here). The check after a shortening runs next to the removal itself.

### 3.2 The second-handle refusal

**The rule as written.** "If the place gets a new value while `x` is still used, `x` keeps the old
value. In the same function, the compiler moves the old value into `x`'s own storage at that line
instead of dropping it" — when it knows the line writes that place. When it cannot tell, "where they
cannot tell, the program is refused" (report, section 2.2, "Why it is memory-safe"). That refusal is
not in the report's list of what is left.

**The programs.**

```ember
class Player:
    name: String = "ann"

fn rename(p: Player, q: Player) -> String:
    old = p.name
    q.name = "bob"         # may be p's name
    return old

n = xs[i]
xs[j] = Node(value=9)      # may be the same element
println(n.value)
```

```python
print(rename(x, x))        # ann (alias_write_same)
n = xs[i]; xs[j] = Node(9); print(n.value)    # 1 with j == i (element_write_same: 1 9)
```

Today `old = p.name` is `E3012: cannot move out of a class field` (`probes/alias_name_write.em`).
Under the rule as written, both programs are refused.

**The fix.** One compare at the write: if the place written is the named place, the old value moves
into the name's own storage (dropped after the name's last use); otherwise it is dropped as usual. The
containers group does the same for a removal it cannot place ("a one-bit flag when the compiler cannot
tell whether it was this element", their section 2.3, rule 3).

**Cost, measured** (`bench/alias_name.c`, 4,096 objects each owning a block, 2^27 steps; each step
names one object's field, replaces another's, which is the same one in 1 step in 4,096, and reads the
name; blocks come from a free list so the allocator does not hide the compare; `runs/alias_name.txt`,
summary `runs/alias_name_summary.txt`, PROVISIONAL):

| Program | median ms | ratio | checksum |
|---|---|---|---|
| no compare (the old value is dropped even when it is the named one: wrong when equal) | 236.696 | 1.00 | 27090862892341538 |
| the fix, first version: the name is a pointer redirected to its own storage | 317.636 | 1.34 | 27090862489880799 |
| the fix, second version: the name's value held in a register, the two places compared | 207.164 | 0.88 | 27090862489880799 |
| Rust's spelling: copy the field first (`.clone()`), with the same cheap free list | 265.623 | 1.12 | 27090862489880799 |

Why the first version is slower was not taken apart (no assembly read: UNVERIFIED); the likely reason
is that a pointer which may be redirected to a local forces the name through memory. The second keeps
the value in a register and is faster than the code without the compare (it reads the value before
the write instead of after). Both fixed versions print the same checksum as the copying one, which is
Python's meaning; the code without the compare prints a different one. The register form is valid only
while nothing changes the named value in place through another name (`old = p.items; q.items.push(1)`
must show the push, as in Python); the summaries tell which, and the compiler uses the pointer form
there. Build: `clang -O2 -DMODE=<0..3> alias_name.c -o alias_name_<n>.exe`.

**Attacked again.** A write inside a call that may replace the named place is still the report's
residue (a compile-time error with the fix-it `copy()` for a struct; a retain for a handle): the
compare must be at the write, and the caller cannot see inside the callee.

### 3.3 The copied-struct divergence

**The rule as written.** "Values of a `Copy` type (numbers, `bool`, and plain structs their author
marked `@derive(Copy)`) are copied instead, as today and as Python's numbers behave (they never change
in place)". A plain struct does change in place: `pos.x += 1.0` changes a field.

**The program** (`probes/copy_struct_name.em`; accepted today, exit 0):

```ember
@derive(Copy)
struct Vec2:
    x: f64
    y: f64

p = Player(pos=Vec2(x=1.0, y=2.0))
pos = p.pos
pos.x += 1.0
println(p.pos.x)          # Ember today and as proposed: 1.0
```

```python
p = Player(Vec2(1.0, 2.0)); pos = p.pos; pos.x += 1.0
print(p.pos.x)            # 2.0 (copy_struct_name)
```

Silent: no error, a different number. It also disagrees with the containers group, whose names apply
to every element "that is not a plain number" (`phase3b-containers.md`, section 2.3, rule 1), so
`for v in vels: v.x *= 0.5` would change the list (Python: `copy_struct_loop [0.5, 1.0]`) while
`v = vels[0]; v.x *= 0.5` would not.

**The fix.** `@derive(Copy)` structs are named like every other value; only the rules that keep an old
value copy them (where a non-`Copy` value would be moved). Numbers and `bool` are unaffected, because a
number cannot be changed in place: `x = p.hp; p.hp -= 1` keeps the old number under either rule. A
`Copy` value stored into a second place is copied (allowed, unlike a non-`Copy` value), and a later
write through its name, which would change only one of the two, is a compile-time error naming both
(Python: `copy_struct_stored_twice 9.0 9.0`, one object in both places).

**Cost.** None: when nothing writes the place during the name's life the compiler loads the value into
registers, which is the copy; when something may, it reads through the place, which is what C does
with a pointer.

**What changes for programs that compile today.** This one prints 2.0 instead of 1.0, toward Python.
It is the only meaning change of a program that compiles today that I found among this report's fixes
(found by reading the rules, not by a search of every program: UNVERIFIED).

### 3.4 The duplicate check

**The rule as written.** "For class handles, one compare `a is b` at the call when not provable", and
for `for it in a.items: b.items.push(it)`, "the compiler emits one comparison, `if a is b: panic(…)`,
before the loop" (report, sections 2.1 and 3.2).

The containers group compiles a loop whose body may change the list it walks as a loop over positions
that sees the appended items (`phase3b-containers.md`, section 4.3, rule 1). Under that loop, `a is b`
is safe without a check and does what Python does: it never ends
(`target/redesign2/ownership/python/py_checks_output.txt`, `same_object_grows_while_iterating`,
stopped by hand).

```ember
for it in a.items:          # a and b may be one object
    b.items.push(it)
```

```python
for it in a.items:
    b.items.append(it)
``` Two rules for one
program is one too many. **The fix:** drop the compare; the position loop covers it. **Cost:** a
run-time check removed.

### 3.5 Names follow values, fixed

The rule with the fixes above: element names are positions with the containers group's rules (the name
keeps a popped value); one compare where a write may replace the named place; `Copy` structs named
too; no `a is b` compare. Attacked again: the report's attack table (section 2.2) still holds;
`old = p.name; p.name = new; log(old)` (one handle) needs no compare, because the compiler knows the
place. **Verdict: saved.**

---

## 4. The lean class: re-measured and checked

The answer (report, section 2.3): an 8-byte header (a 32-bit count and a 32-bit type index), no access
words, counts changed only when a handle is stored.

### 4.1 The model does what the report says the compiler would emit

I read `target/redesign2/ownership/bench/share.c` against the emitted C
(`target/redesign2/ownership/probes/a09_emit/c/release/c/main.c`) and the runtime
(`runtime/ember_rt/include/ember_rt.h`):

- "today, as emitted" has today's 24-byte header (`ember_obj_header`, lines 1426-1438, asserted 24 bytes
  at line 1450), the two access words (`_access_inventory`, `_access_pad0`), and a retain and release of
  `t` every step (emitted lines 310 and 368), with today's null test and count range test (runtime lines
  1528-1547). Correct.
- The lean class drops the per-step count pair. That is legal by the report's own rule: `t`'s last use
  (`t.inventory.clear()`) comes before the re-target, so no statement in `t`'s life writes
  `Watcher.target` or drops a `Player`. The rule needs "may drop a `Player`" worked out through fields
  (dropping a value that holds a `Player` drops it), with an interface-typed field counting as every
  class; section 2.6's fix keeps that precise.
- The lean retain keeps the count range test (an overflowing count panics, as today, `[RC-1]`), and
  drops the null test (a class handle is never null). Correct.
- Every model does the same work: same random numbers, same checksum 80363660820 in every run of every
  program in both of my batches (the summary files' last column).

### 4.2 Re-measured, and the Rust-comparison claim

The report's claims: lean class 1.08x, 1.12x and 1.09x of plain C in three batches; today 1.75x; safe
Rust 1.34x to 1.37x (its section 4). Re-run, with two programs added that store the players in one
array and give each watcher a position instead of a handle: `bench/share_idx.c` (plain C) and
`bench/share_index.rs` (safe Rust, bounds-checked). `runs/share.txt`, summary `runs/share_summary.txt`:

| Program | median ms | ratio to plain C | peak KB | memory ratio |
|---|---|---|---|---|
| plain C, a pointer per watcher, one `malloc` per player | 261.403 | 1.00 | 57,356 | 1.00 |
| the lean class | 287.172 | 1.10 | 57,356 | 1.00 |
| today's compiled code | 419.396 | 1.60 | 65,636 | 1.14 |
| safe Rust, `Rc<RefCell<Player>>` | 363.606 | 1.39 | 61,928 | 1.08 |
| safe Rust, `Rc<Player>` with `Cell` | 338.072 | 1.29 | 61,928 | 1.08 |
| plain C, players in one array, a position per watcher | 186.696 | 0.71 | 47,140 | 0.82 |
| safe Rust, players in one `Vec`, a position per watcher | 208.132 | 0.80 | 47,488 | 0.83 |

The report's numbers hold: the lean class is 1.10x plain C with the same layout (the report's 1.08x
to 1.12x), today's code 1.60x (the report's 1.75x), Rust's `Rc` 1.29x to 1.39x. Its claim "faster than
safe Rust" holds against Rust's `Rc`; Rust written for speed, with one `Vec` and positions, is 0.80x,
so the lean class takes 287.172 / 208.132 = 1.38 times as long as that Rust program.

A second batch with a slab allocator (`bench/share_slab.c`: the lean class taking every `Player` from
one block at a 40-byte stride; `bench/share_slab0.c`: plain C the same way; `runs/share_slab.txt`,
summary `runs/share_slab_summary.txt`):

| Program | median ms | ratio to plain C | peak KB |
|---|---|---|---|
| plain C, one `malloc` per player | 256.402 | 1.00 | 57,356 |
| the lean class, one `malloc` per player | 268.424 | 1.05 | 57,352 |
| plain C, players from one block | 210.584 | 0.82 | 53,284 |
| the lean class, players from one block | 249.324 | 0.97 | 55,328 |
| plain C, one array and positions | 181.344 | 0.71 | 47,140 |
| safe Rust, one `Vec` and positions | 197.877 | 0.77 | 47,484 |

What the two batches say (PROVISIONAL):

- The lean class's own cost, against C with the same layout, is 1.05x to 1.10x with `malloc` (under
  15%) and 249.324 / 210.584 = 1.18x from one block (just over, in one batch), where its 8-byte header
  is a larger share of a denser layout. All three wait for the final measurer.
- A slab per class made the lean class 7% faster (268.424 to 249.324 ms). Under 15%: for the final
  measurer. Good part, for the runtime: classes can take their objects from per-class blocks.
- The rest of the gap to the fastest programs is the layout: a pointer per watcher and a separate
  object per player against one array and a 4-byte position. That is what C pays for heap objects too,
  and it is not a memory-model cost. Ember reaches the fast layout the way Rust does, with a struct in
  an `Array` and positions, which the containers group makes Pythonic (`t = players[ws[i]]` names the
  element). A Python programmer writes the class version; the lint the report proposes for classes that
  are never shared cannot help here, because the players are shared.

The two ways of writing the inner step, in Python and in Ember (the class form is the report's
`probes/a09_emit/src/main.em`; the position form is what `share_idx.c` and `share_index.rs` do):

```python
# objects shared by handle                      # objects in one list, watchers hold positions
t = ws[i].target                                t = players[ws[i]]
t.hp -= 1                                       t.hp -= 1
```

```ember
t = ws[i].target          # class Player           t = players[ws[i]]     # struct Player in an Array: a name, not a copy
t.hp -= 1                                           t.hp -= 1
```

**Restated claim.** The lean class runs at 1.05x to 1.10x of C with the same objects (1.18x from one
block), at 0.79x to 0.85x of Rust's `Rc` (287.172 / 363.606 and 287.172 / 338.072), and at 1.36x to
1.38x of Rust written with one array and positions (268.424 / 197.877 and 287.172 / 208.132).

### 4.3 The header claims, checked

- **The count overflow test** is in the lean model and in today's runtime: kept.
- **The 32-bit type index** replaces today's 8-byte type pointer. Today's runtime reads that pointer for
  interface calls (`[OBJ-2]`: "the interface table is reached through the header's type information")
  and for the run-time `Sync` test in the generic retain (`ember_rt.h` line 1935:
  `object->ti->flags & EMBER_TI_SYNC`). With an index, each becomes `types[index]`: one more load,
  unless the table holds fixed-size records itself. No virtual call is in the benchmark: the cost is
  UNVERIFIED.
- **The weak count in a side record.** The report cites Swift (`phase2-swift-mojo.md`, line 96). Swift's
  header is 16 bytes, and its side-table pointer replaces its 8-byte count word. The lean header has 4
  bytes for the count, so it needs a 31-bit index into a table of side records (or a table keyed by
  address, as Objective-C does; from memory, UNVERIFIED). A flag in the top bit costs no extra
  instruction on the fast path, because the existing range test (`count - 1 < MAX - 1`) already sends
  such a count to the slow path. Weak-handle cost: unmeasured, for the parent-pointer group.
- **The flags** today's runtime keeps in the header (`EMBER_OBJ_FRAME`, `EMBER_OBJ_DEINITIALISING`,
  `EMBER_OBJ_PINNED`, the creating thread in debug builds; `runtime/ember_rt/src/ember_rt.c` lines
  3017-3208) are read only on the slow path, at reload, or in debug builds: they fit a side record or
  the top bits. A debug build can keep a larger header.

**Verdict: kept**, with the speed claim restated as in 4.2.

---

## 5. The answers one by one

### A1, a method borrowing the whole object: saved

The report's answer (field effects for every method, in the interface) stands, with section 2's fixes.
The Python program and the accepted Ember program are the report's (section 3.1). What changed: a
method whose body has `unsafe` or a foreign call keeps a declared `mut self` as its summary (2.1); a
reload may not grow a summary under a running caller (2.4). What is left: a library body change can
change what its callers may do (the report's own residue), and now also what a running program may
reload.

### A9, shared mutable state: saved

Field effects replace the access words (section 2's fixes); the lean class kept (section 4). The
report's residue, "a loop over a class's list across a call that may grow that field of some object of
that class is refused, unless the containers group's loop rule applies", becomes: the containers
group's position loop runs it, and a removal made by code the compiler cannot see is that group's
run-time panic (their section 4.4). The `a is b` check is dropped (3.4).

### A10, choosing among ownership kinds: kept

Two words, `Box` implied, `Cell`, `RefCell` and `Shared` gone. Attacked: two structs that contain each
other need one implied pointer:

```ember
struct Room:
    door: Option[Door] = None
struct Door:
    behind: Option[Room] = None
```

```python
class Room:
    def __init__(self): self.door = None
class Door:
    def __init__(self): self.behind = None
```

The compiler picks
the field that closes the cycle, in declaration order, within one package (cycles between packages
are `E1041`, `[MOD-4]`), so the layout depends on nothing outside it. `Cell` and `RefCell` going rests on
section 2 (saved) and on the threads group (a class whose fields are written must not be reached from
two threads). One correction: the lint "`class Particle` is never shared" can be given only for classes
not visible outside their package; a dependant may share a public one.

### A11, moves, use after move, clone: saved

Names follow values, with section 3's fixes: positions for stored values (the pop hole), one compare
where a write may replace the named place, `Copy` structs named. What is left: a value stored into two
owning places needs `copy()` or a class; a struct replaced inside a call needs `copy()` (the report's
residue); a `Copy` value copied into two places cannot then be changed through its old name.

### A24, struct or class decides what compiles: saved

The sum of A1 and A11 as saved. Row 1 of the report's table (a loop over a list pushed to through
another path) is now the containers group's position loop for both kinds, not a refusal for one and a
panic for the other.

### A31, caching in a read-only method: saved

The plain spelling compiles under field effects: kept. `@cached` is attacked in three ways.

**It goes stale when it reads through a handle.** The report: "empties it at every write to a field the
method read ... That is better than Python, whose cached value goes stale".

```ember
class Owner:
    name: String

struct Mesh:
    owner: Owner

    @cached
    fn label(self) -> String:
        return f"{self.owner.name}!"

o.name = "bob"       # a write to Owner.name through a handle: which Mesh's cache does it reset?
```

```python
first = m.label; o.name = "bob"
print(first, m.label)          # ann! ann!  (cached_through_handle: stale)
```

The line `o.name = "bob"` knows the `Owner`, not the meshes whose caches read it, and a write through
another package's code knows even less. Resetting every cache would need a registry of readers. As
written, the claim is false for any cached method that reads through a handle.

**It goes stale when it changes what it reads.** A cached method whose computation calls something that
writes `self.verts` empties the slot halfway, then fills it with a value computed from the old
`verts`.

**A reference into the cached value.** "Returning a reference into the cached value is safe until a
field it read changes": emptying the slot frees an owned cached value (a `String`) under that
reference unless the emptying counts as replacing the slot.

**The fix.** `@cached` may read only fields of the object's own value (no handles, no globals, no calls
with such reads), and may not write anything it reads: both are compile-time errors with the reason.
Emptying the slot is a replacing write, so a name for the cached value keeps the old value (3.2's rule)
and a loan into it is a loan the field effects already check. The reset is emitted at every write to a
field read, through any handle: that line knows the object, so the reset is complete. **Cost:** the
slot and one branch, as the report says. **What is left:** a cached value that depends on another
object is written by hand (`if self._label is None: …`), which the plain spelling allows.

### A34, clone before move for shared handles: kept

Unchanged and already in Ember. The atomic counts for objects shared between threads are the threads
group's.

### A18, initialisation: saved

**The rule as written.** "A method may run on a half-built `self` if its summary reads only fields
already set, writes only those or the ones it sets".

**The program.** Python's usual constructor helper sets fields rather than returning them:

```ember
struct Game:
    width: int
    assets: Array[String]
    sounds: Array[String]

    fn init(mut self, width: int):
        self.width = width
        self.load()                # sets assets and sounds

    fn load(mut self):
        self.assets = [f"atlas@{self.width}px"]
        self.sounds = ["boom.wav"]
```

```python
class Game:
    def __init__(self, width):
        self.width = width
        self._load()
    def _load(self):
        self.assets = [f"atlas@{self.width}px"]; self.sounds = ["boom.wav"]
# init_helper_sets_fields: ['atlas@640px'] ['boom.wav']
```

Today: `E2100: self is used before all struct fields are initialized` (`probes/init_helper_sets.em`).
Under the rule as written, "writes ... the ones it sets" lets `load` run, but `load` is compiled once,
as a method of a finished `Game`, where `self.assets = …` first drops the old list. On a half-built
`Game` the old list is whatever the stack held: `attacks.exe 6`, `access-violation on unknown address
0xffffffffffffffff`. If instead "the ones it sets" excludes unset fields, Python's helper is refused
and the report's "Python spelling compiles" covers only helpers that return values.

**The fix.** A helper called on a half-built `self` is compiled a second time for that call, with its
stores to fields unset at the call as first stores (no drop); which fields are set is known at the
call. Summaries gain "always sets" (sets on every path), so `init` knows `load` set `assets` and
`sounds`. `attacks.exe 7`: "atlas@640px". A `virtual`, `dyn` or `pub` helper of another package has no
body to copy and stays refused until every field is set (today's `[CLS-11]` rule). **Cost:** none when
running; one more compiled copy of each helper called from a constructor. **Attacked again:** a helper
that sets a field on one path only has "may set", and `init` must set the field itself (E2100, as
today); a helper calling another helper gets its copy too; a helper that reads a field it has not set
yet is refused as before.

### A21, updating a map entry: saved

The report's answers are kept; its one-lookup claim checked against `std/src/collections.em`: `get_or`
(line 811) calls `get`, and `index_set` (line 798) calls `find`, so today's spelling looks the key up
twice, and the find-or-insert is one lookup.

**Attacked: values that are not plain data.** The report's `get(k, default)` "returns the value when
the value type is `Copy`". Python's commonest form has a list:

```ember
println(groups.get("z", []))                   # today: get_or needs Array[String] to implement Copy
groups.setdefault(w[0], []).push(w)            # no setdefault
```

```python
print(groups.get("z", []))                     # [] (get_list_default)
groups.setdefault(w[0], []).append(w)          # {'a': ['ab', 'ac'], 'b': ['b']} (setdefault)
```

Today: `E2072 ... has no method named get_or ... needs Array[String] to implement Copy`
(`probes/get_list_default.em`).

**The fix.** `get(k, default)` for any value type returns a name: of the entry when present, else of
the default, which lives as long as the names made from it (the references group's rule for
temporaries, `phase3b-references.md`, section 6.3). `setdefault(k, default)` is one find-or-insert that
returns a name of the entry; it turns the key into an owned key only when it inserts, which Rust's
`entry` cannot (it takes an owned key every time). **Cost:** one lookup, as Rust. A detail for the
owner's rules: a `Map(default=0)` read of a missing key inserts it in Python (`defaultdict_read_inserts:
1`); if Ember does the same, the read is a reshape, and a loop over that map follows the containers
group's loop rules.

---

## 6. The measurements and commands

All under `target/redesign2/ownership-attack/bench/`. Every round checked
`(Get-CimInstance Win32_Battery).BatteryStatus` is 2; every run was pinned with
`cmd /c start "" /b /wait /affinity C03C03 <exe>`; order forward in odd rounds, reversed in even ones.

- Builds: `"C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=<0|1|3> ../../ownership/bench/share.c -o
  share_<n>.exe` (the report's file, unchanged); `clang -O2 share_idx.c`, `clang -O2 -DMODEL=3
  share_slab.c`, `clang -O2 -DMODEL=0 share_slab0.c`, `clang -O2 -DMODE=<0..3> alias_name.c`;
  `rustc -C opt-level=2 --edition 2021` for `share_refcell.rs`, `share_cell.rs` (the report's, copied)
  and `share_index.rs`. clang 22.1.8, rustc 1.98.1.
- Runs: `powershell -NoProfile -File run.ps1 -Out runs/<name>.txt -Rounds 5 -Exes <list>`.
- Medians: `python -I medians.py runs/<name>.txt <baseline>` (the report's script).
- Results: `runs/share_summary.txt`, `runs/share_slab_summary.txt`, `runs/alias_name_summary.txt`
  (tables in 4.2 and 3.2). Spread within a batch: the lean class ran from 267.324 to 309.502 ms in
  `runs/share.txt`; the alias programs within 2% of their medians.

---

## 7. Good parts of the pieces that were replaced

Nothing was discarded whole. These pieces of the report were replaced by fixes; what was good in each:

| Replaced piece | Why | Good part kept |
|---|---|---|
| its own list of what ends a stored value's name (shifting removal, clear, unknown store) | misses `pop`, `truncate`, `swap_remove`, `insert`, `sort`: a write after free | "the old name follows the value" stays; the containers group supplies the position rules |
| `Copy` structs copied at `x = place` | silently differs from Python (`pos.x += 1.0`) | its warning for a copy that is changed and never read again (the copy-instead-of-share design's idea) stays, for explicit `copy()` |
| detaching before drop code, for `clear()` only | the same hole through assignment | the detaching itself, now for every reshape |
| backends assuming parameters do not overlap | a wrong answer (`noalias.c`) | "the caller checks the callee's summary against what it passes" stays, for reshapes |
| `if a is b: panic` before a transfer loop | unneeded beside the containers group's loop | the observation that the conflict is exactly "the two handles are one object" |
| `@cached` reading anything | stale through handles | the automatic reset at every write of a field read, which is complete for the object's own fields |
| the half-built helper compiled once | drops garbage | "check what the helper reads, not `self` as a whole" stays |

---

## 8. What the saved answers need from the other groups

Changes to the report's section 5 table:

| From | What | Change |
|---|---|---|
| containers (A2, A3, A4) | names of list elements are positions with all of their rules 2-6, and position loops | **now required for memory safety**, not only for ease: the report's own end-of-name list is unsafe (3.1). Requested refinement: a `pop` of a named element gives the value to the name instead of being an error |
| references (A22) | a temporary lives as long as the names made from it | used by A21's `get(k, [])` |
| threads (A15) | as in the report, plus: `@cached`'s slot and any field written through a name make a class unshareable between threads unless the field is atomic or guarded | unchanged in kind |
| parent pointers (A8) | the weak side record is found by a 31-bit index or an address table, since the 8-byte header has no room for Swift's pointer; its cost measured | new detail |
| the spec's hot reload (Annex B) | summaries join the reload comparison (`[HR-3b]`, `[HR-21]`) | new (2.4) |
| the runtime | per-class blocks for objects (7% in one batch, PROVISIONAL, for the final measurer) | new, optional |

---

## 9. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/ownership-attack/`:

- `probes/`: `cppvector_span_push.em`, `copy_struct_name.em`, `copy_struct_loop.em`,
  `init_helper_sets.em`, `inherited_field_loop.em`, `get_list_default.em`, `alias_name_write.em`, and
  `ember_check_output.txt` (every probe's `ember check` output).
- `c/`: `attacks.c` (seven modes, each with the Ember program it models), `attacks_output.txt`
  (AddressSanitizer output), `noalias.c`, `noalias_output.txt`.
- `python/`: `py_attack.py`, `py_attack_output.txt`.
- `bench/`: `share_idx.c`, `share_index.rs`, `share_slab.c`, `share_slab0.c`, `alias_name.c`, copies of
  the report's `share_refcell.rs`, `share_cell.rs`, `harness.rs`, `medians.py`, the run script
  `run.ps1`, and `runs/` (`share.txt`, `share_slab.txt`, `alias_name.txt` and their `_summary.txt`).

---

## 10. The final answer list for the ownership and sharing group

| Id | Annoyance | Final answer | Kind | Cost when running (source) | What is left | Verdict |
|---|---|---|---|---|---|---|
| A1 | a method borrowing the whole object | field effects in every function's interface; declared effects where the body has `unsafe` or foreign calls; drop code runs after the new value is stored; effects keyed by the declaring class; reloads refused when a summary grows under a running caller; `drop` bounded unless declared | (b) | none | a library body change can change its callers' verdict; a `__del__`-style deregistration needs a declaration | saved |
| A9 | shared mutable state | `class` (the lean class) plus the fixed field effects; loops over a class's list across calls that may change it run by position (containers) | (a) + (b) | 1.05x to 1.10x plain C with the same objects, 1.18x from one block; Rust's `Rc` 1.29x to 1.39x; Rust and C with one array and positions 0.71x to 0.80x (`runs/share_summary.txt`, `runs/share_slab_summary.txt`; PROVISIONAL) | a removal by unseen code during a loop is the containers group's run-time panic; the fastest layout needs a struct in an `Array` and positions, written by the programmer | saved |
| A10 | choosing among ownership kinds | `struct` and `class`; `Box` implied; `Cell`, `RefCell`, `Shared` gone; a "never shared" hint for classes not visible outside the package | (a) | none beyond A9's | the copy-or-share choice | kept |
| A11 | moves, use after move, clone | names follow values: stored values' names are positions (containers' rules; a popped value stays with its name); one compare where a write may replace a named place; `Copy` structs named too | (b) | none; the compare measured at 0.88x of the code without it, the first version 1.34x, Rust's copy 1.12x (`runs/alias_name_summary.txt`; PROVISIONAL) | one value in two owning places needs `copy()` or a class; a struct replaced inside a call needs `copy()`; a `Copy` value copied into two places cannot be changed through its old name | saved |
| A24 | struct or class decides what compiles | one rule set: A1 and A11 as saved | (b) | none | copy or share | saved |
| A31 | caching in a read-only method | the plain spelling compiles; `@cached` for computations over the object's own fields, which may not change what they read, reset at every write of a field read | (b) + (a) | the slot and one branch, as hand-written C | a cached value that depends on another object is written by hand | saved |
| A34 | clone before move for shared handles | unchanged: copying a handle retains | (a) | one count increment per capture, atomic across threads (threads group) | none | kept |
| A18 | initialisation | a helper may run on a half-built object if it reads only fields already set; the constructor calls a second compiled copy of it, in which stores to unset fields are first stores; summaries say which fields a helper always sets | (b) | none (one more compiled copy per such helper) | `virtual`, `dyn` or other packages' helpers wait until every field is set | saved |
| A21 | updating a map entry | `get(k, default)` and `setdefault` for every value type, returning names; `Map(default=…)`; one lookup for `m[k] = m.get(k, d) + 1` | (a) + (b) | one lookup, as Rust | whether a read of a missing key in a default map inserts (Python does) is the owner's call | saved |

What the owner should weigh: the class mechanism is on par with C that uses the same heap objects
(1.05x to 1.18x, under or near the 15% line, for the final measurer); neither it nor Rust's `Rc` reaches
the one-array layout, which Ember gives through structs and positions. Every rule above is checked
when compiling; the only run-time additions are one compare where a name may be overwritten through
another handle (measured no slower than without it) and what the containers group already measured and argued.
