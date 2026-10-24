# A5: p05f with the annotation the body implies: what inference would record.
@borrows(a, b)
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
