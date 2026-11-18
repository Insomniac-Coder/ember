#$ test: run-pass
#$ rules: STR-6, STR-2
#$ stdout: 1 5 3
## `[STR-6]` - `fn init(...)` is the constructor: it sets the fields through `this` with no word; every
## default is stored before its body; a field set on each branch, and on an early `return`, is set.

struct A:
    x: int
    y: int = 5
    z: int

    fn init(c: bool):
        if c:
            this.x = 1
            this.z = 3
            return
        this.x = 2
        this.z = 4

fn main():
    a = A(true)
    println(a.x, a.y, a.z)
