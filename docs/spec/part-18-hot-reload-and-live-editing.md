# Part XVIII — Hot Reload and Live Editing

Hot reload is a language feature in Ember, not a scripting-layer trick and not a
property of any one host. The compiler, the object model, the calling convention and
the runtime cooperate so that a source edit becomes running behaviour in a live
process with existing objects, world state and open resources preserved. This part
specifies the mechanism, the guarantees, what is refused, and the cost. Nothing in it
is specific to RageV; Part XXII shows one host wiring it up, and `[HR-30]` states the
host contract any program can satisfy.

The target is a requirement, not an aspiration: `[HR-1]` **an edit to one function
body in a 50k-line reloadable package MUST be visible in the running process within
one second** on `[BUD-1]`'s reference machine, with all live object state preserved.
`[BUD-2]`'s build budgets exist to make that reachable.

## XVIII.1 Model

The unit of reload is the **package**. A package built with reload enabled (`[HR-25]`)
becomes a *reloadable image*: a `cdylib` whose functions are reached through a
permanent thunk and whose types carry migration metadata.

```
edit .em file
  │
  ▼  ember build --reload   (only the changed module is recompiled)
new image  package.rN.dll
  │
  ▼  host calls ember_reload_poll() at a safe point (between frames)
  ┌────────────────────────────────────────────────────────────┐
  │ 1. load new image, read its reload manifest                │
  │ 2. compare schemas (types, statics, exports, enum variants)│
  │ 3. PLAN — refuse here, with a reason, or continue          │
  │ 4. PREPARE  (speculative: may fail, changes nothing)       │
  │ 5. COMMIT   (infallible: cannot fail, cannot allocate)     │
  │ 6. RECLAIM  (drop what the old image owned)                │
  │ 7. old image stays resident (never unloaded)               │
  └────────────────────────────────────────────────────────────┘
  │
  ▼  next frame runs new code
```

* `[HR-2]` **Reload is transactional, and the phase split is what makes that
  implementable.** Steps 3–4 may fail for any reason — a refused schema change, a
  a migration that returns an error, an allocation that cannot be satisfied — and
  failing there changes nothing observable: no old instance has been mutated, no
  field has been moved out of, no destructor has run, and the process continues on
  the old image with a diagnostic. **§XVIII.4a is what makes that last clause true
  rather than merely asserted**: `[HR-34]` makes PREPARE incapable of panicking and
  `[HR-36]` makes its allocation fallible, so "fails" never means "aborts". 0.7.1
  stated this guarantee while `[PAN-1]` still turned a panic in `migrate_from` into
  process death, and no phase ordering repairs that on its own. Step 5 MUST NOT be able to fail: it performs only
  pointer stores over memory reserved in step 4, and an implementation MUST NOT
  emit an allocation, a bounds check, a user callback or any other fallible
  operation inside it. Step 6 runs after the new state is live and its failures are
  ordinary runtime panics, not reload failures. A partially applied reload is a
  defect, not a permitted outcome.
* `[HR-2a]` **Phase contents are normative.** PREPARE allocates every new instance in the transaction arena of `[HR-37]`, fallibly (`[HR-36]`),
  runs every `migrate_from` (`[HR-16]`) against a read-only view of the old
  instance, and computes the full old-address → new-address relocation map. It MUST
  NOT move out of, mutate, or drop any live instance. COMMIT publishes the new
  instances, patches `type_info` pointers, applies the relocation map (`[HR-15]`),
  and repoints the thunk targets. RECLAIM drops the old instances and the fields
  removed by `[HR-14]`, running user `drop` code; a panic here is an ordinary panic
  under `[PAN-1]` and the reload has already succeeded.
* `[HR-3]` **Safe point.** A reload is applied only inside `ember_reload_poll()`,
  and only when the runtime's **Ember-depth counter is zero on every registered
  thread**. The counter is per-thread, incremented on entry to Ember code and
  decremented on exit, maintained at the same boundaries `[FFI-22]` uses for thread
  attachment and by `[THR-1]`'s thread spawn and `[JOB-1]`'s workers. It is **not**
  `[FFI-22]`'s attach flag, which records whether a thread has *ever* entered Ember
  and would be permanently non-zero in any host that calls Ember from a long-lived
  render or worker thread. A poll with any depth non-zero returns
  `EMBER_RELOAD_UNSAFE_POINT` and does nothing.
* `[HR-3a]` **Every thread that can run Ember code MUST be registered**, whether the
  host created it (`[FFI-22]`), `thread.spawn` created it (`[THR-1]`), or it is a
  job worker (`[JOB-1]`). A thread running Ember code without being registered is
  a defect in the runtime, not a permitted configuration; `debug` builds MUST
  assert on entry from an unregistered thread.
* `[HR-4]` **Old images are never unloaded.** A stale return address, a `defer`
  block captured mid-frame or a thunk target retired by a later reload therefore
  remains mapped rather than becoming a wild jump. The cost is a few hundred kB of
  resident image per reload, acceptable in a development session and absent in
  `shipping`. `[HR-4a]` A retired image's *static data* is not reused: statics live
  in the runtime-owned table of `[HR-17]`, never in image memory, so retirement
  never strands a value.

## XVIII.2 Call indirection and function addresses

* `[HR-5]` In a reloadable package, a call to a function defined in a reloadable
  package is emitted as a call to that function's **permanent thunk**. Calls to
  `std`, to `extern` functions, and to functions in non-reloadable packages are
  direct.
* `[HR-6]` **Permanent thunks are the whole mechanism.** Each reloadable function
  instance is allocated one thunk at first load, in runtime-owned executable memory,
  whose address never changes for the life of the process. The thunk loads the
  current target from the runtime's call table and tail-jumps to it; a reload
  rewrites the table entry, never the thunk. **A reloadable function's address, in
  every context that can observe one, is the address of its thunk.**
* `[HR-6a]` It follows — and this is the rule that keeps the rest of the language
  unchanged — that **`fn` values, closure code pointers, vtable slots, `dyn`
  witness tables, drop glue pointers and `extern "C" fn` values all remain ordinary
  code pointers of the shape Part IV §9 and `[OBJ-2]` already specify.** No
  representation anywhere in the language widens, and no rule elsewhere in this
  document changes. `std`, `ember_rt`, non-reloadable packages and foreign code
  therefore need no knowledge that reload exists: they load a slot and call it, and
  it is correct. A stored callback survives a reload because its address is a
  thunk, not because anything that stores it was modified.
* `[HR-7]` **Slot identity is by mangled name** (`[MNG-1]`), never by index, so
  adding, removing or reordering functions does not disturb an existing thunk. A
  function present in the old image and absent from the new one keeps its thunk,
  whose target becomes a stub that panics naming the removed function; a reload
  that would strand a *reachable* such thunk is refused by `[HR-18]` rather than
  deferred to a panic at call time.
* `[HR-8]` **The image carries an explicit reload manifest**, an image section
  listing every reloadable function instance by mangled name with its address,
  every type schema, every static, and the protocol version. The runtime reads that
  section and never the platform export directory, which on PE and ELF alike
  contains only `@export`ed symbols and would leave every ordinary method looking
  removed.
* `[HR-9]` **Cost.** One load from the call table (hot in L1) plus an indirect tail
  jump, both well predicted, and the loss of cross-package inlining for reloadable
  functions. Measured overhead on `perf/` with reload enabled MUST be reported and
  MUST NOT exceed 3%; the measurement configuration — profile, backend, host
  compiler and flags — MUST be recorded with the figure, because the number is
  meaningless without it. `[HR-9a]` The compiler MUST suppress inlining,
  devirtualisation and `[CG-C-3]`'s inline-header emission for every reloadable
  function; a reloadable function inlined into a caller cannot be swapped, and an
  implementation that inlines one has broken `[HR-1]` silently.
* `[HR-10]` `@noreload fn` opts a function out: it is called directly, may be
  inlined across package boundaries, and changing its body requires a restart (the
  reload is refused naming it). Hot loops, `@static_safe` inner functions and
  `@simd`/`@parallel` bodies SHOULD be `@noreload`. `[HR-10a]` A `@noreload`
  function MAY call a reloadable function; the call goes through the callee's
  thunk, and only the caller's own body is guaranteed direct and inlinable. An
  earlier draft of this rule forbade the call outright, which rejected the ordinary
  shape of an opted-out hot loop calling shared helpers and was unsatisfiable for
  `std` generic instances; it is not the rule. `[LNT-5]` warns (`L2005`) where a
  `@noreload` function calls a reloadable one inside a loop, because that is where
  the indirection the annotation was meant to avoid actually costs something.

## XVIII.3 Reloadable types and schemas

* `[HR-11]` Every `class`, `struct` and `enum` in a reloadable package carries a
  **schema**: the type's kind, base class, layout attributes, an ordered list of
  `{field name, field type schema hash, offset, size}`, and — for an `enum` — an
  ordered list of `{variant name, discriminant, payload schema}`. The schema hash
  is BLAKE3 over that structure, name-sensitive and offset-insensitive.
* `[HR-11a]` **Enum identity is by variant name, not by discriminant.** A reload
  that reorders variants, or inserts one, renumbers discriminants; migration MUST
  rewrite the stored discriminant of every live value of that enum to the new
  number for the same variant name. A variant renamed without
  `@renamed_from("old")` is a remove plus an add and follows `[HR-14]`'s rows.
* `[HR-12]` Class instances in a reloadable package are registered in a
  **live-instance list**: two pointers appended to the object header, which grows
  from 24 to 40 bytes **in reloadable builds only**. `[OBJ-1]`'s 24-byte guarantee
  is unchanged for `shipping` and for non-reloadable builds. This list is
  maintained by the runtime in every reloadable build, independently of `[WK-1]`'s
  debug leak reporter, which exists only in `debug` and cannot be relied on in a
  `release` build with reload enabled.
* `[HR-12a]` **The header layout is a whole-process property, and mixing is a link
  error.** Every package linked into one process MUST agree on the header size.
  The toolchain emits a symbol whose name encodes the header layout — the same
  device `[BLD-FFI-1b]` uses for the CRT and `_ITERATOR_DEBUG_LEVEL` — and a program mixing a reloadable and a
  non-reloadable package fails to link with `E9035` naming both, rather than
  writing a field at offset 24 and reading it at offset 40. `[HR-12b]` `std` is
  compiled once per header layout for the same reason, and the toolchain selects
  the matching prebuilt.
* `[HR-13]` Value-typed data (structs in `Array`s, `SoA` columns, ECS component
  storage, arena contents) is not individually registered; it is migrated through
  its **container**, which registers itself with the runtime along with its element
  type's schema. `[HR-13a]` This includes `std` containers holding a reloadable
  element type. `std` is not reloadable, but its containers are generic over types
  that are, so `Array[T]`, `Map[K, V]`, `SoA[T]`, `Pool[T]` and the ECS storages
  MUST register themselves whenever `T` originates in a reloadable package. The
  registration is emitted by the *instantiating* package, which is reloadable, so
  `std` itself needs no reload support. `[HR-13b]` Value data reachable only
  through raw pointers, foreign memory, or a `MutSpan` stored in a foreign
  structure is not reachable by migration; if such a type's schema changed the
  reload is refused (`[HR-18]`).

## XVIII.4 Migration semantics

When a type's schema hash differs between images, every live instance is migrated.
`[HR-14]` Migration is by **field name**, and the table is exhaustive over the
schema of `[HR-11]`:

| Change | Behaviour |
|---|---|
| field added with a default (`x: f32 = 1.0`) | new field initialised to that default |
| field added, type is `Default` | initialised to `Default.default()` |
| field added, no default and not `Default` | **refused** (`[HR-18]`), naming the field and suggesting a default |
| field removed | old value dropped in RECLAIM (`[HR-2a]`) |
| field renamed with `@renamed_from("old")` | value carried over |
| field renamed without the attribute | remove plus add |
| field type changed, losslessly widening (`[TYP-5]`) | converted |
| field type changed to a range type (`[RNG-1]`) | **refused** unless the value is admitted by `T.checked`; `[RNG-10]`'s construction set is closed and migration is not a member of it |
| field type changed otherwise | **refused** unless the type declares `migrate_from` (`[HR-16]`) |
| field's type is itself a migrated type | migrated depth-first; the schema hash of `[HR-11]` is recursive, so the inner change is visible in the outer hash and the inner type's own row decides the outcome |
| field reordered | value carried over (migration is by name) |
| `let` field changed | same rules; `let` does not prevent migration |
| method added, removed or changed | no instance change; witness tables rebuilt |
| class's base class changed | **refused** |
| new class added / class no longer referenced | no effect on live instances |
| class removed while instances live | **refused**, naming the class and the instance count |
| enum variant added | discriminants renumbered per `[HR-11a]`; no other effect |
| enum variant removed while a live value holds it | **refused**, naming the variant and the count |
| enum variant reordered or renamed with `@renamed_from` | discriminant rewritten per `[HR-11a]` |
| enum variant payload changed | the field rows above, applied to that payload |
| `@layout(c)`/`@packed`/`@align` changed on a type used across FFI | **refused** (a foreign structure may hold it) |
| static's initialiser changed, type unchanged | **refused** unless the static is `@reinit_on_reload`; see `[HR-17]` |

* `[HR-15]` **Relocation.** Migration preserves an instance's address wherever the
  new size fits the old allocation. Where it does not, PREPARE records the old and
  new addresses in the relocation map and COMMIT rewrites **every stored reference
  to that object that the runtime can enumerate**: strong class handles in live
  instances and registered containers, `Weak[C]` handles (which by `[OBJ-3]` point
  into the same block and are enumerated from the same live-instance list), and
  interior `ref`/`Span` values held in registered containers. `[HR-15a]` A
  reference the runtime cannot enumerate — in a foreign structure, behind a raw
  pointer, or inside `unsafe` code — cannot be rewritten; the reload is refused
  unless `[HR-20]` covers it. **`[HR-15b]` Refusal is decided in PLAN, from the
  schemas and the registered set, before PREPARE runs** — the reload never
  discovers halfway through migration that it should have refused.
* `[HR-16]` A type may define `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` (`[HR-35]` fixes the signature and the panic-freedom requirement), where
  `OldSelf` is the previous schema surfaced as a generated struct whose name is
  `Old` + the type name, whose fields are the old schema's fields with their old
  types, and which is available only inside that function. Ember calls it instead
  of field-wise migration when present. It runs in PREPARE, in the new image,
  against a read-only view; it MUST NOT mutate the old instance, and a `mut`
  receiver on it is `E2224`.
* `[HR-17]` **Statics** live in a runtime-owned table keyed by mangled name and
  schema hash, never in image memory (`[HR-4a]`). A static whose type and
  initialiser are both unchanged keeps its value; one whose type changed follows
  `[HR-14]`'s rows; a new static is initialised normally, which `[STA-2]`'s
  no-runtime-initialisers rule makes exact and cheap. `[HR-17a]` **An edit to a
  static's initialiser is a visible change, not a no-op.** The schema of a static
  includes a hash of its initialiser expression, so editing `static GRAVITY: f32 =
  9.8` to `12.0` changes the hash; the reload is refused with a message naming the
  static, and `@reinit_on_reload` on the static instead re-runs the initialiser and
  discards the old value. Silently keeping 9.8 while the source reads 12.0 is the
  one outcome this rule exists to prevent.


## XVIII.4a The reload transaction

`[HR-2]` promises that a failed reload leaves the process running the old image with
a diagnostic. The four-phase split of `[HR-2a]` is what makes that *structurally*
possible — no live instance is touched until COMMIT — but it is not sufficient on its
own, and 0.7.1 overclaimed. `[PAN-1]` makes panic **abort** by default and defers
unwinding to v2, so a panic inside a user `migrate_from` would kill the process
rather than return to the old image, and `ember_alloc` panics on exhaustion. A phase
that "may fail" is worthless if failing means the process dies.

0.7.2 closes this by making PREPARE **incapable of panicking**, rather than by
catching panics — the language has no unwinder in v1 and this rule does not
introduce one. This is `[PHIL-3]`'s approach applied to reload: make the bad outcome
unconstructible instead of writing a rule against it.

* `[HR-34]` **PREPARE is panic-free by construction.** Every operation an
  implementation performs during PREPARE MUST be one whose effect set excludes
  `Panic`, or a fallible form returning `Result`. An implementation MUST NOT emit,
  in PREPARE, an operation carrying `Panic` from any source in `[EFF-*]`'s table —
  bounds check, overflow check, integer division, `unwrap`, explicit `panic`, or
  exclusivity check. `[HR-2]`'s guarantee is then discharged by construction and
  needs no unwinder.
* `[HR-35]` **`migrate_from` is fallible and panic-free.** Its signature is
  `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]`, and its inferred
  effect set MUST NOT contain `Panic`; a body that does is `E2225`, whose `help`
  names the total form of the offending operation. The restriction is ordinary
  effect checking, not new machinery: float arithmetic carries no `Panic` and is
  unaffected, so the worked example in XVIII.7 stands; indexing must be written
  `.get(i)`, integer division `checked_div`, and a `None` becomes a `ReloadError`
  through `?` rather than a panic. A migration that genuinely cannot express its
  failure as a value has no business running inside a transaction that promises not
  to fail.
* `[HR-36]` **Allocation inside PREPARE is fallible, and the transaction handles it,
  not the programmer.** For the duration of PREPARE the transaction arena of
  `[HR-37]` is the ambient allocator, and exhaustion in it aborts the transaction
  and surfaces as `ReloadError.Allocation` rather than panicking — so ordinary
  construction inside `migrate_from` (`e = Enemy(old.entity)`) needs no fallible
  spelling and does **not** count as a `Panic` source under `[HR-35]`, which would
  otherwise make that rule unsatisfiable for any migration that builds anything.
  An implementation MUST route PREPARE's allocation through this path and never
  through `[ALC-1]`'s `ember_alloc`, which panics on exhaustion. Exhaustion during a reload
  is a refusal, not a crash: the running program has not asked for the memory, the
  programmer has.
* `[HR-37]` **The transaction arena.** Every allocation PREPARE makes — new
  instances, the relocation map, `OldSelf` views, migration scratch — is made in a
  dedicated `ReloadTransactionArena` obtained for that reload. On a successful
  COMMIT the arena's ownership of the new instances transfers to the ordinary heap
  accounting; on a failed PREPARE the arena is released wholesale (`[ARN-2]`), which
  is a single pointer reset and cannot itself fail. No individual rollback path
  exists, and none is needed.
* `[HR-38]` **`ReloadError`.** `std.hot` declares
  `enum ReloadError: SchemaRefused(TypeName, Reason); Migration(TypeName, InstanceOrdinal, str); Allocation; Foreign(CppError); Timeout`.
  Every failure inside PLAN and PREPARE produces one; the runtime carries it into
  `[HR-18]`'s report unchanged, so the message names the type and the instance
  ordinal rather than "reload failed".
* `[HR-39]` **Foreign failure during PREPARE.** A migration that calls foreign code
  through `[FFI-24]`'s boundary receives a `Result` as it does anywhere else; a
  C++ exception is translated at the thunk (`[FFI-43]`) and becomes
  `ReloadError.Foreign`. No foreign exception and no foreign `terminate` may
  propagate through a reload, and a call to an `@ffi(throws = "noexcept")`
  import from inside `migrate_from` is **rejected at compile time** (`E2227`),
  because such a call can terminate the process and `[HR-34]` says PREPARE cannot
  fail. 0.8.2c permitted it and called the termination "a known consequence"; that
  was the wrong default — it left the one hole in a transaction whose whole value is
  that it has none, and it opened by omission rather than by choice.
  `[HR-43]` is the opt-in for a programmer who genuinely wants it.
* `[HR-40]` **What the phases may therefore contain.** PLAN reads schemas and the
  registered set: no allocation, no user code. PREPARE: fallible allocation in the
  transaction arena, `migrate_from` under `[HR-35]`, no mutation of any live
  instance, no `drop`. COMMIT: pointer stores over memory PREPARE reserved, and
  nothing else — no allocation, no user callback, no check that can fail. RECLAIM:
  user `drop` code, which may panic under `[PAN-1]` exactly as a destructor may
  anywhere else, after the reload has already succeeded. **The dividing line is
  that everything which can fail happens before anything is destroyed, and
  everything after that point cannot fail.**

* `[HR-43]` **Permitting termination is explicit.** `@allow_reload_terminate` on a
  `migrate_from` admits calls to `throws = "noexcept"` imports inside it, and states
  in one place that this migration may kill the process rather than refuse. It is
  the only route past `[HR-39]`'s rejection, it is reported by `ember tcb` beside
  the `unsafe` surface because it is the same kind of claim, and `[GATE-3]` counts
  its uses in the release notes rather than requiring zero — a project that needs it
  should say so, not hide it.

## XVIII.4b Publication and memory ordering

`[HR-3]` establishes that no thread is executing Ember code when COMMIT runs. That
is a precondition, not a memory model: the reloading thread writes thunk targets that
other threads will later read, and nothing so far says when those writes become
visible. This section answers the four questions an implementer will ask.

* `[HR-42]` **Publication.** Each thread's Ember-depth counter (`[HR-3]`) is the
  synchronising object. Entry to Ember code increments it with **acquire**; exit
  decrements with **release**. COMMIT reads every registered thread's counter with
  **acquire**, writes the new thunk targets and `type_info` pointers, and then
  publishes with a single **release** store to a reload-generation counter. A thread
  next entering Ember code acquires that generation, which orders every COMMIT write
  before any call it makes. It follows that:
  * **which threads may run during COMMIT** — any of them, provided none is inside
    Ember code; the runtime does not stop the world, and a thread doing foreign or
    OS work is unaffected;
  * **when new function addresses become visible** — at that thread's next entry to
    Ember code, never mid-call, because a thread inside Ember code is what `[HR-3]`
    excludes;
  * **a thread already executing old code** — cannot exist at COMMIT. The scenario of
    a thread entering an old function, a reload landing, and that thread returning
    through replaced code is not merely survivable but **unreachable**: its depth
    counter would be non-zero and `ember_reload_poll` would have returned
    `EMBER_RELOAD_UNSAFE_POINT`. `[HR-4]`'s resident old images cover the remaining
    case — a code address captured before the reload and called after it, which is a
    thunk under `[HR-6]` and therefore current, or a raw address that escaped, which
    `[HR-20]`/`[HR-15a]` refuse to let escape.
* `[HR-42a]` **The counter is not a lock.** An implementation MUST NOT make entry to
  Ember code take a lock, and MUST NOT let the depth counter's cost scale with the
  number of threads on the entry path: it is one relaxed increment plus one acquire
  fence, and the reload side pays the O(threads) scan because reloads are rare and
  calls are not. `[COST-3]`'s reload-indirection row covers it, and `[HR-9]`'s 3%
  ceiling is measured with it enabled.

## XVIII.5 Refusal and reporting

* `[HR-18]` When any rule says **refused**, `ember_reload_poll` returns
  `EMBER_RELOAD_REFUSED`, changes nothing, and the runtime emits a structured
  report through `cfg.log` and to `target/<profile>/reload-report.json`:

```
reload refused: 2 blocking changes

  class game.enemy.Enemy
    field `armor: ArmorKind` added with no default, and ArmorKind is not Default
    → give it a default (`armor: ArmorKind = ArmorKind.None`), derive Default,
      or add `fn migrate_from(old: ref OldEnemy) -> Enemy`
    147 live instances would be affected

  fn game.physics.step
    marked @noreload and its body changed
    → restart, or remove @noreload (costs one indirect call per invocation)

the process is still running the previous image; no state was lost
```

* `[HR-18a]` A refusal is never a crash and never partial. With `[HR-2]`'s phase
  split this is implementable rather than aspirational: every refusal is decided in
  PLAN, and every failure that can still occur after PLAN occurs in PREPARE, which
  has not touched the live set.
* `[HR-18b]` `ember build --reload --explain` reports, without running anything,
  whether the current source would reload cleanly against a named running process's
  manifest, so a programmer can check before switching to the game window.
* `[HR-19]` **Bodies-only mode.** `reload = "bodies"` restricts a package to
  changes that require no migration at all — the set `[HR-14]` marks as "no
  instance change", plus function bodies — and refuses everything else. It needs
  no schemas, no live-instance list and no 40-byte header, so `[OBJ-1]`'s 24-byte
  header and `[HR-12a]`'s link guard both stay at their `shipping` values. It is
  the tier an implementation MUST provide first (Phase 7a) and the tier a project
  can use before migration is trusted.

* `[HR-41]` **The failure matrix.** Every way a reload can fail, and what each one
  leaves behind. There is no row in which the process dies or the live set is
  partially updated; that is `[HR-2]` restated as a table so it can be audited
  without reading nine rules.

| What happened | Detected in | Old image | New image | Live state | Process |
|---|---|---|---|---|---|
| Source does not compile | before the poll | keeps running | never loaded | unchanged | continues |
| Type or borrow error | before the poll | keeps running | never loaded | unchanged | continues |
| `EMBER_RELOAD_ABI` mismatch | load | keeps running | rejected (`E9037`) | unchanged | continues |
| `.embind` ABI component changed | PLAN | keeps running | rejected (`[HR-23]`) | unchanged | continues |
| Schema change with no defined outcome | PLAN | keeps running | rejected (`[HR-14]`) | unchanged | continues |
| `@noreload` function's body changed | PLAN | keeps running | rejected (`[HR-10]`) | unchanged | continues |
| Non-relocatable reference would have to move | PLAN | keeps running | rejected (`[HR-15a]`) | unchanged | continues |
| `Retained` token with no `on_relocate` must move | PLAN | keeps running | rejected (`[HR-20]`) | unchanged | continues |
| Allocation exhausted during migration | PREPARE | keeps running | discarded | unchanged | continues (`ReloadError.Allocation`) |
| `migrate_from` returns `Err` | PREPARE | keeps running | discarded | unchanged | continues |
| Foreign call in migration throws | PREPARE | keeps running | discarded | unchanged | continues (`ReloadError.Foreign`) |
| GPU resource still in flight after the bound | PREPARE | keeps running | discarded | unchanged | continues (`[HR-24]`) |
| A panic inside `migrate_from` | **cannot occur** | — | — | — | forbidden by `[HR-34]`/`[HR-35]`, rejected at compile time (`E2225`) |
| A failure during COMMIT | **cannot occur** | — | — | — | forbidden by `[HR-2]`; an implementation emitting a fallible operation there is defective |
| Panic in a drop during RECLAIM | RECLAIM | retired | **live** | **migrated** | aborts under `[PAN-1]`, as any destructor panic does — the reload had already succeeded |
| Everything succeeded | — | retired, still mapped (`[HR-4]`) | live | migrated | continues |

## XVIII.6 Reload across the FFI boundary

* `[HR-20]` **Ember objects held by foreign code.** A `Retained[T]` token
  (`[FFI-23]`) registers the foreign holder with the runtime. `std.ffi` declares
  `Retained.pin(handle, on_relocate: Option[extern "C" fn(*void, *void)])`; the
  overlay form `callback=retained` accepts `on_relocate=<fn>` in the same contract
  vocabulary as `[FFI-11]`. On reload, an object with a live token is migrated in
  place where possible; where it must move, the runtime rewrites the token and
  invokes `on_relocate(old, new)` so the foreign side can update its own copy. A
  token with no `on_relocate` whose object must move refuses the reload in PLAN,
  naming the token's creation site.
* `[HR-21]` **Foreign function pointers into Ember** need no special rule:
  by `[HR-6]` an `@export`ed function's address is its permanent thunk, so a table
  the host captured once — RageV's `NativeApi`, or any other — stays valid across
  every reload with no re-read. `[HR-21a]` Adding, removing or changing the
  signature of an `@export` function or an `@export_table` field changes the module
  protocol and is refused; that is a rebuild of the host's contract, not a reload.
* `[HR-22]` **Foreign objects held by Ember.** `ForeignBox[T]`, `CppShared[T]` and
  opaque handles are carried across migration unchanged. Their foreign side is
  untouched by an Ember reload, which is the intent: the renderer, the physics
  world and open files survive. `[HR-22a]` If the *foreign* type's own layout
  changed, Ember cannot detect it and the `.embind` hash of `[HR-23]` is what
  refuses the reload.
* `[HR-23]` **C++ thunks.** The `.embind` hash of `[FFI-14]` is split into two
  components: an **ABI component** over the header path, header content and
  compiler configuration, and an **overlay component** over the Ember-side overlay
  file. A changed ABI component refuses the reload, naming the header and stating
  that the host itself must be rebuilt. A changed overlay component does not:
  overlay edits are Ember-side, change no C++, and are recompiled into the new
  image like any other Ember source. `[HR-23a]` Where the ABI component is
  unchanged the previous image's thunks are reused by symbol.
* `[HR-24]` **In-flight GPU work.** An object registered as borrowed by an
  in-flight frame (`[GPU-4]`) is migrated in place where the new size fits. Where
  it does not, the reload is deferred, at most `frames_in_flight + 1` polls; if it
  still cannot proceed the reload is **refused** naming the resource and its frame,
  rather than deferred again. Unbounded deferral is not a permitted outcome: with
  two or three frames always in flight it never terminates, which is the reason for
  the bound.

## XVIII.7 What the programmer writes

Nothing, in the common case.

```ember
class Enemy(Script):
    max_health: f32 = 100.0
    health: f32 = 100.0

    @renamed_from("speed")
    move_speed: f32 = 3.0

    aggro_radius: f32 = 8.0        # newly added: existing enemies get 8.0

    override fn on_update(mut self, dt: f32):
        ...                         # edit freely; live enemies keep their health


@noreload                           # direct calls, inlinable, no indirection
@static_safe @noalloc
fn integrate(mut p: SoA[Particle], dt: f32):
    ...


extend Enemy:                       # for a change field-wise migration cannot express
    fn migrate_from(old: ref OldEnemy) -> Result[Enemy, ReloadError]:
        e = Enemy(old.entity)                              # allocates in the transaction arena
        e.health = old.hp_percent * old.max_health / 100.0 # f32 math: no Panic effect
        return Ok(e)
```

* `[HR-25]` **Scope is opt-in and explicit.** `reload` is a manifest key with the
  values `"all"`, `"opt-in"`, `"bodies"` and `"none"` (`[MAN-7]`); `@reloadable`
  and `@noreload` apply at module and item level, and item level wins. Reload is
  available in `debug` and `release` and forbidden in `shipping`. Because
  `[HR-12a]` makes the header layout a whole-process property, a program's packages
  MUST agree on whether reload is enabled at all; `"opt-in"` and `"none"` differ in
  which functions get thunks, not in header layout.
* `[HR-26]` `@field`-marked script fields (Part XXII §1) migrate by this same
  mechanism, so an editor inspector and hot reload share one metadata path.

## XVIII.8 Cost and profile behaviour

| | `debug` | `release` | `shipping` |
|---|---|---|---|
| call indirection | yes | opt-in (`reload`) | **never** — all calls direct |
| object header | 40 bytes | 40 with reload, else 24 | **24** |
| live-instance list | yes | with reload | no |
| schema metadata in image | yes | with reload | stripped |
| `@noreload` functions | direct, inlinable | direct, inlinable | identical to every other function |

* `[HR-27]` A `shipping` build MUST be bit-identical whether or not the source
  contains `@noreload`, `@renamed_from`, `@reinit_on_reload` or `migrate_from`;
  reload constructs have no shipping representation and `migrate_from` bodies are
  dead-stripped. This is `[PRF-1]` applied to this part: reload changes performance
  and representation *within* a reloadable build, and changes nothing observable in
  a build without it.
* `[HR-28]` `ember inspect --safety` reports reload indirection per function
  alongside the safety checks, so the cost is visible where every other implicit
  cost is.

## XVIII.9 Host integration

`[HR-29]` The runtime lives in **one** place per process. When reload is enabled,
`ember_rt` MUST be linked dynamically and shared by every image: the call table, the
thunks, the live-instance list, the statics table and the heap all belong to it, and
a reloadable image that statically linked its own copy would load with an empty live
set and a separate heap. The toolchain MUST reject a reloadable package configured
for static runtime linkage with `E9036`.

`[HR-30]` **The host contract** is four calls, and any program that can call them at
a point where no Ember frame is live can hot-reload. There is no requirement that
the host be an engine, be C++, or run frames.

```c
/* once */
ember_reload_config rc = { .watch_dir = "src", .on_report = my_log_fn };
ember_reload_init(&rc);

/* at any point where this thread holds no Ember frame */
switch (ember_reload_poll()) {
  case EMBER_RELOAD_NONE:         break;  /* nothing pending          */
  case EMBER_RELOAD_APPLIED:      break;  /* new code is live         */
  case EMBER_RELOAD_REFUSED:      break;  /* report already emitted   */
  case EMBER_RELOAD_UNSAFE_POINT: break;  /* a thread is inside Ember */
}
```

* `[HR-31]` `ember_reload_poll` is non-blocking: compilation runs in a background
  process started by the file watcher, and the poll applies an image only once it
  is complete. A poll with nothing pending costs one atomic load.
* `[HR-32]` `ember_reload_stats` reports the last reload split into compile / load /
  plan / prepare / commit / reclaim, the live instances migrated, and the refusals,
  so `[HR-1]`'s one-second budget is measurable from inside the host.
* `[HR-33]` `ember run --hot` is the toolchain's own host: it builds under `debug`,
  runs the binary, watches the sources and calls `ember_reload_poll` from a
  supervisor thread at a point the program declares with `hot.checkpoint()` from
  `std.hot`. A program that declares no checkpoint and does not call
  `ember_reload_poll` itself never reloads, and `ember run --hot` MUST say so after
  ten seconds rather than appearing to hang.

---

