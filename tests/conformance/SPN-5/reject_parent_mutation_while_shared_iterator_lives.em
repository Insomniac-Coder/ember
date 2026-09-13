#$ test: compile-fail
#$ rules: SPN-5, BRW-1, BRW-2, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    view = values.as_mut_span()
    iterator = view.iter()
    view[0] = 2 #$ error[E3021]: `view` cannot be written while it is borrowed
    match iterator.next():
        Some(item):
            println(item)
        None:
            pass

