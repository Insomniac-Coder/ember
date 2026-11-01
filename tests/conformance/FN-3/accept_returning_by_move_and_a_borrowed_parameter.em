#$ test: run-pass
#$ rules: FN-3, LT-1
#$ stdout: 3 1
## `[FN-3]` — a function returns by move, and a returned reference may borrow
## its parameter.

fn make() -> Array[int]:
    values = Array[int]()
    values.push(owns 1)
    values.push(owns 2)
    values.push(owns 3)
    return values

fn first(values: ref Array[int]) -> ref int:
    return values[0]

fn main():
    values = make()
    println(values.len(), first(values))
