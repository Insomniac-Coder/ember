---

# Appendix F — Glossary

| Term | Meaning |
|---|---|
| **access state** | a GPU resource's lifecycle: owned by the CPU, recorded, in flight, retired (Annex D) |
| **arena** | a memory manager that hands out memory by bumping a pointer and frees it all at once; named with `using` (§IX) |
| **capture** | a name a lambda uses from the function that made it: the real variable unless the lambda's first line names it with a word after its brackets (`fn() copy of score:`, `fn() owns n:`), which makes it part of the function value's own state |
| **class** | a type whose objects live each in a heap block of their own, with identity (`is`) and subclasses; an object has one owner unless a line says `shares`, and an 8-byte header (Part VIII) |
| **contract** | an attribute stating an effect a function must not have (`@noalloc`), checked over the call graph (§X.2); for FFI, the facts about a foreign pointer that a header does not state (§XVI.4) |
| **`Copy` type** | a type whose `=` copies a value bit for bit, the two copies independent: numbers, `bool`, `ref T`, `Span[T]`, `StrView`, and structs made only of these; class objects, `shares` values, links, `ref mut` views and `MutSpan`s are not `Copy` |
| **copy of** | a new value equal to the old one, made by its type's `Clone`, on which nothing done to the other is seen; before a link it copies the target, never the link |
| **count, weak count** | the number of owners in the header of a value with several owners, and the number of `weakref` links to an object, which keeps its memory block (not the object) until the last link ends; each is compared with its limit before every +1 (its maximum for a one-thread count; half its range for a `thread_shared` count, which threads add to at once, `[RC-4]`), and the program stops there |
| **deferred-drop region** | a walk during which an object whose last owner ends is set aside whole, with everything it owns, marked ended at once and freed when the walk ends |
| **effect** | a kind of thing a function may do — allocate, block, lock, perform I/O, panic, run a run-time check — inferred by the compiler (§X.1) |
| **`.embind`** | the cached, content-addressed result of importing a header with its flags and overlay (`[FFI-14]`) |
| **ended** | the state of an object from the moment its last owner lets it go, before its `drop` runs; a `weakref` link to it then finds nothing. Ending an object counts as a call of its `drop` |
| **enforcement ladder** | prove a property when compiling; a check runs while the program runs only where the property is data the program computes as it runs (an index, an overflow, a `weakref` link's target); what can be proved neither way is refused with the line to write, or allowed only inside `unsafe` (`[PHIL-8]`) |
| **exclusivity** | the rule that while a `ref mut` name is used, its place is read or changed through no other name; checked when compiling, never while the program runs |
| **`exists as`** | the opening of an `Option` or a `weakref` link: `if cam.target exists as p:` runs its block when there is something, and names it `p` |
| **first line** | a function's signature: its parameters, each with the word in front of its name, `owns this`, the result type and `from`; it says nothing about what the function changes or reads |
| **fix-it** | a suggested edit attached to a diagnostic that tools can apply mechanically |
| **function value** | a function or lambda kept in a variable, a list or a field: two words, its code and its state (what it captured with a word); its type says its parameters, its result and whether any thread may run it (`thread_shared fn()`) |
| **generator frame** | a `gen fn`'s suspended state as a sized value; resuming runs the body to the next `yield` (`[CORO-1]`) |
| **hand-over** | the giver's name ends and the receiver becomes the owner, for every type, numbers included: `b = a`, an argument for an `owns` parameter, a capture `owns x` |
| **handle** | a token that names something elsewhere: `Handle[T]`, a generational index into a `Pool` (`[HND-1]`), a `JoinHandle`, a C object's `@ffi(handle)`; never a class object |
| **interior mutability** | a change through a name that only reads, which only the types made to be shared between threads allow, by their own operations: `Atomic`, `Mutex`, `RwLock`, channel ends, and `guarded by` fields under their lock |
| **kept callback** | a function value C keeps after the call that hands it over (`callback(retained)` or `callback(once)`) and runs during later foreign calls |
| **kept view** | a view stored in a struct or a list; the value that holds it lives no longer than what it points into |
| **kind test** | `x is T`, which asks whether a value of an interface or base-class type is a `T`; `if s is Circle as c:` names it |
| **link** | a field or value that names an object it does not own alone: `weakref T`, `shares T`, `shares mut T`, `thread_shared T`, `sharedref mut T` |
| **memory manager** | where a value's heap memory comes from: the built-in one, or one named with `using` (its kind in a type, its instance on the line that makes the value) (§IX) |
| **mode** | the word in front of a parameter's name, which sets what the function may do with it, for every type: none (reads the caller's value), `ref mut` (changes the caller's place), `const ref mut` (changes it and is never given another ref), `owns` (takes the value over) (`[FN-2]`) |
| **native island** | a subsystem kept in C or C++ behind a typed boundary rather than imported (Annex C) |
| **niche** | an invalid bit pattern of a type that `Option` uses for `None`, so `Option[T]` costs no space |
| **noalias** | C's `restrict`: the promise to the C compiler that two pointers never point at the same memory; every `ref mut` parameter is compiled with it, `this` never |
| **`Nondet`** | the effect of an operation whose result may differ between runs or machines (`[DET-2]`) |
| **overlay** | an Ember file that states contracts, renames and wrappers for an imported header (§XVI.4) |
| **owner** | the one place (a local, a field, a list element) that ends a value when it ends; a value made shared has several |
| **panic** | a failure that means the program is wrong; it prints a message and aborts the process (`[PAN-1]`) |
| **parent link** | a `weakref` field naming the object that owns its holder (`parent: weakref Node`), set by the program (`child.parent = weakref this`); read with no test where the compiler proves its target lives |
| **permanent thunk** | the fixed address standing for a reloadable function; a reload changes its target, never its address (`[HR-6]`) |
| **place** | where a value lives: a local, a field, a list element, an enum variant's payload, or what a `ref mut` name stands for |
| **position** | a name for a list element made by indexing, which finds the element again at each use; the list may grow while it is used, and a removal, reorder or clear ends it |
| **profile** | `debug`, `development` or `release`: optimisation and diagnostic settings that never change meaning (`[PRF-1]`) |
| **proven fact** | a fact a verifying operation establishes for two names (`assert_disjoint` after its call, `check_disjoint` inside its `Ok` branch), dropped when either name is given another value (`[DSJ-2]`) |
| **read-only view** | every view except `ref mut T`, `sharedref mut T` and a `MutSpan` held by a `ref mut` name; it promises that what it reads does not change while it is used |
| **real variable** | the very variable a lambda uses without a word, not a copy: `b.on_click(fn(): score += 1)` changes the caller's `score` |
| **reason code** | why a run-time check exists (`[EFF-11]`) |
| **reduction** | a combining change (`total += e`, `max`, `push`) made by scoped tasks to one place: each task keeps a private accumulator, combined at the join in spawn order |
| **reload manifest, reload transaction, safe point** | the image section listing what can be reloaded (`[HR-8]`); the plan and prepare phases that may fail without effect (`[HR-2]`); the poll at which no thread runs Ember code (`[HR-3]`) |
| **reshape** | a change that can free or move memory: replacing a value that owns memory, adding to or removing from a collection, changing an enum's variant, ending an object's last owner; writing a number is not one |
| **Safe Ember** | code outside `unsafe:` blocks, an `unsafe fn` body's included (each of its unsafe operations sits in an `unsafe:` block of its own); the memory-safety guarantee covers it (`[PHIL-10]`) |
| **`Send`, `ThreadShared`** | may be handed to another thread; several threads may hold and read one value at once; both inferred from the words a value is written with (§XI.1) |
| **shape** | a category of error with a required help text (§XVII.6) |
| **shareable generic** | one whose type parameter is used only to call its bounds' methods, so one shared body can serve every instance (`[MONO-5]`) |
| **`shares`, `shares mut`** | one more owner of a value on this thread, counted: a `shares` owner reads it, a `shares mut` owner may also change it |
| **`sharedref mut`** | a link that may change its target and that several holders, and the owner, may share; on two threads it is changed only under a lock, through `Atomic`, or by a reduction |
| **SoA** | structure of arrays: one array per field (`SoA[T]`, §XII.1) |
| **source parameter** | a parameter a returned view may point into without `from`: every view argument, or the one named by `from` (`[LT-1]`) |
| **static** | a program-wide value with a fixed address that never changes; `static mut` is a module variable that may, reached only by the main thread and its scoped tasks unless `guarded by` a `static` lock |
| **`static fn`** | a function in a type body that has no object (`this`), called on the type: `Counter.starting_at(5)` |
| **summary** | what the compiler works out from a function's code, with no word written: what it may change, read and end, through which chain of calls, which locks it may take, and which view parameters it uses after a line that may end what they view; kept in a package's interface file |
| **`this`** | inside a method, the object the method was called on; a method does not list it among its parameters |
| **`thread_shared`** | a value with several owners on any threads, counted atomically, changed only through `Atomic`, a lock, a channel or `guarded by` fields; `thread_shared fn()` is a function type whose functions any thread may run |
| **trampoline subclass** | the generated C++ subclass through which an Ember class extends a C++ base (`[FFI-39]`) |
| **unseen code** | code whose body the compiler cannot read, which is only a foreign (C) function: a call of one counts as possibly running every callback C keeps, unless its declaration says `@ffi(no_callbacks)` |
| **view** | a name or value that points into a place someone else owns and may not outlive it: `ref T`, `ref mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, loop or pattern name, a name opened with `exists as` or named by a kind test, a name for a part (`[TYP-34]`) |
| **`weakref`** | a link that does not keep its target alive; it is read by opening it (`exists as`), which finds nothing once the target has ended |
| **zero-cost** | for a use whose checks are all discharged, emitted code with nothing equivalent C would not contain (`[COST-1]`) |
