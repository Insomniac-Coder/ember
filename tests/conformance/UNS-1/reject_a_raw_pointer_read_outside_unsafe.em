#$ test: compile-fail
#$ rules: UNS-1
#$ error[E3100]: `read` through a raw pointer needs an `unsafe` block
# `[UNS-1]` — reading through a raw pointer needs `unsafe`.

fn first(p: *u8) -> u8:
    return p.read()

fn main():
    a: Array[u8] = [1]
    println(first(a.as_span().as_ptr()))
