# R9-new-gaps-1-24

could_not_cover: All of NG-01 to NG-24 were decided against the current text. Not read: the round-1 JSON records (G3.json, G4.json, G7.json; their structure did not match my one parsing attempt and applied.md gave the dispositions I needed), Appendix F's glossary entries for 'hand-over' and 'region' (the §VII.1 terms and [LT-45]/[ALC-5a] were used instead), and [RT-7]/[RT-8] (the chunk and queue mechanics; [OBJ-1] and [RC-4] sufficed for NG-05/07). The cross-cutting FFI edit (NG-11's [FFI-10] and [FFI-21a] texts) is written once and referred to from NG-12, 13, 15, 16, 17, 19, 20, 21; the lead should apply it once. The internal contradiction on `thread_shared … using` (NG-06, decisions.md N84's two halves) needs the lead's choice between the two safe resolutions before either set of edits is applied.

## NG-01  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: ALC-5a: "A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one, since the target's block stays until the link ends (`[WK-11]`, `[OBJ-3]`): the link carries the instance's region from the line that makes it (`w = weakref n`) through every place it is copied or stored, as a kept view does (`[LT-45]`); keeping it where it would outlive the instance is `E3063`"; E3063 registry row: "a view of the callee's own locals put into the caller's view through `ref mut`, a kept view outliving its target; and a value made in a memory manager's instance, or one holding such a value, kept or returned where it would outlive that instance"; §VII.1 Terms (reshape): "**reshape**: a change that can free or move memory: replacing a value that owns memory; ...; handing a value over (its old place is left empty, and a value handed to another thread may move, `[THR-2]`)"; OWN-3: "A hand-over is a reshape of the giver's place (§VII.1, `[BRW-17]`)"; ALC-5: "While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it and gives blocks back to it ... Reading it stays allowed (`current.made`); any other change, through any name, is `E3021`"; LT-4: "that address is a kept `sharedref mut` link to the instance (`[LT-46]`): the program may use the instance while values made in it live, and each value's growth and end change the instance through it"

**program**: 

**remaining_doubt**: The program is refused twice. `gun.last = weakref b` stores a link that carries `pool`'s region (a local of `fire`, [ALC-5a]) into `gun`, a `ref mut` parameter whose place outlives the frame: E3063 (the registry row's own words, a view of the callee's locals put into the caller's view through `ref mut`). And `return pool` is a hand-over, which §VII.1 and [OWN-3] make a reshape, that is a change of the instance while a value that holds one (the link, by [ALC-5a]) still lives: E3021 under [ALC-5]. The finder's alternative reading, the instance's region following the move into `armory`, has no rule behind it: a region is a place's, a local's region is its function's, and [ALC-5a] fixes the region at the line that makes the link. The same chain covers a `FoundByAddress` manager (no word, but [ALC-5] forbids the hand-over all the same). One residual: [ALC-5]'s E3021 example is a `reset()`, and 'any other change' reaches a hand-over only through §VII.1's definition of reshape; a parenthetical in [ALC-5], '(a hand-over of the instance included, §VII.1)', would make it unmissable. Not needed for safety.

**REFUTER reason**: Both refusals the reader names stand on the current words. `[ALC-5a]`: "the link carries the instance's region from the line that makes it (`w = weakref n`) through every place it is copied or stored, as a kept view does (`[LT-45]`); keeping it where it would outlive the instance is `E3063`", and `[LT-47]`: "Inside the callee only such a view may be put there ... a view of the callee's own locals or of a temporary is `E3063`", so `gun.last = weakref b` with `pool` a local of `fire` is E3063. For `return pool`: §VII.1 "**reshape**: a change that can free or move memory: ...; handing a value over (its old place is left empty ...)", `[OWN-3]` "A hand-over is a reshape of the giver's place (§VII.1, `[BRW-17]`)", and `[ALC-5]` "While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it and gives blocks back to it ... any other change, through any name, is `E3021`"; `[ALC-5a]` makes the live link such a value. A hand-over is by definition a change, so the two-step reading is the text's own, not an inference. The reader's optional parenthetical in `[ALC-5]` is harmless but not needed; the finder's 'region follows the move' has no rule behind it, as the reader says.

## NG-02  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: ALC-5a: "A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one, since the target's block stays until the link ends"; ALC-5a: "A field or element of a `weakref` type whose target was made in an instance carries that instance's region as a kept view field does (`[LT-45]`), so the object or list that holds it lives no longer than the instance (`E3063`)"; WK-11: "A `weakref` link does not keep its target alive, and its type says what kind of target it has."; §VII.1 Terms (into X): "It does not go through links: what a link reaches is another object."; OBJ-3: "When the count of owners reaches 0 the object is ended (`[WK-3]`): its `drop` chain and its field drops run, and the weak count is decremented; the block is freed when the weak count reaches 0"

**program**: 

**remaining_doubt**: The current text says exactly what the finder's fix_hint asks: a link counts as holding a value only when its own target was made in an instance, 'since the target's block stays until the link ends'. The boss's block is `Global`'s; its mesh is a field, ended with the boss (`OBJ-3`: field drops run when the owner count reaches 0, before the weak count matters), so the mesh's block goes back to `level` while `level` lives, and `cam.target` keeps only the boss's `Global` block. [LT-4]'s 'every value that holds one' does not reach through a link (§VII.1: a view or holder 'does not go through links'), and [WK-11] says a link holds nothing. The program compiles and prints 500 then gone. No doubt left.

**REFUTER reason**: `[ALC-5a]` counts only "A `weakref` link to a value made in a memory manager's instance"; the boss is made without `using`, so by `[CLS-1]` "the block comes from the built-in memory manager". The boss itself "holds one" and so by `[LT-4]` "lives no longer than that instance" (E3063 if it would outlive `level`), and by `[OBJ-3]` "When the count of owners reaches 0 the object is ended ... its `drop` chain and its field drops run, and the weak count is decremented; the block is freed when the weak count reaches 0": the mesh goes back to `level` at the boss's end, while `level` lives; the link keeps only the boss's `Global` block, and §VII.1 "It does not go through links: what a link reaches is another object" keeps `[LT-4]`'s 'every value that holds one' from reaching through `cam.target`. A `level.reset()` while the boss lives is E3021 by `[ALC-5]` (the boss holds a value made in it). The program compiles and prints 500 then gone.

## NG-03  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: ALC-5a: "Nothing runs for this: the block a link keeps is given back by the manager's own `deallocate` when the weak count reaches 0, as any other, which for a manager that frees nothing one by one is a no-op, so this check is what keeps the block from reuse."; OBJ-1: "An object carries nothing else for the memory model: no access words, and no flags but a `thread_shared` value's three marks and the kept-by-C mark, the top bit of the type index (`[FFI-35a]`), so a type index is below 2^31."; OBJ-4: "An object made in a manager keeps its instance's address only as `[ALC-1]` says: none in a manager that frees nothing one by one, or in one that implements `FoundByAddress`."; CLS-1: "the block comes from the built-in memory manager, or from the instance the line that makes the object names with `using` (`Enemy(hp=5) using current`, `[ALC-1]`)"; TYP-46: "It may follow an `owns`, `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name: String using LevelArena = String("") using current`); after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`"; GRM-3a: "After `weakref` stands the target's own type, with `thread_shared` before it for a link to a `thread_shared` value"; ALC-1: "**The place names the kind, the making line names the instance.**"

**program**: The finder's program as written compiles under the current text: `a = Enemy(hp=1)` ([CLS-1]: the built-in manager), `b = Enemy(hp=2) using level` ([ALC-1]: a class object made in an instance), both links are `weakref Enemy` ([OWN-13] reads the link's type from the holding words, which `using` is not; a `using` on a `weakref` type is E2114 by [TYP-46]), and `links.clear()` ends the last link of each. The implementer's question: at the end of the last link, [ALC-5a] says the block 'is given back by the manager's own `deallocate`', but the code compiled for one list element must choose among four give-backs, `Global`'s free, a call through the instance word kept in the block, nothing (a manager whose `deallocate` is empty, which keeps no word) and a `FoundByAddress` lookup, and nothing it can read says which: the link's type names no manager, and the block carries 'nothing else for the memory model' ([OBJ-1]), the word being present only for some managers ([OBJ-4]). Whatever one implementation picks is wrong for the other element: a `Global` block handed to `level` or an arena block handed to `free`.

- READER EDIT rule=TYP-46
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`), and a `weakref T` type, where it names the kind
  of manager the target's block comes from (`last: weakref Enemy using Arena`, `Array[weakref Enemy
  using Arena]`), since the link's end gives that block back (`[ALC-5a]`) and must know to whom: a
  `weakref x` takes the kind from the type of what `x` names, as it takes the target's kind from the
  words on the path (`[OWN-13]`), and a link given to a `weakref T` place of another kind is `E2020`
  ("`links` holds links to `Enemy` objects from the built-in manager (`Array[weakref Enemy]`), and `b`
  was made in `level`, an `Arena`; say which kind: `links: Array[weakref Enemy using Arena]`, or make `a`
  in `level` too"); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - why: The place names the kind everywhere else ([ALC-1], verdict 24); a link is the one place that holds a block after its value has ended, so it is the one place that needs the kind and did not say it. Nothing runs for it: the give-back is chosen when compiling, from the type.

- READER EDIT rule=ALC-5a
  - old: Nothing runs for this: the block a
  link keeps is given back by the manager's own `deallocate` when the weak count reaches 0, as any
  other, which for a manager that frees nothing one by one is a no-op, so this check is what keeps
  the block from reuse.
  - new: Nothing runs for this: the block a
  link keeps is given back by the manager's own `deallocate` when the weak count reaches 0, as any
  other, which for a manager that frees nothing one by one is a no-op, so this check is what keeps
  the block from reuse. Which give-back that is, the built-in manager's, a call through the instance
  word the block keeps, nothing, or the instance found from the block's address, is read from the
  link's type (`weakref Enemy`, `weakref Enemy using Arena`, `[TYP-46]`) when the link's end is
  compiled, never from the block, which carries no mark for it (`[OBJ-1]`).
  - why: Says where the implementer finds the kind at the link's end.

- READER EDIT rule=OWN-13
  - old: 
  - new: A `weakref x` whose target was made `using` an instance (its type says `using A`, `[TYP-46]`) is a `weakref T using A`, so the kind of manager travels with the link, and a `weakref` of a `Global` value is a plain `weakref T`.
  - why: The inference rule for the link's type must name the manager kind as it names the thread kind.

**remaining_doubt**: The alternative, a mark in the block saying where it came from, would cost every object with a weak count a word or a flag bit [OBJ-1] does not have; the type route costs nothing and follows [ALC-1]'s own principle. The E2020 row of the registry already covers 'a value given to a `using A` place that was not made in an instance of `A`', so no new code is needed.

**REFUTER reason**: The text accepts the program and leaves the give-back undecidable. The link's type cannot carry the kind: `[TYP-46]` "It may follow an `owns`, `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` ...; after any other type ... it is `E2114`", and `[GRM-3]` "`using` applies to the whole type before it", so `weakref Enemy using Arena` is `(weakref Enemy) using Arena`, E2114. `[OWN-13]` reads the link's type "from the words on the path to `x`", and `using` is none of them, so `weakref b` is a `weakref Enemy` and `links.push(weakref b)` type-checks beside `weakref a`. At the last link's end `[ALC-5a]` says "the block a link keeps is given back by the manager's own `deallocate` when the weak count reaches 0", but `[OBJ-1]` "An object carries nothing else for the memory model" and `[OBJ-4]` "An object made in a manager keeps its instance's address only as `[ALC-1]` says: none in a manager that frees nothing one by one, or in one that implements `FoundByAddress`" leave the compiled code for one list element with four give-backs (`Global`'s free, a call through a word that may not be there, nothing, a piece lookup) and no datum to choose by; an arena block handed to `Global`'s free is an invalid free from Safe Ember (`[PHIL-10]`). The reader's route (the kind in the link's type, `[ALC-1]`'s own principle, nothing at run time) is right; verdict 24 and 107 are kept. What the reader did not see: inside a method `this` carries no `using`, so `weakref this` (`child.parent = weakref this`) has no kind to read under any of these edits; listed under missed.

- BETTER EDIT rule=GRM-3
  - old: `using` applies to the whole type before it, so `owns Mesh using LevelArena` is an
  `owns Mesh` whose block comes from a `LevelArena`.
  - new: `using` applies to the whole type before it, so `owns Mesh using LevelArena` is an
  `owns Mesh` whose block comes from a `LevelArena`, and `weakref Enemy using Arena` a `weakref Enemy`
  whose target's block comes from an `Arena`, the kind the link's end gives that block back to
  (`[TYP-46]`, `[ALC-5a]`).
  - why: The reader's `[TYP-46]` edit adds `using` after a `weakref T` type, but `[GRM-3]` is where the parse of `weakref Enemy using Arena` is fixed (the whole type before `using` is the link), so it must say what the manager of a link is: its target's block, since a link has no block of its own. The reader's three edits stand beside this one.

## NG-04  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: ALC-5a: "A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one, since the target's block stays until the link ends"; ALC-5: "While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it and gives blocks back to it ... any other change, through any name, is `E3021`"; OWN-3: "A hand-over is a reshape of the giver's place (§VII.1, `[BRW-17]`): a view into `a` that is still used after `b = a` is `E3060` at that use, and `a` is not handed over while a `ref mut` name of it is still used (`E3022`)."; §VII.1 Terms (reshape): "**reshape**: a change that can free or move memory: replacing a value that owns memory; ...; handing a value over (its old place is left empty, and a value handed to another thread may move, `[THR-2]`)"

**program**: 

**remaining_doubt**: `moved = counter` hands `counter` over ([OWN-3]); a hand-over is a reshape (§VII.1), 'a change that can free or move memory'; [ALC-5] allows no change of the instance but allocation and give-back 'while a value made in a memory manager's instance lives', and [ALC-5a] makes the live link `cam.target` such a value. So `moved = counter` is E3021, naming the link's line. The finder's claim that a hand-over is not 'a change' in [ALC-5]'s sense is refuted by §VII.1's definition. Residual as in NG-01: a parenthetical in [ALC-5] naming the hand-over would spare an implementer the two-step reading.

**REFUTER reason**: `[OWN-3]` "`b = a`, where `a` is a whole name that owns a value whose type is not `Copy`, hands `a` over" and "A hand-over is a reshape of the giver's place (§VII.1, `[BRW-17]`)"; §VII.1 "**reshape**: a change that can free or move memory: ...; handing a value over"; `[ALC-5]` "While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it and gives blocks back to it ... any other change, through any name, is `E3021`"; `[ALC-5a]` "A `weakref` link to a value made in a memory manager's instance counts, for `[LT-4]` and `[ALC-5]`, as a value that holds one". So `moved = counter` while `cam.target` lives is E3021, naming the link's line. Safe: the instance's struct (and a kept instance word, `[LT-4]` "that address is a kept `sharedref mut` link to the instance") would otherwise move under a live link.

## NG-05  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: OBJ-1: "Which thread made a value is read from the header of the memory chunk its block came from, never from the value: the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator (`[ALC-3]`, `[FFI-27]`, else `[RT-1]`'s), writes the making thread into each chunk's header once, and a chunk belongs to that thread for the life of the process, never given to another thread's allocator, and a block freed goes back to that thread's own pool, so the record never changes."; COST-3 row (A `thread_shared` value's two counts): "blocks made in `Global` come in chunks of the configured allocator, each chunk's header naming its thread, read with one load on the away path (`[OBJ-1]`)"; ALC-2: "`Global` is the package's configured allocator (the one `[ALC-3]` names, else the platform's)"

**program**: 

**remaining_doubt**: The finder's missing rule is now written in [OBJ-1] (decisions.md N84): whatever `Global` is, `TALLY` here, the runtime carves chunks from it and writes the making thread into each chunk's header, so `stats`'s maker is read from memory the runtime laid out. `TALLY` sees chunk-sized calls, not one per block. None.

**REFUTER reason**: `[OBJ-1]`: "Which thread made a value is read from the header of the memory chunk its block came from, never from the value: the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator (`[ALC-3]`, `[FFI-27]`, else `[RT-1]`'s), writes the making thread into each chunk's header once, and a chunk belongs to that thread for the life of the process"; `[ALC-2]` "`Global` is the package's configured allocator (the one `[ALC-3]` names, else the platform's)". `TALLY` is that configured allocator and sees chunk-sized requests; the record is in memory the runtime laid out (ruling N84). Nothing left.

## NG-06  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: THR-1: "it is written before a new value (`st = thread_shared Stats()`) or on a value already `thread_shared` (`fn() thread_shared st:`), and on a value made otherwise it is `E7017`"; RC-1: "`thread_shared x` is written only before a new value or a name that is already a `thread_shared` owner (`E7017`, `[OWN-13]`, `[THR-1]`), since its header and its making thread's record are written where the value is made"; BRW-12: "A call whose result is a new value the callee made (a list it built, a list of views included) hands that value to the name, which owns it"; OBJ-1: "The header is 8 bytes: a 32-bit count and a 32-bit type index. ... A `thread_shared` value's header is 32 bytes (verdict 101), in every program"; OBJ-4: "A `thread_shared` value's block always comes from the runtime allocator (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is `E2114`"; ALC-1: "never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`"; TYP-46: "after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`"; OPT-1: "A `thread_shared` value is never placed in a frame, and never in a memory manager's instance: its making thread is read from its chunk (`[RC-4]`), which only the runtime allocator's blocks have."; OBJ-1: "A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4) and no thread asks which thread made it."; RC-4 point 4: "A `thread_shared` value made `using` an instance of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that makes it"; TYP-46a: "`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making line"; ALC-5: "A `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4), so the rule is checked as written"

**program**: Two questions the implementer cannot answer from the text. (1) Line `stats = thread_shared load_stats(ref mut level)`: [THR-1] and [RC-1] allow `thread_shared` 'before a new value', and [BRW-12] calls a call's result 'a new value the callee made'; [RC-1]'s reason ('its header and its making thread's record are written where the value is made') decides E7017, since the callee made the object with the 8- or 12-byte one-owner header of [OBJ-1], which cannot become the 32-byte one, but the words do not say it, and a compiler reading 'new value' as [BRW-12] does accepts the line and keeps an arena block under a `thread_shared` owner with no chunk header to read. (2) Whether a `thread_shared` value may be made `using` an instance at all: [OBJ-4], [ALC-1], [TYP-46] and [OPT-1] say never (E2114), while [OBJ-1], [RC-4] point 4, [TYP-46a], [ALC-5] and the [COST-3] row describe such a value as made, marked handed over at its making line and refused only for a manager that is not `ThreadShared` (E7015). Both halves come from one ruling (decisions.md N84) and contradict each other; the result type `Stats using Arena` of line 7 falls on the fault line.

- READER EDIT rule=THR-1
  - old: it is written before a new value (`st = thread_shared Stats()`) or on a value
  already `thread_shared` (`fn() thread_shared st:`), and on a value made otherwise it is `E7017`,
  - new: it is written before a value made on that line (`st = thread_shared Stats()`, a constructor
  call, a literal or a struct value, whose 32-byte header is written where it is made, `[OBJ-1]`) or
  on a value already `thread_shared` (`fn() thread_shared st:`), and on any other value, a name or a
  call's result whose type is not `thread_shared` (`stats = thread_shared load_stats(ref mut level)`),
  it is `E7017` ("`load_stats` returns a `Stats` with one owner, made in `load_stats` with a one-owner
  header; to share it between threads, make it shared where it is made: write `-> thread_shared Stats`
  and `return thread_shared Stats()` in `load_stats`"),
  - why: Pins 'a new value' to the line the compiler sees, which is the only place the 32-byte header can be written; the reason in [RC-1] already says so.

- READER EDIT rule=registry row E7017
  - old: | `E7017` | `thread_shared` on a name whose value was not created `thread_shared`;
  - new: | `E7017` | `thread_shared` on a name, or on a call's result, whose value was not created `thread_shared`;
  - why: The row follows the rule.

- READER EDIT rule=OBJ-4
  - old: A `thread_shared` value's block always comes from the runtime allocator
  (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is
  `E2114` ("a `thread_shared` value's block carries its making thread's record, which only the
  runtime allocator keeps; remove `using m`: `names = thread_shared Names()`").
  - new: A `thread_shared` value's block comes from the runtime allocator (`[RT-1]`), in a chunk whose
  header names its making thread (`[OBJ-1]`), unless its making line says `using m`: then it lies in
  memory the runtime did not lay out, is marked handed over at that line and counts every owner in
  its away count (`[RC-4]` point 4), and `m`'s type is `ThreadShared`, since whichever thread ends
  the last owner calls `m`'s `deallocate` (`[TYP-46a]`, `E7015` otherwise).
  - why: Removes the E2114 that contradicts [OBJ-1], [RC-4], [TYP-46a] and [ALC-5]; keeps the feature (a `thread_shared` value in a thread-safe pool) and the safety argument the four rules already give. The owner's condition on the delegation is the better shape, never the cheaper refusal.

- READER EDIT rule=ALC-1
  - old: never a
  `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`)
  - new: a
  `thread_shared` value too, which is then marked handed over at its making line and needs a
  `ThreadShared` manager, `[OBJ-4]`, `[RC-4]` point 4, `[TYP-46a]`)
  - why: Same contradiction, same side.

- READER EDIT rule=TYP-46
  - old: after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: after any other type it is `E2114`
  - why: Same contradiction; `thread_shared T using A` is the type [TYP-46a] already judges.

- READER EDIT rule=OPT-1
  - old: A `thread_shared` value is never placed in a frame, and never in a
  memory manager's instance: its making thread is read from its chunk (`[RC-4]`), which only the
  runtime allocator's blocks have.
  - new: A `thread_shared` value is never placed in a frame: its making thread is read from its chunk
  (`[RC-4]`), which only the runtime allocator's blocks have, and one made `using` an instance is
  marked handed over at its making line instead (`[RC-4]` point 4).
  - why: The optimizer never moves a value into an instance; the sentence as written joined the refusing camp.

**remaining_doubt**: The other resolution of (2), keeping the four E2114 rules and deleting the handed-over-at-making clauses of [OBJ-1], [RC-4] point 4, [TYP-46a], [ALC-5] and the [COST-3] row, is equally safe and smaller in run-time surface, but it drops a feature (thread-shared objects in a `ThreadShared` pool allocator) that four rules were written to support; the lead who wrote N84 must choose. Under either, (1) stands and its edit is needed. For (1) the same words appear in [OWN-13] and [RC-1] ('written only before a new value'); the lead may mirror the change there.

**REFUTER reason**: Half confirmed, half refuted. (2) is a real contradiction, both halves written by N84: `[OBJ-4]` "A `thread_shared` value's block always comes from the runtime allocator (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is `E2114`", `[ALC-1]` "never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`", `[TYP-46]` "after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`" and `[OPT-1]` "never in a memory manager's instance" against `[OBJ-1]` "A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it", `[RC-4]` point 4 "is marked handed over at the line that makes it", `[TYP-46a]` "`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making line", `[ALC-5]` "A `thread_shared` value made in an instance is ended where its last owner ends (`[RC-4]` point 4)" and the `[COST-3]` row "a value made `using` an instance counts every owner atomically, as if handed over at its making line". No owner verdict decides it (verdict 101 fixes the header only); the allow side keeps the feature with the four rules' safety argument (every owner in the atomic away count, `A: ThreadShared`, the end never delayed so `[ALC-5]` holds), so it is the better shape under the delegation's condition, and the reader's four edits for it are sound; the lead rules. (1) is refuted: `[THR-1]` "it is written before a new value (`st = thread_shared Stats()`) or on a value already `thread_shared` (`fn() thread_shared st:`), and on a value made otherwise it is `E7017`", with `[RC-1]`'s reason "since its header and its making thread's record are written where the value is made": the class object `load_stats` returns was made in `load_stats` by `Stats()`, a value 'made otherwise', so the line is E7017 as written; `[BRW-12]`'s "a new value the callee made" says who owns a result, not where its header was written. The reader's `[THR-1]` and E7017 edits should not be applied: they would also refuse `thread_shared make_point()` for a struct result, which has no block yet and can be put in a fresh block at the line as `[RC-1]` does for `shares` of a local ("a value that is not a class object is then made in a block of its own from the line that declares the local").

- BETTER EDIT rule=registry row E2114
  - old: ; `thread_shared` before a value made `using m`, whose block would carry no making-thread record |
  - new:  |
  - why: If the allow side is chosen (the reader's OBJ-4, ALC-1, TYP-46 and OPT-1 edits), the registry row that lists the refusal must go too, or the registry contradicts the rules it points to.

- BETTER EDIT rule=RC-4 point 4
  - old: `thread_shared` value made `using` an instance of a manager that may have several instances
  - new: `thread_shared` value made `using` any instance but `Global` (`System` and a manager with one instance included, whose blocks lie in no chunk of `[OBJ-1]`)
  - why: `[OBJ-1]` lays chunks out for `Global` only; point 4 marks handed over only a value made in 'a manager that may have several instances', so `thread_shared Stats() using System`, or `using` a one-instance `static` manager, would have neither a chunk header to read its making thread from nor the mark: a gap in both resolutions, since under the refuse side `[TYP-46]`'s E2114 covers it but under the allow side nothing does.

- BETTER EDIT rule=RC-4 point 4
  - old: a value made in `Global` or `System` keeps the two counts
  - new: a value made in `Global` keeps the two counts
  - why: Same gap: only a `Global` block has the chunk header the home count relies on.

## NG-07  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: OBJ-1: "the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator (`[ALC-3]`, `[FFI-27]`, else `[RT-1]`'s), writes the making thread into each chunk's header once, and a chunk belongs to that thread for the life of the process"; ALC-3: "A binary package may replace the global allocator with `[build] global_allocator = "module.NAME"` naming a `static` whose type implements `Allocator`; an embedding host may pass allocation functions to `ember_rt_init` (`[FFI-27]`)."; COST-3 row (A `thread_shared` value's two counts): "blocks made in `Global` come in chunks of the configured allocator, each chunk's header naming its thread, read with one load on the away path (`[OBJ-1]`)"

**program**: 

**remaining_doubt**: The same rule as NG-05: `Traced`, or a host's functions through `ember_rt_init`, is 'the configured allocator' from which the runtime takes chunks and in whose chunk headers it records the making thread. The runtime layer the finder said was unstated is stated. None.

**REFUTER reason**: Same rule as NG-05: `[OBJ-1]` "the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator (`[ALC-3]`, `[FFI-27]`, else `[RT-1]`'s), writes the making thread into each chunk's header once"; `[ALC-3]` "an embedding host may pass allocation functions to `ember_rt_init` (`[FFI-27]`)". `Traced`, or the host's functions, supply chunks; the record is the runtime's. Nothing left.

## NG-08  reader=needs_owner  refuter=refuted  should_be=not_a_defect  safety=none

**answered_by**: GRM-3: "`using` applies to the whole type before it, so `owns Mesh using LevelArena` is an `owns Mesh` whose block comes from a `LevelArena`."; GRM-43: "`using` after a value names the instance of the memory manager it is made in (`owns Mesh(points=[0.0, 1.0]) using current`, `[0.0, 1.0] using current`, `Enemy(hp=500) using current`, `[TYP-46]`); it binds tighter than the words before a value, so the mesh is made in `current` and then handed to its place."; TYP-46: "It may follow an `owns`, `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` ... after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`"; ALC-1: "`using A` after a type whose value has heap memory of its own (an `owns`, `shares` or `shares mut` value, a class object, an `Array`, `Map`, `Set` or `String` ...) names the kind of memory manager that memory comes from ... `using` after any other type is `E2114`"; §IV.2a (type alias): "A `type` alias without one is transparent."

**program**: Decided, but by two binding rules that disagree. On a type, `using` covers the whole type ([GRM-3]), so `keywords: thread_shared Array[String] using System` is E2114 ([TYP-46]). Before a value, `using` binds tighter than the words ([GRM-43]), so `thread_shared [String("let")] using System` is `thread_shared ([..] using System)`, a list whose buffer is `System`'s under a shared block of the runtime allocator, accepted, and its type has no direct spelling: only `type SysWords = Array[String] using System` then `keywords: thread_shared SysWords`, through the transparent alias. The mismatch cuts the other way too: [TYP-46]'s own example `mesh: shares Mesh using LevelArena` has, for a struct `Mesh`, no making line, since `shares Mesh(points=[..]) using current` is `shares (Mesh(...) using current)` and a struct value 'has no heap memory of its own' ([ALC-1], E2114). No safety is at stake; it is a readability and consistency choice (verdict 24 is met by either binding). Options: (a) the type binds as the value does: `using` names the memory of the type under the word (`thread_shared Array[String] using System` is the list's buffer, the shared block from the runtime allocator; `shares Mesh using A` for a struct `Mesh` is E2114, as its making line already is), so every making line has a field spelling and no feature that works today is lost; (b) the value binds as the type does: `shares Mesh(...) using current` puts the counted block in `current`, `thread_shared [..] using System` is E2114 as the type is, and the inner form needs brackets, `thread_shared ([..] using System)`, which the type grammar cannot write; (c) keep both, with the alias as the spelling. Recommendation: (a), one binding rule read the same on both sides (N79's own aim, 'the value words bind the same everywhere'), nothing new at run time.

- READER EDIT rule=GRM-3
  - old: `using` applies to the whole type before it, so `owns Mesh using LevelArena` is an
  `owns Mesh` whose block comes from a `LevelArena`.
  - new: `using` binds tighter than the words before a type, as it does before a value (`[GRM-43]`): it
  names where the memory of the type under the word comes from, so `owns Mesh using LevelArena` is an
  `owns Mesh` whose block comes from a `LevelArena`, `Enemy using LevelArena` a class object whose
  block does, and `thread_shared Array[String] using System` a shared list whose buffer is `System`'s
  under a block of the runtime allocator (`[OBJ-4]`), the type of `thread_shared [String("let")]
  using System`.
  - why: If (a) is chosen: one binding on both sides, so the type of every making line can be written in a field, a parameter and a result.

- READER EDIT rule=TYP-46
  - old: It may follow an `owns`,
  `shares` or `shares mut` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name:
  String using LevelArena = String("") using current`); after any other type, a `thread_shared` type
  included (`[OBJ-4]`), it is `E2114`
  - new: It stands after the type whose memory it names, under any word of `[TYP-45]` (`[GRM-3]`): an
  `owns` type, a class type, or an `Array`, a `Map`, a `Set` or a `String` (`name: String using
  LevelArena = String("") using current`, `keywords: thread_shared Array[String] using System`); after
  any other type, a struct or enum type under `shares` or `thread_shared` included, whose counted block
  is the runtime allocator's, it is `E2114`
  - why: If (a) is chosen: the list of types follows the binding; `shares Mesh using A` for a struct goes, since no line could make one.

**remaining_doubt**: The edits are for option (a) only; under (c) nothing changes and the alias stays the spelling. Not a [PHIL-10] matter: no program is unsafe either way.

**REFUTER reason**: The premise fails: the value form the reader says is accepted is refused by `[OBJ-4]`, "`thread_shared` before a value made `using m` is `E2114` (\"a `thread_shared` value's block carries its making thread's record, which only the runtime allocator keeps; remove `using m`: `names = thread_shared Names()`\")", the same answer `[TYP-46]` gives the type ("after any other type, a `thread_shared` type included (`[OBJ-4]`), it is `E2114`"). The two bindings name one thing: `[GRM-43]` "`using` after a value names the instance of the memory manager it is made in ...; it binds tighter than the words before a value, so the mesh is made in `current` and then handed to its place" and `[GRM-3]` "`owns Mesh using LevelArena` is an `owns Mesh` whose block comes from a `LevelArena`" both describe the block `[ALC-1]` names ("takes the mesh's block from the arena `current` by calling `current.allocate(…)`"); `[TYP-46]` "`using` names where that value's own memory comes from", and a `shares` struct's own memory is its counted block (`[RC-1]` "A value made shared lives in a block of its own"), so `shares Mesh(points=[0.0]) using current` makes that block in `current` and `mesh: shares Mesh using LevelArena` is its spelling. The one type with no direct spelling would be `thread_shared T using A`, which exists only under NG-06's allow side and which `[TYP-46a]` already writes (`[TYP-46]`'s list is then edited there, not here). The reader's `[GRM-3]` edit would make `thread_shared Array[String] using System` name the list's buffer, where `[RC-4]` point 4 and `[TYP-46a]` read `using` on a `thread_shared` value as its block: a new ambiguity, no safety cost.

## NG-09  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: RC-4 point 3: "The compiler places them: at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line."; RC-4 point 2: "When another thread ends an owner the making thread counted, the away count goes below zero: that thread puts the value in the making thread's queue ... The making thread empties its queue at each of its check points"; THR-20: "**A `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any thread that holds a count of it ... So a function that may run on another thread and holds such a count counts as possibly running that value's `drop`, and the `drop` of everything it owns: what they change is changed on that thread, and a module variable of the main thread they reach is `E7012`."; THR-11: "What each task changes and reads is worked out from its body and every function it may run (`[EFF-24]`), and is checked against the other tasks and the parent until the task's join"

**program**: 

**remaining_doubt**: The finder read a text without the scope-end check point. In the current [RC-4] point 3 the task's end of `c` queues Voice 1 to main (point 2), and main empties its queue at the check point after the `with s` block's end, 'before the parent's next line': Voice's drop pushes to `SAMPLES` before `p = SAMPLES.as_mut_ptr()`, and the back edges of the unsafe loop find an empty queue. The program is safe as written. A general residual the lead may want: no rule says that a check point counts, for the walk and view rules, as a call of the drops of the values queued to this thread; the text is safe without it because the only places such a drop can reshape are a `static mut` (refused through [THR-20]'s clause, since the thread holding the count 'counts as possibly running that value's drop') and lock-ordered fields, where a drop taking a lock the loop holds at a back edge is a self-deadlock, which [PHIL-11] leaves outside the guarantee. Not this cluster's program.

**REFUTER reason**: `[RC-4]` point 3: "at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line"; point 2: "that thread puts the value in the making thread's queue ... The making thread empties its queue at each of its check points". Voice 1 is queued by the task's end and ended by main at the check point after the scope's end, before `p = SAMPLES.as_mut_ptr()`; `[THR-20]` "a function that may run on another thread and holds such a count counts as possibly running that value's `drop` ... a module variable of the main thread they reach is `E7012`" covers the finder's alternative. The reader's residual (a check point as a call of the queued drops for the view rules) is outside this program and the drop of a `thread_shared` value changes only what `[THR-1]`'s first bullet lets it change.

## NG-10  reader=needs_owner  refuter=narrowed  should_be=needs_owner  safety=none

**answered_by**: RC-4 point 3: "The compiler places them: at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends"; RC-4 point 5: "**So a value's end is delayed only** when its last owner ends on a thread that did not make it and the value was not handed over at a place of point 4 (it travelled inside another object): it is ended at its making thread's next check point. Until then it is intact"; PHIL-11: "What remains possible, and is therefore not a defect of the guarantee: resource exhaustion ...; deadlock; a logical race between correctly synchronised operations"; COST-3 row: "| An Ember function called by C through a function pointer (`[FFI-21]`) | guaranteed present | the thread-local attach check of its C entry (`[FFI-22]`), nothing else |"

**program**: Decided, and safely: Texture 1 travelled inside `u`, so it is ended at main's next check point (point 5); the check points before and after `ui_run()` (a call that may block, every effect being an `extern` function's with no `effects=[…]`, [EFF-3]) run before C's loop starts and after it returns, and `draw` has neither loop nor blocking call, so the texture is held for as long as the window is open. No [PHIL-10] promise is broken (the clean-up runs, late). What is undecided is a design choice: (a) as now, a value whose last owner ends on another thread while the making thread lives inside C waits until C returns; (b) a check point where a thread returns to C from an Ember function C called (an export, a callback, an override, [FFI-22]'s entries), one test of the queue per return, beside the attach test already paid, so a C-driven loop (GTK, Cocoa, libuv, a game engine's frame callback) releases on the next frame; (c) a wake of the making thread by the ending thread, which needs a signal the runtime does not have. Recommendation: (b). It is as safe as the existing back-edge check points (the Ember frames below hold no owner of a value whose last owner is gone, and an opened `weakref thread_shared T` name holds a count, [RC-5] point 2), and its cost is one load per return into C.

- READER EDIT rule=RC-4 point 3
  - old: 3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep) and when the thread ends;
  - new: 3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep), where an Ember function that C called returns to C (an export, a
     callback, an override, `[FFI-22]`), so that a thread living in C's loop ends what is queued to
     it at each return, and when the thread ends;
  - why: If (b) is chosen: the most common C-driven architecture otherwise holds every queued value for the life of C's loop.

**cost_row**: | An Ember function called by C through a function pointer (`[FFI-21]`) | guaranteed present | the thread-local attach check of its C entry (`[FFI-22]`), and one test of the thread's queue at its return (`[RC-4]` point 3), nothing else |

**remaining_doubt**: The finder's premise, that the earlier edit's 'outermost Ember call' wording is in the text, is false for the current text: [RC-4] names no return to C at all. The finder's version of the fix (every return into C) is what (b) proposes.

**REFUTER reason**: Decided and safe: `[RC-4]` point 5 "it is ended at its making thread's next check point. Until then it is intact" and `[PHIL-10]` promises no timely end, so no promise is broken; the reader is right that the earlier 'outermost Ember call' wording is not in the text. What is open is a design choice with a run-time cost (one load and one branch per return into C, beside `[FFI-22]`'s attach check), so needs_owner is the right status and the reader's cost row is right. Narrowed in one point: `[RC-4]` point 3 lists "waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep" and no foreign call, so the current text does not place the check points around `ui_run()` the reader describes; the texture would wait for main's next back edge or thread end. The edit above adds the foreign call to the list, which costs nothing new (a foreign call is already a call), and keeps the reader's (b).

- BETTER EDIT rule=RC-4 point 3
  - old: 3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep) and when the thread ends;
  - new: 3. **Check points.** The compiler places them: at every loop's back edge, before and after every
     call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block,
     a channel's receive, a sleep, and a foreign call, whose code the compiler cannot read,
     `[FFI-52]`), where an Ember function that C called returns to C (an export, a callback, an
     override, `[FFI-22]`), so that a thread living in C's loop ends what is queued to it at each
     return, and when the thread ends;
  - why: The reader's (b), with one word more: the list names no foreign call, so the check points 'before and after `ui_run()`' the reader relies on are an inference; a foreign call may block (the finder's GTK loop), so it belongs in the list, and the cost row the reader gives is then the whole cost.

## NG-11  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_proposed_text

**answered_by**: FFI-21: "A function-pointer parameter of a `safe fn` or of an overlay entry carries a `callback(…)` word, or the declaration is `E5002`"; FFI-21: "The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer among them staying raw (a lambda's body uses it in `unsafe:`)"; FFI-10: "A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`."; FFI-21a: "a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`"; §IV.5 Raw types: "Creating one from a reference is safe (ODR-075) ... The raw pointer keeps no compiler-tracked view. Dereferencing, offsetting, reading and writing through it need `unsafe`."; FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"

**program**: The finder's program compiles under the current text with no `unsafe` anywhere: `netq_drain` is a `safe fn` whose only pointer-shaped parameter is a function pointer with a `callback(borrowed)` word ([FFI-21]); [FFI-10]'s refusal names the foreign function's own parameters and result, not the callback's result; [FFI-21a] refuses only a pointer into a by-value parameter's copy, and `one` is a local; `ref_to_ptr(ref mut one)` is safe (§IV.5); `scratch` coerces to `extern "C" fn(usize) -> *mut u8` ([FN-6]). C then writes 1,500 bytes into `scratch`'s finished frame: a use after return from a program that is Safe Ember, which [PHIL-10] promises cannot happen. The cause is one of direction: [FFI-10] judges what safe code hands to C only in the foreign call's own arguments, and a raw pointer an Ember function returns to C is handed to C as surely.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw pointer, or a
  value of an `unsafe` callable type (a C function pointer, `[FFI-56]`), safe to expose, wherever it is
  held in a parameter's or result's type: directly, in a field of a `@layout(c)` value at any depth, an
  element, a payload, the representation of a `@ffi(newtype)` type (`[FFI-8]`), or the result of a
  `callback(…)` parameter's function type, which an Ember function returns to C: safe code may point
  the pointer at anything and choose any C function for the callable, and C reads through the one and
  calls the other with arguments nothing checks. Every declaration that makes foreign code callable
  from safe code with one, `safe fn`, an overlay entry written `fn` (`[FFI-2]`), `@ffi(safe)` on a
  typedef (`[FFI-56]`) and `@ffi(std_function)` on a type (`[FFI-17a]`), is `E5002`, naming the slot
  ("`netq_drain`'s `next_buffer` returns a `*mut u8` to C, which writes through it after the Ember
  function has returned, and safe code may point one at anything; write `unsafe fn netq_drain`, with
  `@safety("…")` saying what the buffer the callback returns must be"; "`pool_run`'s `cfg` holds `reap`,
  an `unsafe extern "C" fn(usize)`, so safe code chooses which C function `pool_run` calls, and with
  what; write `unsafe fn pool_run`, with `@safety("…")` saying what `reap` must do"). A safe `extern
  "C" fn`, which holds an Ember function whose body is checked (`[FN-6]`), is not one, and a raw
  pointer C passes *to* a callback stays raw and is used in `unsafe:` (`[FFI-21]`).
  - why: One property, judged by direction: what safe code hands to C is never made safe by facts about C, wherever in the type it sits and whichever way it travels. Closes NG-11, NG-12, NG-13, NG-15, NG-16, NG-17 at their one cause.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that
  type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part
  of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and
  aggregates in the function, is `E5042`, shown where it is returned and where the type becomes
  `extern "C" fn`.
  - new: Where a program turns a callable type into `extern "C" fn`, a function whose value has that
  type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part
  of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and
  aggregates in the function, is `E5042`, shown where it is returned and where the type becomes
  `extern "C" fn`. More generally, **an Ember function returns a raw pointer to foreign code only out
  of unsafe code**: a function whose value becomes an `extern "C" fn` (given to a `callback(…)`
  parameter, or stored where C reads it, `[FFI-56]`), an `@unsafe(export)` or `@export_table`
  function (`[FFI-26]`) and an `override fn` of a C++ virtual method (`[FFI-39]`) whose result is a
  raw pointer, or holds one at any depth, is an `unsafe fn`, whose body vouches for where the pointer
  points as every unsafe body does (`[UNS-12]`, `@safety` saying what C may do with it); a safe one is
  `E5042` where its type becomes `extern "C" fn`, at the export attribute or at the `override fn`
  ("`scratch` returns a `*mut u8` to C, which may read or write through it after `scratch` has
  returned, and `scratch` is safe code, which may point one at anything; write `unsafe fn
  scratch(want: usize) -> *mut u8` with `@safety("…")`, and in its body return memory that outlives
  the call"). Such a function's `extern "C" fn` type is `unsafe extern "C" fn(…)` (`[FN-6]`).
  - why: The Ember side of the same direction property, for every way C reaches an Ember function ([FFI-52]'s list): the pointer that leaves Ember is made in unsafe code or not at all. Covers NG-11, NG-17, NG-19, NG-20, NG-21.

- READER EDIT rule=FFI-21
  - old: The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C
  function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer
  among them staying raw (a lambda's body uses it in `unsafe:`);
  - new: The callable type of a `callback(…)` parameter is `fn(P…) -> R`: the C
  function pointer's parameters mapped by `[FFI-8]` with the user-data one removed, a raw pointer
  among them staying raw (a lambda's body uses it in `unsafe:`), and `unsafe fn(P…) -> R` where `R`
  is or holds a raw pointer, which only an `unsafe fn` declaration may take (`[FFI-10]`, `[FFI-21a]`);
  - why: The callable type must admit the `unsafe fn` that [FFI-21a] now requires, and only on a declaration whose call is itself in unsafe code.

- READER EDIT rule=registry row E5042
  - old: | `E5042` | a function C may call returns a pointer into a parameter C passes as a copy | `[FFI-21a]` |
  - new: | `E5042` | a function C may call (a callback, an export, a table field, an override) returns a raw pointer from safe code: into a parameter C passes as a copy, or at all, unless the function is an `unsafe fn` | `[FFI-21a]` |
  - why: The row follows the rule.

**remaining_doubt**: After the edits the finder's program is refused at the declaration (E5002, `safe fn netq_drain` with a pointer-returning callback) and again at the coercion of the safe `scratch` (E5042); the honest program writes `unsafe fn netq_drain`, an `unsafe fn scratch` whose body returns memory that outlives the call, and the call in `unsafe:` with a note. Nothing runs for any of it.

**REFUTER reason**: The text accepts the program. `[FFI-10]`: "safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete", and its refusal reaches only the top of the foreign signature: "complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`". `[FFI-21]`: "A function-pointer parameter of a `safe fn` or of an overlay entry carries a `callback(…)` word, or the declaration is `E5002`" and "The callable type of a `callback(…)` parameter is `fn(P…) -> R`: ... a raw pointer among them staying raw", nothing on `R`. `[FFI-21a]` refuses only "a raw pointer into such a parameter". §IV.5: "Creating one from a reference is safe (ODR-075)". `[FN-6]`: "A function that captures nothing also coerces to `extern \"C\" fn(…)` when every type is FFI-safe". So `scratch` returns a pointer to its own frame and C writes 1,500 bytes into it: a use after free, which `[PHIL-10]` denies. The document's own policy decides the fix: `[FFI-2]` "The safe-to-call fact is never derived, because a scalar can be an address" and `[FFI-10]`'s refusal of a raw pointer at the top both refuse an assertion that safe code can falsify; a pointer-returning callback type, a raw pointer or an `unsafe` callable in a `@layout(c)` field at depth are the same property one level down. The reader's four edits are at the cause, cost nothing at run time and keep verdicts 133 and 136. One hole in the proposed text: the §XVI.4 example breaks under the `[FFI-10]` edit (fixed above). A residual the reader did not raise: a `callback(…)` function type's own parameters are mapped raw ("a raw pointer among them staying raw"), so a pointer C passes in is used in `unsafe:`; nothing more is needed there.

- BETTER EDIT rule=§XVI.4 overlay example
  - old:     @ffi(effects=[FFI, Alloc], no_callbacks)
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - new:     @ffi(effects=[FFI, Alloc], no_callbacks)
    @safety("`pCreateInfo` points to a `VkBufferCreateInfo` whose `pNext` chain and `pQueueFamilyIndices` (`queueFamilyIndexCount` entries) are valid for the call, and `pAllocator` is null or holds live allocation functions")
    unsafe fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status
  - why: Under the reader's `[FFI-10]` edit this entry is `E5002`: `VkBufferCreateInfo` holds `pNext` and `pQueueFamilyIndices`, raw pointers at depth, and `VkAllocationCallbacks` holds C function pointers, so the document's own flagship example would no longer compile (the brief: examples compile). The honest entry is `unsafe fn` with `@safety`, as `[FFI-2]` and `[GRM-35]` already allow. The better shape, for the lead to weigh, is a `type` entry whose fields take `[FFI-11]`'s words (`pQueueFamilyIndices: borrowed count(queueFamilyIndexCount)`), under which a contracted pointer field is the safe type the words produce and the aggregate stays safe to expose; an uncontracted one makes it `E5002`. That keeps Vulkan-style APIs safe to call and is at the same cause.

## NG-12  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-56: "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)`"; FN-6: "A function coerces to any callable type it satisfies, and to its `unsafe` form; an `unsafe fn` coerces only to an `unsafe` callable type (`E2020` otherwise)."; UNS-1: "call an `unsafe fn` ..., or through a value of an `unsafe` callable type (`[FN-6]`)"; FFI-2: "The safe-to-call fact is never derived, because a scalar can be an address (`void free_addr(uintptr_t)`): it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an overlay entry for the function, which asserts that a call within its contract cannot break memory safety"; FFI-10: "safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"

**program**: Compiles with no `unsafe:`. `job.run` imports as `unsafe extern "C" fn(usize)` ([FFI-56]); `run=jobs.pool_release` is a coercion of an `unsafe fn` to an `unsafe` callable type, allowed in safe code ([FN-6]), and only a call through it is unsafe ([UNS-1]); the overlay entry `fn jobs_run_now(j: borrowed one)` asserts the call safe ([FFI-2]), and `job` is a `@layout(c)` value type, which [FFI-10] admits. C calls `pool_release(4096)`, whose `@safety` nobody vouched: an invalid free from Safe Ember. The property is [FFI-10]'s own for raw pointers ('a scalar can be an address'): safe code chooses which unsafe function C calls, with arguments nothing checks; the current words list raw pointers only, and only at the top of a parameter's type.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: (the edit given under NG-11: a raw pointer or a value of an `unsafe` callable type, held anywhere in a parameter's or result's type, is never safe to expose; every asserting declaration with one is `E5002`, with the `pool_run` message)
  - why: Same cause as NG-11, NG-13, NG-15, NG-16: the depth and the kind of value were missing from [FFI-10]'s property.

**remaining_doubt**: A C API whose struct carries a C function pointer is then used through an `unsafe fn` entry and an `unsafe:` block with a note, as a raw-pointer API is today; an Ember function in such a field is a safe `extern "C" fn` and stays safe. Rust's 2024 `safe fn` in `unsafe extern` leaves this to the asserter; the document already refuses the raw-pointer case that Rust also leaves to the asserter, so the edit follows the document's own line.

**REFUTER reason**: `[FFI-56]`: "A function pointer that comes from C — a field of an imported struct, ... — imports as `unsafe extern \"C\" fn(…)`"; `[FN-6]`: "an `unsafe fn` coerces only to an `unsafe` callable type", a coercion safe code may write; `[UNS-1]` makes unsafe only the call "through a value of an `unsafe` callable type"; `[FFI-10]` admits "a `@layout(c)` value type" whole, and `[FFI-2]` "an overlay entry for the function, which asserts that a call within its contract cannot break memory safety" makes `fn jobs_run_now(j: borrowed one)` safe to call. So safe code chooses which `unsafe fn` C calls and with what: `[FFI-2]`'s own reason ("a scalar can be an address") one level down. The reader's single `[FFI-10]` edit (NG-11) covers it; nothing runs for it. The honest API is an `unsafe fn` entry and an `unsafe:` block with a note, as a raw-pointer API is today.

## NG-13  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-17a table row (std::function): "with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function ... never a parameter type (`E5030`)"; FFI-8 table row: "| `T*`, `const T*` | `*mut T`, `*T`; with a contract (`[FFI-11]`), a parameter `x: T` or `ref mut x: T`, `Span[T]`/`MutSpan[T]`, `Option[…]`, `owns T`, `CStrView` |"; FFI-10: "complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`"

**program**: Compiles. `BlockFn` becomes a safe function value `fn(Block)`; nothing judges `A…`: `Block.samples` maps to `*mut f32` ([FFI-8]), safe code fills it with `ref_to_ptr(ref mut one)`, and the call `gain(...)` is a call of a safe function value, so C++ scales 4,096 floats from `one`. [FFI-10]'s E5002 names `safe fn` only, and neither [FFI-2] nor [FFI-56] names the `std_function` entry, which is a third kind of declaration that makes foreign code callable from safe code.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: (the edit given under NG-11, which names `@ffi(std_function)` on a type among the asserting declarations and reaches a field of a `@layout(c)` value at any depth)
  - why: Same cause.

- READER EDIT rule=FFI-17a table row (std::function)
  - old: (`[FFI-17f]`); never a parameter type (`E5030`) |
  - new: (`[FFI-17f]`); its parameter types and its result are judged as a `safe fn`'s are, one holding a raw pointer or an `unsafe` callable at any depth being `E5002` on the entry (`[FFI-10]`), since a call of the value is safe code; never a parameter type (`E5030`) |
  - why: The row names the check at the entry an implementer reads for `std_function`.

**remaining_doubt**: A `std::function<void(Block)>` with a pointer-bearing `Block` is then imported without `@ffi(std_function)`, an opaque object called through an `unsafe fn` entry; or the overlay gives `Block.samples` a contract through a typedef, which [FFI-11] does not yet offer for a struct field. The feature cost is the owner's to weigh; safety is not.

**REFUTER reason**: `[FFI-17a]` row: "with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function"; `[CLO-3]` makes a call through such a value ordinary safe code; `[FFI-8]` maps `float*` to `*mut f32`, so `Block.samples` is a raw pointer safe code fills with `ref_to_ptr(ref mut one)` (§IV.5 "Creating one from a reference is safe"); `[FFI-10]`'s E5002 names only `safe fn`, and neither `[FFI-2]` nor `[FFI-56]` names the `std_function` entry, a third declaration that makes foreign code callable from safe code. C++ reads 4,096 floats from one: out of bounds from Safe Ember. The reader's edits (the `[FFI-10]` text naming `@ffi(std_function)` and the row's own sentence) are at the cause and cost nothing. The feature left is the opaque object called through an `unsafe fn` entry, or field contracts if the lead takes NG-11's better shape.

## NG-14  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_proposed_text

**answered_by**: §IV.5 Raw types: "The raw pointer keeps no compiler-tracked view. Dereferencing, offsetting, reading and writing through it need `unsafe`."; FFI-56: "The end of an `owns T` foreign value calls the destructor its asserted `destroys(f)` names, which the compiler generates."; FFI-8 table row: "| complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |"; FFI-36a: "`std.ffi.adopt[T](h) -> owns T` is an `unsafe fn` that takes ownership of a foreign object, reading its destructor from the contract of `T`'s declaration, `@ffi(destroys(f))` on `type T`; the `owns T` value's end calls `f` once."; TYP-14: "A reference, and an `owns T`, is read through wherever a `T` is wanted — a field access, a method call, an operand, an argument to a plain parameter."

**program**: Compiles and frees `first`'s bytes twice. The `unsafe:` block in `make` vouched for the adoption as made ([FFI-36a]); afterwards `second.data = first.data` is a write of a raw-pointer *value* into a field of an imported struct through an `owns T` ([TYP-14]), which no rule makes unsafe (§IV.5 reserves `unsafe` for writes *through* the pointer); the compiler-generated end then runs `blob_free(second)`, which frees `first.data`, and `blob_free(first)` frees it again. The memory of an imported aggregate is memory C reads later (its destructor here, a table or a later call elsewhere), and a raw-pointer field of it is the one field whose value C dereferences; the row for unions already treats fields C's view of memory makes dangerous as unsafe to read, and the same is needed for writing a pointer field.

- READER EDIT rule=FFI-8 table row (complete struct, union)
  - old: | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |
  - new: | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe`; a field of a raw-pointer type or of a C function-pointer type (`[FFI-56]`) is read in safe code and written only in `unsafe:` (`E3100`: "`second.data = first.data` (line 14) writes a raw pointer into a `blob`, which C reads later (`blob_free`, its destructor, `[FFI-36a]`); put line 14 in an `unsafe:` block with a `# SAFETY:` note saying what C may do with that address"), since an imported aggregate is memory foreign code reads, and what it does with the address nothing checks |
  - why: The write is the point where safe code chooses the address C will act on; making it an unsafe operation puts the vouching where the choice is, as a write through a pointer is already.

- READER EDIT rule=UNS-1
  - old: 
  - new: write a raw-pointer or C function-pointer field of an imported C or C++ aggregate (`[FFI-8]`), whose memory foreign code reads;
  - why: [UNS-1] is the list of unsafe operations an implementer compiles E3100 from.

**remaining_doubt**: An Ember-declared `@layout(c)` struct with a pointer field is not an imported aggregate; it reaches C only through a declaration, where [FFI-10]'s edit (NG-11) judges it, or through a `ref mut` given to an `unsafe fn`, where the caller's block vouches. Reading such a field stays safe (a value, not a dereference).

**REFUTER reason**: The hole is real: `[FFI-36a]` "the `owns T` value's end calls `f` once" and `[FFI-56]` "The end of an `owns T` foreign value calls the destructor its asserted `destroys(f)` names, which the compiler generates"; `[TYP-14]` "A reference, and an `owns T`, is read through wherever a `T` is wanted — a field access"; §IV.5 reserves `unsafe` for "Dereferencing, offsetting, reading and writing through it", and `[FFI-8]`'s row makes only a union's fields unsafe to read. So `second.data = first.data` is safe, and the two generated `blob_free` calls free one pointer twice: a double release `[PHIL-10]` denies, from a program whose only `unsafe:` vouched for the adoption. The reader's fix is at the right place (the write is where safe code chooses the address C acts on) and costs nothing, but misses the whole-value write of a `Copy` aggregate holding the pointer; the edits above close it. The broad scope (every imported aggregate, not only one under a destructor) is needed for NG-20's route, where C's own memory is written through a view.

- BETTER EDIT rule=FFI-8 table row (complete struct, union)
  - old: | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |
  - new: | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe`; a field of a raw-pointer type or of a C function-pointer type (`[FFI-56]`) is read in safe code and written only in `unsafe:`, and so is the whole value of an aggregate that holds one, written into a place foreign code reads (through a `ref mut` to it, `mem.swap`, `mem.replace`) (`E3100`: "`second.data = first.data` (line 14) writes a raw pointer into a `blob`, which C reads later (`blob_free`, its destructor, `[FFI-36a]`); put line 14 in an `unsafe:` block with a `# SAFETY:` note saying what C may do with that address"), since an imported aggregate is memory foreign code reads, and what it does with the address nothing checks |
  - why: The reader's rule names field writes only; a `@layout(c)` struct whose fields are pointers and numbers is `Copy` (the marker table: `Copy` "always for scalars, `ref T`, `Span`, `StrView`, raw pointers, `extern fn`"), so `b = ref mut second` read through `[TYP-14]` then `b = copy of first` writes the same pointer into `second` as a whole value, with no field named, and the destructor frees it twice. The write of the whole value is the same choice of an address C will act on.

- BETTER EDIT rule=UNS-1
  - old: 
  - new: write a raw-pointer or C function-pointer field of an imported C or C++ aggregate (`[FFI-8]`), or a whole value holding one into such an aggregate, whose memory foreign code reads;
  - why: `[UNS-1]` is the list an implementer compiles `E3100` from; it must name the whole-value write too.

## NG-15  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-8 table row (typedef): "| `typedef` | a type alias; with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only |"; TYP-7: "`as` between pointer types, or between a pointer and an integer, requires `unsafe`."; TYP-6: "`as` between any other pair of types is `E2020`, naming both."; FFI-10: "safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"

**program**: Undecided in the current text. `null[*mut tx.tex]() as tx.tex_ref`: the newtype row allows `as` from the representation, [TYP-7] makes `as` between pointer types unsafe, [TYP-6] makes every unlisted pair E2020, and none of the three says whether a nominal type over a pointer typedef is 'a pointer type' for [TYP-7], 'a scalar' for [FFI-10], or a raw pointer for [THR-8], [THR-9] and [TYP-13]. One compiler accepts the line in safe code and the entry `fn tex_draw(cmd)` (`draw_cmd` being a `@layout(c)` value type), after which C reads through a null or stale handle; another refuses it. Both are conforming today.

- READER EDIT rule=FFI-8 table row (typedef)
  - old: | `typedef` | a type alias; with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only |
  - new: | `typedef` | a type alias; with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only, under its representation's rules (`as` from or to a pointer representation is in `unsafe:` only, `[TYP-7]`); for `[FFI-10]`, `[THR-8]`, `[THR-9]` and `[TYP-13]` the type is what its representation is, so a newtype over a pointer is a raw pointer, however it is named |
  - why: A name over a pointer must not change what the pointer is for the safety rules; this says so once, where the newtype is defined.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: (the edit given under NG-11, which names 'the representation of a `@ffi(newtype)` type' among the places a raw pointer is held)
  - why: Same cause as NG-11 to NG-16.

**remaining_doubt**: A handle C hands out and takes back is the `@ffi(handle)` word's job ([FFI-8]: 'an opaque pointer-sized `Copy` value', never dereferenced by Ember, convertible from nothing), which stays safe to expose; the newtype is for a representation safe code may build, and so is judged by it.

**REFUTER reason**: Undecided as the reader says. `[FFI-8]` row: "with `@ffi(newtype)` a nominal type with the typedef's representation, passed to C as the typedef is, converted with `as` only"; `[TYP-7]` "`as` between pointer types, or between a pointer and an integer, requires `unsafe`"; `[TYP-6]` "`as` between any other pair of types is `E2020`, naming both"; `[FFI-10]` "a scalar, a `@layout(c)` value type, or a pointer whose contract is complete". None says whether a nominal type over a pointer typedef is 'a pointer type', 'a scalar' or a raw pointer for `[THR-8]`, `[THR-9]` and `[TYP-13]`, so one conforming compiler accepts `null[*mut tx.tex]() as tx.tex_ref` in safe code and the entry `fn tex_draw(cmd)`, after which C reads through a null handle. The reader's edit (the type is what its representation is, for every safety rule, said once where the newtype is defined) is at the cause and costs nothing; `@ffi(handle)` (`[FFI-8]`: "an opaque pointer-sized `Copy` value" that safe code cannot build) stays the safe spelling for a handle C hands out.

## NG-16  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-10: "safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the declaration), `safe` on any other function being `E5002`"; FN-6: "A function coerces to any callable type it satisfies, and to its `unsafe` form; an `unsafe fn` coerces only to an `unsafe` callable type (`E2020` otherwise)."; UNS-1: "call an `unsafe fn` (`Allocator.deallocate` and `reallocate` among them, `[ALC-2]`), or through a value of an `unsafe` callable type (`[FN-6]`)"; FFI-2: "The safe-to-call fact is never derived, because a scalar can be an address (`void free_addr(uintptr_t)`)"

**program**: Compiles. `PoolConfig` is a `@layout(c)` value type, so `safe fn pool_run(cfg: PoolConfig)` passes [FFI-10]; `reap=free_addr` coerces an `unsafe fn` to the field's `unsafe extern "C" fn(usize)` in safe code ([FN-6]), and only a call is unsafe ([UNS-1]); C then calls `free_addr` on its own arena addresses, which `alloc_addr` never gave: heap corruption from Safe Ember. Same property as a raw pointer ([FFI-2]'s own reason), by a value [FFI-10] does not list, at a depth it does not look.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: (the edit given under NG-11, whose second message is this program's: "`pool_run`'s `cfg` holds `reap`, an `unsafe extern "C" fn(usize)`, so safe code chooses which C function `pool_run` calls, and with what; write `unsafe fn pool_run`, with `@safety("…")` saying what `reap` must do")
  - why: Same cause as NG-12.

**remaining_doubt**: None beyond NG-12's.

**REFUTER reason**: Same property as NG-12 by the by-value route: `[FFI-10]` "safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete" admits `PoolConfig` whole; `[FN-6]` "an `unsafe fn` coerces only to an `unsafe` callable type" lets safe code write `reap=free_addr` into a field `[FFI-56]` imports as `unsafe extern \"C\" fn(usize)`; `[UNS-1]` makes only the call unsafe. C then calls `free_addr` on addresses `alloc_addr` never gave: heap corruption from Safe Ember, by `[FFI-2]`'s own reason ("a scalar can be an address"). The one `[FFI-10]` edit of NG-11 closes it, with the `pool_run` message.

## NG-17  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-21a: "a function whose value has that type MUST NOT return a raw pointer into such a parameter ... A value read through the parameter, and a pointer stored in it, are not into the copy. A pointer returned from a call to another function is not followed."; FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe; it then gets a C entry that attaches the calling thread (`[FFI-22]`) and runs it"; FFI-10: "complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`"

**program**: The same shape as NG-11 (`read_packet(scratch, 1500)` with `callback(borrowed)` and a `*mut u8` result): compiles, and C writes 1,500 bytes into `scratch`'s finished frame. [FFI-21a] names only a pointer into a copy parameter; `byte` is a local. The finder is right that the earlier edit's ODR-075 line ('closes the one route by which its result reached C from safe code') is not true of the current text, which has no such clause at all.

- READER EDIT rule=FFI-10 and FFI-21a
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a
  raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: (the two edits given under NG-11: a pointer-returning callback type is accepted on an `unsafe fn` declaration only, E5002 on a `safe fn`; and an Ember function that returns a raw pointer to foreign code is an `unsafe fn`, E5042 otherwise)
  - why: Same cause as NG-11.

**remaining_doubt**: None beyond NG-11's.

**REFUTER reason**: The same shape as NG-11 and decided the same way: `[FFI-21a]` "MUST NOT return a raw pointer into such a parameter ... A pointer returned from a call to another function is not followed" names only a pointer into a copy parameter, and `byte` is a local; `[FN-6]` coerces `scratch`; `[FFI-10]`'s E5002 does not look into the callback's result. The finder is right that no ODR-075 clause about 'the one route by which its result reached C' is in the current text. NG-11's two edits refuse it at the declaration (`E5002`) and at the coercion (`E5042`); nothing runs for either.

## NG-18  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-36a: "`std.ffi.adopt[T](h) -> owns T` is an `unsafe fn` that takes ownership of a foreign object, reading its destructor from the contract of `T`'s declaration, `@ffi(destroys(f))` on `type T`; the `owns T` value's end calls `f` once."; FFI-56: "The end of an `owns T` foreign value calls the destructor its asserted `destroys(f)` names, which the compiler generates."; §IV.5 Raw types: "Creating one from a reference is safe (ODR-075) ... The raw pointer keeps no compiler-tracked view."; FFI-8 table row: "| complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |"

**program**: Compiles. `canvas.pixels = ref_to_ptr(ref mut stray)` is a safe write of a pointer value into a field of the imported `SDL_Surface`; when `canvas` ends the compiler calls `SDL_FreeSurface`, which frees `pixels`, now a stack byte. The `unsafe:` in `new_canvas` vouched for the adoption, not for later writes; the route to C is memory C reads later, not a call's argument. Same cause as NG-14.

- READER EDIT rule=FFI-8 table row (complete struct, union) and UNS-1
  - old: | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |
  - new: (the edit given under NG-14: a raw-pointer or C function-pointer field of an imported aggregate is written only in `unsafe:`, E3100 with the message naming the destructor that reads it)
  - why: Same cause as NG-14.

**remaining_doubt**: None beyond NG-14's.

**REFUTER reason**: Same cause as NG-14: `[FFI-36a]` "the `owns T` value's end calls `f` once"; §IV.5 "Creating one from a reference is safe (ODR-075)"; `[FFI-8]`'s struct row makes no field write unsafe. `canvas.pixels = ref_to_ptr(ref mut stray)` is a safe write of a pointer value into memory `SDL_FreeSurface` reads at `canvas`'s end, which then frees a stack byte. NG-14's edits, in the form given there (field writes and whole-value writes of an aggregate holding a pointer, `E3100`), close it; the `unsafe:` in `new_canvas` vouched for the adoption only, as the reader says.

## NG-19  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-39: "the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly the virtual methods it overrides (`override fn`), read from the class"; FFI-40: "A `T&` parameter imports as `ref mut x: T`, `const T&` as a plain parameter `x: T`"; FFI-2: "it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an overlay entry for the function, which asserts that a call within its contract cannot break memory safety"; FFI-39c: "a parameter borrowed for the call alone takes the object, or `this`, as any plain parameter, upcast to the C++ base (ODR-156)"; §III.2 grammar: "fn_header     := ["extern" STRING] ["unsafe"] ["static" | "virtual" | "override"] ["gen"]"

**program**: Compiles. The entry `fn decode_frame(arena)` asserts the call safe; `Arena&` is `ref mut arena: codec.Arena` and `StackArena` upcasts to it; the trampoline hands C++ the override `grab`, whose `*mut u8` result nobody judges, and `ref_to_ptr(ref mut one)` is safe. C++ writes 4,096 bytes into `grab`'s finished frame. The current text has no rule for what an override returns to C++; refusing the `decode_frame` entry would refuse it for a C++-made `Arena` too, so the place to judge is the override, as the finder says.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that
  type MUST NOT return a raw pointer into such a parameter:
  - new: (the edit given under NG-11, which names 'an `override fn` of a C++ virtual method (`[FFI-39]`)' among the functions that return a raw pointer to foreign code only as an `unsafe fn`, E5042 at the `override fn` otherwise)
  - why: Same cause as NG-11, by the override route.

- READER EDIT rule=FFI-39
  - old: 
  - new: An `override fn` whose result is a raw pointer, or holds one, is written `unsafe override fn` (the grammar's order, §III.2), its body unsafe code that vouches for where the pointer points, since C++ reads through it (`[FFI-21a]`, `E5042` on a safe one).
  - why: Says it where an implementer of the trampoline reads.

**remaining_doubt**: An override written to return a view (`MutSpan[u8]`) under a lifetime word the overlay gives the base's virtual method would be the safe spelling; [FFI-11]'s words are not yet written for a virtual method's result, so the `unsafe override fn` is the complete answer today.

**REFUTER reason**: `[FFI-39]`: "the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly the virtual methods it overrides (`override fn`), read from the class ... An `override fn` matching no virtual method of the base, by name and signature, is `E5056`", so a virtual returning `uint8_t*` is overridden by `override fn grab(n: usize) -> *mut u8`, whose body is safe code and whose `ref_to_ptr(ref mut one)` is safe (§IV.5); `[FFI-2]` makes the entry `fn decode_frame(arena)` safe to call and `[FFI-40]` maps `Arena&` to `ref mut arena: codec.Arena`, with the upcast of `[FFI-39c]`. No rule judges what an override returns to C++, and refusing the entry would refuse it for a C++-made `Arena` too, so the override is the place. The grammar (§III.2 `fn_header := ["extern" STRING] ["unsafe"] ["static" | "virtual" | "override"] ["gen"]`) already admits `unsafe override fn`; the reader's two edits (NG-11's `[FFI-21a]` text naming the override route, and the sentence in `[FFI-39]`) cost nothing and are at the cause.

## NG-20  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-56: "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)`"; FN-6: "Where an `unsafe fn(…)` is wanted, any function or lambda that meets `fn(…)` may be given, and so may an `unsafe fn`."; FFI-11 table (lifetime): "a view into the receiver, into `p`, or into what lives for the whole program (`[LT-1a]`, `[LT-3]`)"; FFI-52: "every Ember function given to C as an `extern "C" fn` value"

**program**: Compiles. `net_hooks_get` gives a `ref mut net_hooks`; `hooks.grab` imports as `unsafe extern "C" fn(usize) -> *mut u8` ([FFI-56]); the safe `scratch` coerces to it ([FN-6]); neither entry carries a raw pointer. `net_recv` writes 1,500 bytes into `scratch`'s finished frame. Two routes compose: a function given to C through a field write into memory C reads, and a raw pointer that function returns.

- READER EDIT rule=FFI-8 table row (complete struct, union)
  - old: | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union, `@layout(c, union) struct`, whose fields are read only in `unsafe` |
  - new: (the edit given under NG-14: writing a C function-pointer field of an imported aggregate is in `unsafe:` only, E3100, so `hooks.grab = scratch` needs a note)
  - why: The field write is where the function reaches memory C reads.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that
  type MUST NOT return a raw pointer into such a parameter:
  - new: (the edit given under NG-11: `scratch`, stored where C reads it, becomes an `extern "C" fn` with a raw-pointer result and must be an `unsafe fn`, E5042 at `hooks.grab = scratch` otherwise)
  - why: Same cause as NG-11, by the composed route.

**remaining_doubt**: With both edits the honest program has an `unsafe fn scratch` returning memory that outlives the call, stored in an `unsafe:` block whose note says what `net_recv` does with it.

**REFUTER reason**: Two routes compose and both are open: `[FFI-56]` imports `hooks.grab` as `unsafe extern \"C\" fn(usize) -> *mut u8`; `[FN-6]` "Where an `unsafe fn(…)` is wanted, any function or lambda that meets `fn(…)` may be given" lets the safe `scratch` be stored; `[FFI-11]`'s lifetime word gives `net_hooks_get` a `ref mut` into C's memory; `[FFI-8]`'s row makes the field write safe. `net_recv` writes into `scratch`'s finished frame. NG-11's `[FFI-21a]` edit refuses `scratch` where it becomes an `extern \"C\" fn` (`E5042`); NG-14's edit is what refuses the other program on this route, `hooks.release = free_addr` (an `unsafe fn` coerced in safe code into memory C reads), which no declaration judges, so both edits are needed, as the reader says.

## NG-21  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-31b: "`@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may."; FFI-31b: "An export's parameters and result are written in the safe types of `[FFI-10]`'s mapping and lowered to the same carriers"; FFI-26: "So both attributes are written in their unsafe form, `@unsafe(export("game_on_update"))` and `@unsafe(export_table("Name", protocol=N))`, with a `# SAFETY:` note above saying why nothing else defines that name"; FFI-21a: "a function whose value has that type MUST NOT return a raw pointer into such a parameter"

**program**: Undecided, and unsafe on the reading that compiles. [FFI-31b] lists what may and may not cross an export and names raw pointers in neither list ('reachable through a pointer in one' presumes they may); the export's body is safe code, `ref_to_ptr(ref mut byte)` is safe, and the `# SAFETY:` note of `@unsafe(export)` vouches for the symbol's name only ([FFI-26]). The host writes 1,500 bytes into `scratch`'s finished frame. An export is one of [FFI-52]'s ways for C to reach an Ember function, and no declaration of the callback kind covers it.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that
  type MUST NOT return a raw pointer into such a parameter:
  - new: (the edit given under NG-11, which names 'an `@unsafe(export)` or `@export_table` function (`[FFI-26]`)' among the functions that return a raw pointer to foreign code only as an `unsafe fn`, E5042 at the export attribute otherwise)
  - why: Same cause as NG-11, by the export route.

- READER EDIT rule=FFI-31b
  - old: `@layout(c)` values, scalars, opaque handles,
  `CStrView` and `extern "C" fn` may.
  - new: `@layout(c)` values, scalars, opaque handles,
  `CStrView` and `extern "C" fn` may; a raw pointer may appear in an export's result only when the
  exported function is an `unsafe fn`, whose body vouches for where it points (`[FFI-21a]`, `E5042`
  otherwise), and in an export's parameter it is read through only in unsafe code, as any raw pointer
  is (`[UNS-1]`).
  - why: Says it in the export rule an implementer reads for the exported signature.

**remaining_doubt**: A `@export_table` field is a function of the field's type ([FFI-26]), so the same words reach it through 'an `@export_table` function'. A C++ override is NG-19's.

**REFUTER reason**: `[FFI-31b]` lists what MUST NOT cross ("class objects, `owns T`, `shares T`, `thread_shared T`, `weakref T`, `String`, `Array`, `Map`, function values and any type with a `drop`") and what may ("`@layout(c)` values, scalars, opaque handles, `CStrView` and `extern \"C\" fn` may"), names raw pointers in neither, and says "An export's parameters and result are written in the safe types of `[FFI-10]`'s mapping", while `[FFI-10]` says "complete facts do not make a raw-pointer parameter or result safe to expose": the words lean to refusal but no code names a raw pointer in an export, and "reachable through a pointer in one" presumes pointers may appear, so two conforming compilers differ; on the accepting one the host writes into `scratch`'s finished frame, with `[FFI-26]`'s `# SAFETY:` note vouching for the symbol's name only. The reader's edits (an `unsafe fn` export may return a raw pointer, its body vouching, `E5042` otherwise; a raw pointer in an export's parameter read through only in unsafe code, as `[FFI-21]` already does for a callback's) are at the cause, keep the feature a host needs (a buffer returned from a `static`) and cost nothing.

## NG-22  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: THR-15: "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`), so a one-thread owner never crosses through a channel: an end of a channel of such a `T` given to another thread is `E7015` at that line, as any value that is not `Send` (`[THR-2]`)."; THR-8: "nothing reachable from it (its fields, its elements, what its `owns` parts hold, a function value's state) is a one-thread owner (`shares T`, `shares mut T`)"; THR-10: "so a function that holds a one-thread value is `E7015` (`[THR-1]`, `[THR-2]`), and `R` MUST be `Send` (`E7015`, `E7008`)"

**program**: 

**remaining_doubt**: The finder's fix (J7-07) is the current [THR-15] (ruling N80): `Receiver[shares Node]` is not `Send`, so `thread.spawn(owns fn() owns rx: count(rx))` is E7015 at that line. None.

**REFUTER reason**: `[THR-15]`: "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`), so a one-thread owner never crosses through a channel: an end of a channel of such a `T` given to another thread is `E7015` at that line"; `[THR-8]` makes `shares Node` not `Send`; `[THR-10]` "a function that holds a one-thread value is `E7015`". `thread.spawn(owns fn() owns rx: count(rx))` is E7015 at that line (ruling N80). Nothing left.

## NG-23  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: §III.2 grammar: "bound_list    := type {"+" type}                        -- after `dyn` and `some` only"; TYP-5 rule 8: "a value to an interface its type implements, keeping the word that says who owns it: a value read in place to `dyn I`, an `owns T` to `owns dyn I`, a `shares T` to `shares dyn I`, a `shares mut T` to `shares mut dyn I` (`[TYP-45]`)"; THR-8: "`dyn I` when `I` requires it (`interface I: needs Send:`)"; TYP-17: "A bound without `needs`, or needs joined with `+`, is `E0100`, whose message gives the line with `needs`"; §IV.9: "`dyn I` is unsized. ... It is a data pointer and a table pointer"

**program**: The grammar admits `owns dyn Command + Send` (a `bound_list` after `dyn`), and no rule says what it means: [TYP-5] rule 8 converts to a `dyn` of one interface, [THR-8] gives `dyn I` its markers through `needs` on the interface, [TYP-17] refuses `+` in a type parameter's bounds, and §IV.9 gives `dyn I` one table pointer. An implementer cannot say whether `cmd: owns dyn Command + Send = owns Clear(color=3)` converts, which methods `cmd` may call, whether `cmd` is `Send`, or how many table pointers it carries; std's `Box[dyn Error + Send + ThreadShared]` rests on the answer. Not a memory-safety hole; an undecidable line.

- READER EDIT rule=TYP-5 rule 8
  - old: 8. a value to an interface its type implements, keeping the word that says who owns it: a value
     read in place to `dyn I`, an `owns T` to `owns dyn I`, a `shares T` to `shares dyn I`, a
     `shares mut T` to `shares mut dyn I` (`[TYP-45]`);
  - new: 8. a value to an interface its type implements, keeping the word that says who owns it: a value
     read in place to `dyn I`, an `owns T` to `owns dyn I`, a `shares T` to `shares dyn I`, a
     `shares mut T` to `shares mut dyn I` (`[TYP-45]`); a `dyn` list (`bound_list`, §III.2) names
     one interface and any number of the marker interfaces (`Send`, `ThreadShared`, `[TYP-43]`), and
     a value converts to it when its type implements the interface and has each marker (`owns
     Clear(color=3)` to `owns dyn Command + Send` when `Clear` is `Send`); such a value calls the
     interface's methods, has the markers, meets a bound that asks for either, and converts to the
     `dyn` of the interface alone; two interfaces with methods in one list are `E2050`, whose help
     names an interface that requires both ("write `interface SendCommand: needs (Command, Send):`
     and `owns dyn SendCommand`"), so a `dyn` value stays one data pointer and one table pointer;
  - why: Decides the conversion, the method set and the layout in one place, with one table pointer as §IV.9 states, as Rust's `dyn Trait + Send` does.

**remaining_doubt**: If the owner prefers two interfaces in one `dyn`, the layout grows a table pointer per interface and §IV.9 and the `Interface call through dyn` cost row must say so; the marker-only list is the smaller change and what std's `From` needs.

**REFUTER reason**: Confirmed as an undecidable line: §III.2 "bound_list := type {\"+\" type} -- after `dyn` and `some` only" admits `owns dyn Command + Send`; `[TYP-5]` rule 8 converts only "an `owns T` to `owns dyn I`"; `[THR-8]` gives `dyn I` its marker only "when `I` requires it (`interface I: needs Send:`)"; `[TYP-17]`'s E0100 for `+` is for a type parameter's bounds; §IV.9 "It is a data pointer and a table pointer". So whether `cmd` converts, what it may call, whether it is `Send` and how wide it is are all open; no safety is at stake since nothing compiles two ways unsafely. The reader's edit (one interface plus marker interfaces, one table pointer, two method interfaces refused with the `needs` form as the help) is the smaller change and what std's `From` needs; narrowed only for the registry row above. The same `bound_list` after `some` is undecided too and the reader did not raise it: under missed.

- BETTER EDIT rule=registry row E2050
  - old: | `E2050` | interface is not `dyn`-compatible | `[IFC-5]`, `[TYP-22]` |
  - new: | `E2050` | interface is not `dyn`-compatible; two interfaces with methods in one `dyn` list (`dyn A + B`), whose help names an interface that requires both | `[IFC-5]`, `[TYP-22]`, `[TYP-5]` |
  - why: The reader's edit gives `E2050` a second meaning without touching the registry; one code, one defining rule means the row names it and the rule that defines it.

## NG-24  reader=answered  refuter=narrowed  should_be=answered  safety=hole_in_proposed_text

**answered_by**: THR-2: "A line that makes a value `thread_shared` (`t = thread_shared Tree(items=[item])`, `[THR-1]`), and a store under a lock into a field of one (`t.items.push(owns item)`, `[THR-18]`), makes every class object the stored value owns reachable from any thread, and is such a hand-over for each of them: the compare is made there, so a `weakref T` link made before it (`w = weakref item`) finds its object ended afterwards, and no plain weak count is ever touched from two threads."; RC-4 point 4: "when it stores an owner of a `thread_shared` value where other threads can reach it (into any place reachable from a `thread_shared` value, a `guarded by` field or container under its lock; through a channel; ...; every such place is known when compiling)"; THR-2: "An object handed whole to another thread, a scoped task or a job included, leaves its `weakref` links behind (verdict 114; Rust's `Rc::try_unwrap`). At the hand-over the program reads the object's weak count, one compare. With no link, the object goes as it is. With a link, its contents move to a new block for the other thread, and the old block is marked ended"; THR-1: "everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`"; THR-9: "`RwLock[T]` and `thread_shared T` for a `T` that is `Send` and `ThreadShared` (several threads hold a `ref T` at once: `RwLock[Cell[int]]` is not `ThreadShared`)"

**program**: 

- READER EDIT rule=THR-1
  - old:   * everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a
    plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`
    ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any
    thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`");
  - new:   * everything reachable from it is `Send` and `ThreadShared` (`[THR-8]`, `[THR-9]`), since any
    thread may hold it and several may read it at once: a one-thread owner, a `weakref T` link, a
    plain `fn()` value, or a value several threads may not read at once (a `Cell`, an
    `RwLock[Cell[int]]`, a `ref mut`) reachable from it is `E7015` at the line that makes it
    `thread_shared`, naming the field and the fix ("`Bus.handlers` holds plain `fn()` values, and a
    `thread_shared Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] = []`
    in `Bus`"; "`FrameStats.draws` is an `RwLock[Cell[int]]`, which several threads may not read at
    once, and a `thread_shared FrameStats` is read from any thread; write `draws: Atomic[int]`");
  - why: Verdict 165 refuses `thread_shared FrameStats()` at its making line for a field that is not `ThreadShared`; [THR-1] asks for `Send` only, and [THR-9] withholds `ThreadShared` from the result without refusing the line.

**departs_from_verdict**: 165: [THR-1]'s second bullet refuses, at the making line, only what is not `Send`; the verdict refuses there whatever is not `ThreadShared` (`draws: RwLock[Cell[int]]`, E7015 naming the field and `Atomic[int]`). The edit above brings [THR-1] to the verdict; NG-24's own program is unaffected (Texture is both).

**remaining_doubt**: For the program: `g.push(owns hero)` through the `RwLock`'s write guard is 'a store under a lock into a field of one' ([THR-2]) and a store 'into ... a container under its lock' ([RC-4] point 4), so it is the hand-over of verdict 114 for `hero` and the objects it owns: the compare finds `preview`'s link, `hero`'s contents move to a new block, the old block is marked ended, and `preview exists as p` prints `gone`; the worker's `g.clear()` touches the new block, which no link on main names, and main's plain weak count is never touched from two threads. Two small wording doubts an implementer might raise: [THR-2]'s 'every class object the stored value owns' should read 'the stored object and every class object it owns', and its parenthetical cites `[THR-18]` (`guarded by`) where a `Mutex`'s or `RwLock`'s guard (`[THR-3]`) is the same place by [RC-4] point 4's words; neither changes the decision.

**REFUTER reason**: The program is answered as the reader says: `[THR-2]` "a store under a lock into a field of one (`t.items.push(owns item)`, `[THR-18]`), makes every class object the stored value owns reachable from any thread, and is such a hand-over for each of them: the compare is made there, so a `weakref T` link made before it (`w = weakref item`) finds its object ended afterwards", and `[RC-4]` point 4 names "a `guarded by` field or container under its lock" as such a place; `preview exists as p` prints gone and no plain weak count is touched from two threads. The reader's departure note is right: verdict 165 "`thread_shared` is refused where the value is made, when something in it is not safe for several threads to read at once (`ThreadShared`): `stats = thread_shared FrameStats()` with `draws: RwLock[Cell[int]]` is `E7015` at that line" is in no rule (`[THR-1]`'s bullet asks `Send` only; `[THR-9]` withholds `ThreadShared` from the result without refusing the line; the E7015 row lists neither). The proposed edit over-refuses a `Mutex[Cell[int]]` component; the edits above bring `[THR-1]` and the registry to the verdict with `[THR-9]`'s own test. Nothing runs for it.

- BETTER EDIT rule=THR-1
  - old:   * everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref T` link, or a
    plain `fn()` value reachable from it is `E7015` at the line that makes it `thread_shared`
    ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any
    thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`");
  - new:   * everything reachable from it is `Send` (`[THR-8]`), and its type is `ThreadShared` (`[THR-9]`:
    every component is, a `Mutex[T]` for a `Send` `T` is, an `RwLock[Cell[int]]` is not), since
    several threads may read it at once (verdict 165): a one-thread owner, a `weakref T` link or a
    plain `fn()` value reachable from it, or a component that is not `ThreadShared`, is `E7015` at
    the line that makes it `thread_shared`, naming the field and the fix ("`Bus.handlers` holds plain
    `fn()` values, and a `thread_shared Bus` may run them on any thread; write `handlers:
    Array[thread_shared fn()] = []` in `Bus`"; "`FrameStats.draws` is an `RwLock[Cell[int]]`, which
    several threads may not read at once, and a `thread_shared FrameStats` is read from any thread;
    write `draws: Atomic[int]`");
  - why: The reader's wording 'everything reachable from it is `Send` and `ThreadShared`' would refuse `Mutex[Cell[int]]` and every `Mutex[T]` whose content is not `ThreadShared`, which `[THR-9]` makes `ThreadShared` ("`Mutex[T]` for a `Send` `T` (one thread at a time holds its `ref mut T`)"); the property verdict 165 asks for is the value's type being `ThreadShared`, judged by `[THR-9]`'s own structural rule, not every reachable component.

- BETTER EDIT rule=registry row E7015
  - old: `thread_shared T using A` for an `A` that is not `ThreadShared`;
  - new: `thread_shared T using A` for an `A` that is not `ThreadShared`; `thread_shared` before a value whose type is not `ThreadShared` (`draws: RwLock[Cell[int]]`), at the making line (verdict 165);
  - why: The row follows the rule.

# MISSED

## MISSED: A method cannot know which manager `this` was made in

**rules**: `[ALC-1]` "The place names the kind, the making line names the instance" and "Without `using`, it comes from the built-in manager, `Global`"; `[CLS-1]` "the block comes from the built-in memory manager, or from the instance the line that makes the object names with `using`"; `[OWN-13]` "`weakref x` ... whose type is read from the words on the path to `x`" and "Inside a method `this` carries no word, so its link says it: `weakref this`"; `[TYP-46]` allows `shares Mesh using LevelArena`; `[OBJ-4]` "An object made in a manager keeps its instance's address only as `[ALC-1]` says: none in a manager that frees nothing one by one, or in one that implements `FoundByAddress`".

**program**:
```
class Node:
    parent: weakref Node = None
    children: Array[Node] = []
    fn add(child: Node):
        child.parent = weakref this
        this.children.push(child)

fn main():
    level = Arena(1 << 16)
    root = Node() using level
    leaf = Node() using level
    root.add(leaf)
    keep = shares root

The class is compiled once; inside `add`, `this` carries no `using`, so `weakref this` (and `shares this`, or `Node() using ?` for a child made inside a method) has no kind to read, and under NG-03's edits the field `parent: weakref Node` names `Global` while the object was made in an `Arena`: the implementer cannot say whether line 5 is `E2020`, nor which give-back the last `shares` owner of `root` runs (`Global`'s free, nothing, a word that is not there). Also `children: Array[Node]` names `Global` for its elements, so `this.children.push(child)` with an arena-made child is `E2020` by `[ALC-1]`'s "the instance is of the kind the place names", and a tree in an arena cannot be written at all.
```

- MISSED EDIT rule=ALC-1
  - old: 
  - new: A class whose objects a program makes `using A` is, with that kind, a type of its own, as `Array[T] using A` is: `Node using Arena` is distinct from `Node`, its methods are compiled for that kind (its summaries recorded per kind, `[EFF-4]`), inside them `this` carries the kind, and a field of the class written with the class's own type (`parent: weakref Node`, `children: Array[Node]`) takes the kind of the object that holds it, so `weakref this`, `shares this` and `Node() using current` in a method all name one kind, read from `this`; a field written `using B` names `B` whatever kind the object is.
  - why: Every other rule reads the kind from a type the line can see; a method's `this` is the one place with no type to read it from, and a tree made in an arena, the first use of an arena, needs it on every line.

## MISSED: A `thread_shared` value made `using System` or a one-instance manager has neither chunk header nor handed-over mark

**rules**: `[OBJ-1]` "the runtime takes the memory for `thread_shared` blocks made in `Global` in chunks from the configured allocator ... writes the making thread into each chunk's header once"; `[RC-4]` point 4 "A `thread_shared` value made `using` an instance of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that makes it ... a value made in `Global` or `System` keeps the two counts, its block waits for the check point"; `[ALC-1]` "A manager type with no fields has one instance (`Global`, `System`, and any such type a program declares): it needs none on the making line"; `[TYP-46a]` "`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making line".

**program**:
```
from std.mem import System

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    st = thread_shared Stats() using System
    h = thread.spawn(owns fn() thread_shared st: st.hits.fetch_add(1))
    h.join()

Under NG-06's allow side (the four rules that describe a `thread_shared` value made `using` an instance), `System` is "a manager with one instance", not one "that may have several instances", so point 4 does not mark the value handed over, and `[OBJ-1]` lays chunks out for `Global` only, so when the thread ends its owner the subtract on the away path has no chunk header to read the making thread from and no mark to say the home count is unused: the value is ended by nobody, or on the wrong count. Under the refuse side `[TYP-46]`'s `E2114` covers it, so the gap belongs to the resolution the lead chooses.
```

- MISSED EDIT rule=RC-4 point 4
  - old: `thread_shared` value made `using` an instance of a manager that may have several instances
  - new: `thread_shared` value made `using` any instance but `Global` (`System` and a manager with one instance included, whose blocks lie in no chunk of `[OBJ-1]`)
  - why: Only a `Global` block has the chunk header; every other block needs the mark.

- MISSED EDIT rule=RC-4 point 4
  - old: a value made in `Global` or `System` keeps the two counts
  - new: a value made in `Global` keeps the two counts
  - why: Same gap.

## MISSED: `some I + Send` has no meaning

**rules**: §III.2 "bound_list := type {\"+\" type} -- after `dyn` and `some` only"; `[TYP-32]` defines `some I` as one concrete type chosen by the function; `[THR-8]` and `[THR-9]` say nothing of `some`; `[TYP-17]` "A bound without `needs`, or needs joined with `+`, is `E0100`" is for type parameters.

**program**:
```
fn evens(xs: Array[int]) -> some Iterator[Item = int] + Send:
    return xs.into_iter().filter(fn(x) => x % 2 == 0)

fn main():
    it = evens([1, 2, 3, 4])
    h = thread.spawn(owns fn() owns it: it.count())
    println(h.join())

The grammar admits the result type; no rule says whether the hidden type must be `Send`, whether `it` may be handed to the thread (the caller sees only the `some` type), or whether `+ Send` is `E0100`. Not a memory-safety hole (the hidden type's own `Send` is known to the compiler), an undecidable line, the twin of NG-23.
```

- MISSED EDIT rule=TYP-32
  - old: 
  - new: A `some` list (`some Iterator[Item = int] + Send`, `bound_list` in §III.2) names one interface and any number of the marker interfaces (`Send`, `ThreadShared`, `[TYP-43]`), as a `dyn` list does (`[TYP-5]` rule 8): the hidden type implements the interface and has each marker, checked at every `return` of the function (`E2040` otherwise, naming the marker and the returned type), and a caller may rely on the markers written and on nothing else of the hidden type.
  - why: One rule for both `bound_list` positions; a caller handing an opaque result to a thread needs the marker in the type it sees.
