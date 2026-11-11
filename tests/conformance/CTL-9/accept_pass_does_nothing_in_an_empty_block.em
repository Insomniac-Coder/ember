#$ test: run-pass
#$ rules: CTL-9
#$ stdout: ok
# `[CTL-9]` — `pass` does nothing; an empty block is written `pass`.

fn nothing():
    pass

fn main():
    nothing()
    if true:
        pass
    println("ok")
