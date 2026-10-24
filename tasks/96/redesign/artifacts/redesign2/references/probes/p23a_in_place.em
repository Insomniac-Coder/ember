# A23: the state machine written in place (match binds by reference), then the field reassigned.
enum State:
    Idle
    Walking(path: Array[int])
    Dead

struct Npc:
    state: State

    fn update(mut self):
        match self.state:
            State.Walking(path):
                path.pop()
                if path.is_empty():
                    self.state = State.Idle
            _:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
