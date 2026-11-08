# K11-unsafe-drop-override (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K11-unsafe-drop-override

**title:**
May a class's drop, or an override of a safe method, be an unsafe fn?

### members
#### [1]
J5-16

**primary:**
J5-16

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
11

**notes:**
Unsafe code reached from safe code with nobody vouching (principle 11). A also needs a rule that an interface implementation's signature must match, which J5's notes found missing.

## questions
### [1]
**id:**
J5-16

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
medium

**confidence:**
0.55

#### rules
##### [1]
UNS-1

##### [2]
UNS-12

##### [3]
DRP-1

##### [4]
DRP-8

##### [5]
DSP-2

##### [6]
CLS-4

**reason:**
The grammar (p03 63) allows `unsafe` before any method, `override` included, and [UNS-12] makes the whole body unsafe code. [UNS-1] requires unsafe code to call an `unsafe fn`, while [DRP-1]/[DRP-8] run `drop` wherever the value ends, and [DSP-2]/[DSP-5] send a safe virtual call to any override. No rule refuses `unsafe fn drop()` or an `unsafe` override, and none says who vouches (grep 'unsafe fn drop', 'override.*unsafe': nothing). Verdicts 60, 118 and 119 do not reach this case.

**title:**
May a class's `drop`, or an override of a safe method, be an `unsafe fn`?

**problem:**
The grammar lets `unsafe` stand before any method, and the body of an `unsafe fn` is unsafe code as a whole, which invites `unsafe fn drop()` in a class that wraps a C handle. But an `unsafe fn` may be called only from unsafe code, while `drop` runs wherever the object ends: the end of a block in safe code, a list being cleared. Likewise, a safe virtual call may land in an `unsafe` override. No rule refuses either case or says who vouches for it.

**example:**
# SAFETY: these declarations match window.h
unsafe extern "C":
    safe fn create_window() -> *mut u8
    @safety("`w` came from `create_window` and is destroyed once")
    unsafe fn destroy_window(w: *mut u8)

class Window:
    handle: *mut u8

    @safety("`this.handle` came from `create_window` and is destroyed only here")
    unsafe fn drop():                        # the whole body is unsafe code
        destroy_window(this.handle)

fn main():
    w = Window(handle=create_window())       # a C window wrapped in an Ember object
    println("open")
    # w ends here, in safe code: its `unsafe fn drop` runs with no unsafe: block and no SAFETY note

**spec_today:**
p03-grammar.md 63: `fn_header := ["extern" STRING] ["unsafe"] ["static" | "virtual" | "override"] ["gen"]`. p09-memory.md 355-356 [UNS-12]: "An `unsafe fn` is unsafe as a whole ... Its body is unsafe code". p09-memory.md 266-268 [UNS-1]: "An `unsafe` context is required to: ... call an `unsafe fn`". p07-ownership.md 842-844 [DRP-1]: "`fn drop():` runs exactly once per value, at the end of its life ... Calling it explicitly is `E3070`". No rule names `unsafe` on `drop` or on an `override`.

#### options
##### [1]
**letter:**
A

**summary:**
`unsafe` is refused on `drop`, on an override of a safe method, and on an implementation of a safe interface method.

**code:**
# SAFETY: these declarations match window.h
unsafe extern "C":
    safe fn create_window() -> *mut u8
    @safety("`w` came from `create_window` and is destroyed once")
    unsafe fn destroy_window(w: *mut u8)

class Window:
    handle: *mut u8

    fn drop():                               # safe: it runs wherever a Window ends
        # SAFETY: `this.handle` came from `create_window`, and drop runs once
        unsafe:
            destroy_window(this.handle)

fn main():
    w = Window(handle=create_window())
    println("open")

**outcome:**
In the example, `unsafe fn drop()` is refused: "`drop` runs wherever a `Window` ends, in safe code, so it cannot be an `unsafe fn`; write `fn drop():` and put the call in an `unsafe:` block with a `# SAFETY:` note". This version compiles, prints open, and destroys the window when `w` ends. An `unsafe` override of a safe `virtual` method is refused the same way.

##### [2]
**letter:**
B

**summary:**
Allowed, but every end of such an object is unsafe, so it must be ended inside `unsafe:`.

**code:**
# SAFETY: these declarations match window.h
unsafe extern "C":
    safe fn create_window() -> *mut u8
    @safety("`w` came from `create_window` and is destroyed once")
    unsafe fn destroy_window(w: *mut u8)

class Window:
    handle: *mut u8

    @safety("`this.handle` came from `create_window` and is destroyed only here")
    unsafe fn drop():
        destroy_window(this.handle)

fn main():
    w = Window(handle=create_window())
    println("open")
    # SAFETY: the window's handle came from create_window
    unsafe:
        mem.drop(owns w)

**outcome:**
The example is E3100 at the end of `main`, where `w` ends outside unsafe code. This version compiles. Every place a `Window` may end (a list of them cleared, a field replaced) needs the same, and a safe virtual call that may reach an `unsafe` override is E3100 too.

#### recommendation
**letter:**
A

**reason:**
A `drop` and an override are called by code that cannot vouch for them (the end of a block, a safe virtual call), so the vouching belongs inside them, in an `unsafe:` block with its note. Rust does the same: `Drop::drop` is safe.

**checklist:**
8 (a forgotten block is a refusal), 9 (the message gives a line that compiles), 11 (unsafe only where someone vouches), 13 (no unsafe path hidden in a block's end)

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
CLS-4

### [2]
DRP-1

### [3]
DRP-8

### [4]
DSP-2

### [5]
UNS-1

### [6]
UNS-12

## rule_text
**CLS-4:**
p05-declarations.md lines 335-345
* `[CLS-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* A class is final unless declared `open` or `abstract`. Methods are
  non-virtual unless `virtual`; replacing one requires `override`; overriding a non-virtual method is
  `E2110`. A method named like an inherited one replaces it (ODR-028): over a virtual method without
  `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or not; `init`, `drop`
  and `static fn`s are each class's own. An `override` is itself virtual, so a further subclass may
  override it again. `virtual` in a class that cannot be subclassed (neither `open` nor `abstract`) is
  `E2112` ("`Door` cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses
  replace it write `open class Door:`, or remove `virtual`"). A call through a `virtual` method counts
  what every override the program has changes, worked out from their bodies (`[DSP-5]`), so nothing is
  written about what an override changes. A derived `init` MUST call `super.init(…)` exactly once
  (`[CLS-11]`). Fields cannot be overridden. Base fields are laid out first; there is at most one base.

**DRP-1:**
p07-ownership.md lines 842-844
* `[DRP-1]` *(changed in 0.9.15)* `fn drop():` runs exactly once per value, at the end of its life.
  It changes its own object as a given, since it ends it, and its first line says nothing about that.
  Calling it explicitly is `E3070`; `mem.drop(owns x)` ends a value early.

**DRP-8:**
p07-ownership.md lines 865-873
* `[DRP-8]` *(new in 0.9.15)* **Ending a value counts as a call of its `drop`.** Anything that may end a
  value whose type has a `drop` (a list cleared, a value replaced, a local ending with its function, a
  function value or an `owns dyn I` value replaced or ended with what its state owns, `[CLO-10]`) makes
  the changes that `drop` makes, wherever a value of its type may end, for every rule that needs them
  (`[EFF-1]`). With `class Enemy:` whose `fn drop():` pushes to `LOG`, `enemies.clear()` inside
  `for line in LOG:` is `E3020` ("`enemies.clear()` (line 5) ends every `Enemy` in `enemies`, and an
  `Enemy`'s `drop` pushes to `LOG` (its line 12), which this loop walks (line 4); clear after the loop:
  move line 5, `enemies.clear()`, below the loop"), and a function that may end an `Enemy` changes `LOG`.
  A `drop` may change things outside its object (a log line, leaving a list).

**DSP-2:**
p08-classes.md lines 286-287
* `[DSP-2]` A virtual call loads its slot from the object's type table; slots are assigned in
  declaration order, base first, and an `override` reuses the base's slot.

**UNS-1:**
p09-memory.md lines 266-285
* `[UNS-1]` *(changed in 0.9.15)* An `unsafe` context is required to: dereference, read or write
  through `*T`/`*mut T`; offset a pointer; cast between pointers or pointers and integers; call an
  `unsafe fn` (`Allocator.deallocate` and `reallocate` among them, `[ALC-2]`), or through a value of an
  `unsafe` callable type (`[FN-6]`); call an `extern` function without a verified contract (Part XVI);
  access a foreign `static mut` (`[STA-1]`); `transmute`; call `get_unchecked`, `assume_init` or
  `assume_disjoint`; give an object to C with `into_raw()` (`[OWN-12]`); make or write a `MaybeUninit`
  of a type that needs drop (`[ARN-8]`); implement an `unsafe` interface, or declare `Send` by hand
  (`unsafe extend T implements Send:`, `[TYP-43]`). Each of these outside unsafe code (an `unsafe:`
  block, or an `unsafe fn` body outside its `safe:` blocks, `[UNS-12]`) is `E3100`,
  whose message says what the code must vouch for and gives the block to write ("`read_u32_le(buf.as_ptr())`
  (line 9) calls an `unsafe fn`, whose callers must make true what its `@safety` says; put line 9 in an
  `unsafe:` block with a `# SAFETY:` note saying why that holds here: above line 9 write `# SAFETY:` and
  the reason, then `unsafe:`, and indent line 9 under it"). The block to write goes around the whole
  statement the operation is in: the compound statement for one in its header (`elif p.read() > 0:`
  is its `if`'s, "put lines 9 to 12 in an `unsafe:` block"), every line of a statement written over
  several, and "put the statement on line 7 on a line of its own, in an `unsafe:` block" for one on
  the line of a header; outside a function's body (a constant's value), where no block can be
  written, the message gives none. The unsafe operations of one statement are one `E3100`, naming
  them together ("the reads through `p` on line 9"); a pointer's offset is named with the read or
  write it leads to.

**UNS-12:**
p09-memory.md lines 355-367
* `[UNS-12]` *(new in 0.9.15)* **An `unsafe fn` is unsafe as a whole** (verdict 118: "the function
  literally says that it is unsafe"). Its body is unsafe code, as the inside of an `unsafe:` block is.
  Everything written in unsafe code is unsafe (verdict 119): a function declared there is an
  `unsafe fn`, whose body is unsafe, whose calls need unsafe code, and which carries `@safety`
  (`[UNS-7]`); a lambda written there is a value of an `unsafe` callable type, `unsafe fn(…)`
  (`[FN-6]`), which only unsafe code calls (`fn make() -> unsafe fn(*mut u8):` may return one made in
  an `unsafe:` block; `-> fn(*mut u8)` may not). Code inside unsafe code that is to be checked as safe
  code is written in a `safe:` block, which needs no note: in it each operation of `[UNS-1]` is `E3100`
  again ("`p.write(1)` (line 7) is in a `safe:` block, which is checked as safe code; move it out of
  the `safe:` block, or put it in an `unsafe:` block with a `# SAFETY:` note"), a function or lambda
  written in it is safe, and an `unsafe:` block in it is unsafe again. A `safe:` block outside unsafe
  code changes nothing and is `L3020` ("this `safe:` block is not inside unsafe code, so it changes
  nothing: remove `safe:`"); an `unsafe:` block directly inside unsafe code is `L3010` (`[UNS-3]`).

## rules_not_found
