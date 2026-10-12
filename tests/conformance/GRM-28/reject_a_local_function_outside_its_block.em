#$ test: compile-fail
#$ rules: GRM-28
## `[GRM-28]` — a local function is in scope to the end of its block.

fn main():
    if true:
        fn inner() -> int:
            return 1
        println(inner())
    println(inner())    #$ error[E1010]: cannot find `inner` in this scope
