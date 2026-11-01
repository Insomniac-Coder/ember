#$ test: run-pass
#$ rules: CTL-2, BRW-2
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    for value in values.as_span().iter():
        println(value)
    values.push(owns 3)
    println(values[2])
