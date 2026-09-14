#$ test: compile-fail
#$ rules: FN-6b, LT-7, LT-10, TYP-15, TST-20, TST-21
#$ error[E3063]

fn stash(f: @latebound fn(Span[i32]) -> Span[i32], view: Span[i32]):
    stored: Box[Span[i32]] = Box(f(view))

fn identity(value: Span[i32]) -> Span[i32]:
    return value

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    stash(identity, values.as_span())
