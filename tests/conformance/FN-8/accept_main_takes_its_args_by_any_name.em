#$ test: run-pass
#$ rules: FN-8
#$ args: one
#$ stdout: one
# ODR-101 — `main`'s parameter may have any name and mode; its type makes it
# the command line.

fn main(mut words: Array[String]):
    println(words[1])
