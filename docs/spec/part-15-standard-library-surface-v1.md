# Part XV — Standard Library Surface (v1)

The standard library is one package `std` with the modules below. Each module's public surface is listed at the level needed to implement it; exact signatures live in `std/**/*.em` and are the normative source once written. `[STD-1]` The whole of `std.core`, `std.mem`, `std.math`, `std.simd`, `std.span`, `std.arena` MUST be `@noalloc`-clean except functions documented to allocate.

| Module | Contents |
|---|---|
| `std.core` (prelude) | `Option`, `Result`, marker & operator interfaces, `Ordering`, `Iterator` adaptors (`map`, `filter`, `enumerate`, `zip`, `take`, `skip`, `chain`, `rev`, `sum`, `count`, `min_by`, `max_by`, `fold`, `any`, `all`, `find`, `position`, `collect[C]`), `Range*`, `print`/`println`/`eprintln`, `assert*`, `panic`, `todo`, `unreachable`, `mem.{take, replace, swap, drop, forget, size_of, align_of}` |
| `std.mem` | `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Layout`, `Allocator`, `Global`, `ptr.*` (unsafe pointer ops), `keep_alive` |
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
| `std.ffi` | `CString`, `cstr`, `c_int`… type aliases (`c_int` = target `int`), `Callback[F]`, `Retained[T]` (foreign-retained handle), `ForeignBox[T]` (owned foreign pointer with destructor fn), `NativeApiTable` helpers (Part XXI) |
| `std.gpu` | Part XVII (host-side; backend-agnostic) |
| `std.debug` | `backtrace`, `leak_report`, `alloc_stats`, `frame_profiler` markers (`zone("name")` scoped) |

`[STD-2]` `print` and friends allocate only for f-strings; `println("literal")` and `println(some_str)` are `@noalloc`.

---
