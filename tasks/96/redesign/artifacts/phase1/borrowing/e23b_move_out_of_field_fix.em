# A23 workaround: mem.replace with a placeholder.
enum State:
    Idle
    Walking(path: Array[int])
    Dead

struct Npc:
    state: State

    fn update(mut self):
        old = mem.replace(self.state, State.Idle)
        match owned old:
            State.Walking(path):
                path.pop()
                if not path.is_empty():
                    self.state = State.Walking(path)
            other:
                self.state = other

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
