#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ stdout:
#$ 10 5 110
#$ 11 6 210
#$ 12 7 310
# `[CTL-3b]` — `start` is evaluated once, when the chain is made: the loop
# changing the variable it was read from does not change the numbers. (The
# value is held in a hidden local, which is read in place of its source only
# where the source still holds it.)

fn main():
    xs: Array[int] = [5, 6, 7]
    s = 10
    for (i, x) in xs.iter().enumerate(start=s):
        s += 100
        println(f"{i} {x} {s}")
