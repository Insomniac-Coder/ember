#$ test: run-pass
#$ rules: LEX-20, TXT-9
#$ stdout: hello
#$ stdout: 3
## `[LEX-20]` — a string literal is a `str` with the static region, and at a
## site expecting `String` it produces one.

fn length(name: String) -> int:
    return name.len()

fn main():
    s: String = "hello"
    println(s)
    println(length("abc"))
