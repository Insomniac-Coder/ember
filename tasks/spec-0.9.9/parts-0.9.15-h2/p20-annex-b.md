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

* `[HR-2]` *(changed in 0.9.15)* **Reload is transactional.** A failure in plan or prepare leaves the process running the old
  image, with no instance changed, handed over or ended, and a report naming the cause. Commit performs
  only pointer stores into memory reserved in prepare: no allocation, check, callback or other fallible
  operation. A failure in reclaim is an ordinary panic after the reload has succeeded.
* `[HR-2a]` *(changed in 0.9.15)* Plan reads schemas, the facts of `[HR-44]` and the registered set and runs no user code.
  Prepare allocates every new instance in the transaction arena (`[HR-37]`), runs every `migrate_from`
  against a read-only view of the old instance, and computes the old-to-new address map. Commit
  publishes the instances, patches type information, applies the map and repoints the thunks. Reclaim
  frees each migrated instance's old version field by field without the type's own `drop`, since the
  instance lives on in its new form, and ends removed fields, running their `drop`.
* `[HR-3]` *(changed in 0.9.10)* A reload is applied only inside `ember_reload_poll()`, and only when no
  registered thread is executing Ember code (each thread's Ember-depth counter is zero); otherwise the
  poll returns `EMBER_RELOAD_UNSAFE_POINT` and does nothing. The exception is `std.hot.checkpoint()`
  (`[HR-33]`), a poll the program makes itself, where its own thread's frames below the checkpoint stay
  live (`[HR-3b]`).
* `[HR-3a]` Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`),
  `thread.spawn` threads, and job workers. `debug` builds assert on entry to Ember code from an
  unregistered thread.
* `[HR-3b]` *(new in 0.9.10)* *(changed in 0.9.15)* At a checkpoint the frames below it run their old code on after the swap.
  Each function of a reloadable image keeps a record of which of its calls it is in, and the image lists,
  for each call, the types whose layout the rest of the function depends on once the call returns: those
  its old code reads or writes fields of, builds, holds, passes or gets back by value, indexes or
  allocates, and those the functions it calls directly depend on (they run old code too; a call through
  a function value may reach any); and, beside the types, the functions the rest of the function may
  still call, whose facts `[HR-44]` compares. A layout change applies at the checkpoint when no frame
  below it depends on the changed type: the instances migrate as at a poll (`[HR-14]`), and one a
  frame's view points into keeps its address, which `[HR-15]` allows when its new size fits; one that
  must move is refused, since a frame's views are not counted (`[HR-15a]`). A change a frame depends
  on is refused, naming the function and the line of its call. A static's change still waits for a poll
  from the host.
* `[HR-44]` *(new in 0.9.15)* **The facts the compiler worked out are part of what a reload compares.** Every rule that
  decides when compiling whether a line is safe — a loop and a call that may change what it walks, a
  read-only view and a call that may change what it reads, a view argument and a callee that may end or
  move what it points into, a view of an object reached through a link and a call that may end the
  object, a lock this thread holds and a call that may take it, a parent link read with no header test,
  what a thread or a kept callback changes — rests on what each function may change, read and end,
  worked out from its body and from every function the program can put in a function value, behind an
  interface, `dyn` or virtual method, or in C's keeping (`[EFF-4]`). The image records these facts for
  every reloadable function, as a package's interface file does. Plan refuses a reload (`E9030`) whose
  new image changes such a fact of a function that code the reload does not rebuild was checked
  against: a frame below a checkpoint that may still call it (`[HR-3b]`), a suspended generator
  (`[HR-45]`), and another package compiled against this package's facts. The report names the
  function, the fact and the caller that relies on it ("`note` now adds to `LOG`, and `run`, still
  running below the checkpoint on line 10, walks `LOG` across its call of `note` on line 11; restart,
  or let `run` return before the reload"). Code the reload rebuilds is checked again when it compiles,
  so a change that only it relies on is no refusal. A change to a `@cached` method's body clears the
  result each instance of its type keeps: the kept result is migrated as a removed field and an added
  one with no value (`[HR-14]`), and is computed again on the next call. Nothing of this runs while the
  program runs.
* `[HR-42]` **Publication.** Entering Ember code increments the thread's depth counter with acquire
  ordering and leaving decrements it with release. Commit reads every counter with acquire, writes the
  new thunk targets and type information, and publishes them with one release store to a reload
  generation counter, which a thread entering Ember code next acquires.
* `[HR-42a]` The counter is not a lock: entering Ember code is one relaxed increment and one acquire
  fence whatever the number of threads; the reload side pays the scan over threads.
* `[HR-4]` Old images are never unloaded, so a stale return address or retired thunk target stays
  mapped.
* `[HR-4a]` Statics live in a runtime-owned table, never in image memory.
* `[HR-8]` *(changed in 0.9.15)* Each image carries a reload manifest section listing every reloadable function by mangled
  name, every type schema, every static, the facts of `[HR-44]` and the protocol version; the runtime
  reads it rather than the platform's export table.

## B.2 Calls and function addresses

* `[HR-5]` A call to a reloadable function goes through its thunk.
* `[HR-6]` Each reloadable function gets one thunk, at a fixed address for the life of the process,
  that loads the current target from the runtime's call table and jumps to it; a reload rewrites the
  table, never the thunk. A reloadable function's address, wherever it can be observed, is its thunk's.
* `[HR-6a]` *(changed in 0.9.15)* Hence function values (their code word), method tables, drop glue and `extern "C" fn`
  values stay ordinary code pointers; nothing else in the language changes.
* `[HR-7]` Call-table slots are identified by mangled name (`[MNG-1]`), never by index. A function
  removed by an edit keeps its thunk, now pointing at a stub that panics naming it; a reload that would
  leave such a thunk reachable is refused.
* `[HR-9]` A call through a thunk costs one load and one indirect jump; the measured overhead on the
  performance suite with reload enabled is reported with its configuration and must not exceed 3 %.
* `[HR-9a]` *(changed in 0.9.10)* A reloadable function is never inlined, devirtualised or placed in
  the inline header of `[CG-C-3]`, since an inlined body cannot be swapped; this holds over `@inline`
  (`[CG-C-3a]`). A `@noreload` function may be inlined (`[HR-10]`).
* `[HR-28]` `ember inspect --safety` reports the thunk indirection of each function beside its safety
  checks.
* `[HR-10]` *(changed in 0.9.15)* `@noreload fn` is called directly and may be inlined; changing its body requires a restart
  (the reload is refused, naming it). Hot loops and `@no_alias_checks`, `@simd` and `@parallel` bodies
  should be `@noreload`.
* `[HR-10a]` A `@noreload` function may call reloadable functions, through their thunks.
* `[HR-21]` A foreign table of exported function pointers stays valid across every reload, because each
  exported function's address is its thunk. Adding, removing or changing the signature of an
  `@export` function changes the module protocol and is refused: that is a rebuild of the host.

## B.3 Types, schemas and migration

* `[HR-11]` *(changed in 0.9.15)* Every type in a reloadable package has a **schema**: its kind, base, layout attributes, and
  the ordered list of fields (name, type schema hash, offset, size) or variants (name, discriminant,
  payload). A field's type schema includes the words its type is written with (`owns`, `shares`,
  `shares mut`, `thread_shared`, `weakref`, `sharedref mut`, `ref`, `ref mut`, `using A`), so a change
  of one is a change of the field's type.
* `[HR-11a]` Enum values are migrated by variant name, never by discriminant.
* `[HR-12]` *(changed in 0.9.15)* In a reloadable build, class instances are registered in a live-instance list, which adds
  16 bytes to the object header of `[OBJ-1]` (24 bytes instead of 8, and 28 instead of 12 for an object
  `weakref` links may reach); non-reloadable builds keep `[OBJ-1]`'s layout.
* `[HR-12a]` The header size is a whole-process property: every package in a process agrees on whether
  reload is enabled, enforced at link time (`E9035`).
* `[HR-13]` Value-typed data (structs in arrays, `SoA` columns, ECS storage, arena contents) is migrated
  through its container, which registers itself with its element type's schema.
* `[HR-13a]` Standard-library containers of a type from a reloadable package register themselves; the
  instantiating package emits the registration, so `std` needs no reload support.
* `[HR-13b]` Value data reachable only through raw pointers or foreign memory cannot be migrated; if its
  schema changed, the reload is refused.
* `[HR-14]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Migration is by name**, and this table is exhaustive:

  | Change | Behaviour |
  |---|---|
  | field added with a default | initialised to that default |
  | field added, its type is `Default` | initialised to `Default.default()` |
  | field added, neither | refused, naming the field and suggesting a default |
  | field removed | old value ended in reclaim |
  | field renamed with `@renamed_from("old")` | value carried over |
  | field renamed without it | a removal and an addition |
  | field type widened losslessly (`[TYP-5]`) | converted |
  | field type changed to a range type | refused unless every value passes `T.checked` |
  | field type changed otherwise, its words included (`[HR-11]`) | refused unless the type declares `migrate_from` |
  | field of a migrated type | migrated depth-first |
  | fields reordered, `let` changed | carried over by name |
  | methods added, removed or changed | no instance change, except as `[HR-44]` says for a `@cached` method |
  | base class changed | refused |
  | class removed while instances live | refused, naming the class and the count |
  | enum variant added, reordered or renamed with `@renamed_from` | discriminants rewritten |
  | enum variant removed while a value holds it | refused, naming the variant and the count |
  | enum variant payload changed | the field rows, applied to the payload |
  | layout or fields changed on a type used across FFI by value | refused |
  | a `gen fn`'s frame layout or states changed while a frame of it lives | refused (`[HR-45]`) |
  | a value made `using` an instance moved, or a memory manager's type changed while values made in it live | refused (`[HR-46]`) |
  | static's initialiser changed | refused unless the static is `@reinit_on_reload` (`[HR-17a]`, `[HR-47]`) |

* `[HR-15]` *(changed in 0.9.10)* *(changed in 0.9.15)* Migration keeps an instance's address where the new size fits;
  otherwise commit rewrites every owner, link and view of it that the runtime can enumerate: the
  one-owner fields and elements that hold it (`T`, `owns T`), its `shares`, `shares mut` and
  `thread_shared` owners, its `weakref` links, and the kept views into it (`ref T`, `ref mut T`,
  `sharedref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`), in live instances and registered containers,
  in statics, and inside tuples, `owns` blocks and function values' state held by any of these; and the
  instance address a value made `using` an instance keeps (`[HR-46]`), where that instance moves.
* `[HR-15a]` *(changed in 0.9.15)* An owner, link or view the runtime cannot enumerate — in foreign memory, behind a raw
  pointer, in an object handed to C (`[HR-20]`) — cannot be rewritten, and the reload is refused when
  what it names must move.
* `[HR-15b]` Every refusal is decided in plan, before prepare runs.
* `[HR-16]` *(changed in 0.9.15)* `static fn migrate_from(old: OldSelf) -> Result[Self, ReloadError]` replaces field-wise
  migration for its type. `OldSelf` is the previous schema as a generated struct `Old<Name>`, visible
  only in that function; `old` is a read-only view of the old instance.
* `[HR-34]` Prepare cannot panic: every operation it performs is either free of panics or reported as
  a `ReloadError`.
* `[HR-35]` *(changed in 0.9.9)* *(changed in 0.9.15)* `migrate_from` cannot abort the process. It may not contain an
  explicit panic (`panic`, `unwrap`, `expect`, `assert`: `E2225`, whose help names the fallible form);
  and every run-time check inside it — overflow, bounds, a cast, a count at its maximum, a stale
  handle — returns `Err(ReloadError.Migration(type_name, instance, message))` instead of stopping the
  program.
* `[HR-39]` A foreign call inside `migrate_from` returns its failure as `ReloadError.Foreign`; a call to
  a C++ function asserted `noexcept`, which could terminate the process, is rejected at compile time.
* `[HR-43]` `@allow_reload_terminate` on a `migrate_from` admits such calls and states that this
  migration may end the process rather than refuse; `ember tcb` lists every use.
* `[HR-36]` Allocation during prepare comes from the transaction arena; exhausting it fails the reload
  with `ReloadError.Allocation`, so construction inside `migrate_from` needs no special spelling.
* `[HR-37]` The transaction arena holds everything prepare allocates and is released whole if the
  reload fails.
* `[HR-38]` *(changed in 0.9.10)* `std.hot.ReloadError` is `SchemaRefused(type_name: String, reason:
  String)`, `Migration(type_name: String, instance: u64, message: String)`, `Allocation`,
  `Foreign(error: String)` or `Timeout`; `instance` counts the type's live instances from 0.
* `[HR-45]` *(new in 0.9.15)* **A suspended generator keeps its frame.** A `gen fn`'s frame (`[CORO-3]`) has a schema:
  its layout and its states, one per suspension point, each with what lives across it. The runtime
  finds the live frames of each reloadable `gen fn` among the values it can enumerate (`[HR-15]`). A
  body change that leaves the frame's layout and states as they were applies: a live frame resumes in
  the new code at the state it was suspended in, and a paused generator walking a list counts, for
  `[HR-44]`, as a frame that still calls what its body calls. A body change that changes them is
  refused in plan while a frame lives, naming the `gen fn` and the count; with no live frame it
  applies as any body change does. A frame the runtime cannot enumerate (in foreign memory, behind a
  raw pointer) is treated as live.
* `[HR-46]` *(new in 0.9.15)* **Values made in a memory manager's instance.** A value made `using` an instance of a
  memory manager that may have several (`[ALC-1]`) came from that instance and goes back to it, through
  the instance address it keeps. Its migration keeps it in that instance's block: it is migrated in
  place where its new layout has the same size and alignment, so that its end hands the instance back
  the layout it was given; one that would have to move or change size is refused in plan, naming its
  type, the manager's type and the count. Where the instance itself moves (`[HR-15]`), commit rewrites
  the instance address every such value keeps; a value whose instance address the runtime cannot
  enumerate is refused as `[HR-15a]` says. A type that implements `Allocator` is changed only through its
  own methods while values made in one of its instances live, so a change of its schema while such
  values live is refused in plan, naming the type and the count; with none live it migrates as any
  type does.
* `[HR-17]` A static whose type and initialiser are unchanged keeps its value; one whose type changed
  migrates by `[HR-14]`; a new static is initialised normally.
* `[HR-17a]` *(changed in 0.9.9)* *(changed in 0.9.15)* Editing a static's initialiser changes its schema: the reload is
  refused, naming the static, unless the static is `@reinit_on_reload`, which re-runs the initialiser
  and ends the old value. Keeping the old value silently is never an outcome.
* `[HR-47]` *(new in 0.9.15)* `@reinit_on_reload` ends the static's old value when the reload is applied, so nothing may
  still point into it: plan refuses it (naming both statics, or the static and the type) where a view, a `sharedref mut` link
  or the instance address of a value made `using` it, held by another static, a live instance or a
  registered container, may point into the static's value or what it owns, judged from the types that
  can hold one. `shares` owners of what it held keep their objects alive, and `weakref` links to them
  find them ended, as anywhere else.
* What the compiler works out from bodies (where each view is used, what each function changes, reads
  and ends) plays no part in schemas or migration; a change to it is judged by `[HR-44]`.

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
  | a value made `using` an instance must move, or its manager's type changed while it lives | plan | refused (`[HR-46]`) |
  | `@reinit_on_reload` on a static something else may point into | plan | refused (`[HR-47]`) |
  | an owner, link or view that cannot be rewritten names what must move | plan | refused (`[HR-15a]`) |
  | allocation exhausted, `migrate_from` fails, or a foreign call fails | prepare | discarded; old image keeps running |
  | panic in `migrate_from`, or a failure in commit | cannot occur | forbidden by `[HR-35]` and `[HR-2]` |
  | panic in a `drop` during reclaim | reclaim | the reload has succeeded; the panic aborts as any panic does |

* `[HR-19]` *(changed in 0.9.15)* `reload = "bodies"` admits only function-body changes and changes that need no instance
  migration; it needs no type schemas and no larger header, and is the first tier an implementation
  provides. It still records the facts of `[HR-44]` and the frame schemas of `[HR-45]`, and refuses as
  they say.
* `[HR-25]` `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); `@reloadable` and
  `@noreload` apply to modules and items, item level winning.
* `[HR-20]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* An object C holds — handed to a foreign parameter
  whose contract keeps it, or given with `into_raw()` — and the state of a callback that C keeps
  registered are never moved: C holds their addresses, which the runtime cannot rewrite. If the new
  size of such an object or state does not fit, the reload is refused, naming it.
* `[HR-22]` Foreign objects owned by Ember are carried across unchanged.
* `[HR-23]` *(changed in 0.9.10)* A changed C or C++ header refuses the reload and names it (the host
  must be rebuilt): any change to the header or to a file it includes, a comment's too, since the
  host compiled the inline bodies it declares. A changed overlay does not, because overlay edits
  change only Ember code; where the header is unchanged the previous thunks are reused.
* `[HR-24]` *(changed in 0.9.10)* An object in use by an in-flight GPU frame is migrated in place when
  it fits; otherwise the reload waits at most `frames_in_flight + 1` polls and is then refused, naming
  the resource. This is detected in prepare, so the reload is discarded and the old image keeps
  running (`[HR-41]`).
* `[HR-27]` A `release` build is bit-identical whether or not the source uses any reload attribute.
* `[HR-29]` With reload enabled, one shared runtime serves every image in the process
  (`[build] runtime = "shared"`); a reloadable package that links its own runtime copy is `E9036`.
* `[HR-30]` **Host contract:** `ember_reload_init(&config)` once; `ember_reload_poll()` at a point where
  no Ember frame is live (between frames, for a game); `ember_reload_stats()` for timing;
  `ember_reload_shutdown()`.
* `[HR-31]` Polling never blocks; compilation runs in the background.
* `[HR-32]` `ember_reload_stats()` reports the last reload split into compile, load, plan, prepare,
  commit and reclaim, with the instances migrated and any refusal.
* `[HR-33]` `ember run --hot` is the toolchain's own host: it builds under `debug`, runs the program,
  watches the sources and polls at the point the program declares with `std.hot.checkpoint()`; a
  program that declares none is told so after ten seconds rather than appearing to hang.
