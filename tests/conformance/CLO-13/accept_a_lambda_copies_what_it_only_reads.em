#$ test: run-pass
#$ rules: CLO-13, CLO-3, CLO-10
#$ profiles: debug, release, shipping
#$ stdout: 6 30

# `[CLO-13]` (D-554) — where an owned callable value is wanted, a lambda that
# only reads `Copy` variables copies them, so it outlives them: the parameter
# `n` after `adder` returns, each loop's `i` after its turn.

fn adder(n: int) -> fn(int) -> int:
    return fn(x: int) => x + n

fn main():
    fs: Array[fn() -> int] = []
    for i in 0..3:
        fs.push(fn() => i * 10)
    total = 0
    for i in 0..fs.len():
        total = total + fs[i]()
    a = adder(5)
    println(a(1), total)
