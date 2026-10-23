# A15 workaround in Ember, block-lambda form.
import std.thread
from std.sync import Atomic

fn count_words(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total.fetch_add(t.split_whitespace().count()))
    return total.load()

fn count_one(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        scope.spawn(fn() => total.fetch_add(texts[0].len()))
    return total.load()

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
