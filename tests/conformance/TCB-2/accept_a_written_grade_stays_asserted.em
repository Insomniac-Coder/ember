#$ test: compile-pass
#$ rules: TCB-2, TCB-1, GRM-35
#$ warning[W5050]: `instrumented` is claimed with no evidence: the fact stays asserted
#$ warning[W5050]: `proven` is claimed with no evidence: the fact stays asserted
# `[TCB-2]` — writing a grade never raises it: a fact an overlay states is
# asserted, and `@grade(…)` above asserted is a claim that evidence must back
# (`[TCB-1]`); with none recorded the claim is `W5050` and the fact stays
# asserted. `@grade(asserted)` claims nothing more.

unsafe overlay c "zlib.h":
    @grade(instrumented)
    @ffi(effects=[FFI])
    fn zlibVersion() -> borrowed nul_terminated from(static)
    @grade(proven)
    fn adler32(adler, buf: nullable borrowed count(len), len)
    @grade(asserted)
    fn crc32(crc, buf: nullable borrowed count(len), len)

fn main():
    pass
