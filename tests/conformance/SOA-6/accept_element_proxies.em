#$ test: run-pass
#$ rules: SOA-6, CTL-3b, SOA-2
#$ profiles: debug, release
#$ stdout: bob 5 Named(name='bob', score=5)
#$ 15
#$ Some(Named(name='dee', score=1))
#$ ann 3
#$ bob 15
#$ dee 1
#$ 38 Named(name='ann', score=6)
#$ 1
# `[SOA-6]` — `ps[i]` is an element proxy: an `SoAMut` where `ps` is mutable, whose fields are
# places in the columns (`p.score += 10` writes one); `load()` gathers a copy; `ps[i] = v` scatters
# one; `for p in ps` gives `SoARef`s and `for p in ps.iter_mut()` `SoAMut`s (`[CTL-3b]`: a counted
# loop); an `SoAMut` passes where an `SoARef` is wanted.

struct Named:
    name: String
    score: int

fn score_of(p: SoARef[Named]) -> int:
    return p.score

fn main():
    ns = SoA[Named].with_capacity(2)
    ns.push(Named("ann".to_string(), 3))
    ns.push(Named("bob".to_string(), 5))
    ns.push(Named("cy".to_string(), 8))
    p = ns[1]
    println(p.name, p.score, p.load())
    p.score += 10
    println(ns[1].score)
    ns[2] = Named("dee".to_string(), 1)
    println(ns.get(2))
    for q in ns:
        println(q.name, q.score)
    for q in ns.iter_mut():
        q.score *= 2
    s = 0
    for q in ns:
        s += score_of(q)
    println(s, ns[0].load())
    println(score_of(ns[2]) // 2)
