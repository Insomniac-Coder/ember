# K08-export-arguments-from-c

Question: What may an exported function trust about the arguments its C host passes?
Severity: high; impact rank 8
Members: J8-09; primary J8-09
Why together: Single question.
Critic's notes: A NULL read, or a call through any address, in safe code. A reuses FFI-56 and E5054's reason.

## Finding J8-09
Rules: FFI-31b, FFI-56, FFI-26, FFI-21a; file tasks/spec-0.9.9/parts/p16-ffi.md lines 590-598, 647-656; kind contradiction; severity high
Quote from Hardened_11:
(`[BLD-FFI-5a]`). `@layout(c)` values, scalars, opaque handles, `CStrView` and `extern "C" fn` may.

The finder's question:
May an exported function take a non-null `CStrView` or a safe `extern "C" fn` parameter that C supplies, and
then use it in safe code? FFI-56 makes every function pointer that comes from C ("an argument C passes to a
callback") an `unsafe extern "C" fn`, and E5054 refuses `char` because C may pass a value no `char` holds.

Program:
```
# SAFETY: the host defines no other symbol named `game_log`
@unsafe(export("game_log"))
pub extern "C" fn game_log(msg: CStrView, done: extern "C" fn(i32)) -> i32:
    println(msg.to_str())   # host passes NULL; CStrView is non-null (assembled 1365)
    done(0)                 # host passes any address; a safe call
    return 0
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.55)
Reason:
FFI-31b (p16-ffi.md 651) lets an exported signature carry `CStrView` and `extern "C" fn`. CStrView has a
non-null niche (ODR-080's `Option[CStrView]` is how null is said). The `@unsafe(export)` `# SAFETY:` note
vouches only for the symbol name (FFI-26, 629-634). FFI-56 (590-596) makes every function pointer "that comes
from C ... an argument C passes to a callback" an `unsafe extern "C" fn`, because "nothing tells the compiler
what the function it points to does". An export's parameter comes from C in exactly that way. E5054 (652-656)
refuses `char` because "C may pass any `uint32_t`", and the same reason applies to a null `const char*`. No
rule or verdict says how an export's C-supplied parameters are typed.

options:
[{'letter': 'A', 'summary': 'What C passes to an export follows the rules for what C hands back: a function
pointer is `unsafe extern "C" fn`, called in `unsafe:`, and a C string is `Option[CStrView]`, where NULL is
`None`.', 'code': '@unsafe(export("game_log"))\npub extern "C" fn game_log(msg: Option[CStrView], done: unsafe
extern "C" fn(i32)) -> i32:\n    if msg exists as m:\n        match m.to_str():\n            Ok(text):\n
println(text)\n                # SAFETY: the host\'s `done` takes one int and runs no Ember code\n
unsafe:\n                    done(1)\n            Err(_):\n                return 1\n    return 0', 'outcome':
'This compiles. The example as written is refused: `done: extern "C" fn(i32)` and `msg: CStrView` each get the
error giving these types. A NULL from the host becomes `None`, not a crash.'}, {'letter': 'B', 'summary': "As
today. The types the export writes are the host's promise, and the `# SAFETY:` note above
`@unsafe(export(...))` vouches for every argument as well as the symbol name.", 'code': '# SAFETY: the host
defines no other symbol named `game_log`, passes a NUL-terminated string and a valid
`done`\n@unsafe(export("game_log"))\npub extern "C" fn game_log(msg: CStrView, done: extern "C" fn(i32)) ->
i32:', 'outcome': 'The example compiles. A host that passes NULL makes Ember read address 0.'}]

recommendation:
{'letter': 'A', 'reason': "A value arriving from C gets the same type whichever way it arrives (a callback's
argument, a struct field, an export's parameter), and a value C may send that the Ember type cannot hold is
refused, as `char` already is. The NULL test is one compare.", 'checklist': '5 and 6 (one rule with FFI-56 and
E5054), 8, 3 (one compare), 11 (`Option[CStrView]` and `unsafe extern "C" fn` reused)'}

