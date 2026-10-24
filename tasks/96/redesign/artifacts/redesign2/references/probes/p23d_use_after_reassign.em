# A23: the payload used after the field is reassigned (phase3-break-D1 N4's program).
enum State:
    Idle
    Walking(path: Array[int])

struct Npc:
    state: State
    log: Array[int]

    fn stop(mut self):
        match owned self.state:
            State.Walking(path):
                self.state = State.Idle
                for step in path:
                    self.log.push(step)
            other:
                self.state = other

fn main():
    n = Npc(state=State.Walking([1, 2]), log=[])
    n.stop()
