#$ test: compile-fail
#$ rules: GRM-21, CORO-10
#$ profiles: debug
## `[GRM-21]` — `gen fn` is legal wherever `fn` is, except in an `extern` block: a foreign
## function cannot be a generator.

unsafe extern "C":
    gen fn ticks() -> i32    #$ error[E5050]: cannot be generic or a generator

fn main():
    pass
