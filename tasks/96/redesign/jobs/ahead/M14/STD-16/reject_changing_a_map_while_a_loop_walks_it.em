# Clause STD-16.2: "While a loop walks a map (its keys, its values or its items), nothing changes the
# map, through its own name or through a call (`counts[k] *= 2` and `counts.insert(...)` inside it are
# `E3020`)"
#$ test: compile-fail
#$ rules: STD-16, BRW-18
# `[BRW-18]`'s message for the insert, with this file's line numbers. (Its `counts[k] *= 2` message is
# in `ahead/M6a/DIA-10`.)

fn main():
    counts: Map[String, int] = Map()
    counts.insert(owns String("a"), owns 1)
    for k in counts:
        counts.insert(owns k.to_upper(), owns 0)    #$ error[E3020]: `counts.insert(...)` (line 13) adds to `counts`, whose keys this loop walks (line 12); collect the new keys in a list of your own: write `upper: Array[String] = []` before the loop, `upper.push(owns k.to_upper())` in place of this line, and `for u in owns upper: counts.insert(owns u, owns 0)` after the loop
