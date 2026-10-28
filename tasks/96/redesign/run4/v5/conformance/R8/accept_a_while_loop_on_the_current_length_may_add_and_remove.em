#$ test: run-pass
#$ rules: BRW-16
#$ stdout: [1, 3, 4, 5]
## design rules: R8
## R8 (c): a `while` loop that tests its position against the list's current length at every step is
## not a walk: it may add and remove, and each index is checked.

fn main():
    xs = [1, 2, 3, 4]
    i = 0
    while i < xs.len():
        if xs[i] == 2:
            xs.remove(i)
        elif xs[i] == 3:
            xs.push(5)
            i += 1
        else:
            i += 1
    println(xs)
