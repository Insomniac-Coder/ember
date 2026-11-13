#$ test: run-pass
#$ rules: GRM-4
#$ stdout: 6
## `[GRM-4]` — a declaration inside a loop's body declares its name once per round: each round's name is
## a new one (no `E1020` for the second round, and the name does not carry its value over).

fn main():
    total = 0
    for i in 0..3:
        x = i * 2
        total += x
    println(total)
