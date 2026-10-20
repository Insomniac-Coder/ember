# Job N3 — `import cpp`, the first slice of Annex C: notes

Helper agent N, 2026-10-20. Branch `helper-N3`, from `main` at `c040fa5d` (target
0.9.10_Hardened_26). Everything below is committed on the branch and was checked on Windows: the
quick check (`annotations.py` over every test folder, failing 0), the conformance suite inside the
milestones under MSVC and under clang-cl (`EMBER_CC=clang-cl`), `cargo test --profile ci` of
`ember_typeck`, `ember_analysis`, `ember_build` and every `ember_driver` suite (milestones included),
and the gates `rule_index`, `error_pages`, `hardening_check`, `check_branding`, `spec_check`. In WSL,
every C++ test's thunk file compiles with g++ 15 `-std=c++17 -Wall -Wextra -Werror` and its emitted C
with gcc `-Wall -Wextra -Werror`; nothing was run there (WSL has no libclang, so the importer itself
has not run on Linux: the first CI run is the first time libclang parses C++ there).

**Effect on the computed table** (`tools/phase_progress.py`): Overall 91.37% → 92.17% (44356 of
48125 words); Phase 5 77.34% (77 of 116 rules). Ten Annex C rules are done (386 words); five are
recorded unfinished in `docs/phase-status.json` (`FFI-40a`, `FFI-41`, `FFI-48`, `BLD-FFI-4`, and the
existing `FFI-39`). README.md is not edited, so `phase_progress.py --check` reports the table out of
date until it is rewritten on merge.

**For the merge.** C++ thunks are compiled by the backend's C++ driver: `cl /TP /EHsc` or clang-cl,
`g++` or `clang++` beside `gcc`/`clang` (the name with `gcc`→`g++`, `clang`→`clang++`), and a GNU
link adds `stdc++` (`c++` on macOS). The documents job's `error_pages.py` now runs eight pages whose
examples `import cpp` (needs libclang and the C++ standard headers, which Ubuntu's runner has). The
`.embind` record's key now holds the import's instantiations (D-N3-4), so older records are not
reused.

## 1. What was built, feature by feature

### N3.1 The import (`compiler/ember_driver/src/cimport.rs`), `[FFI-17]`, `[FFI-3]`, `[FFI-50]`

`import cpp "h.hpp" with (project = "…", overlay = …, instantiate = […])` goes through the same
importer as `import c`, with Clang told `-x c++ -std=<standard>`: the header becomes an Ember module
written as source (`cpp.h_<stem>`), and every function, method, constructor and destructor is reached
through an `extern "C" … noexcept` thunk in `<stem>.thunks.cpp`, which the build compiles as C++
(`[FFI-3]`: no mangled symbol is ever linked; a function the header declares `extern "C"` is called
directly, as by `import c`). The thunk file starts with a prelude: a thread-local slot for the last
caught exception's type name (demangled through `<cxxabi.h>` where the library has it, else with
MSVC's `class `/`struct ` stripped) and `what()`, read by two `extern "C"` accessors, and `zero<T>()`
for the value a catching thunk returns after a throw.

`[cpp.<project>]` (`[FFI-50]`): `compiler` (`msvc`, `clang-cl`, `clang`, `gcc`), `standard`
(default `c++17`), `defines`, `include_paths`, `flags`, and `cmake = { build_dir, target }`, which
reads the target's C++ compile group from CMake's File API reply (newest index, codemodel, target
reply: defines, include folders made absolute, command fragments, `languageStandard`); with no reply
yet, the query file is written and the error says to configure again. `ember bind h.hpp --emit-cpp
[--std c++20]` writes the thunk file beside the header.

### N3.2 Exception policies, `[FFI-24]`, `[FFI-24b]`, `[FFI-43]`

Derived from the declaration: `noexcept`, `throw()`, `__declspec(nothrow)`, a destructor and a
defaulted special member are *direct* (the Ember function returns `T`); no specification,
`throw(X)`, `throw(...)` are *catching* (`Result[T, CppError]`, `CppError { type_name, what }` in
`std.ffi`; anything that is no `std::exception` is caught too, with an empty `what`). A
`noexcept(expr)` — and a specification an instantiation has not evaluated yet — is asked of Clang in
one more parse: `constexpr bool ember_cpp_fact_k = noexcept(<call>);` with values of the parameters'
types (`std::declval<C&>().m(…)`, `C(…)`, `::f(…)`), for functions, methods, constructors and
template instantiations. An overlay's `@ffi(noexcept)` asserts a throwing function non-throwing: the
thunk still catches and a throw panics naming the function (`[FFI-24b]`);
`@ffi(noexcept=false)` declares one throwing (a header's non-throwing word stands over it). `E5062`
for an entry saying both, and for a second overlay's entry (without `override`) contradicting an
earlier one; `E5061` where a function is used whose policy Clang could not evaluate and no overlay
declares (reached when the project is freestanding, `-nostdinc`: the question finds no
`std::declval`).

### N3.3 Classes, `[FFI-32]`, `[FFI-36]`, `[FFI-40]`, `[FFI-40a]`, `[FFI-41]`

A class that is standard-layout and trivially copyable (asked of Clang: `__is_trivially_copyable(T)
&& __is_standard_layout(T)`), all of whose fields map, is a `@layout(c) @derive(Copy)` struct with
Clang's offsets checked; its constructor is `T.new(…) -> T`. Any other class is an opaque
`@ffi(destroys(…)) type`, made only through `T.new(…) -> ForeignBox[T]` (`E5032` for anything else),
freed by a destructor thunk (the class's own when public, else the implicit one; no constructor is
imported when the destructor is not public). Class results go through an out-pointer and placement
`new` (MSVC returns a class with a constructor in memory, where C returns a struct of its size in
registers). `ForeignBox.get()`/`get_mut()` give `ref T`/`ref mut T` (compiler-known in typeck).
A raw pointer result is `*mut T`, adopted with `adopt[T](p)` (`[FFI-36]`).

Methods: `const` → `self`, other → `mut self`, every `const` method of a class with a `mutable`
member → `mut self` (`[FFI-40a]`), `&&`-qualified → `W5033`, static → associated function,
operators and overloads → `W5002`. Parameters: `T&&` → `owned T` (a by-value class) or the `owned
ForeignBox[T]` the callee moves from (an opaque class that can be moved, asked of Clang), `const T&` →
borrowed `T` or `ref T`, `T&` → `mut T` or `ref mut T` (an opaque class is incomplete and takes no
parameter mode, `E5015`). A method's `T&` result is a `ref mut T` borrowing its receiver (a `ref T`
for a `const T&` or from a `const` method); a free or static function's stays a raw pointer. The
reference is made in the wrapper by a conversion only a module `import cpp` wrote can name, and the
region analysis gives a reference made through a local raw pointer no provenance of its own, so the
signature's elision ties it to the receiver (`[LT-1]`): the object stays borrowed while it lives
(`FFI-40/reject_a_reference_result_borrows_the_receiver.em`).

Namespaces are child modules (`engine::math::dot` is `cpp.engine.math.dot`), a type keeps its
identity across them, an anonymous namespace's entities are `W5034`. A nested type is named
`Outer_Inner` (every public one is imported); a constant static data member is its class's constant
(`Grid.width`); any other, and a namespace variable of C++ linkage, is an associated or module
function giving its address (`Grid.made() -> *mut c_int`), see question 4. Multiple inheritance and
virtual bases are `E5034` (`[FFI-48]`).

### N3.4 Templates, `[FFI-17b]`

`instantiate = ["geo::Pair<int>", "biggest<double>"]`: the import file adds `template class
::geo::Pair<int>; using ember_inst_0 = …;` or `constexpr auto ember_inst_1 = &::biggest<double>;`,
and each is imported under a name made of its template-id (`Pair_int`, `biggest_double`), in its
template's namespace. A class instantiation's fields come from `clang_Type_visitFields`, and its
methods from the address of each public method the template declares once (`&::geo::Pair<int>::sum`:
Clang's visitor shows an instantiation no members); its constructors, operators and overloads are
`W5002`. Any other template is skipped, and naming it is `E5055`.

### N3.5 Error pages

`docs/errors/` E5031, E5032, E5034, E5055, E5061, E5062, W5033, W5034, each with a failing example, the
rendered diagnostic, why the rule exists, and a fix that compiles (`error_pages.py` runs them).

## 2. Rules done, with their tests

| Rule | Tests (under `tests/conformance/` unless said) |
|---|---|
| `[FFI-3]` | `FFI-17/accept_a_cpp_header_is_called_through_thunks.em`, `FFI-41/accept_nested_types_and_static_data_members.em`, milestone `bind_emit_cpp_writes_the_thunk_file` |
| `[FFI-17]` | `FFI-17/…`, `FFI-32/…`, `FFI-40/…`, `FFI-41/accept_namespaces_are_module_paths.em` |
| `[FFI-17b]` | `FFI-17b/accept_explicit_instantiations_are_imported.em`, `FFI-17b/reject_a_template_not_instantiated.em` |
| `[FFI-24]` | `FFI-24/accept_the_policy_is_derived_from_the_declaration.em` (computed `noexcept` on functions, a constructor, methods and an instantiation), `FFI-43/…` |
| `[FFI-24b]` | `FFI-24/accept_an_overlay_asserts_noexcept.em`, `FFI-24/run_fail_an_asserted_noexcept_function_threw.em` |
| `[FFI-32]` | `FFI-32/accept_classes_by_value_and_opaque.em`, `FFI-32/reject_an_opaque_class_is_made_only_by_its_constructors.em`, `FFI-32/reject_a_parameter_that_maps_to_nothing.em` |
| `[FFI-36]` | `FFI-36/accept_a_raw_pointer_result_is_adopted.em` |
| `[FFI-40]` | `FFI-40/accept_methods_and_references.em`, `FFI-40/reject_a_reference_result_borrows_the_receiver.em` |
| `[FFI-43]` | `FFI-43/reject_contradictory_exception_policies.em`, `FFI-43/reject_composed_overlays_that_disagree.em`, `FFI-43/reject_a_policy_neither_derived_nor_declared.em`, `FFI-43/accept_a_declared_throwing_function_is_caught.em` |
| `[FFI-50]` | `FFI-50/accept_the_project_configures_the_import.em` (by hand and from a CMake File API reply), `FFI-50/reject_a_project_the_manifest_has_not.em`, milestone `bind_emit_cpp_writes_the_thunk_file` |

## 3. Rules not done, and why

| Rule | State |
|---|---|
| `[FFI-40a]` | unfinished: the assumption for a class with a `mutable` member is built and tested (`FFI-40a/`); declaring `@ffi(invalidates)` on one method, or saying otherwise, needs an overlay entry naming a method, which `[GRM-35]` has not (question 3) |
| `[FFI-41]` | unfinished: a static data member is not an `extern static` (question 4); nested types are `Outer_Inner` (question 5) |
| `[FFI-48]` | unfinished: only the inheritance row is diagnosed; the virtuals, modules, coroutines, return-type-overload, cast, class-type-NTTP and allocator rows are not (several wait on `[FFI-39]` and `[FFI-17a]`) |
| `[BLD-FFI-4]` | unfinished: the thunks are compiled by Ember's own C compiler (fixed here: they were compiled by the `[c]` section's) with its profile's flags, and a different `[cpp]` compiler is refused, tested in `ember_driver/tests/package.rs`; on GCC and Clang Ember's C is never compiled for LTO, so a project's `-flto` reaches the thunks only. Passing it through needs a flag in `ember_build::LinkRequest` (43 constructions) |
| `[FFI-17a]`, `[FFI-17e]`, `[FFI-17f]` | not built: the bridge types (`CppString`, `CppVector[T]`, `CppShared[T]`, `unique_ptr`/`optional`/`span`/`string_view` mappings); a standard-library type is skipped naming `[FFI-17a]` |
| `[FFI-24c]` | not built: the per-call mode report could be a `FFI call` row of `ember inspect --cost`; the second half (question 7) is unclear |
| `[FFI-24d]` | not built: needs the previous binding as a baseline for a "changed by the header" note |
| `[FFI-42]`, `[FFI-42a]` | not built: operator interfaces and iterators; now possible since a method's `T&` result is a `ref` |
| `[FFI-11f]` | not built: `[[clang::lifetimebound]]` as an asserted fact |
| `[FFI-44]` | not built as a rule: its table's rows are the other rules' |
| `[FFI-39]`… `[FFI-39e]`, `[FFI-17c]`, `[FFI-17d]` | not built: trampolines (`extern class`) |
| `[BLD-FFI-1b]` | not built: the MSVC runtime switch and `_DEBUG`/`_ITERATOR_DEBUG_LEVEL` inherited from the project (Ember builds `/MD` in every profile), `E9020`/`E9021` |
| `[BLD-FFI-5]`, `[BLD-FFI-5a]` | not built: the `--cpp` header; question 8 |
| `[CXX-1]` | not built: the header corpus and a third-party library need the owner's choice of library |

## 4. Defects found and fixed on the way

- **D-N3-1** `null[P]()` and `ref_to_ptr(r)` were refused outside the root module (typeck looked them
  up as program declarations): every module has them now
  (`tests/run-pass/raw_pointer_builtins_in_any_module.em`).
- **D-N3-2** extending an `extern type` of one's own module was `E2120` (`declared_in_this_package`
  did not see opaque types).
- **D-N3-3** type aliases were resolved module by module in reverse list order, which is no
  dependency order: a program importing `c_int` itself plus an `import c` of a header with `typedef
  int t;` failed with `E1010` inside the generated module. All modules' aliases now resolve together
  to one fixpoint (`FFI-8/accept_std_ffi_c_aliases_are_the_targets_c_types.em` covers it).
- **D-N3-4** the `.embind` key did not hold `instantiate=[…]`: an import without it reused the
  binding of one with it.
- **D-N3-5** a type's constant named through its module (`geometry.Vec2.ZERO`) was `E1010`
  (`GRM-24/accept_dot_paths_through_modules.em`, `MOD-2/reject_a_private_constant_of_a_type.em`).
- **D-N3-6** a namespace variable of C++ linkage was declared as a C `extern static` and could not
  link; it is reached through a thunk now.
- **D-N3-7** a C++ class passed by value that is opaque by design was reported as "declared but never
  defined".

The region-analysis change (a reference made through a local raw pointer has the signature's
provenance; it is not `E3060` to return it) is general, and only the `import cpp` conversion makes
such a reference today: Ember has no syntax for a raw pointer's pointee as a place.

## 5. Questions for the owner

Each has a default built; the letter (a) is always what is built.

1. **Instantiation names** (`[FFI-17b]` gives none). Built: (a) the template-id's identifier
   characters joined by `_` (`geo::Pair<int>` → `geo.Pair_int`, `Pair<int, float>` → `Pair_int_float`).
   (b) Require an overlay `rename` for each. (c) Another spelling.
2. **Declaring a function throwing.** `[FFI-24b]` spells the assertion `@ffi(noexcept)`; `[FFI-43]`
   speaks of an overlay marking a function "throwing" without a spelling. Built: (a)
   `@ffi(noexcept=false)`. (b) `@ffi(throws)`. (c) Another.
3. **Naming a method in an overlay** (`[FFI-40a]`'s `@ffi(invalidates)`). `[GRM-35]`'s `overlay_fn`
   takes one IDENT, so no entry can name `Cache::lookup`. Built: (a) nothing; the class-wide assumption
   only. (b) `fn Cache.lookup(self, key)` — a dotted name in `overlay_fn`. (c) Entries inside an
   `extend Cache:` block of the overlay. (d) A type-level attribute, `@ffi(invalidates=[lookup])` on
   `type Cache`.
4. **Static data members as `extern static`** (`[FFI-41]`). `[FFI-3]` forbids the member's mangled
   symbol, and an `extern static` names a C symbol that is the object itself; an `extern static`
   holding its address would be a raw pointer in a static, which is not `Sync` (`E7002`). Built: (a) an
   associated function giving the address, `Grid.made() -> *mut c_int`, the form a static member
   function returning `int&` already has; a constant member is the class's constant. (b) An `extern
   static` of the address, exempt from `E7002`. (c) Amend `[FFI-41]` to say (a).
5. **Nested types "under their outer type's name"** (`[FFI-41]`): Ember has no nested types. Built:
   (a) `Outer_Inner`. (b) A child module per class, `cpp.Outer.Inner`, which needs a type and a module
   of one name to coexist. (c) Another.
6. **Composed overlays that disagree** (`[FFI-43]`'s `E5062` beside ODR-149's `E5010`). Built: (a) a
   second entry without `override` that declares the opposite policy is `E5062`; one that declares
   the same policy or none is `E5010`; with `override` it replaces the first. (b) Every second entry is
   `E5010`, and `E5062` only within one entry.
7. **`ember bind --report` and catching calls** (`[FFI-24c]`): `bind` reads a header, not a program,
   so it cannot know which calls are on `@noalloc` or main-thread paths. (a) Not built. (b) The report
   belongs to `ember build --report` (`[CLI-10]`). (c) `bind --report` lists every catching function,
   for a reviewer to match against such paths.
8. **`[BLD-FFI-5a]` beside `[FFI-31b]`**: no class handle may appear in an exported signature, so the
   C++ header has no class handle to give a smart handle to. (a) Not built. (b) `[FFI-31b]` allows
   `@sync` class handles in exports with the `--cpp` header. (c) `[BLD-FFI-5a]` waits for a later
   version.
9. **A defaulted special member** (`[FFI-24]` lists it among the non-throwing declarations). C++ makes
   a defaulted constructor `noexcept(false)` when a member's initialiser may throw
   (`std::vector<int> cells = std::vector<int>(64);` can throw `bad_alloc`). Built: (a) as the rule
   says, direct, so such a throw ends in `std::terminate` at the thunk. (b) Clang's answer for the
   defaulted member (the probe already asks for unevaluated specifications), which makes it catching
   when C++ says it may throw.
