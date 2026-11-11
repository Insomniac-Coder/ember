# The milestone map refreshed for 0.9.20_Hardened_12 (Step 0.2 of PLAN-0.9.20.md, 2026-11-11)

**Inputs.**
- `diffs-01.md` to `diffs-08.md` and `diffs-xx.md`: word-level diffs of the 383 rules changed or new
  since 0.9.15_Hardened_1, which `MILESTONES.md` was written against. They were made by a script
  (the session scratchpad's `refresh_diffs.py`). `index.tsv` lists the rules.
- `xx` holds the 118 rules no gap file lists, because they had not changed in 0.9.15.

**Classification.** Four Sonnet 5.5 workers, read-only, about 870,000 tokens in all. Their tables:
`class-01-02-08.md`, `class-03-04-07.md`, `class-05-06.md`, `class-xx.md`.

| | rules | wording | meaning | new |
|---|---|---|---|---|
| all | 383 | 68 | 291 | 24 |

**What the lead checked.**
- Each finding below that changes the plan, or that means built code must change now, was read
  against the spec text and the compiler source.
- The other rows are the workers'. Each milestone reads its rules whole against the pinned spec when
  it is built (`MILESTONES.md`, "every MUST of the rules it touches is built"), so a wrong
  classification there is caught then.

## Changes to the plan (checked)

1. **The object header rulings S-1 and S-3 are out of date.** `[BLD-16]`, new in 0.9.20, says that
   no fact of the whole program decides a package's layout. Whether a type carries the weak count is
   decided by the package that declares it (`[OBJ-1]`, `[WK-5]`, `[WK-7]`). A type index is given at
   link time (`[TYP-44a]`). So M6b's header is designed from `[OBJ-1]`, `[BLD-16]` and `[TYP-44a]`,
   in place of S-1's twin type index and S-3's whole-program weak-count fact. `[RC-4]`'s counts for
   `thread_shared` values (home and away) go in the same layout change, so that the layout changes
   once.
2. **A8-7 is reversed by the spec.** A named function given to an `owns f: F` parameter is lent,
   not handed over (`[FN-2a]`, `[STD-19]`, ODR-240, verdict 150). The tree already lends; the item's
   first bullet is dropped.
3. **A8-2 (one-element tuples) is dead.** `(e,)` is `E0100` (`[GRM-29]`). `view[A, B]` needs
   variadic generics (`[TYP-47]`, `[TYP-26]`, verdict 115): new work, before M14, as `MILESTONES.md`
   already notes.
4. **New work items** that no gap item covers:
   - `runtime` names (`[LT-48]`): partly built in `borrows.rs`; D-967 and D-968 are its open defects;
   - module variables re-checked at the adding package (`[MOD-9a]`);
   - the interface file's added records (`[BLD-14]` items 8, 9, 11, 14, 15, for A4-3, M2);
   - `[RC-4]`'s home and away counts (M9, with its layout settled in M6b);
   - `E2113`, the first line of an override (with A4-16, M11).
5. **Items that grew.**
   - A6-2 (was S, M12) now carries `[FFI-51]`'s header reading, GCC's assembly read, and
     `[BLD-FFI-3]`'s `unsafe_libs` with its system tables, the libraries asked for, and the unused
     library check: now L.
   - A6-20 (M13) carries `[HR-2]`, `[HR-2a]`, `[HR-3b]` and `[HR-44]`.
6. **M0 is already in the tree.**
   - The gap's A5-1 says "an `unsafe fn` body is not an unsafe context", and its design tests are
     the opposite of the spec (`[UNS-12]`, verdict of 2026-10-29) and of the tree.
   - A5-1's design tests are retired when M12 moves the tests.
   - M1 was built to a later grammar than Hardened_1: `needs`, `safe:`, `threads`, `pub(package)`.

## Built code that must change now, for the batch "N" (checked)

Small fixes in parts M1, M2 and A1-2 built, or in the 0.9.10 features. Each fix comes with a test
that fails before it.

| # | Rule | Today | The spec |
|---|---|---|---|
| N1 | `[GRM-29]` | `(e,)` parses as a one-element tuple (`ember_parser/src/body.rs:1458`); one test uses it (`TYP-39/accept_collections_print_as_python_shows_them.em:29`) | `E0100`, "a tuple has two or more elements; write `e`" |
| N2 | `[MNG-1]` | an instance's suffix is 16 hex digits of BLAKE3 (`ember_branding/src/lib.rs:150`) | 32 hex digits, 128 bits |
| N3 | `[TYP-22]` | only a `Self` result makes an interface not usable as `dyn` | a `Self` parameter too, by value or inside a type |
| N4 | `[PAR-2]` | `?` is accepted in a `@parallel` body | `E7010` with its message |
| N5 | `[ERR-4]` | `unwrap` texts "called `unwrap` on `None`" and on an `Err` value (`ember_typeck/src/lib.rs:29006`, `:29061`); no `Display` bound | `E: needs Display` (`E2040`), the error's `Display` text, `called unwrap on None` |
| N6 | `[UNS-12]` | no `E2020` for an `unsafe fn drop` | `E2020` with its message; the override and interface cases come with M11 |
| N7 | `[ECS-2]`, `[TYP-44]` | `type_identity` in std and the compiler (13 places) | `std.core.type_id[T]()` |
| N8 | `[FN-4]`, `[LT-1a]` | message texts say `self` and the old rename help | `this`, and the new help |
| N9 | `[GRM-2]` | to check: an import after an item | as `[GRM-2]` says |
| N10 | `[RC-1]`, `[WK-16]` | registry rows (`ember_diag/src/codes.rs:252`, `:255`, `:439`) | the 0.9.20 texts |
| N11 | `[DET-4]` | `std/src/math/det.em`'s names and list | `[DET-4]`'s |

Waiting for their milestones (they are not "now"):
- `libs` and `link=` (the C import, M12, with `unsafe import`);
- `[HR-42]` and `[HR-13b]` (M13);
- `[ARN-5]` (M10);
- `[CORO-6]` (M5);
- `[HND-3]`'s panic text (M15);
- `[FFI-57]` and `[FFI-39b]` (M12).

## Not checked by the lead

- The workers' rows marked "unsure" or low confidence: the "Needs the lead's eye" lists of the four
  tables.
- Each is read when its milestone is built.
