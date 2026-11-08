# K21-cpp-overloads (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K21-cpp-overloads

**title:**
How do C++ overloaded functions and constructors import into Ember?

### members
#### [1]
J8-04

**primary:**
J8-04

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
21

**notes:**
Nearly every C++ header has overloads. A is an exception to TYP-26 that gives its reason (principle 6). K47 (J8-05) names `operator[]`'s const and non-const pair, so ask after this.

## questions
### [1]
**id:**
J8-04

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.75

**reason:**
TYP-26 (p04-types.md 867-868): "There is no overloading: two functions of one name in one scope are `E1030`". FFI-48 (p21-annex-c.md 90) refuses only overloads that differ in return type, which implies that the others import, but no rule says how. FFI-39b (128-129) picks a base constructor "by arity and argument types" for `super.init` only. FFI-17 (10-12) generates "one thunk per imported function, method, constructor", so a class with two constructors gets two `new`. GRM-35's `overlay_fn` (p16-ffi.md 260) and FFI-12 (322-323) match entries by name, count and parameter names, so FFI-48's own fix, "rename one in the overlay", cannot name one overload. No verdict covers this. C++ overloads, constructors above all, are in nearly every C++ header.

**title:**
How do C++ overloaded functions and constructors import into Ember?

**problem:**
Almost every C++ class has overloaded methods or several constructors, and Ember itself has no overloading. The specification refuses only overloads that differ in return type (which C++ cannot declare anyway). It does not say what Ember name or names the ordinary overloads get, how a call picks one, or how an overlay entry names one of them.

**example:**
// renderer.hpp (C++)
// namespace gfx {
// struct Mesh { int vertices; };            // standard-layout: imported by value
// struct Sprite { int frame; };
// class Renderer {
// public:
//     Renderer() noexcept;
//     Renderer(int width, int height) noexcept;
//     void draw(const Mesh& mesh) noexcept;
//     void draw(const Sprite& sprite) noexcept;
// };
// }
import cpp "renderer.hpp" with (project="engine") as eng

fn main():
    mesh = eng.gfx.Mesh(vertices=36)
    hud = eng.gfx.Sprite(frame=2)
    # SAFETY: the renderer draws only what it is given, during the call
    unsafe:
        r = eng.gfx.Renderer.new(1280, 720)   # which `new`? two constructors: E1030?
        r.draw(mesh)                           # one `draw` that picks by type, two names, or skipped?
        r.draw(hud)

**spec_today:**
TYP-26 (p04-types.md 867-868): "There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types." FFI-48 row (p21-annex-c.md 90): "overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay". FFI-39b (128-129): "`super.init(…)` selects the C++ base constructor by arity and argument types". FFI-12 (p16-ffi.md 322-323): "An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`". GRM-35 (260): `overlay_fn := ["unsafe"] "fn" IDENT "(" [overlay_param ...] ")"`.

#### options
##### [1]
**letter:**
A

**summary:**
One Ember name for each C++ name. The call picks the overload by the arguments' types, as C++ does and as `super.init` already does. An overlay entry names one overload by its parameter count and names.

**code:**
    r = eng.gfx.Renderer.new(1280, 720)   # the (int, int) constructor
    r.draw(mesh)                           # draw(const Mesh&)
    r.draw(hud)                            # draw(const Sprite&)

# overlay: one entry per overload, told apart by its parameter names
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        fn new(width, height)
        fn draw(mesh)
        fn draw(sprite)

**outcome:**
Compiles and calls the overload C++ would call. A call that no overload fits, or that two fit equally, is an error naming the candidates.

##### [2]
**letter:**
B

**summary:**
Each overload imports under its own name, built from its C++ parameter types.

**code:**
    r = eng.gfx.Renderer.new_int_int(1280, 720)
    r.draw_Mesh(mesh)
    r.draw_Sprite(hud)

**outcome:**
Compiles. Each overload is a separate Ember function. The names come from types the program never wrote, and they change when the header renames a type.

##### [3]
**letter:**
C

**summary:**
An overload set imports only through an overlay that renames each overload. Without that, every overload is skipped with a warning that gives the overlay lines.

**code:**
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        rename new(width, height) as sized
        rename draw(mesh) as draw_mesh
        rename draw(sprite) as draw_sprite

**outcome:**
Without the overlay, `r.draw(mesh)` is the "skipped: overloaded" diagnostic. With it, `r.draw_mesh(mesh)` compiles.

#### recommendation
**letter:**
A

**reason:**
It keeps what the C++ header means and works with no overlay. It reuses the selection Ember already does for `super.init`, and Ember's own no-overloading rule only fails to describe C++ names, so the exception says why.

**checklist:**
2 (ordinary C++ imports), 4 (no names to invent), 6 (an exception only where the general rule is untrue for C++, with the reason), 11 (FFI-39b's selection reused), 13 (no hidden renaming)

## group
**gid:**
P07

**principle:**
One name in one scope is one function, for everyone: where the library or a C++ header has several functions of one name (by a parameter's word, by parameter types, a const and a non-const operator), they reach Ember by one stated rule that a program could follow too, and an overlay names each one the way the importer names it.

**shared_cause:**
TYP-26 says there is no overloading and allows exceptions only for operator implementations and extend blocks. Yet the library declares two ref_to_ptr, and C++ headers bring overload sets, several constructors and const/non-const operator[] pairs. FFI-48 refuses only overloads that differ in return type. GRM-35's overlay entries match by name, count and parameter names, and have no form for an operator member. No rule says how a set of same-named functions reaches Ember, or how one is named or vouched.

### members
#### [1]
J8-04

#### [2]
J8-05

#### [3]
J2-11

### clusters
#### [1]
K21-cpp-overloads

#### [2]
K47-overlay-operator-names

#### [3]
K51-ref-to-ptr-spelling

### answered
### verdicts_involved
#### [1]
125: C++ members stay unsafe until an overlay entry vouches

#### [2]
131: C++ member lines go under their class

#### [3]
132: @ffi(const) on an entry

**other_places:**
C++ default arguments, constructors chosen by super.init (FFI-39b), operator==, operator<< and operator(), templates and their specialisations, Index/IndexMut and other operator-interface pairs, an operator interface implemented for several right-hand types (String's Add[StrView] under P06's J2-04 A), extend blocks, the library's other twin functions, and the safe cast `ref x as *T` beside ref_to_ptr.

**severity:**
high

**rank:**
7

### depends_on
**notes:**
Ask J8-04 before J8-05: the const and non-const operator[] are an overload pair. Ask J2-11 before P08's J8-12 (no third ref_to_ptr overload). J8-04 A (one Ember name, the call picks by argument types) would make imported C++ the one place with overloading, so it must give its reason or the rule must hold for programs too; J2-11 C is that wider rule. J2-11 B also meets P24's 'one idea, one spelling'. ODR-075 (a lead ruling) kept the name.

## conflicts_naming_a_member
## verdicts_involved
**125:**
JOB.md 1027-1031: 125. **A function imported from C says what its callers must make true.** Owner (2026-10-29): "Yeah for
    functions imported from C should require @safety as C code can't be trusted as safe". A function
    `import c` writes that its overlay entry leaves unsafe to call carries `@safety` in that entry; a
    call of one without it is `L3015`, as for any `unsafe fn` without `@safety` ([UNS-7]). C++ too
    (owner, asked: "yes C++ too, same rule"): a function `import cpp` writes, members included.

**131:**
JOB.md 1048-1051: 131. **An overlay line for a C++ class member goes under its class.** Owner: "yes member lines under
    their class" (a top-level `fn new(x, y)` made both `Handle.new` and `MutHandle.new` safe). A
    top-level line names a function outside any class; one naming only members is `E5010` ([GRM-35],
    [FFI-12], [FFI-40a]).

**132:**
JOB.md 1052-1055: 132. **An overlay entry without `@ffi(const)` makes its method one that changes its object**, whatever
    the importer proves from the header's body; `@ffi(const)` keeps it const. Owner, shown the fix that
    kept the importer's reading: "NO", then, of (1) not const / (2) another spelling: "1" ([FFI-40a];
    reverses the in-place correction of Hardened_5, so a new hardening).

## allowed_rules
### [1]
FFI-12

### [2]
FFI-39b

### [3]
FFI-48

### [4]
GRM-35

### [5]
TYP-26

## rule_text
**FFI-12:**
p16-ffi.md lines 322-325
* `[FFI-12]` *(changed in 0.9.15)* An overlay entry whose name, parameter count or parameter names do not
  match the header is `E5010`, so overlays cannot drift. So is an entry outside a class section that
  names only members of classes (verdict 131): "`new` is a member of `Handle` and of `MutHandle`:
  write it under `class Handle:` or `class MutHandle:`".

**FFI-39b:**
p21-annex-c.md lines 128-130
* `[FFI-39b]` *(changed in 0.9.10)* `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and
  no declared `init` is `E5058`.

**FFI-48:**
p21-annex-c.md lines 83-93
* `[FFI-48]` *(changed in 0.9.10)* **Unsupported constructs** (`E5034`, naming the row):

  | Construct | Why not | Instead |
  |---|---|---|
  | multiple inheritance, virtual bases | the pointer adjustment is ABI-private and differs between MSVC and Itanium | a single-inheritance facade in C++ |
  | C++20 modules | no stable AST for a compiled module interface | parse the headers |
  | C++ coroutines | promise and frame layout are implementation-defined | a callback or completion handle |
  | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  | casts across a hierarchy the importer did not model | no Ember equivalent of `dynamic_cast` there | cast in C++ and export the result |
  | non-type template parameters of class type | mangling differs between compilers | instantiate in C++ and export a typedef |
  | allocator-parameterised containers | the allocator is part of the type and opaque | expose a span, or keep it native |

**GRM-35:**
p16-ffi.md lines 251-282
* `[GRM-35]` *(new in 0.9.9)* *(changed in 0.9.15)* **Overlay grammar.** An overlay is a file whose items are:

  ```text
  overlay_decl  := ["unsafe"] "overlay" ("c" | "cpp") STRING
                   ":" NEWLINE INDENT {overlay_item} DEDENT
  overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  contract      := IDENT ["(" [attr_arg {"," attr_arg} [","]] ")"]
  ```

  In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember
  type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147). An
  item may begin with `override`, replacing an earlier overlay's entry for that name (`[FFI-30c]`,
  ODR-149). An `overlay_class` section holds the entries of a C++ class's members (verdict 131): its
  methods, which list no receiver, its constructors (`new`) and its static functions, each written as
  any other entry; `@ffi(const)` above a method's entry asserts that the method only reads its object,
  and an entry without it makes the method one that changes its object (`[FFI-40a]`, ODR-160, verdict
  132). An entry outside a class section is a function outside any class
  (`[FFI-12]`). Ember records the header each overlay was written for in `ember.overlays`,
  never in the overlay (`[FFI-55]`).

  Contract words take the place of types and are separated by spaces. `overlay`, `rename` and `hide`
  are contextual keywords (`[LEX-15]`). An overlay whose items state any fact — a contract word, an
  `@ffi(…)` attribute, or a function entry, which asserts the function safe to call or, written
  `unsafe fn`, states what its callers must make true (`[FFI-2]`) — is an
  assertion about foreign code and MUST be declared `unsafe overlay` (`E5067`), a boundary of
  `[TIER-1]`; an overlay of only `rename` and `hide` items needs no `unsafe`.

**TYP-26:**
p04-types.md lines 867-871
* `[TYP-26]` *(changed in 0.9.9)* There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. A
  function whose last positional parameter takes a pack's values (`[TYP-47]`) takes any number of
  arguments; the output functions `print`, `println`, `eprint`, `eprintln` (`[STD-9]`) and `format_to`
  (`[STD-18]`) are such functions of the library, not known to the compiler by name (verdict 117).

## rules_not_found
