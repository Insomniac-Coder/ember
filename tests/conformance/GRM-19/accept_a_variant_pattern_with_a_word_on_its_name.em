#$ test: run-pass
#$ rules: GRM-19, GRM-13
#$ stdout: 7
#$ stdout: 2
#$ stdout: 3
## `[GRM-19]` — the pattern form tests a variant other than an `Option`'s (`if Ok(v) = r:`,
## `if State.Walking(ref mut path) = n.state:`), `ref mut` giving the name that may change the part.

enum State:
    Walking(Array[int])
    Idle

struct Node:
    state: State = State.Walking([1])

fn main():
    r: Result[int, String] = Ok(7)
    if Ok(v) = r:
        println(v)
    n = Node()
    if State.Walking(ref mut path) = n.state:
        path[0] = 2
    if State.Walking(ref kept) = n.state:
        println(kept[0])
    if State.Idle = n.state:
        println(0)
    else:
        println(3)
