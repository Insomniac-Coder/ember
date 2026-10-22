# Helper T — C++ interop: D-808 and `[FFI-17a]`'s bridges (notes)

Helper T, 2026-10-22. Branch `worktree-agent-ac63b86829223e483`, fast-forwarded to `main` at
`7b4a5854` (the brief's corpus and D-808 are there; the worktree had been made at `b8c4df20`).
Target 0.9.10_Hardened_33.

## T1. D-808 — an overlay's contract words on a C++ function reached through a wrapper

**Done.** An entry that states every fact (no `TODO(…)`, no `unsafe`) now contracts the thunk's own
declaration, as a C function's is: `render_contracted` (cimport.rs) writes it `safe fn` with
`@ffi(param(…), result(…))`, so the checker's existing lowering does the work (a `count(n)` witness
and an `out` leave the callable signature; a `Span` gives the pointer and length, with its length
checks). What the importer passes itself is a `borrowed one` reference of the same declaration:

| slot | declared as | the wrapper passes |
|---|---|---|
| receiver | `ember_self: ref T` / `ref mut T` (`from(self)` in a result word becomes `from(ember_self)`) | `ref self` / `ref mut self`; a trampoline's `ember_cpp_ref(self.ember_cpp_base.object)` |
| opaque `const T&` / `T&` | `p: ref T` / `ref mut T` | `p` |
| opaque `T&&` | `p: ref mut T` | `p.get_mut()` (the `ForeignBox` frees what is left, as before) |
| C++ `T&` of a type by value | `mut p: T` with `param(p, borrowed, one, exclusive)` | `p` |
| bridge value (`std::vector&` …) | `p: ref CppObject` / `ref mut CppObject` | `ember_cpp_ref[_mut](p.as_ptr() as *CppObject)` |
| trampoline object, `Retained` token | `p: ref T` / `ref mut T` of the opaque C++ class | `ember_cpp_ref[_mut](….ember_cpp_base.object)` |
| class result by value | `mut ember_out: T` with `param(ember_out, out, one)` | (returned by the call) |
| where the thunk says it caught | `mut ember_failed: c_int`, `borrowed one exclusive` | the wrapper's local |

The wrapper's parameters are then the safe types (`safe_parameter`), and it needs `unsafe` only to
name an object by its address (a bridge value's, a trampoline's) or to own a new object the thunk
gives (`adopt`, `from_thunks`). Outputs follow the checker's rule (C++ result first, then each `out`
or the `inout_count`'s count; a `status` result is `Result[outs, N]`); a class by value, which the
thunk writes last, is put first. An entry's own `@ffi(…)` arguments (`effects=[…]`) now reach the
declaration too (they were dropped for wrapped functions). A `class C:` method line is compared with
the contracted wrapper's signature when an entry contracts the method (class sections are now applied
after the overlay's other items).

An entry that leaves a pointer without facts is now `E5002` on a wrapped C++ function, as on a C
function and a `noexcept` free one (before, its wrapper silently stayed `unsafe`).

Tests: `tests/conformance/FFI-17/accept_contract_words_reach_a_wrapped_function.em` (run-pass, no
`unsafe`: throwing functions and methods with `count(n)` spans, a `MutSpan`, a `cstr`, a `from(self)`
reference result, a `T&` beside a span, a bridge value lent mutably beside a span, a class by value,
an opaque class by value and a `std::string` as results, two spans sharing a length, a constructor,
and a `class` method line restating a contracted method), `run_fail_two_counted_views_of_unequal_lengths.em`
(the length check before C++ runs), `reject_a_count_naming_no_length.em` (`E5002`); `texts.hpp`,
`overlays/texts.em`, `overlays/texts_unmatched.em`. Break-tested: the method line written with the
raw signature is `E5010`, naming the contracted one.

`tests/cxx-corpus/program/main.em` now passes its text through `json_api.overlay.em` (`parse`,
`count`, and nlohmann's `is_object`, `size`, `dump`): the program has no `unsafe:` left.
`the_cpp_corpus_binds_as_recorded` copies the overlay into its package (package.rs). D-808's words
are out of `[FFI-11]`'s `unfinished` entry.

### Compiler changes for T1

- `compiler/ember_typeck/src/lib.rs` — `foreign_referent`, `foreign_mut_one`: a `one` contract's
  reference may refer to an incomplete type (`extern type`), and `borrowed one exclusive` is
  `ref mut T` (the `[FFI-11]` table's own form) as well as `mut p: T`. **This fixes a C gap too:**
  a C header's opaque handle (`typedef struct Ctx Ctx; int ctx_get(const Ctx*)`) contracted
  `borrowed one` was `E5015`/`E5002` (`ref Ctx` "refers to a type with no foreign
  representation", `mut ctx: Ctx` "incomplete"). Test: `tests/conformance/FFI-11/accept_an_opaque_handle_is_borrowed_as_one.em`
  (with `handles.c`, `handles.h`, `overlays/handles.em`): `ref`, `ref mut`, nullable of each, a
  result `from(counter)`, no `unsafe`.
- `compiler/ember_driver/src/cimport.rs`:
  - `CType.opaque` (a data pointer to an incomplete type), set in `c_type`; `safe_parameter` gives
    such a pointer's exclusive `one` as `ref mut T` (an incomplete type takes no parameter mode).
    `.embind` version 10 (the field is recorded).
  - `canonical`, `slot_words`: an overlay's words are written into `param(…)`/`result(…)` in the
    order the checker reads them (ownership, count, the rest). **Defect found:** an overlay
    writing `nullable borrowed one` (any order but ownership-count-rest) gave `E0900` "this
    `param(…)` contract is not implemented yet": the checker's parsers are positional
    (`ffi_param_contract_arg` and its neighbours), and the importer copied the overlay's order. The
    importer now canonicalises; hand declarations are still positional (see Defects).
  - `slot_words` also derives `borrowed one exclusive` for a C++ `T&` of a type by value (`mut p:
    T`) when the entry writes nothing: a `noexcept` free C++ function with an `int&` and an entry
    was `E5002` before ("safe fn needs an @ffi contract for mutable parameter").
  - `ADDRESS` (`std::uintptr_t`): a thunk gives the address of an object the caller comes to own
    (an opaque class by value or from its constructor, a bridge value's object, a released
    `std::unique_ptr`, an iteration's range) as an integer, `usize` to Ember, which the wrapper
    turns into the owner (`adopt[T](… as *mut T)`, `CppVector.from_thunks(… as *mut u8, …)`). Why:
    a contracted (`safe`) declaration may not return a raw pointer (`[FFI-10]`), and the only safe
    pointer results are `ref`/`Span`/`cstr`/`ForeignBox` — a `ForeignBox` for a bridge value would
    add the runtime's adoption record (`ember_ffi_adopt`) to every `std::string` result, which a
    C++ caller does not pay; a `ref … from(static)` would be a false declaration. An integer is the
    honest type ("a scalar can be an address", `[FFI-2]`) and costs nothing (one register, a cast
    the C compiler drops). The thunk declares the same type, so C and C++ agree under LTO
    (`-Wlto-type-mismatch` would flag a pointer/integer pair). The thunk prelude includes
    `<cstdint>`.
  - `render_contracted` (new), `contracts_everything`, `signature_of`; `render_wrapper` dispatches
    to it; `render` finds a status enum's error type for it; `method_value` gives `ForeignBox[T]`
    for an opaque class by value.
- `std/src/ffi.em` — `CppObject`, an `extern type` in `std.ffi`: the C++ object a bridge value
  owns, as a contracted declaration takes it (only behind a reference).
- `tests/cxx-corpus/nlohmann_json/*.thunks.cpp` — re-recorded for the `ADDRESS` change (6 thunks and
  the include). The Windows record was written by `record.py`; the Linux one (no libclang here) by
  the same textual rewrite, which reproduces the Windows record byte for byte from its old text; the
  two diffs are identical. **The lead should re-record on Linux** (`python tests/cxx-corpus/record.py
  <linux ember>`) if CI's Linux run disagrees.

## T2. `[FFI-17a]`'s bridges

First a refactor (`ea74c514`): `render_wrapper` writes the contracted and the uncontracted
declaration and wrapper from one list of slots, so each mode the importer passes in its own way is
written once; `render_contracted` is gone, and an `[FFI-40a]` method line is checked against the
signature the wrapper actually has (`method_value` is gone).

### T2.1 `std::string_view`, `std::span<T>`, `std::span<const T>` — done

Zero-copy both ways. A parameter (by value or `const&`) is the wrapper's `Span[u8]` /
`Span[T]` / `MutSpan[T]`; the thunk takes its pointer and length (`const char* a0, std::size_t
a0_size`) and makes the view the callee takes. Contracted, the declaration takes the `Span` with
`param(p, borrowed, count(ember_p_size))` and the witness, so the checker lowers it (its length
checks, its empty-view sentinel); uncontracted, `p.as_ptr()` and `p.len()`. A result: the thunk
gives the data's address (`ADDRESS`) and writes the length (`std::size_t* size`); the wrapper makes
the view with `ember_cpp_span(owner, address, length)` (new, compiler-known in an `import cpp`
module only, `unsafe`, typeck `synth_cpp_span`: `Builtin::CppSpan`, borrowing `owner`; the two-argument
form is `static`) from what the entry's lifetime word names — `from(self)`, `from(<parameter>)`,
`from(static)` (`[FFI-11]`: a result's lifetime is never derived) — and its wrapper has
`@borrows(…)`. An entry for a view result without a lifetime word is `E5002` (apply). With no
entry, a view result is `(*T, usize)`, the caller's to keep (`[FFI-38]`). `std::span<T, N>` of fixed
extent as a parameter is refused (`W5002`, naming why); as a result it is read as any span. A view
lent mutably (`std::string_view&`) is refused.

### T2.2 `std::optional<T>` — done

For a `T` that maps by value. A parameter (by value or `const&`) is `Option[T]`; the thunk takes a
pointer to the value, null for none (`(a0 ? std::optional<T>(*a0) : std::optional<T>())`).
Contracted: `param(p, borrowed, one, nullable)` on `Option[ref T]`, the wrapper passing
`p.as_ref()`; uncontracted: a `match` with `Some(ref inner)` gives the address. A result: the thunk
says whether there is a value and writes it (`T* value`, zeroed when there is none or it caught);
the wrapper gives `Some(v) if has else None`. An optional lent mutably is refused.

### T2.3 a `std::unique_ptr<T>` parameter — done

By value or `&&`: the wrapper takes `owned p: ForeignBox[T]` and the thunk makes the
`std::unique_ptr<T>` C++ owns from the object's address. Contracted: `param(p, owned, one)`, which the
checker lowers (the box leaves the runtime's record and frees nothing); uncontracted:
`p.into_raw()`, a new `std.ffi` method (`pub fn into_raw(owned self) -> *mut T`: the record released,
the box forgotten — what `owned` does). By reference (`std::unique_ptr<T>&`) is refused: no ownership
crosses. A `std::unique_ptr` with a deleter of its own is still refused, as before.

### T2.4 `std::weak_ptr<T>` — done

`CppWeak[T]` (`std.ffi`, `[WK-14]`: never a `Weak`): `lock() -> Option[CppShared[T]]` (the thunk
makes a `std::shared_ptr<T>` only while the object lives, `None` otherwise), `expired()`, `clone()`
(C++ counts it), its end deletes it. It carries the thunks of `std::shared_ptr<T>` for what `lock()`
gives, so it is eight words, as each bridge type holds its thunks inline (N4's design; a per-type
table of thunks would make every bridge value two words — an improvement for all of them, not done
here). It crosses as a result and by `const&`, `&`, `&&` or value like the other bridge types;
`ember inspect --cost` lists its operations (`[FFI-17f]`, `bridge_type` in main.rs). Test:
`accept_a_weak_pointer_observes_a_shared_one.em` (`library.hpp` gains `watch`).

Tests (T2.1–T2.3): `tests/conformance/FFI-17a/views.hpp` (C++20: the directory's `ember.toml`
`[cpp.views] standard = "c++20"`), `accept_views_and_optional_values_cross_safely.em` (contracted:
no `unsafe`; views both ways, `from(self)`, `from(text)`, `from(static)`, `MutSpan` written through,
optionals both ways, a `std::unique_ptr` given to C++ and destroyed there once),
`accept_views_cross_in_unsafe_without_an_overlay.em` (uncontracted), `reject_a_view_result_without_a_lifetime_word.em`
(`E5002`), `reject_a_view_cannot_outlive_what_it_borrows.em` (`E3060`: a `from(self)` view cannot
outlive the `ForeignBox`).

## Defects found (for the lead to number)

1. **Hand declarations read contract words by position.** `@ffi(param(p, nullable, borrowed,
   one))` is `E0900` ("this `param(…)` contract is not implemented yet"); only `param(p, borrowed,
   one, nullable)` is read. `[FFI-10]`/`[FFI-11]` give no order (an overlay's words are "separated
   by spaces", any order). The importer now writes overlays' words in the checker's order, so
   imports are fixed; a hand declaration is not. Fix: read each `param(…)`/`result(…)` as a set of
   words (`ffi_param_contract_arg`, `ffi_result_contract_arg`, `ffi_owned_param_arg`,
   `ffi_retained_param_arg`, `ffi_owned_result_arg`, `ffi_missing_count_arg`,
   `ffi_result_missing_count_arg` in ember_typeck). Open.
2. **An opaque handle could not be borrowed `one`** (C and C++): fixed (T1, typeck).
3. **A `noexcept` free C++ function with a `T&` of a type by value and an entry was `E5002`**:
   fixed (T1, `slot_words`).
4. **An entry's `@ffi(effects=[…])` was dropped for a C++ function reached through a wrapper**:
   fixed for contracted entries (T1); an entry with `TODO`/`unsafe` words still drops it (its
   raw declaration is written without the entry, as before).
5. **Names the importer gives its own slots could collide with a C++ parameter's** (`this`, `out`,
   `failed`, and the wrapper's locals `failed`, `slot`, `result`): a throwing C++ function with a
   parameter named `failed` or `result` declared it twice. Fixed: `ember_self`, `ember_out`,
   `ember_failed`, `ember_slot`, `ember_result` in both the contracted and the uncontracted
   declaration and wrapper (`fill(int* out, …)` in `texts.hpp` takes a parameter named `out`).

6. **`[LT-40]` — two programs in one folder that reach different instantiations of a module's
   private generic stop with an internal error** (pre-existing; `main`'s compiler does it too):
   "cannot load interface cache: cached callable-region metadata disagrees with the current verified
   summary for `m`". Probe, in one folder: `m.em` = `pub fn digit() -> Option[i32]: return some(true,
   7)`, `pub fn plain() -> i32: return 3`, `fn some[T](has: bool, owned value: T) -> Option[T]: …`;
   `a.em` = `import m` + `println(m.plain())`; `b.em` = `import m` + `println(m.digit().unwrap_or(0))`;
   `ember check a.em`, then `ember check b.em`. With `some` not generic there is no error. An
   `import cpp` module is such a module (its catching wrappers call the generic
   `ember_cpp_result[T]`), so two tests of one folder that import one header with one overlay and
   call different functions can fail the quick check (`annotations.py` checks every file into the
   repository's shared `target/`); the milestones harness gives each command its own `--out-dir`, so
   CI does not see it. I gave the new tests that share a header distinct overlays (`texts_dot.em`,
   `views_first.em`) and wrote no new generic helper into import modules (an optional's value is
   `Some(v) if has else None`, inline). Open.
7. **A bridge value's C++ object is a heap allocation a C++ caller would not make** (pre-existing,
   N4's design, which `CppWeak` follows): a `std::string` result is moved into `new
   std::string(…)` so the `CppString` can own it by pointer, and each bridge value carries its thunks
   inline (four to eight words). The allocation has a reason: Ember moves a value by copying its
   bytes, and a C++ object need not survive that (libstdc++'s `std::string` points into itself), so it
   needs an address of its own. It is not needed for a type every standard library lays out as
   pointers that may be moved bytewise (`std::shared_ptr`, `std::weak_ptr`, `std::unique_ptr`, a
   `std::vector` in practice), which could live inside the Ember value (storage of the `sizeof` and
   `alignof` Clang gives, constructed into by the thunk as a class by value is through `out`), and the
   thunks could be one pointer to a per-type table. A design question for the owner's rule ("no
   allocation the C++ does not make"): which types count as movable bytewise is a promise the C++
   standard does not make.

## Spec questions

1. **`[FFI-17a]` `std::variant<…>`: how does an overlay name each alternative?** The table says
   "a generated enum, when every alternative maps and the overlay names each", and `[GRM-35]` has
   no form that names a variant's alternatives (`enum IDENT` names an enum, nothing more). Options:
   (a) an attribute on the `type` entry for the header's alias of the variant, the names in the
   alternatives' order: `using Shape = std::variant<Circle, Square>;` with
   `@ffi(variant=["Circle", "Square"]) type Shape` gives `enum Shape: Circle(Circle), Square(Square)`;
   a variant with no alias cannot be named, and is refused (`W5002`). It crosses by value both ways:
   the thunk takes or writes the tag and the payload. (b) A section restating it in Ember, as
   `class C:` restates methods: `enum Shape:` with a line per alternative — a grammar change.
   (c) Names from the alternatives' types, with no overlay — contradicts "the overlay names each",
   and `int`/`double` give no names. **Recommendation: (a)** — the existing grammar (`@ffi(…)` on a
   `type` item), and a variant is only nameable through the alias a header gives it anyway. Not built.
2. **`[FFI-17a]` `std::function`: what is "an opaque owned object", and what does
   `@ffi(std_function, signature=…)` give, where is it written, and in what syntax?** `[FFI-44]`
   puts `std::function` in the overlay row. Options: (a) a `std::function<R(A…)>` result is, with no
   statement, a `CppFunction` (`std.ffi`, owned, dropped through its destructor, not callable); an
   entry `@ffi(std_function) fn make_adder(n)` makes it an Ember `owned fn(A…) -> R` value whose
   capture is the C++ object (each call one thunk call, no copy; its drop the destructor), the
   signature read from the template argument, `signature="fn(i32) -> i32"` stating it only where the
   header spells the type through something the importer cannot read. (b) Always the opaque object,
   with a `call` method generated per signature (`CppFunction_i32_i32.call(a0)`): a type per
   signature where Ember has callable types already. **Recommendation: (a)**: a C++ callable becomes
   an Ember callable, called as any other. Not built.
3. **`std::span<T, N>` of fixed extent** is not in the table (its rows are `std::span<T>` and
   `std::span<const T>`, whose extent is dynamic). Built: refused as a parameter (a `Span` could be
   of another length, and `std::span<T, N>(p, n)` with `n != N` is undefined behaviour); read as any
   span as a result. Option: `ref [T; N]` / `ref mut [T; N]` both ways (the type says the length, no
   check needed). **Recommendation:** that option, as an addition to the table.
