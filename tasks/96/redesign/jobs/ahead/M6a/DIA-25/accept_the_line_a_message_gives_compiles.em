# Clause DIA-25.1: "The line a message gives compiles under this document, in that program: its fix
# never runs into another error."
#$ test: run-pass
#$ rules: DIA-25, GRM-13
#$ stdout: 1
#$ stdout: a
#$ stdout: 4
# The lines the E0113 and BRW-18 messages give, written in: each program compiles and does what
# the line says. (The other files' accept_the_line... and accept_the_ways_out... do the same for
# their messages; the runner applying every fix by itself is A7-14's tooling.)

enum State:
    Idle
    Walking(Array[int])

struct Walker:
    state: State

fn step(ref mut n: Walker):
    match n.state:
        State.Walking(ref mut path):
            path.pop()
        State.Idle:
            pass

fn main():
    w = Walker(State.Walking([1, 2]))
    step(ref mut w)
    match w.state:
        State.Walking(path):
            println(path.len())
        State.Idle:
            pass
    counts: Map[String, int] = Map()
    counts.insert(owns String("a"), owns 2)
    for k, ref mut v in counts:
        println(k)
        v *= 2
    println(counts["a"])
