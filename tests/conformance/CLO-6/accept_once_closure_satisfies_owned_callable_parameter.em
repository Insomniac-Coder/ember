#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3, OWN-5, TST-19
#$ stdout: 7

# `owned f: fn(...)` carries the `CallableOnce` bound, so it accepts a
# concrete closure which moves an owned capture out of its environment.

fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    println(invoke(task))
