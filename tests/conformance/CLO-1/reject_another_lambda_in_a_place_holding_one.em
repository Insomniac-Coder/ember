#$ test: compile-fail
#$ rules: CLO-1, CLO-3, DIA-2

# `[CLO-1]` (D-555) — each lambda has a type of its own, so a place holding one
# cannot hold another; the message says so in the program's terms (it named
# the compiler's environment types) and points at `fn(A) -> R`.

fn main():
    base = 1
    f = fn() => base
    s = "abc".to_string()
    f = fn() => s.len()    #$ error[E2020]: this place holds another lambda, and each lambda has a type of its own
    println(f())
