#$ test: run-pass
#$ rules: GRM-4, CTL-10
#$ stdout: 7
#$ stdout: 9
## `[GRM-4]` — a name declared in every arm of an exhaustive branch is hoisted to the enclosing block.

fn pick(c: bool) -> int:
    if c:
        v = 7
    else:
        v = 9
    return v

fn main():
    println(pick(true))
    println(pick(false))
