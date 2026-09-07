# Part XIV — Compile-Time Programming

## XIV.1 `comptime`

* `[CT-1]` `comptime:` blocks and `comptime fn` functions are executed by the **MIR interpreter** (Part XVIII §7) during compilation. Any Ember function whose transitive effect set ⊆ `{Panic}` and that uses only `comptime`-supported operations may be called at compile time — there is no separate sub-language. Supported: all arithmetic, structs, enums, `Array`/`String`/`Map` (interpreted heap), `match`, loops, recursion, `assert`; the interpreter emulates target endianness, pointer size and `@layout(c)`.
* `[CT-2]` Not supported at compile time: FFI, threads, `unsafe` raw-pointer deref into non-interpreter memory, I/O except `comptime.read_file(path)` (path relative to the package; recorded as a build dependency) and `comptime.env(name)`.
* `[CT-3]` Limits: 10^8 MIR steps and 256 MB interpreter heap per `comptime` evaluation by default (`ember.toml [comptime]`); exceeding is `E6001`.
* `[CT-4]` `comptime` values are hashed into the module cache key with their inputs; determinism is required (`E6002` if two evaluations of the same block differ — checked in `--verify-comptime` CI mode).
* `[CT-5]` Results of `comptime` blocks are materialised as `static` data (arrays, strings, structs) — no runtime initialisation code is emitted.

```ember
const SIN_TABLE: [f32; 256] = comptime:
    t = [0.0; 256]
    for i in 0..256:
        t[i] = sin(i as f32 / 256.0 * TAU)
    t

comptime:
    assert size_of[Vertex]() == 32, "Vertex layout changed; update the shader"
```

## XIV.2 Reflection

* `[RFL-1]` `reflect[T]()` (comptime) returns a `TypeDesc`: `{name, kind, size, align, fields: [FieldDesc{name, type: TypeDesc, offset, attrs}], variants, methods (v2), attributes}`.
* `[RFL-2]` Runtime reflection is opt-in with `@reflect` on the type: the compiler emits a `TypeInfo` table entry with field descriptors accessible via `type_info_of[T]()` / `h.type_info()` for class handles. Unused runtime metadata is dead-stripped by the linker (each table is its own section/COMDAT).
* `[RFL-3]` `@reflect` fields may carry user attributes readable at runtime (`@ragev.field(range=(0, 1), tooltip="…")`) — the editor bridge in Part XXI uses this exactly as RageV's `RVShowInEditor` markers are used by `rvgen` today.

## XIV.3 Derives

`@derive(...)` invokes compiler-built-in generators. v1 set: `Copy, Clone, Debug, Display(field="…")`, `Eq, Ord, PartialOrd, Hash, Default, SoA, Component, Error, Serialize, Deserialize, Zeroable, Reflect`. `[DRV-1]` Each derive is specified as an equivalent hand-written `extend` block in `std/derive/*.em` (the reference), and the generator MUST produce the same MIR. `[DRV-2]` User-defined derives (procedural macros) are **v2**; the mechanism will be a `comptime fn derive_X(t: TypeDesc) -> Source` sandboxed in the interpreter.

## XIV.4 Serialization

`@derive(Serialize, Deserialize)` generate `fn serialize(self, mut w: dyn Writer) -> Result[void, SerError]` / `fn deserialize(mut r: dyn Reader) -> Result[Self, SerError]` using a **binary, versioned, field-tagged** format (`std.ser.binary`) and a YAML mapping (`std.ser.yaml`, to interoperate with RageV's `yaml-cpp` scene files). Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(version=2)`.

---

