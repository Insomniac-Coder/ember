#$ test: run-pass
#$ rules: THR-11, THR-5, THR-12, THR-1
#$ profiles: debug, development
#$ stdout: 8 words
#$ 3 tags
#$ 55
#$ [5, 7]
#$ 15
# `[THR-11]` — scoped tasks borrow what outlives the block: Part XI's `Stats` (each task copies the
# `str` it reads, `[CLO-13]`), Part XI.3's `sum_halves`, and two halves of one array written by two
# tasks. `[THR-12]` — two scopes in sequence: the first's join ends its borrows, so the second may
# read what the first wrote.

import std.thread
from std.sync import Atomic, Mutex

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    tags: Mutex[Array[String]] = Mutex([])

fn scan(stats: Stats, text: StrView):
    for w in text.split_whitespace():
        stats.words.fetch_add(1)
        if w.starts_with("#"):
            with tags = stats.tags.lock():
                tags.push(owns w.to_string())

fn sum_halves(xs: Span[int]) -> int:
    left, right = xs.split_at(xs.len() // 2)
    with scope = thread.scope():
        a = scope.spawn(owns fn() => left.iter().sum())
        b = scope.spawn(owns fn() => right.iter().sum())
        return a.join() + b.join()

fn add_to(ref mut s: MutSpan[int], n: int):
    s[0] += n

fn total(s: Span[int]) -> int:
    return s[0] + s[1]

fn main():
    stats = Stats()
    texts = ["#ember is fast", "no tags here", "#python #c"]
    with scope = thread.scope():
        for t in texts:
            scope.spawn(owns fn() => scan(stats, t))
    println(stats.words.load(), "words")
    with tags = stats.tags.lock():
        println(tags.len(), "tags")
    xs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    println(sum_halves(xs))
    counts = [0, 0]
    first, second = counts.as_mut_span().split_at(1)
    with scope = thread.scope():
        scope.spawn(owns fn() => add_to(ref mut first, 5))
        scope.spawn(owns fn() => add_to(ref mut second, 7))
    println(counts)
    with scope = thread.scope():
        h = scope.spawn(owns fn() => total(counts) + 3)
        println(h.join())
