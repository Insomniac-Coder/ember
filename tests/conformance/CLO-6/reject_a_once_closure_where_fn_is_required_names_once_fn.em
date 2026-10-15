#$ test: compile-fail
#$ rules: CLO-6
#$ error[E3030]: closure would move a captured value out
#$ help: declare the parameter `f: once fn(...) -> ...`
# `[CLO-6]` — a closure that moves a capture out satisfies `once fn` but not `fn`: supplying it
# where `fn` is required is `E3030`, whose help is the `once fn` the callee could take.

fn consume(owned s: String) -> int:
    return s.len()

fn many(f: fn() -> int) -> int:
    return f()

fn main():
    s = "x".to_string()
    println(many(owned fn() => consume(s)))
