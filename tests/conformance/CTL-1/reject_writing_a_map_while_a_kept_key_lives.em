#$ test: compile-fail
#$ rules: CTL-1, CTL-2, BRW-1, LT-1a
# `[CTL-1]` — a key kept from a `for` over a map borrows the map (`[LT-1a]`, ODR-196), so the map
# cannot be written while the key lives: inside the loop that is `[CTL-2]`'s `E3020`; after the
# loop the loop's borrow is over, and the kept key's is an ordinary one, `E3021` (`[BRW-1]`).

fn main():
    m: Map[String, int] = Map()
    m.insert("a", 1)
    best = ""
    for k in m:
        best = k
    m.insert("b", 2)    #$ error[E3021]: `best` is what keeps it alive
    println(best)
    n: Map[String, int] = Map()
    n.insert("a", 1)
    kept = ""
    for k in n.keys():
        kept = k
        n.insert("c", 3)    #$ error[E3020]: cannot mutate `n` while it is borrowed by this loop
    println(kept)
    s: Set[String] = Set()
    s.add("x")
    top = ""
    for v in s:
        top = v
    s.clear()    #$ error[E3021]: `top` is what keeps it alive
    println(top)
