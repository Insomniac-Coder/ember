---

# Part IV — Type System

## IV.1 Type categories

| Category | Declared by | Stored | `b = a` | Destroyed |
|---|---|---|---|---|
| **Scalar** | built in | inline | copy | trivially |
| **Value** | `struct`, `enum`, tuple, `[T; N]`, closure | inline | copy if `Copy`, else move | end of scope, reverse order |
| **Collection** | `Array`, `String`, `Map`, `Set`, … (library) | inline header + heap buffer | move | end of scope; frees its buffer |
| **Handle** | `class` | pointer to a counted heap object | copy (retain) | when the strong count reaches 0 |
| **View** | `ref T`, `Span[T]`, `MutSpan[T]`, `str`, any struct holding one | inline pointer(s) + compile-time region | copy (`MutSpan`, `ref mut`: move) | never (no destructor) |
| **Raw** | `*T`, `*mut T`, `extern fn` | inline pointer | copy | never |
| **Existential** | `dyn I`, `Box[dyn I]`, a class handle typed by an interface | pointer + table | per underlying | per underlying |
| **Unit / Never** | `void`, `Never` | zero-size | trivial | trivial |

* `[TYP-1]` Every concrete type has, at compile time, a size, an alignment, and the properties
  `Copy`, `Send`, `Sync`, needs-drop, view and niche. `size_of[T]()` and `align_of[T]()` report them.

## IV.2 Scalar types

| Type | Size | Notes |
|---|---|---|
| `i8 i16 i32 i64 i128` | 1 2 4 8 16 | two's complement |
| `u8 u16 u32 u64 u128` | 1 2 4 8 16 | |
| `isize usize` | pointer | for FFI and raw memory; not the size type (`[TYP-31]`) |
| `int` | 8 | prelude alias of `i64`; the default integer and the size and index type |
| `f16 f32 f64` | 2 4 8 | IEEE 754 binary16/32/64 |
| `float` | 8 | prelude alias of `f64`; the default float |
| `bool` | 1 | `true` or `false`; any other bit pattern is invalid (`[TYP-2]`) |
| `char` | 4 | a Unicode scalar value; surrogates are invalid (`[TYP-3]`) |
| `void` | 0 | the unit type; its one value is written `()` |
| `Never` | 0 | the type of an expression that does not complete: `return`, `panic(…)`, an infinite loop; coerces to every type |

* `[TYP-2]` Producing a `bool` other than 0 or 1 is undefined behaviour and possible only in `unsafe`.
* `[TYP-3]` Producing a `char` outside the Unicode scalar values is undefined behaviour and possible
  only in `unsafe`.
* `[TYP-27]` *(new in 0.9.9)* `()` is the value of type `void`; `Ok(())` is the success value of `Result[void, E]`. A
  function with no `->` returns `void`.

### Conversions

* `[TYP-4]` *(changed in 0.9.9)* No value converts implicitly between scalar types in an operator.
  `i32 + i64` is `E2020`, whose note appears only when both operands are numeric and names the exact
  `as` cast on the narrower operand. Untyped literals are not values yet and take their type from
  context (`[LEX-16]`, `[LEX-17]`).
* `[TYP-5]` *(changed in 0.9.9)* **Coercions — the complete list.** At a coercion site (assignment or
  initialisation, argument, return, field initialiser, collection element, default value), and only
  there, a value converts implicitly by exactly these rules:
  1. lossless integer widening: `iN → iM`, `uN → uM`, `uN → iM` for M > N;
  2. float widening: `f16 → f32 → f64`;
  3. range erasure: a range type to its representation (`[RNG-2]`);
  4. string literal to `String` (`[TXT-9]`), and `String` to `str` (a borrow, `[SPN-1]`);
  5. collection literal to the collection type the site expects (`[TYP-38]`);
  6. `Array[T]`, `[T; N]` to `Span[T]` (shared borrow) or `MutSpan[T]` at a `mut` site (`[SPN-1]`);
  7. `ref T` from a place (auto-borrow), and reading through a `ref` where a `T` is expected (`[TYP-14]`);
  8. a class handle to a base class or to an interface it implements (upcast);
  9. `Never` to any type;
  10. a function or lambda to a callable type it satisfies (`[CLO-3]`);
  11. a value of type `T` to `Option[T]`, as `Some(value)`, where `T` is not itself an `Option`: one
      level only, so nothing becomes `Option[Option[T]]` (`return v` in a function returning
      `Option[T]`, `f(5)` for a parameter `x: Option[int] = None`).
  Each rule composes with the others only where stated (widening after range erasure, and rule 11
  after rules 1–4). Nothing
  converts implicitly to or from `bool` or `char`.
* `[TYP-6]` *(changed in 0.9.9)* `x as T` converts explicitly:
  * integer → integer: keeps the low bits (truncation or sign/zero extension);
  * float → integer: rounds toward zero and **saturates** at the target's bounds; NaN becomes 0;
  * integer → float: rounds to nearest, ties to even;
  * float → float: rounds to nearest;
  * `char → u32` and `u8 → char`; other integers become a `char` only through `char.from_u32(x) ->
    Option[char]`;
  * `h as? D` downcasts a class handle: `Option[D]`; `h as! D` downcasts or panics.
  A checked conversion that fails instead of truncating is `T.try_from(x) -> Result[T, RangeError]`.
* `[TYP-7]` `as` between pointer types, or between a pointer and an integer, requires `unsafe`.

### Integer arithmetic

* `[TYP-8]` *(changed in 0.9.9)* **Integer overflow panics in every profile.** An arithmetic operation
  (`+ - * // % **`, unary `-`, and the compound assignments) whose mathematical result does not fit its
  type panics with `integer overflow in '<op>'` naming the operator actually written. There is no
  profile in which overflow wraps silently (`[PHIL-13]`). Three explicit ways to get other behaviour:
  the methods `wrapping_<op>`, `checked_<op> -> Option[T]`, `saturating_<op>`, `overflowing_<op> ->
  (T, bool)`; the attribute `@overflow(wrap)` or `@overflow(saturate)` on a function or module, which
  changes the operators inside it for every profile; and range facts (`[RNG-4]`), which remove a check
  the compiler proves cannot fire. Wrapping arithmetic is two's-complement and is never undefined
  behaviour in the generated code (`[CG-C-1]`).
* `[TYP-28]` *(new in 0.9.9)* **Division.** `/` is **true division** and applies to floats. `/` with two integer
  operands is `E2240`, whose fix-its are `//` (floor division) and converting an operand to `float`.
  For integers:
  * `a // b` is floor division: the quotient rounded toward negative infinity (Python). `-7 // 2 ==
    -4`.
  * `a % b` is floor modulo: `a - (a // b) * b`, which has the sign of `b` (Python). `-7 % 2 == 1`,
    `x % -1 == 0`.
  * `a.div_trunc(b)` and `a.rem_trunc(b)` give C's truncating quotient and remainder, for code that
    wants C's exact meaning.
  * a zero divisor panics (`division by zero`); `MIN // -1` and `MIN.div_trunc(-1)` overflow and panic
    per `[TYP-8]`.
* `[TYP-29]` *(new in 0.9.9)* For floats, `//` and `%` are Python's (ODR-021). `a % b` is the
  exact value of `a - b * floor(a / b)`, rounded once to the float type: it has the sign of `b`, and
  is computed exactly (as `fmod` is) rather than by evaluating the formula in rounded steps. `a // b`
  is the floor quotient consistent with it, so `a == (a // b) * b + a % b` to within one rounding:
  `1.0 // 0.1 == 9.0` and `1.0 % 0.1 == 0.09999999999999995`, because `0.1` is slightly more than a
  tenth. Zero divisors, infinities and NaN follow IEEE.
* `[TYP-10]` *(changed in 0.9.9)* **Shifts.** `a << n` and `a >> n` accept any integer type for `n`.
  If `n` is negative or not less than the bit width of `a`'s type the shift panics, in every profile.
  Bits shifted out are discarded and are not an overflow. `>>` on a signed type is arithmetic
  (sign-extending); on an unsigned type logical. The methods `wrapping_shl`/`wrapping_shr` mask the
  amount instead.
* `[TYP-30]` *(new in 0.9.9)* **Powers.** `a ** b` with integer `a` and non-negative integer `b` is exact, and
  overflows per `[TYP-8]`; a negative exponent is `E2151` when it is a constant and a panic when it is
  not. A float base with a float or integer exponent is `pow`. Other combinations are `E2020`.
* `[TYP-31]` *(new in 0.9.9)* **Sizes and indices are `int`.** Every standard container's `len()` returns `int`.
  Indexing accepts an index of any integer type; an index that is negative or not less than the length
  panics (`index out of bounds`). There is no Python-style negative indexing: a negative literal index
  is `E2011` (`[LEX-24]`). `usize` and `isize` remain for foreign interfaces and raw memory.

### Floating point

* `[TYP-9]` Floating point is strict IEEE 754: no reassociation, no contraction of `a * b + c` into a
  fused operation, no assumption that values are finite, unless a function is `@fastmath` (relaxes all
  of these within it) or `@fp(contract)` (permits contraction only). `NaN == NaN` is false. The
  operators `<`, `<=`, `>`, `>=`, `==`, `!=` on floats are IEEE comparisons; `Ord.cmp` on floats is the
  IEEE totalOrder (`[TYP-37]`).
* `[TYP-9a]` Contraction is off by default and the implementation MUST turn it off in the host C
  compiler (`[CG-C-11]`); a toolchain on which it cannot be turned off is rejected with `E9011`.
* `[TYP-9b]` `@fp(contract)` permits, and requires the backend to enable, fused multiply-add within
  that function only.
* `[TYP-9c]` *(changed in 0.9.9)* A toolchain that cannot honour `@fastmath` or `@fp(…)` for one function is `E9041`,
  naming the toolchain and the attribute. The attribute is never ignored (`[PHIL-12]`).

## IV.2a Range types

A range type is a nominal numeric type that carries its own bounds:

```ember
type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0
type Percent   = u8  in 0 ..= 100

fn from_slider(x: f32) -> Result[Roughness, RangeError]:
    return Roughness.checked(x)

fn demo():
    r: Roughness = 0.5            # a constant in range: no check emitted
    p = Percent.clamped(250)      # total: 100
    println(f"{r} {p}")
```

* `[RNG-1]` A `type` alias with an `in` clause declares a nominal numeric type over the named
  representation, restricted to the range. The endpoints are constant expressions of the
  representation. A `type` alias without one is transparent.
* `[RNG-2]` Two range types are distinct even with equal representation and range (`E2210`). A range
  value converts to its representation at a coercion site (`[TYP-5]`); the reverse is a construction.
* `[RNG-3]` Construction from a value not known to be in range is `T.checked(v) -> Result[T,
  RangeError]`. `RangeError` is a prelude type. Construction from a constant in range, or from a value
  whose known range lies inside the target's, emits no check; a constant outside the range is `E2211`.
* `[RNG-3a]` Every range type with finite endpoints has `T.clamped(v) -> T`, total, with no panic path;
  NaN clamps to the lower endpoint.
* `[RNG-4]` *(changed in 0.9.9)* The compiler tracks a known range for numeric expressions — literals,
  `min`, `max`, `clamp`, the arms of a comparison, and arithmetic on known ranges — and uses it to remove
  `[RNG-3]`'s check, `[TYP-8]`'s overflow check and bounds checks (`[OPT-2]`). The variable of a range
  loop `for i in a..b` carries the fact `a <= i < b` in the body (`a <= i <= b` for `a..=b`), and its
  increment has no overflow check. A fact is derived from an operation's mathematical result only when
  that result provably fits; never inside `@fastmath`.
* `[RNG-4a]` For floats, a range fact comes only from the true arm of a comparison and records whether
  NaN is excluded; a fact that does not exclude NaN never removes a construction check.
* `[RNG-5]` Arithmetic on a range value yields its representation: `r * 2.0` is `f32`. Arithmetic
  between two different range types is `E2214`. `min`, `max` and `clamp` over one range type and
  in-range constants keep the range type.
* `[RNG-5a1]` *(changed in 0.9.9)* The operators on range types come from compiler-generated
  implementations of `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`, `Neg`, `Eq` and `Ord`, placed in
  the type's declaring module; exact-type implementations are chosen before coercion (`[RNG-5a2]`).
  No compound-assignment operator is generated: `r += 0.1` is `E2214` with the fix-it
  `r = T.clamped(r + 0.1)`.
* `[RNG-5a2]` Overload resolution picks an implementation matching the operand types exactly before
  considering any coercion.
* `[RNG-6]` NaN is in no range. A range from `-0.0` to `+0.0` contains both zeros.
* `[RNG-7]` A range type whose range does not cover its whole representation supplies a niche:
  `Option[Percent]` is one byte. One whose range covers every value supplies none.
* `[RNG-8]` A range type is `Copy` when its representation is, has its representation's layout, and
  crosses a foreign boundary as its representation: a value arriving from C becomes a range value only
  through `checked` or `clamped` (`[RNG-3]`, `[RNG-10b]`).
* `[RNG-9]` A range value outside its range is invalid; producing one is undefined behaviour and needs
  `unsafe` (`T.new_unchecked(v)`, which asserts the range in `debug`).
* `[RNG-10]` In Safe code a range value arises only from a constant in range, `checked`, `clamped`, a
  value whose known range fits, or a copy of a valid value. Anything else is `E2215`.
* `[RNG-10a]` A derived `Deserialize` checks every range-typed field with `checked` and maps failure to
  `SerError`.
* `[RNG-10b]` A range type may not appear in a foreign signature, directly or inside an aggregate or
  behind a pointer (`E5054`); the foreign side uses the representation.

Diagnostics: `E2210` wrong range type; `E2211` constant out of range; `E2212` bad or inverted
endpoints; `E2213` `in` clause on a non-numeric or generic alias; `E2214` arithmetic mixing range types;
`E2215` construction outside `[RNG-10]`.

## IV.3 Compound value types

* **Tuples** `(A, B, C)`: fields `.0`, `.1`, …; `Copy` iff every element is.
* **Fixed arrays** `[T; N]`: inline, `N` a constant; indexing is bounds-checked; `Copy` iff `T` is.
* **Structs** (Part V.3) and **enums** (Part V.4).
* `[TYP-11]` A struct's default layout is C's: declaration order, natural alignment, trailing padding.
  `@layout(c)` states it explicitly.
* `[TYP-12]` A unit-only enum is an integer (`@repr(u8)` and friends choose it; the default is the
  smallest signed type that fits). An enum crossing a foreign boundary MUST carry `@repr`. A payload
  enum is a tag plus a union of variants.
* `[TYP-13]` *(changed in 0.9.9)* `Option[T]` has the size of `T` when `T` has a niche: a class
  handle, `Box`, `ref`, `Span`, `str`, a non-nullable `extern fn`, `bool`, `char`, a range type per
  `[RNG-7]`, an enum with unused discriminants, or `NonZero[T]` (`[STD-4]`). For handles, `Box`, `ref` and `extern fn` this is
  guaranteed, so `Option[Handle]` is a nullable pointer across an FFI boundary.

## IV.4 References and views

* `ref T` and `ref mut T` are first-class references: non-null, aligned, pointing to a live `T` for
  their region. `ref T` is `Copy`; `ref mut T` is move-only and reborrowable.
* `[TYP-14]` *(changed in 0.9.9)* A reference, and a `Box[T]`, is read through wherever a `T` is wanted —
  a field access, a method call, an operand, an argument to a borrowed parameter. So a recursive enum
  whose payload is `Box[Tree]` is traversed by passing the payload to a function taking `Tree`. `r = e`
  on a `ref mut` local writes through it; a reference local is never re-seated (`[BRW-1]`).
* `Span[T]` (read) and `MutSpan[T]` (read-write) are a pointer and a length; `Span` is `Copy`,
  `MutSpan` is move-only and reborrowable. Their API is in §VII.7.
* `str` is a `Span[u8]` known to be valid UTF-8.
* `[TYP-34]` *(new in 0.9.9)* *(replaces the 0.9.8 `@view` requirement)* A struct, enum or tuple that contains a
  reference, a `Span`, a `MutSpan`, a `str` or another view is a **view type**. The compiler infers
  this; `@view` on the declaration is optional documentation, and `@view` on a type that is not a view
  is `E2030`.
* `[TYP-15]` *(changed in 0.9.9)* **Where views may be stored.** A view value may be stored only in a
  place whose lifetime is bounded by every region the view carries. Locals, parameters, return values,
  and fields of other view types are bounded. Class fields, statics, `Box` and `Shared` contents,
  heap-collection elements and `owned fn` captures are not bounded, so they may hold only views whose
  every region is `static` — which admits string literals, `bytes` literals and views of `static`
  items: `names = ["ann", "bob"]` is an `Array[str]` of static strings. Any other stored view is
  `E3063 stored view may not outlive its source`, shape B12.
* `[TYP-15a]` The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) and the view containers
  `BorrowList[T]` may hold views bounded by the container's own region.

## IV.5 Raw types

`*T` and `*mut T` are nullable, possibly unaligned, untracked pointers. Creating one from a reference
is safe; dereferencing, offsetting, reading and writing need `unsafe`. `null[T]()` is the null
pointer. `extern "C" fn(i32) -> i32` is a C function pointer: `Copy`, no captures.

## IV.6 Class handles

A `class C` declaration introduces the type `C` whose values are **handles**: non-null pointers to a
counted heap object. `Option[C]` is the nullable form; `Weak[C]` the weak form. Handles are `Copy`
(copying retains). Upcasting to a base class or to `dyn I` is implicit; downcasting is `as?`/`as!`
(`[TYP-6]`). Part VIII gives the semantics.

## IV.7 Generics

* `[TYP-16]` Generic functions and types are monomorphised: each distinct instantiation is a distinct
  function or type in the output. `ember build --report=instantiations` counts them (`[MONO-2]`).
* `[TYP-17]` Type parameters are bounded by interfaces: `fn sum[T: Add[Output = T] + Default](xs:
  Span[T]) -> T`. Inside a generic body only the operations the bounds provide are available, plus
  copy for `T: Copy`, move, drop and `size_of`. A missing bound is `E2040` naming the bound to add.
* `[TYP-40]` *(new in 0.9.9)* **Interfaces are nominal.** A type implements an interface only through an
  `implements` clause or an `extend … implements` block; having methods of the right names and types is
  not enough. This is what gives every implementation one place of declaration (`[TYP-20]`) and one
  method table for `dyn`.
* `[TYP-18]` Generic arguments are inferred from the arguments of a call. Explicit arguments may be
  written: `f[int](x)`, `Array[f32]()`. A constructor's explicit type arguments always fix the
  instantiation, whatever else the context says.
* `[TYP-19]` There is no specialisation, no higher-kinded type and no variadic generic. Two
  implementations whose applicable types overlap are `E2041`.
* `[TYP-35]` *(new in 0.9.9)* A type parameter need not appear in any field (a **phantom** parameter): `struct
  Handle[Tag]: index: u32` is legal. A phantom parameter affects type identity and counts for `Send` and
  `Sync` as a field of that type would, so a marker type can make a type thread-confined without a
  field; it does not make the type a view.
* Const generics: `fn zeros[const N: int]() -> [f32; N]`. Associated types: `interface Iterator: type
  Item`. Default type parameters: `interface Add[Rhs = Self]`.

## IV.8 Interfaces

An `interface` declares methods, associated types and constants, and may give default method bodies.
A type implements an interface in its header (`struct P implements Display:`) or in an `extend P
implements Display:` block.

* `[TYP-20]` *(changed in 0.9.9)* **Coherence is per package.** An implementation of interface `I`
  for type `T` may appear in any module of the package that declares `I` or the package that declares
  `T`. Two implementations of one interface for one type anywhere in a program are `E2041`, naming
  both. (Separate compilation relies on this: an implementation cannot appear from a third package.)
* `[TYP-24]` *(changed in 0.9.9)* An interface method is found through any implementation visible in
  the program; the interface need not be imported. When two interfaces supply a method of one name for
  one type, the call is `E2070` and is disambiguated as `I.m(recv, …)`.

**Marker interfaces**, derived automatically (implementable by hand only with `unsafe extend`):

| Marker | Holds when |
|---|---|
| `Copy` | declared with `@derive(Copy)` on a struct or enum whose fields are all `Copy` and which has no `drop`; always for scalars, `ref T`, `Span`, `str`, raw pointers, `extern fn`, handles, tuples and fixed arrays of `Copy` elements |
| `Send` | every field is `Send`; raw pointers and `ref` are not; a class handle is `Send` iff the class is `Sync` |
| `Sync` | every field is `Sync`; `ref mut`, `Cell`, `RefCell` and `UnsafeCell` are not; see `[THR-1]` for classes |
| `Sized` | everything except `dyn I` and unsized foreign types |

**Standard interfaces.** These are the definitions; `std.core` declares them and the prelude exports
them (`[MOD-5]`).

```ember,ignore
## signature sketch: the standard interfaces as std.core declares them
interface Clone:
    fn clone(self) -> Self

interface Drop:
    fn drop(mut self)

interface Default:
    fn default() -> Self

interface Eq:
    fn eq(self, other: Self) -> bool                 # ==, !=

interface Ord: Eq:
    fn cmp(self, other: Self) -> Ordering            # <, <=, >, >= in generic code; sort, min, max

enum Ordering:
    Less
    Equal
    Greater

interface Hash:
    fn hash[H: Hasher](self, mut h: H)

interface Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]        # f"{x}", print(x)

interface Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]  # f"{x!r}", f"{x:?}"

interface Iterator:
    type Item
    fn next(mut self) -> Option[Item]

interface Iterable:
    type Item
    type Iter: Iterator[Item = Item]
    fn iter(self) -> Iter                                           # for x in c

interface IntoIterator:
    type Item
    type Iter: Iterator[Item = Item]
    fn into_iter(owned self) -> Iter                                # for x in owned c

interface Contains[T]:
    fn contains(self, item: T) -> bool                              # x in c

interface Add[Rhs = Self]:                                          # likewise Sub, Mul, Div,
    type Output                                                     # FloorDiv, Rem, Pow, BitAnd,
    fn add(self, rhs: Rhs) -> Output                                # BitOr, BitXor, Shl, Shr

interface AddAssign[Rhs = Self]:                                    # likewise for each operator
    fn add_assign(mut self, rhs: Rhs)

interface Neg:                                                      # likewise Not (~)
    type Output
    fn neg(self) -> Output

interface Index[Idx]:
    type Output
    fn index(self, i: Idx) -> ref Output                            # a[i] read

interface IndexMut[Idx]: Index[Idx]:
    fn index_mut(mut self, i: Idx) -> ref mut Output                # a[i] write

interface IndexSet[Idx, V]:
    fn index_set(mut self, i: Idx, owned v: V)                      # a[i] = v (Map inserts)

interface Error: Debug + Display:
    fn source(self) -> Option[ref dyn Error]                        # default: None

interface From[T]:
    fn from(owned value: T) -> Self                                 # `?` conversion; Into is derived
```

* `[TYP-36]` *(new in 0.9.9)* **Which types implement which interfaces.** The table is normative; a `—` means the
  implementation does not exist and a bound requiring it is `E2040`.

| Type | Clone | Copy | Eq | Ord | Hash | Default | Display | Debug |
|---|---|---|---|---|---|---|---|---|
| integers, `int` | ✓ | ✓ | ✓ | ✓ | ✓ | 0 | ✓ | ✓ |
| floats, `float` | ✓ | ✓ | IEEE `==` | totalOrder | — | 0.0 | ✓ | ✓ |
| `bool` | ✓ | ✓ | ✓ | `false < true` | ✓ | `false` | ✓ | ✓ |
| `char` | ✓ | ✓ | ✓ | by scalar value | ✓ | `'\0'` | ✓ | ✓ |
| `void` | ✓ | ✓ | ✓ | ✓ | ✓ | `()` | — | ✓ |
| `str` | ✓ | ✓ | ✓ | by bytes | ✓ | `""` | ✓ | ✓ |
| `String` | ✓ | — | ✓ | by bytes | ✓ | empty | ✓ | ✓ |
| tuples, `[T; N]` | if all | if all | if all | lexicographic, if all | if all | if all | if all `Debug` | if all |
| `Option[T]`, `Result[T, E]` | if `T`(,`E`) | if `T`(,`E`) | if `T`(,`E`) | `None < Some` | if `T`(,`E`) | `None` / — | if `Debug` | if `T`(,`E`) |
| `Array[T]` | if `T` | — | if `T` | lexicographic | if `T` | empty | if `T: Debug` | if `T` |
| `Map[K, V]`, `Set[T]` | if all | — | as sets of entries | — | — | empty | if all `Debug` | if all |
| class handles | handle copy | ✓ | identity (`is`) | — | identity | — | — | ✓ (class and address) |
| structs, enums | derived per `[STR-5]` | `@derive(Copy)` | derived per `[STR-5]` | `@derive(Ord)` | `@derive(Hash)` | `@derive(Default)` | explicit | derived per `[STR-5]` |

* `[TYP-39]` *(new in 0.9.9)* Collections, tuples and `Option`/`Result` implement `Display` the way Python's `str()`
  shows them: `[1, 2, 3]`, `{'a': 1}`, `{1, 2}`, `(1, 'x')`, `Some(3)`, `None`, with each element
  formatted by its `Debug` implementation (strings quoted). So `println(xs)` prints a list as Python
  would. A `Map` prints its entries in insertion order and an empty one prints `{}`; an empty `Set`
  prints `set()`. A collection's `Debug` text is its `Display` text. A string's `Debug` text is
  Python's `repr`: single quotes, or double quotes when the text holds a `'` and no `"`; `\`, the
  chosen quote, `\n`, `\r` and `\t` are escaped, and any other control character is `\xNN`.
* `[TYP-37]` *(new in 0.9.9)* Floats implement `Eq` with IEEE `==` (so `NaN != NaN`) and `Ord` with the IEEE-754
  totalOrder predicate (`-NaN < -inf < … < -0.0 < +0.0 < … < +inf < +NaN`). The comparison
  *operators* on float values keep IEEE meaning (`[TYP-9]`); generic code bounded by `Ord`, `sort()`,
  `min()` and `max()` use `cmp`, so sorting floats is total and needs no extra step. Floats do not
  implement `Hash`; a float map key is written with `f.to_bits()`.
* `[TYP-21]` *(changed in 0.9.9)* Operators desugar to these interfaces for non-scalar operands; `a + b` calls
  `Add.add(a, b)` with both operands borrowed. `a += b` calls `AddAssign.add_assign` if implemented,
  else `a = a + b`. Scalar operators are built in. The interfaces' methods are `add`, `sub`, `mul`,
  `div`, `floordiv`, `rem`, `pow`, `bitand`, `bitor`, `bitxor`, `shl`, `shr`, `neg` and `not`, and
  each `…Assign` form's is its operator's followed by `_assign` (ODR-040). A type has an operator
  only by implementing its interface (`[TYP-40]`); a method that merely shares the name is not one.
  Each number type implements the interface of each operator it has, with `Rhs = Self` and `Output =
  Self`: every integer type `Add`, `Sub`, `Mul`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`,
  `BitOr`, `BitXor`, `Shl`, `Shr` and their `…Assign` forms, and every float type `Add`, `Sub`,
  `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg` and theirs, so `[TYP-17]`'s `sum` takes numbers.
  Such a method is the operator: `3.add(4)` is `3 + 4`. `bool`, `char` and `String` implement none
  of them (`String`'s `+` consumes its left operand, `[TXT-11]`). Likewise a type is indexed only
  through `Index`, `IndexMut` and `IndexSet` (ODR-042).
* `[HASH-1]` `Hash.hash` is generic over `H: Hasher` and monomorphised; hashing a value MUST feed a
  deterministic representation of it into the hasher, and values equal under `Eq` MUST hash equally.
  `Hasher` (in `std.collections`, not the prelude) is a move-only hashing state with `write_bytes`,
  `write_u8`, `write_u16`, `write_u32`, `write_u64`, `write_i8`, `write_i16`, `write_i32`,
  `write_i64`, `write_usize`, `write_isize` and `finish(owned self) -> u64`. A user type may
  implement it and serve as a map's hasher.
* `[HASH-2]` *(changed in 0.9.9)* `std.collections.DefaultHasher` is a fixed-seed hasher: the same
  keys hash the same way in every run and on every machine. It is a fast non-cryptographic hash, no
  slower on integer keys than one multiply and one rotate per word (FxHash's class). `std.collections.
  RandomState` is a per-process randomly seeded hasher for maps keyed by untrusted input; a `Map[K, V,
  RandomState]` still iterates in insertion order (`[STD-11]`), so the seed never becomes visible.
  `DefaultHasher`'s values are the same for one Ember version on every target (its state is 64 bits
  on a 32-bit target too), so a table built at compile time is valid at run time; they may change
  between versions, and a program MUST NOT depend on them.
* `[HASH-3]` *(changed in 0.9.9)* `Map` and `Set` MUST NOT weaken equality to compensate for an
  incoherent `Hash`. An incoherent `Hash`, a key changed through a `Cell`, or an inconsistent `Ord`
  given to `sort` gives wrong answers (a missing key, any permutation of the elements) but never
  undefined behaviour, an out-of-bounds access or an operation that does not terminate. A `hash` or
  `eq` that panics aborts the process (`[PAN-1]`), so no map is seen half-changed; one that reaches
  the map again can do so only through a `RefCell`, whose borrow panics.
* `[HASH-4]` `Map[K, V]` requires `K: Eq + Hash`. A map may call `hash` any number of times per
  operation. Safe map APIs never expose `ref mut K`.

## IV.9 Existential and opaque types

* `dyn I` is unsized and used behind `ref dyn I`, `Box[dyn I]` or a class handle. It is a data pointer
  and a table pointer (a class handle reaches its table through the object header, `[OBJ-2]`).
* `[TYP-22]` An interface is usable as `dyn` only if every method has a receiver, is not generic, and
  does not return `Self` by value (methods marked `where Self: Sized` excepted). Violations are
  `E2050`, naming the method and the clause.
* `[TYP-32]` *(new in 0.9.9)* `some I` in a return type means "one concrete type, chosen by the function body, that
  implements `I`". The caller sees only `I`; the compiler knows the type and monomorphises through it,
  so there is no indirection. Every `return` in the body must produce the same concrete type
  (`E2261` otherwise). It is how a function returns an iterator pipeline whose type cannot be written:

```ember
fn evens(xs: Span[int]) -> some Iterator[Item = int]:
    return xs.iter().copied().filter(fn(x) => x % 2 == 0)
```

## IV.10 Type inference

* `[TYP-23]` *(changed in 0.9.9)* Inference is local to a function body and bidirectional. Function
  signatures, fields, statics and constants are annotated (an omitted return type is `void`). A local
  declared by `x = e` takes `e`'s type; a type left open by `e` (`[]`, `Map()`, `None`) is fixed by
  later uses in the same function, and one still open at the end is `E2060`, highlighting the first
  use. In particular `x = None` followed by `x = v` with `v: T` gives `x` the type `Option[T]`, and the
  later assignment converts by `[TYP-5]` rule 11. An untyped literal is never left open: its context
  is the type expected at the literal itself, and an unannotated declaration supplies none, so
  `total = 0` declares an `int` whatever the later uses (ODR-022); a later use that needs another
  type is `E2020`, whose help names the declaration and the annotation (`total: i32 = 0`). Expected
  types flow into literals, lambdas, `None`, `Ok(…)`, `Err(…)` and generic calls; a branch of a
  conditional expression that cannot type itself (`None`, `[]`) takes the other branch's type. A
  lambda with unannotated parameters and no expected callable type is `E2061`. An unannotated lambda
  parameter also takes `owned` from the expected callable type (ODR-025); `mut` is never inferred, so
  a lambda that writes its parameter says `mut` (`E2228`, shape B15). Within one expression, untyped
  literals are resolved last (`1 + x` takes `x`'s type).

## IV.11 Method resolution and calls

For `recv.m(args)`:

1. Strip references and handle indirection from `recv`'s type `R`, and one level of `Box`/`Shared`.
2. Look for an inherent method `m` on `R`, then on `R`'s base classes nearest first, then in any
   interface `R` implements (`[TYP-24]`), then a field of `R` whose type is callable (`[CLO-11]`). An
   inherent method beats an interface method of the same name.
3. Adjust the receiver to the method's declared mode: `self` borrows, `mut self` borrows mutably,
   `owned self` moves (for a class handle, each of these is a copy of the handle, `[CLS-7]`).

* `[TYP-25]` Arguments may be positional or named; positional arguments come first; a parameter with a
  default may be omitted.
* `[TYP-26]` *(changed in 0.9.9)* There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. The
  compiler-known output functions `print`, `println`, `eprint` and `eprintln` (`[STD-9]`) are the only calls with a
  variable number of arguments.
