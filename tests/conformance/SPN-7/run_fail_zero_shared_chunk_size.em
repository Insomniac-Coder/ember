#$ test: run-fail
#$ rules: SPN-7, TST-25
#$ panics: index 0 is out of bounds for a length of 0

fn main():
    values: Array[i32] = Array[i32]()
    _chunks = values.as_span().chunks(0)

