# K08-export-arguments-from-c (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K08-export-arguments-from-c

**title:**
What may an exported function trust about the arguments its C host passes?

### members
#### [1]
J8-09

**primary:**
J8-09

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
8

**notes:**
A NULL read, or a call through any address, in safe code. A reuses FFI-56 and E5054's reason.

## questions
### [1]
**id:**
J8-09

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.55

**reason:**
FFI-31b (p16-ffi.md 651) lets an exported signature carry `CStrView` and `extern "C" fn`. CStrView has a non-null niche (ODR-080's `Option[CStrView]` is how null is said). The `@unsafe(export)` `# SAFETY:` note vouches only for the symbol name (FFI-26, 629-634). FFI-56 (590-596) makes every function pointer "that comes from C ... an argument C passes to a callback" an `unsafe extern "C" fn`, because "nothing tells the compiler what the function it points to does". An export's parameter comes from C in exactly that way. E5054 (652-656) refuses `char` because "C may pass any `uint32_t`", and the same reason applies to a null `const char*`. No rule or verdict says how an export's C-supplied parameters are typed.

**title:**
What may an exported function trust about the arguments its C host passes?

**problem:**
A game module exports `game_log(msg, done)`, which the host calls with a C string and a completion function. Ember lets the export declare a non-null string view and a safe function pointer, so its body reads the string and calls the pointer in safe code. The host may pass NULL, or any address. The specification already treats a function pointer that arrives from C as unsafe to call, and refuses `char` because C may pass a value it cannot hold, but neither rule reaches an export's own parameters.

**example:**
from std.ffi import CStrView

# SAFETY: the host defines no other symbol named `game_log`
@unsafe(export("game_log"))
pub extern "C" fn game_log(msg: CStrView, done: extern "C" fn(i32)) -> i32:
    match msg.to_str():          # reads bytes at `msg`: the host may pass NULL
        Ok(text):
            println(text)
            done(1)              # a safe call through whatever address the host passed
        Err(_):
            done(0)
    return 0

**spec_today:**
FFI-31b (p16-ffi.md 651): "`@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may." FFI-56 (590-596): "A function pointer that comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C passes to a callback — imports as `unsafe extern "C" fn(…)` ... nothing tells the compiler what the function it points to does with its arguments". FFI-31b (652-654): "`char` may not appear in a foreign signature, an export's or a hand declaration's ... (`E5054`) ...: C may pass any `uint32_t`". FFI-26 (629-634): the `# SAFETY:` note says "why nothing else defines that name".

#### options
##### [1]
**letter:**
A

**summary:**
What C passes to an export follows the rules for what C hands back: a function pointer is `unsafe extern "C" fn`, called in `unsafe:`, and a C string is `Option[CStrView]`, where NULL is `None`.

**code:**
@unsafe(export("game_log"))
pub extern "C" fn game_log(msg: Option[CStrView], done: unsafe extern "C" fn(i32)) -> i32:
    if msg exists as m:
        match m.to_str():
            Ok(text):
                println(text)
                # SAFETY: the host's `done` takes one int and runs no Ember code
                unsafe:
                    done(1)
            Err(_):
                return 1
    return 0

**outcome:**
This compiles. The example as written is refused: `done: extern "C" fn(i32)` and `msg: CStrView` each get the error giving these types. A NULL from the host becomes `None`, not a crash.

##### [2]
**letter:**
B

**summary:**
As today. The types the export writes are the host's promise, and the `# SAFETY:` note above `@unsafe(export(...))` vouches for every argument as well as the symbol name.

**code:**
# SAFETY: the host defines no other symbol named `game_log`, passes a NUL-terminated string and a valid `done`
@unsafe(export("game_log"))
pub extern "C" fn game_log(msg: CStrView, done: extern "C" fn(i32)) -> i32:

**outcome:**
The example compiles. A host that passes NULL makes Ember read address 0.

#### recommendation
**letter:**
A

**reason:**
A value arriving from C gets the same type whichever way it arrives (a callback's argument, a struct field, an export's parameter), and a value C may send that the Ember type cannot hold is refused, as `char` already is. The NULL test is one compare.

**checklist:**
5 and 6 (one rule with FFI-56 and E5054), 8, 3 (one compare), 11 (`Option[CStrView]` and `unsafe extern "C" fn` reused)

## group
**gid:**
P04

**principle:**
Safe code relies only on what the compiler checks or on a fact someone vouched for in unsafe code: every fact the compiler cannot check (what C does with a pointer, what C passes in, where a C++ reference points, whether a foreign global changes, what a header runs before main, what an unsafe body does when safe code reaches it, whether a wrapped C object is thread-safe) is refused in safe code until vouched, and one vouching spelling reaches every kind of item and every type.

**shared_cause:**
The rules that let safe code act on an unchecked fact are written per construct. FNI-10's safe-fn list checks parameters but not pointers inside a @layout(c) struct. FFI-31b types an export's C arguments as if checked. FFI-40 derives a C++ reference result's lifetime from a body that does not exist. FFI-41 exempts C++ const globals, ignores mutable members, and leaves C no way to assert. FFI-51 asks for unsafe on C imports only. UNS-1 does not reach drop or a safe virtual call landing in an unsafe override. TYP-43 vouches Send only, for structs and enums only. @ffi(handle) has no meaning. So an unchecked fact reaches safe code by an unlisted route, or a fact that could be vouched has no spelling.

### members
#### [1]
J8-08

#### [2]
J8-09

#### [3]
J8-06

#### [4]
J5-16

#### [5]
J8-07

#### [6]
J8-11

#### [7]
J7-06

#### [8]
J2-12

#### [9]
J8-14

### clusters
#### [1]
K07-raw-pointer-struct-safe-call

#### [2]
K08-export-arguments-from-c

#### [3]
K09-cpp-reference-result

#### [4]
K11-unsafe-drop-override

#### [5]
K15-foreign-const-globals

#### [6]
K16-unsafe-import-cpp

#### [7]
K36-unsafe-extend-vouching

#### [8]
K46-ffi-handle-meaning

### answered
### verdicts_involved
#### [1]
31: a forgotten mark must refuse, not crash

#### [2]
60: anything that can break safety only inside unsafe

#### [3]
118: an unsafe fn's body is unsafe

#### [4]
119: anything inside unsafe code is unsafe

#### [5]
120: a C function in unsafe extern carries @safety

#### [6]
125: C++ too, members included

#### [7]
126: lambdas carry @safety

#### [8]
130: overlay unsafe fn with @safety

#### [9]
132: an unwritten attribute never falls back to an inferred fact

#### [10]
133: hand-written declarations spell unsafe the same way

#### [11]
136: safe fn or unsafe fn, never plain fn

#### [12]
137: a result's view is read from the callee's code (C++ has none)

#### [13]
165: thread_shared needs ThreadShared

**other_places:**
C function pointers arriving from C (FFI-56), C callback arguments, out-parameters and C struct fields in general, unions, variadic C calls (E5013), C strings returned, extern statics C writes, C++ mutable members and const_cast, C++ virtual calls and exceptions, [[clang::lifetimebound]], header-only C libraries and static initialisers in any compiled foreign code, another package's overlays (FFI-30c), Send/ThreadShared/other markers for every kind of type, an unsafe implementation of a safe interface method reached through dyn, interfaces whose correctness memory safety relies on (Hash/Eq for maps), unsafe code at compile time (P17), GPU bindings.

**severity:**
high

**rank:**
4

### depends_on
**notes:**
J8-08 and J8-14 both rewrite XVI.4's Vulkan overlay: write it once, after both answers. J8-08's example line ref_to_ptr(ref mut x) takes P07's spelling (J2-11). J5-16 A also needs the missing rule that an interface implementation's signature must match. J7-06 is P01's only way out for a thread-safe C wrapper under verdict 165. J8-06 B is verdict 132's lesson and verdict 137's reading. J8-07 A has to agree with STA-1 and the rule that replaced ODR-163. ODR-111 (a delegation ruling) once allowed Send|Sync; ThreadShared was dropped without a verdict.

## conflicts_naming_a_member
## verdicts_involved
**31:**
JOB.md 267-278: 31. **A C call counts as possibly running the callbacks C keeps, unless its declaration says it never
    calls back** (area E, reading 3; AN-78). The compiler cannot see inside C, so every C function
    counts as possibly running the kept callbacks, and a call to it makes the changes their first lines
    list: with `register_cb(fn() with owns game changes Game: game.add_point())` kept,
    `for name in game.names: poll_events()` is refused (E3020, "call `poll_events()` before or after the
    loop"); with a kept callback that changes nothing (`fn(): println("event")`) it is allowed. A C
    function that never calls back is marked on its declaration,
    `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`; a missing mark only refuses a loop, with a
    message naming the mark. Registering a kept callback counts as making its changes (verdict 30), so
    `setup` and `main` say `changes Game`. Owner: "B", over (c) `@ffi(runs_callbacks)` on the functions
    that do call back (a forgotten mark lets a crash through). A (d) that guessed by `extern` block was
    withdrawn (it treated `unregister` and `register_cb` as calling back and barely differed from (c)).

**60:**
JOB.md 479-484: 60. **Anything that can break a safety promise is possible only inside an `unsafe` block** (the owner, after
    `mem.forget` turned out to need no `unsafe`: "for unsafe stuff it should only be possible in an unsafe
    block"). The inventory of escape hatches and leaky features (planned next, for the owner's approval)
    checks every one against this: today's marked ones (`unsafe` blocks and `unsafe fn`, raw pointers,
    `assume_disjoint`, the `@ffi(...)` promises written in an `unsafe extern "C":` block) and every
    feature that is not marked but can leak, skip a check, or reach memory the rules cannot see.

**118:**
JOB.md 997-1002: 118. **An `unsafe fn` is unsafe as a whole; `safe:` marks code inside it that is checked as safe code**
    (reverses `[UNS-12]`, which came from the redesign's AN-79 and was never put to the owner). Owner:
    "WHO FUCK SAID THAT AN UNSAFE FUNCTION NEEDS TO MARK ITS UNSAFE CODE? THE FUNCTION LITERALLY SAYS THAT
    IT IS UNSAFE", "the whole unsafe function has to be considered unsafe unless someone marks a piece of
    code safe", "if you want it to be treated as safe you have to write it in safe block"; `safe:` as the
    spelling: "yes".

**119:**
JOB.md 1003-1007: 119. **Anything inside unsafe code is unsafe, functions and lambdas included.** A function written inside
    an `unsafe:` block (or an `unsafe fn` body) is an `unsafe fn`: its body is unsafe, calling it needs
    `unsafe`, its value is an `unsafe fn(…)`, and it carries `@safety`. Owner: "apply my ruling anything
    inside unsafe block is considered unsafe by default even functions"; lambdas, "closure is also a type
    of function right?" then "yes cover this too"; `@safety` on such a function, "yes".

**120:**
JOB.md 1008-1009: 120. **A C function declared in an `unsafe extern` block carries `@safety`, and gets `L3015` without
    it.** Owner: "yes that is my decision for consistency sake and it makes sense".

**125:**
JOB.md 1027-1031: 125. **A function imported from C says what its callers must make true.** Owner (2026-10-29): "Yeah for
    functions imported from C should require @safety as C code can't be trusted as safe". A function
    `import c` writes that its overlay entry leaves unsafe to call carries `@safety` in that entry; a
    call of one without it is `L3015`, as for any `unsafe fn` without `@safety` ([UNS-7]). C++ too
    (owner, asked: "yes C++ too, same rule"): a function `import cpp` writes, members included.

**126:**
JOB.md 1032-1035: 126. **A lambda carries `@safety` like a named function.** The lead had exempted lambdas written in
    unsafe code (no line to write it on); owner, shown that today's compiler refuses
    `set = @safety("…") fn(v: int) => p.write(v)`: "yes build it". `@safety` is written just before the
    lambda, and one without it is `L3015` ([UNS-7], [GRM] `lambda`).

**130:**
JOB.md 1045-1047: 130. **An overlay line for a C function unsafe to call is `@safety("…")` above `unsafe fn name(…)`.**
    Owner: "why is it not unsafe fn lib_step(x)", then "yes unsafe fn, replace -> unsafe": Ember's own
    spelling; the `unsafe` contract word is gone ([GRM-35], [FFI-2], [FFI-11], [CLI-5]).

**132:**
JOB.md 1052-1055: 132. **An overlay entry without `@ffi(const)` makes its method one that changes its object**, whatever
    the importer proves from the header's body; `@ffi(const)` keeps it const. Owner, shown the fix that
    kept the importer's reading: "NO", then, of (1) not const / (2) another spelling: "1" ([FFI-40a];
    reverses the in-place correction of Hardened_5, so a new hardening).

**133:**
JOB.md 1056-1060: 133. **A hand-written C declaration unsafe to call is `@safety("…")` above `unsafe fn`, as in overlays**:
    `@ffi(unsafe)`, `param(p, unsafe)` and `result(unsafe)` go, each refused with the line to write
    (and rewritten by `ember fmt --migrate`); `@ffi(…)` keeps its other words (`link_name`, …). Owner:
    "yes" (the overlay spelling here too), then, shown a hybrid (`@ffi(param(p, unsafe))` on an
    `unsafe fn`) beside the plain form: "yes, go with your suggestion" ([FFI-11], [FFI-10], [UNS-7]).

**136:**
JOB.md 1067-1071: 136. **A hand-written C declaration says whether it is safe to call: `safe fn` or `unsafe fn`; a plain
    `fn` in an `unsafe extern` block is refused**, with the line to write ("write `unsafe fn lib_step`").
    Owner, of N23 (2026-10-31, morning), shown `fn lib_step` with the word left out and the two answers
    (accept it as `unsafe fn`, the lead's ruling; refuse it): "yes good" to refusing it, the lead's
    recommendation on a second look (a plain `fn` is safe to call everywhere else in Ember) ([FFI-10]).

**137:**
JOB.md 1072-1080: 137. **What a call's result is a view of is read from the called function's code, for every result type**:
    a plain `StrView`, `Span`, `ref T` as well as a struct, enum or tuple holding views. The caller locks
    exactly what the code shows the result points into (`fn pick(a: Span[int], b: Span[int]) ->
    Span[int]: return a` locks only `a`). What a body may return is unchanged; the first line's rules
    (`[LT-1]` rules 1 to 3) decide a call only where there is no code to read (a function value, a `dyn`
    or `virtual` call, a C function). Owner, of N26 (2026-11-01, morning), after asking why Ember read the
    code of a method returning `Option[StrView]` and not of one returning `StrView`: "why the fuck is
    reading the code an issue if it breaks, if it is breaking it is clearly because some rule was broken
    right?", then, asked "read the code everywhere?": "yes" ([LT-1], [LT-22], [LT-34]).

**165:**
JOB.md 1253-1253: 165. **`thread_shared` is refused where the value is made, when something in it is not safe for several threads to read at once** (`ThreadShared`): `stats = thread_shared FrameStats()` with `draws: RwLock[Cell[int]]` is `E7015` at that line, naming the field and the fix (`draws: Atomic[int]`). Shown as (a) refuse it where it is made, (b) refuse it only where it reaches another thread, recommendation (a). Owner (2026-11-02, gap-sweep group `K02-thread-shared-not-threadshared`): "a".

## allowed_rules
### [1]
FFI-21a

### [2]
FFI-26

### [3]
FFI-31b

### [4]
FFI-56

## rule_text
**FFI-21a:**
p16-ffi.md lines 424-434
* `[FFI-21a]` *(new in 0.9.10)* *(changed in 0.9.15)* **A parameter C passes as a copy.** C passes a function's arguments by
  value. When a function's value becomes an `extern "C" fn` (`[FN-6]`), a plain parameter
  (`[FN-1]`) whose type is not a pointer is, for a call from C, a copy the call makes, which ends when
  the call returns; called from Ember, it is still a view of the caller's value. Where a program turns
  a callable type into `extern "C" fn`, a function whose value has that type MUST NOT return a raw
  pointer into such a parameter: one made from a view of it or of a part of it
  (`ref_to_ptr(value)`, `ref_to_ptr(value.field)`), passed through copies, conversions and
  aggregates in the function, is `E5042`, shown where it is returned and where the type becomes
  `extern "C" fn`. A value read through the parameter, and a pointer stored in it, are not into the
  copy. A pointer returned from a call to another function is not followed. A function that must give
  C back a pointer it was given takes that parameter as a pointer (`*T`).

**FFI-26:**
p16-ffi.md lines 622-643
* `[FFI-26]` *(changed in 0.9.10)* *(changed in 0.9.15)* `@unsafe(export("symbol"))` gives a function a stable C symbol with the C ABI
  (without a name, the function's own name). `@unsafe(export_table("Name", protocol=N))` on a struct of
  `extern "C" fn` fields exports a constant of that
  struct, a table of function pointers, under the C symbol `Name_vN` (`[ABI-1]`), for hosts that load a
  module through one entry point. A field written with a default, `on_create: extern "C" fn(u64) -> i32
  = door_created`, holds the function its default names; a field without one holds the module's
  function of the field's own name. Either way the function's type is the field's (`E2020`), and a
  field naming no function is `E1010`. A C symbol is one name for the whole process: one that the C
  library, the runtime or another library already defines would be replaced by this function
  everywhere, and the compiler cannot see every name the process links. So both attributes are written
  in their unsafe form, `@unsafe(export("game_on_update"))` and
  `@unsafe(export_table("Name", protocol=N))`, with a `# SAFETY:` note above saying why nothing else
  defines that name (`W3012` when missing); the plain form is `E3100`: "`@export("game_on_update")`
  gives `on_update` a C name the whole process shares, and the compiler cannot see whether the host or
  a library already defines it; say that you vouch: write `@unsafe(export("game_on_update"))` with a
  `# SAFETY:` note above it". A name the compiler knows is taken is refused even so (`E0104`): `main`,
  a name beginning with the runtime's prefix (`[RT-5]`), the module-protocol names of `[FFI-28]`, and
  the functions of the C standard library ("`realloc` is a function of the C standard library, and
  exporting `same_block` under its name would replace it for the whole process, the runtime's own
  allocations included; choose another name"); so is a symbol two Ember functions define. `ember build
  --emit-header` writes `<package>.h` with a prototype for each export, a typedef for each
  `@layout(c)` type they use, and each export's thread contract as a comment.

**FFI-31b:**
p16-ffi.md lines 647-656
* `[FFI-31b]` *(changed in 0.9.15)* No owning Ember value crosses a module boundary: class objects,
  `owns T`, `shares T`, `thread_shared T`, `weakref T`, `String`, `Array`, `Map`, function values and
  any type with a `drop` MUST NOT appear in an exported signature or be reachable through a pointer in
  one (`E5015`), except a `thread_shared` value of a class in a package built with the C++ header
  (`[BLD-FFI-5a]`). `@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may.
  `char` may not appear in a foreign signature, an export's or a hand declaration's, directly or in a
  `@layout(c)` aggregate passed by value or behind a pointer (`E5054`), as a range type may not
  (`[RNG-10b]`): C may pass any `uint32_t`, a surrogate included, which no `char` holds (`[UNS-4]`):
  "`char` has no C equivalent, and C may pass any `uint32_t`; take `c: u32` and make the character with
  `char.from_u32(c)`, which gives `None` for a value that is not one".

**FFI-56:**
p16-ffi.md lines 590-601
* `[FFI-56]` *(new in 0.9.15)* **A C function pointer is unsafe to call.** A function pointer that
  comes from C — a field of an imported struct, a typedef, a result, an `extern static`, an argument C
  passes to a callback — imports as `unsafe extern "C" fn(…)` (`Option[…]` when nullable, `[FN-6]`), so
  a call through it is in `unsafe:` (`E3100`: "`release` is a C function pointer from `hooks.h`, with no
  contract; put line 6 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe"):
  nothing tells the compiler what the function it points to does with its arguments
  (`void free_addr(uintptr_t)`, `[FFI-2]`). An overlay may assert one safe to call with `@ffi(safe)`
  on the typedef's entry (`@ffi(safe) type PFN_log_message`) when every parameter and result is a
  scalar or a `@layout(c)` value type (`E5002` otherwise): an asserted fact, listed by `ember tcb`. A
  function pointer made from an Ember function is a safe `extern "C" fn` (`[FN-6]`). The end of an
  `owns T` foreign value calls the destructor its asserted `destroys(f)` names, which the compiler
  generates.

## rules_not_found
