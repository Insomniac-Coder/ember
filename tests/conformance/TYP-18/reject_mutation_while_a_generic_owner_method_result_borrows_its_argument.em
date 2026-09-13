#$ test: compile-fail
#$ rules: TYP-18, LT-1a, BRW-1

struct Holder[T]:
    value: T

    @borrows(view)
    fn passthrough[U](self, view: Span[U]) -> Span[U]:
        return view

fn main():
    holder = Holder[i32](0)
    values: Array[i32] = Array[i32]()
    values.push(59)
    seen = holder.passthrough(values.as_span())
    values.push(60) #$ error[E3021]: `values` is borrowed here and mutably borrowed elsewhere
    println(seen[0])
