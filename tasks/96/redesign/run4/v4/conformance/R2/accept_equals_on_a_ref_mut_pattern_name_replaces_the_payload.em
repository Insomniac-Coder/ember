#$ test: run-pass
#$ rules: R2
#$ stdout: 0
## R2: the nearest program that is allowed: `=` on a `ref mut` pattern name puts the new list in the
## payload's place, and the old list ends.

enum State:
    Idle
    Walking(path: Array[int])

class Npc:
    state: State = State.Idle

    fn stop() changes state:
        match this.state:
            State.Walking(ref mut path):
                path = []
            State.Idle:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.stop()
    match n.state:
        State.Walking(path):
            println(path.len())
        State.Idle:
            println("idle")
