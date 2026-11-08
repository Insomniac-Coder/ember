# K76-callback-table-full

Question: What happens when a program registers more C callbacks than the table holds?
Severity: low; impact rank 76
Members: J8-18; primary J8-18
Why together: Single question.
Critic's notes: Its example uses K31's spelling.

## Finding J8-18
Rules: FFI-53, HND-3, FFI-21; file tasks/spec-0.9.9/parts/p16-ffi.md lines 515-520, 532-534; kind undecided_case; severity low
Quote from Hardened_11:
for a callback, is a tagged number, the callback's slot in a table that
  never moves in the low 48 bits and the slot's generation in the top 16 ... On a 32-bit target the slot is
the low 16 bits and the generation the top 16, and

The finder's question:
What does a registering call do when the callback table has no free slot (every slot live or retired)? This is
reachable on a 32-bit target with 65,536 slots. Does the safe `register_cb(...)` panic, or return an error?

Program:
```
fn main():
    tokens: Array[Retained[ForeignCallback]] = []
    for i in 0..70000:
        tokens.push(owns register_cb(fn() owns copy of i: println(i)))   # 32-bit: slot 65,537?
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.45)
Reason:
FFI-53 (p16-ffi.md 515-520, 532-533) gives the callback table 48 slot bits (16 on a 32-bit target) and retires
a slot whose generation is used up, "as `[HND-3]` retires a `Pool` slot". HND-3 (p09-memory.md 414-415) says
what a full pool does ("`insert` panics and `try_insert` returns `Err(CapacityError.Full)`"). FFI-53 says
nothing about a full callback table, and a registering `safe fn` has no error result to carry it. A 32-bit
target reaches the limit at 65,536 live callbacks. No verdict covers this.

options:
[{'letter': 'A', 'summary': "The registering call panics when no slot is free, with a message naming the line
and the table's size, as a full `Pool`'s `insert` does.", 'code': '        tokens.push(owns on_click(owns fn()
shares mut hud: hud.clicks += 1))', 'outcome': 'On a 32-bit target the program stops at the 65,537th
registration ("no callback slot is free: 65,536 callbacks C keeps are registered (line 15)"). On 64-bit it
prints 70000.'}, {'letter': 'B', 'summary': 'A registering call whose callback C keeps returns
`Result[Retained[ForeignCallback], CapacityError]`, and the program handles a full table.', 'code': '
tokens.push(owns on_click(owns fn() shares mut hud: hud.clicks += 1)?)', 'outcome': 'Every kept-callback
registration must handle the error, including on 64-bit, where the table cannot fill.'}]

recommendation:
{'letter': 'A', 'reason': 'It is the rule the language already has for a full table of generational slots. It
costs nothing per call, and it does not make every registration handle an error that 64-bit targets cannot
reach.', 'checklist': '5 and 6 (one rule with HND-3), 3, 7 (the limit is stated, not hidden)'}

