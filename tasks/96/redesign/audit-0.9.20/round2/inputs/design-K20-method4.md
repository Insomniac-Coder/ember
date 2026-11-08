## K20-anyerror-send (P01; J6-16): **rejected**, 9 agents

Outcome: `decided_by_parameters`. Intent: Load a level file on a worker thread and give the main thread either its length or the reason it failed, using the language's default error type (`Result[int]`).

Where the cause is: Three places, one property each. (1) The definition of `Error`: IV.8's declaration (p04-types.md 676, `interface Error: needs (Debug, Display):`), which ERR-3 describes (p13-errors.md 87-88) and TYP-17 quotes (p04 459). (2) What an object of an `open` or `abstract` class holds: THR-8's list sentence (p11-concurrency.md 47-48, "class objects are `Send` when every component is") and the marker table (p04 577, "a class object is when its fields are") count only the fields of the object's declared class. That is narrower than THR-8's own definition (p11 43-45, "nothing reachable from it ... is a one-thread owner"), because an object of an open class may be an object of any class derived from it (TYP-5 rule 8, p04 86). THR-9 (p11 63-64) has the same gap, and it has no words for a `dyn` either. (3) IFC-3 (p05 411-413) says that implementers meet an interface's parents, but not that an implementation meets them for every type it covers, which takes in each instance of a generic implementation and each derived class. (reported at: ERR-3, ERR-8, THR-2, THR-8 (it shows through THR-10's "`R` MUST be `Send`", p11 175-176)). Why there: Every error is erased through `Error`: `dyn Error`, the box inside `AnyError` (ERR-8), and so the default error type of every `Result[T]` (ERR-9). THR-8's dyn clause is right as it stands (`dyn I` is Send only when `I` requires it). `Error` requires no thread markers, so every erased error is a one-thread value, and the default `Result[T]` cannot leave its thread by any route. ERR-8's `From`, THR-2's routes and THR-10 are each right as written. The missing words belong in what an error is. Once `Error` requires the markers, the requirement is only as sound as the markers of what an error holds. For a class, THR-8's list and the p04 row read the markers from the declared class alone. So a `GameError` that implements `Error` would be judged Send while one of its objects is a `DocError` holding a `shares` owner. That gap already exists for a plain open-class value handed to a thread. The change would carry it into every AnyError. So the class wording is part of the cause. It has to say what THR-8's definition already says. IFC-3 is where the language says what an implementation promises about its interface's parents, so the rule for generic implementations and derived classes goes there, once for every interface, and not in ERR-3 as a case of `Error`.

**In short: Making every error type safe for any thread (as Swift's errors are) lets the catch-all error (like C++'s `std::exception_ptr`) come back from a worker thread. But as written, an engine library's error that holds a `Base*` can still carry a game's `Derived` object, holding a non-atomic refcount, to another thread.**

step | who | what happens | result
1 | engine library | declares an error that keeps the failing `Script` object; checked alone it is thread-safe | allowed
2 | game (another package) | declares `Enemy`, a kind of `Script`, holding a one-thread shared owner | allowed: nothing checks it against the engine's error
3 | game | puts an `Enemy` into that error, then the error into the catch-all, and hands it to a thread | allowed, though today it is refused
4 | both threads | change the same one-thread count at once | race: freed memory read or freed twice

Today: A worker returning `Result[int]` is refused at `thread.spawn` without a word of explanation, so the race above cannot happen.

* Repair and keep D: a class derived later, in another package or by a reload, must keep the thread-safety the library relied on, and is refused at its own line with the fix
* D as written: unsafe across libraries, so not acceptable
* A: check each error where `?` puts it into the catch-all (the game's own build sees the Enemy, so it is safe), but generic error code and wrappers must write thread words
* B: keep today: the catch-all stays on its thread, and workers name their own error type

Recommendation: Repair and keep D. Ordinary worker code still compiles with nothing written, and the one-thread object is refused at the line of the game's class that holds it.

Arbiter: Not yet. The change answers J6-16 (direction D: every error is safe to hand to and read from any thread, so `Result[int]` comes back from a worker with nothing written). But it is blocked by one safety hole, reported twice (S2-1 and E2-1). Take a library error struct that holds an object of the library's own open class (an engine's `LoadError` holding the failing `Script`). It is judged thread-safe in the library's build, which cannot see the game's classes. A game in another package then derives `Enemy(Script)` holding a one-thread shared owner. The changed IFC-3 checks a derived class only when it derives from a class that implements Error, and Script implements nothing, so no line of the game is checked. AnyError is Send from Error's requirement alone (p11 51), so the owner crosses threads inside an AnyError and its plain count races (use after free or double free). Today the same program is refused at the spawn, so the change causes it. The repair is one property, not a case. Every thread fact a package worked out from one of its open classes and relied on becomes a fact of that package's interface (MOD-9, EFF-4). A class derived later, in another package or by a reload, must keep it, and is refused at its own line, with the message naming the line that relied on it and giving a copy line. The same property closes the generic-derived-class route, and to_verify item 6 (a library's own send of a base value) should move into rule text. Three objections do not block. S2-2: CLS-12 decides it, but the cost report scopes the once-flag too narrowly. E2-2: decision 3's short text hides that option (i) has the far-break property C was turned down for. E2-3: TYP-43's prose needs one clause. All three are real, and two must go to the owner.

### Edit: IV.8 (standard interfaces, the definition ERR-3 cites) (change, p04-types.md 676)

**Before:**

> interface Error: needs (Debug, Display):

**After:**

> interface Error: needs (Debug, Display, Send, ThreadShared):

Why here: This is the cause. `Error` is the interface that every error is erased through, and it was the only one of ERR-3, ERR-8 and THR-8 that said nothing about threads. std/src/core.em line 80 follows it.

Why general: IV.8 holds "the definitions" of the standard interfaces (p04 600). It is the declaration that every bound, every `dyn Error` and every implementer reads.

### Edit: ERR-3 (change, p13-errors.md 87-88)

**Before:**

> `Error` is the interface of error types:
>   `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8).

**After:**

> `Error` is the interface of error types, `interface Error: needs (Debug, Display, Send, ThreadShared):` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8). Any thread may have and read an error, so `dyn Error` and `AnyError` are `Send` and `ThreadShared` (`[THR-8]`, `[THR-9]`). A type that holds a value that stays on one thread is not an error, and is refused where it implements `Error` or where it is derived from a class that does (`E2040`, `[IFC-3]`; `E2080` at `@derive(Error)`, `[DRV-1]`). It may still be a `Result`'s error that its `Display` prints (`[ERR-1]`, `[ERR-12]`).

Why here: ERR-3 describes the declaration of p04 676. The two must say the same thing, or the two texts would decide one program differently.

Why general: It states one property of what an error is: any thread may have and read it. The general rules then decide the rest. IFC-3 says where an implementation that does not meet the parents is refused and what its message gives. THR-8 and THR-9 say what each part of a type holds. ERR-3 lists no cases and no fixes. ERR-1's "any `E: needs Error`", ERR-8's `From` and ERR-9's "any error" stay true as written. The parents are spelled with `needs`, as verdict 110 spells them, and the old `Display + Debug` spelling goes.

### Edit: TYP-17 (its example of an interface's parents) (change, p04-types.md 458-459)

**Before:**

> an interface's parents
>   (`interface Error: needs (Debug, Display):`)

**After:**

> an interface's parents
>   (`interface Error: needs (Debug, Display, Send, ThreadShared):`)

Why here: Left as it was, a reader of the generics chapter would find the old parents, and objection E1-4's `report_later` would compile under one text and be E2040 under the other.

Why general: It is the third copy of the declaration. It still serves as the example of several needs in round brackets.

### Edit: THR-8 (change, p11-concurrency.md 47-48)

**Before:**

> Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is;

**After:**

> Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is, and an object of an `open` or `abstract` class may be an object of any class the program derives from it, a library's included and each instance of a generic one, whose fields are then among its components (as a call through a `virtual` method counts every override the program has, `[EFF-24]`);

Why here: This is where the class markers are stated. Without it, the change's requirement on `Error` would be checked against the declared class alone, and objection S1-1's `DocError` (and an error struct holding a field of a plain open class whose derived object holds a `shares` owner) would cross threads inside an AnyError.

Why general: It makes THR-8's list say what its own definition says ("nothing reachable from it"). This holds for every class hierarchy, error or not: a game's `Script` with an `Enemy` holding a `weakref`, std's ECS `World` holding `Map[u64, AnyStorage]` (std/src/ecs.em 101, 201) whose `[T] AosStorage` holds the program's components, and an error's `GameError` with a `DocError`. Nothing is written: the markers are still inferred from the words of types (TYP-43), here the declarations of the derived classes. A line that needs the marker is refused with E7015, whose message names the derived class and its field (THR-2's "whose message gives the words to write").

### Edit: THR-9 (change, p11-concurrency.md 63-64)

**Before:**

> Structural types are `ThreadShared` when every component is.

**After:**

> Structural types and class objects are `ThreadShared` when every component is (a class object's components as `[THR-8]` counts them), and `dyn I` when `I` requires it (`interface I: needs (Send, ThreadShared):`), as for `Send` (`[THR-8]`, ODR-255).

Why here: Without it, `ref r` of a `Result[int]` given to a scoped task (a `ref T` is Send when `T` is ThreadShared, p11 49) and an AnyError inside a thread_shared object's `RwLock` (verdict 165) stay undecided after the ERR-3 change.

Why general: Every interface gets the ThreadShared half of THR-8's dyn clause, a program's own `interface Command: needs (Send, ThreadShared):` as well as `Error`. ODR-255 already ruled it, and the text never said it. Class objects are counted as for Send.

### Edit: Marker table, Send and ThreadShared rows (TYP-43) (change, p04-types.md 577-578)

**Before:**

> Send row: "a class object is when its fields are;" / ThreadShared row: "every component is `ThreadShared`;"

**After:**

> Send row: "a class object is when its fields are, and those of every class the program derives from its class (`[THR-8]`); `dyn I` when `I` requires it;" / ThreadShared row: "every component is `ThreadShared`, a class object's counted as for `Send`; `dyn I` when `I` requires it;"

Why here: The row "a class object is when its fields are" is the narrow reading that lets a base-class value holding a one-thread derived object through. Leaving it would be a contradiction with the edited THR-8.

Why general: TYP-43 says the table answers the markers (p04 583). It must give the same answer as THR-8 and THR-9.

### Edit: IFC-3 (add, p05-declarations.md 411-413 (after "(ODR-042)."))

**Before:**

> `interface Ord: needs Eq` requires implementers of `Ord` to implement `Eq`. So a bound brings its parents: `T: needs IndexMut[int]` provides `Index[int]`'s methods too, and a binding it writes (`Output = f32`) names the parent's associated type (ODR-042).

**After:**

> (unchanged, then:) An implementation meets its interface's parents for every type it covers: each instance of a generic implementation, and each class derived from a class it covers, whose objects are values of that class (`[TYP-5]` rule 8), each instance of a generic one included. One that does not is `E2040`, either at the generic implementation (`extend[T] Tagged[T] implements Ord:` over `extend[T: needs Eq] Tagged[T] implements Eq:` gives `extend[T: needs Eq] Tagged[T] implements Ord:`) or where the derived class is declared, which its own package checks, whichever package declares the base. The message names the part that does not meet a parent and gives a line that compiles. That line either makes the part meet the parent by the rule that decides it (a type parameter's needs, an interface's needs), or holds in its place a copy of what the type uses from it.

Why here: Decision 2 (a generic error implementation) and objection S1-1 (a derived error class) are both about what an implementation covers. Stating it in ERR-3 would make it a case of `Error` (objection I1-1), and `Command`, `Ord` and every later interface with marker parents would stay undecided.

Why general: It is one property of every implementation of every interface with parents: `Ord` over `Eq`, `IndexMut` over `Index`, a program's own `interface Command: needs (Send, ThreadShared):`, and `Error`. For a parent the base implements (Eq, Debug), a derived class meets it through the base's implementation, so nothing changes there. For a marker parent, which is inferred from contents, the check falls on the derived class's own fields, at the derived class's own line. A derived class may be in another package, but the check is still local. Rust also requires a supertrait to hold for every type an impl covers. The message rule is a property, not a list: it gives a needs line where one exists and a copy line otherwise. That is principle 12, and verdict 73's "how to fix an issue not how to run into another", so `thread_shared` is offered only where it compiles.

### Edit: DRV-1 (add, p14-comptime.md 138 (after "naming the field."))

**Before:**

> A derive whose requirement a field does not meet is `E2080`, naming the field.

**After:**

> (unchanged, then:) On a generic type, a requirement that a field meets only for some arguments of the type's parameters is written as their needs in the generated `extend`, so the derive implements the interface for those instances (`@derive(Copy)` above `[T]` and `pub struct Range:` is `Copy` when `T` is, ODR-027; `[STR-5]`).

Why here: With Error's new parents, `@derive(Error)` on a generic error type (`SpawnError[T]` holding `rejected: T`) must be decided. Read the other way it would be refused even when only `SpawnError[int]` is used, unlike every other derive (objection E1-3).

Why general: Every derive works this way, which is how STR-5 ("when every field implements the interface"), ODR-027 and std's own `@derive(Copy)` above `[T] pub struct Range` (std/src/core.em 1850-1853) already read it. E2080 stays for a field whose own written type fails. The generated code stays what `ember inspect --expand` prints.

### Edit: Code table, E2080 row (change, p17a-codes.md 71)

**Before:**

> | `E2080` | `@derive(Copy)` on a type with a field that is not Copy | `[STR-3]` |

**After:**

> | `E2080` | a derive whose requirement a field does not meet, naming the field (`@derive(Copy)` with a field that is not `Copy`; `@derive(Error)` on a type holding a value that stays on one thread) | `[STR-3]`, `[DRV-1]`, `[ERR-3]` |

Why here: ERR-3 now names E2080 for `@derive(Error)`.

Why general: The row says what DRV-1 already says for every derive.

### Edit: Code table, E2040 row (change, p17a-codes.md 59)

**Before:**

> | `E2040` | unsatisfied interface bound; for a `pub` generic whose bounds are not written, or a generic that writes some of its bounds and not all the body needs, one message on the first line of its body that needs a missing bound, giving the whole line of what it needs | `[IFC-6]`, `[TYP-17]`, `[BLD-14]` |

**After:**

> | `E2040` | unsatisfied interface bound; for a `pub` generic whose bounds are not written, or a generic that writes some of its bounds and not all the body needs, one message on the first line of its body that needs a missing bound, giving the whole line of what it needs; an implementation that does not meet its interface's parents for every type it covers, at the generic implementation or at the derived class | `[IFC-6]`, `[TYP-17]`, `[BLD-14]`, `[IFC-3]`, `[ERR-3]` |

Why here: Objection I1-4.

Why general: It lists the refusal that IFC-3 now states.

### Impact

Rules changed: IV.8 (p04 676), ERR-3 (p13 87-88) and TYP-17's example (p04 459): Error requires Debug, Display, Send and ThreadShared, and a type holding a one-thread part is not an error. THR-8 (p11 47-48), THR-9 (p11 63-64) and the marker rows (p04 577-578): an object of an open or abstract class counts the fields of every class the program derives from it, and THR-9 gains the dyn clause. IFC-3 (p05 411-413): an implementation meets its interface's parents for every type it covers (generic instances, derived classes), with the refusal placed and the message's line stated as a property. DRV-1 (p14 138): a derive on a generic type holds per instance and writes those needs. p17a 59 and 71: the code rows. ERR-1, ERR-8, ERR-9, ERR-12, THR-1, THR-2, THR-10 and CLS-12 are unchanged and stay true.

Readability: Better. No word is added to ordinary code. The worker loses an import, a concrete error type and, with two kinds of failure, a whole error enum. Generic error code (`type Failure: needs Error`), wrappers holding `owns dyn Error`, and class hierarchies whose derived classes hold plain data stay exactly as they are today. Words are added in two places only. A hand-written generic error implementation whose parameter holds the caller's value writes `T: needs (Send, ThreadShared)` (a derive writes it itself). An interface whose values an error holds as `owns dyn I` gets `Send` and `ThreadShared` in its needs, which then holds every implementer of that interface to them, or else the error holds a copy of the data. Under option A, the bound would be written instead in every pub generic function that uses `?` on such an error. Refusals fall on the type that holds the one-thread part: an error type at its declaration, a derived error class at its own line, and a plain open-class value at the line that sends it.

Cost: Compile time: Send and ThreadShared are inferred as today. For each open or abstract class, the program build collects the classes derived from it (a library's included, and each generic instance made). That is one walk of the class declarations, which EFF-24 already makes for virtual calls. Each implementation of an interface with parents is checked once per instance and once per derived class. A `?` into AnyError is checked as today. Run time: the class rule adds nothing. AnyError keeps its layout (the boxed error's data and table pointers, a 16-byte identity, the context list) and its one allocation on the failure path (ERR-11). The success path costs nothing, as in C, and handing an AnyError to another thread moves it, as C passes an error pointer. Two costs are stated. (1) CLS-12 (p05 380): a `@cached` method of a class that implements Error is filled under the once-flag, because its objects may now be read by several threads. Per call, the flag test becomes an acquire load (on x86-64 the same `mov` as a plain load; on ARM64 an `ldar`), and the first call fills under the flag with one compare-and-swap. This is already the duty today for any ThreadShared class given by `ref` to scoped tasks. (2) THR-2's hand-over check (verdict 114, "every object the handed object owns") reaches objects the boxed error owns whose class has weakref links, through the error's table: one indirect call per AnyError handed over. Nothing runs in a program where no error type converted into AnyError owns such an object. To measure: AnyError's size and its failure-path time against today's build; the `@cached` once-flag against a plain flag on ARM64 and x86-64; and the hand-over check's time.

* verdict 61 (`thread_shared fn()`: the type says which threads may run it, not a whole-program search): kept (The Error interface says which threads may have an error, and option C (a search through the `?` lines of the whole program) is turned down for verdict 61's reason. The class rule reads the declarations of derived classes, which are the words of types that TYP-43 infers markers from. It does not read bodies to find which functions a field is given, which is what 61 turned down. Listed for the owner as the nearest tension (decision 3).)
* verdict 86 and 122 (what a function changes is worked out from every function the program can put in a place, a far edit naming the chain): kept (An open class's markers are worked out as its virtual calls already are (EFF-24, p10 94: "from every function the program can put there"). A refusal names the derived class and its field.)
* verdict 94 (thread.spawn takes a thread_shared fn(); R must be Send): kept (spawn's line is unchanged. AnyError now meets `R: Send`.)
* verdict 110 (an interface's parents written with `needs`): kept (`interface Error: needs (Debug, Display, Send, ThreadShared):`, in all three copies.)
* verdict 108, 111 (pub generics and extends write their needs with `needs`): kept (A hand-written generic implementation writes `T: needs (Send, ThreadShared)`: a library's promise is in its line.)
* verdict 113 (a private generic's needs worked out): kept (Unchanged. A derive writes its generated needs itself (DRV-1), as the compiler works out what it can.)
* verdict 132 (a declaration means what it says): kept (`extend[T] X[T] implements Error` is never quietly read as "only for some T". It is refused, with the bounded line given. A derive's generated `extend` says its bounds.)
* verdict 164 (a sharedref mut link never goes to another thread): kept (An error holding one is not Send, so it is refused where it implements Error.)
* verdict 165 (thread_shared refused where the value is made when not ThreadShared): kept (The same 'refused where it is made' shape. A thread_shared holder of results is now accepted, because AnyError is ThreadShared.)
* verdict 23 (a shares value is never used from two threads): kept (No shares owner can be inside an error, inside a derived error class, or inside a base-class value that crosses threads.)
* verdict 114 (a handed-over object's weakref links say gone): kept (THR-2's words already reach objects owned through the error's box. The cost is stated above.)
* verdict ODR-255 (dyn I is Send/Sync when I requires it): kept (Its ThreadShared half is now written in THR-9.)
* verdict ODR-050 (Result's error parameter is unbounded): kept (A one-thread error type is still a valid `Result` error, printed by main through `Display` (ERR-12). It is just not an `Error`.)
* verdict ODR-027 (a range is `Copy` when its bound is): kept (DRV-1 now states that reading for every derive on a generic type.)
* a program changes: E7015 at the spawn line, which no text explains -> compiles; main prints the file's length, or `error: <the system's message>` with status 1
* a program changes: E7015 at the spawn line -> prints `8080` then `error: the text is not a value of this type`
* a program changes: E7015 -> prints 2
* a program changes: E7015 at that line (verdict 165) -> compiles; a thread counts the loaded sizes and prints 1
* a program changes: E7015 at the spawn line -> prints 3; no thread word written
* a program changes: compiles -> E2080 at `@derive(Error)` (E2040 at a written `extend ... implements Error:`), naming the field, with a line that compiles: a copy of what the error uses (`path: String` with `copy of doc.path`). `thread_shared` is offered only where no line then changes the value directly (objection E1-2's `shares mut Doc` gets the copy line). Such a type can still be a `Result`'s error shown with Display.
* a program changes: compiles -> E2080 at the derive, naming the field. The message gives `interface Shape: needs (Debug, Send, ThreadShared):` (which then holds every implementer of `Shape` to the markers, so a `Sprite` holding `shares Texture` becomes E2040 at its `implements Shape`), or a copy of the data (`TooLarge(area: int)`). Under option A, a one-thread renderer that matches `Result[int, DrawError]` would compile.
* a program changes: compiles -> E2040 at its `implements Error`, naming `retry`. The message gives a copy line (the data the retry would read). It does not offer `thread_shared fn()`, whose ThreadShared the text does not yet decide (an existing gap, in notes).
* a program changes: compiles (the Error case); undecided (the others) -> E2040 at that line, giving the bounded line (`extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:`)
* a program changes: compiles -> compiles; the derive implements Error for the instances whose `T` is Debug, Send and ThreadShared; a `?` of a `SpawnError[shares Enemy]` into AnyError is E2040 at the `?`
* a program changes: compiles (it cannot reach another thread inside an AnyError today, because AnyError is not Send) -> E2040 at `class DocError(GameError):`, naming `doc`, checked in DocError's package even when GameError is a library's
* a program changes: compiles, and the derived object's plain count races (an existing hole) -> E7015 at that line, naming the derived class and its field, and giving a copy line (`entity = s.entity` and `fn() copy of entity: entity`)
* a program changes: runs -> runs, unchanged (prints 91 in the test)
* a program changes: always allowed, and races when a component holds a one-thread part -> allowed when every component type the program stores is Send (and ThreadShared for reads), and E7015 at the line otherwise, naming the component and its field
* a program changes: refused (AnyError not ThreadShared) -> prints 68; CLS-12's once-flag applies, since every object of a class that implements Error is an object several threads may reach
* a program changes: may be an Error -> refused where it implements Error, until ThreadShared can be vouched for (open question P04/J7-06)

**Decision (made by the owner's parameters): Where the thread markers of an erased error come from: Should every error type be safe to hand to and read from any thread (as Swift's errors are), so the catch-all error (like C++'s `std::exception_ptr`; Ember's `AnyError`) can come back from a worker thread?**

step | who | what happens | result
1 | error type's author | declares an error of plain values (a code, a text) | allowed: it may go to any thread
2 | worker thread | `fs.read_to_string(path)?` puts the file error into the catch-all | allowed, nothing written
3 | worker thread | returns `Result[int]` (a count or the catch-all error) | allowed: every error may cross threads
4 | main thread | `n = h.join()?` | gets the count, or passes the error on
5 | error type's author | declares an error holding a one-thread shared owner, or an object of an interface that does not say threads may have it | refused at that declaration, with the line to write

Today: The worker returning `Result[int]` is refused at `thread.spawn`, the text never says so, and the worker must name its own error type.

* A: the catch-all checks each error where `?` puts it in; generic error code and error wrappers must add thread words
* B: keep today: the catch-all stays on its thread; every worker names its own error type
* C: the compiler searches the whole program; one one-thread error anywhere breaks every thread hand-off, far from that line
* D: every error type is safe for every thread, checked where the type is declared an error; nothing written anywhere else
* E: the catch-all may be sent but not read by two threads at once; a shared table of results is refused
* F: check while running and stop the program when a one-thread error crosses

Recommendation: D: ordinary worker, server and library code compile with nothing written; only an error holding a one-thread part is refused, at its own line, with the fix.

Chosen: D. By: 1. The three goals at once. B fails 'easy to write': every worker grows its own error type, and an enum when it can fail two ways. A and E fail it for generic error code and wrappers, which grow thread words. C, D and F keep ordinary code as it is today. 2. Safety: A to E all refuse before a one-thread value crosses, and F stops the program. D is sound only with the class components counted as THR-8's definition reads them (decision 3) and with derived classes checked (IFC-3), both in this change. 3. Compile time first: A to E are decided when compiling. F's run-time check cannot make the refused program work: a shares owner inside the error cannot be moved away from its other owners, so F can only stop it. A compile-time refusal with a line to write would become a crash on the least-tested path (checklist 8), so F is out. 4. A written word only for meaning: A and E write markers in bounds and after `dyn`, and B writes a concrete error type. D writes a bound only on a hand-written generic error implementation (the library's promise, verdict 108), and the markers on an interface only when an error holds its values, where they state which threads may have those values. 5. The ordinary program no longer or harder: C lets an edit anywhere break a spawn far away. D refuses only an error type holding a one-thread part, at its own declaration, even in a one-thread program, and the fix is to hold a copy. A refuses the same errors at their `?`, which nearly every error reaches because main returns `Result[void]`. A accepts an error that is only matched, as in objection E1-1's renderer, which D refuses. 6. Consistency: D uses THR-8's own dyn clause, IFC-3, ODR-255, verdict 110's spelling, verdict 61 (the type says which threads may have it) and verdict 165 (refused where it is made). It matches Swift (`Error: Sendable`) and Rust's anyhow (`Send + Sync`). So D.

**Decision (made by the owner's parameters): A generic implementation whose interface has parents (a generic error that holds the caller's value): A generic error that hands the caller's value back (like Rust's `SendError<T>`): is it an error for every `T`, or only for the `T`s any thread may have, and who writes that?**

step | who | what happens | result
1 | library author | writes `extend[T] PoolFull[T] implements Error:` | refused, with the line to write
2 | library author | writes `extend[T: needs (Send, ThreadShared)] PoolFull[T] implements Error:` | allowed
3 | library author | writes `@derive(Error)` on a generic enum | allowed; the derive writes the bound itself
4 | program | `insert(...)?` with `PoolFull[int]` into the catch-all | allowed
5 | one-thread game | `insert(...)?` with `PoolFull[shares Enemy]` into the catch-all | refused at that line: name the error type instead

Today: Nothing says; the implementation covers every `T`, because errors need no thread markers today.

* (i) a hand-written implementation writes the bound, or is refused with the line to write; a derive writes it itself
* (ii) the implementation quietly holds only for the safe `T`s
* (iii) a generic error type must bound its parameter on the type itself

Recommendation: (i): the line says what it means, the compiler gives the line to write, and derives keep working per instance as they already do.

Chosen: i. By: 1. The three goals: all are equal for ordinary code, because generic error types holding a caller's value are rare. (iii) is harder: a type used only as `SpawnError[int]` is refused. 2. Safety: all three are safe. 3. Compile time: all three. 4. A written word only for meaning: the bound on a hand-written implementation is the library's promise of which values it is an error for (verdicts 108, 111). A derive is the compiler writing the implementation, so it writes the bound itself and `ember inspect --expand` shows it (principle 1). 5. The ordinary program: unchanged under (i). 6. Consistency: IFC-3 says implementers implement the parents, Rust requires a supertrait to hold for every type an impl covers, verdict 132 says a declaration's meaning comes only from what it says, and STR-5, ODR-027 and std's `@derive(Copy)` on `[T] pub struct Range` make derives per instance. So (i), stated once in IFC-3 for every interface and in DRV-1 for every derive.

**Decision (made by the owner's parameters): What an object of an open class may hold, for the thread markers: When a base-class object (C++'s `Base*` that may point at a `Derived`) is handed to another thread, should the compiler look at every derived class the program declares?**

step | who | what happens | result
1 | game author | `open class Script`, and `class Door(Script)` with plain fields | a Script may go to another thread
2 | game author | adds `class Enemy(Script)` holding a one-thread shared owner | allowed
3 | worker | `thread.spawn(owns fn() owns s: ...)` with `s: Script` | refused at that line, naming Enemy's field
4 | error author | `class DocError(GameError)` holding a one-thread shared owner, GameError being an error | refused at DocError's line

Today: The compiler looks only at the base class's own fields, so a Script that is an Enemy goes to another thread and its shared count races.

* (i) look at every derived class the program declares; nothing written
* (ii) a base class's objects never go to another thread unless the base promises it through an interface (Swift)
* (iii) every derived class must be as thread-safe as its base's own fields
* (iv) keep today

Recommendation: (i): correct hierarchies keep working with nothing written, and only the class that holds the one-thread part is named.

Chosen: i. By: 1. The three goals: (iii) refuses ordinary class hierarchies. (ii) refuses every open-class value crossing threads or read by tasks unless an interface is written only to carry the markers, and it breaks std's own ECS World. (i) and (iv) keep ordinary code. 2. Safety: (iv) lets a one-thread owner cross, so it is out. 3. Compile time: (i), (ii) and (iii) are all decided when compiling. 4. A written word only for meaning: (i) writes nothing, and (ii) writes an interface that says nothing a declaration does not already say. 5. The ordinary program: (i) accepts every program whose derived classes are thread-safe. Its refusal is at the line that sends the value and names the derived class, the cost verdict 86 accepted for effects. 6. Consistency: (i) is what THR-8's own definition says ("nothing reachable from it"). It is how the language already treats an open class's virtual calls (EFF-24, p10 94: "from every function the program can put there"; verdict 122) and its views (p07 713). And it infers markers from the words of types, as TYP-43 says. (ii) matches THR-8's dyn clause and Swift, but a class has no needs list, and (ii) loses on parameters 1 and 5 first. Where a base does promise the markers, by implementing an interface that needs them (Error), each derived class is checked at its own line (IFC-3, decision 2's property). So (i).

### What blocks it (2)

* S2-1 (S, unsafe_allowed): A use after free and a double free of a one-thread shared value, through the default error type. Nothing is written beyond `thread.spawn`, in the ordinary engine/game split: an engine library with an open `Script` (or `Component`, `Node`, `Widget`) and an error type that keeps the failing object, and a game whose derived class holds a `shares` owner, a `weakref` target or a `Cell`. The ThreadShared half breaks the same way: a derived class's `Cell` read by two scoped tasks through `ref r`, or under a thread_shared RwLock of results. Hot reload reaches the same race by adding a field to a derived class. Every soundness argument for AnyError rests on 'every implementation of Error meets Send and ThreadShared in the final program', and across a package's edge or a reload nothing in the changed text makes that true.
  Arbiter: Walked through the changed text. (1) gamekit's build. THR-8 after (p11 47-48) counts "any class the program derives from it, a library's included". gamekit derives nothing from Script, so LoadError (`script: Script`, `entity: int`) is Send and ThreadShared, and `extend LoadError implements Error:` meets IFC-3 after ("An implementation meets its interface's parents for every type it covers"). (2) game's build. IFC-3 after places the refusal only "either at the generic implementation ... or where the derived class is declared", and the derived-class check reaches only "each class derived from a class it covers". The Error implementation covers LoadError, a struct. It does not cover Script, and Script implements nothing, so `class Enemy(Script):` is checked against nothing. ERR-3 after says the same: "or where it is derived from a class that does". (3) MOD-9 (p05 38-46) says what crosses a package's edge: the written words, plus the facts it lists ("what each function may change, read and end ... the locks it may take; the function values it stores and the callbacks it hands to C"). No edited word records that gamekit relied on Script's markers. No edited word makes game's build check gamekit's implementation again either. (4) The `?` into AnyError needs `E: needs Error` (ERR-8, p13 95), and gamekit's written implementation meets it. AnyError's `Box[dyn Error]` is Send by p11 51 ("`dyn I` when `I` requires it"), which reads the interface and never what the box holds. So the spawn compiles. The worker ends the Enemy's `shares` owner while main changes the same count with plain adds (RC-4 / THR-2 p11 100-101): a race on a non-atomic count, then a use after free or a double free. Today the same program is E7015 at the spawn: p04 676 and p13 88 give Error only Debug and Display, so by p11 51 `dyn Error` is not Send. So the change causes it. The other reading (game's build checks gamekit's line 20 again) is refused at a dependency's line that the text places nowhere. One of the two readings lets a race through, so this is S. The reload route (HR-44 compares only functions' facts) and the generic-derived-class route come from the same missing rule. I did not walk them separately, and the first route settles the ruling.

```
# file gamekit/ember.toml
#   [package]
#   name = "gamekit"
#   version = "0.1.0"
#   language = "0.9.15"
#   kind = "lib"
# file gamekit/src/lib.em (line numbers below count from its first line)
from std.fmt import Formatter, FmtError

pub open class Script:
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

extend LoadError implements Error:
    pass
# file game/ember.toml
#   [package]
#   name = "game"
#   version = "0.1.0"
#   language = "0.9.15"
#   [dependencies]
#   gamekit = { path = "../gamekit" }
# file game/src/main.em (line numbers below count from its first line)
import std.thread
from gamekit import Script, LoadError

class Level:
    name: String = String("forest")

class Enemy(Script):
    level: shares Level

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn make_enemy(entity: int, owns level: shares Level) -> Result[int, LoadError]:
    return Err(LoadError(script=Enemy(entity, owns level)))   # an Enemy, upcast to Script

fn load(entity: int, owns level: shares Level) -> Result[int]:
    return Ok(make_enemy(entity, owns level)?)                # LoadError -> AnyError

fn main():
    level = shares Level()
    r = load(1, shares level)                                  # r's error holds a second owner of the Level
    h = thread.spawn(owns fn() owns r: r.is_err())             # r goes to another thread, which ends it
    for i in 0..100000:
        other = shares level                                   # +1, then -1 at the end of the round: plain adds
        println(i, other.name.len())
    println(h.join())
```

* E2-1 (S, unsafe_allowed): Decision D is sound only if every error's markers hold in the build that uses it. For an error type in one package that holds an object of an open class (a field, an element, an `owns` part), the markers were judged without the classes another package derives later, and every AnyError, channel of results, scoped `ref` and thread_shared holder of results then carries a one-thread owner across threads. Library error types holding a base-class object are ordinary: a game engine's ScriptError holding the failing Script, a UI toolkit's error holding the Widget. The impact report says the class rule closes the open-class hole, but across a package edge it does not, and it gives the library-line placement only in to_verify.
  Arbiter: Same cause as S2-1, a second program. In engine's build, THR-8 after (p11 47-48) sees no derived class, so ScriptError meets IFC-3 after at its `implements Error`. EFF-4 (p10 52-56) says "a fact for which a word exists counts only as written", and the implementation is written. In game's build, IFC-3 after's two refusal places ("at the generic implementation ... or where the derived class is declared" for a class "derived from a class it covers") miss `class Enemy(Script):` and `fail(owns s)?`. `Sender[Result[int]]` is Send because AnyError's `owns dyn Error` is Send by requirement (p11 51, with p04 676 after). So the spawn's `owns tx` compiles, and main ending `e` takes a plain -1 from `level`'s count while the worker's loop changes it: a race, then a double free or a leak. Today it is E7015 at the spawn, because Error does not require Send. So the change causes it. The objection also points out something true: the concrete route `channel[Result[int, ScriptError]]` is refused after the change, because game's build infers ScriptError's Send with Enemy counted. Only the erased route trusts a requirement that was checked where Enemy could not be seen. One repair closes both S2-1 and E2-1. to_verify item 6 ("reported in the program's build, at the line that passes the value") is in no edited rule.

```
# package engine (a library): engine/src/lib.em
from std.fmt import Formatter, FmtError

pub open class Script:
    pub entity: int

    pub fn init(entity: int):
        this.entity = entity

pub struct ScriptError:
    pub script: Script

extend ScriptError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the script failed")

extend ScriptError implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("ScriptError")

extend ScriptError implements Error:
    pass

pub fn fail(owns s: Script) -> Result[int, ScriptError]:
    return Err(ScriptError(script=s))

# package game (uses engine): game/src/main.em
import std.thread
from std.sync import Sender, channel
from engine import Script, fail

class Level:
    name: String = String("forest")

class Enemy(Script):
    level: shares Level

    fn init(entity: int, owns level: shares Level):
        this.level = level
        super.init(entity)

fn tick(owns s: Script) -> Result[int]:
    return Ok(fail(owns s)?)                         # ScriptError -> AnyError

fn work(tx: Sender[Result[int]]):
    level = shares Level()
    tx.send(owns tick(owns Enemy(1, shares level))).expect("receiver is gone")
    n = 0
    for _ in 0..1000000:
        extra = shares level                         # plain add and sub on this thread
        n += extra.name.len()
    println(n)

fn main():
    tx, rx = channel[Result[int]](capacity=1)
    h = thread.spawn(owns fn() owns tx: work(tx))
    for r in rx:
        match r:
            Ok(n) => println(n)
            Err(e) => println(e)                     # the Enemy ends here: a plain sub on the main thread
    h.join()
```

Arbiter's rulings on the open objections: 5 real, 0 refuted, 0 already in today's specification

Objections closed in the debate: 11 (11 fixed, 0 withdrawn after a rebuttal)

Gaps found on the way that exist without the change: 4

