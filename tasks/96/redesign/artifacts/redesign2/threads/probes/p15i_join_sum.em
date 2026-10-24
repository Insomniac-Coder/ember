# A15: per-task results summed from the join handles (Rust's idiomatic shape).
import std.thread

fn count(texts: Array[String]) -> int:
    with scope = thread.scope():
        a = scope.spawn(fn() => texts[0].split_whitespace().count())
        b = scope.spawn(fn() => texts[1].split_whitespace().count())
        return a.join() + b.join()

fn main():
    println(count(["a b".to_string(), "c".to_string()]))
