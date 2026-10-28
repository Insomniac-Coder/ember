# Questions for the owner from the gap map (2026-10-27)

Each is asked in chat on its own, in plain words, with real code, when its milestone comes up or
sooner if the owner wants. Answers become verdicts in `tasks/96/redesign/JOB.md` (101 onward). Every
other question the areas raised is settled in `decisions.md`.

**Answered 2026-10-27:** Q-A (verdict 102), Q-B (verdict 103), Q-C (verdict 104), Q-D (verdict 105, for
now), Q-G (verdict 106), Q-F (verdict 107, 2026-10-28: (b) and (c) together). **Open:** Q-E (the owner could
not answer yet), Q-H and Q-I (not understood: explain again, one at a time,
with real code).

## Q-A (M1): `owned` inside `@ffi(...)` contracts

- **Where:** Part II's list of removed words (`p02-lexical.md:107-109`) says any use of `owned` is E1010.
  `[FFI-11]`'s table (`p16-ffi.md:272`) and its examples keep `owned` as the word for "C takes this
  object": `@ffi(param(widget, owned, one))`.
- **What the verdicts say:** verdicts 58 and 91 say an object C keeps goes to a parameter declared
  `owned`.
- **Proposal:** keep `owned` only inside `@ffi(...)` contract lists, and refuse it everywhere else.
- **Likely a confirmation.**

## Q-B (M4): may `shares` be made from a list element, a field or a plain parameter?

- **Example:** `players: Array[Player]`, then `watcher.target = shares mut players[1]`.
- **The problem:** `players[1]` has one owner, the list.
  - For a class object, sharing it means every end of a one-owner place must test the count instead of
    freeing at once.
  - For a value that is not a class, held inline in the list, there is no block to count. Sharing it
    would change the list's layout everywhere in the program.
- **The design tests** share only locals, new values and existing owners.
- **Options:**
  - (a) refuse, with a message to make the list `[shares mut Player(...)]` from the start;
  - (b) allow it for class objects only;
  - (c) allow it everywhere.
- **Lead's recommendation:** (b).

## Q-C (M9): after `h.join()`, may the parent change what the task used?

- **Rules:** `[THR-11]` and `[THR-13]` check a task's claims "until the join".
- **The question:** inside one `with scope:` block, may `counts[0] += 1` come after `h.join()`? Or do the
  claims last until the block ends, as today and in Rust?
- **Safety:** after the join, the task has finished, so allowing it is safe.
- **Lead's recommendation:** allow it.

## Q-D (M9): integer reductions and overflow

- **Rules:** `[THR-21]` says a whole-number reduction's total is "always the one adding one by one
  gives". But `[TYP-8]` stops the program on any overflowing `+=`.
- **The problem:** the tasks' partial totals can overflow where adding one by one would not, and the
  reverse.
- **Area 5's proposal:** partial totals wrap, and the join adds them in double width, so the program
  stops exactly when the true total does not fit.
- **The lead's caution:** a wrapped partial total has lost its high bits. Each partial total must then
  be kept in double width, not wrapped.
- **To decide:** whether "stops exactly when the true total does not fit" is the wanted meaning, rather
  than "stops where one-by-one adding would".

## Q-E (M9): handing an object to another thread while a `weakref` on this thread points at it

- **Rule:** `[THR-2]` makes it E7015 when compiling.
- **The problem:** links can be stored anywhere, so an exact check is possible only inside one
  function.
- **Options:**
  - (a) refuse wherever a `weakref` to an object of that class may be alive on this thread
    (conservative, whole program);
  - (b) one compare of the weak count at the hand-over, which stops the program.

## Q-F (M10): the word each arena-made value carries

- **Verdict 47** accepted the word for now. The measuring phase was to look for ways to drop it.
- **What was measured:** see `tasks/96/redesign/measure-0.9.15/RESULTS.md`, section "arenas".
- **Options:**
  - (a) keep the word;
  - (b) an arena kind that frees nothing one by one needs no word;
  - (c) memory laid out so a block finds its arena (aligned chunks with a header);
  - (b) and (c) together.

## Q-G (M11): how is a contract written on a function type?

- **Rule:** `[EFF-2]` says "a contract written on a function type is checked against every function
  stored into it".
- **The problem:** the grammar has no place for it.
- **Options:**
  - (a) the contract above a `type` line: `@noalloc` over `type Handler = fn()`;
  - (b) inside the type: `fn() @noalloc`;
  - (c) drop the sentence.

## Q-H (M11): a private generic that writes some of its bounds

- **Rule:** `[IFC-6]` and `[TYP-17]` infer a private generic's bounds from its body.
- **The question:** when it writes some bounds, are the others still inferred? For example,
  `[T: Lookalike]` above `fn direct_add(a: T, b: T) -> T: return a + b`, where `Lookalike` does not
  declare `+`.
- **Options:**
  - (a) `+`'s interface is added to the written bound;
  - (b) it is refused (E2072: no written bound declares `+`).

## Q-I (M14): naming one type in an ECS view

- **Rule:** `[TYP-27]` says a tuple type has two or more elements (ODR-071). ECS-3 and ECS-4 write
  `registry.view[(A,)]()` and `.exclude[(C,)]()`.
- **Options:**
  - (a) allow the one-element tuple `(A,)`, as Python writes it, with `(A)` still meaning `A`;
  - (b) a separate form for one type, `registry.view[A]()`, with tuples for two or more.
