# Phase 3b, ownership and sharing: attacked again, then saved

Written 2026-10-24 by the second attacker of the ownership-and-sharing group's report
(`phase3b-ownership.md`, "the report" below), under the owner's job (`JOB.md`): speed on par with C
and Rust, memory safety the same as Rust's or better, and Python's ease, all at once, with the checking
done when the program is compiled. The job says to save an answer before discarding it, so every
problem below comes with the fix that removes it at the least cost, and the fix is attacked again.

**Why there is a second attack.** The first attacker (its report is kept unchanged in
`tasks/96/redesign/superseded/phase3b-ownership-attack-first-run.md`, its files in
`target/redesign2/ownership-attack/`) and the report's own author timed their programs through
`cmd /c start /affinity ...`. That way of starting programs opened "Windows cannot find" boxes on the
owner's screen for several agents, and a start that fails means the program never ran. The first
attack's timing files show a time and a checksum on every run, so its numbers came from real runs; the
report's author had one failed round ("The system cannot execute the specified program") and re-ran it.
This attack re-made every measurement anyway, with a runner that cannot open a window (section 7), re-ran
every memory attack and every probe it relies on, re-checked the first attacker's findings one by one,
and attacked the report again, including the first attacker's fixes. The re-made timings agree with the
first attack's within the noise (sections 4.3 and 5). Where a finding is the first attacker's, it says
so; where it is new, it says "new".

How every claim here can be traced (all my files are under
`C:/Users/ism19/Code/ember/target/redesign2/ownership-attack2/`, "my files" below):

- **Ember today**: `ember check` output (`target/ci/ember.exe check <file>`) for every probe in
  `probes/`, in `probes/ember_check_output.txt`, `probes/gen_check_output.txt`, and
  `probes/report_probes_recheck.txt` (the report's own probes, re-checked). No Ember program was built
  or run.
- **What the proposed rules would make the compiler emit**: small C programs, each with the Ember
  program it models in a comment. Memory errors are caught with AddressSanitizer (a clang option,
  `-fsanitize=address`, that stops a program with a report the moment it reads or writes freed
  memory): `c/attacks.c`, eleven modes, output in `c/attacks_output.txt`. `c/noalias.c`: output in
  `c/noalias_output.txt`.
- **Python**: `python/py_attack.py`, run with `python -I` (Python 3.14.2), output in
  `python/py_attack_output.txt`. Every "Python prints" below is from that file.
- **Timings**: mine, in `bench/`, under the job's rules (section 7 has every command). Mains power was
  checked before every round; each program ran pinned to the performance cores, one at a time, 5 rounds,
  the order reversed in every second round; medians. **Every timing is PROVISIONAL**: other agents may
  have been running (one batch was visibly disturbed, section 7), and a decision that rests on a
  difference under 15% waits for the final measurer.
- Numbers from other reports name their file and section. Nothing was written outside
  `tasks/96/redesign/` (this file and the copy of the first attack's report) and `target/redesign2/`.

---

## 0. The short version

The report's three rules hold up as ideas, and so do the first attacker's seven fixes, all of which I
reproduced. Attacking again found two more memory-safety holes (a name kept by address in a map, and a
loan kept inside a generator), two weaknesses in the first attacker's fixes, one claim of the first
attacker that was measured against the wrong baseline, one speed claim of the report that does not hold
when objects come from a slab (a block holding many objects of one size, which is how the combined
model allocates them), and two places where a Python program would silently mean something else. Every
one has a fix except the slab speed gap, for which I tried one fix that did not help; it stays open.

| Annoyance | The report's answer | Verdict | What changed |
|---|---|---|---|
| A1, a method borrowing the whole object | field effects (a function's summary of what it changes, computed from its body) | **saved** | the first attack's fixes (unsafe code declares its effects; new value stored before old one dropped; fields keyed by the class that declares them; reloads checked), plus: unsafe code that declares nothing has every effect, globals included; a value a generator walks may grow but not be moved; overlapping arguments get two compiled copies and a compare instead of a false promise to the C compiler |
| A9, shared mutable state | `class` plus field effects | **saved**, speed **open** | as A1; no run-time access words, as the combined model decided (section 3.7); the class's speed: 1.09x to 1.11x of plain C with one `malloc` per object, but 1.16x to 1.17x when both take objects from a slab (above 15%); the cost is counting at handle stores, and the one fix tried (no header in the object) did not help |
| A10, choosing among ownership kinds | two words, `struct` and `class` | **kept** | the 32-bit type index in the header measured at 0.99x to 1.03x of today's type pointer on virtual calls |
| A11, moves, use after move, clone | names follow values | **saved** | the first attack's fixes (popped values stay with their names; one compare where a write may replace a named place; `@derive(Copy)` structs named), plus: a name for a map value is its entry's position or key, never its address; the compare costs 1.00x of correct C (not "0.88x"); the fix-it `copy()` is offered beside "name it again after the call", because they mean different things |
| A24, struct or class decides what compiles | one rule set for both | **saved** | follows A1 and A11 |
| A31, caching in a read-only method | the plain spelling compiles; `@cached` | **saved** | (first attack) `@cached` may read only the object's own fields |
| A34, clone before move for shared handles | unchanged | **kept** | none |
| A18, initialisation | a helper may run on a half-built object if it reads only fields already set | **saved** | (first attack) the helper is compiled a second time for the constructor |
| A21, updating a map entry | `get(k, default)`, `Map(default=…)`, one lookup | **saved** | (first attack) `get` and `setdefault` for any value type; new: the two lookups merge only when the key expression has no effects |

The problems, by name (each is a section below):

| Problem | Found by | Kind | Evidence (my files) | Fix | Cost of the fix |
|---|---|---|---|---|---|
| the unsafe-code hole | first attack, reproduced | memory safety | `attacks.exe 3`: read after free | unsafe bodies use declared effects; **new:** undeclared means every effect, globals included | none when running |
| the drop-order hole | first attack, reproduced | memory safety | `attacks.exe 4`: read after free; `5`: fixed | store the new value, then drop the old; **new:** a cleared list gets its buffer back when the drops left it empty | none |
| the pop hole | first attack, reproduced | memory safety | `attacks.exe 1`: write after free; `2`: fixed | element names are positions with all of the containers group's rules | none |
| the map-name hole | **new** | memory safety | `attacks.exe 8`: write after free; `9`: fixed | a name for a map value is its entry position (or key), found again after a change | none on the common path |
| the generator hole | **new** | memory safety | `attacks.exe 10`: read after free; `11`: fixed | a stored loan pins its place: a walked list may grow but not move; `yield` counts as a call the compiler cannot see | a reload per resume |
| the inherited-field hole | first attack | memory safety | `probes/inherited_field_loop.em` | key effects by the declaring class | none |
| the hot-reload hole | first attack | memory safety | spec `[HR-3b]` | refuse a reload that grows a summary a running frame relies on | none when running |
| the overlap claim | first attack, reproduced | wrong answer | `noalias.exe`: 0 where Python gives 1 | **new save:** two compiled copies and a compare (the containers group's A3 rule), instead of dropping the speed | one compare where two arguments may be one place |
| the half-built-helper hole | first attack, reproduced | memory safety | `attacks.exe 6`: wild free; `7`: fixed | a second copy of the helper for the constructor | code size only |
| the compare's cost | **new** (corrects the first attack) | measurement | `runs/alias_name_summary.txt` | none needed: the fix runs at 1.00x of the correct C program | none |
| the slab speed gap | **new** (the first attack saw it once) | speed | `runs/share_summary.txt`, `share2_summary.txt`, `alloc2_summary.txt` | tried: counts beside the objects instead of in them, 1.16x again | **open** |
| the copy fix-it | **new** | Python changed | `py_attack_output.txt`: `replaced_in_call` | offer both fix-its, saying what each means | none |
| the key-twice merge | **new** | Python changed | `key_expression_calls: 2` | merge two lookups only when the key has no effects | none |
| the copied-struct divergence | first attack | Python changed | `probes/copy_struct_name.em` | `@derive(Copy)` structs are named too | none |
| the stale cache | first attack | false claim | `cached_through_handle` | `@cached` reads only the object's own fields | none |
| the second-handle refusal, setting helper, list default, drop-effects imprecision, duplicate check, Rust-comparison claim | first attack | refusals and overstatements | the first attack's report | as there | none |

---

## 1. Words used

- **Place**: where a value lives: a local variable, a field (`g.log`), a list element (`xs[3]`), a map
  entry (`m[k]`), a field reached through a handle (`cam.target.hp`).
- **Handle**: what a variable of a class type holds: a pointer to a heap object that carries a
  **count** of the handles to it; the object is freed when the count reaches zero.
- **Loan**: a pointer the compiled program holds into a place for a while: a `for` loop's pointer
  into the list it walks, a generator's pointer to the list it was given.
- **Reshape**: the report's word for a change that can free or move memory: replacing a value that owns
  memory, `push`, `pop`, `clear`, changing an enum's variant, dropping an object. Writing a number is not
  a reshape.
- **Effect summary**: what the compiler records about a function from its body: which places it
  reads, writes and reshapes. The report's first rule, **field effects**, checks loans against these.
- **Name**: under the report's second rule, **names follow values**, `x = place` makes `x` a second
  name for the value in the place, not a copy, as in Python.
- **Drop**: what runs when a value ends: freeing its memory, releasing its handles, and a class's own
  `fn drop(self)`, which is program code.
- **Position**: the containers group's way to keep a name for a list element: "element 3 of `xs`",
  looked up again after the list may have moved (`phase3b-containers.md`, section 2.3).
- **Generator**: an Ember `gen fn`, Python's generator function: calling it returns a suspended frame,
  and each `next()` runs it to its next `yield` (spec `[CORO-1]` to `[CORO-13]`).
- **Slab**: one large block from which all objects of one size or class are taken, instead of one
  `malloc` each. The combined model allocates class objects this way (`phase3b-combined.md`, 3.4).
- **`restrict` / `noalias`**: a promise to the C compiler (or LLVM) that two pointers never point at
  the same memory; it lets the compiler reorder reads and writes.
- **The lean class**: the report's class objects: an 8-byte header (a 32-bit count and a 32-bit type
  index), no per-field access words.
- **Kinds (a), (b), (c)**: the job's three kinds of answer: (a) a Pythonic way of writing what Rust
  checks; (b) a smarter compile-time rule that makes Rust's rule unneeded; (c) a new mechanism.

---

## 2. What was re-checked, and what held

Every piece of evidence the first attack and the report rest on, re-run in this session:

| Claim | Re-run | Result |
|---|---|---|
| today's Ember accepts `root.children.push(child); child.value = 5` (the write is lost) | `ember check` on the report's `a11_store_then_use.em`, `a11_store_then_read.em` | both exit 0 (`probes/report_probes_recheck.txt`) |
| a struct containing itself crashes the checker | `a10_box_inferred.em` | "has overflowed its stack", exit 127 |
| `@cached` does not exist; the Python helper in `init` is refused | `a31_cached.em`, `a18_init_variants.em` | `E0104`; `E2100` |
| today emits "release the old handle, then store the new one" | read `target/redesign2/ownership/probes/a09_emit/c/release/c/main.c` lines 482-488 | `ember_release_plain(…target); …target = _67;`: yes |
| the first attack's seven memory attacks | `attacks.exe 1` to `7` under AddressSanitizer | 1: write after free, 2: "2 0" (Python's answer), 3: read after free, 4: read after free, 5: "walked 0 elements" (Python's answer), 6: access violation at 0xffffffffffffffff, 7: "atlas@640px" (`c/attacks_output.txt`) |
| `restrict` on an accepted call gives a wrong answer | `noalias.exe` | "with noalias 0 (c.n is 1), without 1" |
| the first attack's probes | `ember check` on its seven probes, copied | the same errors and exits as its report (`probes/ember_check_output.txt`) |
| the Python behaviour both reports quote | `py_attack.py` | the same 17 lines as the first attack's output, plus mine |

Nothing the first attack found turned out wrong except the baseline of one measurement (section 4.3).

---

## 3. Field effects: attacks and saves

The rule (report, section 2.1): any field may be written through any name; a loan is broken only by a
reshape of its place; every function's summary says what it may reshape, computed from its body; `mut`
is inferred; for objects reached through handles, effects are "by class and field".

### 3.1 The unsafe-code hole (first attack), and a gap in its fix (new)

**The hole, reproduced.** The standard library's `CppVector.push_back` (`std/src/ffi.em`) writes no
field of `self`; it calls C++ code that may move the elements and free the old block. A summary
computed from that body says "reshapes nothing", so a view kept across it survives:

```ember
fn grow_while_viewing(v: CppVector[c_int]):
    s = v.span()            # a view of the vector's elements
    v.push_back(7)          # summary from the body: reshapes nothing
    println(s[0])           # reads the block push_back freed
```

```python
import array
v = array.array("i", [5]); s = memoryview(v)
v.append(7)                 # BufferError: cannot resize an array that is exporting buffers
```

Today's Ember refuses it only because `push_back` is written `mut self`
(`probes/cppvector_span_push.em`: `E3021`). The C it would run: `attacks.exe 3`,
`heap-use-after-free ... READ of size 4`.

**The first attack's fix**: in a function whose body has `unsafe` or calls a foreign function, the
declared effects are the summary (`mut self` means "may reshape anything in `self`", as Rust's
`&mut self`), and "such a function without a declaration gets 'reshapes every parameter it reaches'".

**Attacked again (new).** "Every parameter it reaches" leaves out module variables. An unsafe function
with no parameters that frees memory a module variable points into is then "reshapes nothing". A
sketch (the C library is hypothetical; the module variable is the threads group's A19 spelling,
`phase3b-threads.md` section 6):

```ember
extern "C":
    fn atlas_rebuild()                  # C: frees the atlas's pixel block and makes a new one
    fn atlas_pixels() -> Span[u32]

ATLAS: Span[u32] = atlas_pixels()      # a module variable: a view of the C block

fn reload():                            # no parameters, no declared effects
    unsafe:
        atlas_rebuild()

fn draw():
    for px in ATLAS:                    # a loan into the C block
        reload()                        # summary: "reshapes every parameter it reaches" = nothing
        plot(px)                        # reads the freed block
```

```python
# Python cannot free memory behind a list's back; a C extension can, and Python cannot check it
# either. In Rust the same program needs `unsafe` around a `static mut`, and keeping it correct is
# the unsafe code's author's job.
```

**The fix.** A body with `unsafe` or a foreign call and no declaration has **every** effect, which is
what Ember already gives code it cannot see (`[EFF-2]`, the report's rule 4). A declaration narrows it
and is checked against the body as far as the body is visible. **Cost:** none when running; such
functions refuse more until their author declares what they do, as Rust's author must write
`&mut self` or `&mut CACHE`'s owner. **Attacked again:** a safe wrapper around it takes its declared or
"every" summary; a closure with an unsafe body is the same rule; nothing else can reach memory the type
system cannot see.

### 3.2 The drop-order hole (first attack), and a cost in its fix (new)

**The hole, reproduced.** A class's `drop` that walks the list being replaced:

```ember
class Item:
    world: World
    hits: int = 0
    fn drop(self):
        for other in self.world.items:
            other.hits += 1

fn reset(w: World):
    w.items = []          # today's order: drop the old list in place, then store the new one
```

```python
w.items = []              # Python stores the new list first: the destructors walk 0 elements
```

`attacks.exe 4` (today's order): `heap-use-after-free ... READ of size 8`. `attacks.exe 5` (the first
attack's fix: take the old value out, store the new one, then drop the old): "the drops walked 0
elements", Python's answer (`replace_list_drop_walks_it: 0`).

**Attacked again (new).** For `clear()` the fix means: take the buffer out, leave the list empty, then
drop the elements. An empty list has no buffer, so the next `push` allocates a new one. The usual game
loop clears and refills the same list every frame:

```ember
for frame in range(n):
    bullets.clear()                       # Bullet has `fn drop`: the buffer is taken out
    for i in range(100):
        bullets.push(Bullet(i))           # a fresh allocation, every frame
```

```python
for frame in range(n):
    bullets.clear()                       # CPython's clear also detaches its storage
    for i in range(100):
        bullets.append(Bullet(i))
```

Today the buffer is kept across `clear()`, so the fix as written adds an allocation and a free per
frame that neither today's Ember nor C does.

**The fix of the fix.** After the drops, if the list is still empty and has no buffer (no drop added
to it), the old buffer goes back into it:

```c
/* bullets.clear(), for elements whose drop runs program code: written by me */
Vec old = bullets; bullets = EMPTY_VEC;             /* detach: a drop that reaches `bullets` sees [] */
for (size_t k = 0; k < old.len; k++) bullet_drop(&old.ptr[k]);
if (bullets.cap == 0) { bullets.ptr = old.ptr; bullets.cap = old.cap; }   /* give the buffer back */
else free(old.ptr);
```

Elements whose drop cannot run program code (numbers, strings, lists of those) are cleared in place as
today, because nothing can observe the order. **Cost:** two loads, a compare and two stores per
`clear()` of such a list; no allocation. Not timed: it is the same work as today plus those stores.

### 3.3 The inherited-field hole (first attack)

Re-checked: `probes/inherited_field_loop.em` (a loop over `s.items` while `d.items` grows, `d` a
subclass handle that may be `s`) compiles today, guarded by the field's run-time access word, which the
report removes. Python never ends this loop (`inherited_field_grows_in_loop: 6 8`, stopped at 6). The
fix (effects keyed by the class that declares the field; fields cannot be overridden, `[CLS-4]`) costs
nothing. Kept as the first attack wrote it.

### 3.4 The hot-reload hole (first attack)

Re-checked against the spec: `[HR-3b]` says "At a checkpoint the frames below it run their old code on
after the swap", and calls reach the newest body through a thunk. A frame running the old `tick`, which
holds a loan into `self.players`, calls a reloaded `note` that now pushes to `self.players`. The fix (a
reload is refused when a summary grows under a running frame that relies on it, as `[HR-3b]` already
refuses changed types) costs nothing when running. Kept.

### 3.5 The overlap claim (first attack), saved better (new)

**The claim** (report, 2.1): "Value parameters keep their 'not the same place' guarantee, checked by
the caller (rule 5), so the C and LLVM backends may still assume they do not overlap." Rule 5 checks
only reshapes, so it accepts a call that passes one struct twice to a function that writes numbers:

```ember
struct Counter:
    n: int

fn reset_then_set(a: Counter, b: Counter) -> int:
    a.n = 0
    b.n = 1
    return a.n

c = Counter(n=5)
println(reset_then_set(c, c))
```

```python
print(reset_then_set(c, c))     # 1 (reset_then_set_same: 1)
```

Today's Ember refuses the call (`probes/overlap_numbers.em`: `E3022: c is already mutably borrowed`),
which is why today's backends may promise no overlap. With the promise and the report's rule, clang
computes 0 (`noalias.exe`: "with noalias 0 (c.n is 1), without 1").

**The first attack dropped the promise.** That costs speed wherever Rust's `&mut` lets LLVM vectorise
without a run-time overlap test.

**A better save (new).** The containers group's saved A3 rule already compiles a function twice, once
assuming no overlap and once not, and compares at a call that may pass one place twice
(`phase3b-containers-attack.md`, section 9, A3 rule 2). Applied here:

```c
/* reset_then_set(x, y), when the compiler cannot prove x and y are different places */
r = (x == y) ? reset_then_set_overlapping(x, y) : reset_then_set_restrict(x, y);
```

`noalias.exe`, second line: "reset_then_set(e, e) = 1; reset_then_set(f, g) = 0 (f.n 0, g.n 1)":
Python's answer when they are one place, the fast copy when not. **Cost:** one compare at calls that
may pass one place twice, and a second compiled copy of the function; the containers attack measured
the call-site compare at "element pairs free ... two list parameters 0.98x" (its section 9, A3,
PROVISIONAL). Calls with provably different places, the common case, pay nothing.

### 3.6 The drop-effects imprecision (first attack)

Storing a handle can drop the old object, and an unknown `drop` (an interface-typed field, an `open`
class) has every effect, so every handle store would break every loan. The first attack's fix (a `drop`
may reshape only what its own object owns unless the class declares more; Ember already limits `drop`,
`[CLS-7a]`) costs nothing. Kept.

### 3.7 Run-time access words and loops over code the compiler cannot see (new: a conflict)

The report removes the per-field access words of class objects (8 bytes per object in its measured
`Player`, and their checks). The containers group's attacker, working at the same time, saved its
run-time loop mode (a loop that removes from its list and calls code the compiler cannot see) by
**putting the loop's state into that field's access word** (`phase3b-containers-attack.md`, 5.5). The
two cannot both stand. The combined model already saw this and chose the report's side
(`phase3b-combined.md`, 4.5): no run-time loop state; a loop whose body calls code the compiler cannot
see re-reads the list after each such call (Python's own iterator behaviour, its skip included); a loop
that *also* removes from that list is a compile-time error with fix-its.

```python
for h in self.handlers:
    h(self)                     # a handler may remove itself: Python skips the next one
```

```ember
for h in self.handlers:
    h(self)                     # combined model: re-reads the list after the call, as Python does
```

For this group's answers that means: "no access words" holds, and A9's residue is exactly the combined
model's refusal of a loop that removes from its own list and calls unseen code. I found no cheaper place for the loop state than the access word that keeps the
containers attacker's Python meaning for other code (other code would have to check something on
every access to a walked list); so I agree with the combined choice and record it here as part of A9.

### 3.8 The generator hole (new)

**The rules as written.** Names follow values: "When a value is stored into another place ..., its old
names name the new place" (report, 2.2, rule 3). A move is not in the list of reshapes. The report
does not mention generators. But a generator's frame holds a loan to what it was given, and that loan
is a pointer inside a value, not a name the compiler can move along with the value.

**The program.**

```ember
class World:
    items: Array[int] = []

gen fn walk(xs: Array[int]) -> Generator[int]:
    for x in xs:
        yield x

items = [1, 2]
w = World()
g = walk(items)
println(g.next())            # 1
w.items = items              # names follow values: `items` now names w.items
w.items.push(99)             # grows the list where it now lives: the buffer moves
println(g.next())            # the frame still points at the old place
```

```python
def walk(xs):
    for x in xs:
        yield x
items = [1, 2]; w = World(); g = walk(items); print(next(g))
w.items = items; w.items.append(99)
print(list(g))               # [2, 99] (gen_store_between)
```

Today's Ember refuses the store (`probes/gen_store_between.em`: `E3021: items cannot be moved while it
is borrowed`) and also the plain growth (`gen_push_between.em`: `E3021`). Read as written, the
proposal accepts the store: the value moves and its names follow, but the loan inside `g` does not.
`attacks.exe 10` models what that runs (the frame keeps a pointer to the old header, which still points
at the buffer the push freed): `heap-use-after-free ... READ of size 8`.

**The fix.**

1. A loan stored inside a value (a generator frame, a struct holding a view) pins its place: the
   place may not be moved, replaced or dropped while that loan lives. That is today's rule, and Rust's.
2. `yield` counts as a call the compiler cannot see, for every loan and name in the frame that points
   outside it. So the generator's loop walks by position and re-reads the list's length and buffer at
   each resume (the combined model's rule for a loop over code it cannot see, 4.5), and a class element used after a `yield` is retained
   across it. Then the caller may grow the list between `next()` calls, which today's Ember refuses and
   Python allows.

`attacks.exe 11` is the fixed program with the plain growth (`items.push(99)` between the calls): it
prints "1 2 99", Python's sequence (`gen_push_between: 1 [2, 99]`). The store between the calls stays
refused, with the fix-it "store it after the generator's last use, or walk `w.items`".

**Cost.** One reload of the list's length and buffer pointer per resume; a resume is already a call.
Not timed.

**Attacked again.** A generator that names a struct element across `yield` is the combined model's
"struct element after an unseen call" error; a class element is retained (Python keeps the object). A
generator walking a class field through a handle parameter holds the handle (`[CORO-12]` already
retains `self` in a class's generator method).

### 3.9 Field effects, fixed

The rule with the first attack's fixes and mine: declared effects in unsafe and foreign code, every
effect when undeclared; new value stored before any drop runs, the buffer given back after a clear;
effects on objects keyed by the declaring class; reloads refused when a summary grows under a running
frame; drops bounded unless declared; overlapping arguments by two compiled copies and a compare; a
stored loan pins its place, and `yield` is a call the compiler cannot see. The report's own attack
table (its 2.1) still holds. **Verdict: saved.**

---

## 4. Names follow values: attacks and saves

The rule (report, 2.2): `x = place` names the value; if the place gets a new value while `x` is used,
`x` keeps the old one; a stored value's old names follow it into the new place; a value is in one
owning place at a time.

### 4.1 The pop hole (first attack, reproduced)

```ember
root.children.push(child)          # child now names root.children[child_at]
root.children.pop()                # pop shifts nothing: the report's list of what ends a name misses it
child.children.push(Node(value=7)) # emitted: a write into the popped node's freed list
```

```python
root.children.append(child); root.children.pop(); child.children.append(Node(7))
print(len(child.children), len(root.children))     # 2 0 (store_pop_use)
```

`attacks.exe 1`: `heap-use-after-free ... WRITE of size 8`. The report's own emitted-code comment
assumes "nothing removed from the list since the push", so this is its rule's wording missing `pop`,
`truncate`, `swap_remove`, `insert` and `sort`. Fix (first attack): a stored value's name is a
containers-group position with all of that group's rules, and the name keeps a popped value.
`attacks.exe 2`: "2 0". Cost: none beyond the containers group's.

### 4.2 The map-name hole (new)

**The rule as written.** Rule 3 includes "put in a map", and says growth does not end a name, pointing
to the containers group's rule, which is about lists. It does not say how a name for a map value is
kept.

**The program.**

```ember
d = Map[String, Cell]()
c1 = Cell(x=0)
d["a"] = c1                          # c1 now names d["a"]
for k in range(100):
    d[str(k)] = Cell(x=0)            # insertions
c1.x = 5
println(d["a"].x)
```

```python
d = {}; c1 = Cell(); d["a"] = c1
for k in range(100): d[str(k)] = Cell()
c1.x = 5
print(d["a"].x)                      # 5 (dict_grow_then_name)
```

Ember's `Map` keeps its entries in one list and appends each new one (`std/src/collections.em`,
`push_new`, line 593), so insertions move every entry to a bigger block. If the name keeps the address
the value went to, its next use writes freed memory: `attacks.exe 8`,
`heap-use-after-free ... WRITE of size 8`.

**The fix.** A name for a map value is the entry's position in that list, found again after anything
that may change the map (`attacks.exe 9`: `d["a"].x = 5`, Python's answer). Positions stay valid
across insertions and removals, because `remove` leaves an empty slot (line 644 onward), until the map
compacts its entries (`rebuild`, line 561, which `remove` calls when half the entries are gone and
`reserve` may call when growing); a name used after anything that may compact is found again by its
key (the combined model's "run the path again", 4.2). A removal of the named key written in the same
function moves the value into the name, as for lists (Python: `dict_pop_then_name: 7 0`, the object
lives on). **Cost:** none on the common path (a position is what the compiler would keep for a list);
one hash lookup per use after a possible compaction, which is the lookup Rust's workaround writes by
hand.

### 4.3 The second-handle refusal and the cost of its compare (first attack; its measurement corrected)

```ember
fn rename(p: Player, q: Player) -> String:
    old = p.name
    q.name = "bob"          # may be p's name
    return old
```

```python
print(rename(x, x))         # ann (alias_write_same)
```

The first attack's fix: one compare at the write; when it is the named place, the old value moves into
the name's own storage. It measured the fix at "0.88x of the code without the compare" and presented
that as the cost. **That baseline is the wrong program**: the code without the compare reads the name
*after* the write, so it computes a different (wrong) answer and does different work
(`alias_name.c`, its checksum differs). The fair baseline is what a careful C programmer writes: keep
the old pointer and free the replaced block only after the name's last use, which needs no compare
and is right either way. I added it (`-DMODE=4`) and re-timed all five
(`runs/alias_name_summary.txt`, PROVISIONAL; 4,096 objects, 2^27 steps):

| Program | median ms | ratio to correct C | checksum |
|---|---|---|---|
| correct C: free the replaced block after the use, no compare | 208.897 | 1.00 | 27090862489880799 |
| **the fix: the name's value in a register, one compare at the write** | 208.017 | **1.00** | 27090862489880799 |
| the code with no compare (wrong when the places are one) | 239.538 | 1.15 | 27090862892341538 |
| the fix's first form: the name a pointer redirected to its own storage | 319.963 | 1.53 | 27090862489880799 |
| Rust's spelling: copy the field first (`.clone()`), cheap free list | 267.446 | 1.28 | 27090862489880799 |

**Restated:** the fix costs nothing against correct C (1.00x), as long as the compiler emits the
register form; the pointer form costs 1.53x and must not be emitted where the register form is valid.
Rust's copy is 1.28x.

### 4.4 The copied-struct divergence (first attack)

```ember
@derive(Copy)
struct Vec2:
    x: f64
    y: f64

pos = p.pos
pos.x += 1.0
println(p.pos.x)       # Ember today and as the report proposes: 1.0
```

```python
pos = p.pos; pos.x += 1.0
print(p.pos.x)         # 2.0 (copy_struct_name)
```

Re-checked: `probes/copy_struct_name.em` compiles today (exit 0), so the difference is silent. The fix
(`@derive(Copy)` structs are named like every other value; numbers are unaffected because a number
cannot change in place) costs nothing and is the one change of meaning to a program that compiles
today, toward Python. Kept as the first attack wrote it.

### 4.5 The copy fix-it changes meaning (new)

**The rule as written** (report, 2.2, rule 2): if a call may replace the named place, "a struct or list
value cannot be kept without a copy, so that use is a compile-time error whose fix-it is
`x = copy(place)`".

**The program.**

```ember
struct Bag:
    items: Array[int] = []
    dirty: bool = false

    fn reset(mut self):
        if self.dirty:
            self.items = []

b = Bag()
inv = b.items
b.reset()               # MAY replace b.items
inv.push(1)             # the report: error, fix-it `inv = copy(b.items)`
println(len(b.items))
```

```python
b = Bag(dirty=False); inv = b.items; b.reset(); inv.append(1)
print(len(b.items))     # 1: inv IS b.items when reset did nothing (replaced_in_call(dirty=False))
                        # 0 when reset replaced it (replaced_in_call(dirty=True))
```

With the fix-it applied, `len(b.items)` is 0 in both cases: when `reset` did nothing, Python's `inv`
was `b.items` and the push showed in it; the copy's push does not. The error is right (no static
answer gives Python's meaning in both cases), but its one fix-it silently picks one of the two
meanings.

**The fix.** The error offers both, and says what each means: "`inv = copy(b.items)` keeps the list as
it was before `reset`, whatever `reset` does; or name it after the call (`b.reset(); inv = b.items`) to
get the list `b` has then." **Cost:** none.

### 4.6 A handle variable rebound under a name (new; a clarification, no hole)

```ember
p = Player()
n = p.items           # a name for the list inside the object p points at
p = Player()          # the variable p now points at another object
n.push(1)
```

```python
p = Player(); old = p; n = p.items; p = Player(); n.append(1)
print(len(n), len(old.items), len(p.items))      # 1 1 0 (name_through_handle)
```

Python's `n` is the old object's list. The report says names are "compile-time paths", and the
combined model looks some names up again "by running the path again" (`phase3b-combined.md`, 4.2).
Running `p.items` again after `p = Player()` would push to the *new* object's list, silently. The report's
own rule 1 says the name is for "the value the place holds **now**", which is Python's answer, and its
loan rule (`[RC-5]`: a loan through a handle keeps the object) keeps the old object alive. So: a name's
path is fixed when the name is made: handles and keys on it are taken then, and a rebinding of a
variable on the path does not move the name. Today: `E3012: cannot move out of a class field`
(`probes/name_through_handle.em`). **Cost:** none (the release at `p = Player()` moves to `n`'s last
use).

### 4.7 The duplicate check (first attack)

The report's `if a is b: panic(…)` before `for it in a.items: b.items.push(it)` is not needed: the
containers group's loop over a list its body may change sees the appended items and is safe when `a`
is `b` (Python never ends that loop either). Dropped. Kept as the first attack wrote it.

### 4.8 Names follow values, fixed

Element names are containers-group positions (popped values stay with their names); map-value names
are entry positions or keys, never addresses; one compare, register form, where a write may replace a
named place; `Copy` structs named too; a name's path fixed when it is made; the replaced-inside-a-call
error offers both meanings; no `a is b` panic. **Verdict: saved.**

---

## 5. The lean class: re-measured

### 5.1 The model does what the report says the compiler would emit

I read `target/redesign2/ownership/bench/share.c` against today's emitted C
(`probes/a09_emit/c/release/c/main.c`): the "today" model has the 24-byte header, the two access words
(`_access_inventory`, `_access_pad0`, emitted lines 37-38) and a retain and release of `t` every step
(emitted lines 310 and 368). The lean model leaves out the per-step pair, which the report's rule
allows here (nothing in `t`'s life may write `Watcher.target` or drop a `Player`), keeps the count
overflow test, and drops the null test (a class handle is never null). Every program does the same
work: every run of every program printed checksum 80363660820. Nobody frees a player (the `players`
list keeps a count on each), so plain C's missing counts are not missing frees.

### 5.2 Re-measured with one `malloc` per object

Two batches, `runs/share_summary.txt` and `runs/share2_summary.txt` (PROVISIONAL):

| Program | batch 1 ms | ratio | batch 2 ms | ratio | peak KB |
|---|---|---|---|---|---|
| plain C, a pointer per watcher, one `malloc` per player | 263.731 | 1.00 | 275.191 | 1.00 | 57,356 |
| the lean class | 292.017 | 1.11 | 299.481 | 1.09 | 57,352 |
| the lean object with no counts at all (not safe; to take the cost apart) | | | 279.655 | 1.02 | 57,356 |
| today's emitted code | 470.545 | 1.78 | 471.631 | 1.71 | 65,636 |
| safe Rust, `Rc<RefCell<Player>>` | 362.843 | 1.38 | 391.522 | 1.42 | 61,928 |
| safe Rust, `Rc<Player>` with `Cell` | 346.417 | 1.31 | 372.891 | 1.36 | 61,928 |
| plain C, players in one array, a position per watcher | 201.924 | 0.77 | 195.154 | 0.71 | 47,140 |
| safe Rust, players in one `Vec`, a position per watcher | 209.231 | 0.79 | 211.365 | 0.77 | 47,488 |

The report's numbers hold: the lean class at 1.09x and 1.11x of plain C (the report: 1.08x to 1.12x),
today's code 1.71x and 1.78x (report 1.75x), Rust's `Rc` 1.31x to 1.42x (report 1.34x to 1.37x). Against
Rust's `Rc` the lean class takes 0.77 to 0.84 of the time (292.017 / 346.417 and 299.481 / 391.522);
against Rust written with one `Vec` and positions it takes 1.40 to 1.42 times as long (292.017 /
209.231, 299.481 / 211.365), as the first attack said.

### 5.3 With a slab, the gap is over 15% (new), and the fix I tried did not help

**The finding.** Plain C and the lean class have the same peak memory with `malloc` because the
Windows heap gives a 32-byte and a 40-byte request the same block: plain C's `malloc` version uses
57,352 KB against 53,284 KB when the same players sit in one block, about 16 bytes more per player
(4,068 KB / 262,144), so both requests got 48-byte blocks (inferred from the peaks, not read from the
allocator). The combined model takes class objects from
per-size slabs, where nothing hides the header (`phase3b-combined.md`, 3.4). With both programs taking
their players from one block (`share_slab0.c`, `share_slab.c`, the first attacker's):

| Batch | plain C from one block | the lean class from one block | ratio |
|---|---|---|---|
| `runs/share_summary.txt` | 230.972 ms | 270.076 ms | **1.17** |
| `runs/share2_summary.txt` | 243.196 ms | 285.025 ms | **1.17** |
| `runs/alloc2_summary.txt` | 230.326 ms | 267.254 ms | **1.16** |

The first attacker saw 1.18 in one batch. Three batches here agree, and all are above 15%: in this
workload, with the allocator the combined model uses, the lean class is not on par with C.

**What makes it slow** (`runs/alloc2_summary.txt`, base: plain C from one block, 230.326 ms):

| Program | median ms | ratio | peak KB |
|---|---|---|---|
| plain C from one block (32 bytes per player) | 230.326 | 1.00 | 53,284 |
| the lean class from one block (40 bytes: the 8-byte header) | 267.254 | 1.16 | 55,328 |
| the same 40-byte objects with no counts (not safe) | 236.813 | 1.03 | 55,332 |

The bigger object costs about 3%; counting costs about 13%. Counting here means: when a watcher is
re-targeted (one step in sixteen), the new target's count goes up and the old one's goes down. The old
target was just read, so it is in the cache; the new one is a random player, so raising its count
touches memory plain C never touches (plain C only stores the pointer).

**Trying to save it: no header at all.** If the header's size were the problem, the count could live
beside the objects instead of in them: each 64 KiB slab chunk starts with an array of its objects'
counts, objects keep plain C's 32-byte layout, and the count is found from the address
(`chunk = address & ~0xFFFF`; `slot = (address - chunk - 7168) / 32`); one class per chunk would also
give the type without a type index. `bench/share_chunk.c` (`runs/alloc2_summary.txt`):

| Program | median ms | ratio to plain C from one block |
|---|---|---|
| plain C from the same chunks | 239.755 | 1.04 |
| **counts beside the objects, no header** | 266.272 | **1.16** |
| the same chunks, no counts (not safe) | 227.585 | 0.99 |

No gain: 1.16 again, because the cost is touching the new target, not the header's bytes. Discarded;
its good part is below (section 8).

**What no fix I found removes.** Freeing a shared object when its last handle goes needs to know when
that happens; a count is the cheapest way known without a tracing collector, and a count kept with the
object must be written when a handle to it is stored. Rust's `Rc` pays the same, more (1.31x to 1.42x
here). The way to C's speed in this workload is a different layout: players as structs in one `Array`
and a position per watcher, which Ember can write today and the containers group makes Pythonic
(`t = players[ws[i]]` names the element): 0.71x to 0.77x of plain C's pointer version in C, 0.77x to
0.79x in safe Rust.

```python
t = ws[i].target          # objects shared by handle
t.hp -= 1
```

```ember
t = ws[i].target          # class Player: the lean class, 1.09x to 1.17x of plain C here
t.hp -= 1
```

```ember
t = players[ws[i]]        # struct Player in an Array, a position per watcher: plain C's fastest layout
t.hp -= 1
```

**One direction not built** (UNVERIFIED): a handle stored into a list that provably dies before the
owner of its targets, and while that owner is never shrunk (here: `ws` dies before `players`, and
nothing removes players), needs no count. That is Rust's `&'a Player` in a struct, inferred instead of
written; its ceiling is the no-count program (1.02x to 1.03x). It is the references group's kind of
rule (lifetimes of stored references), and it fails as soon as the list is stored somewhere long-lived.

**Verdict for the lean class's speed: open.** 1.09x to 1.11x of plain C with `malloc`, 1.16x to 1.17x
with a slab, PROVISIONAL; for the final measurer to confirm. The 8-byte header itself costs about 3% in
time and 8 bytes per object in memory with a slab (55,328 against 53,284 KB, +3.8% here).

### 5.4 The type index on virtual calls (new measurement)

The report replaces today's 8-byte type pointer with a 32-bit index "for virtual calls, casts and
dropping through an interface", with no cost stated. `bench/vcall.c`: 65,536 objects of 8 classes, all
32 bytes in every mode so only the way to the method differs, 2^27 calls (`runs/vcall_summary.txt`,
PROVISIONAL):

| Way to the method | mixed classes, ms | ratio | one class, ms | ratio |
|---|---|---|---|---|
| today: the header's type pointer, then the method | 1336.248 | 1.00 | 228.391 | 1.00 |
| the index, then a table of record pointers, then the method | 1349.469 | 1.01 | 235.738 | 1.03 |
| the index into a table of fixed-size records, then the method | 1364.147 | 1.02 | 225.670 | 0.99 |

Within 3% either way, under 15%: the index costs nothing measurable here. (clang turned the table of
pointers into a table of 4-byte offsets, `movslq (%r14,%rax,4)` in `clang -O2 -DMODE=1 -S vcall.c`, as
a compiler that owns the type table can.) **Kept.**

### 5.5 The count's width (new; small)

The report's count is 32 bits and an overflow panics (as today's, `[RC-1]`). Rust's count is 64 bits.
A Python list of 2^32 handles to one object (32 GiB of handles) is rare but valid. The combined model
already moves a large count into a side record (`phase3b-combined.md`, 3.4); that record's count must
then be 64 bits, not the 32 it says. **Cost:** none on the fast path (the slow path only).

---

## 6. The answers one by one

### A1, a method borrowing the whole object: saved

```python
def note(self, msg): self.log.append(f"[{self.frame}] {msg}")
def tick(self):
    for p in self.players:
        p.hp -= 1
        if p.hp <= 0: self.note(f"{p.name} died")
```

```ember
pub fn note(self, msg: String):            # summary: reshapes self.log; reads self.frame
    self.log.push(f"[{self.frame}] {msg}")
pub fn tick(self):
    for p in self.players:
        p.hp -= 1
        if p.hp <= 0:
            self.note(f"{p.name} died")    # accepted: note does not reshape self.players
```

The report's answer with section 3's fixes. Cost when running: none, except one compare at calls that
may pass one place twice (3.5). What is left: a library body change can change what its callers may do
(the report's residue), and what a running program may reload (3.4); unsafe code must declare its
effects or have them all (3.1).

### A9, shared mutable state: saved, speed open

```python
for item in hud.target.inventory:
    cam.target.log.append(item)          # another list of the same object: fine
```

```ember
for item in hud.target.inventory:
    cam.target.log.push(item)            # a different field: no check, no access word
```

`class` plus the fixed field effects; no access words (3.7). What is left: a loop that removes from its
own list and calls code the compiler cannot see is a compile-time error (the combined model's choice, 4.5); the
class's speed is 1.09x to 1.17x of plain C (5.3, open).

### A10, choosing among ownership kinds: kept

```python
class Tree:
    def __init__(self, value, left=None): self.value = value; self.left = left
```

```ember
struct Tree:                         # no Box: the pointer is implied
    value: int
    left: Option[Tree] = None
```

Two words, `Box` implied, `Cell`, `RefCell` and `Shared` gone; the type index measured at no cost
(5.4); the first attack's corrections kept (the "never shared" hint only for classes not visible outside
their package; the field that closes a cycle of two structs picked in declaration order).

### A11, moves, use after move, clone: saved

```python
matrix = []; row = []; matrix.append(row); row.append(1); print(matrix)    # [[1]]
```

```ember
matrix = []; row = []; matrix.push(row); row.push(1); println(matrix)       # [[1]]: row names matrix[0]
```

Names follow values with section 4's fixes. Cost: none; the compare where a write may replace a named
place at 1.00x of correct C (4.3). What is left: a value in two owning places needs `copy()` or a class;
a struct replaced inside a call is a compile-time error offering both meanings (4.5).

### A24, struct or class decides what compiles: saved

The sum of A1 and A11 as saved; only copy-or-share differs between the two kinds.

### A31, caching in a read-only method: saved

```python
@functools.cached_property
def radius(self): return max(abs(v) for v in self.verts)
```

```ember
@cached
fn radius(self) -> f32:
    return max(abs(v) for v in self.verts)    # reset at every write to self.verts
```

The plain spelling compiles under field effects; `@cached` with the first attack's limit (only the
object's own fields, nothing it writes), because a cached value read through a handle goes stale
(`cached_through_handle: ann! ann!`). Cost: the slot and one branch, as hand-written C.

### A34, clone before move for shared handles: kept

Already in Ember and unchanged; atomic counts are the threads group's.

### A18, initialisation: saved

```python
class Game:
    def __init__(self, width):
        self.width = width
        self._load()                  # sets assets and sounds
```

```ember
struct Game:
    width: int
    assets: Array[String]
    sounds: Array[String]
    fn init(mut self, width: int):
        self.width = width
        self.load()                   # accepted: load's copy for init stores into unset fields
```

The first attack's fix, reproduced (`attacks.exe 6` crashes on the helper compiled once, `7` prints
"atlas@640px"). Cost: one more compiled copy of each helper called from a constructor. A constructor
cannot fail with an error value in Ember (`[CLS-2]`: a `return` from `init` hands the object over),
and a panic calls `abort()` with no unwinding (`[PAN-1]`), so there is no half-built object to drop on
an error path.

### A21, updating a map entry: saved

```python
counts[w] = counts.get(w, 0) + 1
groups.setdefault(w[0], []).append(w)
```

```ember
counts[w] = counts.get(w, 0) + 1      # one find-or-insert when the key has no effects
groups.setdefault(w[0], []).push(w)
```

The report's answers and the first attack's (`get` and `setdefault` for any value type). New: the
report merges `m[k] = m.get(k, d) + 1` into one lookup "when the key expression is the same". Python
evaluates the key twice (`key_expression_calls: 2`); merging is right only when the key expression and
the key type's `hash` and `==` have no effects, which their summaries tell. Otherwise two lookups, as
Python. Cost: none.

---

## 7. The measurements and commands

All under `target/redesign2/ownership-attack2/bench/`.

**Runner.** `run.ps1` sets its own process's affinity to the performance cores (mask 0xC03C03) and then
starts each program directly (`& $exe`), so each program inherits the mask from its creation; there is
no `cmd`, no `start`, and no new window. `affinity_check.exe`, started by it, printed "process mask
0xC03C03, system mask 0xFFFFFF" (`runs/affinity.txt`). Before each round it checks
`(Get-CimInstance Win32_Battery).BatteryStatus` is 2 and stops otherwise; it ran every program once per
round, the order reversed in even rounds, 5 rounds:
`powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1 -Out runs/<name>.txt -Rounds 5 -Exes <list>`.
Medians: `python -I medians.py runs/<name>.txt <baseline>` (the report's script), saved as
`runs/<name>_summary.txt`.

**Builds** (clang 22.1.8, rustc 1.98.1):

- `"C:/Program Files/LLVM/bin/clang.exe" -O2 -DMODEL=<0|1|3|7> share.c -o share_<n>.exe` (the report's
  `share.c`, unchanged); `-O2 share_idx.c`; `-O2 -DMODEL=3 share_slab.c -o share_slab.exe`;
  `-O2 -DMODEL=7 share_slab.c -o share_slab7.exe`; `-O2 -DMODEL=0 share_slab0.c`;
  `-O2 -DMODEL=<0|9|10> share_chunk.c -o share_chunk_<n>.exe` (mine);
  `-O2 -DMODE=<0..4> alias_name.c` (the first attack's, `MODE=4` added);
  `-O2 -DMODE=<0..2> [-DONECLASS] vcall.c` (mine); `-O2 affinity_check.c` (mine).
- `rustc -C opt-level=2 --edition 2021 <file>.rs -o <file>.exe` for `share_refcell.rs`, `share_cell.rs`
  (the report's) and `share_index.rs` (the first attack's).
- Memory attacks: `clang -O1 -g -fsanitize=address attacks.c -o attacks.exe` (the AddressSanitizer
  runtime DLL copied next to it from the LLVM install), run as `attacks.exe <1..11>` with
  `ASAN_OPTIONS=abort_on_error=0`; `clang -O2 noalias.c -o noalias.exe`.

**Batches** (every summary's last column shows one checksum per program across all its runs):

| File | What | Note |
|---|---|---|
| `runs/share.txt` | the shared-object workload, 9 programs | worst run 35% above its program's median (one round of the lean class, 394.895 ms); medians are robust to one such run |
| `runs/share2.txt` | the same, 10 programs | worst run 17% above its median (the lean class) |
| `runs/alias_name.txt` | the compare's cost, 5 programs | every run within 3% of its median |
| `runs/vcall.txt` | the type index, 6 programs | worst run 10% above its median |
| `runs/alloc_namecollision.txt` | not used: two programs printed the same name, so `medians.py` merged them | renamed and re-run |
| `runs/alloc.txt` | slab and chunk layouts, 8 programs | disturbed: worst run 56% above its median, and plain C from one block ran from 214.828 to 286.500 ms; not used for any decision (its lean-from-one-block ratio was 1.10) |
| `runs/alloc2.txt` | the same, re-run | worst run 16% above its median; the decisions in 5.3 use it and the two share batches |

The spread shows the machine was shared: every number here is PROVISIONAL.

---

## 8. Good parts of what was replaced or discarded

| Replaced or discarded | Why | Good part kept |
|---|---|---|
| the first attack's "undeclared unsafe reshapes every parameter it reaches" | misses globals (3.1) | declared effects as the summary of unsafe code |
| detaching a cleared list without giving its buffer back | an allocation per clear-and-refill (3.2) | Python's drop order, now without the allocation |
| the first attack's "drop the no-overlap promise" | loses vectorisation Rust gets | none needed: two compiled copies keep both (3.5) |
| a map-value name kept as an address | write after free (4.2) | names for map values, now as entry positions or keys |
| storing a value a generator walks (names following it) | read after free (3.8) | growth between `next()` calls, which today refuses and Python allows |
| "0.88x" as the compare's cost | measured against a wrong program (4.3) | the register form of the fix, now shown at 1.00x of correct C |
| the single fix-it `copy()` | picks one of two meanings silently (4.5) | the error itself, now with both meanings |
| counts beside the objects, no header (`share_chunk.c`, mine) | 1.16x, no faster than the header (5.3) | the object's class can come from its chunk when every chunk holds one class, which would save the type index of `open` classes (not measured: UNVERIFIED); and plain C's object layout for classes whose objects are read far more often than handles to them are stored |
| the report's claims of the first attack's list (the pop hole's wording, `@cached` reading anything, the helper compiled once, the `a is b` panic) | as the first attack found | as the first attack kept them (its section 7) |

---

## 9. What the saved answers need from other groups

| From | What | Change from the report and the first attack |
|---|---|---|
| containers (A2, A3, A4) | element names are positions with all their rules; the two-copies-and-a-compare rule for calls that may pass one place twice; position loops | **new:** the A3 rule is also how this group keeps the no-overlap promise (3.5); the run-time loop mode is replaced by the combined model's three compile-time cases (3.7) |
| references (A5, A22) | a temporary lives as long as the names made from it; returned places | **new:** the uncounted stored handle (5.3) is a lifetimes question for that group, if the owner wants it pursued |
| threads (A14, A15) | as in the report | **new:** `yield` as a call the compiler cannot see for loans out of the frame (3.8), beside the threads group's rule for loans into it |
| the runtime | per-class slabs | **new:** the class costs 1.16x to 1.17x of plain C from a slab in the shared-object workload, open (5.3); a side record's count must be 64 bits (5.5) |
| the combined model | the header, the loop cases | consistent with it; its 4-byte header for classes that are not `open` gives `Player` the same 40 bytes (a 4-byte count, two 4-byte fields and a 24-byte list need 8-byte alignment) |

---

## 10. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/ownership-attack2/`:

- `probes/`: the first attack's seven probes (copied), `overlap_numbers.em`, `name_through_handle.em`,
  `replaced_in_call.em`, `gen_push_between.em`, `gen_store_between.em`; outputs
  `ember_check_output.txt`, `gen_check_output.txt`, `report_probes_recheck.txt`.
- `c/`: `attacks.c` (eleven modes, each with the Ember program it models), `attacks_output.txt`,
  `noalias.c` (with the two-copies save), `noalias_output.txt`.
- `python/`: `py_attack.py` (the first attack's checks plus 14 to 18), `py_attack_output.txt`.
- `bench/`: `share.c` (the report's), `share_idx.c`, `share_index.rs`, `share_slab.c`, `share_slab0.c`,
  `alias_name.c` (the first attack's, `MODE=4` added), `share_refcell.rs`, `share_cell.rs`, `harness.rs`,
  `medians.py` (copies), `share_chunk.c`, `vcall.c`, `affinity_check.c`, `run.ps1` (mine), and `runs/`.

The first attack's report: `tasks/96/redesign/superseded/phase3b-ownership-attack-first-run.md`.

---

## 11. The final answer list for the ownership and sharing group

| Id | Annoyance | Final answer | Kind | Cost when running (source; PROVISIONAL) | What is left | Verdict |
|---|---|---|---|---|---|---|
| A1 | a method borrowing the whole object | effect summaries in every function's interface; unsafe and foreign code use declared effects, and every effect when undeclared; new value stored before the old is dropped, a cleared list's buffer given back; effects keyed by the declaring class; reloads refused when a summary grows under a running frame; `drop` bounded unless declared; two compiled copies and a compare where two arguments may be one place; a loan stored in a value pins its place, and `yield` is a call the compiler cannot see | (b) | none, except one compare at calls that may pass one place twice (containers attack, A3: "two list parameters 0.98x") | a library body change can change its callers' verdict and what a running program may reload | **saved** |
| A9 | shared mutable state | `class` (the lean class) plus the fixed summaries; no access words; loops over code the compiler cannot see by the combined model's three cases | (a) + (b) | 1.09x to 1.11x of plain C with one `malloc` per object, **1.16x to 1.17x from a slab**; Rust's `Rc` 1.31x to 1.42x; C and Rust with one array and positions 0.71x to 0.79x (`runs/share_summary.txt`, `share2_summary.txt`, `alloc2_summary.txt`) | the slab gap (counting at handle stores; no fix found); a loop that removes from its own list and calls unseen code is a compile-time error | **saved**; speed **open** |
| A10 | choosing among ownership kinds | `struct` and `class`; `Box` implied; `Cell`, `RefCell`, `Shared` gone; a "never shared" hint for classes not visible outside the package | (a) | none beyond A9's; the type index 0.99x to 1.03x of today's type pointer on virtual calls (`runs/vcall_summary.txt`) | the copy-or-share choice | **kept** |
| A11 | moves, use after move, clone | names follow values: list-element names are positions (a popped value stays with its name); map-value names are entry positions or keys; one compare, register form, where a write may replace a named place; `Copy` structs named; a name's path fixed when made | (b) | none; the compare 1.00x of correct C, Rust's copy 1.28x (`runs/alias_name_summary.txt`) | one value in two owning places needs `copy()` or a class; a struct replaced inside a call is an error offering both meanings | **saved** |
| A24 | struct or class decides what compiles | one rule set for both | (b) | none | copy or share | **saved** |
| A31 | caching in a read-only method | the plain spelling compiles; `@cached` reading only the object's own fields | (b) + (a) | the slot and one branch, as hand-written C | a cached value that depends on another object is written by hand | **saved** |
| A34 | clone before move for shared handles | unchanged: a captured handle takes its own count | (a) | one count increment per capture, as Rust's `Arc::clone` | atomic counts: the threads group's | **kept** |
| A18 | initialisation | a helper may run on a half-built `self` if it reads only fields already set; it is compiled a second time for the constructor, where its stores to unset fields are first stores | (b) | none; code size | a `virtual`, `dyn` or other-package helper stays refused until every field is set | **saved** |
| A21 | updating a map entry | `get(k, default)`, `setdefault`, `Map(default=…)` for any value type; one lookup when the key expression has no effects | (a) + (b) | one lookup, as Rust's entry API | a `Map(default=…)` read of a missing key inserts it (Python's `defaultdict`), so it is a reshape | **saved** |
| the lean class | (the mechanism under A9, A10, A34) | 8-byte header (count, type index), no access words, counts changed only at handle stores | (b) | as A9 | 1.16x to 1.17x of plain C from a slab; tried: no header, counts beside the objects, 1.16x; untried: uncounted handles proven by lifetimes, ceiling 1.02x to 1.03x | **open** |
