#$ test: compile-fail
#$ rules: CTL-1, BRW-1, LT-1a
# `[CTL-1]` (ODR-211) — an item kept from an adapter borrows what the adapter reads, the map, so
# the map cannot be written while the item lives (`E3021`), as an item of the map's own iterator
# (ODR-196); and an item a consumer gives (`find`) borrows the iterator's source likewise.

fn main():
    m: Map[String, int] = Map()
    m.insert("a", 1)
    m.insert("bb", 2)
    best = ""
    for k in m.keys().filter(fn(k: ref String) => k.len() > 1).take(1):
        best = k
    m.insert("c", 3)    #$ error[E3021]: `best` is what keeps it alive
    println(best)
    words = ["x".to_string(), "yy".to_string()]
    found = words.iter().find(fn(w: ref String) => w.len() == 2)
    words.push("z".to_string())    #$ error[E3021]: `found` is what keeps it alive
    println(found)
