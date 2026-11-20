#$ test: compile-fail
#$ rules: DIA-10, DIA-25, DIA-7a, CTL-2
#$ error[E3020]: `values.push(owns 3)` (line 13) adds to `values`, which this loop walks (line 12); collect what it adds in a list of your own: write `added: Array[i32] = []` before the loop, `added.push(owns 3)` in place of line 13, and `values.extend(owns added)` after the loop
# `[DIA-10]`, `[DIA-25]` — shape B2 gives the lines to write, never a category such as "collect the
# indices first": a push collects in a list of your own (its declaration, the line in place of the
# push, as the program has it, and the `extend` after the loop).

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    for value in values:
        values.push(owns 3)
        println(value)
