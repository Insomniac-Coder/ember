# Phase 3b: attacking and saving the "returning and keeping references" answers (second attack)

Written 2026-10-24 for the owner. This attacks the answers in `phase3b-references.md` (the references
group's report, called "the answers report" below) for five annoyances: returning references (A5),
structs holding references (A6), data that points into itself (A7), a view of a temporary (A22), and
moving a field out from behind `mut self` (A23). For every problem I tried to save the answer first,
and attacked the saved version again.

This is the **second** attack on these answers. The first one (written at 07:21 today) is kept
unchanged beside this file as `phase3b-references-attack-first-run.md`. I did not take its findings on
trust: I re-ran every one of its Ember probes and Rust twins (section 2), and each of its findings
that I keep below says "first attack, rechecked". Then I looked for what it missed, and found it
missed problems in five answers (sections 3.2, 5, 6.3, 7.1, 7.2) and two faults in the C models
(section 8). This file replaces it as the group's attack report; its final answer list (section 12)
supersedes the first one's.

Nothing in the compiler, the standard library, the docs or the tests was changed. My programs are
under `C:/Users/ism19/Code/ember/target/redesign2/references/attack2/` (called `attack2/` below).

---

## 0. The short version

| Annoyance | Answer (what it does) | Verdict | What changed |
|---|---|---|---|
| Returning references (A5) | The body decides what a result borrows (nothing written) | **kept** | No hole. A writable view field (`ref mut`, `MutSpan`, at any depth) makes the result borrow `self` too; today's checker already enforces it (`y05a`, `y05c`) and the inference must keep it. |
| Returning references (A5) | A downcast of a place is a place, and a pool table keyed by the type | **saved** | First attack's four fixes (rechecked) plus one new: a slot number assigned when the program loads (hot reload, separate packages) cost 19% to 22% per read; read once per function it costs nothing (measured). The answers report's C model was not Ember's layout; the faithful model puts the table at 1.05 to 1.11 of plain C. |
| Returning references (A5) | The return type says `ref`, so the `return` need not | **kept** | No hole. Returning a call's result under `-> ref T` stays an error (`y05d`). |
| Structs holding references (A6) | A container of views is a view | **saved** | First attack's fix (type erasure keeps the limit), rechecked. |
| Structs holding references (A6) | An object holding a view carries the view's limit | **saved** | First attack's three fixes (rechecked) plus one new: a handle's drop counts as a use of the limit when the object's `drop` reads the view (Rust's drop check; Rust refuses the twin, `r06d`). |
| Data that points into itself (A7) | A value may hold views of its own heap contents | **saved** | First attack's fixes (rechecked) plus one new: a write through an index (`docs[i].text.push_str(…)`) makes no named field unusable, so `docs[j].words` with `j == i` would read freed memory; fixed by holding the container exclusively until the views are rebuilt. The cost claim "the same code as plain C" is wrong twice: bounds checks, and the measured program splits on `' '` while the Ember code calls `split_whitespace`. |
| A view of a temporary (A22) | A temporary lives as long as the views made from it | **kept** | No hole. |
| A view of a temporary (A22) | The owner follows the view | **kept** | No hole. A `mut` parameter or a field can hold no hidden owner, so those stay refused (`y22a`: today's `E3063`). |
| A view of a temporary (A22) | A view returned as an owned string reuses the temporary's buffer | **saved** | First attack's fix ("reuse only when the result fills at least half the buffer"), rechecked, then attacked: "half" must mean half of the buffer's **capacity**. Read as half its length it kept 131,072,000 bytes for 97,584 bytes of names (measured, `keep2.c`). |
| A view of a temporary (A22) | A temporary is a place for its statement | **saved** | First attack's narrowing (rechecked) plus one case it refused needlessly: a method that writes only through the views the temporary holds. |
| Moving a field out (A23) | A field held exclusively may be emptied, if every path refills it | **saved** | As written it refuses the same state machine written inside a loop over the list (`y23a`; Python runs it). Saved: a local `ref mut` (a loop element, a match binding) counts as held exclusively, and so does an element reached by index while the list is held for it. No run-time cost. |
| Moving a field out (A23) | Bindings follow their use; move at overwrite | **saved** | First attack's three fixes (rechecked) plus one new: with an inferred `ref mut` binding, `path = [0]` writes into the stored state, because assigning to a reference writes through it (`[BRW-1]`, `y23b`); Python only rebinds the name. Saved: assigning to the whole name rebinds it, as in Python. |

No answer had to be discarded. Every saved version is still checked at compile time and adds no
run-time check. All timings are PROVISIONAL (other agents may have been running), and no verdict rests
on a timing difference under 15%.

---

## 1. Words used in this report

- **Place**: something with an address that can be read and written: a variable, a field, an element.
- **View**: a reference (`ref x`, `ref mut x`), a `str` (a piece of a string's characters), a `Span`
  or `MutSpan` (a read-only or writable piece of a list), or a struct holding one of these.
- **Writable view**: a `ref mut` or a `MutSpan`, or a struct holding one.
- **Limit**: the stretch of the program during which a view must stay valid. The spec calls it a
  *region*; the compiler works it out and nothing is written.
- **Loan**: while a view of `x` is in use, `x` is lent: it may not be changed or freed.
- **Exclusive loan**: a loan that also stops everyone else from reading `x`.
- **Summary** of a function: what the compiler records about it for its callers, here what its result
  points into.
- **Heap contents**: the characters of a `String`, the elements of an `Array`, the inside of a `Box`.
  They live in a separate block, so they do not move when the `String` header moves.
- **Capacity**: how many bytes a `String`'s block can hold; its **length** is how many it holds now.
- **Handle**: a value of a `class` type: a counted pointer. Copying it gives a second handle to the
  same object.
- **Type erasure**: turning a value into an interface type (`Box[dyn AnyPool]`), so its static type no
  longer says what it really is. A **downcast** gets the real type back (`cast b as Pool[Pos]`).
- **Method table**: the table of function pointers an erased value carries (a "vtable" in C++ and Rust).
- **Slot number**: the index of a type's slot in the pool table.
- **Drop flag**: a hidden yes/no the compiler keeps when a value is moved away on only some paths
  (`[OWN-3]`).
- **Rebind**: make a name refer to a new value, leaving the old value where it was (Python's `x = y`).
- **Write through**: change the place a reference points to (Ember's `r = e` when `r` is a `ref mut`,
  `[BRW-1]`).
- **Median of 5**: the middle of five runs. **Spread**: (slowest minus fastest) over fastest.
- **PROVISIONAL**: measured while other agents may run; a choice resting on a difference under 15%
  waits for the final measurer.
- **UNVERIFIED**: I believe it but did not check it.

Spec rules are cited by their tags, such as `[LT-1a]`, in
`docs/spec-source/Ember_v0.9.10_Hardened_35.md`.

---

## 2. How I checked

- **Ember**: `C:/Users/ism19/Code/ember/target/ci/ember.exe check <file>` (it prints `ember 0.1.0`,
  `language: 0.9.10`). No Ember program was run. The outputs of my probes are in
  `attack2/probes/ember_check_output.txt`. Since the answers are not built, today's checker shows what
  is refused *today*; each attack says which rule of the *proposed* answer would accept it.
- **The first attack, rechecked**: I re-ran `ember check` on all 16 of its probes (`x05a` to `x23c`)
  and `rustc` on its 5 Rust twins. Every result is the one its report states; `x05d` (a slot of a
  `Map` of erased boxes overwritten with a box of another type) still compiles today. The outputs are
  in `attack2/probes/recheck_first_attack.txt`. I also re-ran its `py_attack.py`: it prints
  `all Python attack checks passed`.
- **Rust**: `rustc --edition 2021 --crate-type lib --emit=metadata`, rustc 1.98.1. Outputs in
  `attack2/probes/rust/rustc_output.txt`.
- **Python**: `python -I attack2/probes/py_attack2.py` (Python 3.14.2) prints
  `all Python checks passed`. It asserts every Python behaviour quoted below.
- **Timings** (`attack2/bench/`):
  - Built with `sh build.sh`: C is `"C:/Program Files/LLVM/bin/clang.exe" -O2` (clang 22.1.8) with the
    shared `target/redesign/bench/harness.h` (it calls `SetErrorMode`, so no dialog can open); Rust is
    `rustc -C opt-level=2 --edition 2021` on the answers report's own `get_rust.rs`.
  - Run with `powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1`:
    `(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 before each batch and is checked before
    every round; each process is pinned to the performance cores (affinity `0xC03C03`) right after it
    starts; one program at a time; 5 rounds; the order reversed every other round, so A and B alternate.
    Before the first batch the whole machine was at 2% to 7% load.
  - I ran the batch three times (`out/results_run1.txt` to `results_run3.txt`), summarised with
    `python -I ../../bench/summarize.py` (the answers report's script) into `out/summary_run1.txt` to
    `summary_run3.txt`. Every run of every program in a workload printed the same checksum, and the
    `get` checksum is the answers report's (16779647745309).
  - **All timings are PROVISIONAL.** The `get` programs' spreads were 1% to 19% in runs 1 and 2, and up
    to 34% in run 3. The `keep2` programs run for 1 to 5 ms and spread up to 47%; only their byte counts
    are used, and those were the same in every run.

---

## 3. Returning references (A5)

### 3.1 "The body decides what a result borrows" (answers report 3.3): kept

The answer: the compiler works out from a function's body which parameters, or which view fields of
parameters, its result may point into, and callers use that as if `@borrows` had been written.

**Memory attack: a result read through a writable view field.** If the inferred set were "what
`self.buf` views" rather than "`self`", two calls would give two writable references to one element.
The first attack tried a `ref mut` field (`x05a`, rechecked: `E3062`). I tried the two shapes it did
not: a `MutSpan` field, and a `ref mut` one struct deeper.

Ember (`attack2/probes/y05a_mutspan_field_borrows.em`, with today's `@borrows` written out to show
what the inference must produce):

```ember
struct Cursor:
    buf: MutSpan[int]

    @borrows(self.buf)
    fn cur(mut self) -> ref mut int:
        return ref mut self.buf[0]

fn main():
    xs: Array[int] = [1, 2]
    c = Cursor(buf=xs)
    a = c.cur()
    b = c.cur()          # a and b would both be writable references to xs[0]
    a += 1
    b += 1
```

Python has no reference to a list slot; the nearest program aliases the list, which is safe there
because Python never frees a list someone still holds:

```python
xs = [1, 2]
a = xs; b = xs
a[0] += 1; b[0] += 1      # xs == [3, 2]
```

Today: `E3062: the returned view points into self, and @borrows names only what self.buf borrows … a
view read through a ref mut field borrows self as well`. The same error for the deeper shape
(`y05c_inner_view_struct.em`: `struct Inner: r: ref mut Array[int]`, `struct Outer: inner: Inner`,
returning `ref mut self.inner.r[0]` under `@borrows(self.inner)`). With nothing written, the second
call is `E3025` (`y05b`). Rust refuses its twin (first attack's `r05a_cursor.rs`, rechecked:
`lifetime may not live long enough`).

**Result: no hole.** The check that today's `@borrows` passes through is the one the inference uses,
and it already covers a `MutSpan` and any depth. One thing must be written down: the spec's sentence
in `[LT-1a]` names only "a `ref mut` field"; the checker is right and the text should say "a writable
view field, at any depth" (section 10, item 3).

**Other attacks, none of which found a hole** (the first two are the first attack's, rechecked by
reading):
- *Recursion*: the set only grows from "borrows nothing", so it stops, with every place any path returns.
- *A function used as a value*: a call through it uses the callable type's rule (`[LT-7]`), never the
  body; converting a function whose set is wider is refused, as an implementation wider than its
  interface is (`E3062`).
- *A `mut self` method whose result points into an owned field*, the case where Rust refuses to turn
  the exclusive loan into a shared one (with `RefCell::get_mut`, a shared loan would let
  `borrow_mut()` run while the result is alive). The answer's set holds only whole parameters and
  view fields, and a whole `mut` parameter is lent exclusively for as long as the result lives
  (`[LT-1]`: "exclusive for `mut`"), so the loan is never weakened.

**Readability:** an error at a call can come from a line in another function; the message must name
it ("`next_word`'s result borrows `p` because line 12 returns a view of `self.buf`"). A diagnostics
requirement, not a fault.

**Cost:** none at run time; the summary exists only in the compiler.

### 3.2 "A downcast of a place is a place, and a pool table keyed by the type" (answers report 3.4): saved

The answer: `cast self.pools[k] as Pool[T]`, on a place holding a `Box[dyn AnyPool]`, gives a
reference into the box after one type check. Better, a standard-library table with one slot per type
(`self.pools[Pool[T]]`) needs no check, because its insert only ever puts a `Pool[T]` in the `Pool[T]`
slot.

Python:

```python
class Registry:
    def get(self, T, e):
        pool = self.pools[T]
        return pool.dense[pool.index[e]]
```

Ember (the answers report's):

```ember
struct Registry:
    pools: TypeTable[AnyPool]

    fn get_mut[T](mut self, e: Entity) -> ref mut T:
        pool = self.pools[Pool[T]]      # ref mut Pool[T]: no cast, no compare
        return pool.dense[pool.index_of(e)]
```

**The first attack's four fixes, rechecked.**

1. *The check compares a type id, never the method table's address.* Two method tables can be merged
   or duplicated. That is concrete on Ember's own toolchain: Ember's C backend links with MSVC's
   `link.exe` (printed by `ember check`), and Microsoft's documentation says "/OPT:ICF can cause the
   same address to be assigned to different functions or read-only data members"
   (https://learn.microsoft.com/en-us/cpp/build/reference/opt-optimizations, fetched today). Rust's
   documentation says the same of its method tables (https://doc.rust-lang.org/std/ptr/fn.eq.html, as
   the first attack quoted). Free: my faithful model below compares an id.
2. *Type erasure keeps the limit of the views inside.* `keep.push(Box(Pool[str](dense=[text[0..5]])))`
   with `text` a local (`x05c`) must be refused at the push, as Rust refuses its twin (`r05c`, `E0597`,
   rechecked). Free.
3. *The table never hands out a slot, only typed access to its contents.* Today a `Map` of erased
   boxes lets any slot be overwritten with a box of another type (`x05d`, rechecked: it compiles):

   ```ember
   _old = mem.replace(pools[1], Box(Pool[Big](dense=[])))   # the "Pos" slot now holds a Pool[Big]
   ```

   Python allows the same, and the mistake shows up later as an `AttributeError`, never as corrupted
   memory: `pools[Pos] = PoolOfBig()`. Free.
4. *Slot numbers come from the compiler*, never from a hash that two types could share. Free.

**New attack 1: a slot number known only when the program loads.** In a hot-reload build a recompiled
module must find the same pools in the same slots, so a slot number cannot be renumbered by each
build: it has to be assigned once by the running program (or at link time, across separately built
packages) and read from memory. If it is read on every get, the read sits in the loop. The first
attack measured this at +7.7% in a run whose spread was 95.7%. I measured it again in a faithful
model (new attack 2 explains "faithful"): `get2.c -DTABLE -DSLOTVAR` against `-DTABLE`, `work_ms`,
median of 5, three runs:

| run | table, slot number known when compiled | table, slot number read on every get | ratio |
|---|---|---|---|
| 1 (`out/summary_run1.txt`) | 96.248 ms | 114.727 ms | 1.192 |
| 2 (`out/summary_run2.txt`) | 95.788 ms | 116.684 ms | 1.218 |
| 3 (`out/summary_run3.txt`) | 97.096 ms | 116.310 ms | 1.198 |

That is 19% to 22% in all three runs, over the 15% line.

- **Why**: the loop is a chain of loads (slot, pool, sparse array, element), and reading the slot
  number from memory adds one more link to every get (`out/get2_table_slotvar.s`: `movl pos_slot(%rip)`,
  `cmpl n_slots(%rip)` and `shlq $4` at the top of the loop). That reason is read from the assembly; I
  did not measure it with hardware counters.
- **Fix: read the slot number once per function.** A slot number, once the running program has given
  it, never changes (a reload keeps it, which is the point of assigning it once), so the generated code
  may read and check it at function entry and keep it in a local. `-DTABLE -DSLOTHOIST` models that;
  its loop is the constant-slot loop with the slot's address in a register (`out/get2_table_slothoist.s`).
  Run 3: 96.711 ms against 97.096 ms for the constant slot, a ratio of 0.996.
- **Attacking the fix**: a function that runs while a reload adds a new type cannot use that type's
  slot, since its code was compiled before the type existed; a reload never changes an existing
  number. Release builds have no hot reload (`[PRF-2]`) and can use constants when the whole program
  is compiled together; whether Ember's C backend always compiles the whole program as one unit is
  UNVERIFIED, which is why the fix covers both cases.

**New attack 2: the C model was not Ember's layout** (the task asks that the C models do exactly what
the compiler would emit). The answers report's `get.c` (its `-DTYPED` and `-DCAST` models):
- used **one length `n`** for both bounds checks, where Ember's pool has two arrays, each with its
  own header `{ptr, len, cap}` (`runtime/ember_rt/include/ember_rt.h`, `ember_vec`), so two lengths
  and two loads;
- stored a **plain pointer** per slot, where a `Box[dyn AnyPool]` is a data pointer and a method-table
  pointer (16 bytes);
- kept the **type id inside the pool**, where an erased box finds it through its method table: one
  more dependent load before the compare.

`attack2/bench/get2.c` fixes all three. `work_ms`, median of 5, 32,000,000 `get(e).x += 1.0`, ratio
to plain C in the same run:

| model | run 1 | run 2 | run 3 |
|---|---|---|---|
| plain C (`get.c -DPLAIN`, unchanged) | 89.688 ms (1.000) | 91.100 ms (1.000) | 87.403 ms (1.000) |
| answers report's table model (`get.c -DTYPED`) | 106.597 (1.189) | 104.187 (1.144) | 104.471 (1.195) |
| **faithful table** (`get2.c -DTABLE`) | 96.248 (1.073) | 95.788 (1.051) | 97.096 (1.111) |
| answers report's cast model (`get.c -DCAST`) | 110.989 (1.238) | 109.429 (1.201) | 110.763 (1.267) |
| faithful cast (`get2.c -DCAST`) | 111.546 (1.244) | 111.370 (1.223) | 107.882 (1.234) |
| safe Rust, `Box<dyn Any>` + `downcast_mut` (`get_rust.rs`, unchanged) | 144.375 (1.610) | 143.078 (1.571) | 142.891 (1.635) |

What it shows:
- **The table is closer to C than the answers report said**: 1.05 to 1.11, not 1.21. Its own loop
  masked the entity number twice (`andl $1048575` appears twice in `out/get_typed.s`, once for each
  32-bit compare); the faithful loop does not. That is the one difference I saw in the assembly; I did
  not measure which instruction difference makes the gap, so the reason is UNVERIFIED.
- **The type check costs more than the answers report said**: faithful cast over faithful table is
  1.159, 1.163 and 1.111 (the answers report: 1.072). It is near the 15% line, so PROVISIONAL; it
  changes nothing, since the table, which needs no check, is the chosen spelling.
- **Safe Rust stays the slowest**, 1.57 to 1.64 of plain C.
- The answers report's `-DCAST` row models an array of boxes. Its Ember code indexes `self.pools`
  with a `type_id[T]()` key, which in its probe `p05c` is a `Map[u128, …]`: a hash lookup per get that
  no row includes. One more reason the table, with compiler-given slot numbers, is the answer.

**Readability:** unchanged, `self.pools[Pool[T]]` as Python's `self.pools[T]`.

**Verdict: saved.** The saved answer is the original plus: compare a type id; erasure keeps the limit;
the table never hands out a slot; slot numbers from the compiler, and where they are assigned at load
time, read once per function.

### 3.3 "The return type says `ref`, so the `return` need not" (answers report 3.5): kept

`fn at(mut self, i: int) -> ref mut int: return self.dense[i]` lends the place because the declared
result says so. Python: `return self.dense[i]`.

**Memory attack: lending a temporary.** The A22 answer 6.6 (section 6.4) makes "a value returned by a
call a place until the end of its statement". Together with this answer, `return self.total()` under
`-> ref int` would lend a value that dies at the `return`. Today's spelling of what this answer would
produce is `return ref self.total()` (`y05d_return_temporary_place.em`):

```ember
struct Acc:
    xs: Array[int]

    fn total(self) -> int:
        s = 0
        for x in self.xs:
            s += x
        return s

    fn total_ref(self) -> ref int:
        return ref self.total()
```

Python (`return self.total()`) returns the number. Today: `E0111: only a place can be borrowed … this is
a value, not a place`. Under the two answers it must still be refused; it would be, by the rule the
answers report cites ("a returned view not derived from a parameter", `E3062`), because a temporary is
not a parameter. The only thing to say in the text: 6.6's "place" is for calling a `mut self` method,
not for lending. **No hole. Cost: none.**

---

## 4. Structs holding references (A6)

### 4.1 "A container of views is a view" (answers report 4.3): saved

The answer: an `Array`, `Map`, `Set`, `Option` or `Box` of a view type is itself a view type and carries
its elements' limits, as Rust's `Vec<View<'a>>`.

Python:

```python
vs = [View(reg, 0), View(reg, 1)]
```

Ember (the answers report's):

```ember
vs: Array[View] = []                 # vs is a view: it carries reg's limit
vs.push(View(reg=ref reg, entity=0))
```

**First attack, rechecked:** a callee storing a short-lived view into the caller's list (`x06d`) is
already refused by `[TYP-15]`'s "passes that to its callers" rule (`E3063` at the call); the hole is
type erasure: a `Box[dyn …]` of views does not name its element type, so the fix "erasure keeps the
limit" (section 3.2, fix 2) is needed here too. Rust refuses the twin (`r05c`). Free.

**One wording fix:** "it carries its elements' limits, one per source" cannot mean one limit per
element (a list's length is not known when compiling). It must mean one limit per view field of the
element type (the element's region vector, `[LT-14]`), each covering every element pushed: Rust's
single `'a` in `Vec<&'a str>`. Not a hole; the text needs it.

**Readability, what remains:** Python lets `reg` change while the list exists; Ember refuses it while
the list is in use, as Rust does. **Cost:** none (limits are compile time).

**Verdict: saved** (with "type erasure keeps the limit").

### 4.2 "An object holding a view carries the view's limit" (answers report 4.4): saved

The answer: a class with a field of a view type is a view type; its handles carry the view's limit, so
a handle cannot be stored where a view cannot.

Python:

```python
class Parser:
    def __init__(self, text): self.lexer = Lexer(text)
```

Ember (the answers report's):

```ember
class Parser:
    lexer: Lexer

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))    # every handle to this Parser carries text's limit
```

**First attack's three fixes, rechecked** (each Rust twin is refused with `E0597`, rechecked):
1. *All handles to one object share one limit, never shortened by a store.* Otherwise `q = p`, then
   storing a view of a short-lived local through `p` inside a function, lets `q` read freed memory
   (`x06a`). Rust's `Rc<RefCell<Parser<'a>>>` fixes one `'a` for every copy.
2. *Converting to an interface handle keeps the limit* (`x06b`).
3. *A `Weak` handle carries the limit too* (`x06c`): a strong cycle is never freed, so `upgrade()`
   would succeed after the text is gone.

**New attack: the drop runs after the text is gone.**
Ember (paper program; today it is refused earlier, at storing a view in a class object, `E3063`):

```ember
class Parser:
    lexer: Lexer

    fn drop(mut self):
        println(f"closing {self.lexer.src}")    # the drop reads the view

fn main():
    p: Option[Parser] = None
    text: String = "let x = 1"
    p = Some(Parser(lexer=Lexer(src=text)))
    println(p.unwrap().lexer.src.len())
    # end of main: locals drop in reverse order of declaration, so `text` is freed first,
    # then `p`, whose release runs Parser.drop, which reads the freed text
```

Python (works: the object keeps the text alive):

```python
p = None
text = "let x = 1"
p = Parser(Lexer(text))
```

The answer's rule is about where a handle may be *stored*; every use of `p` above is inside `text`'s
limit, so nothing in it refuses the program. The read happens in the drop. `[DRP-7]` says "a value
whose drop may read through a reference it holds must be dropped while that reference's region is
valid", but a handle holds no reference itself: its drop is a count release, and only the last
release runs `Parser.drop`. Rust refuses the twin (`attack2/probes/rust/r06d_dropck_handle.rs`):
``E0597: `text` does not live long enough … borrow might be used here, when `p` is dropped and runs
the destructor for type `Option<Rc<Parser<'_>>>` ``.
- **Fix:** dropping a limit-carrying handle counts as a use of the limit when the class's `drop` (or a
  drop of anything it holds) may read the view, as `[DRP-7]` already treats a generic type parameter.
  Here `text` must be declared before `p`, or `p` set to `None` before the end. Free: compile time only.
- **Attacking the fix:** an object in a strong cycle is never freed (`[WK-1]`), so its drop never runs
  and nothing reads; if a cycle collector is ever added, it must not run the drop of an object whose
  limit has ended.

**Readability:** Rust's `Rc<RefCell<Parser<'a>>>` without the spelling; the error must name the view
field that makes the class carry a limit.

**Verdict: saved** (one limit per object; erasure and `Weak` keep it; a handle's drop is a use of it
when the object's drop reads the view).

---

## 5. Data that points into itself (A7): "a value may hold views of its own heap contents" (answers report 5.3): saved

The answer: in `Document(text, words)`, `words` may point into `text`'s characters, because they do
not move when the `Document` moves. Any write to `text` makes `words` unusable until it is rebuilt,
and all fields must be usable again before anything else can see the value.

Python:

```python
class Document:
    def __init__(self, text):
        self.text = text
        self.words = text.split()
```

Ember (the answers report's):

```ember
struct Document:
    text: String
    words: Array[str]          # may point into text's characters

fn load(owned text: String) -> Document:
    return Document(text=text, words=text.split_whitespace().collect())
```

**First attack's fixes, rechecked by reading** (its probes `x07a`, `x07b`, `x07c` are refused today at
an earlier line, `E3063`, as it said):
1. *A view copied out of the value borrows the field it views.* Otherwise `w = d.words[0]`, then
   `docs.push(d)`, then `docs.clear()` frees the characters `w` points into (`x07a`).
2. *Through a class handle, between the write and the rebuild: no call that runs program code, and no
   access to that field through any other expression.* Otherwise `h2.doc.text = "zzz"` followed by
   `println(h1.doc.words[0])`, with `h1` and `h2` the same object, reads freed memory (`x07b`). The run-
   time access word does not catch it: a field assignment's access lasts only "the store and the drop
   of the old value" (spec VIII.3 table, line 2700).
3. *A function's summary records which view fields of a `mut` parameter it leaves pointing into the
   parameter's own fields* (today's `[LT-35]` already does; `x07c`).
4. *The derived clone re-points a view only when it lies inside the old buffer, ends included*
   (`base <= p` and `p + n <= base + len`).

**New attack: a write through an index.** Rule 2 makes "every field that may view it unusable until
that field is assigned again". For `docs[i].text`, the field is `docs[i].words`, but the compiler cannot
tell `docs[i]` from `docs[j]`.
Ember (`attack2/probes/y07a_indexed_source_write.em`):

```ember
fn edit(mut docs: Array[Document], i: int, j: int):
    docs[i].text.push_str(" and more words")   # may move docs[i]'s characters; docs[i].words is unusable
    println(docs[j].words[0])                  # j == i at run time: reads the freed characters
    docs[i].words = docs[i].text.split_whitespace().collect()
```

Python (asserted in `py_attack2.py`: prints the old word, stale but valid):

```python
docs[i].text += " and more words"
print(docs[j].words[0])                 # "the"
docs[i].words = docs[i].text.split()
```

Today this is refused at the first line that stores views in an `Array` (`E3063`); the answer lifts
exactly that refusal. The answer's own state-machine rule (A23, its 7.4 attack 8) refuses moves through
an index for this reason, but A7 has no such clause.
- **Fix (free):** a write that makes views unusable through a path with an index (or a reference held
  in a field) holds the container exclusively from that write until the views are rebuilt, as if the
  program had written `d = ref mut docs[i]` and used `d` for both lines. So `docs[j]` in between is a
  loan conflict, refused, and the rebuild must go through the same element (`docs[i]` with `i`
  unchanged). This is the same rule section 7.1 uses for A23, so the two keep sharing one analysis, as
  the answers report's section 8 intends.
- **Attacking the fix:** `i` reassigned before the rebuild: `docs[i]` then names another element, which
  is an access while the hidden loan is live: refused. A call in between that takes `docs`: refused by
  the loan. A panic in between: Ember aborts (`[PAN-1]`), nobody sees the list. The Python two-line
  spelling (`docs[i].text += …; docs[i].words = …`) is still accepted.

**New attack 2: is "heap contents never move" true?** The answers report asks the containers group
for "a guarantee that `String` and `Array` never store their contents inline". Ember already puts some
contents outside the heap: ADR-141 (`docs/DECISIONS.md`, "a short list or string a function keeps to
itself starts in its frame") gives a local `Array` or `String` a 64-byte buffer in the function's frame.
I checked whether that breaks the answer. It does not: ADR-141 gives the buffer only when every use of
the list is on an allow-list, and "a list that leaves or recurses gets no frame buffer (returned, moved
into a field, recursive)". `Document(text=text, …)` moves `text` into a field, so its characters are on
the heap. A view held in a local of the same function dies before the frame does. So the guarantee A7
needs is exactly: *contents are never inside the header, and a frame buffer is used only for a list
that never leaves its function*, which ADR-141 states. Nothing to fix; the request to the containers
group should be worded that way.

**Speed attack: the cost claim.** The answers report says this answer "emits the same code" as plain
C (`words_plain.c`, 1.000). It does not, for two reasons:
1. *Bounds checks* (first attack, rechecked by reading `words_ember.c`): the random read does
   `words[j]` and `w[0]`, both checked in Ember. The first attack measured 1.046 of plain C
   (`attack/bench/out/summary.txt`: 548.358 ms against 524.168 ms, spread 3.3% and 1.3%), PROVISIONAL.
2. *The split* (new): the Ember code calls `split_whitespace()`; the measured C and Rust programs split
   on `' '` only (`words_plain.c` line 17, `t[i] == ' '`; `words_rust_vecstr.rs` line 8, `split(' ')`).
   Ember's `split_whitespace` checks Python's whole set of space characters for every character
   (`std/src/string.em`, `is_split_space`), and its own comment says "MSVC kept it a call (1.5x the C
   loop)". I did not re-measure: a C program doing Python's `split()` does the same check, so the like-
   with-like comparison is unchanged; but the row "1.000, this answer emits the same code" is not a
   measurement of the program the report shows.

Neither is a cost of *this answer*: the same Ember program with the text held outside the value pays
both. Correct claim: "no cost over the same Ember program with the text kept outside".

**Readability:** Python's shape. What it gives up: the answers report's list, plus: a view copied out
must not outlive a move of its value (fix 1), and a write through an index must be followed by the
rebuild before the list is used again (new fix).

**Verdict: saved** (first attack's four fixes, plus: a write through an index holds the container until
the views are rebuilt; the cost claim corrected).

---

## 6. A view of a temporary (A22)

### 6.1 "A temporary lives as long as the views made from it" (answers report 6.3): kept

Python: `word = line.lower().strip()`. Ember: `word = line.to_lower().trim()`; the lowered `String`
becomes a hidden local of the block, which is exactly the hand fix (`p22a`) today's checker checks.
Attacks (the first attack's, rechecked by reading): a temporary made on one branch needs the drop flag
`[OWN-3]` already has; a temporary whose drop runs program code keeps today's error ("a temporary with
a `drop` of its own" should read "a temporary whose drop is not quiet", `[EXC-19]`).

**What remains, against Python:** the whole temporary stays alive to the end of the block (or to the
view's last use, if the other group's last-use drops are adopted). Python's `strip()` makes a new
string and frees the lowered one at once. With `first = read_file(path).split(",").next()`, Ember keeps
the whole file while `first` is in use; Python keeps one field. Rust's hand fix (`let`) keeps it too,
so this is on par with Rust. **No hole. Cost: none** (the hand fix's code).

### 6.2 "The owner follows the view" (answers report 6.4): kept

Python:

```python
for line in lines:
    w = line.lower().strip()
    if len(w) > len(best): best = w
```

Ember (the answers report's): the same loop; `best = w` moves the lowered line into a hidden owner
beside `best`. It is a rewrite into `p22d`, which today's checker accepts.

**Attack: the outer variable is not a local of this function.**
Ember (`attack2/probes/y22a_owner_beside_param.em`):

```ember
fn pick(mut best: str, lines: Array[String]):
    for line in lines:
        w = line.to_lower().trim()
        if w.len() > best.len():
            best = w
```

Python (asserted): `best = w` rebinds the parameter only; the caller's variable keeps `""`.
A `mut` parameter's storage is the caller's (`[FN-1]`), so there is nowhere to put a hidden owner that
lives as long as it. Today's checker refuses it, and must keep refusing it: ``E3063: a view of `a local`
is stored in `best`, which the caller owns``. The first attack found the same for a field
(`self.best = w`). **Not a hole**, provided the rule says "a local variable of this function", and
the message offers `String(w)` and returns the view instead. The same probe shows today's defect 1 of
the answers report again: ``E3021: `_14` cannot be written while it is borrowed`` names a compiler-made
temporary (section 10).

**Cost:** none against C doing the same work, plus a drop flag where the assignment is on some paths
only (`[OWN-3]`).

### 6.3 "A view returned as an owned string reuses the temporary's buffer" (answers report 6.5): saved

The answer: `return line.to_lower().trim()` with `-> String` moves the trimmed characters to the front
of the temporary's buffer and returns that buffer.

Python:

```python
def first_word(line): return line.lower().strip()
```

**First attack, rechecked by reading its program and results:** as written, the returned string keeps
the temporary's whole buffer. Keeping the first word of 200 lines of 1 MiB kept 209,715,200 bytes for
1,001 bytes of words (peak 210,080 KB against 5,504 KB for a copy), and for short lines it was not
faster than the copy (1.051 of it). Its fix: **reuse only when the result fills at least half the
buffer**, otherwise make the exact-size copy (memory equal to the copy, time 0.983 to 0.849 of it,
PROVISIONAL). Source: `attack/bench/out/summary.txt`.

**New attack on the saved rule: half of what?** The first attack's `keep.c` always made a buffer of
exactly the line's length, so length and capacity were equal. A real buffer often has spare capacity:
a line buffer a reader keeps, a `String` built by pushes. Ember (paper program, modelled in
`attack2/bench/keep2.c`):

```ember
fn read_name(f: File) -> String:
    buf = String.with_capacity(65536)      # a line buffer
    f.read_line(buf)
    return buf.trim()                      # the trimmed name is most of the line's length
```

Python (asserted: the result's size is the name's, under 100 bytes):

```python
def read_name(f): return f.readline().strip()
```

`keep2.c` keeps 2,000 such names (lines of 20 to 80 bytes, up to 2 spaces at each end) and prints the
bytes the kept strings hold. Every run printed the same numbers (`out/results_run1.txt`; the peak commit
is the memory the process holds whether or not it has touched it, median of 5 in run 1):

| rule | kept bytes | peak commit | peak working set |
|---|---|---|---|
| reuse when the result is at least half the buffer's **length** | 131,072,000 | 129,432 KB | 18,784 KB |
| reuse when the result is at least half the buffer's **capacity** | 97,584 | 744 KB | 3,668 KB |
| always the exact copy (today's `.to_string()`) | 97,584 | 744 KB | 3,668 KB |

Read as "half its length", the saved rule keeps 64 KiB for every name: 1,343 times the bytes (131,072,000
over 97,584). That is the same failure the first attack fixed, coming back through spare capacity.
- **Fix:** compare against the buffer's **capacity**: reuse only when the result is at least half of
  it. A kept result then never holds more than twice its own length, whatever the buffer's history.
  Cost: the same one compare (the capacity is in the header beside the length).
- **Attacking the fix:** the worst case is a result of just over half the capacity, which holds at
  most twice its length, the same bound as an `Array` after doubling growth.

**Two precisions the answer needs (free, no hole):**
- *For `Array[T]` with a `T` that has a drop*, the elements outside the view must be dropped, not just
  forgotten; and when the rule chooses the "copy", it must be a **move** of the viewed elements into an
  exact-size block (the old block is about to end, so nothing needs `Clone`).
- *ADR-141's frame buffers*: a returned block must be a heap block. It always is: ADR-141 gives a frame
  buffer only to a list that never leaves its function, and a reused buffer leaves (read in
  `docs/DECISIONS.md`, ADR-141).

**Verdict: saved.** Reuse only when the result is at least half the buffer's capacity; otherwise move
the viewed part into an exact-size block and free the old one.

### 6.4 "A temporary is a place for its statement" (answers report 6.6): saved, narrowed

The answer: a `mut self` method may be called on a value returned by a call, as Rust allows, so
`line.split(" ").next()` compiles (`p22e`, today `E2140`).

Python: `next(iter(line.split(" ")), "")`.

**First attack, rechecked** (`x22a` is `E2140` today): with a copy-returning getter, Python's
`reg.get(0).heal(100)` heals the stored hero, while Ember would heal a copy and lose it, silently,
because the call's result is used and the answer's lint stays quiet. Its fix: allow a `mut self` call on
a temporary only when (1) the temporary is an iterator, or (2) the call's result borrows the temporary
(a builder's `with_x(mut self) -> ref mut Self`); otherwise `E2140` with a help that says the change
would be lost.

**Attacking the narrowed rule:** it refuses one Python habit with no lost change:

```ember
struct Cursor:
    buf: MutSpan[int]

    fn bump_first(mut self):
        self.buf[0] += 1          # writes only through the view it holds

fn cursor(mut xs: Array[int]) -> Cursor:
    return Cursor(buf=xs)

fn main():
    xs: Array[int] = [1, 2]
    cursor(xs).bump_first()       # the change lands in xs, which the user can see
```

```python
Cursor(xs).bump_first()           # changes xs[0]
```

- **Fix: a third allowed case:** (3) the method writes only through the views the temporary holds, by
  its summary of which fields it writes (`[LT-35]`). A method that also writes the temporary's own
  fields (`self.pos += 1`) stays refused, since that part would be lost and a stored copy of a cursor
  is exactly where a user expects it to persist. Free.

**Verdict: saved** (narrowed to iterators, results that borrow the temporary, and methods that write
only through the temporary's views).

---

## 7. Moving a field out from behind `mut self` (A23)

### 7.1 "A field held exclusively may be emptied, if every path refills it" (answers report 7.3): saved

The answer: inside a function, a place reached from a `mut` parameter or from `mut self` by field
names only may be moved out of, if every path puts a value back before anything else can see it. "The
path may not go through an index, a class handle, a `Shared`, or a reference stored in a field."

**What the first attack checked, rechecked by reading the spec:** a panic in between aborts
(`[PAN-1]`); a class object's field is covered by its access word; a `static` is immutable (`[STA-1]`);
there is no `async`; a `drop` method must refill too. One thing it left UNVERIFIED stays so:
`@should_panic` tests must run in their own process (`[TST-3]` says "each isolated"), never catch the
panic in-process.

**New check: a `defer` block.** A `defer` runs at the block's exit (`[GRM-7]`), after the lines that
might have emptied a place. Does the checker see that? On a local, where `[OWN-3]`'s tracking already
exists (`attack2/probes/y23c_defer_reads_moved.em`, `y23d_defer_reads_moved_field.em`):

```ember
fn main():
    p = Person(name="ann")
    if true:
        defer:
            println(p.name)
        old = p.name           # moves the name out; the defer would read the empty field
        println(old)
```

Today: ``E3040: `p` is borrowed after it has been moved out of``, labelled at the `defer`'s read. The
analysis already runs `defer` bodies at the exit, so the extended rule inherits it. Python has no
`defer`; its nearest, `try/finally`, reads `p.name` (still `"ann"`, since Python never empties it).
**No hole.**

**New attack (readability): the same state machine written in the loop.** The answers report's Ember
program puts the `match` in `Npc.update(mut self)`. A Python programmer writes it in the loop over the
list.
Ember (`attack2/probes/y23a_loop_element.em`):

```ember
struct World:
    npcs: Array[Npc]

    fn tick(mut self, hurry: bool):
        for npc in self.npcs.iter_mut():
            match npc.state:
                State.Walking(ref mut path):
                    path.pop()
                    if path.is_empty():
                        npc.state = State.Idle
                    elif hurry:
                        npc.state = State.Running(path)
                _:
                    pass
```

Python (asserted in `py_attack2.py`):

```python
def tick(npcs, hurry):
    for npc in npcs:
        if isinstance(npc.state, Walking):
            npc.state.path.pop()
            if not npc.state.path:
                npc.state = Idle()
            elif hurry:
                npc.state = Running(npc.state.path)
```

`npc` is a `ref mut Npc` held by the loop, reached through the list, not "from a `mut` parameter or
from `mut self` by field names only", so the answer as written refuses it. Today it is
``E3013: cannot move out of a reference`` at `State.Running(path)`; Rust refuses its twin with `E0507`
(`r23a_loop_element.rs`). The answer fixes the method form and leaves the most common form as it is.
- **Fix (free): a local `ref mut` counts as held exclusively.** A place reached by field names from a
  `ref mut` this function holds (a loop element from `iter_mut()`, a `match` binding, `r = ref mut
  xs[i]`) may be emptied, and must be refilled before that reference's last use, before the iteration
  ends, and on every `break`, `continue` and `return`. While the reference is live its loan is
  exclusive, so nothing else can reach the place (`[BRW-1]`).
- **And through an index without a name:** `self.npcs[i].state` emptied directly holds `self.npcs`
  exclusively until it is refilled through the same `self.npcs[i]` with `i` unchanged, the same rule as
  A7's in section 5. So the answer's "E3011 stays" for indexes (its 7.4 attack 8) is no longer needed.
- **Attacking the fix:** a call to `self.helper()` while `npc.state` is empty: the loop holds
  `self.npcs` exclusively, so a helper that uses `npcs` is a loan conflict (refused) and one that uses
  only `log` cannot see the element. `break` before the refill: refused. A panic: abort. Threads: the
  element is reachable only through the exclusive loan.
- **Cost:** none. The loop form emits the method form's code (`state.c`'s `update` is already called
  per element, `update(&npcs[i], …)`).

One more Python habit is in the way, and it belongs to the borrowing group: today
`for npc in self.npcs:` gives read-only elements (`[CTL-1]`), so even `match npc.state` with a
`ref mut` binding is ``E3021: cannot write through a shared reference … this is `ref Npc` ``
(`y23e_loop_plain_for.em`), and the help does not mention `.iter_mut()`, the spelling that works. The
idea of the next answer, "bindings follow their use", would cover it: a loop element written in the
body is `ref mut`.

**Verdict: saved** (a local `ref mut` and an index held for the duration count as held exclusively).

### 7.2 "Bindings follow their use; move at overwrite" (answers report 7.4): saved

The answer: a `match` binding is a shared reference if the arm only reads it, `ref mut` if the arm
writes through it, and a move if the arm moves it. A name bound into a place keeps its old value when
the place is overwritten ("move at overwrite").

Python:

```python
def rename(p, new_name):
    old = p.name
    p.name = new_name
    p.log.append(f"renamed {old}")     # "renamed ann"
```

Ember (the answers report's):

```ember
fn rename(mut p: Person, owned new_name: String):
    old = p.name                  # refers to p.name ...
    p.name = new_name             # ... whose old value moves into `old` just before this overwrite
    p.log.push(f"renamed {old}")
```

**First attack's three fixes, rechecked by reading** (its probes `x23a`, `x23b`, `x23c` are `E3013`
today, as it said):
1. *Every live reference made from that loan is re-pointed together*, or the overwrite is a loan
   conflict (`alias = old` would otherwise see the new name).
2. *A drop flag* where the overwrite happens on some paths only (`[OWN-3]`'s existing mechanism).
3. *Not across a type with its own `drop`* (it would move the drop later than Python's).

**New attack (readability): assigning to the whole binding.**
In Ember, assigning to a reference writes through it: "A reference local is never re-seated: `r = e`
writes through `r`" (`[BRW-1]`). Once the answer infers `ref mut` for a binding the arm writes through,
an assignment to that binding writes into the stored state.
Ember (`attack2/probes/y23b_assign_to_binding.em`, written with today's explicit `ref mut`, which is
what the answer infers; it compiles today):

```ember
fn peek_or_default(mut self) -> int:
    match self.state:
        State.Walking(ref mut path):
            if path.len() > 3:
                path = [0]                 # writes through: the NPC's stored path becomes [0]
            return path.len()
        _:
            return 0
```

Python (asserted: returns 1, and the stored path still has 5 steps):

```python
def peek_or_default(npc):
    if isinstance(npc.state, Walking):
        path = npc.state.path
        if len(path) > 3:
            path = [0]                     # rebinds the local name only
        return len(path)
    return 0
```

Today the user writes `ref mut path`, so the reader can see the write-through. With the answer's
inference the same lines, without `ref mut`, silently change the NPC: the Python habit "assigning a
name never changes the object it came from" breaks, and so does `old = p.name; old = old.upper()`
(asserted in Python: `p.name` stays `"ann"`), since the answer takes `old = p.name` to be a reference
into `p.name` too (its section 9, under A11). Rust refuses its twin as a type error and suggests the
explicit form (`r23b_assign_binding.rs`: ``E0308 … expected `&mut Vec<i64>` … consider dereferencing
here: `*path = vec![0]` ``), so in Rust the write-through is always visible.
- **Fix: assigning to the whole name rebinds it, as in Python.** A binding the user did not write as a
  reference (a `match` binding, `old = p.name`) is changed through its methods, elements and fields
  (`path.pop()`, `path[0] = 1`; Python changes the shared list the same way), but `path = value` gives
  the name a new value and leaves the place alone. From then on the name owns its new value: a hidden
  slot for it, and a drop flag where the paths merge (`[OWN-3]`). A binding written `ref mut path` by
  hand keeps today's write-through meaning, so nothing that compiles today changes meaning.
- **Attacking the fix:** `path = [0]` and then `path.push(1)`: the push goes to the new list, as in
  Python. A rebind on one path and a write through on another (`if c: path = [0]` then `path.pop()`):
  after the merge the name is either the place or its own list; the pop goes to whichever it is, as in
  Python; the drop flag says whether to free. A rebind followed by an overwrite of the place: nothing
  needs to move, since the name no longer refers into it.
- **Cost:** none when there is no rebind; with a rebind, one stack slot and the drop flag `[OWN-3]`
  already uses.

**Speed (first attack, rechecked by reading its programs):** what the saved answer emits is
`state_inplace.c` (the binding is `ref mut`, `pop` works in place, a transition changes only the tag).
It measured 482.251 ms against plain C's 475.109 ms (1.015) and Rust's in-place spelling 472.949 ms
(0.995), with Rust's `mem::replace` idiom 572.097 ms (1.204): `attack/bench/out/summary.txt`,
PROVISIONAL. I did not re-time it: nothing in my fixes changes the code it emits.

**Verdict: saved** (first attack's three fixes; assigning to the whole name rebinds it).

---

## 8. Checking the C models against what Ember would emit

The task asks that the C models do exactly what the compiler would emit and exactly the work. What I
found, model by model:

| Model (answers report) | Does it do Ember's work? | What I did |
|---|---|---|
| `get.c -DTYPED`, `-DCAST` (A5) | No: one length for two arrays, a plain pointer per slot, the type id in the pool instead of the method table; and `-DCAST` has no `Map` lookup though the Ember code's `self.pools[type_id[T]()]` is a `Map` in `p05c` | Wrote `get2.c` with Ember's `{ptr, len, cap}` headers, 16-byte erased boxes and the id in the method table; re-measured (section 3.2) |
| `words_plain.c` as "what A7 emits" | No: no bounds checks (first attack), and it splits on `' '` where the Ember code calls `split_whitespace` (new) | Corrected the claim (section 5); the comparison with the text held outside is unchanged |
| `state.c`, `state_checked.c` (A23) | Close. The path elements are 4-byte `int32_t`; the Ember program declares `Array[int]`, which is 8 bytes. Like with like needs `Array[i32]` in the Ember program; no ratio changes | Noted |
| `keep.c` (first attack, A22 6.5) | Yes for what it tests, but every buffer's capacity equals its length, which hid the capacity case | Wrote `keep2.c` (section 6.3) |
| All of them | The checksum additions (`sum += …`) use C's wrapping arithmetic; Ember checks integer overflow by default in every profile (spec X.4, `[COST-3]` table, "Overflow check on integer arithmetic") | Noted for the final measurer: this is the benchmarks' own bookkeeping, paid the same by every Ember row, not a cost of any answer |

---

## 9. Answers not chosen by the group

I did not attack the answers the group kept but did not choose (accessors that `yield`, counted views)
or the ones it discarded; the task is the chosen answers. The counted-views figures (1.039 of the time,
1.000 of the memory) are the answers report's own measurement, not re-run.

---

## 10. Defects found in passing (today's compiler and spec; nothing was changed)

1. **A compiler-made name in a message, again.** `y22a_owner_beside_param.em` gets
   ``E3021: `_14` cannot be written while it is borrowed``: the answers report's defect 1, in a second
   program.
2. **A reborrow through a `ref mut` field cannot be written, and the message says "moved".** With
   `struct Inner: r: ref mut int`, both `return ref mut self.inner.r` (`y05e`) and `return self.inner.r`
   (`y05f`) are ``E3013: cannot move out of a reference … moved out here``, though the first spelling
   asks for a borrow, and the help ("borrow it, `ref` …") is what the user wrote. Rust accepts the same
   function, `&mut *self.inner.r` (`r05e_reborrow_refmut_field.rs`, compiles), so here Ember is worse
   than Rust.
3. **The spec's `[LT-1a]` names only a `ref mut` field.** The checker also applies "a result read
   through it borrows `self` as well" to a `MutSpan` field (`y05a`) and to a `ref mut` inside a view
   struct field (`y05c`), which is what safety needs; the text should say "a writable view field, at any
   depth".
4. **A help that does not name the fix.** `for npc in self.npcs:` with a body that writes `npc.state`
   gives ``E3021: cannot write through a shared reference … take the borrow as `ref mut` if the callee
   must write`` (`y23e_loop_plain_for.em`); the spelling that works is `self.npcs.iter_mut()`
   (`[CTL-1]`), which the help does not mention.
5. **The first attack's two defects still reproduce** (rechecked): `from std import mem` reports an
   error inside the standard library's `lib.em` (`d01`), and `mem.swap(pools[1], pools[2])` gets a help
   about methods of `self` (`d02`).

---

## 11. Files

All under `C:/Users/ism19/Code/ember/target/redesign2/references/attack2/`.

- `probes/y*.em`: my Ember probes; `probes/ember_check_output.txt`: their `ember check` outputs (three
  probes were rewritten once; the later entry marked "(rerun)" is the one cited).
- `probes/recheck_first_attack.txt`: the first attack's 16 Ember probes and 5 Rust twins, re-run.
- `probes/rust/r*.rs` and `probes/rust/rustc_output.txt`: my Rust twins and their outputs.
- `probes/py_attack2.py`: the Python behaviour of every program here, asserted.
- `bench/get2.c` (`-DTABLE`, `-DTABLE -DSLOTVAR`, `-DTABLE -DSLOTHOIST`, `-DCAST`), `bench/keep2.c`
  (`-DHALF_LEN`, `-DHALF_CAP`, `-DCOPY`), `bench/build.sh`, `bench/run.ps1`.
- `bench/out/results_run1.txt` to `results_run3.txt` (every run), `summary_run1.txt` to
  `summary_run3.txt` (medians), and the assembly read above: `get2_table.s`, `get2_cast.s`,
  `get2_table_slotvar.s`, `get2_table_slothoist.s`, `get_typed.s`, `get_plain.s`.
- The first attack's report, unchanged: `tasks/96/redesign/phase3b-references-attack-first-run.md`.

---

## 12. The final answer list for the group

Each line is the answer as it now stands; "saved" answers replace the originals.

1. **Returning references (A5): the body decides what a result borrows (kept).** Nothing written; the
   inferred set is what today's `@borrows` check accepts, including "a result read through a writable
   view field (`ref mut`, `MutSpan`, at any depth) borrows the receiver too". A public function's set is
   part of its package's interface; a hot reload that widens it is refused; an error caused by a
   callee's body names that line. No run-time cost.
2. **Returning references (A5): a downcast of a place is a place, and a pool table keyed by the type
   (saved).** The cast compares a type id, never a method-table address. Type erasure keeps the limit of
   the views inside. The table hands out typed access to a slot's contents (`self.pools[Pool[T]]`), never
   the slot. Slot numbers come from the compiler; where one is assigned when the program loads (hot
   reload, separately built packages), it is read once per function, not per get. Cost, faithful model:
   the table 1.05 to 1.11 of plain C (bounds checks), the cast 1.11 to 1.16 of the table, safe Rust 1.57
   to 1.64 of plain C; a slot number read per get would add 19% to 22%, read once it adds nothing
   (PROVISIONAL).
3. **Returning references (A5): the return type says `ref`, so the `return` need not (kept).** A call's
   result is never lent this way (a temporary is a place only for calling a `mut self` method).
4. **Structs holding references (A6): a container of views is a view (saved).** Plus: type erasure keeps
   the limit; one limit per view field of the element type, covering every element. No run-time cost.
5. **Structs holding references (A6): an object holding a view carries the view's limit (saved).** One
   limit per object, shared by every handle, never shortened by a store; interface and base-class
   handles, `Weak`, `Shared`, tuples and closures carry it; dropping a handle counts as a use of the
   limit when the object's `drop` may read the view. No run-time cost.
6. **Data that points into itself (A7): a value may hold views of its own heap contents (saved).** Plus:
   a view copied out borrows the field it views; through a class handle, between the write and the
   rebuild, no call that runs program code and no access to that field through another expression;
   through an index, the container is held exclusively from the write until the views are rebuilt;
   summaries record which view fields a `mut` parameter leaves pointing into its own fields; the clone
   re-points a view only when it lies inside the old buffer, ends included. The guarantee asked of the
   containers group: contents never inside the header, and a frame buffer (ADR-141) only for a list that
   never leaves its function. Cost: none over the same Ember program with the text held outside (that
   program is 1.046 of plain C from bounds checks, first attack, PROVISIONAL).
7. **A view of a temporary (A22): a temporary lives as long as the views made from it (kept).** "Has a
   `drop` of its own" reads "has a drop that is not quiet".
8. **A view of a temporary (A22): the owner follows the view (kept).** Only for a local variable of this
   function; a field or a `mut` parameter stays refused, with `String(w)` offered, in the user's names.
9. **A view of a temporary (A22): a view returned as an owned string reuses the temporary's buffer
   (saved).** Reuse only when the result is at least half the buffer's **capacity**; otherwise move the
   viewed part into an exact-size block and free the old one; for an `Array` whose elements need drop,
   drop the elements outside the view. Measured: half of the length would keep 131,072,000 bytes for
   97,584 bytes of names; half of the capacity keeps 97,584, as the copy does.
10. **A view of a temporary (A22): a temporary is a place for its statement (saved, narrowed).** A
    `mut self` method may be called on a temporary that is an iterator, when the call's result borrows
    the temporary, or when the method writes only through the views the temporary holds; otherwise
    `E2140` with a help that explains the lost change.
11. **Moving a field out (A23): a field held exclusively may be emptied, if every path refills it
    (saved).** Held exclusively means: a `mut` parameter, `mut self`, a local `ref mut` (a loop element,
    a `match` binding), or an element reached by index while the list is held for it; refilled before
    the reference's last use, the iteration's end, every exit, and any `defer` that reads it. Relies on
    `[PAN-1]` (no unwinding) and on test panics never being caught in-process. No run-time cost.
12. **Moving a field out (A23): bindings follow their use, move at overwrite (saved).** All live
    references from the loan are re-pointed together, or the overwrite is a loan conflict; a drop flag
    when the overwrite is on some paths only; not across a type with its own `drop`; and assigning to
    the whole name rebinds it, as in Python, so a name the user did not write as a reference never
    writes through by assignment. Cost: within 3.3% of plain C and of Rust's in-place spelling (first
    attack, PROVISIONAL); Rust's `mem::replace` idiom is 1.17 to 1.20 of C.
