# A15 workaround in Ember: `owned fn` copies the loop's reference in; the counter is lent by reference.
import std.thread
from std.sync import Atomic

fn add_words(total: Atomic[int], t: String):
    total.fetch_add(t.split_whitespace().count())

fn count_words(texts: Array[String]) -> int:
    total = Atomic(0)
    tr = ref total
    with scope = thread.scope():
        for t in texts:
            scope.spawn(owned fn() => add_words(tr, t))
    return total.load()

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
