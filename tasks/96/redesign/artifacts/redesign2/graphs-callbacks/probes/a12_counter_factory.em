# A12: a closure returned from its maker, writing its own captured counter (Python: nonlocal).
fn make_counter() -> fn() -> int:
    count = 0
    fn inc() -> int:
        count += 1
        return count
    return inc

fn main():
    c = make_counter()
    _a = c()
    println(c())
