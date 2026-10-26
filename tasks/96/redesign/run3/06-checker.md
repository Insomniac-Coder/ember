# 06 checker: report (run 3, phase 1)

Power: `Online` at the start and at the end.

## What I wrote

| file | what is in it |
|---|---|
| `RUN3/vocabulary.md` | the approved table: the builder's draft copied with `cp`, then checked row by row (255 rows, same columns). Each row I changed carries **06:** in its verdict cell; section 4 (hand-over to the next agents) is rewritten; section 6 lists every change (C-1 .. C-10) with the test it failed, the evidence, and the spec text that keeps an old meaning (for 15) |
| `target/redesign3/06-checker/k1..k10_*.em`, `check_output.txt` | the probes behind my claims, with the full `ember check` output of each |
| `target/redesign3/06-checker/apply_rows.py` | the script that made the row edits in my copy (so every edit can be seen in one place) |

## Verdicts after the check

224 keep, 15 rename, 9 merge, 2 remove, 5 new (the builder's totals were the same; two renames moved:
`Sync` the property and `cstr` became renames, `@nosync` and the effect `Sync` became keeps).

## The changes, in short (section 6 of `vocabulary.md` has the reasons and the evidence)

1. **C-1, the biggest one: the in-out parameter is `ref mut b: B`, not `mut b: B`.** Row 8's own
   test applied to a parameter: `mut b` says "may change" and nothing says "the caller's own `b`".
   Today `mut` on a parameter also means two things by type (struct: change fields; class: re-point
   the handle, "no access on the object", `[FN-9]`), while `mut self` on a class changes the object.
   One rule now: every name that changes someone else's value is written `ref mut name` (parameter,
   lambda parameter, pattern, loop name); `mut self` stays, because `self` says what it is. This also
   answers AN-20 with no new word. Today `fn bump(ref mut x: int)` does not parse (E0100), and
   `for ref mut v in xs` parses but binds a read-only reference (E3021 at the write). Cost: 22 spec
   and 114 `std` parameters change spelling.
2. **C-2: `owned` is refused where the type is known to be `Copy`** (`owned n: int`, `with owned
   score` of an `int`), because there it can only copy, which is the owner's named failure; the draft
   kept it. `x: owned T` as a parameter's type without `owned` on the name is refused (it reads as
   "take it" and means "read it").
3. **C-3: `Copy` means an independent copy only**; class handles, `shared` values and `Weak` are not
   `Copy`. So `owned h: Node` hands the handle over (today it is a share the caller keeps), and AN-44
   (area C) is answered under that meaning: `p2 = player` may not share silently.
4. **C-4: settlement 1 overturned.** The property `Sync` becomes `ThreadShared`, the same family as
   `@thread_shared` (one meaning, one word); the effect keeps `Sync` and `@nosync` stays, since "no
   sync" says exactly what it forbids. The draft's `Synchronise` would have left `Sync` and its own
   long form meaning two things.
5. **C-5:** `cstr` -> `CStrView`, with `str` -> `StrView`. **C-6:** `get` for `upgrade` only if it keeps
   nothing alive, like every other `get`. **C-8:** `let` reworded as one rule for every type. **C-9:**
   `shared dyn I` is already in the vocabulary (AN-66 needs no new word). **C-10:** `Cell` kept, with
   the plain-name default condition for area B.

The agreed rows 3-9 are unchanged (checked word by word against `inputs/JOB.md`).

## For the lead (outside the words)

`Span.to_array()` in `std/src/core.em:988` cannot be called at all today (E3063 inside the library;
`k8_copied_span.em`). It is a library defect, not a vocabulary question; noted here only.

## Points the owner may want to see first

- C-1 changes a spelling used in many places (`mut b: B` -> `ref mut b: B`). It follows from the
  owner's own reasoning on row 8, but it is the change with the widest reach.
- C-4 reverses the builder's settlement and sides with reader 03.
