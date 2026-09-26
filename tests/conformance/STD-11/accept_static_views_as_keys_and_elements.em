#$ test: run-pass
#$ rules: STD-11, STD-16, TYP-15, TYP-38
#$ stdout: {'start': 1, 'stop': 2, 'go': 3} Some(2) true
#$ stdout: {'red', 'green'} true
#$ stdout: ['start', 'stop', 'go'] 6
#$ stdout: {'a': 1, 'b': 2}
# `[STD-11]`, `[TYP-15]` (ODR-069, SP-013) — a `Map` or `Set` may hold views
# under the one storage rule every owning container follows: only `static`
# ones, such as string literals. A key copied out of the map is `static` too.
# `entry(k)` returns an entry holding `k` (ODR-069). An unannotated literal
# keeps owning `String` keys (`[TYP-38]`).

fn main():
    labels: Map[str, int] = {"start": 1, "stop": 2}
    labels["go"] = 3
    labels.entry("stop").or_insert(9)
    println(labels, labels.get("stop"), "go" in labels)
    colours: Set[str] = Set[str]()
    colours.add("red")
    colours.add("green")
    println(colours, "red" in colours)
    keys: Array[str] = Array[str]()
    total = 0
    for k, v in labels.items():
        keys.push(k)
        total += v
    println(keys, total)
    texts = {"a": 1}
    texts.insert(String.from("b"), 2)
    println(texts)
