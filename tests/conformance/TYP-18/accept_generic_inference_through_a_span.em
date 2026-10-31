#$ test: run-pass
#$ rules: TYP-18, SPN-1, LT-1a

[T]
fn identity(view: Span[T]) -> Span[T] from view:
    return view

fn main():
    values: Array[i32] = Array[i32]()
    values.push(61)
    seen = identity(values.as_span())
    println(seen[0])
#$ stdout: 61
