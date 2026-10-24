# A15: scoped tasks append to one list (Python: results.append under the GIL).
import std.thread

fn squares() -> Array[int]:
    results: Array[int] = []
    with scope = thread.scope():
        for i in 0..4:
            scope.spawn(fn(): results.push(i * i))
    return results

fn main():
    println(squares())
