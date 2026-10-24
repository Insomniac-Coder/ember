# A16: the same with the bounds today's rule asks for.
fn sum[T: Add[Output = T] + Copy](xs: Span[T]) -> T:
    acc = xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

fn main():
    println(sum([1, 2, 3]))
    println(sum([1.5, 2.5]))
