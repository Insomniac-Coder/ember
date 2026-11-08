# K29-text-view-in-list

Question: Can a text view be looked for in a list of Strings with in?
Severity: medium; impact rank 29
Members: J6-13; primary J6-13
Why together: Single question.
Critic's notes: 

## Finding J6-13
Rules: STD-8, STD-12, TXT-9; file tasks/spec-0.9.9/parts/p15-stdlib.md lines 276-279; kind inconsistency; severity medium
Quote from Hardened_11:
* `[STD-8]` *(changed in 0.9.15)* **`Contains`.** `x in c` requires `c: Contains[typeof(x)]` (`E2226`
otherwise, whose help
  is `c.iter().any(owns fn(e) => e == x)`). `std` implements it for `Set` and `Map` (by key); for `Array`,
  `Span`, `MutSpan` and `[T; N]` with `T: needs Eq`, as a linear scan;

The finder's question:
Does `who.trim() in names` compile for `names: Array[String]` (and `"bob" in names`)? If so, does the literal
allocate a String for the test?

Program:
```
fn main():
    names: Array[String] = ["ana", "bob"]
    who = input("name? ")
    if who.trim() in names:        # StrView in Array[String]
        println("welcome back")
    if "bob" in names:             # literal: StrView, or a new String (TXT-9)?
        println("bob is here")
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
STD-8 (p15 276-279) gives Array, Span, MutSpan and [T; N] only `Contains[T]` for `T: needs Eq`, while Set and
Map take any `Q: needs AsKey[K]`, so a StrView finds a String key (STD-12, p15 253-263; STD-16 p15 222-224).
So `who.trim() in names` is E2226 for a list of Strings and fine for a set, with no reason given. For a
literal, TXT-9 (p15 131-136) makes it a String only where a String is expected, and `x in c` (`c:
Contains[typeof(x)]`) does not say whether it expects `T`: E2226 or one allocation. No verdict.

options:
[{'letter': 'A', 'summary': 'As for sets and maps: a list of Strings is searched with a text view', 'code':
'return who in names\nif "trudy" in banned:', 'outcome': 'Both compile; each element is compared with the
text, and the literal stays a view, so nothing is allocated (allowed under `@noalloc`). Lists, views and fixed
arrays take the same needles a set or map of that element type takes.'}, {'letter': 'B', 'summary': 'As today:
only a String is looked for in a list of Strings', 'code': 'return who in names', 'outcome': 'E2226, help
`names.iter().any(owns fn(e) => e == who)`; `"trudy" in banned` makes a new String for the test (one
allocation, refused under `@noalloc`).'}]

recommendation:
{'letter': 'A', 'reason': '`x in c` then means the same for a list and a set of the same element type, and the
obvious line compiles without making a String.', 'checklist': "2, 5, 6 (STD-12's view lookup), 7, 11"}

