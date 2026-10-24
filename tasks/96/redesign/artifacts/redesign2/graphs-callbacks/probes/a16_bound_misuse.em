# A16: what a written bound protects: the error is at the call, not deep inside the body.
fn sum[T: Add[Output = T] + Copy](xs: Span[T]) -> T:
    acc = xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

struct Name:
    s: int

fn main():
    println(sum([Name(s=1), Name(s=2)]).s)
