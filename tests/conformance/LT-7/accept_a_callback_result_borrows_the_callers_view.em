#$ test: run-pass
#$ rules: LT-7, LT-1
#$ stdout: 1
# A callback's result may borrow from its arguments as `[LT-1]` allows, for as
# long as the caller keeps them: `publish` returns a view of the caller's
# array. `@latebound` rejected this as an escape (F-168).

fn identity(view: Span[i32]) -> Span[i32]:
    return view

fn publish(f: fn(Span[i32]) -> Span[i32], view: Span[i32]) -> Span[i32]:
    return f(view)

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    escaped: Span[i32] = publish(identity, values.as_span())
    println(escaped[0])
