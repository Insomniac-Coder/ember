#$ test: run-pass
#$ rules: STD-10
#$ stdin: ann
#$ stdin: 42
#$ stdin: end
#$ stdout: name? hi ann
#$ age? 2
#$ end
# `[STD-10]` — `input(prompt="")` prints the prompt, flushes, and returns one
# line of standard input without its ending.

fn main():
    name = input("name? ")
    println(f"hi {name}")
    age = input(prompt="age? ")
    println(age.len())
    last = input()
    println(last)
