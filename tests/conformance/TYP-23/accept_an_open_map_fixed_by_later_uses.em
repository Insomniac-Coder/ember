#$ test: run-pass
#$ rules: TYP-23, TYP-38, STD-16
#$ stdout: {'a': 1} {'b': 2} {3} {'x'} {'a': 2, 'b': 1}
# `[TYP-23]` — `{}`, `Map()` and `Set()` are open like `[]`: a later
# `m[k] = v`, `m.insert(k, v)`, `s.add(x)`, or the type a return wants, says
# what they hold. Python's counting idiom needs nothing more.

fn names() -> Map[String, int]:
    m = Map()
    m.insert(String.from("a"), 1)
    return m

fn tally(words: Array[String]) -> Map[String, int]:
    counts = {}
    for w in words:
        if w in counts:
            counts[w] += 1
        else:
            counts[w] = 1
    return counts

fn main():
    d = {}
    d["b"] = 2
    s = Set()
    s.add(3)
    t = Set()
    t.add("x")
    println(names(), d, s, t, tally(["a", "b", "a"]))
