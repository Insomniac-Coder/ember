#$ test: run-pass
#$ rules: R3
#$ stdout: hi
## R3: the nearest program that is allowed: `String(w)` makes text of your own.

fn main():
    line = String(" hi ")
    w = line.trim()
    c = String(w)
    println(c)
