---

# Annex B — Hot Reload (the Dynamic profile)

Hot reload turns a source edit into running behaviour in a live process, keeping existing objects,
statics and open resources. It is part of the Ember Dynamic conformance profile (`[CONF-5]`), is
available in `debug` and `development` builds; a `release` build ignores `[build] reload` (`[MAN-7]`).

* `[HR-1]` An edit to one function body in a 50k-line reloadable package MUST be running in the process
  within one second on the reference machine (`[BUD-1]`), with all live object state preserved: budget
  B2 (`[BUD-2]`) for the build, plus load, plan, prepare and commit, measured together over a live set
  of 10,000 instances (`[BEN-8]`).
* `[BEN-8]` The end-to-end reload benchmark times three edits — a function body, a field added with a
  default, and a field whose type changes through `migrate_from` — from save to the new code running.

## B.1 Model

The unit of reload is the package. A package built with reload enabled is a shared library whose
functions are reached through permanent thunks and whose types carry schemas. A reload proceeds in
phases: **plan** (compare schemas and the facts the compiler worked out; refuse here with a reason),
**prepare** (build every migrated instance; may fail, changes nothing), **commit** (publish; cannot
fail), **reclaim** (end what the old image owned).

* `[HR-2]` *(changed in 0.9.15)* *(changed in 0.9.20)* **Reload is transactional.** A failure in
  plan or prepare leaves the process running the old image, with no instance changed, handed over or
  ended, and a report naming the cause. Commit performs only stores: pointer stores into memory
  reserved in prepare, and, for an instance migrated in place (`[HR-15]`), the copy of its prepared
  bytes over its old block, of a size prepare computed and into memory that exists, made before the
  map is applied so that no rewritten pointer is followed into an old layout; no allocation, check,
  callback or other fallible operation. A failure in reclaim is an ordinary panic after the reload
  has succeeded.
* `[HR-2a]` *(changed in 0.9.15)* *(changed in 0.9.20)* Plan reads schemas, the facts of `[HR-44]`
  and the registered set and runs no user code. Prepare builds every migrated instance in the
  transaction arena (`[HR-37]`), an in-place one too, copies into the arena the old values of the
  fields that will not be carried over by name (removed fields, the old value of each converted
  field, and every field of an instance rebuilt by `migrate_from`), runs every `migrate_from`
  against a read-only view of the old instance, and computes the old-to-new address map; it writes
  nothing into an old block. Commit publishes the instances, patches type information, applies the
  map and repoints the thunks. Reclaim ends those saved old values, running their `drop`s and never
  the type's own `drop`, since the instance lives on in its new form (a field carried over by name
  has been handed over and is not ended), then frees each old block whose instance moved.
* `[HR-3]` *(changed in 0.9.10)* A reload is applied only inside `ember_reload_poll()`, and only
  when no registered thread is executing Ember code (each thread's Ember-depth counter is zero);
  otherwise the poll returns `EMBER_RELOAD_UNSAFE_POINT` and does nothing. The exception is
  `std.hot.checkpoint()` (`[HR-33]`), a poll the program makes itself, where its own thread's frames
  below the checkpoint stay live (`[HR-3b]`).
* `[HR-3a]` Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`),
  `thread.spawn` threads, and job workers. `debug` builds assert on entry to Ember code from an
  unregistered thread.
* `[HR-3b]` *(new in 0.9.10)* *(changed in 0.9.15)* *(changed in 0.9.20)* At a checkpoint the frames
  below it run their old code on after the swap. Each function of a reloadable image keeps a record
  of which of its calls it is in, and the image lists, for each call, the types whose layout the
  rest of the function depends on once the call returns: those a value it still uses is of, holds,
  or views (a view into a field, an element or a part depends on the type that holds the part and on
  every type on the path to it, through links and `owns` blocks alike, read from the places the view
  carries, `[BCK-1]`); those its old code reads or writes fields of, builds, holds, passes or gets
  back by value, indexes or allocates, and those the functions it calls directly depend on (they run
  old code too; a call through a function value may reach any); and, beside the types, the functions
  the rest of the function may still call, whose facts `[HR-44]` compares. A layout change applies
  at the checkpoint when no frame below it depends on the changed type: the instances migrate as at
  a poll (`[HR-14]`); an instance a frame's view points into is not changed in place while that
  frame lives: a layout change of its type is a dependence of the frame, and is refused naming the
  function, the line of its call and the view, and one that must move is refused too, since a
  frame's views are not counted (`[HR-15a]`). A change a frame depends on is refused, naming the
  function and the line of its call. A static's change still waits for a poll from the host.
* `[HR-44]` *(new in 0.9.15)* *(changed in 0.9.20)* **The facts the compiler worked out are part of what a reload
  compares.** Every rule that decides when compiling whether a line is safe — a loop and a call that
  may change what it walks, a read-only view and a call that may change what it reads, a view
  argument and a callee that may end or move what it points into, a view of an object reached
  through a link and a call that may end the object, a lock this thread holds and a call that may
  take it, a parent link read with no header test, what a thread or a kept callback changes — rests
  on what each function may change, read and end, worked out from its body and from every function
  the program can put in a function value, behind an interface, `dyn` or virtual method, or in C's
  keeping (`[EFF-4]`). The image records these facts for every reloadable function, as a package's
  interface file does, and with them the proofs that read through a reloadable callee (a `@parallel`
  loop's or a vectorised loop's, `[PAR-2b]`, `[SIMD-5]`) and the lock each field is `guarded by`
  (`[HR-11]`), so that a reload that changes the callee so the proof no longer holds, or changes a
  field's lock while code the reload does not rebuild uses the field, is refused in plan (`E9030`),
  naming the loop and the callee, or the field, its new lock and the code that uses it. Plan refuses
  a reload (`E9030`) whose new image changes such a fact of a function that code the reload does not
  rebuild was checked against: a frame below a checkpoint that may still call it (`[HR-3b]`), a
  suspended generator (`[HR-45]`), and another package compiled against this package's facts. The
  report names the function, the fact and the caller that relies on it ("`note` now adds to `LOG`,
  and `run`, still running below the checkpoint on line 10, walks `LOG` across its call of `note` on
  line 11; restart, or let `run` return before the reload"). Code the reload rebuilds is checked
  again when it compiles, so a change that only it relies on is no refusal. A change to a `@cached`
  method's body clears the result each instance of its type keeps: the kept result is migrated as a
  removed field and an added one with no value (`[HR-14]`), and is computed again on the next call.
  Whether a type is plain (`[RC-4]` point 4) is such a fact for every line that makes a
  `thread_shared` value of it; when a reload takes it away, prepare, which walks the live instances
  while every registered thread is at depth zero (`[HR-3]`, `[HR-12]`), hands over every live
  `thread_shared` value of the type as `[RC-4]` point 4 does (its home count moved into its away
  count, the handed-over mark set), and ends one it finds queued whose two counts add up to zero
  with the old image's `drop`, which is plain, so none waits in a queue whose check points may no
  longer run its `drop`. Nothing else of this runs while the program runs.
* `[HR-44a]` *(new in 0.9.20)* **A stored function is a fact too.** Plan enumerates (`[HR-15]`)
  every function value held in a live instance, a registered container, a static or a lambda's
  state, and every callback C keeps (`[HR-20]`), and compares each, by its code word, which `[HR-6]`
  makes the thunk, one pointer compare per held value, with the functions the new image's compile
  counted for that place: `[EFF-2]`'s set from the new source and `[BLD-14]` item 7's from its
  dependencies' interface files, and for C's keeping `[FFI-52]`'s. Where a held function is not
  among them, the new image's checks of that place did not count it: the reload is refused
  (`E9030`), naming the place, the function and the count ("`tree.on_visit` holds `prune` in 1
  instance, which the new `setup` no longer stores there, and the new `walk` (line 8) walks `nodes`
  across its call; keep `prune` among what `setup` may store, or restart"). A function value the
  runtime cannot enumerate is treated as held. Nothing of this runs while the program runs.
* `[HR-42]` *(changed in 0.9.20)* **Publication.** Entering Ember code increments the thread's depth
  counter and then reads the runtime's commit mark, both sequentially consistent (one full fence
  between them where the hardware needs it); a thread that reads the mark set decrements its
  counter, waits until the mark is cleared, and enters again. Leaving Ember code is a release
  decrement. Commit sets the commit mark and then reads every other registered thread's counter,
  with the same ordering, so that of a thread entering and a commit starting at once at least one
  sees the other; a counter that is not zero clears the mark and the poll returns
  `EMBER_RELOAD_UNSAFE_POINT`. With every counter zero, commit writes the instances, the type
  information and the call table, publishes them with one release store to the reload generation
  counter, which a thread entering next acquires, and clears the mark.
* `[HR-42a]` *(changed in 0.9.20)* The counter is not a lock: entering Ember code is one increment,
  one fence and one load whatever the number of threads; only a thread that meets a commit waits,
  for the length of the commit, and the reload side pays the scan over threads.
* `[HR-4]` Old images are never unloaded, so a stale return address or retired thunk target stays
  mapped.
* `[HR-4a]` Statics live in a runtime-owned table, never in image memory.
* `[HR-8]` *(changed in 0.9.15)* Each image carries a reload manifest section listing every
  reloadable function by mangled name, every type schema, every static, the facts of `[HR-44]` and
  the protocol version; the runtime reads it rather than the platform's export table.

## B.2 Calls and function addresses

* `[HR-5]` A call to a reloadable function goes through its thunk.
* `[HR-6]` Each reloadable function gets one thunk, at a fixed address for the life of the process,
  that loads the current target from the runtime's call table and jumps to it; a reload rewrites the
  table, never the thunk. A reloadable function's address, wherever it can be observed, is its
  thunk's.
* `[HR-6a]` *(changed in 0.9.15)* Hence function values (their code word), method tables, drop glue
  and `extern "C" fn` values stay ordinary code pointers; nothing else in the language changes.
* `[HR-7]` *(changed in 0.9.20)* Call-table slots are identified by mangled name (`[MNG-1]`), never
  by index. A function removed by an edit keeps its thunk, now pointing at a stub that panics naming
  it; a reload is refused, naming the function, when its thunk is still named by a function value
  plan enumerates (`[HR-44a]`), by a method table or drop glue of a type with a live instance, by a
  callback C keeps (`[HR-20]`) or by an `@export` (`[HR-21]`); one C holds that the runtime cannot
  see runs the stub, which panics naming the function.
* `[HR-9]` A call through a thunk costs one load and one indirect jump; the measured overhead on the
  performance suite with reload enabled is reported with its configuration and must not exceed 3 %.
* `[HR-9a]` *(changed in 0.9.10)* A reloadable function is never inlined, devirtualised or placed in
  the inline header of `[CG-C-3]`, since an inlined body cannot be swapped; this holds over
  `@inline` (`[CG-C-3a]`). A `@noreload` function may be inlined (`[HR-10]`).
* `[HR-28]` `ember inspect --safety` reports the thunk indirection of each function beside its
  safety checks.
* `[HR-10]` *(changed in 0.9.15)* `@noreload fn` is called directly and may be inlined; changing its
  body requires a restart (the reload is refused, naming it). Hot loops and `@no_alias_checks`,
  `@simd` and `@parallel` bodies should be `@noreload`.
* `[HR-10a]` A `@noreload` function may call reloadable functions, through their thunks.
* `[HR-21]` A foreign table of exported function pointers stays valid across every reload, because
  each exported function's address is its thunk. Adding, removing or changing the signature of an
  `@export` function changes the module protocol and is refused: that is a rebuild of the host.

## B.3 Types, schemas and migration

* `[HR-11]` *(changed in 0.9.15)* *(changed in 0.9.20)* Every type in a reloadable package has a
  **schema**: its kind, base, layout attributes, and the ordered list of fields (name, type schema
  hash, offset, size) or variants (name, discriminant, payload). A field's type schema includes the
  words its type is written with (`owns`, `shares`, `shares mut`, `thread_shared`, `weakref`,
  `sharedref mut`, `ref`, `ref mut`, `using A`) and the lock it is `guarded by` (none, or the lock
  field's name), so a change of one is a change of the field.
* `[HR-11a]` Enum values are migrated by variant name, never by discriminant.
* `[HR-12]` *(changed in 0.9.15)* *(changed in 0.9.20)* In a reloadable build, class instances are
  registered in a live-instance list, which adds 16 bytes to the object header of `[OBJ-1]` (24
  bytes instead of 8, and 28 instead of 12 for an object `weakref` links may reach); non-reloadable
  builds keep `[OBJ-1]`'s layout. A `thread_shared` value's 32-byte header (`[OBJ-1]`) is 48 bytes
  in a reloadable build. Registering costs two stores when an object is made and two when it ends,
  and for an object that is not `thread_shared` touches nothing another thread touches (`[RT-11]`);
  a `thread_shared` object's registration is ordered by the runtime, one lock or one atomic
  exchange, on whichever thread makes or ends it. Plan and prepare walk the list only while every
  other registered thread is at depth zero (`[HR-3]`). A value made shared in a block of its own
  (`[RC-1]`) is registered as a class instance is, with its type's schema, and migrated once,
  whatever the number of owners and links that reach it.
* `[HR-12a]` The header size is a whole-process property: every package in a process agrees on
  whether reload is enabled, enforced at link time (`E9035`). A reload image's new kinds take type
  indices above the process's when the image is loaded (`[TYP-44a]`).
* `[HR-13]` Value-typed data (structs in arrays, `SoA` columns, ECS storage, arena contents) is
  migrated through its container, which registers itself with its element type's schema.
* `[HR-13a]` Standard-library containers of a type from a reloadable package register themselves;
  the instantiating package emits the registration, so `std` needs no reload support.
* `[HR-13b]` *(changed in 0.9.20)* Value data reachable only through raw pointers or foreign memory
  cannot be migrated. Which types such data may have is judged when the program compiles, from
  types, as `[HR-47]` judges: a type `T` is **unenumerable** when the program, its dependencies
  included, makes a raw pointer `*T` or `*mut T` from a place (`ref_to_ptr`, §IX.4) rather than from
  an allocation the runtime registers, a `MaybeUninit[T]` outside a registered container, an
  `into_raw()` of a `T` or of what holds one (`[HR-20]` names those it can), or has a foreign
  parameter whose contract keeps a `T` or what holds one; the image's manifest (`[HR-8]`) lists the
  unenumerable types with the line that makes each so. Plan refuses a schema change of an
  unenumerable type, naming the type and that line, whether or not such a value lives, since the
  runtime cannot count them; the same list decides `[HR-45]`'s frames and `[HR-48]`'s states that
  are treated as live. `ember build --reload --explain` prints the list.
* `[HR-14]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Migration is by
  name**, and this table is exhaustive:

  | Change | Behaviour |
  |---|---|
  | field added with a default | initialised to that default |
  | field added, its type is `Default` | initialised to `Default.default()` |
  | field added, neither | refused, naming the field and suggesting a default |
  | field removed | old value ended in reclaim |
  | a kept view points into a field that is removed, converted or rebuilt | refused (`[HR-15c]`) |
  | a lambda's captures changed while a state of it lives | the field rows, applied to the captures (`[HR-48]`) |
  | field renamed with `@renamed_from("old")` | value carried over |
  | field renamed without it | a removal and an addition |
  | field type widened losslessly (`[TYP-5]`) | converted |
  | field type changed to a range type | refused unless every value passes `T.checked` |
  | field type changed otherwise, its words included (`[HR-11]`) | refused unless the type declares `migrate_from` |
  | field of a migrated type | migrated depth-first |
  | fields reordered, `let` changed | carried over by name |
  | methods added, removed or changed | no instance change, except as `[HR-44]` says for a `@cached` method |
  | base class changed | refused |
  | the promise an `open` or `abstract` class's header makes (`implements Send`, `implements ThreadShared`, an interface that needs them, `[THR-8]`) added or removed | refused (`E9030`) unless the image rebuilds every package that imports the class, whose derived classes and hand-overs are then checked again when they compile (`[HR-44]`) |
  | class removed while instances live | refused, naming the class and the count |
  | enum variant added, reordered or renamed with `@renamed_from` | discriminants rewritten |
  | enum variant removed while a value holds it | refused, naming the variant and the count |
  | enum variant payload changed | the field rows, applied to the payload |
  | layout or fields changed on a type used across FFI by value | refused |
  | a `gen fn`'s frame layout or states changed while a frame of it lives | refused (`[HR-45]`) |
  | a value made `using` an instance moved, or a memory manager's type changed while values made in it live | refused (`[HR-46]`) |
  | static's initialiser changed | refused unless the static is `@reinit_on_reload` (`[HR-17a]`, `[HR-47]`) |

* `[HR-15]` *(changed in 0.9.10)* *(changed in 0.9.15)* Migration keeps an instance's address where
  the new size fits; otherwise commit rewrites every owner, link and view of it that the runtime can
  enumerate: the one-owner fields and elements that hold it (`T`, `owns T`), its `shares`, `shares
  mut` and `thread_shared` owners, its `weakref` links, and the kept views into it (`ref T`, `ref
  mut T`, `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`), in live instances and registered
  containers, in statics, and inside tuples, `owns` blocks and function values' state held by any of
  these; and the instance address a value made `using` an instance keeps (`[HR-46]`), where that
  instance moves.
* `[HR-15a]` *(changed in 0.9.15)* An owner, link or view the runtime cannot enumerate — in foreign
  memory, behind a raw pointer, in an object handed to C (`[HR-20]`) — cannot be rewritten, and the
  reload is refused when what it names must move.
* `[HR-15c]` *(new in 0.9.20)* **A view into a part.** A kept view (`[TYP-15]`) that points into a
  part of a value, a field, an element, a range of a `String` or an array, is rewritten to the
  part's new place when the part is carried over by name (reordered, its holder resized or moved):
  the runtime knows what the view points into from the `from` of the field that holds it
  (`[TYP-15]`), the block of that `let` field, in which the view's old offset names the field, or
  the element and its field, of the old schema, and maps the old offset to the new through the
  schema; a view `from static` points into no instance and is never rewritten. Where the part is
  removed, converted, rebuilt by `migrate_from`, or of a type whose own layout changes, the view has
  no place to be rewritten to, and plan refuses the reload, naming the field, the type that holds
  the view and the count, judged from the types whose fields may hold such a view, as `[HR-47]`
  judges; a view the runtime cannot enumerate (`[HR-15a]`) that may point into such a part refuses
  it the same way.
* `[HR-15b]` Every refusal is decided in plan, before prepare runs.
* `[HR-16]` *(changed in 0.9.15)* *(changed in 0.9.20)* `static fn migrate_from(old: OldSelf) ->
  Result[Self, ReloadError]` replaces field-wise migration for its type. `OldSelf` is the previous
  schema as a generated struct `Old<Name>`, visible only in that function; `old` is a read-only view
  of the old instance. `Old<Name>`'s fields have the types of the previous schema: a field whose
  type is itself migrated in this reload is of that type's `Old` form (`log: shares OldLog`; a
  container field `parts: Array[Part]` is `Array[OldPart]`, read-only, its elements `OldPart`
  values), through links, `owns` blocks, containers and tuples alike, so that nothing read through
  `old` is read with a new layout; a field whose type is unchanged keeps its type. An `Old` type has
  its fields and no methods. A `shares`, `shares mut`, `thread_shared` or `weakref` link read from
  `old` may be stored into the new instance's field of the new type (`log=shares old.log`): it names
  the old instance until commit, and commit's map (`[HR-2a]`) repoints it to the migrated one; so
  `migrate_from` never reads the new instance of another type, and never stores a view of an `Old`
  value (`E3063`, since the old block is freed in reclaim).
* `[HR-34]` Prepare cannot panic: every operation it performs is either free of panics or reported
  as a `ReloadError`.
* `[HR-35]` *(changed in 0.9.9)* *(changed in 0.9.15)* *(changed in 0.9.20)* `migrate_from` cannot abort the process. It
  may not contain an explicit panic (`panic`, `unwrap`, `expect`, `assert`: `E2225`, whose help
  names the fallible form); and every run-time check inside it — overflow, bounds, a cast, a count
  at its maximum, a stale handle, a `runtime` name's test (`[LT-48]`) — returns
  `Err(ReloadError.Migration(type_name, instance, message))` instead of stopping the program. The
  rule covers what `migrate_from` runs, not only what it writes: every function it calls, directly
  or through what it calls, and every `drop` it may run, is compiled for that use as an instance of
  itself under this rule (`[MNG-6]`: `[reload]` among its arguments), with the same `E2225` for an
  explicit panic and the same `Err` returns for a check; a call through a function value, an
  interface, `dyn` or `virtual` method, in `migrate_from` or in what it calls, reaches a body
  compiled to stop the program, and is `E2225` ("`sum_all` calls `parts.reduce(f)` on line 9 through
  a function value, which may stop the program during a reload; call the function by its name, or
  write `@allow_reload_terminate` on `migrate_from`"), which `[HR-43]` admits as it admits a
  `noexcept` call; a call to an `extern` function follows `[HR-39]`, whose refusal is `E2225`
  ("`finish` is asserted `noexcept` and may end the process; call it outside the migration, or write
  `@allow_reload_terminate`"). A field default or `Default.default()` that prepare runs for an added
  field (`[HR-14]`) is compiled the same way, and an explicit panic in it makes the row `field
  added, neither` (refused, naming the field and the panic's line).
* `[HR-39]` A foreign call inside `migrate_from` returns its failure as `ReloadError.Foreign`; a
  call to a C++ function asserted `noexcept`, which could terminate the process, is rejected at
  compile time.
* `[HR-43]` `@allow_reload_terminate` on a `migrate_from` admits such calls and states that this
  migration may end the process rather than refuse; `ember tcb` lists every use.
* `[HR-36]` *(changed in 0.9.20)* Allocation during prepare comes from the transaction arena;
  exhausting it fails the reload with `ReloadError.Allocation`, so construction inside
  `migrate_from` needs no special spelling. A value made `using` an instance of a manager that may
  have several (`[ALC-1]`) takes its memory from that instance, as its type says, never from the
  transaction arena, and keeps that instance's address; a manager's instance asked for a block and
  given it back is not an instance changed in `[HR-2]`'s sense.
* `[HR-37]` *(changed in 0.9.20)* The transaction arena holds everything prepare allocates and is
  released whole if the reload fails. A reload discarded after prepare first ends every instance
  prepare built, field by field and without the type's own `drop` (as reclaim ends an old version,
  `[HR-2a]`), before the arena is released: a count raised through `old` (`shares old.log`) is
  lowered, and a block taken from a manager's instance (`[HR-36]`) goes back to it, so `[HR-2]`'s
  "no instance changed" holds; the `drop`s this may run are compiled under `[HR-35]`.
* `[HR-38]` *(changed in 0.9.10)* `std.hot.ReloadError` is `SchemaRefused(type_name: String, reason:
  String)`, `Migration(type_name: String, instance: u64, message: String)`, `Allocation`,
  `Foreign(error: String)` or `Timeout`; `instance` counts the type's live instances from 0.
* `[HR-45]` *(new in 0.9.15)* **A suspended generator keeps its frame.** A `gen fn`'s frame
  (`[CORO-3]`) has a schema: its layout and its states, one per suspension point, each with what
  lives across it. The runtime finds the live frames of each reloadable `gen fn` among the values it
  can enumerate (`[HR-15]`). A body change that leaves the frame's layout and states as they were
  applies: a live frame resumes in the new code at the state it was suspended in, and a paused
  generator walking a list counts, for `[HR-44]`, as a frame that still calls what its body calls. A
  body change that changes them is refused in plan while a frame lives, naming the `gen fn` and the
  count; with no live frame it applies as any body change does. A frame the runtime cannot enumerate
  (in foreign memory, behind a raw pointer) is treated as live.
* `[HR-48]` *(new in 0.9.20)* **A lambda's state has a schema.** The state of a lambda (`[CLO-10]`)
  is a value of a generated struct whose fields are its captures, in order, with their words
  (`[HR-11]`), kept in the state word or in the state block; the lambda's name (`[MNG-1]`: its
  function, its ordinal and the hash of its text) names the type. It is migrated as any value is:
  through the places that hold it (`[HR-15]`), by `[HR-14]`'s rows, a capture of a migrated type
  depth-first, a capture removed ended in reclaim, a capture added refused naming the field (a
  capture has no default), a word changed refused, since a lambda declares no `migrate_from`; and a
  body change that leaves the captures as they were applies through the thunk. A state the runtime
  cannot enumerate (kept by C, `[HR-20]`; behind a raw pointer, `[HR-13b]`) refuses a change of its
  schema.
* `[HR-46]` *(new in 0.9.15)* **Values made in a memory manager's instance.** A value made `using`
  an instance of a memory manager that may have several (`[ALC-1]`) came from that instance and goes
  back to it, through the instance address it keeps. Its migration keeps it in that instance's
  block: it is migrated in place where its new layout has the same size and alignment, so that its
  end hands the instance back the layout it was given; one that would have to move or change size is
  refused in plan, naming its type, the manager's type and the count. Where the instance itself
  moves (`[HR-15]`), commit rewrites the instance address every such value keeps; a value whose
  instance address the runtime cannot enumerate is refused as `[HR-15a]` says. A type that
  implements `Allocator` is changed only through its own methods while values made in one of its
  instances live, so a change of its schema while such values live is refused in plan, naming the
  type and the count; with none live it migrates as any type does.
* `[HR-17]` A static whose type and initialiser are unchanged keeps its value; one whose type
  changed migrates by `[HR-14]`; a new static is initialised normally.
* `[HR-17a]` *(changed in 0.9.9)* *(changed in 0.9.15)* Editing a static's initialiser changes its
  schema: the reload is refused, naming the static, unless the static is `@reinit_on_reload`, which
  re-runs the initialiser and ends the old value. Keeping the old value silently is never an
  outcome.
* `[HR-47]` *(new in 0.9.15)* `@reinit_on_reload` ends the static's old value when the reload is
  applied, so nothing may still point into it: plan refuses it (naming both statics, or the static
  and the type) where a view, a `sharedref mut` link or the instance address of a value made `using`
  it, held by another static, a live instance or a registered container, may point into the static's
  value or what it owns, judged from the types that can hold one. `shares` owners of what it held
  keep their objects alive, and `weakref` links to them find them ended, as anywhere else.
* What the compiler works out from bodies (where each view is used, what each function changes,
  reads and ends) plays no part in schemas or migration; a change to it is judged by `[HR-44]`.

## B.4 Refusal, scope and cost

* `[HR-18]` A refusal returns a report naming each type and change that caused it; it is never a crash.
* `[HR-18a]` A refusal is never partial: nothing of the new image is live.
* `[HR-18b]` `ember build --reload --explain` predicts the outcome against the running process without
  applying anything.
* `[HR-41]` *(changed in 0.9.10)* *(changed in 0.9.15)* **Failure matrix.**

  | What happened | Detected in | Outcome |
  |---|---|---|
  | source does not compile, or fails checking | before the poll | old image keeps running |
  | reload protocol mismatch | load | refused, `E9037` |
  | a C or C++ header changed | plan | refused (`[HR-23]`) |
  | schema change with no defined outcome | plan | refused (`[HR-14]`) |
  | `@noreload` body changed | plan | refused (`[HR-10]`) |
  | a worked-out fact changed that code not rebuilt relies on | plan | refused (`[HR-44]`) |
  | a live generator frame's layout or states changed | plan | refused (`[HR-45]`) |
  | a live function value's captures or a captured type changed | plan | refused (`[HR-48]`) |
  | a stored function value or kept callback the new image does not count | plan | refused (`[HR-44a]`) |
  | an object in use by an in-flight GPU frame must move | plan, after at most `frames_in_flight + 1` polls | refused, `Timeout` (`[HR-24]`) |
  | a value made `using` an instance must move, or its manager's type changed while it lives | plan | refused (`[HR-46]`) |
  | `@reinit_on_reload` on a static something else may point into | plan | refused (`[HR-47]`) |
  | an owner, link or view that cannot be rewritten names what must move | plan | refused (`[HR-15a]`) |
  | allocation exhausted, `migrate_from` fails, or a foreign call fails | prepare | discarded; old image keeps running |
  | panic in `migrate_from`, or a failure in commit | cannot occur | forbidden by `[HR-35]` and `[HR-2]` |
  | panic in a `drop` during reclaim | reclaim | the reload has succeeded; the panic aborts as any panic does |

* `[HR-19]` *(changed in 0.9.15)* `reload = "bodies"` admits only function-body changes and changes
  that need no instance migration; it needs no type schemas and no larger header, and is the first
  tier an implementation provides. It still records the facts of `[HR-44]` and the frame schemas of
  `[HR-45]`, and refuses as they say.
* `[HR-25]` `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); `@reloadable` and
  `@noreload` apply to modules and items, item level winning.
* `[HR-20]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* An object C holds —
  handed to a foreign parameter whose contract keeps it, or given with `into_raw()` — and the state
  of a callback that C keeps registered are never moved: C holds their addresses, which the runtime
  cannot rewrite. If the new size of such an object or state does not fit, the reload is refused,
  naming it.
* `[HR-22]` Foreign objects owned by Ember are carried across unchanged.
* `[HR-23]` *(changed in 0.9.10)* A changed C or C++ header refuses the reload and names it (the
  host must be rebuilt): any change to the header or to a file it includes, a comment's too, since
  the host compiled the inline bodies it declares. A changed overlay does not, because overlay edits
  change only Ember code; where the header is unchanged the previous thunks are reused.
* `[HR-24]` *(changed in 0.9.10)* *(changed in 0.9.20)* An object in use by an in-flight GPU frame
  is migrated in place when it fits; otherwise the reload waits at most `frames_in_flight + 1` polls
  and is then refused, naming the resource. A `Device` registers with the runtime, as a container
  does (`[HR-13]`), the objects in flight and the frame each waits for, data the device updates when
  it observes a fence (`[GPU-4]`); plan reads that record and decides it as every refusal is decided
  (`[HR-15b]`): an object that must move while in flight makes the poll return
  `EMBER_RELOAD_UNSAFE_POINT`, and after `frames_in_flight + 1` such polls the reload is refused
  with `ReloadError.Timeout` (`[HR-38]`), naming the resource (`[HR-41]`).
* `[HR-27]` A `release` build is bit-identical whether or not the source uses any reload attribute.
* `[HR-29]` With reload enabled, one shared runtime serves every image in the process (`[build]
  runtime = "shared"`); a reloadable package that links its own runtime copy is `E9036`.
* `[HR-30]` **Host contract:** `ember_reload_init(&config)` once; `ember_reload_poll()` at a point
  where no Ember frame is live (between frames, for a game); `ember_reload_stats()` for timing;
  `ember_reload_shutdown()`.
* `[HR-31]` Polling never blocks; compilation runs in the background.
* `[HR-32]` `ember_reload_stats()` reports the last reload split into compile, load, plan, prepare,
  commit and reclaim, with the instances migrated and any refusal.
* `[HR-33]` `ember run --hot` is the toolchain's own host: it builds under `debug`, runs the
  program, watches the sources and polls at the point the program declares with
  `std.hot.checkpoint()`; a program that declares none is told so after ten seconds rather than
  appearing to hang.

