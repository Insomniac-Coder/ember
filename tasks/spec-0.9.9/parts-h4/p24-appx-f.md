---

# Appendix F — Glossary

| Term | Meaning |
|---|---|
| **access (long-term, instantaneous)** | a use of a class object's field; long-term accesses span time and are checked for exclusivity, instantaneous ones (`Copy` field reads and writes) are not (§VIII.3) |
| **access state** | a GPU resource's lifecycle: owned by the CPU, recorded, in flight, retired (Annex D) |
| **arena** | an allocator that hands out memory by bumping a pointer and frees it all at once (§IX) |
| **borrow** | a reference to a place that does not own it; shared (read) or mutable (read and write) |
| **class** | a reference type: instances live on the heap, handles are counted, identity is observable (Part VIII) |
| **contract** | an attribute stating an effect a function must not have (`@noalloc`), checked over the call graph (§X.2); for FFI, the facts about a foreign pointer that a header does not state (§XVI.4) |
| **effect** | a kind of thing a function may do — allocate, block, lock, perform I/O, panic, run a run-time check — inferred by the compiler (§X.1) |
| **`.embind`** | the cached, content-addressed result of importing a header with its flags and overlay (`[FFI-14]`) |
| **enforcement ladder** | prove a property statically, else check it at run time, else require `unsafe` (`[PHIL-8]`) |
| **exclusivity** | the rule that a write access to an object never overlaps another access to it (§VIII.3) |
| **fix-it** | a suggested edit attached to a diagnostic that tools can apply mechanically |
| **generator frame** | a `gen fn`'s suspended state as a sized value; resuming runs the body to the next `yield` (`[CORO-1]`) |
| **handle** | a class reference (counted), or a generational index into a `Pool` (`Handle[T]`) |
| **interior mutability** | mutation through a shared borrow, permitted by `Cell`, `RefCell`, `Atomic` and the locks |
| **loan** | the record a borrow creates, which the borrow checker follows to the borrow's last use (§XVIII.4) |
| **mode** | how a parameter is passed: borrowed (the default), `mut` or `owned` (`[FN-2]`) |
| **move** | transfer of a value's ownership; the source can no longer be used |
| **native island** | a subsystem kept in C or C++ behind a typed boundary rather than imported (Annex C) |
| **niche** | an invalid bit pattern of a type that `Option` uses for `None`, so `Option[T]` costs no space |
| **`Nondet`** | the effect of an operation whose result may differ between runs or machines (`[DET-2]`) |
| **overlay** | an Ember file that states contracts, renames and wrappers for an imported header (§XVI.4) |
| **owned callable value** | a function value that owns its captures, stored in a field, local or collection (`[CLO-3]`) |
| **panic** | a failure that means the program is wrong; it prints a message and aborts the process (`[PAN-1]`) |
| **permanent thunk** | the fixed address standing for a reloadable function; a reload changes its target, never its address (`[HR-6]`) |
| **place** | an expression denoting a storage location: a local, field, element or dereferenced reference |
| **profile** | `debug`, `release` or `shipping`: optimisation and diagnostic settings that never change meaning (`[PRF-1]`) |
| **proof-carrying value** | a value returned by a verifying operation (`assert_disjoint`) that carries the fact it established |
| **reason code** | why a run-time check exists (`[EFF-11]`) |
| **region** | the stretch of a program during which a borrow is live; never written in source |
| **region vector** | the compiler-internal regions of a view type with several borrowed fields; erased before code generation (`[LT-14]`) |
| **reload manifest, reload transaction, safe point** | the image section listing what can be reloaded (`[HR-8]`); the plan and prepare phases that may fail without effect (`[HR-2]`); the poll at which no thread runs Ember code (`[HR-3]`) |
| **Safe Ember** | code outside `unsafe` blocks and functions; the memory-safety guarantee covers it (`[PHIL-10]`) |
| **`Send`, `Sync`** | may be moved to another thread; may be read from several threads at once (§XI.1) |
| **shape** | a category of error with a required help text (§XVII.6) |
| **shareable generic** | one whose type parameter is used only to call its bounds' methods, so one shared body can serve every instance (`[MONO-5]`) |
| **SoA** | structure of arrays: one array per field (`SoA[T]`, §XII.1) |
| **static** | a program-wide value with a fixed address; immutable, with interior synchronisation for mutation |
| **`@sync` class** | a class whose handles may cross threads; its fields are immutable after construction (`[THR-1]`) |
| **trampoline subclass** | the generated C++ subclass through which an Ember class extends a C++ base (`[FFI-39]`) |
| **value world, object world** | code over structs, views and containers, checked statically; code over class handles, counted and checked at run time |
| **view** | a value that borrows: a reference, `Span`, `MutSpan`, `str`, or a struct holding one (`[TYP-34]`) |
| **zero-cost** | for a use whose checks are all discharged, emitted code with nothing equivalent C would not contain (`[COST-1]`) |
