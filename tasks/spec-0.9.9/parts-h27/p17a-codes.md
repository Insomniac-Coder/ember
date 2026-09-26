
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
| `E0100` | unexpected token | `[GRM-2]` |
| `E0101` | unclosed delimiter | `[LEX-6]` |
| `E0102` | membership test chained (`a in b in c`) | `[GRM-23]` |
| `E0103` | match arms mix statement and expression form | `[GRM-10]` |
| `E0104` | attribute or construct not admitted here | `[ATT-1]`, `[PHIL-12]` |
| `E0105` | `;` is not a statement separator | `[GRM-18]` |
| `E0106` | a multi-statement closure cannot be written inside brackets | `[GRM-17]` |
| `E0107` | a jump expression may not be an operand | `[GRM-16]` |
| `E0108` | attribute is not permitted on this statement | `[GRM-20]` |
| `E0109` | `owned` is not permitted in expression position | `[GRM-15]` |
| `E0110` | function declared without a body outside an interface, extern block or abstract class | `[GRM-33]` |
| `E0111` | `ref` of an expression that is not a place | `[GRM-36]` |
| `E0900` | construct not implemented by this compiler | `[PHIL-12]`, `[CLI-19]` |
| `E0901` | construct this specification leaves unspecified | `[PHIL-12]` |
| `E1010` | cannot find name in this scope | `[MOD-3]` |
| `E1020` | name is already declared in this block | `[GRM-4]` |
| `E1021` | name is not linked in this build | `[BLD-11]` |
| `E1030` | two items with the same name in one scope | `[TYP-26]` |
| `E1031` | a name bound by two glob imports is used | `[MOD-8]` |
| `E1041` | import cycle between packages | `[MOD-4]` |
| `E1050` | field is read-only outside its module | `[MOD-7]` |
| `E1051` | `read` visibility is valid on fields only | `[MOD-7]` |
| `E1052` | item, field or constructor not visible here | `[MOD-2]` |
| `E1060` | no module of that name | `[MOD-3]` |
| `E1061` | no item of that name in the module | `[MOD-3]` |
| `E2010` | literal does not fit its type | `[LEX-16]` |
| `E2011` | negative literal index | `[TYP-31]`, `[LEX-24]` |
| `E2020` | mismatched types | `[TYP-4]` |
| `E2030` | `@view` on a type that is not a view | `[TYP-34]` |
| `E2031` | `@borrows` names a parameter the result cannot borrow from (a borrowed `Copy` parameter, or an `owned` one that is not a reference or view) | `[LT-1a]` |
| `E2035` | condition must be `bool` | `[CTL-0]` |
| `E2036` | this pattern always matches | `[GRM-19]` |
| `E2040` | unsatisfied interface bound | `[TYP-17]` |
| `E2041` | overlapping `extend` implementations | `[TYP-19]` |
| `E2042` | `Float` is implemented only by `f32` and `f64` | `[STD-27]` |
| `E2043` | an associated type's default leads back to itself | `[IFC-4]` |
| `E2050` | interface is not `dyn`-compatible | `[TYP-22]` |
| `E2060` | cannot infer type | `[TYP-23]` |
| `E2061` | lambda parameter types cannot be inferred here | `[TYP-23]` |
| `E2062` | ambiguous type | `[TYP-23]` |
| `E2070` | ambiguous interface method | `[TYP-24]` |
| `E2071` | `SoA[T]` of a type that is not a struct | `[SOA-1]` |
| `E2072` | no method or field of that name (with the Ember name for a Python one; shape N13 for an undeclared field) | `[STD-13]` |
| `E2073` | `len` of a string (Python counts characters, `s.len()` bytes) | `[STD-26]` |
| `E2080` | `@derive(Copy)` on a type with a field that is not Copy | `[STR-3]` |
| `E2090` | non-exhaustive match | `[ENM-2]` |
| `E2100` | field read before it is initialised | `[CLS-2]` |
| `E2101` | derived class with a field that has no default needs an `init` | `[CLS-10]` |
| `E2102` | derived `init` uses `self` or an inherited field before `super.init`, or leaves an own field unassigned | `[CLS-11]` |
| `E2110` | override of a method that is not virtual | `[CLS-4]` |
| `E2111` | a method that replaces an inherited virtual one without `override` | `[CLS-4]` |
| `E2120` | inherent extension of a type from another package | `[IFC-2]` |
| `E2130` | a `const` of a type that owns heap memory; use a `static` | `[TYP-1]` |
| `E2131` | array length must be a constant | `[CT-1]` |
| `E2140` | cannot assign to a value expression | `[EXP-5]` |
| `E2150` | `is` on an operand that is neither a handle, a reference nor an `Option` | `[EXP-9]` |
| `E2151` | integer `**` with a constant negative exponent | `[TYP-30]` |
| `E2160` | control flow cannot leave a `defer` block | `[CTL-7]` |
| `E2170` | reference to a field of a packed struct | `[LAY-2]` |
| `E2172` | cannot index with a type | `[GRM-8b]` |
| `E2173` | not a type or const-generic argument | `[GRM-8b]` |
| `E2180` | `?` outside a function returning `Option` or `Result` | `[ERR-2]` |
| `E2181` | `?` on an `Option` in a `Result` function, or the reverse | `[ERR-2]` |
| `E2182` | a function that returns a value can reach the end of its body | `[FN-10]` |
| `E2200` | type has infinite size (a recursive value type without `Box`) | `[TYP-14]` |
| `E2210` | a value of one range type where another was expected | `[RNG-2]` |
| `E2211` | constant outside the target range type | `[RNG-3]` |
| `E2212` | range endpoints are not constants of the representation, or are inverted | `[RNG-1]` |
| `E2213` | invalid `in` clause on a type alias | `[RNG-1]` |
| `E2214` | arithmetic between two distinct range types | `[RNG-5]` |
| `E2215` | range-typed value constructed outside the permitted set | `[RNG-10]` |
| `E2220` | `yield` outside a `gen fn` | `[CORO-2]` |
| `E2221` | borrow held across a `yield` | `[CORO-6]` |
| `E2222` | coroutine where an ordinary function is required | `[CORO-10]` |
| `E2223` | `@never_specialize` on a generic that is not shareable | `[MONO-7]` |
| `E2225` | `migrate_from` may not panic | `[HR-35]` |
| `E2226` | `in` on a type that does not implement `Contains` | `[STD-8]` |
| `E2228` | callable parameter-mode mismatch (shape B15) | `[CLO-3]` |
| `E2229` | class generator method takes `mut self` or holds an access across `yield` | `[CORO-12]` |
| `E2230` | a name assigned in every branch at different types | `[CTL-10]` |
| `E2231` | `yield` while a `@must_drop` value is live | `[CORO-13]` |
| `E2240` | `/` on two integers | `[TYP-28]` |
| `E2250` | format spec does not apply to the value's type | `[LEX-19]` |
| `E2260` | `some` type outside a return position | `[TYP-32]` |
| `E2261` | returns of a `some` function have different types | `[TYP-32]` |
| `E3010` | cannot move out of a field of a type with `drop` | `[EXP-6]` |
| `E3011` | cannot move out of an array or span element | `[EXP-6]` |
| `E3012` | cannot move out of a class field | `[EXP-6]` |
| `E3013` | cannot move out of a reference | `[EXP-6]` |
| `E3014` | scope binding may not be moved | `[THR-5]` |
| `E3015` | value whose drop is required may not be leaked | `[THR-6]` |
| `E3016` | `self` escapes its own drop | `[CLS-7a]` |
| `E3020` | iterable is mutated while the loop borrows it | `[CTL-2]` |
| `E3021` | a shared and a mutable borrow overlap | `[BRW-1]` |
| `E3022` | two mutable borrows of the same place | `[BRW-1]` |
| `E3023` | two writers of one value (shape B4) | `[BRW-1]` |
| `E3024` | a struct field would borrow another field of the same struct (shape B5) | `[TYP-15]` |
| `E3025` | a method takes all of `self` (shape B8) | `[BRW-10]` |
| `E3026` | a closure outlives what it captures (shape B9) | `[CLO-4]` |
| `E3027` | a `mut` argument is not a mutable place (shape B10) | `[FN-2a]` |
| `E3030` | closure would move a captured value out | `[CLO-2]` |
| `E3040` | use of moved value | `[OWN-3]` |
| `E3041` | value moved in a previous loop iteration | `[OWN-4]` |
| `E3042` | partial move then use of the whole value | `[EXP-6]` |
| `E3050` | use of an uninitialised or moved place | `[BRW-7]` |
| `E3060` | borrowed value does not live long enough | `[LT-3]`, `[BRW-8]` |
| `E3061` | arena allocation cannot outlive its arena | `[LT-4]` |
| `E3062` | returned view does not derive from a parameter (shape B6) | `[LT-1]` |
| `E3063` | stored view may not outlive its source | `[TYP-15]` |
| `E3065` | multi-region result provenance cannot be inferred | `[LT-35]` |
| `E3070` | `drop` cannot be called explicitly | `[DRP-1]` |
| `E3080` | overlapping access through the same handle | `[EXC-3]` |
| `E3090` | arena allocation of a type that needs `drop` | `[ARN-3]` |
| `E3095` | disjointness is not establishable for these operands | `[DSJ-1]` |
| `E3096` | arena is scoped here | `[ARN-6]` |
| `E3100` | this operation requires an `unsafe` block | `[UNS-1]` |
| `E3105` | `UnsafeCell` in `@static_safe` code | `[UNS-10]` |
| `E4001` | `@noalloc` function reaches an allocation | `[EFF-5]` |
| `E4002` | `@nosync` function reaches a synchronising operation | `[EFF-5]` |
| `E4003` | `@noblock` function reaches a blocking operation | `[EFF-5]` |
| `E4010` | implementation does not satisfy the interface's contract | `[EFF-2]` |
| `E4020` | `@simd(assert)` loop did not vectorise | `[SIMD-2]` |
| `E4030` | `@static_safe` function performs a dynamically checked access | `[EFF-12]` |
| `E4040` | `@nopanic(explicit)` function reaches a panic | `[EFF-17]` |
| `E4041` | `@noio` function reaches I/O | `[EFF-20]` |
| `E4042` | `@nolock` function acquires a lock | `[EFF-21]` |
| `E4070` | `@deterministic` function reaches a nondeterministic operation | `[DET-1]` |
| `E4072` | `@fastmath` or `@fp(contract)` inside `@deterministic` | `[DET-5]` |
| `E5001` | imported type layout does not match | `[FFI-5]` |
| `E5002` | foreign call requires `unsafe`: its contract is incomplete, or `safe fn` on a function whose contract is | `[FFI-2]`, `[FFI-10]` |
| `E5010` | overlay does not match the C declaration | `[FFI-12]` |
| `E5011` | one C identity imported with two different layouts | `[FFI-30]` |
| `E5012` | pointer contract has no count | `[FFI-11]` |
| `E5014` | two packages request the same implementation macro | `[FFI-29]` |
| `E5015` | type may not cross the boundary | `[FFI-31]` |
| `E5016` | `_Atomic` layout does not match `Atomic[T]` | `[FFI-8]` |
| `E5017` | struct with a flexible array member is unsized | `[FFI-8]` |
| `E5018` | `va_list` may not be constructed | `[FFI-8]` |
| `E5020` | `str` is not NUL-terminated | `[FFI-15]` |
| `E5030` | `std::function` as a parameter | `[FFI-17a]` |
| `E5031` | C++ parameter cannot be mapped | `[FFI-17]` |
| `E5032` | opaque C++ type may not be constructed | `[FFI-32]` |
| `E5034` | unsupported C++ construct | `[FFI-17]` |
| `E5040` | capturing closure passed where a C function pointer is expected | `[FFI-21]` |
| `E5041` | retained callback needs a release function in the overlay | `[FFI-21]` |
| `E5050` | a foreign fact claims a grade whose evidence is absent or stale | `[TCB-1]` |
| `E5051` | a foreign callee retains a pointer its contract does not declare `retained` | `[FFI-35a]` |
| `E5052` | `adopt` on a handle whose overlay declares `adopt = false` | `[FFI-36b]` |
| `E5053` | an instrumented run contradicted a declared foreign effect | `[FFI-37]` |
| `E5054` | range type in a foreign signature | `[RNG-10]` |
| `E5055` | an Ember generic may not instantiate a C++ template | `[FFI-17b]` |
| `E5056` | override of a C++ virtual the overlay does not name | `[FFI-39]` |
| `E5057` | the overlay names a method that is not virtual in the header | `[FFI-39]` |
| `E5058` | C++ base has no default constructor and no declared `init` | `[FFI-39]` |
| `E5059` | C++ trampoline base has no virtual destructor | `[FFI-39]` |
| `E5060` | upcast in a context that cannot hold the `Retained` token | `[FFI-39]` |
| `E5061` | imported C++ function has no exception policy and none can be derived | `[FFI-24]` |
| `E5062` | contradictory C++ exception policies | `[FFI-43]` |
| `E5063` | foreign bytes reach `str` without validation | `[TXT-2]` |
| `E5064` | interior NUL in a value converted to `cstr` | `[FFI-15]` |
| `E5065` | `Shared`/`Weak` and `CppShared`/`CppWeak` do not interconvert | `[WK-14]` |
| `E5066` | an overlay marks a C++ function both `noexcept` and throwing | `[FFI-43]` |
| `E5067` | an overlay that states facts is not declared `unsafe overlay` | `[GRM-35]`, `[TIER-1]` |
| `E5090` | inline assembly is not supported by the C backend | `[UNS-6]` |
| `E6001` | comptime evaluation exceeded its limits, or a constant's value depends on itself | `[CT-3]` |
| `E6004` | panic during compile-time evaluation | `[CT-7]` |
| `E6005` | `comptime(e)` refers to a run-time local | `[CT-6]` |
| `E6010` | operation not available at compile time | `[CT-2]` |
| `E7001` | `@sync` class with a field that is not `Sync`, or a non-`@sync` base or derived class | `[THR-1]` |
| `E7002` | `static` of a type that is not `Sync` | `[STA-1]` |
| `E7003` | write to a field of a `@sync` class after `init`, or a `mut self` method on one | `[THR-1]` |
| `E7004` | value crossing a thread boundary is not `Send` | `[THR-10]`, `[THR-11]`, `[FFI-22]` |
| `E7005` | value shared with a task is not `Sync` | `[THR-11]` |
| `E7006` | memory order an atomic operation does not support | `[THR-14]` |
| `E7010` | parallel loop writes to a shared place | `[PAR-2]` |
| `E7011` | parallel loop has a loop-carried dependency | `[PAR-2a]` |
| `E7020` | systems in one parallel run have conflicting access sets | `[ECS-4]` |
| `E8001` | GPU layout does not match the CPU layout | `[GPU-10]` |
| `E9001` | invalid manifest, including an unknown key | `[MAN-1]`, `[PRF-3]` |
| `E9002` | no C compiler found, or it failed; the help names the host's remedy | `[BLD-FFI-1]` |
| `E9003` | invalid command line | `[CLI-1]` |
| `E9010` | `[lints]` names a lint the compiler does not define | `[MAN-3]` |
| `E9011` | the toolchain cannot disable floating-point contraction | `[CG-C-11]` |
| `E9013` | invalid `[ffi]` manifest section | `[TCB-1]` |
| `E9020` | translation units of one target disagree on an inherited flag | `[BLD-FFI-1a]` |
| `E9021` | C++ standard library and CRT heap could not be determined | `[BLD-FFI-1b]` |
| `E9030` | hot reload refused | `[HR-18]` |
| `E9031` | invalid `reload` value | `[MAN-7]` |
| `E9033` | `reload` is forbidden in `shipping` | `[PRF-2]` |
| `E9034` | invalid `max_instantiations` value | `[MONO-3]` |
| `E9035` | packages in one process disagree about the object-header layout | `[HR-12a]` |
| `E9036` | a reloadable package may not link the runtime statically | `[HR-29]` |
| `E9037` | reload ABI mismatch on image load | `[ABI-1]` |
| `E9040` | two generic instances hash to one symbol | `[MNG-1]` |
| `E9041` | the toolchain cannot honour `@fastmath` or `@fp(…)` for a function | `[TYP-9c]` |
| `L1001` | unused binding | `[LNT-1]` |
| `L1002` | assignment declares a new binding | `[LNT-2]` |
| `L2001` | unnecessary clone | `[LNT-6]` |
| `L2002` | large `Copy` value passed by value | `[LNT-6]` |
| `L2003` | fallible construction where a total one exists | `[RNG-3a]` |
| `L2004` | `gen fn` with no `yield` | `[LNT-4]` |
| `L2005` | `@noreload` function calls a reloadable one in a loop | `[LNT-5]` |
| `L3001` | potential reference cycle | `[WK-1]` |
| `L3002` | borrow held longer than necessary | `[LNT-6]` |
| `L3010` | `unsafe` block larger than necessary | `[UNS-3]` |
| `L3011` | `RefCell` guard held across a call | `[CELL-7]` |
| `L3013` | long-term access held across a call | `[EXC-7]` |
| `L3014` | return region is the intersection of several parameters | `[LT-1b]` |
| `L3015` | undocumented unsafe obligation | `[UNS-7]` |
| `L3016` | `@safety` text still reads `TODO` | `[UNS-7]` |
| `L3017` | reference cycle detected | `[WK-4]` |
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
| `W2111` | `virtual` has no effect in a final class | `[CLS-4]` |
| `W2190` | unused `Result` | `[ERR-5]` |
| `W2220` | instantiation ceiling exceeded | `[MONO-3]` |
| `W3012` | unsafe block with no SAFETY note | `[UNS-8]` |
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
| `E3064` | multi-region view structs are legal |
| `E4071` | an undeclared `extern` is `Nondet` (`[DET-2]`), reported as `E4070` |
| `E4073` | generator frames never allocate (`[CORO-1]`) |
| `E6002` | compile-time evaluation is deterministic by construction (`[CT-4]`) |
| `E9012` | reserved |
| `E9032` | reserved |
| `L3018` | the reason category is part of the `# SAFETY(…):` note and optional (`[UNS-8]`) |
| `L3019` | an object is never deinitialised before its last owner ends (`[RC-3]`, ODR-063) |
| `W0001` | a dangling doc comment is discarded in silence (`[LEX-11]`) |
