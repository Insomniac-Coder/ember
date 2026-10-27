#$ test: run-pass
#$ rules: R5
#$ stdout: 1
## R5: `tick` and `main` both name `FRAME`, the module variable `tick` changes.

static mut FRAME: int = 0

fn tick() changes FRAME:
    FRAME += 1

fn main() changes FRAME:
    tick()
    println(FRAME)
