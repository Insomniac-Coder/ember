fn apply(f: fn(mut i32) -> i32, mut value: i32) -> i32:
    return f(value)

fn main():
    value = 4
    println(apply(fn(mut x) => x, value))
