---

# Annex C — C++ Interoperation (the Native profile, optional)

Calling C++ is a separate, optional conformance claim (`[CONF-4]`). An implementation provides Part
XVI (C) first; nothing in Parts I–XVIII depends on this annex.

* `[FFI-3]` Ember never links against C++ mangled symbols. The importer generates `extern "C"` thunks,
  compiled by the project's own C++ compiler with the project's flags, and Ember calls the thunks.
* `[FFI-17]` `import cpp "Header.hpp" with (project="engine", overlay="…", instantiate=[…])` parses the
  header with libclang in the configuration of the named `[cpp.<project>]` section and generates one
  thunk per imported function, method, constructor and destructor. Overlays for C++ use
  `overlay cpp "Header.hpp":` with the grammar of `[GRM-35]`.
* `[FFI-50]` *(new in 0.9.9)* The manifest section `[cpp.<project>]` names the C++ project: `compiler`
  (`msvc`, `clang-cl`, `clang`, `gcc`), `standard`, `defines`, `include_paths`, `flags`, and `cmake =
  { build_dir, target }` to read the flags from the CMake File API (`[BLD-FFI-2]`). `ember bind
  --emit-cpp` writes the thunk translation unit for inspection.
* `[FFI-17b]` Templates are available only as explicit instantiations listed in `instantiate=[…]`; an
  Ember generic cannot instantiate a C++ template (`E5055`).
* `[BLD-FFI-1b]` The MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` are inherited
  byte for byte from the C++ project, because they change the layout of standard-library types;
  translation units that disagree are `E9020`, and an undeterminable C++ runtime is `E9021`.
* `[BLD-FFI-4]` Ember's emitted C and the project's C++ are compiled by the same compiler, so the
  project's LTO setting inlines calls across the boundary like any other call.
* `[BLD-FFI-5]` `ember build --emit header` with `--cpp` also writes a C++ header whose declarations
  take `std::string_view` and `std::span` where the C header takes pointer-and-length pairs.
* `[BLD-FFI-5a]` The C++ header gives a copyable smart handle only to `@sync` classes, whose counts are
  atomic; any other class gets a handle type that is neither copyable nor movable across threads, so C++
  code cannot count it from two threads.

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

* `[FFI-17a]` *(changed in 0.9.9)*

  | C++ | Ember |
  |---|---|
  | `std::span<T>`, `std::span<const T>` | `MutSpan[T]`, `Span[T]`, zero-copy; a result needs a lifetime word (`[FFI-11]`) |
  | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `str` (`[TXT-2]`) |
  | `std::string` | `CppString`; `.to_str() -> Result[str, Utf8Error]`; `String.from(c)` copies |
  | `std::vector<T>` | `CppVector[T]` with `.span()`, `.span_mut()`, `.to_array()`, `push_back`, `len` |
  | `std::unique_ptr<T>` | `ForeignBox[T]`, dropped through the deleter |
  | `std::optional<T>` | `Option[T]` for a `T` that maps by value |
  | `std::shared_ptr<T>`, `std::weak_ptr<T>` | `CppShared[T]`, `CppWeak[T]` — never `Shared`/`Weak` (`[WK-14]`, `E5065`) |
  | `std::variant<…>` | a generated enum, when every alternative maps and the overlay names each |
  | `std::function` | an opaque owned object, or `@ffi(std_function, signature=…)`; never a parameter type (`E5030`) |

* `[FFI-17e]` The bridge types `CppVector[T]`, `CppString` and `CppShared[T]` are declared in `std.ffi`;
  they are not `Copy`, `Send` or `Sync`; the first two drop through their C++ destructors; and a span
  or string obtained from one borrows it, so it cannot outlive it.
* `[FFI-17f]` `ember inspect` reports each bridge operation that is a thunk call rather than an inlined
  one (`CppShared` clone and drop, `CppVector` `push_back`, `len` and drop, `CppString` drop).
* `[FFI-32]` A C++ class that is standard-layout and trivially copyable, all of whose members map, is
  imported as a `@layout(c) struct` with `@derive(Copy)` and the same field offsets; an opaque class
  cannot be constructed from Ember except through its imported constructors (`E5032`), and a parameter
  that maps to nothing is `E5031`.
* `[FFI-11f]` A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact:
  the attribute is not enforced by C++, so it never counts as checked.
* `[FFI-48]` **Unsupported constructs** (`E5034`, naming the row):

  | Construct | Why not | Instead |
  |---|---|---|
  | multiple inheritance, virtual bases | the pointer adjustment is ABI-private and differs between MSVC and Itanium | a single-inheritance facade in C++ |
  | overriding a virtual not named in `virtuals=[…]` | the trampoline has no slot for it | name it (`E5056`) |
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
* `[FFI-24c]` `ember inspect` reports each C++ call's mode (`noexcept (derived)`, `noexcept
  (asserted)`, `catching`), and `ember bind --report` lists catching calls on `@noalloc` or main-thread
  paths.
* `[FFI-24d]` A header that adds or removes `noexcept` changes the Ember signature; the diagnostic in
  dependent code names the header, the declaration and the change as the cause, and `ember bind
  --report` lists it under API changes.
* `[FFI-43]` An overlay that marks a function both `noexcept` and throwing, or two composed overlays
  that disagree, is `E5062`; a function whose policy cannot be derived and is not declared is `E5061`.
* `[FFI-39e]` A C++ exception never crosses into Ember code, and an Ember panic never crosses into C++:
  both abort at the boundary if they would.

## C.3 Classes and inheritance

* `[FFI-36]` A C++ function returning a raw pointer with no ownership contract yields `*mut T`;
  `ffi.adopt[T](p)` (`unsafe`) turns it into a `ForeignBox[T]` using the contract's destructor.
* `[FFI-39]` An Ember class may derive from a C++ class the overlay declares with `@ffi(trampoline,
  virtuals=[…])`, which makes the base open and sized; the importer generates a C++ subclass whose named
  virtuals call the Ember overrides (naming a method that is not virtual is `E5057`).
  Its constructor is an ordinary `fn init(self, …)` calling `super.init(…)` exactly once.
* `[FFI-39a]` The trampoline calls the overrides through their permanent thunks, so overrides survive
  hot reload (`[HR-6]`).
* `[FFI-39b]` `super.init(…)` selects the C++ base constructor by arity and argument types and runs it
  before any Ember field is initialised; a base with no default constructor and no declared `init` is
  `E5058`.
* `[FFI-39c]` Passing `self` to a C++ API upcasts it and creates a `Retained` token (`[FFI-23]`) that
  keeps the object alive while C++ holds the pointer; an upcast where the token cannot be kept is
  `E5060`.
* `[FFI-17c]` Destruction of an Ember class derived from a C++ class is derived-first, as `[CLS-6]`
  says for every class: the Ember `drop`, then the Ember fields, and only then the C++ base destructor.
* `[FFI-17d]` `@ffi(trampoline, owner="ember")`, the default, makes the Ember count own the object;
  `owner="foreign"` makes a foreign `delete` destroy it. A trampoline base without a virtual destructor
  is `E5059`.
* `[FFI-39d]` Re-entrant calls from C++ into the same Ember object are expected and never panic on
  exclusivity: the trampoline takes no long-term access.
* `[FFI-40]` `const` methods import as `self`, non-`const` methods as `mut self`; `&&`-qualified members
  are skipped (`W5033`). A `T&` parameter imports as `mut T`, `const T&` as a borrowed `T`, and `T&&` as
  `owned T` where the type is movable; a `T&` result is a `ref` borrowing the receiver (`[LT-1]`).
* `[FFI-40a]` `const` is not an aliasing guarantee in C++: a `const` method that mutates `mutable` state
  or invalidates iterators must be declared `@ffi(invalidates)`, which gives it `mut self`, and the
  importer assumes `@ffi(invalidates)` for every `const` method of a type with a `mutable` member unless
  the overlay says otherwise (an asserted fact).
* `[FFI-41]` Static member functions import as associated functions and static data members as
  `extern static`; nested types sit under their outer type's name; namespaces become module paths;
  entities in anonymous namespaces are skipped (`W5034`).
* `[FFI-42]` Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to
  `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to read-through); C++ iterators with
  `begin`/`end` become `Iterable`.
* `[FFI-42a]` Iterating an imported C++ container borrows it mutably for the loop, so any method that
  could invalidate its iterators — every `mut self` method, including `@ffi(invalidates)` ones — is
  rejected inside the loop by the ordinary borrow rules.

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
* `[TCB-4]` Entries are categorised — language, compiler, runtime, standard library, unsafe, C FFI,
  C++ bridge, external library, driver, hardware — and the compiler is in the list, with its version
  and known-defect list. The unsafe section lists each `unsafe` block and function with its note.
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
* `[FFI-37d]` Only effect facts (`Alloc`, `Block`, `Lock`, `Io`) can be `instrumented`; ownership,
  nullability, lifetime, aliasing and exception facts stay asserted unless checked or proven.
* `[CXX-1]` *(changed in 0.9.9)* The conformance suite for this annex includes a corpus of real headers
  and at least one third-party header-only library, each with its expected outcome recorded (by value,
  opaque, behind an overlay, or refused with a named diagnostic) and its generated thunks committed and
  diffed; it runs under MSVC and Clang, with debug and release runtimes and RTTI on and off, and two
  configurations that disagree must fail with `E9020`, never bind differently.
