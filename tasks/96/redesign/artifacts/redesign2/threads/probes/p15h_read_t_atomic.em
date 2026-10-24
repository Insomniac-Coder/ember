# A15: the tasks read the loop variable and add to an Atomic borrowed from the parent.
import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total.fetch_add(t.split_whitespace().count()))
    return total.load()

fn main():
    println(count(["a b".to_string()]))
