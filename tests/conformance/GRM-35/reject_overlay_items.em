#$ test: compile-fail
#$ rules: GRM-35, FFI-11
# `[GRM-35]`, `[FFI-11]` — a contract word is one of the vocabulary's.

unsafe overlay c "string.h":
    fn strlen(s: borrowed nul_terminated forever)    #$ error[E0104]: `forever` is not a contract word
    @safety("not an overlay attribute")    #$ error[E0104]: `@safety` does not apply to a overlay item
    fn strcmp(a: borrowed nul_terminated, b: borrowed nul_terminated)

fn main():
    pass
