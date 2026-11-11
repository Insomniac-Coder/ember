# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 02

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### STR-1 (p05-declarations.md; 76 words changed)
  - …with a default may be omitted. [[It]]
  + …with a default may be omitted. {{Its visibility}}
  + …default may be omitted. It is {{the narrowest among its fields', since construction writes every field:}}
  - …may be omitted. It is `pub` [[iff]]
  + …may be omitted. It is `pub` {{when}}
  - …is `pub` iff every field is [[`pub`.]]
  + …is `pub` iff every field is {{`pub` (a struct with no fields included), `pub(package)` when every field is `pub` or `pub(package)`, and private to its module otherwise (a private, `pub(read)` or `pub(package, read)` field, `[MOD-7]`); a call from where it is not visible is `E1052`, naming the field that narrows it ("`Packet(...)` is not visible here: `seq` is `pub(package)`, so a `Packet` is built only inside the `net` package").}}

### STR-3 (p05-declarations.md; 39 words changed)
  - …`E2080` if a field is not [[`Copy`;]]
  + …`E2080` if a field is not {{`Copy` or the type declares `drop` (`[DRV-1]`: "`Handle` has a `drop` (line 5), which would run once per copy; remove `@derive(Copy)` (line 1), and hand a `Handle` over or write `copy of h` with a `clone` of its own");}}

### STR-5 (p05-declarations.md; 13 words changed)
  - …`Hash`, `Default` and `Display` are never [[implicit]]
  + …`Hash`, `Default` and `Display` are never {{implicit, except that a unit-only enum is `Copy`, `Ord` and `Hash` automatically (`[ENM-3]`),}}

### STR-6 (p05-declarations.md; 17 words changed)
  + …fields through `this`, with no word. {{Before its body runs, every field default is evaluated and stored, as `[CLS-2]` says for a class.}}

### CTL-1 (p06-expressions.md; 54 words changed)
  - …to the loop's name; * an [[`Iterator` value:]]
  + …to the loop's name; * an {{`Iterator`, a place or a temporary:}}
  + …* an `Iterator` value: calls `next` {{on it}}
  - …an `Iterator` value: calls `next` until [[`None`.]]
  + …an `Iterator` value: calls `next` until {{`None`; a place keeps what the loop did not take (`rest = it.next()` after a `break`), and is handed over only by `owns`.}}
  - …a word is `E0116`, `[OWN-13]`). A [[plain]]
  + …`E0116`, `[OWN-13]`). A plain loop name {{that views its element}}
  - …gives `for ref mut v in [[xs:`.]]
  + …gives `for ref mut v in {{xs:`; a loop name under `for x in owns e:` owns its element and may be changed (`[BRW-14]`).}}

### CTL-2 (p06-expressions.md; 74 words changed)
  + …list of the program's own). A {{`for` over a range whose header reads `xs.len()` is a}}
  - …the program's own). A counting loop [[`for i in 0..xs.len():`]]
  + …the program's own). A counting loop {{over `xs`: it}}
  - …i in 0..xs.len():` does not walk [[`xs`:]]
  + …i in 0..xs.len():` does not walk {{`xs`, so}}
  - …not walk `xs`: it may write [[its]]
  - …may write its elements and add [[to it,]]
  + …may write its elements and add {{at the end (`[EFF-24]`'s add-only summary),}}
  - …and add to it, but a [[removal]]
  + …and add to it, but a {{call in its body whose summary may shorten `xs` or replace it}}
  - …to it, but a removal is [[`E3020`;]]
  + …to it, but a removal is {{`E3020` ("`xs.pop()` shortens `xs`, whose length this loop read once on line 3; walk backwards, or collect the indices and remove after the loop"); every other loop walks nothing by its header, and each index is bounds-checked (`[STD-17]`), so}}

### CTL-7 (p06-expressions.md; 83 words changed)
  + …they are not dropped before it. {{Its uses of them are uses at every exit of the block (`[BRW-2]`): a local a `defer` block uses is not handed over below the `defer` line (`E3040`, labelled at the hand-over), and a view it uses is not ended before the block exits (`E3060`).}}
  - …It cannot `return`, `break` or `continue` [[(`E2160`).]]
  + …It cannot `return`, `break` or `continue` {{(`E2160`), nor `yield` (`E2160`: "a `defer` block runs while its block exits, and `yield` would suspend the generator in the middle of that exit; yield before the block exits, or move the `defer` line below the `yield`"; `[CORO-7]`).}}

### BRW-12 (p07-ownership.md; 101 words changed)
  - …silent copy, except that a part [[with no fields]]
  - …a part with no fields of [[its own]]
  + …a part with no fields of {{a `Copy` type}}
  + …own (a number, a `bool`, a {{`char`, a}}
  - …a `bool`, a variant with no [[payload)]]
  + …a `bool`, a variant with no {{payload, a `StrView`, a `Span[T]`, a `ref T`, a `Copy` struct)}}
  - …with no payload) is copied, as [[Python binds]]
  + …with no payload) is copied, as {{`=` copies}}
  - …is copied, as Python binds a [[number.]]
  + …is copied, as Python binds a {{whole name of that type (`[OWN-7]`, `[OWN-9]`): `w = this.rest` for `rest: StrView` is a `StrView` of its own that looks at what the field looks at, so `this.rest = w[i + 1..]` while `w` is used changes nothing `w` reads, and a copied `Copy` struct is not then changed through the new name while the source is used (`[OWN-7]`, `E3023`).}}
  - …first_even(ref mut xs)`), and before a [[`MutSpan`]]
  + …first_even(ref mut xs)`), and before a {{`MutSpan`, or a tuple of them,}}
  - …a call gives it says the [[name]]
  + …a call gives it says the {{names bound from it}}
  - …says the name may write the [[span's]]
  + …says the name may write the {{spans'}}
  - …span's elements (`d = ref mut [[tracks[dst_name].as_mut_span()`);]]
  + …span's elements (`d = ref mut {{tracks[dst_name].as_mut_span()`, `left, right = ref mut xs.split_at(mid)`, `[SPN-3]`);}}

### BRW-14 (p07-ownership.md; 112 words changed)
  - …**Names that keep their place.** A [[parameter,]]
  + …**Names that keep their place.** A {{parameter that is not `owns`,}}
  - …their place.** A parameter, a loop [[name, a]]
  + …their place.** A parameter, a loop {{or}}
  + …a loop name, a pattern name {{that views its value (one under `for x in e:` or `match e:`)}}
  + …through one that is plain (`[BRW-15]`). {{A name that was handed its value owns it, as a local does, and keeps no place: an `owns` parameter (`[FN-1]`), a loop name under `for x in owns e:` (`[CTL-1]`) and a name of the branch that runs under `match owns e:` (`[GRM-13]`) may be changed and given a new value with no word (`job.retries += 1` under `for job in owns jobs:`; `name = String("unnamed")` on `owns name: String` ends the string the caller handed over, as `=` on a local does, `[OWN-3]`), and `kept = job` hands it over.}}

### BRW-15 (p07-ownership.md; 53 words changed)
  - …a plain loop, pattern or opened [[name,]]
  + …a plain loop, pattern or opened {{name (one that views its value; a name handed its value is an owner, `[BRW-14]`),}}
  + …plain local that holds a view {{(a `StrView`, a `Span`, a `ref`, a name for a part, `[BRW-12]`; not a value of its own that holds views, such as an iterator a call made, which the name owns and may advance, `[CTL-1]`)}}
  - …= m.lock():`; `with log.lock: log.lines.push(...)`; `total.fetch_add(n)`; [[`tx.send(v)`;]]
  + …= m.lock():`; `with log.lock: log.lines.push(...)`; `total.fetch_add(n)`; {{`tx.send(owns v)`;}}

### BRW-2 (p07-ownership.md; 2 words changed)
  - …does not matter: `n = v.len(); [[v.push(n)`]]
  + …does not matter: `n = v.len(); {{v.push(owns n)`}}

### BRW-3 (p07-ownership.md; 2 words changed)
  - …computed from it may be passed [[(`v.push(v.len())`);]]
  + …computed from it may be passed {{(`v.push(owns v.len())`);}}

### BRW-11 (p07-ownership.md; 4 words changed)
  - …the notes after: `before = LOG.len()`, [[`LOG.retain(fn(s)]]
  + …the notes after: `before = LOG.len()`, {{`LOG.retain(owns fn(s)}}
  - …LOG.len()`, then `for k in 0..dropped: [[LOG.push(String("dropped]]
  + …LOG.len()`, then `for k in 0..dropped: {{LOG.push(owns String("dropped}}

### BRW-21 (p07-ownership.md; 192 words changed)
  - …the compiler holds the callee to [[it.]]
  + …the compiler holds the callee to {{it: for a plain or `Span` parameter, that what it views is not ended or moved; for a `ref mut` parameter, that its place is reached by no other name (below).}}
  + …code is unseen; `[FFI-52]` covers it. {{A foreign call without `@ffi(no_callbacks)` is an **open line** of the function's summary: for each view parameter, `this` included, and each `static` of its own, that the function uses after such a call (`[FFI-52]`), the summary records the call and what the parameter is used as (walked, read, changed through `ref mut`), without resolving which callbacks run (`[EFF-4]`, `[BLD-14]` item 15). At each call from another package the caller resolves the open lines against the callbacks its program keeps, its exports and its overrides (`[FFI-52]`): where one of them changes what that parameter's argument reaches, the call is `E3031`, naming the chain ("`sim.tick(game)` (line 9) walks `game.names` after it calls `poll_events()` (sim/game.em:9), a C function that may run the callback C keeps from `register_cb` (line 8), which pushes to `game.names`; register the callback after the call, or, if `poll_events` never calls back into Ember, mark it `@ffi(no_callbacks)` in `sim`'s declaration"). Nothing in a library is refused for callbacks it cannot see.}}

### BRW-10 (p07-ownership.md; 4 words changed)
  - …if `bump` touches only `w.count`. `fn [[note(msg:]]
  + …if `bump` touches only `w.count`. `fn {{note(owns msg:}}
  - …`fn note(msg: String):` whose body is [[`this.log.push(msg)`]]
  + …`fn note(msg: String):` whose body is {{`this.log.push(owns msg)`}}

### BRW-16 (p07-ownership.md; 122 words changed)
  - …through locals that own it, parameters, [[`this`]]
  + …through locals that own it, parameters, {{`this`, one-owner fields}}
  - …that own it, parameters, `this` and [[one-owner fields,]]
  + …that own it, parameters, `this` and {{module variables (`static mut`, `[LT-3]`),}}
  + …and one-owner fields, is a position: {{the index is read once, where the name is made, and kept with it (a later change of `i` moves no position made from `enemies[i]`), and}}
  - …element is found again at each [[use.]]
  + …element is found again at each {{use through the list's current buffer.}}
  + …`sharedref mut` link) gives addresses, `[BRW-17]`. {{A list here is an `Array[T]`, whose `xs[i]` and `xs.get(i)` are its own built-in indexing (`[STD-15]`, `[STD-17]`), and an `SoA[T]`'s element proxy is a position likewise (`[SOA-6]`); a name for an element of any other container (a `Deque`, a `SmallArray`, a `Map`'s value, a type of the program's own that implements `Index`) is made by a call of that container's `index`, a view a call gives, and is an address (`[BRW-17]`), which every reshape of the container ends, adding to it included.}}

### BRW-17 (p07-ownership.md; 24 words changed)
  - …unless the compiler knows they differ [[(different types,]]
  + …unless the compiler knows they differ {{(types of which neither can hold a value of the other, for classes neither deriving from the other;}}
  - …they differ (different types, a fresh [[object,]]
  + …they differ (different types, a fresh {{object;}}
  - …mut players[0])`, `w.target.hp -= 1` after [[`players.push(Player())`]]
  + …mut players[0])`, `w.target.hp -= 1` after {{`players.push(owns Player())`}}
  - …a change of their map: E3060 [[("`m.insert(String("z"),]]
  + …a change of their map: E3060 {{("`m.insert(owns String("z"), owns}}

### BRW-18 (p07-ownership.md; 21 words changed)
  - …Array[Enemy] = []` before the loop, [[`spawned.push(Enemy(hp=5,]]
  + …Array[Enemy] = []` before the loop, {{`spawned.push(owns Enemy(hp=5,}}
  - …in place of this line, and [[`enemies.extend(spawned)`]]
  + …in place of this line, and {{`enemies.extend(owns spawned)`}}
  - …Array[String] = []` before the loop, [[`upper.push(k.to_upper())`]]
  + …Array[String] = []` before the loop, {{`upper.push(owns k.to_upper())`}}
  - …and `for u in owns upper: [[counts.insert(u,]]
  + …and `for u in owns upper: {{counts.insert(owns u, owns}}
  - …line 17, `b.click()`, below the loop"; [["`d.log.push("open")`]]
  + …line 17, `b.click()`, below the loop"; {{"`d.log.push(owns "open")`}}
  - …push after `g`'s last use: move [[`d.log.push("open")`]]
  + …push after `g`'s last use: move {{`d.log.push(owns "open")`}}
  - …the generator's last use (`retain`, `extend`, [[`Commands`).]]
  + …the generator's last use (`retain`, `extend`, {{a list of the program's own).}}
  - …(line 3) and line 4, write [[`enemies.retain(fn(e)]]
  + …(line 3) and line 4, write {{`enemies.retain(owns fn(e)}}

### LT-1 (p07-ownership.md; 568 words changed)
  - …it can hold one, so `Map[StrView, [[V].entry(k)]]
  + …it can hold one, so `Map[StrView, {{V].entry(owns k)}}
  - …result points into **all** source parameters [[together (the]]
  + …result points into **all** source parameters {{together. At a call, the}}
  - …source parameters together (the caller keeps [[all of them viewed]]
  + …source parameters together (the caller keeps {{viewed,}}
  - …of them viewed while the result [[lives). Rule 3 always type-checks;]]
  + …of them viewed while the result {{lives, exactly the arguments the called function's code shows the result points into: its summary (`[LT-22]`), read for every result type, so `fn pick(a: Span[int], b: Span[int]) -> Span[int]:` whose body is `return a` keeps only `a` viewed. The three rules decide a call where there is no code to read: a call through a function value (`[LT-7]`), a function with no Ember body. A `dyn` or `virtual` method's call is decided by the code of each kind that may reach it, the types the program gives to the function, followed through its names and calls: with `interface Picker: fn pick(other: Array[int]) -> Span[int]`, `Mine.pick` returning `this.items` and `Yours.pick` returning `other`, `fn show(p: dyn Picker):` doing `r = p.pick(xs)`, `xs.push(owns 3)`, `println(r)` compiles for `show(Mine(items=[9]))`, which prints `[9]`, and is refused at `xs.push(owns 3)` for `show(Yours(items=[9]))` (`E3021`). Where which kind reaches}}
  + …lives). Rule 3 always type-checks; it {{is known only while the program runs (`p: owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])`, then `show(p)`, an `owns dyn Picker` read as the `dyn Picker` the parameter takes, `[TYP-14]`), what one of them makes unsafe is `E3021` ("`xs` changes while `r`}}
  - …Rule 3 always type-checks; it may [[view more than]]
  + …Rule 3 always type-checks; it may {{still look at it: `p` may be a `Yours`, and which kind it is is only known while running", with the help `[LT-48]` gives for the name the call goes through (for a parameter, "to check it while running, write `runtime` on the parameter: `fn show(p: runtime dyn Picker):`"), unless that name says `runtime` (`[LT-48]`). A `virtual` call on a class value (`fn show(a: A):` for an `open class A`, `r = a.first(xs)`), a call through a local whose kind}}
  - …may view more than the function [[needs.]]
  + …may view more than the function {{itself chooses while running (`q: owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])`, then `r = q.pick(xs)` in the same function), and a call on `this` in a method of an open class, are decided by the code of each kind the same way and refused the same way, the message naming the class ("`a` may be a `B`") or the local ("`q` may be a `Yours`"). Across a package's edge the kinds that reach a `pub` function's `dyn`, interface or base-class parameter are its callers' to give, so the package's interface file records, for that parameter, what the body needs of them (`[BLD-14]` item 11): for each call through it, followed through the package's own names and calls, and for each argument of that call and its object, whether the called method's result may point into it, given what the body does while that result lives (`[BRW-1]`, `[BRW-17]`). A call from another package reads, for each kind it gives, the bits `[LT-22]` records in the kind's table (the ones `[LT-48]` reads while running) and refuses the call where a bit is set that the record forbids, with the message of this rule naming the library's line (`E3021`: "`xs` changes while `r` may still look at it (`lib/show.em:9`): `p` is a `Yours`, whose `pick` gives back the list it was handed"). A kind known only while running is `E3021` with the `runtime` help, as within one package.}}
  + …view is used is refused (`[BRW-1]`). {{A returned view may point into `this` and into any source parameter, a method's as well as a function's, with nothing written: `fn pick(other: ref i32) -> ref i32: return other` in a type body is no error, and its callers keep `other` viewed.}}
  - …place (`first(ref mut xs) += 1`; [[`counts.entry(k).or_default()]]
  + …place (`first(ref mut xs) += 1`; {{`counts.entry(owns k).or_default()}}

### LT-1a (p07-ownership.md; 191 words changed)
  + …or write `-> StrView from buf`"). {{Where the body may give a view of what `from` names and of another parameter too (`-> Pair from a` over `return make(a, b)`, `make` a function value), naming the other would be as wrong, and the help says to remove `from`: `E3062` ("the returned view points into `b`, but its first line says `from a`"; help: "remove `from a`: the result may point into `a` and `b`, and with no `from` it points into each of them"). Where the body gives a view of the object itself, which no `from` names (`return ref this.at` under `-> ref int from owner`, `at` a number), the help gives both ways out: "return a view of `owner`, or remove `from owner`", since with no `from` a method's result points into `this` (rule 1).}}
  - …`E2031`. A bare field name names [[a]]
  + …`E2031`. A bare field name names {{any}}
  - …field name names a field of [[`this` whose type is a view,]]
  + …field name names a field of {{`this`,}}
  + …or passed as a copy, `[BRW-8]`): {{for a field whose type is a view,}}
  - …of the object ends with the [[call.]]
  + …of the object ends with the {{call; for any other field (`from log` with `log: Array[int]`), the result points into that field and what it owns, and the object is viewed for as long as the result lives.}}
  - …view of the field write `fn [[next_word(line:]]
  + …view of the field write `fn {{next_word(src_param:}}
  - …a view of the parameter `fn [[next_word(line:]]
  + …a view of the parameter `fn {{next_word(src_param:}}
  - …`fn next_word(line: StrView) -> StrView from [[line:`").]]
  + …`fn next_word(line: StrView) -> StrView from {{src_param:`"; the new name is the parameter's with `_param` after it).}}

### LT-4 (p07-ownership.md; 218 words changed)
  + …and `boss = load_level(ref mut current)`"). {{An unscoped thread, a job outside a scope and a callback foreign code keeps may outlive every instance but a `static` (`[THR-10]`, `[JOB-2]`, `[FFI-21]`), so a value made in an instance, a `thread_shared` one included (`[TYP-46a]`), goes to one of them only when its instance is a `static`, which lives as long as the program (`[LT-3]`) and never changes (`[STA-1]`), so nothing rewinds or frees its memory under the value; from a local, a parameter, a field or a `static mut` (which a line of the main thread may reset while the thread runs, a fact of the program's run that no package compiling that line can see, `[BLD-16]`) it is `E3063` at the hand-over ("`st` was made in `level` (line 7), which ends when `start` returns, and the thread started on line 8 may outlive `start`; make it in the built-in manager, `st = thread_shared Stats()`, in a `static` manager whose `allocate` reads it (a pool that locks inside), or run the thread as a task of a scope that ends before `level`: `with scope = thread.scope():` and `scope.spawn(owns fn() owns st: ...)` in place of line 8").}}
  - …`sharedref mut` link to the instance [[(`[LT-46]`):]]
  + …`sharedref mut` link to the instance {{(`[LT-46]`) for the exclusivity rules, and between threads what `[TYP-46a]` says (a `ref A` to a manager any thread may call, else this link, which never leaves its thread):}}

### LT-46 (p07-ownership.md; 45 words changed)
  - …them may change it at any [[time;]]
  + …them may change it at any {{time (not while a read-only view of what it changes, made through another name, is still used: `E3021`, `[BRW-1]`);}}
  - …whichever holder reaches it (`[BRW-17]`, `[BRW-18]`); [[on two threads]]
  + …(`[BRW-17]`, `[BRW-18]`); on two threads it {{never reaches another thread (it}}
  - …`[BRW-18]`); on two threads it is [[changed]]
  + …`[BRW-18]`); on two threads it is {{neither `Send` nor `ThreadShared`, `[THR-8]`), and the scoped tasks of its thread change its target}}
  - …through `Atomic`, or by a reduction [[from scoped tasks]]

### LT-47 (p07-ownership.md; 2 words changed)
  - …changed is `E3060`. Its message: E3060 [[("`bag.items.push(k)`]]
  + …changed is `E3060`. Its message: E3060 {{("`bag.items.push(owns k)`}}

### LT-22 (p07-ownership.md; 114 words changed)
  + …* `[LT-22]` A function returning a {{view, or a}}
  - …`[LT-22]` A function returning a view [[type gets,]]
  + …`[LT-22]` A function returning a view {{type, gets}}
  - …a view type gets, from its [[body,]]
  + …a view type gets, from its {{body}}
  + …its body, a summary of which {{part of the}}
  - …body, a summary of which result [[field]]
  - …result field points into which parameter [[(and]]
  + …result field points into which parameter {{(a plain view's one part is the view itself; a view type's are its fields), into which module variable (`[LT-3]`), and}}
  - …(and which parameter fields each call [[uses, `[LT-35]`).]]
  + …(and which parameter fields each call {{uses (`[LT-35]`).}}
  - …the summary; source never states it. [[Where]]
  + …the summary; source never states it. {{For an implementation of an interface method usable as `dyn`, and for a `virtual` method and each `override`,}}
  - …never states it. Where the summary [[would need]]
  + …never states it. Where the summary {{is also written as a fact in the method's entry of the kind's table (`[LT-48]`, `[DSP-2]`, `[DSP-3]`). A body may give a result that needs}}
  - …more than the first line allows, [[the result points]]
  + …more than the first line allows, {{pointing}}
  - …line allows, the result points into [[every]]
  + …line allows, the result points into {{any}}
  - …every source parameter (`[LT-1]` rule 3), [[or]]
  + …every source parameter (`[LT-1]` rule 3), {{and its summary says which; with `from`, the result points}}
  + …3), or into what `from` names. {{Where no summary can be made (a view read through a raw pointer), a caller takes the result to point into every source parameter.}}

### SPN-3 (p07-ownership.md; 58 words changed)
  - …the call (`integrate(ref mut p.as_mut_span(), v.as_span(), [[0.5)`);]]
  + …the call (`integrate(ref mut p.as_mut_span(), v.as_span(), {{0.5)`) and before a tuple of spans a call gives, on an `=` line whose targets bind them: `left, right = ref mut samples.split_at(mid)` gives each name the right to write its half's elements, as `d = ref mut a.as_mut_span()` does for one span, and without `ref mut` the names only read (`E3023`, whose help gives that line);}}

### OPT-1 (p08-classes.md; 55 words changed)
  + …run its `drop` at scope end. {{A `thread_shared` value is never placed in a frame: one made with no `using` has its making thread read from its chunk (`[OBJ-1]`), which a frame has not, and one made `using` an instance stays in the block that instance gave, since the instance's `allocate` and `deallocate` are the program's own work (`[RC-4]` point 4).}}

### DSJ-5 (p09-memory.md; 14 words changed)
  - …reason is `establishes_static_fact` (`[EFF-11]`), unless the [[compiler already knows the ranges]]
  + …reason is `establishes_static_fact` (`[EFF-11]`), unless the {{two views}}
  - …compiler already knows the ranges are [[disjoint,]]
  + …compiler already knows the ranges are {{known apart by `[DSJ-11]`,}}
  + …are disjoint, in which case no {{site is recorded and no}}

### DSJ-6 (p09-memory.md; 10 words changed)
  - …in `@noalloc` and `@nosync` code. They [[are]]
  + …in `@noalloc` and `@nosync` code. They {{are, with a `runtime` name's test (`[LT-48]`),}}
  - …and `@nosync` code. They are the [[one]]
  - …They are the one run-time alias [[check]]
  + …They are the one run-time alias {{checks}}
  - …the one run-time alias check there [[is,]]
  + …the one run-time alias check there {{are,}}

### SOA-6 (p12-dod.md; 187 words changed)
  + …`T` of your own (for `T: {{needs}}
  - …ps[0]`"). `ps[i] = v` scatters a [[value.]]
  + …ps[0]`"). `ps[i] = v` scatters a {{value, through a name that may change `ps` (`E3023` otherwise); `ps[i] op= v`, for a `T` with the operator, is the gather, the operation on the gathered value and the scatter, one load and one store per column, the stores that writing its fields one by one would make (the same for a flattened field's proxy, `ps[i].pos += d`, `[SOA-1]`).}}
  + …element through another name is `E3021`. {{An element of a `T` with a `drop` (`[DRP-1]`) is ended as one value: where an element leaves `ps` without being handed over (`pop`'s and `swap_remove`'s result is handed over; `retain`, `clear`, `ps[i] = v` over the old element, and the end of `ps` discard theirs), the compiler gathers its fields from the columns into a `T` and ends that `T` (its `drop` runs once, then its fields end), so `drop` runs exactly once per element. A `T` with a `drop` is otherwise an `SoA` element like any other: its field `ps[i].f` is a place of its column, as `b.f` is a place of `b`. The same holds for a flattened field's struct (`@soa(flatten)`). The gather is a row of §X.4, shown by `ember inspect --cost`.}}

### SIMD-9 (p12-dod.md; 77 words changed)
  + …by the entry test of `[OPT-2]`. {{`load`, `store`, `gather` and `scatter` make no demand on the span's alignment beyond its element type's (`[UNS-4]`): they are the target's unaligned accesses, or a copy into the vector where the target has none. Where the compiler proves the address a multiple of `V`'s alignment (an `SoA` column's base, `[SOA-7]`; a view of an array of `V`) the backend may emit the aligned form; the choice never changes behaviour, and there is no form that requires alignment.}}
