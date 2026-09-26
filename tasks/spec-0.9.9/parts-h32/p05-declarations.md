---

# Part V — Declarations

## V.1 Modules, packages, imports, visibility

* `[MOD-1]` A **package** is a directory tree with an `ember.toml` at its root. A **module** is one
  `.em` file. A module's path is the package name followed by its path under `src/`, with `/` replaced
  by `.` and the extension removed; `src/lib.em` or `src/main.em` is the package's root module, and
  `src/a/mod.em` is module `a` when `a/` has submodules. A single `.em` file with no manifest is a
  package of its own (`[CLI-4]`), and the directory containing it is its source root.
* `[MOD-2]` *(changed in 0.9.9)* Items are private to their module unless marked. `pub(package)` makes an
  item visible within the package; `pub` makes it visible to dependants. Using an item, field or
  constructor that is not visible is `E1052`, naming where it is declared and its visibility; `E1020`
  means only a name declared twice in one block (`[GRM-4]`).
* `[MOD-3]` *(changed in 0.9.9)* `import a.b.c` binds the name `c` to module `a.b.c`, and
  `import a.b.c as d` binds `d`. `from a.b import x, y as z` binds items. A standard module may be
  named without `std.`, as in Python: `import math` is `import std.math`, and `from random import Rng`
  is `from std.random import Rng`; when the package itself or one of its dependencies has that name,
  the package wins, with warning `W1003` naming both. A module path that names no module is
  `E1060 no module named 'a.b.c'` at the import, whose help lists the nearest existing module paths;
  an item missing from an existing module is `E1061`. Neither is ever deferred to the first use
  (`[PHIL-12]`).
* `[MOD-8]` *(new in 0.9.9)* `from m import *` binds every `pub` item of `m`. A name bound by two glob imports and then
  used is `E1031`, naming both sources; an explicit import or a local declaration shadows a glob.
* `[MOD-4]` Import cycles within a package are allowed; cycles between packages are `E1041`.
* `[MOD-7]` A field declared `pub(read)` (or `pub(package, read)`) may be read wherever a `pub`
  (`pub(package)`) field could be, and written only from its declaring module (`E1050` otherwise).
  Construction is a write, so a `pub(read)` field makes the memberwise constructor private to the
  module.
* `[MOD-5]` *(changed in 0.9.9)* **The prelude.** Every module implicitly imports these names from
  `std`, and this list is the only definition of the prelude:

  | Group | Names |
  |---|---|
  | Scalar aliases | `int` (`i64`), `float` (`f64`), `Never` |
  | Options and errors | `Option`, `Some`, `None`, `Result`, `Ok`, `Err`, `Error` (interface), `AnyError`, `RangeError` |
  | Text | `String`, `str`, `format` |
  | Collections | `Array`, `Map`, `Set`, `Span`, `MutSpan` |
  | Ranges | `Range`, `RangeInclusive`, `RangeFrom`, `RangeTo` |
  | Heap and cells | `Box`, `Shared`, `Weak`, `Cell`, `RefCell` |
  | Interfaces | `Copy`, `Clone`, `Drop`, `Default`, `Eq`, `Ord`, `Ordering`, `Hash`, `Debug`, `Display`, `Iterator`, `Iterable`, `IntoIterator`, `Generator`, `Contains`, `From`, `Send`, `Sync`, and the operator interfaces `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`, `Index`, `IndexMut`, `IndexSet`, with the `…Assign` forms of the arithmetic and bitwise interfaces |
  | Functions | `print`, `println`, `eprint`, `eprintln`, `input`, `assert`, `assert_eq`, `assert_ne`, `debug_assert`, `panic`, `todo`, `unreachable`, `min`, `max`, `abs`, `clamp`, and Python's `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` (`[STD-26]`) |
  | Modules | `mem` (the module `std.mem`) |

  A local declaration or explicit import shadows a prelude name. Everything else in the standard
  library is imported explicitly (`from std.math import Vec3`).

## V.2 Functions

```ember
pub fn blend(dst: Array[f32], mut out: Array[f32], owned scratch: Array[f32], weight: f32 = 0.5) -> int:
    for i in 0..out.len():
        out[i] = dst[i] * weight
    return scratch.len()
```

* `[FN-1]` *(changed in 0.9.9)* **Parameter modes.**
  * `a: A` — **borrowed** (the default). The callee reads the caller's `a`, not a copy of it, and
    cannot move it or assign through it; a write through a `Cell` or `RefCell` inside it changes the
    caller's value. Some parameters are passed as a copy instead, such as a small `Copy` value in
    registers (`[BRW-8]`); the meaning is the same.
  * `mut b: B` — **in-out**. The argument must be a mutable place, or a mutable view derived from one
    (`[FN-1a]`). The callee may change it; it cannot move it out except by `mem.replace`/`mem.take`.
  * `owned c: C` — **consumed**. The argument is moved in (copied if `Copy`; a handle is copied without
    a retain). The callee owns it.
* `[FN-1a]` A `mut` parameter whose type is itself a view (`MutSpan[T]`), the receiver included, accepts a view-producing
  expression such as `buf.as_mut_span()` or an `Array` place (coerced, `[SPN-1]`); the mutable-place
  requirement applies to the place the view is taken of.
* `[FN-2]` An omitted mode is borrowed. There is no by-copy mode; a callee that needs its own copy
  takes `owned` and the caller passes `x.clone()` (or `x`, if `Copy`).
* `[FN-2a]` **A call site never writes a mode.** `f(x)` is written whatever mode `f` declares; the
  compiler forms the borrow or move the declaration requires and reports shape B10 if `x` is not a
  suitable place. The editor shows the mode as an inlay hint.
* `[FN-9]` *(new in 0.9.9)* A mode on a **class-handle** parameter governs the handle, not the object. A borrowed handle
  parameter may still be used to read and write the object's fields and call its methods, subject to
  Part VIII's exclusivity rules; `owned` gives the callee a handle it owns — a handle made for the
  call (`take(Node(1))`) is moved in, and one read from a variable or field is a copy, retained
  (`[OWN-7]`), while the caller's own lives on to its end, so the callee's release never frees an
  object the caller still holds (`mem.drop(h)` ends the caller's early, `[OWN-6]`); a retain and
  release that cancel may be removed (`[RC-2]`) (ODR-051); `mut` lets the callee re-point the
  caller's handle, and opens no access on the object (`[EXC-15]` is `mut self`'s alone); a handle
  stored in an object is passed as a copy that is stored back after the call (`[RC-5]`, ODR-065).
* `[FN-3]` *(changed in 0.9.9)* A function returns by move. Returning a reference or view requires its region to derive
  from a parameter the result may borrow from (`[LT-1]`, `[LT-1a]`), or to be `static` (`[LT-3]`).
* `[FN-4]` A method's receiver follows the same modes: `self`, `mut self`, `owned self`. A function in
  a type body with no receiver is an associated function, called `Type.name(…)`.
* `[FN-5]` Default argument expressions are evaluated at each call, after the other arguments are
  bound, and may refer to earlier parameters.
* `[FN-6]` *(changed in 0.9.9)* **Callable types.** A function is a value. A callable type is written
  `fn(A, mut B, owned C) -> R`, with the same parameter modes as a declaration; `once fn(…) -> R` is a
  callable that may be called at most once (`[CLO-6]`). Callable types are compared by their parameter
  types, modes and result. A function coerces to any callable type it satisfies; a capture-free one
  also coerces to `extern "C" fn(…)` when every type is FFI-safe. The source parameters (`[LT-1]`) of a
  callable type get fresh regions at each call, and the result's regions follow `[LT-1]` applied to
  the callable type's own parameters (`[LT-7]`). What a callable type means in each position is
  `[CLO-3]`.
* `[FN-7]` Recursion is permitted; tail calls are not guaranteed to be eliminated.
* `[FN-8]` *(changed in 0.9.9)* `main` is `fn main()`, `fn main() -> Result[void, E]` for any
  `E: Display`, or either form with a first parameter `args: Array[String]`. `args` holds the command
  line, decoded as UTF-8 with invalid bytes replaced by U+FFFD; `std.process.args_os()` returns the raw
  bytes. An `Err` returned from `main` is printed with `Display` to standard error and the process
  exits with status 1. **Scripts:** the statements at file scope of the entry file (`[GRM-2]`) form, in
  source order, the body of an implicit `fn main()`, or `fn main() -> Result[void, AnyError]` when one
  of them uses `?`; items anywhere in the file are declared as usual. A file with such statements that
  also declares `main` has two items named `main` (`E1030`). The names the statements bind are locals
  of the implicit `main`: a function of the file that uses one is `E1010`, whose help names a
  parameter, a `const` or a `static` (unlike Python, where it would read a global). `return` and
  `yield` at file scope are `E0100`, as in Python; `process.exit(code)` ends a script early.
* `[FN-10]` *(new in 0.9.9)* A function whose return type is `Result[void, E]` returns `Ok(())` when
  control reaches the end of its body, as a `void` function returns; an explicit `return Ok(())` is
  still allowed. No other return type has an implicit value: a body that can reach its end is
  `E2182`, reported at the function's name (ODR-023).

## V.3 Structs

```ember
@derive(Copy)
pub struct Vec2:
    pub x: f32
    pub y: f32

    const ZERO: Vec2 = Vec2(0, 0)

    fn length(self) -> f32:
        return (self.x * self.x + self.y * self.y).sqrt()

    fn scale(mut self, k: f32):
        self.x *= k
        self.y *= k
```

* `[STR-1]` Every struct has a **memberwise constructor** `Name(field0, field1, …)` taking positional
  or named arguments in field order; a field with a default may be omitted. It is `pub` iff every
  field is `pub`. A declared `fn init(self, …)` replaces it (`[STR-6]`).
* `[STR-2]` *(changed in 0.9.9)* A field default is any expression; it is evaluated at each
  construction that omits the field, in field order, and may allocate (`items: Array[int] = []`).
* `[STR-3]` A struct with a `drop` method, or with a field that is not `Copy`, is move-only. `Copy` is
  never implicit: `@derive(Copy)` requests it, and is `E2080` if a field is not `Copy`. A field may
  need drop and still be `Copy` — a class handle, released when the copy is dropped (`[OWN-7]`).
* `[STR-4]` A struct may have no fields (`struct Marker: pass`).
* `[STR-5]` *(changed in 0.9.9)* **Implicit derives.** A struct or enum implements `Eq`, `Debug` and
  `Clone` field-wise, automatically, when every field implements the interface; `@no_derive(Eq)`
  (or `Debug`, `Clone`) opts out, and a hand-written implementation replaces the implicit one. A type
  that declares `drop` is not implicitly `Clone`: a field-wise copy would release twice what its
  `drop` releases, so it is `Clone` through `@derive(Clone)` or a written `clone` (ODR-026).
  `Copy`, `Ord`, `Hash`, `Default` and `Display` are never implicit and are requested with
  `@derive(…)` or written by hand.
* `[STR-7]` *(new in 0.9.9)* Inside the body of a `struct`, `enum`, `class` or `extend` block, `Self`
  names the type being declared with its own generic parameters, and the type's name may be written
  with any arguments: in `struct Pair[T]:`, a method may return `Self`, `Pair[T]` or `Pair[int]`.
* `[STR-6]` *(new in 0.9.9)* `fn init(self, …)` on a struct is its constructor: every field without a default MUST be
  assigned on every path before `self` is used as a whole (`E2100`). `mut self` is accepted in an
  `init` and means the same.

## V.4 Enums

```ember
@repr(u8)
enum RenderMode:
    Forward = 0
    Deferred = 1

enum Shape:
    Circle(radius: f32)
    Rect(w: f32, h: f32)
    Empty

    fn area(self) -> f32:
        return match self:
            Circle(r) => 3.14159 * r * r
            Rect(w, h) => w * h
            Empty => 0.0
```

* `[ENM-2]` A `match` on an enum MUST be exhaustive (`E2090` lists the missing variants); `_` covers
  the rest.
* `[ENM-3]` A unit-only enum is `Copy`, `Eq`, `Ord` (declaration order), `Hash` and `Debug`
  automatically, converts to its representation with `as`, and converts back with
  `Mode.from_repr(x) -> Option[Mode]`.
* `[ENM-4]` A payload enum is `Copy` only by `@derive(Copy)` with every payload `Copy`; `[STR-5]`'s
  implicit derives apply to it.

## V.5 Classes

```ember
open class Script:
    let entity: int
    pub(read) enabled: bool = true

    fn init(self, entity: int):
        self.entity = entity

    virtual fn on_update(self, dt: f32):
        pass

class Door(Script):
    angle: f32 = 0.0

    override fn on_update(self, dt: f32):
        self.angle = min(self.angle + 90.0 * dt, 90.0)
```

* `[CLS-1]` A class instance lives on the heap with the header of `[OBJ-1]`. `Name(args)` allocates,
  runs the constructor and returns a handle with strong count 1.
* `[CLS-2]` *(changed in 0.9.9)* `fn init(self, …)` is the constructor. Before any `init` body runs,
  every field default of the class and of its bases is evaluated and stored. Every field without a
  default MUST then be assigned on every path before `self` is used as a whole (`E2100`); `self` is
  used as a whole by a method call on it, by passing, storing or returning it, and by a closure
  capturing it. Several constructors are written as associated functions that call `Self(…)`.
* `[CLS-3]` A class with no `init` and no base class gets a memberwise constructor as a struct does.
* `[CLS-10]` *(new in 0.9.9)* A derived class with no `init` gets its base's constructor: `Door(entity)`
  evaluates `Door`'s field defaults and then runs `Script.init(entity)`. A derived class with a field
  that has no default MUST declare an `init` (`E2101`).
* `[CLS-4]` *(changed in 0.9.9)* A class is final unless declared `open` or `abstract`. Methods are
  non-virtual unless `virtual`; replacing one requires `override`; overriding a non-virtual method is
  `E2110`. A method with a receiver named like an inherited one replaces it (ODR-028): over a virtual
  method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or
  not; `init`, `drop` and associated functions are each class's own. An `override` is itself virtual,
  so a further subclass may override it again. A derived `init` MUST call `super.init(…)` exactly once
  (`[CLS-11]`). Fields cannot be overridden. Base fields are laid out first; there is at most one base.
* `[CLS-11]` *(new in 0.9.9)* Construction is two-phase. In a derived `init`, the code before
  `super.init(…)` MUST assign every field the class itself declares without a default, and MUST NOT
  read an inherited field or use `self` as a whole; the code after it may do anything (`E2102`,
  naming the field or the use). So when any method runs, even a `virtual` method called from a base
  `init`, every field of the object holds a value.
* `[CLS-5]` Interface calls on a class are dispatched statically unless the receiver is `dyn`.
* `[CLS-6]` Destruction runs the derived `drop`, then the base's, then drops the fields in reverse
  declaration order, then frees the memory once no weak handle remains.
* `[CLS-7]` *(changed in 0.9.9)* Inside a class method, `self` is a handle. **Any method may read and
  write the object's fields through `self`**; each access is checked on its own by Part VIII's rules,
  so a class method needs no `mut` to mutate its object (reference semantics, as in Python). `mut self`
  on a class method declares that the method holds a write access to every field of the object for its
  whole duration, which the compiler may use to remove the per-access checks inside it (`[EXC-15]`).
  `self` is the object the method was called on for the whole call: assigning to `self` is `E2103`, and
  a function that re-points a caller's handle takes it as a `mut` parameter (`[FN-9]`) (ODR-072).
* `[CLS-7a]` Inside `drop`, `self` MUST NOT be stored anywhere that outlives the call (`E3016`).
* `[CLS-8]` *(changed in 0.9.9)* A class is `Sync` only as `[THR-1]` allows; every field of a `Sync`
  class is immutable after `init`.
* `[CLS-9]` A `let` field is assignable only in `init`.
* `[CLS-9a]` A `let` field of a non-`Copy` type may still be mutated *through* (`self.items.push(v)`);
  `let` fixes the binding, not the value.

## V.6 Interfaces and `extend`

```ember
pub interface Shape2D:
    fn area(self) -> float
    fn describe(self) -> String:
        return f"shape with area {self.area():.2f}"

struct Square:
    side: float

extend Square implements Shape2D:
    fn area(self) -> float:
        return self.side * self.side
```

* `[IFC-1]` *(changed in 0.9.9)* `extend T:` without `implements` adds inherent methods to `T`; it is
  legal in any module of `T`'s package.
* `[IFC-2]` Adding inherent methods to a type from another package is `E2120`; use an interface or a
  wrapper type.
* `[IFC-3]` *(changed in 0.9.9)* `interface Ord: Eq` requires implementers of `Ord` to implement `Eq`. So a bound brings
  its parents: `T: IndexMut[int]` provides `Index[int]`'s methods too, and a binding it writes
  (`Output = f32`) names the parent's associated type (ODR-042).
* `[IFC-4]` *(changed in 0.9.9)* Interfaces may declare associated types and constants, not statics.
  An associated type may carry bounds (`type Real: Float`), and a type implementing the interface
  states it (`type Real = f64`) with a value that meets them (`E2040` if it is missing or does not).
  `T.Name` names the associated type `Name` of a type parameter `T` whose bound declares it: inside
  the generic body it has what its declared bounds provide, and each call takes it from what the
  argument's type states (ODR-038). An `extend` block that implements several interfaces states an
  associated type once, for each of them that declares it (`extend i32 implements Add, Sub: type
  Output = i32`). `Name = T` inside an interface's brackets is written only in a bound (`T: Add[Output
  = T]`), and is `E2020` in an `implements` list. A bound that leaves an associated type unsaid
  (`T: Add`) makes `a + b` a `T.Output`, which only a signature naming it (`-> T.Output`) can hold
  (`E2040` otherwise) (ODR-040). An interface may give an associated type a **default** (`type Out =
  Self`): an implementation that does not state it takes the default, read with `Self` as the
  implementing type, and the default must meet the associated type's bounds (`E2040`). A default is
  not an equality: a bound `T: I` leaves `T.Out` abstract whatever the default says, and a default
  method body sees `Out` as itself, never as its default, since an implementation may override it
  (ODR-049). A default that leads back to itself, directly or through other defaults, is `E2043`.

## V.7 Constants and statics

* `const NAME: T = e` is a compile-time constant, inlined at each use; `e` MUST be evaluable at compile
  time (Part XIV). Written without `: T` and with an untyped numeric literal as `e`, it is untyped
  (ODR-037): each use is that literal and takes its type as the literal would there (`[TYP-23]`),
  `int` or `float` where nothing fixes it. `T` MUST be `Copy`, `str`, a `bytes` literal type, or a fixed array of those
  (`E2130` otherwise, whose help suggests a `static`). A `const` in the body of a `struct`, `enum`,
  `class` or `extend` block belongs to the type: it is named `T.NAME`, or `Self.NAME` inside the
  type, and is private to its module unless `pub` (`[MOD-2]`). Constants may name one another in
  any order; one whose value depends on itself is `E6001`, and a panic while one is evaluated is
  `E6004` (`[CT-7]`) (ODR-045).
* `[STA-1]` *(changed in 0.9.9)* `static NAME: T = e` is one value per program with a stable address.
  A `static` is immutable; mutation goes through a `Sync` interior type (`Atomic`, `Mutex`, `RwLock`,
  `Once`). `T` MUST be `Sync` (`E7002`). `static mut` exists only for foreign interfaces and every
  access to it requires `unsafe`.
* `[STA-3]` *(new in 0.9.9)* A static whose initialiser can be evaluated at compile time is placed in the image with no
  initialisation code. Any other initialiser runs **once**, on first access, exactly once even under
  concurrent first access; the first access blocks until it completes. A panic during initialisation
  aborts. An initialiser that reaches its own static is `static initialisation cycle`, a panic naming
  the chain.

```ember
from std.sync import Mutex

static GREETING: str = "hello"
static REGISTRY: Mutex[Map[String, int]] = Mutex({})
```

* `[STA-2]` There is no static-initialisation-order problem: statics are initialised at compile time
  or on first access, never by a global constructor.

## V.8 Attributes

* `[ATT-1]` *(changed in 0.9.9)* An attribute that is neither in the table below nor a visible
  `@attribute` struct (`[RFL-3]`) is `E0104`. An attribute on a declaration it does not apply to is
  `E0104` naming the positions it does apply to.
* `[ATT-4]` One attribute per line.
* `[ATT-6]` *(new in 0.9.9)* Every attribute in the table has exactly the effect its rule gives. An implementation that
  has not built an attribute's effect rejects the attribute with `E0900` (`[PHIL-12]`); it never
  accepts and ignores it.
* `[ATT-2]` A statement may carry only `@parallel`, `@unroll`, `@simd` and `@allow`; the first three
  only on a `for` statement (`E0108`).

| Attribute | Applies to | Rule |
|---|---|---|
| `@derive(A, …)`, `@no_derive(A, …)` | struct, enum, class | `[STR-5]`, `[DRV-1]` |
| `@layout(c)`, `@packed`, `@align(N)`, `@repr(T)` | struct, enum | `[LAY-2]` |
| `@gpu_layout(std140\|std430\|scalar)` | struct | Annex D |
| `@noalloc`, `@nosync`, `@noblock`, `@noio`, `@nolock`, `@nopanic(explicit)`, `@static_safe`, `@deterministic`, `@realtime` | fn, interface method; `@deterministic` also module and fn type | Part X |
| `@overflow(panic\|wrap\|saturate)` | fn, module | `[TYP-8]` |
| `@fastmath`, `@fp(contract)` | fn | `[TYP-9]` |
| `@inline`, `@noinline`, `@cold`, `@hot` | fn | `[CG-C-3a]` |
| `@simd`, `@simd(assert)`, `@parallel(…)`, `@unroll(N)` | `for` statement | Part XII |
| `@must_use` | fn, type | `[ERR-5]` |
| `@deprecated(since, note)` | any item | `[VER-3]` |
| `@export("symbol")`, `@export_table("Name", protocol=N)` | fn, static; struct | `[FFI-26]` |
| `@ffi(…)` | extern item, overlay item | Part XVI |
| `@test`, `@bench`, `@should_panic` | fn | `[TST-3]` |
| `@view` | struct, enum | `[TYP-34]` (documentation only) |
| `@sync` | class | `[THR-1]` |
| `@reflect` | type | `[RFL-2]` |
| `@borrows(p, …)` | fn | `[LT-1a]` |
| `@safety("…")` | unsafe fn | `[UNS-7]` |
| `@must_drop` | struct, class | `[THR-6]` |
| `@allow(code, …)` | any item or statement | suppresses the named `W`/`L` diagnostics |
| `@non_exhaustive` | enum | `[FFI-8]` |
| `@component(layout=soa\|aos)`, `@soa(flatten)` | struct; field | Part XII |
| `@always_specialize`, `@never_specialize` | generic fn or type | `[MONO-7]` |
| `@reloadable`, `@noreload`, `@renamed_from("n")`, `@reinit_on_reload`, `@allow_reload_terminate` | see Annex B | Annex B |
| `@prelude` | module (standard library only) | `[MOD-5]` |
| `@attribute` | struct | `[RFL-3]` |
| `@error("…")`, `@from` | variant of an enum deriving `Error` | `[ERR-3]` |
| `@default` | variant of an enum deriving `Default` | `[DRV-1]` |
| `@ser(skip\|rename="…"\|default\|since=N)` | field of a type deriving `Serialize` | `[SER-1]` |
| `@grade(…)` | overlay item | `[TCB-1]` |
| `@assume_noalloc(call)` | a call inside `unsafe` (an expression) | `[EFF-7]` |

*Reserved* (recognised and rejected with `E0104` naming the version): `@nopanic` (without
`(explicit)`), `@no_runtime_checks`, `@allocator(Name)`, `@gpu`.
