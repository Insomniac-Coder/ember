---

# Annex B — Hot Reload (the Dynamic profile)

Hot reload turns a source edit into running behaviour in a live process, keeping existing objects,
statics and open resources. It is part of the Ember Dynamic conformance profile (`[CONF-5]`), is
available in `debug` and `release` builds, and is forbidden in `shipping`.

* `[HR-1]` An edit to one function body in a 50k-line reloadable package MUST be running in the process
  within one second on the reference machine (`[BUD-1]`), with all live object state preserved: budget
  B2 (`[BUD-2]`) for the build, plus load, plan, prepare and commit, measured together over a live set
  of 10,000 instances (`[BEN-8]`).
* `[BEN-8]` The end-to-end reload benchmark times three edits — a function body, a field added with a
  default, and a field whose type changes through `migrate_from` — from save to the new code running.

## B.1 Model

The unit of reload is the package. A package built with reload enabled is a shared library whose
functions are reached through permanent thunks and whose types carry schemas. A reload proceeds in
phases: **plan** (compare schemas; refuse here with a reason), **prepare** (build every migrated
instance; may fail, changes nothing), **commit** (publish; cannot fail), **reclaim** (drop what the
old image owned).

* `[HR-2]` **Reload is transactional.** A failure in plan or prepare leaves the process running the old
  image, with no instance mutated, moved from or dropped, and a report naming the cause. Commit performs
  only pointer stores into memory reserved in prepare: no allocation, check, callback or other fallible
  operation. A failure in reclaim is an ordinary panic after the reload has succeeded.
* `[HR-2a]` Plan reads schemas and the registered set and runs no user code. Prepare allocates every new
  instance in the transaction arena (`[HR-37]`), runs every `migrate_from` against a read-only view of
  the old instance, and computes the old-to-new address map. Commit publishes the instances, patches
  type information, applies the map and repoints the thunks. Reclaim drops the old instances and
  removed fields, running user `drop` code.
* `[HR-3]` A reload is applied only inside `ember_reload_poll()`, and only when no registered thread is
  executing Ember code (each thread's Ember-depth counter is zero); otherwise the poll returns
  `EMBER_RELOAD_UNSAFE_POINT` and does nothing.
* `[HR-3a]` Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`),
  `thread.spawn` threads, and job workers. `debug` builds assert on entry to Ember code from an
  unregistered thread.
* `[HR-42]` **Publication.** Entering Ember code increments the thread's depth counter with acquire
  ordering and leaving decrements it with release. Commit reads every counter with acquire, writes the
  new thunk targets and type information, and publishes them with one release store to a reload
  generation counter, which a thread entering Ember code next acquires.
* `[HR-42a]` The counter is not a lock: entering Ember code is one relaxed increment and one acquire
  fence whatever the number of threads; the reload side pays the scan over threads.
* `[HR-4]` Old images are never unloaded, so a stale return address or retired thunk target stays
  mapped.
* `[HR-4a]` Statics live in a runtime-owned table, never in image memory.
* `[HR-8]` Each image carries a reload manifest section listing every reloadable function by mangled
  name, every type schema, every static and the protocol version; the runtime reads it rather than the
  platform's export table.

## B.2 Calls and function addresses

* `[HR-5]` A call to a reloadable function goes through its thunk.
* `[HR-6]` Each reloadable function gets one thunk, at a fixed address for the life of the process,
  that loads the current target from the runtime's call table and jumps to it; a reload rewrites the
  table, never the thunk. A reloadable function's address, wherever it can be observed, is its thunk's.
* `[HR-6a]` Hence function values, closure code pointers, method tables, drop glue and `extern "C" fn`
  values stay ordinary code pointers; nothing else in the language changes.
* `[HR-7]` Call-table slots are identified by mangled name (`[MNG-1]`), never by index. A function
  removed by an edit keeps its thunk, now pointing at a stub that panics naming it; a reload that would
  leave such a thunk reachable is refused.
* `[HR-9]` A call through a thunk costs one load and one indirect jump; the measured overhead on the
  performance suite with reload enabled is reported with its configuration and must not exceed 3 %.
* `[HR-9a]` A reloadable function is never inlined, devirtualised or placed in the inline header of
  `[CG-C-3]`, since an inlined body cannot be swapped.
* `[HR-28]` `ember inspect --safety` reports the thunk indirection of each function beside its safety
  checks.
* `[HR-10]` `@noreload fn` is called directly and may be inlined; changing its body requires a restart
  (the reload is refused, naming it). Hot loops and `@static_safe`, `@simd` and `@parallel` bodies should
  be `@noreload`.
* `[HR-10a]` A `@noreload` function may call reloadable functions, through their thunks.
* `[HR-21]` A foreign table of exported function pointers stays valid across every reload, because each
  exported function's address is its thunk. Adding, removing or changing the signature of an
  `@export` function changes the module protocol and is refused: that is a rebuild of the host.

## B.3 Types, schemas and migration

* `[HR-11]` Every type in a reloadable package has a **schema**: its kind, base, layout attributes, and
  the ordered list of fields (name, type schema hash, offset, size) or variants (name, discriminant,
  payload).
* `[HR-11a]` Enum values are migrated by variant name, never by discriminant.
* `[HR-12]` In a reloadable build, class instances are registered in a live-instance list, which adds
  16 bytes to the object header (40 bytes instead of 24); non-reloadable builds keep `[OBJ-1]`'s
  layout.
* `[HR-12a]` The header size is a whole-process property: every package in a process agrees on whether
  reload is enabled, enforced at link time (`E9035`).
* `[HR-13]` Value-typed data (structs in arrays, `SoA` columns, ECS storage, arena contents) is migrated
  through its container, which registers itself with its element type's schema.
* `[HR-13a]` Standard-library containers of a type from a reloadable package register themselves; the
  instantiating package emits the registration, so `std` needs no reload support.
* `[HR-13b]` Value data reachable only through raw pointers or foreign memory cannot be migrated; if its
  schema changed, the reload is refused.
* `[HR-14]` *(changed in 0.9.9)* **Migration is by name**, and this table is exhaustive:

  | Change | Behaviour |
  |---|---|
  | field added with a default | initialised to that default |
  | field added, its type is `Default` | initialised to `Default.default()` |
  | field added, neither | refused, naming the field and suggesting a default |
  | field removed | old value dropped in reclaim |
  | field renamed with `@renamed_from("old")` | value carried over |
  | field renamed without it | a removal and an addition |
  | field type widened losslessly (`[TYP-5]`) | converted |
  | field type changed to a range type | refused unless every value passes `T.checked` |
  | field type changed otherwise | refused unless the type declares `migrate_from` |
  | field of a migrated type | migrated depth-first |
  | fields reordered, `let` changed | carried over by name |
  | methods added, removed or changed | no instance change |
  | base class changed | refused |
  | class removed while instances live | refused, naming the class and the count |
  | enum variant added, reordered or renamed with `@renamed_from` | discriminants rewritten |
  | enum variant removed while a value holds it | refused, naming the variant and the count |
  | enum variant payload changed | the field rows, applied to the payload |
  | layout attributes changed on a type used across FFI | refused |
  | static's initialiser changed | refused unless the static is `@reinit_on_reload` (`[HR-17a]`) |

* `[HR-15]` Migration keeps an instance's address where the new size fits; otherwise commit rewrites
  every reference to it that the runtime can enumerate: strong handles in live instances and
  registered containers, `Weak` handles, and interior references in registered containers.
* `[HR-15a]` A reference the runtime cannot enumerate — in foreign memory, behind a raw pointer — cannot
  be rewritten, and the reload is refused unless `[HR-20]` covers it.
* `[HR-15b]` Every refusal is decided in plan, before prepare runs.
* `[HR-16]` `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` replaces field-wise
  migration for its type. `OldSelf` is the previous schema as a generated struct `Old<Name>`, visible
  only in that function.
* `[HR-34]` Prepare cannot panic: every operation it performs is either free of panics or reported as
  a `ReloadError`.
* `[HR-35]` *(changed in 0.9.9)* `migrate_from` cannot abort the process. It may not contain an
  explicit panic (`panic`, `unwrap`, `expect`, `assert`: `E2225`, whose help names the fallible form);
  and every run-time check inside it — overflow, bounds, stale handle — returns
  `Err(ReloadError.Migration(type, instance, message))` instead of panicking.
* `[HR-39]` A foreign call inside `migrate_from` returns its failure as `ReloadError.Foreign`; a call to
  a C++ function asserted `noexcept`, which could terminate the process, is rejected at compile time.
* `[HR-43]` `@allow_reload_terminate` on a `migrate_from` admits such calls and states that this
  migration may end the process rather than refuse; `ember tcb` lists every use.
* `[HR-36]` Allocation during prepare comes from the transaction arena; exhausting it fails the reload
  with `ReloadError.Allocation`, so construction inside `migrate_from` needs no special spelling.
* `[HR-37]` The transaction arena holds everything prepare allocates and is released whole if the
  reload fails.
* `[HR-38]` `std.hot.ReloadError` is `SchemaRefused(type, reason)`, `Migration(type, instance, message)`,
  `Allocation`, `Foreign(error)` or `Timeout`.
* `[HR-17]` A static whose type and initialiser are unchanged keeps its value; one whose type changed
  migrates by `[HR-14]`; a new static is initialised normally.
* `[HR-17a]` *(changed in 0.9.9)* Editing a static's initialiser changes its schema: the reload is
  refused, naming the static, unless the static is `@reinit_on_reload`, which re-runs the initialiser
  and discards the old value. Keeping the old value silently is never an outcome.
* Regions (`[LT-30]`) are compile-time facts and play no part in schemas or migration.

## B.4 Refusal, scope and cost

* `[HR-18]` A refusal returns a report naming each type and change that caused it; it is never a crash.
* `[HR-18a]` A refusal is never partial: nothing of the new image is live.
* `[HR-18b]` `ember build --reload --explain` predicts the outcome against the running process without
  applying anything.
* `[HR-41]` **Failure matrix.**

  | What happened | Detected in | Outcome |
  |---|---|---|
  | source does not compile, or fails checking | before the poll | old image keeps running |
  | reload protocol mismatch | load | refused, `E9037` |
  | C++ header ABI changed | plan | refused (`[HR-23]`) |
  | schema change with no defined outcome | plan | refused (`[HR-14]`) |
  | `@noreload` body changed | plan | refused (`[HR-10]`) |
  | a reference that cannot be rewritten must move | plan | refused (`[HR-15a]`) |
  | allocation exhausted, `migrate_from` fails, or a foreign call fails | prepare | discarded; old image keeps running |
  | GPU resource still in flight past the bound | prepare | discarded (`[HR-24]`) |
  | panic in `migrate_from`, or a failure in commit | cannot occur | forbidden by `[HR-35]` and `[HR-2]` |
  | panic in a `drop` during reclaim | reclaim | the reload has succeeded; the panic aborts as any panic does |

* `[HR-19]` `reload = "bodies"` admits only function-body changes and changes that need no instance
  migration; it needs no schemas and no larger header, and is the first tier an implementation
  provides.
* `[HR-25]` `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); `@reloadable` and
  `@noreload` apply to modules and items, item level winning.
* `[HR-20]` *(changed in 0.9.9)* An object pinned by a `Retained` token that foreign code holds is never
  moved: if its new size does not fit, the reload is refused, unless the token was created with an
  `on_relocate` callback, which the runtime calls with the new address during commit.
* `[HR-22]` Foreign objects owned by Ember are carried across unchanged.
* `[HR-23]` A changed C++ header refuses the reload and names it (the host must be rebuilt); a changed
  overlay does not, because overlay edits change only Ember code; where the header is unchanged the
  previous thunks are reused.
* `[HR-24]` An object in use by an in-flight GPU frame is migrated in place when it fits; otherwise the
  reload waits at most `frames_in_flight + 1` polls and is then refused, naming the resource.
* `[HR-27]` A `shipping` build is bit-identical whether or not the source uses any reload attribute.
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
