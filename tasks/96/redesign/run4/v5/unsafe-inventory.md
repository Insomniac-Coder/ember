# The list of unsafe features (verdict 60)

Every feature of Ember (the new design `run4/v5`, today's spec `Ember_v0.9.10_Hardened_35`, the standard
library, the built-in functions and the runtime) that can break a safety promise outside `unsafe`: corrupt
memory, race, leak, skip a promised check, reach memory the rules cannot see, or make the generated C
undefined. Found by five agents and checked by a sixth (`inventory/01-design.md` to `05-attacks.md`, and
`inventory/06-checked.md`, which holds every program, source line and reason); merged by the lead.

**Totals:** 76 findings in all; after merging duplicates, 34 confirmed, 6 refuted, 3 unsure.
33 of the 34 are settled by the owner's earlier verdicts (mostly 60: fence it inside `unsafe`, or make it
safe with a rule); the owner decided C19 as verdict 99.

## Confirmed, most harmful first

Kinds: 1 memory misuse, 2 race, 3 leak, 4 a skipped check, 5 memory the rules cannot see, 6 undefined C.

| # | problem | kind | fix | settled by |
|---|---|---|---|---|
| C01 | `Allocator.deallocate`/`reallocate` callable without `unsafe` | 1, 6 | `unsafe fn` in the interface; a checked `Layout` | 60 |
| C02 | C runs `@export` functions, C++ overrides and running borrowed callbacks that the foreign-call rule does not count | 1, 6 | count them at every foreign call not marked `no_callbacks` | 31, 39 |
| C03 | two scoped tasks change a one-thread object's weak count | 2, 1 | making or opening a link touches the header: only `thread_shared` objects in tasks | 61 (reason) |
| C04 | a `drop` makes a new owner of its own object | 1 | refuse an owner made from `this` inside `drop` | 56 (intent) |
| C05 | `init` calls a method that lets `this` out before every field is set | 1 | such a method only after every field is set (E2100) | row 7, `[CLS-2]` |
| C06 | a struct named `Cell_...` is judged `Send` by its first field | 2 | recognise the compiler's own structs by module, not by name | `Send` row |
| C07 | a C function pointer from a header is callable without `unsafe` | 5, 1 | import it as `unsafe extern "C" fn` | 60 |
| C08 | hot reload swaps in a body whose worked-out facts changed | 5, 1 | the reload plan compares R17's facts for live frames and callers not rebuilt | 60, R17 |
| C09 | a hand-written ECS query parameter makes `run_parallel` race | 2, 1 | `unsafe interface` for `ParamOps`/`QueryParam` | 60 |
| C10 | a borrowed callback C may run on several threads changes a real variable | 2, 1 | a callback any thread may run is a `thread_shared fn()` | 61 |
| C11 | `jobs.submit` takes a plain `fn()` | 2 | `owns f: thread_shared fn()`, as `thread.spawn` | 61, 94 |
| C12 | `@export("name")` can replace a C library or runtime function | 5, 6, 1 | an unsafe form of the attribute; refuse known names | 60 |
| C13 | the uncounted link of a walk's deferred-drop region over `thread_shared` nodes | 1 | the region only for objects no other thread can end | R11's reason |
| C14 | an arena's instance changed while values made in it live | 1 | only `Allocator` methods change it then | 24, 47, 60 |
| C15 | a `@cached` result not cleared by a change through a link held elsewhere | 1 | refuse such a link into what a cached method reads | 60 |
| C16 | the `Retained` object form: safe code frees or changes an object C still holds | 1, 5 | remove the object form (C keeps objects by verdict 91's ways) | 91 |
| C17 | `[CT-5]` would put a `static mut`'s compile-time heap value in read-only memory | 1, 6 | build it on first access | 8, 60 |
| C18 | hot reload's migration ignores `using` instances and the new links | 1 | rebuild by its instance, or refuse in the plan | 24, 47, 60 |
| C19 | a loop of shared owners leaks, with only a warning | 3 | refused unless the compiler proves the loop is opened again before its objects lose their last outside owner | 99 (the owner) |
| C20 | `Arena.alloc_nodrop` and `MaybeUninit.write` skip clean-up without `unsafe` | 3 | inside `unsafe:` for types that need clean-up | 58, 60 |
| C21 | a channel kept alive by a `Sender` queued inside it | 3 | `Receiver.drop` ends the queued values | 60 |
| C22 | a `thread_shared` object's weak count is not said to be atomic | 2, 1 | say both counts are atomic | 59 |
| C23 | an atomic count compared with its maximum and then added to can pass it | 1, 4 | one `fetch_add`, a stop at a limit with room to spare | 59, 96 |
| C24 | "static re-entry" where the held lock may not be the callee's | 2, 4 | enter past the lock only when proved the same, else E7018 | 43 |
| C25 | C source compiled into the program runs without `unsafe` | 5 | `unsafe import c "./x.c"` | 60 |
| C26 | another package's overlay applied to a header it was not written for | 5, 1 | an overlay records its header's hash | `[TIER-1]`, 60 |
| C27 | GPU draw ranges are not checked against the bound buffers | 5 | check per draw, robust access | 60 |
| C28 | an exported function's `char` parameter takes any `uint32_t` from C | 1 | refuse `char` at the boundary (E5054) | 60 |
| C29 | no stack probes with `--emit c` (macOS unknown) | 1 | probe everywhere; write the flags beside `--emit c` | `[RT-12]` |
| C30 | the trampoline's 16-bit generation can come round again | 4 | retire a slot whose generation is used up | 59, `[HND-3]` |
| C31 | two thread checks run only in the `debug` profile | 2, 4 | run them in every profile | `[PRF-1]` |
| C32 | `Volatile`'s `@safety` texts do not ask for alignment | 6 | add "aligned" | 60 |
| C33 | a `Pool` keeps every retired slot | 3 (bounded) | keep, and say so | JOB: an understandable limitation |
| C34 | the C++ header's copyable handle is keyed to `@sync`, which the redesign removes | 2 | restate it for `thread_shared` | 61 |

## Unsure (technical; the lead settles them safely when the rules are written into the spec)

- **U1** a suspended generator across a reload of its body: refuse, in the reload plan, a body change that
  changes a live frame's layout or states (or keep old resume code for old frames).
- **U2** `@reinit_on_reload` on a static another static's link may point into: the plan refuses it.
- **U3** a `thread_shared` object's `drop` run on another thread through a count a view holds: that count
  counts as "may end" for R5, so the `drop`'s changes count on that thread (E7012 where they reach a
  main-thread `static mut`).

## Refuted: 6 (see `inventory/06-checked.md` section 4)

## Next

The lead writes the 34 fixes (C19 by verdict 99) and the three unsure points into the rules,
the keyword list and the tests, as part of turning the rules into the spec `Ember_v0.9.15_Hardened_1`.
