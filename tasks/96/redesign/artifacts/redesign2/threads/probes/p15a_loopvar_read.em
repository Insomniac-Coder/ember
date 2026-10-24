# A15: a scoped task per text, reading only its loop variable (no shared write).
import std.thread

fn print_lengths(texts: Array[String]):
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() => println(t.len()))

fn main():
    print_lengths(["a b".to_string(), "c".to_string()])
