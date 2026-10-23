# A21: two more spellings of the counter update.
from std.collections import Map

fn count_copied(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get(w).copied().unwrap_or(0) + 1
    return counts

fn count_entry_local(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        c = counts.entry(w.to_string()).or_default()
        c += 1
    return counts

fn main():
    println(count_copied("a b a").len(), count_entry_local("a b a").len())
