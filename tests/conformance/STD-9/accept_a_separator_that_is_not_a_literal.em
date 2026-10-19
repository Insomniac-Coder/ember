#$ test: run-pass
#$ rules: STD-9, EXP-1
#$ profiles: debug, development
#$ stdout: a, b, c
#$ stdout: 1 -> 2 -> 3 -> 4!
#$ stdout: [first] 1 [second] 2
#$ stdout: only
#$ stdout: calls: 1
# `[STD-9]` — `sep` is any text, evaluated once where it is written, as every argument is
# (`[EXP-1]`), and read between the values; `end` too.

fn arrow(mut calls: int) -> String:
    calls += 1
    return " -> "

fn main():
    comma = ", "
    println("a", "b", "c", sep=comma)
    calls = 0
    print(1, 2, 3, 4, sep=arrow(calls), end="!\n")
    labels = ["[first]", "[second]"]
    println(labels[0], 1, labels[1], 2, sep=" ")
    println("only", sep=comma)
    println(f"calls: {calls}")
