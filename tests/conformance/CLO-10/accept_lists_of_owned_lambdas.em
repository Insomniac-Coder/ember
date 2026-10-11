#$ test: run-pass
#$ rules: CLO-10, CLO-3, TYP-23
#$ profiles: debug, release, shipping
#$ stdout: 3 1 21 2

# `[CLO-10]` (D-556) — owned callable values in a typed list, a list whose
# element type an `owned fn` gives it, and a local that holds either of two
# lambdas as `fn() -> int`.

fn pick(flag: bool) -> int:
    base = 1
    f: fn() -> int = fn() => base
    if flag:
        s = "abc".to_string()
        f = owned fn() => s.len()
    return f()

fn main():
    fs: Array[fn() -> int] = []
    for i in 0..3:
        s = f"item {i}!"
        fs.push(owned fn() => s.len())
    total = 0
    for i in 0..fs.len():
        total = total + fs[i]()
    later = []
    for i in 0..2:
        t = f"x{i}"
        later.push(owned fn() => t.len())
    println(pick(true), pick(false), total, later.len())
