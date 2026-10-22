#$ test: compile-fail
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3
#$ help: declare the parameter `f: once fn(...) -> ...` when it is called at most once
# `[CLO-6]` (D-850) — a closure that moves an owned capture out of its environment can be called
# once, so its type satisfies `once fn(A) -> R` and not `fn(A) -> R`: given to `owned f: fn(...)`,
# which is called any number of times, it is `E3030`, whose help names `once fn`. (This test said
# the opposite while an `owned` callable parameter was taken as `once fn`.)

fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn invoke_once(f: once fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    println(invoke(task))    #$ error[E3030]: closure would move a captured value out
    more: Array[i32] = [8]
    println(invoke_once(owned fn() => consume(more)))
