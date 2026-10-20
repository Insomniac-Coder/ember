# Job N4 — the rest of Annex C: notes

Helper agent N, 2026-10-20. Branch `helper-N4`, from `main` at `dde19e54` (the N3 merge, ADR-245;
target 0.9.10_Hardened_26). Main has moved on since (`6eb328d5`: Hardened_28 writes ODR-156 into
`[FFI-39c]`'s text); this branch is not merged with it. Everything below is committed and was
checked on Windows: the quick check (`annotations.py` over every test folder: failing 8, the eight
`LNT-6` cases main fails alike, §6), every FFI conformance folder again under clang-cl and under MSVC
with the C gate on (`EMBER_C_WARNINGS_AS_ERRORS=1`), the conformance suite inside the milestones
under clang-cl, `cargo test --profile ci` of `ember_typeck`, `ember_analysis`, `ember_codegen_c`,
`ember_hir`, `ember_build` and every `ember_driver` suite (milestones, package, reload, cdylib,
staticlib, ui), and the gates `rule_index`, `error_pages`, `hardening_check`, `check_branding`,
`spec_check`, `generate_runtime.py --check`. In WSL every new thunk file compiles with g++
`-std=c++17 -Wall -Wextra -Werror`, the `--cpp` header with g++ `-std=c++20`, and the emitted C of
the new C++ tests with gcc `-Wall -Wextra -Werror`; WSL has no libclang, so the importer itself has
not run on Linux.

**Effect on the computed table** (`tools/phase_progress.py`): Overall 93.96% (45219 of 48125
words); Phase 5 86% (92 of 116 rules). README.md is not edited.

**For the merge.** `.embind` records are version 7 (new keys `trampolines`, `operators`,
`policy_changes`): older records are parsed again. The runtime gains `ember_cpp_object_adopt`
(templates and generated files). New error pages `E5030`, `E5056`–`E5060`, `E9020`, `E9021`. New CLI
option `ember build … --cpp`. `ember_typeck::ForeignImport` gains `trampolines` and
`policy_changes`; `ember_codegen_c::HeaderOutput` gains `cpp_source`. `ember_build` keeps two
process-wide settings a build sets before compiling (`inherit_msvc_runtime`, `inherit_gnu_lto`).

## 1. What was built, feature by feature

### N4.1 The trampoline: an Ember class deriving from a C++ class, `[FFI-39]`, `[FFI-39a]`–`[FFI-39e]`, `[FFI-17c]`, `[FFI-17d]`

An overlay entry `@ffi(trampoline, virtuals=["…"], owner="ember"|"foreign") type X` makes `X` an
open Ember class (`pub open class X`) whose C++ object is a subclass the importer generates
(`ember_tramp_<stem>_<X>`): each named virtual calls the Ember override through an `extern "C"`
entry the Ember side defines, with the Ember object's address. **Decision:** the C++ object is a
separate heap allocation owned by the Ember object through a hidden base field (its pointer and a
drop that runs the C++ destructor's thunk), not embedded in the Ember object: the C++ layout is the
C++ compiler's, and Ember objects move between generations under reload (`[HR-20]`). The base is
"sized" in Ember's sense: a subclass adds fields as to any class.

- `[FFI-39]`: a named method that is not virtual is `E5057`; overriding a C++ virtual the entry does
  not name is `E5056` (`E5056` also for an unnamed *pure* virtual, which no Ember class could leave
  unimplemented — the closest code); a `final` C++ class `E5034`. The constructor is an ordinary
  `fn init(self, …)` calling `super.init(…)`; an override calls the C++ base implementation with
  `super.name(…)` through a thunk that calls `X::name` non-virtually.
- `[FFI-39a]`: the entries are the overrides' permanent thunks under hot reload (`[HR-6]`), so a C++
  object made before a reload reaches the edited override after it. Reloadable images now link the
  imports' shims and thunks (D-N4-2).
- `[FFI-39b]`: `super.init(…)` picks the C++ constructor by arity and argument types; a class with
  no `init` whose base has no default constructor is `E5058`. **Not built:** running it "before any
  Ember field is initialised" (question 1).
- `[FFI-39c]` (ODR-156): a `retained` parameter of a trampoline class takes the `Retained[X]` token
  the program pins itself (`Retained.pin(self)`, which upcasts: `Retained[Derived]` coerces to
  `Retained[Base]`); passing a bare object there is `E5060`, whose note cites ODR-156. A parameter
  that only borrows for the call takes the object itself. The token's thread rule is not enforced.
- `[FFI-39d]`: re-entrant calls reach the same object; an override that begins an access conflicting
  with one its caller holds across the C++ call panics (`[EXC-1]`); `L3013` now also warns where a
  long-term access to an object of a class deriving from a C++ class is held across a call into C++.
- `[FFI-39e]`: a C++ exception that would cross into Ember aborts at the boundary naming where
  (`crossed()` in the thunk prelude); an Ember panic in an override never unwinds into C++.
- `[FFI-17c]`: destruction is derived-first: the Ember `drop`, the Ember fields, then the C++
  destructor (the hidden base field is dropped last).
- `[FFI-17d]`: `owner="ember"` (default): the Ember count owns the object, and a foreign `delete`
  panics; `owner="foreign"`: a parameter contracted `owned` gives C++ a count of its own, which a
  foreign `delete` ends (`ember_cpp_object_adopt` panics if Ember code still holds the object). A
  base without a virtual destructor is `E5059`.

Tests: `tests/conformance/FFI-39/` (accept + 4 rejects), `FFI-39c/`, `FFI-39d/` (with `[lints]
l3013 = "warn"`), `FFI-39e/` (accept + 2 run-fails), `FFI-17c/`, `FFI-17d/` (accept + 2 run-fails),
`compiler/ember_driver/tests/reload.rs` `a_trampoline_reaches_an_edited_override_after_a_reload`.

### N4.2 Bridge types, `[FFI-17a]`, `[FFI-17e]`, `[FFI-17f]`

`std.ffi` declares `CppVector[T]`, `CppString` and `CppShared[T]`, each holding its C++ object and
the thunk pointers of its operations, which the import writes per element type (`data`, `size`,
`push`, `copy`, `destroy`). `std::vector<T>` is `CppVector[T]` (`len`, `span`, `span_mut`,
`to_array`, `push_back`); `std::string` is `CppString` (`bytes`, `to_str`, `String.from(c)` copying
lossily); `std::shared_ptr<T>` is `CppShared[T]` (`get`, `clone` which C++ counts); a
`std::unique_ptr<T>` result is a `ForeignBox[T]`. A bridge value crosses as a result or by `const&`,
`&`, `&&` or value. `[FFI-17e]`: not `Copy`/`Send`/`Sync`; views and pointees borrow the bridge value
(compiler-known `cpp_span`, `cpp_span_mut`, `cpp_pointee`: the result borrows argument 0).
`[FFI-17f]`: `ember inspect --cost` lists each bridge operation and each bridge drop as a thunk call.
A container with another allocator (or a string with other traits) is `E5034` naming `[FFI-48]`'s
row; a `std::function` parameter is `E5030`.

Tests: `FFI-17a/`, `FFI-17e/` (accept + 2 rejects), `FFI-48/reject_an_allocator_parameterised_container.em`,
milestones `inspect_reports_each_bridge_operation_as_a_thunk_call`.

### N4.3 Operators and iteration, `[FFI-42]`, `[FFI-42a]`

A C++ operator — a member, a namespace-level function, or a friend defined in its class (found by
argument-dependent lookup) — imports as the method of its Ember operator interface on the class of
its first operand: `+ - * / % & | ^ << >>` as `add`…`shr`, their compound assignments as
`add_assign`… (the `T&` result dropped), unary `-`/`~` as `neg`/`not`, `==` as `eq`, a `const`
`operator[]` as `index` and another as `index_mut`. **Decision (question 2):** the class implements
the interface (`Add[Rhs]`, `Neg`, `AddAssign[Rhs]`, `Eq`, `Index[Idx]` with `IndexMut[Idx]`) only when
an overlay entry asserts the method safe to call (`[FFI-2]`: an interface's method is never
`unsafe`) and its signature is the interface method's; otherwise it is an `unsafe` method of its
class. `!=`, the orderings, `*`/`->`, `()` and `=` are skipped, saying why.

A class with public `begin()` and `end()` gets `iter(mut self)`, returning a `<Class>_cpp_iter`
whose `next` gives a `ref` to each element, through thunks that walk the C++ iterators (a throw
aborts at the boundary). The element type is `*c.begin()`'s, asked of Clang in the reparse that also
makes `instantiate` lists (the first parse skips function bodies, so a standard library's iterator
class is not yet complete there). `[FFI-42a]`: `iter` takes `mut self`, so a `for` loop borrows the
container mutably for its length and any method that could invalidate the iterators is `E3020`/`E3021`
inside it.

Tests: `FFI-42/` (2 accepts, 2 rejects), `FFI-42a/` (accept, reject). **Unfinished** (phase-status):
read-through by `operator*`/`->` (no Ember interface makes a type read through, `[TYP-14]`), the
operators of a class template's instantiation, and `Iterable` as a declared interface (std declares
none: `for` finds `iter()` by name).

### N4.4 Exceptions and lifetimes, `[FFI-24c]`, `[FFI-24d]`, `[FFI-11f]`

- `[FFI-24c]`: `ember inspect --cost` gives each C++ call its mode: `noexcept (derived)`,
  `noexcept (asserted)`, `catching`. The import's own wrapper bodies are no longer reported as the
  program's calls. **Not built:** `ember bind --report` listing catching calls on `@noalloc` or
  main-thread paths (N3 question 7: `bind` reads a header, not a program).
- `[FFI-24d]`: the binding record keeps each function whose header added or removed `noexcept`
  since the last parse (and has not changed it back); each error at a call of one, or at a pattern of
  a `match` on one, carries a note naming the header, the declaration and the change. `ember bind
  --report` treats `.hpp`/`.hh`/`.hxx` headers as C++, writes each function's policy into its
  baseline (`api <name> noexcept|throwing`), and lists changes since as API changes (exit 1).
  **Decision:** the change is known against the import's previous record in the cache (or the
  baseline for `bind`); with no previous record there is nothing to compare.
- `[FFI-11f]`: a parameter declared `[[clang::lifetimebound]]` (directly or through a macro that
  expands to it) that the thunk takes by address makes a `T&` result a `ref` borrowing it (`@borrows`,
  with `self` too for a method), so the result cannot outlive it; the wrapper's safety note states the
  fact as asserted and never checked, which `ember tcb` lists. A parameter the thunk copies (a
  `const T&` of a class Ember holds by value) is not bound: its reference would dangle.

Tests: milestones `inspect_reports_each_cpp_call_s_exception_mode`,
`a_noexcept_change_in_a_header_is_named_as_the_cause`, `a_lifetimebound_result_is_an_asserted_fact`;
`FFI-11f/` (accept, reject).

### N4.5 The import table and the trust report, `[FFI-44]`, `[TCB-4]`

`[FFI-44]`: a class with one public, non-virtual C++ base from the header has the base's public
methods (its own names hide the base's, as in C++). Tests: `FFI-44/` (accept: every automatic row
with no overlay; reject: the overlay and native-island rows). **Unfinished:** `std::function` as an
opaque owned object or `@ffi(std_function, signature=…)`, and question 3. `[TCB-4]`: `ember tcb`'s
C++ bridge category names the headers whose thunks the program trusts (it said "C++
interoperability is not built").

### N4.6 Build integration, `[BLD-FFI-1b]`, `[BLD-FFI-4]`, `[BLD-FFI-5]`

- `[BLD-FFI-1b]`: with cl or clang-cl, the runtime switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` an
  import's project flags name are inherited by every translation unit of the target (Ember's C, the
  runtime, shims, thunks, the DLL links of `cdylib` and reload). A `/MDd` project makes a program
  whose own C uses the debug C runtime and none of the release one; the runtime then sends the debug
  runtime's reports to standard error, never to a window (D-252). Imports naming different runtimes
  (implied `_DEBUG`/iterator level included) are `E9020` at the second import; a C++ project whose
  flags pass a response file is `E9021` (question 4). Both are checked for every toolchain. Project
  flags now reach libclang in a form it parses (`/D`, `/U`, `/I` as GCC flags; the runtime switch as
  the macros cl defines for it; other MSVC switches dropped) — `/MDd` had made every header
  unparseable (D-N4-7). Test: package `the_cpp_project_s_runtime_is_inherited`; pages `E9020`, `E9021`.
- `[BLD-FFI-4]`: with GCC or Clang, a project's `-flto` (last of `-flto`, `-flto=…`, `-fno-lto`) is
  inherited by the program's compile and link, so the link inlines across the boundary; Clang on
  Windows links it with `lld-link`. Test: package
  `the_cpp_project_s_link_time_optimisation_reaches_the_program` (on Linux it also checks with `nm`
  that the thunk is gone; run locally with `EMBER_CC=clang`).
- `[BLD-FFI-5]`: `--cpp` writes `<package>.hpp` beside the C header: every export in a namespace
  named for the package (`using ::name;`), plus an inline overload taking a `std::string_view`
  (const bytes) or a `std::span` where the C function takes a pointer and the `usize` after it
  (span overloads only where `__cpp_lib_span`). Test: milestones
  `the_cpp_header_takes_views_where_c_takes_pointer_and_length` (compiles a C++20 user of it).
  `[BLD-FFI-5a]` is not built (N3 question 8: no class handle reaches an exported signature).

## 2. Rules

| rule | state | evidence |
|---|---|---|
| `[FFI-39]`, `[FFI-39a]`, `[FFI-39c]`, `[FFI-39d]`, `[FFI-39e]`, `[FFI-17c]`, `[FFI-17d]` | done | N4.1 |
| `[FFI-39b]` | unfinished: question 1 | `FFI-39/` |
| `[FFI-17e]`, `[FFI-17f]` | done | N4.2 |
| `[FFI-17a]` | unfinished: `std::span`, `string_view`, `optional`, `variant`, `std::function` objects, `weak_ptr`, `unique_ptr` parameters and deleters | `FFI-17a/` |
| `[FFI-42a]` | done | `FFI-42a/` |
| `[FFI-42]` | unfinished: read-through, template operators, `Iterable` | `FFI-42/` |
| `[FFI-24d]`, `[FFI-11f]` | done | N4.4 |
| `[FFI-24c]` | unfinished: N3 question 7 | milestones |
| `[FFI-44]` | unfinished: `std::function` objects, question 3 | `FFI-44/` |
| `[BLD-FFI-1b]`, `[BLD-FFI-4]`, `[BLD-FFI-5]` | done | N4.6 |
| `[BLD-FFI-5a]` | unfinished: N3 question 8 | — |
| `[FFI-48]` | unfinished: modules, coroutines, return-type overloads, unmodelled casts, class-type NTTPs not diagnosed (allocator containers built) | `FFI-48/` |
| `[FFI-40a]`, `[FFI-41]` | unchanged: N3 questions 3 and 4 | — |

Not done here, though it is a Part XVI rule with a C++ half: `[BLD-FFI-2]` (47 words, untested):
the CMake module `ember_add_library`/`ember_add_executable` and `--cc-flags-from-target`. Nothing in
it needs an owner answer.

## 3. Owner questions (option (a) is what is built)

1. **`[FFI-39b]` beside `[CLS-2]`/`[CLS-11]`.** `[FFI-39b]`: "`super.init(…)` selects the C++ base
   constructor … and runs it before any Ember field is initialised"; `[CLS-11]` has `init` store field
   defaults and assign fields before `super.init(…)`. For `class Mine(cpp.Layer): count: i32 = 7` with
   `fn init(self): self.count = 3; super.init(5)`: (a) built: the C++ constructor runs at
   `super.init(5)`, after `count` is set (a C++ virtual the constructor calls cannot reach the Ember
   override in either case: the entries are set after it). (b) Run the C++ constructor first, before
   field defaults, which means `super.init`'s arguments cannot use fields.
2. **`[FFI-42]` beside `[FFI-2]`.** Operators map to interfaces "where one exists", and a header
   function stays unsafe to call until an overlay entry asserts it; an interface's method cannot be
   `unsafe`. (a) built: the interface is implemented when an overlay entry asserts the method by its
   Ember name (`fn add(other)`), which today asserts every class's method of that name (the bare-name
   matching N3 question 3 asks about); otherwise `a.add(b)` is an `unsafe` method. (b) Implement the
   interfaces regardless, the wrapper asserting the call (an assertion the header never made).
3. **`[FFI-44]` beside `[FFI-39]`.** The table says single inheritance from a C++ base is
   "automatic, bounded (`[FFI-39]`)"; `[FFI-39]` needs the overlay's `@ffi(trampoline, virtuals=[…])`.
   (a) built: a C++ class's single C++ base is automatic (its methods are the derived class's); an
   Ember class deriving from a C++ class needs the overlay entry. (b) Make a trampoline for every
   C++ class with virtuals, with every virtual named.
4. **What makes a C++ runtime undeterminable (`E9021`).** (a) built: a project's flags passing a
   response file (`@build.rsp`). (b) Also a project with no runtime switch in its flags (MSVC's
   default would be `/MT`), which would refuse every `[cpp.<project>]` section that sets none.

## 4. Defects found

- **D-N4-1** (fixed, `c1aa7fe2`): a function whose every path panics left its return slot unread,
  which MSVC's `/WX` C gate rejects (`C4101`).
- **D-N4-2** (fixed, `1fe85c40`): reloadable images did not link the imports' shims and thunks.
- **D-N4-3** (open, typeck, outside Annex C): an `unsafe fn` may implement a safe interface method,
  and the operator then calls it with no `unsafe`. Probe: `struct P: x: int`, `extend P implements
  Add: type Output = P; @safety("p") unsafe fn add(self, rhs: P) -> P: …`; `c = P(x=1) + P(x=2)`
  checks clean outside `unsafe:`.
- **D-N4-4** (open, codegen): a struct `A_b` and a method `A.b` of one module get one C symbol
  (`em_<module>_A_b`); the C compiler then reports a redefinition. Worked around by naming the
  iterator `<Class>_cpp_iter`. Probe: a module with `struct Bag_iter` and `extend Bag: fn iter(self)`.
- **D-N4-5** (fixed, `b5a97021`): a namespace-level `operator+` in a header broke the whole import
  (the module had `fn operator+`).
- **D-N4-6** (fixed, `0ccf58ac`, mine): `ember_cpp_object_adopt` was added to the generated runtime
  only; regeneration would have dropped it.
- **D-N4-7** (fixed, `0ccf58ac`): a `[cpp.<project>]` flag in MSVC's form (`/MDd`) reached libclang
  as a file name, and the header could not be parsed.
- **D-N4-8** (fixed, `a3c4d292`): a non-throwing C++ function returning a bridge value with no
  wrapped parameter was declared directly, without the wrapper that makes the bridge value.
- **D-N4-9** (fixed, latest commits): `cpp_span`'s owner temporary was set and never read, which
  clang-cl's C gate rejects; the `.embind` FFI-14 test assumed a map head of one byte.
- **D-N4-10** (not mine, reported): `tasks/impl-0.9.9/annotations.py` has no `#$ command: lint`, so the
  eight `LNT-6` cases (which expect `ember lint`'s `L4002`/`L4003`/`L2001`) fail under it on `main`
  too; `ember lint` reports them and the milestones' conformance runner passes them.

## 5. Notes

- `ember_build`'s inherited runtime and LTO flags are process-wide settings the driver sets before
  each build (`ponytail:` one build per process; an embedder building two programs at once in one
  process would need them per request).
- The FFI-17a `String.from(CppString)` copy goes through a typeck special case (`std.ffi.cpp_string_copy`)
  because the compiler-known `String.from` shadowed the `From[CppString]` implementation.

## 6. The quick check

`annotations.py tests/conformance/*/ tests/compile-fail tests/compile-pass tests/run-pass
tests/run-fail tests/milestones tests/std tests/ffi`: failing 8, all
`tests/conformance/LNT-6/accept_*` (D-N4-10); main at `dde19e54` fails the same eight.
