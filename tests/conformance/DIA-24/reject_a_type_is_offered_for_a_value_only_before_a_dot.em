#$ test: compile-fail
#$ rules: DIA-24
#$ error[E1010]: cannot find `Colo` in this scope
#$ not-help: `Color`
# `[DIA-24]` — a candidate is of the right kind for the position: where a value is expected, a type is
# not offered, except before a `.` (`Colr.Red`, which offers `Color`).

enum Color:
    Red
    Green

fn main():
    shade = Colo
    println(shade)
