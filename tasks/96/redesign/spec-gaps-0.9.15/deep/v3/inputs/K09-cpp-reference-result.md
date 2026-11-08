# K09-cpp-reference-result (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K09-cpp-reference-result

**title:**
Must a C++ function returning a reference say where the reference points?

### members
#### [1]
J8-06

**primary:**
J8-06

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
9

**notes:**
Use after free as written today. B is verdict 132's lesson (no fallback to an unwritten fact) and verdict 137's reading (a C++ member has no Ember body to read).

## questions
### [1]
**id:**
J8-06

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.65

**reason:**
FFI-2a (p16-ffi.md 45): "In result position a pointer's lifetime is never derived." FFI-17a's span row (p21-annex-c.md 58): "a result needs a lifetime word". FFI-40 (162-163) instead derives a member's `T&` result as "a `ref mut T` view into the object" (LT-1 rule 1). For an Ember method that rule is backed by reading the body (verdict 137), but C++ has no body to read, and a C++ member that returns into a reference parameter (`Node& first_child(Node& parent)`) then counts as pointing into the wrong object. For free functions LT-1's rules 2 and 3 apply, and a `const T&` parameter of a `Copy` type imports as a plain `x: T`, which is no source parameter (p07-ownership.md 528-529), so the result points into nothing. The `string_view` and fixed-extent span rows say nothing about results. A call is safe only with an overlay entry (FFI-2), and an entry may write `-> from(parent)`, but whether 
an entry without a lifetime word is accepted, and what it then means, is exactly what the two rules disagree on.

**title:**
Must a C++ function returning a reference say where the reference points?

**problem:**
A C++ method returning `Node&` imports as a view, and the specification takes that view to point into the object the method was called on, with nothing written. That is false for a C++ method that returns a reference into one of its parameters. The same specification says a C pointer result's lifetime is never derived, and that a `std::span` result needs a lifetime word, so the two rules disagree on whether an overlay entry must say where a returned reference points.

**example:**
// scene.hpp (C++)
// class Node { public: int id() const noexcept; };
// class Scene {
// public:
//     Scene() noexcept;
//     Node& root() noexcept;                       // a node this scene owns
//     Node& first_child(Node& parent) noexcept;    // a node `parent` owns, not this scene
// };
import cpp "scene.hpp" with (project="engine", overlay="overlays/scene.em") as sc

# overlays/scene.em
# unsafe overlay cpp "scene.hpp":
#     class Node:
#         @ffi(const)
#         fn id()
#     class Scene:
#         fn new()
#         fn root()
#         fn first_child(parent)     # no lifetime word: taken to point into the scene it is called on

fn main():
    scene = sc.Scene.new()
    level = sc.Scene.new()
    child = scene.first_child(ref mut level.root())   # really a node of `level`
    level = sc.Scene.new()                             # ends the old level; allowed, since `child` is taken to view `scene`
    println(child.id())                                # reads a freed node

**spec_today:**
FFI-2a (p16-ffi.md 45): "In **result** position a pointer's lifetime is never derived." FFI-11 lifetime row (293): "`from(this)`, `from(p)`, `from(static)` | a view into the receiver, into `p`, or into what lives for the whole program". FFI-17a (p21-annex-c.md 58): "`std::span<T>` ... zero-copy; a result needs a lifetime word (`[FFI-11]`)". FFI-40 (162-163): "a `T&` result is a `ref mut T` view into the object and a `const T&` result a `ref T` one (`[LT-1]`)." LT-1 (p07-ownership.md 538, 544-545): "1. if it is a method, the result points into `this`" ... "The three rules decide a call where there is no code to read: ... a function with no Ember body."

#### options
##### [1]
**letter:**
A

**summary:**
As today: a C++ reference result needs no word. A method's points into its object and a free function's follows the three rules, with a `const T&` parameter counted as a source when the result is a reference. An entry returning into a parameter writes `-> from(parent)`.

**code:**
#     class Scene:
#         fn root()                              # into the scene
#         fn first_child(parent) -> from(parent)  # must be remembered; leaving it out is accepted

**outcome:**
With `-> from(parent)`, `level = sc.Scene.new()` while `child` is used is refused. Without it, the example compiles and reads freed memory, because the entry's assertion was wrong and nothing asked for the word.

##### [2]
**letter:**
B

**summary:**
A C++ reference or view result needs a lifetime word in its overlay entry, as a `std::span` result and a C pointer result already do. `[[clang::lifetimebound]]` supplies it as an asserted fact.

**code:**
#     class Scene:
#         fn new()
#         fn root() -> from(this)
#         fn first_child(parent) -> from(parent)

**outcome:**
The entry `fn first_child(parent)` with no word is the error naming the missing fact and giving this line. With the words, `level = sc.Scene.new()` while `child` is used is refused (the view of `level` is still in use).

#### recommendation
**letter:**
B

**reason:**
It is one rule for every foreign result that points somewhere (C pointers, `std::span`, C++ references). An entry's meaning then comes only from what it says, and a forgotten word is a refusal, not a read of freed memory. The cost is one word per entry that returns a reference.

**checklist:**
5 and 6 (one rule with FFI-2a and FFI-17a), 8 (a forgotten mark is refused), 15 (no fallback to an unwritten fact), 2

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
FFI-11

### [2]
FFI-11f

### [3]
FFI-17a

### [4]
FFI-2a

### [5]
FFI-40

### [6]
LT-1

## rule_text
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

**FFI-11f:**
p21-annex-c.md lines 81-82
* `[FFI-11f]` A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact:
  the attribute is not enforced by C++, so it never counts as checked.

**FFI-17a:**
p21-annex-c.md lines 54-67
* `[FFI-17a]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.10)* *(changed in 0.9.15)*

  | C++ | Ember |
  |---|---|
  | `std::span<T>`, `std::span<const T>` | `MutSpan[T]`, `Span[T]`, zero-copy; a result needs a lifetime word (`[FFI-11]`) |
  | `std::span<T, N>`, `std::span<const T, N>` (a fixed extent) | `ref mut [T; N]`, `ref [T; N]` |
  | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `StrView` (`[TXT-2]`) |
  | `std::string` | `CppString`; `.to_str() -> Result[StrView, Utf8Error]`; `String.from(c)` copies |
  | `std::vector<T>` | `CppVector[T]` with `.span()`, `.span_mut()`, `.to_array()`, `push_back`, `len` |
  | `std::unique_ptr<T>` | `owns T`, ended through the deleter; one whose deleter holds state is refused (`E5034`): an `owns T` holds one address |
  | `std::optional<T>` | `Option[T]` for a `T` that maps by value |
  | `std::shared_ptr<T>`, `std::weak_ptr<T>` | `CppShared[T]`, `CppWeak[T]` — never `shares`/`weakref` (`[WK-14]`, `E5065`) |
  | `std::variant<…>` | a generated enum, when every alternative maps and the overlay names each: `@ffi(variant=["A", "B"])` on a `type` entry naming the header's alias of the variant, the names in the alternatives' order |
  | `std::function<R(A…)>` | an opaque object Ember owns; with `@ffi(std_function)` on its overlay entry, a function value of type `fn(A…) -> R` (`[CLO-3]`), called as any other, which owns its target; never a parameter type (`E5030`) |

**FFI-2a:**
p16-ffi.md lines 43-45
* `[FFI-2a]` *(changed in 0.9.15)* Derived facts need no `unsafe`: `const T*` is a read-only view, `T*`
  one through which C may write; the layouts of structs and unions, enum underlying types, packing and
  alignment, and the calling convention. In **result** position a pointer's lifetime is never derived.

**FFI-40:**
p21-annex-c.md lines 156-163
* `[FFI-40]` *(changed in 0.9.15)* A method imported from C++ is a method of the imported class: its
  object is `this`, and it lists no receiver. C++'s body is not read as Ember's is, so whether a method
  may change its object is taken from its declaration: a `const` method only reads its object when
  `[FFI-40a]` finds so, and every other method may change it, so it is not called through a name that
  only reads the object (`E3023`) and a call of it is a change of the object for the loop and view
  rules. `&&`-qualified members are skipped (`W5033`). A `T&` parameter imports as `ref mut x: T`,
  `const T&` as a plain parameter `x: T`, and `T&&` as `owns x: T` where the type is movable; a `T&`
  result is a `ref mut T` view into the object and a `const T&` result a `ref T` one (`[LT-1]`).

**LT-1:**
p07-ownership.md lines 518-569
* `[LT-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* **Where a call's view points.** A call's result
  that is a view, and a view a call puts in through a `ref mut` parameter (`[LT-47]`), point into the
  arguments they may come from. A **source parameter** (ODR-024) is:
  * a parameter whose type is a view, in any mode; or
  * a plain or `ref mut` parameter whose type is not `Copy` even when each of its type parameters is
    taken to be `Copy`: an `Array[T]`, `String`, `Map`, an `owns T` or an arena, a struct or enum without
    `@derive(Copy)`, or a tuple or fixed array holding one of these.
  A source parameter of the second kind is the caller's place (`[FN-1]`, `[BRW-8]`): a result that points
  into it may point anywhere in it, into its own storage or into storage it owns, and the call's view is
  on the argument (`[BCK-1]`), read-only for a plain parameter and changeable for a `ref mut` one. A
  `Copy` parameter that is not a view is not a source parameter (so neither is a plain `x: T`, nor a
  function-type parameter, `[CLO-3]`), and nor is an `owns` parameter that is not a view. Which
  parameters are sources depends only on the declared first line. In an instance of a generic, a
  parameter whose type there is a view is a source of that instance too, of the first kind:
  `Tree[StrView].leaf(v: T)` may return a view of what `v` views, and its callers, which call that
  instance, know it (ODR-048). Where rule 1 below ties the result to `this`, such a parameter is a
  source alongside it when its declared type names a type parameter the declared result names too: the
  result's type says it can hold one, so `Map[StrView, V].entry(owns k) -> MapEntry[K, V, H]` returns an
  entry holding `k`, while `get(q) -> Option[ref V]` points only into the map (ODR-069).
  For a function whose result is a view:
  1. if it is a method, the result points into `this`;
  2. else, if exactly one parameter is a source parameter, the result points into it;
  3. else, the result points into **all** source parameters together.
  At a call, the caller keeps viewed, while the result lives, exactly the arguments the called
  function's code shows the result points into: its summary (`[LT-22]`), read for every result type, so
  `fn pick(a: Span[int], b: Span[int]) -> Span[int]:` whose body is `return a` keeps only `a` viewed.
  The three rules decide a call where there is no code to read: a call through a function value
  (`[LT-7]`), a function with no Ember body. A `dyn` or `virtual` method's call is decided by the code of
  each kind that may reach it, the types the program gives to the function, followed through its names
  and calls: with `interface Picker: fn pick(other: Array[int]) -> Span[int]`, `Mine.pick` returning
  `this.items` and `Yours.pick` returning `other`, `fn show(p: dyn Picker):` doing `r = p.pick(xs)`,
  `xs.push(owns 3)`, `println(r)` compiles for `show(Mine(items=[9]))`, which prints `[9]`, and is
  refused at `xs.push(owns 3)` for `show(Yours(items=[9]))` (`E3021`). Where which kind reaches it is
  known only while the program runs (`p: dyn Picker = Mine(items=[9]) if input() == "m" else
  Yours(items=[9])`, then `show(p)`), what one of them makes unsafe is `E3021` ("`xs` changes while `r`
  may still look at it: `p` may be a `Yours`, and which kind it is is only known while running", with
  the help "to check it while running, write `runtime` on the parameter:
  `fn show(p: runtime dyn Picker):`"), unless the parameter says `runtime` (`[LT-48]`). Pointing into an argument means
  pointing into it and into everything it owns (**into**, §VII.1), so a reshape of anything it owns ends
  the view and any other change of it while the view is used is refused (`[BRW-1]`). A returned view
  may point into `this` and into any source parameter, a method's as well as a function's, with nothing
  written: `fn pick(other: ref i32) -> ref i32: return other` in a type body is no error, and its callers
  keep `other` viewed. A returned view that points into any other parameter is refused: `E3063` when
  that parameter is `owns` or is passed as a copy (`[BRW-8]`), whose storage ends with the call, and
  `E3062` otherwise. A call whose result type
  is `ref mut T` or `ref T` is a place (`first(ref mut xs) += 1`; `counts.entry(owns k).or_default() += 1`),
  and a name bound to it follows `[BRW-12]` (`r = ref mut first_even(ref mut xs)`). A generator method's
  result is such a view, and it walks what it views: after `g = d.replay()`, the list it walks
  (`d.log`) does not change while `g` will still be used (`[BRW-18]`, `E3020`), however `replay` walks
  it, and any other change of `d` while `g` will still be used (`d.angle = 90.0`) is a change of what a
  view a call gave points into (`[BRW-1]`, `E3021`), unless `-> Generator[int] from log` narrows the
  view to `d.log`.

## rules_not_found
