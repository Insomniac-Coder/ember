# K62-copy-of-on-loop-names

Question: May a loop name or a match pattern name say copy of, as exists as does?
Severity: low; impact rank 62
Members: J1-10; primary J1-10
Why together: Single question.
Critic's notes: Ask after K26: for a list of views, what `for copy of w in words` gives depends on J2-01.

## Finding J1-10
Rules: GRM-42, GRM-44, GRM-13, BRW-14, CTL-1; file tasks/spec-0.9.9/parts/p03-grammar.md lines 219-220, 420-425, 497; kind inconsistency; severity medium
Quote from Hardened_11:
name_word     := "ref" ["mut"] | "copy" "of"

The finder's question:
Does `for copy of e in enemies:` or the match arm `Some(copy of v):` compile? A name after `exists as` may
take `copy of` (verdicts 66 and 67), but loop and pattern names, which GRM-42 treats as the same kind of list,
have only `ref` and `ref mut` (`primary_pattern := ... | "ref" ["mut"] IDENT`).

Program:
```
for copy of e in enemies:
    e.hp -= 10                  # a preview on a copy of each enemy
    preview.push(owns e)

match scores.get(name):
    Some(copy of v):
        v += 1
    None:
        pass
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.6)
Reason:
`copy of` is allowed on a name after `exists as` (name_word, p03 220; BRW-14 p07 228-229, 'The one exception
is a name opened `exists as copy of v`'), and on captures (p03 330). Loop names and pattern names, which
GRM-42 (p03 419-421) treats as the same kind of list, have only `ref`/`ref mut` (`primary_pattern`, p03 497).
No reason is given. The match form of the same map lookup (`Some(copy of v):`) cannot say what the `exists as`
form says. Verdicts 66 and 67 decided `exists as copy of v` only, and nothing settles loops or patterns.
Lambda parameters stay out, since FN-2 (p05 112) says 'There is no by-copy mode' for parameters.

options:
[{'letter': 'a', 'summary': "`copy of` on loop names and pattern names too, each a changeable copy of the
program's own", 'code': 'for copy of e in enemies:\n    e.hp -= dmg\n    preview.push(owns e)\n\nmatch
scores.get(name):\n    Some(copy of v):\n        v += 1                     # the map is untouched\n
None:\n        pass', 'outcome': "Both compile, and the program prints 30 20. Each `copy of` name is a copy
made by the type's `Clone`, may be changed, and its cost is shown where it is written. Lambda and function
parameters still have no copy mode."}, {'letter': 'b', 'summary': 'Keep it as today: only `exists as` takes
`copy of`', 'code': 'for e in enemies:\n    mine = copy of e\n    mine.hp -= dmg\n    preview.push(owns
mine)', 'outcome': '`for copy of e in enemies:` and `Some(copy of v):` are E0100, whose help gives the
two-line form.'}]

recommendation:
{'letter': 'a', 'reason': 'The same idea, a named copy of the item being looked at, then has one spelling
wherever the language declares names without types. The copy is still written on the line, so nothing is
copied silently.', 'checklist': '5, 11 (the same idea spelled one way), 16 (an existing spelling reaching
every place the same thing happens), 4'}

