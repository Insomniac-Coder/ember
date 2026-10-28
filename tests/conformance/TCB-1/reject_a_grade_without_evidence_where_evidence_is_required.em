#$ test: compile-fail
#$ rules: TCB-1
#$ flags: --require-evidence
#$ profiles: debug
# `[TCB-1]` — under `--require-evidence` (as `ember build --require-evidence` and `ember tcb
# --require` check) a grade an overlay claims with no evidence behind it is `E5050`, not a warning.

unsafe overlay c "zlib.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @grade(instrumented)    #$ error[E5050]: `instrumented` is claimed with no evidence
    @ffi(effects=[FFI])
    fn zlibVersion() -> borrowed nul_terminated from(static)

fn main():
    pass
