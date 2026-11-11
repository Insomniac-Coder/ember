#$ test: compile-fail
#$ rules: DIA-10, DIA-25, DIA-7a, CTL-2
#$ error[E3020]: cannot mutate `values` while it is borrowed by this loop
#$ help: move line 16, `values.push(owns 3)`, below the loop
#$ help: or keep a list of your own (`added = []`), push what the loop adds to `added` inside the loop, and add its items to `values` after the loop
#$ help: or, to drop items, call `values.retain(owns fn(item) => keep(item))` after the loop
# `[DIA-10]`, `[DIA-25]` — the help of shape B2 is the line to write, never a category such as
# "collect the indices first": the call moved below the loop (its line and its text, as the program
# has them), a list of your own added after the loop, `retain` after the loop.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    for value in values:
        values.push(owns 3)
        println(value)
