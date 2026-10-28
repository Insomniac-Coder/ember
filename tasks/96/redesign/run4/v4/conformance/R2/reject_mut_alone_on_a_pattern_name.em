#$ test: compile-fail
#$ rules: R2, R3
#$ help: to change that list through `path`, write `State.Walking(ref mut path):`
## R2, R3: `mut` alone on a name says it may change but not whose list it is, so it is refused (E0113)
## with row 8's line (AN-38's message; `V4/rules.md` section 3, the `mut` row of
## `V4/vocabulary.md`).

enum State:
    Idle
    Walking(path: Array[int])

class Npc:
    state: State = State.Idle

    fn update() changes state:
        match this.state:
            State.Walking(mut path):    #$ error[E0113]: `mut path` says `path` may change but not whose list it is
                path.pop()
            State.Idle:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
