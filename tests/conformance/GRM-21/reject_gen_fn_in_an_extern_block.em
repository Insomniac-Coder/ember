#$ test: compile-fail
#$ rules: GRM-21, CORO-10
#$ profiles: debug
## `[GRM-21]` — `gen fn` is legal wherever `fn` is, except in an `extern` block: a foreign
## function cannot be a generator.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    gen fn ticks() -> i32    #$ error[E0104]: cannot be generic or a generator

fn main():
    pass
