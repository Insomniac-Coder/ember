# K20: Can AnyError, Result, a channel or a scoped task carry an error type across threads when that error holds an object of an open class another package may derive from?

## Design 1

**question**: K20: Can AnyError, Result, a channel or a scoped task carry an error type across threads when that error holds an object of an open class another package may derive from? In particular: how are Send and ThreadShared judged for an open class, and for a value holding one, so that a dependant's subclass cannot break a library's judgement (separate compilation, [BLD-16]), and what are Error's requirements and AnyError's markers?

**summary**: A value held by the type of an `open` or `abstract` class is type-erased exactly as `dyn I` and `fn()` are, and the text already decides those two by a word on the type (`[THR-8]`: "`dyn I` when `I` requires it (`interface I: needs Send:`)"; verdict 61: `thread_shared fn()`, the owner choosing the word over "the compiler finding it from the whole program"), and `[MOD-9]` says "Where a word exists to write a fact, only the written word counts across the edge". So the ruling gives the open class's markers a word: an `open` or `abstract` class writes `implements Send, ThreadShared` (the grammar's existing `implements` list, without `unsafe`), its objects held by base type have exactly the markers it declares, and every class derived from it, in any package or reload, meets them with its own fields or is refused where it is declared (`E2040`), while a class that cannot be subclassed stays inferred from its fields. On that footing `Error` requires `Send` and `ThreadShared` (any thread may have an error), so `dyn Error`, `AnyError` and every `Result[T]` cross threads by every route with nothing written, and nothing runs for any of it while the program runs.

- EDIT rule=CLS-4
  - old: cannot be overridden. Base fields are laid out first; there is at most one base.
  - new: cannot be overridden. Base fields are laid out first; there is at most one base. **An object held by the type of an `open` or `abstract` class may be of any class derived from it, in any package** (`[TYP-5]` rule 8), which the holder's package never sees (`[BLD-16]`); so what a holder may rely on of such a value is what the class declares, never what its own fields happen to hold: the interfaces it implements, the marker interfaces `Send` and `ThreadShared` included (`pub open class Script implements Send, ThreadShared:`, written there without `unsafe`, `[TYP-43]`), each of which every class derived from it meets, checked where that class is declared (`[IFC-3]`, `[THR-8]`, `[THR-9]`). A class that cannot be subclassed is judged from its fields, as a struct is.
  - why: This is where `open` and `abstract` are defined, and the cause: the kind of such an object is erased from its holder, and the text judged it from the declared class's fields alone, a judgement a dependant's subclass could break. Stating it as a declared promise makes the fact one `[MOD-9]`'s written-word rule already carries across the edge, so no whole-program fact enters any package's code.

- EDIT rule=THR-8
  - old: Structs, enums, tuples, fixed arrays, collections and class objects
  are `Send` when every component is;
  - new: Structs, enums, tuples, fixed arrays, collections and objects of a class that cannot be subclassed (`[CLS-4]`) are `Send` when every component is, a class object's base fields among them; an object held by the type of an `open` or `abstract` class, whose kind the holder cannot see, when the class implements `Send` (`open class Script implements Send:`, `[CLS-4]`; `E7015` otherwise at the line that hands it over: "`s` is a `Script`, an `open` class whose objects may be of any class derived from it, and `Script` does not say any thread may have them; write `pub open class Script implements Send:` (gamekit/src/lib.em line 3), after which every class derived from `Script` is checked for it where it is declared");
  - why: THR-8's list sentence "class objects are `Send` when every component is" is narrower than its own definition ("nothing reachable from it") for an open class, since the reachable fields are a derived class's. The list now says what the definition says, by the same shape as its own `dyn I` clause.

- EDIT rule=THR-9
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types, and objects of a class that cannot be subclassed, are `ThreadShared` when every component is; an object held by the type of an `open` or `abstract` class when the class implements `ThreadShared` (`[CLS-4]`), and `dyn I` when `I` requires it (`interface I: needs ThreadShared:`), as for `Send` (`[THR-8]`).
  - why: THR-9 named only structural types: class objects and `dyn I` had no ThreadShared rule at all, so `ref r` of a `Result[int]` given to a scoped task (`ref T` is `Send` when `T` is `ThreadShared`, `[THR-8]`) and an `AnyError` under a `thread_shared` `RwLock` (verdict 165) were undecided. ODR-255 ruled the `dyn` half and the text never said it.

- EDIT rule=§IV marker table, row `Send`
  - old: a class object is when its fields are;
  - new: a class object is when its fields are, where its class cannot be subclassed; an object held by the type of an `open` or `abstract` class when the class implements `Send`, and `dyn I` when `I` requires it (`[THR-8]`, `[CLS-4]`);
  - why: `[TYP-43]` says "the table answers them": the row must give the same answer as THR-8, or the two texts decide one program differently.

- EDIT rule=§IV marker table, row `ThreadShared`
  - old: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`);
  - new: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`); a class object when its fields are, where its class cannot be subclassed; an object held by the type of an `open` or `abstract` class when the class implements `ThreadShared`, and `dyn I` when `I` requires it (`[THR-9]`, `[CLS-4]`);
  - why: The same for the ThreadShared row, which named neither class objects nor `dyn`.

- EDIT rule=TYP-43
  - old: `ThreadShared` is written only in bounds, `Copy` comes from
  `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`,
  - new: `Send` and `ThreadShared` are also written, without `unsafe`, in the `implements` list of an `open` or `abstract` class (`pub open class Script implements Send, ThreadShared:`), whose objects' kinds a holder cannot see: there the word is the class family's promise, which the compiler checks against the class's own fields and, where each is declared, against every class derived from it (`[CLS-4]`, `[IFC-3]`, `E2040`), so it vouches for nothing; on a struct, an enum or a class that cannot be subclassed it is refused as below. Otherwise `ThreadShared` is written only in bounds, `Copy` comes from `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`,
  - why: TYP-43 allowed a hand-written `Send` only as an `unsafe extend` for a raw pointer, because there the compiler cannot check it. On an open class it can check every member of the family at that member's own line, so the written word is a checked declaration like `interface I: needs Send`, not a vouch; the rule must name the new place or `E3100` would refuse it.

- EDIT rule=OWN-13
  - old: A method pins the class
  that declares it and every class derived from it, whose objects run it; a class pinned both ways,
  by its own methods or with a base's, is `E7015` at the one-thread line
  - new: A method pins the class that declares it and every class derived from it, whose objects run it; a class pinned both ways, by its own methods or with a base's, or pinned one-thread while it or a base implements `Send` (`[CLS-4]`), is `E7015` at the one-thread line
  - why: OWN-13 reads a class's kind from its methods and already refuses a class pinned both ways. A method writing `weakref this` or `shares this` in a family that promises `Send` is the same contradiction, and belongs in the same sentence.

- EDIT rule=IFC-3
  - old: (ODR-042).
  - new: (ODR-042). An implementation meets its interface's parents for every type it covers: each instance of a generic implementation (`extend[T] PoolFull[T] implements Error:` covers every `T`, so it is `E2040`, the message giving the bounded line, `extend[T: needs (Debug, Display, Send, ThreadShared)] PoolFull[T] implements Error:`), and each class derived from a class that implements it, whose objects are values of that class (`[TYP-5]` rule 8), checked where the derived class is declared, by the package that declares it, whichever package declares the base (`[BLD-16]`): a parent the base's implementation gives it (`Eq`, `Display`) it has; a marker parent (`Send`, `ThreadShared`, `[CLS-4]`) it meets with its own fields, or is `E2040`, naming the field and the base's line and giving a line that holds a copy of what the field is used for ("`Enemy` is a `Script`, which implements `Send` and `ThreadShared` (gamekit/src/lib.em line 3), so every `Enemy` does too, and `Enemy.level` (line 8) is a `shares Level`, an owner kept on one thread; keep what `Enemy` reads from the level by copy: write `level_name: String` in place of line 8, or share the level between threads: `level: thread_shared Level`").
  - why: IFC-3 is where the language says what an implementation promises about its interface's parents; it said implementers meet them, not that an implementation meets them for every type it covers. A derived class is covered by its base's implementation (TYP-5 rule 8 upcasts it), so the check for a marker parent falls on the derived class's own fields at its own line, local to the package that can see them. The same sentence closes the generic-instance hole (`PoolFull[T]`) once, for every interface, instead of as a case of `Error`.

- EDIT rule=§IV.8 standard interfaces, the sketch
  - old: interface Error: needs (Debug, Display):
    fn source() -> Option[ref dyn Error]                            # default: None
  - new: interface Error: needs (Debug, Display, Send, ThreadShared):   # any thread may have an error
    fn source() -> Option[ref dyn Error]                            # default: None
  - why: This is the declaration every bound, every `dyn Error` and every implementer reads ("These are the definitions"); `std/src/core.em` line 80 follows it.

- EDIT rule=TYP-17
  - old: (`interface Error: needs (Debug, Display):`)
  - new: (`interface Error: needs (Debug, Display, Send, ThreadShared):`)
  - why: The generics chapter's example is a second copy of the declaration; left as it was, two texts would decide one program differently.

- EDIT rule=ERR-3
  - old: `Error` is the
  interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default
  `None`) (§IV.8).
  - new: `Error` is the interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8): any thread may have and read an error, so an error crosses threads by every route a value does (`[THR-2]`), and so do `dyn Error` and `AnyError` (`[THR-8]`, `[THR-9]`, `[ERR-8]`). A type holding a value that stays on one thread (a `shares` or `shares mut` owner, a `weakref T` or `sharedref mut` link, a `Cell`, an object held by the type of an `open` class that does not implement the markers) does not implement `Error`: `E2040` at its `implements Error` (`[IFC-3]`: "`Error` needs `Send` and `ThreadShared`, since any thread may have an error, and `LoadError.script` is a `Script`, an `open` class that does not say any thread may have its objects; write `pub open class Script implements Send, ThreadShared:` (line 3), or keep what the error reports by copy: `entity: int`"), `E2080` at `@derive(Error)`, naming the field (`[DRV-1]`). It may still be a `Result`'s error, matched or printed (`[ERR-1]`, ODR-050).
  - why: ERR-3 describes the declaration; the two must say one thing. It states one property (any thread may have an error) and leaves the placing of refusals to IFC-3 and DRV-1, and the old `Display + Debug` spelling goes (verdict 110).

- EDIT rule=ERR-8
  - old: An `AnyError` passed through `?` into another
  `AnyError` is handed over (`[ERR-7]`).
  - new: An `AnyError` passed through `?` into another `AnyError` is handed over (`[ERR-7]`). `AnyError` is `Send` and `ThreadShared` as its parts are: its `owns dyn Error` because `Error` requires both (`[ERR-3]`, `[THR-8]`, `[THR-9]`), its context a list of `String`; so a worker's `Result[int]` comes back through `join`, a channel or a scoped task's result with nothing written (`[THR-10]`, `[THR-15]`, `[THR-11]`), and the one-thread error it cannot carry was refused where that type was declared an error, never at the `?`.
  - why: The markers of the erased error type were nowhere stated; the gap sweep's program was refused at the spawn with no text saying so. AnyError needs no rule of its own: its markers follow from its fields, which the sentence says.

- EDIT rule=HR-44
  - old: 
  - new: A `pub` class's `implements` list, which the derived classes of other packages were checked against (`[CLS-4]`, `[IFC-3]`), is such a fact too: a reload whose new image removes a marker from it is refused (`E9030`) unless every package whose classes derive from it is rebuilt by the reload, which checks them again; a reload that adds a field to a derived class is checked at that class when the image compiles, before any plan.
  - why: Annex B compares worked-out facts; the promise a derived class in another package was checked against is a written word of the base, and a reload that takes it away would leave a checked class unchecked. The sentence names the two reload routes the method-4 record found open (a base losing its word, a derived class gaining a field).

- EDIT rule=code table, row `E7015`
  - old: a one-thread owner in a trampoline class under `threads=any` | `[THR-1]`
  - new: a one-thread owner in a trampoline class under `threads=any`; a value held by the type of an `open` or `abstract` class that does not implement `Send`, handed to another thread; a method pinning a class one-thread while it or a base implements `Send` | `[CLS-4]`, `[THR-1]`
  - why: The row lists every refusal that carries the code; THR-8 and OWN-13 now name two more.

- EDIT rule=code table, row `E2040`
  - old: has no `Display` | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]` |
  - new: has no `Display`; an implementation that does not meet its interface's parents for every type it covers: a generic implementation (the bounded line given), or a class derived from a class that implements the interface, at its own declaration (`Send` and `ThreadShared` among the parents: a derived class of a class implementing them, or an error type, holding a value that stays on one thread) | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]`, `[IFC-3]`, `[CLS-4]`, `[ERR-3]` |
  - why: One code, one defining rule: the unmet parent is an unsatisfied bound, which is E2040's meaning; the row says where it now falls.

- EDIT rule=code table, row `E3100`
  - old: `Send` declared by hand without `unsafe extend`;
  - new: `Send` declared by hand without `unsafe extend`, other than in an `open` or `abstract` class's `implements` list (`[TYP-43]`);
  - why: The row would otherwise contradict the edited TYP-43.

**refused_program**: # file gamekit/ember.toml
#   [package]
#   name = "gamekit"
#   version = "0.1.0"
#   language = "0.9.20"
#   kind = "lib"
# file gamekit/src/lib.em (line numbers count from its first line)
from std.fmt import Formatter, FmtError

pub open class Script implements Send, ThreadShared:    # line 3: the family's promise
    let entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct LoadError:
    pub script: Script

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the script could not load")

extend LoadError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("LoadError")

extend LoadError implements Error:                      # line 20: meets Send and ThreadShared, by line 3
    pass

# file game/ember.toml
#   [package]
#   name = "game"
#   version = "0.1.0"
#   language = "0.9.20"
#   [dependencies]
#   gamekit = { path = "../gamekit" }
# file game/src/main.em (line numbers count from its first line)
import std.thread
from gamekit import Script, LoadError

class Level:
    name: String = String("forest")

class Enemy(Script):                                    # line 7
    level: shares Level                                 # line 8

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn make_enemy(entity: int, owns level: shares Level) -> Result[int, LoadError]:
    return Err(LoadError(script=Enemy(entity, owns level)))   # an Enemy, upcast to Script

fn load(entity: int, owns level: shares Level) -> Result[int]:
    return Ok(make_enemy(entity, owns level)?)          # LoadError -> AnyError

fn main():
    level = shares Level()
    r = load(1, shares level)
    h = thread.spawn(owns fn() owns r: r.is_err())
    for i in 0..100000:
        other = shares level
        println(i, other.name.len())
    println(h.join())

# Refused in game's build, at game/src/main.em line 7, with nothing of gamekit compiled again:
#
# error[E2040]: `Enemy` is a `Script`, which implements `Send` and `ThreadShared`
#   (gamekit/src/lib.em line 3), so every `Enemy` does too, and `Enemy.level` (line 8) is a
#   `shares Level`, an owner kept on one thread; keep what `Enemy` reads from the level by copy:
#   write `level_name: String` in place of line 8, or share the level between threads:
#   `level: thread_shared Level`
#
# Had gamekit's line 3 been `pub open class Script:` (no promise), the refusal would instead fall in
# gamekit's own build, at line 20:
#
# error[E2040]: `Error` needs `Send` and `ThreadShared`, since any thread may have an error, and
#   `LoadError.script` is a `Script`, an `open` class that does not say any thread may have its
#   objects; write `pub open class Script implements Send, ThreadShared:` (line 3), or keep what the
#   error reports by copy: `entity: int`
#
# and game's `Enemy`, with its `shares Level`, would compile as any one-thread class does. In neither
# build does the `shares` owner reach the worker thread.

**accepted_program**: # file gamekit/src/lib.em: exactly as in the refused program (Script implements Send, ThreadShared;
# LoadError holds a Script and implements Error).
# file game/src/main.em
import std.fs
import std.thread
from gamekit import Script, LoadError

class Door(Script):                 # plain fields only: meets Script's promise, nothing written
    locked: bool = false

fn open_door(owns d: Door) -> Result[int, LoadError]:
    if d.locked:
        return Err(LoadError(script=owns d))     # a Door, upcast to Script, inside the error
    return Ok(d.entity)

fn load(path: StrView) -> Result[int]:
    text = fs.read_to_string(path)?             # io.Error -> AnyError
    d = Door(7)                                  # Script's init, inherited ([CLS-10])
    n = open_door(owns d)?                       # LoadError -> AnyError (it holds a Script: Send, by line 3)
    return Ok(text.len() + n)

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => load("level1.cfg"))   # R = Result[int, AnyError] is Send: compiles
    n = h.join()?                                        # AnyError -> AnyError, handed over ([ERR-7])
    println(n)
    return Ok(())

# Prints the file's length plus 7 and exits 0; with no such file, prints
# `error: <the system's message>` to standard error and exits with status 1 ([ERR-12]).
# Today the spawn line is E7015 with no text saying why. No thread word is written anywhere in game;
# gamekit wrote one, on the class whose objects it lets cross threads.

**runtime_cost**: Nothing. Every judgement here is made when compiling, from a declaration the compiling package sees: the base's `implements` list (in its own text, or in the imported interface file), the derived class's own fields, `Error`'s declared parents. The §X.4 row is the existing "What a function changes, reads and ends; the thread rules; the arena checks | not observable | all checked when compiling (`[EFF-24]`)". No new row (`[COST-5]`). Two existing rows become reachable for an `AnyError` that now crosses, unchanged: "`?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`)", paid before any hand-over as today; and "An object handed whole to another thread | guaranteed present | ... one load and one test, then a call of the kind's hand-over routine, per owned value whose layout the handing package cannot see (held `dyn I`, of an `open` or `abstract` class, a function value's state)", which the boxed `dyn Error` pays once per `AnyError` handed to another thread, on the failure path only, and which does nothing for a kind whose family carries no weak count. The existing "`@cached` method ... on an object several threads reach, the result is filled once under a once-flag" row now covers a `@cached` method of a class whose objects any thread may have; which classes those are must be read, under `[BLD-16]`, from the class's own declaration (it implements `ThreadShared`, or `Error`), which `[CLS-12]` should say (noted below, not this question's cause). The run-time alternative was priced and turned down: a `Send` flag in the kind's `TypeInfo` tested at each hand-over of a base-typed object would cost one load and one test (the load the hand-over routine already makes), but it can only stop the program, since a `shares` owner inside the object cannot be moved away from its other owners; a compile-time refusal with a line to write would become a stop on the least-tested path, for a conflict between two packages that is nobody's bug at the stopping line (docs/AUTOPILOT.md §4, "A panic has to make sense").

**implementer_steps**: 1. Parser: nothing; `class_decl` already has `[implements]` ("implements" type {"," type}). Name resolution: accept `Send` and `ThreadShared` in the `implements` list of a class declared `open` or `abstract`; keep `E3100` (Send) and `E2040` (ThreadShared) for the list of a struct, enum or final class, with the message naming the class as one that cannot be subclassed. 2. Marker inference (the function behind `[TYP-43]`'s table), for a class type C: if C is final, infer from its fields, base fields included, as for a struct; if C is `open` or `abstract`, answer from C's declared markers: those in its `implements` list, those in the `implements` lists of its bases, and those among the `needs` of any interface it or a base implements; never from C's fields. For `dyn I`, answer both `Send` and `ThreadShared` from I's `needs` (THR-9's new clause). 3. Class checker, per class D declared in the compiling package (its own source only): collect the interfaces D implements, inherited from bases through their declarations in imported interface files; for each marker among them (direct or via an interface's parents) infer D's own fields' markers, and refuse `E2040` at D's declaration when one fails, naming the field, the base's file and line, and giving the copy line (the field's type with the one-thread word removed, or `String`/`int` for what it reads) and, where it compiles under `[THR-1]`, the `thread_shared` line. Also: if a method of D, or of a base, pins the class one-thread (`[OWN-13]`'s list) while the family implements `Send`, `E7015` at the pinning line. 4. `[IFC-3]` for generic implementations: for `extend[Ps] X[Ps] implements I`, check I's parents hold for `X[Ps]` under the written bounds alone; where one does not, `E2040` giving the line with the bounds added (`[IFC-6]`'s message shape). 5. Hand-over checks (`[THR-2]`, `[THR-10]`, `[THR-15]`, `[THR-1]`, `[THR-11]`): unchanged code paths; they call step 2, so a base-typed value of a class without `implements Send` is `E7015` with the new message naming the class and its declaration line. 6. `std/src/core.em` line 80: `pub interface Error: needs (Debug, Display, Send, ThreadShared):`; the IV.8 sketch and TYP-17's example with it. `@derive(Error)` generates its `extend` as today; a field that fails a marker is `E2080` naming the field (DRV-1's existing rule), and a generic type's derive holds per instance as `[STR-5]` already reads every derive. `AnyError` needs no code: its markers follow from `Box[dyn Error]`, `u128` and `Array[String]`. 7. Interface files: no new record. The `implements` list of a `pub` class is already part of its declaration there; `[MOD-9]`'s written-word rule carries it. 8. Hot reload (Annex B): the plan's comparison (`[HR-44]`) adds the `pub` classes' `implements` lists to the facts it compares; a removed marker refuses the reload unless the packages deriving from the class are rebuilt by it; a rebuilt derived class is checked by step 3 when the image compiles. A module loaded at run time (`[ABI-1]`) compiles its derived classes against the base's declaration the same way. 9. FFI: no code; an Ember class extending a C++ class under `threads=creator` (`[FFI-17d]`) is thread-bound (`E7008`), so step 3 refuses `implements Send` on it as on any class whose objects are not `Send`. 10. Tests, each with its break test: the accepted program as a run-pass (two packages, a worker returning `Result[int]`); the refused program as `E2040` at the derived class, and its variant without the promise as `E2040` at the library's `implements Error`; `E7015` at a spawn of a base-typed value of a class without the promise; `E3100` for `implements Send` on a final class; `E7015` for `weakref this` in a `Send` family; `E2040` for `extend[T] PoolFull[T] implements Error:`; a reload test removing `Send` from a `pub open` class with a derived class in another package (`E9030`); `channel[Result[int]]` and a scoped task's `ref r` of a `Result[int]` as run-pass; a `Result[int, DocError]` with `doc: shares Doc`, matched and printed, still compiling (ODR-050).

**departs_from_verdict**: None. Verdict 61 (the type says whether any thread may run its functions, chosen over the compiler finding it from the whole program): kept and followed; an open class is erased as `fn()` is, and the ruling puts the word on the class. Verdict 86 (the compiler works out what functions do from code): kept; markers of a type were never a body fact, and a final class stays inferred. Verdict 94 (`thread.spawn` takes `thread_shared once fn()`, `R` Send): kept; `AnyError` now meets `R: Send`. Verdict 110 (an interface's parents with `needs`): kept, `interface Error: needs (Debug, Display, Send, ThreadShared):`. Verdicts 108, 111, 113 (a `pub` generic and `extend` write their needs): kept; the generic error implementation writes its bound, and the message gives the line. Verdict 164 (a `sharedref mut` link is neither marker): kept; an error holding one is refused at its `implements Error`. Verdict 165 (`thread_shared` refused where the value is made when not ThreadShared): kept; a `thread_shared` holder of `Result[int]`s is now accepted because `AnyError` is `ThreadShared`. Verdict 166 (a scoped task touches no one-thread count): kept. Verdict 23 (a `shares` value never used from two threads): kept, no `shares` owner can be inside an error, a derived class of a `Send` family, or a base-typed value that crosses. Verdict 114 and 101 (hand-over leaves links behind; the 32-byte header): kept; the existing hand-over routine walks the boxed error. ODR-050 (`Result`'s error parameter unbounded): kept; a one-thread error type is still a `Result`'s error, printed by `Display`. ODR-255 (`dyn I` Send/Sync when I requires it): now written for ThreadShared in THR-9. ODR-111 (a lead's ruling, not an owner verdict: `Send` by hand only with `unsafe extend`): touched, not reversed in substance; the new written place is checked by the compiler for every member of the family, so it vouches for nothing, which is why `unsafe` is not written there; reported for the lead.

**trade_offs**: Given up: (1) A base-typed value of an open class with plain fields and no `implements Send` cannot cross threads, even when the object was made on the line (`s = Script()`); the fix is one line in the class's package, the cost `dyn I` already carries today. (2) Once an open class promises the markers, every class derived from it, in every package, gives up one-thread fields (`shares`, `weakref T`, `sharedref mut`, `Cell`), refused at its own line with a copy line; ordinary hierarchies whose base makes no promise are untouched. (3) An error type cannot hold a one-thread owner, even in a one-thread program (Rust's anyhow `Send + Sync` and Swift's `Error: Sendable` take the same); it may still be a `Result`'s error, matched and printed, and `@derive(Error)` gives the copy line. (4) A hand-written generic error implementation writes `T: needs (Debug, Display, Send, ThreadShared)`; a derive writes nothing. Kept, against the four goals: C-like speed, nothing runs (the only reachable costs are rows already in the table, on the failure path); Rust-like safety, every check local to a package that can see what it checks, no whole-program fact in any package's code (`[BLD-16]`), the separate-compilation, reload and run-time-module routes all checked at the derived class's own line; Python-like readability, one word on the one thing that crosses threads (the class), nothing on workers, channels, results or `?`; consistency, `dyn I`, `thread_shared fn()` and an open class now decided by one principle (an erased type's markers are declared on it), and `[MOD-9]`'s written-word rule given the word it lacked. The alternatives: inferring the open class's markers from its own fields and making subclasses keep them (option A of the task) costs nothing and is local, but forbids one-thread fields in every subclass of every plain-field base with no way for the base author to decline the promise, which refuses ordinary one-thread games; never-Send-unless-final (option B) is local and free but refuses every polymorphic hand-over and every error holding a base-typed object, with no line to write but "make it final"; the method-4 team's count-every-derived-class puts a whole-program fact in the library's code (`[BLD-16]`) and was shown unsound across a package edge; a recorded "this line relied on Script's Send" (MOD-9a's shape) is local but needs a data-flow record from every relying line to every upcast in every package, far larger than one word; the run-time flag is priced under runtime_cost and can only stop the program.

**remaining_doubt**: (1) Whether to accept a base-typed value whose exact kind the compiler sees on the line (`s = Script()` handed over at once) without the class's promise: a small flow rule would allow it; left out as not the smallest change, so the `dyn I` cost applies. (2) `[CLS-12]`'s "an object several threads reach" is a whole-program fact as written; under `[BLD-16]` it should be read from the class's declaration (it implements `ThreadShared` or `Error`, or its methods pin it `thread_shared`); that is the method-4 record's S2-2 and an existing gap this ruling only makes reachable, not its cause, so no edit here. (3) Two spellings reach one promise: `implements Send` directly, or `implements Command` with `interface Command: needs Send`; both exist in the grammar today and both are checked by the one IFC-3 sentence, so I kept both rather than refuse one. (4) `Error` requires `ThreadShared` as well as `Send`; `Send` alone would admit an error holding a `Cell`, but then `ref r` to scoped tasks and a `thread_shared` table of results (verdict 165's shape) would be refused; both markers follow Rust's anyhow and Swift. (5) The HR-44 sentence assumes a reload image can rebuild dependant packages or refuse; if Annex B's plan compares only the reloadable package's own facts, the lead may prefer the sentence in `[HR-11]` (the `implements` list as part of a class's schema). (6) The E2040 message's second fix (`level: thread_shared Level`) compiles only where no line then changes the level directly (`[THR-1]`); the message should offer it only then, as the method-4 record's objection E1-2 asked; the message rule in IFC-3 gives the copy line first for that reason.

## Design 2

**question**: K20: Can AnyError, Result, a channel or a scoped task carry an error type across threads when that error holds an object of an open class another package may derive from? In particular: how are Send and ThreadShared judged for an open or abstract class, and for a value held by that class's type, so that a dependant's subclass (compiled separately, or added by a reload) cannot break a library's judgement ([BLD-16]); then what Error requires and what AnyError's markers are.

**summary**: An `open` or `abstract` class that another package may derive from says in its own header whether any thread may hold its objects: `open class Script implements Send, ThreadShared:`. Without the word no value of that class type is `Send` or `ThreadShared`, whatever its fields (its value may be an object of a class the package never sees); with it, the promise binds the class and every class derived from it, checked at each class's own declaration in its own package over all its fields (E7015 at the subclass, naming the field), so a dependant's subclass can never break a library's judgement and an upcast needs no check (the shape of `interface I: needs Send` for `dyn I`, of `[OWN-13]`'s method pinning and of `[OBJ-1]`'s family-wide weak count; an open class no other package can subclass is judged from the derived classes its own package sees, as `[EFF-2a]` already does for its virtual calls). On that footing `Error` requires `Send` and `ThreadShared`, so `dyn Error` and `AnyError` are both, and a worker returns `Result[int]`, a channel carries it and a scoped task reads it with nothing written; an error type holding a one-thread part is `E2040` where it implements `Error`, with a copy line. Nothing runs while the program runs.

- EDIT rule=TYP-17 (the example of an interface's parents)
  - old: (`interface Error: needs (Debug, Display):`)
  - new: (`interface Error: needs (Debug, Display, Send, ThreadShared):`)
  - why: The third copy of Error's declaration; left as it was, the generics chapter would decide a program differently from §IV.8 and [ERR-3].

- EDIT rule=§IV.8 Standard interfaces, the signature sketch (the definition [ERR-3] cites)
  - old: interface Error: needs (Debug, Display):
    fn source() -> Option[ref dyn Error]                            # default: None
  - new: interface Error: needs (Debug, Display, Send, ThreadShared):      # any thread may hold and read an error ([ERR-3])
    fn source() -> Option[ref dyn Error]                            # default: None
  - why: This is the declaration every bound, every `dyn Error` and every implementer reads; `std/src/core.em` line 80 follows it. `Error` was the one erasing interface that said nothing about threads, so `dyn Error`, and with it `AnyError` and the default `Result[T]`, could never leave their thread, which no rule said.

- EDIT rule=BLD-16 (the list of whole-program facts a package never reads)
  - old: not by which of its values are made
  `thread_shared`, not by which of its classes a `weakref` names.
  - new: not by which of its values are made
  `thread_shared`, not by which of its classes a `weakref` names, not by which classes a dependant
  derives from one of its `open` or `abstract` classes (`[THR-8]`).
  - why: The case this ruling closes is exactly such a fact: a library judged `Script` `Send` from fields a dependant could add to. Naming it here keeps the list the implementer's checklist.

- EDIT rule=CLS-4
  - old: A class is final unless declared `open` or
  `abstract`.
  - new: A class is final unless declared `open` or
  `abstract`. An `open` or `abstract` class says in its header whether any thread may hold its
  objects, `implements Send`, `implements ThreadShared`, a promise every class derived from it keeps
  (`[THR-8]`, `[THR-9]`); a final class's markers are read from its fields.
  - why: A reader of the class rules finds the header word where classes are defined; the rule itself lives in [THR-8], where `Send` is defined.

- EDIT rule=§IV.8 marker table, `Send` row
  - old: a class object is when its fields are;
  - new: a class object is when its fields are, for a class that is neither `open` nor `abstract`, and when its header writes `implements Send`, for an `open` or `abstract` class, a promise every class derived from it keeps (`[THR-8]`); `dyn I` when `I` requires it;
  - why: [TYP-43] says the table answers the markers; "a class object is when its fields are" is the narrow reading that let a base-typed value holding a one-thread derived object through, and would contradict the edited [THR-8].

- EDIT rule=§IV.8 marker table, `ThreadShared` row
  - old: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`);
  - new: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`); a class object as for `Send`: by its fields for a class that is neither `open` nor `abstract`, by `implements ThreadShared` in its header for an `open` or `abstract` class (`[THR-8]`); `dyn I` when `I` requires it (`[THR-9]`);
  - why: The same answer as [THR-9]; the row had no words for a class object or a `dyn`, and `ref AnyError` to a scoped task, or an `AnyError` under a `thread_shared` holder's lock, is decided by this row.

- EDIT rule=THR-9
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types, and objects of a class that is neither `open` nor `abstract`, are `ThreadShared`
  when every component is; an object of an `open` or `abstract` class only when its header writes
  `implements ThreadShared`, a promise every class derived from it keeps, checked at each class's
  declaration (`[THR-8]`); `dyn I` when `I` requires it (`interface I: needs (Send, ThreadShared):`,
  ODR-255), as for `Send`.
  - why: [THR-9] had the same gap as [THR-8] for class objects and no words for a `dyn` at all; ODR-255 ruled the `dyn` half and the text never said it. `AnyError`'s `ThreadShared` rests on this clause.

- EDIT rule=DRV-1
  - old: A derive whose requirement a field does not meet is `E2080`, naming the field.
  - new: A derive whose requirement a field does not meet is `E2080`, naming the field (`@derive(Error)` on a
  type with a `shares` field: `Error` needs `Send`, `[ERR-3]`). On a generic type the generated
  `extend` writes, as its parameters' needs, what its fields need of them (`@derive(Error)` on `[T]`
  above `enum SpawnError:` with `Rejected(T)` generates `extend[T: needs (Debug, Send, ThreadShared)]
  SpawnError[T] implements Error:`), so the derive holds for the instances that meet them, as
  `@derive(Copy)` on `[T]` above `pub struct Range:` is `Copy` when `T` is (ODR-027, `[IFC-3]`).
  - why: With `Error`'s new parents a derived generic error that hands the caller's value back must be decided; a derive is the compiler writing the implementation, so it writes the needs itself, as every derive on a generic type already behaves (`[STR-5]`, std's `Range`).

- EDIT rule=HR-11 (Annex B)
  - old: A field's type schema includes the words its type is
  written with (`owns`, `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut`, `ref`,
  `ref mut`, `using A`), so a change of one is a change of the field's type.
  - new: A field's type schema includes the words its type is
  written with (`owns`, `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut`, `ref`,
  `ref mut`, `using A`), so a change of one is a change of the field's type. A class's schema
  includes the promise its header writes (`implements Send`, `implements ThreadShared`, `[THR-8]`): a
  new image that drops one from a class is refused in plan (`E9030`) while code the reload does not
  rebuild, another package's or a frame below a checkpoint, was checked against it, as `[HR-44]`
  refuses a changed fact; a derived class rebuilt with a field that breaks a promise is refused where
  it compiles, before any plan.
  - why: Hot reload is a route to the same race: a reload that adds a `shares` field to a derived class, or removes the base's promise, must meet the same check. The added field is refused by the rebuilt package's own compile; the dropped promise is a schema change that plan compares, so nothing runs for it.

- EDIT rule=§XVII code registry, row `E2040`
  - old: `unwrap` or `expect` on a `Result` whose error type has no `Display` | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]` |
  - new: `unwrap` or `expect` on a `Result` whose error type has no `Display`; an implementation that does not meet its interface's parents for every type it covers (a field that is not `Send` in a type implementing `Error`; a generic implementation without the needs its parents put on its parameters), at the implementation; `implements Send` or `implements ThreadShared` on a struct, an enum or a class that cannot be subclassed | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[ERR-4]`, `[IFC-3]`, `[ERR-3]`, `[TYP-43]` |
  - why: One code, one defining rule: the row lists the refusals [IFC-3], [ERR-3] and [TYP-43] now state.

- EDIT rule=§XVII code registry, row `E7015`
  - old: a `thread_shared` object of a class pinned to one thread or with a `weakref T` field;
  - new: a `thread_shared` object of a class pinned to one thread or with a `weakref T` field; a field that is not `Send` or `ThreadShared` in an `open` or `abstract` class whose header promises it, or in a class derived from one that does, at the class; a value of an `open` or `abstract` class type with no such promise reaching another thread, `thread_shared` on a new object of such a class included;
  - why: The registry row names every refusal under the code; [THR-8] is already among its rules.

- EDIT rule=TYP-43
  - old: `ThreadShared` is written only in bounds, `Copy` comes from
  `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`, as is an
  `unsafe extend` of any interface but `Send` or an `unsafe interface` (ODR-111).
  - new: `ThreadShared` is written only in bounds, `Copy` comes from
  `@derive(Copy)`, and `Sized` is never written: each of these written by hand is `E2040`, as is an
  `unsafe extend` of any interface but `Send` or an `unsafe interface` (ODR-111); the one other place
  `Send` and `ThreadShared` are written is the header of an `open` or `abstract` class (`open class
  Script implements Send, ThreadShared:`), where each is a promise for every class of the family that
  the compiler checks at each class's declaration (`[THR-8]`), so it needs no `unsafe`; on a struct,
  an enum or a class that is neither `open` nor `abstract`, whose markers are inferred, it is `E2040`
  ("`Door` cannot be subclassed, so its `Send` is read from its fields; remove `implements Send`").
  - why: [TYP-43] refused every hand-written marker but the `unsafe extend` vouch; the header promise is a bound the compiler checks, as `interface I: needs Send` is, not a vouch, so it is allowed without `unsafe` and refused where inference already answers. The grammar's `class_decl` already parses `implements Send`; no grammar change.

- EDIT rule=IFC-3
  - old: `interface Ord: needs Eq` requires implementers of `Ord` to
  implement `Eq`.
  - new: `interface Ord: needs Eq` requires implementers of `Ord` to
  implement `Eq`: an implementation meets its interface's parents for every type it covers, checked
  where it is written (`E2040`, naming the part that does not and giving the line that compiles). A
  generic implementation covers every instance its brackets allow, so it writes the needs the parents
  put on its parameters (`extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:`;
  `extend[T] PoolFull[T] implements Error:` is `E2040` with that line, since a declaration means what
  it says, verdict 132), as a `pub` generic function writes its bounds (`[IFC-6]`). An `open` or
  `abstract` class meets a marker parent through the promise in its header, which every class derived
  from it keeps (`[THR-8]`), so a derived class in another package is checked at its own declaration
  and the implementation is never checked again.
  - why: [IFC-3] said implementers meet the parents but not that an implementation meets them for every type it covers; a generic error implementation over an unbounded `T` would put `PoolFull[shares Enemy]` into an `AnyError`. Stated once for every interface with parents (`Ord`, `IndexMut`, a program's `Command: needs Send`), never as a case of `Error`.

- EDIT rule=ERR-8 (append)
  - old: 
  - new: `AnyError` is `Send` and `ThreadShared`: its `owns dyn Error` is, since `Error` requires both
  (`[THR-8]`, `[THR-9]`), and its context chain is text. So a worker returns `Result[int]` (`R` is
  `Send`, `[THR-10]`, `[THR-11]`), a channel carries one (`[THR-15]`), a `thread_shared` holder keeps
  one under its lock (`[THR-1]`) and a scoped task reads one through `ref`. Nothing is checked at `?`:
  every `E: needs Error` met the requirement where it implemented `Error`, in its own package, and an
  `AnyError` handed to another thread walks the error it holds by that error's own hand-over routine
  (`[THR-2]`), as any `owns dyn I` value is walked.
  - why: The question's missing words: the erased error's markers, stated by what it may hold. `From[E]` for every `E: needs Error` stays true as written, and the `?` line stays free of any check.

- EDIT rule=ERR-3
  - old: `Error` is the
  interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default
  `None`) (§IV.8).
  - new: `Error` is the
  interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with
  `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8): any thread may hold and read an
  error, so `dyn Error`, and every `Result` whose error is one, goes to a worker thread, through a
  channel and into a scoped task's view as any `Send` and `ThreadShared` value does (`[THR-8]`,
  `[THR-9]`). A type holding a value that stays on one thread (a `shares` owner, a `weakref T` link, a
  `Cell`, an object of an `open` class whose header makes no promise, `[THR-8]`) is not an error:
  where it implements `Error` it is `E2040`, naming the field and giving the line that compiles
  (`[IFC-3]`; "`LoadError.script` is a `Script`, an `open` class whose header does not say that any
  thread may hold its objects, so a `LoadError` may hold an object of a class another package derives
  from `Script`, and `Error` needs `Send` and `ThreadShared`; write `pub open class Script implements
  Send, ThreadShared:` (line 3), which every class derived from `Script` then keeps, or keep the
  script's number, `entity: int`"), and at `@derive(Error)` it is `E2080` (`[DRV-1]`); it may still be
  a `Result`'s error whose `Display` `main` prints (`[ERR-12]`, ODR-050).
  - why: [ERR-3] describes the declaration of §IV.8 and must say the same thing; the `Display + Debug` spelling goes (verdict 110). It states one property, any thread may hold an error, and leaves the cases to [THR-8], [IFC-3] and [DRV-1].

- EDIT rule=THR-8 (the cause: the class-object clause of the `Send` list)
  - old: Structs, enums, tuples, fixed arrays, collections and class objects
  are `Send` when every component is; an `owns T` when `T` is; `ref T` and `Span[T]` when `T` is
  `ThreadShared`, `ref mut T` and `MutSpan[T]` when `T` is `Send`; a `thread_shared T`, and a
  `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`; `dyn I` when `I` requires it
  (`interface I: needs Send:`).
  - new: Structs, enums, tuples, fixed arrays, collections and objects of a class that is
  neither `open` nor `abstract` are `Send` when every component is; an `owns T` when `T` is; `ref T`
  and `Span[T]` when `T` is `ThreadShared`, `ref mut T` and `MutSpan[T]` when `T` is `Send`; a
  `thread_shared T`, and a `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`; `dyn I`
  when `I` requires it (`interface I: needs Send:`). **An `open` or `abstract` class says in its
  header whether any thread may hold its objects.** A value of such a class type may be an object of
  any class derived from it, in a package the class's package never sees (`[TYP-5]` rule 8,
  `[BLD-16]`), so its own fields decide nothing for the type: it is `Send` only when its header
  writes `implements Send`, and `ThreadShared` only when it writes `implements ThreadShared` (`open
  class Script implements Send, ThreadShared:`), a promise for the class and every class derived from
  it, as a method pins a class and its derived classes (`[OWN-13]`) and a weak count is carried by a
  whole family (`[OBJ-1]`). The promise is checked where each class of the family is declared, by
  that class's own package, over every field of the class, inherited ones included, and nothing
  else: a field that is not `Send`, or not `ThreadShared`, where the class or a base promises it is
  `E7015` at the class, naming the field and the promise ("`Enemy.level` is a `shares Level` owner,
  which stays on one thread, and `Script` promises that any thread may hold its objects (`pub open
  class Script implements Send, ThreadShared:`, gamekit/src/lib.em line 3), a promise every class
  derived from it keeps; keep what the enemy needs as a copy, `level_name: String`, or share the
  level between threads, `level: thread_shared Level`, changed only through its own operations
  (`[THR-1]`)"). A derived class may write a promise its base does not; it holds for it and its
  derived classes, never for the base. On a generic class the promise holds for each instance whose
  arguments meet the needs its parameters are bounded by, as a generic `unsafe extend` does
  (`[TYP-43]`). So an upcast keeps the markers with no check (`[TYP-5]` rule 8), and a package reads
  the promise from the declaration it imports (`[MOD-9]`: a word exists, so only the written word
  counts), never from the classes a dependant derives. An `open` or `abstract` class that cannot be
  subclassed outside its package (`[EFF-2a]`) needs no promise: its markers are read from the fields
  of every class its package derives from it, as a call through its `virtual` methods counts every
  override the package has (`[EFF-2a]`, `[CLS-2]`), and a refusal names the derived class and its
  field. A value of a class type whose header makes no promise it needs, reaching another thread by
  any route of `[THR-2]`, `thread_shared Script()` included (`[THR-1]`), is `E7015` at that line,
  whose message gives the promise line, or the value held by a class that cannot be subclassed
  ("`s` is a `Script`, an `open` class whose header does not say that any thread may hold its
  objects, so it may be an object of a class another package derives from `Script`; write `pub open
  class Script implements Send:` (line 3), or hand the object over by its own class, `d: Door`").
  - why: This is the cause. "class objects are `Send` when every component is" counts the declared class's fields, narrower than [THR-8]'s own definition ("nothing reachable from it"), and for an `open` class the components are a whole-program fact [BLD-16] forbids a package to read. Making the family's markers a word of the root's declaration, checked at every class's own line, is the one shape that is local to each package, needs no check at an upcast or a hand-over, and refuses the one-thread field at the class that holds it. It is the shape the text already uses for `dyn I` (`needs Send`), for a class's kind ([OWN-13]) and for the weak count ([OBJ-1]), and the owner chose it for function types over a whole-program search (verdict 61).

**refused_program**: # Two packages, compiled separately. The library writes the promise; the game's subclass breaks it.

# file gamekit/ember.toml
#   [package]
#   name = "gamekit"
#   version = "0.1.0"
#   language = "0.9.20"
#   kind = "lib"
# file gamekit/src/lib.em
from std.fmt import Formatter, FmtError

pub open class Script implements Send, ThreadShared:      # line 3: any thread may hold a Script
    pub entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct LoadError:
    pub script: Script

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the script could not load")

extend LoadError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("LoadError")

extend LoadError implements Error:                          # Error needs Send and ThreadShared: met, by the promise
    pass

pub fn fail(owns s: Script) -> Result[int, LoadError]:
    return Err(LoadError(script=s))

# file game/ember.toml
#   [package]
#   name = "game"
#   version = "0.1.0"
#   language = "0.9.20"
#   [dependencies]
#   gamekit = { path = "../gamekit" }
# file game/src/main.em
import std.thread
from gamekit import Script, fail

class Level:
    name: String = String("forest")

class Enemy(Script):                                        # line 7
    level: shares Level                                     # line 8

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn tick(owns s: Script) -> Result[int]:
    return Ok(fail(owns s)?)                                # LoadError -> AnyError

fn main():
    level = shares Level()
    r = tick(owns Enemy(1, shares level))                   # r's error would hold a second owner of the Level
    h = thread.spawn(owns fn() owns r: r.is_err())          # r would go to another thread, which ends it
    for i in 0..100000:
        other = shares level                                # plain +1 and -1 on this thread
        println(i, other.name.len())
    println(h.join())

# Refused in game's build, at the class, before any line that sends anything:
# error[E7015]: game/src/main.em:8: `Enemy.level` is a `shares Level` owner, which stays on one thread,
#   and `Script` promises that any thread may hold its objects (`pub open class Script implements Send,
#   ThreadShared:`, gamekit/src/lib.em line 3), a promise every class derived from it keeps; keep what the
#   enemy needs as a copy, `level_name: String`, or share the level between threads, `level: thread_shared
#   Level`, changed only through its own operations ([THR-1]).
#
# Had gamekit written `pub open class Script:` with no promise, gamekit's own build is refused instead, at
# its `extend LoadError implements Error:`:
# error[E2040]: gamekit/src/lib.em:19: `LoadError.script` is a `Script`, an `open` class whose header does
#   not say that any thread may hold its objects, so a `LoadError` may hold an object of a class another
#   package derives from `Script`, and `Error` needs `Send` and `ThreadShared`; write `pub open class Script
#   implements Send, ThreadShared:` (line 3), which every class derived from `Script` then keeps, or keep
#   the script's number, `entity: int`.
# Under the current text the race above is reached only through the erased route (today AnyError is not
# Send, so the spawn is E7015 with no explanation); under the method-4 text it compiled and raced.

**accepted_program**: # The same library, and a game whose subclass keeps the promise. Nothing thread-related is written in the
# game: the default error type comes back from a worker thread.

# file gamekit/src/lib.em  (package gamekit, kind = "lib", as in the refused program)
from std.fmt import Formatter, FmtError

pub open class Script implements Send, ThreadShared:
    pub entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct LoadError:
    pub script: Script

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"script {this.script.entity} could not load")

extend LoadError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("LoadError")

extend LoadError implements Error:
    pass

pub fn fail(owns s: Script) -> Result[int, LoadError]:
    return Err(LoadError(script=s))

# file game/src/main.em  (package game, depends on gamekit)
import std.fs
import std.thread
from gamekit import Script, fail

class Door(Script):                     # checked here: `name` is Send and ThreadShared, the promise holds
    name: String = String("gate")

fn load(path: StrView) -> Result[int]:  # fails with any error: AnyError
    text = fs.read_to_string(path)?     # io.Error -> AnyError
    return Ok(text.len())

fn tick(owns s: Script) -> Result[int]:
    return Ok(fail(owns s)?)            # LoadError -> AnyError; the error carries a Door

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => load("level1.cfg"))   # R = Result[int, AnyError], Send ([THR-10])
    g = thread.spawn(owns fn() => tick(owns Door(7)))   # a Script that is a Door crosses inside an AnyError
    n = h.join()?                                        # the worker's error, if any, is main's
    println(n)
    match g.join():
        Ok(_) => println("loaded")
        Err(e) => println(e)                             # prints: script 7 could not load
    return Ok(())
# Prints the file's length and `script 7 could not load`, or `error: <the system's message>` with status 1
# when level1.cfg is missing. Every hand-over of an AnyError is one move; the Door's weak-count compare at
# the hand-over goes through LoadError's own hand-over routine ([THR-2]), as for any `owns dyn I` value.

**runtime_cost**: Nothing new. `Send` and `ThreadShared` are inferred and checked when compiling; the promise is a header word read from the class's declaration (in its package, or in its interface file), and each class of a family is checked once, at its declaration, over the marker properties every type already has (`[TYP-1]`). No check at an upcast, at `?`, at a spawn, at a send or at a join. What runs is already in §X.4: the `[COST-3]` row "An object handed whole to another thread" ("one load and one test, then a call of the kind's hand-over routine, per owned value whose layout the handing package cannot see (held `dyn I`, of an `open` or `abstract` class, a function value's state)"), which an `AnyError` handed to another thread pays for its boxed error exactly as any `owns dyn I` value does today, and `[ERR-11]`'s one allocation on the failure path, unchanged. The row "No data race | always, via `Send`, `ThreadShared` and the thread rules (Part XI) | never | type system; zero cost" stays true. No new §X.4 row.

**implementer_steps**: 1. Parser: nothing. `class_decl := ["open" | "abstract"] "class" IDENT ["(" type ")"] [implements] ...` with `implements := "implements" type {"," type}` already parses `implements Send, ThreadShared`.
2. Class declaration check (the package that declares the class): collect the family's promises = the markers named in this header's `implements` list that are `Send` or `ThreadShared` (resolved to `std.core`'s interfaces by identity, never by name, `[TYP-43]`) plus the base's promises (from the base's declaration in source, or from its interface file for an imported base). On a struct, enum or final class a listed marker is E2040 (remove it). For each promised marker, fold the marker property over every field, inherited fields included; the first field lacking it is E7015 at the class with the message of `[THR-8]` (the promise's line and package, a copy line, and `thread_shared` only where `[THR-1]`'s no-direct-change rule can then hold). A generic class checks with its parameters' written needs.
3. Marker inference `is_send(T)` / `is_thread_shared(T)`: a class type `C` that is final: fold over fields (as today). `C` `open` or `abstract` and subclassable outside its package (`pub`): the promise set holds the marker. `C` `open` or `abstract` and not subclassable outside its package: fold over the fields of every class the package derives from it (the same closed set `[EFF-2a]`/`[CLS-2]` use for its overrides). `dyn I`: `I`'s needs for both markers. Everything else unchanged. Cache per type as the other properties of `[TYP-1]` are.
4. Interface files (`[MOD-9]`): a `pub` class's header, promises included, is already exported as the written declaration; make sure a dependant reading the base from the interface file gets the promise set and the base's own-field markers (needed for a derived class that adds a promise its base lacks). Reload manifest (`[HR-8]`, `[HR-11]`): the schema hash of a class includes its promise set; plan refuses E9030 when an image drops a promise a non-rebuilt package or a live frame was checked against.
5. Interface implementations (`[IFC-3]`): at every `extend X implements I:` and `class X implements I:`, for each parent `P` of `I` (transitively), check `X: P` with the implementation's written bounds; unmet → E2040 at the implementation, naming the field (concrete) or giving the bounded `extend` line (generic). `@derive(…)` on a generic type generates the `extend` with the needs its fields require (`[DRV-1]`), then runs the same check.
6. `std.core`: `pub interface Error: needs (Debug, Display, Send, ThreadShared):` (core.em line 80); `AnyError` (core.em line 92) then infers both markers from `owns dyn Error` and its text chain. Compile std once under the new check: every std error type holds codes and text and passes; `pub open class AnyStorage` (std/src/ecs.em line 101) writes `implements Send, ThreadShared` if `World`s cross threads (ODR-218's `run_parallel`), else nothing.
7. Diagnostics: E7015 at the class (new message), E2040 at the implementation (new message), the existing E7015 for a promise-less open-class value reaching another thread with the promise line; all follow verdict 73 (the given line compiles: the copy line always does).
8. Tests (`[TST-4a]`, one accept and one reject per code): the two-package refused program above (E7015 at the subclass; E2040 at the library's `extend` without the promise); the accepted program (a worker returning `Result[int]`, a `Sender[Result[int]]`, a `ref` of a `Result[int]` in a scoped task, an `AnyError` under a `thread_shared` holder's lock); a private open hierarchy crossing threads with nothing written (accept) and the same with a one-thread subclass (E7015 naming the subclass); `implements Send` on a final class (E2040); a generic `extend[T] X[T] implements Error` (E2040 with the bounded line) and the bounded form (accept); a reload that adds a `shares` field to a subclass of a promised family (refused at compile) and one that drops a promise (E9030 in plan); the `[ERR-12]` path: a one-thread type as a `Result`'s error printed by `main` (accept).

**departs_from_verdict**: None. Verdicts touched and kept: 61 (a type says which threads may hold its values, chosen over a whole-program search: extended from function types to open classes, the header word); 94 (`thread.spawn` takes a `thread_shared fn()` and `R` MUST be `Send`: unchanged, now met by `AnyError`); 110 (parents written with `needs`: `interface Error: needs (Debug, Display, Send, ThreadShared):`); 108, 111, 113 (a `pub` generic and an `extend` write their needs: a generic implementation of `Error` writes `T: needs (Send, ThreadShared)`); 132 (a declaration means what it says: `extend[T] X[T] implements Error:` is refused, never quietly narrowed); 73 (the given fix compiles: a copy line is always offered); 86 and 122 (facts worked out from bodies: untouched, markers come from declarations); 164, 165, 166 (a `sharedref mut`, a non-`ThreadShared` part, a scoped task's counts: each error type holding one is refused where it implements `Error`; a derived class holding one is refused at its declaration); 23 (a `shares` value never used from two threads: none can sit inside an error, a derived class of a promised family, or a base-typed value that crosses); 114 (a handed object's links say gone: `[THR-2]`'s compare reaches the boxed error through its hand-over routine, already in the text); 106 (contracts on function types: untouched). Delegated rulings touched: ODR-255 (its `ThreadShared` half is now written in `[THR-9]`); ODR-111 (`Send` by hand only as `unsafe extend`: kept for the vouch; the class-header promise is a compiler-checked bound, not a vouch, and `[TYP-43]` says so); ODR-050 (a `Result`'s error parameter is unbounded: kept, a one-thread type stays a `Result` error printed by `main`). The owner gave no verdict on K20 itself (J6-16 was routed to him); this ruling is for him, with option A's cost stated: an error type holding a one-thread part is refused where it implements `Error`, even in a one-thread program, the fix being a copy.

**trade_offs**: Safety (never compromised): no program is accepted that breaks a promise. Every class of a family is checked in the package that declares it, from a word in a declaration that package sees, so separate compilation, a library update and a reload cannot slip a one-thread field under a `Send` judgement; the erased routes (`dyn Error`, `AnyError`, `Result[T]`, channels, scoped views, `thread_shared` holders) all rest on `Error`'s requirement, which every implementer met locally; the FFI's `threads(any)` callbacks and `threads=any` trampolines read the same markers. Against option (a) of the question (an open class's markers are its own fields, and every subclass must keep them, with no word): safe, but it would force `Send` on every subclass of every library base whose fields happen to be plain (`Script`, `Node`, `Widget`), refusing the ordinary one-thread game that gives `Enemy(Script)` a `shares Level` or a `weakref Player`; the author of the base never chose that. Against option (b) (a base-typed value is never `Send` unless the class is final, Swift's rule for `Sendable` classes): safe, but no open hierarchy could ever cross threads or be read by scoped tasks (std's own `AnyStorage` family included), and the owner's condition forbids dropping a feature to get past a check. The chosen rule is (b)'s default with (a)'s discipline opted into by one word on the class header, so a one-thread subclass is refused only where the base's author promised threads, and an open class private to its package needs no word at all. What is given up: a `pub open` class's objects cross threads only when its author writes `implements Send` (and `ThreadShared` for shared reads), one word on the thing it describes; a dependant cannot add a one-thread field to a class of a promised family (the message gives a copy); an error type holding a one-thread part, or an object of a promise-less open class, is not an `Error` (it is `E2040` at its `implements Error`, even in a program with one thread), though it stays a `Result` error whose `Display` `main` prints; `Error` requiring `ThreadShared` as well as `Send` refuses an error holding a `Cell` or a `ref mut`, rare, so that `ref AnyError` reaches a scoped task and an `AnyError` sits under a `thread_shared` holder's lock, as Rust's `anyhow` requires `Send + Sync`. Speed: nothing runs; every check is at a declaration; `AnyError` keeps its layout and its one allocation on the failure path. Readability: the worker loses an import, a concrete error type and an error enum; ordinary code writes nothing; the word appears only where threads hold a family, and reads as what it means. Consistency: the same shape as `interface I: needs Send` for `dyn I`, `[OWN-13]`'s pinning of a family by a method, `[OBJ-1]`'s family-wide weak count decided by the root's package, `[EFF-2a]`'s "a class that cannot be subclassed outside its package", `[MOD-9]`'s "where a word exists, only the written word counts across the edge", and `[BLD-16]`. Research used as input: Swift makes a `Sendable` class `final` because a subclass may add non-`Sendable` state; Ember can allow a non-final family because every subclass is compiled by Ember with the check; Rust's `anyhow::Error` requires `Send + Sync`, and `Box<dyn Error>` carries the markers only when written after `dyn`; C++'s `std::exception_ptr` crosses threads with no check at all.

**remaining_doubt**: 1. Syntax: the promise is written as `implements Send, ThreadShared` because `class_decl` already parses it and it reads as what it means for every object of the family; the owner may prefer an interface-style `needs` clause on the class header (verdict 110's spelling), which would need a grammar change. 2. Whether `Error` should require `ThreadShared` as well as `Send`: `Send` alone serves `thread.spawn`, channels and joins; `ThreadShared` is needed only for `ref AnyError` in a scoped task and an `AnyError` under a `thread_shared` holder's lock; both chosen, as the method-4 team and Rust's `anyhow` do, at the cost of refusing an error holding a `Cell` as an `Error`. 3. std must be compiled once under the new check: every std error type is expected to pass (codes and text), and `pub open class AnyStorage` (std/src/ecs.em line 101) needs the promise if a `World` is meant to cross threads (ODR-218); that is a std edit for the hardening, not a spec gap, but it was not verified here (read only). 4. The reload sentence is placed in `[HR-11]` (the schema) rather than `[HR-44]` (worked-out facts), since the promise is a written word; the lead may prefer it in `[HR-44]`. 5. `thread_shared Script()` for a promise-less `pub open` class is stated as `E7015` at the making line in `[THR-8]`'s edit, reading `[THR-1]`'s "everything reachable from it is `Send`" to include the value itself; without that sentence an implementer could read `[THR-1]` as fields only and let a `thread_shared Script` exist that can never leave its thread, harmless but confusing. 6. For an `open` class private to its package, the markers are read from every derived class the package declares, a package-wide fact `[BLD-16]` allows and `[EFF-2a]` already uses for overrides; a lead who wants one rule only can drop that sentence and require the word on every `open` class at the cost of one word in such programs. 7. The library-side refusal (E2040 at `extend LoadError implements Error:` without the promise) means a library update that drops a promise is an interface change a dependant's compile reports with the chain (`[DIA-27]`); the spec says this generally for interface changes, and no words were added for it here.

## Design 3

**question**: K20 (J6-16): Can AnyError, Result, a channel or a scoped task carry an error type across threads when that error holds an object of an open class another package may derive from? In particular: how are Send and ThreadShared judged for an open class and for a value holding one, so that a dependant's subclass cannot break a library's judgement without any whole-program fact ([BLD-16]); and what are Error's requirements and AnyError's markers?

**summary**: The rule a newcomer holds: an error is plain data any thread may hold (`interface Error: needs (Debug, Display, Send, ThreadShared)`), so `AnyError` and every `dyn Error` are `Send` and `ThreadShared` and a worker returns `Result[int]` with nothing written; and a base class whose objects may travel between threads says so in its header (`pub open class Script implements Send, ThreadShared:`), which every subclass is then held to where it is declared. An object held by a `pub` `open` or `abstract` class type is otherwise neither `Send` nor `ThreadShared`, because the package judging it never sees the classes a dependant derives ([BLD-16]); an `open` class no other package can subclass is judged from its own package's derived classes with nothing written, as [EFF-2a] already does for virtual calls; and a `thread_shared T` of an open `T` is `Send` and `ThreadShared` because its making line checked the object's own class (verdict 165). The method-4 hole closes at the library's own line (`extend LoadError implements Error:` is `E2040` while `Script` promises nothing) and, once the base promises, at the game's own line (`class Enemy(Script):` holding `shares Level` is `E7015`), never at a spawn far away.

- EDIT rule=§IV.8 standard interfaces sketch (CURRENT-SPEC.md line 1958, the `std.core` declaration ERR-3 and TYP-17 cite)
  - old: interface Error: needs (Debug, Display):
  - new: interface Error: needs (Debug, Display, Send, ThreadShared):
  - why: The cause: every error is erased through `Error` (`dyn Error`, the box in `AnyError`, the default error of every `Result[T]`), and `dyn I` is `Send` only when `I` requires it ([THR-8]: "`dyn I` when `I` requires it (`interface I: needs Send:`)"). `Error` required only `Debug` and `Display`, so by the rules every erased error was a one-thread value and `Result[int]` could not leave its thread, which no rule said. An error is a report: data any thread may hold. Swift's `Error: Sendable`, Rust's `anyhow` (`Send + Sync`). std/src/core.em line 80 follows.

- EDIT rule=TYP-17, the example of an interface's parents (line 1742)
  - old: (`interface Error: needs (Debug, Display):`), an associated type
  - new: (`interface Error: needs (Debug, Display, Send, ThreadShared):`), an associated type
  - why: The third copy of the declaration; left as it was, the generics chapter would decide a program differently from §IV.8 (consistency: one defining text).

- EDIT rule=ERR-3 (lines 7322-7324; the old text is wrapped in the file as shown)
  - old: `Error` is the
  interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default
  `None`) (§IV.8).
  - new: `Error` is the
  interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with `fn
  source() -> Option[ref dyn Error]` (default `None`) (§IV.8): an error is a value any thread may hold
  and read, so `dyn Error`, and `AnyError` with it, are `Send` and `ThreadShared` (`[THR-8]`,
  `[THR-9]`, `[ERR-8]`), and a `Result` whose error is one crosses to a thread, a channel or a scoped
  task as any `Send` value does (`[THR-10]`, `[THR-15]`, `[THR-2]`). A type that holds a value which
  stays on one thread (a `shares` or `shares mut` owner, a `weakref T` link, a `Cell`, an object held
  by a `pub` `open` or `abstract` class type whose class promises neither marker, `[THR-8]`) is not an
  `Error`: `E2040` where it implements `Error`, naming the field and giving the line to write ("`LoadError`
  implements `Error`, so any thread may hold and read it (`interface Error: needs (Debug, Display, Send,
  ThreadShared)`), and `LoadError.script` is a `Script`, an `open` class another package may derive
  from with fields this package never sees; hold what the error reports, `entity: int`, set from
  `script.entity`, or promise it for every class derived from `Script`: write `pub open class Script
  implements Send, ThreadShared:` (line 3)"), and `E2080` at `@derive(Error)`, naming the field
  (`[DRV-1]`). Such a type may still be a `Result`'s error (`[ERR-1]`), printed by `Display`
  (`[ERR-12]`); it does not pass through `?` into `AnyError`.
  - why: ERR-3 describes the declaration of §IV.8 and must say the same thing; it states the one property (any thread may hold an error) and places the refusal where the error type is declared, with the two lines that compile (a copy of what the error reports, or the base's promise). The spelling with `needs` is verdict 110's; `Display + Debug` was the pre-110 spelling.

- EDIT rule=ERR-8 (append after "`return Err(AnyError.msg(f\"no such file: {p}\"))`.", line 7340)
  - old: 
  - new:  `AnyError` is `Send` and `ThreadShared`: its box is an `owns dyn Error`, which is both because
  `Error` requires both (`[ERR-3]`, `[THR-8]`, `[THR-9]`), and its context chain is text. So a worker
  returns `Result[int]` through `thread.spawn` or a channel, and a scoped task reads one through
  `ref`, with nothing written; the thread that holds an `AnyError` last ends it, and a hand-over of one
  walks the boxed error by its kind's hand-over routine, as any value held `dyn I` (`[THR-2]`).
  - why: The question asked for AnyError's markers in so many words; ERR-8 is where AnyError is defined. It states a consequence of ERR-3 and THR-8, adds no rule of its own, and names the one run-time step that already exists (THR-2's routine for a `dyn` value).

- EDIT rule=THR-8, the class-object clause (lines 6338-6339; wrapped in the file as shown)
  - old: Structs, enums, tuples, fixed arrays, collections and class objects
  are `Send` when every component is; an `owns T` when `T` is;
  - new: Structs, enums, tuples, fixed arrays and collections
  are `Send` when every component is; a class object by its class, since an object held by a base class
  type may be of any class derived from it (`[TYP-5]` rule 8): an object of a class that is neither
  `open` nor `abstract` when the class's fields are; an object of an `open` or `abstract` class that
  another package may subclass (the class is `pub`) only when the class promises it, in its header or in
  an `extend` of its own package (`[TYP-20]`): `Send` itself, an interface that needs it (`pub open
  class Script implements Send, ThreadShared:`, `open class GameError implements Error:`), or
  `threads=any` on a trampoline class (`[FFI-17d]`); the promise holds the class's own fields and methods
  (`[OWN-13]`) and those of every class derived from it, each checked where it is declared, by the
  package that declares it, from the base's interface file (`[MOD-9]`, `[BLD-14]` item 8), so a
  dependant's class that breaks it is refused at its own line (`E7015`: "`Enemy` derives from `Script`,
  whose objects may go to any thread and be read by several at once (`pub open class Script implements
  Send, ThreadShared:`, gamekit/src/lib.em line 3), and `Enemy.level` is a `shares Level`, an owner
  counted on one thread; share the level between threads, `level: thread_shared Level`, made
  `thread_shared Level()`, or hold a copy, `level: Level`") and a base that promises nothing makes a
  value held by its type one no thread may be handed, whatever classes the program derives, which no
  package can see (`[BLD-16]`); an object of an `open` or `abstract` class no other package can subclass
  (`[EFF-2a]`) when its fields and those of every class its package derives from it are, with nothing
  written; an `owns T` when `T` is;
  - why: This is where the markers of a class object are stated, and "when every component is" read only the declared class's fields, narrower than THR-8's own definition ("nothing reachable from it"): the base-class hole exists for any open-class value handed to a thread, error or not. The fix is the dyn clause's own shape applied to the other erased holder: as `dyn I` is `Send` only when `I` requires it, a base-class value is `Send` only when the base promises it, and the promise is checked at every derived class's declaration from a declaration it sees. No whole-program fact enters a package's code ([BLD-16]); the in-package inference for a class no other package can subclass is the shape [EFF-2a] already uses for virtual calls. Verdict 61's shape: the type says which threads may have it, over a search of the program.

- EDIT rule=THR-8, the `thread_shared T` clause (lines 6340-6341)
  - old: a `thread_shared T`, and a
  `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`;
  - new: a `thread_shared T`, and a
  `weakref thread_shared T`, when `T` is `Send` and `ThreadShared`, and for an `open` or `abstract` `T`
  whatever its class promises, since the line that makes a `thread_shared` object checks the object's
  own class (`[THR-1]`, verdict 165) and a `thread_shared` owner is made nowhere else (`E7017`);
  - why: Without it the new class clause would make `thread_shared Script` not `Send` for a `pub open class Script` that promises nothing, though every `thread_shared Enemy()` was checked at its making line; this gives a library a second route for a polymorphic object that must cross threads (`script: thread_shared Script`) and decides a question an implementer would otherwise guess.

- EDIT rule=THR-8, "`Send` is inferred from the words" (lines 6344-6345)
  - old: `Send` is inferred from the words and written only
  in bounds;
  - new: `Send` is inferred from the words and written only
  in bounds and in the header of an `open` or `abstract` class, where it is a promise about every class
  derived from it that the compiler checks (above);
  - why: The sentence would otherwise contradict the clause added above.

- EDIT rule=THR-9, structural types (line 6361)
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types are `ThreadShared` when every component is; a class object as `[THR-8]` judges
  `Send`: by its fields when its class is neither `open` nor `abstract`, by the class's promise (`pub
  open class Script implements Send, ThreadShared:`, an interface that needs it) when another package
  may subclass it, each derived class checked where it is declared (`E7015`), and by its package's
  derived classes when none can; `dyn I` when `I` requires it (`interface I: needs ThreadShared:`), as
  for `Send` (ODR-255), so `dyn Error` is; a `thread_shared T` and a `weakref thread_shared T` of an
  `open` or `abstract` `T` whatever its class promises, as `[THR-8]` says.
  - why: THR-9 had no words for a class object and none for `dyn`, so `ref r` of a `Result[int]` given to a scoped task (`ref T` is `Send` when `T` is `ThreadShared`, THR-8) and an `AnyError` under a `thread_shared` holder's `RwLock` (verdict 165) were undecided; ODR-255 ruled the dyn half and the text never wrote it.

- EDIT rule=TYP-43, the written-by-hand sentence (lines 1868-1871)
  - old: A program writes `Send` by
  hand, for a struct or enum of its own package that holds a raw pointer, only as `unsafe extend T
  implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler
  cannot check
  - new: A program writes `Send` or `ThreadShared` by hand in one checked place, the header of an `open` or
  `abstract` class of its own package (`pub open class Script implements Send, ThreadShared:`, or
  `extend Script implements Send:` in that package), a promise about every class derived from it that
  the compiler checks at each derived class's declaration (`[THR-8]`, `[THR-9]`), so it needs no
  `unsafe`; on any other type `ThreadShared` by hand is `E2040` and `Send` by hand, for a struct or enum
  of its own package that holds a raw pointer, is written only as `unsafe extend T
  implements Send:` with no members and a `# SAFETY:` note, taking the responsibility the compiler
  cannot check
  - why: TYP-43 refused every hand-written marker but the unsafe vouch; the class header's promise is checked, not vouched, so it is carved out here, where the refusal is stated. The grammar already allows it (`class_decl := ["open" | "abstract"] "class" IDENT ["(" type ")"] [implements] ...`); no new syntax.

- EDIT rule=§IV.8 marker table, `Send` row (line 1859)
  - old: a class object is when its fields are;
  - new: a class object is when its fields are, and one held by a `pub` `open` or `abstract` class type only when the class promises `Send` (in its header, or by an interface that needs it), which every class derived from it is checked against where it is declared (`[THR-8]`); `dyn I` when `I` requires it;
  - why: The table answers the markers (TYP-43: "the table answers them"); its row was the narrow reading that let a base-class value through, and it must give THR-8's answer.

- EDIT rule=§IV.8 marker table, `ThreadShared` row (line 1860)
  - old: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`);
  - new: every component is `ThreadShared` (the `ref A` a value made in a manager's instance holds included, `[TYP-46a]`), a class object judged as for `Send`, by its class's promise where another package may subclass it (`[THR-9]`); `dyn I` when `I` requires it;
  - why: Same as the `Send` row: the table and THR-9 must agree.

- EDIT rule=IFC-3 (append after "(ODR-042).", line 2692)
  - old: 
  - new:  An implementation meets its interface's parents for every type it covers: a generic one through the
  bounds it writes (`extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:`), never
  quietly for some arguments only (verdict 132); one whose bounds do not show it is `E2040`, whose
  message gives the bounded line, as `[IFC-6]` gives a `pub` function's ("`PoolFull[T]` holds
  `rejected: T`, and an `Error` is `Send` and `ThreadShared`; write `extend[T: needs (Send,
  ThreadShared)] PoolFull[T] implements Error:`"). A class that implements an interface with marker
  parents promises them for every class derived from it (`[THR-8]`).
  - why: With marker parents on `Error`, a generic error that hands the caller's value back (`SendError[T]`, a pool's `PoolFull[T]`) must be decided: the implementation is a library's promise and writes its needs (verdicts 108, 111), and the compiler gives the line (verdict 113's manner). Stated once for every interface with parents, not as a case of `Error`.

- EDIT rule=DRV-1 (after "A derive whose requirement a field does not meet is `E2080`, naming the field.", line 7538)
  - old: A derive whose requirement a field does not meet is `E2080`, naming the field.
  - new: A derive whose requirement a field does not meet is `E2080`, naming the field. On a generic type, a
  requirement a field meets only for some arguments of a type parameter is written as that parameter's
  needs in the generated `extend` (`@derive(Error)` above `[T]` and `enum SpawnError:` with
  `Rejected(T)` generates `extend[T: needs (Debug, Send, ThreadShared)] SpawnError[T] implements
  Error:`), as `@derive(Copy)` above `[T] pub struct Range` is `Copy` when `T` is (ODR-027,
  `[STR-5]`); `E2080` stays for a field whose written type fails.
  - why: Once `Error` has marker parents, `@derive(Error)` on a generic error type must be decided; every other derive already works per instance (std's `Range`), and the generated code is what `ember inspect --expand` prints, so the bound is written where the compiler writes the implementation.

- EDIT rule=BLD-14 item 8 (lines 9226-9228)
  - old: neither, which a line that makes an object of the class checks (`[OWN-13]`); and
  - new: neither, which a line that makes an object of the class checks (`[OWN-13]`), and, for an `open` or
     `abstract` class, the markers its header or an `extend` of its package promises (`Send`,
     `ThreadShared`, an interface that needs them), which each class derived from it in another
     package is checked against where it is declared (`[THR-8]`, `E7015`); and
  - why: The promise is a written word of a `pub` class, so it crosses the edge in the interface file (MOD-9: "what the package's `pub` items write"); naming it in the list keeps BLD-14 the complete record of what a dependant reads.

- EDIT rule=Annex B §B.3 schema table, after the row "| base class changed | refused |" (line 10870)
  - old: | base class changed | refused |
  - new: | base class changed | refused |
  | `Send` or `ThreadShared` promised by an `open` or `abstract` class added or removed (`[THR-8]`) | refused unless the image rebuilds every package that derives from the class, whose derived classes are then checked again (`[HR-44]`) |
  - why: The unit of reload is the package (B.1). A reload of the base's package that adds the promise would let its new code hand `Script`s to threads while a derived class in a package the reload does not rebuild was never checked against it; HR-44 compares functions' facts, not a class's written promise, so the row says it. A derived class rebuilt in the image is checked again when it compiles (HR-44's own words).

- EDIT rule=Code table, row `E7015` (line 10004)
  - old: a `thread_shared` object of a class pinned to one thread or with a `weakref T` field;
  - new: a `thread_shared` object of a class pinned to one thread or with a `weakref T` field; a class derived from an `open` or `abstract` class that promises `Send` or `ThreadShared` (in its header, by an interface that needs them, or by `threads=any`), whose own fields or methods are not, at the derived class;
  - why: The code table lists every refusal a rule states (one code, one defining rule: the row names [THR-8]).

- EDIT rule=Code table, row `E3100` (line 9940)
  - old: `Send` declared by hand without `unsafe extend`;
  - new: `Send` declared by hand without `unsafe extend`, other than in the header or an `extend` of an `open` or `abstract` class, where it is a checked promise (`[THR-8]`);
  - why: The row would otherwise refuse the promise TYP-43 now allows.

- EDIT rule=Code table, row `E2040` (line 9848)
  - old: `unwrap` or `expect` on a `Result` whose error type has no `Display`
  - new: `unwrap` or `expect` on a `Result` whose error type has no `Display`; an implementation that does not meet its interface's parents for every type it covers (`extend[T] PoolFull[T] implements Error:`, given the bounded line; a type holding a one-thread value implementing `Error`, `[ERR-3]`)
  - why: The refusal IFC-3 and ERR-3 now state needs its row; add `[IFC-3]`, `[ERR-3]` to the row's rule column.

**refused_program**: # file gamekit/ember.toml
#   [package]
#   name = "gamekit"
#   version = "0.1.0"
#   language = "0.9.20"
#   kind = "lib"
# file gamekit/src/lib.em
from std.fmt import Formatter, FmtError

pub open class Script implements Send, ThreadShared:   # line 3: every Script may go to any thread
    pub entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct LoadError:
    pub script: Script                                 # a Script, so a Send and ThreadShared value

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the script could not load")

extend LoadError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("LoadError")

extend LoadError implements Error:                     # accepted: LoadError is Send and ThreadShared
    pass

pub fn fail(owns s: Script) -> Result[int, LoadError]:
    return Err(LoadError(script=s))

# file game/ember.toml
#   [package]
#   name = "game"
#   version = "0.1.0"
#   language = "0.9.20"
#   [dependencies]
#   gamekit = { path = "../gamekit" }
# file game/src/main.em
import std.thread
from gamekit import Script, fail

class Level:
    name: String = String("forest")

class Enemy(Script):                                   # line 7
    level: shares Level                                # line 8

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn load(entity: int, owns level: shares Level) -> Result[int]:
    return Ok(fail(owns Enemy(entity, owns level))?)   # LoadError -> AnyError

fn main():
    level = shares Level()
    r = load(1, shares level)
    h = thread.spawn(owns fn() owns r: r.is_err())
    for i in 0..100000:
        other = shares level
        println(i, other.name.len())
    println(h.join())

# Refused in game's own build, at its own line, from gamekit's interface file:
# error[E7015]: game/src/main.em:8: `Enemy` derives from `Script`, whose objects may go to any thread and
#   be read by several at once (`pub open class Script implements Send, ThreadShared:`, gamekit/src/lib.em
#   line 3), and `Enemy.level` is a `shares Level`, an owner counted on one thread; share the level between
#   threads, `level: thread_shared Level`, made `thread_shared Level()`, or hold a copy, `level: Level`
#
# Had gamekit left line 3 as `pub open class Script:`, gamekit's own build would have refused line 20 instead:
# error[E2040]: gamekit/src/lib.em:20: `LoadError` implements `Error`, so any thread may hold and read it
#   (`interface Error: needs (Debug, Display, Send, ThreadShared)`), and `LoadError.script` is a `Script`, an
#   `open` class another package may derive from with fields this package never sees; hold what the error
#   reports, `entity: int`, set from `script.entity`, or promise it for every class derived from `Script`:
#   write `pub open class Script implements Send, ThreadShared:` (line 3)
# Either way no line of either package can put a one-thread owner into an AnyError that crosses a thread.

**accepted_program**: # file gamekit/ember.toml
#   [package]
#   name = "gamekit"
#   version = "0.1.0"
#   language = "0.9.20"
#   kind = "lib"
# file gamekit/src/lib.em
from std.fmt import Formatter, FmtError

pub open class Script implements Send, ThreadShared:   # a promise every derived class is checked against
    pub entity: int

    pub fn init(entity: int):
        this.entity = entity

    pub virtual fn on_update(dt: f32):
        pass

pub struct LoadError:
    pub script: Script

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"script {this.script.entity} could not load")

extend LoadError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("LoadError")

extend LoadError implements Error:                     # Send and ThreadShared: Script promises both
    pass

pub fn attach(owns s: Script, ok: bool) -> Result[int, LoadError]:
    if ok:
        return Ok(s.entity)
    return Err(LoadError(script=s))

# file game/ember.toml
#   [package]
#   name = "game"
#   version = "0.1.0"
#   language = "0.9.20"
#   [dependencies]
#   gamekit = { path = "../gamekit" }
# file game/src/main.em
import std.fs
import std.thread
from gamekit import Script, attach

class Door(Script):                                    # plain fields: meets Script's promise, nothing written
    angle: f32 = 0.0

    override fn on_update(dt: f32):
        this.angle = min(this.angle + 90.0 * dt, 90.0)

fn load(path: StrView) -> Result[int]:                 # fails with any error: AnyError
    text = fs.read_to_string(path)?                    # io.Error -> AnyError
    n = attach(owns Door(7), text.len() > 0)?          # LoadError (holding a Door as a Script) -> AnyError
    return Ok(text.len() + n)

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => load("level1.cfg"))  # R = Result[int, AnyError], Send: accepted
    n = h.join()?                                      # the error, if any, ends on this thread
    println(n)
    return Ok(())

# Prints the file's length plus 7, or `error: script 7 could not load` (status 1), or the system's message
# for a missing file. Nothing is written about threads in game; the only word is gamekit's promise.

**runtime_cost**: Nothing new. Every judgement here (an open class's promise, each derived class against it, `Error`'s parents, `dyn Error`'s markers) is made when compiling. What runs is already in §X.4: the row "`?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`)", unchanged; and the row "An object handed whole to another thread", whose words "one load and one test, then a call of the kind's hand-over routine, per owned value whose layout the handing package cannot see (held `dyn I`, of an `open` or `abstract` class, ...)" already price the `Err` path of a handed `Result[int]` (its `owns dyn Error`) and a handed base-class object, with "nothing for a value whose type can own no such object". A `@cached` method of a class that implements `Error` falls under the existing clause of the `@cached` row, "on an object several threads reach, the result is filled once under a once-flag", since such objects may now be read by several threads; that clause existed for every `ThreadShared` class. No new row ([COST-5]).

**implementer_steps**: 1. std.core: change `pub interface Error: needs (Debug, Display):` (std/src/core.em line 80) to `needs (Debug, Display, Send, ThreadShared)`. Check std's own error types against it: `io.Error`, `ParseError` (std/src/string.em 25), `CppError` (std/src/ffi.em 149), the two `@derive(Error)` enums in core.em (222, 228) hold plain data; `SendError[T]` (std/src/sync.em 122) gets its `Error` implementation under `[T: needs (Send, ThreadShared)]`. `AnyError`'s markers then follow by inference from `owns dyn Error` plus its text chain; write nothing on it.
2. The marker judge (the function that decides `Send`/`ThreadShared` for a type) gains the class case: (a) class neither `open` nor `abstract`: its fields, plus the one-thread pin from its methods ([OWN-13], [BLD-14] item 8) which makes it neither; (b) `open`/`abstract` and `pub` (another package may subclass it): read the class family's promises, collected along the base chain from each class's header `implements` list and the `extend ... implements` blocks of its own package (coherence, [TYP-20]), an interface's `needs` expanded transitively (`Error` gives `Send` and `ThreadShared`), and `threads=any` on a trampoline class; the marker holds iff promised; (c) `open`/`abstract` and not `pub`: the fields and pins of the class and of every class in this package deriving from it, transitively (the same walk [EFF-2a] makes for a non-`pub` class's virtual calls). (d) `thread_shared T`, `weakref thread_shared T` with `T` in case (b) or (c): `Send` and `ThreadShared` unconditionally. (e) `dyn I`: `ThreadShared` iff `I` requires it, beside the existing `Send` test.
3. At every class declaration `class D(B):`, after fields are resolved: compute B's family promises as in 2(b) (B's own package by its source, another package by its interface file); for each promised marker, judge D's own fields as a struct of those fields (D's own methods' pin included) and report `E7015` at the field (or the pinning method line), with the message naming the base's promise line, the field and the two lines (a `thread_shared` field, or a copy). Record D's resulting promises (the family's) in the interface file as part of [BLD-14] item 8.
4. At `implements I` (header or `extend`), for each marker parent of `I` (transitively), judge the implementing type with the extend's bounds in force (a type parameter has exactly the markers its `needs` list gives); failure is `E2040` with the bounded `extend` line (collect the needed markers per parameter that reaches a field). For `@derive(Error)` (and every derive) on a generic type, generate the `extend` with per-parameter needs collected the same way; the expanded text is what `ember inspect --expand` prints.
5. Interface file: a `pub open`/`abstract` class's header `implements` list and its package's `extend ... implements Send/ThreadShared` blocks are already written words and already exported; make sure the exporter does not drop a memberless `extend`, and that the reader resolves `implements Error` to its marker parents through the interface's recorded `needs`.
6. Hot reload: add the family's promises to an `open`/`abstract` class's schema ([HR-11]'s "kind, base, layout attributes"); Plan compares old and new and refuses (`E9030`) a change unless every package in the loaded set that derives from the class is in the image being reloaded (those are recompiled and so re-run step 3).
7. Diagnostics: add the `E7015`, `E2040` and `E3100` row texts; `Send`/`ThreadShared` in a class header on a class that is neither `open` nor `abstract`, or on a struct or enum, keeps today's refusals (`E3100`, `E2040`).
8. Nothing changes in codegen: no new TypeInfo field, no new header bit, no new runtime call; THR-2's hand-over routine for a `dyn` or open-class value already exists.
9. Tests: the refused program above (`E7015` at game, `E2040` at gamekit without the promise), the accepted program (prints the length), `extend[T] PoolFull[T] implements Error:` (`E2040` with the bounded line), a non-`pub` open class with a `shares`-holding subclass handed to `thread.spawn` (`E7015` naming the subclass's field, nothing written), `thread_shared Script` of a promising-nothing `pub open class Script` handed to a thread (accepted), and a reload that adds the promise without rebuilding the deriving package (`E9030`).

**departs_from_verdict**: None of the owner's. Verdict 61 (the type says which threads may run or have it, over a whole-program search): kept and extended, the class header says it. Verdict 94 (`thread.spawn` takes `thread_shared once fn() -> R`, `R` `Send`): kept, `spawn`'s line is unchanged and `Result[int]` now meets it. Verdict 110 (parents with `needs`): kept, the three copies of `Error`'s declaration use it. Verdicts 108, 111, 113 (a `pub` generic and an `extend` write their needs; the compiler gives the whole line): kept, the generic error implementation writes `[T: needs (Send, ThreadShared)]` and the message gives it. Verdict 132 (a declaration means what it says): kept, no implementation quietly holds for some arguments. Verdicts 164, 165, 166 and 23: kept; 165 is relied on for `thread_shared T` of an open `T`. Verdict 114 (a handed object's links say gone): kept, THR-2's routine already walks a `dyn` value. Verdicts 86 and 122 (what a call changes is worked out from every function the program can put there): untouched; a marker is a property of a type, decided per BLD-16 (lead ruling N76), not a call's effect. ODR-050 (a `Result`'s error is unbounded) and ODR-255 (`dyn I` is Send/Sync when `I` requires it) kept, the latter's `ThreadShared` half now written. The ruling departs from two non-verdicts: the method-4 team's decision 3 option (i) (count every derived class the program declares), which [BLD-16] now forbids and whose hole S2-1/E2-1 blocked that design, and the gap sweep's recommended option A (check at the `?`), which puts three marker words on every generic function that uses `?` into `AnyError`.

**trade_offs**: Given up: (1) An error type holding a one-thread value (a `shares` owner, a `Cell`, a `weakref T` link, an undeclared base-class object) is not an `Error`, even in a program with one thread; it remains a `Result`'s error printed by `Display`, but does not pass through `?` into `AnyError` or stand behind `dyn Error`. The fix is to hold a copy of what the error reports, which is what an error is. Swift refuses the same type at its declaration; Rust's `anyhow` refuses it at the `?`. (2) A library whose error keeps the failing base-class object must either keep a copy of what it reports or make the base promise `Send, ThreadShared`, and that promise forbids every subclass in every dependant a `shares` field, a `Cell`, a `weakref T` link or `weakref this`; an engine that wants one-thread scripts keeps plain data in its errors. (3) An object held by a `pub` `open` or `abstract` class type that promises nothing is handed to no thread, read by no scoped task through `ref`, and sent through no channel, even in a program whose every subclass is plain: the judging package cannot see the subclasses, and the readable alternative is one written promise rather than a fact of the whole program ([BLD-16]). `thread_shared Script` is the second route, checked at each making line. (4) `SendError[T]` is an `Error` only for a `ThreadShared` `T`, so `tx.send(owns cell)?` into `AnyError` on a channel of `Cell[int]` is refused (Rust's `anyhow` has the same edge). (5) A reload that changes a base's promise must rebuild every package deriving from it. (6) One more thing a reader learns: `pub open` classes are judged by their promise, other classes by their fields; the same split [EFF-2a] already draws for virtual calls, so it is one rule twice, not two rules. Against the four goals: speed, nothing new runs and no header grows (the method-4 design's two stated costs, the once-flag and the hand-over routine, already exist in the text for every `ThreadShared` class and every `dyn` value); readability, the ordinary worker, server and library program is shorter than today (no concrete error type, no enum for two failures) and the one written word is on the thing it describes, the base class; safety, no one-thread owner can enter an `AnyError`, a `Result` error, a channel or a scoped task's `ref` through any class declared anywhere, each refusal falling on the class that holds the one-thread part, in its own package, with a line to write; consistency, the class clause now says what THR-8's definition ("nothing reachable from it") always said, the marker table and THR-9 agree with it, and the dyn clause and the class clause are one shape (the erased holder is judged by the declaration that erases it).

**remaining_doubt**: (1) The clause "a `thread_shared T` of an open `T` is `Send` and `ThreadShared` whatever its class promises" rests on every route to a `thread_shared` object passing [THR-1]'s making-line check on the concrete class: `thread_shared x` on a value not made so is `E7017`, upcasts keep the word ([TYP-5] rule 8), and C never makes an Ember object; the lead should confirm hot-reload migration ([HR-15]) and a `retained` object given to C ([FFI-35a]) add no route. If any does, the clause goes and `thread_shared Script` is judged by the promise like a plain `Script`, at the cost of the second route. (2) The in-package inference for an `open` class no other package can subclass lets an added subclass with a `shares` field break a `thread.spawn` elsewhere in the same package, the message naming the subclass's field; verdict 86 accepted that shape inside a package for effects, and the alternative (a promise on every open class) writes a word where the package can see everything. The lead may prefer the promise everywhere for one rule with no split. (3) Whether `Send`/`ThreadShared` written in the header of a class that is neither `open` nor `abstract` should be an error (today's E3100/E2040, kept here) or a harmless redundancy; kept refused for consistency with "written only in bounds". (4) The E2040 placement for a non-generic type that implements `Error` and is not `Send` (at the `implements` line) against FFI-17d's E7015 at the class for `threads=any`: two codes for "a type does not meet a marker it promised"; here E2040 is an unsatisfied interface parent and E7015 a one-thread value where another thread may reach it, as the table already divides them, but the lead may unify. (5) std's own error types were checked by name only (plain fields in `io.Error`, `ParseError`, `CppError`, core.em's two derives); the implementer confirms each compiles under the new parents, and `SendError[T]`'s `Error` implementation takes the bound. (6) The reload row is the smallest statement; whether the promise belongs in [HR-11]'s schema sentence as well is the lead's placement.

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
