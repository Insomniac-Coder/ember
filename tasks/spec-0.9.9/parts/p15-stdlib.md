---

# Part XV — The Standard Library

The standard library is one package, `std`. This Part fixes what every implementation provides and
what each operation means; `std/**/*.em` holds the exact signatures and becomes normative for
anything this Part leaves open.

## XV.1 Organisation and conventions

* `[STD-6]` *(changed in 0.9.15)* `std` is layered, and each layer depends only on the ones before it: **core** (scalars,
  `Option`, `Result`, views, fixed-capacity containers, math, SIMD), **alloc** (`Array`, `String`,
  `Map`, `Set`, and the heap blocks of `owns` and `shares` values), **sync** (atomics, locks, channels, threads, jobs), **io** (console,
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
* `[STD-26]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Python's built-in functions.** The prelude has these functions, with
  Python's meaning; each reads its argument as a `for` loop does (`[CTL-1]`):
  * `len(x)` is `x.len()`: an `int` for a collection or view, and for a range its count
    (`[STD-19]`), in the smallest type that holds it by the numbers the compiler knows
    (`len(0..n)` with `n: int` is an `int`). `len` of a `StrView` or `String` is
    `E2073`: Python counts characters and Ember's `s.len()` counts bytes, so the help offers both
    `s.char_count()` and `s.len()`.
  * `range(stop)`, `range(start, stop)`, `range(start, stop, step)` count as Python's do, including a
    negative `step`; `step == 0` panics. `range(n)` is `0..n`, and every form compiles to a counted
    loop (`[CTL-3b]`).
  * `enumerate(it, start=0)`, `zip(a, b)` (stops at the shorter) and `reversed(x)` (for what can run
    backwards) return lazy iterators, as Python 3's do.
  * `sum(it, start=0)`, `any(it)` and `all(it)` consume an iterable (of numbers, of `bool`s);
    `any(x > 0 for x in xs)` uses a generator expression (`[GRM-38]`).
  * `sorted(it, key=None, reverse=false) -> Array[T]` returns a new sorted array (stable) of the items
    the iterator gives, and copies none. Over an iterator of views (`sorted(m.keys())`) it is a list of
    views, a value that holds views, which lives no longer than what they point into (`[TYP-15]`): a
    change that may reshape `m` ends it (`E3060` at a later use). A list of items of your own is
    written `sorted(m.keys().copy_each())` (`[STD-19]`).
  `min` and `max` stay two-argument functions; `xs.iter().min()` gives an `Option` for a collection.
* `[STD-14]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Failure follows `[ERR-13]`: a caller's bug
  panics; a failure of the world returns `Result` or `Option`. The non-panicking twin `[ERR-13]` asks
  for is an accessor's (`Mat4.col`/`col_checked`, `CppShared.get`); a constructor's or a mutation's
  caller's bug (`stepped(0)`, `BitSet.insert` of a negative, `Entity.new` out of range, `Registry.emplace` on
  a dead entity, `normalize` of zero, `split` on an empty separator) panics with none. Every
  allocating function is documented as allocating and carries `Alloc`.

| Module | Contents |
|---|---|
| prelude | the names of `[MOD-5]` |
| `std.core` | scalars, `Option`, `Result`, `AnyError`, `Ordering`, ranges, `NonZero`, the standard interfaces (§IV.8), iterator adapters, `type_name[T]()` (a type's fully qualified name, which the compiler knows) |
| `std.mem` | `size_of`, `align_of`, `offset_of`, `swap`, `replace`, `take`, `drop`, `keep_alive`, `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Volatile` (`[STD-29]`), `Layout`, `Allocator`, `Global`, `ptr`, `check_disjoint`, `assert_disjoint`, `assume_disjoint` (Part IX) |
| `std.cell` | `Cell`, `UnsafeCell` |
| `std.collections` | `Array`, `Map`, `Set`, `Deque`, `SmallArray`, `BitSet`, `FixedArray`, `FixedString`, `RingBuffer`, `Pool`, `Handle`, `SoA`, `ArenaArray`, `ArenaMap`, `ArenaSoA`, `DefaultHasher`, `RandomState`, `Hasher`, `CapacityError`; the iterator types `SpanIter`, `MutSpanIter`, `SpanChunks`, `MutSpanChunks`, `ArenaArrayIter`, `ArenaArrayIterMut`, `ArenaMapIter` (public, not in the prelude) |
| `std.string` | `String`, `StrView` methods, `StringBuilder`, `ParseError`, `Utf8Error` |
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
| `std.ffi` | `CString`, `CStrView`, `c_int` and the other C aliases, `c_wchar`, `WideString`, `Retained`, `Callback`, `adopt` (which gives an `owns T`) (Part XVI) |
| `std.gpu` | Annex D |

* `[STD-29]` *(new in 0.9.15)* `std.mem.Volatile[*T]`'s `read_volatile` and `write_volatile` are
  `unsafe fn`s (`[UNS-5]`) whose `@safety` asks the caller for a pointer that is valid for a read (a
  write) of a `T` **and aligned to `T`'s alignment**: a volatile access through a misaligned pointer is
  undefined in the C it compiles to, so the caller's promise names it.

## XV.2 Console output, input and formatting

* `[STD-9]` *(new in 0.9.9)* *(changed in 0.9.15)* `print(a, b, …, sep=" ", end="")` and `println(a, b, …, sep=" ",
  end="\n")` write their arguments' text to standard output separated by `sep`;
  `eprint`/`eprintln` write to standard error. They are compiler-known and are the only calls with a
  variable number of positional arguments (`[TYP-26]`). An argument's text is its `Display` text when
  its type has `Display`, and its `Debug` text otherwise, which is Python's `str` → `repr` fallback; a
  type with neither is `E2040`, whose help is to implement `Display`. A class object always has
  `Debug`, its class and address
  (`[TYP-36]`), so a class without a written `Display` prints by it (ODR-113). `println()` prints an
  empty line.
* `[STD-2]` Printing allocates only to build an f-string argument; `println("literal")`,
  `println(some_str)` and `println(n)` for a number are `@noalloc`.
* `[STD-10]` *(new in 0.9.9)* `input(prompt="") -> String` prints the prompt, flushes, reads one line
  and returns it without the line ending. The prelude's console functions treat a console failure as
  fatal, as Python does when the exception is not caught: end of input or a closed stream panics, and
  the message names the `Result` forms in `std.io` (`stdin().read_line()`).
* `[STD-18]` *(new in 0.9.9)* *(changed in 0.9.15)* `format(value, spec="") -> String` formats one value with a format spec
  (`[LEX-19]`), like Python's `format`; the spec is a string literal, checked against the value's type
  where it is written, as an f-string's is (`E2250`, ODR-168).
  `format_to(ref mut buf: MutSpan[u8], …) -> Result[StrView, FmtError] from buf` formats into a
  caller's buffer, called `format_to(ref mut buf, …)`, and gives a view of the text it wrote there; its
  own machinery never allocates, so the call allocates only if a value's
  `Display` does (`[EFF-1]`). An f-string lowers to a `StringBuilder` sized from its
  literal parts.

## XV.3 Text

* `[TXT-1]` *(changed in 0.9.15)* `StrView` is a read-only view of bytes, a `Span[u8]`, known to be valid UTF-8; `String`
  owns a heap buffer with the same guarantee. Safe code may rely on the invariant (`[UNS-4]`).
* `[TXT-2]` *(changed in 0.9.15)* Nothing becomes a `StrView` without validation: every conversion from bytes or foreign text
  returns `Result[StrView, Utf8Error]`, or a `_lossy` form that substitutes U+FFFD. An importer or
  binding that would hand foreign bytes to a `StrView` without validation is `E5063`.
* `[TXT-3]` *(changed in 0.9.15)* `StrView` and `String` are a pointer and a length, not null-terminated, and may contain
  `\0`. A C boundary uses `CStrView`/`CString` (Part XVI); a `c"…"` literal containing a NUL is `E5064`.
* `[TXT-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* Slicing a string, `s[a..b]`, is by byte offset and panics in every
  profile when a bound is not on a character boundary; `s.get(a..b) -> Option[StrView]` is the total
  form. There is no indexing by character position, because it would be O(n).
* `[TXT-5]` *(changed in 0.9.15)* `StrView` → `String` allocates and copies, and is written `String(s)`; `String` →
  `StrView` is free; `Span[u8]` → `StrView` validates in O(n) without allocating; `StrView` →
  `CString` is the fallible allocating `.to_cstring()` of `[FFI-15]` (ODR-081).
* `[TXT-6]` *(changed in 0.9.15)* Across the C ABI a `StrView` is `{const uint8_t *ptr; size_t len}`.
* `[TXT-7]` Text is UTF-8 everywhere. `std.ffi.WideString` converts, explicitly, for APIs that need
  UTF-16.
* `[TXT-8]` *(changed in 0.9.15)* `StrView` is a view, like every `Span`: it lives no longer than the text it points into,
  and a change that may reshape that text (replacing a `String`, `push_str`, `clear`) ends it.
  A `String` place reads as a `StrView` wherever one is expected.
* `[TXT-9]` *(new in 0.9.9)* *(changed in 0.9.15)* **A string literal initialises a `String`.** Wherever a `String` is
  expected — an initialiser, an argument, a field, a collection element, a return value — a string
  literal produces a new `String` holding its text. The allocation happens at the literal and is
  reported there (`Alloc`); where a `StrView` is expected, a literal is a view of text written in the
  program and allocates nothing. A `StrView` *value* never converts implicitly: a `String` of your own
  is written `String(s)` (`s.to_string()` and `String.from(s)` mean the same).
* `[TXT-10]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **`StrView` operations.** `len()` is the length in bytes and
  `char_count()` in characters; `is_empty`, `chars`, `char_indices`, `bytes`, `as_bytes`, `lines`,
  `split(sep)`, `split_whitespace()` (Python's `split()` with no argument), `split_once(sep) ->
  Option[(StrView, StrView)]`, `partition(sep) -> (StrView, StrView, StrView)` (Python's), `trim`,
  `trim_start`, `trim_end`,
  `starts_with`, `ends_with`, `find(sub) -> Option[int]`, `rfind`, `count(sub)`, `replace(old, new)
  -> String`, `to_upper`, `to_lower`, `repeat(n)`, `parse[T]() -> Result[T, ParseError]`,
  `to_string`, `get(range)`, `is_char_boundary(i)`, and `x in s` (`[STD-8b]`). A method that gives
  text it did not make (`trim`, `split`, `lines`) gives views pointing into `s`; one returning several
  strings returns an iterator of such views. Case mapping and `char_count` are
  Unicode-aware; nothing depends on the locale. `char` has `is_alphabetic` (Unicode's Lu, Ll, Lt, Lm
  and Lo), `is_digit` (the digits of every script, and the superscripts), `is_alphanumeric`
  (alphabetic or numeric) and `is_whitespace` (what `split_whitespace` splits at), which mean what
  Python's `isalpha`, `isdigit`, `isalnum` and `isspace` do, and `is_ascii_alphabetic`,
  `is_ascii_digit`, `is_ascii_alphanumeric` and `is_ascii_whitespace`, the same restricted to ASCII.
  `parse[T]()` (ODR-029) reads the whole text as an
  integer type (the 256-bit counts included, over their whole range), a float type, `bool` or `char`,
  skipping no white space (`s.trim().parse[int]()`): an
  integer is an optional sign (`-` only for a signed type) and ASCII decimal digits, and must fit `T`;
  a float is an optional sign and `inf`, `infinity` or `nan` in any case, or decimal digits with an
  optional `.`, fraction and exponent (`e` or `E`, an optional sign, digits), rounded to the nearest
  `T`; a `bool` is `true` or `false`; a `char` is exactly one character. `ParseError`, in `std.string`,
  is a unit-only enum, `Empty`, `Invalid` or `Overflow`, naming the first problem from the left.
* `[TXT-11]` *(new in 0.9.9)* *(changed in 0.9.15)* **`String` operations.** Everything `StrView` has (by read-through),
  plus `String()`, `String(s)` (a `String` of your own made from text), `with_capacity(n)`,
  `push(c: char)`, `push_str(s)`, `insert`, `remove`, `truncate`, `clear`, `capacity`, `reserve`, and
  `as_str` (a `StrView` of the whole text). `a + b` for `a: String, b: StrView` takes `a` over and
  returns it extended; `s += t` appends. `StrView + StrView` is `E2040` whose help is an f-string.
  ODR-098 defines the mutation and capacity methods: `String.with_capacity(n) -> String` makes an
  empty string with room for at least `n` bytes; zero allocates nothing. `capacity() -> int` reads
  the allocated byte capacity. `reserve(additional)` makes room for at least `len() + additional`
  bytes, without changing the text, and does nothing when that capacity already exists.
  `insert(i, c: char)` inserts one character at byte offset `i`, which must be in `0..=len()` and
  on a character boundary. `remove(i) -> char` removes and returns the character starting at byte
  offset `i`, which must be in `0..len()` and on a character boundary. Both close up or move the
  suffix as necessary; neither scans the prefix for a character position. `truncate(n)` retains
  the first `n` bytes when `n < len()` (then `n` must be a character boundary), and otherwise does
  nothing. `clear()` removes all text. `remove`, `truncate` and `clear` keep the capacity.
  These change the string, through a name that may change it, and return `void` unless a result is
  named above; each is a reshape, which ends every view into the text. Counts and
  offsets accept any integer type; a negative literal is `E2011`, and a negative run-time value or
  one not representable as a native length panics before conversion. An invalid offset or character
  boundary panics in every profile. Growth uses the checked allocation rules of `[HEAP-8]`.
  The arguments are evaluated before the string is changed, as for `Array`'s methods that change it.

## XV.4 Collections

* `[STD-15]` *(new in 0.9.9)* *(changed in 0.9.15)* **`Array[T]`** is a growable contiguous list (Python's `list`).
  `len`, `is_empty`, `capacity`, `reserve`, `push`, `pop -> Option[T]`, `insert(i, v)`, `remove(i) -> T`,
  `swap_remove(i) -> T`, `extend(iterable)`, `truncate`, `clear`, `retain(keep)`,
  `drain(r) -> Array[T]`, `dedup`, `reverse`, `sort` (stable; `T: Ord`; an inconsistent order
  permutes but never corrupts, `[HASH-3]`), `sort_by_key(f)`, `sort_by(cmp)`, `sorted() -> Array[T]`,
  `binary_search(x) -> Result[int, int]`, `index_of(x) -> Option[int]`, `first`, `last`,
  `get(i) -> Option[ref T]`, `get_mut`, `get_pair_mut(i, j) -> Option[(ref mut T, ref mut T)]`
  (`[BRW-5]`), `iter`, `iter_mut`, `split_at`, `chunks(n)`, `chunks_mut(n)`, `windows(n)`, `join(sep)`
  (for elements that are `Display`), and `x in xs` (`[STD-8]`). `xs[i]` takes any integer and panics
  when `i < 0` or `i >= len` (`[TYP-31]`); `xs.get(i)` names an element as `xs[i]` does, as a
  position. The methods given a function call it right away and take it as a type parameter:
  `[F: fn(T) -> bool]` above `fn retain(owns keep: F):`, `[K: Ord, F: fn(T) -> K]` above
  `fn sort_by_key(owns f: F):`, and `[F: fn(T, T) -> Ordering]` above `fn sort_by(owns cmp: F):`; the
  call counts as making what the function given changes (`[STD-19]`), so `LOG.retain(keep)` with a
  `keep` that pushes to `LOG` is `E3022`. Each changes the list it is called on, and every method that
  adds, removes, reorders or replaces elements is a reshape: it ends every view into the list, and may
  not run while a loop walks the list (`E3020`). `sort_by` and `sort_by_key` are stable, and
  `sort_by_key` calls `f` once for each element, in order. `windows(n)` yields every run of `n`
  neighbours, one step apart, as read-only views (there is no `windows_mut`); none when `n > len`, and
  `n == 0` panics as `chunks(0)` does (`[SPN-4]`). `drain(r)` hands the elements in `r`, any range of
  integers (`a..b`, `a..=b`, `a..`, `..b`), over to a new `Array` in order, and the rest close up; a
  range reaching outside `0..=len`, or starting after it ends, panics.
* `[STD-11]` *(new in 0.9.9)* *(changed in 0.9.15)* **`Map[K, V, H = DefaultHasher]`** is a hash map that **iterates in
  insertion order**, like Python's `dict`. Re-assigning an existing key keeps its position; removing
  a key keeps the order of the others. Lookup, insertion and removal take expected constant time,
  amortised: a map may grow, or close up removed entries, inside an insertion or a removal;
  iteration is linear in the number of entries. `H: Hasher + Default`, and a map makes a fresh `H`
  for each hash. Its memory comes from the built-in memory manager, or from one of kind `A` written
  `Map[K, V] using A`, whose instance the line that makes the map names (`[ALC-1]`).
  `Set[T, H = DefaultHasher]` (and `Set[T] using A`) is the same with no values. Keys, values and
  elements may be views (`labels: Map[StrView, int] = {"start": 1}`): a map that holds views is a value
  that holds views, which lives no longer than what they point into (`[TYP-15]`), and keeping or
  returning it where it would outlive them is `E3063` (ODR-069). A map of views of text written in the
  program has no such bound. Because order is fixed by the program's operations and the default hasher is
  fixed-seed (`[HASH-2]`), iterating a `Map` or `Set` is not `Nondet`.
* `[STD-16]` *(new in 0.9.9)* *(changed in 0.9.15)* **`Map` operations.** `m[k]` reads the value and panics when the key is
  missing (`key not found: <Debug of k>; use .get(k) for an Option`); `m[k] = v` inserts or replaces
  (`[STD-17]`); `m[k] += 1` requires the key to exist and panics with the same message. The
  lookups take any `q: Q` with `Q: AsKey[K]` (`[STD-12]`): `get(q) -> Option[ref V]`,
  `get_mut(q) -> Option[ref mut V]`, `get_or(q, default) -> V` (for `V: Copy`),
  `contains_key(q) -> bool`, `remove(q) -> Option[V]`, `m[q]` and `q in m`. `insert(k, v) ->
  Option[V]` and `entry(k)` take the key itself; `entry(k)` is a `MapEntry` with `or_insert(v)`,
  `or_insert_with(f)` (`[F: fn() -> V]` above `fn or_insert_with(owns f: F) owns this -> ref mut V:`)
  and `or_default()` (for `V: Default`), each returning `ref mut V`, a place
  (`counts.entry(k).or_default() += 1`).
  `keys()`, `values()`, `values_mut()` and `items()` are the view iterators
  `MapKeys`, `MapValues`, `MapValuesMut` and `MapItems`, yielding `ref K`, `ref V`, `ref mut V` and
  `(ref K, ref V)` in insertion order (`for k, v in m.items():`); `for k in m:` is by key, as
  Python's `dict`, and `for k, ref mut v in m:` names each value as a place the loop may change.
  While a loop walks a map (its keys, its values or its items), nothing changes the map, through its
  own name or through a call (`counts[k] *= 2` and `counts.insert(...)` inside it are `E3020`); the
  loop's own name changes the value it is on (`for k, ref mut v in counts: v *= 2`).
  Also `Map[K, V]()`, `with_capacity(n)`, `len`, `is_empty`, `capacity`,
  `reserve(n)`, `try_reserve(n)`, `clear`, `retain(keep)` (`[F: fn(K, V) -> bool]` above
  `fn retain(owns keep: F):`), `pop_item() -> Option[(K, V)]` (the newest entry, as `dict.popitem`),
  `update(other)` (inserts `other`'s entries in its order) and `into_items() -> Array[(K, V)]` (takes
  the map over, `owns this`, and gives its entries in order).
  `m1 == m2` compares as sets of entries: the same length, and every key of one in the other with an
  equal value; order does not matter. `sorted(m)` is `sorted(m.keys())`, a sorted list of views of the
  keys (`[STD-26]`).
  `Set`: `Set[T]()`, `with_capacity(n)`, `len`, `is_empty`, `capacity`, `reserve(n)`,
  `try_reserve(n)`, `clear`; `add(x) -> bool` (whether `x` was new; an element already there keeps
  its position); taking `q: AsKey[T]`, `remove(q) -> bool`, `contains(q) -> bool` and `q in s`;
  `retain(keep)` (`[F: fn(T) -> bool]` above `fn retain(owns keep: F):`), `pop() -> Option[T]` (the
  newest), `iter()` (a `SetIter` yielding `ref T`); `union`, `intersection`, `difference` and
  `symmetric_difference` of another `Set`, which they read, return a new `Set` of copies (for
  `T: Clone`), this set's elements first in its order and then the
  other's; `is_subset`, `is_superset`, `is_disjoint`; and the operators `|`, `&`, `-` and `^`.
  `s1 == s2` compares as sets.
* `[STD-12]` *(new in 0.9.9)* *(changed in 0.9.15)* **Keys looked up by a view.** Lookup methods, `m[k]` and `k in m` accept
  any key type `Q` with `Q: AsKey[K]`: `K` itself, and `StrView` for `K = String`.
  `interface AsKey[K]: Hash` (in `std.collections`, not the prelude) has the method
  `fn is_key(key: K) -> bool`; `interface ToKey[K]: AsKey[K]` adds `fn to_key() -> K`, which makes the
  key (ODR-067). A lookup hashes `q` and compares it with `is_key`, so it never converts or copies a
  key. `m[q] = v` needs `Q: ToKey[K]` and calls `to_key()` only when the key is new, never to replace
  the value of an existing one, so a new key written through a `StrView` is stored as the map's own
  `String`. Every `K: Eq + Hash` is `AsKey[K]` (`is_key` is `==`), and `ToKey[K]` when it is also
  `Clone` (`to_key` gives `copy of` the key): a key type that cannot be copied is looked up and removed
  by value and inserted with `insert`, which takes the key over. std implements `AsKey[String]` and
  `ToKey[String]` for `StrView`. An
  implementation MUST hash and compare as the key it matches would, and `to_key()` MUST make that
  key; one that does not gives `[HASH-3]`'s wrong answers, never undefined behaviour.
* `[STD-17]` *(new in 0.9.9)* *(changed in 0.9.15)* **Index assignment.** `a[i] = v` calls `IndexSet.index_set(i, v)` when
  the type implements `IndexSet[Idx, V]`, and otherwise assigns through `IndexMut` (ending the old
  value). `Map` implements `IndexSet`; `Array` does not need to. A compound assignment `a[i] op= v`
  always goes through `IndexMut`. `Array` and `MutSpan` implement `Index[int]` and `IndexMut[int]`,
  and `Span` `Index[int]`, through their built-in indexing; `Map[K, V]` implements `Index[Q]` and
  `IndexMut[Q]` for every `Q: AsKey[K]`, and `IndexSet[Q, V]` for every `Q: ToKey[K]` (blanket
  implementations, `[GRM-34]`, ODR-042, ODR-067).
* `[STD-7]` Fixed-capacity containers with no heap allocation: `FixedArray[T, N]`, `FixedString[N]`
  and `RingBuffer[T, N]`; `push` returns `Result[void, CapacityError]` and `push_or_drop` discards when
  full. Also `Deque[T]`, `SmallArray[T, N]` (inline up to `N`, then heap), `BitSet`.
* `[STD-8]` *(changed in 0.9.15)* **`Contains`.** `x in c` requires `c: Contains[typeof(x)]` (`E2226` otherwise, whose help
  is `c.iter().any(fn(e) => e == x)`). `std` implements it for `Set` and `Map` (by key); for `Array`,
  `Span`, `MutSpan` and `[T; N]` with `T: Eq`, as a linear scan; for `StrView`/`String`, as a substring or
  character test; for ranges, as two comparisons. The compiler never synthesises an implementation.
* `[STD-8a]` `ember inspect --cost` reports which `contains` a use selects and its complexity, so
  `x in array` is visibly linear and `x in set` visibly constant.
* `[STD-8b]` *(changed in 0.9.15)* `StrView` implements `Contains[char]` and `Contains[StrView]` only; matching happens at character
  boundaries, so a character never matches inside another's encoding. A byte needle (`Span[u8] in s`)
  is `E2226`; byte search is on `s.as_bytes()`. `x not in c` is `not c.contains(x)`, with each operand
  evaluated once.

## XV.5 Iterators

* `[STD-19]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* Every `Iterator` has the adapters `map`, `filter`, `filter_map`,
  `enumerate`, `zip`, `chain`, `take`, `skip`, `take_while`, `skip_while`, `step_by`, `flat_map`,
  `flatten`, `peekable`, `copy_each`, `inspect`, and `rev` where the iterator can run backwards;
  and the consumers `count`, `sum`, `product`, `min`, `max`, `min_by_key`, `max_by_key`, `fold`,
  `reduce`, `any`, `all`, `find`, `position`, `last`, `nth`, `min_by`, `max_by`, `for_each`,
  `collect[C]()`, `to_array()`,
  and `join(sep)` for `Display` items. `copy_each()` gives, for each item, what `copy of` the item
  would make (for `Item: Clone`): `names = sorted(m.keys().copy_each())` is a list of strings of its
  own, kept past changes to `m`; `.copied()` and `.cloned()` are not Ember (`E1010`, "write
  `.copy_each()`"). Adapters are lazy and allocate nothing themselves; what the functions they apply
  do counts where they run (`[EFF-1]`). A chain of adapters in a
  `for` loop compiles to one loop with no iterator object in memory (`[CTL-3]`). The adapters and
  consumers can also be called directly on an `Iterable` value, reading it as `iter()` would:
  `xs.enumerate()` is `xs.iter().enumerate()`, and a method of the collection's own of the same name
  (`Array.join`) takes precedence. An adapter takes the iterator it wraps over (`owns this`), and so
  does a consumer; `zip` and `chain` take their other iterator over too, so a named iterator given to
  `zip` is handed over (`copy of it` keeps the caller's). **A function is taken as a type
  parameter**, by the adapters that apply one and the consumers alike (`any`, `all`, `find`,
  `position`, `fold`, `reduce`, `for_each`, the `…_by` and `…_by_key` forms): `[F: fn(Item)]` above
  `fn for_each(owns f: F) owns this:`, `[F: fn(Item) -> bool]` above `fn any(owns f: F) owns this ->
  bool:`, `[F: fn(Item)]` above `fn inspect(owns f: F) owns this -> Inspect[Self, F]:`. `F` names the
  exact function given, so the adapter or consumer is compiled once for each function given to it and
  the call through `f` is direct and may be inlined; the shared type, `owns f: fn(Item)`, is legal too,
  and is what keeps functions of different kinds in one list (`handlers: Array[fn()]`). The function
  is handed over as every argument for an `owns` parameter is, so an adapter made from a lambda can be
  kept or returned
  (`return xs.iter().map(fn(x) => x * 2)`), what the lambda holds with it. A function given may change
  things: a consumer's call counts as making what the function changes, now
  (`scores.for_each(fn(s): LOG.push(copy of s.name))` changes `LOG`, so inside a loop over `LOG` it is
  `E3020`), and an adapter keeps the function and runs it at each step, so whatever drives the adapter
  (`for x in it:`, `it.sum()`) makes its changes; driving an `owns dyn Iterator[Item = T]` makes the
  changes of every adapter the program can put there. What a caller
  passes is an `int` (ODR-089): `take(n)` gives at most `n` items and
  `skip(n)` leaves out the first `n`, `nth(n)` is the item at index `n` (`None` past the end),
  `step_by(k)` gives the first item and then every `k`-th, and `enumerate(start=0)` numbers from
  `start` as `[STD-26]`'s `enumerate` does. A negative count or index, or a `k` of zero or less,
  is the caller's bug and panics when the adapter is made or the consumer called (`[STD-14]`),
  never partway through. **Counts and numbers** (the owner's design G8-4, 0.9.10): what Ember
  hands back is in the smallest type that holds every value the iterator can give. `Iterator` has
  the associated types `Count`, what `len()`, `count()` and `position()` give, and `Position`, what
  `enumerate` numbers its items with; each is `int` unless the iterator says otherwise, and each
  implements `std.core.ItemCount` (`int`, `u64`, `u128`, `u256`, `i128` and `i256` do), which
  gives the count operations (`of`, `plus`, `less`, `exceeds`, `to_int`, and `as_count[C]()` from
  one count type to another that holds it). Stored items (arrays, views, text, maps, sets) and
  ranges of 8-, 16- or 32-bit numbers count and number in `int`; a `..` range of 64-bit numbers
  counts in `u64` and a `..=` one in `u128`, a `..` range of 128-bit numbers in `u128` and a `..=`
  one in `u256` (`[TYP-42]`); ranges of 64-bit numbers number in `i128` and of 128-bit numbers in
  `i256`, from a start that may be negative. An adapter that never gives more items than it takes
  counts and numbers as what it wraps. `chain` counts its two sides together, stepping up only as
  far as needed: two `int` counts stay an `int` (stored items and small ranges cannot come near its
  top), otherwise the count is one level above the wider side's (`u64`, then `u128`, then `u256`),
  and it numbers in the type that holds both sides' numbers; `flatten` and `flat_map` count in the
  level that holds the outer count times the inner one (an `int` by an `int` stays an `int`).
  `u256` is the limit: a count past it panics. In generic code these are the counts of the types
  involved, filled in for each use. **The known-numbers rule** (the owner's rulings of 2026-10-06,
  which widen the visible-numbers rule): where `enumerate` is called, and where `len` or `count` is
  called on an iterator, the numbers it gives, and the count, are in the smallest of `int`, `u64`,
  `i128`, `u128` and `i256` that holds every value it can give, read from the numbers the compiler
  knows. It knows a number written as one; a name set once and never changed anywhere in its
  function (no assignment, augmented assignment, `ref mut` name or argument of it, or call of a method
  that changes it, in the function or a lambda in it) holds what its value held, and a range or an `Array`
  literal so set has its bounds or its length; a loop's counter (`for i in a..b`, `range(a, b,
  step)`, and a range's value through `rev`, `skip`, `take` and `step_by`) holds its range's
  numbers; `+`, `-`, `*`, `//` and `%` give what their known operands allow; a list's, a view's or
  a text's length is at most what its items' bytes allow (`[HEAP-8]`: at most `PTRDIFF_MAX` bytes,
  an item of no size counting as one) and a fixed array's is its `N`; `take`, `skip`, `step_by`,
  `zip` and `chain` give what their known arguments allow. Anything else may be any value of its
  type, and an iterator gives at most what its `Count` holds. `enumerate` numbers from `start` to
  `start + n - 1` over at most `n` items, so its type steps up from the iterator's own `Position`
  where the numbers could pass that type's top (`ys.enumerate(start=int.MAX - 1)` over three items
  numbers in `u64`): no number of an `enumerate` overflows. In generic code an `enumerate` numbers
  in its iterator's `Position`; where what it numbers in depends on the item type (a list of `T`,
  whose most items depend on `T`'s size), the function's own check leaves it, and every line that
  uses those numbers, to the check of each use (`[TYP-17]`), which numbers by what it knows. A use
  that a line does not fit is refused there, naming the use (`in labels used with T = u8`); with
  `[TYP-5]` rule 12, `last = i` takes the numbers of a list of `int`s from 2 whether `last` is an
  `int` or a `u64`. An iterator can run backwards when it implements
  `DoubleEndedIterator`, whose method `fn next_back() -> Option[Item]` gives the last item not yet
  given from either end (with `next`, the two ends meet and no item comes twice) and whose method
  `rev` is, so `rev` is exactly where an iterator can run backwards; `ExactSizeIterator`'s
  `fn len() -> Count` is how many items are left (ODR-091). `skip_front(n)` and `skip_back(n)`
  leave out up to `n` items (a `Count`) from either end and give how many of the `n` they could
  not, and `nth_back(n)` leaves out `n` from the back and gives the one before them; a range does
  each at once, and `rev`, `copy_each`, `take`, `skip`, `zip`, `chain`, and `enumerate` from the back
  pass them down, so none of them passes a gigantic run of items one at a time. `ItemCount`,
  `DoubleEndedIterator` and `ExactSizeIterator` are in `std.core` and are not prelude names
  (`[MOD-5]`). The iterators of the ranges with an end and the element iterators of the views run
  backwards and know their length; `copy_each` and `chain` run backwards when what they wrap
  does, and `take`, `skip`, `step_by`, `enumerate` and `zip` when it also knows its length, giving
  the items `next` would, last first; each knows its length when what it wraps does. Run backwards,
  `enumerate` numbers its last item `start + len - 1` first, and the items `skip_back` leaves out
  need no numbers. `collect[C]()` builds any `C` that implements `FromIterator[Item]` (ODR-094), an interface of
  `std.core` that is not a prelude name: `interface FromIterator[T]:` declares
  `[I: Iterator[Item = T]]` above `static fn from_iter(owns it: I) -> Self`. `Array[T]`, `Set[T]`, `Map[K, V]` (from `(K, V)` pairs, a later pair's
  value replacing an earlier one's for the same key) and `String` (from `char`s and from `String`s)
  implement it, and so may a program's own types. `C` is written (`it.collect[Array[int]]()`) or is
  the type the call is expected to give (`a: Array[int] = it.collect()`). `flatten` is for items that
  are themselves iterable (`IntoIterator`). `peekable()` gives the same items with
  `fn peek() -> Option[ref Item]` and `fn peek_mut() -> Option[ref mut Item]`, which take the next item early
  and keep it where it is until `next` gives it; `next_if(pred)` gives the next item when `pred` holds
  for it and leaves it otherwise, and `next_if_eq(v)` when it equals `v` (`Item: Eq`).
* `[STD-5]` *(changed in 0.9.9)* `sum` and `product` combine elements left to right in the element
  type; an empty sum is zero. Integer sums panic on overflow (`[TYP-8]`). Float sums are never
  reassociated outside `@fastmath` (`[TYP-9]`), so they are deterministic; `sum_f64()` accumulates
  `f32` elements in `f64`, and `math.KahanSum` gives compensated summation: an `f64` accumulator with
  Neumaier's correction, `KahanSum.new()`, `add(x)` and `value()`, and `KahanSum.of(xs)` for a
  `Span[f64]` (ODR-043).

## XV.6 Numbers and math

* `[STD-20]` *(new in 0.9.9)* Integer methods, built into every integer type: `abs` (a signed
  `MIN` panics), `pow(e)` (`x ** e` under the default panic policy), `signum` (`-1`, `0` or `1`), `div_trunc` and `rem_trunc`
  (C's truncating quotient and remainder: both panic on a zero divisor, and `div_trunc` also
  panics on signed `MIN / -1`, while `MIN.rem_trunc(-1)` is zero); for each arithmetic operator
  `OP` — `add`, `sub`, `mul`, `floordiv` (`//`), `rem` (`%`), `pow` (`**`) and `neg` (unary `-`) —
  `checked_OP -> Option[T]` (`None` for a mathematical result out of range, a zero
  divisor, a negative exponent), `wrapping_OP -> T` (the result modulo 2^N), `saturating_OP -> T`
  (the nearest of `MIN` and `MAX`) and `overflowing_OP -> (T, bool)` (the wrapping result and
  whether it wrapped), the last three still panicking on a zero divisor or a negative exponent; for
  the shifts, `checked_shl`/`shr` (`None` for an amount outside `0 ≤ n < width`),
  `wrapping_shl`/`shr` (the amount modulo the width) and `overflowing_shl`/`shr` (whether it was
  outside); `count_ones`, `leading_zeros` and `trailing_zeros`, each an `int` counted over the
  type's width; `is_power_of_two`; `next_power_of_two` (the least power of two at or above the
  value, overflowing past `MAX`); and the constants `T.MIN` and `T.MAX` (`int.MAX` through the
  alias). An operator method's other operand has the receiver's type; a shift amount and an
  exponent may be any integer type (`[TYP-10]`, `[TYP-30]`). All these integer methods have fixed
  contracts independent of the caller's lexical overflow policy (ODR-084): in particular, `abs`
  of signed `MIN`, an overflowing `pow` or `next_power_of_two`, and `MIN.div_trunc(-1)` panic;
  named `checked_`, `wrapping_`, `saturating_` and `overflowing_` families retain their stated
  behavior. Float methods: `abs`, `sqrt`, `floor`,
  `ceil`, `trunc`, `fract`, `round` (**half to even**, as Python's `round` and IEEE's default),
  `round_half_away` (C's `round`), `is_nan`, `is_finite`, `is_infinite`, `copysign`, `mul_add`, and
  the constants `INF`, `NAN`, `EPSILON` (the gap between `1.0` and the next value), `MAX` (the
  greatest finite value) and `MIN` (the least finite value, `-MAX`, as an integer's `MIN` is its
  least) (ODR-039). Every float type has the constants, `f16` included (`f16.MAX` is 65504 and
  `f16.EPSILON` 2^-10), though only `f32` and `f64` have the methods (ODR-041). A float's `Display` is the shortest text that reads back as the same value, with `.0`
  for an integral value, as Python's `repr`: `0.1 + 0.2` prints `0.30000000000000004`, `1.0` prints
  `1.0`, and `1e300` prints `1e+300`; a format spec overrides it.
* `[STD-3]` *(changed in 0.9.15)* `math.fma(a, b, c)` (and `mul_add`) is a fused multiply-add with a single rounding, on every
  target; it is how fused arithmetic is written without `@fp(allow_fma)`. `std.math` uses it for matrix
  products and transforms.
* `[STD-4]` `NonZero[T]` for each integer `T` is a `Copy` wrapper with a niche (`Option[NonZero[T]]` is
  `T`-sized), made by `NonZero.new(v) -> Option[NonZero[T]]`. Dividing by a `NonZero` needs no
  division-by-zero check. It is `std.core`'s, imported by name. `get()` returns the integer; the field
  and the memberwise constructor are private, so no `NonZero` holds 0. `T` is one of the twelve
  integer types, and any other type is `E2040`. `x // d` and `x % d` take `d: NonZero[T]` for `x: T`
  and answer a `T`; an overflow still panics (`[TYP-8]`) (ODR-047).
* `[STD-21]` *(new in 0.9.9)* *(changed in 0.9.15)* `std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, `acos`,
  `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sqrt`, `rsqrt`, `cbrt`,
  `hypot`, `lerp`, `smoothstep`, each taking any number type (`[T: Number]` above
  `fn f(x: T) -> T.Real`, `[STD-27]`): an integer's answer is an `f64` (`math.sqrt(9)` is `3.0`), an `f16`'s and an `f32`'s
  an `f32`, and an `f64`'s an `f64`; and the vector, matrix, rotation and shape types of `[STD-28]`,
  with `dot`, `cross`, `length`, `normalize` (panics on a zero vector; `normalize_or_zero` does not)
  and the arithmetic operators. `PI`, `TAU` and `E` are
  untyped constants (V.7): `x: f32 = math.PI` is `PI` rounded to `f32`, and with nothing to fix its
  type a use is `f64`.
* `[STD-27]` *(new in 0.9.9)* `std.math.Number` is every number type. `std.math` adds each with an
  `extend … implements Number` block stating its `type Real: Float`, the decimal type its answers
  come back as (`f64` for every integer type, `f32` for `f16`, which it holds exactly (ODR-041), and
  itself for `f32` and `f64`), and `to_real`, which turns it into one. `[STD-21]`'s functions take `T: Number` and answer in `T.Real` (`[IFC-4]`); a program
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
* `[STD-28]` *(new in 0.9.9)* **Vectors, matrices, rotations and shapes** (ODR-043). `std.math` has
  `Vec2`, `Vec3`, `Vec4` (`f32` components `x`, `y`, `z`, `w`), `IVec2/3/4` (`i32`), `UVec2/3/4`
  (`u32`), `Mat2`, `Mat3`, `Mat4` (`f32`, column-major: the columns are `x_axis`, `y_axis`,
  `z_axis`, `w_axis`, and a transform's translation is `w_axis`), `Quat` (`x`, `y`, `z`, `w`, `w`
  the scalar part), `Transform` (`translation`, `rotation`, `scale`), `Aabb` (`min`, `max`),
  `Sphere` (`center`, `radius`), `Ray` (`origin`, `direction`), `Plane` (`normal`, `d`: the points
  `p` with `normal.dot(p) + d == 0`) and `Frustum` (six planes facing in: `left`, `right`, `bottom`,
  `top`, `near`, `far`). Each is `@layout(c)` and `Copy`, with a public memberwise constructor.
  * Vectors have the constants `ZERO`, `ONE` and the unit axes (`X`, `Y`, `Z`, `W`); `splat`;
    component-wise `+`, `-`, `*`, `/` (`//` and `%` for the integer ones), the same with a scalar
    on either side (`2.0 * v`, `[TYP-21]`), and unary `-` (not `UVec`); `min`, `max` and `abs` (not
    `UVec`), component-wise; `dot`; for the float vectors `length`, `length_squared`, `distance`,
    `distance_squared`, `normalize`, `normalize_or_zero`, `lerp` and `mul_add` (component-wise,
    each rounded once, `[STD-3]`); `Vec3.cross`; and `extend` and `truncate` between sizes.
  * Matrices have `IDENTITY`, `ZERO`, `from_cols` and `from_diagonal`; `m * n` (which applies `n`
    first), `m * v`, `m * s` and `s * m` for an `f32` `s`, `+`, `-` and unary `-`; `transpose`,
    `determinant`, `col(i)`, `row(i)`, `inverse` (a matrix with none panics) and `try_inverse` (an
    `Option`). `m * v` is `m.x_axis * v.x + m.y_axis * v.y + …`, each term after the first added
    by a fused multiply-add (`[STD-3]`). `Mat2` and `Mat3` have `from_angle` (a rotation in the
    plane); `Mat3` and `Mat4` `from_quat` and `from_scale`; `Mat4` `from_translation`,
    `from_rotation_x`/`y`/`z`, `from_scale_rotation_translation`, `look_at_rh`, `perspective_rh`
    and `orthographic_rh` (right-handed, depth in `[0, 1]`), `transform_point3` (`w` = 1, divided
    by the result's `w`) and `transform_vector3` (`w` = 0).
  * Rotations are right-handed. `Quat` has `IDENTITY`, `from_axis_angle` and
    `from_rotation_x`/`y`/`z`; `q * p` (the rotation `p`, then `q`) and `q * v` (the vector
    rotated); `length`, `normalize`, `conjugate`, `inverse`, `dot` and `slerp`. `Transform` has
    `IDENTITY`, `from_translation`, `from_rotation` and `from_scale`; `transform_point` (scale, then
    rotate, then move), `transform_vector` (without the move), `t * u` (`u`, then `t`) and
    `to_mat4`.
  * `Aabb` has `from_center_half_extents`, `center`, `half_extents`, `contains`, `intersects`,
    `union` and `expand`; `Sphere` `contains`, `intersects` and `intersects_aabb`; `Ray` `at(t)` and
    `intersect_plane`, `intersect_sphere` and `intersect_aabb`, each the least `t ≥ 0` at which the
    ray is on the plane or in the ball or box (0 when it starts there), as an `Option[f32]`; `Plane`
    `from_point_normal`, `from_points`, `signed_distance` and `normalize`; `Frustum`
    `from_view_projection(m)`, for `m` a `perspective_rh` or `orthographic_rh` projection times a
    view, with `contains_point`, `intersects_sphere` and `intersects_aabb`.

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
* `[STD-25]` *(new in 0.9.9)* *(changed in 0.9.15)* `random.Rng.seeded(seed)` is a deterministic generator (the same seed
  gives the same sequence on every machine); `Rng.from_entropy()` seeds from the system and is `Nondet`.
  `rng.int_in(a..b)`, `rng.float()` (in `[0, 1)`), `rng.choice(span) -> Option[ref T]`,
  `rng.shuffle(ref mut span)` (its parameter is `ref mut s: MutSpan[T]`). Each call changes `rng`,
  which is therefore a name that may change it.
