#$ test: run-pass
#$ rules: FN-4
#$ stdout: 1
## design rules: R5
## R5, verdict 86: `tick` changes `FRAME`, a module variable, and `main` calls it; neither first line
## says so: the compiler works it out from the bodies.

static mut FRAME: int = 0

fn tick():
    FRAME += 1

fn main():
    tick()
    println(FRAME)
