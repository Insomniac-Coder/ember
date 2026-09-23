#$ test: run-fail
#$ rules: TYP-31
#$ profiles: debug, release, shipping
#$ panics: index -1 is out of bounds for a length of 3
# A negative index computed at run time is out of bounds; nothing counts
# from the end.

fn main():
    xs = [10, 20, 30]
    back = 1
    println(xs[0 - back])
