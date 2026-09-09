# Part V — Declarations and Semantics

## V.1 Modules, packages, visibility

* `[MOD-1]` A **package** is a directory tree with an `ember.toml` at its root. A **module** is one `.em` file. Module path = package name + path from `src/` with `/` → `.` and the extension removed; `src/lib.em` (library root) or `src/main.em` (binary root) is the package root module; `src/math/mod.em` is module `math` when a directory has submodules.
* `[MOD-2]` All items are private to their module unless `pub`. `pub(package)` is visible within the package; `pub` is visible to dependants. There is no `protected`; subclasses in other modules see only `pub` members.
* `[MOD-3]` `import a.b.c` binds `c` as a namespace; `from a.b import x, y as z` binds items; `import a.b.c as d` renames. `from a.b import *` is permitted only for `prelude` modules declared `@prelude` (`E1040` otherwise).
* `[MOD-4]` Import cycles within a package are allowed (name resolution is package-wide); cycles between packages are `E1041`.
* `[MOD-5]` `std.prelude` is imported implicitly into every module: `Option, Some, None, Result, Ok, Err, Array, String, str, Span, MutSpan, Box, Shared, Weak, print, println, assert, assert_eq, panic, Copy, Clone, Drop, Eq, Ord, Hash, Debug, Display, Default, Iterator, Iterable, Send, Sync`.
* `[MOD-6]` A module MAY declare `#! language "0.5"` on its first line; the package's `ember.toml` `language` key is the default. Mismatch with the compiler's supported set is `E0006`. The compiler's supported set MUST include every language version whose source it still accepts, so pinning an older version stays valid. As of this revision that set includes `"0.8.4"`, which selects the 0.8.4 language, and `"0.8.3"`, which stays valid because 0.8.4 is additive over it — the rule above already required this and the example is here so an implementer does not have to derive the currently accepted set from the lineage. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[MOD-7]` **Read-only visibility for fields.** A field declared `pub(read)` (or `pub(package, read)`) may be **read** wherever a `pub` (respectively `pub(package)`) field could be read, but may be **written only from the declaring module**. Outside the declaring module, the following are errors `E1050 field is read-only outside its module`: assignment (`h.value = x`, augmented assignment), taking `ref mut h.value`, passing `h.value` to a `mut` parameter or `mut self` method, and destructuring it with a mutable binding. Reading, copying, taking `ref h.value`, and passing it to a borrowed parameter are allowed. `read` applies to fields of structs and classes only; on any other item it is `E1051`. Because construction is a write, a `pub(read)` field does not count as `pub` for the purpose of the synthesised memberwise constructor (`[STR-1]`).

  Summary of field visibility:

  | Declaration | Read from | Write from |
  |---|---|---|
  | `value: T` | declaring module | declaring module |
  | `pub(package) value: T` | package | package |
  | `pub(package, read) value: T` | package | declaring module |
  | `pub value: T` | anywhere | anywhere |
  | `pub(read) value: T` | anywhere | declaring module |
  | `let value: T` (any visibility) | per the marker | nobody after `init`; not even the declaring module |

## V.2 Functions

```ember
pub fn name[T: Bound](a: A, mut b: B, owned c: C, d: D = default) -> R where T: Other:
    body
```

* Parameter modes `[FN-1]`:
  * `a: A` — **borrowed** (shared). The callee reads through a `ref A`. For `Copy` types smaller than 2 pointers the compiler passes by value in registers (ABI detail; semantics identical). The callee cannot mutate or move `a`.
  * `mut b: B` — **inout** (mutable borrow). The argument MUST be a mutable place; the callee may mutate; no move out (except by `mem.replace`/`take`).
  * `owned c: C` — **consumed**. The argument is moved (or copied if `Copy`; retained if a handle). The callee owns it and will drop it or move it on.
  * `[FN-2]` Missing mode is `borrowed`. There is no by-value-copy mode; if the callee wants its own copy it writes `owned` and the caller writes `f(x.clone())` or `f(x)` for `Copy` types.
* `[FN-3]` Return values are moved out; returning a `ref`/view requires that the region be tied to a parameter by elision (Part VII §5).
* `[FN-4]` The receiver `self` follows the same modes: `self` (borrow), `mut self` (mutable borrow), `owned self` (consume). Absent `self` ⇒ associated function. `self: Type` explicit form is allowed for `self: ref Self`, `self: Box[Self]`, `self: Shared[Self]` (v2 for the latter two).
* `[FN-5]` Default arguments are evaluated in the callee's scope at each call, after positional/named binding. They may reference earlier parameters. They MUST be `@noalloc`-clean if the function is `@noalloc`.
* `[FN-6]` Functions are values of a unique zero-sized function type; they coerce to `fn(A) -> R` (the generic callable bound) and, if they capture nothing and have no generic parameters, to `extern "C" fn(A) -> R` when their types are FFI-safe.
* `[FN-7]` Recursion is permitted; the compiler does not guarantee tail-call elimination in v1.
* `[FN-8]` `main` is `fn main()`, `fn main() -> Result[void, E]` (`E: Error`), or `fn main(args: Span[str])` variants. A non-`Ok` result prints the error with `Display` to stderr and exits with code 1.
* `[FN-2a]` The compiler forms the borrow the callee's declared mode requires. If the argument is not a suitable place for that mode, it reports shape **B10** naming the place required. **A call site never writes the mode** (owner decision `OQ-13`): `f(x)` is written whether `f` declares `x`, `mut x` or `owned x`, preserving Part 0 row 3's guarantee that call sites never carry a sigil. The mode is read from the callee's signature, and `ember inspect` and the editor's inlay hints (`[IDE-7]`) surface it at the argument.

## V.3 Structs

```ember
@derive(Copy, Debug, Eq)
pub struct Vec3:
    pub x: f32
    pub y: f32
    pub z: f32

    const ZERO: Vec3 = Vec3(0, 0, 0)

    fn length(self) -> f32:
        return sqrt(self.dot(self))

    fn dot(self, o: Vec3) -> f32:
        return self.x*o.x + self.y*o.y + self.z*o.z
```

* `[STR-1]` Every struct has a synthesised **memberwise constructor** `Name(field0, field1, …)` accepting positional or named arguments; fields with defaults may be omitted. It is `pub` iff all fields are `pub` (a `pub(read)` field makes it private to the declaring module, since construction is a write; `[MOD-7]`). A user-defined `fn init(mut self, …)` replaces it (Part V.5).
* `[STR-2]` Field defaults are const-evaluable expressions or calls to `Default.default()`.
* `[STR-3]` A struct with a `drop` method, or any `Drop` field, is move-only. `Copy` is derived otherwise via `@derive(Copy)` (never implicitly, so that adding a field later cannot silently change semantics — `E2080` if `@derive(Copy)` is present but a field is not `Copy`).
* `[STR-4]` Zero-sized structs are permitted (`struct Marker: pass`).
* `[STR-5]` Struct equality/ordering/hash are never implicit; `@derive(Eq, Ord, Hash)` generates field-wise implementations.

## V.4 Enums

```ember
@repr(u8)
enum RenderMode:
    Forward = 0
    Deferred = 1
    PathTrace = 2

enum Shape:
    Circle(radius: f32)
    Rect(w: f32, h: f32)
    Empty

    fn area(self) -> f32:
        match self:
            Circle(r) => return PI * r * r
            Rect(w, h) => return w * h
            Empty => return 0
```

* `[ENM-1]` Variant constructors are `Shape.Circle(1.0)` or `Shape.Circle(radius=1.0)`; with `from Shape import *`-style implicit scope inside `match`, bare `Circle(r)` patterns are accepted when the scrutinee's type is known.
* `[ENM-2]` `match` on an enum MUST be exhaustive (`E2090` lists the missing variants). `_` is the catch-all.
* `[ENM-3]` Unit-only enums implement `Copy, Eq, Hash, Debug` automatically and support `as` to their repr integer; the reverse uses `Mode.from_repr(x) -> Option[Mode]`.
* `[ENM-4]` Payload enums derive `Copy` only via `@derive(Copy)` with all payloads `Copy`.

## V.5 Classes

```ember
open class Script:
    entity: Entity
    pub(read) enabled: bool = true         # anyone may read; only this module may write

    fn init(mut self, entity: Entity):
        self.entity = entity

    virtual fn on_create(mut self): pass
    virtual fn on_update(mut self, dt: f32): pass
    fn drop(mut self): pass                    # optional destructor; non-virtual, chained automatically

class Door(Script):
    open_angle: f32 = 0.0
    hinge: Weak[Entity]

    fn init(mut self, entity: Entity, hinge: Entity):
        super.init(entity)                     # MUST be the first statement that touches self
        self.hinge = Weak(hinge)

    override fn on_update(mut self, dt: f32):
        self.open_angle = min(self.open_angle + dt, 90.0)
```

* `[CLS-1]` `class` instances live on the heap with the object header defined in Part VIII §1. `Name(args)` allocates, zero-initialises header, runs `init`, and returns a handle with strong count 1.
* `[CLS-2]` **Constructors.** `fn init(mut self, …)` is the constructor. Definite-initialisation analysis (Part XIX §5.6) requires every field without a default to be assigned on every path before `self` is used as a whole (passed anywhere, method called, escaped). Reading a field before it is assigned is `E2100`. Multiple constructors are not supported by overloading; use defaults or associated functions `fn from_file(path: str) -> Result[Self, E]` that call `Self(...)`.
* `[CLS-3]` If no `init` is declared, the memberwise constructor is synthesised as for structs.
* `[CLS-4]` **Inheritance.** A class is `final` unless declared `open` or `abstract`. `class D(B)` requires `B` to be `open`/`abstract`. Methods are non-virtual unless declared `virtual`; overriding requires `override`; overriding a non-virtual method is `E2110`; `virtual` in a final class is `W2111`. The derived `init` MUST call `super.init(...)` exactly once before using inherited fields. Fields cannot be overridden. Base-class fields are laid out first, so a `D*` is a valid `B*` (single inheritance, no virtual bases).
* `[CLS-5]` A class may implement interfaces; interface methods are dispatched statically unless the receiver is `dyn I`.
* `[CLS-6]` `drop` on a class runs derived-first, then base; then fields are dropped in reverse declaration order; then the memory is released when the weak count is also zero.
* `[CLS-7]` `self` inside a class method is a handle (Copy); `mut self` grants a dynamically checked write access for the duration of the method (Part VIII §3). Storing `self` into another object is allowed and retains (this is how observer patterns work — beware cycles; see Part VIII §5).
* `[CLS-8]` Classes are `Sync` iff every field's type is itself `Sync` (`Atomic`, `Mutex[T]`, `RwLock[T]`, a channel end, or a deeply immutable value type) — Part XI. **`let` does not contribute to this derivation.** Since `OQ-18` a `let` field restrains only the binding, and `[CLS-9a]` permits mutation *through* it, so a `let Array[i32]` field is exactly as unsynchronised as a non-`let` one. Non-`Sync` classes use non-atomic counts and are thread-confined.
* `[CLS-9]` `let` fields: `let name: T` declares an **immutable** field, assignable only in `init`. For a field that the class mutates but outsiders may only read, use `pub(read)` (`[MOD-7]`), not `let`. Immutable fields of `Sync` types keep a class `Sync`.
* `[CLS-7a]` Inside a `drop` body, `self` MUST NOT be copied into a place that outlives the call. The compiler MUST reject the statically visible cases — storing `self`, or a handle-typed projection denoting the same object, into a field of another object, a `static`, a container, an `owned fn` capture, or a `Retained` token — as `E3016 self escapes its own drop`, with `help: a destructor may not publish a handle to the object being destroyed; move the data out with `mem.take` instead`. Cases the compiler cannot see are caught by `[OBJ-5]`.
* `[CLS-9a]` A `let` field of a non-`Copy` type MAY still be mutated *through* by a `mut self` method of the declaring class (`self.items.push(v)` is legal); only assignment to the field itself is restricted to `init`. Code that needs the value frozen must choose a type with no `mut self` API, not `let`. For "outsiders may read, only this module may write", use `pub(read)` (`[MOD-7]`).

## V.6 Interfaces and `extend`

```ember
pub interface Drawable:
    fn bounds(self) -> AABB
    fn draw(self, mut cmd: CommandList)
    fn is_visible(self, frustum: Frustum) -> bool:      # default method
        return frustum.contains(self.bounds())

extend Mesh implements Drawable:
    fn bounds(self) -> AABB: return self.aabb
    fn draw(self, mut cmd: CommandList): cmd.draw_mesh(self)

extend[T: Display] Array[T] implements Display:          # generic impl with bound
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        ...
```

* `[IFC-1]` `extend T:` without `implements` adds inherent methods to `T` from any module in the same package (or the declaring package of `T`); `[IFC-2]` inherent extension of foreign types from a third package is `E2120` (avoids silent API changes); use a wrapper or an interface.
* `[IFC-3]` Interface inheritance `interface Ord: Eq` requires implementers of `Ord` to also implement `Eq`.
* `[IFC-4]` Associated consts and types are permitted; associated statics are not.

## V.7 Constants and statics

* `const NAME: T = expr` — compile-time constant, inlined at each use, no address. `expr` MUST be evaluable by the comptime interpreter (Part XIV). Type annotation optional if inferable from a literal.
* `static NAME: T = expr` — one instance per program with a stable address, initialised at compile time (comptime-evaluable expression) or lazily on first access if `T` requires runtime construction (`static lazy` v2; in v1 non-comptime statics are `E2130`). `static` values are immutable; `[STA-1]` `static mut` exists and any access requires `unsafe`. Safe mutable globals use `static COUNTER: Atomic[u64] = Atomic(0)` or `static REG: Mutex[Registry] = Mutex(Registry.new())` (both `Sync`, comptime-constructible).
* `[STA-2]` Module initialisation order is a non-issue by construction: there are no runtime initialisers, so the C++ static-initialisation-order problem cannot arise.

## V.8 Attributes on declarations

Attributes precede the declaration, one per line, and are validated by the compiler against the table in Part III §7. Attribute arguments are literals, identifiers, or nested attribute-like forms (`@derive(Serialize(rename_all="camel"))`). `[ATT-1]` An attribute not in the Part III §7 tables and not in a registered plugin namespace is `E0104`. An attribute listed as **reserved** is `E0104` with a message naming the version that will introduce it, so that a reservation is never mistaken for a typo. A derive-scoped attribute outside its derive is `E0104` naming the derive that defines it.

* `[ATT-2]` A statement attribute MUST be one of `@simd`, `@parallel`, `@unroll`, `@allow`. Any other attribute in statement position is `E0104`, naming the four that are permitted there.
* `[ATT-5]` The attributes 0.6.3 adds are item attributes and never statement attributes, so `[ATT-2]` is unchanged. `@deterministic` is admitted on a function, a module, an interface method and a function type; `@reloadable` and `@noreload` on a module, a function or a class, and not both on the same item (`E0104`); `@renamed_from` on a field or an enum variant; `@reinit_on_reload` on a static; `@always_specialize` and `@never_specialize` on a generic function or generic type only, and not both on the same item (`E0104`). Each in any other position is `E0104`, naming the positions it does admit.
* `[ATT-3]` A statement attribute attaches to the next `compound_stmt` and MUST NOT precede a `simple_stmt` (`E0108`). `@simd`, `@parallel` and `@unroll` additionally require that statement to be a `for_stmt` (`E0108`). More than one statement attribute MAY precede one statement, one per line (`[ATT-4]`); their order is not significant.
* `[GRM-20]` The `{attribute}` prefix of `statement` is constrained by `[ATT-2]` and `[ATT-3]`: the grammar admits the position and Part V §8 decides which attributes may occupy it. The `{attribute}` prefix of `statement` is constrained by `[ATT-2]` and `[ATT-3]`: the grammar admits the position and Part V §8 decides which attributes may occupy it.
* `[GRM-21]` `gen_fn := "gen" fn_decl`. A `gen fn` is admitted wherever `fn_decl` is admitted except in an `extern` block (`[CORO-10]`), and its declared return type MUST be `Coroutine[R]` for some `R` (`E2222` otherwise).
* `[GRM-23]` **`in` and `not in` are binary operators.** `membership := expression ("in" | "not" "in") expression`, at **comparison precedence** — the same level as `==` and `<` — and **non-associative**: `a in b in c` is `E0104`, because Ember has no chained comparison and the Python reading would surprise a C++ programmer. `not in` is a **single operator**, so `x not in xs` parses as `not_in(x, xs)` and never as `(not x) in xs`; a `not` that is not immediately followed by `in` is the ordinary prefix operator. The token `in` is already used in two other positions and neither is ambiguous with this one, because both are reached in a context that admits no expression at that point: `for pattern in expression:` (`[GRM-*]`'s for header, where `in` follows a pattern) and `type T = R in a ..= b` (`[RNG-1]`, where `in` follows a type). A parser MUST commit on the enclosing construct, not on the token.
* `[GRM-22]` `yield_expr := "yield" [ expression ]`. `yield` occupies the grammatical position and precedence of `return`: it is an expression whose type is the coroutine's resume type (`()` unless the coroutine declares one), so it may appear anywhere an expression may, including as the right-hand side of a binding. A bare `yield` is `yield ()`.
* `[GRM-8d]` `range_clause` is admitted only where `type_alias` appears as an `item`; a `range_clause` on an `interface_member` associated type or on an `extern_item` opaque type is `E2213`. A `type_alias` carrying both `generic_params` and a `range_clause` is `E2213`: a range type is over a concrete representation. The production is LL(2) — no other `type_alias` may be followed by `in`, and a `type_alias` never appears where a `for_stmt` may begin, so `for x in …` is untouched. `range_clause` is admitted only where `type_alias` appears as an `item`; a `range_clause` on an `interface_member` associated type or on an `extern_item` opaque type is `E2213`. A `type_alias` carrying `generic_params` and a `range_clause` is `E2213`: a range type is over a concrete representation. The production is LL(2): no other `type_alias` may be followed by `in`, and `type_alias` never appears where a `for_stmt` may begin, so `for x in …` is untouched.
* `[FFI-34a]` The `grade` suffix is admitted **only** on an argument of `@ffi` in an overlay or `extern` declaration; anywhere else it is `E0104`. `@` introduces an attribute and a grade and nothing else — it is not an operator in expression position — and the four grade words are contextual, remaining ordinary identifiers everywhere else, so `[LEX-15]`'s reserved set is unchanged. The `grade` suffix is admitted **only** on an argument of `@ffi` in an overlay or `extern` declaration; anywhere else it is `E0104`. `@` introduces an attribute and a grade and nothing else: it is not an infix, prefix or suffix operator in expression position, and `[LEX-15]`'s reserved set is unchanged — the four grade words are contextual and remain ordinary identifiers everywhere else.
* `[ATT-4]` One attribute per line. The `attribute` production carries a mandatory `NEWLINE`, matching Part V §8 and the formatter.

---
