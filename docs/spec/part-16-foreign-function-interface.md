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
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32: ...

@export_table("RvScriptApi", protocol=3)                # a function-pointer table struct, RageV style (§Part XXI)
pub struct ScriptApi:
    on_create: extern "C" fn(u64) -> i32
    on_update: extern "C" fn(u64, f32) -> i32
```

* `[FFI-26]` `ember build --emit-header` writes `<package>.h` with prototypes for every `@export`, C typedefs for every `@layout(c)` type used in them, and `#define <PACKAGE>_PROTOCOL_VERSION n`.
* `[FFI-27]` The Ember runtime (`ember_rt`) is a C11 static library with no global constructors. Embedding API:
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

## XVI.11 Build integration

* `[BLD-FFI-1]` `ember.toml` `[cpp.<project>]` records `compiler` (`msvc | clang-cl | clang | gcc`), `standard`, `defines`, `include_paths`, `flags`, `libs`, and optionally `cmake = { build_dir = "build", target = "RageV" }` from which the toolchain reads `compile_commands.json` / the CMake File API to obtain the exact flags of the target — this is the mechanism that guarantees `[FFI-18]`.
* `[BLD-FFI-2]` `cmake/EmberModule.cmake` (shipped with the toolchain) provides `ember_add_library(name SOURCES ... KIND staticlib|cdylib)` and `ember_add_executable(...)`, which invoke `ember build` with `--cc-flags-from-target <cmake-target>` and add the produced C files (C backend) as an OBJECT library to the CMake target graph so that MSVC/clang compile them with the same flags, LTO and debug settings as the rest of the engine. With the LLVM backend it instead adds the produced `.obj`/`.o`.
* `[BLD-FFI-3]` Existing static/shared libraries are consumed by listing them in `link = [...]` or by inheriting the CMake target's link interface.

---

