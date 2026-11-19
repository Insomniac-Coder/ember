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
| CLS-7 | a method that changes `this` is not called through a name that only reads it (`E3023`). `fn f(n: Node): n.set()`, `n.v = 1` and the same through `ref n: Node` are accepted: the object is still guarded by the run-time access words | M6a, M6b |
| CLS-9a | a view field `from` a `let` field is read in `drop`: view fields with `from` are not parsed yet | M5 |
| CLS-12 | `@cached` | M11 |

## Shared owners and links

| Rule | What is missing | Milestone |
|---|---|---|
| RC-2 | its item (b): opening a `weakref` still counts (`RC-2b`) | M5 |
| RC-7 | `sharedref mut`, and the access words | M5, M6b |
| WK-1 | `E3032` | M7 |
| WK-3 | the object is ended before its `drop` runs, seen through a link opened in `drop` | M7 |
