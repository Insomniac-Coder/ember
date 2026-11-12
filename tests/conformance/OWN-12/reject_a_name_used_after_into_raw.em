#$ test: compile-fail
#$ rules: OWN-12, FFI-54, OWN-3
# `[FFI-54]` — `into_raw()` ends the name it is written on: Ember gave the value away, so a later
# use is a use of a name that was handed over (`E3040`).

fn main():
    given: owns i64 = owns 7
    # SAFETY: the address goes nowhere; the block is left to the end of the process
    unsafe:
        first = given.into_raw()
        again = given.into_raw()    #$ error[E3040]
        println(first == again)
