#$ test: run-pass
#$ rules: STD-11, STD-16, CTL-1
#$ stdout: ['b', 'a', 'c']
#$ stdout: ['b', 'c', 'a']
#$ stdout: 200 k0 k198
#$ stdout: [3, 1, 2]
# `[STD-11]` — a `Map` iterates in insertion order, like Python's `dict`:
# re-assigning a key keeps its place, removing one keeps the order of the
# others, and a key inserted again after removal goes to the end. The order
# survives growth, and a `Set` keeps it the same way.

fn keys(m: Map[String, int]) -> Array[String]:
    out: Array[String] = []
    for k in m:
        out.push(k.clone())
    return out

fn main():
    m: Map[String, int] = {"b": 1, "a": 2, "c": 3}
    m["a"] = 20
    println(keys(m))
    m.remove("a")
    m["a"] = 30
    println(keys(m))
    big: Map[String, int] = {}
    for i in range(200):
        big[f"k{i}"] = i
    for i in range(0, 200, 3):
        big.remove(f"k{i}")
    for i in range(0, 200, 3):
        big[f"k{i}"] = i
    order = keys(big)
    println(len(big), order[133], order[199])
    s = {3, 1, 2, 3}
    elements: Array[int] = []
    for x in s:
        elements.push(x)
    println(elements)
