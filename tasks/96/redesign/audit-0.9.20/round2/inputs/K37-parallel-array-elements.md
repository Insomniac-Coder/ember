# K37-parallel-array-elements

Question: May a @parallel loop over a range write the elements of an Array?
Severity: medium; impact rank 37
Members: J7-11; primary J7-11
Why together: Single question.
Critic's notes: 

## Finding J7-11
Rules: PAR-2, PAR-1; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 459-461; kind undecided_case; severity medium
Quote from Hardened_11:
(b) all of them share **one** `k`; and (c) `P` is the elements of a `MutSpan` or an `SoA` column the / loop
uses, not a place reached through a link (`shares`, `shares mut`, `sharedref mut`,

The finder's question:
Does a `@parallel` loop over a range that writes `xs[i]` of an `Array` (a local, or a `ref mut xs:
Array[float]` parameter) compile, or must it be a `MutSpan`?

Program:
```
fn brighten(ref mut pixels: Array[float], gain: float):
    @parallel
    for i in 0..pixels.len():
        pixels[i] *= gain
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.5)
Reason:
Clause (c) of PAR-2 (p11 459-461) names only 'the elements of a `MutSpan` or an `SoA` column the loop uses'.
SPN-1 (p07 812-814) converts an `Array` to a `MutSpan` only 'where one is expected' (a `ref mut` parameter or
name), and indexing inside a body expects none. Every range example in the spec uses a MutSpan parameter (p11
438-441, p19 169-171). PAR-1 accepts a by-element walk of an `Array`, so whether `pixels[i]` on an `Array`
counts has no stated verdict. Nothing else settles it.

options:
[{'letter': 'A', 'summary': "An Array's elements count, as a MutSpan's do", 'code': '# the program above,
unchanged\n# clause (c) covers the elements of any contiguous list the loop reaches by a name that may
change\n# it: an `Array`, a `MutSpan`, an `SoA` column; the body cannot push to or remove from it (that is
a\n# write of the list with no `i + k`)', 'outcome': 'Compiles and prints 0.2.'}, {'letter': 'B', 'summary':
'Only a MutSpan or column: take one first', 'code': 'fn brighten(ref mut pixels: Array[float], gain: float):\n
px = ref mut pixels.as_mut_span()\n    @parallel\n    for i in 0..px.len():\n        px[i] *= gain',
'outcome': 'The original is E7010, whose message gives the `as_mut_span()` line. The rewrite compiles and
prints 0.2.'}]

recommendation:
{'letter': 'A', 'reason': 'The two loops are equally safe, since the body can neither grow nor shrink the
list. Refusing the `Array` form only makes the programmer write a conversion the compiler can see is the
same.', 'checklist': '2 (accepts the correct program), 4 (no line written only for the compiler), 5 (the same
rule for an Array as for its by-element walk and a MutSpan), 7 (general: every contiguous list)'}

