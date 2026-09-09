# Part XVI — Foreign Function Interface

FFI quality decides whether Ember is usable for RageV at all. This part is therefore specified to the level of the generated code.

## XVI.1 Principles

* `[FFI-1]` **Every foreign declaration has a contract**: for each pointer-typed parameter and return, the binding records ownership (`borrowed | owned | retained`), mutability, nullability, and for the function as a whole: effects, thread rules, and whether it can call back into Ember. The generator derives what it can from the C declaration (`const T*` ⇒ borrowed immutable; `T*` ⇒ borrowed mutable; return `T*` ⇒ **unknown**) and marks everything else `unknown`.
* `[FFI-2]` Calling a foreign function whose contract contains any `unknown` requires an `unsafe` block. Supplying the missing facts through an **overlay** (§4) makes the call safe.
* `[FFI-3]` The compiler never links against C++ mangled symbols. All C++ access goes through generated `extern "C"` thunks compiled by the project's own C++ compiler (§7).
* `[FFI-4]` No Ember panic and no C++ exception crosses an ABI boundary (§9).
* `[FFI-5]` Layout is verified, not assumed: the generator records `sizeof`/`alignof`/field offsets for every imported struct as reported by Clang for the *project's* target and flags; the Ember compiler asserts its own computed layout matches (`E5001` with both numbers otherwise).

## XVI.2 Importing C

```ember
import c "vulkan/vulkan.h" with (
    include_paths = ["RageV/vendor/Vulkan-Headers/include"],
    defines       = ["VK_NO_PROTOTYPES", "VK_USE_PLATFORM_WIN32_KHR"],
    link          = [],                        # volk loads at runtime; nothing to link
    overlay       = "overlays/vulkan.em")      # contracts, see §4
import c "./physics_bridge.c"                   # a source file: compiled by the toolchain as a C target and its
                                                # matching header (same stem) imported
```

Pipeline (`[FFI-6]`):

```
header + flags ──► libclang (clang-sys) parse with the project's target triple, language standard, defines,
                   MSVC compatibility mode on Windows (-fms-compatibility -fms-extensions, same _MSC_VER as the
                   configured MSVC), include paths
              ──► Clang AST walk: functions, structs, unions, enums, typedefs, global variables,
                   object-like macros per `[FFI-6]`, function-like macros ignored (W5001)
              ──► Binding IR (BIR): a language-neutral description with layouts computed by Clang
              ──► overlay applied (contracts, renames, hides, wrappers, and the function-like
                   macros `[FFI-6b]` exposes by declaring a signature)
              ──► serialised to .embind (§5), cached under target/<triple>/bind/
              ──► Ember compiler reads .embind as a synthetic module `c` (or the name given by `as`)
```

`[FFI-7]` The header is re-parsed only when its content hash, the flags, or the overlay change.

### Type mapping (`[FFI-8]`)

| C | Ember |
|---|---|
| `void` | `void` (return) / `*void` (pointer) |
| `_Bool`/`bool` | `bool` |
| `char` | `c_char` (= `i8` or `u8` per target; `cstr` when `const char*` with overlay `@ffi(string)`) |
| `signed char`, `short`, `int`, `long`, `long long` | `i8`, `i16`, `c_int`(=`i32`), `c_long` (target), `i64` |
| unsigned variants | `u8`, `u16`, `c_uint`, `c_ulong`, `u64` |
| `size_t`/`ptrdiff_t`/`intptr_t`/`uintptr_t` | `usize`/`isize`/`isize`/`usize` |
| `float`/`double` | `f32`/`f64` |
| `T*` | `*mut T` (raw) by default; via contract: `ref mut T`, `MutSpan[T]` (+ length param), `Option[ref mut T]` (nullable), `ForeignBox[T]` (owned) |
| `const T*` | `*T`; via contract: `ref T`, `Span[T]`, `Option[ref T]`, `cstr` |
| `T[N]` field | `[T; N]` |
| `struct S` (complete) | `@layout(c) struct S` with fields; `Copy` iff all fields POD |
| `struct S` (opaque/incomplete) | `extern type S` (unsized, only behind pointers) |
| `union U` | `@layout(c) union U` (v1: fields accessible only in `unsafe`; `union` keyword is reserved and the generator emits it) |
| `enum E` | `@repr(c_int) enum E` (or the fixed underlying type) with all enumerators; non-exhaustive: `E.from_repr` never fails → `unknown` variant is added `@non_exhaustive` |
| `typedef` | `type` alias (or newtype if `@ffi(newtype)`) |
| function pointer | `extern "C" fn(...) -> R` (nullable ⇒ `Option[extern "C" fn…]`) |
| bit-fields | accessor methods `get_x`/`set_x` on the struct; the field itself is not addressable |
| variadic `...` | callable only inside `unsafe` with explicit argument types; `printf`-style functions get a `@ffi(format=1)` overlay to type-check literals (v2) |
| `#define N 42` | `const N: c_int = 42` (typed by literal suffix rules of C) |
| `#define S "x"` | `const S: cstr = c"x"` |
| `static const` globals | `extern static` (read requires `unsafe` unless `@ffi(immutable)`) |
| `__attribute__((packed))`, `#pragma pack` | `@packed`/`@align` reproduced |
| `restrict`, `volatile` | `volatile` pointers become `Volatile[*T]` with `read_volatile`/`write_volatile` |
| `wchar_t` | `c_wchar` (`u16` on Windows, `i32` on SysV — **`wcstr` is therefore not portable and this specification says so out loud**); `const wchar_t*` + `string` ⇒ `wcstr`, with `std.ffi` gaining `WString`, `str.to_wstring()` and `wcstr.to_string() -> Result[String, Utf16Error]` |
| `char16_t` / `char32_t` | `u16` / `char` |
| anonymous `struct`/`union` member | a generated nested type `<Parent>_anon<N>` **plus transparent field access**, so `p.LowPart` resolves through anonymous members exactly as in C. The generated name is stable across runs and is printed by `--explain` |
| flexible array member | the struct is unsized and MUST NOT be declared, copied, or passed or returned by value (`E5017`); the importer generates `unsafe fn tail(self, n: usize) -> Span[T]` and `tail_mut` |
| `va_list` | `VaList`: opaque, `!Copy`, `!Send`, `!Sync`, forwardable only to another C function taking `va_list`, inside `unsafe`. Constructing one is `E5018` |
| `long double`, `_Float128` | `c_longdouble`, an opaque byte blob with the target's size and alignment; no arithmetic and no literals |
| `_Atomic T` | `Atomic[T]` when the layouts match, else `E5016` naming both |
| `__int128` | `i128` / `u128` |
| function pointer, non-default convention | `extern "<conv>" fn(…)` per `[FFI-9]`; an unsupported convention is `W5002` with the declaration skipped |
| `enum` with no fixed underlying type | `@repr(c_int)` unless an enumerator exceeds `INT_MAX`, in which case `@repr(c_uint)` — **the chosen repr is recorded in the `.embind` and asserted by `[FFI-5a]`, because the choice is implementation-defined and MSVC and GCC do not always agree** |
| C identifier colliding with an Ember keyword | `r#name` (`[LEX-14a]`) |

### Calling convention

`[FFI-9]` `extern "C"` uses the platform C ABI (SysV AMD64, Windows x64, AAPCS64). `extern "system"` = `stdcall` on 32-bit Windows, `C` elsewhere. `extern "vectorcall"`, `extern "fastcall"` supported on x86. Struct passing by value follows the ABI exactly (the C backend gets this for free; the LLVM backend implements the classification rules in `ember_abi`).

## XVI.3 Manual declarations

```ember
unsafe extern "C":
    fn calculate_damage(base: f32, mult: f32) -> f32
    @ffi(link_name="vkCreateBuffer_volk")
    static vkCreateBuffer: Option[extern "C" fn(*VkDevice, *VkBufferCreateInfo, *VkAllocationCallbacks, *mut VkBuffer) -> VkResult]
```

`[FFI-10]` Manual `extern` blocks are always `unsafe` to declare (the programmer asserts the signature) and each function is `unsafe fn` to call unless annotated with a complete `@ffi` contract, in which case the compiler generates the same safe wrapper it would from an overlay.

## XVI.4 Overlays: contracts without editing headers

An overlay is an Ember file that annotates imported declarations by name. It is the mechanism for third-party headers (Vulkan, GLFW, Jolt's C API, ImGui's cimgui) whose sources cannot be modified.

```ember
## overlays/vulkan.em
overlay c "vulkan/vulkan.h":

    ## Opaque handles: typedef struct VkBuffer_T* VkBuffer  →  a Copy handle newtype
    @ffi(handle)            type VkBuffer
    @ffi(handle)            type VkDevice

    ## Status codes: functions returning VkResult become Result[.., VkError]
    @ffi(status, ok=VK_SUCCESS)  enum VkResult

    ## Contracts on a function: parameter names refer to the C declaration's parameters
    @ffi(effects=[FFI, Alloc])
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed ref VkBufferCreateInfo,
                      pAllocator: nullable borrowed, pBuffer: out) -> status

    @ffi(effects=[FFI], destroys=pBuffer_of(vkCreateBuffer))
    fn vkDestroyBuffer(device: borrowed, buffer: owned, pAllocator: nullable borrowed)

    ## Pointer + length pair → Span
    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount: len_of(pViewports),
                        pViewports: span)

    ## Callback registration that retains the user pointer
    fn vkSetDebugUtilsObjectNameEXT(...): unsafe            # leave unsafe; too irregular

    ## Rename / hide
    rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction                                  # do not expose
```

Contract vocabulary (`[FFI-11]`):

| Contract | Meaning / generated Ember type |
|---|---|
| `borrowed` | pointer valid for the call only; `ref T` / `ref mut T` (mutability from `const`) |
| `nullable` | `Option[ref T]` |
| `owned` | callee takes ownership; Ember passes a `ForeignBox[T]`/handle by move |
| `returns_owned(destructor=f)` | return value is owned by Ember; wrapped in `ForeignBox[T]` whose `drop` calls `f` |
| `retained` | callee stores the pointer; the argument must be `Retained[T]` (pins an Ember-owned object; §8) |
| `out` | write-only out-parameter; becomes a return value (tuple/`Result` payload) |
| `span`, `len_of(p)` | pointer + length pair → `Span[T]`/`MutSpan[T]` |
| `string` | `const char*` → `cstr` (NUL-terminated, borrowed) |
| `handle` | opaque pointer typedef → `Copy` newtype with `NULL` |
| `status, ok=X` | return code enum → `Result[…, XError]` |
| `effects=[…]` | effect set (`[EFF-3]`) |
| `threads=any \| main \| creator` | which thread may call |
| `callback=borrowed \| retained \| once` | function-pointer parameter lifetime (§8) |
| `destroys=<param>` | documents pairing; used by the leak checker |
| `unsafe` | leave the function unsafe to call |

`[FFI-12]` An overlay declaration whose C signature does not match the header (wrong parameter count/names/types) is `E5010`, so overlays cannot drift silently. `[FFI-13]` Overlays may also add Ember-side **wrapper methods** (`extend VkDevice: fn create_buffer(self, info: BufferCreateInfo) -> Result[VkBuffer, VkError]: ...`) written in ordinary Ember, which is how idiomatic bindings are layered on top of raw ones.

## XVI.5 The `.embind` artefact

A `.embind` file is a CBOR document (schema versioned; `embind_version = 1`) containing: the source header path and content hash; the effective Clang configuration (target triple, `-std`, defines, include paths, MSVC version); the BIR (all declarations with computed layouts); the overlay hash; the contract for every declaration; C++ thunk list (§7). `[FFI-14]` `ember bind --emit-embind header.h` writes it; `ember bind --explain header.h::fn` prints the derived contract and the reasons. The file is a build input like any other and is content-addressed in the build cache.

## XVI.6 Strings and status codes at the boundary

* `cstr` (borrowed NUL-terminated) and `CString` (owned, NUL-terminated, allocates once). `str.to_cstring()` allocates; `cstr.to_str() -> Result[str, Utf8Error]` validates without copying. `[FFI-15]` Passing a `str` to a `cstr` parameter is a compile error (`E5020`, "not NUL-terminated") — the fix-it is `.to_cstring()`, or `c"literal"`.
* Status codes: `[FFI-16]` an `@ffi(status)` enum `E` generates `struct EError(code: E)` implementing `Error`, and every function returning `E` returns `Result[T, EError]` where `T` is the tuple of `out` parameters (or `void`).

## XVI.7 Importing C++

```ember
import cpp "RageV/src/RageV/Renderer/RHI/RHIDevice.h" with (
    project = "ragev",                       # takes flags from [cpp.ragev] in ember.toml (compiler, standard, defines, includes)
    classes = ["RageV::RHIDevice", "RageV::RHICommandList"],
    instantiate = ["std::vector<float>", "RageV::Handle<RageV::Texture>"],
    overlay = "overlays/rhi.em")
```

`[FFI-17]` The C++ importer produces, for each requested class/function, what the
numbered list below describes. **That list is a reader's summary and is
`NON-NORMATIVE` under `[CAT-1]`**: where it and a rule disagree, the rule governs,
and the rule is named in each item. This demotion is not cosmetic — the list has
been the site of four contradictions with the rules beside it (`std::function`,
`std::string_view`, C++ inheritance, and the CRT device attributed to `[FFI-30]`),
because a prose restatement of a rule drifts from it and nothing detects that. A
future revision should delete from the list every claim a rule already makes rather
than keep two copies in step.

1. A **thunk file** `target/bind/<hash>_thunks.cpp` containing `extern "C"` functions with predictable names (`em_cpp_<ns>_<class>_<method>_<sig-hash>`) that (a) call the C++ member/free function, (b) catch all exceptions and return a status + message buffer (§9), (c) never return C++ objects by value across the boundary — objects are heap-allocated (`new`) and returned as opaque owned pointers, or written into caller-provided storage when trivially copyable.
2. Ember declarations in the synthetic module: an `extern type RHIDevice`, a `struct RHIDeviceRef`/`ForeignBox[RHIDevice]` pairing, and methods on them that call the thunks. Constructors become `RHIDevice.new(...) -> ForeignBox[RHIDevice]`; the destructor is the box's `drop`. Overloads are disambiguated by suffixing parameter types (`draw_indexed_u32`) unless the overlay names them.
3. Templates are only available as **explicit instantiations** listed in `instantiate` or referenced by an imported signature; each instantiation gets its own thunks. `std::vector<T>` maps to `CppVector[T]` with `.span()`/`.span_mut()` (zero-copy, borrowing the vector), `.to_array()` (copy), `push_back`, `len`; `std::string` to `CppString` with `.to_str() -> Result[str, Utf8Error]`; `std::string_view` to `Span[u8]` (**not** `str` — `[TXT-1]`/`[TXT-2]` govern, and a `string_view` carries no UTF-8 guarantee); `std::unique_ptr<T>` to `ForeignBox[T]`; `std::shared_ptr<T>` to `CppShared[T]` (calls the C++ control block through thunks); `std::optional<T>` to `Option[T]` for trivially copyable `T`; `std::function` is not importable (`E5030`) — use a C callback.
4. **Compiler matching** `[FFI-18]`: the thunks are compiled by the *same* compiler, standard and flags as the project (`[cpp.ragev]` section), so they share the C++ ABI with the engine (MSVC ABI when the engine is built with MSVC; clang-cl also targets the MSVC ABI). libclang parses the headers in MSVC-compatibility mode with the same `_MSC_VER`. `[FFI-19]` If the project's compiler is MSVC and libclang cannot parse a header (MSVC-specific extension), the importer reports the exact diagnostic and the declaration is skipped with `W5031`; the programmer then binds it manually through a small C shim.
5. Inheritance and virtual functions: a C++ class hierarchy is imported as opaque types with **upcast thunks**, and **`[FFI-39]` governs subclassing, which v1 supports in a bounded form** — a single foreign base declared `@ffi(trampoline, virtuals=[…])`, an enumerated virtual set, a generated trampoline subclass, `super.init` selecting a base constructor, declared ownership in either direction, derived-first destruction, and a panic boundary on the inbound path. Multiple inheritance, virtual bases and overriding a virtual not named in `virtuals=[…]` remain unsupported (`E5056`, `[FFI-48]`). An earlier draft of this item said Ember cannot subclass C++ classes in v1 and dated the trampoline to v2; that predates `[FFI-39]` and is withdrawn.
6. Namespaces map to Ember module paths under the import name: `cpp.RageV.RHIDevice`.
7. `constexpr`/`const` integral constants and unscoped/scoped enums import like C.


### Standard-library mapping and its order (`[FFI-17a]`)

* `[FFI-17a]` The importer supports the standard-library types below. **P0 types MUST be supported before
  any P1 type, and P1 before any P2**, because a header that uses a P0 type is not usable at all until that
  type maps — `std::span` and `std::string_view` appear in the signature of almost every modern C++ API,
  so an importer that handles `std::vector` first still cannot import the header. A type outside this table
  is opaque and reachable only behind a pointer, reported by `[FFI-20]`.

| C++ type | Priority | Ember mapping |
|---|---|---|
| `std::span<T>` / `std::span<const T>` | **P0** | `MutSpan[T]` / `Span[T]`, region from `[FFI-11d]`'s `from =`. Call-scoped in parameter position; in result position `from =` is required as for any returned view. Crosses a thunk as pointer-plus-length |
| `std::string_view` | **P0** | `Span[u8]`, region from `from =`; `[TXT-2]` supersedes the 0.7.1 mapping to `str`, because a `string_view` carries no UTF-8 guarantee and `[UNS-4]` lets safe code assume one. `.to_str()` is the fallible conversion. **Not NUL-terminated** — the importer MUST NOT pass it where a `cstr` is expected |
| `std::unique_ptr<T>` | P1 | `ForeignBox[T]`; the deleter must be the default or named by the overlay, and its drop calls that deleter through a thunk |
| `std::optional<T>` | P1 | `Option[T]` where `T` maps, by value across the thunk. `std::nullopt` is `None` |
| `std::vector<T>` | P1 | `CppVector[T]` with `.span()`/`.span_mut()` (zero-copy, borrowing the vector), `.to_array()` (copy), `push_back`, `len`. **Never** a layout assumption |
| `std::string` | P1 | `CppString` with `.as_str()`; conversion to `String` is an explicit copy |
| `std::shared_ptr<T>` | P2 | `CppShared[T]`, a foreign handle over the C++ control block. **It is not `Shared[T]`** — Ember's `Shared[T]` uses Ember's own count, and conflating the two would double-free |
| `std::weak_ptr<T>` | P2 | `CppWeak[T]`, with `upgrade() -> Option[CppShared[T]]` calling `lock()` through a thunk. **Not `Weak[T]`**, for the reason above |
| `std::variant<Ts...>` | P2 | an Ember `enum` generated from the alternatives, provided every alternative maps and the overlay names each one. Otherwise opaque |

* `[FFI-17b]` **Templates are available only as explicit instantiations.** A template type or function is
  importable when it is listed in `instantiate` or appears in an imported signature, so that libclang can
  supply a concrete instantiation whose layout and mangled name are known. Passing an Ember generic into a
  C++ template is out of scope for v1 (`E5055`): it would require Ember to reproduce C++ overload resolution
  and template instantiation, which Part 0's C-backend decision exists to avoid.

`[FFI-20]` Everything an importer cannot represent is reported once with the reason (`ember bind --report`), never silently dropped.

## XVI.7a Grading, adoption and instrumentation

XVI.2 to XVI.7 specify how a foreign declaration is imported and what an overlay
may say about it. This section is about **how much any of it is worth**: in 0.5 a
fact the importer derived from a header and a promise somebody typed into an
overlay are written the same way and are indistinguishable afterwards.

```ember
overlay cpp "RageV/VulkanBackend.hpp":

    ## a grade per fact; anything ungraded is `asserted`
    @ffi(effects=[FFI] @instrumented, threads=main @checked)
    fn submit(self: borrowed, cmd: borrowed) -> status
```

* `[FFI-34]` **Every fact in an overlay carries a grade** from `[TCB-1]`, written
  after it and defaulting to `asserted`. A grade above `asserted` requires its
  evidence: `checked` requires the fact to follow from the header, the ABI or the
  build configuration; `instrumented` requires a run under `[CLI-14]` to have
  observed it holding; `proven` requires the adapter itself to have been
  analysed. Claiming a grade whose evidence is absent is `E5050`. Replace "Claiming a grade whose evidence is absent is `E5050`" with: "Claiming a grade whose evidence is absent or stale is `W5050 unbacked grade`: the fact is **reported and used at grade `asserted`**, the report names the missing or stale record and the command that would produce it, and the claimed grade is never honoured. `E5050` is raised instead of the warning under `ember build --require-evidence`, `ember tcb --require` and `ember audit --require`, which is where a project that wants the gate puts it. This is a **reporting strictness**, not an input to acceptance: `[PRF-1]` and `[EFF-15]` forbid a build from accepting or rejecting a program on the strength of an artefact that may or may not exist on this machine."
* `[FFI-35]` **A foreign pointer or reference in return position imports unsafe.** A `T&`, `const T&`, `T*` or `extern type` handle that a foreign function **returns**, or writes through an `out` parameter, produces a raw foreign pointer requiring an `unsafe` block to use, until an overlay supplies both a lifetime fact (`[FFI-11d]`) and an aliasing fact (`[FFI-11e]`). A header does not record how long a returned reference lives or who else may hold it, and mapping one to a safe borrow on the strength of its spelling is how a dangling pointer acquires the borrow checker's endorsement. In that position `borrowed` becomes an unknown-filling contract and joins `[FFI-2a]`'s list, so an overlay that promotes a returned pointer MUST be an `unsafe overlay` (`[TIER-1]`). A pointer or reference in **parameter** position is unchanged: `[FFI-2a]`'s and `[FFI-32b]`'s derivations stand, `borrowed` remains a derived fact requiring no `unsafe`, and a callee that retains the pointer beyond the call is described by `retained` as it always was (`E5051`). `const T&` in parameter position maps to borrowed mode as stated; in return position `[FFI-35]` applies.
* `[FFI-36]` **Ownership transfer is explicit where the interface does not record it.** A foreign call returning a bare `*T`, `*mut T` or `extern type` handle **whose overlay carries no ownership contract** yields that raw representation; it becomes an owned Ember value only through `unsafe adopt(handle)`, which produces `ForeignBox[T]`. `adopt` is an `unsafe` operation under `[TIER-1]` boundary (1) and introduces no fourth boundary. Where the interface *does* record the transfer, the importer produces `ForeignBox[T]` directly and no `adopt` is written: (a) an overlay declaration carrying `returns_owned(destructor=f)` (`[FFI-11]`); (b) a C++ signature returning `std::unique_ptr<T>` (`[FFI-17]`.3); (c) an imported C++ constructor (`[FFI-17]`.2). In cases (a)–(c) the human or the C++ type system has already written the transfer down and a second token adds no fact the compiler lacks. `ForeignBox[T]` remains the sole standard owned representation for a foreign object; there is no `Foreign[T]` type.
* `[FFI-37]` **Declared effects are checked, not believed.** A run under
  `[CLI-14]` links a shim recording allocation, blocking, locking and I/O across
  every foreign call and reports each declared effect fact as confirmed or
  contradicted. A contradicted fact fails the run and names the overlay line
  (`E5053`). This is what moves "submit does not allocate" from a promise to a
  measurement, and it is the only route to the `instrumented` grade.
* `[FFI-38]` The importer MUST reject rather than guess. Where ownership,
  nullability, lifetime or exception behaviour cannot be established, the
  declaration imports unsafe and `[FFI-20]` reports why. Convenient-but-unsound
  is not an import mode. Strike "or exception behaviour" — `[FFI-24]`'s catch-all establishes it universally — and add: "Where the exception specification is dependent and unevaluated, `[FFI-24]`'s catching shape applies and `[FFI-20]` reports it."

**Which foreign code to replace first.** Not normative, and stated so that the
order is not re-argued each time. Good candidates have a clear boundary, few
dependencies, bugs that cost real time, no template machinery in the interface,
and ownership that is already understood — in the reference workload: frame
orchestration, gameplay, ECS systems, tools, editor logic, resource bookkeeping.
Poor candidates are macro-generated code, plugin ABIs, pervasive global state,
shared ownership with custom deleters, and template-heavy headers with
undocumented lifetimes — in the reference workload: the Vulkan and OpenGL
backends, the platform layer, and the allocators. Those stay native, and the
bridge around them is where the contracts and grades concentrate.


### XVI.7b C++ compatibility classification (`[FFI-44]`)

* `[FFI-44]` **What imports, and how.** This table is normative and is the answer to
  "can Ember consume this header". *Automatic* means the importer handles it with no
  overlay; *overlay* means it needs a declared contract; *native island* means it
  stays in C++ behind a hand-written boundary and no importer support is planned.

| C++ construct | v1 | Notes |
|---|---|---|
| `extern "C"` functions | automatic | Part XVI.1–6 |
| POD / value structs | automatic | `[FFI-5a]` verifies layout with the project's own compiler |
| standard-layout, trivially-copyable classes | automatic, by value | becomes an `@layout(c) struct` |
| other classes | automatic, opaque | behind a pointer; methods through thunks |
| `std::span<T>` | automatic | `Span[T]` / `MutSpan[T]`, zero copy |
| `std::string_view` | automatic, **as bytes** | `Span[u8]`; `[TXT-2]` makes the `str` conversion fallible |
| `std::string` | automatic | `CppString`; `.to_str()` is fallible |
| `std::vector<T>` | automatic | `CppVector[T]` with zero-copy `.span()` |
| `std::unique_ptr<T>` | automatic | `ForeignBox[T]`; the deleter must be default or named by the overlay |
| `std::shared_ptr<T>` / `weak_ptr<T>` | automatic | `CppShared[T]` / `CppWeak[T]` — **never** `Shared[T]`/`Weak[T]` (`[SEL-2]`, `E5065`) |
| `std::optional<T>` | automatic | `Option[T]` where `T` is trivial |
| `std::variant<…>` | automatic | generated enum, where every alternative maps |
| `std::function` | overlay | not importable as a parameter (`E5030`); importable as an opaque owned object, and `@ffi(std_function, signature=…)` generates the constructor |
| explicit template instantiation | automatic | named in `instantiate=[…]` (`[FFI-17b]`) |
| any other template | **native island** | Ember generics do not instantiate C++ templates (`E5055`) |
| object-like macros | automatic | constant-valued only |
| function-like macros | overlay | `@ffi(macro_fn)` (`[FFI-6b]`) |
| single inheritance from a foreign base | automatic, **bounded** | `@ffi(trampoline, virtuals=[…])` (`[FFI-39]`) |
| multiple inheritance, virtual bases | **native island** | pointer-adjustment thunks are ABI-specific (`[FFI-48]`) |
| overriding a virtual not in `virtuals=[…]` | rejected | `E5056` |
| exceptions | automatic, **declared** | `[FFI-43]`: `throws = "translate"` or `"noexcept"`, no default |
| custom allocators | overlay or native island | `[FFI-37f]` may report the fact as structurally unobservable |
| C++ metaprogramming, concepts, compiler extensions | **native island** | no importer support planned |
| ownership the header does not state | unsafe | `adopt` (`[FFI-36a]`), and the fact is `asserted` in `ember tcb` |

* `[FFI-48]` **Unsupported C++ constructs**, each with the reason it is unsupported
  and what to do instead. This list is normative; `[FFI-17]` item 14 is the
  non-normative summary of it. Encountering one of these in an imported header is
  `E5034`, naming the construct and its row.

| Construct | Why not | Instead |
|---|---|---|
| multiple inheritance | the `this`-adjustment thunk depends on the ABI's vtable layout and differs between MSVC and Itanium; getting it wrong is a silent wrong-object call | expose a single-inheritance facade in C++, import that |
| virtual base classes | the virtual-base offset table is ABI-private and not discoverable from the AST | as above |
| overriding a virtual not named in `virtuals=[…]` | the trampoline is generated from that list; an unnamed virtual has no override slot | name it in `virtuals=[…]` (`E5056`) |
| C++20 modules | libclang's module support does not expose a stable AST for a compiled module interface | parse the headers |
| exceptions propagating *through* Ember frames | Ember has no unwinder in v1 (`[PAN-1]`), so a frame crossed by an exception cannot run its drops | `[FFI-43]`'s `throws = "translate"` converts at the boundary |
| C++ coroutines | the promise type, the customisation points and the frame layout are all implementation-defined | expose a callback or a completion handle from C++ |
| overloads distinguished only by return type | not expressible; Ember has no return-type overload resolution (`[FN-*]`) | rename one in the overlay (`[FFI-13]`) |
| ABI depending on RTTI beyond the type's own vtable | `dynamic_cast` across a hierarchy the importer did not model has no Ember equivalent | do the cast in C++ and export the result |
| non-type template parameters of class type | the mangling is unstable across the supported compilers | instantiate explicitly in C++ and export a typedef |
| allocator-parameterised containers | the allocator is part of the type's identity and its behaviour is not inspectable | expose the container's span, or keep it native |
| `std::function` as a parameter | a `std::function` is constructed from a callable whose type Ember cannot name (`E5030`) | take it as an opaque owned object, or `@ffi(std_function, signature=…)` |
| compiler extensions (`__declspec`, `__attribute__` beyond layout) | there is no portable meaning to import | wrap in C++ behind a plain signature |
| anything reached only through template metaprogramming | Ember does not instantiate C++ templates (`[FFI-17b]`, `E5055`) | name the instantiation in `instantiate=[…]`, or keep it native |

## XX.13 The C++ importer corpus and migration gate

A C++ importer that has only ever met headers written to exercise it is not
evidence. This section is what turns the design of Part XVI into something
falsifiable.

* `[CXX-1]` **The corpus.** `tests/cxx-corpus/` contains, at minimum: a trivial
  class, a class with a non-trivial destructor, one with virtuals, one with a
  `mutable` member, one derived from another; each of `T*`, `T&`, `const T&`,
  `unique_ptr`, `shared_ptr`, `weak_ptr` in parameter and return position;
  `vector`, `span<T>`, `span<const T>`, `string`, `string_view`, `optional`,
  `variant`; `template<class T> T identity(T)` and `template<class T> struct Box`
  with an explicit instantiation, an unsupported generic use, a nested
  instantiation and a dependent layout; `noexcept`, `noexcept(false)` and
  `noexcept(expr)`; and the eight inheritance cases of `[FFI-39]` including a
  missing virtual declaration, a non-virtual destructor, both ownership modes and
  a re-entrant callback.
* `[CXX-2]` **Real headers, not only fixtures.** The corpus additionally binds a
  representative set of RageV's own headers and at least one third-party
  header-only library (GLM or EnTT), because a fixture the importer's author wrote
  tests the importer against its own assumptions.
* `[CXX-3]` **Every corpus entry has an expected outcome recorded**: imported as
  value, imported as opaque, imported behind an overlay, or refused with a named
  diagnostic. An entry whose outcome is "it worked" without saying which is not a
  test.
* `[CXX-4]` **Golden thunks.** The generated C++ thunk source for each entry is
  committed and diffed, so a change in what the importer emits is visible in review
  rather than discovered by a linker.
* `[CXX-5]` **The corpus runs under every supported configuration**: MSVC and
  clang-cl, debug and release CRT, RTTI on and off, `/Zc` conformance settings, and
  both `_ITERATOR_DEBUG_LEVEL` values. Where two configurations disagree the
  importer MUST fail deterministically with the diagnostic `[BLD-FFI-1b]` requires,
  never bind successfully and differ at runtime.
* `[CXX-6]` **The C++ migration gate**, which `[GATE-4]` incorporates by reference
  and which is met when all of the following hold on the corpus of `[CXX-1]`
  and `[CXX-2]`:
  1. every supported entry imports deterministically — the same input produces the
     same binding, twice, on both compilers;
  2. zero silent declarations: every entity is imported, refused with a
     diagnostic, or listed as skipped with a reason;
  3. zero accepted layout mismatches — `[FFI-5a]`'s asserts hold on every entry;
  4. zero ownership contracts inferred from evidence `[FFI-35]` does not admit;
  5. zero lifetime promotions without an explicit `from =` contract;
  6. zero exception-mode ambiguities — `[FFI-43]` leaves no import undeclared;
  7. MSVC and clang-cl produce the same semantic result on every entry;
  8. generated thunks compile under RageV's exact build configuration;
  9. FFI call overhead meets `[BEN-6]`.
* `[CXX-7]` **A dependency's exception mode is an API change.** If an upgraded
  header changes an imported function's `noexcept`-ness, its Ember signature
  changes under `[FFI-43]` — `Result[T, CppError]` appears or disappears — and
  `ember bind --report` MUST list it under API changes, not under notes. The
  `.embind` hash changes with it, so `[HR-23]` refuses a hot reload across it.

## XVI.8 Callbacks and foreign retention

* `[FFI-21]` A C parameter of function-pointer type accepts: a capture-free Ember `fn` (coerced to `extern "C" fn`), or an `extern "C" fn` value. Capturing closures are rejected (`E5040`) unless the API has a `void* user_data` parameter matched by the overlay contract `callback=…, user_data=<param>`, in which case the generator produces the trampoline: it boxes the closure, passes the box as `user_data`, and generates the `extern "C"` shim that unboxes and calls it.
* `callback=borrowed`: the closure is valid for the call; the box is freed after return. `callback=retained`: the API stores it; Ember returns a `Retained[Callback]` token that must be kept alive by the caller (dropping it unregisters via the paired `destroys=` function if declared, else `E5041 retained callback needs a release function in the overlay`). `callback=once`: freed by the shim after the first invocation.
* `[FFI-22]` **Foreign threads.** Any `extern "C"` function exported from Ember (§XVI.10) or any generated trampoline begins with `ember_rt_thread_attach()` (idempotent, cheap after the first call: a TLS flag) so that thread-local allocators and panic state exist. Detachment happens automatically on thread exit via a TLS destructor. Class handles must not be passed to foreign threads unless the class is `Sync` (`[THR-2]` is enforced at the trampoline's capture check).
* `[FFI-23]` `Retained[T]`: to hand a pointer to an Ember-owned object to C that will keep it, wrap it: `tok = Retained.pin(obj)` (for a class handle: bumps the strong count and sets the header `pinned` flag; for a `Box`: takes ownership into the token). `tok.ptr()` is the raw pointer. Dropping the token releases. This is the only mechanism for foreign retention of Ember-owned memory; there are no GC-style pinning handles.

## XVI.9 Errors, panics and exceptions at the boundary

* `[FFI-24]` Generated C++ thunks have the shape:
  ```cpp
  extern "C" int32_t em_cpp_X_method(X* self, Args..., EmberCppError* err) noexcept {
      try { self->method(args...); return 0; }
      catch (const std::exception& e) { ember_cpp_error_set(err, typeid(e).name(), e.what()); return 1; }
      catch (...)                     { ember_cpp_error_set(err, "unknown", ""); return 1; }
  }
  ```
  The Ember side returns `Result[T, CppError]` for every C++ call unless the overlay says `@ffi(noexcept)`, in which case the thunk is `noexcept` and a throw terminates (documented).
* `[FFI-25]` Ember functions exported to C run under a **panic boundary**: with `panic=abort` a panic aborts (as anywhere); with `panic=unwind` (v2) the exported wrapper catches, logs, and returns the declared error value (`@export(on_panic=return -1)`).

## XVI.10 Exporting Ember to C and embedding the runtime

```ember
@export("rv_script_on_update")                          # stable C symbol, C ABI
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32: pass

@export_table("RvScriptApi", protocol=3)                # a function-pointer table struct, RageV style (§Part XXII)
pub struct ScriptApi:
    on_create: extern "C" fn(u64) -> i32
    on_update: extern "C" fn(u64, f32) -> i32
```

* `[FFI-26]` `ember build --emit-header` writes `<package>.h` with prototypes for every `@export`, C typedefs for every `@layout(c)` type used in them, and `#define <PACKAGE>_PROTOCOL_VERSION n`. `--emit-header` MUST emit the thread contract of each `@export` as a doc comment on its prototype, so the host's C++ author reads the rule at the call site.
* `[FFI-27]` The Ember runtime (`ember_rt`) is a C11 static library with no global constructors. Embedding API: `ember_rt_init`/`ember_rt_shutdown` are the *process* lifecycle used by an executable or a `staticlib` host. A `cdylib` host uses `[FFI-31]` instead and MUST NOT call `ember_rt_shutdown` directly.
  ```c
  ember_rt_config cfg = ember_rt_config_default();
  cfg.alloc = my_malloc; cfg.free = my_free;            // optional: route allocations to the host allocator
  cfg.log = my_log;                                     // panic/log sink
  ember_rt_init(&cfg);                                  // once per process; idempotent
  ember_rt_thread_attach();                             // per thread that will call Ember (idempotent)
  ...                                                   // call exported functions / tables
  ember_rt_shutdown();                                  // runs at-exit hooks, leak report in debug
  ```
* `[FFI-28]` An Ember **package built as `kind = "cdylib"`** produces a DLL/.so exporting only `@export` symbols plus `ember_module_init(const ember_host_api*)` and `ember_module_protocol()`; built as `kind = "staticlib"` it produces a `.lib/.a` plus header, suitable for linking into RageV's static-library build directly.
* `[FFI-31]` **Module lifecycle for `kind = "cdylib"`.** A `cdylib` links a private copy of `ember_rt`. In addition to `[FFI-28]`'s symbols it MUST export `int ember_module_init(const ember_host_api*)` (configures and calls this module's `ember_rt_init`; idempotent) and `void ember_module_shutdown(void)`, which the host MUST call before unloading. `ember_module_shutdown` MUST run the module's at-exit hooks, detach every thread this module attached, release every thread-local allocator, arena and TLS slot the module owns, and emit the leak report in `debug` (ADR-007's report is load-bearing, and this is where a plugin's runs). After it returns, no code or data of the module may be reachable from any thread.
* `[FFI-31a]` `[FFI-22]` is amended: when the package kind is `cdylib`, automatic detachment MUST NOT be implemented with a TLS destructor whose code resides in the module. The runtime keeps an intrusive list of attached threads and detaches them in `ember_module_shutdown`.
* `[FFI-31b]` **No owning Ember value may cross a module boundary.** Because each `cdylib` owns a private allocator and type-info table, class handles, `Box`, `Shared`, `Weak`, `String`, `Array`, `Map`, and any type with drop glue MUST NOT appear in an `@export` or `@export_table` signature, nor be reachable through a pointer in one. Only `@layout(c)` value types, scalars, opaque handles, `cstr` (copied at the boundary) and `extern "C" fn` may cross. Violation is `E5015`, naming the offending type and the reason. This makes Part XXII's `[RV-2]` a language rule rather than a plan convention, and follows directly from XXIII.2's "no stable Ember-to-Ember ABI".
* `[FFI-31c]` A host that must share one runtime between several Ember modules links `ember_rt` as a shared library and builds each module with `[build] runtime = "shared"`. This is the only configuration in which `[FFI-31b]` may be relaxed, and it is **v2**; v1 packages are always privately linked.
* `[FFI-33]` **Thread contracts on exported functions.** `@export` and `@export_table` accept `threads = any | main | creator` with `[FFI-11]`'s meanings, defaulting to `any`; a `@export_table` MAY set it per field; a module MAY declare a default with `#! threads main` (which is what Part XXII §1 assumes). `threads = any`: the compiler checks the exported function's reachable call graph as if it ran on an arbitrary thread — every `static` it reaches MUST be `Sync`, and no non-`Sync` class handle may be reachable from a `static`, a parameter or a captured value; violation is `E7010` naming the reached item and why it is not `Sync`, with `#! threads main` named as the fix. `threads = main`: the exported wrapper asserts, in `debug` and `release`, that the calling thread is the one that called `ember_module_init`/`ember_rt_init`, and panics `ember_panic_thread` otherwise; in exchange the body MAY touch non-`Sync` statics and handles. `threads = creator`: as `main`, but the asserted thread is the one that created the value the call is dispatched on.
* `[FFI-33a]` `[FFI-22]` is amended: `ember_rt_thread_attach()` establishes thread-local runtime state and confers **no** right to touch thread-confined data. In the `debug` profile, under the existing `debug_objects` profile key, an extended object header records the attaching thread id, and `ember_retain`/`ember_release` on a non-`Sync` object from a different thread panics naming the class and both thread ids. **`[OBJ-1]`'s release header layout is unchanged**: the 24 bytes are fully occupied and a thread id does not fit, so this is a debug-only extension, never a change to the ABI `[VER-4]` freezes.
* `[FFI-33b]` `returns_owned(destructor=f)`, `[FFI-23]`'s `Retained[T]` and `Callback[F]` MAY carry `threads=`. A `ForeignBox[T]` whose destructor is `threads = creator` records the creating thread on construction and panics in `debug` when dropped elsewhere; this is what makes GPU, GL-context and COM handles safe to hold in ordinary Ember values. `ember bind --report` lists every foreign destructor with no thread contract. "…records the creating thread **at the point the box is formed** — the `[FFI-36]` importer-generated wrapping, or the `adopt` call — and panics in `debug` when dropped elsewhere."
* `[FFI-20a]` `[FFI-20]` applies to the **C** importer exactly as to the C++ importer. Every declaration the C importer cannot represent MUST be recorded in the `.embind` with the construct that defeated it and a suggested workaround, and reported by `ember bind --report` as `W5002 declaration not imported: <name> — <construct> — <workaround>`. No declaration is ever silently absent from a synthetic module. **A reference to a name that was skipped MUST produce a diagnostic saying it was skipped and why, not "unknown identifier"** — which requires carrying the skip list into the synthetic module's namespace as tombstones.
* `[FFI-29]` **Generated shim translation unit.** For every `import c`, the importer MUST emit `target/<triple>/bind/<hash>_shim.c`, compiled by the project's C compiler with that import's own filtered flags (`[BLD-FFI-1a]`) and linked into the package, containing: (a) for every function with internal linkage or no external definition (`static`, `static inline`, `inline`, `__forceinline`) reachable from the header, an external wrapper `<ret> em_inl_<name>(<params>) { return <name>(<args>); }` — the binding refers to `em_inl_<name>`, and `ember bind --explain` prints the wrapping; (b) the function-like macro wrappers of `[FFI-6b]`; (c) the layout assertions of `[FFI-5a]`.
* `[FFI-29a]` The importer MUST NOT emit a binding that names a symbol with internal linkage. A binding that resolves to no external symbol is a defect: it produces a link error with no Ember diagnostic and no source location.
* `[FFI-29b]` **Single-header libraries.** `import c "miniaudio.h" with (implementation = ["MINIAUDIO_IMPLEMENTATION"])` causes the toolchain to emit `target/<triple>/bind/<hash>_impl.c` containing the `#define`s followed by the `#include`, compiled with the import's flags and linked **exactly once per package**. Two packages in one build requesting the same implementation macro for the same header is `E5014`, naming both; the resolution is for one to expose the library and the other to depend on it.
* `[FFI-29c]` A wrapped call costs one non-inlined call unless the shim TU participates in LTO. `[FFI-9]`'s and M5's "ABI-direct call, zero extra instructions" property applies to externally-defined functions only; `ember inspect` MUST report a wrapped foreign function as `wrapped (static inline)` so the cost is visible. A variadic `static inline` function cannot be wrapped and falls to `W5002`.
* `[FFI-30]` **Identity of imported entities.** Two imported declarations denote the same Ember entity iff they have the same **C identity** — Clang's USR for the declaration, resolved through typedefs to the underlying tag — and the same resolved BIR layout. Identity is independent of the importing module, package, alias and overlay. Two imports of the same C identity with different resolved layouts in one build are `E5011`, listing both flag sets and the first differing field, **with help naming the route out: give one import a distinct alias, accepting that its entities are then distinct and values do not interchange.** (Compiling one library twice with different `-D` sets in one build is legitimate.)
* `[FFI-30a]` An overlay is a **view, not a type constructor.** It changes the signatures, safety and names of imported *functions* and may add wrapper items (`[FFI-13]`), but it MUST NOT change the identity, layout or field set of an imported *type*. `@ffi(handle)`, `@ffi(newtype)` and `rename` produce Ember-side aliases over the same underlying entity. Two packages may therefore carry different overlays for the same header and still exchange `VkDevice` values.
* `[FFI-30b]` `pub import c "…" as vk` re-exports the synthetic module. `[MOD-2]`'s visibility rules apply to a synthetic module exactly as to any other, so a package MAY expose imported types in its public API and a dependant reaches them as `ember_vulkan.vk.VkDevice` without re-importing the header. `[BLD-2]`'s interface hash MUST include the imported entity's layout, so a header change invalidates dependants.
* `[FFI-30c]` **Distributable, composable overlays.** A package MAY distribute an FFI overlay for an imported foreign module, and multiple overlays for the same foreign module MAY be composed in a declared left-to-right order. An overlay MAY add Ember-side aliases, wrappers, metadata, or other permitted bindings and MAY use explicit `override` where the overlay contract allows replacement. Composition MUST diagnose incompatible definitions rather than silently selecting one, and MUST NOT change the identity, layout, field set, or foreign ABI identity of an imported foreign type. The composed overlay identity MUST participate in the build/interface identity used to invalidate dependants when the overlay changes (`[BLD-2]`, `[BLD-3]`).
* `[FFI-32]` **C++ value types.** A C++ class or struct that is standard-layout and trivially copyable, and every one of whose non-static data members maps under `[FFI-8]`, MUST be imported as an Ember `@layout(c) struct` carrying `@derive(Copy)` — **not** as an `extern type`. Public data members become `pub` fields; non-public members become private fields of the same type and offset, so the Ember type is bit-identical. Every such type carries a `[FFI-5a]` layout assertion. Values of it cross thunks by value with no allocation and satisfy `[TYP-11]`. `[FFI-32a]` A class that is standard-layout but **not** trivially copyable is imported as an opaque `extern type`, as today. An overlay MAY force a value mirror with `@ffi(value) class N::C` when the programmer asserts Ember only reads its bits; the mirror is `!Drop` and constructing one is `E5032`. `[FFI-32b]` `const T&` maps to Ember `T` (borrowed mode) when `T` is a value type, and to `ref T` over the opaque type otherwise; `T&` maps to `ref mut T`/`mut` mode; a class parameter **by value** maps to `owned T` for value types and is `E5031` otherwise; `T&&` is `E5031` in v1. `[FFI-32c]` **Default arguments.** Where a C++ default argument is a constant expression, the importer MUST map it to an Ember default argument (`[FN-5]`) with the same value. Otherwise it MUST emit one thunk per arity reachable by omitting trailing defaulted parameters, exposed as Ember default arguments forwarding to the shorter thunk, so a call omitting the argument receives **C++'s own default expression evaluated on the C++ side**. Thunks MAY be emitted lazily for arities Ember actually calls. `[FFI-32d]` Template instantiations follow the same rule: an instantiation satisfying `[FFI-32]` (e.g. `RageV::Handle<RageV::Texture>`) is a value type. `[FFI-17]` item 3's STL mappings are unchanged and take precedence for the named STL types. `[FFI-32e]` `ember bind --report` MUST list, for every class in `classes = […]`, whether it imported as a value type or as opaque, and for opaque types the specific disqualifying property (`non-trivial destructor`, `virtual base`, `member of unmapped type T`), because that determines the cost of every call that touches it.
* `[FFI-11]` Every pointer-typed parameter or return that a contract makes safe MUST carry a **count** axis:
* `[FFI-11a]` `borrowed` alone no longer implies `one`. An overlay supplying ownership, mutability and nullability for a pointer parameter but no count axis is `E5012 pointer contract has no count`. The diagnostic MUST list the five count contracts and MUST name any sibling parameter whose name or type suggests a length (a `uint32_t`/`size_t` parameter, or one whose name ends `Count`, `Len`, `Size`, `N`).
* `[FFI-11b]` The **two-call enumeration idiom** has a named contract: `count: inout_count` paired with `items: span(len_of(count), nullable)`. The generated signature is `fn f(…, items: Option[MutSpan[T]]) -> Result[u32, E]`; the wrapper passes `null` and forwards the caller's count when `items` is `None`, and passes `items.as_mut_ptr()` with `items.len()` otherwise, returning the count the callee wrote or requires.
* `[FFI-2a]` Contracts the generator **derives** from the C declaration are verified facts and MUST NOT require `unsafe`: `const T*` ⇒ borrowed immutable, `T*` ⇒ borrowed mutable, struct/union layouts asserted per `[FFI-5]`, enum underlying types, `@packed`/`@align` reproduction, calling convention. Only `unknown`-filling contracts require it: `owned`, `returns_owned`, `retained`, `span`, `len_of`, `string`, `nullable`, `handle`, `status`, `threads=`, `effects=`, `noexcept`, `callback=`, and `noalias` (`[SIMD-3]`). An overlay containing none of these needs no `unsafe`. After "…calling convention", insert: "In **return position** a pointer or reference contract is unknown-filling (`[FFI-35]`): `borrowed`, `from =` and the aliasing words join the list below." `noexcept` moves out of the unknown-filling list **when derived under `[FFI-24a]`**; an overlay-written `@ffi(noexcept)` remains unknown-filling and requires `unsafe overlay`.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` are part of the **overlay language**, not merely of the tool. `[FFI-11]`'s vocabulary and `[FFI-12]`'s signature check MUST accept them, and `[FFI-2]` MUST treat a declaration carrying one as **uncontracted** — its calls stay `unsafe`. This is what lets a team adopt a header on day one, run everything behind `unsafe:`, and buy safety incrementally rather than writing 2000 lines before the first call compiles. The marker set becomes `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and **`TODO(lifetime)`**. `[FFI-2]` MUST treat a declaration carrying any of them as uncontracted.
* `[FFI-6]` **Macro import.** The importer MUST import an object-like macro whose replacement list, **after full macro expansion**, is a constant expression of integer, floating, string-literal, **null-pointer-constant, or pointer/handle-cast** type as evaluated by Clang under the import's own configuration (a probe TU of one `static const` or `enum` per candidate macro, parsed with the same configuration and cached in the `.embind`). The macro's Ember type is the C type of the evaluated expression after the usual arithmetic conversions. This covers `(~0U)`, `(-1)`, `(1 << 3)`, `sizeof(T)`, `0xffffffffffffffffULL`, `((VkBuffer)0)` — which maps to the imported handle newtype's NULL per `[FFI-11]`'s `handle` contract — and object-like macros whose bodies expand through function-like macros.
* `[FFI-6a]` A macro that does not evaluate to such a constant is skipped with `W5001 macro not imported: <reason>`, where `<reason>` is Clang's evaluation failure — never a blanket category. The macro, its body and its reason MUST appear in the `.embind` and in `ember bind --report`.
* `[FFI-6b]` A **function-like** macro MAY be exposed as a function when an overlay declares its signature (`@ffi(macro_fn) fn VK_MAKE_API_VERSION(variant: u32, …) -> u32`). The importer emits `uint32_t em_mac_VK_MAKE_API_VERSION(…) { return VK_MAKE_API_VERSION(…); }` into the generated shim translation unit (`[FFI-29]`, RFC-043) and binds `em_mac_*` as an ordinary `extern "C"` function. **This adds no macro facility to Ember: the imported entity is a C function.** Where all arguments are compile-time constants the importer MAY fold the call through `[FFI-6]`'s probe TU, so `VK_MAKE_API_VERSION(0,1,3,0)` is usable in a `const` initialiser.
* `[FFI-5a]` In addition to the libclang cross-check, the importer MUST emit into the generated shim TU (`[FFI-29]`) or thunk TU (`[FFI-17]`) a `_Static_assert`/`static_assert` for `sizeof` and `_Alignof` of every imported aggregate that Ember constructs, passes or returns by value, or embeds, and for `offsetof` of every field Ember reads or writes. **The layout contract is therefore verified by the compiler that builds the project, on the user's machine, on every build**, and a mismatch is a build error naming the type, the field and both offsets (`E5001`). XIX §10's cross-compiler tests remain and cover the compiler's own fixtures.
* `[FFI-35a]` **What the header does not say about a parameter is retention, not lifetime.** A parameter pointee's liveness across the call is established by Ember's own borrow checker, not claimed by the header. Where an overlay declares no `retained` contract for a pointer parameter, the importer MUST record the fact "`<fn>` does not retain `<param>`" once per foreign function in `[TCB-1]`'s report at grade `asserted`. `[CLI-11]` MUST group that list by foreign module. This converts the one residual hole in parameter position from unrecorded into enumerated, which is what `[TCB-1..4]` exist for.
* `[FFI-11d]` **Return-position lifetime contract.** `[FFI-11]`'s table gains `from = self | <param> | static`, valid only on a result-position pointer, reference or span. `from = self` gives the result the receiver's region under `[LT-1]` rule 1; `from = p` gives it `p`'s region exactly as `[LT-1a]`'s `@borrows(p)` would; `from = static` gives it `[LT-3]`'s static region and MUST be used only for a pointer into `static` foreign storage. Naming more than one parameter gives the intersection, as `[LT-1a]` specifies. `span(len_of(q), from = self)` is the paired form for a pointer-plus-length result. Naming a parameter that does not exist, or naming `self` on a free function, is `E5010` under `[FFI-12]`. **No region syntax enters Ember source:** the overlay names a *parameter*, exactly as the already-shipped `@borrows(a)` does, so XXIII.2's "no named lifetimes" and `[LT-6]`'s v2 reservation are untouched and no `'a` appears anywhere.
* `[FFI-11e]` **Aliasing contract.** `exclusive` — no other live reference to the referent exists for the contracted lifetime; promotes to `ref mut T`. `aliased` — other readers may exist; promotes to `ref T` only, **never `ref mut T`, whatever the `const` spelling says**. A pointer carrying a lifetime fact and no aliasing fact is `aliased`. `[SIMD-3]`'s `noalias` is unchanged and orthogonal.
* `[FFI-11f]` A `from =` fact derived by the importer from a `[[clang::lifetimebound]]` attribute on the corresponding parameter or on `this` imports at grade **`asserted`**, not `checked`: the attribute is an unenforced source annotation and `[TCB-2]` says a declaration never raises a grade. `ember bind --report` MUST list every result-position pointer with no `from =` fact, because that list is the remaining unsafe surface of an adoption.
* `[FFI-36a]` **`adopt` is a name, not a keyword.** `std.ffi` declares `unsafe fn adopt[T](h: H) -> ForeignBox[T]`, where `H` is the foreign handle representation of `T` (`*T`, `*mut T`, or an `extern type` handle). `[LEX-15]`'s reserved set stays at 48 entries; `adopt` is resolved by the ordinary rules of Part V §1 and a user item of that name shadows it, exactly as for any other `std` item. The call is **compiler-recognised**: the destructor is not an argument and is read from the overlay contract of `T`'s foreign declaration (`[FFI-11]`'s `returns_owned(destructor=f)` or the C++ destructor `[FFI-17]` records). `adopt` carries the `FFI` effect and no other.
* `[FFI-36b]` Adopting a handle whose overlay declares `adopt = false` is `E5052`, reported **statically**. Adopting the same native object twice is not statically decidable in general: in the `debug` and `release` profiles the runtime maintains an adopted-pointer set under the existing `debug_objects` profile key, and a second adoption of a live pointer panics `ember_panic_double_adopt` naming both sites; where both `adopt` calls are on the same local in one function the compiler reports `E5052` statically. `E5052` MUST NOT be documented as catching the general case.
* `[FFI-37a]` **Observation surface.** `[FFI-37]`'s shim MUST interpose a closed, published set of symbols, and the set MUST be recorded in the evidence record (`[TCB-5]`) and printed by `ember tcb`. The v1 set is: `malloc`/`calloc`/`realloc`/`free`; `operator new`/`operator delete` in all array and sized forms; the platform heap entry points (`HeapAlloc`/`HeapFree`, `mmap`/`brk`); `pthread_mutex_*`/`SRWLock*`/ `EnterCriticalSection`; the futex/`WaitOnAddress` wait primitives; and the libc/Win32 file and socket entry points. **A foreign allocation that does not pass through this set — a bump allocation from a pool the callee reserved earlier, a custom allocator statically linked into the engine — is invisible to the shim by construction.**
* `[FFI-37b]` **Coverage is part of the evidence.** The record required by `[TCB-5]` MUST additionally contain, per graded fact: the number of times the foreign entry point was invoked under instrumentation; the number of distinct Ember call sites that reached it; and, for an effect fact, the number of invocations on which the property was actually evaluated. A fact whose invocation count is zero MUST be recorded and reported as **`unexercised`**, MUST NOT satisfy an `instrumented` claim, and MUST appear in `[TCB-3]`'s assumption list at grade `asserted`. A run that did not exercise a declared fact emits `W5054 unexercised foreign fact`, naming the overlay line, the entry point and the observed invocation count. **Silence is not confirmation.**
* `[FFI-37c]` **What the grade means.** `instrumented` MUST be defined in `[TCB-1]` and rendered by `ember tcb` as: "no counterexample was observed, on the paths exercised, through the recorded observation surface". It is **never** rendered as "the fact holds". Where `[CLI-13]`'s reachable foreign call set was not fully exercised, the report renders the fact as `instrumented (partial: N of M call sites)`. Where the declared effect set is a *negative* claim and the callee is known to use an allocator outside `[FFI-37a]`'s surface — which the overlay states with `@ffi(allocator = external)` — the grade MUST NOT exceed `asserted`, and `ember tcb` MUST print the reason.
* `[FFI-37d]` **What `instrumented` may discharge.** `instrumented` is evidence from execution, not a proof, and the toolchain MUST NOT allow it to discharge a fact whose failure is a **memory-safety** failure — ownership, nullability, lifetime, aliasing, or exception escape. Those reach `proven` only through `[TCB-1]`'s fourth grade and otherwise remain `asserted`. **Effect facts** (`Alloc`, `Block`, `Lock`, `Io`) MAY be `instrumented`, because their failure mode is a violated performance contract rather than undefined behaviour.
* `[FFI-37f]` **`instrumented` is not one thing.** A fact graded `instrumented` MUST
  additionally carry how completely the run observed it: **observed** (every
  allocation, free or call on the path passed through the interception point),
  **partially observed** (some did and the toolchain can say which did not),
  **unobserved** (the path did not execute in the run), or **structurally
  unavailable** (the mechanism cannot be intercepted at all — a custom pool, an
  arena reserved before interception was installed, a statically linked allocator,
  a VMA or driver allocation). `ember tcb` prints the label beside the grade, and
  **structurally unavailable is reported as evidence of a gap, not as evidence**:
  it means the run could not have falsified the claim, which `[PHIL-5]` forbids
  treating as support for it.
* `[FFI-37e]` **Configuration and cost.** The evidence record MUST include the C++ build configuration of the instrumented run — the `[cpp.<project>]` flag set of `[BLD-FFI-1]` and the values `[BLD-FFI-1b]` resolves — and it is an **identity** input under `[TCB-6]` as amended by FIX-020: an `instrumented` fact measured against a `debug` C++ build MUST NOT satisfy a claim in a build linking the `release` engine, and `ember tcb` names both. The shim is linked only by `[CLI-14]`'s run and MUST NOT be linked into any `ember build` output; `ember test --instrument-ffi` MUST print its own overhead so the run's slowdown is not mistaken for the program's.
* `[FFI-24a]` **Exception specification is a derived fact.** The C++ importer MUST read each imported function's exception specification and record `noexcept` at grade `checked` (`[TCB-1]`) where the declaration is non-throwing — `noexcept`, `noexcept(true)`, a `noexcept(expr)` Clang evaluates to `true`, or the implicitly non-throwing destructors and defaulted special members. Where it is potentially-throwing, or where the specification is dependent and unevaluated, the fact is `unknown` and `[FFI-24]`'s `Result[T, CppError]` shape applies.
* `[FFI-24b]` **The thunk shape follows the grade, not the assertion.** * Grade `checked` or higher: the thunk is declared `noexcept`, takes no `EmberCppError*`, and the Ember signature returns `T` rather than `Result[T, CppError]`. This is the zero-overhead path. * Grade `asserted` (an `@ffi(noexcept)` an overlay author wrote): the thunk is declared `noexcept` and MUST still contain `catch (...) { ember_panic_foreign_throw("<decl>", "<overlay file>:<line>"); }`, so a violated assertion produces a **named Ember panic** rather than an unattributed `std::terminate`. The Ember signature returns `T`. On both supported ABIs a table-based `try` region costs nothing on the non-throwing path, so this carries no runtime cost the `checked` path avoids.
* `[FFI-24c]` **Disclosure, to `[FFI-29c]`'s standard.** `ember inspect` MUST report, for every imported C++ call, its exception mode as `noexcept (checked)`, `noexcept (asserted)` or `catching (Result[T, CppError])`, and MUST state that a `catching` call is not ABI-direct. `ember bind --report` MUST list every potentially-throwing declaration on a `threads = main` or `@noalloc` path, because those are where the `Result` costs most, **and MUST flag any declaration whose exception mode changed since the previous import.**
* `[FFI-24d]` **Compatibility.** Because the fact is derived, an upstream header adding or removing `noexcept` silently changes an Ember signature between `T` and `Result[T, CppError]` — a source break in the consumer caused by a dependency edit. `[BLD-2]`'s interface hash covers it and will invalidate correctly, and the resulting diagnostic MUST name the header, the declaration and the exception-mode change as the cause rather than reporting a bare type mismatch.
* `[FFI-17e]` **The bridge types are library types.** `CppVector[T]`, `CppString` and `CppShared[T]` are declared in `std.ffi` and MUST carry explicit markers: all three are `!Copy` and `!Send`/`!Sync` in v1; `CppVector[T]` and `CppString` are `Drop` (the thunked C++ destructor) and their `.span()`/`.span_mut()`/`.as_str()` results carry the receiver's region under `[LT-1]` rule 1, so a span may not outlive the vector; `CppShared[T]` is `Drop` and its clone and drop call the C++ control block. **`CppShared[T]` is not `Shared[T]`**: `[HEAP-*]`'s `Shared[T]` has Ember's control block and `[RC-*]`'s elision rules, `CppShared[T]` has C++'s, and **the two never interconvert**. Part XV's `std.ffi` row gains all three.
* `[FFI-17f]` **Cost disclosure, to `[FFI-29c]`'s standard.** `ember inspect` MUST report every C++ bridge operation that is a non-inlinable thunk call, naming `CppShared[T]` clone/drop, `CppVector[T]` `push_back`/`len`/drop and `CppString` drop specifically, and MUST state that `[BEN-6]`'s "FFI call overhead equal to the C++ reference" gate applies to ABI-direct C calls and not to these. `[RC-6]`'s surviving-retain/release report gains a `Surviving: CppShared` section, since a `CppShared` clone inside a loop is the same optimisation barrier for the same reason.

## XVI.11 Build integration


### Extending a C++ class from Ember (`[FFI-39]`)

The case Ember must handle to be structural rather than leaf-only in an existing
engine: an engine base class with virtuals, subclassed from Ember.

```ember
import cpp "RageV/src/RageV/Core/Layer.h" with (project="ragev", overlay="overlays/layer.em")

@ffi(trampoline, virtuals=["OnAttach", "OnDetach", "OnUpdate", "OnEvent"])
extern class cpp.RageV.Layer:
    init(name: CppString)                       # names a C++ base constructor

class DebugOverlay(cpp.RageV.Layer):
    frames: u64 = 0

    init(self):
        super.init(CppString.from("DebugOverlay"))     # [FFI-39b]

    override fn OnAttach(mut self):            log("attached")
    override fn OnUpdate(mut self, dt: f32):   self.frames += 1
```

* `[FFI-39]` **A foreign base is a declared base, not an opaque type.** An
  `extern class` carrying `@ffi(trampoline, virtuals=[…])` declares a foreign type
  that satisfies `[CLS-4]`'s requirement on a base: it is implicitly `open`, it is
  sized (the importer records its layout under `[FFI-5]`/`[FFI-5a]`), and each
  name in `virtuals=[…]` is an implicitly `virtual` method an Ember subclass may
  `override`. `[FFI-8]`'s opaque-`extern type` form remains available and remains
  unsized; the two are different declarations and only this one may be inherited.
  Overriding a virtual not named in `virtuals=[…]` is `E5056`, naming the list;
  naming a method that is not virtual in the header is `E5057`.
* `[FFI-39a]` The importer emits a C++ subclass — `em_tramp_RageV_Layer` — whose
  overridden virtuals call the Ember override through its permanent thunk
  (`[HR-6]`), so overrides survive hot reload with no further machinery, and whose
  constructor stores the Ember instance handle in a member.
* `[FFI-39b]` **Base construction is explicit.** An `extern class` with
  `@ffi(trampoline)` declares one or more `init(…)` signatures naming C++ base
  constructor overloads; the derived Ember `init` MUST call `super.init(…)` exactly
  once, as `[CLS-4]` requires of every class, and the call selects the overload by
  arity and argument types. A base with no default constructor and no declared
  `init` is `E5058`. `super.init` runs the C++ base constructor before any Ember
  field is initialised, matching `[CLS-4]`'s order.
* `[FFI-17c]` **Destruction is derived-first, exactly as `[CLS-6]` specifies for
  every other class.** Dropping the last handle runs the Ember `drop`, then the
  Ember fields' drops in reverse declaration order, and only then the C++ base
  destructor. An Ember `drop` may therefore call an inherited method or pass `self`
  upcast to a foreign API, which base-first ordering would have made a
  use-after-free on a subobject whose destructor had already run.
* `[FFI-17d]` **Ownership of the trampoline object is declared, not assumed.**
  `@ffi(trampoline, owner="ember")` — the default — means the Ember handle count
  owns the pair, and the C++ subobject is destroyed when the count reaches zero.
  `@ffi(trampoline, owner="foreign")` means a foreign `delete` destroys the pair,
  and the Ember side holds a non-owning handle; this is the shape of
  `PushLayer(Layer*)` storing the pointer and deleting it in the host's destructor,
  and it is the dominant engine idiom. Under `owner="foreign"` the upcast of
  `[FFI-39c]` releases Ember's strong reference, and dropping the last Ember handle
  does **not** run the C++ destructor. A `@ffi(trampoline)` type whose base has no
  virtual destructor MUST declare `owner=`; `[FFI-17b]`'s `@ffi(no_virtual_dtor)`
  covers the remaining case, and omitting both is `E5059`.
* `[FFI-39c]` **Upcasting.** `self` in an override, and an owning handle at a call
  site, upcast implicitly to the foreign base pointer for passing to foreign APIs.
  The upcast is a `[FFI-23]` retention point: under `owner="ember"` it produces a
  `Retained[Self]` token, which is what keeps the object alive while the host holds
  the raw pointer and what `[HR-20]` uses to relocate it across a reload. An upcast
  in a context that cannot hold the token — a temporary passed and immediately
  discarded — is `E5060`, naming `Retained.pin` as the fix.
* `[FFI-39d]` **Re-entrancy is expected and MUST NOT panic.** The ordinary engine
  shape is `PushLayer(Layer* l) { m_Layers.push_back(l); l->OnAttach(); }`: Ember
  calls out with a live `mut self` access and the host synchronously calls back
  into an override on the same object. The inbound trampoline path therefore
  **ends the caller's long-term access for the duration of the foreign call**
  (`[EXC-3]`'s access is closed at the call and reopened on return), so the
  override begins its own access without conflict. An implementation MUST NOT
  route this through `[EXC-1]`'s dynamic check. The borrow the caller held is not
  extended across the foreign call, so anything it observed MUST be re-read
  afterwards; this is stated in the diagnostic for `[EXC-1]` and in
  `docs/errors/`.
* `[FFI-39e]` **The inbound path has a panic boundary.** A panic inside an Ember
  override MUST NOT unwind into the C++ frame that called it: the trampoline
  catches it at the boundary exactly as `[FFI-24]` handles the outbound direction,
  reports it, and aborts under `[PAN-1]`'s package policy. Unwinding through a
  foreign frame is never permitted in either direction.

### Member mapping (`[FFI-40]`)

* `[FFI-40]` **`this`-qualifiers and references.** A `const` member function imports
  with a `self` receiver, a non-`const` one with `mut self`, an `&&`-qualified one
  is skipped (`W5033`). `T&` maps to `ref mut T`, `const T&` to `ref T`, `T&&` to an
  `owned` parameter where the type is movable and is otherwise skipped. A function
  returning `T&` returns a `ref T` whose region is elided from the receiver
  (`[LT-1]` rule 1). `[FFI-40a]` **`const` is not an aliasing guarantee in C++**, so
  a `const` member function that mutates through `mutable` state or invalidates
  iterators MUST be declared `@ffi(invalidates)` in the overlay, which gives it a
  `mut self` receiver. An importer MUST assume `@ffi(invalidates)` for any `const`
  member of a type whose header declares a `mutable` member and whose overlay is
  silent, rather than trusting `const`; the overlay may then state otherwise, and
  that statement is an `asserted` fact under `[FFI-35]` appearing in `ember tcb`.
* `[FFI-41]` **Statics, nested types and namespaces.** Static member functions
  import as associated functions (`cpp.RageV.RHIDevice.Create(…)`); static data
  members as `extern static`; nested classes and enums under the outer name
  (`cpp.RageV.RHIDevice.Desc`); namespaces as module paths under the import name.
  Anonymous-namespace entities are skipped (`W5034`), because their identity is
  per translation unit and `[FFI-30]`'s identity rule cannot hold for them.
* `[FFI-42]` **Operators and iteration.** C++ `operator==`, `<`, `+` and friends map
  onto Ember's operator interfaces where the signature allows; `operator[]` to
  `Index`/`IndexMut`; `operator*`/`operator->` on a smart-pointer-shaped type to
  auto-deref. A type with `begin()`/`end()` returning a forward iterator imports an
  `Iterable` driven by thunks, so `for x in cpp_vec:` works. `[FFI-42a]`
  **Iterator invalidation is prevented, not merely discouraged.** The imported
  `Iterable` takes `ref mut` of the container for the loop under `[CTL-2]`, so any
  method needing `mut self` — which by `[FFI-40a]` includes every `const` method
  the overlay has not cleared — is rejected inside the loop by the ordinary borrow
  rules. An earlier draft left this to a `SHOULD`, which left a use-after-free
  reachable from safe Ember; it is a `MUST`.

### Toolchain (`[BLD-FFI-4]`, `[BLD-FFI-5]`)

* `[BLD-FFI-4]` **Cross-language optimisation.** With the C backend, Ember's emitted
  C and the project's C++ are compiled by the same compiler, so the project's LTO
  setting applies to both and calls across the boundary inline like any other call.
  With the LLVM backend, `lto = "thin"` plus matching `-flto` on the C++ side
  achieves the same. `[BLD-FFI-4a]` Reloadable functions are excluded from
  cross-boundary inlining by `[HR-9a]`, and the toolchain MUST NOT enable an LTO
  configuration that would defeat it.
* `[BLD-FFI-5]` **Exporting Ember to C++ with C++ types.** `ember build
  --emit-header --cpp` writes `<package>.hpp` alongside the C header: RAII wrappers
  over the C API, `std::string_view` overloads for `str` parameters, `std::span`
  overloads for `Span`. The C header remains the ABI; the C++ header is a
  header-only convenience layer over it. `[BLD-FFI-5a]` **A smart handle is
  generated only for a `Sync` class.** `[RC-1]`'s reference count is atomic iff the
  class is `Sync`, and a generated C++ wrapper is an ordinary copyable type that
  can be put in a `std::vector`, captured by a lambda and moved to another thread —
  which for a non-`Sync` class would perform a non-atomic increment from two
  threads. For a non-`Sync` class the header emits an explicitly thread-confined
  handle type that is neither copyable nor movable across threads, documented as
  such, and `ember build --emit-header --cpp` reports which classes got which.

* `[BLD-FFI-1]` `ember.toml` `[cpp.<project>]` records `compiler` (`msvc | clang-cl | clang | gcc`), `standard`, `defines`, `include_paths`, `flags`, `libs`, and optionally `cmake = { build_dir = "build", target = "RageV" }` from which the toolchain reads `compile_commands.json` / the CMake File API to obtain the exact flags of the target — this is the mechanism that guarantees `[FFI-18]`.
* `[BLD-FFI-2]` `cmake/EmberModule.cmake` (shipped with the toolchain) provides `ember_add_library(name SOURCES ... KIND staticlib|cdylib)` and `ember_add_executable(...)`, which invoke `ember build` with `--cc-flags-from-target <cmake-target>` and add the produced C files (C backend) as an OBJECT library to the CMake target graph so that MSVC/clang compile them with the same flags, LTO and debug settings as the rest of the engine. With the LLVM backend it instead adds the produced `.obj`/`.o`.
* `[BLD-FFI-3]` Existing static/shared libraries are consumed by listing them in `link = [...]` or by inheriting the CMake target's link interface.
* `[BLD-FFI-1a]` **Flag inheritance is defined, not copied.** When flags come from a CMake target, the toolchain takes the command line of a representative TU and applies this filter. **On Windows the CMake File API is normative**; `compile_commands.json` is used only when the generator produces it. *Dropped:* precompiled-header switches (`/Yc`, `/Yu`, `/Fp`, `/FI`, `-include`, `-include-pch`); output and dependency paths (`/Fo`, `/Fd`, `-o`, `-MD`, `-MF`, `/showIncludes`); whole-program and PGO switches (`/GL`, `/LTCG`, `-flto`, `/analyze`); warning and diagnostic switches; the source operand. ***Every other switch is inherited*** — compilers add switches faster than a specification is revised, so the drop-list is the closed set and the default is inheritance. Explicitly inherited and individually load-bearing: `-D`/`/D`; include paths; `-std`/`/std`; architecture and ISA; the MSVC runtime library (`/MD`, `/MDd`, `/MT`, `/MTd`); the exception model; RTTI; `/Zc:*`; `-fms-compatibility-version`/`_MSC_VER`; structure packing; `-fshort-enums`; `char` signedness. `E9020` if the TUs of one target disagree on any inherited flag — **with an escape hatch, because real targets legitimately carry per-file `-D` overrides: an explicit `[cpp.<project>] flags` entry in `ember.toml` wins over inference, and `E9020`'s help MUST name it.**
* `[BLD-FFI-1b]` The MSVC runtime-library switch and the effective values of `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` MUST be inherited byte-for-byte, because they change the layout of `std::string`/`std::vector` and the identity of the CRT heap. If the toolchain cannot determine them it MUST fail with `E9021`, never default.

---

