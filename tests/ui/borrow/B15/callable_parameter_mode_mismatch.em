fn apply(f: fn(ref mut i32) -> i32, ref mut value: i32) -> i32:
    return f(value)

fn main():
    value: i32 = 4
    println(apply(fn(x) => x, value))
