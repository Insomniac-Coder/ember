#$ test: compile-fail
#$ rules: STD-17, FN-1
## `[STD-17]` — a mutable view of `m[k]` writes `m`: through a borrowed parameter it is `E3023`, as
## `m[k] = v` is (D-818).

fn bump(m: Map[String, Array[int]]):
    for v in m["a"].iter_mut():  #$ error[E3023]
        v += 1
