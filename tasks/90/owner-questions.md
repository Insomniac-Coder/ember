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
   until this is settled. **OPEN.**
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

Each has option (a) built; all **OPEN**.

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
