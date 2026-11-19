# Credit pass after M4 (2026-11-19)

A rule is credited only when every clause is built and tested (the owner: "it only counts as
implemented if the compiler also actually works that way"). This file lists the rules looked at and
not credited, with the clause that is missing and the milestone that builds it.

## Classes

| Rule | What is missing | Milestone |
|---|---|---|
| CLS-1 | `Enemy(hp=5) using current`: `using` does not exist yet | M10 |
| CLS-2 | `init` may call a method once the fields its body reads are set: today every call of a method before all fields are set is `E2100`. The check reads each override's body; in a class that is not `pub` an override that reads nothing unset is accepted | M11 |
| CLS-4 | `E2112` is still the warning `W2111`; `E2113` (an `override` whose first line differs) is registered and never emitted, so a differing `override` is accepted; the header promise of `[THR-8]` for `pub open` classes | M11, M9 |
| CLS-6 | the kept results of `@cached` methods end with the fields: `@cached` does not exist yet | M11 |
| CLS-7, FN-9 | a plain class parameter only reads its object (`E3023` for a change through it). W8 built it, and it is taken out again: std.ecs then needs `ref mut this` given to a call (`Q.Ops.fetch(ref mut this)`, `change.apply(ref mut this)`), which `[CLS-7]`'s own example allows (`this.handlers[this.current](ref mut this)`) but which is sound only with `[BRW-21]`'s `E3031` for a call that may end the object, from the callees' summaries; today every `ref mut this` argument is `E2103`. W8's commits `edf40eb4` and `5faca934` land again with that | M6a |
| CLS-9a | a view field `from` a `let` field is read in `drop`: view fields with `from` are not parsed yet | M5 |
| CLS-12 | `@cached` | M11 |

## Objects and dispatch

| Rule | What is missing | Milestone |
|---|---|---|
| OBJ-1 | the 8 and 12-byte headers, and the 32-byte `thread_shared` header | M6b |
| OBJ-2 | `s: Shape`, a bare interface name as a parameter's type, is accepted; it is `E2173` | M11 |
| OBJ-4, OPT-1 | `using`; a `thread_shared` value's block from the runtime's chunks | M10, M6b |
| OBJ-5 | a method `drop` calls that makes an owner of `this` is not refused (`E3016`): it stops the program instead; needs the call summaries | M6a |
| DSP-2, DSP-3 | the fact `[LT-22]` records, kept beside each table entry | M6a |
| DSP-4 | kind tests: `if s is Circle as c:` is `E0900`, `E2036`, `E2037` and `E2106` follow it | M11 |
| DSP-5 | what a virtual call may change, worked out from every override (the change half of `[CLS-7]`) | M6a |
| DSP-6 | the stop's text (`` `cast s as Circle` on line 9: ... ``), a cast as a place, `E3027` | M11 |

## Inference

| Rule | What is missing | Milestone |
|---|---|---|
| TYP-23 | a lambda given to a generic helper whose result is a `Conditional` of `Ok(…)` and `Err(…)` (`fn(owns v) => Ok(v) if v > 0 else Err(String("no"))`) does not take the expected result's known parts: only a body that is one `Ok(…)`, `Err(…)` or `Some(…)`, or returns one, does (W8's `wrapper_hint`, a narrow form of the rule's "expected types flow into `Ok(…)`") | the inference rework |

## Ownership

| Rule | What is missing | Milestone |
|---|---|---|
| DRP-1 | a view field `from` a `let` field is read in `drop` | M5 |
| DRP-6 | a `sharedref mut` link | M5 |
| DRP-8 | ending a value counts as a call of its `drop`, for every rule that needs it | M6a |
| HEAP-1 | `using` | M10 |
| HEAP-9 | `copy of` an `owns dyn I` whose interface needs `Clone` is `E2116` | M11 |
| HEAP-10 | a `thread_shared` value's two counts | M6b |
| OWN-7, OWN-8 | `sharedref mut` links, and `E2116` for a value whose views point into its own `let` field | M5 |
| OWN-11 | a name opened with `exists as` over a call's result is a view: `done.push(owns job)` in `while jobs.pop() exists as job:` should be `E3013` with the help `match owns jobs.pop():`, and is accepted | M5 |

## Shared owners and links

| Rule | What is missing | Milestone |
|---|---|---|
| RC-2 | its item (b): opening a `weakref` still counts (`RC-2b`) | M5 |
| RC-7 | `sharedref mut`, and the access words | M5, M6b |
| RC-1 | `thread_shared this` as the word that pins every object of a class (`[OWN-13]`) | M9 |
| RC-3 | `with h:` as the statement that keeps `h` alive: today it hands `h` over (`E3040` at the next use) | see below |
| RC-5 | the checks that no line or call in a view's life may end an object of that family | M5, M6a |
| WK-1 | `E3032` | M7 |
| WK-5, WK-6, WK-7 | the graph of owners as `E3032` reads it, the refusal and its proofs | M7 |
| WK-11 | a scoped task touches a link only through `weakref thread_shared T` (Part XI) | M9 |
| WK-12 | opening a `weakref thread_shared T` with the home and away counts | M6b |
| WK-15 | "nothing leaks through a loop of owners" rests on `E3032` | M7 |
| WK-16 | `weakref` of a `shares dyn I` (`shares dyn` is not built) and of a `some I` | M11 |
| WK-3 | a link opened in `drop` already finds the object ended; what is missing is a `thread_shared` value ended at its making thread's next check point (`[RC-4]` points 4 and 5), which needs biased counting | M6b |
