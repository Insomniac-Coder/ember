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
* `[CG-C-11]` *(new in 0.9.9)* *(changed in 0.9.15)* **Floating-point flags.** Every translation unit begins with
  `#pragma STDC FP_CONTRACT OFF` where the compiler implements it (Clang; GCC does not, and warns
  about it, so it has the flag alone, ODR-044) and is compiled with `-ffp-contract=off -fno-fast-math` (Clang, GCC)
  or `/fp:precise` with `#pragma fp_contract(off)` (MSVC), with SSE2 rather than x87 on 32-bit x86,
  and without flush-to-zero. A `@fastmath` or `@fp(allow_fma)` function is emitted in a separate
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
* `[CG-C-3a]` *(changed in 0.9.10)* `@inline` is binding: the function is emitted with `__forceinline`
  or `__attribute__((always_inline))`, except a reloadable function in a hot-reload build, which is never
  inlined (`[HR-9a]`): there its `@inline` has no effect, and builds without hot reload inline it.
  `@noinline`, `@cold` and `@hot` are passed to the C compiler as hints.
* `[CG-C-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Aliasing facts.** Every `ref mut` parameter, a
  `MutSpan` one included, is passed as a pointer declared `restrict`: while it is used, nothing else the
  function uses reaches its place, which the compiler proves at each call (`[BCK-8]`); in a copy
  compiled for one whole variable under two names (`[MONO-10]`) the two names are one pointer. `this`
  is never declared `restrict`, since a method's object may also be reached through a link while the
  method runs. For each loop, the base pointer of every view whose base and length are invariant in the
  loop is hoisted into a local declared `T *restrict` exactly when `[SIMD-3]` (including its condition
  for views reached through a link) proves it disjoint from every other view the loop writes or reads;
  its length is hoisted into a local as well, and a view reassigned inside the loop is not hoisted.
* `[CG-C-5]` Every panic function is `_Noreturn` and cold, and each check branches forward to its panic
  call, so the check costs one compare and one predicted branch on the fast path.
* `[CG-C-6]` *(changed in 0.9.10)* A loop in vectorisable form (`[SIMD-5]`) is preceded by the host
  compiler's vectorisation pragma: `#pragma clang loop vectorize(enable)` for Clang, `#pragma GCC
  ivdep` for GCC. MSVC, which has no pragma that asks for vectorisation (`loop(ivdep)` only asserts
  that no dependency exists), gets none.
* `[CG-C-7]` Locals keep their Ember names in the C (transliterated per `[MNG-3]`), so debuggers show
  them.
* `[CG-C-8]` Every emitted statement is preceded by a `#line` naming the Ember construct that
  produced it, and all C lowered from one Ember statement shares one `#line`, so stepping advances one
  Ember statement at a time. Desugared constructs (`for`, `?`, `with`, f-strings, operator calls) are
  attributed to the source syntax, not to their expansion.
* `[CG-C-9]` *(changed in 0.9.15)* The build emits debugger visualisers (`.natvis`, GDB and LLDB
  scripts) that show `Option`, `Result`, enums, strings, collections, class objects and their links
  (`owns`, `shares`, `weakref`) as Ember values.
* `[CG-C-10]` Stack traces print Ember function paths and `file:line:col`; `ember demangle` converts C
  stacks.

## XVIII.2 Symbol names

* `[MNG-1]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Mangling is injective.** A symbol is `em_`
  followed by each path component (package, modules, item) written as its length in decimal and then
  its text, then for a generic instance `G` and the first 16 hex digits of the BLAKE3 hash of the
  canonical spelling of its arguments (`[MNG-6]`): module `lib/m_x.em`'s `f` is `em_3lib3m_x1f` and
  module `lib/m.em`'s `x_f` is `em_3lib1m3x_f`. The root package's items have no package component
  (its `main` is `em_4main`); a dependency's begin with its package's name. A method's path is its
  type's path and its name, with the interface's path before the name for a method told apart by the
  interface it implements (`[TYP-24]`). A built-in type's path is under `std.core`
  (`std.core.StrView`), so a program's own `struct StrView`, `Never` or `CStrView` never shares one
  with it; a structural type (a tuple, fixed array, function, pointer or `dyn` type) that owns a method
  is `std.core.<kind>`, with its whole spelling as its arguments; `[CLO-3]`'s copy of a function for a
  function passed to it is an instance of that function, the passed function among its arguments; and
  `[MONO-10]`'s copy for one whole variable under two names is an instance too. Two distinct items
  therefore never share a name (`[MOD-1]`). A hash collision between two instances is reported as
  `E9040`, naming both, never as an internal error.
* `[MNG-2]` *(changed in 0.9.15)* `@unsafe(export("name"))` sets the symbol exactly (`[FFI-26]`).
* `[MNG-3]` *(changed in 0.9.10)* Non-ASCII identifier characters are transliterated as `_uXXXX_`
  before the length is taken, and so is an ASCII `_` that begins what would otherwise read as an
  escape (`_u`, four or more hex digits and `_`), as `_u005F_`. A component so transliterated has `u`
  before its length: `café` is `u10caf_u00E9_`, the ASCII name `caf_u00E9_` is
  `u16caf_u005F_u00E9_`, and `é_u00E9_` and `éé` differ. C member names are spelled the same way,
  without the length, so two fields never share one.
* `[MNG-4]` *(changed in 0.9.10)* Object structs, method tables and type information are named
  `em_obj_`, `em_vt_` and `em_ti_` followed by the type's mangled form without its own `em_`
  (`em_obj_6Animal`).
* `[MNG-6]` *(new in 0.9.10)* *(changed in 0.9.15)* **The canonical spelling** of a generic instance's arguments, which
  `[MNG-1]` hashes, is the same in every compiler, so separately compiled packages deduplicate
  instances at link time (`[MONO-1]`). It is `[a, b]` for each generic level that has arguments, in
  path order (the owner type's, then the interface's, then the method's own): a declared type by its
  path, never by a name the compiler gave an instance, and an instance with its arguments; a scalar by
  its keyword; `String`, `Array`, the spans and the generics the compiler makes (`Option`) under
  `std.core`; tuples, fixed arrays, references, pointers and function types as written, the words of a
  type with them (`owns T`, `shares mut T`, `weakref T`, `using A`, `thread_shared fn()`); a type
  parameter as `$`, its name and its slot (`$T0`); a function given for a callable parameter as its
  path after its type (`[fn(i64) -> i64 = add_one]`); and, for `[MONO-10]`'s copy, each pair of
  parameters made one, after the arguments (`[a = b]`).

## XVIII.3 Monomorphisation

* `[MONO-1]` *(changed in 0.9.10)* Generic code is instantiated per distinct set of type arguments, from
  the program's roots (`main`, exports, tests, statics); instances are named deterministically
  (`[MNG-1]`, `[MNG-6]`) and deduplicated at link time, across separately compiled packages too.
* `[MONO-2]` The compiler records, per generic, how many instances it produced and the time they took;
  `ember build --report=instantiations` prints it.
* `[MONO-3]` `[build] max_instantiations = N` sets a per-generic ceiling (unset by default) on the
  package's own generics, the standard library's not being the package's to budget (ODR-135);
  exceeding it is warning `W2220`, naming the generic and its newest instances.
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
* `[MONO-10]` *(new in 0.9.15)* **One whole variable under two names.** A call that the compiler proves,
  when compiling, names one whole variable twice — two `ref mut` arguments, or a `ref mut` argument
  and a module variable the callee uses by its own name (`add_both(ref mut x, ref mut x)`,
  `rename(ref mut LOG)`) — is compiled against its own copy of the callee in which the two names are
  one place, so the copy computes what its lines say for one variable (`add_both`, whose body is
  `a += 1`, `b += 1`, `println(a)`, prints 2). The copy is an instance of the callee (`[MNG-1]`,
  `[MNG-6]`), deduplicated like any instance and never shared (`[MONO-6]`); calls that do not name one
  variable twice use the ordinary body, where a `ref mut` parameter is the only way to its place
  (`[CG-C-4]`). It costs one more copy of the function's code per such pairing and nothing while the
  program runs; `ember inspect` lists each copy with the call that needs it.

## XVIII.4 Checking views and changes

Checking views and changes is specified exactly, so that every implementation accepts the same
programs (`[PHIL-13]`). It runs on each function's control-flow graph after type checking, with every
expression broken into single operations (reads, writes, hand-overs, views made, calls, ends) at
points, and with each call standing for what its callee's summary (`[BCK-8]`) says it does.

* `[BCK-1]` *(new in 0.9.9)* *(changed in 0.9.15)* **Views.** Each expression that makes a view at a
  point creates a **view** of a place: a name bound to a part, `ref` or `ref mut` before a place, a
  view a call gives (`[LT-1]`), a loop, pattern or opened name, a kind test's name, a `ref` capture, a
  span. A view is read-only, or changeable (`ref mut`, `sharedref mut`, a `MutSpan` held by a `ref
  mut` name); and it is a **position** (a list element named by its index, of a list reached through
  locals that own it, parameters, `this` and one-owner fields, found again at each use) or an
  **address** (every other view). Each value that may hold views carries the set of views it may
  hold; copying, passing, returning, storing or projecting a value carries its views with it, and a
  call's result, and a view a call puts in through a `ref mut` parameter, carry the views of the
  arguments they may point into (`[LT-1]`, `from`).
* `[BCK-2]` *(new in 0.9.9)* *(changed in 0.9.15)* **Live views, location-sensitive.** A view is
  **live** at a point when some value carrying it is used at a later point on some path from that
  point, with no intervening assignment of that value. A view carried by a function's result is live
  on the paths that reach the `return`, and only on those paths. A view whose last use is an argument
  of a call is not live across the call: inside it, the parameter makes the promise, and the call is
  checked by the callee's summary (`[BCK-8]`).
* `[BCK-3]` *(new in 0.9.9)* *(changed in 0.9.15)* **Conflicts.** At each point, a read or a change of
  a place, made directly or by a call through its callee's summary, is checked against the views live
  there whose places it may reach:
  - a change of what a live read-only view reads is `E3021`, at the change; a change that may reshape
    what an address points into ends the address instead, and its later use is `E3060`; a removal,
    insertion, reorder, clear or replacement of the list that holds a position ends the position
    (`E3060` at a later use), while adding to the list does not;
  - while a `ref mut` view is live, its place read through another name is `E3021` and changed through
    one `E3022`, a reshape through another name while a `ref mut` address is live included;
  - while a `for` loop walks a collection, or a generator that walks it will still be used, any
    change of the collection other than through the loop's own handle is `E3020`, at the change.

  Two places **may be one** when one is a prefix or an extension of the other (distinct fields,
  distinct `SoA` columns and the halves of a split are not, `[BRW-4]`); when they are elements of one
  list whose indexes are not proved different (different constants, `j` from `i + 1..`, under
  `if i != j:`); or when they lead to values of one type and at least one path goes through a link or
  a kept view, unless the compiler knows they differ (different types, a fresh object, under
  `if a is not b:`). Adding to a list moves only what it stores inline: a view of a class object,
  which lives in its own block, is not ended by a `push` or an `extend` of the list that owns it. A
  conflict is an error in the `E3020`–`E3031` range, classified per §XVII.6.1; where two errors fit one
  line, the one whose fix makes the program compile is shown (`[DIA-28]`).
* `[BCK-4]` *(new in 0.9.9)* *(changed in 0.9.15)* **Storage end.** A place whose storage ends (scope
  exit, `StorageDead`, its owner handed over, replaced or ended) while a view of it is live is `E3060`
  at the view's later use; a view, or a value holding views, kept or returned where it would outlive
  what it points into, a value made in a memory manager's instance included, is `E3063`.
* `[BCK-5]` *(new in 0.9.9)* *(changed in 0.9.15)* **A changeable argument takes effect when the call
  starts.** A `ref mut` argument, and the object of a method that may change it, is *reserved* while
  the call's other arguments are evaluated and *activated* when the call starts; during reservation it
  conflicts only with a change of its place (`[BRW-3]`), so `v.push(v.len())` is accepted. When the
  call starts, its arguments are checked against each other: two of them, at least one changeable (a
  function value counting as changing every place its functions change), name one place or reach into
  each other only where the compiler proves them one whole variable (`[MONO-10]`) (`E3022` where both
  may change it, `E3021` otherwise).
* `[BCK-6]` *(new in 0.9.9)* *(changed in 0.9.15)* **Required acceptances.** Because liveness is
  location-sensitive, these are accepted: a view whose last use precedes a later change (`[BRW-2]`);
  a list added to while a position of it is still used (`boss = ref mut enemies[0]`,
  `enemies.push(Enemy(hp=5))`, then `boss.hp = 1`); a view returned on one branch while the other
  branch changes the place it views, as in

  ```ember
  fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:
      for i in 0..xs.len():
          if xs[i] % 2 == 0:
              return ref mut xs[i]
      xs[0] = 0
      return ref mut xs[0]
  ```

  and a loop that conditionally stores a view in a local declared outside it and changes the place it
  views on rounds where it did not.
* `[BCK-7]` *(new in 0.9.9)* *(changed in 0.9.15)* **Links.** A place reached through a link
  (`shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut`) is in another object, and two
  paths to such places follow `[BCK-3]`'s rule for links. A view of an object reached through a link
  lives only while no line and no call in its life may end an object of its type (`E3060`), and across
  a call that may change an object of its type a read-only view of one does not live (`E3021`), so a
  walk across such a call keeps a `weakref` link and tests it at every step. Nothing is kept alive
  while the program runs for such a view, except a view of a `thread_shared` object opened from a
  `weakref` link, which holds one count for its life (`[RC-5]` point 2, `[RT-10]`).
* `[BCK-8]` *(new in 0.9.15)* **Summaries.** For each function the compiler works out a summary from
  its body: what it may change, read and end, each with the chain of calls that does it, which
  messages name; which elements of a list it touches (a list parameter it only adds to, `push` or
  `extend`, keeps a position of that list usable across the call); which of its view parameters,
  `this` included, it uses after a line that may end or move what they view; what it reaches by
  another name while a `ref mut` parameter is used; whether it sets the parent link of an object it is
  given before reading it; and which locks it may take. A call contributes its callee's summary; a
  call through a function value, or through an interface, `dyn` or virtual method, the union of the
  summaries of every function the program can put there, the libraries' included; starting a thread or
  a task, or handing a function to C to keep, the summary of that function; a foreign call those of
  the Ember functions C may run then (`[FFI-52]`); and ending an object, its `drop`'s. Summaries are
  the least fixed point over the whole program's call graph, so recursion is covered and the result
  depends on nothing but the program; across a package's edge each is recorded in the package's
  interface file (`[EFF-4]`), and a change to it is a change of the package's interface. Each call is
  checked with its callee's summary: an argument that points into what the callee may end or move
  before its last use of that parameter, or, for a `ref mut` parameter, into what the callee reaches
  by another name while the parameter is used, is `E3031`, unless it is one whole variable proved when
  compiling (`[MONO-10]`). Nothing is checked while the program runs.

## XVIII.5 The runtime

* `[RT-1]` *(changed in 0.9.9)* The runtime `ember_rt` is C11 depending on libc and the OS only.
  Allocation uses the system `malloc`/`realloc`/`free` for alignments up to `alignof(max_align_t)` and
  an aligned allocator above it; `realloc` grows in place where the allocator can. A host may supply
  its own allocator (`[FFI-27]`).
* `[RT-2]` The runtime has no global constructors; the generated `main` calls `ember_rt_init`.
* `[RT-3]` `TypeInfo` holds size, alignment, flags, name, base, drop functions, method tables and, for
  `@reflect` types, field descriptors.
* `[RT-4]` A panic prints `panic at <file>:<line>:<col>: <message>` naming Ember entities (`[DIA-23]`),
  a backtrace outside `release`, calls the host's panic hook if set, and aborts.
* `[RT-6]` The runtime ABI version is `EMBER_RUNTIME_ABI`.
* `[RT-5]` Every runtime symbol, macro and header name derives from one constant,
  `EMBER_SYMBOL_PREFIX` (default `ember`); `ember_rt.h` is generated with the literal names so C
  embedders can read it.
* `[RT-7]` *(changed in 0.9.9)* *(changed in 0.9.15)* **A count is checked before it rolls over.** The
  owner count of a `shares`, `shares mut` or `thread_shared` value, and the weak count of an object
  `weakref` links may reach, are 32-bit numbers in its header. Before each +1 of a one-thread count it
  is compared with its maximum, and at the maximum the program stops: "too many owners of a `Player`
  (4,294,967,295)", and for the weak count "too many `weakref` links to a `Player` (4,294,967,295)";
  one compare per added owner or link, no extra memory. A `thread_shared` value's home count
  (`[RC-4]`) is checked as a one-thread count is; its away count and its weak count are raised by one
  atomic add (`[RT-10]`), and the program stops when the value before the add is at or above
  2,147,483,647, half the range, so that no number of threads adding at once can carry it past its
  maximum: "too many owners of a `Log` (2,147,483,647)", and the same for its weak count. The two test
  programs of these stops are compiled with an 8-bit count in each object (the test header
  `#$ count_bits: 8`), stored and compared as the real one is, so 255 owners or links of a one-thread
  value reach the limit and the next stops the program ("too many owners of a `Player` (255)"; 127
  for a `thread_shared` value); every other program keeps the 32-bit count.
* `[RT-8]` *(changed in 0.9.15)* A `thread_shared` value's away count and its weak count (`[RC-4]`)
  are atomic: an owner or a link made is a relaxed increment, one ended an acquire-release decrement;
  putting a value in its making thread's queue is a release, and emptying the queue an acquire. Its home
  count is plain: only its making thread reads or writes it. A `shares` value's counts are plain adds,
  since no `shares` value reaches another thread.
  Publishing a new `thread_shared` value to another thread is a release, and taking it there an
  acquire.
* `[RT-10]` *(new in 0.9.9)* *(changed in 0.9.15)* **Counting is inline.** A count changes only where a
  line makes or ends a `shares`, `shares mut` or `thread_shared` owner or a `weakref` link, and where a
  view of a `thread_shared` object is opened from a `weakref` link; an object with one owner is never
  counted. The fast path of a +1 (the add and its limit test, `[RT-7]`) and of a -1 (the subtract and
  a test for zero) is defined in `ember_rt.h` and inlined into the caller; only ending the object is
  out of line. On a `thread_shared` value's making thread, until the value is handed over (`[RC-4]`
  point 4), making or ending an owner is a plain add or subtract of its home count, inlined as a
  one-thread count's is. On another thread, and after the hand-over, making an owner, or a `weakref`
  link to the value, is one atomic `fetch_add` followed by the limit test on the value it returns, and
  ending one is one atomic subtract, never a compare-exchange loop; a subtract that takes the away
  count below zero puts the value in its making thread's queue in the same atomic step (out of line, as
  are the hand-over and the queue's emptying at a check point). Opening a `weakref` link to a
  `thread_shared` object is the one +1 that may find the object ended: on its making thread it tests
  that the two counts add up to more than zero before adding to the home count; on another thread its
  atomic add finds the ended mark and is undone; neither ever raises an ended object's count
  (ODR-060). Opening a link to a one-thread object only tests its header. The subtract that
  reaches zero marks the object ended before its `drop` runs, so a `weakref` link opened during the
  `drop` finds it ended.
* `[RT-11]` *(new in 0.9.9)* Allocation statistics are kept per thread, or only in `debug`; no
  allocation updates a shared non-atomic counter.
* `[RT-12]` *(new in 0.9.9)* *(changed in 0.9.15)* **Stack overflow faults.** The backend compiles every
  target with stack probes: it passes the probe flag to every C compiler that takes one
  (`-fstack-clash-protection` for Clang and GCC, on every target; MSVC and clang-cl probe by default),
  and refuses a toolchain that cannot probe (`E9014`, as `E9011` refuses one that cannot turn
  contraction off); `ember build --emit c` writes, beside the C sources, the flags they must be
  compiled with, the probe flag among them (`[CLI-2]`). So a frame larger than a page touches each page
  in order; the runtime gives every thread it creates a guard page; and overflow aborts with
  `stack overflow in <function>` in every profile. A stack write can never land beyond the guard page.

## XVIII.6 Correctness of the implementation

* `[IMP-11]` *(new in 0.9.9)* *(changed in 0.9.15)* The reference implementation verifies its intermediate representation
  after every pass in its own debug builds; runs every `run-pass` test through every backend and the
  compile-time evaluator where applicable, requiring identical output; fuzzes the lexer, parser, type
  checker and the checking of views and changes; and compiles the layout tests of `[FFI-5a]` with every
  supported C compiler.

## XVIII.7 The LLVM backend

Restored from 0.9.8 by ODR-087.

* `[CG-LL-1]` *(new in 0.9.9)* *(changed in 0.9.10)* **Two backends over one intermediate representation.** The reference
  implementation has two backends over the same MIR: C11 (§XVIII.1), for bootstrap and portability,
  and LLVM, for optimisation control. Both are part of 1.0. The C backend came first; the LLVM
  backend is built beside it, and becomes the default once it passes the conformance and the
  performance suites; until it exists, `--backend llvm` and `backend = "llvm"` are refused as not yet
  implemented. The C backend stays supported afterwards: it is how Ember reaches every platform that
  has a C compiler.
* `[CG-LL-2]` *(new in 0.9.9)* The LLVM backend maps MIR to LLVM IR with the C backend's type mapping
  and meets every requirement of this Part that is not about C specifically: no undefined behaviour
  in its output, checked arithmetic through LLVM's overflow intrinsics, and the floating-point rules
  of `[TYP-9]` and `[DET-5]`. It implements the platform C ABI's classification of arguments and
  results itself (`[FFI-9]`), and lowers `std.simd`'s vector types to LLVM's (`[SIMD-1]`).
* `[CG-LL-3]` *(new in 0.9.9)* *(changed in 0.9.15)* It passes the facts the compiler proves to LLVM
  as `noalias` (every `ref mut` parameter, a `MutSpan` one included, and never `this`, as `[CG-C-4]`
  says for `restrict`), `readonly`, `dereferenceable(N)` and `nonnull` attributes, under the rule that
  governs the C backend's `restrict` (`[SIMD-3]`: derived, never assumed); emits DWARF or CodeView
  debug information with Ember's type names; and supports profile-guided optimisation through LLVM's
  instrumentation and ThinLTO (`[BLD-6]`).
