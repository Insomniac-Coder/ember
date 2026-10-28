#$ test: compile-fail
#$ rules: R4
#$ help: show(clock)
## R4: `ref mut` written before an argument whose parameter is not `ref mut` is refused, so the call
## never says a change the callee cannot make (AN-20).

struct Clock:
    frame: int = 0

fn show(s: Clock):
    println(s.frame)

fn main():
    clock = Clock()
    show(ref mut clock)    #$ error[E3028]: `show` only reads its argument; remove `ref mut`: `show(clock)`
