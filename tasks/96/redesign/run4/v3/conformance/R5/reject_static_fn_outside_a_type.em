#$ test: compile-fail
#$ rules: R5
#$ help: fn helper(n: int) -> int:
## R5: `static fn` outside a type body is refused: such a function already has no object (E2115).

static fn helper(n: int) -> int:    #$ error[E2115]: `helper` is not in a type, so it already has no object; remove `static`: `fn helper(n: int) -> int:`
    return n + 1

fn main():
    println(helper(1))
