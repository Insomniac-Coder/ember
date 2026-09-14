#$ test: compile-fail
#$ rules: TYP-15, LT-3, LT-14, LT-17, LT-30, DIA-7a, TST-17

# One static field does not make a multi-region view storable. The local Span
# slot remains non-static, so the all-slots rule rejects the unbounded Box.

@view
struct Pair:
    label: str
    values: Span[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    pair = Pair("label", values.as_span())
    _boxed: Box[Pair] = Box(pair) #$ error[E3063]: `Pair` is a view, so it may not be stored in a Box's contents
    println(1)
