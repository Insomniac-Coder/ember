fn apply(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn main():
    step: fn(int) -> int = fn(x) => x * 3 + 1
    total = 0
    for i in 0..200000000:
        total = total + apply(step, i) % 7
    println(total)
