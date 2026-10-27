
## XVII.9 The diagnostic code registry

Every diagnostic code, what it reports and the rule it enforces (`[DIA-6a]`). Codes are never
reused; retired codes are listed with the reason.

| Code | Reports | Rule |
|---|---|---|
| `E0001` | invalid UTF-8 in source | `[LEX-1]` |
| `E0002` | tab used for indentation | `[LEX-4]` |
| `E0003` | inconsistent dedent | `[LEX-5]` |
| `E0004` | expected an indented block | `[LEX-9]` |
| `E0005` | reserved keyword used as an identifier | `[LEX-14]` |
| `E0006` | a `#!` language directive names a version other than the current one | `[VER-8]` |
| `E0008` | unterminated character literal | `[LEX-22]` |
| `E0009` | malformed literal | `[LEX-26]` |
| `E0100` | unexpected token, among them Python's `global`, a statement at module level outside the entry file (whose message gives the declaration to write: `static mut FRAME: int = 0`), and a clause after a function's brackets that Ember does not have (`changes FRAME`, `reads renderer`, `with` before the captures), whose message gives the first line to write | `[GRM-2]` |
| `E0101` | unclosed delimiter or literal | `[LEX-6]` |
| `E0102` | membership test chained (`a in b in c`) | `[GRM-23]` |
| `E0103` | match arms mix statement and expression form | `[GRM-10]` |
| `E0104` | attribute or construct not admitted here, among them an attribute that is gone, with the one to write (`@sync`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`, `@borrows`) | `[ATT-1]`, `[PHIL-12]` |
| `E0105` | `;` is not a statement separator | `[GRM-18]` |
| `E0106` | a multi-statement closure cannot be written inside brackets | `[GRM-17]` |
| `E0107` | a jump expression may not be an operand | `[GRM-16]` |
| `E0108` | attribute is not permitted on this statement | `[GRM-20]` |
| `E0109` | `owns` before a name whose place already hands the value over with no word (`e.mesh = m`; `b = a` for a value that is not a number; an argument; `return a`), or before `fn` | `[OWN-9]`, `[GRM-15]` |
| `E0110` | function declared without a body outside an interface, extern block or abstract class | `[GRM-33]` |
| `E0111` | `ref` of an expression that is not a place | `[GRM-36]` |
| `E0112` | nested more deeply than the compiler accepts | `[GRM-39]` |
| `E0113` | `mut` alone on a name: a pattern name, a parameter, a lambda parameter, `this` (`static mut` is a declaration and is not this) | `[GRM-42]`, `[FN-1]` |
| `E0114` | `sharedref` not followed by `mut` | `[GRM-43]`, `[LT-46]` |
| `E0115` | `Some(...)` in an `if` or `while` header | `[GRM-44]` |
| `E0116` | in a list of names declared without types, a name with no word after a name with a word | `[GRM-42]` |
| `E0117` | `ref` or `ref mut` written in a parameter's type instead of on its name | `[FN-1]` |
| `E0118` | a first line whose parts are out of order (parameters, `owns this`, `-> T`, `from`) | `[GRM-41]` |
| `E0120` | `ref mut` in a lambda's capture list | `[GRM-42]`, `[CLO-2]` |
| `E0900` | construct not implemented by this compiler | `[PHIL-12]`, `[CLI-19]` |
| `E0901` | construct this specification leaves unspecified | `[PHIL-12]` |
| `E1010` | a name or form the program may not use here: not in scope (`this` outside a method or in a `static fn`), gone from Ember (`self`, `Box`, `ForeignBox`, `RefCell`, `Weak`, `Shared`, the mode `owned`, `Sync` as a property, `cast x if D`, `cast x when D`, `upgrade`, `mem.forget`), or written with a word instead (`a.clone()` is `copy of a`; `.copied()` is `.copy_each()`) | `[MOD-3]` |
| `E1020` | name is already declared in this block | `[GRM-4]` |
| `E1021` | name is not linked in this build | `[BLD-11]` |
| `E1030` | two items with the same name in one scope, such as a second `main` (a statement at module level is `E0100`) | `[TYP-26]` |
| `E1031` | a name bound by two glob imports is used | `[MOD-8]` |
| `E1032` | an item has the path of a module or the name of a built-in type | `[MOD-1]` |
| `E1041` | import cycle between packages | `[MOD-4]` |
| `E1050` | field is read-only outside its module | `[MOD-7]` |
| `E1051` | `read` visibility is valid on fields only | `[MOD-7]` |
| `E1052` | item, field or constructor not visible here | `[MOD-2]` |
| `E1060` | no module of that name | `[MOD-3]` |
| `E1061` | no item of that name in the module | `[MOD-3]` |
| `E2010` | literal does not fit its type | `[LEX-16]` |
| `E2011` | negative literal index | `[TYP-31]`, `[LEX-24]` |
| `E2020` | mismatched types, among them a lambda parameter whose word differs from the function type it is given to (a plain `fn(s)` where the type hands the value over, `fn(owns String)`); a `weakref` link used as its target without opening it; an object stored into a `weakref` place without `weakref`; a value given to a `using A` place that was not made in an instance of `A` | `[TYP-4]`, `[CLO-3]` |
| `E2030` | `@view` on a type that is not a view | `[TYP-34]` |
| `E2031` | `from` names a parameter the result cannot point into (a number, or an `owns` parameter that is not a view) | `[LT-1a]` |
| `E2035` | condition must be `bool` | `[CTL-0]` |
| `E2036` | this pattern always matches: an `if` or `while` opening whose left side always holds a value, or a kind test that always passes | `[GRM-19]` |
| `E2037` | a kind test that can never pass: the type tested for does not implement the interface, or derive from the base class, of the value tested | `[TYP-44]`, `[DSP-4]` |
| `E2040` | unsatisfied interface bound; for a `pub` generic whose bounds are not written, one message on the first line of its body that needs a missing bound, giving every bound the body needs | `[IFC-6]`, `[TYP-17]`, `[BLD-14]` |
| `E2041` | overlapping `extend` implementations | `[TYP-19]` |
| `E2042` | `Float` is implemented only by `f32` and `f64` | `[STD-27]` |
| `E2043` | an associated type's default leads back to itself | `[IFC-4]` |
| `E2050` | interface is not `dyn`-compatible | `[IFC-5]`, `[TYP-22]` |
| `E2060` | cannot infer type | `[TYP-23]` |
| `E2061` | lambda parameter types cannot be inferred here | `[TYP-23]` |
| `E2062` | ambiguous type | `[TYP-23]` |
| `E2070` | ambiguous interface method | `[TYP-24]` |
| `E2071` | `SoA[T]` of a type that is not a struct | `[SOA-1]` |
| `E2072` | no method or field of that name (with the Ember name for a Python one; shape N13 for an undeclared field; in a generic body that is not `pub`, a method no bound's interface declares) | `[STD-13]` |
| `E2073` | `len` of a string (Python counts characters, `s.len()` bytes) | `[STD-26]` |
| `E2080` | `@derive(Copy)` on a type with a field that is not Copy | `[STR-3]` |
| `E2090` | non-exhaustive match | `[ENM-2]` |
| `E2100` | field read before it is set, `init` calling a method whose body reads a field not yet set included | `[CLS-2]` |
| `E2101` | derived class with a field that has no default needs an `init` | `[CLS-10]` |
| `E2102` | derived `init` uses `this` or an inherited field before `super.init`, or leaves an own field unassigned | `[CLS-11]` |
| `E2103` | `this` given another object in a class method | `[CLS-7]` |
| `E2104` | a `let` field changed after `init`, for every type | `[CLS-9]` |
| `E2106` | an `if` or `while` opening, or a kind test, that names a name already in use | `[GRM-44]`, `[GRM-45]`, `[DSP-4]` |
| `E2108` | `owns this` on `init` | `[CLS-2]` |
| `E2110` | override of a method that is not virtual | `[CLS-4]` |
| `E2111` | a method that replaces an inherited virtual one without `override` | `[CLS-4]` |
| `E2112` | `virtual` in a class that cannot be subclassed | `[CLS-4]` |
| `E2114` | a type word that cannot apply to its type: `owns` on a class type (it always has its own block); `using` after a type with no heap memory of its own | `[TYP-45]`, `[TYP-46]`, `[ALC-1]` |
| `E2115` | `static fn` outside a type body | `[GRM-41]`, `[FN-4]` |
| `E2116` | `copy of` before what it cannot copy as itself: a `weakref` link, a value of a view type (a struct holding a `ref mut` field included), a type with no `Clone`, a value whose views point into its own `let` field | `[OWN-8]` |
| `E2117` | `@cached` on a method whose result it cannot keep safely: one with other parameters; one whose body changes anything, or reads a `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut` or `Cell` field, or a field other threads change under a lock; a `sharedref mut` link or `shares mut` owner made into what a `@cached` method reads, at the line that makes it | `[CLS-12]`, `[EFF-27]` |
| `E2118` | a `from` that is missing where a view needs one (a view field of a class), names what a view cannot point into for its life (a field that is not `let`, a fixed array or an inline struct field), or names two things (a parameter and a field of one name) | `[LT-45]`, `[TYP-15]` |
| `E2119` | a `weakref` of a part (a field, an element stored inline), whose target does not live on its own | `[WK-16]` |
| `E2120` | inherent extension of a type from another package | `[IFC-2]` |
| `E2121` | an implementation outside the packages that may write it | `[TYP-20]` |
| `E2123` | a value made for a place that owns it alone (an `owns` field, an `Option[owns T]`, a list of `owns` values), at first or later, without `owns` on the line that makes it | `[OWN-10]`, `[GRM-15]`, `[TYP-46]` |
| `E2130` | a `const` of a type that owns heap memory; use a `static` | `[TYP-1]` |
| `E2131` | array length must be a constant | `[CT-1]` |
| `E2140` | a change to something that is not a place, which would be lost: an assignment to a value, or a method whose only effect is changing a temporary that ends with the line | `[BRW-19]`, `[EXP-5]` |
| `E2150` | `is` on operands that are neither class objects, links, references nor `Option`s, and not a kind test (`x is T`) | `[EXP-9]` |
| `E2151` | integer `**` with a constant negative exponent | `[TYP-30]` |
| `E2160` | control flow cannot leave a `defer` block | `[CTL-7]` |
| `E2170` | reference to a field of a packed struct | `[LAY-2]` |
| `E2172` | cannot index with a type | `[GRM-8b]` |
| `E2173` | a type position holding what is not a type: a bare interface name used as a type (`Array[Shape]`), or an argument that is neither a type nor a constant (an interface implemented by values only: ODR-182) | `[TYP-45]`, `[IFC-5]`, `[OBJ-2]` |
| `E2180` | `?` outside a function returning `Option` or `Result` | `[ERR-2]` |
| `E2181` | `?` on an `Option` in a `Result` function, or the reverse | `[ERR-2]` |
| `E2182` | a function that returns a value can reach the end of its body | `[FN-10]` |
| `E2183` | `main` has none of the forms `[FN-8]` allows | `[FN-8]` |
| `E2200` | type has infinite size: a type that contains itself, or holds a view of its own type, with no block of its own between (an `owns` field, a list, a `shares` block) | `[TYP-14]` |
| `E2210` | a value of one range type where another was expected | `[RNG-2]` |
| `E2211` | constant outside the target range type | `[RNG-3]` |
| `E2212` | range endpoints are not constants of the representation, or are inverted | `[RNG-1]` |
| `E2213` | invalid `in` clause on a type alias | `[RNG-1]` |
| `E2214` | arithmetic between two distinct range types | `[RNG-5]` |
| `E2215` | range-typed value constructed outside the permitted set | `[RNG-10]` |
| `E2220` | `yield` outside a `gen fn` | `[CORO-2]` |
| `E2221` | a view into a generator's own frame held across `yield` | `[CORO-6]` |
| `E2222` | coroutine where an ordinary function is required | `[CORO-10]` |
| `E2223` | `@never_specialize` on a generic that is not shareable | `[MONO-7]` |
| `E2225` | `migrate_from` may not panic | `[HR-35]` |
| `E2226` | `in` on a type that does not implement `Contains` | `[STD-8]` |
| `E2230` | a name assigned in every branch at different types | `[CTL-10]` |
| `E2231` | `yield` while a `@must_drop` value is live | `[CORO-13]` |
| `E2240` | `/` on two integers | `[TYP-28]` |
| `E2250` | format spec does not apply to the value's type | `[LEX-19]` |
| `E2260` | `some` type outside a return position | `[TYP-32]` |
| `E2261` | returns of a `some` function have different types | `[TYP-32]` |
| `E3010` | cannot hand over a field of a type with `drop` on its own | `[EXP-6]` |
| `E3011` | cannot hand over an array or span element on its own | `[EXP-6]` |
| `E3013` | a hand-over out of a name or place that does not own the value: a plain parameter's value, a function value's own capture where it may run twice, a plain `once fn()` parameter, or a field of a class object, which other names may reach | `[OWN-11]`, `[EXP-6]`, `[CLO-6]` |
| `E3014` | scope binding may not be handed over | `[THR-5]` |
| `E3015` | value whose drop is required may not be leaked | `[THR-6]` |
| `E3016` | `this` escapes its own `drop`, or an owner of it is made there | `[OBJ-5]`, `[CLS-7a]` |
| `E3020` | a collection (a list, a map, a set or a string) changed while it is walked, whatever the change (an element written, a value under a key replaced, anything added or removed), by a `for` loop or by a generator that walks it and will still be used; through its own name, or through a call that may change it (a call through a function value or a library call by the functions it may run, a `spawn` by what its thread changes, a scoped task started in the walk, a foreign call by what the callbacks C keeps change, the end of an object by what its `drop` changes); a removal inside a counting loop | `[BRW-18]`, `[CTL-2]` |
| `E3021` | a read and a change of one place overlap: a change of what a read-only name reads while it is still used; a `ref mut` name's place read through another name while it is still used; an argument that may reach into a changeable argument of the same call | `[BRW-1]`, `[BRW-11]` |
| `E3022` | one place changed through two names: two changeable arguments of one call naming one place, other than one whole variable proved when compiling (a function value that changes it counts as one); a function value stored in a place that a variable it changes owns; a `ref mut` name's place changed through another name while it is still used, a reshape of a `ref mut` address included | `[BRW-1]`, `[BRW-11]` |
| `E3023` | a change the name does not allow: a change through a name that only reads (a plain parameter; a loop, pattern, opened or part name; a plain name holding a `MutSpan`; a value made when compiling); `=` on a name that keeps its place for its scope (a parameter; a loop, pattern or opened name, one opened `exists as copy of` excepted; a `const ref mut` name given another ref); a read-only `ref` given to a `ref mut` name; a method that may change its object, or a stored function that may change its own state, called through a read-only name; an owner that may change made from one that only reads (a plain parameter of a `shares mut` type included) | `[BRW-14]`, `[BRW-15]` |
| `E3024` | a struct field would point into another field of the same struct (shape B5) | `[TYP-15]` |
| `E3026` | a function value that may outlive a place it uses and does not own: a returned function that changes its maker's local without `owns`; a task or thread given a loop's name with no word, or with `ref` where it may outlive the element | `[CLO-4]` |
| `E3027` | `ref mut` before something this function may not change: an argument, a loop or pattern name over something it only reads, or a place reached through a read-only name | `[FN-2a]` |
| `E3028` | a call's word does not match its parameter: no `ref mut` (or no `const` for a `const ref mut` parameter) before an argument for a `ref mut` parameter, a `MutSpan` one included, or `ref mut` before one for a parameter that is not `ref mut` (shape B10) | `[FN-2a]` |
| `E3031` | a call whose argument the callee's body reaches by another name in a way its parameter cannot allow: a part of what the callee may end or move before its last use of that parameter (a plain, `Span` or `ref mut` parameter, or a method's object); for a `ref mut` parameter, a part of what the callee reads or changes by another name while the parameter is used; at the line that stores a function into a function value, when that function does so with what the value's calls give it (shape B16) | `[BRW-21]`, `[FFI-52]` |
| `E3032` | a loop of owners (`shares`, `shares mut` or `thread_shared` owners, a function value's state, channel ends) that the compiler cannot prove is opened again before its objects lose their last owner from outside (shape B17) | `[WK-6]` |
| `E3040` | a name used after it was handed over or ended (a number handed over with `owns` included); a part handed over on its own, or its owner used whole, before the part is given a new value | `[OWN-3]` |
| `E3050` | a view of an uninitialised place | `[BRW-7]` |
| `E3060` | a view used after what it points into was ended, handed over, replaced or reshaped: a position after a removal or reorder; an address after a reshape; a view put in through `ref mut` after one of the call's arguments changed; a name opened from a link after a line or call that may end its object; a view of what a function value's or an `owns dyn I` value's state owns after that value is replaced or ended | `[BRW-17]`, `[LT-3]` |
| `E3062` | a result that points into something its first line's `from` does not name (shape B6) | `[LT-1a]` |
| `E3063` | a view, or a value holding views, kept or returned where it would outlive what it points into: a view of a temporary, a view out of a lock's `with` block, a view of the callee's own locals put into the caller's view through `ref mut`, a kept view outliving its target; and a value made in a memory manager's instance, or one holding such a value, kept or returned where it would outlive that instance | `[LT-45]`, `[BRW-19]`, `[ALC-1]` |
| `E3070` | `drop` cannot be called explicitly | `[DRP-1]` |
| `E3090` | arena allocation of a type that needs `drop` | `[ARN-3]` |
| `E3095` | two single-object references given to `assert_disjoint` | `[DSJ-4]` |
| `E3096` | arena is scoped here | `[ARN-6]` |
| `E3100` | an unsafe operation outside an `unsafe:` block, an `unsafe fn` body and `into_raw()` included; `Send` declared by hand without `unsafe extend` | `[UNS-1]`, `[UNS-12]`, `[ALC-2]`, `[ARN-8]`, `[FFI-51]`, `[FFI-26]`, `[FFI-54]`, `[FFI-56]` |
| `E3105` | `UnsafeCell` in `@no_alias_checks` code | `[UNS-10]` |
| `E4001` | `@noalloc` function reaches an allocation | `[EFF-5]` |
| `E4002` | `@nosync` function reaches a synchronising operation | `[EFF-5]` |
| `E4003` | `@noblock` function reaches a blocking operation | `[EFF-5]` |
| `E4010` | implementation does not satisfy the interface's contract | `[EFF-2]` |
| `E4020` | `@simd(assert)` loop did not vectorise | `[SIMD-2]` |
| `E4030` | `assert_disjoint`, the one alias check left to run time, inside `@no_alias_checks` | `[DSJ-6]`, `[EFF-12]` |
| `E4040` | `@nopanic(explicit)` function reaches a panic | `[EFF-17]` |
| `E4041` | `@noio` function reaches I/O | `[EFF-20]` |
| `E4042` | `@nolock` function acquires a lock | `[EFF-21]` |
| `E4070` | `@deterministic` function reaches a nondeterministic operation | `[DET-1]` |
| `E4072` | `@fastmath` or `@fp(allow_fma)` inside `@deterministic` | `[DET-5]` |
| `E5001` | imported type layout does not match | `[FFI-5]` |
| `E5002` | foreign call requires `unsafe`: its contract is incomplete, or `safe fn` on a function whose contract is | `[FFI-2]`, `[FFI-10]` |
| `E5010` | overlay does not match the C declaration; an overlay applied to a header other than the one it was written for (its hash differs) | `[FFI-12]`, `[FFI-55]` |
| `E5011` | one C identity imported with two different layouts | `[FFI-30]` |
| `E5012` | pointer contract has no count | `[FFI-11]` |
| `E5013` | a further argument of a variadic C function that C would promote to another type | `[FFI-8]` |
| `E5014` | two packages request the same implementation macro | `[FFI-29]` |
| `E5015` | type may not cross the boundary | `[FFI-31]`, `[FFI-31b]` |
| `E5016` | `_Atomic` layout does not match `Atomic[T]` | `[FFI-8]` |
| `E5017` | struct with a flexible array member is unsized | `[FFI-8]` |
| `E5018` | `va_list` may not be constructed | `[FFI-8]` |
| `E5020` | `StrView` is not NUL-terminated | `[FFI-15]` |
| `E5030` | `std::function` as a parameter | `[FFI-17a]` |
| `E5031` | C++ parameter cannot be mapped | `[FFI-17]` |
| `E5032` | opaque C++ type may not be constructed | `[FFI-32]` |
| `E5034` | unsupported C++ construct | `[FFI-17]` |
| `E5040` | capturing closure passed where a C function pointer is expected | `[FFI-21]` |
| `E5041` | retained callback needs a release function in the overlay | `[FFI-21]` |
| `E5042` | a function C may call returns a pointer into a parameter C passes as a copy | `[FFI-21a]` |
| `E5050` | a foreign fact claims a grade whose evidence is absent or stale | `[TCB-1]` |
| `E5051` | a foreign callee retains a pointer its contract does not declare `retained` | `[FFI-35a]` |
| `E5052` | `adopt` on a handle whose overlay declares `adopt = false` | `[FFI-36b]` |
| `E5053` | an instrumented run contradicted a declared foreign effect | `[FFI-37]` |
| `E5054` | range type, or `char`, in a foreign signature | `[RNG-10]`, `[FFI-31]`, `[FFI-31b]` |
| `E5055` | an Ember generic may not instantiate a C++ template | `[FFI-17b]` |
| `E5056` | override of a C++ virtual the overlay does not name | `[FFI-39]` |
| `E5057` | the overlay names a method that is not virtual in the header | `[FFI-39]` |
| `E5058` | C++ base has no default constructor and no declared `init` | `[FFI-39]` |
| `E5059` | C++ trampoline base has no virtual destructor | `[FFI-39]` |
| `E5060` | an object owned by Ember (`owner="ember"`) given to a C++ parameter that keeps it | `[FFI-39c]` |
| `E5061` | imported C++ function has no exception policy and none can be derived | `[FFI-24]` |
| `E5062` | contradictory C++ exception policies | `[FFI-43]` |
| `E5063` | foreign bytes reach `StrView` without validation | `[TXT-2]` |
| `E5064` | interior NUL: a `c"..."` literal containing a NUL, or a value converted to a `CStrView` | `[TXT-3]`, `[FFI-15]` |
| `E5065` | `shares`/`weakref` and `CppShared`/`CppWeak` do not interconvert | `[WK-14]` |
| `E5066` | an overlay marks a C++ function both `noexcept` and throwing | `[FFI-43]` |
| `E5067` | an overlay that states facts is not declared `unsafe overlay` | `[GRM-35]`, `[TIER-1]` |
| `E5068` | one live foreign object adopted twice | `[FFI-36b]` |
| `E5069` | `@ffi(no_callbacks)` on a foreign function whose own contract takes a callback (`callback(...)`) | `[FFI-52]`, `[FFI-11]` |
| `E5090` | inline assembly is not supported by the C backend | `[UNS-6]` |
| `E6001` | comptime evaluation exceeded its limits, or a constant's value depends on itself | `[CT-3]` |
| `E6004` | panic during compile-time evaluation | `[CT-7]` |
| `E6005` | `comptime(e)` refers to a run-time local | `[CT-6]` |
| `E6010` | operation not available at compile time | `[CT-2]` |
| `E7002` | `static` of a type that is not `ThreadShared` | `[STA-1]` |
| `E7004` | an unscoped thread or a kept callback uses a name with no capture word (a loop's name included) | `[THR-17]`, `[THR-10]`, `[FFI-21]` |
| `E7006` | memory order an atomic operation does not support | `[THR-14]` |
| `E7007` | two tasks of one scope change one place other than by a reduction, under a lock or through an `Atomic` (a task that reads the place before changing it, and a plain assignment, included) | `[THR-11]`, `[THR-21]` |
| `E7008` | a thread-bound value (a guard, a `Scope`, a raw pointer, a `threads(creator)` handle) sent to another thread | `[THR-8]`, `[THR-3]` |
| `E7010` | a parallel loop's chunks may write one place | `[PAR-2]` |
| `E7011` | parallel loop has a loop-carried dependency | `[PAR-2a]` |
| `E7012` | a `static mut` not `guarded by` a lock reached from another thread | `[THR-20]`, `[STA-1]` |
| `E7013` | a `guarded by` field used without holding its lock | `[THR-18]` |
| `E7014` | a `thread_shared` value changed directly | `[THR-1]` |
| `E7015` | a one-thread owner (`shares`, `shares mut`, a `weakref` link of one) or a lent value reaches another thread: held by a function any thread may run (`thread_shared fn()`, `thread.spawn`'s included), in a plain `fn()` field of a class whose object is made `thread_shared` (at that line), or a class object handed to another thread while a `weakref` link on this thread points at it | `[THR-1]`, `[THR-2]`, `[THR-10]`, `[CLO-16]`, `[FFI-22]` |
| `E7016` | a `@parallel` loop over something it cannot split into chunks: neither a range nor a list walked by element | `[PAR-1]` |
| `E7017` | `thread_shared` on a name whose value was not created `thread_shared` | `[THR-1]` |
| `E7018` | a wait for a lock this thread holds, inside that lock's `with` block: a call through a function value, or a `dyn` or virtual method, that may run a function taking that lock (inside a walk under that lock too), or a join of a thread (a scope's end included) whose function changes what the lock guards | `[THR-19]` |
| `E8001` | GPU layout does not match the CPU layout | `[GPU-10]` |
| `E9001` | invalid manifest, including an unknown key | `[MAN-1]`, `[PRF-3]` |
| `E9002` | no C compiler found, or it failed; the help names the host's remedy | `[BLD-FFI-1]` |
| `E9003` | invalid command line | `[CLI-1]` |
| `E9010` | `[lints]` names a lint the compiler does not define | `[MAN-3]` |
| `E9011` | the toolchain cannot disable floating-point contraction | `[CG-C-11]` |
| `E9013` | invalid `[ffi]` manifest section | `[TCB-1]` |
| `E9014` | the C toolchain cannot emit stack probes for a frame larger than a guard page | `[RT-12]`, `[DIA-6a]` |
| `E9020` | translation units of one target disagree on an inherited flag | `[BLD-FFI-1a]` |
| `E9021` | C++ standard library and CRT heap could not be determined | `[BLD-FFI-1b]` |
| `E9030` | hot reload refused | `[HR-18]`, `[HR-44]`, `[HR-45]`, `[HR-46]`, `[HR-47]` |
| `E9031` | invalid `reload` value | `[MAN-7]` |
| `E9034` | invalid `max_instantiations` value | `[MONO-3]` |
| `E9035` | packages in one process disagree about the object-header layout | `[HR-12a]` |
| `E9036` | a reloadable package may not link the runtime statically | `[HR-29]` |
| `E9037` | reload ABI mismatch on image load | `[ABI-1]` |
| `E9040` | two generic instances hash to one symbol | `[MNG-1]` |
| `E9041` | the toolchain cannot honour `@fastmath` or `@fp(…)` for a function | `[TYP-9c]` |
| `L1001` | unused binding | `[LNT-1]` |
| `L1002` | assignment declares a new binding | `[LNT-2]` |
| `L2001` | unnecessary `copy of` | `[LNT-6]` |
| `L2002` | large `Copy` value passed by value | `[LNT-6]` |
| `L2003` | fallible construction where a total one exists | `[RNG-3a]` |
| `L2004` | `gen fn` with no `yield` | `[LNT-4]` |
| `L2005` | `@noreload` function calls a reloadable one in a loop | `[LNT-5]` |
| `L3002` | a view kept in use longer than necessary | `[LNT-6]` |
| `L3010` | `unsafe` block larger than necessary | `[UNS-3]` |
| `L3015` | an `unsafe fn` with no `@safety` | `[UNS-7]` |
| `L3016` | `@safety` text still reads `TODO` | `[UNS-7]` |
| `L3017` | a loop of owners found leaked at exit (made through `unsafe` or foreign code) | `[WK-4]` |
| `L4001` | allocation in a hot loop | `[LNT-6]` |
| `L4002` | dynamic dispatch on a final type | `[LNT-6]` |
| `L4003` | large `Array[int]`/`Array[float]` in a hot loop whose values fit 32 bits | `[LNT-6]` |
| `L5001` | `unsafe extern` declaration with no contract | `[LNT-6]` |
| `L5002` | conversion at the FFI boundary copies | `[LNT-6]` |
| `L7001` | lock held across a call that may block | `[LNT-6]` |
| `W1002` | binding shadows an enum variant of the same name | `[GRM-12]` |
| `W1003` | a package module shadows the standard module of the same name | `[MOD-3]` |
| `W2015` | float literal has more digits than its type keeps | `[LEX-17a]` |
| `W2016` | `debug_assert` argument has side effects | `[PAN-1]` |
| `W2091` | unreachable match arm | `[CTL-5]` |
| `W2190` | unused `Result`, `check_disjoint`'s included | `[ERR-5]` |
| `W2192` | use of a deprecated item | `[VER-3]` |
| `W2220` | instantiation ceiling exceeded | `[MONO-3]` |
| `W3012` | unsafe block with no SAFETY note | `[UNS-8]` |
| `W3013` | a `# SAFETY:` note on an `unsafe fn` line, where `@safety` says it | `[UNS-8]` |
| `W5001` | function-like macro ignored | `[FFI-6]` |
| `W5002` | declaration not imported (unsupported convention or construct) | `[FFI-20a]` |
| `W5031` | C++ declaration skipped: the header could not be parsed | `[FFI-20a]` |
| `W5033` | `&&`-qualified member skipped | `[FFI-40]` |
| `W5034` | anonymous-namespace entity skipped | `[FFI-41]` |
| `W5050` | unbacked or stale grade | `[TCB-1]` |
| `W5054` | unexercised foreign fact | `[FFI-37]` |
| `W9030` | reload requires a restart | `[HR-18]` |

**Retired codes.**

| Code | Why |
|---|---|
| `E0007` | no lifetime syntax exists; a stray `'` is an unterminated character literal (`E0008`) |
| `E1040` | glob imports are allowed from any module (`[MOD-8]`) |
| `E2224` | folded into `E2225`: `migrate_from` sees a read-only view |
| `E2227` | folded into `[HR-35]`: foreign failure becomes `ReloadError.Foreign` |
| `E2228` | folded into `E2020`: a lambda parameter whose word differs from the function type it is given to |
| `E2229` | a generator method's result views its object; a change of it while the generator will still be used is `E3020` or `E3021` |
| `E3012` | folded into `E3013`: a field of a class object is taken with `mem.take` or `mem.replace` |
| `E3025` | what every method changes and reads is worked out from its body, so no method claims all of its object |
| `E3030` | folded into `E3013`: a function value's own capture given away where it may run twice |
| `E3041` | a hand-over in a loop's later round is `E3040` |
| `E3042` | folded into `E3040`: a part handed over on its own, then its owner used whole |
| `E3061` | folded into `E3063`: a value made in a memory manager's instance kept where it would outlive it |
| `E3064` | multi-region view structs are legal |
| `E3065` | a result points into every view argument it may come from, or into what `from` names; nothing is left to infer |
| `E3080` | no check of exclusive access runs while a program runs; overlapping uses are `E3021` and `E3022` |
| `E4071` | an undeclared `extern` is `Nondet` (`[DET-2]`), reported as `E4070` |
| `E4073` | generator frames never allocate (`[CORO-1]`) |
| `E6002` | compile-time evaluation is deterministic by construction (`[CT-4]`) |
| `E7001` | `@sync` is gone: a value shared between threads is `thread_shared`, its changing fields `guarded by` a lock or `Atomic` |
| `E7003` | `@sync` is gone (as `E7001`); a `thread_shared` value changed directly is `E7014` |
| `E7005` | a value reaching a task is `E7007`, `E7008` or `E7015` |
| `E7020` | the ECS has no scheduler (EnTT's design): systems in one parallel run are gone |
| `E9012` | reserved |
| `E9032` | reserved |
| `E9033` | a `release` build ignores `reload` with a note (`[MAN-7]`, ODR-118) |
| `L3001` | a loop of owners is refused when compiling (`E3032`) |
| `L3011` | `RefCell` is gone |
| `L3013` | no access is held while a program runs |
| `L3014` | a result points into every view argument it may come from (as `E3065`) |
| `L3018` | the reason category is part of the `# SAFETY(…):` note and optional (`[UNS-8]`) |
| `L3019` | an object is never deinitialised before its last owner ends (`[RC-3]`, ODR-063) |
| `W0001` | a dangling doc comment is discarded in silence (`[LEX-11]`) |
| `W2111` | `virtual` in a class that cannot be subclassed is refused (`E2112`) |
