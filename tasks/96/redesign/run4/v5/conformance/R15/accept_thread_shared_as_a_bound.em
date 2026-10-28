#$ test: run-pass
#$ rules: THR-1
#$ stdout: 2
## design rules: R15
## R15: `ThreadShared` is written only in bounds (AN-80).

[T: ThreadShared]
fn keep(xs: Array[T]) -> int:
    return xs.len()

fn main():
    println(keep([1, 2]))
