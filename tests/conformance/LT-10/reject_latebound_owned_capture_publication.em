#$ test: compile-fail
#$ rules: FN-6b, LT-7, LT-10, LT-42, TYP-15, TST-20, TST-21
#$ error[E3062]

fn identity(view: Span[i32]) -> Span[i32]:
    return view

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    callback: @latebound fn(Span[i32]) -> Span[i32] = identity
    task = owned fn() => callback(values.as_span())
    escaped: Span[i32] = task()
