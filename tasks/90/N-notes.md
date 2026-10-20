# Job N — the C header importer (`import c`), first slice: notes

Helper agent N, 2026-10-20. Branch `helper-N`. Everything below is built, tested on Windows (MSVC and
clang-cl: the conformance runner, the Rust suites of the crates changed), the emitted C of every new
run-pass test and every shim compiled with gcc 15 in WSL with `-Wall -Wextra -Werror` (never run
there: WSL has no libclang, so the importer itself has not run on Linux), and committed on the branch.

**Effect on the computed table** (`tools/phase_progress.py`, before → after on this branch): Phase 5
51% → 66% (42 → 63 of 116 rules), Phase 4 82% → 88% (`TCB-1`, `TCB-5`), Phase 1 96% → 97%
(`CLI-5`..`CLI-7`), Overall 87.9% → 90.0% (90.02). README.md is not edited here, so `phase_progress.py
--check` reports the table out of date until it is rewritten on merge.

**For the merge:** the importer needs libclang wherever the suites run. CI installs `libclang-dev`
on Ubuntu (test jobs and the documents job); `bench/gcc/gate.sh` in WSL needs it too (`sudo apt
install libclang-dev`), or every `import c` test there fails with `E9002` naming it. The `.embind`
records go to the build cache (`<cache>/embind/`), a few KB each.

## 1. What was built, feature by feature

### N1. The importer (`compiler/ember_driver/src/cimport.rs`), `[FFI-6]`, `[FFI-8]`

**Shape.** `import c "header.h" with (…) [as name]` is turned, while modules are loaded
(`load_package_modules`), into an ordinary module written as Ember source: `@layout(c)` structs,
`@repr(…) @non_exhaustive` enums, `type` aliases, constants, and an `unsafe extern "C"` block of
functions, opaque types and statics — the forms hand declarations use. The import is rewritten to
`import c.h_<stem> as <name or c>`. Everything after the parser (name resolution, the foreign-call
checks, contracts, the C wrapper, codegen) then works on it unchanged, which was the point: the
hand-declaration machinery (ODR-073/074/077-082, ODR-137) is the one contract system. The generated
text is a source-map file named `<import c h.h as c.h_stem>`, so a diagnostic in it shows the
declaration the importer wrote.

*Why text, not AST:* the parser already turns text into exactly the items needed; building AST nodes
by hand would duplicate it, and the text is readable when something goes wrong. Price: one extra
lex/parse of a few hundred lines per import.

**libclang.** `clang-sys` 1.9 with `runtime` (and `clang_9_0`, the oldest version with every function
used): the compiler builds and runs without libclang; a program that imports a header needs it, and
its absence is `E9002` naming what to install (`winget install LLVM.LLVM`, `apt install
libclang-dev`, or `LIBCLANG_PATH`). The importer module is the only `#![allow(unsafe_code)]` in the
driver. The header is parsed as `#include "header"` in an unsaved file placed in the importing
file's folder (so a quoted include finds it as C would), with `-x c -std=c11` (the standard the
generated C is compiled with), the package's `[c] flags`, and the import's `-I`/`-D` (`[BLD-FFI-1a]`).
The target is the host's, as for the backend (Ember has no cross-compilation yet).

**What is imported.** Declarations in the header itself and in the non-system headers it includes;
a declaration of a system header only when one of those uses it (pulled in on demand: `FILE*` brings
`struct _iobuf`). Mapping (`[FFI-8]`):

| C | Ember |
|---|---|
| `char` | `i8`/`u8` as the target's `char` is signed; `short`…`long long`, by size and sign; `long` is `i32` on Windows and `i64` elsewhere (what `c_long` stands for) |
| `size_t`, `uintptr_t` / `ptrdiff_t`, `intptr_t`, `ssize_t` | `usize` / `isize`, by name |
| `wchar_t`, `char16_t`, `char32_t` | `u16` on Windows, `i32` elsewhere; `u16`; `u32` |
| `float`, `double`, `_Bool` | `f32`, `f64`, `bool` |
| `T*`, `const T*`, `void*` | `*mut T`, `*T`, `*mut void` |
| function pointer | `Option[extern "C" fn(…) -> R]` (nullability is unknown, so the safe choice; `Option` of a function pointer is one pointer) |
| `T[N]` field or global | `[T; N]`; a parameter's array decays to a pointer |
| complete struct | `@layout(c) @derive(Copy)` struct, `@packed` (`#pragma pack(1)`, `__attribute__((packed))`), `@align(N)` (`_Alignas`) |
| incomplete struct | `type S` in the extern block |
| enum | `@repr(<underlying>) @non_exhaustive enum`; a second name for a value is `const ALIAS: E = E.FIRST`; an anonymous enum's constants are constants of its integer type |
| typedef | `type T = …`; `typedef struct {…} T` names the struct `T` |
| `extern` global | `static mut` (`static` when `const`); reading needs `unsafe` (`@ffi(immutable)` is never derived) |
| `static const` with a constant value | `const` |
| identifier that is an Ember keyword | `r#name` |

Uncontracted (`[FFI-38]`): every function is an `unsafe` declaration with `@safety(…)` (the
obligation it leaves the caller), so each call needs `unsafe` (`E5002`). The `E5002` of an imported
function prints the overlay entry that states its facts (`fn f(p: TODO(ownership) TODO(count)
TODO(nullable), n)` in an `unsafe overlay c "h.h":`), not the hand form (`[DIA-18]`,
`overlay_unknown_line` in `ember_typeck`).

**Not representable, `W5002`** (`[FFI-20a]`), at the import with the declaration's `file:line`, the
construct, and a workaround: variadic functions, `long double`, `__int128` (argument passing differs
between C compilers), `_Float16` (Ember's `f16` crosses as a 16-bit integer), K&R functions without a
prototype, other calling conventions, globals with internal linkage (`[FFI-29a]`), names taken twice
(C keeps struct tags apart), anything using one of these. A union, a struct with a bit-field, an
anonymous member or a flexible array member is imported as an opaque type (usable behind a pointer)
and reported as such. A reference to a skipped name is `E1010` saying it was not imported and why
(`report_skipped_foreign`: a call through the module, a value, a type).

### N2. Constant macros (`[FFI-6a]`)

Object-like macros of the selected headers are evaluated by Clang in a second parse: one line
`static const __auto_type ember_macro_N = NAME;` each, then `clang_Cursor_Evaluate`. An integer,
floating or string constant becomes a `const` of its C type (`int` → `i32`, `0xFFu` → `u32`,
`1ULL << 40` → `u64`, `'A'` → `i32`, `0.5f` → `f32`, `sizeof` → `usize`, `"text"` → `cstr` with
`c"…"`). Any other is `W5001` with Clang's own diagnostic for that line as the reason ("expected
expression", "initializer element is not a compile-time constant"); a function-like macro is `W5001`
too, suggesting a `static inline` C function that calls it. Unparenthesised on purpose: Clang
evaluates a string only as the literal itself. A macro that expands to nothing (an include guard)
is not reported (owner question 2).

### N3. The shim (`[FFI-29]`, `[FFI-29a]`, `[FFI-5a]`, `[FFI-29b]`)

Per import, `<out>/<profile>/c/c_h_<stem>.shim.c`, compiled into an object of its own with the
import's flags and linked into the program (or the archive/shared library): it includes the header
by its absolute path, asserts each imported struct's size, alignment and every field's offset
(`_Static_assert`, with Clang's numbers), and defines an external wrapper
`ember_cshim_<module>_<name>` for each function with internal linkage (`static inline`); the binding
names the wrapper through `@ffi(link_name=…)`, never the internal symbol. `implementation = [M]`
defines `M` before the header in the shim, so a single-header library's implementation is compiled
there, once; a second import of the header asking for it is `E5014`.

### N4. Layouts verified (`[FFI-5]`)

`ember_typeck::check_foreign_layouts`: each imported struct's Ember layout (`TypeTable::layout`)
against Clang's; `E5001` names the type, each differing field and both numbers. A struct packed to 2
(`#pragma pack(2)`) has no Ember spelling and is the test.

### N5. Build integration (`[BLD-FFI-1]`, `[BLD-FFI-1a]`, `[BLD-FFI-3]`)

`import c "./bridge.c"` compiles `bridge.c` (with the import's flags) and imports `bridge.h`. The
`[c]` section's `compiler` (default: the backend's) compiles shims and sources, its `flags` reach
them and the libclang parse. `link = […]` on an import links each library after the manifest's
`[link] libs`, in the order written; a path is the package's. `ember_build::compile_object_with_flags`
is the new entry point (`compile_object` calls it with none).

### N6. Overlays (`[FFI-2]`, `[FFI-2a]`, `[FFI-12]`, `[FFI-13]`, `[FFI-30a]`)

`with (overlay = "overlays/x.em")`, the path the package's (else the importing file's folder). The
overlay file is loaded as a module of its own (`c.h_<stem>_overlay`) so `check_overlay` checks its
vocabulary, `E5067` and `@grade` as before; its single `overlay c "…":` is applied before the module
is written:

* a `fn` entry makes the function `safe fn` with `@ffi(param(p, words…), result(words…), <the
  entry's own @ffi args>)` and the safe types those words give a hand declaration (`safe_parameter`,
  `safe_result`): `borrowed one` on `const T*` is `ref T`, on `T*` a `mut` parameter; `count(n)` a
  `Span`/`MutSpan`; `fixed(N)` `ref [T; N]`; `nul_terminated` `cstr`; `nullable` an `Option`;
  `owned` a `ForeignBox`; `returns_owned` a `ForeignBox` result; `out`/`inout_count` a `mut`
  parameter. `[FFI-2a]`: mutability comes from `const`, never written; a mutable C pointer's
  contract gains `exclusive` (Ember's mutable borrow is exclusive) unless `aliased` is written. In
  result position nothing is derived: `exclusive` makes a result mutable. A `TODO(…)` or `unsafe`
  word keeps the declaration unsafe (`unsafe` is dropped from the clause: it states no fact). The
  hand-declaration checker then validates the result, so an entry that leaves a pointer without
  facts is `E5002` at the generated declaration. A function with no entry stays unsafe.
* `type T` / `enum E` entries attach their attributes (`@ffi(destroys(f))`, `@ffi(status=…)`) to the
  opaque type or enum; `rename A as B` renames a function (the C symbol kept by `link_name`) and
  gives a type a second name (an alias: its identity stays, `[FFI-30a]`); `hide X` makes `X` private
  to the module (it stays, so `destructor=widget_free` can still name it); `extend` blocks are copied
  into the module (`[FFI-13]`).
* `[FFI-12]`: an entry whose name, parameter count or parameter names do not match, a `type`/`enum`
  entry for something the header does not declare as one, a `hide`/`rename` of nothing, a second
  entry for one function, or an overlay for another header is `E5010`.

### N7. One entity per C declaration (`[FFI-30]`)

A struct, enum or incomplete struct that an earlier import of the build wrote (by Clang's USR) with
the same layout (size, alignment, offsets; an enum's representation and values) is that import's
type: the later module writes `pub type S = ember_c_h_first.S` and imports the first module. So
`left.h` and `right.h` sharing `common.h` exchange values, and so do two imports of one header with
different overlays. An incomplete struct takes any layout. The same declaration with another layout
is a distinct type when the import binds its own name, and `E5011` otherwise.

This needed `[RNG-1]`'s transparent aliases to be transparent everywhere: a struct is now constructed
through an alias that names it, and an enum's variants are reached through one
(`struct_through_alias`, `enum_through_alias` in `ember_typeck`). Before, `Alias(x=1)` was "cannot
find" (defect D-N3 below).

### N7a. Re-exports (`[FFI-30b]`)

`pub import c "h.h" as vk` parses (the grammar allows `pub` on a foreign import only) and re-exports
the module: an importer of `m` reaches it as `m.vk`, or imports it with `from m import vk`
(`Checker::public_namespaces`). A module's other imports are its own: before, `a.m.sqrt(…)` reached
`std.math` through a module `a` that had `import math as m` (D-N7). A path still reaches a module's
child module (`paths.inner`, `[GRM-24]`). `m.vk` used where a value is wanted now says `m` is a
module and what follows is none of its items, instead of "cannot find `m`". The header's module is a
dependency of the re-exporting module in the interface artifacts (`[BLD-2]`), whose layouts hold the
imported structs, so a header change changes the hash dependants record.

### N8. `ember bind` (`[CLI-5]`, `[CLI-6]`, `[CLI-7]`)

`ember bind <header> [-I dir] [-D def] --init [--merge]` writes `<stem>.overlay.em` beside the header:
every pointer fact as `TODO(…)`, `-> unsafe` on a function of scalars (safe-to-call is the one fact no
header states), each derived fact and each skipped declaration as a comment. It compiles unchanged,
is never written over, and `--merge` appends only the functions and skips it lacks. `--report
[--overlay f]` lists skipped declarations, opaque structs and every function still unsafe with its
unknown facts, non-zero on a skip; `--baseline f` counts only skips the file does not list,
`--write-baseline f` writes them. `--check <overlay>` runs `[FFI-12]` and prints "N safe, M still
unsafe, K skipped". `--explain <fn> [--overlay f]` prints the declaration the import writes and, per
parameter and result, its C type, the fact derived from it (a read-only or mutable borrow), what the
overlay states or what is still unknown, and whether it is safe to call and why (`[FFI-14]`).
`--emit-embind` writes `<stem>.embind` beside the header.
`[FFI-35a]`: `--report` also lists, per function an overlay contracts, each pointer parameter with no
`retained` word ("does not retain the pointer") and each pointer result with no lifetime word.

### N8b. The `.embind` record (`[FFI-7]`, `[FFI-14]`)

An import's result is kept in the global build cache (`<cache>/embind/<key>.embind`): versioned CBOR
(a small codec in `cimport.rs`: unsigned integers, text, arrays, maps) holding every file the header
included with its hash, the Clang configuration, the overlay's hash and text (its contracts), the
header's path, the declarations (items, functions with their C and Ember types), the layouts, what
was skipped, the shim, and the `[FFI-30]` entities. The key is the record's version, this compiler
(`compiler_identity`), libclang's path, the arguments (standard, flags, defines, include paths), the
header as written and its folder, the module's name and binding, the overlay's hash, and the
declarations other imports of the build wrote. A record is used when every file it lists still has
its hash; otherwise the header is parsed and the record written (aside, then renamed). So a header is
parsed again only when its content, an included header, its overlay, or a setting changes. `--timings`
says which happened ("import c, parsed" / "import c, from its .embind"): about 140 ms against 100 ms
here, most of the second being libclang's load, which the key needs. The overlay is still applied on
each build (it is cheap and its diagnostics must be repeated).

### N8c. Evidence rests on the header and overlay (`[TCB-5]`)

`ember test --instrument-ffi`'s evidence record gives each function `import c` wrote an
`imported_from` list: the header and the overlay, each with its path and hash
(`ForeignImport::identities`, `InstrumentedRun::imported`). `ember tcb` reads them back: a changed
header or overlay makes the record stale ("its header …"), and the facts go back to asserted
(`[TCB-6]`).

### N8d. Grades backed by evidence (`[TCB-1]`)

`@grade(instrumented)` on an overlay's function entry is met when the package's evidence record
(`ember test --instrument-ffi`, `[ffi] evidence`) still holds for that function — same test binary
and toolchain, the same declaration (the generated module's text hash), an unchanged library, header
and overlay — and graded each of its effect facts `instrumented` (`foreign::backs_instrumented`,
computed while the import loads, passed to the type checker as `ForeignImport::backed` of the
overlay's module). Any other grade above `asserted` is `W5050`; `ember build --require-evidence` and
`ember tcb --require` make it `E5050` (the warnings are raised after checking). `checked` and
`proven` have no evidence mechanism, so they are always unmet. `E5050` keeps its other, older use
(a type with no foreign representation, helper B's noted defect); moving that to `E5015` churns
B's tests and page and is left to the coordinator.

### N8e. Link names C cannot spell (`[FFI-49]`)

`@ffi(link_name="weird$twice")` (any symbol a linker takes; a space, quote, backslash or control
character is `E0104`) binds the declaration through `[MNG-2]`'s alias mechanism, as `@export` symbols
C cannot spell already were: the declaration is named `em0x<hex>` in the C and bound to the symbol by
an assembler label (gcc, clang, clang-cl) or by MSVC's linker (`/ALTERNATENAME:alias=symbol`; the
reverse of an export's). Functions, counted wrappers' C callees and statics alike. With `[link]` and
`link = […]` built, `[FFI-49]` is whole; its unfinished entry is gone. (`@ffi(ownership="transfer")`,
which is no contract word, is still `E0900` "only `@ffi` link names and borrowed-one pointer
contracts are implemented yet" in `FFI-49/reject_unimplemented_ffi_contract.em`; that message is
stale since helper B's contracts, and an unknown word would be `E0104`: left for the coordinator.)

### N8a. Wrapped calls in the cost report (`[FFI-29c]`)

`ember inspect --cost`'s "FFI call" row says `wrapped (static inline)` for a call that goes through
the shim's wrapper (its symbol is the wrapper's), and "one direct call" otherwise. A variadic
`static inline` function is skipped with `W5002` (it is variadic).

### N9. Small fixes on the way

* The overlay contract parser took `unsafe` but not the keyword `owned` as a word (`fn f(p: owned
  one)` did not parse). Fixed in `parse_contract_words`.
* `ember_build::interface`: a foreign `safe fn` with a `from(static)` result of a view other than
  `ref` (`cstr`, `Span`) in a non-root module was an internal compiler error ("`@borrows` must name
  at least one parameter"). The artifact now accepts an empty borrow list for any ABI callable (a
  source `@borrows()` is `E2031` before it gets there). D-N2.
* CI (`.github/workflows/ci.yml`): the Ubuntu test jobs and the documents job (error pages) install
  `libclang-dev`; the Windows image has LLVM.
* Error pages for the codes the importer emits: `docs/errors/E5001.md`, `E5010.md`, `E5011.md`,
  `E5014.md`, `E9002.md`, `W5001.md`, `W5002.md`. A page may now give files its examples are compiled
  beside (```` ```c,file=shape.h ````), in `tools/error_pages.py` and `ember test --doc` alike.

## 2. Tests

| Rule | Test |
|---|---|
| `FFI-6` | `tests/conformance/FFI-6/accept_a_header_and_its_c_source.em`, `accept_static_inline_functions_through_the_shim.em`, `reject_an_unknown_import_option.em` |
| `FFI-6a` | `tests/conformance/FFI-6a/accept_constant_macros_are_constants.em`, `accept_other_macros_are_skipped_with_clangs_reason.em` (left unfinished, §3) |
| `FFI-8` | `FFI-6/accept_a_header_and_its_c_source.em`, `FFI-6a/accept_constant_macros_are_constants.em`, `FFI-5/accept_packed_and_aligned_layouts_match_clangs.em` (left unfinished, §3) |
| `FFI-20a` | `tests/conformance/FFI-20a/accept_each_skipped_declaration_is_reported.em`, `reject_a_skipped_declaration_says_why.em` |
| `FFI-38` | `tests/conformance/FFI-38/*`, `FFI-20a/accept_each_skipped_declaration_is_reported.em` |
| `FFI-5` | `tests/conformance/FFI-5/accept_packed_and_aligned_layouts_match_clangs.em`, `reject_a_layout_ember_cannot_write.em` |
| `FFI-5a` | `ember_driver/tests/cdylib.rs` `import_c_shims_reverify_each_layout_with_the_projects_compiler` (a field only non-Clang compilers see stops MSVC's and gcc's build at the shim's assertion) |
| `FFI-29`, `FFI-29a` | `FFI-6/accept_static_inline_functions_through_the_shim.em` (`assert-c`: the binding names the wrapper) and the milestone above (`FFI-29` left unfinished, §3) |
| `FFI-29b` | `tests/conformance/FFI-29b/*` |
| `FFI-2` | `tests/conformance/FFI-2/accept_an_overlay_makes_calls_safe.em`, `reject_facts_an_overlay_does_not_state.em`, `FFI-38/reject_an_uncontracted_call_needs_unsafe.em`, `FFI-12/accept_an_overlay_matching_its_header.em` |
| `FFI-2a` | `tests/conformance/FFI-2a/*` |
| `FFI-12` | `tests/conformance/FFI-12/*` |
| `FFI-13` | `FFI-2/accept_an_overlay_makes_calls_safe.em` (the overlay's `extend Size`) |
| `FFI-30` | `tests/conformance/FFI-30/*` |
| `FFI-30b` | `tests/conformance/FFI-30b/*`; milestone `a_reexported_header_layout_is_part_of_the_interface_hash` |
| `FFI-30a` | `FFI-2/accept_an_overlay_makes_calls_safe.em`, `FFI-30/accept_two_imports_of_one_declaration_are_one_entity.em` |
| `BLD-FFI-1`, `BLD-FFI-1a`, `BLD-FFI-3` | `ember_driver/tests/cdylib.rs` `import_c_compiles_its_source_with_the_c_section_and_links_in_order`; `FFI-6/accept_a_header_and_its_c_source.em`; `FFI-6/accept_static_inline_functions_through_the_shim.em` (defines) |
| `CLI-5` | milestone `bind_init_writes_a_starter_overlay_that_compiles_unchanged` |
| `CLI-6`, `CLI-7`, `FFI-35a` | milestone `bind_report_and_check_count_what_is_safe_unsafe_and_skipped` |
| `FFI-29c` | milestone `inspect_reports_a_wrapped_static_inline_call` |
| `FFI-49` | `tests/conformance/FFI-49/accept_a_link_name_c_cannot_spell.em`, `reject_a_link_name_no_linker_takes.em` |
| `TCB-1` | milestone `a_grade_on_an_overlay_entry_is_met_only_by_evidence_that_holds` |
| `TCB-5` | milestone `evidence_for_an_imported_declaration_rests_on_its_header_and_overlay` |
| `FFI-7`, `FFI-14` | milestone `import_c_parses_a_header_again_only_when_it_changes`; the codec's unit test `cbor::tests::values_round_trip` |

Test overlays live in an `overlays/` folder beside the tests: the runners take every `*.em` directly
in a rule folder for a test.

## 3. Not built, and why (recorded in `docs/phase-status.json`)

* `[FFI-6a]` — a null-pointer or pointer-cast macro (`#define NIL ((void*)0)`) is skipped with
  `W5001` ("Ember's compile-time values cannot be pointers"): `[CT-5]`/`E6010` forbid a pointer
  `const`. Owner question 1.
* `[FFI-6b]` — function-like macros through an overlay: the overlay grammar gives an entry no types,
  so a macro's signature cannot be written. Owner question 3. With it, `[FFI-29]`'s "wrappers of
  `[FFI-6b]`" (unfinished) are not built.
* `[FFI-8]` — rows not built for imports: unions (no Ember `union` yet), bit-fields (`get_`/`set_`),
  anonymous members (`<Parent>_anon<N>`), flexible array members (unsized structs), `_Atomic`,
  `va_list` and variadic calls, `long double` (`c_longdouble`), `volatile`, `@ffi(newtype)`; and
  `std.ffi` has none of `c_int`, `c_char`, `c_long` and the other C aliases Part XV lists (D-N4).
* `[FFI-30c]` — overlays from several packages, their order and `override`: one overlay file per
  import, holding one overlay. Owner question 5.
* Linux: the importer has not run there (no libclang in WSL here); CI is the first run.

## 4. Owner questions (real choices only)

1. **A pointer constant macro (`[FFI-6a]` against `[CT-5]`).** `[FFI-6a]`: "An object-like macro is
   imported when, after expansion, it is a constant expression of integer, floating, string,
   null-pointer or pointer-cast type as Clang evaluates it; its type is the C type of that value."
   `[CT-5]` (with `E6010`): a compile-time value cannot be a raw pointer ("a `*mut void` refers to
   the evaluator's memory, which the program does not have"). So `#define INVALID_HANDLE ((HANDLE)-1)`
   cannot become `const INVALID_HANDLE: *mut void = …`. Options: (a) allow a pointer `const` whose
   value is an address integer (no evaluator memory: `0 as *mut T`, `-1 as *mut T`), an exception
   to `[CT-5]`; (b) import it as an integer constant of `uintptr_t`'s type (`usize`), the caller
   casting; (c) import it as a `static` with that value; (d) leave pointer macros out of
   `[FFI-6a]`. Built meanwhile: skipped with `W5001` naming `[CT-5]`.
2. **A macro that expands to nothing.** "Any other macro is skipped with W5001": read literally, every
   include guard (`#define MYLIB_H`) and feature flag warns at every import, and `-D warnings` then
   fails every header with an include guard. Options: (a) as written, a warning each; (b) an empty
   macro is not a declaration and is not reported (built); (c) report empty macros only under
   `ember bind --report`.
3. **The signature of a function-like macro (`[FFI-6b]`).** "A function-like macro is exposed only
   when an overlay declares its signature (`@ffi(macro_fn)`)". An overlay `fn` entry's parameters
   take contract words, not types (`[GRM-35]`: `overlay_param := IDENT [":" contract {contract}]`),
   and a macro has no C types for the importer to read. Example: `#define SQUARE(x) ((x) * (x))`.
   Options: (a) the entry names Ember types (`@ffi(macro_fn) fn SQUARE(x: i32) -> i32`), a grammar
   change for this entry only; (b) a `@ffi(macro_fn(param types…, result))` attribute spelling the C
   types; (c) the importer wraps the macro once per use with the argument types of the call. Built
   meanwhile: nothing; `W5001` suggests a `static inline` C function that calls the macro.
5. **Several overlays for one module (`[FFI-30c]`).** "A package may distribute an overlay for a
   foreign module, and several overlays for one module compose in a declared order; they may add
   aliases and wrappers and replace an entry only with an explicit `override`." But `[FFI-6]`'s
   `overlay` option names one file, and `[GRM-35]`'s `overlay_item` has no `override`. Options: (a)
   `overlay = ["dep/overlays/vk.em", "overlays/vk.em"]`, a list in the declared order, and
   `override fn …` in the grammar; (b) each dependency's overlay found by convention
   (`overlays/<header stem>.em`, in dependency order); (c) overlays named in the manifest. Built
   meanwhile: one file, one `overlay c` declaration in it; a second entry for one function is
   `E5010`.
4. **`ember bind --init`'s file name (`[CLI-5]`).** The rule says it "writes a starter overlay" and
   never overwrites a file, but not where. Built: `<stem>.overlay.em` beside the header. Options:
   that; `overlays/<stem>.em` under the package; standard output.

## 5. Defects found (for the coordinator to number)

* **D-N1 (fixed): two imports of one header were parsed from one text.** `SourceMap::add` keeps one
  file per name, and both generated modules were named `<import c "left.h">`, so the second module
  (other defines, another overlay) silently got the first's declarations. Now each is named for its
  module. Probe: `FFI-30/accept_two_imports_of_one_declaration_are_one_entity.em` (the overlaid
  `left.h`'s `left_make` was "requires `unsafe`").
* **D-N2 (fixed): ICE for a `from(static)` view result in a non-root module.** A module with
  `@ffi(result(borrowed, nul_terminated, from(static))) pub safe fn get_word(n: i32) -> cstr`, imported
  by another: "internal compiler error: [LT-40] cannot build interface artifact: … `@borrows` must
  name at least one parameter". The artifact's check allowed only `ref` results.
* **D-N3 (fixed): a type alias was not transparent for construction or variants.** `type P = Point`
  then `P(x=1, y=2)` (or `m.P(…)` through a module) was `E1010 cannot find`, and `type E2 = E` then
  `E2.A` "has no variant"; `[RNG-1]` says an alias without `in` is transparent.
* **D-N4 (open): `std.ffi` lacks the C aliases.** Part XV and the spec's own Part XVI example
  (`from std.ffi import c_int, c_double`) name `c_int`, `c_char`, `c_long`, `c_double`… ;
  `from std.ffi import c_int` is `E1061`. The target-dependent ones (`c_char`, `c_long`, `c_ulong`,
  `c_wchar`) need the compiler to know the target; the importer maps to the concrete types meanwhile.
* **D-N5 (open): a header the runtime header also includes.** The generated C includes `ember_rt.h`,
  which includes `<stdio.h>`, `<string.h>` and `<math.h>`. Importing one of those writes Ember's own
  prototype beside the system one; where the spellings differ (`const char*` is `*i8`, emitted
  `const int8_t*`) gcc refuses the redeclaration ("conflicting types for 'puts'"; MSVC only warns,
  an error under the C gate). Hand declarations have the same problem. Probe: `unsafe extern "C":
  fn puts(s: *i8) -> i32` and a call, then the emitted C (`--emit c --cc gcc`) through gcc. A general
  fix would emit a foreign function's prototype with the C type spellings of its declaration, or
  route a system header's functions through the shim (one call more).
* **D-N6 (open): a foreign declaration and an export of one symbol.** Declaring `add` in an
  `unsafe extern "C"` block of one module and defining `pub extern "C" fn add` in another is an ICE
  ("[BLD-2] import-visible callable `add` has more than one declaration contract") rather than a
  diagnostic.
* **D-N8 (fixed): one C function declared by two modules was an ICE.** Two imports of one header
  with different options (or two hand declarations of one C function in two modules) both declare
  its symbol; the interface artifacts refused the second ("[BLD-2] import-visible callable … has
  more than one declaration contract", then "[LT-40] … present in more than one interface
  artifact"). Declarations that agree are now one callable; ones that disagree keep the error. Test:
  `FFI-30/accept_one_c_function_declared_by_two_imports.em`.
* **D-N7 (fixed): a module's imports leaked through it.** Module `a` with `import math as m`, then
  `import a` and `a.m.sqrt(4.0)` compiled: a path went through any import of the module it named.
  Now only child modules and `pub import c` re-exports are reached; the two tests that went through a
  module did so to its child modules and still pass.
* **Noted, not a defect of this job:** a `const A: i32` binds to a `u32` local without a cast
  (`x: u32 = A`): typed constants coerce like literals. The `[FFI-6a]` test checks types through
  `size_of` and wrapping arithmetic instead.


---

# Part 2 — Job N2: the importer's rulings and open defects (2026-10-20)

Same branch, after `git merge main` (fast-forward, then a second merge for main's 8f7d8ab2: D-747..D-751,
0.9.10_Hardened_24; the one conflict, `docs/phase-status.json`, kept main's `TST-7` entry, since
removed here). Development target: `docs/spec-source/Ember_v0.9.10_Hardened_24.md`. Everything below is
tested on Windows under the C gate with MSVC and clang-cl (the full conformance sweep, both shards,
each compiler), the Rust suites of the crates changed and every milestone; the emitted C of every new
run-pass test and its shim compiled with gcc in WSL (`-Wall -Wextra -Werror`), and the D-743/D-744 tests
also linked there against the runtime (never run).

**Effect on the computed table** (`tools/phase_progress.py`, main 8f7d8ab2 → this branch): Phase 5
63 → 67 of 116 rules (64.5% → 70.3% of words: `FFI-6a`, `FFI-6b`, `FFI-29`, `FFI-30c`), Phase 8
19 → 20 of 42 (`TST-7`, whose baseline is now empty). Overall stays at 90% (rounded).

## 1. The rulings, built

### ODR-145 — a pointer made from an integer is a compile-time value (`[CT-2]`, `[FFI-6a]`)

`const NULL_HANDLE: *mut void = 0 as *mut void` folds: the constant evaluator (`const_eval.rs`) has an
`Address` value — the integer's low bits at the pointer's width, as the cast makes them at run time —
written back into the program as `<n>usize as *T`. A pointer the evaluator made (into its own heap) still
may not leave it (`E6010`). The importer evaluates a macro whose value is a data pointer (`((void*)0)`,
`((Handle*)-1)`) in a third Clang parse, as `(unsigned long long)(MACRO)`, and writes
`const NAME: *mut T = <address>usize as *mut T`; a function pointer stays `W5001`.

*Decision (question Q1):* `[UNS-1]`/`[TYP-7]` make an integer-to-pointer cast need `unsafe`, and a `const`
initialiser has no `unsafe` context to be in. A cast from an integer to a pointer inside a `const`
initialiser therefore needs none: the address is read by nothing until code dereferences it, which needs
`unsafe` anyway. Any other pointer cast there is unchanged.

Tests: `CT-2/accept_a_pointer_made_from_an_integer_is_a_constant.em`,
`CT-2/reject_a_pointer_the_evaluator_made_stays_there.em`, `FFI-6a/accept_constant_macros_are_constants.em`
(null and pointer-cast macros). ODR-146 (an empty macro is not reported) is named by
`FFI-6a/accept_other_macros_are_skipped_with_clangs_reason.em`.

### ODR-147 — `@ffi(macro_fn)` entries, their wrappers and bindings (`[FFI-6b]`, `[FFI-29]`, `[GRM-35]`)

* **Grammar** (`ember_parser/src/decls.rs`, `OverlayItemKind::MacroFn`): in an entry whose attributes
  hold `@ffi(macro_fn)`, `fn NAME(x: type, …) [-> type]` — one Ember type per parameter and result.
* **Matching** (`cimport::apply`): the importer records each function-like macro with its parameter
  names (from Clang's tokens; `...` for a variadic one); an entry matches one by name, parameter count
  and parameter names (`E5010` otherwise, as `[FFI-12]` for functions). A name the header declares as a
  function too (a `getc` macro over a `getc` function) takes an ordinary entry: `E5010` names that.
* **Wrapper** (`[FFI-29]`'s missing part): the shim gets `int32_t ember_cshim_<module>_SQUARE(int32_t a0)
  { return SQUARE(a0); }`, its C types spelled from the entry's Ember types: the scalars (`int32_t`,
  `double`, `_Bool`, `size_t`, `ptrdiff_t`), `std.ffi`'s C aliases (`int`, `long`, `char`…), `cstr`,
  the header's own types by their C spelling (recorded per type at import: `struct Pair`, a typedef's
  name), and raw pointers to those. A type with no C spelling (`String`, an opaque struct by value) is
  `E5015` "type may not cross the boundary" (see Q4). `<stdint.h>` is included for them.
* **Binding**: a function of the module's `unsafe extern "C"` block, bound to the wrapper by
  `link_name`, safe to call — the entry asserts it, as any overlay function entry does (`[FFI-2]`) —
  unless a parameter or the result is a raw pointer: a macro entry has no contract words, so nothing
  makes such a call safe. Other `@ffi(…)` arguments of the entry (`effects=[…]`) are kept; `rename`
  and `hide` reach macros as functions. A declared macro is no longer `W5001`.
* `.embind` version 2 records the macros and the C spellings.
* *Not built:* folding a call whose arguments are all constants ("the importer **may**", optional).

Tests: `FFI-6b/accept_a_declared_macro_is_called_through_its_wrapper.em` (scalars, a struct by value, a
raw pointer under `unsafe`, `bool`), `FFI-6b/reject_a_macro_entry_that_does_not_match_its_header.em`,
`GRM-35/accept_overlay_grammar.em`, `GRM-35/reject_fact_without_unsafe_overlay.em` (a macro entry is a
fact: `E5067` without `unsafe overlay`).

### ODR-148 — `<stem>.overlay.em` beside the header (`[CLI-5]`)

As built in Part 1; the milestone `bind_init_writes_a_starter_overlay_that_compiles_unchanged` now
names ODR-148 and checks `calc.overlay.em` for `calc.h`.

### ODR-149 — several overlays, in order, with `override` (`[FFI-30c]`, `[GRM-35]`)

* `overlay` takes a string or a list of strings (`E0104` otherwise); the files are applied first to last
  into one `Applied`, which remembers which file gave each entry. Entries are keyed per kind: functions
  and macro entries, `type`/`enum` facts, `rename`, `hide`. A second entry for a key is `E5010` unless
  it begins with `override` and comes from a later file, when it replaces the earlier entry (a `type`
  entry's facts are replaced, not added to). `extend` blocks compose. `override` may stand before or
  after the attributes.
* *Decision (question Q3):* an `override` with no earlier entry to replace is `E5010` ("`override`
  replaces an earlier overlay's entry, and none has one for `f`") — a list in the wrong order, or a
  dependency whose overlay dropped the entry, shows up rather than passing silently. A second `override`
  for one name in one file is `E5010` too.
* Each file is a module of its own (`c.h_x_overlay`, `c.h_x_overlay_2`, …) checked as any overlay
  (`E5067`, grades); each has its own `[TCB-1]` backing; every file is an `overlay` identity of the
  import for `[TCB-5]`'s evidence; the import's cache key and `.embind` overlay hash cover all of them.
* "No overlay may change a foreign type's identity or layout": a `rename` of a struct is an alias (the
  value one overlay names `Extent` is the header's `Size`), and a `type` or `enum` entry for a struct is
  `E5010`.

Tests: `FFI-30c/accept_overlays_compose_in_order.em`, `FFI-30c/reject_overlays_that_clash.em`,
`FFI-6/reject_an_unknown_import_option.em` (an `overlay` list of non-strings).

## 2. The defects

### D-742 — `std.ffi`'s C aliases (`[FFI-8]`, `[TST-7]`)

`std/src/ffi.em` declares `c_char`, `c_schar`, `c_uchar`, `c_short`, `c_ushort`, `c_int`, `c_uint`,
`c_long`, `c_ulong`, `c_longlong`, `c_ulonglong`, `c_float`, `c_double` and `c_wchar` (Part XV's "the
other C aliases": every C scalar type `[FFI-8]` names; `c_longdouble` belongs with `long double`, below).
The four that differ between targets are written there as one target's, and the type checker gives
each the type of the target the compiler builds for (`target_c_alias` in `ember_typeck`, by the
compiler's own `cfg` — the C backend compiles with this machine's compiler, and `--target` is not
built): `c_char` is unsigned on ARM, PowerPC, RISC-V and s390x but on Apple's and Windows' ABIs;
`c_long`/`c_ulong` are 32 bits on Windows and 32-bit targets; `c_wchar` is `u16` on Windows. The
importer writes `char`, `int`, `unsigned`, `long`, `unsigned long` and `wchar_t` as those aliases (the
fixed-width forms stay `i8`, `i16`, `i64`…), imports the ones the module uses from `std.ffi`, and keeps
`@repr` of an enum fixed-width. Part XVI's example now checks: `[TST-7]`'s baseline is empty and its
unfinished entry is gone.

Found on the way and fixed (D-N9 below): a type alias of another module named in a signature was
`E1010` when that module's items were collected after the signature's — aliases were collected inside
each module's `collect`. They now have a pass of their own, every module's, dependencies first, before
any signature.

Test: `FFI-8/accept_std_ffi_c_aliases_are_the_targets_c_types.em` — each target-dependent alias agrees
with what this target's C compiler makes of `long`, `char` and `wchar_t` (sizes and signedness, read
through a header), so it is checked on every CI target.

### D-743 and D-744 — one C function, several declarations (`[FFI-6]`, `[FFI-30]`, `[FFI-30a]`)

The two have one cause, as does a third case found while fixing them (D-N11): every foreign function
declaration was emitted as a C prototype under the function's own name, so any other prototype of it in
the one C file — `<stdio.h>`'s `puts` (D-743), another module's declaration of it with contracts of its
own (two overlays of one header, `[FFI-30a]`: `int*` against `const int*`), the program's own
definition (D-744) — met it, and the interface records keyed by symbol refused the second.

*Fix, general:* each foreign function declaration is declared in the C under a private name of its own,
`ember_branding::foreign_alias` = the `[MNG-2]` alias of its symbol (`em_0x<hex>`) followed by the
declaration's qualified name, and bound to the symbol by the mechanism `[FFI-49]` already uses for link
names C cannot spell (an assembler label with gcc and Clang, `/ALTERNATENAME` with MSVC). No two
prototypes of one C function can meet any more, whatever Ember's spellings of its types; calls cost
nothing more (the object file names the real symbol). The coordinator's two suggestions did not reach
every case: a prototype "in its declaration's C spellings" is unknown for a hand declaration (`*c_char`
is `*i8` once checked), and "through the shim" costs every such call a wrapper.

Since the C compiler no longer compares the declarations, the type checker does: every declaration of
one C function, and the program's definition of it (`extern "C" fn`, `@export`), must pass and return
what the others do — each parameter and the result as the C ABI sees it (a pointer, an integer or float
of its size, a C-like enum as its representation, a type by value) — else `E0104` naming both shapes
(question Q2 on the code). A declaration that agrees binds the definition, as a header the package
implements would.

`ponytail:` a `vectorcall` declaration keeps its exact name — MSVC decorates both names (`f@@16`), which
the alias would have to spell — so two `vectorcall` declarations of one function still meet in C. A
counted declaration (`count(n)`, an adapter) is not in the signature comparison: its C prototype is the
adapter's ABI list, not its signature.

Also found and fixed on the way (D-N10): a header that re-declares a C library builtin (`size_t
strlen(const char*)`) had the function skipped, "its result: the C type `__size_t`": Clang types a
builtin with its own `__size_t` sugar, which libclang does not expose. The importer now reads such a
type by its name (`standard_typedef`), else as its canonical type.

Tests: `FFI-6/accept_functions_the_runtime_header_also_declares.em` (an imported `puts` and `strlen`
and a hand-declared `strlen(*c_char)`, beside `<stdio.h>`/`<string.h>`),
`FFI-30/accept_a_declaration_of_a_function_the_program_defines.em`,
`FFI-30/reject_declarations_of_one_function_that_disagree.em`,
`FFI-30/accept_two_declarations_of_one_function_keep_their_contracts.em`. The `#$ assert-c:` needles
of eleven tests that named a foreign prototype by its symbol now name its alias.

### D-746 — `@ffi(ownership=…)`'s stale message (`[FFI-11]`)

The fallback for an `@ffi` argument no form matched said "only `@ffi` link names and borrowed-one pointer
contracts are implemented yet". Now: a word the vocabulary does not have (`ownership="transfer"`, a
misspelt word inside `param(…)`/`result(…)`) is `E0104` listing what `@ffi` takes; `exclusive` with
`aliased` is `E0104`; a form of the vocabulary this compiler has not built (`threads(…)`,
`destroys(f)`, `callback(…)`) is `E0900` naming it. Tests: `FFI-49/reject_unimplemented_ffi_contract.em`
(now claims `FFI-11`), `FFI-11/reject_exclusive_and_aliased_result.em`.

### D-745 — `E5050`'s two meanings (`[TCB-1]`)

The spec's code table gives `E5050` one meaning, `[TCB-1]`'s (a claimed grade whose evidence is absent
or stale). The older use — a type with no foreign representation in an `extern` signature or foreign
static, fourteen sites — moved to `E5015`, "type may not cross the boundary", which already named an
owning value in an exported signature (question Q4 on whether that is the code the owner wants).
`docs/errors/E5015.md` covers both; `docs/errors/E5050.md` is now `[TCB-1]`'s page, whose example is
checked with `--require-evidence` (`tools/error_pages.py` and `ember test --doc` both know that code);
`W5050.md` no longer says the two flags are unbuilt. Ten tests' expected codes changed.

With that, no conformance test asserted `E5050` any more (`rule_index.py`'s `[TST-4]` check): only
the `[TCB-1]` milestone, since the code needs `--require-evidence`. Both test harnesses
(`milestones.rs`, `tasks/impl-0.9.9/annotations.py`) now read `#$ flags:` — one compiler flag per line
for the check of a test's diagnostics — and `tests/conformance/TCB-1/` holds
`reject_a_grade_without_evidence_where_evidence_is_required.em` (`E5050`) and
`accept_a_grade_without_evidence_is_a_warning.em` (`W5050`).

### `[FFI-8]`'s rows, as far as they went

In the table's order: **`long double`** (`c_longdouble`, "opaque bytes") needs a type the C backend
emits as `long double` — its size and alignment are the target's (8 on Windows, 16 on x86-64 and
AArch64 Linux) and System V passes it in the x87 class, so a struct of bytes would be wrong by value:
a new primitive in `ember_types`, not built. **`volatile T*`** maps to `Volatile[*T]`, which `[UNS-5]`
puts in `std.mem` and which does not exist yet: not built. Then:

* **Bit-fields — built.** A struct with integer or `bool` bit-fields is a `@layout(c)` struct laid out
  as this target's C compiler lays it out (the two ABIs differ: MSVC starts a new unit when the type
  changes): its plain fields; each run of bytes its bit-fields share as `pub _bits<k>: [u8; n] = [0, …]`
  (public with a default, so the struct can still be made); `_pad<k>` arrays where C leaves a gap
  Ember's own layout would not. `[FFI-5]` checks every member's offset. Each bit-field has
  `get_<f>(self) -> T` (signed ones sign-extended, `bool` ones as `bool`) and `set_<f>(mut self, value:
  T)`, which writes only its bits; they assemble the bytes little-endian, as the bit numbering is on
  every target Ember builds for (`ponytail:` a big-endian target keeps such structs opaque). An
  enum-typed bit-field, or one spanning more than 8 bytes, still makes the struct opaque (`W5002`).
  Checked also under `--target=x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` (the layout
  check passes). Test: `FFI-8/accept_bit_fields_are_read_and_written_by_accessors.em` (MSVC's and
  System V's layouts differ for its struct); `FFI-20a`'s skipped-struct example is now an enum
  bit-field.
* **Anonymous members — built for structs.** An anonymous struct member is a field `_anon<N>` of a
  generated type `<Parent>_anon<N>`, at the offset C gives it (libclang shows the member as the
  record, not as a field: its offset is its first named field's in the parent less that field's in
  the member), nested ones too. The type checker reads a field of such a member through it — `p.x` is
  `p._anon0.x`, to read and to write — for structs `import c` wrote only (the field and type names
  must match the pattern). Constructing one names the member (`_anon0 = Packet_anon0(…)`). Anonymous
  *unions* stay opaque with unions. Test: `FFI-8/accept_anonymous_members_read_through.em`.
* **Flexible array members** need an unsized struct in the type checker (`E5017` on every by-value
  use) and `tail(self, n)`: not built. **`_Atomic T`** needs libclang 11's `CXType_Atomic` and
  `clang_Type_getValueType` (the importer is built against the libclang 9 API): not built.
  **`va_list`** and variadic calls need `VaList` and variadic call lowering: not built.

`docs/phase-status.json`'s `FFI-8` entry says what is left.

## 3. Rules now done, with their tests

| Rule | Test |
|---|---|
| `[FFI-6a]` (ODR-145, ODR-146) | `FFI-6a/accept_constant_macros_are_constants.em`, `FFI-6a/accept_other_macros_are_skipped_with_clangs_reason.em`; `CT-2/accept_a_pointer_made_from_an_integer_is_a_constant.em`, `CT-2/reject_a_pointer_the_evaluator_made_stays_there.em` |
| `[FFI-6b]` (ODR-147) | `FFI-6b/accept_a_declared_macro_is_called_through_its_wrapper.em`, `FFI-6b/reject_a_macro_entry_that_does_not_match_its_header.em` |
| `[FFI-29]` | the same run-pass test (the wrappers of `[FFI-6b]`), with Part 1's static-inline wrapper tests |
| `[CLI-5]` (ODR-148) | milestone `bind_init_writes_a_starter_overlay_that_compiles_unchanged` |
| `[FFI-30c]` (ODR-149) | `FFI-30c/accept_overlays_compose_in_order.em`, `FFI-30c/reject_overlays_that_clash.em` |
| `[GRM-35]` (ODR-147, ODR-149) | `GRM-35/accept_overlay_grammar.em`, `GRM-35/reject_fact_without_unsafe_overlay.em` |
| `[TST-7]` | milestone `every_example_of_the_specification_checks`, baseline empty |
| `[FFI-8]`'s C aliases (D-742) | `FFI-8/accept_std_ffi_c_aliases_are_the_targets_c_types.em` |

`docs/phase-status.json`: the unfinished entries of `FFI-6a`, `FFI-29`, `FFI-30c` and `TST-7` are gone;
`FFI-8`'s no longer lists the C aliases.

## 4. Owner questions (real choices only)

* **Q1 (ODR-145).** A `const` initialiser's integer-to-pointer cast needs no `unsafe`; at run time the
  same cast does (`[UNS-1]`). (a) As built: a `const`'s address is read by nothing until a dereference,
  which needs `unsafe`. (b) Ask for an `unsafe` form of `const` initialiser.
* **Q2 (D-744).** Two declarations of one C function (or a declaration and the program's definition)
  that pass or return different things are `E0104`, the code of a C symbol defined twice. (a) Keep
  `E0104`. (b) `E5011` ("one C identity imported with two different layouts"), widened to functions.
  (c) A new code.
* **Q3 (ODR-149).** An `override` entry with no earlier entry to replace is `E5010`. (a) As built. (b)
  Allow it: it is then an ordinary entry.
* **Q4 (D-745).** `E5015` ("type may not cross the boundary") now also names a type with no foreign
  representation (`str` in an `extern` signature, an incomplete type by value). (a) Keep it, and cite
  `[FFI-5]`/`[FFI-31b]` in its row. (b) A code of its own. — Either way the row's citation needs a look:
  it cites `[FFI-31]`, which in 0.9.10 is the `cdylib` runtime rule; the boundary rule is `[FFI-31b]`.

## 5. Defects found in this job (for the coordinator to number)

* **D-N9 (fixed): another module's type alias in a signature was `E1010`.** `from std.ffi import c_int`
  then `unsafe extern "C": safe fn abs(x: c_int) -> c_int` in the root: "cannot find type `c_int`"
  (a local `x: c_int` worked). Type aliases were collected inside each module's `collect`, so a module
  collected before the alias's own module could not name it in a signature or field. Probe: the D-742
  test above, with the alias pass moved back into `collect`.
* **D-N10 (fixed): a header re-declaring a C library builtin lost the function.** `size_t
  strlen(const char*);` in a user header: `W5002` "its result: the C type `__size_t`" (Clang's builtin
  types use their own sugar, which libclang does not expose). Probe:
  `FFI-6/accept_functions_the_runtime_header_also_declares.em` before this branch's importer change.
* **D-N11 (fixed): two imports of one header with different overlays was an internal compiler error.**
  One overlay makes `answer_value` safe, the other import has none: "[BLD-2] import-visible callable
  `answer_value` has more than one declaration contract"; and with a pointer parameter the two C
  prototypes (`int*`, `const int*`) conflict under gcc. Fixed with D-743/D-744. Probe:
  `FFI-30/accept_two_declarations_of_one_function_keep_their_contracts.em`.
* **Noted (spec):** `E5015`'s row cites `[FFI-31]`, now the `cdylib` runtime rule (see Q4).
