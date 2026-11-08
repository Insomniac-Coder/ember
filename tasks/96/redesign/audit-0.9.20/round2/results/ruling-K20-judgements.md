# K20: Can AnyError, Result, a channel or a scoped task carry an error type across threads when that error holds an object of an open class another package may derive from?

### Design 1 summary
A value held by the type of an `open` or `abstract` class is type-erased exactly as `dyn I` and `fn()` are, and the text already decides those two by a word on the type (`[THR-8]`: "`dyn I` when `I` requires it (`interface I: needs Send:`)"; verdict 61: `thread_shared fn()`, the owner choosing the word over "the compiler finding it from the whole program"), and `[MOD-9]` says "Where a word exists to write a fact, only the written word counts across the edge". So the ruling gives the open class's markers a word: an `open` or `abstract` class writes `implements Send, ThreadShared` (the grammar's existing `implements` list, without `unsafe`), its objects held by base type have exactly the markers it declares, and every class derived from it, in any package or reload, meets them with its own fields or is refused where it is declared (`E2040`), while a class that cannot be subclassed stays inferred from its fields. On that footing `Error` requires `Send` and `ThreadShared` (any thread may have an error), so `dyn Error`, `AnyError` and every `Result[T]` cross threads by every route with nothing written, and nothing runs for any of it while the program runs.

### Design 2 summary
An `open` or `abstract` class that another package may derive from says in its own header whether any thread may hold its objects: `open class Script implements Send, ThreadShared:`. Without the word no value of that class type is `Send` or `ThreadShared`, whatever its fields (its value may be an object of a class the package never sees); with it, the promise binds the class and every class derived from it, checked at each class's own declaration in its own package over all its fields (E7015 at the subclass, naming the field), so a dependant's subclass can never break a library's judgement and an upcast needs no check (the shape of `interface I: needs Send` for `dyn I`, of `[OWN-13]`'s method pinning and of `[OBJ-1]`'s family-wide weak count; an open class no other package can subclass is judged from the derived classes its own package sees, as `[EFF-2a]` already does for its virtual calls). On that footing `Error` requires `Send` and `ThreadShared`, so `dyn Error` and `AnyError` are both, and a worker returns `Result[int]`, a channel carries it and a scoped task reads it with nothing written; an error type holding a one-thread part is `E2040` where it implements `Error`, with a copy line. Nothing runs while the program runs.

### Design 3 summary
The rule a newcomer holds: an error is plain data any thread may hold (`interface Error: needs (Debug, Display, Send, ThreadShared)`), so `AnyError` and every `dyn Error` are `Send` and `ThreadShared` and a worker returns `Result[int]` with nothing written; and a base class whose objects may travel between threads says so in its header (`pub open class Script implements Send, ThreadShared:`), which every subclass is then held to where it is declared. An object held by a `pub` `open` or `abstract` class type is otherwise neither `Send` nor `ThreadShared`, because the package judging it never sees the classes a dependant derives ([BLD-16]); an `open` class no other package can subclass is judged from its own package's derived classes with nothing written, as [EFF-2a] already does for virtual calls; and a `thread_shared T` of an open `T` is `Send` and `ThreadShared` because its making line checked the object's own class (verdict 165). The method-4 hole closes at the library's own line (`extend LoadError implements Error:` is `E2040` while `Script` promises nothing) and, once the base promises, at the game's own line (`class Enemy(Script):` holding `shares Level` is `E7015`), never at a spawn far away.


## Judgement 1

**winner**: 1

**scores**: [
 {
  "design": 1,
  "safety": 9,
  "speed": 10,
  "readability": 8,
  "consistency": 8,
  "generality": 8,
  "holes": "No accepted program breaks a promise that I could build: the derived class is checked where it is declared (\"checked where the derived class is declared, by the package that declares it, whichever package declares the base (`[BLD-16]`)\"), a generic derived class falls under \"meets with its own fields\" (a type parameter has only what its `needs` give, `[TYP-17]`: \"Inside a generic body only the operations the bounds provide are available\"), the OWN-13 edit closes the method pin (`weakref this` in a `Send` family), and HR-44 covers the reload. Weak points, not holes: (a) the derived-class refusal is `E2040` while `[FFI-17d]` already refuses the same situation (a class that promised any-thread holding a one-thread field) with `E7015` (\"`threads=any` requires the class to be `Send` and `ThreadShared` (`E7015` at the class, naming the field\"), so one situation carries two codes; (b) every `open` class, private ones included, needs the word, though `[WK-18]` already judges a private family from \"no derived class in this package has one, and no `pub` class of the family is `open` or `abstract`, so no other package derives from it\"; (c) the reload sentence sits in HR-44 (worked-out facts) though the promise is a written word, and covers only a removed marker: an *added* marker lets the base's rebuilt code hand `Script`s to threads while a dependant's `Enemy(Script)` with `shares Level`, not rebuilt, was never checked; (d) `thread_shared Script()` on a promise-less `pub open class Script` compiles and can go nowhere, with no message naming the promise."
 },
 {
  "design": 2,
  "safety": 6,
  "speed": 10,
  "readability": 8,
  "consistency": 6,
  "generality": 8,
  "holes": "Safety hole in the text: \"On a generic class the promise holds for each instance whose arguments meet the needs its parameters are bounded by, as a generic `unsafe extend` does\". Program: gamekit `pub open class Script implements Send, ThreadShared:`; game `class Holder[T](Script):` with `item: T` and no bounds; `h = Holder[shares Level](shares level)`; `fail(owns h)?` into `AnyError`; `thread.spawn(owns fn() owns r: r.is_err())`. By that sentence `Holder[shares Level]` does not hold the promise, yet it is a `Script` (`[TYP-5]` rule 8) and the `Script`-typed value is `Send` by the header; the `shares` count races. (The implementer step 2 says the opposite, \"A generic class checks with its parameters' written needs\"; the rule text is what counts.) Ambiguity: \"it is `Send` only when its header writes `implements Send`\" leaves `pub open class GameError implements Error:` undecided as a promise, though IFC-3's edit assumes it. Consistency: TYP-43's edit makes `implements Send` on a final class or struct `E2040` while the `E3100` row (\"`Send` declared by hand without `unsafe extend`\") and TYP-43's own earlier sentence (\"written without `unsafe` it is `E3100`\") stay, so one program has two codes; the `E3100` row is not edited at all, and would still refuse the header promise it allows. HR-11's row covers a dropped promise only, not an added one."
 },
 {
  "design": 3,
  "safety": 8,
  "speed": 10,
  "readability": 7,
  "consistency": 8,
  "generality": 8,
  "holes": "Sound as far as I can break it, with one clause resting on a route the text does not close: \"a `thread_shared T` ... of an `open` or `abstract` `T` whatever its class promises, since the line that makes a `thread_shared` object checks the object's own class\". The making line checks the class as it is when compiled; a reload of the derived class's package that adds `level: shares Level = shares Level()` migrates a live `thread_shared Enemy`, upcast to `thread_shared Script` and already on another thread (made in a package the reload does not rebuild), into an object holding a `shares` owner; Annex B's table refuses a changed field type but says nothing of an added one-thread field on a class whose instances are `thread_shared`. That route is pre-existing for final classes too, so I score it as a doubt the design itself admits (\"the lead should confirm hot-reload migration\"), not a new hole; but the clause is a second rule a newcomer must hold (\"a `thread_shared` of an open class is Send whatever the class says\") for a small gain. Readability: the THR-8 clause is the longest of the three and admits two spellings (\"in its header or in an `extend` of its own package\") and a third promise route (`threads=any`). Consistency: the BLD-14 item 8 edit records the promise among \"facts worked out\", though it is a written word `[MOD-9]` already carries (\"only the written word counts across the edge\")."
 }
]

**grafts**: From design 2: the BLD-16 list gains the fact this ruling forbids (\"not by which classes a dependant derives from one of its `open` or `abstract` classes\"); the derived-class refusal is `E7015` at the class, naming the field, as `[FFI-17d]` and `[OWN-13]` already refuse a class that promised any-thread and holds a one-thread part (one situation, one code), leaving `E2040` for an `implements` line whose parent is unmet; `thread_shared` on a new object of a promise-less `pub open` class is `E7015` at the making line, naming the promise (verdict 165's shape, and design 1's doubt (1) answered); the in-package judgement of a class no other package can derive from, in `[WK-18]`'s exact words (\"no `pub` class of the family is `open` or `abstract`, so no other package derives from it\"), so a private hierarchy writes nothing, as `[EFF-2a]` already treats its virtual calls; and the HR-11 schema clause. From design 3: \"an interface that needs it\" named as a promise (`pub open class GameError implements Error:`), so the question design 2 left open is answered in THR-8; the Annex B table row for a promise \"added or removed\", widened so that an added promise is refused too; the DRV-1 sentence for `@derive` on a generic type (also design 2's). My own closing of design 2's hole, made explicit in THR-8: a generic derived class is checked under its written bounds, the message giving the bounded line. Not taken: design 3's `thread_shared T`-whatever-the-promise clause (a second rule, resting on a reload route Annex B does not close); design 3's `extend Script implements Send:` second spelling (one place, the header); design 3's BLD-14 item 8 edit (a written word, not a worked-out fact); design 1's HR-44 placement (replaced by HR-11 and the B.3 row, where \"base class changed | refused\" lives).

- FINAL EDIT rule=§IV.8 standard interfaces, the signature sketch (CURRENT-SPEC.md line 1958)
  - old: interface Error: needs (Debug, Display):
    fn source() -> Option[ref dyn Error]                            # default: None
  - new: interface Error: needs (Debug, Display, Send, ThreadShared):   # any thread may hold and read an error
    fn source() -> Option[ref dyn Error]                            # default: None
  - why: The cause: every error is erased through `Error` (`dyn Error`, the box in `AnyError`, the default error of every `Result[T]`), and `[THR-8]` decides an erased value by its interface ("`dyn I` when `I` requires it (`interface I: needs Send:`)"); `Error` required only `Debug` and `Display`, so by the rules no `Result[int]` could leave its thread, which no rule said. This is the declaration every bound, every `dyn Error` and every implementer reads; std/src/core.em line 80 follows it.

- FINAL EDIT rule=TYP-17, the example of an interface's parents (line 1742)
  - old: (`interface Error: needs (Debug, Display):`)
  - new: (`interface Error: needs (Debug, Display, Send, ThreadShared):`)
  - why: The second copy of the declaration; left as it was, the generics chapter would decide a program differently from §IV.8.

- FINAL EDIT rule=CLS-4
  - old: A class is final unless declared `open` or
  `abstract`.
  - new: A class is final unless declared `open` or
  `abstract`. An `open` or `abstract` class that another package may derive from says in its header
  whether any thread may hold its objects, `implements Send`, `implements ThreadShared`, or an
  interface that needs them (`[IFC-3]`): a promise every class derived from it keeps, checked where
  each is declared (`[THR-8]`, `[THR-9]`); a class that cannot be subclassed is judged from its
  fields, as a struct is.
  - why: A reader of the class rules finds the header word where `open` and `abstract` are defined; the rule itself lives in `[THR-8]`, where `Send` is defined.

- FINAL EDIT rule=THR-8, the class-object clause (lines 6338-6339)
  - old: Structs, enums, tuples, fixed arrays, collections and class objects
  are `Send` when every component is; an `owns T` when `T` is;
  - new: Structs, enums, tuples, fixed arrays, collections and objects of a class
  that cannot be subclassed are `Send` when every component is, a class object's base fields and the
  kind its methods say (`[OWN-13]`) among them; **an object held by the type of an `open` or
  `abstract` class is judged by what the class declares**, since it may be of any class derived from
  it (`[TYP-5]` rule 8), in a package the holder's package never sees (`[BLD-16]`): it is `Send`
  when the class, or a base of it, writes `implements Send` in its header, or implements an
  interface that needs `Send` (`pub open class Script implements Send, ThreadShared:`, `pub open
  class GameError implements Error:`), a promise for the class and every class derived from it, as a
  method pins a class and its derived classes (`[OWN-13]`) and a weak count is carried by a whole
  family (`[OBJ-1]`); the same for `ThreadShared` (`[THR-9]`). The promise is checked where each
  class of the family is declared, by that class's own package, from the base's declaration in its
  interface file (`[MOD-9]`: a word exists, so only the written word counts), over the class's own
  fields and the kind its methods say, a generic class under its written bounds (a type parameter
  has the markers its `needs` give): a field or a method that breaks the promise is `E7015` at the
  class, naming the field, the promise and its line, and giving a copy line, and the `thread_shared`
  line only where no line then changes the value directly (`[THR-1]`, verdict 73) ("`Enemy` is a
  `Script`, whose objects any thread may hold (`pub open class Script implements Send,
  ThreadShared:`, gamekit/src/lib.em line 3), a promise every class derived from `Script` keeps, and
  `Enemy.level` (line 8) is a `shares Level`, an owner counted on one thread; keep what `Enemy` reads
  from the level by copy: write `level_name: String` in place of line 8, or share the level between
  threads: `level: thread_shared Level`"; for a generic class the message gives the bounded line,
  `[T: needs (Send, ThreadShared)]` above `class Holder(Script):`). A derived class may write a
  promise its base does not; it holds for it and its derived classes, never for the base. An object
  held by the type of an `open` or `abstract` class with no such promise is handed to no thread,
  whatever classes the program derives, which no package can see: at a hand-over by any route of
  `[THR-2]` it is `E7015` ("`s` is a `Script`, an `open` class whose objects may be of any class
  derived from it, and `Script` does not say any thread may hold them; write `pub open class Script
  implements Send:` (gamekit/src/lib.em line 3), after which every class derived from `Script` is
  checked for it where it is declared, or hand the object over by its own class, `d: Door`"), and
  `thread_shared Script()` is `E7015` at the making line with the same message (`[THR-1]`). An
  `open` or `abstract` class no other package can derive from (no `pub` class of its family is
  `open` or `abstract`, `[WK-18]`) needs no promise: its objects are judged from the fields and
  methods of every class its package derives from it, as a call through its `virtual` methods counts
  every override the package has (`[EFF-2a]`), and a refusal names the derived class and its field;
  an `owns T` when `T` is;
  - why: This is the cause. "class objects are `Send` when every component is" counts the declared class's fields, narrower than THR-8's own definition ("nothing reachable from it"), and for an `open` class the components are a dependant's, a fact `[BLD-16]` forbids a package to read. The same shape the text already uses for the other erased holder (`dyn I` by its interface's `needs`), for a class's kind (`[OWN-13]`) and for the weak count (`[OBJ-1]`), and the one the owner chose for function types over a whole-program search (verdict 61). The check is local to the package that declares each class, needs no check at an upcast or a hand-over, and refuses the one-thread field at the class that holds it. `E7015` at the class is `[FFI-17d]`'s code for the same situation.

- FINAL EDIT rule=THR-8, the written-only-in-bounds sentence (lines 6344-6345)
  - old: `Send` is inferred from the words and written only
  in bounds;
  - new: `Send` is inferred from the words and written only
  in bounds and in the header of an `open` or `abstract` class, where it is a promise about every
  class derived from it that the compiler checks (above);
  - why: The sentence would otherwise contradict the clause added above.

- FINAL EDIT rule=THR-9 (line 6361)
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types, and objects of a class that cannot be subclassed, are `ThreadShared` when every
  component is; an object held by the type of an `open` or `abstract` class when the class, or a
  base, writes `implements ThreadShared` or implements an interface that needs it, a promise every
  class derived from it keeps, checked where each is declared (`E7015` at the class, `[THR-8]`), and
  one no other package can derive from by the classes its package derives from it; `dyn I` when `I`
  requires it (`interface I: needs ThreadShared:`), as for `Send` (ODR-255), so `dyn Error` is.
  - why: THR-9 had no words for a class object and none for `dyn`, so `ref r` of a `Result[int]` given to a scoped task (`ref T` is `Send` when `T` is `ThreadShared`, `[THR-8]`) and an `AnyError` under a `thread_shared` holder's lock (verdict 165) were undecided; ODR-255 ruled the `dyn` half and the text never wrote it.

- FINAL EDIT rule=§IV.8 marker table, row `Send` (line 1859)
  - old: a class object is when its fields are;
  - new: a class object is when its fields are, for a class that cannot be subclassed; an object held by the type of an `open` or `abstract` class when the class promises it (`implements Send` in its header, or an interface that needs it), which every class derived from it is checked against where it is declared (`[THR-8]`); `dyn I` when `I` requires it;
  - why: `[TYP-43]` says "the table answers them": the row was the narrow reading that let a base-typed value through, and it must give THR-8's answer.

- FINAL EDIT rule=§IV.8 marker table, row `ThreadShared` (line 1860)
  - old: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`);
  - new: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`); a class object as for `Send`: by its fields for a class that cannot be subclassed, by the class's promise (`implements ThreadShared`, or an interface that needs it) for an object held by the type of an `open` or `abstract` class (`[THR-9]`); `dyn I` when `I` requires it;
  - why: The row named neither class objects nor `dyn`; it must agree with THR-9.

- FINAL EDIT rule=TYP-43 (lines 1875-1876)
  - old: `ThreadShared` is written only in bounds, `Copy` comes from
  `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`,
  - new: `Send` and `ThreadShared` are also written, without `unsafe`, in the header of an `open` or
  `abstract` class (`pub open class Script implements Send, ThreadShared:`), whose objects' kinds a
  holder cannot see: there the word is the class family's promise, which the compiler checks against
  the class's own fields and, where each is declared, against every class derived from it
  (`[THR-8]`, `E7015`), so it vouches for nothing; on a struct, an enum or a class that cannot be
  subclassed, whose markers are inferred, `Send` stays `E3100` and `ThreadShared` `E2040` as below,
  the message giving the line without the word ("`Door` cannot be subclassed, so its `Send` is read
  from its fields; remove `implements Send`"). Otherwise `ThreadShared` is written only in bounds,
  `Copy` comes from `@derive(Copy)`, and `Sized` is never written: each of these written by hand is
  `E2040`,
  - why: TYP-43 refused every hand-written marker but the `unsafe extend` vouch; the header promise is a bound the compiler checks for every member of the family, as `interface I: needs Send` is, not a vouch, so it needs no `unsafe` and is refused where inference already answers. The grammar's `class_decl` already has `[implements]`; no new syntax.

- FINAL EDIT rule=OWN-13 (lines 3603-3605)
  - old: A method pins the class
  that declares it and every class derived from it, whose objects run it; a class pinned both ways,
  by its own methods or with a base's, is `E7015` at the one-thread line
  - new: A method pins the class
  that declares it and every class derived from it, whose objects run it; a class pinned both ways,
  by its own methods or with a base's, or pinned one-thread while it or a base promises `Send`
  (`[CLS-4]`, `[THR-8]`), is `E7015` at the one-thread line
  - why: OWN-13 reads a class's kind from its methods and already refuses a class pinned both ways; `weakref this` or `shares this` in a family that promises `Send` is the same contradiction, and belongs in the same sentence.

- FINAL EDIT rule=IFC-3 (append after "names the parent's associated type\n  (ODR-042).", line 2692)
  - old: names the parent's associated type
  (ODR-042).
  - new: names the parent's associated type
  (ODR-042). An implementation meets its interface's parents for every type it covers, checked where
  it is written: a generic one through the bounds it writes (`extend[T: needs (Send, ThreadShared)]
  PoolFull[T] implements Error:`), never quietly for some arguments only (verdict 132), so
  `extend[T] PoolFull[T] implements Error:` is `E2040`, whose message gives the bounded line, as
  `[IFC-6]` gives a `pub` function's ("`PoolFull[T]` holds `rejected: T`, and an `Error` is `Send`
  and `ThreadShared`; write `extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:`");
  a concrete one over its fields, `E2040` naming the field and giving a copy line (`[ERR-3]`). An
  `open` or `abstract` class that implements an interface with marker parents promises them for
  every class derived from it, each checked at its own declaration (`[THR-8]`), so the
  implementation is never checked again.
  - why: IFC-3 said implementers meet the parents but not that an implementation meets them for every type it covers; a generic error implementation over an unbounded `T` would put `PoolFull[shares Enemy]` into an `AnyError`. Stated once for every interface with parents (`Ord`, `IndexMut`, a program's `Command: needs Send`), never as a case of `Error`.

- FINAL EDIT rule=DRV-1 (line 7538)
  - old: A derive whose requirement a field does not meet is `E2080`, naming the field.
  - new: A derive whose requirement a field does not meet is `E2080`, naming the field (`@derive(Error)` on a
  type with a `shares` field: `Error` needs `Send`, `[ERR-3]`). On a generic type the generated
  `extend` writes, as its parameters' needs, what its fields need of them (`@derive(Error)` above `[T]`
  and `enum SpawnError:` with `Rejected(T)` generates `extend[T: needs (Debug, Send, ThreadShared)]
  SpawnError[T] implements Error:`), so the derive holds for the instances that meet them, as
  `@derive(Copy)` above `[T]` and `pub struct Range:` is `Copy` when `T` is (ODR-027, `[STR-5]`,
  `[IFC-3]`).
  - why: With `Error`'s new parents a derived generic error that hands the caller's value back must be decided; a derive is the compiler writing the implementation, so it writes the needs itself, as every derive on a generic type already behaves, and `ember inspect --expand` prints them.

- FINAL EDIT rule=ERR-3 (lines 7322-7324)
  - old: `Error` is the
  interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default
  `None`) (§IV.8).
  - new: `Error` is the
  interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with
  `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8): any thread may hold and read an
  error, so `dyn Error`, and `AnyError` with it, are `Send` and `ThreadShared` (`[THR-8]`,
  `[THR-9]`, `[ERR-8]`), and a `Result` whose error is one goes to a worker thread, through a
  channel and into a scoped task's `ref` as any such value does (`[THR-10]`, `[THR-15]`, `[THR-2]`).
  A type holding a value that stays on one thread (a `shares` or `shares mut` owner, a `weakref T`
  or `sharedref mut` link, a `Cell`, an object held by the type of an `open` class that makes no
  promise, `[THR-8]`) is not an `Error`: `E2040` where it implements `Error`, naming the field and
  giving the line to write (`[IFC-3]`: "`Error` needs `Send` and `ThreadShared`, since any thread may
  hold an error, and `LoadError.script` is a `Script`, an `open` class that does not say any thread
  may hold its objects, so a `LoadError` may hold an object of a class another package derives from
  `Script`; write `pub open class Script implements Send, ThreadShared:` (line 3), which every class
  derived from `Script` then keeps, or keep what the error reports by copy: `entity: int`"), and
  `E2080` at `@derive(Error)`, naming the field (`[DRV-1]`). It may still be a `Result`'s error,
  matched or printed by `Display` (`[ERR-1]`, `[ERR-12]`, ODR-050).
  - why: ERR-3 describes the declaration of §IV.8 and must say the same thing; the `Display + Debug` spelling was the pre-verdict-110 one. It states one property (any thread may hold an error) and leaves the placing of refusals to IFC-3 and DRV-1.

- FINAL EDIT rule=ERR-8 (line 7338)
  - old: An `AnyError` passed through `?` into another
  `AnyError` is handed over (`[ERR-7]`).
  - new: An `AnyError` passed through `?` into another
  `AnyError` is handed over (`[ERR-7]`). `AnyError` is `Send` and `ThreadShared` as its parts are:
  its `owns dyn Error` because `Error` requires both (`[ERR-3]`, `[THR-8]`, `[THR-9]`), its context
  a list of `String`; so a worker returns `Result[int]` through `thread.spawn` or a channel, a
  `thread_shared` holder keeps one under its lock and a scoped task reads one through `ref`, with
  nothing written (`[THR-10]`, `[THR-15]`, `[THR-1]`, `[THR-11]`). Nothing is checked at `?`: every
  `E: needs Error` met the requirement where it implemented `Error`, in its own package, and an
  `AnyError` handed to another thread walks the error it holds by that error's hand-over routine
  (`[THR-2]`), as any `owns dyn I` value.
  - why: The question asked for `AnyError`'s markers in so many words; ERR-8 is where it is defined. It states a consequence of ERR-3 and THR-8, adds no rule of its own, and names the one run-time step that already exists.

- FINAL EDIT rule=BLD-16 (lines 9274-9275)
  - old: not by which of its values are made
  `thread_shared`, not by which of its classes a `weakref` names.
  - new: not by which of its values are made
  `thread_shared`, not by which of its classes a `weakref` names, not by which classes a dependant
  derives from one of its `open` or `abstract` classes (`[THR-8]`).
  - why: The case this ruling closes is exactly such a fact: a library judged `Script` `Send` from fields a dependant could add to. Naming it keeps the list the implementer's checklist.

- FINAL EDIT rule=HR-11 (Annex B)
  - old: `ref mut`, `using A`), so a change of one is a change of the field's type.
  - new: `ref mut`, `using A`), so a change of one is a change of the field's type. A class's schema includes
  the promise its header writes (`implements Send`, `implements ThreadShared`, an interface that
  needs them, `[THR-8]`): a change of it is a change of the class (§B.3).
  - why: Hot reload is a route to the same race: the promise a derived class in another package was checked against is part of what the class declares, so plan compares it; a derived class rebuilt with a field that breaks a promise is refused where it compiles, before any plan.

- FINAL EDIT rule=Annex B §B.3 schema table, after the row "| base class changed | refused |" (line 10870)
  - old: | base class changed | refused |
  - new: | base class changed | refused |
  | `Send` or `ThreadShared` promised by an `open` or `abstract` class added or removed (`[THR-8]`, `[HR-11]`) | refused (`E9030`) while a package the image does not rebuild names the class; code the reload rebuilds is checked again when it compiles (`[HR-44]`) |
  - why: A removed promise leaves dependants that hand the class's objects to threads unchecked; an added one lets the base's new code hand them to threads while a dependant's derived class, not rebuilt, was never checked. The row sits beside "base class changed", and `[HR-44]`'s own words give the exception.

- FINAL EDIT rule=§XVII code registry, row `E7015` (line 10004)
  - old: a one-thread owner in a trampoline class under `threads=any` | `[THR-1]`
  - new: a one-thread owner in a trampoline class under `threads=any`; a field or a method pin that breaks the `Send` or `ThreadShared` promise of an `open` or `abstract` class, in that class or in one derived from it, at the class; a value held by the type of an `open` or `abstract` class with no such promise handed to another thread, or made `thread_shared` | `[CLS-4]`, `[THR-1]`
  - why: The row lists every refusal that carries the code; THR-8, THR-9 and OWN-13 now name three more.

- FINAL EDIT rule=§XVII code registry, row `E2040` (line 9848)
  - old: has no `Display` | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]` |
  - new: has no `Display`; an implementation that does not meet its interface's parents for every type it covers, at the implementation: a generic one (the bounded line given), or a type holding a one-thread value implementing `Error` (the field named, a copy line given); `ThreadShared` written on a struct, an enum or a class that cannot be subclassed | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]`, `[IFC-3]`, `[ERR-3]`, `[TYP-43]` |
  - why: One code, one defining rule: the unmet parent is an unsatisfied bound, which is E2040's meaning; the row says where it now falls.

- FINAL EDIT rule=§XVII code registry, row `E3100` (line 9940)
  - old: `Send` declared by hand without `unsafe extend`;
  - new: `Send` declared by hand without `unsafe extend`, other than in the header of an `open` or `abstract` class, where it is a checked promise (`[TYP-43]`, `[THR-8]`);
  - why: The row would otherwise refuse the promise TYP-43 now allows.

**final_cost_row**: No new row (`[COST-5]`): every judgement here is made when compiling, from a declaration the compiling package sees (the base's header in its own text or its interface file, the derived class's own fields, `Error`'s declared parents), and is covered by the existing row "What a function changes, reads and ends; the thread rules; the arena checks | not observable | all checked when compiling (`[EFF-24]`)". Two existing rows become reachable for an `AnyError` that now crosses, unchanged: "`?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`)", and "An object handed whole to another thread | guaranteed present | ... one load and one test, then a call of the kind's hand-over routine, per owned value whose layout the handing package cannot see (held `dyn I`, of an `open` or `abstract` class, a function value's state)", which the boxed `dyn Error` pays once per `AnyError` handed to another thread, on the failure path only. The run-time alternative (a `Send` flag in `TypeInfo` tested at each hand-over of a base-typed object) was priced and turned down: it can only stop the program, since a `shares` owner inside the object cannot be moved away from its other owners.

**owner_verdicts_touched**: None overridden. 61 (a type says which threads may have its values, chosen over the compiler finding it from the whole program: kept and extended from function types to open classes, the header word). 94 (`thread.spawn` takes `thread_shared once fn() -> R`, `R` MUST be `Send`: unchanged, now met by `AnyError`). 110 (parents with `needs`: `interface Error: needs (Debug, Display, Send, ThreadShared):` in both copies). 108, 111, 113 (a `pub` generic and an `extend` write their needs; a derive writes its own: the generic error implementation writes `[T: needs (Send, ThreadShared)]` and the message gives the line). 132 (a declaration means what it says: `extend[T] X[T] implements Error:` is refused, never quietly narrowed). 73 (the given fix compiles: the copy line is always offered, `thread_shared` only where `[THR-1]` then holds). 86 and 122 (facts worked out from bodies: untouched; a marker is a property of a type, decided per `[BLD-16]`, lead ruling N76). 164, 165, 166 (a `sharedref mut` is neither marker; `thread_shared` refused where made; a scoped task touches no one-thread count: each error type or derived class holding one is refused where it is declared; 165's shape is reused for `thread_shared Script()`). 23 (a `shares` value never used from two threads: none can sit inside an error, a derived class of a promising family, or a base-typed value that crosses). 114 and 101 (a handed object's links say gone; the 32-byte header: `[THR-2]`'s routine already walks the boxed error). Lead rulings touched, reported: ODR-111 (`Send` by hand only as `unsafe extend`: kept for the vouch; the class header is a compiler-checked promise, so `[TYP-43]` names it without `unsafe`); ODR-255 (`dyn I` Send/Sync when `I` requires it: its `ThreadShared` half now written in `[THR-9]`); ODR-050 (a `Result`'s error parameter unbounded: kept, a one-thread type stays a `Result` error printed by `main`); N76 (`[BLD-16]`: the list gains one fact); N77 (`[OWN-13]`: one more contradiction in the same sentence). The owner gave no verdict on K20 itself (J6-16 was routed to him); this is the proposal, with its cost stated: an error type holding a one-thread part is refused where it implements `Error`, even in a one-thread program, the fix being a copy; a `pub open` class's objects cross threads only when its author writes the promise.

**remaining_doubt**: (1) `[CLS-12]`'s "On an object several threads reach, the kept result is filled once under a once-flag" is a whole-program fact as written; under `[BLD-16]` it should be read from the class's declaration (it promises `ThreadShared`, implements `Error`, or its methods pin it `thread_shared`). Pre-existing (the method-4 record's S2-2), made reachable by this ruling, not caused by it; no edit here, flagged for the lead. (2) A reload that adds a one-thread field with a default (`level: shares Level = shares Level()`) to a class whose live instances are `thread_shared` on other threads, made in a package the image does not rebuild: Annex B's table refuses a changed field type but says nothing of an added one; pre-existing for final classes, independent of this question, and the reason design 3's `thread_shared T`-whatever-the-promise clause was not grafted. (3) Two spellings reach one promise (`implements Send` directly, or `implements Command` with `interface Command: needs Send`); both exist in the grammar and both are checked by the one THR-8 sentence; a lead who wants one may refuse the bare marker when an interface in the list already brings it. (4) The in-package judgement of a class no other package can derive from lets an added subclass with a `shares` field break a `thread.spawn` elsewhere in the same package (the message names the subclass's field); verdict 86 accepted that shape inside a package for effects, and `[WK-18]` and `[EFF-2a]` already draw this exact line; a lead who wants one rule drops that sentence and requires the word on every `open` class. (5) `Error` requiring `ThreadShared` as well as `Send` refuses an error holding a `Cell` as an `Error`; `Send` alone would serve `spawn`, channels and joins, but `ref AnyError` to a scoped task and an `AnyError` under a `thread_shared` holder's lock (verdict 165's shape) would then be refused; both markers follow Rust's anyhow and Swift. (6) std must compile once under the new parents: `io.Error`, `ParseError`, `CppError` and core.em's two derived enums hold plain data; `SendError[T]` (std/src/sync.em 122) gets its `Error` implementation under `[T: needs (Send, ThreadShared)]`; `pub open class AnyStorage` (std/src/ecs.em 101) writes the promise only if a `World` is meant to cross threads (ODR-218); read only, not verified. (7) A base-typed value whose exact kind the compiler sees on the line (`s = Script()` handed over at once) is refused without the promise; a small flow rule would allow it, left out as not the smallest change (the `dyn I` cost already carried today).

## Judgement 2

**winner**: 3

**scores**: [
 {
  "design": 1,
  "safety": 7,
  "speed": 10,
  "readability": 7,
  "consistency": 5,
  "generality": 8,
  "holes": "Reload add route open: its HR-44 sentence refuses only \"a reload whose new image removes a marker from it\"; a reload of gamekit alone that changes `pub open class Script:` to `pub open class Script implements Send, ThreadShared:` lets gamekit's new `thread.spawn(owns fn() owns s: ...)` with `s: Script` cross while the game's `class Enemy(Script): level: shares Level`, not rebuilt, was never checked; `[HR-44]` compares \"what each function may change, read and end\", not a header word. THR-8 is left saying \"`Send` is inferred from the words and written only in bounds\" and THR-9 \"written only in bounds\", and the table header \"(`Send` written by hand only with `unsafe extend`, `[TYP-43]`)\", each contradicting its CLS-4/TYP-43 edits. Its IFC-3 sentence checks a derived class for \"a marker parent\" of an interface the base implements; a base writing `implements Send` directly has `Send` as the interface, not a parent, and CLS-4 points to IFC-3 for that case while IFC-3 points back to THR-8. Its IFC-3 old text \"(ODR-042).\" occurs three times in the file. `thread_shared Script` for a promise-less `Script` is undecided (THR-8: \"a `thread_shared T` ... when `T` is `Send` and `ThreadShared`\" now reads not `Send`, so a checked `thread_shared Enemy()` upcast cannot cross). Its accepted program reads `d.entity` from the game while gamekit declares `let entity: int` without `pub`."
 },
 {
  "design": 2,
  "safety": 7,
  "speed": 10,
  "readability": 6,
  "consistency": 4,
  "generality": 7,
  "holes": "Its THR-8 old text does not match the file (\"`Send`; a\\n  `thread_shared T`, and a\\n  `weakref thread_shared T`\" where the file reads \"`Send`; a `thread_shared T`, and a\\n  `weakref thread_shared T`\"), so the cause edit cannot be applied as written. `implements Send` on a final class is `E2040` in its TYP-43 and E2040-row edits while the E3100 row's \"`Send` declared by hand without `unsafe extend`\" is left unchanged: two codes for one program. Reload: its HR-11 sentence covers \"a new image that drops one from a class\" only; the add route above is open. `open class GameError implements Error:` must also write `implements Send, ThreadShared` (its IFC-3: \"An `open` or `abstract` class meets a marker parent through the promise in its header\"; its step 2 collects only \"the markers named in this header's `implements` list that are `Send` or `ThreadShared`\"), two redundant words on every error hierarchy. Its step 3 reads \"subclassable outside its package (`pub`)\": a private `open class Base:` with `pub open class Mid(Base):` is derivable outside through `Mid`, so a `Base`-typed value judged from the package's own derived classes could hold a dependant's `Evil(Mid)` with a `shares` field; the rule text's \"cannot be subclassed outside its package\" is right only read transitively, which the step does not. THR-9's \"written only in bounds\" and the table header untouched. Cites verdict 132 (`@ffi(const)` overlays) as \"a declaration means what it says\"."
 },
 {
  "design": 3,
  "safety": 8,
  "speed": 10,
  "readability": 6,
  "consistency": 7,
  "generality": 8,
  "holes": "THR-9's \"It is inferred like `Send` and written only in bounds\" and the table header \"(`Send` written by hand only with `unsafe extend`)\" untouched, both contradicting the header promise. Four spellings of one promise (header, `extend` in the class's package, an interface that needs the markers, `threads=any`): the `extend` spelling re-opens TYP-43's \"`Send` by hand only as `unsafe extend`\" for a memberless `extend Script implements Send:` that a reader tells from the vouch only by the class's being open. Its step 2(b) says \"`open`/`abstract` and `pub`\" where the rule says \"no other package can subclass (`[EFF-2a]`)\": the same narrow-step risk as design 2 (a private base with a `pub open` derived class). The `thread_shared T` clause inherits the existing reload gap (Annex B: \"field added with a default | initialised to that default\" on a class another package makes `thread_shared`), not caused by it. Cites verdict 132 wrongly, as design 2 does. Its THR-8 message offers `level: thread_shared Level` unconditionally, which compiles only where no line then changes the level directly (`[THR-1]`, verdict 73)."
 }
]

**grafts**: Winner: the third design (counting the JSON list from 1). All three put the same word on the same thing (an `open` or `abstract` class's header says whether any thread may hold its objects, `Error` needs `Send` and `ThreadShared`, `AnyError`'s markers follow from its fields) and the third is the one whose edits can be applied (its old texts match the file), that fixes THR-8's own \"written only in bounds\" sentence, that keeps `E3100` for `Send` on a final class, and that covers the reload add route (its Annex B row says \"added or removed\"), the route designs 1 and 2 leave open. Taken from the third and kept: the THR-8 class clause by its class (final by fields, derivable-outside by promise, private family by its package's derived classes as `[EFF-2a]` counts overrides), the promise through an interface that needs the markers (`open class GameError implements Error:` needs no second word), the `thread_shared T` clause (a `thread_shared` object is made only by a line that checks its own class, `E7017` elsewhere, so a checked `thread_shared Enemy()` upcast to `thread_shared Script` still crosses), THR-9's class and `dyn` clauses, BLD-14 item 8, the Annex B row, DRV-1's generic derive, IFC-3's generic implementation. Trimmed from the third: the `extend` spelling (the header is the one place, as TYP-43 keeps `unsafe extend` the one place for the vouch) and `threads=any` as a promise (FFI-17d already requires the class to be `Send` and `ThreadShared`, which for an open trampoline class now means the header word; noted under remaining doubt, FFI-17d unchanged); the mis-cited verdict 132 is dropped. Grafted from the second: the generic-class sentence (a promise on `open class Holder[T] implements Send:` holds per instance under the parameters' written needs, as a generic `unsafe extend` does), the BLD-16 list sentence (\"not by which classes a dependant derives from one of its `open` or `abstract` classes\"), the one-sentence pointer in CLS-4 where `open` and `abstract` are defined, the `E7015` message's two fixes (a copy line first, then `thread_shared` only where `[THR-1]` then holds), and the \"derived class may promise what its base does not\" sentence. Grafted from the first: the OWN-13 sentence (a method pinning a class one-thread while it or a base promises `Send` is the same contradiction as a class pinned both ways, `E7015` at the one-thread line), and the ERR-8 sentence that a refused one-thread error was refused where it was declared an error, never at the `?`. Added by the judge, missing from all three: the THR-9 \"written only in bounds\" carve-out, the marker table header's carve-out, the E9030 row entry, the transitive statement of \"another package may derive from it\" (it, or an `open` or `abstract` class derived from it in its package, is `pub`), and the interface-file record of a private open family's inferred markers (a dependant holding one through a `pub` type's private field reads them, `[BLD-14]` item 8).

- FINAL EDIT rule=§IV.8 standard interfaces, the signature sketch (the declaration ERR-3 and TYP-17 cite)
  - old: interface Error: needs (Debug, Display):
    fn source()
  - new: interface Error: needs (Debug, Display, Send, ThreadShared):      # any thread may hold and read an error ([ERR-3])
    fn source()
  - why: This is the declaration every bound, every `dyn Error` and every implementer reads ("These are the definitions; `std.core` declares them"); `std/src/core.em` line 80 follows it. `Error` was the one erasing interface that said nothing about threads, so by `[THR-8]`'s "`dyn I` when `I` requires it" every erased error was a one-thread value and `Result[int]` could not leave its thread, which no rule said.

- FINAL EDIT rule=TYP-17, the example of an interface's parents
  - old: (`interface Error: needs (Debug, Display):`)
  - new: (`interface Error: needs (Debug, Display, Send, ThreadShared):`)
  - why: The third copy of the declaration; left as it was, the generics chapter would decide a program differently from §IV.8 and `[ERR-3]`.

- FINAL EDIT rule=ERR-3
  - old: `Error` is the
  interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default
  `None`) (§IV.8).
  - new: `Error` is the
  interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with `fn
  source() -> Option[ref dyn Error]` (default `None`) (§IV.8): an error is a value any thread may
  hold and read, so `dyn Error`, and `AnyError` with it, are `Send` and `ThreadShared` (`[THR-8]`,
  `[THR-9]`, `[ERR-8]`), and a `Result` whose error is one goes to a worker thread, through a channel
  and into a scoped task's view as any such value does (`[THR-10]`, `[THR-15]`, `[THR-11]`,
  `[THR-2]`). A type holding a value that stays on one thread (a `shares` or `shares mut` owner, a
  `weakref T` or `sharedref mut` link, a `Cell`, an object held by the type of an `open` or
  `abstract` class another package may derive from whose header promises neither marker, `[THR-8]`)
  is not an `Error`: where it implements `Error` it is `E2040`, naming the field and giving the line
  that compiles (`[IFC-3]`: "`Error` needs `Send` and `ThreadShared`, since any thread may hold an
  error, and `LoadError.script` is a `Script`, an `open` class whose header does not say that any
  thread may hold its objects, so a `LoadError` may hold an object of a class another package
  derives from `Script`; keep what the error reports as a copy, `entity: int`, or write `pub open
  class Script implements Send, ThreadShared:` (line 3), which every class derived from `Script`
  then keeps"), and at `@derive(Error)` it is `E2080`, naming the field (`[DRV-1]`); it may still
  be a `Result`'s error, matched or printed by its `Display` (`[ERR-1]`, `[ERR-12]`, ODR-050), and
  it does not pass through `?` into `AnyError`.
  - why: ERR-3 describes the declaration of §IV.8 and the two must say one thing; the `Display + Debug` spelling is the one verdict 110 replaced. It states one property (any thread may hold an error) and leaves the placing of refusals to IFC-3 and DRV-1; the message gives the copy line first, which always compiles (verdict 73), and the base's promise second.

- FINAL EDIT rule=ERR-8 (append after the hand-over sentence)
  - old: An `AnyError` passed through `?` into another
  `AnyError` is handed over (`[ERR-7]`).
  - new: An `AnyError` passed through `?` into another
  `AnyError` is handed over (`[ERR-7]`). `AnyError` is `Send` and `ThreadShared` as its parts are:
  its `owns dyn Error` because `Error` requires both (`[ERR-3]`, `[THR-8]`, `[THR-9]`), its identity
  a number, its context a list of `String`; so a worker returns `Result[int]` through `thread.spawn`
  or a channel, a `thread_shared` holder keeps one under its lock and a scoped task reads one through
  `ref`, with nothing written (`[THR-10]`, `[THR-15]`, `[THR-1]`, `[THR-11]`). Nothing is checked at
  `?`: every `E: needs Error` met the requirement where it implemented `Error`, in its own package,
  so the one-thread error an `AnyError` cannot carry was refused there, never at the `?`; an
  `AnyError` handed to another thread walks the error it holds by that error's kind's hand-over
  routine, as any `owns dyn I` value is walked (`[THR-2]`).
  - why: The question asked for `AnyError`'s markers in so many words, and ERR-8 is where `AnyError` is defined; the sentence states a consequence of ERR-3 and THR-8, adds no rule of its own, and names the one run-time step that already exists (THR-2's routine for a `dyn` value, a row of §X.4).

- FINAL EDIT rule=THR-8, the class-object clause of the `Send` list (the cause)
  - old: Structs, enums, tuples, fixed arrays, collections and class objects
  are `Send` when every component is; an `owns T` when `T` is;
  - new: Structs, enums, tuples, fixed arrays and collections
  are `Send` when every component is; a class object by its class, since an object held by a base
  class type may be of any class derived from it (`[TYP-5]` rule 8): an object of a class that is
  neither `open` nor `abstract` when the class's fields, inherited ones included, are; an object of
  an `open` or `abstract` class that **another package may derive from** (it, or an `open` or
  `abstract` class derived from it in its package, is `pub`) only when the class **promises** it in
  its header, `implements Send`, `implements ThreadShared`, or an interface that needs them (`pub
  open class Script implements Send, ThreadShared:`, `open class GameError implements Error:`),
  written without `unsafe` since the compiler checks it (`[TYP-43]`): the promise holds the class's
  own fields and methods (`[OWN-13]`) and those of every class derived from it, each checked where
  that class is declared, by the package that declares it, from the base's declaration in its
  source or in its interface file (`[MOD-9]`, `[BLD-14]` item 8), so a derived class that breaks it
  is `E7015` at its own line, naming the field and the promise and giving a copy line, and the
  `thread_shared` line only where `[THR-1]` then holds ("`Enemy.level` is a `shares Level` owner,
  which stays on one thread, and `Script` promises that any thread may hold its objects (`pub open
  class Script implements Send, ThreadShared:`, gamekit/src/lib.em line 3), a promise every class
  derived from it keeps; keep what the enemy reads from the level as a copy, `level_name:
  String`"); a derived class may promise what its base does not, for itself and the classes
  derived from it, never for the base; a generic class's promise holds for each instance whose
  arguments meet the needs its parameters are bounded by, as a generic `unsafe extend` does
  (`[TYP-43]`); an object of such a class whose header promises nothing is one no thread may be
  handed, whatever classes the program derives, which no package can see (`[BLD-16]`): a value held
  by its type reaching another thread by any route of `[THR-2]` is `E7015` at that line ("`s` is a
  `Script`, an `open` class whose header does not say that any thread may hold its objects, so it
  may be an object of a class another package derives from `Script`; write `pub open class Script
  implements Send:` (gamekit/src/lib.em line 3), after which every class derived from `Script` is
  checked for it where it is declared, or hand the object over by its own class, `d: Door`"); and
  an object of an `open` or `abstract` class that no other package can derive from when its fields
  and those of every class its package derives from it are, with nothing written, as a call through
  its `virtual` methods counts every override the package has (`[EFF-2a]`), a refusal naming the
  derived class and its field; an `owns T` when `T` is;
  - why: This is the cause. "class objects are `Send` when every component is" counted the declared class's fields, narrower than THR-8's own definition ("nothing reachable from it"), and for a class another package may derive from the components are a fact `[BLD-16]` forbids a package to read. The family's markers become a word of the class's own declaration, checked at every class's own line by the package that sees its fields: local to each package, no check at an upcast, a hand-over or a `?`, and the refusal falls on the class that holds the one-thread field. It is the shape the text already uses for `dyn I` (`needs Send`), for a class's kind (`[OWN-13]`) and for the weak count (`[OBJ-1]`), the one the owner chose for function types over a whole-program search (verdict 61), and `[EFF-2a]`'s split for a family no other package can reach.

- FINAL EDIT rule=THR-8, the `thread_shared T` clause
  - old: a `thread_shared T`, and a
  `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`;
  - new: a `thread_shared T`, and a
  `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`, and for an `open` or `abstract`
  `T` whatever its class promises, since a `thread_shared` object is made only by a line that checks
  the object's own class (`[THR-1]`, verdict 165; `E7017` anywhere else) and an upcast keeps the
  word (`[TYP-5]` rule 8);
  - why: Without it the class clause would make `thread_shared Script` not `Send` for a promise-less `pub open class Script`, though every `thread_shared Enemy()` was checked at its making line; the clause keeps a polymorphic object that must cross threads (`script: thread_shared Script`) without forcing the promise on every subclass, and decides a question an implementer would otherwise guess.

- FINAL EDIT rule=THR-8, "`Send` is inferred from the words"
  - old: `Send` is inferred from the words and written only
  in bounds;
  - new: `Send` is inferred from the words and written only
  in bounds and in the header of an `open` or `abstract` class, where it is a promise about every
  class derived from it that the compiler checks (above);
  - why: The sentence would otherwise contradict the clause added above.

- FINAL EDIT rule=THR-9, "It is inferred like `Send`"
  - old: It is inferred
  like `Send` and written only in bounds (`[T: needs ThreadShared]` above `fn keep(xs: Array[T]) ->
  int:`) and in messages;
  - new: It is inferred
  like `Send` and written only in bounds (`[T: needs ThreadShared]` above `fn keep(xs: Array[T]) ->
  int:`), in the header of an `open` or `abstract` class as a checked promise (`[THR-8]`,
  `[TYP-43]`) and in messages;
  - why: None of the three designs touched it; left as it was, THR-9 would refuse the `implements ThreadShared` that THR-8 and TYP-43 allow.

- FINAL EDIT rule=THR-9, structural types
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types are `ThreadShared` when every component is; a class object as `[THR-8]` judges
  `Send`: by its fields when its class is neither `open` nor `abstract`, by the class's promise
  (`implements ThreadShared`, or an interface that needs it) when another package may derive from
  it, each derived class checked where it is declared (`E7015`), and by its package's derived
  classes when none can; `dyn I` when `I` requires it (`interface I: needs ThreadShared:`), as for
  `Send` (ODR-255), so `dyn Error` is (`[ERR-3]`); a `thread_shared T` and a `weakref thread_shared
  T` of an `open` or `abstract` `T` whatever its class promises, as `[THR-8]` says.
  - why: THR-9 had no words for a class object and none for a `dyn`, so `ref r` of a `Result[int]` given to a scoped task (`ref T` is `Send` when `T` is `ThreadShared`, `[THR-8]`) and an `AnyError` under a `thread_shared` holder's `RwLock` (verdict 165) were undecided; ODR-255 ruled the `dyn` half and the text never wrote it.

- FINAL EDIT rule=§IV.8 marker table, the header line
  - old: **Marker interfaces**, derived automatically (`Send` written by hand only with `unsafe extend`, `[TYP-43]`):
  - new: **Marker interfaces**, derived automatically (`Send` written by hand only with `unsafe extend`, and `Send` and `ThreadShared` as the checked promise in the header of an `open` or `abstract` class, `[TYP-43]`):
  - why: The header would otherwise contradict the edited TYP-43; none of the three designs touched it.

- FINAL EDIT rule=§IV.8 marker table, row `Send`
  - old: a class object is when its fields are;
  - new: a class object is when its fields are, where its class is neither `open` nor `abstract`; an object held by the type of an `open` or `abstract` class another package may derive from only when the class's header promises `Send` (`implements Send`, or an interface that needs it), which every class derived from it is checked against where it is declared, and one held by the type of such a class no other package can derive from when the fields of every class its package derives from it are (`[THR-8]`); `dyn I` when `I` requires it;
  - why: `[TYP-43]` says "the table answers them": the row was the narrow reading that let a base-class value holding a one-thread derived object through, and it must give THR-8's answer.

- FINAL EDIT rule=§IV.8 marker table, row `ThreadShared`
  - old: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`);
  - new: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`); a class object as for `Send`, by its class's header promise (`implements ThreadShared`, or an interface that needs it) where another package may derive from it, each derived class checked where it is declared (`[THR-9]`); `dyn I` when `I` requires it;
  - why: The row named neither a class object nor a `dyn`; it must agree with THR-9.

- FINAL EDIT rule=TYP-43, the written-by-hand sentence
  - old: A program writes `Send` by
  hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend T
  implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler
  cannot check
  - new: A program writes `Send` or `ThreadShared` in one checked place, the header of an `open` or
  `abstract` class (`pub open class Script implements Send, ThreadShared:`), a promise about every
  class derived from it that the compiler checks at each class's declaration (`[THR-8]`,
  `[THR-9]`), so it needs no `unsafe`; on a generic class it holds for each instance whose arguments
  meet the needs its parameters are bounded by; on a struct, an enum or a class that is neither
  `open` nor `abstract`, whose markers are inferred, the word is refused as below (`E3100` for
  `Send`: "`Door` cannot be subclassed, so whether any thread may have a `Door` is read from its
  fields; remove `implements Send`"; `E2040` for `ThreadShared`). Otherwise a program writes `Send`
  by hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend
  T implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler
  cannot check
  - why: TYP-43 refused every hand-written marker but the `unsafe extend` vouch; the header promise is a bound the compiler checks for every class of the family, as `interface I: needs Send` is, not a vouch, so it is allowed without `unsafe` and refused where inference already answers, with today's codes. The grammar's `class_decl := ["open" | "abstract"] "class" IDENT ["(" type ")"] [implements]` already parses it; no new syntax.

- FINAL EDIT rule=CLS-4
  - old: A class is final unless declared `open` or
  `abstract`.
  - new: A class is final unless declared `open` or
  `abstract`. An `open` or `abstract` class that another package may derive from says in its header
  whether any thread may hold its objects, `implements Send`, `implements ThreadShared`, or an
  interface that needs them, a promise every class derived from it keeps (`[THR-8]`, `[THR-9]`); a
  final class's markers are read from its fields.
  - why: A reader of the class rules finds the header word where `open` and `abstract` are defined; the rule itself lives in THR-8, where `Send` is defined.

- FINAL EDIT rule=OWN-13
  - old: A method pins the class
  that declares it and every class derived from it, whose objects run it; a class pinned both ways,
  by its own methods or with a base's, is `E7015` at the one-thread line
  - new: A method pins the class
  that declares it and every class derived from it, whose objects run it; a class pinned both ways,
  by its own methods or with a base's, or pinned one-thread while it or a base promises `Send`
  (`[THR-8]`), is `E7015` at the one-thread line
  - why: OWN-13 reads a class's kind from its methods and already refuses a class pinned both ways; `weakref this` in a family that promises `Send` is the same contradiction and belongs in the same sentence.

- FINAL EDIT rule=IFC-3
  - old: `interface Ord: needs Eq` requires implementers of `Ord` to
  implement `Eq`.
  - new: `interface Ord: needs Eq` requires implementers of `Ord` to
  implement `Eq`: an implementation meets its interface's parents for every type it covers, checked
  where it is written (`E2040`, naming the part that does not and giving the line that compiles). A
  generic implementation covers every instance its brackets allow, so it writes the needs the
  parents put on its parameters (`extend[T: needs (Send, ThreadShared)] PoolFull[T] implements
  Error:`; `extend[T] PoolFull[T] implements Error:` is `E2040` with that line, never read as
  holding for some arguments only), as a `pub` generic function writes its bounds (`[IFC-6]`). A
  class that implements an interface with marker parents (`Send`, `ThreadShared`) promises them for
  every class derived from it, each checked where it is declared (`[THR-8]`), and the implementation
  itself is never checked again.
  - why: IFC-3 said implementers meet the parents but not that an implementation meets them for every type it covers; a generic error implementation over an unbounded `T` would put `PoolFull[shares Enemy]` into an `AnyError`. Stated once for every interface with parents (`Ord`, `IndexMut`, a program's `Command: needs Send`), never as a case of `Error`; the bound on a hand-written generic implementation is the library's promise (verdicts 108, 111) and the message gives the whole line (verdict 113's manner).

- FINAL EDIT rule=DRV-1
  - old: A derive whose requirement a field does not meet is `E2080`, naming the field.
  - new: A derive whose requirement a field does not meet is `E2080`, naming the field (`@derive(Error)` on
  a type with a `shares` field: `Error` needs `Send`, `[ERR-3]`). On a generic type the generated
  `extend` writes, as its parameters' needs, what its fields need of them (`@derive(Error)` above
  `[T]` and `enum SpawnError:` with `Rejected(T)` generates `extend[T: needs (Debug, Send,
  ThreadShared)] SpawnError[T] implements Error:`), so the derive holds for the instances that meet
  them, as `@derive(Copy)` above `[T]` and `pub struct Range:` is `Copy` when `T` is (ODR-027,
  `[STR-5]`, `[IFC-3]`); `E2080` stays for a field whose written type fails.
  - why: With `Error`'s new parents a derived generic error that hands the caller's value back must be decided; a derive is the compiler writing the implementation, so it writes the needs itself, as every derive on a generic type already behaves (`[STR-5]`, std's `Range`), and the generated text is what `ember inspect --expand` prints.

- FINAL EDIT rule=BLD-14 item 8
  - old: neither, which a line that makes an object of the class checks (`[OWN-13]`); and
  - new: neither, which a line that makes an object of the class checks (`[OWN-13]`); for an `open` or
     `abstract` class another package may derive from, the markers its header promises (`Send`,
     `ThreadShared`, an interface that needs them), which each class derived from it in another
     package is checked against where it is declared (`[THR-8]`, `E7015`); and for an `open` or
     `abstract` class no other package can derive from, whether its objects are `Send` and
     `ThreadShared`, read from the classes its package derives from it, which a dependant holding
     one through a `pub` type's field reads (`[THR-8]`); and
  - why: The promise is a written word of a `pub` class and crosses the edge in the interface file (`[MOD-9]`); a private family's inferred markers are a worked-out fact a dependant needs when a `pub` struct's private field holds one. Naming both keeps BLD-14 the complete record of what a dependant reads.

- FINAL EDIT rule=BLD-16, the list of whole-program facts
  - old: not by which of its values are made
  `thread_shared`, not by which of its classes a `weakref` names.
  - new: not by which of its values are made
  `thread_shared`, not by which of its classes a `weakref` names, not by which classes a dependant
  derives from one of its `open` or `abstract` classes (`[THR-8]`).
  - why: The case this ruling closes is exactly such a fact: a library judged `Script` `Send` from fields a dependant could add to. Naming it keeps the list the implementer's checklist.

- FINAL EDIT rule=Annex B §B.3 schema table, after the row "base class changed"
  - old: | base class changed | refused |
  - new: | base class changed | refused |
  | the promise an `open` or `abstract` class's header makes (`implements Send`, `implements ThreadShared`, an interface that needs them, `[THR-8]`) added or removed | refused (`E9030`) unless the image rebuilds every package that imports the class, whose derived classes and hand-overs are then checked again when they compile (`[HR-44]`) |
  - why: The unit of reload is the package. A reload of the base's package alone that adds the promise would let its new code hand `Script`s to threads while a derived class in a package the reload does not rebuild was never checked; one that removes it would leave a dependant's hand-over relying on a word that is gone. `[HR-44]` compares functions' worked-out facts, not a class's written promise, so the row says it; a rebuilt package is checked again when it compiles (HR-44's own words).

- FINAL EDIT rule=§XVII code table, row `E7015` (the cases)
  - old: a `thread_shared` object of a class pinned to one thread or with a `weakref T` field;
  - new: a `thread_shared` object of a class pinned to one thread or with a `weakref T` field; a field or a method that is not `Send` or `ThreadShared` in a class whose header, or a base's, promises it (`open class Script implements Send:`), at the class; a value held by the type of an `open` or `abstract` class another package may derive from, whose header promises nothing, reaching another thread; a method pinning a class one-thread while it or a base promises `Send`;
  - why: The row lists every refusal that carries the code; THR-8, THR-9 and OWN-13 now name three more, and `E7015` at the class is the code `[FFI-17d]` already uses for a one-thread field in a class whose objects any thread may reach (`threads=any`).

- FINAL EDIT rule=§XVII code table, row `E7015` (the rule column)
  - old: a one-thread owner in a trampoline class under `threads=any` | `[THR-1]`,
  - new: a one-thread owner in a trampoline class under `threads=any` | `[THR-9]`, `[CLS-4]`, `[THR-1]`,
  - why: The rule column names the rules that state the new cases (`[THR-8]` and `[OWN-13]` are already listed).

- FINAL EDIT rule=§XVII code table, row `E2040`
  - old: `unwrap` or `expect` on a `Result` whose error type has no `Display` | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]` |
  - new: `unwrap` or `expect` on a `Result` whose error type has no `Display`; an implementation that does not meet its interface's parents for every type it covers, at the implementation (a type holding a one-thread value implementing `Error`; a generic implementation without the needs its parents put on its parameters, given the bounded line); `ThreadShared` written by hand on a struct, an enum or a class that cannot be subclassed | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]`, `[IFC-3]`, `[ERR-3]`, `[TYP-43]` |
  - why: One code, one defining rule: the unmet parent is an unsatisfied bound, which is `E2040`'s meaning; the row says where it now falls.

- FINAL EDIT rule=§XVII code table, row `E3100`
  - old: `Send` declared by hand without `unsafe extend`;
  - new: `Send` declared by hand without `unsafe extend`, other than in the header of an `open` or `abstract` class, where it is a checked promise (`[TYP-43]`, `[THR-8]`);
  - why: The row would otherwise refuse the promise TYP-43 now allows; on a final class, a struct or an enum it keeps today's code.

- FINAL EDIT rule=§XVII code table, row `E9030`
  - old: hot reload refused; a kept view into a part that is removed, converted or rebuilt;
  - new: hot reload refused; the promise of an `open` or `abstract` class's header changed without every package that imports it rebuilt (Annex B §B.3, `[THR-8]`); a kept view into a part that is removed, converted or rebuilt;
  - why: The row lists the refusals the code carries; the new schema row adds one.

**final_cost_row**: No new row (`[COST-5]`): every judgement here is made when compiling, from a declaration the compiling package sees (the base's header in its source or its interface file, the derived class's own fields, `Error`'s declared parents), and no check runs at an upcast, a `?`, a spawn, a send or a join. The existing rows that apply, unchanged: \"What a function changes, reads and ends; the thread rules; the arena checks | not observable | all checked when compiling (`[EFF-24]`)\"; \"`?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`); a concrete error type moves without allocating\"; and \"An object handed whole to another thread | guaranteed present | one compare of the weak count for the handed object and for each object it owns ... one load and one test, then a call of the kind's hand-over routine, per owned value whose layout the handing package cannot see (held `dyn I`, of an `open` or `abstract` class, a function value's state); nothing for a value whose type can own no such object\", which an `AnyError` handed to another thread now pays for its boxed error, on the failure path only, exactly as any `owns dyn I` value does, and which does nothing for a kind whose family carries no weak count. The `@cached` row's clause \"on an object several threads reach, the result is filled once under a once-flag\" now reaches a `@cached` method of a class whose objects any thread may have (one that promises `ThreadShared`, or implements `Error`), a clause that already existed for every `ThreadShared` class. The run-time alternative was priced and turned down: a `Send` flag in the kind's `TypeInfo` tested at each hand-over of a base-typed object costs one load and one test but can only stop the program, since a `shares` owner inside the object cannot be moved away from its other owners (docs/AUTOPILOT.md §4, \"A panic has to make sense\").

**owner_verdicts_touched**: None overridden; no edit departs from a verdict. Kept and relied on: 61 (a type says which threads may have it, the owner's \"b\" over the compiler finding it from the whole program: extended from function types to the header of an open class); 94 (`thread.spawn` takes `thread_shared once fn() -> R`, `R` `Send`: unchanged, now met by `AnyError`); 110 (parents written with `needs`: all three copies of `Error`'s declaration); 108, 111 and 113 (a `pub` generic and an `extend` write their needs, the compiler gives the whole line: the generic error implementation writes `[T: needs (Send, ThreadShared)]` and the `E2040` message gives it); 73 (the given line compiles: every message gives the copy line first, and `thread_shared` only where `[THR-1]` then holds); 86 and 122 (facts worked out from bodies inside a package: the in-package inference for a family no other package can derive from is that shape, and a marker of a type is never a body fact); 164, 165, 166 (a `sharedref mut` link, a part that is not `ThreadShared`, a scoped task's counts: an error type holding one is refused where it implements `Error`; 165's making-line check is what the `thread_shared T` clause rests on); 23 (a `shares` value never used from two threads: none can sit inside an error, a derived class of a promising family, or a base-typed value that crosses); 114 and 101 (a handed object's links say gone, the 32-byte header: `[THR-2]`'s existing routine walks the boxed error, no header changes). Lead rulings touched: ODR-255 (`dyn I` Send/Sync when `I` requires it: its `ThreadShared` half is now written in THR-9); ODR-111 (`Send` by hand only as `unsafe extend`: kept for the vouch; the header promise is a compiler-checked bound and TYP-43 says so, reported for the lead); ODR-050 (`Result`'s error unbounded: kept, a one-thread type stays a `Result` error `main` prints); ODR-027 (a derive on a generic type holds per instance: now stated in DRV-1); N76/`[BLD-16]` (the class-family fact is read from a declaration the package sees). Noted for the lead: designs 2 and 3 cite verdict 132 as \"a declaration means what it says\"; verdict 132 in JOB.md is about `@ffi(const)` overlay entries, so the final text does not cite it. The owner gave no verdict on K20 itself (J6-16 was routed to him); this ruling is for him, with the cost stated: an error type holding a one-thread part is refused where it implements `Error`, even in a program with one thread, the fix being a copy of what the error reports, and a `pub open` class's objects cross threads only when its author writes the promise.

**remaining_doubt**: (1) The `thread_shared T` clause (a `thread_shared` object of an open `T` is `Send` and `ThreadShared` whatever its class promises) rests on every route to a `thread_shared` object passing `[THR-1]`'s making-line check on the concrete class: `thread_shared x` on a value not made so is `E7017`, upcasts keep the word, C never makes an Ember object. One existing gap it inherits (not caused): Annex B's row \"field added with a default | initialised to that default\" lets a reload of package A add a one-thread field to a class whose objects package B, not rebuilt, made `thread_shared`; that holds for `thread_shared Enemy` today, independent of this question, and belongs in a separate finding. If the lead prefers one rule only, drop the clause and `thread_shared Script` is judged by the promise, at the cost of that route. (2) `[FFI-17d]`'s \"`threads=any` requires the class to be `Send` and `ThreadShared` (`E7015` at the class, naming the field)\": for an open trampoline class this now means the header word, and the message should name it; FFI-17d is not edited here. (3) `[CLS-12]`'s \"an object several threads reach\" is a whole-program fact as written; under `[BLD-16]` it should be read from the class's declaration (it promises `ThreadShared`, implements `Error`, or its methods pin it `thread_shared`); an existing gap this ruling makes reachable, not its cause (the method-4 record's S2-2). (4) std under the new parents, read only: `io.Error`, `ParseError`, `CppError`, core.em's `Message` and `Missing` hold text and codes and pass; `SendError[T]` in `std/src/sync.em` implements no `Error` today, so nothing to bound; `pub open class AnyStorage` (`std/src/ecs.em` line 101) needs the promise only if a `World` is meant to cross threads (ODR-218); the implementer confirms each compiles. (5) `E7015` at the derived class (by `[FFI-17d]`'s precedent for a class whose objects any thread may reach) against `E2040` at an `implements Error` whose type is not `Send` (an unsatisfied parent): two codes divided as the table already divides them; design 1 used `E2040` for both and the lead may unify. (6) The in-package inference for a family no other package can derive from lets an added subclass with a `shares` field break a `thread.spawn` elsewhere in the same package, the message naming the subclass's field (verdict 86's shape); a lead who wants no split can require the word on every `open` class at the cost of one word in such programs. (7) Whether the reload row should also appear as a sentence in `[HR-11]`'s schema definition (\"its kind, base, layout attributes\") is the lead's placement; the row alone decides the program.
