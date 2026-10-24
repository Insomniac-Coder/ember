# A23: the in-place state machine with the binding written `ref mut path` (GRM-13's way to write through it).
enum State:
    Idle
    Walking(path: Array[int])
    Dead

struct Npc:
    state: State

    fn update(mut self):
        match self.state:
            State.Walking(ref mut path):
                path.pop()
                if path.is_empty():
                    self.state = State.Idle
            _:
                pass

fn main():
    n = Npc(state=State.Walking([1, 2]))
    n.update()
