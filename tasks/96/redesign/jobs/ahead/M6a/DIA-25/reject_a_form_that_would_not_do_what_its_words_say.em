# Clause DIA-25.2: "A form that would compile but would not do what its words say is refused, with
# the line that does (`State.Walking(mut path):`, which says "may change" but not whose list, is
# `E0113`: "to change that list through `path`, write `State.Walking(ref mut path):`")."
#$ test: compile-fail
#$ rules: DIA-25, GRM-13, BRW-14
# The spec's message.

enum State:
    Idle
    Walking(Array[int])

struct Walker:
    state: State

fn step(ref mut n: Walker):
    match n.state:
        State.Walking(mut path):    #$ error[E0113]: to change that list through `path`, write `State.Walking(ref mut path):`
            path.pop()
        State.Idle:
            pass

fn main():
    w = Walker(State.Walking([1, 2]))
    step(ref mut w)
