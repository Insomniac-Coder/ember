#$ test: run-pass
#$ rules: GRM-28, CLO-12, CLO-2
#$ stdout: 15
#$ stdout: 120
#$ stdout: 10
#$ stdout: 11
#$ stdout: hi
## `[GRM-28]` — a `fn` inside a block is a local function: it captures the
## enclosing function's locals as a lambda does, may call itself, and is a
## callable value like any closure (D-585: it did not parse).

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn main():
    base = 10
    fn add(x: int) -> int:
        return x + base
    fn fact(n: int) -> int:
        if n <= 1:
            return 1
        return n * fact(n - 1)
    fn count_down(n: int) -> int:
        if n == 0:
            return base
        return count_down(n - 1)
    fn greet():
        println("hi")
    println(add(5))
    println(fact(5))
    println(count_down(3))
    println(apply(add, 1))
    greet()
