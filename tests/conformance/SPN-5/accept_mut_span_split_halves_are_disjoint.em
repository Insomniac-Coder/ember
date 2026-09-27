#$ test: run-pass
#$ rules: SPN-5, BRW-4, BRW-5, TST-25
#$ stdout:
#$ 11
#$ 22
#$ 33
#$ 11
#$ 22
#$ 33

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    values.push(3)
    view = values.as_mut_span()
    (left, right) = view.split_at(1)

    # Keep both mutable halves live and write through each one.
    left[0] = 11
    right[0] = 22
    right[1] = 33
    println(left[0])
    println(right[0])
    println(right[1])

    # Once the halves' last uses end, the original owner observes both writes.
    println(values[0])
    println(values[1])
    println(values[2])
