# A12: an escaping closure reads a variable written after it was made (phase3-break-D1.md B8).
fn make() -> fn() -> int:
    n = 1
    f = fn() => n
    n = 2
    return f

fn main():
    println(make()())
