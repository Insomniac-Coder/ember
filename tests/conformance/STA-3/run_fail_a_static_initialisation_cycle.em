#$ test: run-fail
#$ rules: STA-3
#$ panics: static initialisation cycle: A -> B -> A
#$ stdout: before
## `[STA-3]` — an initialiser that reaches its own static is a panic naming
## the chain.

fn a() -> int:
    return B + 1

static A: int = a()
static B: int = A + 1

fn main():
    println("before")
    println(A)
