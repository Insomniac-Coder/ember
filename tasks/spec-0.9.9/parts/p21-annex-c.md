---

# Annex C — C++ Interoperation (the Native profile, optional)

Calling C++ is a separate, optional conformance claim (`[CONF-4]`). An implementation provides Part
XVI (C) first; nothing in Parts I–XVIII depends on this annex.

* `[FFI-3]` Ember never links against C++ mangled symbols. The importer generates `extern "C"`
  thunks, compiled by the project's own C++ compiler with the project's flags, and Ember calls the
  thunks.
* `[FFI-17]` `import cpp "Header.hpp" with (project="engine", overlay="…", instantiate=[…])` parses
  the header with libclang in the configuration of the named `[cpp.<project>]` section and generates
  one thunk per imported function, method, constructor and destructor. Overlays for C++ use `overlay
  cpp "Header.hpp":` with the grammar of `[GRM-35]`.
* `[FFI-50]` *(new in 0.9.9)* The manifest section `[cpp.<project>]` names the C++ project:
  `compiler` (`msvc`, `clang-cl`, `clang`, `gcc`), `standard`, `defines`, `include_paths`, `flags`,
  and `cmake = { build_dir, target }` to read the flags from the CMake File API (`[BLD-FFI-2]`).
  `ember bind --emit-cpp` writes the thunk translation unit for inspection.
* `[FFI-17b]` Templates are available only as explicit instantiations listed in `instantiate=[…]`;
  an Ember generic cannot instantiate a C++ template (`E5055`).
* `[BLD-FFI-1b]` The MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` are inherited
  byte for byte from the C++ project, because they change the layout of standard-library types;
  translation units that disagree are `E9020`, and an undeterminable C++ runtime is `E9021`.
* `[BLD-FFI-4]` Ember's emitted C and the project's C++ are compiled by the same compiler, so the
  project's LTO setting inlines calls across the boundary like any other call.
* `[BLD-FFI-5]` `ember build --emit header` with `--cpp` also writes a C++ header whose declarations
  take `std::string_view` and `std::span` where the C header takes pointer-and-length pairs.
* `[BLD-FFI-5a]` *(changed in 0.9.15)* *(changed in 0.9.20)* With the C++ header, an exported
  function may take or return a `thread_shared` value of a class, the one owning Ember value that
  crosses (`[FFI-31b]`): the header presents it as a copyable handle class of the same name with the
  class's `pub` methods a `thread_shared` name may call in Ember (`[THR-1]`: those whose summary
  changes the object only through its `Atomic`, lock, channel and `guarded by` fields), the others
  left out with a comment naming the rule, each of them an export under `threads=any` (`[FFI-33]`)
  whose wrapper attaches the calling thread (`[FFI-22]`), whose body is checked as such an export's
  is, and whose parameters and result follow `[FFI-31b]`; each copy one more owner on the value's
  away count (`[RC-4]`, `[RT-8]`) and each end one fewer, its limit checked as an Ember owner's is
  (`[RT-7]`). No other Ember value has a C++ handle, so C++ code never counts a one-thread value,
  from one thread or from two.

## C.1 What imports, and how

* `[FFI-44]` *(changed in 0.9.9)* *Automatic* means no overlay is needed; *overlay* means a contract
  must be written; *native island* means it stays in C++ behind a hand-written boundary.

  | C++ construct | Import |
  |---|---|
  | `extern "C"` functions, POD structs | automatic (Part XVI) |
  | standard-layout, trivially copyable classes | automatic, by value (`[FFI-32]`) |
  | other classes | automatic, opaque behind a pointer; methods through thunks |
  | the standard-library types of `[FFI-17a]` | automatic |
  | explicit template instantiations | automatic (`[FFI-17b]`) |
  | other templates, metaprogramming, concepts, compiler extensions | native island |
  | function-like macros, `std::function` | overlay |
  | single inheritance from a C++ base | automatic, bounded (`[FFI-39]`) |
  | multiple or virtual inheritance | native island (`[FFI-48]`) |
  | exceptions | automatic, with a derived or declared policy (`[FFI-24]`) |
  | custom allocators | overlay or native island |
  | ownership the header does not state | unsafe until `adopt` or a contract (`[FFI-36]`) |

* `[FFI-17a]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.10)* *(changed in 0.9.15)* *(changed in 0.9.20)*

  | C++ | Ember |
  |---|---|
  | `std::span<T>`, `std::span<const T>` | `MutSpan[T]`, `Span[T]`, zero-copy; a result needs a lifetime word (`[FFI-11]`) |
  | `std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]` |
  | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`) |
  | `std::string` | `CppString`; `.to_str() -> Result[StrView, Utf8Error]`; `String.from(c)` copies |
  | `std::vector<T>` | `CppVector[T]` with `.span()`, `.span_mut()`, `.to_array()`, `push_back`, `len` |
  | `std::unique_ptr<T>` | `owns T`, ended through the deleter; one whose deleter holds state is refused (`E5034`): an `owns T` holds one address |
  | `std::optional<T>` | `Option[T]` for a `T` that maps by value; the thunk converts at the call, in both directions (`has_value()` and the value: one branch and one copy of `T`), since the two layouts differ, as it does for a `std::variant`'s generated enum, a conversion `ember inspect` reports (`[FFI-17f]`) |
  | `std::shared_ptr<T>`, `std::weak_ptr<T>` | `CppShared[T]`, `CppWeak[T]` — never `shares`/`weakref` (`[WK-14]`, `E5065`) |
  | `std::variant<…>` | a generated enum, when every alternative maps and the overlay names each: `@ffi(variant=["A", "B"])` on a `type` entry naming the header's alias of the variant, the names in the alternatives' order |
  | `std::function<R(A…)>` | an opaque object Ember owns; with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function: for `[EFF-2]` and `[CLO-17]` that function is one the program can put in every place the value may reach (recorded across a package's edge as the function values a package stores, `[EFF-4]`), with every effect (`[EFF-3]`) and the callbacks, exports and overrides a foreign call may run (`[FFI-52]`); `@ffi(no_callbacks)` beside `std_function` on the type's entry says it runs none; the value is neither `Send` nor `ThreadShared` (`[FFI-17e]`), and its end runs the `std::function`'s destructor through a thunk (`[FFI-17f]`); never a parameter type (`E5030`) |

* `[FFI-17e]` *(changed in 0.9.15)* *(changed in 0.9.20)* The bridge types `CppVector[T]`,
  `CppString` and `CppShared[T]` are declared in `std.ffi`; they, and the function value a
  `std_function` import makes (`[FFI-17a]`), are not `Copy`, `Send` or `ThreadShared`; the first two
  end through their C++ destructors; and a span or string obtained from one is a view into it, so it
  cannot outlive it.
* `[FFI-17f]` *(changed in 0.9.20)* `ember inspect` reports each bridge operation that is a thunk
  call rather than an inlined one (`CppShared` clone and drop, `CppVector` `push_back`, `len` and
  drop, `CppString` drop, a `std_function` value's drop, and the conversion of a `std::optional` or
  `std::variant` at a call).
* `[FFI-32]` *(changed in 0.9.15)* A C++ class that is standard-layout and trivially copyable, all
  of whose members map, is imported as a `@layout(c) struct` with `@derive(Copy)` and the same field
  offsets; an opaque class cannot be constructed from Ember except through its imported constructors
  (`E5032`), and a parameter that maps to nothing is `E5031`. A function returning an opaque class
  by value gives an `owns T`, as a constructor does: the result is moved to an object of its own,
  ended by the class's destructor; an abstract class, or one without a public destructor, is refused
  (ODR-166).
* `[FFI-11f]` A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact:
  the attribute is not enforced by C++, so it never counts as checked.
* `[FFI-48]` *(changed in 0.9.10)* **Unsupported constructs** (`E5034`, naming the row):

  | Construct | Why not | Instead |
  |---|---|---|
  | multiple inheritance, virtual bases | the pointer adjustment is ABI-private and differs between MSVC and Itanium | a single-inheritance facade in C++ |
  | C++20 modules | no stable AST for a compiled module interface | parse the headers |
  | C++ coroutines | promise and frame layout are implementation-defined | a callback or completion handle |
  | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  | casts across a hierarchy the importer did not model | no Ember equivalent of `dynamic_cast` there | cast in C++ and export the result |
  | non-type template parameters of class type | mangling differs between compilers | instantiate in C++ and export a typedef |
  | allocator-parameterised containers | the allocator is part of the type and opaque | expose a span, or keep it native |

## C.2 Exceptions

* `[FFI-24]` *(changed in 0.9.9)* **Every imported C++ function has an exception policy, with a
  default.** The policy is derived: a function whose declaration is non-throwing (`noexcept`, a
  `noexcept(expr)` that evaluates true, a destructor, a defaulted special member) is called directly
  and returns `T`. Every other function returns `Result[T, CppError]`: its thunk catches every
  exception and returns it as a `CppError` holding the type name and `what()`.
* `[FFI-24b]` An overlay may assert `@ffi(noexcept)` for a function the header leaves throwing; its
  thunk still catches, and a throw then panics naming the declaration — never an unexplained
  `std::terminate`.
* `[FFI-24c]` *(changed in 0.9.10)* `ember inspect --cost`, which reads the built program, reports each
  C++ call's mode (`noexcept (derived)`, `noexcept (asserted)`, `catching`) and lists the catching
  calls on `@noalloc` or main-thread paths; `ember bind --report` reads a header and lists none.
* `[FFI-24d]` A header that adds or removes `noexcept` changes the Ember signature; the diagnostic in
  dependent code names the header, the declaration and the change as the cause, and `ember bind
  --report` lists it under API changes.
* `[FFI-43]` An overlay that marks a function both `noexcept` and throwing, or two composed overlays
  that disagree, is `E5062`; a function whose policy cannot be derived and is not declared is `E5061`.
* `[FFI-39e]` A C++ exception never crosses into Ember code, and an Ember panic never crosses into C++:
  both abort at the boundary if they would.

## C.3 Classes and inheritance

* `[FFI-36]` *(changed in 0.9.15)* A C++ function returning a raw pointer with no ownership contract
  yields `*mut T`; `ffi.adopt[T](p)` (`unsafe`) turns it into an `owns T` ended through the
  contract's destructor.
* `[FFI-39]` *(changed in 0.9.10)* *(changed in 0.9.15)* *(changed in 0.9.20)* An Ember class may
  derive from a C++ class with no overlay entry (`[FFI-44]`), which makes the base open and sized;
  the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly
  the virtual methods it overrides (`override fn`), read from the class. Its constructor is an
  ordinary `fn init(…)` calling `super.init(…)` exactly once. An `override fn` matching no virtual
  method of the base, by name and signature, is `E5056`. An object of such a class is one block: the
  header (`[OBJ-1]`), then the C++ base subobject (the trampoline's `this`, at a fixed offset every
  upcast adds), then the Ember fields. The trampoline subclass declares a virtual destructor that
  first marks the object ended through `ember_rt` (`[WK-3]`), then runs the Ember `drop`, ends the
  Ember fields and runs the base destructor (`[FFI-17c]`); and an `operator delete` that returns the
  block to `ember_rt`, which frees it only when no `weakref` link remains (`[CLS-6]`), subtracting
  the base subobject's fixed offset to reach the header; so C++'s `delete` frees nothing itself and
  does what the end of the last Ember owner would. An upcast, a hand-over to C++ and `into_raw()` on
  such an object give the base subobject's address, the header's plus that offset (`[FFI-54]`).
* `[FFI-39a]` The trampoline calls the overrides through their permanent thunks, so overrides
  survive hot reload (`[HR-6]`).
* `[FFI-39b]` *(changed in 0.9.10)* `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and
  no declared `init` is `E5058`.
* `[FFI-39c]` *(changed in 0.9.10)* *(changed in 0.9.15)* *(changed in 0.9.20)* A C++ parameter that
  keeps the pointer it is given (contracted `owns` in the overlay) takes the object over: its class
  is declared `@ffi(trampoline, owner="foreign")` (`[FFI-17d]`), the argument is handed over
  (`panel.add_child(owns b)`: `b` ends, upcast to the C++ base), and C++'s `delete` ends the object
  later. For a C++ function whose declaration cannot say so (a raw pointer it keeps), `b.into_raw()`
  inside `unsafe:` gives C++ the object's address (`[FFI-54]`). An object of an `owner="ember"`
  class given to a parameter that keeps it is `E5060`, whose message gives both lines (declare the
  class `owner="foreign"` and hand the object over, or give `into_raw()` inside `unsafe:`); a
  parameter borrowed for the call alone takes the object, or `this`, as any plain parameter, upcast
  to the C++ base (ODR-156).
* `[FFI-17c]` Destruction of an Ember class derived from a C++ class is derived-first, as `[CLS-6]`
  says for every class: the Ember `drop`, then the Ember fields, and only then the C++ base
  destructor.
* `[FFI-17d]` *(changed in 0.9.15)* *(changed in 0.9.20)* `@ffi(trampoline, owner="ember")`, the
  default, makes Ember's owner own the object, which ends when its last Ember owner does;
  `owner="foreign"` makes a foreign `delete` end it, once it has been handed to C++ (`[FFI-39c]`);
  until then the object is Ember's and ends when its last Ember owner does, as an `owner="ember"`
  one. `@ffi(trampoline, owner=…, threads=creator|any)` says which threads C++ may call the class's
  overrides from and end its objects on; the default is `creator`: an object of the class is bound
  to the thread that made it (`[THR-8]`: not `Send`, `E7008` if Ember hands it to another thread),
  whose id the trampoline subobject holds, one word; every override thunk and the trampoline's
  destructor attach the calling thread first (`[FFI-22]`), compare it with that id, in every
  profile, and stop the program otherwise ("C++ called `on_click` of a `Button` made on another
  thread (line 16); its class says `threads=creator`"). `threads=any` requires the class to be
  `Send` and `ThreadShared` (`E7015` at the class, naming the field: "`Button.log` is a `shares mut`
  owner, which stays on one thread, and C++ may call `Button`'s overrides from any thread
  (`threads=any`); make the log `thread_shared`, with `lines` `guarded by` a lock"), and each
  override is then checked as an export under `threads=any` is (`[FFI-33]`, `[THR-20]`). A
  trampoline base without a virtual destructor is `E5059`.
* `[FFI-39d]` *(changed in 0.9.9)* *(changed in 0.9.15)* Re-entrant calls from C++ into an Ember
  object are expected, and are checked when compiling: a call into C++ not marked
  `@ffi(no_callbacks)` counts as possibly running every override of an Ember class derived from a
  C++ class whose objects C++ may hold (`[FFI-52]`), and makes every change those overrides make. So
  a walk, a read-only name or a `ref mut` name the caller holds across the call is refused where an
  override may change what it views (`E3020`, `E3021`, `E3022`, `E3060`): "`cpp.dispatch_all()`
  (line 14) may run `on_event`, an override of `Button` whose objects C++ may hold (line 6), which
  pushes to a `Button`'s `codes`, and this loop walks `b.codes` (line 13); call `cpp.dispatch_all()`
  after the loop: move line 14 below the loop". Nothing is checked while the program runs.
* `[FFI-40]` *(changed in 0.9.15)* *(changed in 0.9.20)* A method imported from C++ is a method of
  the imported class: its object is `this`, and it lists no receiver. C++'s body is not read as
  Ember's is, so whether a method may change its object is taken from its declaration: a `const`
  method only reads its object when `[FFI-40a]` finds so, and every other method may change it, so
  it is not called through a name that only reads the object (`E3023`) and a call of it is a change
  of the object for the loop and view rules. `&&`-qualified members are skipped (`W5033`). A `T&`
  parameter imports as `ref mut x: T`, `const T&` as a plain parameter `x: T`, and `T&&` as `owns x:
  T` where the type is movable; a `T&` or `const T&` result is a pointer result (`[FFI-2a]`): its
  lifetime is never derived, and the overlay's entry gives it a lifetime word, `from(this)`,
  `from(p)` or `from(static)` (`[FFI-11]`), under which it is a `ref mut T` or `ref T` view into
  what the word names (`[LT-1a]`); without one the method stays unsafe to call, its result a `*mut
  T` or `*T`, listed by `ember bind --report` among the result-position pointers with no lifetime
  word (`[FFI-35a]`), and a call of it outside `unsafe:` is `E3100` ("`child.parent()` returns a C++
  reference whose target no contract names; put line 7 in an `unsafe:` block with a `# SAFETY:`
  note, or give `parent` a lifetime word in the overlay: `fn parent() -> borrowed one from(this)`").
* `[FFI-40a]` *(changed in 0.9.15)* `const` is not an aliasing guarantee in C++: a `const` method of
  a class with a `mutable` member, its own or a base's, may change its object. The importer reads
  the method's body, which the header has: it imports the method as only reading its object only
  when the body changes no field — it assigns none (`=`, a compound assignment, `++`, `--`), takes
  no field's address, binds no non-`const` reference or pointer to one, calls no non-`const` method
  of one, hands neither the object nor a field to a function by non-`const` reference or pointer,
  and calls no method of the object that may change it, nor anything it cannot follow — and as
  changing it otherwise, or when the header does not have the body (ODR-160). An overlay's `class
  C:` section holds the entry of each member it states (`[GRM-35]`, verdict 131): `@ffi(const)`
  above a method's entry (`fn lookups()`) asserts that the method only reads its object, an asserted
  fact, which `ember tcb` lists; an entry without it imports the method as one that changes its
  object, whatever the importer reads from the body (verdict 132): an entry says what the method
  does, and leaves nothing to a reading the reader of the overlay cannot see. An entry whose name or
  parameters differ from the import's member is `E5010` (`[FFI-12]`).
* `[FFI-41]` *(changed in 0.9.15)* Static member functions import as functions of the type with no
  object, as a `static fn` is (`[FN-4]`). A static data member, and a variable of a namespace,
  imports as a foreign static (`[FFI-10]`), read and written as a global (`cpp.engine.Grid.made +=
  1`), in `unsafe` unless it is `const` and holds no pointer, each use going through a thunk that
  gives its address, since its symbol is mangled or, for an `inline` one, emitted only where C++
  uses it (`[FFI-3]`, ODR-161). One holding a pointer imports like any other: it is read and written
  in `unsafe`, so `[STA-1]` does not ask it to be `ThreadShared` (ODR-163). Nested types sit under
  their outer type's name; namespaces become module paths, an inline one adding nothing to a name,
  as in C++ (`nlohmann::json_abi_v3_11_3::json` is `cpp.nlohmann.json`, ODR-165); entities in
  anonymous namespaces are skipped (`W5034`).
* `[FFI-42]` *(changed in 0.9.10)* Overloaded operators map to Ember operator interfaces where one
  exists (`operator[]` to `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to
  read-through: such a class imports `get()`, and `get_mut()` where the operator is not `const`,
  through which `->` is reached); C++ iterators with `begin`/`end` become `Iterable`.
* `[FFI-42a]` *(changed in 0.9.15)* A `for` loop over an imported C++ container walks it, so nothing
  changes the container while the loop walks it: a call of any method imported as changing its
  object (`[FFI-40]`), the `const` ones `[FFI-40a]` imports so included, is refused inside the loop
  (`E3020`), since it could invalidate C++'s iterators.

## C.4 Trust and evidence

* `[TCB-1]` *(changed in 0.9.9)* Every fact an overlay states about foreign code carries a grade:
  **asserted** (someone wrote it; the default), **instrumented** (a test run observed no
  counterexample on the paths exercised), **checked** (verified against the header) or **proven**.
  The grade is written as an attribute on the overlay item, `@grade(instrumented)`. `ember tcb
  [--module m]` prints every assumption the program's safety rests on, grouped by module. A claimed
  grade whose evidence is missing or stale is `W5050`, and `E5050` under `ember build
  --require-evidence` or `ember tcb --require`. Evidence records live under `[ffi] evidence` (default
  `.ember/ffi-evidence`).
* `[TCB-2]` The report separates what the language guarantees from what external components supply,
  and writing a grade never raises it: no fact above asserted comes from declaration alone.
* `[TCB-3]` Every assumption a guarantee relies on appears in the report with its source and grade.
* `[TCB-4]` *(changed in 0.9.15)* Entries are categorised — language, compiler, runtime, standard
  library, unsafe, C FFI, C++ bridge, external library, driver, hardware — and the compiler is in the
  list, with its version and known-defect list. The unsafe section lists each `unsafe:` block,
  `unsafe fn`, `unsafe import c` (`[FFI-51]`) and `@unsafe(export(…))` or `@unsafe(export_table(…))`
  (`[FFI-26]`) with its note; the C FFI section lists each `safe fn`, `@ffi(no_callbacks)`
  (`[FFI-52]`), `@ffi(safe)` (`[FFI-56]`) and thread contract, and the C++ bridge section each
  `@ffi(const)` (`[FFI-40a]`).
* `[TCB-5]` An instrumented fact is valid only for the foreign library, header, overlay, toolchain and
  test binary it was measured against; its evidence record stores their identities and hashes.
* `[TCB-6]` A change to any identity input makes the record stale; a change to an environment input
  (machine, timing) is reported but does not.
* `[FFI-37]` `ember test --instrument-ffi` runs the tests with the allocation, lock, wait and I/O entry
  points intercepted, to grade declared effect facts as `instrumented`.
* `[FFI-37a]` The intercepted set is published and recorded in the evidence; an allocation through a
  mechanism outside it (a custom pool, a driver) is reported as structurally unobservable, never as
  evidence.
* `[FFI-37b]` The evidence records how often each foreign function was called and from how many call
  sites; a fact never exercised is reported `unexercised` (`W5054`) and does not count.
* `[FFI-37c]` `instrumented` means "no counterexample was observed on the paths exercised", is shown as
  partial when not every call site ran, and is never shown as "holds"; a contradicted fact is `E5053`.
* `[FFI-37d]` *(changed in 0.9.15)* Only effect facts (`Alloc`, `Block`, `Lock`, `Io`) can be
  `instrumented`; ownership, nullability, lifetime, aliasing, exception, thread and callback
  (`no_callbacks`) facts stay asserted unless checked or proven.
* `[CXX-1]` *(changed in 0.9.9)* The conformance suite for this annex includes a corpus of real headers
  and at least one third-party header-only library, each with its expected outcome recorded (by value,
  opaque, behind an overlay, or refused with a named diagnostic) and its generated thunks committed and
  diffed; it runs under MSVC and Clang, with debug and release runtimes and RTTI on and off, and two
  configurations that disagree must fail with `E9020`, never bind differently.
