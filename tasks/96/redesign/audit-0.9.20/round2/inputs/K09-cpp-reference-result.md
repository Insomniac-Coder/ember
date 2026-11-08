# K09-cpp-reference-result

Question: Must a C++ function returning a reference say where the reference points?
Severity: high; impact rank 9
Members: J8-06; primary J8-06
Why together: Single question.
Critic's notes: Use after free as written today. B is verdict 132's lesson (no fallback to an unwritten fact) and verdict 137's reading (a C++ member has no Ember body to read).

## Finding J8-06
Rules: FFI-40, FFI-2a, FFI-17a, LT-1, FFI-11f; file tasks/spec-0.9.9/parts/p21-annex-c.md lines 58-60, 156-163; p16-ffi.md 43-45; kind contradiction; severity high
Quote from Hardened_11:
`T&&` as `owns x: T` where the type is movable; a `T&`
  result is a `ref mut T` view into the object and a `const T&` result a `ref T` one (`[LT-1]`).

The finder's question:
What does a C++ reference or view result point into, and does it need a lifetime word? FFI-2a says a result's
lifetime is never derived, and FFI-17a requires one for `std::span` results. FFI-40 derives "into the object"
for member `T&` results. Nothing covers a free function's `const T&` result, a `std::string_view` result or a
fixed-extent span result.

Program:
```
// util.hpp
// const int& larger(const int& a, const int& b);
// class Scene { public: Node& pick(Node& start); std::string_view name() const; };
import cpp "util.hpp" with (project="engine") as u

fn main():
    r = u.larger(1, 2)      # plain `a: i32` is Copy, so no source parameter (LT-1): r points into nothing?
    println(r)              # C++ returned the address of a temporary
    s = u.Scene.new()
    n = s.pick(ref mut root)  # "into the object" by FFI-40, though C++ returns into `start`
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.65)
Reason:
FFI-2a (p16-ffi.md 45): "In result position a pointer's lifetime is never derived." FFI-17a's span row
(p21-annex-c.md 58): "a result needs a lifetime word". FFI-40 (162-163) instead derives a member's `T&` result
as "a `ref mut T` view into the object" (LT-1 rule 1). For an Ember method that rule is backed by reading the
body (verdict 137), but C++ has no body to read, and a C++ member that returns into a reference parameter
(`Node& first_child(Node& parent)`) then counts as pointing into the wrong object. For free functions LT-1's
rules 2 and 3 apply, and a `const T&` parameter of a `Copy` type imports as a plain `x: T`, which is no source
parameter (p07-ownership.md 528-529), so the result points into nothing. The `string_view` and fixed-extent
span rows say nothing about results. A call is safe only with an overlay entry (FFI-2), and an entry may write
`-> from(parent)`, but whether an entry without a lifetime word is accepted, and what it then means, is
exactly what the two rules disagree on.

options:
[{'letter': 'A', 'summary': "As today: a C++ reference result needs no word. A method's points into its object
and a free function's follows the three rules, with a `const T&` parameter counted as a source when the result
is a reference. An entry returning into a parameter writes `-> from(parent)`.", 'code': '#     class Scene:\n#
fn root()                              # into the scene\n#         fn first_child(parent) -> from(parent)  #
must be remembered; leaving it out is accepted', 'outcome': "With `-> from(parent)`, `level = sc.Scene.new()`
while `child` is used is refused. Without it, the example compiles and reads freed memory, because the entry's
assertion was wrong and nothing asked for the word."}, {'letter': 'B', 'summary': 'A C++ reference or view
result needs a lifetime word in its overlay entry, as a `std::span` result and a C pointer result already do.
`[[clang::lifetimebound]]` supplies it as an asserted fact.', 'code': '#     class Scene:\n#         fn
new()\n#         fn root() -> from(this)\n#         fn first_child(parent) -> from(parent)', 'outcome': 'The
entry `fn first_child(parent)` with no word is the error naming the missing fact and giving this line. With
the words, `level = sc.Scene.new()` while `child` is used is refused (the view of `level` is still in use).'}]

recommendation:
{'letter': 'B', 'reason': "It is one rule for every foreign result that points somewhere (C pointers,
`std::span`, C++ references). An entry's meaning then comes only from what it says, and a forgotten word is a
refusal, not a read of freed memory. The cost is one word per entry that returns a reference.", 'checklist':
'5 and 6 (one rule with FFI-2a and FFI-17a), 8 (a forgotten mark is refused), 15 (no fallback to an unwritten
fact), 2'}

