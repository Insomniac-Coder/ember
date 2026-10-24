# A5: a conditionally chosen mutable part, no annotation (Hylo's `min` subscript).
fn smaller(mut a: int, mut b: int) -> ref mut int:
    if a <= b:
        return ref mut a
    return ref mut b

fn main():
    x = 3
    y = 5
    r = smaller(x, y)
    r = 1
    println(x, y)
