# Phase 3b: attacking and saving the "returning and keeping references" answers

Written 2026-10-24 for the owner. This attacks the answers in `phase3b-references.md` (the references
group's report, called "the answers report" below) for five annoyances: returning references (A5),
structs holding references (A6), data that points into itself (A7), a view of a temporary (A22), and
moving a field out from behind `mut self` (A23). For every problem I found I tried to save the
answer first, and attacked the saved version again.

Nothing in the compiler, the standard library, the docs or the tests was changed. My programs are
under `target/redesign2/references/attack/`: Ember, Rust and Python programs in `probes/`, timing
programs in `bench/`.

---

## 0. The short version

| Annoyance | Answer (what it does) | Verdict | What changed |
|---|---|---|---|
| A5 | The body decides what a result borrows (nothing written) | **kept** | No hole found. The inferred borrow set must keep one rule today's `@borrows` already enforces: a result read through a `ref mut` field also borrows `self` (`x05a`, `x05b`). |
| A5 | A downcast of a place is a place, and a pool table keyed by the type | **saved** | Four fixes, all free at run time: compare a type id, never the method-table address; a type-erased box keeps the limit of the views inside it; the table never hands out its slots; slot numbers come from the compiler, not a hash. |
| A5 | The return type says `ref`, so the `return` need not | **kept** | No problem found. |
| A6 | A container of views is a view | **saved** | Converting to an interface type (type erasure) must keep the limit; Rust refuses the same program (`r05c`). Free. |
| A6 | An object holding a view carries the view's limit | **saved** | All handles to one object must share one limit, which never shrinks (Rust's rule for `Rc<RefCell<T<'a>>>`); converting to an interface handle and making a `Weak` handle keep the limit. Free. Without this, three accepted programs read freed memory (`x06a`, `x06b`, `x06c`). |
| A7 | A value may hold views of its own heap contents | **saved** | Three fixes, all free: a view copied out of the value borrows the field it views (`x07a`); a source field written through one object handle stays locked against every other handle until its views are rebuilt (`x07b`); the clone's re-pointing test is made exact. The cost claim is corrected: 1.046 of plain C, not 1.000, from Ember's bounds checks, which the answer does not add. |
| A22 | A temporary lives as long as the views made from it | **kept** | No hole found (it is a rewrite into a program today's checker checks). |
| A22 | The owner follows the view | **kept** | No hole found. Its limits are wider than the answers report says: also a field (`self.best = w`), and two outer variables keeping one temporary. |
| A22 | A view returned as an owned string reuses the temporary's buffer | **saved** | As written it kept 200 MiB alive for 1,001 bytes of words (measured, `keep.c`), and it was not faster than a copy when the result was short. Saved: reuse only when the result fills at least half the buffer. |
| A22 | A temporary is a place for its statement | **saved** | As written it silently loses Python's `reg.get(0).heal(100)` when `get` returns a copy (`x22a`); today's E2140 is the only thing that catches it. Saved, narrowed: allowed on iterators and where the result borrows the temporary. |
| A23 | A field held exclusively may be emptied, if every path refills it | **kept** | No hole found. Every way for other code to see the empty field is already closed by Ember's no-unwinding rule or its run-time access words for class objects. |
| A23 | Bindings follow their use (move at overwrite) | **saved** | Three gaps, closed at no measurable cost: a second name for the same reference (`x23a`); an overwrite on some paths only, which needs the drop flag Ember already has (`x23b`); a type with its own `drop` (`x23c`). |

No answer had to be discarded. Every saved version is still checked at compile time and adds no
run-time check. All timings below are PROVISIONAL (other agents were running); none of my verdicts
rests on a timing difference under 15%.

---

## 1. Words used in this report

- **Place**: something with an address that can be read and written: a variable, a field, an element.
- **View**: a reference (`ref x`, `ref mut x`), a `str` (a piece of a string's characters), a
  `Span` (a piece of a list), or a struct holding one of these.
- **Limit**: the stretch of the program during which a view must stay valid. The spec calls it a
  *region*; the compiler works it out and nothing is written.
- **Loan**: while a view of `x` is in use, `x` is lent: it may not be changed or freed.
- **Summary** of a function: what its result points into, and what it stores where. Callers use it
  instead of reading the body.
- **Heap contents**: the characters of a `String`, the elements of an `Array`, the inside of a `Box`.
  They live in a separate heap block, so they do not move when the `String` header moves.
- **Handle**: a value of a `class` type. It is a counted pointer; copying a handle gives a second
  handle to the same object (spec, Part IV.6).
- **Type erasure**: turning a value into an interface type (`Box[dyn AnyPool]`, a handle typed by an
  interface, a base-class handle). The static type then no longer says what the value really is.
- **Downcast**: getting the real type back from an erased value (`cast b as Pool[Pos]`).
- **Method table**: the table of function pointers an erased value carries (a "vtable" in C++ and Rust).
- **Invariant limit**: a limit that every copy of a handle shares and that can never be made shorter
  by storing something. Rust's `RefCell<Parser<'a>>` has one.
- **Drop flag**: a hidden yes/no variable the compiler keeps when a value is moved away on only some
  paths, so it knows at the end whether to free it. Ember already has them (`[OWN-3]`).
- **Quiet call**: Ember's term (`[EXC-19]`) for a call that runs none of the program's own code.
- **Access word**: the per-field number Ember keeps in a class object to check at run time that no
  one writes a field while someone else uses it (`[EXC-19]`).
- **Median of 5**: the middle of five runs. **Spread**: (slowest minus fastest) over fastest.
- **PROVISIONAL**: measured while other agents ran; a choice resting on a difference under 15% waits
  for the final measurer.
- **UNVERIFIED**: I believe it but did not check it.

---

## 2. How I checked

- **Ember**: `C:/Users/ism19/Code/ember/target/ci/ember.exe check <file>` (it prints `ember 0.1.0`,
  `language: 0.9.10`). No Ember program was run. Every output is in
  `attack/probes/ember_check_attack_output.txt`. Since the answers are not built, today's checker
  usually refuses my attack programs at an earlier line; each attack below says which rule of the
  *proposed* answer would accept it, and what today's checker does.
- **Rust**: `rustc --edition 2021 --crate-type lib --emit=metadata -o /dev/null <file>`, rustc 1.98.1.
  Outputs in `attack/probes/rust/rustc_attack_output.txt`. All five Rust twins are refused.
- **Python**: `python -I attack/probes/py_attack.py` (Python 3.14.2) prints `all Python attack checks
  passed`. It asserts every Python behaviour quoted below.
- **Timings** (`attack/bench/`):
  - Built with `sh build.sh`: C is `"C:/Program Files/LLVM/bin/clang.exe" -O2` (clang 22.1.8) with the
    shared `target/redesign/bench/harness.h` (it calls `SetErrorMode`); Rust is
    `rustc -C opt-level=2 --edition 2021` on the answers report's own Rust files.
  - Run with `powershell -NoProfile -ExecutionPolicy Bypass -File run.ps1`, the same method as the
    answers report: `(Get-CimInstance Win32_Battery).BatteryStatus` was 2 before the run and is checked
    before every round; each process is pinned to the performance cores (affinity `0xC03C03`) right
    after it starts; one program at a time; 5 rounds; the order reversed every other round, so A and B
    alternate.
  - Summarised with `python -I ../../bench/summarize.py out/results.txt` (the answers report's
    script); the output is saved in `out/summary.txt`. Every run of every program in one workload
    printed the same checksum.
  - **All timings are PROVISIONAL.** The spread was 1% to 19% on most rows, 46% on one, and 96% to
    162% on three of the `get` rows (named where used).

---

## 3. A5: returning references

### 3.1 "The body decides what a result borrows" (answers report 3.3): kept

The answer: the compiler works out from a function's body which parameters (or which view fields of
parameters) its result may point into, and callers use that as if `@borrows` had been written.

**Memory attack 1: a result read through a `ref mut` field.** If the inferred set were "the result
points into what `self.buf` views" instead of "into `self`", two calls would give two writable
references to the same element. Ember (`probes/x05a_refmut_field_named.em`, here with today's
`@borrows` written out to show what the inference must produce):

```ember
struct Cursor:
    buf: ref mut Array[int]

    @borrows(self.buf)
    fn cur(mut self) -> ref mut int:
        return ref mut self.buf[0]

fn main():
    xs: Array[int] = [1, 2]
    c = Cursor(buf=ref mut xs)
    a = c.cur()
    b = c.cur()        # a and b would both be writable references to xs[0]
    a += 1
    b += 1
```

Python has no reference to a list slot; the nearest program aliases the list, which is safe there
because Python never frees a list someone still holds:

```python
xs = [1, 2]
a = xs; b = xs
a[0] += 1; b[0] += 1        # xs == [3, 2]
```

Today's checker refuses the annotated version: `E3062: the returned view points into self, and
@borrows names only what self.buf borrows … a view read through a ref mut field borrows self as well`
(`[LT-1a]`). With nothing written (`x05b`), the second call is `E3025`. Rust refuses its twin
(`rust/r05a_cursor.rs`: `lifetime may not live long enough`).

**Result:** no hole. The answers report says the inferred set "is the set it already computes when it
checks a body against a written `@borrows`", and that check is the one that refuses `x05a`. The
saved text only needs to say so explicitly: *a result read through a `ref mut` field borrows the
receiver too.*

**Other attacks, none of which found a hole:**
- **Recursion.** The answer starts from "borrows nothing" and grows the set until it stops changing.
  Each round can only add places, so it ends, and it ends with every place any path returns.
- **A function used as a value.** A call through a function value uses the callable type's own rule
  (`[LT-7]`), never the body; converting a function whose inferred set is wider than that rule allows
  must be refused at the conversion, as an implementation wider than its interface is (`E3062`).
- **Readability.** Because the set is read from the body, an error at a call site can be caused by a
  line in another function. The message must name that line ("`next_word`'s result borrows `p`
  because line 12 returns a view of `self.buf`"). This is a diagnostics requirement, not a fault.

**Cost:** none at run time; the summary exists only in the compiler.

### 3.2 "A downcast of a place is a place, and a pool table keyed by the type" (answers report 3.4): saved

The answer: `cast self.pools[k] as Pool[T]`, on a place holding a `Box[dyn AnyPool]`, gives a
reference into the box after one type check. Better, a standard-library table with one slot per type
(`self.pools[Pool[T]]`) needs no check at all, because its insert only ever puts a `Pool[T]` in the
`Pool[T]` slot. Python:

```python
pool = self.pools[T]
return pool.dense[pool.index[e]]
```

**Memory attack 1: what the type check compares.** The answers report left open whether the check
compares a type id or the address of the method table ("UNVERIFIED"). It must be a type id. Rust's
documentation says comparing method-table addresses is unreliable both ways: "pointers to values of
the same underlying type can compare inequal (because vtables are duplicated in multiple codegen
units), and pointers to values of *different* underlying type can compare equal (since identical
vtables can be deduplicated within a codegen unit)" (https://doc.rust-lang.org/std/ptr/fn.eq.html).
For pools this is concrete: `count()` reads only `dense.len()`, so the method tables of `Pool[Pos]`
and of a `Pool[Big]` with 64-byte elements are byte-for-byte the same, and a linker that merges
identical data would make `cast … as Pool[Pos]` succeed on a `Pool[Big]`.
- **Fix:** compare a per-type id stored in the table (or a pointer to a per-type record that is never
  merged). That is one more load from the same cache line.
- **Cost:** none to re-measure: the answers report's `get.c` cast model already compares an id stored
  in the pool (`cmpl $-1640531527, (%rdx)` in `get_cast.s`), so its measured cost is the fixed
  version's.

**Memory attack 2: an erased box forgets that it holds views.** Ember (`probes/x05c_dyn_box_local_view.em`):

```ember
fn fill(mut keep: Array[Box[dyn AnyPool]]):
    text: String = "short lived"
    keep.push(Box(Pool[str](dense=[text[0..5]])))   # a pool of views of a local

fn main():
    keep: Array[Box[dyn AnyPool]] = []
    fill(keep)
    # a later `cast keep[0] as Pool[str]` would read `text`, already freed
```

Python (works: the slice is a copy):

```python
def fill(keep):
    text = "short lived"
    keep.append(Pool([text[0:5]]))
```

The answers report's A6 rule makes "a `Box` whose element type is a view type" a view type, but
`Box[dyn AnyPool]` does not name its element type, so nothing says the box carries `text`'s limit.
Two values of one view type with different limits have "one type identity" (`[LT-30]`), so neither a
type id nor the table can tell them apart afterwards.
- **Today:** refused for another reason, `E3063` (a box's contents may hold only static views,
  `[TYP-15]`); the answers would lift that rule.
- **Rust** refuses the twin (`rust/r05c_dyn_any_local_view.rs`): `E0597 text does not live long
  enough … due to object lifetime defaults, Box<dyn Any> actually means Box<(dyn Any + 'static)>`.
- **Fix (type erasure keeps the limit):** a value converted to an interface type carries the limit of
  the views inside it, and a downcast gives that limit back. So `keep` above becomes a view of `text`,
  and `fill` is refused at the push, as storing any view of a local through a `mut` parameter is
  today. Free: limits are compile-time only.
- **Attacking the fix:** a generic function that boxes a `T` as `dyn` is checked per instance, as
  `[LT-1]` already does for generics (ODR-048), so `T = str` gets the limit. A downcast of a
  limit-carrying box stored in a long-lived place is refused, because the result inherits the limit.

**Memory attack 3: someone puts the wrong pool in a slot.** The table's read skips the check, so it is
safe only if no code can ever put a `Pool[Big]` in the `Pool[Pos]` slot. A table built on a `Map` of
boxes that hands out its slots does not have that property. This compiles today
(`probes/x05d_swap_erased_slots.em`, exit 0):

```ember
pools = Map[int, Box[dyn AnyPool]]()
_a = pools.insert(1, Box(Pool[Pos](dense=[])))
_b = pools.insert(2, Box(Pool[Big](dense=[])))
_old = mem.replace(pools[1], Box(Pool[Big](dense=[])))   # the "Pos" slot now holds a Pool[Big]
```

Python allows the same, and the mistake shows up later as an `AttributeError`, never as corrupted
memory:

```python
pools[Pos] = PoolOfBig()
```

- **Fix:** the table is a standard-library type whose only writable access to a slot is typed by the
  slot's type: `insert[V](v: V)` puts `v` in `V`'s slot, `self.pools[Pool[T]]` gives a
  `ref mut Pool[T]`, and iteration gives the *contents* as `ref dyn AnyPool` or `ref mut dyn AnyPool`,
  never the box. A `dyn` value has no size (spec IV, "`dyn I` is unsized"), so it cannot be assigned
  or swapped whole. UNVERIFIED: whether today's checker refuses `mem.swap` on two `ref mut dyn`
  places for that reason; my probe (`x05e`) was refused for a different reason (`E3023`, a
  borrowed parameter).
- **Slot numbers.** If slots were found by hashing a 128-bit type id, two types could in principle
  share a slot, and the unchecked read would then be a type confusion. Fix: the compiler gives each
  type used as a key a slot number (one per instance, as the answers report says). If a hash is ever
  used, the *insert* compares the full type identity once; the read stays unchecked.

**Speed attack: does the table's read really cost only bounds checks?** The answers report's model
uses a constant slot number (`pools[3]`). With separate packages or hot reload, the slot number may
only be known when the program loads, so each read loads it and checks it against the table length.
I added that model (`bench/get_attack.c -DTYPED -DSLOTVAR`, made from `get.c` by
`make_get_attack.py`). `work_ms`, median of 5, 32,000,000 gets:

| model | work | ratio to plain C | spread |
|---|---|---|---|
| plain C, no checks (`get.c -DPLAIN`) | 90.020 ms | 1.000 | 19.2% |
| table, slot number known when compiled (`-DTYPED`) | 105.373 ms | 1.171 | 9.1% |
| table, slot number loaded at run time (`-DTYPED -DSLOTVAR`) | 113.448 ms | 1.260 | 95.7% |
| cast of a `Box[dyn]` place, one type-id compare (`-DCAST`) | 109.927 ms | 1.221 | 162.0% |
| safe Rust, `Box<dyn Any>` + `downcast_mut` (`get_rust.rs`) | 162.234 ms | 1.802 | 159.4% |

Source: `attack/bench/out/summary.txt`. The answers report measured the type check at 7.2%; I got
4.3% (109.927 over 105.373); the run-time slot number cost 7.7% over the constant one. All of these
are under 15% and their spreads are large, so they are PROVISIONAL and decide nothing. The table is
chosen for its spelling, not its speed, and its speed is within the measured noise of the cast; Rust's
idiom stays the slowest in both runs.

**Readability:** unchanged, `self.pools[Pool[T]]` as in Python's `self.pools[T]`.

**Verdict: saved.** The saved answer is the original plus: compare a type id, erasure keeps the
limit, the table never hands out a slot, slot numbers come from the compiler.

### 3.3 "The return type says `ref`, so the `return` need not" (answers report 3.5): kept

`fn at(mut self, i: int) -> ref mut int: return self.dense[i]` lends the place, because the declared
result says so. I tried returning a local (`m = self.dense[0]; return m` with `-> ref int`): that is
today's `E3060` (a reference to a local), a clear error, not a silent copy. Returning something that
is not a place (`return self.dense[i] + 0`) is a type error. No problem found. Python's
`return self.dense[i]` reads the same.

---

## 4. A6: structs holding references

### 4.1 "A container of views is a view" (answers report 4.3): saved

The answer: an `Array`, `Map`, `Set`, `Option` or `Box` of a view type is itself a view type and carries
its elements' limits, as Rust's `Vec<View<'a>>`. Python: `vs = [View(reg, 0), View(reg, 1)]`.

**Memory attack 1: a callee stores a short-lived view into the caller's list.**
Ember (`probes/x06d_callee_stores_into_list.em`):

```ember
fn put(mut v: Array[str], s: str):
    v.push(s)

fn fill(mut v: Array[str]):
    word: String = "short lived"
    put(v, word)             # a view of fill's local goes into main's list

fn main():
    vs: Array[str] = []
    fill(vs)
    println(vs[0])           # word is freed by now
```

Python (works, prints `short lived`):

```python
def put(v, s): v.append(s)
def fill(v):
    word = "short lived"
    put(v, word)
```

No hole: `[TYP-15]` already says a function that stores a view it was given into what a `mut`
parameter points to "passes that to its callers", and today's checker applies it across calls:
``E3063: `str` is a view, so it may not be passed to `put`, which stores it …``, at `put(v, word)`.
Under the answer the list `v` carries a limit instead of being unbounded, so the same rule makes
`fill`'s `v` borrow `word`, and `fill` is refused because `v` outlives `word`. Nothing new is needed.

**Memory attack 2: type erasure.** The `Box[dyn AnyPool]` program of 3.2 (`x05c`) is this answer's
hole as much as A5's: a `Box` of views is a view, but a `Box[dyn …]` of views is not, unless erasure
keeps the limit. Same fix, same cost (none).

**Readability, what remains:** Python lets you change `reg` while the list of views exists
(`reg.hp.append(30)`); Ember refuses it while the list is in use, as Rust does. Only counting (the
answers report's counted views, 5.5) removes that.

**Cost:** none (limits are compile-time). Measured layout: Rust's `Vec<&str>` row below, 1.027 of
plain C in my run (1.018 in the answers report's run).

**Verdict: saved** (with "type erasure keeps the limit").

### 4.2 "An object holding a view carries the view's limit" (answers report 4.4): saved

The answer: a class with a field of a view type is a view type; its handles carry the view's limit,
so a handle cannot be stored where a view cannot.

**Memory attack 1: two handles, and a shorter view stored through one of them.**
Ember (`probes/x06a_class_alias_store_shorter.em`):

```ember
struct Lexer:
    src: str

class Parser:
    lexer: Lexer

fn reset(p: Parser, s: str):
    p.lexer = Lexer(src=s)

fn make_and_reset(p: Parser):
    short: String = "short lived"
    reset(p, short)                 # the object now views `short`

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))
    q = p                           # a second handle to the same object
    make_and_reset(p)
    println(q.lexer.src)            # q still "carries text's limit": reads `short`, which is freed
```

Python (works, prints `short lived`; the string stays alive because the object holds it):

```python
p = Parser(Lexer(text)); q = p
make_and_reset(p)
print(q.lexer.src)
```

The answer's rule tracks a limit per handle. `[TYP-15]`'s store rule then narrows the limit of the
handle the store went through (`p`, inside `make_and_reset`), but `q` is a different variable and the
compiler does not know it is the same object. Rust refuses its twin (`rust/r06a_invariance.rs`:
``E0597 `short` does not live long enough``), because `RefCell<Parser<'a>>` fixes `'a` for every
`Rc` copy.
- **Fix (one limit per object, never shortened):** every handle to an object shares the object's one
  limit. A store into the object's view fields, through any handle or any function, needs the stored
  view to outlive that limit, which is worked out from every use of every handle the compiler can see,
  as Rust works out `'a`. Here the limit must include `q`'s use in `main`, `short` does not live that
  long, and `reset(p, short)` is refused.
- **Attacking the fix:** Python's "make it empty, fill it later"
  (`p = Parser(lexer=Lexer(src="")); p.lexer = Lexer(src=text)`) is accepted, because the limit comes
  from all uses, not from the first value. A handle passed into a function that stores a view of its
  own local is refused, because the object's limit includes uses after the call. Free: compile time
  only.

**Memory attack 2: converting to an interface handle forgets the limit.**
Ember (`probes/x06b_upcast_erases_limit.em`):

```ember
interface Named:
    fn describe(self) -> String

class Parser:
    lexer: Lexer

extend Parser implements Named:
    fn describe(self) -> String:
        return self.lexer.src.to_string()

class Registry:
    items: Array[Named]             # a class field: only handles with no limit may go here

fn add_parser(reg: Registry):
    text: String = "let x = 1"
    reg.items.push(Parser(lexer=Lexer(src=text)))    # Parser -> Named: the type no longer says "views text"

fn main():
    reg = Registry(items=[])
    add_parser(reg)
    println(reg.items[0].describe())                 # reads freed `text`
```

Python (works):

```python
def add_parser(reg):
    text = "let x = 1"
    reg.items.append(Parser(Lexer(text)))
```

Rust refuses its twin (`rust/r06b_upcast_dyn.rs`: ``E0597 … `Rc<dyn Named>` actually means
`Rc<(dyn Named + 'static)>` ``). The same hole exists for a base-class handle.
- **Fix:** the same "type erasure keeps the limit" as 3.2: a `Named` made from a limit-carrying
  `Parser` carries that limit, so `reg.items.push(...)` is refused (a class field is unbounded).

**Memory attack 3: a `Weak` handle.** Ember (`probes/x06c_weak_outlives_limit.em`):

```ember
class Parser:
    lexer: Lexer
    me: Option[Parser] = None

class Registry:
    seen: Array[Weak[Parser]]

fn add_parser(reg: Registry):
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))
    p.me = Some(p)                  # a cycle: Ember never frees it ([WK-1])
    reg.seen.push(Weak(p))

fn main():
    reg = Registry(seen=[])
    add_parser(reg)
    match reg.seen[0].upgrade():    # succeeds: the object is alive, its `text` is not
        Some(p):
            println(p.lexer.src)
        None:
            println("gone")
```

Python (works, prints `let x = 1`):

```python
p.me = p
reg.seen.append(weakref.ref(p))
```

The answer lists `Array`, `Map`, `Set`, `Option` and `Box` as carriers, but not `Weak`. Without it,
`Array[Weak[Parser]]` looks unbounded. Because a strong cycle is never freed, `upgrade()` succeeds
after `text` is gone. Rust refuses its twin (`rust/r06c_weak.rs`, `E0597`), because `Weak<Parser<'a>>`
carries `'a`.
- **Fix:** every type that can hold a handle carries its limit: `Weak`, `Shared`, interface and
  base-class handles, tuples, closures (non-`owned` closures already record limits, `[LT-42]`), plus
  the answer's list. Free.

**Today:** all three programs are refused at the first line that stores a view in a class object
(`E3063`); the answer removes exactly that refusal, so it must bring these three rules with it.

**Readability:** a class holding a view now behaves like Rust's `Rc<RefCell<Parser<'a>>>` without the
spelling. The error appears where a limit-carrying handle is stored somewhere long-lived; the message
must name the view field that makes the class carry a limit.

**Verdict: saved** (one limit per object, never shortened; erasure and `Weak` keep the limit).

---

## 5. A7: a value that points into itself

### 5.1 "A value may hold views of its own heap contents" (answers report 5.3): saved

The answer: in `Document(text, words)`, `words` may point into `text`'s characters, because those
characters do not move when the `Document` moves. Writing `text` makes `words` unusable until it is
rebuilt. Python:

```python
class Document:
    def __init__(self, text):
        self.text = text
        self.words = text.split()
```

**Memory attack 1: a view copied out, then the value moved and freed.**
Ember (`probes/x07a_view_out_then_move.em`):

```ember
fn main():
    d = load("the quick brown fox")
    w = d.words[0]          # w points into d.text's characters
    docs: Array[Document] = []
    docs.push(d)            # the answer's attack 7 allows this move: the characters stay where they are
    docs.clear()            # ... and this frees them
    println(w)              # reads freed memory
```

Python (works, prints `the`):

```python
words = "the quick brown fox".split()
w = words[0]; docs = [words]; docs.clear()
print(w)
```

The answer says a self-viewing value has the limit "itself" and may be stored anywhere, and that
moving it is fine. Both are right for the views *inside* it. It does not say what limit a view gets
when it is copied *out*. If `w` got "itself", `docs.push(d)` and `docs.clear()` would be allowed.
- **Fix:** a view read out of a field whose source is the value itself borrows the field it views
  (here `d.text`). Then `docs.push(d)` is moving a value while part of it is lent, which today's rules
  refuse. The same applies to a function's summary: "the result points into what `self.words` views"
  means, at a call on a self-viewing value, "the result borrows that value's `text`".
- **Attacking the fix:** the Python program is refused; the user keeps `w` as `String(w)` or uses it
  before the push, as in Rust. Free.

**Memory attack 2: two handles to one object.** The answer's extra checkpoint for objects is "before
any call that is not quiet, because other handles to the object can only run code during calls". But
the same function can read through a second handle with no call at all.
Ember (`probes/x07b_alias_handles_write_source.em`):

```ember
class Holder:
    doc: Document

fn main():
    h1 = Holder(doc=load("the quick brown fox"))
    h2 = h1                                  # the same object
    h2.doc.text = "zzz"                      # frees the old characters; h2.doc.words becomes unusable
    println(h1.doc.words[0])                 # nothing marks h1.doc.words unusable: reads freed memory
    h2.doc.words = h2.doc.text.split_whitespace().collect()
```

Python (works; prints `the`, the stale but valid old word):

```python
h1 = Holder(Document("the quick brown fox")); h2 = h1
h2.doc.text = "zzz"
print(h1.doc.words[0])
```

Ember's run-time access words do not catch it either: assigning `h2.doc.text` is a write access that
lasts only "the store and the drop of the old value" (spec VIII.3), so it is over before the read.
- **Fix (static, free):** through a class handle, between the write that makes views unusable and
  the line that rebuilds them, (1) no call that is not quiet, as the answer says, and (2) no access to
  that field (`doc`) through any other expression that could reach an object of the same class. So
  `println(h1.doc.words[0])` is refused there. Python's two-line spelling
  (`h2.doc.text = …; h2.doc.words = …`) is still accepted.
- **When the user needs code in between:** write it as a `mut` method of `Document`
  (`h2.doc.set_text("zzz")`). A `mut` method call on a class field already holds the field's write
  access for the whole call (spec VIII.3 table), and the answer's rule 3 makes the struct whole again
  before the method returns. That access check is one Ember already makes for every such call; this
  answer adds none.
- **Attacking the fix:** a read through `h1` inside a quiet call is impossible (a quiet call begins no
  access). A read through a handle stored in a field (`self.other.doc`) is "another expression that
  could reach the class", so it is refused too. Threads cannot reach a non-`@sync` object.

**Memory attack 3: a method re-points a view into the value's own text.**
Ember (`probes/x07c_mut_call_repoints.em`):

```ember
struct Tok:
    buf: String
    cur: str

    fn refill(mut self):
        self.buf = "fresh words"
        self.cur = self.buf[0..5]     # allowed by the answer: a field views a sibling's characters

fn main():
    text: String = "hello"
    t = Tok(buf="", cur=text[0..5])   # here the caller knows t.cur views `text`
    t.refill()                        # after the call, t.cur views t.buf
    t.buf = "x"                       # frees "fresh words"
    println(t.cur)
```

Python (works, prints `fresh`):

```python
t.refill(); t.buf = "x"; print(t.cur)
```

No hole, but only because of machinery the answer must keep: today's checker already records that
`refill` stores a view of `self.buf` into `self.cur` (`[LT-35]`), and refuses `t.buf = "x"` with
``E3021: `t.buf` cannot be written while it is borrowed … mutable borrow of `t.buf` starts here
(t.refill())``. The saved text says: a function's summary records which view fields of a `mut`
parameter may point into which of its own fields on return, and a call through a function value or
`dyn` method is assumed to do so.

**Memory attack 4: the clone's re-pointing.** The answer's derived `clone` re-points each view that
points into the old buffer by "one compare and one add". The test must be exact: re-point a view
`(p, n)` when `base <= p` and `p + n <= base + len`. A plain `p < base + len` test leaves an empty view
at the very end pointing into the old buffer, and a `p <= base + len` test would wrongly move a
non-empty view of a different buffer that happens to start right after the old one. The exact test
costs one more compare. Small, but it is a correctness detail the answer did not state.

**Speed attack: the cost claim.** The answers report says the answer "emits the same code" as plain
C (`words_plain.c`, 1.000). It does not: every Ember index is bounds-checked, and the workload's
random read does two (`words[j]`, `w[0]`). I wrote that model (`bench/words_ember.c`: `words_plain.c`
plus those two checks). `work_ms`, median of 5 (`attack/bench/out/summary.txt`):

| model | work | ratio | peak memory | spread |
|---|---|---|---|---|
| plain C (`words_plain.c`) | 524.168 ms | 1.000 | 99,360 KB | 1.3% |
| Ember with A7: the same plus Ember's two bounds checks (`words_ember.c`) | 548.358 ms | 1.046 | 99,364 KB | 3.3% |
| safe Rust, `Vec<&str>` with the text outside (`words_rust_vecstr.rs`) | 538.244 ms | 1.027 | 99,748 KB | 1.6% |

The 4.6% is Ember's bounds checks, which the same program pays with the text held outside the
`Document`; the answer itself adds nothing. Under 15%: PROVISIONAL. Correct claim: "no cost over the
same Ember program with the text kept outside".

**Readability:** unchanged from the answer: Python's shape. What it still gives up is listed in the
answers report 5.3 ("What it gives up"), plus attack 1's refusal.

**Verdict: saved** (views copied out borrow their field; the handle rule; call summaries say what they
re-point; the exact clone test; the cost claim corrected).

---

## 6. A22: a view of a temporary

### 6.1 "A temporary lives as long as the views made from it" (answers report 6.3): kept

`word = line.to_lower().trim()` becomes the hand-written program with a hidden local, which today's
checker checks. Attacks: a temporary made on one branch only needs a drop flag (the same `[OWN-3]`
flag, and the same `if` a C programmer writes); a temporary in a loop body lives one iteration; a
temporary whose drop runs program code keeps today's error. One precision: "a temporary with a `drop`
of its own" should read "a temporary whose drop is not quiet" (`[EXC-19]`), so an `Array[File]`
temporary counts too. No hole. Python: `word = line.lower().strip()`.

### 6.2 "The owner follows the view" (answers report 6.4): kept

`best = w` inside a loop moves the lowered line into a hidden owner beside `best`. It is a rewrite into
`p22d`, which today's checker accepts, so it cannot accept anything unsafe. Its limits are wider than
the answers report lists:
- `self.best = w` (a field) has no place for a hidden owner, so it is refused (Python accepts it);
- two outer variables keeping one temporary (`if a: best = w` and `if b: other = w`) get one owner, so
  the second keeper is refused when `best` is reassigned.

Both refusals must offer `String(w)`, the copy Python makes, and must name the user's variables, never
a hidden one (the answers report's defect 1 shows `_14` today). Python:

```python
for line in lines:
    w = line.lower().strip()
    if len(w) > len(best): best = w
```

### 6.3 "A view returned as an owned string reuses the temporary's buffer" (answers report 6.5): saved

The answer: `return line.to_lower().trim()` with `-> String` moves the trimmed characters to the front
of the temporary's buffer and returns that buffer. It says: "Not measured: it is strictly less work
than today's spelling." Python:

```python
def first_word(line): return line.lower().split(" ")[0]
```

**Memory attack: the returned string keeps the whole buffer.** The reused buffer keeps its full
capacity. Keep the first word of 200 lines of 1 MiB, as a caller collecting results would
(`firsts.push(first_word(line))`). In Python each kept word is its own small string
(`sys.getsizeof` under 100 bytes, asserted in `py_attack.py`).

**Speed attack: it is not always less work.** A kept short result in a large buffer is more memory to
touch and keep. I measured both cases (`bench/keep.c`; each variant is one `-D`):
- `-DREUSE`: the answer as written;
- `-DCOPY`: today's `.to_string()`, and what Python makes: an exact-size copy, the buffer freed;
- `-DSHRINK`: my first try at saving it: reuse, then `realloc` the buffer down when more than half of
  it and more than 64 bytes are unused;
- `-DHALF`: the saved answer: reuse only when the result fills at least half the buffer, otherwise copy.

Three workloads: first word of 2,000,000 lines of 8 to 80 bytes ("short"); the whole trimmed line of
the same lines ("trim"); first word of 200 lines of 1 MiB ("long"). `work_ms` and peak memory, median
of 5; "kept" is the exact number of bytes the kept strings hold (sum of capacities), printed by the
program:

| workload | variant | work | ratio to copy | peak memory | kept bytes |
|---|---|---|---|---|---|
| short | as written (reuse) | 182.651 ms | 1.051 | 269,532 KB | 87,997,368 |
| short | copy (today, Python) | 173.748 ms | 1.000 | 186,996 KB | 9,977,116 |
| short | realloc down (first try) | 182.032 ms | 1.048 | 247,520 KB | 66,830,530 |
| short | **reuse only if at least half (saved)** | 170.711 ms | 0.983 | 188,004 KB | 10,442,615 |
| trim | as written (reuse) | 173.820 ms | 0.820 | 269,532 KB | 87,997,368 |
| trim | copy | 211.978 ms (spread 46.3%) | 1.000 | 266,072 KB | 84,661,626 |
| trim | **reuse only if at least half (saved)** | 179.896 ms | 0.849 | 269,528 KB | 87,997,368 |
| long | as written (reuse) | 451.826 ms | 1.002 | 210,080 KB | 209,715,200 |
| long | copy | 451.105 ms | 1.000 | 5,504 KB | 1,001 |
| long | **reuse only if at least half (saved)** | 455.551 ms | 1.010 | 5,504 KB | 1,001 |

Source: `attack/bench/out/summary.txt` and `out/results.txt`.

What it shows:
- **As written, the answer fails on memory**, and that is not a timing: it kept 209,715,200 bytes
  (200 MiB) alive for 1,001 bytes of words, a peak of 210,080 KB against 5,504 KB for the copy. With
  short lines it kept 8.8 times the bytes (87,997,368 against 9,977,116). Python keeps the small amount.
- **"Strictly less work" is not true** for a short result: as written it took 1.051 of the copy's
  time on the short lines. That difference is under 15% (PROVISIONAL), but it is not "less".
- **The first fix (`realloc` down) did not save it**: with lines under 64 spare bytes it rarely fires
  (66,830,530 bytes kept), and it was no faster. Discarded for the next one; nothing in it is worth
  keeping beyond the measurement.
- **Reuse only when the result fills at least half the buffer** keeps a result to at most twice its
  length, so memory matches the copy (5,504 KB long, 188,004 KB short) and stays as fast as reuse
  where reuse helps (trim: 0.849 of the copy, against 0.820 for reuse as written). The trim copy row
  has a 46.3% spread, so these trim ratios are PROVISIONAL; the memory results are not.

**Saved rule:** when the declared result is an owned `String` or `Array` and the returned view points
into a temporary or local that ends here, reuse that buffer if the view is at least half its length;
otherwise make the exact-size copy and free the buffer. The cost is one compare. Attacking the saved
rule: the worst case is a result of just over half the buffer, which holds at most twice its length,
the same bound as an `Array` after doubling growth.

**Verdict: saved.**

### 6.4 "A temporary is a place for its statement" (answers report 6.6): saved, narrowed

The answer: a `mut self` method may be called on a value returned by a call, as Rust allows, so
`line.split(" ").next()` compiles. Assigning a field of a temporary stays `E2140`; a lint catches a
mutating call whose result is thrown away.

**Readability attack: a lost change that Python makes.** Ember (`probes/x22a_mut_call_on_copy.em`):

```ember
@derive(Copy)
struct Hero:
    hp: int

    fn heal(mut self, n: int) -> bool:
        self.hp += n
        return self.hp > 50

struct Registry:
    heroes: Array[Hero]

    fn get(self, i: int) -> Hero:       # returns a copy
        return self.heroes[i]

fn main():
    reg = Registry(heroes=[Hero(hp=10)])
    if reg.get(0).heal(100):            # heals the copy; its result is used, so the lint is silent
        println("healed")
    println(reg.heroes[0].hp)           # 10
```

Python (prints 110; asserted in `py_attack.py`):

```python
if reg.get(0).heal(100): print("healed")
print(reg.heroes[0].hp)
```

Today this is ``E2140: a `mut self` method needs a variable to write through``, and that error is the
only thing standing between a Python habit and a silently lost update. The answer removes it.
- **Fix (narrowed):** a `mut self` method may be called on a temporary when (1) the temporary's type
  is an iterator (the case Python writes as `next(iter(...))`, and `p22e`'s `split(" ").next()`), or
  (2) the call's result borrows the temporary (by the function's summary), so the change can be seen
  through the result (a builder's `with_x(mut self) -> ref mut Self`). Otherwise `E2140` stays, with a
  help line: "`get` returns a copy, so this change would be lost; store it first, or make `get` return
  `ref mut Hero`".
- **Attacking the fix:** changing a copied iterator is never visible to anyone, so (1) cannot lose a
  change the user can see. (2) only allows calls whose effect stays reachable. Free.

**Verdict: saved** (narrowed to iterators and to results that borrow the temporary).

---

## 7. A23: moving a field out from behind `mut self`

### 7.1 "A field held exclusively may be emptied, if every path refills it" (answers report 7.3): kept

The answer: inside a function holding `self` (or a `mut` parameter) exclusively, a field reached by
field names only may be moved out, if every path puts a value back before anything else can see it.

I looked for every way other code could see the empty field:
- **A panic in between.** Ember never unwinds: "There is no unwinding in this version; `defer` blocks
  and destructors do not run on panic" (`[PAN-1]`). `@should_panic` tests are "each isolated"
  (`[TST-3]`). UNVERIFIED: `@should_panic` and `std.testing.expect_panic` are not implemented yet
  (no match in `compiler/src` or `std/`); they must run the test in its own process, never catch the
  panic in-process, or this rule breaks.
- **The struct sits in a class object and the method calls code that reaches it through another
  handle.** A `mut self` call on a class field holds that field's write access for the whole call
  whenever the call runs program code (spec VIII.3 table, `[EXC-19]`), so the other handle's access
  panics before it can read the empty field. Ember already pays that check today; this answer adds none.
- **Globals.** "A `static` is immutable; mutation goes through a `Sync` interior type (`Atomic`,
  `Mutex`, `RwLock`, `Once`)", and `static mut` needs `unsafe` (`[STA-1]`). A `Mutex` guard is
  exclusive, so code that tries to reach the field through the global waits or fails; it never reads it.
- **`async`.** Ember has no `async` (a reserved word, Part XI); `yield` is a checkpoint in the answer.
- **A `drop` method.** `fn drop(mut self)` is a function holding `self`; the answer's rule 3 makes it
  refill before returning. (Allowing `drop` to leave a field empty and skip dropping it would be a
  free extra; not needed.)

No hole found. Bonus for readability: Python's swap `self.a, self.b = self.b, self.a` becomes legal
with no `mem.swap`, since both fields are refilled before anything else runs.

### 7.2 "Bindings follow their use; move at overwrite" (answers report 7.4): saved

The answer: in `old = p.name; p.name = new_name; log(old)`, `old` refers into `p.name`, and just before
the overwrite the old value moves into `old`, so `old` keeps the old name, as in Python.

**Memory and readability attack 1: two names for the same reference.**
Ember (`probes/x23a_alias_then_overwrite.em`):

```ember
fn rename(mut p: Person, owned new_name: String):
    old = p.name
    alias = old                     # a second reference into p.name
    p.name = new_name               # "its value moves out": into old; alias still points at p.name
    p.log.push(f"renamed {alias}")  # would say "renamed bob"
```

Python (asserted: `renamed ann`):

```python
old = p.name; alias = old; p.name = "bob"
p.log.append(f"renamed {alias}")
```

The rule says "its value moves out" for *a* name. With two names, the value can go to one only; the
other keeps pointing at `p.name` and silently sees the new value. If the reference had been stored
inside another value (a struct, a closure), it could not be re-pointed at all.
- **Fix:** at the overwrite, every live reference made from that loan must be a plain local of this
  function; the compiler moves the old value into one hidden owner and points all of them at it.
  A reference held inside another value at that point makes the overwrite a borrow conflict, as today
  (refused, never silent). Free: re-pointing a local is a register move.
- **Also:** a name that refers into an *inline* part of the overwritten value (`x = p.pos.x` with `pos`
  a plain struct) is re-pointed to the same offset in the hidden owner; one into heap contents
  (`first = p.items[0]`) needs nothing, because the moved array's elements do not move.

**Speed attack: "no copy and no placeholder" is true, "nothing" is not always.**
Ember (`probes/x23b_overwrite_on_one_path.em`):

```ember
fn maybe_rename(mut p: Person, owned new_name: String, really: bool):
    old = p.name
    if really:
        p.name = new_name     # old's value moves out on this path only
    println(old)              # old owns the old name, or still refers into p.name
```

Python: `old = p.name; if really: p.name = new; print(old)` prints the old name either way.

Which one `old` is depends on `really`, so at the end the compiler must know at run time whether to
free the hidden owner: a drop flag, the same as `[OWN-3]` already uses for a move made on some paths
only. In a loop that overwrites `p.name` every iteration, the first overwrite moves and later ones drop
normally; the same flag tells them apart. Cost: one byte and a test at the overwrite and at the end,
only in these shapes, exactly what Rust's drop flags cost. Not measured: it is Ember's existing
mechanism, not a new one.

**Readability attack 2: a type with its own `drop`.** Ember (`probes/x23c_drop_type_overwrite.em`):

```ember
struct Guard:
    name: String

    fn drop(mut self):
        println(f"released {self.name}")

fn swap_guard(mut g: Guard, owned other: Guard):
    old = g.name
    g = other           # Python releases the old guard here
    println(old)
```

Python (asserted): prints `released ann`, then `ann`.

To keep `g.name` alive for `old`, the compiler would either move a field out of a type with a `drop`
(today's `E3010`, because `drop` needs the whole value), or keep the whole old guard alive until `old`
is last used, which moves the `released` line later than Python's. A lock released later than it looks
is the change of behaviour the answers report itself refuses for lock-guard temporaries (its 6.3).
- **Fix:** move at overwrite does not apply when a type with its own `drop` lies between the
  overwritten place and the part the name refers to; it is a borrow conflict with the help
  `old = g.name.clone()`. Free.

**Speed attack: re-measuring the claim.** The answers report models the answer with `state.c` /
`state_checked.c`, which read the path out and write it back on every walking tick. Looking at
`state_checked.s`, clang already removed most of that write-back: on a tick with no change it still
stores the tag and the new length. What the saved answer emits is closer to `bench/state_inplace.c`:
`path` is a `ref mut` binding, `pop` works in place, and a transition changes only the tag, because
`Walking` and `Running` keep the path at the same offset (`state_inplace.s`: tag stores only on
transitions). Total ms, median of 5, 1,000,000 NPCs over 100 ticks (`attack/bench/out/summary.txt`):

| model | time | ratio | spread |
|---|---|---|---|
| plain C (`state.c`) | 475.109 ms | 1.000 | 16.0% |
| answers report's Ember model (`state_checked.c`) | 490.957 ms | 1.033 | 15.8% |
| saved answer, in place (`state_inplace.c`, with Ember's `pop` check) | 482.251 ms | 1.015 | 13.9% |
| safe Rust, `mem::replace` idiom (`state_rust.rs`) | 572.097 ms | 1.204 | 5.9% |
| safe Rust, in place with `mem::take` (`state_rust_inplace.rs`) | 472.949 ms | 0.995 | 11.4% |

Peak memory was 113,684 to 114,028 KB in every row. The Ember rows are within 3.3% of C and of Rust's
in-place spelling: PROVISIONAL, and no decision rests on it. Rust's `mem::replace` idiom is 1.204 here
(1.173 in the answers report's run), the only difference over 15%.

**Verdict: saved** (all live references re-pointed or refused; the drop flag stated; types with
`drop` excluded).

---

## 8. Answers not chosen by the group

I did not re-attack the answers the group kept but did not choose (accessors that `yield`, counted
views) or the ones it discarded. The counted-views figures (1.039 of the time, 1.000 of the memory)
are the answers report's own measurement; I did not re-run them.

---

## 9. Defects found in passing (today's compiler; nothing was changed)

1. **An error inside the standard library for a user's typo.** `from std import mem`
   (`probes/d01_from_std_import.em`) reports `E1061: module 'std' has no item named mem` and then
   ``E0100: expected a declaration --> std\src\lib.em:31:1 … found `pass` ``: the checker parses the
   standard library's `lib.em` and blames its line 31. The user did nothing wrong in that file.
2. **A help for the wrong shape.** `mem.swap(pools[1], pools[2])` on one `Map`
   (`probes/d02_swap_two_elements.em`) is refused correctly (two writable borrows of `pools`), but the
   message says ``cannot call a method on `pools` `` and its help and note talk about methods
   borrowing the fields of `self` (`[BRW-10]`), with "inline the field access … or split the method".
   No method of `self` is involved; the useful help is a swap by index on the collection.

---

## 10. Files

All under `target/redesign2/references/attack/`.

- `probes/x*.em`: the attack programs; `probes/d*.em`: the defect checks. Outputs of
  `ember check` for all of them: `probes/ember_check_attack_output.txt` (the first `x05d` entry used a
  wrong import; the corrected run is the later entry).
- `probes/rust/r*.rs`: the Rust twins; outputs in `probes/rust/rustc_attack_output.txt`.
- `probes/py_attack.py`: the Python behaviour of every attack, asserted.
- `bench/words_ember.c`, `bench/state_inplace.c`, `bench/keep.c`, `bench/make_get_attack.py` (writes
  `get_attack.c` from the answers report's `get.c`), `bench/build.sh`, `bench/run.ps1`.
- `bench/out/results.txt` (every run), `bench/out/summary.txt` (the medians), and the assembly read
  above: `state_inplace.s`, `state_checked.s`, `get_typed_slotvar.s`.

---

## 11. The final answer list for the group

Each line is the answer as it now stands; "saved" answers replace the originals.

1. **A5, the body decides what a result borrows (kept).** Nothing written; the inferred set is exactly
   what today's `@borrows` check accepts, including "a result read through a `ref mut` field borrows
   the receiver too". A public function's set is part of its package's interface; a hot reload that
   widens it is refused; an error caused by a callee's body names that line.
2. **A5, a downcast of a place is a place, and a pool table keyed by the type (saved).** The check
   compares a type id, never a method-table address. Type erasure keeps the limit of the views inside.
   The table hands out only typed access to a slot (`self.pools[Pool[T]]`) and the contents, never the
   slot itself. Slot numbers are given by the compiler; a hashed id would be checked at insert, never
   at read. Cost: the table's read is bounds checks only (1.171 of plain C, provisional); a run-time
   slot number added 7.7% (provisional, noisy).
3. **A5, the return type says `ref`, so the `return` need not (kept).**
4. **A6, a container of views is a view (saved).** As written, plus: type erasure keeps the limit; a
   callee storing into a container reuses `[TYP-15]`'s existing "passes that to its callers" rule.
5. **A6, an object holding a view carries the view's limit (saved).** One limit per object, shared by
   every handle and never shortened by a store; worked out from every use, so "make it empty, fill it
   later" still works. Interface and base-class handles, `Weak`, `Shared`, tuples and closures carry the
   limit too. No run-time cost.
6. **A7, a value may hold views of its own heap contents (saved).** As written, plus: a view copied
   out borrows the field it views; through a class handle, between the write and the rebuild there is
   no non-quiet call and no access to that field through any other expression (or the change is made
   in a `mut` method of the struct); summaries record which view fields a `mut` parameter may leave
   pointing into its own fields (today's `[LT-35]` already does); the clone re-points a view only when
   it lies inside the old buffer, ends included. Cost: none over the same program with the text kept
   outside (that program is 1.046 of plain C from bounds checks, provisional).
7. **A22, a temporary lives as long as the views made from it (kept).** "Has a `drop` of its own" reads
   "has a drop that is not quiet".
8. **A22, the owner follows the view (kept).** Fields and two keepers of one temporary are refused with
   `String(w)` offered, in the user's names.
9. **A22, a view returned as an owned string reuses the temporary's buffer (saved).** Reuse only when the
   result fills at least half the buffer; otherwise the exact copy. Measured: memory equal to the copy
   (5,504 KB against 210,080 KB as written, in the long case), time 0.983 to 0.849 of the copy
   (provisional).
10. **A22, a temporary is a place for its statement (saved, narrowed).** Allowed when the temporary is
    an iterator or the call's result borrows it; otherwise `E2140` with a help that explains the lost
    change.
11. **A23, a field held exclusively may be emptied, if every path refills it (kept).** Relies on
    `[PAN-1]` (no unwinding) and on test panics never being caught in-process.
12. **A23, bindings follow their use, move at overwrite (saved).** All live references from the loan
    must be locals, re-pointed together, or the overwrite is a borrow conflict; a drop flag when the
    overwrite is on some paths only; not across a type with its own `drop`. Cost: within 3.3% of plain
    C and of Rust's in-place spelling (provisional); Rust's `mem::replace` idiom is 1.204 of C.
