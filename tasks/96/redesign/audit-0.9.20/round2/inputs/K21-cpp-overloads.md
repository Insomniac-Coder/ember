# K21-cpp-overloads

Question: How do C++ overloaded functions and constructors import into Ember?
Severity: high; impact rank 21
Members: J8-04; primary J8-04
Why together: Single question.
Critic's notes: Nearly every C++ header has overloads. A is an exception to TYP-26 that gives its reason (principle 6). K47 (J8-05) names `operator[]`'s const and non-const pair, so ask after this.

## Finding J8-04
Rules: FFI-48, TYP-26, FFI-39b, FFI-12, GRM-35; file tasks/spec-0.9.9/parts/p21-annex-c.md lines 83-93, 128-130; kind undecided_case; severity high
Quote from Hardened_11:
| overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |

The finder's question:
How does a set of C++ overloads that differ in parameter types (ordinary C++) import, given that Ember has no
overloading (TYP-26, E1030)? What are the Ember names of `draw(const Mesh&)` and `draw(const Sprite&)`, and of
two constructors? How does an overlay entry, `rename` or `hide` pick one of them?

Program:
```
// renderer.hpp
// class Renderer { public: Renderer(); Renderer(int w, int h);
//   void draw(const Mesh& m); void draw(const Sprite& s); };
import cpp "renderer.hpp" with (project="engine") as eng

fn frame(ref mut r: eng.Renderer, mesh: eng.Mesh, hud: eng.Sprite):
    r.draw(mesh)     # which import? one `draw`, two, E1030, E5034?
    r.draw(hud)

fn main():
    r = eng.Renderer.new(1280, 720)   # which `new`?
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.75)
Reason:
TYP-26 (p04-types.md 867-868): "There is no overloading: two functions of one name in one scope are `E1030`".
FFI-48 (p21-annex-c.md 90) refuses only overloads that differ in return type, which implies that the others
import, but no rule says how. FFI-39b (128-129) picks a base constructor "by arity and argument types" for
`super.init` only. FFI-17 (10-12) generates "one thunk per imported function, method, constructor", so a class
with two constructors gets two `new`. GRM-35's `overlay_fn` (p16-ffi.md 260) and FFI-12 (322-323) match
entries by name, count and parameter names, so FFI-48's own fix, "rename one in the overlay", cannot name one
overload. No verdict covers this. C++ overloads, constructors above all, are in nearly every C++ header.

options:
[{'letter': 'A', 'summary': "One Ember name for each C++ name. The call picks the overload by the arguments'
types, as C++ does and as `super.init` already does. An overlay entry names one overload by its parameter
count and names.", 'code': '    r = eng.gfx.Renderer.new(1280, 720)   # the (int, int) constructor\n
r.draw(mesh)                           # draw(const Mesh&)\n    r.draw(hud)                            #
draw(const Sprite&)\n\n# overlay: one entry per overload, told apart by its parameter names\nunsafe overlay
cpp "renderer.hpp":\n    class Renderer:\n        fn new(width, height)\n        fn draw(mesh)\n        fn
draw(sprite)', 'outcome': 'Compiles and calls the overload C++ would call. A call that no overload fits, or
that two fit equally, is an error naming the candidates.'}, {'letter': 'B', 'summary': 'Each overload imports
under its own name, built from its C++ parameter types.', 'code': '    r = eng.gfx.Renderer.new_int_int(1280,
720)\n    r.draw_Mesh(mesh)\n    r.draw_Sprite(hud)', 'outcome': 'Compiles. Each overload is a separate Ember
function. The names come from types the program never wrote, and they change when the header renames a
type.'}, {'letter': 'C', 'summary': 'An overload set imports only through an overlay that renames each
overload. Without that, every overload is skipped with a warning that gives the overlay lines.', 'code':
'unsafe overlay cpp "renderer.hpp":\n    class Renderer:\n        rename new(width, height) as sized\n
rename draw(mesh) as draw_mesh\n        rename draw(sprite) as draw_sprite', 'outcome': 'Without the overlay,
`r.draw(mesh)` is the "skipped: overloaded" diagnostic. With it, `r.draw_mesh(mesh)` compiles.'}]

recommendation:
{'letter': 'A', 'reason': "It keeps what the C++ header means and works with no overlay. It reuses the
selection Ember already does for `super.init`, and Ember's own no-overloading rule only fails to describe C++
names, so the exception says why.", 'checklist': "2 (ordinary C++ imports), 4 (no names to invent), 6 (an
exception only where the general rule is untrue for C++, with the reason), 11 (FFI-39b's selection reused), 13
(no hidden renaming)"}

