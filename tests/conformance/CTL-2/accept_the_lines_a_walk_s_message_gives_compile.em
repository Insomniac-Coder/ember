#$ test: run-pass
#$ rules: CTL-2, BRW-18, DIA-25
#$ stdout: [5, 5]
#$ stdout: [1, 2, 1, 2]
#$ stdout: a 2
#$ stdout: 4
#$ stdout: 3
#$ stdout: 4
# `[CTL-2]`, `[BRW-18]`, `[DIA-25]` — the lines `E3020`'s message gives compile: the loop's own handle for
# an element (`for ref mut x in xs:`, `x = 5`) and a map's value (`for k, ref mut v in counts:`, `v *= 2`); a
# list of your own for a push (`added: Array[int] = []`, `added.push(owns y)`, `ys.extend(owns added)`), for
# a map's new keys (`for u in owns new_keys: counts.insert(owns u, owns 0)`) and a set's new items.

fn main():
    xs = [1, 2]
    for ref mut x in xs:
        x = 5
    println(xs)
    ys = [1, 2]
    added: Array[int] = []
    for y in ys:
        added.push(owns y)
    ys.extend(owns added)
    println(ys)
    counts = Map[String, int]()
    counts.insert(owns String("a"), owns 1)
    for k, ref mut v in counts:
        v *= 2
        println(k, v)
    keyed = Map[int, int]()
    keyed.insert(owns 1, owns 1)
    keyed.insert(owns 2, owns 1)
    new_keys: Array[int] = []
    for k in keyed:
        new_keys.push(owns (k + 10))
    for u in owns new_keys:
        keyed.insert(owns u, owns 0)
    println(keyed.len())
    st: Set[int] = Set[int]()
    st.add(owns 1)
    st.add(owns 2)
    more: Array[int] = []
    for x in st:
        more.push(owns (x + 1))
    for u in owns more:
        st.add(owns u)
    println(st.len())
    pairs = Map[int, int]()
    pairs.insert(owns 1, owns 5)
    pairs.insert(owns 2, owns 6)
    new_entries: Array[(int, int)] = []
    for k, v in pairs:
        new_entries.push(owns (k + 10, v))
    for (u, w) in owns new_entries:
        pairs.insert(owns u, owns w)
    println(pairs.len())
