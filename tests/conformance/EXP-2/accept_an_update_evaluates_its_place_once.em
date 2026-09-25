#$ test: run-pass
#$ rules: EXP-2, STD-17
#$ stdout: x
#$ i
#$ x
#$ i
#$ [1, 12, 3] 7
#$ {'a': 3, 'b': 2} 2 {'s': 9}
# D-296 — `a[i] op= x` evaluates `x`, then the place once, then reads,
# operates and writes. The index is computed once (it used to be computed
# twice), a map value's field or element is updated through one `index_mut`,
# and `m["a"] += m["b"]` reads `m["b"]` before `m` is borrowed for the write.

struct P:
    n: int

struct Stat:
    n: int

fn i() -> int:
    println("i")
    return 1

fn x() -> int:
    println("x")
    return 10

fn main():
    a = [1, 2, 3]
    a[i()] += x()
    ps = [P(1), P(2)]
    ps[i()].n += x() - 5
    println(a, ps[1].n)
    m: Map[String, int] = {"a": 1, "b": 2}
    m["a"] += m["b"]
    stats: Map[String, Stat] = {"a": Stat(1)}
    stats["a"].n += 1
    powers: Map[String, int] = {"s": 3}
    powers["s"] **= 2
    println(m, stats["a"].n, powers)
