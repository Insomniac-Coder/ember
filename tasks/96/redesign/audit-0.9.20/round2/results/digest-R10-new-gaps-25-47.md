# R10-new-gaps-25-47

could_not_cover: Nothing in NG-25 to NG-47 was left unjudged. Two limits of the reading: (1) the K07, K10, K20 and K21 design documents in the inputs folder (marked rejected or unresolved) were scanned for their direction only, not read whole, since the task is to judge against CURRENT-SPEC.md, which contains none of their edits; the edits proposed for NG-32 to NG-41 are one coherent set (FFI-10's list, FFI-21a by direction, the std::function row, UNS-1's store) rather than K07's wording, and the lead should merge them with K07's answer, including the rewrite of the XVI.4 Vulkan overlay that K07 and K46 both touch. (2) The `old` texts are copied from the hard-wrapped CURRENT-SPEC.md with the line breaks joined by one space, as the asserting scripts re-wrap; a lead applying them against the wrapped file should match on whitespace-normalised text.

## NG-25  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: THR-8: "Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is; an `owns T` when `T` is"; THR-8: "`dyn I` when `I` requires it (`interface I: needs Send:`)"; TYP-5 rule 8: "a class object to a base class (upcast), with its word kept: a `Dog` to an `Animal`"; THR-10: "`R` MUST be `Send` (`E7015`, `E7008`)"; ERR-3: "`Error` is the interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`)"; CLS-4: "A class is final unless declared `open` or `abstract`."; BLD-16: "No fact of the whole program decides a package's code or layout."

**program**: class SaveSlot:
    index: int = 0

open class GameError:
    message: String
    fn init(owns message: String):
        this.message = message

extend GameError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(this.message)
extend GameError implements Error:
    pass

class SaveError(GameError):
    slot: shares SaveSlot
    fn init(owns message: String, slot: shares SaveSlot):
        this.slot = shares slot
        super.init(owns message)

fn save(slot: shares SaveSlot) -> Result[int, GameError]:
    return Err(SaveError(owns String("disk full"), shares slot))   # upcast, TYP-5 rule 8

fn main():
    slot = shares SaveSlot()
    r = save(slot)
    h = thread.spawn(owns fn() owns r: r.is_err())   # R and the capture: Result[int, GameError]; GameError's one field is a String, so Send by THR-8's list
    again = shares slot                              # plain +1 on this thread while the worker ends the SaveError and its `shares slot` with a plain -1
    println(h.join(), again.index)

The current text judges `GameError` by its own fields (THR-8's list), and nothing judges the object that may stand behind a value of an `open` class type (TYP-5 rule 8 upcasts it with its word kept). So the spawn compiles and the `SaveSlot` count is touched from two threads: a data race and a read after free (a PHIL-10 promise broken).

- READER EDIT rule=THR-8
  - old: Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is;
  - new: Structs, enums, tuples, fixed arrays, collections and objects of a class that cannot be subclassed are `Send` when every component is; a value whose type is an `open` or `abstract` class may hold an object of any class derived from it, in any package (`[TYP-5]` rule 8, `[BLD-16]`), so it is `Send` as a `dyn I` is: when an interface the class implements requires it (`interface I: needs Send:`), never from its own fields alone; a class derived from one that implements such an interface meets it too, checked where the derived class is declared, as an implementation's parents are (`[IFC-3]`): a field of it that is not `Send` is `E7015` at the field ("`SaveError.slot` is a `shares SaveSlot`, a one-thread owner, and a `GameError` may go to another thread (`extend GameError implements Shared`, line 19, `Shared: needs Send`); hold a copy, `slot_index: int`, or share the slot between threads, `slot: thread_shared SaveSlot`");
  - why: The cause is THR-8's list, which reads a class's own fields where a base-class value may hold a derived object the declaring package never sees. Reusing the `dyn I` rule keeps BLD-16 (the base's interfaces are in its interface file, so a dependant's derived class is checked against them where it is declared) and adds no word; under the current `Error` (no `Send`), `Result[int, GameError]` is simply not `Send` and the spawn is `E7015`, a safe refusal; with K20's `Error: needs Send` the refusal moves to `SaveError.slot`, with the line to write.

- READER EDIT rule=THR-9
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types are `ThreadShared` when every component is; a value of an `open` or `abstract` class type is `ThreadShared` as a `dyn I` is, when an interface the class implements requires it, checked for every derived class where it is declared (`[THR-8]`).
  - why: The same hole for the second marker: several threads reading a base-class value that holds a derived object with a `Cell` or a `shares` field.

**remaining_doubt**: Which closure the owner prefers is a design choice safety does not settle: (a) a derived class keeps every marker its base's own fields give (refuses ordinary game hierarchies, `class Enemy(Script)` with a `weakref Player`, whenever the base has plain fields); (b) this edit, the `dyn I` rule (a base-class value crosses threads only when the class implements a marker-requiring interface; one `interface Shared: needs (Send, ThreadShared): pass` and one `extend` line make it so); (c) the K20 design's choice (every derived class the program declares, a library's included), which BLD-16 forbids. Recommendation (b).

**REFUTER reason**: The hole is real. THR-8 judges a class value by its own fields: "Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is"; TYP-5 rule 8 lets a derived object stand behind a base-class value: "a class object to a base class (upcast), with its word kept: a `Dog` to an `Animal`"; and nothing else reads a base-class value's markers (THR-2 names "an object of an `open` or `abstract` class (`[CLS-4]`)" only for the weak-count walk). So `Result[int, GameError]` with `open class GameError: message: String` is `Send`, THR-10's "`R` MUST be `Send`" and the `owns r` capture pass, and the `SaveError`'s `shares SaveSlot` ends on the worker with a plain subtract while `again = shares slot` adds on main: the data race PHIL-10 forbids. THR-2's own parenthetical "(a handed value owns no `shares` value and holds no `weakref T` link, `[THR-8]`)" rests on the promise THR-8 here fails to keep; the K20 design record states the same gap ("That gap already exists for a plain open-class value handed to a thread"), and no verdict decides it. The reader's closure (b), the `dyn I` rule, is the right shape (opt-in per base through an interface the base's interface file carries, so a dependant's derived class is checked where it is declared, `[BLD-16]`; no run-time cost), but its text has two gaps: (1) it says an open class is `Send` "never from its own fields alone" yet relies on `extend GameError implements Shared` with `Shared: needs Send`, which IFC-3 checks by `Send` inferred from the class's fields, so the base's own fields must be judged at the `extend` or the rule is circular; (2) the marker table in TYP-24 says "a class object is when its fields are" and the `E7015` registry row lists the cases, and both must change with THR-8 or two texts decide one program differently. The better edits below add both and keep the reader's THR-9 twin.

- BETTER EDIT rule=THR-8
  - old: Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is;
  - new: Structs, enums, tuples, fixed arrays, collections and objects of a class that cannot be subclassed are `Send` when every component is. A value whose type is an `open` or `abstract` class may hold an object of any class derived from it, declared in any package (`[TYP-5]` rule 8, `[BLD-16]`), so it is `Send` as a `dyn I` is: when an interface the class implements requires it (`interface Shared: needs Send:`), never from its fields alone; at the `extend` the class's own fields meet the requirement (`[IFC-3]`, `E2040`), and a class derived from one that implements such an interface meets it too, judged from its own fields where it is declared: a field that is not `Send` is `E7015` at the field ("`SaveError.slot` is a `shares SaveSlot`, a one-thread owner, and a `GameError` may go to another thread (`extend GameError implements Shared`, line 19; `Shared: needs Send`); hold what the slot gives, `slot_index: int`, or share the slot between threads, `slot: thread_shared SaveSlot`"); a value of such a class given to another thread where no interface the class implements requires `Send` is `E7015` at that line ("`r` holds a `GameError`, an `open` class, so the object behind it may be of any derived class; to send `GameError` values between threads write `interface Shared: needs (Send, ThreadShared): pass` and `extend GameError implements Shared: pass`");
  - why: THR-8's list is where a base-class value is judged by the wrong fields; the `dyn I` rule already in the same sentence is the general answer, and the base's own fields judged at the `extend` closes the circularity the reader's text left.

- BETTER EDIT rule=TYP-24 marker table, row `Send`
  - old: a class object is when its fields are;
  - new: a class object is when its fields are, for a class that cannot be subclassed; a value of an `open` or `abstract` class type is when an interface the class implements requires `Send`, as a `dyn I` is (`[THR-8]`);
  - why: The table repeats THR-8's class sentence; left as it is, the two texts would decide one program differently (consistency goal: every cross-reference claims what its rule says).

- BETTER EDIT rule=THR-9
  - old: Structural types are `ThreadShared` when every component is.
  - new: Structural types are `ThreadShared` when every component is; a value of an `open` or `abstract` class type is `ThreadShared` as a `dyn I` is, when an interface the class implements requires it, the class's own fields judged at the `extend` and every derived class's where it is declared (`[THR-8]`).
  - why: The same hole for the second marker: several threads reading a base-class value that holds a derived object with a `Cell` or a `shares` field.

- BETTER EDIT rule=registry row E7015
  - old: a one-thread owner in a trampoline class under `threads=any` |
  - new: a one-thread owner in a trampoline class under `threads=any`; a value of an `open` or `abstract` class type given to another thread where no interface the class implements requires `Send`, and a field that is not `Send` in a class derived from one that implements such an interface, at the field |
  - why: One code, one defining rule: the row lists THR-8's cases.

## NG-26  reader=needs_owner  refuter=confirmed  should_be=needs_owner  safety=none

**answered_by**: TYP-26: "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types."; FFI-48: "| overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |"; FFI-40: "a `const` method only reads its object when `[FFI-40a]` finds so, and every other method may change it, so it is not called through a name that only reads the object (`E3023`)"; FFI-40a: "`@ffi(const)` above a method's entry (`fn lookups()`) asserts that the method only reads its object"; FFI-12: "An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`"

**program**: The implementer's question: `class Config { const Entry& at(size_t) const; Entry& at(size_t); }` imports two methods named `at` on one class. TYP-26 makes two functions of one name in one scope `E1030`, FFI-48 refuses only overloads that differ in return type (so the others import, but no rule says as what), and GRM-35's entries and FFI-12 match an entry by name, count and parameter names, which cannot tell the two `at` apart. So `cfg.at(0)` on a local `cfg` is E1030, E2070, or one of the two, and two compilers disagree. K21's design (unresolved) decides this; no verdict does.

**remaining_doubt**: Options: (a) a const and a non-const method that both fit the object are ambiguous (`E2070`): safe, but the non-const `at`, `front`, `back`, `data`, `begin` can never be called through Ember, the ordinary C++ accessor pairs; (b) the object counts in the fit as an argument does: a changeable name fits the method that may change the object as its own type and the read-only method through a read-only coercion (`[TYP-24]`'s ranking), so a changeable object runs the non-const `at` and a read-only one the const `at`, as C++ does, the non-const result a changing view of the object while it is used; (c) the overlay chooses by `@ffi(const)` on one entry and not the other (K47's shape), the header alone leaving it `E2070`. Recommendation (b), which runs what C++ runs at no run-time cost and refuses nothing C++ accepts; it presupposes K21's first decision (one Ember name per C++ name, each use standing for the declaration its words fit), without which the import of two `at` has no answer at all.

**REFUTER reason**: An implementer cannot decide the line. TYP-26: "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types." FFI-48 refuses only "overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay", which implies every other overload imports, yet no rule says as what, and the two `at` differ in the receiver's `const` as well as in the result, so even FFI-48's row does not clearly reach them. The overlay cannot tell them apart either: FFI-12 matches "name, parameter count or parameter names", and FFI-30c makes "any other second entry for one name" `E5010`, so `@ffi(const)` on one entry and not the other (the reader's option (c)) is not writable today. Only FFI-39b decides a C++ name declared several times, and only for `super.init(…)` ("selects the C++ base constructor by arity and argument types"). The K21 design is marked unresolved and no verdict (130 to 136 are the overlay verdicts) touches overload import, so the question is open for the lead's design round, not for a line edit here. The reader's recommendation (b) is safe (the non-const result is a reference result, which FFI-40 keeps `unsafe` without a lifetime word), and refuses nothing C++ accepts; nothing to check in edits since none is proposed.

## NG-27  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: TYP-26: "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types."; FFI-39: "An `override fn` matching no virtual method of the base, by name and signature, is `E5056`."; FFI-48: "| overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |"

**program**: 

- READER EDIT rule=TYP-26
  - old: except operator-interface implementations and `extend` blocks for different types.
  - new: except operator-interface implementations, `extend` blocks for different types, and the `override fn`s of a class derived from a C++ class whose header declares one virtual name several times (`[FFI-39]`): the class writes one `override fn` per inherited declaration, each replacing the declaration its parameters fit, since the header, not the program, made them several.
  - why: The refusal is at TYP-26, whose reason (two functions of one name written by the program) does not hold for overrides the header forces; the exception states why, as principle 6 asks.

- READER EDIT rule=FFI-39
  - old: An `override fn` matching no virtual method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual method of the base, by name and signature, is `E5056`; where the base declares a virtual name several times, each `override fn` of that name replaces the one declaration its parameters fit, one per declaration (`[TYP-26]`), and two overrides fitting one declaration are `E1030`.
  - why: Names the matching rule where the trampoline is built, so the slot each override fills is decided by its parameters, as FFI-39's own 'by name and signature' already reads them.

**remaining_doubt**: The text decides the program safely (`E1030` at the class's second `on`), only not as the finder wished: the visitor and event-handler pattern over a C++ base is refused. The edit depends on K21's first decision (one Ember name per C++ name); if K21 chooses per-overload names (its option B or C), the overrides follow those names instead and this edit is moot.

**REFUTER reason**: The status is right: the text decides the program, and safely. TYP-26: "two functions of one name in one scope are `E1030`" refuses the class's second `override fn on`, and FFI-39's "An `override fn` matching no virtual method of the base, by name and signature, is `E5056`" is the only matching rule for overrides. The edits, however, should not be applied in this pass. They presuppose K21's first decision (one Ember name per C++ name, each use standing for the declaration its words fit), which the K21 record marks unresolved: with the edit alone, `InputLog` may write two `on`s, while the base's two imported `on`s are still `E1030` or undecided (NG-26), and an Ember-side call `log.on(k)` has no rule to choose between the class's two `on`s, since the exception covers only how the trampoline fills its slots. So the edit opens the question it needs K21 to answer. Carry the finding into K21's revision (the reader's wording of the FFI-39 matching sentence is a good draft for it), and apply nothing now.

## NG-28  reader=not_answered  refuter=refuted  should_be=answered  safety=none

**answered_by**: FFI-8: "| `short`, `int`, `long`, `long long` and unsigned forms | `i16`, `c_int`, `c_long`, `i64`, and `u16`, `c_uint`, `c_ulong`, `u64` |"; STD table (std.ffi row): "`CString`, `CStrView`, `c_int` and the other C aliases, `c_wchar`, `WideString`, `Retained`, `Callback`, `adopt` (which gives an `owns T`) (Part XVI); layer: **ffi**"; FFI-8: "`c_wchar` (`u16` on Windows, `i32` elsewhere — not portable)"

**program**: The implementer's question: is `c_int` the type `i32`, and `c_long` the type `i32` on MSVC and `i64` on SysV (aliases, as the std table's words 'the other C aliases' and the `c_wchar` row suggest), or a type of its own? On an alias reading `put(int)` and `put(long)` import with one signature on MSVC, `out.put(n)` can name neither, and a program that passes an `i32` where a C `int` is wanted compiles on one reading and is `E2020` on the other. No rule says.

- READER EDIT rule=FFI-8
  - old: | `short`, `int`, `long`, `long long` and unsigned forms | `i16`, `c_int`, `c_long`, `i64`, and `u16`, `c_uint`, `c_ulong`, `u64` |
  - new: | `short`, `int`, `long`, `long long` and unsigned forms | `i16`, `c_int`, `c_long`, `i64`, and `u16`, `c_uint`, `c_ulong`, `u64`; `c_char`, `c_int`, `c_uint`, `c_long`, `c_ulong`, `c_wchar` and `c_longdouble` are types of their own (`std.ffi`), each with its target's representation, never aliases of a fixed-size type: a value of one converts to a fixed-size type of the same or a wider representation by `[TYP-5]` rule 1 (lossless on every target: `c_int` to `i32`, `c_long` to `i64`) and a fixed-size value to one that holds it on every target the same way (`i32` to `c_int`, `i16` to `c_long`); every other conversion is written `as`; so two C declarations that differ only in `int` against `long` are two signatures on every target, and an overlay entry can name either (`[GRM-35]`) |
  - why: The identity of the C integer types is left unsaid where the mapping is given; stating it there decides every program that mixes them with Ember's fixed-size types, and keeps a C++ overload set over `int` and `long` nameable on every target, which an alias cannot.

**remaining_doubt**: The alternative is Rust's (plain aliases), which is simpler to write against but makes a program's acceptance differ by target (an `int`/`long` pair collides on MSVC only) and which the std table's word 'aliases' may already intend; K21's design (unresolved) counts literals as C++ types them (`c_int`), which the distinct-type reading needs. Low severity; the lead may rule it under the delegation.

**REFUTER reason**: The text decides it: the C integer types are transparent aliases. The std table's row for `std.ffi` lists "`c_int` and the other C aliases", FFI-8's rows read `c_char` as "(`i8` or `u8` per target)" and `c_wchar` as "(`u16` on Windows, `i32` elsewhere — not portable)", and RNG-1 fixes what an alias without an `in` clause is: "A `type` alias without one is transparent." So a program that passes an `i32` where a C `int` is wanted compiles on every target, as `safe fn abs(x: c_int) -> c_int` called with a literal in XVI.2's own example shows. The consequence the reader fears, `put(int)` and `put(long)` importing with one signature on MSVC, is a question of how a C++ name declared several times imports at all, which is K21's (NG-26), not of the aliases. The reader's edit would also add two coercion rules to TYP-5's "complete list" (`c_int` to `i32`, `i32` to `c_int`) and make `abs(5)` depend on a literal-typing rule the language does not have; it should not be applied.

## NG-29  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-40: "A `T&` parameter imports as `ref mut x: T`, `const T&` as a plain parameter `x: T`, and `T&&` as `owns x: T` where the type is movable"; FFI-39: "the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly the virtual methods it overrides (`override fn`), read from the class."; FFI-32: "A function returning an opaque class by value gives an `owns T`, as a constructor does: the result is moved to an object of its own, ended by the class's destructor"; FFI-11: "an `owns x: owns T` parameter, whose argument is handed over and which Ember then never frees"

**program**: // jobs.hpp: class Payload { explicit Payload(int); Payload(Payload&&) noexcept; ~Payload() /* frees buffer_ */; int size() const; };
//           class Sink { virtual ~Sink(); virtual void take(Payload&& p) noexcept {} };
//           void feed(Sink& s, int size) noexcept;   // { Payload p(size); s.take(std::move(p)); }  ~Payload() of p at its end
# overlays/jobs.em
unsafe overlay cpp "jobs.hpp":
    class Payload:
        @ffi(const)
        fn size()
# src/main.em
from std.ffi import c_int
import cpp "jobs.hpp" with (project="engine", overlay="overlays/jobs.em") as jb

class Counter(jb.jobs.Sink):
    bytes: c_int = 0
    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()
    override fn take(owns p: jb.jobs.Payload):   # FFI-40: `T&&` is `owns p`
        this.bytes += p.size()                   # `p` ends here: FFI-32 runs ~Payload on it

fn main():
    c = Counter()
    # SAFETY: `feed` calls `take` on the sink during the call only
    unsafe:
        jb.jobs.feed(ref mut c, 64)
    println(c.bytes)

Nothing says the trampoline makes `p` an object of its own: read plainly, the override ends C++'s stack object (a free of a stack address) and `feed` then runs ~Payload on it again (a double free of `buffer_`). The call-side twin, `add(owns splash)` to `add(Texture&&)`, hands the object over under FFI-11's `owns` ('Ember then never frees'), so the moved-from object's destructor never runs (a leak, not unsafety).

- READER EDIT rule=FFI-40
  - old: and `T&&` as `owns x: T` where the type is movable;
  - new: and `T&&` as `owns x: T` where the type is movable: C++ lends the object for a move, so this `owns` is not `[FFI-11]`'s. At a call from Ember the argument is handed over (`cache.add(owns splash)`), the thunk passes its address, and when the call returns the moved-from object is ended by `T`'s destructor, as C++ ends a moved-from local; in an `override fn` (`[FFI-39]`) the trampoline makes the parameter's object by `T`'s move constructor from the reference C++ passed, before the override runs, so the override ends an object of its own (`[FFI-32]`) and C++'s object stays C++'s to end; a `T&&` parameter of a type with no public move constructor is skipped (`W5033`), as an `&&`-qualified member is;
  - why: FFI-40 is where `T&&` gets its Ember spelling, and the spelling's meaning on each side of the boundary was left to the reader; the two sentences give the only reading under which each object is ended exactly once (`[OWN-2]`), on the side that owns it.

**remaining_doubt**: One move constructor per `T&&` override call is C++'s own cost for `std::move` into a by-value callee, not a cost Ember inserts; `ember inspect` may list it beside `[FFI-17f]`'s thunk calls.

**REFUTER reason**: The text accepts the program and leaves the trampoline's handling of `T&&` undecided, and the plain reading double-frees. FFI-40: "`T&&` as `owns x: T` where the type is movable" gives the override's parameter type; FFI-39: "the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly the virtual methods it overrides" says nothing of what the trampoline passes for a `T&&`; FFI-32's "the result is moved to an object of its own, ended by the class's destructor" speaks of results only; FFI-39c's "a parameter borrowed for the call alone takes the object, or `this`, as any plain parameter" speaks of Ember objects handed to C++. An `owns p: Payload` ends at the override's end (`[OWN-2]`), so without a moved object the Ember side runs `~Payload` on C++'s stack object and `feed` runs it again. The reader's edit decides both directions with each object ended exactly once, costs one move constructor per `T&&` override call (a movable type's own cheap operation; C++'s `std::move` into a by-value callee does the same work) and one destructor call on the moved-from shell at a call from Ember, as C++ ends a moved-from local, and it also decides the case FFI-40's "where the type is movable" left open (a `T&&` of a type with no public move constructor, skipped with `W5033` as an `&&`-qualified member is). It departs from no verdict (102 is about C pointer contracts). What it leaves: the inserted move is a thunk operation `[FFI-17f]` should list, noted under missed.

## NG-30  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-39: "An `override fn` matching no virtual method of the base, by name and signature, is `E5056`."; CLS-4: "A method named like an inherited one replaces it (ODR-028): over a virtual method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not"; DSP-2: "slots are assigned in declaration order, base first, and an `override` reuses the base's slot"

**program**: The C++ half is decided: `override fn on(e: gui.ui.KeyEvent)` over `virtual void on(const Event&)` matches no virtual method 'by name and signature' and is `E5056` (FFI-39). The Ember half is not: 

open class Event:
    pass
class KeyEvent(Event):
    code: int
class MouseEvent(Event):
    x: int

open class Handler:
    virtual fn on(e: Event):
        pass

class KeyLog(Handler):
    last: int = 0
    override fn on(e: KeyEvent):        # named like the inherited virtual `on`: CLS-4 says it replaces it; DSP-2 puts it in the base's slot
        this.last = e.code

fn send(h: Handler, e: Event):
    h.on(e)                             # the slot runs KeyLog.on with a MouseEvent read as a KeyEvent

fn main():
    log = KeyLog()
    send(log, MouseEvent(x=3))
    println(log.last)

No rule compares an override's parameter types, words or result with the method it replaces; the slot call reads a `MouseEvent` as a `KeyEvent` (a type confusion from safe code).

- READER EDIT rule=CLS-4
  - old: A method named like an inherited one replaces it (ODR-028): over a virtual method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not;
  - new: A method named like an inherited one replaces it (ODR-028): over a virtual method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not; an `override` has exactly the parameter words, parameter types and result of the method it replaces, since a call through the base's slot (`[DSP-2]`) passes what the base declared, and one that differs is `E2113` ("`KeyLog.on` takes `e: KeyEvent`, and the `Handler.on` it overrides takes `e: Event`; write `override fn on(e: Event):` and test the kind inside: `if e is KeyEvent as k:`");
  - why: CLS-4 is where replacement is decided by name alone; the slot's call passes the base's declared types, so the override must take them, and the message gives the kind test (`[DSP-4]`) as the line to write.

**new_code**: | `E2113` | an `override` whose parameter words, parameter types or result differ from the virtual method it replaces | `[CLS-4]` |

**remaining_doubt**: Whether a wider parameter type or a narrower result should be allowed (contra- and covariance) is a later refinement; 'exactly' is the safe floor and matches DSP-2's fixed slot width.

**REFUTER reason**: The hole is real. CLS-4 decides replacement by name alone: "A method named like an inherited one replaces it (ODR-028): over a virtual method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not"; DSP-2: "an `override` reuses the base's slot"; the only signature rule for overrides is FFI-39's `E5056`, for a C++ base. No rule compares an Ember override's parameters or result with the method it replaces (the K11 critic's note records the same gap: "needs a rule that an interface implementation's signature must match, which J5's notes found missing"). So `override fn on(e: KeyEvent)` fills `Handler.on(e: Event)`'s slot and `send(log, MouseEvent(x=3))` reads a `MouseEvent` as a `KeyEvent` from safe code: type confusion PHIL-10 forbids. The reader's fix is at the cause and `exactly` is the safe floor; it is narrowed only in wording: an override's first line also carries the words on `this` (`owns this`) and the parameter names, which calls by name use, so the rule should say "the first line of the method it replaces after its name" and list those too, or an `override fn f()` over `virtual fn f() owns this` would hand `this` over through a slot that lends it. The new code `E2113` is free in the registry. Better edits below.

- BETTER EDIT rule=CLS-4
  - old: A method named like an inherited one replaces it (ODR-028): over a virtual method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not;
  - new: A method named like an inherited one replaces it (ODR-028): over a virtual method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not; an `override` has the first line of the method it replaces after its name: the same parameter names, words and types, the same words on `this` and the same result, since a call through the base's slot (`[DSP-2]`) passes what the base declared and reads what it promised, and one that differs is `E2113` ("`KeyLog.on` takes `e: KeyEvent`, and the `Handler.on` it overrides takes `e: Event`; write `override fn on(e: Event):` and test the kind inside: `if e is KeyEvent as k:`");
  - why: CLS-4 is where replacement is decided by name alone; the slot's call passes the base's declared types and words, so the override must take them, and the message gives the kind test (`[DSP-4]`) as the line to write.

- BETTER EDIT rule=registry row E2113
  - old: 
  - new: | `E2113` | an `override` whose first line differs from the virtual method it replaces: a parameter's name, word or type, the words on `this`, or the result | `[CLS-4]` |
  - why: One code, one defining rule.

## NG-31  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: TYP-26: "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types."; FFI-39: "An `override fn` matching no virtual method of the base, by name and signature, is `E5056`."; FFI-48: "| overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |"

**program**: 

- READER EDIT rule=TYP-26
  - old: except operator-interface implementations and `extend` blocks for different types.
  - new: except operator-interface implementations, `extend` blocks for different types, and the `override fn`s of a class derived from a C++ class whose header declares one virtual name several times (`[FFI-39]`): the class writes one `override fn` per inherited declaration, each replacing the declaration its parameters fit, since the header, not the program, made them several.
  - why: Same cause and same edit as NG-27 (the two clusters are one program): the refusal is TYP-26's, whose reason does not hold for overrides the header forces.

- READER EDIT rule=FFI-39
  - old: An `override fn` matching no virtual method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual method of the base, by name and signature, is `E5056`; where the base declares a virtual name several times, each `override fn` of that name replaces the one declaration its parameters fit, one per declaration (`[TYP-26]`), and two overrides fitting one declaration are `E1030`.
  - why: Names the matching rule where the trampoline is built.

**remaining_doubt**: Duplicate of NG-27 (the same program and finding re-tested). Decided safely today (`E1030`); the edit depends on K21's first decision.

**REFUTER reason**: A duplicate of NG-27, with the same program; the same verdict. The text decides it safely (TYP-26: "two functions of one name in one scope are `E1030`"), and the edits presuppose K21's unresolved first decision (one Ember name per C++ name): applied alone they let the class write two `on`s while the base's two imported `on`s stay `E1030` or undecided and an Ember-side call `log.on(k)` has no rule to choose. Carry into K21's revision; apply nothing now.

## NG-32  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-10: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"; FFI-10: "A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`."; FFI-21: "A C function-pointer parameter accepts a capture-free Ember function (as an `extern "C" fn`) or an `extern "C" fn` value."; FFI-21a: "a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`"; FFI-21a: "A value read through the parameter, and a pointer stored in it, are not into the copy."; §IV.5: "Creating one from a reference is safe (ODR-075)"; FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"

**program**: # stream.c (linked): void stream_read(size_t want, void (*get_buf)(size_t n, uint8_t **out)); it calls get_buf(want, &p), then writes `want` bytes at p
unsafe extern "C":  # SAFETY: the declaration matches stream.c
    @ffi(param(get_buf, callback(borrowed)))
    safe fn stream_read(want: usize, get_buf: extern "C" fn(usize, ref mut *mut u8))

fn spare(n: usize, ref mut out: *mut u8):
    x: u8 = 0
    out = ref_to_ptr(ref mut x)     # safe code: a raw pointer keeps no view; written through a `ref mut`, not through a raw pointer

fn main():
    stream_read(4096, spare)
    println("read")

FFI-10's list accepts `get_buf` (a pointer whose contract is complete), FFI-21a refuses only a pointer into the copied parameter that is returned, and the write through `out` is safe code; C writes 4,096 bytes into `spare`'s dead frame. (The text also leaves unsaid whether a hand declaration may write `ref mut` in a callback's type at all; with `*mut *mut u8` the write would be unsafe and vouched, which is one more reason to decide it.)

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`, shown where it is returned and where the type becomes `extern "C" fn`.
  - new: Where a program turns a callable type into `extern "C" fn` (a callback, a field of a `@layout(c)` struct, `[FFI-21]`), declares an export or an `@export_table` field (`[FFI-26]`), or declares an `override fn` of a C++ virtual method (`[FFI-39]`), a function whose value has that type MUST NOT hand foreign code a raw pointer made in safe code: one made from a view (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`, `as_ptr()`), from `null[P]()` or from a cast, passed through copies, conversions and aggregates in the function, that the function returns, itself or inside its result, or writes into a place foreign code passed it (a `ref mut` or `MutSpan` parameter, or a field of one, at any depth) is `E5042`, shown where it is handed back and where the type becomes `extern "C" fn` or the export or override is declared ("`spare` writes `ref_to_ptr(ref mut x)` (line 7) into `out`, which C reads after `spare` has returned; make the pointer inside an `unsafe:` block with a `# SAFETY:` note saying why C may use it, or give C a pointer it gave `spare`"). A pointer made inside `unsafe:` is the programmer's to vouch for (`[UNS-3]` does not report that block); a pointer foreign code gave the function (a parameter, a value read through one) is not followed, nor is one returned from a call to another function; a pointer into a parameter C passes as a copy is one such pointer.
  - why: FFI-21a is the one rule that already limits what a function foreign code runs may hand back, at the point where the function becomes callable from C; the property is the direction (a pointer safe code chose, read by foreign code), not the result slot, and every way C reaches an Ember function (`[FFI-52]`'s list) takes the same limit.

- READER EDIT rule=registry row E5042
  - old: | `E5042` | a function C may call returns a pointer into a parameter C passes as a copy | `[FFI-21a]` |
  - new: | `E5042` | a function foreign code may call (a callback, a `@layout(c)` field, an export, an `@export_table` field, an override of a C++ virtual) hands it a raw pointer made in safe code, by its result or through a `ref mut` or `MutSpan` parameter it was passed, a pointer into a parameter C passes as a copy included | `[FFI-21a]` |
  - why: One code, one defining rule: the row follows the widened rule.

**remaining_doubt**: ODR-075 (making a raw pointer is safe) stays as it is: the limit falls where the pointer is handed to foreign code, and the line to write is the `unsafe:` block around its making, which L3010 must then not report (said in the edit). The same edit answers NG-33, NG-34, NG-38 and NG-41.

**REFUTER reason**: The hole is real but the fix belongs elsewhere. §IV.5: "Creating one from a reference is safe (ODR-075)" and "The raw pointer keeps no compiler-tracked view"; FFI-21a refuses only a pointer returned "into such a parameter" (a copied one), and "A pointer returned from a call to another function is not followed"; FFI-10 accepts the declaration ("a pointer whose contract is complete", the callback with its `callback(borrowed)` word); so `spare` hands C a pointer to its own dead frame and C writes 4,096 bytes there. (Whether a hand-declared callback type may carry `ref mut *mut u8` at all is unsaid, noted under missed; with `*mut *mut u8` the store is already `unsafe` by UNS-1, so the finding stands only on the `ref mut` reading, and the fix below decides both.) The reader's rewrite of FFI-21a is the wrong shape: it tracks a pointer's provenance ("made in safe code" against "made inside `unsafe:`") through copies, conversions and aggregates, a taint analysis the document has nowhere else, and it leaks by its own words, "nor is one returned from a call to another function": `fn forge() -> *mut u8: x: u8 = 0; return ref_to_ptr(ref mut x)` and `return forge()` in the callback passes, as does a pointer read from a `static mut` or a struct field the callback itself filled. The property is the operation, not the pointer's history: giving foreign code a raw pointer is what lets unchecked code use an unchecked address, and UNS-1 already lists that operation for one route, "give an object to C with `into_raw()` (`[OWN-12]`)". One UNS-1 entry covers every route NG-32 to NG-34, NG-36, NG-38 and NG-41 take (a return from a function foreign code may call, a store into a place foreign code reads), is judged by the type at the return or store and by the function's place in `[FFI-52]`'s list, needs no provenance, keeps FFI-21a's outright `E5042` for the one case that is always a bug, and gives the same line to write (the `unsafe:` block with its `# SAFETY:` note), which UNS-3's `L3010` then never reports since the statement needs `unsafe`. Compile time only; no verdict touched.

- BETTER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref mut` or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`);
  - why: UNS-1 is the list of operations that need `unsafe`, and it already holds the one route by which Ember gives C an address (`into_raw()`); the entry states the property (foreign code reads a pointer safe code chose) by the operation, needs no provenance, and reaches a callback, an export, an `@export_table` field, a C++ override and a store through a foreign view alike.

- BETTER EDIT rule=registry row E3100
  - old: | `E3100` | an unsafe operation outside unsafe code (in a `safe:` block included), `into_raw()` included;
  - new: | `E3100` | an unsafe operation outside unsafe code (in a `safe:` block included), `into_raw()` included, a raw pointer returned from a function foreign code may call or stored where foreign code reads it included;
  - why: The row follows the rule.

## NG-33  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-39: "the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly the virtual methods it overrides (`override fn`), read from the class."; FFI-39c: "a parameter borrowed for the call alone takes the object, or `this`, as any plain parameter, upcast to the C++ base (ODR-156)"; FFI-52: "every override of an Ember class derived from a C++ class whose objects C++ may hold (`[FFI-39d]`)"; GRM-35: "a function entry, which asserts the function safe to call"; FFI-1: "for each pointer-typed parameter and result, its ownership, mutability, nullability, count and lifetime"; FFI-21a: "Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter"

**program**: // buffers.hpp: class Source { virtual ~Source(); virtual std::uint8_t* get_buf(std::size_t n) noexcept; /* must give n writable bytes */ };
//               inline void fill_zero(Source& src, std::size_t n) noexcept;   // writes n zeros at src.get_buf(n)
# overlays/buffers.em
unsafe overlay cpp "buffers.hpp":  # SAFETY(ffi): the entry matches its declaration in the header
    fn fill_zero(src, n)
# main.em
import cpp "buffers.hpp" with (overlay = "overlays/buffers.em")

class Spare(cpp.Source):
    override fn get_buf(n: usize) -> *mut u8:    # matches the virtual by name and signature (FFI-39)
        x: u8 = 0
        return ref_to_ptr(ref mut x)             # safe code

fn main():
    s = Spare()
    cpp.fill_zero(ref mut s, 4096)               # safe: `Source&` is `ref mut src`, `n` a scalar, the entry asserts the call safe
    println("filled")

The override is no `extern "C" fn` coercion, so FFI-21a's words never reach it, and the entry has no pointer-typed slot for FFI-1 to ask a fact of. C++ writes 4,096 bytes into `get_buf`'s dead frame.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`, shown where it is returned and where the type becomes `extern "C" fn`.
  - new: Where a program turns a callable type into `extern "C" fn` (a callback, a field of a `@layout(c)` struct, `[FFI-21]`), declares an export or an `@export_table` field (`[FFI-26]`), or declares an `override fn` of a C++ virtual method (`[FFI-39]`), a function whose value has that type MUST NOT hand foreign code a raw pointer made in safe code: one made from a view (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`, `as_ptr()`), from `null[P]()` or from a cast, passed through copies, conversions and aggregates in the function, that the function returns, itself or inside its result, or writes into a place foreign code passed it (a `ref mut` or `MutSpan` parameter, or a field of one, at any depth) is `E5042`, shown where it is handed back and where the type becomes `extern "C" fn` or the export or override is declared ("`spare` writes `ref_to_ptr(ref mut x)` (line 7) into `out`, which C reads after `spare` has returned; make the pointer inside an `unsafe:` block with a `# SAFETY:` note saying why C may use it, or give C a pointer it gave `spare`"). A pointer made inside `unsafe:` is the programmer's to vouch for (`[UNS-3]` does not report that block); a pointer foreign code gave the function (a parameter, a value read through one) is not followed, nor is one returned from a call to another function; a pointer into a parameter C passes as a copy is one such pointer.
  - why: The same edit as NG-32: the override is one of the ways `[FFI-52]` lists for foreign code to run an Ember function, and the limit is placed at the override, not at every entry that takes the class (which would also refuse a `Source` C++ made).

**remaining_doubt**: None beyond NG-32's: the check reads the override's body as `[FFI-21a]` already reads a callback's.

**REFUTER reason**: The hole is real: FFI-39 matches the override "by name and signature" and says nothing of what it returns; FFI-21a's words reach only "Where a program turns a callable type into `extern "C" fn`", which an override is not; the entry `fn fill_zero(src, n)` has no pointer slot for FFI-1's facts (`Source&` is "`ref mut x: T`" under FFI-40); and §IV.5 makes `ref_to_ptr(ref mut x)` safe. C++ writes into `get_buf`'s dead frame from safe code. The reader rightly places the limit at the override and not at every entry that takes the class (which would refuse a `Source` C++ made, the reach the K07 arbiter's E2-3 objected to), but its FFI-21a rewrite has the provenance leaks given under NG-32. The better fix is the same UNS-1 entry: `return ref_to_ptr(ref mut x)` in an `override fn` of a C++ virtual method is an unsafe operation, `E3100` with the block to write, judged by the result's type and FFI-52's list ("every override of an Ember class derived from a C++ class whose objects C++ may hold").

- BETTER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref mut` or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`);
  - why: The same edit as NG-32: the override is one of the functions `[FFI-52]` lists, so its return of a raw pointer is the same operation.

## NG-34  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-10: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"; §IV.5: "`extern "C" fn(i32) -> i32` is a C function pointer: `Copy`, no captures."; FFI-52: "and every Ember function given to C as an `extern "C" fn` value"; FFI-21a: "Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter"

**program**: # hooks.c (linked): `set_hooks` keeps `hooks`; `scratch(n)` calls `hooks.alloc(n)` and writes `n` bytes at the result
@layout(c)
struct Hooks:
    alloc: extern "C" fn(usize) -> *mut u8
    release: extern "C" fn(*mut u8)

unsafe extern "C":  # SAFETY: the declarations match hooks.c
    @ffi(no_callbacks)
    safe fn set_hooks(hooks: Hooks)        # a `@layout(c)` value type: FFI-10's list accepts it
    safe fn scratch(n: usize)

fn spare(n: usize) -> *mut u8:
    x: u8 = 0
    return ref_to_ptr(ref mut x)

fn drop_buf(p: *mut u8):
    pass

fn main():
    set_hooks(Hooks(alloc=spare, release=drop_buf))   # `spare` becomes an `extern "C" fn` here (FN-6)
    scratch(4096)                                      # C calls `spare` and writes 4,096 bytes into its dead frame
    println("parsed")

FFI-21a reaches `spare`'s coercion but refuses only a pointer into the copied parameter; the returned pointer to a local passes. The allocator-hook pattern (cJSON_InitHooks) corrupts the stack from safe code.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`, shown where it is returned and where the type becomes `extern "C" fn`.
  - new: Where a program turns a callable type into `extern "C" fn` (a callback, a field of a `@layout(c)` struct, `[FFI-21]`), declares an export or an `@export_table` field (`[FFI-26]`), or declares an `override fn` of a C++ virtual method (`[FFI-39]`), a function whose value has that type MUST NOT hand foreign code a raw pointer made in safe code: one made from a view (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`, `as_ptr()`), from `null[P]()` or from a cast, passed through copies, conversions and aggregates in the function, that the function returns, itself or inside its result, or writes into a place foreign code passed it (a `ref mut` or `MutSpan` parameter, or a field of one, at any depth) is `E5042`, shown where it is handed back and where the type becomes `extern "C" fn` or the export or override is declared ("`spare` writes `ref_to_ptr(ref mut x)` (line 7) into `out`, which C reads after `spare` has returned; make the pointer inside an `unsafe:` block with a `# SAFETY:` note saying why C may use it, or give C a pointer it gave `spare`"). A pointer made inside `unsafe:` is the programmer's to vouch for (`[UNS-3]` does not report that block); a pointer foreign code gave the function (a parameter, a value read through one) is not followed, nor is one returned from a call to another function; a pointer into a parameter C passes as a copy is one such pointer.
  - why: The same edit as NG-32; the field write `alloc=spare` is already the place where `spare`'s type becomes `extern "C" fn`, so the limit reaches a function given to C inside a struct with no further words.

**remaining_doubt**: None beyond NG-32's.

**REFUTER reason**: The hole is real: FFI-10 accepts `safe fn set_hooks(hooks: Hooks)` ("a `@layout(c)` value type"), FFI-21's `callback(…)` requirement reaches only "A function-pointer parameter of a `safe fn`", not a field of a struct parameter, FN-6 coerces `spare` at `Hooks(alloc=spare, …)` ("A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"), and FFI-21a refuses only a pointer "into such a parameter". C writes 4,096 bytes into `spare`'s dead frame from safe code (the cJSON hook shape). The reader's FFI-21a rewrite reaches the field write, but with the provenance leaks given under NG-32 (`return forge()` passes). The UNS-1 entry decides it at the cause: `return ref_to_ptr(ref mut x)` in a function whose value becomes an `extern "C" fn` is `E3100`, shown at the return and judged by the coercion FFI-21 already uses to check the body ("its body is checked as an export under `threads=any` is").

- BETTER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref mut` or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`);
  - why: The same edit as NG-32.

## NG-35  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-17a: "with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function"; FFI-56: "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)`"; FFI-56: "An overlay may assert one safe to call with `@ffi(safe)` on the typedef's entry (`@ffi(safe) type PFN_log_message`) when every parameter and result is a scalar or a `@layout(c)` value type (`E5002` otherwise)"; FFI-2: "A call to a foreign function whose contract contains an `unknown` fact requires `unsafe`."; UNS-1: "or through a value of an `unsafe` callable type (`[FN-6]`)"

**program**: // dsp.hpp: using RenderFn = std::function<void(float* out, std::size_t frames)>; RenderFn make_silence() noexcept;   // the result writes `frames` zeros at `out`
# overlays/dsp.em
unsafe overlay cpp "dsp.hpp":  # SAFETY(ffi): the entries match their declarations in the header
    @ffi(std_function)
    type RenderFn
    @ffi(no_callbacks)
    fn make_silence()
# main.em
import cpp "dsp.hpp" with (overlay = "overlays/dsp.em") as dsp

fn main():
    x: f32 = 1.0
    render = dsp.make_silence()           # by FFI-17a a value of the safe type `fn(*mut f32, usize)`
    render(ref_to_ptr(ref mut x), 4096)   # safe code: C++ writes 4,096 floats over the 4-byte `x`
    println(x)

Two readings: FFI-17a makes the value a safe `fn(A…) -> R` 'called as any other', while its target is 'a foreign function' whose pointer parameters have no contract (FFI-1, FFI-2 would want `unsafe`). FFI-56 fences the same value when it arrives as a C function pointer and says nothing of a `std::function`. Under the reading the row's type gives, a stack overflow from safe code.

- READER EDIT rule=FFI-17a (the `std::function` row)
  - old: with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function:
  - new: with `@ffi(std_function)` on its overlay entry, a function value of type `unsafe fn(A…) -> R` (`[FN-6]`, `[CLO-3]`), called in `unsafe:` as a C function pointer is (`[FFI-56]`, `E3100`: "`render` is a `std::function` from `dsp.hpp`, with no contract; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe"), or of type `fn(A…) -> R` when the entry is `@ffi(std_function, safe)`, which asserts its target safe to call under `[FFI-56]`'s condition (every `A` and `R` a scalar or a `@layout(c)` value type holding no raw pointer at any depth, `[FFI-10]`; `E5002` otherwise, naming the parameter), an asserted fact `ember tcb` lists; the value owns its target, a foreign function:
  - why: The row is where the value's type is given; a `std::function` from C++ is a function from foreign code exactly as a C function pointer is, so it takes FFI-56's property, and the asserted form reuses FFI-56's word and FFI-10's list.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for one held inside a parameter or result at any depth, as the Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`); and for a value of an `unsafe` callable type held so (a C function pointer, `[FFI-56]`, which safe code may hold from a constant, `[FFI-6a]`, or from C). Each is a value whose target nothing checks and which safe code may set to anything (`ref_to_ptr`, `null[P]()`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry (`[GRM-35]`), `@ffi(safe)` (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`buf_zero`'s `b.data` is a raw pointer, which safe code may point anywhere, so no call of `buf_zero` can be safe; write `unsafe fn buf_zero(b: Buf)` with `@safety(\"`b.data` points at `b.len` writable bytes\")` above it, and build the struct inside an `unsafe:` block of a wrapper that takes `ref mut bytes: MutSpan[u8]`"). A pointer foreign code hands Ember (a result, an `out` value, a callback's parameter) is not refused: reading through it is `unsafe` already (`[UNS-1]`).
  - why: The asserted form of a `std::function` must share the one list every safe-to-call assertion uses, or the `Block`-with-`float*` struct of the finder's program passes it; the list is FFI-10's, so the struct property is stated there once.

**remaining_doubt**: ODR-199 (delegated: `@ffi(std_function)` makes an owned callable value) is kept; only the callable's `unsafe` word changes. A `std::function` with scalar parameters now needs `safe` on its entry to be called outside `unsafe:`, one word per entry.

**REFUTER reason**: The hole is real and the row's own words make it: FFI-17a gives the `std::function` import "a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function", a safe callable CLO-3 lets the program call anywhere, while FFI-56 fences the same thing arriving as a pointer: "A function pointer that comes from C … imports as `unsafe extern "C" fn(…)` … nothing tells the compiler what the function it points to does with its arguments", and FFI-2's "A call to a foreign function whose contract contains an `unknown` fact requires `unsafe`" has no slot to apply to a `std::function`'s parameters. So `render(ref_to_ptr(ref mut x), 4096)` compiles and C++ writes 4,096 floats over four bytes. The reader's first edit is right: the value's type is `unsafe fn(A…) -> R` (FN-6: "`unsafe fn(…) -> R` one whose call is an `unsafe` operation"), with `@ffi(std_function, safe)` asserting the target safe under FFI-56's condition; ODR-199 is kept. Narrowed on the second edit: it is the FFI-10 text proposed for NG-37, which should be applied in the NG-37 better form (without the `unsafe`-callable clause NG-39 refutes, and without the sentence that contradicts FFI-10's own refusal of a raw-pointer result). The better edits restate the row edit unchanged and give the FFI-10 text.

- BETTER EDIT rule=FFI-17a (the `std::function` row)
  - old: with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`) which owns its target, a foreign function:
  - new: with `@ffi(std_function)` on its overlay entry, a function value of type `unsafe fn(A…) -> R` (`[FN-6]`, `[CLO-3]`), called in `unsafe:` as a C function pointer is (`[FFI-56]`, `E3100`: "`render` is a `std::function` from `dsp.hpp`, with no contract; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe"), or of type `fn(A…) -> R` when the entry is `@ffi(std_function, safe)`, which asserts its target safe to call under `[FFI-56]`'s condition (every `A` and `R` a scalar or a `@layout(c)` value type holding no raw pointer at any depth, `[FFI-10]`; `E5002` otherwise, naming the parameter), an asserted fact `ember tcb` lists; the value owns its target, a foreign function:
  - why: The reader's edit, kept: the row is where the value's type is given, and a `std::function` from C++ is foreign code exactly as a C function pointer is.

- BETTER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for a raw pointer held inside a parameter at any depth, as the parameter's Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`). Safe code may set such a field or element to a pointer nothing checks (`null[P]()`, `ref_to_ptr`, `as_ptr()`, §IV.5, `[SPN-8]`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry not written `unsafe fn` (`[GRM-35]`, `[FFI-2]`), `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`count_spaces`'s `s.p` is a raw pointer, which safe code may point anywhere, so no call of `count_spaces` can be safe; write `unsafe fn count_spaces(s)` with `@safety(\"`s.p` points at `s.n` readable bytes\")` above it, and build the `Slice` inside an `unsafe:` block of a wrapper that takes `text: Span[u8]`"). A raw pointer held inside a result or an `out` value, which foreign code set, is not refused: reading through it is `unsafe` already (`[UNS-1]`), and a call that carries it back to foreign code is judged by this sentence where that call's function is declared.
  - why: The one list every safe-to-call assertion uses (K07's property, which its arbiter said holds), stated by what the call lets foreign code read, with the `unsafe`-callable clause dropped (NG-39) and the result sentence made consistent with FFI-10's own refusal of a direct raw-pointer result.

## NG-36  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-11: "`ref mut` and `MutSpan[T]` results allowed only with `exclusive`; a result is `aliased` unless stated"; FFI-11: "A lifetime word asserts two things of the memory the result points into: it lives at least as long as the word says, and nothing writes it while the view may be used"; SPN-8: "`as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is safe, using one needs `unsafe`, and extraction extends no region."; UNS-1: "An `unsafe` context is required to: dereference, read or write through `*T`/`*mut T`; offset a pointer; cast between pointers or pointers and integers; call an `unsafe fn`"; §IV.5: "The raw pointer keeps no compiler-tracked view."

**program**: // engine.h: typedef struct frame_input { float delta_time; const uint8_t *ini_path; uint32_t ini_len; } frame_input;
//           frame_input *engine_input(void);   /* the engine's input block, alive for the whole program */
//           void engine_begin_frame(void);     /* reads the block; saves settings to the `ini_len` bytes at `ini_path` */
# overlays/engine.em
# SAFETY: the entries below match engine.h
unsafe overlay c "engine.h":
    @ffi(no_callbacks)
    fn engine_input() -> borrowed one exclusive from(static)   # a `ref mut frame_input` view of C's block
    @ffi(no_callbacks)
    fn engine_begin_frame()
# main.em
import c "engine.h" with (overlay = "overlays/engine.em") as en

fn set_settings_file():
    name: Array[u8] = [97, 46, 105, 110, 105]
    input = en.engine_input()
    input.ini_path = name.as_span().as_ptr()   # safe code: a field write through a `ref mut` view, of a pointer extracted safely
    input.ini_len = 5

fn main():
    set_settings_file()
    en.engine_begin_frame()                    # C reads 5 bytes at a pointer into a freed Array
    println("frame")

No call carries the pointer to C: it is stored into C's memory through a view C gave, and C reads it later. UNS-1 names writing through a raw pointer, not storing one where foreign code reads.

- READER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); store a raw pointer, or a value holding one at any depth, into memory foreign code holds and reads with no call Ember sees (a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, a field of an `owns T` foreign value, `[FFI-36a]`, or a foreign `static mut`), since code nothing checks reads the pointer there (`E3100`: "`input.ini_path = name.as_span().as_ptr()` (line 7) stores a pointer into C's input block, which C reads later; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying how long the bytes live");
  - why: UNS-1 is the list of what needs `unsafe`; a pointer stored into foreign memory is read by foreign code as surely as one passed to a call, and the two other routes (`[FFI-10]`'s list, `[FFI-21a]`) cannot see a store.

**remaining_doubt**: The finder filed this as a wrong impact report of a proposed change; against the current text the program compiles and C reads freed memory, so it is a hole in its own right. Writing a scalar field of C's block stays safe; only a pointer-holding value takes the block.

**REFUTER reason**: The hole is real and the reader's shape is nearly right. FFI-11's lifetime word makes `engine_input()` a `ref mut frame_input` view of C's block ("`ref mut` and `MutSpan[T]` results allowed only with `exclusive`"), the contract is honest ("it lives at least as long as the word says, and nothing writes it while the view may be used": the view's last use is before `engine_begin_frame()`), SPN-8 makes `as_ptr()` safe and UNS-1 fences only "dereference, read or write through `*T`/`*mut T`", not a store of a pointer value into a place. So C reads five bytes of a freed `Array` with no `unsafe` in the program. The reader puts the entry in UNS-1, which is the cause (the operation that lets unchecked code read an unchecked address), but writes it for stores alone while putting the returns of NG-32 to NG-34, NG-38 and NG-41 in FFI-21a: two mechanisms for one property. The better edit is one UNS-1 entry covering both the store (this cluster: a place reached through a foreign result's `ref mut` view, a foreign `static mut`, a `ref mut` or `MutSpan` parameter of a function foreign code may call) and the return. "a field of an `owns T` foreign value" in the reader's text names nothing the program can write (an opaque class has no fields Ember reaches, FFI-32), so it is dropped; a scalar field written through the view stays safe.

- BETTER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref mut` or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`);
  - why: The same UNS-1 entry as NG-32, which holds the reader's store case and the return cases together.

## NG-37  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-40: "A `T&` parameter imports as `ref mut x: T`, `const T&` as a plain parameter `x: T`"; FFI-1: "for each pointer-typed parameter and result, its ownership, mutability, nullability, count and lifetime"; GRM-35: "a function entry, which asserts the function safe to call"; FFI-10: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"; §IV.5: "`null[P]()` (ODR-076) is safe and returns the null pointer of the exact raw-pointer type `P`"

**program**: // text.hpp: namespace txt { struct Slice { const char* p; std::size_t n; }; std::size_t count_spaces(const Slice& s) noexcept; /* counts ' ' in the n bytes at p */ }
# overlays/text.em
unsafe overlay cpp "text.hpp":  # SAFETY(ffi): the entry matches its declaration in the header
    @ffi(no_callbacks)
    fn count_spaces(s)          # `const Slice&` is a plain `s: Slice`: no pointer slot, no fact asked
# main.em
from std.ffi import c_char
import cpp "text.hpp" with (overlay = "overlays/text.em") as t

fn main():
    s = t.txt.Slice(p=null[*c_char](), n=64)   # safe code
    println(t.txt.count_spaces(s))             # C++ reads 64 bytes at null

The struct's pointer field is what safe code set, and the entry asserts the call safe with no fact about the field; the same for a `Slice` passed by value or behind `borrowed one`.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for one held inside a parameter or result at any depth, as the Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`); and for a value of an `unsafe` callable type held so (a C function pointer, `[FFI-56]`, which safe code may hold from a constant, `[FFI-6a]`, or from C). Each is a value whose target nothing checks and which safe code may set to anything (`ref_to_ptr`, `null[P]()`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry (`[GRM-35]`), `@ffi(safe)` (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`buf_zero`'s `b.data` is a raw pointer, which safe code may point anywhere, so no call of `buf_zero` can be safe; write `unsafe fn buf_zero(b: Buf)` with `@safety(\"`b.data` points at `b.len` writable bytes\")` above it, and build the struct inside an `unsafe:` block of a wrapper that takes `ref mut bytes: MutSpan[u8]`"). A pointer foreign code hands Ember (a result, an `out` value, a callback's parameter) is not refused: reading through it is `unsafe` already (`[UNS-1]`).
  - why: FFI-10's list is the one list every safe-to-call assertion uses (an overlay entry's through FFI-2 and GRM-35, a typedef's through FFI-56); stating the reach by what the call lets foreign code read, however the declaration passes it, decides by-value, pointer and C++-reference passing alike.

**remaining_doubt**: This is the K07 question (J8-08) itself, which the current text does not yet contain; the edit is the K07 recommendation (A) written by the property rather than by a list of passing modes. The specification's own XVI.4 Vulkan overlay (`pCreateInfo: borrowed one`, a struct with `pNext`) is then refused and must be rewritten with K46's answer, as K07's notes say.

**REFUTER reason**: The hole is real and is J8-08 (K07), whose arbiter found "The core of the change holds" and sent it back only for a sentence that over-reached. The quotes decide it: FFI-40 makes `const Slice&` "a plain parameter `x: T`", FFI-10's list accepts "a `@layout(c)` value type", FFI-1's contract covers "each pointer-typed parameter and result" and so never a field, FFI-30a says an overlay "never changes a type's identity, layout or fields", and §IV.5 makes `null[P]()` safe; so `count_spaces(Slice(p=null[*c_char](), n=64))` is a safe call that has C++ read 64 bytes at null. The reader's edit is K07's recommendation (A) written by the property at FFI-10, where the existing sentence for the direct case already stands, and names the three spellings as the arbiter asked. Narrowed on two points: (1) the clause for "a value of an `unsafe` callable type held so" refuses hook tables for nothing, since safe code cannot make a C function pointer value (NG-39), and is dropped; (2) its last sentence, "A pointer foreign code hands Ember (a result, …) is not refused", contradicts the sentence before it, "complete facts do not make a raw-pointer parameter or result safe to expose", so it must say a pointer held inside a result or an `out` value. The XVI.4 Vulkan entry (`pCreateInfo: borrowed one`, line 8363) is then refused and must be rewritten with K46's answer, as the reader says; that is the trade-off written down.

- BETTER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for a raw pointer held inside a parameter at any depth, as the parameter's Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`). Safe code may set such a field or element to a pointer nothing checks (`null[P]()`, `ref_to_ptr`, `as_ptr()`, §IV.5, `[SPN-8]`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry not written `unsafe fn` (`[GRM-35]`, `[FFI-2]`), `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`count_spaces`'s `s.p` is a raw pointer, which safe code may point anywhere, so no call of `count_spaces` can be safe; write `unsafe fn count_spaces(s)` with `@safety(\"`s.p` points at `s.n` readable bytes\")` above it, and build the `Slice` inside an `unsafe:` block of a wrapper that takes `text: Span[u8]`"). A raw pointer held inside a result or an `out` value, which foreign code set, is not refused: reading through it is `unsafe` already (`[UNS-1]`), and a call that carries it back to foreign code is judged by this sentence where that call's function is declared.
  - why: The reader's property, kept at FFI-10 beside the sentence it generalises, with the needless `unsafe`-callable clause dropped and the result sentence made consistent.

## NG-38  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-31b: "`@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may."; FFI-26: "with a `# SAFETY:` note above saying why nothing else defines that name"; FFI-21a: "Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter"; SPN-8: "`as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is safe, using one needs `unsafe`, and extraction extends no region."

**program**: @layout(c)
struct Reply:
    text: *u8
    len: usize

# SAFETY: the host defines no other symbol named `plugin_reply`
@unsafe(export("plugin_reply"))
pub extern "C" fn reply() -> Reply:        # a `@layout(c)` value: FFI-31b allows it in an export
    buf: Array[u8] = [111, 107]
    return Reply(text=buf.as_span().as_ptr(), len=2)   # safe code; `buf` is freed at the return

fn main():
    println("built")

// host.c: struct Reply r = plugin_reply(); fwrite(r.text, 1, r.len, stdout);   /* reads 2 bytes of a freed Array */

An export is a function C calls whose result reaches C by the same route as a callback's, and FFI-21a's words reach only a type turned into `extern "C" fn`; the export's `# SAFETY:` note vouches for the symbol's name only.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`, shown where it is returned and where the type becomes `extern "C" fn`.
  - new: Where a program turns a callable type into `extern "C" fn` (a callback, a field of a `@layout(c)` struct, `[FFI-21]`), declares an export or an `@export_table` field (`[FFI-26]`), or declares an `override fn` of a C++ virtual method (`[FFI-39]`), a function whose value has that type MUST NOT hand foreign code a raw pointer made in safe code: one made from a view (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`, `as_ptr()`), from `null[P]()` or from a cast, passed through copies, conversions and aggregates in the function, that the function returns, itself or inside its result, or writes into a place foreign code passed it (a `ref mut` or `MutSpan` parameter, or a field of one, at any depth) is `E5042`, shown where it is handed back and where the type becomes `extern "C" fn` or the export or override is declared ("`spare` writes `ref_to_ptr(ref mut x)` (line 7) into `out`, which C reads after `spare` has returned; make the pointer inside an `unsafe:` block with a `# SAFETY:` note saying why C may use it, or give C a pointer it gave `spare`"). A pointer made inside `unsafe:` is the programmer's to vouch for (`[UNS-3]` does not report that block); a pointer foreign code gave the function (a parameter, a value read through one) is not followed, nor is one returned from a call to another function; a pointer into a parameter C passes as a copy is one such pointer.
  - why: The same edit as NG-32: exports and `@export_table` fields are named among the places the limit is placed, so one property covers every function foreign code may call (`[FFI-52]`'s list).

**remaining_doubt**: FFI-31b's list of what may appear in an export stays: a `@layout(c)` value with a pointer field may still be returned when the pointer is made inside `unsafe:` with its note (a pointer into a `static`, or one C gave).

**REFUTER reason**: The gap is real though the promise it breaks is the weaker one: FFI-31b admits the result ("`@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may") and refuses views only ("a result that is a view (`E5015`), whose lifetime nothing states"), FFI-21a reaches only "Where a program turns a callable type into `extern "C" fn`", FFI-26's note vouches for the name alone ("saying why nothing else defines that name"), and SPN-8 makes `as_ptr()` safe; so the export hands the host a pointer into a freed `Array` from safe code, while UNS-1 already makes the other way of giving C an address, `into_raw()`, an unsafe operation. The reader's FFI-21a rewrite has the provenance leaks given under NG-32. The UNS-1 entry decides it: a `return` of a value holding a raw pointer from an export is `E3100`, with the `unsafe:` block and its `# SAFETY:` note as the line to write, so a pointer into a `static` or one C gave is still returnable, vouched, and FFI-31b's list stays as it is.

- BETTER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref mut` or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`);
  - why: The same edit as NG-32: an export is among the functions `[FFI-52]` lists.

## NG-39  reader=not_answered  refuter=refuted  should_be=not_a_defect  safety=none

**answered_by**: FFI-56: "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)`"; FFI-6a: "A null-pointer or pointer-cast macro (`((void*)0)`, `((HANDLE)-1)`) is a `const` of that pointer type holding the address as an integer"; UNS-1: "or through a value of an `unsafe` callable type (`[FN-6]`)"; FFI-10: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"; GRM-35: "a function entry, which asserts the function safe to call"

**program**: // timer.h: typedef void (*tick_fn)(int ms); #define TICK_ROM_HANDLER ((tick_fn)0x1FFF0000)
//          typedef struct timer_spec { tick_fn fire; int ms; } timer_spec; void timer_run(timer_spec spec);   /* calls spec.fire(spec.ms) */
# overlays/timer.em
# SAFETY: the entry below matches timer.h
unsafe overlay c "timer.h":
    fn timer_run(spec)                   # a `@layout(c)` struct with no pointer slot: no fact unknown
# main.em
import c "timer.h" with (overlay = "overlays/timer.em") as tm

fn main():
    spec = tm.timer_spec(fire=tm.TICK_ROM_HANDLER, ms=16)   # a `const` of type `unsafe extern "C" fn(c_int)` (FFI-6a, FFI-56); holding it is safe (UNS-1 fences calls only)
    tm.timer_run(spec)                                       # C calls address 0x1FFF0000
    println("ran")

A forged (or, with `fire=None` on a nullable field, a null) C function pointer reaches C from safe code through a struct a safe entry accepts: the same property as a raw pointer (nothing checks its target).

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for one held inside a parameter or result at any depth, as the Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`); and for a value of an `unsafe` callable type held so (a C function pointer, `[FFI-56]`, which safe code may hold from a constant, `[FFI-6a]`, or from C). Each is a value whose target nothing checks and which safe code may set to anything (`ref_to_ptr`, `null[P]()`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry (`[GRM-35]`), `@ffi(safe)` (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`buf_zero`'s `b.data` is a raw pointer, which safe code may point anywhere, so no call of `buf_zero` can be safe; write `unsafe fn buf_zero(b: Buf)` with `@safety(\"`b.data` points at `b.len` writable bytes\")` above it, and build the struct inside an `unsafe:` block of a wrapper that takes `ref mut bytes: MutSpan[u8]`"). A pointer foreign code hands Ember (a result, an `out` value, a callback's parameter) is not refused: reading through it is `unsafe` already (`[UNS-1]`).
  - why: The same edit as NG-37, which names a value of an `unsafe` callable type beside a raw pointer: both are values whose target nothing checks; a safe `extern "C" fn` made from an Ember function is checked and stays allowed, its hand-backs judged by `[FFI-21a]`.

**remaining_doubt**: A binding that must fill a C hook table with a C-provided function does so in a wrapper's `unsafe:` block, one block per table, which is the cost of the refusal.

**REFUTER reason**: Safe code cannot make the value the finding needs. A C function pointer value reaches safe code only as C or the header gave it: FFI-6a ("a `const` of that pointer type holding the address as an integer") and FFI-56 ("A function pointer that comes from C … imports as `unsafe extern "C" fn(…)`"), or as a checked Ember function (FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"). It cannot be forged: TYP-7 "`as` between pointer types, or between a pointer and an integer, requires `unsafe`", and `null[P]()` takes a raw-pointer type ("`P` must be a raw-pointer type"), not a callable type. So `TICK_ROM_HANDLER` is the header's own address, and C calling it is "a bug in foreign code reached through a correctly declared boundary" or the header's intent (PHIL-11), not a pointer safe code chose; and `fire=None` is writable only where the field's type is `Option`, which the importer gives only "when nullable". That differs from a raw pointer, which `null[P]()`, `ref_to_ptr` and `as_ptr()` make in safe code. The reader's clause for "a value of an `unsafe` callable type held so" would refuse every safe entry taking a hook table filled with C's or Ember's own functions (`VkAllocationCallbacks`, SDL, cJSON) and buy nothing; it is dropped from the FFI-10 edit (NG-37). What the cluster does show is left unsaid: how an imported struct's function-pointer field becomes `Option` (listed under missed).

## NG-40  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FFI-17a: "| `std::vector<T>` | `CppVector[T]` with `.span()`, `.span_mut()`, `.to_array()`, `push_back`, `len` |"; FFI-40: "`const T&` as a plain parameter `x: T`"; §IV.5: "`null[P]()` (ODR-076) is safe and returns the null pointer of the exact raw-pointer type `P`"; GRM-35: "a function entry, which asserts the function safe to call"; FFI-10: "`safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"

**program**: // labels.hpp: namespace ui { std::vector<const char*> default_names() noexcept; void print_all(const std::vector<const char*>& names) noexcept; /* std::puts each name */ }
# overlays/labels.em
# SAFETY: the entries below match labels.hpp
unsafe overlay cpp "labels.hpp":
    @ffi(no_callbacks)
    fn default_names()
    @ffi(no_callbacks)
    fn print_all(names)                 # `const std::vector<const char*>&` is a plain `names: CppVector[*c_char]`
# main.em
from std.ffi import c_char
import cpp "labels.hpp" with (project = "ui", overlay = "overlays/labels.em") as u

fn main():
    names = u.ui.default_names()
    names.push_back(null[*c_char]())    # safe code: `push_back` is a bridge operation, `null[P]()` is safe
    u.ui.print_all(names)               # C++ calls std::puts(nullptr)
    println("printed")

Nothing judges the elements of a bridge container whose type argument is a raw pointer; the entry asserts the call safe with no fact about them.

- READER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for one held inside a parameter or result at any depth, as the Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`); and for a value of an `unsafe` callable type held so (a C function pointer, `[FFI-56]`, which safe code may hold from a constant, `[FFI-6a]`, or from C). Each is a value whose target nothing checks and which safe code may set to anything (`ref_to_ptr`, `null[P]()`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry (`[GRM-35]`), `@ffi(safe)` (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`buf_zero`'s `b.data` is a raw pointer, which safe code may point anywhere, so no call of `buf_zero` can be safe; write `unsafe fn buf_zero(b: Buf)` with `@safety(\"`b.data` points at `b.len` writable bytes\")` above it, and build the struct inside an `unsafe:` block of a wrapper that takes `ref mut bytes: MutSpan[u8]`"). A pointer foreign code hands Ember (a result, an `out` value, a callback's parameter) is not refused: reading through it is `unsafe` already (`[UNS-1]`).
  - why: The same edit as NG-37: the reach is drawn by what safe code could have set, as the Ember type declares it, which for a bridge container is its type argument (`CppVector[*c_char]`, `Atomic[*mut T]`, `Option[*T]`), and not by whether a type declares fields; an opaque class's members and `CppString`'s buffer, which only foreign code sets, are not reached.

**remaining_doubt**: A `CppVector[Mesh]` of a pointer-free struct stays accepted, so the common case keeps its safe entry.

**REFUTER reason**: The hole is real: FFI-17a gives `CppVector[T]` "with `.span()`, `.span_mut()`, `.to_array()`, `push_back`, `len`" as safe bridge operations, §IV.5 makes `null[P]()` safe, FFI-40 makes the `const std::vector<const char*>&` a plain `names: CppVector[*c_char]`, and the entry asserts the call safe with no fact about the elements (FFI-1's facts attach to "pointer-typed parameter and result" only). C++ calls `puts(nullptr)` from safe code. The reader's FFI-10 edit reaches it through "an element of … a bridge type (`CppVector[T]`, `[FFI-17a]`)", and a `CppVector[Mesh]` of a pointer-free struct keeps its safe entry. Narrowed only to apply the FFI-10 text in NG-37's better form (the `unsafe`-callable clause dropped, the result sentence made consistent).

- BETTER EDIT rule=FFI-10
  - old: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`.
  - new: A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. The same holds for a raw pointer held inside a parameter at any depth, as the parameter's Ember type declares it: a field of a `@layout(c)` value type, an element of a fixed array or of a bridge type (`CppVector[T]`, `[FFI-17a]`), the payload of an `Option`, the pointee of a contracted pointer, a value passed by a C++ reference (`[FFI-40]`). Safe code may set such a field or element to a pointer nothing checks (`null[P]()`, `ref_to_ptr`, `as_ptr()`, §IV.5, `[SPN-8]`), so no declaration asserts a call that carries one to foreign code safe: `safe fn`, an overlay function entry not written `unsafe fn` (`[GRM-35]`, `[FFI-2]`), `@ffi(safe)` on a function-pointer typedef (`[FFI-56]`) and `@ffi(std_function, safe)` (`[FFI-17a]`) with one are `E5002`, naming the field or element ("`count_spaces`'s `s.p` is a raw pointer, which safe code may point anywhere, so no call of `count_spaces` can be safe; write `unsafe fn count_spaces(s)` with `@safety(\"`s.p` points at `s.n` readable bytes\")` above it, and build the `Slice` inside an `unsafe:` block of a wrapper that takes `text: Span[u8]`"). A raw pointer held inside a result or an `out` value, which foreign code set, is not refused: reading through it is `unsafe` already (`[UNS-1]`), and a call that carries it back to foreign code is judged by this sentence where that call's function is declared.
  - why: The same FFI-10 text as NG-37.

## NG-41  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: FN-6: "A callable type is written `fn(A, ref mut B, owns C) -> R`, with the same parameter words as a declaration."; FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"; FFI-21: "A C function-pointer parameter accepts a capture-free Ember function (as an `extern "C" fn`) or an `extern "C" fn` value."; FFI-21a: "a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`"; FFI-31b: "`ref mut x: T` as `T*`"

**program**: @layout(c)
struct BufDesc:
    data: *mut u8      # the decoder writes `len` bytes here
    len: usize

unsafe extern "C":
    @ffi(param(get_buf, callback(borrowed)))
    safe fn decode_frame(get_buf: extern "C" fn(ref mut BufDesc))   # a `ref mut` parameter in a callback type, lowered as an export's is (FFI-31b: `T*`)

fn fill(ref mut d: BufDesc):
    x: u8 = 0
    d.data = ref_to_ptr(ref mut x)   # safe: a pointer value stored through a `ref mut`, no write through a raw pointer
    d.len = 4096

fn main():
    decode_frame(fill)
    println("decoded")

`fill` returns nothing, so FFI-21a's words (a returned pointer into the copied parameter) never reach it; C writes 4,096 bytes into `fill`'s dead frame (FFmpeg's `get_buffer2` shape). Whether an `extern "C" fn` type may carry `ref mut` is itself unsaid; FFI-31b lowers one for an export, and FFI-21 checks a callback's body 'as an export under `threads=any`', so the reading that accepts it is the natural one.

- READER EDIT rule=FFI-21a
  - old: Where a program turns a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw pointer into such a parameter: one made from a view of it or of a part of it (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and aggregates in the function, is `E5042`, shown where it is returned and where the type becomes `extern "C" fn`.
  - new: Where a program turns a callable type into `extern "C" fn` (a callback, a field of a `@layout(c)` struct, `[FFI-21]`), declares an export or an `@export_table` field (`[FFI-26]`), or declares an `override fn` of a C++ virtual method (`[FFI-39]`), a function whose value has that type MUST NOT hand foreign code a raw pointer made in safe code: one made from a view (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`, `as_ptr()`), from `null[P]()` or from a cast, passed through copies, conversions and aggregates in the function, that the function returns, itself or inside its result, or writes into a place foreign code passed it (a `ref mut` or `MutSpan` parameter, or a field of one, at any depth) is `E5042`, shown where it is handed back and where the type becomes `extern "C" fn` or the export or override is declared ("`spare` writes `ref_to_ptr(ref mut x)` (line 7) into `out`, which C reads after `spare` has returned; make the pointer inside an `unsafe:` block with a `# SAFETY:` note saying why C may use it, or give C a pointer it gave `spare`"). A pointer made inside `unsafe:` is the programmer's to vouch for (`[UNS-3]` does not report that block); a pointer foreign code gave the function (a parameter, a value read through one) is not followed, nor is one returned from a call to another function; a pointer into a parameter C passes as a copy is one such pointer.
  - why: The same edit as NG-32: the limit is stated by direction (what the function leaves in a place C passed it, a `ref mut` parameter's field included), not by the result slot.

**remaining_doubt**: None beyond NG-32's.

**REFUTER reason**: The hole is real on the reading that accepts `ref mut BufDesc` in a callback type (unsaid; FFI-21 says only "its body is checked as an export under `threads=any` is" and FFI-31b lowers "`ref mut x: T` as `T*`" for exports), and under it `d.data = ref_to_ptr(ref mut x)` is a safe store (UNS-1 fences a write "through `*T`/`*mut T`", not a store of a pointer value) that FFI-21a never sees (`fill` returns nothing). C writes 4,096 bytes into `fill`'s dead frame (FFmpeg's `get_buffer2` shape). The reader's FFI-21a rewrite has the provenance leaks given under NG-32; the UNS-1 entry decides it by the operation: a store of a raw pointer into "a `ref mut` or `MutSpan` parameter of such a function, or a field of one" is `E3100` with the block to write. Whether an `extern "C" fn` type may carry `ref mut` at all is listed under missed.

- BETTER EDIT rule=UNS-1
  - old: access a foreign `static mut` (`[STA-1]`);
  - new: access a foreign `static mut` (`[STA-1]`); give foreign code a raw pointer, as `into_raw()` does: return a raw pointer, or a value holding one at any depth, from a function foreign code may call (one whose value becomes an `extern "C" fn`, `[FN-6]`, `[FFI-21]`; an export or an `@export_table` field's function, `[FFI-26]`; an `override fn` of a C++ virtual method, `[FFI-39]`: the functions `[FFI-52]` lists), or store one into a place foreign code reads with no call Ember sees (a `ref mut` or `MutSpan` parameter of such a function, or a field of one; a place reached through the `ref mut` view a foreign result's lifetime word gives, `[FFI-11]`, `[FFI-40]`; a foreign `static mut`), since safe code may make a pointer that is null, dangling or shorter than a count beside it (§IV.5, `[SPN-8]`) and code nothing checks reads it there (`E3100`: "`spare` returns `ref_to_ptr(ref mut x)` (line 7) to C, which uses the pointer after `spare` has returned; put line 7 in an `unsafe:` block with a `# SAFETY:` note saying why C may use it"); a pointer into a parameter C passes as a copy stays refused outright (`E5042`, `[FFI-21a]`);
  - why: The same edit as NG-32.

## NG-42  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: THR-2: "A line that makes a value `thread_shared` (`t = thread_shared Tree(items=[item])`, `[THR-1]`), and a store under a lock into a field of one (`t.items.push(owns item)`, `[THR-18]`), makes every class object the stored value owns reachable from any thread, and is such a hand-over for each of them: the compare is made there, so a `weakref T` link made before it (`w = weakref item`) finds its object ended afterwards, and no plain weak count is ever touched from two threads."; THR-2: "The walk follows the handed value's fields and elements by their types, through `owns` blocks, `Option`s and collections"; THR-2: "A value whose layout the handing package cannot see, an object held `dyn I`, an object of an `open` or `abstract` class (`[CLS-4]`), or a function value's state, is walked by the **hand-over routine** its own package compiled beside its `drop` and kept in its `TypeInfo`"; ERR-8: "a `dyn Error` held alone (`owns dyn Error`) plus an optional chain of context messages"; OBJ-1 (as the X.4 row states it): "a type that can be named from outside its package, or that a `weakref` type in its own package names"; WK-12: "`else:` runs when the link holds nothing or its target is ended (a target handed to another thread is ended here, `[THR-2]`)"

**program**: 

**remaining_doubt**: Line 31 (`log = thread_shared Log(results=Mutex([fail(owns e)]))`) is 'a line that makes a value `thread_shared`', so THR-2's compare runs there for the `LoadError` object reached through `Mutex` → `Array` → `Result` → `AnyError`'s `owns dyn Error` (walked by the routine for a `dyn I`); `w` then finds it ended and the program prints `done`, as the finder's property asks. `LoadError` carries the weak count since `weakref e` in its own package names it (`[OBJ-1]`). Nothing left open.

**REFUTER reason**: The text decides it, safely. THR-2: "A line that makes a value `thread_shared` (`t = thread_shared Tree(items=[item])`, `[THR-1]`), and a store under a lock into a field of one (`t.items.push(owns item)`, `[THR-18]`), makes every class object the stored value owns reachable from any thread, and is such a hand-over for each of them: the compare is made there, so a `weakref T` link made before it (`w = weakref item`) finds its object ended afterwards, and no plain weak count is ever touched from two threads." Line 31, `log = thread_shared Log(results=Mutex([fail(owns e)]))`, is such a line; the walk "follows the handed value's fields and elements by their types, through `owns` blocks, `Option`s and collections" to `AnyError`'s "`owns dyn Error`" (ERR-8), and "an object held `dyn I` … is walked by the hand-over routine its own package compiled beside its `drop`". `LoadError` carries the weak count by OBJ-1 ("or when a `weakref` type in the package names it"), so the compare runs, the old block is marked ended, WK-12's `else:` path runs ("a target handed to another thread is ended here, `[THR-2]`") and the program prints `done`. The finder tested a proposed change's cost line, not the current text, which already lists the `thread_shared` line among the hand-overs.

## NG-43  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: THR-15: "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`), so a one-thread owner never crosses through a channel: an end of a channel of such a `T` given to another thread is `E7015` at that line, as any value that is not `Send` (`[THR-2]`)."; THR-8: "nothing reachable from it (its fields, its elements, what its `owns` parts hold, a function value's state) is a one-thread owner (`shares T`, `shares mut T`)"; THR-10: "so a function that holds a one-thread value is `E7015` (`[THR-1]`, `[THR-2]`)"

**program**: 

**remaining_doubt**: J7-07 is closed in the current text (ruling N80): `Receiver[shares Doc]` is not `Send`, so `thread.spawn(owns fn() owns rx: consume(rx))` is `E7015` at that line. The finder's complaint was about a change's impact report; the text now says what the report claimed.

**REFUTER reason**: Decided by THR-15 as rewritten under ruling N80: "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`), so a one-thread owner never crosses through a channel: an end of a channel of such a `T` given to another thread is `E7015` at that line, as any value that is not `Send` (`[THR-2]`)." `shares Doc` is a one-thread owner (THR-8: "nothing reachable from it … is a one-thread owner (`shares T`, `shares mut T`)"), so `Receiver[shares Doc]` is not `Send` and `thread.spawn(owns fn() owns rx: consume(rx))` is `E7015` at that line. The finder's complaint was about a change's impact column quoting the old THR-15; the current text says what the column claimed. The refusal is safe.

## NG-44  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: THR-15: "`Sender[T]` and `Receiver[T]` are `Send`, and `Sender[T]` is `ThreadShared`, exactly when `T` is `Send` (`[THR-8]`)"; ERR-8: "a `dyn Error` held alone (`owns dyn Error`) plus an optional chain of context messages"; THR-8: "an `owns T` when `T` is"; THR-8: "`dyn I` when `I` requires it (`interface I: needs Send:`)"; ERR-3: "`Error` is the interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`)"; ERR-9: "The error parameter defaults to `AnyError` (§XIII.2), so `fn load(path: StrView) -> Result[Texture]` is a function that can fail with any error."; THR-10: "so a function that holds a one-thread value is `E7015` (`[THR-1]`, `[THR-2]`), and `R` MUST be `Send` (`E7015`, `E7008`)"

**program**: 

**remaining_doubt**: The three report statements the finder corrected are moot against the current text, which decides the program one way: `Result[int]` is `Result[int, AnyError]` (ERR-9), `AnyError` holds an `owns dyn Error`, `Error` requires neither marker, so `AnyError` is not `Send` (THR-8), `Sender[Result[int]]` is not `Send` (THR-15), and `thread.spawn(owns fn() owns tx: produce(tx))` is `E7015`; THR-8 and THR-15 no longer clash. The refusal is safe but needless for an ordinary program; that is K20's open question (whether `Error` requires `Send` and `ThreadShared`), outside this cluster.

**REFUTER reason**: The current text decides the program one way, and safely: ERR-9 "The error parameter defaults to `AnyError`", ERR-8 "a `dyn Error` held alone (`owns dyn Error`)", THR-8 "`dyn I` when `I` requires it (`interface I: needs Send:`)" and ERR-3's `Error` as "`Display + Debug` with `fn source() -> Option[ref dyn Error]`", which requires no marker, so `AnyError` is not `Send`; THR-15 then makes `Sender[Result[int]]` not `Send`, and THR-10's "a function that holds a one-thread value is `E7015`" refuses `thread.spawn(owns fn() owns tx: produce(tx))`. The clash the finder quoted between the old THR-15 and THR-8 is gone (N80). The refusal is safe; that it is needless for an ordinary program is K20's open question (whether `Error` requires the markers), outside this cluster, and K20's record itself depends on NG-25's hole being closed first.

## NG-45  reader=not_a_defect  refuter=confirmed  should_be=not_a_defect  safety=none

**answered_by**: TXT (std.string parse): "`ParseError`, in `std.string`, is a unit-only enum, `Empty`, `Invalid` or `Overflow`, naming the first problem from the left."; ERR-4: "`unwrap` and `expect` on a `Result[T, E]` need `E: needs Display` (`E2040` otherwise, whose help is to implement `Display`) and panic with the error's `Display` text"; FN-8: "An `Err` returned from `main` is printed with `Display` to standard error and the process exits with status 1."; DRV-1: "| `Display` | on a unit-only enum, the variant name; on a struct or payload enum, the `Debug` text |"

**program**: 

**remaining_doubt**: The finding is about two tests of a proposed change asserting std's message text (`the text is not a value of this type`, `std/src/string.em`), not about the specification: the specification fixes `ParseError`'s variants and how an `Err` from `main` is printed, and leaves a library error's `Display` text to the library, as it does for every std error but `called unwrap on None`. The test-authoring advice stands (assert a code, or a text the program itself fixes, `AnyError.msg(...)` or an `@error("…")` of its own); it needs no rule. Should the lead want std's texts fixed by the document, that is a std-reference question for every error type, not this one.

**REFUTER reason**: Not a defect of the specification. TXT-10 fixes `ParseError`'s shape only: "`ParseError`, in `std.string`, is a unit-only enum, `Empty`, `Invalid` or `Overflow`, naming the first problem from the left"; FN-8 fixes how `main`'s error is printed ("An `Err` returned from `main` is printed with `Display` to standard error and the process exits with status 1"); DRV-1's `Display` row applies to a derived `Display`, and nothing says `ParseError`'s is derived. The text `the text is not a value of this type` is std's (`std/src/string.em`), which the document leaves to the library as it does every std error text but ERR-4's `called unwrap on None`. The finder's advice (assert a code, or a text the program itself fixes through `AnyError.msg(...)` or an `@error("…")` of its own) needs no rule; fixing std's texts in the document would be a std-reference question for every error type, not this one.

## NG-46  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: RC-4 point 3: "at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends"; RC-4 point 5: "**So a value's end is delayed only** when its last owner ends on a thread that did not make it and the value was not handed over at a place of point 4 (it travelled inside another object): it is ended at its making thread's next check point. Until then it is intact"; RC-4 point 2: "The making thread empties its queue at each of its check points"; PHIL-11: "What remains possible, and is therefore not a defect of the guarantee: resource exhaustion (a panic, `[ALC-4]`), a count reaching its limit included (`[RC-1]`); deadlock;"

**program**: 

- READER EDIT rule=RC-4 point 3
  - old: a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line.
  - new: a channel's receive, a sleep, a job worker's wait for work, `[JOB-2]`) and when the thread ends. A thread waiting in such a call stays at the point before it: a value put in its queue while it waits wakes it, once, on the ending thread's out-of-line path (§X.4); it empties its queue there, where the queued drops were already judged as the point's own (`[THR-19]`, `[THR-20]`), and waits again, so a value another thread let go of is never held for as long as its maker waits. A thread waiting inside a foreign call (`[FFI-52]`) is not woken: its next point is the one after the call. The one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line.
  - why: Point 3 is where the hidden ends are placed; the wait is at a point already placed, so the wake adds no place and changes no judgment, and one property covers every blocking wait (the job worker's included) instead of one case.

- READER EDIT rule=§X.4 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point (`[RC-4]`), the queue's work for a value whose last owner ends on another thread without a hand-over, and one wake of the making thread, on the ending thread's out-of-line path, when the maker waits at a check point (`[RC-4]` point 3).
  - why: `[PHIL-15]`: every cost the language inserts has a row.

**cost_row**: §X.4 row "A `thread_shared` value's two counts": one wake of the making thread, on the ending thread's out-of-line queue path, when the maker waits at a check point.

**remaining_doubt**: Decided, and safely: the current text says plainly that the end is delayed to the maker's next check point and the value stays intact (points 3 and 5), so the reader's `Buffer` is freed when `stop.recv()` returns, and the program's own spin on `FREED` is what hangs it (PHIL-11 lists deadlock outside the guarantee). The finder expected a wake, which the text has for no kind of wait (the job-worker wake it reviewed is K10's design, not in the text). The edit is K10's recommendation (A) generalised; it is a run-time addition with a cost row, so the lead may prefer to carry it with K10 rather than alone. Verdict 101 ('the making thread deletes at its next check point ... always before the thread ends') is kept: the wake runs the queue at a placed point.

**REFUTER reason**: The status is right: the current text decides the program, memory-safely. RC-4 point 3 places check points "at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends", and point 5 says plainly "it is ended at its making thread's next check point. Until then it is intact"; so the `Buffer` is freed when `stop.recv()` returns, and the program's own spin on `FREED` is what hangs it, a deadlock PHIL-11 lists outside the guarantee. The finder measured the text against the K10 design's wake for job workers, which is in no rule of the current text. The reader's edits should not be applied alone: (1) their safety premise, "where the queued drops were already judged as the point's own (`[THR-19]`, `[THR-20]`)", is not in the text (THR-19 and THR-20 say nothing of a check point's queued drops; that judgment is exactly what the K10 design adds and what its arbiter sent back over E3-1), so the edit imports one piece of a rejected design without the piece that makes drops at a point safe to run; (2) it is a run-time addition that changes the shape of every blocking call (a wait that the queue can interrupt, a loop of wait, empty, wait again) beyond the one wake its §X.4 row names; (3) K10's own decision chose the wake for job workers only, and widening it is K10's revision to weigh. Verdict 101 ("That thread deletes it at its next check point") is consistent with either. Carry the finding into K10; apply nothing here.

## NG-47  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: RC-4 point 3: "at every loop's back edge, before and after every call that may block the thread (waiting for a lock, a `join`, the end of a `with scope` block, a channel's receive, a sleep) and when the thread ends"; RC-4 point 5: "**So a value's end is delayed only** when its last owner ends on a thread that did not make it and the value was not handed over at a place of point 4 (it travelled inside another object): it is ended at its making thread's next check point. Until then it is intact"; THR-10: "`h.join() -> R` waits and returns the result."; PHIL-11: "What remains possible, and is therefore not a defect of the guarantee: resource exhaustion (a panic, `[ALC-4]`), a count reaching its limit included (`[RC-1]`); deadlock;"

**program**: 

- READER EDIT rule=RC-4 point 3
  - old: a channel's receive, a sleep) and when the thread ends; the one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line.
  - new: a channel's receive, a sleep, a job worker's wait for work, `[JOB-2]`) and when the thread ends. A thread waiting in such a call stays at the point before it: a value put in its queue while it waits wakes it, once, on the ending thread's out-of-line path (§X.4); it empties its queue there, where the queued drops were already judged as the point's own (`[THR-19]`, `[THR-20]`), and waits again, so a value another thread let go of is never held for as long as its maker waits. A thread waiting inside a foreign call (`[FFI-52]`) is not woken: its next point is the one after the call. The one after a `join` or a scope's end makes a value a joined task ended on its own thread end before the parent's next line.
  - why: The same edit as NG-46: main waiting in `h.join()` is one instance of a thread waiting at a placed point; with it the `Session` is ended during the join, `CLOSED` becomes 1, the server returns and the program prints `1 1`.

- READER EDIT rule=§X.4 row "A `thread_shared` value's two counts"
  - old: a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.
  - new: a test of the thread's queue at each check point (`[RC-4]`), the queue's work for a value whose last owner ends on another thread without a hand-over, and one wake of the making thread, on the ending thread's out-of-line path, when the maker waits at a check point (`[RC-4]` point 3).
  - why: `[PHIL-15]`: every cost the language inserts has a row.

**cost_row**: §X.4 row "A `thread_shared` value's two counts": one wake of the making thread, on the ending thread's out-of-line queue path, when the maker waits at a check point.

**remaining_doubt**: Same as NG-46: the text decides the delay (points 3 and 5), memory-safely; the hang is the program's own spin on a destructor the text says is delayed. The finder's 'special case' (a wake for job workers only) is not in the current text at all, so against it the two clusters are one question: whether any waiting thread is woken. Verdict 101 is kept by the edit; the K10 design's own decision chose the wake for job workers, and the general form costs the same one wake.

**REFUTER reason**: The same as NG-46. RC-4 points 3 and 5 decide the program: main's `Session`, which "travelled inside another object", "is ended at its making thread's next check point. Until then it is intact", and main's next point is after `h.join()` ("before and after every call that may block the thread (… a `join` …)"), so `CLOSED` stays 0 while `serve` spins on it: the program's own deadlock (PHIL-11). The finder's "special case" is K10's design, absent from the text. The reader's wake edits are not to be applied alone, for the three reasons given under NG-46 (a safety premise the text does not hold, a run-time reshaping of every blocking call beyond the named cost, and K10's own scope decision); their general form (every waiting thread woken) is the right question for K10's revision, where its E3-1 objection (which thread's function counts the last drops) must be settled with it.

# MISSED

## MISSED: The parameter words an `extern "C" fn` type may carry are unsaid

**rules**: FN-6: "A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe"; FFI-21: "its body is checked as an export under `threads=any` is (`[FN-6]`, `[FFI-33]`)"; FFI-31b: "An export's parameters and result are written in the safe types of `[FFI-10]`'s mapping and lowered to the same carriers: … `ref mut x: T` as `T*`". FFI-31b names the set and the carriers for an export's signature only; FN-6 says "FFI-safe" without naming the set; FFI-21 maps a callback's parameters from the C pointer type "a raw pointer among them staying raw", which a hand declaration has no C type to map from. So NG-32's `extern "C" fn(usize, ref mut *mut u8)` and NG-41's `extern "C" fn(ref mut BufDesc)` are accepted on one reading (lowered as an export's `T*`) and `E2020` on another, and two compilers disagree.

**program**:
```
@layout(c)
struct Desc:
    len: usize

unsafe extern "C":  # SAFETY: the declaration matches dec.c
    @ffi(param(cb, callback(borrowed)))
    safe fn run(cb: extern "C" fn(ref mut Desc))

fn fill(ref mut d: Desc):
    d.len = 1

fn main():
    run(fill)        # accepted (lowered as an export's `T*`) or `E2020` (`ref mut` is no FFI-safe type): the text does not say
```

- MISSED EDIT rule=FN-6
  - old: A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe;
  - new: A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe: its parameters and result are those `[FFI-31b]` admits in an export's signature, written with the same words and lowered to the same carriers (`ref mut x: T` as `T*`, `Span[T]` as a pointer and a `size_t`), and any other is `E2020`, naming the parameter;
  - why: FN-6 is where the coercion is granted; naming the set once by the export rule that already lists it decides every hand-declared callback type and every `@layout(c)` function-pointer field the same way.

## MISSED: How an imported struct's function-pointer field becomes `Option`

**rules**: FFI-56: "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)` (`Option[…]` when nullable, `[FN-6]`)"; FFI-30a: an overlay "never changes a type's identity, layout or fields"; FFI-11: the nullability word attaches to contracted pointer slots. No contract reaches a struct field, so "when nullable" names a fact no declaration can state for a field: an implementer cannot decide whether `timer_spec.fire` is `unsafe extern "C" fn(c_int)` (safe code can never store null; C's own null is a value invalid for its type, `[UNS-4]`) or `Option[unsafe extern "C" fn(c_int)]` (safe code may store `None`, and the struct reaches C through a safe entry).

**program**:
```
// timer.h: typedef void (*tick_fn)(int ms); typedef struct timer_spec { tick_fn fire; int ms; } timer_spec; void timer_run(timer_spec spec);  /* calls spec.fire if non-null */
unsafe overlay c "timer.h":  # SAFETY: the entry matches timer.h
    fn timer_run(spec)
import c "timer.h" with (overlay = "overlays/timer.em") as tm

fn main():
    spec = tm.timer_spec(fire=None, ms=16)   # compiles if the field is an `Option`; `E2020` if it is not: the text does not say which
```

- MISSED EDIT rule=FFI-56
  - old: imports as `unsafe extern "C" fn(…)` (`Option[…]` when nullable, `[FN-6]`)
  - new: imports as `unsafe extern "C" fn(…)` (`Option[…]` when its slot's contract says `nullable`, `[FFI-11]`; a field of an imported struct, which no contract reaches (`[FFI-30a]`), is always the `Option`, since C may leave it null and safe code must be able to read what C wrote)
  - why: The one place the nullable form is granted; a field has no contract, so the rule must decide it itself, and the `Option` is the only form that keeps `[UNS-4]`'s 'every value is valid for its type' true for a struct C filled.

## MISSED: The move a `T&&` override makes is a thunk operation `ember inspect` should list

**rules**: FFI-17f: "`ember inspect` reports each bridge operation that is a thunk call rather than an inlined one (`CppShared` clone and drop, `CppVector` `push_back`, `len` and drop, `CppString` drop, a `std_function` value's drop, and the conversion of a `std::optional` or `std::variant` at a call)"; PHIL-15: "Every cost the language can insert without the programmer writing it … has a row in §X.4 and is reported per site by `ember inspect`". With NG-29's edit the trampoline runs `T`'s move constructor before an `override fn take(owns p: T)` runs, and the thunk runs `T`'s destructor on the moved-from object after a call from Ember to a `T&&` parameter; neither is in FFI-17f's list.

**program**:
```
class Counter(jb.jobs.Sink):
    bytes: c_int = 0
    override fn take(owns p: jb.jobs.Payload):   # one move constructor of `Payload` per call, inserted by the trampoline
        this.bytes += p.size()
```

- MISSED EDIT rule=FFI-17f
  - old: a `std_function` value's drop, and the conversion of a `std::optional` or `std::variant` at a call).
  - new: a `std_function` value's drop, the conversion of a `std::optional` or `std::variant` at a call, the move constructor a trampoline runs for a `T&&` parameter of an `override fn` and the destructor of the moved-from object after a call from Ember to a `T&&` parameter (`[FFI-40]`)).
  - why: PHIL-15: a cost the language inserts is reported per site; this one is new with NG-29's edit.

## MISSED: An interface implementation's method need not have the interface's first line

**rules**: IFC-1: "`extend T:` without `implements` adds inherent methods to `T`"; IFC-3: "`interface Ord: needs Eq` requires implementers of `Ord` to implement `Eq`"; IFC-4: an associated type "with a value that meets them (`E2040` if it is missing or does not)"; DSP-3: "Each entry of an interface's table holds, beside the implementation's code, the fact `[LT-22]` records for it, in the width the interface's declaration fixes". No rule says an `extend T implements I` writes each method with the parameter words, types and result `I` declares (the K11 critic's note records the gap: "needs a rule that an interface implementation's signature must match, which J5's notes found missing"); a call through `dyn I` passes what the interface declared into code that reads something else, the same confusion as NG-30 for `virtual`.

**program**:
```
interface Scaler:
    fn scale(k: f64) -> f64

struct Sq:
    side: f64

extend Sq implements Scaler:
    fn scale(k: i64) -> f64:          # not the interface's first line; no rule refuses it
        return this.side * (k as f64)

fn apply(s: dyn Scaler) -> f64:
    return s.scale(2.5)               # the table's entry runs `Sq.scale` reading the `f64` bits as an `i64`

fn main():
    println(apply(Sq(side=2.0)))
```

- MISSED EDIT rule=IFC-3
  - old: 
  - new: An implementation writes each of the interface's methods with the interface's first line after the name: the same parameter names, words and types, the same words on `this` and the same result, since a call through `dyn I` (`[DSP-3]`) passes what the interface declared; one that differs is `E2040`, naming the method and the interface's line ("`Sq.scale` takes `k: i64`, and `Scaler.scale` takes `k: f64`; write `fn scale(k: f64) -> f64:`").
  - why: The same hole as NG-30 for the second table of `[LT-48]`; IFC-3 is where what an implementation promises its interface is stated, so the sentence goes there once for every interface (the lead may place it where `extend T implements I` is checked, if that rule has another id).
