#$ test: run-pass
#$ rules: CTL-1, EXP-6
#$ stdout: drop va
#$ stdout: key a
#$ stdout: drop vb
#$ stdout: key b
#$ stdout: ['a', 'b']
#$ stdout: y z
#$ stdout: got x1
#$ stdout: drop x1
#$ stdout: got x2
#$ stdout: drop x2
#$ stdout: drop x3
#$ stdout: after
# `[CTL-1]` — `for x in owned e:` consumes `e` through its `into_iter()`
# (`IntoIterator`) and yields owned values: a `Map`'s keys in insertion
# order, dropping each value as its key is taken; a `Set`'s elements; an
# `Array`'s elements, of which those a `break` leaves are dropped with the
# loop. What it yields may be moved on, as the keys are here.

struct Noisy:
    name: String

    fn drop(mut self):
        println("drop", self.name)

fn main():
    m: Map[String, Noisy] = Map()
    m["a"] = Noisy("va")
    m["b"] = Noisy("vb")
    kept: Array[String] = []
    for k in owned m:
        println("key", k)
        kept.push(k)
    println(kept)
    s: Set[String] = {"y", "z"}
    seen: Array[String] = []
    for e in owned s:
        seen.push(e)
    println(seen[0], seen[1])
    xs = [Noisy("x1"), Noisy("x2"), Noisy("x3")]
    for x in owned xs:
        println("got", x.name)
        if x.name == "x2":
            break
    println("after")
