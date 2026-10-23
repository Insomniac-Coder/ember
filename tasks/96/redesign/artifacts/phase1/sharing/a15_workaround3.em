# A15 workaround in Ember: scoped tasks plus an Atomic, through a helper function.
import std.thread
from std.sync import Atomic

fn add_words(total: Atomic[int], t: String):
    total.fetch_add(t.split_whitespace().count())

fn count_words(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() => add_words(total, t))
    return total.load()

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
