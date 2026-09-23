#$ test: run-pass
#$ rules: STD-9
#$ profiles: debug, release, shipping
#$ stdout: 1 2.5 true x
#$ 1, 2, 3
#$ a-b!
#$ [f][f]Ada 7 7
#$
#$ done
# `print`/`println` take several arguments, separated by `sep` (default one
# space) and followed by `end` (a newline for `println`). Every argument is
# evaluated before anything is printed, as in Python. A `String` argument
# prints its text.

fn f() -> int:
    print("[f]")
    return 7

fn main():
    name: String = "Ada"
    println(1, 2.5, true, "x")
    println(1, 2, 3, sep=", ")
    print("a", "b", sep="-", end="!\n")
    println(name, f(), f())
    println()
    println("done")
