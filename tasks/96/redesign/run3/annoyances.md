# 05 builder: the annoyances of today's Ember, each given to one design area (run 3, phase 1)

The full list of what is annoying in today's Ember, gathered from phase 1
(`tasks/96/redesign/phase1-borrowing.md`, `phase1-sharing.md`), `FINDINGS.md` (section 5),
`FINDINGS-phase3b.md` (section 5), the five `phase3b-<area>.md` reports, and this run's four readers
(`RUN3/01-reader-1.md` .. `RUN3/04-reader-4.md`). Each annoyance has an id, a one-line title, the
program's intent in one line, where it was found (file and line), exactly one design area, and whether
an agreed row already settles it.

Where-found paths: `p1b` = `tasks/96/redesign/phase1-borrowing.md`, `p1s` = `phase1-sharing.md`,
`F` = `FINDINGS.md`, `F3b` = `FINDINGS-phase3b.md`, `3b-cont` / `3b-refs` / `3b-own` / `3b-graph` /
`3b-thr` = `phase3b-containers.md` / `-references.md` / `-ownership.md` / `-graphs-callbacks.md` /
`-threads.md` (all in `tasks/96/redesign/`), `r1`..`r4` = `RUN3/01-reader-1.md`..`RUN3/04-reader-4.md`,
`05` = `target/redesign3/05-builder/check_output.txt` (this agent's probes, outputs pasted in
`RUN3/vocabulary-draft.md` section 5). The phase-1 id (A1..A34) is given where there is one.

**Areas.** A lists, loops, temporary values. B references kept or returned, locks, objects still being
built. C ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines. D linked
objects, callbacks, interface types. E threads, locks across threads, `unsafe`.

**Agreed rows, given as the duty says.** Row 5 and "delete after the loop" -> A; rows 3, 4, 7 -> B;
rows 8, 9 and `owned` replacing `Box` -> C; row 6 -> D. "Settled" = yes when an agreed row (or the
owner's ruling that `owned` replaces `Box`, or the owner's "delete after the loop" note) already decides
the answer; the area still writes the answer in the eight-part form and the error message.

**What counts as an annoyance here.** A program a Python programmer would write that today's Ember
refuses, or makes harder to write than Rust, or accepts while doing something the words do not say;
a word that does not say what it does; a refusal whose message does not give the line to write; and
the costs last run's model could not remove (marked "last run's limit"). Defects that are plain
compiler bugs are listed where a programmer meets them as an annoyance.

## 1. The list

### Area A: lists, loops, temporary values (16)

| id | title | intent (one line) | found | area | settled by an agreed row |
|---|---|---|---|---|---|
| AN-01 | Holding an element while adding to the list (A2) | `split_boss` halves the first enemy's hp and adds a minion with the other half | p1b:161-228 (`e02` E3022; the Python spelling `boss = enemies[0]` is E3040 where `[EXP-6]` says E3011, p1b:47) | A | no |
| AN-02 | A struct element's name used after a call that may change its list is refused (last run's limit) | keep updating `enemies[i]` after calling a helper that may add enemies | F3b:89 | A | no |
| AN-03 | Two elements of one list in one call (A3) | `collide(bodies[i], bodies[j])` changes the speeds of two bodies | p1b:230-285 (`e03` E3022) | A | no |
| AN-04 | Changing a list while looping over it (A4) | remove dead enemies and add their splits during the update loop | p1b:287-357 (`e04` E3020) | A | **yes**: "delete after the loop" (JOB): an understandable limitation; the queued delete stays and the message must name it |
| AN-05 | A loop that removes from its own list while calling code the compiler cannot see (last run's limit) | the loop calls a helper that may also remove from the list | F3b:88 | A | **yes**: "delete after the loop" (the list may not change while a loop walks it) |
| AN-06 | ECS: creating and destroying entities inside a view loop | a system destroys dead entities and spawns new ones while iterating a view | 3b-cont:671-798; F3b:84 (group swaps made 39,602 wrong visits in 5,000 loops) | A | **yes**: "delete after the loop" names Bevy's `commands.entity(e).despawn()` |
| AN-07 | A local list of string views (`words = text.split()`) is refused (A25) | split a line into words and count them, without copying each word | p1b:952-986 (`e06e` E3063); `BorrowList` not implemented, p1b:55 | A | no |
| AN-08 | A view of a temporary kept past its statement (A22) | `word = line.to_lower().trim()` then use `word.len()` | p1b:811-860 (`e22b` E3060); r4:256-275 | A | **yes**: row 5 |
| AN-09 | Returning a view of a temporary: the message names `_5`, and the agreed advice `String(word)` is refused today | return the cleaned word from `clean(line)` | r4:256-275 (E3060 with `_5`); r4:618-627 and r3:55 (`String(word)` is E2020 "`String()` takes no arguments") | A | **yes**: row 5 (the constructor `String(x)` must exist for its message) |
| AN-10 | A `mut self` method on a temporary (`line.split(" ").next()`) is E2140 with no help; Rust accepts it | take the first word of a line in one expression | p1b:837; 3b-refs:1185 | A | no |
| AN-11 | A `MutSpan` parameter cannot be written without also writing `mut`; the spec's own `[BCK-6]` and `@parallel` examples are refused; E3023's help offers `RefCell` and `class` and writes `i64` for `int` | `integrate(pos, vel, dt)` adds each velocity to its position in place | r4:81-106 (t20, t18, t21); r3:29-33, 205-228 (t1, t2); r2:99-102 | A | no |
| AN-12 | Updating a map entry in place (A21): `counts.get(w).unwrap_or(0)` is E2020, there is no `get(k, default)` | count the words of a text in a map | p1b:778-809 (`e21`, `e21b`) | A | no |
| AN-13 | `SoA.get(i)` silently copies, while `Array.get(i)` gives a reference | read particle `i` from column storage | r3:48-50, 267-279 (t6) | A | no |
| AN-14 | `assert_disjoint` returns a `Result` and never stops the program | check that two views do not overlap before writing both | r2:90-92 (spec 3145-3174) | A | no |
| AN-15 | The ECS query marker `Mut[T]` is a second spelling of `ref mut T` | a system asks for changeable positions and read-only velocities | r3:191 (spec 3787-3838) | A | no |
| AN-16 | Messages use the compiler's names, not the program's (`_5`, `_14`, `i64` for `int`, `*self.items`, `Callable0`), and many helps do not give the line to write (A17) | every refusal names what was refused in the program's own names and the line to write | r4:641-642, r4:245; 3b-refs:1183; 3b-graph:1147-1150; p1b:681-723 (A17) | A | no |

### Area B: references kept or returned, locks, objects still being built (17)

| id | title | intent (one line) | found | area | settled by an agreed row |
|---|---|---|---|---|---|
| AN-17 | A method changes one field while a loop walks another (A1) | `tick` walks `self.players` and calls `self.note(...)`, which only appends to `self.log` | p1b:73-159 (`e01d` E3020; a `pub` method of another package borrows all of `self`, 3b-own:1013); r4:229-248 (t06, classified B2 instead of B8) | B | **yes**: row 3 |
| AN-18 | `mut self` on a class method means "hold a write access on every field for the whole call", not "may change" | a class method says that it changes the object | r2:44-53 (spec `[CLS-7]` 1740); r1:29 | B | **yes**: row 3 (the method's first line says what it changes) |
| AN-19 | Plain `self` in a class method changes fields with no word | `fn open(self): self.angle = 90.0` opens a door | r1:30, 319-321 (t5); r2:54-61 (t4) | B | **yes**: row 3 (what a method changes is written on its first line and checked against its body) |
| AN-20 | A plain parameter, a `for` name or a `Cell` field changes a class object with no word on the name (`fn bump(c: Counter): c.n += 1`; `for p in players: p.health = 100.0`; `s.frame.set(...)`); `[FN-1]` says the default "cannot assign through it" | heal every player in a list, or bump a counter passed in | r2:54-61, 392-394 (t8, t11); r1:30 (t5); r2:78-84 | B | no |
| AN-21 | Caching inside a read-only method (A31): the struct form is E3023 with `RefCell`/`class` help | `radius()` computes the mesh radius once and keeps it | p1s:904-978 (`a31_lazy_cache.em`) | B | no |
| AN-22 | Returning references (A5): a result borrows all of `self` unless `@borrows(self.src)` is written on its own line above; the ECS pool `return ref mut pool.dense[i]` is E3060 where Rust accepts | `next_word` returns a word of the source; `get_mut[T]` returns a component in its pool | p1b:359-448 (`e06b` E3025, `ecs5/pos.em` E3060); r2:85-89 (`@borrows` off the signature) | B | no |
| AN-23 | A call that returns `ref mut` is not a place: `first(xs) += 1` and `counts.entry(k).or_default() += 1` are E2140 | add 1 to the element a function hands back | 05 (`b1_ref_mut_forms.em`); p1b:50; F:150 | B | no |
| AN-24 | Structs holding references (A6): a view struct cannot be stored in an `Array` | keep a list of handles that each refer into one registry | p1b:450-494 (`e06d` E3063) | B | no |
| AN-25 | Self-referential data (A7): a value owning its text and the word views into it | a `Document` holds its text and the list of its words | p1b:496-550 (`e07` E3063; E3024) | B | no |
| AN-26 | `str` is a borrowed view while Python's `str` is the owned string: `name: str` in a class compiles for a literal and is refused for a `String`; `-> str` hides what the result points into | keep a player's name in the player | r1:31, 328-341 (t8, t8b E3063); r2:93-98 (t12); r4:73 (P26) | B | no |
| AN-27 | `ref T` / `ref mut T` as a parameter's type: `x: ref int` means the same as `x: int`; `x: ref mut int` cannot be written and its help (`mut x: ref mut i64`) does not compile | `bump(n)` adds 1 to the caller's number | r1:36, 249-303 (t3, t3c, t6) | B | no |
| AN-28 | `r = ref mut b` on a `ref mut` local is accepted and copies `b` into `r`'s first target; `r = e` writes through where Python's `r = e` rebinds | keep a changeable reference and later point it at another variable | 05 (`b2_ref_mut_rebind.em`, exit 0); r2:123 (spec `[BRW-1]` 2286) | B | no |
| AN-29 | `ref String` is not accepted where `str` is expected at a call (E2020) | pass a list's string element to a function that reads text | 3b-thr:1245 | B | no |
| AN-30 | A reference out of a lock, `RefCell` or `RwLock` guard compiles and is unsafe (use after free, data race) (A33) | `top(scores)` returns the best score from a locked table | p1s:1054-1132 (`a33_lock*.em`, `a09_refcell_escape.em`); 3b-thr:1230; r4:250-254 (t07); F:147 | B | **yes**: row 4 |
| AN-31 | Initialisation (A18): a helper called in `init` before every field is set is E2100 with no help | `init` sets the renderer, then loads assets with a helper that only reads the renderer | p1b:725-776 (`e18`, `e18b`); r4:277-290 (t09) | B | **yes**: row 7 |
| AN-32 | `mut self` on `init` is accepted and means the same as `self` | write a constructor | r2:51-53 (t4; spec 1658, 1717) | B | no |
| AN-33 | A `let` field still changes through `push`; E2104 (assigning it) has no help | a log whose list is set once in `init` and never replaced | r2:62-67, 366-374 (t2, t2b); r1:32; r4:74 | B | no |

### Area C: ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines (19)

| id | title | intent (one line) | found | area | settled by an agreed row |
|---|---|---|---|---|---|
| AN-34 | Moves (A11): `snapshot = items` hands the list over; using `items` after is E3040, whose help says "clone" | keep a snapshot of a list before changing it | p1b:621-679 (`e11` E3040); r4:311-329 (t02) | C | **yes**: row 9 |
| AN-35 | Partial move and move in a loop: `n = p.name` then `keep(p)` is E3042; `v.push(template)` in a loop is E3041 | log a player's name and keep the player; fill a list with copies of a template | p1b:625-650 (`e11` E3042, E3041) | C | no |
| AN-36 | A write to a handed-over value is accepted and lost: `root.children.push(child); child.value = 5` prints the old value | add a child to a tree, then set its value | 3b-own:1001-1008 (`a11_store_then_use.em`) | C | **yes**: row 9 (a handed-over name may not be used; the refusal must also cover a field write) |
| AN-37 | Moving a field out from behind `mut self` (A23): `match owned self.state` is E3013; `mem.replace` needed | `Npc.update` shortens the walk path and goes idle when it is empty | p1b:862-916 (`e23` E3013) | C | **yes**: row 8 (`State.Walking(ref mut path):` shortens in place) |
| AN-38 | `State.Walking(path):` then `path.pop()` is E3021 with a `.clone()` help; `State.Walking(mut path):` is accepted with `path` still read-only | the same NPC update, written the obvious way | r1:208-222 (t1); r4:292-309 (t03, t04, t05); 3b-refs:1188 | C | **yes**: row 8 |
| AN-39 | Choosing among ownership kinds (A10): eleven mechanisms (`struct`, `Box`, `class`, `Shared`, `SyncShared`, `Weak`, arenas, `Pool`+`Handle`, `Cell`/`RefCell`, `Mutex`/`RwLock`/`Atomic`) | two entities share one mesh and one of them edits it | p1s:153-247 (`a10_ownership_kinds.em` E3040); F3b:90 ("one value in two owning places needs `copy(x)` or a `class`") | C | no (its `Box` part is AN-40) |
| AN-40 | `Box[T]` says nothing about one owner; `ForeignBox[T]` carries the same word | an entity owns its mesh alone; a pool table owns its pools | r1:33, 190-202; r2:74-77; r3:51-52; r4:67-68 | C | **yes**: the owner's ruling, `owned` replaces `Box` (`mesh: owned Mesh`) |
| AN-41 | A type that contains itself (`next: Option[Node]`, `left: Option[Tree]`, `parent: Option[ref Node]`) crashes the checker instead of naming the fix | a linked list or a binary tree of values | r4:609-616 (t15, exit 127); 3b-own:1009-1012; p1b:45-46; F:148 | C | no (the fix's spelling, `Option[owned Tree]`, follows the `owned` ruling) |
| AN-42 | `struct`, `Box` or `class` decides what compiles and what panics (A24); a test that must panic is marked run-pass | the same ECS registry written with class pools or with boxed pools | p1b:918-950; F:155 | C | no |
| AN-43 | `owned` on a parameter means three things by type: a move (`Array`), a copy (`int`), a retained copy the caller keeps (class handle, `[FN-9]`, while `[FN-1]` says "without a retain"); `mem.drop(owned x)` moves a handle | `take(x)` takes over what it is given | r2:27-43, 322-348 (t6, t6b, t7, t9) | C | no |
| AN-44 | `b = a` has no word and does three things by type: copy (`int`), hand over (list), share (class handle); `Copy` covers class handles, whose copy is a share | `p2 = player` and `snapshot = items` in one program | r1:44-47; r4:120; r2:125-126 | C | no (row 9 settles the list case only) |
| AN-45 | `.clone()` on a class handle shares the object instead of copying it | make an independent copy of a player object and change it | r2:68-73, 389-390 (t5, t15) | C | no |
| AN-46 | Silent copies no word names: `comptime(e)` of a heap value is cloned at each run-time use; `sorted(m.keys())` clones the keys | use a constant table; print a map's keys in order | r3:321-324 (spec 4038-4040, 4155, 4322) | C | no |
| AN-47 | `owned` in an expression: E0109 says it is never allowed, Annex A writes `task = owned fn() => ...` and it is accepted; E0109's help for `b = owned a` offers an `owned` parameter where `b = a` already hands over | hand a value to a new name | r4:643-645 (t01, t11) | C | no |
| AN-48 | Shared mutable state (A9): a camera and a HUD both change one player; the struct form is E3022, the class form is left to run-time access checks that panic | the camera and the HUD both change the player | p1s:37-151 (`a09_shared_state.em`, `a09_shared_state_struct.em`) | C | no |
| AN-49 | `RefCell` (and `Ref`, `RefMut`, `borrow`, `borrow_mut`) hides a run-time check that panics; E3023 offers it to a beginner who only forgot `mut` | change a value reached through a shared path | r1:38; r3:173; r4:72; r2:78-84 | C | no |
| AN-50 | "handle" names a class object's counted pointer in prose and messages, and also `Handle[T]`, a checked pool index | read a message about a stale handle and know which kind it means | r1:40; r3:59-62; r4:203 | C | no |
| AN-51 | A count pair the spec allows to remove is emitted every loop step (13.5% to 13.9% of the workload) | read each watcher's target in a loop | 3b-own:1015-1018 | C | no |
| AN-52 | Shared class objects that are re-pointed often ran at 1.16x to 1.17x plain C in last run's model, with no fix found (last run's limit) | game objects keep re-pointing their targets | F3b:92 | C | no |

### Area D: linked objects, callbacks, interface types (18)

| id | title | intent (one line) | found | area | settled by an agreed row |
|---|---|---|---|---|---|
| AN-53 | A closure that changes a variable of the function that made it (A12): a plain lambda is E3026; the suggested `owned fn` changes a copy, silently | the button adds 1 to `game`'s score | p1s:251-366 (`a12_closures.em`, `a12_closures_owned.em`); F:43 | D | **yes**: row 6 |
| AN-54 | `owned fn` moves a `String` capture and copies an `int` one; `[DIA-10]` holds it up as a model help | a callback that uses a name and a score | r1:28, 224-247 (t2, t2b); r2:29-34, 303-320 (t1, t10); r3:22-28, 230-249 (t3, t8); r4:66, 224-225 | D | **yes**: row 6 |
| AN-55 | The compiler decides a capture is moved from which function receives the lambda (`[CLO-15]`), not from what is written | `thread.spawn(fn() => work(x))` and adapter calls | r2:119-122; r3:325-326 | D | **yes**: row 6 (a capture is the real variable unless named) |
| AN-56 | A lambda parameter written without a mode takes `owned` from the expected callable type (`[TYP-23]`, ODR-025) | pass `fn(x) => ...` where a callable is expected | r1:48-50 (spec 1468) | D | no |
| AN-57 | E3026 is reported inside the callee, not where the lambda is written; a lambda passed to a plain callable parameter that is stored gets E2020 naming `Callable0` | find which call wrote the bad lambda | p1s:1189-1192; 3b-graph:1147-1150; F:153 | D | no |
| AN-58 | A closure stored in a local list that ends first is still E3026 | collect a few local callbacks in a list and run them | 3b-graph:1154-1155 (`a12_local_list.em`) | D | no |
| AN-59 | A closure that gives a capture away, passed where it may run many times, gets E3013 with ten options instead of "declare the parameter `once fn()`" | `call_twice(fn() => consume(s))` | r4:629-639 (t13); r4:437 | D | no |
| AN-60 | A callback receiving the object that holds it, and a handler that removes itself (A32): `for h in self.handlers: h(self)` is E3021, its help leads to E3080 | the UI calls each handler with the UI; a one-shot handler removes itself | p1s:980-1050 (`a32_reentrant_callback.em`); 3b-graph:1151-1153 | D | no |
| AN-61 | Unsound today: a handler list dispatched by index (`h = self.handlers.get_mut(i).unwrap(); h(self)`) is accepted and writes freed memory when a handler adds or removes handlers | dispatch handlers by index so each may change the list | 3b-graph:1124-1141 | D | no |
| AN-62 | A loop that calls a stored callback whose effects are unknown pays a count pair per step, 1.27x to 1.32x plain C, in last run's model (last run's limit) | call every stored handler each frame | F3b:93 | D | no |
| AN-63 | Trees with parent pointers, graphs, cycles (A8): `Weak` and `upgrade` at every use; the Python spelling (strong `parent`) compiles and leaks; owning cycles leak | a scene tree whose nodes know their parent | p1b:552-619 (`e08b`, `e08c`); 3b-graph:1145-1146; F3b:91 | D | no |
| AN-64 | L3001 tells the programmer to make the owning edge (`children`) weak | follow the warning to fix a tree's cycle | p1b:53-54, 576-582; 3b-graph:1142-1144; F:151 | D | no |
| AN-65 | `upgrade` is Rust's word for "the object, if it is still alive" | read a weak link's target | r2:107-108 (spec 2604, 2660, 2810) | D | no |
| AN-66 | A bare interface type (`s: Shape`) is a counted, shared, dynamically dispatched handle with no word, while one owner needs `Box[dyn I]` | store shapes of several kinds and draw them | r2:112-115 (spec `[OBJ-2]` 2631-2634); owner's `no_share(dyn Pool)` (JOB) | D | no (one owner is `owned dyn I` by the `owned` ruling; the shared case has no word) |
| AN-67 | Trait objects and downcasting (A13): `cast` is refused on `Box[dyn AnyPool]`; a reference into a downcast pool is E3060; a generic method makes an interface not `dyn`-compatible (E2050); the spec writes `as?`/`as!`, which the compiler refuses for `cast` | `get_mut[T]` finds the pool of `T` in the registry | p1s:368-452 (`a13_dyn.em`); 05 (`d1_cast_forms.em`) | D | no |
| AN-68 | `if c = cast s if Circle:` gets E2036 "this pattern always matches" with a help that is not what was meant | do something only when a shape is a circle | 3b-graph:1156-1158 | D | no |
| AN-69 | Generics friction (A16): the orphan rule (E2121), bounds written by hand (E2040), no specialisation (E2041); "the most specific wins" unproven | print a list of vectors; sum any addable values | p1s:650-738 (`a16_generics.em`); F3b:95 | D | no |
| AN-70 | `virtual` in a class that cannot be subclassed is accepted with a warning that it does nothing (W2111) | declare an overridable method | r4:561 (t10) | D | no |

### Area E: threads, locks across threads, `unsafe` (14)

| id | title | intent (one line) | found | area | settled by an agreed row |
|---|---|---|---|---|---|
| AN-71 | Threads (A15): a task cannot capture the loop variable `t` (E3021, E3060), `total += ...` from tasks is E3022, E7004 offers `@sync` | count the words of several texts on several threads | p1s:555-648 (`a15_threads.em`) | E | no |
| AN-72 | Errors land on the wrong variable: `t`'s E3021/E3060 in `a15_threads.em` are caused by `total` | read the error and fix the right line | 3b-thr:1234-1237 | E | no |
| AN-73 | "cannot find `n`" (E1010) inside a lambda that calls an `Atomic` method | `fn() => total.fetch_add(n)` | p1s:1186-1188; 3b-thr:1238-1240; F:154 | E | no |
| AN-74 | A `@sync` class freezes every field after `init` (E7003), so counters become `Atomic`/`Mutex`; in last run's model a class object written by several tasks needs a lock or a "claim" at 1.12x to 1.36x | several tasks update one statistics object | r3:253-265 (t5), r3:328; F3b:94 | E | no |
| AN-75 | Clone-before-move for shared handles (A34): absent today only because `owned fn` retains each capture; it can come back once `owned fn` goes (row 6) | two threads append to one log | p1s:1134-1174 (`a34_clone_dance.em`) | E | no |
| AN-76 | Global mutable state (A19): `FRAME += 1` on a static says "cannot find `FRAME`"; `LOG.push` is E3021; a module-level `FRAME = 0` gives "`main` is already declared" | count frames and keep a log in module variables | p1s:740-821, 1193-1195; 3b-thr:1241-1244 | E | no |
| AN-77 | Async and generators (A14): a generator walking its own local list across `yield` is E2221; a class-field borrow across `yield` is E2229; `async` is reserved | a generator yields lines from its own buffer | p1s:454-553 (`a14_generators.em`, `a14_async_kw.em`) | E | no |
| AN-78 | Unsafe and FFI (A20): a C callback with user data needs a written contract; today's trampoline costs 1.47x plain C per callback; C callbacks written `owned fn` depend on row 6's replacement | a C library calls back into the game | p1s:823-902 (`a20_ffi.em`); 3b-thr:1247-1249; r3:327 | E | no |
| AN-79 | `unsafe` points two ways ("I vouch here" on a block, "the caller must vouch" on `unsafe fn`), and an `unsafe fn` body is unchecked with no mark per operation | write a small unsafe helper and call it | r1:37; r4:142; r2:156 | E | no |
| AN-80 | `Sync` names three things (an effect, a property, `@sync`); `@sync` reads "synchronised" and the spec must forbid that reading; `@nosync` reads "not `@sync`" | mark a class that threads may share; forbid locks in a hot function | r3:34-40; r1:34; r4:70, 154 (spec 3184-3186, 3445, 3473-3475) | E | no |
| AN-81 | `@static_safe` says "all safety proved when compiling" and forbids only aliasing checks; bounds, overflow and stale checks remain | mark a hot loop as having no run-time checks | r3:41-44; r1:35; r4:69 (spec 3260) | E | no |
| AN-82 | `@no_runtime_checks` is reserved, promising a switch the spec says will never exist | turn off safety checks in a release build | r2:109-111 (spec 1875-1876, 2708, 3173) | E | no |
| AN-83 | `@fp(contract)` allows fused multiply-add, while "contract" elsewhere means a checked promise | allow fused multiply-add in one function | r3:45-47; r4:71 | E | no |
| AN-84 | Two error codes mean two unrelated things each: E5015 (a promoted C variadic argument / an owning value in an export) and E7010 (a parallel loop's write / a non-`Sync` static reachable from an `any`-thread export) | read an error code and look it up | r3:63-66 (spec 4738, 4959; 3589, 4966) | E | no |

## 2. Counts and the phase-1 map

| area | ids | count | settled by an agreed row |
|---|---|---|---|
| A | AN-01 .. AN-16 | 16 | AN-04, AN-05, AN-06 ("delete after the loop"); AN-08, AN-09 (row 5) |
| B | AN-17 .. AN-33 | 17 | AN-17, AN-18, AN-19 (row 3); AN-30 (row 4); AN-31 (row 7) |
| C | AN-34 .. AN-52 | 19 | AN-34, AN-36 (row 9); AN-37, AN-38 (row 8); AN-40 (`owned` replaces `Box`) |
| D | AN-53 .. AN-70 | 18 | AN-53, AN-54, AN-55 (row 6) |
| E | AN-71 .. AN-84 | 14 | none |
| all | AN-01 .. AN-84 | 84 | 18 |

Every phase-1 id has one home: A1 AN-17; A2 AN-01; A3 AN-03; A4 AN-04; A5 AN-22; A6 AN-24; A7 AN-25;
A8 AN-63; A9 AN-48; A10 AN-39; A11 AN-34; A12 AN-53; A13 AN-67; A14 AN-77; A15 AN-71; A16 AN-69;
A17 AN-16; A18 AN-31; A19 AN-76; A20 AN-78; A21 AN-12; A22 AN-08; A23 AN-37; A24 AN-42; A25 AN-07;
A31 AN-21; A32 AN-60; A33 AN-30; A34 AN-75. Every item of `FINDINGS-phase3b.md` section 5 has one:
1 AN-05, 2 AN-02, 3 AN-39, 4 AN-63, 5 AN-52, 6 AN-62, 7 AN-74, 8 AN-69. Every defect of `FINDINGS.md`
section 5: the guard hole AN-30; the `Option[ref Node]` crash AN-41; E3011 vs E3040 AN-01; the
`or_default` E2140 AN-23; L3001's wrong edge AN-64; `BorrowList` AN-07; E3026 in the callee AN-57;
"cannot find `n`" AN-73; the run-pass test that must panic AN-42.

## 3. Notes for the area designers

- **Error messages are part of each answer.** Reader-4's section 2.3 (`r4:331-573`) lists 200 registry
  codes, and section 2.4 five run-time messages, that do not give the line to write. Each area rewrites
  the messages of its own annoyances. The codes that belong to no annoyance here (lexing, names, the
  build, C++, GPU) are outside the memory model and are left to the lead, not to an area.
- **Annoyances that need another area's answer.** AN-20 (B) and AN-44 (C) are one question seen from
  two sides: B says how a name says it changes a class object; C says what `=` does with one. AN-60 (D)
  needs A's loop rule when the dispatch loop is a `for`. AN-75 (E) depends on row 6's capture rule (D).
  AN-66 (D) needs C's `shared`. AN-74 (E) depends on whether `@sync` stays (vocabulary settlement 1).
- **Agreed rows stay exactly as agreed.** A settled annoyance still gets the full eight-part answer;
  the agreed spelling and meaning are its NEW code and may not be changed.
