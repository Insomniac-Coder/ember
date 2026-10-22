# Owner questions from the 90% push (2026-10-19/20)

Real choices the specification leaves open, raised by the helpers. On 2026-10-20 all but one were
ruled under the delegation (ODR-138..ODR-143, 0.9.10_Hardened_21, ADR-225), each the owner's to
overturn; **ECS question 1 and the Box question are open.** Each question is written out in full, with
the specification quoted, a real example and the options, in the helper's notes.

## The C header importer (`[FFI-6a]`, `[FFI-6b]`, `[CLI-5]`, `[FFI-30c]`), helper N — `tasks/90/N-notes.md` §4

Ruled under the delegation 2026-10-20 as ODR-145..ODR-149 (0.9.10_Hardened_23, ADR-231), each the
owner's to overturn.
1. A pointer-constant macro (`#define NIL ((void*)0)`, `((HANDLE)-1)`): `[FFI-6a]` imports it with its
   C pointer type, `[CT-5]` forbids a pointer `const`. Built meanwhile: skipped with `W5001`. **Ruled ODR-145: a
   pointer made from an integer is a compile-time value; imported as a `const`.**
2. A macro that expands to nothing (include guards): a `W5001` each, as written, or not reported.
   Built meanwhile: not reported. **Ruled ODR-146: not reported.**
3. A function-like macro's signature (`[FFI-6b]`, `@ffi(macro_fn)`): the overlay grammar gives an
   entry no types. Built meanwhile: nothing. **Ruled ODR-147: a `macro_fn` entry writes Ember types.**
4. Where `ember bind --init` writes its starter overlay (`[CLI-5]`). Built: `<stem>.overlay.em` beside
   the header. **Ruled ODR-148: as built.**
5. Several overlays for one module, their order and `override` (`[FFI-30c]`). Built meanwhile: one
   overlay file per import. **Ruled ODR-149: a list in order, `override`, else `E5010`.**

## Box (`[STR-5]`, IX.1), found 2026-10-20 (ADR-226)

1. Is `Box[T]` `Clone` and `Eq` when `T` is? IX.1 gives `Box[T]` "one owner; `Box(v)`, auto-deref,
   `b.get()` … Move-only" and no `Clone` or `Eq`, so by `[STR-5]` a struct or enum with a `Box` field
   derives neither: `struct P: b: Box[int]` has no `p == q` (`E2040`) and no `p.clone()` (`E1010`),
   and a recursive enum (`Node(Box[Tree])`) can be neither compared nor cloned. (`b.clone()` and
   `b == c` on a `Box` itself compile, by auto-deref, and act on the payloads.) Options: (a) as
   built, neither; (b) `Box[T]` is `Clone` (a new box holding a clone) and `Eq` (payloads equal)
   when `T` is, so derives see through it — a recursive type's derived `eq` then has to be a
   function, not the inline comparison built today; (c) `Clone` only. **OPEN.**

## Layout attributes (`[LAY-2]`), helper L — `tasks/90/L-notes.md` §4

1. `@packed` together with `@align(N)`, on one type or a packed type holding an aligned one: packed
   wins, `@align` wins, or both refused. **Ruled ODR-138: both refused, `E0104`.**
2. `@repr` on a struct: does not apply (`E0104`, the attribute table's row split), or a meaning to
   give it. **Ruled ODR-139: does not apply, `E0104`; the table row split.**
3. A reference to a one-byte-aligned field of a `@packed` type: the code table says `E2170` covers any
   field of a packed struct, the rule's text only unaligned ones. Built meanwhile: allowed (the
   rule's text). **Ruled ODR-140: the rule's text; the code table follows it.**

## ECS (`[ECS-3]`..`[ECS-6]`), helper M — `tasks/90/M-notes.md` "Owner questions"

1. `run_parallel` runs its systems one after another: build real parallel runs (and decide when a
   function value or callable parameter is `Send`/`Sync`, and whether components must be), rule
   that running in turn meets "at once", or parallel for literals only. `[ECS-4]` stays unfinished
   until this is settled. **ANSWERED 2026-10-22 (a): real parallel runs; a function value is `Send`
   when everything it captures is `Send` (a plain function captures nothing, so it is); a component
   must be `Send` to be used by `run_parallel`.** (ODR-218; not built yet.)
2. `run(system, args…)` without variadic parameters: one argument (a tuple for several, `()` for
   none, built), a separate no-argument form, or a language feature. **Ruled ODR-141: one argument.**
3. `Not[D]`'s item: `()` (built, a `()` slot in the item tuple) — confirm. **Ruled ODR-141: `()`.**
4. A query with no required parameter: panics when iterated (built), iterates every live entity, or
   a compile-time error. **Ruled ODR-142: every live entity (built 2026-10-20).**
5. Part XII.3 "no compiler support of its own" against `[ECS-4]`'s compile-time `E7020`: keep the
   narrow support built and say so, a general mechanism, or drop the literal form. **Ruled ODR-141: keep it, Part XII.3 says so.**
6. `Entity` as a query parameter and `Option[P]` over any parameter (built, so that
   `q.commands().destroy(e)` has an `e`) — confirm. **Ruled ODR-141: confirmed.**
7. D-690: lend `self` to a shared call as a read (built), let the callee's writes through, or lend
   exactly the method's live loans. **Ruled ODR-143: lent as a read.**

## `import cpp`, helper N3, 2026-10-20 (ADR-245) — `tasks/90/N3-notes.md` §5

Each has option (a) built; all **OPEN** except 3, 4 and 8, which the owner answered 2026-10-21:

- **Q3 ANSWERED (FFI-40a): none of (a)-(d); instead the compiler checks.** For a `const` method of a
  class with a `mutable` field, the importer reads the method's body when the header has it: a write
  to a field (assignment, `++`, a changing call, a call it cannot see) makes it `mut self`, none
  `self`. A body it cannot see is `mut self`. The overlay may restate a method in a `class C:` section
  as an ordinary Ember method line (`fn lookups(self) -> int`), which wins (an asserted fact, listed
  by `ember tcb`). Real example: `tests/conformance/FFI-40a`, where `cache.get().lookups()` is then
  accepted and `lookup()` still needs `get_mut()`. BUILT 2026-10-21 (ADR-253).
- **Q4 ANSWERED (FFI-41): (b).** A static data member (and a C++ namespace variable) reads and
  writes like a global, `cpp.engine.Grid.made += 1`; underneath, each use still goes through the
  address thunk that exists today. BUILT 2026-10-21 (ADR-253); one holding a pointer is not
  imported (ODR-163).
- **Q8 ANSWERED (BLD-FFI-5a): (b).** An object of a `@sync` class may be handed to C++ by an
  `@export` function (FFI-31b gains that exception); the class needs no `@export` of its own, it
  reaches the `--cpp` header through the exported function, with its `pub` methods; C++ sees an
  ordinary class of the same name whose copies share the object and whose last copy frees it. A
  class without `@sync` is still refused ("mark `World` `@sync` to hand it to C++"). NOT BUILT YET.

1. How an instantiation is named (`[FFI-17b]` gives none): (a) the template-id's characters joined
   by `_` (`geo::Pair<int>` is `geo.Pair_int`); (b) an overlay `rename` for each; (c) another spelling.
2. How an overlay declares a function throwing (`[FFI-43]` names no spelling; `[FFI-24b]` spells the
   assertion `@ffi(noexcept)`): (a) `@ffi(noexcept=false)`; (b) `@ffi(throws)`.
3. How an overlay names a method, for `[FFI-40a]`'s `@ffi(invalidates)` (`[GRM-35]`'s `overlay_fn`
   takes one name): (a) not possible: the class-wide assumption only; (b) `fn Cache.lookup(…)`;
   (c) entries inside `extend Cache:`; (d) `@ffi(invalidates=[lookup])` on `type Cache`.
4. A static data member (`[FFI-41]`), whose mangled symbol `[FFI-3]` forbids and whose address in a
   static `E7002` refuses: (a) an associated function giving its address, `Grid.made() -> *mut c_int`;
   (b) an `extern static` of the address exempt from `E7002`; (c) amend `[FFI-41]` to say (a).
5. Nested types "under their outer type's name" (`[FFI-41]`), Ember having none: (a) `Outer_Inner`;
   (b) a child module per class (`cpp.Outer.Inner`).
6. Composed overlays that disagree on a policy (`E5062`, `[FFI-43]`, beside ODR-149's `E5010`):
   (a) a second entry without `override` declaring the opposite policy is `E5062`, the same or none
   `E5010`; (b) every second entry `E5010`, `E5062` only within one entry.
7. `[FFI-24c]`'s `ember bind --report` reads a header, not a program, so it cannot see `@noalloc` or
   main-thread paths: (a) not built; (b) the report belongs to `ember build --report`; (c) list every
   catching function.
   **ANSWERED 2026-10-22 by the owner: the list goes in `ember inspect --cost`** (which reads the built
   program and already prints every C++ call's mode and place): it also marks the catching calls inside
   `@noalloc` or main-thread code (ODR-220; not built yet).
8. `[BLD-FFI-5a]` beside `[FFI-31b]`, which keeps class handles out of exported signatures: (a) not
   built; (b) `[FFI-31b]` allows `@sync` handles in exports with the `--cpp` header; (c) a later version.
9. A defaulted special member C++ makes `noexcept(false)` (`[FFI-24]` lists it as non-throwing):
   (a) direct, as the rule says (a throw ends in `std::terminate`); (b) Clang's answer, catching when
   C++ says it may throw.

## The importer's second job (helper N), 2026-10-20 (ADR-238) — `tasks/90/N-notes.md` Part 2

1. Inside a `const` initialiser, an integer-to-pointer cast (`0 as *mut T`, ODR-145) needs no
   `unsafe`. **Ruled under the delegation: as built** (the value is a number until it is used, and
   using it is what `[UNS-1]` guards).
2. Declarations of one C function that disagree (what they pass or return) are now caught by the
   type checker, since C no longer compares them (D-743/D-744). Built with `E0104`. (a) keep
   `E0104`; (b) widen `E5011` to functions; (c) a new code. **OPEN** (a new code is the owner's).
3. An `override` overlay entry with nothing earlier to replace is `E5010`. **Ruled under the
   delegation: as built.**
4. `E5015` now also covers a type with no foreign representation (it was one of `E5050`'s two
   meanings, D-745). (a) keep it, and correct its code-table row, which cites `[FFI-31]` (in 0.9.10
   the cdylib runtime rule) where it means `[FFI-5]`/`[FFI-31b]`; (b) a code of its own. **OPEN.**

## The owner's answer of 2026-10-21 on C++ variables outside any class

- **ANSWERED (b):** a C++ variable outside any class (`inline Node* kept_node = nullptr;` in
  `tests/conformance/FFI-17c/nodes.hpp`) becomes an Ember variable, like a class's `static` member:
  `cpp.kept_node = node`, `cpp.recorded_events += 1`, every use inside `unsafe:`. NOT BUILT.
- **NEXT QUESTION, not yet asked:** the "no pointer" rule (`E7002`) refuses an Ember variable holding a
  pointer, so it breaks a C file's variable (Part XVI), a C++ class's `static` member (question 4) and
  now these. Whether it stops applying to variables from C/C++ files is the owner's to decide.
- All 96 ODRs ruled by Claude under the delegation are OPEN pending the owner's review (owner,
  2026-10-21); ODR-163 is rejected.
- **ODR-021 REVERSED by the owner (2026-10-21):** "Ember will not give Python's garbage answers for those two
  operators": float `//` and `%` are not Python's (`1.0 // 0.1` is not 9, `1.0 % 0.1` not
  0.09999999999999995). They follow the formula `[TYP-29]` wrote, `floor(a / b)` and `a - b * floor(a / b)`,
  each step rounded as the machine rounds it (`1.0 // 0.1` is 10, `1.0 % 0.1` is 0.0). Integer `//` and `%`
  are not part of ODR-021. `[PHIL-14]` ("Python spelling, Python meaning") is not the owner's rule (added in
  0.9.9). BUILT 2026-10-22 as ODR-164 (runtime, compile-time evaluator, `[TYP-29]`'s text).

## The owner's review of Claude's delegated rulings, 2026-10-21

The owner reviewed all 91 rulings Claude made under the delegation (`docs/reviews/odr-review-2026-10-21.pdf`)
and **accepted all of them except ODR-021 (reversed: decimal `//` and `%` follow `[TYP-29]`'s formula, not
Python) and ODR-163 (rejected: a variable holding a pointer is not left out of an import).** Accepted
(89): ODR-016, ODR-022, ODR-023, ODR-024, ODR-025, ODR-026, ODR-027, ODR-028, ODR-029, ODR-030, ODR-031, ODR-032, ODR-033, ODR-034, ODR-035, ODR-036, ODR-037, ODR-039, ODR-040, ODR-041, ODR-042, ODR-043, ODR-044, ODR-045, ODR-046, ODR-047, ODR-048, ODR-065, ODR-070, ODR-071, ODR-072, ODR-073, ODR-074, ODR-075, ODR-076, ODR-077, ODR-078, ODR-079, ODR-080, ODR-081, ODR-082, ODR-083, ODR-084, ODR-089, ODR-090, ODR-091, ODR-092, ODR-093, ODR-095, ODR-096, ODR-097, ODR-098, ODR-101, ODR-102, ODR-103, ODR-104, ODR-105, ODR-106, ODR-107, ODR-108, ODR-109, ODR-110, ODR-111, ODR-112, ODR-132, ODR-133, ODR-134, ODR-135, ODR-136, ODR-137, ODR-138, ODR-139, ODR-140, ODR-141, ODR-142, ODR-143, ODR-144, ODR-145, ODR-146, ODR-147, ODR-148, ODR-149, ODR-150, ODR-151, ODR-152, ODR-155, ODR-157, ODR-158, ODR-159. The delegation itself stays revoked: new spec questions go to the owner.

## The owner's ruling replacing ODR-163, 2026-10-21

- A C++ variable outside any class (`inline Node* kept_node = nullptr;`) becomes an Ember variable
  (`cpp.kept_node`), read and written inside `unsafe:`, numbers and pointers alike: nothing is left out
  of an import.
- The `E7002` "safe to share between threads" check does not apply to such a variable (every use is
  already inside `unsafe:`). A C++ class's `static` member is exempt already (question 4, (b)).
- **The same holds for a variable of a C header** (`extern struct Node* kept_node;` with `import c`):
  imported, used inside `unsafe:`, not refused by `E7002` (owner: "yes").
- BUILT 2026-10-22 (ADR-253, spec ODR-163 in `0.9.10_Hardened_31`).

## The delegation restored, 2026-10-22

The owner restored the delegation to rule spec ambiguities, on the condition: "do not opt for cheap
solutions that compromise on quality and use elegant, high performance solutions to the problems"
(`docs/AUTOPILOT.md` §3).

