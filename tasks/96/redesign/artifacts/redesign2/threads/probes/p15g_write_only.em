# A15: the tasks write a shared counter but do not read the loop variable.
import std.thread

fn count(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += 1)
    return total

fn main():
    println(count(["a".to_string()]))
