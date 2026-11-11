#$ test: run-pass
#$ rules: DIA-10, DIA-25, CTL-2
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
#$ stdout: 1
# `[DIA-10]` — the lines the help of shape B2 gives compile and do what they say: the call moved
# below the loop, and `retain` after it.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    for value in values:
        println(value)
    values.push(owns 3)
    println(values.len())
    values.retain(owns fn(v) => v < 2)
    println(values.len())
