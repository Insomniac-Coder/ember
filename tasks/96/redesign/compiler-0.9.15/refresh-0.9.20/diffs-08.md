# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 08

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### ECS-2 (p12-dod.md; 154 words changed)
  - …array walks at least as fast. [[A component type's identity is the 64-bit FNV-1a hash of its fully qualified name (`ecs.type_hash[T]()`), stable across builds and modules.]]
  - …across builds and modules. The registry [[keeps]]
  + …across builds and modules. The registry {{keys}}
  - …modules. The registry keeps its pools [[as]]
  + …modules. The registry keeps its pools {{by the type itself, the identity a kind test compares (`[TYP-44]`, `[TYP-44a]`):}}
  - …registry keeps its pools as `pools: [[Map[u64,]]
  + …registry keeps its pools as `pools: {{Map[TypeId,}}
  - …pools as `pools: Map[u64, owns dyn [[AnyPool]`:]]
  + …pools as `pools: Map[u64, owns dyn {{AnyStorage]` with `TypeId` the value `std.core.type_id[T]()` gives (a function the compiler knows), which two types of one program never share, whatever their names;}}
  - …place, `pool = ref mut cast [[this.pools[ecs.type_hash[T]()]]]
  + …place, `pool = ref mut cast {{this.pools[std.core.type_id[T]()]}}
  - …= ref mut cast this.pools[ecs.type_hash[T]()] as [[Pool[T]`]]
  + …= ref mut cast this.pools[ecs.type_hash[T]()] as {{Storage[T]`}}
  - …mut cast this.pools[ecs.type_hash[T]()] as Pool[T]` (`[TYP-44]`, [[`[DSP-4]`).]]
  + …mut cast this.pools[ecs.type_hash[T]()] as Pool[T]` (`[TYP-44]`, {{`[DSP-4]`), and every method that reaches a pool by type (`remove`, `erase`, `all_of`, `any_of`, `clear_of`, `orphan`, `ctx()`) reaches it as `storage[T]()` does. `ecs.type_hash[T]()`, the 64-bit FNV-1a hash of `std.core.type_name[T]()` (a type's fully qualified name, the package's major version included, `[VER-5]`, as Rust's `std::any::type_name`), is stable across builds and names a type in a snapshot (`[ECS-8]`) and to other engines; a snapshot that holds two types of one hash is `SerError.Malformed`, naming both, and no pool is ever reached by the hash alone; the pool type is `Storage[T]`, EnTT's `storage<T>` (`std.collections.Pool[T]` is the handle pool, `[HND-1]`).}}

### ECS-3 (p12-dod.md; 27 words changed)
  + …Span[Entity], value: T)` for a `T: {{needs}}
  - …value: T) -> ref mut T`; [[`all_of[(A, B)](e)`]]
  + …value: T) -> ref mut T`; {{`all_of[A, B](e)`}}
  - …ref mut T`; `all_of[(A, B)](e)` and [[`any_of[(A, B)](e)`;]]
  + …ref mut T`; `all_of[(A, B)](e)` and {{`any_of[A, B](e)`;}}
  + …in reg.storage[T]()` walks. Several types are {{listed in the brackets, any number of them (`[A, B]`,}}
  - …reg.storage[T]()` walks. Several types are one [[tuple type (`[(A, B)]`), since Ember has neither]]
  + …reg.storage[T]()` walks. Several types are one {{alone `[A]`;}}
  - …B)]`), since Ember has neither variadic [[generics nor overloading (`[TYP-19]`).]]
  + …B)]`), since Ember has neither variadic {{generics, `[TYP-47]`, verdict 115).}}

### ECS-4 (p12-dod.md; 33 words changed)
  - …* `[ECS-4]` **Views.** [[`reg.view[(A, B)]()`]]
  + …* `[ECS-4]` **Views.** {{`reg.view[A, B]()`}}
  - …read-only (`for e, a, b in [[reg.view[(A, B)]():`); `reg.view_mut[(A, B)]()`]]
  + …read-only (`for e, a, b in {{reg.view[A, B]():`); `reg.view_mut[A, B]()`}}
  - …mut a, ref mut b in [[reg.view_mut[(A, B)]():`),]]
  + …mut a, ref mut b in {{reg.view_mut[A, B]():`),}}
  - …one entity are values of different [[types,]]
  + …one entity are values of different {{types (`[ECS-4a]`),}}
  - …so they never name one place. [[`.exclude[(C,)]()`]]
  + …so they never name one place. {{`.exclude[C]()`}}
  - …smallest pool and probes the others. [[The library implements its]]
  + …smallest pool and probes the others. {{A}}
  - …others. The library implements its view [[interface for tuples]]
  + …others. The library implements its view {{takes any number}}
  - …its view interface for tuples of [[1 to 8 types]]
  + …its view interface for tuples of {{component types,}}
  - …of 1 to 8 types with [[a blanket implementation over the element types (`[GRM-34]`), so]]
  + …of 1 to 8 types with {{no limit (`reg.view[Position]()`, `reg.view[Position, Velocity, Health, Armor]()`; variadic generics, `[TYP-47]`), and}}

### ECS-5 (p12-dod.md; 2 words changed)
  - …Array[Entity] = []` before the loop, [[`dead.push(e)`]]
  + …Array[Entity] = []` before the loop, {{`dead.push(owns e)`}}

### ECS-8 (p12-dod.md; 84 words changed)
  - …`on_update[T]()` and `on_destroy[T]()` give a signal; [[`.connect(f)` and `.disconnect(f)` take a]]
  + …`on_update[T]()` and `on_destroy[T]()` give a signal; {{`.connect(owns f)` keeps}}
  - …take a `f: fn(ref mut Registry, [[Entity)`, which]]
  + …take a `f: fn(ref mut Registry, {{Entity)` in the registry, so `f` is given with `owns` (`[FN-2a]`) and what it holds goes with it (a listener that views a local the registry outlives is `E3063`), and `.disconnect(f)` compares and keeps nothing;}}
  + …Registry, Entity)`, which the registry calls {{a listener}}
  + …the registry calls after the change {{is made and before the method's result view}}
  - …`[BRW-21]`, holds for the library's own [[code).]]
  + …`[BRW-21]`, holds for the library's own {{code): `emplace`'s `ref mut T` is found again once its listeners have run, and a listener keeps the `ref mut Registry` it is given no longer than its call, as any view.}}
  - …(`emplace`, `get`, `get_mut`, `find`, `contains`, `erase`). [[`group[(A, B)]()`]]
  + …(`emplace`, `get`, `get_mut`, `find`, `contains`, `erase`). {{`group[A, B]()`}}

### STD-6 (p15-stdlib.md; 73 words changed)
  + …(console, files, time, processes) and **ffi**. {{Each module's row in §XV.1 names its layer, and a name whose layer differs from its module's is marked there (`AnyError`, `std.core`'s name but the **alloc** layer's type, `[ERR-8]`).}}
  + …how firmware and kernels use Ember. {{A use of a name of a layer the package does not declare is `E1042`, naming the layer and the line to write ("`Result[int]` means `Result[int, AnyError]`, and `AnyError` needs the `alloc` layer, which `sensor` does not declare; name the error type: `Result[int, SensorError]`").}}

### STD-26 (p15-stdlib.md; 9 words changed)
  - …loop (`[CTL-3b]`). * `enumerate(it, start=0)`, `zip(a, [[b)` (stops]]
  + …loop (`[CTL-3b]`). * `enumerate(it, start=0)`, `zip(a, {{b, …)` (any number of iterables, `[TYP-47]`; stops}}
  - …start=0)`, `zip(a, b)` (stops at the [[shorter)]]
  + …start=0)`, `zip(a, b)` (stops at the {{shortest)}}

### STD-9 (p15-stdlib.md; 70 words changed)
  - …write to standard error. They are [[compiler-known and are]]
  + …write to standard error. They are {{ordinary functions of}}
  - …They are compiler-known and are the [[only calls]]
  + …They are compiler-known and are the {{library,}}
  - …are the only calls with a [[variable number of positional arguments (`[TYP-26]`). An]]
  + …are the only calls with a {{pack (`[TYP-47]`, verdict 117): `[Ts...: needs Printable]` above `pub fn println(values: Ts..., sep: StrView = " ", end: StrView = "\n"):`. The compiler implements `Printable` (§IV.8, in the prelude) for every type that has `Display` or `Debug`, which no program can write by hand: an}}
  + …help is to implement `Display`. A {{program's own function takes any number of printable values the same way (`[Ts...: needs Printable]` above `fn log(values: Ts...):`). A}}

### STD-18 (p15-stdlib.md; 7 words changed)
  - …(`E2250`, ODR-168). `format_to(ref mut buf: MutSpan[u8], [[…)]]
  + …(`E2250`, ODR-168). `format_to(ref mut buf: MutSpan[u8], {{values: Ts...)}}
  - …MutSpan[u8], …) -> Result[StrView, FmtError] from [[buf`]]
  + …MutSpan[u8], …) -> Result[StrView, FmtError] from {{buf`, under `[Ts...: needs Printable]`,}}

### STD-15 (p15-stdlib.md; 32 words changed)
  + …`reserve`, `push`, `pop -> Option[T]`, `insert(i, {{owns}}
  - …`remove(i) -> T`, `swap_remove(i) -> T`, [[`extend(iterable)`,]]
  + …`remove(i) -> T`, `swap_remove(i) -> T`, {{`extend(owns items)` (an iterable of `T`, taken over as `push(owns value: T)` takes its value; a caller that keeps its list writes `xs.extend(owns copy of ys)`),}}
  + …Array[T]`, `dedup`, `reverse`, `sort` (stable; `T: {{needs}}
  + …it as a type parameter: `[F: {{needs}}
  + …above `fn retain(owns keep: F):`, `[K: {{needs}}
  + …retain(owns keep: F):`, `[K: Ord, F: {{needs}}
  + …`fn sort_by_key(owns f: F):`, and `[F: {{needs}}

### STD-11 (p15-stdlib.md; 47 words changed)
  - …in the number of entries. `H: [[Hasher + Default`,]]
  + …in the number of entries. `H: {{needs (Hasher, Default)`,}}
  + …`Map` or `Set` is not `Nondet`. {{A map or set keyed by what an outside party chooses (a request's fields, a file's keys, the command line) is made with `RandomState` (`Map[K, V, RandomState]`, `[HASH-2]`), since a fixed seed lets that party choose keys that collide and make every operation linear.}}

### STD-16 (p15-stdlib.md; 23 words changed)
  - …exist and panics with the same [[message.]]
  + …exist and panics with the same {{message, except as a scoped task's reduction statement, whose join inserts the key (`[THR-21]`).}}
  + …take any `q: Q` with `Q: {{needs}}
  + …`get_or(q, default) -> V` (for `V: {{needs}}
  + …a `MapEntry` with `or_insert(v)`, `or_insert_with(f)` (`[F: {{needs}}
  + …mut V:`) and `or_default()` (for `V: {{needs}}
  - …returning `ref mut V`, a place [[(`counts.entry(k).or_default()]]
  + …returning `ref mut V`, a place {{(`counts.entry(owns k).or_default()}}
  + …`capacity`, `reserve(n)`, `try_reserve(n)`, `clear`, `retain(keep)` (`[F: {{needs}}
  + …and `q in s`; `retain(keep)` (`[F: {{needs}}
  + …new `Set` of copies (for `T: {{needs}}

### STD-12 (p15-stdlib.md; 10 words changed)
  + …any key type `Q` with `Q: {{needs}}
  + …for `K = String`. `interface AsKey[K]: {{needs}}
  + …is_key(key: K) -> bool`; `interface ToKey[K]: {{needs}}
  + …copies a key. `m[q] = v` {{takes a `Q:}}
  - …a key. `m[q] = v` needs [[`Q:]]
  - …the map's own `String`. Every `K: [[Eq + Hash`]]
  + …the map's own `String`. Every `K: {{needs (Eq, Hash)`}}

### STD-17 (p15-stdlib.md; 2 words changed)
  + …`Index[Q]` and `IndexMut[Q]` for every `Q: {{needs}}
  + …and `IndexSet[Q, V]` for every `Q: {{needs}}

### STD-8 (p15-stdlib.md; 91 words changed)
  - …* `[STD-8]` **`Contains`.** `x in c` [[requires `c: Contains[typeof(x)]`]]
  + …* `[STD-8]` **`Contains`.** `x in c` {{is `c.contains(x)` (`[STD-8b]`), so `c` MUST implement `Contains[Q]` for the type of `x`}}
  - …Contains[typeof(x)]` (`E2226` otherwise, whose help is [[`c.iter().any(fn(e)]]
  + …Contains[typeof(x)]` (`E2226` otherwise, whose help is {{`c.iter().any(owns fn(e)}}
  - …otherwise, whose help is `c.iter().any(fn(e) => [[e]]
  + …otherwise, whose help is `c.iter().any(fn(e) => {{x}}
  - …help is `c.iter().any(fn(e) => e == [[x)`).]]
  + …help is `c.iter().any(fn(e) => e == {{e)`); a literal needle is a `StrView` (`[LEX-20]`) where an implementation takes one, and makes a `String` (`[TXT-9]`, one allocation) only where none does.}}
  - …it for `Set` and `Map` (by [[key);]]
  + …it for `Set` and `Map` (by {{key, for every `Q: needs AsKey[K]`, `[STD-12]`);}}
  + …`MutSpan` and `[T; N]` with `T: {{needs}}
  - …with `T: Eq`, as a linear [[scan;]]
  + …with `T: Eq`, as a linear {{scan, and for those of `String` with a `StrView` needle as well, each element read as a view (`[TYP-5]`), so a text view is looked for in any collection of `String`s alike and `who.trim() in names` or `"bob" in names` makes no `String`;}}

### STD-19 (p15-stdlib.md; 126 words changed)
  + …the item would make (for `Item: {{needs}}
  + …over too, so a named iterator {{is}}
  - …a named iterator given to `zip` [[is handed over (`copy]]
  + …a named iterator given to `zip` {{with `owns` (`a.zip(owns it)`; `a.zip(owns copy}}
  - …`zip` is handed over (`copy of [[it`]]
  + …`zip` is handed over (`copy of {{it)`}}
  + …the `…_by` and `…_by_key` forms): `[F: {{needs}}
  + …for_each(owns f: F) owns this:`, `[F: {{needs}}
  + …F) owns this -> bool:`, `[F: {{needs}}
  - …kinds in one list (`handlers: Array[fn()]`). [[The]]
  + …kinds in one list (`handlers: Array[fn()]`). {{A}}
  + …one list (`handlers: Array[fn()]`). The function {{written in the call}}
  - …list (`handlers: Array[fn()]`). The function is [[handed over]]
  + …list (`handlers: Array[fn()]`). The function is {{given to it, with `owns`}}
  - …every argument for an `owns` parameter [[is,]]
  + …every argument for an `owns` parameter {{is (`[FN-2a]`),}}
  - …can be kept or returned (`return [[xs.iter().map(fn(x)]]
  + …can be kept or returned (`return {{xs.iter().map(owns fn(x)}}
  + …what the lambda holds with it. {{A function from a variable is lent to the call and takes no word, as to any function that takes it as a type parameter (`[FN-2a]`): after `xs.iter().for_each(h)`, `h` is the caller's still and `h(5)` runs it (ODR-240).}}
  - …making what the function changes, now [[(`scores.for_each(fn(s): LOG.push(copy]]
  + …making what the function changes, now {{(`scores.for_each(owns fn(s): LOG.push(owns copy}}
  - …outer count times the inner one [[(an]]
  + …outer count times the inner one {{(the smallest of `int`, `u64`, `u128` and `u256` that holds the product of the greatest counts the two sides can give, by the numbers the compiler knows: two}}
  - …times the inner one (an `int` [[by]]
  + …times the inner one (an `int` {{counts whose product may pass `int.MAX` count in `u128`, and stay}}
  - …one (an `int` by an `int` [[stays an `int`).]]
  + …one (an `int` by an `int` {{where the known numbers show the product fits), and number their items (`Position`) in the same type.}}
  + …prelude name: `interface FromIterator[T]:` declares `[I: {{needs}}
  + …`next_if_eq(v)` when it equals `v` (`Item: {{needs}}

### STD-21 (p15-stdlib.md; 1 words changed)
  + …each taking any number type (`[T: {{needs}}
