#$ test: run-pass
#$ rules: STD-16, STD-12, HASH-4
#$ stdout: None Some(1) 2 Some(1) true
#$ stdout: 11 -1 2 false
#$ stdout: {'a': 3, 'b': 1}
#$ stdout: {'x': [5, 6]}
#$ stdout: a b | 11 2 | a=11 b=2
#$ stdout: 1 Some(('a', 110)) 0 true
#$ stdout: [('p', 7), ('q', 8)]
#$ stdout: true false ['a', 'b', 'c']
# `[STD-16]` (ODR-032) — the `Map` operations with their signatures.

fn main():
    m: Map[String, int] = {}
    first = m.insert("a", 1)
    again = m.insert("a", 1)
    m.insert("b", 2)
    println(first, again, len(m), m.get("a"), m.contains_key("b"))
    match m.get_mut("a"):
        Some(v):
            v += 10
        None:
            pass
    println(m["a"], m.get_or("z", -1), len(m), m.is_empty())
    counts: Map[String, int] = {}
    for w in ["a", "b", "a", "a"]:
        n = counts.entry(String.from(w)).or_insert(0)
        n += 1
    println(counts)
    lists: Map[String, Array[int]] = {}
    lists.entry("x").or_default().push(5)
    lists.entry("x").or_insert_with(fn() => [0]).push(6)
    println(lists)
    text = String.from("")
    for k in m.keys():
        text += f"{k} "
    text += "|"
    for v in m.values():
        text += f" {v}"
    text += " |"
    for k, v in m.items():
        text += f" {k}={v}"
    println(text)
    for v in m.values_mut():
        v *= 10
    m.retain(fn(k, v) => v > 100)
    println(len(m), m.pop_item(), len(m), m.is_empty())
    other: Map[String, int] = {"p": 7}
    other.update({"q": 8})
    println(other.into_items())
    x: Map[String, int] = {"a": 1, "b": 2}
    y: Map[String, int] = {"b": 2, "a": 1}
    x.clear()
    println(y == {"a": 1, "b": 2}, x == y, sorted({"c": 1, "a": 2, "b": 3}))
