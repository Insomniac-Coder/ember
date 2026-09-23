#$ test: run-pass
#$ rules: STD-9, TXT-9
#$ profiles: debug, release, shipping
#$ stdout: abc
#$ built
# D-191 — a `String` argument prints its text, alone as among several.

fn name() -> String:
    return "built"

fn main():
    s: String = "abc"
    println(s)
    println(name())
