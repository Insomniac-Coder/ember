# K07-raw-pointer-struct-safe-call (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K07-raw-pointer-struct-safe-call

**title:**
May a safe C call take a struct that holds raw pointers?

### members
#### [1]
J8-08

**primary:**
J8-08

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
7

**notes:**
Memory corruption from safe code. A changes XVI.4's vkCreateBuffer entry. Rewrite that example once, together with K46 (J8-14, `device: borrowed`). The example's `ref_to_ptr(ref mut x)` takes whatever spelling K51 (J2-11) keeps.

## questions
### [1]
**id:**
J8-08

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.65

**reason:**
FFI-10 (p16-ffi.md 59-61) accepts a `safe fn` when "every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete", and lines 82-84 refuse a raw pointer even with complete facts. Wrapped in a `@layout(c)` struct, the same raw pointer passes the list. Contract words attach only to parameters, and FFI-30a (210-212) bars an overlay from touching fields, so nothing can state a field pointer's count or lifetime. Safe code can fill such a field with any address (`ref_to_ptr` is safe, p04-types.md 340-342, ODR-075). FFI-56 (596-598) uses the same list for `@ffi(safe)` typedefs. The spec's own XVI.4 overlay asserts `vkCreateBuffer(pCreateInfo: borrowed one)` safe, and its create-info struct holds `pQueueFamilyIndices` and `pNext`. No verdict covers this.

**title:**
May a safe C call take a struct that holds raw pointers?

**problem:**
A safe C function may not take a raw pointer, because the compiler cannot check what C does through it. The same pointer inside a `@layout(c)` struct (a buffer description, a Vulkan create-info) passes the rule, although nothing can state its length or lifetime. Safe Ember code can then fill the struct with any address and length, and C writes through it.

**example:**
@layout(c)
struct PixelBuf:
    data: *mut u8      # C writes `len` bytes here
    len: usize

unsafe extern "C":
    @ffi(no_callbacks)
    safe fn clear_pixels(buf: PixelBuf)    # accepted by the parameter list today

fn main():
    x: u8 = 7
    clear_pixels(PixelBuf(data=ref_to_ptr(ref mut x), len=4096))   # every line is safe code; C writes 4,096 bytes over one
    println(x)

**spec_today:**
FFI-10 (p16-ffi.md 59-61): "`safe fn` ..., safe to call only when every parameter and result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete"; (82-84): "complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`." FFI-30a (210-211): "An overlay changes the signatures, safety and names of *functions* ...; it never changes a type's identity, layout or fields". FFI-56 (596-598): "`@ffi(safe)` on the typedef's entry ... when every parameter and result is a scalar or a `@layout(c)` value type". IV.5 (p04-types.md 340-341): "Creating one from a reference is safe (ODR-075): `ref_to_ptr(r: T) -> *T`".

#### options
##### [1]
**letter:**
A

**summary:**
A `@layout(c)` type that holds a raw pointer, by value or behind a contracted pointer, counts as a raw pointer. A `safe fn` (or an `@ffi(safe)` typedef) taking one is refused. The safe interface is an ordinary wrapper that builds the struct from a checked view inside `unsafe:`.

**code:**
unsafe extern "C":
    @safety("`buf.data` points at `buf.len` writable bytes")
    @ffi(no_callbacks)
    unsafe fn clear_pixels(buf: PixelBuf)

fn clear(ref mut bytes: MutSpan[u8]):
    # SAFETY: the struct describes exactly the bytes of `bytes`
    unsafe:
        clear_pixels(PixelBuf(data=bytes.as_mut_ptr(), len=bytes.len()))

**outcome:**
`safe fn clear_pixels(buf: PixelBuf)` is E5002, naming the field `data`. The wrapper compiles, and safe callers can only clear bytes they own. The XVI.4 example's `vkCreateBuffer` becomes an `unsafe fn` entry with `@safety`, or gets a wrapper.

##### [2]
**letter:**
B

**summary:**
As today. `safe fn` is the declarer's promise and also covers the struct's pointer fields.

**code:**
unsafe extern "C":
    @ffi(no_callbacks)
    safe fn clear_pixels(buf: PixelBuf)

**outcome:**
The example compiles, and C overwrites 4,095 bytes past `x` (memory corruption in safe code). `ember tcb` lists the assertion.

#### recommendation
**letter:**
A

**reason:**
A raw pointer is refused on a safe C call whether it travels alone or in a struct, and C's view of the memory comes from a span the compiler checked. This keeps Rust-level safety with no run-time cost.

**checklist:**
5 and 6 (one rule for raw pointers, no struct-shaped way around it), 8, 2 (correct programs keep a safe wrapper), 7

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
FFI-10

### [2]
FFI-11

### [3]
FFI-30a

### [4]
FFI-56

## rule_text
**FFI-10:**
p16-ffi.md lines 56-110
* `[FFI-10]` *(changed in 0.9.9)* *(changed in 0.9.15)* An `unsafe extern "C":` block declares foreign functions, statics
  and opaque types; `unsafe` records that the programmer asserts the signatures. The declared names
  are items of the enclosing module. Each function in the block says whether it is safe to call
  (verdict 136): `safe fn` (a contextual keyword, `[LEX-15]`), safe to call only when every parameter and
  result is a scalar, a `@layout(c)` value type, or a pointer whose contract is complete (given with
  `@ffi(…)` on the declaration), `safe` on any other function being `E5002`, naming the fact that is
  missing; or `unsafe fn`, whose calls are in unsafe code, with `@safety("…")` above it saying what its
  callers must make true (`[UNS-7]`), as an overlay entry is (`[GRM-35]`, verdict 133). A plain `fn`, which
  everywhere else in Ember is safe to call, says neither and is `E5002`: "`lib_step` does not say whether
  it is safe to call: write `unsafe fn lib_step`, with `@safety("…")` above it saying what its callers
  must make true, or `safe fn lib_step` if a call within its contract cannot break memory safety", which
  `ember fmt --migrate` rewrites to `unsafe fn` (`[CLI-20]`); `safe unsafe fn` is `E5002` too
  ("`lib_step` is declared both `safe` and `unsafe`"). A call of an `unsafe fn` outside unsafe code is
  `E3100`, as any `unsafe fn`'s: "`raw_abs(-3)` (line 11) calls an `unsafe fn`, whose callers must make true what its
  `@safety` says". `unsafe` is no word of `@ffi(…)`: `@ffi(unsafe)`, `param(p, unsafe)` and
  `result(unsafe)` are `E0104` ("`unsafe` is no contract word; write the declaration
  `unsafe fn peek(…)`, with `@safety("…")` above it saying what its callers must make true"), which
  `ember fmt --migrate` rewrites (`[CLI-20]`); `@ffi(…)` keeps its other words. `safe fn` is an
  asserted fact, listed by `ember tcb` (`[TCB-1]`). Its effects are
  those its `@ffi(effects=[…])` states plus `FFI`, and every effect when none are stated (`[EFF-3]`);
  which Ember code a call of it may run is `[FFI-52]`'s, and `@ffi(no_callbacks)` says it runs none.
  On a hand declaration, `@ffi(param(name, word, …), result(word, …))` attaches `[FFI-11]`'s
  contract words to each named pointer parameter and the pointer result (ODR-073). Each slot is named
  at most once; `link_name`, `effects` and `no_callbacks` can share the same `@ffi`. A `safe fn` writes
  the safe Ember types and the words in front of its parameters' names (`ref mut`, `owns`) produced by
  those words; the compiler checks that
  mapping and lowers it to the C ABI carrier. A raw `*T` can be declared for an unsafe call, but
  complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is
  `E5002`. Missing facts or a type/contract mismatch is `E5002`, naming the slot.
  On a hand-declared `safe fn` with `count(n)`, `n` is written as an integer scalar parameter at
  its C ABI position (ODR-074). The contract marks it as an ABI witness: it is absent from the
  callable Ember signature, and the compiler supplies the corresponding `Span[T]` or `MutSpan[T]`
  length at the call. The written integer type fixes the C width and signedness. A length that does
  not fit that type, or unequal lengths from two pointer clauses sharing `n`, panics before the
  foreign call. A missing or non-integer witness or an incompatible pointer surface is `E5002`.
  An unsafe raw-pointer declaration keeps its written C ABI parameters; only a mapped `safe fn`
  erases the witness.
  For an input `param(p, borrowed, count(n), nullable)`, the safe type is `Option[Span[T]]`, or, with
  `exclusive`, `Option[MutSpan[T]]` on a `ref mut` parameter, whose argument the call marks `ref mut`
  (`[FN-2a]`) (ODR-079). `None` sends C a null pointer and integer zero.
  `Some` sends its checked length, and a non-null pointer even when empty: if the view's data
  pointer is null, the wrapper supplies a statically stored, properly aligned `T` sentinel.
  The C function must not access an element when the count is zero. Several input clauses sharing
  `n` compare effective lengths, with `None` contributing zero. A counted result sharing `n`
  receives that effective length. This does not define `inout_count`.
  A borrowed pointer **result** contracted `count(n)` is safe only if at least one borrowed input
  `Span` or `MutSpan` also has `count(n)` (ODR-077). That input supplies the hidden C witness and
  the returned view length; several such inputs must have equal lengths before C. A result is
  `Span[T]` through a C `const T*`, or `MutSpan[T]` through a mutable C `T*` when contracted
  `exclusive` (ODR-078). With `nullable`, a null pointer is `None`, otherwise the result is
  `Some(Span[T])` or `Some(MutSpan[T])` with that length. A non-null zero-length result is still
  `Some`.
  A result-only `count(n)` on a `safe fn` is `E5002`: the compiler has no input length to send to C.
  The result still needs `from(p)` or `from(static)`. An unsafe raw declaration keeps its written
  count parameter; this rule does not define `inout_count` or another output-count carrier.

**FFI-11:**
p16-ffi.md lines 283-309
* `[FFI-11]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **Contract vocabulary.** A pointer contract has five axes;
  mutability comes from `const` and is never written. The same words appear after a parameter name
  in an overlay and inside `@ffi(param(name, word, …), result(word, …))` on a hand declaration
  (ODR-073); that attribute does not create a separate contract system.

  | Axis | Words | Ember type produced |
  |---|---|---|
  | ownership | `borrowed` (valid for the call), `owns` (callee takes it; verdict 102), `returns_owned(destructor=f)`, `retained` (callee keeps the pointer after the call) | a parameter that views the caller's value for the call (`x: T`, `Span[T]`; `ref mut x: T`, `MutSpan[T]` with `exclusive`); an `owns x: owns T` parameter, whose argument is handed over and which Ember then never frees; an `owns T` result, whose end calls `f`; for a callback, `callback(retained)` (`[FFI-21]`), and on any other pointer no safe type, since C keeps Ember memory only when it is handed over (`owns`) or given with `into_raw()` (`[FFI-23]`, `[FFI-54]`): the call stays `unsafe`, with a raw pointer |
  | count | `one`, `count(n)` (as many elements as parameter `n`, which then leaves the callable signature; a hand declaration writes its ABI witness under `[FFI-10]`, ODR-074; a result needs an input span sharing `n`, ODR-077; nullable input carriers follow ODR-079), `nul_terminated`, `fixed(N)`, `inout_count(p)` (`[FFI-11b]`) | one `T` (a parameter `x: T`, a result `ref T`), `Span[T]`/`MutSpan[T]`, `CStrView`, `[T; N]` (a parameter `x: [T; N]`, a result `ref [T; N]`) |
  | nullability | `nullable` | `Option[…]` |
  | lifetime (results) | `from(this)`, `from(p)`, `from(static)` | a view into the receiver, into `p`, or into what lives for the whole program (`[LT-1a]`, `[LT-3]`) |
  | aliasing | `exclusive`, `aliased` | `ref mut` and `MutSpan[T]` results allowed only with `exclusive`; a result is `aliased` unless stated |

  A parameter `borrowed one exclusive`, and one whose count makes a `MutSpan[T]` that C writes, is
  written with `ref mut` in front of its name (`ref mut p: T`, `ref mut p: MutSpan[T]`), for an opaque
  type too, and the call marks its argument `ref mut` (`[FN-2a]`); a plain parameter only reads what it
  views.

  Other contract words: `out` (an out-parameter, returned instead); `status` (with `@ffi(status=E,
  ok=X, error=N)` on the enum, `[FFI-16]`); `handle`; `callback(borrowed | retained | once, user_data=p)`
  (`[FFI-21]`); `threads(any | main | creator)`; `destroys(f)` (`f`'s parameters may be the registering
  call's own and its result, `id = listen(…)` then `unlisten(id)`: the token keeps them); and, on a
  function, `no_callbacks` (it never calls back into Ember,
  `[FFI-52]`), and on a function-pointer typedef, `safe` (`[FFI-56]`).
  `@ffi(effects=[…])` states effects. An overlay entry and a hand declaration leave their calls unsafe
  by being written `unsafe fn` (`[GRM-35]`, `[FFI-10]`, verdicts 130 and 133); `unsafe` is no contract
  word, in an overlay or in `@ffi(…)` (`E0104`).

**FFI-30a:**
p16-ffi.md lines 210-212
* `[FFI-30a]` An overlay changes the signatures, safety and names of *functions* and adds wrappers; it
  never changes a type's identity, layout or fields, so packages with different overlays for one header
  still exchange its values.

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
