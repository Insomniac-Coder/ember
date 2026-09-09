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

`[TYP-1]` Every concrete type has compile-time-known `size`, `align`, `is_copy`, `is_send`, `is_sync`, `needs_drop`, `is_view`, `has_niche`. The compiler computes these in the `TypeInfo` table (Part XIX §4.3).

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

`[TYP-4]` **No implicit conversions between scalar types in operators.** `i32 + i64` is `E2020`. `[TYP-5]` **Coercion sites** (assignment, argument, return, field initialiser, array element) allow **lossless widening**: `iN → iM` (M>N), `uN → uM` (M>N), `uN → iM` (M>N), `f32 → f64`, `f16 → f32`; and **range erasure**: a value of a range type `T` over representation `R` (`[RNG-1]`) coerces to `R`. Range erasure is a distinct coercion step that composes with the widening rules above, so `Roughness → f32 → f64` and `Percent → u8 → u32` are coercions. **No coercion produces a range type** — construction is `[RNG-3]`/`[RNG-3a]`. Nothing converts to/from `bool` or `char` implicitly. `[TYP-6]` Everything else uses `as`:

* `x as T` for numeric types: truncation for narrowing integers (bit truncation), float→int saturating with NaN→0 (Rust semantics), int→float round-to-nearest.
* `x as u8` from `char`, `x as char` from `u8` only (wider ints via `char.from_u32() -> Option[char]`).
* `[TYP-7]` `as` between pointer types and between pointer and `usize` requires `unsafe`.

**Integer overflow** `[TYP-8]`: in the `debug` profile every arithmetic operation that overflows panics with `E-panic: integer overflow` and the source location. In `release`/`shipping`, `+ - *` and `<<` wrap two's-complement; `/` and `%` by zero always panic; `i32.MIN / -1` always panics. `@overflow(panic|wrap|saturate)` on a function or module overrides the profile. Explicit methods `wrapping_add`, `checked_add -> Option`, `saturating_add`, `overflowing_add -> (T, bool)` always exist.

**Floating point** `[TYP-9]`: strict IEEE semantics; no fast-math, no FMA contraction, no reassociation unless the function is `@fastmath`, in which case the C backend emits `#pragma float_control(precise, off)`/`-ffast-math`-equivalent attributes for that function only. `NaN == NaN` is false; `Ord` is not implemented for floats — use `partial_cmp` or `total_cmp`.

**Shifts** `[TYP-10]`: shift amount ≥ bit width panics in debug and is masked in release (like Rust).

## IV.2a Range and domain types

A range type is a nominal numeric type that carries its own bounds. Two of them
over the same representation are different types, so a roughness cannot be
passed where a metallic is wanted even though both are `f32` — which is the
point, because that confusion is not detectable in any other way.

```ember
type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0
type Fov       = f32 in 1.0 ..= 179.0
type Percent   = u8  in 0 ..= 100

fn demo():
    r: Roughness = 0.5      ## in range at compile time; no check is emitted
    m: Metallic  = r        ## E2210: `Roughness` is not `Metallic`
```

A value the compiler cannot place in range is converted fallibly:

```ember
fn from_slider(x: f32) -> Result[Roughness, RangeError]:
    return Roughness.checked(x)

fn clamped(x: f32) -> Roughness:
    y = min(max(x, 0.0), 1.0)
    return Roughness.checked(y).unwrap()      ## [RNG-4] discharges the check
```

* `[RNG-1]` A `type` alias carrying an `in` clause declares a **nominal**
  numeric type over the named representation, restricted to that range. A `type`
  alias without one is transparent, exactly as `[LEX-15a]` specifies. The clause
  takes a range expression (`a .. b` or `a ..= b`) whose endpoints are constant
  expressions of the representation type.
* `[RNG-2]` Two range types are distinct types even when representation and
  range are identical (`E2210`). A range type converts to its representation
  implicitly; the reverse requires `[RNG-3]`. The implicit conversion this rule names is `[TYP-5]`'s range erasure and is admitted at `[TYP-5]`'s coercion sites only; a range type never converts implicitly in operator position except through `[RNG-5a1]`'s generated impls.
* `[RNG-3]` Construction from a value not statically known to be in range is
  `T.checked(v) -> Result[T, RangeError]`. Construction from a constant in
  range, or from a value whose known range is contained in the target's, emits
  no check.
* `[RNG-4]` The compiler tracks a known range for every numeric expression it
  can — literals, `min`/`max`/`clamp`, the arms of an `if` or `match` that
  compared the value, and arithmetic on operands with known ranges — and uses it
  to discharge `[RNG-3]`'s check and `[TYP-8]`'s overflow check. A range fact is
  never derived from inside a `@fastmath` function. A range fact derived from an arithmetic operation is that operation's **mathematical** range only when the mathematical range is contained in the representation's range — that is, when the operation provably cannot overflow, in which case `[TYP-8]`'s check for it is discharged by the same fact. Otherwise the derived range is the representation's full range, or, under an effective `@overflow(saturate)`, the mathematical range clamped to the representation's. A range fact MUST NOT be derived from the mathematical range of an operation that can overflow, **in any profile**: `[TYP-8]`'s policy differs between `debug` and `release`, and `[PRF-1]` and `[PHIL-5]` forbid the set of checks the compiler emits from depending on that difference. **`[RNG-5a]`'s range-preserving clamp family is exempt**: `min`, `max` and `clamp` cannot overflow, so their mathematical range is always the computed one and `T.clamped` keeps its fact.
* `[RNG-5]` Arithmetic involving a range type normally yields its **representation**,
  not the range type: `r * 2.0` is `f32`. A range value MAY participate directly in
  arithmetic with an ordinary value of its representation type. Arithmetic between
  two **distinct nominal range types** is rejected (`E2214`) unless at least one
  operand is explicitly converted to its representation type. Thus `Roughness +
  Roughness` is permitted, `Roughness * 2.0` is permitted, but `Roughness +
  Metallic` is rejected. Producing a range type again is a construction and goes
  through `[RNG-3]`. This preserves the range types' nominal purpose without
  inventing a range-propagating result type for every operator.
* `[RNG-6]` Range reasoning over floats obeys `[TYP-9]`'s strict IEEE semantics.
  NaN is in no range. A range with endpoints `-0.0` and `+0.0` contains both
  zeros. `[TYP-9a]`'s prohibition on contraction is what keeps a range fact from
  being invalidated by an FMA the backend introduced.
* `[RNG-7]` A range type is a niche for `[TYP-13]`: `Option[Percent]` occupies
  one byte. "…`Option[Percent]` occupies one byte. **A range type supplies a niche only where its range does not exhaust its representation**: `type Full = u8 in 0 ..= 255` has no invalid value and `Option[Full]` is two bytes. The compiler MUST NOT claim a niche it does not have."
* `[RNG-8]` "…and crosses an FFI boundary as its representation (`[FFI-5]`). A range type is not itself writable in a foreign signature (`[RNG-10b]`); a value arriving from foreign code enters at the representation type and becomes a range value only through `[RNG-3]` or `[RNG-3a]`."
  * `[RNG-9]` **Range validity is an invariant, not a convention.** A value of a range type whose representation does not lie in the declared range is **invalid**; producing one is undefined behaviour and requires `unsafe`, exactly as `[TYP-2]` provides for `bool`. `[RNG-4]` MAY assume validity, which is what licenses it to discharge `[TYP-8]`'s overflow check and `[OPT-2]`'s bounds check. Invalidity is not merely a wrong number: by `[RNG-7]` a range type is a niche, so an out-of-range representation is a bit pattern that is neither a payload nor a discriminant.
  * `[RNG-10]` **The construction set is closed.** In Safe code a range-typed value arises only from: (a) a constant the compiler placed in range (`[RNG-3]`); (b) `T.checked(v)`; (c) `T.clamped(v)` (`[RNG-3a]`); (d) a value whose `[RNG-4]` range is contained in the target's; (e) a copy or move of an already-valid value. Any other route is `unsafe` and is `T.new_unchecked(v)`, whose safety condition is written in its documentation and whose `debug` build MUST `debug_assert` the range. Constructing a range-typed value outside this set in Safe code is `E2215`. * `[RNG-10a]` A generated `deserialize` (`@derive(Deserialize)`, XIV.4) MUST emit `T.checked(...)` for every range-typed field, transitively through nested aggregates, and MUST map a failure to `SerError`. A derive that omits the check is a compiler defect, not a performance option. * `[RNG-10b]` A range type MUST NOT appear as a parameter type, return type or field type in an `extern` declaration, an `@ffi` overlay signature, or an imported foreign type — **nor in any aggregate transitively containing one, nor as the pointee of any pointer passed to or returned from foreign code.** It is `E5054`, whose help is to declare the representation and construct with `T.checked(...)` in the Ember-side wrapper (`[FFI-13]`). An `unsafe extern` does not discharge this: `[TIER-1]`'s boundary lets the programmer assert a signature, not a range, and `[FFI-38]`'s "the importer MUST reject rather than guess" applies. * `[RNG-10c]` `unsafe` reads through a raw pointer, `mem` reinterpretation and uninitialised storage may produce a value at a range type; that is the `unsafe` block's obligation under `[UNS-4]`, and `[UNS-*]` is unchanged.
  * `[RNG-3a]` **Total construction.** Every range type whose endpoints are finite provides `T.clamped(v: Repr) -> T`, defined as `min(max(v, lo), hi)` for an inclusive range and as the nearest representable value strictly inside a half-open one. It is **total**: it has no failure mode and introduces no `Panic` and no `RuntimeCheck(k)`, and `[EFF-16]` is amended to state that `T.clamped` introduces no `Panic(Explicit)`. For a float representation, `NaN` maps to `lo` — which the `docs/errors/` reference page MUST state — and `-0.0`/`+0.0` follow `[RNG-6]`. On the C backend it lowers to two compares or the target's `min`/`max` instruction pair, **strictly cheaper than `[RNG-3]`'s check-and-branch-to-panic**. `T.checked` remains for code that must distinguish an out-of-range input from a clamped one.
  * `[RNG-5a]` **The clamp family is range-preserving.** Where `min`, `max` or `clamp` is applied to operands of one range type `T`, or to a `T` and constants of its representation lying within `T`'s range, the result is `T`, not the representation. This is the one exception to `[RNG-5]` and is sound because the result's range is contained in `T`'s by construction; `[RNG-4]` discharges it with no emitted check. FIX-013's `[RNG-4]` amendment **exempts this case explicitly**, so `clamped` keeps the fact that makes it free. `L2003` warns where a fallible construction (`T.checked`) is written and a total one (`T.clamped`, or a literal the compiler can place in range) would do, because a `Result` the programmer immediately unwraps is a panic path that need not exist.
  * `[RNG-4a]` **Float facts come only from the true arm.** Over a float representation, a range fact is derived only from the **true** arm of a comparison. `not (x > hi)` does not establish `x <= hi`: `[RNG-6]` puts NaN in no range and NaN fails both comparisons, so the false arm of a float comparison establishes no bound. A fact derived from a float comparison MUST record whether NaN is excluded, and a fact that does not exclude NaN MUST NOT discharge a `[RNG-3]` construction.
  * `[RNG-5a1]` **Operators on range types resolve through interfaces, not through a built-in rule.** For every range type `T` over representation `R` the compiler generates, **in `T`'s declaring module**, the impls `T: Add[T, Output = R]`, `T: Add[R, Output = R]`, `R: Add[T, Output = R]` and the corresponding `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `PartialEq`, `PartialOrd` forms, defined by erasing each operand to `R` (`[TYP-5]`) and applying `R`'s operator. Because they are emitted in the type's own declaring module they satisfy `[TYP-20]`'s orphan rule as written and **need no exemption**. No `*Assign` form is generated: `r += 1.0` would produce an `R` where a `T` is required and is `E2214`, whose help names `r = Roughness.clamped(r + 0.1)` or the fallible form. An operator with operands of two **distinct** range types resolves to no generated impl and is `E2214`, whose help names the explicit erasure — `Roughness + Metallic` is therefore rejected by ordinary overload resolution, not by a special case.
  * `[RNG-5a2]` **Exact impl before coercion.** `[TYP-24]`'s resolution selects an impl matching the operand types **exactly** before applying any `[TYP-5]` coercion. Without this, `Roughness + Roughness` matches both the generated `Roughness: Add[Roughness]` and, after erasure, `f32: Add[f32]`, and resolution is ambiguous.

Diagnostics: `E2210` a value of one range type where another was expected;
`E2211` a constant outside the target's range; `E2212` an `in` clause whose
endpoints are not constants of the representation, or are inverted; `E2213` an
`in` clause on a non-numeric representation.

```text
error[E2211]: 1.4 is outside `Roughness`

    roughness: Roughness = 1.4
                           ^^^ `Roughness` holds 0.0 ..= 1.0

  = help: clamp it, or take the fallible form: `Roughness.checked(1.4)`
```

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

* `[TYP-16]` Generic functions and types are **monomorphised**: every distinct instantiation produces a distinct symbol. Code size is the programmer's responsibility; the compiler deduplicates identical instantiations across modules at link time (COMDAT in the C backend via `inline`/`selectany`, weak symbols via LLVM). XIX §4.11a gives the programmer the instrument that sentence assumes: a count, a report, a budget, and — for a generic that uses its parameter only to call its bounds' methods — the option of one shared function in place of the set.
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

The algorithm is Hindley–Milner-style unification over a per-function inference table (union–find of type variables with occurs check), with interface-bound obligations collected and solved after unification (Part XIX §4.4). There is no let-polymorphism for locals.

## IV.11 Method resolution and auto-ref

For `recv.m(args)`:

1. Determine the type `R` of `recv` after stripping `ref`/`ref mut` and class-handle indirection (auto-deref), at most one level of `Box`/`Shared` deref.
2. Look for an inherent method `m` on `R`, then on `R`'s base classes (nearest first), then in interfaces implemented by `R` that are in scope (imported), then in `dyn` vtables. `[TYP-24]` An inherent method always beats an interface method of the same name; ambiguity between two interfaces is `E2070` (disambiguate with `I.m(recv, ...)`).
3. Adjust the receiver to the method's declared mode: `self` → shared borrow (or copy of a handle), `mut self` → mutable borrow (`E3xxx` if `recv` is not mutable/not a mutable place), `owned self` → move (or handle copy for classes).

Named arguments `f(x=1, y=2)` match parameter names; positional arguments MUST precede named; `[TYP-25]` parameters with defaults may be omitted. Overloading by arity or type is **not** supported (use defaults, generics, or distinct names); `[TYP-26]` two functions with the same name in one scope is `E1030` — except operator interface impls and `extend` blocks for distinct types.

---

