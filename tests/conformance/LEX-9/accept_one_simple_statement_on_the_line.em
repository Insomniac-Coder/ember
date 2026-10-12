#$ test: run-pass
#$ rules: LEX-9, CTL-9
#$ stdout: 1
#$ stdout: ok
## `[LEX-9]` — the block may be one simple statement on the `:` line;
## `[CTL-9]` — `pass` does nothing, and is how an empty block is written.

fn nothing():
    pass

fn main():
    if true: println(1)
    nothing()
    if false:
        pass
    println("ok")
