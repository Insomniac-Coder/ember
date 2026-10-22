#$ test: compile-fail
#$ rules: LEX-26
# `[LEX-26]` — a raw string's prefix is at most eight `#` and a `"`: nine is a malformed literal
# (`E0009`).

fn main():
    a = r#########"nine"#########    #$ error[E0009]: a raw string may have at most 8 hashes
    println(a)
