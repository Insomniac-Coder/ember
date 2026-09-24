#$ test: run-pass
#$ rules: STR-5, COST-1
#$ stdout: 1 1
#$ assert-c: contains(em_Used_clone)
#$ assert-c: !contains(em_Unused_clone)
# `[COST-1]` — an implicit `clone` is code only where the program clones:
# `Used` is cloned and has its function; `Unused` never is, and has none.

struct Used:
    n: int
    tags: Array[String]

struct Unused:
    n: int
    tags: Array[String]

fn main():
    a = Used(n=1, tags=["x"])
    b = a.clone()
    c = Unused(n=1, tags=["y"])
    println(b.n, c.n)
