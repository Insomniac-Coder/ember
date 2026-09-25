#$ test: compile-fail
#$ rules: STD-17, FN-1
# `[STD-17]` — `m[k] = v` writes `m`: through a borrowed parameter it is
# `E3023`, as any write is.

fn bad(m: Map[String, int]):
    m["a"] = 2  #$ error[E3023]
