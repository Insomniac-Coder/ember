#$ test: run-pass
#$ rules: LEX-26
#$ stdout: A~
#$ stdout: ab
## `[LEX-26]` — a text literal takes `\x` with `00` to `7F`; a raw string's prefix is `#`s and a `"`.

fn main():
    println("\x41\x7E")
    println(r##"ab"##)
