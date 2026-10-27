# Gap map 05: threads and unsafe (THR, PAR, JOB, CORO, UNS, TIER, TCB, GPU)

Agent 05, 2026-10-27. Read-only: nothing was built or run. Scratch scripts are in
`target/gap-0.9.15/05/`: `extract.py` prints a rule's text from both spec trees, and `design.py`
counts the design tests by their `#$ rules:` lines.

**Scope.** The H.41 table lists 45 rules of these families: 43 `changed`, 2 `added`, none
`removed`. THR-16 is marked "added" in an earlier H section (line 241), but its text is the same in
both trees, so it is not in H.41. Two more rows cover features the compiler has under removed names:
`@sync`/`SyncShared` and the `fn() + Send` callable. That makes 47 rows.

**How the old tests were classified.** I read all 105 files in my rules' folders by hand. No script
did the classification. My convention:
- `respell`: the program means the same and is accepted or refused the same way. Only spellings change
  (`self` to `this`, `owned` to `owns`, `.clone()` to `copy of`, `mut xs` to `ref mut xs`, the `[T]`
  line of `[GRM-40]`, `str` to `StrView`). An expected error code or message changes only because the
  wording or the code table changed.
- `rewrite`: what is accepted or refused changes.
- `retire`: the test checks only a removed feature.

A warning the runner was not told to expect does not fail a test. The runner checks only the
warnings and lints a test declares (`compiler/ember_driver/tests/milestones.rs:107-112, 226-229`).
So the new W3013 on `unsafe fn …:  # SAFETY:` lines does not by itself make a test respell.

**Design tests.** 127 of the 841 name at least one of my rules. Many tests in R15 are tagged
`THR-1` loosely, including C-callback and walk tests that belong to the FFI and effect areas. The
counts per rule come from `design.py`.

---

## 1. Rule table

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| CORO-3 | changed | A frame is not `Copy` and is handed over by `=`. Generators of different functions are kept as `owns dyn Iterator`. | Frames are distinct sized structs (`ember_typeck/src/coroutines.rs`). Mixed generators are kept as `Box[dyn …]`. | text only | Spelling `owns dyn` comes from the ownership area. No checker change. | keep 2; respell 1: `accept_generic_generator_methods` (`self`→`this`, `[U]`/`[T]` lines per GRM-40) | 0 |
| CORO-5 | changed | A frame is an ordinary value handed over by `=`. `owns dyn` is used only to keep generators of different functions together. | `ember_analysis/src/coroutines.rs:644-649` sizes held frames; `E2200` refuses a frame that holds itself. | text only | none | keep 1; respell 1: `accept_a_generator_allocates_nothing` (`struct Holder[G]`→`[G]` line) | 0 |
| CORO-6 | changed | A view into the frame's own local across `yield` is `E2221`. A `for` over a list, string, map or set whose body yields walks by position, re-reading the collection after each resume, so it compiles. | `ember_analysis/src/coroutines.rs:574-612` refuses any borrow of a local live across a `yield`. That includes the iterator of `for x in xs` over a local list. | partly | Lower a gen fn's `for` over its own collection by position: an index kept in the frame, the collection re-read and the length re-checked after each resume. Exempt that loop from E2221. Write the new E2221 message. | respell 2: `accept_a_suspended_frame_moves` (`.clone()`, `owned g`, `[G…]` line), `accept_views_of_parameters_across_a_yield` (`str`, `self`, `for x in owned`→`owns`); rewrite 1: `reject_a_borrow_of_a_local_across_a_yield` (`over_a_local` now compiles) | 2 (R15) |
| CORO-12 | changed | A gen fn method has `this`. Its frame is a view of the object (CORO-14), and its body changes the object as any method's does. | `ember_analysis/src/coroutines.rs:540-550` gives E2229 for `mut self`; `:613-624` gives E2229 for a field borrow held across a `yield`. | partly | Remove both E2229 refusals (the code is retired, `p17a-codes.md:284`). The frame becomes a view of the object (A5-7). | respell 1: `accept_a_class_generator_method` (`self`→`this`); retire 1: `reject_mut_self_and_a_held_field` | 0 |
| CORO-13 | changed | A `yield` while a `@must_drop` value is live is E2231. | `ember_analysis/src/coroutines.rs:629-641` | text only | Reword the note: "may never be resumed", not "forgotten". | keep 2 | 0 |
| CORO-14 | added | A generator is a walk of what it views. While it will still be used, a change to the collection it walks is E3020, any other change to what it views is E3021, and `from` narrows the view. | Nothing: no `from` on a gen fn, and no tracking of a generator value as a walk. | not built | A5-7 | none (no folder) | 0 tagged (`R15/run_a_generator_walks_a_list_by_position.em` shows `from log`) |
| GPU-2 | changed | `Frame` has one owner; `end_frame(owns frame)` takes it over. | No `std.gpu`: nothing in `std/src`, the runtime or the compiler (grep for `begin_frame`/`std.gpu`: no hits). | not built | A5-22 | none | 0 |
| GPU-3 | changed | `frame.arena()` is a memory-manager instance; a value made `using` it is E3063 past the frame. | none | not built | A5-22 (needs ALC-1 `using`) | none | 0 |
| GPU-6 | changed | Wording: "ending the device". | none | not built | A5-22 | none | 0 |
| GPU-7 | changed | Wording: "ending a device". | none | not built | A5-22 | none | 0 |
| GPU-9 | changed | `pass.native(fn(ref mut cmd) => …)` | none | not built | A5-22 | none | 0 |
| GPU-12 | added | Every draw and dispatch is compared against the bound buffers in every profile, with robust access on, `device.robust_access()`, an asserted fact in `ember tcb`, and `unsafe: cmd.native()`. | none | not built | A5-22 (runtime compares) | none | 0 |
| JOB-2 | changed | `JobScope.submit` takes a plain `fn()`. `jobs.submit` takes `owns f: thread_shared fn()`: a function holding a one-thread value is E7015, and a name with no capture word is E7004. | `ember_typeck/src/lib.rs:52513-52549` (`synth_jobs_submit`) plus `owned_task_refused` `:52339-52376`, which give E7004 for a non-Send capture and E3026/E3063 for a view. `:52662` (`synth_job_scope_submit`) plus `scoped_task_refused` `:52726-52759` give E7005/E7004. `std/src/jobs.em:23` is `submit(owned f: fn())`. | partly | As `thread.spawn` (A5-12, A5-13); scope jobs as scoped tasks (A5-14). | respell 4: `accept_submit_wait_and_scopes` (`owned fn`→capture words, `mut xs`, call words); `reject_a_job_holding_a_view_of_a_parameter` (E3063→E7015 with `copy of text`); `reject_moving_a_job_scope` (message "moved"→"handed over"); `reject_what_a_job_may_not_hold` (`Shared`→`shares`, E7004→E7015) | 0 |
| JOB-3 | changed | `submit_after` orders execution but does not relax THR-11. | Ordered jobs conflict through the borrow model (`ember_analysis/src/regions.rs:1136`), E3022. | partly | The conflict comes from A5-14 as E7007. | respell 2: `accept_a_job_after_others` (`mut xs`, call words); `reject_ordering_is_not_exclusion` (E3022→E7007) | 0 |
| PAR-1 | changed | `@parallel` splits a range, or a list walked by element (`for ref mut p in players:`). Anything else is E7016. | `ember_typeck/src/lib.rs:51936-51939` accepts only `a..b` and gives E7010 otherwise. | partly | Add the element walk (chunks of the list, the `ref mut` loop name) and E7016. | keep 1; respell 1: `accept_a_loop_run_in_chunks` (`ref mut xs = …as_mut_span()`, verdict 34) | 1 |
| PAR-2 | changed | Iterations are independent. What the loop uses follows THR-11 and THR-2. A place reached through a link (`shares`, `shares mut`, `sharedref mut`, `thread_shared`), a `Cell` or a raw pointer is refused. In an element walk, only declared reductions are written, and `Array[shares mut T]` is refused. E7010 messages give the fix line. | The offset analysis is built (`ember_typeck/src/lib.rs:52268-52331`, PAR-2a/2b included). Captures are judged by Send/Sync, E7004/E7005 (`:52226-52261`). | partly | Refuse link-reached places and lists of `shares mut`. Add the element-walk mode. Judge captures by A5-8's words. New messages. | respell 3: `accept_independent_iterations` (`mut ps`, call words), `reject_a_capture_its_chunks_cannot_share` (`Shared`→`shares`, E7005→E7015), `reject_iterations_that_are_not_independent` (`mut xs`→`ref mut xs`) | 2 |
| TCB-4 | changed | The unsafe section lists `unsafe:` blocks, `unsafe fn`, `unsafe import c`, `@unsafe(export…)` and `@unsafe(export_table…)`. The C FFI section lists `safe fn`, `@ffi(no_callbacks)`, `@ffi(safe)` and thread contracts. The C++ section lists `@ffi(const)`. | `ember_driver/src/lib.rs:2305-2530` lists blocks, `unsafe fn` and the foreign declarations. Categories are at `:1719-1740`. | partly | A5-5 | none (no folder) | 0 |
| THR-1 | changed | A value several threads own is `thread_shared`: E7017 when the value was not made so, E7014 for a direct change, E7015 for a one-thread value reachable from it. `thread_shared fn()`. | Only `@sync`: `ember_typeck/src/lib.rs:6202-6240` (E7001 fields), `:6131-6141` (E7001 hierarchy), `:23790-23797` and `:44424` (E7003), attribute table `:60332`, `is_sync` set at `:6004`/`:6979`. Nothing for `thread_shared`. | not built | A5-9, A5-10, A5-11 | retire 12: every file in `THR-1/` (`accept_generic_sync_class_of_int`, `accept_sync_class_uses_atomic_reference_counts`, `reject_a_sync_class_holding_an_owned_callable_value`, `reject_cell_field_even_when_let`, `reject_generic_sync_class_of_non_sync_handle`, `reject_mixed_sync_hierarchy`, `reject_mut_self_method`, `reject_mutable_borrow_of_sync_field`, `reject_mutating_collection_field`, `reject_non_sync_field`, `reject_post_init_field_write`, `reject_write_after_init_publishes_self`) | 80 (R12 8, R15 63, R3 2, R9 7) |
| THR-2 | changed | A one-thread value never reaches another thread by any route. Making, ending or opening a `weakref` in a task or job is allowed only for a `thread_shared` target. Handing over an object while a `weakref` on this thread points at it is E7015. | `ember_types/src/lib.rs:1803-1806`: a class handle is Send iff `@sync`. Runtime creator-thread check: `runtime/ember_rt/src/ember_rt.c:3119-3130`. | not built | A5-8, A5-12, A5-14 | rewrite 2: `accept_a_sync_class_handle_crosses_threads` (now `thread_shared`); `reject_a_handle_of_a_class_that_is_not_sync` (`Plain` and `Holder` handed over with `owns` are now Send, so it is accepted) | 14 |
| THR-3 | changed | Mutex and RwLock guards. A guard sent to another thread is E7008. A view through a guard does not leave its `with` (E3063). Lock types and channel ends change through any name with no word at the call (`ref mut` there is E3028). | `std/src/sync.em:53-80` (guards are `@view` structs). `ember_types/src/lib.rs:1842-1844`: a guard is not Send, so sending one gives E7004. | partly | A5-19 | keep 4; respell 1: `reject_a_write_through_a_read_guard` (E3021 message wording) | 1 |
| THR-5 | changed | A `with` scope as before, and also: it cannot be held in a function value's state, and E7018 when its block ends inside a lock that a task's function needs. | `ember_typeck/src/lib.rs:51542-51558` (E3014 not bound by `with`); `ember_analysis/src/drops.rs:1769-1790` (E3014 moved). | partly | E7018 (A5-17); "handed over" wording | keep 2; respell 1: `reject_moving_a_scope` (message) | 0 |
| THR-6 | changed | `@must_drop` may not be stored in a class field, a `shares` or `thread_shared` value, an `owns` block, a collection or a capture. No `mem.forget`. | `ember_typeck/src/lib.rs:6145-6182`; `ember_types/src/lib.rs:2279-2330` (Box, Shared, SyncShared, collections, owned fn capture) | partly | A5-19 (retarget to the new containers) | respell 1: `accept_a_must_drop_value_ends_with_its_scope` (`drop(mut self)`, `owned g`); rewrite 1: `reject_where_a_must_drop_value_may_not_go` (`mem.forget` and `Box` lines are now E1010 or `owns`) | 0 |
| THR-7 | changed | `thread_shared` means exactly this; no diagnostic calls it "thread-safe". | none | text only | New messages must follow it | none | 0 |
| THR-8 | changed | `Send` is inferred from the words. `unsafe extend T implements Send` is required for a raw pointer (E3100). A thread-bound value sent is E7008. | `(send, sync)` pair at `ember_types/src/lib.rs:1777-1918`. The class rule is `is_sync`. Hand-written markers: `ember_typeck/src/lib.rs:14011` (E3100 without `unsafe extend`, built) and `:14040`. | partly | A5-8 | respell 2: `accept_a_dyn_of_an_interface_requiring_send_moves_to_a_thread` (`Box`→`owns dyn`, `Sync`→`ThreadShared`, `self`, capture words), `accept_send_written_by_hand` (`need_sync`→`ThreadShared`, `[T]` lines, `owns r`); rewrite 2: `reject_a_dyn_of_an_interface_not_requiring_send_or_sync` (class `Handle` is now Send, so the E2040 line goes), `reject_markers_written_wrongly` (`Sync` is now E1010, not a class rule) | 1 |
| THR-9 | changed | `ThreadShared` replaces `Sync`. `Sync` in a bound is E1010. Members: `Atomic`, `Once`, `Lock`, `Mutex[Send]`, `RwLock[Send+TS]`, `thread_shared T`, `Sender`. | Sync half of `thread_properties` (`ember_types/src/lib.rs:1838-1870`). Interfaces `std.core.Sync` (`:1774`). | partly | A5-8 | respell 1: `accept_once_runs_its_initialiser_once` (`str`); rewrite 6: `accept_a_mutex_is_sync_when_its_value_is_send`, `accept_a_mutex_of_a_cell_is_sync`, `accept_an_rwlock_is_sync_when_its_value_is_send_and_sync`, `reject_a_mutex_of_a_value_that_is_not_send`, `reject_a_sync_shared_of_a_value_that_is_not_sync`, `reject_an_rwlock_of_a_value_that_is_not_sync` (all built on `@sync`/`SyncShared`; restate on `thread_shared`, E7015) | 0 |
| THR-10 | changed | `spawn(owns f: thread_shared fn() -> R)`. E7015 or E7008 when `R` is not Send. The capture list follows THR-17. A spawn inside a walk of what it changes is E3020, except under the walk's lock. A join inside that lock is E7018. | `ember_typeck/src/lib.rs:52793-52832` (`synth_thread_spawn`): `owned_task_refused` `:52339-52376` (E7004 non-Send capture, E3026 view), `R` Send E7004. `std/src/thread.em:25` is `spawn[R](owned f: fn() -> R)`. | partly | A5-12, A5-13, A5-17 | respell 3: `accept_a_thread_capturing_a_static_view` (`copy of greeting`), `reject_a_view_of_a_local_sent_to_a_thread` (E3063→E7015), `reject_join_under_noblock` (`owns h`); rewrite 2: `accept_threads_spawn_join_and_detach` (`SyncShared` is gone), `reject_what_a_thread_may_not_take` (`Plain` is now Send; a plain `fn()` value is a type mismatch; no-word `total` is E7004) | 18 |
| THR-11 | changed | `scope.spawn(owns f: fn() -> R)` uses the caller's variables. Each task's effects are worked out and checked against other tasks and the parent: E7007 unless by reduction, lock or Atomic. A task started in a walk is E3020. A task makes no `shares` owner, and `weakref` only to `thread_shared` targets. | `ember_typeck/src/lib.rs:51564-51617` plus `scoped_task_refused` `:52726-52759` (E7005/E7004). Conflicts come from the borrow checker: the closure flows into the `Scope` (`ember_analysis/src/regions.rs:1136-1141`, `borrows.rs:220-247`), giving E3021/E3022. | partly | A5-14, A5-15 | keep 1; respell 4: `reject_a_handle_outliving_its_scope` (E3060→E3063 by the new table), `reject_a_task_sharing_what_is_not_sync` (`RefCell`→`Cell`, E7005→E7015), `reject_two_tasks_writing_one_place` (E3022→E7007, `ref mut`), `reject_writing_what_a_task_reads` (E3021→E7007, pending Q2); rewrite 1: `accept_scoped_tasks_borrow_the_block` (`@sync` is gone; `t` with no word is now E3026) | 6 |
| THR-12 | changed | Wording: "views". | Two scopes in sequence work. | text only | none | none | 0 |
| THR-13 | changed | The data-race guarantee: the list of shared places and how each is ordered. | None as such; it is the sum of THR-1 to THR-21. | not built | The items of THR-1 to THR-21 | rewrite 2: `accept_the_ways_threads_share`, `reject_sharing_by_any_other_way` (both use `@sync`/`SyncShared`/`Shared`) | 0 |
| THR-15 | changed | `Sender` is ThreadShared and Clone (`copy of tx`). When the `Receiver` ends, it takes the queued values out under the lock and ends them after letting the lock go. | `std/src/sync.em:216-219` (`clone`). `:255-262`: the Receiver's drop sets `receiver_gone` and leaves the values queued. | partly | A5-19 | respell 4: `accept_producers_bounds_and_disconnection` (`tx.clone()`, capture words), `accept_the_producer_example` (`fn() owns tx`), `reject_a_second_receiver` (E3040 wording), `reject_cloning_a_receiver` (`copy of rx` refused instead of `.clone()` E2072) | 0 |
| THR-17 | added | An unscoped thread, unscoped job or kept callback names its captures (`owns`, `thread_shared`, `copy of`), else E7004. The loop's variable or a round local goes to a task with `copy of t` or `ref t`, E3026 without; to a thread with `copy of t`, E7004 without; `ref t` in a thread is E3026. | `owned fn` closures only (`owned_task_refused`, `ember_typeck/src/lib.rs:52339`). No capture lists. | not built | A5-12, A5-13 | none | 4 |
| THR-18 | added | `Lock` and `guarded by`: E7013 outside the lock, `init` exempt, summaries "needs lock held", entry past a held lock only when proven the same lock, else E7018. The verdict-89 exception lets one object reach a call through many names. | Nothing: no `Lock` in `std/src/sync.em`, no `guarded by` in the parser. | not built | A5-16 | none | 4 (about 10 more tagged THR-1) |
| THR-19 | added | Inside a lock's `with`: a call through a function value, `dyn` or virtual method that may take a lock of that kind; a join or scope end whose task changes what the lock guards; a direct call that may take it. All E7018. | nothing | not built | A5-17 | none | 10 |
| THR-20 | added | `static mut` is reached only by the main thread and its scoped tasks unless `guarded by` a static `Lock`; else E7012. A `drop` runs where the last owner ends, which counts for E7012. | Ember `static mut` is refused outright: `ember_typeck/src/lib.rs:10259-10264` (E1010 "needs `unsafe`, which is not in this phase"). Statics: `:10196` (E7002 `is_sync`). | not built | A5-18 (needs the STA area's `static mut`) | none | 2 |
| THR-21 | added | Reductions (`+= -= *= \|= &= ^=`, max, min, push, extend, add, `m[k] op=`) on locals or reached fields. Private accumulators are folded at the join in spawn order. Floats are allowed. Reading before the join, or a plain `=`, is E7007. | Nothing for tasks. `@parallel`'s declared reductions exist and can be reused: `ember_typeck/src/lib.rs:52146-52220` (`reduction_identity`, `reduction_combiner`). | not built | A5-15 | none | 6 |
| TIER-1 | changed | Boundaries: `unsafe:` (also inside an `unsafe fn`), `unsafe extern`, `unsafe overlay`, `unsafe import c`, `@unsafe(export("name"))`, `unsafe extend`. | An `unsafe fn` body is an unsafe context: `in_unsafe` is set from `decl.is_unsafe` at `ember_typeck/src/lib.rs:20476, 20548, 22958, 23981, 40089, 40396`. Blocks: `:26299-26308`. | partly | A5-1; FFI area for `unsafe import c`/`@unsafe(export)` | none (no folder) | 10 |
| UNS-1 | changed | The unsafe list adds `into_raw()`, MaybeUninit writes of a needs-drop type, `Allocator.deallocate`/`reallocate`, a hand-written `Send` and `assume_disjoint`. Only a *foreign* `static mut` needs unsafe. Each is E3100, with a message giving the block to write. | E3100 sites: `ember_typeck/src/lib.rs:14011, 16261, 16478, 16654, 17416, 22806, 37584, 40702, 45800, 47580, 47718, 50021, 51679, 55528`. Foreign calls and foreign static writes give E5002 (`:22786-22804`, `:25834`). `into_raw` exists only as generated C++-bridge text (`ember_driver/src/cimport.rs:7448`). | partly | A5-1, A5-4; message text; Q1 | keep 6; respell 6: `accept_a_raw_pointer_is_read_written_and_offset` (inner `unsafe:` in the `unsafe fn` per UNS-12, `@safety`), `accept_unsafe_generic_call_in_block` and `reject_unsafe_generic_call_in_safe_code` (`[T]` line), `accept_unsafe_method_call_in_block`, `reject_unsafe_method_call_in_safe_code`, `reject_an_unsafe_method_of_a_generic_type_outside_unsafe` (`self`, `[T]`); rewrite 1: `reject_an_unsafe_implementation_of_a_safe_interface_method` (its `warning[W3012]` on an `unsafe fn` no longer holds, UNS-8) | 4 |
| UNS-2 | changed | Wording only: "the checks of Parts VII and VIII". | built | text only | none | none | 0 |
| UNS-4 | changed | The obligations on unsafe code, restated for views, counts and `weakref`. | not checkable | text only | none | none | 0 |
| UNS-7 | changed | *Every* `unsafe fn` carries `@safety`, else L3015. A `# SAFETY:` note on an `unsafe fn` line is W3013. | `ember_typeck/src/lib.rs:43034-43062`: L3015 only when `is_public` (`:43057`). No W3013 in `codes.rs`. | partly | A5-2 | keep 6; respell 2: `accept_documented_public_unsafe_method`, `warn_public_unsafe_method_without_safety` (`self`) | 1 |
| UNS-8 | changed | W3012 for `unsafe:` blocks and for `unsafe extend`/`extern`/`overlay`, not for `unsafe fn`. `ember tcb` lists each block. | `ember_driver/src/lib.rs:7352-7369` over `module.unsafe_sites`. The parser pushes `unsafe fn` sites (`ember_parser/src/decls.rs:992`) and blocks (`body.rs:216`). | partly | A5-2 | none (no folder) | 2 |
| UNS-10 | changed | `UnsafeCell`: never ThreadShared; refused in `@no_alias_checks` code (E3105). | `ember_typeck/src/lib.rs:19841` (`@static_safe`); `ember_types/src/lib.rs:1836` (Send of T, never Sync). | partly | The rename follows the ATT area's `@static_safe`→`@no_alias_checks` (A5-3) | keep 5; respell 2: `accept_layout_drop_and_generic_substitution` (`drop(mut self)`, `[T]` line, `owns cell`), `reject_use_after_move_even_for_copy_payload` (E3040 wording) | 0 |
| UNS-10a | changed | Wording only. | built | text only | none | keep 1; respell 1: `reject_unsafe_block_disabling_borrow_checking` (E3022 wording) | 0 |
| UNS-11 | added | Nothing can break a safety promise outside `unsafe`, and no switch turns a check off. `@no_runtime_checks` is E0104 with the given message. `unsafe` has one meaning by position. | `ember_typeck/src/lib.rs:60367` keeps `no_runtime_checks` in `RESERVED_ATTRIBUTES` (E0104 "reserved"). Manifest switches are refused, `ember_driver/src/lib.rs:6789-6798` (E9001). | partly | A5-3 | none | 0 tagged (`R16/reject_no_runtime_checks.em` names ATT rules) |
| UNS-12 | added | An `unsafe fn` body is not an unsafe context: each unsafe step needs its own `unsafe:` block, else E3100. | Not built: see TIER-1's six sites. | not built | A5-1 | none | 4 |
| `@sync` class and `SyncShared[T]` (the old THR-1/THR-9/THR-13 text, HEAP-10's type) | removed name | Gone. `@sync` is E0104 (`p17a-codes.md:21`); E7001/E7003/E7005 are retired. | Typeck sites above. `std/src/sync.em:104` (`SyncShared`) and `:168, 222, 278` (channel state). `std/src/ecs.em:1206`. Per-type atomic flag `EMBER_TI_SYNC`: `compiler/ember_codegen_c/src/lib.rs:4994, 5365`, `runtime/ember_rt/include/ember_rt.h:1416, 1933-1947`, `ember_rt.c:3005, 3153, 3178`. | remove | A5-9 | counted above; 37 test files use `@sync` and 7 use `SyncShared` across `tests/` (other areas' folders: HEAP-10, ECS-4, RC-3, CELL-3/8, CLS-8, TYP-35, EFF-5, PHIL-1, RT-10, BLD-FFI-5a, perf) | - |
| `fn() + Send` (ODR-264) | removed name | Replaced by `thread_shared fn()` (THR-1, verdict 61). | `TyKind::Fn { send }`: `ember_types/src/lib.rs:223, 1796`; `callable_with_send` at `ember_typeck/src/lib.rs:40857`. | remove | A5-10 (reuse the bit as `thread_shared`) | - | - |

**Status counts (47 rows):** built 0, partly 21, not built 16, remove 2, text only 8.

**Old tests (105 files):** keep 31, respell 43, rewrite 18, retire 13.

---

## 2. Work items

These are ordered. "Area X" means the area mapping the X family; I do not know the other agents'
numbers. Every item updates `compiler/ember_diag/src/codes.rs` and `docs/errors/` for its own codes
(see A5-21).

### Slice U: unsafe. Independent of the ownership rework, so it can land first.

**A5-1. An `unsafe fn` body is not an unsafe context** (UNS-12, TIER-1 in part)
- Files: `ember_typeck/src/lib.rs` (stop seeding `in_unsafe` from `decl.is_unsafe` at 20476, 20548,
  22958, 23981, 40089, 40396; E3100's message gives the block to write); `std/src/mem.em:28-40`
  (volatile bodies get their own `unsafe:`).
- Keep `body.is_unsafe` for CORO-9's frame pointers (`ember_analysis/src/coroutines.rs:579`).
- Size: S.
- Tests: R16 3 (`accept_an_unsafe_fn_vouches_block_by_block`,
  `reject_an_unsafe_operation_in_an_unsafe_fn_body_outside_a_block`,
  `reject_calling_an_unsafe_fn_outside_unsafe`). Old: respell `UNS-1/accept_a_raw_pointer_is_read_written_and_offset`
  and any other test or std `unsafe fn` doing raw work in its body (32 test files contain `unsafe fn`;
  most bodies are trivial).
- Depends on: nothing.

**A5-2. Safety notes** (UNS-7, UNS-8)
- Files: `ember_typeck/src/lib.rs:43057` (drop the `is_public` condition); new `W3013` in `codes.rs`;
  `ember_driver/src/lib.rs:7352` (W3012 for blocks and `unsafe extend`/`extern`/`overlay`, W3013 for a
  note on an `unsafe fn` line); `ember_parser/src/decls.rs:992` (mark the site's kind); the tcb
  listing at `ember_driver/src/lib.rs:2305-2530`.
- Size: S.
- Tests: R16 3 (`accept_a_safety_note_on_an_unsafe_fn_line_is_warned`,
  `accept_an_unsafe_block_without_a_safety_note_is_warned`, `accept_an_unsafe_fn_without_safety_is_warned`).
  Old: rewrite `UNS-1/reject_an_unsafe_implementation_of_a_safe_interface_method`.
- Depends on: nothing.

**A5-3. No switch turns a check off** (UNS-11, UNS-10's E3105 wording)
- Files: `ember_typeck/src/lib.rs:60367` (give `no_runtime_checks` its own E0104 with the spec's
  message) and `:19841`.
- Size: S.
- Tests: `R16/reject_no_runtime_checks.em`.
- Depends on: area ATT's rename of `@static_safe` to `@no_alias_checks`.

**A5-4. `into_raw()` built in, usable only inside `unsafe`** (UNS-1, OWN-12, verdict 91)
- Files: `ember_typeck` (builtin method on an `owns` class object: E3100 outside `unsafe`, then the
  name ends), `ember_mir` (no drop), both backends.
- Size: M.
- Tests: R16 2 (`accept_into_raw_inside_unsafe_gives_c_the_address`, `reject_into_raw_outside_unsafe`).
- Depends on: the ownership area's `owns` representation. The rest of UNS-1's list is done by its own
  family's area: `MaybeUninit` (area ARN), `Allocator` (area ALC), Ember `static mut` no longer unsafe
  (area STA).

**A5-5. `ember tcb` sections** (TCB-4)
- Files: `ember_driver/src/lib.rs:1719-1740, 2305-2530`.
- Size: S.
- Depends on: the FFI area's `unsafe import c`, `@unsafe(export…)`, `@ffi(no_callbacks)`,
  `@ffi(safe)` and `@ffi(const)`.

### Slice G: generators

**A5-6. A gen fn's `for` over its own collection walks by position** (CORO-6)
- Files: `ember_mir/src/lower.rs` (for a `for` over an Array, String, Map or Set inside a gen fn whose
  body yields: index in the frame, re-read and length check after each resume);
  `ember_analysis/src/coroutines.rs:574-612` (no E2221 for it; new message).
- Cost: the generator protocol, 1.18x of a plain C loop (`rules.md` section 2, AN-77).
- Size: M.
- Tests: R15 `run_a_generator_walks_a_list_by_position` (first half),
  `reject_a_view_of_the_frames_own_local_across_yield`, `reject_a_generator_walk_held_by_views_of_its_own_frame`.
  Old: rewrite `CORO-6/reject_a_borrow_of_a_local_across_a_yield`.
- Depends on: nothing.

**A5-7. A gen method's frame is a view of its object, and a generator is a walk** (CORO-12, CORO-14)
- Files: `ember_parser` (`-> Generator[…] from field`); `ember_analysis/src/coroutines.rs:540-550,
  613-624` (delete E2229); the walk analysis (a generator value counts as a walk of what it views until
  its last use: E3020 for a change to the walked collection, E3021 otherwise).
- Size: L.
- Tests: R15 `run_a_generator_walks_a_list_by_position` (the `Door.replay` half),
  `accept_a_generator_views_what_its_caller_keeps`, `accept_a_generator_yields_its_own_number`.
  Old: retire `CORO-12/reject_mut_self_and_a_held_field`.
- Depends on: area BRW's walk rule (BRW-18, verdict 95); the methods area's `this`; LT-1's `from`
  (lifetimes area).

### Slice T: thread facts and removals. Must land with the ownership words.

**A5-8. `Send` and `ThreadShared` worked out from the words** (THR-8, THR-9)
- Files: `ember_types/src/lib.rs:1757-1918` (replace `(send, sync)` with `(send, thread_shared)`; see
  shared decision 2); `ember_typeck/src/lib.rs:40784-40798` and the bound checks; `std` core marker
  `ThreadShared`; `Sync` in a bound is E1010; `unsafe extend … Send` stays (`:14011`).
- Class objects become structural over their fields, with no `is_sync`.
- Size: M.
- Tests: R15 `accept_thread_shared_as_a_bound`, `accept_a_class_with_atomic_fields_is_shared_without_a_mark`,
  `accept_send_written_by_an_unsafe_extend_author`, `reject_implementing_send_without_unsafe`,
  `reject_sync_as_a_bound`. Old: THR-8 respell 2 and rewrite 2; THR-9 respell 1.
- Depends on: the ownership area's type representation of `owns`, `shares`, `shares mut`,
  `thread_shared`, `weakref`, `sharedref mut`.

**A5-9. Remove `@sync`, `SyncShared` and the per-type atomic flag** (THR-1, THR-2; `remove` rows)
- Files:
  - `ember_typeck/src/lib.rs`: 6131-6240, 23790-23797, 44424, 52763-52778 (`sync_class_advice`),
    60332 (`@sync` becomes E0104 with the spec's message);
  - `ember_types` (`ClassDef.is_sync`);
  - `ember_codegen_c/src/lib.rs:4994, 5365`; `ember_codegen_llvm/src/glue.rs`;
  - `runtime/ember_rt` (`EMBER_TI_SYNC` replaced by shared decision 1; `ember_obj_check_thread`
    kept only for FFI `threads(creator)` trampolines);
  - `std/src/sync.em` (channels on `thread_shared`), `std/src/ecs.em:1206` (the ECS area rewrites it).
- Retire E7001, E7003, E7005 in `codes.rs` as the table says (`p17a-codes.md:297-299`).
- Size: M.
- Tests: retire the 12 `THR-1/` files; rewrite THR-2 2, THR-9 6, THR-13 2, THR-10 1; R15
  `reject_the_sync_class_mark`.
- Depends on: A5-8; area RC's atomic counts for `thread_shared`.

**A5-10. `thread_shared fn()`** (THR-1 third rule, verdict 61, verdict 94)
- Files: `ember_parser` (type `thread_shared fn(…) -> R`); `ember_types` (`TyKind::Fn.send` renamed
  `thread_shared`); `ember_typeck` (coercion `thread_shared fn()` to `fn()`, never the reverse; a
  lambda is `thread_shared` only when its captures are `thread_shared`, handed-over Send values or
  copies; E7015 naming the capture, and at the line making a `thread_shared` object whose class has a
  plain `fn()` field).
- Delete `callable_with_send` (`:40857`).
- Size: M.
- Tests: R12 4 (bus and handler tests); R15 `accept_a_thread_stores_a_handler_in_a_bus_threads_share`,
  `reject_a_handler_any_thread_may_run_holding_a_shares_mut_owner`,
  `reject_a_thread_takes_a_shares_mut_bus_to_store_a_handler`,
  `run_a_thread_runs_a_handler_holding_a_thread_shared_owner`.
- Depends on: A5-8; area CLO's capture lists (`fn() owns x, copy of y:`).

**A5-11. `thread_shared` values** (THR-1, THR-7)
- Files: `ember_typeck` (E7017 for `thread_shared` on a name not made so; E7014 for a direct change,
  except through `Atomic`, `Mutex`, `RwLock`, a channel end or a `guarded by` field under its lock;
  E7015 at the making line for whatever is reachable from it).
- Size: M.
- Tests: R3 2, R9 4 (`accept_a_thread_shared_object_filled_where_it_is_made`,
  `accept_a_thread_shared_owner_reaching_another_thread`, `reject_a_thread_shared_object_changed_directly`,
  `run_a_thread_shared_object_read_by_a_thread_and_its_maker`), R15 3.
- Depends on: A5-8, A5-10; the ownership area (the `thread_shared` value word, atomic counts);
  A5-16 for the `guarded by` exception.

### Slice S: spawning and scoped tasks

**A5-12. `thread.spawn` and `jobs.submit` take a `thread_shared fn()`** (THR-10, THR-17 unscoped part, JOB-2, THR-2)
- Files: `ember_typeck/src/lib.rs:52339-52376, 52513-52549, 52793-52832` (rewrite on A5-10: E7004 for
  a name with no word, E7015 for one-thread owners and lent views, E7015/E7008 when `R` is not Send,
  E7015 for an object handed over while a `weakref` on this thread may point at it, Q4);
  `std/src/thread.em:25`, `std/src/jobs.em:23` (verdict 94's signature, `[R]` line).
- A spawn inside a walk is E3020 by the thread's effects, except under the walk's lock.
- Size: M.
- Tests: about 16 R15/R12/R9 tests (for example `reject_a_thread_captures_a_one_thread_owner`,
  `reject_an_unscoped_thread_using_a_name_with_no_capture_word`,
  `reject_an_object_a_weakref_points_at_handed_to_another_thread`,
  `accept_an_unscoped_thread_takes_its_object_over`, `accept_a_thread_started_in_a_walk_under_the_lock_it_needs`).
  Old: THR-10 respell 3 and rewrite 1; JOB-2 respell 3.
- Depends on: A5-10, A5-11; area CLO's capture lists; area EFF's effect of a function value and of a
  spawn (EFF-26).

**A5-13. The loop's variable and round locals given to tasks and threads** (THR-17 loop part, verdicts 65 and 93)
- Files: `ember_typeck/src/lib.rs:51564-51617` and A5-12's code. Know the `for` pattern's names and
  the locals declared in the loop body. Scoped task: no word is E3026 (the message gives both words),
  `ref t` is allowed. Thread or job: no word is E7004 with `copy of t`, and `ref t` is E3026.
- Size: S.
- Tests: R15 `accept_a_scoped_task_looks_at_the_loops_text_with_ref`,
  `accept_a_task_takes_a_copy_of_a_round_local`, `accept_a_thread_takes_a_copy_of_the_loops_text`,
  `reject_a_task_uses_the_loop_name_without_a_copy`, `reject_a_round_local_used_by_a_task_without_a_copy`,
  `reject_a_thread_given_the_loops_text_with_ref`,
  `reject_a_thread_started_in_a_walk_uses_the_loop_name_without_a_copy`. Old: rewrite
  `THR-11/accept_scoped_tasks_borrow_the_block`.
- Depends on: A5-12; area CLO.

**A5-14. Scoped tasks checked by their effects, not by borrows** (THR-11, THR-13, JOB-2/JOB-3 scope part, THR-2 in tasks)
- Files:
  - remove the task-closure-into-`Scope` borrow flow (`ember_analysis/src/regions.rs:1131-1141`,
    `borrows.rs:220-247`);
  - a new pass per `with scope`/`jobs.scope` block that takes each task's EFF-24 summary and the
    parent's statements up to the join, and reports E7007 (other than reductions, a lock or an
    `Atomic`/`Mutex`) and E3020 for a task started in a walk of what it changes;
  - in a task: no `shares` owner made, `weakref` only to `thread_shared` targets (E7015); `R` Send;
  - `scoped_task_refused` at `ember_typeck/src/lib.rs:52726` reduced to "a task uses what it uses".
- Size: L.
- Tests: R15/R9 about 6 (`reject_a_task_reads_a_field_another_task_adds_to`,
  `reject_a_task_update_that_uses_the_old_value`, `reject_two_tasks_set_one_player_through_two_sharedref_links`,
  `accept_one_scoped_task_sets_a_player_through_a_sharedref_link`,
  `accept_a_task_started_after_the_walk_changes_the_list`, `reject_a_task_changes_the_list_its_parent_walks`).
  Old: THR-11 respell 3, JOB-3 respell 1, THR-13 rewrite 2.
- Depends on: area EFF's whole-program summaries with place paths and function-value targets (EFF-24);
  area BRW's walk sets (BRW-18); A5-8; Q2.

**A5-15. Reductions** (THR-21, verdicts 77 and 92)
- Files:
  - `ember_typeck`: detect the reduction forms on a local or on a field reached by every task;
  - `ember_mir/src/lower.rs`: a private accumulator per reduced place in each task's environment,
    folded into the place at the join in spawn order; reuse `reduction_identity`/`reduction_combiner`,
    `ember_typeck/src/lib.rs:52146-52220`;
  - `ember_codegen_c`, `ember_codegen_llvm/src/sync.rs:229`, `ember_interp` (fold at scope end and at
    `h.join()`);
  - E7007 for a read before the join and for `=`.
- Cost: nothing beyond C's per-thread counter (`rules.md` section 2, AN-71: 1.00 against 10.22x for an
  atomic add).
- Size: L.
- Tests: R15/R9 7 (`run_scoped_tasks_add_their_parts_to_a_total`, `run_tasks_push_their_parts_in_spawn_order`,
  `run_tasks_add_to_a_field_of_an_object_they_reach`, `accept_a_float_sum_added_once_per_task`,
  `run_a_float_sum_added_more_than_once_per_task_is_a_reduction`, `reject_a_task_reads_total_before_the_join`,
  `run_two_tasks_take_hp_off_one_player_through_two_sharedref_links`).
- Depends on: A5-14; Q3.

### Slice L: locks

**A5-16. `Lock` and `guarded by`** (THR-18, BRW-20, the verdict-89 exception of BRW-11)
- Files:
  - `ember_parser`: `guarded by name` after a field's or a `static mut`'s type; `with x.lock:` as a
    block form;
  - `std/src/sync.em`: a `Lock` struct with a runtime mutex and no value;
  - `ember_typeck`: E7013 (same object name, `this.lock`; `init` exempt; the thread holds it);
  - area EFF's summary gains "needs lock L held";
  - proven re-entry past a held lock (same name or `this` of the held object), otherwise E7018;
  - an object whose changing fields are all guarded, `Atomic`, `Mutex` or `RwLock` may reach one call
    through several names.
- Size: L.
- Tests: about 14 (R15 4 `reject_a_guarded_*`, `accept_a_task_reads_and_adds_to_a_guarded_field_under_its_lock`,
  `accept_init_sets_guarded_fields_without_the_lock`, `accept_tasks_change_stats_under_its_lock`,
  `accept_threads_share_a_log_and_push_under_its_lock`, `accept_a_direct_call_that_takes_the_held_lock_enters_past_it`,
  `accept_a_task_takes_the_lock_itself`, `run_two_threads_update_a_guarded_pair_under_one_lock`,
  `accept_one_call_reaches_a_lock_guarded_log_through_two_names`; R12
  `accept_add_line_changes_a_guarded_field_through_a_plain_parameter`; R4 1).
- Depends on: area EFF's summaries; area BRW (BRW-11, BRW-20).

**A5-17. Waiting for a lock this thread holds** (THR-19, THR-5's scope end, THR-10's join)
- Files: a pass over `with` lock blocks. E7018 for:
  - a call through a function value, `dyn` or virtual method whose possible targets take a lock of the
    held kind;
  - a direct call that may take it and is not proven to enter past it;
  - `h.join()` or a scope end whose task changes what the lock guards.
- Size: M.
- Tests: about 13 (R15 `reject_a_call_through_a_function_value_that_may_take_the_held_lock`,
  `reject_a_function_value_that_changes_another_log_inside_the_lock`,
  `reject_a_join_inside_the_lock_its_thread_needs`, `reject_a_scope_that_ends_inside_the_lock_its_task_needs`,
  `reject_a_walk_under_the_lock_across_a_function_value_that_changes_log`,
  `reject_running_handlers_that_change_their_bus_under_its_lock`, and the matching accepts; R12 2).
- Depends on: A5-16, A5-12, A5-14; area EFF/CLO's function-value targets.

**A5-18. Module variables and threads** (THR-20)
- Files: a whole-program set of functions that may run on another thread: reachable from
  `thread.spawn`, an unscoped `jobs.submit`, any-thread callbacks and `threads=any|creator` exports,
  plus the `drop` of `thread_shared` values those functions hold counts of. E7012 for a `static mut`
  reached from one, unless it is guarded by a static `Lock` and used under it. A `static` of a type
  that changes only through itself is usable everywhere.
- Size: M.
- Tests: R15 4 (`reject_a_thread_changes_an_unguarded_static_mut`,
  `reject_an_any_thread_export_changes_a_static_mut`, `accept_guarded_statics_changed_by_a_thread`,
  `accept_an_any_thread_export_changes_a_guarded_static`) plus `accept_a_scoped_task_changes_an_unguarded_static_mut`.
- Depends on: area STA (Ember `static mut` does not exist today, `ember_typeck/src/lib.rs:10259`);
  A5-16; area EFF's call graph; area FFI's export thread policies.

### Slice P: library and parallel loops

**A5-19. Lock types, channels and `@must_drop` containers** (THR-3, THR-5 storage, THR-6, THR-15)
- Files:
  - `ember_typeck`: a guard sent to another thread is E7008; `ref mut` before a lock-type argument is
    E3028; changes through plain names; a `Scope` in a function value's state is E3014; `@must_drop`
    in a `shares`/`thread_shared` value, an `owns` block or a capture is E3015 (`:6145-6182`,
    `ember_types/src/lib.rs:2307-2330`);
  - `std/src/sync.em`: `copy of tx`; the Receiver's drop takes queued values out under the lock and
    ends them after letting it go (`:255-262`).
- Size: M.
- Tests: R15 `reject_a_lock_guard_sent_to_a_task`, `reject_ref_mut_before_a_channel_end_at_a_call`,
  `run_a_producer_thread_sends_and_a_consumer_thread_receives`. Old: THR-3 respell 1, THR-5 respell 1,
  THR-6 respell 1 and rewrite 1, THR-15 respell 4.
- Depends on: A5-8; the ownership area.

**A5-20. `@parallel` on the new words** (PAR-1, PAR-2)
- Files: `ember_typeck/src/lib.rs:51922-52331`:
  - the element walk with the loop's `ref mut` name, E7016;
  - places reached through links are refused, `Array[shares mut T]` is refused;
  - captures judged by A5-8 (E7015 replaces E7005);
  - fix-line messages;
  - the backends' `ParallelFor` gets an element-chunk form.
- Size: M.
- Tests: R15 5 (`accept_a_parallel_loop_over_a_range`, `accept_a_parallel_loop_over_one_owner_objects`,
  `reject_a_parallel_loop_over_shares_mut_objects`, `reject_a_parallel_loop_writing_at_another_index`,
  `reject_a_parallel_while_loop`). Old: PAR-1 respell 1, PAR-2 respell 3.
- Depends on: A5-8; the ownership area.

**A5-21. Error codes and pages** (cross-cutting; landed with each item)
- New: E7007, E7008, E7012, E7013, E7014, E7015, E7016, E7017, E7018, W3013.
- Retired: E7001, E7003, E7005, E2229 (`p17a-codes.md:284-299`; E7020 is the ECS area's).
- Changed meaning: E7004 (no capture word), E3014, E3026, E3105, E7002 (`ThreadShared`).
- Files: `ember_diag/src/codes.rs:395-403, 446`; `docs/errors/E70xx.md` (11 missing pages).
- Size: S.
- Depends on: the items that emit each code.

**A5-22. `std.gpu`** (GPU-2, 3, 6, 7, 9, 12)
- Files: a new `std/src/gpu.em` (Device interface, Frame, handles, CommandList with the
  bound-buffer compares, graph); the runtime for robust access and the compares; `ember tcb`'s
  asserted fact.
- Size: L.
- Depends on: the ownership area (`owns frame`); area ALC (`using`); area HND (handles); area FFI. Not
  needed by any other item.

**Smallest consistent slices, in order:**
1. U (A5-1, A5-2, A5-3) at once.
2. G1 (A5-6).
3. T (A5-8 to A5-11, with the ownership area's words and RC's atomic counts), together with A5-19
   and A5-20 so that nothing still uses `Sync`, `@sync` or `SyncShared`.
4. S (A5-12, A5-13; then A5-14 and A5-15 together, since removing the borrow model needs E7007 and
   reductions the same day).
5. L (A5-16, A5-17, A5-18).
6. A5-4, A5-5, A5-7 when their other areas land.
7. A5-22 last.

---

## 3. Shared decisions (proposed)

**1. Atomic or plain counts are chosen when compiling, by the word in the type, with one header bit as
a fallback.**
- Today the runtime tests `ti->flags & EMBER_TI_SYNC` on *every* retain and release
  (`runtime/ember_rt/include/ember_rt.h:1933-1947`): a load and a branch per count change, even for
  one-thread objects.
- Proposal: in 0.9.15 the type says it. `thread_shared T` and a `weakref` link to one emit
  `lock xadd` plus the limit compare (as Rust's `Arc`). `shares`/`shares mut` and every other
  object's weak count emit a plain add plus compare (as `Rc`). No run-time branch.
- One header bit, set when an object is made `thread_shared`, serves only the paths where the static
  type cannot say:
  - a method making an owner or `weakref` of `this` (a class's methods serve both kinds of object);
  - the leak report;
  - the opened view of a `weakref` (THR-20's "holds one for its life").
- Cost: the plain path is exactly `Rc`'s. A method that makes a link to `this` pays one
  well-predicted branch.
- The alternative, compiling two copies of each such method, costs code size for a rare path.
- The bit's place in the 8-byte header is the RC area's call.

**2. Thread properties are `(send, thread_shared)`, worked out structurally from the words.**

| type | send | thread_shared |
|---|---|---|
| `shares T`, `shares mut T` | no | no |
| `weakref` of a non-`thread_shared` target | no | no |
| `thread_shared T` and its `weakref` | `T` send and TS | `T` send and TS |
| `owns T`, a struct, enum, tuple, collection, or a class object owned once | all components send | all components TS |
| `ref T`, `Span` | `T` TS | `T` TS |
| `ref mut T`, `MutSpan` | `T` send | no |
| `thread_shared fn()` | yes | — |
| plain `fn()` with state | no | — |
| guards, `Scope`, raw pointers, `threads(creator)` handles | no | no |
| `UnsafeCell`, `Cell` | `T` send | no |
| `Mutex[T]` | `T` send | `T` send |
| `RwLock[T]` | `T` send | `T` send and TS |
| `Atomic`, `Once`, `Lock`, `Sender` | yes | yes |

This replaces `ember_types/src/lib.rs:1777-1918` and drops `ClassDef.is_sync`. The table costs
nothing at run time.

**3. `thread_shared fn()` is the existing `TyKind::Fn.send` bit, renamed.** A `thread_shared fn()`
coerces to `fn()`, never the reverse. A lambda gets the bit only from its capture list.

**4. A task's effect set.** Area EFF's summary (EFF-24) holds entries `(place path, kind)` where kind
is one of:
- read;
- change;
- reduce(op), the reduction-shaped changes of THR-21;
- under(lock path or lock kind).

Place paths are rooted at a caller local, a static, or a field through a link. The scope check
(A5-14) compares the tasks' sets pairwise and against the parent's statements up to the join.
Without the `reduce` and `under` kinds, A5-14 and A5-15 cannot be built on EFF's summary. This must
be agreed with area EFF before either side builds.

**5. Lock identity.**
- Statically, a lock is a path: an object path plus the `Lock` field, or a static `Lock`.
- For function values, `dyn` and virtual calls (THR-19), it is a kind: (class, lock field).
- Summaries carry `takes(kind)` and `needs_held(path)`.

**6. "May run on another thread"** is one whole-program set, computed once with the call graph and
function-value targets. A5-18 (E7012), the FFI area's callbacks and A5-12 all use it.

**7. Reductions.**
- Each task's environment holds one accumulator per reduced place, initialised to the operator's
  identity; a private list for `push`/`extend`/`add`.
- At the join, in spawn order, the parent applies `place op= acc`.
- The fold reuses `@parallel`'s `reduction_combiner`.
- Integer overflow handling waits on Q3.

**8. Generator walks by position** keep `(collection place, index)` in the frame and re-read both
after each resume (A5-6). The collection place is the frame's own field, or the object's field for a
method.

---

## 4. Questions for the owner

**Q1. E3100 or E5002 for a foreign call and a foreign `static mut`.**
- `[UNS-1]` lists "call an `extern` function without a verified contract" and "access a foreign
  `static mut`", and says: "Each of these outside an `unsafe` block is `E3100`".
- The code table keeps "`E5002` | foreign call requires `unsafe`: its contract is incomplete…"
  (`p17a-codes.md:163`).
- The compiler gives E5002 for both (`ember_typeck/src/lib.rs:22786-22804`, `:25834`).
- Which code does each get? This blocks the UNS-1 and FFI tests' expectations and the registry.

**Q2. Does an early `h.join()` end a task's claims?**
- `[THR-11]` checks a task "against the other tasks and the parent until the join", and
  `[THR-13]` says "neither the parent nor another task changes it until the join".
- "the join" can mean the task's own `h.join()` or the block's end.
- Example (`tests/conformance/THR-11/reject_writing_what_a_task_reads.em`): `h = scope.spawn(fn():
  read(counts))`, `h.join()`, then `counts[0] = 5` in the same block. It is refused today. Rust also
  refuses it, because the borrow lasts for the scope.
- Is it E7007, or allowed? This blocks A5-14's end point for each task, and when reductions fold.

**Q3. Overflow inside a reduction.**
- `[THR-21]` says "A whole-number total is always the one adding one by one gives".
- `[TYP-8]` panics on any overflowing `+=`.
- A task's private accumulator starts at 0, not at the place's value. So it can overflow where adding
  one by one would not: the place is −10, and a task adds `MAX` then `1`. The reverse can happen too.
- Proposal: accumulators wrap, and the fold at the join is checked in a doubled width (i128 for i64).
  The program then panics exactly when the true total does not fit. That costs one extra add-with-carry
  per reduced step.
- The alternative is to check each step in the accumulator, which can panic on totals that fit.
- Which does the owner want? This blocks A5-15's code generation.

**Q4. Handing over an object while a `weakref` link on this thread points at it.**
- `[THR-2]` makes this E7015 when compiling.
- A link to the object may have been made anywhere: stored in a registry, or passed to a function that
  kept it. A compile-time check is exact only for links made and still live in the same function.
- Proposal: refuse the hand-over when a whole-program analysis finds that a non-`thread_shared`
  `weakref` to an object of that class may be live on this thread. This may refuse programs whose
  link has in fact ended.
- The alternative is one compare of the weak count at the hand-over. That is a run-time check, but on
  data the program computes, as count limits are.
- Is the conservative compile-time refusal what the owner wants? It is buildable, but its
  false-refusal rate is unknown.

---

## 5. Risks

- **Almost everything waits on other areas.** A5-12 to A5-18 need area EFF's whole-program effect
  summaries (with place paths, reduction and lock kinds), area CLO's capture lists, and the ownership
  area's type words. Only Slice U and A5-6 can start now.
- **The scoped-task model is the borrow checker today.** Task closures flow into the `Scope`
  (`regions.rs:1131-1141`), and their captures are treated as borrows until the block ends
  (`borrows.rs:220-247`). Removing that without E7007 and reductions in place would let races
  through. A5-14 and A5-15 must land together.
- **`ember_typeck/src/lib.rs` is one 62k-line file.** The thread checks are spread through it
  (`owned_task_refused`, `scoped_task_refused`, `sync_class_advice`, the `@sync` validators, the
  must-drop checks), mixed with closure machinery (`closure_calls`, `owned` closures, `lend_named_callable`).
- **Ember `static mut` does not exist** (`ember_typeck/src/lib.rs:10259-10264`). THR-20 and the
  guarded statics of THR-18 are blocked on area STA building it.
- **The runtime's per-type `EMBER_TI_SYNC` and the creator-thread check** (`ember_rt.c:3119`) run
  today. Both go, except for FFI trampolines. The 8-byte header redesign (RC area) is a precondition.
- **PAR-2 against THR-21.** PAR-2 says a `@parallel` loop's outside uses "follow the rules of a
  scoped task", which have automatic reductions. Its clause (c) and PAR-3 still require *declared*
  reductions. I read the loop as needing them declared: E7010 for an undeclared `seen += x`, as the
  old test expects. I did not raise this as a question because PAR-2 (c) is explicit.
- **GPU (Annex D)** has no code at all. A5-22 is a whole library, and its size is a guess.
- **Not verified:** the LLVM backend's concurrency lowering (`ember_codegen_llvm/src/sync.rs`) and the
  evaluator (`ember_interp/src/builtin.rs`) were only located, not read. The exact new message
  substrings for the respelled tests are inferred from the spec's examples. Where a test's new code is
  between E7015 and E7008 (a non-Send `dyn` sent to a thread), I picked E7015 by the code table's
  wording.
- **Old-test convention:** 31 keep, 43 respell, 18 rewrite, 13 retire, all read by hand. Tests in other
  families' folders that use `@sync` (37 files) or `SyncShared` (7) are counted only where they are in
  my folders.
