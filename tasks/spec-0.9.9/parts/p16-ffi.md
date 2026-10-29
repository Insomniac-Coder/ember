---

# Part XVI — Calling C

Being as fast as C includes using C's libraries with no glue. Ember calls a C function with one
direct call, lays out `@layout(c)` types exactly as the C compiler does, and reads C headers
directly. What a header cannot say — who owns a pointer, how many elements it points to, whether it
may be null, whether the function calls back into Ember — is stated once, in a **contract**, after
which calls are safe. C++ interoperation is Annex C.

```ember
from std.ffi import c_int, c_double

unsafe extern "C":
    @ffi(effects=[], no_callbacks)
    safe fn abs(x: c_int) -> c_int
    @ffi(effects=[], no_callbacks)
    safe fn cos(x: c_double) -> c_double

fn main():
    println(abs(-5), cos(0.0))
```

## XVI.1 Principles

* `[FFI-1]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Every foreign declaration has a **contract**: for each pointer-typed
  parameter and result, its ownership, mutability, nullability, count and lifetime (§XVI.4), and for
  the function its effects, its thread rule, whether it may call back into Ember (`[FFI-52]`) and
  whether it is **safe to call** at all. Facts derivable
  from the C declaration are derived (`[FFI-2a]`); the rest are `unknown` until a contract supplies
  them. `threads(creator)` and `handle` are words of a type's or a result's contract, not a function's:
  on a function each is a contract error (`E5002`).
* `[FFI-2]` *(changed in 0.9.9)* A call to a foreign function whose contract contains an `unknown` fact
  requires `unsafe`. Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe. The safe-to-call fact is never derived, because a scalar can be an address
  (`void free_addr(uintptr_t)`): it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an
  overlay entry for the function, which asserts that a call within its contract cannot break memory
  safety; such an overlay is an `unsafe overlay` (`[GRM-35]`). A header function with no overlay entry
  therefore stays unsafe to call. The diagnostic,
  `E5002`, names each missing fact and the overlay line that supplies it (`[DIA-18]`).
* `[FFI-2a]` *(changed in 0.9.15)* Derived facts need no `unsafe`: `const T*` is a read-only view, `T*`
  one through which C may write; the layouts of structs and unions, enum underlying types, packing and
  alignment, and the calling convention. In **result** position a pointer's lifetime is never derived.
* `[FFI-4]` No Ember panic crosses into foreign code (`[FFI-25]`).
* `[FFI-5]` Layouts are verified, not assumed: the importer records `sizeof`, `alignof` and field
  offsets from Clang for the project's target and flags, and the compiler checks its own layout
  against them (`E5001`, naming the type, field and both numbers).
* `[FFI-5a]` The generated shim (`[FFI-29]`) contains a `_Static_assert` for the size and alignment of
  every imported aggregate Ember constructs, passes, returns or embeds, and for the offset of every
  field Ember accesses, so the project's own C compiler re-verifies the layout on every build.

## XVI.2 Declaring C functions by hand

* `[FFI-10]` *(changed in 0.9.9)* *(changed in 0.9.15)* An `unsafe extern "C":` block declares foreign functions, statics
  and opaque types; `unsafe` records that the programmer asserts the signatures. The declared names
  are items of the enclosing module. A function in the block is **safe to call** only when it is
  declared `safe fn` (a contextual keyword, `[LEX-15]`) and every parameter and result is a scalar, a
  `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the
  declaration); `safe` on any other function is `E5002`, naming the fact that is missing. Every other
  call is in `unsafe`. `safe fn` is an asserted fact, listed by `ember tcb` (`[TCB-1]`). Its effects are
  those its `@ffi(effects=[…])` states plus `FFI`, and every effect when none are stated (`[EFF-3]`);
  which Ember code a call of it may run is `[FFI-52]`'s, and `@ffi(no_callbacks)` says it runs none.
  On a hand declaration, `@ffi(param(name, word, …), result(word, …))` attaches `[FFI-11]`'s
  contract words to each named pointer parameter and the pointer result (ODR-073). Each slot is named
  at most once; `link_name`, `effects` and `no_callbacks` can share the same `@ffi`. A `safe fn` writes
  the safe Ember types and the words in front of its parameters' names (`ref mut`, `owns`) produced by
  those words; the compiler checks that
  mapping and lowers it to the C ABI carrier. A raw `*T` can be declared for an unsafe call, but
  complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is
  `E5002`. Missing facts or a type/contract mismatch is `E5002`, naming the slot.
  On a hand-declared `safe fn` with `count(n)`, `n` is written as an integer scalar parameter at
  its C ABI position (ODR-074). The contract marks it as an ABI witness: it is absent from the
  callable Ember signature, and the compiler supplies the corresponding `Span[T]` or `MutSpan[T]`
  length at the call. The written integer type fixes the C width and signedness. A length that does
  not fit that type, or unequal lengths from two pointer clauses sharing `n`, panics before the
  foreign call. A missing or non-integer witness or an incompatible pointer surface is `E5002`.
  An unsafe raw-pointer declaration keeps its written C ABI parameters; only a mapped `safe fn`
  erases the witness.
  For an input `param(p, borrowed, count(n), nullable)`, the safe type is `Option[Span[T]]`, or, with
  `exclusive`, `Option[MutSpan[T]]` on a `ref mut` parameter, whose argument the call marks `ref mut`
  (`[FN-2a]`) (ODR-079). `None` sends C a null pointer and integer zero.
  `Some` sends its checked length, and a non-null pointer even when empty: if the view's data
  pointer is null, the wrapper supplies a statically stored, properly aligned `T` sentinel.
  The C function must not access an element when the count is zero. Several input clauses sharing
  `n` compare effective lengths, with `None` contributing zero. A counted result sharing `n`
  receives that effective length. This does not define `inout_count`.
  A borrowed pointer **result** contracted `count(n)` is safe only if at least one borrowed input
  `Span` or `MutSpan` also has `count(n)` (ODR-077). That input supplies the hidden C witness and
  the returned view length; several such inputs must have equal lengths before C. A result is
  `Span[T]` through a C `const T*`, or `MutSpan[T]` through a mutable C `T*` when contracted
  `exclusive` (ODR-078). With `nullable`, a null pointer is `None`, otherwise the result is
  `Some(Span[T])` or `Some(MutSpan[T])` with that length. A non-null zero-length result is still
  `Some`.
  A result-only `count(n)` on a `safe fn` is `E5002`: the compiler has no input length to send to C.
  The result still needs `from(p)` or `from(static)`. An unsafe raw declaration keeps its written
  count parameter; this rule does not define `inout_count` or another output-count carrier.
* `[FFI-9]` `extern "C"` is the platform C ABI (SysV AMD64, Windows x64, AAPCS64). `extern "system"`
  is `stdcall` on 32-bit Windows and C elsewhere; `extern "vectorcall"` and `extern "fastcall"` exist
  on x86. Struct arguments and results follow the ABI exactly. An ABI-direct call is one `call`
  instruction and nothing else.
* `[FFI-49]` *(new in 0.9.9)* `@ffi(link_name="sym")` binds a declaration to a differently named symbol.
  Libraries are linked by `[link] libs = ["m", "vulkan-1"]` in the manifest or `link = [...]` on an
  `import c` (`[BLD-FFI-3]`).

## XVI.3 Importing C headers

```ember,fragment
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
* `[FFI-51]` *(new in 0.9.15)* **C compiled into the program is vouched for.** An import that compiles C
  into the program — a C source file (`import c "./bridge.c"`, `[BLD-FFI-1]`), or a header with
  `implementation = [...]` (`[FFI-29b]`) — runs C code that nothing checks, a constructor in it even
  before `main`, so it is written `unsafe import c`, with a `# SAFETY:` note above it saying why that
  code is sound (`W3012` when missing). Without `unsafe` it is `E3100`: "`import c "./bridge.c"`
  compiles `bridge.c` into the program, where its code runs unchecked, before `main` too; say that
  you vouch for it: write `unsafe import c "./bridge.c" as b` with a `# SAFETY:` note above it". An
  import of a header alone needs no word: its functions run only when they are called, and those
  calls are fenced by `[FFI-2]`. A library linked by name or path (`link = [...]`, `[link] libs`) is
  listed by `ember tcb` as an external library (`[TCB-4]`).

  ```ember,fragment
  # SAFETY: bridge.c is ours: it defines `add` and runs nothing before `main`
  unsafe import c "./bridge.c" as b
  ```
* `[FFI-6a]` *(changed in 0.9.10)* An object-like macro is imported when, after expansion, it is a constant expression of
  integer, floating, string, null-pointer or pointer-cast type as Clang evaluates it; its type is the
  C type of that value. A null-pointer or pointer-cast macro (`((void*)0)`, `((HANDLE)-1)`) is a
  `const` of that pointer type holding the address as an integer (`[CT-2]`, ODR-145). A macro that
  expands to nothing (an include guard, a feature flag) declares nothing and is not reported
  (ODR-146). Any other macro is skipped with `W5001`, giving Clang's reason.
* `[FFI-6b]` *(changed in 0.9.10)* A function-like macro is exposed only when an overlay declares its signature
  (`@ffi(macro_fn)`); the importer emits a C wrapper function for it into the shim. Ember gains no
  macro facility. A macro has no C types to derive contracts from, so its entry writes Ember types in
  their place: `@ffi(macro_fn) fn SQUARE(x: i32) -> i32` (`[GRM-35]`, ODR-147). When every argument is a constant the importer may fold the call, so the result can
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
* `[FFI-8]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Type mapping.**

  | C | Ember |
  |---|---|
  | `_Bool`, `char`, `signed/unsigned char` | `bool`, `c_char` (`i8` or `u8` per target), `i8`, `u8` |
  | `short`, `int`, `long`, `long long` and unsigned forms | `i16`, `c_int`, `c_long`, `i64`, and `u16`, `c_uint`, `c_ulong`, `u64` |
  | `size_t`, `ptrdiff_t`, `intptr_t`, `uintptr_t` | `usize`, `isize`, `isize`, `usize` |
  | `float`, `double`, `long double` | `f32`, `f64`, `c_longdouble` (opaque bytes; no arithmetic) |
  | `__int128`, `unsigned __int128`, `_Float16` | `i128`, `u128`, `f16` |
  | `wchar_t`, `char16_t`, `char32_t` | `c_wchar` (`u16` on Windows, `i32` elsewhere — not portable), `u16`, `u32` (C may hand back a surrogate or a value above U+10FFFF; `char.from_u32(v) -> Option[char]` converts; Ember's `char` itself never crosses, `[FFI-31b]`) |
  | `volatile T*` | `Volatile[*T]` with `read_volatile`/`write_volatile` |
  | `__attribute__((packed))`, `#pragma pack`, `alignas` | `@packed`, `@align(N)` |
  | `T*`, `const T*` | `*mut T`, `*T`; with a contract (`[FFI-11]`), a parameter `x: T` or `ref mut x: T`, `Span[T]`/`MutSpan[T]`, `Option[…]`, `owns T`, `CStrView` |
  | `T[N]` field | `[T; N]` |
  | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |
  | incomplete `struct` | `extern type S`, used only behind pointers |
  | `enum` | `@repr(<underlying>) enum`, `@non_exhaustive` (any value may arrive) |
  | `typedef` | a type alias; with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only |
  | `static const` global, `extern` global | `extern static`; reading it needs `unsafe` unless `@ffi(immutable)` asserts a `const` C object and an unchanged value (ODR-082). The generated C declaration is `extern const T` with that assertion, or `extern T` without it; a safe read also requires `T: needs ThreadShared`. `@ffi(immutable)` on `static mut` is `E5002`. A global read only in `unsafe` need not be `ThreadShared` (`[STA-1]`), so one holding a pointer imports as any other (ODR-163). |
  | `#define S "text"` | `const S: CStrView = c"text"` |
  | function pointer | `unsafe extern "C" fn(…) -> R` (`[FFI-56]`), `Option[…]` when nullable; another calling convention is `unsafe extern "<conv>" fn` (`[FFI-9]`), and an unsupported one is skipped with `W5002` |
  | bit-field | `get_<f>`/`set_<f>` methods; not addressable |
  | anonymous member | a generated type `<Parent>_anon<N>` with transparent field access |
  | flexible array member | an unsized struct, `@layout(c, flexible)` (never by value, `E5017`), with `unsafe fn tail(n: usize) -> Span[T]` |
  | `_Atomic T` | `Atomic[T]` when layouts match, else `E5016` |
  | `va_list`, variadic `...` | `VaList`, forwardable only and never constructed (`E5018`); variadic calls only in `unsafe`, each further argument of a written type (a literal's suffix or `as`); one C's default argument promotions would change (`bool`, an integer narrower than `int`, `f32`) is `E5013`, which names only C's promotion, since what the C function reads is in C (verdict 123): "C passes `narrow` as a 4-byte `int` through `...`; write the size `sum_wide` expects: `narrow as c_int` for 4 bytes, or `narrow as i64` for 8"; for an `f32`, "C passes `single` as an 8-byte `double` through `...`; write `single as f64`" |
  | an identifier that is an Ember keyword | a raw identifier `r#type` (`[LEX-14]`) |

* `[FFI-29]` For every `import c` the importer emits a **shim** C file, compiled with that import's
  flags and linked into the package, containing the layout assertions (`[FFI-5a]`), an external
  wrapper for every `static inline` function Ember calls, and the wrappers of `[FFI-6b]`.
* `[FFI-29a]` A binding never names a symbol with internal linkage.
* `[FFI-29b]` *(changed in 0.9.15)* `implementation = ["MINIAUDIO_IMPLEMENTATION"]` compiles a
  single-header library's implementation exactly once per build, in an `unsafe import c` (`[FFI-51]`);
  two packages requesting it for one header is `E5014`.
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
* `[FFI-30c]` *(changed in 0.9.10)* *(changed in 0.9.15)* A package may distribute an overlay for a foreign module, and several overlays for one
  module compose in a declared order; they may add aliases and wrappers and replace an entry only with
  an explicit `override`. Incompatible entries are an error, never a silent choice, and no overlay may
  change a foreign type's identity or layout. The order is declared where the module is imported:
  `overlay` takes one file or a list, applied first to last (a dependency's before the package's
  own); a later file's `override` entry replaces an earlier file's for that name, and any other
  second entry for one name is `E5010` (ODR-149). An overlay of another package is applied only to the
  header it was written for (`[FFI-55]`).

## XVI.4 Contracts and overlays

An **overlay** is an Ember file that states contracts for the declarations of a header without
editing it.

```ember,overlay
unsafe overlay c "vulkan/vulkan.h":
    @ffi(handle)
    type VkBuffer
    @ffi(handle)
    type VkDevice
    @ffi(status=VkResult, ok=VK_SUCCESS, error=VkError)
    enum VkResult

    @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status

    @ffi(no_callbacks)
    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount,
                        pViewports: borrowed count(viewportCount))

    rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction
```

* `[GRM-35]` *(new in 0.9.9)* *(changed in 0.9.15)* **Overlay grammar.** An overlay is a file whose items are:

  ```text
  overlay_decl  := ["unsafe"] "overlay" ("c" | "cpp") STRING
                   ":" NEWLINE INDENT {overlay_item} DEDENT
  overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} fn_header NEWLINE} DEDENT
  overlay_fn    := "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  contract      := IDENT ["(" [attr_arg {"," attr_arg} [","]] ")"]
  ```

  In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember
  type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147). An
  item may begin with `override`, replacing an earlier overlay's entry for that name (`[FFI-30c]`,
  ODR-149). An `overlay_class` section restates methods of a C++ class, each as an Ember method line
  with no body, whose `@ffi(const)`, when written, asserts that the method only reads its object
  (`[FFI-40a]`, ODR-160). Ember records the header each overlay was written for in `ember.overlays`,
  never in the overlay (`[FFI-55]`).

  Contract words take the place of types and are separated by spaces. `overlay`, `rename` and `hide`
  are contextual keywords (`[LEX-15]`). An overlay whose items state any fact — a contract word, an
  `@ffi(…)` attribute, a function entry, which asserts the function safe to call (`[FFI-2]`), or a
  method line — is an
  assertion about foreign code and MUST be declared `unsafe overlay` (`E5067`), a boundary of
  `[TIER-1]`; an overlay of only `rename` and `hide` items needs no `unsafe`.
* `[FFI-11]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Contract vocabulary.** A pointer contract has five axes;
  mutability comes from `const` and is never written. The same words appear after a parameter name
  in an overlay and inside `@ffi(param(name, word, …), result(word, …))` on a hand declaration
  (ODR-073); that attribute does not create a separate contract system.

  | Axis | Words | Ember type produced |
  |---|---|---|
  | ownership | `borrowed` (valid for the call), `owns` (callee takes it; verdict 102), `returns_owned(destructor=f)`, `retained` (callee keeps the pointer after the call) | a parameter that views the caller's value for the call (`x: T`, `Span[T]`; `ref mut x: T`, `MutSpan[T]` with `exclusive`); an `owns x: owns T` parameter, whose argument is handed over and which Ember then never frees; an `owns T` result, whose end calls `f`; for a callback, `callback(retained)` (`[FFI-21]`), and on any other pointer no safe type, since C keeps Ember memory only when it is handed over (`owns`) or given with `into_raw()` (`[FFI-23]`, `[FFI-54]`): the call stays `unsafe`, with a raw pointer |
  | count | `one`, `count(n)` (as many elements as parameter `n`, which then leaves the callable signature; a hand declaration writes its ABI witness under `[FFI-10]`, ODR-074; a result needs an input span sharing `n`, ODR-077; nullable input carriers follow ODR-079), `nul_terminated`, `fixed(N)`, `inout_count(p)` (`[FFI-11b]`) | one `T` (a parameter `x: T`, a result `ref T`), `Span[T]`/`MutSpan[T]`, `CStrView`, `[T; N]` (a parameter `x: [T; N]`, a result `ref [T; N]`) |
  | nullability | `nullable` | `Option[…]` |
  | lifetime (results) | `from(this)`, `from(p)`, `from(static)` | a view into the receiver, into `p`, or into what lives for the whole program (`[LT-1a]`, `[LT-3]`) |
  | aliasing | `exclusive`, `aliased` | `ref mut` and `MutSpan[T]` results allowed only with `exclusive`; a result is `aliased` unless stated |

  A parameter `borrowed one exclusive`, and one whose count makes a `MutSpan[T]` that C writes, is
  written with `ref mut` in front of its name (`ref mut p: T`, `ref mut p: MutSpan[T]`), for an opaque
  type too, and the call marks its argument `ref mut` (`[FN-2a]`); a plain parameter only reads what it
  views.

  Other contract words: `out` (an out-parameter, returned instead); `status` (with `@ffi(status=E,
  ok=X, error=N)` on the enum, `[FFI-16]`); `handle`; `callback(borrowed | retained | once, user_data=p)`
  (`[FFI-21]`); `threads(any | main | creator)`; `destroys(f)` (`f`'s parameters may be the registering
  call's own and its result, `id = listen(…)` then `unlisten(id)`: the token keeps them); `unsafe`
  (leave the call unsafe); and, on a function, `no_callbacks` (it never calls back into Ember,
  `[FFI-52]`), and on a function-pointer typedef, `safe` (`[FFI-56]`).
  `@ffi(effects=[…])` states effects.
* `[FFI-11a]` A pointer contract with no count word is `E5012`, which lists the five count words and
  names any sibling parameter that looks like a length (an unsigned integer, or a name ending `Count`,
  `Len`, `Size` or `N`).
* `[FFI-11b]` *(changed in 0.9.10)* *(changed in 0.9.15)* The two-call enumeration idiom (call once for the count, again to
  fill) is `pCount: inout_count(pItems)` with `pItems: nullable count(pCount)`; the Ember function
  takes an `Option[MutSpan[T]]` on a `ref mut` parameter and returns the count. When the C function has
  a result of its own, the Ember function returns a tuple: that result first, then each pointer output
  (`out`, the count) in the C declaration's order; a status result follows `[FFI-16]` instead.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and `TODO(lifetime)` are contract words
  that mark a fact as still unknown: the declaration stays uncontracted and its calls stay `unsafe`.
  A header can be adopted on the first day with everything behind `unsafe:`, and made safe one
  function at a time.
* `[FFI-12]` An overlay entry whose name, parameter count or parameter names do not match the header is
  `E5010`, so overlays cannot drift.
* `[FFI-55]` *(new in 0.9.15)* **Every overlay is checked against the header it was written for, by a
  record Ember keeps** (verdict 121). A package with an overlay of a C or C++ header has a file
  `ember.overlays` beside its `ember.toml`, written by Ember and never by hand. For each overlay it
  holds the overlay's file, its header and one `blake3:` fingerprint of each declaration the overlay
  describes, as the importer binds it (`[FFI-14]`):

  ```text
  # ember.overlays: the headers this package's overlays were checked against.
  # Written by ember; `ember bind --merge` rewrites it.

  [[overlay]]
  file = "overlays/sdl.em"
  header = "SDL.h"
  SDL_FreeSurface = "blake3:…"
  ```

  Ember writes the record the first time the package's own build applies the overlay, and only
  `ember bind --merge` rewrites it (`[CLI-5]`). Every build that applies an overlay, its own package's
  and any other's, compares each declaration the overlay describes with the header it imports. One
  that differs is `E5010`, naming the overlay, the declaration as recorded and as imported, and the
  way out ("`SDL_FreeSurface` was `void SDL_FreeSurface(SDL_Surface*)` when `overlays/sdl.em` was
  checked, and the `SDL.h` imported here declares `void SDL_FreeSurface(SDL_Surface*, int)`; check the
  overlay against this header, then run `ember bind --merge`"); an overlay its package has no record
  of is `E5010` too. A change to a declaration the overlay does not describe changes nothing. Nothing
  of this is written in the overlay or in the program.
* `[FFI-13]` An overlay may contain `extend` blocks with ordinary Ember wrapper methods; this is how an
  idiomatic API is layered over a raw one.
* `[FFI-14]` The importer's result is a `.embind` file (versioned CBOR: header hash, Clang
  configuration, declarations with layouts, overlay hash, contracts), a content-addressed build input.
  `ember bind --explain header.h fn` prints a function's derived contract and why.
* `[FFI-35a]` *(changed in 0.9.15)* A parameter with no `retained` word is recorded as "does not retain" in the foreign-trust
  report (`ember bind --report`), which lists every such assumption per module, and every
  result-position pointer with no lifetime word. A class object given to a `retained` parameter is
  marked in its header as kept by C and never moves: handed to another thread while a `weakref` link on
  this thread points at it, it stops the program (`[THR-2]`, verdict 114).

## XVI.5 Strings, status codes and handles

* `[FFI-15]` *(changed in 0.9.15)* `CStrView` is a read-only view of a NUL-terminated C string
  (`c"…"` literals are `CStrView`); `CString` owns one. `s.to_cstring() -> Result[CString, NulError]`
  scans a `StrView` for its first interior NUL before allocating; `NulError.index` is that byte offset
  (ODR-081). On success it copies the bytes and appends one terminator. `CString.as_cstr() -> CStrView`
  is a view into its owner and never allocates; there is no implicit `CString` to `CStrView`
  conversion. `c.to_str() -> Result[StrView, Utf8Error]` validates without copying. Passing a
  `StrView` where a `CStrView` is expected is `E5020`, which suggests a `c"…"` literal or handling
  `.to_cstring()`'s result and viewing the successful `CString` with `.as_cstr()`. For a safe
  hand-declared borrowed `nul_terminated` pointer with `nullable`, the parameter or result type is
  `Option[CStrView]` (ODR-080). `None` crosses as a null C pointer and `Some` as its non-null character
  pointer. A null result becomes `None`; a non-null result becomes `Some` and points into what the
  contract's `from(p)` or `from(static)` names. The nullable word changes neither the NUL-termination
  obligation nor the result's lifetime obligation.
* `[FFI-16]` *(changed in 0.9.10)* An enum `E` marked `@ffi(status=E, ok=X, error=N)` (its success
  value `X`, its error type's name `N`) generates `struct N(code: E)` implementing `Error`; every
  function contracted `-> status` returns `Result[T, N]`, where `T` is the tuple of its `out`
  parameters (or `void`).
* `[FFI-36a]` *(changed in 0.9.15)* `std.ffi.adopt[T](h) -> owns T` is an `unsafe fn` that takes
  ownership of a foreign object, reading its destructor from the contract of `T`'s declaration,
  `@ffi(destroys(f))` on `type T`; the `owns T` value's end calls `f` once. Where the contract already
  records the transfer (`returns_owned(destructor=f)`), the import returns an `owns T` directly and
  no `adopt` is written. A foreign type has one destructor: every `returns_owned(destructor=f)` giving
  an `owns T` and `T`'s `@ffi(destroys(f))` name the same `f`, and two different ones are `E5002`
  ("`Widget` is ended by `widget_free` (line 3), and line 12 says `widget_destroy`; a foreign type has
  one destructor"), so an `owns T` foreign value stays one word.
* `[FFI-36b]` Adopting a type whose overlay says `adopt = false` is `E5052`. Adopting one live object
  twice is caught at run time in every profile (a panic naming both sites) and, when both calls are on
  one local in one function, at compile time (`E5068`).

## XVI.6 Callbacks, what a foreign call may run, and foreign retention

* `[FFI-21]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* A C function-pointer parameter accepts a capture-free Ember function
  (as an `extern "C" fn`) or an `extern "C" fn` value. A function that captures anything is `E5040`
  unless the parameter's contract names a `user_data` parameter, in which case the importer generates
  the checked trampoline (`[FFI-53]`): the function value, its code and its state, is moved into a
  block of its own, and C is given the trampoline as the function pointer and the block's tagged
  number as the user data. With `callback(borrowed)` C runs the function only during the call, the
  block is freed when the call returns, and the function may use the caller's real variables, as a
  function given to any call that runs it then may (Part VI), within `[FFI-22]`'s thread rule. With
  `callback(retained)` or `callback(once)` C keeps the function after the call: its capture list says
  what it holds (`owns`, `copy of`, `thread_shared`; `shares`, `shares mut` and `weakref` only under
  `threads(creator)` or `threads(main)`, `[FFI-22]`), and a name it uses with no word is `E7004`: "C
  keeps this function after `setup` returns (`callback(retained)`), so it cannot use `setup`'s `game`;
  say what it holds: write `return register_cb(fn() owns game: game.add_point())` in place of line 17
  (`setup`'s owner of the game), or capture `fn() weakref game` with
  `if game exists as ref mut g: g.add_point()` as its body (a link that lets the game end)".
  `retained` returns a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the
  callback through the declared `destroys(f)` (`E5041` if none is declared), and with `once` the
  trampoline ends the block after the first call. A callback with several parameters of its user
  data's type names the one that receives it: `user_data=p@N`, the callback's `N`th parameter
  (1-based, as in the C signature); without `@N`, exactly one parameter has that type, or the contract
  is `E5002`, whose help shows the `@N` form. A `nullable` callback with user data is an `Option` of a
  function value: `None` passes `NULL` and no user data. Handing a function to a `callback(retained)`
  or `callback(once)` parameter counts as making what it changes, now and whenever C runs it
  (`[FFI-52]`).
* `[FFI-21a]` *(new in 0.9.10)* *(changed in 0.9.15)* **A parameter C passes as a copy.** C passes a function's arguments by
  value. When a function's value becomes an `extern "C" fn` (`[FN-6]`), a plain parameter
  (`[FN-1]`) whose type is not a pointer is, for a call from C, a copy the call makes, which ends when
  the call returns; called from Ember, it is still a view of the caller's value. Where a program turns
  a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw
  pointer into such a parameter: one made from a view of it or of a part of it
  (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and
  aggregates in the function, is `E5042`, shown where it is returned and where the type becomes
  `extern "C" fn`. A value read through the parameter, and a pointer stored in it, are not into the
  copy. A pointer returned from a call to another function is not followed. A function that must give
  C back a pointer it was given takes that parameter as a pointer (`*T`).
* `[FFI-22]` *(changed in 0.9.9)* *(changed in 0.9.15)* Every exported function and trampoline first attaches the calling
  thread to the runtime (`ember_rt_thread_attach()`, a thread-local check after the first time). The
  compiler cannot see which thread C calls a callback from, so the callback's contract is the only
  evidence, and it is an asserted fact (`[TCB-1]`): `threads(main)` or `threads(creator)`, which the
  trampoline checks in every profile (`[FFI-53]`), or `threads(any)`, the default. A callback C may
  call from any thread, borrowed or kept, is a `thread_shared fn()` (Part VI): every value it holds is
  `Send` (`[THR-8]`): a one-thread owner (`shares`, `shares mut`, a `weakref` link of one) is `E7015`,
  a thread-bound value (a lock's guard, a raw pointer, a `threads(creator)` handle) `E7008`; and it
  changes the caller's real variables only by their own operations, as an `Atomic`, a lock or a
  `guarded by` field under its lock is changed (`E7015`: "C may call `visit` from any thread at once
  (`visit_each`'s `visit` has no `threads(...)` word), and it pushes to `seen`, a list of this thread;
  count with an atomic, or, if C calls it only on this thread, say so in the overlay:
  `visit: callback(borrowed, user_data=data) threads(creator)`"). The `E7015` message for a kept
  callback that holds a `shares` owner gives the lines that share the value between threads
  (`thread_shared`, with `Atomic` fields).
  Its message: E7015 ("C may call this function from any thread (`threads(any)`), and `shares` owners stay on one thread; share the game between threads: write `game = thread_shared Game(names=[String("ann"), String("bo")])` and `token = setup(game)` in `main`, `fn setup(game: thread_shared Game) -> Retained[ForeignCallback]:` and `return register_cb(fn() thread_shared game: game.add_point())` in `setup`, and, since `add_point` changes `score`, `from std.sync import Atomic` at the top, `score: Atomic[i32] = Atomic(0)`, `this.score.fetch_add(1)` as `add_point`'s body, and `println(game.score.load())`").
* `[FFI-52]` *(new in 0.9.15)* **What a foreign call may run.** The compiler cannot read a foreign
  function's code, so a call of one counts as possibly running, on the calling thread, every Ember
  function C can reach then: every callback C keeps (one handed to a `callback(retained)` or
  `callback(once)` parameter and not yet unregistered) whose thread contract allows this thread; every
  `@export` and `@export_table` function whose thread contract (`[FFI-33]`) allows it; every override
  of an Ember class derived from a C++ class whose objects C++ may hold (`[FFI-39d]`); every
  `callback(borrowed)` function whose own call is still running on this thread; and every Ember
  function given to C as an `extern "C" fn` value. The call makes every change those functions make,
  worked out from their bodies as every call's changes are (Part X), so the rules for loops, read-only
  names, `ref mut` names and views of objects reached through links (`[BCK-3]`, `[BCK-7]`) apply to it,
  and to every function that calls it: a `fn tick(): poll_events()` changes what the kept callbacks
  change, and so does every function that calls `tick`. A foreign function that never calls back into
  Ember says so on its declaration or overlay entry, `@ffi(no_callbacks)`: a call of it runs none of
  them. The mark is an asserted fact, listed by `ember tcb` (`[TCB-1]`); on a function whose own
  contract takes a callback it is `E5069`: "`register_cb` takes a callback (`param(cb,
  callback(retained, user_data=ud))`), so it may call back into Ember; remove `no_callbacks` from its
  `@ffi(...)`". A missing mark only refuses a line that the reachable functions' changes would make
  unsafe, and the message names the mark (`E3020`): "`sqrt(d)` (line 21) is a C function, which may run
  the callback C keeps from `register_cb` (line 14), and that callback changes `game.names`, which this
  loop walks (line 20); if `sqrt` never calls back into Ember, say so on its declaration:
  `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`; otherwise move line 21 below the loop"; for an
  export, "`host_poll()` (line 12) may run `log_line` (exported, line 5), which adds to `LOG` (line 7)
  while this loop walks `LOG` (line 11); call `host_poll()` before or after the loop, or, if
  `host_poll` never calls back into Ember, mark it `@ffi(no_callbacks)` in `host.overlay.em`". Across a
  package's edge, the callbacks a package hands to C, the functions it exports and the overrides it
  gives C++ are recorded in its interface file (`[EFF-4]`), so a program that uses the package counts
  them at every foreign call. Nothing runs while the program runs for this rule.

  ```ember
  from std.ffi import ForeignCallback, Retained

  class Game:
      score: i32 = 0
      names: Array[String] = []

      fn add_point():
          this.score += 1

  unsafe extern "C":
      fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
      @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
      safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
      safe fn poll_events()
      @ffi(no_callbacks)
      safe fn sqrt(x: f64) -> f64

  fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
      return register_cb(fn() owns game: game.add_point())

  fn main():
      game = shares mut Game(names=[String("ann"), String("bo")])
      token = setup(shares mut game)
      for name in game.names:
          println(name, sqrt(name.len() as f64))
      poll_events()
      println(game.score)
  ```

  The callback C keeps holds `setup`'s owner of the game (`fn() owns game`), a one-thread owner, which
  its `threads(creator)` contract allows. `poll_events()` may run it, and so changes `game.score`; the
  callback changes nothing in `game.names`, so a C call inside the walk of `game.names` would be
  allowed here too. With a callback that pushes to `game.names`, `poll_events()` inside the walk is
  `E3020`, and so is `sqrt` there without its mark.
* `[FFI-53]` *(new in 0.9.15)* **The checked trampoline.** C never holds the address of an Ember value
  for a callback: the user data it is given is a tagged number, the callback's slot in a table that
  never moves in the low 48 bits and the slot's generation in the top 16, which are never all zero, so
  the number is no user-space address and a C library that wrongly writes through it faults before
  writing anything. On a 32-bit target the slot is the low 16 bits and the generation the top 16, and
  the number may be a user-space address. Each call through the trampoline checks, in every profile, that the slot is inside
  the table, then the generation and the function's type in one compare, then, for a
  `threads(creator)` or `threads(main)` callback, the calling thread; a `callback(once)` slot carries a
  used-up mark. A failed check stops the program before the callback's state is read, with these
  texts, in the program's lines:

  | check | the stop's text |
  |---|---|
  | a call after the callback's token ended | "C called the callback registered on line 16 after its token ended; it was unregistered" |
  | a `callback(once)` function called a second time | "C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once" |
  | a `threads(creator)` callback called on another thread | "C called the callback registered on line 16 on another thread, but its contract says `threads(creator)`" |

  A slot whose generation reaches its last value is retired and never reused, as `[HND-3]` retires a
  `Pool` slot, so a stale number can never name a later callback: one compare per registration, nothing
  per call. A `threads(any)` callback that can be unregistered while it runs marks its slot as running,
  and unregistering waits until it returns. A callback that ends its own token while it runs (its body
  sets the field holding the `Retained` token to `None` and then uses its state) is unregistered at
  once, so C never runs it again, and the block that holds its state is ended and freed only when it
  returns, so the rest of its body runs as written. The trampoline costs 1.02x to 1.05x of a plain C
  call through a function pointer, and, in the `release` profile, one compare of the thread per
  `threads(creator)` or `threads(main)` callback.
* `[FFI-23]` *(changed in 0.9.10)* *(changed in 0.9.15)* `Retained[ForeignCallback]` is the token a
  registering call returns for a callback C keeps (`[FFI-21]`): it keeps the callback's block on
  Ember's side while C holds its tagged number, and its end unregisters the callback through
  `destroys(f)` and then ends the block (`[FFI-53]` for a callback that ends its own token). `Retained`
  gives no object away and keeps none for C: an Ember value C keeps is handed to a parameter whose
  contract says `owns` (the name ends and Ember frees nothing, `[FFI-11]`) or, for a C function whose
  declaration cannot say so, given with `into_raw()` inside `unsafe:` (`[FFI-54]`); C gives a foreign
  object back through `adopt` inside `unsafe:` (`[FFI-36a]`). An `ObjectPtr[C]` is the address of a
  class object, which `into_raw()` gives, with no count of its own (to C, one pointer): `Copy`, equal
  when it is the address of the same object.
* `[FFI-54]` *(new in 0.9.15)* **Giving an object to C.** A C function declared as keeping an object
  (`@ffi(param(widget, owns, one))` with `owns widget: owns Widget`) is given one by an ordinary
  hand-over, `widget_take(first)`: the name ends and Ember frees nothing, with no word. For a C function
  whose declaration cannot say so (a plain pointer, `fn set_user_data(p: *mut Widget)`, or a function
  with `...`), `w.into_raw()` gives the address of a value Ember holds alone in its own block, an
  `owns T` (as a `*mut T`, or a `*mut u8` for an `owns dyn I`) or a class object with one owner (as an
  `ObjectPtr[C]`), and stops Ember freeing it, as C++'s `unique_ptr::release()` does; the name ends.
  `into_raw()` is built into the compiler, which makes that step itself, so nothing else can skip a
  value's clean-up (`mem.forget` is not Ember). Outside `unsafe:` it is `E3100`: "`into_raw()` stops
  Ember from freeing the widget, so a mistake leaks it or frees it twice; write it inside `unsafe:`:
  above line 18 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 18 under it".

  ```ember
  unsafe extern "C":
      @ffi(destroys(widget_free))
      type Widget
      fn widget_free(widget: *mut Widget)
      @ffi(result(returns_owned(destructor=widget_free), one))
      safe fn widget_new(id: i32) -> owns Widget
      @ffi(param(widget, owns, one))
      safe fn widget_take(owns widget: owns Widget)
      fn set_user_data(p: *mut Widget)

  fn address_for_c(owns w: owns Widget) -> *mut Widget:
      # SAFETY: the address goes to `set_user_data`, whose C side keeps the widget and frees it with `widget_free`
      unsafe:
          return w.into_raw()

  fn main():
      first = widget_new(1)
      widget_take(first)
      second = widget_new(2)
      p = address_for_c(second)
      # SAFETY: C keeps the widget as its user data and frees it with `widget_free`
      unsafe:
          set_user_data(p)
  ```
* `[FFI-56]` *(new in 0.9.15)* **A C function pointer is unsafe to call.** A function pointer that
  comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C
  passes to a callback — imports as `unsafe extern "C" fn(…)` (`Option[…]` when nullable, `[FN-6]`), so
  a call through it is in `unsafe:` (`E3100`: "`release` is a C function pointer from `hooks.h`, with no
  contract; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe"):
  nothing tells the compiler what the function it points to does with its arguments
  (`void free_addr(uintptr_t)`, `[FFI-2]`). An overlay may assert one safe to call with `@ffi(safe)`
  on the typedef's entry (`@ffi(safe) type PFN_log_message`) when every parameter and result is a
  scalar or a `@layout(c)` value type (`E5002` otherwise): an asserted fact, listed by `ember tcb`. A
  function pointer made from an Ember function is a safe `extern "C" fn` (`[FN-6]`). The end of an
  `owns T` foreign value calls the destructor its asserted `destroys(f)` names, which the compiler
  generates.
* `[FFI-33b]` *(changed in 0.9.15)* `returns_owned` results and callbacks may carry
  `threads(creator)`: an `owns T` foreign value of such a type records its creating thread and, in
  every profile, stops the program when it ends on another, naming the type, the line that made it and
  both threads — the rule that makes GPU, GL-context and COM handles safe to hold in ordinary values.
  A callback's thread is the trampoline's to check (`[FFI-53]`).
* `[FFI-33a]` *(changed in 0.9.15)* Attaching a thread (`[FFI-22]`) gives it no right to touch another
  thread's values. What may go to another thread is decided when compiling (`[THR-8]`, `[THR-9]`,
  `E7008`, `E7015`): no one-thread owner reaches C (`[FFI-31b]`). The run-time thread checks of
  `[FFI-33b]`, `[FFI-33c]` and `[FFI-53]` back the thread contracts asserted for foreign code, and run
  in every profile (`[PRF-1]`).

## XVI.7 Exporting Ember to C

```ember
# SAFETY: the host defines no other symbol named `game_on_update`
@unsafe(export("game_on_update"))
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32:
    return 0
```

* `[FFI-26]` *(changed in 0.9.10)* *(changed in 0.9.15)* `@unsafe(export("symbol"))` gives a function a stable C symbol with the C ABI
  (without a name, the function's own name). `@unsafe(export_table("Name", protocol=N))` on a struct of
  `extern "C" fn` fields exports a constant of that
  struct, a table of function pointers, under the C symbol `Name_vN` (`[ABI-1]`), for hosts that load a
  module through one entry point. A field written with a default, `on_create: extern "C" fn(u64) -> i32
  = door_created`, holds the function its default names; a field without one holds the module's
  function of the field's own name. Either way the function's type is the field's (`E2020`), and a
  field naming no function is `E1010`. A C symbol is one name for the whole process: one that the C
  library, the runtime or another library already defines would be replaced by this function
  everywhere, and the compiler cannot see every name the process links. So both attributes are written
  in their unsafe form, `@unsafe(export("game_on_update"))` and
  `@unsafe(export_table("Name", protocol=N))`, with a `# SAFETY:` note above saying why nothing else
  defines that name (`W3012` when missing); the plain form is `E3100`: "`@export("game_on_update")`
  gives `on_update` a C name the whole process shares, and the compiler cannot see whether the host or
  a library already defines it; say that you vouch: write `@unsafe(export("game_on_update"))` with a
  `# SAFETY:` note above it". A name the compiler knows is taken is refused even so (`E0104`): `main`,
  a name beginning with the runtime's prefix (`[RT-5]`), the module-protocol names of `[FFI-28]`, and
  the functions of the C standard library ("`realloc` is a function of the C standard library, and
  exporting `same_block` under its name would replace it for the whole process, the runtime's own
  allocations included; choose another name"); so is a symbol two Ember functions define. `ember build
  --emit-header` writes `<package>.h` with a prototype for each export, a typedef for each
  `@layout(c)` type they use, and each export's thread contract as a comment.
* `[FFI-25]` *(changed in 0.9.9)* *(changed in 0.9.15)* A panic in an exported function, or anywhere below it, aborts the
  process (`[PAN-1]`); no panic unwinds into C. `on_panic=abort` is the only value the export
  attribute takes for it in this version.
* `[FFI-31b]` *(changed in 0.9.15)* No owning Ember value crosses a module boundary: class objects,
  `owns T`, `shares T`, `thread_shared T`, `weakref T`, `String`, `Array`, `Map`, function values and
  any type with a `drop` MUST NOT appear in an exported signature or be reachable through a pointer in
  one (`E5015`), except a `thread_shared` value of a class in a package built with the C++ header
  (`[BLD-FFI-5a]`). `@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may.
  `char` may not appear in a foreign signature, an export's or a hand declaration's, directly or in a
  `@layout(c)` aggregate passed by value or behind a pointer (`E5054`), as a range type may not
  (`[RNG-10b]`): C may pass any `uint32_t`, a surrogate included, which no `char` holds (`[UNS-4]`):
  "`char` has no C equivalent, and C may pass any `uint32_t`; take `c: u32` and make the character with
  `char.from_u32(c)`, which gives `None` for a value that is not one".
* `[FFI-31c]` With `[build] runtime = "shared"`, several Ember modules in one process share one runtime
  (hot reload requires it, `[HR-29]`), and the rule of `[FFI-31b]` still applies across their exported boundaries in
  this version.
* `[FFI-33]` *(changed in 0.9.15)* The export attribute's `threads=any | main | creator`
  (`@unsafe(export("score", threads=any))`) states which threads may call an export; the
  default is the file's `#! threads` directive (`[GRM-37]`), else `any`. Under `any`, everything the
  export reaches is checked as if it ran on any thread: every `static` it reads is `ThreadShared`, and
  a `static mut` it reaches is `guarded by` a lock it holds there (`E7012`: "`bump` may be called from
  any thread (`threads=any`, line 3), and it changes `HITS`, a module variable of the main thread
  (line 1)", whose help gives the lock's lines and the `Atomic`'s). The same holds under `creator`.
  For a static free-function export, `creator` is the thread whose successful initialization
  established that module's export contract; initialization establishes the contract, not the
  linked function address or a per-callable instance. For a static library or executable using a
  private runtime, this is the thread of the first successful `ember_rt_init`. For a `cdylib`, it
  is the thread of that module's successful `ember_module_init`, even when the runtime is shared;
  the identity is per module, not global to the shared runtime. The wrapper attaches the calling
  thread before checking the contract. A call before initialization or after shutdown panics.
  Repeated initialization while the module remains initialized does not transfer the contract;
  initialization after shutdown establishes a new creator. In every profile, `creator` checks
  that the caller is this thread and permits what is bound to that thread (`threads(creator)`
  values, `[FFI-33b]`). The generated export-header comment preserves the resolved `creator` policy,
  including when inherited from the file directive. This rule concerns static free-function
  exports only; `[FFI-33b]`'s `owns T` foreign values and the callbacks' thread contracts remain
  distinct.
* `[FFI-33c]` *(new in 0.9.9)* *(changed in 0.9.15)* Under `threads = main`, the exported wrapper checks, in every profile, that
  the calling thread is the one that initialised the module, and panics otherwise; in exchange the body
  may use thread-confined state, the module's `static mut`s among it. *(changed in 0.9.10)* A field of an `@export_table` takes the thread
  contract of the function it holds (`[FFI-33]`); the field itself carries none.
* `[FFI-27]` The runtime `ember_rt` is a C11 static library with no global constructors. A C host calls
  `ember_rt_init(&cfg)` once (optionally routing allocation and logging to its own functions),
  `ember_rt_thread_attach()` on each thread that calls Ember, and `ember_rt_shutdown()` at the end,
  which runs at-exit hooks and, in `debug`, prints the leak report (`[WK-15]`).
* `[FFI-28]` *(changed in 0.9.10)* A package built as `kind = "staticlib"` produces a library and
  header to link into a C or C++ program; `kind = "cdylib"` a shared library exporting only its
  `@export` symbols, its `@export_table`s, `ember_module_init`, `ember_module_shutdown`, and the
  marker `ember_module_protocol_vN`, `N` the module protocol's version (`[ABI-1]`), and the functions
  its header declares for the `thread_shared` class handles its exports give (`[BLD-FFI-5a]`).
* `[FFI-31]` *(changed in 0.9.10)* A `cdylib` links its own copy of the runtime. Before calling into it
  the host looks up the marker of the module protocol version it was built for and refuses a module
  without it; it calls `ember_module_init` before using the module and `ember_module_shutdown` before
  unloading it; shutdown runs at-exit hooks, detaches
  every thread the module attached, releases its thread-local state and reports leaks in `debug`.
* `[FFI-31a]` Its thread detachment never relies on a TLS destructor in the module's code.

## XVI.8 Build integration

* `[BLD-FFI-1]` *(changed in 0.9.15)* The manifest's `[c]` section names the C compiler and flags used
  for shims and C source files (`unsafe import c "./bridge.c"` compiles `bridge.c` and imports
  `bridge.h`, `[FFI-51]`). By default they are the compiler Ember uses as its backend.
* `[BLD-FFI-1a]` Each `import c` is parsed and its shim compiled with the package's C flags plus the
  import's own `defines` and `include_paths`, and no others.
* `[BLD-FFI-2]` *(changed in 0.9.9)* The toolchain ships a CMake module: `ember_add_library(name KIND
  staticlib|cdylib SOURCES …)` and `ember_add_executable(…)` build Ember packages as ordinary targets of
  a C or C++ project, taking the target's compiler and flags with `--cc-flags-from-target <target>` or,
  for C++, through the CMake File API (Annex C).
* `[BLD-FFI-3]` *(changed in 0.9.15)* Existing libraries are linked by name (`link = ["vulkan-1"]`) or
  path; link order follows declaration order. Each is listed by `ember tcb` as an external library
  (`[TCB-4]`).
