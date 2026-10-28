#$ test: run-pass
#$ rules: OWN-8
#$ stdout: hi
## design rules: R3
## R3: the nearest program that is allowed: `String(w)` makes text of your own.

fn main():
    line = String(" hi ")
    w = line.trim()
    c = String(w)
    println(c)
