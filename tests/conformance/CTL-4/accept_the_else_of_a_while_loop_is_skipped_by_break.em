#$ test: run-pass
#$ rules: CTL-4
#$ stdout: 3
#$ stdout: done
# `[CTL-4]` — a `while` loop's `else` runs when its condition ends it, and not after a `break`.

fn main():
    i = 0
    while i < 3:
        i += 1
        if i == 10:
            break
    else:
        println(i)
    while true:
        break
    else:
        println("never")
    println("done")
