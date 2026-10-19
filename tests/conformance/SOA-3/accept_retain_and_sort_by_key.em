#$ test: run-pass
#$ rules: SOA-3, SOA-6
#$ profiles: debug, development
#$ stdout: n0 0
#$ n4 6
#$ n5 4
#$ n6 2
#$ n7 0
#$ sorted
#$ n0 0
#$ n7 0
#$ n6 2
#$ n5 4
#$ n4 6
# `[SOA-3]` — `retain` keeps the elements its function is true of, in order (the others' `String`s
# dropped); `sort_by_key` sorts by a key of each element, stably, every column moved together.

struct Named:
    name: String
    score: int

fn main():
    ns = SoA[Named].with_capacity(2)
    for i in range(8):
        ns.push(Named(f"n{i}", (i * 5) % 7))
    ns.retain(fn(p) => p.score % 2 == 0)
    for p in ns:
        println(p.name, p.score)
    ns.sort_by_key(fn(p) => p.score)
    println("sorted")
    for p in ns:
        println(p.name, p.score)
