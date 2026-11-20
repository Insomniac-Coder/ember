# Clause BCK-2.1: "A view is **live** at a point when some value carrying it is used at a later point
# on some path from that point"
#$ test: compile-fail
#$ rules: BCK-2, BRW-17
# On the path through the `if`, `v` is used after `names` is cleared. No text in the spec: code only.

fn main():
    names = [String("a")]
    v = names[0]
    if len(names) > 0:
        names.clear()
    println(v)    #$ error[E3060]
