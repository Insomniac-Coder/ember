---

# Part XVI — Calling C

Being as fast as C includes using C's libraries with no glue. Ember calls a C function with one
direct call, lays out `@layout(c)` types exactly as the C compiler does, and reads C headers
directly. What a header cannot say — who owns a pointer, how many elements it points to, whether it
may be null — is stated once, in a **contract**, after which calls are safe. C++ interoperation is
Annex C.

```ember
from std.ffi import c_int, c_double

unsafe extern "C":
    @ffi(effects=[])
    fn abs(x: c_int) -> c_int
    @ffi(effects=[])
    fn cos(x: c_double) -> c_double

fn main():
    println(abs(-5), cos(0.0))
```

## XVI.1 Principles

* `[FFI-1]` Every foreign declaration has a **contract**: for each pointer-typed parameter and result,
  its ownership, mutability, nullability, count and lifetime (§XVI.4), and for the function its
  effects and thread rule. Facts derivable from the C declaration are derived (`[FFI-2a]`); the rest
  are `unknown` until a contract supplies them.
* `[FFI-2]` A call to a foreign function whose contract contains an `unknown` fact requires `unsafe`.
  Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the call safe.
  The diagnostic, `E5002`, names each missing fact and the overlay line that supplies it
  (`[DIA-18]`).
* `[FFI-2a]` Derived facts need no `unsafe`: `const T*` is a read-only borrow, `T*` a mutable one; the
  layouts of structs and unions, enum underlying types, packing and alignment, and the calling
  convention. In **result** position a pointer's lifetime is never derived.
* `[FFI-4]` No Ember panic crosses into foreign code (`[FFI-25]`).
* `[FFI-5]` Layouts are verified, not assumed: the importer records `sizeof`, `alignof` and field
  offsets from Clang for the project's target and flags, and the compiler checks its own layout
  against them (`E5001`, naming the type, field and both numbers).
* `[FFI-5a]` The generated shim (`[FFI-29]`) contains a `_Static_assert` for the size and alignment of
  every imported aggregate Ember constructs, passes, returns or embeds, and for the offset of every
  field Ember accesses, so the project's own C compiler re-verifies the layout on every build.

## XVI.2 Declaring C functions by hand

* `[FFI-10]` *(changed in 0.9.9)* An `unsafe extern "C":` block declares foreign functions, statics
  and opaque types; `unsafe` records that the programmer asserts the signatures. The declared names
  are items of the enclosing module. A function in the block is **safe to call** when every parameter
  and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete (given
  with `@ffi(…)` on the declaration); otherwise each call is in `unsafe`. Its effects are those its
  `@ffi(effects=[…])` states plus `FFI`, and every effect when none are stated (`[EFF-3]`).
* `[FFI-9]` `extern "C"` is the platform C ABI (SysV AMD64, Windows x64, AAPCS64). `extern "system"`
  is `stdcall` on 32-bit Windows and C elsewhere; `extern "vectorcall"` and `extern "fastcall"` exist
  on x86. Struct arguments and results follow the ABI exactly. An ABI-direct call is one `call`
  instruction and nothing else.
* `[FFI-49]` *(new in 0.9.9)* `@ffi(link_name="sym")` binds a declaration to a differently named symbol.
  Libraries are linked by `[link] libs = ["m", "vulkan-1"]` in the manifest or `link = [...]` on an
  `import c` (`[BLD-FFI-3]`).

## XVI.3 Importing C headers

```ember
import c "vulkan/vulkan.h" with (
    include_paths = ["vendor/Vulkan-Headers/include"],
    defines = ["VK_NO_PROTOTYPES"],
    overlay = "overlays/vulkan.em",
) as vk
```

* `[FFI-6]` `import c "header" with (…) [as name]` parses the header with libclang under the project's
  target, language standard, defines and include paths, and exposes its functions, structs, unions,
  enums, typedefs, globals and constant macros as a module (`c`, or `name`). Options: `include_paths`,
  `defines`, `link`, `overlay`, `implementation` (`[FFI-29b]`). An unknown option is `E0104`.
* `[FFI-6a]` An object-like macro is imported when, after expansion, it is a constant expression of
  integer, floating, string, null-pointer or pointer-cast type as Clang evaluates it; its type is the
  C type of that value. Any other macro is skipped with `W5001`, giving Clang's reason.
* `[FFI-6b]` A function-like macro is exposed only when an overlay declares its signature
  (`@ffi(macro_fn)`); the importer emits a C wrapper function for it into the shim. Ember gains no
  macro facility. When every argument is a constant the importer may fold the call, so the result can
  initialise a `const`.
* `[FFI-7]` A header is re-parsed only when its content, its overlay, or a setting that can change a
  binding's meaning or ABI changes (defines, target, language standard, packing, the C runtime); the
  result is cached as a `.embind` file (`[FFI-14]`).
* `[FFI-20a]` A declaration the importer cannot represent is recorded with the construct that defeated
  it and a workaround, and reported as `W5002`. A reference to it is a diagnostic saying it was skipped
  and why, never "unknown identifier". No declaration is silently absent.
* `[FFI-38]` The importer rejects rather than guesses: a fact it cannot establish leaves the
  declaration uncontracted (its calls need `unsafe`) and is reported; convenient-but-unsound is never
  an import mode.
* `[FFI-8]` **Type mapping.**

  | C | Ember |
  |---|---|
  | `_Bool`, `char`, `signed/unsigned char` | `bool`, `c_char` (`i8` or `u8` per target), `i8`, `u8` |
  | `short`, `int`, `long`, `long long` and unsigned forms | `i16`, `c_int`, `c_long`, `i64`, and `u16`, `c_uint`, `c_ulong`, `u64` |
  | `size_t`, `ptrdiff_t`, `intptr_t`, `uintptr_t` | `usize`, `isize`, `isize`, `usize` |
  | `float`, `double`, `long double` | `f32`, `f64`, `c_longdouble` (opaque bytes; no arithmetic) |
  | `__int128`, `unsigned __int128`, `_Float16` | `i128`, `u128`, `f16` |
  | `wchar_t`, `char16_t`, `char32_t` | `c_wchar` (`u16` on Windows, `i32` elsewhere — not portable), `u16`, `char` |
  | `volatile T*` | `Volatile[*T]` with `read_volatile`/`write_volatile` |
  | `__attribute__((packed))`, `#pragma pack`, `alignas` | `@packed`, `@align(N)` |
  | `T*`, `const T*` | `*mut T`, `*T`; with a contract, `ref`, `MutSpan`/`Span`, `Option[…]`, `ForeignBox[T]`, `cstr` |
  | `T[N]` field | `[T; N]` |
  | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union's fields are read only in `unsafe` |
  | incomplete `struct` | `extern type S`, used only behind pointers |
  | `enum` | `@repr(<underlying>) enum`, `@non_exhaustive` (any value may arrive) |
  | `typedef` | a type alias, or a new type with `@ffi(newtype)` |
  | `static const` global, `extern` global | `extern static`; reading it needs `unsafe` unless `@ffi(immutable)` |
  | `#define S "text"` | `const S: cstr = c"text"` |
  | function pointer | `extern "C" fn(…) -> R`, `Option[…]` when nullable; another calling convention is `extern "<conv>" fn` (`[FFI-9]`), and an unsupported one is skipped with `W5002` |
  | bit-field | `get_<f>`/`set_<f>` methods; not addressable |
  | anonymous member | a generated type `<Parent>_anon<N>` with transparent field access |
  | flexible array member | an unsized struct (never by value, `E5017`) with `unsafe fn tail(self, n) -> Span[T]` |
  | `_Atomic T` | `Atomic[T]` when layouts match, else `E5016` |
  | `va_list`, variadic `...` | `VaList`, forwardable only and never constructed (`E5018`); variadic calls only in `unsafe` with explicit argument types |
  | an identifier that is an Ember keyword | a raw identifier `r#type` (`[LEX-14]`) |

* `[FFI-29]` For every `import c` the importer emits a **shim** C file, compiled with that import's
  flags and linked into the package, containing the layout assertions (`[FFI-5a]`), an external
  wrapper for every `static inline` function Ember calls, and the wrappers of `[FFI-6b]`.
* `[FFI-29a]` A binding never names a symbol with internal linkage.
* `[FFI-29b]` `implementation = ["MINIAUDIO_IMPLEMENTATION"]` compiles a single-header library's
  implementation exactly once per build; two packages requesting it for one header is `E5014`.
* `[FFI-29c]` A call to a wrapped `static inline` function costs one extra call unless the shim takes
  part in LTO; `ember inspect` reports it as `wrapped (static inline)`. A variadic `static inline`
  function cannot be wrapped and is skipped with `W5002`.
* `[FFI-30]` Two imports of one C declaration (by Clang's USR, through typedefs) with the same layout
  are **one** Ember entity, whichever module, alias or overlay imports them; the same declaration with
  different layouts in one build is `E5011`, whose help is to give one import its own alias, making its
  entities distinct.
* `[FFI-30a]` An overlay changes the signatures, safety and names of *functions* and adds wrappers; it
  never changes a type's identity, layout or fields, so packages with different overlays for one header
  still exchange its values.
* `[FFI-30b]` `pub import c "…" as vk` re-exports the module under the ordinary visibility rules; the
  package's interface hash (`[BLD-2]`) includes the imported layouts, so a header change rebuilds
  dependants.
* `[FFI-30c]` A package may distribute an overlay for a foreign module, and several overlays for one
  module compose in a declared order; they may add aliases and wrappers and replace an entry only with
  an explicit `override`. Incompatible entries are an error, never a silent choice, and no overlay may
  change a foreign type's identity or layout.

## XVI.4 Contracts and overlays

An **overlay** is an Ember file that states contracts for the declarations of a header without
editing it.

```ember,overlay
overlay c "vulkan/vulkan.h":
    @ffi(handle)
    type VkBuffer
    @ffi(handle)
    type VkDevice
    @ffi(status, ok=VK_SUCCESS)
    enum VkResult

    @ffi(effects=[FFI, Alloc])
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status

    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount,
                        pViewports: borrowed count(viewportCount))

    rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction
```

* `[GRM-35]` *(new in 0.9.9)* **Overlay grammar.** An overlay is a file whose items are:

  ```text
  overlay_decl  := "overlay" "c" STRING ":" NEWLINE INDENT {overlay_item} DEDENT
  overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl)
  overlay_fn    := "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  contract      := IDENT ["(" [attr_arg {"," attr_arg}] ")"]
  ```

  Contract words take the place of types and are separated by spaces. `overlay`, `rename` and `hide`
  are contextual keywords (`[LEX-15]`).
* `[FFI-11]` *(changed in 0.9.9)* **Contract vocabulary.** A pointer contract has five axes;
  mutability comes from `const` and is never written.

  | Axis | Words | Ember type produced |
  |---|---|---|
  | ownership | `borrowed` (valid for the call), `owned` (callee takes it), `returns_owned(destructor=f)`, `retained` (callee keeps it, `[FFI-23]`) | `ref`/`Span`; a moved `ForeignBox[T]`; a `ForeignBox[T]` whose drop calls `f`; `Retained[T]` |
  | count | `one`, `count(n)` (as many elements as parameter `n`, which then leaves the signature), `nul_terminated`, `fixed(N)`, `inout_count(p)` (`[FFI-11b]`) | `ref T`, `Span[T]`/`MutSpan[T]`, `cstr`, `ref [T; N]` |
  | nullability | `nullable` | `Option[…]` |
  | lifetime (results) | `from(self)`, `from(p)`, `from(static)` | the region of the receiver, of `p`, or static (`[LT-1a]`) |
  | aliasing | `exclusive`, `aliased` | `ref mut` allowed only with `exclusive`; a result is `aliased` unless stated |

  Other contract words: `out` (an out-parameter, returned instead); `status` (with `@ffi(status,
  ok=X)` on the enum, `[FFI-16]`); `handle`; `callback(borrowed | retained | once, user_data=p)`
  (`[FFI-21]`); `threads(any | main | creator)`; `destroys(f)`; `unsafe` (leave the call unsafe).
  `@ffi(effects=[…])` states effects.
* `[FFI-11a]` A pointer contract with no count word is `E5012`, which lists the five count words and
  names any sibling parameter that looks like a length (an unsigned integer, or a name ending `Count`,
  `Len`, `Size` or `N`).
* `[FFI-11b]` The two-call enumeration idiom (call once for the count, again to fill) is `pCount:
  inout_count(pItems)` with `pItems: nullable count(pCount)`; the Ember function takes
  `Option[MutSpan[T]]` and returns the count.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and `TODO(lifetime)` are contract words
  that mark a fact as still unknown: the declaration stays uncontracted and its calls stay `unsafe`.
  A header can be adopted on the first day with everything behind `unsafe:`, and made safe one
  function at a time.
* `[FFI-12]` An overlay entry whose name, parameter count or parameter names do not match the header is
  `E5010`, so overlays cannot drift.
* `[FFI-13]` An overlay may contain `extend` blocks with ordinary Ember wrapper methods; this is how an
  idiomatic API is layered over a raw one.
* `[FFI-14]` The importer's result is a `.embind` file (versioned CBOR: header hash, Clang
  configuration, declarations with layouts, overlay hash, contracts), a content-addressed build input.
  `ember bind --explain header.h fn` prints a function's derived contract and why.
* `[FFI-35a]` A parameter with no `retained` word is recorded as "does not retain" in the foreign-trust
  report (`ember bind --report`), which lists every such assumption per module, and every
  result-position pointer with no lifetime word.

## XVI.5 Strings, status codes and handles

* `[FFI-15]` `cstr` is a borrowed, NUL-terminated C string (`c"…"` literals are `cstr`); `CString`
  owns one. `s.to_cstring()` allocates and fails on an interior NUL; `c.to_str() -> Result[str,
  Utf8Error]` validates without copying. Passing a `str` where a `cstr` is expected is `E5020`, whose
  fix-its are `.to_cstring()` and a `c"…"` literal.
* `[FFI-16]` An enum marked `@ffi(status, ok=X)` generates `struct <E>Error(code: E)` implementing
  `Error`; every function contracted `-> status` returns `Result[T, <E>Error]`, where `T` is the tuple
  of its `out` parameters (or `void`).
* `[FFI-36a]` `std.ffi.adopt[T](h) -> ForeignBox[T]` is an `unsafe fn` that takes ownership of a foreign
  object, reading its destructor from the contract of `T`'s declaration. Where the contract already
  records the transfer (`returns_owned(destructor=f)`), the import returns a `ForeignBox[T]` directly and
  no `adopt` is written.
* `[FFI-36b]` Adopting a type whose overlay says `adopt = false` is `E5052`. Adopting one live object
  twice is caught at run time in every profile (a panic naming both sites) and, when both calls are on
  one local in one function, at compile time.

## XVI.6 Callbacks and foreign retention

* `[FFI-21]` A C function-pointer parameter accepts a capture-free Ember function (as an `extern "C"
  fn`) or an `extern "C" fn` value. A capturing closure is `E5040` unless the parameter's contract
  names a `user_data` parameter, in which case the importer generates the trampoline: the closure is
  boxed and passed as `user_data`. With `callback(borrowed)` the box is freed after the call; with
  `callback(retained)` the call returns a `Retained` token whose drop unregisters through the declared
  `destroys(f)` (`E5041` if none is declared); with `callback(once)` the trampoline frees it after the
  first call.
* `[FFI-22]` Every exported function and trampoline first attaches the calling thread to the runtime
  (`ember_rt_thread_attach()`, a thread-local check after the first time). A closure or handle that
  reaches a foreign thread MUST be `Send` (`[THR-8]`).
* `[FFI-23]` `Retained.pin(v)` hands an Ember-owned object to C for keeping: for a class handle it
  retains the object and sets the header's pinned flag; for a `Box` it takes ownership. `tok.ptr()`
  is the raw pointer and dropping the token releases it. It is the only way to let foreign code keep
  Ember memory.
* `[FFI-33b]` `returns_owned`, `Retained` and callbacks may carry `threads(creator)`: a `ForeignBox`
  records its creating thread and, in `debug`, panics when dropped on another — the rule that makes
  GPU, GL-context and COM handles safe to hold in ordinary values.
* `[FFI-33a]` Attaching a thread (`[FFI-22]`) gives it no right to touch another thread's objects; in
  `debug` a retain or release of a non-`@sync` object from a thread other than its creator panics,
  naming the class and both threads.

## XVI.7 Exporting Ember to C

```ember
@export("game_on_update")
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32:
    return 0
```

* `[FFI-26]` `@export("symbol")` gives a function a stable C symbol with the C ABI.
  `@export_table("Name", protocol=N)` on a struct of `extern "C" fn` fields exports a table of function
  pointers under that name and protocol version, for hosts that load a module through one entry point. `ember build
  --emit-header` writes `<package>.h` with a prototype for each export, a typedef for each
  `@layout(c)` type they use, and each export's thread contract as a comment.
* `[FFI-25]` *(changed in 0.9.9)* A panic in an exported function, or anywhere below it, aborts the
  process (`[PAN-1]`); no panic unwinds into C. `@export(on_panic=abort)` is the only form in this
  version.
* `[FFI-31b]` No owning Ember value crosses a module boundary: class handles, `Box`, `Shared`, `Weak`,
  `String`, `Array`, `Map` and any type with drop glue MUST NOT appear in an exported signature or be
  reachable through a pointer in one (`E5015`). `@layout(c)` values, scalars, opaque handles, `cstr`
  and `extern "C" fn` may.
* `[FFI-31c]` With `[build] runtime = "shared"`, several Ember modules in one process share one runtime
  (hot reload requires it, `[HR-29]`), and the rule of `[FFI-31b]` still applies across their exported boundaries in
  this version.
* `[FFI-33]` `@export(threads=any | main | creator)` states which threads may call an export; the
  default is the file's `#! threads` directive (`[GRM-37]`), else `any`. Under `any`, everything the export reaches is checked as if it ran on any thread: every
  static it reaches is `Sync` and no non-`@sync` class handle is reachable from a static (`E7010`).
* `[FFI-33c]` *(new in 0.9.9)* Under `threads = main`, the exported wrapper checks, in every profile, that
  the calling thread is the one that initialised the module, and panics otherwise; in exchange the body
  may use thread-confined state. An `@export_table` may set `threads` per field.
* `[FFI-27]` The runtime `ember_rt` is a C11 static library with no global constructors. A C host calls
  `ember_rt_init(&cfg)` once (optionally routing allocation and logging to its own functions),
  `ember_rt_thread_attach()` on each thread that calls Ember, and `ember_rt_shutdown()` at the end,
  which runs at-exit hooks and, in `debug`, prints the leak report (`[WK-15]`).
* `[FFI-28]` A package built as `kind = "staticlib"` produces a library and header to link into a C or
  C++ program; `kind = "cdylib"` a shared library exporting only its `@export` symbols plus
  `ember_module_init` and `ember_module_shutdown`.
* `[FFI-31]` A `cdylib` links its own copy of the runtime. The host calls `ember_module_init` before
  using it and `ember_module_shutdown` before unloading it; shutdown runs at-exit hooks, detaches
  every thread the module attached, releases its thread-local state and reports leaks in `debug`.
* `[FFI-31a]` Its thread detachment never relies on a TLS destructor in the module's code.

## XVI.8 Build integration

* `[BLD-FFI-1]` The manifest's `[c]` section names the C compiler and flags used for shims and C source
  files (`import c "./bridge.c"` compiles `bridge.c` and imports `bridge.h`). By default they are the
  compiler Ember uses as its backend.
* `[BLD-FFI-1a]` Each `import c` is parsed and its shim compiled with the package's C flags plus the
  import's own `defines` and `include_paths`, and no others.
* `[BLD-FFI-2]` *(changed in 0.9.9)* The toolchain ships a CMake module: `ember_add_library(name KIND
  staticlib|cdylib SOURCES …)` and `ember_add_executable(…)` build Ember packages as ordinary targets of
  a C or C++ project, taking the target's compiler and flags with `--cc-flags-from-target <target>` or,
  for C++, through the CMake File API (Annex C).
* `[BLD-FFI-3]` Existing libraries are linked by name (`link = ["vulkan-1"]`) or path; link order
  follows declaration order.
