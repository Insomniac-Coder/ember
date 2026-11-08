# R8-part-XVI-b

could_not_cover: Nothing in the seven clusters was left unread. Not re-derived here: K21's answer on C++ overload pairs, which K47's edit assumes keeps constness as the distinguishing mark (as `[FFI-42]`'s `get()`/`get_mut()` does), and K51's spelling of `ref_to_ptr`, which K52 relies on as §IV.5 now states it (one built-in whose result follows the argument's word).

## K47-overlay-operator-names  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: GRM-35: "overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")" ["->" contract {contract}] NEWLINE"; GRM-35: "An `overlay_class` section holds the entries of a C++ class's members (verdict 131): its methods, which list no receiver, its constructors (`new`) and its static functions, each written as any other entry"; FFI-42: "Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to read-through: such a class imports `get()`, and `get_mut()` where the operator is not `const`, through which `->` is reached); C++ iterators with `begin`/`end` become `Iterable`."; FFI-40: "a `T&` or `const T&` result is a pointer result (`[FFI-2a]`): its lifetime is never derived, and the overlay's entry gives it a lifetime word, `from(this)`, `from(p)` or `from(static)` (`[FFI-11]`), under which it is a `ref mut T` or `ref T` view into what the word names (`[LT-1a]`); without one the method stays unsafe to call, its result a `*mut T` or `*T`"; FFI-12: "An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`, so overlays cannot drift."; FFI-2: "A header function with no overlay entry therefore stays unsafe to call."; Part IV §8 (interface Index): "interface Index[Idx]:
    type Output
    fn index(i: Idx) -> ref Output"

**program**: // grid.hpp
// class Grid { public: float& operator[](size_t i); const float& operator[](size_t i) const; size_t size() const; };

# overlays/grid.em
unsafe overlay cpp "grid.hpp":
    class Grid:
        @ffi(const)
        fn size()
        fn operator[](i)        # not an IDENT: refused by [GRM-35]; no other spelling exists

# main.em
import cpp "grid.hpp" with (project="sim", overlay="overlays/grid.em") as sim

fn sample(g: sim.Grid) -> f32:
    return g[3]

The implementer's question the text cannot answer: `[FFI-42]` says `operator[]` maps to `Index`/`IndexMut`, whose `index` returns `ref Output`; `[FFI-40]` says a `float&` result is a `*mut f32` and the method unsafe to call until an overlay entry gives a lifetime word; `[GRM-35]` gives an entry only the form `fn IDENT(...)` and `[FFI-12]` refuses an entry whose name does not match the header. So no entry can name `operator[]`, and whether `g[3]` is accepted as `Index.index` (a safe `ref f32` from an unchecked C++ read), refused as `E3100`, or something else, is undecided.

- READER EDIT rule=FFI-42
  - old: Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to read-through: such a class imports `get()`, and `get_mut()` where the operator is not `const`, through which `->` is reached); C++ iterators with `begin`/`end` become `Iterable`.
  - new: Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to read-through: such a class imports `get()`, and `get_mut()` where the operator is not `const`, through which `->` is reached); C++ iterators with `begin`/`end` become `Iterable`. An operator is a member of its class as any method is (`[FFI-40]`): an overlay's `class C:` section names it by the Ember method it becomes (the `const` `operator[]` is `index` and the other `index_mut`; `get` and `get_mut`; the method names of the operator interfaces of §IV.8), and that name matches the header's operator for `[FFI-12]`. A `T&` result of an operator takes its lifetime word in that entry (`fn index(i) -> borrowed one from(this)`, `fn index_mut(i) -> borrowed one exclusive from(this)`), and the entry asserts, as every entry does, that a call within its contract cannot break memory safety (`[FFI-2]`): that the operator checks its index is the asserter's fact (`[TCB-1]`). Without an entry the operator stays unsafe to call, as every member does (`[FFI-2]`, `[FFI-40]`), and the operator's syntax on the imported class outside `unsafe:` is `E3100`: "`g[3]` calls `Grid`'s `operator[]`, a C++ reference result whose target no contract names; put line 5 in an `unsafe:` block with a `# SAFETY:` note, or give the operator an entry in the overlay: under `class Grid:` write `@ffi(const)` then `fn index(i) -> borrowed one from(this)` for the `const` one, and `fn index_mut(i) -> borrowed one exclusive from(this)` for the other".
  - why: The cause is `[FFI-42]`, which maps the operator to an interface but names no entry for it; `[GRM-35]` already accepts `index` and `index_mut` as IDENTs, so the grammar needs no change, and `[FFI-12]`'s match-the-header rule is told which header member an Ember name matches. The property is general: every operator is named by the method it becomes, the result lifetime follows `[FFI-40]`, and no operator is ever safe without an entry (`[FFI-2]`, verdict 125).

**remaining_doubt**: The const and non-const `operator[]` are an overload pair (K21's question); this edit assumes K21 keeps constness as the thing that tells them apart, which `[FFI-42]`'s `get()`/`get_mut()` already does. Whether a C++ `operator[]` that takes several index types (several non-const overloads) needs K21's rename form is K21's to decide.

**REFUTER reason**: The hole is real. `[FFI-42]`: "Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to `Index`/`IndexMut` ...)", and §IV.8 gives `Index` "fn index(i: Idx) -> ref Output", a safe view; `[FFI-40]`: "a `T&` or `const T&` result is a pointer result (`[FFI-2a]`): its lifetime is never derived, and the overlay's entry gives it a lifetime word ... without one the method stays unsafe to call, its result a `*mut T` or `*T` ... and a call of it outside `unsafe:` is `E3100`"; `[GRM-35]`: "overlay_fn := ["unsafe"] "fn" IDENT ..."; `[FFI-12]`: "An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`". So one reading (FFI-42 as a derived mapping) hands safe code a `ref f32` from an unchecked C++ read, the other (FFI-40) leaves every operator unsafe for ever because no entry can name `operator[]`; an implementer cannot decide `g[3]`, and the compiler's own ruling that already names operators by their Ember method (decisions.md row for FFI-40a (132): "the free operator functions C++ declares outside the class and Ember imports as the class's methods (`operator+(const Money&, const Money&)` as `Money.add`): their entries need `@ffi(const)` like any method's") is in no rule of CURRENT-SPEC. K21 chose b (constness picks the overload), so the reader's `index`/`index_mut` split holds. The reader's edit is at the cause and respects verdicts 125, 131 and 132, but two things are off: (1) its `E3100` message says "put line 5 in an `unsafe:` block", which makes `g[3]` inside `unsafe:` yield a raw pointer, while `a[i]` goes only through `Index`/`IndexMut`/`IndexSet` (ODR-042, Part IV table "`a[i]` | `Index` / `IndexMut` / `IndexSet`") whose `index` returns `ref Output`, never `*T`: without an entry the class implements no `Index`, and the unsafe route is the method by name (`g.index(3) -> *f32`), as `[FFI-40]`'s `child.parent()` is; (2) `IndexMut` "needs Index[Idx]", so a class whose only `operator[]` is not `const` needs saying (both names from the one entry, the read through a changeable name C++ allows). The edit adds no run-time cost (the thunk is the same call) and departs from no verdict.

- BETTER EDIT rule=FFI-42
  - old: 
  - new: An operator, a member or a free function whose first operand is the class, is a method of the imported class as any member is (`[FFI-40]`, verdict 131): an overlay's `class C:` section names it by the method of the interface it maps to (§IV.8: the `const` `operator[]` is `index` and the other `index_mut`, `operator+` is `add`, `operator*`/`->` are `get` and `get_mut`), and that name matches the header's operator for `[FFI-12]`; a class whose only `operator[]` is not `const` has it as `index_mut`, and `index` with it from the one entry, since `IndexMut` needs `Index` and C++ reads through it. A `T&` or `const T&` result takes its lifetime word in that entry as `[FFI-40]` says (`@ffi(const)` then `fn index(i) -> borrowed one from(this)`; `fn index_mut(i) -> borrowed one exclusive from(this)`), and the entry asserts, as every entry does, that a call within its contract cannot break memory safety (`[FFI-2]`): that the operator checks its index is the asserter's fact (`[TCB-1]`). The class implements the interface once its methods have entries; until then it implements none (`[TYP-40]`), the operator is reached only as a method of that name inside `unsafe:`, its reference result a `*mut T` or `*T` (`[FFI-40]`), and the operator's syntax on the class is `E3100`: "`g[3]` calls `Grid`'s `operator[]`, a C++ reference result whose target no contract names; call `g.index(3)` in an `unsafe:` block with a `# SAFETY:` note, or give the operator an entry in the overlay: under `class Grid:` write `@ffi(const)` then `fn index(i) -> borrowed one from(this)` for the `const` one, and `fn index_mut(i) -> borrowed one exclusive from(this)` for the other".
  - why: Keeps the reader's property (an operator is named by the method it becomes, unsafe until its entry) and removes the conflict with ODR-042 and `Index.index -> ref Output`: the unsafe route is the method by name, as `[FFI-40]`'s own example, and the interface exists only once its entries do, as `[TYP-40]` asks of every implementation. The one-operator class is said once, generally.

## K50-adopt-ends-by-own-destructor  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: FFI-54: "`std.ffi.from_raw[C](p: ObjectPtr[C]) -> C`, and `from_raw[T](p: *mut T) -> owns T` for an `owns T` given so, is an `unsafe fn` (`@safety("`p` is an address `into_raw()` gave, taken back once")`) that makes the value owned again, by one name, so its end frees it as before; a class object comes back with one owner, so a `weakref` link made before `into_raw()` still finds it alive (`[UNS-4]`); taking one address back twice is caught as adopting twice is (`[FFI-36b]`: at run time in every profile, and `E5068` on one local)."; FFI-54: "A C host ends an object given with `into_raw()` with `ember_rt_object_release(void*)` (`[FFI-27]`), on an attached thread, which does what its last owner's end would"; FFI-23: "C gives a foreign object back through `adopt` inside `unsafe:` (`[FFI-36a]`). An `ObjectPtr[C]` is the address of a class object, which `into_raw()` gives, with no count of its own (to C, one pointer): `Copy`, equal when it is the address of the same object."; FFI-36: "A C++ function returning a raw pointer with no ownership contract yields `*mut T`; `ffi.adopt[T](p)` (`unsafe`) turns it into an `owns T` ended through the contract's destructor."; FFI-36a: "`std.ffi.adopt[T](h) -> owns T` is an `unsafe fn` that takes ownership of a foreign object, reading its destructor from the contract of `T`'s declaration, `@ffi(destroys(f))` on `type T`; the `owns T` value's end calls `f` once."; FFI-32: "A function returning an opaque class by value gives an `owns T`, as a constructor does: the result is moved to an object of its own, ended by the class's destructor; an abstract class, or one without a public destructor, is refused (ODR-166)."; FFI-17a: "| `std::unique_ptr<T>` | `owns T`, ended through the deleter; one whose deleter holds state is refused (`E5034`): an `owns T` holds one address |"; FFI-17: "generates one thunk per imported function, method, constructor and destructor"

**program**: J8-13 is decided (see remaining_doubt). J8-16 is not:

// factory.hpp: class Enemy { public: ~Enemy(); int hp; };  Enemy* make_enemy(int kind);   // caller deletes
import cpp "factory.hpp" with (project="game") as f

fn spawn() -> owns f.Enemy:
    # SAFETY: make_enemy returns a new Enemy the caller deletes, and nobody else holds it
    unsafe:
        p = f.make_enemy(3)?
        return std.ffi.adopt[f.Enemy](p)

The implementer's question: `[FFI-36]` ends the adopted `owns f.Enemy` "through the contract's destructor" and `[FFI-36a]` reads that destructor from `@ffi(destroys(f))` on `type T`; an automatically imported C++ class (`[FFI-44]`: "other classes | automatic, opaque behind a pointer") has no such entry and no `type` line, while `[FFI-32]` and `[FFI-17a]` end the same class's by-value results and `unique_ptr`s by the class's destructor and the deleter. Whether `adopt` here is `E5002` (no `destroys`), or ends the enemy with `delete` through a generated thunk, is undecided; both readings are consistent with the text.

- READER EDIT rule=FFI-36
  - old: yields `*mut T`; `ffi.adopt[T](p)` (`unsafe`) turns it into an `owns T` ended through the contract's destructor.
  - new: yields `*mut T`; `ffi.adopt[T](p)` (`unsafe`) turns it into an `owns T` ended through the contract's destructor: `@ffi(destroys(f))` on the class's overlay entry where one is written, else the class's own destructor and its deallocation function, `delete` in a thunk the importer generates (`[FFI-17]`), as `[FFI-32]` ends a by-value result and `[FFI-17a]` a `std::unique_ptr` with the default deleter; that the pointer was made with `new` and is held by nothing else is the `@safety` of the `adopt` (`[UNS-7]`). A class with no public destructor, or an abstract one, is refused as `[FFI-32]` refuses it (`E5002`: "`Enemy` has no public destructor, so nothing Ember can call ends an adopted one; name a free function in the overlay: `@ffi(destroys(destroy_enemy))` above `type Enemy`").
  - why: The cause is `[FFI-36]`'s "the contract's destructor" naming a word an automatically imported C++ class never has. The property stated is the one `[FFI-32]` and `[FFI-17a]` already use for the same classes: a C++ class's own declaration says what ends it, and a written `destroys(f)` says otherwise.

- READER EDIT rule=FFI-36a
  - old: reading its destructor from the contract of `T`'s declaration, `@ffi(destroys(f))` on `type T`; the `owns T` value's end calls `f` once.
  - new: reading its destructor from the contract of `T`'s declaration, `@ffi(destroys(f))` on `type T`, or, for an imported C++ class with no such entry, the class's own destructor and `delete` (`[FFI-36]`); the `owns T` value's end calls `f` once.
  - why: So the defining rule of `adopt` and the Annex C rule say one thing; `[FFI-36a]`'s "a foreign type has one destructor" keeps holding (a written `destroys(f)` replaces `delete`, and two written ones stay `E5002`).

**remaining_doubt**: J8-13 is answered, differently from the finder's recommendation A (reuse `adopt`): `[FFI-54]` adds `std.ffi.from_raw[C](p: ObjectPtr[C]) -> C` and `from_raw[T](p: *mut T) -> owns T`, an `unsafe fn` with the twice-taken check of `[FFI-36b]` (`E5068`, a run-time panic), and `ember_rt_object_release` for a C host; the user-data pattern now frees its object and a `weakref` made before `into_raw()` still finds it. That is safe and complete; no verdict asked for one word. The only loose end is the registry: `E5068`'s row names `from_raw`, so nothing is missing there. For J8-16 the recommended `delete` route rests on the adopter's `@safety`, as every `adopt` does; the alternative (require `destroys(f)` for C++ as for C) is safe too but makes every caller-deletes C++ factory need a free function written in C++, which `[FFI-32]` does not ask of by-value results.

**REFUTER reason**: J8-13 is answered as the reader says: `[FFI-54]`: "`std.ffi.from_raw[C](p: ObjectPtr[C]) -> C`, and `from_raw[T](p: *mut T) -> owns T` for an `owns T` given so, is an `unsafe fn` ... taking one address back twice is caught as adopting twice is (`[FFI-36b]`: at run time in every profile, and `E5068` on one local)", and `E5068`'s row names `from_raw`. J8-16 is undecided: `[FFI-36]` "turns it into an `owns T` ended through the contract's destructor"; `[FFI-36a]` "reading its destructor from the contract of `T`'s declaration, `@ffi(destroys(f))` on `type T`"; `[FFI-44]` "other classes | automatic, opaque behind a pointer; methods through thunks" and "ownership the header does not state | unsafe until `adopt` or a contract (`[FFI-36]`)", so `adopt` is the route for exactly the class that has no `type T` entry, and `[FFI-17]` "generates one thunk per imported function, method, constructor and destructor" gives the destructor a thunk nothing names as `adopt`'s. `E5002` and `delete` are both consistent readings. The reader's `delete` default is the right one (what `[FFI-17a]` does for `std::unique_ptr<T>`: "`owns T`, ended through the deleter") and costs nothing new (a thunk call `[FFI-17f]` already reports). Three corrections: (1) "an abstract one" is refused for no reason of its own: `[FFI-32]` refuses an abstract class because a by-value result cannot be moved into an object, while `delete` through a public virtual destructor of an abstract base is exactly how C++ ends such objects; (2) `[FFI-32]`'s by-value result is in a block Ember made and is "ended by the class's destructor", not by `delete`, so the parallel is `[FFI-17a]`'s deleter, not `[FFI-32]`'s deallocation; (3) the new `E5002` case needs its registry row ("one code, one defining rule").

- BETTER EDIT rule=FFI-36
  - old: contract's destructor.
  - new: contract's destructor: `@ffi(destroys(f))` on the class's overlay entry where one is written, else, for an imported C++ class, its own destructor and deallocation, `delete` in a thunk the importer generates (`[FFI-17]`), as a `std::unique_ptr<T>` with the default deleter ends it (`[FFI-17a]`); that the pointer is to an object made with `new` that nothing else ends is what the `adopt`'s `@safety` asks of its caller (`[UNS-7]`). A class with no public destructor is refused (`E5002`: "`Enemy` has no public destructor, so nothing Ember can call ends an adopted one; name the function that does in the overlay: `@ffi(destroys(destroy_enemy))` above `type Enemy`").
  - why: At the cause (`[FFI-36]`'s "the contract's destructor" naming a word an automatic import never has), with the rule the text already uses for the same class's `unique_ptr`; the abstract class is not refused, since `delete` through its public destructor is C++'s own way, and the by-value parallel is kept accurate.

- BETTER EDIT rule=FFI-36a
  - old: `@ffi(destroys(f))` on `type T`; the `owns T` value's end calls `f` once.
  - new: `@ffi(destroys(f))` on `type T`, or, for an imported C++ class with no such entry, the class's own destructor and `delete` (`[FFI-36]`); the `owns T` value's end calls `f` once.
  - why: The defining rule and the Annex C rule say one thing; "a foreign type has one destructor" keeps holding, a written `destroys(f)` replacing `delete`.

- BETTER EDIT rule=registry row E5002
  - old: or two destructors for one foreign type (a call of a function without a verified contract outside `unsafe` is `E3100`);
  - new: or two destructors for one foreign type, or `adopt` of a C++ class with no public destructor (a call of a function without a verified contract outside `unsafe` is `E3100`);
  - why: Every case a code has is in its row.

## K52-c-keeps-address-ember-owns  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: FFI-11: "for a callback, `callback(retained)` (`[FFI-21]`), and on any other pointer no safe type, since C keeps Ember memory only when it is handed over (`owns`) or given with `into_raw()` (`[FFI-23]`, `[FFI-54]`): the call stays `unsafe`, with a raw pointer"; FFI-10: "A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`."; §IV.5: "`ref_to_ptr(ref x) -> *T` and `ref_to_ptr(ref mut x) -> *mut T` are one function the compiler knows, not two of one name (`[TYP-26]`): its argument is a view written with its word (a plain parameter, `ref x`, or `ref mut x`), and the result's type follows that word."; FFI-35a: "A class object given to a `retained` parameter is marked in its header as kept by C, the top bit of its type index (`[OBJ-1]`): the mark is set by the call, one store, by the wrapper of a function with a parameter contracted `retained` whose C type is a pointer to a class type (`*mut Names`), through the pointer before the call, since a class object's address is its header (`[OBJ-2]`), and that the pointer is a live object's is the unsafe caller's obligation (`[UNS-4]`), as every raw pointer's is; a `retained` pointer to anything else marks nothing. A marked object never moves: handed to another thread while a `weakref` link on this thread points at it, it stops the program (`[THR-2]`, verdict 114)."; THR-2: "An object whose address C keeps (`[FFI-35a]`) never moves: a link to it at the hand-over stops the program ("`names` goes to another thread while `w`, a `weakref` link on this thread, points at it, and C keeps its address (`register(names)`, line 4), so it cannot move; end the link first"; UNS-4: "A raw pointer made from a view is not used after the view's last use, nor after what it points into is reshaped or ended, nor after a hand-over of its object to another thread, which may move it (`[THR-2]`)."; OBJ-1 (via decisions.md N87): "The kept-by-C mark is the top bit of the type index, set by the call's wrapper through a `retained` pointer to a class type; a type index is below 2^31."

**program**: 

**remaining_doubt**: Decided, but not with the finder's spelling: the address is `ref_to_ptr(ref mut names)`, a `*mut Names` (§IV.5), given to an `unsafe fn register(p: *mut Names)` declared `@ffi(param(p, retained, one))` inside `unsafe:` with a `# SAFETY:` note; the finder's `safe fn register(p: Names)` is `E5002` (`[FFI-10]`, `[FFI-11]`), and no `ObjectPtr` is involved (that type is `into_raw()`'s, `[FFI-23]`). The generated wrapper sets the mark through the pointer, the owner stays with Ember, and the hand-over stop of verdict 114 has a program it applies to; a `weakref` cleared first lets the object go at the same address, which the SAFETY note must cover (`[UNS-4]`). Safe. One editorial looseness: `[THR-2]`'s message shows `register(names)` as the line C keeps the address from, which under `[FFI-11]` must be an Ember wrapper around the unsafe call or a shortened example; no implementer decision hangs on it.

**REFUTER reason**: The text decides it, and safely. `[FFI-11]`'s ownership row: `retained` on "any other pointer no safe type, since C keeps Ember memory only when it is handed over (`owns`) or given with `into_raw()` ...: the call stays `unsafe`, with a raw pointer"; `[FFI-10]`: "complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`" and "An unsafe raw-pointer declaration keeps its written C ABI parameters"; §IV.5: "`ref_to_ptr(ref x) -> *T` and `ref_to_ptr(ref mut x) -> *mut T` are one function the compiler knows"; `[FFI-35a]`: "the mark is set by the call, one store, by the wrapper of a function with a parameter contracted `retained` whose C type is a pointer to a class type (`*mut Names`), through the pointer before the call, since a class object's address is its header (`[OBJ-2]`), and that the pointer is a live object's is the unsafe caller's obligation (`[UNS-4]`)". So the spelling is `ref_to_ptr(ref mut names)` into an `unsafe fn register(p: *mut Names)` declared `@ffi(param(p, retained, one))`, inside `unsafe:` with a `# SAFETY:` note; the finder's `safe fn register(p: Names)` is `E5002`. The owner stays with Ember, the mark makes verdict 114's stop reachable (`[THR-2]`: "An object whose address C keeps (`[FFI-35a]`) never moves: a link to it at the hand-over stops the program"), and the object's end while C still holds the address is the SAFETY note's to exclude, as for every raw pointer (`[UNS-4]`). Nothing runs beyond the one store the `[COST-3]`-covered wrapper makes. The reader's editorial point stands: `[THR-2]`'s message shows `register(names)` as the line C keeps the address from, which under `[FFI-11]` cannot be a direct safe call of the C function; listed under missed with the line to write.

## K53-once-callback-token  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: FFI-21: "A function-pointer parameter of a `safe fn` or of an overlay entry carries a `callback(…)` word, or the declaration is `E5002` ("`set_timer`'s `cb` is a function pointer, and nothing says whether C keeps it or which thread runs it; write `param(cb, callback(retained, threads(any)))`, or `callback(borrowed)` if C runs it only during the call"); an `unsafe fn` may take one with no word."; FFI-21: "`retained` and `once` return a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the callback through the declared `destroys(f)` (`E5041` if none is declared, for `once` too, since a token that ends before C runs the function must unregister it); with `once` the trampoline ends the block after the first call, and the token, which keeps the block until C has run the function or the token ends, whichever is first, records that the call ran: a token ending before the call unregisters the callback through `destroys(f)` and ends the block, and one ending after it calls nothing, since C gave the registration up by running it, and frees the slot's record."; FFI-53: "a `callback(once)` slot carries a used-up mark, taken with one atomic exchange before any of the callback's state is read, so of two threads calling it at once one runs it and the other stops the program with the second-call text, and the block is ended when the call that ran it returns."; FFI-23: "A token is bound to the thread that registered the callback, whatever the callback's contract (`[THR-8]`: neither `Send` nor `ThreadShared`; handing it to another thread is `E7008`"; FFI-11: "`destroys(f)` (`f`'s parameters may be the registering call's own and its result, `id = listen(…)` then `unlisten(id)`: the token keeps them)"; E5041 registry row: "| `E5041` | a `retained` or `once` callback needs a release function (`destroys(f)`) in the overlay | `[FFI-21]` |"

**program**: 

- READER EDIT rule=FFI-21
  - old: `retained` and `once` return a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the callback through the declared `destroys(f)`
  - new: `retained` and `once` return a `Retained[ForeignCallback]` token (`[FFI-23]`), the call's only result: a result of the C function's own (a timer or listener id) is kept in the token for `destroys(f)` (`[FFI-11]`) and reaches the program by no other route, since a program holding the id could unregister with it while the token lives and C would then be asked to unregister twice; a registering function whose own result the program needs is declared `unsafe fn`. The token's end unregisters the callback through the declared `destroys(f)`
  - why: Adjacent to the cluster's question (the finder's "result `c_int` or a token?"): the text gives the token and lets `destroys(f)` take the C result, but does not say whether the program also receives it. Handing the id out would let safe code make C unregister twice, which only the asserted `destroys(f)` contract stands behind, so the token alone is the safe answer; the property is general (one route to C's unregister).

**remaining_doubt**: Decided as the finder's option A (a token and `E5041` for `once` too), not the recommended C (a token only where `destroys` is written). A is the safe one: a library with a one-shot callback and no cancel function cannot be declared `safe fn` with `callback(once)` and is written `unsafe fn` with no word, which the text allows. The `atexit` case changed since the checker's reading: a `safe fn` with a function-pointer parameter now needs a `callback(…)` word (`E5002`), and `retained` with no `destroys` is `E5041`, so `atexit` is declared `unsafe fn atexit(func: extern "C" fn()) -> c_int` and called inside `unsafe:`; a capture-free function still passes as an `extern "C" fn` whose C entry attaches the thread (`[FFI-21]`, `[FN-6]`). Nothing runs twice: the used-up mark is one atomic exchange already in the cost table's "A C callback" row.

**REFUTER reason**: The cluster's question is decided as the reader says, as option A: `[FFI-21]`: "`retained` and `once` return a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the callback through the declared `destroys(f)` (`E5041` if none is declared, for `once` too, since a token that ends before C runs the function must unregister it); ... one ending after it calls nothing, since C gave the registration up by running it, and frees the slot's record"; `[FFI-53]`: "a `callback(once)` slot carries a used-up mark, taken with one atomic exchange" (already in `[COST-3]`'s "A C callback" row). `atexit` is decided too: `[FFI-21]` "A function-pointer parameter of a `safe fn` or of an overlay entry carries a `callback(…)` word, or the declaration is `E5002` ...; an `unsafe fn` may take one with no word", so it is `unsafe fn atexit(func: extern "C" fn()) -> c_int`. Safe: a library without a cancel function is declared `unsafe fn`, never safe. The reader's edit decides a real adjacent point (what the call returns when the C function has a result of its own; `[FFI-11]` says only that `destroys(f)`'s parameters "may be the registering call's own and its result ...: the token keeps them", ODR-246), but its reason is not a safety one: a program holding the id can only pass it to a `safe fn` whose asserter vouched for every argument (`[FFI-2]`), or to the `unsafe fn` unregister inside `unsafe:`; the trampoline keeps every stale call safe (`[FFI-53]`). Hiding the result and sending such functions to `unsafe fn` is an exemption that exists only because a form is missing, the lesson of verdict 132 and ODR 13, when the text already has the form: `[FFI-11b]` "When the C function has a result of its own, the Ember function returns a tuple: that result first, then each pointer output ...; a status result follows `[FFI-16]` instead".

- BETTER EDIT rule=FFI-21
  - old: return a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the callback through
  - new: return a `Retained[ForeignCallback]` token (`[FFI-23]`), the call's result (where the C function has a result of its own, the Ember function returns both as `[FFI-11b]` orders, that result first, then the token, or under `-> status` the token as the `Result`'s value, `[FFI-16]`; the token keeps the result for `destroys(f)`, `[FFI-11]`), whose end unregisters the callback through
  - why: Decides the same open point with the rule the text already has for a C result beside a produced value, keeps `safe fn` available to registering functions that return an id or a status, and takes nothing from safety: the id in the program's hands reaches C only through calls whose contracts stand on their own.

## K57-export-threads-main-creator  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: FFI-33: "The same holds under `creator`: `creator` and `main` name the same thread, and under `main` (`[FFI-33c]`) the body may also use `threads(creator)` values, while under `creator` it may not reach a `static mut` without its lock, because `creator` is also the word of a callback's and a handle's contract and says only that the caller is the maker, not that the module's state is its own."; FFI-33: "Under `any`, everything the export reaches is checked as if it ran on any thread: every `static` it reads is `ThreadShared`, and a `static mut` it reaches is `guarded by` a lock it holds there (`E7012`: "`bump` may be called from any thread (`threads=any`, line 3), and it changes `HITS`, a module variable of the main thread (line 1)", whose help gives the lock's lines and the `Atomic`'s)."; FFI-33c: "Under `threads = main`, the exported wrapper checks, in every profile, that the calling thread is the one that initialised the module, and panics otherwise; in exchange the body may use thread-confined state, the module's `static mut`s among it."; THR-20: "a function that may run on another thread (reachable from `thread.spawn`, from a job submitted outside a scope, from a callback C may run on a thread other than the main one, or from an `@unsafe(export("name", threads=any))` or `@unsafe(export("name", threads=creator))` function) and reads or changes it is `E7012`"; GRM-37: "`#! threads main|any|creator` sets the default thread contract of the file's exports (`[FFI-33]`)"; FFI-33: "A call before initialization or after shutdown panics. Repeated initialization while the module remains initialized does not transfer the contract; initialization after shutdown establishes a new creator."

**program**: 

**remaining_doubt**: The finder's program is decided: `#! threads creator` with `HITS += 1` in an export is `E7012` (`[FFI-33]`, `[THR-20]`), and the same body under `#! threads main` compiles (`[FFI-33c]`). The text keeps the difference, as option A did, but gives a naming reason (`creator` is a contract word that says only who the caller is) rather than A's re-initialisation reason; the operational facts are the same for both words ("`creator` and `main` name the same thread", "initialization after shutdown establishes a new creator"), and a re-initialisation on another thread after shutdown is memory-safe either way, since every call after shutdown panics and no owning value crosses an export (`[FFI-31b]`), so no thread but the new one touches the module's `static mut`s. Safety does not decide between the two words; readability does, and the text has written the trade-off down.

**REFUTER reason**: Decided, and safely. `[FFI-33]`: "a `static mut` it reaches is `guarded by` a lock it holds there (`E7012` ...). The same holds under `creator`: `creator` and `main` name the same thread, and under `main` (`[FFI-33c]`) the body may also use `threads(creator)` values, while under `creator` it may not reach a `static mut` without its lock, because `creator` is also the word of a callback's and a handle's contract and says only that the caller is the maker, not that the module's state is its own"; `[THR-20]` lists "an `@unsafe(export("name", threads=any))` or `@unsafe(export("name", threads=creator))` function" among those that may run on another thread, `E7012`; `[FFI-33c]`: "Under `threads = main`, the exported wrapper checks, in every profile, that the calling thread is the one that initialised the module, and panics otherwise; in exchange the body may use thread-confined state, the module's `static mut`s among it". So `#! threads creator` with `HITS += 1` is `E7012` and the same body under `#! threads main` compiles. The `main` permission is safe: the wrapper's check runs in every profile (its row in `[COST-6]`: "C called `tick` on another thread, but its export says `threads=main`"), "A call before initialization or after shutdown panics", so after "initialization after shutdown establishes a new creator" no thread but the new one reaches the `static mut`s, and no owning value crosses an export (`[FFI-31b]`). The trade-off between the two words is written down in the rule, as the brief asks.

## K75-overlay-header-mismatch  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: GRM-35: "overlay_decl  := ["unsafe"] "overlay" ("c" | "cpp") STRING"; GRM-35: "Ember records the header each overlay was written for in `ember.overlays`, never in the overlay (`[FFI-55]`)."; FFI-55: "For each overlay it holds the overlay's file, its header and one `blake3:` fingerprint of each declaration the overlay describes, as the importer binds it (`[FFI-14]`)"; FFI-55: "Every build that applies an overlay, its own package's and any other's, compares each declaration the overlay describes with the header it imports. One that differs is `E5010`"; FFI-30c: "An overlay of another package is applied only to the header it was written for (`[FFI-55]`)."; FFI-12: "An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`, so overlays cannot drift."; FFI-17: "Overlays for C++ use `overlay cpp "Header.hpp":` with the grammar of `[GRM-35]`."; FFI-6: "`import c "header" with (…) [as name]` parses the header with libclang under the project's target, language standard, defines and include paths, and exposes its functions, structs, unions, enums, typedefs, globals and constant macros as a module (`c`, or `name`). Options: `include_paths`, `defines`, `link`, `overlay`, `implementation` (`[FFI-29b]`)."

**program**: # overlays/sdl.em
unsafe overlay c "SDL.h":
    @ffi(no_callbacks)
    fn SDL_Quit()

# main.em
import c "SDL2/SDL_video.h" with (overlay = "overlays/sdl.em") as sdl

fn main():
    sdl.SDL_Quit()

The implementer's question: the overlay's first line names `SDL.h`, the import names `SDL2/SDL_video.h` (which declares `SDL_Quit` through its includes), and `[FFI-55]`'s record holds "its header" with no rule saying where that name comes from. `[FFI-30c]` applies another package's overlay "only to the header it was written for" but names no code for an import of another header, and `[FFI-12]` and `[FFI-55]` check only declarations. So whether this import is refused, applied (the STRING a comment), or applied with the record's `header` taken from the import, cannot be read from the text. Not a safety hole: every declaration the overlay describes is fingerprinted against the header imported, so a different `SDL_Quit` is `E5010` whatever the line says.

- READER EDIT rule=GRM-35
  - old: Ember records the header each overlay was written for in `ember.overlays`, never in the overlay (`[FFI-55]`).
  - new: The STRING names the header the overlay is written for: the file the import that applies the overlay names (`[FFI-6]`, `[FFI-17]`), the two spellings resolving to one file under that import's include paths, and the record's `header` is this name (`[FFI-55]`); an import of another header with the overlay is `E5010`: "`overlays/sdl.em` is written for `SDL.h` (line 1), and this import is of `SDL2/SDL_video.h`; import `SDL.h` here, or write an overlay for `SDL2/SDL_video.h`". Ember records the header's fingerprints in `ember.overlays`, never in the overlay (`[FFI-55]`).
  - why: The cause is the grammar keeping a header name on the overlay line after verdict 121 moved the fingerprint out, without saying what the name does. Keeping the name (a readable word on the thing the file describes) and refusing a mismatch is the reading verdict 121 allows (only the hash left the file); matching by the resolved file, not the spelling, keeps the rule a property the compiler reads.

**remaining_doubt**: Option B (drop the STRING, `unsafe overlay c:`) is equally safe and smaller in grammar, but takes the header's name out of the one file a reader opens; verdict 121 removed only the hash ("The overlay line's `with (header_hash = ...)` ... goes"), so A is the reading that changes nothing decided. A package's own overlay applied by two imports of two headers (both of which declare the described functions) is refused under this edit at the second import; a package wanting that writes two overlays, or one `import` and re-exports it (`[FFI-30b]`).

**REFUTER reason**: The hole is real but not a safety one. `[GRM-35]`: "overlay_decl := ["unsafe"] "overlay" ("c" | "cpp") STRING" gives the STRING no meaning; `[GRM-35]` "Ember records the header each overlay was written for in `ember.overlays`, never in the overlay (`[FFI-55]`)" and `[FFI-55]` "For each overlay it holds the overlay's file, its header and one `blake3:` fingerprint of each declaration" name a header the record holds without saying where its name comes from (`[CLI-7]` writes the record from the header `ember bind <header>` is given; the first build from the import's); `[FFI-30c]` "An overlay of another package is applied only to the header it was written for (`[FFI-55]`)" names no code for the import of another header, and `[FFI-12]` and `[FFI-55]` check declarations only. So whether the program compiles cannot be read. Safe either way: `[FFI-55]` "Every build that applies an overlay ... compares each declaration the overlay describes with the header it imports. One that differs is `E5010`". The reader's option A is the one verdict 121 leaves ("The overlay line's `with (header_hash = ...)` ... goes": only the hash left the line), and matching by the resolved file is a property the compiler reads. Two corrections: (1) the refusal and its message belong in `[FFI-55]`, the rule that already defines `E5010` for the overlay-against-header check and whose subject is "the header it was written for", with `[GRM-35]` saying only what the STRING is, so `E5010` keeps its two defining rules instead of a third; (2) the registry row must list the new case. The reader's edit also loses the sentence that the record, not the overlay, holds the fingerprints; kept here.

- BETTER EDIT rule=GRM-35
  - old: never in the overlay (`[FFI-55]`).
  - new: never in the overlay (`[FFI-55]`). The STRING names the header the overlay is written for, the one the import that applies it names (`[FFI-55]`).
  - why: The grammar says what its token means and points to the rule that checks it.

- BETTER EDIT rule=FFI-55
  - old: 
  - new: An overlay is applied only by an import of the header its first line names (`[GRM-35]`), the two spellings resolving to one file under the import's include paths (`[FFI-6]`), and the record's `header` is that name; an import of another header with it is `E5010`: "`overlays/sdl.em` is written for `SDL.h` (line 1), and this import is of `SDL2/SDL_video.h`; import `SDL.h` here, or write an overlay for `SDL2/SDL_video.h`".
  - why: The check sits with the other header checks of the overlay and the code they share; a package that wants one overlay on two headers writes two, or one import re-exported (`[FFI-30b]`).

- BETTER EDIT rule=registry row E5010
  - old: an entry outside a class section that names only members of classes | `[FFI-12]`, `[FFI-55]` |
  - new: an entry outside a class section that names only members of classes; an overlay applied by an import of a header other than the one its first line names | `[FFI-12]`, `[FFI-55]` |
  - why: Every case a code has is in its row.

## K76-callback-table-full  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: FFI-53: "the user data it is given is a tagged number, the callback's slot in a table that never moves in the low 48 bits and the slot's generation in the top 16"; FFI-53: "On a 32-bit target the slot is the low 16 bits and the generation the top 16, and the number may be a user-space address."; FFI-53: "A slot whose generation reaches its last value is retired and never reused, as `[HND-3]` retires a `Pool` slot, so a stale number can never name a later callback: one compare per registration, nothing per call."; HND-3: "When every slot is retired or live, `insert` panics ("`pool.insert(...)` on line 11: every slot is live or retired; use `try_insert` for a `Result`") and `try_insert` returns `Err(CapacityError.Full)`."; COST-6: "| a full `Pool` (`[HND-3]`) | "`pool.insert(...)` on line 11: every slot is live or retired; use `try_insert` for a `Result`" |"; COST-3: "| A C callback | guaranteed present | the checked trampoline (Part XVI): C holds a tagged number, never an address; each call compares the callback's generation, and, in every profile, the thread of a `threads(creator)` callback and the used-up mark of a `callback(once)` one |"

**program**: from std.ffi import ForeignCallback, Retained

unsafe extern "C":
    @safety("`cb` and `ud` are a pair `on_click` was given")
    unsafe fn off_click(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(off_click))
    safe fn on_click(cb: extern "C" fn(*mut void), ud: *mut void)

class Hud:
    clicks: int = 0

fn main():
    hud = shares mut Hud()
    tokens: Array[Retained[ForeignCallback]] = []
    for i in 0..70000:
        tokens.push(owns on_click(fn() shares mut hud: hud.clicks += 1))
    println(tokens.len())

The implementer's question: on a 32-bit target the slot field is 16 bits, so the 65,537th live registration finds no slot, and nothing says what `on_click`, a `safe fn` with no error result, does then (stop the program as `[HND-3]`'s `insert` does, return an error, or reuse a slot, which `[FFI-53]`'s promise that "a stale number can never name a later callback" forbids). A program that registers and unregisters callbacks for a long time reaches the same state through retired slots. On a 64-bit target the text does not say how a table of up to 2^48 slots "that never moves" is laid out (grown by chunks, or bounded), so whether the 64-bit table can fill is undecided too.

- READER EDIT rule=FFI-53
  - old: A slot whose generation reaches its last value is retired and never reused, as `[HND-3]` retires a `Pool` slot, so a stale number can never name a later callback: one compare per registration, nothing per call.
  - new: A slot whose generation reaches its last value is retired and never reused, as `[HND-3]` retires a `Pool` slot, so a stale number can never name a later callback: one compare per registration, nothing per call. The table grows by chunks and never moves a slot, as an `Arena` grows (`[ARN-4]`), so a slot is free while memory is and the slot field is wide; when every slot the number can name is live or retired, which only a 32-bit target's 65,536 slots can reach, the registering call stops the program, in every profile, as a full `Pool` does: "`on_click(...)` on line 16: no callback slot is free; every one of the 65,536 slots a 32-bit target has is live or retired" (`[COST-6]`).
  - why: The cause is `[FFI-53]` sizing the table and retiring slots without saying what a registration does when none is free; `[HND-3]` is the rule the text already borrows for retirement, and its answer (a stop in the program's lines) costs nothing per call and nothing on a 64-bit target, where the table cannot fill. An error result on every registering call would tax 64-bit programs for a limit they cannot reach.

**cost_row**: §X.4 `[COST-6]`, a new row: | the callback table full (`[FFI-53]`) | "`on_click(...)` on line 16: no callback slot is free; every one of the 65,536 slots a 32-bit target has is live or retired" |  (nothing new runs per call; the free-slot test is the registration's own allocation, already counted in the "A C callback" row of `[COST-3]`)

**remaining_doubt**: `docs/AUTOPILOT.md` §4 has the owner's rule that a panic is right for the caller's bug "never for a limit of the implementation"; here the limit is C's 32-bit `void*`, which no Ember choice widens (any split of 32 bits between slot and generation is a limit, and a wider slot field shortens the reuse count before retirement), so the stop is for a limit the target sets, not one Ember chose. If the lead reads the rule as forbidding the stop even so, the alternative is `Result[Retained[ForeignCallback], CapacityError]` from every kept-callback registration (the finder's B), which every 64-bit program must then handle for a case it cannot reach; a middle way, a `try_` form of the registering call, has no declaration syntax today. Verdicts 164 to 166 are untouched.

**REFUTER reason**: Real and undecided. `[FFI-53]`: "the callback's slot in a table that never moves in the low 48 bits and the slot's generation in the top 16 ... On a 32-bit target the slot is the low 16 bits and the generation the top 16" and "A slot whose generation reaches its last value is retired and never reused, as `[HND-3]` retires a `Pool` slot, so a stale number can never name a later callback"; nothing says what a registration does when every slot is live or retired, which 65,536 live tokens on a 32-bit target reach in the finder's program, and a long-lived program reaches through retirement; `[HND-3]` answers only for a `Pool` ("When every slot is retired or live, `insert` panics ... and `try_insert` returns `Err(CapacityError.Full)`"), and a `safe fn` registering call has no error result. Reusing a slot would break `[FFI-53]`'s promise, so the choice is a stop or a `Result`; both are safe, and the reader weighs them and writes the trade-off down: the stop costs nothing per call and nothing on 64-bit, where `[ARN-4]`'s chunked growth ("grows by taking a new chunk: nothing it has given out moves") makes the table free while memory is, its failure the existing allocation-failure row. The edit adds its `[COST-6]` row and departs from no verdict. The lead should rule the one open point the reader names: `docs/AUTOPILOT.md` §4, "A panic is right for the caller's bug ..., never for a limit of the implementation, such as a count kept in a type too small for what it counts"; here the type is C's 32-bit `void*`, which no split of its bits escapes (a wider slot field shortens the generation and so the reuse count), so the reader's reading that the limit is the target's is defensible, and the finder's B (a `Result` on every kept-callback registration) is the alternative if the lead reads the rule as forbidding the stop.

# MISSED

## MISSED: A free C++ operator whose first operand is not the class implements nothing, and the text does not say so

**rules**: `[FFI-42]`: "Overloaded operators map to Ember operator interfaces where one exists"; `[TYP-40]`: "A type implements an interface only through an `implements` clause or an `extend … implements` block"; `[FFI-41]`: "Static member functions import as functions of the type with no object". Nothing says which type a free operator function (`operator+(const Money&, const Money&)`, `operator*(double, const Vec&)`) becomes a method of, nor what `Rhs` is; the compiler's own ruling (decisions.md, FFI-40a (132): imported "as the class's methods (`operator+(const Money&, const Money&)` as `Money.add`)") is in no rule, and a left operand that is `double` has no imported class to carry the implementation.

**program**:
```
// vec.hpp
// struct Vec { double x, y; };
// Vec operator*(const Vec& v, double k);
// Vec operator*(double k, const Vec& v);
import cpp "vec.hpp" with (project="sim") as sim

fn main():
    v = sim.Vec(x=1.0, y=2.0)
    a = v * 2.0        # `Vec.mul`? which `Rhs`?
    b = 2.0 * v        # `f64 implements Mul[Vec]` from an import, or refused, or skipped?

An implementer cannot say whether `b` compiles, and an import adding an implementation to `f64` has no rule allowing it.
```

- MISSED EDIT rule=FFI-42
  - old: 
  - new: A free operator function is a method of the imported class that is its first operand, with the other operand's type as `Rhs` (`operator*(const Vec&, double)` is `Vec.mul` with `Rhs = f64`); one whose first operand is not a class this header declares (`operator*(double, const Vec&)`) implements nothing, since an import adds no implementation to a type it did not declare (`[TYP-40]`), and is skipped with `W5033`, its arithmetic reached through an `extend` wrapper in the overlay (`[FFI-13]`) that calls it by a name the overlay gives.
  - why: States the property (an operator belongs to the class of its first operand) the compiler already applies, and refuses the one shape no rule can carry instead of leaving it to a guess.

- MISSED EDIT rule=registry row W5033
  - old: | `W5033` | `&&`-qualified member skipped | `[FFI-40]` |
  - new: | `W5033` | `&&`-qualified member skipped; a free operator whose first operand is not a class of the header skipped | `[FFI-40]`, `[FFI-42]` |
  - why: Every case a code has is in its row.

## MISSED: `[FFI-10]` says the tagged number is 64 bits, which is false on a 32-bit target

**rules**: `[FFI-10]`: "`p` is written as a raw pointer, or an integer wide enough for the tagged number (`E5002` otherwise, since `[FFI-53]`'s number is 64 bits)"; `[FFI-53]`: "On a 32-bit target the slot is the low 16 bits and the generation the top 16". On a 32-bit target the number is 32 bits, so whether a `uint32_t` user-data parameter is `E5002` there cannot be read: the rule says "wide enough" and the parenthesis says 64.

**program**:
```
unsafe extern "C":
    @safety("`cb` and `ud` are a pair `on_tick` was given")
    unsafe fn off_tick(cb: extern "C" fn(u32), ud: u32)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(off_tick))
    safe fn on_tick(cb: extern "C" fn(u32), ud: u32)     # 32-bit target: wide enough, or E5002 "64 bits"?
```

- MISSED EDIT rule=FFI-10
  - old: 64 bits), at its C ABI position,
  - new: a pointer's width: 64 bits on a 64-bit target, 32 on a 32-bit one, `[FFI-53]`), at its C ABI position,
  - why: The parenthesis states the fact `[FFI-53]` gives for each target instead of one target's number.

## MISSED: `[THR-2]`'s message shows a safe call that `[FFI-11]` refuses

**rules**: `[THR-2]`: "C keeps its address (`register(names)`, line 4), so it cannot move; end the link first"; `[FFI-11]`: `retained` on a non-callback pointer has "no safe type ...: the call stays `unsafe`, with a raw pointer"; `[FFI-10]`: "`safe fn` with one is `E5002`". A class object given directly to a C function under `retained` is `E5002`, so the line the message names cannot be the C call; the message should show the line that compiles (K52's spelling), or the example reads as syntax the text refuses.

**program**:
```
class Names:
    items: Array[String] = []

unsafe extern "C":
    @ffi(param(p, retained, one))
    safe fn register(p: Names)      # E5002 under [FFI-10] and [FFI-11]; yet [THR-2]'s message shows `register(names)`
```

- MISSED EDIT rule=THR-2
  - old: (`register(names)`, line 4)
  - new: (`register(ref_to_ptr(ref mut names))`, line 4, an `unsafe` call whose `p` is `retained`)
  - why: The message names a line the language accepts (§IV.5, `[FFI-11]`, `[FFI-35a]`), so the example compiles as the brief asks of every example.

## MISSED: The ODR-166 row says `ForeignBox[T]` where `[FFI-32]` says `owns T`

**rules**: Appendix row: "| ODR-166 | delegated: a C++ function returning an opaque class by value gives a `ForeignBox[T]` | `[FFI-32]`, Annex C's import table |"; `[FFI-32]`: "A function returning an opaque class by value gives an `owns T`, as a constructor does"; §II: "`Box`, `Shared`, `Weak`, `SyncShared`, `ForeignBox`, `RefCell`, `Ref` and `RefMut` are not types" and the migration "`ForeignBox[T]` to `owns T`". A cross-reference claims a type the rule it names does not give and the language no longer has.

**program**:
```
No program: a consistency defect of a cross-reference (the brief's "every cross-reference claims what its rule says"). A reader of the ODR table is sent to `[FFI-32]` for a `ForeignBox[T]` that is `E1010` ("gone from Ember").
```

- MISSED EDIT rule=appendix row ODR-166
  - old: | ODR-166 | delegated: a C++ function returning an opaque class by value gives a `ForeignBox[T]` | `[FFI-32]`, Annex C's import table |
  - new: | ODR-166 | delegated: a C++ function returning an opaque class by value gives an `owns T` (written `ForeignBox[T]` when ruled, a name since gone, §II) | `[FFI-32]`, Annex C's import table |
  - why: The row claims what `[FFI-32]` says and keeps the ruling's history in one clause.
