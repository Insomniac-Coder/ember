# K31-c-kept-callback-declaration (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K31-c-kept-callback-declaration

**title:**
How is a C function that keeps an Ember callback declared and called?

### members
#### [1]
J8-01

**primary:**
J8-01

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
31

**notes:**
Its call word follows K12. A named function given to `register_cb` takes `owns` under K12's unified rule. K53 (J8-03) and K76 (J8-18) use this spelling in their examples.

## questions
### [1]
**id:**
J8-01

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.75

**reason:**
The spec's only hand declaration of a C function that keeps a callback (FFI-52, p16-ffi.md 489-499) breaks FFI-10 in three ways. It is a `safe fn` with raw `*mut void` parameters, which FFI-10 lines 82-84 refuse. It writes no result, yet `setup` returns its value as `Retained[ForeignCallback]`. It is called with one argument where two are declared. FFI-10 says how a `count(n)` length leaves the callable signature (lines 85-92, ODR-074), and nothing says the same for `user_data`. FFI-11's ownership row (line 290) gives a callback no Ember type, only "`callback(retained)` (`[FFI-21]`)". FFI-21 (lines 403-405) moves the function into a block C keeps, which is a hand-over, and verdicts 147 and 158 put `owns` at the call for a hand-over. Neither verdict covers a foreign callback parameter, because the parameter is never written as `owns`. No rule, verdict or ODR settles any of this.

**title:**
How is a C function that keeps an Ember callback declared and called?

**problem:**
A game gives a C library a function that C keeps and runs later, through a function-pointer and user-data pair. The specification's own declaration of such a function writes raw `*mut void` parameters on a `safe fn`, which the hand-declaration rule refuses. It also writes no result, although the caller returns a token from it, and its call passes one argument where two are declared. The specification does not say what the callback parameter's Ember type is, whether the user-data pointer leaves the call as a counted length does, or whether the call writes `owns` before a function C takes over.

**example:**
from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0

    fn add_point():
        this.score += 1

unsafe extern "C":
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    # C keeps `cb` and calls it later with `ud`; only on the thread that registered it
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)   # raw pointers on a `safe fn`: E5002 by FFI-10; no result written
    safe fn poll_events()                                                 # C runs the kept callbacks here

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())   # one argument for two; a token from a function with no result; no `owns` before the function C takes over

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

**spec_today:**
FFI-10 (p16-ffi.md 79-84): "A `safe fn` writes the safe Ember types and the words in front of its parameters' names (`ref mut`, `owns`) produced by those words ... A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`." FFI-10 (85-87): "On a hand-declared `safe fn` with `count(n)`, `n` is written as an integer scalar parameter at its C ABI position (ODR-074). The contract marks it as an ABI witness: it is absent from the callable Ember signature". FFI-11 ownership row (290): "for a callback, `callback(retained)` (`[FFI-21]`)" (no type given). FFI-21 (402-405): "the function value, its code and its state, is moved into a block of its own, and C is given the trampoline as the function pointer and the block's tagged number as the user data." FFI-21 (415): "`retained` returns a 
`Retained[ForeignCallback]` token". FFI-52 example (492-499): `safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)` ... `return register_cb(fn() owns game: game.add_point())`.

#### options
##### [1]
**letter:**
A

**summary:**
The declaration writes the Ember side: `owns cb: fn()`, the user-data pointer as a hidden witness (like a counted length), and the token as its result. The call says `owns`, as it does for every argument to an `owns` parameter.

**code:**
unsafe extern "C":
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void) -> Retained[ForeignCallback]   # `ud` is written at its C position and left out of the call

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(owns fn() owns game: game.add_point())
# a `callback(borrowed)` parameter is written `cb: fn()` and called with no word, as a function given to a call that runs it then

**outcome:**
Compiles. `register_cb(fn() owns game: ...)` without `owns` is the error that asks for `owns`, as `take(xs)` is. Under `threads(any)` the parameter is `owns cb: thread_shared fn()`. Calling it with two arguments is the error for too many arguments.

##### [2]
**letter:**
B

**summary:**
The same declaration with a plain parameter `cb: fn()`. The call takes no word, because the capture list already says what C keeps.

**code:**
unsafe extern "C":
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: fn(), ud: *mut void) -> Retained[ForeignCallback]

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

**outcome:**
Compiles. A function is handed over here with no word at the call, unlike every other `owns` hand-over.

#### recommendation
**letter:**
A

**reason:**
C keeps the function, so it is handed over, and the language already writes a hand-over as `owns` on the parameter and at the call, a function written in the call included. The user-data witness is written exactly as the counted-length witness is.

**checklist:**
2 (callback APIs become declarable), 4 (hand-over written, plumbing hidden), 5 and 11 (`owns` and the ABI-witness form reused), 16 (the call-site `owns` verdicts reach foreign calls)

## group
**gid:**
P08

**principle:**
Ownership that crosses between Ember and C is written in Ember's own words and has one owner at every moment: what C keeps (a callback, an object's address, a raw owner) is tracked on Ember's side by an owner or token whose end releases C's hold exactly once, and what Ember takes from C (adopt) is ended by the type's own end (its drop, its C++ destructor, or the destroys(f) its declaration names).

**shared_cause:**
Part XVI defines these crossings piecemeal. FFI-21/23 give a token and an unregister requirement only to retained callbacks. FFI-52's own declaration of a kept callback breaks FFI-10. FFI-11 gives a retained non-callback pointer no safe type. into_raw has no inverse. adopt reads its end only from destroys(f), which an Ember class or a plain C++ class lacks. So several crossings have no Ember owner, no spelling, or no stated end.

### members
#### [1]
J8-01

#### [2]
J8-03

#### [3]
J8-12

#### [4]
J8-13

#### [5]
J8-16

### clusters
#### [1]
K31-c-kept-callback-declaration

#### [2]
K53-once-callback-token

#### [3]
K52-c-keeps-address-ember-owns

#### [4]
K50-adopt-ends-by-own-destructor

### answered
### verdicts_involved
#### [1]
31: a C call may run the callbacks C keeps

#### [2]
39: a function calling C names what kept callbacks change (now inferred, 86)

#### [3]
62: a callback that removes its own token stays alive until it finishes

#### [4]
91: into_raw only inside unsafe; a C function declared as keeping an object takes it by an ordinary hand-over

#### [5]
102: owns in a C contract

#### [6]
114: fix 3, an object C keeps never moves

#### [7]
147

#### [8]
150

#### [9]
158

**other_places:**
once and retained callbacks, user-data pointers (glfwSetWindowUserPointer), std::unique_ptr and std::shared_ptr results, by-value opaque C++ results (FFI-32), borrowed C results, a buffer Ember made that C frees (manager mismatch), tokens ended after C has finished or on another thread, unregister functions that fail, objects C keeps that are then handed to another thread, and capacity of the callback table (P27).

**severity:**
high

**rank:**
8

### depends_on
#### [1]
P06

#### [2]
P07

**notes:**
J8-01's call word follows P06's rule (`owns` before a named function C keeps). J8-03 and J8-18 then use J8-01's spelling. J8-12's spelling follows J2-11 (P07): the cast `body as ObjectPtr[Body]` or a function of its own, never a third ref_to_ptr. J8-13 and J8-16 answer together (phase 3's 'the way the type's own declaration ends it'). Lead fix J6-03 gives the kept-by-C mark a place in OBJ-1. Verdict 114 decided the stop for an object C keeps; this group gives it a program that compiles.

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

**39:**
JOB.md 329-333: 39. **A function that calls C names what C's kept callbacks change** (the rule writer's question 5).
    With `setup` handing C `fn() with owns game changes Game, LOG: game.add_point()`, a function that calls
    `poll_events()` says so: `fn tick() changes Game, LOG:`, as does every function that calls `tick`; a
    C function marked `@ffi(no_callbacks)` is not such a call. Owner: "tick is a" (the question as written
    in `run4/v3/question-foreign-call.md`).

**62:**
JOB.md 496-507: 62. **A C callback that removes its own token stays alive until it finishes** (safety validation (b), hole
    6b: C1's G27). With `callback = fn() with owns my_game changes Game:` whose body is
    `my_game.token = None` then `my_game.add_point()`, the first line destroys the token (the field owns it),
    and the compiler-inserted drop runs the generated `callback_release`, which unregisters the callback and
    frees its block, where `my_game` lives, while the callback is still running; the second line would read
    `my_game` from freed memory. Now: when a token is destroyed while its callback is running, C forgets
    the callback at once, but `drop_env` and `ember_free` of its block wait until the callback returns, so the
    second line works, as the program reads (the same protection R12 gives an Ember function that removes
    itself). Owner: "a", over (b), refusing a callback that destroys its own token. What finally explained
    it: the generated C (`=` inserts the old token's drop, which ends in `callback_release`), then "my_game
    is the link and lives in the callback block; the programmer still calls add_point because nothing
    tells them line 1 kills my_game".

**91:**
JOB.md 697-708: 91. **`into_raw()` stays, usable only inside `unsafe`, built into the compiler; `mem.forget` stays removed**
    (step 3's question 11; the rule writer's item 1). A C function declared as keeping an object
    (`@ffi(param(widget, owned, one))` with `owns widget: owns Widget`) is given one by an ordinary
    hand-over, `widget_take(first)`: the name ends and Ember frees nothing, no word needed. For a C
    function whose declaration cannot say so (a plain pointer: `fn set_user_data(p: *mut Widget)`, a C
    function with `...`), `second.into_raw()` gives the address and stops Ember freeing the object (as
    C++'s `unique_ptr::release()`); the compiler does that step itself, so nothing else can skip a value's
    clean-up. Outside `unsafe:` it is refused ("`into_raw()` stops Ember from freeing the widget, so a
    mistake leaks it or frees it twice; write it inside `unsafe:`"). Verdict 60's rule. Owner: "can you
    make it so that into_raw() is only available for use in unsafe blocks?" (Correction to verdict 58's
    record: the lead had said handing an object to C "goes through the `Retained` token"; `Retained`
    keeps an object on Ember's side while C holds a callback, it does not give an object away.)

**102:**
JOB.md 814-818: 102. **In a C contract the word for "C takes this object" is `owns`** (owner, 2026-10-27, gap map Q-A:
    "owned is now owns").
    - `@ffi(param(widget, owns, one))`: `[FFI-11]`'s table and its examples write `owns`, as verdict 19
      does everywhere else.
    - `owned` is refused everywhere, a C contract included (E1010, Part II's removed words).

**114:**
JOB.md 915-943: 114. **An object handed to another thread while a `weakref` on this thread points at it goes, and the
    links here then say "gone"** (owner, 2026-10-28, gap map Q-E, "aight approved with these fixes").
    Rust's run-time way (`Rc::try_unwrap`), which the lead checked with `rustc` 1.98.1: Rust's
    compile-time way refuses even `drop(w)` then `thread::spawn(move || names.items.len())` (E0277,
    "`Rc<Names>` cannot be sent between threads safely"), so it refuses fine programs as (a) did; with
    `Rc::try_unwrap(names)` the thread gets the object and `w.upgrade()` is `None` ("0 true").
    - At the hand-over the program reads the object's weak count, one compare. No link: the object
      goes as it is. A link: its contents move to a new place for the other thread, the old place is
      marked ended (its `drop` does not run; the new one's runs once, on the other thread) and stays
      here until its last link ends, so every `weakref` here says "gone". Nothing is refused and
      nothing stops. `names = Names()`, `w = weakref names`,
      `h = thread.spawn(fn() owns names: names.items.len())`, then `if w exists as p:` is skipped and
      `println(h.join())` prints 0 (the design test that refused this becomes a run-pass test).
    - Fix 1, objects inside it: the check also covers every object the handed object owns whose class
      a `weakref` is made to anywhere in the program (the compiler knows those classes);
      `cam.target = weakref level.enemies[0]` then `thread.spawn(fn() owns level: ...)` moves enemy 0
      and `cam.target` says "gone". One compare per such object (10,000 enemies, 10,000 compares).
      Today's text of the rule misses this case too.
    - Fix 2, a link open at that moment: a hand-over to another thread joins the lines after which a
      view opened from a link to an object of that type is not used (`if ROSTER[0] exists as p:`, the
      hand-over of a `Names`, then `p.items.len()` is refused when compiling); nothing runs for it.
    - Fix 3, an address C keeps: an object given to C under a `retained` contract never moves; for it
      a link at the hand-over stops the program (gap map Q-E's (b)). `unsafe` code gets the written
      obligation that an object handed to another thread may move.
    - Checked and fine: the other thread never touches this thread's counts (the old place stays here
      with the links; the new one starts with none); an object holding a `weakref` to a one-thread
      object cannot go to another thread at all; a lock in the object is one futex word, movable, and
      cannot be held across the hand-over.
    - Goes into the next hardening (`[THR-2]`, `[RC-5]`, `[WK-12]`, `[FFI-35a]`, `[UNS-4]`) with M9.

**147:**
JOB.md 1149-1156: 147. **An argument for an `owns` parameter says `owns` at the call, whatever is passed**: with `fn take(owns
    xs: Array[int])`, `take(owns xs)` hands `xs` over and `take(xs)` is an error; a copy is `take(owns copy
    of xs)`, which keeps `xs`; a list made in the call is `take(owns [1, 2])`. Owner (2026-11-01), shown his
    rule of 2026-10-24 ("For every name, point at the word in the line that says what it is") and verdict 3
    (`bump(ref mut a)`): "YES"; of `owns copy of xs`: "I think owns copy of will make things more clear"; of
    `take(owns [1, 2])`: "yes". The spec's "An argument for an `owns` parameter is handed over with no word"
    (`[FN-2a]`, `[OWN-9]`) and "the caller passes `copy of x` to keep its own, or `x` to hand it over"
    (`[FN-2]`) were never the owner's and go.

**150:**
JOB.md 1164-1168: 150. **A function passed from a variable to a standard-library function is borrowed; the caller keeps it**
    (ODR-240 of 2026-10-22 stands): `h = fn(x: int): println(x)`, `[1, 2].iter().for_each(h)` prints 1, 2,
    and `h(5)` prints 5. `[STD-19]`'s "The function is handed over as every argument for an `owns`
    parameter is" was the lead's (0.9.15_Hardened_1, drawn from verdicts 38 and 84) and goes. Owner
    (2026-11-01): "SO IT BORROWS RIGHT?", then "YES GOOD".

**158:**
JOB.md 1210-1215: 158. **A function written in a call to a standard-library function says `owns` too** (verdict 147 for a new
    value, beside verdict 150 for a named one): `fn for_each(owns f: F) owns this` is called
    `[1, 2].iter().for_each(owns fn(x: int): println(x))`, `evens = xs.iter().filter(owns fn(x: int) -> bool:
    x % 2 == 0)`, `return xs.iter().map(owns fn(x: int) -> int: x * 2)`; a function from a variable stays
    `for_each(h)`, borrowed. Owner (2026-11-01), shown the three lines and (b) no word for a function written
    in the call: "a".

## allowed_rules
### [1]
FFI-10

### [2]
FFI-11

### [3]
FFI-21

### [4]
FFI-52

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

**FFI-21:**
p16-ffi.md lines 400-423
* `[FFI-21]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* A C function-pointer parameter accepts a capture-free Ember function
  (as an `extern "C" fn`) or an `extern "C" fn` value. A function that captures anything is `E5040`
  unless the parameter's contract names a `user_data` parameter, in which case the importer generates
  the checked trampoline (`[FFI-53]`): the function value, its code and its state, is moved into a
  block of its own, and C is given the trampoline as the function pointer and the block's tagged
  number as the user data. With `callback(borrowed)` C runs the function only during the call, the
  block is freed when the call returns, and the function may use the caller's real variables, as a
  function given to any call that runs it then may (Part VI), within `[FFI-22]`'s thread rule. With
  `callback(retained)` or `callback(once)` C keeps the function after the call: its capture list says
  what it holds (`owns`, `copy of`, `thread_shared`; `shares`, `shares mut` and `weakref` only under
  `threads(creator)` or `threads(main)`, `[FFI-22]`), and a name it uses with no word is `E7004`: "C
  keeps this function after `setup` returns (`callback(retained)`), so it cannot use `setup`'s `game`;
  say what it holds: write `return register_cb(fn() owns game: game.add_point())` in place of line 17
  (`setup`'s owner of the game), or capture `fn() weakref game` with
  `if game exists as ref mut g: g.add_point()` as its body (a link that lets the game end)".
  `retained` returns a `Retained[ForeignCallback]` token (`[FFI-23]`) whose end unregisters the
  callback through the declared `destroys(f)` (`E5041` if none is declared), and with `once` the
  trampoline ends the block after the first call. A callback with several parameters of its user
  data's type names the one that receives it: `user_data=p@N`, the callback's `N`th parameter
  (1-based, as in the C signature); without `@N`, exactly one parameter has that type, or the contract
  is `E5002`, whose help shows the `@N` form. A `nullable` callback with user data is an `Option` of a
  function value: `None` passes `NULL` and no user data. Handing a function to a `callback(retained)`
  or `callback(once)` parameter counts as making what it changes, now and whenever C runs it
  (`[FFI-52]`).

**FFI-52:**
p16-ffi.md lines 451-514
* `[FFI-52]` *(new in 0.9.15)* **What a foreign call may run.** The compiler cannot read a foreign
  function's code, so a call of one counts as possibly running, on the calling thread, every Ember
  function C can reach then: every callback C keeps (one handed to a `callback(retained)` or
  `callback(once)` parameter and not yet unregistered) whose thread contract allows this thread; every
  `@export` and `@export_table` function whose thread contract (`[FFI-33]`) allows it; every override
  of an Ember class derived from a C++ class whose objects C++ may hold (`[FFI-39d]`); every
  `callback(borrowed)` function whose own call is still running on this thread; and every Ember
  function given to C as an `extern "C" fn` value. The call makes every change those functions make,
  worked out from their bodies as every call's changes are (Part X), so the rules for loops, read-only
  names, `ref mut` names and views of objects reached through links (`[BCK-3]`, `[BCK-7]`) apply to it,
  and to every function that calls it: a `fn tick(): poll_events()` changes what the kept callbacks
  change, and so does every function that calls `tick`. A foreign function that never calls back into
  Ember says so on its declaration or overlay entry, `@ffi(no_callbacks)`: a call of it runs none of
  them. The mark is an asserted fact, listed by `ember tcb` (`[TCB-1]`); on a function whose own
  contract takes a callback it is `E5069`: "`register_cb` takes a callback (`param(cb,
  callback(retained, user_data=ud))`), so it may call back into Ember; remove `no_callbacks` from its
  `@ffi(...)`". A missing mark only refuses a line that the reachable functions' changes would make
  unsafe, and the message names the mark (`E3020`): "`sqrt(d)` (line 21) is a C function, which may run
  the callback C keeps from `register_cb` (line 14), and that callback changes `game.names`, which this
  loop walks (line 20); if `sqrt` never calls back into Ember, say so on its declaration:
  `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`; otherwise move line 21 below the loop"; for an
  export, "`host_poll()` (line 12) may run `log_line` (exported, line 5), which adds to `LOG` (line 7)
  while this loop walks `LOG` (line 11); call `host_poll()` before or after the loop, or, if
  `host_poll` never calls back into Ember, mark it `@ffi(no_callbacks)` in `host.overlay.em`". Across a
  package's edge, the callbacks a package hands to C, the functions it exports and the overrides it
  gives C++ are recorded in its interface file (`[EFF-4]`), so a program that uses the package counts
  them at every foreign call. Nothing runs while the program runs for this rule.

  ```ember
  from std.ffi import ForeignCallback, Retained

  class Game:
      score: i32 = 0
      names: Array[String] = []

      fn add_point():
          this.score += 1

  unsafe extern "C":
      @safety("`cb` and `ud` are a pair `register_cb` was given")
      unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
      @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
      safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
      safe fn poll_events()
      @ffi(no_callbacks)
      safe fn sqrt(x: f64) -> f64

  fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
      return register_cb(fn() owns game: game.add_point())

  fn main():
      game = shares mut Game(names=[String("ann"), String("bo")])
      token = setup(shares mut game)
      for name in game.names:
          println(name, sqrt(name.len() as f64))
      poll_events()
      println(game.score)
  ```

  The callback C keeps holds `setup`'s owner of the game (`fn() owns game`), a one-thread owner, which
  its `threads(creator)` contract allows. `poll_events()` may run it, and so changes `game.score`; the
  callback changes nothing in `game.names`, so a C call inside the walk of `game.names` would be
  allowed here too. With a callback that pushes to `game.names`, `poll_events()` inside the walk is
  `E3020`, and so is `sqrt` there without its mark.

## rules_not_found
