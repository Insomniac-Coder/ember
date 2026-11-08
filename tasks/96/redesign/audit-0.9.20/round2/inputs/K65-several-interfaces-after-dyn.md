# K65-several-interfaces-after-dyn

Question: Are several interfaces after dyn or some written in brackets, as needs are?
Severity: low; impact rank 65
Members: J1-13; primary J1-13
Why together: Single question.
Critic's notes: Show `[T: needs (Add, Copy)]` beside `owns dyn (Shape, Debug)`. After `dyn` the brackets come without `needs`, which the draft should state plainly.

## Finding J1-13
Rules: TYP-17, GRM-3, TYP-32; file tasks/spec-0.9.9/parts/p03-grammar.md lines 71-72, 171; kind inconsistency; severity low
Quote from Hardened_11:
bound_list    := type {"+" type}                        -- after `dyn` and `some` only

The finder's question:
Are several interfaces after `dyn` or `some` still joined with `+` (`owns dyn Shape + Debug`), when TYP-17
(verdicts 109 to 112) makes 'needs joined with `+`' an `E0100` everywhere else a list of needs is written?

Program:
```
fn visible(xs: Span[int]) -> some Iterator[Item = int] + Clone:
    return xs.iter().copy_each().filter(owns fn(x) => x > 0)

shapes: Array[owns dyn Shape + Debug] = []
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.55)
Reason:
`bound_list := type {"+" type} -- after dyn and some only` (p03 72, 171) keeps `+`. TYP-17 (p04 460-461) makes
'needs joined with `+`' E0100 in type-parameter lines, `extend`, interface parents, associated types and
`where`, following verdicts 109 to 112 ('The same spelling goes everywhere a list of needs is written'). The
owner was asked about each of those places separately, and `dyn`/`some` was never put to him. OWNER-DIRECTION
lists '`+` between bounds (109)' among the rules written without a verdict. No example in the specification
joins two interfaces after `dyn` or `some` (grep). So it is the owner's decision, and the verdict must not be
widened to cover it.

options:
[{'letter': 'a', 'summary': 'Round brackets, as after `needs`; `+` refused there too', 'code': 'shapes:
Array[owns dyn (Shape, Debug)] = [owns Square(side=2.0)]\n\nfn visible(xs: Span[int]) -> some (Iterator[Item =
int], Clone):\n    return xs.iter().copy_each().filter(owns fn(x) => x > 0)', 'outcome': 'Compiles. `owns dyn
Shape + Debug` is E0100, whose message gives `owns dyn (Shape, Debug)`. One interface needs no brackets: `owns
dyn Shape`.'}, {'letter': 'b', 'summary': 'Keep `+` after `dyn` and `some`, with the reason written in the
specification', 'code': 'shapes: Array[owns dyn Shape + Debug] = [owns Square(side=2.0)]', 'outcome':
'Compiles as today. Two spellings for a list of interfaces stay in the language.'}]

recommendation:
{'letter': 'a', 'reason': 'A list of interfaces a value must have already has one spelling (round brackets,
commas), and `+` was removed everywhere it was put to the owner. Keeping it only here is the second spelling
that consistency removes.', 'checklist': '11 (the same idea spelled the same way, the older spelling goes),
16, 5'}

