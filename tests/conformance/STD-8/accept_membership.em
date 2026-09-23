#$ test: run-pass
#$ rules: STD-8, STD-8b
#$ profiles: debug, release, shipping
#$ stdout: true false true
#$ true false
#$ true true false true
#$ false true true
# `x in c`: a scan of an Array, a substring or character test on text, and two
# comparisons on a range written in place; `not in` negates.

fn main():
    xs = [3, 1, 4]
    println(4 in xs, 7 in xs, 7 not in xs)
    names = ["ann", "bo"]
    println("bo" in names, "cy" in names)
    s = "héllo world"
    println("world" in s, 'é' in s, "xyz" in s, 'q' not in s)
    n = 5
    println(n in 0..5, n in 0..=5, 3 not in 1..3)
