# K49-ser-skip-field-value

Question: What does a @ser(skip) field hold after a save file is read back?
Severity: medium; impact rank 49
Members: J6-15; primary J6-15
Why together: Single question.
Critic's notes: 

## Finding J6-15
Rules: SER-1, SER-2, DRV-1; file tasks/spec-0.9.9/parts/p14-comptime.md lines 138, 144-153; kind undecided_case; severity medium
Quote from Hardened_11:
A derive whose requirement a field does not meet is `E2080`, naming the field.

The finder's question:
Does `@derive(Serialize, Deserialize)` compile for a struct with a `shares`, `weakref`, `sharedref mut`, view
or `owns dyn I` field? What does a `@ser(skip)` field hold after `deserialize`?

Program:
```
class Player:
    name: String

@derive(Serialize, Deserialize)
struct SaveGame:
    level: int
    captain: shares Player
    scout: weakref Player        # what is written; what does reading give?
    pools: Array[owns dyn Pool]  # which concrete type does reading make?
    @ser(skip)
    cache: Map[String, int]      # value after deserialize: Default? E2080 without one?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
Narrowed. What fields a derive needs is settled in practice: the derive writes ordinary code calling each
field's serialize/deserialize (DRV-1, p14 138-139), std implements Serialize/Deserialize for no link type
(std/src/ser/mod.em 200-330), and 'a derive whose requirement a field does not meet is E2080, naming the
field' (p14 138). XIV.4 should still say it (see notes). Open: what a `@ser(skip)` field holds after reading
back. SER-2 (p14 151-153) and the XIV.2 text (p14 116-117) say only that a field missing from the data takes
its type's Default under `@ser(default)` or `@ser(since=N)`, and that a missing field without them is an
error; a skipped field is always missing, and nothing says what it takes or whether one with no default
compiles. No verdict.

options:
[{'letter': 'A', 'summary': 'A skipped field takes what `@derive(Default)` would give it', 'code':
'@ser(skip)\nframes_drawn: int', 'outcome': 'Compiles; prints `3 0`: `path_cache` is `{}` (its declared
default) and `frames_drawn` is 0 (int\'s Default). A skipped field with neither (`captain: shares Player`) is
E2080 naming the field: "a skipped field is not in the data, so reading it back needs a value: give it a
default".'}, {'letter': 'B', 'summary': 'A skipped field must write its default', 'code':
'@ser(skip)\nframes_drawn: int = 0', 'outcome': '`frames_drawn: int` with no `= ...` is E2080 with the line
above as its help; with it, prints `3 0`.'}]

recommendation:
{'letter': 'A', 'reason': "It reuses the rule the Default derive already has (the field's declared default,
else its type's), so a field's value after reading needs no second rule, and only a field that truly has no
value is refused, with a line that compiles.", 'checklist': "2, 5, 6 (DRV-1's Default row), 9, 11"}

