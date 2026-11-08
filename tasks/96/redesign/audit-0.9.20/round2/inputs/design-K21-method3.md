## K21-cpp-overloads (P07; J8-04): **unresolved**, 7 agents

Outcome: `decided_by_parameters`. Intent: Build a C++ renderer with its (width, height) constructor, then draw a mesh and a sprite through its two `draw` overloads, running the functions the C++ header means.
The rule that decides it: FFI-39b (p21-annex-c.md 128-130): "`super.init(…)` selects the C++ base constructor by arity and argument types and runs it at `super.init(…)` (`[CLS-11]`)"

### Edit: TYP-26 (add, p04-types.md 867-868)

**Before:**

> There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types.

**After:**

> A C++ name of several functions imports as one name (`[FFI-39b]`): the header, not the program, declares them.

Why general: Says why the no-overloading rule does not reach a header (principle 6). Refusing a header's overloads would refuse correct programs over code they did not write and cannot change. It covers every C++ name of several functions (namespace functions, methods, static functions, constructors) without naming any of them.

### Edit: FFI-39b (add, p21-annex-c.md 128-130)

**Before:**

> `super.init(…)` selects the C++ base constructor by arity and argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and no declared `init` is `E5058`.

**After:**

> Each use of a C++ name of several functions (as `[FFI-30]` counts them) stands for the one it fits, ranked as `[TYP-24]` ranks instances, an untyped literal counting as C++ types it (`c_int`, `f64`) and an object as `ref` through a name that only reads it or in an entry with `@ffi(const)`, else `ref mut`: a call (`new`, `super.init(…)`) by its object and its arguments' number, names, types and words; an `override fn` by its parameters; a function value by its expected callable type; an overlay entry by what it writes. What the code around it permits is checked after, on the one chosen. None, or several fitting equally, is `E2070` (an entry `E5010`), naming each.

Why general: One property for every use of such a name, not just calls. A use stands for the one function it fits, ranked the way Ember already ranks an interface's instances (exact type before coercion). Literals count as C++ types them, and the object counts as C++ counts its implied object, so where C++ picks a function Ember picks the same one or refuses. Nothing more is needed for constructors, `super.init`, methods, namespace functions, overrides, function values and overlay entries. A redeclared function counts once. What the code around a use permits (unsafe code, effects, values handed over, threads, views) is checked on the chosen function, so a line runs the same function wherever it stands. A use that fits nothing, or ties, is refused with the candidates named.

### Edit: GRM-35 (add, p16-ffi.md 266-267)

**Before:**

> In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147).

**After:**

> An entry may write each parameter's and the result's word and type as imported (`owns value: Texture`), checked, before its contract words; `[FFI-30c]` and `[FFI-55]` count entries by function.

Why general: The single entry form gets an optional typed parameter. It is Ember's own spelling of a parameter, already written in `@ffi(macro_fn)` entries. With FFI-39b's 'an overlay entry by what it writes', the type is needed exactly where parameter names cannot tell functions apart; elsewhere it may be written and is checked. Every overlay rule that counts entries (`override`, a second entry, the `ember.overlays` record) then counts per function, for every overloaded name.

### Decision (made by the owner's parameters): How a C++ name declared several times imports, and which function each use stands for

**A C++ class with two constructors and two `draw(...)` overloads: what Ember calls them, and which one `r.draw(mesh)` runs.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | importer | reads `Renderer()`, `Renderer(int, int)`, `draw(const Mesh&)`, `draw(const Sprite&)` | one `new` and one `draw` on `Renderer` |
| 2 | program | writes `Renderer.new(1280, 720)` and `r.draw(mesh)` | no new word |
| 3 | compiler | keeps the functions the call's own words fit (count, names, types, `owns` / `ref mut`, the object) | `Renderer(int, int)`, `draw(const Mesh&)` |
| 4 | compiler | takes the exact fit; none, or still two, is an error naming each; then checks `unsafe` and the rest on that one | the same function wherever the line stands |
| 5 | program runs | a direct call of that function | the same cost as the C++ call |

Today: Nothing says. Read literally, two functions of one name are an error, and only `super.init` picks a C++ constructor by its arguments.

* A: one name per C++ name; each use picks by its own words, as C++ does and as `super.init` already does
* B: one name per overload, built from its types (`draw_Mesh`, `new_int_int`)
* C: overloads skipped unless an overlay renames each one
* D: as A, but a call picks among the functions it could compile as where it stands, so moving it into an `unsafe:` block can change which one runs

Recommendation: A: the C++ program's own calls work unchanged, one line always runs one function, and an unclear call is refused with the candidates named.

Chosen: A. By: 1. The three goals: A keeps C++'s names, adds no word and costs nothing while running. B and C add names the program must write or keep. 2. Safety: A runs one function per line wherever the line stands, and refuses a use that no function fits, or that two fit equally. D lets the surrounding code change what runs, with no diagnostic. 3. Compile time first: A is decided when compiling, as a direct call of the chosen thunk. 4. A written word only for meaning: B's type-built names and C's renames are bookkeeping. 5. The ordinary program: A is the C++ program's own lines. B is longer and breaks when a type is renamed. C needs an overlay line per overload before anything works. 6. Consistency: FFI-39b already picks `super.init`'s constructor by arity and argument types. TYP-24 already chooses among one name's interface instances by a call's arguments. FFI-48 refuses only overloads no call can tell apart. FFI-44 imports classes automatically, and FFI-40 and FFI-41 keep C++'s names. C++ chooses by the call alone, never by the code around it.

Option A:

```
r = eng.gfx.Renderer.new(1280, 720)   # Renderer(int, int)
r.draw(mesh)                           # draw(const Mesh&)
r.draw(hud)                            # draw(const Sprite&)
```

Compiles. There is one Ember name per C++ name. Each use stands for the function its own words fit: a call by its object and arguments, an override by its parameters, a function value by its expected type. Exact types are taken first. What the code around the use permits is checked afterwards, on that function. None, or two fitting equally, is an error naming each C++ declaration.

Option B:

```
r = eng.gfx.Renderer.new_int_int(1280, 720)
r.draw_Mesh(mesh)
r.draw_Sprite(hud)
```

Compiles. Each overload is its own Ember function, named from C++ types the program never wrote, and the names change when the header renames a type.

Option C:

```
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        rename new(width, height) as sized
        rename draw(mesh) as draw_mesh
        rename draw(sprite) as draw_sprite
```

Without the overlay every overload is skipped, and `r.draw(mesh)` gives a 'skipped: overloaded' diagnostic. With it, `r.draw_mesh(mesh)` compiles. Today's `rename` grammar takes only a name, so the grammar would have to change too.

Option D:

```
ring.push(pcm)            # push(int32_t): the only one with an overlay entry
# SAFETY: push(int16_t) only stores the sample in the ring's own buffer
unsafe:
    ring.push(pcm)        # push(int16_t): the exact fit
```

Compiles, but the same line runs two different C++ functions. A call chooses among the functions it could compile as where it stands, so moving it into or out of an `unsafe:` block, or into a `@noalloc` function, silently changes which one runs.

### Decision (made by the owner's parameters): How an overlay entry names one of several C++ functions alike in name, parameter count and parameter names

**Two C++ overloads with the same parameter names, `set(const char* name, float value)` and `set(const char* name, int value)`: how an overlay line says which one it vouches for.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | overlay writer | writes `fn set(name: borrowed nul_terminated, value)` | it fits both overloads |
| 2 | compiler | refuses it, naming both C++ declarations | error |
| 3 | overlay writer | writes `fn set(name: *c_char borrowed nul_terminated, value: f32)`, and the same line with `value: c_int` | each line names one overload |
| 4 | compiler | checks each line against its own overload, as for any entry | both overloads safe to call |

Today: Overlay lines are matched by name, count and parameter names only, so such a pair cannot be told apart, and a second line for one name is an error.

* a: one line covers every overload it fits
* b: the line may write the parameters' Ember types, as Ember writes a function's first line, and must where the names do not tell
* c: refuse, with no way to vouch for either overload
* d: write the C++ signature in an attribute, `@ffi(cpp="set(const char*, float)")`

Recommendation: b: Ember's own spelling of a parameter, needed only where the names do not already tell the overloads apart.

Chosen: b. By: 1. The three goals: b keeps the overlay readable as Ember and costs nothing while running. 2. Safety: a lets one line vouch for a function its writer never named (verdict 131's problem). b, c and d refuse an entry that fits several functions. 3. Compile time first: all four are decided when compiling. 4. A written word only for meaning: which function a line vouches for is meaning. Under b the type is needed only where the names do not tell; elsewhere it is optional and checked. c makes it impossible to write, an exemption that exists only because syntax is missing (ODR 13's lesson). 5. The ordinary overlay: unchanged under b; d adds an attribute line per overload. 6. Consistency: b is Ember's own spelling of a parameter (`value: f32`, `owns value: Texture`), optional as a variable's type is, and overlays already write types in `@ffi(macro_fn)` entries (checklist 11). d is a second spelling. On 2026-10-29 the lead removed the separate typed member line ('one way to write an overlay line'); b keeps one entry form, whose parameters may carry a type.

Option a:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        fn set(name: borrowed nul_terminated, value)   # covers both `set` overloads
```

Compiles. One line makes both overloads safe to call, including one its writer may never have looked at, and both must take the same contract words.

Option b:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        fn set(name: *c_char borrowed nul_terminated, value: f32)
        fn set(name: *c_char borrowed nul_terminated, value: c_int)
```

Compiles. An entry may write a parameter's type and word as the import maps them, checked against the header. Each of these lines therefore names one function and is checked against it. The untyped line fits both and is E5010, naming both. Entries whose names already differ are written as today.

Option c:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        fn set(name: borrowed nul_terminated, value)   # E5010: fits two functions
```

Refused, and no line can name either overload. Both stay unsafe to call, with no place to write their `@safety`.

Option d:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        @ffi(cpp="set(const char*, float)")
        fn set(name: borrowed nul_terminated, value)
        @ffi(cpp="set(const char*, int)")
        fn set(name: borrowed nul_terminated, value)
```

Compiles. Each overload is named by C++ text in a string: a second spelling, in another language, of what a parameter list already says.

### Decision (made by the owner's parameters): Which overload an untyped number literal picks

**C++ `set(const char* name, int value)` and `set(const char* name, float value)`: which one `s.set(c"mode", 2)` runs.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | program | writes `s.set(c"mode", 2)` | a plain `2` |
| 2 | compiler | counts `2` as C++ counts it, an `int` | |
| 3 | compiler | `set(…, int)` takes it as it is, `set(…, float)` only by conversion | `set(const char*, int)` |
| 4 | compiler | where C++ itself cannot choose (`value(int64_t)` / `value(double)` with `7`), an error naming both | `7 as i64` picks one |

Today: Nothing says. Read as Ember's other overload choice, `2` counts as Ember's 64-bit `int`, which neither function takes, so the call is an error.

* i: count a literal as Ember's default (64-bit `int`, `f64`): `s.set(c"mode", 2)` is an error; write `2 as c_int`
* ii: a whole-number literal prefers any whole-number type: `set` works, but `put(int)` / `put(long)` with `5`, and `scale(float)` / `scale(double)` with `2.0`, are errors
* iii: count a literal as C++ does (`int`, `double`): Ember runs what C++ runs, and refuses where C++ refuses

Recommendation: iii: the C++ lines work unchanged and never run a different function from the one C++ would.

Chosen: iii. By: 1. The three goals: iii keeps the C++ program's lines with no cast and costs nothing while running. 2. Safety: all three are decided when compiling and refuse rather than guess. iii never runs a different function from C++, because Ember's exact fit is C++'s and Ember's conversions are fewer. 3. Compile time first: all three. 4. A written word only for meaning: i's `as c_int` on every literal is bookkeeping, and so are the casts ii needs. 5. The ordinary program: under iii it is the C++ line. i and ii refuse common calls (uniforms, int/long, float/double). 6. Consistency: when choosing among one name's instances, the nearest rule (TYP-24) already counts a literal as its default type. iii does exactly that, with the defaults of the language the header is written in (`int`, `double`).

Option i:

```
s.set(c"mode", 2)            # error: `2` counts as Ember's `int` (i64); both `set` reach it by conversion
s.set(c"mode", 2 as c_int)   # what the program must write
```

Refused, with `as c_int` to write on every literal call of an int/float overload pair; `super.init(16, 8)` too.

Option ii:

```
s.set(c"mode", 2)            # set(const char*, int): a whole-number literal prefers whole-number types
out.put(5)                   # error: put(int) and put(long) both take whole numbers
m.scale(2.0)                 # error: scale(float) and scale(double) both take floats
```

Compiles for `set`. Refuses the int/long and float/double pairs, which C++ calls without trouble.

Option iii:

```
s.set(c"mode", 2)            # set(const char*, int): `2` counts as C++'s `int`
out.put(5)                   # put(int)
m.scale(2.0)                 # scale(double): `2.0` is C++'s `double`
w.value(7)                   # error, as in C++: value(int64_t) and value(double) both convert
```

Compiles wherever C++ picks one, running the same function. Refused, naming both, where C++ is ambiguous too.

### Decision (made by the owner's parameters): Which of a const and a non-const C++ method a call runs

**C++ `const Entry& at(size_t) const` and `Entry& at(size_t)`: which one `cfg.at(0)` runs.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | program | writes `cfg.at(0).port = 8080` in `main`, where `cfg` may be changed | no new word |
| 2 | compiler | the non-const `at` fits `cfg` as it is, the const one only as a read-only view | `Entry& at(size_t)` |
| 3 | compiler | in `port_of(cfg: Config)`, `cfg` only reads, so only the const `at` fits | `const Entry& at(size_t) const` |
| 4 | program runs | prints `8080` | as in C++ |

Today: Nothing says. Read literally, two functions of one name are an error.

* a: both fit a changeable name equally: an error, and the non-const `at` can never be called
* b: as in C++: a changeable name picks the non-const `at`, a read-only one the const `at`
* c: the const `at` unless the result is changed, as Rust picks between its two indexing interfaces

Recommendation: b: what C++ runs, decided by the call alone, and every function in the pair can be called.

Chosen: b. By: 1. The three goals: b keeps C++'s meaning with no word and no run-time cost. 2. Safety: b and c are safe, but c runs a different function from C++ for the same call; a refuses. 3. Compile time first: all three. 4. A written word only for meaning: none of them needs one. a needs a helper function to reach the const `at` through a changeable name, and leaves the other `at` unreachable, an exemption that exists only because syntax is missing (ODR 13's lesson). 5. The ordinary program: b compiles the C++ lines; a refuses them. 6. Consistency: the object counts as one more argument under the nearest rule (exact before coercion; a read-only view of a changeable place is a coercion, TYP-5 rule 7), as C++ counts its implied object. An overlay entry's `@ffi(const)` (verdict 132) then names which of the pair it describes, with no new word. c would choose by the code around the call, which the first decision excludes. K47 (J8-05) keeps its own question, the overlay name of `operator[]`, whose pair goes through Index/IndexMut (FFI-42), not this selection.

Option a:

```
cfg.at(0).port = 8080      # error: both `at` fit a changeable `cfg` equally
```

Refused through every changeable name, which is the ordinary local. No line can ever call `Entry& at`.

Option b:

```
cfg.at(0).port = 8080      # Entry& at(size_t): `cfg` may be changed
# in fn port_of(cfg: net.srv.Config):
return cfg.at(0).port      # const Entry& at(size_t) const: `cfg` only reads
```

Compiles and runs what C++ runs. The non-const result is a changing view of `cfg` while it is used, as C++'s `Entry&` is, so two such results held at once through one changeable name are refused.

Option c:

```
e = cfg.at(0)              # const at: the result is only read
cfg.at(0).port = 8080      # Entry& at: the result is changed
```

Compiles more Ember programs, but which C++ function a call runs depends on the lines after it, and can differ from C++. For example, a copy-on-write string's non-const accessor copies and the const one does not.

Readability: No longer and no harder. The ordinary program is shorter than today's shim. Extra words appear only on the rare overlay entry whose parameter names cannot say which function it vouches for.
Cost: None at run time: it is decided when compiling. Each call is a direct call of the chosen function's thunk, just as a C++ call resolves its overload when compiling. Per call that is the same single thunk call as any imported C++ function, inlined across the boundary under the project's LTO. There is no table and no check while running. Compile time adds one fit test per candidate function at each use of an overloaded name (to measure).

Final check of the last breakers' holes: 9 confirmed real, 0 refuted, 1 already in today's specification (only the confirmed count)

### Holes left (9)

* **unsafe_allowed**, high: `send_focus` calls `h.on(FocusEvent())`. The slot runs `KeyLog.on` with a FocusEvent taken as a KeyEvent, and `e.code()`, which the overlay vouches safe and which is true for a real KeyEvent, reads `code_` at offset 8 of an 8-byte object. In C++ terms this is an unchecked `static_cast` downcast, a read past the object, and it happens in safe Ember code: both `unsafe:` notes are true. C++ itself refuses this class ('marked override but does not override'). The scalar case goes wrong the same way: `override fn on(timer_id: i16)` fits `on(int)` by widening, so C++'s 70000 reaches an `i16` truncated, while the safe direction, `override fn on(timer_id: int)`, fits nothing. For an override the coercion runs backwards, because C++'s argument flows into the Ember parameter.

```
// input.hpp (C++)
// namespace ui {
// class Event {
// public:
//     Event() noexcept;
//     virtual ~Event() noexcept;
// };
// class KeyEvent : public Event {
// public:
//     explicit KeyEvent(int code) noexcept : code_(code) {}
//     int code() const noexcept { return code_; }
// private:
//     int code_;
// };
// class FocusEvent : public Event {        // no field of its own: 8 bytes; a KeyEvent is 16
// public:
//     FocusEvent() noexcept;
// };
// class Handler {
// public:
//     Handler() noexcept;
//     virtual ~Handler() noexcept;
//     virtual void on(const Event& e) noexcept {}
//     virtual void on(int timer_id) noexcept {}
// };
// void send_key(Handler& h, int code) noexcept;   // calls h.on(KeyEvent(code))
// void send_focus(Handler& h) noexcept;           // calls h.on(FocusEvent())
// }
# overlays/input.em
unsafe overlay cpp "input.hpp":
    class KeyEvent:
        @ffi(const)
        fn code()

# src/main.em
from std.ffi import c_int
import cpp "input.hpp" with (project="ui", overlay="overlays/input.em") as gui

class KeyLog(gui.ui.Handler):
    last: c_int = 0

    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()

    override fn on(e: gui.ui.KeyEvent):
        this.last = e.code()

fn main():
    log = KeyLog()
    # SAFETY: the senders call `on` on the handler during the call only
    unsafe:
        gui.ui.send_key(ref mut log, 27)
        gui.ui.send_focus(ref mut log)
    println(log.last)
```

* **needless_refusal**, medium: C++ ranks argument by argument: set(int, int) beats set(int, float) on the second argument and ties on the first, and the non-const `light` beats the const one on the object. Ember refuses both ordinary lines. The same refusal hits every index call of a const/non-const accessor with a literal or an `int` index, and every pair where one shared argument needs a widening. The way out is a cast on each argument (`slot as c_int`, `0 as usize`), the bookkeeping decisions 1 and 4 say the edit removes. S1-3's own program (`cfg.at(0).port = 8080`) is still refused, so S1-3 and E1-10 are not repaired. The same cause ties a nearer base against a farther one (`draw(const Shape&)` / `draw(const Circle&)` with a FilledCircle) and a promotion against a conversion (`put(int)` / `put(long)` with an `i16`), both of which C++ picks.

```
// scene.hpp (C++)
// namespace gfx {
// struct Light { float intensity; };
// class Scene {
// public:
//     Scene() noexcept : lights_{}, counts_{} {}
//     void set(int slot, int value) noexcept { counts_[slot & 3] = value; }
//     void set(int slot, float value) noexcept { lights_[slot & 3].intensity = value; }
//     const Light& light(size_t i) const noexcept { return lights_[i & 3]; }
//     Light& light(size_t i) noexcept { return lights_[i & 3]; }
// private:
//     Light lights_[4];
//     int counts_[4];
// };
// }
# overlays/scene.em
unsafe overlay cpp "scene.hpp":
    class Scene:
        fn new()
        fn set(slot: c_int, value: c_int)
        fn set(slot: c_int, value: f32)
        @ffi(const)
        fn light(i)
        fn light(i)

# src/main.em
import cpp "scene.hpp" with (project="engine", overlay="overlays/scene.em") as gfx

fn main():
    scene = gfx.gfx.Scene.new()
    for slot in 0..4:
        scene.set(slot, 4)
    scene.light(0).intensity = 0.5
    println(scene.light(0).intensity)
```

* **undecided**, medium: Under the import's reading no line can call either `at`, unsafe code included, and no overlay line can vouch for either. That is the exemption that exists only because a word is missing (checklist 12). Bodies in a .cpp are the usual case for a non-template class. A `mutable` member (a cache, a mutex) in a const method is common too, and FFI-40a reads it as changing as well. E1-7's program (`visit` declared without a body) fails the same way, so E1-7 is not repaired. The answer's config sibling works only because its bodies are in the header ('The importer reads that the const `at` changes no field').

```
// config.hpp (C++): bodies in config.cpp, as for most non-template classes
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept;
//     const Entry& at(size_t i) const noexcept;   // defined in config.cpp: returns entries_[i & 3]
//     Entry& at(size_t i) noexcept;               // defined in config.cpp: returns entries_[i & 3]
// private:
//     Entry entries_[4];
// };
// }
# overlays/config.em
unsafe overlay cpp "config.hpp":
    class Config:
        fn new()
        @ffi(const)
        fn at(i)
        fn at(i)

# src/main.em
from std.ffi import c_int
import cpp "config.hpp" with (project="server", overlay="overlays/config.em") as net

fn port_of(cfg: net.srv.Config, i: usize) -> c_int:
    return cfg.at(i).port

fn main():
    cfg = net.srv.Config.new()
    first: usize = 0
    cfg.at(first).port = 8080
    println(port_of(cfg, first))
```

* **undecided**, medium: Under the first answer `show(s: Server)` is E3023 ('`port` may change its object'). Under the second it compiles and prints 0. The same program compiles on one implementation and not on another, and runs a different C++ function, which matters wherever the pair differs (copy-on-write detaching, or a const accessor that is thread-safe beside a non-const one that is not). C++ decides by the method's own `const`, which Ember does not write. A lambda using a real variable (`fn() => println(cfg.at(i).port)`, CLO-2) is circular the same way. Wrapping a C++ object in an Ember class field is an ordinary pattern.

```
// config.hpp (C++)
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept : entries_{} {}
//     const Entry& at(size_t i) const noexcept { return entries_[i & 3]; }
//     Entry& at(size_t i) noexcept { return entries_[i & 3]; }
// private:
//     Entry entries_[4];
// };
// }
# overlays/config.em
unsafe overlay cpp "config.hpp":
    class Config:
        fn new()
        @ffi(const)
        fn at(i)
        fn at(i)

# src/main.em
from std.ffi import c_int
import cpp "config.hpp" with (project="server", overlay="overlays/config.em") as net

class Server:
    cfg: net.srv.Config

    fn init():
        this.cfg = net.srv.Config.new()

    fn port(i: usize) -> c_int:
        return this.cfg.at(i).port

fn show(s: Server, i: usize):
    println(s.port(i))

fn main():
    s = Server()
    show(s, 0)
```

* **regression**, high: A promise written and checked for `draw(const Mesh&)` silently vouches for a function nobody reviewed: `count=100000` reads past the renderer's table in safe code (in C++ terms, a buffer over-read). This happens for every C++ entry whose function's parameter types change, overloaded or not, which is exactly the header update verdict 121's record exists to catch. The edit leaves the key to tooling ('the key format is the lead's tooling'), but the key decides whether the check happens.

```
// renderer.hpp (C++), version 2 of the engine SDK
// namespace gfx {
// struct Batch { int first; int count; };
// class Renderer {
// public:
//     Renderer() noexcept;
//     ~Renderer() noexcept;
//     void draw(const Batch& mesh) noexcept;   // draws meshes first .. first+count of its table, unchecked
// };
// }
// (version 1, which the overlay was checked against and ember.overlays recorded:
//  `struct Mesh { int vertices; };` and `void draw(const Mesh& mesh) noexcept;`, reading only the mesh given)
# overlays/renderer.em
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        fn new()
        fn draw(mesh)

# src/main.em
import cpp "renderer.hpp" with (project="engine", overlay="overlays/renderer.em") as eng

fn main():
    r = eng.gfx.Renderer.new()
    r.draw(eng.gfx.Batch(first=0, count=100000))
```

* **undecided**, medium: Under the import reading, an ordinary C++ class whose const accessor is defined out of line cannot be called through any name, and no overlay can name either function. The same holds for a class with a `mutable` member (a mutex), or whose const body calls something the importer cannot follow. The edit's verdict-132 claim, '`@ffi(const) fn visit(ref mut v: Visitor)` describes the const `visit`', fails for its own E1-7 header, whose `void visit(Visitor& v) const noexcept;` has no body. C++ runs `Entry& at` in `main`, runs the const `at` in `port_of`, and prints 8080.

```
// config.hpp (C++): the methods are defined in config.cpp
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept;
//     ~Config() noexcept;
//     const Entry& at(size_t i) const noexcept;   // aborts unless i < 4
//     Entry& at(size_t i) noexcept;               // aborts unless i < 4
// private:
//     Entry* entries_;
// };
// }
# overlays/config.em
unsafe overlay cpp "config.hpp":
    class Config:
        fn new()
        @ffi(const)
        fn at(i)
        fn at(i)

# src/main.em
import cpp "config.hpp" with (project="server", overlay="overlays/config.em") as net

fn port_of(cfg: net.srv.Config) -> c_int:
    return cfg.at(0).port

fn main():
    cfg = net.srv.Config.new()
    cfg.at(0).port = 8080
    println(port_of(cfg))
```

* **needless_refusal**, medium: In C++, `int heading() const { return transform().angle; }` runs the const one and prints 90. An Ember class over an engine's C++ base is FFI-39's main use, and such bases commonly pair const and non-const accessors. Under the edit, a method that only reads is refused through every read-only name, and no word can say it only reads (verdict 86 removed `reads`). The way out is a free helper that takes the component as a plain parameter, which is the reason decision 4 gives for rejecting option a.

```
// scene.hpp (C++)
// namespace eng {
// struct Transform { int x; int y; int angle; };
// class Component {
// public:
//     Component() noexcept;
//     virtual ~Component() noexcept;
//     const Transform& transform() const noexcept { return transform_; }
//     Transform& transform() noexcept { return transform_; }
// private:
//     Transform transform_{0, 0, 90};
// };
// }
import cpp "scene.hpp" with (project="engine") as core

class Spinner(core.eng.Component):
    fn init():
        # SAFETY: the base constructor only sets its own transform
        unsafe:
            super.init()

    fn heading() -> c_int:
        # SAFETY: `transform` only gives a view of the component's own transform
        unsafe:
            return this.transform().angle

fn show(s: Spinner):
    println(s.heading())

fn main():
    s = Spinner()
    show(s)
```

* **needless_refusal**, medium: C++ runs `show(int)` (an `unsigned char` to `int` is a promotion, which beats the conversions to `long long` and `double`) and prints `int 3`. Under the edit the line needs `lives as c_int`. That is the bookkeeping cast decision iii removed for literals, back again for every narrow number (game counters in `u8`, audio samples in `i16`) passed to the int/long/double overload sets common in C++ APIs. C++ also ranks argument by argument; the edit ranks the whole call.

```
// hud.hpp (C++)
// #include <cstdio>
// namespace ui {
// class Counter {
// public:
//     Counter() noexcept;
//     void show(int value) noexcept { std::printf("int %d\n", value); }
//     void show(long long value) noexcept { std::printf("long long %lld\n", value); }
//     void show(double value) noexcept { std::printf("double %.2f\n", value); }
// };
// }
import cpp "hud.hpp" with (project="game") as hud

fn main():
    lives: u8 = 3
    # SAFETY: the counter only prints the number it is given
    unsafe:
        c = hud.ui.Counter.new()
        c.show(lives)
```

* **not_general**, medium: C++ prints `int 5` on every target. Under the edit the same Ember program compiles on Linux and is refused on Windows, and the C++ overload sets that cover every integer type (`QString::number`, `std::to_string`, `abs`) cannot be used at all on one target or the other. This also re-tests E1-12: its premise, that identity is undecided, is refuted, because the library says aliases. The failure comes from the edit ranking on the aliased Ember types, while its literal rule already appeals to C++'s own types. Under the strict test (refused today too) a checker may class it pre_existing; it is filed here because decision iii states this program's outcome as one its words give.

```
// stream.hpp (C++)
// #include <cstdio>
// namespace io {
// class Out {
// public:
//     Out() noexcept;
//     void put(int v) noexcept { std::printf("int %d\n", v); }
//     void put(long v) noexcept { std::printf("long %ld\n", v); }
// };
// }
import cpp "stream.hpp" with (project="tools") as s

fn main():
    # SAFETY: `put` only prints its argument
    unsafe:
        out = s.io.Out.new()
        out.put(5)
```

New gaps found on the way (not this edit's): 6

Tests rejected by the script, not written (10):
* override_of_overloaded_virtual: the expected output is a description, not the exact text: C++'s meaning: stdout "36 108\n" once, and draw(const Label&) stays the base's. Undecided 
* widget_override_mesh_slot: the expected output is a description, not the exact text: 36 108 (printed once)
* keylog_upcast_override_refused: the program is a description, not code: (hole H1's program, whole: input.hpp with Event, KeyEvent, FocusEvent, Handler with on(con
* scene_per_argument_ranking: the program is a description, not code: (hole H2's program, whole: scene.hpp with set(int, int), set(int, float), const/non-const 
* config_out_of_line_const_pair: the program is a description, not code: (hole H3's program, whole: config.hpp with both `at` declared, bodies in config.cpp; overl
* voice_deleted_overload: the program is a description, not code: (hole H4's program, whole: voice.hpp with set_volume(float linear) and set_volume(int) = d
* server_this_field_choice: the program is a description, not code: (hole H9's program, whole: config.hpp with inline const/non-const at; overlays/config.em; 
* inventory_const_assertion_on_overload: the program is a description, not code: (hole H7's program, whole: legacy.hpp with non-const count() and count(int kind); overlays
* sink_rvalue_virtual_override: the program is a description, not code: (hole H5's program, whole: jobs.hpp with Payload (move ctor, dtor), Sink with virtual take
* keylog_single_virtual_mismatch: the program is a description, not code: (hole H6's program, whole: input.hpp with Handler's single virtual on(const Event& e); cla

To verify on the compiler:
* Renderer: `import cpp` of renderer.hpp (two constructors, two `draw`) gives no `E1030`. `Renderer.new(1280, 720)` runs `Renderer(int, int)`, and `r.draw(mesh)` / `r.draw(hud)` run `draw(const Mesh&)` / `draw(const Sprite&)`. Check with `ember bind --emit-cpp` and a run that prints which overload ran.
* Shader: `s.set(c"light_count", 4)` runs the `int` overload and `s.set(c"exposure", 1.5)` the `float` one. The untyped entry `fn set(name: borrowed nul_terminated, value)` is `E5010`, naming both. `super.init(16, 8)` under `Widget(int, int)` / `Widget(float, float)` runs `(int, int)`.
* Literals: `out.put(5)` under `put(int)` / `put(long)` runs `put(int)`, and `m.scale(2.0)` under `scale(float)` / `scale(double)` runs `scale(double)`. `w.value(7)` under `value(int64_t)` / `value(double)` is `E2070`, naming both. Compare each case with MSVC, Clang and GCC on Windows and Linux, since `int64_t` is `long long` on one and `long` on the other.
* Asset cache: `cache.add(atlas)` runs `add(const Texture&)` and leaves `atlas` usable; `cache.add(owns splash)` runs `add(Texture&&)`. A second `cache.add(owns splash)` is `E3040` at that line, not `E2070`.
* Config: `cfg.at(0).port = 8080` through a changeable local runs the non-const `at`; through a read-only parameter, the const one runs; the program prints 8080. Two non-const results held at once through one changeable name are refused (E3022): check the message.
* Mixer: `ring.push(pcm)` (an `i16`) runs `push(int16_t)` both inside and outside `unsafe:`. Outside, it is `E3100` naming that function. The lead's wording should also name each safe overload the call would reach with another argument type (`m.gain(1.5 as f32)`).
* Effects: in a `@noalloc` function, the same call picks the same overload as elsewhere, and `E4001` is given on that overload if it lacks `effects=[]`.
* Override: `override fn on(e: gui.ui.KeyEvent)` replaces only `on(const KeyEvent&)` in the trampoline, while the click runs the base's `on(const MouseEvent&)`; the program prints 1. With the widget header, `36 108` is printed once. Check the generated C++ with `ember bind --emit-cpp`. Also check whether Ember code still reaches the base's other overload through the derived class.
* Function value: `project: fn(vm.Vec3i, vm.Vec3i) -> c_int = vm.dot` prints 32. `f = vm.dot` with no expected type is `E2070`, naming both.
* Redeclaration: a member declared in its class and defined below it, or a function declared twice, is one function (one USR). No `E2070` and no `E5010`.
* Overlay record: the renderer overlay gives no `E5010`. `ember.overlays` keys each entry by its function (the key format is the lead's tooling). A new overload whose parameter has the same name makes the old entry `E5010`, naming both.
* Const pair in an overlay: `@ffi(const) fn visit(ref mut v: Visitor)` describes the const `visit` and `fn visit(ref mut v: Visitor)` the other; `root.visit(ref mut tally)` through a read-only `root` runs the const one.
* Typed entries: how a type named like a contract word (`handle`, `one`, `status`) is read before the contract words. The lead decides the reading, for example a raw identifier `r#handle`.
* `L3015` for an overloaded C++ name comes once per function and names every other call. Its message gives an entry line that compiles, with types where the names do not tell the functions apart.
* `E2070`'s message for an imported overload set (the code table says 'ambiguous interface method') is the lead's wording. It names each C++ declaration and why each does not fit; where all fail for one reason (E3023 for every overload), it gives that reason.
* Whether `c_int` and `c_long` are distinct Ember types on MSVC (pre-existing, E1-12). If they are aliases, `put(int)` / `put(long)` import with one signature and every call ties.

