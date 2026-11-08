# K56-unsafe-at-compile-time

Question: Can code with an unsafe block run at compile time?
Severity: medium; impact rank 56
Members: J6-04; primary J6-04
Why together: Single question.
Critic's notes: 

## Finding J6-04
Rules: CT-1, CT-2, EFF-1; file tasks/spec-0.9.9/parts/p14-comptime.md lines 32-36, 43-47; kind contradiction; severity medium
Quote from Hardened_11:
contains nothing beyond `Alloc`, `Panic(Explicit)` and `RuntimeCheck` — no `Io`, `FFI`, `Sync`,
  `Block`, `Unsafe` or `Nondet`.

The finder's question:
Can a `const` initialiser call a function whose `unsafe:` block writes only into the evaluator's heap?
`Unsafe` passes up the call graph like every effect (EFF-1). Does that make every std function with an
internal `unsafe:` block (for example `AnyError.downcast_ref`, std/src/core.em:141) unusable at compile time?

Program:
```
fn crc_table() -> [u32; 256]:
    t: [u32; 256] = [0; 256]
    unsafe:
        p = t.as_mut_ptr()          # a pointer into the evaluator's heap
        for i in 0..256:
            p.add(i).write(crc_of(i as u32))
    return t

const CRC: [u32; 256] = comptime(crc_table())
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
CT-1 (p14 32-36) lets compile time call only functions whose effect set holds nothing beyond Alloc,
Panic(Explicit) and RuntimeCheck, so no Unsafe. The Unsafe effect is 'the body contains unsafe' (p10 18) and
EFF-1 (p10 22-29) passes every effect up from callees, so a library function's private `unsafe:` block bars
every caller. CT-2 (p14 43-47) instead makes only 'unsafe pointer access outside the evaluator's heap' E6010,
and ODR-145 lets raw pointers be compile-time values, which implies unsafe access inside the evaluator's heap
runs. The two rules decide the same program differently. No verdict.

options:
[{'letter': 'A', 'summary': 'Unsafe code runs at compile time; each pointer access is checked as it runs',
'code': 'const CRC: [u32; 256] = comptime(crc_table())', 'outcome': "Compiles: the table is built while
compiling. A pointer access outside the compiler's own memory (an address made from a number, a C pointer) is
E6010 naming the chain of calls. `Unsafe` leaves CT-1's list; a library function with a private `unsafe:`
block can be called at compile time."}, {'letter': 'B', 'summary': 'Unsafe code never runs at compile time',
'code': 'const CRC: [u32; 256] = comptime(crc_table())', 'outcome': 'E6010: "`crc_table` runs `unsafe` code
(line 9), which compile time does not run; write the entries by index: `t[i] = crc_of(i as u32)`". Every
function that reaches an `unsafe:` block, a library\'s own included, is refused at compile time; CT-2\'s
\'outside the evaluator\'s heap\' goes.'}]

recommendation:
{'letter': 'A', 'reason': "The compiler's evaluator sees every pointer it runs, so it can refuse a bad access
instead of refusing all unsafe code; this accepts correct programs, keeps the unsafe block's writer the one
who vouches, and stops a library's private unsafe code from silently barring its callers.", 'checklist': "2,
3, 6 (CT-2's per-operation rule kept), 7, 13 (no hidden bar from a library's internals), 15"}

