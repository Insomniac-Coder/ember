#$ test: run-pass
#$ rules: SPN-5, SPN-2, TST-25
#$ stdout:
#$ 0
#$ 10
#$ 30
#$ 10
#$ 20
#$ 30
#$ 3
#$ 0

fn main():
    values: Array[i32] = Array[i32]()
    values.push(10)
    values.push(20)
    values.push(30)
    view = values.as_span()

    at_start = view.split_at(0)
    println(at_start.0.len())
    println(at_start.1[0])
    println(at_start.1[2])

    middle = view.split_at(1)
    println(middle.0[0])
    println(middle.1[0])
    println(middle.1[1])

    at_end = view.split_at(view.len())
    println(at_end.0.len())
    println(at_end.1.len())
