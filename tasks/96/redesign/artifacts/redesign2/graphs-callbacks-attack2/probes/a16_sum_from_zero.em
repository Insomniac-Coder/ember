# A16 attack: Python's sum starts from 0. With bounds written (Add), the literal 0 is still an int.
# Python: def total(xs): acc = 0; for x in xs: acc = acc + x; return acc   ->  total([1.5, 2.5]) == 4.0
fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:
    acc = 0
    for x in xs:
        acc = acc + x
    return acc

fn main():
    println(total([1.5, 2.5]))
