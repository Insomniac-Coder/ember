---

# Part XIV — Compile-Time Programming

Ember runs ordinary Ember code at compile time. There is no second language for it, no macro system,
and no `comptime fn` marker: a function can run at compile time when what it does can.

```ember
from std.math import sin, TAU

@layout(c)
struct Vertex:
    pos: [f32; 3]
    normal: [f32; 3]
    uv: [f32; 2]

fn sine_table() -> [float; 256]:
    t = [0.0; 256]
    for i in 0..256:
        t[i] = sin(TAU * i as float / 256.0)
    return t

const SINE: [float; 256] = comptime(sine_table())

comptime:
    assert(mem.size_of[Vertex]() == 32, "Vertex layout changed; update the shader")
```

## XIV.1 Compile-time evaluation

* `[CT-1]` *(changed in 0.9.9)* These are evaluated during compilation: `comptime(e)`; `comptime:`
  blocks; `const` initialisers; array lengths and `const` generic arguments; and `static`
  initialisers that qualify under `[STA-3]`. Any function may be called there if its effect set
  contains nothing beyond `Alloc`, `Panic(Explicit)` and `RuntimeCheck` — no `Io`, `FFI`, `Sync`,
  `Block`, `Unsafe` or `Nondet`. Allocation is served by the evaluator's own heap.
* `[CT-6]` *(new in 0.9.9)* `comptime(e)` is an expression whose value is `e` evaluated at compile
  time. `e` may refer to constants, generic parameters, and statics initialised at compile time, not
  to run-time locals (`E6005`).
* `[CT-7]` *(new in 0.9.9)* A `comptime:` block at item level, or as a statement, runs once during
  compilation for its checks. A failed `assert` or any other panic there is `E6004`, which shows the
  panic message and the compile-time call chain.
* `[CT-2]` *(changed in 0.9.9)* An operation that cannot run at compile time — an `extern` call, a
  thread, a clock, `unsafe` pointer access outside the evaluator's heap, I/O — is `E6010`, naming the
  call chain. Two inputs are provided: `comptime.read_file(path)` (relative to the package root) and
  `comptime.env(name)`. Both are recorded as build inputs.
* `[CT-3]` Each evaluation is limited to 10⁸ steps and 256 MB of evaluator heap by default
  (`[comptime]` in the manifest); exceeding either is `E6001`.
* `[CT-4]` *(changed in 0.9.9)* Compile-time evaluation is deterministic by construction: the evaluator
  has no source of `Nondet` (`[DET-2]`), uses `std.math.det` for transcendental functions, and
  computes integer and floating-point results exactly as the target does, including overflow panics
  and the target's pointer size, endianness and layout. A result is cached with its inputs (source,
  `read_file` contents, `env` values).
* `[CT-5]` *(changed in 0.9.9)* **Where results live.** A compile-time result is placed in the image as
  constant data. A value that owns heap memory (`Array`, `String`, `Map`, `Box`, …) is handled so that
  no run-time code ever grows or frees constant data:
  * in a `static` whose type gives Safe code no way to obtain `ref mut` to the value (no `Mutex`,
    `RwLock`, `Cell`, `RefCell` or `Atomic` on the path to it), the value is materialised in
    read-only memory; statics are never dropped;
  * in any other `static` (`static LOG: Mutex[Array[String]] = Mutex([])`), the initialiser runs at
    run time on first access (`[STA-3]`), producing an ordinary heap value;
  * a `comptime(e)` whose type owns heap memory is materialised once, and each run-time evaluation of
    the expression **clones** it into a fresh allocation (`Alloc`), so the program owns an ordinary
    value it may grow. Binding the value to a `static` and borrowing it avoids the copy.

  A `const` cannot own heap memory (§V.7).

## XIV.2 Reflection

* `[RFL-1]` `reflect[T]()` at compile time returns a `TypeDesc`: name, kind, size, alignment, fields
  (name, type, offset, attributes), variants, and attributes. It is not available at run time.
* `[RFL-2]` `@reflect` on a type emits run-time type information: `type_info[T]()`, and
  `h.type_info()` on a class handle, return a `ref TypeInfo` with the same field descriptors. Tables
  no code reaches are removed at link time.
* `[RFL-3]` *(changed in 0.9.9)* **User attributes are declared.** A struct marked `@attribute` may be
  written as an attribute on the fields and declarations of `@reflect` types: `@Range(min=0.0,
  max=1.0)` is checked exactly like the constructor call `Range(min=0.0, max=1.0)`, and its value is
  readable through `TypeDesc` and `TypeInfo`. An attribute that names no built-in attribute and no
  visible `@attribute` struct is `E0104` (`[PHIL-12]`).

## XIV.3 Derives

* `[DRV-1]` *(changed in 0.9.9)* `@derive(…)` requests generated implementations. The built-in derives,
  and exactly what each generates, are:

  | Derive | Generates |
  |---|---|
  | `Copy` | the marker; every field MUST be `Copy` and the type MUST have no `drop` (`E2080`) |
  | `Clone` | field-wise clone (implicit per `[STR-5]`) |
  | `Eq` | field-wise equality in declaration order; enums compare variant first (implicit per `[STR-5]`) |
  | `Debug` | `Point(x=1.0, y=2.0)` for a struct, `Shape.Circle(r=1.0)` for a variant, `Mode.Fast` for a unit variant; strings quoted (implicit per `[STR-5]`) |
  | `Display` | on a unit-only enum, the variant name; on a struct or payload enum, the `Debug` text |
  | `Ord` | lexicographic over fields in declaration order; enums by variant order, then fields |
  | `Hash` | feeds each field in declaration order; enums feed the variant index first |
  | `Default` | each field's declared default, else its type's `Default`; for an enum, the variant marked `@default` |
  | `Error` | `[ERR-3]` |
  | `Serialize`, `Deserialize` | §XIV.4 |
  | `Zeroable` | `[ARN-11]` |
  | `Component` | `[ECS-7]` |

  A derive whose requirement a field does not meet is `E2080`, naming the field. The generated code
  is ordinary Ember that `ember inspect --expand <type>` prints.
* `[DRV-2]` User-defined derives are not in this version; `macro` is reserved for them.

## XIV.4 Serialisation (`std.ser`)

* `[SER-1]` *(new in 0.9.9)* `@derive(Serialize, Deserialize)` implements `serialize[W: ser.Writer](self,
  mut w: W) -> Result[void, SerError]` and `deserialize[R: ser.Reader](mut r: R) -> Result[Self,
  SerError]` over a field-tagged, versioned format; both are monomorphised per format.
  `std.ser.binary` and `std.ser.json` provide a writer and reader each. Field attributes: `@ser(skip)`, `@ser(rename="…")`,
  `@ser(default)`, `@ser(since=N)`.
* `[SER-2]` *(new in 0.9.9)* Deserialising never produces an invalid value: a range-typed field is
  checked (`[RNG-10a]`), an unknown enum tag is an error, and a missing field without `@ser(default)`
  or `@ser(since=…)` is an error.
