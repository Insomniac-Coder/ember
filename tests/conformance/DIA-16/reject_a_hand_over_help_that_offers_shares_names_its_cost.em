#$ test: compile-fail
#$ rules: DIA-16
# `[DIA-16]` — the help of a hand-over error that offers `moved = shares values` as one fix names its cost too.

fn main():
    values = [1, 2]
    moved = values
    println(values.len())   #$ error[E3040]: a `shares` or `shares mut` owner costs, while the program runs, one count per owner made or ended, and one compare with the count's maximum per owner made
    println(moved.len())
