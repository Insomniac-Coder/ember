# Design choices waiting for the owner: `Map`/`Set`, and iteration

Two pieces of Phase 1 that are large enough that the shape should be chosen before
they are built (owner rule: price both shapes first). Both are M1 items in
`docs/MIGRATION-0.9.9.md`. Nothing here is built.

## 1. `Map[K, V, H = DefaultHasher]` and `Set[T, H]`

What the spec asks (`[STD-11]`, `[STD-16]`, `[STD-12]`, `[STD-17]`, `[HASH-1]`–`[HASH-4]`,
`[TYP-38]`, `[GRM-26]`, `[CTL-1]`):

- a hash map that **iterates in insertion order** (Python's `dict`); re-assigning keeps the
  position, removing keeps the others' order; expected O(1) lookup, insert, remove;
- `K: Eq + Hash`, hashed through the `Hash` protocol with a fixed-seed default hasher, and a
  hasher type parameter (`RandomState`);
- `m[k]` (panics with the key's `Debug`), `m[k] = v` (`IndexSet`), `get`, `get_mut`, `get_or`,
  `insert`, `remove`, `entry(k).or_insert(v)`, `k in m`, `for k in m` (keys), `keys`, `values`,
  `values_mut`, `items`, `len`, `is_empty`, `clear`, `retain`; borrowed keys (`str` for `String`);
- `Set`: `add`, `remove`, `in`, `union`, `intersection`, `difference`, `is_subset`, `|`, `&`, `-`;
- literals `{k: v}`, `{a, b}`, `{}` (empty map), `Set[T]()`; `{…}` comprehensions;
- implicit `Eq` (as sets of entries), `Debug`, `Clone` (`[STR-5]` table).

### Shape A — written in Ember, in `std/src/collections.em`

`Map[K, V, H]` as a generic struct: an entries `Array` of `(K, V)` slots in insertion order
(tombstones on removal, compacted when they pass a threshold) and an open-addressed index
`Array[u32]` into it. Methods are ordinary generic Ember; the compiler only routes syntax
(`m[k]`, `m[k] = v`, `k in m`, `for`, literals, printing) to them, as `[ERR-4]`'s `map` is routed.

- **For:** it is what Part XX says the library becomes; one implementation for every `K`, `V`,
  `H`; the hasher parameter is real; each gap it finds in generics (bounds on type parameters,
  `Hash` through `H: Hasher`, `Option[V]` of a generic `V`, drop of a generic payload) is fixed
  once for all library code, which `[STD-19]`'s adapters need too.
- **Against:** slower to build (each generics gap is found and fixed on the way); `Hash` is not
  implemented for `String`, `str`, `char` or tuples in `std` yet, and `@derive(Hash)` is not built.
- **Price:** large. Several sessions, most of it generics work that pays off elsewhere.

### Shape B — compiler-known, with a C table

`TyKind::Map { key, value }` like `TyKind::Vec`, and a runtime table in `ember_rt` parameterised
by element sizes and per-type callbacks (hash, eq, drop) that codegen emits, as it emits
`ArrayHelper`s today.

- **For:** faster to a working `dict` (the patterns exist: `Vec`, drop glue, printing, eq).
- **Against:** a second, compiler-private collection mechanism; the hasher parameter is fixed
  or faked; every method is a builtin in the checker; `Set` doubles the builtins; it does not
  move the library forward, and would be replaced when generics can carry it.
- **Price:** medium.

**Recommendation: A**, built after the generics gaps it needs are listed (a short probe
session), because the spec's `H` parameter and Part XX both point there, and the same
generics work is required by `[STD-19]`.

## 2. Iteration: generators (`[CORO-*]`), generator expressions (`[GRM-38]`), adapters (`[STD-19]`)

What the spec asks: `gen fn` frames that implement `Iterator` (`next` resumes to the next
`yield`); `(e for x in xs if c)` as a lazy value; `map`, `filter`, `enumerate`, `zip`, `take`,
`skip`, `step_by`, `rev`, `chain`, `flat_map`, … and the consumers; **and** `[CTL-3b]`: a chain in
a `for` loop compiles to one induction-variable loop with no iterator object and no call per
element, whatever the C compiler does.

### Shape A — a coroutine transform in MIR, adapters as library generics, fusion in the checker

`gen fn` bodies become a state machine (a frame struct of the locals live across a `yield`, and a
`next` that switches on the state), done once in MIR. A generator expression is an anonymous
`gen fn`. Adapters are generic structs in `std` over any `Iterator`. `[CTL-3b]` is met by the
checker recognising an adapter chain whose source is a range, view or array **written in the
`for` header** (or consumed at once by `sum`/`any`/`collect`…) and lowering it to one counted
loop, as comprehensions are lowered today; a chain stored in a variable is an ordinary object.

### Shape B — no general coroutines: fuse everything at the use

Treat generator expressions and adapter chains as compiler pipelines, never objects, and lower
every use site to a loop; store one in a variable only if it is consumed in the same function.

- **Against:** `gen fn` (a spec feature in its own right) is still unbuilt; a pipeline passed to a
  function or returned has no representation; `next()` by hand has none either.

**Recommendation: A.** The coroutine transform is the largest single piece of compiler work left
in Phase 1, and it serves three rule families at once.

## Order proposed

1. Generics gaps probe (for `Map` and adapters), then `Hash` for `String`/`str`/`char`/tuples
   and `@derive(Hash)`.
2. `Map`/`Set` in Ember (shape A).
3. The coroutine transform; generator expressions; adapters with `[CTL-3b]` fusion.
