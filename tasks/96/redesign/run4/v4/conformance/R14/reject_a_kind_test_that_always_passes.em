#$ test: compile-fail
#$ rules: R14
## R14: "A test that always passes (`s` is already a `Circle`) is refused (E2036)" (AN-68).

struct Circle:
    r: int

fn total(circles: Array[Circle]) -> int:
    t = 0
    for s in circles:
        if s is Circle as c:    #$ error[E2036]: `s` is a `Circle` already, so this `if` would always run; to name it write `c = s` in place of the `if` line, and use `c` in its block, moved out one level
            t += c.r
    return t

fn main():
    println(total([Circle(r=1), Circle(r=2)]))
