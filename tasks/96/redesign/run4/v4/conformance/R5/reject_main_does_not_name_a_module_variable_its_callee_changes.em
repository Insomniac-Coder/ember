#$ test: compile-fail
#$ rules: R5
#$ help: fn main() changes FRAME:
## R5: `changes NAME` names a module variable a function changes, and a function that calls one that
## changes it names it too, `main` included (E2105).

static mut FRAME: int = 0

fn tick() changes FRAME:
    FRAME += 1

fn main():
    tick()    #$ error[E2105]: `main` calls `tick`, which changes `FRAME`; add it to `main`'s first line: `fn main() changes FRAME:`
    println(FRAME)
