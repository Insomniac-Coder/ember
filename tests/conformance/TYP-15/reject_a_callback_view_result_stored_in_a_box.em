#$ test: compile-fail
#$ rules: TYP-15, LT-7
#$ error[E3063]
# A view a callback returns is still a view: a Box may not hold it.

fn stash(f: fn(Span[i32]) -> Span[i32], view: Span[i32]):
    _stored: Box[Span[i32]] = Box(f(view))

fn identity(value: Span[i32]) -> Span[i32]:
    return value

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    stash(identity, values.as_span())
