#$ test: compile-fail
#$ rules: GRM-28
## `[GRM-28]` — a local function is in scope from its declaration on.

fn main():
    println(later())    #$ error[E1010]: cannot find `later` in this scope
    fn later() -> int:
        return 1
