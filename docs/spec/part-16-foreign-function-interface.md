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
              ──► Clang AST walk: functions, structs, unions, enums, typedefs, global variables, object-like
                   macros that expand to integer/float/string literals, function-like macros are ignored (W5001)
              ──► Binding IR (BIR): a language-neutral description with layouts computed by Clang
              ──► overlay applied (contracts, renames, hides, wrappers)
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

`[FFI-17]` The C++ importer produces, for each requested class/function:

1. A **thunk file** `target/bind/<hash>_thunks.cpp` containing `extern "C"` functions with predictable names (`em_cpp_<ns>_<class>_<method>_<sig-hash>`) that (a) call the C++ member/free function, (b) catch all exceptions and return a status + message buffer (§9), (c) never return C++ objects by value across the boundary — objects are heap-allocated (`new`) and returned as opaque owned pointers, or written into caller-provided storage when trivially copyable.
2. Ember declarations in the synthetic module: an `extern type RHIDevice`, a `struct RHIDeviceRef`/`ForeignBox[RHIDevice]` pairing, and methods on them that call the thunks. Constructors become `RHIDevice.new(...) -> ForeignBox[RHIDevice]`; the destructor is the box's `drop`. Overloads are disambiguated by suffixing parameter types (`draw_indexed_u32`) unless the overlay names them.
3. Templates are only available as **explicit instantiations** listed in `instantiate` or referenced by an imported signature; each instantiation gets its own thunks. `std::vector<T>` maps to `CppVector[T]` with `.span()`/`.span_mut()` (zero-copy, borrowing the vector), `.to_array()` (copy), `push_back`, `len`; `std::string` to `CppString` with `.as_str()`; `std::string_view` to `str`; `std::unique_ptr<T>` to `ForeignBox[T]`; `std::shared_ptr<T>` to `CppShared[T]` (calls the C++ control block through thunks); `std::optional<T>` to `Option[T]` for trivially copyable `T`; `std::function` is not importable (`E5030`) — use a C callback.
4. **Compiler matching** `[FFI-18]`: the thunks are compiled by the *same* compiler, standard and flags as the project (`[cpp.ragev]` section), so they share the C++ ABI with the engine (MSVC ABI when the engine is built with MSVC; clang-cl also targets the MSVC ABI). libclang parses the headers in MSVC-compatibility mode with the same `_MSC_VER`. `[FFI-19]` If the project's compiler is MSVC and libclang cannot parse a header (MSVC-specific extension), the importer reports the exact diagnostic and the declaration is skipped with `W5031`; the programmer then binds it manually through a small C shim.
5. Inheritance and virtual functions: a C++ class hierarchy is imported as opaque types with **upcast thunks** only; Ember cannot subclass C++ classes in v1. Overriding a C++ virtual from Ember is done through a generated C++ "trampoline subclass" listed in `overlay` with `@ffi(trampoline)` (v2).
6. Namespaces map to Ember module paths under the import name: `cpp.RageV.RHIDevice`.
7. `constexpr`/`const` integral constants and unscoped/scoped enums import like C.

`[FFI-20]` Everything an importer cannot represent is reported once with the reason (`ember bind --report`), never silently dropped.

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

@export_table("RvScriptApi", protocol=3)                # a function-pointer table struct, RageV style (§Part XXI)
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
* `[FFI-31b]` **No owning Ember value may cross a module boundary.** Because each `cdylib` owns a private allocator and type-info table, class handles, `Box`, `Shared`, `Weak`, `String`, `Array`, `Map`, and any type with drop glue MUST NOT appear in an `@export` or `@export_table` signature, nor be reachable through a pointer in one. Only `@layout(c)` value types, scalars, opaque handles, `cstr` (copied at the boundary) and `extern "C" fn` may cross. Violation is `E5015`, naming the offending type and the reason. This makes Part XXI's `[RV-2]` a language rule rather than a plan convention, and follows directly from XXII.2's "no stable Ember-to-Ember ABI".
* `[FFI-31c]` A host that must share one runtime between several Ember modules links `ember_rt` as a shared library and builds each module with `[build] runtime = "shared"`. This is the only configuration in which `[FFI-31b]` may be relaxed, and it is **v2**; v1 packages are always privately linked.
* `[FFI-33]` **Thread contracts on exported functions.** `@export` and `@export_table` accept `threads = any | main | creator` with `[FFI-11]`'s meanings, defaulting to `any`; a `@export_table` MAY set it per field; a module MAY declare a default with `#! threads main` (which is what Part XXI §1 assumes). `threads = any`: the compiler checks the exported function's reachable call graph as if it ran on an arbitrary thread — every `static` it reaches MUST be `Sync`, and no non-`Sync` class handle may be reachable from a `static`, a parameter or a captured value; violation is `E7010` naming the reached item and why it is not `Sync`, with `#! threads main` named as the fix. `threads = main`: the exported wrapper asserts, in `debug` and `release`, that the calling thread is the one that called `ember_module_init`/`ember_rt_init`, and panics `ember_panic_thread` otherwise; in exchange the body MAY touch non-`Sync` statics and handles. `threads = creator`: as `main`, but the asserted thread is the one that created the value the call is dispatched on.
* `[FFI-33a]` `[FFI-22]` is amended: `ember_rt_thread_attach()` establishes thread-local runtime state and confers **no** right to touch thread-confined data. In the `debug` profile, under the existing `debug_objects` profile key, an extended object header records the attaching thread id, and `ember_retain`/`ember_release` on a non-`Sync` object from a different thread panics naming the class and both thread ids. **`[OBJ-1]`'s release header layout is unchanged**: the 24 bytes are fully occupied and a thread id does not fit, so this is a debug-only extension, never a change to the ABI `[VER-4]` freezes.
* `[FFI-33b]` `returns_owned(destructor=f)`, `[FFI-23]`'s `Retained[T]` and `Callback[F]` MAY carry `threads=`. A `ForeignBox[T]` whose destructor is `threads = creator` records the creating thread on construction and panics in `debug` when dropped elsewhere; this is what makes GPU, GL-context and COM handles safe to hold in ordinary Ember values. `ember bind --report` lists every foreign destructor with no thread contract.
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
* `[FFI-2a]` Contracts the generator **derives** from the C declaration are verified facts and MUST NOT require `unsafe`: `const T*` ⇒ borrowed immutable, `T*` ⇒ borrowed mutable, struct/union layouts asserted per `[FFI-5]`, enum underlying types, `@packed`/`@align` reproduction, calling convention. Only `unknown`-filling contracts require it: `owned`, `returns_owned`, `retained`, `span`, `len_of`, `string`, `nullable`, `handle`, `status`, `threads=`, `effects=`, `noexcept`, `callback=`, and `noalias` (`[SIMD-3]`). An overlay containing none of these needs no `unsafe`.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` are part of the **overlay language**, not merely of the tool. `[FFI-11]`'s vocabulary and `[FFI-12]`'s signature check MUST accept them, and `[FFI-2]` MUST treat a declaration carrying one as **uncontracted** — its calls stay `unsafe`. This is what lets a team adopt a header on day one, run everything behind `unsafe:`, and buy safety incrementally rather than writing 2000 lines before the first call compiles.
* `[FFI-6]` **Macro import.** The importer MUST import an object-like macro whose replacement list, **after full macro expansion**, is a constant expression of integer, floating, string-literal, **null-pointer-constant, or pointer/handle-cast** type as evaluated by Clang under the import's own configuration (a probe TU of one `static const` or `enum` per candidate macro, parsed with the same configuration and cached in the `.embind`). The macro's Ember type is the C type of the evaluated expression after the usual arithmetic conversions. This covers `(~0U)`, `(-1)`, `(1 << 3)`, `sizeof(T)`, `0xffffffffffffffffULL`, `((VkBuffer)0)` — which maps to the imported handle newtype's NULL per `[FFI-11]`'s `handle` contract — and object-like macros whose bodies expand through function-like macros.
* `[FFI-6a]` A macro that does not evaluate to such a constant is skipped with `W5001 macro not imported: <reason>`, where `<reason>` is Clang's evaluation failure — never a blanket category. The macro, its body and its reason MUST appear in the `.embind` and in `ember bind --report`.
* `[FFI-6b]` A **function-like** macro MAY be exposed as a function when an overlay declares its signature (`@ffi(macro_fn) fn VK_MAKE_API_VERSION(variant: u32, …) -> u32`). The importer emits `uint32_t em_mac_VK_MAKE_API_VERSION(…) { return VK_MAKE_API_VERSION(…); }` into the generated shim translation unit (`[FFI-29]`, RFC-043) and binds `em_mac_*` as an ordinary `extern "C"` function. **This adds no macro facility to Ember: the imported entity is a C function.** Where all arguments are compile-time constants the importer MAY fold the call through `[FFI-6]`'s probe TU, so `VK_MAKE_API_VERSION(0,1,3,0)` is usable in a `const` initialiser.
* `[FFI-5a]` In addition to the libclang cross-check, the importer MUST emit into the generated shim TU (`[FFI-29]`) or thunk TU (`[FFI-17]`) a `_Static_assert`/`static_assert` for `sizeof` and `_Alignof` of every imported aggregate that Ember constructs, passes or returns by value, or embeds, and for `offsetof` of every field Ember reads or writes. **The layout contract is therefore verified by the compiler that builds the project, on the user's machine, on every build**, and a mismatch is a build error naming the type, the field and both offsets (`E5001`). XVIII §10's cross-compiler tests remain and cover the compiler's own fixtures.

## XVI.11 Build integration

* `[BLD-FFI-1]` `ember.toml` `[cpp.<project>]` records `compiler` (`msvc | clang-cl | clang | gcc`), `standard`, `defines`, `include_paths`, `flags`, `libs`, and optionally `cmake = { build_dir = "build", target = "RageV" }` from which the toolchain reads `compile_commands.json` / the CMake File API to obtain the exact flags of the target — this is the mechanism that guarantees `[FFI-18]`.
* `[BLD-FFI-2]` `cmake/EmberModule.cmake` (shipped with the toolchain) provides `ember_add_library(name SOURCES ... KIND staticlib|cdylib)` and `ember_add_executable(...)`, which invoke `ember build` with `--cc-flags-from-target <cmake-target>` and add the produced C files (C backend) as an OBJECT library to the CMake target graph so that MSVC/clang compile them with the same flags, LTO and debug settings as the rest of the engine. With the LLVM backend it instead adds the produced `.obj`/`.o`.
* `[BLD-FFI-3]` Existing static/shared libraries are consumed by listing them in `link = [...]` or by inheriting the CMake target's link interface.
* `[BLD-FFI-1a]` **Flag inheritance is defined, not copied.** When flags come from a CMake target, the toolchain takes the command line of a representative TU and applies this filter. **On Windows the CMake File API is normative**; `compile_commands.json` is used only when the generator produces it. *Dropped:* precompiled-header switches (`/Yc`, `/Yu`, `/Fp`, `/FI`, `-include`, `-include-pch`); output and dependency paths (`/Fo`, `/Fd`, `-o`, `-MD`, `-MF`, `/showIncludes`); whole-program and PGO switches (`/GL`, `/LTCG`, `-flto`, `/analyze`); warning and diagnostic switches; the source operand. ***Every other switch is inherited*** — compilers add switches faster than a specification is revised, so the drop-list is the closed set and the default is inheritance. Explicitly inherited and individually load-bearing: `-D`/`/D`; include paths; `-std`/`/std`; architecture and ISA; the MSVC runtime library (`/MD`, `/MDd`, `/MT`, `/MTd`); the exception model; RTTI; `/Zc:*`; `-fms-compatibility-version`/`_MSC_VER`; structure packing; `-fshort-enums`; `char` signedness. `E9020` if the TUs of one target disagree on any inherited flag — **with an escape hatch, because real targets legitimately carry per-file `-D` overrides: an explicit `[cpp.<project>] flags` entry in `ember.toml` wins over inference, and `E9020`'s help MUST name it.**
* `[BLD-FFI-1b]` The MSVC runtime-library switch and the effective values of `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` MUST be inherited byte-for-byte, because they change the layout of `std::string`/`std::vector` and the identity of the CRT heap. If the toolchain cannot determine them it MUST fail with `E9021`, never default.

---

