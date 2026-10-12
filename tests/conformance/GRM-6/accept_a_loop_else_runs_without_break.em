#$ test: run-pass
#$ rules: GRM-6, CTL-4
#$ stdout: no break
#$ stdout: 2
## `[GRM-6]` — a loop's `else` runs when the loop ends without `break`.

fn main():
    for i in 0..3:
        if i == 5:
            break
    else:
        println("no break")
    n = 0
    while n < 3:
        n += 1
        if n == 2:
            break
    else:
        println("not printed")
    println(n)
