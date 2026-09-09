#$ test: run-pass
#$ rules: CLO-2, CLO-1
# Capture mode is inferred per variable, and a closure may capture more than
# one. The environment gets a field per captured name, in the order the body
# first mentions them; nothing about the call site changes.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    a = 2
    b = 100
    println(apply(fn(x) => x * a + b, 4))
#$ stdout: 108
