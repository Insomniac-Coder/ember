---

# Part IV — Type System

## IV.1 Type categories

| Category | Declared by | Stored | `b = a` | Destroyed |
|---|---|---|---|---|
| **Scalar** | built in | inline | copy | trivially |
| **Value** | `struct`, `enum`, tuple, `[T; N]`, function value | inline | copy if `Copy`, else hand-over | end of scope, reverse order |
| **Collection** | `Array`, `String`, `Map`, `Set`, … (library) | inline header + heap buffer, from the memory manager its type names (`[TYP-46]`) | hand-over | end of scope; frees its buffer |
| **Object** | `class` | its own heap block, with a header (`[OBJ-1]`) | hand-over | when its last owner ends |
| **Owned block** | `owns T`, `owns dyn I` | pointer to a heap block it owns alone | hand-over | when its owner ends; frees the block |
| **Shared** | `shares T`, `shares mut T`, `thread_shared T`, `shares dyn I`, `shares mut dyn I` | pointer to a counted heap block | hand-over; one more owner only with `shares`, `shares mut` or `thread_shared` | when its last owner ends |
| **Link** | `weakref T` | pointer to its target's block, holding one weak count | hand-over; another link only with `weakref` | trivially, giving its weak count back |
| **View** | `ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, any struct holding one | inline pointer(s) + compile-time region | a read-only view of what `a` views (for `ref T`, `Span` and `StrView`, a copy) | a plain view trivially; a guard ends its access (`[DRP-6]`) |
| **Raw** | `*T`, `*mut T`, `extern fn` | inline pointer | copy | never |
| **Existential** | `dyn I` (a parameter's type), `owns dyn I`, `shares dyn I`, `shares mut dyn I` | pointer + table | per its word | per underlying |
| **Unit / Never** | `void`, `Never` | zero-size | trivial | trivial |

* `[TYP-1]` *(changed in 0.9.15)* Every concrete type has, at compile time, a size, an alignment, and the properties
  `Copy`, `Send`, `ThreadShared`, needs-drop, view and niche. `size_of[T]()` and `align_of[T]()` report them.

## IV.2 Scalar types

| Type | Size | Notes |
|---|---|---|
| `i8 i16 i32 i64 i128` | 1 2 4 8 16 | two's complement |
| `u8 u16 u32 u64 u128` | 1 2 4 8 16 | |
| `u256 i256` | 32 | the 256-bit counts: a count, not a general integer (`[TYP-42]`) |
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
* `[TYP-42]` *(new in 0.9.10)* **The 256-bit counts.** `u256` and `i256` are what the iterators over
  ranges of 128-bit numbers count and number in (`[STD-19]`): a `..=` range of 128-bit numbers holds
  up to 2^128 values, one more than a `u128` holds, so its `len()` is a `u256`, and `enumerate`
  numbers such a range in `i256`, since its start may be negative. Each adds and subtracts (an
  overflow is `[TYP-8]`'s), compares, prints and formats, hashes, and converts with `as` to and from
  the other integer types: a narrowing cast takes the low bits and a cast between the two keeps them,
  as every integer cast does; the narrower unsigned integers widen to `u256` and every integer to
  `i256` (`[TYP-5]`). Each implements the operator interfaces of what it does: `Add`, `Sub`, `Neg`,
  `AddAssign` and `SubAssign` (ODR-040). Nothing else: `*`, `/`, `//`, `%`, `**`, the bitwise
  operators and shifts, the integer methods, `MIN` and `MAX`, a conversion to or from a float, and
  a range of one are `E2020` (a range of 256-bit numbers would need a 512-bit count). `parse` reads
  one over its whole range (`[TXT-10]`; the owner's ruling of 2026-10-06).
  As an index or a size (`[TYP-31]`), one that `usize` does not hold is out of bounds. A literal of one is within
  `i128`'s range for an `i256` and `u128`'s for a `u256`. They are 32 bytes, aligned as `i128` is.
* `[TYP-27]` *(new in 0.9.9)* `()` is the value of type `void`; `Ok(())` is the success value of `Result[void, E]`. A
  function with no `->` returns `void`. Written as a type, `()` is `void`: a tuple type has two or more
  elements (ODR-071).

### Conversions

* `[TYP-4]` *(changed in 0.9.9)* *(changed in 0.9.10)* No value converts implicitly between scalar
  types in an operator, but one: of two whole numbers of different types, one whose every value the
  other's type holds, by the numbers the compiler knows (`[STD-19]`), converts to it, to `int` where
  each would (`[TYP-5]` rule 12; the owner's ruling of 2026-10-06): an `i32` always fits an `i64`, so
  `i32 + i64` is an `i64` addition. `a + b` for an `int` and a `u64` that may be any of their values is
  `E2020` (the `int` may be negative, the `u64` may pass `int`'s top), whose note appears only when
  both operands are numeric and names the exact `as` cast on the narrower operand. Untyped literals
  are not values yet and take their type from context (`[LEX-16]`, `[LEX-17]`).
* `[TYP-5]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Coercions — the complete list.** At a coercion site (assignment or
  initialisation, argument, return, field initialiser, collection element, default value), and only
  there, a value converts implicitly by exactly these rules:
  1. lossless integer widening: `iN → iM`, `uN → uM`, `uN → iM` for M > N;
  2. float widening: `f16 → f32 → f64`;
  3. range erasure: a range type to its representation (`[RNG-2]`);
  4. string literal to `String` (`[TXT-9]`), and `String` to `StrView` (a read-only view, `[SPN-1]`);
  5. collection literal to the collection type the site expects (`[TYP-38]`);
  6. `Array[T]`, `[T; N]` to `Span[T]` (a read-only view) or, given with `ref mut`, to `MutSpan[T]`
     (`[SPN-1]`), and `MutSpan[T]` to `Span[T]`, a read-only view of the same elements (`[SPN-3]`,
     ODR-116);
  7. `ref T` from a place (a read-only view of it), and reading through a `ref` or an `owns T` where a
     `T` is expected (`[TYP-14]`);
  8. a value to an interface its type implements, keeping the word that says who owns it: a value read
     in place to `dyn I`, an `owns T` to `owns dyn I`, a `shares T` to `shares dyn I`, a `shares mut T`
     to `shares mut dyn I` (`[TYP-45]`); and a class object to a base class (upcast);
  9. `Never` to any type;
  10. a function or lambda to a callable type it satisfies (`[CLO-3]`);
  11. a value of type `T` to `Option[T]`, as `Some(value)`, where `T` is not itself an `Option`: one
      level only, so nothing becomes `Option[Option[T]]` (`return v` in a function returning
      `Option[T]`, `f(5)` for a parameter `x: Option[int] = None`);
  12. a whole number (a signed or unsigned integer of any size: `int`, `i8` to `i256`, `u8` to
      `u256`, `isize`, `usize`) to another whole-number type, where the compiler knows every value it
      can have fits that type, by the known-numbers rule (`[STD-19]`: numbers written as numbers,
      names set once and never changed, loop counters, `enumerate`'s numbers, arithmetic on known
      numbers, lengths). It converts as `as` does (`[TYP-6]`), which then loses nothing. After
      `n = 5`, `y: u64 = n` is accepted; `y: u64 = x` for a parameter `x: int` is `E2020`. In a
      function written for any type, a number whose type each use decides (`[STD-19]`) is checked
      at each use (the owner's ruling of 2026-10-06).
  Each rule composes with the others only where stated (widening after range erasure, and rule 11
  after rules 1–4). Nothing
  converts implicitly to or from `bool` or `char`.
* `[TYP-6]` *(changed in 0.9.9)* *(changed in 0.9.15)* `x as T` converts explicitly:
  * integer → integer: keeps the low bits (truncation or sign/zero extension);
  * float → integer: rounds toward zero and **saturates** at the target's bounds; NaN becomes 0;
  * integer → float: rounds to nearest, ties to even;
  * float → float: rounds to nearest;
  * `char → u32` and `u8 → char`; other integers become a `char` only through `char.from_u32(x) ->
    Option[char]`.
  A downcast is not a conversion: `cast h as D` gives the `D` or stops the program, and
  `if h is D as d:` tests (`[TYP-44]`).
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
  behaviour in the generated code (`[CG-C-1]`). The policy is lexical (ODR-084): a module's
  `#! module overflow(panic|wrap|saturate)` supplies its default, otherwise `panic`; a function's
  explicit attribute overrides the enclosing policy. A lambda or local function inherits the
  policy where it is declared unless it has a permitted explicit override. Generic instantiation,
  importing, inlining and calling never replace a declaration's policy with the caller's.
  Parameter defaults use the declaring function's effective policy (`[FN-5]`); supplied argument
  expressions use their own lexical policy. Saturation clamps the exact mathematical result to
  the nearest representable bound. These policies do not change zero-divisor or negative-exponent
  checks, casts, or shift-amount checks (`[TYP-10]`). They change the written operators, not the
  contracts of called functions or integer methods (`[STD-20]`).
* `[TYP-28]` *(new in 0.9.9)* **Division.** `/` is **true division** and applies to floats. `/` with two integer
  operands is `E2240`, whose fix-its are `//` (floor division) and converting an operand to `float`.
  For integers:
  * `a // b` is floor division: the quotient rounded toward negative infinity (Python). `-7 // 2 ==
    -4`.
  * `a % b` is floor modulo: `a - (a // b) * b`, which has the sign of `b` (Python). `-7 % 2 == 1`,
    `x % -1 == 0`.
  * `a.div_trunc(b)` and `a.rem_trunc(b)` give C's truncating quotient and remainder, for code that
    wants C's exact meaning.
  * a zero divisor panics (`division by zero`); `MIN // -1` overflows per `[TYP-8]` (panic by
    default, `MIN` under wrapping, `MAX` under saturation). `MIN.div_trunc(-1)` always panics
    under that method's fixed contract (`[STD-20]`); `MIN.rem_trunc(-1)` is zero.
* `[TYP-29]` *(new in 0.9.9)* For floats, `a // b` is `floor(a / b)` and `a % b` is
  `a - b * floor(a / b)`, each operation rounded to the float type as it is evaluated (ODR-164,
  reversing ODR-021): `1.0 // 0.1 == 10.0` and `1.0 % 0.1 == 0.0`, since `1.0 / 0.1` rounds to
  `10.0`. Zero divisors, infinities and NaN follow IEEE.
* `[TYP-10]` *(changed in 0.9.9)* **Shifts.** `a << n` and `a >> n` accept any integer type for `n`.
  If `n` is negative or not less than the bit width of `a`'s type the shift panics, in every profile.
  Bits shifted out are discarded and are not an overflow. `>>` on a signed type is arithmetic
  (sign-extending); on an unsigned type logical. The methods `wrapping_shl`/`wrapping_shr` mask the
  amount instead.
* `[TYP-30]` *(new in 0.9.9)* **Powers.** `a ** b` with integer `a` and non-negative integer `b` is exact, and
  overflows per `[TYP-8]`; a negative exponent is `E2151` when it is a constant and a panic when it is
  not. A float base with a float or integer exponent is `pow`. Other combinations are `E2020`.
* `[TYP-31]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Sizes and indices are `int`.** Every standard container's `len()` returns `int`.
  Indexing accepts an index of any integer type; an index that is negative or not less than the length
  panics with the bounds check's stop text (`[COST-6]`), whatever its width: one wider than `usize` is compared whole, never
  cut to its low bits (D-527). There is no Python-style negative indexing: a negative literal index
  is `E2011` (`[LEX-24]`). `usize` and `isize` remain for foreign interfaces and raw memory.

### Floating point

* `[TYP-9]` *(changed in 0.9.15)* Floating point is strict IEEE 754: no reassociation, no contraction of `a * b + c` into a
  fused operation, no assumption that values are finite, unless a function is `@fastmath` (relaxes all
  of these within it) or `@fp(allow_fma)` (permits contraction only; `@fp(contract)` is `E0104`, whose
  message gives `@fp(allow_fma)`). The mode is lexical, as
  `[TYP-8]`'s overflow policy is (ODR-084, ODR-090): a lambda takes the mode of the function it is
  written in; a parameter default keeps its declaring function's mode, and a field default is strict,
  wherever either is evaluated; instantiation, inlining and calls keep each function's own.
  Neither mode relaxes what the language defines exactly: a range value is in its range (`[RNG-9]`),
  `x as T` is `[TYP-6]`'s, `is_nan`, `is_finite` and `is_infinite` are exact, and a NaN or infinite
  value never makes a program's behaviour undefined (`[PHIL-10]`).
  `NaN == NaN` is false. The operators `<`, `<=`, `>`, `>=`, `==`, `!=` on floats are IEEE
  comparisons; `Ord.cmp` on floats is the IEEE totalOrder (`[TYP-37]`).
* `[TYP-9a]` Contraction is off by default and the implementation MUST turn it off in the host C
  compiler (`[CG-C-11]`); a toolchain on which it cannot be turned off is rejected with `E9011`.
* `[TYP-9b]` *(changed in 0.9.15)* `@fp(allow_fma)` permits, and requires the backend to enable, fused multiply-add within
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
  that result provably fits. Inside `@fastmath` no fact about a float is derived, as the function may
  be compiled as if no value were NaN; its integer facts are derived as anywhere else, since no float
  mode changes integer arithmetic (ODR-090).
* `[RNG-4a]` For floats, a range fact comes only from the true arm of a comparison and records whether
  NaN is excluded; a fact that does not exclude NaN never removes a construction check.
* `[RNG-5]` Arithmetic on a range value yields its representation: `r * 2.0` is `f32`. Arithmetic
  between two different range types is `E2214`. `min`, `max` and `clamp` over one range type and
  in-range constants keep the range type.
* `[RNG-5a1]` *(changed in 0.9.9)* The operators on range types come from compiler-generated
  implementations, placed in the type's declaring module, of `Eq` and `Ord` and of each operator
  interface the representation implements over itself: `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`,
  `Pow`, `Neg`, `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl` and `Shr`, each with the representation as
  its `Output` (ODR-095: `l | m` and `l ** 2` are the representation's, and `T: BitOr[T]` accepts a
  range type as the operator does). Exact-type implementations are chosen before coercion
  (`[RNG-5a2]`).
  No compound-assignment operator is generated: `r += 0.1` is `E2214` with the fix-it
  `r = T.clamped(r + 0.1)`.
* `[RNG-5a2]` Overload resolution (`[TYP-24]`) picks an implementation matching the operand types
  exactly before considering any coercion, for range types as for every type (ODR-096).
* `[RNG-6]` NaN is in no range. A range from `-0.0` to `+0.0` contains both zeros.
* `[RNG-7]` A range type whose range does not cover its whole representation supplies a niche:
  `Option[Percent]` is one byte. One whose range covers every value supplies none.
* `[RNG-8]` A range type is `Copy` when its representation is, has its representation's layout, and
  crosses a foreign boundary as its representation: a value arriving from C becomes a range value only
  through `checked` or `clamped` (`[RNG-3]`, `[RNG-10b]`). It is `Clone`, `Hash`, `Display` and
  `Debug` when its representation is, and its generated `Eq` and `Ord` (`[RNG-5a1]`) are its
  representation's; a range value shows as its representation's value (`println(r)` prints `0.25`,
  and a format spec applies as to the representation). It is never `Default`: its representation's
  default may lie outside its range, and a range value arises only as `[RNG-10]` allows (ODR-093).
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
* `[TYP-13]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Option[T]` has the size of `T` when `T` has a niche: a class
  object, an `owns T`, a `ref`, a `Span`, a `StrView`, a `CStrView` (ODR-080), a non-nullable `extern fn`, `bool`, `char`, a range type per
  `[RNG-7]`, an enum with unused discriminants, or `NonZero[T]` (`[STD-4]`). For class objects, `owns T`, `ref` and `extern fn` this is
  guaranteed, so `Option[C]` of a class `C`, and `Option[owns T]`, is a nullable pointer across an FFI
  boundary. `CStrView` is non-null; `Option[CStrView]` has the size and alignment of a C character
  pointer, with null representing `None`.

## IV.4 References and views

* `ref T`, `ref mut T` and `sharedref mut T` are views of a place someone else owns: non-null,
  aligned, pointing to a live `T` for their region. `ref T` reads, is `Copy`, and any number of names
  may view one place. `ref mut T` may change its place and is the only name that reads or changes it
  while it is used (`[BRW-1]`); it is not `Copy`, and `b = r` makes `b` a read-only view of the same
  place. `sharedref mut T` may change its target, and several holders, and its owner, may name one
  object; it is not `Copy` either, and another holder is made with `sharedref mut` (`[TYP-45]`).
* `[TYP-14]` *(changed in 0.9.9)* *(changed in 0.9.15)* A reference, and an `owns T`, is read through wherever a `T` is wanted —
  a field access, a method call, an operand, an argument to a plain parameter. So a recursive enum
  whose payload is `owns Tree` is traversed by passing the payload to a function taking `Tree`, and
  `.get()` on an `owns` value is `E2072` ("an `owns Mesh` reads as a `Mesh`; remove `.get()`"). `=` on
  a `ref mut` name writes the place it names: `r = ref mut a` then `r = 20` writes `a`. `r = ref mut b`
  gives it another ref, after which `r = 30` writes `b`; the line that gives another ref says `ref mut`
  again, for every type, and a read-only `ref b` given to a `ref mut` name is `E3023` ("to point `r`
  at `b` write `r = ref mut b`, or for a read-only name `rb = ref b`"). A name written `const ref mut`
  is never given another ref (`E3023`): `const` says the name itself stays as it is, as on a value. A
  read-only view name given a new view with `=` views that instead (`rest = tail.trim()`).
* `Span[T]` (read) and `MutSpan[T]` (read-write) are a pointer and a length; `Span` is `Copy`,
  `MutSpan` is not. A `MutSpan`'s elements are written only through a name that says `ref mut`
  (`d = ref mut xs.as_mut_span()`, `ref mut dst: MutSpan[float]`); a plain name holding one only reads
  them (`E3023`). Their API is in §VII.7.
* `StrView` is a `Span[u8]` known to be valid UTF-8, a view of text someone else owns; `String(w)`
  makes a `String` of one's own from it. `CStrView` is a view of NUL-terminated C text.
  A type that contains itself, or a view of its own type, with no block of its own between, is E2200 ("`Tree` contains itself through `left`, so a `Tree` would have no end; hold the child in its own block: `left: Option[owns Tree]`"; "`Node.parent` would be a view of the `Node` that holds this one, which no struct can keep; for a link back to the parent make `Node` a class with `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this` before `this.children.push(child)`").
* `[TYP-34]` *(new in 0.9.9)* *(replaces the 0.9.8 `@view` requirement)* *(changed in 0.9.15)* A struct, enum or tuple that contains a
  view (`ref T`, `ref mut T`, `sharedref mut T`, a `Span`, a `MutSpan`, a `StrView`) or another view
  type is a **view type**, and lives no longer than what its view fields point into. A field written
  `from static` (`[TYP-15]`) does not make its struct a view, and a class is never a view type: a view
  field of a class says what it points into. The compiler infers this; `@view` on the declaration is
  optional documentation, checked, and `@view` on a type that is not a view is `E2030` ("`@view` on
  `Point`, which holds no view; remove `@view`"). A view type may also need drop, as a guard does
  (`[DRP-6]`).
* `[TYP-15]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Where views may be stored.** A view may be stored in any place — a
  local, a field of a view type, a list's elements, a struct a call makes — and whatever holds it, and
  every value that holds that, then lives no longer than what the view points into, whoever owns it:
  `names = ["ann", "bob"]` is an `Array[StrView]` of text written in the program, which may live as
  long as the program; a list of views of a local's text lives no longer than that local. A view, or a
  value holding views, returned or kept where it would outlive what it points into is `E3063` (shape
  B12). The rule is about the stored value, wherever the store happens (ODR-069):
  * a class object is never a view: a view field of a class says what it points into, `from` a `let`
    field of the same object whose contents live in a block of their own, a `String`, an `Array` or an
    `owns T` (`words: Array[StrView] from text`), or `from static`, text written in the program
    (`kind: StrView from static`; on a struct's field too, which then does not make the struct a view)
    (`[GRM-46]`). A view field of a class without such a `from`, or with one that names what a view
    cannot point into for the object's whole life (a field that is not `let`, a fixed array, a struct
    stored inline), is `E2118` ("`words` points into `text`, so `text` must never change after `init`;
    write `let text: String`"; "`pair` is stored inside the object, so moving the object moves it;
    keep it in a block of its own: write the field as `let pair: owns Pair`");
  * a `static` holds only views of text and data written in the program (`[LT-3]`);
  * a function value holds a view only as a `ref` capture or as a real variable it uses, and does not
    outlive what it views (`E3026`, `[CLO-2]`);
  * a view put into a caller's place through a `ref mut` parameter (`fn put(ref mut x: StrView, v:
    StrView): x = v`, called `put(ref mut name, v)`) is one of the call's view arguments or a part of
    one, or text written in the program, and after the call the caller's place may point into what it
    pointed into before or into any of the call's other arguments (`[LT-21]`); a view of the callee's
    own locals or of a temporary put there is `E3063`.
* `[TYP-15a]` The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) and the view containers
  `BorrowList[T]` may hold views bounded by the container's own region.

## IV.5 Raw types

`*T` and `*mut T` are nullable, possibly unaligned, untracked pointers. Creating one from a reference
is safe (ODR-075): `ref_to_ptr(r: T) -> *T` and `ref_to_ptr(ref mut r: T) -> *mut T`, called
`ref_to_ptr(x)` and `ref_to_ptr(ref mut x)`, keep the predecessor's name. An explicit `r as *T`
is also safe when the pointee type is unchanged and the
cast does not strengthen mutability: `ref T` to `*T`, or `ref mut T` to `*T` or `*mut T`.
The raw pointer keeps no compiler-tracked view. Dereferencing, offsetting, reading and
writing through it need `unsafe`. Raw-pointer-to-raw-pointer and pointer/integer casts still
require `unsafe` (`[TYP-7]`). `null[P]()` (ODR-076) is safe and returns the null pointer of the exact
raw-pointer type `P`: write `null[*i32]()` or `null[*mut i32]()`. `P` must be a raw-pointer type;
neither its mutability nor its pointee is inferred from context. `extern "C" fn(i32) -> i32` is a C
function pointer: `Copy`, no captures.

## IV.6 Class objects

A `class C` declaration introduces the type `C` whose values are **objects**: each lives in its own
heap block, with a header (`[OBJ-1]`), and has one owner unless a line says `shares`, as every value
has: `p2 = p` hands the object over, `p2 = shares p` makes a second owner that reads it, `p2 = shares
mut p` one that may also change it, and `p2 = copy of p` a new object (`[OWN-8]`). A class object is
not `Copy`, and `owns` before a class type is `E2114` (`[TYP-45]`). `Option[C]` is the nullable form;
`weakref C` a link that does not keep the object alive. Upcasting to a base class or to `dyn I` is
implicit; a downcast is a kind test, `if p is D as d:`, or a cast, `cast p as D` (`[TYP-44]`). Part
VIII gives the semantics.

## IV.6a The words in a type: who owns a value, and where its memory comes from

```ember
struct Mesh:
    points: Array[float]

struct Level:
    mesh: owns Mesh                    # held alone, in a heap block of its own

class Player:
    hp: int = 10

class Camera:
    target: weakref Player = None      # a link that does not keep the player alive

fn main():
    lvl = Level(mesh=owns Mesh(points=[0.0, 1.0]))
    p = shares mut Player()            # one of several owners, which may change it
    reader = shares p                  # one more owner, which only reads it
    cam = Camera()
    cam.target = weakref p
    p.hp += 5
    if cam.target exists as t:         # the player still lives: call it `t`
        println(t.hp, reader.hp, lvl.mesh.points.len())
```

* `[TYP-45]` *(new in 0.9.15)* **Who owns a value, in its type.** A type says how its value is held,
  with the word on the type it describes:
  * `T`: held alone: a struct or enum value inside its owner, a class object in its own heap block
    with one owner (§IV.6);
  * `owns T`: held alone, in a heap block of its own (a recursive type's child, an interface value, a
    foreign object with a destructor), read as a `T` (`[TYP-14]`). A class object always has its own
    block, so `owns` on a class type is `E2114` ("a `Player` object always lives in its own block;
    remove `owns`: `p: Player`"). The line that
    makes a value for a place that owns it alone (an `owns` field, an `Option[owns T]`, a list of
    `owns` values), at first or later, says `owns` before it: `mesh=owns Mesh(points=[0.0])`,
    `left=Some(owns Tree(value=2))`, `r.pools.push(owns IntPool(items=[1, 2]))`, `e.mesh = owns
    Mesh(points=[1.0, 2.0])`, which frees the old mesh. Without it the line is `E2123` ("`mesh` owns its
    mesh; say so where it is made: `mesh=owns Mesh(points=[0.0, 1.0]) using current`") (`[GRM-15]`);
  * `shares T` and `shares mut T`: one of several owners of a value on one thread, counted; a
    `shares` owner reads it and a `shares mut` owner may also change it (Part VIII);
  * `thread_shared T`: one of several owners on any threads, counted atomically, and never changed
    directly by any owner (Part XI);
  * `weakref T`: a link that does not keep its target alive and may hold none (`parent: weakref Node
    = None`); it is opened before its target is used (`[GRM-44]`). Used as its target without opening
    it, or given an object without the word, it is `E2020` ("`cam.target` may be gone; open it:
    `if cam.target exists as p:`"; "`=` alone would hand `player` over; write `cam.target = weakref
    player`") (Part VIII);
  * `ref T`, `ref mut T` and `sharedref mut T`: views of a place someone else owns (§IV.4), kept in a
    field or a list as a kept view; `sharedref` without `mut` is `E0114` (`[GRM-43]`);
  * `owns dyn I`, `shares dyn I` and `shares mut dyn I`: a value of some type implementing `I`, held as
    the word says; `dyn I` alone is a parameter's type, a read-only view of the value (`ref mut v:
    dyn I` may change it). An interface named bare as a type does not say who owns its values and is
    `E2173` ("`Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns
    them: write `shapes: Array[owns dyn Shape]` in place of line 19, and `scene =
    Scene(shapes=[owns Square(side=2)])` in place of line 22");
  * a function type says its parameters with their words, its result, and whether any thread may run
    its functions (`fn()`, `fn(ref mut Ui)`, `fn(owns String) -> int`, `thread_shared fn()`), and
    nothing about what its functions change, which the compiler works out from every function the
    program can put in a place of that type (`[CLO-3]`); a long one is named once with `type`
    (`type Handler = fn()`, then `handlers: Array[Handler] = []`).

  `Box`, `Shared`, `Weak`, `SyncShared`, `ForeignBox`, `RefCell`, `Ref` and `RefMut` are not types of
  Ember: naming one is `E1010`, whose message gives what to write (`owns T` for `Box[T]` and
  `ForeignBox[T]`, `owns dyn I` for `Box[dyn I]`, `owns T using A` for `Box[T, A]`, `shares T` or
  `shares mut T` for `Shared[T]`, `weakref T` for `Weak[T]`, `thread_shared T` for `SyncShared[T]`, and
  for `RefCell` the field written plainly and changed through a name that may change it).
* `[TYP-46]` *(new in 0.9.15)* **Which memory manager.** `using A` after a type whose value has heap
  memory of its own names the kind of memory manager that memory comes from, a type implementing
  `Allocator` (`[ALC-2]`): `mesh: owns Mesh using LevelArena`, `mesh: shares Mesh using LevelArena`,
  `names: Array[String] using LevelArena`. It may follow an `owns`, `shares`, `shares mut` or
  `thread_shared` type, a class type, or an `Array`, a `Map` or a `Set`; after any other type it is
  `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove `using`"). Without
  `using`, the memory comes from the built-in manager (`Global`). The line that makes such a value
  names the instance after the value it makes (`owns Mesh(points=[0.0, 1.0]) using current`,
  `Enemy(hp=500) using current`, `[0.0, 1.0] using current`; `[GRM-43]`), and takes the value's block
  from it by calling `current.allocate(...)`; a manager with one instance (`Global`, `System`) needs
  none there. A value given to a `using A` place that was not made in an instance of `A` is `E2020`
  ("`mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh`
  comes from the built-in manager; say which arena: `mesh=owns Mesh(points=[0.0, 1.0]) using
  current`"); a value made in an instance, and every value that holds one, lives no longer than that
  instance (`E3063`, `[ALC-1]`). `using` names where that value's own memory comes from; what it holds
  comes from where its own fields' types say (the `points` list of that `Mesh` from the built-in
  manager). A value whose manager may have several instances keeps its instance's address, one word,
  so that its growth and its end go back to the instance it came from.

## IV.7 Generics

* `[TYP-16]` Generic functions and types are monomorphised: each distinct instantiation is a distinct
  function or type in the output. `ember build --report=instantiations` counts them (`[MONO-2]`).
* `[TYP-17]` *(changed in 0.9.10)* *(changed in 0.9.15)* Type parameters are bounded by interfaces: `[T: Add[Output = T] + Default]`
  above `fn sum(xs: Span[T]) -> T` (`[GRM-40]`). Inside a generic body only the operations the bounds
  provide are available, plus copy for `T: Copy`, hand-over, drop and `size_of`. A `pub` function
  writes its bounds: a missing one is `E2040`, one message on the first line of the body that needs a
  bound the function does not write, giving every bound the body needs ("`total` is `pub`, so its
  bound is written: write `[T: Add[T, Output = T]]` in place of `[T]` above `pub fn total(a: T, b: T)
  -> T:`"). A function that is not `pub` may leave them out: its bounds are worked out from its body,
  each method called on a parameter giving the interface in scope that declares it (two that declare
  it are `E2070`, `[TYP-24]`), and a method no interface in scope declares is `E2072` ("`T` has no
  `weight`: no interface in scope declares `fn weight`; declare one, `interface Weighed:` with `fn
  weight() -> int`, and write `struct Crate implements Weighed:` for the type the function is given;
  the bound follows").
  The body is checked once with its parameters standing for any type, except where the type of
  `enumerate`'s numbers depends on a parameter (`[STD-19]`): that type, and every line using those
  numbers, is checked for each use, with the parameters filled in.
* `[TYP-40]` *(new in 0.9.9)* **Interfaces are nominal.** A type implements an interface only through an
  `implements` clause or an `extend … implements` block; having methods of the right names and types is
  not enough. This is what gives every implementation one place of declaration (`[TYP-20]`) and one
  method table for `dyn`.
* `[TYP-18]` Generic arguments are inferred from the arguments of a call. Explicit arguments may be
  written: `f[int](x)`, `Array[f32]()`. A constructor's explicit type arguments always fix the
  instantiation, whatever else the context says.
* `[TYP-19]` *(changed in 0.9.15)* There is no specialisation, no higher-kinded type and no variadic generic. Two
  implementations whose applicable types overlap are `E2041`. Whether two overlap is decided where
  they are declared, whether or not any instance meets both: they overlap when one type could be
  both, each implementation's type parameters standing for any type (`extend[T] Pair[T, int]` and
  `extend[U] Pair[StrView, U]` overlap at `Pair[StrView, int]`), and when their interface's arguments could
  be the same too. A bound separates two only where the type it bounds is written out in full and
  does not meet it (`extend[T: Copy] Pair[T, int]` and `extend Pair[String, int]` do not overlap); a
  bound on a type left open never does, as a type meeting both bounds can be declared later, in
  another package (ODR-092).
* `[TYP-35]` *(new in 0.9.9)* *(changed in 0.9.15)* A type parameter need not appear in any field (a **phantom** parameter): `[Tag]`
  above `struct Handle:` with the one field `index: u32` is legal. A phantom parameter affects type
  identity and counts for `Send` and `ThreadShared` as a field of that type would, so a marker type
  can make a type thread-confined without a field; it does not make the type a view.
* Const generics: `[const N: int]` above `fn zeros() -> [f32; N]`. Associated types: `interface
  Iterator: type Item`. Default type parameters: `interface Add[Rhs = Self]`.
* `[TYP-41]` *(new in 0.9.9)* **Const arguments.** A `const` generic argument, like an array length,
  is any expression compile time evaluates (`[CT-1]`) over constants and the enclosing declaration's
  own `const` parameters: `[f32; rows * cols]` and `zeros[side * side]()` are as legal as a literal,
  and a function may compute one. Whether two const arguments are equal, and so whether two types
  that differ only in them are one type, is decided by these rules and no others (ODR-066):
  1. An argument whose `const` parameters are all known (every argument of an instance, after
     substitution) is its value, evaluated as `[CT-1]` evaluates; two are equal when their values
     are. An evaluation that overflows or panics is `E6004`, and one past `[CT-3]`'s limits `E6001`,
     where the argument is written; neither is ever reported as a type mismatch.
  2. In generic code, an integer argument that names the declaration's `const` parameters is compared
     by its **normal form**: a sum of terms, each an integer coefficient times a product of
     parameters and opaque terms, with like terms collected and zero terms dropped, computed in
     mathematical integers. `+`, `-`, multiplication and parentheses are what the normal form sees
     through; any other operation (`/`, `%`, a shift, a comparison, a call) is an **opaque term**,
     equal to another only when it is the same operation, or a call of the same function, on
     arguments whose normal forms are equal. Two arguments are equal when their normal forms are the
     same: `N + 1` and `1 + N` are, and `2 * N` and `N + N`; `N / 2 * 2` and `N` are not. An argument
     of another type is equal to another only when both are the same parameter.
  3. Arguments rules 1 and 2 do not show equal make different types (`E2020`). When either names a
     parameter, the diagnostic says the two could not be shown equal, rather than that they differ;
     it prints both normal forms and suggests computing the value once, in one shared expression or a
     `const` parameter taken for it.

  The rules depend only on the source, never on what an optimiser can prove. Each instance
  evaluates its arguments again (rule 1), so an equality rule 2 accepts holds at every instance whose
  arguments evaluate; an instance whose arguments overflow is `E6004` there.

## IV.8 Interfaces

An `interface` declares methods, associated types and constants, and may give default method bodies.
A type implements an interface in its header (`struct P implements Display:`) or in an `extend P
implements Display:` block.

* `[TYP-20]` *(changed in 0.9.9)* **Coherence is per package.** An implementation of interface `I`
  for type `T` may appear in any module of the package that declares `I` or the package that declares
  `T`, or, where one of `I`'s arguments is that package's own, in that package: reading `T` and then
  `I`'s arguments, the first type the package declares (one of its types, or an instance of one of
  its generic types) comes before any type parameter of the implementation standing alone
  (ODR-097: `extend f32 implements Mul[Vec3]` is written in `Vec3`'s package; `extend bool implements
  Add[bool]` and `extend Array[Vec3] implements Display` are written nowhere but `std`). An
  implementation anywhere else is `E2121`. Two implementations of one interface for one type anywhere
  in a program are `E2041`, naming both. (Separate compilation relies on this: an implementation
  cannot appear from a third package, and only one package can write each one.)
* `[TYP-24]` *(changed in 0.9.9)* *(changed in 0.9.15)* An interface method is found through any implementation visible in
  the program; the interface need not be imported. When two interfaces supply a method of one name for
  one type, the call is `E2070` and is disambiguated as `I.m(recv, …)`. Two instances of one generic
  interface (`Add[i64]` and `Add[i32]`) are told apart by a call's arguments: an instance whose
  parameter types are the arguments' own types, an untyped literal counting as its default type
  (`int`, `f64`), is chosen before one the arguments reach only through a coercion (`[RNG-5a2]`);
  arguments that fit two equally are `E2070`, disambiguated as `I[A].m(recv, …)`. An operator chooses
  among instances as its method's call does (`[TYP-21]`, ODR-096).

**Marker interfaces**, derived automatically (`Send` written by hand only with `unsafe extend`, `[TYP-43]`):

| Marker | Holds when |
|---|---|
| `Copy` | declared with `@derive(Copy)` on a struct or enum whose fields are all `Copy` and which has no `drop`, which gives it `Clone` too; always for scalars, `ref T`, `Span`, `StrView`, raw pointers, `extern fn`, tuples and fixed arrays of `Copy` elements; never for a class object, an `owns`, `shares`, `shares mut` or `thread_shared` value, a `weakref` or `sharedref mut` link, `ref mut T` or `MutSpan[T]` |
| `Send` | inferred: every component is `Send`, so nothing reachable from the value is a one-thread owner (`shares`, `shares mut`), a `weakref` link to one, or bound to its thread (a lock's guard, a `Scope`, a raw pointer); `ref T` and `Span[T]` are when `T` is `ThreadShared`, `ref mut T` and `MutSpan[T]` when `T` is `Send`; a class object is when its fields are; a `thread_shared T` value is when `T` is `ThreadShared` and `Send` (`[THR-8]`) |
| `ThreadShared` | inferred: several threads may hold and read one value of the type at once; every component is `ThreadShared`; `ref mut`, `Cell`, `UnsafeCell`, raw pointers, a `shares` or `shares mut` value and a `weakref` link to one are not (`[THR-9]`) |
| `Sized` | everything except `dyn I` and unsized foreign types |

* `[TYP-43]` *(new in 0.9.10)* *(changed in 0.9.15)* The marker interfaces are interfaces of `std.core`, exported by the
  prelude, so a bound names them as it names any (`T: Send + ThreadShared`, `where Self: Sized`), and
  the table answers them. `Send` and `ThreadShared` are inferred from what a type holds, which its
  words say. The types the compiler declares itself (a `Cell`, the counted block of a `shares` value)
  are recognised by the module that declares them, never by their names, so a program's own struct
  whose name begins `Cell_` is judged by all of its fields like any other. A program writes `Send` by
  hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend T
  implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler
  cannot check (written without `unsafe` it is `E3100`: "say that you vouch for it: write `unsafe
  extend Counter implements Send:` with a `# SAFETY:` note above it"); a generic one (`unsafe
  extend[T: Send] W[T] implements Send:`) holds for each instance whose arguments meet the markers its
  parameters are bounded by. `ThreadShared` is written only in bounds, `Copy` comes from
  `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`, as is an
  `unsafe extend` of any interface but `Send` or an `unsafe interface` (ODR-111). `Sync` is the name
  of an effect (Part X), not of a property: `T: Sync` in a bound is `E1010`, whose message gives
  `T: ThreadShared`.
  An `unsafe interface` (`Allocator`, `[ALC-2]`) is implemented only by an `unsafe extend`, with its
  members as for any interface; without `unsafe` it is `E3100`.

**Standard interfaces.** These are the definitions; `std.core` declares them and the prelude exports
them (`[MOD-5]`).

```ember,ignore
## signature sketch: the standard interfaces as std.core declares them
interface Clone:
    fn clone() -> Self                                  # what `copy of x` runs

interface Drop:
    fn drop()

interface Default:
    static fn default() -> Self

interface Eq:
    fn eq(other: Self) -> bool                          # ==, !=

interface Ord: Eq:
    fn cmp(other: Self) -> Ordering                     # sort, min, max; a float's `<` stays IEEE

enum Ordering:
    Less
    Equal
    Greater

interface Hash:
    [H: Hasher]
    fn hash(ref mut h: H)

interface Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]          # f"{x}", print(x)

interface Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]    # f"{x!r}", f"{x:?}"

interface Iterator:
    type Item
    fn next() -> Option[Item]

interface Iterable:
    type Item
    type Iter: Iterator[Item = Item]
    fn iter() -> Iter                                               # for x in c

interface IntoIterator:
    type Item
    type Iter: Iterator[Item = Item]
    fn into_iter() owns this -> Iter                                # for x in owns c

interface Contains[T]:
    fn contains(item: T) -> bool                                    # x in c

interface Add[Rhs = Self]:                                          # likewise Sub, Mul, Div,
    type Output                                                     # FloorDiv, Rem, Pow, BitAnd,
    fn add(rhs: Rhs) -> Output                                      # BitOr, BitXor, Shl, Shr

interface AddAssign[Rhs = Self]:                                    # likewise for each operator
    fn add_assign(rhs: Rhs)

interface Neg:                                                      # likewise Not (~)
    type Output
    fn neg() -> Output

interface Index[Idx]:
    type Output
    fn index(i: Idx) -> ref Output                                  # a[i] read

interface IndexMut[Idx]: Index[Idx]:
    fn index_mut(i: Idx) -> ref mut Output                          # a[i] write

interface IndexSet[Idx, V]:
    fn index_set(i: Idx, owns v: V)                                 # a[i] = v (Map inserts)

interface Error: Debug + Display:
    fn source() -> Option[ref dyn Error]                            # default: None

interface From[T]:
    static fn from(owns value: T) -> Self                           # `?` conversion; Into is derived
```

Inside each method its object is `this`; whether an implementation changes it (`next`, `add_assign`,
`index_set`) is worked out from that implementation's body, and through an interface value from every
implementation the program has (Part X).

* `[TYP-36]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Which types implement which interfaces.** The table is normative; a `—` means the
  implementation does not exist and a bound requiring it is `E2040`. A read-only view `ref T`
  implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as `T` does (so `sorted(m.keys())` works).
  An `owns T` is `Clone` when `T` is, and `owns dyn I` when `I` requires `Clone` (§IX.1, `[TYP-22]`).
  Two `owns` values compare with `==` when what they hold does; `owns dyn I` compares
  when `I` requires `Eq` (`interface Shape: Eq:`, a method-table entry as for `Clone`): values of two
  different kinds are never equal, and two of one kind compare their contents. `copy of x` makes a
  value with `Clone` (`[OWN-8]`), and every `Copy` type is `Clone`; before what it cannot copy as
  itself (a `weakref` link, a value of a view type, a type with no `Clone`) it is `E2116`. Before an `owns`, `shares` or
  `sharedref mut` value it copies what that value holds, never the link; a struct that holds links
  among its fields is copied field by field, as a C++ struct holding a `shared_ptr` and a `weak_ptr`
  is: what it owns alone is copied itself, a `shares` or `thread_shared` field becomes one more owner
  of the same value, a `weakref` field one more link to the same target and a `sharedref mut` field
  one more holder of the same link (`struct Team:` with `captain: shares Player` and `scout: weakref
  Player`: `t2 = copy of t` is a new `Team` whose captain and scout are the same player).

| Type | Clone | Copy | Eq | Ord | Hash | Default | Display | Debug |
|---|---|---|---|---|---|---|---|---|
| integers, `int` | ✓ | ✓ | ✓ | ✓ | ✓ | 0 | ✓ | ✓ |
| floats, `float` | ✓ | ✓ | IEEE `==` | totalOrder | — | 0.0 | ✓ | ✓ |
| `bool` | ✓ | ✓ | ✓ | `false < true` | ✓ | `false` | ✓ | ✓ |
| `char` | ✓ | ✓ | ✓ | by scalar value | ✓ | `'\0'` | ✓ | ✓ |
| `void` | ✓ | ✓ | ✓ | ✓ | ✓ | `()` | — | ✓ |
| `StrView` | ✓ | ✓ | ✓ | by bytes | ✓ | `""` | ✓ | ✓ |
| `String` | ✓ | — | ✓ | by bytes | ✓ | empty | ✓ | ✓ |
| tuples, `[T; N]` | if all | if all | if all | lexicographic, if all | if all | if all | if all `Debug` | if all |
| `Option[T]`, `Result[T, E]` | if `T`(,`E`) | if `T`(,`E`) | if `T`(,`E`) | `None < Some` | if `T`(,`E`) | `None` / — | if `Debug` | if `T`(,`E`) |
| `Array[T]` | if `T` | — | if `T` | lexicographic | if `T` | empty | if `T: Debug` | if `T` |
| `Map[K, V]`, `Set[T]` | if all | — | as sets of entries | — | — | empty | if all `Debug` | if all |
| class objects | `@derive(Clone)` or written: a new object | — | identity (`is`) | — | identity | — | — | ✓ (class and address) |
| range types | as the representation | as the representation | as the representation | as the representation | as the representation | — | as the representation | as the representation |
| structs, enums | derived per `[STR-5]` | `@derive(Copy)` | derived per `[STR-5]` | `@derive(Ord)` | `@derive(Hash)` | `@derive(Default)` | explicit | derived per `[STR-5]` |

* `[TYP-39]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Collections, tuples and `Option`/`Result` implement `Display` the way Python's `str()`
  shows them: `[1, 2, 3]`, `{'a': 1}`, `{1, 2}`, `(1, 'x')`, `Some(3)`, `None`, with each element
  formatted by its `Debug` implementation (strings quoted). So `println(xs)` prints a list as Python
  would. A `Map` prints its entries in insertion order and an empty one prints `{}`; an empty `Set`
  prints `set()`. A collection's `Debug` text is its `Display` text. A string's `Debug` text is
  Python's `repr`: single quotes, or double quotes when the text holds a `'` and no `"`; `\`, the
  chosen quote, `\n`, `\r` and `\t` are escaped, and any other control character is `\xNN`. An
  `owns T` and a `Cell[T]` show as the `T` they hold, in `Display` and `Debug` alike (an `owns int`
  holding 5 prints `5`; ODR-094). A `Cell` shows a copy of its value, as `get` reads it: a value's own `Display` or
  `Debug` is the program's code and may `set` the cell, and a `Cell` lends its value to nothing
  (`[CELL-2]`), so one that is not `Copy` and writes its own is not shown from a `Cell` (`E2040`;
  ODR-109).
* `[TYP-37]` *(new in 0.9.9)* Floats implement `Eq` with IEEE `==` (so `NaN != NaN`) and `Ord` with the IEEE-754
  totalOrder predicate (`-NaN < -inf < … < -0.0 < +0.0 < … < +inf < +NaN`). The comparison
  *operators* on float values keep IEEE meaning (`[TYP-9]`) in concrete and generic code alike: `a < b`
  with `T: Ord` and `T = f64` is IEEE `<`, false when either side is NaN. `cmp`, `sort()`, `sorted()`,
  `binary_search`, `min()` and `max()` use `cmp`, so sorting floats is total and needs no extra step.
  A float's `==` and `cmp(…) == Equal` can differ (`-0.0 == 0.0`, which `cmp` orders; a NaN is not `==`
  itself), so an algorithm keeps to one relation: the ordered ones to `cmp` throughout, the
  equality-based ones (`contains`, `index_of`, `dedup`) to `==` (ODR-055). Floats do not implement
  `Hash`; a float map key is written with `f.to_bits()`.
* `[TYP-21]` *(changed in 0.9.9)* *(changed in 0.9.15)* Operators desugar to these interfaces for non-scalar operands; `a + b` calls
  `Add.add(a, b)` with both operands read in place. `a += b` calls `AddAssign.add_assign` if implemented,
  else `a = a + b`. Scalar operators are built in. The interfaces' methods are `add`, `sub`, `mul`,
  `div`, `floordiv`, `rem`, `pow`, `bitand`, `bitor`, `bitxor`, `shl`, `shr`, `neg` and `not`, and
  each `…Assign` form's is its operator's followed by `_assign` (ODR-040). A type has an operator
  only by implementing its interface (`[TYP-40]`); a method that merely shares the name is not one.
  Each number type implements the interface of each operator it has, with `Rhs = Self` and `Output =
  Self`: every integer type `Add`, `Sub`, `Mul`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`,
  `BitOr`, `BitXor`, `Shl`, `Shr` and their `…Assign` forms, and every float type `Add`, `Sub`,
  `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg` and theirs, so `[TYP-17]`'s `sum` takes numbers.
  Such a method is the operator: `3.add(4)` is `3 + 4`. `bool`, `char` and `String` implement none
  of them (`String`'s `+` takes its left operand over, `[TXT-11]`). Likewise a type is indexed only
  through `Index`, `IndexMut` and `IndexSet` (ODR-042).
* `[HASH-1]` *(changed in 0.9.15)* `Hash.hash` is generic over `H: Hasher` and monomorphised; hashing a value MUST feed a
  deterministic representation of it into the hasher, and values equal under `Eq` MUST hash equally.
  `Hasher` (in `std.collections`, not the prelude) is a hashing state that is not `Copy`, with `write_bytes`,
  `write_u8`, `write_u16`, `write_u32`, `write_u64`, `write_i8`, `write_i16`, `write_i32`,
  `write_i64`, `write_usize`, `write_isize` and `finish() owns this -> u64`. A user type may
  implement it and serve as a map's hasher.
* `[HASH-2]` *(changed in 0.9.9)* `std.collections.DefaultHasher` is a fixed-seed hasher: the same
  keys hash the same way in every run and on every machine. It is a fast non-cryptographic hash, no
  slower on integer keys than one multiply and one rotate per word (FxHash's class). `std.collections.
  RandomState` is a per-process randomly seeded hasher for maps keyed by untrusted input; a `Map[K, V,
  RandomState]` still iterates in insertion order (`[STD-11]`), so the seed never becomes visible.
  `DefaultHasher`'s values are the same for one Ember version on every target (its state is 64 bits
  on a 32-bit target too), so a table built at compile time is valid at run time; they may change
  between versions, and a program MUST NOT depend on them.
* `[HASH-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Map` and `Set` MUST NOT weaken equality to compensate for an
  incoherent `Hash`. An incoherent `Hash`, a key changed through a `Cell`, or an inconsistent `Ord`
  given to `sort` gives wrong answers (a missing key, any permutation of the elements) but never
  undefined behaviour, an out-of-bounds access or an operation that does not terminate. A `hash` or
  `eq` that panics aborts the process (`[PAN-1]`), so no map is seen half-changed. What a `hash` or
  `eq` does is worked out from its body like any function's (Part X), so one that may change or end
  the map it is called from, reached by another name (a link, a module variable), makes the map
  operation that calls it refused when compiling (`E3031`); nothing is checked while the program runs.
* `[HASH-4]` `Map[K, V]` requires `K: Eq + Hash`. A map may call `hash` any number of times per
  operation. Safe map APIs never expose `ref mut K`.

## IV.9 Existential and opaque types

* `dyn I` is unsized. It is a parameter's type (a read-only view of the value; `ref mut v: dyn I` may
  change it), a view's (`ref dyn I`), or held as `owns dyn I`, `shares dyn I` or `shares mut dyn I`
  (`[TYP-45]`). It is a data pointer and a table pointer (a class object reaches its table through the
  object header, `[OBJ-2]`).
* `[TYP-22]` *(changed in 0.9.10)* *(changed in 0.9.15)* An interface is usable as `dyn` only if every method has an object
  (none is a `static fn`), does not return `Self` by value (methods marked `where Self: Sized`
  excepted), and is not generic, except over a type parameter used only as the type of a parameter
  that views its value (a plain or `ref mut` parameter), which gets one `dyn` entry (`[V: Visitor]`
  above `fn visit(ref mut v: V)`). Violations are `E2050`, naming the method and giving the line to
  write ("`Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value has one
  entry per method; take the visitor in instead of returning it, `[V: Visitor]` above `fn make(ref mut
  v: V)`"). An interface that requires `Clone`
  (`interface Shape: Clone:`) is usable as `dyn` all the same: every type implementing it must be
  `Clone`, as for any required interface, and its method table holds an entry that copies the value
  into a new block, so `owns dyn Shape` is `Clone` (§IX.1).
* `[TYP-44]` *(new in 0.9.15)* **Kind tests and casts.** A value of an interface type (`dyn I`,
  `owns dyn I`, `shares dyn I`, `shares mut dyn I`) or of a base class is tested for its kind with
  `is` (`[GRM-45]`). `if s is Circle as c:` runs its block when `s` is a `Circle` and names it `c`
  there, declared with its words as a name opened with `exists as` is (`[GRM-44]`): plain `c` is a
  read-only view, and, since a value tested in its place stays a place, `if this.pools[k] is Pool[T]
  as ref mut pool:` names a view of that pool that may change it, where the function may change the
  place. `if s is Circle:` tests without a name. A choice among kinds is `if`, `elif` and `else` of
  kind tests; there is no "maybe a `Circle`" value to `match` on. A test that always passes (`s` is a
  `Circle` already) is `E2036` ("to name it write `c = s` in place of the `if` line, and use `c` in its
  block, moved out one level"); a test that can never pass, of a type that does not implement the
  interface or derive from the base class, is `E2037` ("`s` is a `Shape`, which can never be a `Path`;
  remove the test: the `if` line and its block"); a name already in use is `E2106` ("`as` declares a
  new name: write `if s is Circle as c:`, and `c.r` for `s.r` in its block"). `cast s as Circle`
  gives the `Circle`, on `owns dyn` and `shares dyn` values too, and stops the program when `s` is not
  one, with the text "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`", in the
  program's own names and line. A cast of a place is a place: the name bound from it is read-only
  unless written `ref mut` (`pool = ref mut cast this.pools[k] as Pool[T]`), and `ref mut` taken
  through a plain one (`return ref mut pool.dense[pool.sparse[e]]`) is `E3027`. A cast between
  unrelated types is `E2020`. Before a type name `is` means "is of this kind"; before a value it means
  "the same object" (`a is b`, `x is None`). Each test and each cast is one compare of type indices.
  `as?`, `as!`, `cast s if Circle` and `cast s when Circle` are not Ember (`E1010`, `[GRM-30]`).
* `[TYP-32]` *(new in 0.9.9)* `some I` in a return type means "one concrete type, chosen by the function body, that
  implements `I`". The caller sees only `I`; the compiler knows the type and monomorphises through it,
  so there is no indirection. Every `return` in the body must produce the same concrete type
  (`E2261` otherwise). A method returns one as a function does; an interface's method does not
  (`[GRM-31]`). It is how a function returns an iterator pipeline whose type cannot be written:

```ember
fn evens(xs: Span[int]) -> some Iterator[Item = int]:
    return xs.iter().copy_each().filter(fn(x) => x % 2 == 0)
```

## IV.10 Type inference

* `[TYP-23]` *(changed in 0.9.9)* *(changed in 0.9.15)* Inference is local to a function body and bidirectional. Function
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
  lambda with unannotated parameters and no expected callable type is `E2061`. A lambda parameter's
  word is what is written on it, never inferred, and matches the callable type it is given to: where
  the type hands the value over (`fn(owns String)`) the lambda writes `fn(owns s) =>
  println(s.len())`, and `fn(s)` there is `E2020` ("`each_name` hands each name over (`fn(owns
  String)`), so this function receives it as its own: write `fn(owns s) => println(s.len())`"); a plain
  parameter reads, and one that changes the caller's value says `ref mut` (`fn(ref mut ui)`), as the
  type does (`fn(ref mut Ui)`). Within one expression, untyped literals are resolved last (`1 + x`
  takes `x`'s type).

## IV.11 Method resolution and calls

For `recv.m(args)`:

1. Strip views from `recv`'s type `R` (`ref`, `ref mut`, `sharedref mut`), and one level of `owns`,
   `shares`, `shares mut` or `thread_shared`. A `weakref` link is not stripped: it is opened first
   (`[GRM-44]`), and a method called on it as if on its target is `E2020`.
2. Look for an inherent method `m` on `R`, then on `R`'s base classes nearest first, then in any
   interface `R` implements (`[TYP-24]`), then a field of `R` whose type is callable (`[CLO-11]`). An
   inherent method beats an interface method of the same name.
3. The object a method is called on is not marked (`a.reset()`): inside the method it is `this`, and
   what the method does to it is worked out from its body (Part X). A method that may change its
   object is not called through a name that only reads it (`E3023`), and one written `owns this`
   takes its object over, so the caller's name ends (`E3040` at a later use) (`[FN-4]`).

* `[TYP-25]` Arguments may be positional or named; positional arguments come first; a parameter with a
  default may be omitted.
* `[TYP-26]` *(changed in 0.9.9)* There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. The
  compiler-known output functions `print`, `println`, `eprint`, `eprintln` (`[STD-9]`) and `format_to`
  (`[STD-18]`) are the only calls with a variable number of arguments (ODR-060).
