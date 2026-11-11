#$ test: run-pass
#$ rules: CTL-4
#$ stdout: found 2
#$ stdout: not found
# `[CTL-4]` — the `else` of `while` or `for` runs when the loop ends without `break`.

fn find(xs: Array[i32], want: i32):
    for x in xs:
        if x == want:
            println("found", x)
            break
    else:
        println("not found")

fn main():
    xs: Array[i32] = Array[i32]()
    xs.push(owns 1)
    xs.push(owns 2)
    find(xs, 2)
    find(xs, 9)
