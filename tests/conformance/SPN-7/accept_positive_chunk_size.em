#$ test: run-pass
#$ rules: SPN-7, SPN-6, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    shared = values.as_span().chunks(1)
    match shared.next():
        Some(chunk):
            println(chunk.len())
        None:
            pass
    mutable = values.as_mut_span().chunks_mut(1)
    match mutable.next():
        Some(chunk):
            println(chunk.len())
        None:
            pass
#$ stdout: 1
#$ stdout: 1
