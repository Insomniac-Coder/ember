# A21: updating a map value by key, three spellings.
from std.collections import Map

fn count_index(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] += 1
    return counts

fn count_get(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get(w).unwrap_or(0) + 1
    return counts

fn count_entry(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts.entry(w.to_string()).or_default() += 1
    return counts

fn main():
    println(count_get("a b a").len(), count_entry("a b a").len(), count_index("a").len())
