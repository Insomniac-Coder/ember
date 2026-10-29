#$ test: compile-fail
#$ rules: GRM-35, FFI-11
# `[GRM-35]`, `[FFI-11]` — a contract word is one of the vocabulary's.

unsafe overlay c "string.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn strlen(s: borrowed nul_terminated forever)    #$ error[E0104]: `forever` is not a contract word
    @safety("not an overlay attribute")    #$ error[E0104]: `@safety` applies only to an `unsafe fn`
    fn strcmp(a: borrowed nul_terminated, b: borrowed nul_terminated)

fn main():
    pass
