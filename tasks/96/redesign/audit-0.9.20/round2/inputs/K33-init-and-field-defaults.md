# K33-init-and-field-defaults

Question: Does a struct's init make a field's default first, even when it sets that field itself?
Severity: medium; impact rank 33
Members: J3-06; primary J3-06
Why together: Single question.
Critic's notes: B changes CLS-2 for classes too. A default with a side effect (`id: int = next_id()`) then no longer runs when init sets the field (C++'s initialiser-list behaviour), and whether it runs depends on the init's body. Say so in the draft (checklist 1, 15; MOD-9).

## Finding J3-06
Rules: STR-6, STR-2, CLS-2; file tasks/spec-0.9.9/parts/p05-declarations.md lines 253-256; kind undecided_case; severity medium
Quote from Hardened_11:
* `[STR-6]` *(new in 0.9.9)* *(changed in 0.9.15)* `fn init(…)` on a struct is its constructor; it sets
  the fields through `this`, with no word. Every field without a default MUST be set on every path
  before `this` is used as a whole (`E2100`), as `[CLS-2]` says for a class.

The finder's question:
When a struct's `init` sets a field that has a default, is the default evaluated anyway, before the body
(allocating and running its code)? May `init` read a defaulted field before setting it?

Program:
```
struct Buffer:
    data: Array[u8] = Array.with_capacity(4096)
    pos: int = 0

    fn init(size: int):
        if this.pos == 0:
            this.data = Array.with_capacity(size)

fn main():
    b = Buffer(64)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
For a class, CLS-2 (p05:314-316) evaluates and stores every default before any `init` body. For a struct,
STR-6 (p05:253-256) copies only the E2100 sentence, "as `[CLS-2]` says for a class". STR-2 (p05:230-231)
evaluates a default "at each construction that omits the field", and an `init` that sets the field does not
omit it. So for a struct `init` the spec reads both ways: whether the default is made and then dropped, and
whether `init` may read a defaulted field before setting it. Nothing else in Parts V-VIII settles it.

options:
[{'letter': 'A', 'summary': 'As a class does today: every default is made and stored before the `init` body,
for structs too.', 'code': '    b = Buffer(64)                              # makes the 4 KB list, then init
replaces it', 'outcome': 'Compiles and prints `0` twice. Each `Buffer(64)` allocates 4 KB, frees it and
allocates 64 bytes. `init` may read any defaulted field.'}, {'letter': 'B', 'summary': 'For structs and
classes alike, a default is made only for a field that `init` does not set on every path before reading it.',
'code': '    b = Buffer(64)                              # `data` is made once, at 64 bytes; `pos` gets 0
before the body', 'outcome': 'Compiles and prints `0` twice. `data` is allocated once. `pos` is not set by
`init`, so its default is stored before the body and `println(this.pos)` reads 0. The class rule changes to
match: a class field that `init` sets first is not given its default either.'}]

recommendation:
{'letter': 'B', 'reason': 'A default exists for the constructions that leave the field out, which is what
STR-2 already says. Making and freeing a value `init` will replace is wasted work that C++ avoids with its
initialiser list. The compiler already works out "set on every path before used" for E2100.', 'checklist': "1
(does what the program means: one buffer); 3 (decided when compiling, no wasted allocation, as fast as C++); 5
and 6 (one rule for structs and classes, in STR-2's own words); 7 (general)."}

