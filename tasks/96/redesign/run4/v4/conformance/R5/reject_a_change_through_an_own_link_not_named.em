#$ test: compile-fail
#$ rules: R5
#$ help: fn main() changes Clock:
## R5, verdict 64: a change through a link that is the function's own local is named too (E2105).

class Clock:
    frame: int = 0

fn main():
    clock = shares mut Clock()
    clock.frame += 1    #$ error[E2105]: `main` changes the `Clock` its own link `clock` names (`clock.frame += 1`, line 11); write `fn main() changes Clock:`
    println(clock.frame)
