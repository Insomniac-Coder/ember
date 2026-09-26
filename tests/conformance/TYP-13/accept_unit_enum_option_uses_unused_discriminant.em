#$ test: run-pass
#$ rules: TYP-13, TYP-12
#$ profiles: debug, release, shipping
#$ stdout: 1 1
#$ stdout: Ready
#$ stdout: none
# A unit enum with two variants has an unused byte value for None.
enum State:
    Ready
    Waiting

fn main():
    println(mem.size_of[Option[State]](), mem.size_of[State]())
    present: Option[State] = Some(State.Ready)
    match present:
        Some(state):
            println(state)
        None:
            println("unexpected")
    absent: Option[State] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")
