# Rules changed or new from 0.9.15_Hardened_1 to 0.9.20_Hardened_12, gap area 05

`[[old words]]` were removed and `{{new words}}` added; each line starts with six words of context.

### TIER-1 (p01-overview.md; 25 words changed)
  - …an `unsafe:` block lexically enclosing the [[operation (inside an]]
  + …an `unsafe:` block lexically enclosing the {{operation, or the}}
  + …the operation (inside an `unsafe fn` {{whose}}
  - …operation (inside an `unsafe fn` body [[too, where every unsafe operation sits in its own `unsafe:` block, `E3100`);]]
  + …operation (inside an `unsafe fn` body {{encloses it, outside the body's `safe:` blocks (`[UNS-12]`);}}
  + …(`[GRM-35]`, `[FFI-2]`); an `unsafe import c` {{or `unsafe import cpp`}}
  - …`[FFI-2]`); an `unsafe import c` of [[C source]]
  + …`[FFI-2]`); an `unsafe import c` of {{foreign code}}
  + …C source compiled into the program {{that runs uncalled}}

### CORO-6 (p06-expressions.md; 92 words changed)
  - …view of one of its own [[locals)]]
  + …view of one of its own {{locals, or of a parameter the frame holds: one taken `owns`, or a `Copy` value, which is copied in, `[FN-1]`)}}
  - …write `yield n` instead of line [[5").]]
  + …write `yield n` instead of line {{5"; for a parameter, "`r` (line 2) points into this generator's own frame, where its parameter `p` lives, and the frame may move while it is suspended at the `yield` on line 3; use `p` itself: remove line 2 and write `yield p.x` instead of line 4").}}
  - …over the generator's own `buf` compiles). [[Views]]
  + …over the generator's own `buf` compiles). {{A view}}
  - …compiles). Views that came in as [[parameters,]]
  + …compiles). Views that came in as {{a parameter (`xs: Span[int]`, `p: Player`, a view type),}}
  - …that came in as parameters, or [[are]]
  + …that came in as parameters, or {{one}}
  - …as parameters, or are made from [[them,]]
  + …as parameters, or are made from {{what such a view points into,}}
  - …from them, may be held across [[`yield`;]]
  + …from them, may be held across {{`yield`, since it points outside the frame;}}

### CORO-14 (p06-expressions.md; 4 words changed)
  - …it, by position included (`E3020`, `[BRW-18]`: [["`d.log.push("open")`]]
  + …it, by position included (`E3020`, `[BRW-18]`: {{"`d.log.push(owns "open")`}}
  - …push after `g`'s last use: move [[`d.log.push("open")`]]
  + …push after `g`'s last use: move {{`d.log.push(owns "open")`}}

### UNS-1 (p09-memory.md; 370 words changed)
  + …(Part XVI); access a foreign `static {{mut`, or read a foreign static outside the safe reads of `[FFI-8]`'s static row and `[FFI-41]` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref}}
  - …XVI); access a foreign `static mut` [[(`[STA-1]`); `transmute`;]]
  + …XVI); access a foreign `static mut` {{or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`; a raw-pointer or C function-pointer field of an imported C or C++ aggregate, or a whole value holding one written into such an aggregate, whose memory foreign code reads, `[FFI-8]`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`); `transmute`, `copy` or `copy_nonoverlapping` (`[UNS-5]`);}}
  + …Send:`, `[TYP-43]`). Each of these outside {{unsafe code (an `unsafe:` block, or}}
  - …`[TYP-43]`). Each of these outside an [[`unsafe` block]]
  + …`[TYP-43]`). Each of these outside an {{`unsafe fn` body outside its `safe:` blocks, `[UNS-12]`)}}
  + …and indent line 9 under it"). {{The block to write goes around the whole statement the operation is in: the compound statement for one in its header (`elif p.read() > 0:` is its `if`'s, "put lines 9 to 12 in an `unsafe:` block"), every line of a statement written over several, and "put the statement on line 7 on a line of its own, in an `unsafe:` block" for one on the line of a header; outside a function's body (a constant's value), where no block can be written, the message gives none. The unsafe operations of one statement are one `E3100`, naming them together ("the reads through `p` on line 9"); a pointer's offset is named with the read or write it leads to.}}

### UNS-4 (p09-memory.md; 16 words changed)
  - …it points into is reshaped or [[ended.]]
  + …it points into is reshaped or {{ended, nor after a hand-over of its object to another thread, which may move it (`[THR-2]`).}}

### UNS-7 (p09-memory.md; 275 words changed)
  - …stating what its callers must make [[true; without]]
  + …stating what its callers must make {{true: a function declared inside unsafe code, which is an `unsafe fn` (`[UNS-12]`, verdict 119), one declared in an `unsafe extern` block (`[FFI-10]`, verdict 120), and a function `import c` or `import cpp` writes (`[FFI-6]`, `[FFI-17]`) that its overlay entry leaves unsafe to call (verdict 125), which carries it in that entry, above `unsafe fn name(…)` (`[GRM-35]`), included. Without}}
  - …`@safety("...")`, with what `p` must point [[to"). It]]
  + …`@safety("...")`, with what `p` must point {{to"; a declaration with no body is named without a step). For an imported function, `L3015` is given once for each function, at its first call in unsafe code by place: in the file built first, then in its imports, the earliest line in each (verdict 134): "`lib_step` is a C function the program calls in unsafe code, and nothing says what its callers must make true; say it in the overlay entry for `lib_step`: `@safety("...")` above `unsafe fn lib_step(…)`", with a note for each other call of it, `also called at src/sim.em:14:9`. A lambda written inside unsafe code, an `unsafe fn(…)` value, carries `@safety` too, written just before it (`set = @safety("`p` points to a writable `int`") fn(v: int) => p.write(v)`, verdict 126), and without it is `L3015`; the text stays with the lambda, which `ember tcb` lists, while a value's type `unsafe fn(…)` carries none, as for a named function. `@safety`}}
  - …`# SAFETY:` note on an `unsafe [[fn`]]
  + …`# SAFETY:` note on an `unsafe {{fn`, alone on the}}
  + …note on an `unsafe fn` line {{before it or at the end of its line (`[LEX-23]`),}}
  - …a `# SAFETY:` note says why [[one]]
  + …a `# SAFETY:` note says why {{an `unsafe:`}}
  - …note says why one block is [[sound]]
  + …note says why one block is {{sound: remove this line"; "… remove this note" for one at the end of the function's own line;}}
  - …why one block is sound and [[goes]]
  + …why one block is sound and {{on a function with no `@safety`, "`read_u32_le` should say what its callers must make true with `@safety("...")`}}
  - …block is sound and goes above [[each `unsafe:` block inside it:]]
  + …block is sound and goes above {{it, not with a `# SAFETY:` note: say this in `@safety` and}}

### UNS-11 (p09-memory.md; 17 words changed)
  - …vouches, with `# SAFETY:` saying why [[(`[UNS-8]`).]]
  + …vouches, with `# SAFETY:` saying why {{(`[UNS-8]`); `safe:` inside unsafe code says the opposite, that its code is checked as safe code (`[UNS-12]`).}}

### UNS-12 (p09-memory.md; 399 words changed)
  - …* `[UNS-12]` [[An]]
  + …* `[UNS-12]` {{**An}}
  + …* `[UNS-12]` An `unsafe fn` {{is unsafe as a whole** (verdict 118: "the function literally says that it is unsafe"). Its}}
  - …`[UNS-12]` An `unsafe fn` body is [[not itself an `unsafe` context: each]]
  - …itself an `unsafe` context: each unsafe [[operation in it sits in]]
  + …itself an `unsafe` context: each unsafe {{code, as the inside of}}
  - …it sits in an `unsafe:` block [[of its own, with its note, and one outside such]]
  + …it sits in an `unsafe:` block {{is. Everything written in unsafe code is unsafe (verdict 119):}}
  - …note, and one outside such a [[block is `E3100` ("`read_u32_le`]]
  + …note, and one outside such a {{function declared there}}
  + …`E3100` ("`read_u32_le` is an `unsafe fn`, {{whose body is unsafe, whose calls need unsafe code, and}}
  - …("`read_u32_le` is an `unsafe fn`, which [[says its callers must vouch;]]
  + …("`read_u32_le` is an `unsafe fn`, which {{carries `@safety` (`[UNS-7]`); a lambda written there is a value of an `unsafe` callable type, `unsafe fn(…)` (`[FN-6]`), which only unsafe code calls (`fn make() -> unsafe fn(*mut u8):` may return one made in an `unsafe:` block; `-> fn(*mut u8)` may not). Code inside unsafe code that is to be checked as safe code is written in a `safe:` block, which needs no note: in it each operation of `[UNS-1]` is `E3100` again ("`p.write(1)` (line 7) is in a `safe:` block, which is checked as safe code; move it out of}}
  - …says its callers must vouch; the [[reads through `p` on line 4 are this function's own unsafe steps, and nothing says why they are sound;]]
  + …says its callers must vouch; the {{`safe:` block, or}}
  - …says why they are sound; put [[line 4]]
  + …says why they are sound; put {{it}}
  - …`unsafe:` block with a `# SAFETY:` [[note saying why]]
  + …`unsafe:` block with a `# SAFETY:` {{note"), a function or lambda written in it is safe, and an `unsafe:` block in it is unsafe again. A `safe:` block outside unsafe code changes nothing and is `L3020` ("this `safe:` block is not inside unsafe code, so it changes nothing: remove `safe:`"); an `unsafe:` block directly inside unsafe code is `L3010` (`[UNS-3]`). A function that safe code runs without naming it is never an `unsafe fn`: `drop`, which runs wherever a value ends (`[DRP-1]`, `[DRP-8]`); an `override` of a method that is not `unsafe`, which a virtual call through}}
  - …`# SAFETY:` note saying why the [[read]]
  + …`# SAFETY:` note saying why the {{base reaches (`[DSP-2]`); and a method implementing an interface method the interface declares without `unsafe`, which a call through `dyn I` or through a bound reaches. Each written `unsafe fn`}}
  - …note saying why the read is [[safe: above line 4]]
  + …note saying why the read is {{`E2020`, an `unsafe fn` where a safe one is wanted (`[FN-6]`): "`drop` runs wherever a `Window` ends, in safe code, so it cannot be an `unsafe fn`;}}
  + …is safe: above line 4 write {{`fn drop():` and put `destroy_window(this.handle)` in an `unsafe:` block with a}}
  + …above line 4 write `# SAFETY:` {{note"; "`show` overrides `Widget.show`, which safe code calls through any `Widget`; write `override fn show():`}}
  + …line 4 write `# SAFETY:` and {{put `p.write(1)` in an `unsafe:` block with a `# SAFETY:` note". A `virtual` method or an interface method declared `unsafe fn` is called in unsafe code through}}
  - …4 write `# SAFETY:` and the [[reason, then `unsafe:`, and indent line 4 under it").]]
  + …4 write `# SAFETY:` and the {{base or the interface as any `unsafe fn` is (`[UNS-1]`), with its `@safety` (`[UNS-7]`); a safe override or implementation of one is allowed, as a safe function converts to an `unsafe fn(…)` type (`[FN-6]`).}}

### THR-8 (p11-concurrency.md; 661 words changed)
  - …(`shares T`, `shares mut T`), a [[`weakref`]]
  + …(`shares T`, `shares mut T`), a {{`weakref T`}}
  + …`shares mut T`), a `weakref` link {{(one}}
  - …a `weakref` link to a value [[that is not `thread_shared`,]]
  + …a `weakref` link to a value {{on one thread, `[WK-11]`), a `sharedref mut` link (a view several holders on one thread may change, `[LT-46]`),}}
  - …`threads(creator)` handle). Structs, enums, tuples, fixed [[arrays,]]
  + …`threads(creator)` handle). Structs, enums, tuples, fixed {{arrays and}}
  - …Structs, enums, tuples, fixed arrays, collections [[and class objects]]
  + …are `Send` when every component is; {{a class object by its class, since an object held by a base class type may be of any class derived from it (`[TYP-5]` rule 8): an object of a class that is neither `open` nor `abstract` when the class's fields, inherited ones included, are; an object of an `open` or `abstract` class that **another package may derive from** (it, or an `open` or `abstract` class derived from it in its package, is `pub`) only when the class **promises** it in its header, `implements Send`, `implements ThreadShared`, or an interface that needs them (`pub open class Script implements Send, ThreadShared:`; `open class Step implements Job:` for an `interface Job: needs (Send, ThreadShared):`), written without `unsafe` since the compiler checks it (`[TYP-43]`): the promise holds the class's own fields and methods (`[OWN-13]`) and those of every class derived from it, each checked where that class is declared, by the package that declares it, from the base's declaration in its source or in its interface file (`[MOD-9]`, `[BLD-14]` item 8), so a derived class that breaks it is `E7015` at its own line, naming the field and the promise and giving a copy line, and the `thread_shared` line only where `[THR-1]` then holds ("`Enemy.level` is a `shares Level` owner, which stays on one thread, and `Script` promises that any thread may hold its objects (`pub open class Script implements Send, ThreadShared:`, gamekit/src/lib.em line 3), a promise every class derived from it keeps; keep what the enemy reads from the level as a copy, `level_name: String`"); a derived class may promise what its base does not, for itself and the classes derived from it, never for the base; a generic class's promise holds for each instance whose arguments meet the needs its parameters are bounded by, as a generic `unsafe extend` does (`[TYP-43]`); an object of such a class whose header promises nothing is one no thread may be handed, whatever classes the program derives, which no package can see (`[BLD-16]`): a value held by its type reaching another thread by any route of `[THR-2]` is `E7015` at that line ("`s` is a `Script`, an `open` class whose header does not say that any thread may hold its objects, so it may be an object of a class another package derives from `Script`; write `pub open class Script implements Send:` (gamekit/src/lib.em line 3), after which every class derived from `Script` is checked for it where it is declared, or hand the object over by its own class, `d: Door`"); and an object of an `open` or `abstract` class that no other package can derive from when its fields and those of every class its package derives from it are, with nothing written, as a call through its `virtual` methods counts every override the package has (`[EFF-2a]`), a refusal naming the derived class and its field;}}
  - …`Send`; a `thread_shared T`, and a [[`weakref` link to one,]]
  + …`Send`; a `thread_shared T`, and a {{`weakref thread_shared T`,}}
  - …one, when `T` is `Send` and [[`ThreadShared`;]]
  + …one, when `T` is `Send` and {{`ThreadShared`, and for an `open` or `abstract` `T` whatever its class promises, since a `thread_shared` object is made only by a line that checks the object's own class (`[THR-1]`, verdict 165; `E7017` anywhere else) and an upcast keeps the word (`[TYP-5]` rule 8);}}
  + …when `I` requires it (`interface I: {{needs}}
  - …the words and written only in [[bounds;]]
  + …the words and written only in {{bounds and in the header of an `open` or `abstract` class, where it is a promise about every class derived from it that the compiler checks (above);}}
  + …sent to another thread is `E7008`. {{So a class with a `weakref T` field has no `thread_shared` objects: `root = thread_shared Node()` for a `class Node:` with `parent: weakref Node = None` is `E7015` at that line ("`Node.parent` is a `weakref Node`, a link kept on one thread, so a `Node` is not `Send`; for nodes shared between threads write `parent: weakref thread_shared Node = None` (line 3), link them `child.parent = weakref thread_shared this` (line 9), and make every node `thread_shared Node()`"); so has a class whose own methods write `weakref this` (`[OWN-13]`).}}

### THR-9 (p11-concurrency.md; 313 words changed)
  + …and written only in bounds (`[T: {{needs}}
  - …ThreadShared]` above `fn keep(xs: Array[T]) -> [[int:`)]]
  + …ThreadShared]` above `fn keep(xs: Array[T]) -> {{int:`), in the header of an `open` or `abstract` class as a checked promise (`[THR-8]`, `[TYP-43]`)}}
  + …'threads may share it' write `[T: {{needs}}
  - …types are `ThreadShared` when every component [[is.]]
  + …types are `ThreadShared` when every component {{is; a class object as `[THR-8]` judges `Send`: by its fields when its class is neither `open` nor `abstract`, by the class's promise (`implements ThreadShared`, or an interface that needs it) when another package may derive from it, each derived class checked where it is declared (`E7015`), and by its package's derived classes when none can; `dyn I` when `I` requires it (`interface I: needs ThreadShared:`), as for `Send` (ODR-255), so the `dyn (Error, Send, ThreadShared)` an `AnyError` holds is (`[ERR-8]`, `[TYP-22]`); a `thread_shared T` and a `weakref thread_shared T` of an `open` or `abstract` `T` whatever its class promises, as `[THR-8]` says.}}
  + …`ThreadShared` when every component is. `ref {{mut T`, `MutSpan[T]`, `sharedref}}
  + …mut T`, `Cell`, `UnsafeCell`, raw pointers, {{a `Retained[ForeignCallback]` token (`[FFI-23]`),}}
  + …pointers, `shares T`, `shares mut T`, {{a `weakref T` link,}}
  + …`shares T`, `shares mut T`, and {{an object of}}
  - …T`, `shares mut T`, and a [[`weakref` link to]]
  + …T`, `shares mut T`, and a {{class with}}
  - …and a `weakref` link to a [[value]]
  + …and a `weakref` link to a {{`@cached` method}}
  + …`weakref` link to a value that {{declares nothing saying its objects are shared (`[CLS-12]`: no `ThreadShared` promise in its header or a base's, no interface that needs it, no method that pins it `thread_shared`, `[OWN-13]`), whose kept result}}
  - …link to a value that is [[not `thread_shared`]]
  + …link to a value that is {{then filled with plain stores,}}
  - …value that is not `thread_shared` are [[not.]]
  + …value that is not `thread_shared` are {{not, so that `static UNIT: Circle` is `E7002` and `thread_shared Circle(r=1.0)` is `E7015` (`[THR-1]`, verdict 165), each naming the `@cached` line ("`Circle.label` is `@cached` (line 4), and `Circle` says nowhere that threads share its objects, so its kept result is filled with plain stores; say so in the class, `class Circle implements Shared:` with `interface Shared: needs ThreadShared:`, or remove `@cached` (line 4)"); an `extern type`, an opaque C++ class or a `@ffi(handle)` type is only where the program declares it (`[FFI-8]`).}}
  + …at once: `RwLock[Cell[int]]` is not `ThreadShared`), {{a `weakref thread_shared T` for such a `T`, a channel's `Sender[T]` for a `Send` `T` (`[THR-15]`), and `thread_shared fn()`, whose state is `ThreadShared` (`[THR-1]`), are; a `thread_shared once fn()` is `Send` and not `ThreadShared`, since it is handed over and run once (`[CLO-6]`, `[THR-10]`),}}
  - …`RwLock[Cell[int]]` is not `ThreadShared`), and a [[channel's `Sender` are.]]
  + …`RwLock[Cell[int]]` is not `ThreadShared`), and a {{`Receiver[T]` is `Send` and not `ThreadShared`, one consumer's (`[THR-15]`).}}

### THR-1 (p11-concurrency.md; 236 words changed)
  - …a new value (`st = thread_shared [[Stats()`) or]]
  + …a new value (`st = thread_shared {{Stats()`),}}
  - …a value already `thread_shared` (`fn() thread_shared [[st:`),]]
  + …a value already `thread_shared` (`fn() thread_shared {{st:`) or before `this` in a method, which says that every object of its class is `thread_shared` (`[OWN-13]`),}}
  - …a value made otherwise it is [[`E7017`]]
  + …a value made otherwise it is {{`E7017`, as `shares` or `shares mut` on a `thread_shared` owner is (`[RC-1]`)}}
  - …lives until its last owner ends. [[Both]]
  + …lives until its last owner ends. {{Its owners are counted in two numbers,}}
  - …its last owner ends. Both its [[counts, of owners]]
  + …its last owner ends. Both its {{making thread's}}
  - …Both its counts, of owners and [[of `weakref` links, are atomic,]]
  + …Both its counts, of owners and {{the other threads' (atomic),}}
  + …of `weakref` links, are atomic, and {{its weak count is atomic (`[RC-4]`);}}
  - …`Atomic`, a `Mutex`, an `RwLock`, a [[channel end,]]
  + …`Atomic`, a `Mutex`, an `RwLock`, a {{`Sender` (a `Receiver` is one consumer's, `[THR-11]`, and not `ThreadShared`, `[THR-9]`),}}
  - …lock, under that lock (`[THR-18]`); * [[everything reachable from it]]
  + …lock, under that lock (`[THR-18]`); * {{its type}}
  - …everything reachable from it is `Send` [[(`[THR-8]`):]]
  + …everything reachable from it is `Send` {{and `ThreadShared` (`[THR-8]`, `[THR-9]`, each judged by the type's components as the marker is: a `Mutex[T]` of a `Send` `T` is `ThreadShared` whatever `T` holds), since any thread may hold it and several may read it at once:}}
  - …`Send` (`[THR-8]`): a one-thread owner, a [[`weakref` link to]]
  + …`Send` (`[THR-8]`): a one-thread owner, a {{`weakref T` link,}}
  - …owner, a `weakref` link to a [[value]]
  + …owner, a `weakref` link to a {{plain `fn()` value, or a part}}
  - …to a value that is not [[`thread_shared`,]]
  + …to a value that is not {{`ThreadShared` (a `Cell`, a `sharedref mut` link}}
  + …value that is not `thread_shared`, or {{an `RwLock[Cell[int]]` outside}}
  - …that is not `thread_shared`, or a [[plain `fn()` value]]
  + …that is not `thread_shared`, or a {{`Mutex`; a raw pointer; a `Receiver`)}}
  - …at the line that makes it [[`thread_shared` ("`Bus.handlers`]]
  + …at the line that makes it {{`thread_shared`, naming the field and the fix (verdict 165: "`FrameStats.draws` is an `RwLock[Cell[int]]`, which several threads may not read at once, and a `thread_shared FrameStats` may be read from any thread; write `draws: Atomic[int] = Atomic(0)` in `FrameStats`"; "`Bus.handlers`}}
  - …never a one-thread owner or a [[`weakref`]]
  + …never a one-thread owner or a {{`weakref T`}}
  - …one-thread owner or a `weakref` link [[to one]]
  + …and pass `fn() thread_shared count: count.n.fetch_add(1)`"). {{Where the type is not a `once fn`, several threads may run one value of it at once, so its state is `ThreadShared` too (`[THR-9]`; a `copy of` a `Cell` in a function given to `handlers: Array[thread_shared fn()]` is `E7015` where it is given, `[CLO-16]`), while a `thread_shared once fn()` is handed over and run once (`[THR-10]`) and needs only `Send` state.}}

### THR-2 (p11-concurrency.md; 749 words changed)
  - …So no such owner, and no [[`weakref` link to such a value,]]
  + …So no such owner, and no {{`weakref T` link,}}
  - …target's owner, it is allowed only [[for]]
  + …target's owner, it is allowed only {{through}}
  - …it is allowed only for a [[`thread_shared` target,]]
  + …it is allowed only for a {{`weakref thread_shared T`,}}
  + …only for a `thread_shared` target, whose {{target's}}
  - …a `thread_shared` target, whose counts are [[atomic]]
  + …a `thread_shared` target, whose counts are {{kept for several threads}}
  - …counts are atomic and whose opened [[view]]
  + …counts are atomic and whose opened {{name}}
  - …atomic and whose opened view holds [[an atomic]]
  + …atomic and whose opened view holds {{a}}
  - …opened view holds an atomic count [[(`E7015`:]]
  + …opened view holds an atomic count {{(`[RC-4]`, `[RC-5]` point 2; `E7015`:}}
  - …scope, or make the player `thread_shared`"). [[An]]
  + …scope, or make the player `thread_shared`"). {{**A `weakref T` link names only an object reached through this thread's own owners** (as Rust's `Rc` never sits inside an `Arc`): `weakref x` where the place `x` is reached through a value every thread may reach, a `thread_shared` value, a `static` (not an unguarded `static mut`, which the main thread and its scoped tasks alone reach, `[THR-20]`), a `guarded by` `static mut` or a `comptime(e)` result (`[STA-1]`, `[CT-5]`), is `E7015` ("`ORIGIN` is a `static`, which every thread may read, and `weakref ORIGIN` would count its links on this thread alone; read `ORIGIN` by its name, or share it between threads: write `static ORIGIN: thread_shared Node = thread_shared Node(x=0)` in place of line 4, after which `weakref ORIGIN` is a `weakref thread_shared Node`"), at the line or at a call whose callee makes such a link to its parameter, by the callee's summary (`[OWN-13]`, `[BLD-14]` item 9); so the thread that holds a `weakref T` link owns the link's target, which is why its count needs no atomic. **An}}
  - …An object handed whole to another [[thread while]]
  + …An object handed whole to another {{thread,}}
  + …whole to another thread while a {{scoped task or a job included, leaves its}}
  - …to another thread while a `weakref` [[link]]
  + …to another thread while a `weakref` {{links behind** (verdict 114; Rust's `Rc::try_unwrap`). At the hand-over the program reads the object's weak count, one compare. With no link, the object goes as it is. With a link, its contents move to a new block for the other thread, and the old block is marked ended (its `drop` does not run; the new block's runs once, on the other thread) and stays}}
  - …a `weakref` link on this thread [[points]]
  + …a `weakref` link on this thread {{until its last link ends, so every link here finds it ended (`[WK-12]`): after `w = weakref names` and `h = thread.spawn(owns fn() owns names: names.items.len())`, `if w exists as p:` is skipped and `h.join()` gives 0. The same is done for every object the handed object owns, all the way down, whose class family carries a weak count (`[OBJ-1]`, read from the class's declaration, the same in every package; one compare each), and for no object of a family that carries none, so after `cam.target = weakref level.enemies[0]`, handing `level` over leaves `cam.target` finding enemy 0 ended. The walk follows the handed value's fields and elements by their types, through `owns` blocks, `Option`s and collections (a handed value owns no `shares` value and holds no `weakref T` link, `[THR-8]`), and stops}}
  + …link on this thread points at {{a `thread_shared` value, whose counts are kept for several threads. A value whose layout the handing package cannot see, an object held `dyn I`, an object of an `open` or `abstract` class (`[CLS-4]`), or a function value's state, is walked by the **hand-over routine** its own package compiled beside its `drop` and kept in its `TypeInfo` (`[RT-3]`; a generic instance's by the package that instantiates it, as its `drop` is, `[MONO-1]`), so a kind declared in a package the handing package never saw is walked as any other (`[BLD-16]`); a kind that carries no weak count and owns nothing that does has no routine there, one load and one test. A value whose type can own no object of a carrying family is handed over with no walk at all. No package needs to know which classes the program links to. A line that makes a value `thread_shared` (`t = thread_shared Tree(items=[item])`, `[THR-1]`), and a store under a lock into a field of one (`t.items.push(owns item)`, `[THR-18]`), makes every class object the stored value owns reachable from any thread, and is such a hand-over for each of them: the compare is made there, so a `weakref T` link made before}}
  + …on this thread points at it {{(`w = weakref item`) finds its object ended afterwards, and no plain weak count}}
  - …this thread points at it is [[`E7015`, since this thread could open]]
  + …this thread points at it is {{ever touched from two threads. An object whose address C keeps (`[FFI-35a]`) never moves: a link to it at}}
  - …since this thread could open the [[link while]]
  + …since this thread could open the {{hand-over stops}}
  - …could open the link while the [[other thread ends the object]]
  + …could open the link while the {{program}}
  - …the object ("`names` goes to another [[thread, and `w` (line 15),]]
  + …the object ("`names` goes to another {{thread while `w`,}}
  - …a `weakref` link on this thread, [[still]]
  + …this thread, still points at it, {{and C keeps its address (`register(ref_to_ptr(ref mut names))`, line 4, an `unsafe` call whose `p` is `retained`),}}
  - …thread, still points at it, so [[this]]
  + …thread, still points at it, so {{it cannot move; end the link first: `w = None`"). A hand-over to another}}
  - …points at it, so this thread [[could read]]
  + …points at it, so this thread {{is a reshape of what is handed over (§VII.1): no view into it, a `sharedref mut` link into a list of struct values included, is used after}}
  - …so this thread could read it [[while the]]
  + …so this thread could read it {{(`E3060`), and a view opened from a link is not used after a hand-over that may move its object (`[RC-5]` point 1). The}}
  - …read it while the other thread [[ends it; give]]
  + …read it while the other thread {{never touches this thread's counts:}}
  - …other thread ends it; give the [[thread a copy of its own: write `h = thread.spawn(fn() copy of names: names.items.len())` in place of line 16").]]
  + …other thread ends it; give the {{old block stays here with the links, and the new one starts with none.}}

### THR-7 (p11-concurrency.md; 5 words changed)
  - …on several threads, its counts are [[atomic,]]
  + …on several threads, its counts are {{kept for several threads (`[RC-4]`),}}

### THR-13 (p11-concurrency.md; 15 words changed)
  - …(`[THR-21]`); * an `Atomic`, `Mutex`, `RwLock`, [[channel end]]
  + …(`[THR-21]`); * an `Atomic`, `Mutex`, `RwLock`, {{`Sender`}}
  - …it directly, and its counts are [[atomic (`[THR-1]`);]]
  + …it directly, and its counts are {{kept for several threads (`[THR-1]`, `[RC-4]`);}}
  + …only by the main thread and {{the tasks and jobs of}}
  - …by the main thread and its [[scoped tasks,]]
  + …by the main thread and its {{scopes,}}

### THR-10 (p11-concurrency.md; 23 words changed)
  + …above `pub fn spawn(owns f: thread_shared {{once}}
  + …thread_shared fn() -> R) -> JoinHandle[R]:`, {{which runs it once and so accepts a function that gives its captures away as well as one that does not (`[CLO-6]`),}}

### THR-17 (p11-concurrency.md; 18 words changed)
  - …mut` captures, and a `weakref` capture [[of]]
  + …mut` captures, and a `weakref` capture {{whose link is}}
  - …and a `weakref` capture of a [[value that is not `thread_shared`,]]
  + …and a `weakref` capture of a {{`weakref T`,}}
  - …one allocation and copy per task): [[`thread.spawn(fn()]]
  + …one allocation and copy per task): {{`thread.spawn(owns fn()}}
  - …end of the `with scope` block [[(`scope.spawn(fn()]]
  + …end of the `with scope` block {{(`scope.spawn(owns fn()}}
  - …gone; give it its own copy: [[`thread.spawn(fn()]]
  + …gone; give it its own copy: {{`thread.spawn(owns fn()}}
  - …text; give it its own copy, [[`scope.spawn(fn()]]
  + …text; give it its own copy, {{`scope.spawn(owns fn()}}
  - …it look at this round's text, [[`scope.spawn(fn()]]
  + …it look at this round's text, {{`scope.spawn(owns fn()}}

### THR-3 (p11-concurrency.md; 4 words changed)
  - …g = m.lock():` and, inside it, [[`g.push(1)`,]]
  + …g = m.lock():` and, inside it, {{`g.push(owns 1)`,}}
  - …it, `g.push(1)`, above line 11, and [[`scope.spawn(fn():]]
  + …it, `g.push(1)`, above line 11, and {{`scope.spawn(owns fn():}}

### THR-18 (p11-concurrency.md; 111 words changed)
  + …x.lock:` holds it for the block. {{The block holds a read-only view of `x.lock` whose life is the block, as `with g = m.lock():` holds `g` (`[THR-3]`, `[BRW-1]`, `[BRW-2]`): while it runs, `x.lock` is not given another value, swapped or taken out, and `x` is not handed over, ended or given another value, through any name (`E3021` for the change of what the view reads, `E3060` where the change reshapes or ends what it points into: "`log.lock = Lock()` (line 10) replaces the lock this block holds (line 9), whose end would release a lock never taken; give the log a new lock after the block, or keep the one made with it").}}
  - …is `with log.lock:` and, inside it, [[`log.lines.push(String("hi"))`).]]
  + …is `with log.lock:` and, inside it, {{`log.lines.push(owns String("hi"))`).}}
  - …[] fn push(s: StrView): with this.lock: [[this.lines.push(String(s))]]
  + …[] fn push(s: StrView): with this.lock: {{this.lines.push(owns String(s))}}

### THR-19 (p11-concurrency.md; 193 words changed)
  - …thread holds is refused when compiling.** [[Inside]]
  + …thread holds is refused when compiling.** {{While this thread holds}}
  - …is refused when compiling.** Inside a [[lock's]]
  + …is refused when compiling.** Inside a {{lock, inside its}}
  + …compiling.** Inside a lock's `with` block, {{from `g = m.lock()`, `g = rw.read()` or `g = rw.write()` to the last use of `g` or of a value holding it (`[BRW-2]`), or for as long as a guard made as a temporary lives (`[EXP-4]`: to the end of its statement, or of the block or arm whose name views what it guards),}}
  + …lock of the kind this thread {{holds, a `Mutex`, a `Lock` or an `RwLock` (its `write()` while this thread}}
  + …of the kind this thread holds {{its `read()`, or the reverse, included)}}
  - …changes what the held lock guards; [[and]]
  - …not proved to enter past it [[(`[THR-18]`).]]
  + …not proved to enter past it {{(`[THR-18]`), a foreign call not declared `@ffi(no_callbacks)` being such a call by the callbacks C keeps (`[EFF-26]`); a first access of a `static` whose initialiser runs on first access (`[STA-3]`), which counts as a direct call of that initialiser; and a second `read()` of an `RwLock` this thread holds `read()`, since a writer that arrived between the two would wait for the first and the second for it ("`rw.read()` (line 9) takes a read lock this thread already holds (line 7), and a writer arriving between them would make each wait for the other for ever; use the guard `g` already held, or take the second read after the first ends").}}

### THR-20 (p11-concurrency.md; 122 words changed)
  + …the main thread and the tasks {{and jobs}}
  + …main thread and the tasks of {{the}}
  - …and the tasks of scopes it [[opens,]]
  + …and the tasks of scopes it {{opens (`[THR-11]`, `[JOB-2]`),}}
  + …`tick`'s body inside `with TICK_LOCK:`"). A {{`static mut` the tasks of a scope reach is a place `[THR-11]` checks as any other: two tasks changing it, or one reading what another changes, is `E7007`, other than by a reduction, under a lock or through an `Atomic`. A}}
  - …itself (`Lock`, `Atomic`, `Mutex`, `RwLock`, a [[channel end)]]
  + …itself (`Lock`, `Atomic`, `Mutex`, `RwLock`, a {{`Sender`; a `Receiver` is not `ThreadShared`, `E7002`, `[STA-1]`)}}
  - …`drop` runs where the last owner [[ends.**]]
  + …`drop` runs where the last owner {{ends**, or, for a value of a plain type queued back to the thread that made it (`[RC-4]` points 2 and 4), at that thread's next check point, where a plain `drop` reaches nothing this Part judges; a `drop` that is not plain never waits in a queue, so it runs where its last owner ends, on the thread this rule judges.}}
  - …= x * x with RESULTS_LOCK: [[RESULTS.push(r)]]
  + …= x * x with RESULTS_LOCK: {{RESULTS.push(owns r)}}
  - …with RESULTS_LOCK: for r in RESULTS: [[handles.push(thread.spawn(fn()]]
  + …with RESULTS_LOCK: for r in RESULTS: {{handles.push(owns thread.spawn(owns fn()}}

### THR-15 (p11-concurrency.md; 111 words changed)
  + …bound and allocates as it grows. {{A channel carries its values to another thread, so `T` MUST be `Send` (`[THR-8]`): `channel[T]` or `channel_unbounded[T]` for a `T` that is not is `E7015` at the line that makes the channel, naming what keeps `T` on one thread (verdict 164: "`Watcher` holds `target`, a `sharedref mut Player` link, which other names on this thread may change, so no channel carries a `Watcher`; let a scoped task use `w` where it is (`scope.spawn(owns fn(): ...)`) instead of sending it"); so `Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, for every channel, and a one-thread owner never crosses through one (`[THR-2]`).}}
  - …allocates as it grows. `Sender` is [[`Send`, `ThreadShared` and]]
  - …is another sender of the same [[channel); `Receiver` is `Send`. `tx.send(v)]]
  + …is another sender of the same {{channel). `tx.send(owns v)}}
  - …`Sender` queued in its own channel [[(`tx.send(copy]]
  + …`Sender` queued in its own channel {{(`tx.send(owns copy}}

### THR-5 (p11-concurrency.md; 23 words changed)
  + …`return` is computed before the join. {{A task runs on a thread the runtime owns, never on the thread that opened the scope (`[RC-4]` point 4 rests on it).}}

### THR-11 (p11-concurrency.md; 439 words changed)
  + …function, `[R]` above `fn spawn(owns f: {{once}}
  - …spawn(owns f: fn() -> R) -> [[ScopedJoinHandle[R]:`.]]
  + …spawn(owns f: fn() -> R) -> {{ScopedJoinHandle[R]:` (run once, so it accepts a function that gives its captures away, `[CLO-6]`).}}
  + …tasks and the parent until the {{task's}}
  + …and the parent until the join: {{`h.join()` on its handle, or the end of the `with scope:` block for a task not joined before it (verdict 104). After `h.join()` the parent may change or read what that task used, in the same block (`counts[0] += 1` after `h.join()`), checked only against the tasks not yet joined:}}
  - …other than by a reduction (`[THR-21]`), [[under]]
  + …other than by a reduction (`[THR-21]`), {{through the lock that guards the place (a `guarded by` field inside `with` its lock, `[THR-18]`;}}
  - …by a reduction (`[THR-21]`), under a [[lock,]]
  + …by a reduction (`[THR-21]`), under a {{`Mutex`'s or `RwLock`'s content through its guard, `[THR-3]`; a `with` block of a lock that does not guard the place is no exception), through an `Atomic`,}}
  - …(`[THR-21]`), under a lock, or through [[an `Atomic`.]]
  + …(`[THR-21]`), under a lock, or through {{a `Sender` by its own operations (`[THR-15]`). A `Receiver` is one consumer: it is used by one task, or by the parent, until the join, as any other place (`E7007`: "two tasks take values out of `rx`, which one consumer owns; give each task a channel of its own, or let one task receive and hand the values on").}}
  - …task that pushes to `names` (`fn(): [[names.push(String("x"))`),]]
  + …task that pushes to `names` (`fn(): {{names.push(owns String("x"))`),}}
  - …fix would leave the walk refused. [[A]]
  + …fix would leave the walk refused. {{**A}}
  - …leave the walk refused. A task [[makes, ends]]
  + …leave the walk refused. A task {{touches no one-thread count.** Making}}
  - …refused. A task makes, ends or [[opens]]
  + …refused. A task makes, ends or {{ending}}
  - …task makes, ends or opens a [[`weakref` link]]
  + …task makes, ends or opens a {{`shares` or `shares mut` owner, making, ending or opening a `weakref T` link, and ending an object whose class family carries a weak count (`[OBJ-1]`), each write a header with plain instructions (`[RC-4]`, `[WK-11]`), which the parent may write at the same moment. So a task does each of them}}
  - …or opens a `weakref` link only [[to]]
  + …or opens a `weakref` link only {{for}}
  - …`weakref` link only to a `thread_shared` [[target,]]
  + …`weakref` link only to a `thread_shared` {{target (`[THR-2]`), worked out from its body}}
  + …only to a `thread_shared` target, and {{every function it may run (`[EFF-24]`, `[DRP-8]`): a task that may end a `shares` owner, or may end an object of a class family that carries a weak count, is `E7015` at the spawn ("the task started on line 11 clears `a`, which holds `shares Player` owners, whose count is kept with plain adds on one thread; clear it after the join: move `a.clear()` below the `with scope` block"; "the task started on line 18 may end the `Player` that `w` links (`q = Player()`), while this thread may test `w`; give `q` its new value after the `with scope` block"). A `thread_shared` value a task or a job}}
  + …to a `thread_shared` target, and makes {{has}}
  - …a `thread_shared` target, and makes no [[`shares`]]
  + …a `thread_shared` target, and makes no {{one-thread count either: it is marked handed over at its making (`[RC-4]` point 4, by its thread), so nothing is ever queued to the task's thread to be ended after the join, and its `drop` runs where its last}}
  - …target, and makes no `shares` owner [[(`[THR-2]`).]]
  + …target, and makes no `shares` owner {{ends, which this rule judges until the join and `[THR-20]` after it. A value a task or a job takes with `owns` is handed to another thread: `[THR-2]`'s compare and move happen at the spawn, so every link on this thread then finds it ended, and the task ends it with no header this thread reads.}}

### THR-21 (p11-concurrency.md; 467 words changed)
  - …* `[THR-21]` **Reductions.** A task's [[`total += e` (and `-=`, `*=`, `|=`, `&=`, `^=`, `max`, `min`, `push`, `extend`, `add`, `m[k] op= e`)]]
  + …* `[THR-21]` **Reductions.** A task's {{statement of one of these forms}}
  + …`add`, `m[k] op= e`) on a {{place `P` that the task otherwise neither reads nor writes, `P op= e` for `+`, `-`, `*`, `|`, `&`, `^`, `P = max(P, e)` and `P = min(P, e)`, `P.push(owns e)`, `P.extend(owns e)` and `P.add(owns e)`, and `P[k] op= e` on a `Map`, where `P` is a}}
  + …`m[k] op= e`) on a local, {{a `static mut` (`[THR-20]`),}}
  - …op= e`) on a local, or [[on]]
  - …of an object the tasks reach [[(`scope.spawn(fn()]]
  + …of an object the tasks reach {{(`scope.spawn(owns fn()}}
  - …included, is a reduction: each task [[adds into]]
  + …included, is a reduction: each task {{gathers its changes in}}
  - …place at the join, in spawn [[order.]]
  + …place at the join, in spawn {{order, each by the place's own operator. The accumulator of `+`, `*` and `^` starts from the operator's neutral value (0, 1, 0), that of `push`, `extend` and `add` empty, that of `P[k] op= e` an empty `Map`; for `-=` it gathers what the task takes away (`acc += e`) and the join takes it from the place (`P -= acc`), so an unsigned place never goes below zero in a task whose one-by-one result would not (`hp: u8 = 10` with `-= 1` and `-= 2` from two tasks leaves 7, and a task taking away more than 255 in all stops there); on a signed place `+=` and `-=` are one operator, the accumulator holding the signed change from 0; for `max`, `min`, `&` and `|`, which applied twice give what applied once gives, the accumulator starts from the place's value at the spawn, which nothing changes before the join, so a task whose statement never runs leaves the place as it is. All the tasks' statements on one place use one operator: `+=` in one task and `*=` in another, or `+=` and `-=` on an unsigned place, is `E7007`, whose message gives the lock, since one accumulator holds one operator's changes and the joined result would otherwise depend on which task's statement is applied first.}}
  - …and write `println(total)` above line 15"). [[A]]
  + …and write `println(total)` above line 15"). {{Each task's accumulator has the place's type, and its operations are checked as the place's are, and so are the combining operations at the join (verdict 105): a}}
  - …line 15"). A whole-number total is [[always]]
  + …one adding one by one gives {{whenever no task's own total overflows}}
  - …`hp` is 10, leave 7 every [[time);]]
  + …`hp` is 10, leave 7 every {{time), and a task whose own total overflows stops the program there (`[TYP-8]`), even where adding one by one would not (`[[-100], [100, 100]]` summed into an `i8` by two tasks stops in the second, at 100 + 100, where adding one by one gives 100);}}
  - …part in [[0.1], [0.2, 0.3]]:` with [[`scope.spawn(fn()]]
  + …part in [[0.1], [0.2, 0.3]]:` with {{`scope.spawn(owns fn()}}
  + …x`) need a lock (`E7007`). A {{field of a `thread_shared` value is not a reduction place: its owners never change it directly (`E7014`, `[THR-1]`), and the message gives the `Atomic` or the `guarded by` field. For `P[k] op= e` the accumulator is a `Map` of the task's own keys, and the join applies `P[k] op= acc[k]` for each key in spawn order, inserting a key absent from `P` with what the operator applied to the value type's zero and the accumulator gives (the accumulator itself for `+`, `|` and `^`, `0 - acc[k]` for `-`). A}}

### PAR-2 (p11-concurrency.md; 189 words changed)
  + …body MUST NOT contain `break`, `return`, {{`?`,}}
  - …or a `continue` naming an outer [[loop,]]
  + …or a `continue` naming an outer {{loop (`E7010`, for `?`: "`?` on line 4 would return from `parse_all` in the middle of the parallel loop; keep each result instead: write `out[i] = parse_int(lines[i])` into an `Array[Result[int, ParseError]]` and check them after the loop"),}}
  - …rules of a scoped task (`[THR-11]`, [[`[THR-2]`).]]
  + …rules of a scoped task (`[THR-11]`, {{`[THR-2]`), except that a total the iterations combine is declared (`[PAR-3]`): a task's undeclared `total += x` is a reduction, and a loop body's is `E7010`, whose message gives the `reduce=` line.}}
  - …each place `P` written in the [[body]]
  + …each place `P` written in the {{body, other than a declared reduction variable or a place `[THR-11]` excepts, changed by its own operations (an `Atomic`, a `Sender`, a `Mutex`'s or `RwLock`'s content through its guard, a `guarded by` field inside `with` its lock),}}
  - …`P` is the elements of a [[`MutSpan`]]
  + …`P` is the elements of a {{contiguous list the loop reaches by a name that may change it, an `Array[T]`, a `[T; N]`, a `MutSpan[T]`}}
  + …a `MutSpan` or an `SoA` column {{(`fn brighten(ref mut pixels: Array[float], gain: float):` with `pixels[i] *= gain` compiles as}}
  - …`MutSpan` or an `SoA` column the [[loop uses,]]
  + …`MutSpan` or an `SoA` column the {{`MutSpan` form does), which the body neither grows nor shrinks (a `push`, `pop`, `insert`, `remove` or `clear` of the list in the body is a write of the list itself, reached through no index, which (a) refuses),}}
  - …writes nothing else but declared reductions [[(`[PAR-3]`);]]
  + …writes nothing else but declared reductions {{(`[PAR-3]`) and the places `[THR-11]` excepts, each by its own operations;}}
  - …in place of line 3, and [[`players.push(Player(hp=10))`]]
  + …in place of line 3, and {{`players.push(owns Player(hp=10))`}}

### JOB-2 (p11-concurrency.md; 2 words changed)
  + …takes a plain function (`owns f: {{once}}
  + …run, `pub fn submit(owns f: thread_shared {{once}}

### JOB-3 (p11-concurrency.md; 4 words changed)
  - …* `[JOB-3]` `s.submit_after(deps, [[f)`]]
  + …* `[JOB-3]` `s.submit_after(deps, {{owns f: once fn())`}}

### TCB-4 (p21-annex-c.md; 27 words changed)
  - …list, with its version and known-defect [[list.]]
  + …list, with its version and known-defect {{list; an external library a header names is listed with the header and line that names it (`[FFI-51]`).}}
  + …block, `unsafe fn`, `unsafe import c` {{and `unsafe import cpp`}}
  - …the C++ bridge section each `@ffi(const)` [[(`[FFI-40a]`).]]
  + …the C++ bridge section each `@ffi(const)` {{(`[FFI-40a]`) and `@ffi(std_function, safe)` (`[FFI-17a]`).}}

### GPU-2 (p22-annex-d.md; 53 words changed)
  - …`end_frame(owns frame: Frame)` takes it over [[(`device.end_frame(frame)`]]
  + …`end_frame(owns frame: Frame)` takes it over {{(`device.end_frame(owns frame)`}}
  + …one twice, is refused when compiling. {{A `Frame` whose owner ends without `end_frame` is ended by its `drop`, which ends the frame as `end_frame` does with nothing recorded: the device submits nothing for it and marks its fence observed, so no later `begin_frame` waits for it, and what was made in its arena ends with it (`[GPU-3]`).}}

### GPU-12 (p22-annex-d.md; 6 words changed)
  + …asserted fact that `ember tcb` lists {{for every `Device` an overlay implements}}
