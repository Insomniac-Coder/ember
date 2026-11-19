#$ test: run-pass
#$ rules: CLS-4
#$ stdout: 1
# `[CLS-4]`: `virtual` is for a class that can be subclassed: `open` or `abstract`.

open class Door:
    virtual fn label() -> int:
        return 1

abstract class Shape:
    virtual fn sides() -> int:
        return 0

fn main():
    d = Door()
    println(d.label())
