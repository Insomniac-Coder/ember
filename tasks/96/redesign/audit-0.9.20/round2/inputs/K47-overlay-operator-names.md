# K47-overlay-operator-names

Question: How does an overlay name a C++ operator[] so that indexing can be safe?
Severity: medium; impact rank 47
Members: J8-05; primary J8-05
Why together: Single question.
Critic's notes: Ask after K21: the const and non-const `operator[]` are an overload pair.

## Finding J8-05
Rules: FFI-42, GRM-35, FFI-2, FFI-40a; file tasks/spec-0.9.9/parts/p21-annex-c.md lines 188-191; kind undecided_case; severity medium
Quote from Hardened_11:
Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to
  `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to read-through: such a class

The finder's question:
Is `grid[i]` on an imported C++ class with `operator[]` safe to write? If it needs an overlay entry to become
safe (FFI-2), or `@ffi(const)` to only read its object (verdict 132), how does that entry name the operator?

Program:
```
// grid.hpp: class Grid { public: float& operator[](size_t i); const float& operator[](size_t i) const; };
import cpp "grid.hpp" with (project="sim") as sim

fn sample(g: sim.Grid) -> f32:
    return g[3]          # safe? `unsafe:` needed? C++ does not check the index

# overlay: unsafe overlay cpp "grid.hpp":
#     class Grid:
#         fn operator[](i)   # not an IDENT in [GRM-35]
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
FFI-2 (p16-ffi.md 40-41) and verdict 125 ("C++ too, same rule ... members included") keep every header member
with no overlay entry unsafe to call. FFI-42 (p21-annex-c.md 188-189) maps `operator[]` to `Index`/`IndexMut`,
whose methods `index`/`index_mut` (p04-types.md 666-671) are ordinary safe methods behind `x[i]`. GRM-35's
`overlay_fn := ["unsafe"] "fn" IDENT ...` has no form for an operator member, so no entry can assert the
operator safe or mark it `@ffi(const)` (verdict 132). Whether `g[3]` on an imported class compiles outside
`unsafe` is not stated, and if it does not, the syntax to make it safe is missing (OWNER-DIRECTION checklist
12).

options:
[{'letter': 'A', 'summary': 'An overlay entry names an operator member by the Ember method it becomes
(`index`, `index_mut`). With no entry, `g[3]` is unsafe code, as any header member is.', 'code': 'unsafe
overlay cpp "grid.hpp":\n    class Grid:\n        fn new(n)\n        @ffi(const)\n        fn size()\n
@ffi(const)\n        fn index(i)          # the const operator[]\n        fn index_mut(i)      # the non-const
operator[]\n\nfn sample(g: sim.Grid) -> f32:\n    return g[3]', 'outcome': 'With the entries, `g[3]` compiles
outside `unsafe`, and the entry vouches that C++ never reads out of bounds. Without them, `g[3]` is the
unsafe-call error, whose help gives these lines.'}, {'letter': 'B', 'summary': 'An overlay entry names an
operator by its C++ spelling, in quotes.', 'code': 'unsafe overlay cpp "grid.hpp":\n    class Grid:\n
@ffi(const)\n        fn "operator[]"(i)', 'outcome': 'Same result as A. The overlay gets a second kind of name
that appears nowhere else in Ember.'}]

recommendation:
{'letter': 'A', 'reason': "The operator already becomes Ember's `index`/`index_mut`, so the overlay uses the
name Ember gives it. The missing syntax is added instead of leaving operators always unsafe, and FFI-2's rule
holds for operators unchanged.", 'checklist': "11 (Ember's own method names), 12 (syntax added, no exemption),
5 (one rule for every member), 15"}

