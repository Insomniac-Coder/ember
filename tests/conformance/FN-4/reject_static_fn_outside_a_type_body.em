#$ test: compile-fail
#$ rules: FN-4, GRM-41
## `[FN-4]`, `[GRM-41]` — `static fn` is a function of a type body: outside one it is `E2115`,
## whose help is to remove `static`.

static fn helper() -> int:    #$ error[E2115]: `static fn` is a function of a type body
    return 1

fn main():
    println(helper())
