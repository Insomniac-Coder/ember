#$ test: compile-fail
#$ rules: SPN-6, BRW-1, BRW-5, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    view = values.as_mut_span()
    chunks = view.chunks_mut(1)
    first = chunks.next()
    _split = view.split_at(1) #$ error[E3022]: `view` is already mutably borrowed
    match first:
        Some(chunk):
            println(chunk[0])
        None:
            pass

