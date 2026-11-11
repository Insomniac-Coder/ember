# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 03

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### OBJ-1 (p08-classes.md; 785 words changed)
  + …a 32-bit type index. A value {{of}}
  - …32-bit type index. A value a [[`weakref` link may reach]]
  + …32-bit type index. A value a {{type that **carries a weak count**}}
  - …has a third 32-bit number there, [[its]]
  + …has a third 32-bit number there, {{the}}
  - …that share one root base class) [[of which some `weakref` link in the program may name an object,]]
  + …that share one root base class) {{that carries one,}}
  - …may name an object, and every [[shared]]
  + …may name an object, and every {{`shares` or `shares mut`}}
  - …every shared value of a type [[some]]
  + …every shared value of a type {{that carries one. Whether a type carries one is decided by the package that declares it (for a class family, the package that declares its root base class; every class of the family carries it alike, since a link to a base may name a derived object), from that package's own text and from nothing else (`[BLD-16]`): a type carries a weak count when **it can be named from outside its package** (a class of the family, or the type, is `pub`; or is named in a `pub` item's signature: a `pub` function's parameters or result, a `pub` field's type, a `pub` alias, or a type argument in one of these, through which a dependant's name takes its type by inference) **or when a `weakref` type in the package names it** (written, inferred at a `weakref x`, or in a generic instance the package makes, `[MONO-1]`). A tuple, a fixed array and a function type, which every package can write, carry one. A `pub(package)` type, and a private type its package never links to, carry none, and no}}
  - …of a type some `weakref` link [[may name.]]
  + …of a type some `weakref` link {{anywhere can name a value of one: no line outside the package holds such a value by its type (an object held `dyn I` or `some I` is never linked, `E2119`, `[WK-16]`), and no line inside links to one. The fact is part of the type's layout in its package's interface file (`[BLD-2]`, `[MOD-9]`) and a flag of its `TypeInfo` (`[RT-3]`), so a package compiled once lays out its objects the same in every program that uses it. A `weakref` whose operand's type carries no weak count can arise only by a route this rule does not list, and is `E0901`. A `thread_shared` value's header is 32 bytes (verdict 101), in every program: its making thread's count of its owners, the type index, the weak count, the other threads' count with its marks, and its place in the making thread's queue (`[RC-4]`). Which thread made a value is read from the header of the memory chunk its block came from, never from the value: the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator (`[ALC-3]`, `[FFI-27]`, else `[RT-1]`'s), writes the making thread into each chunk's header once, and a chunk belongs to that thread for the life of the process, never given to another thread's allocator, and a block freed, on whichever thread its last owner ends, goes back to the chunk's own pool, the making thread's, by a return the runtime synchronises (a thread that is not the chunk's pushes the block onto that pool's list of blocks returned from other threads with one atomic exchange, and the making thread takes that list over when it next allocates from the pool), so the record never changes. A chunk taken by a thread that keeps no home count (`[RC-4]` point 4: a scoped task's or job's thread, a job worker, a host thread) names no thread, and a `thread_shared` block from it is marked handed over in the header the runtime writes as it lays the block out, so no thread ever finds itself its maker and every count operation on it takes the away path; a block of a type that is not plain is marked the same by the line that makes it. A `thread_shared` value made `using` any manager but `Global` (`[ALC-1]`, `[TYP-46a]`; `System` and a one-instance manager a program declares included) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4) and no thread asks which thread made it; it is never queued, so where it keeps its instance's address (`[ALC-1]`) that address stands in the header's queue place, which only the subtract that takes the away count below zero writes (`[RC-4]` point 2), as the away count of a value every owner of which it counts never does, so the value costs no word beyond the 32 bytes and its fields lie at the offset they have in `Global`, one method body serving every manager (`[OBJ-2]`); so is a `thread_shared` value a `static` or a `comptime(e)` holds, placed in the image when compiling (`[CT-5]`).}}
  - …for the memory model: no access [[words]]
  + …for the memory model: no access {{words,}}
  - …model: no access words and no [[flags.]]
  + …model: no access words and no {{flags but a `thread_shared` value's three marks and the kept-by-C mark, the top bit of the type index (`[FFI-35a]`), so a type index is below 2^31, and a kind test, a cast and a table load read the index with that bit cleared, one `and` on the loaded word (`[TYP-44a]`, §X.4), so a marked object still answers `is` and a `dyn` call.}}

### OBJ-4 (p08-classes.md; 110 words changed)
  + …`using` (`[ALC-1]`), except as `[OPT-1]` permits. {{An object made in a manager keeps its instance's address only as `[ALC-1]` says: none in a manager that frees nothing one by one, or in one that implements `FoundByAddress`. A `thread_shared` value's block comes from the runtime's chunks (`[OBJ-1]`) when it is made with no `using`, and from the instance named when it is made `using` one, by calling `m.allocate(…)` as for any value made in `m` (`[TYP-46a]` says when): the first has its making thread read from its chunk's header, the second keeps its instance word as `[ALC-1]` says, is counted away from the line that makes it and asks no thread which made it (`[RC-4]` point 4).}}

### OBJ-5 (p08-classes.md; 21 words changed)
  - …ended: an owner made from `this` [[(`shares this`,]]
  + …ended: an owner made from `this` {{(`thread_shared this`; `shares this` and}}
  - …from `this` (`shares this`, `shares mut [[this`, `thread_shared this`)]]
  + …from `this` (`shares this`, `shares mut {{this` are refused in every method, `[RC-1]`)}}
  - …a copy of what you need: [[`NAMES.push(copy]]
  + …a copy of what you need: {{`NAMES.push(owns copy}}
  - …need: `NAMES.push(copy of this.name)`". `weakref this` [[stays allowed:]]
  + …need: `NAMES.push(copy of this.name)`". `weakref this` {{and `weakref thread_shared this` stay allowed (`[OWN-13]`):}}

### RC-1 (p08-classes.md; 523 words changed)
  + …with one owner is not counted: {{no owner is added or subtracted, and}}
  + …not counted: its owner ends it. {{Where its type carries a weak count (the 12-byte header, `[OBJ-1]`) its count is written twice, 1 at its making and 0 at its end, before its `drop` runs and before its weak count is decremented (`[OBJ-3]`), so that opening a link tests one number (`[WK-12]`: the target lives while its count is not zero; the old block of `[THR-2]` is marked the same way); where it carries none, the count is never written. **Only a whole value is shared** (verdict 103): `shares x` and `shares mut x` are written before a new value, a local of this function that owns its value (which becomes one of the owners: a value that is not a class object is then made in a block of its own from the line that declares the local (`xs = make()` puts the result in that block), as if that line had said `shares`, so nothing moves and no view of it ends at the `shares` line, and the local's type is the shared type throughout the function) or a name that is already a `shares` or `shares mut` owner (for an `owns` parameter, which this function owns from its first line, the block is made at the function's entry, before any view of it is made, so that the parameter's type is the shared type throughout the function and the caller's line is unchanged); `thread_shared x` is written only before a new value, a name that is already a `thread_shared` owner, or `this` in a method, which says that every object of the class is `thread_shared`, so that `this` is one (`[OWN-13]`) (`E7017` anywhere else, `[THR-1]`), since its header and its making thread's record are written where the value is made, and `shares` or `shares mut` written before a `thread_shared` owner is `E7017` too, since its owners are counted for several threads and a plain count would race them ("`p` is `thread_shared` (made on line 10), so its owners are counted for several threads; for one more owner write `q = thread_shared p`, and for a name that only reads it `q = ref p`"). A place that has one owner and is not a local of this function, a list element (`shares mut players[1]` of an `Array[Player]`), a field (`shares this.items`), a plain parameter (`fn keep(p: Player): LIST.push(shares p)`) or `this` in a method (`shares this`, `shares mut this`: the object may have one owner, which ends it with no count to test; "no second owner of a `Node` is made from inside its own method, since this `Node` may have one owner; let the caller give one: write `static fn enrol(owns me: shares mut Node):` and call it `Node.enrol(shares mut n)`"), is `E2122`: "`players[1]` has one owner, the list `players`, and no second owner can be made of one element; to let other names own the players, make the list share them from the start: write `players = [shares mut Player(), shares mut Player()]` in place of line 15". So a value in a one-owner place is ended by that place at once, with no count to test.}}

### RC-4 (p08-classes.md; 1818 words changed)
  - …subtracts, and never cross threads (`E7015`). [[**Both]]
  + …subtracts, and never cross threads (`E7015`). {{**A `thread_shared` value's owners are counted in two numbers** (verdict 101; biased counting, Choi, Shull and Torrellas, 2018): 1. **Home and away.** The thread that made the value}}
  + …never cross threads (`E7015`). **Both counts {{the owners it holds in the value's home count, with plain adds and subtracts; every other thread counts its owners in the away count, with atomic ones (`[RT-8]`). The value lives while the two add up to more than zero. Only the making thread reads both. When the making thread's own subtract takes the home count to zero, it sets the handed-over mark and reads the away count in one atomic step on the away word: when that count is zero it ends the value itself; otherwise the value is counted away from then on (point 4), and the subtract that takes the away count to zero on a value marked handed over ends it. 2. **The hand-over to the making thread.** When another thread ends an owner the making thread counted, the away count goes below zero: that thread puts the value in the making thread's queue, marking it queued in the same atomic step as its subtract, and only the making thread then ends it. The making thread empties its queue at each of its check points: it adds each value's home count into its away count, and ends the value when the sum is zero; when the sum is positive it moves the home count into the away count in one atomic step that clears the queued mark and sets the handed-over mark, so the value is counted away from then on and no later subtract finds a stale mark. A queued value is ended by nothing else. **The end is one atomic step**: whichever thread ends a `thread_shared` value sets the ended mark with a compare-exchange on the away word that succeeds only if the count is the one it read; an opening of a `weakref thread_shared T` link that added to the count meanwhile (`[WK-12]`) makes it fail, and the thread reads again. 3. **Check points.** The compiler places them: at the back edge of every loop whose body makes a call or an operation on an `Atomic` or a channel end (a loop that may wait for another thread, which may be ending values this thread made; a loop of plain operations has none, and its end is the next point), before and after every call that may block the thread (waiting for a lock, a `join`, `jobs.wait`, the end of a `with scope` block, a channel's receive, a sleep, and a foreign call, whose code the compiler cannot read, `[FFI-52]`), at the return of every exported function and trampoline (the entries C calls, `[FFI-22]`), and when the thread ends, before its `join` returns; the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line. What a check point ends is a value this thread made whose type is plain (point 4): its `drop` frees memory and ends what the value owns, and does nothing else; the values it ends have no owner left, so no view, walk or task of this thread reads them (a name opened from a `weakref thread_shared T` link holds a count, `[RC-5]` point 2, and a `weakref T` link reaches no `thread_shared` value, `[THR-2]`). A check point is therefore not a line or a call for the rules that judge where it stands (`[THR-19]`, `[BRW-18]`, `[RC-5]`, `[THR-11]`, `[EFF-1]`, §X.2), adds nothing to its function's effects or summary (`[EFF-24]`, `[DRP-8]`), and may stand inside a lock's `with` block or a guard's life, a walk, a view's life, a `with scope` block or a function with a contract; its own work (the test of the queue, the merge of point 2, a plain `drop`) is the runtime's, as the handed-over test before a count operation is (point 4). A value whose making thread has ended is ended at once by the thread that ends its last owner: a thread's end empties and closes its queue in one atomic step, and a subtract that finds the queue closed reads the home count, which nothing changes once the making thread has ended (the close is a release and the subtract that finds it an acquire, `[RT-8]`), and ends the value only when the home count and the away count after its subtract add up to zero, with the step of point 2; so no value waits for a thread that is gone, and none ends while an owner the making thread counted still holds it. 4. **Handed over at once.** The making thread, when it stores an owner}}
  + …(`E7015`). **Both counts of a `thread_shared` {{value where other threads can reach it (into any place reachable from a `thread_shared`}}
  + …**Both counts of a `thread_shared` value, {{a `guarded by` field or container under}}
  - …counts of a `thread_shared` value, its [[owners']]
  + …counts of a `thread_shared` value, its {{lock; through a channel; into the state of a `thread_shared fn()` value when it is made, `[CLO-16]`; into the capture of a thread or task it starts; into the token of a callback C keeps; every such place is known when compiling), first adds its own count of the value into the away count and marks the value handed over; from then on every thread, the making thread included, counts in the away count, and a later end on another thread never goes through the queue. A `thread_shared` value made `using` any manager but `Global` (`System`, a `static` manager and every instance alike, `[ALC-1]`, `[TYP-46a]`) is marked handed over at the line that makes it, in the header's first store, before any other thread can reach it: its block lies in memory the runtime did not lay out, so no chunk header names its making thread and no pool of the runtime takes it back (`[OBJ-1]`),}}
  - …`thread_shared` value, its owners' and its [[`weakref` links',]]
  + …`thread_shared` value, its owners' and its {{instance may be reset or end as soon as the compiler sees its last owner end (`[ALC-5]`, `[LT-4]`), which an end delayed to a check point would miss; so every owner of such a value is counted in its away count, its end is never delayed, and the thread that ends it gives the block back through the manager's `deallocate` there. Where the type of the place at the line says `using A` (`[TYP-46]`, `[BLD-16]`) the compiler emits the away path with no test of the mark and no read of a chunk; where it does not, the mark decides, as after any hand-over; the runtime's own routines, which see no type (`ember_rt_object_release`, the hand-over walk of `[THR-2]`, a reload's registration, `[HR-12]`), test the mark before anything reads a chunk header, so a block the runtime did not lay out is never asked for its making thread. A value made in `Global`, with or without `using Global` written, keeps the two counts, its block waits for the check point and nothing reuses it. Two more values}}
  - …owners' and its `weakref` links', are [[atomic:**]]
  + …owners' and its `weakref` links', are {{marked handed over at the line that makes them, for the same reason: an end delayed to}}
  - …its `weakref` links', are atomic:** a [[relaxed increment]]
  + …its `weakref` links', are atomic:** a {{check point would come at a place, or on a thread, where its `drop` may not run. **By its type.** A type is plain when it has no `drop`, or its `drop`, taken with the `drop`s of everything the value owns, all the way down, has an effect set (`[EFF-1]`) with none of the effects a contract may forbid (§X.2: `Alloc`, `Sync`, `Block`, `Io`, `Lock`, `Panic(Explicit)`, `RuntimeCheck(Aliasing)`, `Nondet`) apart from the count steps of ending the owners and `weakref thread_shared T` links the value holds, which every end performs; its summary (`[EFF-24]`) reaches no `static mut`; it runs no code its package cannot read whole (a call through a function value, a call through a `dyn`, `virtual` or interface method that a package it does not see may implement, `[EFF-6]`, a foreign call not declared `@ffi(no_callbacks)`); and every type the value owns is plain, the target of a `thread_shared` owner it holds included (a `dyn I`, a function value}}
  - …atomic:** a relaxed increment and an [[acquire-release decrement (`[RT-8]`).]]
  + …atomic:** a relaxed increment and an {{object of an `open` or `abstract` class it owns are not plain). The package that declares the type decides, from its own text and its imports' interface files, and records the fact beside the type's layout in its interface file and in its `TypeInfo` (`[BLD-14]`, `[BLD-16]`, `[RT-3]`), as it records the weak count (`[OBJ-1]`).}}
  + …an acquire-release decrement (`[RT-8]`). A `thread_shared` {{value of a type that is not plain is marked handed over at the line that makes it, so its `drop` never waits in a queue and runs where its last owner ends, as a written call there (`[DRP-8]`, `[THR-20]`). **By its thread.** A `thread_shared` value made on any thread other than the main thread or a `thread.spawn` thread (the thread of a scope's task or job, which the runtime runs on a thread it owns and never on the thread that opened the scope, `[THR-5]`, `[THR-11]`, `[JOB-2]`; a job worker running `jobs.submit` or a chunk of a `@parallel` loop, §XI.5; a host thread attached by `[FFI-22]`; a chunk the calling thread runs itself, `[PAR-4]`, is made on that thread) is marked handed over at its making: those threads run Ember code only while given a task, a job or a call from C, no `join` of the program waits for their end, and a task's claims are checked only until its join (`[THR-11]`), so a queue to them would have no bound and could run a `drop` after the window the rules checked. The runtime, which lays out the block and knows its thread's kind, sets the mark in the header it writes (`[OBJ-1]`, §X.4). So the queue of point 2 holds only a value of a plain type made on the main thread or a `thread.spawn` thread. Another thread's store of an owner it holds moves nothing: its owners are counted away already. On the making thread every}}
  + …acquire-release decrement (`[RT-8]`). A `thread_shared` count {{operation on a `thread_shared` value, and every opening of a `weakref thread_shared T` link to one, first tests the handed-over mark, one load and one branch (§X.4), and a name opened on the home count (`[RC-5]` point 2)}}
  + …decrement (`[RT-8]`). A `thread_shared` count is {{undone on the count the mark names at the name's end. 5. **So a value's end is delayed only** for a value of a plain type made on the main thread or a `thread.spawn` thread (point 4), when its last owner ends on a thread that did not make it and the value was not handed over at a place of point 4 (it travelled inside another object): it is ended at its making thread's next check point, and its `drop`, which is plain, runs there; a making thread that waits in one call for its life (`rx.recv()`) ends it when the call returns or at its own end, holding only memory meanwhile. Every other `thread_shared` value is ended where its last owner ends. Until then it is intact, and a `weakref` link opened on another thread may still find it (`[WK-3]`). The weak count of a `thread_shared` value is atomic. The away count, and the weak count, are}}
  - …count is raised with one atomic [[add,]]
  + …count is raised with one atomic {{add}}
  - …raised with one atomic add, and [[its]]
  + …raised with one atomic add, and {{their}}
  - …count can roll over, whatever their [[number;]]
  + …count can roll over, whatever their {{number. The home count, and}}
  - …number; a count that is not [[`thread_shared` keeps]]
  + …number; a count that is not {{`thread_shared`, keep}}

### RC-5 (p08-classes.md; 99 words changed)
  - …may end an object of that [[type**:]]
  + …may end an object of that {{type's class family** (`[EXC-3]`: an `Array[Entity]` cleared ends a `Player` derived from `Entity`), held directly or through anything that owns it, all the way down (a list of teams cleared ends their players):}}
  - …object whose `drop` may end one [[(`[DRP-8]`).]]
  + …object whose `drop` may end one {{(`[DRP-8]`); a hand-over of one, or of an object that owns one, to another thread, which may move it (`[THR-2]`).}}
  - …while it is lent. 2. A [[view of]]
  + …while it is lent. 2. A {{name opened from}}
  - …lent. 2. A view of a [[`thread_shared` object reached through a `weakref`]]
  + …lent. 2. A view of a {{`weakref thread_shared T`}}
  + …object reached through a `weakref` link {{(`[WK-11]`)}}
  - …through a `weakref` link holds one [[atomic]]
  + …this thread can be checked against. {{On the thread that made the object, until it is handed over (`[RC-4]` point 4), that count is a plain add to its home count; on another thread, one atomic add to its away count.}}

### WK-1 (p08-classes.md; 9 words changed)
  - …through `shares`, `shares mut` or `thread_shared` [[fields,]]
  + …through `shares`, `shares mut` or `thread_shared` {{owners, directly or through the values they own alone,}}

### WK-11 (p08-classes.md; 217 words changed)
  - …* `[WK-11]` **`weakref` links.** [[`weakref T` is a]]
  + …* `[WK-11]` **`weakref` links.** {{A `weakref`}}
  - …links.** `weakref T` is a link [[that]]
  - …that does not keep its target [[alive. Its]]
  + …that does not keep its target {{alive, and its type says what kind of}}
  - …keep its target alive. Its target [[lives]]
  + …keep its target alive. Its target {{it has. `weakref T` links a value}}
  - …target alive. Its target lives on [[its own:]]
  + …target alive. Its target lives on {{one thread:}}
  - …lives on its own: a class [[object,]]
  + …lives on its own: a class {{object with one owner or with `shares` or `shares mut` owners,}}
  - …or a value held `shares` or [[`thread_shared`.]]
  + …or a value held `shares` or {{`shares mut` (`[WK-13]`); one type serves them all, since each has the header of `[OBJ-1]` and is tested the same way, and its weak count is changed with plain adds and subtracts. `weakref thread_shared T` links a `thread_shared T`, a type of its own: its target's header is `[RC-4]`'s, its weak count is atomic, and its opening holds a count (`[WK-12]`). The kind of the target is in the link's type and nowhere else: no flag is read while the program runs (`[OBJ-1]`), and the two types never convert into each other (`E2020`, `[TYP-45]`). A value held `owns` lives in a block with no header and is not a target: a `weakref` of it is `E2119` ("`b` holds its mesh in a block with no header, which a link cannot test; a link names a class object or a value held `shares` or `thread_shared`: write `b = shares Mesh(points=[0.0])` in place of line 3, then `w = weakref b`").}}
  - …makes a link to what `x` [[names;]]
  + …makes a link to what `x` {{names, of the kind the words on the path to `x` say (`[OWN-13]`);}}
  - …to what `x` names; before a [[`weakref`]]
  + …`weakref` link it makes another link {{of the same type}}
  - …a `Player` (4,294,967,295)" (`[RC-4]` for a [[`thread_shared` target).]]
  + …a `Player` (4,294,967,295)" (`[RC-4]` for a {{`weakref thread_shared T`).}}
  - …a scoped task does it only [[for]]
  + …a scoped task does it only {{through}}
  - …task does it only for a [[`thread_shared` target]]
  + …task does it only for a {{`weakref thread_shared T`}}

### WK-16 (p08-classes.md; 153 words changed)
  + …field, or an element stored inline), {{of a value held `owns`, or of a new value,}}
  - …target does not live on its [[own,]]
  + …target does not live on its {{own (`[WK-11]`, `[OWN-13]`),}}
  + …over; write `cam.target = weakref player`". {{A `weakref` of a value held `dyn I`, `owns dyn I`, `shares dyn I` or `some I`, and the type `weakref dyn I`, are `E2119` too: the kind of the object is known only while the program runs, and whether its header holds a weak count is its class's to fix (`[OBJ-1]`): "`weakref dyn Listener` (line 5) names a kind known only while the program runs, and a `weakref` link needs a class that fixes its header; link to a base class: write `open class Listener:` with `virtual fn notify(msg: StrView): pass` in place of lines 1 and 2, `class Button(Listener):` in place of line 8, and `listeners: Array[weakref Listener] = []` in place of line 5". A link to the base class is one pointer, opened with one header test, and every object of the family carries the count.}}

### WK-12 (p08-classes.md; 121 words changed)
  - …holds nothing or its target is [[ended. Each opening]]
  + …holds nothing or its target is {{ended (a target handed to another thread is ended here, `[THR-2]`). Opening a `weakref T`}}
  + …is one test of the header, {{a load and a compare, and nothing else,}}
  - …except a parent-link read proved by [[`[WK-17]`. For]]
  + …except a parent-link read proved by {{`[WK-17]`, which exists only where no `drop` runs when}}
  - …read proved by `[WK-17]`. For a [[`thread_shared`]]
  + …read proved by `[WK-17]`. For a {{node ends; the kind of the}}
  + …by `[WK-17]`. For a `thread_shared` target {{is in}}
  - …`[WK-17]`. For a `thread_shared` target the [[opening]]
  + …`[WK-17]`. For a `thread_shared` target the {{link's type, so no flag}}
  + …a `thread_shared` target the opening is {{read and no branch chooses the path. Opening}}
  - …`thread_shared` target the opening is a [[compare-exchange loop that]]
  + …`thread_shared` target the opening is a {{`weakref thread_shared T`}}
  + …a compare-exchange loop that adds one {{owner only}}
  - …compare-exchange loop that adds one to [[the count only while the count]]
  + …compare-exchange loop that adds one to {{a value that}}
  - …count only while the count is [[above zero,]]
  + …count only while the count is {{not ended,}}
  - …count for its life (`[RC-5]` point [[2).]]
  + …count for its life (`[RC-5]` point {{2): on the making thread, until the value is handed over (`[RC-4]` point 4), it tests that the home and away counts add up to more than zero and adds one to the home count; on any other thread it is one atomic add to the away count, undone when the value is marked ended.}}

### WK-3 (p08-classes.md; 112 words changed)
  - …parent is being ended), finds it [[ended,]]
  + …parent is being ended), finds it {{ended (a one-owner object's count is written 0 at its end, before its `drop`, `[RC-1]`, and that is what the opening tests),}}
  - …object is never read, nor counted [[again.]]
  + …object is never read, nor counted {{again; the test-free read of `[WK-17]` exists only where ending a node runs no `drop`.}}
  + …which that count keeps in memory. {{A `thread_shared` value whose last owner ends on a thread that did not make it, and that was not handed over (a value of a plain type made on the main thread or a `thread.spawn` thread, `[RC-4]` point 4), is ended at its making thread's next check point (`[RC-4]` point 5), and its `drop`, which frees memory and does nothing else, runs there; every other `thread_shared` value is ended where its last owner ends (`[THR-20]`).}}

### WK-17 (p08-classes.md; 179 words changed)
  - …field: `child.parent = weakref this`, then [[`this.children.push(child)`.]]
  + …field: `child.parent = weakref this`, then {{`this.children.push(owns child)`.}}
  - …into a place `o` owns alone [[(`o.children.push(x)`,]]
  + …into a place `o` owns alone {{(`o.children.push(owns x)`,}}
  + …or clears it (`c.parent = None`); {{a line gives the field a target by any route: `x.parent = weakref o`, a constructor argument (`Node(parent=weakref o)`), `mem.swap`, `mem.replace` or `mem.take` on the field or on a value that holds it, and `copy of` an object of the class, whose `Clone` copies the link (`[OWN-8]`), and a route that names no `o` (a copy, a swap) can be followed by no placing line, so a class of the family that has or derives `Clone`, or whose link is swapped or taken, fails the condition, in this package or in any package that can see the class (`[EFF-4]`);}}
  + …whole while it holds them; the {{field is a `weakref T`, the}}
  - …the objects have one owner (not [[`shares`, not `thread_shared`)]]
  + …the objects have one owner (not {{`shares`),}}
  - …owner (not `shares`, not `thread_shared`) and [[their class has]]
  + …owner (not `shares`, not `thread_shared`) and {{ending one runs}}
  - …and their class has no `drop` [[method]]
  + …and their class has no `drop` {{(`[DRP-8]`): no class of the family, and no type of a value an object of it owns, has a `drop`}}
  + …condition 1 fails, every read tests. {{Inside a `drop`, and in every function a `drop` may call (`[EFF-1]`), the object being ended is not known to live (`[WK-3]`) and its parent may be ending with it: a read of a parent link reached from it is never proved and tests.}}

### WK-5 (p08-classes.md; 126 words changed)
  - …edges among types, after generic substitution, [[over]]
  + …edges among types, after generic substitution, {{from}}
  - …types, after generic substitution, over the [[whole program:]]
  + …types, after generic substitution, over the {{types a package can see, its own and its imports' (`[BLD-16]`):}}
  + …value, or a collection of them; {{every place a type owns alone (a plain field or element, an `owns T`, a struct or enum payload stored inline), to the type held there, since what a value owns alone reaches what that owns; a `dyn I`, `owns dyn I`, `shares dyn I` or base-class place, with the edges of every kind the package can see there, and}}
  - …of them; a function value's state, [[from]]
  + …of them; a function value's state, {{with those of}}
  - …function value's state, from every function [[the program]]
  + …function value's state, from every function {{it}}
  - …from every function the program can [[put there (`[EFF-1]`);]]
  + …from every function the program can {{see (`[EFF-1]`, `[EFF-2]`);}}
  - …end that its own queue can [[hold.]]
  + …end that its own queue can {{hold; a `Retained[ForeignCallback]` token, to the callback's state, which it keeps for C (`[FFI-23]`).}}
  - …are not owner edges. A `let` [[field, set when its object]]
  + …are not owner edges. A `let` {{field}}
  - …field, set when its object is [[made,]]
  + …field, set when its object is {{an owner edge; it is the line that sets it, at the object's making, that closes no loop, since it}}
  - …can name only objects that exist [[already, so it never closes a loop.]]
  + …can name only objects that exist {{already.}}

### WK-6 (p08-classes.md; 4 words changed)
  - …make the link back `weakref`: `next: [[Option[weakref Node]`")]]
  + …make the link back `weakref`: `next: {{weakref Node = None`")}}

### WK-7 (p08-classes.md; 991 words changed)
  + …A `weakref` link back always works. {{**When a line may close a loop.** A store of an owner (`shares`, `shares mut`, `thread_shared`, a value that holds one, or a function value whose state owns one) into a place of an object may close a loop when the stored value's type can reach the holder's type through owner edges (`[WK-5]`); such a line is `E3032` unless the loop it closes is proved opened (above) or one of these proves it closes none, and every implementation applies exactly these: (1) the stored value cannot reach the holder: it was made in this function, on the line or earlier, and every owner that has been stored into it or into anything it owns, at its making (a constructor argument, `Node(next=Some(shares mut a))`) or on a later line, names a value that meets this same test and is neither the holder (the object whose place the line stores into) nor an owner of the holder (`root.children.push(shares mut child)` with `child = shares mut Node(...)` made above it and given no owner: a tree built from the top down closes nothing; `b.next = Some(shares mut a)` after `a.next = Some(shares mut b)` fails it, since `a` owns `b`, the holder); (2) the holder was made in this function and no owner of it has been stored anywhere yet (only this function's locals hold it), so nothing the stored value reaches can own it. **A store is judged in the package that can see both ends.** A store whose stored value is a parameter or `this`, or a value into which a parameter or an owner of one has been stored, or whose holder is a parameter or `this`, is not judged in the body: the function's summary records which parameter (or what holds it) it stores into a place of which other parameter or of `this`, directly or through a call (in the interface file across a package's edge, `[EFF-4]`, `[BLD-14]` item 10), and each call of the function is that store, judged there by the two proofs with the argument as the stored value and the other argument, or the object, as the holder, with the edges the calling package sees. A call through a function value, or through a `dyn`, interface or `virtual` method, is judged by the summaries of every function the package can put there (`[EFF-2]`). A function value given as the argument is the stored value, and its state's owners are its edges (`[WK-5]`): `p.btn.on_click(owns fn() owns q: println(q.name))`, with `q` a `shares mut` owner of the panel that holds the button and `on_click` storing its parameter into `this.handler`, closes `p` → `btn` → the handler's `q` → `p` and is `E3032` at the call. So a loop a dependant closes through a library's method is refused at the dependant's call (`h.add(owns b)` for a `Host.add` that pushes its parameter into `this.plugins`, with `b.host` a `shares mut lib.Host`: `E3032`, "this closes a loop of owners (`h` → `b` → `h`): `add` stores `b` in `h.plugins` (`lib`, its line 7), and `b.host` owns `h`; make the link back `weakref`: `host: weakref lib.Host`"). **A function another package adds.** Where another package may put a function behind a call (`[EFF-2a]`: a method of a `pub` interface called through `dyn`, a `virtual` method of a `pub` class, a function-typed place another package can fill), what that function stores of what the call gives it is judged where the function is seen, by one of two routes, so that no loop is closed by a member no judging package saw (`[BCK-8]`). *A kind the caller gives:* for a call through a `dyn`, interface or base-class parameter of a function (`this` of an `open` or `abstract` class's method included), the function's summary records the call with which of the function's own parameters, or what holds them, its object and each of its arguments are (`[BLD-14]` item 11); a call of the function applies, for each kind it gives, the called method's own record of the owners it stores (item 10) as the function's, and is judged with them. With `pub fn install(owns host: shares mut Host, owns p: owns dyn Plugin):` in `lib`, whose body is `p.attach(host)` and then `host.plugins.push(owns p)`, and a program's `struct MyPlugin implements Plugin:` whose `fn attach(host: shares mut Host):` is `this.host = Some(shares host)`, the call `install(shares mut h, owns MyPlugin())` is `E3032` ("this closes a loop of owners (`h` → the plugin → `h`): `install` gives `h` to `MyPlugin.attach` (`lib`, its line 7), which keeps an owner of it (line 4), and then stores the plugin in `h.plugins` (its line 8); keep a link in the plugin instead: write `host: weakref Host = None` in `MyPlugin` (line 2) and `this.host = weakref host` in place of line 4"), and the same call with a plugin whose `attach` keeps nothing compiles. *A kind a place holds:* for a call through such a method or place whose object the package holds (a field, an element, a `static`), the line's record (`[MOD-9a]`) names, for its object and each argument an owner can be made of (a `shares`, `shares mut` or `thread_shared` owner, or a value holding one), which of the others owns it there by the package's own stores, or will by a later line of the function; a package that adds an implementation, an override or a stored function whose record stores an owner of one of them into another that the line's record says owns it is refused at the line that adds it (`E3032`, naming the library's line and its own store). A kind the judging package cannot see (a `dyn` value another package made and gave it) counts as storing an owner of every argument an owner can be made of: where that closes a loop the call is `E3032`, whose help is to make the value where its type is seen, or the `weakref` link. Nothing runs for any of it while the program runs.}}

### HND-3 (p09-memory.md; 15 words changed)
  + …is retired or live, `insert` panics {{("`pool.insert(...)` on line 11: every slot is live or retired; use `try_insert` for a `Result`")}}

### CELL-1 (p09-memory.md; 16 words changed)
  - …a `T` that is replaced whole: [[`Cell(v)`, `set(v)`, `replace(v)]]
  + …a `T` that is replaced whole: {{`Cell(v)` (a constructor's argument takes no word, `[FN-2a]`), `set(owns v)`, `replace(owns v)}}
  + …T`, `take() -> T` (for `T: {{needs}}
  + …`into_inner()`, `get() -> T` for `T: {{needs}}
  + …`T: Copy`, and `update(f)` for `T: {{needs}}
  + …`update(f)` for `T: Copy` or `T: {{needs}}

### CELL-13 (p09-memory.md; 4 words changed)
  - …change it in a method: `fn [[add(e:]]
  + …change it in a method: `fn {{add(owns e:}}
  - …`fn add(e: int):` with the body [[`this.entities.push(e)`;]]
  + …`fn add(e: int):` with the body {{`this.entities.push(owns e)`;}}

### BCK-5 (p18-implementation.md; 2 words changed)
  - …change of its place (`[BRW-3]`), so [[`v.push(v.len())`]]
  + …change of its place (`[BRW-3]`), so {{`v.push(owns v.len())`}}

### BCK-6 (p18-implementation.md; 2 words changed)
  - …used (`boss = ref mut enemies[0]`, [[`enemies.push(Enemy(hp=5))`,]]
  + …used (`boss = ref mut enemies[0]`, {{`enemies.push(owns Enemy(hp=5))`,}}

### BCK-7 (p18-implementation.md; 32 words changed)
  - …may end an object of its [[type (`E3060`),]]
  + …may end an object of its {{class family, held directly or through anything that owns it (`[RC-5]` point 1, `[EXC-3]`; `E3060`),}}
  + …across a call that may change {{such}}
  - …call that may change an object [[of its type]]
  - …for such a view, except a [[view of a `thread_shared` object]]
  + …for such a view, except a {{name}}
  - …a `thread_shared` object opened from a [[`weakref`]]
  + …a `thread_shared` object opened from a {{`weakref thread_shared T`}}
  - …a `weakref` link, which holds one [[atomic]]
  - …one atomic count for its life [[(`[RT-10]`).]]
  + …one atomic count for its life {{(`[RC-5]` point 2, `[RT-10]`).}}

### BCK-8 (p18-implementation.md; 122 words changed)
  + …whole variable proved when compiling (`[MONO-10]`). {{**Which build checks a line.** A line whose verdict rests on what the whole program puts in a place (a call through a function value, an interface, a `dyn` or `virtual` method, a foreign call, the end of a `dyn` value) is checked by its own package with what that build sees (`[EFF-2a]`), and checked again, with the same rules, by every package that puts a new function, implementation, override or callback there, at the line that puts it (`[MOD-9a]`, `[BLD-14]` items 12 and 16, `[BRW-21]`'s open lines), or, for a kind a caller gives through a parameter, by that caller at its call (`[BLD-14]` item 11); a line checked this way by no build is a defect of the build (`[PHIL-12]`).}}
