# Owner questions from the 90% push (2026-10-19/20)

Real choices the specification leaves open, raised by the helpers and kept here for the owner. What
was built meanwhile is in force until the owner decides. Each question is written out in full, with
the specification quoted, a real example and the options, in the helper's notes.

## Layout attributes (`[LAY-2]`), helper L — `tasks/90/L-notes.md` §4

1. `@packed` together with `@align(N)`, on one type or a packed type holding an aligned one: packed
   wins, `@align` wins, or both refused. Built meanwhile: `E0901` for both.
2. `@repr` on a struct: does not apply (`E0104`, the attribute table's row split), or a meaning to
   give it. Built meanwhile: `E0901`.
3. A reference to a one-byte-aligned field of a `@packed` type: the code table says `E2170` covers any
   field of a packed struct, the rule's text only unaligned ones. Built meanwhile: allowed (the
   rule's text).

## ECS (`[ECS-3]`..`[ECS-6]`), helper M — `tasks/90/M-notes.md` "Owner questions"

1. `run_parallel` runs its systems one after another: build real parallel runs (and decide when a
   function value or callable parameter is `Send`/`Sync`, and whether components must be), rule
   that running in turn meets "at once", or parallel for literals only. `[ECS-4]` stays unfinished
   until this is settled.
2. `run(system, args…)` without variadic parameters: one argument (a tuple for several, `()` for
   none, built), a separate no-argument form, or a language feature.
3. `Not[D]`'s item: `()` (built, a `()` slot in the item tuple) — confirm.
4. A query with no required parameter: panics when iterated (built), iterates every live entity, or
   a compile-time error.
5. Part XII.3 "no compiler support of its own" against `[ECS-4]`'s compile-time `E7020`: keep the
   narrow support built and say so, a general mechanism, or drop the literal form.
6. `Entity` as a query parameter and `Option[P]` over any parameter (built, so that
   `q.commands().destroy(e)` has an `e`) — confirm.
7. D-690: lend `self` to a shared call as a read (built), let the callee's writes through, or lend
   exactly the method's live loans.
