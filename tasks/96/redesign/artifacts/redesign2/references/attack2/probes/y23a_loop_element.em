# Attack on A23 (answers report 7.3, rule 1: the path may not go through an index or a reference).
# The state machine written the way a game loop is written: the match sits in the loop body, on the
# loop's element (`iter_mut()`, [CTL-1]), not in a `mut self` method of Npc.
enum State:
    Idle
    Walking(path: Array[int])
    Running(path: Array[int])

struct Npc:
    state: State

struct World:
    npcs: Array[Npc]

    fn tick(mut self, hurry: bool):
        for npc in self.npcs.iter_mut():
            match npc.state:
                State.Walking(ref mut path):
                    path.pop()
                    if path.is_empty():
                        npc.state = State.Idle
                    elif hurry:
                        npc.state = State.Running(path)
                _:
                    pass

fn main():
    w = World(npcs=[Npc(state=State.Walking([1, 2]))])
    w.tick(true)
