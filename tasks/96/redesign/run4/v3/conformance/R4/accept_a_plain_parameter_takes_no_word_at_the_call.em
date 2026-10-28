#$ test: run-pass
#$ rules: R4
#$ stdout: 0
## R4: an argument given to a plain parameter takes no word at the call; the parameter reads the
## caller's value (AN-20's `show`).

struct Clock:
    frame: int = 0

fn show(s: Clock):
    println(s.frame)

fn main():
    clock = Clock()
    show(clock)
