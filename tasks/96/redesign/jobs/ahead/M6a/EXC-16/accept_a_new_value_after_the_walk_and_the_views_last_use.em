# Clause EXC-16.1: "inside a `for` loop over the same field it is `E3020`"; "a view into the old
# value is not used after it"
#$ test: run-pass
#$ rules: EXC-16, EXC-19
#$ stdout: 1
#$ stdout: a
#$ stdout: 0 b
# The same new values after the walk and after `n`'s last use; a new value for another field
# inside the walk (`[EXC-19]`).

class Hold:
    items: Array[int] = []
    name: String

    fn init(owns name: String):
        this.name = name

    fn reset():
        this.items = []

fn main():
    h = Hold(String("a"))
    h.items.push(owns 1)
    for x in h.items:
        println(x)
        h.name = String("a")
    h.reset()
    n = h.name
    println(n)
    h.name = String("b")
    println(h.items.len(), h.name)
