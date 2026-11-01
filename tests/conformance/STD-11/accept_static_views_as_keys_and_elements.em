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
    labels: Map[StrView, int] = {"start": 1, "stop": 2}
    labels["go"] = 3
    labels.entry(owns "stop").or_insert(owns 9)
    println(labels, labels.get("stop"), "go" in labels)
    colours: Set[StrView] = Set[StrView]()
    colours.add(owns "red")
    colours.add(owns "green")
    println(colours, "red" in colours)
    keys: Array[StrView] = Array[StrView]()
    total = 0
    for k, v in labels.items():
        keys.push(owns k)
        total += v
    println(keys, total)
    texts = {"a": 1}
    texts.insert(owns String.from("b"), owns 2)
    println(texts)
