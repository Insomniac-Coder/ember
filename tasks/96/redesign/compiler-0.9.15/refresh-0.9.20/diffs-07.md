# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 07

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### PHIL-2 (p01-overview.md; 66 words changed)
  - …(`shares x`, `shares mut x`, `thread_shared [[x`, `[RC-1]`);]]
  + …(`shares x`, `shares mut x`, `thread_shared {{x`): the block is made where the value is made, so for a local shared on a later line the allocation is the local's own line, which `ember inspect --alloc` lists with the sharing line beside it (`[RC-1]`);}}
  + …a value that owns memory (`[OWN-8]`); {{a one-owner object handed to another thread while a `weakref` link names it (`[THR-2]`);}}
  - …f-string; a function value whose state [[exceeds]]
  + …f-string; a function value whose state {{has a block (`[CLO-10]`: more than one word, or a word}}
  - …function value whose state exceeds its [[inline capacity (`[CLO-10]`);]]
  + …function value whose state exceeds its {{body writes);}}

### PHIL-8 (p01-overview.md; 18 words changed)
  + …handle's generation, a foreign callback's token, {{the kind of a value marked `runtime` at a line one of its kinds would make unsafe (`[LT-48]`),}}

### LEX-23 (p02-lexical.md; 9 words changed)
  + …`unsafe overlay`), an `unsafe import c` {{or `unsafe import cpp`}}
  - …overlay`), an `unsafe import c` of [[C source]]
  + …overlay`), an `unsafe import c` of {{code compiled into the program}}

### LEX-14 (p02-lexical.md; 49 words changed)
  + …imported C fields such as `type`). {{A keyword written where a name is expected, any keyword of the table below, is `E0005`: `type = 3` is "`type` is a keyword", with the help "write `r#type` to use the word as a name" (`yield = 1`, a field `class: int` and a parameter `in: int` alike).}}

### LEX-15 (p02-lexical.md; 89 words changed)
  - …`safe` before `fn` in an extern [[block;]]
  + …`safe` before `fn` in an extern {{block and at the start of a statement before `:` (`[UNS-12]`);}}
  - …`import` or `overlay` and a string; [[`language`]]
  + …`import` or `overlay` and a string; {{`language`, `threads`}}
  - …`overlay` and a string; `language` and [[`threads`]]
  + …`overlay` and a string; `language` and {{`module`}}
  - …a string; `language` and `threads` after [[`#!`;]]
  + …a string; `language` and `threads` after {{`#!` (the word after `#! threads` is an identifier, one of `main`, `any`, `creator`, `[FFI-33]`); `package` and `read` inside the round brackets after `pub`;}}
  + …can have its `union` method (`[STD-16]`); {{`needs` directly after the `:` of a type parameter, an interface's header, an associated type or a `where` clause (`[TYP-17]`);}}
  + …`if`, `elif` or `while` header (`[GRM-44]`); {{`runtime` after the `:` of a parameter or of a local declared with its type, directly before the type (`[LT-48]`);}}
  - …this`, `mesh: owns Mesh`, `fn() owns [[x:`)]]
  + …this`, `mesh: owns Mesh`, `fn() owns {{x:`, and in a C contract `@ffi(param(widget, owns, one))`, verdict 102)}}

### LEX-21 (p02-lexical.md; 2 words changed)
  + …`//` before `/`, `**=` before `**`, {{`...` and}}

### GRM-2 (p03-grammar.md; 39 words changed)
  + …help `move it into a function`. {{Items and statements of the entry file stand in any order; the statements, in source order, are `main`'s body (`[FN-8]`). An import below an item or a statement is `E0100` ("imports come first: move `import std.math` above line 2").}}

### GRM-40 (p03-grammar.md; 129 words changed)
  - …indentation and below its attributes: `[T: [[Add[Output]]
  + …indentation and below its attributes: `[T: {{needs (Add[Output}}
  - …below its attributes: `[T: Add[Output = [[T] + Default]`]]
  + …below its attributes: `[T: Add[Output = {{T], Default)]`}}
  + …`fn sum(xs: Span[T]) -> T:`, `[F: {{needs}}
  + …`type Pair[T] = (T, T)`, `extend[T: {{needs}}
  - …directly above one of them is [[`E0100`.]]
  + …directly above one of them is {{`E0100`: a blank line or a comment line between `[T]` and its `fn` is `E0100` too, and so is every line that begins with `[` anywhere else, one that begins with a list included (`[10, 20].len()` on a line of its own; `n = [10, 20].len()` is the way), except the line of a `match` arm, a pattern that ends in `:` where no bound list and no statement can stand (`["quit"]:` under `match words:`, `[GRM-13]`), with one message: "a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to" (for an enum, an interface, a `type` alias or an `extend` block, the help gives the brackets after the name: `enum E[T]:`).}}

### GRM-3 (p03-grammar.md; 28 words changed)
  - …Mesh` whose block comes from a [[`LevelArena`.]]
  + …Mesh` whose block comes from a {{`LevelArena`, and `weakref Enemy using Arena` a `weakref Enemy` whose target's block comes from an `Arena`, the kind the link's end gives that block back to (`[TYP-46]`, `[ALC-5a]`).}}

### GRM-28 (p03-grammar.md; 18 words changed)
  - …passed by its name (`a = [[thread.spawn(push_a)`).]]
  + …passed by its name (`a = {{thread.spawn(owns push_a)`, the call's word as for any argument, `[FN-2a]`; `apply(push_a)` for an `F`-bounded parameter, which lends it).}}

### GRM-15 (p03-grammar.md; 414 words changed)
  + …in owns xs:`, `match owns e:`); {{before an argument for an `owns` parameter, whatever is given (`take(owns xs)`, `take(owns copy of xs)`, `take(owns [1, 2])`, `xs.iter().for_each(owns fn(x: int): println(x))`; `[FN-2a]`, `E3028` without it), except a value made in the call with a word that makes an owner or a link, which says what is given and takes no `owns` before it (`setup(shares mut game)` for `fn setup(owns game: shares mut Game)`, `ROSTER.push(weakref this)`, `thread_shared x` and `sharedref mut x` alike), `owns` itself among those words: where the parameter's own type is an `owns T`, the value made in the call is made for a place that owns it alone, and one `owns` says both that it has a block of its own and that the call takes it over (`r.pools.push(owns IntPool(items=[1, 2]))` for `pools: Array[owns dyn Pool]`; `meshes.insert(owns String("boss"), owns Mesh(points=[0.0]))` for a `Map[String, owns Mesh]`), a second `owns` before it being `E0109` ("one `owns` already says that the pool has a block of its own and that `push` takes it over; remove one: `r.pools.push(owns IntPool(items=[1, 2]))`"), and a constructor's argument, which takes no word and works as `=` does (`Words(names=xs)`, `[FN-2a]`);}}
  - …`b = owns n` for a [[number, which]]
  + …`b = owns n` for a {{value}}
  - …for a number, which `=` would [[copy,]]
  + …for a number, which `=` would {{copy (a number, a `bool`, a `StrView`, `[OWN-9]`),}}
  - …would copy, so that `n` ends [[there.]]
  + …would copy, so that `n` ends {{there, and before a part on the right of `=` (`body = owns req.body`), which hands the part over into the new name as `names.push(owns req.body)` does (`[OWN-11]`, with its refusals: a field of a type with `drop` `E3010`, an element `E3011`, a field of a class object or a part reached through a name that does not own it `E3013`), where `=` alone makes a view (`[BRW-12]`). `owns` before a lambda covers the whole lambda, whose body extends as far right as possible (level 2) and, inside a call, ends at the closing bracket or the next comma at the same depth (`[LEX-6a]`); another word before a lambda (`copy of fn ...`) is refused as `[GRM-42]` says. Before a value made on the line that is given to a place that is not an `owns T` (`b = owns (n * 2)`, `b = owns f()`, `xs = owns [1, 2]`; the required case is `[TYP-45]`'s) it ends nothing and is `E0109`: "`=` already hands a value made on the line over; remove `owns`: `b = n * 2`".}}
  - …`b = a` for a value [[that is not a number, an argument,]]
  + …`b = a` for a value {{`=` hands over,}}
  - …an argument, `return a`), and before [[`fn`,]]
  + …an argument, `return a`), and before {{`fn` anywhere but an argument for an `owns` parameter,}}
  + …each captured name: `fn() owns report:`). {{Before an argument for a parameter that only reads (`show(owns xs)`) it is `E3028`, "`show` only reads its argument; remove `owns`: `show(xs)`".}}

### GRM-42 (p03-grammar.md; 4 words changed)
  + …count:`, `fn() thread_shared log:`, `fn() weakref {{this:`, `fn() weakref thread_shared}}

### GRM-43 (p03-grammar.md; 147 words changed)
  + …mut` and `const ref mut` stand {{directly}}
  - …and bind as a prefix `-` [[does:]]
  + …and bind as a prefix `-` {{does, wherever they stand, in a call's argument and on the right of `=` as anywhere else, so that the word is always on the value it acts on:}}
  + …mut items`, `cam.target = weakref player`, {{`child.parent = weakref thread_shared this` (`[OWN-13]`),}}
  - …mut a)`, `blend(const ref mut d, [[s)`.]]
  + …mut a)`, `blend(const ref mut d, {{s)`, `m.insert(owns (i * 7), owns i)` and `tx.send(owns (i * i))` (the brackets say the product is what is handed over, `[GRM-47]`), `q = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])` (each branch says its word). A conditional expression or a `match` expression given as one argument or as one value is given branch by branch: each branch says the word the place takes, and a branch without it is what `[FN-2a]` says for the argument (`E3028`). `owns i ** 2` is `owns (i ** 2)`, as `-i ** 2` is `-(i ** 2)` (row 17), and `owns Mesh(points=[0.0]) using current` is `owns (Mesh(points=[0.0]) using current)` (row 18).}}

### GRM-27 (p03-grammar.md; 4 words changed)
  - …x in xs if p(x)]` is [[`xs.iter().filter(fn(x)]]
  + …x in xs if p(x)]` is {{`xs.iter().filter(owns fn(x)}}
  - …xs if p(x)]` is `xs.iter().filter(fn(x) => [[p(x)).map(fn(x)]]
  + …xs if p(x)]` is `xs.iter().filter(fn(x) => {{p(x)).map(owns fn(x)}}

### GRM-38 (p03-grammar.md; 3 words changed)
  - …is a parenthesised expression, and `(e,)` [[a tuple.]]
  + …is a parenthesised expression, and `(e,)` {{is `E0100` (`[GRM-29]`).}}

### GRM-13 (p03-grammar.md; 196 words changed)
  + …branch that runs owns its part. {{A list pattern `[p, q]` matches a `Span[T]`, `MutSpan[T]`, `Array[T]` or `[T; N]` of exactly that many elements and binds each name to its element as above (an `N` that can never equal the count is `E2020`, naming both); `[p, q, ..]` matches one with at least that many; `[p, ..rest]` names the remaining elements `rest`, a `Span[T]` of them (a `MutSpan[T]` where the scrutinee is a `ref mut` name of the list), a view like every pattern name, which the list's rules govern (`[BRW-1]`), and `[..rest]` alone names the whole; a `..rest` name reads its list in place, so under `match owns xs:` it is `E3013` (it would view what the match hands over, `[OWN-11]`), whose help is to match `xs` in place and take what is needed with `rest.iter().copy_each().to_array()` (`[STD-19]`) or `xs.drain(1..)`. A list pattern against a `String` or `StrView` is `E2020`, whose help gives `split`. A list pattern stands nested, on a `for` line, or first on the line of a `match` arm, which ends in `:` and so is neither a bound list nor a statement (`["quit"]:` and `[first, ..rest]:` under `match words:`); first on any other line it is `E0100` (`[GRM-40]`).}}

### TYP-5 (p04-types.md; 235 words changed)
  - …of it), and reading through a [[`ref` or]]
  + …of it), and reading through a {{`ref`,}}
  + …a `ref` or an `owns T` {{or a `shares`, `shares mut` or `thread_shared` owner}}
  - …class object to a base class [[(upcast);]]
  + …class object to a base class {{(upcast), with its word kept: a `Dog` to an `Animal`, and a `shares Dog`, `shares mut Dog`, `thread_shared Dog`, `weakref Dog`, `weakref thread_shared Dog` or `ref Dog` to the same word on `Animal` (`=` on a `shares mut` name hands the owner over, `[HEAP-6]`, and never writes the block, so a wider owner cannot put another kind into the place); never a `ref mut Dog` to a `ref mut Animal`, since `=` on a `ref mut` name writes the place (`[TYP-14]`) and the place is a `Dog`'s (`E2020`: "`ref mut d` names a `Dog` place, which a `ref mut Animal` could fill with another kind; pass `d` to a parameter that reads, or `shares mut d` if the dog is shared"); and never the reverse, which is a kind test or a cast (`[TYP-44]`);}}
  - …composes with the others only where [[stated (widening]]
  + …composes with the others only where {{stated: widening}}
  - …only where stated (widening after range [[erasure,]]
  + …only where stated (widening after range {{erasure; rule 7 before any rule, since a read through a view or an `owns T` converts nothing but reaches the value (`greet(n)` for `n: ref String` and `fn greet(s: StrView)`, rule 7 then 4; `f(b)` for `b: owns Array[int]` and `fn f(xs: Span[int])`, rule 7 then 6);}}
  - …range erasure, and rule 11 after [[rules 1–4).]]
  + …range erasure, and rule 11 after {{any one of the other rules, so an `Option[T]` place accepts as `Some` whatever a `T` place accepts (`cap: Option[u64] = n` after `n = 5`; `slot: Option[owns dyn Weapon] = owns Sword(edge=4)`; `on_hit: Option[Handler] = log_hit`), still one level only.}}

### TYP-6 (p04-types.md; 138 words changed)
  - …of truncating is `T.try_from(x) -> Result[T, [[RangeError]`.]]
  + …of truncating is `T.try_from(x) -> Result[T, {{RangeError]`; * `bool → integer`: `true` is 1 and `false` 0 (Python's `int(True)`, `[PHIL-14]`); nothing converts to `bool` (`x != 0` is the test); * a unit-only enum → an integer type that holds every one of its discriminants (`Color.Green as u8`, `[ENM-3]`); an integer never converts to an enum, since a value of an enum is one of its variants and `Option[Color]` keeps `None` in an unused discriminant (`[TYP-13]`): `7 as Color` is `E2020` ("an integer may be no `Color`; write `Color.from_repr(7)`, an `Option[Color]`, and open it: `if Color.from_repr(7) exists as c:`"); * `as` between any other pair of types is `E2020`, naming both, and where two of the conversions above join them its help gives the chain (`c as int` for a `char`: "a `char` converts to a `u32` only; write `(c as u32) as int`").}}

### TYP-13 (p04-types.md; 24 words changed)
  + …class object, an `owns T`, a {{`shares`, `shares mut` or `thread_shared` value, a `sharedref mut` link, a}}
  + …an `owns T`, a `ref`, a {{`ref mut`, a}}
  + …`owns T`, a `ref`, a `Span`, {{a `MutSpan`,}}
  - …(`[STD-4]`). For class objects, `owns T`, [[`ref`]]
  + …(`[STD-4]`). For class objects, `owns T`, {{the shared owners and links, `ref`, `ref mut`}}

### TYP-14 (p04-types.md; 105 words changed)
  - …* `[TYP-14]` A reference, [[and]]
  + …A reference, and an `owns T`, {{and a `shares T`, `shares mut T` or `thread_shared T` owner,}}
  - …operand, an argument to a plain [[parameter.]]
  + …operand, an argument to a plain {{parameter (a `thread_shared T` as a read, which `[THR-1]` allows every owner; the owner word says who owns the value, not what it is).}}
  + …views that instead (`rest = tail.trim()`). {{`=` on a `ref mut` name of a `dyn I` type never gives the place a whole value, since the place's own kind is known only while running and its size is the kind's: it is `E3023` ("`v` views a value of a kind known only while running; change it through its methods, `v.scale(2.0)`, or take the value over, `owns v: owns dyn Shape`, and return the new one").}}

### TYP-45 (p04-types.md; 356 words changed)
  - …several owners on any threads, counted [[atomically,]]
  + …several owners on any threads, counted {{for several threads (`[RC-4]`),}}
  + …XI); * `weakref T`: a link {{to a value on one thread (a class object with one owner or with `shares` or `shares mut` owners, or a value held `shares` or `shares mut`)}}
  - …a link that does not keep [[its target]]
  + …a link that does not keep {{it}}
  - …none (`parent: weakref Node = None`); [[it]]
  + …none (`parent: weakref Node = None`); {{`weakref thread_shared T`: a link to a `thread_shared T`, a type of its own, written so in a field (`parent: weakref thread_shared Node = None`), a local's annotation (`up: weakref thread_shared Node`), a parameter, a result, an element (`Array[weakref thread_shared Player]`) and a bound, whose opening holds a count for the opened name's life (`[WK-12]`). The kind of the target is in the link's type and nowhere else (`[WK-11]`); a capture (`fn() weakref p:`) and a local declared by `w = weakref p` take the type `weakref p` has (`[OWN-13]`). Neither type converts to the other: a link of one given to a place of the other is `E2020` ("`cam.target` is a `weakref Player`, a link to a player on one thread, and `p` is a `thread_shared Player`; for a link to a shared player write `target: weakref thread_shared Player = None` in `Camera` (line 7)"). Each}}
  - …opened before its target is used [[(`[GRM-44]`).]]
  + …opened before its target is used {{(`[GRM-44]`); the name opened from a `weakref thread_shared T` reads the value as a parameter `p: thread_shared T` does (`[THR-1]`), and `ref mut` after `exists as` on one is `E7014` ("`w` links a `thread_shared Player`, which no owner changes directly; open it read-only, `if w exists as p:`, and change the player through its `Atomic` fields or under its lock").}}
  - …given an object without the word, [[it]]
  + …given an object without the word, {{a link}}
  - …mut v: dyn I` may change [[it).]]
  + …mut v: dyn I` may change {{it); anywhere else a view of such a value is `ref dyn I`, `ref mut dyn I` or `sharedref mut dyn I` (`[IFC-5]`, `[TYP-45a]`), one pointer and one table, governed as every view of its word is (`[LT-46]`, `[TYP-15]`).}}
  - …fn()`, then `handlers: Array[Handler] = []`). [[`Box`, `Shared`, `Weak`, `SyncShared`, `ForeignBox`, `RefCell`, `Ref` and `RefMut` are not types of Ember: naming one is `E1010`, whose message gives what to write (`owns T` for `Box[T]` and `ForeignBox[T]`, `owns dyn I` for `Box[dyn I]`, `owns T using A` for `Box[T, A]`, `shares T` or `shares mut T` for `Shared[T]`, `weakref T` for `Weak[T]`, `thread_shared T` for `SyncShared[T]`, and for `RefCell` the field written plainly and changed through a name that may change it).]]

### TYP-46 (p04-types.md; 236 words changed)
  - …`owns`, `shares`, `shares mut` or `thread_shared` [[type,]]
  + …`owns`, `shares`, `shares mut` or `thread_shared` {{type (a `thread_shared T using A` is a `thread_shared T` whose block comes from an `A` and whose owners are counted away from its making line, `[RC-4]` point 4, allowed where `[TYP-46a]` says),}}
  - …or `thread_shared` type, a class type, [[or]]
  + …or `thread_shared` type, a class type, {{a standard collection whose storage is on the heap:}}
  - …class type, or an `Array`, a [[`Map`]]
  + …class type, or an `Array`, a {{`Map`, a `Set`, a `String` (`name: String using LevelArena = String("") using current`), a `Deque`, an `SoA` (`[SOA-1]`), and a `SmallArray`, whose elements beyond `N` are (`[STD-7]`), and a `weakref T` type, where it names the kind of manager the target's block comes from (`last: weakref Enemy using Arena`, `Array[weakref Enemy using Arena]`), since the link's end gives that block back (`[ALC-5a]`) and must know to whom: a `weakref x` takes the kind from the type of what `x` names, as it takes the target's kind from the words on the path (`[OWN-13]`), and a link given to a `weakref T` place of another kind is `E2020` ("`links` holds links to `Enemy` objects from the built-in manager (`Array[weakref Enemy]`), and `b` was made in `level`, an `Arena`; say which kind: `links: Array[weakref Enemy using Arena]`,}}
  - …or an `Array`, a `Map` or [[a `Set`;]]
  + …or an `Array`, a `Map` or {{make `b` in the built-in manager, `b = Enemy()`, as the links' other targets are");}}
  - …or a `Set`; after any other [[type]]
  + …or a `Set`; after any other {{type, one with no heap memory of its own (an `int`, a struct held inline, a `FixedArray`, a `RingBuffer`, a `BitSet`, an arena container, `[ARN-5]`),}}
  + …made in an instance of `A` {{(a place with no `using` is a `using Global` place, so a place's type always names its values' manager)}}

### TYP-17 (p04-types.md; 215 words changed)
  - …parameters are bounded by interfaces: `[T: [[Add[Output]]
  + …parameters are bounded by interfaces: `[T: {{needs (Add[Output}}
  - …bounded by interfaces: `[T: Add[Output = [[T] + Default]`]]
  + …bounded by interfaces: `[T: Add[Output = {{T], Default)]`}}
  + …`fn sum(xs: Span[T]) -> T` (`[GRM-40]`). {{What a type parameter needs is written after `needs`: one bare (`[T: needs Copy]`), several in round brackets, separated by commas (`[T: needs (Add[T, Output = T], Copy)]`), since a comma also separates the type parameters (`[K: needs (Eq, Hash), V]`); the same in an `extend` block's brackets (`extend[T: needs Display] Array[T] implements Display:`), an interface's parents (`interface Error: needs (Debug, Display):`), an associated type (`type Iter: needs Iterator[Item = Item]`) and a `where` clause. A bound without `needs`, or needs joined with `+`, is `E0100`, whose message gives the line with `needs` (`[T: Add + Copy]` gives `[T: needs (Add, Copy)]`); interfaces joined with `+` after `dyn` or `some` are `E0100` too, whose message gives the brackets (`owns dyn Shape + Debug` gives `owns dyn (Shape, Debug)`; after `dyn` and `some` the brackets stand without `needs`, the word before them saying what the value is).}}
  + …are available, plus copy for `T: {{needs}}
  + …its bound is written: write `[T: {{needs}}
  - …A function that is not `pub` [[may leave]]
  + …A function that is not `pub` {{writes all of}}
  - …is not `pub` may leave them [[out:]]
  + …is not `pub` may leave them {{or none: with none,}}
  - …the function is given; the bound [[follows").]]
  + …the function is given; the bound {{follows"); a list that leaves out one the body needs is `E2040`, whose message gives the whole line ("`direct_add` writes what `T` needs, so it writes all of it: `a + b` (line 2) needs `Add`, which `Lookalike` does not give; write `[T: needs (Lookalike, Add[T, Output = T])]` in place of `[T: needs Lookalike]`").}}

### TYP-19 (p04-types.md; 15 words changed)
  - …* `[TYP-19]` There is no [[specialisation,]]
  + …* `[TYP-19]` There is no {{specialisation and}}
  - …There is no specialisation, no higher-kinded [[type and no variadic generic.]]
  + …There is no specialisation, no higher-kinded {{type; a generic may take any number of types (`[TYP-47]`, verdict 115).}}
  + …and does not meet it (`extend[T: {{needs}}

### TYP-24 (p04-types.md; 35 words changed)
  + …are `E2070`, disambiguated as `I[A].m(recv, …)`. {{Inside a generic body the same holds for bounds: a method that two of a type parameter's bounds both declare is `E2070`, whose message names both bounds and the call to write (`Shape.area(x)` or `Drawable.area(x)`).}}

### TYP-43 (p04-types.md; 246 words changed)
  - …them as it names any (`T: [[Send + ThreadShared`,]]
  + …them as it names any (`T: {{needs (Send, ThreadShared)`,}}
  + …(`T: Send + ThreadShared`, `where Self: {{needs}}
  + …any other. A program writes `Send` {{or `ThreadShared` in one checked place, the header of an `open` or `abstract` class (`pub open class Script implements Send, ThreadShared:`), a promise about every class derived from it that the compiler checks at each class's declaration (`[THR-8]`, `[THR-9]`), so it needs no `unsafe`; on a generic class it holds for each instance whose arguments meet the needs its parameters are bounded by; on a struct, an enum or a class that is neither `open` nor `abstract`, whose markers are inferred, the word is refused as below (`E3100` for `Send`: "`Door` cannot be subclassed, so whether any thread may have a `Door` is read from its fields; remove `implements Send`"; `E2040` for `ThreadShared`). Otherwise a program writes `Send`}}
  - …writes `Send` by hand, for a [[struct]]
  + …writes `Send` by hand, for a {{struct, enum}}
  - …by hand, for a struct or [[enum]]
  + …by hand, for a struct or {{class}}
  + …own package that holds a raw {{pointer (the vouching is about the}}
  + …package that holds a raw pointer, {{not the kind of type holding it, `[THR-8]`),}}
  - …taking the responsibility the compiler cannot [[check]]
  + …taking the responsibility the compiler cannot {{check; the declaration vouches for the type's raw pointers alone, and every other component is inferred as for any type, so a `shares` owner or a `weakref T` link in such a type still keeps it on one thread (`E7015`)}}
  + …it"); a generic one (`unsafe extend[T: {{needs}}
  - …by. `ThreadShared` is written only in [[bounds,]]
  + …by. `ThreadShared` is written only in {{bounds and in that header,}}
  - …`unsafe extend` of any interface but [[`Send` or]]
  + …`unsafe extend` of any interface but {{`Send`,}}
  - …interface but `Send` or an `unsafe [[interface`]]
  + …interface but `Send` or an `unsafe {{interface`, or `ThreadShared` on an `extern type`, an opaque C++ class or a `@ffi(handle)` type, whose contents the compiler cannot see (`[FFI-8]`; `E3100` without `unsafe`)}}
  + …`Send` or an `unsafe interface` (ODR-111). {{A struct holding a raw pointer is never `ThreadShared` (`[THR-9]`): a wrapper several threads read at once holds the foreign object as a `@ffi(handle)` type vouched `ThreadShared`, not as a raw pointer.}}
  + …X), not of a property: `T: {{needs}}
  + …is `E1010`, whose message gives `T: {{needs}}

### TYP-36 (p04-types.md; 190 words changed)
  + …is `E2040`. A read-only view `ref {{T`, and a `shares T`, `shares mut}}
  + …`E2040`. A read-only view `ref T` {{or `thread_shared T` owner,}}
  - …`Hash`, `Display` and `Debug` as `T` [[does]]
  + …`Hash`, `Display` and `Debug` as `T` {{does, on the value it views or owns}}
  - …`Debug` as `T` does (so `sorted(m.keys())` [[works).]]
  + …`Debug` as `T` does (so `sorted(m.keys())` {{works, `a == b` on two owners compares the two values, and `println(scores)` prints the list as `[10, 20, 30]`); whether two owners own one value is `a is b` (`[EXP-9]`), and a class object's `==` stays its identity.}}
  + …when `I` requires `Eq` (`interface Shape: {{needs}}
  - …two of one kind compare their [[contents.]]
  + …two of one kind compare their {{contents; that entry is the one way `Eq` is reached through `dyn` (`[TYP-22]`). Of the other interfaces `I` requires, `owns dyn I` (and `shares dyn I`, `shares mut dyn I`, and `dyn I` where a view may stand) has those whose methods `[TYP-22]` reaches through `dyn`, `Display`, `Debug` and `Hash` (each method names `Self` only as `this`, and `hash`'s hasher is a parameter that views its value, so it gets one entry), and never `Ord` (`cmp` takes `other: Self`): a bound needing `Ord` on it is `E2040`, whose help is `sort_by` with the kinds tested first.}}
  - …cannot copy as itself (a `weakref` [[link,]]
  + …cannot copy as itself (a `weakref` {{link;}}
  - …as itself (a `weakref` link, a [[value]]
  + …as itself (a `weakref` link, a {{`ref mut T` view, a `MutSpan[T]` or a struct holding one}}
  - …(a `weakref` link, a value of [[a view type,]]
  + …(a `weakref` link, a value of {{the two;}}
  - …type with no `Clone`) it is [[`E2116`.]]
  + …type with no `Clone`) it is {{`E2116`; a read-only view is `Copy`, so `copy of` before one gives one more view of the same place (`[OWN-8]`).}}

### TYP-21 (p04-types.md; 46 words changed)
  - …desugar to these interfaces for non-scalar [[operands;]]
  + …desugar to these interfaces for non-scalar {{operands:}}
  - …b)` with both operands read in [[place.]]
  + …b)` with both operands read in {{place, for `String` too (`String` implements `Add[StrView, Output = String]` and `AddAssign[StrView]`, `[TXT-11]`, so it meets `T: needs Add[StrView, Output = T]`, and `a + b` on it makes a new `String`).}}
  - …operator: `3.add(4)` is `3 + 4`. [[`bool`,]]
  + …operator: `3.add(4)` is `3 + 4`. {{`bool` and}}
  - …is `3 + 4`. `bool`, `char` [[and `String`]]
  - …`char` and `String` implement none of [[them (`String`'s `+` takes its left operand over, `[TXT-11]`).]]
  + …`char` and `String` implement none of {{them.}}

### HASH-1 (p04-types.md; 6 words changed)
  + …`[HASH-1]` `Hash.hash` is generic over `H: {{needs}}
  - …not the prelude) is a hashing [[state that is not `Copy`,]]
  + …not the prelude) is a hashing {{state,}}

### TYP-22 (p04-types.md; 481 words changed)
  - …object (none is a `static fn`), [[does]]
  + …object (none is a `static fn`), {{names `Self` nowhere but as `this`:}}
  - …is a `static fn`), does not [[return `Self`]]
  + …is a `static fn`), does not {{as a result}}
  + …fn`), does not return `Self` by {{value, and not as a parameter's type or inside one (`fn eq(other: Self)`, `fn add(rhs: Self)`, `fn merge(owns other: Self)`), since through a `dyn I`}}
  + …does not return `Self` by value {{the kind of any other value is unknown}}
  + …by value (methods marked `where Self: {{needs}}
  - …value (methods marked `where Self: Sized` [[excepted),]]
  + …value (methods marked `where Self: Sized` {{excepted, which a `dyn` value does not have),}}
  + …which gets one `dyn` entry (`[V: {{needs}}
  + …in instead of returning it, `[V: {{needs}}
  - …Visitor]` above `fn make(ref mut v: [[V)`").]]
  + …Visitor]` above `fn make(ref mut v: {{V)`"; "`Same` cannot be used as `dyn Same`: `same` takes `other: Self`, and through a `dyn Same` the kind of `other` is unknown; take `other: dyn Same` and test its kind in the body, `if other is Tag as o:`, or mark `same` with `where Self: needs Sized` and call it on the concrete types"). A method of an interface that `I` requires (`interface Shape: needs (Eq, Ord):`) is reached through `dyn I` only where it names `Self` nowhere but as `this`, and through the two entries this document gives: `clone` (§IX.1) and `eq` (`[TYP-36]`, the kinds compared first); any other such method called through a `dyn I` value is `E2050` at the call ("`cmp` takes `other: Self`, and through a `dyn Shape` the kind of `other` is unknown; test the kinds first, `if a is Circle as x:` and inside it `if b is Circle as y:`, and call `x.cmp(y)`"). A `dyn I` names every associated type of `I` with `Name = T` (`owns dyn Iterator[Item = int]`, `[GRM-8c]`); one left unnamed is `E2060` ("`dyn Iterator` leaves `Item` open; write `owns dyn Iterator[Item = int]`").}}
  + …interface that requires `Clone` (`interface Shape: {{needs}}
  + …`owns dyn Shape` is `Clone` (§IX.1). {{`dyn (A, B)` is a value of one type implementing every listed interface, each of which is itself usable as `dyn`, held with one data pointer and one table pointer as `dyn I` is. The table of the set for a type is written by the package that converts a value of that type to `dyn (A, B)` (`[TYP-5]` rule 8), which sees both, under one symbol per set and type (`[MNG-4]`), so the linker keeps one copy (`[MONO-1]`) and every package lays it out and reads it alike (`[BLD-16]`): first the type's index (`[TYP-44a]`), then a pointer to each listed interface's own table for the type, then `A`'s entries and `B`'s, each interface's in its declaration order. A call through it is `[DSP-3]`'s, one indirect call; a kind test is one compare, as on `dyn I`; a `dyn (A, B)` value given to a `dyn A` place converts by `[TYP-5]` rule 8 with one load of the member's table pointer, and to a `dyn C` of an interface not listed it is `E2020`. `some (A, B)` is one concrete type meeting both, with no table (`[TYP-32]`). A marker interface in the list (`Send`, `ThreadShared`, `[TYP-43]`) has no entries and records the marker, which the type converted must have (`[TYP-5]` rule 8, `E2020` otherwise), so that the value has it (`owns dyn (Command, Send)` is `Send`, `[THR-8]`), as `interface I: needs Send:` does. One interface takes no brackets (`owns dyn Shape`).}}

### TYP-44 (p04-types.md; 145 words changed)
  + …(`return ref mut pool.dense[pool.sparse[e]]`) is `E3027`. {{The words before a value act on a cast as on any value (`[GRM-43]`), after the kind test, which runs first and stops the program before anything is handed over: `owns cast s as Circle` hands the whole owner `s` over as its kind, the same block and no copy (an `owns Circle` from an `owns dyn Shape`; a `Circle` object with one owner from a `Unit`), `shares cast s as Circle` makes one more owner typed `shares Circle` from a `shares dyn Shape` or a `shares Shape`, and `return cast u as Player` hands `u` over as a `Player`, as `return u` would (`[FN-2a]`); a cast of a part is a part (`[OWN-11]`), and a cast of a name that does not own its value is handed over by nobody (`E3013`). A cast of a temporary gives the value, held as the temporary was.}}

### ATT-1 (p05-declarations.md; 11 words changed)
  + …p` after the result type, `[LT-1a]`; {{`@component`: any type is a component, with no mark (Part XII);}}

### TYP-38 (p06-expressions.md; 43 words changed)
  - …array). With no context it is [[`Array[T]`.]]
  + …array). With no context it is {{`Array[T]`, `T` the elements' own type: `["ann", "bob"]` is an `Array[StrView]` of text written in the program, allocating no `String` (`[TYP-15]`), unlike a map or set literal (below); a list that is to own its text says so (`names: Array[String] = ["ann", "bob"]`).}}

### DET-2 (p10-effects.md; 251 words changed)
  - …reassociation or any `@fastmath` relaxation; a [[transcendental]]
  + …reassociation or any `@fastmath` relaxation; a {{floating-point}}
  + …any `@fastmath` relaxation; a transcendental function {{the platform's C library computes whose result IEEE 754 does not fix to one correctly rounded answer (`[STD-27]`'s methods `sin` through `hypot`, `cbrt` and `pow` among them, and `a ** b` on a float base, which is `pow` (`[TYP-30]`); not `sqrt`, `fma`/`mul_add`, `abs`, `copysign`, `floor`, `ceil`, `trunc`, `round`, `round_half_away` or `fract`, which every target computes alike, nor `rsqrt`, `lerp` and `smoothstep`, which `std.math` builds}}
  + …`@fastmath` relaxation; a transcendental function from {{those), where `std.math.det`'s is not used, `E4070` naming}}
  - …relaxation; a transcendental function from the [[platform library instead of `std.math.det`;]]
  + …relaxation; a transcendental function from the {{function or operator and the line to write ("`boost` is `@deterministic`, and `x ** 2` (line 5) is the platform's `pow`, whose last bits differ between C libraries; write `x * x`, or `det.pow(x, 2.0)`"); a SIMD `_approx` operation (`[SIMD-4]`), whose bits are the processor's own;}}
  + …source, thread and job completion order; {{a read whose value depends on which thread acted first: a receive on a channel more than one thread may send to (one sender on one thread gives the sent order), `try_lock`'s and `try_recv`'s answer, and a lock's view or an `Atomic` read of a place another thread, or a task not yet joined, may change (`[THR-11]`; after the join the place holds what the joined tasks left, in spawn order for a reduction, `[THR-21]`), `E4070` naming the senders ("`gather` is `@deterministic`, and `rx.recv()` (line 9) receives from a channel two threads send to (lines 7 and 8), so which value comes first differs between runs; give each task its own result and join them in order: write `h1 = scope.spawn(...)`, `h2 = scope.spawn(...)` returning the values, and `out.push(owns h1.join())`, `out.push(owns h2.join())`");}}

### DET-3 (p10-effects.md; 41 words changed)
  + …parameter is checked per instance (`[EFF-2]`). {{A call through a `dyn I` value or a `virtual` method of a `pub` interface or class is admitted under `@deterministic` only when the method declares `@deterministic` itself, as `[EFF-6]` says of every contract; otherwise it is `E4070` at the call.}}

### COST-6 (p10-effects.md; 360 words changed)
  + …(`dst_name` and `src_name` name one track)" {{| | a `runtime` name's test (`[LT-48]`) | "`xs` changes while `r` looks at it: `p` is a `Yours`, whose `pick` gives back the list it was handed"}}
  + …`Square`, not a `Circle`" | | {{an export called on another thread than its `threads=creator` or `main` says (`[FFI-33]`, `[FFI-33c]`) | "C called `tick` on another thread, but its export says `threads=main`" | | an `owns T` of a `threads(creator)` type ended on another thread (`[FFI-33b]`) | "`ctx`, a `GlContext` made on thread 1 (line 9), ended on thread 2; a `threads(creator)` value ends where it was made" | |}}
  + …may run only once" | | {{a C++ override call or `delete` of a trampoline-class object on another thread (`[FFI-17d]`) | "C++ called `on_click` of a `Button` made on another thread (line 16); its class says `threads=creator`" | |}}
  + …a length of 3; use `.get(i)` {{for an `Option`" | | an overflow check (`[TYP-8]`) | "`integer overflow in '+'`: `x + 1` on line 3 (9,223,372,036,854,775,807 + 1); use `x.checked_add(1)` for an `Option`" | | division by zero (`[TYP-6]`) | "`division by zero`: `7 // (y - y)` on line 4" | | a shift out of range (`[TYP-10]`) | "`shift out of range`: `1 << n` on line 9 shifts an `int` by 70; use `wrapping_shl`" | | capacity overflow (`[HEAP-8]`) | "`capacity overflow`: `xs.reserve(...)` on line 6 asks for 18,446,744,073,709,551,615 elements" | | allocation failure (`[ALC-4]`, `[ALC-6]`) | "`out of memory`: `xs.reserve(...)` on line 6 asked `Global` for 1,073,741,824 bytes; use `try_reserve`" | | an invalid `Layout` (`[ALC-2]`) | "`invalid Layout`: `Layout(3, 3)` on line 7, an alignment that is not a power of two" | | a `FixedArena` exhausted (`[ARN-4]`) | "`fixed.alloc_zeroed(...)` on line 9: `fixed` has 16 bytes free and 32 were asked; use `try_alloc_zeroed` for an `Option`" | | a full `Pool` (`[HND-3]`) | "`pool.insert(...)` on line 11: every slot is live or retired; use `try_insert` for a `Result`" | | the callback table full (`[FFI-53]`) | "`on_click(...)` on line 16: no callback slot is free; every one of the 65,536 slots a 32-bit target has is live or retired" | | a stale handle in `pool[h]` (`[HND-1]`) | "`pool[h]` on line 12: the handle is stale (slot 3, generation 2 of 5); use `pool.get(h)`}}

### CT-5 (p14-comptime.md; 335 words changed)
  - …`static` (not `static mut`) whose type [[gives no]]
  + …`static` (not `static mut`) whose type {{is not one a plain}}
  - …mut`) whose type gives no name [[a way to change the value (no `Mutex`, `RwLock`, `Cell`, `Atomic` or `guarded by` field on the path to it),]]
  + …mut`) whose type gives no name {{may change,}}
  - …the value is materialised in read-only [[memory; statics]]
  + …the value is materialised in read-only {{memory, except a value that is, or owns, a `thread_shared` value, whose counts another owner (`fn() thread_shared st:`) or a `weakref thread_shared T` link writes with atomics: that value is materialised in the image in memory that may be written, with no initialisation code, each `thread_shared` value in it marked handed over with its one owner, the static's, counted in its away count (`[RC-4]` point 4, as a value made `using` an instance is), so that no thread asks which thread made it and every count operation on it finds the mark first; its count never reaches zero, since the static holds that owner for the life of the program. Statics}}
  - …in read-only memory; statics are never [[dropped;]]
  + …in read-only memory; statics are never {{dropped. No other header in constant data is ever written: a `weakref T` link to a value a `static` or a `comptime(e)` result reaches is `E7015` (`[THR-2]`), and `shares` of one is `E2122` (`[RC-1]`). A `comptime(e)` whose value is, or owns, a `thread_shared` value is materialised the same way; A type a plain name may change is one with, on any path to a field, a `Mutex`, `RwLock`, `Cell`, `Atomic`, `Once`, a channel end, a `guarded by` field, or a class with a `@cached` method (`[FN-1]`, `[CLS-12]`);}}
  + …your own, which may grow (`Alloc`). {{Its type is the read-only view of the value: `Span[T]` for an `Array[T]`, `StrView` for a `String`, `ref T` for any other value that owns heap memory, with the `static` region (`[LT-3]`); `copy of comptime(e)` has the value's own type. A `comptime(e)` whose type is one a plain name may change is `E6010`, since its value would be constant data a line may write ("`comptime(fresh())` (line 11) makes a `Stats` when compiling, which is read-only, and `Stats.hits` is an `Atomic`, which a plain name changes; make the value while running: write `s = fresh()`, or keep it in a `static`"); `copy of comptime(e)` of such a type is `E6010` the same way.}}

### DRV-1 (p14-comptime.md; 81 words changed)
  - …not meet is `E2080`, naming the [[field.]]
  + …not meet is `E2080`, naming the {{field (`@derive(Copy)` on a type with a `String` field, which is not `Copy`). On a generic type the generated `extend` writes, as its parameters' needs, what its fields need of them (`@derive(Error)` above `[T]` and `enum SpawnError:` with `Rejected(T)` generates `extend[T: needs Debug] SpawnError[T] implements Error:`), so the derive holds for the instances that meet them, as `@derive(Copy)` above `[T]` and `pub struct Range:` is `Copy` when `T` is (ODR-027, `[STR-5]`, `[IFC-3]`); `E2080` stays for a field whose written type fails.}}

### SER-1 (p14-comptime.md; 97 words changed)
  + …`@derive(Serialize, Deserialize)` implements the method `[W: {{needs}}
  + …Result[void, SerError]` and the function `[R: {{needs}}
  + …Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(since=N)`. {{A `@ser(skip)` field is written to no format and read from none: `deserialize` gives it what `@derive(Default)` gives a field (`[DRV-1]`: its declared default, else its type's `Default`), so it is never a missing field (`[SER-2]`'s error is for a field the type writes and the data lacks); a skipped field with no declared default whose type has no `Default` is `E2080`, naming the field ("`captain` is `@ser(skip)`, so reading a `SaveGame` back must give it a value, and `shares Player` has no `Default`; give the field a default: `captain: shares Player = shares Player()`").}}

### TXT-6 (p15-stdlib.md; 20 words changed)
  - …`StrView` is `{const uint8_t *ptr; size_t [[len}`.]]
  + …`StrView` is `{const uint8_t *ptr; size_t {{len}` (a `StrView` crosses only as an import's contract makes one, `[FFI-10]`, or as the `Span[u8]` an export takes, `[FFI-31b]`).}}

### TXT-11 (p15-stdlib.md; 68 words changed)
  - …b` for `a: String, b: StrView` [[takes]]
  + …b` for `a: String, b: StrView` {{reads both operands in place and gives a new `String`, `a`'s text then `b`'s, in one allocation (`String` implements `Add[StrView, Output = String]`, `[TYP-21]`; a `b: String` is read as a `StrView`, `[TXT-8]`), so}}
  - …`a: String, b: StrView` takes `a` [[over and returns it extended;]]
  + …`a: String, b: StrView` takes `a` {{stays usable, as in Python;}}
  - …returns it extended; `s += t` [[appends.]]
  + …returns it extended; `s += t` {{appends to `s` in place (`AddAssign[StrView]`) and allocates only to grow, which is how text is built in a loop (`s = s + t` copies `s` each round).}}

### GRM-35 (p16-ffi.md; 330 words changed)
  - …["unsafe"] "overlay" ("c" | "cpp") STRING [[["with" "(" "header_hash" "=" STRING ")"]]]
  + …NEWLINE | "enum" IDENT NEWLINE | {{overlay_name | extend_decl | overlay_class) overlay_name :=}}
  - …| "enum" IDENT NEWLINE | "rename" [[IDENT]]
  + …| "enum" IDENT NEWLINE | "rename" {{member_ref}}
  - …IDENT "as" IDENT NEWLINE | "hide" [[IDENT]]
  + …IDENT "as" IDENT NEWLINE | "hide" {{member_ref}}
  - …IDENT NEWLINE | "hide" IDENT NEWLINE [[| extend_decl | overlay_class)]]
  - …"class" IDENT ":" NEWLINE INDENT {{attribute} [[fn_header NEWLINE}]]
  + …"class" IDENT ":" NEWLINE INDENT {{attribute} {{(overlay_fn | overlay_name)}}}
  + …{{attribute} fn_header NEWLINE} DEDENT overlay_fn := {{["unsafe"]}}
  + …[overlay_param {"," overlay_param} [","]] ")" ["->" {{(type {contract} |}}
  - …{"," overlay_param} [","]] ")" ["->" contract [[{contract}]]]
  + …{"," overlay_param} [","]] ")" ["->" contract {{{contract})]}}
  + …["->" contract {contract}] NEWLINE overlay_param := {{[param_word]}}
  - …{contract}] NEWLINE overlay_param := IDENT [":" [[contract]]
  + …{contract}] NEWLINE overlay_param := IDENT [":" {{[type]}}
  + …overlay_param := IDENT [":" contract {contract}] {{member_ref := IDENT ["(" [overlay_param {"," overlay_param} [","]] ")" ["->" type]]}}
  + …no C declaration to read (ODR-147). {{In any other entry a parameter may carry, before its contract words, the word and the type the import gives it from the header's declaration (`owns tex: Texture`, `value: c_int`, `s: *c_char`), by the spelling of `[FFI-8]`'s, `[FFI-17a]`'s and `[FFI-40]`'s tables, so that `c_long` names C++'s `long` on every target; each is checked against the header (`[FFI-12]`), and is written where the entry's name and parameter names fit several members of one name (`[FFI-57]`). After the colon, a word of `[FFI-11]`'s vocabulary is a contract word and any other word is the type; a type of the header whose name is a contract word is first given another (`rename status as Status`), which the message of `E5010` says. A `rename` or a `hide` with a parameter list names one member of such a name (`rename put(v: c_long) as put_long`, `hide put(v: c_long)`, `rename get() -> f64 as get_f64` for two members alike in every parameter), inside a class section one of its members (verdict 131); without one it names every member of the name; a `rename` gives the member a name of its own, outside the set.}}
  - …an earlier overlay's entry for that [[name]]
  + …an earlier overlay's entry for that {{function}}
  - …name (`[FFI-30c]`, ODR-149). An `overlay_class` section [[restates methods]]
  + …name (`[FFI-30c]`, ODR-149). An `overlay_class` section {{holds the entries}}
  - …section restates methods of a C++ [[class,]]
  + …section restates methods of a C++ {{class's members (verdict 131): its methods, which list no receiver, its constructors (`new`) and its static functions,}}
  + …methods of a C++ class, each {{written}}
  - …of a C++ class, each as [[an Ember method line with no body, whose `@ffi(const)`, when written,]]
  + …of a C++ class, each as {{any other entry; `@ffi(const)` above a method's entry}}
  + …that the method only reads its {{object, and an entry without it makes the method one that changes its}}
  - …method only reads its object (`[FFI-40a]`, [[ODR-160). `header_hash`]]
  + …method only reads its object (`[FFI-40a]`, {{ODR-160, verdict 132). An entry outside a class section is a function outside any class (`[FFI-12]`). Ember}}
  + …its object (`[FFI-40a]`, ODR-160). `header_hash` records {{the header each overlay was written for in `ember.overlays`, never in the overlay (`[FFI-55]`). The STRING names}}
  - …`header_hash` records the header the overlay [[was]]
  + …`header_hash` records the header the overlay {{is}}
  - …the header the overlay was written [[for]]
  + …the header the overlay was written {{for, the one the import that applies it names}}
  + …a contract word, an `@ffi(…)` attribute, {{or}}
  - …asserts the function safe to call [[(`[FFI-2]`), or a method line]]
  + …asserts the function safe to call {{or, written `unsafe fn`, states what its callers must make true (`[FFI-2]`)}}

### CLI-20 (p17-toolchain.md; 151 words changed)
  + …line, what it could not rewrite. {{From 0.9.15: the `#! language` directive to the current version; the changes of 0.9.20 refuse programs that were unsafe and rewrite nothing (§H.52 to §H.63).}}
  - …`assert_disjoint` to `check_disjoint` and `assert_disjoint_or_panic` to [[`assert_disjoint`.]]
  + …`assert_disjoint` to `check_disjoint` and `assert_disjoint_or_panic` to {{`assert_disjoint`, a function's type parameters in brackets after its name to a line of their own above it (`fn f[T: Add](x: T)` to `[T: needs Add]` above `fn f(x: T)`, `[GRM-40]`), an `x: ref T` parameter to `x: T`, a `T: Sync` bound to `T: needs ThreadShared`, `for x in xs.iter_mut():` to `for ref mut x in xs:`, and a declaration left unsafe to call by the word `unsafe` to `unsafe fn`: an overlay entry's `unsafe` contract words (verdict 130), and `@ffi(unsafe)`, `param(p, unsafe)` and `result(unsafe)` on a hand declaration (verdict 133), the word taken out and an `@ffi(…)` left empty removed, and a plain `fn` in an `unsafe extern` block (verdict 136). It writes no `@safety` text: `L3015` asks for any that is missing (`[UNS-7]`).}}

### BLD-2 (p17-toolchain.md; 26 words changed)
  + …(signatures, layouts, effect sets, inline bodies, {{the bodies of `pub` generics, which the instantiating package compiles, the bodies `[MONO-10]` needs, which are part of the interface hash as inline bodies are (`[CG-C-3b]`),}}

### PRF-1 (p17-toolchain.md; 56 words changed)
  - …`debug_assert`, checked only where `[PRF-3]` says: [[it may not have side effects (`W2016`),]]
  + …`debug_assert`, checked only where `[PRF-3]` says: {{its argument changes nothing (`E3023`, §VI.6; an effect of its own that changes nothing is `W2016`),}}
  - …computes the same results in every [[profile.]]
  + …computes the same results in every {{profile; a C sanitizer (`[PRF-3]`) is a tool for finding defects in foreign and `unsafe` code, outside Ember's checks: it stops the program only where that code is already outside the guarantee (`[PHIL-10]`), and never changes where Safe Ember stops.}}

### PRF-4 (p17-toolchain.md; 22 words changed)
  - …shift amounts); `assert_disjoint`'s compare of two [[spans;]]
  + …shift amounts); `assert_disjoint`'s compare of two {{spans and the test of a `runtime` name's fact at a line that changes what a call's result may point into (`[LT-48]`);}}

### DIA-21 (p17-toolchain.md; 16 words changed)
  + …in a signature | `E0100` | {{`[Ts...]` above the function and `args: Ts...` (`[TYP-47]`),}}
  - …`Span` parameter, or keyword arguments with [[defaults; Ember has no variadic functions (§I.6)]]
  + …`Span` parameter, or keyword arguments with {{defaults}}
  + …field or a list, or `[T: {{needs}}

### MNG-1 (p18-implementation.md; 285 words changed)
  - …a generic instance `G` and the [[first 16]]
  + …a generic instance `G` and the {{32}}
  - …the canonical spelling of its arguments [[(`[MNG-6]`):]]
  + …the canonical spelling of its arguments {{(`[MNG-6]`), 128 bits, so that no two spellings a program can write collide by chance or by design:}}
  + …is `em_4main`); a dependency's begin with {{one component made of}}
  - …a dependency's begin with its package's [[name.]]
  + …a dependency's begin with its package's {{name, `_v` and its major version from its manifest (`[MAN-8]`; a package whose manifest has no `version` is major 0): `11geometry_v2`, so two majors of one package (`[VER-5]`) never share a symbol, and the same component spells a declared type's path in `[MNG-6]`'s canonical spelling, in interface files, in schemas (`[HR-11]`) and in `ember demangle`'s output, which prints it as `geometry@2.Vec2`. A lambda's path is its enclosing function's, then `lambda`, its ordinal among the lambdas of that function in source order and the hash of its canonical text with its captures (`[MNG-6]`), so that a lambda moved by an edit above it keeps its name and one whose text or captures changed gets a new one, which `[HR-7]` treats as a removal and an addition.}}
  - …apart by the interface it implements [[(`[TYP-24]`).]]
  + …apart by the interface it implements {{(`[TYP-24]`); a member of a C++ name declared several times (`[FFI-57]`) adds, after its name, `O` and the 32 hex digits of the BLAKE3 hash of its declaration as the header spells it (the key of `[FFI-55]`), so that no two members share a symbol or a thunk name.}}
  - …therefore never share a name (`[MOD-1]`). [[A hash collision between]]
  + …therefore never share a name (`[MOD-1]`). {{Beside each instance's symbol the package records the canonical spelling it hashed, in its interface file and in the object's manifest section; before the linker runs, the build compares the spellings of every instance symbol}}
  - …(`[MOD-1]`). A hash collision between two [[instances is reported as]]
  + …(`[MOD-1]`). A hash collision between two {{packages both define, and two that differ are}}
  - …instances is reported as `E9040`, naming [[both,]]
  + …instances is reported as `E9040`, naming {{both spellings and both packages,}}
  - …reported as `E9040`, naming both, never [[as]]
  - …naming both, never as an internal [[error.]]
  + …naming both, never as an internal {{error and never one body for two instances; within one package the same compare runs when the instance is named. The compare runs once per build, over the interface files, and nothing of it is in the program.}}

### MNG-6 (p18-implementation.md; 18 words changed)
  + …T`, `shares mut T`, `weakref T`, {{`weakref thread_shared T`,}}
  - …one, after the arguments (`[a = [[b]`).]]
  + …one, after the arguments (`[a = {{b]`); and, for a function compiled for a reload's prepare (`[HR-35]`), `[reload]` among its arguments.}}

### RT-7 (p18-implementation.md; 133 words changed)
  + …no extra memory. A `thread_shared` value's {{home}}
  + …extra memory. A `thread_shared` value's count {{(`[RC-4]`)}}
  + …memory. A `thread_shared` value's count is {{checked as a one-thread count is; its away count and its weak count are}}
  + …other program keeps the 32-bit count. {{The add of a home count into the away count, at a hand-over (`[RC-4]` point 4) and at the queue's emptying (point 2), is checked as a +1 is: the sum is computed in 64 bits from the home count and the away count read before the add, and when it would be 2,147,483,647 or more the program stops with the same message ("too many owners of a `Log` (2,147,483,647)"); otherwise the home count is added with one atomic add (point 4) or a plain store (point 2, on the making thread, which alone reads both). One compare per hand-over or per queued value, never on the fast path; the 8-bit test program reaches it at 127.}}

### RT-8 (p18-implementation.md; 70 words changed)
  - …* `[RT-8]` [[Both counts of a]]
  + …* `[RT-8]` {{A}}
  - …`[RT-8]` Both counts of a `thread_shared` [[value, its owner]]
  + …`[RT-8]` Both counts of a `thread_shared` {{value's away}}
  - …its owner count and its weak [[count,]]
  + …its owner count and its weak {{count (`[RC-4]`)}}
  - …relaxed increment, one ended an acquire-release [[decrement.]]
  + …relaxed increment, one ended an acquire-release {{decrement; putting a value in its making thread's queue is a release, and emptying the queue an acquire; closing the queue at the thread's end is a release, and the subtract that finds it closed an acquire, so that thread reads the final home count (`[RC-4]` point 3). Its home count is plain: only its making thread reads or writes it.}}

### RT-10 (p18-implementation.md; 147 words changed)
  - …a `weakref` link, and where a [[view of a `thread_shared` object]]
  + …a `weakref` link, and where a {{name}}
  - …`thread_shared` object is opened from a [[`weakref`]]
  + …`thread_shared` object is opened from a {{`weakref thread_shared T`}}
  - …the object is out of line. [[Making]]
  + …the object is out of line. {{On a `thread_shared` value's making thread, until the value is handed over (`[RC-4]` point 4), making or ending}}
  + …out of line. Making an owner {{is a plain add or subtract}}
  + …of line. Making an owner of {{its home count, inlined as}}
  - …line. Making an owner of a [[`thread_shared` value,]]
  + …line. Making an owner of a {{one-thread count's is. On another thread, and after the hand-over, making an owner,}}
  - …value, or a `weakref` link to [[one,]]
  + …value, or a `weakref` link to {{the value,}}
  + …test on the value it returns, {{and ending one is one atomic subtract,}}
  - …value it returns, never a compare-exchange [[loop.]]
  + …value it returns, never a compare-exchange {{loop; a subtract that takes the away count below zero puts the value in its making thread's queue in the same atomic step (out of line, as are the hand-over and the queue's emptying at a check point).}}
  - …never a compare-exchange loop. Opening a [[`weakref`]]
  + …never a compare-exchange loop. Opening a {{`weakref thread_shared T`}}
  - …compare-exchange loop. Opening a `weakref` link [[to a `thread_shared` object]]
  - …one +1 that may find the [[owner count at zero:]]
  + …one +1 that may find the {{object ended: on its making thread}}
  - …the owner count at zero: it [[is a compare-exchange]]
  + …the owner count at zero: it {{tests}}
  - …zero: it is a compare-exchange that [[then fails and]]
  + …zero: it is a compare-exchange that {{the two counts add up to more than zero before adding to the home count; on another thread its atomic add}}
  - …that then fails and finds the [[object ended, never raising a]]
  + …that then fails and finds the {{ended mark and is undone; neither ever raises an ended object's}}
  - …object ended, never raising a count [[from zero]]

### RT-12 (p18-implementation.md; 58 words changed)
  + …never land beyond the guard page. {{A thread C created and attached (`[FFI-22]`) keeps the stack C gave it: that it ends in a guard page is the host's obligation, listed as asserted (`[TCB-1]`) for every `@unsafe(export)` whose `threads=` admits a thread the runtime did not create and every callback contract without `threads(main)`: "stack guard: the host's thread stacks end in a guard page".}}
