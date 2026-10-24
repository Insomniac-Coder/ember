# Phase 3b: attacking and saving the containers-and-loops answers

Written 2026-10-24 for the owner, in two passes. It attacks the answers in `phase3b-containers.md`
(called "the answers report" below) for five annoyances: holding a list element while adding to the
list (A2), two elements of one list in one call (A3), changing a list while looping over it (A4), a list
of string views (A25), and creating and destroying entities inside an ECS view loop. For every problem
found I tried to save the answer first, measured the fix where it costs anything, and attacked the fixed
version again.

**Why there are two passes.** The first pass was written at 08:03. When the workflow was resumed after
the error-box problem (programs started through `cmd start` made Windows show "cannot find" boxes), this
step ran again, at 08:43. The first pass was **not** hit by that problem: its 89 shell commands contain no
`cmd /c start` (checked in its transcript, `agent-acefdd51781db2001.jsonl`), every C program it ran calls
`SetErrorMode` through the shared `harness.h`, every Rust program it ran through `rharness.rs`, and the two
Rust files without it were only checked (`--emit=metadata`), never run. I rebuilt and ran all of its
checking programs; every output is identical to the saved one (section 2.2). So its findings stand, and
this file keeps them under the same section numbers (the combined report cites them, for example
"containers attack 5.6"). The second pass attacked the first pass's saved versions and the answers
again; its findings are in the subsections marked **second pass** (3.6, 4.6, 5.7, 5.8, 6.6, 7.5), and
sections 0, 8, 9 and 10 are updated.

Nothing in the compiler, the standard library, the docs or the tests was changed. The first pass's
programs are under `target/redesign2/containers-attack/` (called `attack/` below), the second pass's under
`target/redesign2/containers-attack2/` (called `attack2/` below). No Ember program was run.

---

## 0. The short version

| Annoyance | The answer (what it does) | Verdict | First pass (re-checked) | Second pass (new) |
|---|---|---|---|---|
| A2, holding an element while adding to the list | A name for a list element is its position; it is looked up again after a change | **saved** | No memory hole. Four Python programs gave a wrong answer silently or were refused (the swap idiom swapped nothing; an element passed to a method that removes other elements was written over another enemy). Fixed at compile time: replacing a named slot moves the old value into the name; a function that may change the element's list gets a place (list and position), not a copy. | The rule for a name held across code the compiler cannot see was never built; it is the same mechanism as A4's mode for such code, and it shares that mode's verdict (5.8). One precision item: a place's list must be compared by identity when class handles may be one object. |
| A3, two elements of one list in one call | Two names may be one element; only a view into a part a write can replace is checked | **saved** | One memory hole (a view passed into `extend` through the other name) fixed by a range compare and a copy. The "Rust 2.1x" claim withdrawn (a random branch in the Rust model): Rust 1.13x, the answer 1.08x. A compile-time error Rust does not have removed (two compiled copies of the function). | **Two more memory holes** in the first pass's saved rules: a slice passed with the list of squads to a function that grows every squad's list read freed memory; a slice kept across `clear()` through the other name read freed names. Both fixed at compile time by Python's own meaning (the slice becomes a copy, only where the compiler cannot rule out that its source changes). Rust refuses both programs. |
| A4, changing a list while looping over it | The loop is compiled into a one-pass in-place filter | **saved** | Four memory holes in the mode for code the compiler cannot see, fixed; reads by position fixed (1.01x); other code sees Python's list. | **One more memory hole** in that mode (a list inside a list: replacing or appending to the outer list freed or moved the walked list), fixed by publishing the path. **Three Python programs** that went wrong, fixed for free: writes through the list by position were lost (fixed, 0.98x of plain C); `enumerate` plus `del xs[i]` removed a live enemy, then panicked (the index is now the element's position); the Python habit `for e in xs[:]` was refused, or would lose every write (now compiled as the same filter with the copy's bounds). **Speed:** the mode for unseen code costs 1.00x to 1.02x of plain C written to be correct for an unknown callback; the first pass's 1.20x to 1.29x was against C that knows its callback only reads the length. |
| A25, a local list of string views | A local list of views carries the lifetime of its text | **saved** | The usual file loop was refused; fixed by taking each line's text along (1.00x, Rust's workaround 1.13x). | **One memory hole**: a word kept across `clear()` of a list that took its lines along read a freed line. Fixed at compile time (the free moves to the word's last use; a compile-time error only where blocks would pile up in a loop). |
| A25, returning a list of views | A returned list takes its text along | **saved** | 24-byte owner field shrunk to an 8-byte pointer; copies need a count; a shrinking list must not keep texts. | The count is written by every copy; a list copied in two threads at once (a shared object that is never written) would race. Fixed with the threads group's shared flag (atomic only for shared blocks). |
| ECS, creating and destroying entities in a view loop | The library walks the pool backwards; components are names keyed by the entity; a cursor keeps "every entity once" | **saved** | The timed model's `destroy` broke the pool on two destroys in one step; one guard fixes it (1.09x of plain C, safe Rust 1.13x). | Checked in the layout the owner chose (EnTT's full-owning group): with `remove[Velocity]` and entities joining the group inside the loop, the rule holds over 5,000 random loops **only if every operation that takes an entity out of the walked range applies it**; `remove[T]` without it visits an entity it must not (seed 2: one that joined during the loop). The rule belongs in the group's "leave" operation. |

No answer had to be discarded whole. Every timing is PROVISIONAL (five agents were running while I
timed). No verdict rests on a timing difference under 15%, except where I say so.

---

## 1. Words used

- **List**: Ember's `Array[T]`, Python's `list`: a pointer to a block of memory holding the elements
  (the **buffer**), the length, and the capacity.
- **Reallocate**: a full list that grows gets a bigger buffer and frees the old one.
- **View**: an address, or an address and a length, into memory something else owns: `ref T`, `str`,
  `Span[T]`. Ember's slice `xs[a..b]` is a view (spec line 1928); Python's `xs[a:b]` is a copy.
- **Bounds check**: `i < len` before `xs[i]`; failing it is a **panic**, a stop with a message. Ember's
  panics abort; nothing is unwound (`[PAN-1]`).
- **Name** (A2's answer): `boss = enemies[0]` makes `boss` mean "element 0 of `enemies`".
- **Place**: a list and a position (or, in the ECS, a pool and an entity): what a name is.
- **Path**: how a place is reached from a variable, for example `self.squads[0].units`.
- **Walk**: a loop that changes the list it walks, compiled as the answers report's one-pass filter:
  a read position `r`, a write position `w`, kept elements copied down from `r` to `w`. The slots
  between `w` and `r` are **the gap**. The **logical list** (what Python would show) is the slots before
  `w` followed by the slots from `r` on.
- **Code the compiler cannot see**, or **unseen code**: a call through a function value, a virtual
  method, a foreign function (the answers report's rule 9 list).
- **Effect summary**: what the compiler records about a function: which lists (by type and path) it may
  add to, remove from, reorder or replace (answers report section 7 item 4).
- **Access word**: the per-field number today's Ember keeps in a class object, checked at run time so
  no one writes a field while someone else uses it (`[EXC-19]`, spec line 2712). A loop over `h.items`
  is a read access for the loop's duration (spec line 2699).
- **No-alias copy / alias copy** (the first pass's A3 save): a function with two parameters that may be
  one place is compiled twice; a call that cannot rule out that they are one place compares them and
  calls the **no-alias copy** when they differ, the **alias copy** when they are the same.
- **Spill / reload**: store a value held in CPU registers to memory before a call, and load it back after.
- **Store-forwarding stall**: when the CPU loads from memory that a recent store wrote, it normally
  takes the value straight from the pending store; if the load is wider than the store it cannot, and
  waits until the store reaches the cache (the answers report's section 1.4 found this cost 4.7x).
- **Poisoning allocator**: a test allocator used in the memory attacks: a freed block is filled with the
  byte `0xDD` and never handed back, so a read of freed memory shows `0xDD` (as an `int32`, -572662307)
  and a second free of one block is counted instead of being silent. Nothing crashes.
- **EnTT full-owning group**: the ECS layout the owner chose for W1: the entities that have both
  components sit at the front of both pools, in the same order (slots `0` to `glen - 1`).
- **Median of 5**: the middle of five runs. **PROVISIONAL**: measured while other agents ran.
  **UNVERIFIED**: I believe it but did not check it.

---

## 2. How I checked

### 2.1 Tools and timing

- **Ember**: `target/ci/ember.exe check <file>` (it prints `ember 0.1.0`, `language: 0.9.10`,
  `backend: c`). Today's checker refuses every attack program at the line the proposed rules would
  accept (first pass: `attack/probes/ember_check_attack_output.txt`; second pass:
  `attack2/probes/ember_check_attack2_output.txt`). Each attack below says which rule of the proposed
  answer accepts it.
- **Rust**: rustc 1.98.1. `rustc --edition 2021 --crate-type lib --emit=metadata <file>` for borrow
  checks (`attack/probes/rust/rustc_attack_output.txt`, `attack2/probes/rust/rustc_attack2_output.txt`);
  `rustc -C opt-level=2 --edition 2021` for timed programs.
- **Python**: `C:/Python314/python.exe -I <file>` (Python 3.14.2). `attack/probes/py_attack.py` and
  `attack2/probes/py_attack2.py` assert every Python behaviour quoted below and print `all Python attack
  checks passed` (outputs beside them). `py_attack2.py` also holds a Python model of the walk (A4 rules
  1 to 4 with the first pass's logical-position mapping), used for the readability attacks of 5.7.
- **C**: `"C:/Program Files/LLVM/bin/clang.exe" -O2 <file>.c -o <file>.exe`. Every C program calls
  `SetErrorMode` (the shared `target/redesign/bench/harness.h`).
- **Models**: each "as written" C program is what the answer's rules compile the Ember program into,
  built from the answers report's own models (`containers/bench/`); each "saved" program is the fix. The
  memory attacks are not timed; they count double frees, reads of freed memory and elements owned twice.
- **Timing**: the answers report's runner, copied to `attack/run.ps1`: 5 rounds, one program at a
  time, started with PowerShell's `Start-Process -NoNewWindow`, pinned to the performance cores
  (`ProcessorAffinity = 0xC03C03`), order forward in odd rounds and reversed in even rounds,
  `(Get-CimInstance Win32_Battery).BatteryStatus` checked to be 2 (mains) before every round. Command:
  `powershell -NoProfile -File attack/run.ps1 <results file> <exe> ...`; medians with
  `sh attack/summarize.sh <results file>`. No round aborted and no affinity warning was printed. Every
  run of every program printed the same checksum as the plain C program it is compared with, except
  where a wrong checksum is the finding. "1.08x" is a median divided by the plain C median of the same
  file.
- **One disturbed run** (first pass): `attack/runs/ecs.txt` had plain C at 231.430 ms where five other runs
  of the same program gave 92 to 112 ms, and every program 2.3x to 2.8x slower. It was repeated as
  `attack/runs/ecs_2.txt`, and only that one is used.

### 2.2 The first pass re-checked (second pass)

Rebuilt with clang -O2 into `attack2/repro/` and run; each output compared with `diff` against the first
pass's saved output: `a2/a2_models` same, `a3/extend_attack` same, `a4/attacks` same, `a4/walker_check`
same (`walker-check ok`), `a4/count_calls` same (101,705,840 callback calls), `ecs/cursor_check2` built
three ways same (the original `del` breaks the pool at seed 3; the fixed one prints `cursor-check2 (del
fixed) ok`), `probes/py_attack.py` same. The four first-pass Ember probes give the same errors today
(E3063, E3022, E3040, E3021). The first pass's timings were not repeated except A4's base loop (5.7), which
the first pass had taken from the answers report without re-timing.

---

## 3. A2: names are positions

### 3.1 The answer

`boss = enemies[0]` makes `boss` a name for position 0 of `enemies` (rule 1). The compiler keeps the
element's address while nothing can change the list and reads the buffer again after anything that
may (rule 2). A removal or insertion before the name moves it; removing the named element moves its
value into the name (rule 3). Reordering while the name is needed is a compile-time error (rule 4). A
change made by code the compiler cannot see is covered by A4's walkers count (rule 5). A name passed to
a function that may change the list is copied in and written back after the call, and an element that
owns memory is then a compile-time error (rule 6).

### 3.2 Memory attacks

None found that the rules accept. Every use either goes through an address held only across code the
compiler proved cannot change the list, or reads the buffer again, and a position that may be out of
range is bounds-checked; a wrong position gives a wrong element, never freed memory. Two precision
items, both free: "may change the list" must cover every container on the name's path (`grid[i][j]`
when `grid` grows, as the answers report says; also `s.platoons[0].units[0]` when `s.platoons` is
cleared), and every path that may be the same list under A3 (`b.enemies.push(...)` changes `a.enemies`
when `a` may be `b`). Rule 5 rests on A4's walkers count; A4's memory holes 3 and 4 (section 5.2) apply
to names too, and A4's fix covers them.

### 3.3 Readability attacks and saves

**Attack 1: the swap idiom.** Rule 1 makes `tmp` a name (a `Vec2` is not a plain number), so the
third line reads position `i` after the second line replaced it.

| Python | Ember (`attack/probes/x_a2_swap.em`) |
|---|---|
| `tmp = xs[i]` | `tmp = xs[i]` |
| `xs[i] = xs[j]` | `xs[i] = xs[j]` |
| `xs[j] = tmp` | `xs[j] = tmp` |

Python gives `[E2, E1]`; the rules as written give `[E2, E2]` (`attack/a2/a2_models.c`, output in
`a2_models_output.txt`). The same for `old = xs[0]; xs[0] = Enemy(9); print(old.hp)`: Python prints 1
(`py_attack_output.txt`), positions print 9. No error, a wrong answer.

*Save.* Replacing a slot (`xs[k] = v`) while a name of that position is still needed is treated as
removing the element and inserting `v` at the same place, so rule 3 applies: the old value moves into
the name. When the compiler knows `k` is the name's position (the swap), it copies the old value into
the name before the write, which is what C's `tmp = xs[i]` does; when it cannot tell, one compare
(`k == j`) and the copy on the equal path. Plain data is copied; an element that owns memory is moved
(it would otherwise be dropped). Saved model: `[E2, E1]`. Cost: none beyond what the swap itself does.

**Attack 2: an element passed to a method that removes other elements of its list.**

| Python | Ember (`attack/probes/x_a2_hit_purge.em`) |
|---|---|
| `def hit(self, e):` | `fn hit(mut self, mut e: Enemy):` |
| `    e.hp -= 10` | `    e.hp -= 10` |
| `    self.purge()` (removes dead enemies) | `    self.purge()` |
| `w.hit(w.enemies[1])` with `[E0, E50, E70]` | `w.hit(w.enemies[1])` |

Python leaves `[E40, E70]`. Rule 6 copies enemy 50 in, the purge removes enemy 0 so every position moves
down, and the write-back puts the copy at position 1, over enemy 70: `[E50, E40]` (`a2_models.c`). The
hit enemy is unchanged and another enemy is overwritten, with no error. For an element that owns memory
rule 6 refuses the call instead.

*Save: pass a place, not a copy.* When a function may change the list a name points into (or any
container on its path), the parameter is passed as the list and the position; the function is compiled
once more for such callers with that position as a hidden in-out parameter, and its own removals move
it by rule 3's compare and subtract (`j -= (k < j)`). Saved model: `[E40, E70]`. This removes rule 6's
copies and its compile-time error for elements that own memory, and it also fixes A3's `i == j` case
(two places with one position are one element, section 4.4). Cost: a compare and a subtract per removal
inside such a function, the cost rule 3 already pays; one more compiled copy of the function, as the
answers report's A4 rule 8 already does.

**Attack 3: a name held across a visible call that removes.** `boss = w.enemies[2]; w.purge();
boss.hp -= 1` is Python's `[E50, E69]` (`py_attack_output.txt`). The answers report leaves this to
"rule 3 or 4": rule 3 needs the removed position, which the caller does not know after a call, so it is
rule 4's compile-time error. *Save:* the same hidden in-out position: the callee's removals move the
caller's live names.

**Attack 4: a name held across a call the compiler cannot see that removes.** `boss = w.enemies[2];
w.handler.on_tick(w); boss.hp -= 1`, where `on_tick` is virtual and purges. Python runs it. Rule 5
panics. *Save:* the name publishes its position with the list's walking state of A4's fix (section 5.5),
and a removal made by other code through the field moves it as rule 3 does. The arithmetic is the one
`attack/a4/walker_check.c` checks for loops; for names it is not built (UNVERIFIED). Second pass: see 3.6.

**Attack 5: two names, one removed element.** `a = xs[0]; b = xs[0]; xs.pop(0); a.hp = 7; print(b.hp)`
prints 7 in Python (`py_attack_output.txt`). Rule 3 moves the value into "the name's own slot", and only
one name can own it. *Save:* the second name follows the first name's slot (a one-bit flag, set at
compile time when both names come from the same expression, by one compare otherwise). The same for
`x = xs.pop()` of a named element: the name follows `x` instead of being a compile-time error.

### 3.4 Speed

Unchanged: every fix is on a rare path (a replacement, a removal, a call that may change the list while
a name is live), each a compare and a copy or subtract next to the memory move the change itself makes.
Nothing to time.

### 3.5 Verdict (first pass)

**Saved.** What is left: reordering the list (`sort`, `reverse`, `swap`) while a name into it is still
needed is a compile-time error, as in the answers report; using a name whose value was moved into a
variable that was then given away is a compile-time error (Python would share the object; whether
objects can be shared belongs to the objects group).

### 3.6 Second pass

**Attack 4's save is A4's mode for unseen code, applied to a name.** A name held across a call the
compiler cannot see is a walk with one position and no gap: the call publishes the position, other
code's removals before it move it down, and the name reads its position back after the call. The
arithmetic is the special case `w == r` of the mapping `attack/a4/walker_check.c` checks (3,000 random
walks, `walker-check ok`), so it needs no new run-time piece; several names into one list need several
published positions (a short list per walked field, scanned by a removal made by other code). It shares
A4's mode's cost, dependency and verdict (section 5.8). Where that mode is not taken, a struct element's
name used after such a call is a compile-time error with two fix-its (copy it first, or make the element
a class), which is the combined model's rule (`phase3b-combined.md` section 0, item 2).

**Precision item, free.** In the combined model a class object is shared through handles
(`phase3b-combined.md` section 0), so `w1.enemies` and `w2.enemies` are one list when `w1` and `w2` are
one object. A place passed into a function whose summary says it removes from `World.enemies` of some
other handle must compare the two lists by identity (one compare of two pointers) before moving the
position; a removal from another `World` leaves it alone. Without the compare the name would follow the
wrong list's removals: a wrong element, never freed memory (positions stay bounds-checked).

**Verdict after the second pass: saved.** The saved version is the list in section 9.

---

## 4. A3: two names may be one element

### 4.1 The answer

`collide(bodies[i], bodies[j])` is allowed with `i == j` (rule 1). Inside a function, two parameters of
one type, or a `T` and an element of an `Array[T]`, may be the same place (rule 2); a view into a part a
write can replace (a `String`'s characters, an `Array`'s elements, an enum's payload) taken through one
of them, used *after* a write through the other that may replace that part, is a compile-time error.
For two spans the call compares the ranges and calls a `restrict` copy of the function when they do not
overlap (`restrict` tells the C compiler two pointers never overlap).

### 4.2 Memory attack: a view passed into the write

| Python | Ember (`attack/probes/x_a3_extend_view.em`) |
|---|---|
| `def merge(a, b):` | `fn merge(mut a: Squad, b: Squad):` |
| `    a.units.extend(b.units[0:2])` | `    a.units.extend(b.units[0..2])` |
| `merge(squads[0], squads[0])` | `merge(squads[0], squads[0])` |

Rule 1 allows the call. Rule 2 forbids using the view `b.units[0..2]` *after* the write; here it is used
*during* the write, inside `extend`. `extend` is written the usual way (Rust's `extend_from_slice`):
make room, then copy. When `a` and `b` are one squad, making room frees the buffer the view points into.
The model (`attack/a3/extend_attack.c`) prints:

```text
as written (reads freed memory):   [1, 2, 3, -572662307, -572662307]
saved (Python: [1, 2, 3, 1, 2]):   [1, 2, 3, 1, 2]
```

`-572662307` is `0xDDDDDDDD`, the poison of the freed block. Rust refuses its twin
(`v.extend_from_slice(&v[0..2])`: `error[E0502]: cannot borrow *v as mutable because it is also borrowed
as immutable`, `rustc_attack_output.txt`).

*Save.* A call that changes a list and also receives a view that may point into that same list compares
the view's address range with the list's buffer (one unsigned compare) and, when it is inside, copies
the view's elements to a temporary first. That is Python's meaning (`ys.extend(ys[0:2])` gives
`[1, 2, 3, 1, 2]`, `py_attack_output.txt`: Python's slice is a copy) and numpy's rule for overlapping
operands ("results from such operations are now defined to be the same as for equivalent operations
where there is no memory overlap", NumPy 1.13.0 release notes,
https://numpy.org/doc/stable/release/1.13.0-notes.html). It costs the compare when the view is
elsewhere, and a copy of the view's elements when it is inside. (Second pass: this compare is right only
when the call changes one known list; section 4.6 shows where it is not enough.)

Precision items, free: rule 2's list of replaceable parts must be "anything a write can free or
reinterpret": also `Map`, `Set` and `Box` contents. And a parameter that may itself live inside a
replaceable part of another parameter (`f(s.platoons[0].units[0], s)`) is covered by A2's place rule
(section 3.3, attack 2), not by rule 2.

### 4.3 Speed attacks

**The comparison with Rust was unfair.** The answers report's `rust_span.rs` branches on `a < b`, whose
outcome is random; removing that branch (below) takes safe Rust from 2.12x to 1.13x, so the branch was
the cost. That it costs through misprediction is inferred from this and from the answers report's own
first C version of the overlap check, which had a random branch too and took 187 to 190 ms; I did not
count mispredictions (UNVERIFIED). Its C models also take `&pool[a]` with no
bounds check, while Ember's slice `pool[a..a+n]` is bounds-checked and the Rust twins pay that check.
I wrote safe Rust without the branch (`attack/a3/rust_span_fair.rs`: split once at the higher start,
take both slices, choose which is written with a select) and the answer with its bounds checks and the
saved overlap rule of section 4.2 (`attack/a3/span_answer.c`). Span length unknown to the compiler,
`attack/runs/a3_span.txt`:

| program | what it is | median ms | ratio |
|---|---|---|---|
| `span_add.c -DRESTRICT` | two `restrict` spans, no checks (the answers report's baseline) | 90.551 | 1.00 |
| `span_add.c -DCALLCHECK` | the answers report's model of the answer | 97.183 | 1.07 |
| `span_answer.c` | the answer as Ember would emit it: bounds checks, overlap compare, copy on overlap, one compiled copy | 97.784 | **1.08** |
| `rust_span_fair.rs` | safe Rust without the random branch | 102.418 | **1.13** |
| `rust_span.rs` | safe Rust as the answers report wrote it | 192.112 | 2.12 |

So the answer is on par with C (1.08x) and with safe Rust (1.13x); the "2.1x" for Rust is withdrawn.
The 1.08x against 1.13x is under 15% and waits for the final measurer.

**Two list parameters of one type.** Rule 2 makes them possibly the same list, so a loop over one that
pushes to the other must read the walked list's length and buffer again after every push.

| Python | Ember (`attack/a3/alias_lists.c` models it) |
|---|---|
| `def copy_alive(src, dst):` | `fn copy_alive(src: Array[Enemy], mut dst: Array[Enemy]):` |
| `    for e in src:` | `    for e in src:` |
| `        if e.hp > 0: dst.append(e)` | `        if e.hp > 0: dst.push(e)` |

1,000,000 enemies, about 90% alive, 100 calls (`attack/runs/a3_lists.txt`):

| program | median ms | ratio |
|---|---|---|
| headers cannot alias (`-DRESTRICT`, what Rust's `&Vec` and `&mut Vec` give) | 219.764 | 1.00 |
| headers may alias (`-DALIAS`, the answer as written) | 237.358 | 1.08 |
| saved: the call compares the two headers and calls the no-alias copy when they differ (`-DCALLCHECK`) | 216.429 | 0.98 |
| safe Rust (`rust_lists.rs`) | 216.334 | 0.98 |

The 1.08x is under 15% (PROVISIONAL); the saved form removes it either way.

**Code doubling.** The answers report lists "the span check doubles the code of functions that take two
spans of one element type". With section 4.2's copy on overlap, a function that writes one span and only
reads the other is compiled once (the `restrict` copy): on overlap the read span is copied first.
Only two *written* spans that may overlap still need the second copy.

### 4.4 Readability attacks and saves

**Attack 1: the Python claim.** The answers report says C's element-by-element result for overlapping
spans "is also Python's for overlapping slices of one list". It is not: Python list slices are copies,
so `add(xs[0:4], xs[2:6])` leaves `xs` unchanged (`py_attack_output.txt`). Python's `memoryview` slices
are views and give C's result (`[0.0, 1.0, 1.0, 2.5, 3.0, 5.0]`, same file); numpy gives "as if copied".
The saved rule of section 4.2 gives numpy's result when one span is only read; two written spans that
overlap keep C's (and `memoryview`'s) result.

**Attack 2: a compile-time error Rust does not have.** Rule 2 is checked inside every function with two
parameters of one type, whether or not any caller passes one element twice:

```ember
fn rename_both(mut a: Unit, mut b: Unit) -> int:
    old = a.name.as_str()       # a view of a's characters
    b.name = "renamed"          # may replace them, if a and b are one Unit
    return old.len()            # rule 2: compile-time error
```

Its Rust twin compiles (`x_rename_both.rs`, exit 0 in `rustc_attack_output.txt`: two `&mut` never alias),
and the Python version runs. *Save:* compile such a function twice. Calls whose arguments are proven
different, or checked different at the call (`i != j`; the answers report measured that compare as free:
89.9 ms with it, 94.9 ms without `restrict`, its `runs/a3.txt`), use the no-alias copy, where rule 2 has
nothing to check. In the alias copy, a write that replaces a heap part (a `String`'s characters, an
`Array`'s elements, a `Map`, `Set` or `Box`'s contents) that a view still uses keeps the old buffer alive
until the view's last use, instead of an error: the rule the answers report already uses for A25 ("keeps
the old buffer alive until the words' last use"), and Python's meaning (`old` keeps the old string).
Cost: one compare at such calls; on the equal path, a free moved to the view's last use. A view into an
enum payload stored inline (`a.shape` is `Circle(r)`, `b.shape = Square(s)`) cannot be kept alive that
way, so that one stays a compile-time error, now reported at a call that may pass one element twice
instead of at every function. (Second pass: "keeps the old buffer alive" is not enough when the write
removes elements in place; section 4.6.)

**Attack 3: one element twice to a function that may change the list.** A2's rule 6 copies both in and
writes both back, so with `i == j` the last write-back wins: neither Python's result nor an error. A2's
save (places, section 3.3) fixes it: two places with one position are one element.

### 4.5 Verdict (first pass)

**Saved.** What is left: a view into an inline enum payload, kept across a write through another name
that may be the same element, is a compile-time error at the call that may pass one element twice; two
written spans that overlap get C's element-by-element result.

### 4.6 Second pass: two more memory holes in the saved rules

**Hole 1: a slice passed with the list of squads to a function that grows every squad's list.**

| Python | Ember (`attack2/probes/x_reinforce.em`) |
|---|---|
| `def reinforce(squads, picked):` | `fn reinforce(mut squads: Array[Squad], picked: Span[int]):` |
| `    for s in squads:` | `    for s in squads:` |
| `        s.units.extend(picked)` | `        s.units.extend(picked)` |
| `reinforce(squads, squads[0].units[0:2])` | `reinforce(squads, squads[0].units[0..2])` |

Python gives `[[1, 2, 3, 1, 2], [4, 1, 2]]` (`attack2/probes/py_attack2_output.txt`, check 4). No rule of
the first pass refuses it: the two parameters have different types (rule 2 is about two parameters of
one type, or a `T` and an element of an `Array[T]`), and the saved rule 4 ("a call that changes a list
and receives a view that may point into it compares the view's range with the buffer") runs inside each
`extend`. For `s = squads[0]` the view is inside, so it is copied and that `extend` is right; but that
`extend` moves `squads[0].units` to a bigger buffer and frees the old one, and for `s = squads[1]` the
view is not inside `squads[1]`'s buffer, so it is read from freed memory. `attack2/a3/view_attacks.c`
(poisoning allocator), output `view_attacks_output.txt`:

```text
demo R: reinforce(squads, squads[0].units[0..2]); Python [[1, 2, 3, 1, 2], [4, 1, 2]]
  written: [[1, 2, 3, 1, 2], [4, -572662307, -572662307]]
  saved:   [[1, 2, 3, 1, 2], [4, 1, 2]]
```

Rust refuses the twin (`attack2/probes/rust/x_reinforce.rs`: `error[E0502]: cannot borrow *squads as
immutable because it is also borrowed as mutable`); today's Ember refuses it too (`E3021`,
`ember_check_attack2_output.txt`).

*Save.* The question is decided at the outer call, where the compiler knows both sides: the view's source
path (`squads[0].units`) and the callee's effect summary (`reinforce` may grow `squads[*].units`). When the
source may be one of the lists the callee may grow, remove from, reorder or replace, the call passes a copy
of the view's elements (Python's slice is a copy); when it cannot be (the view comes from a local list, or
from a list of another type, or the summary says the callee changes nothing of that type), the view is
passed as it is. Inside the callee, a view parameter is then never invalidated by the callee's own
changes, which is what the first pass's no-alias reasoning assumed. The range compare of 4.2 stays for the
common case where the callee changes one known list (`a.units.extend(b.units[0..2])`), where it is
cheaper than a copy. Cost: a copy of the view's elements, only at calls whose summaries may change the
view's source; Python pays that copy at every slice.

**Hole 2: a slice kept across `clear()` through the other name, in the alias copy.**

| Python | Ember (`attack2/probes/x_regroup_clear.em`) |
|---|---|
| `def regroup(a, b):` | `fn regroup(mut a: Squad, b: Squad) -> int:` |
| `    first_two = b.units[0:2]` | `    first_two = b.units[0..2]` |
| `    a.units.clear()` | `    a.units.clear()` |
| `    return first_two[0].name` | `    return first_two[0].name.len()` |
| `regroup(s, s)` | `regroup(s, s)` |

Python returns `['ann', 'bob']` (check 5). The first pass's saved rule 3 ("in the alias copy, a write that
replaces a heap part another view still uses keeps the old buffer alive until the view's last use") keeps
nothing that matters here: `clear()` does not replace the buffer, it drops the elements inside it, so the
units' names are freed while the view still shows them. Same program:

```text
demo C: regroup(s, s): a view kept across a.units.clear(); Python ['ann', 'bob']
  written: first_two names: <freed>, <freed>   (reads of freed memory: 2)
  saved:   first_two names: ann, bob   (reads of freed memory: 0)
```

Rust refuses the twin (`x_regroup.rs`: `error[E0502]`); today's Ember refuses it (`E3021`).

*Save.* In the alias copy, a view that is still used after a write that may remove, replace or move the
elements it views is copied just before that write (its elements cloned: Python's slice is a copy). The
"keep the old buffer alive" rule stays for what it is right for: a view of a `String`'s characters across
a replacement of the whole string (`old = a.name.as_str(); b.name = "renamed"`), where Python's `old` is
the old string object and keeping its buffer costs nothing. Cost: the copy, only in the alias copy, that
is, only when the call really passed one squad twice.

*Attack on the saved versions.* A view stored somewhere that outlives the call (a field of a struct the
callee returns) is a references-group question (A6); its lifetime is checked as any view's, so it cannot
outlive the copy. A callee that changes the view's source through a third parameter (`reinforce(squads,
view, more_squads)`) is covered by the same summary test at the call (the summary names every path the
callee may change). Two views into one list passed to a callee that changes it are each tested. A view
whose source the caller cannot name (it is itself a parameter, `fn outer(squads, v: Span[int]):
reinforce(squads, v)`) counts as possibly any list of its element type the callee changes, so it is
copied, unless `outer` was itself reached through its no-alias copy; that is a copy Python makes anyway,
but it is the one place where the saved rule copies more often than needed. I found no program that gets a
view past the saved test.

**Verdict after the second pass: saved.** What is left is unchanged from 4.5.

---

## 5. A4: the loop becomes a one-pass filter

### 5.1 The answer

A loop that changes its own list is compiled as an in-place filter with a read position `r` and a write
position `w`, to the live length (rule 1); `e` is the element at `r` (rule 2); `enemies.remove(e)` drops
the current element without skipping the next (rule 3); other removals are mapped (rule 4); inserting,
sorting or reversing inside the loop is a compile-time error (rule 5); removed elements are dropped at
the loop's end (rule 7); a visible method that removes is compiled into the loop (rule 8). For code the
compiler cannot see (rule 9), the list carries a **walkers count** in the top byte of its capacity word;
while it is above zero, removing, inserting, reordering, replacing and dropping the list panic, and
adding is allowed. Rule 10: after such a call the loop re-checks `r < len` and `w <= r`, and for
elements that own memory the gap is filled with the removed elements (swap-fill), so "memory safety must
not depend on the count". The model keeps `e` in CPU registers (`ember.c`, `ember_swapfill.c`).

### 5.2 Memory attacks

`attack/a4/attacks.c` runs each Ember loop below, compiled as rules 7, 9 and 10 say, over units that
own a `String` name, with the poisoning allocator; output in `attack/a4/attacks_output.txt`:

| # | Ember loop (unseen code in comments) | As written | Saved |
|---|---|---|---|
| 1 | `for u in units:` / `u.name = "renamed"` / `if u.hp <= 0: units.remove(u)` / `on_step()` | 2 double frees | 0 |
| 2 | `for u in w.units:` / `u.hp -= 1` / `w.on_step()` (renames every unit) / `print(u.name)` | 3 double frees, 3 reads of freed memory | 0 |
| 3 | the same loop as the 256th loop over one list (a recursive event dispatcher) whose callback does `self.units.remove(0)` | 1 double free | panic "more than 255 loops over one list at once" |
| 4 | `for u in w.units:` / `w.on_step()` (does `mem.swap(self.units, self.reserve)`) | 1 double free, 1 element owned by both lists; afterwards the list in `units` reads 255 walkers and the one in `reserve` reads 1 | panic "write access to World.units while a loop walks it" |

What goes wrong, in plain words:

1. The element is held in registers and changed there. A removed element is "not stored", so its slot
   keeps the bits from *before* the body ran, including the name the body already freed; rule 7 drops
   it again at the loop's end. *Fix:* on the removal path, store the element back to its slot.
2. The callback writes the current element through the list (allowed: rule 9 forbids only structural
   changes). The copy in registers still holds the old name, which the callback freed; the loop prints
   it and stores it back. *Fix:* spill the element to its slot before a call the compiler cannot see and
   reload it after.
3. The count is 8 bits; the 256th loop wraps it to 0, a removal is allowed, and rule 10's two re-checks
   both pass (`r = 2 < len = 4`, `w = 1 <= 2`), yet one element ends up in two slots. So memory safety
   *does* depend on the count, and the count must never be wrong. *Fix:* an overflow check (part of the
   final fix below).
4. Moving the list out of its field is not an operation on the list, so the count inside the list's
   header never sees it; the loop goes on over the other list at the same place. *Fix:* keep the walking
   state in the field's access word, which today's Ember already checks for every long-term access to a
   field, assignment and `mem.swap` included (`[EXC-1]`, `[EXC-16]`, `[EXC-19]`).

Today's Ember is open to none of these: the removal in the loop is a compile-time error (E3020), and a
loop over `h.items` holds a read access, so the callback's write panics (`[EXC-1]`). The answer's rule 9
relaxes that so callbacks may add; the relaxation has to live in the access word, not beside it.

### 5.3 Speed attacks

**A body that reads the list by position.** The model keeps `e` in registers because "nothing but `e`
touches `enemies[r]`". A body that reads `enemies[k]` may touch it, so by the answer's own condition `e`
is the element in its slot: the body changes the slot and the step ends by copying the slot down, the
form the answers report's section 1.4 measured at 4.7x. Workload: the answers report's A4 loop plus
`lead += enemies[0].hp` each step (`attack/a4/`, `attack/runs/a4_read.txt`):

| program | what it is | median ms | ratio |
|---|---|---|---|
| `plain_read.c` | plain C, element in registers | 129.306 | 1.00 |
| `ember_read_naive.c` | the answer as written (element in its slot, copied down) | 459.255 | **3.55** |
| `ember_read_wb.c` | first save: store `e` back to its slot before each read by position | 184.075 | 1.42 |
| `ember_read_sel.c` | **saved**: a field read through the list compares the mapped position with `r` and takes the field from `e` | 131.221 | **1.01** |

The write-back is kept only for code that takes the whole element by reference.

**The mode for code the compiler cannot see.** The answers report measured that mode at 1.16x with
`ember_swapfill.c`, which has no call in it and keeps `e` in registers where the call would be (unsafe:
section 5.2, attack 2). With a real call through a function pointer every step (the callback reads the
list's length), 101,705,840 calls in the run (`attack/a4/count_calls_output.txt`), `attack/runs/a4_cb_2.txt`:

| program | what it is | median ms | ratio |
|---|---|---|---|
| `plain_cb.c` | plain C, no checks; the callback sees the physical length | 170.040 | 1.00 |
| `plain_cb_logical.c` | plain C telling the callback `r` and `w`, so it sees Python's length | 182.994 | 1.08 |
| `ember_cb_report.c` | the answers report's model plus the call, the count and the re-checks (unsafe) | 184.961 | 1.09 |
| `ember_cb_saved2.c` | memory holes fixed: spill and reload, store back on removal, access word | 216.493 | 1.27 |
| `ember_cb_walker.c` | **saved**, final: also Python's meaning for other code (section 5.5), no swap-fill | 235.312 | 1.38 (1.29 of `plain_cb_logical`) |

A second run (`attack/runs/a4_cb_3.txt`, noisier: plain C 212.755 ms) gave 1.02, 1.00, 1.13, 1.22 (1.20 of
`plain_cb_logical`), and 1.23 for `ember_cb_walker2.c` (reload only when other code wrote the element).
Divided per call: making the mode safe costs 0.31 ns per call ((216.493 - 184.961) ms / 101,705,840 in
`a4_cb_2`; 0.27 ns in `a4_cb_3`), and the final form 0.51 ns against plain C doing the same visible work
(0.42 ns in `a4_cb_3`). The callback here does nothing; a real one does more, so this is the worst
case. Safe Rust cannot express this program (a callback cannot reach a list while it is being walked);
its idiom for the callback-free loop took 1.62x (answers report, `runs/a4.txt`). (Second pass, 5.7: plain
C written to be correct for a callback it does not know pays this same 0.5 ns.)

My first saved version (`ember_cb_saved.c`, `attack/runs/a4_cb.txt`) took 472.530 ms, 2.42x: clang turned the
spill into in-place 4-byte stores (`movss %xmm0, (%rax,%r13)` for `x`) and merged two 4-byte reloads
after the call into one 8-byte load (`movsd (%rax,%r13), %xmm7`, `x` and `vx`), which waits for the
4-byte store: the store-forwarding stall again. Reloading at the widths of the stores (`ember_cb_saved2.c`)
fixed it. The emitted code must keep that pairing.

The walkers count's only hot-path cost in the answers report was masking it off in `push` (+1.4%); with
the state in the access word, `push` is unchanged.

### 5.4 Readability attacks

1. **Other code sees removed enemies.** The model sets the list's length only at the loop's end
   (`l.len = w`), so a callback reading `len(w.enemies)` while the loop runs sees the old length and the
   removed elements. Python (with the loop over a copy, the habit) sees `[3, 3, 2, 2]` on
   `[E0, E5, E0, E5]` (`py_attack_output.txt`); the model's callback would see 4 every step.
2. **A virtual `update` that removes its own enemy panics.** The most common shape of Python game code:

   | Python | Ember |
   |---|---|
   | `class Grunt(Enemy):` | `class Grunt(Enemy):` |
   | `    def update(self, world):` | `    override fn update(self, world: World):` |
   | `        self.hp -= 1` | `        self.hp -= 1` |
   | `        if self.hp <= 0: world.enemies.remove(self)` | `        if self.hp <= 0: world.remove_enemy(self)` |
   | `for e in self.enemies: e.update(self)` | `for e in self.enemies: e.update(self)` |

   `e.update` is virtual, so rule 9 panics at the removal. Python runs it but skips the enemy after each
   removed one: on `[G1, G1, G3]` it leaves `[E1, E2]` (`py_attack_output.txt`); the second grunt is
   never updated. Under the saved rule below every enemy is updated once and `[E2]` remains (worked by
   hand from the rule, which `walker_check.c` checks in general).
3. **`len(enemies)` inside the body** is given as `w + (len - r)`; after `enemies.remove(e)` it must be
   one less, since the current element is no longer in the list. Fixed in the mapping below.

### 5.5 The save (first pass)

Rules 1 to 8 stay, with two changes: a field read through the list by position uses a compare and a
select (section 5.3), and a removed element is dropped at the end of its step. Rules 9 and 10 are
replaced:

- Other code can reach a list a loop is walking only through a class field (or another place that has
  its own access state, such as a `Shared`): a local list cannot be reached at all. So the walking state
  is a kind of access in that field's access word (`[EXC-19]`), set when the loop starts and cleared
  when it ends. It allows reads, writes of elements and appends; assigning, moving (`mem.swap`,
  `mem.take`), dropping or reordering the list, holding a view of it, and starting a second removing loop
  over it panic. Its count is checked for overflow. (Second pass, 5.7: a list inside a list needs the path.)
- Before each call the compiler cannot see, the loop publishes `r`, `w` and a flags word in its frame and
  spills `e`; after it, it reads back `w` and the flags (other code's removals never move `r`), and
  reloads `e` at the spill's widths when the flags say other code wrote it.
- While the field is walked, other code's accesses through it see the logical list: its length is
  `len - (r - w)`, less one if the current element was removed; position `k` is `k` if `k < w`, else
  `k + (r - w)` (plus one past a removed current element); removing before `w` closes the gap from the
  front, removing the current element sets a flag, removing ahead shifts the tail; appending appends.
  Because no other code can see the gap, it needs no swap-fill.

`attack/a4/walker_check.c` checks this mapping against a plain reference list with Python's meaning
(minus its skip) over 3,000 random walks in which "other code" removes up to two elements per step (the
current one sometimes), appends and reads; after every operation the two lists must be equal, and so
must the visit order and the final list. It prints `walker-check ok`. Each of three bugs I put in by
hand (the mapping without the removed current element, removing before `w` without moving `w`, removing
the current element as an ordinary removal) made it fail.

*Attack on the saved version.* Demos 1 to 4 pass (table in 5.2). A view of the walked field held by
other code panics instead of reading the gap. A second walker on the same list started by other code
panics (a nested reading loop through the field goes through the mapping instead). Insert by other code
panics. Memory safety rests on the mapping, a fixed piece of the runtime, as it
rests on the bounds check; the self-check above is its test.

### 5.6 Verdict (first pass)

**Saved.** Cost: 1.01x for a body that reads the list by position; 0.3 to 0.5 ns per call the compiler
cannot see in loops that change the list they walk (1.20x to 1.29x of plain C in the worst case of a
callback that does nothing, PROVISIONAL; above 15%, so it is a real cost, the price of letting unseen
code change the list safely, which C does unsafely and Rust refuses). What is left: inserting, sorting
or reversing the walked list (visible code: compile-time error; other code: panic); a view of a walked
list held by other code panics. (Second pass, 5.7 and 5.8: the 1.20x to 1.29x is measured against C that
knows its callback; against C written for a callback it does not know, the mode costs 1.00x to 1.02x.)

### 5.7 Second pass

**Attack 1 (readability): the Python habit `for e in xs[:]` is refused, or loses every write.** A Python
programmer who knows that removing from a list while walking it skips elements walks a copy:

| Python | Ember (`attack2/probes/x_copy_loop.em`) |
|---|---|
| `for e in es[:]:` | `for e in enemies[..]:` |
| `    e.hp -= 1` | `    e.hp -= 1` |
| `    if e.hp <= 0:` | `    if e.hp <= 0:` |
| `        es.remove(e)` | `        enemies.remove(e)` |
| `        es.append(E(9))` | `        enemies.push(Enemy(hp=9))` |

Python visits the four original enemies only and leaves `[E2, E4, E9, E9]` (check 1 of
`py_attack2_output.txt`). In Ember `enemies[..]` is a view of the list, not a copy, and the answer's rules
cover a loop over the list itself, not over a view of it, so a loop over the view while the body changes
the list is the view-across-a-change case: refused (memory-safe, but Python's program fails; today:
`E3020 cannot mutate *enemies while it is borrowed by this loop`, `x_copy_loop_view.em`). If instead the
slice were made a copy, as Python's is, the copy would hold copies of the struct elements, so `e.hp -= 1`
would change the copy and every write would be lost (check 1b: the list stays `[E1, E3, E1, E5]`). The
same for `enemies.clone()` (`x_copy_loop_clone.em`). Rust refuses the twin (`x_copy_loop.rs`: `E0499`).

*Save, free.* A `for` loop over a copy or a whole slice of the list its body changes (`xs[..]`, `xs[a..]`,
`xs.clone()`, `Array(xs)`) is compiled as the walk itself, with the copy's bounds: it visits only the
elements that were there when it started (it ends at that original end, which moves down when the body
removes one of the original elements ahead of it), and `e` names the element in the list, so writes
reach the list, as in Python, where the copy and the list hold the same objects. No copy is made. The
Python model of the walk with that bound gives `[E2, E4, E9, E9]`, Python's result (check 1d); the walk
without it visits the appended enemies too, `[E2, E4, E8, E8]` (check 1c). Cost: the loop's test is `r <
end` instead of `r < len`, the same instructions as plain C written for the same meaning. One difference
from Python is left: an original element removed by the body before its turn is not visited (Python
visits it from the copy, after it has left the list).

**Attack 2 (readability, a wrong element removed): `enumerate` and `del xs[i]`.**

| Python | Ember (the answers report's own probe `containers/ember/a4_closest.em` uses this shape) |
|---|---|
| `for i, e in enumerate(es):` | `for i, e in enumerate(enemies):` |
| `    e.hp -= 1` | `    e.hp -= 1` |
| `    if e.hp <= 0: del es[i]` | `    if e.hp <= 0: enemies.remove(i)` |

On `[E1, E1, E5, E1]` Python leaves `[E1, E4]` (its skip: the second enemy is never updated; check 2). The
answer does not say what `i` is; if it is the number of steps so far, as `enumerate` counts, then after the
first removal `i` is one more than `e`'s position in the logical list, rule 4 maps `remove(i)` to the
element after `e`, and the walk removes the live `E5` instead of the dead enemy (which stays in the list),
then panics with "index out of bounds" at the next dead enemy (check 2, in the Python model of the walk). *Save, free:* in a loop that changes
its own list, `enumerate`'s index is `e`'s position in the list now, which is `w` (already in a register).
That is what Python's `i` is too: Python fetches `e` at its iterator position `i`, which is `e`'s position
at that moment. The model then leaves `[E4]`: every dead enemy removed, none skipped (check 2).

**Attack 3 (readability, a lost write): writing the current element through the list by position.** The
first pass's save compares the mapped position with `r` for reads only. A write through the list goes to
the slot, and the end of the step stores `e` from registers over it:

| Python | Ember |
|---|---|
| `for e in es:` | `for e in enemies:` |
| `    e.hp -= 1` | `    e.hp -= 1` |
| `    es[0].hp += 10` | `    enemies[0].hp += 10` |

Python gives `[E20, E4]`; the first pass's rules give `[E10, E4]` (check 3). *Save:* a write through the
list compares the mapped position with `r` too and writes `e` itself when they are equal. Timed on the
answers report's workload plus `enemies[0].x += 1.0` each step (`attack2/a4/`, `attack2/runs/a4_write.txt`):

| program | what it is | median ms | ratio | checksum |
|---|---|---|---|---|
| `plain_write.c` | plain C, element in registers, writes the register copy while `w == 0` | 226.098 | 1.00 | 159635288 |
| `ember_write_sel.c` | **saved**: the write compares the mapped position with `r` | 221.462 | **0.98** | 159635288 |
| `ember_write_lost.c` | the first pass's rules (one run, not timed) | | | 159635284 (wrong) |

The saved form emits the same test plain C makes by hand (`slot == r` is `w == 0` for position 0).

**Attack 4 (memory): a list inside a list, in the mode for unseen code.** The first pass's save keeps the
walking state in "that field's access word". When the walked list is a field of a struct element of a
class field, the only class field on the path is the outer list:

| Python | Ember (`attack2/probes/x_nested_walk.em`) |
|---|---|
| `for u in self.squads[0].units:` | `for u in self.squads[0].units:` |
| `    u.hp -= 1` | `    u.hp -= 1` |
| `    if u.hp <= 0: self.squads[0].units.remove(u)` | `    if u.hp <= 0: self.squads[0].units.remove(u)` |
| `    self.on_step(self)` | `    self.on_step(self)`  (a stored function value) |

The walking access on `squads` allows "writes of elements and appends", so two callbacks pass it:
`self.squads[0] = Squad(units=[Unit(7)])` (a write of an element of `squads`) drops the walked list, whose
removed elements were already dropped at the end of their steps; `self.squads.push(Squad(units=[]))` moves
the `Squad` that holds the walked list's header, and the loop, which keeps the header's address as the
first pass's model does for a class field, reads it from freed memory. `attack2/a4/nested_attack.c`
(poisoning allocator), output `nested_attack_output.txt`:

```text
demo (i): the callback replaces squads[0], which holds the walked list
    panic: list changed under the loop
  written: double frees 2, reads of freed memory 0
    panic: squads[0] holds a list a loop is walking
  saved:   double frees 0, reads of freed memory 0
demo (ii): the callback appends to squads, moving the walked list's header
    the loop read its list's header from freed memory
  written: double frees 0, reads of freed memory 1
    after the loop: squads[0].units has 1 units: hp 2
  saved:   double frees 0, reads of freed memory 0
```

In (i) the re-check of rule 10 does panic, but only after the two double frees. Python goes on over the
old list object (check 7). *Save:* the walk publishes its **path** (field `squads`, position 0), not only
the field; while it runs, replacing, removing, inserting before, reordering or dropping any value on the
path panics, appending to an outer list is allowed, and the loop finds its list again through the path
after each unseen call (it never keeps the header's address across one, which is A2's rule applied to the
loop's own list). Cost: on the walk, the path re-evaluation after an unseen call (one or two loads); on
other code, the test it already makes on that field's state, with the published position compared only on
operations that remove, insert, replace or reorder.

**Attack 5 (speed): what plain C does for a callback it does not know.** The first pass compared the mode
with plain C whose callback is known to read only the list's length, so that C keeps the element in
registers across the call. A C programmer who passes the list to a callback that may read or write the
current element, or remove elements, must store the element before the call, read the positions back and
reload the element after it, exactly the protocol of the saved mode minus its checks. I wrote that C
(`attack2/a4/plain_cb_any.c`) and timed it beside the first pass's two programs, twice (101,705,840 calls
per run):

| program | what it is | `a4_cb.txt` median ms | ratio | `a4_cb_2.txt` median ms | ratio |
|---|---|---|---|---|---|
| `plain_cb_logical.c` | plain C that knows its callback only reads the length (the first pass's baseline) | 216.207 | 1.00 | 163.643 | 1.00 |
| `plain_cb_any.c` | plain C correct for any callback the Ember program allows, no checks | 267.286 | 1.24 | 218.263 | 1.33 |
| `ember_cb_walker.c` | the first pass's saved mode (state in the access word, two re-check panics) | 272.722 | 1.26 (**1.02 of `plain_cb_any`**) | 218.509 | 1.34 (**1.00 of `plain_cb_any`**) |

(`attack2/runs/`, summaries `a4_cb_summary.txt`, `a4_cb_2_summary.txt`; the first run's spread was wide,
192.9 to 229.2 ms for `plain_cb_logical`.) Per call, not knowing the callback costs 0.50 ns and 0.54 ns
((267.286 - 216.207) ms / 101,705,840 and (218.263 - 163.643) ms / 101,705,840), in C and in Ember
alike; the mode's own checks add 0.05 ns and 0.002 ns. So the mode is on par with C doing the same work;
the 1.20x to 1.29x of 5.6 is the price of not knowing what the callback does, which C pays too.

**Re-timing the base loop** (the claim "0.98x" rested on the answers report's single file):
`attack2/runs/a4_base.txt`, the answers report's `plain.c` 88.651 ms and `ember.c` 85.817 ms, **0.97x**.
The two programs emit the same loop (they differ only in how the removal is written in C).

### 5.8 Verdict after the second pass

**Saved.** The one-pass filter, with: writes and reads through the list by position compared with `r`
(0.98x and 1.01x); `enumerate`'s index is `e`'s current position; a loop over a copy or slice of the list it
changes is the same walk with the copy's bounds; and, for code the compiler cannot see, the saved mode with
the path published (1.00x to 1.02x of plain C written for an unknown callback).

**What the mode for unseen code needs, said plainly.** It is a run-time mechanism: other code that may run
during a walk tests a walking state on every access to a list held in a class field. Within a loop that
contains no unseen call nothing can start or end a walk, so that test is hoisted out of the loop; it is one
load and one predictable branch per such access sequence (argued from the code; its cost on a whole
program is UNMEASURED). The first pass put the state in today's per-field access words. The combined model
(`phase3b-combined.md` section 4.5) removed those words with the ownership group's lean class, and chose
Rust's rule for this one case instead: a loop that both removes from its list and calls unseen code that
may change that list is a compile-time error (fix-its: collect the removals and apply them after the loop,
or state the callee's effects). So there are two complete answers for this case, and the owner picks:

| | the run-time mode (this file) | Rust's compile-time rule (combined model) |
|---|---|---|
| Python's program | runs, without Python's skip | refused, with fix-its |
| speed of the loop | 1.00x to 1.02x of C written for an unknown callback; 1.26x to 1.34x of C that knows its callback (all PROVISIONAL) | the fixed program's speed (Rust's idiom for the callback-free loop: 1.62x, answers report) |
| what every other program pays | a hoisted state test on accesses to list fields in code that may run during a walk; a home for the state (a per-field word, or one bit in the object's count word plus a per-thread record of walks, the cheaper home: UNVERIFIED that bit 27 of the count word can be spared, `phase3b-combined.md` 3.4) | nothing |
| memory safety | as Rust's, by the mapping (`walker_check.c`) and the path rule (this section) | as Rust's |

The same choice decides A2's names held across unseen calls (section 3.6).

---

## 6. A25: lists of string views

### 6.1 The answers

A local list of views carries the lifetime of its text, inferred (the local answer). A returned list
whose views all point into one local text takes that text along in a hidden owner field (the returned
answer); every list of views gets a 24-byte owner field in its header.

### 6.2 Memory attacks

None found in the answers as written. Views are checked by lifetimes (Rust's rule, inferred) and a taken
text is owned privately by the list. One dependency on the objects group: a `String` field of a class
object changed by code the compiler cannot see while local words view it is caught today by the access
word (`[EXC-1]` panics, which is safe); Python would rebind the field. That residue is theirs. (Second pass,
6.6: one hole in the first pass's saved take-along rule.)

### 6.3 Readability attack: the usual file loop

| Python | Ember (`attack/probes/x_a25_lines.em`) |
|---|---|
| `all_words = []` | `all_words: Array[str] = []` |
| `for line in text.split("\n"):` | `for line in text.split("\n"):` |
| `    all_words.extend(line.lower().split())` | `    low = line.to_lower()` |
| | `    for w in low.split_whitespace(): all_words.push(w)` |

Each lowered line is a temporary that dies at the end of its iteration while `all_words` lives on: views
of many texts with different lifetimes. The answers report lists it as what is left ("cannot escape both
lifetimes without copies"); its fix-it is an owned `String` per word. Today's checker: `E3063`.

*Save: the list takes each text along.* When views of a text that dies are stored in a list that lives
longer, the compiler moves the text into the list's owner block (one pointer appended per text, not per
word); the block is freed with the list. Workload `attack/a25/lines.h`: the answers report's 32 MiB text
with a newline after every 72nd-or-later column, each line lowered into its own buffer, split, appended,
then every word hashed; 5 passes (`attack/runs/a25_lines.txt`):

| program | words are | median ms | ratio | peak KB | ratio |
|---|---|---|---|---|---|
| `lines_takealong.c` | **saved**: views, lines taken along | 1011.185 | 1.00 | 208,620 | 1.00 |
| `rust_lines.rs --cfg twophase` | safe Rust's workaround: collect the lowered lines first, then `Vec<&str>` into them | 1139.865 | 1.13 | 219,528 | 1.05 |
| `lines_owned_sso.c` | owned, short-string layout | 1128.997 | 1.12 | 299,432 | 1.44 |
| `lines_owned_malloc.c` | owned, one allocation per word (the fix-it as written) | 1790.752 | 1.77 | 321,980 | 1.54 |
| `rust_lines.rs` | safe Rust `Vec<String>` | 2197.118 | 2.17 | 322,252 | 1.54 |

The spread in this file is wide (the saved program ran 937.7 to 1254.6 ms), so 1.00 against 1.13 waits
for the final measurer; against one allocation per word (1.77x) the difference is clear.

*Attack on the save: a list that is shrunk keeps its texts.* A rolling window (`recent.extend(
line.split()); del recent[:500]`) would keep every line it ever took, without bound, while Python frees
each word. *Fix:* take texts along only into a list the program never shrinks while it holds taken texts
(the effect summaries the answers report already needs say whether a function may remove from a list);
`clear()` frees the block. Otherwise the owned fix-it stays, and that is what is left.

### 6.4 The owner field

A 24-byte owner field in every list-of-views header doubles it (24 bytes to 48), for every `Array[str]`,
`Array[Span[T]]` and list of structs holding views, taken or not: for 1,000,000 inner lists in an
`Array[Array[str]]` that is 24,000,000 more bytes (arithmetic, not measured). *Save:* an 8-byte pointer
to the owner block, null when the list took nothing (8,000,000 bytes in the same example). The owner
block holds the taken buffers; its first use above needs exactly that.

*Copying.* If the objects group makes `b = a` copy lists, a copy of a list that took texts along must not
point into the original's texts after the original is dropped. *Fix:* the owner block carries a count
(only these blocks; plain non-atomic, since a list is moved, frozen or locked when another thread can
reach it, the answers report's section 7 item 7). This depends on that group's decision. (Second pass,
6.6: "frozen" is not enough for a plain count.)

### 6.5 Verdict (first pass)

Both **saved**. What is left: a list of views that the program shrinks cannot take texts along, so it
needs owned copies (1.12x with the short-string layout, 1.77x without); a single word returned from a
function that made its text is returned as `String` (unchanged).

### 6.6 Second pass

**Hole: a word kept across `clear()` of a list that took its lines along.**

| Python | Ember (`attack2/probes/x_best_clear.em`) |
|---|---|
| `words = []` | `words: Array[str] = []` |
| `for line in text.split("\n"):` | `for line in text.split("\n"):` |
| `    words.extend(line.lower().split())` | `    low = line.to_lower()` |
| | `    for w in low.split_whitespace(): words.push(w)` |
| `best = words[0]` | `best = words[0]` |
| `words.clear()` | `words.clear()` |
| `print(best)` | `println(best)` |

Python prints `alpha` (check 6). In Ember, `best` is a name for `words[0]` (A2 rule 1: a `str` is not a
plain number); `clear()` removes the named element, so A2 rule 3 moves its value, a view, into `best`; and
the same `clear()` frees the owner block holding the line that view points into (the first pass's rule).
`attack2/a25/best_clear.c` (poisoning allocator), output `best_clear_output.txt`:

```text
best = words[0]; words.clear(); print(best)   (Python prints alpha)
  written: print(best) reads FREED memory: <freed>
  saved:   print(best) reads live memory: alpha
```

*Save.* A view read out of a list that took texts along lives no longer than the block it points into.
When such a view is still used after `clear()`, the compiler moves the block's free from `clear()` to that
view's last use: the same keep-alive the answers report uses for a reassigned text (5.3 (a)). When the view
is used across a loop that clears and refills the list again, those blocks would pile up without bound, so
that one is a compile-time error with the fix-it `best = words[0].to_string()` (one allocation, as
Python's own `str`). Both decided when compiling; cost: none at run time (a free moved later).

**Precision item: the copy count and threads.** A "frozen" list (shared with threads and never written
after) can be copied by two threads at once; each copy increments the owner block's count, a plain write to
shared memory, so increments can be lost and the block freed while a copy still points into it. This is
the count race the copy-on-write values design's attacker found (`phase3-break-D1.md` B3, as quoted in
`phase3b-threads.md` line 506). *Fix:* the owner block takes the threads group's shared flag: its count is
atomic only when the block is reachable from an object that crosses threads (the flag test the class
counts already make, `phase3b-combined.md` 3.4). No model built; the mechanism is the threads group's.

**Verdict after the second pass: both saved.** What is left is unchanged from 6.5, plus: a word from a
take-along list kept across a loop that clears that list is a compile-time error with a `to_string()`
fix-it.

---

## 7. ECS: creating and destroying entities inside a view loop

### 7.1 The answer

The view loop is library code. It walks the smaller pool backwards by position; `pos` and `vel` are
names keyed by the entity; the loop keeps the pools' arrays in locals and reads them again after the
rare branch; before a call that may destroy, it stores its position in the pool's `cursor`, and
`destroy` of an entity the loop has not reached yet moves the last unreached entity into its slot and
then removes that slot as a visited one (rule 4), so every entity is visited once.

### 7.2 Correctness attack: two destroys in one step

An explosion destroys the current entity and its neighbours:

| Python (`for e in list(vel):`) | Ember |
|---|---|
| `if hit(e):` | `if hit(e):` |
| `    for n in near(e): del pos[n]; del vel[n]` | `    for n in near(e): reg.destroy(n)` |
| `    del pos[e]; del vel[e]` | `    reg.destroy(e)` |

The answers report's self-check (`containers/bench/ecs/cursor_check.c`) lets one step destroy the
current entity *or* one other entity, never both. `attack/ecs/cursor_check2.c` lets a step destroy the
current entity and up to three others in any order, and checks the pool after every destroy:

```text
del as written in ember_direct.c:        seed 3: pool broken: ents[21]=262 but sparse says 262
the same, without the pool check:         del: entity 262 has no valid slot (sparse=262, n=156): a correct program panics
del with the guard:                       cursor-check2 (del fixed) ok    (2,000 seeds)
```

The cause: in the first step of a walk the current entity is the last of the pool. Destroying it makes
the cursor equal the length; destroying an entity not yet reached then moves the last unreached entity
(which is now also the last of the pool) into the freed slot, and the second move, "last into the
unreached slot", moves it onto itself and points its `sparse` entry back at the old slot, past the end.
Later lookups of that entity find nothing or another entity's component, and destroying it panics with
"index out of bounds". Memory stays safe (every index is checked), but a correct program stops. The
answers report's other model, `ember_registry.c`, already skips that move with `if (j != last)`; the
timed model `ember_direct.c` and the self-check lack it.

*Save:* the guard (`attack/ecs/ember_direct_fixed.c`). Free: it is on the rare path.

### 7.3 Speed

Re-measured with the fix (`attack/runs/ecs_2.txt`):

| program | median ms | ratio |
|---|---|---|
| `plain_sparse.c` (plain C, the answers report's baseline) | 98.594 | 1.00 |
| `ember_direct.c` (the answer as written) | 108.952 | 1.11 |
| `ember_direct_fixed.c` (saved) | 107.038 | 1.09 |
| `rust_sparse.rs` (safe Rust) | 111.061 | 1.13 |

This agrees with the answers report's 1.03x to 1.12x. The differences are under 15%.

I found no other hole. The models look entities up without the generation check; `std.ecs`'s lookup
`slot_in` compares the whole entity, generation included (`std/src/ecs.em` lines 83 to 90; ids are reused
at the next generation by `World.create` and
`destroy`), so a name used after `destroy(e)` and a `create()` that reuses `e`'s index finds nothing,
as rule 2 needs.

### 7.4 Verdict (first pass)

**Saved.** What is left: as the answers report says, in loops nested over one pool, an inner loop that
destroys entities other than its own current one keeps "every entity once" only for itself.

### 7.5 Second pass: the layout the owner chose

The answer and both checks above are in EnTT's sparse layout (one pool walked, the other probed). The
owner chose EnTT's full-owning group for this loop (the workflow's W1 note), and the combined model's
`combined/w1.c` carries the cursor rule into the group's `destroy`; its own self-check
(3,000 random loops) prints `cursor-check ok` when I build it with `clang -O2 -DCURSOR_CHECK
../../combined/w1.c` in `attack2/ecs/` and run it. That check destroys and creates only. A Python game also
takes one component away from an entity that lives on (`del vel[e2]`, Ember `reg.remove[Velocity](e2)`:
the entity leaves the group) and gives one to an entity that had only the other (it joins the group in the
middle of the loop).

`attack2/ecs/group_check.c` copies the combined model's library unchanged and adds `remove[Velocity]`;
5,000 random loops in which each step does up to four of: destroy the current entity, destroy another,
remove `Velocity` from another, add `Velocity` to an entity that had only `Position` (it joins), create an
entity with both (it joins); after every change the pools and the group must be consistent; every entity
in the group at the start and still in it at its turn is visited once; an entity that joins is not
visited. Output `group_check_output.txt`:

```text
== group_check.c (remove_vel with the cursor rule)
group-check ok
== -DREMOVE_AS_DESTROY_WITHOUT_CURSOR
seed 2: entity 63 joined during the loop and was visited
```

So the rule holds in the owning group, but only if **every** operation that takes an entity out of the
walked range applies it: `remove[T]` written as a plain group leave (swap with the last group slot) moves
an entity from the part the loop has already passed (here one that joined during the loop) into the part it
has not reached, and the loop visits it. Memory stays
safe (every index is checked); "every entity once" breaks. *Save, free:* the cursor test lives in the
group's one "leave" operation (and, in the sparse layout, in the pool's one delete), which `destroy` and
`remove[T]` both call; the answers report's wording ("`destroy` compares the removed position with the
cursor") becomes "every removal from the walked range compares its position with the cursor".

**Verdict after the second pass: saved.** What is left is unchanged from 7.4. Timing in the group layout
is the final measurer's (`combined/w1.c`).

---

## 8. Parts replaced, and their good parts

Nothing was discarded whole. These parts of the answers, or of the first pass's saves, were replaced;
what was good in them is noted.

| Replaced part | From | Replaced by | Good part kept |
|---|---|---|---|
| The walkers count in the top byte of the capacity word | answers report A4 rule 9 | a walking state in the field's access word (first pass), publishing its path (second pass) | the count itself (now overflow-checked); "adding is allowed while a loop walks" |
| Swap-fill of the gap for elements that own memory | answers report A4 rule 10 | other code sees the logical list, so the gap is never seen | "every slot always holds one valid element": the defence to use if a mapping ever has to be skipped (`ember_cb_saved2.c`, 1.27x in `a4_cb_2`) |
| Copy in, write back after the call | answers report A2 rule 6 (Swift's in-out) | a place: list and position, moved by the callee's removals | copies are right when the callee cannot change the list (unchanged) |
| Writing the element back before a read by position | first pass's first save (1.42x) | compare and select (1.01x), for writes too (0.98x, second pass) | still used when the whole element is taken by reference |
| Two compiled copies for two spans | answers report A3 | one `restrict` copy, the read span copied on overlap | the second copy stays for two written spans |
| 24-byte owner field | answers report A25 | an 8-byte pointer to an owner block | the take-along idea, now also used for local lists |
| Rule 2 checked in every function with two parameters of one type | answers report A3 | a no-alias copy for checked or proven calls, keep-alive in the alias copy | the compile-time check, now only where aliasing is real |
| A range compare inside the call that changes the list, for every view argument | first pass's A3 rule 4 | (second pass) the outer call copies a view whose source the callee may change, decided from the callee's summary; the range compare kept for one known list | the range compare: cheaper than a copy when the callee changes one known list |
| "Keep the old buffer alive" for every write through the other name | first pass's A3 rule 3 | (second pass) kept for replacing a whole string; a view of elements is copied before a removal, replacement or move | keep-alive: free for strings |
| The access-word state keyed by the class field | first pass's A4 save | (second pass) keyed by the walk's path | the access word as the place the state lives (if the owner keeps the mode) |
| `clear()` frees the take-along block | first pass's A25 save | (second pass) the free moves to the last use of a view taken from the list | the block itself |
| A plain count on the owner block | first pass's A25 save | (second pass) the threads group's shared flag on the block | plain count for blocks that never cross threads |
| The cursor test inside `destroy` | answers report ECS rule 4 | (second pass) the cursor test in the one operation that takes an entity out of the walked range | the rule itself, checked over 5,000 random loops in the owning group |

---

## 9. The final answer list for the group

All of it is decided when the program is compiled, except where a line says "at run time"; every run-time
part is for code the compiler cannot see.

**A2: holding an element while adding to the list. "Names are positions", saved.**
1. `boss = enemies[0]`, where the element is not a plain number, names position 0 of `enemies`.
2. The element's address is kept only across code that cannot change the list or any container on the
   name's path, counting writes through names that may be the same list (A3); after a change the buffer
   is read again (no bounds check after adding; checked after anything that may remove).
3. A removal or insertion before the name moves it (`j -= (k < j)`); removing the named element moves
   its value into the name. Replacing the named slot (`enemies[k] = v`) does the same (the swap idiom
   works). A second name of the same element, or a variable the named element is popped into, is
   followed by a one-bit flag.
4. Sorting, reversing or swapping the list while the name is still needed is a compile-time error.
5. Passing a name to a function that may change its list passes the place (list, position); the
   function's removals move it, after comparing the list it removes from with the place's list by
   identity (class handles may be one object). No copies, no compile-time error for elements that own
   memory.
6. Across a call the compiler cannot see that may remove from the list: with A4's run-time mode, the
   name's position is published and other code's removals move it (the gap-free case of the mapping
   `walker_check.c` checks; not built for names, UNVERIFIED); without it, using a struct element's name
   after such a call is a compile-time error (fix-its: copy it first, or make the element a class).
Cost: nothing on the common path; a compare and a subtract or copy per change while a name is live.

**A3: two elements of one list in one call. "Two names may be one element", saved.**
1. `collide(bodies[i], bodies[j])` is allowed, `i == j` included, with Python's meaning.
2. Functions are compiled assuming no aliasing; a call that may pass one place twice compares (`i != j`,
   two ranges, two list headers) and calls the no-alias copy when they differ, the alias copy otherwise.
3. In the alias copy, a view of a `String`'s characters used after a write that replaces the string keeps
   the old buffer alive until the view's last use; a view of a list's elements used after a write that may
   remove, replace or move them is copied just before that write. A view into an inline enum payload
   across such a write is a compile-time error at the call.
4. A view passed to a call: when the callee's effect summary may change the view's source list, the call
   passes a copy (Python's slice copy); when the callee changes one known list, a range compare and a copy
   only when the view is inside; otherwise the view itself. A function with one written and one read span
   is compiled once.
Cost: element pairs free (answers report); spans 1.08x of `restrict` C with bounds checks, safe Rust 1.13x;
two list parameters 0.98x (all PROVISIONAL, first pass); copies only where Python copies anyway.

**A4: changing a list while looping over it. "The loop becomes a one-pass filter", saved.**
1. A loop whose body may change the list it walks is an in-place filter with `r` and `w`, to the live
   length; appended elements are visited; `enemies.remove(e)` drops the current element without skipping
   the next; other removals in the body are mapped; `len` is `w + (len - r)`, less one after removing
   the current element; `enumerate`'s index is `e`'s current position (`w`).
2. A loop over a copy or a whole slice of the list its body changes (`xs[..]`, `xs[a..]`, `xs.clone()`,
   `Array(xs)`) is the same walk, bounded by the elements there at the start; no copy is made and writes
   through `e` reach the list.
3. `e` stays in registers; a field read or written through the list by position compares the mapped
   position with `r` and uses `e` when they are equal; code that takes the whole element by reference
   gets it stored first.
4. A removed element is dropped at the end of its step.
5. Inserting, sorting or reversing the walked list in the body, and removing in a loop nested over the
   same list, are compile-time errors; `break` closes the gap; a visible method that removes is compiled
   into the loop.
6. Code the compiler cannot see that may change the walked list, in a loop that also removes from it: the
   owner picks one of two (section 5.8). Either the run-time mode: the loop holds a walking state for its
   path (overflow-checked), publishes `r`, `w` and flags before each such call, spills `e` and reloads it
   at the spill's widths if other code wrote it, and finds its list again through the path; other code sees
   the logical list (length, reads, writes, removals, appends); replacing, moving, dropping or reordering
   anything on the path, holding a view of the list, and a second removing loop over it panic (at run
   time). Or Rust's rule: a compile-time error with fix-its (the combined model's choice).
Cost: 0.97x of plain C for the answers report's loop (re-timed, second pass); 1.01x with reads by position;
0.98x with writes by position; the run-time mode 1.00x to 1.02x of plain C written for an unknown callback,
1.26x to 1.34x of plain C that knows its callback (all PROVISIONAL).

**A25: lists of string views. "A list carries its text's lifetime, or its text", saved.**
1. A local list of views carries the lifetime of what it views, inferred.
2. Views of a text that dies (a temporary, a loop's line) stored in a list that lives longer move that
   text into the list's owner block, if the list is never shrunk while it holds taken texts.
3. A returned or stored list whose views point into texts that would die takes them along the same way.
4. The owner is an 8-byte pointer in the header, null when nothing was taken; a copied list shares the
   block with a count (if lists copy, which the objects group decides), atomic only when the block crosses
   threads (the threads group's shared flag).
5. `clear()` frees the block, unless a view read out of the list is used after it: then the free moves to
   that view's last use; such a view used across a loop that clears the list again is a compile-time error
   with a `to_string()` fix-it.
Cost: 1.00x (lines taken along) against safe Rust's two-phase workaround 1.13x and owned words 1.77x,
at 0.65 of the owned words' memory (PROVISIONAL, first pass).

**ECS: creating and destroying entities inside a view loop. "Walk backwards with a cursor", saved.**
1. The view loop (library code) walks the smaller pool backwards (in the owning group: the group's range
   backwards); components are names keyed by the entity, looked up again after anything that may change
   the pools; the pools' arrays are held in locals on the common path.
2. The loop publishes its position in the cursor before calls that may change the walked range; every
   operation that takes an entity out of that range (`destroy`, `remove[T]`: the pool's delete, or the
   group's leave) moves the last unreached entity into the leaving one's slot when the leaving one has not
   been reached, then removes that slot as a visited one, skipping the second move when that slot is the
   last; loops save and restore the cursor.
Cost: 1.09x of plain C in the sparse layout, safe Rust 1.13x (PROVISIONAL, first pass); the owning-group
version is the combined model's `w1.c`, timed by the final measurer.

What these need from the other groups is unchanged from the answers report's section 7, plus: if the owner
keeps A4's run-time mode, a home for its walking state (the objects group's access words, which the lean
class removed, or a bit in the count word with a per-thread record of walks); A2's places use the
references group's returned places.

---

## 10. Files

First pass, under `target/redesign2/containers-attack/`:

- `probes/`: `py_attack.py` and `py_attack_output.txt`; the Ember attack programs `x_a2_swap.em`,
  `x_a2_hit_purge.em`, `x_a3_extend_view.em`, `x_a25_lines.em` with `ember_check_attack_output.txt`;
  `rust/x_extend_own.rs`, `rust/x_rename_both.rs` with `rust/rustc_attack_output.txt`.
- `a2/a2_models.c` and `a2_models_output.txt`: the swap idiom and the hit-and-purge call, as written and
  saved.
- `a3/extend_attack.c` and `extend_attack_output.txt`; timed: `span_answer.c`, `span_add.c` (a copy of
  the answers report's, built with `-DRESTRICT` and `-DCALLCHECK`), `rust_span_fair.rs`, the answers
  report's `rust_span.rs`; `alias_lists.c` (built with `-DRESTRICT`, `-DALIAS`, `-DCALLCHECK`),
  `rust_lists.rs`.
- `a4/attacks.c` and `attacks_output.txt`; `walker_check.c` and `walker_check_output.txt`; `count_calls.c`
  and `count_calls_output.txt`; timed: `plain_read.c`, `ember_read_naive.c`, `ember_read_wb.c`,
  `ember_read_sel.c`, `plain_cb.c`, `plain_cb_logical.c`, `ember_cb_report.c`, `ember_cb_saved.c`,
  `ember_cb_saved2.c`, `ember_cb_walker.c`, `ember_cb_walker2.c` (all include the answers report's
  `containers/bench/a4/common.h`).
- `a25/lines.h`, `lines_takealong.c`, `lines_owned_sso.c`, `lines_owned_malloc.c`, `rust_lines.rs`
  (built with and without `--cfg twophase`).
- `ecs/cursor_check2.c` (built with `-DDEL_ANSWER`, with `-DDEL_ANSWER -DNO_POOL_CHECK`, and plain; output
  in `cursor_check2_output.txt`), `ember_direct_fixed.c`; the answers report's `plain_sparse.c`,
  `ember_direct.c` and `rust_sparse.rs` were built from `containers/bench/ecs/`.
- `run.ps1`, `summarize.sh`, `harness.h`, `rharness.rs` (copies of the answers report's); results in
  `runs/a4_read.txt`, `a4_cb.txt`, `a4_cb_2.txt`, `a4_cb_3.txt`, `a3_span.txt`, `a3_lists.txt`,
  `a25_lines.txt`, `ecs.txt` (disturbed, not used), `ecs_2.txt`.

Second pass, under `target/redesign2/containers-attack2/`:

- `repro/py_attack_output.txt`: the first pass's Python checks, rerun (identical); the first pass's C
  checks were rebuilt there and their outputs compared with `diff` (2.2).
- `probes/py_attack2.py` and `py_attack2_output.txt` (Python behaviour, and the Python model of the walk);
  the Ember programs `x_copy_loop.em`, `x_copy_loop_view.em`, `x_copy_loop_clone.em`, `x_reinforce.em`,
  `x_regroup_clear.em`, `x_best_clear.em`, `x_nested_walk.em`, with `ember_check_attack2_output.txt`;
  `rust/x_reinforce.rs`, `rust/x_regroup.rs`, `rust/x_copy_loop.rs` with `rust/rustc_attack2_output.txt`.
- `a3/view_attacks.c` and `view_attacks_output.txt` (the two A3 holes, as written and saved).
- `a4/nested_attack.c` and `nested_attack_output.txt` (the list inside a list); timed: `plain_cb_any.c`,
  `plain_write.c`, `ember_write_sel.c`, and `ember_write_lost.c` (checksum only); the first pass's
  `plain_cb_logical.c` and `ember_cb_walker.c` and the answers report's `plain.c` and `ember.c` were built
  from their own folders.
- `a25/best_clear.c` and `best_clear_output.txt`.
- `ecs/group_check.c` (built plain and with `-DREMOVE_AS_DESTROY_WITHOUT_CURSOR`) and
  `group_check_output.txt`; the combined model's `w1.c` built with `-DCURSOR_CHECK`.
- Results in `runs/a4_base.txt`, `a4_cb.txt`, `a4_cb_2.txt`, `a4_write.txt`, each with a `_summary.txt`
  (`sh ../containers-attack/summarize.sh <file>`), timed with `powershell -NoProfile -File
  ../containers-attack/run.ps1 <file> <exe> ...` on mains power (`BatteryStatus` 2).

The `.exe` files were deleted to keep the repository small; each program rebuilds with the commands in
section 2.
