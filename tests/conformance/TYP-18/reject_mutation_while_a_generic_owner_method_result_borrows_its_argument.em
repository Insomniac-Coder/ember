#$ test: compile-fail
#$ rules: TYP-18, LT-1a, BRW-1

[T]
struct Holder:
    value: T

    [U]
    fn passthrough(view: Span[U]) -> Span[U] from view:
        return view

fn main():
    holder = Holder[i32](0)
    values: Array[i32] = Array[i32]()
    values.push(owns 59)
    seen = holder.passthrough(values.as_span())
    values.push(owns 60)
    println(seen[0])    #$ error[E3060]
