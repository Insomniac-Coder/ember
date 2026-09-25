---

# Part XV — The Standard Library

The standard library is one package, `std`. This Part fixes what every implementation provides and
what each operation means; `std/**/*.em` holds the exact signatures and becomes normative for
anything this Part leaves open.

## XV.1 Organisation and conventions

* `[STD-6]` `std` is layered, and each layer depends only on the ones before it: **core** (scalars,
  `Option`, `Result`, views, fixed-capacity containers, math, SIMD), **alloc** (`Array`, `String`,
  `Box`, `Map`, `Set`, `Shared`), **sync** (atomics, locks, channels, threads, jobs), **io** (console,
  files, time, processes) and **ffi**. A package that declares `[package] layers = ["core"]` may use
  only core; this is how firmware and kernels use Ember.
* `[STD-1]` Every function in `std.core`, `std.mem`, `std.math`, `std.simd` and `std.arena` is
  `@noalloc` unless its documentation says it allocates.
* `[STD-13]` *(new in 0.9.9)* **One naming convention.** Names are `snake_case` words joined by
  underscores (`starts_with`, `to_upper`, `push`, `is_empty`). Where Python's name for a **method**
  differs (`append`, `strip`, `startswith`, `upper`, `splitlines`), there is no second name: the call
  is `E2072`, `no method 'append' on Array[int]`, whose help names the Ember method
  (the full table is Appendix E). A Python name that Ember keeps (`partition`, `items`, `count`, and
  the built-in functions of `[STD-26]`) keeps Python's meaning (`[PHIL-14]`).
* `[STD-26]` *(new in 0.9.9)* **Python's built-in functions.** The prelude has these functions, with
  Python's meaning; each borrows its argument as a `for` loop would (`[CTL-1]`):
  * `len(x) -> int` is `x.len()` for a collection, view or range. `len` of a `str` or `String` is
    `E2073`: Python counts characters and Ember's `s.len()` counts bytes, so the help offers both
    `s.char_count()` and `s.len()`.
  * `range(stop)`, `range(start, stop)`, `range(start, stop, step)` count as Python's do, including a
    negative `step`; `step == 0` panics. `range(n)` is `0..n`, and every form compiles to a counted
    loop (`[CTL-3b]`).
  * `enumerate(it, start=0)`, `zip(a, b)` (stops at the shorter) and `reversed(x)` (for what can run
    backwards) return lazy iterators, as Python 3's do.
  * `sum(it, start=0)`, `any(it)` and `all(it)` consume an iterable (of numbers, of `bool`s);
    `any(x > 0 for x in xs)` uses a generator expression (`[GRM-38]`).
  * `sorted(it, key=None, reverse=false) -> Array[T]` returns a new sorted array (stable).
  `min` and `max` stay two-argument functions; `xs.iter().min()` gives an `Option` for a collection.
* `[STD-14]` *(new in 0.9.9)* Failure follows `[ERR-13]`: a caller's bug panics and has a
  non-panicking twin; a failure of the world returns `Result` or `Option`. Every allocating function
  is documented as allocating and carries `Alloc`.

| Module | Contents |
|---|---|
| prelude | the names of `[MOD-5]` |
| `std.core` | scalars, `Option`, `Result`, `AnyError`, `Ordering`, ranges, `NonZero`, the standard interfaces (§IV.8), iterator adapters |
| `std.mem` | `size_of`, `align_of`, `offset_of`, `swap`, `replace`, `take`, `drop`, `forget`, `keep_alive`, `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Volatile`, `Layout`, `Allocator`, `Global`, `ptr`, `assert_disjoint`, `assert_disjoint_or_panic`, `assume_disjoint` (Part IX) |
| `std.cell` | `Cell`, `RefCell`, `Ref`, `RefMut`, `UnsafeCell` |
| `std.collections` | `Array`, `Map`, `Set`, `Deque`, `SmallArray`, `BitSet`, `FixedArray`, `FixedString`, `RingBuffer`, `Pool`, `Handle`, `SoA`, `ArenaArray`, `ArenaMap`, `ArenaSoA`, `DefaultHasher`, `RandomState`, `Hasher`, `CapacityError`; the iterator types `SpanIter`, `MutSpanIter`, `SpanChunks`, `MutSpanChunks`, `ArenaArrayIter`, `ArenaArrayIterMut`, `ArenaMapIter` (public, not in the prelude) |
| `std.string` | `String`, `str` methods, `StringBuilder`, `ParseError`, `Utf8Error` |
| `std.fmt` | `Formatter`, `FmtError`, `format`, `format_to`, the format-spec mini-language (`[LEX-19]`) |
| `std.math` | constants, scalar functions, `Vec2/3/4`, `IVec2/3/4`, `UVec2/3/4`, `Mat2/3/4`, `Quat`, `Transform`, `Aabb`, `Sphere`, `Ray`, `Plane`, `Frustum`, `KahanSum`, `det` (`[DET-4]`) |
| `std.simd`, `std.cpu` | Part XII; `prefetch`, `pause`, `cycle_counter`, `cache_line_size`, `logical_cores` |
| `std.arena` | `Arena`, `FixedArena`, `ScopedArena`, `ThreadArena` |
| `std.io`, `std.fs`, `std.path` | `stdin`, `stdout`, `stderr`, `Read`/`Write`, `BufReader`, `BufWriter`, files, directories, `Path`, `PathBuf`, `io.Error` |
| `std.time` | `Instant`, `Duration`, `SystemTime`, `sleep` |
| `std.process`, `std.env` | `args`, `args_os`, `exit`, `abort`, `Command`; `var`, `vars`, `current_dir` |
| `std.random` | `Rng` (seeded, deterministic), `Rng.from_entropy()` (`Nondet`) |
| `std.thread`, `std.sync`, `std.jobs` | Part XI |
| `std.ecs` | Part XII |
| `std.ser` | Part XIV |
| `std.testing`, `std.debug` | `assert_approx_eq`, `expect_panic`, the `bench` harness (Part XVII); `backtrace`, `leak_report`, `alloc_stats`, and profiler zones `with debug.zone("name"):` |
| `std.ffi` | `CString`, `cstr`, `c_int` and the other C aliases, `c_wchar`, `WideString`, `ForeignBox`, `Retained`, `Callback`, `adopt` (Part XVI) |
| `std.gpu` | Annex D |

## XV.2 Console output, input and formatting

* `[STD-9]` *(new in 0.9.9)* `print(a, b, …, sep=" ", end="")` and `println(a, b, …, sep=" ",
  end="\n")` write their arguments' text to standard output separated by `sep`;
  `eprint`/`eprintln` write to standard error. They are compiler-known and are the only calls with a
  variable number of positional arguments (`[TYP-26]`). An argument's text is its `Display` text when
  its type has `Display`, and its `Debug` text otherwise, which is Python's `str` → `repr` fallback; a
  type with neither (a class without them) is `E2040`, whose help is to implement `Display`. `println()` prints an empty line.
* `[STD-2]` Printing allocates only to build an f-string argument; `println("literal")`,
  `println(some_str)` and `println(n)` for a number are `@noalloc`.
* `[STD-10]` *(new in 0.9.9)* `input(prompt="") -> String` prints the prompt, flushes, reads one line
  and returns it without the line ending. The prelude's console functions treat a console failure as
  fatal, as Python does when the exception is not caught: end of input or a closed stream panics, and
  the message names the `Result` forms in `std.io` (`stdin().read_line()`).
* `[STD-18]` *(new in 0.9.9)* `format(value, spec="") -> String` formats one value with a format spec
  (`[LEX-19]`), like Python's `format`. `format_to(buf: MutSpan[u8], …) -> Result[str, FmtError]` formats
  into a caller's buffer and never allocates. An f-string lowers to a `StringBuilder` sized from its
  literal parts.

## XV.3 Text

* `[TXT-1]` `str` is a borrowed `Span[u8]` known to be valid UTF-8; `String` owns a heap buffer with the
  same guarantee. Safe code may rely on the invariant (`[UNS-4]`).
* `[TXT-2]` Nothing becomes a `str` without validation: every conversion from bytes or foreign text
  returns `Result[str, Utf8Error]`, or a `_lossy` form that substitutes U+FFFD. An importer or binding
  that would hand foreign bytes to a `str` without validation is `E5063`.
* `[TXT-3]` `str` and `String` are a pointer and a length, not null-terminated, and may contain `\0`. A
  C boundary uses `cstr`/`CString` (Part XVI); a `c"…"` literal containing a NUL is `E5064`.
* `[TXT-4]` *(changed in 0.9.9)* Slicing a string, `s[a..b]`, is by byte offset and panics in every
  profile when a bound is not on a character boundary; `s.get(a..b) -> Option[str]` is the total form.
  There is no indexing by character position, because it would be O(n).
* `[TXT-5]` `str` → `String` allocates and copies; `String` → `str` is free; `Span[u8]` → `str`
  validates in O(n) without allocating; `str` → `CString` allocates.
* `[TXT-6]` Across the C ABI a `str` is `{const uint8_t *ptr; size_t len}`.
* `[TXT-7]` Text is UTF-8 everywhere. `std.ffi.WideString` converts, explicitly, for APIs that need
  UTF-16.
* `[TXT-8]` `str` is a view type and carries a region, like every `Span`.
* `[TXT-9]` *(new in 0.9.9)* **A string literal initialises a `String`.** Wherever a `String` is
  expected — an initialiser, an argument, a field, a collection element, a return value — a string
  literal produces a new `String` holding its text. The allocation happens at the literal and is
  reported there (`Alloc`); where a `str` is expected, a literal is a static `str` and allocates
  nothing. A `str` *value* never converts implicitly: `s.to_string()` or `String.from(s)` is written.
* `[TXT-10]` *(new in 0.9.9)* **`str` operations.** `len()` is the length in bytes and
  `char_count()` in characters; `is_empty`, `chars`, `char_indices`, `bytes`, `as_bytes`, `lines`,
  `split(sep)`, `split_whitespace()` (Python's `split()` with no argument), `split_once(sep) ->
  Option[(str, str)]`, `partition(sep) -> (str, str, str)` (Python's), `trim`, `trim_start`, `trim_end`,
  `starts_with`, `ends_with`, `find(sub) -> Option[int]`, `rfind`, `count(sub)`, `replace(old, new)
  -> String`, `to_upper`, `to_lower`, `repeat(n)`, `parse[T]() -> Result[T, ParseError]`,
  `to_string`, `get(range)`, `is_char_boundary(i)`, and `x in s` (`[STD-8b]`). Methods returning
  several strings return iterators of `str` borrowing `s`. Case mapping and `char_count` are
  Unicode-aware; nothing depends on the locale. `parse[T]()` (ODR-029) reads the whole text as an
  integer type, a float type, `bool` or `char`, skipping no white space (`s.trim().parse[int]()`): an
  integer is an optional sign (`-` only for a signed type) and ASCII decimal digits, and must fit `T`;
  a float is an optional sign and `inf`, `infinity` or `nan` in any case, or decimal digits with an
  optional `.`, fraction and exponent (`e` or `E`, an optional sign, digits), rounded to the nearest
  `T`; a `bool` is `true` or `false`; a `char` is exactly one character. `ParseError`, in `std.string`,
  is a unit-only enum, `Empty`, `Invalid` or `Overflow`, naming the first problem from the left.
* `[TXT-11]` *(new in 0.9.9)* **`String` operations.** Everything `str` has (by read-through), plus
  `String()`, `with_capacity(n)`, `push(c: char)`, `push_str(s)`, `insert`, `remove`, `truncate`,
  `clear`, `capacity`, `reserve`, and `as_str`. `a + b` for `a: String, b: str` consumes `a` and
  returns it extended; `s += t` appends. `str + str` is `E2040` whose help is an f-string.

## XV.4 Collections

* `[STD-15]` *(new in 0.9.9)* **`Array[T]`** is a growable contiguous list (Python's `list`).
  `len`, `is_empty`, `capacity`, `reserve`, `push`, `pop -> Option[T]`, `insert(i, v)`, `remove(i) -> T`,
  `swap_remove(i) -> T`, `extend(iterable)`, `truncate`, `clear`, `retain(pred)`,
  `drain(r) -> Array[T]`, `dedup`, `reverse`, `sort` (stable; `T: Ord`; an inconsistent order
  permutes but never corrupts, `[HASH-3]`), `sort_by_key[K: Ord](f: fn(T) -> K)`,
  `sort_by(cmp: fn(T, T) -> Ordering)`, `sorted() -> Array[T]`, `binary_search(x) -> Result[int, int]`,
  `index_of(x) -> Option[int]`, `first`, `last`, `get(i) -> Option[ref T]`, `get_mut`, `iter`,
  `iter_mut`, `split_at`, `chunks(n)`, `chunks_mut(n)`, `windows(n)`, `join(sep)` (for elements that
  are `Display`), and `x in xs` (`[STD-8]`). `xs[i]` takes any integer and panics when `i < 0` or
  `i >= len` (`[TYP-31]`). `sort_by` and `sort_by_key` are stable, and `sort_by_key` calls `f` once
  for each element, in order. `windows(n)` yields every run of `n` neighbours, one step apart, as
  shared views (there is no `windows_mut`); none when `n > len`, and `n == 0` panics as `chunks(0)`
  does (`[SPN-4]`). `drain(r)` moves the elements in `r`, any range of integers (`a..b`, `a..=b`,
  `a..`, `..b`), out into a new `Array` in order, and the rest close up; a range reaching outside
  `0..=len`, or starting after it ends, panics.
* `[STD-11]` *(new in 0.9.9)* **`Map[K, V, H = DefaultHasher, A = Global]`** is a hash map that **iterates in
  insertion order**, like Python's `dict`. Re-assigning an existing key keeps its position; removing
  a key keeps the order of the others. Lookup, insertion and removal take expected constant time,
  amortised: a map may grow, or close up removed entries, inside an insertion or a removal;
  iteration is linear in the number of entries. `H: Hasher + Default`, and a map makes a fresh `H`
  for each hash; `A` is its allocator (`[ALC-1]`). `Set[T, H = DefaultHasher, A = Global]` is the
  same with no values. Keys, values and elements are not view types (`E3063`, whose help names
  `String`): a view inside would make the map a view (`[TYP-34]`), confined to locals (`[TYP-15]`). Because order is fixed by the program's operations and the default hasher is
  fixed-seed (`[HASH-2]`), iterating a `Map` or `Set` is not `Nondet`.
* `[STD-16]` *(new in 0.9.9)* **`Map` operations.** `m[k]` reads the value and panics when the key is
  missing (`key not found: <Debug of k>; use .get(k) for an Option`); `m[k] = v` inserts or replaces
  (`[STD-17]`); `m[k] += 1` requires the key to exist and panics with the same message. The
  lookups take any `q: Q` with `Q: AsKey[K]` (`[STD-12]`): `get(q) -> Option[ref V]`,
  `get_mut(q) -> Option[ref mut V]`, `get_or(q, default) -> V` (for `V: Copy`),
  `contains_key(q) -> bool`, `remove(q) -> Option[V]`, `m[q]` and `q in m`. `insert(k, v) ->
  Option[V]` and `entry(k)` take the key itself; `entry(k)` is a `MapEntry` with `or_insert(v)`,
  `or_insert_with(f: fn() -> V)` and `or_default()` (for `V: Default`), each returning `ref mut V`.
  `keys()`, `values()`, `values_mut()` and `items()` are the view iterators
  `MapKeys`, `MapValues`, `MapValuesMut` and `MapItems`, yielding `ref K`, `ref V`, `ref mut V` and
  `(ref K, ref V)` in insertion order (`for k, v in m.items():`); `for k in m:` is by key, as
  Python's `dict`. Also `Map[K, V]()`, `with_capacity(n)`, `len`, `is_empty`, `capacity`,
  `reserve(n)`, `try_reserve(n)`, `clear`, `retain(keep: fn(ref K, ref V) -> bool)`,
  `pop_item() -> Option[(K, V)]` (the newest entry, as `dict.popitem`), `update(other)` (inserts
  `other`'s entries in its order) and `into_items() -> Array[(K, V)]` (consumes the map, in order).
  `m1 == m2` compares as sets of entries: the same length, and every key of one in the other with an
  equal value; order does not matter. `sorted(m)` sorts the keys into an `Array[K]` of clones.
  `Set`: `Set[T]()`, `with_capacity(n)`, `len`, `is_empty`, `capacity`, `reserve(n)`,
  `try_reserve(n)`, `clear`; `add(x) -> bool` (whether `x` was new; an element already there keeps
  its position); taking `q: AsKey[T]`, `remove(q) -> bool`, `contains(q) -> bool` and `q in s`;
  `retain(keep: fn(ref T) -> bool)`, `pop() -> Option[T]` (the newest), `iter()` (a `SetIter`
  yielding `ref T`); `union`, `intersection`, `difference` and `symmetric_difference` of a borrowed
  `Set` return a new `Set` (for `T: Clone`), this set's elements first in its order and then the
  other's; `is_subset`, `is_superset`, `is_disjoint`; and the operators `|`, `&`, `-` and `^`.
  `s1 == s2` compares as sets.
* `[STD-12]` *(new in 0.9.9)* **Borrowed keys.** Lookup methods, `m[k]` and `k in m` accept any key
  type `Q` with `Q: AsKey[K]`: `K` itself, and `str` for `K = String`. `interface AsKey[K]: Hash` (in
  `std.collections`, not the prelude) has `is_key(self, key: K) -> bool` and `to_key(self) -> K`. A
  lookup hashes `q` and compares it with `is_key`, so it never converts; `m[k] = v` converts with
  `to_key()` only when the key is new. Every `K: Eq + Hash` is `AsKey[K]` (`is_key` is `==`;
  `to_key` clones, so `m[k] = v` with a `K` needs `K: Clone`, where `insert` moves the key); std
  implements `AsKey[String]` for `str`. An implementation MUST hash and compare as `to_key()` would;
  one that does not gives `[HASH-3]`'s wrong answers, never undefined behaviour.
* `[STD-17]` *(new in 0.9.9)* **Index assignment.** `a[i] = v` calls `IndexSet.index_set(i, v)` when
  the type implements `IndexSet[Idx, V]`, and otherwise assigns through `IndexMut` (dropping the old
  value). `Map` implements `IndexSet`; `Array` does not need to. A compound assignment `a[i] op= v`
  always goes through `IndexMut`.
* `[STD-7]` Fixed-capacity containers with no heap allocation: `FixedArray[T, N]`, `FixedString[N]`
  and `RingBuffer[T, N]`; `push` returns `Result[void, CapacityError]` and `push_or_drop` discards when
  full. Also `Deque[T]`, `SmallArray[T, N]` (inline up to `N`, then heap), `BitSet`.
* `[STD-8]` **`Contains`.** `x in c` requires `c: Contains[typeof(x)]` (`E2226` otherwise, whose help
  is `c.iter().any(fn(e) => e == x)`). `std` implements it for `Set` and `Map` (by key); for `Array`,
  `Span`, `MutSpan` and `[T; N]` with `T: Eq`, as a linear scan; for `str`/`String`, as a substring or
  character test; for ranges, as two comparisons. The compiler never synthesises an implementation.
* `[STD-8a]` `ember inspect --cost` reports which `contains` a use selects and its complexity, so
  `x in array` is visibly linear and `x in set` visibly constant.
* `[STD-8b]` `str` implements `Contains[char]` and `Contains[str]` only; matching happens at character
  boundaries, so a character never matches inside another's encoding. A byte needle (`Span[u8] in s`)
  is `E2226`; byte search is on `s.as_bytes()`. `x not in c` is `not c.contains(x)`, with each operand
  evaluated once.

## XV.5 Iterators

* `[STD-19]` *(new in 0.9.9)* Every `Iterator` has the adapters `map`, `filter`, `filter_map`,
  `enumerate`, `zip`, `chain`, `take`, `skip`, `take_while`, `skip_while`, `step_by`, `flat_map`,
  `flatten`, `peekable`, `copied`, `cloned`, `inspect`, and `rev` where the iterator can run backwards;
  and the consumers `count`, `sum`, `product`, `min`, `max`, `min_by_key`, `max_by_key`, `fold`,
  `reduce`, `any`, `all`, `find`, `position`, `last`, `nth`, `min_by`, `max_by`, `for_each`,
  `collect[C]()`, `to_array()`,
  and `join(sep)` for `Display` items. Adapters are lazy and allocate nothing; a chain of adapters in a
  `for` loop compiles to one loop with no iterator object in memory (`[CTL-3]`). The adapters and
  consumers can also be called directly on an `Iterable` value, borrowing it as `iter()` would:
  `xs.enumerate()` is `xs.iter().enumerate()`, and a method of the collection's own of the same name
  (`Array.join`) takes precedence.
* `[STD-5]` *(changed in 0.9.9)* `sum` and `product` combine elements left to right in the element
  type; an empty sum is zero. Integer sums panic on overflow (`[TYP-8]`). Float sums are never
  reassociated outside `@fastmath` (`[TYP-9]`), so they are deterministic; `sum_f64()` accumulates
  `f32` elements in `f64`, and `math.KahanSum` gives compensated summation.

## XV.6 Numbers and math

* `[STD-20]` *(new in 0.9.9)* Integer methods, built into every integer type: `abs` (a signed
  `MIN` panics), `pow(e)` (`x ** e`), `signum` (`-1`, `0` or `1`), `div_trunc` and `rem_trunc`
  (C's truncating `/` and `%`, panicking where `//` and `%` do); for each arithmetic operator
  `OP` — `add`, `sub`, `mul`, `floordiv` (`//`), `rem` (`%`), `pow` (`**`) and `neg` (unary `-`) —
  `checked_OP -> Option[T]` (`None` wherever the operator would panic: a result out of range, a zero
  divisor, a negative exponent), `wrapping_OP -> T` (the result modulo 2^N), `saturating_OP -> T`
  (the nearest of `MIN` and `MAX`) and `overflowing_OP -> (T, bool)` (the wrapping result and
  whether it wrapped), the last three still panicking on a zero divisor or a negative exponent; for
  the shifts, `checked_shl`/`shr` (`None` for an amount outside `0 ≤ n < width`),
  `wrapping_shl`/`shr` (the amount modulo the width) and `overflowing_shl`/`shr` (whether it was
  outside); `count_ones`, `leading_zeros` and `trailing_zeros`, each an `int` counted over the
  type's width; `is_power_of_two`; `next_power_of_two` (the least power of two at or above the
  value, overflowing past `MAX`); and the constants `T.MIN` and `T.MAX` (`int.MAX` through the
  alias). An operator method's other operand has the receiver's type; a shift amount and an
  exponent may be any integer type (`[TYP-10]`, `[TYP-30]`). Float methods: `abs`, `sqrt`, `floor`,
  `ceil`, `trunc`, `fract`, `round` (**half to even**, as Python's `round` and IEEE's default),
  `round_half_away` (C's `round`), `is_nan`, `is_finite`, `is_infinite`, `copysign`, `mul_add`, and
  the constants `INF`, `NAN`, `EPSILON` (the gap between `1.0` and the next value), `MAX` (the
  greatest finite value) and `MIN` (the least finite value, `-MAX`, as an integer's `MIN` is its
  least) (ODR-039). A float's `Display` is the shortest text that reads back as the same value, with `.0`
  for an integral value, as Python's `repr`: `0.1 + 0.2` prints `0.30000000000000004`, `1.0` prints
  `1.0`, and `1e300` prints `1e+300`; a format spec overrides it.
* `[STD-3]` `math.fma(a, b, c)` (and `mul_add`) is a fused multiply-add with a single rounding, on every
  target; it is how fused arithmetic is written without `@fp(contract)`. `std.math` uses it for matrix
  products and transforms.
* `[STD-4]` `NonZero[T]` for each integer `T` is a `Copy` wrapper with a niche (`Option[NonZero[T]]` is
  `T`-sized), made by `NonZero.new(v) -> Option[NonZero[T]]`. Dividing by a `NonZero` needs no
  division-by-zero check.
* `[STD-21]` *(new in 0.9.9)* `std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, `acos`,
  `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sqrt`, `rsqrt`, `cbrt`,
  `hypot`, `lerp`, `smoothstep`, each taking any number type (`fn f[T: Number](x: T) -> T.Real`,
  `[STD-27]`): an integer's answer is an `f64` (`math.sqrt(9)` is `3.0`), an `f32`'s an `f32` and an
  `f64`'s an `f64`; and the vector types `Vec2`, `Vec3`,
  `Vec4` (of `f32`, `@layout(c)`), `IVec2/3/4`, `Mat3`, `Mat4` (column-major), `Quat`, `Transform`,
  `Aabb`, `Ray`, `Plane`, `Frustum`, with `dot`, `cross`, `length`, `normalize` (panics on a zero
  vector; `normalize_or_zero` does not) and the arithmetic operators. `PI`, `TAU` and `E` are
  untyped constants (V.7): `x: f32 = math.PI` is `PI` rounded to `f32`, and with nothing to fix its
  type a use is `f64`.
* `[STD-27]` *(new in 0.9.9)* `std.math.Number` is every number type. `std.math` adds each with an
  `extend … implements Number` block stating its `type Real: Float`, the decimal type its answers
  come back as (`f64` for every integer type, itself for `f32` and `f64`), and `to_real`, which turns
  it into one. `[STD-21]`'s functions take `T: Number` and answer in `T.Real` (`[IFC-4]`); a program
  may add its own number types the same way (ODR-038). `std.math.Float` is the decimal types:
  `std.math` makes `f32` and `f64` implement it (`extend f32 implements Float`), and implementing it
  for any other type is `E2042`. A bound `T: Float` provides, inside a generic body, `Copy`, `Eq`, `Ord` (`[TYP-37]`) and `Default`;
  the arithmetic operators `+`, `-`, `*`, `/`, `//`, `%`, `**` and unary `-` on two `T`s, with a `T`
  result; an untyped numeric literal adopting `T` wherever a `T` is expected, as it would adopt `f32`
  or `f64`; and as methods the float methods of `[STD-20]` and `[STD-21]`'s `sin` through `hypot`
  (`x.sqrt()`, `y.atan2(x)`, `a.hypot(b)`), which are built into `f32` and `f64`: each free function
  turns its number into its `Real` and calls the method (`math.sqrt(x)` is `x.to_real().sqrt()`);
  `rsqrt`, `lerp` and `smoothstep` are free functions only. A method is the C
  library's function of its name for `f64` and its `f` form for `f32` (`ln` is C's `log`, `round` its
  `nearbyint`, `round_half_away` its `round`, `mul_add` its `fma`); the transcendental ones are
  `[DET-2]`'s platform functions.

## XV.7 I/O, files, time and processes

* `[STD-22]` *(new in 0.9.9)* Every operation that touches the outside world returns `Result[T,
  io.Error]` and carries `Io`: `io.stdin().read_line() -> Result[Option[String], io.Error]` (`None` at
  end of input), `lines()`; `fs.read(path) -> Result[Array[u8], io.Error]`, `fs.read_to_string`,
  `fs.write(path, data)`, `fs.append`, `File.open`, `File.create`, `fs.exists`, `fs.remove`,
  `fs.create_dir_all`, `fs.read_dir`, `fs.metadata`. A `File` closes when dropped; `BufReader` and
  `BufWriter` buffer, and `BufWriter` flushes when dropped.
* `[STD-23]` *(new in 0.9.9)* `time.Instant.now()` is monotonic and `Nondet`; `Duration` is an integer
  count of nanoseconds with checked arithmetic; `time.sleep(d)` has `Block`.
* `[STD-24]` *(new in 0.9.9)* `process.exit(code) -> Never` flushes the standard streams and exits
  without running destructors or `defer` blocks; returning from `main` is the clean exit.
  `process.args()` and `args_os()` follow `[FN-8]`; `env.var(name) -> Option[String]`.
* `[STD-25]` *(new in 0.9.9)* `random.Rng.seeded(seed)` is a deterministic generator (the same seed
  gives the same sequence on every machine); `Rng.from_entropy()` seeds from the system and is `Nondet`.
  `rng.int_in(a..b)`, `rng.float()` (in `[0, 1)`), `rng.choice(span) -> Option[ref T]`,
  `rng.shuffle(mut_span)`.
