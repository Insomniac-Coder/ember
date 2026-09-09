# Part XV — Standard Library Surface (v1)

The standard library is one package `std` with the modules below. Each module's public surface is listed at the level needed to implement it; exact signatures live in `std/**/*.em` and are the normative source once written. `[STD-1]` The whole of `std.core`, `std.mem`, `std.math`, `std.simd`, `std.span`, `std.arena` MUST be `@noalloc`-clean except functions documented to allocate.

| Module | Contents |
|---|---|
| `std.core` (prelude) | `Option`, `Result`, `Cell`, `RefCell` (`OQ-10`), marker & operator interfaces, `Ordering`, `Iterator` adaptors (`map`, `filter`, `enumerate`, `zip`, `take`, `skip`, `chain`, `rev`, `sum`, `count`, `min_by`, `max_by`, `fold`, `any`, `all`, `find`, `position`, `collect[C]`), `Range*`, `print`/`println`/`eprintln`, `assert*`, `panic`, `todo`, `unreachable`, `mem.{take, replace, swap, drop, forget, size_of, align_of}` |
| `std.mem` | `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Layout`, `Allocator`, `Global`, `ptr.*` (unsafe pointer ops), `keep_alive` |
| `std.cell` | `Cell`, `RefCell`, `Ref`, `RefMut` (Part IX §7) |
| `std.mem` (cont.) | `assert_disjoint`, `assert_disjoint_or_panic`, `unsafe assume_disjoint` (Part IX §8) |
| `std.collections` | `Array`, `SmallArray`, `Deque`, `Map`, `Set`, `BitSet`, `Pool`, `Handle`, `SoA` (derive support), `ArenaArray` |
| `std.string` | `String`, `str` methods (`len`, `chars`, `bytes`, `split`, `trim`, `starts_with`, `find`, `parse[T]`, `to_upper`…), `StringBuilder`, `CString`, `cstr` |
| `std.fmt` | `Formatter`, `Display`, `Debug`, `format(…) -> String`, `format_to(mut buf: MutSpan[u8], …) -> Result[str, FmtError]` (`@noalloc`), f-string lowering targets |
| `std.math` | `Vec2/3/4`, `IVec*`, `UVec*`, `Mat2/3/4`, `Quat`, `Transform`, `AABB`, `Sphere`, `Plane`, `Ray`, `Frustum`, scalar funcs (`sin cos tan atan2 sqrt rsqrt pow exp log floor ceil round abs min max clamp lerp smoothstep`), constants; all `@derive(Copy)`, `@layout(c)`, and **layout-compatible with GLM's float types** (`Vec3` = 3×f32, 4-byte aligned; `Mat4` column-major) so they cross to RageV unchanged |
| `std.simd` | vector types, masks, intrinsics (`std.cpu`: `prefetch`, `pause`, `rdtsc`, `cores`, cache line size) |
| `std.arena` | `Arena`, `FixedArena`, `ScopedArena`, `ThreadArena` |
| `std.io` | `Read`/`Write` interfaces, `stdin/stdout/stderr`, buffered wrappers, `Error` |
| `std.fs` | `read`, `write`, `File` (move-only, `drop` closes), `metadata`, `read_dir`, `Path`/`PathBuf` |
| `std.time` | `Instant`, `Duration`, `sleep`, `SystemTime` |
| `std.thread`, `std.sync`, `std.jobs`, `std.atomic` | Part XI |
| `std.process` | `exit`, `args`, `env`, `Command` (spawn child), `abort` |
| `std.ecs` | Part XII §3 |
| `std.ser` | binary + YAML (de)serialisation |
| `std.testing` | `@test` support, `assert_approx_eq`, `expect_panic`, `bench` harness |
| `std.ffi` | `CString`, `cstr`, `c_int`… type aliases (`c_int` = target `int`), `Callback[F]`, `Retained[T]` (foreign-retained handle), `ForeignBox[T]` (owned foreign pointer with destructor fn), `NativeApiTable` helpers (Part XXII) |
| `std.gpu` | Part XVII (host-side; backend-agnostic) |
| `std.debug` | `backtrace`, `leak_report`, `alloc_stats`, `frame_profiler` markers (`zone("name")` scoped) |

`[STD-2]` `print` and friends allocate only for f-strings; `println("literal")` and `println(some_str)` are `@noalloc`.
* `[STD-3]` `std.math` MUST provide scalar `fma(a: f32, b: f32, c: f32) -> f32` (and the `f64` form) and `Vec2/3/4.mul_add`, lowered to `fmaf`/`fma`/`_mm_fmadd_ps`/`vfmaq_f32`, so that fused multiply-add is expressible as an explicit, IEEE-defined operation **with no float-control attribute at all**. These are the recommended form for `Mat*` multiply, `dot` and transform composition, and `std.math` MUST use them internally.
* `[STD-4]` `std.core` provides `NonZero[T]` for each integer `T`: a `Copy` newtype with a niche (`Option[NonZero[T]]` is `T`-sized per `[TYP-13]`), constructed by `NonZero.new(v) -> Option[NonZero[T]]` or `unsafe new_unchecked`. Division by a `NonZero` divisor carries no `Panic(Explicit)`. This is the mechanism by which a divide in a `@nopanic(explicit)` function is expressible. `debug_assert*` is permitted in `release` and `shipping`, where it compiles to nothing.

---
* `[STD-6]` `std` is **layered**, and the layers are a build-time choice, not a
  convention: **core** (primitives, `Option`, `Result`, views, fixed-capacity
  containers, math), **alloc** (`Array`, `String`, `Box`, `Map`), **sync**
  (`Atomic`, `Mutex`, channels), **io** (files, sockets, console), **ffi**,
  **verify** (proof-only helpers). A layer may depend only on the layers before
  it in that order.
* `[STD-8]` **`Contains`.** `std.core` declares `interface Contains[T]: fn contains(self, item: T) -> bool`. `x in coll` requires `typeof(coll): Contains[typeof(x)]` and is `E2226` otherwise, whose `help` names the explicit form — `coll.iter().any(|e| e == x)` — so the fix is always available and always visibly O(n). `std` implements it for: `Set[T]` and `Map[K, V]` (**by key**, matching `[STD-*]`'s iteration of a `Map` as pairs — `k in map` tests the key and never a value); `Array[T]`, `Span[T]`, `MutSpan[T]` and `[T; N]` where `T: Eq`, **as a linear scan**; `str` and `String`, as a substring test; and `Range[T]`, as two comparisons.
* `[STD-8b]` **`str` membership is by Unicode scalar, never by byte.** `str` and `String` implement `Contains[char]` — does the string contain that scalar value — and `Contains[str]`, a substring test matching only at a codepoint boundary, and **nothing else**. `Span[u8] in str` is `E2226`: a byte needle has no well-posed answer in a type whose invariant is that it is valid UTF-8 (`[TXT-1]`), and admitting it would let `x in s` be true for a needle that is not a substring of `s` as any reader sees it. A `char` needle therefore can never match a continuation byte, and a `str` needle never matches a split codepoint — `[TXT-4]`'s boundary rule applied to search rather than to slicing. Byte-level search stays available, and well-posed, on `Span[u8]`, which is what `s.as_bytes()` returns.
* `[STD-8a]` **`in` never hides its cost.** `Contains` is an ordinary interface method, so `[COST-3]`'s row governs and `ember inspect --cost` reports the complexity of the implementation actually selected. This is the whole reason the operator lowers to a **declared bound** rather than to a compiler-synthesised scan: `id in players` on a `Map` is a hash lookup and on an `Array` is a scan, the two are told apart by the container's type, and a container that has no cheap answer either implements `Contains` and is reported as linear or does not implement it and is a type error. An implementation MUST NOT synthesise a `Contains` impl for a type that does not declare one.
* `[STD-7]` `core` provides `FixedArray[T, N]`, `FixedString[N]` and
  `RingBuffer[T, N]`: compile-time capacity, no heap allocation, and an explicit
  policy when full — `push` returns `Result`, `push_or_drop` does not. They
  take an integer capacity parameter under IV.7's `const N: usize` generic form,
  which v1 has. `[STD-7a]` records that **arbitrary const-generic expressions**
  — arithmetic over capacity parameters — are a milestone, not a gap in the
  parameter form itself.

* `[STD-7a]` General const-generic support for integer capacity parameters is a
  prerequisite for the unbounded `FixedArray[T, N]`, `FixedString[N]` and
  `RingBuffer[T, N]` forms. The implementation plan therefore contains an explicit
  **const-generic milestone before the fixed-capacity container milestone**. A
  bootstrap implementation MAY expose a finite set of capacities temporarily, but
  it MUST diagnose a non-supported capacity as a capability limitation rather than
  as a malformed type, and it MUST NOT claim `[STD-7]` complete until arbitrary
  compile-time `N` is accepted. `[GRM-8]`, `[CT-1]`, `[MONO-1]` and `[TYP-19]` govern
  parsing, constant evaluation, monomorphisation and identity of const-generic
  arguments respectively.


## XV.4a The string and text model

Strings are the highest-frequency interoperability type in a language whose purpose
is migrating a C++ codebase, and 0.7.1 spread their rules across `[STD-*]`,
`[FFI-17]`, `[UNS-4]` and Part IV. This section is normative and supersedes any
weaker statement elsewhere.

* `[TXT-1]` **Four types, one guarantee.** `str` is a borrowed `Span[u8]` **known to
  be valid UTF-8**; `String` owns a heap buffer with the same guarantee; `CppString`
  is an opaque owned `std::string` (`[FFI-17e]`); `*const c_char` is a foreign
  pointer with no guarantee at all. Only the first two carry the UTF-8 invariant,
  and `[UNS-4]` lists it among the invariants safe code may assume.
* `[TXT-2]` **Nothing becomes a `str` without validation.** Every conversion from
  foreign or untrusted bytes — `CppString.as_str()`, `std::string_view`,
  `*const c_char`, a `Span[u8]` — is fallible and returns `Result[str, Utf8Error]`,
  or its `_lossy` form which substitutes U+FFFD and is total. The importer MUST NOT
  map `std::string` or `std::string_view` to `str` directly; it maps them to
  `CppString` and `Span[u8]` respectively, from which `[TXT-2]`'s conversion is the
  only route. An unvalidated route is `E5063`. This closes the hole by which a
  `yaml-cpp` scalar, a `cgltf` name, an ImGui buffer or a CP-1252 path could enter
  safe Ember as a `str` and be walked by a UTF-8 decoder.
* `[TXT-3]` **No null termination.** `str` and `String` are pointer + length and MAY
  contain interior nulls. A C boundary needs `CStr`/`CString` (`std.ffi`), whose
  construction from a `str` is fallible on an interior null (`E5064` at compile time
  where the value is a literal, `Result` otherwise) and which is the only form the
  importer accepts where a header says `const char*`.
* `[TXT-4]` **Slicing is by byte index and rejects a split codepoint.** `s[a..b]`
  panics in `debug`/`release` when either bound is not a codepoint boundary, and
  `s.get(a..b) -> Option[str]` is the total form. Indexing by codepoint is not
  provided: it is O(n) and hides that cost.
* `[TXT-5]` **Conversion costs are stated, and `ember inspect --alloc` reports
  them.** `str → String` allocates and copies. `String → str` is free.
  `CppString → String` allocates, copies and validates. `str → CStr` allocates
  unless the source is a literal, which the compiler null-terminates statically.
  `Span[u8] → str` validates in O(n) and does not allocate.
* `[TXT-6]` **The C ABI representation of `str` is `{const u8*, usize}`**, matching
  `Span[u8]`, and it is **not** `const char*`. A `str` parameter in an `@export`ed
  function appears in the generated header as that pair; `[BLD-FFI-5]`'s C++ header
  additionally offers a `std::string_view` overload, which is the zero-copy form.
* `[TXT-7]` Encoding is UTF-8 everywhere. There is no UTF-16 or wide-string type in
  the language; a Windows API needing `wchar_t*` gets it from `std.ffi`'s
  `WideString`, an explicitly-converting owned buffer, and the conversion is visible
  at the call site rather than implicit.
* `[TXT-8]` `str` is a view type (`[TYP-15]`) and carries a region: a `str` borrowed
  from a `CppString` cannot outlive it, and the borrow checker enforces it exactly
  as for any other `Span`.

## VIII.5a Cycle diagnosis

* `[WK-4]` **The leak reporter names the cycle, not just the leak.** `[WK-1]`'s
  debug runtime reports leaked instances at shutdown; where the leaked set contains
  a strong-reference cycle it MUST additionally report the **shortest strong cycle**
  through each leaked object, as a path of `Type.field` edges, and suggest which
  edge to weaken:

```
L3017: reference cycle detected — 3 objects, 1 cycle

  World
    └─> EntityManager        World.entities
         └─> Entity          EntityManager.dense[i]
              └─> World      Entity.world          ← suggested Weak

  make `Entity.world` a `Weak[World]`; every other edge on this cycle is
  load-bearing (removing it would orphan the object)
```

  The suggestion names the edge that closes the cycle back to the object with the
  most incoming strong references, which is the owner in every ordinary graph shape.
  This is a diagnostic, not a collector: `[WK-1]`'s statement that cycles leak is
  unchanged, and no tracing pass exists in any profile.

## IX.5a Unsafe categorisation

* `[UNS-9]` **Every `unsafe` block carries a machine-readable reason category**, in
  addition to `[UNS-7]`'s prose obligation: `unsafe(reason = "ffi" | "layout" |
  "aliasing" | "intrinsic" | "performance" | "uninit")`. A block without one is
  `L3018` under `edition_lints = "strict"` and a warning otherwise; the category is
  a fixed closed set, because a free-form field would not be aggregable and the
  point of the field is the aggregate. `ember tcb` reports the distribution:

```
unsafe surface: 17 blocks in 6 modules
  ffi          8    (all in ffi/vulkan.em — the generated boundary)
  layout       3
  aliasing     2
  performance  4
```

  `[UNS-9a]` The category is advisory to the compiler and normative to the audit:
  it changes no check and no codegen, and `[TCB-*]`'s reports MUST carry it so that
  a reviewer can ask "how much of our unsafe surface is performance work we could
  delete" and get an answer.

---

