# Job N — the C header importer (`import c`), first slice: notes

Helper agent N, 2026-10-20. Branch `helper-N`. Everything below is built, tested on Windows (MSVC and
clang-cl: the conformance runner, the Rust suites of the crates changed), the emitted C of every new
run-pass test and every shim compiled with gcc 15 in WSL with `-Wall -Wextra -Werror` (never run
there: WSL has no libclang, so the importer itself has not run on Linux), and committed on the branch.

## 1. What was built, feature by feature

### N1. The importer (`compiler/ember_driver/src/cimport.rs`), `[FFI-6]`, `[FFI-8]`

**Shape.** `import c "header.h" with (…) [as name]` is turned, while modules are loaded
(`load_package_modules`), into an ordinary module written as Ember source: `@layout(c)` structs,
`@repr(…) @non_exhaustive` enums, `type` aliases, constants, and an `unsafe extern "C"` block of
functions, opaque types and statics — the forms hand declarations use. The import is rewritten to
`import c.h_<stem> as <name or c>`. Everything after the parser (name resolution, the foreign-call
checks, contracts, the C wrapper, codegen) then works on it unchanged, which was the point: the
hand-declaration machinery (ODR-073/074/077-082, ODR-137) is the one contract system. The generated
text is a source-map file named `<import c "h.h" as c.h_stem>`, so a diagnostic in it shows the
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
too. Unparenthesised on purpose: Clang evaluates a string only as the literal itself. A macro that
expands to nothing (an include guard) is not reported (owner question 2).

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

### N8. `ember bind` (`[CLI-5]`, `[CLI-6]`, `[CLI-7]`)

`ember bind <header> [-I dir] [-D def] --init [--merge]` writes `<stem>.overlay.em` beside the header:
every pointer fact as `TODO(…)`, `-> unsafe` on a function of scalars (safe-to-call is the one fact no
header states), each derived fact and each skipped declaration as a comment. It compiles unchanged,
is never written over, and `--merge` appends only the functions and skips it lacks. `--report
[--overlay f]` lists skipped declarations, opaque structs and every function still unsafe with its
unknown facts, non-zero on a skip; `--baseline f` counts only skips the file does not list,
`--write-baseline f` writes them. `--check <overlay>` runs `[FFI-12]` and prints "N safe, M still
unsafe, K skipped". `--explain` and `--emit-embind` are refused (`[FFI-14]` is not built).

### N9. Small fixes on the way

* The overlay contract parser took `unsafe` but not the keyword `owned` as a word (`fn f(p: owned
  one)` did not parse). Fixed in `parse_contract_words`.
* `ember_build::interface`: a foreign `safe fn` with a `from(static)` result of a view other than
  `ref` (`cstr`, `Span`) in a non-root module was an internal compiler error ("`@borrows` must name
  at least one parameter"). The artifact now accepts an empty borrow list for any ABI callable (a
  source `@borrows()` is `E2031` before it gets there). D-N2.
* CI (`.github/workflows/ci.yml`): Ubuntu jobs install `libclang-dev`; the Windows image has LLVM.

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
| `FFI-30a` | `FFI-2/accept_an_overlay_makes_calls_safe.em`, `FFI-30/accept_two_imports_of_one_declaration_are_one_entity.em` |
| `BLD-FFI-1`, `BLD-FFI-1a`, `BLD-FFI-3` | `ember_driver/tests/cdylib.rs` `import_c_compiles_its_source_with_the_c_section_and_links_in_order`; `FFI-6/accept_a_header_and_its_c_source.em`; `FFI-6/accept_static_inline_functions_through_the_shim.em` (defines) |
| `CLI-5` | milestone `bind_init_writes_a_starter_overlay_that_compiles_unchanged` |
| `CLI-6`, `CLI-7` | milestone `bind_report_and_check_count_what_is_safe_unsafe_and_skipped` |

Test overlays live in an `overlays/` folder beside the tests: the runners take every `*.em` directly
in a rule folder for a test.

## 3. Not built, and why (recorded in `docs/phase-status.json`)

* `[FFI-6a]` — a null-pointer or pointer-cast macro (`#define NIL ((void*)0)`) is skipped with
  `W5001` ("Ember's compile-time values cannot be pointers"): `[CT-5]`/`E6010` forbid a pointer
  `const`. Owner question 1.
* `[FFI-6b]` — function-like macros through an overlay: the overlay grammar gives an entry no types,
  so a macro's signature cannot be written. Owner question 3. With it, `[FFI-29]`'s "wrappers of
  `[FFI-6b]`" (unfinished) and `[FFI-29c]`'s `ember inspect` report are not built.
* `[FFI-8]` — rows not built for imports: unions (no Ember `union` yet), bit-fields (`get_`/`set_`),
  anonymous members (`<Parent>_anon<N>`), flexible array members (unsized structs), `_Atomic`,
  `va_list` and variadic calls, `long double` (`c_longdouble`), `volatile`, `@ffi(newtype)`; and
  `std.ffi` has none of `c_int`, `c_char`, `c_long` and the other C aliases Part XV lists (D-N4).
* `[FFI-7]`, `[FFI-14]` — no `.embind` cache: each build parses each header again (about 0.2–1 s per
  import); `ember bind --explain` and `--emit-embind` are refused.
* `[FFI-30b]` — `pub import c … as vk` (re-export) does not parse: Ember has no re-exports or
  module-in-module namespaces to build it on.
* `[FFI-30c]` — overlays from several packages, their order and `override`: one overlay file per
  import, holding one overlay.
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
   meanwhile: nothing (`W5001` says an overlay can declare it).
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
* **Noted, not a defect of this job:** a `const A: i32` binds to a `u32` local without a cast
  (`x: u32 = A`): typed constants coerce like literals. The `[FFI-6a]` test checks types through
  `size_of` and wrapping arithmetic instead.
