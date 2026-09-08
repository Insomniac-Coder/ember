# Part IV — Type System

## IV.1 Type categories

Every type belongs to exactly one **category**, which determines storage, copy/move behaviour, and how it interacts with the borrow checker and the runtime:

| Category | Declared by | Stored | Assignment `b = a` | Destroyed |
|---|---|---|---|---|
| **Scalar** | built-in | inline | copy | never (trivial) |
| **Value** | `struct`, `enum`, tuple, `[T; N]`, closures | inline | copy if `Copy`, else move | end of scope, reverse order |
| **Reference** (handle) | `class` | pointer to counted heap object | handle copy (retain) | when strong count → 0 |
| **View** | `ref T`, `Span[T]`, `@view struct` | inline pointer(s) + compile-time region | copy (views are always `Copy`) | never (no destructor); region checked |
| **Raw** | `*T`, `*mut T`, `extern fn` | inline pointer | copy | never |
| **Existential** | `dyn I`, `Box[dyn I]`, class handle upcast | fat pointer (data + vtable) | per underlying | per underlying |
| **Unit / Never** | `void`, `!` | zero-size | trivial | trivial |

`[TYP-1]` Every concrete type has compile-time-known `size`, `align`, `is_copy`, `is_send`, `is_sync`, `needs_drop`, `is_view`, `has_niche`. The compiler computes these in the `TypeInfo` table (Part XVIII §4.3).

## IV.2 Scalar types

| Type | Size | Notes |
|---|---|---|
| `i8 i16 i32 i64 i128` | 1 2 4 8 16 | two's complement |
| `u8 u16 u32 u64 u128` | 1 2 4 8 16 | |
| `isize usize` | pointer | `usize` is the index and size type |
| `f16 f32 f64` | 2 4 8 | IEEE 754; `f16` arithmetic is performed in f32 and rounded on the C backend |
| `bool` | 1 | values 0/1 only; `[TYP-2]` producing any other bit pattern is UB and requires `unsafe` |
| `char` | 4 | Unicode scalar value; `[TYP-3]` surrogates are invalid |
| `void` | 0 | the unit type; value `()` |
| `!` | 0 | never type; coerces to every type; result of `panic`, `return`, `break`, `continue`, infinite `while true` |

`[TYP-4]` **No implicit conversions between scalar types in operators.** `i32 + i64` is `E2020`. `[TYP-5]` **Coercion sites** (assignment, argument, return, field initialiser, array element) allow **lossless widening**: `iN → iM` (M>N), `uN → uM` (M>N), `uN → iM` (M>N), `f32 → f64`, `f16 → f32`. Nothing converts to/from `bool` or `char` implicitly. `[TYP-6]` Everything else uses `as`:

* `x as T` for numeric types: truncation for narrowing integers (bit truncation), float→int saturating with NaN→0 (Rust semantics), int→float round-to-nearest.
* `x as u8` from `char`, `x as char` from `u8` only (wider ints via `char.from_u32() -> Option[char]`).
* `[TYP-7]` `as` between pointer types and between pointer and `usize` requires `unsafe`.

**Integer overflow** `[TYP-8]`: in the `debug` profile every arithmetic operation that overflows panics with `E-panic: integer overflow` and the source location. In `release`/`shipping`, `+ - *` and `<<` wrap two's-complement; `/` and `%` by zero always panic; `i32.MIN / -1` always panics. `@overflow(panic|wrap|saturate)` on a function or module overrides the profile. Explicit methods `wrapping_add`, `checked_add -> Option`, `saturating_add`, `overflowing_add -> (T, bool)` always exist.

**Floating point** `[TYP-9]`: strict IEEE semantics; no fast-math, no FMA contraction, no reassociation unless the function is `@fastmath`, in which case the C backend emits `#pragma float_control(precise, off)`/`-ffast-math`-equivalent attributes for that function only. `NaN == NaN` is false; `Ord` is not implemented for floats — use `partial_cmp` or `total_cmp`.

**Shifts** `[TYP-10]`: shift amount ≥ bit width panics in debug and is masked in release (like Rust).

## IV.3 Compound value types

**Tuples** `(A, B, C)`: value category; fields `.0 .1`; `Copy` iff all elements `Copy`. Destructured by `a, b = t`.

**Fixed arrays** `[T; N]`: inline, `N` is a const generic; index bounds-checked; `Copy` iff `T: Copy`. Literal `[0.0; 16]`. Coerces to `Span[T]`/`MutSpan[T]` at coercion sites.

**Structs**: see Part V. Field order in memory follows declaration order unless `@layout(rust)` is given (which permits reordering for size — v2). `[TYP-11]` Default layout is **C-compatible** (`@layout(c)` is implied); this is deliberate so that every plain struct can cross an FFI boundary.

**Enums**: unit-only enums have an integer discriminant (`@repr(u8)` etc., default `i32`-sized-or-smaller chosen by the compiler; `[TYP-12]` `@repr` is required for FFI). Payload enums are tagged unions; layout: `{tag, union of variants}` with the tag placed at offset 0 unless a niche makes it free.

**Niche optimisation** `[TYP-13]`: `Option[T]` where `T` is a class handle, `Box`, `ref`, `Span` (non-null pointer), `bool`, `char`, or an enum with fewer variants than its repr allows, has the same size as `T`. This is *guaranteed* for handles, `Box`, `ref` and `*fn` so that `Option[Handle]` is ABI-compatible with a nullable pointer.

## IV.4 Reference and view types

* `ref T` / `ref mut T`: a first-class reference. Non-null, aligned, points to a live `T` for the duration of its region. `Copy` for `ref T`; `ref mut T` is **move-only** (reborrowable). Auto-dereferenced: `r.field`, `r.method()`, and use of **any expression of type `ref T`/`ref mut T`** where a `T` is wanted all read through — a named reference, a call that returns one, a field read, an operand of an operator, an argument. Reading through is a property of the *type in a value context*, not of the form the reference was written in. `[TYP-14]` A `ref mut` local written with `=` writes through to the referent (C++ reference semantics). Rebinding is not possible; shadow instead.
* `Span[T]` (read) and `MutSpan[T]` (read/write): pointer + length. `Copy` and move-only respectively. Bounds-checked indexing; `.len()`, `.iter()`, `.iter_mut()`, `.split_at(i)`, `.chunks(n)`, `.as_ptr()` (unsafe result).
* `str`: `Span[u8]` known to be valid UTF-8.
* `@view struct`: any struct containing a `ref`, `Span`, `MutSpan`, `str` or another view type is automatically a view type. The attribute is required on the declaration as documentation; omitting it is `E2030` with a fix-it. A view type has exactly one implicit **region parameter** (Part VII §5).

`[TYP-15]` A view-typed value MUST NOT be stored in a place whose region is not outlived by the view's region. Class fields, non-view struct fields, `static`s, `Box[T]` and `Shared[T]` contents, container elements and `owned fn` captures have no bounding region and are therefore always forbidden (`E3063 stored view may not outlive its source`, shape B12). Views MAY live in locals, parameters, return values, and in tuple, enum, `Option` and `Result` payloads. Whether a generic container instantiated at a view type may itself become a view type is reserved (OQ-19).

`[TYP-15a]` **Specialized containers of views.** `BorrowList[T]` and `ViewList[T]` MAY contain view-typed elements when all elements are bounded by one compiler-inferred region. The region is inferred from the container's construction and mutation context and MUST NOT be written by the programmer. The container's element region MUST satisfy the one-region model of `[LT-2]`; an operation that would require two independent element regions is rejected. These types are specialized borrowing containers, not ordinary owning generic containers, and they MUST NOT be used to smuggle a view into a class field, `static`, `Box`, `Shared`, or another place forbidden by `[TYP-15]`. Arbitrary owning containers instantiated at a view type (`Array[str]`, `Map[str, V]`, `Array[MutSpan[T]]`) remain rejected.

## IV.5 Raw types

`*T`, `*mut T`: nullable, unaligned-permitted, untracked. Creating one from a `ref` is safe (`ref_to_ptr`); dereferencing, offsetting, reading, writing require `unsafe`. `null[T]()` produces a null pointer. `*void` is permitted for FFI.

`extern "C" fn(i32) -> i32`: a raw function pointer with the C calling convention; `Copy`; cannot capture.

## IV.6 Class handles

A `class C` declaration introduces the type `C` whose values are **handles** (non-null pointers to counted heap objects). `Option[C]` is the nullable form. `Weak[C]` is a weak handle. Handles are `Copy` (copying retains). Upcasting a handle to a base class or to `dyn I` is implicit; downcasting uses `h as? Derived` → `Option[Derived]` (runtime type check) or `h as! Derived` (panics). Semantics in Part VIII.

## IV.7 Generics

* `[TYP-16]` Generic functions and types are **monomorphised**: every distinct instantiation produces a distinct symbol. Code size is the programmer's responsibility; the compiler deduplicates identical instantiations across modules at link time (COMDAT in the C backend via `inline`/`selectany`, weak symbols via LLVM).
* Type parameters are **bounded** by interfaces: `fn sum[T: Numeric](xs: Span[T]) -> T`. `[TYP-17]` Inside a generic body, only operations provided by the bounds (and universal operations: copy if `T: Copy`, move, drop, `size_of`) are permitted. There is no duck typing; a missing bound is `E2040` with a suggested bound.
* Const generics: `fn zero[T, const N: usize]() -> [T; N]`.
* Associated types in interfaces: `interface Iterator: type Item; fn next(mut self) -> Option[Item]`. Bindings: `Iterator[Item = i32]`.
* Default type parameters: `interface Add[Rhs = Self]`.
* `[TYP-18]` Generic parameters are inferred from arguments at call sites by unification; explicit instantiation `f[i32](x)` is allowed and required when no argument mentions the parameter (`Array[f32]()`).
* `[TYP-19]` No specialisation, no higher-kinded types, no variadic generics in v1. Overlapping `extend` impls are `E2041`.
* `[TYP-9a]` **Contraction is off by default and MUST be made off.** `[TYP-9]`'s prohibition on FMA contraction is not the default of any supported host C compiler. The C backend MUST emit `#pragma STDC FP_CONTRACT OFF` at the head of every translation unit **and** pass the corresponding flag, because GCC does not implement that pragma: Clang and GCC `-ffp-contract=off`; MSVC `/fp:precise` with `#pragma fp_contract(off)`. A toolchain on which contraction cannot be disabled MUST be rejected at configure time with `E9011`, naming the compiler and version.
* `[TYP-9b]` `@fp(contract)` on a function permits — and requires the backend to enable — FMA contraction within that function only. It permits no reassociation, no NaN/Inf assumptions and no other `@fastmath` relaxation. Mapping: Clang `#pragma clang fp contract(fast)` around the body; MSVC `#pragma fp_contract(on)` around the definition; GCC, which has no reliable per-function control, MUST emit the function into its own translation unit compiled with `-ffp-contract=fast` — and such a function is therefore **excluded from `[CG-C-3]`'s inline header on GCC**, since inlining it into a non-contracting TU would silently lose the attribute.
* `[TYP-9c]` If the host toolchain cannot honour `@fastmath` or `@fp(…)` at function granularity, the compiler MUST report `E9010` naming the toolchain and the attribute. It MUST NOT silently compile the function under the translation unit's default float control; a silently ignored float-control attribute is the worst outcome, because the programmer believes the contract holds (`[PHIL-6]`).

## IV.8 Interfaces

An `interface` declares required methods, associated types/consts, and may provide default method bodies. Types implement interfaces in their header (`struct X implements I, J:`) or in an `extend X implements I:` block. `[TYP-20]` **Coherence:** an implementation of interface `I` for type `T` may appear only in the module that declares `I` or the module that declares `T` (orphan rule), which keeps resolution local and incremental.

**Marker interfaces derived automatically** (cannot be implemented by hand except via `unsafe extend`):

| Marker | Derived when |
|---|---|
| `Copy` | struct/enum/tuple whose fields are all `Copy`, has no `drop`, and is not `@move_only`; scalars, views, raw pointers, handles, `extern fn` |
| `Send` | all fields `Send`; raw pointers are `!Send`; class handles are `Send` iff the class is `Sync` (Part XI) |
| `Sync` | all fields `Sync`; `ref mut` and interior-mutable types are `!Sync` unless synchronised |
| `Sized` | everything except `dyn I` and `str`/`[T]` (unsized types appear only behind a pointer) |
| `Drop` | type has a `fn drop(mut self)` method or a field that is `Drop` |

**Standard interfaces** that the compiler knows about (spelled as ordinary interfaces in `std.core`):

```ember
interface Clone:
    fn clone(self) -> Self

interface Drop:
    fn drop(mut self)                                      # called exactly once at end of life

interface Eq:
    fn eq(self, other: Self) -> bool                       # ==, !=

interface Ord: Eq:
    fn cmp(self, other: Self) -> Ordering                  # < > <= >=

interface PartialOrd: Eq:
    fn partial_cmp(self, other: Self) -> Option[Ordering]

interface Hash:
    fn hash(self, mut h: Hasher)

interface Default:
    fn default() -> Self

interface Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]        # f"{x}"

interface Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]  # f"{x:?}"

# Sub, Mul, Div, Rem, Neg, BitAnd, BitOr, BitXor, Shl, Shr and Not are declared
# exactly as Add is; each has a matching *Assign form declared as AddAssign is.
interface Add[Rhs = Self]:
    type Output
    fn add(self, rhs: Rhs) -> Output

interface AddAssign[Rhs = Self]:
    fn add_assign(mut self, rhs: Rhs)

interface Index[Idx]:
    type Output
    fn index(self, i: Idx) -> ref Output                   # a[i] read

interface IndexMut[Idx]: Index[Idx]:
    fn index_mut(mut self, i: Idx) -> ref mut Output       # a[i] write

interface Iterator:
    type Item
    fn next(mut self) -> Option[Item]

interface IntoIterator:
    type Item
    type Iter: Iterator[Item = Item]
    fn into_iter(owned self) -> Iter

interface Iterable:
    type Item
    type Iter: Iterator[Item = Item]
    fn iter(self) -> Iter                                  # for x in v

interface IterableMut: Iterable:
    type IterMut: Iterator[Item = ref mut Item]
    fn iter_mut(mut self) -> IterMut

interface Callable[Args, R]:
    fn call(self, args: Args) -> R                         # closures; compiler-implemented

interface Error: Debug + Display:
    fn source(self) -> Option[ref dyn Error]               # default None

interface From[T]:
    fn from(owned value: T) -> Self                        # `?` conversion; Into is blanket
```

`[TYP-21]` Operators desugar to these interface calls with **auto-referencing**: `a + b` calls `Add.add(a, b)` with `a` and `b` passed in the interface's declared modes (both borrowed for `Add` as declared above, i.e. `Vec3 + Vec3` does not consume). `a += b` calls `AddAssign.add_assign` if implemented, else `a = a + b`.

## IV.9 Existentials (`dyn`)

`dyn I` is an unsized type; it is used behind `ref dyn I`, `Box[dyn I]`, or a class handle whose class implements `I` (`I` alone in handle position means "any class handle implementing I"). A fat pointer is `{data*, vtable*}`; the vtable is `{drop, size, align, method0, method1, …}` in interface declaration order plus supertraits' tables. `[TYP-22]` An interface is `dyn`-compatible only if every method has a receiver, no method is generic, and no method returns `Self` by value (except in default methods marked `where Self: Sized`). Violations are `E2050` when `dyn I` is formed.

## IV.10 Type inference

`[TYP-23]` Inference is **local to a function body** and bidirectional:

1. Function signatures, struct fields, statics and consts MUST be fully annotated (return type omitted ⇒ `void`).
2. Locals declared by `x = expr` get the type of `expr`; if `expr` contains unresolved inference variables (e.g. `Array()`), they are solved by later uses within the same function; unresolved at end of body ⇒ `E2060 cannot infer type of x` with the first use highlighted.
3. Expected types flow downward (checking mode) into literals, lambdas, `Array()`, `None`, `Ok(..)`, `Err(..)`, and generic calls.
4. Lambda parameter types are inferred from the expected function type; a lambda with unannotated parameters in a context without an expected type is `E2061`.
5. Untyped literals are resolved last (`[LEX-16/17]`).
6. Method resolution on an inference variable is deferred until the variable is solved; if a method call forces resolution and multiple types are possible, `E2062`.

The algorithm is Hindley–Milner-style unification over a per-function inference table (union–find of type variables with occurs check), with interface-bound obligations collected and solved after unification (Part XVIII §4.4). There is no let-polymorphism for locals.

## IV.11 Method resolution and auto-ref

For `recv.m(args)`:

1. Determine the type `R` of `recv` after stripping `ref`/`ref mut` and class-handle indirection (auto-deref), at most one level of `Box`/`Shared` deref.
2. Look for an inherent method `m` on `R`, then on `R`'s base classes (nearest first), then in interfaces implemented by `R` that are in scope (imported), then in `dyn` vtables. `[TYP-24]` An inherent method always beats an interface method of the same name; ambiguity between two interfaces is `E2070` (disambiguate with `I.m(recv, ...)`).
3. Adjust the receiver to the method's declared mode: `self` → shared borrow (or copy of a handle), `mut self` → mutable borrow (`E3xxx` if `recv` is not mutable/not a mutable place), `owned self` → move (or handle copy for classes).

Named arguments `f(x=1, y=2)` match parameter names; positional arguments MUST precede named; `[TYP-25]` parameters with defaults may be omitted. Overloading by arity or type is **not** supported (use defaults, generics, or distinct names); `[TYP-26]` two functions with the same name in one scope is `E1030` — except operator interface impls and `extend` blocks for distinct types.

---

