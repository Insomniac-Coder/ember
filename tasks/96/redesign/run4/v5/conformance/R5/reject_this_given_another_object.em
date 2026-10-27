#$ test: compile-fail
#$ rules: R5
#$ help: this.v = 0
## R5: `this` in a class method is the object the method was called on; it is never given another
## object, whatever else the method may change (AN-50, E2103).

class Node:
    v: int = 0

    fn reset():
        this = Node()    #$ error[E2103]: `this = Node()` cannot replace the `Node` object `reset` was called on; set its fields instead: `this.v = 0`

fn main():
    n = Node(v=3)
    n.reset()
    println(n.v)
