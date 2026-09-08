# Ember v0.6 — proposed

**Status:** proposal. Nothing here is normative until the owner rules on it.
**Depends on:** the v0.5 normative specification (`docs/spec/`).
**Principle:** safe by default, provable by request, native when necessary.

This revision takes eleven things from the Aegis design document and states them
in Ember's own grammar and rule numbering. Where v0.5 already covers a subject,
this document **amends the existing rule by id** rather than restating it — that
is the mistake that put six defects into v0.5 itself, and it is not repeated
here.

---

## 1. What changes for someone writing Ember

| You write | Today it does | Under v0.6 it does |
|---|---|---|
| `roughness: f32 = 2.0` passed to a material | compiles; the renderer draws something wrong | `type Roughness = f32 in 0.0 ..= 1.0` makes it a compile error, or a checked conversion you must handle |
| `metallic = roughness` | compiles, both are `f32` | rejected: they are different types |
| an index into a span inside a hot loop | bounds check survives unless the loop shape is one the compiler already recognises | the prover discharges it and the check is gone, with checks still on in `shipping` |
| `@noalloc` on a frame function | the compiler rejects a call that allocates | the same, and a build can omit the allocating half of the library so there is nothing to call |
| an overlay saying a C++ call does not allocate | believed | graded: asserted, checked, instrumented or proven, and the ungraded ones are listed |
| `const Image&` from a C++ header | imported as a borrow | imported unsafe until someone writes down how long it lives |
| "is this function still calling into C++?" | read the code | `ember calls --foreign Renderer.render` |

---

## 2. Range types

### What it does

A number that carries its own limits, and that cannot be confused with another
number that happens to share its representation.

```ember
type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0
type Fov       = f32 in 1.0 ..= 179.0
type Percent   = u8  in 0 ..= 100

r: Roughness = 0.5          ## proved at compile time, no check emitted
m: Metallic  = r            ## E2210: `Roughness` is not `Metallic`
```

When the value is not a constant, the conversion is fallible and must be
handled:

```ember
fn from_slider(x: f32) -> Result[Roughness, RangeError]:
    return Roughness.checked(x)

fn clamped(x: f32) -> Roughness:
    y = min(max(x, 0.0), 1.0)
    return Roughness.checked(y).unwrap()      ## the check is proved away
```

### Rules

* `[RNG-1]` A `type` alias with an `in` clause declares a **nominal** numeric
  type over the named representation, restricted to the given range. A `type`
  alias without one is transparent, exactly as `[LEX-15a]` already says. The
  clause takes a range expression (`a .. b` or `a ..= b`) whose ends are
  constant expressions of the representation type.
* `[RNG-2]` Two range types are distinct types even when their representation
  and range are identical (`E2210`). A range type converts to its
  representation implicitly; the reverse requires `[RNG-3]`.
* `[RNG-3]` Construction from a value the compiler cannot prove in range is
  `T.checked(v) -> Result[T, RangeError]`. Construction from a constant, or
  from a value whose known range is contained in the target's, emits no check.
* `[RNG-4]` The compiler tracks a known range for every numeric expression it
  can — literals, `min`/`max`/`clamp`, the bodies of `if` and `match` arms that
  compared the value, and arithmetic on operands with known ranges — and uses
  it to discharge `[RNG-3]`'s check and `[TYP-8]`'s overflow check. A range fact
  is never assumed from a `@fastmath` function.
* `[RNG-5]` Arithmetic on a range type yields its **representation**, not the
  range type: `r * 2.0` is `f32`. Producing a range type again is a
  construction and goes through `[RNG-3]`. Without this rule every operator
  would need a range-arithmetic result type, and `Roughness * Roughness` would
  have no honest answer.
* `[RNG-6]` Float range reasoning obeys `[TYP-9]`'s strict IEEE semantics. NaN
  is in no range. A range whose ends are `-0.0` and `+0.0` contains both zeros.
  Ranges over floats are compared after `[TYP-9a]`'s contraction prohibition,
  so a range fact cannot be invalidated by an FMA the backend introduced.
* `[RNG-7]` A range type is a niche for `[TYP-13]`: `Option[Percent]` occupies
  one byte.
* `E2210` a value of one range type where another was expected;
  `E2211` a constant outside the target's range;
  `E2212` an `in` clause whose ends are not constants of the representation, or
  are inverted;
  `E2213` an `in` clause on a non-numeric representation.

### Diagnostic shape

```text
error[E2211]: 1.4 is outside `Roughness`

    roughness: Roughness = 1.4
                           ^^^ `Roughness` holds 0.0 ..= 1.0

  = help: clamp it, or take the fallible form: `Roughness.checked(1.4)`
```

---

## 3. Contracts

### What it does

A function writes down what it needs from its caller and what it promises in
return. The compiler checks them while the program runs, hoists them where it
can, and later hands them to the prover.

```ember
@requires(index < buffer.len())
fn sample(buffer: Span[f32], index: usize) -> f32:
    return buffer[index]

@ensures(result >= 0.0)
@ensures(result <= 1.0)
fn clamp01(x: f32) -> f32:
    return min(max(x, 0.0), 1.0)

@requires(length(v) > 0.0)
@ensures(abs(length(result) - 1.0) < 0.0001)
fn normalize(v: Vec3) -> Vec3:
    return v / length(v)
```

**Contracts are attributes, not keywords.** `requires`, `ensures`, `invariant`
and `decreases` are ordinary identifiers today and are in neither the reserved
list nor the reserved-for-future list of `[LEX-15]`. Spelling them as clauses
would break any existing program that uses one as a name. As attributes they
need no lexical change at all, and they sit beside `@borrows`, which is already
a contract in everything but name.

### Rules

* `[CTR-1]` `@requires(expr)` is a precondition, checked in the **caller's**
  frame before the call, so a failure names the call site. `@ensures(expr)` is a
  postcondition, checked in the callee before each return. Both may appear more
  than once; the meaning is their conjunction.
* `[CTR-2]` Inside `@ensures`, the name `result` denotes the returned value. It
  is bound only inside that attribute and is an ordinary identifier everywhere
  else (`[LEX-15]` is unchanged). `old(place)` denotes the value a place held on
  entry, and is available only in `@ensures`.
* `[CTR-3]` A contract expression MUST be free of the `Alloc`, `Block`, `Io`,
  `Sync`, `FFI` and `Panic(Explicit)` effects (`E4050`). It may read `let`
  fields, call `@pure` functions and index views. A contract that could allocate
  or block cannot be checked in a frame path, which is where contracts are worth
  the most.
* `[CTR-4]` **Contract arithmetic is always checked, in every profile.** An
  overflow inside a contract expression is a contract failure, never a wrap.
  This is the rule that makes `[PRF-1]` survive: without it `@ensures(result >
  x)` is true under `debug`'s panicking arithmetic and false under `release`'s
  wrapping, and one source would have two meanings.
* `[CTR-5]` `@invariant(expr)` on a `struct` or `class` must hold after every
  constructor and after every method that takes `mut self`. On a loop it must
  hold on entry and after every iteration.
* `[CTR-6]` `@decreases(expr)` names a quantity that strictly decreases and is
  bounded below, for a loop or a recursive function. It is what lets `[PRV-*]`
  say anything at all about a loop; without it every loop is an unknown.
* `[CTR-7]` Contracts are part of the public interface. `[EFF-4]`'s caching rule
  applies: changing a contract invalidates callers. An `override` of a `virtual`
  method MUST NOT strengthen `@requires` or weaken `@ensures` — the same
  inheritance direction `[EFF-8]` uses for effects.
* `[CTR-8]` Contract checking is controlled by `--contracts=off|debug|all`,
  defaulting to `debug` in `debug`, `debug` in `release` and `off` in
  `shipping`. **This is a checking policy, not a semantic profile**: a contract
  that fails is a program with a bug in every setting, so `[PRF-1]` is not
  weakened by it. A contract discharged by `[PRV-*]` emits no check in any
  setting.
* `E4050` a contract expression with a forbidden effect;
  `E4051` a contract that cannot be evaluated (names a local, mutates);
  `E4052` an `override` strengthening a precondition;
  `E4053` `@decreases` on something that is not a loop or a recursive function.

---

## 4. The prover

### What it does

A solver tries to establish, before the program runs, that every contract and
every runtime check holds for every possible input. Where it succeeds the check
is deleted. For a renderer this is a **speed** feature first: `[OPT-2]` already
removes the bounds check from a counted loop whose shape the compiler
recognises, and this generalises it to the loops it cannot see through, with
checks still on in a shipping build.

```ember
@verified
@ensures(result >= 0.0)
@ensures(result <= 1.0)
fn clamp01(x: f32) -> f32:
    return min(max(x, 0.0), 1.0)
```

### Rules

* `[PRV-1]` `@verified` on a function asks for its contracts, its `[TYP-8]`
  overflow checks, its bounds checks and its `[RNG-3]` constructions to be
  discharged by proof. Every obligation has exactly one of three outcomes:
  **proved**, **disproved**, **unknown**. A solver timeout is unknown.
* `[PRV-2]` In a build that requires proof, **unknown is a failure**. An
  unproven obligation is never reported as proved, and never silently becomes a
  runtime check without saying so.
* `[PRV-3]` Proof is **modular**: a function is proved against the declared
  contracts of the functions it calls, not by inlining them. A call to a
  function with no contract contributes no facts.
* `[PRV-4]` Verification does not replace any other analysis. The borrow
  checker, the effect checker, the move analysis and the exclusivity analysis
  run unchanged and their results are not weakened by a proof.
* `[PRV-5]` A proved obligation may be removed from the emitted code. An
  obligation proved under an assumption (`[TCB-3]`) may be removed only when the
  assumption is recorded in the manifest.
* `[PRV-6]` Obligations are emitted in SMT-LIB form for an external solver. The
  compiler does not implement a prover. The solver's name, version and
  configuration are part of the build identity `[BLD-2]` and of the manifest, so
  the same source, compiler and solver give the same answer tomorrow.
* `[PRV-7]` A failed proof reports the source of the obligation, the values that
  break it where the solver supplies them, and which of four things went wrong:
  the code, an insufficient contract on a callee, an explicit assumption, or the
  solver giving up.
* `[PRV-8]` `--verify=off|check|prove`. **Verification is not a profile axis.**
  It is a flag orthogonal to `debug`/`release`/`shipping`, and `[PRF-1]` is
  untouched: verification never changes which branch is taken or what a program
  computes, only whether a check the program would have executed is still there.
* `E4060` an obligation disproved, with counterexample;
  `E4061` an obligation unknown in a build that requires proof;
  `E4062` `@verified` on a construct the obligation generator does not support —
  reported as unsupported, never as proved.

### Diagnostic shape

```text
error[E4060]: postcondition `result <= limit` does not hold

    fn clamp_speed(requested: Speed, limit: Speed) -> Speed
       ^^^^^^^^^^^

  counterexample:
      requested = 5.4
      limit     = 5.0
      result    = 5.4

  = note: the branch at line 63 returns `requested` without comparing it to `limit`
```

---

## 5. The proof manifest

* `[PRV-9]` A build with `--verify=prove` writes `target/<profile>/verify.json`:
  compiler version, standard-library version, target triple, solver name and
  version, the hash of every module, one entry per obligation with its id,
  source span, outcome and time, every assumption relied on, and the trusted
  components from `[TCB-*]`.
* `[PRV-10]` The manifest is evidence and a reproducibility record. It is not a
  certification artefact and the toolchain MUST NOT describe it as one.

---

## 6. The trust report, and grading

### What it does

A list of everything the compiler could not establish for itself, and — the
part Ember has no answer for today — **how strong each claim is**. Right now a
fact the importer derived from a header and a promise somebody typed into an
overlay are indistinguishable once written down.

```text
$ ember tcb

Trusted for memory safety
  borrow checker, RC runtime, arena allocator      language

Unsafe
  unsafe blocks                                    12
  raw pointer operations                            7

Foreign
  C functions                                      23   19 checked, 4 asserted
  C++ bridge calls                                 14    2 proven, 9 instrumented, 3 asserted
  still requiring `unsafe`                          3

Assumptions relied on (7)
  VulkanBackend  the destructor does not throw                       asserted
  VulkanBackend  submit() does not allocate                          instrumented
  vkGetDevice    returns a valid device                              asserted
  …
```

### Rules

* `[TCB-1]` Every foreign fact carries one of four **grades**:
  **asserted** (a human wrote it), **checked** (tooling derived or confirmed it
  from the header, the ABI or the build), **instrumented** (a test run observed
  it holding), **proven** (discharged by analysis of the adapter). A fact with
  no grade is asserted.
* `[TCB-2]` The report MUST separate what the language guarantees from what an
  external component supplies. A grade may never be raised by a declaration
  alone.
* `[TCB-3]` Every assumption a proof relied on and did not discharge appears in
  the report and in `[PRV-9]`'s manifest, with its source and grade.
* `[TCB-4]` Entries are categorised: language, compiler, runtime, standard
  library, unsafe, C FFI, C++ bridge, external library, GPU driver, hardware,
  solver, assumption. **The compiler is in the list**: its own defects are part
  of what a claim rests on, and the report says so rather than implying the
  toolchain is outside the question.
* `[CLI-11]` `ember tcb [--json] [--module <m>]` prints it. `ember audit`
  prints the one-page summary: the core safety checks, the effect totals, the
  foreign boundary and the verification counts.
* `[CLI-12]` `ember why --unsafe|--alloc|--block|--io|--ffi <path.to.fn>` prints
  the shortest call chain from that function to the thing named, using the
  effect sets `[EFF-1]` already computes.
* `[TST-12]` The conformance suite MUST contain a program for each grade and
  assert that the report gives it that grade, and MUST assert that a declaration
  alone never produces `checked` or higher.

---

## 7. A standard library you can build without its allocating half

### What it does

`@noalloc` says a function must not allocate. This goes one step further: a
build can leave the allocating and I/O parts of the library out entirely, so in
a frame path allocation is not discouraged, it is absent. Alongside it,
containers with a fixed capacity that never touch the heap.

```ember
## ember.toml
[package]
std-layers = ["core", "sync"]        ## no alloc, no io
```

```ember
commands: FixedArray[DrawCommand, 32]
name: FixedString[64]
history: RingBuffer[Frame, 128]
```

### Rules

* `[STD-6]` `std` is layered: **core** (primitives, `Option`, `Result`, views,
  fixed-capacity containers, math), **alloc** (`Array`, `String`, `Box`, `Map`),
  **sync** (`Atomic`, `Mutex`, channels), **io** (files, sockets), **ffi**,
  **verify** (proof-only helpers). A layer may depend only on layers above it in
  that order.
* `[STD-7]` `FixedArray[T, N]`, `FixedString[N]` and `RingBuffer[T, N]` are core
  types with no heap allocation, a compile-time capacity, and a full-container
  policy that is explicit at the call (`push` returns `Result`, `push_or_drop`
  does not). **They need const generics**, which v1 does not have; that
  dependency is real and is named here rather than discovered later.
* `[BLD-11]` A package declares which layers it links. Omitting `alloc` removes
  the allocating types from the build; a reference to one is a name-resolution
  error at compile time, not a lint (`E9030`). This is stronger than a coding
  rule that says do not allocate, and it is checkable by a person reading the
  manifest.
* `[BLD-12]` A package that omits a layer MUST NOT depend on a package that
  requires it. The build reports the dependency path that reintroduces it.

---

## 8. Finer effects

* `[EFF-18]` The effect set gains **`Io`** (file, socket, console) and
  **`Lock`** (acquiring a synchronisation primitive that can wait), separating
  them from `Block` and `Sync`. "This function touches the disk" and "this
  function might wait" become different questions, which is what a frame budget
  needs. `@noio` and `@nolock` are the corresponding contracts and behave like
  `@noalloc` under `[EFF-6]` and `[EFF-8]`.
* `[EFF-19]` `@realtime` on a function expands to a configured set — by default
  `@noalloc @nolock @noblock @nopanic(explicit)` — and is a **marker for that
  set, not a timing guarantee**. The compiler MUST NOT state or imply that a
  `@realtime` function meets a deadline: hardware, scheduling, cache behaviour
  and foreign code decide that, and none of them is visible here.

---

## 9. C and C++ interoperability

v0.5 already specifies the import machinery: `[FFI-17]`'s `import cpp … with
(classes, instantiate, overlay)`, the generated `extern "C"` glue, the
exception-catching wrapper of `[FFI-24]`, the standard-library mapping, the
overlay contract vocabulary of `[FFI-11]`, and the adoption gate of `[CLI-6]`.
**None of that is restated here.** What follows is what Aegis has and v0.5 does
not.

* `[FFI-34]` **Every fact in an overlay carries a `[TCB-1]` grade.** The grade
  is written per fact and defaults to asserted:
  ```ember
  overlay cpp "RageV/VulkanBackend.hpp":
      @ffi(effects=[FFI] @instrumented, threads=main @checked)
      fn submit(self: borrowed, cmd: borrowed) -> status
  ```
  A grade above asserted requires evidence: **checked** requires the fact to
  follow from the header, the ABI or the build configuration; **instrumented**
  requires a test run under `ember test --instrument-ffi` to have observed it;
  **proven** requires the adapter itself to have been analysed. Claiming a grade
  without its evidence is `E5050`.
* `[FFI-35]` **A C++ reference or raw pointer imports as unsafe.** `T&`,
  `const T&` and `T*` produce a foreign pointer that requires an `unsafe` block
  to use, until an overlay supplies the lifetime and aliasing facts that make it
  a `ref T`/`ref mut T`. A C++ header does not record how long a returned
  reference lives or who else may hold it, and mapping one to a safe borrow on
  the strength of its spelling is how a dangling pointer acquires the borrow
  checker's endorsement. This **narrows** `[FFI-11]`'s `borrowed`: that contract
  is now the thing a human writes to make the pointer safe, not the default.
* `[FFI-36]` **Ownership transfer is a line of code.** A C++ call that yields an
  object Ember must free returns a `Foreign[T]` — a raw foreign handle, unsafe
  to dereference — and becomes an owned Ember value only through
  `adopt(handle)`, written at the call site. The overlay says whether adoption
  is permitted and what destroys the object; it does not perform the transfer
  invisibly. In a review, the moment Ember became responsible for freeing
  something is a line you can point at.
* `[FFI-37]` **`ember test --instrument-ffi`** links a shim that records
  allocation, blocking, locking and I/O across every foreign call, and reports
  each declared effect fact as confirmed or contradicted. A contradicted fact
  fails the run and names the overlay line. This is what turns "submit does not
  allocate" from a promise into a measurement.
* `[CLI-13]` **`ember calls --foreign <path.to.fn>`** lists every foreign
  function reachable from that function, with the chain to each and its grades.
  Run against the render loop it is the number that says how much of the engine
  is still native, and it falls as subsystems are replaced.
* `[FFI-38]` The importer MUST reject rather than guess. When ownership,
  nullability, lifetime or exception behaviour cannot be established, the
  declaration imports unsafe and `[FFI-20]` reports why. Convenient-but-unsound
  is not an import mode.
* `E5050` a grade claimed without its evidence;
  `E5051` a foreign pointer used outside `unsafe` with no lifetime contract;
  `E5052` `adopt` on a handle the overlay does not permit adopting;
  `E5053` an instrumented run contradicting a declared effect.
* `[TST-13]` The suite MUST cover: each grade and its evidence requirement; a
  reference imported unsafe and then promoted by an overlay; adoption and
  double-adoption; an instrumented run that contradicts a claim; and the
  reachability report against a known call graph.

### Which C++ to move first

Not a rule — the criteria a migration uses, so the order stops being a
judgement call each time.

**Good candidates:** a clear boundary, few dependencies, bugs that cost real
time, no template machinery in the interface, ownership that is already
understood. In RageV: frame orchestration, gameplay, ECS systems, tools, editor
logic, resource bookkeeping.

**Poor candidates:** macro-generated code, a plugin ABI, pervasive global state,
shared ownership with custom deleters, template-heavy headers with undocumented
lifetimes. In RageV: the Vulkan and OpenGL backends, the platform layer, the
allocators. These stay native, and the bridge around them is where the contracts
and grades concentrate.

---

## 10. Three things to decide before any of this is written

1. **The syntax for contracts.** This document uses attributes
   (`@requires(...)`) so that `requires` and `ensures` stay ordinary
   identifiers. The alternative — clauses after the signature — reads better and
   breaks every existing program that uses one of those words as a name, unless
   they are reserved in a v0.5 patch first so that `[LEX-14a]`'s message can
   name the version. **Recommendation: attributes.**
2. **`in` as the range clause.** `type X = f32 in 0.0 ..= 1.0` uses only
   existing keywords, at the cost of `type` meaning two things — nominal with
   the clause, transparent without. The alternative is a new keyword `range`,
   which is a plausible variable name and would break programs.
   **Recommendation: `in`, with `[RNG-1]` stating the nominal/transparent split
   in one sentence.**
3. **Contract arithmetic.** `[CTR-4]` makes it always checked, so a contract
   means one thing in every profile. The alternative is to let contracts inherit
   `[TYP-8]`'s per-profile policy, under which `@ensures(result > x)` is true in
   `debug` and false in `release` — which would make `[PRF-1]` false.
   **Recommendation: `[CTR-4]` as written.**

---

## 11. What this amends

| Existing rule | Change |
|---|---|
| `[LEX-15a]` | a `type` alias with an `in` clause is nominal |
| `[TYP-8]` | `[RNG-4]`'s range facts discharge overflow checks; `[CTR-4]` exempts contract expressions from the profile's policy |
| `[TYP-13]` | a range type is a niche |
| `[EFF-*]` | `Io` and `Lock` join the effect set; `@noio`, `@nolock`, `@realtime` |
| `[OPT-2]` | generalised by `[PRV-1]` where a proof is available |
| `[PRF-1]` | unchanged, and `[CTR-4]` and `[PRV-8]` are what keep it true |
| `[FFI-11]` | `borrowed` becomes the promotion a human writes, not the default (`[FFI-35]`); every fact carries a grade (`[FFI-34]`) |
| `[FFI-17]` | a C++ call yielding an owned object returns `Foreign[T]`; `adopt` is explicit (`[FFI-36]`) |
| `[CLI-6]` | joined by `ember tcb`, `ember audit`, `ember why`, `ember calls` |
| `[STD-*]` | layered, with fixed-capacity containers in core |
| `[BLD-2]` | the solver identity joins the build identity |

---

## 12. Soundness requirements

1. A declaration is never evidence. No grade above asserted comes from a
   declaration alone.
2. Unknown is never success. A solver timeout does not discharge an obligation.
3. A proof never replaces the borrow checker, the effect checker or the move
   analysis.
4. A range check is removed only when a proof or a tracked range fact
   establishes it.
5. A C++ reference or pointer never becomes a safe borrow without a written
   lifetime.
6. Ownership transfer across the C++ boundary is always explicit in the source.
7. Contracts never weaken memory, ownership, borrowing or concurrency
   guarantees.
8. Every assumption a proof relied on appears in the report.
9. The compiler is inside the trusted base and the report says so.
10. A profile never changes what a program computes — `[PRF-1]` holds after
    every rule in this document.

---

## 13. Build order

Each step is useful on its own and none of them blocks the engine work.

1. **Range types** (`[RNG-*]`) — the smallest thing with a real bug class behind
   it, and the prover's first customer.
2. **Contracts, checked at run time** (`[CTR-*]`) — reuses the assert machinery
   `[TYP-8]` already emits.
3. **Finer effects and the report** (`[EFF-18]`, `[CLI-11..13]`) — reports over
   data the compiler already computes.
4. **Grading and instrumentation on the foreign boundary** (`[FFI-34..38]`) —
   needs the FFI work of Phase 5 underneath it.
5. **The library split and fixed containers** (`[STD-6..7]`, `[BLD-11..12]`) —
   needs const generics.
6. **The prover** (`[PRV-*]`) — last, because it needs contracts and ranges to
   have something to prove, and it is the only item with an external dependency.
