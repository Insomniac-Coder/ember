# K21: How do C++ overloaded functions and constructors import into Ember?

## Design 1

**question**: K21 (J8-04): How do C++ overloaded functions and constructors import into Ember? How is an overload set named, how is an ambiguous use refused, how does an overlay entry (and `rename`/`hide`) name one overload, what does `ember bind` write, and which codes; with default arguments, `super.init` ([FFI-39b]) and overrides covered, nothing running while the program runs, and nothing of the whole program read by a package ([BLD-16]).

**summary**: A C++ name the header declares for several functions (functions, methods, static functions, constructors as `new`, the listed instances of a function template, a function with default arguments once per count of arguments it accepts) is one Ember name, exempt from [TYP-26] because the program declared none of them; each use stands for one function, chosen when compiling from the use's own words in three steps that reuse [TYP-24]'s ranking (count and names; exact types, a literal as its default type, the receiver as its name's kind; then [TYP-5]'s coercions, a literal as a type of its own kind, a changeable name reaching a reading method as a view), and none or several is the new `E5035`, naming each function and the line to write. An overlay entry, a `rename` and a `hide` name one function by the name alone where it has one, otherwise by the parameters' count, names, words and types (and a result type) written in Ember's own spelling, `@ffi(const)` naming the `const` one of a pair; an entry fitting none or several is `E5010`; `ember bind --init` writes one entry per function and `ember.overlays` keeps one keyed line per function. Every function keeps its own thunk, entry, `@safety` and record line, so a call is the same direct, LTO-inlinable thunk call as any imported C++ function: no table, no tag, no check, no new row in §X.4.

- EDIT rule=code table row E5056 (§XVII registry, line beginning "| `E5056` |")
  - old: | `E5056` | an `override fn` matching no virtual method of the C++ base, by name and signature | `[FFI-39]` |
  - new: | `E5056` | an `override fn` matching no virtual method of the C++ base, by name and exact signature (no coercion), among several virtual methods of one name included | `[FFI-39]` |
  - why: The row says what [FFI-39] now says: an override is matched exactly, so `override fn on(e: KeyEvent)` against `on(const Event&)` (which imports `e: Event`) is refused instead of being reached by an upcast that runs backwards for an override (the earlier design's hole: a FocusEvent read as a KeyEvent in safe code).

- EDIT rule=FFI-39
  - old: An `override fn` matching no virtual
  method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual
  method of the base, by name and exact signature (its parameters the types `[FFI-40]` imports, with no
  coercion), is `E5056`; of several virtual methods of one name (`[FFI-17g]`) it replaces the one it
  matches so, and the others stay the base's.
  - why: The cause of the earlier design's unsafe hole is coercion in override matching: an argument C++ passes flows into the Ember parameter, so a fit by widening or upcast is a narrowing in that direction. Exact matching is what the text already meant by "signature"; the clause says it and names the overloaded case.

- EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`);
  - new: `super.init(…)` selects the C++ base constructor as `[FFI-17g]` chooses among
  a name's several functions (by its arguments' count, names and types) and runs it at `super.init(…)`
  (`[CLS-11]`);
  - why: One selection rule for every use of a C++ name; `super.init` was the only place the text already chose by arguments, so it now cites the general rule instead of carrying a private one.

- EDIT rule=FFI-17
  - old: one thunk per imported function, method, constructor and destructor.
  - new: one thunk per imported function, method, constructor and destructor, each of a name's several
  functions (`[FFI-17g]`) and each count of arguments a default argument lets a function accept its own
  (the thunk passes the arguments given and C++ supplies the defaults, as at a C++ call).
  - why: Says what the generator emits for an overload set and for default arguments: a direct thunk per function and per accepted arity, so a call is one direct call, inlined under the project's LTO (`[BLD-FFI-4]`), and a default argument costs exactly what the C++ call site pays.

- EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. A C++
  header's several functions of one name reach Ember as one name (`[FFI-17g]`), not as `E1030`: the
  program declares none of them, and each use stands for one of them, chosen when compiling.
  - why: [TYP-26] is where the refusal is caused; the exception states its reason (the functions are the header's, not the program's) and stays general over every kind of C++ name. §I.6's non-goal stands: an Ember program still cannot declare two functions of one name.

- EDIT rule=FFI-12
  - old: An overlay entry whose name, parameter count or parameter names
  do not match the header is `E5010`, so overlays cannot drift.
  - new: An overlay entry whose name, parameter count or parameter names
  do not match the header is `E5010`, so overlays cannot drift; so is an entry, a `rename` or a `hide`
  that fits none, or more than one, of a name's several functions (`[FFI-17g]`, `[GRM-35]`) by what it
  writes, the message naming each and the word to add ("`fn set(name: borrowed nul_terminated, value)`
  fits two functions of `Shader`, `set(const char*, int)` and `set(const char*, float)`; write the type
  that tells them apart: `value: c_int` or `value: f32`").
  - why: An entry vouches for one function (verdict 131's problem was a line vouching for two). The refusal is at the rule that already matches entries, with the same code, and its message gives the line to write.

- EDIT rule=FFI-55
  - old: 
  - new: A function of a name that has several (`[FFI-17g]`) has a line of its own, keyed by the words of its
  entry that name it, quoted as TOML asks (`"Renderer.draw(mesh: Mesh)" = "blake3:…"`), its fingerprint
  that one declaration's, so that a changed parameter type of either function is a changed
  declaration; a default argument's expression is still not.
  - why: The record keyed by name alone could hold one fingerprint for two functions; keyed by the entry's own words it checks each function the overlay describes (verdict 121: Ember writes the record, never a person), and the earlier design's "the key is tooling" hole closes.

- EDIT rule=CLI-5
  - old: 
  - new: For a name of several functions (`[FFI-17g]`) it writes one entry per function, with the
  parameters' words and types that tell them apart and no more.
  - why: `ember bind --init` is where the starter overlay is written; a name of several functions needs one entry each, written with the least that names it.

- EDIT rule=FFI-48 table row "overloads differing only in return type"
  - old:   | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  - new: 
  - why: A C++ header cannot declare two functions differing only in the result (ill-formed), so the row refused nothing and sent the reader to a `rename` that could not name one overload. The case that can occur, two functions alike in all Ember sees (template instances differing only in the result; `int` and `long` where a target makes both `i32`), is refused at the call by `[FFI-17g]` (`E5035`) with a `hide` or `rename` by parameters as the way out, and the header stays imported.

- EDIT rule=code table row E5010
  - old: | `E5010` | overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes | `[FFI-12]`, `[FFI-55]` |
  - new: | `E5010` | overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes; an entry, `rename` or `hide` that fits none or several of a name's C++ functions | `[FFI-12]`, `[FFI-55]` |
  - why: The registry row names every case its defining rules give.

- EDIT rule=code table, new row after E5034
  - old: | `E5034` | unsupported C++ construct | `[FFI-48]` |
  - new: | `E5034` | unsupported C++ construct | `[FFI-48]` |
| `E5035` | a use of a C++ name of several functions that none fits, or that several fit equally (a function declared `= delete` chosen included); a function value of such a name with no expected type | `[FFI-17g]` |
  - why: One code, one defining rule: `E2070` stays [TYP-24]'s ambiguous interface method; the imported set gets its own code, defined by the new rule.

- EDIT rule=GRM-35 grammar block
  - old:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  - new:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | overlay_name | extend_decl | overlay_class)
  overlay_name  := "rename" member_ref "as" IDENT NEWLINE | "hide" member_ref NEWLINE
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} (overlay_fn | overlay_name)} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" (type {contract} | contract {contract})] NEWLINE
  overlay_param := [param_word] IDENT [":" [type] {contract}]
  member_ref    := IDENT ["(" [overlay_param {"," overlay_param} [","]] ")" ["->" type]]
  - why: The one entry form gains the words Ember's own `param` already has (`[param_word] IDENT ":" type`), optional, and `rename`/`hide` gain a parameter list, so one line can name one of several functions and a member line can be renamed or hidden under its class (verdict 131). No second entry form, no C++ text in a string.

- EDIT rule=GRM-35 prose (append)
  - old: 
  - new: A parameter's word (`ref mut`, `owns`) and type, and the result's type, are written where they tell a
  name's several functions apart (`[FFI-17g]`), and may be written anywhere; each is the one the import
  gives (`[FFI-8]`, `[FFI-17a]`, `[FFI-40]`) and is checked against it (`[FFI-12]`). After the colon a
  word that is one of `[FFI-11]`'s is a contract word and any other is the type, so a type whose name is
  a contract word is first given another (`rename status as Status`); where one target's types make two
  functions alike, the importer's spelling tells them apart (`c_long` against `c_int`). An entry, a
  `rename` or a `hide` names one function: by the name alone where it has one, else by its parameters'
  count, names, words and types; `@ffi(const)` on a method's entry names the `const` one of two methods
  alike otherwise (`[FFI-40a]`, verdict 132). `rename` with parameters gives that function a name of its
  own, outside the set; `hide` with parameters removes it from the set; either needs no `unsafe`.
  - why: States how the new grammar is read, with no new vocabulary: types are Ember's, contract words are the closed list of [FFI-11], and the one collision is solved by a `rename` the grammar already has.

- EDIT rule=Annex C §C.1, new bullet after [FFI-17f]
  - old: 
  - new: * `[FFI-17g]` *(new in 0.9.20)* **Several C++ functions of one name.** A C++ name the header declares
  for several functions — a namespace's or a class's functions, a class's constructors (`new`), its
  static functions, a virtual method and its overloads, the instances of a function template listed in
  `instantiate=[…]` (`[FFI-17b]`), a function with default arguments counting once for each number of
  arguments it accepts (never a count that leaves out a parameter another parameter's contract word
  names), a redeclared function counting once (`[FFI-30]`) — is one Ember name, as `[FFI-40]`,
  `[FFI-41]` and `[FFI-44]` give it, and not `E1030` (`[TYP-26]`): the program declares none of them.
  Each use of the name stands for one function, chosen when compiling from the use's own words and
  nothing around it: (1) the functions the call's number of arguments and the names it gives fit; one
  left is chosen, and the call is then checked against it as any call (an untyped literal taking the
  type that parameter expects, `[LEX-16]`, `[LEX-17]`); (2) of several, the one whose parameters are
  the arguments' own types, an untyped literal counting as its default type (`int`, `f64`), and whose
  object a method's receiver fits as its name is (a name that may change the object fits a method that
  may change it, a name that only reads fits one that only reads, `[FFI-40]`; `this` is a name that may
  change the object where any other line of its method changes it, else one that only reads, as what a
  method does to its object is read from its code, `[CLS-7]`, verdict 86); (3) else the one every
  argument reaches by a coercion of `[TYP-5]`, an untyped literal as a type of its own kind (an
  integer literal a whole-number type, a float literal a float type), a name that may change the
  object reaching a method that only reads it as a view (`[TYP-5]` rule 7); of a `const` and a
  non-`const` method alike in all else as imported, the one without `const` (C++'s choice for an
  object that may change; an entry with `@ffi(const)` makes them differ, `[FFI-40a]`). A function
  declared `= delete` takes part and, chosen, is refused as deleted. None, or several at one step, is
  `E5035`, naming each function as the header declares it and why it does or does not fit, with the
  line to write ("`out.put(5)` (line 6) fits `void put(int v)` and `void put(long v)` of `io::Out`
  alike: `5` is a whole number either takes; write `out.put(5 as c_int)` or `out.put(5 as c_long)`;
  where this target makes `int` and `long` one type, hide the one not meant in the overlay, under
  `class Out:`: `hide put(v: c_long)`"). `super.init(…)` chooses the base constructor so
  (`[FFI-39b]`); an `override fn` matches one virtual method by its exact signature (`[FFI-39]`); a
  function value of the name is the function whose type its place expects exactly (`[TYP-5]` rule 10),
  and with no expected type `E5035`. What the code around the use permits — `unsafe:`, effects, the
  loop and view rules, what a name allows — is checked afterwards, on the chosen function, so a line
  stands for the same function wherever it stands. Each function has its own thunk (`[FFI-17]`), its
  own overlay entry (`[GRM-35]`, `[FFI-12]`), its own `@safety` and `L3015` (`[UNS-7]`, verdict 125)
  and its own line of `ember.overlays` (`[FFI-55]`); the chosen function is called as any imported C++
  function is, one direct call of its thunk, and nothing runs while the program runs for this rule.
  - why: The missing rule, placed with the C++ import rules it governs. It is a property (a name of several functions and what a use's words fit), never a program's shape; it reuses the ranking Ember already has for one name's interface instances ([TYP-24], ODR-096) and the receiver rule [FFI-40] already states; it reads only the import and the use's own words, so a package decides it alone ([BLD-16]); and it adds no instruction to any call.

**refused_program**: // stream.hpp (C++)
// #include <cstdio>
// namespace io {
// class Out {
// public:
//     Out() noexcept;
//     void put(int v) noexcept { std::printf("int %d\n", v); }
//     void put(long v) noexcept { std::printf("long %ld\n", v); }
// };
// }
# overlays/stream.em
unsafe overlay cpp "stream.hpp":
    class Out:
        fn new()
        fn put(v: c_int)
        fn put(v: c_long)

# src/main.em
import cpp "stream.hpp" with (project="tools", overlay="overlays/stream.em") as s

fn main():
    out = s.io.Out.new()
    out.put(5)

Refused with E5035 at src/main.em:6: "`out.put(5)` fits `void put(int v)` and `void put(long v)` of `io::Out` alike: step 1 keeps both (one argument, no name), step 2 fits neither (`5` is an `int`, 64 bits; `put(int)` takes a `c_int`, `put(long)` a `c_long`), step 3 fits both (`5` is a whole number either takes). Write the one meant: `out.put(5 as c_int)` or `out.put(5 as c_long)`; where this target makes `int` and `long` one type (`i32`), hide the one not meant in the overlay, under `class Out:`: `hide put(v: c_long)`." (On Linux the cast decides; on MSVC, where `c_int` and `c_long` are both `i32`, the two functions are alike to Ember and the `hide` line is the way out; the overlay's two entries still each name one function, by the importer's spelling `c_int`/`c_long`.) The same program with `fn put(v)` twice in the overlay is E5010 at the first of them: "`fn put(v)` fits two functions of `Out`, `put(int v)` and `put(long v)`; write the type that tells them apart: `v: c_int` or `v: c_long`".

**accepted_program**: // renderer.hpp (C++)
// namespace gfx {
// struct Mesh { int vertices; };            // standard-layout: imported by value ([FFI-32])
// struct Sprite { int frame; };
// class Renderer {
// public:
//     Renderer() noexcept;
//     Renderer(int width, int height) noexcept;
//     void draw(const Mesh& mesh) noexcept;
//     void draw(const Sprite& sprite) noexcept;
//     void set(const char* name, int value) noexcept;
//     void set(const char* name, float value) noexcept;
//     void resize(int width, int height, bool keep_aspect = true) noexcept;
//     const Mesh& last(size_t i) const noexcept;   // defined in renderer.cpp
//     Mesh& last(size_t i) noexcept;               // defined in renderer.cpp
// };
// }
# overlays/renderer.em
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        fn new()
        fn new(width, height)
        fn draw(mesh)
        fn draw(sprite)
        fn set(name: borrowed nul_terminated, value: c_int)
        fn set(name: borrowed nul_terminated, value: f32)
        fn resize(width, height, keep_aspect)
        @ffi(const)
        fn last(i) -> borrowed one from(this)
        fn last(i) -> borrowed one exclusive from(this)

# src/main.em
from std.ffi import c_int
import cpp "renderer.hpp" with (project="engine", overlay="overlays/renderer.em") as eng

fn vertices_of(r: eng.gfx.Renderer) -> c_int:
    return r.last(0).vertices             # const Mesh& last(size_t) const: `r` only reads, so only the reading `last` fits

fn main():
    mesh = eng.gfx.Mesh(vertices=36)
    hud = eng.gfx.Sprite(frame=2)
    r = eng.gfx.Renderer.new(1280, 720)   # Renderer(int, int): the one `new` of two arguments; the literals take `c_int`
    r.draw(mesh)                           # draw(const Mesh&): exact on `Mesh`
    r.draw(hud)                            # draw(const Sprite&): exact on `Sprite`
    r.set(c"light_count", 4)              # set(const char*, int): `4` is a whole number, so only `c_int` is of its kind
    r.set(c"exposure", 1.5)               # set(const char*, float): `1.5` is a float, so only `f32` is of its kind
    r.resize(640, 480)                     # resize(int, int, bool = true): the two-argument thunk; C++ fills keep_aspect
    r.last(0).vertices = 12                # Mesh& last(size_t): `r` may change, so the changing `last` is exact and the reading one only a view
    println(vertices_of(r))                # prints 12

Every call compiles to one direct call of the chosen function's thunk. The `last` pair's bodies are out of line, so without the `@ffi(const)` entry both would import as changing and tie; the entry names the `const` one and makes it reading (verdict 132's words), after which the receiver's name chooses, as C++'s `const` does.

**runtime_cost**: Nothing. Which function a use stands for is decided when compiling from the import and the use's own words; the chosen function is called as every imported C++ function already is: §X.4 row "FFI call | guaranteed present | one direct call for an ABI-direct C function", the thunk inlined under the project's LTO ([BLD-FFI-4]). No overload table, no type tag, no run-time test, no extra argument. A default argument costs exactly the C++ call site's cost: the arity's thunk passes what was given and the C++ compiler evaluates the default inside the thunk, as it would at a C++ call. The only new code is more thunks (one per function and per accepted arity), which is code size: implementation-defined under [COST-2], not a cost of running. [COST-5] asks for no new row because no instruction is added to any line.

**implementer_steps**: 1. Importer (libclang, writing the `.embind` of [FFI-14]): within each namespace and class, group declarations by their Ember name ([FFI-40], [FFI-41], constructors under `new`), one function per Clang USR so redeclarations merge ([FFI-30]); for each function record its parameters (names, Ember type by the [FFI-8]/[FFI-17a]/[FFI-40] tables and the importer's spelling such as `c_long`, the word `ref mut`/`owns`), the method's `const` qualifier and what [FFI-40a] derives, `= delete`, the least count of arguments its default arguments allow, and the instances of each function template from `instantiate=[…]` ([FFI-17b]) as members of the template's name. 2. Thunk generation (`ember bind --emit-cpp`, [FFI-50]): for each function and each accepted arity emit one `extern "C"` thunk whose symbol is derived from the USR hash and the arity (so two packages importing one header emit the same symbol, given vague linkage or kept per package), calling the C++ function with the arguments given; the C++ compiler fills the defaults. 3. Name resolution in the type checker: an imported name resolves to its set; a call runs the three steps of [FFI-17g] before the ordinary call check, with step 2 and 3 reusing the fit tests the TYP-24/ODR-096 instance chooser already has (exact type; literal default type; the coercion list of [TYP-5]; a literal of its own kind); a function-value use consults the expected callable type only; `super.init` and `override fn` call the same matcher, the override in exact mode (E5056). 4. The receiver's kind comes from the place's declaration (local or `ref mut`: may change; plain parameter, loop or pattern name: reads); for `this`, compute the method's change summary with its calls to const/non-const pairs left out, then resolve those calls by the result (the fixed point verdict 86's inference already computes for every method); apply the `const` tie-break where the pair is alike. 5. Everything after selection is unchanged: E3023, E3100, L3015, the effect and loop rules run on the one chosen function. 6. Overlay matcher ([FFI-12], [GRM-35]): parse `[param_word] IDENT [":" [type] {contract}]`, classifying the first word after the colon by the closed [FFI-11] list; an entry, `rename` or `hide` maps to the functions it fits by name, count, names, words, types (by imported spelling), `@ffi(const)` and result type; zero or several is E5010 with the distinguishing word in the message; `rename` with parameters moves that function out of the set under the new name, `hide` drops it. 7. `ember.overlays` writer ([FFI-55]): one line per function keyed by the entry's own naming words, quoted for TOML; the fingerprint is already the canonical type plus names, so a changed overload is caught. 8. `ember bind --init` ([CLI-5]) writes one entry per function with the fewest words that name it; `--report` ([CLI-6]) may list each name whose functions this target's types make alike. 9. Diagnostics: an E5035 builder that prints each candidate as the header declares it, the step at which it fell out and why, and the cast or `hide` line; L3015 and E5010 name the function with the words that tell it apart. 10. Tests: the two programs above on gcc, clang and MSVC (the alias case differs by target); break tests per step (remove step 2 and `r.draw(mesh)` ties; remove the exact override rule and the KeyEvent override is accepted; remove the `const` tie-break and the out-of-line pair ties); `ember bind --emit-cpp` inspected for one thunk per function and arity; a run printing which overload ran.

**departs_from_verdict**: none. Verdict 125 (a C++ function a program calls in unsafe code carries `@safety` in its entry) holds per function: each overload has its own entry and its own L3015. Verdict 131 (member lines under their class) holds, and `rename`/`hide` of a member go under the class too. Verdict 132 (`@ffi(const)` keeps a method const, an entry without it makes it changing) holds and is what names the `const` one of an alike pair. Verdict 121 (`ember.overlays` written by Ember, never by hand) holds; only its key becomes per function. Verdict 86 (what a method does to its object is read from its code) is what decides `this`. Verdict 117 and [TYP-26]'s pack clause are untouched.

**trade_offs**: Speed: identical in every option considered (one name, mangled names, overlay-only names): a direct thunk call chosen when compiling; this design adds thunks per arity (code size) and one fit test per candidate per use at compile time, nothing else. Safety: refuses rather than guesses at every tie, matches overrides exactly (closing the earlier design's backwards-coercion hole), and checks `unsafe:`/E3023/effects on the one chosen function so a line never changes meaning by moving. Given up against C++: (a) no per-argument ranking and no promotion-before-conversion, so a narrow number or a bare literal among `int`/`long`/`long long`/`double` overloads that C++ ranks is refused with a cast as the way out (`lives as c_int`, `5 as c_int`); (b) a bare literal is Ember's `int` (i64), so between `f(int)` and `f(long long)` it is exact on the latter where C++ takes the former, kept because one literal has one meaning everywhere ([TYP-24], [LEX-16]) rather than a C++ meaning inside C++ calls; (c) Ember may pick one function where C++ is ambiguous (an `i64` literal for `value(int64_t)`/`value(double)`), never one C++ would not consider. Readability: ordinary C++ lines compile unchanged; overlay entries carry a word or type only where parameter names do not tell functions apart; `this` resolves by the method's inferred constness, which is not written on the line, as no change fact is since verdict 86. Consistency: one rule for calls, `super.init`, overrides, function values and overlay entries; no second entry form; `rename`/`hide` gain the parameter list that FFI-48's "rename one in the overlay" always needed.

**remaining_doubt**: (1) `c_int` and `c_long` are aliases (§XV: "`c_int` and the other C aliases"), so on MSVC `f(int)`/`f(long)` are alike to Ember and every call ties until an overlay `hide`; a program portable across targets writes that line, or `c_long` becomes a distinct type, which is another question (E1-12). (2) Instances of one function template that differ only in template arguments or the result tie; the overlay names one by `rename get() -> f64 as get_f64`, but an Ember spelling of an instance at the call (`eng.get[f64]()`) belongs to the templates question ([FFI-17b]). (3) `this` deciding by the method's other lines is the owner's style (verdict 86) but not his ruling; the alternative, `this` always a name that may change, costs nothing more and refuses the wrapper-class pattern through plain parameters. (4) A C++ parameter the header leaves unnamed cannot be given by name at a call or told apart by name in an entry; the type then does the work. (5) The const/non-const `operator[]` pair goes through `Index`/`IndexMut` ([FFI-42]), not this selection: K47's question, unchanged here. (6) The exact wording of E5035 and of the per-overload L3015 is the lead's.

## Design 2

**question**: K21: How do C++ overloaded functions and constructors import into Ember? (How an overload set is named, how a use picks one member, how ambiguity is refused, how an overlay entry names one overload, what `ember bind` writes, the codes; default arguments and const/non-const pairs included.)

**summary**: A C++ name the header declares several times imports as one Ember name of several members, one per declaration (a const/non-const pair is first split into `at` and `at_mut`, as `get`/`get_mut` and `span`/`span_mut` already are; a default argument gives one member per arity, its thunk letting C++ supply the default). Each use stands for the one member its own words fit — the arguments' number, the word before each (`ref mut`, `owns`, none) and their types, an exact fit before a coercion as `[TYP-24]` already ranks, an untyped literal exact for C++'s `int`/`double` — and only then is the chosen member checked for `unsafe:`, read-only names and effects, so the same line runs the same function wherever it stands; none or a tie is `E5035` naming each member's first line and the cast or `rename` line to write. An overlay entry, `rename` or `hide` of such a name writes each parameter's type (`fn set(name: *c_char borrowed nul_terminated, value: f32)`), so it names exactly one declaration and `ember.overlays` keys it per member; an `override fn` matches a virtual member exactly, an upcast or widening counting for nothing (`E5056`). Nothing runs while the program runs: a use is a direct call of the member's own thunk, named by its mangled name.

- EDIT rule=§C.1, new rule [FFI-57] placed directly after [FFI-32]
  - old: 
  - new: * `[FFI-57]` *(new in 0.9.20)* **A name declared several times.** A C++ name the header declares with several parameter lists — a function of a namespace, a method, a static member function, a constructor (`new`, `[GRM-35]`), or the explicit instantiations of one template (`[FFI-17b]`) — imports as one Ember name of several **members**, one per declaration, each with its own thunk (`[FFI-17]`), its own identity (`[FFI-30]`), its own entry and its own line of `ember.overlays` (`[FFI-55]`). A const and a non-const pair is split first (`[FFI-40b]`), and a parameter with a default argument makes its function a name of several members too: one per arity the defaults allow, each arity's thunk calling the function with the trailing arguments left out, so that C++ supplies the defaults as it would at a call of its own. Each use of such a name stands for one member, read from the use alone. A call (a method call, `T.new(…)`, `super.init(…)`, `[FFI-39b]`) fits a member when that member, declared alone, would accept its arguments: their number, the word before each (`ref mut`, `owns`, none: `[FN-2a]`, `[FFI-40]`) and their types under `[TYP-5]`; of the members it fits, one whose every parameter's type is its argument's own type — an untyped literal counting as its own for a parameter whose C++ type is `int` (a whole number) or `double` (a fraction), the types C++ gives those literals — is chosen before one that some argument reaches only through a coercion (`[TYP-24]`). A name used without a call stands for the member whose first line is exactly the callable type expected there (`[CLO-3]`). What the code around the use allows — unsafe code (`[FFI-2]`), a read-only name (`E3023`), its effects — is checked after, on the member chosen, so one line stands for one member wherever it is written. A use that fits no member, or two with none exact, is `E5035`, which names each member by its first line and why it does not fit and, for a tie, the argument to write with its type or the `rename` line that gives one member a name of its own: "`c.show(lives)` fits two members of `ui.Counter.show`, `show(value: c_int)` and `show(value: i64)`, each by widening `lives` (`u8`); write `lives as c_int` or `lives as i64`". Two members whose first lines are one on the target compiled for (`put(v: c_int)` and `put(v: c_long)` where `long` is 32 bits) fit every call together; the `rename` line tells them apart on every target, since an overlay writes the table's spelling of the C++ type (`c_long` for `long`, `[FFI-8]`, `[FFI-17a]`), not the target's width. The entry of a member in an overlay writes each parameter's type, as the import maps the C++ type, as the first word after its colon, before its contract words (`fn set(name: *c_char borrowed nul_terminated, value: f32)`), and so does a `rename` or `hide` of one member (`rename put(v: c_long) as put_long`, `[GRM-35]`); an entry, `rename` or `hide` of such a name that writes no type, or whose types name no member or two, is `E5010`, whose message gives each member's entry line (`[FFI-12]`); an entry of a name declared once is written as before. `L3015` (`[UNS-7]`) and `E5002` (`[FFI-2]`) give a member's typed entry line. `ember bind --report` lists each name of several members with the first line of each; `ember bind --emit-cpp` writes one thunk per member, named by the member's mangled name (`[MNG-1]`). Nothing runs for this while the program runs: a use is one direct call of the member's thunk, as any imported C++ function's.
  - why: Annex C is silent on the construct nearly every C++ header has. The rule is a property (a name declared several times), decided from the header and the line alone (`[BLD-16]`), with no coercion ranking beyond `[TYP-24]`'s, so a call never silently runs a member C++ would not and is refused, with the line to write, wherever it is unclear.

- EDIT rule=§C.3, new rule [FFI-40b] placed directly after [FFI-40a]
  - old: 
  - new: * `[FFI-40b]` *(new in 0.9.20)* **A const and a non-const pair.** A non-`const` method beside a `const` method of the same name and the same parameters, as the import maps them, imports with `_mut` added to its name — `at` and `at_mut`, as `get` and `get_mut` (`[FFI-42]`) and `span` and `span_mut` (`[FFI-17a]`) — so the two are two names, and which one a call runs is read from the call (`cfg.at(i)`, `cfg.at_mut(i).port = 8080`), never from the name the object is reached through nor from the lines after it; `at_mut` may change its object (`[FFI-40]`), and the `const` one is read as `[FFI-40a]` says. An overlay entry names each by its Ember name (`@ffi(const) fn at(i)`, `fn at_mut(i)`, `[FFI-12]`). A class that itself declares `at_mut` beside such a pair is the `[FFI-48]` row.
  - why: Choosing by the receiver's kind is circular inside a class method (whether `this` may change is worked out from the body that contains the call) and lets a read-only parameter silently run another function than a local does. Two names make the choice a word on the line, which Ember's bridge already does for `get_mut` and `span_mut`, and let an overlay vouch for either member.

- EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types. A C++ name a header declares several times is one Ember name of several members, each use standing for one (`[FFI-57]`): the declarations are the header's, which a program cannot change, and a function Ember declares beside an imported name is `E1030` as any other.
  - why: The rule as written refuses every C++ class with two constructors; the exception says where it stops and why.

- EDIT rule=§I.6 Non-goals
  - old: implicit numeric narrowing; function overloading
  - new: implicit numeric narrowing; function overloading (a C++ header's overloads import as one name of several members, each use standing for one, `[FFI-57]`)
  - why: The non-goal stays true for Ember's own functions; the cross-reference stops a reader finding Annex C contradicting Part I.

- EDIT rule=FFI-17
  - old: one thunk per imported function, method, constructor and destructor
  - new: one thunk per imported function, method, constructor and destructor, a name declared several times giving one per member and a default argument one per arity (`[FFI-57]`)
  - why: The thunk count is where a reader of this rule asks what two constructors become.

- EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and no declared `init` is `E5058`.
  - new: `super.init(…)` runs the C++ base constructor its arguments select as `[FFI-57]` selects a member of the base's `new` (an exact fit before a coercion; none or a tie `E5035`) and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and no declared `init` is `E5058`.
  - why: One selection rule for constructors wherever they are called; "by arity and argument types" said neither how a literal counts nor what a tie is.

- EDIT rule=FFI-39
  - old: An `override fn` matching no virtual method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual method of the base, by name and first line — each parameter's word and type exactly those the import gives the virtual method's (`[FFI-40]`), and its result; an upcast or a widening counts for nothing, since C++ hands the override its own argument (a `FocusEvent` for an `on(const Event&)`) — is `E5056`, whose message gives each virtual method of that name as the `override fn` line to write; where the base declares the name several times (`[FFI-57]`), the override replaces the one member it matches and the trampoline passes on that one, the others staying the base's.
  - why: An override matched through an upcast would read a smaller base object as a derived one in safe code, and one matched through a widening would truncate C++'s argument: both are what the checker exists to refuse. "By name and signature" is made exact and told how an overloaded virtual is replaced.

- EDIT rule=FFI-12
  - old: An overlay entry whose name, parameter count or parameter names do not match the header is `E5010`, so overlays cannot drift.
  - new: An overlay entry whose name (a member's Ember name, `at_mut` for `[FFI-40b]`'s), parameter count or parameter names do not match the header, or, for a name declared several times, whose parameter types name no member or two (`[FFI-57]`), is `E5010`, so overlays cannot drift and no entry vouches for a function it does not name.
  - why: Name, count and parameter names cannot tell `set(const char*, int)` from `set(const char*, float)`; an entry must name one declaration or it vouches for one nobody looked at.

- EDIT rule=FFI-55
  - old: 
  - new: A member of a name declared several times (`[FFI-57]`) has a line of its own, keyed by the declaration as the header spells it, quoted (`"gfx::Renderer::draw(const gfx::Mesh&)" = "blake3:…"`); a `rename` in the overlay changes its key not at all.
  - why: The record is keyed by name; with two declarations of one name the check of verdict 121 would compare an entry against the wrong declaration or lose one.

- EDIT rule=FFI-30c
  - old: a later file's `override` entry replaces an earlier file's for that name, and any other second entry for one name is `E5010` (ODR-149).
  - new: a later file's `override` entry replaces an earlier file's for that name — for a name declared several times, for the member its types name (`[FFI-57]`) — and any other second entry for one name or member is `E5010` (ODR-149).
  - why: Composition counts entries by name; two typed entries for two members of one name are not a second entry for one name.

- EDIT rule=GRM-35 grammar
  - old:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  - new:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" overlay_name "as" IDENT NEWLINE | "hide" overlay_name NEWLINE
                   | extend_decl | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn
                   | "rename" overlay_name "as" IDENT NEWLINE | "hide" overlay_name NEWLINE} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" [type] {contract}]
  overlay_name  := IDENT ["(" [IDENT ":" type {"," IDENT ":" type} [","]] ")"]
  - why: `rename` and `hide` take a bare name and cannot name one member; a class section allows no `rename` or `hide` of a member (verdict 131 puts member lines under the class); a parameter's slot needs the type position `[FFI-57]` uses.

- EDIT rule=GRM-35 prose
  - old: In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147).
  - new: In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147). In an entry, a `rename` or a `hide` of a name the header declares several times, the first word after a parameter's colon is its type, as the import maps the C++ type, and the contract words follow (`[FFI-57]`); in every other entry the words after the colon are contract words. A class section also holds the `rename` and `hide` items of its members.
  - why: The grammar alone cannot tell a type from a contract word; the reading is fixed by a property of the header (the name is declared several times), never by what the word happens to spell.

- EDIT rule=FFI-48 table, row "overloads differing only in return type"
  - old: | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  - new: | overloads differing only in return type (template instantiations of one parameter list) | Ember has no return-type overloading, and no overlay line can tell two first lines apart by their result (`[FFI-57]`) | a C++ wrapper of its own name for each |
  - why: The row's "rename one in the overlay" claimed a `rename` that cannot name either member, since the two write one first line.

- EDIT rule=FFI-48 table, new row after "allocator-parameterised containers"
  - old: 
  - new: | a const and non-const pair whose `_mut` name the class also declares | one name would hold two members (`[FFI-40b]`) | `rename` one in the overlay |
  - why: The generated name may collide with the header's own; the collision is refused, naming the row, with the way out.

- EDIT rule=FFI-44 table, new row after "explicit template instantiations"
  - old: 
  - new: | a name declared several times (overloads, default arguments, a const and non-const pair) | automatic (`[FFI-57]`, `[FFI-40b]`) |
  - why: The table says what imports and how; the most common construct of C++ headers was missing from it.

- EDIT rule=MNG-1
  - old: A method's path is its type's path and its name, with the interface's path before the name for a method told apart by the interface it implements (`[TYP-24]`).
  - new: A method's path is its type's path and its name, with the interface's path before the name for a method told apart by the interface it implements (`[TYP-24]`); a member of a C++ name declared several times (`[FFI-57]`) adds, after its name, `O` and the 32 hex digits of the BLAKE3 hash of its declaration's canonical parameter types and qualifiers as the header spells them, so that no two members share a symbol or a thunk.
  - why: Mangling by path alone gives two members one symbol; `[HR-7]` names call-table slots by mangled name, and `[FFI-3]`'s thunks need distinct `extern "C"` names.

- EDIT rule=COST-3 table, row "FFI call"
  - old: | FFI call | guaranteed present | one direct call for an ABI-direct C function |
  - new: | FFI call | guaranteed present | one direct call for an ABI-direct C function; one direct call of the member's thunk for a C++ name declared several times, and, for a call that leaves a default argument out, the default's own expression run in the thunk, as a C++ call site would run it (`[FFI-57]`) |
  - why: `[COST-5]`: every cost has a row; the only thing that runs is what the equivalent C++ call runs.

- EDIT rule=CLI-5
  - old: every derived contract, every unknown fact as a `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason.
  - new: every derived contract, every unknown fact as a `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason, and each member of a name declared several times as its own entry with its parameters' types (`[FFI-57]`).
  - why: The starter overlay must compile unchanged, so it writes the typed form where `[FFI-57]` requires it.

- EDIT rule=registry row E5034 (new row E5035 directly after it)
  - old: | `E5034` | unsupported C++ construct | `[FFI-48]` |
  - new: | `E5034` | unsupported C++ construct | `[FFI-48]` |
| `E5035` | a use of a C++ name declared several times (a call, `T.new(…)`, `super.init(…)`, a function value) that fits no member, or fits two with none exact | `[FFI-57]` |
  - why: One code, one defining rule; `E2070` belongs to `[TYP-24]`'s interface methods.

- EDIT rule=registry row E5056
  - old: | `E5056` | an `override fn` matching no virtual method of the C++ base, by name and signature | `[FFI-39]` |
  - new: | `E5056` | an `override fn` matching no virtual method of the C++ base by name and first line exactly, an upcast or a widening counting for nothing | `[FFI-39]` |
  - why: The row claims what its rule now says.

- EDIT rule=registry row E5010
  - old: an entry outside a class section that names only members of classes | `[FFI-12]`, `[FFI-55]` |
  - new: an entry outside a class section that names only members of classes; an entry, `rename` or `hide` of a name declared several times that writes no parameter types or whose types name no member or two | `[FFI-12]`, `[FFI-55]`, `[FFI-57]` |
  - why: The row lists every case of the code.

**refused_program**: // input.hpp (C++)
// namespace ui {
// class Event { public: Event() noexcept; virtual ~Event() noexcept; };
// class KeyEvent : public Event {
// public:
//     explicit KeyEvent(int code) noexcept : code_(code) {}
//     int code() const noexcept { return code_; }
// private:
//     int code_;                              // a KeyEvent is 16 bytes
// };
// class FocusEvent : public Event { public: FocusEvent() noexcept; };   // 8 bytes
// class Handler {
// public:
//     Handler() noexcept;
//     virtual ~Handler() noexcept;
//     virtual void on(const Event& e) noexcept {}
//     virtual void on(int timer_id) noexcept {}
// };
// void send_focus(Handler& h) noexcept;      // calls h.on(FocusEvent())
// }
from std.ffi import c_int
import cpp "input.hpp" with (project="ui") as gui

class KeyLog(gui.ui.Handler):
    last: c_int = 0

    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()

    override fn on(e: gui.ui.KeyEvent):      # line 12
        this.last = e.code()

fn main():
    log = KeyLog()
    # SAFETY: send_focus calls `on` on the handler during the call only
    unsafe:
        gui.ui.send_focus(ref mut log)

Refused at line 12 with `E5056`: "`on(e: gui.ui.KeyEvent)` overrides no virtual method of `ui.Handler`: the base declares `on(e: gui.ui.Event)` and `on(timer_id: c_int)`, and C++ hands `on` any `Event`, a `FocusEvent` included, never only a `KeyEvent`; write `override fn on(e: gui.ui.Event):` or `override fn on(timer_id: c_int):`". (Under a matching by coercion, the override would run with a `FocusEvent` taken as a `KeyEvent` and `e.code()` would read 8 bytes past the object in safe code.)

A second refusal, of a tie (`E5035`), for `// class Counter { void show(int); void show(long long); void show(double); }` imported as `hud`:
    lives: u8 = 3
    # SAFETY: the counter only prints the number it is given
    unsafe:
        c = hud.ui.Counter.new()
        c.show(lives)                         # line 6
"`c.show(lives)` (line 6) fits two members of `ui.Counter.show`, `show(value: c_int)` and `show(value: i64)`, each by widening `lives` (`u8`); `show(value: f64)` takes no whole number; write `lives as c_int` or `lives as i64`".

**accepted_program**: // renderer.hpp (C++)
// namespace gfx {
// struct Mesh { int vertices; };            // standard-layout: imported by value
// struct Sprite { int frame; };
// struct Entry { int port; };
// class Renderer {
// public:
//     Renderer() noexcept;
//     Renderer(int width, int height) noexcept;
//     void draw(const Mesh& mesh) noexcept;
//     void draw(const Sprite& sprite) noexcept;
//     void set(const char* name, int value) noexcept;
//     void set(const char* name, float value) noexcept;
//     void resize(int width, int height = 720) noexcept;
//     const Entry& at(size_t i) const noexcept;   // bodies in renderer.cpp
//     Entry& at(size_t i) noexcept;
// };
// }

# overlays/renderer.em
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        fn new()
        fn new(width: c_int, height: c_int)
        fn draw(mesh: Mesh)
        fn draw(sprite: Sprite)
        fn set(name: *c_char borrowed nul_terminated, value: c_int)
        fn set(name: *c_char borrowed nul_terminated, value: f32)
        fn resize(width, height)
        @ffi(const)
        fn at(i) -> borrowed one from(this)
        fn at_mut(i) -> borrowed one exclusive from(this)

# src/main.em
from std.ffi import c_int
import cpp "renderer.hpp" with (project="engine", overlay="overlays/renderer.em") as eng

fn port_of(r: eng.gfx.Renderer, i: usize) -> c_int:
    return r.at(i).port                   # the const `at`: the only member named `at`

fn main():
    mesh = eng.gfx.Mesh(vertices=36)
    hud = eng.gfx.Sprite(frame=2)
    r = eng.gfx.Renderer.new(1280, 720)   # Renderer(int, int): the member of two parameters
    r.draw(mesh)                          # draw(const Mesh&): `mesh` is a Mesh, exactly
    r.draw(hud)                           # draw(const Sprite&)
    r.set(c"light_count", 4)              # set(const char*, int): `4` is C++'s int, the exact fit
    r.set(c"exposure", 1.5)               # set(const char*, float): a float literal fits no c_int
    r.resize(1920)                        # resize(int, int = 720): the one-argument member; C++ supplies 720
    r.at_mut(0).port = 8080               # Entry& at(size_t): the call says `_mut`; `r` may change
    println(port_of(r, 0))                # 8080

Every call is decided from its own words when compiling; each is one direct call of that member's thunk. Moving a line into an `unsafe:` block, or reaching `r` through a read-only parameter, changes which member is checked, never which member is chosen.

**runtime_cost**: Nothing new. A use of a member is one direct call of that member's `extern "C"` thunk, the §X.4 row "FFI call" ("one direct call for an ABI-direct C function"), inlined across the boundary under the project's LTO as any imported C++ call (`[BLD-FFI-4]`). A call that leaves a default argument out runs the default's expression inside the arity's thunk, exactly what a C++ call site that omits the argument runs; the edit adds that clause to the "FFI call" row so `[COST-5]` is met. Selection, the const/non-const split, overlay matching and the override check are all done when compiling; no table, flag or test exists while the program runs. Code size grows by one thunk per member and per default-argument arity (implementation-defined under `[COST-2]`).

**implementer_steps**: 1. Importer (libclang pass, `[FFI-17]`): after mapping each declaration, group the functions of one scope by Ember name. Split const/non-const pairs with identical mapped parameters, naming the non-const one `<name>_mut` (`[FFI-40b]`); refuse with `E5034` (the new `[FFI-48]` row) when the class declares that name itself. For each function with k defaulted trailing parameters, make k+1 members, one per arity, whose thunk calls the C++ function with the trailing arguments omitted. Record for each member: its USR (`[FFI-30]`), its Ember first line (parameter words and types, with the table's spelling of each C++ type, `c_long` for `long`), its C++ declaration as the header spells it (the `ember.overlays` key), whether an untyped literal is exact for each parameter (C++ type `int` or `double`), and its mangled name under `[MNG-1]` with the `O` + BLAKE3 component. Emit one `extern "C"` thunk per member named by that mangled name (`ember bind --emit-cpp`), and in `.embind` (`[FFI-14]`) the member list per name.
2. Overlay checker (`[FFI-12]`, `[FFI-55]`, `ember bind --check`): for a name with two or more members, parse each entry's, `rename`'s and `hide`'s parameters as `IDENT ":" type {contract}`; match by name, count, parameter names and the written type spellings against the members' table spellings; `E5010` when no types are written, or none or two members match, the message listing each member's typed entry line. Write and compare one `ember.overlays` line per member, keyed by the header's spelling. `--init` writes the typed form for such names. For a name with one member, nothing changes.
3. Name resolution in the compiler: an imported name may denote a member set. At a call, `T.new(…)`, `super.init(…)` or a value use: for each member, run the existing argument-acceptance check (`[FN-2a]` words, `[TYP-5]`, untyped literals left open as `[LEX-16]`/`[LEX-17]` allow) with the unsafe, read-only-name and effect checks deferred; mark a fit exact when every argument's type is the parameter's own, an untyped literal counting as exact for a C++ `int`/`double` parameter; choose the sole fit or the sole exact fit; then run the deferred checks on the chosen member (`E3100`, `E3023`, `E4001`, `L3015` with the typed entry line); otherwise `E5035` with each member's first line and reason, and the cast or `rename` line. For a value use, match the expected callable type exactly.
4. Overrides (`[FFI-39]`): compare the `override fn` first line exactly (word and type per parameter, result) with each virtual member of that name; `E5056` with the lines to write when none matches; the trampoline overrides only the matched C++ declaration.
5. Hot reload (Annex B): nothing special; each member is a reloadable-callable thunk slot under its own mangled name (`[HR-7]`), so a header that adds an overload adds a slot and changes no existing call.
6. Tests: the accepted program through the C and LLVM backends with a run that prints which overload ran (compare with the C++ twin on MSVC, clang and gcc); the two refusals; an overlay without types for an overloaded name (`E5010`, message checked); a header update that adds `set(const char*, float)` beside an entry written untyped (`E5010` naming both members); `put(int)`/`put(long)` with a `c_long` argument on Windows (`E5035` with the `rename` line) and on Linux (compiles); `resize(1920)` printing the C++ default; `ember.overlays` with two keyed lines for `draw`; `ember bind --report` and `--init` output.

**departs_from_verdict**: None. Verdict 131 (member lines under their class) is kept and extended to `rename`/`hide` of members; verdict 132 (`@ffi(const)` on an entry, an entry without it makes the method one that changes its object) is kept, and `[FFI-40b]` makes the const member of a pair nameable by its own entry; verdict 125 and 134 (`L3015` per imported function, once, naming every call) apply per member with the typed entry line; verdict 121 (`ember.overlays`, nothing in code files) is kept, the record gaining one line per member, written by Ember alone; verdicts 130, 133 and 136 (`unsafe fn` spelling) are untouched; verdict 117 and `[TYP-26]`'s pack text are untouched.

**trade_offs**: Against the earlier teams' option A (C++-like ranking by argument, conversion tiers, receiver-based const choice): this ruling keeps only `[TYP-24]`'s exact-before-coercion tier, so some calls C++ accepts are refused with a cast (`u8` into `show(int)`/`show(long long)`; an integer literal into `scale(float)`/`scale(double)`), the price of never running a member C++ would not and of a rule a reader can hold in one sentence; speed and safety are untouched (nothing runs, nothing is guessed). The `_mut` suffix is a name the header never wrote, but it is the convention Ember's own bridge already uses (`get_mut`, `span_mut`) and it removes the one selection that depended on context (the circular `this.cfg.at(i)` hole) and lets an overlay vouch for either member; the cost is a collision refused when a class declares `at_mut` itself. Typed overlay entries for overloaded names add a few words per such entry, in exchange for an entry whose identity never depends on which other overloads exist (a later overload with the same parameter names cannot silently widen an old promise) and a grammar whose reading is fixed by a property of the header rather than by whether a word spells a type. Default arguments cost one thunk per arity (code size only) and give the C++ default evaluated by C++, which no Ember expression could reproduce. Two members whose first lines coincide on one target (`c_int`/`c_long` on Windows) are refused there and accepted on Linux; the `rename` line is the portable fix, and the real cure (nominal `c_long`) is outside K21. Readability: the ordinary program is the C++ program's own lines; the extra words appear only in overlays of overloaded names and on the rare cast. Consistency: `E2070` stays `[TYP-24]`'s; one new code, two new rules, every other change a pointer or a row.

**remaining_doubt**: (1) `c_int`, `c_long` and friends are aliases (`std.ffi`, "`c_int` and the other C aliases"), so a program cannot name the `long` overload by a value's type on Windows; the overlay `rename` works on every target, but making the C aliases nominal (the pre-existing E1-12 question) would remove the refusal altogether and is for another ruling. (2) Whether parameter names may be given at a call of an imported function (C++ names are not part of the signature and may differ between declaration and definition); the ruling selects by number, word and type only and leaves names to `[FFI-12]`'s matching of overlays. (3) How an overlay spells a namespaced type in a typed parameter (`Mesh` as the header's own scope sees it, or `gfx.Mesh`); the example writes the header's scope, as `class Renderer:` already does, and the lead's reading of `[FFI-41]` decides. (4) `operator[]`'s const/non-const pair goes through `Index`/`IndexMut` (`[FFI-42]`) and is K47's; the ruling leaves it untouched. (5) `E5035`'s number is free in the registry and the retired table as read; the registry tool decides. (6) `[FFI-55]`'s fingerprint resolves typedefs to the target's types, so `int64_t` members fingerprint differently per platform; pre-existing, not changed here.

## Design 3

**question**: K21: How do C++ overloaded functions and constructors import into Ember? (One name with Ember-side selection by the arguments, distinct names, or mangled suffixes; how ambiguity is refused; how an overlay entry names one overload; what `ember bind` writes; the codes; default arguments; the const/non-const pair; templates.) Angle: the newcomer first, then safety and cost. Checked against CURRENT-SPEC.md (0.9.20 after Hardened_2): every rule the earlier teams quoted still reads as they quoted it, except that [FFI-40]/[FFI-40a] (verdict 132), [FFI-55] (verdict 121, `ember.overlays`), [FFI-39] ("by name and signature") and [BLD-16] are now in force and shape the answer.

**summary**: The rule a Python programmer holds in one sentence: a C++ name the header declares several times is one name in Ember, and a call runs the declaration its arguments fit best, exact types before conversions, and when none or two fit alike the compiler names them and the cast to write (`E5035`). The current text already decides the edges of this (overrides "by name and signature", [FFI-39]; `super.init` by arity and types, [FFI-39b]; `operator[]`'s const pair as `Index`/`IndexMut`, [FFI-42]; one function per USR, [FFI-30]; the record keyed per declaration, [FFI-55]) but has no rule for the middle, so a new `[FFI-45]` states it, [TYP-26] says why a header's overloads are not the program's, and the overlay gains the one thing it lacks: a parameter may write the type the import gives it (`value: c_int`), required only where names cannot tell two declarations apart, so an entry always names exactly one function. A `const`/non-`const` pair with the same parameters is one method with two forms (as `[]` already is), which removes the circularity the earlier designs hit on `this`; nothing runs while the program runs.

- EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types; a name imported from a C++ header stands for every declaration the header gives it (`[FFI-45]`), since the header, not the program, declared them, and a call of it picks one when compiling.
  - why: The cause: [TYP-26] read literally makes every C++ header with an overload set or two constructors `E1030`, over code the program did not write. The exception names its reason (principle 6) and points to the one rule that says how a call picks. §I.6's non-goal stands: no Ember declaration overloads.

- EDIT rule=FFI-17
  - old: generates
  one thunk per imported function, method, constructor and destructor.
  - new: generates
  one thunk per imported function, method, constructor and destructor, each declaration of an overloaded name its own (`[FFI-45]`), and for a function with default arguments one per number of arguments it accepts, each passing only the arguments given, so that C++ supplies the defaults.
  - why: [FFI-17]'s "one thunk per function" left two constructors with one `new` thunk and said nothing of default arguments. One thunk per declaration, and one per accepted arity for defaults, keeps every default's value in C++ (its expression is never translated) and costs nothing at run time beyond the call itself.

- EDIT rule=FFI-17b
  - old: an Ember generic cannot instantiate a C++ template (`E5055`).
  - new: an Ember generic cannot instantiate a C++ template (`E5055`); the listed instantiations of one function template are declarations of one name (`[FFI-45]`).
  - why: `instantiate=["max<int>", "max<double>"]` makes two functions of one name; the rule says they follow the same selection and need no invented names.

- EDIT rule=Annex C §C.1, new rule after [FFI-32] (before [FFI-11f])
  - old: 
  - new: * `[FFI-45]` *(new in 0.9.20)* **A name the header declares several times.** A C++ name declared more than once with different parameters — a namespace function, a method, a static function, a class's constructors (its `new`), the listed instantiations of one function template (`[FFI-17b]`) — imports as one Ember name standing for every declaration: the header, not the program, declared them (`[TYP-26]`). Each declaration is one function (`[FFI-30]`, by USR: a member declared in its class and defined below it is one), with a thunk of its own (`[FFI-17]`) named by its path and the 32 hex digits of the BLAKE3 of its canonical C++ signature, as a generic instance's symbol is (`[MNG-1]`); `ember bind --report` lists such a name with each declaration as the header writes it. A call of the name runs the one declaration it fits best, decided when compiling. A call fits a declaration when it gives every parameter without a default argument and no other, each argument by position or by name (`[TYP-25]`; a parameter with a default may be omitted, and C++ supplies the value), with the word its parameter has (`owns` for one imported from `T&&`, `ref mut` for `T&`, none for `const T&` or a value, `[FFI-40]`), of the parameter's imported type or of one that reaches it by a coercion of `[TYP-5]`. An untyped literal counts as the type C++ gives its spelling, `c_int` for a whole number and `f64` for a decimal, where a declaration takes that type at its position, and else as `[LEX-16]` and `[LEX-17]` say. Of two declarations a call fits, one beats the other when, argument by argument, each of its parameters takes its argument at least as directly — the argument's own type before a coercion, and among coercions a nearer type (the narrowest number type the argument widens to, the nearest base class) before a farther — and one takes it more directly; the call runs the declaration that beats every other it fits, so where C++ picks one Ember picks the same, and Ember's fewer conversions never pick another. A call that fits none, or two that neither beats, is `E5035`, naming each declaration as the header writes it and the way out: "`w.value(7)` (line 8) fits `void value(long long)` and `void value(double)` of `ui::Widget` alike: `7` is a whole number both take as it is; write the type, `w.value(7 as i64)` or `w.value(7.0)`"; where two declarations are one Ember signature on this target (`int` and `long` on Windows, `[FFI-8]`), the way out is a `rename` of one by its parameters in the overlay (`[GRM-35]`). What the code around the call permits is checked after, on the declaration chosen — `unsafe:` (`[FFI-2]`, `E3100` naming that declaration's entry line), its effects, a `ref mut` or `owns` argument's name, the loop and view rules — so a line runs one declaration wherever it stands. A `const` and a non-`const` declaration of one member with the same parameters are not two declarations but one method with two forms, as `operator[]`'s pair is `Index` and `IndexMut` (`[FFI-42]`): through a name that only reads the object the `const` form runs; through any other name, `this` included, the `const` form runs where the call's result is a view or pointer that is only read, and the non-`const` form where the result is changed, given as `ref mut` or kept as a `ref mut` view, or where the result is no view; only the non-`const` form may change the object, and the `const` form only reads it where `[FFI-40a]` finds so or the member's one entry says `@ffi(const)`, else the call through a read-only name is `E3023`, naming that entry line. `super.init(…)` selects as a call does (`[FFI-39b]`); an `override fn` matches a virtual method exactly (`[FFI-39]`), never through this ranking. A use of the name that is not a call (a function value, `[CLO-3]`) stands for the declaration the expected callable type fits exactly, and with none expected is `E5035`. Nothing runs while the program runs for this rule: each call is the direct call of the chosen declaration's thunk, as for every imported C++ function.
  - why: The missing rule. One name per C++ name is what the header means and what a Python programmer expects (pybind11 does the same); selection is Ember's own coercion list ranked per argument, which is C++'s ranking restricted to Ember's conversions, so it never runs a function C++ would not; refusal names the candidates. The pair-as-one-method rule follows [FFI-42]'s precedent and avoids the circularity of choosing by `this`'s changeability (a method's effect on `this` is derived from its body, verdict 86). The thunk name and the report are what `ember bind` writes.

- EDIT rule=FFI-39
  - old: An `override fn` matching no virtual
  method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual
  method of the base, by name and signature — each parameter's type the virtual method's imported type itself, never one it reaches by a coercion, since C++'s argument flows into the override's parameter — is `E5056`; a base's virtual methods of one name are matched one by one, and those no `override fn` matches stay the base's.
  - why: The current words already say "by name and signature"; the clause makes exact matching explicit so an implementer does not apply [FFI-45]'s call ranking backwards (an `override fn on(e: KeyEvent)` must never stand in for `on(const Event&)`: that is an unchecked downcast and a read past the object).

- EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`)
  - new: `super.init(…)` selects the C++ base constructor as a call of an imported name selects its declaration (`[FFI-45]`) and runs it at `super.init(…)` (`[CLS-11]`)
  - why: One selection rule, not two: `super.init(16, 8)` under `Widget(int, int)` / `Widget(float, float)` picks `(int, int)` by the same literal rule as `Renderer.new(1280, 720)`.

- EDIT rule=FFI-40a
  - old: An entry whose name or
  parameters differ from the import's member is `E5010` (`[FFI-12]`).
  - new: A `const` and a non-`const` member of one name and parameters, one method with two forms (`[FFI-45]`), has one entry, whose `@ffi(const)` describes its `const` form. An entry whose name or
  parameters differ from the import's member, or fit two members, is `E5010` (`[FFI-12]`).
  - why: Verdict 132 kept as written (an entry without `@ffi(const)` makes the method changing; with it, const): for a pair the word describes the form it can describe, and the writer writes the member once, as the header's reader thinks of it.

- EDIT rule=FFI-12
  - old: An overlay entry whose name, parameter count or parameter names
  do not match the header is `E5010`, so overlays cannot drift.
  - new: An overlay entry whose name, parameter count, parameter names, or the words and types it writes
  do not match one declaration of the header, or that fits two declarations of one name (`[FFI-45]`), is `E5010`, so overlays cannot drift and no entry vouches for a function its writer did not name; for two, the message gives the entry lines with the types that tell them apart: "`fn set(name, value)` fits `void set(const char*, float)` and `void set(const char*, int)` of `gl::Shader`; write one entry per declaration: `fn set(name: borrowed nul_terminated, value: f32)` and `fn set(name: borrowed nul_terminated, value: c_int)`".
  - why: An entry asserts that a call within its contract cannot break memory safety ([FFI-2]); it must therefore name exactly one function. Matching by names alone cannot tell `set(const char*, float)` from `set(const char*, int)` apart, which is why the earlier teams could not write their overlay.

- EDIT rule=FFI-30c
  - old: a later file's `override` entry replaces an earlier file's for that name, and any other
  second entry for one name is `E5010` (ODR-149).
  - new: a later file's `override` entry replaces an earlier file's for that declaration, and any other
  second entry for one declaration is `E5010` (ODR-149; a C++ name of several declarations has one entry per declaration, `[FFI-45]`).
  - why: "one name" would make the second `draw` entry an error; the unit an overlay describes is the declaration.

- EDIT rule=GRM-35 (prose)
  - old: replacing an earlier overlay's entry for that name
  - new: replacing an earlier overlay's entry for that declaration
  - why: Same unit as [FFI-30c].

- EDIT rule=GRM-35 (grammar)
  - old:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  - new:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | overlay_rename | overlay_hide | extend_decl | overlay_class)
  overlay_rename:= "rename" IDENT [overlay_params] "as" IDENT NEWLINE
  overlay_hide  := "hide" IDENT [overlay_params] NEWLINE
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} (overlay_fn | overlay_rename
                   | overlay_hide)} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT overlay_params ["->" contract {contract}] NEWLINE
  overlay_params:= "(" [overlay_param {"," overlay_param} [","]] ")"
  overlay_param := [param_word] IDENT [":" [type] {contract}]
  - why: The smallest grammar that lets an entry, a `rename` or a `hide` name one declaration of an overloaded name: the parameter may carry the word and type Ember's own first line gives it ([FFI-11] already writes `ref mut p: T` in front of a name), and the brackets `rename` and `hide` may take are the entry's own. `rename`/`hide` in a class section follows verdict 131 (a member's line goes under its class). Note: the old `overlay_param := IDENT [":" contract {contract}]` line is replaced by the last line above.

- EDIT rule=GRM-35 (prose, after the `@ffi(macro_fn)` sentence)
  - old: since a macro has no C declaration to read (ODR-147).
  - new: since a macro has no C declaration to read (ODR-147). In any entry a parameter may carry the word and the type the import gives it (`owns tex: Texture`, `value: c_int`, `s: *c_char`), matched as the import spells it (`[FFI-8]`'s table, so `c_long` names C++'s `long` on every target) and checked against the header; it must where the entry's name and parameter names fit two declarations of one name (`[FFI-45]`, `[FFI-12]`). After the colon a word that names a type the import or the program knows and is not a contract word of `[FFI-11]` is the type; a type spelled like a contract word is written as a raw identifier (`r#status`, `[LEX-14]`). `rename` and `hide` with a parameter list name one declaration of such a name (`rename put(v: c_long) as put_long`), inside a class section one of its members; without one they name every declaration of the name.
  - why: Says how the new grammar is read: one entry form, the type optional and checked, the collision with contract words settled by the raw-identifier mechanism the language already has, and the matching by spelling so that a Windows build, where `c_int` and `c_long` are one type, can still name `long`.

- EDIT rule=FFI-55
  - old: one `blake3:` fingerprint of
  each declaration the overlay describes, as the importer binds it (`[FFI-14]`):
  - new: one `blake3:` fingerprint of
  each declaration the overlay describes, as the importer binds it (`[FFI-14]`), keyed by the declaration's name and, for a C++ name of several declarations (`[FFI-45]`), by its signature as `ember bind --report` prints it (`"Renderer::draw(const gfx::Mesh&)"`), one line per declaration:
  - why: The record's example keys a line by name; two overloads would share a key. The fingerprint already holds the canonical type, so a header that changes one overload's parameter type makes that overlay entry `E5010`, as verdict 121 asks.

- EDIT rule=registry row E5010
  - old: an entry outside a class section that names only members of classes | `[FFI-12]`, `[FFI-55]` |
  - new: an entry outside a class section that names only members of classes; an entry that fits two declarations of one C++ name, or none | `[FFI-12]`, `[FFI-55]` |
  - why: The row names every case its rules refuse.

- EDIT rule=registry, new row after E5034
  - old: | `E5034` | unsupported C++ construct | `[FFI-48]` |
  - new: | `E5034` | unsupported C++ construct | `[FFI-48]` |
| `E5035` | a call of a C++ name of several declarations that none fits or that two fit alike; a function value of such a name with no expected type | `[FFI-45]` |
  - why: One code, one defining rule: `E2070` belongs to [TYP-24] (interface instances); the FFI range has `E5035` free (codes are never reused; `E5034` is the last used before `E5040`).

**refused_program**: // widget.hpp (C++)
// namespace ui {
// class Widget {
// public:
//     Widget() noexcept;
//     void value(long long v) noexcept;
//     void value(double v) noexcept;
// };
// }
import cpp "widget.hpp" with (project="ui") as ui

fn main():
    # SAFETY: `value` only stores the number in the widget
    unsafe:
        w = ui.ui.Widget.new()
        w.value(7)

Refused: E5035 at line 8: "`w.value(7)` (line 8) fits `void value(long long)` and `void value(double)` of `ui::Widget` alike: `7` is a whole number both take as it is; write the type, `w.value(7 as i64)` or `w.value(7.0)`". (C++ refuses the same call as ambiguous. `7` is not C++'s `int` here because no `value` takes `c_int`, so it takes each declaration's type as its own, [LEX-16]. Either written form compiles and runs the named declaration.)

A second refusal, in the overlay: `unsafe overlay cpp "shader.hpp":` / `    class Shader:` / `        fn set(name: borrowed nul_terminated, value)` for a header with `set(const char* name, float value)` and `set(const char* name, int value)` is E5010: "`fn set(name, value)` fits `void set(const char*, float)` and `void set(const char*, int)` of `gl::Shader`; write one entry per declaration: `fn set(name: borrowed nul_terminated, value: f32)` and `fn set(name: borrowed nul_terminated, value: c_int)`".

**accepted_program**: // renderer.hpp (C++)
// namespace gfx {
// struct Mesh { int vertices; };            // standard-layout: imported by value ([FFI-32])
// struct Sprite { int frame; };
// struct Light { float intensity; };
// class Renderer {
// public:
//     Renderer() noexcept;
//     Renderer(int width, int height) noexcept;
//     void draw(const Mesh& mesh) noexcept;
//     void draw(const Sprite& sprite) noexcept;
//     void set(const char* name, int value) noexcept;
//     void set(const char* name, float value) noexcept;
//     const Light& light(int i) const noexcept;    // defined in renderer.cpp
//     Light& light(int i) noexcept;                // defined in renderer.cpp
// };
// }
# overlays/renderer.em
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        fn new()
        fn new(width, height)
        fn draw(mesh)
        fn draw(sprite)
        fn set(name: borrowed nul_terminated, value: c_int)
        fn set(name: borrowed nul_terminated, value: f32)
        @ffi(const)
        fn light(i) -> borrowed one from(this)

# src/main.em
import cpp "renderer.hpp" with (project="engine", overlay="overlays/renderer.em") as eng

fn brightness(r: eng.gfx.Renderer) -> f32:
    return r.light(0).intensity           # `r` only reads: the const form, which the entry says only reads

fn main():
    mesh = eng.gfx.Mesh(vertices=36)
    hud = eng.gfx.Sprite(frame=2)
    r = eng.gfx.Renderer.new(1280, 720)   # Renderer(int, int): both literals are C++'s `int`
    r.draw(mesh)                           # draw(const Mesh&): a Mesh fits only it
    r.draw(hud)                            # draw(const Sprite&)
    r.set(c"light_count", 4)               # set(const char*, int): `4` counts as `c_int`
    r.set(c"exposure", 1.5)                # set(const char*, float): no `set` takes `f64`, so `1.5` is the `f32` its context expects
    r.light(0).intensity = 0.5             # the result is changed: the non-const form, through a name that may change `r`
    println(brightness(r))

Compiles with no `E1030`, prints `0.5`. Every call is a direct call of one declaration's thunk, chosen when compiling; the overlay names each declaration once (`draw` by parameter names, `set` by the one type the names cannot tell, `light` once for its two forms); the `ember.overlays` record has one line per declaration (`"Renderer::set(const char*, int)" = "blake3:…"`).

**runtime_cost**: nothing. Selection is decided when compiling; each call is the same single call of the chosen declaration's `extern "C"` thunk that every imported C++ call already is ([FFI-3], inlined under the project's LTO by [BLD-FFI-4]), so the existing row "FFI call | guaranteed present | one direct call" covers it and no new §X.4 row is needed. A default argument costs only what the C++ caller pays (C++ evaluates it inside the thunk). Compile time adds one fit test per candidate declaration at each call of an overloaded name.

**implementer_steps**: 1. Importer (libclang): group the header's function-like declarations by qualified name and kind (every constructor of a class under its `new`; redeclarations merged by USR, [FFI-30]); for each declaration record its canonical C++ signature string, parameter names, default-argument flags, each parameter's imported Ember type *as spelled by [FFI-8]'s table* (`c_int`, `c_long`, `*c_char`, `Texture`) and its word (`owns` for `T&&`, `ref mut` for `T&`, [FFI-40]), and its thunk symbol: the Ember path plus the BLAKE3 of the canonical signature ([MNG-1]'s form). Merge a `const` and a non-`const` member with equal parameter lists into one method with two forms, each form keeping its own thunk and its [FFI-40a] reading. Write all of it into the `.embind` ([FFI-14]); `ember bind --report` prints each overloaded name with its declarations as the header writes them; `ember bind --emit-cpp` emits one thunk per declaration and, for a function with k default arguments, k+1 thunks that forward only the arguments given.
2. Typechecker, a call of an imported name: collect the candidates; drop those the call does not fit (arity with defaults, named arguments, words, and each argument's type equal to the parameter's or reaching it by a [TYP-5] coercion); type each untyped literal first as `c_int`/`f64` if some candidate takes that type at its position, else by [LEX-16]/[LEX-17] against each candidate; rank each argument 0 (own type) or 1 + distance (widening steps, base-class distance) and compare candidates per argument; take the one that dominates every other; otherwise emit `E5035` with the candidates printed from the `.embind` and the cast lines. Then run every existing check on the chosen declaration: `unsafe` ([FFI-2], `E3100` naming its entry line), `E3023`, effects, [FFI-39d], `L3015` once per declaration (verdicts 125, 134).
3. The pair's form: after selection, pick the `const` form through a read-only receiver, and through any other receiver by the result's use, with the same analysis that chooses `Index` over `IndexMut` for `a[i]` ([TYP-21], [STD-17]); the `const` form "only reads" iff [FFI-40a]'s body reading or the entry's `@ffi(const)` says so.
4. Overlay checker ([FFI-12], [CLI-7]): parse `[param_word] IDENT [":" [type] {contract}]`, reading the word after the colon as a type when it names a known type and is not a contract word (`r#` escapes a collision); match each entry, `rename` and `hide` against the declarations by name, count, parameter names, words and spelled types; none or more than one is `E5010` with the typed lines to write; a second entry for one declaration is `E5010` ([FFI-30c]), `override` replaces per declaration; `ember.overlays` keys per declaration by signature ([FFI-55]).
5. `super.init(…)` calls the same resolution as step 2 on the base's constructors ([FFI-39b]); `override fn` matching stays an exact signature match per virtual ([FFI-39], `E5056`), never step 2's ranking.
6. Function values: an overloaded name used as a value takes the declaration the expected callable type equals; none expected is `E5035`.
7. Tests ([CXX-1] corpus): the two programs above; `out.put(5)` under `put(int)`/`put(long)` runs `put(int)` on Linux and Windows, an `i32` variable there is `E5035` on Windows with the `rename` line and compiles on Linux; `c.show(lives)` with `lives: u8` under `show(int)`/`show(long long)`/`show(double)` runs `show(int)`; `cache.add(atlas)` / `cache.add(owns splash)` run the `const T&` / `T&&` declarations; a redeclared member gives no `E5035` and no `E5010`; a header that changes one overload's parameter type makes only that entry `E5010`; run under MSVC, Clang and GCC on both targets.

**departs_from_verdict**: none. Verdicts touched and kept: 121 (`ember.overlays` keyed by Ember, now one line per declaration; nothing in the overlay or the program); 125 and 134 (`@safety`/`L3015` per declaration, once each, members included); 130/133 (`unsafe fn` entries unchanged); 131 (member lines, now `rename`/`hide` too, stay under their class); 132 (an entry without `@ffi(const)` makes the method changing, with it const; for a pair the one entry's `@ffi(const)` describes the `const` form, which is the only form the word can describe); 86 (no `reads` word returns: the pair's form is read from the receiver and the result's use); 117 (`[TYP-26]`'s pack sentence untouched); N74/F24 of the 0.9.20 audit (`ref_to_ptr` is one built-in, not an overload: consistent, Ember declares no overload).

**trade_offs**: Readability: the ordinary program is the C++ program's own lines (`Renderer.new(1280, 720)`, `r.draw(mesh)`), one name per thing, no invented `draw_Mesh` or `new_int_int`, no overlay needed to call anything; the one new thing to learn is that an overlay parameter may carry a type, needed only where two overloads share parameter names. Safety: unchanged in kind; every declaration keeps its own entry, `@safety`, `L3015` and `unsafe` rule, an entry never covers a function its writer did not name, and a line runs one declaration wherever it stands (the surrounding `unsafe:` or effects never change the choice). Speed: nothing at run time; a fit test per candidate at compile time. Consistency: the selection is [TYP-24]'s own ranking (own type before coercion) made per argument, the pair follows [FFI-42], overrides and `super.init` keep their rules. Given up: (1) Ember's conversions are fewer than C++'s, so Ember never runs a different overload than C++ would but can accept a call C++ calls ambiguous (`set(name, 1.5)` runs the `float` overload, which C++ refuses) and can refuse one C++ accepts (an `int` variable of unknown range to `c_int`: no narrowing, §I.6). (2) On Windows `int` and `long` are one Ember type, so an `i32` variable to `put(int)`/`put(long)` is `E5035` there and not on Linux; literals still pick `int`, and the `rename` by spelled type is the portable way out — the honest price of [FFI-8]'s aliases, which this ruling does not change. (3) For a `const`/non-`const` pair Ember may run the `const` form where C++ on a non-const object would run the non-`const` one (a read of a copy-on-write accessor does not detach), exactly as [FFI-42] already does for `operator[]`; values are the same, only which C++ body runs differs. (4) A function with k default arguments gets k+1 thunks, a little more generated C++. Option B (type-built names) was rejected because it names things the program never wrote and breaks when the header renames a type; option C (overlay renames only) because nothing works until an overlay exists and today's `rename` cannot name an overload; option D (choose among what compiles where the line stands) because moving a line into `unsafe:` would silently change which function runs.

**remaining_doubt**: (a) `E5035` needs a line in §XVII.6.2's shape table (N11 lists `E2070`); the lead adds it to N11 or a new shape under [DIA-12]. (b) Default arguments: one thunk per accepted arity (k+1) was chosen over one per arity the package calls, to keep the thunk translation unit independent of the calls; the lead may prefer the other. (c) The C++ parameter-name rule of [FFI-12] already fails for a header that leaves parameters unnamed (`void draw(const Mesh&);`), for C as for C++; the typed entry now covers it but the text does not say what an unnamed parameter's entry name is — a pre-existing gap worth a sentence. (d) Constructors of a by-value standard-layout class ([FFI-32] imports it as a `@layout(c) struct`) are not addressed by the current text or by this ruling; K21's question was about opaque classes' `new`. (e) `rename`/`hide` inside a class section is new for every member, not only overloaded ones; harmless, but the lead may wish to restrict it. (f) The rule index table at the end of the document and §H.54 are generated by the hardening recipe; they are not listed as edits. (g) K47 (`operator[]`'s overlay name) is answered in part: the pair is one `Index`/`IndexMut` operation and one entry describes it; its spelling in an overlay is still K47's.

## Judgement 1

**winner**: 2

**scores**: [
 {
  "design": 1,
  "safety": 8,
  "speed": 9,
  "readability": 6,
  "consistency": 6,
  "generality": 8,
  "holes": "(1) The `this` rule breaks the design's own promise. [FFI-17g] says a use is \"chosen when compiling from the use's own words and nothing around it\", then \"`this` is a name that may change the object where any other line of its method changes it, else one that only reads\". Program: `class Server:` / `    cfg: net.srv.Config` / `    fn set_port(i: usize, p: c_int):` / `        this.cfg.at(i).port = p` with the header's `const Entry& at(size_t) const` and `Entry& at(size_t)`. No *other* line changes `this`, so `this` only reads, step 2 fits the `const` `at` alone, its result is a `ref` view and the assignment is `E3023`; the ordinary setter is refused. Adding `this.n += 1` anywhere in the method makes the same line run `Entry& at`: which C++ function a line calls depends on the lines around it (the hole D1 itself names as the reason to reject option D). (2) No `[MNG-1]` edit: \"A method's path is its type's path and its name\" gives `draw(const Mesh&)` and `draw(const Sprite&)` one path, so `[HR-7]` (\"Call-table slots are identified by mangled name\") gives both one slot and the two `extern \"C\"` thunks one symbol; the USR hash lives only in the implementer steps, not in the text. (3) `[FFI-55]` keyed \"by the words of its entry that name it\": an entry edited after the header changed (a type added to tell two functions apart) has a new key, the record \"lacks\" it, and \"a declaration the overlay describes that the record lacks, an entry just written, is added by the package's own build\" \u2014 the changed declaration is recorded fresh with no person's check, which is what verdict 121's record exists to prevent. (4) Step 2 counts a literal as Ember's `int` (i64), so `out.put(5)` for `put(int)`/`put(long)` ties on every target where C++ runs `put(int)`; the commonest literal call needs `5 as c_int`. (5) The receiver's kind sits inside the selection while \"what a name allows ... is checked afterwards\": a pair imported as changing (bodies out of line, no entry) called through a read-only name fits at no step and is `E5035`, where the cause is `E3023`'s."
 },
 {
  "design": 2,
  "safety": 9,
  "speed": 10,
  "readability": 8,
  "consistency": 8,
  "generality": 7,
  "holes": "(1) The registry row \"fits two with none exact\" and the rule's \"fits two with none exact\" give no code for two *exact* fits: `n: i32` into `put(c_int)`/`put(c_long)` on Windows, where `[FFI-8]`'s `c_int` and `c_long` are both `i32`; the prose says \"fit every call together\" and \"the `rename` line tells them apart\" but names no refusal. (2) Whole-call exactness refuses what C++ picks argument by argument: `for slot in 0..4: scene.set(slot, 4)` with `set(int slot, int value)`/`set(int slot, float value)`: `slot` reaches `c_int` by `[TYP-5]` rule 12 on both, `4` is exact on one, so neither member is exact and the call is `E5035`, a cast on every indexed literal call. (3) The reading of a typed entry is fixed by a header property: \"In an entry ... of a name the header declares several times, the first word after a parameter's colon is its type ... in every other entry the words after the colon are contract words\". A header update adding `get(const char*)` beside `get(int)` makes the unchanged entry `fn get(p: borrowed one)` read `borrowed` as a type and fail `E5010` with a message about types; and every entry of an overloaded name must carry types even where the count already tells (`fn new(width: c_int, height: c_int)`). (4) The `_mut` collision row says \"`rename` one in the overlay\" but gives no order: at rename time the header's `at_mut` and the generated one both carry the name, so `rename at_mut(i: usize) as \u2026` fits two and is `E5010`; it needs \"renames apply before the pair is named\". (5) \"the ruling selects by number, word and type only and leaves names\": a named argument (`r.resize(width=3)`, `[TYP-25]`) has no stated fit."
 },
 {
  "design": 3,
  "safety": 7,
  "speed": 9,
  "readability": 6,
  "consistency": 6,
  "generality": 8,
  "holes": "(1) One entry for a `const`/non-`const` pair: \"has one entry, whose `@ffi(const)` describes its `const` form\". The non-`const` form's `Entry&` result is a `ref mut` view, and `[FFI-11]` says \"`ref mut` and `MutSpan[T]` results allowed only with `exclusive`\" while \"a result is `aliased` unless stated\": one entry cannot say `exclusive` for one form and not the other, and one `@safety` (verdict 125, \"members included\") vouches for two bodies \u2014 verdict 131's problem, a line standing for two functions. (2) The form is chosen by the lines after the call: \"the `const` form runs where the call's result is a view or pointer that is only read, and the non-`const` form where the result is changed\", against the same rule's \"so a line runs one declaration wherever it stands\"; a copy-on-write accessor detaches in one body and not the other, and the method3 record's decision 4 rejected exactly this (\"c would choose by the code around the call, which the first decision excludes\"). (3) \"a type spelled like a contract word is written as a raw identifier (`r#status`, `[LEX-14]`)\": `[LEX-14]` is for keywords, contract words are not keywords, and `docs/AUTOPILOT.md` \u00a74 says \"never require `r#`\". (4) The thunk name (\"its path and the 32 hex digits of the BLAKE3 of its canonical C++ signature\") is stated inside [FFI-45] only; `[MNG-1]` still reads \"A method's path is its type's path and its name\", so the two rules disagree on a symbol. (5) The distance tier (\"the narrowest number type the argument widens to, the nearest base class\") is a third rule a newcomer must hold, and \"nearest base class\" needs the hierarchy modelled, which `[FFI-48]` says the importer may lack (\"casts across a hierarchy the importer did not model\"). (6) No `[COST-3]` row for the default argument's expression, though the design says \"C++ evaluates it inside the thunk\" \u2014 something that runs with no row (`[COST-5]`)."
 }
]

**grafts**: Design 2 wins: the `_mut` split follows Annex C's own precedent ([FFI-42]: \"such a class imports `get()`, and `get_mut()` where the operator is not `const`\"; [FFI-17a]: `.span()`, `.span_mut()`), makes which C++ body runs a word on the line, and removes the one contextual choice (D1's `this` fixed point, D3's choice by the result's use) that let a line's meaning depend on its neighbours; it alone edits [MNG-1], [COST-3], [FFI-44] and §I.6. Grafted from design 3: the per-argument comparison (a member is chosen when, argument by argument, each parameter takes its argument at least as directly and some more directly), kept to two levels (own type before coercion) and without D3's distance tier, which repairs D2's hole (2) and is still C++'s own rule restricted to Ember's coercions. Grafted from design 1: the typed parameter is optional, needed only where names do not tell two functions apart, and the reading of the word after the colon is fixed by [FFI-11]'s closed list (a contract word is one of that list, any other word is the type; a type named like a contract word is first renamed) instead of D2's header-property reading, which repairs D2's hole (3); `= delete` members take part and are refused when chosen; the listed instances of one function template are members of one name; `rename`/`hide` take a `member_ref` with an optional `-> type` so two members alike in parameters (template instances differing by result) can be named, and the `[FFI-48]` return-type row goes (a C++ header cannot declare such a pair otherwise, and the construct is now refused at the use with the overlay line as the way out, not at import). Kept from design 2 and sharpened: the two-exact (alias) case named as `E5035`, the `_mut` collision row with \"renames apply before the pair is named\" (D2 hole 4), the record key as the header spells the declaration, named arguments fit as in any call ([TYP-25]). Added by the judge: the N11 shape row gains `E5035` ([DIA-12] asks a shape for every name error).

- FINAL EDIT rule=§I.6 Non-goals
  - old: implicit numeric narrowing; function overloading
  - new: implicit numeric narrowing; function overloading (a C++ header's several functions of one name reach Ember as one name of several members, each use standing for one, `[FFI-57]`)
  - why: The non-goal stays true of Ember's own declarations; the cross-reference stops a reader finding Annex C contradicting Part I.

- FINAL EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. A C++
  header's several functions of one name reach Ember as one name of several members, each use
  standing for one (`[FFI-57]`), and not as `E1030`: the program declares none of them, and a
  function Ember declares beside an imported name is `E1030` as any other.
  - why: Read literally the rule refuses every C++ class with two constructors; the exception states its reason (the functions are the header's) and where it stops.

- FINAL EDIT rule=FFI-17
  - old: one thunk per imported function, method, constructor and destructor.
  - new: one thunk per imported function, method, constructor and destructor, each member of a name declared
  several times its own (`[FFI-57]`), and one per number of arguments a function with default
  arguments accepts, passing only the arguments given so that C++ supplies the defaults.
  - why: "one thunk per function" left two constructors with one `new` thunk and said nothing of default arguments; one thunk per member and per arity keeps every default's expression in C++ and costs only the call.

- FINAL EDIT rule=FFI-17b
  - old: an Ember generic cannot instantiate a C++ template (`E5055`).
  - new: an Ember generic cannot instantiate a C++ template (`E5055`); the listed instantiations of one
  function template are members of one name (`[FFI-57]`).
  - why: `instantiate=["max<int>", "max<double>"]` makes two functions of one name; they follow the one selection and need no invented names.

- FINAL EDIT rule=FFI-44 table, row after "explicit template instantiations"
  - old:   | explicit template instantiations | automatic (`[FFI-17b]`) |
  - new:   | explicit template instantiations | automatic (`[FFI-17b]`) |
  | a name declared several times (overloads, constructors, default arguments, a `const` and a non-`const` pair) | automatic (`[FFI-57]`, `[FFI-40b]`) |
  - why: The table says what imports and how; the commonest construct of C++ headers was missing from it.

- FINAL EDIT rule=Annex C §C.1, new rule [FFI-57] placed directly before [FFI-11f]
  - old: * `[FFI-11f]` A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact:
  - new: * `[FFI-57]` *(new in 0.9.20)* **A name declared several times.** A C++ name the header declares for
  several functions — a namespace's functions, a class's methods, its static functions, its
  constructors (`new`), the instances of one function template listed in `instantiate=[…]`
  (`[FFI-17b]`), a function with default arguments, which counts once for each number of arguments
  it accepts, a redeclared function counting once (`[FFI-30]`) — imports as one Ember name of
  several **members**, one per function, and is not `E1030` (`[TYP-26]`): the program declares none
  of them. A `const` and a non-`const` method alike in name and parameters are two names first
  (`[FFI-40b]`). Each member has its own thunk (`[FFI-17]`), its own symbol (`[MNG-1]`), its own
  overlay entry (`[GRM-35]`, `[FFI-12]`), its own `@safety` and `L3015` (`[UNS-7]`, verdicts 125
  and 134) and its own line of `ember.overlays` (`[FFI-55]`). Each use of the name stands for one
  member, read when compiling from the use's own words and the import, and from nothing around it.
  A call (of a function, a method, `T.new(…)`, `super.init(…)`, `[FFI-39b]`) *fits* a member when
  that member, declared alone, would take its arguments: their number and names (`[TYP-25]`), the
  word before each (`ref mut`, `owns`, none: `[FN-2a]`, `[FFI-40]`) and their types, each the
  parameter's own or one that reaches it by a coercion of `[TYP-5]`; an untyped literal is the
  parameter's own type where the parameter's C++ type is `int` (a whole number) or `double` (a
  number with a fraction), the types C++ gives those literals, and else takes the type the parameter
  expects as `[LEX-16]` and `[LEX-17]` say. Of the members a call fits, one is chosen when, argument
  by argument, each of its parameters takes its argument at least as directly as the other member's
  (the argument's own type before a coercion) and some parameter more directly; so where C++ chooses
  a function Ember chooses the same one or refuses, and never another, since Ember's coercions are
  among C++'s conversions. A name used without a call stands for the member whose first line is
  exactly the callable type its place expects (`[CLO-3]`). A use that fits no member, or two that
  neither beats (two alike in all Ember sees included: instances of one template differing only in
  the result, `int` and `long` where this target makes them one type), or that chooses a member
  declared `= delete`, is `E5035`, which names each member as the header declares it, why it does
  or does not fit, and the line to write — the argument with its type, or the overlay line that
  names one member (`[GRM-35]`): "`w.value(7)` (line 8) fits `void value(long long)` and `void
  value(double)` of `ui::Widget` alike: `7` reaches each by a coercion; write `w.value(7 as i64)`
  or `w.value(7.0)`"; "`out.put(n)` (line 6) fits `void put(int)` and `void put(long)` of
  `io::Out` alike: on this target both take an `i32`; name the one meant in the overlay, under
  `class Out:`: `hide put(v: c_long)`, or `rename put(v: c_long) as put_long`". What the code
  around the use permits — unsafe code (`[FFI-2]`, `E3100` naming the member's entry line), a name
  that only reads (`E3023`), its effects, the loop and view rules — is checked after, on the member
  chosen, so a line stands for the same member wherever it stands. An `override fn` matches a
  virtual member by its exact first line (`[FFI-39]`), never through this choice. `ember bind
  --report` lists each name of several members with each member as the header declares it. Nothing
  runs for this while the program runs: a use is one direct call of the member's thunk, as any
  imported C++ function's (`[COST-3]`); a call that leaves a default argument out runs that
  default's expression inside the arity's thunk, as a C++ call site that leaves it out would.
* `[FFI-11f]` A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact:
  - why: The missing rule, a property (a name declared several times, and what a use's own words fit) decided from the import and the line alone (`[BLD-16]`), with `[TYP-24]`'s own ranking (own type before coercion) applied argument by argument as C++ applies its own, so Ember never runs a member C++ would not and refuses, naming the line to write, wherever it is unclear.

- FINAL EDIT rule=FFI-48 table, row "overloads differing only in return type"
  - old:   | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |

  - new: 
  - why: A C++ header cannot declare two functions differing only in the result; the case that occurs (instances of one template, `int` and `long` on a target where they are one Ember type) is refused at the use by `[FFI-57]` (`E5035`) with `rename … -> type` or `hide` as the way out, and the header stays imported. The row's "rename one in the overlay" named a `rename` that could not name one member.

- FINAL EDIT rule=FFI-48 table, new row after "allocator-parameterised containers"
  - old:   | allocator-parameterised containers | the allocator is part of the type and opaque | expose a span, or keep it native |
  - new:   | allocator-parameterised containers | the allocator is part of the type and opaque | expose a span, or keep it native |
  | a class declaring the `_mut` name that `[FFI-40b]` gives a pair's non-`const` member | the two would be one name | `rename` the class's own member in the overlay; renames apply before the pair is named |
  - why: The generated name may collide with the header's own; the collision is refused naming the row, and the order (renames first) makes the way out work.

- FINAL EDIT rule=FFI-39
  - old: An `override fn` matching no virtual
  method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual
  method of the base, by name and first line — each parameter's word and type exactly those the
  import gives the virtual method's (`[FFI-40]`), and its result; an upcast or a widening counts
  for nothing, since C++ hands the override its own argument (a `FocusEvent` for an `on(const
  Event&)`) — is `E5056`, whose message gives each virtual method of that name as the `override fn`
  line to write; where the base declares the name several times (`[FFI-57]`), the override replaces
  the one member it matches and the trampoline passes on that one, the others staying the base's.
  - why: An override matched through an upcast would read a smaller base object as a derived one in safe code, and one matched through a widening would truncate C++'s argument; "by name and signature" is made exact and told how an overloaded virtual is replaced.

- FINAL EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`);
  - new: `super.init(…)` runs the C++ base constructor its arguments choose as `[FFI-57]` chooses a
  member of the base's `new` (the argument's own type before a coercion, argument by argument; none
  or a tie `E5035`) and runs it at `super.init(…)` (`[CLS-11]`);
  - why: One selection rule for constructors wherever they are called; "by arity and argument types" said neither how a literal counts nor what a tie is.

- FINAL EDIT rule=§C.3, new rule [FFI-40b] placed directly before [FFI-41]
  - old: * `[FFI-41]` *(changed in 0.9.15)* Static member functions import as functions of the type with no
  - new: * `[FFI-40b]` *(new in 0.9.20)* **A `const` and a non-`const` pair.** A non-`const` method declared
  beside a `const` method of the same name and the same parameters, as the import maps them, imports
  with `_mut` added to its name — `at` and `at_mut`, as `get` and `get_mut` (`[FFI-42]`) and `span`
  and `span_mut` (`[FFI-17a]`) — so the two are two names, and which one a call runs is a word on the
  line (`cfg.at(i).port`, `cfg.at_mut(i).port = 8080`), never read from the name the object is
  reached through nor from the lines after the call; `at_mut` may change its object (`[FFI-40]`),
  and the `const` one is read as `[FFI-40a]` says, an entry with `@ffi(const)` keeping it so
  (verdict 132). Each has its own entry, by its Ember name (`@ffi(const) fn at(i) -> borrowed one
  from(this)`, `fn at_mut(i) -> borrowed one exclusive from(this)`, `[FFI-12]`). An overlay's
  `rename` and `hide` items apply before the pair is named, so a class that itself declares the
  `_mut` name gives the pair's non-`const` member no name until the class's own is renamed, the
  `[FFI-48]` row.
* `[FFI-41]` *(changed in 0.9.15)* Static member functions import as functions of the type with no
  - why: Choosing by the receiver's kind is circular inside a class method (what `this` may do is worked out from the body that holds the call, verdict 86) and choosing by the result's use lets the lines after a call pick a C++ body; two names make the choice a word on the line, the convention Annex C already uses for `get_mut` and `span_mut`, and let an overlay vouch for either member alone.

- FINAL EDIT rule=FFI-12
  - old: An overlay entry whose name, parameter count or parameter names
  do not match the header is `E5010`, so overlays cannot drift.
  - new: An overlay entry whose name, parameter count or parameter names
  do not match the header, or whose parameter words or types differ from the import's, is `E5010`,
  so overlays cannot drift; so is an entry, a `rename` or a `hide` that fits none, or more than one,
  of a name's several members (`[FFI-57]`, `[GRM-35]`), so that no entry vouches for a function its
  writer did not name, the message giving each member and the word that tells them apart: "`fn
  set(name: borrowed nul_terminated, value)` fits `void set(const char*, int)` and `void set(const
  char*, float)` of `gl::Shader`; write one entry per member, with the type that tells them apart:
  `value: c_int` and `value: f32`".
  - why: An entry asserts that a call within its contract cannot break memory safety (`[FFI-2]`), so it must name exactly one function; name, count and parameter names cannot tell `set(const char*, int)` from `set(const char*, float)`.

- FINAL EDIT rule=FFI-30c
  - old: a later file's `override` entry replaces an earlier file's for that name, and any other
  second entry for one name is `E5010` (ODR-149).
  - new: a later file's `override` entry replaces an earlier file's for that function — of a name declared
  several times, the member its words name (`[FFI-57]`, `[GRM-35]`) — and any other second entry
  for one function is `E5010` (ODR-149).
  - why: Composition counts entries by name; two entries for two members of one name are not a second entry for one function.

- FINAL EDIT rule=GRM-35 grammar block
  - old:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  - new:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | overlay_name | extend_decl | overlay_class)
  overlay_name  := "rename" member_ref "as" IDENT NEWLINE | "hide" member_ref NEWLINE
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} (overlay_fn | overlay_name)} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" (type {contract} | contract {contract})] NEWLINE
  overlay_param := [param_word] IDENT [":" [type] {contract}]
  member_ref    := IDENT ["(" [overlay_param {"," overlay_param} [","]] ")" ["->" type]]
  - why: The one entry form gains the words Ember's own `param` already has (`[param_word] IDENT ":" type`), optional, and `rename`/`hide` gain the same parameter list and an optional result type, so one line can name one of several members and a member can be renamed or hidden under its class (verdict 131); no second entry form, no C++ text in a string.

- FINAL EDIT rule=GRM-35 prose, the `override` sentence
  - old: item may begin with `override`, replacing an earlier overlay's entry for that name (`[FFI-30c]`,
  - new: item may begin with `override`, replacing an earlier overlay's entry for that function (`[FFI-30c]`,
  - why: The unit an overlay describes is the function, as `[FFI-30c]` now says.

- FINAL EDIT rule=GRM-35 prose (append after the last sentence of the rule)
  - old: an overlay of only `rename` and `hide` items needs no `unsafe`.
  - new: an overlay of only `rename` and `hide` items needs no `unsafe`.

  A parameter's word (`ref mut`, `owns`) and type, and a result's type, may be written in any entry
  and are checked against the import (`[FFI-12]`): each is the one the import gives (`[FFI-8]`'s
  spelling, so `c_long` names C++'s `long` on every target; `[FFI-11]`, `[FFI-17a]`, `[FFI-40]`), a
  type of the header named as its own `class` line names it. After a parameter's colon, a word of
  `[FFI-11]`'s vocabulary is a contract word and any other word is the type, so a type whose name is
  a contract word is first given another (`rename status as Status`, then `s: Status`). An entry, a
  `rename` or a `hide` names one function: by the name alone where the header declares it once, else
  by as many of its parameters' count, names, words and types, and its result's type, as tell it
  from the name's other members (`[FFI-57]`); one that fits none, or two, is `E5010` (`[FFI-12]`). A
  `rename` with a parameter list gives that one member a name of its own, outside the set; a `hide`
  with one removes it from the set; a `rename` or `hide` of a member goes under its class (verdict
  131); either needs no `unsafe`.
  - why: Says how the new grammar is read with no new vocabulary: types are Ember's, contract words are `[FFI-11]`'s closed list, so the reading never changes when a header adds an overload, and the one collision is solved by the `rename` the grammar already has.

- FINAL EDIT rule=FFI-55
  - old: one `blake3:` fingerprint of
  each declaration the overlay describes, as the importer binds it (`[FFI-14]`):
  - new: one `blake3:` fingerprint of
  each declaration the overlay describes, as the importer binds it (`[FFI-14]`), keyed by its name
  and, for a C++ name of several members (`[FFI-57]`, `[FFI-40b]`), by the declaration as the
  header spells it, quoted as TOML asks (`"gfx::Renderer::draw(const gfx::Mesh&)" = "blake3:…"`),
  one line per member, which a `rename` in the overlay leaves as it is:
  - why: The record's example keys a line by name; two members would share a key, and a key taken from the overlay's words would move when an entry is edited and let a changed declaration be recorded fresh without a person's check (verdict 121).

- FINAL EDIT rule=MNG-1
  - old: A method's path is its type's path and its name, with the interface's path before the name for a
  method told apart by the interface it implements (`[TYP-24]`).
  - new: A method's path is its type's path and its name, with the interface's path before the name for a
  method told apart by the interface it implements (`[TYP-24]`); a member of a C++ name declared
  several times (`[FFI-57]`) adds, after its name, `O` and the 32 hex digits of the BLAKE3 hash of
  its C++ declaration's canonical parameter types and qualifiers, so that no two members share a
  symbol, a thunk or a call-table slot (`[HR-7]`).
  - why: Mangling by path alone gives two members one symbol; `[HR-7]` names call-table slots by mangled name and `[FFI-3]`'s thunks need distinct `extern "C"` names.

- FINAL EDIT rule=COST-3 table, row "FFI call"
  - old: | FFI call | guaranteed present | one direct call for an ABI-direct C function |
  - new: | FFI call | guaranteed present | one direct call for an ABI-direct C function, and of the thunk for a C++ function (`[FFI-3]`), the member its use stands for where the name has several (`[FFI-57]`); a call that leaves a default argument out runs that default's expression inside the thunk, as a C++ call site that leaves it out would |
  - why: `[COST-5]`: every cost has a row; the only thing that runs is what the equivalent C++ call runs.

- FINAL EDIT rule=CLI-5
  - old: every derived contract, every unknown fact as a
  `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason.
  - new: every derived contract, every unknown fact as a
  `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason, and each
  member of a name declared several times (`[FFI-57]`) as its own entry, with the fewest of its
  parameters' words and types that name it.
  - why: The starter overlay must compile unchanged, so it writes one entry per member with the least that names it.

- FINAL EDIT rule=registry row E5010
  - old: | `E5010` | overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes | `[FFI-12]`, `[FFI-55]` |
  - new: | `E5010` | overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes; an entry, `rename` or `hide` that fits none or several of a C++ name's members | `[FFI-12]`, `[FFI-55]` |
  - why: The row names every case its defining rules give.

- FINAL EDIT rule=registry, new row E5035 after E5034
  - old: | `E5034` | unsupported C++ construct | `[FFI-48]` |
  - new: | `E5034` | unsupported C++ construct | `[FFI-48]` |
| `E5035` | a use of a C++ name of several members (a call, `T.new(…)`, `super.init(…)`, a function value) that fits no member, fits two that neither beats, or chooses one declared `= delete` | `[FFI-57]` |
  - why: One code, one defining rule: `E2070` stays `[TYP-24]`'s ambiguous interface method; `E5035` is free in the registry and the retired table.

- FINAL EDIT rule=registry row E5056
  - old: | `E5056` | an `override fn` matching no virtual method of the C++ base, by name and signature | `[FFI-39]` |
  - new: | `E5056` | an `override fn` matching no virtual method of the C++ base by name and first line exactly, an upcast or a widening counting for nothing | `[FFI-39]` |
  - why: The row claims what its rule now says.

- FINAL EDIT rule=§XVII.6.2 shape table, row N11
  - old: | N11 | ambiguous method (`E2070`) | `I.m(recv, …)` for each candidate |
  - new: | N11 | ambiguous method (`E2070`); a use of a C++ name of several members that fits none or two (`E5035`) | `I.m(recv, …)` for each candidate; each member as the header declares it, and the cast, `rename` or `hide` line to write (`[FFI-57]`) |
  - why: `[DIA-12]`: every name error has a shape; the new code's help is stated once.

**final_cost_row**: | FFI call | guaranteed present | one direct call for an ABI-direct C function, and of the thunk for a C++ function (`[FFI-3]`), the member its use stands for where the name has several (`[FFI-57]`); a call that leaves a default argument out runs that default's expression inside the thunk, as a C++ call site that leaves it out would |  — nothing else runs: which member a use stands for is decided when compiling from the import and the use's own words; no table, tag, test or extra argument; the only growth is one thunk per member and per default-argument arity (code size, implementation-defined under `[COST-2]`), inlined under the project's LTO (`[BLD-FFI-4]`).

**owner_verdicts_touched**: None overridden. 86 (`changes`/`reads` removed; a method's effect on `this` read from its body): kept, and no longer load-bearing for selection, since the `_mut` split makes the pair two names and `this` decides nothing. 117 (`[TYP-26]`'s pack sentence): untouched. 121 (`ember.overlays` written by Ember, never by hand): kept; the record gains one line per member, keyed by the header's own spelling of the declaration, still written only by Ember. 125 and 134 (`@safety`/`L3015` per imported C++ function, members included, once each): held per member, each with its own entry line. 130, 133, 136 (`unsafe fn` spelling): untouched. 131 (member lines under their class): kept and extended to `rename`/`hide` of members. 132 (`@ffi(const)` keeps a method const; an entry without it makes it changing): kept as written, on the `const` member's own entry; the non-`const` member (`at_mut`) is changing in any case. N74/F24 of AUDIT-0.9.20 (`ref_to_ptr` is one built-in, not an overload): consistent, Ember declares no overload. N83 (`ember.overlays` only beside a manifest): untouched.

**remaining_doubt**: (1) `c_int`, `c_long` and the other C aliases are aliases (§XV: \"`c_int` and the other C aliases\"), so on Windows `put(int)`/`put(long)` are one first line to Ember and a typed variable ties at every call until an overlay `hide`/`rename`; literals still pick `put(int)` by the C++-type rule; making the aliases nominal (E1-12) is another ruling. (2) No promotion tier: `lives: u8` into `show(int)`/`show(long long)` ties and needs `lives as c_int`, where C++ promotes; D3's distance tier would accept it at the price of a third rule and a modelled hierarchy; the lead may add it later as a graft without changing anything else. (3) A C++ parameter the header leaves unnamed cannot be given by name at a call nor told apart by name in an entry (pre-existing for `[FFI-12]`); the type now does the work, but the text still does not say what an unnamed parameter's entry name is. (4) How a typed entry spells a type of another namespace (`gfx.Mesh` or `Mesh` as the header's scope sees it); the edit says \"as its own `class` line names it\" and the lead's reading of `[FFI-41]` decides. (5) Constructors of a by-value standard-layout class (`[FFI-32]` imports it as a `@layout(c) struct`) are addressed by neither the current text nor this ruling. (6) `operator[]`'s `const`/non-`const` pair goes through `Index`/`IndexMut` (`[FFI-42]`) and is K47's; untouched. (7) `rename`/`hide` inside a class section is new for every member, not only overloaded ones; harmless. (8) A function with `k` default arguments gets `k+1` thunks whether or not the package calls each arity, keeping the thunk translation unit independent of the calls; the lead may prefer thunks only for arities called. (9) The rule index table and §H.54 are generated by the hardening recipe and are not listed as edits; `E5035`'s exact message wording is the lead's.

## Judgement 2

**winner**: 2

**scores**: [
 {
  "design": 1,
  "safety": 8,
  "speed": 10,
  "readability": 4,
  "consistency": 6,
  "generality": 6,
  "holes": "(1) Needless refusal of the ordinary setter: [FFI-17g] makes `this` \"a name that may change the object where any other line of its method changes it, else one that only reads\", and its implementer step computes the summary \"with its calls to const/non-const pairs left out\". So `fn set_port(i: usize, p: c_int): this.cfg.at(i).port = p` has no *other* changing line, `this` only reads, step 2 picks the const `at`, and the assignment through its `ref` view is `E3023`; no word can reach the non-const `at` from inside the class. (2) One line's member depends on the method's other lines (add `this.hits += 1` anywhere in the method and `this.cfg.at(i)` runs another C++ function), against the design's own \"from the use's own words and nothing around it\". (3) Through a read-only name, a pair imported as changing (bodies out of line, no `@ffi(const)`) fits nothing at steps 2 and 3, so the refusal is `E5035` \"fits none\" where the cause is `E3023`. (4) Whole-call ranking refuses `scene.set(slot, 4)` under `set(int,int)`/`set(int,float)` (a loop counter reaches `c_int` only by `[TYP-5]` rule 12, so neither member is exact at step 2 and both fit at step 3) and `out.put(5)` under `put(int)`/`put(long)` on every target, both of which C++ accepts; the lead's method-3 decision chose the literal rule iii (\"`5` counts as C++'s `int`\"), which this design reverses to option ii. (5) `[FFI-55]` keyed \"by the words of its entry that name it\" gives one declaration a different key in every overlay that spells it differently (`fn draw(mesh)` in one, `fn draw(mesh: Mesh)` in another), so two overlays for one header cannot share a record line. (6) The thunk's distinct name lives only in the implementer steps (\"derived from the USR hash\"); the text says \"its own thunk\" and nothing of its symbol. No accepted program breaks memory safety: overrides are matched exactly, every tie is refused."
 },
 {
  "design": 2,
  "safety": 9,
  "speed": 10,
  "readability": 7,
  "consistency": 7,
  "generality": 7,
  "holes": "(1) Whole-call exact tier: `scene.set(slot, 4)` under `void set(int, int)`/`void set(int, float)`, `slot` a loop counter of `0..4` (reaches `c_int` by `[TYP-5]` rule 12 only), fits both \"with none exact\" and is `E5035`; C++ runs `set(int,int)`. Same for `c.show(lives)` (`u8`) under `show(int)`/`show(long long)`: C++ promotes to `int`, Ember refuses. Refusals, never wrong members. (2) Types required in every entry of a name declared several times (\"an entry ... that writes no type ... is `E5010`\"): `fn draw(mesh: Mesh)`, `fn new(width: c_int, height: c_int)` where the parameter names already tell, words with no meaning (owner principle 4, and the lead's decision b: \"needed only where the names do not tell\"); and the reading of a parameter's words flips when a header gains an overload. (3) `E5010` gains a third defining rule in the registry (`[FFI-12]`, `[FFI-55]`, `[FFI-57]`) and `[FFI-57]` itself says \"is `E5010`\": one code, two places. (4) Overstatement: \"Two members whose first lines are one on the target compiled for ... fit every call together\": under its own literal rule `put(5)` is exact on C++'s `int` and picks `put(int)` on Windows too; only a typed `i32` argument ties. (5) `[MNG-1]` is cited for thunk symbols, which `[FFI-3]` says the importer generates as `extern \"C\"` names; `[HR-7]` call-table slots hold Ember functions, not imported ones, so the \"so that no two members share a symbol\" motive is right but the `[HR-7]` reason is not. (6) Static and non-static members of one name, and named arguments (\"leaves names to `[FFI-12]`\" although `[TYP-25]` lets a call name a parameter), are unaddressed."
 },
 {
  "design": 3,
  "safety": 6,
  "speed": 10,
  "readability": 6,
  "consistency": 5,
  "generality": 8,
  "holes": "(1) The pair as \"one method with two forms\" with \"one entry\" cannot be written: `[FFI-11]` says \"`ref mut` and `MutSpan[T]` results allowed only with `exclusive`; a result is `aliased` unless stated\". The design's own accepted overlay, `@ffi(const) fn light(i) -> borrowed one from(this)`, has no `exclusive`, so the non-`const` form's result is a `ref Light`, and its own line `r.light(0).intensity = 0.5` is `E3023` (a change through a view that only reads); written with `exclusive`, the `const` form's result would be a `ref mut` from a method asserted to only read. One entry cannot give two functions their two result words. (2) One entry, one `@safety`, vouches for two C++ functions, the very thing verdict 131's problem (\"a top-level `fn new(x, y)` made both ... safe\") and `[FFI-2]` (\"an overlay entry for the function, which asserts that a call within its contract cannot break memory safety\") forbid; the design's own \"every declaration keeps its own entry\" contradicts it. (3) The form is chosen \"where the call's result is a view ... that is only read\" or \"kept as a `ref mut` view\": a use-after analysis the text does not define, and the lead's method-3 decision 4 rejected option c for this reason (\"which C++ function a call runs depends on the lines after it\"). `[TYP-21]`/`[STD-17]` are cited as precedent, but `[STD-17]` chooses `IndexMut` by the form `a[i] = v`, a word on the line. (4) `r#status` rests on `[LEX-14]`, which reads \"A raw identifier `r#name` uses a keyword as a name\"; contract words are not keywords, so the sentence adds a new reading of `r#` that `[LEX-14]` does not give. (5) `ember bind --init` (`[CLI-5]`, \"writes a starter overlay ... It compiles unchanged\") is not edited, so what it writes for `set(const char*, int)`/`set(const char*, float)` is unsaid, though the question asks it. (6) The per-argument dominance ranking is the design's strength (it accepts `scene.set(slot, 4)` and `c.show(lives)` as C++ does) and is safe: every coercion ranked is a lossless widening or an upcast. Hole (1) is why safety scores 6: an accepted-looking program whose overlay the text cannot check."
 }
]

**grafts**: Winner: the second design ([FFI-57] with the `_mut` split, [FFI-40b]). Its frame is kept: one Ember name of several members, each member its own thunk, entry, `@safety`, `L3015` and record line; selection from the header and the use's own words only, exact before coercion, a literal exact for C++'s `int`/`double`; no receiver-kind or result-use selection; overrides matched exactly; the pair split into `at`/`at_mut` so that no entry vouches for two functions and nothing is circular on `this`; the record keyed per member by the header's spelling; the `[MNG-1]` component for distinct symbols; the §X.4 row clause; the `[CLI-5]`, `[FFI-44]`, `[FFI-30c]`, §I.6 edits. Grafted from design 3: per-argument dominance (\"takes each argument at least as directly, and one more directly\"), with nearness for typed arguments (narrowest widening, nearest base), so `scene.set(slot, 4)` and `c.show(lives)` run what C++ runs and a literal reaches every type not its own alike; named arguments (`[TYP-25]`); the typed overlay parameter optional anywhere and required only where names do not tell the members apart (the lead's decision b), with the reading \"a word of `[FFI-11]`'s vocabulary is a contract word, any other word is the type\"; `[FFI-17b]`'s clause that a template's instantiations are members of one name. Grafted from design 1: `= delete` members take part and are refused when chosen; one member per accepted arity with the `count(n)` caveat; the `rename`/`hide` with a result type (`rename get() -> f64 as get_f64`) in place of the `[FFI-48]` row, which is removed because the construct now imports; `rename status as Status` (existing grammar) for a type named like a contract word, instead of design 3's `r#status`; the thunk's distinct name stated in `[FFI-17]`. Dropped from design 2: the requirement that every entry of an overloaded name write types (the type is optional and checked where names tell), the header-dependent reading of a parameter's words, the `[FFI-48]` row for a `_mut` collision (a class that declares `at_mut` itself simply makes a name of several members, told apart by `rename`), and `[FFI-57]` as a third defining rule of `E5010` (the refusal is defined in `[FFI-12]`, as today). Dropped from design 1: the three-step receiver and `this` rule. Dropped from design 3: the one-entry two-forms pair and the result-use choice.

- FINAL EDIT rule=TYP-26
  - old: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types.
  - new: There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. A C++
  name a header declares several times is one Ember name of several members, each use standing for
  one (`[FFI-57]`): the header, not the program, declared them, and a function the program declares
  beside an imported name is `E1030` as any other.
  - why: Read literally the rule refuses every C++ class with two constructors; the exception names its reason and its one rule, and §I.6's non-goal stays true for Ember's own functions.

- FINAL EDIT rule=§I.6 Non-goals
  - old: implicit numeric narrowing; function overloading
  - new: implicit numeric narrowing; function overloading (a C++ header's several functions of one name reach Ember as one name of several members, each use standing for one, `[FFI-57]`)
  - why: A reader of Part I must not find Annex C contradicting it; the non-goal still holds for every function Ember declares.

- FINAL EDIT rule=FFI-17
  - old: one thunk per imported function, method, constructor and destructor.
  - new: one thunk per imported function, method, constructor and destructor, a name declared several
  times giving one per member and a function with default arguments one per number of arguments it
  accepts, each passing only the arguments given so that C++ supplies the defaults (`[FFI-57]`); each
  thunk's `extern "C"` name is derived from its declaration (`[FFI-30]`, `[MNG-1]`), so no two share
  one.
  - why: The thunk count is where a reader asks what two constructors become; one thunk per member and per arity keeps every default argument's value in C++ and makes a call one direct call; distinct symbols are stated in the text, not left to the implementer.

- FINAL EDIT rule=FFI-17b
  - old: an Ember generic cannot instantiate a C++ template (`E5055`).
  - new: an Ember generic cannot instantiate a C++ template (`E5055`); the listed instantiations of one
  function template are the members of one name (`[FFI-57]`).
  - why: `instantiate=["max<int>", "max<double>"]` makes two functions of one name; they follow the same selection and need no invented names.

- FINAL EDIT rule=Annex C §C.1, new bullet directly after [FFI-32] (before [FFI-11f])
  - old: 
  - new: * `[FFI-57]` *(new in 0.9.20)* **A name declared several times.** A C++ name the header declares
  with several parameter lists — a function of a namespace, a method, a static member function, a
  constructor (`new`, `[GRM-35]`), the listed instantiations of one function template (`[FFI-17b]`) —
  imports as one Ember name of several **members**, one per declaration (a redeclared function
  counting once, `[FFI-30]`), each with its own thunk (`[FFI-17]`), its own entry (`[GRM-35]`), its
  own `@safety` and `L3015` (`[UNS-7]`, verdicts 125 and 134) and its own line of `ember.overlays`
  (`[FFI-55]`); a `const` and a non-`const` pair is split into two names first (`[FFI-40b]`). A
  function with default arguments is such a name too: one member for each number of arguments it
  accepts, each arity's thunk calling the function with the trailing arguments left out, so that C++
  supplies the defaults as at a call of its own (a count that leaves out a parameter another
  parameter's contract word names, `count(n)`, but not that parameter, is no member). The program
  declares none of them, so the name is not `E1030` (`[TYP-26]`). Each use of the name stands for one
  member, read from the header and the use's own words, nothing around it. A call (a method call,
  `T.new(…)`, `super.init(…)`, `[FFI-39b]`) fits a member when that member, declared alone, would
  accept it: its arguments' number, by position or by name (`[TYP-25]`; a parameter with a default
  may be left out), the word before each (`ref mut`, `owns`, none: `[FN-2a]`, `[FFI-40]`) and each
  argument's type, the parameter's own or one that reaches it by a coercion of `[TYP-5]`; an
  untyped literal's own type is the one C++ gives its spelling, `c_int` for a whole number and `f64`
  for one with a point, and any other type it may take (`[LEX-16]`, `[LEX-17]`) it reaches as by a
  coercion. Of the members a call fits, one beats another when, argument by argument, it takes each
  at least as directly — the argument's own type before a coercion and, for an argument of a known
  type, among coercions the nearer type (the narrowest number type it widens to, the nearest base
  class) before a farther; an untyped literal reaches every type not its own alike — and takes one
  more directly; the call stands for the member that beats every other it fits, so where C++ chooses
  one Ember chooses the same, and never one C++ would not consider. A member declared `= delete`
  takes part and, chosen, is refused. A name used without a call stands for the member whose first
  line is exactly the callable type expected there (`[CLO-3]`), and with none expected is `E5035`.
  What the code around the use allows — unsafe code (`[FFI-2]`, `E3100`), a read-only name
  (`E3023`), its effects, the loop and view rules — is checked after, on the member chosen, so one
  line stands for one member wherever it is written. A use that fits no member, or two with neither
  beating the other, is `E5035`, naming each member as the header declares it and why it does or does
  not fit, and the way out, the argument written with its type or the overlay line: "`w.value(7)`
  (line 8) fits `void value(long long)` and `void value(double)` of `ui::Widget` alike: `7` is C++'s
  `int`, which neither takes as it is, and reaches each; write `w.value(7 as i64)` or
  `w.value(7.0)`". Two members alike in every parameter as the import spells them (two instances of
  one template differing only in the result), or alike on this target (`put(int)` and `put(long)` for
  an `i32` argument where `long` is 32 bits; a literal still takes `int`), fit such a call together;
  the overlay tells them apart by the tables' spelling or by the result, which no target changes
  (`rename put(v: c_long) as put_long`, `hide put(v: c_long)`, `rename get() -> f64 as get_f64`,
  `[GRM-35]`), and the message of `E5035` gives that line. An `override fn` matches one virtual
  member by its first line exactly (`[FFI-39]`), never through this ranking. `ember bind --report`
  lists each such name with its members as the header writes them; `ember bind --emit-cpp` writes
  one thunk per member, named by the member's mangled name (`[MNG-1]`); `ember bind --init` writes
  one entry per member (`[CLI-5]`). Nothing runs for this while the program runs: a use is one direct
  call of its member's thunk, as any imported C++ function's (§X.4, "FFI call").
  - why: The missing rule, a property (a name declared several times, and what a use's own words fit) decided from the header and the line alone (`[BLD-16]`), with `[TYP-24]`'s exact-before-coercion made per argument so that the ordinary C++ lines (`set(slot, 4)`, `show(lives)`, `put(5)`) run what C++ runs and every unclear call is refused with the line to write; each member keeps its own entry, `@safety`, record line and thunk, so no promise is shared between two functions.

- FINAL EDIT rule=Annex C §C.3, new bullet directly after [FFI-40a]
  - old: 
  - new: * `[FFI-40b]` *(new in 0.9.20)* **A `const` and a non-`const` pair.** A non-`const` method beside a
  `const` method of the same name and the same parameters, as the import maps them, imports with
  `_mut` added to its name — `at` and `at_mut`, as `get` and `get_mut` (`[FFI-42]`) and `span` and
  `span_mut` (`[FFI-17a]`) — so the two are two names, and which one a call runs is a word on the
  line (`cfg.at(i)`, `cfg.at_mut(i).port = 8080`), never read from the name the object is reached
  through nor from the lines around it; `at_mut` may change its object (`[FFI-40]`), and the `const`
  one only reads it where `[FFI-40a]` finds so or its entry says `@ffi(const)`. An overlay enters
  each under its own name (`@ffi(const) fn at(i) -> borrowed one from(this)`, `fn at_mut(i) ->
  borrowed one exclusive from(this)`, `[FFI-12]`), and an `override fn` of a virtual pair names the
  member it replaces the same way (`[FFI-39]`). A class that itself declares the added name makes it
  a name of several members (`[FFI-57]`), which a `rename` in the overlay tells apart (`[GRM-35]`).
  `operator[]`'s pair is `Index` and `IndexMut` (`[FFI-42]`), not this rule.
  - why: Choosing by the receiver's kind is circular inside a class method (what `this` may do is worked out from the body that holds the call, `[CLS-7]`, verdict 86) and choosing by the result's use reads the lines after the call; two names make the choice a word on the line, as Ember's bridge already does, and give each member an entry whose two result words (`exclusive` or not) `[FFI-11]` can check.

- FINAL EDIT rule=FFI-39
  - old: An `override fn` matching no virtual
  method of the base, by name and signature, is `E5056`.
  - new: An `override fn` matching no virtual
  method of the base, by name and first line — each parameter's word and type exactly those the
  import gives the virtual method's (`[FFI-40]`), and its result; an upcast or a widening counts for
  nothing, since C++ hands the override its own argument (a `FocusEvent` for an `on(const Event&)`)
  — is `E5056`, whose message gives each virtual method of that name as the `override fn` line to
  write ("`on(e: gui.ui.KeyEvent)` overrides no virtual method of `ui.Handler`: it declares `on(e:
  gui.ui.Event)` and `on(timer_id: c_int)`; write `override fn on(e: gui.ui.Event):` or `override fn
  on(timer_id: c_int):`"); where the base declares the name several times (`[FFI-57]`), the override
  replaces the one member it matches and the trampoline passes on that one, the others staying the
  base's.
  - why: An override matched through an upcast would read a smaller base object as a derived one in safe code, and one matched through a widening would truncate C++'s argument: the coercion runs backwards for an override. "By name and signature" is made exact and told how an overloaded virtual is replaced.

- FINAL EDIT rule=FFI-39b
  - old: `super.init(…)` selects the C++ base constructor by arity and
  argument types and runs it at `super.init(…)` (`[CLS-11]`);
  - new: `super.init(…)` selects the C++ base constructor as a call of the base's `new`
  selects a member (`[FFI-57]`: by its arguments' number, names, words and types, exact before
  coercion; none or a tie `E5035`) and runs it at `super.init(…)` (`[CLS-11]`);
  - why: One selection rule for constructors wherever they are called; "by arity and argument types" said neither how a literal counts nor what a tie is.

- FINAL EDIT rule=FFI-40a
  - old: An entry whose name or
  parameters differ from the import's member is `E5010` (`[FFI-12]`).
  - new: An entry whose name or
  parameters differ from the import's member, or that fits none or several members of one name
  (`[FFI-57]`), is `E5010` (`[FFI-12]`); the non-`const` member of a pair is entered under its own
  name, `at_mut` (`[FFI-40b]`).
  - why: The class section's matching rule names the pair's second name and the several-members case where it already names `E5010`.

- FINAL EDIT rule=FFI-12
  - old: An overlay entry whose name, parameter count or parameter names
  do not match the header is `E5010`, so overlays cannot drift.
  - new: An overlay entry whose name, parameter count or parameter names,
  or the words and types it writes (`[GRM-35]`), do not match one declaration of the header, or
  that fits none or several members of a name declared several times (`[FFI-57]`), is `E5010`, so
  overlays cannot drift and no entry vouches for a function its writer did not name; for several,
  the message gives each member's entry line with the type that tells them apart: "`fn set(name:
  borrowed nul_terminated, value)` fits `void set(const char*, int)` and `void set(const char*,
  float)` of `gl::Shader`; write one entry per member: `fn set(name: borrowed nul_terminated, value:
  c_int)` and `fn set(name: borrowed nul_terminated, value: f32)`". A `rename` or a `hide` with a
  parameter list is matched as an entry is.
  - why: An entry asserts that a call within its contract cannot break memory safety (`[FFI-2]`), so it must name exactly one function; name, count and parameter names cannot tell `set(const char*, int)` from `set(const char*, float)`. The refusal stays at the rule that already matches entries, with the same code.

- FINAL EDIT rule=FFI-30c
  - old: a later file's `override` entry replaces an earlier file's for that name, and any other
  second entry for one name is `E5010` (ODR-149).
  - new: a later file's `override` entry replaces an earlier file's for that function (for a name
  declared several times, the member its words name, `[FFI-57]`), and any other second entry for
  one function is `E5010` (ODR-149).
  - why: Composition counts entries by name; two entries for two members of one name are not a second entry for one function.

- FINAL EDIT rule=GRM-35 grammar block
  - old:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl
                   | overlay_class)
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} overlay_fn} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  - new:   overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | overlay_name | extend_decl | overlay_class)
  overlay_name  := "rename" member_ref "as" IDENT NEWLINE | "hide" member_ref NEWLINE
  overlay_class := "class" IDENT ":" NEWLINE INDENT {{attribute} (overlay_fn | overlay_name)} DEDENT
  overlay_fn    := ["unsafe"] "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" (type {contract} | contract {contract})] NEWLINE
  overlay_param := [param_word] IDENT [":" [type] {contract}]
  member_ref    := IDENT ["(" [overlay_param {"," overlay_param} [","]] ")" ["->" type]]
  - why: The one entry form gains the optional word and type Ember's own `param` already has (`[param_word] IDENT ":" type`), and `rename`/`hide` gain a parameter list and a result type, so one line can name one of several members, and a member's `rename` or `hide` can go under its class (verdict 131). No second entry form, no C++ text in a string.

- FINAL EDIT rule=GRM-35 prose
  - old: since a macro has no C declaration to read (ODR-147). An
  item may begin with `override`, replacing an earlier overlay's entry for that name (`[FFI-30c]`,
  ODR-149).
  - new: since a macro has no C declaration to read (ODR-147). In any other entry a parameter may carry,
  before its contract words, the word and the type the import gives it from the header's
  declaration (`owns tex: Texture`, `value: c_int`, `s: *c_char`), by the spelling of `[FFI-8]`'s,
  `[FFI-17a]`'s and `[FFI-40]`'s tables, so that `c_long` names C++'s `long` on every target; each is
  checked against the header (`[FFI-12]`), and is written where the entry's name and parameter names
  fit several members of one name (`[FFI-57]`). After the colon, a word of `[FFI-11]`'s vocabulary
  is a contract word and any other word is the type; a type of the header whose name is a contract
  word is first given another (`rename status as Status`), which the message of `E5010` says. A
  `rename` or a `hide` with a parameter list names one member of such a name (`rename put(v:
  c_long) as put_long`, `hide put(v: c_long)`, `rename get() -> f64 as get_f64` for two members
  alike in every parameter), inside a class section one of its members (verdict 131); without one it
  names every member of the name; a `rename` gives the member a name of its own, outside the set. An
  item may begin with `override`, replacing an earlier overlay's entry for that function
  (`[FFI-30c]`, ODR-149).
  - why: States how the new grammar is read with no new vocabulary: types are the tables' spellings, contract words are `[FFI-11]`'s closed list, the one collision is solved by a `rename` the grammar already has, and the unit an `override` replaces is the function.

- FINAL EDIT rule=FFI-55
  - old: one `blake3:` fingerprint of
  each declaration the overlay describes, as the importer binds it (`[FFI-14]`):
  - new: one `blake3:` fingerprint of
  each declaration the overlay describes, as the importer binds it (`[FFI-14]`), keyed by the
  declaration's name and, for a member of a C++ name declared several times (`[FFI-57]`), by the
  declaration as the header spells it, quoted as TOML asks (`"gfx::Renderer::draw(const
  gfx::Mesh&)" = "blake3:…"`), one line per member, which a `rename` in the overlay does not
  change:
  - why: The record's example keys a line by name; two members would share a key and the check of verdict 121 would compare an entry against the wrong declaration or lose one. The header's spelling is the same in every overlay of the header.

- FINAL EDIT rule=FFI-48 table row "overloads differing only in return type"
  - old:   | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  | casts across a hierarchy the importer did not model |
  - new:   | casts across a hierarchy the importer did not model |
  - why: A C++ header cannot declare two functions differing only in the result; the case that can occur, instances of one template alike in every parameter, now imports under `[FFI-57]` (a call that fits both is `E5035`, `rename get() -> f64 as get_f64` the way out), so the row named a construct that is no longer unsupported and a `rename` the old grammar could not write.

- FINAL EDIT rule=FFI-44 table, new row after "explicit template instantiations"
  - old:   | explicit template instantiations | automatic (`[FFI-17b]`) |
  - new:   | explicit template instantiations | automatic (`[FFI-17b]`) |
  | a name declared several times (overloads, default arguments, a `const` and non-`const` pair) | automatic (`[FFI-57]`, `[FFI-40b]`) |
  - why: The table says what imports and how; the most common construct of C++ headers was missing from it.

- FINAL EDIT rule=CLI-5
  - old: `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason.
  - new: `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason, and each
  member of a name declared several times as its own entry, with the words and types that tell it
  from the others and no more (`[FFI-57]`, `[GRM-35]`).
  - why: The starter overlay must compile unchanged (`[CLI-5]`), so it writes one entry per member in the least form that names it.

- FINAL EDIT rule=COST-3 table, row "FFI call"
  - old: | FFI call | guaranteed present | one direct call for an ABI-direct C function |
  - new: | FFI call | guaranteed present | one direct call for an ABI-direct C function, and one direct call of the member's thunk for a C++ name declared several times (`[FFI-57]`); a call that leaves a default argument out runs the default's expression in the thunk, as the C++ call site that leaves it out would |
  - why: `[COST-5]`: the only thing that runs is what the equivalent C++ call runs, and the row now says so for members and defaults.

- FINAL EDIT rule=MNG-1
  - old: A method's path is its type's path and its name, with the interface's path before the name for a
  method told apart by the interface it implements (`[TYP-24]`).
  - new: A method's path is its type's path and its name, with the interface's path before the name for a
  method told apart by the interface it implements (`[TYP-24]`); a member of a C++ name declared
  several times (`[FFI-57]`) adds, after its name, `O` and the 32 hex digits of the BLAKE3 hash of
  its declaration as the header spells it (the key of `[FFI-55]`), so that no two members share a
  symbol or a thunk name.
  - why: Mangling by path alone would give two members one symbol; the component reuses the record's key, so one spelling names a member everywhere.

- FINAL EDIT rule=registry row E5010
  - old: | `E5010` | overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes | `[FFI-12]`, `[FFI-55]` |
  - new: | `E5010` | overlay does not match the C declaration; a declaration an overlay describes differs from the one `ember.overlays` records, or the package has no record of the overlay; an entry outside a class section that names only members of classes; an entry, `rename` or `hide` whose words fit none or several members of a C++ name declared several times | `[FFI-12]`, `[FFI-55]` |
  - why: The row lists every case of the code; the defining rules stay the two that match entries and the record.

- FINAL EDIT rule=registry, new row after E5034
  - old: | `E5034` | unsupported C++ construct | `[FFI-48]` |
  - new: | `E5034` | unsupported C++ construct | `[FFI-48]` |
| `E5035` | a use of a C++ name declared several times (a call, `T.new(…)`, `super.init(…)`, a function value) that fits no member, that two members fit with neither beating the other, or whose chosen member is declared `= delete` | `[FFI-57]` |
  - why: One code, one defining rule; `E2070` stays `[TYP-24]`'s ambiguous interface method, and `E5035` is free in the registry and the retired list.

- FINAL EDIT rule=registry row E5056
  - old: | `E5056` | an `override fn` matching no virtual method of the C++ base, by name and signature | `[FFI-39]` |
  - new: | `E5056` | an `override fn` matching no virtual method of the C++ base by name and first line exactly, an upcast or a widening counting for nothing | `[FFI-39]` |
  - why: The row claims what its rule now says.

**final_cost_row**: | FFI call | guaranteed present | one direct call for an ABI-direct C function, and one direct call of the member's thunk for a C++ name declared several times (`[FFI-57]`); a call that leaves a default argument out runs the default's expression in the thunk, as the C++ call site that leaves it out would |  — nothing else runs: which member a use stands for, the `_mut` split, the overlay matching and the override check are decided when compiling; no table, tag or test exists while the program runs; code size grows by one thunk per member and per default-argument arity (implementation-defined under `[COST-2]`); compile time adds one fit test per member at each use of such a name.

**owner_verdicts_touched**: None overridden. 86 (`changes`/`reads` removed; what a method does to `this` is read from its code): untouched, and the `_mut` split means no selection ever reads `this`'s kind. 117 (`[TYP-26]`'s pack sentence): untouched. 121 (`ember.overlays` written by Ember, never by hand, every overlay checked): kept; the record gains one line per member, keyed by the header's spelling, still written by Ember alone. 125 and 134 (`@safety` in the entry of every imported function left unsafe, `L3015` once per function naming every call, C++ members included): kept per member, each member its own entry and its own `L3015`. 130 and 133 (`unsafe fn` spelling in entries and hand declarations): untouched. 131 (member lines under their class): kept and extended to a member's `rename` and `hide`. 132 (an entry without `@ffi(const)` makes its method changing; with it, const): kept; the `const` member of a pair has an entry of its own under its name, the non-`const` one under `at_mut`, so the word describes exactly one function. Method-3 lead decisions (not owner verdicts): decision 1 (option A) and 3 (literal rule iii) are kept; decision 2 (option b, the type optional and required where names do not tell) is kept; decision 4 (option b, the receiver's kind picks the pair's member) is departed from, for the circularity on `this` and the setter refusal shown in design 1's holes, in favour of two names.

**remaining_doubt**: (1) `c_int`, `c_long` and the other C aliases are aliases (§XV: \"`c_int` and the other C aliases\"), so a typed `i32` argument to `put(int)`/`put(long)` ties on Windows and compiles on Linux; the overlay `rename` by the table's spelling is the portable way out, and making the aliases nominal is the pre-existing question E1-12, not decided here. (2) The `_mut` name is one the header never wrote; Ember's bridge already does so (`get_mut`, `span_mut`), but the lead may prefer one name with two entries told apart by `@ffi(const)` and a call-side rule; every such rule found reads either `this`'s kind (circular) or the result's use (lines around the call), and one entry cannot carry both result words, so two names is the choice made here. (3) How an overlay spells a namespaced type in a typed parameter (`Mesh` as the header's scope sees it, or `gfx.Mesh`): the examples write the header's scope, as `class Renderer:` already does; the lead's reading of `[FFI-41]` decides. (4) `rename status as Status` renames a type; `[FFI-30a]` says an overlay changes \"the signatures, safety and names of *functions*\" and never a type's identity, layout or fields; a name is none of those three, but the lead may wish to add \"and names of types\" to `[FFI-30a]`. (5) A static and a non-static member of one name are told apart by the call's form (`T.f(…)`, `obj.f(…)`, `[FFI-41]`); the text does not say so and C++ allows it; a clause in `[FFI-57]` if the lead wants it. (6) Nearness among coercions: two widenings to types of one width (`u8` to `c_int` and to `c_uint`) tie and are refused, where C++'s promotion picks `int`; refusing is the owner's rule 1, but the lead may add \"a signed type before an unsigned one of the same width\" if the corpus shows the tie common. (7) `E5035` is in the FFI range, outside §XVII.6.2's name-and-type shapes (`[DIA-12]`); if the lead classifies FFI codes there, a shape \"ambiguous C++ member\" with the cast or `rename` line as help is needed. (8) Constructors of a by-value standard-layout class (`[FFI-32]` imports it as a `@layout(c) struct`) are outside this ruling, as they were outside the question. (9) `operator[]`'s pair goes through `Index`/`IndexMut` (`[FFI-42]`) and its overlay name is K47's. (10) The rule index table at the end of the document and §H's tables are generated by the hardening recipe and are not listed as edits; the `*(changed in 0.9.20)*` marks on the edited rules are the lead's. (11) `[FFI-55]`'s fingerprint resolves typedefs to the target's types, so `int64_t` members fingerprint differently per platform: pre-existing, unchanged here.
