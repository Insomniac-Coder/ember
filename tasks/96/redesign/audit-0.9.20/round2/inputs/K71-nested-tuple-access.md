# K71-nested-tuple-access

Question: Does t.0.1, element 1 of a nested tuple's element 0, compile?
Severity: low; impact rank 71
Members: J1-15; primary J1-15
Why together: Single question.
Critic's notes: 

## Finding J1-15
Rules: LEX-18, LEX-21, GRM postfix "." INT; file tasks/spec-0.9.9/parts/p03-grammar.md lines 352 (p03); 126, 153 (p02); kind undecided_case; severity low
Quote from Hardened_11:
               | "[" expr_list "]" | "." IDENT | "." INT

The finder's question:
Does `edge.0.1` (element 1 of element 0 of a nested tuple) parse? Maximal munch lexes `0.1` as one FLOAT, so
the postfix `"." INT` never sees `0` and `1`.

Program:
```
fn first_end(edges: Span[((int, int), (int, int))]) -> int:
    return edges[0].0.1
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.55)
Reason:
`float_lit := dec_lit "." dec_lit` (p02 126) and maximal munch (LEX-21, p02 188) lex `0.1` in `t.0.1` as one
FLOAT, so postfix `"." INT` (p03 352) never sees `0` and `1`. LEX-18 (p02 153) decides only '`1.` followed by
an identifier character' and `1.0`. No rule or example settles nested tuple access (grep: none). Writing
`(t.0).1` works, so this is a corner.

options:
[{'letter': 'a', 'summary': 'Right after a `.`, digits are read as whole numbers only', 'code': 'return
edges[0].0.1', 'outcome': 'Compiles and prints 5: `edges[0].0.1` is `(edges[0].0).1`. A float after a `.`
cannot be meant there, since a `.` is followed by a name or an element number.'}, {'letter': 'b', 'summary':
'Refuse it; nested access is written with brackets', 'code': 'return (edges[0].0).1', 'outcome':
'`edges[0].0.1` is E0100, whose help gives `(edges[0].0).1`.'}]

recommendation:
{'letter': 'a', 'reason': 'The grammar already lets element accesses follow one another, and the only thing
stopping the natural line is how a number is cut into tokens. A correct line should not need brackets for
that.', 'checklist': '2, 7, 9 (no message needed for a correct line), 5'}

