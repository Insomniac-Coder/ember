#$ test: compile-fail
#$ rules: LT-1a, BRW-8
#$ help: return `StrView` instead of `ref StrView`
# Whether or not `@borrows` names `x`, `ref x` points at `x`'s
# own copy, which ends with the call whatever `@borrows` says (`[BRW-8]`).

fn pick(a: StrView, x: StrView) -> ref StrView from a:
    return ref x    #$ error[E3060]: `x` does not live long enough

fn main():
    println(pick("a", "b"))
