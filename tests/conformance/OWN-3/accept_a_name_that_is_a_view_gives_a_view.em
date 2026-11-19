#$ test: run-pass
#$ rules: OWN-3
#$ stdout: 2 2 2
# `[OWN-3]` — `b = a` where `a` is itself a view makes `b` a view of the same value; `ref` may say so.
# A view hands nothing over.

fn count(xs: Array[int]) -> int:
    view = xs
    cur = ref xs
    return view.len() + cur.len() - xs.len()

fn main():
    xs = [1, 2]
    println(count(xs), count(xs), xs.len())
