#$ test: compile-fail
#$ rules: CTL-7
# `[CTL-7]` — a `defer` block cannot `return`: `E2160`.

fn leaves() -> i32:
    defer:
        return 1                    #$ error[E2160]: control flow cannot leave a `defer` block
    return 0

fn main():
    println(leaves())
