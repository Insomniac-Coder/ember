# A23: the Python-shaped state machine: pop in place, and on one path carry the path into the next state.
enum State:
    Idle
    Walking(path: Array[int])
    Running(path: Array[int])

struct Npc:
    state: State

    fn update(mut self, hurry: bool):
        match self.state:
            State.Walking(ref mut path):
                path.pop()
                if path.is_empty():
                    self.state = State.Idle
                elif hurry:
                    self.state = State.Running(path)

            _:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update(true)
