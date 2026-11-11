# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 04

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### MOD-9 (p05-declarations.md; 61 words changed)
  - …line that may end what they [[view;]]
  + …line that may end what they {{view, which it makes a one-thread `weakref` link to, which it stores an owner of into a place of which other, and what it needs of the kinds that reach a `dyn` parameter; for a class, whether its methods say its objects are on one thread or `thread_shared`;}}
  - …and the callbacks it hands to [[C)]]
  + …and the callbacks it hands to {{C; each type's layout, with whether its values carry the weak count, `[OBJ-1]`)}}

### CLS-2 (p05-declarations.md; 138 words changed)
  - …of it is made (`weakref this`, [[`shares]]
  + …of it is made (`weakref this`, {{`thread_shared}}
  + …the compiler works out from that {{body, and for a `virtual` method from every override's}}
  - …compiler works out from that body [[(`[EFF-24]`).]]
  + …compiler works out from that body {{the program has (`[DSP-5]`, `[EFF-24]`), since a base `init` cannot know which kind it builds. In a class that is `pub` and `open` or `abstract`, an override of a `virtual` method may be any dependant's, which this package never sees (`[BLD-16]`): its `init` calls a `virtual` method only after every field is set, and a call before that is `E2100` ("`init` calls `on_make()` (line 7) before `name` is set (line 8), and `Script` is `pub` and `open`, so any package may override `on_make` with a body that reads `name` or keeps a link to the object; set every field first: move line 7 below line 8"); in any other class every override is the package's own, and the check reads their bodies.}}
  + …once every field that method's body {{(every override's, for a `virtual` method)}}

### CLS-4 (p05-declarations.md; 147 words changed)
  + …final unless declared `open` or `abstract`. {{An `open` or `abstract` class that another package may derive from says in its header whether any thread may hold its objects, `implements Send`, `implements ThreadShared`, or an interface that needs them, a promise every class derived from it keeps (`[THR-8]`, `[THR-9]`); a final class's markers are read from its fields.}}
  + …it is `E2110`, `override` or not; {{an `override` has the first line of the method it replaces after its name: the same parameter names, words and types (a generic base's type parameters standing for the arguments the derived class gives them), the same words on `this` and the same result, since a call through the base's slot (`[DSP-2]`) passes what the base declared and reads what it promised, and one that differs is `E2113` ("`KeyLog.on` takes `e: KeyEvent`, and the `Handler.on` it overrides takes `e: Event`; write `override fn on(e: Event):` and test the kind inside: `if e is KeyEvent as k:`");}}

### CLS-6 (p05-declarations.md; 13 words changed)
  + …ended before its `drop` runs (`[WK-3]`). {{The kept results of the object's `@cached` methods (`[CLS-12]`) end with its fields.}}

### CLS-7a (p05-declarations.md; 10 words changed)
  - …no owner of it is made [[(`shares this`, `thread_shared this`):]]
  + …no owner of it is made {{(`thread_shared this`; `shares this` is refused in every method, `[RC-1]`):}}

### CLS-9a (p05-declarations.md; 51 words changed)
  - …through a `let` field, for every [[type:]]
  + …through a `let` field, for every {{type, inside `drop` included (`[DRP-1]` waives only the word `changes this`; a view field `from` a `let` field is still read in `drop`, `[GRM-46]`, and a `let` field that must be flushed or closed does that in its own type's `drop`, which runs when the field ends, after the object's, `[CLS-6]`):}}

### CLS-12 (p05-declarations.md; 490 words changed)
  + …* `[CLS-12]` **`@cached`.** `@cached` above a {{class}}
  + …method's result after the first call, {{in the object (a struct or enum has no hidden state, `[PHIL-9]`, `[TYP-11]`),}}
  + …use is `E3060`). On an object {{of a class whose objects}}
  + …`E3060`). On an object several threads {{may}}
  + …On an object several threads reach, {{read from the class's declaration (it promises `ThreadShared` or implements an interface that needs it, `[THR-8]`, or its methods pin it `thread_shared`, `[OWN-13]`; never from which values a program makes `thread_shared`, `[BLD-16]`),}}
  + …is filled once under a once-flag. {{A class with a `@cached` method that declares none of these is not `ThreadShared` (`[THR-9]`): no object of it is made `thread_shared` or held by a `static` (`E7015`, `E7002`, naming the `@cached` line), since the stores below would race.}}
  + …giving the `@cached` line to remove: {{on a method of a struct or an enum ("a `Vec2` has no hidden state to keep a result in; keep it in a field of a class, or remove `@cached` (line 2)");}}
  - …or `Cell` field, or a field [[other threads]]
  + …or `Cell` field, or a field {{another thread may}}
  + …or a field other threads change {{by its own operations,}}
  - …field other threads change under a [[lock, whose changes need]]
  + …field other threads change under a {{lock or}}
  - …a lock, whose changes need not [[pass through its]]
  + …a lock, whose changes need not {{(an `Atomic`, a `Mutex`, an `RwLock`, a channel end, a field `guarded by` a lock): on an}}
  + …need not pass through its object {{of such a class a kept result is filled under a once-flag and never cleared, so the method reads only fields that never change after `init`}}
  - …could be stale; remove `@cached` (line [[18)").]]
  + …could be stale; remove `@cached` (line {{18)"; "`summary` reads `hits`, an `Atomic` other threads change, and a kept result on a shared object is never cleared; remove `@cached` (line 6)"); and on a `virtual` method or an `override` ("`label` is `virtual`, and a kept result follows one body; keep it in a method no subclass replaces, or remove `@cached` (line 3)").}}
  + …be stale; remove `@cached` (line 13)"). {{A change made by foreign code, or through a raw pointer, passes through no name the compiler sees: unsafe code that changes a field a `@cached` method reads answers for it, as for every obligation of `unsafe` (`[UNS-4]`). A `@cached` body reads nothing a link may change without passing through the holder: reading a field of a class object reached through a field or element of `this` (not `this` itself), of a class that carries a weak count (`[OBJ-1]`), is `E2117` ("`first_name` reads `items[0].name`, and a `Book` may be changed through a `weakref Book` link, made from any name before or after the book entered `shelf` (`[WK-11]`) and opened `ref mut` (`[GRM-44]`), without `Shelf` seeing it, so a kept result could be stale; remove `@cached` (line 7), or keep the names in `Shelf` itself: `names: Array[String]`"); a class that carries no weak count is named by no link, and a `sharedref mut` link or `shares mut` owner into the read set is refused where it is made (`[EFF-27]`). `@cached` is `E2117` too on a method whose body reads anything not reached through `this` that may change (a `static mut`, or a `static` `Mutex`, `RwLock`, `Atomic`, `Cell`, channel end or `guarded by` variable, `[STA-1]`, directly or through a call, `[EFF-24]`), since a change of it passes through no name of the object and clears nothing ("`area` reads `SCALE`, a `static mut` that may change without `Circle` seeing it, so a kept result could be stale; remove `@cached` (line 5), or keep the scale in the object: `scale: float`").}}

### IFC-5 (p05-declarations.md; 106 words changed)
  - …mut v: dyn I` may change [[it).]]
  + …mut v: dyn I` may change {{it), and `dyn I` with no owner word is written only as a parameter's type: a view of an interface value anywhere else (a local, a field, an element, a result) is `ref dyn I`, or `ref mut dyn I` to change it, which `[TYP-15]` and `[TYP-34]` govern as every view (`s: ref dyn Shape = ref sq`; `-> ref dyn Shape from xs`), and a bare `dyn I` in such a position is `E2173` ("`main: dyn Shape` does not say whether `Scene` owns or views its shape; write `main: owns dyn Shape` to own it, or `main: ref dyn Shape` for a view").}}
  + …in instead of returning it, `[V: {{needs}}
  + …parameter instead of `dyn Inspect`: `[I: {{needs}}

### IFC-6 (p05-declarations.md; 83 words changed)
  - …A function that is not `pub` [[may leave]]
  + …A function that is not `pub` {{writes all of}}
  - …may leave its type parameters' bounds [[unwritten:]]
  + …may leave its type parameters' bounds {{or none. With none,}}
  + …its bound is written: write `[T: {{needs}}
  + …total(a: T, b: T) -> T:`"). {{A function that is not `pub` and writes some of them writes every one its body needs: a list that leaves one out is `E2040`, whose message gives the whole line (`[T: needs Lookalike]` above `fn direct_add(a: T, b: T) -> T:` with `return a + b`, where `Lookalike` declares an ordinary `fn add` and is not `Add`, gets `[T: needs (Lookalike, Add[T, Output = T])]`); nothing is added to a written list (verdict 113).}}

### CLO-1 (p06-expressions.md; 90 words changed)
  + …* `[CLO-1]` A {{named function, a}}
  - …* `[CLO-1]` A lambda [[or]]
  + …* `[CLO-1]` A lambda {{and a}}
  - …`[CLO-1]` A lambda or local function [[has]]
  + …`[CLO-1]` A lambda or local function {{each have}}
  - …lambda or local function has a [[unique]]
  - …function has a unique type of [[its own,]]
  + …function has a unique type of {{their own (ODR-278),}}
  - …its own, which implements its callable [[type.]]
  + …its own, which implements its callable {{type; one that keeps no state is a value of size zero.}}
  + …**function value** of that type (`[CLO-10]`). {{Where two different functions meet in one place (`if c: f = a else: f = b`, a second assignment, a conditional's two branches, a collection's elements), the place has the callable type both convert to. A local holding a function that keeps no state is not captured: a lambda that uses it keeps nothing for it (`[CLO-2]`), since giving it the same function again changes nothing.}}

### CLO-2 (p06-expressions.md; 4 words changed)
  - …the element of that round. So [[`b.on_click(fn():]]
  + …the element of that round. So {{`b.on_click(owns fn():}}
  - …1)` changes the real `score`, and [[`b.on_click(fn()]]
  + …1)` changes the real `score`, and {{`b.on_click(owns fn()}}

### CLO-14 (p06-expressions.md; 45 words changed)
  + …the line above the function: `[F: {{needs}}
  - …function changes, worked out from it [[(`[EFF-2]`).]]
  + …function changes, worked out from it {{(`[EFF-2]`); `owns f: F` and `f: F` are one first line, since both run the function given and keep nothing of it, and `ref mut f: F` is not written (the call says `ref mut` where the function changes its own state, `[FN-2a]`).}}
  + …away takes it this way (`[F: {{needs}}

### CLO-10 (p06-expressions.md; 440 words changed)
  - …state of one word or less [[may be]]
  + …state of one word or less {{that the function's body never writes is}}
  - …the state word itself, with no [[block;]]
  + …the state word itself, with no {{block and no `Alloc`: a `copy of` of a `Copy` value it only reads, or an owner or a link, whether the body only reads through it or changes its target through it (a change of the target through a `shares mut`, `thread_shared` or `sharedref mut` owner or link, or under a lock, writes the target and not the word, and every copy of the two words reaches the same target; such a capture never needs a block, and a call through it is a change of the target, `[EFF-24]`, not of the function value's own state, `[CLO-18]`) (a body writes its state when, in the body or in a function it calls with the captured name, the name is assigned or augmented, named `ref mut`, given to a `ref mut` or `owns` parameter, handed over, or has a method called on it that changes it, worked out as `[EFF-24]` works out what a function changes); such a state lives in a block of its own, so that a call through a copy of the two words (a list element, a field, `[BRW-1]`) changes the state the holder keeps; which of the two a function value has is decided from its captures and its body alone, the same in every implementation (`[PHIL-13]`), its `Alloc` is at the line that makes the function value, which `ember inspect --alloc` lists, and `E4001` there names it ("`counter` is `@noalloc`, and the function made on line 4 keeps `n`, which it changes (`n += 1`), in a block of its own; keep the count in the caller's variable instead: `fn counter(ref mut n: int)`");}}
  - …real variables it uses live, which [[costs]]
  + …real variables it uses live, which {{is one address whatever their number: that of the frame of the function that made it, through which every real variable it uses is reached (a variable of an enclosing function through that frame's own state word), and which `[CLO-4]` keeps alive for the value's whole life. A function value that uses real variables and captured}}
  - …it uses live, which costs nothing [[beyond their addresses.]]
  + …it uses live, which costs nothing {{with a word keeps that address in its state word, with no block and no `Alloc`; one that uses real variables and also captured something with a word keeps the frame's address and its captures together in a block of its own, since the state word holds one of them (`Alloc` at its line; in a `@noalloc` function `E4001`: "`fall` is `@noalloc`, and the function made on line 4 uses the real `dt` and keeps its own copy of `k`, so its state is two words and lives in a block of its own; use the real `k` too: `fn(ref mut p): p.vy *= k * dt`").}}

### CLO-4 (p06-expressions.md; 2 words changed)
  - …owns count: shares mut Counter):` and [[`b.on_click(fn()]]
  + …owns count: shares mut Counter):` and {{`b.on_click(owns fn()}}

### CLO-6a (p06-expressions.md; 126 words changed)
  + …* `[CLO-6a]` A `once fn` value {{is called out of a place that owns it and that no other name reaches: a local, a part of a struct local (`[OWN-11]`:}}
  - …`[CLO-6a]` A `once fn` value the [[program owns, including one in an `owns` block]]
  + …`[CLO-6a]` A `once fn` value the {{part is `E3040` until given a new value),}}
  - …in an `owns` block or a [[collection, is callable:]]
  + …in an `owns` block or a {{collection element the calling API removes (`jobs.pop_front()()`);}}
  + …hands the value out of its {{place. A call of one held in a class object's field, in an `owns` block (`[EXP-6]`), or reached through a `shares`, `shares mut`, `thread_shared`, `weakref` or `sharedref mut` link or a view, is `E3013` ("`j.task` is a field of a `Job` object, which other names may reach and would find empty; take the function out first: `t = mem.replace(ref mut j.task, fn(): pass)`, then `t()`"), as every hand-over out of such a}}
  - …the value out of its place [[(the place becomes empty, and a collection element]]
  - …empty, and a collection element is [[removed by the calling API, e.g. `jobs.pop_front()`).]]
  + …empty, and a collection element is {{(`[EXP-6]`).}}

### CLO-7 (p06-expressions.md; 4 words changed)
  + …and `jobs.submit` take `owns f: thread_shared {{once}}
  + …and `scope.spawn` a plain `owns f: {{once}}
  - …after its brackets what it holds [[(`thread.spawn(fn()]]
  + …after its brackets what it holds {{(`thread.spawn(owns fn()}}

### CLO-12 (p06-expressions.md; 40 words changed)
  - …written inside the call (`a = [[thread.spawn(fn()]]
  + …written inside the call (`a = {{thread.spawn(owns fn()}}
  - …the body `log.push("a")`, then `a = [[thread.spawn(push_a)`).]]
  + …the body `log.push("a")`, then `a = {{thread.spawn(owns push_a)`, with `owns` as before every argument for an `owns` parameter (`[FN-2a]`): `spawn` keeps the function past the call (`[THR-10]`), so `push_a` is handed over, and `push_a()` after that line is `E3040`; `a = thread.spawn(push_a)` is `E3028`).}}

### CLO-16 (p06-expressions.md; 152 words changed)
  - …`thread_shared` values or copies, never a [[`shares` or]]
  + …`thread_shared` values or copies, never a {{`shares`,}}
  + …never a `shares` or `shares mut` {{or `weakref T`}}
  - …a `shares` or `shares mut` value [[or]]
  + …a `shares` or `shares mut` value {{(a `weakref thread_shared T` it may keep: `fn() weakref log:` for `log: thread_shared Log`, `fn() weakref thread_shared this:` in}}
  - …or `shares mut` value or a [[`weakref` link]]
  + …or `shares mut` value or a {{method}}
  - …value or a `weakref` link of [[one]]
  + …value or a `weakref` link of {{a class whose objects are `thread_shared`)}}
  - …go to another thread, meets `thread_shared [[fn()`.]]
  + …go to another thread, meets `thread_shared {{fn()` only if, through its chain of calls (`[EFF-24]`), it reaches no `static mut` that is not `guarded by` a `static` `Lock` and no one-thread value by any route; the check is made where the function is given to, stored into or coerced to such a type, whichever thread runs it (`E7012`: "`on_frame` changes `FRAME`, a module variable of the main thread, and `Bus.handlers` holds functions any thread may run (`thread_shared fn()`); guard it: write `static TICK_LOCK: Lock = Lock()` and `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 3, and put the change inside `with TICK_LOCK:`"), so that the type says it and no package needs to see which thread calls through the place (`[BLD-16]`).}}

### CLO-17 (p06-expressions.md; 106 words changed)
  - …not stored in a place that [[a real variable it changes owns,]]
  + …not stored in a place that {{its body may reshape or end, or that is owned,}}
  - …it changes owns, all the way [[down (`b.on_click(fn():]]
  + …it changes owns, all the way {{down, by anything its body may reshape or end, through any name it reaches: a real variable, a name it captured with a word (a `shares mut`, `thread_shared` or `sharedref mut` owner or link, a `weakref` link it opens `ref mut`), a parameter the call gives it (`ref mut u` in `fn(ref mut u: Ui)`), `this`, or a `static mut` (`BUS.handlers.push(owns fn(): BUS.handlers.clear())`), read through the chain of its calls (`[EFF-24]`), since the run would end or move the function's own state (`b.on_click(owns fn():}}
  + …b.handlers.clear())`, `t.on_visit = fn(n: Node): t.root.children.clear()`, {{`bus.handlers.push(owns fn() weakref bus: if bus exists as ref mut b: b.handlers.clear())`,}}

### CLO-18 (p06-expressions.md; 105 words changed)
  + …arguments. A body that may end {{or move}}
  - …own state (a handler that removes [[itself) and uses]]
  + …own state (a handler that removes {{itself from the list}}
  + …that removes itself) and uses that {{holds it, or clears that list) is refused where the function value is stored (`E3022`, `[CLO-17]`): a}}
  - …removes itself) and uses that state [[afterwards holds one count on]]
  + …removes itself) and uses that state {{block is not counted (`[CLO-10]`), so nothing could keep}}
  + …afterwards holds one count on it {{alive}}
  + …holds one count on it across {{the call. The two words copied out before the call (`[CLO-10]`) point at the state the holder keeps, never at a copy of it: a function value}}
  + …one count on it across that {{changes its own state has a block, so the change is the holder's after the}}
  + …count on it across that call. {{A handler that must go hands its removal to its holder: a result the holder reads after the call, or a list of removals the holder applies after the loop.}}

### CLO-19 (p06-expressions.md; 2 words changed)
  - …to keep your own, write `fn(s): [[names.push(copy]]
  + …to keep your own, write `fn(s): {{names.push(owns copy}}

### EXC-3 (p08-classes.md; 11 words changed)
  - …unless the compiler knows they differ: [[different classes,]]
  + …unless the compiler knows they differ: {{classes of which neither is the other nor derives from it,}}

### EXC-19 (p08-classes.md; 4 words changed)
  - …never conflict: `for c in this.children: [[this.log.push(c.name)`]]
  + …never conflict: `for c in this.children: {{this.log.push(owns copy of c.name)`}}

### EXC-17 (p08-classes.md; 19 words changed)
  - …at any time on one thread [[(`[LT-46]`);]]
  + …at any time on one thread {{(`[LT-46]`), except while a read-only view of it made through another name is still used (`E3021`, the sentence before);}}

### DSP-3 (p08-classes.md; 30 words changed)
  + …for repeated calls on one value. {{Each entry of an interface's table holds, beside the implementation's code, the fact `[LT-22]` records for it, in the width the interface's declaration fixes, read by a `runtime` check (`[LT-48]`).}}

### DSP-4 (p08-classes.md; 4 words changed)
  + …(`dyn I`, `owns dyn I`, `shares {{dyn I`, `shares mut}}

### DSP-6 (p08-classes.md; 13 words changed)
  - …Pool[T]`"). Each cast is one type-id [[compare.]]
  + …Pool[T]`"). Each cast is one type-id {{compare; what the words before a cast hand over or share is `[TYP-44]`'s.}}

### EFF-1 (p10-effects.md; 59 words changed)
  + …generator runs none of its body; {{a read of a `static` built on first access (`[STA-3]`) carries its initialiser's effects, `Block` included;}}
  - …`next` carries the effects of the [[body up to]]
  + …`next` carries the effects of the {{whole body,}}
  - …of the body up to the [[next `yield`,]]
  + …of the body up to the {{union over its segments, since which `yield` it resumes from is a state the program computes while it runs (a generator a `@noalloc` function drives therefore allocates nowhere in its body),}}
  - …`yield`, and dropping a suspended one [[those of]]
  + …`yield`, and dropping a suspended one {{carries}}
  + …suspended one those of the drops {{at every `yield`}}
  - …one those of the drops it [[performs]]
  + …one those of the drops it {{may be suspended at}}

### EFF-2 (p10-effects.md; 195 words changed)
  + …bounded by a function type (`[F: {{needs}}
  - …that is the function given, per [[instance.]]
  + …that is the function given, per {{instance: the package that gives the function compiles that instance and sees the function (`[COST-3]`).}}
  - …function given, per instance. For a [[parameter,]]
  + …per instance. For a parameter, field, {{a}}
  - …instance. For a parameter, field, list [[element]]
  + …instance. For a parameter, field, list {{element, a parameter}}
  + …a parameter, field, list element or {{a}}
  + …or local of a function type, {{the shared type, which is compiled once and keeps functions of different kinds together (verdict 38),}}
  + …is every lambda and named function {{the program puts in that place, a library's included (verdict 122): each field of each struct and class apart from every other, a list's elements with the place that holds the list, a parameter with every argument given to it, and a place a value is copied into with the functions of the place it came from. A value the compiler cannot follow, read through a raw pointer or given by C, has those of every function}}
  + …lambda and named function stored into {{its type. A place no function ever reaches is called by no code}}
  - …and named function stored into that [[type anywhere in]]
  + …and named function stored into that {{runs, and contributes nothing. Dropping a function value has}}
  - …into that type anywhere in the [[program,]]
  + …into that type anywhere in the {{effects of dropping each function that reaches its place: none for}}
  - …type anywhere in the program, a [[library's included.]]
  + …type anywhere in the program, a {{named function, its captured values' drops for a lambda.}}
  + …if none are declared. A contract {{is}}
  - …contract written on a function type [[is checked against]]
  + …contract written on a function type {{above a `type` line that names one (verdict 106): `@noalloc` above `type Handler = fn()`;}}
  + …checked against every function stored into {{a place of that type is checked against}}
  - …stored into it, with the contract's [[code.]]
  + …stored into it, with the contract's {{code, and a call through such a place has no effect the contract excludes.}}

### EFF-4 (p10-effects.md; 93 words changed)
  + …of the scoped tasks it starts; {{for a class, whether its own methods say its objects are on one thread, `thread_shared`, or neither, and for a function the parameters, `this` included, of which it makes a `weakref T` link (`[OWN-13]`, `[BLD-14]` items 8 and 9), which parameter it stores an owner of into a place of which other parameter or of `this` (`[WK-7]`, item 10), and what its body needs of the kinds that reach a `dyn`, interface or base-class parameter (`[LT-1]`, `[OWN-13]`, `[WK-7]`, item 11);}}
  - …an interface change and re-checks every [[caller:]]
  + …an interface change and re-checks every {{caller, and every line checked against a place the change fills (`[BCK-8]`, `[MOD-9a]`):}}

### EFF-9 (p10-effects.md; 25 words changed)
  - …the compare of `assert_disjoint` and `check_disjoint` [[(`[DSJ-1]`),]]
  + …the compare of `assert_disjoint` and `check_disjoint` {{(`[DSJ-1]`) and}}
  - …of `assert_disjoint` and `check_disjoint` (`[DSJ-1]`), the [[one]]
  + …of `assert_disjoint` and `check_disjoint` (`[DSJ-1]`), the {{test of a `runtime` name's fact at a line that changes what the call's result may point into (`[LT-48]`), the two}}
  - …`check_disjoint` (`[DSJ-1]`), the one run-time alias [[check]]
  + …`check_disjoint` (`[DSJ-1]`), the one run-time alias {{checks}}
  - …the one run-time alias check there [[is;]]
  + …the one run-time alias check there {{are;}}

### EFF-25 (p10-effects.md; 6 words changed)
  - …= [] fn note(owns msg: String): [[LOG.push(msg)]]
  + …= [] fn note(owns msg: String): {{LOG.push(owns msg)}}
  - …note(owns msg: String): LOG.push(msg) fn tick(): [[note(String("tick"))]]
  + …note(owns msg: String): LOG.push(msg) fn tick(): {{note(owns String("tick"))}}
  - …LOG.push(msg) fn tick(): note(String("tick")) fn main(): [[LOG.push(String("start"))]]
  + …LOG.push(msg) fn tick(): note(String("tick")) fn main(): {{LOG.push(owns String("start"))}}

### EFF-26 (p10-effects.md; 4 words changed)
  - …calls the function it is given [[(`scores.for_each(fn(s): LOG.push(copy]]
  + …calls the function it is given {{(`scores.for_each(owns fn(s): LOG.push(owns copy}}

### EFF-27 (p10-effects.md; 63 words changed)
  - …change through it passes through the [[holder.]]
  + …change through it passes through the {{holder; a `weakref` link may be made from any name at any time, so the class objects it may name are kept out of a `@cached` body's reads instead (`[CLS-12]`).}}
  + …result of its type (Annex B). {{A change made by foreign code, or through a raw pointer, passes through no name the compiler sees: unsafe code that changes a field a `@cached` method reads answers for it (`[UNS-4]`, `[CLS-12]`).}}

### EFF-12 (p10-effects.md; 63 words changed)
  - …the run-time compare of `assert_disjoint` or [[`check_disjoint`,]]
  + …the run-time compare of `assert_disjoint` or {{`check_disjoint` and the test of a `runtime` name (`[LT-48]`),}}
  - …compiler proves apart need no compare, [[and]]
  - …no compare, and a copy needs [[none (`E4030`:]]
  + …no compare, and a copy needs {{none, and a kind known when compiling is checked when compiling (`E4030` at the word: "`present` is `@no_alias_checks`, and `runtime` on `r` (line 19) tests while running whether `scratch` changes while `frame` looks at it; give `present` a kind known when compiling, `fn present(r: Gpu, ref mut scratch: Array[u8]):`, or remove `@no_alias_checks`"; `E4030`:}}

### EFF-15 (p10-effects.md; 2 words changed)
  - …this document specifies — `[OPT-2]`, `[RNG-4]`, [[`[SIMD-7]`]]
  + …this document specifies — `[OPT-2]`, `[RNG-4]`, {{`[SIMD-7]`, `[DSJ-11]`}}

### ERR-4 (p13-errors.md; 47 words changed)
  + …and `context` (`[ERR-10]`). `unwrap` and `expect` {{on a `Result[T, E]` need `E: needs Display` (`E2040` otherwise, whose help is to implement `Display`) and}}
  - …`expect` panic with the error's `Display` [[text.]]
  + …`expect` panic with the error's `Display` {{text, `expect(msg)` with `<msg>: <Display text>`; on an `Option` they panic with `called unwrap on None` and with `msg`.}}
  + …it (ODR-025): * `Option[T]`: `[U, F: {{needs}}
  + …owns this -> Option[U]`; `[U, F: {{needs}}
  + …F) owns this -> Option[U]`; `[F: {{needs}}
  + …F) owns this -> Option[T]`; `[F: {{needs}}
  + …F) owns this -> Option[T]`; `[F: {{needs}}
  + …this -> T`; and `[E, F: {{needs}}
  + …E]`; * `Result[T, E]`: `[U, F: {{needs}}
  + …this -> Result[U, E]`; `[F, G: {{needs}}
  + …this -> Result[T, F]`; `[U, F: {{needs}}
  + …this -> Result[U, E]`; `[F, G: {{needs}}
  + …this -> Result[T, F]`; and `[F: {{needs}}

### ERR-3 (p13-errors.md; 321 words changed)
  - …`Error` is the interface of error [[types: `Display + Debug`]]
  + …`Error` is the interface of error {{types, `interface Error: needs (Debug, Display):`}}
  - …-> Option[ref dyn Error]` (default `None`) [[(§IV.8).]]
  + …-> Option[ref dyn Error]` (default `None`) {{(§IV.8): what an error reports is its `Display`, what it chains is `source`, and which thread may hold it is its own type's affair, read from what it holds as for any type (`[THR-8]`, `[THR-9]`), so a type holding a value that stays on one thread (a `shares` or `shares mut` owner, a `weakref T` or `sharedref mut` link, a `Cell`, an object held by the type of an `open` or `abstract` class another package may derive from whose header promises neither marker, `[THR-8]`) is an `Error` on its thread as any other: matched or printed by its `Display` (`[ERR-1]`, `[ERR-12]`, ODR-050), returned in a `Result[T, LoadError]`, and passed through `?` into another `Result` whose error type takes it (`[ERR-2]`). The error any thread may hold is `AnyError` (`[ERR-8]`), whose `From[E]` needs `E: needs (Error, Send, ThreadShared)`, so such an error does not pass through `?` or `.into()` into `AnyError`: that line is `E2040`, naming the field and giving the lines that compile ("`LoadError` cannot become an `AnyError`, which any thread may hold: `LoadError.script` is a `Script`, an `open` class whose header does not say that any thread may hold its objects, so a `LoadError` may hold an object of a class another package derives from `Script`; return `Result[Level, LoadError]` from `load`, keep what the error reports as a copy, `entity: int`, or write `pub open class Script implements Send, ThreadShared:` (line 3), which every class derived from `Script` then keeps"), decided when compiling from `LoadError`'s declaration, nothing while the program runs; a `Result[T, AnyError]` goes to a worker thread, through a channel and into a scoped task's view as any `Send` and `ThreadShared` value does (`[THR-10]`, `[THR-15]`, `[THR-11]`, `[THR-2]`).}}
  - …marked `@from` whose single field is [[an error]]
  + …marked `@from` whose single field is {{of a}}
  + …single field is an error type {{`E: needs Error`}}
  - …field is an error type gets [[`From` for that type]]
  + …field is an error type gets {{`From[E]`}}
  - …`From` for that type and returns [[it]]
  + …`From` for that type and returns {{the field}}
  - …that type and returns it from [[`source`.]]
  + …that type and returns it from {{`source`; `@from` on a variant with no field, two fields, or a field of another type (`AnyError` included, `[ERR-8]`) is `E2080`, naming the field.}}

### ERR-8 (p13-errors.md; 163 words changed)
  - …is the prelude's "any error" type: [[a `dyn Error`]]
  + …is the prelude's "any error" type: {{an error any thread may hold,}}
  - …`dyn Error` held alone (`owns dyn [[Error`)]]
  + …`dyn Error` held alone (`owns dyn {{(Error, Send, ThreadShared)`, `[TYP-22]`),}}
  - …It implements `From[E]` for every `E: [[Error`,]]
  + …It implements `From[E]` for every `E: {{needs (Error, Send, ThreadShared)`,}}
  + …`E: Error`, so `?` converts any {{such}}
  - …so `?` converts any error into [[it,]]
  + …so `?` converts any error into {{it (a one-thread error is `E2040` at that line, `[ERR-3]`),}}
  - …and `[E]` above `fn is() -> [[bool`.]]
  + …and `[E]` above `fn is() -> {{bool`, which compare the identity a kind test compares (`[TYP-44]`, `std.core.type_id[E]()`), never a hash of a name, so two majors of one package (`[VER-5]`) are never confused.}}
  + …another `AnyError` is handed over (`[ERR-7]`). {{`AnyError` is `Send` and `ThreadShared` as its parts are: its `owns dyn (Error, Send, ThreadShared)` by the markers in its list (`[TYP-22]`, `[THR-8]`, `[THR-9]`), its identity a number, its context a list of `String`; so a worker returns `Result[int]` through `thread.spawn` or a channel, a `thread_shared` holder keeps one under its lock and a scoped task reads one through `ref`, with nothing written (`[THR-10]`, `[THR-15]`, `[THR-1]`, `[THR-11]`). What `?` checks is the bound of `From[E]`, from `E`'s declaration when compiling, nothing while the program runs; an `AnyError` handed to another thread walks the error it holds by that error's kind's hand-over routine, as any `owns dyn I` value is walked (`[THR-2]`).}}

### BLD-14 (p17-toolchain.md; 795 words changed)
  - …6. the locks a function may [[take;]]
  + …6. the locks a function may {{take, and the locks it needs held (`[THR-18]`);}}
  + …values the package stores into each {{place a}}
  + …the package stores into each function {{value is called through (each field of each}}
  + …package stores into each function type {{apart, `[EFF-2]`)}}
  - …at every foreign call not marked [[`@ffi(no_callbacks)`. A library update]]
  + …at every foreign call not marked {{`@ffi(no_callbacks)`; 8. for a class, whether its own methods, and its bases', say}}
  - …marked `@ffi(no_callbacks)`. A library update that [[changes]]
  + …marked `@ffi(no_callbacks)`. A library update that {{its objects are on one thread (`weakref this`), `thread_shared` (`thread_shared this`, `weakref thread_shared this`), or neither, which a line that makes an object of the class checks (`[OWN-13]`); for an `open` or `abstract` class another package may derive from, the markers its header promises (`Send`, `ThreadShared`, an interface that needs them), which each class derived from it in another package is checked against where it is declared (`[THR-8]`, `E7015`); and for an `open` or `abstract` class no other package can derive from, whether its objects are `Send` and `ThreadShared`, read from the classes its package derives from it, which a dependant holding one through a `pub` type's field reads (`[THR-8]`); and 9. for each function, the parameters, `this` included, of which it makes a `weakref T` link, directly or through a call, which a caller giving one a value reached through a `thread_shared` value checks (`[OWN-13]`, `E7015`); 10. for each function, which parameter, `this` included, or value holding one, it stores an owner of into a place of which other parameter or of `this`, directly or through a call, which each call judges as that store (`[WK-7]`); and 11. for each `pub` function with a `dyn`, interface or base-class parameter (`this` of an `open` or `abstract` class's method included), what its body needs of the kinds that reach it: for each call through the parameter and each argument of that call, whether the called method's result may point into it (`[LT-1]`), and which of the function's own parameters, or what holds them, the call's object and each argument are, so that the links the called method makes of them and the owners it stores (items 9 and 10) count as the function's for that kind (`[OWN-13]`, `[WK-7]`), which a call from another package checks against the kinds it gives; and 12. for each line that calls through a `virtual` method of a `pub` class, a method of a `pub` interface used as `dyn`, or a `pub` function-typed place, what the line walks, holds `ref mut` and has lent, and, for its object and each argument, which of the others owns it there or will by a later line of the function, and whether it is reached through a `thread_shared` value (`[WK-7]`, `[OWN-13]`), which a package adding an override, an implementation or a stored function checks that member against (`[MOD-9a]`); 13. for each `pub` function, the places each of its function-typed parameters flows into (`[EFF-2]`: a place a value is copied into), so that a dependant checks `[CLO-17]` at the call that hands the function over ("`on_click` would keep it in `b.handlers`"); and 14. for each `pub` type implementing `Allocator`, whether its `deallocate` has an empty body and whether it implements `FoundByAddress`, which decide whether values made in its instances keep the instance word (`[ALC-1]`), a layout fact of every package that writes `using` that type; and 15. for each function, its open lines: each foreign call without `@ffi(no_callbacks)` after which it uses a view parameter, `this` included, or a `static` of its own, with what the parameter is used as, which a caller from another package resolves against the callbacks, exports and overrides its program has (`[BRW-21]`, `[FFI-52]`); and 16. for each call through a `pub` interface or `virtual` method, or through a `pub` place of function type (a field, a parameter, an element), the facts the package's own checks relied on at that call: what the functions there may change, read and end, as `[HR-44]` records them for a reload (a call through}}
  + …library update that changes such a {{place is not a}}
  + …update that changes such a fact {{a word}}
  - …that changes such a fact can [[make]]
  + …that changes such a fact can {{write, since the contracts of `[EFF-5]` say nothing of what}}
  + …such a fact can make a {{function may end, so it is recorded, never assumed). A package that adds to such a place, an implementation of the interface, an override, a function it stores there or passes to it, is checked against them when it compiles: an implementation, override or store whose summary breaks a fact a recorded}}
  + …a fact can make a line {{relied on is refused at its own line, with the code}}
  - …fact can make a line of [[a program]]
  + …fact can make a line of {{the rule}}
  - …a line of a program that [[uses it stop compiling;]]
  + …a line of a program that {{line relied on (`E3020`, `E3021`, `E3031` or `E3060`), naming}}
  - …that uses it stop compiling; the [[message names]]
  + …that uses it stop compiling; the {{library's line and}}
  - …message names the chain of calls [[(`[DIA-27]`). A parent link proves itself only when]]
  + …message names the chain of calls {{("`Pruner.visit` clears `tree.nodes`, and `lib`'s `Tree.walk` (lib/tree.em line 7) walks `nodes` across its call of `visit`; keep what to remove in a list of your own and clear after the walk"), so that}}
  - …link proves itself only when no [[other]]
  + …link proves itself only when no {{package's accepted line is made unsafe by a}}
  - …itself only when no other package [[can change]]
  - …no other package can change it [[or]]
  + …no other package can change it {{never saw, and nothing runs for it while}}
  - …package can change it or the [[places that own its holders. What]]
  + …package can change it or the {{program runs; the record lives in the interface file beside item 7, and in}}
  - …that own its holders. What a [[generic function changes through its type parameters' methods is worked out for each instance, whose body the instantiating package compiles.]]
  + …that own its holders. What a {{reloadable image's manifest (`[HR-8]`), which `[HR-44]` compares.}}
