#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: to change that list through `path`, write `State.Walking(ref mut path):`
## design rules: R2, R5
## R2, R5: a plain pattern name is a read-only view of the payload; `path.pop()` through it is refused
## (E3023) with row 8's line (AN-38).

enum State:
    Idle
    Walking(path: Array[int])

class Npc:
    state: State = State.Idle

    fn update():
        match this.state:
            State.Walking(path):
                path.pop()     #$ error[E3023]: `path.pop()` changes the walk list in `this.state`, but `path` only reads it
            State.Idle:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
