# K15-foreign-const-globals

Question: Is reading a foreign const global safe, the same way for C and C++?
Severity: medium; impact rank 15
Members: J8-07; primary J8-07
Why together: Single question.
Critic's notes: Today's C++ rule lets safe code race on a `mutable` member. A must agree with STA-1 and ODR-163.

## Finding J8-07
Rules: FFI-41, FFI-8, STA-1, FFI-40a; file tasks/spec-0.9.9/parts/p21-annex-c.md lines 178-184; p16-ffi.md 186; kind inconsistency; severity medium
Quote from Hardened_11:
(`cpp.engine.Grid.made += 1`), in `unsafe` unless it is `const` and holds no pointer, each use going

The finder's question:
Is reading a foreign constant global safe? A C `extern const int MAX` needs `@ffi(immutable)` plus
`ThreadShared` to be read outside `unsafe` (FFI-8, ODR-082/163). A C++ `extern const int MAX` is safe with no
assertion and no `ThreadShared` check (FFI-41). Is a C++ `const` object whose class has a `mutable` member,
which its `const` methods may change on another thread, safe to read?

Program:
```
// limits.h (C):     extern const int MAX_PLAYERS;
// limits.hpp (C++): namespace game { extern const int MAX_PLAYERS;
//                   struct Stats { mutable long hits; long size() const; }; extern const Stats STATS; }
import c "limits.h" as lc
import cpp "limits.hpp" with (project="engine") as lp

fn main():
    println(lp.game.MAX_PLAYERS)   # safe by FFI-41
    println(lc.MAX_PLAYERS)        # needs `unsafe` or `@ffi(immutable)` by FFI-8
    println(lp.game.STATS.hits)    # safe by FFI-41, yet C++ may change it
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
FFI-8's static row (p16-ffi.md 186): a foreign global's read "needs `unsafe` unless `@ffi(immutable)` asserts
a `const` C object and an unchanged value (ODR-082) ... a safe read also requires `T: needs ThreadShared`".
GRM-35's overlay grammar (256-258) has no item for a variable, so a global from an imported C header can never
be given `@ffi(immutable)` and is always read in `unsafe`. FFI-41 (p21-annex-c.md 181) makes a C++ namespace
or static-member variable safe to read "unless it is `const` and holds no pointer", with no assertion and no
`ThreadShared` check. It also ignores FFI-40a's own warning that `const` is no guarantee for a class with a
`mutable` member. The owner's answer behind FFI-41 (tasks/90/owner-questions.md Q4) says only that it "reads
and writes like a global", and the rule replacing ODR-163 makes pointer-holding variables `unsafe`. Neither
decides the `const` exemption.

options:
[{'letter': 'A', 'summary': 'Derived, for C and C++ alike. A global the header declares `const` reads safely
when its type is `ThreadShared` and holds no `mutable` member. Anything else is read in `unsafe`. A hand
declaration keeps `@ffi(immutable)` as its way of writing the `const` a header would show.', 'code': 'fn
main():\n    println(lp.game.MAX_PLAYERS)    # compiles\n    println(lc.MAX_PLAYERS)         # compiles: the C
header says `const`\n    println(lp.game.STATS.hits)     # unsafe-read error: `Stats` has a `mutable` member',
'outcome': 'The two `MAX_PLAYERS` reads compile with no `unsafe`. The `STATS` read outside `unsafe:` is
refused, with a message naming `mutable long hits`.'}, {'letter': 'B', 'summary': 'Asserted, for C and C++
alike. An overlay gains a line for a variable (`@ffi(immutable)` above `static MAX_PLAYERS`), and without that
line every foreign global is read in `unsafe`.', 'code': 'unsafe overlay cpp "limits.hpp":\n
@ffi(immutable)\n    static MAX_PLAYERS\n\nunsafe overlay c "limits.h":\n    @ffi(immutable)\n    static
MAX_PLAYERS', 'outcome': 'With the lines, both `MAX_PLAYERS` reads compile, and `ember tcb` lists each as an
asserted fact. Without them, both need `unsafe:`. `STATS` is safe only if someone asserts it.'}]

recommendation:
{'letter': 'A', 'reason': "`const` in a header is a fact read from the declaration, as a `const T*` already is
(FFI-2a), so C and C++ get one rule with nothing to write. The `ThreadShared` and `mutable` tests keep it as
safe as C's asserted path.", 'checklist': "5 and 6 (one rule for C and C++), 4 (the derivable fact is the
compiler's), 2, 7"}

