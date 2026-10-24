# A16: a private generic helper written as Python does: no bounds.
fn sum[T](xs: Span[T]) -> T:
    acc = xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

fn main():
    println(sum([1, 2, 3]))
    println(sum([1.5, 2.5]))
