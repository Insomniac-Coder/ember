#$ test: compile-fail
#$ rules: R15
## R15: "`ThreadShared` and `Send` are inferred and written only in bounds"; `Sync` names the effect
## `@nosync` forbids (AN-80).

[T: Sync]    #$ error[E1010]: `Sync` is the effect `@nosync` forbids, not a property of a type; for "threads may share it" write `T: ThreadShared`
fn keep(xs: Array[T]) -> int:
    return xs.len()

fn main():
    println(keep([1, 2]))
