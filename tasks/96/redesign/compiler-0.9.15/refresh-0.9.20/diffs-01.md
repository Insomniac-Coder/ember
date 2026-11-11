# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 01

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### FN-1 (p05-declarations.md; 49 words changed)
  - …it (`[OWN-11]`), and what `a` reads [[does]]
  + …it (`[OWN-11]`), and what `a` reads {{is}}
  - …and what `a` reads does not [[change]]
  + …and what `a` reads does not {{ended or moved}}
  - …not change while `a` is used [[(`[BRW-1]`).]]
  + …not change while `a` is used {{(`[BRW-1]`, `[BRW-21]`); a number another name of the same place writes across the call (a module variable, a name opened `ref mut` from a link, `[WK-11]`) shows through `a`, since the compiler never assumes a value unchanged where it cannot prove the names apart (`[EXC-15]`).}}

### FN-2 (p05-declarations.md; 6 words changed)
  - …it `owns`, and the caller passes [[`copy]]
  + …it `owns`, and the caller passes {{`owns copy}}
  - …x` to keep its own, or [[`x`]]
  + …x` to keep its own, or {{`owns x`}}
  - …own, or `x` to hand it [[over.]]
  + …own, or `x` to hand it {{over (`[FN-2a]`).}}

### FN-2a (p05-declarations.md; 568 words changed)
  - …before an argument the callee may [[change**,]]
  + …before an argument the callee may {{change, and `owns` before one it takes over**,}}
  - …dst: MutSpan[float], src: Span[float]):`, `mem.take(ref mut [[this.items)`.]]
  + …dst: MutSpan[float], src: Span[float]):`, `mem.take(ref mut {{this.items)`; `take(owns xs)` for `fn take(owns xs: Array[int]):`, whatever is given: `take(owns copy of xs)` gives a copy and keeps `xs`, `take(owns [1, 2])` a list made in the call, `m.insert(owns (i * 7), owns i)` a value computed in the call, in brackets (`[GRM-43]`, `[GRM-47]`), `names.push(owns p.name)` a part (`[OWN-11]`), and a function written in the call is given the same way, `xs.iter().for_each(owns fn(x: int): println(x))` (`[STD-19]`).}}
  - …Span[float]):`, `mem.take(ref mut this.items)`. A missing [[`ref mut` or `const`,]]
  + …Span[float]):`, `mem.take(ref mut this.items)`. A missing {{word,}}
  - …reads its argument; remove `ref mut`: [[`show(a)`"),]]
  + …reads its argument; remove `ref mut`: {{`show(a)`"; "`take` takes `xs` over (its first line says `owns xs: Array[int]`); say so at the call: `take(owns xs)`", with the help "to keep `xs`, give it a copy: `take(owns copy of xs)`"; for a computed argument, "`insert` takes this argument over (its first line says `owns key: K`); say so at the call: write `owns` before it, in brackets: `m.insert(owns (i * 7), owns i)`"; "`show` only reads its argument; remove `owns`: `show(xs)`"),}}
  - …`for ref mut p in players:`"). [[An]]
  + …`for ref mut p in players:`"). {{`return a` and a constructor's argument take no `owns`: a value given to a memberwise constructor, or to an `owns` parameter of a written `init`, is handed over as `=` does (`[OWN-9]`; `Emitter(label)` for `fn init(owns name: String):`, after which `label` is `E3040`; `Emitter(copy of label)` keeps it; `Emitter(owns label)` is `E0109`, "`Emitter(...)` already hands `label` over; remove `owns`"); a value given to an `init` parameter with no word is read for the call and stays the caller's, as at every call (`Label(caption)` for `fn init(name: String):`, then `println(caption)` prints); and `ref mut` is written before an}}
  + …in players:`"). An argument for an {{`init`'s `ref mut` parameter as at every call (`Widget(ref mut reg, label)` for `fn init(ref mut registry: Registry, owns name: String):`, `E3028` without it); nor does a value made in the call with a word that makes an owner or a link (`setup(shares mut game)`, `ROSTER.push(weakref this)`, `thread_shared x`, `sharedref mut x`), since that word says what is given (verdicts 147 and 160; `owns shares mut game` is `E0109`). A function from a variable given to a parameter whose type is a type parameter bounded by a function type (`owns f: F` under `[F: needs fn(int)]`, `[CLO-14]`), in any package, is lent for the call: `F` names that very function, the callee runs it and keeps nothing of it, so after `xs.iter().for_each(h)` and after `apply(h)` alike `h` is the caller's still. It takes no word when its functions keep no state or only read it; one whose functions may change their own state is given `ref mut h`, since the call changes it (`[CLO-18]`, `E3023` without the word: "`counter` changes its own `n` each time it runs, and `apply` runs it; say so at the call: `apply(ref mut counter)`"). `owns h` before such an argument is `E0109` ("`apply` runs `h` and keeps nothing of it; remove `owns`: `apply(h)`"); a function written in the call is given with `owns` (verdict 158), and a value the call gives back that keeps the lent function (an iterator adapter, `[STD-19]`) is a value holding a view of `h`, which lives no longer than `h` (`[LT-45]`). A parameter of a callable type (`owns f: fn(int)`) keeps what it is given and takes `owns` as any}}
  - …An argument for an `owns` parameter [[is handed over with no word, for every type (`f(n)` ends `n` when `f` takes `owns x: int`), as `=` hands a value over.]]
  + …An argument for an `owns` parameter {{does.}}
  - …takes no word at the call: [[`top(scores)`").]]
  + …takes no word at the call: {{`top(scores)`"); an `owns` parameter of one of these types takes `owns` as every such argument does (`start_worker(owns rx)` for `fn start_worker(owns rx: Receiver[Job]):`).}}

### FN-3 (p05-declarations.md; 22 words changed)
  + …that is a view points into {{what}}
  - …is a view points into the [[arguments]]
  + …is a view points into the {{function's code shows}}
  - …view points into the arguments it [[may come from, a method's into `this` and a function's into]]
  + …view points into the arguments it {{points into:}}
  - …and a function's into its view [[arguments;]]
  + …and a function's into its view {{arguments and, for a method, `this` (`[LT-1]`);}}

### FN-4 (p05-declarations.md; 84 words changed)
  - …or a free function is `E1010` [[("`starting_at`]]
  + …or a free function is `E1010` {{("`this` can only be used in a method, and `starting_at`}}
  - …is `E1010` ("`starting_at` is a `static [[fn`: it is called on]]
  + …is `E1010` ("`starting_at` is a `static {{fn`, not a method", with}}
  - …fn`: it is called on the [[type, `Counter.starting_at(5)`, and has no object, so no `this`; build]]
  + …fn`: it is called on the {{help "build}}
  - …build the counter from `k`: `return [[Counter(n=k)`");]]
  + …build the counter from `k`: `return {{Counter(n=k)`"; "`this` can only be used in a method, and `double` is not a method", with the help "to make `double` a method, move it into the struct or class it belongs to"). The help that builds the value is given where the function returns its own type and sets each field it names through `this` from its parameters;}}

### FN-6 (p05-declarations.md; 124 words changed)
  - …parameter types and words, their result, [[`unsafe`]]
  + …parameter types and words, their result, {{`once`, `unsafe`, `extern "C"`}}
  - …and words, their result, `unsafe` and [[`thread_shared`.]]
  + …and words, their result, `unsafe` and {{`thread_shared`; `fn(…)` converts to `once fn(…)` and to `unsafe fn(…)`, never the reverse (`E3013` for a `once fn` where a function may run twice, `[CLO-6]`; `E2020` for an `unsafe fn` where a safe one is wanted).}}
  - …"C" fn(…)` when every type is [[FFI-safe.]]
  + …"C" fn(…)` when every type is {{FFI-safe: its parameters and result are those `[FFI-31b]` admits in an export's signature, written with the same words and lowered to the same carriers (`ref mut x: T` as `T*`, `Span[T]` as a pointer and a `size_t`), and any other is `E2020`, naming the parameter; it then gets a C entry that attaches the calling thread (`[FFI-22]`) and runs it, and its body is checked as an export under `threads=any` is (`[FFI-21]`, `[FFI-33]`), since the compiler cannot see which thread C calls it from.}}

### FN-8 (p05-declarations.md; 1 words changed)
  + …-> Result[void, E]` for any `E: {{needs}}

### STA-1 (p05-declarations.md; 129 words changed)
  + …`Mutex`, `RwLock` or `Once` in it. {{An assignment to it (`HOOK = v` in a function) is `E3023`, whose help names `static mut HOOK` or another name for a local; it never makes a local that hides the `static`.}}
  - …only by the main thread and [[its scoped]]
  + …only by the main thread and {{the}}
  + …main thread and its scoped tasks {{and jobs of the scopes it opens (`[THR-20]`)}}
  + …it otherwise is `E7012` (Part XI). {{A `guarded by` `static mut` is a `Mutex[T]` written as a module variable (`[THR-18]`), so its `T` MUST be `Send` (`E7002`: "`CACHE` is guarded by `L`, so any thread may reach it under the lock, and a `shares mut Table` stays on one thread; share it between threads: write `static mut CACHE: thread_shared Table guarded by L = thread_shared Table()`"); an unguarded `static mut` asks nothing of `T`, since one thread and its scoped tasks reach it.}}
  - …it between threads; one read without [[`unsafe`]]
  + …it between threads; one read without {{`unsafe`,}}
  + …read without `unsafe` under `@ffi(immutable)` (`[FFI-10]`) {{or as a header's `const` object (`[FFI-8]`, `[FFI-41]`),}}

### STA-3 (p05-declarations.md; 66 words changed)
  - …with no initialisation code, except a [[`static mut` whose]]
  + …except a `static mut` whose value {{that}}
  - …`static mut` whose value owns heap [[memory, which its functions]]
  + …`static mut` whose value owns heap {{memory (an `Array`, `String`, `Map`, `Set`, an `owns T`, a class object) in a `static mut`, or in a `static` whose type is one a plain name}}
  - …heap memory, which its functions may [[grow]]
  + …heap memory, which its functions may {{change (`[CT-5]`: a `Mutex`, `RwLock`, `Cell`, `Atomic`, `Once`, channel end}}
  - …which its functions may grow or [[free:]]
  + …which its functions may grow or {{`guarded by` field on the path to it, or a class with a `@cached` method):}}
  - …on first access, never in read-only [[memory (`[CT-5]`).]]
  + …on first access, never in read-only {{memory, so that nothing grows, changes or frees constant data.}}

### EXP-4 (p06-expressions.md; 90 words changed)
  - …**condition** of an `if`, `elif` or [[`while`]]
  + …**condition** of an `if`, `elif` or {{`while`, or the **subject** of a `match` (`match owns JOBS.lock().pop():` holds the lock for the pop and lets the guard go before the arm runs),}}
  + …is dropped before the chosen block {{or arm}}
  + …chosen block runs, unless the condition {{or the arm's pattern}}
  - …(`exists as`, a kind test, a [[pattern) and]]
  + …(`exists as`, a kind test, a {{pattern): the temporary}}
  - …a pattern) and the name refers [[to it,]]
  + …a pattern) and the name refers {{to, and every temporary of the condition that the name's value views, directly or through what it holds (`[BRW-19]`: `if names().first() exists as p:` keeps the list `names()` made while `p` views a string}}
  - …the name refers to it, in [[which case it lives]]
  + …the name refers to it, in {{it), then live}}
  - …to the end of that block [[only —]]
  + …to the end of that block {{or arm only,}}
  - …block only — never into an [[`elif`]]
  + …block only — never into an {{`elif`, an `else`}}
  - …— never into an `elif` or [[`else`.]]
  + …— never into an `elif` or {{another arm; the rest are dropped before the block or arm runs.}}

### EXP-6 (p06-expressions.md; 74 words changed)
  + …a field of a class object, {{a part of a value with several owners (`shares mut`),}}
  + …not own it, are `E3013` (`[OWN-11]`). {{The value inside an `owns T` place belongs to that place: handing it out on its own (`m: Mesh = b` for `b: owns Mesh`) is `E3013` too ("`b` holds its mesh in a block of its own; hand the block over, `m = b`, or take the value out and leave one: `mem.take(ref mut b)`"), while reading through it stays a view (`[TYP-14]`).}}

### EXP-9 (p06-expressions.md; 70 words changed)
  - …is b` tests whether two class [[objects (or]]
  + …is b` tests whether two class {{objects,}}
  - …whether two class objects (or two [[views) are]]
  + …whether two class objects (or two {{views, two owners (`shares`, `shares mut`, `thread_shared`) or two links (`weakref`, `weakref thread_shared`, `sharedref mut`) name}}
  - …(or two views) are the same [[object,]]
  + …(or two views) are the same {{object or value,}}
  - …are the same object, by comparing [[addresses.]]
  + …are the same object, by comparing {{addresses; two links that hold nothing are `is` each other, and a link whose target has ended still names its block, which stays while a link points at it (`[WK-11]`), so the compare is exact and reads nothing of the target. A class object's `==` is this test (`[TYP-36]`).}}

### OWN-3 (p07-ownership.md; 45 words changed)
  + …a `Copy` type `=` copies. A {{hand-over is a reshape of the giver's place (§VII.1, `[BRW-17]`): a view into `a` that is still used after `b = a` is `E3060` at that use, and `a` is not handed over while a `ref mut` name of it is still used (`E3022`). A}}

### OWN-6 (p07-ownership.md; 2 words changed)
  - …parameter its declaration says it takes [[(`widget_take(first)`),]]
  + …parameter its declaration says it takes {{(`widget_take(owns first)`),}}

### OWN-9 (p07-ownership.md; 259 words changed)
  - …An argument for an `owns` parameter [[is handed over with no word: `f(n)`]]
  + …An argument for an `owns` parameter {{says `owns` at the call (`[FN-2a]`): `f(owns n)`}}
  - …`f` takes `owns x: int`, and [[`f(copy]]
  + …`f` takes `owns x: int`, and {{`f(owns copy}}
  - …own. `return a` hands `a` over [[too.]]
  + …own. `return a` hands `a` over {{with no word, and so does a constructor's argument, as `=` does, and so does a value made in the call with a word that makes an owner or a link (`setup(shares mut game)`, `ROSTER.push(weakref this)`), whose word says what is given: `w = Words(names=xs)` hands `xs` over, `Point(x=a)` copies `a`, and `Emitter(label)` for a written `fn init(owns name: String):` hands `label` over (an `init` parameter with no word reads its argument for the call only, and one written `ref mut` takes `ref mut` at the call, `[FN-2a]`); an element of a collection literal is given the same way (`xs = [a, b]` hands `a` and `b` over and a later use of `a` is `E3040`; `[copy of a, b]` keeps `a`; a `Copy` value is copied; a part, or a value reached through a name that does not own it, is `E3013` as elsewhere, `[OWN-11]`).}}
  - …a` hands `a` over too. A [[number given]]
  + …a` hands `a` over too. A {{value `=` copies (a number, a `bool`, a `char`, a `StrView`, a `ref T`, a `Span[T]`, a raw pointer, a tuple or fixed array of these, a struct or enum marked `@derive(Copy)`, a unit-only enum) is copied}}
  - …too. A number given with `=` [[is copied,]]
  + …too. A number given with `=` {{too,}}
  - …owns n` is how `=` hands [[a number]]
  + …owns n` is how `=` hands {{one}}
  - …there, and a later use is [[`E3040`.]]
  + …there, and a later use is {{`E3040` (`f = owns flag` ends `flag`).}}
  + …place already hands the value over, {{after `return`,}}
  + …the value over, nor before `fn` {{written on its own, nor before a value made on the line for a place that is not an `owns T` (`[GRM-15]`)}}
  + …already hands `a` over; remove `owns`"; {{"`return` always hands the value over; remove `owns`: `return xs`";}}
  - …on each captured name: `fn() owns [[report:`").]]
  + …on each captured name: `fn() owns {{report:`"); before a function written as an argument for an `owns` parameter it is the call's word (`xs.iter().for_each(owns fn(x: int): println(x))`).}}

### OWN-10 (p07-ownership.md; 15 words changed)
  - …IntPool(items=[1, 2]))` for `pools: Array[owns dyn [[Pool]`;]]
  + …IntPool(items=[1, 2]))` for `pools: Array[owns dyn {{Pool]`, where the one `owns` says both the block and the hand-over to `push` (`[GRM-15]`);}}

### OWN-11 (p07-ownership.md; 79 words changed)
  - …(`push`, `return`, a field, an `owns` [[argument)]]
  + …(`push`, `return`, a field, an `owns` {{argument, `body = owns req.body` on the right of `=`, `[GRM-15]`)}}
  - …own: for a struct value `p`, [[`names.push(p.name)`]]
  + …own: for a struct value `p`, {{`names.push(owns p.name)`}}
  - …value first, or keep a copy: [[`names.push(copy]]
  + …value first, or keep a copy: {{`names.push(owns copy}}
  - …p.name)`". A field of a class [[object]]
  + …p.name)`". A field of a class {{object, and a part of a value that has several owners (`shares mut inv`, `[RC-1]`),}}
  - …since other names may reach the [[object]]
  + …since other names may reach the {{value}}
  - …object and would see the hole [[(`E3013`:]]
  + …object and would see the hole {{(`E3013`, with the same help, `names.push(owns mem.take(ref mut inv.items))`:}}
  - …leave a value in its place: [[`names.push(mem.take(ref]]
  + …leave a value in its place: {{`names.push(owns mem.take(ref}}
  - …own the value is `E3013` too: [["`v.push(template)`]]
  + …own the value is `E3013` too: {{"`v.push(owns template)`}}
  - …your own in the list write [[`v.push(copy]]
  + …your own in the list write {{`v.push(owns copy}}
  - …in the list write `v.push(copy of [[template)`".]]
  + …in the list write `v.push(copy of {{template)`"; for a name opened with `exists as` over a call's result (`while jobs.pop() exists as job:` then `done.push(owns job)`) the help also gives the owning form, `match owns jobs.pop():` with `Some(job):` (`[GRM-13]`).}}

### OWN-12 (p07-ownership.md; 2 words changed)
  - …takes it by an ordinary hand-over, [[`widget_take(first)`:]]
  + …takes it by an ordinary hand-over, {{`widget_take(owns first)`:}}

### OWN-13 (p07-ownership.md; 1033 words changed)
  - …more owner that may also change [[it (`[RC-1]`);]]
  + …more owner that may also change {{it, each written only before a new value, a local or an owner (`[RC-1]`, `E2122`);}}
  - …a new value (`names = thread_shared [[Names()`) or]]
  + …a new value (`names = thread_shared {{Names()`),}}
  + …Names()`) or a value already `thread_shared` {{or `this` in a method of a class it pins (below)}}
  - …that does not keep it alive [[(`[WK-11]`);]]
  + …that does not keep it alive {{(`[WK-11]`), whose type is read from the words on the path to `x`: `weakref thread_shared T` when the value `x` names is a `thread_shared T` (an owner `log: thread_shared Log`, a parameter or a capture of that type, a field of that type crossed last on the path, a name opened from a `weakref thread_shared Log`), and `weakref T` otherwise (a class object with one owner, a `shares` or `shares mut` owner, a `weakref T` link, a name opened from one, a view of any of these), so that `w = weakref log` is a `weakref thread_shared Log` and `cam.target = weakref p` a `weakref Player`, with no second word. **A `weakref T` link is made only to a value that no `thread_shared` value reaches**, so that only this thread can end it and count its links: a value reached through a `thread_shared` value with no `thread_shared` word on the value itself takes no `weakref T` (`E7015`: "`tree.items[0]` is reached through `tree`, which threads share, so another thread may end the item, and `weakref tree.items[0]` would count its links on this thread alone; share the items between threads: write `items: Array[thread_shared Item]` in `Tree` (line 3), after which `weakref tree.items[0]` is a `weakref thread_shared Item`"). In the package the path says it (for a view, what it points into, `[LT-1]`); across a call it is the callee's summary (`[BLD-14]` item 9): a function that makes a `weakref T` link to what a parameter names, `this` included, directly or through a call, records it, and a call that gives that parameter a value reached through a `thread_shared` value is `E7015` at the call, naming the chain ("`f` makes a link to the item `n` names (`w = weakref n`, its line 2), counted on one thread, and `tree.items[0]` is reached through `tree`, which threads share; share the items between threads: write `items: Array[thread_shared Item]` in `Tree` (line 3) and `fn f(n: thread_shared Item):`"). A call through a method another package may implement or override, or a function-typed place it may fill (`[EFF-2a]`), is judged by the two routes of `[WK-7]`: for a kind the caller gives through a `dyn`, interface or base-class parameter, the call applies that kind's own record of the links it makes (item 9) as the function's (`[BLD-14]` item 11: a library's `visit(ref mut v: dyn Visitor, n: Item)` whose body is `v.see(n)`, given a `Keeper` whose `see` does `this.last = weakref n` and `tree.items[0]` of a `thread_shared` tree, is `E7015` at that call, naming `Keeper.see`'s line); for a kind a place holds, the line's record names each argument reached through a `thread_shared` value, and a member that makes a `weakref T` link of that parameter is `E7015` at the line that adds it (`[MOD-9a]`); a kind the judging package cannot see counts as making the link. Inside a method `this` carries no word, so its link says it: `weakref this`, and a `weakref` of anything reached through `this` with no `thread_shared` word on the way (`weakref this.root`), is a `weakref` link to an object on one thread and says that every object of the class is on one thread; `weakref thread_shared this`, the one place `weakref thread_shared` is written before a value, is a `weakref thread_shared C` and says, as `thread_shared this` does, that every object of the class is `thread_shared`. **A class's kind is read from its own methods** and recorded in its interface (`[EFF-4]`, `[BLD-14]` item 8): one-thread when a method of it makes a one-thread link of `this` (`weakref this`, `weakref this.f` with no `thread_shared` word on the way, `sharedref mut this`; an owner of `this` on one thread, `shares this` or `shares mut this`, is `E2122` in every method, `[RC-1]`); `thread_shared` when a method of it writes `thread_shared this` or `weakref thread_shared this`; neither otherwise, and such a class serves both kinds (`Log`, Annex A). A method pins the class that declares it and every class derived from it, whose objects run it; a class pinned both ways, by its own methods or with a base's, or pinned one-thread while it or a base promises `Send` (`[THR-8]`), is `E7015` at the one-thread line ("`Node.add_child` (line 9) links its nodes on one thread (`weakref this`), and `Node.share` (line 12) shares them between threads (`thread_shared this`); a class's objects are one or the other: for nodes shared between threads write `child.parent = weakref thread_shared this` on line 9, for one-thread nodes remove `share`"). A line that makes an object of the other kind is refused there: `thread_shared Node()` for a one-thread class is `E7015` ("`Node` keeps links to its nodes on one thread (`add_child`, line 7: `child.parent = weakref this`), and a `thread_shared Node` may be reached from any thread; for nodes shared between threads write `parent: weakref thread_shared Node = None` (line 4) and `child.parent = weakref thread_shared this` (line 7), and make every node `thread_shared Node()`"), and `Node()` for a `thread_shared` class is `E7017` ("every `Node` is shared between threads (`add_child`, line 9: `child.parent = weakref thread_shared this`), and line 14 makes one with one owner; write `root = thread_shared Node()`"). `weakref thread_shared x` before any value but `this` is `E0109` where `x`'s type already says `thread_shared` ("`log` is a `thread_shared Log`, so a link to it is a `weakref thread_shared Log` without the word; write `w = weakref log`") and `E7017` where it does not ("`names` was not created `thread_shared`, so no link to a shared value can name it; write `w = weakref names`, or create it shared: `names = thread_shared Names()`"). `weakref` before a new value (`weakref Player()`) is `E2119` ("the player would end at the end of the line and the link would find nothing; keep it in a name first: write `p = Player()` above, then `cam.target = weakref p`");}}
  + …mut vel` to change it too"). {{A `weakref x` whose target was made `using` an instance (its type says `using A`, `[TYP-46]`) is a `weakref T using A`, so the kind of manager travels with the link, and a `weakref` of a `Global` value is a plain `weakref T`.}}

### OWN-7 (p07-ownership.md; 15 words changed)
  - …is never `Copy` (ODR-051). A `Copy` [[struct]]
  + …is never `Copy` (ODR-051). A `Copy` {{value with parts (a struct, a tuple, a fixed array)}}
  + …used afterwards is not then changed {{through its parts or whole}}

### OWN-8 (p07-ownership.md; 129 words changed)
  - …= copy of player`); for a [[`shares`]]
  + …= copy of player`); for a {{`shares`, `shares mut`, `thread_shared`}}
  - …mut` link, a copy of its [[target.]]
  + …mut` link, a copy of its {{target, a `T` with one owner (`p2 = copy of p` for `p: shares mut Player` is a `Player` that `p2` alone owns; `p3 = shares p2` starts a second group of owners).}}
  - …= copy of n`"; * a [[value of]]
  + …= copy of n`"; * a {{`ref mut T` view,}}
  - …n`"; * a value of a [[view type (`StrView`, `Span[T]`,]]
  + …n`"; * a value of a {{`MutSpan[T]`, or}}
  - …type (`StrView`, `Span[T]`, a struct holding [[a `ref mut` field): "`w` is a]]
  + …type (`StrView`, `Span[T]`, a struct holding {{one, which have no `Clone` (`[TYP-36]`), since no second changing}}
  - …field): "`w` is a view of [[`line`'s text; to make your]]
  + …field): "`w` is a view of {{a place can be made: "`d` is the one name that writes `xs`'s elements, and a copy of it would be a second; for a copy of the elements write `copy of xs`"; a read-only view (`StrView`, `Span[T]`, `ref T`), and a `Copy` struct holding one, is `Copy` (verdict 68), so `copy of w` is one more view of the same text, as `w2 = w` is, and text of one's}}
  - …`line`'s text; to make your own [[copy write `String(w)`";]]
  + …`line`'s text; to make your own {{is `String(w)`;}}

### DRP-1 (p07-ownership.md; 26 words changed)
  - …its first line says nothing about [[that.]]
  + …its first line says nothing about {{that; what is `let` stays unchanged in it too (`[CLS-9a]`), since a view field `from` a `let` field is still read while `drop` runs.}}
  - …that. Calling it explicitly is `E3070`; [[`mem.drop(x)`]]
  + …that. Calling it explicitly is `E3070`; {{`mem.drop(owns x)`}}

### DRP-8 (p07-ownership.md; 48 words changed)
  + …(a log line, leaving a list). {{The one end no line writes, a queued `thread_shared` value's at a check point, is the end of a plain type (`[RC-4]` point 4), whose `drop` changes nothing outside its object, and `[RC-4]` point 3 says what it is for the rules that judge the lines around it.}}

### HEAP-7 (p09-memory.md; 25 words changed)
  - …* `[HEAP-7]` [[`weakref T` links to]]
  + …* `[HEAP-7]` {{A `weakref` link names}}
  + …value that lives on its own: {{`weakref T`}}
  - …lives on its own: a class [[object,]]
  + …lives on its own: a class {{object with one owner or `shares` owners,}}
  - …class object, or a value held [[`shares` or `thread_shared`.]]
  + …class object, or a value held {{`shares`, all on one thread; `weakref thread_shared T` a `thread_shared T` (`[WK-11]`).}}

### HEAP-10 (p09-memory.md; 29 words changed)
  - …`[HEAP-10]` A `shares` owner, and a [[`weakref` link to a value held `shares`,]]
  + …`[HEAP-10]` A `shares` owner, and a {{`weakref T` link,}}
  - …across threads is `thread_shared T`, whose [[counts,]]
  + …across threads is `thread_shared T`, whose {{counts are kept for several threads (its making thread's owners with plain instructions, the other threads' owners and}}
  - …T`, whose counts, the weak count [[included, change]]
  - …weak count included, change with atomic [[instructions;]]
  + …weak count included, change with atomic {{ones, `[RC-4]`);}}

### ARN-1 (p09-memory.md; 162 words changed)
  - …longer than the arena (`[LT-1]`, `[LT-44]`). [[`reset()`, `scope()` (`[ARN-6]`)]]
  + …longer than the arena (`[LT-1]`, `[LT-44]`). {{`reset()`}}
  - …`scope()` (`[ARN-6]`) and the arena's end [[may]]
  + …later use of one is `E3060`. {{A scope (`[ARN-6]`) ends no view made before it: while it is live the parent, and every view into it, is not used (`E3096`, since the scope holds a `ref mut` view of the arena, `[LT-46]`), and its end reuses only the scope's own bytes, ending the views into the scope, which live no longer than it in any case (`[LT-1]`). These facts are the compiler's own for the types of `std.arena` (`Arena`, `FixedArena`, `ScopedArena`, `ThreadArena`), which it recognises by their module as it does the types it declares itself (`[TYP-43]`), not by their names or their code. No other type has them: a method of the program's own type that returns a view of its object is an ordinary method, whose result points into `this` (`[LT-1]`) and is exclusive with it while used (`[BRW-1]`, `E3021`), and a manager of the program's own is used through `Allocator` and `using` (`[ALC-1]`), whose values are places of their own (`[ALC-5]`).}}

### ARN-2 (p09-memory.md; 30 words changed)
  + …of a type that needs drop {{(Appendix F)}}
  - …arena, which frees nothing one by [[one.]]
  + …arena, which frees nothing one by {{one; such an object keeps its block for its `weakref` links as any object does (`[OBJ-3]`), so the links end before the arena is reset or ends (`[ALC-5a]`).}}

### ARN-3 (p09-memory.md; 56 words changed)
  + …not to need drop and `T: {{needs}}
  + …not to need drop and `T: {{needs}}
  + …(ODR-064). `alloc_uninit[T](n) -> MutSpan[MaybeUninit[T]]` initialises nothing. {{Each has a `try_` form for an arena that may be full (`[ARN-4]`): `try_alloc(owns v: T) -> Result[ref mut T, owns T]` gives `v` back as the `Err` when the arena is full, and `try_alloc_array[T](n)`, `try_alloc_zeroed[T](n)` and `try_alloc_uninit[T](n)` give `None`; `alloc(owns v)` on a full arena stops the program before it touches `v` (`[COST-6]`).}}

### ARN-4 (p09-memory.md; 52 words changed)
  - …`Arena` carries `Alloc` on its growth [[path.]]
  + …`Arena` carries `Alloc` on its growth {{path, and grows by taking a new chunk: nothing it has given out moves, so every outstanding view stays where it is.}}
  - …carries no `Alloc` on allocation, and [[panics]]
  + …carries no `Alloc` on allocation, and {{stops the program}}
  - …on allocation, and panics on exhaustion [[(`try_alloc` returns `Option`).]]
  + …on allocation, and panics on exhaustion {{("`fixed.alloc_zeroed(...)` on line 9: `fixed` has 16 bytes free and 32 were asked; use `try_alloc_zeroed` for an `Option`"; the `try_` forms of `[ARN-3]` give a value instead).}}

### ARN-7 (p09-memory.md; 9 words changed)
  - …No operation lowers an arena's allocation [[pointer or]]
  + …No operation lowers an arena's allocation {{pointer,}}
  + …allocation pointer or reuses its bytes {{or moves what it has given out}}

### ALC-1 (p09-memory.md; 566 words changed)
  + …or `thread_shared` value, a class object, {{a standard collection whose storage is on the heap:}}
  - …value, a class object, an `Array`, [[`Map`]]
  + …value, a class object, an `Array`, {{`Map`, `Set`, `String`, `Deque`, `SoA`,}}
  - …class object, an `Array`, `Map` or [[`Set`)]]
  + …class object, an `Array`, `Map` or {{a `SmallArray` beyond its `N`; a `thread_shared` value's block then comes from the instance and never from the runtime's chunks, so it is counted away from its making line, `[RC-4]` point 4, `[TYP-46a]`)}}
  - …and `Array.new_in(current)` makes an empty one. [[A]]
  + …and `Array.new_in(current)` makes an empty one. {{`using x` names the instance through a name that may change it (its owner, a `ref mut` name, or a `sharedref mut` link) where its `allocate` changes the instance (`[EXC-15]`, `[ALC-5]`), and through any name where its `allocate` reads it (a `static`, `[STA-1]`; a manager any thread may call, which locks inside, `[TYP-46a]`); through a plain parameter of a manager whose `allocate` changes it, `E3023` ("`allocate` changes the arena (`LevelArena.allocate`, its line 4), and `current` only reads it; write `fn fill(ref mut current: LevelArena)`"); a}}
  + …one. A manager with one instance {{is a `static` whose `allocate` reads `this` (`[STA-1]`). Whether a `pub` manager type's `deallocate` is empty, and whether it implements `FoundByAddress`, is recorded in its package's interface (`[BLD-14]` item 14), the same for every package that writes `using` it. A manager type with no fields has one instance}}
  - …A manager with one instance (`Global`, [[`System`)]]
  + …A manager with one instance (`Global`, {{`System`, and any such type a program declares): it}}
  - …`System`) needs none on the making [[line.]]
  + …`System`) needs none on the making {{line, and its values keep no instance word; every other manager type may have several instances.}}
  + …to the instance it came from {{(`[LT-4]`), except in two kinds of manager, whose values keep no word (verdict 107): (1) a manager that frees nothing one by one, whose `deallocate` does nothing (an empty body, checked when compiling): nothing it hands out is given back value by value, and all of it is freed together when the instance is cleared or ends; a value that grows (an `Array`, `Map`, `Set`, `String`, `Deque`, `SoA` or `SmallArray`) made in it keeps the word, since its growth goes back to the instance; (2) a manager that implements `FoundByAddress` (`[ALC-2]`), whose values' instance is found from their address, as C's `free(p)` finds its heap. Neither changes what is checked: that no value outlives its instance or is used after a reset is checked when compiling and never reads the word}}
  + …the instance it came from (`[LT-4]`). {{Between threads a value made in an instance follows its manager: for `Send` and `ThreadShared` it holds a `ref A` to the instance when `A` is a manager any thread may call, else the `sharedref mut` link of `[LT-4]`, and its growth and end are changes of the instance for `[THR-11]` where they call its methods (`[TYP-46a]`). An instance held inline in a struct beside values made in it would be a view into another field of the same value (`[LT-25]`, `E3024`) and would move with the struct: such an instance is held in a block of its own (`arena: owns LevelArena`), or the values are made `using` an instance the struct does not hold; a manager with one instance needs none of this.}}
  + …using A` and `Array[T] using A`. {{A class whose objects a program makes `using A` is, with that kind, a type of its own, as `Array[T] using A` is: `Node using Arena` is distinct from `Node`, its methods are compiled for that kind (its summaries recorded per kind, `[EFF-4]`), inside them `this` carries the kind, and a field of the class written with the class's own type (`parent: weakref Node`, `children: Array[Node]`) takes the kind of the object that holds it, so `weakref this`, `thread_shared this` and `Node() using current` in a method all name one kind, read from `this`; a field written `using B` names `B` whatever kind the object is.}}

### ALC-2 (p09-memory.md; 111 words changed)
  + …given back or the instance ends. {{`std.mem` also declares `pub unsafe interface FoundByAddress`, with no methods, and `const PIECE: usize = 65536` (verdict 107). Implementing it (`unsafe extend PoolArena implements FoundByAddress:`) promises that every block the manager hands out lies in a piece of memory that starts at a multiple of `PIECE` bytes and holds the instance's address in its first word, within 64 bytes the manager never hands out, a value bigger than a piece taking a piece of its own; a value's instance is then read from the start of its piece, one mask and one load, and the value keeps no word (`[ALC-1]`). A manager whose `deallocate` does nothing needs no such promise (`[ALC-1]`).}}

### ALC-5 (p09-memory.md; 74 words changed)
  - …still lives; end it first: write [[`mem.drop(names)`]]
  + …still lives; end it first: write {{`mem.drop(owns names)`}}
  + …under a value made in it. {{A `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4), so the rule is checked as written; one goes to an unscoped thread, a job outside a scope or a callback foreign code keeps only from a `static` instance (`[LT-4]`, `E3063` otherwise), which never changes (`[STA-1]`), so no change of this rule's kind is ever made to it and nothing is checked after the hand-over.}}

### MONO-10 (p18-implementation.md; 263 words changed)
  + …with the call that needs it. {{The copy is compiled by the caller's package from the callee's body: for a function of its own package, from the source; for a `pub` function of another package, from that package's interface file, which carries the body of every `pub` function that has two or more `ref mut` parameters, or one `ref mut` parameter and a `pub static mut` it uses by name (`[BLD-2]`, as `[CG-C-3b]`'s header carries inline bodies), the copy's body the instantiating package compiles, as a generic instance's is (`[BLD-14]`). The copy is made down the chain: every function the copy calls with the merged place under a `ref mut` parameter, and which reaches the other name while that parameter is used (its summary, `[BCK-8]`), is called through a copy of its own with the same pair made one, an instance like the first; `ember inspect` lists the chain. Where no body can be had, a call through a function value, an interface, `dyn` or `virtual` method, or of an `extern` function, a call that names one whole variable twice is `E3022` ("`f(ref mut x, ref mut x)` names `x` twice, and `f` is a function value, which may hold any function, so no copy of it can be made for one variable; call the function by its name, `add_both(ref mut x, ref mut x)`, or give the second name a value of its own: `y = x` above the call and `f(ref mut x, ref mut y)`"; "`c_add(ref mut x, ref mut x)` names `x` twice, and `c_add` is C code, which has no copy for one variable").}}
