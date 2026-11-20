# Clause FFI-56.1: "A function pointer that comes from C — a field of an imported struct, a typedef, a
# result (...) — imports as `unsafe extern "C" fn(…)` (...), so a call through it is in `unsafe:`
# (`E3100`"
#$ test: compile-fail
#$ rules: FFI-56, FFI-8, UNS-1
# The spec's message, with this file's line number: `release` is the result of `get_release`, whose
# overlay entry makes the call that gets it safe, and the pointer's call is still unsafe. The test
# only compiles.

import c "hooks.h" with (overlay = "overlays/hooks.em") as h

fn main():
    release = h.get_release()
    release(1)    #$ error[E3100]: `release` is a C function pointer from `hooks.h`, with no contract; put line 14 in an `unsafe:` block with a `# SAFETY:` note saying why the call is safe
