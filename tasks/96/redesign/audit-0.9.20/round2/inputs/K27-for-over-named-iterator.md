# K27-for-over-named-iterator

Question: Does for t in toks: over a named iterator use it up, or continue after break?
Severity: medium; impact rank 27
Members: J3-09; primary J3-09
Why together: Single question.
Critic's notes: 

## Finding J3-09
Rules: CTL-1, FN-2a; file tasks/spec-0.9.9/parts/p06-expressions.md lines 127-134; kind undecided_case; severity medium
Quote from Hardened_11:
  * `for x in owns e:`, or a value `e` (a temporary) whose type is `IntoIterator`: hands `e` over, and
    each element to the loop's name;
  * an `Iterator` value: calls `next` until `None`.

The finder's question:
Does `for t in toks:`, where `toks` is a local Iterator (not Iterable), hand `toks` over (a later use is
E3040), or drive it in place, so that a second loop after `break` continues where the first stopped?

Program:
```
fn parse_args(line: StrView) -> (Array[String], Array[String]):
    flags: Array[String] = []
    files: Array[String] = []
    toks = line.split_whitespace()
    for t in toks:
        if t == "--":
            break
        flags.push(owns String(t))
    for t in toks:
        files.push(owns String(t))
    return flags, files
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
CTL-1 (p06:127-134) has three cases: a place whose type is `Iterable` (calls `iter()`), `owns e` or a
temporary that is `IntoIterator` (handed over), and "an `Iterator` value: calls `next` until `None`". VI.2
(p06:29-32) makes a local a place, not a value, and an `Iterator` is not `Iterable` (p04:638-645). So a local
that holds an iterator or a generator (`toks = line.split_whitespace()`, `g = countdown(3)`) falls under no
case. Nothing says whether the loop hands it over or drives it in place. STD-19's "whatever drives the adapter
(`for x in it:`, `it.sum()`)" (p15:320-321) names the form without deciding it.

options:
[{'letter': 'A', 'summary': 'A place holding an iterator is advanced in place, as calling `next` on it is, and
keeps its position after the loop. `for t in owns toks:` hands it over.', 'code': '    for t in toks:
# advances `toks`\n        if t == "--":\n            break\n        flags.push(owns String(t))\n    for t in
toks:                          # goes on from after `--`\n        files.push(owns String(t))', 'outcome':
'Compiles and prints `["-v", "-q"] ["a.txt", "b.txt"]`. An iterator reached through a name that only reads it
(a plain parameter) cannot be advanced: E3023, whose help gives `ref mut`.'}, {'letter': 'B', 'summary': 'A
loop over a place holding an iterator hands it over, so the iterator is used up.', 'code': '    while
toks.next() exists as t:          # needed to stop and go on later\n        if t == "--":\n            break\n
flags.push(owns String(t))\n    for t in owns toks:\n        files.push(owns String(t))', 'outcome': 'The
program as written is E3040 at the second `for`, because `toks` was handed over by the first. It works when
written with `while ... exists as` as above (Rust needs `by_ref()` here).'}]

recommendation:
{'letter': 'A', 'reason': 'A loop over an iterator does what calling `next` on it does, and the object a
method is called on is not marked. Handing the iterator over is still written when wanted, `for t in owns
toks:`.', 'checklist': "1 (does what the parser means, as in Python); 2 (accepts the correct program); 4 (a
hand-over only where `owns` is written); 5 and 6 (one rule with `toks.next()`); 10 (Python's reading)."}

