# M1 — the 0.9.15 grammar, the spelling-only forms and the migrator: work plan (lead, 2026-10-30)

M1 lands as **one commit** on `main` (MILESTONES.md: "M1 refuses every removed spelling and migrates the
repository in one commit, so nothing is accepted two ways"). Its items: A7-4 to A7-9, A7-15 (area 7);
A1-1 (`owns` for `owned`); A2-1's spelling half (`from` for `@borrows`); A8-6; the mechanical part of
A8-9 and A8-10. Each item's files and rules are in `gap/` (07, 01, 02, 08).

Before it: A4-1 (the call graph, done) and A4-2's first slice (`ember_analysis::summaries`, done,
06fb0bbc), whose "changes `this`" fact gives a method's change of its object, which 0.9.15 works out from
the body (`[FN-4]`, `[EXC-15]`) instead of reading `mut self`.

## The order inside the one commit

Work happens on a local branch, `m1`, and is squashed onto `main` when the whole is green (full Windows
check, bench). Inside it:

1. **New spellings accepted beside the old.** The lexer's table gains the 0.9.15 reserved words
   (`this`, `owns`, `shares`, `sharedref`, `thread_shared`, `using`, `weakref`; `[LEX-14]`, 52) and the
   contextual ones (`copy of`, `exists`, `guarded by`, `from` after a result or field type; `[LEX-15]`),
   after a scan of the repository for names they take. The syntax tree (A7-5) and the parser (A7-6,
   A7-7) take every new form; the old ones still parse.
2. **The checker maps each new form onto today's meaning** (A7-8): `this` implicit (the receiver's mode
   from the summary: a method that may change `this` is a `mut self` one), `ref` / `ref mut` / `owns`
   onto today's modes, `const ref mut`, `copy of` through `Clone`, `exists as` on an `Option`, `from`
   for `@borrows` (A2-1), `str`→`StrView`, `cstr`→`CStrView`, `String(s)`, the attribute renames;
   `LANGUAGE_VERSION` "0.9.15". At this step both spellings compile to the same MIR.
3. **The migrator** (A7-9) rewrites every 0.9.10 form `[CLI-20]` lists, in every file of a package,
   and runs over `std/`, `tests/**`, `docs/book/`, `docs/errors/` and the Ember programs inside the
   Rust tests (a script over their strings). The suites run green on the migrated tree.
4. **The old spellings are refused** (A7-4's reserved-not-words class, A7-15, A1-1's `owned`): each gone
   form an error naming the form to write (`E1010`, `E0104`), and the habit texts (`[DIA-21]`).
5. One commit: the compiler, the migrated repository, the tests of the new forms, the gap rows
   closed, README by `tools/phase_progress.py`.

## What can be cut into pieces earlier, on `main`

Nothing that changes what a program means: every piece of M1 changes the language's spelling, and
`main` must never accept two spellings. Analysis that only adds facts (A4-2's later slices, A4-3) lands
on `main` separately, as M2 allows.

## Risks

- The keyword scan: a name the new reserved words take (a field `using`, a local `this`) must be
  renamed by the migrator, or the migration cannot run on its own output.
- The receiver's mode from the summary: the summary is worked out on MIR, after type checking, so the
  checker types every method's `this` as a view the borrow check then holds to the summary (a method
  that may change `this`, called through a name that only reads, is `E3023`). This is M1's first use of
  the summary, and the place it is put on `Body`.
- Size: about 3,600 Ember files and 220 programs in Rust tests move; the migrator must be exact enough
  that the suites' expectations (lines, columns in messages) move with the text.
