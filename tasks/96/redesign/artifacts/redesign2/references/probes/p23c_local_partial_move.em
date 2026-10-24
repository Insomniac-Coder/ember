# A23: the same move out and reassignment on an owned local struct (no reference in between).
enum State:
    Idle
    Walking(path: Array[int])
    Running(path: Array[int])

struct Npc:
    state: State

fn main():
    n = Npc(state=State.Walking([1, 2]))
    match owned n.state:
        State.Walking(path):
            n.state = State.Running(path)
        other:
            n.state = other
    println("ok")
