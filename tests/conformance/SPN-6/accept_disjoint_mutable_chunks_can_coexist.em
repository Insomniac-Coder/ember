#$ test: run-pass
#$ rules: SPN-6, BRW-5, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    values.push(3)
    values.push(4)
    values.push(5)
    view = values.as_mut_span()
    chunks = view.chunks_mut(2)
    first = chunks.next()
    second = chunks.next()
    third = chunks.next()
    match first:
        Some(a):
            a[0] = 10
            println(a.len())
        None:
            pass
    match second:
        Some(b):
            b[0] = 30
            println(b.len())
        None:
            pass
    match third:
        Some(c):
            c[0] = 50
            println(c.len())
        None:
            pass
    println(view[0])
    println(view[2])
    println(view[4])
#$ stdout: 2
#$ stdout: 2
#$ stdout: 1
#$ stdout: 10
#$ stdout: 30
#$ stdout: 50

