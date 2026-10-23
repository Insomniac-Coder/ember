# A23: replace a field with a value computed from its old value (state machine).
enum State:
    Idle
    Walking(path: Array[int])
    Dead

struct Npc:
    state: State

    fn update(mut self):
        match owned self.state:
            State.Walking(path):
                path.pop()
                if path.is_empty():
                    self.state = State.Idle
                else:
                    self.state = State.Walking(path)
            other:
                self.state = other

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
