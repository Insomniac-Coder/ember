# A21: the entry spelling through a local, and the Python `in` spelling.
from std.collections import Map

fn count_entry_local(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        c = counts.entry(w.to_string()).or_default()
        c += 1
    return counts

fn count_in(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        if w in counts:
            counts[w] += 1
        else:
            counts[w] = 1
    return counts

fn main():
    println(count_entry_local("a b a").len(), count_in("a b a").len())
