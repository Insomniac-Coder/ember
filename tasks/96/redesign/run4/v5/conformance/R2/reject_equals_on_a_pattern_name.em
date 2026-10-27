#$ test: compile-fail
#$ rules: R2
#$ help: to replace that list write `State.Walking(ref mut path):`
## R2: a pattern name stands for what the `match` gave and keeps it for the whole branch: `=` on it is
## refused (E3023), unless it is `ref mut`.

enum State:
    Idle
    Walking(path: Array[int])

class Npc:
    state: State = State.Idle

    fn stop():
        match this.state:
            State.Walking(path):
                path = []      #$ error[E3023]: `path` is the list `this.state` holds and stays it for the whole branch
            State.Idle:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.stop()
