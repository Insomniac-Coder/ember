# Proposed syntax, readability attack. Python's word count from threads: a dict of counts. The reduction
# rule (answers report 4.4) covers `x += e` on a captured place and push/extend/add on lists and sets;
# `counts[w] += 1` writes a different place each time and is not in its list, so two tasks writing the map
# conflict (refused).
import std.thread

fn word_counts(texts: Array[String]) -> Map[String, int]:
    counts: Map[String, int] = {}
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn():
                for w in t.split_whitespace():
                    counts[w.to_string()] = counts.get(w, 0) + 1)
    return counts

fn histogram(data: Array[u8]) -> Array[int]:
    hist = [0] * 256
    @parallel
    for b in data:
        hist[b] += 1
    return hist
