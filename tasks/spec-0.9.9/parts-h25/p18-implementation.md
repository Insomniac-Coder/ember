---

# Part XVIII — Requirements on Implementations

This Part states what every implementation MUST guarantee about the code it produces and the
programs it accepts. How a compiler is organised internally is not specified.

## XVIII.1 The C backend

The reference implementation compiles Ember to C11 and hands it to the platform's C compiler.

* `[CG-C-1]` *(changed in 0.9.9)* **The emitted C has no undefined behaviour.** Checked signed
  arithmetic uses the compiler's overflow builtins (`__builtin_add_overflow` and friends; checked
  helpers on MSVC); wrapping arithmetic is done in the unsigned type and converted back; shifts are
  range-checked before they execute; `MIN // -1` and `MIN % -1` are handled before the C operator
  runs; type punning uses `memcpy`; no object is accessed through an lvalue of an incompatible type.
  The C compiles without warnings under `-std=c11 -Wall -Wextra` (Clang, GCC) and `/W3` (MSVC).
* `[CG-C-2]` *(changed in 0.9.9)* **Accepted programs compile.** A program Ember accepts never produces
  C that the C compiler rejects or warns about; such a case is a compiler defect (`[TST-27]`), as is
  an internal compiler error on any input. The emitted C is deterministic: the same input gives the
  same text.
* `[CG-C-11]` *(new in 0.9.9)* **Floating-point flags.** Every translation unit begins with
  `#pragma STDC FP_CONTRACT OFF` where the compiler implements it (Clang; GCC does not, and warns
  about it, so it has the flag alone, ODR-044) and is compiled with `-ffp-contract=off -fno-fast-math` (Clang, GCC)
  or `/fp:precise` with `#pragma fp_contract(off)` (MSVC), with SSE2 rather than x87 on 32-bit x86,
  and without flush-to-zero. A `@fastmath` or `@fp(contract)` function is emitted in a separate
  translation unit compiled with the relaxed flags, and is not placed in the inline header of
  `[CG-C-3]`, where it would lose them. This is what makes `[TYP-9]` and `[DET-5]` true.
* `[CG-C-3]` **Cross-module inlining.** Because each module is one translation unit, the backend emits
  per package an inline header, included by every module of the package and of its dependants, holding
  a `static inline` definition of every function that is `@inline`; an operator implementation on a
  type of at most 64 bytes; `len`, `is_empty`, `as_span`, `iter`, `next`, a field accessor or a
  read-through method of a standard view or container; or any other function of at most 40 statements
  that makes no foreign call. So the C compiler inlines across modules without LTO. A function emitted
  there is not also emitted with external linkage unless it is exported or its address is taken.
* `[CG-C-3b]` The bodies it exports are part of the interface hash (`[BLD-2]`).
* `[CG-C-3a]` `@inline` is binding: the function is emitted with `__forceinline` or
  `__attribute__((always_inline))`. `@noinline`, `@cold` and `@hot` are passed to the C compiler as
  hints.
* `[CG-C-4]` *(changed in 0.9.9)* **Aliasing facts.** For each loop, the base pointer of every view whose base and length
  are invariant in the loop is hoisted into a local declared `T *restrict` exactly when `[SIMD-3]`
  (including its condition for views reached through a class handle) proves it disjoint from every other view the loop writes or reads; its length is hoisted into a local
  as well, and a view reassigned inside the loop is not hoisted.
* `[CG-C-5]` Every panic function is `_Noreturn` and cold, and each check branches forward to its panic
  call, so the check costs one compare and one predicted branch on the fast path.
* `[CG-C-6]` A loop in vectorisable form (`[SIMD-5]`) is preceded by the host compiler's vectorisation
  pragma.
* `[CG-C-7]` Locals keep their Ember names in the C (transliterated per `[MNG-3]`), so debuggers show
  them.
* `[CG-C-8]` Every emitted statement is preceded by a `#line` naming the Ember construct that
  produced it, and all C lowered from one Ember statement shares one `#line`, so stepping advances one
  Ember statement at a time. Desugared constructs (`for`, `?`, `with`, f-strings, operator calls) are
  attributed to the source syntax, not to their expansion.
* `[CG-C-9]` The build emits debugger visualisers (`.natvis`, GDB and LLDB scripts) that show `Option`,
  `Result`, enums, strings, collections and class handles as Ember values.
* `[CG-C-10]` Stack traces print Ember function paths and `file:line:col`; `ember demangle` converts C
  stacks.

## XVIII.2 Symbol names

* `[MNG-1]` *(changed in 0.9.9)* **Mangling is injective.** A symbol is `em_` followed by each path
  component (package, modules, item) written as its length in decimal and then its text, then for a
  generic instance `G` and the first 16 hex digits of the BLAKE3 hash of the canonical spelling of its
  arguments: module `lib/m_x.em`'s `f` is `em_3lib3m_x1f` and module `lib/m.em`'s `x_f` is
  `em_3lib1m3x_f`. Two distinct items therefore never share a name. A hash collision between two
  instances is reported as `E9040`, naming both, never as an internal error.
* `[MNG-2]` `@export("name")` sets the symbol exactly.
* `[MNG-3]` Non-ASCII identifier characters are transliterated as `_uXXXX_` before the length is taken.
* `[MNG-4]` Object structs, method tables and type information are named `em_obj_`, `em_vt_` and
  `em_ti_` followed by the mangled type.

## XVIII.3 Monomorphisation

* `[MONO-1]` Generic code is instantiated per distinct set of type arguments, from the program's roots
  (`main`, exports, tests, statics); instances are named deterministically and deduplicated at link
  time.
* `[MONO-2]` The compiler records, per generic, how many instances it produced and the time they took;
  `ember build --report=instantiations` prints it.
* `[MONO-3]` `[build] max_instantiations = N` sets a per-generic ceiling (unset by default); exceeding it
  is warning `W2220`, naming the generic and its newest instances.
* `[MONO-5]` A generic is **shareable** at a parameter `T` when `T` appears in its body only as the
  receiver of calls to methods of `T`'s bounds.
* `[MONO-6]` Only for a shareable generic whose instance count exceeds the ceiling of `[MONO-3]`, and
  none of whose calls lies in a loop the compiler judges hot, the compiler MAY emit one shared body
  taking a method table in place of `T`. With no ceiling set, nothing is shared.
* `[MONO-8]` A shared body computes exactly what the specialised ones would; it costs one indirect
  call per bound-method call and allocates nothing.
* `[MONO-9]` A shared body is its own symbol and is deduplicated like any instance; an exported or
  `extern` function is never shared.
* `[MONO-7]` `@always_specialize` forbids sharing for a generic; `@never_specialize` requires it, and is
  `E2223` on a generic that is not shareable, naming the use of `T` that prevents it. Neither changes
  what any program means, and `ember inspect` reports for each generic whether it was specialised or
  shared, and why.

## XVIII.4 Borrow checking

Borrow checking is specified exactly, so that every implementation accepts the same programs
(`[PHIL-13]`). It runs on each function's control-flow graph after type checking, with every
expression broken into single operations (reads, writes, moves, borrows, calls, drops) at points.

* `[BCK-1]` *(new in 0.9.9)* **Loans.** Each borrow expression at a point creates a **loan** of a place,
  shared or mutable. Each reference, view or borrowing closure value carries the set of loans it may
  have been derived from; copying, reborrowing, projecting, passing or returning a value carries its
  loans with it, and a call's result carries the loans of the arguments it may borrow from (`[LT-1]`).
* `[BCK-2]` *(new in 0.9.9)* **Live loans, location-sensitive.** A loan is **live** at a point when some
  value carrying it is used at a later point on some path from that point, with no intervening
  assignment of that value. A loan carried by a function's result is live on the paths that reach
  the `return`, and only on those paths.
* `[BCK-3]` *(new in 0.9.9)* **Conflicts.** At each point, an access to a place is checked against the
  loans live there whose places overlap it (a place overlaps its prefixes and extensions; distinct
  fields, distinct `SoA` columns and the halves of a split do not, `[BRW-4]`): a write, move or drop
  conflicts with any live loan; a read conflicts with a live mutable loan; a new mutable loan
  conflicts with any live loan; a new shared loan conflicts with a live mutable loan. A conflict is an
  error in the `E3020`–`E3029` range, classified per §XVII.6.1.
* `[BCK-4]` *(new in 0.9.9)* **Storage end.** A place whose storage ends (scope exit, `StorageDead`)
  while a loan of it is live is `E3060`/`E3061`.
* `[BCK-5]` *(new in 0.9.9)* **Two-phase borrows.** The mutable loan made for a method's `mut self`
  receiver or a `mut` argument is *reserved* while the call's other arguments are evaluated and
  *activated* when the call starts; during reservation it conflicts only with writes (`[BRW-3]`).
* `[BCK-6]` *(new in 0.9.9)* **Required acceptances.** Because liveness is location-sensitive, these are
  accepted: a borrow whose last use precedes a later mutation (`[BRW-2]`); a borrow returned on one
  branch while the other branch mutates the borrowed place, as in

  ```ember
  fn first_even(xs: MutSpan[int]) -> ref mut int:
      for i in 0..xs.len():
          if xs[i] % 2 == 0:
              return ref mut xs[i]
      xs[0] = 0
      return ref mut xs[0]
  ```

  and a loop that conditionally stores a borrow in a local declared outside it and mutates the
  source on iterations where it did not.
* `[BCK-7]` *(new in 0.9.9)* **Class handles.** An access through a class handle is not a loan of the
  handle's local; it is governed by the exclusivity rules of §VIII.3. A borrow of a field reached
  through a handle creates a loan of that path and additionally keeps the object alive (`[RC-5]`).

## XVIII.5 The runtime

* `[RT-1]` *(changed in 0.9.9)* The runtime `ember_rt` is C11 depending on libc and the OS only.
  Allocation uses the system `malloc`/`realloc`/`free` for alignments up to `alignof(max_align_t)` and
  an aligned allocator above it; `realloc` grows in place where the allocator can. A host may supply
  its own allocator (`[FFI-27]`).
* `[RT-2]` The runtime has no global constructors; the generated `main` calls `ember_rt_init`.
* `[RT-3]` `TypeInfo` holds size, alignment, flags, name, base, drop functions, method tables and, for
  `@reflect` types, field descriptors.
* `[RT-4]` A panic prints `panic at <file>:<line>:<col>: <message>` naming Ember entities (`[DIA-23]`),
  a backtrace outside `shipping`, calls the host's panic hook if set, and aborts.
* `[RT-6]` The runtime ABI version is `EMBER_RUNTIME_ABI`.
* `[RT-5]` Every runtime symbol, macro and header name derives from one constant,
  `EMBER_SYMBOL_PREFIX` (default `ember`); `ember_rt.h` is generated with the literal names so C
  embedders can read it.
* `[RT-7]` *(changed in 0.9.9)* A reference count never wraps: a retain that would overflow panics with
  `reference count overflow on <Class>`.
* `[RT-8]` Counts of `@sync` objects use a relaxed increment for retain and an acquire-release decrement
  for release (weak counts likewise); publication of an initialised object is a release and its
  acquisition an acquire.
* `[RT-10]` *(new in 0.9.9)* **Counting is inline.** The fast path of retain (one increment and an
  overflow test) and release (one decrement and a test for zero) is defined in `ember_rt.h` and
  inlined into the caller; only deinitialisation is out of line. Copying an existing strong handle
  of a `@sync` object is one atomic `fetch_add`, never a compare-exchange loop; `Weak.upgrade` is the
  one retain that may find the strong count at zero, and it is a compare-exchange that then fails,
  never raising a count from zero (`[WK-12]`) (ODR-060). The deinitialising and resurrection checks of `[OBJ-5]`
  run on the release-to-zero path only.
* `[RT-11]` *(new in 0.9.9)* Allocation statistics are kept per thread, or only in `debug`; no
  allocation updates a shared non-atomic counter.
* `[RT-12]` *(new in 0.9.9)* **Stack overflow faults.** The backend compiles with stack probes
  (`-fstack-clash-protection` where the C compiler has it; MSVC probes by default), so a frame larger
  than a page touches each page in order; the runtime gives every thread it creates a guard page;
  and overflow aborts with `stack overflow in <function>` in every profile. A stack write can never
  land beyond the guard page.

## XVIII.6 Correctness of the implementation

* `[IMP-11]` *(new in 0.9.9)* The reference implementation verifies its intermediate representation
  after every pass in its own debug builds; runs every `run-pass` test through every backend and the
  compile-time evaluator where applicable, requiring identical output; fuzzes the lexer, parser, type
  checker and borrow checker; and compiles the layout tests of `[FFI-5a]` with every supported C
  compiler.
