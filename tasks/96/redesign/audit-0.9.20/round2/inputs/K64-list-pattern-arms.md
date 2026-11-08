# K64-list-pattern-arms

Question: May a match arm's pattern be a list pattern, a line starting with [?
Severity: low; impact rank 64
Members: J1-05; primary J1-05
Why together: Single question.
Critic's notes: The draft must quote verdict 155 ('Every line that starts with `[` and is not directly above a function, struct or class is `E0100`') and say that A narrows it (verdict clash). A also needs the meaning of list patterns written (names are views, `..rest` a Span), and the grammar cannot yet produce `[..rest]`.

## Finding J1-05
Rules: GRM-40, GRM-10, CTL-5; file tasks/spec-0.9.9/parts/p03-grammar.md lines 135-137, 496; kind contradiction; severity medium
Quote from Hardened_11:
`E0100`: a blank line or a comment line between `[T]` and its `fn` is `E0100` too, and so is every
  line that begins with `[` anywhere else, one that begins with a list included (`[10, 20].len()` on a

The finder's question:
Does a `match` arm whose pattern is a list pattern compile? Its line begins with `[` and the next line is not
a function, struct or class, so GRM-40 (verdict 155) makes it `E0100`, while `primary_pattern` produces `[p,
q, ..rest]`.

Program:
```
fn parse_cmd(words: Span[StrView]) -> Command:
    return match words:
        ["quit"] => Command.Quit
        ["go", dir] => Command.Go(String(dir))
        _ => Command.Unknown
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.75)
Reason:
The pattern grammar produces list patterns (p03 496: `"[" [pattern {"," pattern}] ["," ".." [IDENT]] "]"`),
and only a match arm can begin a line with one. GRM-40 (p03 135-137) and verdict 155 make 'every line that
begins with `[` anywhere else' E0100. The verdict was put to the owner with a stray expression line (`[10,
20].len()`) and a bracket above an enum, not with a match arm. No rule anywhere gives list patterns a meaning
(grep: no slice or list pattern in any rule or example), and nothing says what `..rest` binds. Taken
literally, verdict 155 refuses every top-level list-pattern arm, so whether such a `match` compiles is the
owner's call. The verdict cannot be widened or narrowed without him.

options:
[{'letter': 'a', 'summary': "Inside a match, a line starting with [ is an arm's pattern", 'code': 'match
words:\n    ["quit"]:\n        return Command.Quit\n    ["go", dir]:\n        return Command.Go(String(dir))\n
["say", ..rest]:                 # `rest` looks at the words after "say"\n        return Command.Unknown\n
_:\n        return Command.Unknown', 'outcome': 'Compiles and prints Go("north"). The rule for stray `[` lines
still covers every line where an item or a statement begins (`[10, 20].len()` on its own line stays E0100). A
list pattern\'s names are read-only views like other pattern names, and `..rest` names a Span of the remaining
elements.'}, {'letter': 'b', 'summary': 'Keep the rule as written: list patterns only inside another pattern',
'code': 'match words.len():\n    1:\n        if words[0] == "quit":\n            return Command.Quit\n
return Command.Unknown\n    2:\n        if words[0] == "go":\n            return
Command.Go(String(words[1]))\n        return Command.Unknown\n    _:\n        return Command.Unknown',
'outcome': '`["quit"]:` as an arm is E0100 (\'a `[...]` line gives type parameters ...\'). A list pattern may
still be written nested, as in `Some(["quit"]):`.'}, {'letter': 'c', 'summary': 'Remove list patterns from the
language', 'code': 'match words.len():\n    1:\n        if words[0] == "quit":\n            return
Command.Quit\n        return Command.Unknown\n    _:\n        return Command.Unknown', 'outcome': 'A `[` in
pattern position is E0100 everywhere, nested included, and the grammar loses the list pattern.'}]

recommendation:
{'letter': 'a', 'reason': "The rule's purpose holds: a stray `[` line where an item or statement starts gets
one clear message. But no type-parameter line or statement can stand in a match's arm list, so refusing a
correct parser there is a side effect nobody chose. The list pattern's meaning must then be written: names are
views by the pattern default, and `..rest` is a Span.", 'checklist': '1, 2 (correct program accepted), 7, 13
(no hidden refusal from an unrelated rule), 6'}

