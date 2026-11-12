#$ test: compile-fail
#$ rules: CLO-1, CLO-2, CLO-3, CLO-6, OWN-3
#$ help: take the parameter `owns f: once fn(...) -> ...` in `invoke` when it runs the function at most once
# `[CLO-6]` (D-850) — a closure that moves an owned capture out of its environment can be called
# once, so its type satisfies `once fn(A) -> R` and not `fn(A) -> R`: given to `owned f: fn(...)`,
# which is called any number of times, it is `E3013` (`E3030` is retired), whose help names `owns f: once fn`. (This test said
# the opposite while an `owned` callable parameter was taken as `once fn`.)

fn consume(owns values: Array[i32]) -> i32:
    return values[0]

fn invoke(owns f: fn() -> i32) -> i32:
    return f()

fn invoke_once(f: once fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 7)
    task = owned fn() => consume(owns values)
    println(invoke(task))    #$ error[E3013]: this function gives `values` away, so it can run only once, and `invoke` may run it more than once
    more: Array[i32] = [8]
    println(invoke_once(owns owned fn() => consume(owns more)))
