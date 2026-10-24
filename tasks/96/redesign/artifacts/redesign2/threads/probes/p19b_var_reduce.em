# A19 proposed: a module variable that scoped tasks add to (a reduction), then read on the main thread.
import std.thread

var WORDS: int = 0

fn count(texts: Array[String]):
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): WORDS += t.split_whitespace().count())

fn main():
    count(["a b".to_string(), "c".to_string()])
    println(WORDS)
