#$ test: run-pass
#$ rules: SPN-5, BRW-5, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    view = values.as_mut_span()
    iterator = view.iter_mut()
    first = iterator.next()
    second = iterator.next()
    match owned first:
        Some(a):
            replacement: i32 = 10
            a = ref mut replacement
        None:
            pass
    match owned second:
        Some(b):
            replacement: i32 = 20
            b = ref mut replacement
        None:
            pass
    println(view[0])
    println(view[1])
#$ stdout: 10
#$ stdout: 20
