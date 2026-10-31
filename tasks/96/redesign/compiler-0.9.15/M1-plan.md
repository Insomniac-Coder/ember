# M1 — the 0.9.15 grammar, the spelling-only forms and the migrator: work plan (lead, 2026-10-30)

M1 lands as **one commit** on `main` (MILESTONES.md: "M1 refuses every removed spelling and migrates the
repository in one commit, so nothing is accepted two ways"). Its items: A7-4 to A7-9, A7-15 (area 7);
A1-1 (`owns` for `owned`); A2-1 (`from` for `@borrows`, and its other half: `[LT-22]`'s summaries, rule 1
for every method, `E3065` and `L3014` retired; below); A8-6; the mechanical part of A8-9 and A8-10. Each item's files and rules are in `gap/` (07, 01, 02, 08).

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

## Pieces, in the order built on `m1` (lead, 2026-10-30)

The gap map was written against `0.9.15_Hardened_1`; the grammar of `Hardened_5` adds one spelling
change M1 takes: `needs` before every bound (`[T: needs (Add, Default)]`, `where T: needs Ord`,
`interface Named: needs Display:`, verdicts 109-112), the old `T: A + B` refused with `E0100`. Packs
(`Ts...`, verdict 116) and `safe:` are new features, not spellings, and are not M1's.

Each piece is built with its new spelling accepted beside the old, the fast check green, and a `wip`
commit on `m1`:

1. the reserved words; `needs`; type-parameter lines (`[GRM-40]`) — done, `53fb1859`;
2. parameter words `ref`, `ref mut`, `const ref mut`, `owns`, on declarations, lambdas and callable
   types, mapped onto today's modes; `const` kept in the tree for the printers (nothing gives a
   parameter another ref yet, A2-8, so it is checked as `ref mut` is) — done, `848d4d01`;
3. methods: `this`, `owns this`, `static fn`, the object implicit (below);
4. `copy of`, `exists as`, `owns e`, `from`, `str`/`cstr`, the attribute renames, the rest of A7-7/A7-8;
5. the migrator's rewrites, run over the repository; then the old spellings refused.

### The object of a method (piece 3)

A function in a type body is a method unless written `static fn`, and its object's mode is worked out
from its body (`[EXC-15]`, `[BRW-10]`). 0.9.10's `fn f(k: int)` in a type body is a static function
and 0.9.15's a method, so the two cannot both be accepted; the switch is made at once, and the
0.9.10 code moves by the migrator: a call `Counter.starting_at(5)` of a method whose object is
implicit meets an error, whose migration writes `static` before its `fn`.

Typeck types a method's body before the bodies that decide its mode are typed, and builds the body
differently for each mode (`mut self` is a `ref mut` local, `self` a view or a copy), so the mode is
decided on MIR and the MIR made to agree with it:

- **typeck**: an implicit object is a `Mut` receiver flagged implicit: the body may write `this`;
  calls pass it as `ref mut` (a temporary is spilled, as a borrowed argument's is); the read-only
  checks of the receiver (a plain parameter, a read-only loop name, a view) are made but recorded,
  not reported. Mode equality between an interface's method and its implementation is not asked of
  an implicit one.
- **MIR**: lowering records each `ref mut` it makes as the object of a call of an implicit method.
  After drop elaboration, before the borrow checks, a pass decides each mode: a method **may change
  its object** when its MIR writes, or takes `ref mut` or a `*mut` of, a place of `this` other than as
  the object of a call of a method that only reads it; least fixed point over those calls, a call
  through an interface or a `virtual` method taking the union over the program's implementations
  (`[DSP-5]`). A method that only reads gets a read-only object (`ref Self`), and each call of it a
  read-only borrow; a method that may change keeps both, and each recorded read-only receiver of a
  call of it is `E3023`.
- This is MIR's own mutability, not A4-2's summary: the summary follows a `ref mut` argument into a
  callee that may not change it, but a `ref mut` argument stays a `ref mut` borrow whatever the
  callee does, so the summary's answer could not be made to agree with the MIR.
- `init` (`[CLS-2]`) and `drop` keep a changing object; `owns this` is `Owned`.
- **The object is passed by address**, as C++'s `this` and Rust's `&self`: a read-only object of a
  small `Copy` struct, passed in registers by today's `self`, is a pointer. The C compiler inlines the
  short methods where it matters; the bench says whether it costs.
- **Incremental builds**: a kept body's receiver borrows were decided by its callees' modes, so a
  callee's mode is among the facts a kept body depends on.

**The known numbers and the object worked out later (2026-10-31).** `[STD-19]`'s rule follows a name
set once and never changed, and checks a body again when the body changes one it relied on. A method's
implicit object is `ref mut` until `objects.rs` has every body, so the rule counted every method call on
a name a change, and `kept = 0 .. n; e: int = kept.iter().len()` lost `kept` (`E2020`, `u64`). Now the
rule takes a method whose object is implicit to read it, records the method (`assumed_object_reads`),
and `objects.rs` returns those of them that change their object; `check` then runs again, from the
caller's types (`TypeTable` is `Clone`), counting them changes (`object_changers`). Each round adds a
method, so the rounds end; a program that changes no such name is checked once. Not taken: a second
check of every program (twice the type checking), or counting each call a change (the rule's
rulings lost for every method of `Range`). Two conformance tests in `STD-19` (a method that changes
the range, written after its call, and one that only reads it). The same pass: a `dyn` slot whose
interfaces the program makes no value of only reads (`slot_reads`, as the decision already had it:
`dyn_interface_*` were `E2140`), and the rule reads a method's object through its borrow of either kind
(`object_argument`).

**The run-time checks while M1 stands (2026-10-31).** Until M6b moves them to compiling, a method that
changes its object holds every field for its call, as 0.9.10's `mut self` did: the run-time checks,
and std written with `mut self` (`std.ecs`), are built on it. Tried and reverted: each access on its
own, as 0.9.10's `self` (`[EXC-19]`'s per-field rule): the fast check went from 15 to 48 (std.ecs's
own accesses, `E3020` lost where the call no longer passed `ref mut`). So a 0.9.15 program written in
0.9.10's `self` style, one field viewed while a method changing another runs (`GRM-13` before its
rewrite), panics until M6b; two cases keep the access-by-access form because the call cannot hold the
object: an override of a C++ class's virtual (C++ calls it back while it holds the object's C++ part,
`[FFI-39d]`) and a class's generator method (its body runs after the call returns, `[CORO-12]`). Tests
written for 0.9.10's `mut self` on a method that did not change its object make it change it; ones
whose 0.9.10 `self` method wrote are refused when compiling now (`E3020`, `E3080`) and say so. Open
for M6b: `E3023` for a class generator method that changes its object, through a name that only reads.

The same pass: a held read-only check is keyed by its call as well as its place (a `for` loop's
`iter()` and its iterator's `next()` are passed at one span, and the second released the first's
`E3023`); an interface call finds one held as its interface's method's; and a method's object under
an upcast (`d.first()` on a `Derived`) is read as the call's object and passed as the method takes it
(it was passed as a pointer to the handle to a method taking the handle, and read the wrong memory).

**A loop's name and a method that changes its object (2026-10-31).** `for till in tills: till.tick()`
with `tick` changing its `Checkout` is `E3023` in `[EXC-15]`'s shape: "`tick` changes the `Checkout` it
is called on, and `till` only reads it; write `for till in tills.iter_mut():` in place of line 59". The
spec's line is `for ref mut till in tills:` (`[CTL-1]`), which is A2-6, M5's: M1 names `iter_mut()`,
which works today, and M5 switches the help (`loop_name_changed` in `ember_typeck`). The first-week
corpus's two programs that do it take that help (`R1:B4`). A field written through the loop's name
directly (`c.n += 1`) is still accepted, as 0.9.10 accepted it: that refusal is M5's too.

### The words before and after values (piece 4, 2026-10-31)

Each new form is mapped onto what 0.9.10 means by its forerunner, and checked as that is:

- **`e exists as p`** (`[GRM-44]`) is parsed into the pattern `Some(p)`, marked `exists` so the
  formatter and the tree dump give it back as written; its names take their words (`ref mut c`, a pair
  `(x, ref y)`, `copy of v`). What A7-12 adds later: `E2036` and `E2106` with the rule's texts, and
  the `weakref` half.
- **`copy of e`** (`[GRM-43]`, `[OWN-8]`) is `e`'s `clone` (`synth_copy_of`); before a view it is
  `E2116` ("to make your own copy write `String(w)`"), before a type with no `Clone` `E2116` with the
  `@derive(Clone)` line. **`copy of` a value holding a class object is `E0900`**: 0.9.10's `Clone` of
  a class copies the handle, so the "copy" would be the same object; a new object is M4's ("class
  objects have one owner; `copy of`"). A pattern name `copy of v` is a view of the part with a copy
  made first in the branch or loop body (`take_pattern_copies`), as the rule says `exists as copy of v`
  is `exists as v` then `v2 = copy of v`.
- **`owns e`** is taken where `owned e` was (a `for` iterable, a comprehension's, a `match`
  scrutinee); before any other value it is `E0900`, since an `owns` place is M3's. The formatter
  prints `owns`.
- **`-> T from name`** (`[GRM-41]`, `[LT-1a]`) names a parameter, or a field of `this` written bare;
  every message names `from`. A gone `@borrows` is `E0104` alone, read quietly for what it means.
- **`StrView`**, **`CStrView`** name `str` and `cstr`; **`String(s)`** is `String.from(s)`, one path
  (`string_of_text`). Types are still shown `str` in messages: the display changes with piece 5, when
  `str` is refused, together with the tests that quote it.
- **`@no_alias_checks`** and **`@fp(allow_fma)`** are taken beside `@static_safe` and `@fp(contract)`;
  every message names the new spelling.
- **Capture words** after a lambda's brackets are parsed and refused by name (`E0900`, the CLO area's);
  `with` before them is `E0100` and `ref mut` among them `E0120`, with the rule's texts.
- **`if s is Circle as c:`** (`[GRM-45]`) is parsed and refused by name (`E0900`): kind tests are
  A7-12's checker, M11's.

Not in piece 4: `from` and `guarded by` after a field's type (`[GRM-46]`) and the type words
(`[GRM-3]`), whose meanings are M3's, M5's and the threads area's.

### The migration and the refusals (piece 5, planned 2026-10-31)

`[CLI-20]`'s rewrites of the forms M1 replaces, each made by the repository's own migration in round 0
from the tree (no checker: a test may be written to meet a refusal), and as a migration on the
refusal each old form meets once it is refused:

| 0.9.10 | 0.9.15 | in the repository today |
|---|---|---|
| `owned x: T`, `owned e` | `owns x: T`, `owns e` | 597 (`owned fn` stays: the capture words are the CLO area's) |
| `mut x: T` | `ref mut x: T` (the call words are M3's) | 489 |
| `x: ref mut T` | `ref mut x: T` (`E0117`) | |
| `x: ref T` | `x: T` — **M4's**: as `x: T` a `Copy` type is passed as a copy, a class handle among them until M4, and a view into it ends with the call (`std.ecs`'s `view(fetch: ref AosStorage[T])` returns spans into the object) | 76 |
| `str`, `cstr` | `StrView`, `CStrView` (and the type display) | std 95 |
| `if/elif/while Some(p) = e:` | `if e exists as p:` (`E0115`); `Some(mut v)` is `copy of v` | 62 |
| `@borrows(p)`, `@borrows(self.f)` | `-> T from p`, `-> T from f` (`E0104`) | 148 |
| `@static_safe`, `@fp(contract)` | `@no_alias_checks`, `@fp(allow_fma)` (`E0104`) | |
| `.copied()`, `.cloned()` | `.copy_each()` (`E1010`) | 73 |
| `fn f[T: A](x)` | `[T: needs A]` above `fn f(x)` | 716 |

The refusals, each with the one to write in the program's own names: `self` listed or used in a
method (`E1010`, P14), the mode `owned` and `owned` before a value (`E1010`; `owned fn` stays, the
CLO area's), `mut` alone on a parameter or a pattern name (`E0113`), a mode in a parameter's type
(`E0117`; not in a foreign declaration, where `x: ref mut T` is the pointer C takes, `[FFI-11]`),
`Some(...)` in a header (`E0115`), type parameters after a name and a bound without `needs` (`E0100`),
`str` and `cstr` where no type of the program's has the name (`E1010`), `.copied()` and `.cloned()`
(`E1010`), `@static_safe`, `@fp(contract)`, `@no_runtime_checks` and every `@borrows` (`E0104`).
A parameter of a type with no size of its own (`ref mut p: dyn I`, a plain `p: dyn I`, an opaque
C++ class: "for an opaque type too", `[FFI-11]`) is passed by address as `[BRW-8]` says, so it is
checked as 0.9.10's `p: ref mut dyn I` (`passed_by_address`); `owns p: dyn I` is a value, `E2020`, and
a foreign declaration's parameters are C's. A refused form is read as its fix writes it where that is one reading (`c: ref mut int` as
`ref mut c: int`), so what is checked after is what was meant.

The code the compiler writes is 0.9.15 too: the C and C++ importer (`cimport.rs`: methods with no
listed object, `owns`, `ref mut`, `CStrView`, type-parameter lines, `from`; its internal mode strings
stay, written through `written_mode`), a derived `Component`'s `StrView`, the reload wrapper, and the
view contracts `[DOC-3]`'s report prints. Not M1's (their
replacements are other milestones'): `Box`, `Weak`, `Array[T, A]`, `cast x if D`, `.upgrade()`,
`for x in xs.iter_mut():`, `T: Sync`.

**A2-1's other half is built in M1** (owner, 2026-10-31: build what a goal needs first; `@borrows`
naming several sources had no 0.9.15 form, MILESTONES.md placing that half nowhere). Every `@borrows`
is `E0104`: one name is `from`, `this` and several go. In their place:

- `[LT-1]` rule 1 for every method, `owns this` too. A method's body may return a view of another
  source parameter; its callers then view every source parameter (rule 3, `[LT-22]`:
  `callers_result`). A plain view result (`ref`, `Span`, `StrView`) is the first line's rule at the
  caller, which may view more than the body needs.
- `[LT-22]`: a result of a view type (`[TYP-34]`: `Option[ref T]`, a struct of views) points where the
  body's summary says, within the first line, so std's adapters compose (`Filter` over `Nested`,
  `Skip` over `Chain`: `from inner` holds). A call result with several region slots is the union of
  what its first line ties, which a summary carries on, where 0.9.10 gave up (`E3065`).
- `E3065` and `L3014` are retired (their pages say so; `l3014` in `[lints]` is `E9010`); `[LT-1b]`,
  `[LT-2a]` and shape B14 go with their tests. `from` naming one source of a multi-region result whose
  fields point elsewhere is `E3062` (`[LT-23]`).
- `[EXC-15]`: the object mode worked out from a body reaches the module's interface (a generic owner's
  method's from its opaque check), so an importer sees a method that only reads as one, and a body
  that starts changing its object changes the interface its importers depend on.
- The C and C++ importer writes no `@borrows`: one source is `from`, several go unsaid.

The tests `gap/02` lists for these are rewritten here (LT-1, LT-1a, LT-22, LT-23, HEAP-5's two
`ref mut` handles now chosen by the caller, a non-`Copy` object's plain view and its iterator's
`Option`).

**The rules M1 lists as built** (`docs/phase-status.json` `built_for_0915`, each read whole against
`Hardened_7`): `GRM-40`, `HASH-1`, `LEX-14`, `LT-6`, `LT-22`, `LT-23`, `LT-44`, `MOD-1`, `STD-3`,
`TXT-1`, `TXT-5`; `LANGUAGE_VERSION` is "0.9.15", and `ember fmt --migrate` rewrites
`#! language "0.9.10"`. Read and left out, a part of each not built: `LT-1a` (`from log` on a field
that is not a view, which needs the borrow check to tell one field's loans from another's; `E2118` is
built), `GRM-41` (`E0118` for a first line out of order), `GRM-13` (a plain pattern name changed is
`E3021`, not `E3023`), `LEX-21` (`as?` is `E0100` with `cast x if D`, not `E1010` with the kind test),
`TYP-17` (bounds of a non-`pub` function written all or none), `ATT-1`/`MOD-3`/`MOD-5`/`CLI-20`/`MNG-1`/
`STD-19`/`GRM-15`/`GRM-43`/`GRM-44`/`LEX-15` (the parts other milestones build: `@sync`, `Box`,
`weakref`, `.clone()`, `using`, the field `from`, hand-over).

**`x.clone()` stays until M4** (144 in the repository): `[CLI-20]` rewrites it to `copy of x` and
`[MOD-3]` refuses it (`E1010`), but `copy of` a value holding a class object is `E0900` until M4 makes
a class object's `Clone` a new object, so rewriting it now would break every class clone and refusing
it would leave a class value no copy at all. M4 replaces it whole, as M1 does each form here.

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
