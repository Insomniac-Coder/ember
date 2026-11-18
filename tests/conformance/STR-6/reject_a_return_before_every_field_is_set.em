#$ test: compile-fail
#$ rules: STR-6, CLS-2
## `[STR-6]` - every field without a default is set on every path; a `return` from `init`, like the end
## of its body, hands the object to its caller, so every such field is set on that path too (`E2100`).

struct A:
    x: int
    y: int = 5

    fn init(c: bool):
        if c:
            return    #$ error[E2100]: field `x` is not definitely initialized by struct `init`
        this.x = 1

fn main():
    a = A(false)
    println(a.x, a.y)
