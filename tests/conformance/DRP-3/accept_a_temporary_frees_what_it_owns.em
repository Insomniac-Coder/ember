#$ test: run-pass
#$ rules: DRP-3, EXP-4, OWN-2
#$ assert-c: contains("ember_vec_free")
# The leak `[DRP-3]` exists to prevent. `R(a)` is a temporary that owns an
# `Array[i32]`, built three times and bound to nothing; before temporaries were
# dropped, all three buffers leaked. The `#$ assert-c` is what proves it — the
# program's output is identical either way, which is exactly why the leak
# survived.

struct R:
    pub v: Array[i32]

fn take(r: R) -> i32:
    return r.v.len() as i32

fn main():
    i: i32 = 0
    while i < 3:
        a: Array[i32] = Array[i32]()
        a.push(1)
        println(take(R(a)))
        i = i + 1
#$ stdout: 1
#$ 1
#$ 1
