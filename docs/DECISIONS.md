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

**2026-09-28.** The 0.9.9 rewrite dropped every mention of LLVM with the other
implementation-plan rules; ODR-087 restores it in Hardened_42 as `[CG-LL-1]`–`[CG-LL-3]`,
with this decision's order unchanged.

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
> coexist. At the time of this decision, `[CLO-6]`'s `owned f` was refused
> rather than mis-compiled: consumption is a property of the bound, not of the
> closure's type, and treating `CallableOnce` as `Callable` would permit the
> second call the rule forbids. That implementation residual closed on
> 2026-09-14: HIR/MIR now carry an explicit consuming indirect-callee move, and
> ordinary move/drop analysis turns a second call into E3040. The remaining
> body-level distinction closed on 2026-09-14 as D-121: a concrete `owned fn`
> becomes `CallableOnce` only when its checked body moves a non-`Copy` capture;
> supplying it to plain `Callable` is E3030, while a read-only move-capture
> closure stays reusable. The clarification that was missing is amendment A5 in `docs/spec-amendments.md`; no
> specification decision changed to close the implementation gap.

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

**Implementation status at decision time.** None of `[UNS-10]`, `[UNS-10a]`
or `[UNS-10b]` was built when this ADR was written. They were baselined under
`[TST-4c]`, with E3105 registered ahead of its emitter. Implementation commit
`a02c0a5` later completed the public type, narrow raw-pointer boundary,
consuming extraction, `@static_safe` rejection, and conformance coverage; the
baselines shrank accordingly. Threading-trait enforcement remains blocked on
`CELL-SYNC-1`. Building `UnsafeCell` did not rebuild `RefCell` on top of it:
ADR-019's route still stands.

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

## ADR-028 — `Arena` is a narrow `@borrows` provenance source

**Owner ruling, 2026-09-12, resolving ODR-008 / ERR-049.** Implementing
`[LT-4]` exposed a real public-signature boundary. An arena allocation carries
the arena borrow, but `[LT-1a]` previously allowed `@borrows` to name only a
view-typed parameter. A wrapper such as `fn allocate(arena: Arena) -> ref mut
T` therefore had no way to express its valid return provenance even though
the caller's arena owns the returned storage.

**The decision.** A function returning a view backed by a growing `Arena`
parameter may, and must, write `@borrows(arena)`. The named Arena contributes
its borrow region to the return. This exception is provenance-only: `Arena`
does not become a view type, its lifetime is not extended, ownership is not
transferred, and ordinary borrow/reset/drop conflicts remain enforced.

The exception is deliberately exact. It does not include arbitrary non-view
parameters, owned returns, or a view derived from another source. A wrapper
without the annotation is E3061; an annotation that lies about the returned
view remains E3062; an unrelated non-view parameter remains E2031. Nested
wrappers repeat the annotation in each public signature.

**Implementation boundary.** H9 records `[LT-4a]`, `[LT-4b]`, and `[TST-22]`.
The compiler retains the existing by-value ABI representation for ordinary
default-mode value parameters but creates a semantic shared loan at a direct
call whose returned view is tied to an Arena argument. This keeps reset, drop,
and caller-local escape checks load-bearing without making Arena a view.

**Terminology/API normalization.** The ruling's examples named otherwise
undefined `alloc_span` and `alloc_mut_span` helpers. H9 demonstrates the same
decision with the already-specified `alloc` and `alloc_array` APIs, and
normalizes `[TST-LT-ARENA-RETURN]` to the next numeric rule ID `[TST-22]`.

**Version treatment.** H8 was frozen, so ADR-023 requires
`0.9.5_Hardened_9`. H9 is the new frozen development target. The adopted
0.8.5 specification remains unchanged pending the separate 0.9.5 adoption
gates.

## ADR-029 — Arena bulk initialization has one explicit safety boundary

**Owner rulings, 2026-09-12, resolving ODR-009 / ERR-050.** H9 advertised
`alloc_array`, `alloc_uninit`, `Zeroable`, and `MaybeUninit`, but did not define
enough of their validity, drop, transition, failure, API, and phase behavior to
implement them without guessing. The owner selected the complete-contract path
and supplied the canonical public API in a follow-up ruling.

**Bulk allocation.** Ordinary `alloc_array[T]` always rejects
`needs_drop(T)`. It deterministically uses `Zeroable` first, then `Default`,
and reports existing E2040 when neither applies. `Default` construction is
transactional with respect to the Arena cursor; no partially initialized result
escapes. `alloc_uninit[T]` has neither capability bound and returns only
`MutSpan[MaybeUninit[T]]`.

**Validity and ownership.** `Zeroable` asserts exactly that the all-zero object
representation is valid initialized `T`; compiler proof is recursive, and a
manual implementation, if exposed, is unsafe and audited. `MaybeUninit[T]` has
`T`'s size/alignment, never drops `T`, and is `Copy` iff `T` is. Safe `write`
and `write_at` initialize storage; consuming `assume_init` is unsafe and asserts
one value or the complete span is initialized. No second safe conversion exists.

**Signature and ID normalization.** The ruling's prose requires moving the
written value and consuming `assume_init`, so H10 explicitly writes `owned`
and `mut` modes rather than letting Ember's borrowed default contradict the
prose. Its `[ARN-4]`–`[ARN-7]` headings collided with H9's frozen rules; H10
preserves those meanings and assigns the new clauses `[ARN-8]`–`[ARN-13]`.
The mnemonic conformance heading becomes numeric `[TST-23]`.

**Version treatment.** The ruling completes the intended behavior already
advertised by 0.9.5 and is owner-classified as a hardening. H9 was frozen, so
ADR-023 requires `0.9.5_Hardened_10`. H10 is the new frozen development target;
the adopted 0.8.5 specification remains unchanged pending adoption gates.

## ADR-030 — 0.9.6 consolidates compiler mechanisms without weakening semantics

**Owner rulings, 2026-09-13.** The owner supplied and iteratively corrected the
Simplicity Consolidation RFC, approved Revision 5 as the design basis for
`0.9.6_Hardened_1`, and explicitly approved the final clarification of H10
`[ARN-10]`. The received Revision 5 file is preserved byte-for-byte under
`docs/spec-source/as-received/` with SHA-256
`593E86A61738B32DCFAAB14875C12EA3A7BBF1A16BAD5286260EE1BAFD803923`.

**Architecture decision.** The reference compiler converges on reusable
`TypeIdentity`, `BorrowCapability`, `OwnershipGraph`, `AccessContract`,
`InitializationState`, `LayoutDescriptor`, and `EffectSet` facts. A canonical
borrow capability keeps provenance root, optional source place, storage
identity, projection, region, access permission, representation kind,
ownership, acquisition/checking, unsafe authority, synchronization, validity,
and escape constraints distinct. Raw-pointer representation is not an access
permission; unsafe is an authority boundary rather than a proof algorithm;
region equality or inequality is not an overlap or `noalias` proof.

This is shared machinery, not shared semantics. Arena remains a region
allocator. `Cell`, `RefCell`, synchronization guards, and `UnsafeCell` retain
different invariants. ECS remains library-owned; FFI remains contract-driven;
coroutines gain no escape exception; and H10's class-exclusivity runtime path
does not turn ordinary borrow failures into runtime acceptance. The three
replacement orderings — ordinary assignment drop-before-store after RHS
evaluation, `Cell.set` store-before-drop, and `MaybeUninit.write` store with no
old-value drop — remain deliberately different.

**Compiler boundary.** Initial MIR precedes initialization, ownership, borrow,
and region analyses; verified MIR is the code-generation boundary. Type and
interface solving constructs typed HIR from the resolved AST. Callable
field-access and field-to-region summaries use exact, trusted-declared, or
conservative-unknown facts. Trusted metadata comes only from verified MIR or
an audited unsafe/FFI contract. Dynamic dispatch includes every contractually
valid current, separately compiled, and hot-reload target. Both summaries
invalidate callers, interface hashes, generics, and caches together; stale
metadata is a hard compiler failure and none of it enters runtime ABI or reload
state.

**`[ARN-10]` resolution.** When `alloc_array[T]` uses `T: Default`, successful
construction remains specified. In v1 `T.default()` has no recoverable failure
path, and panic terminates the process through `abort()` under `[PAN-1]`; there
is no post-panic Arena state to observe and no unwinding requirement. No
rollback obligation arises unless a separate API explicitly specifies both a
recoverable construction-failure path and transactional rollback semantics.
The existing `[ARN-10]` is amended directly; no new suffixed rule is created.

**Version and evidence treatment.** The owner selected a fresh 0.9.6
architecture line, so the hardening counter resets to 1. Apart from accepting
the additive `0.9.6` selector, H1 preserves H10's accepted/rejected ordinary
source sets and observable semantics. H10 remains immutable. H1 is a frozen
development target, not the adopted repository specification, until its
implementation, conformance, tooling, and adoption gates pass. The adopted
source remains `docs/spec-source/ember-spec.md` (`0.8.5_Hardened_1`).

## ADR-031 — Arena-backed collections are fixed-capacity Arena views

**Owner rulings resolving ODR-011 / ERR-052.** H1 named `ArenaArray[T]` and
`ArenaMap[K, V]` as Arena-backed `@view` containers but did not define an
executable API. The owner selected fixed-capacity, single-allocation
containers and then explicitly closed the remaining public identity,
iterator-type, and initial-state questions.

**Storage and provenance.** Both constructors borrow a shared `Arena` through
`@borrows(arena)`, allocate their backing storage once, and return an empty
container with the Arena's region and `len() == 0`. No later operation grows,
reallocates, moves the backing buffer, mutates the Arena cursor, or creates a
hidden Arena region. Both containers and all named iterators are `@view`
types governed by `[TYP-15]` and ordinary borrowing.

**Failure and destruction.** Capacity exhaustion is recoverable:
`Err(CapacityError.Full)`. The unit-only public enum is owned by
`std.collections` and is not a prelude name. `ArenaArray[T]` requires
`!needs_drop(T)`; `ArenaMap[K, V]` requires `!needs_drop(K)` and
`!needs_drop(V)`. Arena rewind/drop never performs per-element destruction.

**API and iteration.** `std.collections` publicly owns `ArenaArray`,
`ArenaMap`, `CapacityError`, `ArenaArrayIter`, `ArenaArrayIterMut`, and
`ArenaMapIter`; none is in the prelude. The named iterator views implement
the existing associated-type `Iterator[Item = ...]` contract. Array iteration
is deterministic index order. Map iteration has no promised bucket order but
is deterministic for unchanged state and implementation configuration.
`ArenaMap` uses the ordinary `K: Eq + Hash` contract and duplicate insertion
returns the replaced value.

**Borrowing boundary.** Mutating methods take `mut self`; existing whole-place
borrow rules therefore reject an overlapping mutation while an element or
iterator borrow is live. No container-specific invalidation mechanism is
introduced.

**Version and evidence treatment.** The owner explicitly approved this exact
completion as `0.9.6_Hardened_2`. H1 remains frozen. H2 is the new frozen
development target but not the adopted repository specification until its
implementation, `[TST-24]` conformance, and adoption gates pass. This
owner-selected treatment closes an already advertised but unexecutable API; it
does not create a general implementation-agent exception to the project's
language-revision policy.

## ADR-032 — Hashing is an explicit protocol with coherent, read-only Map keys

**Owner ruling resolving ODR-012 / ERR-053, 2026-09-13.** H2 required
`K: Eq + Hash` for ordinary and Arena-backed Maps, but the inherited
`Hash.hash(self, mut h: Hasher)` declaration referred to a `Hasher` whose
identity, operations, lifetime boundary, and default implementation were never
defined. The implementation stopped rather than treating raw bytes, a no-op,
or another language's hasher as Ember semantics.

**Public contract.** `std.collections` publicly owns the `Hash` and `Hasher`
interfaces and the concrete `DefaultHasher`. Existing `[MOD-5]` keeps `Hash`
in the prelude; `Hasher` and `DefaultHasher` are not added. `Hasher` is a
stateful, move-only context with canonical byte and signed/unsigned integer
write operations. Finalization consumes it and yields `u64`; H3 spells this
`finish(owned self)` because existing receiver-mode rules make bare `self`
borrowed while the owner's normative prose explicitly requires consumption.

**Coherence and safety.** A `Hash` implementation feeds a deterministic
representation into the supplied context, does not retain or inspect the
context after return, and no hasher retains an input span past a write call.
For `T: Eq + Hash`, equality implies equal hashes; distinct values need not
hash distinctly. Safe Map APIs never expose `ref mut K`, preventing a resident
key's equality/hash identity from being invalidated in place.

**Implementation freedom.** `Map` and `Set` use `DefaultHasher` until an
explicit future custom-hasher API exists. The mixing algorithm is deliberately
not source-level compatibility behavior. Implementers may replace it without a
language revision while preserving all observable semantic requirements.

**Version and evidence treatment.** The owner classified this as a
standard-library API completion, so immutable H2 is succeeded by
`0.9.6_Hardened_3`. `[HASH-1]`–`[HASH-4]`, HC-096-03, closed ODR-012, and
ERR-053 preserve the authority chain. H3 remains a frozen development target,
not the adopted repository specification, until its implementation,
conformance, and adoption gates pass.

## ADR-033 — `Hash.hash` uses static generic Hasher dispatch

**Owner ruling resolving ODR-013 / ERR-054, 2026-09-13.** H3 defined the
public Hasher protocol but retained `Hash.hash(self, mut h: Hasher)`. Existing
`[TYP-22]` does not turn a concrete struct into a bare interface value:
interfaces require either an explicit generic bound or a legal `dyn`
representation. Because `DefaultHasher` is a concrete move-only context, the
H3 signature left dispatch, representation, and ABI unresolved.

**Decision.** The canonical signature is
`fn hash[H: Hasher](self, mut h: H)`. The concrete `H` is inferred at the call
and normally monomorphized through Ember's existing generic/interface
machinery. `DefaultHasher` implements `Hasher`. No mandatory dynamic dispatch,
bare-interface concrete parameter, or compiler-only coercion is introduced.
A future API may use `dyn Hasher` only by explicitly specifying a legal
runtime-polymorphic representation.

**Normalization.** The owner-supplied patch placed angle brackets around the
ordinary parameter list. Existing grammar uses parentheses directly, so H4
removes those transport markers while preserving the selected generic bound.

**Version and evidence treatment.** H3 remains immutable and is succeeded by
`0.9.6_Hardened_4`. `[HASH-1]`, HC-096-04, closed ODR-013, and ERR-054 preserve
the authority chain. H4 is a frozen development target rather than an
implementation claim or adopted repository source.

## ADR-034 — Post-H4 simplicity is architectural consolidation, not semantic collapse

**Owner approval, 2026-09-13.** The owner approved Revision 5 of the second
Ember 0.9.6 Simplicity Consolidation RFC. The received document is preserved
byte-for-byte as
`docs/spec-source/as-received/Ember_0.9.6_Simplicity_RFC_Revision_5.md`,
SHA-256
`0AB0F9C2F4F52E10492A0CD94897BBD9C8B47C08D86E7246EA2FA29F18007525`.

**Decision.** The RFC governs compiler architecture and development process
for the remainder of the 0.9.6 line. Reduce programmer-visible concepts and
duplicate mechanisms, reuse existing abstractions and authoritative compiler
facts, infer facts where sound, and treat unknown information conservatively.
Derived aggregates, caches, indexes, generated documentation, and architecture
labels never become independent semantic authorities.

Necessary distinctions remain intact. `AccessContract` and `EffectSet` stay
separate authoritative facts; initialization retains the H4
`Uninit | Maybe | Init` lattice and its refinements; provenance, storage
identity, disjointness, access permission, callable modes, destruction, unsafe
authority, synchronization, validity, and escape constraints must not be
collapsed merely because they can share implementation machinery. Existing
collection interfaces are reused only where their complete semantics match;
this RFC adds no `Len` or `Capacity` interface and no parallel safety system.

**Authority and version treatment.** This is an architecture/process decision,
not a source-language semantic amendment, implementation-status claim, or
adoption of the 0.9.6 specification as repository-normative. The adopted
normative source remains `docs/spec-source/ember-spec.md`.

**Subsequent owner clarification, 2026-09-13.** The owner clarified that approval
was intended to materialize the RFC's concrete architecture, tooling,
documentation, and development-contract changes as the next hardening. Therefore
`Ember_v0.9.6_Hardened_5.md` incorporates those requirements and becomes the
frozen development target; H4 remains its immutable predecessor. This later
ruling supersedes only the RFC's statements that adoption by itself would not
create H5. It does not authorize a source-language semantic change, alter the
accepted/rejected program sets, claim implementation/conformance, or install H5
as `ember-spec.md`. HC-096-05 and the H4-to-H5 classified diff preserve the
materialization evidence.

## ADR-035 — Span iteration, chunking, and pointer extraction use existing abstractions

**Owner ruling resolving ODR-014, 2026-09-13.** H5 advertised `iter`,
`iter_mut`, `chunks`, `chunks_mut`, and `as_ptr` across `Span[T]` and
`MutSpan[T]`, but did not provide enough public type, receiver, failure, or
unsafe-boundary information to implement those operations without choosing
accepted-program behavior. The implementation stopped at that boundary.

**Public iterator contract.** `std.collections` publicly exports
`SpanIter[T]`, `MutSpanIter[T]`, `SpanChunks[T]`, and `MutSpanChunks[T]`; none
is in the prelude. They are named `@view` types implementing the existing
associated-type `Iterator` interface. Their items are respectively `ref T`,
`ref mut T`, `Span[T]`, and `MutSpan[T]`. No opaque return type, compiler-only
ownership category, or second iterator model is introduced.

**Borrowing and failure.** A shared operation on `MutSpan` creates a shared
reborrow. `iter_mut` and `chunks_mut` create mutable reborrows and do not
consume the original view; ordinary NLL permits later reuse. Mutable chunks
are non-overlapping and retain distinct storage ranges under `[BRW-5]`.
`chunks(0)` and `chunks_mut(0)` panic through the existing failure mechanism
in every supported profile and may not become empty, fallible, zero-width, or
non-progressing iterators.

**Raw pointers.** `Span.as_ptr` and `MutSpan.as_ptr` safely extract `*T`;
`MutSpan.as_mut_ptr` takes a mutable reborrow and safely extracts `*mut T`.
Extraction neither accesses memory nor retains or extends the source lifetime.
Dereference, access, arithmetic, reference construction, aliasing, and validity
remain obligations of the existing unsafe contract. The owner text's
`*const T` is normalized to Ember's pre-existing `*T` spelling without a
semantic change.

**Version and evidence treatment.** H5 remains immutable and is succeeded by
`0.9.6_Hardened_6`. `[SPN-4]`–`[SPN-10]`, `[TST-25]`, HC-096-06, closed
ODR-014, and the preserved owner ruling form the authority chain. H6 is a
frozen development target, not an implementation claim or adopted repository
source.

**Implementation evidence.** Commit `d077563` implements this contract through
the existing Iterator, borrow, region, MIR-assert, and raw-pointer paths. The
four public types are declared in `std.collections`; compiler-known lowering
is only the bootstrap representation. Twenty new SPN-4..10/TST-25 Ember
sources cover identity, generics, NLL/provenance, parent conflicts, coexisting
disjoint mutable items/chunks, partial and zero-width chunks, pointer
authority/non-retention, and generated-C erasure. Removing the iterator result
provenance edge or the chunk cursor advance makes its adversarial test fail.
No new ownership category, opaque return, runtime region metadata, or unsafe
boundary was introduced.

## ADR-036 — Late-bound callbacks use one callable-type modifier

**Owner ruling resolving ODR-015, 2026-09-14.** The existing `[LT-7]` and
`with_views` rules required fresh invocation-local callback regions and escape
rejection, but H6 supplied no general declaration mechanism. The owner selected
the single callable-type modifier:

```ember
f: @latebound fn(Span[T]) -> R
```

**Decision.** `@latebound` belongs to the expected callable boundary, never to
a named function declaration. Each invocation receives fresh compiler-internal
regions for every borrowed or view parameter. A result, store, owned-closure
capture, FFI publication, or other publication path that retains one of those
regions is rejected by the existing region/escape machinery. An ordinary
callback remains ordinary; no global inference changes unrelated APIs.

**Representation.** The modifier participates in canonical callable identity,
generic substitution, expected-callable checking, interface compatibility,
EMIF, and incremental invalidation. It is not named lifetime syntax, runtime
metadata, an ABI or reload-schema field, a dynamic-dispatch requirement, or a
second ownership model. `std.borrow.with_views*` remains ordinary library code;
the compiler must not recognize those names specially.

**Version and evidence treatment.** This changes accepted/rejected callback
programs, so H6 remains immutable and `0.9.7_Hardened_1` is the owner-selected
language-revision successor. The target is not adopted. The current compiler
checkpoint implements and verifies the first slice: the minimal
`with_views2(..., first)` escape reproducer, mutable and lambda result escape,
nested scalar composition and view-escape rejection, EMIF
identity/invalidation, and generated-code erasure. A known capture-free callback
that returns a statically independent `str` now has direct positive evidence,
including `Box[str]` storage. Freshness, FFI-publication, and
separate-compilation cases remain required evidence before adoption;
owned-capture publication has a direct negative conformance case.

**Implementation status note — 2026-09-14 (non-normative).** The callable
modifier now survives parsing, canonical type identity, callable bounds,
substitution, expected-callable coercion, HIR/MIR calls, and EMIF schema 7.
Region analysis records compiler-only invocation-specific late-bound origins
and the existing E3062/E3063 checks reject return and unbounded-Box escapes.
This note records repository evidence only; it does not amend the owner ruling
or install H1 as the adopted specification.

## ADR-037 — `Shared[T]` uses the counted-owner and weak-owner model

**Owner ruling resolving ODR-017, 2026-09-20.** The previous target described
`Shared[T]` as a counted heap value with `s.get()` and a `Weak[T]` companion,
but did not define construction, access modes, weak construction, upgrade, or
whether that weak form was distinct from class `Weak[C]`. Implementing any one
of those choices would have created a public ownership API without authority.

**Decision.** `Shared(value: T) -> Shared[T]` is the strong-owner constructor.
`Shared[T].get() -> ref T` returns a shared borrow without retaining or moving
the owner. `Shared[T].get_mut(mut self) -> ref mut T` is a long-term mutable
access using existing static borrowing and dynamic exclusivity; `mut self` is
not a uniqueness proof. `Shared[T]` is Copy by strong retain and its final
release runs `T`'s ordinary destruction.

`Weak[O]` is the weak form of a counted owner, not a separate payload-pointer
family. In v1, `O` is either a class handle `C` or `Shared[T]`. Consequently
the existing `Weak[C]` remains valid and unchanged, while a shared payload uses
`Weak[Shared[T]]`. `Weak(owner)`, `Weak[O].empty()`, copying, dropping, and
`upgrade() -> Option[O]` reuse the object header's weak count and the existing
no-resurrection rule. `Weak[C]`, `Weak[Shared[T]]`, and C++ bridge owners never
interconvert.

**Alternatives rejected.** A separate `Weak[T]` payload family would overload
`Weak[Foo]` between a weak class and a weak `Shared[Foo]` interpretation.
Treating `get_mut` as requiring unique strong ownership would make a shared
owner secretly unique. A new `Shared`-specific aliasing mechanism would
duplicate the class access-state model. The selected surface instead reuses the
ordinary constructor, Copy/drop, borrow, Option, ARC, and exclusivity rules.

**Version and evidence treatment.** This supplies previously undefined public
source semantics, so `0.9.7_Hardened_3` remains immutable and
`0.9.8_Hardened_1` is the owner-selected language-revision successor. The
received ruling is preserved byte-for-byte at
`docs/spec-source/as-received/ODR-017_Shared_Weak_API_completion.md` with
SHA-256 `939028B039073BD1F086EE304327338C2DB7743F1721BB99C6AF6B953FC71D0E`.
The new target remains non-normative until its implementation, conformance, and
adoption gates are satisfied.

## ADR-038 — Cycle tools share one explicit analysis root

**Owner ruling resolving ODR-018, 2026-09-20.** `[WK-9]` named a class or field
for `ember explain --cycle` but supplied no program from which to construct the
ownership graph. Selecting cwd, a previous build, or one of several packages
would make observable CLI behavior without an owner rule.

**Decision.** The canonical command is `ember explain --cycle <path>
<Class[.field]>`. The same `<path>` resolver serves it and `ember inspect
--cycle <path>`: a package directory directly containing `ember.toml` loads
its manifest and ordinary module/import closure; one `.em` path is a standalone
file under the existing single-file synthesis. No cwd scan, stale-build
fallback, prior-root inference, or arbitrary package choice is allowed. A
module path has only the standalone interpretation, never a partial-package
mode.

Target names resolve only within that selected universe. `::` qualifies modules
and types; `.` selects a field. Existing name/member diagnostic families handle
unknown classes and fields. An unqualified class with multiple candidates is
an error that names the qualified candidates rather than choosing one.

**Boundary and evidence treatment.** This is `0.9.8_Hardened_2`, a CLI/tooling
hardening. It does not change ownership or lifetime semantics, source-language
acceptance, the existing ownership graph, ABI, runtime-object behavior, or
runtime metadata. The ruling is preserved byte-for-byte at
`docs/spec-source/as-received/ODR-018_Cycle_explanation_analysis_root.md` with
SHA-256 `BF735AFF393CB5EAEB08D73409AF8DF7F29E7D51773CA38D8E2950A26B5CCD7C`.
H2 is a frozen development target, not an implementation or conformance claim;
implementation and executable evidence remain separately required.

## ADR-039 — Float `//` and `%` are Python's

**Ruling on ODR-021 under the owner's delegation for 0.9.9, 2026-09-23.**
`[TYP-29]` defined float `//` and `%` by a formula that, evaluated in IEEE
arithmetic, can return a remainder of the wrong sign and lose
`a == (a // b) * b + a % b` far beyond rounding.

**Decision.** The result is Python's: the floor modulo computed exactly and
rounded once, and a quotient consistent with it. Zero divisors, infinities and
NaN follow IEEE through `fmod` and the division. `0.9.9_Hardened_3` states the
exact definition in `[TYP-29]`; the runtime implements it as
`ember_floordiv_f32/f64` and `ember_floorrem_f32/f64`.

## ADR-040 — `x = 0` is `int` at its declaration

**Ruling on ODR-022 under the owner's delegation for 0.9.9, 2026-09-23.**
`[LEX-16]`/`[LEX-17]` left open whether a later use can fix the type of a local
initialised by an untyped literal.

**Decision.** The declaration fixes it: `total = 0` is `int` and `x = 0.5` is
`float`, whatever later uses need. A later use cannot change an earlier
variable's width, and with it where its arithmetic overflows. `[]`, `Map()`
and `None` stay open because they have no default. A later mismatch is `E2020`
whose help names the declaration and the annotation that fixes it
(`total: i32 = 0`). `0.9.9_Hardened_3` states the rule in `[TYP-23]`.

## ADR-041 — A value function that can reach its end is `E2182`

**Ruling on ODR-023 under the owner's delegation for 0.9.9, 2026-09-23.**
`[FN-10]` gives only `void` and `Result[void, E]` an implicit value at the end
of a body, but named no diagnostic for any other function whose end is
reachable; the compiler accepted such a function and returned an
uninitialised value (D-186).

**Decision.** A dedicated code, `E2182` — "a function that returns a value can
reach the end of its body" — shown at the function's name, with the help to
return on every path or end with `panic(…)`. It is not a type mismatch the
programmer wrote. `0.9.9_Hardened_4` adds the sentence to `[FN-10]` and the row
to §XVII.9.

## ADR-042 — A borrowed parameter is the caller's place

**Ruling on ODR-024 under the owner's delegation for 0.9.9, 2026-09-24**
(a panel of four agents, synthesised by a judge). `[LT-1]` let a returned view
borrow only reference and view parameters, while `[FN-1]`/`[FN-2]` make the
default mode a borrow of the caller's value and milestone M2 returned
`ref xs[0]` from a borrowed `Array`. The compiler passed a borrowed parameter
as a shallow copy, which lost `Cell` writes and corrupted the heap through
`RefCell` (D-209, D-210).

**Decision.** A **source parameter** is a reference or view, or a borrowed or
`mut` parameter whose type is not `Copy`, each type parameter counting as
`Copy`. Rules 2 and 3 count source parameters; rule 1 takes any borrowed
receiver. A borrowed parameter is passed by address, except a view (passed as
itself, keeping its fields' regions) and a `Copy` value holding no `Cell`
(which may be copied) unless it is the receiver of a view-returning method.
`0.9.9_Hardened_5` carries the text; `docs/OWNER-QUEUE.md` has the options
rejected and why.

**Implementation decisions the text does not force.**
- The type checker computes the source set once, from the declared signature
  (for an instantiation, the generic one), and carries it on the MIR body as
  `sources`. The borrow checker never re-derives it from concrete types, so
  every instantiation, caller and `dyn` adapter agrees (D-213).
- A by-address parameter is a `ref T` local, so the existing `mut` and `ref`
  paths do the work. A shared `ref` is emitted as `T*` without `const`,
  because a `Cell` write through one is legal.
- The interface cache's compiler identity includes the executable's size and
  modification time, so a rebuilt compiler never loads records written under
  another ABI (D-215).

## ADR-043 — `[ERR-4]`'s function-taking methods are routed `std.core` generics

**Ruling on ODR-025 under the owner's delegation for 0.9.9, 2026-09-24.** The
methods are eager, move the payload in (`filter` borrows it) and take
`once fn`; a lambda's unannotated parameter takes `owned`, never `mut`, from
the expected callable type. `0.9.9_Hardened_6` carries the signatures.

**Implementation decisions the text does not force.**
- The methods are written in Ember, as private generic functions in
  `std/src/core.em` (`option_map`, `result_map_err`, …), rather than desugared
  in the type checker as `unwrap_or` is. A callback then goes through the one
  path every generic callable argument takes: its parameter types and mode come
  from the helper's bound, its result is inferred, closures are called directly,
  and the borrow checker sees an ordinary call. A desugar would have needed its
  own lambda typing and closure-call construction.
- `x.map(f)` binds the already-checked receiver to a local named `$receiver`,
  which no program can spell, and calls the helper with that local as its first
  argument; diagnostics name the method (`routed_method`), not the helper.
- The helpers take `f: fn(…)` until `once fn` parameter types are built
  (`docs/DEVIATIONS.md` D6).

## ADR-044 — Range values are prelude structs the compiler counts over

**Ruling on ODR-027 under the owner's delegation for 0.9.9, 2026-09-24.** A
range is a Python-style value: `Range[T]`, `RangeInclusive[T]`, `RangeFrom[T]`
and `RangeTo[T]` have public bounds and are `Copy` when `T` is; a `for` over one
counts over a copy of its bounds. `0.9.9_Hardened_8` carries it in `[CTL-3]`.

**Implementation decisions the text does not force.**
- The types are declared in `std/src/core.em`, and the compiler recognises them
  by origin (`range_parts`), as it does `SpanIter`; nothing about them is
  compiler-private except how a `for`, `in` and `len` lower. So they print,
  compare and copy through the ordinary implicit interfaces.
- There is one counted loop (`check_for_counted`). `a..` is a `while true`
  whose counter is advanced before the body, so a `continue` needs no increment
  of its own and counting to the maximum is the ordinary checked `+`.
- `len` reuses `RangeCount` (exact in 64 bits) and asserts the count fits an
  `int` before the conversion, because `as` keeps the low bits (`[TYP-6]`).
- D-232's fix keeps two spellings of a type in `TypeTable`: `display` for the
  user and `symbol_name` for everything that names C. The compiler-built
  generics (`Cell`, `RefCell`, `Ref`, `RefMut`, `MaybeUninit`) record their
  spelling in a table of their own rather than in `origin`, which also decides
  how a type resolves (an interface adapter, for one, is refused to a struct
  with an origin and no declaring module).

## ADR-045 — A field default is checked where it is used, as its declaration reads

**Decided 2026-09-24 with D-238's fix.** `[STR-2]` says a struct field default
is evaluated at each construction that omits the field, in field order. It does
not say how the expression's names resolve, or how often a mistake in it is
reported when many constructions evaluate it.

- The default is checked at each construction that omits the field, so it is
  real code in the constructing function (it may allocate, and each
  construction gets its own value). Its names resolve as the declaration reads:
  in the struct's module, with none of the constructing function's locals in
  scope (`check_field_default` swaps the module and the scope stack). A default
  that could see its caller's locals would change meaning with each caller.
- A mistake in a default is reported at its first use and checked quietly after
  (`reported_defaults`, the set `[FN-5]`'s parameter defaults already use), so
  one mistake is one error (`[DIA-14]`).
- Class field defaults still resolve at the construction site; aligning them is
  a follow-up, not part of D-238.

## ADR-046 — The runtime is compiled once, into a machine-wide cache

**Decided 2026-09-25, with the owner-approved test speedups.** Every build used
to compile the whole runtime (`ember_rt.c`, 5,438 lines, about 145 ms with
clang) together with the program. With clang and gcc it is now compiled once
per toolchain and profile into `<cache>/runtime/<key>.o` and linked as an
object (`ember_build::runtime_object`).

- **Where.** The cache is machine-wide: the variable `cache_dir_var()` names
  (`EMBER_CACHE`), else `%LOCALAPPDATA%\<cli>\cache`, `~/Library/Caches/<cli>`
  or `$XDG_CACHE_HOME/<cli>` (`~/.cache/<cli>`), else the temporary directory.
  Go's and Zig's build caches live in the same places. An object under
  `target/` would be rebuilt by every test case, because each case has its own
  output folder. `[BLD-5]` governs a build's output, which stays in
  `target/<profile>/`. The runtime object is toolchain state, like the runtime
  in `[TOOL-1]`'s archive. The B6 budget ("clean `debug` build, cold cache")
  already tells a clean build from a cold cache.
- **Key.** A BLAKE3 hash (the crate already depends on it) of the compile
  command (flags, include directories, source path), the source, each file in
  the include directories, and the compiler's file on disk: its path, size and
  modification time. An upgraded compiler, or another one first on `PATH`,
  gets a new object. The key only names the object; none of it reaches the
  artefact (`[BLD-13]`).
- **Side by side.** The object is compiled under `<key>.o.tmp<pid>` and renamed
  into place, as the interface cache has done since D-189. A rename lost to
  another build keeps that build's identical object.
- **MSVC** (added with D-251's fix, when MSVC first ran). `debug` and
  `release` reuse a runtime object compiled with `/Z7`, which keeps the debug
  information inside the object; `/Zi` would tie a shared object to a PDB
  beside it. `shipping` compiles the runtime with the program, because `/GL`
  optimises the two together at link time.
- **MSVC's environment** is cached too: `<cache>/msvc/<key>.env` holds what
  `vcvars64.bat` set, meaning the variables it changed, plus always `INCLUDE`,
  `LIB`, `LIBPATH` and `PATH`. Its key covers the batch file and the toolset
  version file beside it. A cached `INCLUDE` folder that no longer exists
  means running the batch file again. Running it had cost 1.2 s on every
  build: a hello build went from 1.6 s to 0.21 s.
- **Fallback.** When the cache directory cannot be created, the build compiles
  the source with the program, as before.
- **Not done.** Nothing prunes old objects (each runtime edit leaves up to
  three, one per profile), and only the include directories' own files are
  hashed, not their subdirectories (the runtime's headers are flat).

## ADR-047 — What each type feeds a `Hasher`, and `DefaultHasher`'s mixer

**Decided 2026-09-25, with D-265.** `[HASH-1]` asks for "a deterministic
representation" that equal values share; `[DRV-1]` fixes a derived type's
order ("each field in declaration order; enums feed the variant index first");
`[HASH-2]` caps `DefaultHasher`'s cost at FxHash's and leaves the mixer
replaceable. The rest was open, and is:

- **Scalars** feed the write of their width; `bool` a `u8`; `char` its scalar
  value as a `u32`; `i128`/`u128` the low `u64` then the high one.
- **Text** feeds its bytes, then `0xFF`, a byte no UTF-8 text contains, so a
  tuple `("ab", "c")` and `("a", "bc")` feed different streams. `String`
  feeds exactly what its `str` feeds (`[STD-12]` needs that).
- **Sequences** (`Array`, `Span`, and a fixed array as its `Span`) feed their
  length as a `usize`, then each element: `[[1], [2, 3]]` and `[[1, 2], [3]]`
  differ. An `Array[u8]` is a `String` inside the checker and feeds as one;
  equal byte arrays still hash equally.
- **`Option`** feeds a `u8` tag (0 for `None`, 1 for `Some`) then the payload;
  **`Result`** 0 for `Ok`, 1 for `Err`. A **unit-only enum** feeds its
  discriminant as a `u64`; a **derived enum** its variant index as a `u64`.
  **Tuples** and **derived structs** feed their fields in order; **`void`**
  feeds nothing; a **range type** feeds its representation.
- **Where it lives.** What std can extend is written in Ember in
  `std/src/collections.em`. Tuples, fixed arrays, unit-only enums, range types,
  `void` and derived types cannot be extended from std, so the checker builds
  their `hash` from the same parts (`synth_hash_of`), and `implements` walks
  the same kinds (`hashes`).
- **`DefaultHasher`** starts at 0 and mixes each word as
  `state = (rotl(state, 5) ^ word) * 0x517cc1b727220a95` (FxHash's step:
  one rotate, one xor, one multiply). Bytes go in eight to a word,
  little-endian, with a short last word. The 0.9.8 mixer (`rotl 7`, xor)
  made `hash(k)` of a lone `u64` equal to `k ^ 128`, so keys differing only
  above a power-of-two table's mask all collided. `finish` returns the state;
  a table takes its index from the high bits, where the multiply mixes best.

## ADR-048 — `i128` and `u128` in C: `__int128` where it exists, two halves elsewhere

**Decided 2026-09-26, with D-272.** `[TYP-1]` gives both types 16 bytes and
`[FFI-8]` maps C's `__int128` to `i128`; nothing says how the C backend
carries them where C has no 128-bit integer (MSVC).

- **Two forms, one set of operations.** Where the C compiler has `__int128`
  (GCC and Clang, except clang-cl) `ember_i128`/`ember_u128` are C's own
  types, as `[FFI-8]` needs. Elsewhere they are a struct of two `uint64_t`
  halves, two's complement, low half first (the bytes a little-endian
  `__int128` has), aligned to 16 as the type table says. clang-cl has
  `__int128` but its division calls compiler-rt functions that MSVC's linker
  does not bring in, so it takes the halves.
- **The generated C never applies a C operator to a 128-bit value**: every
  operation is a runtime helper (`ember_i128_add`, `ember_ck_mul_u128`,
  `ember_i128_to_f64`, …), a one-line `static inline` operator in the native
  form. So one generated program means the same under every compiler. The
  alternative, C operators with the halves only where C lacks the type, would
  have made the MSVC build the only check of the backend's routing, and CI's
  only MSVC jobs the only place a missed route showed.
- **Conversions** round to nearest, ties to even, from the bits (the halves
  never go through a 64-bit conversion, which would round twice); a float
  converts toward zero and saturates, NaN to 0 (`[TYP-6]`).
- **Division** in the halves is Hacker's Delight's: `divlu` (128/64 in 32-bit
  digits) and the doubleword quotient estimated from the divisor's top 64
  bits, corrected once.
- **Ranges.** `range(a, b, step)` over a 128-bit type counts in 128 bits; a
  count past `usize`'s maximum is that maximum (no loop runs so long), and
  `len` then panics as for any range too long for an `int`. A 128-bit bound
  of `get` or `drain` outside `int` is out of bounds, not its low 64 bits.
- **Tests.** `ember_build`'s `the_128_bit_halves_agree_with_int128` compares
  every helper in the halves with `__int128`, bit for bit, over edge values and
  two million random ones (GCC and Clang; MSVC has nothing to compare with).
  The driver's `the_128_bit_programs_run_with_the_halves` builds the 128-bit
  conformance programs with the halves forced (`EMBER_SOFT_INT128`), with
  `-Werror`, where a missed route would be a compile error. The C test source
  is a template in `runtime/ember_rt/templates/tests/`, rendered by
  `tools/generate_runtime.py`, since it calls runtime functions by name
  (`[RT-5]`).

## ADR-049 — the operator interfaces: how numbers meet them, and how an operator finds its method

**Decided 2026-09-26, with ODR-040 and D-313 to D-315.** ODR-040 says which
interfaces each number type implements and what the methods are called; it
does not say how the compiler supplies a number's methods or finds a type's
operator.

- **A number's methods are its built-in operators.** `std.core`'s
  `extend i32 implements Add, Sub, …: type Output = i32` blocks have no method
  bodies: the implementation check takes a number's operator in place of
  each method (`builtin_operator_method_fits`, as `[STD-27]`'s float methods
  are built into `f32` and `f64`), and `x.add(y)` is checked as `x + y`
  (`synth_number_operator_method`), panicking where the operator does. The
  alternative, a body per method in `std` (`fn add(self, rhs: i32) -> i32:
  return self + rhs`), is some 330 methods that say nothing the operators do
  not, each a call where an operator was. The memberships themselves are
  written in `std.core`, not supplied by the compiler.
- **Generic code is checked through its bounds, and each instance on its own
  types.** Inside `sum[T: Add[Output = T]]`, `total + x` is a call of the
  bound's `add` whose type the binding gives; the instance for `i32` is
  checked again on `i32`, where `+` is the built-in operator, so going through
  the interface costs a number nothing.
- **An operator finds its method through the implemented-interface
  registry** (D-315): the type must implement the prelude interface
  (`std.core.Add`, in any instance), and the method is then chosen among that
  interface's instances by the other operand's type (D-313). Asking the
  method which interface it came from would not do: a struct body's methods
  are registered as inherent even when its header says `implements Add`.
- **`a op= b`** calls `OpAssign`'s method where the type or a bound has it,
  and is otherwise `a = a op b` with a result of `a`'s type (`[TYP-21]`). In
  both, the operand is evaluated first and the place once (`[EXP-2]`), as the
  numeric form does.
- **`not` is a member name only after `fn` and `.`** (the parser's
  `expect_member_name`); everywhere else it is the keyword it was.
- **An `Output` a bound leaves unsaid.** `T: Add` makes `a + b` a
  `T.Output` that no binding resolves. The checker reports it where the
  operator is (`E2040`, naming `T.Output`, the binding to add and the
  signature that would name it), instead of the later mismatch "expected
  `T`, found `Self.Output`", which named neither.
- **A generic extension's associated types** are resolved over the
  extension's parameters when it is collected and substituted when it is
  applied to an instance (D-317), the way its methods are.

## ADR-050 — `f16` in C: its bits in a `uint16_t`, every operation in `double`

**Decided 2026-09-26, with D-316.** IV.2 makes `f16` IEEE binary16 and
`[TYP-9]` asks for IEEE arithmetic; nothing says how the C backend computes
with a type that standard C does not have.

- **Storage is the bits.** An `f16` is a `uint16_t` holding its binary16
  encoding, so it has its layout (`[RNG-8]`, `[TYP-11]`) on every compiler.
  C's `_Float16` is not on MSVC, and where it exists its arithmetic may be
  done in `float` or emulated differently per compiler.
- **Every operation widens to `double` and rounds back once**
  (`ember_f16_to_f64`, exact; `ember_f64_to_f16`, ties to even, from the
  bits). For `+ - * /` this is binary16's correctly rounded result, since
  binary64 has more than twice binary16's precision plus two bits (rounding
  twice cannot then differ from rounding once); `//` and `%` of two `f16`s
  are exact in `double`, so they too round once. `**` is `pow` in `double`,
  rounded once. A comparison compares the widened values, so `-0.0 == 0.0`
  and NaN equals nothing; sorting uses totalOrder on the bits
  (`ember_total_lt_f16`). `-x` and `abs` touch only the sign bit.
- **Conversions** go through `double`: exact from `f16`; to `f16` one
  rounding, since an integer too large for a `double`'s precision is far past
  `f16`'s largest value (it is infinity whatever the first rounding did). An
  `f16` to an integer saturates, NaN to 0 (`[TYP-6]`).
- **Constants** are encoded by the compiler (`ember_types::f16_bits`, the same
  algorithm as the runtime's, checked against every `f16` and its midpoints).
- **Text.** Printing is the fewest digits (at most five) that read back as the
  same `f16`. `parse[f16]()` rounds the text to `double` and then to `f16`;
  that can differ from rounding once only when the `double` lands exactly on a
  midpoint between two `f16`s, and then the text is compared, digit by digit,
  with the midpoint's exact expansion to settle it
  (`compare_decimal_magnitude`). The runtime's routines were checked outside
  the suite against a Python model on 590,474 values, every `f16` through
  print and parse, and the midpoint cases.

## ADR-051 — blanket implementations: generic methods, and implementations registered on demand

**Decided 2026-09-26, with ODR-042 and D-321.** ODR-042 says what an `extend`
parameter that only the implemented interfaces name means; this is how the
checker carries it.

- **The methods are generic over the blanket's parameters.** Before any
  collection, `desugar_blanket_extensions` moves such a parameter from the
  `extend` onto each method of the block, as its first generic parameters,
  and keeps it in the new `ast::ExtendDecl::blanket` (the parser leaves that
  empty). `Map`'s `index` then has the shape it had as an inherent method,
  `fn index[Q: AsKey[K] + Hash + Debug](self, q: Q) -> ref V`, and a call
  infers `Q` as any generic call does. The rewrite works on a copy of the
  modules and changes no item's or member's position.
- **The implementation is a record, registered when first needed.** A type
  implements a blanket's interface for infinitely many arguments, so nothing
  is registered per instance. `BlanketImpl` keeps the parameters, the target
  and the interfaces as patterns; when a bound check needs `Map[String, int]:
  Index[str]` (`implements_or_blanket`), `apply_blanket` matches the target
  and the interface's arguments, checks every bound with the arguments
  substituted, and records the implementation and its associated types as a
  written one's, checking the methods then (`check_blanket_instance`, which
  reports at the `extend`). `implements` itself stays a read-only query.
- **Nominal indexing** asks whether a type implements some instance of
  `Index` (`implements_origin`), which a blanket record answers by its target
  alone.
- **A bound brings its parents** when the parameter is declared: each bound's
  supertraits are added, and a binding is copied to the parent that declares
  its associated type.
- **Not done:** `[TYP-19]`'s overlap check (`E2041`) does not yet compare a
  blanket implementation with a written one, and a blanket method with
  generics of its own besides the blanket's is not compared with the
  interface.

## ADR-052 — constants are evaluated while compiling, by folding the checked initialiser

**Decided 2026-09-26, with ODR-045 and D-323.** V.7 and `[CT-1]` say a `const`
is evaluated at compile time and inlined at each use; the compiler has no
compile-time evaluator yet (Part XIV is Phase 4). This is how a constant
expression is carried until it has one.

- **The initialiser is checked once and folded.** `expr_const` checks the
  value against the declared type in the declaring module (as a field default
  is), then `const_eval::fold` reduces the checked HIR to a `Folded` value:
  integers exact in their own width, floats rounded to their type after each
  operation (an `f16` from `double`, as the runtime does), float `//` and `%`
  by the runtime's own floor forms, shifts and floor division as the runtime
  checks them. Each use is `Folded::to_expr`, the value's literals, so a use
  costs nothing and cannot panic. An overflow, a division by zero or a shift
  out of range is `E6004` at the declaration, with the message the program
  would have panicked with.
- **What this phase evaluates** is what `is_const_expr` admits: literals,
  other constants, arithmetic, comparisons, `and`/`or`/`not`, tuples, arrays,
  and struct and enum constructions. A call is `E1010`: evaluating one needs
  the interpreter `[CT-1]` describes, and running it at each use instead
  would turn its panic into a run-time one and its transcendental functions
  into the platform's (`[CT-4]` wants `std.math.det`'s).
- **Lazily, in any order.** A constant is recorded when collected
  (`PendingConst`) and evaluated at its first use or when collection ends
  (`settle_consts`), in its declaring module, with no locals, no type
  parameters and `Self` its type. A use while it is being evaluated is a
  cycle, `E6001`.
- **Untyped literals stay untyped** (ODR-037): such a constant has no folded
  value, and each use checks the literal again where it is used, with that
  check's diagnostics dropped.
- **Warnings once.** A literal's `W2015` is raised by the declaration's check;
  `warn_if_literal_loses_precision` does not repeat a warning the sink already
  holds for the same literal (D-326).

## ADR-053 — a number on the left of a program's type, and sibling instances of one interface

**Decided 2026-09-26, with ODR-043.** `[TYP-21]` makes `2.0 * v` the number
type's `Mul[Vec3]`, which `std.math` implements (`extend f32 implements
Mul[Vec3]`). Two things in the checker had to change for it.

- **Dispatch.** In `synth_binary`, when the left operand is a number and the
  right one is not, `number_with_operator_for` finds the number type whose
  operator method takes the right operand's type: the left operand's own type,
  or for an untyped literal the first of `i64`, `i32`, …, `f64`, `f32`, `f16`
  that has one, which the literal adopts (`2 * m` is `i64`'s where `i64:
  Mul[Money]`). Between two numbers the interface dispatch is skipped and the
  built-in operator stays: `f32: Mul[Vec3]` must not make `k * 2.0` a call.
- **Sibling instances.** A number type implements `Mul` (its own, `Rhs =
  Self`) and `Mul[Vec3]`, `Mul[Mat3]`, … as separate instances of one generic
  interface. `check_implementation_of` used to accept any method named `mul`
  as the implementation of each, so `f32`'s own `Mul` was checked against
  `Mul[Vec3]`'s method. A method recorded for another instance of the same
  generic interface is no longer a candidate. One with no interface, one of
  the same instance, one recorded under the bare origin (as a generic
  extension records it) and one of an unrelated interface still are, as
  before.

## ADR-054 — `std.math.det`: fdlibm in Ember, with arithmetic where fdlibm uses bits

**Decided 2026-09-26, with ODR-046.** `[DET-4]` wants the same bits on every
target; any fixed sequence of IEEE operations gives that, now that no C
compiler may fuse a multiply and an add (D-325). This is the sequence.

- **The algorithms are fdlibm's**, as FreeBSD's `msun` has them: `exp`,
  `log`, the `sin`/`cos`/`tan` kernels, the medium range reduction, `atan`,
  `atan2` and `pow`, with their constants. They are written in Ember in
  `std/src/math/det.em` (so `std.math` became `std/src/math/mod.em`,
  `[MOD-1]`), not in the C runtime: the language's own arithmetic rules make
  them deterministic, and a compile-time evaluator can reuse them (`[CT-4]`).
- **No bit access.** Where fdlibm reads or writes a float's words, the code
  scales by exact powers of two (`two_to`, `scalbn`), finds an exponent by
  comparing with powers of two (`split_exponent`), and replaces "clear the
  low word" with Veltkamp's split (`high_part`, 26 significant bits), which
  keeps every product fdlibm needs exact (26 + 26 bits fit in 53).
- **Huge arguments** (|x| ≥ 2^20 π/2) are reduced by Payne and Hanek's
  method with integers: the 53-bit mantissa times a 192-bit window of 2/π's
  bits (a table of 23 words, generated from 2/π to 1600 bits), modulo 2^192,
  gives the quadrant and 190 fraction bits; the fraction becomes a
  double-double and is multiplied by π/2 with Dekker's product.
- **`f32`** calls the `f64` function and rounds once; `sqrt` is the IEEE
  square root of either type. The functions are generic over a private
  interface, `Deterministic`, that `f32` and `f64` implement.
- **How it was checked.** A Python model of the same operations (Python
  floats are IEEE doubles) was compared with mpmath at 200 to 1400 bits over
  20000 random arguments per range: below one unit in the last place
  everywhere, `atan2` below 1.26; the hardest argument for reduction,
  6381956970095103 × 2^797, is exact. The Ember module was then compared with
  the model bit for bit over 1455 arguments and eight functions, in every
  profile and under clang and gcc, and over 600 `f32` arguments. The model is
  `tools/det_model.py`, and `tools/check_det.py` repeats the comparison in CI.

## ADR-055 — The compiler runs on a thread with a 256 MB stack

**Decided 2026-09-26, with D-330.** The specification says nothing about the
compiler's own stack, and the host decided it: 1 MB for a main thread on
Windows, 8 MB on Linux. A program that compiled on Linux could overflow on
Windows.

- **One stack everywhere.** `main` starts the compiler on a thread with a
  256 MB stack and waits for it. How deep an expression can nest is then the
  same on every host, for a given build of the compiler.
- **Why 256 MB.** A debug build of the compiler spends about 65 KB of stack
  on each level of a binary operator, and a parenthesised level costs more:
  a polynomial in Horner form a hundred levels deep needs 16 to 32 MB. 256 MB
  holds about four thousand levels of a sum in a debug build. The size is a
  reservation of address space on Linux, macOS and Windows; pages are
  committed as the recursion reaches them, so a shallow program costs what
  it did.
- **Panics.** An internal error panics on the compiler's thread, which is
  named `main` so that the message reads as it did; `main` resumes the panic,
  and the process ends with the same status.
- **Not done.** A limit on nesting with a diagnostic (D-331) needs the
  specification to state one. Growing the stack on demand needs a dependency
  or platform code at each recursion point.

## ADR-056 — `Option[NonZero[T]]` is a `T`, and a `NonZero` divisor has no zero check

**Decided 2026-09-26, with ODR-047.** `[STD-4]` and `[TYP-13]` fix what a
program sees: an `Option[NonZero[T]]` the size of `T`, and no check for zero
when dividing by a `NonZero`. This is how.

- **Which `Option`s.** `TypeTable::option_niche` answers for an `Option`
  instance (a `None` without fields and a `Some` with one) whose payload is
  `std.core.NonZero[T]`, known by its declaration's name. `[TYP-13]`'s other
  niches (handles, `Box`, `ref`, `bool`, `char`, ranges, enums with unused
  discriminants) are not done; `option_niche` is where each joins.
- **Layout.** The `Option`'s C type is a `typedef` of the payload's struct.
  `Some(v)` is `v`, and `None` is the payload with its field 0. The
  discriminant is `(x.value == 0) ? None : Some`, compared through `eq_expr`,
  so `i128` and `u128` use the runtime's helpers where MSVC has no 128-bit
  integer. Every place codegen reads a tag or a variant's field goes through
  `enum_tag` and `enum_member`: construction, the discriminant, a field after
  a downcast, printing, equality, drop glue, retains, debug edges, and
  `Array.pop`'s helper, whose `None` is zero bytes.
- **The division.** `std.core`'s implementations divide by the field,
  `self // d.value`. MIR lowering leaves out the zero check for a divisor
  that is a `NonZero`'s field (`nonzero_divisor`), and for an integer
  constant other than 0, which `[EFF-15]`'s table also names (the C compiler
  folded that check away before). The overflow check stays: `MIN // -1`
  panics (`[TYP-8]`).
- **Not done.** `NonZero` has no `Ord` until `@derive(Ord)` exists, and prints
  as a struct, `NonZero(value=5)`, until `std` declares `Display`.

## ADR-057 — a method implementing an interface is as visible as the interface

**Decided 2026-09-26, fixing D-328.** `[MOD-2]` makes an item private to its
module unless marked, and `[BRW-10]` speaks of a method's visibility, so a
method written without `pub` is private to its module. The spec does not say
how visible a method is that implements an interface; `[TYP-24]` finds an
interface method "through any implementation visible in the program".

- **The ruling.** A method that implements an interface, in an `extend T
  implements I` block or in a type's `implements` header, is as visible as the
  interface, whatever it writes: the interface's methods are what it offers,
  and an implementation cannot offer less (a `pub` interface's method written
  as plain `fn` in the block is callable anywhere), nor more (a private
  interface's methods, `std.math.det`'s `det_sin`, stay in its module). Any
  other method or associated function has the visibility written with it.
- **How.** `method_vis` maps a method's definition to its visibility and
  module (`member_visibility`), filled where methods are registered and, for
  a header's interfaces, by `adopt_interface_visibility`; a generic
  instance's method falls back to the interfaces its owner implements.
  `check_method_visible` reports `E1052` naming the declaring module.
- **The compiler's own calls.** Where the compiler routes a call through a
  `std` helper (`a.sort()` to `sort_ord`), the helper is the compiler's
  implementation, not the program's call, and is not checked (`routed_call`).
- **Alternative rejected.** Requiring `pub fn` on every implementing method
  would make `pub` on it mean nothing (the interface decides who can see the
  method through `I.m(x)` anyway), and would let a public type's
  implementation of a public interface hide the interface's methods.

## ADR-058 — how the simplification pass is built: associated defaults, opaque default bodies, `alloc_zeroed`

**Decided 2026-09-26, with ODR-049 and ODR-064 (the owner's simplification pass).**
The rulings fix what a program sees; these are the implementation's choices.

- **Associated-type defaults.** An interface's `type Out = Self` is resolved where the
  interface is collected, with `Self` the placeholder parameter and the other associated
  names abstract (`InterfaceDef.assoc_defaults`); a generic interface's instance
  substitutes its arguments into it. An implementation takes it in `check_implementation_of`
  (`fill_assoc_defaults`), which runs after every module is collected and before any body is
  checked, so a header's and an `extend`'s implementations are served alike: `Self` becomes
  the implementing type, and a default naming another associated type is resolved against
  the implementation. Cycles are found on the written types (`type_expr_names`), before
  anything is filled, and the cyclic defaults are dropped after `E2043`.
- **Default bodies (D-344).** `check_default_bodies_opaquely` checks each default body once
  with `Self` a parameter placed after the interface's own and the method's own parameters,
  bounded by the interface (or its instance over opaque arguments), its parents closed; the
  declaration's signature gains the receiver `method_signature` leaves out of an interface.
  Each implementation's copy is still checked, since it is what is emitted, but into a quiet
  sink, reporting only what the opaque check could not see, as D-280 does for generic
  methods.
- **`alloc_array` and `alloc_zeroed`.** `default_initialisation` answers `Some(None)` for a
  scalar, whose standard `Default` is zero bytes (checked against `std/src/core.em`), so
  `alloc_array[f32](n)` stays one `ember_arena_alloc_zeroed` rather than a loop of calls;
  every other type calls its `default` per element. A type parameter is answered from its
  bounds and checked again in each instance.
- **`@derive(Zeroable)`.** The type table keeps the declared names of the structs that derive
  it (`derive_zeroable`) rather than a flag on every `StructDef` (thirty construction sites);
  `is_builtin_zeroable` proves a struct field by field only when its declared name, or its
  generic origin's, is there. `Zeroable` joins `Copy`, `Display` and `Debug` among the
  undeclared markers a bound is answered for from the compiler's table.

## ADR-059 — how `[TYP-15]`'s storage rule is checked: stores followed, summaries, weak updates

**Decided 2026-09-26, with ODR-069 (SP-013) and D-352, D-353, D-198.** The ruling fixes which
programs are accepted; these are the implementation's choices.

- **One place for the rule.** `Regions` (`regions.rs`) finds every store and where it lands
  (`store_target`): a local's own slots, the target of the first reference on the path, or
  storage no local bounds (`Unbounded`: an `Array`'s element, a class object's field, a span's
  element, a `Box`'s or `Shared`'s contents, a reference it cannot follow). `stores.rs` judges
  what the stored value carries. The earlier `check_array_storage_regions` and
  `check_box_storage_regions` are subsumed and removed; their messages are kept.
- **Stores through a reference are weak updates.** A store through a reference joins the
  reference's own slot and every place its loans borrow (`loan_places`, matched by type), or a
  reference parameter's slot, which stands for the caller's place. Joining rather than
  replacing is what keeps a store through `r = ref mut name` alive in `name`, and keeps the
  reference's own loan. A store to part of a slot (a fixed array's element) joins the slot.
- **A view copied out of an `Array`'s element or a class field is `static`** (`heap_reads`):
  only `static` views go in, and every store is checked. A span's element is not, since a span
  may view a local fixed array.
- **Containers that cannot be viewed into.** A value's regions are its roots (loans and
  parameters' entry regions) and origins. A loan of, or origin in, a container that owns no
  storage the value could point into (a `str` from an `Array[str]` or a `Map[str, V]`, not from
  a `String`) adds nothing, since what the value holds from it is carried separately or is
  `static` (`copies_only`). This is what lets `[n for n in names]` and a map's keys be stored.
- **Summaries, to a fixpoint.** A function's `StoreSummary` lists the parameter slots it stores
  where only `static` goes, and the slots it stores into a `mut` (or reference) parameter's
  place; `infer_store_summaries` computes them with a worklist (a body again when a callee's
  grows), and each call applies them: a requirement is checked against the argument (the
  pointee of a reference argument, through its loans), a flow is a store through the argument.
  `mem.replace` and `mem.swap` have fixed flows. The summaries reach the borrow check through
  `CallRegionContract.stores`; the result-provenance summaries are inferred without them.
- **Bodies that are called dynamically publish nothing** (`dynamic_bodies`: virtual methods,
  closures, functions used as values, `dyn` adapters' methods, foreign-ABI functions) and are
  held to the rule themselves.
- **Instances' results (`declared_borrows`).** A recipe method's declared result type is kept
  (`declared_rets`); an instance whose rule-1 receiver ties its result gets the parameters whose
  declared types share a type parameter with the result, where the instance makes them views,
  as a synthesized `@borrows`, which its callers read like a written one.
- **Cost.** Summaries re-run region inference for a body when a callee's summary grows; a
  map-heavy test checks about 15% slower in a debug build (3.05 s to 3.50 s), others unchanged.

## ADR-060 — how `[EXC-19]`'s per-field access is built

**Decided 2026-09-26, with D-202, D-218, D-359 and ODR-072.** The rules fix what is checked;
these are the implementation's choices.

- **The layout.** A class object holds a `uint32_t` access word in front of each non-`Copy` field
  (`class_field_has_access_word`; `_access_<field>` in the C struct). The type info's `fields` and
  `field_count` list every word of the class, its bases' included (`ember_field_desc`: name and
  offset), and `ember_obj_new` zeroes the whole object. The header's `access` word stays a `Shared`
  payload's.
- **The whole-object write is the caller's.** A `mut self` call on a class object is bracketed by
  `ember_object_begin_write`/`end_write` in its caller, over every word of the object's dynamic
  class; a call on the caller's own `self` is a reborrow and takes nothing (`[EXC-5]`). Inside the
  method `self`'s fields are covered (`Body::mut_self`). Through an interface handle or a `dyn`
  box, the dynamic adapter, which knows the class, takes the write, and the caller none. Taking it
  in the callee, as before, could not tell a reborrow from an alias (D-359).
- **Field accesses follow loans.** `field_accesses` (`borrows.rs`), placed by the pass that
  brackets `Shared` accesses, gives each loan of a place through a field with a word (a view, a
  `ref`, a field passed or iterated) that word for the loan's exact NLL region, in its mode; a read
  or write through such a field without a loan (`h.items[i]`, `len(h.name)`, a store, a drop)
  holds it for that statement, or across a call. An access that would begin and end at one point
  is not emitted.
- **A returned view carries its access (`[EXC-18]`).** A body's summary lists the fields (or
  objects) of its arguments its result may borrow, computed to a fixpoint with the `Shared`
  summaries; the callee's access ends at its return and the caller begins the same access where
  the call returns, until the result's last use. Through a virtual or interface call the caller
  cannot know the field, and reads every field of the receiver object.
- **Cost.** The field checks are `static inline` in `ember_rt.h`: a load, a test and a store,
  with only a conflict a call. A `mut self` call checks one word per non-`Copy` field of the
  class. `[EXC-3]` elides no `mut self` write, since its callee receives the caller's handle and
  could copy it; `[EXC-8]` hoists a stable receiver's `mut self` calls in a counted loop
  (`mut_self_call_on_root`).

## ADR-061 — `String` is an `Array[u8]`'s buffer with a flag, not a kind of its own

**Decided 2026-09-26, with D-201.** `TyKind::Vec` carries `text: bool`; `String` is
`Vec { elem: u8, text: true }` (`CommonTypes::string`). Every generic part of the compiler (drops,
clones, layout, the borrow and store analyses, the C type `ember_vec`) handles both alike without
a new case, and Rust's exhaustiveness made every construction site choose. A kind of its own
(`TyKind::String`) would have needed a case in each of those, where a missed one fails silently
(a `String` never freed). Text is keyed on the flag, never on the element type.

## ADR-062 — one representation of `void` in the C backend

**Decided 2026-09-26, with D-355.** C has no zero-sized type and no `sizeof(void)`. Ember's `void`
is: zero bytes in a buffer, a box or `size_of` (`c_size`, `c_align`), with every element at the
buffer's base (`element_pointer`); a byte as a struct member or a parameter (`c_member_type`), since
C can declare neither as `void`; the value `0`; and `&(uint8_t){0}` where a runtime call copies
from an address, which copies zero bytes from it.

## ADR-063 — export main-thread identity follows the runtime lifetime

**Decided 2026-09-27.** `[FFI-33c]` requires the module's initialization
thread, including in shipping builds. For the current single-private-runtime
output, initialization records the runtime generation in a separate TLS word.
The exported body attaches first, then compares that word with the live atomic
generation. Detach does not clear main authority; shutdown advances the
generation, revoking every old authority before reinitialization.

An OS thread identifier would require platform-specific code and protection
against identifier reuse. A TLS address could likewise be reused after thread
exit. A per-thread generation avoids both and costs one TLS word, one atomic
load and comparison per checked entry. The existing generation-overflow abort
prevents wraparound from reviving an old identity. Initialization/shutdown
remain host-serialized lifecycle operations, as in the existing runtime.

This implements the current private runtime, not shared-runtime module
identity. Before shared-runtime or independently initialized module packaging
is accepted, the same authority must be scoped to each module instance;
the runtime's initialization thread cannot stand in for every module's.
No specification change or ODR is needed for this implementation choice.

## ADR-064 — capture-free function values retain both ABI entry points

**Decided 2026-09-27, with D-364 and D-365.** `[FN-6]` permits a capture-free
function value to become a C callback. Native borrowed records arrive by
address, while `[FFI-9]` requires their C values at the foreign boundary;
`[FFI-22]` also requires attachment on the callback itself.

A native function value is one pointer to a static descriptor containing its
native entry and, for a supported FFI-safe signature, a C adapter entry. The
fields are erased function pointers, cast back to the exact signature before
calling; there is no conversion between function and data pointers. Native
calls retain the original argument addresses. The C adapter attaches and
adapts C record values to native borrowed addresses before calling the body.
Already-C functions remain raw C function pointers. Captured/owned callable
representations are unchanged.

Native-to-C coercion is an explicit MIR operation, selecting the descriptor's
C entry even when a stored value was chosen dynamically. It allocates nothing
and performs no search. Native indirect calls still have one indirect call,
with one metadata load; each used native function needs two static function
pointers plus its eligible adapter. Direct-only functions need no descriptor.

Retyping a raw pointer loses the ABI distinction (D-365). Changing all native
calls to the C ABI would copy borrowed records and change their identity.
Searching a table at each conversion would preserve semantics but make cost
grow with the number of candidate functions. The descriptor keeps native
semantics and constant conversion cost. These are representation decisions;
the language rules already decide the required behavior, so no ODR or new
hardening is warranted.

## ADR-065 — export headers reuse the C backend's public type plan

**Decided 2026-09-27.** A generated header describes the currently defined
C-ABI functions, including ordinary `extern "C"` definitions. It excludes
imports and generated internal adapters. Reuse the C backend's signature,
definition, and type-name plan, but traverse only types reachable from those
exports so private native implementation types do not leak into the header.
Emit standard C includes and include `ember_rt.h` only when an exported
signature or reachable public type uses an existing 128-bit runtime carrier.

`ember build --emit header` emits only the header. `--emit-header` adds a
header to the existing build or `--emit c` output. The profile output is
`lib/<package>.h`, using the source stem when no enclosing manifest exists and
the enclosing manifest's `package.name` otherwise. Preserve the existing
`--emit c` stdout behavior; with an explicit output directory, also write
`c/<source>.c` as defined by CLI-2. Header and C emission remain available to
no-main library targets. Packaging archives and shared libraries is separate
follow-up work.

A reachable native-only function-pointer type that the C header cannot
represent produces an explicit header limitation error; this does not reject
the source program or emit an invalid header. These are output conventions and
representation choices, not new language rules or ODR changes beyond ODR-083.

## ADR-066 — package artifacts and runtime support use separate paths

**Decided 2026-09-27.** Keep the package header and static archive flat under
the profile's `lib/` directory. Put runtime support artifacts under the
branded `lib/ember_runtime/` directory, deriving the subdirectory name from
`SYMBOL_PREFIX`. A generated header that uses a 128-bit runtime carrier
includes `ember_runtime/ember_rt.h`. Header-only and additive header emission
copy the support header only when the generated header needs it; static-library
output always publishes both the support header and runtime archive.

This separates package names from runtime support names without reserving
package names such as `ember_rt`. File/prototype semantics do not change, and
the path layout requires no new hardening or ODR.

## ADR-067 — static-library packages namespace generated C types

**Decided 2026-09-27.** A manifest `staticlib` package takes its entry source
from `src/lib.<SOURCE_EXT>` or `build.entry`. Directory and no-input builds
produce a package archive and header plus the runtime support subtree from
ADR-066. Native implementation functions and count-wrapper helpers use
internal C linkage; declared C exports use external linkage, and the generated
C has no process `main`.

In static-library mode, generated named and structural C type names include
an injective namespace encoding the package-name bytes. Apply the same mapping
to generated C definitions and header declarations; leave foreign opaque tags
unchanged. This prevents separate package archives and headers from colliding
on record names or private standard helpers. MSVC objects embed debug data with
`/Z7`, retaining profile checks and LTO flags; rebuilding an archive starts
fresh so stale members cannot survive. Unsupported `cdylib`, native-library,
and shared-runtime packaging modes fail explicitly.

These are build and representation choices. File/prototype semantics do not
change; no new ODR or hardening is required.

## ADR-068 — preserve declaration overflow policy across compiler boundaries

**Decided 2026-09-27; implementation verified in the current H40 batch.** ODR-084/H40 makes overflow
policy lexical. Resolve a source module's default once and retain each
callable's effective declaration policy in its checked signature. Generic
instantiation preserves that fact. A closure captures the policy of its
declaration context; calling it does not replace the policy with the caller's.

When default expressions are inserted into a caller, use an explicit HIR
expression policy boundary. MIR lowering saves and restores policy at that
boundary, so supplied arguments keep caller policy and inserted defaults keep
declaration policy. Fixed-contract integer methods use the same mechanism to
protect their synthesized arithmetic; their receiver and supplied argument
expressions still evaluate under the caller's policy, before entering that
fixed computation. Constant evaluation consumes the same
resolved policies. Prefer boundaries to copying policy onto every synthesized
operator, which would duplicate state and make a missed constructor silently
change semantics. The boundary must remain transparent to ordinary type,
place, region, and effect traversal.

Effective declaration policy is part of the serialized callable contract and
its interface hash. Bump the artifact schema and reject invalid encoded policy
values; old schema records are invalidated before use. A module-only policy
edit must change the dependency identity seen by an importer whose defaults
or generic code can depend on it. This is compile-time metadata, with no
runtime policy parameter or change to callable ABI/type compatibility.

The present compiler still checks the whole loaded module graph each run.
Its interface artifacts are not a complete executable-code reuse cache:
default-expression and generic/inline expansion source identities remain
incomplete independently of this policy field. They must be included before
reusing such code across source changes. This decision does not claim that
broader cache work is complete or introduce unrelated private source text into
every public interface hash.

## ADR-069 — the dynamic exclusivity checks cost what they must

**Decided 2026-09-27 (ODR-085, owner-approved; Hardened_41).** The owner asked for the
`[EXC-19]` counters to be measured against a Swift-style per-thread list, then for the code to
come as close to C speed as possible. Four changes carry the measured design; each was first
built in a scratch copy behind a switch, measured against a no-check build on MSVC and clang, and
tested against every annotated test.

1. **Check-only accesses.** `insert_shared_accesses` asks of each class-field access whether it is
   *held*: whether code other than the access runs while it is active. It is not held when:
   * no statement in its interval is a drop whose type's drop can run program code
     (`drop_is_quiet`);
   * no terminator is a call other than a built-in that calls no code back (`quiet_builtin`, a
     closed list, conservative by default);
   * no other access begins at any of its points, including its own.

   Such an access is placed as its begin immediately followed by its end, and the C backend
   lowers that pair to `ember_field_check_read/write`: a load and a compare. The same-point rule
   keeps every verdict identical; dropping it breaks `EXC-1/run_fail_pushing_while_iterating_the_field.em`.
2. **Quiet functions.** `quiet_functions` is a greatest fixpoint over every body. It starts from
   all of them and removes any body that:
   * calls anything but a quiet built-in or another quiet function;
   * drops noisily;
   * touches a class field with a word, apart from its own covered `self` fields;
   * borrows a `Shared` payload;
   * returns a view.

   `convert_quiet_calls` moves the lowering's end of the caller's whole-object write to directly
   after its begin, but only when that access alone begins before the call and nothing begins
   after it. The conversion runs after `[EXC-8]` hoisting (`convert_quiet_calls_all` in the
   driver), so a loop's repeated counted writes on one receiver first become one checked interval,
   as the EXC-8 tests expect. For a final class the backend reads the object's packed words in place, eight bytes
   at a time. Any other class goes through `ember_object_check_read/write` over the dynamic
   class's type info. Treating every function as quiet breaks
   `EXC-2/run_fail_a_read_while_a_mut_self_call_writes.em`.
3. **Packed words.** Each class level's access words stand side by side before that level's own
   fields, with a zero `_access_padN` word after an odd count (objects are zeroed at allocation).
   A word then costs 4 bytes instead of 8, and a base class's layout stays a prefix. Type-info
   offsets come from `offsetof`, so the runtime's whole-object loops are unchanged.
4. **Checks nothing held could fail** (`access.rs`, `remove_never_firing_checks_all`, run last,
   after `[EXC-8]` hoisting).
   * Every access that is not a check is recorded as held: a field access by (declaring class,
     field name), a whole-object access by its class family, the root of the base chain.
   * A read check survives only if a write to its field or to its family's whole objects is held
     somewhere. A write check survives only if any such access is held. Every other check is
     removed and recorded as `no_conflicting_hold` (`[EXC-3a]`).
   * A `dyn` or interface call, or a `dyn`/interface place, turns the pass off, because the dyn
     adapters' accesses are made by the backend, out of the pass's sight.
   * Two tests break when their held writes are ignored:
     `EXC-3/run_fail_a_read_during_a_field_write_held_by_a_call.em` (field-level) and
     `EXC-3/run_fail_a_read_during_a_write_held_elsewhere.em` (object-level).

**Measured** (median of 7, the benchmarks in the handoff): every benchmark is within 0–4% of the
same program with no exclusivity checks, on MSVC and on clang. Before:

| Benchmark | MSVC before | clang before |
|---|---|---|
| Read loop | +52% | +1% |
| Walks over a million objects | +36–49% | +47–56% |
| `mut self` calls | up to +181% | up to +345% |

With six list fields per object, a million objects take 242 MB, against 275 MB before and 225 MB
without words.

**Rejected, measured:**
* A per-thread list of live accesses: slower in 6 of 8 benchmarks even with these changes, and
  up to +451% with 16 accesses live.
* A per-thread held-count gate: Windows TLS reads are not hoisted; clang's read loop became 52%
  slower.
* One-byte counters in the header's spare `access` word: they would move the check to the
  object's first cache line, away from the field.

**Tests.**

* **New run-fail tests:**
  * `EXC-19/run_fail_a_quiet_mut_self_call_while_a_field_is_viewed.em` and
    `run_fail_a_method_that_only_calls_quiet_functions_is_still_checked.em`;
  * `EXC-19/run_fail_a_derived_mut_self_call_while_a_base_field_is_viewed.em` (packing with a
    base class);
  * `EXC-3/run_fail_a_read_during_a_write_held_elsewhere.em` and
    `run_fail_a_read_during_a_field_write_held_by_a_call.em`.
* **New run-pass tests:** `EXC-19/accept_packed_access_words_in_a_base_and_a_derived_class.em`
  and `EXC-3/accept_a_check_nothing_held_could_fail_is_removed.em`.
* **New milestone tests** assert the check-only lowering, the quiet call, the packed layout and
  the `no_conflicting_hold` record.
* **Updated tests.** Seven run-pass programs asserted the old emitted `ember_object_begin_write`
  or `ember_field_begin_write`. Where nothing in the program can conflict, they now assert the
  calls are gone. The interface test counts the adapter's own call, since the `dyn` table's
  `access` entry also names the function (`[EXC-18]`).
* **EXC-8 test programs.** The two EXC-8 "keeps dynamic check" programs gained a `report` helper
  that holds a `Counter` across other code. The checks can then fail somewhere and stay, so those
  programs still test the hoisting boundary.
* **Safety-report test.** The safety side-table milestone builds
  `run-pass/class_mut_method_access_kept.em`, whose `mut self` method prints, so exactly one
  check stays.

**Limits.** Separately compiled Ember modules are outside the removal pass's view (it trusts
one compilation's bodies). Programs that use `dyn` or interfaces keep all checks the other
changes leave.

## ADR-070 — the speed audit's six fixes

**Decided 2026-09-27 (owner: "do all fixes check them and report the final result").** The
audit measured every Ember program in the set against a hand-written C program doing the same
work (C++ for classes and data-oriented code), built with Ember's own C flags, on MSVC and clang.
Six fixes, in the owner's order; none changes the language.

1. **Reference counts inline.** `ember_retain`/`ember_release` do the common case in the
   header; a class that is not `@sync` gets `ember_retain_plain`/`ember_release_plain`
   (`[THR-1]`: a class family has one Sync-ness). The last release, panics and atomic counts
   stay out of line.
2. **MSVC checked arithmetic.** 64-bit `+`, `-` and `*` use `_add_overflow_i64`,
   `_sub_overflow_i64` and `_mul_overflow_i64` on x64 from VS 2022 17.3 (`_MSC_VER` 1933), one
   flag test as clang's builtins give; `__mulh`/`__umulh` and a branch-free sign test otherwise.
   The old multiply found overflow with a division. Checked against the previous code on edge
   values and 20 million random pairs: no difference.
3. **Interface calls.** `ember_itable_lookup` is inline and scans the object's own entries; a
   class's list now carries its bases' entries too (`class_itable_entries`), so the out-of-line
   walk up the base chain (`ember_itable_lookup_slow`) is only a fallback. `[OBJ-2]` keeps an
   interface handle one pointer, so this is the lookup `[DSP-3]` describes, without the loop and
   the call. Found and fixed on the way: D-375.
4. **Push inline.** `ember_vec_push` stores in place when there is room; only growth calls out.
   A scalar or a handle is pushed by value (`ember_vec_push_i64`, `_f64`, `_ptr`, …), copied to
   memory only on the growth path. Passing `&local` had made every pushed local, a loop counter
   included, live in memory for the whole function; on clang, where that store landed decided
   whether p4 ran at 0.07 s or 0.10 s from run to run (4K aliasing against the length field).
5. **`[OPT-2]` loop versioning and `[SIMD-7]` grouped overflow checks** (`loop_version.rs`,
   run last).
   * Every counted loop whose bounds checks index a view at `i + c`, the view's base and length
     unchanged by the loop, gets an entry test and an unchecked copy, outer loops first, so the
     loops inside each copy are versioned in turn. The original blocks stay the checked loop.
   * Invariance is read through everything the loop runs: its statements; every call, through a
     summary of what each function (and everything it calls) can write that its caller sees —
     class fields by family, writes through each reference parameter, or anywhere — joined per
     symbol and computed to a fixpoint (a virtual call reads every override, an interface or
     indirect call and a built-in that runs program code the union of all); and every drop,
     through the `drop` functions it can run. A reference parameter and a class handle are never
     the same place as far as a loop can observe: a lent class field is held for the call. A
     `Sync` object is never invariant.
   * Every operation in the entry test is proved not to overflow by the tests before it. In the
     unchecked copy each view's header is read once into a local nothing else reaches, and the
     comparison that fed each removed check goes with it, so the C compiler keeps the base in a
     register; that is what let clang vectorise a11.
   * Vectorisable form (`[SIMD-5]`) is computed here: a counted loop with one exit, no bounds
     check left, every memory access a local or a view held in a local at the counter plus a
     constant, each written view at one offset and disjoint from every other view, no call (N4),
     no drop that runs anything, no check but integer overflow, no floating-point running total
     (N5). Such a loop runs in groups of 16. Each checked `+`/`-` computes its wrapped result
     and overflow bit in plain arithmetic, which clang vectorises. When no view the loop writes
     is also read in it, one flag per group: on a set flag the carried locals (found by a
     liveness pass) are restored and the loop as it was re-runs the group, so it panics at the
     first overflow exactly. Otherwise one bit per check and iteration, and the first set bit
     reports. The masks are tested before any access check and any inner loop in a group, so an
     earlier overflow is never overtaken. (Item 7 replaced the bits in most loops.)
   * A running total of at most 32-bit elements into a 64-bit total gets a copy with no check
     when the trip count is at most 2³¹ and the total starts within 2⁶² − 1 of zero. A running
     total the widths cannot prove safe keeps its checks one per operation: whether `[SIMD-7]`
     requires grouping it is ODR-086 (ERR-055), and nothing moves until the owner rules.
   * Tests: `tests/conformance/OPT-2/` (10) and `tests/conformance/SIMD-7/` (5), each checked
     against the previous compiler's output. Fifteen RC-2e retain counts grew because a
     versioned loop holds its retains once per copy.
   * D7 (recorded, then closed the same day) is in `DEVIATIONS.md`'s closed table.
6. **Memory.** A block whose alignment malloc already gives (two pointers' worth) is a plain
   `malloc` block, not `_aligned_malloc`'s, which added a prefix to every object; growth uses
   `realloc`, which can grow in place; `reserve(n)` and `with_capacity(n)` hold `n`, not the next
   power of two (pushes still double). `[OBJ-1]` fixes the 24-byte header, so shrinking it is
   the owner's decision (proposal in the audit report), not done here.

7. **Loops MSVC vectorises** (2026-09-28, owner: "just fix that msvc issue"). MSVC vectorised
   none of Ember's loops: it takes only a C `for` loop, and every Ember loop was labels and
   `goto`s. Three changes, each found with MSVC's own report (`/Qvec-report:2`):
   * **`for` loops.** A counted loop whose body is one chain of blocks (no `if` inside) is
     written as `for (; i < n; ++i) { … }` (`for_loops`, `emit_for_loop` in the C backend). A
     128-bit counter keeps the `goto` form (on MSVC it is a runtime struct). A loop whose body
     branches is not a `for` loop, so MSVC does not vectorise it; `[SIMD-6]` makes the C
     compiler's vectorisation evidence only, and clang takes such loops as before.
   * **Loop-private locals.** MSVC refused a loop that sets a function-wide temporary, since it
     cannot prove the value dead after the loop (reason 1104). A local that every `for` loop
     naming it sets before reading, in every iteration, and also reads, and that nothing
     outside a loop body names, is declared inside each such loop (`loop_scoped_locals`).
   * **No comparison in a group.** MSVC counts a comparison turned into a value
     (`overflow = x < 0`) as control flow (reason 1100). A grouped `+`/`-` computes an overflow
     word with shifts and masks only (`wrapping_form`): a `u64` whose top bit is set when it
     overflowed. The group's flag is those words OR-ed; its top bit is tested once, after the
     loop. Moving each word's bit down first (the first version) cost MSVC 11% on a list loop:
     clang merges those shifts itself, MSVC does not.
   * **Group size**, measured on list loops: 32 iterations for a loop with one check, 16 with
     several (`group_size`). With one check, 32 was the fastest on both compilers (at 16 MSVC
     spent 5% more than clang setting up each group: it keeps a small inner loop and re-tests
     that the lists do not overlap for every group, where clang unrolls the group). With
     several, 16 kept the slower compiler fastest.
   * **How a group reports** (`group_overflow_checks`):
     - *Report*, when the loop has one overflow check: the group runs once, storing as it goes;
       a set flag panics at the group's end with that check's message and location, which are
       the first overflow's whatever iteration it was. This is `[SIMD-7]`'s own description:
       the stores after the overflow are never read, because the process aborts.
     - *Re-run*, several checks: as item 5. When the loop also reads a view it writes, each
       group's elements of that view are copied aside at its start and put back before the
       re-run (`undo_saves`, `element_copies`), so the group runs once (owner, 2026-09-28:
       "make the last loop run in one pass"). That needs the loop to access the view on every
       iteration: then every element the group will touch is known to exist.
     - *Detect, then commit*, several checks and a view read and written but not on every
       iteration: each group runs twice. The first run stores nothing (each written view's element is a local, loaded
       where the iteration first needs its value; access checks are left to the second run;
       assignments nothing in it reads are dropped) and only sets the flag; the carried locals
       are then restored. A set flag re-runs the group checked on unchanged memory, which panics
       at the first overflow exactly; a clear one runs the group again for real with no check.
     - *Bits*, only where detect-then-commit cannot place its loads either (an element written
       on one path through the body and read after it on another): as item 5.
   * **Measured**, a list of 100,000 numbers, 20,000 rounds, against hand-written C with the
     same flags. MSVC does not vectorise the hand-written C of these loops (reason 1203), so
     its row compares Ember with checks on against Ember with checks off.

     | Loop | clang, checks on | MSVC, checks on |
     |---|---|---|
     | `out[i] = ((out[i] ^ round) + a[i]) & 1023` (one check) | 3% slower than C | 5% slower than clang (was 17%) |
     | `out[i] = (((out[i] ^ round) + a[i]) - 7) & 1023` (two checks, `out` read back) | 1.6 times C's time | 16% slower than clang (was 34%) |
     | `out[i] = (((a[i] ^ round) + b[i]) - 7) & 1023` (two checks, `out` not read) | 1.5 times C's time | 10% slower than clang |
     | Any of them with `@overflow(wrap)` | same as C | same as clang or faster |

     With several checks the compilers pull opposite ways, so the C says which form each gets
     (owner, 2026-09-28: "make ember write different C for each compiler"). The shift is
     `Const::OverflowShift`, written `EMBER_OVERFLOW_SHIFT`, which the runtime header sets to
     0 for MSVC (whole words ORed, the flag moved down once) and 63 for clang and GCC (each
     bit moved down first). Each compiler then runs its faster form: on the second loop of the
     table MSVC 0.68 s against 0.73 s the other way, clang 0.62 s against 0.68 s. MSVC's
     fastest form still trails clang's there: it spends more instructions on the same
     arithmetic.

     Running the second loop of the table in one pass instead of two took MSVC from 0.835 s
     to 0.754 s and clang from 0.665 s to 0.649 s. The second pass was not most of the cost:
     computing two overflow checks for every number is, which C does not do.

     Before this item the shorter version of the first loop (2,000 rounds) was 12% slower than C
     on clang and 3.4 times C on MSVC; it is now C's time on clang. A group of 64 or 256 instead
     of 16 was no faster (clang slower at 256). The p-set and a00–a12 are unchanged within
     noise, except a01 on MSVC (1.2 → 1.0 times C) and a11 on MSVC (0.83 → 0.53).
   * Checked: the quick check, the workspace suite on MSVC (305) and on clang (304; the one
     failure is `the_128_bit_halves_agree_with_int128`, whose runtime self-test cannot link on
     Windows clang: no `__divti3`), the gates, and clang `-Wall -Wextra -Werror` on every
     SIMD-7 and OPT-2 test. A sweep of every test program under clang found D-376, older than
     this work.
   * Tests: SIMD-7 gains `accept_a_rewritten_element_is_read_back_within_the_iteration`,
     `run_fail_an_overflow_after_a_rewrite_in_the_iteration_is_found`,
     `run_fail_a_rewritten_view_reports_the_first_overflow_by_iteration` and
     `run_fail_a_conditionally_written_view_reports_the_first_overflow`,
     `run_fail_a_view_accessed_in_a_branch_runs_each_group_twice`,
     `run_fail_a_view_written_on_one_path_keeps_a_bit_per_iteration`, and for the overflow
     words of every width `accept_grouped_checks_of_every_width` and
     `run_fail_a_grouped_{u8_add_overflow,u16_sub_borrow,i32_sub_overflow,u64_add_carry,u64_sub_borrow}_is_reported`;
     `run_fail_a_read_then_written_view_reports_the_first_overflow` now proves the report mode.

8. **Running totals block by block** (2026-09-28, ODR-086, owner: "for now I accept the 64
   block fix"). Neither reading of ERR-055 gave speed: grouping a running total's checks
   cannot vectorise it, and checking every addition left the sum 2.5× (MSVC) and 3.4× (clang)
   of C. Measured first in hand-written C: blocks of 64 were the fastest of six shapes.
   * A signed running total of at least 16 bits that the widths cannot prove safe
     (`block_totals`) runs in groups of 64 (`TOTAL_BLOCK`). Before each, the total must lie in
     [−2^(w−2), 2^(w−2)) (`totals_start_test`, a guard in `group_entry`), else the loop as it
     was runs the rest. In the group each value the total adds, offset by 2^(w−9), is ORed
     into a size word; a bit at or above w − 8 at the group's end means a value was too big,
     and the group re-runs checked from its start like any `Rerun` group. When the test
     passes no partial sum can leave the type in any order.
   * When nothing else in the loop reads the total, the group adds into an unsigned copy,
     written back at the group's end: MSVC vectorises only that form as a sum (reason 1105
     otherwise). Where the loop reads it, the total itself is added in wrapping arithmetic.
   * Measured, adding up 1,000 numbers 300,000 times: MSVC 2.5× → 1.5× C, clang 3.4× → 1.6×.
     The rest is the size test: three operations per number against C's one.
   * Tests: SIMD-7 `accept_a_sum_is_checked_block_by_block`,
     `accept_a_sum_with_a_huge_value_runs_its_block_checked`,
     `accept_a_32_bit_total_is_checked_block_by_block`,
     `accept_a_total_read_inside_the_loop_is_checked_block_by_block`,
     `run_fail_a_sum_that_overflows_in_a_block_is_reported`,
     `run_fail_a_total_that_starts_large_is_checked`,
     `run_fail_a_subtracted_total_that_overflows_is_reported`,
     `run_fail_a_32_bit_total_that_overflows_is_reported`.

9. **A handle copied from a list uses the list's count** (2026-09-28, `[RC-3]`, owner: "okay
   build it"). `t = things[i]; t.bump()` spent two thirds of its time retaining and releasing
   `t`: the loop was 3× C++ on both compilers, and deleting those two lines from the emitted C
   by hand made it match C++ on clang.
   * `uncounted_handles.rs` marks a class-handle local whose every assignment copies an
     element of a list (`Array` or fixed array of handles) that a user-written local or an
     `owned` parameter holds, never a bitwise copy of another place. On every path from each
     copy to the local's drop, the function must neither write, move nor drop the list (a
     field of an element's object may be written) nor take a mutable reference to it. Then
     nothing can take the object out of the list while the local lives: the borrow checker
     allows no mutable reference to the list across the copy, and a raw pointer may not be
     used after its reference's region (`[UNS-4]`). The list's count keeps the object alive,
     so the retain and release cancel and removing them moves no `drop`, no `Weak.upgrade`
     outcome and no foreign release.
   * The local must stay the same handle: never moved, never assigned anything else, every
     drop unconditional, and lent mutably only to a `mut self` receiver (`E2103` forbids
     assigning `self`) or a parameter that is not `mut`, through a call that returns no
     reference. Copies of it count themselves as usual.
   * The mark is `Body::uncounted_handles`, set on the final MIR (after `[OPT-2]`/`[SIMD-7]`);
     the C backend emits no retain for a copy into such a local and no release at its drop.
   * Measured, a `mut self` method on an object taken from a list, 100 million times: clang
     3.3× → same speed as C++ (all three programs); MSVC 3.0× → 2.0–2.1×. MSVC's rest is the
     loop's checks, not counting: C++ has no bounds or overflow check and uses `i & 1` for
     `i % 2`, and MSVC unrolls it five times; Ember's loop keeps the list-bounds check, the
     overflow check and the floor remainder's sign test, and is not unrolled.
   * Tests: RC-3 `accept_a_handle_copied_from_a_list_uses_the_lists_count` (retains counted),
     `accept_a_handle_outlives_its_list_being_cleared`,
     `accept_a_handle_outlives_its_list_cleared_by_a_call`,
     `accept_a_handle_outlives_its_list_element_being_replaced`,
     `accept_a_handle_repointed_by_a_mut_parameter_keeps_its_count` (each keeps the count;
     a wrong elision prints a `drop` early).

10. **Range facts over the MIR** (2026-09-28, `[RNG-4]`, owner: "proceed with range facts",
   after rejecting two pattern fixes as case-specific: "always suggest a general and optimised
   solution"). `[RNG-4]` says the compiler tracks known ranges and uses them to remove overflow
   and bounds checks; only its range-type use was built (in the type checker, D-141..D-150).
   * `range_facts.rs` computes, for every whole-number local no reference can write and the
     length of every list or span reached from a local through struct fields and references,
     an interval at each point and relations `a <= b + c`. Facts come from constants, types
     (a range type's declared range), arithmetic, copies and conversions (a conversion of a
     non-negative value is at most its source), `len()`, the arm of a comparison or `min`'s
     `TotalLess`, and a check that passed. Loop headers widen, then two narrowing rounds. A
     block that only sets `bool`s and branches is followed per incoming path, so `and`, `or`
     and `not` keep what each path knew. A list's length is forgotten at a write, move or
     mutable borrow of it or a place holding it, and at every call when some mutable borrow
     of it exists in the function; through a shared reference it cannot change (`[UNS-4]`).
   * Removed where the facts decide the check: overflow (the exact result, bounded for `a - b`
     by what is known between `a` and `b`, fits), bounds, division by zero, `MIN // -1`, shift
     range. A floor `//`/`%` made plain becomes a shift or mask for a power-of-two divisor, C's
     unsigned operators when both sides are non-negative, else the runtime's new flag-free
     `ember_floordiv_*`/`ember_floorrem_*` (the backend wrote C's `/` for an unchecked signed
     floor op, never reached before). Runs after the access passes, before `[OPT-2]`.
   * `[OPT-2]` now versions a loop for any index whose largest value the facts bound: a
     constant (`i % 4`, a mask, an inner counter, which the outer loop's test then covers), or
     `v + c` for a local the loop does not write (`n - 1 - i`); the `i + c` form is unchanged.
     A loop whose checks the facts remove outright is not copied.
   * `[RC-3]`'s list-element handles follow the header copy an unchecked loop reads through
     back to its list, so the method-call loop keeps both skips.
   * `[EFF-10]`: `Body::removed_checks`; the side table lists each as `STATIC_ELIDED` (range
     facts) or `DYNAMIC_HOISTED_LOOP` (loop entry test). Emitted bounds and overflow checks
     are still not listed there (older than this work).
   * Measured, a method on an object taken from a list 100 million times: MSVC 2.0-2.1x ->
     1.15-1.34x C++ (the floor `%` sign fix and the per-turn bounds check gone); clang the same
     speed as C++ before and after. The rest on MSVC is the language's overflow check on
     `count += 1` and MSVC not batching loops that have an exit.
   * Three regressions the first benchmark run found, fixed the general way: MSVC stopped
     inlining `push` once loop versioning made a function larger, so the runtime's
     per-operation fast paths (checked arithmetic, floor forms, push, reference counts, field
     checks) are `EMBER_INLINED`, forced inline on every C compiler; a floor `%`/`//` by a
     known-positive divisor with an unknown-sign dividend is C's operator corrected by the
     remainder's sign bit, with no branch; and ODR-088 (Hardened_43) keeps a loop whose integer
     division no vector instruction set does out of vectorisable form, so its checks are not
     grouped (`scalar_only` in `loop_version.rs`).
   * Tests: SIMD-5 `accept_a_loop_dividing_64_bit_integers_keeps_a_check_per_operation`,
     `accept_a_loop_dividing_32_bit_integers_by_a_constant_is_grouped`; RNG-4
     `accept_range_facts_remove_overflow_checks`,
     `accept_range_facts_remove_bounds_checks`, `accept_range_facts_make_floor_division_plain`,
     `accept_range_facts_remove_division_and_shift_checks`, and eight `run_fail_*` where a
     check must stay (a list emptied in the loop, by a call, in an object; a negative index;
     overflow, zero divisor, shift past the width, `MIN // -1` outside the facts); OPT-2
     `accept_an_index_bounded_by_range_facts_is_checked_once`,
     `run_fail_an_entry_test_that_fails_runs_the_checked_loop`,
     `run_fail_a_reversed_index_past_the_end_still_panics`. Counts updated in 15 RC-2e tests
     and 3 OPT-2 tests: loops the facts now prove are no longer copied, so their bodies (and
     retains) appear once.

11. **Across calls: `[CLO-3]` copies, inlining, running totals, division forms** (2026-09-28,
   autonomous, the owner's goal "address the other slow operations"). Calling a function passed
   as a value was 1.73x C++ on MSVC; MSVC calls through the function-value descriptor and does
   not inline the target.
   * `[CLO-3]` built (`callable_arguments.rs`): a call passing a known Ember function (named, a
     lambda, or a local that only ever holds it) to a callable parameter goes to the callee's
     copy for that function, in which calls through the parameter are direct; the signature is
     unchanged, and a copy that passes the parameter on specialises the next callee (memoised).
   * Inlining in the MIR (`inline.rs`), after `[CLO-3]` and before `[RNG-4]`: a function called
     from exactly one place is inlined there (the code does not grow), bottom-up and in a fixed
     order (a function only once nothing it calls is waiting; hash order once made the output
     vary between builds). A borrowed argument's parameter joins `uncounted_handles` (no retain
     in, none out). Not inlined: methods (`self`; tables and drop glue find them by type),
     implicit derives, closures with an environment, bodies with access transfers, `[FN-5]`
     bindings, hoisted intervals, interface or `dyn` parameters (their itable cache), foreign
     and exported functions, recursion. An inlined function nothing else names is not emitted.
   * `[RNG-4]` running totals: a local a counted loop changes only by one chain of checked `+`/`-`
     back to itself is bounded at the header by its entry range plus the most turns times each
     turn's change (the terms' ranges); seeded when it fits the type, then the analysis solves
     again. Checks in blocks the facts show never run go too.
   * Division by a positive constant of a non-negative value: unsigned when the unsigned magic
     number fits the width (`unsigned_magic_fits`, Hacker's Delight 10-10: two instructions),
     else C's signed operator, in the operands' width either way (so `[SIMD-5]` sees the division
     it is); by a variable, unsigned, 32-bit when both fit. A 32-bit narrowing of constant
     divisions was tried and dropped: it put loops C compilers do not vectorise into vectorisable
     form, whose grouped checks then cost more (MSVC 1.06x -> 1.62x on function values).
   * Measured: function values MSVC 1.73x -> 1.01x C++, clang 1.01x; integer arithmetic MSVC
     19% faster than C, clang the same speed (all its checks gone, the running total's included).
   * Tests: CLO-3 `accept_a_function_argument_is_called_directly`; RNG-4
     `accept_a_function_called_once_is_inlined_for_range_facts`,
     `run_fail_a_running_total_that_can_overflow_is_checked`, `range_facts` unit test of the
     magic numbers. Seven tests that inspect one function's C now call it twice, or run a loop
     to `xs.len()`, so it stays that function; `SIMD-5`'s per-operation test sums values
     nothing bounds.

12. **`sort` specialised to the element type** (2026-09-28, autonomous, same goal). Sorting
   numbers was 1.17x C++ on MSVC: the runtime's one merge sort compared through a function
   pointer and moved elements with `memcpy` of a runtime size, and it always allocated a buffer.
   * Each element type the compiler sorts itself (numbers, `bool`, `char`, text, `[STD-15]`) now
     gets its own sort, emitted from runtime macros with the comparison inline and elements moved
     as values: `EMBER_SORT_UNSTABLE` / `EMBER_SORT_STABLE` in `ember_rt.h`. `ember_vec_sort` is
     gone; `sorted` uses the same helper.
   * `sort` is stable (`[STD-15]`). Where values that compare equal are the same value
     (integers, floats under totalOrder, `bool`, `char`, range types of them;
     `equal_means_identical` in the C backend), no order among equals can be seen, so the sort
     is introsort: median-of-three quicksort, insertion sort under 16 elements, heapsort past
     twice log2(n) levels, so never worse than n log n, and no allocation. Text keeps a stable
     sort (runs of 32 by insertion, then bottom-up merges), since equal strings can still differ
     in capacity. Element types with an `Ord` of their own still go to std's `sort_ord` (D-256).
   * Measured (sorting 5 million `int`s, `a12`): MSVC 1.17x -> 0.82x C++ `std::sort`, clang
     0.96x -> 0.88x.
   * Test: STD-15 `accept_sort_past_one_run` (a text sort and an `int` sort past one run, with
     sorted, reversed and all-equal input; the C holds one of each macro).

13. **A `for` over a view's element iterator is a counted loop** (2026-09-29, autonomous, same
   goal). `for p in ps.iter_mut()` ran `next` each turn: an `Option` built from a null-or-element
   pointer, a switch on it and a done flag, which MSVC does not simplify (moving 100,000
   particles 2,000 times: 1.69x C). Now any `for` over a `SpanIter`/`MutSpanIter` (`iter()`,
   `iter_mut()`, of a list, a view or a fixed array, or an iterator already moved on) keeps the
   iterator as the loop's hidden local and counts from its cursor to its view's length,
   yielding `&source[i]` (`&mut` for `iter_mut`) exactly as `next` would (`check_for_indexed`,
   given a mutability and a first index). Measured: MSVC 1.69x -> 1.45x C, clang 1.03x -> 0.98x.
   What is left on MSVC is not in the loop: MSVC swaps the C twin's two loops and keeps each
   particle in registers for all 2,000 steps, which it does only for a perfect nest (nothing
   between the two loop headers). Ember's outer loop rebuilds the view each step, so it is not
   one. Tested by hand: the same nest with the view set up before the outer loop gets the swap
   (0.085 s, the C's 0.084 s); a C twin that keeps its particles in a growable-list struct, as
   Ember and `std::vector` do, takes 0.392 s, 3x Ember's. Open: hoisting loop-invariant view
   construction out of loops in the MIR. Test: CTL-1 `accept_a_view_iterator_loop_is_counted`.

14. **Element ranges; widening only what a loop writes** (2026-09-29, autonomous, same goal).
   A list loop with overflow checks on was 1.5-1.8x C (`out[i] = ((out[i] ^ round) + a[i] - 7)
   & 1023`): nothing bounded what a list held, so every `+` kept its check.
   * `[RNG-4]` element ranges (`element_lists`, `element_ranges`): an integer list local this
     function makes with `Array()` and changes only by element writes, `push`, `insert`,
     `extend` (any value) and calls that only reorder or remove (`sort`, `pop`, `clear`, ...),
     each through a `&mut` written once and used for nothing else, holds only values it stored.
     Its elements' range is the hull of every store's value range; reads through `xs[i]`, a
     shared view's `v[i]` and a shared element reference (`for x in xs`) get it. A copy of its
     header, a parameter, any other mutable borrow or way in, and the list is not followed. The
     facts are solved again with the ranges, twice at most: each round's come from facts that
     assumed the last round's, so each is sound, and they only narrow. A store by a checked
     operation counts its exact result: a failing check ends the program first.
   * Widening at a loop header now pushes to the type's end only the locals that loop writes;
     any other grew on the way in, from an enclosing loop that widens it, and is joined. Before,
     an outer counter (`round`) lost its range inside the inner loop, and narrowing could not
     bring it back through the inner back edge.
   * Measured (against clang's C, as the README compares list loops): b15 MSVC 0.93x, clang
     0.97x (was 1.8x / 1.6x); b16 0.98x / 1.02x (was 1.7x / 1.6x); b13 0.96x / 1.00x (was
     1.27x on MSVC). A total over `for x in xs` keeps its check: the list's length is not
     bounded, so neither is the total.
   * Tests: RNG-4 `accept_list_elements_hold_what_was_stored` (no check left),
     `run_fail_a_list_element_near_the_top_keeps_its_check`,
     `run_fail_a_list_changed_by_a_call_keeps_its_checks`,
     `run_fail_a_list_doubling_its_own_elements_keeps_its_check`.

15. **Interface calls try the implementing classes first** (2026-09-29, autonomous, same goal).
   Calls through an interface were 1.65x C++ on both compilers: each went object header, type
   information, table list, matching entry, method slot, then an indirect call, twice the
   dependent loads of a C++ virtual call. `[DSP-3]` fixes the representation (a class handle
   and a searched per-class list; `[OBJ-1]`/`[VER-4]` freeze the layouts), so fat pointers
   would need a spec change. Inside it: an interface call on a class handle, when the program
   has at most four classes whose table list holds the interface, first compares the object's
   type information with each (`interface_implementers`, `MAX_DEVIRTUALISED`) and calls that
   class's adapter directly, which the C compiler can inline; the table search stays as the
   fallback for any other class (one from elsewhere, a reloaded type). The adapter is the
   function the table holds, and it already reaches an override through the object's own class
   table (D-375), so the same method runs: `[PHIL-5]`, optimisation never changes observable
   behaviour. Not applied where `[DSP-3]`'s hidden-local cache already serves the call, or to
   a table of several interfaces.
   * Measured: MSVC 1.65x -> 1.26x, clang 1.64x -> 1.21x. The loop itself now matches C++
     (0.062 s both, zero-round runs subtracted). The rest is setup: the C++ twin never deletes
     its million shapes, while Ember frees them when the list's scope ends (0.019 s), and
     Ember's objects are larger (the 24-byte header `[OBJ-1]` fixes; 0.005 s).
   * Test: DSP-3 `accept_interface_calls_try_the_implementing_classes_first` (three classes,
     one inheriting the interface and overriding the method; five classes keep the search).

16. **A loop's invariant access check runs once, before the loop** (2026-09-29, autonomous,
   same goal). Refilling a list while other lists are viewed was 1.07-1.11x C on clang: each
   `push` checked that no access to `sink.items` was active (`[EXC-19]`), every turn. The
   existing hoister (`loop_access.rs`) moves a whole-object access around one direct call; this
   one (`check_hoisting.rs`, after loop versioning) moves a check, an access begun and ended
   back to back, which reads the access word and changes nothing. In a loop that begins and
   ends no access but such checks, drops nothing and calls only built-ins that run no Ember
   code (list and text growth and reads, printing, clearing plain values), on a handle local
   the loop never writes and nothing lends mutably, every turn's check reads the same word. When
   the check is also the first thing a turn does that anything could see (only pure local
   statements before it, on the one path in), it moves to a guard before the loop: the loop's
   test computed again into fresh locals and, when the loop will run, the check. A loop that
   runs no turn checks nothing, as before; one whose check fails panics at the same point, with
   the same output before it (`[PHIL-5]`). Recorded in the side table as
   `DYNAMIC_HOISTED_LOOP`, proof `no_access_in_loop`.
   * Measured: the three refill programs 0.95-1.04x C on both compilers (were up to 1.11x);
     removing the check by hand gave the same.
   * Tests: EXC-19 `accept_a_loop_check_runs_once_before_the_loop` (a zero-turn loop on a
     viewed list does not panic), `run_fail_a_loop_check_before_the_loop_still_panics`.

**Superseded by item 8:** a sum of 64-bit `int`s no longer keeps one overflow check per
element; ODR-086 was ruled on 2026-09-28.

## ADR-071 — a default method that builds a type from `Self` is made on use

2026-09-29, autonomous (D-379). Registering a generic instance's interfaces instantiated every
default method of every interface it implements. A default whose signature holds `Self` inside
another type (`fn take(owned self, n: int) -> Take[Self]`) names a new instance when it is
instantiated, and when that instance implements the interface too, as an iterator adapter does,
instantiating its defaults names the next: checking never ended.

* **Decision.** Such a default (`signature_nests_self`: `Self` in a nominal type's arguments, a
  tuple, an array, a view or a function type) is not made for a generic instance until a method
  call names it (`deferred_defaults`, made by `materialize_deferred_default` in
  `synth_registered_method`, the path every method call and every `Interface.method(x)` call
  takes). The implementation check counts a deferred default as defined. Every other default
  is still made at once, so nothing that worked changes.
* **Why not a depth limit** (Rust's `recursion_limit` for instantiation): every program with
  such an interface would build the limit's worth of instances of each adapter over each type
  and fail on the first chain longer than it; made on use, an unused adapter costs nothing
  (`[COST-1]`) and a used one is made once.
* **Why not every default on use:** 43 places read the method tables directly; the deferral is
  kept to the defaults that can recurse, and the one path that names them materialises them.
* **Amended the same day (`[STD-19]`):** a default with an `owned self` receiver waits too. No
  `dyn` table can hold one, so nothing needs it before a call; `Iterator` has fifteen, and made
  eagerly each would be built and checked for every iterator instance in every program. The
  concrete path defers them for a generic instance as well, and method calls materialise a
  deferred default at their start (`synth_method_call`), before any built-in method path looks.
  Checking a two-line program costs the same with and without the fifteen (73 ms).

## ADR-072 — `[STD-19]`'s adapters are written in std

2026-09-29, autonomous (ODR-089, Hardened_44). `Iterator` in `std.core` gets its adapters as
default methods returning generic structs (`fn take(owned self, n: int) -> Take[Self]`), and
each struct implements `Iterator` in a generic extension (`extend[I: Iterator] Take[I]
implements Iterator: type Item = I.Item`). Built: `take`, `skip`, `step_by`, `enumerate(start=0)`
and `zip`; the consumers `count`, `last`, `nth`, `fold`, `any`, `all`, `find`, `position`,
`for_each` and `to_array`. Every std iterator implements `Iterator`: the map and set iterators
through their own `next`, the view iterators (`SpanIter`, `MutSpanIter`, `SpanChunks`,
`MutSpanChunks`, `SpanWindows`) through their built-in one, which the implementation check
accepts as `Iterator.next` as it accepts built-in indexing as `Index.index`.

* **Why std, not the compiler:** `[STD-19]` puts the adapters in `std.core`, and the codebase
  rule is that a standard-library file says what it means. It took D-379 to D-382 to make that
  possible; each was a real gap a user's own generic code would have met.
* **Not yet:** `map`, `filter`, `filter_map`, `take_while`, `skip_while`, `flat_map`, `inspect`
  and `peekable` hold a closure, and a capturing closure cannot be stored yet (owned callable
  values, `[CLO-3]`). `chain` needs `J: Iterator[Item = I.Item]` between two parameters.
  `collect[C]()` and `join(sep)` need more than a bound on `Item` (a target collection, a
  `Display` item). `rev` needs a double-ended iterator.
  `[CTL-3b]`'s counted lowering of adapter chains in a `for` header came next (ADR-073).
* **More consumers, the same day.** `max`, `min` (`where Item: Ord`; among equals `max` gives
  the last and `min` the first, as Rust's do), `max_by_key`, `min_by_key`, `max_by`, `min_by`
  and `reduce` are defaults in std. `sum` and `product` are not: an iterator over a list gives
  references, and `[STD-5]` sums "in the element type", so the sum of `ref int`s is an `int`,
  which no bound on `Item` can say without references implementing the operator interfaces.
  The checker builds them (`synth_iterator_total`) for any `Iterator` whose items are numbers or
  references to numbers, as the loop a program would write (`total = 0; for x in it: total +=
  x`), so each `+` is checked as the operator is and floats add left to right; a type's own
  `sum` method comes first. A default method can now also call through its interface's
  associated type (`Item.default()`), as it names it in types (D-381).
* **The `Iterable` forms, the same day.** A method call whose name no method of the receiver's
  type has, but `Iterator` has (or `sum`/`product`), on a receiver with an `iter()` (a list, a
  view, a fixed array, a map, a set, any type with one) that is not itself an iterator nor
  text, is checked as `recv.iter().name(…)` from the receiver's source (`iterable_form`,
  reached where each method path would report the missing method), so the collection is
  borrowed, not moved, and a collection's own method of the name comes first (`Array.join`).
  `to_array()` over a list's iterator is refused as storing its references would be
  (ODR-069); an owned copy takes `copied()`/`cloned()`.
* **`copied` and `cloned`, the same day.** `std.core.Copied[I, T]` and `Cloned[I, T]` are
  adapters written in std, `extend[T: Copy, I: Iterator[Item = ref T]] Copied[I, T] implements
  Iterator` (the binding names `T`, so it comes first), each `next` giving the value its
  iterator's reference reaches. The methods are the checker's (`referenced_item`,
  `value_adapter`): `Item` being `ref T` is not a bound a default method can state, so a call
  on an iterator whose items are references builds the adapter over it, and one on
  non-`Copy` (or non-`Clone`) values is `E2040`.
* **`[CTL-3b]` in `for` headers, the same day.** `(a..b).step_by(k)` is the counted loop
  `range(start, stop, step)` already was (`check_for_stepped_range`, now with a `step_by` mode):
  the values are counted once, each is computed from its index, and a `k` of zero or less
  panics before the loop with `Iterator.step_by`'s words (a guard built as source, over a hidden
  local). `for x in view` over a `MutSpan` is `view.iter_mut()`, a counted loop, the view
  reborrowed rather than moved. The per-index value (`ember_range_nth_*`) is inline in the
  header now: `[CTL-3b]` allows no call per element, and `range(a, b, k)` made one. The
  chains of adapters followed (ADR-073).
* **`[RNG-4]` for stepped loops, measured and chased (the owner's rule).** Summing every third
  element of a million, 300 times, was 1.17x C on MSVC and 1.11x on clang, a bounds check on
  `xs[i]` in each turn among the cost. Range facts now know a stepped value: `RangeNth(start,
  step, index)` with `index` below a `RangeCount(start, stop, step)` of the same once-written
  locals and `step >= 1` lies between `start` and `stop - 1` (`range_counts`,
  `range_nth_value`). Three general fixes let the chain from the value to the list's length
  survive: a relation through a local that is forgotten (a temporary's storage ends right after
  it is read) is kept between the others (`a <= x + c1`, `x <= b + c2` give `a <= b + c1 +
  c2`); built-ins that only compute (`RangeCount`, `RangeNth`, the lengths, `TotalLess`) no
  longer forget a lent list's length; and the loop keeps its `stop` in a local that lives for
  the loop, as it did its `start` and `step`, which also evaluates the three left to right as
  Python does. Measured: MSVC 1.14x, clang 1.07x C. The rest, priced by hand-edited C: the
  overflow check on the running total, which the language requires (the total is not bounded
  across the two loops), and filling the list by `push` against the C's one `malloc`; without
  both checks the loop is 0.062 s against the C's 0.059 s. Tests: RNG-4
  `accept_a_stepped_loop_needs_no_bounds_check`,
  `run_fail_a_stepped_loop_past_the_list_keeps_its_check`.

## ADR-073 — a `for` over a chain of adapters is one counted loop

2026-09-29, autonomous (goal 2). `[CTL-3b]` is a MUST: a `for` over ranges or views with
`enumerate`, `zip`, `take`, `skip`, `step_by` and `copied` composed over them compiles to an
induction-variable loop, with no iterator object and no call per element. Until now such a chain
ran through each adapter's `next`.

* **Read off the checked iterator, not the source text.** `check_for_iterator` looks at the
  iterator expression after it is checked (`fused_shape`): calls of `Iterator`'s own `take`,
  `skip`, `step_by`, `enumerate` and `zip` (recorded by `DefId` when the default is made,
  `iterator_adapters`; a generic one, `zip`, through its instance's source), `Copied` literals,
  and at the bottom a view's element iterator (`SpanIter`, `MutSpanIter`) or a range's
  (`RangeIter`, `RangeInclusiveIter`, over integers of 64 bits or fewer). A type's own method of
  one of those names is not std's, so it is never taken for one. The pattern must bind what the
  chain gives: names and `_` anywhere, tuples over pairs, anything through a borrowed element
  (`fused_pattern_fits`); otherwise the loop runs through `next` as before.
* **The loop.** The chain's iterators and arguments are evaluated once, in the order the calls
  would have, each count checked where its adapter checks it, with its words (ODR-089;
  `adapter_count_guard`, shared with `(a..b).step_by(k)`). The number of turns is computed from
  the lengths and counts (`take` a minimum, `skip` a difference, `step_by` `(n - 1) // k + 1`,
  `zip` a minimum). Each turn computes each element from the counter: an index `k * step + skip`
  per adapter above it, then `xs[cursor + index]` read with no bounds check, since every index is
  below the view's length by construction, or `start + index` for a range's value. The read is
  `get_unchecked`, which MIR now lowers to the element's place (`&view[i]`) rather than a built-in
  call, so the loop is the plain loop a C compiler sees as one. That
  arithmetic never leaves `0..len`, so it wraps rather than checks. A range `a..=b` over 64-bit
  integers may hold 2^64 values, which no 64-bit count holds; its count is kept as "any, and the
  last index" and the loop is `0..=last`, so `int.MIN..=int.MAX` is exact.
* **Python's `enumerate(xs)` and `zip(xs, ys)` take the same path** (`[STD-26]`), so they gain
  what follows; `check_for_indexed` keeps only `for x in view` and `reversed`. A single name now
  binds `enumerate`'s pair (`for p in enumerate(xs)`), which was "needs two names, for now".
* **`enumerate`'s numbers, exactly as std's.** An item's number is `start + k`, and one past
  `int`'s top panics. A check on every turn kept the loop from vectorising (2.1x C on clang), so
  the loop takes only the turns whose numbers fit, with no check, and, when it runs to its end,
  panics there if std's adapters would have by then (`fused_numbers`, `fused_pulls`); the number
  is added as `usize`s and read back, since a C signed `+` that wraps is undefined. std's
  `skip` and `step_by` pull the items they pass over through `Enumerate.next`, which numbers
  them, and the call that finds a chain exhausted pulls the rest of `step_by`'s gap and one item
  past a `zip`'s shorter side; the end check follows those pulls down the chain. A differential
  test ran 17 chains at 8 starts near `int`'s top, fused and through `next` (136 programs, 57 of
  which panic): the same output and the same panic every time. std's `Enumerate` now holds the
  next number and whether the last one given was `int.MAX`, so the item numbered `int.MAX` is
  given and only the one after it panics; it panicked on giving the item numbered `int.MAX` (its
  `+ 1` for the next), and a count of items given would itself overflow after 2^63 items.
* **Ranges are `Iterable`,** as Python's `range` is: `r.iter()` (std's `RangeIter`,
  `RangeInclusiveIter`, `RangeFromIter`, generic over the integer types through std's private
  `Integer`, which gained `successor`), so `(a..b).skip(1)` is `(a..b).iter().skip(1)` by the
  `Iterable` forms, outside a `for` header too. `a..=b` ends with a `done` flag, so a range ending
  at its type's top never steps past it. D-384 came out of this.
* **A `_` still reads its element** into a hidden local, so the view, and the list it borrows,
  stays in use for every turn: `for _ in xs.iter().take(2): xs.push(0)` is `E3020`, as
  `for _ in xs` is (`[CTL-2]`). **A view that is not a variable is evaluated in the loop's own
  hidden binding** (`__xs`, a `for_iterator` one), so the temporaries it borrows live until the
  loop ends (`[EXP-4]`): `for x in head(make()).iter().take(2)`, and the plain
  `for x in head(make())`, which was `E3020` since before 0.9.9, now run.
* **Reviewed before it was committed** (the owner asked; an adversarial review, three reviewers
  and a skeptic per finding): four findings, all real, all fixed as the two bullets above and
  the unsigned number and std's `Enumerate` say. The reviewers found the evaluation order, the
  counts over every integer width and the pulls model sound.
* **Withdrawn: loops with calls as C `for` loops.** The first version kept the element read a
  built-in call and taught the C backend to write a counted loop with calls in it as a C `for`
  loop. MSVC then stopped inlining a `mut self` method called in such a loop (the method-call
  benchmarks went from 1.19x to 2.5x C: the same statements, a different inlining choice). The
  read became a place instead and the backend change was withdrawn.
* **Running values for clang and gcc** (`ember_analysis::strength_reduce`, the last MIR pass).
  In a counted loop, a value computed each turn as the counter times a number the loop does not
  change, plus another (`k * step + skip`, `start + k`, a stepped range's value; through `+`, `-`,
  `*`, casts between 64-bit types and `RangeNth`), gets a running value: set in a new block
  before the loop from the counter's value there, advanced by its stride in the loop's step
  (which a `continue` runs too), and read where the value was computed; the steps that computed
  it, now unread, are removed. Only 64-bit values, in 64-bit unsigned arithmetic, so a running
  value is exactly the value, and the step past the last turn cannot overflow; a checked
  operation is never touched (one the range facts proved safe is plain arithmetic by then).
  Hand-edited C showed clang rebuilding such a value from the counter in every vector lane
  (a multiply and an add), 1.26x C, and at C's speed with a running value. MSVC's vectoriser takes
  no second running value (reason 1104, the `enumerate` chains went 1.5x to 2.3x), so C for
  MSVC keeps the computed form: the pass runs when the C is not for `cl` (`c_for_msvc`: `--cc`,
  `EMBER_CC`, else the compiler the build finds), under the owner's rule allowing C written for
  each compiler. `SIMD-7/accept_independent_checks_are_grouped` now counts the grouped `+`'s two
  element casts, which the pass's own casts no longer disturb under clang. Test:
  `CTL-3b/accept_a_value_kept_running_is_the_same_value` (`continue`, `break`, nesting, `a..=b`,
  a value read after the loop, `@overflow(wrap)` arithmetic), its output the same from both
  compilers; break-tested (a wrong stride fails five `CTL-3b` tests under clang).
* **Measured** (`scratchpad/chain`, 1 million elements, 300 rounds, against hand-written C at
  the same optimisation; MSVC / clang): `xs.iter_mut().enumerate(start=round)` 1.48x / 1.08x;
  `xs.iter_mut().zip(ys.iter())` 0.98x / 1.09x; `xs.iter().skip(round).step_by(3).take(n)` 1.00x /
  1.05x; `xs.iter().copied().enumerate().skip(round)` 1.08x / 1.05x; `(round..n).step_by(2).enumerate()`
  0.99x / 0.98x. The few percent on the list rows is filling the list by `push` against the C's one
  `malloc` (17 ms against 12 ms for the two lists of the `zip` program); the range row, with no
  list, is at C's speed on both. The full README set was run too: every row within noise of the
  pushed compiler (A/B, interleaved runs). MSVC's 1.48x is not the loop: MSVC swaps
  the C's two loops (rounds inside, the element in a register), which it does only for a perfect
  nest; it is the open item particles already has (HANDOFF's speed verdicts). The first
  version of these benchmarks XORed into one total, and MSVC folded the C's rounds away, so they
  were rewritten to change the list each round.
* **Not yet:** `rev` (no iterator runs backwards yet), `cloned` (a clone is a call), a chain over
  `a..` (no count) or a 128-bit range, and `SoA` columns (`[SOA-*]` is not built); these run
  through `next`.

## ADR-074 — a view an outer loop leaves alone is made once; MSVC gets its perfect nest

2026-09-29, autonomous (goal 1, the owner's ">30% slower first"). Moving 100,000 particles 2,000
times ran 1.43x to 1.6x C on MSVC and at C's speed on clang: MSVC turns the C twin's two loops
around (each particle held in registers for all 2,000 steps), which it does only for a perfect
nest, and Ember's outer loop rebuilt the list's view and iterator before its inner loop every
step. Hand-edited C (scratchpad `nest/`) found what the nest needs, each alone doing nothing:

* **The view made before the outer loop** (`hoist_invariant_views_all`, in `loop_version.rs`
  beside `[OPT-2]`, after the other loop passes): out of every counted loop, the pure work each
  turn begins with whose inputs the loop cannot change moves to a block before it, rounds
  repeating so a view hoisted out of an inner loop moves on out of the outer one. Movable: the
  address of a local, a copy, a cast, `+ - *` and comparisons, a struct literal, a field read, a
  read through a reference whose target no write in the loop can reach (`loop_writes`,
  `may_alias`, with calls read through their summaries), and the view built-ins (`SpanFrom`,
  the reborrows, the lengths). Its result is set once in the loop, needs no drop, is read
  nowhere outside the loop and not earlier in the turn; an access check, a drop, a checked
  operation or a call ends the movable run. A division or shift stays (it could trap where the
  loop would not have run). The writes analysis also learned that making a view, reborrowing
  one or reading a length writes nothing through its argument: what is later written through
  the view is an element, which no header is (it counted `&mut list` given to `SpanFrom` as a
  write of the list, so no view was invariant).
* **A branch's comparison written into the `if`** (`folded_tests`): a comparison held in a
  temporary nothing else names is emitted as `if (!(i < n))` rather than through a
  function-wide `bool`, as the structured loops already did; MSVC did not take the loop for a
  canonical one otherwise.
* **A view's elements through a pointer of its own** (`view_pointers`): a view local set only
  whole, never inside a loop (the control-flow graph's cycles, by Tarjan's components) and
  never borrowed (an element reference leaves the view alone) gets `T* _N_ptr`, set wherever
  the view is and on entry for a parameter, and `view[i]` is `_N_ptr[i]`; MSVC re-read the
  struct field and would not reorder.

Measured: particles 1.02x C on MSVC (was 1.43x to 1.6x), 0.99x on clang. The full README set
with these changes: every other row within noise of the pushed compiler (A/B, interleaved
runs: 0.94x to 1.04x). Test: `OPT-2/accept_a_view_an_outer_loop_leaves_alone_is_made_once`
(an outer loop that pushes to its list, or rebinds it, makes the view again each turn and
sees the change; break-tested: with the loop's writes ignored it prints a stale total).

## ADR-075 — branches the range facts decide are folded; `enumerate(start=round)` gets its perfect nest

2026-09-29, autonomous (goal 1, the owner's ">30% slower first"). Numbering each item of a
1-million-number list while changing it, 300 rounds (`xs.iter_mut().enumerate(start=round)`), ran
1.47x C on MSVC and 1.07x on clang. MSVC turns the C twin's loops around (each element held in a
register for all 300 rounds), which it does only for a perfect nest. Ember's outer loop computed,
each round, how many numbers fit below `int`'s top (the room above `round`), cut the inner loop's
count to it and checked after the loop; and it held `start` in a hidden local. Hand-edited C
(scratchpad `en/`) found what the nest needs: with only the inner loop left, still 0.064 s, as
long as anything stood between the two loops; the round's counter read directly in the inner
loop, 0.047 s (C 0.043 s). Dead stores between the loops made no difference. Five changes:

* **One guard over the number machinery** (`fused_numbers`, typeck): no number can pass the top
  when every `enumerate` has no more items below it than its room, a single `safe` test; the loop
  then keeps its count and has no end check, both chosen by `safe`.
* **A length bounded by its element size** (`range_facts.rs`, `len_ranges`): a view's length is at
  most `PTRDIFF_MAX / size`, since no list holds more bytes (`[HEAP-8]`, the runtime's
  `vec_max_elems`) and no C object is larger; the element's size is taken at its fewest bytes
  (scalars' widths summed through structs, tuples and fixed arrays), elements of no size counting
  as one byte as the runtime counts them. With `round` below 300 and 8-byte items, `safe` is
  decided.
* **Branches the facts decide are folded** (`decided_branches`): a switch every run reaching it
  takes one way becomes a jump. A block that only branches has no state of its own (the solve
  follows edges through it), so each is judged on the states of the edges reaching it. The blocks
  no run reaches then are emptied, and temporaries nothing reads are removed. A fold can remove
  what a function reads (`for i in 0..0` over `self`), so `[MIR-REG-1]`'s summaries are made again
  after any fold, as after inlining.
* **Copy propagation** (`copies.rs`, before the view hoisting): a hidden scalar local set once as a
  plain copy of another local, whose address is never taken, is read from that local wherever the
  local still holds the value copied (a must-analysis: every path in passes the copy, and no write
  of the source nor its storage beginning or ending comes after); the copy goes once nothing reads
  it. Locals the programmer named keep their reads, for the debugger.
* **Hoisting keeps a nest perfect** (`hoist_from`): a computation from locals alone, which every C
  compiler moves itself, leaves a loop only when it leaves every loop around it too (nothing on a
  cycle through the loop's header changes its inputs). Moved into an outer loop's turn it made the
  nest imperfect (the round's `(usize)round`). Reads through references and the view built-ins are
  hoisted as before (ADR-074).

Measured: the `enumerate` row 1.09x C on MSVC (was 1.47x), 1.07x on clang (unchanged); what is
left is filling the list push by push against the C twin's single `malloc`. The full README set
against the pushed compiler (A/B, interleaved runs): every other row within noise (0.98x to 1.03x after re-running the three that first read 1.04x to 1.06x with 21 runs). The differential test (136 fused
chains against their unfused loops) shows no difference on either compiler.

Tests: `RNG-4/accept_a_length_is_bounded_by_its_element_size` (`len() * 8` of a view of `int` has
no check; of `u8` keeps one), `RNG-4/accept_a_branch_the_facts_decide_goes_one_way` (the code only
the other way reached is gone from the C),
`CTL-3b/accept_a_provably_safe_enumerate_checks_no_number` (no end check),
`CTL-3b/accept_a_start_changed_in_the_loop_keeps_its_first_value` (the copy is not read past a
write of its source). Break-tested: sizes ignored, folds off and copy kills ignored each fail
their test (the last prints 111 and 212 for 11 and 12). `SIMD-7/run_fail_a_view_accessed_in_a_branch_runs_each_group_twice`
now sets one `c` element to 0: with every element 1 the facts fold its `if`, and `xs[i]` is then
accessed on every iteration, which is not what the test is about.

## ADR-076 — at the end of `main`, a drop that only frees memory is left to the operating system

2026-09-29, autonomous (goal 1). Calls through an interface ran 1.24x C++ on MSVC and 1.15x on
clang, but the call loop itself was at C++'s speed (0.051 s against 0.054 s): 21 of the 27 ms were
`main` freeing its million objects one by one at the end, which the C++ twin (like the twins of
the two million-objects rows) leaves to the operating system. `[PHIL-5]` forbids only changing
observable behaviour, and freeing memory the process is about to give back is not observable.

`skip_exit_drops_all` (`exit_drops.rs`), run just before C emission in release and shipping
builds of a program with a `main` (not a static library, not `--leak-check`): from each return of
`main` back, the drops that end it go while each can only free memory: no `drop` method anywhere
in what it reaches (a class handle may point at any class derived from its class, an interface
value at any class of the program), no `@sync` class or `SyncShared` another thread could hold,
`Shared` and `Box` judged by what they hold, `Weak` never dropping its target. The first drop that
runs a `drop` method, or any other statement, stops the walk. Debug builds keep every drop for
their leak check.

Measured: interface calls 1.02x C++ on MSVC, 1.05x on clang; 1 million objects with a name and a
list 0.88x / 0.90x (was 1.06x / 1.05x); with 6 lists 0.91x / 0.91x (was 1.09x). Tests:
`DRP-2/accept_drops_at_the_end_of_main` (a `drop` that prints still runs, alone and through an
interface a class with a `drop` implements) and the milestone
`exit_drops_that_only_free_memory_go_in_release` (release frees one list fewer than debug in
`main`; break-tested with the pass off and with every type judged memory-only). Nineteen tests
that read weak releases and drop glue in `main`'s C now hold their program in `fn program()`, so
its values die before `main`'s last statement. The full benchmark A/B against the pushed compiler
(the pass off against on, interleaved runs): interface calls 0.84x to 0.85x the old time, the two
million-objects rows 0.84x to 0.88x, every other row within noise (the six that first read 1.04x to
1.09x: 0.98x to 1.02x over 21 runs).

## ADR-077 — a counted loop that pushes on every turn asks for its room first

2026-09-29, autonomous (goal 1). The enum `match` row ran 1.13x C on clang: filling a list of a
million shapes push by push (doubling, about twenty copies and every page touched twice) against
the C twin's single `malloc`. Hand-edited C with the room reserved first ran 1.03x (0.111 s ->
0.102 s, C 0.099 s), and faster than C on MSVC.

`reserve_pushed_lists_all` (`reserve_pushes.rs`, after the view hoisting): a counted loop with no
early exit (every block of it goes back to its header or on inside it), whose counter moves only in
its step and whose limit not at all, and that pushes onto a local list at least `k` times on every
path through a turn (the fewest pushes on any path, inner loops' back edges dropped), gets a block
before it that makes a reference to the list and calls the new built-in
`ArrayReserveHint { per_turn: k, inclusive }` with the counter and the limit. The list is the local
each push's reference was made from (`r = &xs`, made once, on each branch, or on each copy of a loop
that `[OPT-2]` versioned: every value the reference is given is `&` of that one local); in the loop the list
is not assigned, dropped or moved, every mutable reference to it is one the pushes use, the pushes'
references are used for nothing else, and nothing in the loop takes elements away (`pop`, `clear`,
`remove`, `drain`, `truncate`, `swap_remove`). C: `counter < limit ? ember_vec_reserve_hint(list,
size, limit - counter, k) : (void)0`, the difference taken in `uint64_t`.

`ember_vec_reserve_hint` never fails: a count past `[HEAP-8]`'s limit, or no memory (a
`realloc` that reports failure, growing in place when it can, as `ember_realloc` does), leaves the
list as it was, to grow and fail (if at all) where it would have
(`[PHIL-11]`). It acts only when the pushes would grow the buffer more than once (one growth, the
first push's four slots or a doubling, the pushes make as before), and it grows at least by
doubling, so a hint on each turn of an outer loop stays amortised. The capacity is the only thing it
changes, so the pass runs only in a program that never asks a list for its `capacity()`
(`[PHIL-5]`); std never does. A loop whose turns are constants and whose pushes fit in the first
growth gets no hint at all: a two-object fill (the `mut self` rows) kept its C byte for byte, where
an idle hint moved MSVC's hot loop (1.08x to 1.13x, layout alone).

Measured: enum `match` 1.00x C on clang (was 1.13x), 0.85x on MSVC, with C's memory (24.8 MB, was
more from doubling). The full set against the pushed compiler (A/B, interleaved): sort 0.91x the old time on both compilers, enum `match` 0.95x on clang (0.91x on MSVC in the first run), the chain rows 0.93x to 0.96x, every other row within noise (the one row that first read 1.10x: 0.99x over 31 runs, its C byte for byte the same). Tests:
`STD-15/accept_a_loop_that_pushes_on_every_turn_asks_for_its_room_first` (exactly three hints:
a push a turn, a push on each branch, an inner loop; none for a push on one branch only, a
`break`, or three pushes in all), `STD-15/accept_a_hint_counts_the_turns_of_any_counted_range`
(`a..=b`, a negative start, two pushes a turn, two lists in one loop, a range of no turn, a
versioned loop's two copies), `STD-15/accept_a_program_that_asks_for_capacity_sees_the_list_grow`
and `STD-15/accept_capacity_read_anywhere_keeps_every_list_growing_as_written` (`capacity()` read in
another function: 8 and 16, no hint anywhere). Break-tested: the `capacity()` gate off, "most"
pushes for "fewest", early exits allowed, and one reference definition only each fail a test. A
read-only review (three reviewers, a skeptic per finding) found no unsoundness; it found the
versioned loops missed, the hint's copy where `realloc` grows in place, and the test gaps above, all
fixed.

## ADR-078 — copy propagation sees every copy of a versioned loop

2026-09-29, autonomous (goal 1). ADR-075's copy propagation took a hidden local given one value,
`copy = source`. Loop versioning (`[OPT-2]`) copies a loop's blocks, and with them the statement
that holds a range's limit (`_22 = n` in the round loop of "Add two lists plus the round number into a third"), so the local is
given the same copy two or three times and was left alone, as ADR-077's pass first missed the
reference each copy makes. `copies.rs` now takes a local whose every value is a copy of the same
source: each of those statements starts the stretch where the local and its source agree, a write of
the source or its storage beginning or ending ends it, and all of them go once nothing reads the
local.

The adding-two-lists-plus-the-round row (w13) on MSVC loses its per-round copy but not its time:
the C twin runs in 0.014 s on MSVC (Ember 0.043 s, clang's C 0.045 s). Its machine code shows why:
MSVC turns the two loops around, so each position is read and written once and its 2,000 rounds run
in a register; it may do that only because the twin's lists carry `restrict`. Measured by hand
(scratchpad `wrap/demo/`): `restrict` added to Ember's C around the loop changes nothing, because
MSVC acts on it only on the parameters of a function it keeps separate (or on pointers set straight
from `malloc`); the loop moved into its own `noinline` function with `restrict` list parameters runs
0.027 s, and 0.014 s (the twin's time) with the round count, a constant in the program, written into
that function; the twin itself with a run-time count runs 0.026 s. The owner approved building that
(next ADR).

Measured: the full set against the pushed compiler (A/B, interleaved): every row within noise (the three that first read 1.06x to 1.09x: 0.99x to 1.01x over 31 to 41 runs; one of them had byte-identical C). Test:
`OPT-2/accept_a_range_limit_is_read_once_in_every_copy_of_a_loop` (a limit held once, the variable
it came from changed in the loop, the loop versioned: 25 and 9; break-tested: with the writes of the
source ignored it prints 16 and 4).

## ADR-079 — for MSVC, a loop nest that writes separate lists it never reads runs in a `restrict` function

2026-09-29, owner-approved ("go ahead with restrict, measure it first", then "yes build it"; "I
don't think it's a good idea to hard code numbers unless they are actually constant"). With MSVC,
"Add two lists plus the round number into a third" ran 0.043 s against the hand-written C's
0.014 s: MSVC turns the C twin's two loops around (each element read and written once, its rounds in
a register), which it does only because the twin's lists carry `restrict`. Measured by hand first
(scratchpad `wrap/demo/`): `restrict` written into Ember's C around the loop, on locals, in a block,
or on pointers taken from a list's header changes nothing; MSVC acts on it only on the parameters of
a function it keeps separate (`noinline`), or on pointers set straight from `malloc`. In such a
function Ember's loop ran 0.027 s, and 0.014 s once the round count and the round counter's start,
both constants in the program, were written in; the hand-written C in such a function with a count
known only at run time also runs 0.026 s.

`outline_list_kernels_all` (`kernels.rs`, MSVC only, after the view hoisting and the reserve hint):
a counted loop whose turn holds another, that calls nothing and leaves only through its header,
whose lists are locals it reaches only by index (or reads the length of), that writes at least one
list and reads at least one other, and that **never reads a list it writes**, moves into a body of
its own (`restrict_views`). Its parameters: a view per list, each length it reads, and each value it
reads before writing; a value that is a constant when the nest starts (set once to a constant, or,
for one the nest changes such as a counter, set to a constant on the single path into it) is written
in as that constant, so a count the program fixes stays fixed and one known only at run time stays a
parameter. A value the nest sets that is read after it keeps the nest where it is. The C generator
gives such a body's views as `T* restrict` element pointers (`EMBER_NOINLINE`, new in the runtime
header), and the caller passes each view's pointer.

The write-only condition is measured, not assumed: with it absent, "Change every number using a
second list" and "...reading back what it wrote" ran 2.2x slower and the decimal lists 2x: where a
round reads back what the last one wrote, the turned-around order makes each element's rounds a
chain no vector instruction can share (the same happens to MSVC's build of the C twins, README note
2). `restrict` is true for the views: two list locals never share elements, except two borrowed
parameters, which the nest can only read, where `restrict` promises nothing; the compiler's own
header copies of a list (`_45 = _4`, from `[OPT-2]`) are followed back to the list, and a written
list must share its root with no other.

Measured with MSVC: "Add two lists plus the round number into a third" 0.014 s against the C twin's
0.014 s, checks off and on (was 3.1x and 3.0x slower); "Change every number twice, writing to a
separate list" 0.086 s (was 0.39 s; MSVC's build of the C twin takes 5.2 s to 6.2 s, README note 2).
Every other row: identical C or within noise (the two that first read 1.07x to 1.09x were clang
rows with byte-identical C: 0.95x and 0.98x over 21 runs). clang is untouched. Tests:
`OPT-2/accept_a_loop_nest_over_separate_lists_keeps_its_results` (fixed counts and run-time counts
move; a value read after the nest and a nest reading back what it writes stay; same results on every
compiler), `OPT-2/run_fail_a_check_in_a_loop_nest_over_separate_lists_names_its_line` (a division by
zero in a moved nest names its line), and the milestone
`loop_nests_over_separate_lists_get_restrict_functions_for_msvc` (two moved nests, `restrict`
pointers, nothing for clang). Break-tested: with the value-read-after rule off `last` prints 0, and
with the pass off the milestone fails. A nest whose checks are grouped (`[SIMD-7]`) may leave its
loop other than through the header and is not moved; that is a limit of this pass, not a defect.

## ADR-080 — for MSVC, a loop nest that adds the same whole number to each element every round moves too

2026-09-30, owner-approved after the explanation ("okay go ahead with all 3"). With MSVC, "Add one
list into another" (`out[i] = (out[i] + a[i]) & 1023`, 2,000 rounds over 100,000 numbers) ran 46 ms
against the hand-written C's 36 ms. ADR-079 leaves such a nest in place because it reads back the
list it writes. MSVC's build of the C twin turns its loops around and then adds five rounds at once
(`x = (x + 5 * a[i]) & 1023`, 400 steps per number instead of 2,000; read in its machine code): adding
the same number five times is adding five times it once, and `& 1023` keeps that true. Where a round
adds something different (`^ round`), or decimals, whose rounding makes five additions differ from
one, nothing merges, and the turned-around loop takes each number's rounds one after another where
the loop as written adds to two numbers per instruction: 2x slower (the reason for ADR-079's rule).

`adds_the_same_each_round` (`kernels.rs`) lets such a nest move when the list it both reads and
writes holds whole numbers; its one element read and its one element write are at the inner loop's
counter (a copy or cast of it); the value written is that element plus a number no round changes
(built from constants, the inner counter, values the nest never writes, and elements of lists it
only reads), with no overflow check left on the addition, and at most `& (2^k - 1)` on the sum; the
nest has one inner loop, and no branch but the two loops' tests. A value the nest sets is the same
in every round only if each turn (or round) sets it before reading it: one carried from the last
turn (or round) does not count.

Measured with MSVC (11 runs each, the middle one): "Add one list into another" 0.048 s -> 0.039 s
with checks on and 0.047 s -> 0.039 s with checks off; side by side over 31 runs, 0.046 s -> 0.036 s
against the C twin's 0.036 s. The earlier estimate of about 0.030 s came from a noisy run; the build
of the first test compiler, which moved every read-back nest, also takes 0.037 s side by side. Every
other program of the 39: the same C. Staying in place, as measured to need: "Change every number
using a second list" and "...reading back what it wrote" (a different number each round), the
decimal lists; and the test programs multiplying by 3, adding the round, and xor. clang is
untouched. Tests: `OPT-2/accept_a_loop_nest_adding_the_same_each_round_keeps_its_results` (four
nests over the same lists; same results on every compiler and profile) and the milestone
`loop_nests_adding_the_same_each_round_get_restrict_functions_for_msvc` (only the nest adding `a[i]`
under `& 1023` moves; `^ round`, `& 1000` and an addition keeping its overflow check stay).
Break-tested: accepting any mask moves the `& 1000` nest too, and the rule off moves none; the
milestone fails both ways. Limits of this rule, not defects: a nest reading its list more than once
per turn, an addition whose overflow check stays, and subtraction or xor steps are left in place
unmeasured.

## ADR-081 — for clang and gcc, only a function holding a check is inlined before the C compiler

2026-09-30, owner-approved ("test the narrower larger of two", after the first version was stopped).
With clang, "A generic function (larger of two)" ran 0.143 s against C++'s 0.105 s. Ember inlines a
function called from one place in the MIR (`inline.rs`); with `larger` already inlined, clang proves
both values non-negative and turns the signed `max` into an unsigned one (`cmovbe`, two micro-ops on
this processor against `cmovle`'s one; LLVM issue 113965). Left a function of its own, clang inlines
it itself and keeps the signed form: 0.103 s by hand, 0.104 s built.

The first version left every such function to clang and gcc. All 39 benchmark programs agreed (5
changed C, none slower), but `RNG-4/accept_a_function_called_once_is_inlined_for_range_facts`
failed: Ember inlines a function so its caller's range facts reach the function's checks (`scale(i)`
with `i` below 1,000 cannot overflow `x * 3 + 1`), which a C compiler cannot do for checks already
written. So `inline_single_calls_all(.., only_checked)` inlines, for clang and gcc, only a function
holding a check (an `Assert` terminator or a checked operation), bottom-up as before: a function
whose callee brought a check in is inlined in the next round. A function with none is left to the C
compiler, which inlines as well by itself. For MSVC, whose inliner is weaker, nothing changes.

Measured with clang (11 runs each, the middle one): "larger of two" 0.147 s -> 0.109 s; two other
programs changed C (map of numbers, enum `match`), the same speed over 31 runs (0.139 s -> 0.140 s,
0.119 s -> 0.116 s); the other 36 programs have the same C. MSVC's C is the same for all 39. The
full suite passes with MSVC and clang, `RNG-4`'s inlining test and `CLO-3`'s lambda copies included
(a lambda holding a checked `-` is still inlined).

## ADR-082 — `[STD-19]`'s `chain` takes an iterator of the same items

2026-09-30, autonomous (the owner: "work on the implementation, just features that you can finish
in one hour"). `fn chain[J: Iterator[Item = Item]](owned self, other: J) -> Chain[Self, J]` in
`std.core`'s `Iterator`, and `extend[I: Iterator, J: Iterator[Item = I.Item]] Chain[I, J] implements
Iterator`: the first iterator's items, then `other`'s. A `done` flag stops asking the first once it
has said `None`, which an iterator need not keep saying. `other` is an `Iterator`, as `zip`'s is;
the spec lists `chain` among the adapters without saying more, and `[STD-19]`'s `Iterable` forms are
about the receiver (`xs.chain(ys.iter())`), so a range is chained as `(7..9).iter()`. Items of two
types are `E2040` at the call, naming both. It needed D-389 (the bound naming `I.Item` and the
binding naming the interface's own `Item`). Not built: `[CTL-3b]`'s counted lowering of a `chain`
in a `for` header (it runs through `next`, as `rev` and a fused `cloned` would; not yet compared with
C, which writes two loops). Tests: `STD-19/accept_chain_gives_one_iterator_then_the_other` (views,
ranges, copies, empty either side, a first iterator that panics if asked again, adapters and
`fold` after it) and `STD-19/reject_chain_of_a_different_item_type`.

## ADR-083 — a `for` over `a.chain(b)` is one counted loop per part

2026-09-30, autonomous (the owner: "resume the work", then "autonomous mode"). `[CTL-3b]`: a `for`
over a chain of adapters is a counted loop with no iterator object. `chain` (ADR-082) ran through
`Chain.next`: with one list of 1 million numbers chained to another, 300 rounds of
`total ^= x + round`, Ember took 3.7x the C twin's time with MSVC and 2.3x with clang (the twin
writes two loops).

* **The loops.** `check_for_chained`: when every part of `a.chain(b)` (any number of them,
  `a.chain(b).chain(c)` or `a.chain(b.chain(c))`) is a chain `fused_shape` accepts, and all give
  the same shape, the `for` is one `ForRange` per part, each the next one's `else`. A `break` in a
  part ends its loop and skips its `else`, so the rest never run, as `Chain` never asks `b` then;
  `continue` goes on in the same part; the written `else` is the last part's, so it runs only when
  no part breaks. Break depths are relative, and a `for`'s `else` is lowered after its loop leaves
  the loop stack, so the body's `break` and `continue` mean the same in every part. Every part's
  iterators and counts are evaluated first, in the order the calls make them (so a bad count in `b`
  panics before `a` runs, as it does when `b.take(n)` is made). Each loop reads its item, leaf by
  leaf, into hidden locals every loop shares; the pattern is bound from them and the body is
  checked once, and each loop runs a copy (the checked program's statements are now `Clone`). A
  `chain` under another adapter (`a.chain(b).enumerate()`, a `zip` over a chain) is not one loop,
  and runs through `next` as before.
* **Range facts through element references.** An element read through such a loop is `*r` with
  `r = &v[i]` of a view of a list; `r` is written once per part, from a different list each, and
  the pattern's name is a copy of it. `[RNG-4]`'s element ranges followed only a reference written
  once from one list, so `x + round`, which cannot overflow, kept its check and the loop did not
  vectorise (clang 1.8x). A local every whole write of which is an element reference or a copy of
  one now reaches every list those do, and holds the hull of their element ranges. This holds for
  every fused loop over a view, not only `chain`.
* **Measured** (11 to 21 runs, the middle one), `total ^= x + round`, overflow checks on: clang
  1.83x -> 1.03x, and 1.02x with checks off. The first version of the benchmark, `total ^= x ^
  round`, read 1.16x with clang only because the twin's constant count let clang see that an even
  number of `^ round` cancel; `+ round` compares the same work. With MSVC it stays 1.8x: MSVC does
  not treat the running `total` as a reduction in a function where a list's header was passed to
  a call (`push`), whatever the loop's form (a loop-carried dependence, reason 1200; a
  block-local total, a `restrict` local and `#pragma loop(ivdep)` do not change that). Moved by
  hand into a function of its own taking the view's pointer, the same loops run 0.117 s against the
  twin's 0.116 s. That is the next change (MSVC only).
* **Tests:** `CTL-3b/accept_a_for_over_a_chain_is_one_loop_per_part` (every part in order,
  `break` in either part with and without `else`, `continue`, a labelled `break` from an inner
  loop, adapted parts, ranges, copies, writing through `iter_mut`, `_`, and a `zip` over a chain
  that runs through `next`), `CTL-3b/accept_a_for_over_a_chain_makes_no_iterator` (no `Chain.next`
  in the C, two `for` loops; with the fusion off it finds `next` three times),
  `RNG-4/accept_an_element_read_through_an_adapter_keeps_its_range` (no check left; two without
  this change) and `RNG-4/run_fail_an_element_through_an_adapter_that_can_overflow_is_checked` (a
  part holding `int.MAX` keeps the check and panics).

## ADR-084 — for MSVC, a loop nest carrying one running value over views runs in a function of its own

2026-09-30, autonomous. After ADR-083, 300 rounds over two chained lists of 1 million numbers
(`total ^= x + round`) ran at the C twin's speed with clang and 1.8x with MSVC. Both inner loops were
plain C `for` loops that MSVC did not vectorise (reason 1200, a loop-carried dependence), where it
vectorised the twin's. Reduced by hand in the scratchpad (`chain/tiny*.c`): the same loops vectorise
in a function of their own, and stop vectorising once a list's header has been passed to a call
(`push(&a, ...)`, as Ember's lists are filled) in the same function; a block-local total, a
`restrict` local pointer and `#pragma loop(ivdep)` change nothing. The loops moved by hand into a
`noinline` function taking the view's pointer ran 0.117 s against the twin's 0.116 s.

`outline_list_kernels_all` (ADR-079) now also moves a nest whose lists may be views (`Span`
locals, passed to the new function as they are, as `restrict` element pointers) when it carries one
running value: a scalar set in the nest and read after it. The function returns it; the caller
assigns the call's result to it. Such a nest must write no memory (no element, field or write
through a reference, no mutable reference), so its turns share nothing through memory and
`restrict` on the views promises nothing false; the value comes in as a parameter when it is not a
constant at the nest's start (ADR-079's rule). Two running values, or a nest that writes a list and
carries a value, stay.

A first version also moved single loops carrying a value; measured over the 39 programs it gained
nothing and cost two: a final summing loop moved out of `main` made MSVC compile a loop that did not
move with two `xor`s on its running value's chain instead of one ("Number each item ... while
changing it", 1.31x), and an inner loop MSVC already vectorised in place ran 1.11x slower in a
function called each round. So only nests move, as ADR-079's do.

Measured with MSVC: the chained benchmark 0.210 s -> 0.117 s (the twin 0.116 s), checks on and off;
clang unchanged (1.01x). The 39 benchmark programs: the same C. Tests:
`OPT-2/accept_a_loop_carrying_a_running_value_keeps_its_results` (three nests move, over two views,
over one view with the value coming in, and a decimal one; two running values, a nest writing a
list and a single loop stay; the same results on every compiler) and the milestone
`loop_nests_carrying_a_running_value_get_their_own_function_for_msvc`; break-tested (with the mode
off no nest moves).

## ADR-085 — `@fastmath` and `@fp(contract)`: a C unit per float mode

2026-09-30, autonomous. `[TYP-9]` relaxes floating point within a function marked `@fastmath`
(every relaxation) or `@fp(contract)` (a multiply and an add may fuse, and nothing else), and
`[CG-C-11]` builds such a function "in a separate translation unit compiled with the relaxed
flags", so strict code keeps its own. ODR-090 (Hardened_45) ruled what the text left open: a float
mode is lexical, as ODR-084 made the overflow policy.

**What was built.**
- The checker reads the attributes (`FpMode` on signatures, HIR functions and MIR bodies). `@fp`
  names `contract` and nothing else, `@fastmath` takes no arguments (`E0104`, once where the
  attribute is written, however many types take an interface default carrying it). Either on a
  method with no body (an interface's, an abstract one) is `E0104`: it would govern nothing. A
  lambda takes the mode of the function it is written in.
- The backend writes the main unit and one unit per relaxed mode the program uses
  (`<module>.contract.c`, `<module>.fastmath.c`). Every unit declares every function and defines
  only those of its own mode; the entry point and the type information are in the main unit.
  `--emit c` prints the main unit, then each relaxed one.
- `ember_build::compile_relaxed_object` builds each relaxed unit with the profile's flags,
  relaxed: `-ffp-contract=fast` for `-ffp-contract=off` (clang, gcc), and under `@fastmath`
  `-funsafe-math-optimizations -fno-math-errno` for `-fno-fast-math`: every relaxation of
  `-ffast-math` but the assumption that no value is NaN or infinite, under which clang makes such an
  argument undefined behaviour (it marks the parameters `nofpclass(nan inf)` and then folds even a
  test of the bits; a `@fastmath` function called with a NaN could drop a bounds check, which
  `[PHIL-10]` forbids). clang-cl, which reads `/fp:fast` as all of `-ffast-math`, gets
  `/fp:precise` and clang's relaxations through `/clang:` (CI's clang-cl job caught it: the NaN
  test answered as under `-ffast-math`). MSVC gets `/fp:precise /fp:contract`, or `/fp:fast`, with `/Z7` for `/Zi`
  and no `/GL`, because link-time code generation could inline across the two modes (MSVC's
  documentation does not say it keeps them apart, so the object is finished code). The program
  links with the strict flags, so the fast-math link step, which sets flush-to-zero for the whole
  process, never runs. `@fp(contract)` permits fusing; the build targets baseline x86-64, which
  has no fused multiply-add instruction, so no x86-64 build fuses today (`math.fma` and
  `mul_add` are the single rounding that always holds, `[STD-3]`).
- What the language defines exactly for NaN and infinity holds in every mode (ODR-090):
  `is_nan`, `is_finite` and `is_infinite` test the bits (the runtime's `ember_f64_is_*`, in every
  unit); `x as int` finds NaN and the infinities by the bits before any comparison
  (`EMBER_FLOAT_TO_INT`); and a range value never holds NaN: in a relaxed function a float
  range's `checked` test (`RangeContains`, a new built-in the lowering uses there) and `clamped`
  call the runtime's `ember_range_contains_*` and `ember_range_clamp_*`, compiled strict and out of
  line. Before this, in a `@fastmath` function, `nan as int` was `-9223372036854775808` with
  MSVC and clang and `x.is_nan()` was `false` with clang.
- What a program has one of and compares by address is defined once, in the main unit, and
  declared `extern` in each relaxed one: an interface's id (the interface lookup compares it), a
  function value's descriptor (function values compare by it) and a `Shared` type's information.
  Written once per unit, a function value made in the `@fastmath` file was unequal to the same
  function made in the main file.
- The MIR inliner never inlines across modes (ADR-081's pass).
- A parameter default of another mode than the calling function's is checked as written first;
  if it does float work (`float_work`: float arithmetic, a comparison or conversion of floats, a
  built-in taking or giving one; a call's callee keeps its own mode), it is instead the body of a
  closure of its declaration's mode, called at the call, with `self` and the earlier parameters
  passed to it borrowed, as a closure does not capture `self`. A field default, a struct's or a
  class's, is strict the same way. Where no closure can hold it (a `mut` parameter's place, a
  mutating default), the call is `E0900` (NOT-IMPLEMENTED N7); a default is never evaluated in
  the wrong mode, and never silently: a first version re-checked a failed closure in the caller's
  scope and could leave the call an empty argument with no error.
- Class field defaults are now checked as a struct's are (`check_field_default`): in the declaring
  module, with the constructing function's locals out of scope, once per default however many
  constructions evaluate it (`[STR-2]`, `[DIA-14]`). D-393, found by the review: a class default
  saw the caller's locals, so `k: int = scale * 2` took `main`'s `scale`.
- `[RNG-4]`: inside `@fastmath` no fact about a float is taken from a comparison or from `min`,
  `max` or `clamp`, and inside either relaxed mode none from a float operation's result. Integer
  facts are unchanged.
- `[SIMD-5]`: a float running total no longer keeps a `@fastmath` function's loop out of
  vectorisable form (NOT-IMPLEMENTED N5, now waiting on `@parallel` only), so its integer checks
  are grouped (`[SIMD-7]`).
- A static library with a relaxed function is `E0900` (NOT-IMPLEMENTED N6): a library keeps its
  functions internal to its one unit, which a relaxed unit could not call.

**Found on the way: MSVC's reductions.** Compared with C (a dot product of two lists of 4 million
numbers, 300 rounds, scratchpad `fpbench/`), the `@fastmath` version ran 1.43x C with MSVC: MSVC
vectorised the C loop and not Ember's (reason 1105, an unrecognised reduction). Reduced by hand
(`fpbench/variants.py`): hoisting the list pointers, a counter of the loop's own, a local total
and a block around it changed nothing; writing `_3 = _3 + _8;` for the backend's
`_3 = (_3 + _8);` alone made MSVC vectorise it. MSVC finds a reduction only in the form
`x = x + e;`. An assigned binary operation is now written without its outer parentheses (`=`
binds loosest, so they never mattered). Fifteen tests whose C needles ended in that parenthesis
(`" >> 56ULL)"` and the like, in `SIMD-7/`, `RNG-4/` and `CTL-1/`) now end in the assignment's
`;`, each checked to count the same in every profile. The 39 benchmark programs: MSVC 0.98x to 1.05x
(`test/now`, where above 1 is faster now), every output the same; clang 0.98x to 1.13x, every output the same, and the three lowest
at 0.98x to 1.01x timed again 21 times (clang reads the two forms alike).

**Reviewed before commit** by an adversarial workflow the owner approved (three read-only
reviewers, then one skeptic per finding whose truth turned on the spec's reading; results in
`build/review-typ9/`). Seventeen findings, fifteen distinct. Fixed: the silent default (above),
class field defaults (not strict, and D-393), a `mut` default (now `E0900`), NaN and infinity in
relaxed files (above), the attribute on a bodyless method, the repeated `E0104`, `min`/`max`/
`clamp` facts inside `@fastmath`, defaults doing no float work made closures anyway, two test
gaps and three stale records. Refuted: that `@fp(contract)` breaks `[TYP-9b]` because x86-64 has
no FMA instruction (it is a permission; noted above), and that `min`/`max`/`clamp` facts are
unsound in `@fastmath` (they compare by totalOrder in the strict runtime; gated anyway, to match
the rule's words).

**Measured** (median of 21 runs; clang of 7; with the review's flags, 11 runs: clang 0.793 s
against 0.806 s, 0.98x; MSVC 0.802 s against 0.776 s, 1.03x):

| | C | Ember | Ember ÷ C |
|---|---|---|---|
| MSVC `@fastmath` | 0.754 s | 0.751 s | 1.00x (was 1.43x) |
| clang `@fastmath` | 0.712 s | 0.724 s | 1.02x |
| MSVC strict | 0.879 s | 1.095 s | 1.25x |
| clang strict | 0.863 s | 0.899 s | 1.04x |

The strict MSVC gap was 1.24x before this change too, and its cause is not found yet: in
that build ADR-081's inliner puts `dot` in `main`, where MSVC does not unroll the loop, and C with
the same inlining runs 0.80 s. It is the next speed item in the handoff.

**Tests.** `TYP-9/accept_a_relaxed_function_is_built_in_its_own_unit` (each relaxed function
defined once, in its unit; the interface id defined once and declared twice; a function value
equal across units; a lambda in its function's unit), `TYP-9/reject_a_float_mode_that_is_not_one`,
`TYP-9/accept_a_default_keeps_its_declarations_float_mode` (each default's arithmetic in the
right unit), `RNG-4/reject_a_float_fact_inside_fastmath` (the integer fact and the `@fp(contract)`
comparison stay), `SIMD-5/accept_a_float_total_in_fastmath_is_in_vectorisable_form`,
`SIMD-5/run_fail_a_float_total_in_fastmath_reports_its_first_overflow`, `ember_build`'s
`a_relaxed_unit_is_compiled_to_fuse` (the relaxed flags let clang fuse `a * b + c`), `staticlib.rs`'s
`a_static_library_with_a_relaxed_function_is_refused` and `ember_codegen_c`'s
`only_parentheses_around_the_whole_are_dropped`; from the review,
`TYP-9/accept_nan_and_infinity_keep_their_meaning_in_fastmath` (fails with clang under
`-ffast-math`), `TYP-9/reject_a_float_mode_on_a_method_without_a_body`,
`TYP-9/reject_a_mut_default_doing_float_work_from_another_mode` and
`STR-2/reject_a_class_default_does_not_see_the_constructors_locals`, with a class default, a
lambda's constant and the `clamp` case added to the tests above and a grouping needle to the
`SIMD-5` run-fail one. Break-tested: without the closures for defaults, the unit split, the single
definition, the float-fact rule, the relaxed flags, the `[SIMD-5]` admission, the lambda's mode,
the class defaults' strictness, the bodyless-method check or the finite-math exclusion, its test
fails.

## ADR-086 — for MSVC, a counted loop runs on block-local copies of the numbers it sets

2026-09-30. ADR-085's strict dot product (two lists of 4 million numbers, 300 rounds, scratchpad
`fpbench/`) ran 1.17x its C twin with MSVC (1.119 s against 0.954 s) and 1.05x with clang. Both
loops are one multiply and one add per element on one register chain; MSVC unrolled the C loop
four times (ten when the C `dot` is inlined into `main`, as ADR-081 inlines Ember's) and Ember's
not at all. Reduced by hand (`fpbench/unroll_variants.py`, assembly only): its own counter or one
index copy changed nothing; loading the list pointers once before the loop, or keeping the total
in a block-local copy, each made MSVC unroll it four times. Ember's total is a variable of the
whole function, set in the loop and read after it: MSVC does not unroll a loop that sets one, as
ADR-070 item 7 found it does not vectorise one.

`emit_for_loop` now wraps a counted loop that sets such a variable, for MSVC:
`{ T _7_in = _7; { T _7 = _7_in; for (...) {...} _7_in = _7; } _7 = _7_in; }`. The inner block
redeclares the name, so the loop's C is unchanged. The variables are the integer and float
locals the loop's statements set whole, other than the counter and the loop's own temporaries,
whose address the function never takes (no reference could reach the copy instead), that are not
parameters, view pointers or folded tests (`carried_scalars`).

Measured, median of 15: MSVC 1.119 s -> 0.967 s, 1.01x the C twin (0.954 s). The 39 benchmark
programs with MSVC: 0.97x to 1.03x, the same outputs. With clang the copies were not neutral:
`a01_int_math` ran 1.25x slower (21 runs), because clang then used 64-bit division by 3 and 5
where the loop's counter had let it use 32-bit ones, and `w13_checked` 1.12x. clang unrolls
either form, so its C keeps the totals where they are (`emit`'s `for_msvc`, from the driver's
`c_for_msvc`), and its C is byte for byte what it was. Tests:
`CTL-3b/accept_a_running_total_keeps_its_results` (five totals: float, integer, two in one loop,
one read after a loop reading it inside; the results on every compiler) and the milestone
`loop_totals_are_block_local_copies_for_msvc` (ten copy lines for MSVC, none for clang);
break-tested (without the copies the milestone fails).

## ADR-087 — `rev`: iterators that run backwards, and `rev` in a counted loop

2026-10-01, autonomous (the owner: "continue working autonomously but solo"). ODR-091
(Hardened_46) ruled which iterators run backwards; this is how it is built.

**In std** (`std/src/core.em`, `collections.em`). `DoubleEndedIterator: Iterator` declares
`next_back` and the default `rev`, which makes a `Rev[I]` whose `next` is `next_back` and the other
way round; `ExactSizeIterator: Iterator` declares `len`. The ranges' iterators implement both
(`RangeIter` moves its `end` down; `RangeInclusiveIter` sets `done` when the two ends meet, so a
range at a type's bottom never steps below it); `std`'s private `Integer` gains `predecessor` and
`distance_to` (a length as an `int`, panicking past `int.MAX`). `Copied`, `Cloned` and `Chain` run
backwards over what runs backwards; `Take`, `Skip`, `StepBy`, `Enumerate` and `Zip` over what also
knows its length, each written as Rust's is: `take` and `zip` trim from the back to the length
`next` would give, `step_by` finds its last item from the length and whether its first was taken,
`enumerate` numbers the back item `number + len - 1`. `(0..=n).rev()` on a range, and `xs.rev()` on
a list, are `iter().rev()` as the other adapters' `Iterable` forms are.

**Built in.** A view's element iterator (`SpanIter`, `MutSpanIter`) runs backwards by shrinking the
view it holds: `next_back` lowers to `len -= 1` and the element at the new length
(`lower_span_iter_next_back`, reusing `SpanIterNext`'s extraction); `len` is the view's length less
the cursor. The two ends meet, so mutable items stay disjoint. Both are declared in std with no
body (`pass`), as `next` is.

**In a `for` (`[CTL-3b]`).** `rev` is one more link of a fused chain: every node below it gets the
step `Rev(last)`, which maps the loop's counter `k` to `last - k` before the steps below apply, and
the count is unchanged. So `rev` anywhere in a chain of `take`, `skip`, `step_by`, `enumerate`,
`zip` and `copied` over a range or a view is a counted loop with no iterator and no `next_back`
(`xs.iter().skip(2).rev()` reads `last - k + 2`). An `enumerate` below a `rev` gives its greatest
number first, so the loop cannot cut its count to the numbers that fit as it does forwards
(ODR-089): the check is before the loop, `start + n - 1 > int.MAX` panics when the loop has a turn,
as `Enumerate.next_back` panics at its first call.

**Measured** against C twins counting down (`for (i = n; i-- > 0;)`), 10 million elements, 60
rounds, median of 11: a float sum 0.332 s against 0.366 s with MSVC (0.91x) and 0.336 s against
0.317 s with clang (1.06x); a store `out[i] = i * 3 + r` 0.314 s against 0.325 s (0.97x) and 0.333 s
against 0.322 s (1.03x). The clang sum's difference is all in building the list: with no rounds
Ember takes 0.028 s (10 million `push`es) and C 0.009 s (stores into `malloc`'s block); the loops
themselves take 0.305 s and 0.310 s.

**Found on the way.** D-394: a child interface could not name its parent's associated type when
the parent was collected after it, as std's `Iterator` was for any user module (interfaces were
collected importers first) or a parent later in the same file. Interfaces are now collected
dependencies first, and a parent before its children within a module.

**Tests.** `STD-19/accept_rev_runs_an_iterator_backwards` (each adapter backwards, `next` and
`next_back` meeting, `len`, ranges at a type's top and bottom),
`CTL-3b/accept_a_for_over_rev_is_a_counted_loop` (every position of `rev`, a mutable view, and
`(0..=n).rev()`; no `next_back` in the C), `STD-19/reject_rev_of_an_iterator_that_cannot_run_backwards`
(an endless range; the error says why), `STD-19/run_fail_a_reversed_enumerate_past_the_top_panics_first`
and `STD-19/accept_a_reversed_enumerate_at_the_top_fits` (both the loop and `to_array`; the second
caught an off-by-one in the loop's check), and `IFC-3/accept_a_child_interface_names_its_parents_associated_type`
(D-394).

## ADR-088 — overlapping implementations are found where they are declared

2026-10-01, autonomous (the owner: "continue working autonomously but solo"). ODR-092
(Hardened_47) ruled when two implementations overlap; this is how the checker finds them.

**One pass, after every implementation is collected** (`check_overlapping_implementations`, run
once the bounds are known). Every implementation is a record: the type it is for, split into a
shape and arguments (`Pair` and `[T, int]`; a reference, a view, a tuple, a function type and a
fixed array are shapes of their own; any other type is only itself), the interface's name and
arguments, and the type parameters it is over, with their bounds. The records come from the generic
extensions, a generic type's own `implements`, the blanket implementations, and the concrete
implementations collected. Two records of one interface overlap when their arguments unify, each
side's parameters kept apart from the other's even when they share a name (`T` in two `extend`
blocks is two variables), with an occurs check; an associated type unifies with anything. Then each
parameter the unification made a type with no parameter left must meet its bounds; a parameter
left open, or made a type that still has one, meets any (ODR-092). An overlap is `E2041` at the
later implementation, naming the earlier, once per implementation.

**What a generic recipe's interface is.** A generic extension keeps its `implements` as written, to
resolve at each instance. The pass resolves it once over the extension's own parameters (the
recipe's instance over them for `Self`, found, never made); what that resolution reports is rolled
back, as applying the recipe reports it.

**The old check stays for two concrete implementations** (`collect_implements`: the same type and
interface twice), now naming the first. When either implementation of a collision there comes from a
generic recipe, the pass reports it instead, so an overlap is reported once where it is declared,
not at every instance that meets both. A concrete implementation refused there because a recipe's
already gave the type the interface is set aside for the pass (`set_aside`).

**Cost.** Pairs are compared only within one interface; a trivial program checks in the same time
(0.043 s against 0.042 s, median of seven).

**Tests.** `TYP-19/reject_overlapping_implementations_are_found_where_declared` (five overlaps no
program uses: parameters named apart and bounded apart, `Pair[T, int]` against `Pair[str, U]`, a
nested instance, a type's own `implements` against an extension, a generic against a concrete; each
compiled before), `TYP-19/accept_implementations_that_cannot_meet_are_both_kept` (a bound a named
type misses, different interface arguments, different shapes), and
`TYP-20/reject_two_implementations_of_one_interface_name_both`.

## ADR-089 — N1 built: a range type's generated implementations are answered by rule

2026-10-01, autonomous. Closes ADR-016's deviation and NOT-IMPLEMENTED N1: the operator interfaces
exist (ODR-040), so a bound can ask for a range type's generated implementations, and
`fn twice[T: Add[T]]` refused a `Unit`.

**How.** The implementations are not written anywhere: `implements` answers them by rule
(`range_operator_implements`). For each of `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem` that the
representation `R` has over itself, a range type `T` over `R` has `Op[T]` and `Op[R]`, and `R` has
`Op[T]`; `T` has `Neg` when `R` does. Each one's `Output` is `R` (`range_operator_output`, read by
`implementation_assoc` and `project`, so `T.Output` of a bound is `R`). `Ord` is the
representation's (`totally_ordered`); `Eq` and `Hash` already were. A call through a bound checked
again on a range value is the built-in operator, as `[RNG-5]` defines it, and so is the method form:
`r.add(x)` erases `r` to `R` and is `R`'s `+`, where a `Unit` argument coerces. Answering by rule
rather than recording implementations in the declaring module has the same observable content:
`[TYP-20]` asks where an implementation may be written, and these are written nowhere, by the
compiler, for types of the declaring package.

**ODR-093, built with it.** A range value printed, formatted or shown inside an aggregate is its
representation's value (`range_as_repr` at `print` and f-strings; codegen's `builtin_suffix` for an
element), and `Display`/`Debug` bounds hold as the representation's. `Default` does not.

**Tests.** `RNG-5a1/accept_generated_implementations_meet_bounds` (`Add[T]`, `Add[f64]`, `f64: Add[Unit]`,
`Neg`, `Ord`, `Display`, `T.Output`, `r.add(x)`, a `Map[Level, int]`; `E2040` before),
`RNG-5a1/reject_what_a_range_type_does_not_implement` (`Default`, `Add` of another range type, an
integer's `Div`, a float range as a `Map` key), `RNG-8/accept_a_range_value_shows_as_its_representation`
(`println`, f-string specs, inside an `Array`; `E0900` before).

## ADR-090 — `RandomState`: SipHash-1-3 keyed once per process from the operating system

2026-10-01, autonomous. `[HASH-2]` names `std.collections.RandomState`, "a per-process randomly
seeded hasher for maps keyed by untrusted input"; it did not exist.

**The hash.** SipHash-1-3, the keyed hash Rust's `RandomState` uses for the same purpose: which
keys collide depends on a 128-bit key an attacker does not know. The message is the hasher's writes,
each one 64-bit word (bytes eight to a word, as `DefaultHasher` takes them), and the result is
SipHash-1-3 of those words' little-endian bytes, checked against a reference that reproduces the
published SipHash-2-4 vectors (four inputs, equal to the bit). `write_bytes` ends with the tail's
byte count in the tail word's top byte (D-405), so no two byte strings give one message.

**The key.** Drawn at the first `RandomState.new()` by the runtime's `ember_process_key`: `rand_s`
on Windows (the CRT's call to `RtlGenRandom`, so no library to link), `getrandom` on Linux and
`arc4random_buf` elsewhere; should the source fail, the time, a clock reading and two addresses. The
first caller draws it and publishes it with a release compare-exchange; any other waits for that.
std reaches it through a private `process_key(which)` whose call the checker makes the built-in
(`Builtin::ProcessKey`), as `std.mem`'s staged functions are.

**Not built:** ODR-033 makes creating a `RandomState` `Nondet`; the effect system that would record
it is Phase 4's (`[EFF-*]`, `[DET-2]`).

**Tests.** `HASH-2/accept_a_random_state_map_keeps_insertion_order` (a `Map[String, int,
RandomState]` in insertion order; one key, one hash within a run),
`HASH-2/accept_bytes_differing_in_their_tail_hash_apart` (D-405, both hashers), and
`random_state_is_keyed_per_process` in `milestones.rs` (one program run twice hashes a key two ways).

## ADR-108 — what the README run and CI found in ADR-106: unused std instances, a view's copy, gcc

2026-10-02, while finishing ADR-106 and ADR-107 for the README run.

**Unused std instances are not kept (D-474).** Every implementing type gets a copy of each of an
interface's provided methods, and every generic type's instance its methods; ADR-106's six text
iterators therefore gave every program about 150 bodies to borrow-check, 4x the compile time. Such
a body, when its declaration is std's, is now kept only if a call, a function value or an
interface table's implementation reaches it (`[COST-1]`'s rule for implicit derives, which
`prune_unused_implicit` already applied). A class's method and a `drop` stay, since a vtable and
drop glue name them without a call. The MIR inliner still treats these bodies as before (they are
not `emit_if_used`). 10-line program: 0.38 s → 0.11 s (release), 2.5 s → 0.47 s (debug). A copy of
a user interface's provided method is checked as before, used or not.

**A body whose callees did not change is not inferred again (D-476).** The fixpoint of callable
summaries inferred every body in every round, five fixpoints a compilation; a body whose direct
callees' summaries all stayed the same in the last round now keeps its summary (a closure body's
change still infers every body, since capture contracts reach their creators without a call). Same
rounds, same fixpoint, the same C for every benchmark program; a 57-line program of seven adapter
chains 2.56 s → 1.94 s.

**A view is copied by its fields only from a place written by fields (D-475).** ADR-106 copied
every view assigned whole by its two fields; that made MSVC stop interchanging `a05_structs`'s loop
nest (1.00x → 1.47x the C; clang 1.15x, gcc 1.28x), where the source is an iterator's view written
whole. Copied whole everywhere, `lines()` ran 1.98x the C with MSVC: its iterator's rest is written
by a slice field by field and read back for each item, and a whole read waits for both stores. So a
copy is by fields exactly when its function writes the source by fields (`parted_views`).

**`[CG-C-3]`'s `static inline` with gcc, measured.** With gcc, `a04_map_int` and
`a14_map_gap_1024` are 8% and 4% slower than before ADR-106 (still 0.78x and 0.75x the C at the
last run). The cause: ADR-106 made every function of at most 40 statements `static inline`
(`[CG-C-3]` (d)); gcc inlines a `static` function called once whatever its size
(`-finline-functions-called-once`), so `Map.index` moves into `main`'s lookup loop, which
`-funroll-loops` then unrolls 2 turns deep instead of 8 (the same C with that one option off:
40 ms against 47 ms; a branch hint on the slot width changes nothing). Two ways out were measured
on all 48 timed programs, each on the same emitted C: `-fno-inline-functions-called-once`
(geometric mean 0.995; the maps 0.87–0.92, but `a05_structs` 1.05, `s3_skip_step_take` 1.06,
`s4_copied_enumerate_skip` 1.05) and no `static inline` for gcc (0.994; the maps 0.93–0.96,
`a05_structs` 1.09, `a13_map_scrambled` 1.06). Neither gains overall; each moves time from some
programs to others, so `[CG-C-3]` stays as the specification writes it. clang and MSVC run these
programs as fast as before ADR-106 or faster (new/old 0.96–1.01).

**Measured against the compiler before ADR-106** (`f02ee12`, each program built by both and timed
interleaved): clang, `a16_map_text`, `w13_int_lanes`, `b15_checked`, `b16_checked`,
`s3_skip_step_take`, `a04_map_int`, `a14_map_gap_1024`: 0.97–1.01; MSVC, the same with
`a05_structs`: 0.96–1.00. The clang rows that looked slower in the previous README run (`a16`
0.91x → 1.06x the C, `w13_int_lanes` 0.89x → 1.03x, `b15`/`b16` 0.91–0.94x → 0.99x) were that
run's C references running slower (`a16`'s C++ 0.117 s then, 0.095 s now), not Ember.

## ADR-107 — for MSVC, a `for` loop's step through a standard iterator is copied into the loop

2026-10-02, the owner's pick ("option A") after ADR-106 left `split()` 1.13x the C with MSVC
(clang 0.94x on the same C). Every `for` over a standard iterator calls its `next`, which hands
back an `Option`; the loop then tests which variant came back. clang, once it inlines `next`,
sends each `return None` straight out of the loop and each `return Some(x)` straight into the
body, and drops the test. MSVC keeps building the `Option` and testing it, every item.

**Built** (MSVC only, after the MIR inliner: `inline_loop_steps_all` in `inline.rs`).
1. A `for` loop's call of `next` on a type of std's (`next(&mut it)`, `it` the loop's iterator)
   is replaced by a copy of `next`'s body; `next` itself stays (interface tables, other callers).
   The copy's reference to the iterator is forwarded to the iterator itself when it is used for
   nothing but reaching its fields, so the C never takes the iterator's address.
2. `thread_known_variants`: a block that only asks which variant a local holds
   (`d = discriminant(x)` and a switch on `d`), entered from a block that has just given `x` a
   variant (`x = Some(...)`, `x = None`, or a whole copy of a local just given one), is jumped
   over: the entering block goes to that variant's branch. With part 1, the loop no longer tests
   what the step gave back; inside the step, `find`'s result is no longer tested either. Not for a
   local whose address is taken, nor a `d` read anywhere but its switch.

**Result** (MSVC; Ember's time ÷ the C's): `split_whitespace()` 0.93x → 0.87x, `lines()` 1.11x →
1.09x, `split()` 1.13x → 1.12x. The milestone `msvc_loops_copy_a_standard_step_and_skip_its_answer`
fails with either part undone.

**What is left in `split()` with MSVC, measured, each by hand on the generated C.** The iterator's
fields as separate variables with the separator a constant: MSVC then keeps everything in registers
and searches for `','` directly, and the time does not change. The slices' checks deleted: no
change. `find`'s none tested as `== -1` instead of `< 0`: no change. A loop written by hand in
Ember's shape (a view of the rest moved past each part, `find`'s position with `-1` for none, a
finished flag) is 1.05x to 1.07x the C, whose loop takes the pointer `memchr` returns as it is; C's
own loop moves less than 1% with its code laid out differently, so the gap is not layout. So what
is left is the per-part cost of that shape with MSVC (clang runs the same C at 0.94x); no single
change measured here removes it.

## ADR-106 — text iteration at C speed

2026-10-02, after the owner's rule that every feature is made as fast as it can be, attended or
not (AUTOPILOT §4), and "you only stop investigating if the answer to the slowdown is overhead or a
safety check". `[TXT-10]`'s iterators were written in std as `Copy` views (ADR-105) and timed
against the loop a C programmer writes over a 3 MB text (`for c in text`, `chars`, `bytes`,
`char_indices`, `lines`, `split(",")`, `split_whitespace`), on the performance cores: 1.8x to 21.8x
the C. Each cause was found in the generated C and the compilers' machine code, and removed where
it lives: the compiler, the runtime or std.

**Causes and changes.**
* **Decoding.** The text loop called two runtime functions per character (decode, then the width
  again from the character), and MSVC compares the width out of the character a second time. One
  built-in now decodes and steps (`StrCharNext`, the runtime's inline `ember_str_char_next`), as
  C's loop does; std's iterators reach it as `text.next_char(at)`, a method only std can call.
* **`for` over the text iterators.** `for c in s.chars()` is the text loop, `for (i, c) in
  s.char_indices()` the text loop with each character's offset, and `for b in s.bytes()` a counted
  loop over the bytes, with no iterator object (`[CTL-3]`'s spirit, as for `range`, `enumerate` and
  an `Array`). Through `next`, MSVC kept a test of each item's `Option` (1.16x), and a byte loop
  could not group its overflow checks (`[SIMD-7]`). An iterator held in a variable, or an
  adapter's, still calls `next`.
* **`[CG-C-3]` in one translation unit.** The program is one C unit, so the functions the inline
  header would hold are its `static inline` functions: any of at most 40 statements with no
  foreign call, and every standard view's or container's `len`, `is_empty`, `as_span`, `iter` and
  `next`, which are forced inline (`EMBER_INLINED`), since a `for` loop calls `next` for every item
  and MSVC left `chars()`'s a call (1.4x). **`[CG-C-3a]` is built:** `@inline` forces inlining,
  `@noinline`, `@cold` and `@hot` reach the C compiler (they were `E0900`); the MIR inliner keeps a
  `@noinline` or `@cold` function a call. std's `is_split_space` is `@inline` (MSVC kept it a call:
  `split_whitespace` 1.5x), with the ASCII spaces as bits of one word and the rest a function of
  their own.
* **Values written by their fields.** MSVC builds a compound literal in memory with narrow stores
  and reads it back with one wider load, which waits for them: an `Option[(int, char)]` returned by
  an inlined `next` cost `char_indices()` 15x the C. A struct, tuple or payload variant assigned to
  a local is now written field by field, the tag first; a unit variant writes its tag only (D-360's
  zeroing took the value's address, which kept it in memory: `bytes()` 19x); a view a built-in makes
  (a slice, `as_bytes`) and an `Option` of a view's `None` take their pointer and length one at a
  time, and so is a view copied whole from such a place (every whole copy so cost `a05_structs`
  1.47x: D-475, ADR-108).
* **Searches.** `find`, `contains`, `count` and `replace` compared with `memcmp` at every byte;
  they now find the needle's first byte with `memchr` and compare the rest, and `find`,
  `contains`, `starts_with`, `ends_with` and the character-boundary test are inline (MSVC passes a
  view to a call through a copy it reads whole). `lines()` went from 7x to 1.1x the C.
* **Loops.** A `for` over an iterator with no `else` is `while true` with `None: break`: no flag
  tested each turn (`[CTL-4]`'s `else` keeps it). A counted loop pushing a literal onto a `String`
  on every turn has its room reserved first, as ADR-099 does for `Array.push`.
* **std.** `lines`, `split` and `split_whitespace` define `next` in their `implements Iterator`
  blocks (an inherent one broke the program's adapters: D-473).

**Result** (Ember's time ÷ the C's; MSVC, clang): `for c in s` 1.84x, 1.85x → 0.98x, 0.82x;
`chars()` 5.16x, 4.83x → 0.98x, 0.83x; `char_indices()` 21.8x, 5.37x → 1.00x, 0.93x; `bytes()` 3.40x, 2.66x → 1.00x, 0.97x;
`lines()` 7.54x, 4.75x → 1.12x, 1.05x; `split()` 6.54x, 4.54x → 1.12x, 0.95x; `split_whitespace()` 3.78x, 2.43x → 0.93x, 0.65x.
Measured on the performance cores, on mains power (`scratchpad/txtbench`, 11 runs, the median); the
README's tables get the seven programs with the next full benchmark run.

**What is left, and why.**
* `lines()`: its slices' bounds and character-boundary checks. With them deleted by hand it is
  1.04x (MSVC) and 0.99x (clang): safety checks the compiler cannot prove (a line ends where `find`
  found a `\n`).
* `split()` with MSVC, 1.13x (clang runs the same C at 0.94x). The cause first named here (the
  iterator kept in memory) was measured after this commit and is not it: with the iterator in
  registers nothing changes. About 6% was the loop testing what `next` gave back, which clang
  removes and MSVC does not; the owner chose to remove it for every loop over a standard iterator
  (ADR-107), which also measures what is left.
* Found on the way: `for x in span` binds each element by reference, and `[SIMD-5]`'s vectorisable
  form refuses a reference, so such a loop never groups its overflow checks (`for b in
  text.as_bytes()` 1.5x the C); `bytes()` is now a counted loop by value and is not affected.

## ADR-105 — a reference has a region slot of its own; a `Copy` view's `mut self` is a first-kind source

2026-10-02, D-470, after the owner's "start working on the language again". `[TXT-10]`'s iterators
(`chars`, `split`, `lines`) were written and taken out again: an iterator holding a `str` and giving
parts of it failed inside std's `Iterator` defaults, and keeping two of its items was `E3025`. The
region model gave a reference to a value with view fields no slot of its own: a borrow `&mut w` put
its loan, and what `w`'s fields borrow, into the same slots, so a view copied out through the
reference carried the loan of `w`, and so did every item `next` returned.

**Options priced.** (1) Treat what `Iterator.next` returns as never borrowing the iterator: shaped
to one interface and wrong for a lending iterator; rejected (general solutions only). (2) Give each
reference a slot of its own, the borrow of the place it refers to, beside the slots of the views
behind it, and route every fact by slot. General: it also covers `ref mut str` parameters and any
struct of views. Built.

**Built** (`regions.rs`, `borrows.rs`).
* `view_region_paths`: a reference's own slot first, then the slots of the views behind it
  (`Deref` paths); each slot knows whether it holds a mutable reference or span (`unique`).
* A borrow puts its loan, and what holds the borrowed place, in the reference's own slot
  (`holding_facts`: the innermost reference or view the place lies in, and each one outside it
  while the inner one is mutable, since a shared reference can be copied out of its holder and a
  mutable one cannot); each slot behind it gets what that view of the place borrows.
* A value read through a reference carries the facts of the views it reads (`value_regions`);
  liveness still keeps the reference (`place_regions`); an access is attributed to everything the
  place is reached through.
* `[LT-1]`: a result tied to a `mut` parameter of a `Copy` view type, which is passed by reference
  for its mode only (a source of the first kind), borrows what the view behind the reference
  borrows (`Elision::Named { through }`), as does a span iterator's next element. A non-`Copy` `mut`
  receiver is the caller's place (the second kind) and keeps the loan, so keeping two items of such
  an iterator stays `E3025`.
* A result summary field covers its own slots only (`field_destinations`); `[LT-22]`'s
  multi-region check counts a reference and the view behind it once (`independent_regions`).

**Result.** An iterator that is a `Copy` view gives items that outlive the next call; std's
defaults keep them. What must still fail fails: growing a list while a pair of references into it
lives (`BRW-5`), and a second reference through a held mutable reference
(`LT-20/reject_a_reference_through_a_held_mutable_reference_keeps_its_holder_borrowed`). Every
test passes; no benchmark program's C changes. `[TXT-10]`'s iterators can now be written as
`Copy` views.

## ADR-104 — an f-string reserves its room; integers format without `printf`

2026-10-02, the owner: "target all of them" (the programs more than 10% slower than the C). The
text-key map (`m[f"key{i}"]`: 200,000 keys, then a million lookups) ran 2.07x the C++ with MSVC
and 1.56x to 1.77x with clang (0.71x with gcc on Linux, whose `malloc` is quicker). Each lookup
formatted `i` with `snprintf`, allocated four bytes for `"key"`, reallocated when the digits did
not fit (`[HEAP-2]`: growth from four elements), and freed the text after the lookup; the C++ twin's
`std::to_string` uses a digit loop and keeps text this short inside the string. Priced by variants
of the generated C (one performance core, 15 runs):

| Variant | Time | vs C++ |
|---|---:|---:|
| As emitted | 178 ms | 2.07x |
| Integers formatted by a digit loop, not `snprintf` | 155 ms | 1.79x |
| and the key's text allocated once | 128 ms | 1.49x |
| and the lookup key built on the stack (no allocation) | 107 ms | 1.24x |
| The C++ twin | 86 ms | 1.00x |

**Built.**
* The runtime formats an `i64` or `u64`, printed or put in text, with its own digit loop
  (`decimal_u64`, `decimal_i64`): MSVC's `snprintf` parses its format and consults the locale on
  every call.
* `[LEX-19]` — an f-string all of whose pieces have a longest text (literal text; an integer, its
  digits and sign; a `bool`; a `char`, four bytes of UTF-8) reserves that room before it appends
  them (`fstring_room` in `lower.rs`, through the existing `ArrayReserve`): one allocation, where
  growing from four bytes took two. Any other piece, or a format spec, leaves it as it was.

**Result** (the README's run, on the performance cores): with MSVC 2.09x → 1.51x, with clang
1.77x → 1.14x, with gcc 0.71x → 0.43x. Its times move from batch to batch (the C++ twin took 111 to
116 ms in the 11-run pass and 81 ms in the 21-run re-run, where Ember with clang was 0.91x and
1.14x). `STD-20/accept_every_integer_prints_its_digits` and
`LEX-19/accept_an_fstring_of_bounded_pieces_reserves_its_room` (its `assert-c` fails without the
reservation; the first fails with the digit loop broken). **Not built, each the owner's call:** the
lookup key built on the stack (it never leaves the call it is lent to, but a `String` whose buffer is
on the stack must never be freed or grown, which the compiler would have to prove); a small-block
allocator in front of `malloc` (every small allocation gains; a change to how memory is allocated);
short text kept inside the `String` (C++'s short-string optimisation; a representation change).
What is left with all three is the map's layout: the key's text is one memory access further away
than in the C++ twin's node.

## ADR-103 — MSVC: a small checked loop several turns to a pass; a call takes an address directly

2026-10-02, the same request. The three `mut self` benchmarks (`t = things[i % 2]; t.bump()`, a
hundred million times) ran 1.30x, 1.35x and 1.31x the C++ with MSVC, 1.00x with clang and gcc.

**Measured** (pinned to one performance core, 21 to 31 runs; `scratchpad/msvc_slow`, `unroll`).
* MSVC unrolls no loop that can leave other than by its test, and the overflow check on
  `count += 1` (inlined from `bump`) is such an exit, so Ember's loop ran one turn per pass. MSVC
  unrolls the check-free C++ twin five times: a divisor of its constant count (with a count of
  100,000,001, or one known only at run time, it does not unroll the twin at all).
* Run to run, Ember's loop took 35 to 65 ms where the twin held 31 ms: one add instruction updated
  the two objects in turn, and the processor mispredicted which earlier store each load read.
  When a pass has at least as many turns as there are objects in the rotation, each instruction
  keeps to one object's chain: 0.81x, steady. With 2, 3, 4, 5 and 7 objects in rotation, eight
  turns to a pass were at or under the C++ every time; four turns were not at five (1.14x).
* Grouping the checks (`[SIMD-7]`'s MAY) instead: 2.5x. MSVC chains the flag through every turn and
  still does not unroll. Rejected.

**Built** (`ember_codegen_c`, for MSVC only; clang's and gcc's C are unchanged on all 45 benchmark
programs).
* `for_loops(body, inlined)`: a block of a counted loop's body may end in a call MSVC inlines (to a
  function of the program within `[CG-C-3]`'s 40 statements counting what it calls,
  `inline_sizes`), so such a loop is a C `for` loop too (`emit_for_body`, `fall_into`); a call into
  the runtime, through a table or a value, or to a larger function costs more than the loop, and
  such a loop stays as blocks. For a loop over 1,000 objects in turn, 2.16x → 1.44x the C++.
* `unrolled_turns`: a body with a check or such a call is written in passes of the most turns, a
  power of two up to 8, whose statements, the inlined callees' counted, stay within 200 (gcc's
  `-funroll-loops` limits, `max-unroll-times` and `max-unrolled-insns`, which Ember's gcc build
  already asks for, ADR-099), then the turns left one at a time; each turn keeps its own checks, in
  order, so nothing a program can see changes. Not for a loop carrying a scalar from turn to turn (a
  running total), where each turn waits for the one before.
* `folded_refs`: a reference temporary set by the statement before a call, and read by that call
  only, is written into the call (`bump(&_4)`). Unrolled, MSVC stopped inlining `bump` while it was
  called through the temporary (2.5x the C++); with the address written in, 0.82x.

**Result**, MSVC, the README's programs (one performance core): the three `mut self` programs
0.82x each; programs more than 10% slower than the C, 5 → 2. Of the two, the list sum (`p1`) is the
overflow check on its running total, measured: with `@overflow(wrap)` the same program is 0.97x the
C. It first went 1.47x → 1.66x when its leftover running-total loop was unrolled, an effect of where
its hot loop landed (removing cold code moved it between 32 and 40 ms); running totals are left
alone and it is back to 1.47x. Blocks of up to 1,024 additions with each value under 2^51 (the same
proof as `[SIMD-7]`'s 64 under 2^55) would make it 1.30x; that changes the spec's numbers, so it is
the owner's call. `msvc_writes_small_checked_loops_several_turns_a_pass` (milestones) and
`CTL-3/accept_a_counted_loop_runs_every_turn_in_any_pass`: the first fails with any of the four
parts undone (the passes, the addresses, running totals left alone, MSVC only). Six conformance
tests counted emitted text an unrolled loop repeats (`for (;`, a view pointer's reads, a reference
temporary's assignment); each now counts what is the same for every compiler (a loop's one-turn
header `; ++_`, where each view pointer is set, the address itself).

## ADR-102 — an access begun on only some paths keeps a flag; the verifier follows it

2026-10-02, with D-455 (the second review's G5-4). `[EXC-18]` holds a class field's access while a
loan of it lives *on the path taken*. A view made in one branch and used after the join is a loan
only on that branch, but its region (where the view may still be used) covers the join on both
paths, so ending the access where the region stops being live ended it on paths that never began it.

**Options priced.** (1) Begin the access on the other paths too, so every path holds it at the
join: no run-time state, but a write to the field through another handle on a path where the view
is of something else would panic, so a legal program fails; rejected. (2) Duplicate the code after
the join for each path until the region ends: exact, but the code grows with every such view;
rejected. (3) A flag per such access: exact, and the cost is one store at function entry, one as
the access begins and a test at each end, in functions that make a view on some paths only.

**Built** (`borrows.rs`, `verify.rs`). Each access's placement is planned (`plan_access`) and checked
by a forward pass over the original blocks (`plan_is_exact`): every end must meet the access begun,
every begin meet it ended, every block be entered with it in one state. An access that fails keeps
a `bool` temporary named `access-open` (`ember_mir::ACCESS_FLAG_NAME`, a name no Ember identifier
can have): cleared on entry, set right after the access begins, and each end becomes a branch on it
to a block that ends the access and clears it. An end inside a block splits the block there; an end
on an edge gets its two blocks on that edge. Everything is ordinary MIR, so every later pass
handles it unchanged. The MIR verifier keeps the states reaching a block apart by the values of
these flags (at most 256), follows a branch on a known flag one way, and otherwise checks as before:
an end must meet its access open and a return must meet none open. `try_borrow` of a `RefCell` class
field, whose guard is made on the success path only, is the commonest case. No benchmark program's C
changes.

## ADR-101 — range facts: a moving bound widens to the loop's own constants first; a call that returns one of its arguments keeps their range

2026-10-02, the owner's go ("add this feature and see whether it works if it doesn't use your
autonomy to remove it"). With gcc the generic `larger` benchmark ran 1.76x its C++ twin:

```ember
for i in 0..200000000:
    best = larger(best, (i * 7919) % 100003)
    total = total + best
```

The whole gap was the overflow check on `total` (gcc computed each turn twice around it; with
`#! module overflow(wrap)` the program ran as fast as the C++). `[RNG-4]`'s range facts could not
remove it, for two reasons:

* `best` only grows, and widening moved a growing bound straight to the end of its type, so
  nothing bounded what `total` adds each turn.
* `best`'s new value comes from a call, and a call's result had no range unless the callee was a
  built-in the analysis knows.

**Built** (`compiler/ember_analysis/src/range_facts.rs`):

* **Widening with thresholds.** At a loop's header, a bound that moved goes first to the nearest
  integer constant written in that loop, or one either side of one (at most 64 of them, those
  nearest zero), and to its type's end only past the last. `best`'s upper bound stops at 100,002
  (one below the `% 100003`), and the next round confirms it. If a body's facts do not settle
  within the rounds allowed this way, it is solved again without thresholds, so no body loses a
  fact it had. Widening with thresholds is the standard refinement of interval widening (Blanchet
  et al., "A static analyzer for large safety-critical software", PLDI 2003, the Astrée analyser).
* **Returned arguments.** Each function is summarised first: if every value it returns is one of
  its parameters, unchanged (its return place is only ever a copy of a parameter and is never lent
  mutably, and no parameter is written, lent mutably or given a call's or a checked operation's
  result), the summary lists those parameters. At a direct call to it, the result's range is the
  union of those arguments' ranges. `larger(a, b)` returns `a` or `b`, so `best`'s new value lies
  within `best`'s range or the remainder's. Bodies that share a symbol must agree, or the symbol
  has no summary.
* With `best` between 0 and 100,002 and the loop counted (200,000,000 turns), the running-total
  rule (`accumulator_bounds`) bounds `total` below 200,000,000 × 100,002, which fits an `int`: its
  check goes. The loop versioning pass (`loop_version.rs`) runs the same analysis and gets both.

**Measured** (median of 21; Ember's time over the C++ twin's, each built by the same compiler):

| Compiler | before | after |
|---|---|---|
| gcc 15.2 (WSL) | 0.177 s, 1.76x | 0.099 s, 0.98x |
| MSVC | 0.117 s, 0.94x | 0.117 s, 0.93x |
| clang | 0.110 s, 0.99x | 0.108 s, 1.00x |

Of the 45 Windows benchmark programs, built by the previous commit's compiler and by this one with
MSVC and with clang, only this program's C changed. The analysis runs before every
compiler-specific pass, so the same holds for gcc. Compile time (`ember build --emit c`, median of
7): 1% to 3% on the benchmark programs, within noise; 12% (0.210 s to 0.234 s) on
`GRM-39/accept_256_levels_of_nesting`, whose 256 nested calls of `id(x)` now each carry a range.

**Tests.** `RNG-4/accept_a_value_a_loop_raises_towards_a_limit_keeps_it` (the loop written out: no
overflow check left in the C; fails with thresholds off),
`RNG-4/accept_a_call_returning_one_of_its_arguments_keeps_their_range` (the same through a generic
`larger` called from two places, so the C compiler does not inline it away first; fails with
thresholds off and with the call rule off), and
`RNG-4/reject_a_value_doubled_past_every_constant_keeps_its_check` (a value doubled every turn
passes every constant of its loop: its check stays and fires). The quick check, the full suite with
MSVC and with clang, and the conformance suite with gcc pass.

## ADR-100 — `Map`: a number key at its own remainder, a prime-length table, the multiply only when keys pile up; 4-byte entry numbers

2026-10-01/02, the owner's go. With gcc the `Map` benchmark (a million keys `i * 7`, then five
million lookups in the same order) ran 2.02x its C++ twin. `DefaultHasher` multiplied every key by
FxHash's constant and the table took a key's place from the product's high bits (ADR-047), so keys
next to each other landed about 3.8 MB apart and every lookup waited on memory; libstdc++'s
`unordered_map` uses an integer as its own hash and a prime number of buckets, so the same keys sit
in neighbouring buckets and are read in order.

**Built** (`std/src/collections.em`):
* `DefaultHasher.mix` spreads the state by the multiply before each word is xored in: a one-word key
  (any integer) hashes to itself, a key of several words still mixes them all. Still one multiply,
  one rotate and one xor per word (`[HASH-2]`); its values change, which `[HASH-2]` allows between
  versions.
* The table is a prime number of places long (`prime_at_most`, the largest prime at most the power
  of two the map would have used, so the seven-eighths rule and the growth are unchanged), and a
  key's home is its hash's remainder by that length: keys close together get neighbouring places,
  and keys a power of two apart do not share one.
* The one pattern that piles up under a remainder is keys a multiple of the table's length apart.
  When storing an entry, in `push_new` or in `rebuild` (a table can reach that length inside a
  rebuild), walks past more than 100 used places, the map switches for good (`use_multiply`) to
  ADR-047's placement, the hash times FxHash's constant and its high bits, in a table a power of two
  long, and rebuilds. 20,000 keys `i * 32749` (the length of their table): 0.75 s before the switch
  existed, 0.0015 s with it (ADR-047's map 0.0013 s).
* A place holds a 4-byte entry number (`slots: Array[i32]`) while the table is at most 2^31 − 1
  long; a rebuild to a longer table uses 8-byte ones (`wide_slots`, `wide`). `find` and `place`
  choose the list once and walk it, and the table's length is read once per operation, so the
  choice costs 0% to 4% against a 4-byte-only table.

**Measured** (median of 11; Ember's time over the C++ twin's, each built by the same compiler):

| Program | gcc before → after | MSVC after | clang after |
|---|---|---|---|
| keys `i * 7` in order (the benchmark) | 2.02x → 0.67x | 0.33x | 0.20x |
| scrambled keys | 0.77x → 0.94x | 0.58x | 0.50x |
| keys 1,024 apart | 1.34x → 0.76x | 0.40x | 0.34x |
| keys 1,048,576 apart | 1.09x → 0.60x | 0.40x | 0.28x |
| text keys | 0.67x → 0.80x | 1.71x | 1.56x |

The remainder is a division where the multiply and shift were not, so scrambled keys (no
neighbours to gain from) are about 10% slower with gcc than before; MSVC and clang measured them
even. Text keys with MSVC and clang were as slow before; that gap is not the placement and is not
looked into yet. The four new rows are README benchmarks (`a13` to `a16`).

**Tests.** `STD-11/accept_a_map_whose_keys_share_a_place_switches_its_placement` (a map of keys
`i * 7` keeps a prime table, capacity 28,655; one of keys `i * 32749` switches to a power-of-two
table, capacity 28,672; every key found; fails with the switch disabled). The 8-byte path cannot be
reached on this machine (a table past 2^31 places): the whole conformance, run-pass and std suites
pass with gcc against a std whose switch to 8-byte numbers happens at 64 places, so every larger map
in them ran on the 8-byte list. The quick check, the full suite with MSVC and with clang, and the
conformance suite with gcc pass.

## ADR-099 — gcc unrolls loops; a list a loop only pushes onto lives in a local; an interface call's fallback search is out of line

2026-10-01, the owner's choice ("turn it on, option A"), after the gcc comparison of ADR-098. The
gcc build of the benchmarks against their twins built by the same gcc with the same flags (median
of 11 or 21 interleaved runs, WSL Ubuntu, gcc 15.2).

**`-funroll-loops` for gcc's release build** (`gcc_flags`; the twins get it too). gcc unrolls no
loop at `-O2`; clang does. A loop whose turn adds one to an object's field under the overflow check
waits on the `jo` of each add before the next turn's add to the same field can retire (measured by
hand-written variants of Ember's C: the check written into the field, into a local, as a compare
with `MAX - 1`, and the add-to-memory form clang emits, all 0.060 to 0.067 s; no check 0.029 s, the
C++ 0.030 s). clang unrolls by two, so each add instruction always meets the same object: at C's
speed. With unrolling, gcc does the same: the three `mut self` programs 1.98x, 1.91x, 1.91x before,
0.98x, 0.99x, 0.98x after; the checked sum 1.31x to 1.04x. Two programs fall behind the C instead:
the interface calls (1.00x to 1.18x) and the million objects (0.89x to 1.14x). Ember is no slower
in either; the twin gains more. The interface-call gap is the object header (`[OBJ-1]`'s 24 bytes):
the C++ shapes padded by 16 bytes to Ember's 32 run at Ember's speed (0.99x). The million-object
gap is half that and half the running total's overflow check (1.06x with `overflow(wrap)`). The
owner chose the trade (option A) over keeping it off. Shipping (`-O3`) is unchanged: not measured.

**For clang and gcc, a list an innermost loop only pushes onto is kept in a local for the loop**
(`list_locals.rs`).
The C compiler cannot tell a list's buffer from its header when the runtime made both, so after
each pushed element it read the length and the buffer from memory again (the C twin's buffer comes
from `realloc`, which it knows is memory of its own). The pass reads the list into a local before
the loop and writes it back on every edge out; nothing else in the loop may reach the list (no call
but the built-ins, no other use of it or of what holds it, no handle of any class mentioned when it
is in an object, no reference that could point at it when it is a local), and each push's reference
gets its value in the loop. A panic leaves the list as the loop found it, which nothing sees
(`[PAN-1]`: a panic aborts). The push's growth (`ember_vec_grown`, cold) now takes and returns the
list by value, so the local's address never escapes and the C compiler keeps it in registers. The
view-held benchmark: 1.03x the C before, 0.47x after (0.39x unrolled); the three views-alive
programs 1.01x to 1.09x before, 0.38x after; the shape priced by hand first (0.42x); with clang
0.50x and 0.44x. Not for MSVC (the driver's `c_for_msvc`): MSVC keeps the local list in memory, and
the views-alive program ran 0.386 s with the pass and the by-value growth, 0.058 s with the pass and
the old growth, 0.050 s without the pass (either growth), the C twin 0.050 s.

**An interface call's fallback search is the runtime's out-of-line one** behind the tests of the
classes the program knows (`[DSP-3]`), and the out-of-line search is `EMBER_COLD`: the inline search
is a loop, and a loop inside the caller's loop kept gcc from unrolling the caller's (it unrolled the
never-taken search eight times instead); cold, gcc lays the known classes' calls on the straight
path. The interface calls unrolled: 1.24x before, 1.14x after.

**Tests.** `gcc_release_asks_for_the_cheap_vectoriser_model_and_unrolling`,
`a_list_a_loop_only_pushes_onto_is_kept_in_a_local`, `a_devirtualised_interface_call_searches_out_of_line`
(each fails with its change undone), and `OPT-2/accept_a_list_a_loop_only_pushes_onto_keeps_every_push`
(a list in an object left by `break` and by the loop's end, two lists in one loop, a loop where
another handle reads the list and one where a function does, a versioned loop, pushed tuples; it
fails when the pass ignores what else reaches the list). `CTL-3b`'s `for`-`else` caught a wrong
write-back numbering before it shipped.

## ADR-098 — gcc: `restrict` loop functions, and the vectoriser model that takes a tail

2026-10-01, autonomous, the owner's request: "build the gcc version of ember on wsl and test how it
runs in comparison to gcc C and g++ C code and try to optimise that as well". The 39 benchmark
programs, built with gcc 15.2 under WSL (Ubuntu) on the owner's machine, each against its C or C++
twin built by the same gcc with the same optimisation flags (median of 11 interleaved runs). Before:
the list loops (adding one list into another, changing every number using a second list, the decimal
lists, adding two lists plus the round) 1.52x to 1.72x the C, `enumerate` and `zip` 1.44x and 1.38x,
the checked sum 2.52x.

**Two causes, each measured by editing Ember's C by hand first.**

* gcc vectorises a loop over two lists it cannot prove apart only with a run-time overlap test, which
  its `-O2` cost model never adds; and it takes `restrict` from a function's parameters, not from a
  block's pointers read from a list's header (adding one list into another: 0.071 s, against the
  C twin's 0.042 s, whose lists are `restrict` from `malloc`; `restrict` block pointers in Ember's C,
  0.071 s; the loop in a function with `restrict` list parameters, inlinable or not, 0.042 s).
* At `-O2` gcc's vectoriser takes only a loop whose vector code leaves no tail, so a loop over a
  list's elements, its count known only at run time, stays scalar, where the twin's count is a
  constant (`enumerate`: 1.45x at gcc's default, 0.99x with `-fvect-cost-model=cheap`, both sides
  built alike). clang's and MSVC's `-O2` vectorise such a loop with a scalar tail.

**Built.** ADR-079's pass (`kernels.rs`) takes a target. For gcc it moves any outermost counted loop
over two lists or more that writes one into a function of its own whose list parameters are
`restrict` (`KernelTarget::Gcc`): gcc does not turn loops around, so MSVC's conditions on a nest
reading back what it writes, and ADR-084's running value, are MSVC's only. Lists are separate by
the same proof as ADR-079's. The driver asks for it when the C compiler is gcc (`c_for_gcc`, as
`c_for_msvc`). And gcc's release build adds `-fvect-cost-model=cheap` (`gcc_flags`), which allows the
tail but no run-time overlap test; `-O3` (the shipping profile) already has a model at least as
permissive. The benchmark twins are built with the same flag.

**After** (gcc, same flags both sides): the eleven list-loop rows 0.97x to 1.04x, `enumerate`
0.97x, `zip` 1.00x, the checked sum 1.31x, copying a list one item at a time 1.04x; no row slower.
What remains is in `docs/HANDOFF.md`: the rows where the overflow check the language requires is the
whole of the gap (measured with `#! module overflow(wrap)`, which brings each to 0.96x-1.00x), and
the `Map` row.

**Tests.** `loops_over_separate_lists_get_restrict_functions_for_gcc` (every loop of the
separate-lists program moves for gcc, three of four nests reading back their list, one for MSVC;
fails with the gcc target off), `gcc_release_asks_for_the_cheap_vectoriser_model` (`ember_build`).
The conformance suite passes with gcc 15.2 in WSL.

## ADR-097 — ODR-094 built: `collect`, `peekable`, `flat_map`, `flatten`, `join`, and printing `Box` and `Cell`

2026-10-01, autonomous: the owner's ruling (ODR-094, Hardened_51), and the rest of `[STD-19]` D-407
had blocked.

**`collect`.** `std.core.FromIterator[T]` (not a prelude name, as ODR-091's interfaces are not) has
one function, `from_iter[I: Iterator[Item = T]](owned it: I) -> Self`; `Iterator.collect[C:
FromIterator[Item]](owned self) -> C` is `C.from_iter(self)`, so every target is written in the
library, a program's type included (`AUTOPILOT` §4: the meaning lives in the file). `C` comes from
`[C]` or from the expected type (D-407 (5)'s hint). `Array`, `Set`, `Map` (pairs, `insert`'s
replacement) and `String` (from `char`s and from `String`s, two instances of one interface, which
needed D-440) implement it.

**`peekable`.** `struct Peekable[I: Iterator]: inner: I, peeked: Option[I.Item], looked: bool`: one
`Option`, not `Option[Option[Item]]`, with `looked` saying whether `peeked` holds the next item or
the end. `peek` and `peek_mut` fill it once and return `peeked.as_ref()`/`as_mut()` (the owner's B2:
`Option.as_ref`/`as_mut` are the compiler's, `peek` is Ember); `next_if` and `next_if_eq` go through
`peek`. `next_if_eq` is in a block `extend[T: Eq, I: Iterator[Item = T]] Peekable[I]`, which D-438
lets apply.

**`flat_map`, `flatten`, `join`.** `FlatMap[I, U: IntoIterator, F]` holds the current inner
iterator as `Option[U.Iter]` (D-407's hidden parameter) and reaches it through `as_mut`;
`Flatten[I, U: IntoIterator]` names the item type `U` so its bound can be stated (a struct has no
`where`), `I: Iterator[Item = U]` in its extension. `join(sep) where Item: Display` formats each
item (D-407 (5)); a collection's own `join` goes first, even where its bound is missed (the iterator's
is not tried then: `[STD-19]`).

**Printing `Box` and `Cell`.** `TypeTable::printed_as` gives the type a compiler-known wrapper
prints as (`Box[T]` but `Box[dyn I]`, `Cell[T]`); the checker's `has_display`, `formattable_in` and
`fstring_spec` ask it of the payload, and the C backend's `printed_payload` turns the value into its
payload's place (`*v`, `v.value`) in `debug_stmt` and in the print and format calls, a `String`
payload as its text.

**Not built here:** a `for` over `flat_map`, `flatten` or `peekable` runs their `next`, not a fused
counted loop (`[CTL-3]`'s "one loop with no iterator object"); `str.chars()`, which `[STD-15]` lists,
does not exist yet. Both are on the handoff's list.

**Tests.** `STD-19/accept_collect_builds_any_from_iterator_target`,
`STD-19/accept_peekable_peeks_and_takes_conditionally`, `STD-19/accept_flat_map_flatten_and_join`,
`TYP-39/accept_box_and_cell_print_as_what_they_hold`, `ERR-4/accept_as_ref_and_as_mut_reach_the_payload_where_it_is`,
and the defects' (D-438 to D-444). `STD-9/reject_printing_a_value_with_no_printer` now uses a
`RefCell`, as a `Cell` prints.

## ADR-096 — a generic interface method's own parameters never share the caller's slots

2026-10-01, autonomous (D-437). A method's own type parameters were numbered from 0 wherever an
interface's method was read for a use, the slots the enclosing generic body's own parameters hold.
Two readings collided: an instance of a generic interface over the caller's parameters
(`Gather[T]` in `fn build[T, C: Gather[T], I: …]`) gave `gather`'s own `I` the slot of the caller's
`T`, and a call through a type parameter's bound read the declaration as written, `Self` included.

**The rule, one for every reading: the caller's parameters keep their slots, and a method's own
come after them.** D-280 already made it for a generic type's methods over an opaque owner
(`generic_prefix`: the caller's parameters the signature mentions, solved to themselves, then the
method's own). Now:

* `instantiate_interface` numbers each generic method's own parameters after the highest caller's
  parameter the instance's arguments mention, and records the mentioned ones as the declaration's
  prefix. `implementation_signature_matches` and `register_instantiated_default` follow that
  prefix, so an implementation that names the extension's parameter
  (`fn fill[I: Iterator[Item = T]](…, extra: T)` in `extend[T: Copy] Array[T] implements Fill[T]`)
  matches, and a method's own projection (`U.Iter`) moves with its base.
* A call through a bound is checked against `bound_call_signature`: the declaration with `Self` as
  the type parameter, its associated types as the parameter's through the bound (D-417), and the
  method's own parameters after every one of the caller's. `it.zip(other)` is a `Zip[I, J]`,
  `M.make(3)` an `M`. The reading is a signature of its own, kept per method, parameter, bound and
  caller; like every call through an opaque bound it is checked and never emitted, as the instance
  rechecks the body against the implementation.

**Rejected:** substituting `Self` at the call only: the method's own `I` and
the caller's `T` still shared slot 0, so a bound's binding read one as the other; renaming the
method's parameters by name: the solver works by slot.

**Tests.** The three `IFC-4` programs D-437 names, each failing without its part.

## ADR-095 — projections in generic types, interfaces' associated types first

2026-10-01, the owner's design for D-407 (2)-(4) and (6), built autonomously. `[STD-19]`'s
`flat_map`, `flatten`, `peekable`, `collect` and `join` need a generic type to hold an associated
type of its parameter (`current: Option[U.Iter]`), an interface's method to name one of an
interface declared after it (`Iterator.flat_map`'s `U.Iter`), chains (`Item.Iter`), and a reference
into a held `Option` (`peek`).

**Hidden parameters for a generic struct.** As D-380 does for an extension, each `U.Name` a
generic struct's fields or method signatures mention is one hidden parameter after the written
ones, bounded as `Name` is declared (with the bindings its bounds write). Where a type is made the
hidden ones are filled from the arguments, through the bound that declares the name (D-417); an
opaque instance passes them with its own. Instances of such a struct are named by the hidden
arguments too: an opaque `Each[U]` means a different `U.Iter` in each generic body that makes one,
and a concrete one's are fixed by its arguments, so the name stays a function of the type. A
signature or an extension that names `Each[U]` over its own `U` declares the same hidden parameters,
so its signature and its body meet one instance. The written parameters alone are what a program
writes and what a type shows (`Each[Array[int]]`).

**Interfaces' associated types first.** Generic types are collected before interfaces (an
interface's methods name generic types), so before either, each interface's associated type names
and its parents are noted; a projection through an interface not collected yet is declared with no
bounds, closed once interfaces are. A method's own projections are declared in its signature, as a
function's are, and filled at its call. Chains are declared in order, each base a parameter or a
projection declared before it; a body may project a projection.

**An associated type's bound with a binding.** `type Iter: Iterator[Item = Item]` named no bound
at all (the binding made it no plain name), so `u.into_iter().next()` failed in every generic body.
The bound is kept, and its binding read with the interface's own names abstract; a hidden parameter
for `U.Iter` takes it with `Self.Item` as `U.Item`.

**`Option.as_ref` and `as_mut`** (the owner's choice over a by-reference pattern or a built-in
`peek`) are the compiler's: a match on the place binding the payload by reference, so a `Copy`
payload is referenced where `[GRM-13]` would copy it; `as_mut` is the write `ref mut` in a pattern
is (D-295).

## ADR-094 — a label in another file is shown under that file's own header

2026-10-01, autonomous; D-411, found by the 2026-10-01 agent review (G1-4, G1-N2). XIX §6's
human format shows a diagnostic's labels under one `-->` header, for the primary's file, and the
renderer dropped every label in another file: `E2041` across two modules named one implementation,
and a program's `extend int implements Default` pointed only into `std/src/core.em`. `--json`
carried both. The spec shows the one-file case only.

**Each file its own group.** The primary's file comes first, under `-->` at the primary's place;
each other file follows, in the order its first label comes, under `:::` at its first label's
place (rustc's form), its labels drawn as in the first group. One gutter width serves all groups.

## ADR-093 — a callable argument meets a callable bound as a call's arguments meet parameters

2026-10-01, autonomous; D-423, found by the 2026-10-01 agent review (G3-2). `unify_generic_argument`
solved a callable bound's parameters from the argument's signature and checked nothing else, so
`apply[F: fn(int) -> int](fn(v: str) => 1, 2)` was reported inside `apply`, where `f(x)` passes an
`int` to a `str`, and `xs.iter().map(takes_str)` inside `Map.next` in `std/src/core.em`.

**The check.** With what the call has solved put into the bound, each parameter type the callee
passes must reach the argument's parameter as a call's argument reaches a parameter: the same type,
a widening, a borrow (`[TYP-5]` rule 7: a lambda written `fn(v: ref int)` for `fn(int)`), or a
`Copy` value read through a `ref` (a lambda written `fn(v: int)` where the items are `ref int`).
The argument's result must reach the bound's the same way. Anything still open, or already an
error, is not compared. A mismatch is `E2020` at the argument, naming both signatures, and the call
is not instantiated around it. `[FN-6a]`'s exact equality of `fn` types was not taken: programs the
corpus has always accepted (`CTL-3b`) pass a borrowing lambda for a by-value bound, and the
instance's body, checked with the real signature, applies exactly those coercions.

## ADR-092 — an instance is named by its arguments' structure, once

2026-10-01, autonomous; D-412, found by the 2026-10-01 agent review (G1-5). An instance of a generic
type, of a generic interface or of a compiler-known wrapper is a nominal type kept in `named_types`
under a name, and the name is also its C name. It was the constructor and each argument's symbol
name with every run of punctuation made `_` (`type_stem`), so `Wrap[*int]` and `Wrap[int]` were
both `Wrap_i64`, `Pair[(int, int), int]` and `Pair[int, (int, int)]` both `Pair_i64_i64_i64`, and
whichever was made first was the other as well: the second took the first's fields.

**Two layers, one function.** `unique_instance_name(constructor, args)` names every instance:

1. **The spelling keeps the structure** (`instance_stem`): a pointer is `ptr_T` (`ptr_mut_T`), a
   tuple `tupN_A_B`, a fixed array `arrN_T`, a function type `fnN` with its parameters' modes, its
   ABI and its return (`fn1_i64_i64`). Everything `type_stem` already told apart is spelled as it
   was (`i64`, a declared type, another instance, `Array_T`, `Span_T`, `ref_T`, `ref_mut_T`), so
   the names programs and tests see do not move. The spelling is a function of the type alone, the
   same in every compilation, which a name in a C header or a library needs.
2. **A name is never shared.** The name is recorded against its (constructor, arguments); asked
   again, the same name comes back. A spelling already held by another instance, a declared type
   or an interface (a program may declare `struct Wrap_i64`, or `struct ptr_i64`) gets `_2`, `_3`,
   … after it. So identity never rests on a spelling, even where a declared name mimics one.

The alternative of a hash of the full symbol name on every instance was priced and not taken: it
changes every instance's C name (tests pin some), reads worse, and still needs the second layer for
exactness. Prefixes are kept (`Option_`, `Cell_`, `RefCell_`), which `ember_types` and the C
backend test.

## ADR-091 — the closure adapters, and their stages in a counted loop

2026-10-01, autonomous. `[STD-19]` lists `map`, `filter`, `filter_map`, `take_while`, `skip_while`
and `inspect`; none existed, as they need an adapter that holds a callable, and the handoff had them
waiting on `[CLO-3]`'s owned callable values.

**They do not need owned callable values.** `[STD-19]` says adapters "allocate nothing themselves",
and an owned callable value may allocate (`[CLO-10]`). So each adapter holds the callable by value,
as a type parameter bounded by the callable type: `fn map[R, F: fn(Item) -> R](owned self, f: F) ->
Map[Self, R, F]`, `struct Map[I, R, F]: inner: I; f: F`. A lambda's value is its environment, so the
adapter is monomorphised per lambda, each call direct, and a lambda that borrows makes the adapter
a view (`[TYP-15]`). This is `[CLO-14]`'s explicit bound, which was not built: D-406. The bound's
`I.Item` is a projection read once every parameter is declared, as an interface bound's bindings
are.

**In a `for` (`[CTL-3b]`).** A chain is now links (the counted ones: `take`, `skip`, `step_by`,
`enumerate`, `zip`, `copied`, `rev`) under stages (these six). The stages must all be above every
link: under `skip`, `step_by` or `zip` a stage would run for items the link leaves out, which the
counted loop never computes, so such a chain runs through `next` (a `take`, `enumerate` or `rev`
over a `map` could be fused exactly; not done). Each stage's callable is evaluated once, after the
links' iterators and counts, in call order, into a hidden local (a function, or a lambda that
captures nothing, is called by name instead, so the call is direct under every C compiler). Each
turn materialises the links' item and runs the stages on it as their `next` would: `map` makes the
next item, `inspect` calls, `filter` goes on to the next turn, `filter_map`'s rest of the turn is its
`Some` arm, `skip_while` keeps whether it has started, and `take_while` ends the loop; a loop with an
`else` then runs it (the loop did run to its end) without the links' exhaustion checks (nothing
below was exhausted), and a `break` still skips it. A tuple item binds its parts by field. The
calls are checked as source calls on the hidden names, so modes and borrows are the adapter's:
`inspect(fn(v) => seen.push(v))` holds `seen` for the loop, and reading it in the body is `E3021`.

**Measured** (10 million `int`s, 60 rounds, median of 11). `for y in xs.iter().map(fn(x) => x * 3 +
1): total += y`: through `next` 3.063 s with MSVC and 0.422 s with clang; fused 0.363 s and 0.371 s,
against the same loop written by hand in Ember 0.379 s and 0.390 s, and C 0.299 s and 0.382 s (the
difference from C is the overflow checks, as for the hand-written loop). `filter` (`x % 2 == 0`):
through `next` 0.776 s and 0.372 s; fused 0.472-0.512 s and 0.433 s, by hand 0.431 s and 0.451 s, C
0.264 s and 0.324 s. The C of the fused and hand-written `filter` loops differs by a copied pointer
and a negated `bool`; moving the inlined predicate's block beside its caller changed nothing
(0.518 s against 0.510 s), so MSVC's remaining 10-19 % there is not the layout.

**Not built.** The stages are not double-ended (ODR-091 lists the adapters that run backwards, and
these did not exist; a ruling would add them); `flat_map`, `flatten` and `peekable`.

**Tests.** `STD-19/accept_the_closure_adapters` (each adapter through consumers and the `Iterable`
form), `CTL-3b/accept_closure_adapters_are_stages_of_a_counted_loop` (each stage in a `for`, an
`enumerate` pair through a `filter`, a `take_while` `else`, a `break` past a mapped `rev().skip()`;
no `next` in the C), `CLO-14/accept_an_explicit_callable_bound_is_called_like_a_function` and
`CLO-14/reject_a_bound_that_is_no_interface_or_callable` (D-406).


## ADR-109 — read-only element references in vectorisable loops

2026-10-03, autopilot resumed by the owner. D-477; `[SIMD-5]`, `[SIMD-7]`.

**Context.** Array and Span iteration binds an element by reference, but the
vectorisable-form analysis refused all reference creation and dereferences.
It therefore missed the existing running-total width and block proofs for
ordinary iteration, including `for b in text.as_bytes()`.

**Decision.** Preserve the reference in MIR and C. Trace a read-only local's
dominating, single definition to its indexed source in `vf_place`, following
copies too, with the same bounded walk used by the affine-index proof. Admit
only references to the indexed accesses that analysis already accepts. A
reference carried from the previous iteration remains excluded, since a
block dominance test alone does not prove the current iteration defined it
before a use in that same block. Reuse every existing alias, written-offset,
overflow and bounds condition. This is an analysis change, with no additional
run-time operation and no language or specification change.

Changing iteration to bind values was rejected: it changes the language's
reference semantics and is not general for non-Copy elements. Rewriting
reference places in another MIR pass would duplicate the existing definitions
and affine-access model, and add transformations the optimization does not
need. Following the existing model costs at most 16 reference steps per access;
the C compiler can already eliminate these local pointers.

**Evidence.** Six new conformance cases exercise an Array total, a Span total
through a copied reference, the width proof for i32 and UTF-8 bytes, checked
overflow, rejection of unknown and strided references, and reading through a
reference before rewriting a view (including first-overflow order). The
first case's C had zero block proofs before the fix and has two afterwards.
Every case's C assertions pass for MSVC and clang in debug, release and
shipping. A chain of two element iterators retains two logical loops, with a
guarded unchecked copy of each; its fixture now expects four generated loops.
The full MSVC run passes every other target, and its complete conformance
target passes after that fixture correction. The full clang workspace suite,
the annotation sweep and all eleven gates pass. Appendix A leaves the
adopted specification unchanged byte for byte.

**Speed.** One million bytes summed 4,000 times, outputs identical, on the
owner's laptop on mains power and its performance cores, median of 11
interleaved runs: MSVC Ember 0.4246 s, fixed-count C 0.4040 s, runtime-length
C 0.4277 s; clang Ember 0.4032 s and C 0.4032 s. The MSVC assembly has two
independent totals for the runtime-count loop and four for the fixed-count
loop. Giving the generated C a fixed count takes 0.3999 s, recovering the
whole gap; changing only its unsigned addition to signed takes 0.4282 s.
Ember matches the C with the same runtime length. This is the host compiler's
runtime-trip-count cost, not a remaining per-element safety check. Tracking
exact collection lengths from construction into loop specialization is a
possible general future optimization, outside this reference-access change.
Raw samples, C, assembly and the diagnostic scripts are in the session folder
named in the handoff. No benchmark numbers were written to the README.

**Amended by ADR-124 (D-499):** a total updated on only some turns no longer takes the block form
this change made reachable for byte iteration.

## ADR-110 — String mutation over the shared byte buffer

2026-10-03. D-478 and D-479; delegated ODR-098, Hardened_52.

**Context.** `[TXT-11]` promised seven methods with no implementation and
incomplete signatures. ODR-098 chooses byte offsets, character-boundary
checks, additional-byte reservation, capacity retained when shrinking and
integer inputs checked before conversion. No character-position scan or raw
mutable bytes are exposed to safe code.

**Decision.** Reuse the existing buffer for all seven methods. Construction
starts an empty String and reserves bytes; capacity, reserve and clear share
Array's proven paths over u8. Insert, remove and truncate use inline UTF-8
helpers, moving only the suffix, and passing the original call's source
location to any boundary or bounds panic. The existing exported decoder is
retained as a wrapper around the shared inline codec. Existing UTF-8 encoding
callers use the same inline encoder, with no duplicate algorithm.

Check a size's original integer before handing a native length to the
runtime: bind it once, cast, check a round trip and (for signed integers)
nonnegativity. Range values erase to their representation first. This uses
the existing HIR comparisons and assertions and is valid on any pointer
width; it needs no new numeric operation or run-time conversion function.
The C compiler removes redundant round-trip comparisons for native-width
values. A cast alone lets wide offsets wrap into a valid index and negative
truncation become an unintended no-op.

Evaluate and reserve the receiver before its arguments, preserving `[EXP-1]`
left-to-right effects; activate its mutable borrow only at the call, as
`[BRW-3]`/`[BCK-5]` require. Binding arguments in an outer block reverses
receiver-index effects and permits writes before the receiver reservation.
The original reservation analysis followed only a single-successor chain,
so conditional arguments (including exact-size checks) incorrectly activated
early. Its general control-flow search now requires every returning path to
reach one consuming call with no earlier borrower use or reassignment. Panic
paths may terminate; cycles or escaping paths remain conservative. Method
autoref diagnostics reuse that provenance search. Genuine user-bound mutable
references remain active from creation (D-039), and writes during reservation
or shared borrows live at activation still fail.

Implementing another String allocator or a raw-byte mutation API was rejected:
the current checked buffer already supplies growth, allocation accounting and
freeing, and UTF-8 is a safe-code invariant. A character-position interface
would scan the prefix and disagree with text slicing; ODR-098 rejects it.

**Evidence.** The pre-fix combined method probe fails at
`String.with_capacity`; afterwards it inserts/removes a Unicode character and
reads the requested capacity. Twenty-one new TXT-11 cases test all seven
methods, ASCII and multi-byte Unicode, NUL, empty text, preserved capacity,
argument reads before mutation, invalid bounds and boundaries, wide/negative
sizes, and rejection while a text view is live. The first standalone UTF-8
panic prints its call's location without a dialog. Two BRW-3 cases cover
conditional argument reads and both branches' borrows live at activation.
The receiver-order probe printed offset/character/receiver before the fix;
it now prints receiver/offset/character. Break-testing outer argument bindings
accepts a mutation during reservation, and refusing reservation branches
rejects valid conditional reads. Both fixes restore the correct result.
Focused annotations and final MSVC/clang/GCC workspace correctness suites pass
after these review fixes. The complete final-code matrix and wider performance
audit remain open.

**Performance.** No timings ran during the owner's battery-power interval.
After mains was restored, 40 million Unicode edit turns with all seven methods
were compared to hand-written C, with identical output. Median of 11
interleaved runs on the performance cores: msvc: Ember 0.4477 s, C 0.4703 s (0.95x); clang: Ember 0.4551 s, C 0.4579 s (0.99x).

The original clang assembly called `ember_vec_reserve_more` twice on each
turn, even with adequate capacity. A hand-edited checked inline fast path
takes 0.4506 s against the original 0.4625 s and C's 0.4537 s, removing the
whole gap. Build that general fast path: it checks the original length and
addition, then calls the existing growth function only when capacity is
insufficient. Array reserve/extend and String use it; the exported reserve-more
function delegates to the same helper, preserving its ABI. Bounds, UTF-8 and
size checks remain. Array and String C assertions fail when the compiler's
inline-reserve calls are removed.

The owner explicitly required GCC performance comparisons too (2026-10-03).
Under GCC 15.2 with the same release flags (`-O2`, cheap vectorizer and loop
unrolling), matching output and 11 interleaved runs: GCC: Ember 0.4166 s, C 0.4390 s (0.95x).
Both programs run on mains pinned to WSL guest CPU 0; host performance-core
affinity is unverified. This is a within-WSL comparison, not an absolute-time
comparison against Windows. The handoff records raw-sample paths and power
checks; these focused numbers do not regenerate README charts.

## ADR-111 — single-byte string searches use one guarded `memchr`

2026-10-03, while regenerating the full benchmark matrix. The owner clarified
that the five existing over-10% programs have accepted memory-safety overhead
explanations; every newly added entry needs investigation. MSVC's `split` moved
around the threshold (1.09x–1.13x C), so it was investigated before publication.

The generated C's generic string search carried the separator length and
multi-byte retry loop even for a one-byte separator. Initial removal of the
overflow and slice checks did not remove the gap. Forcing the constructor to
inline, replacing the preallocated text fill with `memcpy`, and spelling the
iterator's separator as a literal did not remove the whole gap either. A hand
written rest-view loop was faster than the reference, so an iterator-shape
explanation alone was insufficient.

**Built:** `ember_str_search` handles every one-byte needle with one `memchr`
over the remaining view. It checks `from < s.len` before adding to the pointer
or reading the needle, including an empty String's null backing pointer. Empty
needles still match through the view's end; multi-byte needles retain their
candidate search and comparison loop. This benefits `find`, `contains`,
`count`, `replace` and all callers, with no special case for the benchmark or
comma. No overflow, slice or UTF-8 safety check is removed.

**Priced**, median of 51 interleaved runs on mains and Windows performance
cores, identical output `10000200 590000000`: original Ember 0.0670024 s,
hand-written C 0.0608453 s (1.1012x); checked single-byte path 0.0644052 s
(1.0585x). Removing overflow and slice checks only from the diagnostic with
that path gives 0.0604966 s (0.9943x), accounting for the remaining gap.
The old generic path alone kept it even with checks removed; this is why the
initial no-checks result was not accepted as an explanation.

The new TXT-10 case covers empty backing buffers, first/last-byte matches,
UTF-8 byte offsets, NUL, repeated matches, replacement and empty split parts.
The standalone MSVC debug panic still exits 3 without a window. Full MSVC,
clang and GCC correctness validation now passes after the final code changes.
The final 52-program measurements and wider slowdown audit remain open. Diagnostic C, assembly and all raw
samples are preserved beside `price-split-search.py` in
`C:\Users\ism19\AppData\Local\Temp\ember-codex-autopilot-20261003`.

## ADR-112 — byte reductions widen with SAD after the existing proof

2026-10-03, the owner's stricter slowdown audit. A runtime collection length
and a C compiler's choice of partial sums are not reasons to stop: an available
safe implementation must be tried. Only the five previously investigated
over-10% programs are exempt; new close-to-C entries also require investigation.

**Measured before implementation**, 11 interleaved runs on mains and Windows
performance cores, matching output `499992480000`: original Ember 0.41185 s,
C 0.39456 s; two/four scalar partial sums 0.39022/0.39068 s (0.989x/0.990x C).
Eight/sixteen are much worse (10.510/10.535 s) and discarded. A second paired
experiment prices SSE2, with the original entry proof, checked copy and outer
total's overflow check: one/two/four vector accumulators 0.08578/0.06805/0.05755 s
against original Ember 0.41168 s and C 0.39459 s. Four sums are 0.146x C.
These are diagnostic variants; final production timings follow validation.

**Built:** the C backend follows the complete data flow of an integer byte
reduction, including copies of the induction variable, immutable element
references and widening casts. Only a straight, read-only unit-stride stream
with one 64-bit total and dead loop intermediates qualifies. Integer add and
subtract, signed and unsigned byte values, and arbitrary Array/Span views use
the same selection. Any remaining check, call, mutation, live intermediate,
prefix-result store or unsupported expression keeps the original loop. Native
size counters and exclusive bounds are required in this first implementation.
No source name, benchmark size or literal byte value is recognized.

`ember_sum_bytes` returns the sum modulo 2^64. On SSE2 targets four independent
accumulators use SAD to widen each group of eight unsigned bytes. Signed bytes
are biased by 128 and unbiased modulo 2^64; scalar tails sign-extend correctly.
The loop loads a full 64-byte group only if that many bytes remain. Other
targets use a scalar loop. The helper is used only on the existing unchecked
path: its arithmetic and bounds proof and original checked fallback remain.
An empty loop never forms an address from its empty/null backing pointer.
The low vector lane is copied with C `memcpy`, avoiding a register-extraction
intrinsic absent on 32-bit SSE2 targets. No object ABI or spec change.

Instruction choice was informed by [Microsoft's intrinsic list](https://learn.microsoft.com/en-us/cpp/intrinsics/x64-amd64-intrinsics-list)
and [Intel's instruction reference](https://cdrdv2-public.intel.com/789581/325383-sdm-vol-2abcd.pdf),
then priced in our actual generated C; documentation alone was not evidence
of a speedup. Raw samples and assembly are beside `price-byte-partials.py`
and `price-byte-sse.py` in
`C:\Users\ism19\AppData\Local\Temp\ember-codex-autopilot-20261003`.

**Verification:** all byte values, signed subtraction, nonzero starting
totals, unaligned slices and every short tail agree with an independent
closed-form answer under MSVC, clang and GCC in debug/release/shipping. Prefix
outputs retain the original loop, and a near-limit starting sum still panics
on its first overflowing add. A separate scalar C oracle checks views beside
inaccessible pages, every length through a page and unaligned offsets; all
three compilers pass, and GCC passes the portable scalar fallback too. Removing
the backend selection fails the conformance C assertion. An external header
control that drops the vector length guard faults at the guard page. Final
MSVC/clang/GCC workspace correctness validation passes; final production timing
and the wider performance audit remain open.

The virtual-call and text-iterator gaps remain open investigations. A safe
allocation prototype beat the old incomplete C++ twin while retaining the full
object header, and moving temporary owners also helped. That twin omitted
object deletion while Ember destroys its owners; those historical ratios do
not settle the gap. The corrected external raw-pointer twin adds delete after
printf and is ready, unexecuted, for new pricing. Neither header size nor one no-checks
control proves that the remaining overhead cannot be optimized safely.

## ADR-113 — allocation statistics describe the calling thread

2026-10-03, found during the strict virtual-call allocation audit. `[RT-11]`
explicitly requires per-thread statistics or debug-only counters, and forbids
shared non-atomic writes on allocation. The runtime used one plain global
`g_stats` in every profile. No design or spec change is needed to fix it.

**Built:** keep the existing allocator and statistics ABI and place the three
counters in runtime TLS. Allocation and free totals describe operations on
the calling thread. A free of another thread's block is charged to the freeing
thread; live bytes are that thread's balance, saturated at zero. They cannot
be read as a global leak total, and the public header now states this. This
avoids adding global atomics or an allocation-owner prefix to every block.
The zero clamp also makes free agree with the existing realloc accounting.

**Verification:** a C oracle uses a real OS worker thread, checks an initially
empty worker view, allocates/frees locally, frees the parent's block, joins,
and checks the parent's independent totals. It fails deterministically with
the old shared counter without requiring concurrent accesses to provoke UB.
The assertion for a cross-thread free larger than a positive local balance
also fails before the clamp. The final probe passes with MSVC, clang and GCC.
The mandatory standalone Windows panic prints and exits without a window;
final three-compiler batch correctness validation passes. The wider allocation
performance investigation remains open.

## ADR-114 — transfer fresh temporary owners only when destruction cannot advance

2026-10-03, D-483, found while investigating GCC's virtual-call comparison.
Fresh ClassNew values passed through class/interface temporaries and Array
push paid count pairs despite a replacement owner living until the old drop.
The optimization is in general MIR ownership analysis, not a benchmark shape.

Transfer only a counted Temp with one whole definition, one whole Copy use
and one unconditional whole Drop, whose cached origin is a known fresh
non-Sync class allocation through temporary copies/upcasts. The replacement
class/interface temporary must itself be a counted owner. Array push must
be followed by a quiet all-path interval to the original drop: intervening
calls, drops or mutations that could clear/repoint the array are excluded.
Dominance, storage lifetime, references, moves, conditional/projected drops,
cycles and unknown origins are checked conservatively. Change Copy to Move
and erase only the old owner's now-redundant drop.

Do not transfer the caller's last fresh owner into an owned function argument.
The callee can explicitly drop its owner before returning, and advancing the
object's observable deinit would change semantics. Named owners and Sync
allocations remain counted; this does not alter the safety or counting model.

Build one block-dominator tree and interval index per body, then query statement
order and block dominance in constant time. Cache iterative origin-chain proofs
across both transfer stages; avoid an entry-graph traversal per candidate or a
quadratic dominator matrix. Fifteen units compare CFG answers to an independent
path-removal oracle, cover irreducible edges and verify operation counts on
16,384-node graphs/chains. Six new conformance cases cover interface/base
sinks, named/Weak lifetime, explicit callee drop, and retained Sync counts.
A later review found overlapping intervals could revisit the same statements
quadratically even though dominance and origin lookup were cached. Uncounted
membership is now a bitmap and drop-point membership a hash set. One shared
interval budget charges traversal starts, pops and edges across both stages;
its limit is 32 times statements + blocks + edges. On exhaustion, leave all
unproved ownership operations intact. This bounds interval traversal without
claiming a linear worst case for dominance construction. Units cover 1,024
overlapping lifetimes, declined-transfer preservation and 1,024 cheap transfers.
Array intervals also reject Assert because configured panic hooks run before
abort and may observe owners; no callback non-observation assumption remains.
Removing the transfer pass makes the positive C assertion fail. Three-compiler
profiles and final workspace correctness suites pass. Corrected C++ lifetime-
parity comparison and the wider performance investigation remain open.

## ADR-115 — a char contributes its scalar upper bound to integer range facts

2026-10-03, D-484. A valid char is in 0..=0x10FFFF ([TYP-3]); use that interval
in the existing type-range machinery. It conservatively includes the invalid
surrogate gap, which cannot create a false proof. Invalid chars are possible
only through unsafe behavior already ruled UB. FFI char32_t is mapped to u32,
whose arbitrary bits are not restricted by this fact ([FFI-8]).

A noinline fixture checks widening char to u32 and adding one, including NUL,
ASCII and the maximum scalar, while preserving overflow checks for a larger
addition and an unrestricted u32 parameter. This does not prove text-loop
accumulations safe; that separate investigation remains open. Disabling only
the Char fact makes the noinline C assertion RED (three checked additions
instead of two); restored code retains the two required checks. Final batch
full correctness validation passes; dynamic text-reduction proof work remains open.

## ADR-116 — native text cursors keep their actual unsigned pointer type

2026-10-03, D-485. Diagnostic 31-run alternatives preserved all checks.
Unsigned internal state improved GCC; common advance improved MSVC, while
clang favored the stock branch-local updates. These results identify safe
alternatives and do not make GCC's remaining gap reasonable or unavoidable.

Native text-loop lowering now uses hidden usize state, compares it directly
with the evaluated valid view's byte length, and copies char_indices offsets
as int. Every pre-step cursor is below len. A valid UTF-8 scalar advances by
1..4 to at most len, so usize cursor arithmetic is representable. All 32-bit
native offsets fit int; managed 64-bit buffers cap bytes at PTRDIFF_MAX.
Foreign views do not universally enforce that cap: a hypothetical allocation-
backed 64-bit view beyond INT64_MAX exposes the existing public signed-length
contract gap (D-487), also present in .len() and stored iterators. Invalid
allocation extents are unsafe UB; oversize alone is not expressly ruled UB.
No realizable supported-target reproducer was found. Empty text never decodes.
The saved view, keep-alive/borrowing, step-before-body and loop control remain.

StrCharNext codegen inspects its actual mutable-reference pointee and dispatches
to distinct size_t* and int64_t* helpers. Std Chars/CharIndices keep their int
state and old signed helper. Equal widths do not permit pointer punning. One
private header macro defines both codecs, then is undefined. Genuine MSVC
uses cached lead byte/common advance; clang/GCC retain the measured stock
shape. No public API change or new unsafe primitive is exposed.

Two run-pass fixtures cover UTF-8 widths, NUL, maximum scalar, signed public
offsets, native/stored iterators, null-backed empty String and validated empty
byte views, exhaustion and continue/break/else. Two privacy negatives keep the
unchecked helper and iterator cursor mutation inaccessible to users. An
independent encode/decode oracle covers all 1,112,064 valid scalars, both
typed helpers and zero/nonzero starting cursors, with each scalar ending at a
protected page. It passes prior compiler validation and actual 32-bit MSVC. An
external deliberate overread faults at the protected page (0xC0000005); the
canonical header remains unchanged. Restoring signed native lowering makes the
unsigned-helper C assertion RED. Production pricing is diagnostic; GCC
reductions still need safe general alternatives investigated. Final full
correctness suites after the latest codegen and capability correction pass.

## ADR-117 — split_once produces the two sides of one proven text match

2026-10-03, D-486. General range analysis would need projected conditional
Option facts, value identity and three-term relations to recover every search
postcondition after lowering. The existing public split_once already names
this exact semantic operation. Implement it directly, then use it in every
std Split.next. This improves all users/separators without recognizing a
benchmark, expression spelling or panic message, and requires no new API.

Snapshot receiver then separator in the existing HIR Let wrapper; a separator
expression may repoint the original receiver descriptor. C emission snapshots
both evaluated descriptors before writing any output field, so output/source
overlap is safe. One str_search gives SIZE_MAX or a match. Success proves
at <= text.len and separator.len <= text.len-at; compute tail=(len-at)-sep.len
and consumed=len-tail, avoiding a checked signed endpoint addition. A complete
valid UTF-8 needle starts and ends at scalar boundaries of valid UTF-8 text.
Construct the ordinary tagged Option[(str,str)] directly using existing enum
layout helpers, without another slice validation, pointer punning or runtime
result ABI. The suffix uses consumed==0 ? ptr : ptr+consumed, including a
null-backed empty String and empty needle. Never form NULL+0.

Both Some tuple views have exact nested result-field provenance from argument
zero only. A temporary separator may die while the views live; the source may
not. Split still borrows/stores its separator for later iteration. Public
split_once with empty separator remains Some(("",text)); Split's constructor
still rejects empty separators, returns trailing empty fields and stays done.
Partition's distinct empty-separator panic remains unchanged. This does not
whitelist/elide any general MIR Assert.

Nine TXT-10 fixtures cover Unicode/NUL/no match/empty/trailing fields, both
retained accumulator checks, receiver evaluation before a mutating separator,
source-only borrowing, wrapper return/tag access, stored iterator exhaustion,
copy chains and each result field's source-mutation rejection. One TYP-13 case
covers shared-Span niche copy chains through reference/index storage, empty
Some and joined None, with a wholly written element preserved.

Fresh MSVC production initially regressed because nested descriptors were
written by fields then read whole, and copied destinations lost that producer
classification. A 31-round rotated/reversed diagnostic isolates the cause:
baseline 1.728664x C, tag-only None 1.721775x, first payload reads by fields
1.723587x, propagated descriptor copies 1.040194x, and combined 1.025482x. The
general correction seeds both nested descriptors and follows exact-type
Copy/Move chains and exact-payload niche Some wrapping/extraction. Niche
Some.0 is canonicalized to its physical descriptor; all real
dereferences/indices and str/span, shared/mutable and element-type
distinctions remain. No arbitrary alias merging, tuple/struct recursive
scalarization or source-body recognition is introduced. Wholly written
unrelated view sources remain whole (D-475). None writes only its live tag; no
inactive None payload is read or scalarized. This does not remove object
allocation zeroing.

The copy-chain C milestone is RED before adoption. Removing propagation or
niche storage canonicalization independently is RED; the inverse
whole-copy/Lines milestone remains GREEN. Independent before/after provenance
breaks lose exactly the corresponding text-mutation rejection; receiver-order,
old split-check and separator-provenance breaks are also RED. A completely
absent result contract is still rejected by the general validator (E3065), so
that control is not misreported as broken-code acceptance. The
temporary-source guard remains E3060.

Post-fix 11-run production diagnostic Ember/C ratios are MSVC 1.038572x, clang
0.949404x and GCC 0.941660x. The complete normalized MSVC C equals the
combined prototype, with eight whole-copy replacements; clang differs only by
its seven descriptor replacements and tag-only None. Both input snapshots and
both accumulator checks remain. Windows a05_structs remains
1.003614x/1.002660x. GCC full optimized binaries for all seven paired programs
(both Ember and C) are byte-identical before/after fields, including split
despite its C changing: timing movement under GCC does not establish a
machine-code speedup from this correction. These are diagnostic evidence, not
README updates or a reasonable/unavoidable-overhead claim. Final full correctness
suites pass; the wider slowdown audit remains OPEN.

Final correctness evidence for ADR-110 through ADR-117 and D-489/D-491:
final-fields-workspace-{msvc,clang}.log, final-fields-gcc-{build,workspace,annotations}.log,
final-fields-msvc-annotations.log and final-fields-validation.json in the same
external directory. The MSVC/GCC annotation sweeps report failing 0; all eleven
gates pass, and Appendix A is byte-identical. This validation does not close the
performance investigation or adopt the external startup/allocation prototypes.
The first MSVC suite log is preserved separately: unexpected shared-Span
alias/wrapped L1001 warnings were fixed by deliberate unused-name prefixes,
then the complete rerun passed; the focused runner now checks unexpected
diagnostics. D-490 remains OPEN.

Evidence: msvc-current-split-aggregate.json, split-copy-break-results.json,
split-each-field-break.json, text-break-summary.json,
current-feature-after-fields-{windows,gcc}.json and
post-fields-binary-identity.json in the external session directory.


## ADR-118 — discard unreachable managed GCC executable sections

2026-10-04, D-492. Implemented and correctness validated; production
performance confirmation and the wider audit remain OPEN. No semantic or ABI
amendment is required.

The empty GCC program's runtime object carries unused functions, constant
strings and TLS. A controlled outside-repository comparison, three balanced
windows and 378 retained samples per label, reduces .text from 69,195 to 9,608
bytes, .rodata from 50,400 to 335 and TLS from 40 to 8 with function/data
sections and linker removal. PIE, RELRO, BIND_NOW, non-executable stack and CRT
initialization/finalization remain. Its ratio to C is 1.038796 versus the
baseline's 1.053066; the sectioned no-GC control is 1.051706. The isolated
GC/sectioned ratio is 0.987725 (cycle bootstrap 95% interval 0.9791–0.998935).
The identical-binary control is 0.994749 of baseline: these small process
measurements require caution. This is whole-process launch/loader/I/O work,
not a claim that the residual cost is intrinsic. The retained libm dependency
and other safe startup alternatives remain OPEN. No README values are updated.
Host mains checks passed and the exact UTC window has zero Kernel-Power 105
source transitions; WSL guest CPU0 is pinned, host performance-core affinity
is unverified. Evidence: external gcc-startup-investigation/completed-first.

Use explicit additive build APIs for driver-owned executables. On Linux GCC,
compile their generated C, cached runtime and relaxed-FP objects into individual
function/data sections. The changed command participates in the runtime cache
hash; source fallback receives the same policy. Generic compile_and_link,
runtime_object, compile_relaxed_object, compile_object and static archives
keep their original policy. FP/profile/protection defaults and RT-1's raw
allocator remain unchanged. Separate sections can increase build cost or alter
layout; final production comparisons are required before claiming a gain.
[GCC documents the section flags](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html).

Link removal must retain the public C definitions even without an Ember call.
Collect the same definition predicate as the generated header: ABI C, neither
extern declaration nor abstract. Sort/deduplicate actual Body.symbol values,
including selected @export names, written extern-C names, imported definitions
and relaxed-FP units. Use --gc-sections and --require-defined for these roots:
missing definitions fail, and relocations keep their dependencies. Do not root
unused foreign declarations, all native global symbols, or export everything
dynamically. Retention does not promise dynamic symbol discovery; export
tables remain outside the implemented language surface.
[GNU ld defines the root and relocation behavior](https://sourceware.org/binutils/docs/ld/Options.html).

GCC does not guarantee GNU ld. Ask GCC for its selected linker, and require
both documented options in that linker's successful help output. Failed or
unsupported capability discovery uses ordinary linking, which keeps the
complete image. The integration test independently attempts an actual link,
so an always-false detector cannot silently skip the optimization assertions.
A simulated linker lacking required-root support verifies the complete fallback.

The rt_init(NULL) entry experiment has been reverted. Paired native GCC
whole-process sampling gives NG_lm/G_lm 1.002681 (cycle bootstrap 95% interval
0.995958–1.012479) and NG_nolm/G_nolm 1.000870 (0.993833–1.007373), both neutral.
Ordinary main therefore retains its original default-config construction.
Initialization/main/shutdown, leak-check flags and ownership-edge registration
remain. Allocation counters remain cumulative across reinitialization.

Focused regression covers three profiles, cached/source-fallback runtime,
unused native removal and all supported C root kinds. A real native fixture
checks CRT constructors/finalizers, indirect callbacks, custom alloc/free/panic
hooks, TLS accounting, default and configured init/reinit, exact panic abort,
non-executable stack, RELRO and missing-root errors. Removing C roots and
section removal separately is RED; exact candidate restoration is GREEN.
Standalone production NULL-metadata panics passed window-free under MSVC and
clang before batch checks. Full MSVC, clang and GCC workspace suites pass;
MSVC/GCC annotations report failing 0, all eleven gates pass, and Appendix A
regeneration is byte-identical (SHA256 f8ec2010eadc6f7393e81c67c77dcb8268b8604578f419bc03eedad422dcc4af).
The 4,260-file frozen input manifest has source ID
d4f301cabcdc6a859c8008426770726a77d4b98e15e13b1b8ff2385bf54ce3dc.
Evidence is under external d492-validation/full-short-1791054923844312300
and continued-1791056120781943700. The first deeply nested MSVC temporary root
exceeded MAX_PATH; the same unchanged milestone is RED there and GREEN with a
short temporary root, followed by complete workspace reruns. The first GCC
annotation invocation omitted its documented EMBER=target/debug/ember setting;
its launch failure is preserved, and the corrected remaining checks pass without
source edits. These harness failures do not establish a production regression.
That earlier frozen-source validation predates the neutral entry-experiment
reversion and D-493 regression/guard. Fresh validation now passes on the complete
4,262-file source ID
c936abc3b1eaec9d0d2263dcb2ec6cc87efefc2a54e1ff34e2739242481da18c:
all three full workspaces, MSVC/GCC annotations (failing 0), all eleven gates,
and byte-identical Appendix A. The first launcher completed the Windows suites
then failed before GCC testing because its nested shell expanded the saved exit
status. The corrected literal-file --exec continuation verified the original
source, tools, record chain and every log before reusing only those two completed
Windows suites; it then executed all remaining groups. Both results remain
preserved. Evidence: external d493-validation/run-20261004-continuation-03/
combined-validation.json, SHA256
6b92f222c821e43efcd9d8617be5f5281a548b19d2fbbb5d6bf9073d97ad8810.
No final README matrix or reasonable-overhead conclusion follows from this
correctness record.

**The probe's answer is cached** (2026-10-04, autopilot). Asking GCC for its linker and the
linker for its options took two processes on every gcc build. The answer is kept in the global
build cache (`linker/`), keyed by the compiler file (path, size, modification time) and
`COMPILER_PATH`/`GCC_EXEC_PREFIX`, and it holds while the linker GCC named is found at the same
file, unchanged (a bare name is looked up on `PATH` again); otherwise it is asked again. Measured
in WSL on mains, a small program's gcc build 124.7 → 122.6 ms (31 interleaved runs, guest CPU 0),
the two processes alone 1.3 ms. `the_linker_probe_is_cached_until_the_linker_changes` counts the
processes a fake GCC is asked for.

**An answer the probe could not finish is not kept** (D-510, 2026-10-05). A program the probe
could not start was cached as "no" (gc-sections off) until the compiler file changed, and the test
above failed once in WSL: it runs its scripts just after writing them, and on Linux a file just
written is "text file busy" to `exec` while another thread's new child still holds it open, for a
moment. The probe now tries a program that did not start again (four times, 10 to 80 ms apart),
and keeps only an answer it finished; the test checks that a linker that cannot be started leaves
no answer behind.

## ADR-119 — reject unrepresentable aligned-allocation round-up

2026-10-04, D-493. Fixed and production correctness validated.

The non-MSVC default allocator rounds a size to a multiple of its power-of-two
alignment. Near SIZE_MAX its addition can wrap to zero, violating the requested
allocation size. Reject size > SIZE_MAX - (align - 1) before that addition, only
in this C11 over-aligned branch. Return NULL through the existing allocation
failure path, with unchanged counters. No new PTRDIFF_MAX ceiling is imposed
on the raw allocation API; ordinary malloc and configured hooks still receive
the original request. MSVC forwards directly to _aligned_malloc and has no
round-up addition. Zero-size default normalization remains one byte.

The public try_alloc regression includes the actual generated runtime after
NULL-only allocator interception. It checks ordinary and aligned warmups before
boundary requests, small/exact/largest representable multiples, overflow rejection,
custom-hook forwarding and unchanged statistics. Every intercepted allocator
returns NULL; no huge real allocation, dereference or free occurs. The permanent
GCC test fails on the original runtime with requested_or_rounded=0, aligned=1.
External guarded/inverse/restored variants pass their independent assertion
oracles, and MSVC/LLVM confirm their unchanged forwarding branch. The permanent
regression is GREEN in full MSVC, explicit LLVM clang and GCC production
workspace suites on source c936abc3. MSVC/GCC annotations report failing 0,
all eleven gates pass, and outside-repository Appendix A regeneration is
byte-identical. Standalone production NULL-metadata panics pass on Windows
before batch tests; the window watchdog remains clear. The frozen manifest
contains both new regression files and the template/generated-runtime guard.
The original GCC launcher failure and the verified continuation described in
ADR-118 remain intact; no source was edited during these suites.

## ADR-120 — bound a pure text reduction by semantic decoder progress

2026-10-04, D-494. Implemented; focused, native and workspace correctness
validated. The completed pre-pause suites/annotations/gates are historical;
current focused evidence and remaining performance work are recorded below.

A UTF-8 decoder advances its private native-width byte cursor by at least one
byte per iteration. A loop can therefore bound its remaining additions without
a benchmark-specific pattern or a new counting builtin. Apply this fact to one
signed-i64 addition of a proven nonnegative term; keep the original checked
loop whenever the proof is unavailable or the runtime guard fails. This is
independent of the existing CountedLoop proof and does not relax that contract.

The initial slice accepts a single-entry natural loop of at most 128 blocks,
one typed StrCharNext step, one signed-i64 CheckedBinaryOp Add and its overflow
Assert, with only the normal header-false exit. Cutting the header backedges
must leave a pure DAG: branches may skip the addition, but hidden cycles,
extra entries/exits, calls or observable effects decline. The cursor must reach
zero through every incoming path, and its sole mutable reference is private to
the decoder. Dominance, definite initialization, exact operand types and
whole-body alias/flag-use inventories are checked. Copied accumulator carries,
unsupported casts/signs, callable argument-binding or hoisted-access metadata
also decline. Term bounds support constants, scalar/range limits, byte offsets
at their original statement point, copies, safe numeric casts and supported
bitwise expressions; decoder chars are bounded by 0x10FFFF. This uses existing
valid-UTF-8 and decoder-boundary invariants, not new runtime validation.

The guard first proves length <= INT64_MAX and cursor <= native length.
Remaining bytes give an upper bound on updates. A zero remaining count falls
back before the guard reads the accumulator; a zero term bound needs no
division. Otherwise reinterpret the initial total as u64, compute unsigned
INT64_MAX - total_bits, and require remaining <= room / term_upper. This equals
the mathematical available room even for INT64_MIN. No product or signed
INT64_MAX-total calculation is emitted. Only the certified Add and its overflow
Assert change in the clone. The checked fallback retains its operands, spans,
first-overflow panic and effect order. The safety side table records the removed
overflow check with LoopEntryTest evidence.

Proof work is deliberately bounded, not advertised as globally linear. The
whole-body initialization lattice is capped at 262,144 block/local cells;
bodies, statements/edges, candidate headers, recursive term depth/work and
entry-zero walks have explicit limits. At most one loop is committed per body;
the enlarged candidate is budget-checked before a fresh initialization lattice.
Over-budget or unsupported candidates preserve their checks.

Run the pass after final pruning and exit-drop removal. Retain its opaque
complete certificate batch, including immutable whole original bodies and
original body indices. Refresh callable-region summaries when MIR changes.
Immediately before the unconditional ordinary for_codegen gate, verify the
complete progress batch in every profile. Verification re-extracts the proof
from originals, compares unchanged original/fallback material, independently
checks the actual typed guard and cloned loop, and also replays the construction.
Missing bodies/certificates, duplicate entries, changed spans/operands/proof-relevant metadata
or malformed guards fail compilation. No mutating pass follows that verification.

The first actual native compile exposed a prerequisite backend bug: a str
descriptor's length was emitted as tuple field ._1. Render str fields 0/1 as
.ptr/.len and type field 1 as internal usize, through direct, dereferenced and
nested projections. Do not inherit Vec's .cap fallback. Public len/index
semantics remain unchanged. Preserve the failed compilation and its focused
RED-to-GREEN correction separately from the optimization evidence.

Verification: all 18 actual-MIR proof/mutation/initialization/budget tests
and the driver regression passed before pause, with meaningful RED controls.
The pause checkpoint records 45 native cases, complete MSVC/clang/GCC workspaces,
completed annotation sweeps and eleven gates after the test-only branding fix.
Those full passes precede the later decoder/append production changes and are
not new full-suite evidence. The resumed current-source slice builds on Windows
and Linux and passes nine backend tests, 54 native cases across all profiles and
compiler families, and all three exhaustive Unicode guard-page oracles. The
large t6 pair is now faster than C on all three compilers after ADR-122/123;
the actual short-input guard still adds avoidable setup costs (D-498). Its
performance audit is OPEN. No native32 or final README matrix claim is made.
Evidence and failed harness controls: TEXT-PERFORMANCE-2026-10-04.md.

## ADR-121 — construct a borrowed String descriptor through the view-field path

2026-10-04, D-495. Implemented; focused, native and workspace correctness
validated. The completed pre-pause suites/annotations/gates are historical;
current focused evidence and remaining performance work are recorded below.

The StringAsStr runtime helper copies two fields from a borrowed String; it
does not inspect bytes or perform a safety check. Express that same operation
through the backend's existing view_parts path so generated C receives paired
pointer/byte-length assignments and the existing descriptor-copy classification
can follow their consumers. This is a general producer classification, not a
benchmark recognizer or a blanket aggregate-scalarization policy.

Eligibility comes from the actual operand reference type: exactly one argument,
an immutable Ref to Vec with text=true, and a str destination. Copy or Move of
that shared reference is eligible; neither transfers the String owner. Owned
String values, raw pointers, mutable references, nontext Arrays, Span/MutSpan
destinations and incorrect arity decline this producer path. The ordinary
helper emission remains the fallback for valid calls outside its eligibility. Do not infer eligibility merely from builtin arg_ty.

Read both fields from the same already-lowered shared receiver. Emit the
const-byte pointer and native byte length, then refresh the existing indexing
pointer cache. Seed the existing parted-view copy graph so projected/niche
copies retain the paired-field route. The change leaves receiver evaluation,
borrow/result provenance, class access scopes, retains and public helper ABI
intact. The ordinary ember_vec_as_str declaration and definition remain
available to native callers and unsupported emission forms. Empty/reserved
String views and Some(empty) preserve their descriptors and tags; no pointer
arithmetic or byte load is added.

Verification: four backend tests cover eligibility, rejected forms,
destination/arity and str projections. The driver checks adjacent paired fields
and their projected Option route under all nine selectors; its meaningful RED
without the producer arm is preserved. The historical pre-pause complete
validation is recorded in the pause checkpoint. Current focused evidence is
54 native cases across all profiles/compilers, including named-field exclusivity,
views, Unicode/NUL/empty descriptors and the new append case. Nine backend tests
pass after integration. Do not attribute the combined whole-program t6 gain
to this producer alone or describe the old complete suites as fresh. The wider
audit and README matrix remain pending. See TEXT-PERFORMANCE-2026-10-04.md.

## ADR-122 — keep typed UTF-8 steps inside versioned text loops

2026-10-04, D-496. Implemented and focused correctness/performance validated.

The MSVC cursor codec bodies now use EMBER_INLINED (amended 2026-10-04: gcc and clang keep
`static inline`; forcing gcc's costs `split_whitespace` 10%, D-496). This is the existing
policy for small per-operation fast paths; no decoder masks, cursor type, width,
pointer access or valid-UTF-8 precondition changes. Versioning enlarged t6's
caller enough that MSVC emitted a decoder call for every character. Actual
assembly, before and after this annotation, establishes removal of those calls.
The original checked fallback still emits overflow/panic behavior. The isolated
MSVC Ember/C ratio moves from 1.186 to 1.006; clang and GCC remain faster than C.
The exhaustive guard-page oracle and all focused native cases pass on all three.

[Microsoft documents that ordinary inline expansion is discretionary and even
forceinline is not an absolute guarantee](https://learn.microsoft.com/en-us/cpp/cpp/inline-functions-cpp?view=msvc-170).
Therefore actual assembly and measurements, not the keyword alone, are the
evidence. Complete current validation remains required before push. The short
guard setup cost is separate and remains OPEN under D-498.

## ADR-133 — a Sync object's counts are inline atomic operations (`[RT-10]`, D-509); its counts tested under threads (`[RC-4]`, `[WK-12]`)

2026-10-04, autopilot. `[RT-10]`: "Counting is inline. The fast path of retain (one increment and
an overflow test) and release (one decrement and a test for zero) is defined in `ember_rt.h` and
inlined into the caller; only deinitialisation is out of line. Copying an existing strong handle of
a `@sync` object is one atomic `fetch_add`, never a compare-exchange loop ... The deinitialising and
resurrection checks of `[OBJ-5]` run on the release-to-zero path only." A Sync object's retain was
the out-of-line compare-exchange loop `Weak.upgrade` uses (on MSVC its loads were locked
instructions too: three per retain), its release out of line, and the plain retain's fast path also
tested the deinitialising flag (D-509).

`ember_rt.h` now has `ember_retain_sync` (one atomic add, relaxed) and `ember_release_sync` (one
atomic subtract, acquire-release, `[RT-8]`), on gcc and clang's `__atomic` builtins, MSVC's
`_InterlockedExchangeAdd`, or C11 atomics. Only a count at its end leaves them: a retain that found
zero or the maximum panics (`ember_obj_retain_failed`), and a release that found one deinitialises
(`ember_obj_release_ended`, which panics at zero). The C backend calls them for a Sync class; a
handle whose class is not known statically (a `Shared`, an interface's) keeps `ember_retain`, which
now picks between the two inline forms by the type information. The plain retain no longer tests
the deinitialising flag: a deinitialising object's count is zero, which already leaves the fast
path, so the test bought nothing. `Weak.upgrade` stays the one compare-exchange (`[WK-12]`).

Ember has no threads of its own yet (Phase 6), so the counts were never run concurrently. The
runtime test `sync_counts` (`templates/tests/sync_counts.c.in`, run by
`ember_build`'s `sync_counts_hold_under_threads`) drives them from C: four threads retain and
release one Sync object 200,000 times each, some nested, and the count must come back exactly; then
300 rounds where four threads upgrade a weak handle while the last strong one is released, an
upgrade never holding a deinitialised object and each object deinitialised once. With the atomic
add made a plain increment it fails with both MSVC and clang (lost updates end the object early).

Measured 2026-10-05, 00:06 to 00:08, on mains and the performance cores, 11 interleaved runs,
medians, each program built by the compiler and runtime before and after
(`scratchpad/rc4bench`). A Sync object's handle stored into a field fifty million times, two
handles taking turns, against the same work with `std::shared_ptr`: MSVC 1.045 s before, 0.524 s
after (×0.501), the C++ 0.524 s (after ×1.000 of it, before ×1.995); clang 0.554 s, 0.525 s
(×0.947), the C++ 0.417 s (after ×1.258, before ×1.328; the rest not investigated yet). The
11 benchmark programs whose C retains a handle are unchanged with both compilers (×0.977 to
×1.006), except clang's `p4_views_alive_01`: 33.2 ms before, 19.4 ms after. Its retain is in the
setup loop, not the measured one, its C is the same before and after, and the published number
is 19 ms, so the difference is most likely where the measured loop was placed, not this change;
recorded as a lead.

## ADR-132 — a virtual call one body answers is a direct call (`[DSP-1]`, `[DSP-5]`)

2026-10-04, autopilot. `[DSP-1]` makes a call on a final class's handle a static call; `[DSP-5]`
lets a virtual call with exactly one reachable implementation become a direct call when the whole
program is visible, and has `--emit-optimization-report` list each. Neither was built: every call
to a virtual method read the object's table (D-508 for `[DSP-1]`), and the flag did not exist.

`FuncRef::Virtual` carries the receiver's class as the program wrote it, under the upcast to the
method's own class (and the borrow of a `mut self` receiver). The object is of that class or of
one below it, and a build holds every class of the program (a library's too: a C host cannot
derive from an Ember class, and `extern class`, N2, is not built), so the C backend looks at the
slot in that class and in every class below it. When they all hold one body, the call is made to
it directly, an argument cast where the body's C type differs from the slot's, as the override
adapters do. An abstract slot is passed over only in an abstract class, of which no object exists,
so an abstract class with one concrete class below it calls that class's body. A final class has
only itself: that is `[DSP-1]`. For an open or abstract class the answer took the whole program,
and `ember build --emit-optimization-report` (or `ember run`) lists the call on standard error, so
it never mixes with `--emit c`: `<file>:<line>:<col>: calls `Shape.sides` directly, the only
implementation for class `Shape` ([DSP-5])`, or `none`.

The analyses still treat these calls as virtual (they take every body the call could reach), which
stays sound. When hot reload (`--reload`) is built, a reloadable class can gain a subclass, so
`[DSP-5]` must then be off for it; `[DSP-1]` stays. An interface table's adapter for an open class
still calls through the object's table (D-375); the same test applies there and is left for later.

Measured 2026-10-04, 22:04, on mains (no Kernel-Power 105 event) and the performance cores, 11
interleaved runs, medians: a thousand objects of an open class nothing derives from, each asked a
virtual question in every one of 100,000 passes, against the same work in C++ (an open class with
one implementation, called through the base pointer; `scratchpad/dsp5bench`). MSVC: 73.9 ms
before, 48.5 ms after, C++ 66.5 ms (after ×0.657 of before, ×0.730 of the C++). clang: 72.5 ms,
36.3 ms, 64.9 ms (×0.501, ×0.560). Before, Ember took 1.11x the C++ time with both compilers.
The C of all 52 benchmark programs for MSVC, clang and gcc is byte for byte the same before and
after (none makes a virtual call), so no published number moves.

## ADR-131 — `ember inspect --counts`: the count operations left inside loops

2026-10-04, autopilot. `[RC-6]` has `ember inspect` list every retain and release that survives
inside a loop, with the reason it could not be removed: a surviving count operation blocks
vectorisation. It had not been built. A build now writes `<module>.counts.json` beside the safety
side table, and `ember inspect --counts <path>` prints it (`--json`, `--function <name>`).

The analysis (`count_report`) reads the final MIR the backend is given and mirrors what the
backend emits. A counted value is one that is `Copy` and still needs a drop: a class or interface
handle, a `Shared` or `Weak`, or a value holding one. Copying one retains (an assignment's copy,
unless it lands in an uncounted handle, `[RC-3]`; a literal's copied field or element; an
upcast's copy; a copy pushed into a list; a copy passed to an `owned` parameter), and dropping one
releases. Loops are the CFG's natural loops, and each operation names its innermost loop. A
reason says what keeps the count: a store into a field or element (the store's own temporary is
followed to where it moves), a list, an `owned` parameter, a copy into a local whose source is
not shown to hold the object, a value's end, or the old value a store replaces (`[OWN-5]`). The
loops of a list's drop glue are not the program's and are not listed.

## ADR-130 — the debug leak report is on by default, scales, and names each object's cycle

2026-10-04, D-507, autopilot. `[WK-15]` has `ember run` (and `ember test`, not built yet) in the
debug profile report leaked objects and their cycles at exit by default, `--no-leak-check`
turning it off; `[WK-4]` has the report name, for each leaked object on a cycle, the shortest
strong cycle through it as `Type.field` edges and the edge to weaken (`L3017`). The report
existed behind `--leak-check` (`[WK-8]`), on a registry that could not be on by default: a list
every release searched, and a component search that rescanned all edges from every node.

* The live set is open addressing keyed by the header's address, at most half full counting
  removed slots: track and untrack are O(1). A spin lock on the runtime's own atomics guards it,
  since an attached host thread can make and release objects.
* The report maps objects to indices through a second such table, groups edges by owner, and
  finds the strongly connected components of the strong edges with an iterative Tarjan: linear
  in objects and edges, with no recursion a long chain of owners could overflow.
* Each component prints as before (objects, strong edges, static prediction, the suggested weak
  edge, the edges), under `warning[L3017]: reference cycle detected`, then `shortest cycles:`:
  for each object, a breadth-first search inside the component from it meets an edge back to it
  first on its shortest cycle; equal shapes are printed once with the objects that share them.
  At most 64 objects and edges of a component are listed and traced (a leaked million-node list
  is one component), and one search stops after 65,536 objects.
* `ember run` in the debug profile sets `--leak-check` unless `--no-leak-check`; the other
  profiles report only when asked. The error-page gate runs a runtime code's examples instead of
  checking them, and the test harness gains `#$ stderr:`.

No spec change.

## ADR-129 — the guaranteed count elisions `[RC-2c]` and `[RC-2d]`

2026-10-04, D-505, D-506, autopilot. `[RC-2]` guarantees no retain or release in five cases and
says each is tested by counting retains in the emitted C. Auditing Part VIII found RC-2b, RC-2c
and RC-2d without a test, and two of them not met:

* `[RC-2d]` (D-505): a handle stored into a field from a temporary is a move. The assignment
  (`[OWN-5]`: evaluate, drop the old value, store) read its own temporary as a copy, which
  retains a counted `Copy` type, and the temporary's end released it; it now moves the temporary
  in. A struct, tuple, array or enum literal does the same for a field or element made in a
  temporary (`lower_operand_kept`); a place is still copied and retained, since its owner keeps
  its own count. An ordinary operand stays a copy: a comparison or call that only reads a moved
  handle would leak its reference.
* `[RC-2c]` (D-506): a retain followed by a release of the same handle, with nothing between that
  could end the object, cancels. The `[RC-3]` uncounted-handle analysis already proved this for a
  copy out of a list the function owns; it now takes a local as the source too, when that local
  keeps a count of its own (a user-written local or `owned` parameter never assigned a copy, so
  no elision took its count away) and no path from the copy to the copy's end re-points, moves,
  drops or mutably lends it. A field written through it changes no count. Removing the pair
  moves no `drop` and no `Weak.upgrade` outcome (`[RC-3]`): the source's count stays above zero
  for the copy's whole life.

Each case has a test asserting the count in the C (`RC-2b/`, `RC-2c/`, `RC-2d/`). No benchmark
program's C changes. No spec change.

## ADR-128 — a name hoisted out of a branch is one variable with each arm's declaration of it

2026-10-04, D-490, autopilot. `[CTL-10]` declares a name set in every completing arm in the
enclosing block. The compiler kept each arm's local and copied it into the hoisted one at the
arm's end; when the arm had moved the value (into a constructor, an owned `match`), that copy read
it again: `E3040` on a valid program, at the declaration, and `L1001` on names the arm did read.
Each completing arm's local now records the hoisted local it declares (`hoisted_into`), through
any enclosing branch that hoists it again, and MIR lowering gives them one MIR local: the hoisted
name is declared (storage, scope) before the branch, and the arm's declaration only sets it. A
name an arm moved is moved after the branch on that path, so reading it there is `E3040` at the
read, as for any variable. No copy remains, so nothing changes for values that are not moved.
No spec change.

## ADR-127 — temporaries end where the source says: conditions, early exits, written-out evaluations

2026-10-04, D-488, D-503, D-504, autopilot. `[EXP-4]` ends a statement's temporaries at its end, a
condition's before the block it chooses and an iterable's with its loop; `[CTL-8]` has an early
exit run the pending `defer`s and then drop what the scopes it leaves own. MIR lowering kept one
list of the statement's temporaries and ended it only at the statement's normal end:

* A condition's temporaries were dropped after the chosen block, and a `while` reused its
  condition's temporary each turn and dropped only the last (D-503). The condition's value is now
  copied to a slot, its temporaries dropped, then the branch. A condition that makes no temporary
  with a destructor keeps its shape: storage-only temporaries cannot be told apart in the source,
  and the loop headers the analyses read stay as they were.
* A `return`, `break` or `continue` ended the locals of the scopes it left and none of the
  temporaries of the statements it left (D-504). Locals and temporaries are now numbered as they
  are registered, and an early exit drops both lists in one order, the last first: a statement's
  temporaries are registered after the locals around it and before those of its inner blocks, so
  the order nests. A `return`'s own temporaries end first, then the `defer`s, then the rest; a
  loop records the temporaries pending when its body starts, so a `break` or `continue` ends only
  those made inside it. As for locals, the lists are not shortened.
* A written-out evaluation (`ExprKind::Block`: `split_once`'s receiver and separator, a tuple
  compared field by field) was lowered statement by statement, so each hidden step ended its
  temporaries, and its bindings dropped with the hidden block. A separator's `String` died before
  the call that used its view (`E3060` on a valid program), and destructors ran before the source
  statement was over (D-488). Each of its statements is now a step of the source statement: its
  temporaries stay pending, and a hidden binding is registered as one of them. The other agent
  proposed a HIR marker per synthesized statement; it is not needed, because every
  `ExprKind::Block` is compiler-written (the AST has none) and the user's statements appear only
  inside its statements' own blocks (a loop body), which keep their statement ends.

The benchmark programs' C is byte-identical. No spec change: the spec already says this.

## ADR-126 — the runtime calls nothing in libm

2026-10-04, D-502, autopilot. glibc keeps `fmin`, `fmax` and `copysign` in libm, and GNU ld
decides `--as-needed` before `--gc-sections` removes the functions that call them, so one use in
the runtime made every Linux program load libm (about 25 µs at start-up, the whole of the empty
program's gap to C). The runtime's two uses are written with comparisons and `signbit`, exact for
every input they get; a program whose own C does maths links libm as before. MSVC and clang on
Windows are unaffected (the C runtime holds these functions).

## ADR-125 — the one-byte search falls through; only a view read from memory is copied by fields

2026-10-04, D-500 and D-501, autopilot. Every benchmark was built by the published compiler
(`e2cbba3`) and by this batch and timed interleaved (21 runs, performance cores, mains; gcc in WSL
on guest CPU 0). With MSVC two programs were slower than before the other agent's work.

**`lines()` (D-500).** ADR-111's one-byte search returned `hit ? hit - s.ptr : SIZE_MAX`; MSVC
made that a `cmov` after `memchr` and laid the non-empty case out as a taken jump, every line. The
same function with the found case falling through (`if (from < len) { ...; if (hit != NULL) return
...; } return SIZE_MAX;`) gives MSVC the previous loop, apart from one equivalent compare one byte
shorter. Measured on the same C: 69.7 → 65.1 ms (`split()` keeps 58.8 ms against 62.7 with the old
generic search). The early return alone still produced the `cmov`.

**`split_whitespace()` (D-501).** ADR-117 extended D-475's rule (a view copied from a place written
by fields is copied by fields) along every copy. Four of those copies in `split_whitespace`'s loop
read a plain local MSVC keeps in registers; made whole by hand they recover the previous 61.3 ms
(63.5 → 61.5). Built rule: copy by fields only when the source is read from memory, a projection or
a local whose address the body takes. A niche `Option`'s payload counts as a projection: MSVC keeps
an `Option` it tests by its fields in memory, and treating it as a register made `split()` 1.23x.

**Measured after both, against `e2cbba3`** (MSVC / clang): `split_whitespace` 0.998 / 0.997,
`split()` 0.927 / 0.98, `char_indices()` 0.93 / 0.94, `chars()` 0.96 / 0.93, `a05_structs` 1.00 /
1.00. `lines()` with MSVC is 1.045: the loop above is the previous one instruction for instruction
except that compare, and the previous compiler's own C on the new runtime moves by the same amount,
so the rest is the loop's placement in memory, not work.

## ADR-124 — a running total updated on only some turns keeps its check

2026-10-04, D-499, autopilot (the owner: "fix and improve issues with other agent's implementation").
`[SIMD-7]`'s block proof (ODR-086) runs a loop in blocks of 64 turns: an entry test, every value
added tested small enough, the block's total checked at its end, the block run again checked if a
test fails. That pays when the C compiler vectorises the block. A total updated only on some turns
(`if b == 111: count += 1`) puts a branch in the loop, which neither MSVC nor clang vectorises here,
so the blocks only add their tests: `a03_strings` 1.27x the previous compiler with MSVC, 1.02x with
clang, once ADR-109 let byte iteration reach the proof.

**Built:** in `group_overflow_checks`, after the unchecked copy (`unchecked_totals`: one entry
test, no per-block work) is tried and refused, a running total whose checked operation does not
dominate the loop's step keeps its check on each operation. An unconditional total keeps its
blocks. No benchmark shape is recognised: the rule reads the loop's control flow.

**Measured** (MSVC and clang, 21 interleaved runs on the performance cores, on mains): MSVC 86.5 →
57.0 ms (68.5 ms with ADR-109 reverted alone; ADR-123's append gives the rest), clang 68.7 → 67.5 ms.

## ADR-123 — expose checked String appends where the measured compiler benefits

2026-10-04, D-497. Implemented; focused native and whole-program validation pass.

StringPush emits a private header route. Under MSVC it exposes the existing
zero-count no-op, checked additional-byte reservation, memcpy and length update.
Under clang/GCC the name aliases the original exported ember_vec_extend call.
The public declaration, ABI and implementation remain unchanged. Arguments and
receiver still come from the same lowered temporaries; pointer/length reads add
no evaluation, borrow scope or ownership operation. No capacity check is removed.

Forcing that append body inline on every compiler is rejected: GCC t6 moves from
0.985 to 1.021 of C. The selected general compiler policy gives MSVC/clang/GCC
0.958/0.906/0.985, in six eleven-pair cycles with power/affinity evidence. The new
append regression is RED without the emitter redirect and passes every profile
on each compiler. It checks receiver/argument order, retained reserved capacity,
growth, empty append and Unicode/NUL bytes. Full current suites, normal-return
cleanup diagnostics and the wider speed audit remain outstanding; no README
number or acceptable-overhead conclusion follows. Raw receipts and uncertainty
are in TEXT-PERFORMANCE-2026-10-04.md.
