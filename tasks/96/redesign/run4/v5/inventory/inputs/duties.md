# Duties for the unsafe-features inventory (read `inputs/brief.md` and `JOB.md` first)

`INV` = `tasks/96/redesign/run4/v5/inventory/`. Each sweeper writes `INV/<number>-<name>.md`.

## 01 The new design (`INV/01-design.md`)

Read `V5/rules.md` and `V5/vocabulary.md` in full, and the answers and tests where a rule points to them.
Find every feature of the new design that can break a safety promise outside `unsafe`: every place a
rule says something is checked only while running, assumed, left to the programmer, or not covered;
every escape the rules name (`unsafe`, `@no_alias_checks`, `assert_disjoint`, `@ffi(...)` promises,
`Send` by hand, raw pointers, arenas and memory managers, `static mut` module variables, generators,
`drop`, hot reload, C callbacks); and every rule whose wording leaves a gap. Also check the six kinds of
the brief one by one against the rules: for each kind, which rule stops it, or that none does.

## 02 Today's spec: C calls, `unsafe`, the runtime, build and command-line flags (`INV/02-spec-ffi.md`)

Read the rule families of `SPEC` for C and C++ calls (`[FFI-*]`), `unsafe` (`[UNS-*]`), the runtime
(`[RT-*]`), building and the command line (`[BLD-*]`, `[CLI-*]`, `[VER-*]`), and every flag or attribute
that turns a check off or changes what is checked (search `SPEC` for "off", "disable", "unchecked",
"trust", "assume", "release", "profile"). For each feature the new design keeps (see `V5/vocabulary.md`),
the brief's entry.

## 03 Today's spec: everything else the redesign keeps (`INV/03-spec-rest.md`)

Read every other rule family of `SPEC` that `V5/vocabulary.md` does not replace: types and casts
(`[TYP-*]`), hot reload (`[HR-*]`), generators (`[CORO-*]`), compile-time code (`[CT-*]`), SIMD
(`[SIMD-*]`), memory managers, arenas and the heap (`[MNG-*]`, `[ARN-*]`, `[HEAP-*]`), threads outside the
memory model (`[THR-*]`), exceptions and stops (`[EXC-*]`, `[ERR-*]`), determinism (`[DET-*]`), the
standard library's spec (`[STD-*]`), text (`[TXT-*]`, `[STR-*]`), and the rest. For each feature that can
break a safety promise, the brief's entry.

## 04 The standard library, built-in functions and the runtime (`INV/04-library.md`)

Read `std/src/*.em` in full, the compiler's built-in (intrinsic) functions (search `compiler/` for where
built-in names such as `mem.`, `ptr`, `transmute`, `size_of`, `assume`, `unreachable` are resolved), and
the functions of `runtime/ember_rt/include/ember_rt.h` a program can reach. Find every function or type a
program can call without `unsafe` that can break a safety promise (for example a function marked `unsafe`
inside the library but exposed through a safe wrapper whose promise is not checked, an `unchecked` access,
a function that leaks, a type that is `Send` by hand), and every `unsafe` one, with whether its
`@safety` text says what the caller must make true. The brief's entry for each.

## 05 The attacker (`INV/05-attacks.md`)

Write ordinary programs in the new design's spelling (`V5`), with no `unsafe`, that break a safety promise
anyway, using the rules as written. Try at least: a loop of `shares` owners (a leak); a `static mut`
module variable changed from two threads; a stop (panic) inside `drop`, inside a lock, inside a callback
C runs, and while a list is half-changed; deep recursion; integer overflow in every operator, shifts and
conversions; a value read before it is written (a field, an array made with a size, a struct from C);
generators kept after what they walk ends; hot reload changing a type's layout under live objects;
reductions and `@parallel` loops with views; `weakref` links across threads; function values stored and
replaced while they run; interfaces and casts (`cast x as T`) of the wrong kind; arenas whose instance ends
before a value made in it; `Retained` tokens and C callbacks after unregister; `@cached` results after a
change through a link. For each program, which rule (with its line) refuses it or makes it safe, or that
none does: only the last kind is a finding, in the brief's form.

## 06 The checker (phase 2, `INV/06-checked.md`)

Read `INV/01-design.md` to `INV/05-attacks.md`. For every finding, try to disprove it: find the rule, test
or spec line that already fences it or makes it safe, check that its program really does what it says
under the rules, and that it is not something the new design removes. Mark each **confirmed** (with your
reason), **refuted** (with the line that refutes it) or **unsure** (with what would decide it). Merge
duplicates across files (keep the clearest program). Then list the confirmed findings in order of harm,
each with its recommendation and whether it is "for the owner" (only if no verdict settles it and it
changes how an ordinary program is written or what it does).
