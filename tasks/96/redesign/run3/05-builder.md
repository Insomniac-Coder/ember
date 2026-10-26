# 05 builder: report (run 3, phase 1)

Power: `Online` at the start and at the end.

## What I wrote

| file | what is in it |
|---|---|
| `RUN3/vocabulary-draft.md` | one table, one row per word (255 rows): every word on `inputs/must-cover.md` (checked by script: none missing, no word twice), plus the words the readers added and the new words of the agreed rows. Columns: word; today it means; it will mean; where it is written; replaces; stays refused; verdict and reason. Section 1 lists the agreed rows 3-9 with their words, unchanged. Section 2 settles 20 disagreements between readers. Section 5 holds the evidence for the words I judged myself |
| `RUN3/annoyances.md` | 84 annoyances, AN-01 .. AN-84, each with a title, the program's intent in one line, where it was found (file and line), exactly one area, and whether an agreed row settles it. A 16, B 17, C 19, D 18, E 14; 18 settled by an agreed row (or by the owner's `owned`-replaces-`Box` ruling or "delete after the loop"). Every phase-1 id (A1..A34), every item of `FINDINGS-phase3b.md` section 5 and every defect of `FINDINGS.md` section 5 has one home |
| `target/redesign3/05-builder/*.em`, `check_output.txt` | the probes behind the claims I added, and the full `ember check` output of each |

## Verdicts

224 keep, 15 rename, 9 merge, 2 remove, 5 new.

- **New** (agreed rows): `copied` (rows 6, 9), `shared` (row 9), `changes` (row 3), `reads` (row 7),
  `String(...)` (row 5's message). `owned T` as the one-owner type (owner's ruling) is a new use of
  `owned`, in its row.
- **Remove:** the `owned fn` prefix (the owner's named failure); `@no_runtime_checks`.
- **Merge:** `Box` and `Box[dyn I]` into `owned`; `Shared` and `SyncShared` into `shared`; `.clone()`
  and `.copy()` into `copied`; `Mut[T]` into `ref mut`; `as?` and `as!` into `cast` (the compiler's
  spelling; the spec text is behind it).
- **Rename** (placeholders for 06 and 07): `str` -> `StrView`; `RefCell` -> `RuntimeChecked` with
  `Ref`/`RefMut`/`borrow`/`borrow_mut` -> `ReadGuard`/`WriteGuard`/`read`/`write`; `ForeignBox` ->
  `Foreign`; `upgrade` -> `get`; `assert_disjoint` -> `check_disjoint`; `@sync` -> `@thread_shared`;
  `@nosync` -> `@nosynchronise` and the effect `Sync` -> `Synchronise`; `@static_safe` ->
  `@no_alias_checks`; `@fp(contract)` -> `@fp(fuse)`; `@borrows(p)` -> a `from p` clause after the
  result type.
- **Kept with a changed meaning** (06 must check each): `let` (contents fixed too, as Swift's `let`),
  `self` (read-only for every type), `mut self` (one meaning: may change any field), `owned`
  (hand-over for every non-`Copy` type, plus `Box`'s jobs), `mem.drop` (ends the name for every type),
  `MutSpan` (writable without `mut`), `SoA.get` (a reference, not a copy).

## The two must-cover words no reader judged

- **`ref mut`: keep.** 52 uses in lines 1-6930, in four positions (type, expression, pattern, result)
  and a local whose assignment writes through. One meaning everywhere: a name for someone else's place
  that may change it (row 8). Two forms do not do what the words say and are refused in the draft:
  `x: ref mut T` as a parameter's type (reader-1's t3/t3c) and `r = ref mut b` on a reference local,
  which today's compiler accepts and which copies `b` into `r`'s first target (`b2_ref_mut_rebind.em`,
  exit 0). A call returning `ref mut` is not a place today (`first(xs) += 1` is E2140,
  `b1_ref_mut_forms.em`).
- **`some I`: keep.** `[TYP-32]`, `[GRM-31]`; the spec's example compiles (`b3`); E2260 (`b4`) and
  E2261 (`b5`) give a line to write. One meaning, plain English, on the return type it describes.

## Choices a reader of the draft should look at first

1. Settlement 1 renames `@sync` although three readers kept it: the spec itself must forbid the
   word's natural reading (`[THR-7]`, 3473-3475).
2. Settlement 17 keeps `owned` on a parameter copying a `Copy` value (as `b = a` does) instead of
   ending the giver's name for every type; the class-handle case goes to area C with AN-44.
3. `copied` is refused on a view (`str`, `Span`) with the advice `String(word)`, so that `copied` and
   row 5's `String(...)` never mean the same thing.
