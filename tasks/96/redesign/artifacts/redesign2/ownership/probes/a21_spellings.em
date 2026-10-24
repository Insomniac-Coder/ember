# A21: the Python spellings of a word count, one per function.
from std.collections import Map

fn python_get_default(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get(w, 0) + 1
    return counts

fn ember_get_or(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get_or(w, 0) + 1
    return counts

fn unwrap_or(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get(w).unwrap_or(0) + 1
    return counts

fn entry_or_default(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts.entry(w.to_string()).or_default() += 1
    return counts

fn main():
    println(ember_get_or("a b a").len())
