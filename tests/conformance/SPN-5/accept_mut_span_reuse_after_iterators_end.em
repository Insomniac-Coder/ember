#$ test: run-pass
#$ rules: SPN-5, BRW-2, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(3)
    values.push(4)
    view = values.as_mut_span()

    for item in view.iter():
        println(item)

    for item in view.iter_mut():
        replacement: i32 = item + 10
        item = ref mut replacement

    view[0] = view[0] + 100
    println(view[0])
    println(view[1])
#$ stdout: 3
#$ stdout: 4
#$ stdout: 113
#$ stdout: 14

