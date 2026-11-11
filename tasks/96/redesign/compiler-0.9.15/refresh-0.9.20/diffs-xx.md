# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area xx

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### PHIL-13 (p01-overview.md; 6 words changed)
  - …one check a profile turns off; [[it may not have side effects,]]
  + …one check a profile turns off; {{its argument changes nothing (`E3023`, §VI.6),}}

### NEW LEX-1a (p02-lexical.md)
* `[LEX-1a]` A Unicode bidirectional format character (U+202A to U+202E, U+2066 to U+2069) anywhere in a source file, an overlay (`[GRM-35]`) or `ember.toml`, in a comment, a string, raw string, byte or C string literal included, is `E0012` ("line 3 holds U+202E, which reorders how the line is shown; write the text as an escape, `\u{202E}`, inside the literal, or remove it"): what a reviewer sees must be what the compiler reads (`[PHIL-16]`). The one way to put one in a program is the escape of `[LEX-26]`, and the formatter never writes one. This is the one diagnostic a comment produces (`[LEX-11]`).

### LEX-10 (p02-lexical.md; 35 words changed)
  - …block comments. A line beginning `#!` [[before the first import or item]]
  - …import or item is a directive [[(`[GRM-37]`), not]]
  + …import or item is a directive {{wherever it stands (`[GRM-37]`) and never}}
  - …is a directive (`[GRM-37]`), not a [[comment; anywhere else it]]
  + …is a directive (`[GRM-37]`), not a {{comment: one below an import or an item is `E0104` ("`#! module overflow(wrap)`}}
  - …comment; anywhere else it is a [[comment.]]
  + …comment; anywhere else it is a {{directive, and directives come first; move it above `import std.io`").}}

### LEX-11 (p02-lexical.md; 25 words changed)
  - …is discarded in silence. Apart from [[`[LEX-23]`,]]
  + …is discarded in silence. Apart from {{`[LEX-23]` and `[LEX-1a]`, and a comment line between a type-parameter line and its declaration, which leaves the bracket line not directly above it (`E0100`, `[GRM-40]`),}}

### LEX-16 (p02-lexical.md; 41 words changed)
  + …fit the resulting type is `E2010`. {{A literal with a suffix has the suffix's type; one that does not fit it is `E2010`. A unary minus directly before it forms the negative constant of that type: `-128i8` is −128 of type `i8`, and `128i8` alone is `E2010`.}}

### LEX-18 (p02-lexical.md; 15 words changed)
  + …on `1`; `1.0` is a float. {{After `.` the lexer reads an integer, never a float: `t.0.1` is two field accesses.}}

### LEX-26 (p02-lexical.md; 67 words changed)
  + …byte or C string ASCII characters {{and the escapes but `\u{…}` (`E0009`: "a byte string holds bytes; write the UTF-8 bytes, `\xF0\x9F\x98\x80`,}}
  - …or C string ASCII characters or [[escapes,]]
  + …or C string ASCII characters or {{a text literal"), a C string none of `\0` and `\x00` either (`E0009`: "a C string ends at its first NUL; write `c"ab"`"), a text literal `\x` with `00` to `7F` only, since its text is UTF-8 (`[LEX-20]`; above it is `E0009`: "write the character, `\u{FF}`, or a byte string, `b"\xFF"`"),}}

### GRM-34 (p03-grammar.md; 41 words changed)
  + …* `[GRM-34]` `extend [T: {{needs}}
  - …does makes a **blanket implementation**: `extend[K: [[Eq + Hash,]]
  + …does makes a **blanket implementation**: `extend[K: {{needs (Eq, Hash),}}
  + …`extend[K: Eq + Hash, V, Q: {{needs}}
  + …two it stays open (`E2060`, ODR-112). {{An `extend` block adds methods, constants and associated types; a field in one is `E0100` ("an `extend` block adds no field: a type's layout is its declaration's (`[TYP-11]`); write `y: int = 0` in `struct P:`").}}

### NEW GRM-3a (p03-grammar.md)
* `[GRM-3a]` **What follows `weakref`.** After `weakref` stands the target's own type, with `thread_shared` before it for a link to a `thread_shared` value: `parent: weakref Node = None` links a node with one owner or with `shares` owners, `parent: weakref thread_shared Node = None` a `thread_shared` node (`[TYP-45]`, `[WK-11]`). Any other word between `weakref` and the type is `E0100`, whose message gives the line to write: `weakref shares T` and `weakref shares mut T` ("a link does not say how its target's owners hold it: `weakref Player` links a player with one owner or with `shares` owners alike; write `scout: weakref Player = None`"); `weakref owns T` ("a value held `owns` has no header for a link to test; a link names a class object or a value held `shares` or `thread_shared`: write `mesh: shares Mesh` in place of line 4 and `last: weakref Mesh = None` here"); `weakref ref T`, `weakref ref mut T`, `weakref sharedref mut T`, `weakref weakref T` and `weakref *T` ("a link names the value itself, not a view, a link or a pointer to it; write `target: weakref Player = None`"). In a capture list and before a value, `weakref thread_shared` stands only before `this` (`[OWN-13]`).

### GRM-31 (p03-grammar.md; 1 words changed)
  + …associated type states it (`type Iter: {{needs}}

### GRM-5 (p03-grammar.md; 57 words changed)
  + …before any target is written (`[EXP-2]`). {{Each target follows `[GRM-4]` on its own: a name not in scope is declared with its part's type, one in scope is assigned (`a, b = b, a` swaps two names in scope; `q, r = divmod(7, 2)` declares both), and `_` discards its part (`[LEX-13]`); a name that appears twice in one target list is `E1020`.}}

### GRM-29 (p03-grammar.md; 42 words changed)
  - …An `expr_list` of two or more [[expressions, or of one expression followed by a comma,]]
  + …An `expr_list` of two or more {{expressions}}
  + …`return x, y` returns a tuple. {{`(e,)`, and one expression followed by a comma in an `expr_list` (`return x,`), a `target_list` (`a, = e`) or a pattern (`(p,)`), is `E0100` ("a tuple has two or more elements; write `e`").}}

### GRM-17 (p03-grammar.md; 62 words changed)
  - …function on the preceding line (`[GRM-28]`). [[Outside brackets, a]]
  + …function on the preceding line (`[GRM-28]`). {{A lambda's}}
  + …(`[GRM-28]`). Outside brackets, a `:` body {{is one simple statement, inside brackets (`[LEX-6a]`) and outside alike: outside brackets it stands}}
  - …`:` body on the lambda's own [[line is one simple statement,]]
  + …`:` body on the lambda's own {{line,}}
  - …simple statement, ended by the statement's [[`NEWLINE`: `h]]
  + …simple statement, ended by the statement's {{`NEWLINE` (`h}}
  - …+= x` assigns the lambda to [[`h`.]]
  + …+= x` assigns the lambda to {{`h`), and a `NEWLINE` and `INDENT` after the `:` is `E0106` ("a lambda's body is one simple statement; declare a local function: `fn add(x):` on the line above with the block below it, then `h = add`", `[GRM-28]`).}}

### NEW GRM-47 (p03-grammar.md)
* `[GRM-47]` **`owns` is never an operand.** An operator reads its operands in place (`a + b` calls `Add.add(a, b)` with both read in place, `[TYP-21]`), so `owns` before an operand hands nothing to the operator and only ends the name: `owns i * 7` is `(owns i) * 7`, which ends `i` and multiplies what it held. `owns` is the one word that ends what it stands before; so an `owns` expression that is an operand of an operator of rows 4 to 17 of the table (`or` to `**`: a comparison, `is`, `in`, a range's end, `as`, a prefix `-`, `~` or `not` included), with or without brackets of its own, is `E0119`, whose message names the operator and gives the line to write: in an argument, "`owns` here is an operand of `*`, which only reads `i`: `i` would be handed over before the multiplication, and the product is a new value; write the product in brackets: `m.insert(owns (i * 7), owns i)`"; on the right of `=`, "`owns` here is an operand of `*`, which only reads `n`: `n` would end before the multiplication, and `=` hands the product over with no word; write `b = n * 2`". The other words make a value and end nothing, so before an operand each means what it means elsewhere (`copy of a + b` adds a copy of `a` to `b`; `ref a + b` is `E0111`, `[GRM-36]`).

### TYP-10 (p04-types.md; 18 words changed)
  - …type the shift panics, in every [[profile.]]
  + …type the shift panics, in every {{profile ("`shift out of range`: `1 << n` on line 9 shifts an `int` by 70; use `wrapping_shl`").}}

### RNG-5a1 (p04-types.md; 1 words changed)
  + …2` are the representation's, and `T: {{needs}}

### TYP-12 (p04-types.md; 13 words changed)
  + …* `[TYP-12]` A unit-only enum is {{stored as}}
  - …is the smallest signed type that [[fits).]]
  + …is the smallest signed type that {{fits); its values are its variants, and the conversions are `[ENM-3]`'s.}}

### TYP-15a (p04-types.md; 30 words changed)
  - …The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) [[and the view containers `BorrowList[T]`]]
  - …views bounded by the container's own [[region.]]
  + …views bounded by the container's own {{region (`[ARN-5]`); no other container bounds the views it holds by its own region: an `Array[StrView]` holds views bounded by what each points into (`[TYP-15]`).}}

### NEW TYP-45a (p04-types.md)
* `[TYP-45a]` **One word on a type.** A word of `[TYP-45]` stands before a type that has no word of its own: a `path_type`, a tuple, a fixed array, a function type or a `dyn` bound (`owns Mesh`, `shares mut Player`, `weakref Node`, `ref Span[int]`, `owns dyn Shape`, `thread_shared fn()`), and `weakref` also before `thread_shared T`, the link to a `thread_shared` value (`[GRM-3a]`). `ref` and `ref mut` also stand before an owner word or a link (`ref mut owns Mesh`, `ref weakref Node`): the view of a place that holds such a value, read through as `[TYP-14]` says, and `=` on the `ref mut` one replaces what the place holds, as `e.mesh = owns Mesh(...)` does; never before `ref`, `ref mut` or `sharedref mut`: `ref ref T` and `ref mut ref mut T` are `E0100` ("a view of a view names the same place; write `ref mut T`"). `*T` and `*mut T` are raw and stand before any type (`[TYP-7]`, §IV.5). Every other word before a word (`owns ref T`, `shares mut ref mut T`, `owns owns T`, `owns shares T`, `thread_shared shares T`; `weakref` before any word but `thread_shared` is `[GRM-3a]`'s) is `E0100`, whose help gives the type with one word ("`shares mut ref mut Item` stacks two words; a counted block holds a value of its own: write `shares mut Item`, or, for one view that several names may change, `sharedref mut Item`"). A value that is to hold a view is a struct with a view field (`[TYP-34]`), and one that is to hold a link is a struct with a link field. `Box`, `Shared`, `Weak`, `SyncShared`, `ForeignBox`, `RefCell`, `Ref` and `RefMut` are not types of Ember: naming one is `E1010`, whose message gives what to write (`owns T` for `Box[T]` and `ForeignBox[T]`, `owns dyn I` for `Box[dyn I]`, `owns T using A` for `Box[T, A]`, `shares T` or `shares mut T` for `Shared[T]`, `weakref T` for `Weak[T]`, `thread_shared T` for `SyncShared[T]`, and for `RefCell` the field written plainly and changed through a name that may change it).

### NEW TYP-46a (p04-types.md)
* `[TYP-46a]` **A value made in an instance follows its manager between threads.** For `Send` and `ThreadShared`, a value whose growth or end calls its instance's methods (one that keeps the instance word, and one in a `FoundByAddress` manager, whose end finds the instance from its address, `[ALC-1]`) holds a `ref A` to that instance: it is `Send`, and `ThreadShared`, only when `A` is a manager any thread may call (below; `[THR-8]`, `[THR-9]`), since those calls run on whichever thread holds or ends it. A value made in a manager that frees nothing one by one and does not grow holds nothing. A manager **any thread may call** is a `ThreadShared` type whose `deallocate` and `reallocate` read `this` (`[EXC-15]`, a fact of its summary, `[BLD-14]` item 1; a `static` manager's do, `[ALC-1]`, and so do those of a type that locks inside, `inner: Mutex[Slab]`), since a method a `ref A` may call is one that reads its object; a value whose end or growth would call a method that changes its instance holds the `sharedref mut` link of `[LT-4]` instead, which is neither `Send` nor `ThreadShared` (`[LT-46]`, verdict 164), whatever the instance's type. So `thread_shared T using A` is `E7015` at the making line when the value made would hold a link to an instance that is not one any thread may call: a value that grows made in an `Arena` (`thread_shared [String("a")] using frame`, the list of verdict 165), or any value in a manager whose `deallocate` changes `this` ("`msg` would end on whichever thread holds its last owner, and that end gives its block back through `Pool.deallocate` (line 12), which changes `pool`; make it in the built-in manager, `msg = thread_shared Msg()`, or in a manager any thread may call, whose `deallocate` reads `this` and locks inside"); a place of such a type (`log: thread_shared Log using Pool`) is `E7015` at the type, since no value could fill it. A `thread_shared` value that holds nothing (one that does not grow, made in a manager that frees nothing one by one: `stats = thread_shared Stats() using level` with `level` an `Arena`, `[ARN-2]`) is `Send` and `ThreadShared` when `T` is, and is accepted, to a scoped task or a job of a scope from any instance and to an unscoped thread, a job outside a scope or a callback foreign code keeps from a `static` instance alone (`[LT-4]`, `E3063` otherwise): its end on any thread runs its `drop` and touches the arena not at all, every owner of it is counted in its away count from its making line (`[RC-4]` point 4), and the arena is neither reset nor ended while it lives (`[ALC-5]`, `[LT-4]`). For `[THR-11]` a value's growth and end count as the calls they make on its instance, by those methods' summaries (`[EFF-24]`): a change of the instance where the method changes it, a read where it reads it (a manager any thread may call), nothing where the value holds nothing; so two scoped tasks growing values made in one instance are `E7007` naming the instance ("the tasks on lines 7 and 9 grow `xs` and `ys`, both made in `current`, so both change `current`; give each task an arena of its own, `a = Arena(1 << 20)` inside its function, or make the lists in the built-in manager, `xs = []`"), unless `A` is a manager any thread may call, whose `reallocate` and `deallocate` read the instance, and two tasks ending values made in such a manager read it, which conflicts with nothing but a change of what they read. For the exclusivity rules `[ALC-5]` stands: growth and end are the changes of the instance it allows.

### NEW TYP-47 (p04-types.md)
* `[TYP-47]` **Any number of types, and of values** (verdicts 115 and 116, C++'s spelling). `...` after a type parameter's name makes it a **pack**: any number of types, none included (`[Cs...]` above `fn view() -> View[Cs...]:`). A pack's needs apply to each of its types (`[Ts...: needs Display]`). A function's last parameter may take the pack's values, any number of them, each of its own type: `[Ts...: needs Display]` above `fn log(values: Ts...):`, called `log("hp", hp, pos)`. `Cs...` among type arguments spells the pack's types out (`View[Cs...]`), and so does it among a tuple type's elements (`(Entity, Cs...)`) and a function type's parameters (`fn(Ts...)`, `Array[fn(Ts...)]`); a type written over a pack is formed once the pack is spelled out, so a tuple type that then has two or more elements is a tuple, one that has one is that type itself, and one that has none is `void` (`[TYP-27]`: there is no one-element tuple; `View[Position]`'s items are `(Entity, Position)`, `zip(xs)` of one iterable gives each item plainly, and a tuple's interfaces are written once from two elements up, `[A, B, Ts...]` above `extend (A, B, Ts...) implements Display:`). A `struct` or `class` takes a pack as a function does (`[Cs...]` above `struct View:`), its last type parameter. `values...` among a call's arguments passes the values on (`log(values...)`), and `values.len()` is their number, known when compiling. `for v in values:` is written out by the compiler once per value, each `v` of its own type, and nothing runs for the loop itself. Each use is made for its exact types, as any generic is (`[MONO-1]`): nothing is looked up while the program runs, and each value is checked as an ordinary parameter's is, its word included (`owns values: Ts...` takes each one over). A pack is the last of its type parameters, and its values' parameter is the function's last positional one: a parameter after it has a default and is passed by name only, as after Python's `*args` (`fn println(values: Ts..., sep: StrView = " ", end: StrView = "\n")`, called `println(a, b, sep=", ")`); anywhere else either is `E0100`. C's untyped `...` stays as `[FFI-8]` says, in `unsafe` only. `...` is written after a pack's name only (`Cs...`, `values...`); a type formed over each type of the pack (`Option[Cs]...`, C++'s pattern expansion) is not Ember and is `E0100`, whose help is a second pack over the types the use needs (`[Cs..., Os...]` is not it either, two packs being `E0100`, `[TYP-47]`: write the function over the pack it takes and form the other types in its body).

### TYP-37 (p04-types.md; 1 words changed)
  + …alike: `a < b` with `T: {{needs}}

### HASH-2 (p04-types.md; 2 words changed)
  - …one rotate per word (FxHash's class). [[`std.collections. RandomState`]]
  + …one rotate per word (FxHash's class). {{`std.collections.RandomState`}}

### HASH-4 (p04-types.md; 3 words changed)
  - …* `[HASH-4]` `Map[K, V]` requires `K: [[Eq + Hash`.]]
  + …* `[HASH-4]` `Map[K, V]` requires `K: {{needs (Eq, Hash)`.}}

### NEW TYP-44a (p04-types.md)
* `[TYP-44a]` **One index per type in a process.** A type index is a number no two types in one process share, given when the program is linked, and for a reload image when it is loaded (`[HR-12a]`), never when a package is compiled: compiled code names a kind by its type-information symbol (`em_ti_…`, `[MNG-4]`), the linker gives each such symbol in the process one index, so a generic instance deduplicated at link (`[MONO-1]`) has one, and fills each compare with it. A kind test or cast is then one load of the header's index, one `and` clearing the kept-by-C mark (`[OBJ-1]`) and one compare against a constant (§X.4); where the C toolchain cannot fill an immediate from a link-time value, one load of that constant per function, which `ember inspect` reports on the same row.

### TYP-32 (p04-types.md; 78 words changed)
  - …iterator pipeline whose type cannot be [[written:]]
  + …iterator pipeline whose type cannot be {{written (the example below). A `some` list (`some (Iterator[Item = int], Send)`, `bound_list` in §III.2, `[TYP-22]`) names one or more interfaces and any number of the marker interfaces (`Send`, `ThreadShared`, `[TYP-43]`): the hidden type implements each interface and has each marker, checked at every `return` of the function (`E2040` otherwise, naming the interface or marker and the returned type), and a caller may rely on the interfaces and markers written and on nothing else of the hidden type.}}

### TYP-26 (p04-types.md; 73 words changed)
  - …and `extend` blocks for different types. [[The compiler-known]]
  + …and `extend` blocks for different types. {{A C++ name a header declares several times is one Ember name of several members, each use standing for one (`[FFI-57]`): the header, not the program, declared them, and a function the program declares beside an imported name is `E1030` as any other. A function whose last positional parameter takes a pack's values (`[TYP-47]`) takes any number of arguments; the}}
  + …`eprintln` (`[STD-9]`) and `format_to` (`[STD-18]`) are {{such functions of}}
  - …(`[STD-9]`) and `format_to` (`[STD-18]`) are the [[only calls with a variable number of arguments (ODR-060).]]
  + …(`[STD-9]`) and `format_to` (`[STD-18]`) are the {{library, not known to the compiler by name (verdict 117).}}

### MOD-8 (p05-declarations.md; 68 words changed)
  + …a local declaration shadows a glob. {{The prelude (`[MOD-5]`) counts as a glob import of `std` written first: a name bound by a glob import and by the prelude, and then used, is `E1031` naming both ("`len` is bound by `from util import *` (line 1) and by the prelude; write `from util import len` to take `util`'s, or `std.len` for the prelude's"), so no package's update can take a prelude name in silence.}}

### NEW MOD-9a (p05-declarations.md)
* `[MOD-9a]` **A library's call through what a dependant may fill.** For each line of a package's functions that calls through a `virtual` method of a `pub` class, a method of a `pub` interface used as `dyn`, or a function-typed place another package can fill (`[EFF-2a]`), the package records in its interface file, beside the function's summary (`[BLD-14]` item 12): the place it calls through; the collections walked across it and the generators still used after it (`[BRW-18]`); the read-only views, `ref mut` names and positions live across it, with what each points into (`[BRW-1]`, `[BRW-17]`); the names opened from links live across it, with their class families (`[RC-5]`); the parameters and `this` used after it (`[BRW-21]`); the locks held at it (`[THR-19]`); and, for its object and each argument, which of the others owns it at that line by the package's own stores, or will by a later line of the function (`[WK-7]`), and whether it is reached through a `thread_shared` value, a `static` or another place every thread may reach (`[OWN-13]`, `[THR-2]`). A package that adds an override, an implementation or a stored function value checks it, at its declaration or at the storing line, against every recorded line it may be called from, with the chain named: a body whose summary changes what such a line walks is `E3020` ("`Door.on_update` clears `WORLD.scripts` (line 9), which `Script.run_all` walks while it calls `on_update` (its line 4); clear after the walk: give `run_all`'s caller the list of doors to remove"), one that changes what it holds or has lent is `E3022`, one that may end what a name opened from a link reads `E3060`, one that takes a lock held there `E7018`, one that stores an owner of its object or of one of its parameters into another that the line's record says owns it `E3032` ("`Keeper.attach` keeps an owner of the host it is given (line 4), and `Host.start` (`lib`, line 12) gives each plugin in `host.plugins` the host that owns it, so the two would own each other; keep a link instead: write `host: weakref Host = None` (line 2) and `this.host = weakref host` in place of line 4"), and one that makes a `weakref T` link of a parameter the record says a `thread_shared` value reaches `E7015`, each at the line that adds the member ("`b.on(...)` (line 7) stores a function that pushes to `b2.log`, and `Bus.flush` (`bus`, line 10) walks `this.log` while it runs the handlers (its line 12); push to a list of your own in the handler and move it into the log after `flush`"). A member no recorded line refuses is accepted; nothing runs for it while the program runs.

### FN-5 (p05-declarations.md; 141 words changed)
  - …* `[FN-5]` [[Default]]
  + …* `[FN-5]` {{A default is the}}
  - …* `[FN-5]` Default argument [[expressions are]]
  + …* `[FN-5]` Default argument {{the caller did not write: it is}}
  + …Default argument expressions are evaluated at {{the call, in the caller, after every written argument and in parameter order, as an argument expression is (`[EXP-1]`, `[EFF-1]`), and it takes no word, since it makes the value it gives: a default is written only for a plain or an `owns` parameter (`fn take(owns xs: Array[int] = []):`,}}
  - …argument expressions are evaluated at each [[call, after]]
  + …argument expressions are evaluated at each {{call makes its own list), and one on a `ref mut` parameter is `E3027` ("`ref mut xs` names}}
  - …evaluated at each call, after the [[other arguments are bound,]]
  + …evaluated at each call, after the {{caller's place,}}
  + …the other arguments are bound, and {{a default would be a temporary that ends with the call; take the list over, `owns xs: Array[int] = []`, or let the caller pass one: `reset(ref mut items)`"). It}}
  - …other arguments are bound, and may [[refer to]]
  + …other arguments are bound, and may {{read an}}
  - …bound, and may refer to earlier [[parameters.]]
  + …bound, and may refer to earlier {{parameter and neither changes it nor hands it over (`E3023`, `E3013`, as a plain name): `t: String = s` reads `s`, which stays the function's own.}}

### STR-2 (p05-declarations.md; 153 words changed)
  - …A field default is any expression; [[it is evaluated]]
  - …any expression; it is evaluated at [[each]]
  + …any expression; it is evaluated at {{a}}
  + …each construction that omits the field, {{the written arguments are evaluated first, left to right (`[EXP-1]`, `[FN-5]`), then the omitted fields' defaults}}
  + …omits the field, in field order, {{each at that construction (every field's, before the body, where the type declares an `init`, `[STR-6]`, `[CLS-2]`),}}
  + …the field, in field order, and {{a default}}
  + …may allocate (`items: Array[int] = []`). {{Names and type expressions in a field default resolve in the type's declaration scope, with the type's generic parameters substituted at instantiation, as `[FN-5]`'s do; a field is named only through `this`, which a default has not, since it runs before the value exists (`E1010`: "`this` can only be used in a method, and a field default runs before `init`, when there is no object yet; set `on_tick` in `init`: write `this.on_tick = fn(): ...` below `this.name = n`"; "`cap`'s default reads `n`, a field of the same `Buf`, which has no value when defaults run; set `cap` in an `init`: `fn init(n: int):` with `this.n = n` and `this.cap = n * 2`").}}

### CLS-3 (p05-declarations.md; 165 words changed)
  - …a memberwise constructor as a struct [[does.]]
  + …a memberwise constructor as a struct {{does (`[STR-1]`), and that constructor is its `init` for every purpose: a derived `init` calls it as `super.init(…)`, with positional or named arguments in field order, a field with a default left out keeping the default `[CLS-2]` has already stored (`super.init(name=name)` for a base with `name: String` and `visible: bool = true`); `[CLS-11]`'s two phases hold around that call as around any `super.init(…)`; and a derived class with no `init` gets it as its base's constructor (`[CLS-10]`: `Panel(name=String("side"))`), and passes it on the same way to a class derived from it. The arguments of `super.init(…)` take the words a call of that constructor takes: none for a memberwise one, whose values are handed over as to every constructor (`[OWN-9]`, so the derived `init` holds them `owns`: `fn init(owns name: String, owns label: String):`), and, for a written base `init`, the words a call of that constructor takes (`[FN-2a]`: none before an argument for its `owns` parameter, `ref mut` before one for its `ref mut` parameter).}}

### CLS-10 (p05-declarations.md; 11 words changed)
  - …`Door`'s field defaults and then runs [[`Script.init(entity)`.]]
  + …`Door`'s field defaults and then runs {{`Script.init(entity)` (the base's memberwise constructor where it declares no `init`, `[CLS-3]`).}}

### IFC-3 (p05-declarations.md; 224 words changed)
  + …* `[IFC-3]` `interface Ord: {{needs}}
  - …requires implementers of `Ord` to implement [[`Eq`.]]
  + …requires implementers of `Ord` to implement {{`Eq`: an implementation meets its interface's parents for every type it covers, checked where it is written (`E2040`, naming the part that does not and giving the line that compiles). A generic implementation covers every instance its brackets allow, so it writes the needs the parents put on its parameters (`extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:`; `extend[T] PoolFull[T] implements Error:` is `E2040` with that line, never read as holding for some arguments only), as a `pub` generic function writes its bounds (`[IFC-6]`). A class that implements an interface with marker parents (`Send`, `ThreadShared`) promises them for every class derived from it, each checked where it is declared (`[THR-8]`), and the implementation itself is never checked again.}}
  + …a bound brings its parents: `T: {{needs}}
  + …names the parent's associated type (ODR-042). {{An implementation writes each of the interface's methods with the interface's first line after the name: the same parameter names, words and types (with `Self`, the interface's type parameters and its associated types standing for the implementing type and what the `implements` clause and the `type` lines give them, so `fn eq(other: Point)` meets `fn eq(other: Self)`), the same words on `this` and the same result, since a call through `dyn I` (`[DSP-3]`) passes what the interface declared; one that differs is `E2040`, naming the method and the interface's line ("`Sq.scale` takes `k: i64`, and `Scaler.scale` takes `k: f64`; write `fn scale(k: f64) -> f64:`").}}

### IFC-4 (p05-declarations.md; 4 words changed)
  + …type may carry bounds (`type Real: {{needs}}
  + …written only in a bound (`T: {{needs}}
  + …leaves an associated type unsaid (`T: {{needs}}
  + …not an equality: a bound `T: {{needs}}

### CTL-10 (p06-expressions.md; 17 words changed)
  + …completes normally, at the **same type**, {{or, where the arms give it different functions, at the callable type they all convert to (`[CLO-1]`),}}

### CORO-7 (p06-expressions.md; 112 words changed)
  - …its suspension point, in reverse declaration [[order.]]
  + …its suspension point, in reverse declaration {{order, then its parameters, in reverse order, and nothing else: the drop is not an exit of the blocks the `yield` lies in, so a `defer` block registered before the `yield` does not run (`[CTL-8]` runs it only when its block exits by the body's own path, on a later resume), and the drop's effects are those of the locals' drops alone (`[EFF-1]`); a value whose clean-up must run however the generator ends is a value with a `drop`, which the frame's drop ends, not a `defer` block; a generator that has run to its end has ended its parameters as a function's `return` does, holds nothing, and dropping it does nothing.}}

### LT-3 (p07-ownership.md; 152 words changed)
  + …String literals, `bytes` literals, `static` items {{(which never change, `[STA-1]`)}}
  + …the `static` region, which outlives everything. {{A `static mut` is a place with one owner, the program, that each function naming it reaches as it reaches a local of its own (`[BRW-16]`: `LOG[0]` in a function is a position, and `[BRW-17]` for a view into its contents): a result that may point into one is recorded in the function's summary as pointing into that variable, beside the parameters (`[LT-22]`, `[EFF-4]`), a call through a function value counts every module variable a function the package can put there returns a view of (`[EFF-2]`), and a caller keeps it viewed as it keeps an argument, so a reshape of it while the result lives is `E3060` at the later use ("`LOG.clear()` (line 7) may free the text `s` views (`head()` returns a view of `LOG`, its line 4), and `s` is used on line 8; keep your own copy: `s = String(head())`") and a change `E3021`.}}

### NEW LT-48 (p07-ownership.md)
* `[LT-48]` **`runtime`: the kind of the name a call goes through, checked while the program runs.** `runtime` is written on a parameter or on a local declared with its type, after the `:` and before `dyn` (under any holding word: `dyn I`, `ref dyn I`, `owns dyn I`, `shares dyn I`, `shares mut dyn I`) or before the name of an `open` or `abstract` class (under any holding word likewise): `fn show(p: runtime dyn Picker):`; `fn show(a: runtime A):` for `open class A`; `q: runtime owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])`; `me: runtime ref A = ref this`; a lambda's typed parameter takes it in the same slot. It has one meaning wherever it stands, and it stands before the whole type, the holding word included. A `dyn` or `virtual` call through that name whose result is a view, and which `[LT-1]` would refuse for some of the kinds that may reach the name and not for others, is not refused when compiling where which kind it is is known only while the program runs. The function then reads, at the call, from the called method's entry in the table of the value's kind (an interface's table for a `dyn` value, `[DSP-3]`; the method's slot in the class's type table for a `virtual` method, `[DSP-2]`), which of the method's parameters its result may point into, keeps that fact with the result for as long as the result is used, and at each later line that changes a place the result may point into for one of those kinds, while the result will still be used, tests the fact's bit for that place and stops the program when it is set. The fact is read once, at the call, so a name given another value between the call and the change (`q = owns Yours(items=[1])`) cannot name the wrong kind: that line is itself a change of what the result may point into for a kind whose result points into `this`, and is tested as any other. The fact is the one `[LT-22]` records for each implementation and each override: bit 0 for `this`, bit k for the method's k-th parameter in the order of its declaration, counted from 1, in one word of 64 bits and one more word for each further 64 parameters, the count taken from the interface's or the base class's declaration of the method, so that every package writes the entry in one shape; it is kept beside the code pointer in the method's entry of the kind's table, so a kind declared in another package, which the function's own package never saw, is checked as any other; a compare of kind identities instead would let such a kind through. The kinds that may reach a parameter are what its callers give it, and those that may reach a local are what the function's own lines give it, followed through names and calls as `[LT-1]` says; where they are not all known when compiling (a `pub` function's parameter, a value read from a field, an element, a call's result, a parameter, or given through a `ref mut` view) every kind that implements the interface or descends from the class is taken to reach the name. Where the kinds are known when compiling and the line is unsafe for every one of them, or where the kind itself is known, the word changes nothing: the line is refused when compiling, as without it (`E3021`); given `show(Yours(items=[9]))` or `show(B())`, `show` is refused at the change. `runtime` guards a call's result; a view put in through a `ref mut` parameter is what `[LT-47]` says, whatever the word. `runtime` is a word on the name, not part of its type: the name's type is the type written after it, the value is given on with no word (`show(q)` for `fn show(p: dyn Picker):`, which decides its own lines by its own first line), a result that leaves the function (returned, stored in a field or a list, kept by a callee, captured) carries no fact and is what `[LT-1]` rule 3 and `[LT-30]` make it, checked when compiling, and a function that calls through a name of its own says `runtime` on that name. With `[LT-1]`'s `Picker`, `fn show(p: runtime dyn Picker):` and `p: owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])`, `show(p)` prints `[9]` for a `Mine`, and for a `Yours` stops at `xs.push(owns 3)`: "panic at main.em:17:5: `xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed". The local `q` above, with `xs = [1, 2]`, `r = q.pick(xs)`, `xs.push(owns 3)`, `println(r)` in the same function, prints `[9]` for a `Mine` and stops the same way for a `Yours`, naming `q`. With `open class A` whose `virtual fn first(other: Array[int]) -> Span[int]` returns `this.items` and `class B(A)` whose `override` returns `other`, `fn show(a: runtime A):` doing `xs = [1]`, `r = a.first(xs)`, `xs.push(owns 2)`, `println(r)`, given `a: A = B() if input() == "b" else A()` and `show(a)`, prints for an `A` and stops for a `B`: "panic at main.em:14:5: `xs` changes while `r` looks at it: `a` is a `B`, whose `first` gives back the list it was handed". The stop names the kind as the compiler shows types, another package's qualified (`support.pickers.Yours`), read from the kind's `TypeInfo` (`[RT-3]`), and what the result points into as the result type shows it ("the list it was handed" for an `Array` or a `Span`, "the text it was handed" for text, "the `T` it was handed" otherwise, "what it holds" for a result that points into the receiver). Without the word, `E3021`'s help names the one line to write, for the name the call goes through: for a parameter, "to check it while running, write `runtime` on the parameter: `fn show(p: runtime dyn Picker):`"; for a local, "to check it while running, write `runtime` where `q` is declared: `q: runtime owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])`" (the declaration line written out, with its type where the program left it to be inferred); for a `virtual` call on `this` in a method of an open class, which has no declaration line of its own, "to check it while running, name the object with `runtime` and call through the name: `me: runtime ref A = ref this`, then `r = me.first(xs)`"; for a value with no name of its own (`ps[0].pick(xs)`, `this.picker.pick(xs)`, `make().pick(xs)`), "to check it while running, name the value with `runtime` and call through the name: `p: runtime ref dyn Picker = ref this.picker`, then `r = p.pick(xs)`" (`ref` of a place holding `owns dyn I`, `shares dyn I` or `shares mut dyn I` is a `ref dyn I`, a read-only view of the value it holds, §IV.9, `[TYP-14]`). `runtime` is written nowhere else. Before a type with one kind (a struct, an enum, a class neither `open` nor `abstract`, `some I`, a list, a number) it is `E2114` ("`Circle` is a final class, so a `Circle` is always a `Circle`; remove `runtime`: `c: Circle`"); on a field, a result type, a type argument, a capture or a local declared without its type it is `E0100` ("`runtime` goes on the parameter or the local a call goes through, so that the function whose line pays says so, not on a field; in the method that calls through `picker`, name its value first: `p: runtime ref dyn Picker = ref this.picker`"; "`runtime` goes on the parameter or the local a call goes through, not on a result type; remove it: `fn make() -> owns dyn Picker:`, and write it where the result is held: `q: runtime owns dyn Picker = make()`"). It is a word only after the `:` of a parameter or of a local declared with its type (`[LEX-15]`); it costs one load at the call and one bit test at each line it guards, and nothing where no line needs them (§X.4). Where a test is emitted it enters the function's effect set as `RuntimeCheck(Aliasing)` (`[EFF-9]`), with the reason `not_provable_in_principle` (`[EFF-11]`), as the compare of `assert_disjoint` does: inside `@no_alias_checks` the word is `E4030` (`[EFF-12]`), naming the kind known when compiling that needs no test; `@nopanic(explicit)` admits it, as it admits every `RuntimeCheck` (`[EFF-16]`). A `runtime` name whose every kind is known when compiling, where nothing is tested, carries nothing, and `ember inspect --safety` lists each test with its line (`[EFF-10]`).

### DSP-2 (p08-classes.md; 95 words changed)
  + …an `override` reuses the base's slot. {{A slot holds, beside the method's code, the fact `[LT-22]` records for the method that fills it (bit 0 for `this`, bit k for its k-th parameter, in the width the base's declaration fixes), read by a `runtime` check (`[LT-48]`); a slot a subclass does not override keeps the base's entry whole. The entry is written by the package that compiles the method's body, and a reload that rebuilds the body writes its new fact beside its new code (Annex B), so an object made before the reload is checked by the code it runs.}}

### ARN-5 (p09-memory.md; 46 words changed)
  - …taken from an arena once, at [[construction;]]
  + …taken from an arena once, at {{construction, through a name that may change it, `ArenaArray.in(ref mut frame, capacity)` and `ArenaMap.in(ref mut frame, capacity)`, and held as a kept `MutSpan` view (`[LT-45]`): each lives no longer than the arena, `reset()` ends it (`[ARN-1]`), and `ember inspect --alloc` reports it as a `view` (`[SEL-1]`);}}

### ARN-5c (p09-memory.md; 3 words changed)
  - …`is_empty`, `get(i) -> Option[ref T]`, `get_mut`, [[`push(v)]]
  + …`is_empty`, `get(i) -> Option[ref T]`, `get_mut`, {{`push(owns v)}}
  + …`get_mut`, `push(v) -> Result[void, CapacityError]`, `insert(i, {{owns}}

### ARN-5d (p09-memory.md; 6 words changed)
  - …* `[ARN-5d]` `ArenaMap[K, V]` (with `K: [[Eq + Hash`)]]
  + …* `[ARN-5d]` `ArenaMap[K, V]` (with `K: {{needs (Eq, Hash)`)}}
  - …provides `len`, `capacity`, `is_empty`, `get`, `get_mut`, [[`insert(k,]]
  + …provides `len`, `capacity`, `is_empty`, `get`, `get_mut`, {{`insert(owns k, owns}}

### ARN-11 (p09-memory.md; 60 words changed)
  + …(a `NonZero`'s field is an integer). {{An enum is `Zeroable` when it declares `@derive(Zeroable)` and all-zero bytes are one of its values: the variant whose discriminant is zero carries no payload or a `Zeroable` one, checked when compiling (`E2080`, naming the variant or field that is not); an enum whose zero bytes are no variant is never `Zeroable`, and the derive on it is `E2080`.}}
  + …is invalid is never `Zeroable`. `T: {{needs}}

### NEW ALC-5a (p09-memory.md)
* `[ALC-5a]` **A link holds the block.** A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one, since the target's block stays until the link ends (`[WK-11]`, `[OBJ-3]`): the link carries the instance's region from the line that makes it (`w = weakref n`) through every place it is copied or stored, as a kept view does (`[LT-45]`); keeping it where it would outlive the instance is `E3063` ("`w` links to a `Node` made in `frame` (line 10), and `frame` ends when `main` returns; end the link first: write `w = None` above the return"), and while it lives a change of the instance that `[ALC-5]` does not allow is `E3021` ("`frame.reset()` (line 13) reuses the memory of the `Node` that `w` links to (made in `frame` on line 10), and `w` is used on line 15; end the link first: write `w = None` above line 13"). A field or element of a `weakref` type whose target was made in an instance carries that instance's region as a kept view field does (`[LT-45]`), so the object or list that holds it lives no longer than the instance (`E3063`). Nothing runs for this: the block a link keeps is given back by the manager's own `deallocate` when the weak count reaches 0, as any other, which for a manager that frees nothing one by one is a no-op, so this check is what keeps the block from reuse. Which give-back that is, the built-in manager's, a call through the instance word the block keeps, nothing, or the instance found from the block's address, is read from the link's type (`weakref Enemy`, `weakref Enemy using Arena`, `[TYP-46]`) when the link's end is compiled, never from the block, which carries no mark for it (`[OBJ-1]`).

### NEW ALC-6 (p09-memory.md)
* `[ALC-6]` **A making line has its instance's effects, and fails as a container does.** A line that makes a value `using x`, and every growth and end of that value, carries the effects of the `Allocator` methods it calls on `x`'s type (`[EFF-1]`); `Alloc` is the effect of `ember_alloc`, so of `Global`'s and `System`'s `allocate` and `reallocate` and of every implementation that reaches them, and a `FixedArena`'s `allocate` has none (`[ARN-4]`). When the instance's `allocate` or `reallocate` returns null, the compiler's own call panics, "out of memory: `owns Cmd(id=1) using fixed` (line 8) asked `fixed` for 16 bytes", the allocation failure `Panic(Explicit)` lists (`[EFF-16]`), so under `@nopanic(explicit)` such a line is `E4040` ("`tick` is `@nopanic(explicit)`, and `owns Cmd(id=1) using fixed` (line 8) panics when `fixed` is full; make it where failure is a value: write `c = fixed.try_alloc(owns Cmd(id=1))` and open its `Result`"), as a container's `push` is unless it is `try_push` or after `try_reserve` (`[ALC-4]`). A `reallocate` that returns null leaves the old block as it was.

### UNS-3 (p09-memory.md; 19 words changed)
  - …block containing statements that need no [[`unsafe`.]]
  + …block containing statements that need no {{`unsafe`, and one written directly inside unsafe code (an `unsafe fn` body, another `unsafe:` block), which adds nothing (`[UNS-12]`).}}

### UNS-5 (p09-memory.md; 89 words changed)
  - …`std.mem` provides `Volatile[*T]` (volatile reads and [[writes),]]
  + …`std.mem` provides `Volatile[*T]` (volatile reads and {{writes, `[STD-29]`),}}
  - …(volatile reads and writes), `transmute[A, B]`, [[`copy_nonoverlapping`,]]
  + …(volatile reads and writes), `transmute[A, B]`, {{`copy` and `copy_nonoverlapping` (which read and write through the raw pointers they are given),}}
  + …`transmute[A, B]`, `copy_nonoverlapping`, `zeroed[T]()` (requires `T: {{needs}}
  + …`zeroed[T]()` (requires `T: Zeroable`) and `MaybeUninit`. {{`transmute`, `copy` and `copy_nonoverlapping` are `unsafe fn`s (`[UNS-1]`): a call of one outside unsafe code is `E3100` ("`copy_nonoverlapping(p, q, 4)` (line 8) writes through the raw pointer `q`, and nothing checks that four `int`s are writable there; put line 8 in an `unsafe:` block with a `# SAFETY:` note saying why that holds here"), and each carries `@safety` (`[UNS-7]`). `zeroed`, `swap`, `replace`, `take`, `drop` and `keep_alive` are safe (`[ARN-11]` vouches for `zeroed`).}}

### HND-1 (p09-memory.md; 20 words changed)
  - …handle yields `None` (or panics in [[`pool[h]`).]]
  + …handle yields `None` (or panics in {{`pool[h]`: "`pool[h]` on line 12: the handle is stale (slot 3, generation 2 of 5); use `pool.get(h)` for an `Option`").}}

### HND-2 (p09-memory.md; 50 words changed)
  - …and a 12-bit generation in a [[`u32`).]]
  + …and a 12-bit generation in a {{`u32`), and its handle names the split as the pool does: `Pool[T, B].insert` gives a `Handle[T, B]`, with `B` defaulting to `Split32`, so `Handle[T]` is `Handle[T, Split32]`, a `u64`, and `Handle[T, Split20]` a `u32`; handles of pools with different splits are different types (an `Array[Handle[Texture]]` takes no `Split20` handle, `E2020`).}}

### NEW DSJ-11 (p09-memory.md)
* `[DSJ-11]` **Views known apart.** Two views are known apart, with no compare, exactly when the compiler proves them apart as `[BRW-11]` and §IX.8 say: the two results of one `split_at`, `get_pair_mut` or `chunks_mut` call; two columns of one SoA; two `alloc*` results of one arena (`[ARN-1]`); two ranges of one list whose bounds are constants, or `j` from `i + 1..`, that do not overlap; views of two different whole variables neither reached through the other; and two names already given the fact by an earlier `assert_disjoint` or `check_disjoint` (`[DSJ-2]`). Nothing else is, whatever an implementation could see: this is the one disjointness proof, and `[EFF-15]` names it.

### NEW EFF-2a (p10-effects.md)
* `[EFF-2a]` **A place another package can fill.** A function given through a type parameter (`[F: needs fn(Item)]`) is compiled once per function given, by the package that gives it, as any generic instance is (`[EFF-4]`, `[COST-3]`), so every function reaching it is seen where its line compiles. Every place of a function type a function value is called through, a parameter of a function type included (the shared type: one body and one indirect call, verdict 38), and every `virtual` or interface method, is compiled by the package that declares it, which sees only its own stores, calls and overrides (`[EFF-2]`, `[BLD-16]`). Where another package can fill such a place (a function-typed parameter of a `pub` function, a `pub` field, a `pub` static, an element of a list or map one of them holds, a place a `pub` function stores a function-typed parameter into, and every place a value from one of these is copied into), override such a method (a `virtual` method of a `pub` class) or implement it (a method of a `pub` interface called through `dyn`), a call through it counts, beside the functions this package puts there, a function another package may put there: one that may call every `pub` function of this package, change and end every place a `pub` name of this package reaches (a `pub static mut`; the fields of an object a `pub` function is given, returns or holds), take every lock a `pub` function takes, and have every effect the place's or method's declared contracts do not exclude (verdict 106); what it keeps of what the call gives it, an owner it stores or a `weakref T` link it makes, is judged where the function is seen (`[WK-7]`, `[OWN-13]`). A place or method no other package can reach keeps `[EFF-2]`'s rule, and so does a `virtual` method of a class that cannot be subclassed outside its package. The message names the route: `E3020` "`h()` (line 11) may add to `HANDLERS` (through `add`, which a function another package stores into a `Handler` may call) while this loop walks `HANDLERS` (line 10); walk a list of your own: write `todo = mem.take(ref mut HANDLERS)` above line 10, `for h in todo:` in place of line 10, and `HANDLERS.extend(owns todo)` below the loop"; `E4001` "`run_all` is `@noalloc`, and `h()` (line 11) may run a function another package stores into a `Handler`; say what a `Handler` may do: write `@noalloc` above `type Handler = fn()`". A dependant's stores and overrides, and the functions it passes to a `pub` function's function-typed parameter, are counted by the dependant at its own lines (`[EFF-4]`, `[CLO-17]`, `[MOD-9a]`, `[BLD-14]` item 16), and `[DET-3]` counts the same way.

### EFF-6 (p10-effects.md; 203 words changed)
  - …checked over the whole reachable call [[graph —]]
  + …checked over the whole reachable call {{graph,}}
  + …callees, drop glue, default arguments, operator {{implementations,}}
  - …drop glue, default arguments, operator and [[interface]]
  + …drop glue, default arguments, operator and {{the}}
  - …default arguments, operator and interface implementations [[reached statically —]]
  + …default arguments, operator and interface implementations {{and overrides the paragraph below admits,}}
  + …full chain to the offending operation: {{**A contract through an interface.** Inside a function that carries a contract, a call through a `dyn I` value, through a `virtual` method, or through a type parameter's bound, counts the implementations and overrides the function's own package sees. Where another package may add one (`I` or the `open`/`abstract` class is `pub`, `[TYP-20]`), the call is admitted under the contract only when the interface's method or the `virtual` method declares that contract itself (`[EFF-2]`), which holds every implementation to it where it is written (`E4010`); a generic bounded by the interface is checked per instance, by the package that makes the instance (`[EFF-4]`). Otherwise the call is the contract's error (`E4001`, `E4002`, …) at the call, in the package that writes it, and its help gives both lines ("`total` is `@noalloc`, and `s.area()` runs whichever `area` the program gives a `Shape`; hold every `Shape` to it: write `@noalloc` above `fn area() -> float` in `interface Shape`, or take the shape as a type parameter: `[S: needs Shape]` above `fn total(xs: Array[S])`"). An interface or class that is not `pub` is implemented in this package only, and such a call counts exactly its implementations.}}

### DET-4 (p10-effects.md; 101 words changed)
  - …* `[DET-4]` `std.math.det` [[provides]]
  + …* `[DET-4]` `std.math.det` {{provides, under the names `[STD-21]` gives them, every function of `std.math` whose platform form is `Nondet` (`[DET-2]`):}}
  + …`[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, {{`asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`,}}
  - …`std.math.det` provides `sin`, `cos`, `tan`, `exp`, [[`log`,]]
  + …`std.math.det` provides `sin`, `cos`, `tan`, `exp`, {{`exp2`, `ln`, `log2`, `log10`,}}
  - …`sin`, `cos`, `tan`, `exp`, `log`, `pow`, [[`atan2`]]
  + …`sin`, `cos`, `tan`, `exp`, `log`, `pow`, {{`cbrt` and `hypot`,}}
  + …`exp`, `log`, `pow`, `atan2` and `sqrt` {{beside them (not `Nondet`, kept so that `det` has every name a `@deterministic` body may need), so that `E4070`'s help for a `std.math` call is always `det`'s function of the same name ("`fall_off` is `@deterministic`, and `math.cbrt(d)` (line 6) is the platform's `cbrt`, whose last bits differ between C libraries; write `det.cbrt(d)`"; `det.log` is `E1010`, whose help is "the natural logarithm is `ln` in `std.math` and `std.math.det` alike: write `det.ln(p)`"),}}

### PAR-2b (p11-concurrency.md; 64 words changed)
  - …inlined: each call to a function [[`[CG-C-3]`]]
  + …inlined: each call to a function {{that meets `[CG-C-3]`'s criteria (`@inline`, an operator on a type of at most 64 bytes, the listed accessors, or at most 40 statements and no foreign call), whether or not this build}}
  + …call to a function `[CG-C-3]` places {{it}}
  - …function `[CG-C-3]` places in the inline [[header,]]
  + …function `[CG-C-3]` places in the inline {{header (a reloadable function in a hot-reload build, `[HR-9a]`, or a build on the LLVM backend),}}
  - …header, recursively until a call is [[recursive —]]
  + …header, recursively until a call is {{recursive, and}}
  + …see into that receives a captured {{list,}}
  - …that receives a captured `MutSpan` or [[column]]
  + …that receives a captured `MutSpan` or {{column, by `ref mut` or by a name that may change it,}}

### PAR-3 (p11-concurrency.md; 96 words changed)
  + …`[PAR-3]` `@parallel(reduce=[total: +, best: max])` declares {{the}}
  - …`@parallel(reduce=[total: +, best: max])` declares variables [[combined after]]
  - …max])` declares variables combined after the [[loop; inside]]
  + …max])` declares variables combined after the {{loop's iterations combine, whole numbers included, unlike a scoped task's (`[THR-21]`): the declaration is what `[SIMD-5]` reads for a float total. Inside}}
  - …body each chunk has its own [[copy.]]
  + …body each chunk has its own {{accumulator, which starts and is combined as a task's does (`[THR-21]`).}}
  - …the serial left-to-right sum, which is [[why a float]]
  + …the serial left-to-right sum, which is {{one reason every}}
  - …which is why a float reduction [[must be]]
  + …which is why a float reduction {{is}}
  + …a float reduction must be declared. {{A whole-number reduction's chunk results have the variable's type and stop the program on overflow, as `[THR-21]`'s task totals do (verdict 105): the result is the serial one whenever no chunk's own total overflows, and since chunk boundaries depend only on the trip count and `chunk`, whether it stops does not depend on the machine.}}

### JOB-5 (p11-concurrency.md; 255 words changed)
  - …* `[JOB-5]` `jobs.local_arena()` [[returns an]]
  + …* `[JOB-5]` `jobs.local_arena()` {{gives a `ref mut ThreadArena`, the}}
  - …* `[JOB-5]` `jobs.local_arena()` returns an arena [[view]]
  + …* `[JOB-5]` `jobs.local_arena()` returns an arena {{of the job the calling code runs in. Its region is the function the job runs, as a local of that function has: a value made in it that would leave that function is `E3063` (`[LT-4]`). **The arena is the job's own.** When a job starts, the runtime opens a scope of its worker thread's arena}}
  - …`jobs.local_arena()` returns an arena view for [[the current job, reset]]
  + …`jobs.local_arena()` returns an arena view for {{it (`[ARN-6]`) and ends that scope}}
  - …current job, reset when the job [[ends; allocations from]]
  + …current job, reset when the job {{ends, so the arena is reset by giving back the job's own bytes and no others: a job that runs inside another job's wait on the same thread (a chunk the calling thread runs itself, `[PAR-4]`; a job a waiting worker takes up) allocates above the waiting job's bytes and gives back only its own, and every view the waiting job holds stays valid. **It is one place for each job:** every call of `jobs.local_arena()` in one job names the same arena, so a second call while the first's result is still used is a second `ref mut` name of one place (`E3022`: "`b` would be a second name for this job's arena, which `a` (line 3) may change until line 9; use `a`, or give}}
  - …the job ends; allocations from it [[cannot escape]]
  + …the job ends; allocations from it {{on: `fill(ref mut a)`"), and a function that calls it is recorded as changing its job's arena (`[BCK-8]`), so a call of such a function while}}
  - …allocations from it cannot escape the [[job.]]
  + …allocations from it cannot escape the {{job's own name for the arena is still used is refused the same way. A call of it outside a job stops the program ("`jobs.local_arena()` on line 7: `tick` is not running as a job").}}

### SOA-1 (p12-dod.md; 121 words changed)
  + …a struct field is one column. {{A flattened field has no column of its own: `ps.pos` is `E2072` ("`pos` is `@soa(flatten)`, so `ps` has no `pos` column; name one of its columns: `ps.pos.x`, `ps.pos.y`, `ps.pos.z`"), while `ps[i].pos` names the element's `pos` over those columns as `ps[i]` names the element over all of them (`[SOA-6]`): a proxy, read-only through `ps[i]` and changeable through `ref mut ps[i]` or a name that may change `ps`, whose field `ps[i].pos.x` is a place in the column `pos.x`; `copy of ps[i].pos` gathers a `Vec3` of your own from them (`load()` at a use is `E1010`, as for `ps[i]`); `ps[i].pos = v` scatters `v`'s fields into them, as `ps[i] = v` scatters an element, and `ps[i].pos += d` is a proxy's compound assignment (`[SOA-6]`).}}

### SIMD-5 (p12-dod.md; 14 words changed)
  - …the body is to a function [[`[CG-C-3]` places in the inline header,]]
  + …the body is to a function {{that meets `[CG-C-3]`'s criteria,}}
  + …header, read through as `[PAR-2b]` says, {{whether or not this build inlines it and}}

### SIMD-7 (p12-dod.md; 144 words changed)
  + …`Sync`, `Io`, `FFI`, `Unsafe` and `Block`, {{and no operation on an `Atomic` (a relaxed store is not `Sync`, and another thread could read what it stored before the process aborts),}}
  + …location of the first overflowing iteration. {{The group ends, and its accumulated bits are tested, before any instruction leaves the body's straight path: before a `return`, `break` or `yield`, before a panic of any other kind (a `Bounds` check, an explicit `panic`, an allocation's capacity check), and before a call not inlined into the group; so no line outside the group runs on a wrapped value, and no panic names a wrapped value's consequence as its cause. A test placed at such an instruction is the group's end for the lanes before it.}}
  + …overflowing iteration. This is not observable: {{a wrapped value reaches no memory another thread or process reads (the body has none of}}
  - …iteration. This is not observable: the [[body's writes go only to memory this thread owns,]]
  + …iteration. This is not observable: the {{effects above), leaves the group by no path before the test,}}
  - …owns, and the process aborts before [[anything]]
  + …owns, and the process aborts before {{any line outside the group}}
  - …the process aborts before anything reads [[them.]]
  + …the process aborts before anything reads {{it.}}

### NEW ECS-4a (p12-dod.md)
* `[ECS-4a]` The component types listed in one `view`, `view_mut`, `all_of`, `any_of`, `group` or `exclude` are distinct, judged after generic parameters are filled in (`[TYP-17]`, at each use): a type listed twice, or listed and excluded in one view, is `E2074`, naming the type and the line to write ("`view_mut[Position, Position]` (line 9) names `Position` twice; a view gives each component of an entity once, so two of its items would name one place: write `view_mut[Position]`"). A generic function whose list becomes a repeat for one use is refused at that use, naming it ("in `step` used with `A = B = Position`"). With the types distinct, the pools a view takes `ref mut` are different elements of `pools` (`[ECS-2]`), which the library's code shows by their keys (`[BRW-5]`).

### ERR-1 (p13-errors.md; 7 words changed)
  + …`[ERR-8]`), and converting one into `AnyError` {{takes any `E:}}
  - …and converting one into `AnyError` needs [[`E: Error`.]]
  + …and converting one into `AnyError` needs {{(Error, Send, ThreadShared)` (`[ERR-8]`).}}

### ERR-10 (p13-errors.md; 2 words changed)
  - …if any, displays as `msg: <original>`; [[`r.with_context(fn()]]
  + …if any, displays as `msg: <original>`; {{`r.with_context(owns fn()}}

### ERR-12 (p13-errors.md; 1 words changed)
  + …or each `source` of an `E: {{needs}}

### CT-1 (p14-comptime.md; 27 words changed)
  - …`RuntimeCheck` — no `Io`, `FFI`, `Sync`, [[`Block`, `Unsafe`]]
  + …`RuntimeCheck` — no `Io`, `FFI`, `Sync`, {{`Block`}}
  - …`Io`, `FFI`, `Sync`, `Block`, `Unsafe` or [[`Nondet`.]]
  + …`Io`, `FFI`, `Sync`, `Block`, `Unsafe` or {{`Nondet`; `unsafe` code runs there, each of its pointer operations checked by the evaluator as `[CT-2]` says, since the evaluator knows every value it holds.}}

### NEW CT-1a (p14-comptime.md)
* `[CT-1a]` A function run at compile time reaches no module variable that lives at run time: a read or a change of a `static mut`, or a read of a `static` whose initialiser runs on first access (`[STA-3]`, `[CT-5]`), by the function or by anything it calls (its summary, `[EFF-24]`), is `E6010` naming the static and the call chain ("`fresh` changes `NEXT_ID`, a `static mut`, and a value made when compiling cannot see what the program will hold while running: `comptime(fresh())` (line 8) → `fresh` (line 3) → `NEXT_ID += 1` (line 4); pass the number in: `fn fresh(id: int) -> int`"). A `static` initialised at compile time (`[STA-3]`, placed in the image) is read as the constant it is. So every compile-time result is a function of its inputs alone, which is what `[CT-4]`'s cache assumes.

### CT-2 (p14-comptime.md; 387 words changed)
  + …`extern` call, a thread, a clock, {{inline assembly (`[UNS-6]`), I/O, and, in}}
  + …call, a thread, a clock, `unsafe` {{code, a read or write through a}}
  - …a thread, a clock, `unsafe` pointer [[access outside]]
  + …a thread, a clock, `unsafe` pointer {{that does not point at}}
  - …clock, `unsafe` pointer access outside the [[evaluator's heap, I/O]]
  + …clock, `unsafe` pointer access outside the {{bytes of a value the evaluator holds and has not ended (a pointer made from an integer, one past a value's end, one into a value already ended, a foreign address), the evaluator keeping the extent and the life of every value it makes}}
  + …I/O — is `E6010`, naming the {{operation and the}}
  - …— is `E6010`, naming the call [[chain.]]
  + …— is `E6010`, naming the call {{chain ("`comptime(crc_table())` (line 15) → `crc_table` (line 7) → `p.add(i).write(…)` (line 12) writes past the end of `t` when `i` is 256; the table has 256 entries: write `for i in 0..256:`").}}
  - …are provided: `comptime.read_file(path)` (relative to the [[package root, and only inside]]
  + …are provided: `comptime.read_file(path)` (relative to the {{root of}}
  - …root, and only inside the package [[directory,]]
  + …root, and only inside the package {{whose source holds the `comptime.read_file` call, the folder of that package's `ember.toml`, whatever package's item is being evaluated and whatever package is built; the path is resolved with every symbolic link followed and must then lie under that root,}}
  - …dependency's build cannot read the builder's [[files)]]
  + …dependency's build cannot read the builder's {{files: a dependency's function that reads a file for its caller reads the dependency's own files,}}
  - …cannot read the builder's files) and [[`comptime.env(name)`.]]
  + …cannot read the builder's files) and {{a file of the root package is read by a `read_file` written in the root package and given to the dependency as text) and `comptime.env(name)`, which a call written in the root package's source may make for any variable and one written in a dependency's only for a variable the root package's manifest lists for it (`[comptime] env = ["CC", "TARGET_OS"]`, by default none; `E6010` otherwise), so a dependency cannot read the builder's environment into the program it builds. `comptime.read_file(path) -> StrView` is the file's text (`E6010` when the file is missing, unreadable or not UTF-8, naming the path and the call chain), and `comptime.env(name) -> Option[StrView]` the variable's value, `None` when it is unset; each result is kept in the image as constant data with the `static` region (`[CT-5]`, `[LT-3]`). Both run only while compiling, wherever they are written: the call is evaluated as a `comptime(e)` is, so its argument is a compile-time value (`E6005` otherwise: "`comptime.read_file(name)` (line 2) reads a file while compiling, and `name` is known only while running; give it the path as text: `comptime.read_file(\"shaders/sky.wgsl\")`"), the read is the compiler's and not the program's, so neither carries `Io`, and a function that calls one runs at run time with what was read when it was compiled (`sky_shader()` gives the same view on every path, and no file is opened while the program runs), as C's `#embed`.}}

### CT-3 (p14-comptime.md; 49 words changed)
  - …Each evaluation is limited to 10⁸ [[steps]]
  + …Each evaluation is limited to 10⁸ {{steps, a step being one operation of the source the evaluator performs (a call, an operator, an index, a field read, a loop's test or a statement; a built-in copy or compare of n bytes counts ⌈n / 8⌉ steps), so the count is the same on every implementation,}}

### STD-27 (p15-stdlib.md; 17 words changed)
  + …Number` block stating its `type Real: {{needs}}
  + …into one. `[STD-21]`'s functions take `T: {{needs}}
  + …type is `E2042`. A bound `T: {{needs}}
  - …`round_half_away` its `round`, `mul_add` its `fma`); [[the transcendental ones]]
  + …`round_half_away` its `round`, `mul_add` its `fma`); {{those of them whose result IEEE 754 does not fix}}
  - …the transcendental ones are `[DET-2]`'s platform [[functions.]]
  + …the transcendental ones are `[DET-2]`'s platform {{functions, and carry `Nondet`.}}

### FFI-2 (p16-ffi.md; 716 words changed)
  - …an overlay (§XVI.4), makes the call [[safe.]]
  + …an overlay (§XVI.4), makes the call {{safe, within the limit on raw pointers below.}}
  + …overlay is an `unsafe overlay` (`[GRM-35]`). {{An entry written `unsafe fn` asserts nothing of the kind, nor does a hand declaration written so (verdict 133): the function stays unsafe to call, and its `@safety` says what its callers must make true (`[UNS-7]`, verdict 130).}}
  + …overlay line that supplies it (`[DIA-18]`). {{**The limit on raw pointers.** Safe code never hands foreign code an address it made up. No assertion of the fact — `safe fn` (`[FFI-10]`), an overlay function entry not written `unsafe fn` (`[GRM-35]`), `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) or `@ffi(std_function, safe)` on a `std::function` type (`[FFI-17a]`) — covers a value the caller supplies that names memory or code nothing checks: a raw pointer (`*T`, `*mut T`, an `ObjectPtr[C]`, `[FFI-54]`) or a C function pointer (`unsafe extern "C" fn(…)`, `[FFI-56]`), alone or inside an `Option`, that the call carries to foreign code: an argument itself; one held, at any depth, by what an argument passes, by value or behind a contracted pointer (`one`, `count(n)`, `fixed(N)`, `exclusive`, `owns`, `nullable`), a C++ method's object included, read from the argument's Ember type as it is declared, its fields, a union's fields, its elements, its type arguments (`[T; N]`, `Span[T]`, `MutSpan[T]`, `Option[T]`, `CppVector[T]`, `Volatile[T]`) and the representation of a `@ffi(newtype)` (`[FFI-8]`); and the result, itself or at any depth in it, of a safe callable type among them (a `callback(…)` parameter's `fn(P…) -> R`, `[FFI-21]`; a field of a safe `extern "C" fn` type, which only an Ember function fills, `[FFI-56]`), since that is what an Ember function given to the call returns to foreign code, its parameters being what foreign code hands the function. A type that declares no field holds none: an `extern type`, an opaque class, a `@ffi(handle)` type (`[FFI-8]`), `CppString`; a `Span`, a `MutSpan` and a `CStrView` are checked views, not raw pointers, and only a span's elements are read. Safe code can make such a value null (`null[P]()`, `None`), pointing at fewer elements than a count beside it says or at a value already ended (`ref_to_ptr`, `as_mut_ptr()`, §IV.5, `[SPN-8]`), or holding any address (a pointer-cast constant, `[FFI-6a]`; a function pointer C handed back, whose target nothing describes), and no contract reaches it: words attach to parameters and results only (`[FFI-1]`, `[FFI-11]`), no overlay touches a field (`[FFI-30a]`), and a `safe fn`'s own function-pointer parameter, a safe `fn(P…) -> R`, takes no such value (`[FFI-21]`, `[FN-6]`). So an assertion of the fact for a function whose calls the other rules make safe, and whose calls would carry one, is `E5002` where it is written, naming where the value is held: "`clear_pixels` is declared `safe fn`, but `buf.data` is a raw pointer (`*mut u8`, line 3) that its call hands C, and safe code can put any address in it and any count in `buf.len` (line 12); write `@safety(\"…\")` above `unsafe fn clear_pixels(buf: PixelBuf)`, saying what `buf.data` must point to, and put line 12 in an `unsafe:` block with a `# SAFETY:` note, or give safe callers a wrapper that fills `PixelBuf` from a `MutSpan[u8]` inside `unsafe:`". The lines that leave the calls unsafe are `@safety("…")` above the declaration or entry written `unsafe fn` (a hand declaration keeping its C ABI parameters, `[FFI-10]`), or the typedef's or type's entry without `safe`; each call then goes in an `unsafe:` block with a `# SAFETY:` note (`[UNS-1]`), or behind a wrapper in an `extend` block (`[FFI-13]`) that fills the value from a checked view inside `unsafe:`; for an entry in another package's overlay, the message gives that entry as an `override` in the importing package's own overlay (`[FFI-30c]`). A message elsewhere that offers an assertion of the fact (`safe fn lib_step`, a `callback(…)` word, `@ffi(safe)`) offers it only where this limit allows it (`[DIA-25]`). An entry whose own words already leave its calls unsafe (an unknown fact, a `TODO(…)` word, `retained` on a plain pointer; `[FFI-11]`, `[FFI-11c]`) is as it was. A value foreign code hands Ember (a result, an `out` value, a callback's parameter, a field of what it returns) is outside this limit: reading or writing through it is unsafe code (`[UNS-1]`), and `[FFI-10]` decides a hand declaration's raw-pointer result. The limit is read from the declaration and the header's types, which every package importing the header sees alike (`[FFI-30a]`, `[BLD-16]`), and nothing runs for it while the program runs.}}

### FFI-49 (p16-ffi.md; 10 words changed)
  - …symbol. Libraries are linked by `[link] [[libs]]
  + …symbol. Libraries are linked by `[link] {{unsafe_libs}}
  - …libs = ["m", "vulkan-1"]` in the [[manifest or `link = [...]` on an `import c`]]
  + …libs = ["m", "vulkan-1"]` in the {{manifest, and by nothing else}}

### FFI-6 (p16-ffi.md; 33 words changed)
  - …(`c`, or `name`). Options: `include_paths`, `defines`, [[`link`,]]
  - …`implementation` (`[FFI-29b]`). An unknown option is [[`E0104`.]]
  + …`implementation` (`[FFI-29b]`). An unknown option is {{`E0104`; for `link`, which named a library before 0.9.20_Hardened_8, the message says where libraries go: "`link` is not an option of `import c`; name the library under `[link] unsafe_libs` in `ember.toml`" (`[BLD-FFI-3]`).}}

### FFI-20a (p16-ffi.md; 14 words changed)
  - …and a workaround, and reported as [[`W5002`.]]
  + …and a workaround, and reported as {{`W5002`; for a C++ header the importer could not parse, each declaration is `W5031`.}}

### FFI-11a (p16-ffi.md; 27 words changed)
  + …ending `Count`, `Len`, `Size` or `N`). {{A result whose C++ type carries its count (a span, a fixed-extent span or `std::string_view`, `[FFI-17a]`) takes its lifetime word alone (`-> from(this)`), with no count word.}}

### FFI-12 (p16-ffi.md; 139 words changed)
  - …whose name, parameter count or parameter [[names]]
  + …whose name, parameter count or parameter {{names, or the words and types it writes (`[GRM-35]`),}}
  + …or parameter names do not match {{one declaration of}}
  - …parameter names do not match the [[header]]
  + …parameter names do not match the {{header, or that fits none or several members of a name declared several times (`[FFI-57]`),}}
  - …header is `E5010`, so overlays cannot [[drift.]]
  + …header is `E5010`, so overlays cannot {{drift and no entry vouches for a function its writer did not name; for several, the message gives each member's entry line with the type that tells them apart: "`fn set(name: borrowed nul_terminated, value)` fits `void set(const char*, int)` and `void set(const char*, float)` of `gl::Shader`; write one entry per member: `fn set(name: borrowed nul_terminated, value: c_int)` and `fn set(name: borrowed nul_terminated, value: f32)`". A `rename` or a `hide` with a parameter list is matched as an entry is. So is an entry outside a class section that names only members of classes (verdict 131): "`new` is a member of `Handle` and of `MutHandle`: write it under `class Handle:` or `class MutHandle:`".}}

### FFI-35a (p16-ffi.md; 146 words changed)
  - …lists every such assumption per module, [[and]]
  + …per module, and every result-position pointer {{or C++ reference}}
  - …every result-position pointer with no lifetime [[word.]]
  + …every result-position pointer with no lifetime {{word, and each `from(static)` and `from(p)` result and each `out` parameter as the assertion it is (`[FFI-11]`). A class object given to a `retained` parameter is marked in its header as kept by C, the top bit of its type index (`[OBJ-1]`): the mark is set by the call, one store, by the wrapper of a function with a parameter contracted `retained` whose C type is a pointer to a class type (`*mut Names`), through the pointer before the call, since a class object's address is its header (`[OBJ-2]`), and that the pointer is a live object's is the unsafe caller's obligation (`[UNS-4]`), as every raw pointer's is; a `retained` pointer to anything else marks nothing. A marked object never moves: handed to another thread while a `weakref` link on this thread points at it, it stops the program (`[THR-2]`, verdict 114).}}

### FFI-27 (p16-ffi.md; 15 words changed)
  + …on each thread that calls Ember, {{`ember_rt_object_release(void*)` to end an object Ember gave it with `into_raw()`, on the thread `[FFI-54]` allows,}}

### FFI-28 (p16-ffi.md; 16 words changed)
  - …`ember_module_protocol_vN`, `N` the module protocol's version [[(`[ABI-1]`).]]
  + …`ember_module_protocol_vN`, `N` the module protocol's version {{(`[ABI-1]`), and the functions its header declares for the `thread_shared` class handles its exports give (`[BLD-FFI-5a]`).}}

### BLD-FFI-1a (p16-ffi.md; 8 words changed)
  - …shim compiled with the package's C [[flags]]
  + …shim compiled with the package's C {{flags, as `[MAN-10]` admits them for a dependency,}}

### CLI-4 (p17-toolchain.md; 20 words changed)
  - …accept a single file with no [[manifest, as]]
  + …accept a single file with no {{manifest: the first compile writes}}
  + …file with no manifest, as a {{default one beside it, which names the}}
  - …with no manifest, as a package [[named]]
  - …as a package named after the [[file with default settings.]]
  + …as a package named after the {{file's folder (`[MAN-9]`).}}
  + …A first program needs no manifest {{written by hand}}

### CLI-21 (p17-toolchain.md; 33 words changed)
  - …console and file I/O work; foreign [[calls]]
  + …console and file I/O work; foreign {{calls, `thread.spawn`, `thread.scope`, `jobs.submit`}}
  - …file I/O work; foreign calls and [[threads]]
  + …file I/O work; foreign calls and {{`jobs.scope`}}
  + …calls and threads are unavailable (`E6010`); {{a `@parallel` loop runs on the one thread, chunk by chunk in chunk order, as `[PAR-4]`'s calling thread may in any run, and gives the same bits (`[PAR-5]`);}}

### CLI-5 (p17-toolchain.md; 75 words changed)
  - …declaration as a comment with its [[reason.]]
  + …declaration as a comment with its {{reason, and each member of a name declared several times as its own entry, with the words and types that tell it from the others and no more (`[FFI-57]`, `[GRM-35]`).}}
  - …existing overlay, keeping hand-written contracts and [[comments.]]
  + …existing overlay, keeping hand-written contracts and {{comments, and records the header as it now is in that overlay's line of the package's `ember.overlays` (`[FFI-55]`). A declaration it leaves unsafe to call, one whose calls `[FFI-2]`'s limit on raw pointers keeps from being safe among them, is written `unsafe fn` (verdict 130).}}

### CLI-7 (p17-toolchain.md; 38 words changed)
  + …* `[CLI-7]` `ember bind {{<header>}}
  + …are safe, still `unsafe`, and skipped. {{With `--merge`, once the overlay passes, it writes the header's declarations as they are now into the overlay's line of `ember.overlays` (`[FFI-55]`): with `--init --merge` (`[CLI-5]`), the only ways a line of the record changes once written.}}

### MAN-2 (p17-toolchain.md; 22 words changed)
  + …`--locked` fails if it would change. {{Beside it, `ember.overlays` records the headers the package's overlays were checked against (`[FFI-55]`); Ember writes both, and neither is edited by hand.}}

### NEW MAN-9 (p17-toolchain.md)
* `[MAN-9]` **Every program is a package** (verdict 127). A command that compiles a file (`ember build`, `check`, `run`, `test`, `lint`) with no `ember.toml` in its folder or any folder above it first writes a default one in the file's folder, `[package]` with the folder's name as `name` and `version = "0.1.0"`, and says so on standard error: "note: created `tools/ember.toml`: every program needs one, and this folder had none" (verdict 135); `ember bind` does so in a header's folder no package holds. The program is then that package, and `ember.overlays` sits beside its manifest (`[FFI-55]`), so the record of what each overlay was checked against is kept for every program, a single file's included, and no program is compiled apart from the others with settings no file shows. (0.9.20_Hardened_2 to 0.9.20_Hardened_4 wrote no file and kept no record for such a program; the owner restored verdicts 127 and 135 on 2026-11-10, §H.56.)

### NEW MAN-10 (p17-toolchain.md)
* `[MAN-10]` **A dependency's manifest decides only its own code, never what runs on the builder's machine.** Of a dependency's manifest the build reads `[package]`, `[dependencies]`, `[lints]`, `[realtime]` and `[ffi]`. Its `[c]` section is taken with `compiler` ignored (the root's, or the backend's, `[BLD-FFI-1]`) and `flags` limited to `-D<name>[=<value>]`, `-U<name>`, `-I<path>` for a path that, with every symbolic link followed, lies under the dependency's own root (`[CT-2]`), `-std=<c-standard>`, `-O<n>`, and `-W<name>` other than `-Wl,…`, `-Wa,…` and `-Wp,…`, which pass arguments to the linker, assembler and preprocessor; its `[link] unsafe_libs` entries, and those of its `[link.windows]`, `[link.linux]` and `[link.macos]` sections, are taken for names with no path separator (`"m"`) and for paths under its own root, resolved the same way (a library one of its headers asks for is named there too, `[FFI-51]`); and its `[cpp.<project>]` section (whose configuration runs the project's own build code, Annex C), any other `[c] flags` entry, any other `[link]` path, and its `[build]`, `[profiles]`, `[jobs]` and `[comptime]` sections are honoured only when the root package's manifest names the dependency in `[build] native = ["parser"]`, and are `E9001` otherwise, naming the dependency, the key and the line to add ("`parser`'s manifest sets `[c] flags = [\"-fplugin=./vendor/helper.so\"]`, which would run its code inside this build's C compiler; a dependency's flags are limited to defines, include paths under its root, a C standard, an optimisation level and warnings; to take the rest as written, add `native = [\"parser\"]` under `[build]` in `ember.toml`"). What a dependency's C is compiled from is limited the same way, since a header names any file of the machine itself, whatever its `-I` paths (`#include "/home/ci/.ssh/config"`, C23's `#embed`): the compile of a dependency's shim, thunk unit or C source reads only files under the dependency's own root and the toolchain's headers, which the build checks against the files the C compiler reports it read, included and embedded alike (a build that cannot learn what a compiler embedded refuses `#embed` there). Each such file is judged by where it really is, not by the name it was reached by: the build opens it and asks the system for its real path, every symbolic link, junction and `..` resolved and, on Windows, the case and short names made canonical, and that path lies under the real path of the dependency's root or of a toolchain folder; a file under the dependency's root that has more than one hard link is refused too, since its real path names only the copy inside (a package cache copies files and never hard-links them). The toolchain's headers are in the folders the C compiler searches for `#include <…>` when it is given no flag from any manifest and none of the environment variables that add folders (`INCLUDE`, `CPATH`, `C_INCLUDE_PATH`, `CPLUS_INCLUDE_PATH`, `OBJC_INCLUDE_PATH`), taken as the compiler lists them itself (`-E -v`; for MSVC's `cl`, the MSVC and Windows SDK installations the toolchain finds, `[TOOL-2]`), each by its real path; the build compiles a dependency's C with those variables removed, and reading a toolchain header is not vouching for it (`[FFI-51]`). That C holds no `asm` statement, at file scope or in a function's body, judged after preprocessing, since an assembler directive reads a file no report names (`.incbin`) and a macro of the assembler builds one out of pieces no search of the text finds. And where the C compiler writes assembly text and runs an assembler on it (GCC), the build reads that text before it is assembled and refuses it when it holds, outside a quoted name, a directive that reads a file or makes directives (`.incbin`, `.include`, `.macro`, `.irp`, `.irpc`, `.rept`, `.altmacro`): GCC writes none of these itself, so one there came from a string the source gave it (a label, a section name, a `symver`, a `weakref` target, an `#ident` line), while Clang writes such strings quoted and needs no such read. A file from anywhere else, or assembly, is `E9001` unless the root names the dependency in `native` ("`parser`'s `lexer.h` (line 3) reads `/home/ci/.aws/credentials` into the program with `#embed`; a dependency's C reads only its own folder and the toolchain's headers; to allow it, add `native = [\"parser\"]` under `[build]` in `ember.toml`"). Each flag reaches the C compiler as one argument of its own, never through a shell, so a define's value cannot add a second flag. `ember tcb` lists every dependency so named as an external library (`[TCB-4]`).

### MAN-8 (p17-toolchain.md; 22 words changed)
  - …`backend`, `c_compiler`, `runtime`, `max_instantiations` `[MONO-3]`, `reload` [[`[MAN-7]`);]]
  + …`backend`, `c_compiler`, `runtime`, `max_instantiations` `[MONO-3]`, `reload` {{`[MAN-7]`, `native` `[MAN-10]`);}}
  - …(`[PRF-3]`); `[c]` (`compiler`, `flags`, `[BLD-FFI-1]`); `[link]` [[(`libs`);]]
  + …(`[PRF-3]`); `[c]` (`compiler`, `flags`, `[BLD-FFI-1]`); `[link]` {{(`unsafe_libs`, `timeout` `[BLD-15]`), and `[link.windows]`, `[link.linux]` and `[link.macos]` (`unsafe_libs`, for a build for that system, `[BLD-FFI-3]`);}}
  - …`[realtime]` (`contracts`, `[EFF-19]`); `[comptime]` (`max_steps`, `max_heap_mb`, [[`[CT-3]`);]]
  + …`[realtime]` (`contracts`, `[EFF-19]`); `[comptime]` (`max_steps`, `max_heap_mb`, {{`[CT-3]`; `env`, `[CT-2]`);}}

### NEW BLD-16 (p17-toolchain.md)
* `[BLD-16]` **No fact of the whole program decides a package's code or layout.** The layout of a package's types and the instructions it emits for a line are decided by the package's own source, by the interface files of the packages it imports (`[MOD-9]`, `[EFF-4]`, `[BLD-14]`) and by the generic instances it makes (`[MONO-1]`); never by a package that depends on it, and never by the program as a whole, which the package never sees (`[BLD-1]`): not by which lines, anywhere, write a word on a value of one of its types, not by which of its values are made `thread_shared`, not by which of its classes a `weakref` names, not by which classes a dependant derives from one of its `open` or `abstract` classes (`[THR-8]`). A rule whose condition would be such a fact states it so that the package compiling the line reads it from a declaration it sees (a type's words, its `pub`, whether it is `Send`: `[OBJ-1]`, `[WK-18]`), from the type of the place at the line (`[THR-2]`, `[OWN-13]`), or, where the fact is one of a kind known only while the program runs, from the kind's own table (`[LT-48]`, `[THR-2]`); where none of these holds, the instruction is emitted and the test is paid. With the whole program visible, the final link may remove only what every program would let it remove (a virtual call with one reachable implementation, `[DSP-5]`): it changes no layout and no meaning (`[BLD-14]`). So a package compiled once lays out its objects, and tests what it tests, the same in every program that uses it, in `ember check` of the package alone, in its tests, in a hot-reload image and in a module loaded at run time (`[ABI-1]`, `[VER-4]`). A library update that changes such a fact can make a line of a program that uses it stop compiling; the message names the chain of calls (`[DIA-27]`). The layout of each type is recorded with it: whether its values carry the weak count and whether they may own values that do (`[OBJ-1]`), decided by the package that declares the type and the same in every program. A parent link proves itself only when no other package can change it or the places that own its holders. What a generic function changes through its type parameters' methods is worked out for each instance, whose body the instantiating package compiles.

### NEW BLD-15 (p17-toolchain.md)
* `[BLD-15]` **A link that hangs is an error** (verdict 124). Each link the build runs has a time limit, 30 minutes unless `[link] timeout` sets another number of seconds. A link still running at its limit is ended, and the build fails with `E9042`, naming the linker, its command and how long it ran, and keeping the link's command file for whoever looks into it. A link that was ended leaves no program behind and none in the program cache, and nothing is tried again by itself.

### DIA-6a (p17-toolchain.md; 31 words changed)
  - …(§XVII.9). Each code is defined by [[exactly one rule.]]
  + …(§XVII.9). Each code is defined by {{the first rule its registry row names, which owns its page (`[DIA-6]`); the other rules the row names use it, and each of them has a reject case for it (`[TST-4b]`).}}

### DIA-18 (p17-toolchain.md; 11 words changed)
  - …* `[DIA-18]` A foreign [[call]]
  + …* `[DIA-18]` A foreign {{declaration}}
  - …because its contract has unknown facts [[(`E5002`)]]
  + …because its contract has unknown facts {{(`E5002`, `[FFI-2]`; a call of it outside `unsafe` is `E3100`)}}

### CG-C-3a (p18-implementation.md; 72 words changed)
  + …to the C compiler as hints. {{`@inline` on a function that may call itself through direct calls alone (its own body, or the bodies of the functions it calls by name, the libraries' inline bodies included; a call through a function value, an interface, `dyn` or `virtual` method breaks the cycle) is `E0104`, naming the cycle ("`depth` calls itself on line 4, and an `@inline` body cannot be inlined into itself; remove `@inline` (line 1), or write `@hot`").}}

### RT-1 (p18-implementation.md; 84 words changed)
  + …may supply its own allocator (`[FFI-27]`). {{Its libraries, the runtime's own (`[BLD-FFI-3]`), are the C runtime the profile chooses with the libraries that runtime asks for itself, the C compiler's own support library (`libgcc` with GCC, compiler-rt's builtins with Clang) and the system libraries `ember_rt` calls: on Windows `libcmt`, `msvcrt` or a debug form of either, which brings its `ucrt`, `vcruntime` and `kernel32`, with `oldnames`, and `synchronization` for `WaitOnAddress`; on Linux `libc`, `libm` and `libpthread`; on macOS `libSystem`. A library outside this list is the program's, named in its manifest.}}

### RT-3 (p18-implementation.md; 24 words changed)
  - …* `[RT-3]` `TypeInfo` holds size, alignment, [[flags,]]
  + …* `[RT-3]` `TypeInfo` holds size, alignment, {{flags (among them whether the type's values carry the weak count and whether they may own values that do, `[OBJ-1]`),}}
  + …alignment, flags, name, base, drop functions, {{the hand-over routine (`[THR-2]`),}}

### CG-LL-1 (p18-implementation.md; 36 words changed)
  - …C backend came first; the LLVM [[backend is]]
  + …C backend came first; the LLVM {{backend,}}
  - …LLVM backend is built beside it, [[and becomes the default once it passes]]
  + …LLVM backend is built beside it, {{passed}}
  - …passes the conformance and the performance [[suites; until it exists, `--backend llvm`]]
  + …passes the conformance and the performance {{suites}}
  - …until it exists, `--backend llvm` and [[`backend = "llvm"` are refused as not yet implemented.]]
  + …until it exists, `--backend llvm` and {{is the default.}}
  - …yet implemented. The C backend stays [[supported afterwards:]]
  + …yet implemented. The C backend stays {{supported:}}
  - …every platform that has a C [[compiler.]]
  + …every platform that has a C {{compiler, and `--backend c` or `backend = "c"` chooses it.}}

### NEW HR-44a (p20-annex-b.md)
* `[HR-44a]` **A stored function is a fact too.** Plan enumerates (`[HR-15]`) every function value held in a live instance, a registered container, a static or a lambda's state, and every callback C keeps (`[HR-20]`), and compares each, by its code word, which `[HR-6]` makes the thunk, one pointer compare per held value, with the functions the new image's compile counted for that place: `[EFF-2]`'s set from the new source and `[BLD-14]` item 7's from its dependencies' interface files, and for C's keeping `[FFI-52]`'s. Where a held function is not among them, the new image's checks of that place did not count it: the reload is refused (`E9030`), naming the place, the function and the count ("`tree.on_visit` holds `prune` in 1 instance, which the new `setup` no longer stores there, and the new `walk` (line 8) walks `nodes` across its call; keep `prune` among what `setup` may store, or restart"). A function value the runtime cannot enumerate is treated as held. Nothing of this runs while the program runs.

### HR-42 (p20-annex-b.md; 112 words changed)
  - …code increments the thread's depth counter [[with acquire ordering]]
  - …depth counter with acquire ordering and [[leaving]]
  + …depth counter with acquire ordering and {{then reads the runtime's commit mark, both sequentially consistent (one full fence between them where the hardware needs it); a thread that reads the mark set}}
  - …with acquire ordering and leaving decrements [[it with release.]]
  + …with acquire ordering and leaving decrements {{its counter, waits until the mark is cleared, and enters again. Leaving Ember code is a release decrement.}}
  + …leaving decrements it with release. Commit {{sets the commit mark and then}}
  + …it with release. Commit reads every {{other registered thread's counter, with the same ordering, so that of a thread entering and a commit starting at once at least one sees the other; a}}
  - …with release. Commit reads every counter [[with acquire,]]
  + …with release. Commit reads every counter {{that is not zero clears the mark and the poll returns `EMBER_RELOAD_UNSAFE_POINT`. With every counter zero, commit}}
  - …every counter with acquire, writes the [[new thunk targets]]
  + …every counter with acquire, writes the {{instances, the type information}}
  - …writes the new thunk targets and [[type information, and]]
  + …writes the new thunk targets and {{the call table,}}
  - …them with one release store to [[a]]
  + …them with one release store to {{the}}
  - …generation counter, which a thread entering [[Ember code]]
  - …a thread entering Ember code next [[acquires.]]
  + …a thread entering Ember code next {{acquires, and clears the mark.}}

### HR-42a (p20-annex-b.md; 20 words changed)
  - …lock: entering Ember code is one [[relaxed increment]]
  + …lock: entering Ember code is one {{increment, one fence}}
  - …is one relaxed increment and one [[acquire fence]]
  + …is one relaxed increment and one {{load}}
  + …fence whatever the number of threads; {{only a thread that meets a commit waits, for the length of the commit, and}}

### HR-7 (p20-annex-b.md; 58 words changed)
  + …that panics naming it; a reload {{is refused, naming the function, when its thunk is still named by a function value plan enumerates (`[HR-44a]`), by a method table or drop glue of a type with a live instance, by a callback C keeps (`[HR-20]`) or by an `@export` (`[HR-21]`); one C holds}}
  - …panics naming it; a reload that [[would leave such a thunk reachable is refused.]]
  + …panics naming it; a reload that {{the runtime cannot see runs the stub, which panics naming the function.}}

### HR-12a (p20-annex-b.md; 17 words changed)
  + …enabled, enforced at link time (`E9035`). {{A reload image's new kinds take type indices above the process's when the image is loaded (`[TYP-44a]`).}}

### HR-13 (p20-annex-b.md; 28 words changed)
  - …registers itself with its element type's [[schema.]]
  + …registers itself with its element type's {{schema, as an object registers: with plain stores in its thread's list where the container's own type is not `Send`, ordered by the runtime where it is (`[HR-12]`).}}

### HR-13b (p20-annex-b.md; 146 words changed)
  - …pointers or foreign memory cannot be [[migrated; if]]
  + …pointers or foreign memory cannot be {{migrated. Which types such data may have is judged when the program compiles, from types, as `[HR-47]` judges: a type `T` is **unenumerable** when the program,}}
  + …memory cannot be migrated; if its {{dependencies included, makes a raw pointer `*T` or `*mut T` from a place (`ref_to_ptr`, §IX.4) rather than from an allocation the runtime registers, a `MaybeUninit[T]` outside a registered container, an `into_raw()` of a `T` or of what holds one (`[HR-20]` names those it can), or has a foreign parameter whose contract keeps a `T` or what holds one; the image's manifest (`[HR-8]`) lists the unenumerable types with the line that makes each so. Plan refuses a}}
  - …cannot be migrated; if its schema [[changed,]]
  + …cannot be migrated; if its schema {{change of an unenumerable type, naming}}
  - …migrated; if its schema changed, the [[reload is refused.]]
  + …migrated; if its schema changed, the {{type and that line, whether or not such a value lives, since the runtime cannot count them; the same list decides `[HR-45]`'s frames and `[HR-48]`'s states that are treated as live. `ember build --reload --explain` prints the list.}}

### NEW HR-15c (p20-annex-b.md)
* `[HR-15c]` **A view into a part.** A kept view (`[TYP-15]`) that points into a part of a value, a field, an element, a range of a `String` or an array, is rewritten to the part's new place when the part is carried over by name (reordered, its holder resized or moved): the runtime knows what the view points into from the `from` of the field that holds it (`[TYP-15]`), the block of that `let` field, in which the view's old offset names the field, or the element and its field, of the old schema, and maps the old offset to the new through the schema; a view `from static` points into no instance and is never rewritten. Where the part is removed, converted, rebuilt by `migrate_from`, or of a type whose own layout changes, the view has no place to be rewritten to, and plan refuses the reload, naming the field, the type that holds the view and the count, judged from the types whose fields may hold such a view, as `[HR-47]` judges; a view the runtime cannot enumerate (`[HR-15a]`) that may point into such a part refuses it the same way.

### HR-36 (p20-annex-b.md; 53 words changed)
  + …inside `migrate_from` needs no special spelling. {{A value made `using` an instance of a manager that may have several (`[ALC-1]`) takes its memory from that instance, as its type says, never from the transaction arena, and keeps that instance's address; a manager's instance asked for a block and given it back is not an instance changed in `[HR-2]`'s sense.}}

### HR-37 (p20-annex-b.md; 69 words changed)
  + …released whole if the reload fails. {{A reload discarded after prepare first ends every instance prepare built, field by field and without the type's own `drop` (as reclaim ends an old version, `[HR-2a]`), before the arena is released: a count raised through `old` (`shares old.log`) is lowered, and a block taken from a manager's instance (`[HR-36]`) goes back to it, so `[HR-2]`'s "no instance changed" holds; the `drop`s this may run are compiled under `[HR-35]`.}}

### NEW HR-48 (p20-annex-b.md)
* `[HR-48]` **A lambda's state has a schema.** The state of a lambda (`[CLO-10]`) is a value of a generated struct whose fields are its captures, in order, with their words (`[HR-11]`), kept in the state word or in the state block; the lambda's name (`[MNG-1]`: its function, its ordinal and the hash of its text) names the type. It is migrated as any value is: through the places that hold it (`[HR-15]`), by `[HR-14]`'s rows, a capture of a migrated type depth-first, a capture removed ended in reclaim, a capture added refused naming the field (a capture has no default), a word changed refused, since a lambda declares no `migrate_from`; and a body change that leaves the captures as they were applies through the thunk. A state the runtime cannot enumerate (kept by C, `[HR-20]`; behind a raw pointer, `[HR-13b]`) refuses a change of its schema.

### HR-24 (p20-annex-b.md; 71 words changed)
  - …is then refused, naming the resource. [[This]]
  + …is then refused, naming the resource. {{A `Device` registers with the runtime, as a container does (`[HR-13]`), the objects in flight and the frame each waits for, data the device updates when it observes a fence (`[GPU-4]`); plan reads that record and decides it as every refusal}}
  - …refused, naming the resource. This is [[detected]]
  + …refused, naming the resource. This is {{decided (`[HR-15b]`): an object that must move while}}
  - …the resource. This is detected in [[prepare, so]]
  + …the resource. This is detected in {{flight makes the poll return `EMBER_RELOAD_UNSAFE_POINT`, and after `frames_in_flight + 1` such polls}}
  - …in prepare, so the reload is [[discarded and]]
  + …in prepare, so the reload is {{refused with `ReloadError.Timeout` (`[HR-38]`), naming}}
  - …the reload is discarded and the [[old image keeps running]]
  + …the reload is discarded and the {{resource}}

### FFI-17 (p21-annex-c.md; 53 words changed)
  - …per imported function, method, constructor and [[destructor.]]
  + …per imported function, method, constructor and {{destructor, a name declared several times giving one per member and a function with default arguments one per number of arguments it accepts, each passing only the arguments given so that C++ supplies the defaults (`[FFI-57]`); each thunk's `extern "C"` name is derived from its declaration (`[FFI-30]`, `[MNG-1]`), so no two share one.}}

### FFI-17b (p21-annex-c.md; 15 words changed)
  - …generic cannot instantiate a C++ template [[(`E5055`).]]
  + …generic cannot instantiate a C++ template {{(`E5055`); the listed instantiations of one function template are the members of one name (`[FFI-57]`).}}

### FFI-44 (p21-annex-c.md; 19 words changed)
  + …instantiations | automatic (`[FFI-17b]`) | | {{a name declared several times (overloads, default arguments, a `const` and non-`const` pair) | automatic (`[FFI-57]`, `[FFI-40b]`) | |}}

### FFI-17f (p21-annex-c.md; 46 words changed)
  - …`CppVector` `push_back`, `len` and drop, `CppString` [[drop).]]
  + …`CppVector` `push_back`, `len` and drop, `CppString` {{drop, a `std_function` value's drop, the conversion of a `std::optional` or `std::variant` at a call, the move constructor a trampoline runs for a `T&&` parameter of an `override fn` and the destructor of the moved-from object after a call from Ember to a `T&&` parameter (`[FFI-40]`)).}}

### NEW FFI-57 (p21-annex-c.md)
* `[FFI-57]` **A name declared several times.** A C++ name the header declares with several parameter lists — a function of a namespace, a method, a static member function, a constructor (`new`, `[GRM-35]`), the listed instantiations of one function template (`[FFI-17b]`) — imports as one Ember name of several **members**, one per declaration (a redeclared function counting once, `[FFI-30]`), each with its own thunk (`[FFI-17]`), its own entry (`[GRM-35]`), its own `@safety` and `L3015` (`[UNS-7]`, verdicts 125 and 134) and its own line of `ember.overlays` (`[FFI-55]`); a `const` and a non-`const` pair is split into two names first (`[FFI-40b]`). A function with default arguments is such a name too: one member for each number of arguments it accepts, each arity's thunk calling the function with the trailing arguments left out, so that C++ supplies the defaults as at a call of its own (a count that leaves out a parameter another parameter's contract word names, `count(n)`, but not that parameter, is no member). The program declares none of them, so the name is not `E1030` (`[TYP-26]`). Each use of the name stands for one member, read from the header and the use's own words, nothing around it. A call (a method call, `T.new(…)`, `super.init(…)`, `[FFI-39b]`) stands for the member C++ chooses (verdict 175). The importer writes each call of a name with several members as a C++ call in the package's thunk unit, with the arguments' C++ types and words (`owns` an rvalue, `ref mut` a non-const lvalue, none a const lvalue, and an untyped literal as C++ spells its value, `1_000` as `1000`, so that C++ gives it the type it gives that spelling: `int` for `7`, `double` for `2.5`, and for `3000000000` the first of `long` and `long long` that holds it), a parameter with a default left out where the call leaves it out, and Clang's own overload resolution, which the importer already runs to read the header (`[FFI-6]`), names the member: every rule of C++ is applied, its choices between a function and a template, between two templates and between references included, and Ember keeps no copy of C++'s ranking to drift from it. A member Clang reaches only through a conversion Ember does not make (a converting constructor, a user-defined conversion) is not chosen: the call is `E5035`, naming it. So Ember chooses what C++ chooses or refuses, never another: for `n: i8`, `w.set(n)` with `set(short)` and `set(int)` stands for `set(int)`; `w.g(b)` for `b: c_int`, with a plain `g(int)` and a template `g(T)`, stands for the plain one; and with `set(short)` and `set(long long)` the call is ambiguous in C++ and so here (`E5035`: "write `w.set(n as i16)` or `w.set(n as i64)`"). It costs one Clang parse of a package's calls of such names when the package compiles, kept with the build's other results, and nothing while the program runs. A member declared `= delete` takes part and, chosen, is refused. A name used without a call stands for the member whose first line is exactly the callable type expected there (`[CLO-3]`), and with none expected is `E5035`. What the code around the use allows — unsafe code (`[FFI-2]`, `E3100`), a read-only name (`E3023`), its effects, the loop and view rules — is checked after, on the member chosen, so one line stands for one member wherever it is written. A use that fits no member, or that C++ finds ambiguous, is `E5035`, naming each candidate as the header declares it, with Clang's reason why it does or does not fit, and the way out, the argument written with its type or the overlay line: "`w.value(7)` (line 8) fits `void value(long long)` and `void value(double)` of `ui::Widget` alike: `7` is C++'s `int`, which neither takes as it is, and reaches each; write `w.value(7 as i64)` or `w.value(7.0)`". Two members alike in every parameter as the import spells them (two instances of one template differing only in the result), or alike on this target (`put(int)` and `put(long)` where `long` is 32 bits, each taking an `i32`), have one callable type, so a name used without a call cannot be told between them and is `E5035`. A call is decided by Clang as any other is: an `i32` argument, whose C++ type is `int32_t`, and a literal stand for `put(int)`, as in C++, and a call of the two instances fits neither, since C++ cannot infer the template's parameter from it. The overlay tells such members apart by the tables' spelling or by the result, which no target changes (`rename put(v: c_long) as put_long`, `hide put(v: c_long)`, `rename get() -> f64 as get_f64`, `[GRM-35]`), and the message of `E5035` gives that line. An `override fn` matches one virtual member by its first line exactly (`[FFI-39]`), not by overload resolution; a class derived from a C++ class whose header declares one virtual name several times writes one `override fn` per member it replaces, each matched so, and two overrides fitting one member are `E1030`. `ember bind --report` lists each such name with its members as the header writes them; `ember bind --emit-cpp` writes one thunk per member, named by the member's mangled name (`[MNG-1]`); `ember bind --init` writes one entry per member (`[CLI-5]`). Nothing runs for this while the program runs: a use is one direct call of its member's thunk, as any imported C++ function's (§X.4, "FFI call").

### FFI-48 (p21-annex-c.md; 20 words changed)
  - …callback or completion handle | | [[overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay | |]]

### FFI-39b (p21-annex-c.md; 25 words changed)
  + …`super.init(…)` selects the C++ base constructor {{as a call of the base's `new` selects a member (`[FFI-57]`:}}
  - …selects the C++ base constructor by [[arity]]
  + …selects the C++ base constructor by {{its arguments' number, names, words}}
  - …C++ base constructor by arity and [[argument types]]
  + …C++ base constructor by arity and {{types, exact before coercion; none or a tie `E5035`)}}

### NEW FFI-40b (p21-annex-c.md)
* `[FFI-40b]` **A `const` and a non-`const` pair.** A non-`const` method beside a `const` method of the same name and the same parameters, as the import maps them, imports with `_mut` added to its name — `at` and `at_mut`, as `get` and `get_mut` (`[FFI-42]`) and `span` and `span_mut` (`[FFI-17a]`) — so the two are two names, and which one a call runs is a word on the line (`cfg.at(i)`, `cfg.at_mut(i).port = 8080`), never read from the name the object is reached through nor from the lines around it; `at_mut` may change its object (`[FFI-40]`), and the `const` one only reads it where `[FFI-40a]` finds so or its entry says `@ffi(const)`. An overlay enters each under its own name (`@ffi(const) fn at(i) -> borrowed one from(this)`, `fn at_mut(i) -> borrowed one exclusive from(this)`, `[FFI-12]`), and an `override fn` of a virtual pair names the member it replaces the same way (`[FFI-39]`). A class that itself declares the added name makes it a name of several members (`[FFI-57]`), which a `rename` in the overlay tells apart (`[GRM-35]`). `operator[]`'s pair is `Index` and `IndexMut` (`[FFI-42]`), not this rule.

### FFI-42 (p21-annex-c.md; 407 words changed)
  + …C++ iterators with `begin`/`end` become `Iterable`. {{An operator, a member or a free function whose first operand is the class, is a method of the imported class as any member is (`[FFI-40]`, verdict 131): an overlay's `class C:` section names it by the method of the interface it maps to (§IV.8: the `const` `operator[]` is `index` and the other `index_mut`, `operator+` is `add`, `operator*`/`->` are `get` and `get_mut`), and that name matches the header's operator for `[FFI-12]`; a class whose only `operator[]` is not `const` has it as `index_mut` alone: `Index.index` promises a read of `this` (`[IFC-3]`), which a non-`const` operator does not (`[FFI-40]`; `std::map`'s inserts), so the class implements neither `Index` nor `IndexMut` (`IndexMut` needs `Index`), and `g[3]` on it is `E2072`, whose help names the method and what it says ("`Grid`'s one `operator[]` is not `const`, so it may change `g`, and `Grid` has no `index`; write `g.index_mut(3)`, or declare a `const operator[]` in the header"). A `T&` or `const T&` result takes its lifetime word in that entry as `[FFI-40]` says (`@ffi(const)` then `fn index(i) -> borrowed one from(this)`; `fn index_mut(i) -> borrowed one exclusive from(this)`), and the entry asserts, as every entry does, that a call within its contract cannot break memory safety (`[FFI-2]`): that the operator checks its index is the asserter's fact (`[TCB-1]`). The class implements the interface once its methods have entries; until then it implements none (`[TYP-40]`), the operator is reached only as a method of that name inside `unsafe:`, its reference result a `*mut T` or `*T` (`[FFI-40]`), and the operator's syntax on the class is `E3100`: "`g[3]` calls `Grid`'s `operator[]`, a C++ reference result whose target no contract names; call `g.index(3)` in an `unsafe:` block with a `# SAFETY:` note, or give the operator an entry in the overlay: under `class Grid:` write `@ffi(const)` then `fn index(i) -> borrowed one from(this)` for the `const` one, and `fn index_mut(i) -> borrowed one exclusive from(this)` for the other". A free operator function is a method of the imported class that is its first operand, with the other operand's type as `Rhs` (`operator*(const Vec&, double)` is `Vec.mul` with `Rhs = f64`); one whose first operand is not a class this header declares (`operator*(double, const Vec&)`) implements nothing, since an import adds no implementation to a type it did not declare (`[TYP-40]`), and is skipped with `W5033`, its arithmetic reached through an `extend` wrapper in the overlay (`[FFI-13]`) that calls it by a name the overlay gives.}}

### GPU-8 (p22-annex-d.md; 37 words changed)
  + …writes of `prev` within a frame. {{`prev` is a `ReadHandle[T]` and `cur` a `WriteHandle[T]`, two types `Copy` as handles are; `cmd.write` takes a `WriteHandle` and `cmd.read` a `ReadHandle`, so the wrong one is `E2020` when compiling, at no cost while the program runs.}}

### GPU-10 (p22-annex-d.md; 4 words changed)
  - …set built for another interface is [[a type error.]]
  + …set built for another interface is {{`E2020`, naming both interfaces.}}
