# Phase 1 catalogue: sharing, closures, dynamic types, concurrency, globals, FFI

Annoyance ids A9, A10, A12, A13, A14, A15, A16, A19, A20, plus new ids A31 to A34.
Written 2026-10-24 by the phase-1 "sharing" research agent.

Every program named below is in `target/redesign/phase1/sharing/`. The full compiler output is saved
beside them: `rustc_output.txt` (rustc 1.98.1, `rustc --edition 2021 --crate-type lib <file>`) and
`ember_check_output.txt` (`target/ci/ember.exe check <file>`). Every rustc error quoted here was
produced by that rustc on this machine; none is quoted from memory. Every Ember result is from
`ember check` (type and borrow checking only): no Ember program was run, so every statement about
what an Ember program does at run time comes from the spec and is marked as such.

## Summary

| id | annoyance | severity (Python programmer) | Ember today | Rust error (real) | Ember result (real) |
|---|---|---|---|---|---|
| A9 | shared mutable state | high | partly | E0499; workaround panics `RefCell already borrowed` | classes share and mutate (compiles); `ref mut` struct version E3022; a conflicting field access is a run-time panic |
| A10 | choosing among ownership kinds | high | partly | E0382, then E0596 after switching to `Rc` | E3040 for `Box`; `class` and `Shared` versions compile |
| A12 | closures capturing mutable state | high | worse | E0373 + E0503; `move` compiles with a warning, returns 0 | E3026, reported inside the callee; `owned fn` compiles silently and bumps a copy |
| A13 | trait objects, dyn compatibility, downcasting | medium | partly | E0038, E0599 | E2050; `cast` refused on `Box[dyn I]` (E2020) |
| A14 | async: Pin, Send, borrows across await | medium | partly | "future cannot be sent between threads safely", E0373, E0599 (Pin) | `async` is E0005 (reserved); generators: E2221, E2229 |
| A15 | threads: Send/Sync, sharing, scoped threads | medium | partly | E0277, E0521, E0373, E0499, E0503 | E3021, E3022, E3060, E7004; workaround compiles |
| A16 | generics friction: orphan rule, bounds, no specialisation | medium | same | E0117, E0369, E0119 | E2121, E2040, E2041 |
| A19 | global mutable state | medium | same | E0133, E0596 | E1010 (misleading), E3021 |
| A20 | unsafe code and FFI boundaries | low | partly | E0308 | E5040 without a contract; with `callback(…, user_data=…)` it compiles with no `unsafe` |
| A31 (new) | caching inside a read-only method | medium | partly | E0594 | E3023 for a struct; a class compiles |
| A32 (new) | callbacks that receive the object holding them | high | same | E0499 | E3080 (class), E3020 (struct) |
| A33 (new) | lock ceremony and references out of a guard | medium | worse (unsound) | E0515 | compiles: a reference escapes the guard; a use-after-free and a data race compile (see "Found in passing") |
| A34 (new) | the clone-before-move dance for shared handles | medium | absent | E0382 | compiles: handles are `Copy`, copying retains |

Found in passing (details at the end): a soundness hole in Ember's guards (`RefCell` and `Mutex`
references outlive their guard; four probes compile that Rust refuses), a name-resolution bug inside
lambdas, an error reported in the wrong function, and a misleading "cannot find" for a static.

---

## A9 Shared mutable state

### (1) Rust

`a09_shared_state.rs`: a camera and a HUD both change the player.

```rust
pub struct Camera<'a> { pub target: &'a mut Player }
pub struct Hud<'a> { pub target: &'a mut Player }
let cam = Camera { target: &mut p };
let hud = Hud { target: &mut p };
cam.target.hp -= 1;
```

```text
error[E0499]: cannot borrow `p` as mutable more than once at a time
 8 |     let cam = Camera { target: &mut p };
   |                                ------ first mutable borrow occurs here
 9 |     let hud = Hud { target: &mut p };
   |                             ^^^^^^ second mutable borrow occurs here
10 |     cam.target.hp -= 1;
   |     ------------------ first borrow later used here
```

Workaround: `Rc<RefCell<Player>>` (`a09_workaround.rs`), with `Rc::clone` and `.borrow_mut()` at every
use. It compiles; walking the inventory while the camera writes `hp` panics. I built and ran this Rust
program: `thread 'main' panicked at a09_workaround.rs:16:20: RefCell already borrowed`, exit 101.
The two accesses touch different fields (`inventory` and `hp`); `RefCell` locks the whole value.

### (2) Python

```python
class Player:
    def __init__(self): self.hp = 100; self.inventory = []
p = Player(); cam = Camera(p); hud = Hud(p)
cam.target.hp -= 1
hud.target.inventory.append("key")
for item in hud.target.inventory:
    cam.target.hp -= len(item)
```

### (3) Ember today: partly

`a09_shared_state.em` (classes) compiles: two handles to one object, both write it, and writing the
`Copy` field `hp` while `inventory` is iterated is accepted, because an access to a `Copy` field is
instantaneous and never checked (`[EXC-17]`) and access words are per field (`[EXC-19]`). The same
file's second loop, which pushes to the list being iterated through another handle, also compiles;
by `[EXC-1]` it panics at run time (`exclusivity violation: write access to Player.inventory while a
read access …`; not run, from the spec and `tests/conformance/CLS-7/run_fail_*`).

`a09_shared_state_struct.em` (a struct holding `ref mut`, Rust's shape) is refused as Rust is:

```text
error[E3022]: `p` is already mutably borrowed
15 |     cam = Camera(ref mut p)
   |                  --------- mutable borrow of `p` starts here
16 |     hud = Hud(ref mut p)
   |               ^^^^^^^^^ conflicting access here
```

So Ember removes the wrapper noise (`class` is the one spelling) and the whole-value lock (per-field
words), but keeps a run-time panic where a pointer into a field is live across a write to that field.
Cost today: a 24-byte header per object (`[OBJ-1]`), one 4-byte access word per non-`Copy` field
(`[EXC-19]`), counted handles (`[RC-1]`).

### (4) How often, severity

- LogLog Games, "Leaving Rust gamedev after 3 years" (2024-04-26,
  https://loglog.games/blog/leaving-rust-gamedev/): complains of "crashes due to double borrows when
  doing anything with dynamic borrow checking".
- Warp, "Why is building a UI in Rust so hard" (Aloke Desai, 2023-02-14,
  https://www.warp.dev/blog/why-is-building-a-ui-in-rust-so-hard): "Using `RefCell` mostly works, but
  is by no means ergonomic."
- Catherine West, RustConf 2018 closing keynote text (https://kyren.github.io/2018/09/14/rustconf-talk.html):
  generational indexes are "way easier than self borrowing or Rc<RefCell>".
- 2023 State of Rust survey (https://blog.rust-lang.org/2024/02/19/2023-Rust-Annual-Survey-2023-results/):
  the borrow checker is named among the three areas people "struggle with the most" (no percentage given).

Severity: **high**. Shared objects are the default Python design.

### (5) What Rust's rule prevents, and eliminations

Prevents: one path replacing or growing a part of an object while another path holds a pointer into
that part. Without it, the loop above would read `inventory`'s old buffer after a push through the
other handle reallocated and freed it (use-after-free).

(a) **Pythonic spelling of the same guarantee.** Keep `class` as the only word for "shared"; no
`Rc`, `RefCell`, `borrow_mut`. Ember already has this. What remains is to report statically the
panics the compiler can see: the second loop of `a09_shared_state.em` goes through two locals known
to hold one object (`cam.target` and `hud.target` were built from `p`); a warning there costs nothing
at run time. Gives up nothing.

(b) **Smarter: a pointer into a shared container is a path, not an address.** The hazard exists only
because the loop holds a raw address into `inventory`'s buffer across code that can reallocate it.
Python's list iterator holds the list and an index and re-reads the length each step, so appending
while iterating is harmless. Ember can do the same for containers reached through a class handle:

```ember
for item in hud.target.inventory:          # lowered as: i = 0; while i < len(obj.inventory): item = obj.inventory[i] …
    cam.target.inventory.push(item.clone()) # legal; the loop sees the new items, as in Python
```

and `item`, when it is not `Copy`, is re-resolved as `obj.inventory[i]` at each use (one bounds check)
instead of being an address. Cost: per step, one load of the length and one compare instead of a
pointer increment; per use of a non-`Copy` `item`, one bounds check; the loop over a class field loses
vectorisation unless the compiler proves no write (the `[EXC-8]` hoisting proof already does that).
Memory: none. Gives up: the panic becomes defined behaviour (Python's), so a program that appends
forever loops forever, as in Python. Applies equally to A2/A4 (other agents' ids).

A second (b), for comparison: generational references (Vale): every reference carries the object's
generation and checks it on dereference. Vale's own benchmark (BenchmarkRL terrain generator,
https://verdagon.dev/blog/generational-references) reports +10.84% time over unsafe code, against
+25.29% for reference counting. One benchmark, by the language's author; not measured here.

---

## A10 Choosing among ownership kinds

### (1) Rust

`a10_ownership_kinds.rs`: two entities share a mesh. With `Box`:

```text
error[E0382]: use of moved value: `m`
 8 |     let m = Box::new(Mesh { verts: vec![0.0; 9] });
   |         - move occurs because `m` has type `Box<Mesh>`, which does not implement the `Copy` trait
 9 |     let a = Entity { mesh: m };
   |                            - value moved here
10 |     let b = Entity { mesh: m };
   |                            ^ value used here after move
```

Switch the field to `Rc<Mesh>` to share it, and editing it fails:

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
16 |     a.mesh.verts.push(1.0);
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Mesh>`
```

Workaround: `Rc<RefCell<Mesh>>`, and `Arc<Mutex<Mesh>>` once a thread touches it; each change edits
every type that mentions the field.

### (2) Python

```python
m = Mesh([0.0] * 9)
a, b = Entity(m), Entity(m)
a.mesh.verts.append(1.0)
```

### (3) Ember today: partly

`a10_ownership_kinds.em`: the `Box` version is refused as in Rust:

```text
error[E3040]: `m` has been moved out of
11 |     a = Entity(m)
   |         --------- moved here
12 |     b = Entity(m)
   |         ^^^^^^^^^ used here after the move
   = help: clone the value, or borrow it instead of moving it
```

The `class MeshC` version and the `Shared[Mesh]` version (`a.mesh.get_mut().verts.push(1.0)`) compile
(only the `build()` error is reported). The choice is made once, at the declaration (`class` or
`struct`), not at every use, which is better than Rust; but the spec's table (§IX.0) still lists
eleven mechanisms (`struct`, collections, `Box`, `class`, `Shared`, `SyncShared`, `Weak`, arenas,
`Pool`+`Handle`, `Cell`/`RefCell`, `Mutex`/`RwLock`/`Atomic`), and `[SEL-1]` says the declared type
decides storage and "the compiler never picks between them".

### (4) How often, severity

- 2024 State of Rust survey: 45.2% cited complexity as a worry, as reported by devclass.com
  (https://devclass.com/2025/02/18/state-of-rust-survey-2024-most-rust-developers-worry-about-the-future-of-the-language)
  from the survey results; I did not check the figure against the survey's own report (UNVERIFIED).
- 2023 survey (link under A9): 43% were most worried about "Rust becoming too complex" (from the
  official post).
- The A9 sources describe the same choice (`Rc<RefCell>` as the reach-for-it answer).

Severity: **high**. Python has one kind of reference.

### (5) What Rust's rule prevents, and eliminations

Prevents: freeing a value twice or never, and mutating a value other owners assume is unchanged.
Without a single owner per `Box`, `a` and `b` would each free the mesh at their end (double free).

(a) **Pythonic spelling: two words, the rest inferred.** `class` (identity, shared) and `struct`
(value). `Box` is inferred where it is needed for layout (a recursive field, a `dyn` field), as
Swift's `indirect` but unwritten; `Shared[T]` is a class; `SyncShared[T]` is a `@sync class`; `Cell`
and `RefCell` are unnecessary inside a class (fields are already mutable through any handle);
`Weak` is a field modifier:

```ember
class Node:
    children: Array[Node] = []
    weak parent: Node            # today: parent: Weak[Node] = Weak.empty()
struct Tree:
    left: Option[Tree]            # today: Option[Box[Tree]]; boxed by the compiler
```

Cost: none beyond today's (the inferred `Box` is the same allocation). Gives up: the reader no longer
sees the allocation in the type; `ember inspect --alloc` (`[SEL-1]`) still reports it.

(b) **Smarter: one semantic kind, representation chosen by the compiler.** Every non-`Copy` aggregate
has reference semantics in the source; the compiler picks inline storage, a unique box, or a counted
object by escape and uniqueness analysis (Ember already promotes class objects to the stack when
nothing escapes, `[OPT-1]`, and removes counts, `[RC-2]`). Cost: where the analysis fails, a 24-byte
header and counts (`[OBJ-1]`, `[RC-1]`); speed becomes harder to predict, which conflicts with the
C-speed goal and with `[SEL-1]`. Gives up: the guarantee that a `struct` never allocates. I rate (a)
the better fit for the owner's goal; (b) is listed because it removes the decision entirely.

---

## A12 Closures and callbacks capturing mutable state

### (1) Rust

`a12_closures.rs`: a button handler bumps the score.

```rust
pub fn on_click(&mut self, f: impl FnMut() + 'static) { self.handlers.push(Box::new(f)); }
let mut score = 0;
b.on_click(|| score += 1);
b.click();
score
```

```text
error[E0373]: closure may outlive the current function, but it borrows `score`, which is owned by the current function
11 |     b.on_click(|| score += 1);
   |                ^^ ----- `score` is borrowed here
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `score` (and any other referenced variables), use the `move` keyword
error[E0503]: cannot use `score` because it was mutably borrowed
13 |     score
   |     ^^^^^ use of borrowed `score`
```

Following the help (`move ||`) compiles and returns 0, because the closure bumps its own copy; rustc
warns: `warning: value captured by `score` is never read … = help: did you mean to capture by
reference instead?`. The real workaround is `let score = Rc::new(Cell::new(0));` plus a clone moved
into each closure.

### (2) Python

```python
score = 0
def on_click():
    nonlocal score
    score += 1
b.on_click(on_click); b.click()
return score          # 1
```

### (3) Ember today: worse

`a12_closures.em` (plain lambda):

```text
error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> a12_closures.em:7:28
7 |         self.handlers.push(f)
  |                            ^ stored here
  = help: write it `owned fn`, which moves or copies each capture into the value
```

The error points into `Button.on_click`, not at the line that wrote the lambda (`b.on_click(fn():
score += 1)`); in a file with several callers (`a12_closures.em` has three) it does not say which one.
`a12_closures_owned.em` follows the help (`owned fn(): score += 1`) and **compiles with no warning**;
by `[CLO-2]` the `int` is copied in, so `game_owned()` returns 0 (from the spec; not run). Rust warns
in the same case; Ember is silent. `a12_closures_class.em` (score in a class object, `owned fn`)
compiles and shares as Python does; `a12_closures_class_plain.em` (the same with a plain lambda) is
E3026 again, though capturing a handle by retain is always safe.

### (4) How often, severity

- Raph Levien, "Xilem: an architecture for UI in Rust" (2022-05-07,
  https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html): the standard callback approach
  "works poorly in Rust, though it can be done"; "it requires _shared mutable_ access to that state".
- Warp (link under A9): Rust "discourages the use of shared mutable state".
- 2024 survey (https://blog.rust-lang.org/2025/02/13/2024-State-Of-Rust-Survey-results/): async closures
  were among the most desired features (no percentage given).

Severity: **high**. Event handlers and callbacks are everyday Python.

### (5) What Rust's rule prevents, and eliminations

Prevents: a stored closure writing a variable whose stack frame has ended. Without it, a click after
`game()` returned would write `score`'s dead stack slot (use-after-free of the stack).

(a) **Pythonic spelling: an escaping captured variable moves to the heap.** What Python, Swift, C#
and JavaScript do: when a closure that captures a variable escapes (is stored or returned), the
variable itself is placed in a small counted box shared by the function and the closure, so both see
one `score`:

```ember
fn game() -> int:
    score = 0
    b.on_click(fn(): score += 1)    # legal: `score` is boxed because the lambda escapes
    b.click()
    return score                    # 1, as in Python
```

Cost: for each escaping captured variable that is written, one heap allocation when the variable is
created (count + value; the count need not be atomic, as for `Shared`, `[RC-4]`) and one indirection
per access in both the function and the closure. Non-escaping closures (a parameter-position `fn`,
`[CLO-3]`'s zero-cost form) are unchanged. A boxed variable is not `Send`, so such a closure cannot go
to a thread (the existing `[THR-8]` rule). Gives up: nothing a program can observe except the cost;
the silent copy of today's `owned fn` disappears. Handles (class objects) need no box: they are already
shared, so a plain lambda capturing one should capture it by retain (removes `a12_closures_class_plain.em`'s E3026).

(b) **Smarter: callbacks that cannot outlive what they borrow.** Two ways remove the escape itself.
Scoped registration, the same structure as `thread.scope()` (`[THR-5]`):

```ember
with b.on_click(fn(): score += 1):    # unregistered at the end of the block
    run_frame(b)
```

or handlers that get the state as an argument instead of capturing it (Elm style):

```ember
b.on_click(fn(g: Game): g.score += 1)    # the dispatcher passes the game in; nothing captured
```

Cost: zero (borrowing closures, direct calls). Gives up: a handler that outlives the function that
registered it (scoped form), or captures at all (argument form).

---

## A13 Trait objects: dyn, dyn compatibility, downcasting (the ECS registry)

### (1) Rust

`a13_dyn.rs`: a registry `HashMap<TypeId, Box<dyn AnyPool>>`, `get_mut::<T>()` downcasts; and a trait
with a generic method used as `dyn`.

```text
error[E0599]: no method named `downcast_mut` found for mutable reference `&mut Box<(dyn AnyPool + 'static)>` in the current scope
13 |         let pool = pool.downcast_mut::<Pool<T>>().unwrap();
error[E0038]: the trait `Inspect` is not dyn compatible
20 | pub trait Inspect { fn inspect<V: Visitor>(&self, v: &mut V); }
   |           -------      ^^^^^^^ ...because method `inspect` has generic type parameters
   = help: consider moving `inspect` to another trait
```

Workaround (`a13_workaround.rs`, compiles): `trait AnyPool: Any`, `T: 'static` on every impl and
method, and an explicit upcast `let pool: &mut dyn Any = ….as_mut();` before `downcast_mut`. Before
Rust 1.86 (2025-04-03, https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/) this needed a hand-written
`as_any()` method in every implementation.

### (2) Python

```python
class Registry:
    def __init__(self): self.pools = {}
    def get(self, t, i): return self.pools[t].dense[i]      # no cast at all
```

### (3) Ember today: partly

`a13_dyn.em`:

```text
error[E2020]: `cast … as` and `cast … if` need related class handles, found `Box[dyn AnyPool]` and `Pool[T]`
18 |         pool = cast self.pools[7u128] as Pool[T]
error[E2050]: interface `Inspect` is not `dyn`-compatible
27 | fn all(xs: Array[Box[dyn Inspect]]) -> int:
   |                      ^^^^^^^^^^^ method `inspect` is generic
```

No `'static` noise, and casting works between class handles (`target/analysis/ecs5/shape.em`
compiles), but the class route then refuses returning a reference into the pool
(`target/analysis/ecs5/pos.em`: E3060, "a returned reference must derive from a parameter (LT-1)"),
which is ODR-286's story. The missing piece the ODR names, `cast` of a `dyn I` place giving a
reference, is confirmed missing by E2020 above. Dyn compatibility (E2050) is the same rule as Rust's.

### (4) How often, severity

- The Rust project fixed the `as_any` boilerplate only in 1.86 (2025, link above); the Rust users
  forum thread (https://users.rust-lang.org/t/downcast-ref-not-working-on-a-trait-that-does-inherit-from-any/137687)
  says `as_any()` methods existed as a workaround for the lack of upcasting (search summary; not read in full).
- `tasks/95/ecs-entt-design.md` and ODR-286: the owner hit it directly building the registry.

Severity: **medium**. Few Python programmers write registries; those who build engines all do.

### (5) What Rust's rule prevents, and eliminations

Prevents: reading a value as the wrong type. Without the type check on downcast, `get_mut[f32]`
on the `int` pool would reinterpret its bytes (type confusion); without dyn compatibility a vtable
would need a slot for every possible instantiation of `inspect`, which separate compilation cannot
know.

(a) **Pythonic spelling: `cast` works on any `dyn` place, and borrows it.**

```ember
fn get_mut[T](mut self, i: int) -> ref mut T:
    pool = cast self.pools[type_identity[T]()] as Pool[T]   # ref mut Pool[T] borrowed from the map
    return ref mut pool.dense[i]                             # existing static borrow rules apply
```

The interface table of every `dyn I` carries the type identity (a class object already reaches its
type through the header, `[OBJ-1]`). Cost: one word per interface table (per type, not per object);
one load and compare per cast; a failed cast panics (`as!`) or returns `None` (`as?`). Gives up nothing.

(b) **Smarter: no downcast, and generic methods in `dyn`, for a whole program.** Ember monomorphises
every generic (`[TYP-16]`) and sees the whole program. (1) The registry's pools can be found by a
compile-time index per component type, `self.pools[type_index[T]()]`, a dense array instead of a map,
typed at the use site, so the cast is checked once at the type level (no run-time compare). (2) A
`dyn` interface with a generic method can get one table slot per instantiation the program actually
uses, removing E2050. Cost: (1) none at run time, one array slot per component type; (2) table size
grows with the number of instantiations used. Gives up: separate compilation of `dyn` users and hot
reload (Annex B) for (2); a component type added at run time (plugins) for (1).

---

## A14 Async: Pin, Send, borrows across await

### (1) Rust

`a14_async.rs` (std only; `spawn` stands for an executor's spawn with tokio's bounds):

```text
error: future cannot be sent between threads safely
10 | /     spawn(async move {
11 | |         let n = fetch().await;
12 | |         println!("{}", cache.len() + n);
   = help: within `{async block@a14_async.rs:10:11: 10:21}`, the trait `Send` is not implemented for `Rc<Vec<i32>>`
error[E0373]: async block may outlive the current function, but it borrows `name`, which is owned by the current function
error[E0599]: no method named `poll` found for mutable reference `&mut Box<(dyn Future<Output = ()> + 'static)>` in the current scope
```

The last one is Pin: a stored future can be polled only as `Pin<&mut _>`. Workarounds: `Arc` instead
of `Rc`, `async move` with clones, `Box::pin`, `Pin<Box<dyn Future + Send>>`.

### (2) Python

```python
async def serve():
    cache = [1, 2, 3]
    n = await fetch()
    print(len(cache) + n)
asyncio.create_task(serve())
```

### (3) Ember today: partly

`a14_async_kw.em`:

```text
error[E0005]: `async` is reserved for v2
3 | async fn fetch() -> int:
```

So Pin and Send-on-futures do not exist yet (§XI.6). Ember's generators use the same frame lowering
(§XI.6: "the same frame lowering serves an `async fn`"), and `a14_generators.em` shows what an async
design inherits:

```text
error[E2221]: a borrow of `buf`, a local of this generator, is held across this `yield`
7 |         yield s.clone()
  = help: iterate the collection `owned` (`for x in owned xs:`), or by index; borrows of the generator's parameters may be held
error[E2229]: a borrow of a field of a class object is held across this `yield`
15 |             yield entry.len()
  = help: copy what is needed out of the field before the `yield`, or iterate by index
```

E2221 refuses a generator walking its own local list, which Python accepts and which Rust's `async`
accepts too (through Pin). Here Ember is stricter than Rust.

### (4) How often, severity

- 2023 survey (link under A9): async is first of the three areas people "struggle with the most".
- "What we heard about Rust's challenges" (2026-03-20, https://blog.rust-lang.org/2026/03/20/rust-challenges,
  about 70 interviews): "async was consistently something that many people had issues with".
- without.boats, "Pin" (2024-07-19, https://without.boats/blog/pin/): Pin is one of the least accessible
  and most misunderstood parts of async Rust; users are "thrown off a cliff" into unidiomatic APIs
  (search summary of the post).
- The Send-bound problem for async trait methods is still unstabilised: return type notation is on
  nightly and its stabilisation is a 2026 project goal (https://rust-lang.github.io/goals/2026/rtn.html,
  search summary).

Severity: **medium** for Ember's audience (games use generators, `[CORO-11]`); high for server code.

### (5) What Rust's rule prevents, and eliminations

Prevents: (Pin) a suspended frame moving while a pointer into itself lives; (Send) a task resumed on
another thread touching non-atomic counts; ('static) a task outliving the stack it borrows. Without
Pin, `for s in &buf { … .await }` would hold `&buf` into the old frame location after the executor
moved the future (dangling).

(a) **Pythonic spelling: frames that never move.** A task's frame is allocated once at spawn and never
moved, so borrows of its own locals across `await` are ordinary borrows with no Pin in the language;
for generators, a frame consumed in place (`for s in lines():`, the frame in the caller's slot, never
moved) may hold borrows of its locals:

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    for s in buf:            # legal when the compiler sees the frame is not moved
        yield s.clone()
```

Cost: one allocation per spawned task (Rust executors also store each spawned future in a heap task;
for tokio this is from memory, UNVERIFIED); zero for in-place generators. Gives up: moving a suspended
generator that borrows its own locals (the compiler refuses the move instead of the borrow).

(b) **Smarter: remove each hazard.** (1) Self-borrows: lower `for s in buf` over a frame-owned
collection to an index loop that re-resolves `buf[i]` after each resume (the fix E2221's help asks the
user to write by hand); then moving the frame is harmless. Cost: one bounds check per step. (2) Send:
thread-per-core executors, where a task never leaves the thread that spawned it, so non-`@sync` class
handles are fine across `await`. Cost: no work stealing, so load imbalance between cores. (3) 'static:
structured tasks inside a `with` block, as `thread.scope()` already does (`[THR-5]`). Cost: none.
Gives up: (2) migration of tasks between threads; (3) fire-and-forget tasks without a scope.

---

## A15 Threads: Send/Sync, sharing data, scoped threads

### (1) Rust

`a15_threads.rs`: count words in several texts on several threads.

```rust
for t in texts {
    handles.push(thread::spawn(|| { total += t.split_whitespace().count(); }));
}
```

Five errors, among them:

```text
error[E0521]: borrowed data escapes outside of function
error[E0373]: closure may outlive the current function, but it borrows `total`, which is owned by the current function
error[E0499]: cannot borrow `total` as mutable more than once at a time
error[E0277]: `Rc<Vec<String>>` cannot be sent between threads safely
```

Workaround (`a15_workaround.rs`, compiles): `thread::scope` (Rust 1.63, 2022-08-11,
https://blog.rust-lang.org/2022/08/11/Rust-1.63.0/), an `AtomicUsize` with `Ordering::Relaxed`,
`let total = &total;` and `move` so each thread copies the references.

### (2) Python

```python
total = 0; lock = threading.Lock()
def work(t):
    global total
    with lock: total += len(t.split())
threads = [threading.Thread(target=work, args=(t,)) for t in texts]
```

### (3) Ember today: partly

`a15_threads.em`:

```text
error[E3021]: `t` cannot be written while it is borrowed
7 |         for t in texts:
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
error[E3022]: `total` is already mutably borrowed
error[E3060]: `t` does not live long enough
error[E7004]: the thread's function captures `names`, a `Names`, which is not Send
   = help: declare it `@sync class Names`: its fields are all `Send` and `Sync` [THR-1]
```

`thread.scope()` with `with` exists and needs no lifetime, but the loop variable `t` cannot be borrowed
by a task (the copy-capture rule `[CLO-13]` excludes variables "assigned … anywhere in a loop around
the closure", and a loop variable is), so the working version (`a15_workaround4.em`, compiles) needs
Rust's shape: `tr = ref total` before the loop and `owned fn` to copy `t` and `tr` in. A `@sync` class
must have every field immutable after `init` (`[THR-1]`), so shared counters become `Atomic` or `Mutex`
as in Rust. Two intermediate versions (`a15_workaround.em`, `a15_workaround2.em`) hit a compiler bug
(E1010, see "Found in passing").

### (4) How often, severity

- The Rust 1.63 release post introduces scoped threads because `thread::spawn` closures must be
  `'static`, so borrowed data could not be passed (link above).
- 2023 survey (link under A9): the borrow checker and traits are among the hardest areas.

Severity: **medium**. Python programmers use threads less than callbacks; the GIL hides races.

### (5) What Rust's rule prevents, and eliminations

Prevents: two threads writing `total` at once (lost updates, a data race), and a thread reading
`texts` after the function returned and freed it. Without Send, two threads would update an `Rc`'s
plain count at once and free the vector while one still uses it.

(a) **Pythonic spelling: per-iteration variables and reductions.** Each loop iteration's `t` is a new
variable that a task may capture by copy (what C# 5 and JavaScript's `let` do), and `+=` on a captured
integer from scoped tasks is compiled as one private accumulator per task, added in task order at the
end of the scope (Ember's `@parallel(reduce=…)` already works this way, `[PAR-3]`):

```ember
with scope = thread.scope():
    for t in texts:
        scope.spawn(fn(): total += t.split_whitespace().count())   # legal: a per-task sum
```

Cost: one integer per task and one add per task at the join; no atomics, deterministic (`[PAR-5]`).
Gives up: reading `total` inside the tasks (refused: it is not yet the sum).

(b) **Smarter: move whole object graphs to a thread instead of making them `@sync`.** A non-`@sync`
object may go to another thread when the sender provably keeps no handle into its graph (an isolated
graph, Pony's `iso` capability). Then plain fields, plain counts and no `Atomic` wrappers suffice on
the receiving thread. Cost: zero at run time when proven statically; proving that no handle inside the
graph points out of it needs a capability or region analysis that Ember does not have; a run-time
check of `strong == 1` proves only the root is unshared, not the graph (so a run-time-only version is
not sound). Gives up: sharing the moved graph with the sender.

---

## A16 Generics and traits friction: orphan rule, bound noise, no specialisation

### (1) Rust

`a16_generics.rs`:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
5 | impl fmt::Display for Vec<Vec3> {
  = note: define and implement a trait or new type instead
error[E0369]: cannot add `T` to `T`
help: consider restricting type parameter `T` with trait `Add`
 9 | pub fn sum<T: std::ops::Add<Output = T>>(xs: &[T]) -> T {
error[E0119]: conflicting implementations of trait `Describe` for type `i32`
```

Workarounds: a newtype `struct Vecs(Vec<Vec3>)` with forwarding methods; spelling every bound; no
workaround for specialisation on stable.

### (2) Python

```python
def sum_all(xs):
    acc = xs[0]
    for x in xs[1:]: acc = acc + x       # duck typing
    return acc
def describe(x): return f"int {x}" if isinstance(x, int) else repr(x)
```

### (3) Ember today: same

`a16_generics.em`:

```text
error[E2121]: `Array[Vec3]` and `Neg` are both another package's, so this package cannot implement one for the other
  = help: implement it where `Neg` or `Array[Vec3]` is declared, or wrap the value in a type of this package
error[E2040]: `T`'s bounds do not provide a matching `Add` operator
error[E2041]: `i32` already implements `Describe`
   = note: two implementations of one interface may not apply to one type [TYP-19]
```

The same three rules (`[TYP-20]`, `[TYP-17]`, `[TYP-19]`). The probe uses `Neg` because the spec's own
test does (`tests/conformance/TYP-20/reject_an_implementation_outside_the_packages_that_may_write_it.em`).
Ember's `Display` for collections is built in (`[TYP-39]`), so the commonest orphan case
(printing a list) does not arise.

### (4) How often, severity

- Specialisation: RFC 1210 merged 2016-02-23 (https://github.com/rust-lang/rfcs/pull/1210); the 2026
  project goal "Specialization: minimal implementation" (https://rust-lang.github.io/goals/2026/specialization.html)
  says the original design has soundness issues (search summary).
- Orphan rule: Ixrec's community catalogue (https://github.com/Ixrec/rust-orphan-rules) says "there are
  a lot of impls that people want to write" but cannot; Effective Rust item 6 (https://lurklurk.org/effective-rust/newtype.html)
  notes every operation on a newtype must forward to the inner type (search summaries).
- 2023 survey: traits and generics are among the three areas people struggle with most; the most
  desired improvements are in traits (link under A9).

Severity: **medium**.

### (5) What Rust's rule prevents, and eliminations

Prevents: two packages each implementing `Display for Array[Vec3]`, so that which one runs depends on
link order (incoherence); a generic body using an operation some instantiation lacks, found only
when a user instantiates it, far from the cause.

(a) **Pythonic spelling.** (1) Let a leaf package (a binary nobody depends on) implement any
interface for any type: nothing downstream can conflict, so coherence holds. (2) Infer the bounds of a
non-`pub` generic function from its body and print them on request, so private helpers read like
Python while public signatures stay explicit:

```ember
fn sum[T](xs: Span[T]) -> T:      # private: bounds inferred as Add[Output = T] + Copy
    acc = xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc
```

Cost: zero at run time; compile time for inference. Gives up: (1) the leaf package can no longer be
turned into a library without moving those implementations; (2) error locality for private functions.

(b) **Smarter: specialisation by "most specific wins" at monomorphisation.** Every generic is
instantiated with concrete types (`[TYP-16]`) and Ember's regions are inferred, not part of type
identity; Rust's specialisation soundness problems are commonly attributed to lifetimes in impl
selection, which Ember would not have (UNVERIFIED: needs a proof for Ember's rules). The compiler picks
the most specific implementation per instance. Cost: none at run time. Gives up: adding an
implementation in a dependency can change which one an existing program uses (a versioning hazard).

---

## A19 Global mutable state

### (1) Rust

`a19_globals.rs`:

```text
error[E0133]: use of mutable static is unsafe and requires unsafe function or block
6 |     FRAME += 1;
  = note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
error[E0596]: cannot borrow immutable static item `LOG` as mutable
7 |     LOG.push(format!("tick"));
```

Workaround (`a19_workaround.rs`, compiles): `AtomicU64` with an `Ordering`, `Mutex<Vec<String>>` with
`.lock().unwrap()`; `OnceLock`/`LazyLock`/`thread_local!` for the rest.

### (2) Python

```python
FRAME = 0; LOG = []
def tick():
    global FRAME
    FRAME += 1; LOG.append("tick")
```

### (3) Ember today: same

`a19_globals.em`:

```text
error[E1010]: cannot find `FRAME` in this scope
7 |     FRAME += 1
error[E3021]: cannot write through a shared reference
8 |     LOG.push("tick".to_string())
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is `ref Array[String]`, which only reads
```

The rule is Rust's (`[STA-1]`: a static is immutable and `Sync`; mutation through `Atomic`, `Mutex`,
`RwLock`, `Once`), and the first message is wrong: `FRAME` is found for reading; the error should say
statics are immutable and name `Atomic[int]`. `a19_workaround.em` (an `Atomic` and a `Mutex` with
`with log = LOG.lock():`) compiles, with less noise than Rust (no `.unwrap()`, no explicit ordering).

### (4) How often, severity

- Rust 2024 edition makes references to `static mut` a deny-by-default error (`static_mut_refs`;
  https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html, search summary).
- LogLog Games (link under A9) argues that the dislike of global state is misplaced for games, which
  are effectively single-threaded.

Severity: **medium**. Python scripts and games use module globals routinely.

### (5) What Rust's rule prevents, and eliminations

Prevents: two threads updating `FRAME` at once (lost updates), and, on one thread, a function holding a
reference into `LOG` while a function it calls pushes to `LOG` and reallocates it (use-after-free).

(a) **Pythonic spelling: module variables confined to the main thread.**

```ember
var FRAME: int = 0               # a module variable, main thread only
var LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push("tick".to_string())
```

Accesses are checked as class fields are: a `Copy` variable's access is instantaneous and unchecked
(`[EXC-17]`'s reasoning), a non-`Copy` one has one access word (`[EXC-19]`). Threads: a function
reachable from `thread.spawn`/`jobs` that touches a `var` is refused, using the effect inference Ember
already has (§X.1) with one more effect. Cost: zero for `Copy` variables; one word and a load-compare
per long-term access for others. Gives up: touching these variables from other threads (they use
`Atomic`/`Mutex` statics, as today).

(b) **Smarter: thread-local by default.** Each thread gets its own copy of a module variable, so no
race exists; process-wide sharing is written as a `static` with `Atomic`/`Mutex` as today. Cost: one
thread-local access per use (a thread-pointer-relative load on common targets; not measured here),
one copy per thread. Gives up: Python's process-wide meaning for globals (a worker thread sees its own
`FRAME`), which can surprise.

---

## A20 Unsafe code and FFI boundaries

### (1) Rust

`a20_ffi.rs`: a C library calls back into the game with a user-data pointer.

```text
error[E0308]: mismatched types
8 |     unsafe { register_cb(cb, std::ptr::null_mut()) }
  |                          ^^ expected fn pointer, found closure
note: closures can only be coerced to `fn` types if they do not capture any variables
7 |     let cb = |_ud: *mut c_void| game.score += 1;
  |                                 ^^^^ `game` captured here
```

Workaround (in the same file, compiles): an `extern "C" fn on_event(ud: *mut c_void)` that rebuilds
the object with `unsafe { &mut *(ud as *mut Game) }`, a reference whose lifetime nothing bounds (the
probe even types it `&'static mut Game`, and rustc accepts it: the workaround alone, saved as
`out/a20_workaround_only.rs`, compiles with exit 0).

### (2) Python

```python
CB = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
lib.register_cb(CB(lambda ud: setattr(game, "score", game.score + 1)), None)
```

### (3) Ember today: partly

`a20_ffi.em`: without a contract, refused like Rust, with better help:

```text
error[E5040]: expected `extern "C" fn(*mut void) -> void`, found a lambda that captures
   = help: where C hands the function a user data pointer back, contract the parameter `callback(borrowed, user_data=p)`: it then takes this lambda
```

With the contract `@ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))`,
`token = register_cb(owned fn(): game.score += 1)` compiles with no `unsafe` (only `L1001`: `token` is
never read). The trampoline, the boxed state and the unregistering token are generated (`[FFI-21]`,
`[FFI-23]`); a non-`Send` capture without `threads(…)` is E7004
(`tests/conformance/FFI-21/reject_what_a_callback_captures.em`). The annoyance remains only as the
contract someone must write.

### (4) How often, severity

- The Rustonomicon, "Unbounded Lifetimes" (https://doc.rust-lang.org/nomicon/unbounded-lifetimes.html):
  dereferencing a raw pointer is the most common source of an unbounded lifetime (search summary).
- Armin Ronacher, "Unsafe Rust is Too Hard" (2022, https://lucumr.pocoo.org/2022/1/30/unsafe-rust/);
  Chad Austin, "Unsafe Rust Is Harder Than C" (2024) (search summaries; not read in full).

Severity: **low** for most Python programmers (they use bindings someone else wrote); medium for
engine authors.

### (5) What Rust's rule prevents, and eliminations

Prevents: C calling a callback after the state it captured is gone, or from another thread. Without
it, `on_event` after the `Game` was dropped writes freed memory, and a call from C's worker thread
races with the game loop.

(a) **Pythonic spelling: infer the contract from the header.** When a C function takes a function
pointer and a `void*` that the function pointer's signature also takes, the importer infers
`user_data` and defaults to `retained` (the safe assumption), so the call site is Python-like with no
annotation:

```ember
import c "events.h"
token = c.register_cb(owned fn(): game.score += 1)
```

Cost: zero at run time beyond today's trampoline (one indirect call; one box for retained state).
Gives up: an inferred contract is still an asserted fact; it must be listed in `ember tcb`
(`[TCB-1]`) like a written one, and a wrong inference (a `void*` that is not user data) is unsound.

(b) **Smarter: C holds handles, never Ember addresses.** A registered callback is a generational
index into a runtime table (Ember's `Handle[T]`, `[HND-1]`); the trampoline looks it up and compares
the generation, so a call after unregistering finds nothing instead of freed memory. Cost: one table
load and one compare per callback; one table slot per registration. Gives up: handing C interior
pointers for zero-copy work, which stays `unsafe`.

---

## A31 (new) Caching inside a read-only method

### (1) Rust

`a31_lazy_cache.rs`: a getter that fills a cache the first time.

```text
error[E0594]: cannot assign to `self.radius`, which is behind a `&` reference
6 |             self.radius = Some(self.verts.iter().fold(0.0f32, |m, v| m.max(v.abs())));
help: consider changing this to be a mutable reference
4 |     pub fn radius(&mut self) -> f32 {
```

Workaround: `radius: Cell<Option<f32>>` or `OnceCell<f32>` (stabilised in Rust 1.70, from memory,
UNVERIFIED); following the help instead makes every caller hold `&mut`.

### (2) Python

```python
def radius(self):
    if self._radius is None:
        self._radius = max(abs(v) for v in self.verts)
    return self._radius
```

### (3) Ember today: partly

`a31_lazy_cache.em`: the struct version:

```text
error[E3023]: cannot mutate borrowed parameter `self`
12 |             self.radius_cache = Some(r)
   = help: restructure to a single owner and declare this parameter `mut self: Mesh`
   = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
   = help: if the value has identity, make it a `class`
```

The class version in the same file compiles (a `Copy` field written through `self`, `[CLS-7]`,
`[EXC-17]`). Better help than Rust's, same refusal for values.

### (4) How often, severity

Frequency: UNVERIFIED (no survey separates it). It is the textbook example in Rust's `Cell`
documentation (from memory, UNVERIFIED). Severity: **medium**.

### (5) What Rust's rule prevents, and eliminations

Prevents: writing a value that other readers may be viewing, and, for a `Sync` type, two threads
writing the cache at once. For a `Copy` field written on one thread nothing can dangle (Ember's own
`[EXC-17]` argument); the hazards are a non-`Copy` cache (replacing a `String` frees what a reader
views) and threads.

(a) **Pythonic spelling: writes to `Copy` fields through a shared `self` are allowed for types that
are not `Sync`.** The same reasoning Ember already applies to class fields (`[EXC-17]`). Cost: the
compiler may no longer assume a `Copy` field is unchanged behind a shared reference (it loses that
alias fact for such fields, as C without `restrict`); a struct that does it is not `Sync`. Gives up:
some optimisation on shared borrows, and `Sync` for such structs.

(b) **Smarter: memoisation as a declaration, with automatic invalidation.**

```ember
struct Mesh:
    verts: Array[f32]

    @cached
    fn radius(self) -> f32:          # the result is stored in a hidden slot, returned by value
        return max(abs(v) for v in self.verts)
```

The compiler knows which fields the method reads (`verts`), so every write to `verts` clears the
slot. Cost: a hidden `Option` slot per cached method (an `Option[f32]` here), one branch
per call, one store per write to a field it reads; a `Sync` type needs a `Once`-style atomic slot.
Gives up: caching a result that depends on something outside the object (refused, or not invalidated).

---

## A32 (new) Callbacks that receive the object that holds them

### (1) Rust

`a32_reentrant_callback.rs`: a UI calls each handler with the UI (to count clicks or add handlers).

```text
error[E0499]: cannot borrow `*self` as mutable more than once at a time
5 |         for h in self.handlers.iter_mut() {
  |                  ------------------------ first mutable borrow occurs here
6 |             h(self);
  |               ^^^^ second mutable borrow occurs here
```

Workaround: `let mut hs = std::mem::take(&mut self.handlers); for h in &mut hs { h(self) } self.handlers = hs;`
(handlers added during the loop are lost unless merged by hand).

### (2) Python

```python
def fire(self):
    for h in self.handlers:       # a handler may append; the loop then calls it too
        h(self)
```

### (3) Ember today: same

`a32_reentrant_callback.em`, class version:

```text
error[E3080]: `self.handlers` cannot be read while it is mutably borrowed
8 |         for h in self.handlers.iter_mut():
9 |             h(self)
   = help: bind the borrow of `self.handlers` to a local and finish with it before this line, or copy the value out first
```

and the struct version is E3020 ("cannot mutate `*self.handlers` while it is borrowed by this loop").
Classes do not help here: passing `self` while iterating one of its fields is refused statically
(`[EXC-3]`'s "conflicting accesses through one local … rejected at compile time"). The ECS design notes
the same problem for EnTT's signals (`tasks/95/ecs-entt-design.md`, "Signals' reentrancy").

### (4) How often, severity

Frequency: UNVERIFIED (no source found that counts it). It is the core of every observer, signal and
UI event system. Severity: **high** for game and UI code.

### (5) What Rust's rule prevents, and eliminations

Prevents: a handler pushing to `handlers` while the loop holds a pointer into it (the push reallocates
and frees the array the loop and the running closure live in), or a handler replacing itself and
freeing its own captured state while it runs.

(a) **Pythonic spelling: dispatch by index, the running handler moved out of its slot.** The loop
re-reads the length each step (Python's list iterator), and the call takes the handler out of its
slot, calls it, and puts it back, so no reference into the array lives during the call:

```ember
fn fire(self):
    for h in self.handlers:      # lowered: i = 0; while i < len: f = take(handlers[i]); f(self); restore(i, f); i += 1
        h(self)
```

Cost: two moves of an owned callable value per call (a function pointer plus up to three words of
state, `[CLO-10]`), one bounds check per step. Gives up: a handler that removes or replaces its own
slot needs a defined outcome (a per-slot generation decides whether the put-back happens).

(b) **Smarter: structural changes during dispatch are deferred.** While `fire` runs, `push` and
`remove` on the handler list go to a pending list merged when the dispatch ends (EnTT-style signal
semantics; also Godot's deferred calls, from memory, UNVERIFIED). Cost: one pending array and a merge
per dispatch that changed the list. Gives up: Python's behaviour that a handler added during dispatch
runs in the same dispatch.

---

## A33 (new) Lock ceremony and references out of a guard

### (1) Rust

`a33_lock.rs`: hand out the top name from a locked table.

```text
error[E0515]: cannot return value referencing temporary value
4 |     &scores.lock().unwrap()[0]
  |     ^----------------------^^^
  |     ||
  |     |temporary value created here
  |     returns a value referencing data owned by the current function
```

Workaround: return a clone, or return the guard. The ceremony around it: `.lock().unwrap()` at every
use for poisoning, and guard temporaries whose scope extends through `if let`/`match` bodies (changed
in the 2024 edition for `if let`, from memory, UNVERIFIED), which can deadlock.

### (2) Python

```python
def top(scores):
    with lock:
        return scores[0]       # returns the object; Python refcounts keep it alive
```

### (3) Ember today: worse (the refusal is missing; unsound)

`a33_lock.em` compiles:

```ember
fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]
```

The ceremony is absent (no poisoning, `[THR-3]`; `with` releases the guard), but so is the safety.
Two follow-ups also compile (exit 0, no diagnostic):

- `a33_lock_uaf2.em`, one thread: `r = top(m); clear(m); println(r)` where `clear` locks and clears.
  By construction `r` points at the `String` that `clear` drops: a use-after-free in Safe Ember.
- `a33_lock_race.em`: `r = top(m)` read by the main thread while a scoped task locks `m` and clears it:
  a data race and use-after-free.

The same holds for `RefCell` (`a09_refcell_escape.em`, compiles with only `L3011`). Not run (the rules
forbid running Ember programs); the failure follows from the code. The reference's region is taken
from the borrowed parameter (`scores`), not from the guard, so the borrow checker sees two shared
borrows of `m` and allows them. Only the variant that locks `m` directly in `main`
(`a33_lock_uaf.em`) is refused (E3021).

### (4) How often, severity

Frequency of the Rust annoyance: UNVERIFIED (no counting source found). Severity: **medium**. The
Ember soundness hole is a separate, higher-priority matter (see "Found in passing").

### (5) What Rust's rule prevents, and eliminations

Prevents: reading lock-protected data after the lock is released, while another thread (or later code
on this thread) changes or frees it. `a33_lock_race.em` is exactly that failure.

(a) **Pythonic spelling: return values, not references, out of a lock.** A reference derived through a
guard gets the guard's region (the fix to the hole); returning one is refused with help that names the
copy, and returning a class handle out of a lock retains it, which is what Python does:

```ember
fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0].clone()
```

Cost: the copy (or one retain for a handle). Gives up nothing that was safe.

(b) **Smarter: no shared locked data at all for the common case.** The data lives on one thread and
others send it messages (Ember's `channel`, `[THR-15]`), or access happens only inside a closure the
lock runs (`scores.update(fn(g): …)`), so no reference can leave. Cost: a queue operation and a copy
of the message per access (channel form); none (closure form). Gives up: direct shared access.

---

## A34 (new) The clone-before-move dance for shared handles

### (1) Rust

`a34_clone_dance.rs`: two threads append to one log held in an `Arc`.

```text
error[E0382]: the type `Arc` does not implement `Copy`
5 |     let a = thread::spawn(move || log.lock().unwrap().push("a".to_string()));
  |                           ------- --- variable moved due to use in closure
6 |     let b = thread::spawn(move || log.lock().unwrap().push("b".to_string()));
  |                           ^^^^^^^ --- use occurs due to use in closure
help: consider cloning the value before moving it into the closure
5 ~     let value = log.clone();
```

Workaround: `let log2 = Arc::clone(&log);` before every closure.

### (2) Python

```python
threading.Thread(target=lambda: log.append("a")).start()
threading.Thread(target=lambda: log.append("b")).start()
```

### (3) Ember today: absent

`a34_clone_dance.em` compiles: `log` is a `@sync class` handle; handles are `Copy` and copying retains
(§IV.6, `[RC-1]`), so each `owned fn` capture takes its own count. Cost: an atomic increment per capture
and an acquire-release decrement per release (`[RC-4]`), the same as Rust's `Arc::clone`.

### (4) How often, severity

Frequency: UNVERIFIED; rustc's own help text above shows the expected fix is the clone. Severity:
**medium** in Rust; none in Ember.

### (5) What Rust's rule prevents, and eliminations

Prevents: one count serving two owners (a double release when both threads finish). Ember's implicit
retain on copy is already the Pythonic spelling (a). (b) A scoped task borrows the handle with no count
change at all (`thread.scope()`, `[THR-11]`), which Ember also has. Nothing to add.

---

## Found in passing (outside the catalogue; for the owner)

1. **Soundness hole: a reference out of a `RefCell` or `Mutex` guard outlives the guard.** Probes
   `a33_lock.em`, `a33_lock_uaf2.em`, `a33_lock_race.em`, `a09_refcell_escape.em` all pass `ember
   check`; Rust refuses the equivalent (E0515). `a33_lock_uaf2.em` is a single-threaded use-after-free
   by construction; `a33_lock_race.em` a data race. The guard types are compiler-known (`std/src/sync.em`
   lines 47-49: "The compiler knows these types by their names here … lowers `lock` and `try_lock` to
   the runtime's lock word"), so the fix is in the region given to a read-through of a guard. Not run.
2. **E1010 inside lambdas:** `x_capture_in_method_arg.em`: `f = fn() => total.fetch_add(n)` (an
   `Atomic` method) reports "cannot find `n` in this scope"; the same lambda calling a class method
   (`b.add(n)`) is accepted. Also seen as `a15_workaround.em` and `a15_workaround2.em`.
3. **E3026 reported in the callee:** `a12_closures.em`'s error points at `self.handlers.push(f)` inside
   `on_click`, not at the call that wrote the lambda. `[TYP-15]` says the store requirement passes to
   callers ("E3063 at the call otherwise"), and `[CLO-15]` says a lambda written at a consumed callable
   parameter whose state must be static "captures by move"; neither is what the compiler does here.
4. **Misleading message for a static:** `a19_globals.em`: `FRAME += 1` on `static FRAME: int` reports
   "E1010 cannot find `FRAME` in this scope"; it should say statics are immutable (`[STA-1]`) and name
   `Atomic[int]`.

## Tools

All tools needed were present: rustc 1.98.1, `target/ci/ember.exe`, WebSearch and WebFetch. Several web
sources were read through search summaries rather than in full; each such use is marked "search
summary". No program was downloaded or installed. No Ember program was run; one Rust program
(`a09_workaround.rs`) was built and run to get its real panic text.
