# Phase 3b: salvaging last night's three memory models

Written 2026-10-24 by the phase-3b salvage agent, for the owner's job in `JOB.md` (speed on par with C
and Rust, memory safety the same as Rust's or better, Python-like to read and write; before anything
counts as failed, try to save it).

The three designs are named by what they do:

- **"copy instead of share"** (`phase3-design-D1.md`): lists and dicts are values that copy on write,
  classes are counted objects, parts of values are reached through "projections".
- **"checked pointers"** (`phase3-design-D2.md`): objects live in slots that never move; a pointer
  carries a generation number that is checked when it is used.
- **"today's rules, but never an error"** (`phase3-design-D3.md`): today's compile-time borrow checking,
  with an automatic run-time fallback wherever today's compiler would refuse the program.

What I read: `JOB.md`, the three designs, their three attacks (`phase3-break-*.md`), `phase3-measurements.md`,
`FINDINGS.md`, every C model under `target/redesign/bench/`, the breaker's probe copies in
`target/redesign/break-D2/`, and the parts of phase 1 and phase 2 cited below.

What I made (all under `C:/Users/ism19/Code/ember/target/redesign2/salvage/`):

- **Plain C baselines** (C with no safety checks, written for speed) in the layouts the task asks for:
  `plain/w1_group.c` (new, W1 in EnTT's owning-group layout), the four phase-3 baselines copied, and two
  attribution variants (`plain/w2_emberlist.c`, `plain/w4_env32.c`).
- **Safe Rust baselines** doing the same work: `rust/w1_group.rs`, `rust/w2.rs`, `rust/w3.rs`, `rust/w4.rs`
  (harness in `rust/harness.rs`; its only `unsafe` is three Windows calls for the memory figure).
- **Salvaged C models**, one per design and workload: `d1/` ("copy instead of share"), `d2/` ("checked
  pointers"), `d3/` ("today's rules, but never an error"), each built from the phase-3 model with the fixes
  named in its header comment. Build switches (`-DNAME`) make the attribution builds.
- `build.sh` (every build command), `runs/ab.ps1` and `runs/final.ps1` (the timing), `runs/summ.py` (the
  medians), `runs/SUMMARY.txt` (all medians and ratios), `py_salvage.py` (the Python behaviours quoted).

Every one of the 50 programs printed the known checksum on every run (W1 2111831520, W2 647034814,
W3 9288233970, W4 34946993000; `runs/final_w*.txt`, column "checksum" of `runs/SUMMARY.txt`).

No Ember program was compiled or run. Nothing was downloaded or installed. No file outside
`tasks/96/redesign/phase3b-salvage.md` and `target/redesign2/` was written.

---

## 0. The short version

**All three designs are partly saved.** Their slowness was mostly implementation, and every
critical and major safety hole has a fixed rule that survived the attacks I tried. What no fix removes
is in each design's Python-likeness:

| Design | Before (phase-3 model, ratio to plain C in the same layout) | After (salvaged, same) | Verdict | What no fix removes |
|---|---|---|---|---|
| copy instead of share | W1 1.39, W2 1.89, W3 1.15, W4 1.39 | W1 1.21, W2 1.24, W3 0.82, W4 1.04 | partly saved | its central rule: a list is a value, so `matrix.append(row); row.append(1)` gives `[[]]` where Python gives `[[1]]`, silently |
| checked pointers | W1 2.34, W2 1.75 (1.38 faithful), W3 1.05 (1.14 faithful), W4 1.08 (1.36 faithful) | W1 1.10, W2 1.24, W3 0.94, W4 0.96 | partly saved | an object dies with its one owner: `old = p.items; p.reset(); for x in old:` panics where Python works, when the overwrite happens in a called function |
| today's rules, but never an error | W1 1.78, W2 1.11, W3 0.97, W4 1.03 | W1 1.06, W2 1.07, W3 0.97, W4 0.96 | partly saved | its promise of no annotations and no refusals: ownership by "first place stored" makes two equal-looking lines decide when objects die, threads need compile-time refusals, and its W2 speed rests on a whole-program fact (without it: 1.95) |

Source of every ratio: my runs, `runs/final_w1.txt` .. `final_w4.txt` (7 rounds each, medians in
`runs/SUMMARY.txt`). "Same layout" means: W1 against plain C in the same EnTT layout (the phase-3
models against the sparse-set view, the salvaged ones against the owning group); W2 against plain C
with Ember's list layout (`plain/w2_emberlist.c`, section 1.3); W3 and W4 against the phase-3 baselines.
"Faithful" is the phase-3 breaker's corrected version of the "checked pointers" model. All timings are
PROVISIONAL (section 1.1).

What made them slow, in one line each (details per design below):

- **Allocation and layout, not the memory model**: one `malloc` per object instead of a slab (W3 +23%,
  W4 +21%), a type pointer instead of a class id for virtual calls (W3 +6%), a 64-bit owner word beside
  every ECS component and no swap-and-pop, a list kept as a separate object (W4 +26%), a pin on every
  ECS loop.
- **A check or a count in the wrong place**: a count pair around every closure call (W4 +11%), a pin
  around every closure call (small), a generator that would fetch its pools again on every step if not
  compiled into the loop (W1 2.91 against 1.21).
- **A reference that costs an address calculation on a chain of dependent loads**: `base + index * 40`
  per hop of W2's parent walk; the packed 64-bit reference (one AND per hop) costs +16% against a raw
  pointer, the index form +64%.
- **What is left after the fixes**: a check that only a whole-program fact removes (W2: +18% for the
  weak test, +27% for the generation check with packed references, +83% for the 12- and 16-byte
  watchers; each against the same design with the fact), and copy-on-write uniqueness tests in library
  code (W1 +15% for "copy instead of share").

Against safe Rust doing the same work (my Rust programs, same runs): Rust is 1.17 (W1), 1.48 (W2),
1.28 (W3) and 0.71 (W4) of plain C. All three salvaged designs beat Rust on W2 and W3; on W4 Rust is
faster, with a differently shaped program (the handler cannot reach the dispatcher; section 1.2).

The decision for the owner is the same as last night's, now with the speed taken out of it: when an
object is still referenced and its owner lets go, does it stay alive (Python's meaning; it needs
counting, which the salvaged "copy instead of share" object model does at 0.82 of plain C on W3 and 1.04
on W4) or does it die and the reference read `None` or panic (the other two)? And: are lists values
(not Python) or shared objects?

---

## 1. How this was measured

### 1.1 Method

- Machine: the owner's laptop (Intel Core Ultra 9 275HX), on mains power: `(Get-CimInstance
  Win32_Battery).BatteryStatus` returned 2 before every round, checked by `runs/ab.ps1`, which aborts
  otherwise (no round aborted).
- Compilers: `clang version 22.1.8`, `-O2`; `rustc 1.98.1`, `-C opt-level=2 --edition 2021`.
  Every command is in `build.sh`.
- Runner: `runs/ab.ps1`. Each program is started with `cmd /c start "" /b /wait /affinity C03C03`, so it
  is pinned to the performance cores (the "P-cores", affinity mask 0xC03C03) from its first instruction,
  opens no window, and runs alone.
- `runs/final.ps1`: for each workload, 7 rounds; in each round every program of that workload runs once,
  in a fixed interleaved order, so A and B alternate. The tables give the **median** of the 7 runs (the
  middle value) and the median peak working set (the harness's memory figure).
- **Spread** is (slowest - fastest) / fastest over the 7 runs: W1 9% to 27%, W2 7% to 19%, W3 at most
  3.3%, W4 2% to 9% (`runs/SUMMARY.txt`). Other agents were running on the same laptop, and an earlier
  W2 batch (`runs/w2.txt`) had one disturbed round with spreads up to 55%; I re-ran it. **All my
  timings are PROVISIONAL.** A conclusion resting on a difference under 15% waits for the final measurer;
  I mark those.

### 1.2 The baselines

| Workload | plain C (ms, KB) | safe Rust: ratio time (memory) | What the Rust program does |
|---|---|---|---|
| W1, ECS, EnTT owning group | 88.5, 39,172 | 1.17 (1.01) | the same pools and group; an index loop with bounds checks, because Rust refuses `&mut` into the pools across `world.destroy(e)` |
| W1, archetype layout (reference) | 42.5, 17,120 | | |
| W1, EnTT sparse-set view (phase 3's layout) | 95.2, 39,120 | | |
| W2, tree | 529.3, 54,940 | 1.48 (0.95) | an arena `Vec<Node>` with `u32` parent and child indices, every index bounds-checked |
| W3, game objects | 444.9, 9,756 | 1.28 (1.18) | an arena of generational slots (the `slotmap` idea, written out), targets as `{index, generation}` keys checked on use |
| W4, callbacks | 23.76, 4,076 | 0.71 (1.10) | boxed closures, `Rc<Cell<i64>>` for the shared total; the handler RETURNS "unregister me" and the dispatcher does it after the call, because Rust refuses a handler that changes the list it is called from |

Source: `runs/final_w1.txt` .. `final_w4.txt`.

Terms. **EnTT's layout**: one "sparse set" per component type: `sparse[id]` gives the slot, `ents[slot]`
the entity id, `data[slot]` the component. An **owning group** of Position and Velocity keeps the
entities that have both in slots `0 .. glen-1` of both pools, in the same order, so the loop reads both
arrays straight through (`plain/w1_group.c`). The **archetype** layout keeps each entity's components
side by side in one table (`plain/w1_arch.c`, phase 3's `plain/w1.c`).

Two facts about the baselines the owner should have:

1. **The EnTT layout costs 2.08x the archetype layout on W1 in plain C** (88.5 against 42.5 ms), and the
   owning group gains only 7% over the sparse-set view (88.5 against 95.2 ms). The owner chose EnTT; this
   is the price of that layout in this workload, before any memory model.
2. **Rust beats plain C on W4 (0.71), with a differently shaped program.** In the Rust loop the
   list's length stays in a register across the handler call (the handler cannot reach the list); in the
   C loop, which hands the dispatcher to the handler as the workload's Python shape does, the length
   and buffer are loaded again from the dispatcher after every call and `d.cur` is stored before it.
   I read both loops in the assembly (`rustc --emit asm` of `rust/w4.rs`; `clang -S plain/w4.c`); that
   these differences account for the whole 29% is UNVERIFIED.

### 1.3 Comparing like with like

A model is compared with plain C written in the same data layout wherever the layout is not the memory
model's own choice. Two cases needed a second plain C program:

- **W2's list layout.** Every model gives each node a child list with Ember's header (pointer, 64-bit
  length, 64-bit capacity, growth from 4, `[HEAP-2]`); the plain baseline uses 32-bit length and
  capacity and grows from 2. `plain/w2_emberlist.c` is the plain baseline with Ember's list: **1.09x the
  time and 1.25x the memory of plain C** (579.0 against 529.3 ms, 68,664 against 54,940 KB). That is a
  standard-library choice, the same in every memory model; I report W2 against both.
- **W1's layout**: section 1.2.

One more layout difference stays in every W2 model, because the Python program has it: `nodes` is a
list of references to `Node` objects (1,048,576 slots of 8 bytes = 8,192 KB), where plain C keeps the
nodes themselves in one array. The models use 76,564 to 77,116 KB against 68,664 KB for plain C with
Ember's list (`runs/final_w2.txt`).

---

## 2. "copy instead of share"

### 2.1 What it is

Lists, dicts, strings and structs are **values**: `b = a` gives `b` its own copy, made lazily
(**copy-on-write**: the two share one buffer with a count; the first change through either copies the
buffer first, after a **uniqueness test**, `count == 1`). Classes are **objects** shared through
**counted handles** (a reference count goes up on each copy of a handle and down when it dies; the object
is freed at zero); a `weak` field does not keep its object alive and reads `None` once the object is gone.
Parts of values are reached in place through **projections**: a `mut` parameter, `for x in place`,
`with x = place:`, and accessor functions that `yield` a place. A projection is a path (root, fields,
indices), so after any call that may change what it points into, the compiler fetches its address again
from the path.

### 2.2 What made it slow, per workload, and the fixes

| Workload | Cause found | Idea good, implementation weak? | Fix tried | Measured (ratio to plain C, same layout) |
|---|---|---|---|---|
| W1 | The view is a generator; the model assumed it was compiled into the loop. Without that, the design's "fetch again" rule (its D1-6) fetches both pool projections again on every resume (the breaker's M1) | Yes: the generator idea is fine, the compile step was missing | Rule: a generator consumed by a `for` whose body is visible is compiled into that loop (the same transformation Rust needs for iterators). The breaker's B9 fix (below) makes every generator that holds a `mut` projection such a generator | not compiled in: **2.91**; compiled in: **1.21** (`d1_w1_group_noinline`, `d1_w1_group`) |
| W1 | Uniqueness tests and the `cast` compare in library code (registry, pools), on every emplace, destroy and group swap | Partly: the tests are the price of copy-on-write; a compiler that knew the registry is never copied could drop them | Experiment `-DD1_PROVEN`: every uniqueness test and the cast compare removed | **1.06**. The analysis that would justify it must see through the library's calls (the registry is a local of `main` that is never copied); I did not show such an analysis can be built: UNVERIFIED as a compile step |
| W1 | (phase 3) the sparse-set probe per entity | layout | owning group | the phase-3 model was 1.39 of the sparse-set C; this model is in the group layout |
| W2 | One `malloc` per object (nodes scattered) and a 16-byte header with a type pointer that a final class does not need | Yes | Objects from a **slab** (a block of same-size slots handed out in order, then reused from a free list); an 8-byte header (strong count, weak count) for a final class: Node is 32 bytes | phase-3 model **1.89** → **1.24** (`orig_D1_w2` 1092.4 ms → `d1_w2` 718.3 ms; against `plain_w2_emberlist`) |
| W2 | The weak test on every hop of the parent walk (`parent.strong == 0`) | The test is the price of `weak`; a whole-program fact ("no Node's count reaches 0 while `main` runs") removes it | Experiment `-DD1_PROVEN` (only the test removed) | **1.05** (`d1_w2_proven` 606.5 ms). The test costs +18% |
| W3 | One `malloc` per object | Yes | slab with free list (a block goes back when its weak count reaches 0) | attribution build `-DD1_MALLOC`: 1.01 against **0.82** with the slab |
| W3 | Virtual call through object → type pointer → method (two dependent loads: the second address comes from the first load) | Yes | a 4-byte class id in the header and one table per method, indexed by class id (as plain C does) | attribution build `-DD1_TYPEPTR`: 0.87 against **0.82** |
| W4 | The CALLER retains a stored closure's box around every call, because the call may remove that closure (the design's lend rule, its D1-7): a count pair on each of 10,000,000 events | Yes: the protection is needed, but only where the box is used after a call that may release it | **Protect at the point of use**: a function that uses a counted handle (its own box included) after a call that may release it retains it across that call. The handler does, once per 100,000 events, on its rare path; the dispatcher uses nothing after the call and retains nothing | **1.39 → 1.25** (`d1_w4_callee`) |
| W4 | Each closure box is 32 bytes (a count word more than plain C's 24-byte environment) and malloc'd | Yes | boxes from a 32-byte slab | **1.25 → 1.04** (`d1_w4`). Check from the other side: plain C with one more 4-byte word in its malloc'd environment is itself 1.12 (`plain_w4_env32`) |
| W4 | The program stores two closures per handler (one to read the counter at the end): 32-byte elements | the program's spelling | attribution build `-DONECLOSURE` | 0.99 (`d1_w4_oneclosure`) |
| W4 | An Array value is one pointer, so the dispatcher loads the length through it | tested: no | `-DARRAY3` (length beside the pointer) | 1.04, no gain (`d1_w4_array3`) |

Source: `runs/final_w1.txt` .. `final_w4.txt`; files `d1/w1_group.c`, `d1/w2.c`, `d1/w3.c`, `d1/w4.c`.

Memory, salvaged: W1 0.95, W2 1.12 (against same-layout C; 1.40 against plain C), W3 0.86, W4 1.12.

The W4 fix, in the program the owner reads (the Python version beside it):

```ember
fn make_handler(m: int) -> Handler:
    counter = 0
    fn on_event(d: Dispatcher, k: int, v: int):
        counter += v * m
        total += 1
        if k % 100_000 == 99_999:
            d.unregister_current()                 # may release this closure's own box, and
            d.register(make_handler(k % 17 + 1))   # this line still reads the box (its link to
                                                   # `total`): the compiler retains the box across
                                                   # the call above, on this path only
    return Handler(on_event=on_event, counter=fn() => counter)
```

```python
def make_handler(m):
    counter = 0
    def on_event(d, k, v):
        nonlocal counter
        global total
        counter += v * m
        total += 1
        if k % 100_000 == 99_999:
            d.unregister_current()
            d.register(make_handler(k % 17 + 1))
    return on_event
```

The C it compiles to (`d1/w4.c`): the dispatcher calls `h.on_event(h.ev_box, d, k, k % 1000)` with
no count change; inside the rare branch, `b->rc++; dispatcher_unregister_current(d); ...;
hbox_release(b);`.

### 2.3 What made it unsafe: each critical and major hole, the fixed rule, and the attack

The breaker (`phase3-break-D1.md`) found use-after-free in six ways (its B1, B2, B5, B6, B8, B9: critical),
a wrong-type read (B7) and two data races (B3, B4) (major). For each: the fixed rule, the program I
attacked it with, and what happens. The programs are the breaker's (its report runs their Python
versions) or mine; where Python's behaviour matters, `py_salvage.py` prints it.

| Hole | Fixed rule | Attack | Result | Cost |
|---|---|---|---|---|
| A list and its own element passed together to be changed (B1: `grow(v, v[0])`, `push` frees the buffer `x` points into) | The exemption "two names for plain data cannot free anything" holds only when BOTH arguments are plain values; a place and a part of it are never passed together if either is changed | `grow(o1.items, o2.items[0])` with `o1` and `o2` the same object at run time | the element is lent from a list reached through an object, so the design's lend rule (its D1-7) pins `o2.items` for the call and `push` panics instead of reallocating | none at run time; one more refusal (in Python this call does not even change `v[0]`: a float is passed by value) |
| A local captured by a closure counted as unreachable (B2: a closure clears the list holding it, then writes its own box) | A local captured by reference by an escaping closure is treated as an object field by the "fetch again" rule (D1-6) and the lend rule (D1-7: retain, pin) | `grow(squads[0], fn(): squads = [])` | `squads` is pinned for the call; the reassignment panics | a pin or retain only on calls that involve such locals; a panic where Python works (Python changes the squad and prints `[1, 2, 3]`, `py_salvage.py` item 12) |
| Nested lends clear a one-bit pin (B5) | The pin is a counter; a buffer released while pinned is freed at the last unpin; buffer counts are checked for overflow | three nested lends, the owner dropped in the middle (the breaker's B6c) | the free waits for the last unpin | the same stores; one compare per count increment |
| What a lend does not protect (B6: element replaced through another name, a `Map` entry lent, the owner of a pinned list dies, a `drop` inside a library method) | Caller: every buffer on a lent path is pinned (a pinned `Map` takes inserts into an overflow table, as in "today's rules"), every object on the path retained for the call. Callee: projections derived from a `mut` parameter are fetched again after any call that is not quiet (`[EXC-19]`'s word: cannot reach them). Library containers finish their own change before releasing what they removed | each of the breaker's four programs | (a) `self.bag` is fetched again after `team.respawn(0)`: the new, empty bag, bounds-checked; (b) no rehash under the lend; (c) the retained owner keeps the buffer; (d) the release runs after `remove` is done | in W1: `Registry.destroy(mut self)` fetches `self.pools` again after each `dyn remove` call: **included in the measured `d1_w1_group`**. Elsewhere unmeasured: no workload lends through objects |
| An escaping closure reads a variable written later (B8) | Box every variable that an escaping closure captures and that is written after the capture | `n = 1; f = fn() => n; n = 2; return f` | `f()` returns 2, as Python (checked by the breaker) | one allocation per frame with such a variable |
| A generator frame holding a projection outlives its root (B9: `return evens(local)`) | A generator's borrowed parameters are copies (count +1) when the generator value leaves the expression that made it; a generator with a `mut` parameter must be consumed by the `for` that makes it, and is compiled into that loop | `return evens(local)`; `it = reg.group[...]()` | the first holds a copy of `local`, Python's meaning (`py_salvage.py` item 6 prints 2); the second is refused | a count pair per escaping generator; the W1 loop gets faster (2.91 → 1.21) |
| A count race on a copy out of a lock (B3, major) | A copy that leaves a `Mutex` guard or is read from a `@sync` object is a deep copy; a value sent to a thread is moved after a walk that deep-copies buffers it shares | `with g = scores.lock(): return g[0]` on two threads | each thread gets its own string; no buffer is reachable from two threads, so counts stay plain | an allocation and copy per copy-out (the breaker's other option, atomic counts, is 5.8 ns against 0.84 ns per count pair, `phase2-rc-elision.md` lines 299-300) |
| A scoped task reads a value its parent changes (B4, major) | Compile time: a value a scoped task captures is read-only in the parent until the scope ends. A rule on names suffices here because values never alias in this design (copy-on-write) and objects cross threads only as `@sync` | `ys = xs; ys.push(1)` in the parent while the task sums `xs` | `ys` copies its buffer first; the task's `xs` is untouched | none; a compile error in a racy program |
| Fetching again through a `match` arm, a `cast` or a weak read (B7, major) | Each refetch checks the variant tag, the type or the liveness (one compare); and statically, an arm that assigns to the place it matched binds the payload by copy, not in place | the breaker's `match self.state: Walking(path): self.state = Idle; for step in path:` | the static rule keeps Python's meaning (`py_salvage.py` item 10); if the assignment happens inside a called method, the refetch finds another variant and panics | one compare per refetch through a variant, cast or weak step |

Every fixed rule here is either compile time or a cost on paths the four workloads take rarely or
never. The one that runs in a workload (the refetch in `destroy`) is in the measured model.

### 2.4 Python habits it breaks

The breaker's N1 is the one no fix removes. Lists and dicts are values, so storing one copies it:

```ember
matrix = []
row = []
matrix.append(row)      # a copy (count +1)
row.append(1)           # row copies its buffer first
print(matrix)           # [[]]
```

```python
matrix = []
row = []
matrix.append(row)
row.append(1)
print(matrix)           # [[1]]   (py_salvage.py item 1a)
```

The compiler cannot flag this: the change to `row` is seen (in `row`), it is just not shared. Extending
E-COPYLOST to "a value stored and then changed" would refuse the everyday loop that builds a row,
stores it and goes on using the variable. The only fix is to make lists shared objects, which removes
the design's central rule. The bucket idiom (`lst = d.get("a") ... d["a"] = lst; lst.append(1)`, Python
gives `{'a': [1, 2]}`, item 1b) fails the same way.

Fixable habits (fixed rules, no run-time cost): a plain assignment to a loop variable rebinds it instead
of writing into the list (the breaker's N2; Python leaves the list unchanged, item 9); freeing a value
at its last use is done only for values whose destruction runs no user code (N7).

### 2.5 Verdict: partly saved

- **Speed: saved, with one gap.** W1 1.21 (1.06 if uniqueness tests can be proven away in library code,
  UNVERIFIED as a compile step), W2 1.24 of same-layout C (1.05 with a whole-program fact), W3 0.82,
  W4 1.04. The W1 and W2 gaps are above 15%.
- **Safety: saved on paper.** Nine fixed rules, each attacked with the breaker's program and one of
  mine; none broke. The one fix that runs in a workload is in the measured W1 model.
- **Python-likeness: not saved.** Lists and dicts as values silently diverge from Python
  (`matrix.append(row)`). No fix keeps the rule. If the owner rules that value semantics for lists is
  wanted, this residue becomes a deliberate difference; without that ruling it is a failure of goal 3.

**Good parts**, with where they came from:

1. **The object half of this design**: counted classes, `weak` back edges read by a strong-count test
   and cleared lazily, objects from slabs, class-id dispatch. It was the fastest on W3 (0.82 of plain C,
   which itself clears targets through observer lists) and gives Python's meaning that a referenced
   object stays alive. From this design (its rule D1-3) and Swift's weak references (phase2-swift-mojo.md).
2. **Protect at the point of use** (retain or pin only where a reference is used after a call that may
   release it): W4 1.39 → 1.25 on its own. Made in this salvage; it generalises "checked pointers"'
   O5 (an object running its own method is destroyed when it returns) and `[RC-3]`'s rule that whoever
   uses a handle after a releasing call keeps it.
3. **Projections and yield accessors** (`with x = place:`, `fn get(...)` that `yield`s a place): the
   W1 group loop compiles to plain C's loop once the generator is compiled in. From Hylo and Swift
   (phase2-hylo-csharp.md), and it answers ODR-286's `get` without lifetimes.
4. **Fetch again from the path after a change** instead of forbidding the change: makes adding to a list
   while holding one of its elements harmless (phase 1's annoyance A2). From this design (its rule D1-6).
5. **A generator that changes its arguments in place is consumed where it is made, and compiled into
   that loop**: fixes B9 and the 2.91x of M1 together. Made in this salvage from the breaker's B9 and M1.
6. **E-COPYLOST** ("this change to a copy is lost") for any language that keeps value structs. From this
   design (its rule D1-8).
7. **Copy-on-write for immutable shared data** (strings): fixes the "checked pointers" habit
   `old = p.name; p.name = "Bob"; print(old)` for strings. From this design (its rule D1-1).
8. **A lexical freeze of values lent to scoped tasks**, sound because values never alias. Made in this
   salvage from the breaker's B4 fix (b).
9. **Size-class slabs for objects and closure boxes** (W3 -19%, W4 -17%): a runtime choice any design can
   take. From "today's rules" (its object slabs) and plain C's pool.

---

## 3. "checked pointers"

### 3.1 What it is

Every object lives in a **store**: a reserved range of same-size **slots** that never move. Each slot
starts with a 32-bit **generation**, increased when its object is destroyed, so a reference that still
carries the old generation is **stale**. An object has one owner; everything else holds references
(phase 3: `{slot index, generation}`, 8 bytes). Using a reference compares the generations where no
proof shows the object alive; a stale reference reads `None` or panics. Six proofs (P1 to P6) remove
checks, for example P3: "the only owner of every Node is a list that only grows, so no Node reference
needs a check".

### 3.2 What made it slow, per workload, and the fixes

| Workload | Cause found | Idea good, implementation weak? | Fix tried | Measured (ratio to plain C, same layout) |
|---|---|---|---|---|
| W1 | A 64-bit owner word stored beside every component (16-byte slots), tombstones instead of swap-and-pop so that slots never move, pools in reserved address ranges | Yes: the idea "the slot's occupant is its generation" is good; it was stored twice | The occupant is **EnTT's own packed entity array** (`ents[k]`). A component reference is `{pool, slot k, entity e}`, valid exactly when `k < pool.n` and `ents[k] == e`. Because it re-resolves by index and checks the occupant, slots may move: swap-and-pop is back, pools are ordinary growable arrays | phase-3 model **2.34** of sparse-set C → **1.10** of owning-group C (`d2_w1_group`; under 15%: waits for the final measurer) |
| W2 | The reference is an index: every hop of the parent walk computes `base + index * 40` before it can load, on a chain where each load waits for the previous one | Yes | **The packed reference**: one 64-bit word, `{48-bit slot address, 16-bit generation}` (x64 user addresses fit in 47 bits). Following it is one AND; a slot whose 16-bit generation runs out is retired (`[HND-3]`), so a stale reference can never match a new occupant | as written **1.75** → **1.24** (`orig_D2_w2` 1014.8 ms → `d2_w2` 719.4 ms) |
| W2 | (breaker) the child list as a separate list object: the faithful model was 1.38 time and 1.34 memory | Yes | **Owned fields inline**: a field initialised with a new object and never re-pointed (`children: list[Node] = []`) lives inside its owner's slot | included above (Node slot 40 bytes, memory 1.12 of same-layout C) |
| W2 | The AND itself, on the latency chain (assembly of `d2/w2.c`: `andq %r14,%rcx; movq 8(%rcx),%rcx` per hop; `d3/w2.c`: `movq 8(%rcx),%rcx`) | partly | with P3 holding, a class whose references never need a check can store raw pointers; that compiles to the same walk as "today's rules"' W2 | +16% for the AND (719.4 against 618.6 ms, `d2_w2` against `d3_w2`, same 40-byte slots); raw: **1.07** |
| W2 | When P3 does not hold (one `nodes.pop()` anywhere in the program), every hop checks | the check is the design's | `-DCHECKED` | **1.58** (`d2_w2_checked` 912.6 ms) |
| W3 | (breaker) `objects.remove(o)` is Python's linear search | the program's spelling | written `objects.retain(fn(o) => o.health > 0)` (one compaction pass, the work the model as written did) | faithful **1.14** → as written 1.05 |
| W3 | index references | Yes | packed references | **1.05 → 0.94** (`d2_w3`) |
| W4 | (breaker) the handler list as a separate object: +26% (32.29 against 25.58 ms) | Yes | owned fields inline (now a stated rule, with references to such a field being a reference to its owner plus the field's offset) | faithful **1.36** → as written 1.08 |
| W4 | index references to the closure state and the `total` cell | Yes | packed references | **1.08 → 0.96** (`d2_w4`) |

Source: `runs/final_w1.txt` .. `final_w4.txt`; files `d2/w1_group.c`, `d2/w2.c`, `d2/w3.c`, `d2/w4.c`.

Memory, salvaged: W1 1.00, W2 1.12 (against same-layout C), W3 0.95, W4 1.07.

The packed reference is not visible in programs. What the W2 walk looks like, and what it compiles to:

```ember
for n in nodes:
    m = n
    while m.parent is not None:     # P3: no check
        depth_sum += 1
        m = m.parent
```

```python
for n in nodes:
    m = n
    while m.parent is not None:
        depth_sum += 1
        m = m.parent
```

```c
/* phase 3 (index): m = (NodeSlot *)base + m->parent.idx;   salvaged (packed): */
while (m->parent != 0) { m = (NodeSlot *)(m->parent & 0x0000FFFFFFFFFFFF); d++; }
```

### 3.3 What made it unsafe: each critical and major hole, the fixed rule, and the attack

The breaker (`phase3-break-D2.md`) found two critical holes (H1, H2) and six major ones (H3 to H8).
The programs are the breaker's (its report runs their Python versions) or mine.

| Hole | Fixed rule | Attack | Result | Cost |
|---|---|---|---|---|
| References to values on the stack are not specified, and storing references is unrestricted (H1, critical) | A reference to a value that lives on the stack (a struct local, `self` of a struct method called on a local) is **second-class**: it can be passed down and used, never stored in a field, a list or an escaping closure, never returned; a generator or async frame holding one is consumed where it is made. A reference to a value inside a contiguous list is a position (G3), fetched again after any call that may change that list. To keep a reference, make the thing a `class` (a store slot, checked and storable) or put it in a `Store` | (1) `f(ref s, ref s.items)` where `f` overwrites `s`; (2) store `ref self` into a module variable; (3) `ps[3].scale(2.0)` where `scale` appends to `ps` | (1) inline plain data cannot free anything; `s.items` is an object with a generation, so the second reference reads stale; (2) refused at compile time; (3) `self` is fetched again after the append: still `ps[3]`, bounds-checked | none at run time. Python has no stack structs: a Python programmer writes `class` and gets storable references |
| References carry no store and stores are per thread, so moved, frozen or locked data resolves against the wrong store (H2, critical) | (1) A reference holds its slot's address (the packed form above), so it names the same slot on every thread. (2) A type may cross threads if every non-owning reference it holds points to a frozen class or a `Mutex` (inferred structurally, as Rust's `Send` is derived, never written). (3) A move increases every moved object's generation and hands the receiver fresh references, so references the sender kept read `None`; a destroyed slot goes back to its store through a cross-thread free queue. (4) Frozen objects are deeply immutable and freed after the threads sharing them join, or by an atomic count. (5) Generation words are read and written with relaxed atomic operations (a plain `mov` on x64). (6) A reference derived inside `with m.lock()` cannot leave the block (already in the design) | the breaker's `spawn(work, j)` with `j.log` referring to main's Logger; the sender keeping a reference into the moved Job | the first is refused at compile time (the fix-it: move the Logger too, freeze it, or put it in a `Mutex`); the second reads `None` | O(size of the moved graph) per send; refusals, as Rust's |
| P3 and P4 (P4: objects owned by `main`'s top-level locals or by module variables are live whenever main-thread code runs) are false during teardown and in `drop` (H3) | The proofs that remove checks do not apply in a `drop` body or in code a `drop` can reach; `destroy` checks the generation | the breaker's `REPORTER.drop` writing to a dead `Tally` | checked: reads `None` | checks inside drops only |
| A loop whose body destroys the list it iterates (H4) | A list being iterated is not destroyed until its outermost loop ends (its iteration count defers the destroy, as O5 defers an object running its own method) | a generator suspended inside the loop and then dropped | dropping the generator's frame ends its iteration | one test per list destroy |
| A loop whose body may change its list re-reads the buffer only once per iteration (the design's L5; H5) | Element references from a contiguous list are positions, fetched again after any call that may insert | the breaker's `ps.append` in the middle of an iteration | the write goes to the element's new address | a reload and a bounds check after such calls |
| `Store` element references hold raw addresses (H6) | An element reference is `{store reference, index, occupant}`: the store's generation, the index against the length, the occupant; entity ids are 64-bit serials, or 32-bit index plus version with the index retired when its version runs out | the breaker's two registries that both have entity 0 | the dead registry's store fails its check | three compares per unproven use; none in the view loop (P5: a reference made by the iteration step and used before any change needs no check) |
| Copies of structs split ownership; `destroy` trusts the index (H7) | `destroy` checks the generation (a double destroy panics instead of corrupting the free list: **in the measured `d2_w3` and `d2_w4`**). An owning reference carries an owner bit (one of the packed reference's spare bits); copying a struct clears it in the copy; a drop releases only references whose bit is set | the breaker's `inbox.append(m); println(m.to)` | no double destroy; `inbox[0].to` does not own the string, so when `m` dies it reads stale | one compare per destroy; one bit test per drop |
| Zeroed slots plus a relaxed `E2100` break range types (H8) | `E2100` (a field read before it is set) stays an error for every field whose type is not `Zeroable` (`[ARN-11]`) | the breaker's `Lane = int in 1 ..= 4` | refused at compile time | none |

### 3.4 Python habits it breaks

One owner per object: when the owner lets go, the object dies even if other references remain. The
breaker's N1, with the overwrite in a called function:

```ember
class Worker:
    pending: list[Job] = []
    fn reset(self):
        self.pending = []           # the owner is overwritten: the old list is destroyed

fn flush(w: Worker):
    pending = w.pending             # a reference
    w.reset()
    for job in pending:             # stale reference: panic
        job.run()
```

```python
class Worker:
    def __init__(self): self.pending = ["j1", "j2"]
    def reset(self): self.pending = []

w = Worker()
pending = w.pending
w.reset()
print([j for j in pending])         # ['j1', 'j2']   (py_salvage.py item 2b)
```

Fixes tried: (a) "a field read into a local and overwritten later in the same function moves into the
local" covers the same-function spelling, not the call above; (b) immutable strings as values with
shared, counted buffers (good part 7 of "copy instead of share") cover `old = p.name; rename(p); print(old)`
(item 2a). For lists, dicts and objects overwritten by a callee, only counting keeps Python's meaning.
Not fixed.

### 3.5 Verdict: partly saved

- **Speed: saved except W2 without a proof.** W1 1.10 (under 15%: waits for the final measurer),
  W3 0.94, W4 0.96. W2 1.24 of same-layout C with packed references and P3, 1.07 if P3 also lets the
  class store raw pointers, 1.58 when P3 does not hold.
- **Safety: saved on paper.** Both critical holes have fixed rules that hold against the breaker's
  programs and mine: second-class stack references (compile time) and address references plus inferred
  sendability (compile time) with a generation bump on move (run time, per send). The six majors are
  cheap; the generation-checked destroy is in the measured models.
- **Python-likeness: partly saved.** A referenced object dies with its owner; the common same-function
  spelling and strings are fixed; a callee overwriting an owner still panics where Python works.

**Good parts**, with where they came from:

1. **Generational references, packed into one 64-bit word** `{48-bit address, 16-bit generation}`: 8 bytes,
   one AND to follow, the same meaning on every thread. The generational reference is from Vale
   (phase2-vale-austral-pony.md); the packing is made in this salvage (W2 1.75 → 1.24, W3 1.05 → 0.94,
   W4 1.08 → 0.96). UNVERIFIED, from memory and not checked here: ARM's top-byte-ignore and Intel's
   linear address masking let loads ignore an address's top bits, which would remove the AND; whether
   Windows enables either was not checked.
2. **The occupant as the generation, using EnTT's packed entity array**: component references
   `{pool, slot, entity}` with zero extra memory, swap-and-pop allowed. From this design's section 3.5,
   moved onto EnTT's layout in this salvage.
3. **O5**: an object running its own method is destroyed when that method returns (a pin on the rare
   path only). From this design.
4. **The proofs P1 to P6** that remove checks (owner access, no-destroy regions, grow-only owners,
   program-lifetime owners, fresh references, `self` at entry). From this design; P3 and P4 must not
   apply in `drop` (H3).
5. **Owned fields stored inline** in their owner's slot (W4: 1.36 → 1.08 before the packing). From the
   phase-3 model as written, now a stated rule.
6. **The `own` word** on owning fields: one word, and it is what "today's rules" needs (section 4.4).
   From this design.
7. **Second-class references to stack values**: no lifetimes, no run-time cost. From Hylo and today's
   `[TYP-15]` view rule, applied here.
8. **Generation-checked destroy and retirement of exhausted slots** (`[HND-3]`).

---

## 4. "today's rules, but never an error"

### 4.1 What it is

Today's borrow checker stays as the fast path. Where today's compiler refuses a program, five run-time
fallbacks are inserted instead: **PIN** (a list holds still while a borrow into it lives: a removal leaves
a **tombstone**, a marker in the slot, and an add goes in place or into an overflow page; the list
**settles**, applying them, when the pin count returns to zero), **WATCH** (a stored reference to an
object carries a generation), **COPY**, **CELL** (a counted heap cell for a captured variable) and
**CAST**. An object's owner, its **home**, is the first place it is stored; every other reference
**watches** it. The compiler's **effect summaries** (per function: what it may add, remove, destroy)
decide where fallbacks go.

### 4.2 What made it slow, per workload, and the fixes

| Workload | Cause found | Idea good, implementation weak? | Fix tried | Measured (ratio to plain C, same layout) |
|---|---|---|---|---|
| W1 | Every view loop PINNED both pools with a run-time counter: a tombstone test per element, removals as tombstones, a settle at the end of every step | Yes: the pin was not needed. The loop's cursor can be an index the library re-reads each step, and the references it yields (`pos`, `vel`) end at their last use, before `reg.destroy(e)`. That is non-lexical borrowing, which the static checker this design keeps already does. No borrow is live across the change, so the static check passes and no fallback is inserted | index cursor, no pin, swap-and-pop at once (EnTT's order) | phase-3 model **1.78** of sparse-set C → **1.06** of owning-group C (`d3_w1_group`; under 15%: waits for the final measurer) |
| W1 | Two bounds checks per entity (the cursor against both pools) | they are Rust's too | attribution `-DNOBOUNDS` | 1.01 (`d3_w1_group_nobounds`) |
| W2 | Ember's list layout, and the `nodes` list of references (section 1.3) | layout | `plain_w2_emberlist` as the baseline | as written 1.21 of plain C = **1.11** of same-layout C; this model: **1.07** (`d3_w2`) |
| W2 | The proven case rests on a whole-program fact: `nodes` is the only home of every Node and only grows. Phase 3 did not model the fallback | the fallback is the design's | `-DUNPROVEN`: Node gets a generation, `parent` a 12-byte watcher, each child a 16-byte watcher, a check per hop | **1.95** time, 1.49 memory (`d3_w2_unproven`). With "checked pointers"' packed 8-byte references instead of 12- and 16-byte watchers, the fallback compiles to `d2_w2_checked`: **1.58** |
| W3 | none: one generation compare per target use, on the line the update reads anyway | | the safety fixes below cost nothing here: no `update` can destroy an object (effect summaries) | **0.97** (`d3_w3`, unchanged) |
| W4 | A pin and an unpin around every one of 10,000,000 calls, because the closure's state lives inline in the list | the pin costs little, but it ties the state to the list | **Protect at the point of use**: state that is written after the closure may be removed lives in a box from the 32-byte slab; the list holds `{function, box}` (16 bytes); the dispatcher copies the value out and uses nothing after the call; the handler pins its own box across `d.unregister_current()` on its rare path, and a removal that finds it pinned frees it at the unpin. The list is never pinned: `remove` is a plain `memmove`, with no tombstones, overflow pages, settle or slow index path | **1.03 → 0.96** (`d3_w4`) |

Source: `runs/final_w1.txt` .. `final_w4.txt`; files `d3/w1_group.c`, `d3/w2.c`, `d3/w3.c`, `d3/w4.c`.

Memory, salvaged: W1 0.95, W2 1.12 (against same-layout C; the unproven build 1.49), W3 0.84, W4 1.05.

The W1 loop, unchanged for the user; what changed is what the compiler proves about it:

```ember
for e, pos, vel in reg.group[(Position, Velocity)]():
    pos.x += vel.dx                  # pos and vel are last used here ...
    pos.y += vel.dy
    if e.id() % 100 == s:
        reg.destroy(e)               # ... so nothing borrowed is live across the change: no pin
        n = reg.create()
        reg.emplace(n, Position(x=0.0, y=0.0))
        reg.emplace(n, Velocity(dx=1.0, dy=1.0))
```

```python
for e, (pos, vel) in reg.group(Position, Velocity):
    pos.x += vel.dx
    pos.y += vel.dy
    if e % 100 == s:
        reg.destroy(e)
        n = reg.create()
        reg.add(n, Position(0.0, 0.0))
        reg.add(n, Velocity(1.0, 1.0))
```

If the body used `pos` after `reg.destroy(e)`, the static check would fail and the pin would come back:
the fallback is still there for the program that needs it.

The same loop gives the same semantic rule in all three salvaged designs: destroying the CURRENT
entity takes effect at once; destroying another entity during the loop must be deferred to the loop's
end by the library (it publishes its cursor, one compare per destroy), or the swapped-in entity is
visited twice (EnTT documents this case as not allowed; `phase3-break-D1.md` N12). W1 destroys only the
current entity, so this is not in the measurements.

### 4.3 What made it unsafe: each critical and major hole, the fixed rule, and the attack

The breaker (`phase3-break-D3.md`) found five critical holes (1.1 to 1.5) and three major ones (1.6 to 1.8).
The programs are the breaker's; where Python's behaviour matters, `py_salvage.py` prints it.

| Hole | Fixed rule | Attack | Result | Cost |
|---|---|---|---|---|
| A reference (parameter, `self`, local, interior alias) used after a call that destroys its object (1.1, critical) | **Protect at the point of use**: a function that uses an object reference after a call whose effect summary may destroy objects of that class pins the object across the call (a 4-byte keep-alive count in such classes' headers). A destroy of a pinned object raises its generation at once, so watchers read `None`, and runs its whole destruction (`drop` and frees) at the last unpin. A dynamic call (virtual, `dyn`, closure) is assumed to destroy anything its types allow | the breaker's `hit(w, s)` calling `w.kill(s)` and then writing `s.hp = 0` | the write lands in the removed, not yet freed ship: Python's meaning (`py_salvage.py` item 7 prints `0` and `0` ships left) | two read-modify-writes per protected call; **in the measured `d3_w4`** (the handler's rare path). W3's update loop needs none: no override destroys |
| `pop()` under a pin hands out a value the tombstone still holds (1.2, critical) | Under a pin, a removal that returns a value owning memory hands out a copy and keeps the original in the tombstone until the settle; for an object element the home moves to the receiver and the tombstone becomes a watcher | the breaker's `it = bag.items[0]; bag.discard_last(); bag.empty_trash(); println(it.name)` | prints `sword`, as Python (item 8) | one copy per pinned `pop` of an owning value |
| Copying a value that holds an object's home makes two homes (1.3, critical) | Homes and watchers are both "checked pointers"' packed 64-bit references; a home has the owner bit set; a copy clears it; only a set bit destroys | the breaker's `a.push(s); b.push(s); a.clear(); b[0].leader.hp -= 1` | no double destroy; `b[0].leader` reads stale | one bit test per drop; a copy is a copy plus a mask |
| A stale watcher passes its check on an object of another class (1.4, critical) | Every slab slot keeps a generation word, whatever class is in it (a class without watchers still keeps the 4 bytes) | the breaker's Bullet and Tally sharing a 32-byte slot | the Tally's slot has a generation word: the check fails | 4 bytes for classes without watchers; in W2's Node it fills existing padding (40 bytes either way) |
| Scoped tasks and `@parallel` lost their race freedom with aliasing-xor-mutation (1.5, critical) | Compile time: a value a scoped task borrows is read-only in the parent until the join, and a task may borrow only values that hold no object references unless those objects are frozen; `@parallel` keeps `[BRW-11]`'s distinctness for spans, or tests the two spans for overlap at entry (two compares) and runs serially if they overlap | the breaker's `xs[0] = 10` while a task reads `xs`; `shift_left(xs.as_mut_span(), xs.as_span())` | the first is refused; the second runs serially | refusals (phase 1's threads annoyance A15, as in Rust); two compares per parallel loop |
| Hot reload widens an effect summary its callers relied on (1.6) | A reload whose new summary is wider than the one its callers were compiled against is refused, naming the callers to rebuild | the breaker's `log_hit` that starts pushing to `w.enemies` | refused | refused reloads |
| A settle runs `drop` code that changes the container being settled (1.7) | The settle keeps the pin while it destroys removed elements and repeats until no tombstone is left | the breaker's `tick` whose drops remove other objects | the nested removals become tombstones and the next round applies them | none on fast paths |
| The pin covers append and remove only (1.8) | Pin counts live in the heap buffer's own header, which does not move when elements move (sort, swap, insert, element assignment); under a pin, inserts and reorderings are applied at the settle, and an element assigned over goes to the side record until then | the breaker's `l3`, `l5` views and `b.reorder()` | each view pins its own label buffer; the reorder moves only the elements; nothing pinned is freed | one header load on changes; the slow index path while changes are pending |

### 4.4 Python habits it breaks

The breaker's 2.1: the home is the first place an object is stored, so the ordinary parent-pointer idiom
makes the child the owner of the root:

```ember
class Node:
    parent: Option[Node] = None
    children: Array[Node] = []

root = Node()
c = Node()
root.children.push(c)       # c's home: root.children
c.parent = Some(root)       # root's first store after its local: c.parent becomes root's home
root.children.clear()       # destroys c, and with it root
println(len(root.children)) # root names a place inside a destroyed object
```

```python
class Node:
    def __init__(self): self.parent = None; self.children = []

root = Node(); c = Node()
root.children.append(c)
c.parent = root
root.children.clear()
print(len(root.children), c.parent is root)   # 0 True   (py_salvage.py item 3)
```

Fixes: "checked pointers"' `own` word on owning fields (`children: list[own Node]`) removes the
dependence on line order, at the cost of one word per owning field; nothing removes it while keeping
both "no annotation" and "no counts". With `own`, the residue is "checked pointers"' one-owner deaths
(section 3.4), and the breaker's 2.2 (struct or class changes what panics) stays: a struct pushed to two
lists is copied, a class is owned once (item 11: Python keeps it alive in `b`).

### 4.5 Verdict: partly saved

- **Speed: saved, with the same W2 condition.** W1 1.06 (under 15%), W2 1.07 of same-layout C with
  the whole-program fact, 1.95 without it (1.58 with packed references), W3 0.97, W4 0.96.
- **Safety: saved on paper**, with fixes that borrow from the other designs: packed references with an
  owner bit and a generation word in every slot ("checked pointers"), protect-at-the-point-of-use pins
  (made in this salvage), compile-time thread rules (Rust's). The keep-alive pin runs in the measured
  W4 model.
- **Python-likeness: not as promised.** The design's own idea ("never an error, no annotations") does
  not survive: threads need compile-time refusals, ownership needs `own` (or line order decides deaths),
  and a distant edit still changes a class's layout and checks (the breaker's 2.8).

**Good parts**, with where they came from:

1. **Static borrowing first, with a report of every fallback** (`ember inspect --checks`) and `@static`
   to turn fallbacks into errors. From this design. The W1 result shows that non-lexical borrowing plus
   an index cursor is often enough, with no fallback at all.
2. **Watchers proven raw when the homes cannot let go**: W2 at 1.07 of same-layout C, the best W2 of the
   three. From this design; it is a whole-program fact.
3. **The pin protocol** (tombstones, overflow pages, settle) for the program that really does hold a
   reference across a change; with the pin count in the buffer's header (1.8's fix) it covers reordering.
   Cheap where measured: the phase-3 W4 model pinned every call at 1.03. From this design.
4. **Effect summaries per function** (what it may add, remove or destroy), which decide where any of the
   three designs needs a check, a refetch or a pin. From this design.
5. **Generation compare per target use on the target's own cache line**: W3 at 0.97 of plain C with
   observer lists, with a 40-byte object against plain C's 56. From this design.
6. **Overflow pages and tombstones for pinned maps** (used by the "copy instead of share" B6 fix). From
   this design.
7. **One object kind** instead of class, `Box`, `Weak`, `Shared` and `Handle`, and a 0-to-8-byte header.
   From this design.

---

## 5. What the salvage shows across the three

1. **After the fixes the three compile the four workloads to nearly the same machine code**: W1 1.06 to
   1.21, W3 0.82 to 0.97, W4 0.96 to 1.04 of plain C (`runs/SUMMARY.txt`; W2 in the next item). Last
   night's spread (W1 1.39 to 2.34 of the sparse-set C) was implementation.
2. **What still costs, in all three, is a check that only a whole-program fact removes.** W2's parent
   walk: +18% for a weak test, +27% for a generation check with packed references, +83% with 12- and
   16-byte watchers (each against the same design's proven build). Safe Rust pays more here (1.48 of
   plain C, 1.35 of same-layout C) because its arena checks bounds and computes an index on every hop.
   A fact that does not depend on the whole program is an explicit arena (Ember's `[ARN-*]`): nodes
   allocated in an arena that is freed as a whole need no check by type. Not measured here.
3. **Most fixed safety rules are compile time or rare-path costs**, and several are shared: protect at
   the point of use (all three), packed references with an owner bit (two), a generator or frame that
   holds a borrowed reference is consumed where it is made (two), compile-time freezing for scoped tasks
   (all three), refusing to send values whose references leave the moved graph (two).
4. **What separates them is how a Python program fails**: silently ("copy instead of share": lists are
   values), or with a deterministic panic where Python keeps an object alive (the other two). The owner's
   question from last night stands, now without a speed argument on either side: counted objects that stay
   alive while referenced measured fastest on W3 and on par on W4 here.

Open questions for the owner (not ruled here):

1. Lists and dicts: values (copy on write, not Python's aliasing) or shared objects?
2. A referenced object whose owner lets go: kept alive (counting, Python's meaning) or dead (`None` or
   a panic at the use)?
3. W2-style structures: accept a whole-program fact for check-free parent walks, or ask for an explicit
   arena?

---

## 6. Files and commands

Everything under `C:/Users/ism19/Code/ember/target/redesign2/salvage/`:

| File | What it is |
|---|---|
| `sh.h` | includes the shared harness `target/redesign/bench/harness.h`; adds an optional phase timer (`-DPHASES`, not used in timed builds) |
| `plain/w1_group.c` | plain C, W1 in EnTT's owning-group layout (new baseline) |
| `plain/w1_arch.c`, `w2.c`, `w3.c`, `w4.c` | phase 3's plain baselines, include path changed |
| `plain/w2_emberlist.c`, `plain/w4_env32.c` | attribution variants (Ember's list layout; a 32-byte environment) |
| `rust/harness.rs`, `rust/w1_group.rs`, `w2.rs`, `w3.rs`, `w4.rs` | safe Rust baselines |
| `d1/d1rt.h`, `d1/w1_group.c`, `w2.c`, `w3.c`, `w4.c` | "copy instead of share", salvaged; switches `D1_PROVEN`, `NOINLINE`, `D1_MALLOC`, `D1_TYPEPTR`, `MALLOCBOX`, `ONECLOSURE`, `ARRAY3` |
| `d2/w1_group.c`, `w2.c`, `w3.c`, `w4.c` | "checked pointers", salvaged; switch `CHECKED` |
| `d3/w1_group.c`, `w2.c`, `w3.c`, `w4.c` | "today's rules, but never an error", salvaged; switches `NOBOUNDS`, `UNPROVEN` |
| `build.sh` | builds all 50 programs (the phase-3 models are rebuilt from `target/redesign/` unchanged) |
| `runs/ab.ps1`, `runs/final.ps1` | the pinned, interleaved runner and the batch behind every table |
| `runs/final_w1.txt` .. `final_w4.txt` | raw lines, one per run |
| `runs/summ.py`, `runs/SUMMARY.txt` | medians, ratios to each baseline, spreads, checksum agreement |
| `runs/w1.txt`, `w2.txt`, `w2b.txt`, `w3.txt`, `w4.txt`, `w4b.txt` | earlier batches (5 or 7 rounds) made while the models were still changing; not used in the tables |
| `py_salvage.py` | the Python 3.14.2 behaviours quoted (`python -I py_salvage.py`) |

To reproduce: `bash build.sh`; then in PowerShell `& runs/final.ps1`; then
`python -I runs/summ.py runs/final_w2.txt plain_w2_emberlist` (or any baseline name).

Sources outside my runs: `phase3-measurements.md` (phase 3's method and numbers, consistent with my
re-runs of the same models: e.g. W3 1.16/1.05/0.97 there, 1.15/1.05/0.97 here), `phase3-break-D1.md`,
`phase3-break-D2.md`, `phase3-break-D3.md` (holes and their labels), `phase2-rc-elision.md` lines 299-300
(0.84 ns per plain count pair, 5.8 ns per atomic pair), `phase2-vale-austral-pony.md` (generational
references), `phase2-hylo-csharp.md` (projections), the spec `docs/spec-source/Ember_v0.9.10_Hardened_35.md`
(`[HND-3]` line 3095, `[ARN-11]` line 2976, `[RC-3]` line 2657, `[THR-11]` line 3564).
