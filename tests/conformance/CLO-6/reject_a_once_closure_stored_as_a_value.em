#$ test: compile-fail
#$ rules: CLO-6, CLO-3

# `[CLO-6]` — an `owned fn` that moves its capture out can be called once, so
# it is not an owned callable value `fn(A) -> R`: `E3030`.

fn main():
    s = "hi".to_string()
    f: fn() -> String = owned fn() => s    #$ error[E3030]: closure would move a captured value out
    println(f())
