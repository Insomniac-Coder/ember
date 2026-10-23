# Phase 3: breaking design D2 (generational references and stable stores)

Written 2026-10-24 by the phase-3 adversary agent for D2. Target: `phase3-design-D2.md` and its C
models in `target/redesign/bench/D2/w1.c` to `w4.c`.

How this was done, so every claim can be traced:

- I read the whole design, all four models, `harness.h`, the D1 and D3 W3 models (for comparison only),
  the spec rules I cite by id (`docs/spec-source/Ember_v0.9.10_Hardened_35.md`), and the phase-2
  sections the design quotes (`phase2-vale-austral-pony.md` lines 291-312, `phase2-hylo-csharp.md`
  lines 402-414, `phase2-rc-elision.md` line 299). The design quotes those numbers correctly.
- I did not change the models. I copied them into `target/redesign/break-D2/` and changed only the
  copies, one thing each (file names below), built them with clang 22.1.8 `-O2` (the design's compiler
  and flags), and ran the original and the changed copy alternately, 3 runs each, pinned to logical
  CPU 1 (a P-core) with `start /b /wait /affinity 2`, on mains power (`Win32_Battery.BatteryStatus` = 2
  checked before every batch). These are my runs, not the measurer's: three runs, one machine.
- "Python does X" claims were checked by running the Python programs with Python 3.14.2 (`python -I`).
- No Ember program was run. `ember check` cannot parse D2's new syntax, so it was not used. Programs
  below are in D2's syntax as section 3 of the design writes it.
- Web: the Windows address-space limit and the EnTT in-place-delete fact were checked on the web
  (sources at the end). No tool was missing.

Severity words: **critical** = a safety hole in a core rule, or a gap whose every fix gives up one of
the owner's three goals; **major** = a safety hole with a cheap fix, or an annoyance a Python
programmer will hit often; **minor** = the rest.

---

## 0. Verdict

D2's core mechanism holds up where the design applies it: a checked `{slot, generation}` reference
into a store that never moves cannot read freed memory, and the four workloads really run with
almost no checks (I confirmed in the models: the only generation check that runs often is W3's
target check). But the design is not yet memory safe as written, and it is less Pythonic than its
annoyance table says:

1. **Two critical gaps.** (a) References to values that live on the stack (a struct local, the `self`
   of a mutating struct method) are not specified at all, and the design allows storing and
   returning references "with no restriction"; every way to close this gives up one goal (a dangling
   pointer, a borrow-style escape rule, or a heap cell per such local). (b) The thread story is broken:
   a reference is `{slot, generation}` with no store or thread in it, stores are per thread, so a
   reference inside a moved graph, a frozen object or a `Mutex` is resolved against the wrong thread's
   store. Generations start at 1 in every store, so the check often passes on the wrong object.
2. **Six major holes with cheap fixes**: proofs P3/P4 are false during teardown and in `drop`; a loop
   whose body destroys the list it iterates reads a freed buffer (the design's own A32 sketch does
   this); L5's once-per-iteration buffer reload is too coarse; `Store[T]` element references hold raw
   addresses; O6 makes ownership a per-value run-time property of structs, and the models' `destroy`
   does not check the generation, so a double destroy corrupts the free list; and relaxing `E2100`
   over zeroed slots breaks range types, whose range facts remove bounds checks.
3. **The worst annoyance the design does not admit**: one owner per string and list makes the most
   common Python lines panic: `old = p.name; p.name = "Bob"; print(old)`, and the drain idiom
   `pending = self.pending; self.pending = []; for job in pending: ...`. The design marks A23 (moving
   a field out) "removed".
4. **Model shortcuts**: W4 keeps the handler list inside the dispatcher's slot (a separate list object,
   as the design defines lists, cost +26% in my runs); W3 removes the loop's element in O(1) where the
   program's `objects.remove(o)` is a search (+5% to +9%); W2 keeps each node's child list inside the
   node's slot (the faithful layout uses 15.6 MB more memory, +20%, but runs 16% faster, so the model
   is wrong in both directions). Several smaller ones are listed in section 3.

---

## 1. Memory safety: programs D2's rules accept that still break it

### H1 (critical). References to stack values are unspecified, and storing them is unrestricted

Section 1.1 says values (structs, numbers) "live inline (in a variable, a field, a list's buffer)",
`ref x` "names a reference to a value that lives elsewhere (a field, a store element)", and
"references may be copied, stored in fields and collections, captured, returned ... with no
restriction". Section 1.4 gives a layout for a reference into a store slot (16 bytes, the slot's
address and its expected occupant), but none for a reference to a local variable, which has no slot
and no generation. Searching the design for "stack", "frame", "ref self" or "escape" finds only the
closure and async paragraphs. Yet a mutating method on a struct needs a reference to wherever the
struct is, and a struct in a local is on the stack:

```ember
struct Vec2:
    x: f32
    y: f32
    fn scale(ref self, k: f32):        # D2 does not say how a mutating struct method is spelled
        self.x *= k
        LAST.v = ref self              # storing a reference: "no restriction" (1.1)

class Holder:
    v: Option[ref Vec2] = None
LAST = Holder()                        # module variable

fn f():
    p = Vec2(x=1.0, y=2.0)             # a local value: no store slot, no generation
    p.scale(2.0)

fn main():
    f()                                # f's frame is gone
    other_work()                       # reuses the same stack bytes
    LAST.v.x = 99.0                    # writes into whatever now occupies f's old frame
```

Why the rules let it through: nothing in O1-O7, G1-G3 or P1-P6 mentions locals as reference
targets, and the only compile-time ownership errors (O7) are about objects.

Fixes, each with its cost:
- (a) References to stack values are second-class: they may be passed down but not stored or returned.
  That is Ember's current view rule (`[TYP-15]`, which 1.9 lists as removed), and A5/A6 return for
  values.
- (b) Any local whose reference may escape lives in a heap cell (a store slot), found by escape
  analysis. Cost: an allocation per such local per call wherever the analysis cannot see the callee
  (virtual calls, closures, separately compiled code). That is the Java/Go trade-off, not C's speed.
- (c) Every frame is a store object with a generation (the design already does this for suspended
  frames, A14). Cost: a slot allocation and a generation bump per call.

The design must pick one; today it is a dangling pointer.

### H2 (critical). Per-thread stores with untagged references: moved, frozen and locked data resolve against the wrong store

A reference is `{u32 slot, u32 generation}` and "a slot number turns into an address with ...
`base + index * slot_size`, where `base` is loaded once per function" from the current thread's
store (1.2, 1.4). Section 1.7: "stores are per thread, so references do not cross threads. An object
reaches another thread by being moved (with everything it owns; it is relocated into the receiver's
stores and the sender's references go stale) or frozen". Three programs it accepts:

```ember
class Logger:
    lines: list[String] = []
    fn write(self, s: String):
        self.lines.append(s)

class Job:
    data: list[int] = []
    log: Logger                    # a reference; the Logger is owned by main, not by the Job

fn work(j: Job):
    j.log.write("started")         # {slot 1, gen 1} resolved in the WORKER's Logger store

fn main():
    logger = Logger()
    j = Job(data=[1, 2, 3], log=logger)
    t = spawn(work, j)             # j moves "with everything it owns": the Logger is not owned, it stays
    logger.write("main")
    t.join()
```

- The moved `Job` carries a reference out of the moved graph. In the worker, `base` is the worker's
  Logger store. If the worker has created one Logger, its slot 1 has generation 1 too (every store's
  first object gets generation 1), the check passes and `write` appends to the wrong Logger. If the
  worker has none, the address is in a reserved but uncommitted page: an access violation, not a
  deterministic panic. If instead the implementation tags moved objects with the sender's store, both
  threads write the same `lines` list at once, and one `append` can free the buffer the other is
  writing: a data race and a use-after-free.
- Frozen objects: "shared read-only" means another thread holds references to them, and those
  references are resolved against that thread's store. The design never says where a frozen object
  lives or how its reference names that place.
- `Mutex`: "Shared mutable state between threads lives in a `Mutex`". A `Mutex[list[String]]` holds a
  reference to a list object, which lives in one thread's store; the thread that locks it resolves the
  reference in its own store. Every useful protected value in a Python-style program (a list, a dict,
  a string, a class instance) is an object, so this is not an edge case.

Why the rules let it through: the move rule only covers references into the moved graph (they go
stale); it says nothing of references out of it, nor of where frozen or locked objects live. Section
1.7 calls the thread part "design only", but section 2 still claims A15 is "softened" with
`Send`/`Sync` "inferred, never written".

Fixes, each with its cost:
- A compile-time isolation check at `spawn` and `freeze` (Pony's `iso` and `val`): the moved graph
  must not refer outside itself, and a frozen graph must be deeply immutable. That brings back a
  reference-capability discipline, which is an A15 annoyance in another form.
- Or a run-time walk at the move that relocates the graph and sets every outward reference to `None`
  (cost proportional to the graph; a non-optional outward reference then panics in the receiver).
- Plus, for frozen and locked objects: a process-wide shared store, a reference that says which store
  it points into (a tag bit, which shrinks the slot or generation field, or a second reference type,
  which is A10 noise), and reclamation that cannot race the check (atomic generation updates and
  deferred reuse). Every use of such a reference pays for that.

### H3 (major, cheap fix). P3 and P4 are false during teardown and inside `drop`

P4: "Objects owned by `main`'s top-level locals that `main` never reassigns or deletes, and module
variables, are live whenever code runs on the main thread." P3: grow-only owners need no check
"while it lives". Both fail once `main` returns, because drops still run on the main thread:

```ember
class Tally:
    n: int = 0

class Reporter:
    t: Option[Tally] = None
    fn drop(self):
        if self.t is not None:     # under P4 this is only "generation != 0" (as w2.c tests parent.gen != 0)
            self.t.n += 1          # P4: no check

REPORTER = Reporter()              # module variable: destroyed after main returns

fn main():
    tally = Tally()                # main's top-level local, never reassigned or deleted: P4 applies
    REPORTER.t = tally
    # main returns: `tally` is destroyed (O4). Then module teardown runs REPORTER.drop(),
    # which writes into the dead slot.
```

The same happens with P3 when a `Node` has a `drop` that reads its parent: the owning list's own
teardown destroys node 0 before node 5, and node 5's drop reads node 0 with no check.

Why it is worse than a wrong read: section 1.2 keeps the free list "threaded through the dead slots",
and the models keep the link in the first field after the generation (`w3.c`: `cls`, `w4.c`:
`counter`, `w1.c`: the value bytes). An unchecked write into a dead slot rewrites the link, and the
next allocation from that store hands out a slot that is still live, or one past the end.

Fix: P3 and P4 never apply in a `drop` body or anything a `drop` can call, and P3's "outlives every
use" counts uses during the owner's own teardown. Cost: checks inside drops only. Hardening worth
adding anyway (H10).

### H4 (major, cheap fix). A loop whose body destroys the list it iterates reads a freed buffer

L5: "In a loop that may [change the container], the length and the buffer address are re-read each
iteration". Re-read from the list object, but the body may destroy that object by overwriting its
owner:

```ember
class Ui:
    handlers: list[own fn(Ui)] = []

    fn fire(self):
        for h in self.handlers:
            h(self)

fn reset(ui: Ui):
    ui.handlers = []               # O4: the old list object is destroyed now, its buffer freed
```

The design's own C sketch for this program (section 4, A32) is:

```c
for (i64 i = 0; i < n0; i++) {
    Callable c = ui_list->ptr[i];                    /* buffer re-read */
    ...
```

with no generation check on `ui_list`. After `reset` runs from a handler, `ui_list` points at a dead
slot whose `ptr` is the freed buffer (or the free-list link): the next iteration reads freed memory.
O5 protects only "an object whose own method or closure body is running", not a container being
iterated, and L4's iteration count is tested only by `remove`.

Python iterates the old list to the end (checked: a loop that sets `self.items = []` on its first
element still visits all three). Fix: the iteration count also pins the container (a destroy of a
list whose count is not zero is deferred to the end of the outermost loop, as O5 defers objects),
which gives Python's meaning. Cost: one test on each list destroy. A generation check per iteration
also works but panics where Python does not.

### H5 (major, cheap fix). L5 reloads the buffer once per iteration; a call in the middle can reallocate it

G3 says a reference to a value in a contiguous list is "re-resolved with a bounds check at each use";
L5 says the buffer and length are "re-read each iteration". If the loop variable over a list of
structs names the element in place (the design does not say whether `for p in ps` copies or names;
W1's view names components in place), a literal implementation of L5 keeps the element address for
the whole iteration:

```ember
struct Particle:
    x: f32
    y: f32

fn step(ps: list[Particle]):
    for p in ps:
        p.x += 1.0
        if p.x > 10.0:
            ps.append(Particle(x=0.0, y=0.0))   # allowed (L2); may move the buffer
        p.y += 1.0                              # written through this iteration's old buffer address
```

Fix: state that G3 governs (re-resolve after any call that may insert). Cost: a reload and a bounds
check per use after such a call. W1's view is not affected, because store slots never move.

### H6 (major). `Store[T]` element references hold raw slot addresses; a destroyed store leaks or dangles

Section 1.4: "reference to a value inside a slot (a store element, a field): 16 bytes: the slot's
address and the occupant it expects"; section 3.5 builds `Registry.get` on it, with the entity id as
the occupant. Section 1.2's "a store keeps its high-water mark" is about the per-class stores; a
`Store[T]` is a library container that programs create and destroy (one per pool, so one per
component type per registry):

```ember
fn load_level() -> ref Position:
    reg = Registry()                      # its pools own Store[Slot[Position]] objects
    e = reg.create()                      # entity id 0: ids are serials per registry
    reg.emplace(e, Position(x=1.0, y=2.0))
    return reg.get[Position](e)           # {slot address, expected owner 0}

fn main():
    p = load_level()                      # reg destroyed: its stores are destroyed
    reg2 = Registry()                     # may get the same address range back
    e2 = reg2.create()                    # id 0 again
    reg2.emplace(e2, Velocity(dx=0.0, dy=0.0))
    p.x += 1.0                            # check: owner word at the old address == 0: passes
```

Either a destroyed `Store[T]` never gives back its address range (every level load leaks its pools'
reserved and committed memory), or it does, and the check itself reads memory that is unmapped (a
crash, not a panic) or reused by another store, where small entity ids like 0 match by chance: the
write lands in another registry's component, possibly of another type.

Fix: make the reference `{store reference, index, occupant}`, checking the store's own generation
first (two dependent checks per unproven use), and make entity serials process-wide so an occupant
never repeats across stores. Cost: one more load and compare where no proof applies.

### H7 (major). O6 makes ownership a per-value run-time property, and the modelled `destroy` trusts it

O6: "Copying a struct that owns objects copies references to them; the original keeps ownership,
except at the original's last use, where the copy takes it."

```ember
struct Msg:
    to: String              # owns its String (1.1: "a struct ... may own objects")
    body: String

fn main():
    inbox: list[Msg] = []
    m = Msg(to="ann", body="hi")
    inbox.append(m)         # not m's last use: inbox[0] holds references (O6)
    println(m.to)
    m2 = Msg(to="bob", body="yo")
    inbox.append(m2)        # m2's last use: inbox[1] owns its strings
    # scope end: inbox[0] must not destroy its strings, inbox[1] must
```

Two elements of one `list[Msg]` differ in ownership, so each struct value needs run-time owner bits
(memory per value, a test per drop), which the design's cost table (1.8) does not list. If the
implementation instead drops by static type, `m`'s strings are destroyed twice. In the models, the
destroy path takes a slot index and does not check the generation (`w3.c` `obj_destroy(u32 s)`,
`w4.c` `hstate_free_now(u32 s)`): a second destroy pushes the same slot onto the free list twice, so
two later objects share one slot (type confusion inside a hierarchy).

Fixes: make a struct copy at a non-last use a compile-time error with the fix `.copy()` (deep copy),
which is the A11 `clone()` annoyance back, or copy deeply always (allocation per copy), or carry
owner bits per value. In every case, destroy through `{slot, generation}` and check the generation
(one compare per destroy), so a double destroy is a no-op instead of a corrupted free list.

### H8 (major, cheap fix). Zeroed slots plus a relaxed `E2100` break range types, and range facts remove bounds checks

A18's row: "Store slots start zeroed ... `E2100` can be relaxed to a lint." Ember keeps range types
(`[RNG-1]`), whose facts remove bounds and overflow checks (`[RNG-4]`, and the check table in Part I:
"Index in bounds ... the index comes from ... a range fact"). A zero is outside many ranges; the spec
already says so (`[ARN-11]`: "a range excluding zero, or an enum whose zero discriminant is invalid
is never `Zeroable`").

```ember
type Lane = int in 1 ..= 4

class Road:
    cars: [int; 4]
    lane: Lane                       # no default

    fn init(self, l: Lane):
        self.count_here()            # reads self.lane before it is set: A18 says it reads 0
        self.lane = l

    fn count_here(self) -> int:
        return self.cars[self.lane - 1]   # range fact: 0 <= lane - 1 <= 3, so no bounds check (RNG-4)
```

With `lane` = 0 the index is -1: an out-of-bounds read with the check removed. An enum with
discriminants starting at 1 read in an exhaustive `match` (no default arm) can jump outside its table
the same way.

Fix: keep `E2100` as an error for every field whose type is not `Zeroable` (`[ARN-11]` already
defines the set). No run-time cost; A18 is then only partly removed.

### H9 (minor). List slices: "check the object, then bounds" does not say which length

Section 1.4: a slice of a list is `{object reference, u32 start, u32 length}`, used by "check the
object, then bounds". The list object survives `clear()`, so its generation does not change:

```ember
items = [1, 2, 3, 4, 5]
s = items[1:4]          # a view {items, 1, 3} in D2 (a copy in Python)
items.clear()           # the list object lives on; its buffer may shrink or be freed
println(s[2])           # within s's own length 3: must still be refused
```

If the bound is the slice's own length, this reads past the list's end. Fix: bound by
`min(slice length, list length - start)`, one more load and compare per use. (Semantics: Python
prints 4 here because its slice is a copy; D2 must panic.)

### H10 (minor, hardening). Any proof bug becomes an arbitrary write

Three choices multiply the damage of any mistake in P1-P6 (or of H3, H7, or C code returning bad
user data): the free-list link lives in the dead slot's body (1.2); the models' allocator trusts the
popped index; the C trampoline checks the generation but not that the slot index is below the
store's length (1.7). A forged or stale index turns `base + slot * size` into any address in a
window of 2^32 slots, far beyond the reservation (`w2.c` reserves 256 MiB; 2^32 x 40 bytes is
160 GiB). Fix: bound-check the index on free-list pop and in the trampoline (one compare each),
check the generation on destroy (H7). Cost: negligible.

### What I tried that held

- A stale non-optional reference: panics (G2). A stale optional: reads `None`. Slot reuse by another
  subclass: the generation differs. Generation exhaustion: slots retire (`w3.c`, `w4.c` do it).
- Entity reuse in W1: ids are never reused, so a stale component reference never matches a new owner
  (true within one registry; see H6 across registries).
- Destroying the running handler in W4: O5's pin works in `w4.c` (the pin, the `DOOMED` mark, the
  free on the last return). Re-registering itself (`d.unregister(handler); d.register(handler)`) is
  refused by O7 at compile time, so the window cannot be used to put a doomed object into an owning
  list.
- Aliasing writes (A3): writing plain data frees nothing; an owning field overwritten through one
  alias is caught by the generation check on the other alias's derived reference, provided effect
  analysis marks the overwrite as a destroy.

---

## 2. Annoyances D2 keeps or adds that the design does not admit

The new ones below are filed under the existing ids (my task gave no range for new ids). "Python"
lines were run with Python 3.14.2.

### N1 (major; A23, A11). Alias, then overwrite the owner: the most common Python lines panic

```ember
class Person:
    name: String

fn rename(p: Person, new_name: String):
    old = p.name                 # a reference (O3: storing anywhere but an owning place)
    p.name = new_name            # O4: the owner is overwritten, so the old String is destroyed
    log(f"renamed {old} to {new_name}")      # stale reference: panic

class Worker:
    pending: list[Job] = []
    fn flush(self):
        pending = self.pending   # a reference
        self.pending = []        # destroys the list `pending` refers to
        for job in pending:      # panic
            job.run()
```

Python keeps the old string and drains the queue (both checked). The design's A23 row says moving a field out
is "removed" because "`self.state = Idle` destroys the old state object"; that destruction is
exactly what breaks the Python spelling. Section 6.1 item 1 admits one-owner deaths only for the
filtered-copy pattern. Fixes: (a) define "read a field into a local, then overwrite the field in the
same function" as a move into the local (covers both programs above; does not cover an overwrite done
by a callee); (b) make `String` a value with a shared immutable buffer, which needs a count (what D2
removed) or a copy per read.

### N2 (major; A12). Two closures sharing a written local, and a debug print that changes ownership

1.7: a written capture of an escaping closure "lives in a small cell object owned by the declaring
scope; if the closure is that variable's only user after it is created, the closure owns it."

```ember
fn make_counter():
    count = 0
    fn inc():
        count += 1
    fn get() -> int:
        return count
    return (inc, get)            # two users: the cell is owned by make_counter's scope, dead on return

fn setup(btn: Button):
    clicks = 0
    btn.on_click(fn(): clicks += 1)   # the closure is the only user: it owns `clicks`, fine ...
    println(clicks)                    # ... until this debug line: now the scope owns it, and the
                                       # first click after setup returns panics
```

Python: `make_counter` works (prints 2 after two calls), and a debug print never changes behaviour.
The A12 row mentions the stale-cell panic in its cost column; section 6.1 does not, and the
action-at-a-distance (adding a read turns a working program into a later panic) is not mentioned
anywhere. Fix: a cell shared by several escaping closures is kept alive while any of them lives,
which is a count on cells only (cheap, rare), or a compile-time error when an escaping closure
captures a cell owned by a scope that ends first (always detectable here: the closure is returned or
stored past the scope).

### N3 (major; A11, A10). Ownership of parameters and returns is inferred from bodies

W4's `register(self, h: fn(Event))` "takes ownership (inferred)" because its body stores `h` into an
owning list. Programs Python programmers write:

```ember
class Keeper:
    kept: list[own Node] = []
    fn maybe_keep(self, n: Node):
        if n.value > 5:
            self.kept.append(n)   # owning store on one path only

fn main():
    k = Keeper()
    a = Node(value=3)
    k.maybe_keep(a)               # if this call takes ownership, `a` dies when maybe_keep returns
    println(a.value)              # panic (Python prints 3)

cache: dict[int, own Node] = {}
fn get_or_make(i: int) -> Node:
    if i in cache:
        return cache[i]           # a reference
    n = Node(value=i)
    cache[i] = n
    return Node(value=-i) if i < 0 else n   # an owner on one path, a reference on another
```

The design does not say whether `maybe_keep` takes ownership always (then the object dies on the
not-kept path), never (then its `append` is an O7 error inside a library), or at run time (a flag per
call, not in 1.8). Same for a function that returns an owner on one path and a reference on another.
O3 also contradicts the design's own W2 program: O3 moves ownership "at the owner's last use", but W2
does `nodes.append(n)` and then `p.children.append(n)`, so `nodes.append(n)` is not `n`'s last use;
read literally, O7 refuses section 3.2. The rule has to say that any store into an owning place
moves ownership and leaves the name as a reference; then `k.keep(a); k2.keep(a)` is an O7 error
(admitted in A11 as "needs `.copy()`"), and the conditional cases above still need a ruling.

### N4 (major; A22, A25). Slices of a temporary die at the end of the statement

O2: a temporary lives on only if "a local holds a reference or slice into it". Indirect holders are
not covered:

```ember
fn first_word(line: str) -> str:
    return line.lower().split()[0]     # a slice of the temporary lower(): destroyed at the end of the statement

fn words_of(line: str) -> list[str]:
    words = line.lower().split()       # `words` holds a list of slices, not a slice: O2 does not apply
    return words
```

Python returns `hello`. The design's A25 example binds `lower = line.lower()` first, which is the
spelling that works; the one-liner every Python programmer writes panics at first use. Fixes: O2
extended to "while anything refers to it" (a count, or a scope rule as complex as region inference),
or `split()` returns owning strings (an allocation per word, as Python does, which gives up the A25
slice savings).

### N5 (minor; A3, A1). Struct elements copy: `p = ps[i]; p.x += 1` changes nothing

Values "are copied on assignment as in C" (1.1). A Python programmer coming from objects writes:

```ember
p = particles[3]
p.x += 1.0                    # changes a copy; particles[3] is unchanged, silently
for p in particles:
    p.x += 1.0                # copy or in place? the design does not say
```

C# and Swift warn or refuse here; D2 says nothing. Fix: a warning when a copied struct local is
written and then never read, and a stated rule for the loop variable (in place, which makes H5's fix
necessary).

### N6 (minor; A8). Identity sets and dicts do not keep their keys alive

```ember
seen: set[Node] = set()
seen.add(Node(value=7))       # O7 refuses this one ("nothing would own it") ...
n = Node(value=8)
seen.add(n)                   # ... but this compiles: a reference
del n                         # Python only unbinds; D2 destroys (O4)
for x in seen:
    println(x.value)          # panic; and `stale in seen` is still True (same bits)
```

Python prints 7 (checked). `del` destroying the object is itself a surprise for a Python programmer
(Python's `del` removes a name).

### N7 (minor; A4). Tombstones change what a callee sees, and nested store loops share one cursor

L4 makes a list "being iterated" a global state that changes indexing everywhere:

```ember
fn first_after_pop(xs: list[int]) -> int:
    xs.pop(0)
    return xs[0]              # inside a caller's `for x in xs`: index 0 is a tombstone

for x in items:
    if x == 1:
        println(first_after_pop(items))   # Python prints 2
```

The design does not say what a tombstone is in a `list[int]` (an integer has no spare value, so a
side bitmap is needed), what `len()` returns during the loop, or what `insert(0, v)`, `sort()` and
`reverse()` do then. A generator suspended inside a `for` over a list (A14) keeps the list in this
state indefinitely. Separately, `w1.c` has one `iter_limit` per pool; a view nested inside another
view over the same pool resets it to `UINT32_MAX` when the inner loop ends, so the outer loop's
spawns can take free slots ahead of the outer cursor and be visited in the same step, which breaks
L1 (not a memory hole). Fix: a small stack of cursors per store (the minimum is the limit).

### N8 (minor; A10, A17). Performance cliffs and capacity limits nobody sees

- P3 holds only if every owner of `Node` anywhere in the program is a grow-only owning collection. One
  unrelated `tmp = Node(value=0)` in any function, or one `nodes[i] = Node(...)`, puts a generation
  check back on every parent hop in W2. Only `ember inspect --checks` shows it.
- Stores are reserved address ranges per class hierarchy per thread. The user part of the address
  space on Windows x64 is 128 TB (Microsoft, source below). 2,000 classes and closure types times
  16 threads times 4 GiB each is 125 TiB, so each store gets a few GiB at most, and a class whose
  population exceeds it panics. In the model the limit is far lower: `w2.c` reserves 256 MiB for 40-byte
  nodes, at most 6,710,886 nodes; W2 with N = 10,000,000 would panic "store capacity exhausted".
  `w1.c`'s 32 MiB sparse range caps a run at 8,388,608 entity ids; W1 issues 1,856,653 (my run of
  `w1_count.c`), so 450 steps of the same churn would hit it. Python has no per-class limit.
- Per-thread stores keep their high-water memory per class per thread (6.1 item 6 admits per store,
  not the thread multiple).

### N9 (minor; A12). `h.counter` is not Python, and `fn(Event)` cannot type it

Section 3.4 reads `h.counter` on elements of `list[own fn(Event)]`, "as Python's function attributes
are". Python does not expose captured variables as attributes (checked: `hasattr(h, "c")` is
`False`; the value is in `h.__closure__[0].cell_contents`). And a `fn(Event)` type does not know which
captures its value has, so the checksum loop does not type-check unless the element type is the
specific closure type of `make_handler`, which the program does not write.

### N10 (minor). "No borrow, lifetime, move or exclusivity errors" is overstated

What remains at compile time: O7, which is flow-sensitive (`if c: a_list.append(n)` then
`b_list.append(n)` is a "possibly moved" error by another name); the A33 rule that a reference may
not leave a `with` block, which is an escape (lifetime) rule; and whichever of H1's and H2's fixes is
chosen (an escape rule for stack references, an isolation rule for threads).

---

## 3. The C models against the design and the workloads

All four models compute the specified work; my rebuilds print the same checksums as the design's
table (W1 2111831520, W2 647034814, W3 9288233970, W4 34946993000). W3 destroys 9,173 objects in
total (my run of `w3_scan.c`), W1 issues 1,856,653 entity ids (my run of `w1_count.c`), matching the
design's figure. The random numbers are drawn in the specified order in every model. The design's
reading of W3's "kind = id % 3" as the new object's id is the same in D1 (`D1/w3.c` line 140) and D3
(`D3/w3.c` line 162).

Measured effect of the shortcuts I could isolate (3 alternating runs each, CPU 1, mains, my machine):

| workload | model as written | with the shortcut removed | change | file |
|---|---|---|---|---|
| W4 | 25.2, 25.3, 25.4 ms; peak 4,352-4,728 KB | 31.8, 31.9, 32.1 ms; 4,328-4,732 KB | **+26% time** | `w4_listobj.c` |
| W3 | 447, 477, 484 ms | 489, 507, 519 ms | +5% to +9% time | `w3_scan.c` |
| W2 | 879, 883, 897 ms; peak 76,588-76,988 KB | 731, 743, 744 ms; 92,216-92,572 KB | **+20% memory**, -16% time | `w2_listobj.c` |

### W1 (ECS)

Matches the design: pools on reserved ranges with tombstones and a LIFO free list, entity serials as
occupants, view led by the smaller pool, sparse probes bounds-checked, no generation check on `p`/`v`
(P5), `destroy` through each pool's function table, the cursor published before the rare-path
inserts. Shortcuts and differences:

1. `sparse` is a reserved, page-committed range; the design's `Pool` declares `sparse: list[u32] = []`
   (section 3.5), a doubling list. The list would copy on each doubling and end with capacity
   2,097,152 entries (8 MiB) per pool against the 7.08 MiB used, by arithmetic from the 1,856,653 ids;
   at the last doubling the old 4 MiB and new 8 MiB buffers coexist. Effect on peak memory: up to a
   few MB, not measured. Minor.
2. The single `iter_limit` per pool (N7) is a semantic gap, not a speed shortcut.
3. No other shortcut found.

### W2 (tree with parent pointers)

Matches: 40-byte node slots with generations, 8-byte references, P3 (no check on any hop, which is
valid for this program because `Node` has no `drop` and `nodes` only grows), bounds checks kept on
both random indexings, Python `list.remove` order-keeping removal (more work than the workload's
"order does not matter" requires, so not flattering).

1. **The child list lives inside the node's slot** (`RefList children` inline, 24 bytes). The design
   says a `list` is an object in a store slot with its own generation (1.1, 1.2) and that the node
   "owns this list" (3.2), so the node should hold an 8-byte reference to a separate list object.
   `w2_listobj.c` does that (node slot 24 bytes, list slot 32 bytes): peak memory rises by about
   15.6 MB (+20%), and the run gets 16% faster, because the hot parent walk now strides 24-byte
   slots instead of 40. The model therefore flatters W2's memory and understates its speed. If D2
   wants owned objects inlined into their owner's slot, it must specify that and say how a reference
   to the inlined list is represented (a 16-byte interior reference, so `list[Node]` parameters would
   need two representations).
2. The design says the parent walk is "a `lea`, a load and a compare per hop". My assembly of the
   model (`w2_orig.s`, loop `.LBB0_16`) has two `lea`s, the load of `parent.idx`, and a compare
   that loads `parent.gen` from the same slot: one load more than described, in the same cache
   line. Minor.

### W3 (game objects)

Matches: one store for the open hierarchy with a class id, virtual `update` through a table, the
target check as the only generation check, destroy by generation bump, tombstones and one compaction
pass, LIFO slot reuse, slot retirement, correct order of random draws.

1. **`objects.remove(o)` is modelled as an O(1) tombstone at the loop's position.** The program calls
   Python's `list.remove(x)`, which searches for the first equal element; the design never states a
   rule that `remove` of the loop's own element is done at the cursor. W2 and W4's models do search
   (`reflist_remove`, `dispatcher_unregister`). With the search (`w3_scan.c`), W3 takes 5% to 9%
   longer. D3's model makes the same shortcut (`D3/w3.c` line 143), D1 spells the program with
   `retain`, so this does not skew D2 against D3, but it does skew D2 against C and against D1.
   Fix: either specify the cursor rule, or write the program with `retain`.
2. `dead: list[int] = []` is created inside the frame loop in the D2 program, so each frame allocates
   a list object, grows it from 4, and destroys it; the model reuses one buffer with `dead.len = 0`.
   About 1,000 allocations and a few reallocations each: small, not measured.
3. The bounds check on `objects[h_rand() % i]` in the build loop is dropped ("in range: objects holds
   i elements"). Proving `len(objects) == i` needs an induction fact across `append`; `w2.c` keeps the
   same check and D1 keeps it. 100,000 compares: negligible, but inconsistent.
4. L4's iteration count on the list header (`iterating++`/`--`, the tombstone flag) is replaced by a
   local `first_removed`: two stores per loop, negligible.

### W4 (callbacks)

Matches: 40-byte closure state with generation and pin count, 16-byte callables, P1/P4/P6 as the
design states them, the division-by-zero check kept, the O5 pin on the rare path only, unregister
by search with order kept.

1. **The handler list lives inside the dispatcher's slot** (`CallList handlers` inline). A list is an
   object (1.1); with a separate list object (`w4_listobj.c`) every event pays one more dependent
   load, which the indirect call stops the compiler from hoisting, because a handler could replace
   `d.handlers`. Measured +26%. This is the largest flattery in the four models.
2. `remove` does not test the list's iteration count (L4): rare path, negligible.
3. The bounds check on `handlers[k % n]` is removed by range reasoning (k is non-negative, n is the
   length just read). A compiler can do this; acceptable.
4. Not a shortcut but a cost the design's table omits: the 4-byte `pins` field in the closure state.
   Section 1.8 lists per-object cost as the generation plus, in open hierarchies, a class id; any class
   whose methods can destroy their own object needs a pin count somewhere too.

### Same for every design, stated by the design

Integer overflow checks and teardown after `h_report` are left out of all models. Fine for the
comparison, but note that H3 lives exactly in the teardown the models skip.

---

## 4. What D2 would need to change (short list)

1. Specify references to stack values (H1) and choose; each option costs one goal.
2. Redo the thread part (H2): isolation at `spawn`/`freeze`, a shared store for frozen and locked
   objects, references that name their store.
3. Cheap rule fixes: P3/P4 off in drop-reachable code (H3); the iteration count pins the container
   (H4); G3 per-use resolution in loops that may insert (H5); `{store, index, occupant}` element
   references and process-wide entity serials (H6); generation-checked destroy, bounded free-list pop
   and trampoline (H7, H10); `E2100` kept for non-`Zeroable` fields (H8); slice bounds against the
   live length (H9).
4. Rule on ownership in the places Python code does it most: field-read-then-overwrite as a move
   (N1), shared closure cells (N2), conditional stores and mixed returns (N3), slices of temporaries
   (N4), struct copies (H7, N5).
5. Re-run W2 and W4 with lists as separate objects (or specify inlining), and W3 with a specified
   `remove` rule, before the measurer compares D2 with C.

---

## Files

- Report: `tasks/96/redesign/phase3-break-D2.md` (this file).
- Probe copies (originals untouched), all in `target/redesign/break-D2/`: `w1_count.c` (prints ids
  issued), `w2_listobj.c` (child lists as separate list objects), `w3_scan.c` (`remove` as a
  search; prints the destroyed count), `w4_listobj.c` (handler list as a separate list object),
  `w2_orig.c`/`w3_orig.c`/`w4_orig.c` (the models with only the include path changed, rebuilt for
  the A/B runs), `w2_orig.s` (the assembly read for W2), and the `.exe` files built from them.

## Sources

- The design and models: `tasks/96/redesign/phase3-design-D2.md`, `target/redesign/bench/D2/w1.c`-`w4.c`,
  `target/redesign/bench/harness.h`; for comparison `target/redesign/bench/D1/w3.c`, `D3/w3.c`.
- Spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`: `[RNG-1]`, `[RNG-4]` (line 1047), the check
  table (lines 236-246), `[TYP-13]` (line 1104), `[ARN-11]` (line 2976), `E2100` (`[CLS-2]`, line 1712).
- Phase reports quoted by the design: `phase2-vale-austral-pony.md` (lines 291-312),
  `phase2-hylo-csharp.md` (lines 402-414), `phase2-rc-elision.md` (line 299); annoyance ids A21-A25 in
  `phase1-borrowing.md` (lines 37-41), A31-A34 in `phase1-sharing.md` (lines 26-29).
- Windows x64 user address space, 128 TB on Windows 8.1 and later:
  https://learn.microsoft.com/en-us/windows/win32/memory/memory-limits-for-windows-releases
- EnTT's in-place delete (the design's UNVERIFIED item, now confirmed to exist): added in EnTT v3.9.0,
  https://newreleases.io/project/github/skypjack/entt/release/v3.9.0 ; swap-and-pop is the default,
  https://github.com/skypjack/entt/issues/1101
- Measurements: my runs described at the top, files above.
