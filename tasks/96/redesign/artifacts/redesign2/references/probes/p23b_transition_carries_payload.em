# A23: a transition that carries the payload into the next state (Python: self.state = Running(self.state.path)).
enum State:
    Idle
    Walking(path: Array[int])
    Running(path: Array[int])

struct Npc:
    state: State

    fn hurry(mut self):
        match owned self.state:
            State.Walking(path):
                self.state = State.Running(path)
            other:
                self.state = other

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.hurry()
