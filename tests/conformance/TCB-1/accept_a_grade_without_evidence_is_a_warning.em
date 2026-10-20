#$ test: compile-pass
#$ rules: TCB-1
#$ profiles: debug
# `[TCB-1]` — in an ordinary build a grade an overlay claims with no evidence behind it is `W5050`:
# the fact stays asserted, and the build goes on.

unsafe overlay c "zlib.h":
    @grade(instrumented)    #$ warning[W5050]: `instrumented` is claimed with no evidence
    @ffi(effects=[FFI])
    fn zlibVersion() -> borrowed nul_terminated from(static)

fn main():
    pass
