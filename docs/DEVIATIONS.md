# Implementation deviations

Where the compiler does not do what the specification says, on purpose and for
a recorded reason.

This is a short list by design. A deviation is a debt, and the standing rule is
that the compiler moves rather than the document: when the two disagree and the
rule is sound, the rule wins. Everything here is either waiting on a phase that
has not started, or waiting on a mechanism that does not exist yet.

**What does not belong here.** A rule with nothing built against it is a *gap*,
not a deviation — 833 rules are stated and 36 have conformance directories, so
gaps are the overwhelming majority and listing them would bury this. A deviation
is where something *is* built and behaves differently from what the rule says.

**How to read the fields.** *Observable today* is the one that decides urgency:
a deviation nobody can write a program to detect costs nothing until the feature
that would expose it lands, and the entry says which feature that is.

---

## D1 — `[RNG-5a1]` operators are a type-checker rule, not generated impls

| | |
|---|---|
| **Rule** | `[RNG-5a1]` |
| **Normative behaviour** | for every range type `T` over `R`, the compiler *generates* the impls `T: Add[T, Output = R]`, `T: Add[R, Output = R]`, `R: Add[T, Output = R]` and the `Sub`/`Mul`/`Div`/`Rem`/`Neg`/`PartialEq`/`PartialOrd` forms, in `T`'s declaring module |
| **Current behaviour** | operators on range types are decided in the type checker. No impls exist |
| **Soundness impact** | none |
| **Observable today** | **no** — and only because no operator interface exists at all. There is no `Add` in `std`, so nothing can bound a generic on it and nothing can observe whether the impls are there |
| **Behavioural agreement** | complete. `r + 0.25` yields `f32`, `Roughness + Metallic` is `E2214` by "ordinary overload resolution" as the rule requires, and `tests/conformance/RNG-5a1/` holds the pair |
| **Reason** | operator interfaces over scalars are not built. Generating impls into a table nothing reads would be scaffolding with no consumer |
| **Fix plan** | implement it *before* exposing operator interfaces to user generics, not after. The day `Add` is declarable, a user writing `fn sum[T: Add[T]]` will expect a range type to satisfy it, and it will not |
| **Owner** | ADR-016 |
| **Target** | with operator interfaces (Phase 3, block D) |

## D2 — `[CLO-6]`'s `owned f: fn(A) -> R` is refused, not implemented

| | |
|---|---|
| **Rule** | `[CLO-6]` |
| **Normative behaviour** | `owned f: fn(A) -> R` is a generic bounded by `CallableOnce`; calling it consumes it, and a second call is `E3040` under `[OWN-3]` |
| **Current behaviour** | the declaration is rejected with "not supported yet in this phase", naming `[CLO-6]` |
| **Soundness impact** | none — a refusal admits no program |
| **Observable today** | **yes**, as a rejection: a program the rule permits does not compile |
| **Reason** | the consumption is a property of the **bound**, not of the closure's type. The same closure is called repeatedly through a `Callable` parameter and once through a `CallableOnce` one, so it cannot be had by making the environment move-only — the call itself has to consume its callee, and calls do not do that yet |
| **Why refused rather than approximated** | treating `CallableOnce` as `Callable` would compile every such program and silently permit the second call the rule exists to forbid. A refusal is visible; a wrong acceptance is not |
| **Fix plan** | give a call the ability to move its callee, then the mode selects the bound as the rule says it already does |
| **Owner** | — |
| **Target** | Phase 3 |

## D3 — `extern class` parses and is refused

| | |
|---|---|
| **Rule** | `[FFI-39]`, and the production added as amendment A9 |
| **Normative behaviour** | a declared foreign base: sized, known layout, implicitly `open`, inheritable under `@ffi(trampoline, virtuals=[…])` |
| **Current behaviour** | parses, then `E1010` "not supported yet in this phase" |
| **Soundness impact** | none |
| **Observable today** | **yes**, as a rejection |
| **Reason** | the C++ importer, the thunks and the trampoline are Phase 7. None exists |
| **Why refused rather than ignored** | a declaration that parses, is stored, and is ignored by every later stage is the shape of three defects already found in this compiler. For a base class it would be the worst of them: the program would link and be wrong |
| **Fix plan** | Phase 7, with the importer |
| **Owner** | ERR-037 |
| **Target** | Phase 7 |

## D4 — `E9012` is registered and never emitted

| | |
|---|---|
| **Rule** | XX §6 names `E9012` under "manifest sections"; no rule assigns it |
| **Current behaviour** | registered, unemitted |
| **Soundness impact** | none |
| **Observable today** | no |
| **Reason** | `[DIA-6a]` requires every code the document names to be in the registry, and the document names this one. `[MAN-3]`, which briefly held it, now correctly reports `E9010` as its rule says |
| **Fix plan** | nothing to fix. It stops being unemitted when a rule claims it |
| **Owner** | ERR-039 |
| **Target** | — |

---

## Closed

Kept because a deviation that was closed is evidence about how long the next
one might last.

| | Rule | Closed by |
|---|---|---|
| **ADR-018** | `[CLO-3]` | `fn(A) -> R` in parameter position is now an implicit generic bounded by `Callable`, monomorphised per argument type; a capturing lambda is an anonymous struct of its `[CLO-2]` captures. It had been a C function pointer, which erases the environment, so every capturing lambda was rejected |
| **ADR-012** | `[LT-2]` | superseded. `E3064` was reachable all along and the programs it is for were being reported as shape B3; the classifier was wrong, not the narrowing |
| **ERR-026** | `[GRM-23]` | the compiler emits `E0104` as the rule says, and `E0102` stays with ordinary chained comparison |
| **ERR-039** | `[MAN-3]` | the compiler registers `E9010` as the rule says |
