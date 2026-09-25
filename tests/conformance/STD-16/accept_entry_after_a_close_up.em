#$ test: run-pass
#$ rules: STD-16, STD-11
#$ stdout: {'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6, 'g': 7, 'h': 9}
# `[STD-16]` — `entry(k).or_insert(v)` on a new key returns the inserted
# value even when the insertion first closes up removed entries.

fn main():
    m: Map[String, int] = {"a": 1, "b": 2, "c": 3, "d": 4, "e": 5, "f": 6, "g": 7}
    m.remove("a")
    n = m.entry(String.from("h")).or_insert(8)
    n += 1
    println(m)
