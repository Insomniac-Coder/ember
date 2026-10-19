#$ test: run-pass
#$ rules: LEX-19, STD-9
#$ profiles: debug, development, release
#$ stdout: ab ab
#$ ab
# D-194 — an f-string reads a `String` it writes; it does not consume it.

fn main():
    s: String = "ab"
    println(f"{s} {s}")
    println(s)
