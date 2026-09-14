#$ test: compile-fail
#$ rules: FN-6b, LT-7, LT-10, TST-20, TST-21
#$ error[E3062]

fn identity(view: Span[i32]) -> Span[i32]:
    return view

fn publish(f: @latebound fn(Span[i32]) -> Span[i32], view: Span[i32]) -> Span[i32]:
    return f(view)

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    escaped: Span[i32] = publish(identity, values.as_span())
