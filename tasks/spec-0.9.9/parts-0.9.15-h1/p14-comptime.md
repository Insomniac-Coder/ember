---

# Part XIV — Compile-Time Programming

Ember runs ordinary Ember code at compile time. There is no second language for it, no macro system,
and no `comptime fn` marker: a function can run at compile time when what it does can.

```ember,fragment
from std.math import TAU
import math.det

@layout(c)
struct Vertex:
    pos: [f32; 3]
    normal: [f32; 3]
    uv: [f32; 2]

fn sine_table() -> [float; 256]:
    t = [0.0; 256]
    for i in 0..256:
        t[i] = det.sin(TAU * i as float / 256.0)
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
* `[CT-2]` *(changed in 0.9.9)* *(changed in 0.9.10)* An operation that cannot run at compile time — an `extern` call, a
  thread, a clock, `unsafe` pointer access outside the evaluator's heap, I/O — is `E6010`, naming the
  call chain. A raw pointer made from an integer (`0 as *mut T`, `-1 as *mut T`) is a compile-time
  value: it is an address, read by nothing, and a `const` of pointer type may hold one; a pointer into
  the evaluator's heap may not leave it (`E6010`) (ODR-145). Two inputs are provided: `comptime.read_file(path)` (relative to the package root, and only inside the
  package directory, `E6010` otherwise, so a dependency's build cannot read the builder's files) and
  `comptime.env(name)`. Both are recorded as build inputs.
* `[CT-3]` Each evaluation is limited to 10⁸ steps and 256 MB of evaluator heap by default
  (`[comptime]` in the manifest); exceeding either is `E6001`.
* `[CT-4]` *(changed in 0.9.9)* *(changed in 0.9.10)* Compile-time evaluation is deterministic by construction: the evaluator
  has no source of `Nondet` (`[DET-2]`). A transcendental function runs at compile time when the
  source names `std.math.det`'s, with the same result it gives at run time; the platform library's
  `sin` is `Nondet` and does not run at compile time, and the evaluator never puts `det.sin` in its
  place (ODR-056). The evaluator is the compiler's own (`[CLI-21]`): a `const` value is never computed
  by building and running a C program, and the test suite runs every program both compiled and on the
  evaluator and fails on any difference. The evaluator computes integer and floating-point results exactly as the target does, including overflow panics
  and the target's pointer size, endianness and layout. The lexical overflow policy is the same
  at compile time and run time (`[TYP-8]`, ODR-084), including module constants and arithmetic
  inside a function; calling another function preserves that function's policy. A result is cached with its inputs (source,
  `read_file` contents, `env` values).
* `[CT-5]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Where results live.** A compile-time result is placed in the image as
  constant data. A value that owns heap memory (`Array`, `String`, `Map`, an `owns T`, …) is handled so
  that no run-time code ever grows, changes or frees constant data:
  * in a `static` (not `static mut`) whose type gives no name a way to change the value (no `Mutex`,
    `RwLock`, `Cell`, `Atomic` or `guarded by` field on the path to it), the value is materialised in
    read-only memory; statics are never dropped;
  * in a `static mut`, and in any other `static` whose memory a safe operation may write
    (`static LOG: Mutex[Array[String]] = Mutex([])`), the initialiser runs at run time on first access
    (`[STA-3]`), producing an ordinary heap value, so `static mut LOG: Array[String] = [String("start")]`
    may later grow, or replace its strings, without touching constant data;
  * a `comptime(e)` whose type owns heap memory is materialised once, as constant data, and the
    expression is a read-only view of it, with no copy: `t = comptime(table())` then `t[i]` reads the
    table made when compiling, and a change through `t` is `E3023`. `copy of comptime(e)` makes a value
    of your own, which may grow (`Alloc`).

  A `const` cannot own heap memory (§V.7).

## XIV.2 Reflection

* `[RFL-1]` `reflect[T]()` at compile time returns a `TypeDesc`: name, kind, size, alignment, fields
  (name, type, offset, attributes), variants, and attributes. It is not available at run time.
* `[RFL-2]` *(changed in 0.9.15)* `@reflect` on a type emits run-time type information: `type_info[T]()`, and
  `h.type_info()` on a class object, return a `ref TypeInfo` with the same field descriptors. Tables
  no code reaches are removed at link time.
* `[RFL-3]` *(changed in 0.9.9)* **User attributes are declared.** A struct marked `@attribute` may be
  written as an attribute on the fields and declarations of `@reflect` types: `@Range(min=0.0,
  max=1.0)` is checked exactly like the constructor call `Range(min=0.0, max=1.0)`, and its value is
  readable through `TypeDesc` and `TypeInfo`. An attribute that names no built-in attribute and no
  visible `@attribute` struct is `E0104` (`[PHIL-12]`).

`std.ser` declares the descriptions (ODR-158): `TypeDesc` (`name`, `kind: TypeKind`, `size`, `align`,
`fields: Array[FieldDesc]`, `variants: Array[VariantDesc]`, `attributes: Array[AttributeDesc]`), and
`TypeInfo` is the same type. A description owns its text (`String`), so it is no view and lives in a
`static`; it owns arrays, so it is no `const` (`static D: TypeDesc = reflect[Point]()`). A field names
its type by name (`type_name`): a description of each field's type would not end for a recursive
type. A class field's offset is counted from the end of the object header, which the runtime owns
(`[OBJ-1]`). An attribute is its name and its arguments as `(name, AttrValue)` pairs, `AttrValue`
being `Int(i64)`, `Float(f64)`, `Str(String)`, `Bool(bool)` or `Char(char)`: a user attribute's
argument is named by the field it sets, a built-in attribute's positional argument by its position,
and a name written as an argument (`@derive(Eq)`) is its text. An `@attribute` struct's fields are
of those types (`E0104` otherwise). `@reflect` implements `std.ser.Reflect`, which `type_info[T: Reflect]()`
asks for (`E2040` without it), and a class derived from a `@reflect` class is `@reflect` too;
`reflect[T]()` outside what compile time evaluates is `E6010`.

`std.ser` declares the serialisation interfaces (ODR-159): `Writer` (a struct's `begin_struct(name,
version)`, `field(name)` before each field's value and `end_struct()`; an enum value's
`begin_variant(name)` and `end_variant()`; `begin_list(count)`/`end_list()`; `write_none()`,
`write_int`, `write_float`, `write_bool`, `write_str`) and `Reader` (the same, reading: `begin_struct()`
gives the version and `next_field()` each field's name in the data's order, `None` after the last;
`next_element()`, `read_none()`, `skip_value()`). A struct's version is the highest `@ser(since=N)` of its
fields, 0 with none. `std.ser.json` writes a struct as an object of its fields, `"$version"` first when
above 0; an enum value as an object whose `"$variant"` names the variant, followed by its payload's
fields; `None` as `null`. `std.ser.binary` is self-describing, a tag byte before each value, so a reader
skips a field it does not know. A field missing from the data takes its type's `Default` under
`@ser(default)` or `@ser(since=N)`.

## XIV.3 Derives

* `[DRV-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* `@derive(…)` requests generated implementations. The built-in derives,
  and exactly what each generates, are:

  | Derive | Generates |
  |---|---|
  | `Copy` | the marker, and `Clone` with it, so `copy of` works on the type; every field MUST be `Copy`, the type MUST have no `drop`, and it MUST be a struct or an enum, since a class object is never `Copy` (`E2080`) |
  | `Clone` | field-wise clone (implicit per `[STR-5]`), which `copy of` runs: a field the value owns alone is cloned itself, and a link field is copied as one more link to the same target (a `shares` or `shares mut` field one more owner, a `weakref` field one more `weakref` link, a `sharedref mut` field one more holder) |
  | `Eq` | field-wise equality in declaration order; enums compare variant first (implicit per `[STR-5]`) |
  | `Debug` | `Point(x=1.0, y=2.0)` for a struct, `Shape.Circle(r=1.0)` for a variant, `Mode.Fast` for a unit variant; strings quoted (implicit per `[STR-5]`) |
  | `Display` | on a unit-only enum, the variant name; on a struct or payload enum, the `Debug` text |
  | `Ord` | lexicographic over fields in declaration order, each by `cmp`, the next only when it gives `Equal`; enums by variant order, then fields |
  | `Hash` | feeds each field in declaration order; enums feed the variant index first |
  | `Default` | each field's declared default, else its type's `Default`; for an enum, the variant marked `@default` |
  | `Error` | `[ERR-3]` |
  | `Serialize`, `Deserialize` | §XIV.4 |
  | `Zeroable` | `[ARN-11]` |

  A derive whose requirement a field does not meet is `E2080`, naming the field. The generated code
  is ordinary Ember that `ember inspect --expand <type>` prints.
* `[DRV-2]` User-defined derives are not in this version; `macro` is reserved for them.

## XIV.4 Serialisation (`std.ser`)

* `[SER-1]` *(new in 0.9.9)* *(changed in 0.9.15)* `@derive(Serialize, Deserialize)` implements the method
  `[W: ser.Writer]` above `fn serialize(ref mut w: W) -> Result[void, SerError]` and the function
  `[R: ser.Reader]` above `static fn deserialize(ref mut r: R) -> Result[Self, SerError]`, over a
  field-tagged, versioned format; both are monomorphised per format, and each call says `ref mut`
  before the writer or reader (`p.serialize(ref mut w)`, `Point.deserialize(ref mut r)`).
  `std.ser.binary` and `std.ser.json` provide a writer and reader each. Field attributes: `@ser(skip)`, `@ser(rename="…")`,
  `@ser(default)`, `@ser(since=N)`.
* `[SER-2]` *(new in 0.9.9)* Deserialising never produces an invalid value: a range-typed field is
  checked (`[RNG-10a]`), an unknown enum tag is an error, and a missing field without `@ser(default)`
  or `@ser(since=…)` is an error.
