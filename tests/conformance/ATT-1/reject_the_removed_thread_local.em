#$ test: compile-fail
#$ rules: ATT-1
#$ profiles: debug
#$ error[E0104]: `@thread_local` is not an attribute
# `@thread_local` was removed in 0.9.9 (Appendix H.2).

@thread_local
static counter: int = 0

fn main():
    println(counter)
