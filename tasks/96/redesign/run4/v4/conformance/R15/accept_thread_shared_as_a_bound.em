#$ test: run-pass
#$ rules: R15
#$ stdout: 2
## R15: `ThreadShared` is written only in bounds (AN-80).

[T: ThreadShared]
fn keep(xs: Array[T]) -> int:
    return xs.len()

fn main():
    println(keep([1, 2]))
