#$ test: run-pass
#$ rules: BRW-12
#$ stdout: 1
## design rules: R2
## R2: the nearest program that is allowed for the two refusals above: row 8's `ref mut path`
## (AN-38's NEW).

enum State:
    Idle
    Walking(path: Array[int])

class Npc:
    state: State = State.Idle

    fn update():
        match this.state:
            State.Walking(ref mut path):
                path.pop()
            State.Idle:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
    match n.state:
        State.Walking(path):
            println(path.len())
        State.Idle:
            println("idle")
