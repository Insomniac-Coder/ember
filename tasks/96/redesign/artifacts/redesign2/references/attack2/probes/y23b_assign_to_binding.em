# Attack on A23 7.4 ("an arm's binding is `ref mut` when the arm writes through it"): an assignment to
# the whole binding. In Python `path = []` only rebinds the local name. In Ember an assignment to a
# reference writes through it ([BRW-1]), so once the binding is (inferred) `ref mut`, the same line
# empties the NPC's stored path. Written here with today's explicit `ref mut` to show the meaning.
enum State:
    Idle
    Walking(path: Array[int])

struct Npc:
    state: State

    fn peek_or_default(mut self) -> int:
        match self.state:
            State.Walking(ref mut path):
                if path.len() > 3:
                    path = [0]
                return path.len()
            _:
                return 0

fn main():
    n = Npc(state=State.Walking([1, 2, 3, 4, 5]))
    println(n.peek_or_default())
    match n.state:
        State.Walking(path):
            println(path.len())
        _:
            pass
