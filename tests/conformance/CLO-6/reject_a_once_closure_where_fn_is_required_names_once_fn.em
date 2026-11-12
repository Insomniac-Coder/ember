#$ test: compile-fail
#$ rules: CLO-6
#$ error[E3013]: this function gives `s` away, so it can run only once, and `many` may run it more than once
#$ help: take the parameter `owns f: once fn(...) -> ...`
# `[CLO-6]` — a closure that moves a capture out satisfies `once fn` but not `fn`: supplying it
# where `fn` is required is `E3013`, whose help is the `once fn` the callee could take.

fn consume(owns s: String) -> int:
    return s.len()

fn many(f: fn() -> int) -> int:
    return f()

fn main():
    s = "x".to_string()
    println(many(owned fn() => consume(owns s)))
