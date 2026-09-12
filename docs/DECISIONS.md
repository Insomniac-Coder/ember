# Ember — decision record

ADR format: context, decision, consequences, spec rule affected. One entry per
decision that is not already forced by the specification.

Per Part XXII.1 the nine questions below require an owner decision and an agent
MUST NOT choose one silently. On 2026-09-07 the owner confirmed all nine as the
specification writes them. Entries marked **deferred** are confirmed as a
direction but their concrete answer waits for the evidence named in the entry.

---

## ADR-001 — Float literals default to `f32`

**Spec rule:** `[LEX-17]`, Part 0 #12.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** An unsuffixed float literal has to land on some type when no
context supplies one. C, Python and Rust all land on 64-bit. The alternative
is `f64` with `f32` suffixes written throughout engine code.

**Decision.** Untyped float literals resolve to `f32` absent context.
`1.0f64` or an annotation selects `f64`.

**Consequences.** Graphics and engine arithmetic — the reference workload —
never accidentally promotes a temporary to double. Numeric code ported from
C or Python that assumes double precision will silently lose it unless
annotated; the risk is real and is accepted. `[LEX-16]`/`[LEX-17]` are the
only literal coercions, so the rule is at least uniform.

---

## ADR-002 — Block scoping, not Python function scoping

**Spec rule:** Part 0 #13, `[GRM-4]`, `[DRP-2]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** Deterministic destruction needs a defined point at which a value
dies. Function scoping (Python) has no such point inside a function body.

**Decision.** Block scoping. Values drop at end of block in reverse
declaration order. NLL governs borrows.

**Consequences.** Required for `with`, `defer` and RAII to mean anything.
Combined with `[GRM-4]` (`x = expr` declares when `x` is not in scope) it
carries a cost worth naming: a misspelled name inside a nested block declares
a fresh local rather than failing, and the outer variable is left unwritten.
This is Python's oldest footgun in a language whose premise is that the
compiler catches such things. A lint could recover most of it — "assignment
declares a new binding that is never read" — but no such lint is specified.
Raised for the record; the rule stands.

---

## ADR-003 — `pub` for visibility

**Spec rule:** Part 0 #11, `[MOD-2]`, `[MOD-7]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** `export`, `public` and `pub` were all candidates.

**Decision.** `pub`, with `pub(package)` and the field-only `pub(read)`.

**Consequences.** Short, reads naturally on fields. Familiar to Rust readers,
unfamiliar to the Python and C# readers who are the scripting audience.

---

## ADR-004 — Class exclusivity checks stay enabled in `release`

**Spec rule:** `[EXC-1]`, `[EXC-2]`, `[PRF-1]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** Overlapping mutable access through aliasing class handles is
memory-unsafe. The check is a load, a compare and a store on a header word
(~2 ns uncontended). Swift enforces it in release; the alternative is
debug-only checking and undefined behaviour in release.

**Decision.** Checked in `debug` and `release`. `exclusivity = "unchecked"`
is permitted only in the `shipping` profile, where a violation is UB.

**Consequences.** Object-world code carries a small, predictable per-access
cost in shipped-but-not-shipping builds. Classes are documented as unsuitable
for inner loops; the data-oriented facilities in Part XII are the answer
there. `[PRF-1]` already lists this as one of the three permitted
profile-dependent semantic differences.

---

## ADR-005 — Generic arguments use `[]`, not `<>`

**Spec rule:** `[GRM-8]`, `[GRM-9]`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** `<>` forces the parser to disambiguate against comparison
operators (C++'s and Rust's problem; Rust pays for it with the turbofish).
`[]` collides with indexing instead.

**Decision.** `[]`. `name[...]` in expression position parses to a single
`IndexOrInstantiate` node and is resolved during name resolution by what
`name` resolves to.

**Consequences.** Keeps Python's look, and type position needs no
disambiguation at all. The cost is an AST node whose meaning is unknown until
after name resolution, so the parser cannot report "indexing a non-indexable
type" and index expressions cannot be folded pre-resolution. Accepted.

---

## ADR-006 — C11 backend first, LLVM in v2

**Spec rule:** Part 0 #7, Part XVIII.6, XVIII.7.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** LLVM first delays a working toolchain and RageV integration by an
estimated one to two phases and requires an LLVM install at every consumer
site. Without a GC there are no safepoints, so C loses nothing essential.

**Decision.** `ember_codegen_c` is the v1 backend. `ember_codegen_llvm`
begins only when the C backend passes the full conformance suite, and becomes
default only when it also passes the performance suite.

**Consequences.** MSVC is a first-class target, which costs the workarounds
already named in Part XVIII.6 (`ember_i128`, `ember_ck_*` for checked
arithmetic, `_Alignas` handling). Console toolchains remain reachable.
Optimisation reports, PGO and precise vectorisation wait for v2.

---

## ADR-007 — Reference cycles leak, with tooling rather than a collector

**Spec rule:** `[WK-1]`, `[WK-2]`, `L3001`.
**Status:** confirmed by the owner, 2026-09-07.

**Context.** Reference counting cannot reclaim cycles. The alternative is a
backup cycle collector (CPython's approach), which adds a tracing pass to a
runtime that otherwise has none.

**Decision.** Cycles leak. This is documented behaviour, not a defect.
Mitigations: `Weak[C]` for back-pointers, the debug runtime's cycle report
(`ember run --leak-check`) walking the intrusive live-object list at exit and
printing strongly connected components with the field names that form them,
and lint `L3001` for statically visible cycles.

**Consequences.** The runtime stays free of tracing. Long-running processes
with programmer-authored cycles grow. The debug live-object list is what makes
this tolerable, so it is not an optional feature — it is load bearing and must
land with Phase 3.

---

## ADR-008 — RageV ECS integration shape — **deferred to measurement**

**Spec rule:** Part XXI.3, `[ECS-2]`.
**Status:** direction confirmed 2026-09-07; the answer waits for Phase 6.

**Context.** Two shapes. (a) Bridge: the engine exposes pool data
(`entities*, count, components*, stride`) and Ember wraps it as a `Query` over
borrowed `Span`s, zero-copy, with systems running inside `Scene::Update`.
(b) Ember-owned ECS: `std.ecs.World` becomes the store and the engine's C++
systems read back through the C API.

**Decision.** (a) is the default and (b) is attempted only if (a) measures
unacceptably. The measurement is the pass criterion in XXI.6: `Get<T>` in two
loads, iteration order stable, transform walk within 1.05x of C++.

**Consequences.** `[ECS-2]`'s component identity must be a 64-bit FNV-1a hash
of the fully-qualified type name, byte-identical to RageV's `TypeHash`, or (a)
is impossible. That constraint binds from Phase 6, and any earlier design that
would break it is a defect.

---

## ADR-009 — Language and CLI naming — **open, check pending**

**Spec rule:** Part XXII.1 #9.
**Status:** confirmed as a task, not as an answer. Owner asked for the check.

**Context.** The specification asks for a conflict check against crates.io,
PyPI and npm before anything is published. `ember` is also the name of a
large, long-established JavaScript framework (Ember.js), which at minimum
contends for the `ember` name on npm and for search results generally. The
names at stake are the CLI (`ember`), the compiler crate (`emberc`), the
manifest (`ember.toml`), the source extension (`.em`) and the runtime
(`ember_rt`).

**Decision.** Not yet taken. The check runs before the first published
artefact. Nothing before publication depends on the name being final, but the
cost of changing it rises with every test annotation, CMake module and
documentation page written.

**Consequences.** If the name changes, the rename touches crate names, the CLI
binary, `ember.toml`, the `.em`/`.embind` extensions, `EMBER_RT_ABI` and every
`ember_*` runtime symbol, `cmake/EmberModule.cmake`, and every `#!` test
annotation. Mechanical, but wide. Recorded here so the decision is not made by
default.

---

## ADR-010 — Regions are location-insensitive, and a reference local is assigned once

**Spec rule:** Part XVIII §4.7 steps 1–4, `[LT-5]`.
**Status:** taken 2026-09-09, on the permission §4.7 gives.

**Context.** §4.7 says "Polonius-style location-sensitive reasoning is not
required for v1", so a constraint may propagate a whole point set rather than
the points reachable from where the assignment sits. What that costs is
precision in one shape: a reference local assigned twice, where the second
loan's region would absorb the first's live range and the first would look live
during it.

**Decision.** Constraints propagate whole point sets. The imprecision is
unreachable in Ember, because a reference local cannot be re-seated: `r = ref
mut m` writes *through* `r` (`[TYP-14]`), so every reference local is assigned
exactly once, at its declaration.

**Consequences.** The inference is a fixpoint over one edge list rather than a
reachability computation per constraint, which is most of why the pass is short.
**If a future revision adds a way to re-point a reference** — a `ref` field
assigned twice, a reference in a container, a loop that rebinds one — this
decision expires with it, and the checker will report a borrow as live across a
range where it is dead. The module header in
`compiler/ember_analysis/src/regions.rs` states the assumption where it is
relied on.

---

## ADR-011 — `E3060` for storage, `E3062` for elision

**Spec rule:** Part XVIII §4.7 step 6, `[LT-1]`, `[LT-1a]`, ERR-024.
**Status:** taken 2026-09-09.

**Context.** §4.7 step 6 names two codes and a returned borrow of a local
satisfies the description of both: its region outlives the borrowed place's
storage (`E3060`), and it is a return that derives from no parameter
(`E3062`). Shape B7 is keyed to the first and B6 to the second, so the choice
decides which `help` the programmer is shown.

**Decision.** The split is by *what is wrong*, not by where it was noticed.
`E3060` when the borrowed storage ends first — a local, or a parameter passed
by value (ERR-024). `E3062` when the storage outlives the call but elision does
not tie the return to it — `@borrows` naming a different parameter, or
`[LT-1]` rule 1's borrowing receiver where the result points into an argument.

**Consequences.** The two diagnostics say different things and neither is a
fallback for the other: B7 tells you the value dies too soon, B6 tells you the
signature does not permit what the body did. A program can hit both, and does
— `tests/compile-fail/a_returned_view_elision_cannot_tie.em` carries one of
each. The cost is that "returned reference does not derive from a parameter",
`E3062`'s registry title, is now narrower than its wording: a borrow of a local
derives from no parameter either and is `E3060`.

---

## ADR-012 — `E3064` stays unemitted until a program needs it

> **Superseded, 2026-09-09.** The question is answered: `E3064` is reachable,
> and the programs it is for were being reported as B3. `[LT-2]`'s one-region
> model is what makes them reachable — a view struct bundling two views holds
> both loans while any part of it is live. The compiler was rejecting the right
> programs under the wrong shape, so neither the rule nor the narrowing was
> wrong; the classifier was. See D-011.
>
> **Reopened by the owner, 2026-09-09.** This ADR is a decision to *defer*,
> not a finding that `E3064` is unreachable. I had concluded the latter and
> written it into `[LT-2]` as amendment A4; the owner withdrew both, on the
> grounds that the evidence does not yet distinguish "the compiler narrows
> something the rule means to reject" from "the rule is stricter than it needs
> to be". Neither side is to be changed until it does. See D-011.

**Spec rule:** `[LT-2]`, `[DIA-7a]`, §XIX.6.1 shape B13.
**Status:** taken 2026-09-09. Revisit when `[TYP-15a]`'s `BorrowList`/`ViewList`
land, which is where two element regions could first be asked for.

**Context.** `E3064` "two independent regions in one view struct" is registered
and keyed to shape B13. `[LT-2]` says a view struct built from several
references takes the **intersection** of their regions — so construction always
has an answer, and the compiler narrows rather than refusing. No program has
been found that reaches the error.

**Decision.** Leave it unemitted and record why, rather than invent a trigger.
`docs/DEFECTS.md` carries it as an open row.

**Consequences.** `[DIA-7a]` is satisfied either way — it forbids emitting a
code that is *absent* from the shape table, not leaving a present one unused.
The risk accepted is that a program which should be rejected is quietly
narrowed instead; the intersection is sound, so it is a precision loss and not
a soundness one. **The alternative rejected:** writing a diagnostic against a
guessed trigger, which enforces the rule in a shape nobody asked for and is
harder to remove later than to add now.

---

## ADR-013 — `@ffi(no_virtual_dtor)` is not adopted; `owner=` alone carries it

**Spec rule:** `[FFI-17d]`, `[FFI-39]`, `E5059`. Errata ERR-034.
**Status:** provisional, taken 2026-09-09. Phase 7. Flagged for the owner.

**Context.** `[FFI-17d]` says a `@ffi(trampoline)` type whose base has no
virtual destructor MUST declare `owner=`, and that "`[FFI-17b]`'s
`@ffi(no_virtual_dtor)` covers the remaining case". `[FFI-17b]` is about
template instantiations and says nothing about destructors; the attribute is
named nowhere else, and Part III §7 has no row for it, so `[ATT-1]` makes
writing it `E0104`. There is no way to tell which rule was meant, because no
rule defines it.

**Decision.** Read `[FFI-17d]` as requiring `owner=` and nothing else.
`@ffi(no_virtual_dtor)` is not recognised. `E5059` fires when a
`@ffi(trampoline)` base has no virtual destructor and no `owner=`, which is
what its registry title says.

**Consequences.** Under `owner="foreign"` Ember never runs the C++ destructor,
so a missing virtual destructor is the host's problem and always was. Under
`owner="ember"` the pair is destroyed through the generated trampoline
subclass, whose own destructor is the derived one, so the non-virtual base
destructor is never the one called through a base pointer Ember holds. The
attribute therefore has no case left to cover. **If the owner rules the other
way**, Part III §7 gains a row and some `[FFI-*]` id gains the text; nothing
implemented before Phase 7 depends on the answer.

---

## ADR-014 — `extern "C"` belongs on `fn_header`

**Spec rule:** Part III §2, `[FFI-26]`, `[FFI-28]`, `[FFI-31b]`, `[HR-21]`.
Errata ERR-036.
**Status:** provisional, taken 2026-09-09. Phase 5. Flagged for the owner; the
grammar is **not** changed until the owner rules.

**Context.** XVI.10 writes `pub extern "C" fn on_update(entity: u64, dt: f32)
-> i32`, and `[FFI-28]`, `[FFI-31b]` and `[HR-21]` all reason about `@export`ed
functions with the C calling convention. Part III §2's `fn_header` has no
`extern` prefix; `extern` may precede `fn` only in `fn_type` (a function
*type*) and `extern_block` (foreign functions Ember calls, not Ember functions
foreign code calls). The example parses under no production.

**Decision.** Read the prefix as belonging on `fn_header`, after the optional
`unsafe` and admitted only at item level on a function with no generic
parameters — a calling convention has no meaning for an uninstantiated generic.

**Consequences.** It is the smallest change; it is what all three examples
write; it keeps the convention visible at the declaration, which `[PHIL-6]`
asks for; and it leaves `[FN-6]`'s coercion of a capture-free function to
`extern "C" fn(A) -> R` untouched, because that is a coercion of a *value* and
this is a *definition*. **The alternative rejected:** letting `@export("name")`
alone imply the C ABI. Fewer tokens, but the convention becomes invisible at
the declaration and `[MNG-2]`'s "`@export` overrides the symbol entirely" would
have to grow a second meaning.

---

## ADR-015 — `yield` in operand position is `E0100`, not `E0107`

**Spec rule:** `[GRM-22]`, `[GRM-16]`, `[LEX-15b]`.
**Status:** taken 2026-09-09.

**Context.** `[GRM-22]` gives `yield` "the grammatical position and precedence
of `return`", which is the lowest there is, so `1 + yield 2` has no parse. But
`yield` is not a jump — its type is the coroutine's resume type, not `!`, which
is exactly why `x = yield e` is legal where `x = return e` is not. `E0107`'s
message is "a jump expression may not be an operand" and `[GRM-16]` defines it
over `return`, `break` and `continue`. The document names no code for `yield`
in operand position.

**Decision.** `E0100`, the parser's ordinary "unexpected token", with a
primary label naming the precedence and a `help` that names the fix
(`v = yield e`, then use `v`).

**Consequences.** No code is invented, and `E0107` keeps one meaning, which is
what `[DIA-6a]` requires. The guiding sentence in "How to use this document"
governs: this is a diagnostic's wording, which the specification leaves
genuinely open, so the rule is simplest-to-implement-soundly and
most-predictable. **The alternative rejected:** widening `E0107` to cover
`yield`. It would make one code mean two things and would tell the programmer
that `yield` is a jump, which is the one thing about it that is not true.

---

## ADR-016 — `[RNG-5a1]`'s generated operator impls are implemented as behaviour, not as impls

**Spec rule:** `[RNG-5]`, `[RNG-5a1]`, `[RNG-5a2]`, `[TYP-20]`, `[TYP-21]`.
**Status:** taken 2026-09-09. Revisit when scalar operators go through the
operator interfaces.

**Context.** `[RNG-5a1]` specifies operators on range types as **generated
impls**: for every range type `T` over representation `R` the compiler emits,
*in `T`'s declaring module*, `T: Add[T, Output = R]`, `T: Add[R, Output = R]`
and `R: Add[T, Output = R]`, plus the `Sub`, `Mul`, `Div`, `Rem`, `Neg`,
`PartialEq` and `PartialOrd` forms, "defined by erasing each operand to `R`
(`[TYP-5]`) and applying `R`'s operator". Emitting them in the declaring module
is what satisfies `[TYP-20]`'s orphan rule without an exemption.

This compiler resolves operators on **scalars** as built-in operations, not
through `Add`. `[TYP-21]` is implemented for user types — `synth_binary` looks
up an `add` method and calls it — and falls through to the built-in path when
the operand is a scalar. There is therefore nowhere for `f32: Add[Roughness]`
to live: `f32` has no impl table.

**Decision.** Implement the rule's **observable content** exactly, and not its
mechanism. Specifically:

* `T op T`, `T op R` and `R op T` are accepted for the arithmetic and
  comparison operators, and each erases both operands to `R` and applies `R`'s
  operator — which is precisely how `[RNG-5a1]` defines the generated impls;
* the result is `R`, per `[RNG-5]`;
* two **distinct** range types resolve to no impl and are `E2214`;
* no `*Assign` form exists, so `r += 1.0` is `E2214` with the `help` that names
  `r = Roughness.clamped(r + 0.1)`;
* `[RNG-5a2]`'s "exact impl before coercion" is honoured by checking the
  operand types **before** any `[TYP-5]` coercion runs — which is why the
  range case sits above the untyped-literal rules in `synth_binary` rather
  than below them.

**Consequences.** Every program is accepted or rejected as `[RNG-5a1]`
requires, and every result has the type it requires. One thing differs and is
recorded here rather than discovered later: a user cannot today write
`extend Roughness implements Add[Vec3]:` and have it participate, because the
generated impls are not in an impl table for the resolver to see alongside it.
`[TYP-20]`'s orphan rule would place such an impl in `Roughness`'s declaring
module, so the conflict is reachable in principle. It is unreachable in
practice until operator interfaces cover scalars, at which point the honest
implementation is to generate the impls for real and delete the special case.

**The alternative rejected:** making scalars carry impl tables now, so the
generated impls have somewhere to live. That is a change to `[TYP-21]`'s whole
implementation for one feature's benefit, it costs a table lookup on every
`i32 + i32` unless it is then specialised back out, and `[BUD-5]` gives the
compile-time budget veto power over exactly this kind of change. The narrow
form is recorded and can be widened; the wide form cannot be narrowed.

---

## ADR-017 — `mut xs: MutSpan[T]` passes the view by value

**Spec rule:** `[FN-1]`, `[SPN-1]`, `[SPN-3]`, Part VII §7's worked example.
**Status:** taken 2026-09-09.

**Context.** `[FN-1]` says of the `mut` mode: "**inout** (mutable borrow). The
argument MUST be a mutable place; the callee may mutate; no move out." Every
other `mut` parameter is therefore a `ref mut T` inside the callee, and the
call site passes an address.

Part VII §7 then writes:

```ember
fn normalize(mut xs: MutSpan[f32]):
    total = xs.iter().sum()
    for x in xs.iter_mut():
        x /= total

buf = Array[f32]([1, 2, 3])
normalize(buf.as_mut_span())          # or simply normalize(buf)
left, right = buf.as_mut_span().split_at(1)
```

`buf.as_mut_span()` is a **call result**. It is not a place, and under the
uniform reading of `[FN-1]` the document's own example is `E2140`.

**Decision.** Where a `mut` parameter's declared type is `MutSpan[T]`, the view
is passed **by value** and is not wrapped in a `ref mut`. `mut` on it means
"the callee may write through this view", and `[FN-1]`'s mutable-place
requirement lands on whatever the view was taken *of* — which `[SPN-1]`'s
coercion checks when it builds one from an `Array[T]` or a `[T; N]`.

**Why this is the reading rather than a relaxation.** A `MutSpan[T]` already
*is* a mutable borrow: it carries the pointer, and `[SPN-3]` makes it move-only
precisely so that there is exactly one of it, which is the same guarantee
`[BRW-1]` gets from `ref mut`. Wrapping one in a `ref mut` would make a
reference to a reference, and the second level would guarantee nothing the
first does not. `[BRW-6]`'s "Passing a `ref mut` local to a `mut` parameter
reborrows rather than moving it" is the same observation about the same shape.

**Consequences.** `normalize(buf)`, `normalize(buf.as_mut_span())` and
`normalize(v)` for a `MutSpan` local all compile, which is the set Part VII §7
shows. Reassigning the parameter inside the callee — `xs = other` — rebinds the
callee's own view and is not visible to the caller, which is what `[SPN-3]`'s
move-only-ness already implies and what a `ref mut` to a view would have made
confusingly different.

**The alternative rejected:** spilling the coerced view into a temporary and
passing its address, so that `mut` stays uniformly `ref mut`. It compiles
`normalize(buf)` and still rejects `normalize(buf.as_mut_span())`, so it does
not save the example; and it adds an indirection to every element access in
exactly the code `[SPN-*]` exists to make fast.

---

## ADR-018 — `fn(A) -> R` is a function pointer in this phase, not a generic — **SUPERSEDED, closed 2026-09-09**

> **Closed.** The owner ruled that the rule stands and the compiler catches up:
> *"Do not change the Ember spec to accommodate the current compiler
> implementation."* `fn(A) -> R` in parameter position is now an implicit
> generic bounded by `Callable`, monomorphised per argument type; a capturing
> lambda is an anonymous struct whose fields are its `[CLO-2]` captures, and
> calling one is a direct call with the environment first. `[FN-6]` keeps
> `fn(A) -> R` an ordinary function-pointer type everywhere *except* parameter
> position, which is what lets a `fn`-typed local and a capturing argument
> coexist. `[CLO-6]`'s `owned f` is refused rather than mis-compiled: the
> consumption is a property of the bound, not of the closure's type, and until
> a call can consume its callee, treating `CallableOnce` as `Callable` would
> permit the second call the rule forbids. The clarification that was missing
> is amendment A5 in `docs/spec-amendments.md`.

**Spec rule:** `[CLO-1]`, `[CLO-3]`, `[COST-3]`, `[FN-6]`.
**Status:** taken 2026-09-09. **This is the deviation that capturing closures
close**; it is not permanent and the upgrade is described below.

**Context.** `[CLO-3]` says: "A parameter declared `f: fn(A) -> R` is a generic
over `Callable` (static dispatch, monomorphised)." `[CLO-1]` says a closure's
type is "a unique anonymous struct type implementing `Callable`", holding its
captures. Together they mean `fn(A) -> R` in parameter position is sugar for a
generic parameter bounded by `Callable`, and each closure type instantiates it.

This compiler implements `TyKind::Fn { params, ret }` as a **concrete function
pointer**. A named function and a capture-free closure both fit it, because
both are code with no environment; a capturing closure does not, because it has
one.

**Decision.** Keep the function-pointer reading for now, and refuse a capturing
closure by name — with a diagnostic that says what it is and what to do —
rather than compiling it to something that silently drops the captures.

**What differs, precisely.**

* **Accepted programs.** A capturing closure is rejected. Nothing else
  differs: every program the generic reading accepts and this one also accepts
  behaves identically.
* **Cost.** `[COST-3]` classes "generic call, specialised" as *not observable*
  — "direct call, inlinable". A function pointer is one **indirect** call. So
  a program that compiles under both is slower here than the specification
  allows, which `[COST-1]`'s definition of zero-cost cares about even though
  no test can currently see it.

**The upgrade, which is the same work as capturing closures.**

1. A closure literal builds a **synthetic struct** whose fields are its
   captures — `ref T` for a read-only capture, `ref mut T` for a mutated one
   (`[CLO-2]`) — with a synthesised `call` method. `[CLO-1]`'s "it is a view
   type if it captures anything by reference" then falls out of `is_view`,
   which already reports a struct with a `ref` field as one, and `[CLO-4]`'s
   escape rule falls out of `[TYP-15]`.
2. A parameter written `f: fn(A) -> R` becomes a synthetic **generic
   parameter** carrying that signature as its bound.
3. `f(args)` in the body resolves to the parameter's `call`, which
   `synth_bound_method` already does for a bound's method.
4. Instantiation re-checks the body with the parameter bound to the closure's
   struct type, which `check_instantiations` already does for every other
   generic.

Steps 3 and 4 are existing machinery. Step 2 is the new part, and step 1 is
where `[CLO-2]`'s capture inference lives.

**Why it is not done here.** It is a redesign of the type a `fn(A) -> R`
annotation produces, reaching the signature collector, the call checker, the
instantiation cache and the backend. Half of it — closures that build an
environment and a `fn(A) -> R` that is still a pointer — would compile
programs that drop captures on the floor, which is worse than refusing them.

## ADR-019 — `Cell`, `RefCell` and `Arena` are compiler-known, not built on `UnsafeCell`

**Decided 2026-09-09.** An implementation choice, not a language question:
`[CELL-2]` already fixes the observable behaviour, and this decides only what
builds it.

**The fork.** `[CELL-1]`'s `set` takes `self` — a *shared* borrow — and writes.
Nothing in Safe Ember can do that, so something has to be the exception. Two
candidates:

1. **`UnsafeCell`**, which `[CELL-9]` names, with `Cell`/`RefCell`/`Arena`
   written in Ember on top of it.
2. **Compiler-known types with builtins**, as `Array`, `String`, `Option`,
   `Result` and `Span` already are under Part XX.1 — "compiler-known until
   Phase 2's generics let the standard library write them".

**Chosen: 2, and the reason is that 1 is not available.** `UnsafeCell` appears
**exactly once** in the whole document — in `[CELL-9]`, as a citation to
`[UNS-*]` — and no rule defines it. `[UNS-5]`, which enumerates what `std.mem`
provides, does not list it. Building on it would mean inventing its semantics,
and its semantics are the one construct that suspends `[BRW-1]`: that is
answering what Ember means, which is the owner's to do. Filed as ERR-043.

**And `[CELL-9]` does not actually ask for route 1.** Its sentence is about a
*user package*: "A package that requires an interior-mutability primitive with
no check uses `unsafe` (`UnsafeCell`)". That governs what a third party may
build, not how `std`'s own `Cell` is built. Route 2 is consistent with it.

**What route 2 costs.** `UnsafeCell` stays undefined, so a *user* package cannot
write its own interior-mutability primitive. That is a real limitation and it is
the owner's to lift; it blocks nothing in `std`.

**What makes it sound.** `[CELL-2]` — "`Cell` never hands out a reference to its
contents, so no aliasing rule can be violated and **no runtime check is
needed**". Because nothing escapes, the builtin's write is not an aliasing
question at all, and the borrow checker can be told that directly rather than
being weakened. `RefCell` moves the same question to a counter (`[CELL-5]`),
`Arena` to a region (`[ARN-1]`); amendment A13 records that the three share the
implementation concern and not the concept.

**What would expire this.** Interface generics good enough for `std` to write
these in Ember, plus an owner-defined `UnsafeCell`. Then all three move out of
the compiler, and `[CELL-9]`'s sentence starts governing `std` too.

**What this ADR must never become.** It decides *how the standard `Cell` is
recognised by the compiler*. It does **not** decide what the language permits an
arbitrary package to implement, and it must not later be cited as though it did.
The failure mode to guard against is precise: someone reaches ERR-043, finds
that the compiler already mutates through a shared borrow for `Cell`, and
concludes that the language therefore permits it — turning an implementation
mechanism into the justification for a language rule. That is the same
compiler-justifies-specification move this project spent a session unlearning,
arriving by a longer road. `UnsafeCell`'s semantics remain unwritten and remain
the owner's, and the existence of `Builtin::CellSet` is not evidence about
them.


## ADR-020 — the overwrite drop is inserted in lowering and left to `[OWN-3]` to elaborate

**Decided 2026-09-09**, fixing D-035. `[OWN-5]` forces the *order* — evaluate
the new value, drop the old one, store — and says nothing about the mechanism.
Two parts of the mechanism are decisions, and this records them.

**The new value goes through a temporary.** `x = f(x)` and `x = f()` both
arrive from a call, and a call writes its result through a `Terminator`, not a
statement — there is no point "after the call and before the store" to push a
`Drop` into, because the call *is* the store. Routing every droppable
assignment through a temporary creates that point. The temporary costs nothing
in the emitted C: storing it into the place is a move, so `[OWN-3]`'s
elaboration deletes its own statement-end drop, and the backend's copy is the
one the assignment was going to make anyway.

The alternative — special-casing call-shaped right-hand sides and pushing the
drop before the call — was rejected because it puts the drop *before* the new
value is evaluated, which is the one ordering `[OWN-5]`'s parenthetical exists
to forbid, and because "call-shaped" is a growing list.

**Which drops survive is not decided here.** The inserted `Drop` is
unconditional and `drops.rs` then does what it already does for every other
drop: a local that is `Moved` at that point loses the statement, a `Maybe`
local gets a drop flag, a `Live` one keeps it. That is what makes a first
initialisation free — a local is `Moved` immediately after `StorageLive`, so
the drop at its initialising assignment is deleted — without lowering having to
know anything about initialisation.

The alternative was to ask, in lowering, whether the place is live. Lowering
does not know: liveness is a dataflow fact over the whole body, and the answer
on one path differs from the answer on another. Deciding it early would have
meant either a second liveness analysis or a conservative guess, and both
directions of a guess are wrong — dropping an uninitialised place is a crash,
and not dropping a live one is the leak this fixes.

**What this does not decide.** `[CELL-1]` requires the *opposite* order for
`Cell.set` and `Cell.replace` — store the new value, **then** drop the old —
because a drop can re-enter the same cell and observe it uninitialised. That is
not an exception to `[OWN-5]` granted here; it is a separate rule about a
separate operation, and it is why `Cell`'s store is its own builtin rather than
an ordinary assignment. Nothing in this ADR should be read as permission to
vary `[OWN-5]`'s order anywhere else.

## ADR-021 — `RefCell[T]` is move-only, and `[CELL-4]` does not reach it

**Owner ruling, 2026-09-10.** Recorded as an ADR as well as amendment S3
because the *shape* of the decision matters more than its content, and the next
person to build a compiler-known container will meet it again.

**How it came up.** `Cell[T]` was built as a transparent one-field struct
precisely so `[CELL-4]` would need no code: `is_copy` on a struct already asks
whether every field is `Copy`, so the answer falls out. That was the right
design and it is why `Cell` cost almost nothing. Applied mechanically to
`RefCell`, the same derivation makes the cell `Copy` whenever `T` is.

**Why that is wrong, in the owner's words.** *"Copying the value would
duplicate that state and therefore create two logically independent cells with
inconsistent knowledge of the same storage/borrow state. That would make the
runtime borrow invariant unsound."* `Cell`'s payload is inert; `RefCell`'s
counter is *coupled to the storage it guards*, so duplicating the value forks
the guarantee.

**The decision.** `RefCell[T]` is never `Copy`, whatever `T` is. Moving one
transfers the whole cell, borrow state included. Copying the contained `T` is a
separate question and unaffected. `[CELL-12]` states the exception in the
document rather than leaving an implementer to derive it.

**What this ADR is really about.** A derivation that is free and correct for one
type is not thereby correct for the next one that reuses its machinery. The
`Cell` implementation is a template for `RefCell`'s *shape* and explicitly not
for its `Copy` answer, and `docs/HANDOFF.md` says so at the point where the
template is recommended.

**Provenance, kept deliberately.** An earlier revision of `HANDOFF.md` asserted
"`RefCell` is never `Copy`" as settled, when Part IX said nothing about it. The
reasoning was sound; the sourcing was invented — the shape of both withdrawn
amendments, committed by the agent that had just written the section warning
against it. It was corrected to an open question, escalated, and is normative
now only because the owner ruled. **The correct answer arrived at incorrectly is
still incorrect**, and the route mattered more than the destination.

## ADR-022 — `UnsafeCell[T]` is a real primitive, and `Builtin::CellSet` never was evidence about it

**Owner ruling, 2026-09-10**, resolving ERR-043. Amendment S2 carries the
normative text; this records why the question could not be answered any other
way, and what it retires.

**The hole.** `UnsafeCell` appeared exactly once in 5,526 lines — `[CELL-9]`,
naming it as *the* primitive a package uses for unchecked interior mutability —
and no rule defined it. ADR-019 therefore made `Cell` compiler-known rather than
building it on `UnsafeCell`, "because 1 is not available". That was right for
`std` and left the language somewhere the owner named as unacceptable: `std`
would have interior mutability the compiler knows about, and no third-party
package could reproduce it.

**The decision.** `UnsafeCell[T]` is retained as the lowest-level
interior-mutability primitive, in `std.mem`, with narrow semantics: mutation
through shared access **only from `unsafe` code**; no safe reference handed
out; no runtime check; no synchronisation; `[BRW-1]`, lifetime, region, type and
bounds checking all still in force; no further safety tier; never `Copy`,
always `!Sync`, `Send` when `T: Send`. It completes a hierarchy — `Cell`,
`RefCell`, `Mutex`/`RwLock`, `UnsafeCell` — rather than opening a hole in one.

**What ADR-019 warned about, now discharged.** ADR-019 closed with a specific
prediction: that someone would reach ERR-043, observe that the compiler already
mutates through a shared borrow for `Cell`, and conclude that the language
therefore permits it — *"turning an implementation mechanism into the
justification for a language rule ... the same compiler-justifies-specification
move this project spent a session unlearning, arriving by a longer road."*

That did not happen, and the record should show why: the question went to the
owner as a question. `Builtin::CellSet` was never offered as evidence, and it is
not evidence now. The language permits `UnsafeCell` because the owner said so,
in a ruling that spells out what it does and does not suspend — not because the
compiler was already doing something adjacent.

**What was decided here rather than by the owner.** The ruling fixed the
semantics and not the spelling. The API surface and module were put to the owner
as a question, because both change the accepted program set; `std.mem` with a
raw-pointer accessor was chosen. `get(self) -> *mut T` needs `unsafe` under
`[UNS-1]` because it is a raw pointer, not because of a rule invented for
`UnsafeCell` — which is the property that keeps it inside the existing machinery
instead of beside it.

**What is not built.** All of it. `[UNS-10]`, `[UNS-10a]` and `[UNS-10b]` are
0.8.5 specification with no implementation, baselined under `[TST-4c]`'s one
permitted reason. `E3105` is registered ahead of its emitter so `[DIA-6a]`
holds. Building it is not part of `RefCell`, and `RefCell` must not be built on
it: ADR-019's route stands until interface generics can express these in Ember.

## ADR-023 — Each issued specification-hardening pass advances `Hardened_N`

**Owner ruling, 2026-09-12.** The owner supplied
`0.9.5_Hardened_3` and directed that a correction to that artifact must not be
left under the same revision identity: the audited correction pass is
`0.9.5_Hardened_4`, with H3 as its immediate predecessor. A later independently
issued hardening pass advances to H5, and so on.

This is a provenance and release-discipline decision, not permission to place a
semantic change in a hardening. The existing split remains: a hardening may add
clarification, source recovery, implementation invariants, conformance detail,
or editorial repair without changing accepted language semantics; a semantic
change still requires an owner-approved language revision.

An individual hardening artifact may reconcile several findings discovered in
one bounded audit. The rule is that an edited artifact is never presented under
the predecessor's frozen identity. Received files remain byte-for-byte under
`docs/spec-source/as-received/`; working candidates and eventual frozen cuts
carry their own incremented revision number.

Once that bounded pass is closed, the new hardening becomes the development
target. Its contents are frozen under that identity: a later-discovered flaw is
not patched silently into the same file, but repaired in the next hardening,
which then becomes the target. Normative repository adoption is a separate gate
and may lag the target while a recorded source or conformance blocker remains.

## ADR-024 — Recover `[LT-8]`–`[LT-13]` in H5 without regressing 0.9.5

**Owner-supplied source, 2026-09-12.** H3 claimed that the Hardened 14
`std.borrow.with_views2/3/4` rules had been recovered but did not contain their
definitions. H4 corrected that unsupported claim, opened ODR-004, and froze.
The owner then supplied the definitions, so ADR-023 requires the recovered
artifact to be `0.9.5_Hardened_5`, with unchanged H4 as its predecessor.

**What was recovered.** `[LT-8]`–`[LT-13]` establish canonical allocation-free
two-, three-, and four-view helpers over structurally stated `Span` signatures;
late-bound independent callback regions; no borrowed-region escape; ordinary
alias/mutability checking; no hidden allocation; and the then-current boundary
for persistent multi-owner aggregates. The supplied text's HTML entities,
escaped punctuation, and malformed fences were transport damage and were
normalized without changing the rule substance.

**Supersession boundary.** Hardened 14 predates the owner-approved 0.9.5
multi-region-view revision. Its statement that persistent independently-lived
aggregates require workaround shapes cannot revoke the later `[LT-2]` rule.
H5 therefore preserves the helpers and every safety constraint while recording
that 0.9.5 supersedes only the former single-region aggregate limitation.
`[LT-10]`, `[TYP-15]`, `[TYP-15a]`, and ordinary borrowing remain fully in
force. This is lineage reconciliation, not a new semantic decision.

**What was not decided.** `[LT-8]` spells shared-`Span` signatures, whereas
`[LT-11]` and `[TST-16]` refer to mutable inputs. The text does not select an
exact `MutSpan` overload family. Because that choice changes the public API and
accepted programs, it is ODR-005 rather than an inferred addendum to this ADR.

## ADR-025 — `with_views` has separate shared and all-mutable helper families

**Owner ruling, 2026-09-12, resolving ODR-005 / ERR-046.** The boundary exposed
by ADR-024 is real: shared-only signatures cannot make `[LT-11]`'s mutable-input
contract executable. The owner selected explicit mutable helpers instead of a
single magically generic `Span`/`MutSpan` family.

The canonical structural API has `with_views2/3/4` for all-shared `Span`
arguments and `with_views2_mut/3_mut/4_mut` for all-mutable `MutSpan` arguments.
Each callback parameter preserves the corresponding input's mutability. Mixed
shared/mutable overloads were not selected and are not implied. Exact public
spelling may follow the module overload convention only while arity,
mutability, and type relationships remain intact.

The mutable family is not a new ownership mechanism. Ordinary exclusive-borrow
rules decide whether all inputs can coexist; potentially aliasing mutable
sources are rejected, while established disjointness remains usable. Late-bound
regions, no escape, and `@noalloc` apply identically to both families.

**Terminology correction.** The ruling's patch spelled the mutable type
`SpanMut[T]`. Ember's existing and pervasive type is `MutSpan[T]`; H6 uses that
canonical spelling. This reflects the decision's stated mutable-span intent and
does not add an alias or a second type.

**Version treatment.** H5 was already frozen, so ADR-023 requires a new H6
artifact. This owner-approved API clarification completes an ambiguity in the
not-yet-adopted 0.9.5 target; it does not change the currently adopted 0.8.5
program set and does not claim implementation evidence. It closes which helper
families and view types exist; ODR-006 separately asks how the helper and
callback parameter modes are represented under `[FN-1]` and `fn_type`.

## ADR-026 — Callable types carry ordinary parameter modes

**Owner ruling, 2026-09-12, partially resolving ODR-006 / ERR-047.** H6 could
name the all-mutable helper family but could not type its callbacks faithfully:
the `fn_type` grammar admitted only a list of types, while `[FN-1]` and `[FN-2]`
make parameter mode part of the callee's borrowing and ownership contract.
Treating `MutSpan[T]` as implicitly mutable would have created a type-specific
exception to those ordinary rules.

**The decision.** Callable types admit the same three modes as declarations:
an omitted mode is borrowed, `mut` is an inout/mutable borrow, and `owned`
consumes the argument. Thus `fn(A) -> R`, `fn(mut A) -> R`, and
`fn(owned A) -> R` are distinct callable contracts, but not a second ownership
model. Their call sites obey the same mutability, place, move, and lifetime
rules as ordinary functions. An `extern "C" fn` remains available only where
the complete mode-bearing signature satisfies the existing FFI-safety rules.

The `_mut` `with_views` callbacks therefore spell every callback parameter
`mut MutSpan[T]`. `[LT-11a]` requires a mode-mismatch diagnostic instead of
pretending the `MutSpan` type supplies authority, and `[TST-20]` records the
corresponding conformance obligations. The ruling's `SpanMut[T]` spelling is
again normalized to Ember's existing `MutSpan[T]`; no alias is introduced.
The supplied mnemonic `[TST-LT-MUT]` is likewise normalized to the next unused
numeric test-rule ID because Ember's normative rule-ID grammar requires a
numeric suffix; this changes no test obligation.

**What remains undecided.** The ruling does not assign a mode to the helper
functions' own `MutSpan` inputs. They are still unmarked and therefore shared;
ODR-006 remains partially open over `mut` versus `owned`. It also does not say
how the mode vector survives `[CLO-3]`'s mapping from `fn(A) -> R` to
`Callable[(A), R]`, whose ordinary `Args` tuple carries only types. ODR-007
records that distinct public/compiler representation choice. Neither boundary
may be inferred from this ADR.

**Version treatment.** H6 was already frozen, so ADR-023 requires this ruling
to be issued as `0.9.5_Hardened_7`. H7 remains a development target rather than
the adopted repository specification, and it claims no implementation or
conformance evidence.

## ADR-027 — `with_views` reborrows inputs and `Callable` preserves modes internally

**Owner ruling, 2026-09-12, closing ODR-006 and ODR-007.** H7 made callback
parameter modes expressible but deliberately left two decisions open: whether
the mutable helpers consume or reborrow their input views, and how the existing
`Callable[Args, R]` abstraction distinguishes those modes.

**Helper input decision.** Shared helpers take their `Span[T]` inputs with the
default borrowed mode. Mutable helpers take every `MutSpan[T]` input as `mut`.
Neither family takes an input `owned`. The mutable family therefore performs
invocation-scoped reborrowing and does not consume the caller's view. The
helper never owns the underlying storage, extends its lifetime, or converts a
borrowed view into an owned value.

**Callable bridge decision.** The public `Callable[Args, R]` abstraction stays
in place. Its compiler-known canonical type identity additionally carries the
complete borrowed/`mut`/`owned` parameter-mode vector. That metadata survives
generic bounds, type and borrow checking, overload resolution, and
monomorphisation. It is erased before runtime and introduces no mode dispatch,
ABI change beyond existing ABI rules, public mode-vector generic parameter, or
second ownership system. `[FN-6a]` and `[CLO-3]` make explicit that the `Args`
tuple notation does not erase the internal mode vector.

**Conformance.** `[LT-8a]` binds the helpers to that callable representation;
`[TST-21]` covers helper input modes, caller usability, generic forwarding, and
mode mismatch/erasure. The owner's transport text used `SpanMut[T]` and
`[TST-LT-MODE]`; H8 normalizes them to the already-defined `MutSpan[T]` and the
next unused numeric rule ID without changing substance.

**Version treatment.** H7 was frozen before this ruling arrived, so ADR-023
requires `0.9.5_Hardened_8`. H8 becomes the frozen development target. It is
not the adopted repository specification and makes no compiler implementation
or conformance claim.
