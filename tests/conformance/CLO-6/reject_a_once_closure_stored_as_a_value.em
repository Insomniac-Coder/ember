#$ test: compile-fail
#$ rules: CLO-6, CLO-3

# `[CLO-6]` — an `owned fn` that moves its capture out can be called once, so
# it is not an owned callable value `fn(A) -> R`: `E3013` (`E3030` is retired).

fn main():
    s = "hi".to_string()
    f: fn() -> String = owned fn() => s    #$ error[E3013]: this function gives `s` away, so it can run only once, and `fn() -> String` may run it more than once
    println(f())
