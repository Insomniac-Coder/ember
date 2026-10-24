# Phase 3b: returning and keeping references (A5, A6, A7, A22, A23)

Written 2026-10-24 by the references group agent, for the owner. Nothing in the language, the
compiler, the standard library, the docs or the tests was changed. My programs are under
`target/redesign2/references/`: the Ember and Rust probes in `probes/`, the timing programs in
`bench/`.

Where the evidence comes from:

- Every Ember result is real output of `target/ci/ember.exe check <file>` (it prints `ember 0.1.0`,
  `language: 0.9.10`). All outputs are saved in `probes/ember_check_output.txt`. No Ember program was run.
- Every Rust result is real output of `rustc 1.98.1 --edition 2021 --crate-type lib`, saved in
  `probes/rust/rustc_output.txt`.
- Every Python behaviour was run with Python 3.14.2: `python -I probes/py_checks.py` prints
  `all Python checks passed`.
- Every timing is mine, from `bench/out/results.txt`. Section 2 gives the method. **All timings are
  PROVISIONAL**: other agents were running at the same time, so a choice that rests on a difference
  under 15% waits for the final measurer. I say so where it applies.

---

## 0. The short version

| Annoyance | Best answer, in plain words | Kind (the job's three) | Cost, measured or argued | Meets all three goals? |
|---|---|---|---|---|
| **A5** returning references | The compiler reads what a function's result points into from the function's body, so `@borrows` is never written. A downcast of a place gives a place. A pool table keyed by the type gives `self.pools[Pool[T]]` with no cast at all. | (a) a Pythonic spelling, plus (b) a smarter compile-time solution for the table | None at run time; it is all compile time. The type check that the table removes measured 7.2% (inlined) and 7.8% (out of line); that is under 15%, so provisional. Rust's own downcast was 1.71x and 1.61x plain C. | **Yes.** What is left: an interface method has no body to read, so it keeps today's rule. |
| **A6** structs holding references | A list (or map, set, box) of views is itself a view, which is Rust's `Vec<View<'a>>` with nothing written. An object that holds a view carries the view's limit. To outlive the source, move the source in (that is A7's answer). | (a) | None. The same layout as Rust's `Vec<&str>`, which measured 1.018x plain C. | **Yes, for every use whose source outlives it.** It lacks one thing: keeping a view longer than its source, with no way to move the source into the keeper, is refused, as it is in Rust. Python allows it by counting. |
| **A7** self-referential data | A value may hold views of the *heap contents* of its own fields. Writing to such a field makes those views unusable until they are set again. | (b) | None: plain C's layout and loop (1.000). Rust's workaround measured 1.205x. Today's Ember workaround (an owned string per word) measured 1.451x the time and 2.122x the memory. | **Yes.** What is left: to edit the text in place you rebuild the words, and a struct's inline fixed arrays cannot be borrowed by its other fields. |
| **A22** a view of a temporary | A temporary lives as long as the views made from it. A view kept in an outer variable takes its temporary with it. A view returned as an owned string reuses the temporary's buffer. A `mut self` method may be called on a temporary. | (a), with two (b) parts | None. It is the code a programmer writes by hand today, or less (no extra allocation). | **Yes.** What is left: views of temporaries pushed into a list are refused, and the help offers the copy Python makes. Temporaries that have their own `drop` (lock guards) keep today's error. |
| **A23** moving a field out from behind `mut self` | A field the function holds exclusively may be emptied, as long as every path fills it again before it is used, before the function ends, and before anything else could see it. A name bound into the field keeps its value when the field is overwritten. | (b), with an (a) part | None: measured 1.000 against plain C. Rust's documented `mem::replace` idiom measured 1.173x, because it rewrites every state. Rust's in-place spelling measured 0.984x. | **Yes.** It relies on Ember never unwinding after a panic (`[PAN-1]`). |

Every answer above is checked at compile time, and none adds a run-time check. Three of them are
rewrites into programs today's checker already accepts (A5's inferred borrows, and A22's first two).
Section 8 shows how the five fit together, and section 9 says what this group needs from the others.

---

## 1. Words used in this report

- **Place**: something with an address that can be read and written: a variable, a field of a place,
  an element of a list, what a reference points to.
- **Reference**: `ref x` (read only) or `ref mut x` (read and write). It is a pointer the compiler
  checks.
- **View**: a reference, or a value that contains one: `str` (a piece of a string's characters),
  `Span[T]` (a piece of a list), or a struct holding one of these (a *view type*, `[TYP-34]`).
- **Limit** (the spec says *region*): the stretch of the program during which a view must stay valid.
  The compiler works it out; nothing is written in the source.
- **Loan**: while a view of `x` is in use, `x` is *lent*, so `x` may not be changed or freed.
- **Summary** (of a function): what the function's result points into, for example "the result
  points into the text that `self.src` views". Callers use the summary instead of reading the body.
- **Heap contents**: the characters of a `String`, the elements of an `Array`, the contents of a
  `Box`. They sit in a separate heap buffer, so they do **not** move when the `String`, `Array` or
  `Box` header moves (spec, Part IV type table: a collection is "inline header + heap buffer").
- **Temporary**: a value made in the middle of an expression and given no name, like the lowered
  string in `line.to_lower().trim()`. Today it is freed at the end of its statement (`[EXP-4]`).
- **Empty field**: a field whose value has been moved away and not yet replaced.
- **Unusable**: a field that may not be read until it is assigned again. An empty field is unusable;
  so, in my A7 answer, is a view whose source was written.
- **Quiet call**: Ember's term (`[EXC-19]`) for a call that runs none of the program's own code: a
  built-in that calls nothing back and drops nothing whose `drop` runs code.
- **Type id**: a number the compiler gives every type, used to check a downcast.
- **Inlined / out of line**: whether the compiler copies a function's body into its caller, or
  calls it.
- **Median of 5**: the middle value of five runs.
- **Peak memory**: the largest working set the process reached, as the harness reports it.

The job's three kinds of answer:
- **Kind (a)** is a Pythonic spelling of the same compile-time guarantee Rust gives.
- **Kind (b)** is a smarter compile-time solution, so Rust's rule is not needed.
- **Kind (c)** is a new mechanism.

---

## 2. How the costs were measured

- **Machine:** the owner's laptop (Intel Core Ultra 9 275HX), on mains power.
  `(Get-CimInstance Win32_Battery).BatteryStatus` returned 2 before the run, and the run script
  checks it again before each round.
- **Compilers:** `clang version 22.1.8`, and `rustc 1.98.1 (48a229cea 2026-09-01)`, host
  `x86_64-pc-windows-msvc`.
- **Build commands**, run in `target/redesign2/references/bench/`:
  - C: `"C:/Program Files/LLVM/bin/clang.exe" -O2 -I target/redesign/bench <file>.c -o out/<name>.exe`.
    `get.c` was built three times, with `-DPLAIN`, `-DTYPED` and `-DCAST`, each with and without
    `-DNOINLINE`.
  - Rust: `rustc -C opt-level=2 --edition 2021 <file>.rs -o out/<name>.exe`, and
    `get_rust.rs` once more with `--cfg noinline`.
- **Harness:** the C programs use the shared `target/redesign/bench/harness.h`. It calls
  `SetErrorMode`, so no dialog can open, and it prints the time, the peak working set and a checksum.
  The Rust programs use `bench/harness.rs`, my twin of it: the same random sequence and the same
  output line.
  - Each program also prints `work_ms`: the timed work only, without building the input data.
  - The `state` programs print only the total time.
- **Runner:** `powershell -NoProfile -ExecutionPolicy Bypass -File bench/run.ps1`.
  - 5 rounds. In each round every program runs once, one at a time.
  - The order is reversed on every other round, so that A and B alternate.
  - Each process is pinned to the performance cores (affinity mask `0xC03C03`) right after it starts,
    as the shared phase 3 runner does. So the first milliseconds of each run are unpinned.
- **Summary:** `python -I bench/summarize.py bench/out/results.txt` prints the medians, the spread
  (max minus min, over min) and whether the checksums agree.
- **Checksums:** every run of every program in a workload printed the same checksum. The words
  workload printed 11949232456339631932, the get workload 16779647745309, the state workload
  8398665211.
- **A discarded run:** `bench/out/results_first_run.txt` is an earlier run. Its `get` rows are not
  valid. In the assembly I saw that clang had proved the type id constant and removed the compare I
  was trying to measure. I made the pool table opaque (a non-static table and a compiler barrier)
  before the run used here. The words and state rows of the two runs agree within a few percent.

---

## 3. A5: returning references

### 3.1 What Rust's rule protects against

A returned reference must not outlive what it points into. The caller must also know which
argument it points into, so that it does not grow or free that argument while the reference is in
use. Without the rule, this is the C failure:

```c
Pos *get_pos(Registry *r, uint32_t e) { Pool *p = r->pools[T_POS]; return &p->dense[p->sparse[e]]; }

Pos *q = get_pos(reg, e);
emplace(reg, other, (Pos){0, 0});   /* dense may be reallocated: q now points into freed memory */
q->x += 1.0f;                       /* writes into freed memory: no error, a corrupted heap */
```

Rust refuses that misuse (phase 1: `a05_ecs_misuse.rs`, E0499). It also wants the link between
argument and result written out wherever it cannot guess it. My twin `probes/rust/r05f_conditional_place.rs`:

```text
error[E0106]: missing lifetime specifier
2 | pub fn smaller(a: &mut i32, b: &mut i32) -> &mut i32 {
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter: pub fn smaller<'a>(a: &'a mut i32, b: &'a mut i32) -> &'a mut i32
```

### 3.2 The Python program, and what Ember does with it today

Python, the ECS `get` of ODR-286 (run in `py_checks.py`):

```python
class Registry:
    def get(self, T, e):
        pool = self.pools[T]
        return pool.dense[pool.index[e]]

reg.get(Pos, 7).x += 1.0
```

Ember today. With the pools as class objects (`target/analysis/ecs5/pos.em`) it is refused:

```text
error[E3060]: `pool.dense[…]` does not live long enough
20 |         pool = cast self.pools[7u128] as Pool[T]
   |         ---------------------------------------- `pool` is a local, so its storage ends with the frame
21 |         return ref mut pool.dense[i]
  = help: return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns
```

With the pools in `Box[dyn AnyPool]`, the owner's verdict on ODR-286 (`probes/p05c_box_dyn_cast_place.em`), it is refused too:

```text
error[E2020]: `cast … as` and `cast … if` need related class handles, found `Box[dyn AnyPool]` and `Pool[T]`
18 |         pool = cast self.pools[7u128] as Pool[T]
```

With `Box[Pool[int]]`, a single concrete type with no downcast, it compiles (`target/analysis/ecs5/box1.em`). Changing the
pools while the result is in use is refused at compile time (`box_bad.em`, E3025), which is Rust's
guarantee. Two smaller annoyances remain:

- A method's result is tied to its whole receiver unless `@borrows(self.src)` is written.
  `e06b_two_words.em` keeps two words from one parser and gets `E3025: cannot call a method on p
  while p.lexer is mutably borrowed`. `e06c` adds `@borrows(self.src)` and compiles.
- `probes/p05f_conditional_place.em` returns `ref mut a` or `ref mut b`:

  ```text
  error[E3062]: the returned view points into `a`, which the result does not borrow
    = help: write `@borrows(a)` above the declaration, or return the value instead of a view of it
  ```

  The same function with `@borrows(a, b)` compiles (`probes/p05g_conditional_place_borrows.em`).
- `probes/p05a_return_place_no_ref.em` declares `-> ref mut int` and returns `self.dense[i]`:

  ```text
  error[E2020]: expected `ref mut i64`, found `i64`
    = help: borrow it explicitly: `ref mut` before the place
  ```

Already accepted today: `longer(a, b)` with no annotation (`e05a`), and NLL problem case #3
(`e05b`), which Rust still refuses.

### 3.3 Answer: the body decides what a result borrows (kind (a))

Python:

```python
class Parser:
    def __init__(self, src): self.src, self.pos = src, 0
    def next_word(self):
        rest = self.src[self.pos:]
        end = rest.find(" ") if " " in rest else len(rest)
        self.pos += end + 1
        return rest[:end]

p = Parser("let x = 1"); a = p.next_word(); b = p.next_word()
```

Ember. This is `e06b` exactly as written. Today it is E3025; proposed, it compiles:

```ember
struct Parser:
    src: str
    pos: int = 0

    fn next_word(mut self) -> str:          # the summary says: the result points into what self.src views
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()                       # fine: a borrows text, not p
    println(a, b)
```

And `smaller` with no annotation:

```ember
fn smaller(mut a: int, mut b: int) -> ref mut int:     # summary: the result points into a or b
    if a <= b:
        return ref mut a
    return ref mut b
```

**The rule.** For every function that returns a reference or a view, the compiler computes from the
body which places the result may point into: whole parameters, or the view fields of parameters.
This is the set it already computes when it checks a body against a written `@borrows`.
- The set is recorded as the function's summary, and callers use it as if `@borrows` had been written.
- `[LT-22]` already does this for functions that return view types; the rule extends it to every function.
- A recursive function starts from "borrows nothing" and grows the set until it stops changing.
- A function with no body, such as an interface method, keeps today's elision rule (`[LT-1]`). An
  implementation that borrows more than its interface is `E3062`.

**Why it is safe.** The summary *is* a `@borrows` annotation, and the body is checked against it with
today's rules. `@borrows` is already sound: `p05g` and `e06c` pass, and returning anything else is
E3062. So inference cannot accept a program that the hand-written annotation would refuse.
Attacks I tried:
1. **A public function's body changes later**, so that its result borrows more. Callers in other
   packages were compiled against the old summary. The summary is therefore part of the package's
   interface, and a build that widens a public function's summary reports it as an interface change,
   as changing a parameter type would be. This is the same question the borrowing group faces for
   `[BRW-10]` (a public method borrows all of `self`), and it should get one answer for both.
2. **Hot reload of a body that widens its summary.** The reload is refused. Last night's
   'today's rules, but never an error' design had exactly this hole for its inferred effects
   (`phase3-break-D3.md` 1.6).
3. **Calls through a trait object, a closure or a function value.** The caller cannot see the body,
   so it uses the declared signature (`[LT-7]`), never the body.
4. **Generic functions.** Each instance gets its own summary, as `[LT-22]` already does.

**Cost.** None at run time. The summary exists only inside the compiler, and the emitted C is the
same as with the annotation written.
**Readability.** Python-like: nothing is written.
**What it gives up.** A public function's signature no longer shows what its result borrows;
`ember inspect` can print it.

### 3.4 Answer: a downcast of a place is a place, and a pool table keyed by the type

These answers are kind (a) for the cast and kind (b) for the table.

The rule for the cast comes from Hylo's spec ("The result of a downcast expression `e` is a projection
of the left operand", quoted in `phase2-hylo-csharp.md` 1.1):

- `cast <place> as T`, where the place holds a `Box[dyn I]`, gives a **reference** to the `T`
  inside the box.
- The reference is writable if the place is.
- There is one type check, and a wrong type panics, as `cast` does today.

Ember, with the owner's `Box` pools:

```ember
fn get_mut[T](mut self, e: Entity) -> ref mut T:
    pool = cast self.pools[type_id[T]()] as Pool[T]    # a reference into the box: one type check
    return pool.dense[pool.index_of(e)]                # (with 3.5, no `ref mut` needed here)
```

Better: a standard-library **pool table keyed by the type**. Its insert puts a value in the slot of
the value's own type, so the slot for `Pool[Pos]` can only ever hold a `Pool[Pos]`, and reading it
needs no type check. Python keys its dict by the class, and the spelling is the same:

```python
pool = self.pools[T]
```

```ember
struct Registry:
    pools: TypeTable[AnyPool]           # one slot per type; a slot holds only a value of its own type

    fn get_mut[T](mut self, e: Entity) -> ref mut T:
        pool = self.pools[Pool[T]]      # ref mut Pool[T]: no cast, no compare
        return pool.dense[pool.index_of(e)]
```

How the table is implemented: the one unchecked downcast lives inside the standard library, and the
insert rule makes it correct. The user writes no cast. The exact spelling of a type used as a key is
a proposal; the owner decides it.

**Why it is safe.**
- The result is an ordinary loan of `self.pools`. While `pool` or the returned element is in use,
  `self.pools` cannot change (`box_bad.em`: E3025). That is Rust's guarantee.
- The type check, or the table's insert rule, guarantees the type.

Attacks I tried:
1. **A second handle to the same pool grows `dense`.** This was ODR-286's counterexample for class
   pools. It cannot happen: a `Box` has one owner.
2. **Inserting a new pool while the element is in use.** Refused. A map can rehash, though the
   boxes' contents would not move; the borrowing group's path-based rule (`phase2-vale-austral-pony.md`
   5.1) could allow it later.
3. **Two `get_mut` results for two types at once.** Both borrow `self.pools` for writing, so the
   second is refused, as in Rust (E0499). That is A3, the borrowing group's question.
4. **Two instances of a generic with the same type id.** Ids are assigned by the compiler, one per
   instance.

**Cost.** Measured with `bench/get.c` (Ember models and plain C) and `bench/get_rust.rs`: 32,000,000
`get(e).x += 1.0` on random entities, out of 1,048,576 scattered over the dense array. `work_ms`,
median of 5:

| model | inlined | out of line | peak memory |
|---|---|---|---|
| plain C: no type check, no bounds checks (EnTT's `static_cast`) | 90.266 ms (1.000) | 116.590 ms (1.000) | 15,784 KB |
| Ember, pool table keyed by the type: bounds checks, no type check | 109.069 ms (1.208) | 126.346 ms (1.084) | 15,780 KB |
| Ember, `cast` of a `Box[dyn]` place: bounds checks plus one type check | 116.874 ms (1.295) | 136.170 ms (1.168) | 15,784 KB |
| safe Rust, `Box<dyn Any>` plus `downcast_mut` | 154.331 ms (1.710) | 187.301 ms (1.606) | 16,160 KB |

How to read it:
- **The type check** costs 7.2% inlined and 7.8% out of line (the cast row over the table row).
  That is under 15%, so it is PROVISIONAL. The table does not rest on it anyway: it is also the
  Pythonic spelling.
- **Plain C against the table row** is Ember's bounds checks on `sparse[e]` and `dense[i]`. Every
  Ember index pays them, and they are not this group's question. I note them for the final measurer.
  The spread on these rows was 7% to 37%.
- **Rust is slower** because `downcast_mut` calls `type_id()` through the trait object's table on
  every get: in `out/get_rust.s` the timed loop contains `callq *24(%rax)`, followed by a 16-byte
  compare (`pcmpeqb`). Ember does not need that call. A `Box[dyn I]` is "pointer + table" (spec,
  Part IV type table), so the check is a load and a compare. Whether the compiler would compare a
  type id or the table pointer itself is UNVERIFIED.

### 3.5 Answer: the return type says `ref`, so the `return` need not (kind (a), small)

Python: `return self.dense[i]`.

Ember. This is `p05a`; today it is E2020, and proposed it compiles:

```ember
struct Pool:
    dense: Array[int]

    fn at(mut self, i: int) -> ref mut int:
        return self.dense[i]           # the declared result is `ref mut int`, so this place is lent
```

The rule: when the declared result is `ref T` or `ref mut T`, a `return` of a place of type `T`
lends that place with that mode. Nothing changes in the code: it is the same reference. It is
safe because it is exactly the `return ref mut ...` that today's checker already checks.

### 3.6 Kept, not chosen: accessors that `yield` (kind (b))

Taken from Hylo and Swift, `phase2-hylo-csharp.md` H1. Python's closest form is `@contextmanager`:

```python
@contextmanager
def component(self, T, e):
    pool = self.pools[T]
    yield pool.dense[pool.index[e]]
    pool.touched += 1                  # runs when the with-block ends
```

```ember
fn component[T](mut self, e: Entity) -> ref mut T:
    pool = cast self.pools[type_id[T]()] as Pool[T]
    yield pool.dense[pool.index_of(e)]  # the caller works on the part here
    pool.touched += 1                   # runs after the caller's last use of the part
```

The rules:
- The accessor yields exactly once on every path.
- The part lives in the caller like a returned reference, under the same loans.
- The code after the `yield` runs at the caller's last use of the part, on every path.
- A part cannot be passed further up past the caller, unless the caller is itself an accessor that
  yields it.

It is safe because the accessor's locals stay alive until the code after the `yield` has run. So
"`pool` is a local, its storage ends with the frame" (E3060) cannot happen.

Cost:
- Inlined, it is the same instructions as writing the code at the call.
- Out of line, it is two direct calls: the part before the `yield`, and the part after it (only if
  there is code after it), with the accessor's locals in a frame on the caller's stack. That is what
  a C begin/end pair costs. It is not measured, because there is no code beyond what C would run.

**Why it is not chosen.** With `Box` pools, 3.3 to 3.5 give a plain `return`, with no frame to keep
alive. What only `yield` can do is run code after the caller is done. That matters to the classes
group, for ending a class pool's run-time access (`[EXC-18]`) if pools become class objects again,
and to A33 (references out of a lock guard). A computed property such as `a.degrees += 90` is better
written as Python's getter-and-setter pair (`py_checks.py`), which needs no reference at all.

### 3.7 Discarded for A5, with the reason and the good parts

- **Checked returned references**, from the 'checked pointers' design (D2). Every use of a returned
  reference compares a generation number. Phase 2 measured it: +0.57 ns per use in order (+23%),
  +4.61 ns random (+45%), and 8 bytes more per reference (`FINDINGS.md` section 2).
  - Try to save it: remove the check wherever nothing between the get and the use can destroy the
    element. That proof is exactly the loan rule, and once it holds the check is gone: the saved
    version *is* 3.3 and 3.4. So for returned references the generation adds nothing.
  - Good part: it is right for references to objects stored in fields across frames, which is the
    classes group's A8.
- **Pinning the pool while the returned element is in use**, from 'today's rules, but never an
  error' (D3).
  - Its attacker found that `pop()` under a pin hands out a value the tombstone still holds
    (`phase3-break-D3.md` 1.2). The fix costs a clone per pinned `pop`.
  - With `Box` pools, the loan rule refuses the change at compile time instead, at no cost.
  - Good part: letting a loop add to a pool, which belongs to the containers group (A4, paged pools).
- **Returning copies**, from 'copy instead of share' (D1). `reg.get(e).x += 1` would change a copy.
  The design itself used a yielded place for this, which is 3.6.

**Choice for A5:** 3.3, 3.4 and 3.5 together. They give Rust's compile-time guarantee with Python's
spelling, at no run-time cost; the table even removes the cast that Rust and C both need.

---

## 4. A6: structs holding references

### 4.1 What Rust's rule protects against

A struct that holds a view must not outlive what it views. Without the rule, in C:

```c
Lexer make_lexer(const char *path) {
    char *text = read_file(path);
    Lexer l = { text, 0 };
    free(text);
    return l;                          /* l.src points into freed memory */
}
```

### 4.2 The Python program, and what Ember does with it today

```python
vs = [View(reg, 0), View(reg, 1)]                    # handles kept in a list
class Parser:
    def __init__(self, text): self.lexer = Lexer(text)
```

- **A struct holding views:** Ember accepts it with no lifetime syntax (`e06_struct_refs.em`).
- **Views kept in a local list:** refused (`probes/p06a_shared_views_in_local_array.em`):

  ```text
  error[E3063]: `View` is a view, so it may not be stored in an `Array` unless it is `static`
    = note: the view borrows `reg`, and this place has no bounding region …
  ```

- **An object holding a view:** refused (`probes/p06b_class_holds_view.em`):

  ```text
  error[E3063]: `Lexer` is a view, so it may not be stored in a class object unless it is `static`
  ```

- **Rust:** `probes/rust/r06a_shared_views_in_vec.rs` (a `Vec<View<'a>>`) and `r06b_rc_holds_view.rs`
  (an `Rc<Parser<'a>>`) both compile with no error.

So here Ember is worse than Rust. That is phase 1's A25 again: `[TYP-15]` makes every element of a
list unbounded, so only views of static data may go there.

### 4.3 Answer: a container of views is a view (kind (a))

Python: `vs = [View(reg, 0), View(reg, 1)]`.

Ember (`p06a`, unchanged; proposed, it compiles):

```ember
struct View:
    reg: ref Registry
    entity: int

    fn hp(self) -> int:
        return self.reg.hp[self.entity]

fn main():
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []                 # vs is a view: it carries reg's limit
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    total = 0
    for v in vs:
        total += v.hp()
    println(total)
```

**The rule.** An `Array`, `Map`, `Set`, `Option` or `Box` whose element type is a view type is itself
a view type. It carries its elements' limits, one per source, like `[LT-14]`'s region vector. It may
live wherever a view may (locals, parameters, results, fields of view types) and is refused wherever
a view is refused. This is Rust's `Vec<View<'a>>`, with nothing written.

**Why it is safe.** While the list is in use, every source it views must stay valid. That is the
same loan rule as for a single view (`[LT-17]`: a view is valid only while every limit it needs is).
Attacks I tried:
1. **Storing the list in an object's field or in a static.** Refused: those places are unbounded.
2. **Pushing a view of a source that ends sooner.** The list's limit grows to include it, so the list
   must be done with before that source ends.
3. **Returning the list.** Allowed when its sources are source parameters (`[LT-1]`).
4. **A generic function that stores the elements in a global.** Already covered by `[TYP-15]`'s rule
   that such a function "passes that to its callers".

**Cost.** None: limits exist only at compile time (`[LT-30]`: one layout, one symbol). The same
layout measured as Rust's `Vec<&str>` row in section 5.4: 544.758 ms against plain C's 534.895 ms
(1.018, provisional).

**Needs** the containers group: this is the same rule as their A25.

### 4.4 Answer: an object holding a view carries the view's limit (kind (a))

Python: `self.lexer = Lexer(text)` inside an object.

Ember (`p06b`, unchanged; proposed, it compiles while `text` lives):

```ember
struct Lexer:
    src: str
    pos: int = 0

class Parser:
    lexer: Lexer

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text))    # every handle to this Parser carries text's limit
    println(p.lexer.pos)
```

**The rule.** A class with a field of view type is a view type. Its handles carry the view's limits,
so a handle cannot be stored where a view cannot: a static, a field of an unbounded object, the
captures of an `owned fn`, or another thread. This is Rust's `Rc<Parser<'a>>` (`r06b` compiles).
It is safe because a handle can never reach a place that outlives its view's source. It costs
nothing, since limits are compile-time only. It **needs** the classes group: may a class handle
carry a limit?

### 4.5 The two other cases

- **Keeping a view beyond its source.** Move the source into the keeper. That is A7's answer (section 5).
- **Two `ref mut` handles to one registry** (`e06d_handles_in_array.em`). Rust refuses this too
  (two `&mut`). It is A3, the borrowing group's question.

### 4.6 Not chosen for A6

- **Counted views:** a view keeps its whole buffer alive with a count. Measured and kept under A7
  (section 5.5). They would let a view be stored *anywhere*, as in Python.
- **Generation-checked stored references:** these are for references to *objects* (the classes
  group's A8), not for views of values.

**Choice for A6:** 4.3 and 4.4, with A7 for the long-lived case. **What it lacks:** a view kept longer
than its source, when the source cannot be moved into the keeper, is refused at compile time, as in
Rust. One example is a global list of names cut out of a file buffer that is later freed. Python
allows it by counting. Ember's spelling for it is an owned `String` per name, an allocation per name,
as Python's `split()` also makes.

---

## 5. A7: self-referential data

### 5.1 What Rust's rule protects against

There are two failures. In C, the views point into the struct's own text and the text is grown:

```c
typedef struct { char *text; size_t n; Slice *words; size_t nw; } Document;
doc.text = realloc(doc.text, 2 * doc.n);   /* moves the characters: every word now points into freed memory */
```

And when the viewed data is stored inline in the struct:

```c
typedef struct { char text[64]; Slice words[8]; } Small;   /* words point into text */
Small b = a;            /* the copy's words still point into a.text, which may go away */
```

### 5.2 The Python program, and what Ember does with it today

```python
class Document:
    def __init__(self, text):
        self.text = text
        self.words = text.split()
```

- `e07_self_ref.em` is refused at storing views in an `Array` (E3063).
- With a single view and no list (`probes/p07a_one_view_of_sibling.em`) the real rule shows:

  ```text
  error[E3024]: `text` would be moved into this value while another of its fields borrows it
    = note: a field cannot borrow another field of the same value: moving the value would move what the view points into (TYP-15)
  ```

- The Python order, `Document(text=text, words=text.split_whitespace().collect())`
  (`probes/p07b_document_python_order.em`), is E3063.
- Rust: `a07_self_ref.rs` gets E0515 and E0505. The usual workaround stores index ranges.

### 5.3 Answer: a value may hold views of its own heap contents (kind (b))

Ember (proposed: compiles as written):

```ember
struct Document:
    text: String
    words: Array[str]          # may point into text's characters

fn load(owned text: String) -> Document:
    return Document(text=text, words=text.split_whitespace().collect())
```

The same as a class, in Python's `__init__` shape:

```ember
class Document:
    text: String
    words: Array[str]

    fn init(mut self, owned text: String):
        self.text = text
        self.words = self.text.split_whitespace().collect()
```

**The rules:**

1. **What may be viewed.** A view held in a field, or inside a container held in a field, may point
   into the *heap contents* of another field of the same value: a `String`'s characters, an
   `Array`'s elements, a `Box`'s contents, at any depth. It may not point into *inline* storage of
   the value: a fixed array, a plain struct field, the `String` header itself. That stays E3024,
   because moving the value moves those. A class object never moves, so in a class even inline
   fields may be viewed this way.
2. **Writing the viewed field.** Any write to the viewed field makes every field that may view it
   unusable until that field is assigned again. A write is an assignment, a `mut` method (`push`,
   `clear`), `mem.take`, `mem.replace`, `mem.swap`, a move out, `ref mut`, or passing it to a `mut`
   parameter. Reading the viewed field is always fine.
3. **Checkpoints.** All fields must be usable at three kinds of point:
   - wherever the whole value is used: passed on, returned, moved, stored, copied, at a `yield`;
   - at the end of a function, for a `mut` parameter;
   - at the end of a `ref mut` loan through which a field was written.

   For a field reached through a class handle there is one more: before any call that is not quiet
   (`[EXC-19]`), because other handles to the object can only run code during calls.
4. **Which field may view which** is worked out from the field types, with no annotation: a `str`
   may view a `String`, a `Span[T]` an `Array[T]`, and a view struct whatever its views can. Inside
   one function the compiler knows each value's actual sources and uses them. A parameter whose
   sources are unknown is treated as if its views do point into its own fields.
5. **Storage.** A value whose views point only into static data or into itself may be stored
   anywhere: its limit is "itself". This extends `[TYP-15]`: "static or self".
6. **Order of construction.** In a struct literal, a field filled by moving a local is moved in when
   the literal is complete, after all the field values have been worked out. So `text=text` and
   `text.split…` can sit in the same literal, in Python's order. The loan held by `words` passes
   from the local `text` to the field `text`; the characters did not move.
7. **Clone.** A derived `clone` re-bases each view that points into the old buffer onto the new one:
   one compare and one add per view, in a clone that already allocates and copies. The minimal
   alternative is to refuse `@derive(Clone)` on such a type, as Rust's `ouroboros` crate does.

**Why it is safe.** The characters of a `String` do not move when the `String` header moves: Ember's
`String` is "inline header + heap buffer" (spec, Part IV type table), with no inline small-string
form. The only ways they move or are freed are writes to the `String`: growing may reallocate,
assignment frees, a move or `mem.take` hands the buffer to someone else. Rule 2 makes every such
write leave the views unusable, and rule 3 makes sure no code outside the function sees a value with
unusable views. Attacks I tried:

1. `d.text.push_str("x")` then `println(d.words[0])`: refused, the words are unusable.
2. `t = mem.take(d.text)`, then `mem.drop(t)`, then use `d.words`: refused, `mem.take` is a write.
3. `fn f(mut d: Document): d.text = "x"`, returning without refilling `d.words`: refused at the end
   of `f`.
4. `grow(d.text)` with `fn grow(mut s: String)`: refused afterwards, passing to `mut` is a write.
5. `r = ref mut d`, then `r.text.push_str("x")`, then the loan of `r` ends: refused, rule 3 at the
   end of the loan.
6. Through a class handle: `h.doc.text.push_str("x")` followed by a call that runs program code
   before `h.doc.words` is refilled. Refused: another handle could read the words during that call.
7. `docs.push(d)`: allowed. The header bytes move, the characters do not, and the views stay right.
8. A view into a `String` that sits in an `Array[String]` field, then `texts.push(...)`: refused.
   This is conservative: the strings' characters would not move, but a write to `texts` counts.
9. `mem.swap(d1.text, d2.text)`: both documents' words become unusable.
10. `Document(text, words) = d`: gives back two locals in today's local relation, so `text` cannot be
    freed while `words` is in use.
11. A future inline small-string form would break rule 1. Rule 1 therefore needs the guarantee that
    `String` and `Array` never store their contents inline (section 9).

**Cost.** None at run time. `Document` is laid out as in C (`text` pointer, length and capacity;
`words` pointer, length and capacity), and `load` is the loop in `bench/words_plain.c`. There is no
separate model, because the emitted code *is* the plain C program. Measured in 5.4.

**Readability.** The Python program, with the same shape.

**What it gives up:**
- Changing `text` in place means rebuilding `words` after.
- A struct's inline fixed array cannot be viewed by its other fields (an object's can).
- Rule 4 is conservative for a struct that holds both a `String` and views of something *else*:
  writing the `String` in a function that received the struct as a parameter makes the views
  unusable there.

### 5.4 What it costs against C and Rust (measured)

The workload is the same in every row (`bench/words_common.h`, `bench/words_common.rs`):
- the text is 4,000,000 words of 1 to 12 letters;
- it is split into stored words;
- 10 passes hash every word's bytes;
- 8,000,000 random words are read;
- then everything is dropped.

`work_ms`, median of 5:

| model | work | ratio | peak memory | ratio |
|---|---|---|---|---|
| plain C, words as (pointer, length); **this answer emits the same code** (`words_plain.c`) | 534.895 ms | 1.000 | 99,340 KB | 1.000 |
| safe Rust, `Vec<&str>` with the text owned outside (no self-reference) | 544.758 ms | 1.018 | 99,744 KB | 1.004 |
| safe Rust's self-reference workaround: index ranges, the text sliced again on each use | 644.386 ms | 1.205 | 99,736 KB | 1.004 |
| Ember today: an owned `String` per word (what E3063's help asks for; Python also allocates per word) | 776.319 ms | 1.451 | 210,800 KB | 2.122 |
| safe Rust, `Vec<String>` | 813.861 ms | 1.522 | 211,200 KB | 2.126 |
| counted views as last night's 'copy instead of share' design specified them (5.5) | 586.593 ms | 1.097 | 132,644 KB | 1.335 |
| counted views, saved (5.5) | 555.842 ms | 1.039 | 99,368 KB | 1.000 |

Rust's index-range workaround slices `&self.text[a..b]` on every use, which checks the bounds and
both character boundaries. The answer here keeps C's (pointer, length) views, so it pays neither.

### 5.5 Kept, not chosen: counted views (kind (c))

This is phase 1's A7(b), the string of the 'copy instead of share' design (D1). A word is (buffer,
start, length), and the buffer has a count, so a word keeps its text alive. That needs no rule at
all: a word can go anywhere.

Python behaves this way for strings: `str` is immutable, and a kept object keeps its data.

- **As specified** (`words_counted24.c`): a 24-byte word, a count change for every word made and
  every word dropped. 1.097x the time and 1.335x the memory.
- **Saved** (`words_counted16.c`). Three changes:
  - a 16-byte word (a pointer and two 32-bit numbers, so one buffer is at most 4 GiB);
  - the retains made in one loop are added once, after the loop (the compiler sees that every pushed
    word retains the same buffer);
  - the list's drop releases runs of words that share a buffer with one decrement per run.

  The result is 1.039x the time and 1.000x the memory: under 15%, so provisional.

**Why it is not chosen.** The static answer gives the same program for nothing. The saved counted
version still has costs no change to it removes:
- a count word per buffer;
- a retain on every store the compiler cannot group, and a release that touches every element when
  a list of words is dropped;
- strings that must be immutable, or copied on write (phase 2 measured +10% per write);
- atomic counts, or freezing, for strings shared between threads;
- one kept five-letter word keeps the whole text alive. Java changed `substring` to copy in 7u6
  because of that (`phase3-break-D1.md` N8).

**Kept** because it is the one way to store a view anywhere with no rule. **Good parts:** the
grouped retains and the run-length release are general count-reduction techniques for any counted
type.

### 5.6 Not needed: index ranges

Index ranges are Rust's workaround. Measured above at 1.205x. Nothing in Ember needs them under 5.3.

**Choice for A7:** 5.3. It meets all three goals; what is left is listed under "What it gives up".

---

## 6. A22: a view of a temporary

### 6.1 What Rust's rule protects against

In C:

```c
char *lower = to_lower(line);
const char *word = trim(lower);
free(lower);                       /* the temporary is freed at the end of the statement ... */
return strlen(word);               /* ... and word points into it: a read of freed memory */
```

### 6.2 The Python program, and what Ember does with it today

```python
word = line.lower().strip()
best = ""
for line in lines:
    w = line.lower().strip()
    if len(w) > len(best): best = w
def first_word(line): return line.lower().strip()
```

- **The view kept past its statement** (`e22b_temporary.em`):

  ```text
  error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
  3 |     word = line.to_lower().trim()
    = help: bind the value to a variable first, so it lives as long as the borrow
  ```

  The hand fix compiles (`probes/p22a_hand_fix.em`).
- **The best word kept after the loop** (`probes/p22c_best_in_loop.em`): two errors.
  - `E3021: `_14` cannot be written while it is borrowed`, which names a compiler-internal
    temporary (section 11).
  - E3060.

  The hand fix with an owner beside `best` compiles (`probes/p22d_best_in_loop_hidden_owner.em`).
- **A view of a temporary returned as a `String`** (`probes/p22b_return_view_of_temp_owned.em`):

  ```text
  error[E2020]: expected `String`, found `str`
    = help: make a `String` of it: `.to_string()`
  ```

- **A `mut self` method on a temporary** (`probes/p22e_mut_method_on_temporary.em`,
  `line.split(" ").next()`): `E2140: a `mut self` method needs a variable to write through`, with no
  help line. The Rust twin (`probes/rust/r22e.rs`) compiles, so here Ember is worse than Rust.

### 6.3 Answer: a temporary lives as long as the views made from it (kind (a))

Python: `word = line.lower().strip()`.

```ember
fn first_word_len(line: str) -> int:
    word = line.to_lower().trim()      # the lowered String lives on, as a hidden local
    return word.len()
```

**The rule.** When a value made in an expression is borrowed by a value that is used after the
statement, the temporary becomes a hidden local of the statement's block. It is dropped at the end
of the block, which is what the hand fix (`p22a`) does. If the other group adopts freeing
memory-only values at their last use, it is dropped there instead.

Temporaries with a `drop` of their own (lock guards, files) keep today's error. A lock held longer
than the line suggests is a change of behaviour the reader cannot see. Rust 2024 *shortened*
tail-expression temporaries for that reason (the edition guide, cited in phase 1's A22).

With the containers group's rule (4.3), `words = line.to_lower().split()` is covered too: the list of
views holds the temporary.

**Why it is safe.** The compiler rewrites the program into the hand fix, and today's checker checks
that. If the rewritten program fails, the user gets today's error.
**Cost.** None: the same code as the hand fix, and the temporary lives exactly as long as there.

### 6.4 Answer: the owner follows the view (kind (b))

Python: the `best` loop above.

```ember
fn longest(lines: Array[String]) -> int:
    best: str = ""
    for line in lines:
        w = line.to_lower().trim()
        if w.len() > best.len():
            best = w                    # the lowered line moves into a hidden owner beside `best`
    return best.len()
```

**The rule.** When a view of a temporary is assigned to a variable of an outer block, the temporary
moves into a hidden owner declared beside that variable. Assigning the variable again drops the
previous owner, after the new value is made. When the assignment happens on some paths only, a drop
flag says whether to free, as `[OWN-3]` already has for moves made on some paths only. That is the
`if (owned) free(old)` a C programmer writes.

**Why it is safe.** It is a rewrite into `p22d`, which today's checker accepts. If another view still
points into the old owner when the variable is assigned again, today's checker refuses it. Attacks I
tried:
1. Two variables keeping views of one temporary: the owner goes beside the outermost of them, and
   the checker verifies the others.
2. `best` later assigned a view of something that is not a temporary: the hidden owner keeps or drops
   its old value, and the checker decides whether that is allowed.

**Cost.** None against C doing the same work: C keeps the best buffer and frees the one it replaces.
**Limit.** It does not apply to list elements, since each would need its own owner. Those are refused,
with a help that names `String(w)`: an allocation per element, as Python makes.

### 6.5 Answer: a view returned as an owned string reuses the temporary's buffer (kind (b))

Python: `def first_word(line): return line.lower().strip()`.

```ember
fn first_word(line: str) -> String:
    return line.to_lower().trim()      # the trimmed characters are moved to the front of the
                                       # temporary's buffer, and that String is returned
```

**The rule.** When the declared result is an owned `String` (or `Array`), and the returned view points
into a temporary or local that this function owns and that ends here, the compiler moves the view's
characters to the start of that buffer, sets the length, and returns it. This applies only when
exactly one owned result is made from that buffer. Otherwise the user gets today's E2020, with
`.to_string()` as the help.

**Why it is safe.** The buffer belongs to this function and ends here, so nothing else can see it.
**Cost.** One `memmove` of the result's characters. That is less than today's fix (`.to_string()`
allocates, copies, and frees the temporary), and the same as a C programmer narrowing a buffer in
place. Not measured: it is strictly less work than today's spelling.

### 6.6 Answer: a temporary is a place for its statement (kind (a))

Python: `n = len(next(iter(line.split(" ")), ""))`, or simply `line.split(" ")[0]`.

```ember
fn first_len(line: str) -> int:
    return line.split(" ").next().unwrap_or("").len()    # p22e: today E2140
```

**The rule.** A value returned by a call is a place until the end of its statement, so a `mut self`
method may be called on it. Rust accepts this (`r22e`). Assigning to a field of a temporary
(`f(x).y = 1`) stays E2140, because the change could never be seen. A lint can catch a mutating call
whose effect is thrown away (`make_list().push(3)`). Cost: none; the temporary already lives on the
stack.

### 6.7 Discarded for A22

- **Copying a view when it escapes**, from 'today's rules, but never an error' (D3). It is a hidden
  allocation. 6.5 needs none, and the explicit `.to_string()` stays for the remaining cases.
  - Good part: the escape is visible at compile time, and 6.5 uses exactly that.
- **"A temporary lives on while a local refers to it"**, from 'checked pointers' (D2). Too narrow: a
  list of slices of a temporary was not covered (`phase3-break-D2.md` N4). Saved as 6.3 plus the
  containers rule.
- **Counted views:** as A7 (5.5).

**Choice for A22:** 6.3 to 6.6.

---

## 7. A23: moving a field out from behind `mut self` (state machines)

### 7.1 What Rust's rule protects against

In Rust, a panic between the move out and the reassignment unwinds to a caller that still owns
`*self`. That caller then drops the moved-out field a second time: a double free. Any early exit
that forgets to refill the field does the same. In C:

```c
Vec32 path = npc->state.path;        /* moved out */
if (bad) return;                     /* forgot to refill: npc->state still says WALKING with this path */
free(path.p);                        /* ... and the caller frees npc->state.path again: a double free */
```

### 7.2 The Python program, and what Ember does with it today

```python
def update(self, hurry):
    if isinstance(self.state, Walking):
        self.state.path.pop()
        if not self.state.path: self.state = Idle()
        elif hurry: self.state = Running(self.state.path)

def rename(p, new_name):
    old = p.name
    p.name = new_name
    p.log.append(f"renamed {old}")     # prints "renamed ann" (py_checks.py)
```

- **Python-shaped** (`probes/p23f_python_shaped.em`):

  ```text
  error[E3013]: cannot move out of a reference
  17 |                     self.state = State.Running(path)
     = help: … move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap` …
  ```

- **Keeping the old name** (`probes/p23g_keep_old_value.em`, `old = p.name`): E3013 as well.
- **Writing in place with a plain binding** (`probes/p23a_in_place.em`, `path.pop()`):
  `E3021: cannot write through a shared reference`. The `match` bound `path` read-only. The spelling
  `State.Walking(ref mut path)` compiles (`p23e`).
- **What already exists:** on an owned local, moving the field out and putting a value back compiles
  (`p23c_local_partial_move.em`). So the analysis that tracks an empty field already exists, for
  locals only.
- **Rust:** `probes/rust/r23f_python_shaped.rs` gets `E0507: cannot move out of `*path` which is
  behind a mutable reference`, with the help `path.clone()`. The documented workaround is
  `mem::replace`.

### 7.3 Answer: a field held exclusively may be emptied, if every path refills it (kind (b))

**The rule.**

1. **What may be emptied.** Inside a function, a place reached from a `mut` parameter or from
   `mut self` by field names only may be moved out of. Field names include the fields of an enum
   variant inside a `match` arm, and the contents of a `Box`. The path may not go through an index,
   a class handle, a `Shared`, or a reference stored in a field. A move can be:
   - a `match` binding by move;
   - `x = place`;
   - passing the place to an `owned` parameter;
   - using it inside a new value, as in `State.Running(path)`.
2. **While it is empty.** From the move until the place is assigned again, the place is empty. These
   are errors, and the message names the line of the move:
   - reading it or borrowing it;
   - using what contains it as a whole: passing `self` on, returning, a closure that captures it;
   - calling a method of `self` that uses that field, by `[BRW-10]`'s summary of the fields each
     method uses (a public method uses all of `self`).
3. **Refilling.** Every path refills it before the function returns (including `?` and early
   returns) and before a `yield`. In a loop, it must be refilled before the iteration ends if the
   next iteration reads it (`[OWN-4]`).
4. **Overwriting.** Assigning to a place that is partly empty drops only the parts still there,
   which is known on each path at compile time, then stores.
5. **Panics.** Ember never unwinds (`[PAN-1]`: a panic calls `abort()`). A panic between the move and
   the refill ends the process, and no caller sees the empty field. If unwinding is ever added, this
   rule must be revisited.
6. **Class objects** are a question for the classes group. For a field of a class object, the empty
   stretch can be a held write access on that field's access word (`[EXC-19]`), checked at run time
   only when code runs in between. Or it can be static: no call that runs program code between the
   move and the refill. Today's E3012 stays until that is decided.

### 7.4 Answer: bindings follow their use (kind (a))

Ember: the Python-shaped program, `p23f` without `ref mut`. Proposed, it compiles:

```ember
enum State:
    Idle
    Walking(path: Array[int])
    Running(path: Array[int])

struct Npc:
    state: State

    fn update(mut self, hurry: bool):
        match self.state:
            State.Walking(path):              # written through below, so a `ref mut` (inferred)
                path.pop()
                if path.is_empty():
                    self.state = State.Idle
                elif hurry:
                    self.state = State.Running(path)   # moves the path out, then overwrites the state
            _:
                pass

fn rename(mut p: Person, owned new_name: String):
    old = p.name                  # refers to p.name ...
    p.name = new_name             # ... whose old value moves into `old` just before this overwrite
    p.log.push(f"renamed {old}")  # "renamed ann", as in Python
```

**The rules:**

- An arm's binding is a shared reference when the arm only reads it. It is `ref mut` when the arm
  writes through it; that is `p23e`'s spelling, inferred. It is a move when the arm moves it, as
  `State.Running(path)` does, and the move empties the place under 7.3 on that path only.
- **Move at overwrite.** If a name that refers into the place is still used after the place is
  overwritten in the same body, its value moves out just before the overwrite, with no copy and no
  placeholder. This applies only to places the function holds exclusively (a `mut` parameter,
  `mut self`, an owned local), where every write to the place is in this body. A write made by a
  *call* while the name is in use is a borrow conflict, refused as today, never a silent change.

**Why it is safe.** The rule extends `[OWN-3]`'s empty-field tracking from locals to places the
function holds exclusively, and that tracking is already sound for locals (`p23c`). Attacks I tried:
1. An early return without refilling: refused (rule 3).
2. `?`: the same.
3. Calling `self.helper()` while the field is empty: refused if `helper` uses `state`; allowed if it
   uses only `log`, by `[BRW-10]`'s exact summary of the fields each method uses.
4. A public method on `self`: refused, because it uses all of `self`.
5. A closure capturing `self`: refused.
6. `yield`: refused, and `mut self` is not allowed on a `gen fn` anyway (`[CORO-12]`).
7. Loops: `[OWN-4]`.
8. Through an index (`self.items[i]`): E3011 stays, because the list's other operations cannot know
   which element is empty.
9. Through a class handle: rule 6.
10. Threads: `mut self` is exclusive, so no other thread can see the place.
11. Hot reload of a helper that starts using `state` while its caller has emptied it: the helper's
    field summary widens, so the reload is refused (the same check as 3.3).
12. Dropping the containing value while the field is empty: behind `mut` it is not dropped in this
    function. For an owned local, the end of the scope is a whole use, so the field must be refilled
    first. That means `E3010` (no moving a field out of a type with its own `drop`) could also be
    relaxed: the value is whole whenever its `drop` can run.
13. Move at overwrite across a call (`p.upgrade()` changing `p.weapon` inside a method). That is
    what broke the same idea in 'today's rules, but never an error' (`phase3-break-D3.md` 2.3). Here
    it is a borrow conflict while the name is in use, so it is refused, not silently given a
    different meaning.

**Cost.** None. `bench/state.c` is plain C, and it is what the rule emits: read the payload, compute,
store the new state. No placeholder is written, and no drop flag is tested, because a path that does
not refill is a compile error. The workload is 1,000,000 NPCs over 100 ticks, with paths of 1 to 32
steps, transitions that carry the path, and frees when a path empties. Total ms, median of 5:

| model | time | ratio | peak memory |
|---|---|---|---|
| plain C (`state.c`) | 507.654 ms | 1.000 | 113,688 KB |
| the Ember model: `state.c` plus the emptiness check Ember's `pop()` makes (`state_checked.c`) | 507.427 ms | 1.000 | 113,684 KB |
| safe Rust, the documented `mem::replace` idiom (`state_rust.rs`) | 595.473 ms | 1.173 | 114,024 KB |
| safe Rust in place, `mem::take` only on transitions (`state_rust_inplace.rs`) | 499.465 ms | 0.984 | 114,028 KB |

**Why the `mem::replace` row is slower.** I read its loop in `out/state_rust.s`. It stores the
placeholder tag (`movq $0, (%rdi)`) and then writes all four 8-byte words of the state back, for
every NPC on every tick, including idle NPCs whose state does not change. `state.c` and the in-place
Rust write only when the state changes. That is one reason, seen in the assembly; I did not measure
it with hardware counters.

**Readability.** The Python shape, with `match` in place of `isinstance`, and no `mem.replace` or
placeholder value.

### 7.5 Saved from last night's designs

- **'Copy instead of share' (D1): "there are no moves; `match` binds in place".** Its attacker showed
  that using `path` after `self.state = State.Idle` panics or reads garbage (`phase3-break-D1.md` N4).
  Saved as move at overwrite (7.4): `path` keeps the old value, with no check.
- **'Checked pointers' (D2):** `old = p.name; p.name = new; print(old)` panics there
  (`phase3-break-D2.md` N1). With 7.4 it prints the old name, as Python does, with no check.
- **'Today's rules, but never an error' (D3): "a name keeps the old value".** It broke across calls
  (`phase3-break-D3.md` 2.3). Saved by allowing it only on places held exclusively, where every write
  is visible in the body.

**Choice for A23:** 7.3 plus 7.4.

---

## 8. How the five answers fit together

1. **One summary per function** of what its result points into (3.3). It is recorded in the package
   interface and checked on hot reload. It serves all five annoyances:
   - A5: callers know what each result borrows;
   - A6: which sources a stored view carries;
   - A7: whether a result's views point into the result itself (rule 5: limit "itself");
   - A22: which temporary a kept or returned view depends on.
2. **One analysis of unusable fields:** `[OWN-3]`'s tracking of empty fields, extended from locals to
   places a function holds exclusively.
   - A23's empty field and A7's views made unusable by a write are the same state: "not usable until
     assigned".
   - They have the same checkpoints: before a whole use, before the function ends, and, on objects,
     before a call that runs program code.
3. **One fact about memory:** heap contents do not move when their owner moves. It is used by A7
   (views of another field's characters), by A22's 6.5 (the temporary's buffer becomes the result),
   and by A5 (a reference into a box's contents).
4. **One rewrite for temporaries:** 6.3 and 6.4 rewrite into programs today's checker accepts, and
   6.5 reuses the buffer that this rewrite keeps alive.
5. **One rule for where a view may be stored.** A view may be stored wherever its limit is known:
   - locals, results and fields of view types (today);
   - containers of views (4.3);
   - objects that carry the limit (4.4);
   - its own value (A7, limit "itself").

   The one thing left out is "anywhere, with no limit". Counted views (5.5) would give that, at the
   measured cost.

---

## 9. What this group needs from the other groups

- **Containers (A2, A4, A25):**
  - Containers of views carry their elements' limits (4.3; this is A25's fix too). Without it, A7
    works only for single views (`p07a`'s `first: str`), and 6.3 does not cover
    `words = line.to_lower().split()`.
  - A guarantee that `String` and `Array` never store their contents inline (A7 rule 1).
  - Whether paged pools let `emplace` run while a returned element is in use. If they do, A5 needs no
    refusal there.
- **Borrowing (A1, A3):**
  - One decision for the public-function boundary, covering both `[BRW-10]`'s field summaries and my
    result summaries (3.3).
  - Two `ref mut` handles to one registry (`e06d`).
  - Path-based invalidation, which would accept `self.pools.insert` while a reference into a box's
    contents is in use.
- **Moves (A11):** what `x = place` means for a value that cannot be copied. My 7.4 assumes a name
  bound into an exclusively held place is a reference by default, as `match` already binds, and
  becomes a move only at an overwrite or an explicit move.
- **Classes (A8, A9, A10, A24):**
  - May a class handle carry a view's limit (4.4)?
  - Emptied class fields (7.3, rule 6).
  - Unusable views reached through a handle (A7 rule 3, refill before any call that runs code).
  - If pools become class objects again, the `yield` accessor (3.6) carries their run-time access.
- **Closures and generators (A12, A14):** the heap-contents fact would also let a generator keep a
  view into its own local `String` across a `yield`. Today `[CORO-6]` forbids that, because frames
  move; heap contents do not.
- **Diagnostics (A17):** the defects in section 11.

---

## 10. Good parts kept from answers not chosen

| Idea | Where it came from | What is good in it | Where it may be useful |
|---|---|---|---|
| Counted views; grouped retains; run-length release; the 16-byte word | 'copy instead of share' (D1); phase 1 A7(b) | measured 1.039x time, 1.000x memory (provisional) | storing a view anywhere; any counted type |
| Generation-checked references | 'checked pointers' (D2); Vale | a stale use is a clean panic, never a use after free | references to objects stored across frames (A8) |
| Pins: a container holds still while referenced | 'today's rules, but never an error' (D3) | lets a loop add to a container | A4, with paged pools |
| "A name keeps the old value" | 'today's rules, but never an error' (D3) | Python's meaning for `old = x.f; x.f = new` | saved here as 7.4, for places held exclusively |
| Accessors that `yield` | Hylo, Swift (phase 2 H1) | code runs after the caller is done with a part | class access ends, lock guards (A33) |
| A downcast of a place is a place | Hylo's spec (phase 2 H2) | no handle and no copy for a type-erased part | used in 3.4 |
| Copying a view when it escapes | 'today's rules, but never an error' (D3) | escape is visible at compile time | used by 6.5 to reuse the buffer instead of copying |

---

## 11. Defects found in passing (today's compiler; nothing was changed)

1. **A compiler-internal name in a message.** `probes/p22c_best_in_loop.em` gets
   ``E3021: `_14` cannot be written while it is borrowed``. `_14` is a name the compiler made up.
2. **E2140 has no help, and is worse than Rust.** `E2140: a `mut self` method needs a variable to
   write through` has no help line, and it refuses `line.split(" ").next()` (`p22e`), which Rust
   accepts (`r22e`).
3. **A misleading help.** In a `mut self` method, `match self.state:` binds `path` read-only, so
   `path.pop()` gets E3021 (`p23a`). Its help, "a reference is not pointed elsewhere … keep a copy of
   it (`.clone()`), or its index", does not name the fix that works: `State.Walking(ref mut path)`
   (`p23e` compiles).
4. **Confirmed from phase 1 (its defect 6).** A `pub` method called from its own package borrows only
   the fields it uses (`p05e` compiles, `reg.next_id += 1` while the result is in use).

---

## 12. Files

All under `target/redesign2/references/`.

- `probes/*.em`: the Ember probes named above. Their `ember check` outputs are in
  `probes/ember_check_output.txt` (24 files: 8 compile, 16 are refused).
- `probes/rust/*.rs`: the Rust twins. Their outputs are in `probes/rust/rustc_output.txt`: `r05f` and
  `r23f` are refused, `r06a`, `r06b` and `r22e` compile.
- `probes/py_checks.py`: every Python program in this report, with its result asserted.
- `bench/words_common.h`, `words_plain.c`, `words_counted24.c`, `words_counted16.c`, `words_owned.c`:
  the A7/A22/A6 C models.
- `bench/words_common.rs`, `words_rust_vecstr.rs`, `words_rust_ranges.rs`, `words_rust_owned.rs`: the
  Rust twins of the words workload.
- `bench/get.c` (`-DPLAIN`, `-DTYPED`, `-DCAST`, optionally `-DNOINLINE`) and `get_rust.rs`: A5.
- `bench/state.c`, `state_checked.c`, `state_rust.rs`, `state_rust_inplace.rs`: A23.
- `bench/harness.rs` (the Rust harness), `bench/run.ps1` (the runner), `bench/summarize.py` (the
  medians).
- `bench/out/results.txt`: the run used here. `bench/out/results_first_run.txt` is the earlier run,
  whose `get` rows are not valid (section 2).
- `bench/out/get_rust.s`, `get_cast.s`, `get_cast_noinline.s`, `state_rust.s`,
  `state_rust_inplace.s`, `state.s`: the assembly read for sections 3.4 and 7.4.
